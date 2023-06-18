#![allow(unused)]

use std::ffi::c_uint;

pub const PHP_INFO_GENERAL: c_uint = 1 << 0;
pub const PHP_INFO_CREDITS: c_uint = 1 << 1;
pub const PHP_INFO_CONFIGURATION: c_uint = 1 << 2;
pub const PHP_INFO_MODULES: c_uint = 1 << 3;
pub const PHP_INFO_ENVIRONMENT: c_uint = 1 << 4;
pub const PHP_INFO_VARIABLES: c_uint = 1 << 5;
pub const PHP_INFO_LICENSE: c_uint = 1 << 6;
pub const PHP_INFO_ALL: c_uint = 0xFFFFFFFF;

extern "C" {
    pub fn php_print_info(flags: c_uint);
}
