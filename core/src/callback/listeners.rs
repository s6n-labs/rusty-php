use std::ffi::{c_char, c_double, c_int, c_uint, c_void, CStr, CString};
use std::ptr::null_mut;
use std::sync::Arc;

use libc::{gid_t, uid_t};
use map_in_place::MapVecInPlace;
use tracing::debug;

use crate::callback::{SapiCallback, GLOBAL_CALLBACK};
use crate::result::Ok;
use crate::sys::sapi::{SapiHeaderOpEnum, SapiHeaderStruct, SapiHeadersStruct, SapiModuleStruct};
use crate::sys::zend::{HashTable, ZendResult, ZendStat, Zval};

fn callback() -> Arc<dyn SapiCallback> {
    unsafe {
        Arc::clone(
            &GLOBAL_CALLBACK
                .as_ref()
                .expect("SAPI callback must be registered before starting up")
                .listener,
        )
    }
}

pub(crate) extern "C" fn on_startup(_sapi_module: *mut SapiModuleStruct) -> c_int {
    debug!("CALLBACK: on_startup");
    callback().on_startup().into()
}

pub(crate) extern "C" fn on_shutdown(_sapi_module: *mut SapiModuleStruct) -> c_int {
    debug!("CALLBACK: on_shutdown");
    callback().on_shutdown().into()
}

pub(crate) extern "C" fn on_activate() -> c_int {
    debug!("CALLBACK: on_activate");
    callback().on_activate().into()
}

pub(crate) extern "C" fn on_deactivate() -> c_int {
    debug!("CALLBACK: on_deactivate");
    callback().on_deactivate().into()
}

pub(crate) extern "C" fn on_ub_write(str: *const c_char, str_length: usize) -> usize {
    debug!("CALLBACK: on_ub_write");
    callback().on_ub_write(&unsafe { CStr::from_ptr(str) }.to_bytes()[..str_length])
}

pub(crate) extern "C" fn on_flush(_server_context: *mut c_void) {
    debug!("CALLBACK: on_flush");
    callback().on_flush();
}

pub(crate) extern "C" fn on_get_stat() -> *mut ZendStat {
    debug!("CALLBACK: on_get_stat");
    match callback().on_get_stat() {
        Ok(v) => Box::leak(Box::new(v)),
        _ => null_mut(),
    }
}

pub(crate) extern "C" fn on_getenv(name: *const c_char, name_len: usize) -> *mut c_char {
    debug!("CALLBACK: on_getenv");
    match callback().on_get_env(&unsafe { CStr::from_ptr(name) }.to_bytes()[..name_len]) {
        Some(v) => unsafe { CString::from_vec_unchecked(v) }.into_raw(),
        _ => null_mut(),
    }
}

#[allow(clippy::unnecessary_cast)]
pub(crate) unsafe extern "C" fn on_sapi_error(ty: c_int, error_msg: *const c_char, mut _args: ...) {
    debug!("CALLBACK: on_sapi_error");
    callback().on_sapi_error(ty as i32, unsafe { CStr::from_ptr(error_msg) }.to_bytes());
}

pub(crate) extern "C" fn on_header_handler(
    sapi_header: *mut SapiHeaderStruct,
    op: SapiHeaderOpEnum,
    sapi_headers: *mut SapiHeadersStruct,
) -> c_int {
    debug!("CALLBACK: on_header_handler");
    callback()
        .on_header_handler(unsafe { &*sapi_header }, op, unsafe { &mut *sapi_headers })
        .into()
}

pub(crate) extern "C" fn on_send_headers(sapi_headers: *mut SapiHeadersStruct) -> c_int {
    debug!("CALLBACK: on_send_headers");
    callback().on_send_headers(unsafe { &*sapi_headers })
}

pub(crate) extern "C" fn on_send_header(
    sapi_header: *mut SapiHeaderStruct,
    _server_context: *mut c_void,
) {
    debug!("CALLBACK: on_send_header");
    callback().on_send_header(unsafe { &*sapi_header });
}

