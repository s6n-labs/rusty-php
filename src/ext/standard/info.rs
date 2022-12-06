use std::ffi::c_uint;

use crate::php_lib;

pub const PHP_INFO_GENERAL: c_uint = 1 << 0;
pub const PHP_INFO_CREDITS: c_uint = 1 << 1;
pub const PHP_INFO_CONFIGURATION: c_uint = 1 << 2;
pub const PHP_INFO_MODULES: c_uint = 1 << 3;
pub const PHP_INFO_ENVIRONMENT: c_uint = 1 << 4;
pub const PHP_INFO_VARIABLES: c_uint = 1 << 5;
pub const PHP_INFO_LICENSE: c_uint = 1 << 6;
pub const PHP_INFO_ALL: c_uint = 0xFFFFFFFF;

php_lib! {
    pub struct Info<InfoRaw> {
        pub php_print_info: extern "C" fn(flags: c_uint),
    }
}
