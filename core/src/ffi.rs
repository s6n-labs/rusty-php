use std::ffi::{c_char, CStr};

pub fn char_array_as_bytes<'a>(array: &[c_char], len: usize) -> &'a [u8] {
    &(unsafe { CStr::from_ptr(array.as_ptr()).to_bytes() })[..len]
}
