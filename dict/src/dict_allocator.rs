use std::{mem, ptr};

pub(crate) struct DictAllocator {}

impl DictAllocator {
    pub(crate) fn new() -> DictAllocator {
        DictAllocator {}
    }

    pub(crate) fn alloc<T>(&self, len: usize) -> *mut T {
        unsafe {
            let size = len.saturating_mul(size_of::<T>());
            libc::malloc(size) as *mut T
        }
    }

    pub(crate) fn dealloc<T>(&self, ptr: *mut T) {
        unsafe {
            if !ptr.is_null() {
                ptr::drop_in_place(ptr);
            }
            libc::free(ptr as *mut libc::c_void)
        }
    }
}

#[repr(transparent)]
pub(crate) struct DictBox<T> {
    ptr: ptr::NonNull<T>,
}

impl<T> DictBox<T> {
    pub(crate) fn new(val: T) -> Option<Self> {
        let allocator = DictAllocator::new();
        let ptr = allocator.alloc::<T>(1);

        if ptr.is_null() {
            return None;
        }

        unsafe {
            ptr::write(ptr, val);
            Some(Self {
                ptr: ptr::NonNull::new_unchecked(ptr),
            })
        }
    }

    /// Consumes the box and returns the raw pointer.
    /// The caller is responsible for cleaning up the memory.
    pub(crate) fn into_raw(self) -> *mut T {
        let ptr = self.ptr.as_ptr();
        mem::forget(self);
        ptr
    }
}

impl<T> Drop for DictBox<T> {
    fn drop(&mut self) {
        let allocator = DictAllocator::new();
        allocator.dealloc(self.ptr.as_ptr());
    }
}

impl<T> std::ops::Deref for DictBox<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> std::ops::DerefMut for DictBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut() }
    }
}
