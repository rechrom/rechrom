// UNVERIFIED v86 Pin receiver propagation only; zero repeated algorithm credit.
// Copyright 2018 Google LLC. BSD-style license.
use crate::include::core::SkBlurTypes::SkBlurStyle;
use crate::src::core::{
    SkMask::{Format, SkMask, SkMaskBuilder},
    SkMatrix::SkMatrix,
    SkPoint::SkIPoint,
    SkRect::SkRect,
};
pub trait SkMaskFilterBase: crate::src::core::SkFlattenable::SkFlattenableView {
    // Rust type-view adaptation of native checked static_cast. This borrows the
    // actual concrete object and carries no copied blur state or ownership.
    fn asBlurImpl(&self) -> Option<&crate::src::core::SkBlurMaskFilterImpl::SkBlurMaskFilterImpl> {
        None
    }

    // Source: src/core/SkMaskFilterBase.h:45-45
    fn getFormat(&self) -> Format;
    /// Safety: non-null source image covers the declared bounds/stride. Null source image
    /// requests bounds only, and implementations must not allocate an image in that case.
    // Source: src/core/SkMaskFilterBase.h:60-61
    unsafe fn filterMask(
        &self,
        dst: &mut SkMaskBuilder,
        src: &SkMask,
        matrix: &SkMatrix,
        margin: Option<&mut SkIPoint>,
    ) -> Result<bool, crate::RasterError>;
    fn r#type(&self) -> Type;
    fn computeFastBounds(&self, src: SkRect, dst: &mut SkRect) -> Result<(), crate::RasterError> {
        compute_fast_bounds(self, src, dst)
    }
    fn asABlur(&self, _rec: Option<&mut BlurRec>) -> bool {
        default_as_blur()
    }
    fn asImageFilter(
        &self,
        _ctm: &SkMatrix,
        _paint: &crate::src::core::SkPaint::SkPaint,
    ) -> (
        Option<std::sync::Arc<dyn crate::src::core::SkImageFilter::SkImageFilter>>,
        bool,
    ) {
        default_as_image_filter()
    }

    /// Safety: uphold destination pixel lease and all native draw_nine blitter/clip contracts.
    unsafe fn filterRRect(
        &self,
        rrect: &crate::src::core::SkRRect::SkRRect,
        matrix: &SkMatrix,
        clip: &crate::src::core::SkRasterClip::SkRasterClip,
        mut blitter: core::pin::Pin<&mut dyn crate::src::core::SkBlitter::SkBlitter>,
        cache: Option<&mut crate::src::core::SkResourceCache::SkResourceCache>,
    ) -> Result<bool, crate::RasterError> {
        unsafe { filter_rrect(self, rrect, matrix, clip, blitter, cache) }
    }
    /// Safety: uphold destination pixel lease and all native draw_nine blitter/clip contracts.
    unsafe fn filterRects(
        &self,
        rects: &[SkRect],
        matrix: &SkMatrix,
        clip: &crate::src::core::SkRasterClip::SkRasterClip,
        mut blitter: core::pin::Pin<&mut dyn crate::src::core::SkBlitter::SkBlitter>,
        cache: Option<&mut crate::src::core::SkResourceCache::SkResourceCache>,
    ) -> Result<FilterReturn, crate::RasterError> {
        unsafe { filter_rects(self, rects, matrix, clip, blitter, cache) }
    }
    unsafe fn filterRectsToNine(
        &self,
        _rects: &[SkRect],
        _matrix: &SkMatrix,
        _clip: &crate::src::core::SkRect::SkIRect,
        _patch: core::pin::Pin<&mut NinePatchStorage>,
        _cache: Option<&mut crate::src::core::SkResourceCache::SkResourceCache>,
    ) -> Result<FilterReturn, crate::RasterError> {
        Ok(default_rects_to_nine())
    }
    unsafe fn filterRRectToNine(
        &self,
        _rrect: &crate::src::core::SkRRect::SkRRect,
        _matrix: &SkMatrix,
        _clip: &crate::src::core::SkRect::SkIRect,
        _patch: core::pin::Pin<&mut NinePatchStorage>,
        _cache: Option<&mut crate::src::core::SkResourceCache::SkResourceCache>,
    ) -> Result<bool, crate::RasterError> {
        Ok(default_rrect_to_nine())
    }
}

