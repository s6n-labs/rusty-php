use std::ffi::{c_char, c_double, c_int, c_uchar, c_uint, c_void};

use libc::{gid_t, uid_t};

use crate::zend::*;

pub const SAPI_HEADER_SENT_SUCCESSFULLY: c_int = 1;
pub const SAPI_HEADER_DO_SEND: c_int = 2;
pub const SAPI_HEADER_SEND_FAILED: c_int = 3;

#[repr(C)]
pub struct SapiHeaderStruct {
    header: *mut c_char,
    header_len: usize,
}

#[repr(C)]
pub struct SapiHeadersStruct {
    headers: ZendLlist,
    http_response_code: c_int,
    send_default_content_type: c_uchar,
    mimetype: *mut char,
    http_status_line: *mut char,
}

#[repr(C)]
#[allow(clippy::enum_variant_names)]
pub enum SapiHeaderOpEnum {
    SapiHeaderReplace,
    SapiHeaderAdd,
    SapiHeaderDelete,
    SapiHeaderDeleteAll,
    SapiHeaderSetStatus,
}

#[repr(C)]
pub struct SapiModuleStruct {
    pub(crate) name: *mut c_char,
    pub(crate) pretty_name: *mut c_char,
    pub(crate) startup: extern "C" fn(sapi_module: *mut SapiModuleStruct) -> c_int,
    pub(crate) shutdown: extern "C" fn(sapi_module: *mut SapiModuleStruct) -> c_int,
    pub(crate) activate: extern "C" fn() -> c_int,
    pub(crate) deactivate: extern "C" fn() -> c_int,
    pub(crate) ub_write: extern "C" fn(str: *const c_char, str_length: usize) -> usize,
    pub(crate) flush: extern "C" fn(server_context: *mut c_void),
    pub(crate) get_stat: extern "C" fn() -> *mut ZendStat,
    pub(crate) getenv: extern "C" fn(name: *const c_char, name_len: usize) -> *mut c_char,
    pub(crate) sapi_error: unsafe extern "C" fn(ty: c_int, error_msg: *const c_char, ...),
    pub(crate) header_handler: extern "C" fn(
        sapi_handler: *mut SapiHeaderStruct,
        op: SapiHeaderOpEnum,
        sapi_headers: *mut SapiHeadersStruct,
    ) -> c_int,
    pub(crate) send_headers: extern "C" fn(sapi_headers: *mut SapiHeadersStruct) -> c_int,
    pub(crate) send_header:
        extern "C" fn(sapi_header: *mut SapiHeaderStruct, server_context: *mut c_void),
    pub(crate) read_post: extern "C" fn(buffer: *mut c_char, count_bytes: usize) -> usize,
    pub(crate) read_cookies: extern "C" fn() -> *mut c_char,
    pub(crate) register_server_variables: extern "C" fn(track_vars_array: *mut Zval),
    pub(crate) log_message: extern "C" fn(message: *const c_char, syslog_type_int: c_int),
    pub(crate) get_request_time: extern "C" fn(request_time: *mut c_double) -> ZendResult,
    pub(crate) terminate_process: extern "C" fn(),
    pub(crate) php_ini_path_override: *mut c_char,
    pub(crate) default_post_reader: extern "C" fn(),
    pub(crate) treat_data: extern "C" fn(arg: c_int, str: *mut c_char, dest_array: Zval),
    pub(crate) executable_location: *mut c_char,
    pub(crate) php_ini_ignore: c_int,
    pub(crate) php_ini_ignore_cwd: c_int,
    pub(crate) get_fd: extern "C" fn(fd: *mut c_int) -> c_int,
    pub(crate) force_http_10: extern "C" fn() -> c_int,
    pub(crate) get_target_uid: extern "C" fn(*mut uid_t) -> c_int,
    pub(crate) get_target_gid: extern "C" fn(*mut gid_t) -> c_int,
    pub(crate) input_filter: extern "C" fn(
        arg: c_int,
        var: *const char,
        val: *mut *mut char,
        val_len: usize,
        new_val_len: *mut usize,
    ) -> c_uint,
    pub(crate) ini_defaults: extern "C" fn(configuration_hash: *mut HashTable),
    pub(crate) phpinfo_as_text: c_int,
    pub(crate) ini_entries: *mut c_char,
    pub(crate) additional_functions: *const ZendFunctionEntry,
    pub(crate) input_filter_init: extern "C" fn() -> c_uint,
}
