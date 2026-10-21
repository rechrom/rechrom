//! Native binary writer. Some serializer producers remain explicit dependencies;
//! no missing producer is replaced by an empty encoded result.
use crate::src::core::{
    SkData::SkData,
    SkFlattenable::SkFlattenableView,
    SkMatrix::SkMatrix,
    SkPoint::SkPoint,
    SkPoint3::SkPoint3,
    SkRect::{SkIRect, SkRect},
    SkWriter32::SkWriter32,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
// Source: src/core/SkWriteBuffer.h:43-93
// Native virtual write calls borrow the actual concrete writer; no opaque byte proxy.
pub trait SkWriteBuffer {
    unsafe fn writePad32(&mut self, buffer: *const u8, bytes: usize);
    unsafe fn writeByteArray(&mut self, data: *const u8, size: usize);
    fn writeDataAsByteArray(&mut self, data: Option<&SkData>) {
        if let Some(d) = data {
            unsafe { self.writeByteArray(d.data(), d.size()) }
        } else {
            self.write32(0)
        }
    }
    fn writeBool(&mut self, value: bool);
    fn writeScalar(&mut self, value: f32);
    fn writeScalarArray(&mut self, values: &[f32]);
    fn writeInt(&mut self, value: i32);
    fn writeIntArray(&mut self, values: &[i32]);
    fn writeUInt(&mut self, value: u32);
    fn write32(&mut self, value: i32) {
        self.writeInt(value)
    }
    fn writeString(&mut self, value: &str);
    // Same native byte-string writer; no UTF-8 conversion, substitution or extra owner.
    fn writeStringBytes(&mut self, value: &[u8]);
    // Source: src/core/SkWriteBuffer.h:84-84
    unsafe fn writeStream(
        &mut self,
        stream: core::pin::Pin<&mut dyn crate::src::core::SkStream::SkStream>,
        length: usize,
    ) -> usize;
    // Source: src/core/SkWriteBuffer.h:81-81
    fn writeRegion(&mut self, region: &crate::src::core::SkRegion::SkRegion);
    // Source: src/core/SkWriteBuffer.h:83-83
    fn writePath(&mut self, path: &crate::src::core::SkPath::SkPath);
    // Source: src/core/SkWriteBuffer.h:85-85
    unsafe fn writeImage(&mut self, image: &dyn crate::src::image::SkImage::SkImage);
    // Source: src/core/SkWriteBuffer.h:87-87
    fn writePaint(
        &mut self,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError>;
    // Source: src/core/SkWriteBuffer.h:89-89
    fn serialProcs(&self) -> &crate::include::core::SkSerialProcs::SkSerialProcs;
    fn writeFlattenable(
        &mut self,
        value: Option<&dyn SkFlattenableView>,
    ) -> Result<(), crate::RasterError>;
    fn writeColor(&mut self, color: u32);
    fn writeColor4f(&mut self, color: &crate::src::core::SkColor::SkColor4f);
    fn writeColor4fArray(&mut self, values: &[crate::src::core::SkColor::SkColor4f]);
    fn writeM44(&mut self, matrix: &crate::src::core::SkM44::SkM44);
    // Source: src/core/SkWriteBuffer.h:86-86
    fn writeTypeface(&mut self, typeface: Option<&crate::src::core::SkTypeface::SkTypefaceRef>);
    fn writeColorArray(&mut self, values: &[u32]);
    fn writePoint(&mut self, point: &SkPoint);
    fn writePointArray(&mut self, values: &[SkPoint]);
    fn writePoint3(&mut self, point: &SkPoint3);
    fn writeMatrix(&mut self, matrix: &SkMatrix);
    fn writeIRect(&mut self, rect: &SkIRect);
    fn writeRect(&mut self, rect: &SkRect);
    fn writeSampling(
        &mut self,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
    );
}
// Source: src/core/SkWriteBuffer.h:158-164
// Fields drop in native reverse-member order, followed by the inherited procs.
pub struct SkBinaryWriteBuffer {
    // Source: src/core/SkWriteBuffer.h:164
    fFlattenableDict: HashMap<usize, u32>,
    // Source: src/core/SkWriteBuffer.h:161
    fWriter: SkWriter32,
    // Source: src/core/SkWriteBuffer.h:159-159
    fTFSet: Option<crate::src::core::SkPtrRecorder::SkRefCntSetOwner>,
    // Source: src/core/SkWriteBuffer.h:158-158
    fFactorySet: Option<crate::src::core::SkPtrRecorder::SkFactorySetOwner>,
    // Source: src/core/SkWriteBuffer.h:92 (inherited base, destroyed last)
    fProcs: crate::include::core::SkSerialProcs::SkSerialProcs,
}
impl SkBinaryWriteBuffer {
    // Source: src/core/SkWriteBuffer.cpp:36-37
    pub fn new(procs: crate::include::core::SkSerialProcs::SkSerialProcs) -> Self {
        Self {
            fProcs: procs,
            fFactorySet: None,
            fTFSet: None,
            fWriter: unsafe { SkWriter32::new(core::ptr::null_mut(), 0) },
            fFlattenableDict: HashMap::new(),
        }
    }
    // Source: src/core/SkWriteBuffer.cpp:39-40
    pub unsafe fn with_storage(
        storage: *mut u8,
        size: usize,
        procs: crate::include::core::SkSerialProcs::SkSerialProcs,
    ) -> Self {
        Self {
            fProcs: procs,
            fFactorySet: None,
            fTFSet: None,
            fWriter: unsafe { SkWriter32::new(storage, size) },
            fFlattenableDict: HashMap::new(),
        }
    }
    fn writeNamedFlattenable(
        &mut self,
        name: &str,
        flatten: impl FnOnce(&mut dyn SkWriteBuffer) -> Result<(), crate::RasterError>,
    ) -> Result<(), crate::RasterError> {
        debug_assert!(!name.is_empty());
        let key = name.as_ptr() as usize;
        if let Some(index) = self.fFlattenableDict.get(&key).copied() {
            debug_assert_eq!(index >> 24, 0);
            self.write32((index << 8) as i32);
        } else {
            self.writeString(name);
            let index = self.fFlattenableDict.len() as u32 + 1;
            self.fFlattenableDict.insert(key, index);
        }
        self.writeFlattenablePayload(flatten)
    }
    fn writeFlattenablePayload(
        &mut self,
        flatten: impl FnOnce(&mut dyn SkWriteBuffer) -> Result<(), crate::RasterError>,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            self.fWriter.reserve(4);
        }
        let offset = self.fWriter.bytesWritten();
        flatten(self)?;
        let size = self.fWriter.bytesWritten() - offset;
        unsafe {
            self.fWriter.overwriteTAt(offset - 4, size as u32);
        }
        Ok(())
    }
    // Source: src/core/SkWriteBuffer.cpp:261-263
    /// Safety: until this recorder is replaced or the buffer is dropped, all
    /// accesses to this exact shared payload are externally synchronized with
    /// buffer writes. Recorder operations and final payload destruction must not
    /// reenter the same set; no conflicting payload references may remain live.
    pub unsafe fn setTypefaceRecorder(
        &mut self,
        rec: Option<crate::src::core::SkPtrRecorder::SkRefCntSetOwner>,
    ) {
        // Incoming complete owner moves here; assignment releases the old
        // owner only after incoming retain/creation, matching native sk_sp move.
        self.fTFSet = rec;
    }
    // Source: src/core/SkWriteBuffer.cpp:257-259
    /// Safety: same externally synchronized, nonreentrant operation lease as
    /// setTypefaceRecorder; the owner retains storage, not exclusion.
    pub unsafe fn setFactoryRecorder(
        &mut self,
        rec: Option<crate::src::core::SkPtrRecorder::SkFactorySetOwner>,
    ) {
        // Same complete allocation and single owner count as the caller.
        self.fFactorySet = rec;
    }
    // Source: src/core/SkWriteBuffer.h:89-89
    pub fn serialProcs(&self) -> &crate::include::core::SkSerialProcs::SkSerialProcs {
        &self.fProcs
    }
    // Source: src/core/SkWriteBuffer.h:104-106
    pub unsafe fn write(&mut self, buffer: *const u8, bytes: usize) {
        unsafe { self.fWriter.write(buffer, bytes) }
    }
    // Source: src/core/SkWriteBuffer.h:111-113
    pub unsafe fn reset(&mut self, storage: *mut u8, size: usize) {
        unsafe { self.fWriter.reset(storage, size) }
    }
    // Source: src/core/SkWriteBuffer.h:115-115
    pub fn bytesWritten(&self) -> usize {
        self.fWriter.bytesWritten()
    }
    // Source: src/core/SkWriteBuffer.cpp:44-46
    pub fn usingInitialStorage(&self) -> bool {
        self.fWriter.usingInitialStorage()
    }
    // Source: src/core/SkWriteBuffer.cpp:152-154
    pub unsafe fn writeToStream(
        &self,
        stream: core::pin::Pin<&mut dyn crate::src::core::SkStream::SkWStream>,
    ) -> bool {
        unsafe { self.fWriter.writeToStream(stream) }
    }
    // Source: src/core/SkWriteBuffer.h:151-151
    pub unsafe fn writeToMemory(&self, dst: *mut u8) {
        unsafe { self.fWriter.flatten(dst) }
    }
    // Source: src/core/SkWriteBuffer.h:152-152
    pub fn snapshotAsData(&self) -> Arc<SkData> {
        unsafe { self.fWriter.snapshotAsData() }
    }
}
impl SkWriteBuffer for SkBinaryWriteBuffer {
    // Source: src/core/SkWriteBuffer.h:89-89
    fn serialProcs(&self) -> &crate::include::core::SkSerialProcs::SkSerialProcs {
        &self.fProcs
    }

    // Source: src/core/SkWriteBuffer.h:107-109
    unsafe fn writePad32(&mut self, buffer: *const u8, bytes: usize) {
        unsafe { self.fWriter.writePad(buffer, bytes) }
    }
    // Source: src/core/SkWriteBuffer.cpp:48-51
    unsafe fn writeByteArray(&mut self, data: *const u8, size: usize) {
        self.fWriter.write32(size as i32);
        unsafe { self.fWriter.writePad(data, size) }
    }
    // Source: src/core/SkWriteBuffer.cpp:53-55
    fn writeBool(&mut self, value: bool) {
        self.fWriter.writeBool(value);
    }
    // Source: src/core/SkWriteBuffer.cpp:57-59
    fn writeScalar(&mut self, value: f32) {
        self.fWriter.writeScalar(value)
    }
    // Source: src/core/SkWriteBuffer.cpp:61-64
    fn writeScalarArray(&mut self, values: &[f32]) {
        self.fWriter.write32(values.len() as i32);
        unsafe {
            self.fWriter
                .write(values.as_ptr().cast(), core::mem::size_of_val(values))
        }
    }
    // Source: src/core/SkWriteBuffer.cpp:66-68
    fn writeInt(&mut self, value: i32) {
        self.fWriter.write32(value)
    }
    // Source: src/core/SkWriteBuffer.cpp:70-73
    fn writeIntArray(&mut self, values: &[i32]) {
        self.fWriter.write32(values.len() as i32);
        unsafe {
            self.fWriter
                .write(values.as_ptr().cast(), core::mem::size_of_val(values))
        }
    }
    // Source: src/core/SkWriteBuffer.cpp:75-77
    fn writeUInt(&mut self, value: u32) {
        self.fWriter.write32(value as i32)
    }
    // Source: src/core/SkWriteBuffer.cpp:79-81
    fn writeString(&mut self, value: &str) {
        self.writeStringBytes(value.as_bytes());
    }
    fn writeStringBytes(&mut self, value: &[u8]) {
        unsafe { self.fWriter.writeString(value.as_ptr().cast(), value.len()) }
    }
    // Source: src/core/SkWriteBuffer.cpp:83-85
    fn writeColor(&mut self, color: u32) {
        self.fWriter.write32(color as i32)
    }
    // Source: src/core/SkWriteBuffer.cpp:87-90
    fn writeColorArray(&mut self, values: &[u32]) {
        self.fWriter.write32(values.len() as i32);
        unsafe {
            self.fWriter
                .write(values.as_ptr().cast(), core::mem::size_of_val(values))
        }
    }
    // Source: src/core/SkWriteBuffer.cpp:92-94
    fn writeColor4f(&mut self, color: &crate::src::core::SkColor::SkColor4f) {
        unsafe {
            self.fWriter.write(
                (color as *const crate::src::core::SkColor::SkColor4f).cast(),
                core::mem::size_of_val(color),
            )
        }
    }
    // Source: src/core/SkWriteBuffer.cpp:96-99
    fn writeColor4fArray(&mut self, values: &[crate::src::core::SkColor::SkColor4f]) {
        self.fWriter.write32(values.len() as i32);
        unsafe {
            self.fWriter
                .write(values.as_ptr().cast(), core::mem::size_of_val(values))
        }
    }
    // Source: src/core/SkWriteBuffer.cpp:115-117
    fn writeM44(&mut self, matrix: &crate::src::core::SkM44::SkM44) {
        unsafe {
            self.fWriter.write(
                crate::src::core::SkMatrixPriv::SkMatrixPriv::M44ColMajor(matrix)
                    .as_ptr()
                    .cast(),
                64,
            )
        }
    }
    // Source: src/core/SkWriteBuffer.cpp:226-251
    fn writeTypeface(&mut self, obj: Option<&crate::src::core::SkTypeface::SkTypefaceRef>) {
        if let Some(obj) = obj {
            if let Some(proc) = self.fProcs.fTypefaceProc {
                if let Some(data) = unsafe { proc(obj, self.fProcs.fTypefaceCtx) } {
                    let mut size = data.size();
                    if size > i32::MAX as usize {
                        size = 0;
                    }
                    self.fWriter.write32(-(size as i32));
                    if size != 0 {
                        unsafe { self.writePad32(data.data(), size) }
                    }
                    return;
                }
            }
        } else {
            self.fWriter.write32(0);
        }
        // Source: src/core/SkWriteBuffer.cpp:250-250
        let index = self.fTFSet.as_ref().map_or(0, |set| {
            // Unsafe setter establishes the native external operation lease;
            // only this synchronous add borrows the original complete payload.
            unsafe { (&mut *set.get()).add(obj) }
        });
        self.fWriter.write32(index as i32);
    }
    // Source: src/core/SkWriteBuffer.cpp:101-104
    fn writePoint(&mut self, point: &SkPoint) {
        self.fWriter.writeScalar(point.fX);
        self.fWriter.writeScalar(point.fY);
    }
    // Source: src/core/SkWriteBuffer.cpp:106-108
    fn writePoint3(&mut self, point: &SkPoint3) {
        unsafe {
            self.writePad32(
                (point as *const SkPoint3).cast(),
                core::mem::size_of::<SkPoint3>(),
            )
        }
    }
    // Source: src/core/SkWriteBuffer.cpp:110-113
    fn writePointArray(&mut self, values: &[SkPoint]) {
        self.fWriter.write32(values.len() as i32);
        unsafe {
            self.fWriter
                .write(values.as_ptr().cast(), core::mem::size_of_val(values))
        }
    }
    // Source: src/core/SkWriteBuffer.cpp:119-121
    fn writeMatrix(&mut self, matrix: &SkMatrix) {
        self.fWriter.writeMatrix(matrix)
    }
    // Source: src/core/SkWriteBuffer.cpp:123-125
    fn writeIRect(&mut self, rect: &SkIRect) {
        unsafe {
            self.fWriter.write(
                (rect as *const SkIRect).cast(),
                core::mem::size_of::<SkIRect>(),
            )
        }
    }
    // Source: src/core/SkWriteBuffer.cpp:127-129
    fn writeRect(&mut self, rect: &SkRect) {
        self.fWriter.writeRect(rect)
    }
    // Source: src/core/SkWriteBuffer.cpp:135-137
    fn writeSampling(
        &mut self,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
    ) {
        self.fWriter.writeSampling(sampling)
    }
    // Source: src/core/SkWriteBuffer.cpp:131-133
    fn writeRegion(&mut self, region: &crate::src::core::SkRegion::SkRegion) {
        self.fWriter.writeRegion(region);
    }
    // Source: src/core/SkWriteBuffer.cpp:139-141
    fn writePath(&mut self, path: &crate::src::core::SkPath::SkPath) {
        self.fWriter.writePath(path);
    }
    // Source: src/core/SkWriteBuffer.cpp:204-224
    unsafe fn writeImage(&mut self, image: &dyn crate::src::image::SkImage::SkImage) {
        let mut flags = 0u32;
        let mips = image.asImageBase().onPeekMips();
        if mips.is_some() {
            flags |= SkWriteBufferImageFlags::kHasMipmap;
        }
        if image.alphaType()
            == crate::include::core::SkAlphaType::SkAlphaType::kUnpremul_SkAlphaType
        {
            flags |= SkWriteBufferImageFlags::kUnpremul;
        }
        self.write32(flags as i32);
        let data = unsafe { serialize_image(image, self.fProcs) };
        crate::SkASSERT!(@source b"src/core/SkWriteBuffer.cpp\0".as_ptr(),217; @condition b"data\0".as_ptr(); data.is_some());
        self.writeDataAsByteArray(data.as_deref());
        if flags & SkWriteBufferImageFlags::kHasMipmap != 0 {
            let mip_data = unsafe {
                serialize_mipmap(mips.expect("native non-null mipmap flag"), self.fProcs)
            };
            self.writeDataAsByteArray(mip_data.as_deref());
        }
    }
    // Source: src/core/SkWriteBuffer.cpp:253-255
    fn writePaint(
        &mut self,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        crate::src::core::SkPaintPriv::Flatten(paint, self)
    }
    // Source: src/core/SkWriteBuffer.cpp:143-150
    unsafe fn writeStream(
        &mut self,
        stream: core::pin::Pin<&mut dyn crate::src::core::SkStream::SkStream>,
        length: usize,
    ) -> usize {
        self.fWriter.write32(length as i32);
        let written = unsafe { self.fWriter.readFromStream(stream, length) };
        if written < length {
            unsafe {
                self.fWriter.reservePad(length - written);
            }
        }
        written
    }
    // Source: src/core/SkWriteBuffer.cpp:265-314
    fn writeFlattenable(
        &mut self,
        flattenable: Option<&dyn SkFlattenableView>,
    ) -> Result<(), crate::RasterError> {
        let Some(flattenable) = flattenable else {
            self.write32(0);
            return Ok(());
        };
        if let (Some(factory), Some(recorder)) =
            (flattenable.getFactory(), self.fFactorySet.as_ref())
        {
            // Source: src/core/SkWriteBuffer.cpp:284-284
            // End the actual set borrow before flatten callbacks execute.
            let index = unsafe { (&mut *recorder.get()).add(factory) };
            self.write32(index as i32);
            self.writeFlattenablePayload(|writer| flattenable.flatten(writer))?;
        } else {
            self.writeNamedFlattenable(
                flattenable
                    .getTypeName()
                    .expect("native flattenable type name"),
                |writer| flattenable.flatten(writer),
            )?;
        }
        Ok(())
    }
}

// Source: src/core/SkWriteBuffer.h:167-174
pub mod SkWriteBufferImageFlags {
    pub const kVersion_bits: u32 = 8;
    pub const kCurrVersion: u32 = 0;
    pub const kHasSubsetRect: u32 = 1 << 8;
    pub const kHasMipmap: u32 = 1 << 9;
    pub const kUnpremul: u32 = 1 << 10;
}
// Source: src/core/SkWriteBuffer.cpp:156-171
unsafe fn serialize_image(
    image: &dyn crate::src::image::SkImage::SkImage,
    procs: crate::include::core::SkSerialProcs::SkSerialProcs,
) -> Option<Arc<SkData>> {
    let mut data = None;
    if let Some(proc) = procs.fImageProc {
        data = unsafe { proc(image, procs.fImageCtx) };
    }
    if data.is_some() {
        return data;
    }
    data = image.refEncodedData();
    if data.is_some() {
        return data;
    }
    None
}
// Source: src/core/SkWriteBuffer.cpp:173-196
unsafe fn serialize_mipmap(
    mipmap: &crate::src::core::SkMipmap::SkMipmap,
    procs: crate::include::core::SkSerialProcs::SkSerialProcs,
) -> Option<Arc<SkData>> {
    let count = mipmap.countLevels();
    let mut buffer = SkBinaryWriteBuffer::new(Default::default());
    buffer.write32(count);
    for i in 0..count {
        let mut level = crate::src::core::SkMipmap::Level {
            fPixmap: crate::src::core::SkPixmap::SkPixmap::default(),
            fScale: crate::include::core::SkSize::SkSize::Make(0.0, 0.0),
        };
        if mipmap.getLevel(i, Some(&mut level)) {
            let image = unsafe {
                crate::src::image::SkImage_RasterFactories::RasterFromPixmap(
                    &level.fPixmap,
                    None,
                    core::ptr::null_mut(),
                )
            };
            let data = unsafe {
                serialize_image(
                    image.as_deref().expect("native valid mip level image"),
                    procs,
                )
            };
            buffer.writeDataAsByteArray(data.as_deref());
        } else {
            return None;
        }
    }
    Some(buffer.snapshotAsData())
}
