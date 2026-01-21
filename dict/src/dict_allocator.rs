use std::{mem, ptr};

pub(crate) struct DictAllocator { }

impl DictAllocator {
    pub(crate) fn new() -> DictAllocator {
        DictAllocator{}
    }

    pub(crate) fn alloc<T>(&self, len: usize) -> *mut T {
        unsafe {
            libc::malloc(len * mem::size_of::<T>()) as *mut T
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