#![feature(c_variadic)]

use std::error::Error;
use std::ffi::c_void;
use std::path::Path;

use libloading::os::unix::{RTLD_GLOBAL, RTLD_NOW};
use libloading::Library;

use crate::ext::{Extensions, ExtensionsRaw};
use crate::sapi::SapiModuleStruct;
use crate::streams::{Streams, StreamsRaw};
use crate::zend::stream::ZendFileHandle;
use crate::zend::{Zend, ZendRaw, ZendResult};

pub mod ext;
pub mod sapi;
pub mod streams;
pub mod zend;

#[macro_export]
macro_rules! php_lib {
    (
        $v:vis struct $n:ident < $m:ident > {
            $($fv:vis $f:ident : fn( $($arg_name:ident : $arg_ty:ty ,)* ) $(-> $ret:ty)?,)*
            $({ $( $sv:vis $s:ident : $st:ident < $str:ty >,)* })?
        }
    ) => {
        $v struct $n<'lib> {
            $($fv $f: libloading::Symbol<'lib, fn ($($arg_name: $arg_ty,)*) $(-> $ret)?>,)*
            $($($sv $s: $st<'lib>,)*)?
            $v _phantom: &'lib std::marker::PhantomData<()>,
        }

        impl<'lib> $n<'lib> {
            $v fn load(lib: &'lib libloading::Library) -> Result<Self, libloading::Error> {
                #[allow(unused_unsafe)]
                Ok(unsafe {
                    Self {
                        $($f: lib.get(stringify!($f).as_bytes())?,)*
                        $($($s: $st::<'lib>::load(lib)?,)*)?
                        _phantom: &std::marker::PhantomData,
                    }
                })
            }

            $v fn into_raw(self) -> $m {
                $m {
                    $($f: unsafe { self.$f.into_raw() },)*
                    $($($s: self.$s.into_raw(),)*)?
                }
            }
        }

        $v struct $m {
            $($fv $f: libloading::os::unix::Symbol<fn ($($arg_name: $arg_ty,)*) $(-> $ret)?>,)*
            $($($sv $s: $str,)*)?
        }

        impl $m {
            $($v fn $f(&self, $($arg_name: $arg_ty,)*) $(-> $ret)? {
                tracing::debug!("BEGIN: {}", stringify!($f));
                let ret = (self.$f)($($arg_name,)*);
                tracing::debug!("OK: {}", stringify!($f));
                ret
            })*
        }
    }
}

php_lib! {
    pub struct PhpLib<Php> {
        pub php_request_startup: fn() -> ZendResult,
        pub php_request_shutdown: fn(dummy: *mut c_void,) -> ZendResult,
        pub php_module_startup: fn(sf: *mut SapiModuleStruct, additional_module: *mut c_void,) -> ZendResult,
        pub php_module_shutdown: fn(),
        pub php_execute_script: fn(primary_file: *mut ZendFileHandle,),
        pub sapi_startup: fn(sf: *mut SapiModuleStruct,),
        pub sapi_shutdown: fn(),
        {
            pub ext: Extensions<ExtensionsRaw>,
            pub streams: Streams<StreamsRaw>,
            pub zend: Zend<ZendRaw>,
        }
    }
}

impl Php {
    pub fn load<P>(path: P) -> Result<Self, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        #[cfg(unix)]
        let php = unsafe {
            libloading::os::unix::Library::open(
                Some(path.as_ref().as_os_str()),
                RTLD_NOW | RTLD_GLOBAL,
            )
        }?;

        #[cfg(not(unix))]
        let php = unsafe { Library::new("/opt/homebrew/bin/php") }?;

        let php = Library::from(php);
        Ok(PhpLib::load(&php)?.into_raw())
    }
}
