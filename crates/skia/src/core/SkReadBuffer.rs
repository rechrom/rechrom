// Copyright 2012 Google LLC. BSD-style license.
use super::SkFlattenable::{Factory, FlattenableOwner, Type};
use crate::include::core::SkRefCnt::sk_sp;
use crate::include::core::SkSerialProcs::SkDeserialProcs;
use std::marker::PhantomData;
// Source: src/core/SkReadBuffer.h:260-285
// Native borrowed pointers retain their original storage/array leases through 'a.
// Private Rust representation follows native reverse member destruction.
// Constructor expression order remains the original named initializer order.
pub struct SkReadBuffer<'a> {
    lease: PhantomData<&'a [u8]>,
    // Source: src/core/SkReadBuffer.h:285
    fError: bool,
    // Source: src/core/SkReadBuffer.h:284
    fAllowSkSL: bool,
    // Source: src/core/SkReadBuffer.h:282
    fRecursionLimit: i32,
    // Source: src/core/SkReadBuffer.h:276
    fProcs: SkDeserialProcs,
    // Source: src/core/SkReadBuffer.h:274
    fFactoryCount: i32,
    // Source: src/core/SkReadBuffer.h:273
    fFactoryArray: Option<&'a [Option<Factory>]>,
    // Source: src/core/SkReadBuffer.h:271
    fTFCount: i32,
    // Source: src/core/SkReadBuffer.h:270
    fTFArray: Option<&'a [Option<super::SkTypeface::SkTypefaceRef>]>,
    // Source: src/core/SkReadBuffer.h:268
    fVersion: i32,
    // Source: src/core/SkReadBuffer.h:265
    fFlattenableDict: std::collections::HashMap<u32, Option<Factory>>,
    // Source: src/core/SkReadBuffer.h:262
    fBase: *const u8,
    // Source: src/core/SkReadBuffer.h:261
    fStop: *const u8,
    // Source: src/core/SkReadBuffer.h:260
    fCurr: *const u8,
}
// Source: src/core/SkReadBuffer.h:57-57
impl Default for SkReadBuffer<'_> {
    fn default() -> Self {
        Self {
            fCurr: std::ptr::null(),
            fStop: std::ptr::null(),
            fBase: std::ptr::null(),
            fFlattenableDict: Default::default(),
            fVersion: 0,
            fTFArray: None,
            fTFCount: 0,
            fFactoryArray: None,
            fFactoryCount: 0,
            fProcs: Default::default(),
            fRecursionLimit: 100,
            fAllowSkSL: true,
            fError: false,
            lease: PhantomData,
        }
    }
}
impl<'a> SkReadBuffer<'a> {
    // Source: src/core/SkReadBuffer.h:58-60
    pub fn new(data: &'a [u8]) -> Self {
        let mut b = Self::default();
        b.setMemory(data);
        b
    }
    // Source: src/core/SkReadBuffer.h:67-70
    pub fn isVersionLT(&self, target: i32) -> bool {
        debug_assert!(target > 0);
        self.fVersion > 0 && self.fVersion < target
    }
    // Source: src/core/SkReadBuffer.h:72-72
    pub fn getVersion(&self) -> u32 {
        self.fVersion as u32
    }
    // Source: src/core/SkReadBuffer.h:75-78
    pub fn setVersion(&mut self, version: i32) {
        debug_assert!(self.fVersion == 0 || version == self.fVersion);
        self.fVersion = version;
    }
    // Source: src/core/SkReadBuffer.h:80-82
    pub fn size(&self) -> usize {
        (self.fStop as usize).wrapping_sub(self.fBase as usize)
    }
    pub fn offset(&self) -> usize {
        (self.fCurr as usize).wrapping_sub(self.fBase as usize)
    }
    pub fn eof(&self) -> bool {
        self.fCurr >= self.fStop
    }
    // Source: src/core/SkReadBuffer.h:85-85
    pub fn available(&self) -> usize {
        (self.fStop as usize).wrapping_sub(self.fCurr as usize)
    }
    // Source: src/core/SkReadBuffer.h:87-89
    // Native pointer return: this does not form an aligned/initialized Rust T
    // reference. The retained input lease bounds readability; callers remain
    // responsible for T alignment, validity and unaligned field projections.
    pub fn skipT<T>(&mut self) -> *const T {
        self.skip(size_of::<T>())
            .map_or(std::ptr::null(), |bytes| bytes.as_ptr().cast())
    }
    // Source: src/core/SkReadBuffer.h:90-92
    pub fn skipT_count<T>(&mut self, count: usize) -> *const T {
        self.skipCount(count, size_of::<T>())
            .map_or(std::ptr::null(), |bytes| bytes.as_ptr().cast())
    }
    // Source: src/core/SkReadBuffer.h:102-108
    pub fn read32LE(&mut self, max: u32) -> u32 {
        let mut value = self.readUInt();
        if !self.validate(value <= max) {
            value = 0;
        }
        value
    }
    // Source: src/core/SkReadBuffer.h:130-132
    pub fn readPaint(&mut self) -> Result<super::SkPaint::SkPaint, crate::RasterError> {
        super::SkPaintPriv::Unflatten(self)
    }
    // Source: src/core/SkReadBuffer.h:139-139
    pub fn readColorFilter(
        &mut self,
    ) -> Result<Option<std::sync::Arc<dyn super::SkColorFilter::SkColorFilter>>, crate::RasterError>
    {
        Ok(self
            .readFlattenable(Type::kSkColorFilter_Type)?
            .map(|owner| {
                let FlattenableOwner::ColorFilter(owner) = owner else {
                    panic!("checked native child requires its actual retained owner variant");
                };
                owner
            }))
    }
    // Source: src/core/SkReadBuffer.h:140-140
    pub fn readImageFilter(
        &mut self,
    ) -> Result<Option<std::sync::Arc<dyn super::SkImageFilter::SkImageFilter>>, crate::RasterError>
    {
        Ok(self
            .readFlattenable(Type::kSkImageFilter_Type)?
            .map(|owner| {
                let FlattenableOwner::ImageFilter(owner) = owner else {
                    panic!("checked native child requires its actual retained owner variant");
                };
                owner
            }))
    }
    // Source: src/core/SkReadBuffer.h:141-141
    pub fn readBlender(
        &mut self,
    ) -> Result<sk_sp<dyn crate::include::core::SkBlender::SkBlender>, crate::RasterError> {
        Ok(match self.readFlattenable(Type::kSkBlender_Type)? {
            Some(FlattenableOwner::Blender(owner)) => owner,
            None => sk_sp::default(),
            _ => panic!("checked native blender requires its actual intrusive owner"),
        })
    }
    // Source: src/core/SkReadBuffer.h:142-142
    pub fn readMaskFilter(
        &mut self,
    ) -> Result<Option<std::sync::Arc<dyn super::SkMaskFilter::SkMaskFilter>>, crate::RasterError>
    {
        Ok(self
            .readFlattenable(Type::kSkMaskFilter_Type)?
            .map(|owner| {
                let FlattenableOwner::MaskFilter(owner) = owner else {
                    panic!("checked native child requires its actual retained owner variant");
                };
                owner
            }))
    }
    // Source: src/core/SkReadBuffer.h:143-143
    pub fn readPathEffect(
        &mut self,
    ) -> Result<sk_sp<dyn super::SkPathEffect::SkPathEffect>, crate::RasterError> {
        let Some(mut obj) = self.readFlattenable(Type::kSkPathEffect_Type)? else {
            return Ok(sk_sp::default());
        };
        let FlattenableOwner::Intrusive(mut obj) = obj else {
            panic!("checked path-effect factory requires its actual intrusive owner");
        };
        let owner = obj.as_ptr().expect("factory returned live owner");
        let projected = unsafe { owner.as_ref().project_path_effect_owner(owner.cast()) }
            .expect("checked native path-effect type requires actual projection");
        obj.release();
        Ok(unsafe { sk_sp::from_raw(projected.as_ptr()) })
    }
    // Source: src/core/SkReadBuffer.h:144-144
    pub fn readShader(
        &mut self,
    ) -> Result<
        Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>,
        crate::RasterError,
    > {
        Ok(self.readFlattenable(Type::kSkShader_Type)?.map(|owner| {
            let FlattenableOwner::Shader(owner) = owner else {
                panic!("checked native shader requires its actual Arc owner");
            };
            owner
        }))
    }
    // Source: src/core/SkReadBuffer.h:170-173
    pub fn setTypefaceArray(&mut self, array: &'a [Option<super::SkTypeface::SkTypefaceRef>]) {
        self.fTFArray = Some(array);
        self.fTFCount = array.len() as i32;
    }
    // Source: src/core/SkReadBuffer.h:179-182
    pub fn setFactoryPlayback(&mut self, array: &'a [Option<Factory>]) {
        self.fFactoryArray = Some(array);
        self.fFactoryCount = array.len() as i32;
    }
    // Source: src/core/SkReadBuffer.h:185-185
    pub fn getDeserialProcs(&self) -> &SkDeserialProcs {
        &self.fProcs
    }
    // Source: src/core/SkReadBuffer.h:187-188
    pub fn allowSkSL(&self) -> bool {
        self.fAllowSkSL
    }
    pub fn setAllowSkSL(&mut self, allow: bool) {
        self.fAllowSkSL = allow;
    }
    // Source: src/core/SkReadBuffer.h:190-195
    pub fn allowTags(&self, tag: u32) -> bool {
        let Some(proc) = self.fProcs.fAllowTagsProc else {
            return true;
        };
        unsafe { proc(tag, self.fProcs.fAllowTagsCtx) }
    }
    // Source: src/core/SkReadBuffer.h:201-206
    pub fn validate(&mut self, valid: bool) -> bool {
        if !valid {
            self.setInvalid();
        }
        !self.fError
    }
    // Source: src/core/SkReadBuffer.h:213-216
    pub fn validateCanReadN<T>(&mut self, n: usize) -> bool {
        self.validate(n <= self.available() / size_of::<T>())
    }
    // Source: src/core/SkReadBuffer.h:218-221
    pub fn isValid(&self) -> bool {
        !self.fError
    }
    pub fn validateIndex(&mut self, index: i32, count: i32) -> bool {
        self.validate(index >= 0 && index < count)
    }
    // Source: src/core/SkReadBuffer.h:229-231
    /// Safety: every integer in the requested range must be a defined value of T;
    /// native conversion of both bounds to int must be defined. No enum fabricated.
    pub unsafe fn checkRange<T: SkReadBufferRange>(&mut self, min: T, max: T) -> T {
        T::from_i32(self.checkInt(min.into_i32(), max.into_i32()))
    }
    // Source: src/core/SkReadBuffer.h:240-243
    pub fn downLevel(&mut self) {
        self.fRecursionLimit -= 1;
        self.validate(self.fRecursionLimit > 0);
    }
    // Source: src/core/SkReadBuffer.h:244-247
    pub fn upLevel(&mut self) {
        self.fRecursionLimit += 1;
    }
    // Source: src/core/SkReadBuffer.h:248-251
    pub fn setRecursionLimit(&mut self, limit: i32) {
        self.fRecursionLimit = limit;
        self.validate(limit > 0);
    }
    // Source: src/core/SkReadBuffer.h:258-258
    pub fn isAvailable(&self, size: usize) -> bool {
        size <= self.available()
    }
    // Source: src/core/SkReadBuffer.h:278-280
    // (Fields are grouped above; the native pointer-alignment helper stays here.)
    fn is_align4(p: *const u8) -> bool {
        p as usize & 3 == 0
    }
    // Source: src/core/SkReadBuffer.cpp:53-59
    pub fn setMemory(&mut self, data: &'a [u8]) {
        self.validate(
            Self::is_align4(data.as_ptr()) && data.len().wrapping_add(3) & !3 == data.len(),
        );
        if !self.fError {
            self.fBase = data.as_ptr();
            self.fCurr = self.fBase;
            self.fStop = self.fBase.wrapping_add(data.len());
        }
    }
    // Source: src/core/SkReadBuffer.cpp:61-67
    fn setInvalid(&mut self) {
        if !self.fError {
            self.fCurr = self.fStop;
            self.fError = true;
        }
    }
    // Source: src/core/SkReadBuffer.cpp:69-80
    pub fn skip(&mut self, size: usize) -> Option<&'a [u8]> {
        let inc = size.wrapping_add(3) & !3;
        self.validate(inc >= size);
        let addr = self.fCurr;
        self.validate(Self::is_align4(addr) && inc <= self.available());
        if self.fError {
            return None;
        }
        self.fCurr = self.fCurr.wrapping_add(inc);
        if addr.is_null() {
            return None;
        }
        if size == 0 {
            return Some(&[]);
        }
        Some(unsafe { std::slice::from_raw_parts(addr, size) })
    }
    // Source: src/core/SkReadBuffer.cpp:82-84
    pub fn skipCount(&mut self, count: usize, size: usize) -> Option<&'a [u8]> {
        self.skip(count.checked_mul(size).unwrap_or(usize::MAX))
    }
    // Source: src/core/SkReadBuffer.cpp:86-89
    pub fn setDeserialProcs(&mut self, procs: SkDeserialProcs) {
        self.fProcs = procs;
        self.setAllowSkSL(self.fProcs.fAllowSkSL);
    }
    // Source: src/core/SkReadBuffer.cpp:91-96
    pub fn readBool(&mut self) -> bool {
        let value = self.readUInt();
        self.validate(value & !1 == 0);
        value != 0
    }
    // Source: src/core/SkReadBuffer.cpp:98-100
    pub fn readColor(&mut self) -> u32 {
        self.readUInt()
    }
    // Source: src/core/SkReadBuffer.cpp:102-110
    pub fn readInt(&mut self) -> i32 {
        if !self.validate(Self::is_align4(self.fCurr) && self.available() >= 4) {
            return 0;
        }
        let value = unsafe { self.fCurr.cast::<i32>().read() };
        self.fCurr = self.fCurr.wrapping_add(4);
        value
    }
    // Source: src/core/SkReadBuffer.cpp:112-120
    pub fn readScalar(&mut self) -> f32 {
        if !self.validate(Self::is_align4(self.fCurr) && self.available() >= 4) {
            return 0.0;
        }
        let value = unsafe { self.fCurr.cast::<f32>().read() };
        self.fCurr = self.fCurr.wrapping_add(4);
        value
    }
    // Source: src/core/SkReadBuffer.cpp:122-124
    pub fn readUInt(&mut self) -> u32 {
        self.readInt() as u32
    }
    // Source: src/core/SkReadBuffer.cpp:126-128
    pub fn read32(&mut self) -> i32 {
        self.readInt()
    }
    // Source: src/core/SkReadBuffer.cpp:130-136
    pub fn peekByte(&mut self) -> u8 {
        if self.available() == 0 {
            self.fError = true;
            return 0;
        }
        unsafe { self.fCurr.read() }
    }
    // Source: src/core/SkReadBuffer.cpp:138-146
    /// output is writable for len bytes; it may be uninitialized arena storage.
    /// Its write lease is exclusive and does not overlap the serialized input.
    pub unsafe fn readPad32_raw(&mut self, output: *mut u8, len: usize) -> bool {
        let Some(src) = self.skip(len) else {
            return false;
        };
        if len != 0 {
            unsafe {
                core::ptr::copy_nonoverlapping(src.as_ptr(), output, len);
            }
        }
        true
    }
    pub fn readPad32(&mut self, output: &mut [u8]) -> bool {
        unsafe { self.readPad32_raw(output.as_mut_ptr(), output.len()) }
    }
    // Source: src/core/SkReadBuffer.cpp:148-158
    pub fn readStringBytes(&mut self) -> Option<&'a [u8]> {
        let len = self.readUInt() as usize;
        let text = self.skipCount(len.wrapping_add(1), 1);
        if self.validate(text.is_some_and(|t| t[len] == 0)) {
            text.map(|t| &t[..len])
        } else {
            None
        }
    }
    // Source: src/core/SkReadBuffer.cpp:160-167
    pub fn readString(&mut self, output: &mut Vec<u8>) {
        output.clear();
        if let Some(bytes) = self.readStringBytes() {
            output.extend_from_slice(bytes);
        }
    }
    // Source: src/core/SkReadBuffer.cpp:169-173
    pub fn readColor4f(&mut self, color: &mut super::SkColor::SkColor4f) {
        let bytes = unsafe {
            std::slice::from_raw_parts_mut(
                (color as *mut super::SkColor::SkColor4f).cast(),
                size_of::<super::SkColor::SkColor4f>(),
            )
        };
        if !self.readPad32(bytes) {
            *color = super::SkColor::SkColor4f {
                fR: 0.0,
                fG: 0.0,
                fB: 0.0,
                fA: 0.0,
            };
        }
    }
    // Source: src/core/SkReadBuffer.h:118-118
    pub fn readPoint(&mut self) -> super::SkPoint::SkPoint {
        let mut point = super::SkPoint::SkPoint { fX: 0.0, fY: 0.0 };
        self.readPointInto(&mut point);
        point
    }
    // Source: src/core/SkReadBuffer.cpp:175-178
    pub fn readPointInto(&mut self, point: &mut super::SkPoint::SkPoint) {
        point.fX = self.readScalar();
        point.fY = self.readScalar();
    }
    // Source: src/core/SkReadBuffer.cpp:180-182
    pub fn readPoint3(&mut self, point: &mut super::SkPoint3::SkPoint3) {
        let bytes = unsafe {
            std::slice::from_raw_parts_mut(
                (point as *mut super::SkPoint3::SkPoint3).cast(),
                size_of::<super::SkPoint3::SkPoint3>(),
            )
        };
        self.readPad32(bytes);
    }
    // Source: src/core/SkReadBuffer.cpp:184-193
    pub fn readM44(&mut self, matrix: &mut super::SkM44::SkM44) {
        if self.isValid() {
            if let Some(bytes) = self.skip(64) {
                let mut values = [0.0; 16];
                for (i, dst) in values.iter_mut().enumerate() {
                    *dst = f32::from_ne_bytes(bytes[i * 4..i * 4 + 4].try_into().unwrap());
                }
                *matrix = super::SkM44::SkM44::ColMajor(&values);
            }
        }
        if !self.isValid() {
            *matrix = super::SkM44::SkM44::default();
        }
    }
    // Source: src/core/SkReadBuffer.cpp:195-205
    pub fn readMatrix(&mut self, matrix: &mut super::SkMatrix::SkMatrix) {
        let mut size = 0;
        if self.isValid() {
            let bytes = if self.available() == 0 {
                &[]
            } else {
                unsafe { std::slice::from_raw_parts(self.fCurr, self.available()) }
            };
            size = super::SkMatrixPriv::SkMatrixPriv::ReadFromMemory(matrix, bytes);
            self.validate(size.wrapping_add(3) & !3 == size && size != 0);
        }
        if !self.isValid() {
            matrix.reset();
        }
        self.skip(size);
    }
    // Source: src/core/SkReadBuffer.cpp:207-211
    pub fn readIRect(&mut self, rect: &mut super::SkRect::SkIRect) {
        let bytes = unsafe {
            std::slice::from_raw_parts_mut(
                (rect as *mut super::SkRect::SkIRect).cast(),
                size_of::<super::SkRect::SkIRect>(),
            )
        };
        if !self.readPad32(bytes) {
            rect.set_empty();
        }
    }
    // Source: src/core/SkReadBuffer.cpp:213-217
    pub fn readRectInto(&mut self, rect: &mut super::SkRect::SkRect) {
        let bytes = unsafe {
            std::slice::from_raw_parts_mut(
                (rect as *mut super::SkRect::SkRect).cast(),
                size_of::<super::SkRect::SkRect>(),
            )
        };
        if !self.readPad32(bytes) {
            rect.setEmpty();
        }
    }
    // Source: src/core/SkReadBuffer.cpp:219-225
    pub fn readRect(&mut self) -> super::SkRect::SkRect {
        let mut rect = super::SkRect::SkRect::default();
        self.readRectInto(&mut rect);
        rect
    }
    // Source: src/core/SkReadBuffer.cpp:227-243
    pub fn readSampling(&mut self) -> crate::include::core::SkSamplingOptions::SkSamplingOptions {
        use crate::include::core::SkSamplingOptions::{
            SkCubicResampler, SkFilterMode, SkMipmapMode, SkSamplingOptions,
        };
        if !self.isVersionLT(super::SkPicturePriv::kAnisotropicFilter) {
            let max_aniso = self.readInt();
            if max_aniso != 0 {
                return SkSamplingOptions::Aniso(max_aniso);
            }
        }
        if self.readBool() {
            let b = self.readScalar();
            let c = self.readScalar();
            SkSamplingOptions::with_cubic(SkCubicResampler { B: b, C: c })
        } else {
            let filter = self.read32LE(SkFilterMode::kLinear as u32);
            let mipmap = self.read32LE(SkMipmapMode::kLinear as u32);
            let filter = if filter == 0 {
                SkFilterMode::kNearest
            } else {
                SkFilterMode::kLinear
            };
            let mipmap = match mipmap {
                0 => SkMipmapMode::kNone,
                1 => SkMipmapMode::kNearest,
                _ => SkMipmapMode::kLinear,
            };
            SkSamplingOptions::with_filter_mipmap(filter, mipmap)
        }
    }
    // Source: src/core/SkReadBuffer.cpp:245-254
    pub fn readRRect(&mut self, rrect: &mut super::SkRRect::SkRRect) {
        let mut size = 0;
        if !self.fError {
            let bytes = if self.available() == 0 {
                &[]
            } else {
                unsafe { std::slice::from_raw_parts(self.fCurr, self.available()) }
            };
            size = rrect.readFromMemory(bytes);
            if !self.validate(size.wrapping_add(3) & !3 == size && size != 0) {
                rrect.setEmpty();
            }
        }
        self.skip(size);
    }
    // Source: src/core/SkReadBuffer.h:126-126
    // Source: src/core/SkReadBuffer.cpp:256-265
    pub fn readRegion(&mut self, region: &mut super::SkRegion::SkRegion) {
        let mut size = 0usize;
        if !self.fError {
            size = unsafe { region.readFromMemory(self.fCurr, self.available()) };
            if !self.validate(size.wrapping_add(3) & !3 == size && size != 0) {
                region.set_empty();
            }
        }
        let _ = self.skip(size);
    }
    // Source: src/core/SkReadBuffer.cpp:286-290
    unsafe fn readArray_raw(&mut self, output: *mut u8, count: usize, element_size: usize) -> bool {
        let read_count = self.readUInt() as usize;
        if !self.validate(count == read_count) {
            return false;
        }
        let Some(bytes) = count.checked_mul(element_size) else {
            self.skip(usize::MAX);
            return false;
        };
        unsafe { self.readPad32_raw(output, bytes) }
    }
    fn readArray(&mut self, output: &mut [u8], count: usize, element_size: usize) -> bool {
        debug_assert_eq!(count.checked_mul(element_size), Some(output.len()));
        unsafe { self.readArray_raw(output.as_mut_ptr(), count, element_size) }
    }
    // Source: src/core/SkReadBuffer.cpp:292-294
    /// output addresses len exclusively writable bytes, including uninitialized
    /// arena storage. No Rust reference to those bytes is formed before writing.
    pub unsafe fn readByteArray_raw(&mut self, output: *mut u8, len: usize) -> bool {
        unsafe { self.readArray_raw(output, len, 1) }
    }
    pub fn readByteArray(&mut self, output: &mut [u8]) -> bool {
        unsafe { self.readByteArray_raw(output.as_mut_ptr(), output.len()) }
    }
    // Source: src/core/SkReadBuffer.cpp:296-298
    pub fn readColorArray(&mut self, values: &mut [u32]) -> bool {
        let count = values.len();
        let bytes = unsafe {
            std::slice::from_raw_parts_mut(
                values.as_mut_ptr().cast::<u8>(),
                std::mem::size_of_val(values),
            )
        };
        self.readArray(bytes, count, size_of::<u32>())
    }
    // Source: src/core/SkReadBuffer.cpp:300-302
    pub fn readColor4fArray(&mut self, values: &mut [super::SkColor::SkColor4f]) -> bool {
        let count = values.len();
        let bytes = unsafe {
            std::slice::from_raw_parts_mut(
                values.as_mut_ptr().cast::<u8>(),
                std::mem::size_of_val(values),
            )
        };
        self.readArray(bytes, count, size_of::<super::SkColor::SkColor4f>())
    }
    // Source: src/core/SkReadBuffer.cpp:304-306
    pub fn readIntArray(&mut self, values: &mut [i32]) -> bool {
        let count = values.len();
        let bytes = unsafe {
            std::slice::from_raw_parts_mut(
                values.as_mut_ptr().cast::<u8>(),
                std::mem::size_of_val(values),
            )
        };
        self.readArray(bytes, count, size_of::<i32>())
    }
    // Source: src/core/SkReadBuffer.cpp:308-310
    pub fn readPointArray(&mut self, values: &mut [super::SkPoint::SkPoint]) -> bool {
        let count = values.len();
        let bytes = unsafe {
            std::slice::from_raw_parts_mut(
                values.as_mut_ptr().cast::<u8>(),
                std::mem::size_of_val(values),
            )
        };
        self.readArray(bytes, count, size_of::<super::SkPoint::SkPoint>())
    }
    // Source: src/core/SkReadBuffer.cpp:312-314
    pub fn readScalarArray(&mut self, values: &mut [f32]) -> bool {
        let count = values.len();
        let bytes = unsafe {
            std::slice::from_raw_parts_mut(
                values.as_mut_ptr().cast::<u8>(),
                std::mem::size_of_val(values),
            )
        };
        self.readArray(bytes, count, size_of::<f32>())
    }
    // Source: src/core/SkReadBuffer.cpp:316-323
    pub fn skipByteArray(&mut self) -> (Option<&'a [u8]>, usize) {
        let count = self.readUInt() as usize;
        let buf = self.skip(count);
        (buf, if self.isValid() { count } else { 0 })
    }
    // Source: src/core/SkReadBuffer.cpp:325-336
    pub fn readByteArrayAsData(&mut self) -> Option<std::sync::Arc<super::SkData::SkData>> {
        let n = self.getArrayCount() as usize;
        if !self.validate(n <= self.available()) {
            return None;
        }
        let mut bytes = vec![0; n];
        if !self.readByteArray(&mut bytes) {
            return None;
        }
        Some(unsafe { super::SkData::SkData::MakeWithCopy(bytes.as_ptr(), bytes.len()) })
    }
    // Source: src/core/SkReadBuffer.cpp:338-345
    pub fn getArrayCount(&mut self) -> u32 {
        if !self.validate(Self::is_align4(self.fCurr) && self.available() >= 4) {
            return 0;
        }
        unsafe { self.fCurr.cast::<u32>().read() }
    }
    // Source: src/core/SkReadBuffer.cpp:468-536
    pub fn readRawFlattenable(&mut self) -> Result<Option<FlattenableOwner>, crate::RasterError> {
        if !self.isValid() {
            return Ok(None);
        }
        let mut factory: Option<Factory> = None;
        if self.fFactoryCount > 0 {
            let array = self
                .fFactoryArray
                .expect("native factory-count requires live playback array");
            let mut index = self.read32();
            if index == 0 || !self.isValid() {
                return Ok(None);
            }
            if index < 0 {
                self.validate(false);
                return Ok(None);
            }
            index -= 1;
            if index as u32 >= self.fFactoryCount as u32 {
                self.validate(false);
                return Ok(None);
            }
            factory = array[index as usize];
        } else {
            if self.peekByte() != 0 {
                if let Some(name) = self.readStringBytes() {
                    factory = super::SkFlattenable::NameToFactory(name);
                    self.fFlattenableDict
                        .insert(self.fFlattenableDict.len() as u32 + 1, factory);
                }
            } else {
                let index = self.readUInt() >> 8;
                if index == 0 {
                    return Ok(None);
                }
                if let Some(found) = self.fFlattenableDict.get(&index) {
                    factory = *found;
                }
            }
            if !self.validate(factory.is_some()) {
                return Ok(None);
            }
        }
        let size_recorded = self.read32() as u32;
        let obj = if let Some(factory) = factory {
            let offset = self.offset();
            let obj = factory(self)?;
            let size_read = self.offset().wrapping_sub(offset);
            if size_recorded as usize != size_read {
                self.validate(false);
                return Ok(None);
            }
            obj
        } else {
            self.skip(size_recorded as usize);
            None
        };
        if !self.isValid() {
            return Ok(None);
        }
        Ok(obj)
    }
    // Source: src/core/SkReadBuffer.cpp:538-546
    pub fn readFlattenable(
        &mut self,
        expected: Type,
    ) -> Result<Option<FlattenableOwner>, crate::RasterError> {
        let obj = self.readRawFlattenable()?;
        if obj
            .as_ref()
            .is_some_and(|o| o.as_view().getFlattenableType() != expected)
        {
            self.validate(false);
            return Ok(None);
        }
        Ok(obj)
    }
    // Source: src/core/SkReadBuffer.cpp:550-558
    pub fn checkInt(&mut self, min: i32, max: i32) -> i32 {
        crate::SkASSERT!(@source b"src/core/SkReadBuffer.cpp\0".as_ptr(),551; @condition b"min <= max\0".as_ptr(); min <= max);
        let mut value = self.read32();
        if value < min || value > max {
            self.validate(false);
            value = min;
        }
        value
    }
}

