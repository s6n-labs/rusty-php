use std::marker::PhantomData;

use crate::sys::zend::stream::ZendFileHandle;

pub struct ZFileHandle<'a> {
    raw: ZendFileHandle,
    _phantom: &'a PhantomData<()>,
}

impl<'a> ZFileHandle<'a> {
    pub fn set_primary_script(&mut self, is_primary: bool) {
        self.raw.primary_script = is_primary;
    }
}

impl<'a> From<ZendFileHandle> for ZFileHandle<'a> {
    fn from(value: ZendFileHandle) -> Self {
        Self {
            raw: value,
            _phantom: &PhantomData,
        }
    }
}

impl<'a> From<&'a mut ZFileHandle<'a>> for &'a mut ZendFileHandle {
    fn from(value: &'a mut ZFileHandle) -> Self {
        &mut value.raw
    }
}