// Source: src/core/SkMaskFilterBase.h:63-68
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    kBlur,
    kEmboss,
    kSDF,
    kShader,
    kTable,
}
// Source: src/core/SkMaskFilterBase.h:86-88
#[derive(Clone, Copy)]
pub struct BlurRec {
    pub fSigma: f32,
    pub fStyle: SkBlurStyle,
}
// Source: src/core/SkMaskFilterBase.h:124-127
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterReturn {
    kFalse,
    kTrue,
    kUnimplemented,
}
// Source: src/core/SkMaskFilterBase.h:130-140
// No Copy/Clone or movable initialized constructor: this native object deletes
// move construction because its cache/image ownership cannot be transferred.
pub struct NinePatch {
    pub fMask: SkMask,
    pub fOuterRect: crate::src::core::SkRect::SkIRect,
    pub fCenter: SkIPoint,
    fCache: Option<crate::src::core::SkCachedData::SkCachedData<()>>,
    _pin: core::marker::PhantomPinned,
}
// Inline std::optional storage. The empty slot can move before it is pinned;
// emplacement writes each field only at the caller's final pinned address.
// No Box or second mask/cache owner substitutes for the native inline object.
pub struct NinePatchStorage {
    value: core::mem::MaybeUninit<NinePatch>,
    initialized: bool,
    _pin: core::marker::PhantomPinned,
}
impl NinePatchStorage {
    pub fn empty() -> Self {
        Self {
            value: core::mem::MaybeUninit::uninit(),
            initialized: false,
            _pin: core::marker::PhantomPinned,
        }
    }
    pub fn as_patch(self: core::pin::Pin<&Self>) -> Option<core::pin::Pin<&NinePatch>> {
        if self.initialized {
            Some(unsafe { core::pin::Pin::new_unchecked(self.get_ref().value.assume_init_ref()) })
        } else {
            None
        }
    }
    pub fn is_none(self: core::pin::Pin<&Self>) -> bool {
        !self.initialized
    }
    // Source: src/core/SkMaskFilterBase.h:132-134
    /// Safety: with no cache transfer the malloc-compatible mask image; otherwise
    /// transfer exactly its matching native cache client reference. The caller
    /// keeps this original pinned inline storage alive through every patch use.
    pub unsafe fn emplace(
        self: core::pin::Pin<&mut Self>,
        mask: &SkMask,
        outer: crate::src::core::SkRect::SkIRect,
        center: SkIPoint,
        cache: Option<crate::src::core::SkCachedData::SkCachedData<()>>,
    ) {
        let slot = unsafe { self.get_unchecked_mut() };
        if slot.initialized {
            slot.initialized = false;
            unsafe {
                core::ptr::drop_in_place(slot.value.as_mut_ptr());
            }
        }
        let dst = slot.value.as_mut_ptr();
        unsafe {
            core::ptr::addr_of_mut!((*dst).fMask).write(*mask);
            core::ptr::addr_of_mut!((*dst).fOuterRect).write(outer);
            core::ptr::addr_of_mut!((*dst).fCenter).write(center);
            core::ptr::addr_of_mut!((*dst).fCache).write(cache);
            core::ptr::addr_of_mut!((*dst)._pin).write(core::marker::PhantomPinned);
        }
        slot.initialized = true;
    }
}
impl Drop for NinePatchStorage {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                core::ptr::drop_in_place(self.value.as_mut_ptr());
            }
        }
    }
}
// Source: src/core/SkMaskFilterBase.cpp:35-39
// Source: src/core/SkMaskFilterBase.cpp:41-41
impl Drop for NinePatch {
    fn drop(&mut self) {
        if let Some(cache) = &self.fCache {
            crate::SkASSERT!(@source b"src/core/SkMaskFilterBase.cpp\0".as_ptr(),37; @condition b"(const void*)fMask.fImage == fCache->data()\0".as_ptr(); self.fMask.fImage==cache.data());
        } else {
            unsafe {
                SkMaskBuilder::FreeImage(self.fMask.fImage.cast_mut().cast());
            }
        }
        // Dropping the owned cache field releases the transferred native client reference.
    }
}
// Source: src/core/SkMaskFilterBase.cpp:45-46
fn default_as_blur() -> bool {
    false
}
// Source: src/core/SkMaskFilterBase.cpp:49-51
fn default_as_image_filter() -> (
    Option<std::sync::Arc<dyn crate::src::core::SkImageFilter::SkImageFilter>>,
    bool,
) {
    (None, false)
}

