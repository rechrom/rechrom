// Copyright 2011 Google Inc. BSD-style license. Unverified native translation.
use crate::src::core::{
    SkFlattenable::{self as flat, FactoryResult, FlattenableOwner, SkFlattenableView},
    SkMask::{AllocType, Format, SkMask, SkMaskBuilder},
    SkMaskFilterBase::{SkMaskFilterBase, Type},
    SkMatrix::SkMatrix,
    SkPoint::SkIPoint,
};
use std::sync::Arc;
// Source: src/effects/SkTableMaskFilter.cpp:38-64
pub struct SkTableMaskFilterImpl {
    fTable: [u8; 256],
}
impl SkTableMaskFilterImpl {
    // Source: src/effects/SkTableMaskFilter.cpp:66-70
    fn new_identity() -> Self {
        Self {
            fTable: core::array::from_fn(|i| i as u8),
        }
    }
    // Source: src/effects/SkTableMaskFilter.cpp:72-74
    fn new(table: &[u8; 256]) -> Self {
        Self { fTable: *table }
    }
    // Source: src/effects/SkTableMaskFilter.cpp:139-146
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> FactoryResult {
        let mut table = core::mem::MaybeUninit::<[u8; 256]>::uninit();
        if !unsafe { buffer.readByteArray_raw(table.as_mut_ptr().cast(), 256) } {
            return Ok(None);
        }
        Ok(Some(FlattenableOwner::MaskFilter(Create(unsafe {
            table.assume_init_ref()
        }))))
    }
}
impl SkFlattenableView for SkTableMaskFilterImpl {
    fn getFactory(&self) -> Option<flat::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkTableMaskFilterImpl")
    }
    fn getFlattenableType(&self) -> flat::Type {
        flat::Type::kSkMaskFilter_Type
    }
    // Source: src/effects/SkTableMaskFilter.cpp:135-137
    fn flatten(
        &self,
        w: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            w.writeByteArray(self.fTable.as_ptr(), 256);
        }
        Ok(())
    }
}
impl SkMaskFilterBase for SkTableMaskFilterImpl {
    // Source: src/effects/SkTableMaskFilter.cpp:131-133
    fn getFormat(&self) -> Format {
        Format::kA8_Format
    }
    fn r#type(&self) -> Type {
        Type::kTable
    }
    // Source: src/effects/SkTableMaskFilter.cpp:78-129
    unsafe fn filterMask(
        &self,
        dst: &mut SkMaskBuilder,
        src: &SkMask,
        _ctm: &SkMatrix,
        margin: Option<&mut SkIPoint>,
    ) -> Result<bool, crate::RasterError> {
        if src.fFormat != Format::kA8_Format || src.fBounds.width() > (1 << 30) {
            return Ok(false);
        }
        *dst.bounds() = src.fBounds;
        *dst.rowBytes() = (dst.fBounds.width() as u32 + 3) & !3;
        *dst.format() = Format::kA8_Format;
        *dst.image() = core::ptr::null_mut();
        if !src.fImage.is_null() {
            let size = dst.computeImageSize();
            if size == 0 {
                return Ok(false);
            }
            *dst.image() = SkMaskBuilder::AllocImage(size, AllocType::kUninit_Alloc);
            let mut sp = src.fImage;
            let mut dp = dst.fImage.cast_mut();
            let width = dst.fBounds.width();
            let extra = dst.fRowBytes as i32 - width;
            let mut y = dst.fBounds.height() - 1;
            while y >= 0 {
                let mut x = width - 1;
                while x >= 0 {
                    dp.add(x as usize)
                        .write(self.fTable[sp.add(x as usize).read() as usize]);
                    x -= 1;
                }
                sp = sp.add(src.fRowBytes as usize);
                dp = dp.add(width as usize);
                let mut i = extra - 1;
                while i >= 0 {
                    dp.write(0);
                    dp = dp.add(1);
                    i -= 1;
                }
                y -= 1;
            }
        }
        if let Some(m) = margin {
            m.set(0, 0);
        }
        Ok(true)
    }
    // Source: src/effects/SkTableMaskFilter.cpp:148-155
    fn asImageFilter(
        &self,
        _ctm: &SkMatrix,
        _paint: &crate::src::core::SkPaint::SkPaint,
    ) -> (
        Option<Arc<dyn crate::src::core::SkImageFilter::SkImageFilter>>,
        bool,
    ) {
        let cf = crate::src::effects::colorfilters::SkTableColorFilter::TableARGB(
            Some(&self.fTable),
            None,
            None,
            None,
        );
        (
            crate::src::effects::imagefilters::SkColorFilterImageFilter::ColorFilter(
                cf, None, None,
            ),
            false,
        )
    }
}
// Source: src/effects/SkTableMaskFilter.cpp:158-160
pub fn Create(table: &[u8; 256]) -> Arc<dyn SkMaskFilterBase> {
    Arc::new(SkTableMaskFilterImpl::new(table))
}
// Source: src/effects/SkTableMaskFilter.cpp:162-167
pub fn CreateGamma(gamma: f32) -> Arc<dyn SkMaskFilterBase> {
    let mut table = core::mem::MaybeUninit::<[u8; 256]>::uninit();
    unsafe {
        MakeGammaTable_raw(table.as_mut_ptr().cast(), gamma);
        Create(table.assume_init_ref())
    }
}
// Source: src/effects/SkTableMaskFilter.cpp:169-174
pub fn CreateClip(min: u8, max: u8) -> Arc<dyn SkMaskFilterBase> {
    let mut table = core::mem::MaybeUninit::<[u8; 256]>::uninit();
    unsafe {
        MakeClipTable_raw(table.as_mut_ptr().cast(), min, max);
        Create(table.assume_init_ref())
    }
}
#[cfg_attr(target_family = "unix", link(name = "m"))]
unsafe extern "C" {
    fn powf(x: f32, y: f32) -> f32;
}
// Source: src/effects/SkTableMaskFilter.cpp:176-187
unsafe fn MakeGammaTable_raw(table: *mut u8, gamma: f32) {
    let dx = 1.0f32 / 255.0;
    let mut x = 0.0;
    for i in 0..256 {
        table.add(i).write(
            crate::src::core::SkFloatingPoint::sk_float_round2int(powf(x, gamma) * 255.0)
                .clamp(0, 255) as u8,
        );
        x += dx;
    }
}
pub fn MakeGammaTable(table: &mut [u8; 256], gamma: f32) {
    unsafe {
        MakeGammaTable_raw(table.as_mut_ptr(), gamma);
    }
}
// Source: src/effects/SkTableMaskFilter.cpp:189-211
unsafe fn MakeClipTable_raw(table: *mut u8, mut min: u8, mut max: u8) {
    if max == 0 {
        max = 1;
    }
    if min >= max {
        min = max - 1;
    }
    crate::SkASSERT!(@source b"src/effects/SkTableMaskFilter.cpp\0".as_ptr(),188; @condition b"min < max\0".as_ptr(); min<max);
    let scale = 65536i32 * 255 / (max - min) as i32;
    core::ptr::write_bytes(table, 0, min as usize + 1);
    for i in min as usize + 1..max as usize {
        let value = (scale * (i as i32 - min as i32) + 32768) >> 16;
        crate::SkASSERT!(@source b"src/effects/SkTableMaskFilter.cpp\0".as_ptr(),194; @condition b"value <= 255\0".as_ptr(); value<=255);
        table.add(i).write(value as u8);
    }
    core::ptr::write_bytes(table.add(max as usize), 255, 256 - max as usize);
}
pub fn MakeClipTable(table: &mut [u8; 256], min: u8, max: u8) {
    unsafe {
        MakeClipTable_raw(table.as_mut_ptr(), min, max);
    }
}
// Source: src/effects/SkTableMaskFilter.cpp:231-235
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn RegisterFlattenables() {
    unsafe {
        flat::Register(b"SkTableMaskFilterImpl", SkTableMaskFilterImpl::CreateProc);
        flat::Register(b"SkTableMF", SkTableMaskFilterImpl::CreateProc);
    }
}
