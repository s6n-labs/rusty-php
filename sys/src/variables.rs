use std::ffi::c_char;

use crate::php_lib;
use crate::zend::Zval;

php_lib! {
    pub struct Variables<VariablesRaw> {
        pub php_register_variable_safe: fn(
            var: *const c_char,
            val: *const c_char,
            val_len: usize,
            track_vars_array: *mut Zval,
        ),
    }
}
