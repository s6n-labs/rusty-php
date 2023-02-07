//! High-level API for reading and writing Zend values.

use std::ffi::CStr;

use rusty_php_sys::zend::{Zval, IS_DOUBLE, IS_LONG, IS_STRING};

#[derive(Debug, PartialEq)]
pub struct ZSlice {}

#[derive(Debug, PartialEq)]
pub enum Value<'a> {
    Long(i64),
    Double(f64),
    String(&'a [u8]),
    Array(ZSlice),
    // TODO: Object
    // TODO: Resource
    // TODO: Reference
    // TODO: AstRef
    Value(Box<Value<'a>>),
    // TODO: ClassEntry
    // TODO: Function
}

impl<'a> From<Zval> for Value<'a> {
    fn from(value: Zval) -> Self {
        #[allow(clippy::unnecessary_cast)]
        match unsafe { value.type_info.type_info } {
            IS_LONG => Self::Long(unsafe { value.value.lval } as i64),
            IS_DOUBLE => Self::Double(unsafe { value.value.dval } as f64),
            IS_STRING => Self::String(unsafe {
                &CStr::from_ptr((*value.value.str).val.as_ptr()).to_bytes()
                    [..(*value.value.str).len]
            }),
            _ => unimplemented!(),
        }
    }
}
