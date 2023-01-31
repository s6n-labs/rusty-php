#![feature(c_variadic)]

use std::error::Error;
use std::ffi::{c_char, c_double, c_int, c_uint, c_void, CStr, CString};
use std::io::{stderr, stdout, Write};
use std::mem::MaybeUninit;
use std::ptr::null_mut;

use clap::{Parser, Subcommand};
use lazy_static::lazy_static;
use libc::{gid_t, uid_t};
use rusty_php::sapi::{
    SapiHeaderOpEnum, SapiHeaderStruct, SapiHeadersStruct, SapiModuleStruct,
    SAPI_HEADER_SENT_SUCCESSFULLY,
};
use rusty_php::zend::stream::ZendFileHandle;
use rusty_php::zend::{HashTable, ZendResult, ZendResultCode, ZendStat, Zval};
use rusty_php::Php;
use tracing::level_filters::LevelFilter;
use tracing::{debug, error, warn};
use tracing_subscriber::EnvFilter;

const NAME: &[u8] = b"rusty-php";
const PRETTY_NAME: &[u8] = b"rusty-php";
const PHP_PATH: &[u8] = b"/opt/homebrew/bin/php";

lazy_static! {
    static ref PHP: Php = Php::load().unwrap();
}

extern "C" fn on_startup(_sapi_module: *mut SapiModuleStruct) -> c_int {
    debug!("CALLBACK: on_startup");
    0
}

extern "C" fn on_shutdown(_sapi_module: *mut SapiModuleStruct) -> c_int {
    debug!("CALLBACK: on_shutdown");
    0
}

extern "C" fn on_activate() -> c_int {
    debug!("CALLBACK: on_activate");
    0
}

extern "C" fn on_deactivate() -> c_int {
    debug!("CALLBACK: on_deactivate");
    0
}

extern "C" fn on_ub_write(str: *const c_char, str_length: usize) -> usize {
    debug!("CALLBACK: on_ub_write");

    let bytes = stdout()
        .write(unsafe { CStr::from_ptr(str) }.to_bytes())
        .unwrap();

    match bytes == str_length {
        true => debug!("WROTE: {} bytes", bytes),
        _ => warn!("WROTE: {} bytes (!= {} bytes)", bytes, str_length),
    }

    bytes
}

extern "C" fn on_flush(_server_context: *mut c_void) {
    debug!("CALLBACK: on_flush");
    stdout().flush().unwrap();
}

extern "C" fn on_get_stat() -> *mut ZendStat {
    debug!("CALLBACK: on_get_stat");
    null_mut()
}

extern "C" fn on_getenv(_name: *const c_char, _name_len: usize) -> *mut c_char {
    debug!("CALLBACK: on_getenv");
    null_mut()
}

unsafe extern "C" fn on_sapi_error(ty: c_int, error_msg: *const c_char, mut _args: ...) {
    debug!("CALLBACK: on_sapi_error");
    error!(
        "ERROR: [{}] {}",
        ty,
        CStr::from_ptr(error_msg).to_string_lossy(),
    )
}

extern "C" fn on_header_handler(
    _sapi_handler: *mut SapiHeaderStruct,
    _op: SapiHeaderOpEnum,
    _sapi_headers: *mut SapiHeadersStruct,
) -> c_int {
    debug!("CALLBACK: on_header_handler");
    0
}

extern "C" fn on_send_headers(_sapi_headers: *mut SapiHeadersStruct) -> c_int {
    debug!("CALLBACK: on_send_headers");
    SAPI_HEADER_SENT_SUCCESSFULLY
}

extern "C" fn on_send_header(_sapi_header: *mut SapiHeaderStruct, _server_context: *mut c_void) {
    debug!("CALLBACK: on_send_header");
}

extern "C" fn on_read_post(_buffer: *mut c_char, _count_bytes: usize) -> usize {
    debug!("CALLBACK: on_read_post");
    0
}

extern "C" fn on_read_cookies() -> *mut c_char {
    debug!("CALLBACK: on_read_cookies");
    null_mut()
}

extern "C" fn on_register_server_variables(_track_vars_array: *mut Zval) {
    debug!("CALLBACK: on_register_server_variables");
}

extern "C" fn on_log_message(message: *const c_char, syslog_type_int: c_int) {
    debug!("CALLBACK: on_log_message");
    debug!(
        "LOG: [{}] {}",
        syslog_type_int,
        unsafe { CStr::from_ptr(message) }.to_string_lossy()
    )
}

extern "C" fn on_get_request_time(_request_time: *mut c_double) -> ZendResult {
    debug!("CALLBACK: on_get_request_time");
    ZendResultCode::Success
}

extern "C" fn on_terminate_process() {
    debug!("CALLBACK: on_terminate_process");
    std::process::exit(1);
}

extern "C" fn on_default_post_reader() {
    debug!("CALLBACK: on_default_post_reader");
}

extern "C" fn on_treat_data(_arg: c_int, _str: *mut c_char, _dest_array: Zval) {
    debug!("CALLBACK: on_treat_data");
}

