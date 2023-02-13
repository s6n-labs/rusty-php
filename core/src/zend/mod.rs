//! High-level API for reading and writing Zend values.

pub mod array;
pub mod llist;
pub mod string;

use rusty_php_sys::zend::{Zval, IS_ARRAY, IS_DOUBLE, IS_LONG, IS_STRING};

use crate::zend::array::ZArray;
use crate::zend::string::ZStr;

#[derive(Debug, PartialEq)]
pub enum Value<'a> {
    Long(i64),
    Double(f64),
    String(ZStr<'a>),
    Array(ZArray<'a>),
    // TODO: Object
    // TODO: Resource
    // TODO: Reference
    // TODO: AstRef
    Value(Box<Value<'a>>),
    // TODO: ClassEntry
    // TODO: Function
}

impl<'a> From<&Zval> for Value<'a> {
    fn from(value: &Zval) -> Self {
        let union = &value.value;

        #[allow(clippy::unnecessary_cast)]
        match unsafe { value.type_info.type_info } & 0xf {
            IS_LONG => Self::Long(unsafe { union.lval } as i64),
            IS_DOUBLE => Self::Double(unsafe { union.dval } as f64),
            IS_STRING => Self::String(unsafe { &*union.str }.into()),
            IS_ARRAY => Self::Array(unsafe { &*union.arr }.into()),
            _ => unimplemented!(),
        }
    }
}

impl<'a> From<Zval> for Value<'a> {
    fn from(value: Zval) -> Self {
        Self::from(&value)
    }
}
