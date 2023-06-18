use std::ffi::c_char;

use crate::zend::{ZendResult, Zval};

extern "C" {
    pub fn zend_eval_string_ex(
        str: *const c_char,
        retval_ptr: *mut Zval,
        string_name: *const c_char,
        handle_exceptions: bool,
    ) -> ZendResult;
}
