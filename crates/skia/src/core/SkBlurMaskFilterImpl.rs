// Copyright 2006 The Android Open Source Project. BSD-style license.
use crate::include::core::SkBlurTypes::SkBlurStyle;
use crate::src::core::{
    SkMask::{CreateMode, Format, SkMask, SkMaskBuilder},
    SkMaskFilter::SkMaskFilter,
    SkMaskFilterBase::{BlurRec, SkMaskFilterBase, Type},
    SkMatrix::SkMatrix,
    SkPoint::SkIPoint,
    SkRect::SkRect,
};
use std::sync::Arc;
impl SkBlurMaskFilterImpl {
    // Source: src/core/SkBlurMaskFilterImpl.h:41-41
    fn header_type(&self) -> Type {
        Type::kBlur
    }
    // Source: src/core/SkBlurMaskFilterImpl.h:49-51
    pub fn blurStyle(&self) -> SkBlurStyle {
        self.fBlurStyle
    }
    pub fn sigma(&self) -> f32 {
        self.fSigma
    }
    pub fn ignoreXform(&self) -> bool {
        !self.fRespectCTM
    }
}
// Source: src/core/SkBlurMaskFilterImpl.h:73-75
pub struct SkBlurMaskFilterImpl {
    fSigma: f32,
    fBlurStyle: SkBlurStyle,
    fRespectCTM: bool,
}
impl SkBlurMaskFilterImpl {
    // Source: src/core/SkBlurMaskFilterImpl.cpp:51-56
    pub fn new(sigma: f32, style: SkBlurStyle, respectCTM: bool) -> Self {
        crate::SkASSERT!(@source b"src/core/SkBlurMaskFilterImpl.cpp\0".as_ptr(),55; @condition b"fSigma > 0\0".as_ptr(); sigma>0.0);
        crate::SkASSERT!(@source b"src/core/SkBlurMaskFilterImpl.cpp\0".as_ptr(),56; @condition b"(unsigned)style <= kLastEnum_SkBlurStyle\0".as_ptr(); (style as u32)<=crate::include::core::SkBlurTypes::kLastEnum_SkBlurStyle as u32);
        Self {
            fSigma: sigma,
            fBlurStyle: style,
            fRespectCTM: respectCTM,
        }
    }
    // Source: src/core/SkBlurMaskFilterImpl.cpp:59-60
    pub fn getFormat(&self) -> Format {
        Format::kA8_Format
    }
    // Source: src/core/SkBlurMaskFilterImpl.cpp:63-65
    // Source: src/core/SkBlurMaskFilterImpl.cpp:68-70
    // Source: src/core/SkBlurMaskFilterImpl.cpp:72-72
    pub fn asABlur(&self, rec: Option<&mut BlurRec>) -> bool {
        if self.ignoreXform() {
            return false;
        }
        if let Some(rec) = rec {
            rec.fSigma = self.fSigma;
            rec.fStyle = self.fBlurStyle;
        }
        true
    }
    // Source: src/core/SkBlurMaskFilterImpl.cpp:75-76
    // Source: src/core/SkBlurMaskFilterImpl.cpp:79-80
    // Source: src/core/SkBlurMaskFilterImpl.cpp:87-89
    // Source: src/core/SkBlurMaskFilterImpl.cpp:93-93
    // Source: src/core/SkBlurMaskFilterImpl.cpp:95-96
    // Source: src/core/SkBlurMaskFilterImpl.cpp:98-99
    // Source: src/core/SkBlurMaskFilterImpl.cpp:101-102
    // Source: src/core/SkBlurMaskFilterImpl.cpp:104-106
    // Source: src/core/SkBlurMaskFilterImpl.cpp:108-108
    pub fn asImageFilter(
        &self,
        ctm: &SkMatrix,
        _paint: &crate::src::core::SkPaint::SkPaint,
    ) -> (
        Option<Arc<dyn crate::src::core::SkImageFilter::SkImageFilter>>,
        bool,
    ) {
        use crate::include::core::{SkBlurTypes::SkBlurStyle::*, SkTileMode::SkTileMode};
        use crate::src::core::SkBlendMode::SkBlendMode;
        let mut sigma = [self.fSigma, self.fSigma];
        if self.ignoreXform() {
            let xScaleFactor = self.fSigma / ctm.mapVectorXY(self.fSigma, 0.0).length();
            let yScaleFactor = self.fSigma / ctm.mapVectorXY(0.0, self.fSigma).length();
            sigma = [self.fSigma * xScaleFactor, self.fSigma * yScaleFactor];
        }
        let filter = crate::src::effects::imagefilters::SkBlurImageFilter::Blur(
            sigma[0],
            sigma[1],
            SkTileMode::kDecal,
            None,
            None,
        );
        match self.fBlurStyle {
            kInner_SkBlurStyle => (
                crate::src::effects::imagefilters::SkBlendImageFilter::Blend(
                    SkBlendMode::kDstIn,
                    filter,
                    None,
                    None,
                ),
                false,
            ),
            kSolid_SkBlurStyle => (
                crate::src::effects::imagefilters::SkBlendImageFilter::Blend(
                    SkBlendMode::kSrcOver,
                    filter,
                    None,
                    None,
                ),
                false,
            ),
            kOuter_SkBlurStyle => (
                crate::src::effects::imagefilters::SkBlendImageFilter::Blend(
                    SkBlendMode::kDstOut,
                    filter,
                    None,
                    None,
                ),
                false,
            ),
            kNormal_SkBlurStyle => (filter, false),
        }
    }
    // Source: src/core/SkBlurMaskFilterImpl.cpp:111-114
    pub fn computeXformedSigma(&self, ctm: &SkMatrix) -> f32 {
        let xformedSigma = if self.ignoreXform() {
            self.fSigma
        } else {
            ctm.mapRadius(self.fSigma)
        };
        if 128.0 < xformedSigma {
            128.0
        } else {
            xformedSigma
        }
    }
    // Source: src/core/SkBlurMaskFilterImpl.cpp:117-121
    pub unsafe fn filterMask(
        &self,
        dst: &mut SkMaskBuilder,
        src: &SkMask,
        matrix: &SkMatrix,
        margin: Option<&mut SkIPoint>,
    ) -> Result<bool, crate::RasterError> {
        let sigma = self.computeXformedSigma(matrix);
        Ok(unsafe {
            crate::src::core::SkBlurMask::BoxBlur(dst, src, sigma, self.fBlurStyle, margin)
        })
    }
    // Source: src/core/SkBlurMaskFilterImpl.cpp:124-128
    // Source: src/core/SkBlurMaskFilterImpl.cpp:130-130
    pub unsafe fn filterRectMask(
        &self,
        dst: &mut SkMaskBuilder,
        r: &SkRect,
        matrix: &SkMatrix,
        margin: Option<&mut SkIPoint>,
        createMode: CreateMode,
    ) -> bool {
        let sigma = self.computeXformedSigma(matrix);
        unsafe {
            crate::src::core::SkBlurMask::BlurRect(
                sigma,
                dst,
                r,
                self.fBlurStyle,
                margin,
                createMode,
            )
        }
    }
}
// Source: src/core/SkBlurMaskFilterImpl.cpp:133-134
// Source: src/core/SkBlurMaskFilterImpl.cpp:136-141
// Source: src/core/SkBlurMaskFilterImpl.cpp:143-145
// Source: src/core/SkBlurMaskFilterImpl.cpp:147-147
fn prepare_to_draw_into_mask(bounds: &SkRect, mask: &mut SkMaskBuilder) -> bool {
    *mask.bounds() = bounds.roundOut();
    *mask.rowBytes() = (mask.fBounds.width() as u32 + 3) & !3;
    *mask.format() = Format::kA8_Format;
    let size = mask.computeImageSize();
    if size == 0 {
        return false;
    }
    *mask.image() =
        SkMaskBuilder::AllocImage(size, crate::src::core::SkMask::AllocType::kZeroInit_Alloc);
    if mask.fImage.is_null() {
        return false;
    }
    true
}
// Source: src/core/SkBlurMaskFilterImpl.cpp:150-152
// Source: src/core/SkBlurMaskFilterImpl.cpp:155-158
// Source: src/core/SkBlurMaskFilterImpl.cpp:160-162
// Source: src/core/SkBlurMaskFilterImpl.cpp:164-164
// Source: src/core/SkBlurMaskFilterImpl.cpp:166-170
// Source: src/core/SkBlurMaskFilterImpl.cpp:172-173
// Source: src/core/SkBlurMaskFilterImpl.cpp:175-176
/// Safety: proc must uphold the configured Draw destination/clip lease and may not retain it.
unsafe fn draw_into_mask(
    mask: &mut SkMaskBuilder,
    bounds: &SkRect,
    proc: impl FnOnce(
        &crate::src::core::SkDraw::Draw<'_>,
        &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError>,
) -> Result<bool, crate::RasterError> {
    if !prepare_to_draw_into_mask(bounds, mask) {
        return Ok(false);
    }
    let dx = mask.fBounds.fLeft;
    let dy = mask.fBounds.fTop;
    let mut rclip = crate::src::core::SkRasterClip::SkRasterClip::from_rect(mask.fBounds);
    rclip.set_rect(mask.fBounds.make_offset(-dx, -dy));
    crate::SkASSERT!(@source b"src/core/SkBlurMaskFilterImpl.cpp\0".as_ptr(),160; @condition b"mask->fFormat == SkMask::kA8_Format\0".as_ptr(); (mask.fFormat)==(Format::kA8_Format));
    let info = crate::src::core::SkImageInfo::SkImageInfo::MakeA8(
        mask.fBounds.width(),
        mask.fBounds.height(),
    );
    let pm = crate::src::core::SkPixmap::SkPixmap::new(&info, mask.fImage, mask.fRowBytes as usize);
    let ctm = SkMatrix::Translate(-(dx as f32), -(dy as f32));
    let mut draw = crate::src::core::SkDraw::Draw::new();
    // Error-boundary forwarding preserves the native chooser and returned arena lifetime.
    draw.fBlitterChooser = |dst, ctm, paint, alloc, coverage, clip, props, bounds| unsafe {
        crate::src::core::SkBlitter_A8::SkA8Blitter_Choose(
            dst, ctm, paint, alloc, coverage, clip, props, bounds,
        )
        .map_err(Into::into)
    };
    draw.fCTM = Some(&ctm);
    draw.fDst = pm;
    draw.fRC = Some(&rclip);
    let mut paint = crate::src::core::SkPaint::SkPaint::default();
    paint.setAntiAlias(true);
    // The explicit Rust stage-error boundary has no native counterpart. Release this new
    // allocation before propagating failure; successful callers receive its original ownership.
    if let Err(error) = proc(&draw, &paint) {
        unsafe {
            SkMaskBuilder::FreeImage(mask.fImage.cast_mut().cast());
        }
        *mask.image() = core::ptr::null_mut();
        return Err(error);
    }
    Ok(true)
}
// Source: src/core/SkBlurMaskFilterImpl.cpp:179-184
// Source: src/core/SkBlurMaskFilterImpl.cpp:186-190
// Source: src/core/SkBlurMaskFilterImpl.cpp:192-192
unsafe fn draw_rects_into_mask(
    rects: &[SkRect],
    mask: &mut SkMaskBuilder,
) -> Result<bool, crate::RasterError> {
    crate::SkASSERT!(@source b"src/core/SkBlurMaskFilterImpl.cpp\0".as_ptr(),180; @condition b"rects.size() == 1 || rects.size() == 2\0".as_ptr(); rects.len()==1||rects.len()==2);
    unsafe {
        draw_into_mask(mask, &rects[0], |draw, paint| {
            if rects.len() == 1 {
                draw.drawRect(&rects[0], paint, None, None)
            } else {
                use crate::include::core::SkPathTypes::{SkPathDirection, SkPathFillType};
                let mut builder = crate::src::core::SkPathBuilder::SkPathBuilder::new();
                let path = builder
                    .addRectDefault(&rects[0], SkPathDirection::kCW)
                    .addRectDefault(&rects[1], SkPathDirection::kCW)
                    .setFillType(SkPathFillType::kEvenOdd)
                    .detach(None);
                draw.drawPath(&path, paint, None)
            }
        })
    }
}
// Source: src/core/SkBlurMaskFilterImpl.cpp:195-198
unsafe fn draw_rrect_into_mask(
    rrect: &crate::src::core::SkRRect::SkRRect,
    mask: &mut SkMaskBuilder,
) -> Result<bool, crate::RasterError> {
    unsafe {
        draw_into_mask(mask, &rrect.rect(), |draw, paint| {
            draw.drawRRect(rrect, paint)
        })
    }
}
// Source: src/core/SkBlurMaskFilterImpl.cpp:201-203
fn rect_exceeds(r: &SkRect, v: f32) -> bool {
    r.fLeft < -v || r.fTop < -v || r.fRight > v || r.fBottom > v || r.width() > v || r.height() > v
}
// Source: src/core/SkBlurMaskFilterImpl.cpp:206-212
// Source: src/core/SkBlurMaskFilterImpl.cpp:214-217
// Source: src/core/SkBlurMaskFilterImpl.cpp:219-219
/// Safety: mask owns its malloc-compatible image, covering computeTotalImageSize bytes.
/// Any borrowed image views must end before this call replaces and frees that allocation.
/// Returned client handle must remain alive until its replacement mask image is no longer used.
unsafe fn copy_mask_to_cacheddata(
    mask: &mut SkMaskBuilder,
    cache: Option<&mut crate::src::core::SkResourceCache::SkResourceCache>,
) -> Option<crate::src::core::SkCachedData::SkCachedData<()>> {
    let size = mask.computeTotalImageSize();
    let data = if let Some(cache) = cache {
        cache.newCachedData(size)
    } else {
        crate::src::core::SkResourceCache::NewCachedData(size)
    };
    if let Some(data) = &data {
        unsafe {
            core::ptr::copy_nonoverlapping(mask.fImage, data.writable_data(), size);
            SkMaskBuilder::FreeImage(mask.fImage.cast_mut().cast());
        }
        *mask.image() = data.writable_data();
    }
    data
}
// Source: src/core/SkBlurMaskFilterImpl.cpp:222-227
unsafe fn find_cached_rrect(
    mask: &mut Option<SkMask>,
    sigma: f32,
    style: SkBlurStyle,
    rrect: &crate::src::core::SkRRect::SkRRect,
    cache: Option<&mut crate::src::core::SkResourceCache::SkResourceCache>,
) -> Option<crate::src::core::SkCachedData::SkCachedData<()>> {
    unsafe { crate::src::core::SkMaskCache::FindAndRefRRect(sigma, style, rrect, mask, cache) }
}
// Source: src/core/SkBlurMaskFilterImpl.cpp:230-237
// Source: src/core/SkBlurMaskFilterImpl.cpp:239-239
unsafe fn add_cached_rrect(
    mask: &mut SkMaskBuilder,
    sigma: f32,
    style: SkBlurStyle,
    rrect: &crate::src::core::SkRRect::SkRRect,
    mut cache: Option<&mut crate::src::core::SkResourceCache::SkResourceCache>,
) -> Option<crate::src::core::SkCachedData::SkCachedData<()>> {
    let cached = unsafe { copy_mask_to_cacheddata(mask, cache.as_deref_mut()) };
    if let Some(cached) = &cached {
        unsafe {
            crate::src::core::SkMaskCache::AddRRect(sigma, style, rrect, mask, cached, cache);
        }
    }
    cached
}
// Source: src/core/SkBlurMaskFilterImpl.cpp:242-247
unsafe fn find_cached_rects(
    mask: &mut Option<SkMask>,
    sigma: f32,
    style: SkBlurStyle,
    rects: &[SkRect],
    cache: Option<&mut crate::src::core::SkResourceCache::SkResourceCache>,
) -> Option<crate::src::core::SkCachedData::SkCachedData<()>> {
    unsafe { crate::src::core::SkMaskCache::FindAndRefRects(sigma, style, rects, mask, cache) }
}
// Source: src/core/SkBlurMaskFilterImpl.cpp:250-257
// Source: src/core/SkBlurMaskFilterImpl.cpp:259-259
unsafe fn add_cached_rects(
    mask: &mut SkMaskBuilder,
    sigma: f32,
    style: SkBlurStyle,
    rects: &[SkRect],
    mut cache: Option<&mut crate::src::core::SkResourceCache::SkResourceCache>,
) -> Option<crate::src::core::SkCachedData::SkCachedData<()>> {
    let cached = unsafe { copy_mask_to_cacheddata(mask, cache.as_deref_mut()) };
    if let Some(cached) = &cached {
        unsafe {
            crate::src::core::SkMaskCache::AddRects(sigma, style, rects, mask, cached, cache);
        }
    }
    cached
}
impl SkBlurMaskFilterImpl {
    // Source: src/core/SkBlurMaskFilterImpl.cpp:262-268
    // Source: src/core/SkBlurMaskFilterImpl.cpp:270-270
    // Source: src/core/SkBlurMaskFilterImpl.cpp:272-272
    // Source: src/core/SkBlurMaskFilterImpl.cpp:274-276
    // Source: src/core/SkBlurMaskFilterImpl.cpp:279-279
    // Source: src/core/SkBlurMaskFilterImpl.cpp:282-285
    // Source: src/core/SkBlurMaskFilterImpl.cpp:290-291
    // Source: src/core/SkBlurMaskFilterImpl.cpp:296-297
    // Source: src/core/SkBlurMaskFilterImpl.cpp:320-323
    // Source: src/core/SkBlurMaskFilterImpl.cpp:348-351
    // Source: src/core/SkBlurMaskFilterImpl.cpp:355-356
    // Source: src/core/SkBlurMaskFilterImpl.cpp:359-359
    // Source: src/core/SkBlurMaskFilterImpl.cpp:361-362
    // Source: src/core/SkBlurMaskFilterImpl.cpp:364-364
    // Source: src/core/SkBlurMaskFilterImpl.cpp:367-368
    // Source: src/core/SkBlurMaskFilterImpl.cpp:370-371
    // Source: src/core/SkBlurMaskFilterImpl.cpp:373-373
    // Source: src/core/SkBlurMaskFilterImpl.cpp:377-379
    // Source: src/core/SkBlurMaskFilterImpl.cpp:381-381
    // Source: src/core/SkBlurMaskFilterImpl.cpp:383-385
    // Source: src/core/SkBlurMaskFilterImpl.cpp:394-395
    // Source: src/core/SkBlurMaskFilterImpl.cpp:397-397
    // Source: src/core/SkBlurMaskFilterImpl.cpp:410-412
    // Source: src/core/SkBlurMaskFilterImpl.cpp:414-415
    // Source: src/core/SkBlurMaskFilterImpl.cpp:417-418
    // Source: src/core/SkBlurMaskFilterImpl.cpp:422-423
    // Source: src/core/SkBlurMaskFilterImpl.cpp:428-435
    /// Safety: uphold native cache synchronization and mask allocation ownership contracts.
    pub unsafe fn filterRRectToNine(
        &self,
        rrect: &crate::src::core::SkRRect::SkRRect,
        matrix: &SkMatrix,
        _clipBounds: &crate::src::core::SkRect::SkIRect,
        patch: core::pin::Pin<&mut crate::src::core::SkMaskFilterBase::NinePatchStorage>,
        mut cache: Option<&mut crate::src::core::SkResourceCache::SkResourceCache>,
    ) -> Result<bool, crate::RasterError> {
        use crate::src::core::SkMask::SkAutoMaskFreeImage;
        use crate::src::core::SkMaskFilterBase::NinePatch;
        use crate::src::core::SkRRect::{Corner::*, Type as RType};
        use crate::src::core::SkScalar::SkScalarCeilToInt;
        match rrect.getType() {
            RType::kEmpty_Type => return Ok(false),
            RType::kRect_Type => {
                crate::SkDEBUGFAIL!(@source b"src/core/SkBlurMaskFilterImpl.cpp\0".as_ptr(),274; b"Should use a different special case\0".as_ptr());
                return Ok(false);
            }
            RType::kOval_Type => return Ok(false),
            RType::kSimple_Type | RType::kNinePatch_Type | RType::kComplex_Type => {}
        }
        if self.fBlurStyle == SkBlurStyle::kInner_SkBlurStyle {
            return Ok(false);
        }
        if rect_exceeds(rrect.rect(), 32767.0) {
            return Ok(false);
        }
        let mut margin = SkIPoint::default();
        let mut srcM = SkMaskBuilder::from_image(
            core::ptr::null_mut(),
            rrect.rect().roundOut(),
            0,
            Format::kA8_Format,
        );
        let mut dstM = SkMaskBuilder::new();
        if !unsafe { self.filterMask(&mut dstM, &srcM, matrix, Some(&mut margin)) }? {
            return Ok(false);
        }
        let ul = rrect.radii(kUpperLeft_Corner);
        let ur = rrect.radii(kUpperRight_Corner);
        let lr = rrect.radii(kLowerRight_Corner);
        let ll = rrect.radii(kLowerLeft_Corner);
        // std::max uses its first argument when comparison is false, including unordered values.
        let max = |a: f32, b: f32| if a < b { b } else { a };
        let leftUnstretched = SkScalarCeilToInt(max(ul.fX, ll.fX)) + margin.fX;
        let rightUnstretched = SkScalarCeilToInt(max(ur.fX, lr.fX)) + margin.fX;
        let stretchSize = 1;
        let totalSmallWidth = leftUnstretched + rightUnstretched + stretchSize;
        if totalSmallWidth as f32 >= rrect.rect().width() {
            return Ok(false);
        }
        let topUnstretched = SkScalarCeilToInt(max(ul.fY, ur.fY)) + margin.fY;
        let botUnstretched = SkScalarCeilToInt(max(ll.fY, lr.fY)) + margin.fY;
        let totalSmallHeight = topUnstretched + botUnstretched + stretchSize;
        if totalSmallHeight as f32 >= rrect.rect().height() {
            return Ok(false);
        }
        let smallR = SkRect::MakeWH(totalSmallWidth as f32, totalSmallHeight as f32);
        let mut smallRR = crate::src::core::SkRRect::SkRRect::default();
        smallRR.setRectRadii(&smallR, &[ul, ur, lr, ll]);
        let sigma = self.computeXformedSigma(matrix);
        let mut cachedMask = None;
        let mut cached = unsafe {
            find_cached_rrect(
                &mut cachedMask,
                sigma,
                self.fBlurStyle,
                &smallRR,
                cache.as_deref_mut(),
            )
        };
        if cached.is_none() {
            if !unsafe { draw_rrect_into_mask(&smallRR, &mut srcM) }? {
                return Ok(false);
            }
            let _amf = unsafe { SkAutoMaskFreeImage::from_raw(srcM.fImage.cast_mut()) };
            let mut filterM = SkMaskBuilder::new();
            if !unsafe { self.filterMask(&mut filterM, &srcM, matrix, None) }? {
                return Ok(false);
            }
            crate::SkASSERT!(@source b"src/core/SkBlurMaskFilterImpl.cpp\0".as_ptr(),414; @condition b"filterM.fBounds.width() == (srcM.fBounds.width() + 2*margin.fX)\0".as_ptr(); filterM.fBounds.width()==srcM.fBounds.width()+2*margin.fX);
            crate::SkASSERT!(@source b"src/core/SkBlurMaskFilterImpl.cpp\0".as_ptr(),415; @condition b"filterM.fBounds.height() == (srcM.fBounds.height() + 2*margin.fY)\0".as_ptr(); filterM.fBounds.height()==srcM.fBounds.height()+2*margin.fY);
            cached =
                unsafe { add_cached_rrect(&mut filterM, sigma, self.fBlurStyle, &smallRR, cache) };
            cachedMask = Some(*filterM);
        }
        let cachedMask = cachedMask.expect("native hit or rendered mask initialized");
        let mut bounds = cachedMask.fBounds;
        bounds.offset_to(0, 0);
        let center = SkIPoint {
            fX: margin.fX + leftUnstretched,
            fY: margin.fY + topUnstretched,
        };
        unsafe {
            patch.emplace(
                &SkMask::new(
                    cachedMask.fImage,
                    bounds,
                    cachedMask.fRowBytes,
                    cachedMask.fFormat,
                ),
                dstM.fBounds,
                center,
                cached,
            );
        }
        Ok(true)
    }
    // Source: src/core/SkBlurMaskFilterImpl.cpp:438-445
    // Source: src/core/SkBlurMaskFilterImpl.cpp:449-450
    // Source: src/core/SkBlurMaskFilterImpl.cpp:455-456
    // Source: src/core/SkBlurMaskFilterImpl.cpp:459-460
    // Source: src/core/SkBlurMaskFilterImpl.cpp:462-463
    // Source: src/core/SkBlurMaskFilterImpl.cpp:466-469
    // Source: src/core/SkBlurMaskFilterImpl.cpp:472-473
    // Source: src/core/SkBlurMaskFilterImpl.cpp:490-492
    // Source: src/core/SkBlurMaskFilterImpl.cpp:495-497
    // Source: src/core/SkBlurMaskFilterImpl.cpp:499-507
    // Source: src/core/SkBlurMaskFilterImpl.cpp:511-512
    // Source: src/core/SkBlurMaskFilterImpl.cpp:516-518
    // Source: src/core/SkBlurMaskFilterImpl.cpp:521-521
    // Source: src/core/SkBlurMaskFilterImpl.cpp:524-527
    // Source: src/core/SkBlurMaskFilterImpl.cpp:529-532
    // Source: src/core/SkBlurMaskFilterImpl.cpp:535-543
    // Source: src/core/SkBlurMaskFilterImpl.cpp:546-546
    // Source: src/core/SkBlurMaskFilterImpl.cpp:548-549
    // Source: src/core/SkBlurMaskFilterImpl.cpp:551-554
    // Source: src/core/SkBlurMaskFilterImpl.cpp:557-558
    // Source: src/core/SkBlurMaskFilterImpl.cpp:560-566
    /// Safety: uphold native cache synchronization and sole ownership of returned mask pixels.
    pub unsafe fn filterRectsToNine(
        &self,
        rects: &[SkRect],
        matrix: &SkMatrix,
        _clipBounds: &crate::src::core::SkRect::SkIRect,
        patch: core::pin::Pin<&mut crate::src::core::SkMaskFilterBase::NinePatchStorage>,
        mut cache: Option<&mut crate::src::core::SkResourceCache::SkResourceCache>,
    ) -> Result<crate::src::core::SkMaskFilterBase::FilterReturn, crate::RasterError> {
        use crate::src::core::SkMask::SkAutoMaskFreeImage;
        use crate::src::core::SkMaskFilterBase::{FilterReturn::*, NinePatch};
        crate::SkASSERT!(@source b"src/core/SkBlurMaskFilterImpl.cpp\0".as_ptr(),445; @condition b"rects.size() == 1 || rects.size() == 2\0".as_ptr(); rects.len()==1||rects.len()==2);
        if self.fBlurStyle == SkBlurStyle::kInner_SkBlurStyle
            || self.fBlurStyle == SkBlurStyle::kOuter_SkBlurStyle
        {
            return Ok(kUnimplemented);
        }
        if rect_exceeds(&rects[0], 32767.0) {
            return Ok(kUnimplemented);
        }
        let mut margin = SkIPoint::default();
        let mut srcM = SkMaskBuilder::from_image(
            core::ptr::null_mut(),
            rects[0].roundOut(),
            0,
            Format::kA8_Format,
        );
        let mut dstM = SkMaskBuilder::new();
        let filterResult = if rects.len() == 1 {
            unsafe {
                self.filterRectMask(
                    &mut dstM,
                    &rects[0],
                    matrix,
                    Some(&mut margin),
                    CreateMode::kJustComputeBounds_CreateMode,
                )
            }
        } else {
            unsafe { self.filterMask(&mut dstM, &srcM, matrix, Some(&mut margin)) }?
        };
        if !filterResult {
            return Ok(kFalse);
        }
        let mut smallR = [SkRect::default(); 2];
        let rectCount;
        let center;
        let mut smallW = dstM.fBounds.width() - srcM.fBounds.width() + 2;
        let mut smallH = dstM.fBounds.height() - srcM.fBounds.height() + 2;
        let innerIR;
        if rects.len() == 1 {
            rectCount = 1;
            innerIR = srcM.fBounds;
            center = SkIPoint {
                fX: smallW,
                fY: smallH,
            };
        } else {
            rectCount = 2;
            innerIR = rects[1].roundIn();
            center = SkIPoint {
                fX: smallW + (innerIR.fLeft - srcM.fBounds.fLeft),
                fY: smallH + (innerIR.fTop - srcM.fBounds.fTop),
            };
        }
        smallW += 1;
        smallH += 1;
        let dx = (innerIR.width() - smallW) as f32;
        let dy = (innerIR.height() - smallH) as f32;
        if dx < 0.0 || dy < 0.0 {
            return Ok(kUnimplemented);
        }
        smallR[0].setLTRB(
            rects[0].fLeft,
            rects[0].fTop,
            rects[0].fRight - dx,
            rects[0].fBottom - dy,
        );
        if smallR[0].width() < 2.0 || smallR[0].height() < 2.0 {
            return Ok(kUnimplemented);
        }
        if rectCount == 2 {
            smallR[1].setLTRB(
                rects[1].fLeft,
                rects[1].fTop,
                rects[1].fRight - dx,
                rects[1].fBottom - dy,
            );
            crate::SkASSERT!(@source b"src/core/SkBlurMaskFilterImpl.cpp\0".as_ptr(),532; @condition b"!smallR[1].isEmpty()\0".as_ptr(); !smallR[1].isEmpty());
        }
        let sigma = self.computeXformedSigma(matrix);
        let mut cachedMask = None;
        let smallRects = &smallR[..rectCount];
        let mut cached = unsafe {
            find_cached_rects(
                &mut cachedMask,
                sigma,
                self.fBlurStyle,
                smallRects,
                cache.as_deref_mut(),
            )
        };
        if cached.is_none() {
            let mut filterM = SkMaskBuilder::new();
            if rectCount == 2 {
                if !unsafe { draw_rects_into_mask(smallRects, &mut srcM) }? {
                    return Ok(kFalse);
                }
                let _amf = unsafe { SkAutoMaskFreeImage::from_raw(srcM.fImage.cast_mut()) };
                if !unsafe { self.filterMask(&mut filterM, &srcM, matrix, None) }? {
                    return Ok(kFalse);
                }
            } else if !unsafe {
                self.filterRectMask(
                    &mut filterM,
                    &smallR[0],
                    matrix,
                    None,
                    CreateMode::kComputeBoundsAndRenderImage_CreateMode,
                )
            } {
                return Ok(kFalse);
            }
            cached = unsafe {
                add_cached_rects(&mut filterM, sigma, self.fBlurStyle, smallRects, cache)
            };
            cachedMask = Some(*filterM);
        }
        let cachedMask = cachedMask.expect("native hit or rendered mask initialized");
        let mut bounds = cachedMask.fBounds;
        bounds.offset_to(0, 0);
        unsafe {
            patch.emplace(
                &SkMask::new(
                    cachedMask.fImage,
                    bounds,
                    cachedMask.fRowBytes,
                    cachedMask.fFormat,
                ),
                dstM.fBounds,
                center,
                cached,
            );
        }
        Ok(kTrue)
    }
    // Source: src/core/SkBlurMaskFilterImpl.cpp:569-570
    // Source: src/core/SkBlurMaskFilterImpl.cpp:574-574
    // Source: src/core/SkBlurMaskFilterImpl.cpp:576-577
    pub fn computeFastBounds(&self, src: SkRect, dst: &mut SkRect) {
        let pad = 3.0 * self.fSigma;
        dst.setLTRB(
            src.fLeft - pad,
            src.fTop - pad,
            src.fRight + pad,
            src.fBottom + pad,
        );
    }
}
// Native C++ base dispatch delegates to the ordered concrete methods above.
impl SkMaskFilterBase for SkBlurMaskFilterImpl {
    fn asBlurImpl(&self) -> Option<&SkBlurMaskFilterImpl> {
        Some(self)
    }

    fn getFormat(&self) -> Format {
        Self::getFormat(self)
    }
    fn asABlur(&self, rec: Option<&mut BlurRec>) -> bool {
        Self::asABlur(self, rec)
    }
    unsafe fn filterMask(
        &self,
        dst: &mut SkMaskBuilder,
        src: &SkMask,
        matrix: &SkMatrix,
        margin: Option<&mut SkIPoint>,
    ) -> Result<bool, crate::RasterError> {
        unsafe { Self::filterMask(self, dst, src, matrix, margin) }
    }
    fn computeFastBounds(&self, src: SkRect, dst: &mut SkRect) -> Result<(), crate::RasterError> {
        Self::computeFastBounds(self, src, dst);
        Ok(())
    }
    fn asImageFilter(
        &self,
        ctm: &SkMatrix,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> (
        Option<Arc<dyn crate::src::core::SkImageFilter::SkImageFilter>>,
        bool,
    ) {
        Self::asImageFilter(self, ctm, paint)
    }
    unsafe fn filterRectsToNine(
        &self,
        rects: &[SkRect],
        matrix: &SkMatrix,
        clip: &crate::src::core::SkRect::SkIRect,
        patch: core::pin::Pin<&mut crate::src::core::SkMaskFilterBase::NinePatchStorage>,
        cache: Option<&mut crate::src::core::SkResourceCache::SkResourceCache>,
    ) -> Result<crate::src::core::SkMaskFilterBase::FilterReturn, crate::RasterError> {
        unsafe { Self::filterRectsToNine(self, rects, matrix, clip, patch, cache) }
    }
    unsafe fn filterRRectToNine(
        &self,
        rrect: &crate::src::core::SkRRect::SkRRect,
        matrix: &SkMatrix,
        clip: &crate::src::core::SkRect::SkIRect,
        patch: core::pin::Pin<&mut crate::src::core::SkMaskFilterBase::NinePatchStorage>,
        cache: Option<&mut crate::src::core::SkResourceCache::SkResourceCache>,
    ) -> Result<bool, crate::RasterError> {
        unsafe { Self::filterRRectToNine(self, rrect, matrix, clip, patch, cache) }
    }
    fn r#type(&self) -> Type {
        self.header_type()
    }
}
// Source: src/core/SkBlurMaskFilterImpl.cpp:598-600
// Source: src/core/SkBlurMaskFilterImpl.cpp:602-602
pub fn MakeBlur(style: SkBlurStyle, sigma: f32, respectCTM: bool) -> Option<Arc<dyn SkMaskFilter>> {
    if crate::src::core::SkFloatingPoint::SkIsFinite(&[sigma]) && sigma > 0.0 {
        Some(Arc::new(SkBlurMaskFilterImpl::new(
            sigma, style, respectCTM,
        )))
    } else {
        None
    }
}

// Source: src/core/SkBlurMaskFilterImpl.h:71-71
impl crate::src::core::SkFlattenable::SkFlattenableView for SkBlurMaskFilterImpl {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkBlurMaskFilterImpl")
    }
    fn getFlattenableType(&self) -> crate::src::core::SkFlattenable::Type {
        crate::src::core::SkFlattenable::Type::kSkMaskFilter_Type
    }
    // Source: src/core/SkBlurMaskFilterImpl.cpp:590-594
    fn flatten(
        &self,
        writer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        writer.writeScalar(self.fSigma);
        writer.writeUInt(self.fBlurStyle as u32);
        writer.writeUInt((!self.fRespectCTM) as u32);
        Ok(())
    }
}
impl SkBlurMaskFilterImpl {
    // Source: src/core/SkBlurMaskFilterImpl.cpp:580-588
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        let sigma = buffer.readScalar();
        let style =
            buffer.read32LE(crate::include::core::SkBlurTypes::kLastEnum_SkBlurStyle as u32);
        let flags = buffer.read32LE(3);
        let respect_ctm = flags & 1 == 0;
        let style = match style {
            0 => SkBlurStyle::kNormal_SkBlurStyle,
            1 => SkBlurStyle::kSolid_SkBlurStyle,
            2 => SkBlurStyle::kOuter_SkBlurStyle,
            _ => SkBlurStyle::kInner_SkBlurStyle,
        };
        Ok(MakeBlur(style, sigma, respect_ctm)
            .map(crate::src::core::SkFlattenable::FlattenableOwner::MaskFilter))
    }
}
// Source: src/core/SkBlurMaskFilterImpl.cpp:596-596
/// Safety: exclusive native initialization before registry publication, with fixed-table capacity.
pub unsafe fn sk_register_blur_maskfilter_createproc() {
    unsafe {
        crate::src::core::SkFlattenable::Register(
            b"SkBlurMaskFilterImpl",
            SkBlurMaskFilterImpl::CreateProc,
        );
    }
}
