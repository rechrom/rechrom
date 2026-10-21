use core::{mem, ptr};
// Source: src/core/SkBuffer.h:18-18
pub type SkScalar = f32;
// Source: src/core/SkBuffer.h:75-78
pub struct SkRBuffer {
    fData: *const u8,
    fPos: *const u8,
    fStop: *const u8,
    fValid: bool,
}
impl SkRBuffer {
    // Source: src/core/SkBuffer.h:29-29
    pub fn empty() -> Self {
        Self {
            fData: ptr::null(),
            fPos: ptr::null(),
            fStop: ptr::null(),
            fValid: true,
        }
    }
    // Source: src/core/SkBuffer.h:33-38
    /// Safety: data covers size immutable bytes through the last buffer access.
    pub unsafe fn new(data: *const u8, size: usize) -> Self {
        debug_assert!(!data.is_null() || size == 0);
        Self {
            fData: data,
            fPos: data,
            fStop: data.wrapping_add(size),
            fValid: true,
        }
    }
    // Source: src/core/SkBuffer.h:43-43
    pub fn pos(&self) -> usize {
        (self.fPos as usize).wrapping_sub(self.fData as usize)
    }
    // Source: src/core/SkBuffer.h:47-47
    pub fn size(&self) -> usize {
        (self.fStop as usize).wrapping_sub(self.fData as usize)
    }
    // Source: src/core/SkBuffer.h:52-52
    pub fn eof(&self) -> bool {
        self.fPos >= self.fStop
    }
    // Source: src/core/SkBuffer.h:54-54
    pub fn available(&self) -> usize {
        (self.fStop as usize).wrapping_sub(self.fPos as usize)
    }
    // Source: src/core/SkBuffer.h:56-56
    pub fn isValid(&self) -> bool {
        self.fValid
    }
    // Source: src/core/SkBuffer.h:64-64
    pub fn readU8(&mut self, x: &mut u8) -> bool {
        unsafe { self.read((x as *mut u8).cast(), 1) }
    }
    // Source: src/core/SkBuffer.h:65-65
    pub fn readS32(&mut self, x: &mut i32) -> bool {
        unsafe { self.read((x as *mut i32).cast(), 4) }
    }
    // Source: src/core/SkBuffer.h:66-66
    pub fn readU32(&mut self, x: &mut u32) -> bool {
        unsafe { self.read((x as *mut u32).cast(), 4) }
    }
    // Source: src/core/SkBuffer.h:70-72
    pub fn skipCount<T>(&mut self, count: usize) -> *const T {
        self.skip(crate::src::core::SkSafeMath::SkSafeMath::Mul(
            count,
            mem::size_of::<T>(),
        ))
        .cast()
    }
    // Source: src/core/SkBuffer.cpp:16-24
    pub fn skip(&mut self, size: usize) -> *const u8 {
        if self.fValid && size <= self.available() {
            let pos = self.fPos;
            self.fPos = self.fPos.wrapping_add(size);
            return pos;
        }
        self.fValid = false;
        ptr::null()
    }
    // Source: src/core/SkBuffer.cpp:26-32
    /// Safety: dst covers size exclusive writable bytes, without source overlap.
    pub unsafe fn read(&mut self, dst: *mut u8, size: usize) -> bool {
        let src = self.skip(size);
        if !src.is_null() {
            if size != 0 {
                unsafe { ptr::copy_nonoverlapping(src, dst, size) }
            }
            return true;
        }
        false
    }
    // Source: src/core/SkBuffer.cpp:34-45
    pub fn skipToAlign4(&mut self) -> bool {
        let pos = self.fPos as usize;
        let n = pos.wrapping_add(3) & !3;
        let n = n.wrapping_sub(pos);
        if self.fValid && n <= self.available() {
            self.fPos = self.fPos.wrapping_add(n);
            true
        } else {
            self.fValid = false;
            false
        }
    }
}
// Source: src/core/SkBuffer.h:129-131
pub struct SkWBuffer {
    fData: *mut u8,
    fPos: *mut u8,
    fStop: *mut u8,
}
impl SkWBuffer {
    // Source: src/core/SkBuffer.h:91-91
    pub fn empty() -> Self {
        Self {
            fData: ptr::null_mut(),
            fPos: ptr::null_mut(),
            fStop: ptr::null_mut(),
        }
    }
    // Source: src/core/SkBuffer.h:92-92
    /// Safety: data is null (count-only) or covers every subsequent write exclusively.
    pub unsafe fn new(data: *mut u8) -> Self {
        let mut s = Self::empty();
        unsafe { s.reset(data) };
        s
    }
    // Source: src/core/SkBuffer.h:93-93
    pub unsafe fn with_size(data: *mut u8, size: usize) -> Self {
        let mut s = Self::empty();
        unsafe { s.reset_with_size(data, size) };
        s
    }
    // Source: src/core/SkBuffer.h:95-99
    pub unsafe fn reset(&mut self, data: *mut u8) {
        self.fData = data;
        self.fPos = data;
        self.fStop = ptr::null_mut();
    }
    // Source: src/core/SkBuffer.h:101-106
    pub unsafe fn reset_with_size(&mut self, data: *mut u8, size: usize) {
        debug_assert!(!data.is_null() || size == 0);
        self.fData = data;
        self.fPos = data;
        self.fStop = data.wrapping_add(size);
    }
    // Source: src/core/SkBuffer.h:108-108
    pub fn pos(&self) -> usize {
        (self.fPos as usize).wrapping_sub(self.fData as usize)
    }
    // Source: src/core/SkBuffer.h:111-115
    pub unsafe fn write(&mut self, buffer: *const u8, size: usize) {
        if size != 0 {
            unsafe { self.writeNoSizeCheck(buffer, size) }
        }
    }
    // Source: src/core/SkBuffer.h:119-119
    pub unsafe fn writePtr(&mut self, x: *const core::ffi::c_void) {
        unsafe {
            self.writeNoSizeCheck(
                (&x as *const *const core::ffi::c_void).cast(),
                mem::size_of_val(&x),
            )
        }
    }
    // Source: src/core/SkBuffer.h:120-120
    pub unsafe fn writeScalar(&mut self, x: f32) {
        unsafe { self.writeNoSizeCheck((&x as *const f32).cast(), 4) }
    }
    // Source: src/core/SkBuffer.h:121-121
    pub unsafe fn write32(&mut self, x: i32) {
        unsafe { self.writeNoSizeCheck((&x as *const i32).cast(), 4) }
    }
    // Source: src/core/SkBuffer.h:122-122
    pub unsafe fn write16(&mut self, x: i16) {
        unsafe { self.writeNoSizeCheck((&x as *const i16).cast(), 2) }
    }
    // Source: src/core/SkBuffer.h:123-123
    pub unsafe fn write8(&mut self, x: i8) {
        unsafe { self.writeNoSizeCheck((&x as *const i8).cast(), 1) }
    }
    // Source: src/core/SkBuffer.h:124-124
    pub unsafe fn writeBool(&mut self, x: bool) {
        unsafe { self.write8(x as i8) }
    }
    // Source: src/core/SkBuffer.cpp:49-53
    pub unsafe fn skip(&mut self, size: usize) -> *mut u8 {
        let result = self.fPos;
        unsafe { self.writeNoSizeCheck(ptr::null(), size) };
        if self.fData.is_null() {
            ptr::null_mut()
        } else {
            result
        }
    }
    // Source: src/core/SkBuffer.cpp:55-61
    unsafe fn writeNoSizeCheck(&mut self, buffer: *const u8, size: usize) {
        debug_assert!(
            self.fData.is_null()
                || self.fStop.is_null()
                || self.fPos.wrapping_add(size) <= self.fStop
        );
        if !self.fData.is_null() && !buffer.is_null() && size != 0 {
            unsafe { ptr::copy_nonoverlapping(buffer, self.fPos, size) }
        }
        self.fPos = self.fPos.wrapping_add(size);
    }
    // Source: src/core/SkBuffer.cpp:63-77
    pub unsafe fn padToAlign4(&mut self) -> usize {
        let pos = self.pos();
        let n = pos.wrapping_add(3) & !3;
        let n = n.wrapping_sub(pos);
        if n != 0 && !self.fData.is_null() {
            let mut p = self.fPos;
            let stop = p.wrapping_add(n);
            loop {
                unsafe { p.write(0) }
                p = p.wrapping_add(1);
                if p >= stop {
                    break;
                }
            }
        }
        self.fPos = self.fPos.wrapping_add(n);
        n
    }
}