extern "C" fn on_get_fd(_fd: *mut c_int) -> c_int {
    debug!("CALLBACK: on_get_fd");
    0
}

extern "C" fn on_force_http_10() -> c_int {
    debug!("CALLBACK: on_force_http_10");
    0
}

extern "C" fn on_get_target_uid(_uid: *mut uid_t) -> c_int {
    debug!("CALLBACK: on_get_target_uid");
    0
}

extern "C" fn on_get_target_gid(_gid: *mut gid_t) -> c_int {
    debug!("CALLBACK: on_get_target_gid");
    0
}

extern "C" fn on_input_filter(
    _arg: c_int,
    _var: *const char,
    _val: *mut *mut char,
    _val_len: usize,
    _new_val_len: *mut usize,
) -> c_uint {
    debug!("CALLBACK: on_input_filter");
    0
}

extern "C" fn on_ini_defaults(_configuration_hash: *mut HashTable) {
    debug!("CALLBACK: on_ini_defaults");
}

extern "C" fn on_input_filter_init() -> c_uint {
    debug!("CALLBACK: on_input_filter_init");
    0
}

fn create_cstring(bytes: &[u8]) -> CString {
    unsafe { CString::from_vec_unchecked(bytes.to_vec()) }
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

    let cli = Cli::parse();

    let mut module = SapiModuleStruct {
        name: create_cstring(NAME).into_raw(),
        pretty_name: create_cstring(PRETTY_NAME).into_raw(),
        startup: on_startup,
        shutdown: on_shutdown,
        activate: on_activate,
        deactivate: on_deactivate,
        ub_write: on_ub_write,
        flush: on_flush,
        get_stat: on_get_stat,
        getenv: on_getenv,
        sapi_error: on_sapi_error,
        header_handler: on_header_handler,
        send_headers: on_send_headers,
        send_header: on_send_header,
        read_post: on_read_post,
        read_cookies: on_read_cookies,
        register_server_variables: on_register_server_variables,
        log_message: on_log_message,
        get_request_time: on_get_request_time,
        terminate_process: on_terminate_process,
        php_ini_path_override: null_mut(),
        default_post_reader: on_default_post_reader,
        treat_data: on_treat_data,
        executable_location: create_cstring(PHP_PATH).into_raw(),
        php_ini_ignore: 0,
        php_ini_ignore_cwd: 0,
        get_fd: on_get_fd,
        force_http_10: on_force_http_10,
        get_target_uid: on_get_target_uid,
        get_target_gid: on_get_target_gid,
        input_filter: on_input_filter,
        ini_defaults: on_ini_defaults,
        phpinfo_as_text: 1,
        ini_entries: null_mut(),
        additional_functions: null_mut(),
        input_filter_init: on_input_filter_init,
    };

    PHP.zend.zend_signal_startup();
    PHP.sapi_startup(&mut module as *mut SapiModuleStruct);

    match PHP.php_module_startup(&mut module as *mut SapiModuleStruct, null_mut()) {
        ZendResult::Success => debug!("OK: php_module_startup"),
        ZendResult::Failure => error!("NG: php_module_startup"),
    }

    match PHP.php_request_startup() {
        ZendResult::Success => debug!("OK: php_request_startup"),
        ZendResult::Failure => error!("NG: php_request_startup"),
    }

    // (PHP.ext.standard.info.php_print_info)(PHP_INFO_GENERAL);
    // debug!("OK: php_print_info");

    PHP.streams._php_stream_open_wrapper_ex(
        create_cstring(b"php://stdin").into_raw(),
        create_cstring(b"rb").into_raw(),
        0,
        null_mut(),
        null_mut(),
    );

    PHP.streams._php_stream_open_wrapper_ex(
        create_cstring(b"php://stdout").into_raw(),
        create_cstring(b"wb").into_raw(),
        0,
        null_mut(),
        null_mut(),
    );

    PHP.streams._php_stream_open_wrapper_ex(
        create_cstring(b"php://stderr").into_raw(),
        create_cstring(b"wb").into_raw(),
        0,
        null_mut(),
        null_mut(),
    );

    match &cli.action {
        Action::Eval { script } => {
            let mut retval = MaybeUninit::<Zval>::uninit();

            PHP.zend.execute.zend_eval_string_ex(
                create_cstring(script.as_bytes()).into_raw(),
                retval.as_mut_ptr(),
                create_cstring(b"Command line begin code").into_raw(),
                true,
            );
        }
        Action::Execute { filename } => {
            let mut file_handle = MaybeUninit::<ZendFileHandle>::uninit();

            PHP.zend.stream.zend_stream_init_filename(
                file_handle.as_mut_ptr(),
                create_cstring(filename.as_bytes()).into_raw(),
            );

            let mut file_handle = unsafe { file_handle.assume_init() };
            file_handle.primary_script = true;

            PHP.php_execute_script(&mut file_handle as *mut ZendFileHandle);
        }
    };

    PHP.php_module_shutdown();
    PHP.sapi_shutdown();

    Ok(())
}
