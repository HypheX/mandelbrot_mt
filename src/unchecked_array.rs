pub struct UncheckedSyncArray<'a, T>(*mut T, usize, core::marker::PhantomData<&'a mut T>);

unsafe impl<'a, T: Send + Sync> Sync for UncheckedSyncArray<'a, T> {}

impl<'a, T> UncheckedSyncArray<'a, T> {
    pub fn from_slice(v: &'a mut [T]) -> Self {
        UncheckedSyncArray(v.as_mut_ptr(), v.len(), core::marker::PhantomData)
    }

    /// # Safety:
    /// As this has no mechanism to ensure more than 1 thread accesses the same index at a time,
    /// if more than 1 thread accesses the same index at a time UB will occur.
    /// It will also not check if the given index `idx` is in the bounds of the array.
    /// Trying to store something in an index that is out of bounds is instantly UB.
    pub unsafe fn store_unchecked(&self, idx: usize, item: T) {
        // SAFETY: no other threads are accessing this index, so we can safely write to it
        // we drop the T given to us by replace, this lets us hack dropping the old T
        unsafe { core::ptr::replace(self.0.add(idx), item) };
    }
}