pub(crate) extern "C" fn on_read_post(buffer: *mut c_char, count_bytes: usize) -> usize {
    debug!("CALLBACK: on_read_post");
    callback()
        .on_read_post(unsafe { std::slice::from_raw_parts_mut(buffer as *mut u8, count_bytes) })
}

pub(crate) extern "C" fn on_read_cookies() -> *mut c_char {
    debug!("CALLBACK: on_read_cookies");
    match callback().on_read_cookies() {
        Some(v) => unsafe { CString::from_vec_unchecked(v) }.into_raw(),
        _ => null_mut(),
    }
}

pub(crate) extern "C" fn on_register_server_variables(track_vars_array: *mut Zval) {
    debug!("CALLBACK: on_register_server_variables");
    callback().on_register_server_variables(unsafe { &mut *track_vars_array });
}

#[allow(clippy::unnecessary_cast)]
pub(crate) extern "C" fn on_log_message(message: *const c_char, syslog_type_int: c_int) {
    debug!("CALLBACK: on_log_message");
    callback().on_log_message(
        unsafe { CStr::from_ptr(message) }.to_bytes(),
        syslog_type_int,
    )
}

pub(crate) extern "C" fn on_get_request_time(request_time: *mut c_double) -> ZendResult {
    debug!("CALLBACK: on_get_request_time");
    callback()
        .on_get_request_time()
        .writing_raw(request_time)
        .into()
}

pub(crate) extern "C" fn on_terminate_process() {
    debug!("CALLBACK: on_terminate_process");
    callback().on_get_request_time();
}

pub(crate) extern "C" fn on_default_post_reader() {
    debug!("CALLBACK: on_default_post_reader");
    callback().on_default_post_reader();
}

pub(crate) extern "C" fn on_treat_data(arg: c_int, str: *mut c_char, dest_array: *mut Zval) {
    debug!("CALLBACK: on_treat_data");
    callback().on_treat_data(arg, unsafe { CStr::from_ptr(str) }.to_bytes(), dest_array);
}

pub(crate) extern "C" fn on_get_fd(fd: *mut c_int) -> c_int {
    debug!("CALLBACK: on_get_fd");
    callback().on_get_fd().writing_raw(fd).into()
}

pub(crate) extern "C" fn on_force_http_10() -> c_int {
    debug!("CALLBACK: on_force_http_10");
    callback().on_force_http_10().into()
}

pub(crate) extern "C" fn on_get_target_uid(uid: *mut uid_t) -> c_int {
    debug!("CALLBACK: on_get_target_uid");
    callback().on_get_target_uid().writing_raw(uid).into()
}

pub(crate) extern "C" fn on_get_target_gid(gid: *mut gid_t) -> c_int {
    debug!("CALLBACK: on_get_target_gid");
    callback().on_get_target_gid().writing_raw(gid).into()
}

#[allow(clippy::unnecessary_cast)]
pub(crate) extern "C" fn on_input_filter(
    arg: c_int,
    var: *const c_char,
    val: *mut *mut c_char,
    val_len: usize,
    new_val_len: *mut usize,
) -> c_uint {
    debug!("CALLBACK: on_input_filter");
    callback()
        .on_input_filter(
            arg as i32,
            unsafe { CStr::from_ptr(var) }.to_bytes(),
            unsafe { Vec::from_raw_parts(val, val_len, val_len) }
                .map_in_place(|p| unsafe { CString::from_raw(p) }.into_bytes().leak())
                .leak(),
        )
        .writing_raw(new_val_len)
        .into()
}

pub(crate) extern "C" fn on_ini_defaults(configuration_hash: *mut HashTable) {
    debug!("CALLBACK: on_ini_defaults");
    callback().on_ini_defaults(unsafe { &mut *configuration_hash });
}

pub(crate) extern "C" fn on_input_filter_init() -> c_uint {
    debug!("CALLBACK: on_input_filter_init");
    callback().on_input_filter_init().into()
}
