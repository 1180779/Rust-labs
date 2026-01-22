use crate::dict_allocator::{DictAllocator, DictBox};
use std::fmt::Display;
use std::ptr;

#[repr(C)]
pub struct DictString {
    ptr: *mut u8,
    // Length of string excluding the null terminator
    len: usize,
}

impl Default for DictString {
    fn default() -> Self {
        Self::new()
    }
}

impl DictString {
    pub fn new() -> DictString {
        DictString {
            ptr: ptr::null_mut(),
            len: 0,
        }
    }

    pub fn as_str(&self) -> &str {
        if self.ptr.is_null() {
            return "";
        }

        unsafe {
            let slice = std::slice::from_raw_parts(self.ptr, self.len);
            // ptr should contain ascii, which is also utf8, but check just in case
            let str = std::str::from_utf8(slice);
            str.unwrap_or("")
        }
    }

    /// The [bytes] parameter must not contain 0s
    pub fn from(bytes: &[u8]) -> Option<DictString> {
        if bytes.contains(&0) {
            return None;
        }

        let len = bytes.len();
        unsafe {
            let allocator = DictAllocator::new();
            let ptr = allocator.alloc::<u8>(bytes.len() + 1);
            if ptr.is_null() {
                return None;
            }

            ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, len);
            *ptr.add(len) = 0;

            Some(DictString { ptr, len })
        }
    }

    pub fn from_ascii(str: &str) -> Option<DictString> {
        if !str.is_ascii() {
            return None;
        }
        DictString::from(str.as_bytes())
    }
}

impl Display for DictString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.as_str())
    }
}

impl PartialEq for DictString {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl std::fmt::Debug for DictString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

impl From<&str> for DictString {
    fn from(str: &str) -> Self {
        DictString::from_ascii(str).unwrap_or_default()
    }
}

impl Drop for DictString {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            let allocator = DictAllocator::new();
            allocator.dealloc(self.ptr);
        }
    }
}

impl DictString {
    unsafe fn validate_raw_input(src_ptr: *mut u8, len: usize) -> bool {
        if src_ptr.is_null() {
            return false;
        }
        unsafe {
            let slice = std::slice::from_raw_parts(src_ptr, len);
            if len > 0 && slice.contains(&0) {
                return false;
            }
            true
        }
    }
}

/// # Safety
/// - [src_ptr] must point to a valid, readable memory region of at least [len] bytes.
/// - Proper cleanup of the returned [DictString] is the caller's responsibility.
///   The ownership of the allocated memory is transferred to the caller.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dict_string_from_raw(src_ptr: *mut u8, len: usize) -> *mut DictString {
    unsafe {
        if !DictString::validate_raw_input(src_ptr, len) {
            return ptr::null_mut();
        }

        let allocator = DictAllocator::new();
        let str_internal_ptr = allocator.alloc::<u8>(len + 1);
        if str_internal_ptr.is_null() {
            return ptr::null_mut();
        }

        ptr::copy_nonoverlapping(src_ptr, str_internal_ptr, len);
        *str_internal_ptr.add(len) = 0;

        let Some(dict_box) = DictBox::new(DictString {
            ptr: str_internal_ptr,
            len,
        }) else {
            allocator.dealloc(str_internal_ptr);
            return ptr::null_mut();
        };

        dict_box.into_raw()
    }
}

/// # Safety
/// The [s] must be NULL or point to a valid [DictString]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dict_string_str_ptr(s: *const DictString) -> *const libc::c_char {
    if s.is_null() {
        return ptr::null();
    }

    unsafe { (*s).ptr as *const libc::c_char }
}

/// # Safety
/// The [s] must be NULL or point to a valid [DictString]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dict_string_len(s: *const DictString) -> libc::size_t {
    if s.is_null() {
        return 0;
    }

    unsafe { (*s).len }
}

/// # Safety
/// The [s] must be NULL or point to a valid [DictString].
/// The [s] will be freed and must not be used by the caller from this point on.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dict_string_free(s: *mut DictString) {
    if !s.is_null() {
        let allocator = DictAllocator::new();
        allocator.dealloc(s);
    }
}

#[cfg(test)]
mod rust_api_tests {
    use super::*;

    #[test]
    fn empty() {
        let _dict = DictString::new();
    }

    #[cfg(test)]
    mod from_bytes {
        use super::*;

        #[test]
        fn empty() {
            let _dict = Option::unwrap(DictString::from(&[]));
            assert_eq!(_dict.as_str(), "");
        }

        #[test]
        fn from_bytes() {
            let _dict = Option::unwrap(DictString::from(b"abc"));
            assert_eq!(_dict.as_str(), "abc");
        }
    }

    #[cfg(test)]
    mod from_ascii {
        use super::*;

        #[test]
        fn empty() {
            let _dict = Option::unwrap(DictString::from_ascii(""));
            assert!(_dict.as_str().is_empty());
        }

        #[test]
        fn valid() {
            let _dict = Option::unwrap(DictString::from_ascii("abc"));
            assert_eq!(_dict.as_str(), "abc");
        }

        #[test]
        fn invalid() {
            let _dict = DictString::from_ascii("albo lub czy bądź");
            assert!(_dict.is_none());
        }
    }
}

#[cfg(test)]
mod c_api_tests {}