impl SkReadBuffer<'_> {
    // Source: src/core/SkReadBuffer.cpp:560-562
    pub fn checkFilterQuality(&mut self) -> super::SkSamplingPriv::SkLegacyFQ {
        unsafe {
            self.checkRange(
                super::SkSamplingPriv::SkLegacyFQ::kNone_SkLegacyFQ,
                super::SkSamplingPriv::SkLegacyFQ::kHigh_SkLegacyFQ,
            )
        }
    }
}

impl SkReadBuffer<'_> {
    // Source: src/core/SkReadBuffer.cpp:267-284
    pub fn readPath(&mut self) -> Option<super::SkPath::SkPath> {
        if self.fError {
            return None;
        }
        let mut size = 0;
        let path = unsafe {
            super::SkPath::SkPath::ReadFromMemory(self.fCurr, self.available(), Some(&mut size))
        };
        self.validate((size.wrapping_add(3) & !3) == size && path.is_some());
        self.skip(size);
        path
    }
}

// Source: src/core/SkReadBuffer.cpp:346-356
// The actual registered deserializer owns the image. No decoder or fallback
// image is fabricated when the native proc slots are unset.
unsafe fn deserialize_image(
    data: std::sync::Arc<super::SkData::SkData>,
    procs: SkDeserialProcs,
    alpha_type: Option<crate::include::core::SkAlphaType::SkAlphaType>,
) -> Option<std::sync::Arc<dyn crate::src::image::SkImage::SkImage>> {
    let mut image = None;
    if let Some(proc) = procs.fImageDataProc {
        image = unsafe { proc(data, alpha_type, procs.fImageCtx) };
    } else if let Some(proc) = procs.fImageProc {
        image = unsafe { proc(data.data(), data.size(), alpha_type, procs.fImageCtx) };
    }
    image
}

