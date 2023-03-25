#![feature(c_variadic)]
#![allow(improper_ctypes)]

use std::ffi::c_void;

use crate::sapi::SapiModuleStruct;
use crate::zend::stream::ZendFileHandle;
use crate::zend::ZendResult;

pub mod ext;
pub mod sapi;
pub mod streams;
pub mod zend;

extern "C" {
    pub fn php_module_startup(
        sf: *mut SapiModuleStruct,
        additional_module: *mut c_void,
    ) -> ZendResult;
    pub fn php_module_shutdown() -> ZendResult;
    pub fn php_request_startup() -> ZendResult;
    pub fn php_request_shutdown(dummy: *mut c_void) -> ZendResult;
    pub fn php_execute_script(primary_file: *mut ZendFileHandle);
    pub fn sapi_startup(sf: *mut SapiModuleStruct);
    pub fn sapi_shutdown();
}
