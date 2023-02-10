use std::env::current_dir;
use std::ffi::{c_int, CStr, CString};
use std::io::{stderr, Write};
use std::mem::MaybeUninit;
use std::ptr::null_mut;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

use axum::body::Body;
use axum::http::header::HeaderName;
use axum::http::uri::PathAndQuery;
use axum::http::{HeaderMap, HeaderValue, Method, Request};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use rusty_php::callback::{Callback, SapiCallback};
use rusty_php::sapi::Sapi;
use rusty_php::sys::sapi::{
    SapiHeaderOpEnum, SapiHeaderStruct, SapiHeadersStruct, SAPI_HEADER_SENT_SUCCESSFULLY,
};
use rusty_php::sys::zend::stream::ZendFileHandle;
use rusty_php::sys::zend::Zval;
use rusty_php::{php, PhpInit, Result};
use tracing::debug;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::EnvFilter;

fn setenv(key: &str, value: &str, track_vars_array: &mut Zval) {
    let key = key.as_bytes().to_vec();
    let value = value.as_bytes().to_vec();
    let val_len = value.len();

    php().unwrap().variables.php_register_variable_safe(
        unsafe { CString::from_vec_unchecked(key) }.into_raw(),
        unsafe { CString::from_vec_unchecked(value) }.into_raw(),
        val_len,
        track_vars_array,
    );
}

struct SapiCallbackImpl {
    session: HttpSession,
}

impl SapiCallback for SapiCallbackImpl {
    fn on_ub_write(&self, str: &[u8]) -> usize {
        self.session
            .response
            .write()
            .unwrap()
            .body
            .write(str)
            .unwrap()
    }

    fn on_header_handler(
        &self,
        header: &SapiHeaderStruct,
        op: SapiHeaderOpEnum,
        headers: &mut SapiHeadersStruct,
    ) -> Result<()> {
        println!("{header:?}, {op:?}, {headers:?}");
        println!(
            "{:?}",
            unsafe { CStr::from_ptr(header.header) }.to_string_lossy()
        );
        Result::Ok(())
    }

    fn on_send_headers(&self, headers: &mut SapiHeadersStruct) -> c_int {
        let php = php().unwrap();
        let mut cursor = headers.headers.head;

        while !cursor.is_null() {
            let element = unsafe { &*cursor };
            let header = match headers.headers.traverse_ptr.is_null() {
                true => php
                    .zend
                    .zend_llist_get_first_ex(&mut headers.headers, null_mut()),
                _ => php
                    .zend
                    .zend_llist_get_next_ex(&mut headers.headers, null_mut()),
            } as *mut SapiHeaderStruct;

            let header = unsafe { &*header };
            let value = unsafe { CStr::from_ptr(header.header) }.to_string_lossy();

            debug!("HEADER: {value}");

            let parts = value.splitn(2, ':').collect::<Vec<_>>();

            self.session.response.write().unwrap().headers.insert(
                HeaderName::from_str(parts[0].trim()).unwrap(),
                HeaderValue::from_str(parts[1].trim()).unwrap(),
            );

            cursor = element.next;
        }

        SAPI_HEADER_SENT_SUCCESSFULLY
    }

    fn on_register_server_variables(&self, track_vars_array: &mut Zval) {
        setenv("SCRIPT_FILENAME", "/Users/siketyan/.local/src/github.com/siketyan/rusty-php/http/examples/symfony/public/index.php", track_vars_array);
        setenv(
            "REQUEST_METHOD",
            self.session.request.method.as_str(),
            track_vars_array,
        );
        setenv(
            "REQUEST_URI",
            self.session.request.path.as_str(),
            track_vars_array,
        );
    }
}

struct HttpRequest {
    method: Method,
    path: PathAndQuery,
}

#[derive(Default)]
struct HttpResponse {
    headers: HeaderMap,
    body: Vec<u8>,
}

struct HttpSession {
    request: HttpRequest,
    response: RwLock<HttpResponse>,
}

struct SapiImpl {
    callback: Arc<SapiCallbackImpl>,
}

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
        Callback::new(Arc::clone(&self.callback))
    }
}

#[tokio::main]
async fn main() {
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

    let router = Router::new().route(
        "/",
        get(move |request: Request<Body>| {
            let request_info = HttpSession {
                request: HttpRequest {
                    method: request.method().to_owned(),
                    path: request.uri().path_and_query().unwrap().to_owned(),
                },
                response: Default::default(),
            };

            let sapi = SapiImpl {
                callback: Arc::new(SapiCallbackImpl {
                    session: request_info,
                }),
            };

            let php = PhpInit::<SapiImpl>::new(&sapi)
                .init()
                .unwrap()
                .startup_module()
                .unwrap()
                .startup_request()
                .unwrap();

            let mut file_handle = MaybeUninit::<ZendFileHandle>::uninit();

            php.as_ref().zend.stream.zend_stream_init_filename(
                file_handle.as_mut_ptr(),
                unsafe {
                    CString::from_vec_unchecked(
                        current_dir()
                            .unwrap()
                            .join("./examples/symfony/public/index.php")
                            .to_string_lossy()
                            .as_bytes()
                            .to_vec(),
                    )
                }
                .into_raw(),
            );

            let mut file_handle = unsafe { file_handle.assume_init() };
            file_handle.primary_script = true;

            php.as_ref().php_execute_script(&mut file_handle);
            php.shutdown_all();

            async move {
                Response::new(Body::from(
                    sapi.callback.session.response.read().unwrap().body.to_vec(),
                ))
            }
        }),
    );

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap();
}
