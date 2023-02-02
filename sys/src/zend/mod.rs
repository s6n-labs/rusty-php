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
    pub val: *mut [c_char; 1],
}

#[repr(C)]
#[derive(Debug)]
pub struct ZendBucket {
    pub val: Zval,
    pub h: ZendUlong,
    pub key: *mut ZendString,
}

#[repr(C)]
#[derive(Debug)]
pub struct ZendArray {
    pub gc: ZendRefCounted,
    pub n_table_mask: u32,
    pub array_data: *mut ZendBucket,
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
    // pub func: *mut ZendFuntion,
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
    type_info: u32,
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
    next: *mut ZendLlistElement,
    prev: *mut ZendLlistElement,
    data: *mut c_void,
}

type LlistDtorFunc = extern "C" fn(*mut c_void);

#[repr(C)]
#[derive(Debug)]
pub struct ZendLlist {
    head: *mut ZendLlistElement,
    tail: *mut ZendLlistElement,
    count: usize,
    size: usize,
    dtor: LlistDtorFunc,
    persistent: c_uchar,
    traverse_ptr: *mut ZendLlistElement,
}

php_lib! {
    pub struct Zend<ZendRaw> {
        pub zend_signal_startup: fn(),
        {
            pub execute: Execute<ExecuteRaw>,
            pub stream: Stream<StreamRaw>,
        }
    }
}