impl SkReadBuffer<'_> {
    // Source: src/core/SkReadBuffer.cpp:443-466
    pub fn readTypeface(&mut self) -> Option<super::SkTypeface::SkTypefaceRef> {
        let index = self.read32();
        if index == 0 {
            return None;
        }
        if index > 0 {
            if !self.validate(index <= self.fTFCount) {
                return None;
            }
            return self
                .fTFArray
                .expect("native validated nonempty typeface playback")[(index - 1) as usize]
                .clone();
        }
        // Widen before negation: native sk_negate_to_size_t accepts INT32_MIN.
        let size = (-(index as i64)) as usize;
        let data = self.skip(size);
        if !self.validate(data.is_some() && self.fProcs.fTypefaceStreamProc.is_some()) {
            return None;
        }
        let data = data.expect("native validated serialized typeface bytes");
        let mut stream = core::pin::pin!(unsafe {
            super::SkStream::SkMemoryStream::from_memory(data.as_ptr(), size, false)
        });
        let proc = self
            .fProcs
            .fTypefaceStreamProc
            .expect("native validated stream callback");
        unsafe { proc(stream.as_mut(), self.fProcs.fTypefaceCtx) }
    }
}
// Source: src/core/SkReadBuffer.cpp:37-45
// The native fallback intentionally refuses all pixel generation. Inheriting
// the real generator's onGetPixels=false is the original failure semantics.
struct EmptyImageGenerator {
    base: super::SkImageGenerator::SkImageGeneratorState,
}
impl super::SkImageGenerator::SkImageGenerator for EmptyImageGenerator {
    fn state(&self) -> &super::SkImageGenerator::SkImageGeneratorState {
        &self.base
    }
}
// Source: src/core/SkReadBuffer.cpp:46-49
fn MakeEmptyImage(
    width: i32,
    height: i32,
) -> Option<std::sync::Arc<dyn crate::src::image::SkImage::SkImage>> {
    let generator = EmptyImageGenerator {
        base: super::SkImageGenerator::SkImageGeneratorState::new(
            &super::SkImageInfo::SkImageInfo::MakeN32Premul(width, height),
            0,
        ),
    };
    crate::src::image::SkImage_Lazy::DeferredFromGenerator(Some(Box::new(generator)))
}
// Source: src/core/SkReadBuffer.cpp:357-399
// Result carries the existing CPU scalar backend error out without converting
// it into a native decode failure or the native empty-image fallback.
unsafe fn add_mipmaps(
    img: std::sync::Arc<dyn crate::src::image::SkImage::SkImage>,
    data: std::sync::Arc<super::SkData::SkData>,
    procs: SkDeserialProcs,
    alpha_type: Option<crate::include::core::SkAlphaType::SkAlphaType>,
) -> Result<std::sync::Arc<dyn crate::src::image::SkImage::SkImage>, crate::RasterError> {
    let builder = super::SkMipmapBuilder::SkMipmapBuilder::new(img.imageInfo())?;
    let bytes = unsafe { core::slice::from_raw_parts(data.data(), data.size()) };
    let mut buffer = SkReadBuffer::new(bytes);
    let count = buffer.read32();
    if builder.countLevels() != count {
        return Ok(img);
    }
    for i in 0..count {
        let size = buffer.read32() as usize;
        let Some(bytes) = buffer.skip(size) else {
            return Ok(img);
        };
        // The outer data owner outlives both temporary encoded data and mip image.
        let mip = unsafe {
            deserialize_image(
                super::SkData::SkData::MakeWithoutCopy(bytes.as_ptr(), size),
                procs,
                alpha_type,
            )
        };
        let Some(mip) = mip else {
            return Ok(img);
        };
        let pm = builder.level(i);
        if mip.dimensions() != pm.dimensions() {
            return Ok(img);
        }
        if !unsafe { mip.readPixels_default(&pm, 0, 0) }? {
            return Ok(img);
        }
    }
    if !buffer.isValid() {
        return Ok(img);
    }
    let Some(raster) = (unsafe { img.makeRasterImage_default() })? else {
        return Ok(img);
    };
    let rasterWithMips = unsafe { builder.attachTo(&raster) }?;
    Ok(rasterWithMips)
}
impl SkReadBuffer<'_> {
    // Source: src/core/SkReadBuffer.h:167-167
    // Source: src/core/SkReadBuffer.cpp:404-441
    pub fn readImage(
        &mut self,
    ) -> Result<Option<std::sync::Arc<dyn crate::src::image::SkImage::SkImage>>, crate::RasterError>
    {
        use super::SkWriteBuffer::SkWriteBufferImageFlags as Flags;
        let flags = self.read32() as u32;
        let mut alphaType = None;
        if flags & Flags::kUnpremul != 0 {
            alphaType = Some(crate::include::core::SkAlphaType::SkAlphaType::kUnpremul_SkAlphaType);
        }
        let mut image = {
            let Some(data) = self.readByteArrayAsData() else {
                self.validate(false);
                return Ok(None);
            };
            unsafe { deserialize_image(data, self.fProcs, alphaType) }
        };
        if flags & Flags::kHasSubsetRect != 0 {
            let mut subset = super::SkRect::SkIRect::default();
            self.readIRect(&mut subset);
            if let Some(img) = image.take() {
                image = img.makeSubset(None, &subset, Default::default())?;
            }
        }
        if flags & Flags::kHasMipmap != 0 {
            let Some(data) = self.readByteArrayAsData() else {
                self.validate(false);
                return Ok(None);
            };
            if let Some(img) = image.take() {
                image = Some(unsafe { add_mipmaps(img, data, self.fProcs, alphaType) }?);
            }
        }
        Ok(image.or_else(|| MakeEmptyImage(1, 1)))
    }
}

