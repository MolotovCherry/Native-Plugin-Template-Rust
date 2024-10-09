use std::ops::Deref;

use windows::{
    core::{self, Free},
    Win32::Foundation::HANDLE,
};

pub type OwnedHandleResult = core::Result<OwnedHandle>;

#[derive(Debug)]
pub struct OwnedHandle(HANDLE);

impl From<HANDLE> for OwnedHandle {
    fn from(value: HANDLE) -> Self {
        Self(value)
    }
}

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        unsafe {
            self.0.free();
        }
    }
}

pub trait OwnedHandleConvert {
    fn to_owned(self) -> core::Result<OwnedHandle>;
}

impl<T: Into<OwnedHandle>> OwnedHandleConvert for core::Result<T> {
    fn to_owned(self) -> core::Result<OwnedHandle> {
        self.map(|t| t.into())
    }
}

pub struct ThreadedWrapper<T>(T);
unsafe impl<T> Send for ThreadedWrapper<T> {}
unsafe impl<T> Sync for ThreadedWrapper<T> {}

impl<T> ThreadedWrapper<T> {
    /// # Safety
    /// Caller asserts that T is safe to use in Send+Sync contexts
    pub unsafe fn new(t: T) -> Self {
        Self(t)
    }
}

impl<T> Deref for ThreadedWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
