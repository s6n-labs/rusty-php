use std::ffi::CString;
use std::ptr::null_mut;

use crate::callback::listeners::*;
use crate::callback::{register_global_callback, Callback};
use crate::sys::sapi::SapiModuleStruct;

pub(crate) fn create_cstring(bytes: &[u8]) -> CString {
    unsafe { CString::from_vec_unchecked(bytes.to_vec()) }
}

pub trait Sapi {
    fn name(&self) -> &[u8];
    fn pretty_name(&self) -> &[u8];
    fn executable_location(&self) -> &[u8];
    fn callback(&self) -> Callback;
}

pub trait SapiExt {
    fn register(&self);
    fn to_raw(&self) -> SapiModuleStruct;
}

impl<T> SapiExt for T
where
    T: Sapi,
{
    fn register(&self) {
        register_global_callback(self.callback());
    }

    fn to_raw(&self) -> SapiModuleStruct {
        SapiModuleStruct {
            name: create_cstring(self.name()).into_raw(),
            pretty_name: create_cstring(self.pretty_name()).into_raw(),
            startup: on_startup,
            shutdown: on_shutdown,
            activate: on_activate,
            deactivate: on_deactivate,
            ub_write: on_ub_write,
            flush: on_flush,
            get_stat: on_get_stat,
            getenv: None,
            sapi_error: on_sapi_error,
            header_handler: None,
            send_headers: on_send_headers,
            send_header: on_send_header,
            read_post: on_read_post,
            read_cookies: on_read_cookies,
            register_server_variables: on_register_server_variables,
            log_message: on_log_message,
            get_request_time: None,
            terminate_process: on_terminate_process,
            php_ini_path_override: null_mut(),
            default_post_reader: on_default_post_reader,
            treat_data: on_treat_data,
            executable_location: create_cstring(self.executable_location()).into_raw(),
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
        }
    }
}
