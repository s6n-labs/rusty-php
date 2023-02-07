use rusty_php_sys::zend::{ZendArray, ZendBucket, Zval, HASH_FLAG_PACKED};

use crate::zend::string::ZStr;
use crate::zend::Value;

#[derive(Copy, Clone, Debug)]
enum ZArrayElementRaw<'a> {
    Packed(&'a Zval),
    Normal(&'a ZendBucket),
}

#[derive(Debug)]
pub struct ZArrayElement<'a> {
    raw: ZArrayElementRaw<'a>,
}

impl<'a> ZArrayElement<'a> {
    pub fn key(&self) -> Option<ZStr<'a>> {
        match self.raw {
            ZArrayElementRaw::Packed(_) => None,
            ZArrayElementRaw::Normal(b) => Some(unsafe { &*b.key }.into()),
        }
    }

    pub fn value(&self) -> Value<'a> {
        match self.raw {
            ZArrayElementRaw::Packed(v) => v.into(),
            ZArrayElementRaw::Normal(b) => (&b.val).into(),
        }
    }
}

impl<'a> From<&'a ZendBucket> for ZArrayElement<'a> {
    fn from(value: &'a ZendBucket) -> Self {
        Self {
            raw: ZArrayElementRaw::Normal(value),
        }
    }
}

impl<'a> From<&'a Zval> for ZArrayElement<'a> {
    fn from(value: &'a Zval) -> Self {
        Self {
            raw: ZArrayElementRaw::Packed(value),
        }
    }
}

pub struct ZArrayIter<'a> {
    inner: ZArray<'a>,
    cursor: usize,
}

impl<'a> Iterator for ZArrayIter<'a> {
    type Item = ZArrayElement<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.inner.len() {
            return None;
        }

        let value = match self.inner.is_packed() {
            true => (&self.inner.as_raw_slice_packed()[self.cursor]).into(),
            _ => (&self.inner.as_raw_slice()[self.cursor]).into(),
        };

        self.cursor += 1;
        Some(value)
    }
}

#[derive(Debug)]
pub struct ZArray<'a> {
    pub raw: &'a ZendArray,
}

impl<'a> ZArray<'a> {
    pub fn len(&self) -> usize {
        self.raw.n_num_of_elements as usize
    }

    pub fn is_packed(&self) -> bool {
        self.raw.flags & HASH_FLAG_PACKED != 0
    }

    pub fn as_raw_slice(&self) -> &'a [ZendBucket] {
        unsafe {
            std::slice::from_raw_parts(
                self.raw.array_data.ar_data,
                self.raw.n_num_of_elements as usize,
            )
        }
    }

    pub fn as_raw_slice_packed(&self) -> &'a [Zval] {
        unsafe {
            std::slice::from_raw_parts(
                self.raw.array_data.ar_packed,
                self.raw.n_num_of_elements as usize,
            )
        }
    }
}

impl<'a> PartialEq for ZArray<'a> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.raw, other.raw)
    }
}

impl<'a> From<&'a ZendArray> for ZArray<'a> {
    fn from(value: &'a ZendArray) -> Self {
        Self { raw: value }
    }
}

impl<'a> IntoIterator for ZArray<'a> {
    type Item = ZArrayElement<'a>;
    type IntoIter = ZArrayIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ZArrayIter::<'a> {
            inner: self,
            cursor: 0,
        }
    }
}
