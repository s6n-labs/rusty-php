use std::marker::PhantomData;

use crate::sys::zend::{ZendLlist, ZendLlistElement};

#[derive(Debug)]
pub struct ZLlistElement<'a, T> {
    raw: &'a ZendLlistElement,
    _phantom: PhantomData<fn() -> T>,
}

impl<'a, T> Clone for ZLlistElement<'a, T> {
    fn clone(&self) -> Self {
        Self {
            raw: self.raw,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T> Copy for ZLlistElement<'a, T> {}

impl<'a, T> ZLlistElement<'a, T> {
    fn from_raw(raw: *mut ZendLlistElement) -> Option<Self> {
        if raw.is_null() {
            None
        } else {
            Some(Self::from(unsafe { &*raw }))
        }
    }

    pub fn next(&self) -> Option<Self> {
        Self::from_raw(self.raw.next)
    }

    pub fn data(&self) -> &T {
        &(unsafe { std::slice::from_raw_parts(self.raw.data.as_ptr() as *const T, 1) })[0]
    }
}

impl<'a, T> From<&'a ZendLlistElement> for ZLlistElement<'a, T> {
    fn from(value: &'a ZendLlistElement) -> Self {
        Self {
            raw: value,
            _phantom: PhantomData,
        }
    }
}

pub struct ZLlistIter<'a, T> {
    #[allow(unused)]
    inner: ZLlist<'a, T>,
    cursor: Option<ZLlistElement<'a, T>>,
}

impl<'a, T> Iterator for ZLlistIter<'a, T> {
    type Item = ZLlistElement<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.cursor.map(|c| {
            self.cursor = c.next();
            c
        })
    }
}

#[derive(Debug)]
pub struct ZLlist<'a, T> {
    pub raw: &'a ZendLlist,
    _phantom: PhantomData<fn() -> T>,
}

impl<'a, T> ZLlist<'a, T> {
    pub fn len(&self) -> usize {
        self.raw.count
    }

    pub fn is_empty(&self) -> bool {
        self.raw.head.is_null()
    }

    pub fn head(&self) -> Option<ZLlistElement<'a, T>> {
        ZLlistElement::<T>::from_raw(self.raw.head)
    }
}

impl<'a, T> PartialEq for ZLlist<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.raw, other.raw)
    }
}

impl<'a, T> From<&'a ZendLlist> for ZLlist<'a, T> {
    fn from(value: &'a ZendLlist) -> Self {
        Self {
            raw: value,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T> IntoIterator for ZLlist<'a, T>
where
    T: 'a,
{
    type Item = ZLlistElement<'a, T>;
    type IntoIter = ZLlistIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        ZLlistIter::<'a, T> {
            cursor: self.head(),
            inner: self,
        }
    }
}
