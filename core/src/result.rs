use std::convert::Infallible;
use std::ffi::{c_int, c_uint};
use std::ops::{ControlFlow, FromResidual, Try};

pub use self::Result::{Err, Ok};
use crate::sys::zend::ZendResultCode;

#[derive(Copy, Clone)]
pub enum Result<T> {
    Ok(T),
    Err,
}

impl<T> Result<T> {
    pub fn unwrap(self) -> T {
        match self {
            Ok(v) => v,
            Err => panic!("error occurred while communicating with PHP"),
        }
    }
}

impl<T> Result<T>
where
    T: Copy,
{
    pub(crate) fn writing_raw(self, ptr: *mut T) -> Self {
        if let Ok(v) = &self {
            unsafe { *ptr = *v };
        }
        self
    }
}

impl<T> FromResidual for Result<T> {
    fn from_residual(_: <Self as Try>::Residual) -> Self {
        Err
    }
}

impl<T> Try for Result<T> {
    type Output = T;
    type Residual = Result<Infallible>;

    #[inline]
    fn from_output(output: Self::Output) -> Self {
        Ok(output)
    }

    #[inline]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Ok(v) => ControlFlow::Continue(v),
            Err => ControlFlow::Break(Err),
        }
    }
}

impl From<c_int> for Result<()> {
    fn from(value: c_int) -> Self {
        match value {
            0 => Ok(()),
            _ => Err,
        }
    }
}

impl<T> From<Result<T>> for c_int {
    fn from(value: Result<T>) -> Self {
        match value {
            Ok(_) => 0,
            _ => 1,
        }
    }
}

impl<T> From<Result<T>> for c_uint {
    fn from(value: Result<T>) -> Self {
        c_int::from(value) as c_uint
    }
}

impl From<ZendResultCode> for Result<()> {
    fn from(value: ZendResultCode) -> Self {
        match value {
            ZendResultCode::Success => Ok(()),
            ZendResultCode::Failure => Err,
        }
    }
}

impl<T> From<Result<T>> for ZendResultCode {
    fn from(value: Result<T>) -> Self {
        match value {
            Ok(_) => Self::Success,
            _ => Self::Failure,
        }
    }
}
