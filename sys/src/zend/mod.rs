use std::ffi::{c_char, c_double, c_uchar, c_void};
use std::fmt::{Debug, Formatter};
use std::mem::ManuallyDrop;

use nix::sys::stat::FileStat;

use crate::php_lib;
use crate::zend::execute::{Execute, ExecuteRaw};
use crate::zend::long::{ZendLong, ZendUlong};
use crate::zend::stream::{Stream, StreamRaw};

pub mod execute;
pub mod stream;

pub const IS_UNDEF: u32 = 0;
pub const IS_NULL: u32 = 1;
pub const IS_FALSE: u32 = 2;
pub const IS_TRUE: u32 = 3;
pub const IS_LONG: u32 = 4;
pub const IS_DOUBLE: u32 = 5;
pub const IS_STRING: u32 = 6;
pub const IS_ARRAY: u32 = 7;
pub const IS_OBJECT: u32 = 8;
pub const IS_RESOURCE: u32 = 9;
pub const IS_REFERENCE: u32 = 10;
pub const IS_CONSTANT_AST: u32 = 11; // Constant expressions

// Fake types used only for type hinting.
// These are allowed to overlap with the types below.
pub const IS_CALLABLE: u32 = 12;
pub const IS_ITERABLE: u32 = 13;
pub const IS_VOID: u32 = 14;
pub const IS_STATIC: u32 = 15;
pub const IS_MIXED: u32 = 16;
pub const IS_NEVER: u32 = 17;

// Internal types
#[allow(unused)]
pub(crate) const IS_INDIRECT: u32 = 12;
#[allow(unused)]
pub(crate) const IS_PTR: u32 = 13;
#[allow(unused)]
pub(crate) const IS_ALIAS_PTR: u32 = 14;

pub const HASH_FLAG_CONSISTENCY: u32 = (1 << 0) | (1 << 1);
pub const HASH_FLAG_PACKED: u32 = 1 << 2;
pub const HASH_FLAG_UNINITIALIZED: u32 = 1 << 3;
pub const HASH_FLAG_STATIC_KEYS: u32 = 1 << 4; // long and interned strings
pub const HASH_FLAG_HAS_EMPTY_IND: u32 = 1 << 5;
pub const HASH_FLAG_ALLOW_COW_VIOLATION: u32 = 1 << 6;

#[repr(C)]
#[derive(Debug, PartialEq)]
pub enum ZendResultCode {
    Success = 0,
    Failure = -1,
}

pub type ZendResult = ZendResultCode;
pub type ZendUchar = c_uchar;

#[cfg(feature = "zend_enable_zval_long64")]
mod long {
    pub type ZendLong = i64;
    pub type ZendUlong = u32;
}

#[cfg(not(feature = "zend_enable_zval_long64"))]
mod long {
    pub type ZendLong = i32;
    pub type ZendUlong = u32;
}

#[repr(C)]
pub union ZendRefCountedHTypeInfo {
    pub type_info: u32,
}

impl Debug for ZendRefCountedHTypeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe { write!(f, "Union(type_info: {:?})", &self.type_info) }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct ZendRefCountedH {
    pub ref_count: u32,
    pub u: ZendRefCountedHTypeInfo,
}

#[repr(C)]
#[derive(Debug)]
pub struct ZendRefCounted {
    pub gc: ZendRefCountedH,
}

#[repr(C)]
#[derive(Debug)]
pub struct ZendString {
    pub gc: ZendRefCountedH,
    pub h: ZendUlong,
    pub len: usize,
    pub val: [c_char; 1],
}

#[repr(C)]
#[derive(Debug)]
pub struct ZendBucket {
    pub val: Zval,
    pub h: ZendUlong,
    pub key: *mut ZendString,
}

#[repr(C)]
pub union ZendArrayData {
    pub ar_hash: *mut u32,
    pub ar_data: *mut ZendBucket,
    pub ar_packed: *mut Zval,
}

impl Debug for ZendArrayData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            write!(
                f,
                "Union(ar_hash: {:?}, ar_data: {:?}, ar_packed: {:?})",
                &self.ar_hash, &self.ar_data, &self.ar_packed,
            )
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct ZendArray {
    pub gc: ZendRefCounted,
    pub flags: u32,
    pub n_table_mask: u32,
    pub array_data: ZendArrayData,
    pub n_num_used: u32,
    pub n_num_of_elements: u32,
    pub n_table_size: u32,
}

pub type HashTable = ZendArray;

#[repr(C)]
#[derive(Debug)]
pub struct ZendValueWw {
    w1: u32,
    w2: u32,
}

#[repr(C)]
pub union ZendValue {
    pub lval: ZendLong,
    pub dval: c_double,
    pub counted: *mut ZendRefCounted,
    pub str: *mut ZendString,
    pub arr: *mut ZendArray,
    // pub obj: *mut ZendObject,
    // pub res: *mut ZendResource,
    // pub ref_: *mut ZendReference,
    // pub ast: *mut ZendAstRef,
    pub zv: *mut Zval,
    pub ptr: *mut c_void,
    // pub ce: *mut ZendClassEntry,
    // pub func: *mut ZendFunction,
    pub ww: ManuallyDrop<ZendValueWw>,
}

impl Debug for ZendValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            write!(f, "Union(lval: {:?}, dval: {:?}, counted: {:?}, str: {:?}, arr: {:?}, zv: {:?}, ptr: {:?}, ww: {:?})", &self.lval, &self.dval, &self.counted, &self.str, &self.arr, &self.zv, &self.ptr, &self.ww)
        }
    }
}

#[repr(C)]
pub union ZvalTypeInfoUnion {
    pub type_info: u32,
}

impl Debug for ZvalTypeInfoUnion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe { write!(f, "Union(type_info: {:?})", &self.type_info) }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Zval {
    pub value: ZendValue,
    pub type_info: ZvalTypeInfoUnion,
    pub u2: u32,
}

pub type ZendStat = FileStat;

#[repr(C)]
#[derive(Debug)]
pub struct ZendFunctionEntry {
    fname: *const c_char,
    // handler: ZifHandler,
    // arg_info: *const ZendInternalArgInfo,
    num_args: u32,
    flags: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct ZendLlistElement {
    pub next: *mut ZendLlistElement,
    pub prev: *mut ZendLlistElement,
    pub data: [c_void; 1],
}

type LlistDtorFunc = extern "C" fn(*mut c_void);

#[repr(C)]
#[derive(Debug)]
pub struct ZendLlist {
    pub head: *mut ZendLlistElement,
    pub tail: *mut ZendLlistElement,
    pub count: usize,
    pub size: usize,
    pub dtor: LlistDtorFunc,
    pub persistent: c_uchar,
    pub traverse_ptr: *mut ZendLlistElement,
}

php_lib! {
    pub struct Zend<ZendRaw> {
        pub zend_signal_startup: fn(),
        pub zend_llist_get_first_ex: fn(l: *mut ZendLlist, pos: *mut ZendLlistElement,) -> *mut c_void,
        pub zend_llist_get_next_ex: fn(l: *mut ZendLlist, pos: *mut ZendLlistElement,) -> *mut c_void,
        {
            pub execute: Execute<ExecuteRaw>,
            pub stream: Stream<StreamRaw>,
        }
    }
}
