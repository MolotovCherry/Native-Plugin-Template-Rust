use windows::{Win32::Foundation::HANDLE, core::Owned};

/// Owned HANDLE that CloseHandle on Drop
pub type OwnedHandle = Owned<HANDLE>;

/// Allows a !Send||!Sync type to be sent/shared regardless
pub struct ThreadedWrapper<T>(T);
unsafe impl<T> Send for ThreadedWrapper<T> {}
unsafe impl<T> Sync for ThreadedWrapper<T> {}

impl<T> ThreadedWrapper<T> {
    /// # Safety
    /// Caller asserts that T is Send+Sync safe, or you ensure
    /// T is never misused
    pub unsafe fn new(t: T) -> Self {
        Self(t)
    }

    pub fn inner(&self) -> &T {
        &self.0
    }
}
