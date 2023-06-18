#![feature(local_key_cell_methods)]

mod pool;

use std::cell::RefCell;
use std::env::{current_dir, var};
use std::ffi::{c_int, CStr, CString, OsStr};
use std::io::{stderr, Write};
use std::net::SocketAddr;
use std::os::unix::ffi::OsStrExt;
use std::str::FromStr;

use anyhow::Result;
use hyper::body::{Bytes, HttpBody};
use hyper::header::{HeaderName, HeaderValue};
use hyper::http::uri::PathAndQuery;
use hyper::server::conn::Http;
use hyper::service::service_fn;
use hyper::{Body, HeaderMap, Method, Request, Response};
use rusty_php::callback::{Callback, SapiCallback};
use rusty_php::sapi::Sapi;
use rusty_php::sys::sapi::{SapiHeaderStruct, SapiHeadersStruct, SAPI_HEADER_SENT_SUCCESSFULLY};
use rusty_php::sys::variables::php_register_variable_safe;
use rusty_php::sys::zend::Zval;
use rusty_php::zend::llist::ZLlist;
use rusty_php::{PhpInit, PhpModule};
use tokio::net::TcpListener;
use tracing::{debug, error, info};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::EnvFilter;

use crate::pool::Pool;

pub struct RequestInfo {
    method: Method,
    path: Option<PathAndQuery>,
    buf: Bytes,
}

pub struct Session {
    buf: Vec<u8>,
    headers: HeaderMap,
    request: RequestInfo,
}

thread_local! {
    pub static THREAD_SESSION: RefCell<Option<Session>> = RefCell::new(None);
    pub static THREAD_PHP: PhpModule = PhpInit::new(SapiImpl).init().unwrap().startup_module().unwrap();
}

fn setenv(key: &str, value: &str, track_vars_array: &mut Zval) {
    let key = key.as_bytes().to_vec();
    let value = value.as_bytes().to_vec();
    let val_len = value.len();

    unsafe {
        php_register_variable_safe(
            unsafe { CString::from_vec_unchecked(key) }.into_raw(),
            unsafe { CString::from_vec_unchecked(value) }.into_raw(),
            val_len,
            track_vars_array,
        );
    }
}

struct SapiCallbackImpl;

impl SapiCallback for SapiCallbackImpl {
    fn on_ub_write(&self, str: &[u8]) -> usize {
        THREAD_SESSION
            .with_borrow_mut(|v| v.as_mut().and_then(|v| v.buf.write(str).ok()).unwrap_or(0))
    }

    fn on_get_env(&self, name: &[u8]) -> Option<Vec<u8>> {
        debug!("ENV: {}", String::from_utf8_lossy(name));
        var(OsStr::from_bytes(name))
            .ok()
            .map(|v| v.as_bytes().to_vec())
    }

    fn on_send_headers(&self, headers: &SapiHeadersStruct) -> c_int {
        ZLlist::<SapiHeaderStruct>::from(&headers.headers)
            .into_iter()
            .for_each(|h| {
                let value = unsafe { CStr::from_ptr(h.data().header) }.to_string_lossy();

                debug!("HEADER: {value}");

                let parts = value.splitn(2, ':').collect::<Vec<_>>();

                THREAD_SESSION.with_borrow_mut(|r| {
                    if let Some(r) = r.as_mut() {
                        r.headers.insert(
                            HeaderName::from_str(parts[0].trim()).unwrap(),
                            HeaderValue::from_str(parts[1].trim()).unwrap(),
                        );
                    }
                });
            });

        SAPI_HEADER_SENT_SUCCESSFULLY
    }

    fn on_read_post(&self, mut buffer: &mut [u8]) -> usize {
        THREAD_SESSION.with_borrow(|s| {
            buffer
                .write(s.as_ref().unwrap().request.buf.as_ref())
                .unwrap()
        })
    }

    fn on_register_server_variables(&self, track_vars_array: &mut Zval) {
        THREAD_SESSION.with_borrow_mut(|s| {
            info!("{}", s.as_ref().unwrap().request.path.as_ref().unwrap().as_str());
            setenv("SCRIPT_FILENAME", "/Users/siketyan/.local/src/github.com/siketyan/rusty-php/http/examples/symfony/public/index.php", track_vars_array);
            setenv(
                "REQUEST_METHOD",
                s.as_ref().unwrap().request.method.as_str(),
                track_vars_array,
            );
            setenv(
                "REQUEST_URI",
                s.as_ref().unwrap().request.path.as_ref().unwrap().as_str(),
                track_vars_array,
            );
        });
    }
}

struct SapiImpl;

impl Sapi for SapiImpl {
    fn name(&self) -> &[u8] {
        b"rusty-php-http"
    }

    fn pretty_name(&self) -> &[u8] {
        b"rusty-php-http"
    }

    fn executable_location(&self) -> &[u8] {
        b"/opt/homebrew/bin/php"
    }

    fn callback(&self) -> Callback {
        Callback::new(SapiCallbackImpl)
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .compact()
        .without_time()
        .with_target(false)
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::DEBUG.into())
                .from_env_lossy(),
        )
        .with_writer(stderr)
        .init();

    #[cfg(target_family = "unix")]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sigint = signal(SignalKind::interrupt())?;
        let mut sigterm = signal(SignalKind::terminate())?;

        tokio::select!(
            r = run() => {
                match r {
                    Ok(_) => (),
                    Err(e) => error!("{}", e),
                }
            },
            _ = sigint.recv() => {},
            _ = sigterm.recv() => {},
        );

        info!("Gracefully shutting down...");
    }

    #[cfg(not(target_family = "unix"))]
    match run().await {
        Ok(_) => (),
        Err(e) => error!("{}", e),
    }

    Ok(())
}

async fn run() -> Result<()> {
    let mut pool = Pool::new(4);

    pool.spawn_many(3, |stream| async move {
        if let Err(http_err) = Http::new()
            .serve_connection(stream, service_fn(hello))
            .await
        {
            eprintln!("Error while serving HTTP connection: {http_err}");
        }
    });

    let addr: SocketAddr = ([127, 0, 0, 1], 8080).into();
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        pool.send(stream)?;
    }
}

async fn hello(mut req: Request<Body>) -> Result<Response<Body>> {
    let request_body = req
        .body_mut()
        .data()
        .await
        .and_then(|r| r.ok())
        .unwrap_or_default();

    println!("{}", String::from_utf8_lossy(request_body.as_ref()));

    THREAD_SESSION.with(|r| {
        r.replace(Some(Session {
            buf: Vec::new(),
            headers: HeaderMap::new(),
            request: RequestInfo {
                method: req.method().to_owned(),
                path: req.uri().path_and_query().map(|p| p.to_owned()),
                buf: request_body,
            },
        }));
    });

    THREAD_PHP.with(|php| {
        let request = php.startup_request().unwrap();
        let path = current_dir()
            .unwrap()
            .join("http/examples/symfony/public/index.php");

        let mut file_handle = php.init_stream_path(path);
        file_handle.set_primary_script(true);
        request.execute_script(&mut file_handle);

        let session = THREAD_SESSION.with(|s| s.replace(None)).unwrap();
        let mut response = Response::builder();
        for (k, v) in session.headers {
            response = response.header(k.unwrap(), v);
        }

        Ok(response.body(Body::from(session.buf))?)
    })
}
