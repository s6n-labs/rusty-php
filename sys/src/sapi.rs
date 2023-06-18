use std::ffi::{c_char, c_double, c_int, c_uchar, c_uint, c_void};

use libc::{gid_t, uid_t};

use crate::streams::PhpStream;
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
    pub mimetype: *mut c_char,
    pub http_status_line: *mut c_char,
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
pub struct SapiPostEntry {
    pub content_type: *mut c_char,
    pub content_type_len: u32,
    pub post_reader: extern "C" fn(),
    pub post_handler: extern "C" fn(content_type_dup: *mut c_char, arg: *mut c_void),
}

#[repr(C)]
#[derive(Debug)]
pub struct SapiRequestInfo {
    pub request_method: *const c_char,
    pub query_string: *mut c_char,
    pub cookie_data: *mut c_char,
    pub content_length: ZendLong,
    pub path_translated: *mut c_char,
    pub request_uri: *mut c_char,
    pub request_body: *mut PhpStream,
    pub content_type: *const c_char,
    pub headers_only: bool,
    pub no_headers: bool,
    pub headers_read: bool,
    pub post_entry: *mut SapiPostEntry,
    pub content_type_dup: *mut c_char,
    pub auth_user: *mut c_char,
    pub auth_password: *mut c_char,
    pub auth_digest: *mut c_char,
    pub argv0: *mut c_char,
    pub current_user: *mut c_char,
    pub current_user_length: c_int,
    pub argc: c_int,
    pub argv: *mut *mut c_char,
    pub proto_num: c_int,
}

#[repr(C)]
#[derive(Debug)]
pub struct SapiGlobalsStruct {
    pub server_context: *mut c_void,
    pub request_info: SapiRequestInfo,
    pub sapi_headers: SapiHeadersStruct,
    pub read_post_bytes: i64,
    pub post_read: c_uchar,
    pub headers_sent: c_uchar,
    pub global_stat: ZendStat,
    pub default_mimetype: *mut c_char,
    pub default_charset: *mut c_char,
    pub rfc1867_uploaded_files: *mut HashTable,
    pub post_max_size: ZendLong,
    pub options: c_int,
    pub sapi_started: bool,
    pub global_request_time: c_double,
    pub known_post_content_types: HashTable,
    pub callback_func: Zval,
    // TODO: pub zend_fcall_info_cache: ZendFcallInfoCache
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
    pub header_handler: Option<
        extern "C" fn(
            sapi_handler: *mut SapiHeaderStruct,
            op: SapiHeaderOpEnum,
            sapi_headers: *mut SapiHeadersStruct,
        ) -> c_int,
    >,
    pub send_headers: extern "C" fn(sapi_headers: *mut SapiHeadersStruct) -> c_int,
    pub send_header:
        Option<extern "C" fn(sapi_header: *mut SapiHeaderStruct, server_context: *mut c_void)>,
    pub read_post: extern "C" fn(buffer: *mut c_char, count_bytes: usize) -> usize,
    pub read_cookies: extern "C" fn() -> *mut c_char,
    pub register_server_variables: Option<extern "C" fn(track_vars_array: *mut Zval)>,
    pub log_message: extern "C" fn(message: *const c_char, syslog_type_int: c_int),
    pub get_request_time: extern "C" fn(request_time: *mut c_double) -> ZendResult,
    pub terminate_process: extern "C" fn(),
    pub php_ini_path_override: *mut c_char,
    pub default_post_reader: extern "C" fn(),
    pub treat_data: Option<extern "C" fn(arg: c_int, str: *mut c_char, dest_array: *mut Zval)>,
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

#[cfg(feature = "zts")]
extern "C" {
    pub static sapi_globals_id: c_int;
    pub static sapi_globals_offset: usize;
}

#[cfg(not(feature = "zts"))]
extern "C" {
    pub static mut sapi_globals: SapiGlobalsStruct;
}

#[cfg(feature = "zts")]
#[macro_export]
macro_rules! sg {
    ($v: ident) => {
        $crate::zend::zend_tsrmg_fast!(
            $crate::sapi::sapi_globals_offset,
            *mut $crate::sapi::SapiGlobalsStruct,
            $v
        )
    };
}

#[cfg(not(feature = "zts"))]
#[macro_export]
macro_rules! sg {
    ($v: ident) => {
        $crate::sapi::sapi_globals.$v
    };
}

pub use sg;
