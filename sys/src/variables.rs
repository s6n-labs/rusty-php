use std::ffi::c_char;

use crate::zend::Zval;

extern "C" {
    pub fn php_register_variable_safe(
        var: *const c_char,
        val: *const c_char,
        val_len: usize,
        track_vars_array: *mut Zval,
    );
}
