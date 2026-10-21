// UNVERIFIED: user prohibited all validation.
use super::SkSLOutputStream::{OutputStream, OutputStreamBytes};
use crate::src::core::SkStream::{SkDynamicMemoryWStream, SkWStream};
use core::{
    cell::{Ref, RefCell},
    ffi::{c_char, c_void},
    mem::ManuallyDrop,
    pin::Pin,
};
// Source: src/sksl/SkSLStringStream.h:17-17
// Source: src/sksl/SkSLStringStream.h:52-52
// Source: src/sksl/SkSLStringStream.h:53-53
// Native mutable const-cache uses RefCell borrow guards; real block stream ownership is retained.
#[repr(C)]
pub struct StringStream {
    fStream: ManuallyDrop<RefCell<SkDynamicMemoryWStream>>,
    fString: ManuallyDrop<RefCell<Vec<u8>>>,
}
impl StringStream {
    pub fn new() -> Self {
        Self {
            fStream: ManuallyDrop::new(RefCell::new(SkDynamicMemoryWStream::new())),
            fString: ManuallyDrop::new(RefCell::new(Vec::new())),
        }
    }
}
unsafe impl OutputStreamBytes for StringStream {
    // Source: src/sksl/SkSLStringStream.h:19-22
    fn write8(&mut self, b: u8) {
        debug_assert!(self.fString.get_mut().is_empty());
        unsafe { Pin::new_unchecked(self.fStream.get_mut()) }.write8(b as u32);
    }
}
unsafe impl OutputStream for StringStream {
    // Source: src/sksl/SkSLStringStream.h:24-27
    unsafe fn writeText(&mut self, s: *const c_char) {
        debug_assert!(self.fString.get_mut().is_empty());
        unsafe { Pin::new_unchecked(self.fStream.get_mut()) }.writeText(s);
    }
    // Source: src/sksl/SkSLStringStream.h:29-32
    unsafe fn write(&mut self, s: *const c_void, size: usize) {
        debug_assert!(self.fString.get_mut().is_empty());
        unsafe { Pin::new_unchecked(self.fStream.get_mut()) }.write(s.cast(), size);
    }
}
impl StringStream {
    // Source: src/sksl/SkSLStringStream.h:34-36
    pub fn bytesWritten(&self) -> usize {
        self.fStream.borrow().bytesWritten()
    }
    // Source: src/sksl/SkSLStringStream.h:38-44
    pub fn str(&self) -> Ref<'_, Vec<u8>> {
        if self.fString.borrow().is_empty() {
            let data = {
                let mut stream = self.fStream.borrow_mut();
                unsafe { Pin::new_unchecked(&mut *stream) }.detachAsData()
            };
            let value = if data.size() == 0 {
                Vec::new()
            } else {
                unsafe { core::slice::from_raw_parts(data.data(), data.size()).to_vec() }
            };
            *self.fString.borrow_mut() = value;
        }
        self.fString.borrow()
    }
    // Source: src/sksl/SkSLStringStream.h:46-49
    pub fn reset(&mut self) {
        unsafe { Pin::new_unchecked(self.fStream.get_mut()) }.reset();
        self.fString.get_mut().clear();
    }
}

// Native reverse-member destruction: cached string dies before block stream.
impl Drop for StringStream {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.fString);
            ManuallyDrop::drop(&mut self.fStream);
        }
    }
}