// Source: src/core/SkMaskFilterBase.cpp:54-55
// Source: src/core/SkMaskFilterBase.cpp:57-63
pub(crate) unsafe fn extract_mask_subset(
    src: &SkMask,
    mut bounds: crate::src::core::SkRect::SkIRect,
    newX: i32,
    newY: i32,
) -> SkMask {
    crate::SkASSERT!(@source b"src/core/SkMaskFilterBase.cpp\0".as_ptr(),55; @condition b"src.fBounds.contains(bounds)\0".as_ptr(); src.fBounds.contains_rect(&bounds));
    let dx = bounds.fLeft - src.fBounds.fLeft;
    let dy = bounds.fTop - src.fBounds.fTop;
    bounds.offset_to(newX, newY);
    SkMask::new(
        unsafe {
            src.fImage.add(
                (dy as u32)
                    .wrapping_mul(src.fRowBytes)
                    .wrapping_add(dx as u32) as usize,
            )
        },
        bounds,
        src.fRowBytes,
        src.fFormat,
    )
}
// Source: src/core/SkMaskFilterBase.cpp:66-70
/// Safety: source mask storage is valid and disjoint from the blitter's exclusive destination;
/// every coordinate admitted by bounds intersect clipR lies in that writable destination.
pub(crate) unsafe fn blit_clipped_mask(
    mut blitter: core::pin::Pin<&mut dyn crate::src::core::SkBlitter::SkBlitter>,
    mask: &SkMask,
    bounds: &crate::src::core::SkRect::SkIRect,
    clipR: &crate::src::core::SkRect::SkIRect,
) {
    let mut r = crate::src::core::SkRect::SkIRect::default();
    if r.intersect_ab(*bounds, *clipR) {
        unsafe {
            blitter.as_mut().blitMask(mask, &r);
        }
    }
}
// Source: src/core/SkMaskFilterBase.cpp:74-77
/// Safety: caller holds the blitter's exclusive writable destination lease; every coordinate
/// in rect intersect clipR lies in that destination. Clipping alone does not establish a lease.
pub(crate) unsafe fn blit_clipped_rect(
    mut blitter: core::pin::Pin<&mut dyn crate::src::core::SkBlitter::SkBlitter>,
    rect: &crate::src::core::SkRect::SkIRect,
    clipR: &crate::src::core::SkRect::SkIRect,
) {
    let mut r = crate::src::core::SkRect::SkIRect::default();
    if r.intersect_ab(*rect, *clipR) {
        unsafe {
            blitter
                .as_mut()
                .blitRect(r.fLeft, r.fTop, r.width(), r.height());
        }
    }
}
// Source: src/core/SkMaskFilterBase.cpp:81-86
// Source: src/core/SkMaskFilterBase.cpp:89-94
// Source: src/core/SkMaskFilterBase.cpp:98-103
// Source: src/core/SkMaskFilterBase.cpp:107-112
// Source: src/core/SkMaskFilterBase.cpp:116-122
// Source: src/core/SkMaskFilterBase.cpp:125-131
// Source: src/core/SkMaskFilterBase.cpp:134-134
/// Safety: A8 mask storage is valid with zero-origin bounds and is disjoint from the blitter's
/// exclusive writable destination. Outer rect fits fixed edges, center lies in mask, and all
/// outerR intersect clipR coordinates lie in that destination. Scratch run lengths fit i16.
pub(crate) unsafe fn draw_nine_clipped(
    mask: &SkMask,
    outerR: &crate::src::core::SkRect::SkIRect,
    center: &SkIPoint,
    fillCenter: bool,
    clipR: &crate::src::core::SkRect::SkIRect,
    mut blitter: core::pin::Pin<&mut dyn crate::src::core::SkBlitter::SkBlitter>,
) {
    use crate::src::core::SkRect::SkIRect;
    let cx = center.fX;
    let cy = center.fY;
    let mut bounds = mask.fBounds;
    bounds.fRight = cx;
    bounds.fBottom = cy;
    if bounds.width() > 0 && bounds.height() > 0 {
        let m = unsafe { extract_mask_subset(mask, bounds, outerR.fLeft, outerR.fTop) };
        unsafe {
            blit_clipped_mask(blitter.as_mut(), &m, &m.fBounds, clipR);
        }
    }
    bounds = mask.fBounds;
    bounds.fLeft = cx + 1;
    bounds.fBottom = cy;
    if bounds.width() > 0 && bounds.height() > 0 {
        let m = unsafe {
            extract_mask_subset(mask, bounds, outerR.fRight - bounds.width(), outerR.fTop)
        };
        unsafe {
            blit_clipped_mask(blitter.as_mut(), &m, &m.fBounds, clipR);
        }
    }
    bounds = mask.fBounds;
    bounds.fRight = cx;
    bounds.fTop = cy + 1;
    if bounds.width() > 0 && bounds.height() > 0 {
        let m = unsafe {
            extract_mask_subset(mask, bounds, outerR.fLeft, outerR.fBottom - bounds.height())
        };
        unsafe {
            blit_clipped_mask(blitter.as_mut(), &m, &m.fBounds, clipR);
        }
    }
    bounds = mask.fBounds;
    bounds.fLeft = cx + 1;
    bounds.fTop = cy + 1;
    if bounds.width() > 0 && bounds.height() > 0 {
        let m = unsafe {
            extract_mask_subset(
                mask,
                bounds,
                outerR.fRight - bounds.width(),
                outerR.fBottom - bounds.height(),
            )
        };
        unsafe {
            blit_clipped_mask(blitter.as_mut(), &m, &m.fBounds, clipR);
        }
    }
    let innerR = SkIRect::make_ltrb(
        outerR.fLeft + cx - mask.fBounds.fLeft,
        outerR.fTop + cy - mask.fBounds.fTop,
        outerR.fRight + (cx + 1 - mask.fBounds.fRight),
        outerR.fBottom + (cy + 1 - mask.fBounds.fBottom),
    );
    if fillCenter {
        unsafe {
            blit_clipped_rect(blitter.as_mut(), &innerR, clipR);
        }
    }
    let innerW = innerR.width();
    // Adapted: src/core/SkMaskFilterBase.cpp:135-138
    // Native allocation choice and runs/alpha layout are retained. Rust slice validity requires
    // initialized gaps in the sparse runs, so scratch is zeroed before typed mutable views.
    let storageSize =
        (innerW as usize + 1) * (core::mem::size_of::<i16>() + core::mem::size_of::<u8>());
    let mut storage = crate::src::core::SkAutoMalloc::SkAutoSMalloc::<4096>::with_size(storageSize);
    let ptr = storage.get(); // final stack placement; do not reborrow/move storage until views expire
    unsafe {
        core::ptr::write_bytes(ptr, 0, storageSize);
    }
    let runs = unsafe { core::slice::from_raw_parts_mut(ptr.cast::<i16>(), innerW as usize + 1) };
    let alpha = unsafe {
        core::slice::from_raw_parts_mut(ptr.add((innerW as usize + 1) * 2), innerW as usize + 1)
    };
    // Source: src/core/SkMaskFilterBase.cpp:140-140
    // Source: src/core/SkMaskFilterBase.cpp:142-151
    // Source: src/core/SkMaskFilterBase.cpp:155-164
    // Source: src/core/SkMaskFilterBase.cpp:168-175
    // Source: src/core/SkMaskFilterBase.cpp:178-185
    let mut r = SkIRect::make_ltrb(innerR.fLeft, outerR.fTop, innerR.fRight, innerR.fTop);
    if r.intersect(clipR) {
        let startY = 0.max(r.fTop - outerR.fTop);
        let stopY = startY + r.height();
        let width = r.width() as usize;
        runs[0] = width as i16;
        runs[width] = 0;
        for y in startY..stopY {
            alpha[0] = unsafe { *mask.getAddr8(cx, mask.fBounds.fTop + y) };
            unsafe {
                blitter
                    .as_mut()
                    .blitAntiH(r.fLeft, outerR.fTop + y, alpha, runs);
            }
        }
    }
    r = SkIRect::make_ltrb(innerR.fLeft, innerR.fBottom, innerR.fRight, outerR.fBottom);
    if r.intersect(clipR) {
        let startY = outerR.fBottom - r.fBottom;
        let stopY = startY + r.height();
        let width = r.width() as usize;
        runs[0] = width as i16;
        runs[width] = 0;
        for y in startY..stopY {
            alpha[0] = unsafe { *mask.getAddr8(cx, mask.fBounds.fBottom - y - 1) };
            unsafe {
                blitter
                    .as_mut()
                    .blitAntiH(r.fLeft, outerR.fBottom - y - 1, alpha, runs);
            }
        }
    }
    r = SkIRect::make_ltrb(outerR.fLeft, innerR.fTop, innerR.fLeft, innerR.fBottom);
    if r.intersect(clipR) {
        let leftMask = SkMask::new(
            mask.getAddr8(
                mask.fBounds.fLeft + r.fLeft - outerR.fLeft,
                mask.fBounds.fTop + cy,
            ),
            r,
            0,
            Format::kA8_Format,
        );
        unsafe {
            blitter.as_mut().blitMask(&leftMask, &r);
        }
    }
    r = SkIRect::make_ltrb(innerR.fRight, innerR.fTop, outerR.fRight, innerR.fBottom);
    if r.intersect(clipR) {
        let rightMask = SkMask::new(
            mask.getAddr8(
                mask.fBounds.fRight - outerR.fRight + r.fLeft,
                mask.fBounds.fTop + cy,
            ),
            r,
            0,
            Format::kA8_Format,
        );
        unsafe {
            blitter.as_mut().blitMask(&rightMask, &r);
        }
    }
}
// Source: src/core/SkMaskFilterBase.cpp:189-190
// Source: src/core/SkMaskFilterBase.cpp:192-193
// Source: src/core/SkMaskFilterBase.cpp:195-195
// Source: src/core/SkMaskFilterBase.cpp:197-202
/// Safety: caller retains exclusive writable blitter destination for the entire call. The
/// mask/outer/center obey draw_nine_clipped's storage/disjointness/geometry contract and the raster
/// clip admits only coordinates inside that destination. Wrapper clipping cannot validate pixels.
pub(crate) unsafe fn draw_nine(
    mask: &SkMask,
    outerR: &crate::src::core::SkRect::SkIRect,
    center: &SkIPoint,
    fillCenter: bool,
    clip: &crate::src::core::SkRasterClip::SkRasterClip,
    mut blitter: core::pin::Pin<&mut dyn crate::src::core::SkBlitter::SkBlitter>,
) {
    let mut wrapper = core::pin::pin!(
        crate::src::core::SkRasterClip::SkAAClipBlitterWrapper::from_clip(clip, blitter)
    );
    // One split borrow after final wrapper placement keeps the inline AA blitter and fallback
    // region stable throughout clipping. No moves or later whole-wrapper mutable reborrows.
    let (region, mut blitter) = wrapper.as_mut().parts();
    let mut clipper = crate::src::core::SkRegion::Cliperator::new(region, *outerR);
    if !clipper.done() {
        loop {
            unsafe {
                draw_nine_clipped(
                    mask,
                    outerR,
                    center,
                    fillCenter,
                    clipper.rect(),
                    blitter.as_mut(),
                );
            }
            clipper.next();
            if clipper.done() {
                break;
            }
        }
    }
}
// Source: src/core/SkMaskFilterBase.cpp:206-208
// Source: src/core/SkMaskFilterBase.cpp:210-212
// Source: src/core/SkMaskFilterBase.cpp:214-214
pub(crate) fn countNestedRects(
    raw: &crate::src::core::SkPathRaw::SkPathRaw<'_>,
    rects: &mut [SkRect; 2],
) -> i32 {
    if crate::src::core::SkPathPriv::SkPathPriv::IsNestedFillRects(raw, Some(&mut *rects), None) {
        return 2;
    }
    if let Some(r) = raw.isRect() {
        rects[0] = r;
        return 1;
    }
    0
}
// Source: src/core/SkMaskFilterBase.cpp:217-221
// Source: src/core/SkMaskFilterBase.cpp:225-226
// Source: src/core/SkMaskFilterBase.cpp:228-229
// Source: src/core/SkMaskFilterBase.cpp:231-232
unsafe fn filter_rrect(
    filter: &(impl SkMaskFilterBase + ?Sized),
    rrect: &crate::src::core::SkRRect::SkRRect,
    matrix: &SkMatrix,
    clip: &crate::src::core::SkRasterClip::SkRasterClip,
    mut blitter: core::pin::Pin<&mut dyn crate::src::core::SkBlitter::SkBlitter>,
    cache: Option<&mut crate::src::core::SkResourceCache::SkResourceCache>,
) -> Result<bool, crate::RasterError> {
    let mut storage = core::pin::pin!(NinePatchStorage::empty());
    if !unsafe {
        filter.filterRRectToNine(rrect, matrix, &clip.get_bounds(), storage.as_mut(), cache)
    }? {
        return Ok(false);
    }
    let patch = storage
        .as_ref()
        .as_patch()
        .expect("native successful ninepatch initialized");
    unsafe {
        draw_nine(
            &patch.fMask,
            &patch.fOuterRect,
            &patch.fCenter,
            true,
            clip,
            blitter,
        );
    }
    Ok(true)
}
// Source: src/core/SkMaskFilterBase.cpp:235-240
// Source: src/core/SkMaskFilterBase.cpp:242-247
// Source: src/core/SkMaskFilterBase.cpp:249-252
// Source: src/core/SkMaskFilterBase.cpp:254-255
// Source: src/core/SkMaskFilterBase.cpp:257-257
// Source: src/core/SkMaskFilterBase.cpp:259-259
unsafe fn filter_rects(
    filter: &(impl SkMaskFilterBase + ?Sized),
    rects: &[SkRect],
    matrix: &SkMatrix,
    clip: &crate::src::core::SkRasterClip::SkRasterClip,
    mut blitter: core::pin::Pin<&mut dyn crate::src::core::SkBlitter::SkBlitter>,
    cache: Option<&mut crate::src::core::SkResourceCache::SkResourceCache>,
) -> Result<FilterReturn, crate::RasterError> {
    let mut patch = core::pin::pin!(NinePatchStorage::empty());
    let result = unsafe {
        filter.filterRectsToNine(rects, matrix, &clip.get_bounds(), patch.as_mut(), cache)
    }?;
    match result {
        FilterReturn::kFalse => {
            crate::SkASSERT!(@source b"src/core/SkMaskFilterBase.cpp\0".as_ptr(),246; @condition b"!patch.has_value()\0".as_ptr(); patch.as_ref().is_none())
        }
        FilterReturn::kTrue => {
            let patch = patch
                .as_ref()
                .as_patch()
                .expect("native successful ninepatch initialized");
            unsafe {
                draw_nine(
                    &patch.fMask,
                    &patch.fOuterRect,
                    &patch.fCenter,
                    rects.len() == 1,
                    clip,
                    blitter,
                );
            }
        }
        FilterReturn::kUnimplemented => {
            crate::SkASSERT!(@source b"src/core/SkMaskFilterBase.cpp\0".as_ptr(),255; @condition b"!patch.has_value()\0".as_ptr(); patch.as_ref().is_none())
        }
    }
    Ok(result)
}
impl dyn SkMaskFilterBase + '_ {
    // Source: src/core/SkMaskFilterBase.h:184-189
    // Source: src/core/SkMaskFilterBase.cpp:262-322
    /// Safety: retain native path/context and exclusive destination pixel leases
    /// through DrawToMask, filterMask and all clipped blitter calls.
    pub unsafe fn filterPath(
        &self,
        devRaw: &crate::src::core::SkPathRaw::SkPathRaw<'_>,
        matrix: &SkMatrix,
        clip: &crate::src::core::SkRasterClip::SkRasterClip,
        mut blitter: core::pin::Pin<&mut dyn crate::src::core::SkBlitter::SkBlitter>,
        style: crate::src::core::SkStrokeRec::InitStyle,
        cache: Option<&mut crate::src::core::SkResourceCache::SkResourceCache>,
    ) -> Result<bool, crate::RasterError> {
        use crate::src::core::SkMask::{CreateMode, SkAutoMaskFreeImage};
        let mut rects = [SkRect::default(); 2];
        let rectCount = if style == crate::src::core::SkStrokeRec::InitStyle::kFill_InitStyle {
            countNestedRects(devRaw, &mut rects)
        } else {
            0
        };
        if rectCount > 0 {
            match self.filterRects(
                &rects[..rectCount as usize],
                matrix,
                clip,
                blitter.as_mut(),
                cache,
            )? {
                FilterReturn::kFalse => return Ok(false),
                FilterReturn::kTrue => return Ok(true),
                FilterReturn::kUnimplemented => {}
            }
        }
        let mut srcM = SkMaskBuilder::new();
        let mut dstM = SkMaskBuilder::new();
        // Source: src/core/SkMaskFilterBase.cpp:286-290
        // UNVERIFIED: exact native fuzzer-only raw path complexity boundary.
        // It runs after nested-rect handling, before mask allocation/rendering.
        #[cfg(SK_BUILD_FOR_FUZZER)]
        if devRaw.verbs().len() > 1000 || devRaw.points().len() > 1000 {
            return Ok(false);
        }
        let drawn = crate::src::core::SkDraw::DrawToMask(
            devRaw,
            &clip.get_bounds(),
            self,
            matrix,
            &mut srcM,
            CreateMode::kComputeBoundsAndRenderImage_CreateMode,
            style,
        );
        // A Rust execution error can follow native allocation. Acquire the sole
        // image guard before propagating it, so partially rendered masks are freed.
        let _autoSrc = SkAutoMaskFreeImage::from_raw(srcM.mask.fImage.cast_mut());
        if !drawn? {
            return Ok(false);
        }
        let filtered = match self.filterMask(&mut dstM, &srcM.mask, matrix, None) {
            Ok(value) => value,
            Err(error) => {
                let _autoDst = SkAutoMaskFreeImage::from_raw(dstM.mask.fImage.cast_mut());
                return Err(error);
            }
        };
        if !filtered {
            return Ok(false);
        }
        let _autoDst = SkAutoMaskFreeImage::from_raw(dstM.mask.fImage.cast_mut());
        let mut wrapper = core::pin::pin!(
            crate::src::core::SkRasterClip::SkAAClipBlitterWrapper::from_clip(clip, blitter)
        );
        let (region, mut blitter) = wrapper.as_mut().parts();
        let mut clipper = crate::src::core::SkRegion::Cliperator::new(region, dstM.mask.fBounds);
        if !clipper.done() {
            loop {
                blitter.as_mut().blitMask(&dstM.mask, clipper.rect());
                clipper.next();
                if clipper.done() {
                    break;
                }
            }
        }
        Ok(true)
    }
}
// Source: src/core/SkMaskFilterBase.cpp:324-326
fn default_rrect_to_nine() -> bool {
    false
}
// Source: src/core/SkMaskFilterBase.cpp:329-334
fn default_rects_to_nine() -> FilterReturn {
    FilterReturn::kUnimplemented
}
// Source: src/core/SkMaskFilterBase.cpp:337-339
// Source: src/core/SkMaskFilterBase.cpp:341-345
// Source is copied before the destination write, permitting the original in-place call.
fn compute_fast_bounds(
    filter: &(impl SkMaskFilterBase + ?Sized),
    src: SkRect,
    dst: &mut SkRect,
) -> Result<(), crate::RasterError> {
    let srcM = SkMask::new(core::ptr::null(), src.roundOut(), 0, Format::kA8_Format);
    let mut dstM = SkMaskBuilder::new();
    let mut margin = SkIPoint::make(0, 0);
    if unsafe { filter.filterMask(&mut dstM, &srcM, &SkMatrix::default(), Some(&mut margin)) }? {
        dst.set(&dstM.fBounds);
    } else {
        dst.set(&srcM.fBounds);
    }
    Ok(())
}

