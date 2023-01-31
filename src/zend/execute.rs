use std::ffi::c_char;

use crate::php_lib;
use crate::zend::{ZendResult, Zval};

php_lib! {
    pub struct Execute<ExecuteRaw> {
        pub zend_eval_string_ex: fn(
            str: *const c_char,
            retval_ptr: *mut Zval,
            string_name: *const c_char,
            handle_exceptions: bool,
        ) -> ZendResult,
    }
}