// Rust static_cast adapter for src/core/SkReadBuffer.h:229-231.
// UNVERIFIED: implementors must preserve native integer casts; from_i32 may be
// called only for a value valid in the destination type's native value domain.
pub unsafe trait SkReadBufferRange: Copy {
    fn into_i32(self) -> i32;
    unsafe fn from_i32(value: i32) -> Self;
}
macro_rules! integer_read_buffer_range {
    ($($ty:ty),*) => {$(unsafe impl SkReadBufferRange for $ty {
        fn into_i32(self)->i32 {self as i32}
        unsafe fn from_i32(value:i32)->Self {value as Self}
    })*};
}
integer_read_buffer_range!(i8, u8, i16, u16, i32, u32, i64, u64, isize, usize);
unsafe impl SkReadBufferRange for super::SkSamplingPriv::SkLegacyFQ {
    fn into_i32(self) -> i32 {
        self as i32
    }
    unsafe fn from_i32(value: i32) -> Self {
        core::mem::transmute(value)
    }
}
unsafe impl SkReadBufferRange for crate::include::core::SkClipOp::SkClipOp {
    fn into_i32(self) -> i32 {
        self as i32
    }
    unsafe fn from_i32(value: i32) -> Self {
        unsafe { core::mem::transmute(value) }
    }
}
unsafe impl SkReadBufferRange for super::SkCanvas::PointMode {
    fn into_i32(self) -> i32 {
        self as i32
    }
    unsafe fn from_i32(value: i32) -> Self {
        unsafe { core::mem::transmute(value) }
    }
}
unsafe impl SkReadBufferRange for super::SkCanvas::SrcRectConstraint {
    fn into_i32(self) -> i32 {
        self as i32
    }
    unsafe fn from_i32(value: i32) -> Self {
        unsafe { core::mem::transmute(value) }
    }
}
