#![feature(c_variadic)]

use std::error::Error;
use std::ffi::CString;
use std::io::stderr;
use std::mem::MaybeUninit;
use std::ptr::null_mut;

use clap::{Parser, Subcommand};
use rusty_php::callback::{Callback, SapiCallback};
use rusty_php::sapi::Sapi;
use rusty_php::sys::zend::stream::ZendFileHandle;
use rusty_php::sys::zend::Zval;
use rusty_php::PhpInit;
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

    let php = PhpInit::new(SapiImpl)
        .init()?
        .startup_module()
        .unwrap()
        .startup_request()
        .unwrap();

    php.as_ref().streams._php_stream_open_wrapper_ex(
        create_cstring(b"php://stdin").into_raw(),
        create_cstring(b"rb").into_raw(),
        0,
        null_mut(),
        null_mut(),
    );

    php.as_ref().streams._php_stream_open_wrapper_ex(
        create_cstring(b"php://stdout").into_raw(),
        create_cstring(b"wb").into_raw(),
        0,
        null_mut(),
        null_mut(),
    );

    php.as_ref().streams._php_stream_open_wrapper_ex(
        create_cstring(b"php://stderr").into_raw(),
        create_cstring(b"wb").into_raw(),
        0,
        null_mut(),
        null_mut(),
    );

    let cli = Cli::parse();
    match &cli.action {
        Action::Eval { script } => {
            let mut retval = MaybeUninit::<Zval>::uninit();

            php.as_ref().zend.execute.zend_eval_string_ex(
                create_cstring(script.as_bytes()).into_raw(),
                retval.as_mut_ptr(),
                create_cstring(b"Command line begin code").into_raw(),
                true,
            );
        }
        Action::Execute { filename } => {
            let mut file_handle = MaybeUninit::<ZendFileHandle>::uninit();

            php.as_ref().zend.stream.zend_stream_init_filename(
                file_handle.as_mut_ptr(),
                create_cstring(filename.as_bytes()).into_raw(),
            );

            let mut file_handle = unsafe { file_handle.assume_init() };
            file_handle.primary_script = true;

            php.as_ref().php_execute_script(&mut file_handle);
        }
    };

    php.shutdown().shutdown().shutdown();

    Ok(())
}