// Source: src/core/SkMaskFilterBase.h:113-115
pub fn GetFlattenableType() -> crate::src::core::SkFlattenable::Type {
    crate::src::core::SkFlattenable::Type::kSkMaskFilter_Type
}
// Source: src/core/SkMaskFilterBase.h:208-210
// Native public/base classes share the same actual Rust trait, so the checked
// view changes no data pointer/vtable, retains null, and adds no owner/counter.
pub fn as_MFB_mut(
    filter: Option<&mut dyn super::SkMaskFilter::SkMaskFilter>,
) -> Option<&mut dyn SkMaskFilterBase> {
    filter
}
// Source: src/core/SkMaskFilterBase.h:212-214
pub fn as_MFB(
    filter: Option<&dyn super::SkMaskFilter::SkMaskFilter>,
) -> Option<&dyn SkMaskFilterBase> {
    filter
}
// Source: src/core/SkMaskFilterBase.h:216-218
pub fn as_MFB_shared(
    filter: &Option<std::sync::Arc<dyn super::SkMaskFilter::SkMaskFilter>>,
) -> Option<&dyn SkMaskFilterBase> {
    filter.as_deref()
}
// Source: src/core/SkMaskFilterBase.h:221-221
pub use super::SkBlurMaskFilterImpl::sk_register_blur_maskfilter_createproc;
