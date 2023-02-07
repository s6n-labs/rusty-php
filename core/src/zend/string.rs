use rusty_php_sys::zend::ZendString;

use crate::ffi::char_array_as_bytes;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ZStr<'a> {
    buf: &'a [u8],
}

impl<'a> From<&'a [u8]> for ZStr<'a> {
    fn from(value: &'a [u8]) -> Self {
        Self { buf: value }
    }
}

impl<'a> From<&'a str> for ZStr<'a> {
    fn from(value: &'a str) -> Self {
        Self::from(value.as_bytes())
    }
}

impl<'a> From<&'a ZendString> for ZStr<'a> {
    fn from(value: &'a ZendString) -> Self {
        Self {
            buf: char_array_as_bytes(&value.val, value.len),
        }
    }
}

impl<'a> ToString for ZStr<'a> {
    fn to_string(&self) -> String {
        unsafe { String::from_utf8_unchecked(self.buf.to_vec()) }
    }
}
