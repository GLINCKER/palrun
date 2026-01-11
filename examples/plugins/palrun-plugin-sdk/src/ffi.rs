//! FFI utilities for WASM plugins.
//!
//! This module provides low-level utilities for memory management
//! and data transfer between the host and plugin.

/// Convert a String to a raw pointer for FFI.
///
/// The caller (host) is responsible for freeing this memory
/// using the `dealloc` function.
///
/// # Safety
///
/// The returned pointer is valid until `dealloc` is called.
#[inline]
pub fn string_to_ptr(s: String) -> *mut u8 {
    let bytes = s.into_bytes();
    let ptr = bytes.as_ptr() as *mut u8;
    std::mem::forget(bytes);
    ptr
}

/// Convert a raw pointer and length to a String.
///
/// # Safety
///
/// The pointer must be valid for `len` bytes and point to valid UTF-8.
///
/// # Panics
///
/// Panics if the slice is not valid UTF-8.
#[inline]
pub unsafe fn ptr_to_string(ptr: *const u8, len: usize) -> String {
    // SAFETY: Caller guarantees ptr is valid for len bytes
    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    std::str::from_utf8(slice)
        .expect("invalid UTF-8 from host")
        .to_string()
}

/// Convert a raw pointer and length to a string slice.
///
/// # Safety
///
/// The pointer must be valid for `len` bytes and point to valid UTF-8.
#[inline]
pub unsafe fn ptr_to_str<'a>(ptr: *const u8, len: usize) -> &'a str {
    // SAFETY: Caller guarantees ptr is valid for len bytes
    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    std::str::from_utf8(slice).unwrap_or("")
}

/// Allocate memory for the host to write into.
///
/// # Arguments
///
/// * `size` - Number of bytes to allocate
///
/// # Returns
///
/// Pointer to allocated memory.
#[inline]
pub fn alloc(size: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// Deallocate memory previously allocated by `alloc` or returned by plugin functions.
///
/// # Arguments
///
/// * `ptr` - Pointer to memory to free
/// * `size` - Size of the allocation
///
/// # Safety
///
/// The pointer must have been allocated by this plugin.
#[inline]
pub fn dealloc(ptr: *mut u8, size: usize) {
    // SAFETY: We're reconstructing a Vec from a pointer we allocated
    unsafe {
        let _ = Vec::from_raw_parts(ptr, size, size);
    }
}

/// Result buffer for returning data to the host.
///
/// This is used to return both the data and its length.
#[repr(C)]
pub struct ResultBuffer {
    /// Pointer to the data.
    pub ptr: *mut u8,
    /// Length of the data.
    pub len: usize,
}

impl ResultBuffer {
    /// Create a new result buffer from a string.
    pub fn from_string(s: String) -> Self {
        let bytes = s.into_bytes();
        let len = bytes.len();
        let ptr = bytes.as_ptr() as *mut u8;
        std::mem::forget(bytes);
        Self { ptr, len }
    }

    /// Create an empty result buffer.
    pub fn empty() -> Self {
        Self {
            ptr: std::ptr::null_mut(),
            len: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_to_ptr() {
        let s = String::from("hello");
        let ptr = string_to_ptr(s);
        assert!(!ptr.is_null());

        // Clean up
        dealloc(ptr, 5);
    }

    #[test]
    fn test_ptr_to_string() {
        let original = "hello world";
        let ptr = original.as_ptr();
        let len = original.len();

        let result = unsafe { ptr_to_string(ptr, len) };
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_alloc_dealloc() {
        let ptr = alloc(1024);
        assert!(!ptr.is_null());
        dealloc(ptr, 1024);
    }

    #[test]
    fn test_result_buffer() {
        let buf = ResultBuffer::from_string(String::from("test"));
        assert!(!buf.ptr.is_null());
        assert_eq!(buf.len, 4);

        // Clean up
        dealloc(buf.ptr, buf.len);
    }

    #[test]
    fn test_result_buffer_empty() {
        let buf = ResultBuffer::empty();
        assert!(buf.ptr.is_null());
        assert_eq!(buf.len, 0);
    }
}
