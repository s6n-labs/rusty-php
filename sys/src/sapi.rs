use std::ffi::{c_char, c_double, c_int, c_uchar, c_uint, c_void};

use libc::{gid_t, uid_t};

use crate::zend::*;

pub const SAPI_HEADER_SENT_SUCCESSFULLY: c_int = 1;
pub const SAPI_HEADER_DO_SEND: c_int = 2;
pub const SAPI_HEADER_SEND_FAILED: c_int = 3;

#[repr(C)]
#[derive(Debug)]
pub struct SapiHeaderStruct {
    pub header: *mut c_char,
    pub header_len: usize,
}

#[repr(C)]
#[derive(Debug)]
pub struct SapiHeadersStruct {
    pub headers: ZendLlist,
    pub http_response_code: c_int,
    pub send_default_content_type: c_uchar,
    pub mimetype: *mut char,
    pub http_status_line: *mut char,
}

#[repr(C)]
#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum SapiHeaderOpEnum {
    SapiHeaderReplace,
    SapiHeaderAdd,
    SapiHeaderDelete,
    SapiHeaderDeleteAll,
    SapiHeaderSetStatus,
}

#[repr(C)]
#[derive(Debug)]
pub struct SapiModuleStruct {
    pub name: *mut c_char,
    pub pretty_name: *mut c_char,
    pub startup: extern "C" fn(sapi_module: *mut SapiModuleStruct) -> c_int,
    pub shutdown: extern "C" fn(sapi_module: *mut SapiModuleStruct) -> c_int,
    pub activate: extern "C" fn() -> c_int,
    pub deactivate: extern "C" fn() -> c_int,
    pub ub_write: extern "C" fn(str: *const c_char, str_length: usize) -> usize,
    pub flush: extern "C" fn(server_context: *mut c_void),
    pub get_stat: extern "C" fn() -> *mut ZendStat,
    pub getenv: extern "C" fn(name: *const c_char, name_len: usize) -> *mut c_char,
    pub sapi_error: unsafe extern "C" fn(ty: c_int, error_msg: *const c_char, ...),
    pub header_handler: extern "C" fn(
        sapi_handler: *mut SapiHeaderStruct,
        op: SapiHeaderOpEnum,
        sapi_headers: *mut SapiHeadersStruct,
    ) -> c_int,
    pub send_headers: extern "C" fn(sapi_headers: *mut SapiHeadersStruct) -> c_int,
    pub send_header: extern "C" fn(sapi_header: *mut SapiHeaderStruct, server_context: *mut c_void),
    pub read_post: extern "C" fn(buffer: *mut c_char, count_bytes: usize) -> usize,
    pub read_cookies: extern "C" fn() -> *mut c_char,
    pub register_server_variables: extern "C" fn(track_vars_array: *mut Zval),
    pub log_message: extern "C" fn(message: *const c_char, syslog_type_int: c_int),
    pub get_request_time: extern "C" fn(request_time: *mut c_double) -> ZendResult,
    pub terminate_process: extern "C" fn(),
    pub php_ini_path_override: *mut c_char,
    pub default_post_reader: extern "C" fn(),
    pub treat_data: extern "C" fn(arg: c_int, str: *mut c_char, dest_array: *mut Zval),
    pub executable_location: *mut c_char,
    pub php_ini_ignore: c_int,
    pub php_ini_ignore_cwd: c_int,
    pub get_fd: extern "C" fn(fd: *mut c_int) -> c_int,
    pub force_http_10: extern "C" fn() -> c_int,
    pub get_target_uid: extern "C" fn(uid: *mut uid_t) -> c_int,
    pub get_target_gid: extern "C" fn(gid: *mut gid_t) -> c_int,
    pub input_filter: extern "C" fn(
        arg: c_int,
        var: *const c_char,
        val: *mut *mut c_char,
        val_len: usize,
        new_val_len: *mut usize,
    ) -> c_uint,
    pub ini_defaults: extern "C" fn(configuration_hash: *mut HashTable),
    pub phpinfo_as_text: c_int,
    pub ini_entries: *mut c_char,
    pub additional_functions: *const ZendFunctionEntry,
    pub input_filter_init: extern "C" fn() -> c_uint,
}
