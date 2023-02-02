use std::ffi::{c_int, c_void};
use std::fmt::{Debug, Formatter};
use std::mem::ManuallyDrop;
use std::os::fd::RawFd;

use libc::c_char;

use crate::php_lib;
use crate::zend::{ZendString, ZendUchar};

pub type ZendStreamFsizer = extern "C" fn(handle: *mut c_void);
pub type ZendStreamReader = extern "C" fn(handle: *mut c_void, buf: *mut c_char, len: usize);
pub type ZendStreamCloser = extern "C" fn(handle: *mut c_void);

#[repr(C)]
#[derive(Debug)]
pub struct ZendStream {
    handle: *mut c_void,
    isatty: c_int,
    reader: ZendStreamReader,
    fsizer: ZendStreamFsizer,
    closer: ZendStreamCloser,
}

#[repr(C)]
pub union ZendFileHandleUnion {
    pub fp: RawFd,
    pub stream: ManuallyDrop<ZendStream>,
}

impl Debug for ZendFileHandleUnion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe { write!(f, "Union(fp: {:?}, stream: {:?})", &self.fp, &self.stream) }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct ZendFileHandle {
    pub handle: ZendFileHandleUnion,
    pub filename: *mut ZendString,
    pub opened_path: *mut ZendString,
    pub ty: ZendUchar,
    pub primary_script: bool,
    pub in_list: bool,
    pub buf: *mut c_char,
    pub len: usize,
}

php_lib! {
    pub struct Stream<StreamRaw> {
        pub zend_stream_init_fp: fn(handle: *mut ZendFileHandle, fp: RawFd, filename: *const c_char,),
        pub zend_stream_init_filename: fn(handle: *mut ZendFileHandle, filename: *const c_char,),
    }
}
