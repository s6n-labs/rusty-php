use std::ffi::c_int;
pub use std::ffi::c_void;
use std::ptr::null_mut;

pub type TsRsrcId = c_int;

extern "C" {
    pub fn ts_resource_ex(id: TsRsrcId, th_id: *mut c_void); // TODO: THREAD_T

    pub fn tsrm_get_ls_cache() -> *mut c_void;
}

#[inline]
pub unsafe fn ts_resource(id: TsRsrcId) {
    ts_resource_ex(id, null_mut())
}

#[macro_export]
macro_rules! tsrmg_fast_bulk {
    ($offset: expr, $ty: ty) => {
        ($crate::tsrm::tsrm_get_ls_cache() as *mut ::std::ffi::c_char).offset($offset as isize)
            as $ty
    };
}

#[macro_export]
macro_rules! tsrmg_fast {
    ($offset: expr, $ty: ty, $element: ident) => {
        (*($crate::tsrm::tsrmg_fast_bulk!($offset, $ty))).$element
    };
}

#[macro_export]
macro_rules! tsrmg_cache {
    () => {
        crate::_TSRM_LS_CACHE
    };
}

#[macro_export]
macro_rules! tsrmg_cache_define {
    () => {
        pub static mut _TSRM_LS_CACHE: *mut ::std::ffi::c_void = ::std::ptr::null_mut();
    };
}

#[macro_export]
macro_rules! tsrmg_cache_update {
    () => {
        crate::_TSRM_LS_CACHE = $crate::tsrm::tsrm_get_ls_cache();
    };
}

#[macro_export]
macro_rules! tsrmg_fast_bulk_static {
    ($offset: expr, $ty: ty) => {
        ($crate::tsrm::tsrmg_cache!() as *mut ::std::ffi::c_char).offset($offset as isize) as $ty
    };
}

#[macro_export]
macro_rules! tsrmg_fast_static {
    ($offset: expr, $ty: ty, $element: ident) => {
        (*($crate::tsrm::tsrmg_fast_bulk_static!($offset, $ty))).$element
    };
}

pub use {
    tsrmg_cache, tsrmg_cache_define, tsrmg_cache_update, tsrmg_fast, tsrmg_fast_bulk,
    tsrmg_fast_bulk_static, tsrmg_fast_static,
};
