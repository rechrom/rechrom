use crate::include::private::SkTemplates::AutoTMalloc;
use crate::src::core::SkData::SkData;
use core::{mem, ptr};
use std::sync::Arc;
// Source: src/core/SkWriter32.h:256-260
// Adapted private field representation: native reverse member destruction.
// Named construction below retains native constructor evaluation order.
pub struct SkWriter32 {
    // Source: src/core/SkWriter32.h:260-260
    fInternal: AutoTMalloc<u8>,
    // Source: src/core/SkWriter32.h:259-259
    fExternal: *mut u8,
    // Source: src/core/SkWriter32.h:258-258
    fUsed: usize,
    // Source: src/core/SkWriter32.h:257-257
    fCapacity: usize,
    // Source: src/core/SkWriter32.h:256-256
    fData: *mut u8,
}
impl SkWriter32 {
    // Source: src/core/SkWriter32.h:43-45
    /// Safety: external is null or aligned writable storage valid until reset/drop; no aliased access.
    pub unsafe fn new(external: *mut u8, bytes: usize) -> Self {
        let mut s = Self {
            fData: ptr::null_mut(),
            fCapacity: 0,
            fUsed: 0,
            fExternal: ptr::null_mut(),
            fInternal: AutoTMalloc::new(),
        };
        unsafe { s.reset(external, bytes) };
        s
    }
    // Source: src/core/SkWriter32.h:48-48
    pub fn bytesWritten(&self) -> usize {
        self.fUsed
    }
    // Source: src/core/SkWriter32.h:52-52
    pub fn usingInitialStorage(&self) -> bool {
        self.fData == self.fExternal
    }
    // Source: src/core/SkWriter32.h:54-64
    /// Safety: same external storage lease as new; prior returned pointers are invalidated.
    pub unsafe fn reset(&mut self, external: *mut u8, mut bytes: usize) {
        debug_assert_eq!(external as usize & 3, 0);
        bytes &= !3;
        self.fData = external;
        self.fCapacity = bytes;
        self.fUsed = 0;
        self.fExternal = external;
    }
    // Source: src/core/SkWriter32.h:67-76
    /// Safety: caller initializes reserved bytes before reading/snapshot; pointers expire on growth/reset.
    pub unsafe fn reserve(&mut self, size: usize) -> *mut u32 {
        debug_assert_eq!((size + 3) & !3, size);
        let offset = self.fUsed;
        let required = self.fUsed.checked_add(size).expect("writer size overflow");
        if required > self.fCapacity {
            self.growToAtLeast(required);
        }
        self.fUsed = required;
        self.fData.wrapping_add(offset).cast()
    }
    // Source: src/core/SkWriter32.h:83-87
    pub unsafe fn readTAt<T>(&self, offset: usize) -> &T {
        debug_assert_eq!(offset & 3, 0);
        crate::SkASSERT!(@source b"src/core/SkWriter32.h\0".as_ptr(),85; @condition b"offset < fUsed\0".as_ptr(); offset < self.fUsed);
        unsafe { &*self.fData.add(offset).cast() }
    }
    // Source: src/core/SkWriter32.h:94-98
    pub unsafe fn overwriteTAt<T: Copy>(&mut self, offset: usize, value: T) {
        debug_assert_eq!(offset & 3, 0);
        crate::SkASSERT!(@source b"src/core/SkWriter32.h\0".as_ptr(),96; @condition b"offset < fUsed\0".as_ptr(); offset < self.fUsed);
        unsafe { self.fData.add(offset).cast::<T>().write(value) }
    }
    // Source: src/core/SkWriter32.h:100-103
    pub fn writeBool(&mut self, value: bool) -> bool {
        self.write32(value as i32);
        value
    }
    // Source: src/core/SkWriter32.h:105-107
    pub fn writeInt(&mut self, value: i32) {
        self.write32(value)
    }
    // Source: src/core/SkWriter32.h:109-111
    pub fn write8(&mut self, value: i32) {
        unsafe { self.reserve(4).write((value & 255) as u32) }
    }
    // Source: src/core/SkWriter32.h:113-115
    pub fn write16(&mut self, value: i32) {
        unsafe { self.reserve(4).write((value & 65535) as u32) }
    }
    // Source: src/core/SkWriter32.h:117-119
    pub fn write32(&mut self, value: i32) {
        unsafe { self.reserve(4).write(value as u32) }
    }
    // Source: src/core/SkWriter32.h:121-123
    pub fn writeScalar(&mut self, value: f32) {
        unsafe { self.reserve(4).cast::<f32>().write(value) }
    }
    // Source: src/core/SkWriter32.h:125-127
    pub fn writePoint(&mut self, value: &crate::src::core::SkPoint::SkPoint) {
        unsafe {
            self.reserve(mem::size_of_val(value))
                .cast::<crate::src::core::SkPoint::SkPoint>()
                .write(*value)
        }
    }
    // Source: src/core/SkWriter32.h:129-131
    pub fn writePoint3(&mut self, value: &crate::src::core::SkPoint3::SkPoint3) {
        unsafe {
            self.reserve(mem::size_of_val(value))
                .cast::<crate::src::core::SkPoint3::SkPoint3>()
                .write(*value)
        }
    }
    // Source: src/core/SkWriter32.h:133-135
    pub fn writeRect(&mut self, value: &crate::src::core::SkRect::SkRect) {
        unsafe {
            self.reserve(mem::size_of_val(value))
                .cast::<crate::src::core::SkRect::SkRect>()
                .write(*value)
        }
    }
    // Source: src/core/SkWriter32.h:137-139
    pub fn writeIRect(&mut self, value: &crate::src::core::SkRect::SkIRect) {
        unsafe {
            self.reserve(mem::size_of_val(value))
                .cast::<crate::src::core::SkRect::SkIRect>()
                .write(*value)
        }
    }
    // Source: src/core/SkWriter32.h:141-143
    pub fn writeRRect(&mut self, rrect: &crate::src::core::SkRRect::SkRRect) {
        let size = crate::src::core::SkRRect::SkRRect::kSizeInMemory;
        let p = unsafe { self.reserve(size) };
        rrect.writeToMemory(unsafe { core::slice::from_raw_parts_mut(p.cast(), size) });
    }
    // Source: src/core/SkWriter32.h:145-149
    // Real native serializer dependency; SkPath::writeToMemory remains pending.
    pub fn writePath(&mut self, path: &crate::src::core::SkPath::SkPath) {
        let size = unsafe { path.writeToMemory(core::ptr::null_mut()) };
        debug_assert_eq!((size + 3) & !3, size);
        unsafe {
            path.writeToMemory(self.reserve(size).cast());
        }
    }
    // Source: src/core/SkWriter32.h:153-157
    // Real native serializer dependency; SkRegion::writeToMemory remains pending.
    pub fn writeRegion(&mut self, region: &crate::src::core::SkRegion::SkRegion) {
        let size = unsafe { region.writeToMemory(core::ptr::null_mut()) };
        debug_assert_eq!((size + 3) & !3, size);
        unsafe {
            region.writeToMemory(self.reserve(size).cast());
        }
    }
    // Source: src/core/SkWriter32.h:162-164
    pub unsafe fn writeMul4(&mut self, values: *const u8, size: usize) {
        unsafe { self.write(values, size) }
    }
    // Source: src/core/SkWriter32.h:170-173
    pub unsafe fn write(&mut self, values: *const u8, size: usize) {
        debug_assert_eq!((size + 3) & !3, size);
        let dst = unsafe { self.reserve(size) };
        if size != 0 {
            unsafe { ptr::copy_nonoverlapping(values, dst.cast(), size) }
        }
    }
    // Source: src/core/SkWriter32.h:179-187
    pub unsafe fn reservePad(&mut self, size: usize) -> *mut u32 {
        let aligned = (size + 3) & !3;
        let p = unsafe { self.reserve(aligned) };
        if aligned != size {
            debug_assert!(aligned >= 4);
            unsafe { p.add(aligned / 4 - 1).write(0) }
        }
        p
    }
    // Source: src/core/SkWriter32.h:192-194
    pub unsafe fn writePad(&mut self, src: *const u8, size: usize) {
        let dst = unsafe { self.reservePad(size) };
        if size != 0 {
            unsafe { ptr::copy_nonoverlapping(src, dst.cast(), size) }
        }
    }
    // Source: src/core/SkWriter32.h:212-218
    pub fn writeData(&mut self, data: Option<&SkData>) {
        let len = data.map_or(0, |d| d.size() as u32);
        self.write32(len as i32);
        if let Some(data) = data {
            unsafe { self.writePad(data.data(), len as usize) }
        }
    }
    // Source: src/core/SkWriter32.h:220-222
    pub fn WriteDataSize(data: Option<&SkData>) -> usize {
        4 + ((data.map_or(0, |d| d.size()) + 3) & !3)
    }
    // Source: src/core/SkWriter32.h:228-232
    pub fn rewindToOffset(&mut self, offset: usize) {
        debug_assert_eq!(offset & 3, 0);
        crate::SkASSERT!(@source b"src/core/SkWriter32.h\0".as_ptr(),230; @condition b"offset <= bytesWritten()\0".as_ptr(); offset <= self.bytesWritten());
        self.fUsed = offset;
    }
    // Source: src/core/SkWriter32.h:235-237
    pub unsafe fn flatten(&self, dst: *mut u8) {
        if self.fUsed != 0 {
            unsafe { ptr::copy_nonoverlapping(self.fData, dst, self.fUsed) }
        }
    }
    // Source: src/core/SkWriter32.h:239-241
    pub unsafe fn writeToStream(
        &self,
        stream: core::pin::Pin<&mut dyn crate::src::core::SkStream::SkWStream>,
    ) -> bool {
        unsafe { stream.write(self.fData, self.fUsed) }
    }
    // Source: src/core/SkWriter32.h:245-247
    pub unsafe fn readFromStream(
        &mut self,
        stream: core::pin::Pin<&mut dyn crate::src::core::SkStream::SkStream>,
        length: usize,
    ) -> usize {
        let dst = unsafe { self.reservePad(length) };
        unsafe { stream.read(dst.cast(), length) }
    }
    // Source: src/core/SkWriter32.cpp:18-22
    pub fn writeMatrix(&mut self, matrix: &crate::src::core::SkMatrix::SkMatrix) {
        let size = crate::src::core::SkMatrixPriv::SkMatrixPriv::WriteToMemory(matrix, None);
        debug_assert_eq!(size & 3, 0);
        let p = unsafe { self.reserve(size) };
        let out = unsafe { core::slice::from_raw_parts_mut(p.cast::<u8>(), size) };
        crate::src::core::SkMatrixPriv::SkMatrixPriv::WriteToMemory(matrix, Some(out));
    }
    // Source: src/core/SkWriter32.cpp:24-36
    pub fn writeSampling(
        &mut self,
        s: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
    ) {
        self.write32(s.maxAniso());
        if !s.isAniso() {
            self.writeBool(s.useCubic());
            if s.useCubic() {
                self.writeScalar(s.cubic().B);
                self.writeScalar(s.cubic().C);
            } else {
                self.write32(s.filter() as i32);
                self.write32(s.mipmap() as i32);
            }
        }
    }
    // Source: src/core/SkWriter32.cpp:38-53
    pub unsafe fn writeString(&mut self, mut s: *const core::ffi::c_char, mut len: usize) {
        if s.is_null() {
            s = c"".as_ptr();
            len = 0;
        }
        if (len as isize) < 0 {
            len = unsafe { core::ffi::CStr::from_ptr(s) }.to_bytes().len();
        }
        let p = unsafe { self.reservePad(4 + len + 1) };
        unsafe {
            p.write(len as u32);
            let chars = p.add(1).cast::<u8>();
            if len != 0 {
                ptr::copy_nonoverlapping(s.cast(), chars, len)
            }
            chars.add(len).write(0);
        }
    }
    // Source: src/core/SkWriter32.cpp:55-63
    pub unsafe fn WriteStringSize(s: *const core::ffi::c_char, mut len: usize) -> usize {
        if (len as isize) < 0 {
            debug_assert!(!s.is_null());
            len = unsafe { core::ffi::CStr::from_ptr(s) }.to_bytes().len();
        }
        (4 + len + 1 + 3) & !3
    }
    // Source: src/core/SkWriter32.cpp:65-76
    fn growToAtLeast(&mut self, size: usize) {
        let external = !self.fExternal.is_null() && self.fData == self.fExternal;
        self.fCapacity = 4096 + size.max(self.fCapacity + self.fCapacity / 2);
        self.fInternal.realloc(self.fCapacity);
        self.fData = self.fInternal.data_mut();
        if external && self.fUsed != 0 {
            unsafe { ptr::copy_nonoverlapping(self.fExternal, self.fData, self.fUsed) }
        }
    }
    // Source: src/core/SkWriter32.cpp:78-80
    pub unsafe fn snapshotAsData(&self) -> Arc<SkData> {
        unsafe { SkData::MakeWithCopy(self.fData, self.fUsed) }
    }
}
// Source: src/core/SkWriter32.h:275-280
#[repr(C, align(8))]
struct InlineStorage<const SIZE: usize> {
    bytes: [core::mem::MaybeUninit<u8>; SIZE],
}
// Source: src/core/SkWriter32.h:269-283
pub struct SkSWriter32<const SIZE: usize> {
    // Source: src/core/SkWriter32.h:269-269
    // Keep parent/inline payload field positions; control parent destruction
    // explicitly instead of moving or allocating either original subobject.
    writer: core::mem::ManuallyDrop<SkWriter32>,
    // Source: src/core/SkWriter32.h:275-279
    storage: core::mem::ManuallyDrop<InlineStorage<SIZE>>,
    _pinned: core::marker::PhantomPinned,
}
impl<const SIZE: usize> SkSWriter32<SIZE> {
    // Source: src/core/SkWriter32.h:271-271
    pub fn new() -> core::pin::Pin<Box<Self>> {
        let mut s = Box::pin(Self {
            writer: core::mem::ManuallyDrop::new(unsafe {
                SkWriter32::new(core::ptr::null_mut(), 0)
            }),
            storage: core::mem::ManuallyDrop::new(InlineStorage {
                bytes: [core::mem::MaybeUninit::uninit(); SIZE],
            }),
            _pinned: core::marker::PhantomPinned,
        });
        s.as_mut().reset();
        s
    }
    // Source: src/core/SkWriter32.h:273-273
    pub fn reset(self: core::pin::Pin<&mut Self>) {
        let s = unsafe { self.get_unchecked_mut() };
        unsafe { s.writer.reset(s.storage.bytes.as_mut_ptr().cast(), SIZE) }
    }
    /// Safety: do not move/swap the embedded writer out, change its external lease,
    /// or retain a reference/pointer after this pinned owner is dropped.
    pub unsafe fn writer(self: core::pin::Pin<&mut Self>) -> &mut SkWriter32 {
        unsafe { &mut self.get_unchecked_mut().writer }
    }
}

// Source: src/core/SkWriter32.h:269-281
// Native implicit derived destructor: trivial inline union first, then parent.
// The union has no destructor/owned payload; its explicit trivial drop cannot
// free or move the storage borrowed by the pinned parent. ManuallyDrop is
// transparent and adds neither an owner nor a construction flag.
impl<const SIZE: usize> Drop for SkSWriter32<SIZE> {
    fn drop(&mut self) {
        unsafe {
            core::mem::ManuallyDrop::drop(&mut self.storage);
            core::mem::ManuallyDrop::drop(&mut self.writer);
        }
    }
}
