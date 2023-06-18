use std::ffi::{c_char, c_int, c_void};

use crate::zend::ZendString;

pub type PhpStream = c_void; // TODO

extern "C" {
    pub fn _php_stream_open_wrapper_ex(
        path: *const c_char,
        mode: *const c_char,
        options: c_int,
        opened_path: *mut *mut ZendString,
        context: *mut c_void,
    ) -> *mut PhpStream;
}
