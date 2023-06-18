#![feature(c_variadic)]
#![feature(pointer_byte_offsets)]

use std::error::Error;
use std::ffi::{c_char, c_int, CString};
use std::io::stderr;
use std::mem::MaybeUninit;
use std::ptr::null_mut;

use clap::{Parser, Subcommand};
use map_in_place::MapVecInPlace;
use rusty_php::callback::{Callback, SapiCallback};
use rusty_php::sapi::Sapi;
use rusty_php::sys::zend::stream::ZendFileHandle;
use rusty_php::sys::zend::Zval;
use rusty_php::PhpInit;
use rusty_php_sys::php_execute_script;
use rusty_php_sys::sapi::sg;
use rusty_php_sys::streams::_php_stream_open_wrapper_ex;
use rusty_php_sys::zend::execute::zend_eval_string_ex;
use rusty_php_sys::zend::stream::zend_stream_init_filename;
use tracing::debug;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

fn create_cstring(bytes: &[u8]) -> CString {
    unsafe { CString::from_vec_unchecked(bytes.to_vec()) }
}

struct SapiCallbackImpl;

impl SapiCallback for SapiCallbackImpl {}

struct SapiImpl;

impl Sapi for SapiImpl {
    fn name(&self) -> &[u8] {
        b"rusty-php"
    }

    fn pretty_name(&self) -> &[u8] {
        b"rusty-php"
    }

    fn executable_location(&self) -> &[u8] {
        b"/opt/homebrew/bin"
    }

    fn callback(&self) -> Callback {
        Callback::new(SapiCallbackImpl)
    }
}

#[derive(Subcommand)]
enum Action {
    Eval { script: String },
    Execute { filename: String },
}

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    action: Action,
}

fn main() -> Result<(), Box<dyn Error>> {
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

    let php = PhpInit::new(SapiImpl).init()?.startup_module().unwrap();

    let mut args = std::env::args()
        .map(|arg| {
            let mut bytes = arg.into_bytes();
            bytes.push(b'\0');
            bytes.map_in_place(|b| b as c_char)
        })
        .collect::<Vec<_>>();

    let mut c_args = args
        .iter_mut()
        .map(|arg| arg.as_mut_ptr())
        .collect::<Vec<_>>();

    unsafe {
        #[cfg(feature = "zts")]
        rusty_php_sys::tsrm::ts_resource(0);

        sg!(sapi_started) = true;

        sg!(request_info).argc = args.len() as c_int;
        sg!(request_info).argv = c_args.as_mut_ptr();
    }

    std::mem::forget(c_args);
    std::mem::forget(args);

    let php = php.startup_request().unwrap();

    unsafe {
        _php_stream_open_wrapper_ex(
            create_cstring(b"php://stdin").into_raw(),
            create_cstring(b"rb").into_raw(),
            0,
            null_mut(),
            null_mut(),
        );

        _php_stream_open_wrapper_ex(
            create_cstring(b"php://stdout").into_raw(),
            create_cstring(b"wb").into_raw(),
            0,
            null_mut(),
            null_mut(),
        );

        _php_stream_open_wrapper_ex(
            create_cstring(b"php://stderr").into_raw(),
            create_cstring(b"wb").into_raw(),
            0,
            null_mut(),
            null_mut(),
        );
    }

    let cli = Cli::parse();
    match &cli.action {
        Action::Eval { script } => {
            let mut retval = MaybeUninit::<Zval>::uninit();

            unsafe {
                zend_eval_string_ex(
                    create_cstring(script.as_bytes()).into_raw(),
                    retval.as_mut_ptr(),
                    create_cstring(b"Command line begin code").into_raw(),
                    true,
                );
            }

            debug!("EVAL: {:?}", unsafe { retval.assume_init() });
        }
        Action::Execute { filename } => {
            let mut file_handle = MaybeUninit::<ZendFileHandle>::uninit();

            unsafe {
                zend_stream_init_filename(
                    file_handle.as_mut_ptr(),
                    create_cstring(filename.as_bytes()).into_raw(),
                );
            }

            let mut file_handle = unsafe { file_handle.assume_init() };
            file_handle.primary_script = true;

            unsafe {
                php_execute_script(&mut file_handle);
            }
        }
    };

    php.shutdown_all();

    Ok(())
}
