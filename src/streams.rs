use std::ffi::{c_char, c_int, c_void};

use crate::php_lib;
use crate::zend::ZendString;

pub type PhpStream = c_void; // TODO

php_lib! {
    pub struct Streams<StreamsRaw> {
        pub _php_stream_open_wrapper_ex: extern "C" fn(
            path: *const c_char,
            mode: *const c_char,
            options: c_int,
            opened_path: *mut *mut ZendString,
            context: *mut c_void,
        ) -> *mut PhpStream,
    }
}
