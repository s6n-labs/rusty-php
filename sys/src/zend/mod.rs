use std::ffi::{c_char, c_double, c_uchar, c_void};
use std::fmt::{Debug, Formatter};
use std::mem::ManuallyDrop;

use libc::c_int;
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

pub struct ZendType {
    ptr: *mut c_void,
    type_mask: u32,
}

pub struct ZendArgInfo {
    name: *mut ZendString,
    type_: ZendType,
    default_value: *mut ZendString,
}

#[repr(C)]
#[derive(Debug)]
pub struct ZendFunctionCommon {
    type_: ZendUchar,          // never used
    arg_flags: [ZendUchar; 3], // bitset of arg_info.pass_by_reference
    fn_flags: u32,
    function_name: *mut ZendString,
    scope: *mut ZendClassEntry,
    prototype: *mut ZendFunction,
    num_args: u32,
    required_num_args: u32,
    arg_info: *mut ZendArgInfo, // index -1 represents the return value info, if any
    attributes: *mut HashTable,
    t: u32, // number of temporary variables
    run_time_cache: *mut *mut c_void,
}

#[repr(C)]
pub union ZendFunction {
    type_: ZendUchar,
    quick_arg_flags: u32,
    common: ManuallyDrop<ZendFunctionCommon>,
    // op_array: ZendOpArray,
    // internal_function: ZendInternalFunction,
}

impl Debug for ZendFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            write!(
                f,
                "Union(type: {:?}, quick_arg_flags: {:?}, common: {:?})",
                &self.type_, &self.quick_arg_flags, &self.common,
            )
        }
    }
}

#[repr(C)]
pub union ZendClassEntryParent {
    parent: *mut ZendClassEntry,
    parent_name: *mut ZendString,
}

impl Debug for ZendClassEntryParent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            write!(
                f,
                "Union(parent: {:?}, parent_name: {:?})",
                &self.parent, &self.parent_name,
            )
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct ZendClassEntry {
    type_: c_char,
    name: *mut ZendString,
    parent: ZendClassEntryParent,
    ref_count: c_int,
    ce_flags: u32,
    default_properties_count: c_int,
    default_static_members_count: c_int,
    default_properties_table: *mut Zval,
    default_static_members_table: *mut Zval,
    static_members_table: *mut Zval,
    function_table: *mut HashTable,
    properties_info: *mut HashTable,
    constants_table: *mut HashTable,
    // mutable_data: *mut ZendClassMutableData,
    // inheritance_cache: *mut ZendInheritanceCacheEntry,
    // properties_info_table: *mut *mut ZendPropertyInfo,
}

#[repr(C)]
#[derive(Debug)]
pub struct ZendObjectHandlers {
    offset: c_int,
    // TODO
}

#[repr(C)]
#[derive(Debug)]
pub struct ZendObject {
    pub gc: ZendRefCountedH,
    pub handle: u32, // may be removed ???
    pub ce: *mut ZendClassEntry,
    pub handlers: *const ZendObjectHandlers,
    pub properties: *mut HashTable,
    pub properties_table: [Zval; 1],
}

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
    pub obj: *mut ZendObject,
    // pub res: *mut ZendResource,
    // pub ref_: *mut ZendReference,
    // pub ast: *mut ZendAstRef,
    pub zv: *mut Zval,
    pub ptr: *mut c_void,
    pub ce: *mut ZendClassEntry,
    pub func: *mut ZendFunction,
    pub ww: ManuallyDrop<ZendValueWw>,
}

impl Debug for ZendValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            write!(
                f,
                "Union(lval: {:?}, dval: {:?}, counted: {:?}, str: {:?}, arr: {:?}, obj: {:?}, zv: {:?}, ptr: {:?}, ce: {:?}, func: {:?}, ww: {:?})",
                &self.lval, &self.dval, &self.counted, &self.str, &self.arr, &self.obj, &self.zv, &self.ptr, &self.ce, &self.func, &self.ww,
            )
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
