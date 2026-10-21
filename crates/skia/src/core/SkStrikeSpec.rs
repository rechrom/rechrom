use crate::src::core::SkFont::SkFont;
use crate::src::core::SkFontPriv::SkFontPriv;
use crate::src::core::SkMatrix::SkMatrix;
use crate::src::core::SkPaint::{SkPaint, Style};

use crate::include::core::SkRefCnt::sk_sp;
use crate::src::core::SkDescriptor::{SkAutoDescriptor, SkDescriptor};
use crate::src::core::SkMaskFilter::SkMaskFilter;
use crate::src::core::SkPathEffect::SkPathEffect;
use crate::src::core::SkTypeface::{SkTypeface, SkTypefaceRef};
use std::sync::Arc;
impl SkStrikeSpec {
    // Source: src/core/SkStrikeSpec.h:90-93
    /// Safety: retain actual typeface/backend/effect leases until the returned scaler drops.
    pub unsafe fn createScalerContext(
        &self,
    ) -> Box<dyn crate::src::core::SkScalerContext::SkScalerContextVirtual> {
        let effects = crate::src::core::SkScalerContext::SkScalerContextEffects::new(
            self.fPathEffect.get(),
            self.fMaskFilter.as_deref(),
        );
        unsafe {
            // Source: src/core/SkStrikeSpec.h:90-93
            // Borrow the existing native strong handle for the owner receiver;
            // this call neither retains another face nor adopts a raw borrow.
            self.fTypeface
                .as_ref()
                .expect("native nonnull typeface required")
                .createScalerContext(
                    &effects,
                    self.descriptor(),
                    &self.fPathEffect,
                    self.fMaskFilter.as_ref(),
                )
        }
    }
}
impl SkStrikeSpec {
    // Source: src/core/SkStrikeSpec.h:95-95
    pub fn descriptor(&self) -> &SkDescriptor {
        self.fAutoDescriptor.get_desc()
    }
}
impl SkStrikeSpec {
    // Source: src/core/SkStrikeSpec.h:96-96
    pub fn typeface(&self) -> &SkTypeface {
        self.fTypeface
            .as_deref()
            .expect("native nonnull typeface required")
    }
}
impl SkStrikeSpec {
    /// Associated native entry point; algorithm lives immediately below in source order.
    pub fn ShouldDrawAsPath(paint: &SkPaint, font: &SkFont, matrix: &SkMatrix) -> bool {
        ShouldDrawAsPath(paint, font, matrix)
    }
}
// Adapted source: src/core/SkStrikeSpec.h:39-40: Rust owning storage declaration separated from native class inline interface scope
// Source: src/core/SkStrikeSpec.h:108-112
// MaskFilter retains its current canonical Arc bridge; Typeface and PathEffect
// use actual native strong handles. Field destruction order remains explicit.
pub struct SkStrikeSpec {
    fAutoDescriptor: SkAutoDescriptor,
    fMaskFilter: Option<Arc<dyn SkMaskFilter>>,
    fPathEffect: sk_sp<dyn SkPathEffect>,
    fTypeface: Option<SkTypefaceRef>,
}
impl SkStrikeSpec {
    // Source: src/core/SkStrikeSpec.cpp:23-25
    pub fn from_descriptor(descriptor: &SkDescriptor, typeface: Option<SkTypefaceRef>) -> Self {
        Self {
            fAutoDescriptor: SkAutoDescriptor::from_descriptor(descriptor),
            fMaskFilter: None,
            fPathEffect: sk_sp::default(),
            fTypeface: typeface,
        }
    }
}
// Source: src/core/SkStrikeSpec.cpp:27-27
impl Clone for SkStrikeSpec {
    fn clone(&self) -> Self {
        Self {
            fAutoDescriptor: self.fAutoDescriptor.clone(),
            fMaskFilter: self.fMaskFilter.clone(),
            fPathEffect: self.fPathEffect.clone(),
            fTypeface: self.fTypeface.clone(),
        }
    }
}
impl SkStrikeSpec {
    // Source: src/core/SkStrikeSpec.cpp:28-28
    pub fn move_from(that: &mut Self) -> Self {
        Self {
            fAutoDescriptor: SkAutoDescriptor::take_from(&mut that.fAutoDescriptor),
            fMaskFilter: that.fMaskFilter.take(),
            fPathEffect: sk_sp::move_from(&mut that.fPathEffect),
            fTypeface: that.fTypeface.take(),
        }
    }
}
// Source: src/core/SkStrikeSpec.cpp:29-29
impl Drop for SkStrikeSpec {
    fn drop(&mut self) {
        drop(self.fTypeface.take());
        drop(std::mem::take(&mut self.fPathEffect));
        drop(self.fMaskFilter.take());
        // Descriptor field destruction follows, after the three native owning handles.
    }
}
impl SkStrikeSpec {
    // Source: src/core/SkStrikeSpec.cpp:31-37
    // Source: src/core/SkStrikeSpec.h:51-56
    pub fn MakeMask(
        font: &SkFont,
        paint: &SkPaint,
        props: &crate::include::core::SkSurfaceProps::SkSurfaceProps,
        flags: crate::src::core::SkScalerContext::SkScalerContextFlags,
        matrix: &SkMatrix,
    ) -> Result<Self, crate::RasterError> {
        Self::from_font(font, paint, props, flags, matrix)
    }
    // Source: src/core/SkStrikeSpec.cpp:39-47
    // Source: src/core/SkStrikeSpec.h:60-65
    pub fn MakeTransformMask(
        font: &SkFont,
        paint: &SkPaint,
        props: &crate::include::core::SkSurfaceProps::SkSurfaceProps,
        flags: crate::src::core::SkScalerContext::SkScalerContextFlags,
        matrix: &SkMatrix,
    ) -> Result<Self, crate::RasterError> {
        let mut source = font.clone();
        source.setSubpixel(false);
        Self::from_font(&source, paint, props, flags, matrix)
    }
    // Source: src/core/SkStrikeSpec.cpp:49-67
    // Source: src/core/SkStrikeSpec.h:68-72
    pub fn MakePath(
        font: &SkFont,
        paint: &SkPaint,
        props: &crate::include::core::SkSurfaceProps::SkSurfaceProps,
        flags: crate::src::core::SkScalerContext::SkScalerContextFlags,
    ) -> Result<(Self, f32), crate::RasterError> {
        let mut path_paint = paint.clone();
        let mut path_font = font.clone();
        path_font.setSubpixel(false);
        let scale = path_font.setupForAsPaths(Some(&mut path_paint));
        Ok((
            Self::from_font(&path_font, &path_paint, props, flags, &SkMatrix::I())?,
            scale,
        ))
    }
    // Source: src/core/SkStrikeSpec.cpp:69-89
    pub fn MakeCanonicalized(
        font: &SkFont,
        paint: Option<&SkPaint>,
    ) -> Result<(Self, f32), crate::RasterError> {
        let mut canonical_paint = paint.cloned().unwrap_or_default();
        let mut path_font = None;
        let mut scale = 1.0;
        if ShouldDrawAsPath(&canonical_paint, font, &SkMatrix::I()) {
            let mut path = font.clone();
            scale = path.setupForAsPaths(None);
            path_font = Some(path);
            canonical_paint.reset();
        }
        let canonical_font = path_font.as_ref().unwrap_or(font);
        Ok((Self::from_font(canonical_font,&canonical_paint,&crate::include::core::SkSurfaceProps::SkSurfaceProps::default(),
            crate::src::core::SkScalerContext::SkScalerContextFlags::FAKE_GAMMA_AND_BOOST_CONTRAST,&SkMatrix::I())?,scale))
    }
    // Source: src/core/SkStrikeSpec.cpp:91-99
    pub fn MakeWithNoDevice(
        font: &SkFont,
        paint: Option<&SkPaint>,
        flags: crate::src::core::SkScalerContext::SkScalerContextFlags,
    ) -> Result<Self, crate::RasterError> {
        let setup = paint.cloned().unwrap_or_default();
        Self::from_font(
            font,
            &setup,
            &crate::include::core::SkSurfaceProps::SkSurfaceProps::default(),
            flags,
            &SkMatrix::I(),
        )
    }
}

// Language forwarding keeps the published module entry point and native associated entry
// point on this one algorithm.
// Source: src/core/SkStrikeSpec.cpp:101-102
// Source: src/core/SkStrikeSpec.cpp:105-107
// Source: src/core/SkStrikeSpec.cpp:110-112
// Source: src/core/SkStrikeSpec.cpp:114-115
// Source: src/core/SkStrikeSpec.cpp:118-119
// Source: src/core/SkStrikeSpec.cpp:121-123
// Source: src/core/SkStrikeSpec.cpp:125-127
pub fn ShouldDrawAsPath(paint: &SkPaint, font: &SkFont, view_matrix: &SkMatrix) -> bool {
    if paint.getStyle() == Style::kStroke_Style && paint.getStrokeWidth() == 0. {
        return true;
    }
    if view_matrix.hasPerspective() {
        return true;
    }
    let mut text_matrix = SkFontPriv::MakeTextMatrixForFont(font);
    text_matrix.postConcat(view_matrix);
    const MEMORY_LIMIT: f32 = 256.;
    const MAX_SIZE_SQUARED: f32 = MEMORY_LIMIT * MEMORY_LIMIT;
    let distance = |x: f32, y: f32| x * x + y * y;
    distance(text_matrix.getScaleX(), text_matrix.getSkewY()) > MAX_SIZE_SQUARED
        || distance(text_matrix.getSkewX(), text_matrix.getScaleY()) > MAX_SIZE_SQUARED
}

impl SkStrikeSpec {
    // Source: src/core/SkStrikeSpec.cpp:133-146
    fn from_font(
        font: &SkFont,
        paint: &SkPaint,
        props: &crate::include::core::SkSurfaceProps::SkSurfaceProps,
        flags: crate::src::core::SkScalerContext::SkScalerContextFlags,
        matrix: &SkMatrix,
    ) -> Result<Self, crate::RasterError> {
        let (descriptor, effects) =
            crate::src::core::SkScalerContext::CreateDescriptorAndEffectsUsingPaint(
                font, paint, props, flags, matrix,
            )?;
        // MakeRecAndEffects borrows these exact Paint owners. Preserve native ref
        // increments through canonical handles, rather than reconstructing Arc from a borrow.
        let mask = paint.refMaskFilter();
        let path = paint.refPathEffect();
        debug_assert!(match (effects.fMaskFilter, mask.as_ref()) {
            (None, None) => true,
            (Some(a), Some(b)) => std::ptr::addr_eq(a, b.as_ref()),
            _ => false,
        });
        debug_assert!(match (effects.fPathEffect, path.get()) {
            (None, None) => true,
            (Some(a), Some(b)) => std::ptr::addr_eq(a, b),
            _ => false,
        });
        Ok(Self {
            fAutoDescriptor: descriptor,
            fMaskFilter: mask,
            fPathEffect: path,
            fTypeface: Some(font.refTypeface()),
        })
    }
}

impl SkStrikeSpec {
    // Source: src/core/SkStrikeSpec.cpp:153-155
    /// Safety: the full retained backend/effect graph obeys the cache's cross-thread owner contract.
    // Source: src/core/SkStrikeSpec.h:86
    pub unsafe fn findOrCreateStrike(&self) -> Arc<crate::src::core::SkStrike::SkStrike> {
        unsafe {
            crate::src::core::SkStrikeCache::SkStrikeCache::GlobalStrikeCache()
                .findOrCreateStrike(self)
        }
    }
    // Source: src/core/SkStrikeSpec.cpp:157-159
    /// Safety: same retained-resource transfer contract as findOrCreateStrike;
    /// an explicit cache remains alive and stationary for subsequent cache-using
    /// strike operations (including accounting after unlock), not mere Arc drop.
    // Source: src/core/SkStrikeSpec.h:88
    pub unsafe fn findOrCreateStrikeIn(
        &self,
        cache: &crate::src::core::SkStrikeCache::SkStrikeCache,
    ) -> Arc<crate::src::core::SkStrike::SkStrike> {
        unsafe { cache.findOrCreateStrike(self) }
    }
}
// Source: src/core/SkStrikeSpec.h:122-125
// Actual retained strike and native inline20/heap scratch. Pin<Box> keeps inline
// storage stable when the Rust owner moves; this extra pin allocation is adapted.
// Strike drops before pointer scratch, matching native reverse member destruction.
// Source: src/core/SkStrikeSpec.cpp:164-164
// Native default destructor is actual Rust drop glue: fStrike first, fGlyphs second.
pub struct SkBulkGlyphMetrics {
    fStrike: Arc<crate::src::core::SkStrike::SkStrike>,
    fGlyphs: std::pin::Pin<
        Box<
            crate::include::private::SkTemplates::AutoSTArray<
                std::ptr::NonNull<crate::src::core::SkGlyph::SkGlyph>,
                20,
            >,
        >,
    >,
}
impl SkBulkGlyphMetrics {
    // Source: src/core/SkStrikeSpec.cpp:161-162
    /// Safety: same real cross-thread owner and synchronous glyph leases as the strike cache.
    pub unsafe fn new(spec: &SkStrikeSpec) -> Self {
        Self {
            fGlyphs: Box::pin(crate::include::private::SkTemplates::AutoSTArray::new()),
            fStrike: unsafe { spec.findOrCreateStrike() },
        }
    }
    // Source: src/core/SkStrikeSpec.cpp:166-169
    /// Safety: retained strike and backing remain alive; no incompatible mutation aliases glyph reads.
    pub unsafe fn glyphs(
        &mut self,
        ids: &[u16],
    ) -> &[std::ptr::NonNull<crate::src::core::SkGlyph::SkGlyph>] {
        // Native scratch entries are unspecified until metrics fills every slot;
        // dangling placeholders are never dereferenced or returned before that fill.
        self.fGlyphs.as_mut().reset_with_constructor(
            i32::try_from(ids.len()).expect("native scratch count fits int"),
            std::ptr::NonNull::dangling,
        );
        unsafe {
            self.fStrike
                .metrics(ids, self.fGlyphs.as_mut().as_mut_slice().unwrap());
        }
        self.fGlyphs.as_ref().get_ref().as_slice().unwrap()
    }
    /// Safety: complete native transfer/operation leases hold, and consume does
    /// not reenter the same strike or retain references after it returns.
    pub(crate) unsafe fn withGlyphs<R>(
        &mut self,
        ids: &[u16],
        consume: impl FnOnce(&[std::ptr::NonNull<crate::src::core::SkGlyph::SkGlyph>]) -> R,
    ) -> R {
        self.fGlyphs.as_mut().reset_with_constructor(
            i32::try_from(ids.len()).expect("native scratch count fits int"),
            std::ptr::NonNull::dangling,
        );
        unsafe {
            self.fStrike
                .withMetrics(ids, self.fGlyphs.as_mut().as_mut_slice().unwrap(), consume)
        }
    }
    // Source: src/core/SkStrikeSpec.cpp:171-173
    pub unsafe fn glyph(
        &mut self,
        id: u16,
    ) -> std::ptr::NonNull<crate::src::core::SkGlyph::SkGlyph> {
        unsafe { self.glyphs(&[id])[0] }
    }
}

// Source: src/core/SkStrikeSpec.h:138-140
// Strike drops before pinned native inline20/heap scratch; the pin allocation is Rust adaptation.
// Source: src/core/SkStrikeSpec.cpp:181-181
// Native default destructor is actual Rust drop glue: fStrike first, fGlyphs second.
pub struct SkBulkGlyphMetricsAndPaths {
    fStrike: Arc<crate::src::core::SkStrike::SkStrike>,
    fGlyphs: std::pin::Pin<
        Box<
            crate::include::private::SkTemplates::AutoSTArray<
                std::ptr::NonNull<crate::src::core::SkGlyph::SkGlyph>,
                20,
            >,
        >,
    >,
}
impl SkBulkGlyphMetricsAndPaths {
    // Source: src/core/SkStrikeSpec.cpp:175-176
    /// Safety: retain the same complete native cross-thread graph and synchronous glyph leases as the cache.
    pub unsafe fn new(spec: &SkStrikeSpec) -> Self {
        Self {
            fGlyphs: Box::pin(crate::include::private::SkTemplates::AutoSTArray::new()),
            fStrike: unsafe { spec.findOrCreateStrike() },
        }
    }
    // Source: src/core/SkStrikeSpec.cpp:178-179
    pub fn from_strike(strike: Arc<crate::src::core::SkStrike::SkStrike>) -> Self {
        Self {
            fGlyphs: Box::pin(crate::include::private::SkTemplates::AutoSTArray::new()),
            fStrike: strike,
        }
    }
    // Source: src/core/SkStrikeSpec.cpp:183-186
    /// Safety: no incompatible glyph/pixel mutation aliases the returned pointer-span lease.
    pub unsafe fn glyphs(
        &mut self,
        ids: &[u16],
    ) -> &[std::ptr::NonNull<crate::src::core::SkGlyph::SkGlyph>] {
        self.fGlyphs.as_mut().reset_with_constructor(
            i32::try_from(ids.len()).expect("native scratch count fits int"),
            std::ptr::NonNull::dangling,
        );
        unsafe {
            self.fStrike
                .preparePaths(ids, self.fGlyphs.as_mut().as_mut_slice().unwrap());
        }
        self.fGlyphs.as_ref().get_ref().as_slice().unwrap()
    }
    // Source: src/core/SkStrikeSpec.cpp:188-190
    pub unsafe fn glyph(
        &mut self,
        id: u16,
    ) -> std::ptr::NonNull<crate::src::core::SkGlyph::SkGlyph> {
        unsafe { self.glyphs(&[id])[0] }
    }
}
impl SkBulkGlyphMetricsAndPaths {
    // Source: src/core/SkStrikeSpec.cpp:192-198
    /// Safety: glyph is a path-prepared member of this retained strike and native intercept/output leases hold.
    pub unsafe fn findIntercepts(
        &self,
        bounds: &[f32; 2],
        scale: f32,
        x_pos: f32,
        glyph: std::ptr::NonNull<crate::src::core::SkGlyph::SkGlyph>,
        array: Option<&mut [f32]>,
        count: &mut i32,
    ) {
        unsafe {
            self.fStrike
                .findIntercepts(bounds, scale, x_pos, glyph, array, count)
        }
    }
}
// Source: src/core/SkStrikeSpec.h:152-154
// Strike drops before pinned native inline20/heap scratch; the pin allocation is Rust adaptation.
// Source: src/core/SkStrikeSpec.cpp:206-206
// Native default destructor is actual Rust drop glue: fStrike first, fGlyphs second.
pub struct SkBulkGlyphMetricsAndDrawables {
    fStrike: Arc<crate::src::core::SkStrike::SkStrike>,
    fGlyphs: std::pin::Pin<
        Box<
            crate::include::private::SkTemplates::AutoSTArray<
                std::ptr::NonNull<crate::src::core::SkGlyph::SkGlyph>,
                20,
            >,
        >,
    >,
}
impl SkBulkGlyphMetricsAndDrawables {
    // Source: src/core/SkStrikeSpec.cpp:200-201
    /// Safety: retain the same complete native cross-thread graph and synchronous glyph leases as the cache.
    pub unsafe fn new(spec: &SkStrikeSpec) -> Self {
        Self {
            fGlyphs: Box::pin(crate::include::private::SkTemplates::AutoSTArray::new()),
            fStrike: unsafe { spec.findOrCreateStrike() },
        }
    }
    // Source: src/core/SkStrikeSpec.cpp:203-204
    pub fn from_strike(strike: Arc<crate::src::core::SkStrike::SkStrike>) -> Self {
        Self {
            fGlyphs: Box::pin(crate::include::private::SkTemplates::AutoSTArray::new()),
            fStrike: strike,
        }
    }
    // Source: src/core/SkStrikeSpec.cpp:208-211
    /// Safety: no incompatible glyph/pixel mutation aliases the returned pointer-span lease.
    pub unsafe fn glyphs(
        &mut self,
        ids: &[u16],
    ) -> &[std::ptr::NonNull<crate::src::core::SkGlyph::SkGlyph>] {
        self.fGlyphs.as_mut().reset_with_constructor(
            i32::try_from(ids.len()).expect("native scratch count fits int"),
            std::ptr::NonNull::dangling,
        );
        unsafe {
            self.fStrike
                .prepareDrawables(ids, self.fGlyphs.as_mut().as_mut_slice().unwrap());
        }
        self.fGlyphs.as_ref().get_ref().as_slice().unwrap()
    }
    // Source: src/core/SkStrikeSpec.cpp:213-215
    pub unsafe fn glyph(
        &mut self,
        id: u16,
    ) -> std::ptr::NonNull<crate::src::core::SkGlyph::SkGlyph> {
        unsafe { self.glyphs(&[id])[0] }
    }
}
// Source: src/core/SkStrikeSpec.h:167-169
// Strike drops before pinned native inline64/heap scratch; the pin allocation is Rust adaptation.
// Source: src/core/SkStrikeSpec.cpp:223-223
// Native default destructor is actual Rust drop glue: fStrike first, fGlyphs second.
pub struct SkBulkGlyphMetricsAndImages {
    fStrike: Arc<crate::src::core::SkStrike::SkStrike>,
    fGlyphs: std::pin::Pin<
        Box<
            crate::include::private::SkTemplates::AutoSTArray<
                std::ptr::NonNull<crate::src::core::SkGlyph::SkGlyph>,
                64,
            >,
        >,
    >,
}
impl SkBulkGlyphMetricsAndImages {
    // Source: src/core/SkStrikeSpec.cpp:217-218
    /// Safety: retain the same complete native cross-thread graph and synchronous glyph leases as the cache.
    pub unsafe fn new(spec: &SkStrikeSpec) -> Self {
        Self {
            fGlyphs: Box::pin(crate::include::private::SkTemplates::AutoSTArray::new()),
            fStrike: unsafe { spec.findOrCreateStrike() },
        }
    }
    // Source: src/core/SkStrikeSpec.cpp:220-221
    pub fn from_strike(strike: Arc<crate::src::core::SkStrike::SkStrike>) -> Self {
        Self {
            fGlyphs: Box::pin(crate::include::private::SkTemplates::AutoSTArray::new()),
            fStrike: strike,
        }
    }
    // Source: src/core/SkStrikeSpec.cpp:225-228
    /// Safety: no incompatible glyph/pixel mutation aliases the returned pointer-span lease.
    pub unsafe fn glyphs(
        &mut self,
        ids: &[crate::src::core::SkGlyph::SkPackedGlyphID],
    ) -> Result<&[std::ptr::NonNull<crate::src::core::SkGlyph::SkGlyph>], crate::RasterError> {
        self.fGlyphs.as_mut().reset_with_constructor(
            i32::try_from(ids.len()).expect("native scratch count fits int"),
            std::ptr::NonNull::dangling,
        );
        unsafe {
            self.fStrike
                .prepareImages(ids, self.fGlyphs.as_mut().as_mut_slice().unwrap())?;
        }
        Ok(self.fGlyphs.as_ref().get_ref().as_slice().unwrap())
    }
    // Source: src/core/SkStrikeSpec.cpp:230-232
    pub unsafe fn glyph(
        &mut self,
        id: crate::src::core::SkGlyph::SkPackedGlyphID,
    ) -> Result<std::ptr::NonNull<crate::src::core::SkGlyph::SkGlyph>, crate::RasterError> {
        Ok(unsafe { self.glyphs(&[id])? }[0])
    }
}
impl SkBulkGlyphMetricsAndImages {
    // Source: src/core/SkStrikeSpec.cpp:234-236
    pub fn descriptor(&self) -> &SkDescriptor {
        self.fStrike.getDescriptor()
    }
}

impl SkStrikeSpec {
    // Source: src/core/SkStrikeSpec.h:75-76
    // Rust has no default arguments; retain the native omitted-paint entry.
    pub fn MakeCanonicalizedDefault(font: &SkFont) -> Result<(Self, f32), crate::RasterError> {
        Self::MakeCanonicalized(font, None)
    }
    // Source: src/core/SkStrikeSpec.h:79-81
    pub fn MakeWithNoDeviceDefault(font: &SkFont) -> Result<Self, crate::RasterError> {
        Self::MakeWithNoDeviceWithPaint(font, None)
    }
    // Source: src/core/SkStrikeSpec.h:79-81
    pub fn MakeWithNoDeviceWithPaint(
        font: &SkFont,
        paint: Option<&SkPaint>,
    ) -> Result<Self, crate::RasterError> {
        Self::MakeWithNoDevice(
            font,
            paint,
            crate::src::core::SkScalerContext::SkScalerContextFlags::FAKE_GAMMA_AND_BOOST_CONTRAST,
        )
    }
}

#[cfg(all(target_vendor = "apple", target_arch = "aarch64"))]
impl SkStrikeSpec {
    // Source: src/core/SkStrikeSpec.h:98-98
    // Source: src/core/SkStrikeSpec.cpp:129-131
    pub fn dump(&self) -> crate::src::core::SkString::SkString {
        self.fAutoDescriptor.get_desc().dumpRec()
    }
}

impl SkStrikeSpec {
    // Source: src/core/SkStrikeSpec.h:83-84
    // Source: src/core/SkStrikeSpec.cpp:148-151
    /// Safety: the actual cache stays alive at its original address for later
    /// cache-using strike operations, and its retained resource graph satisfies
    /// the native transfer contract. Plain Strike drop performs no cache read.
    /// No borrowed reference is extended to the returned owner.
    pub unsafe fn findOrCreateScopedStrike(
        &self,
        cache: &dyn crate::src::text::StrikeForGPU::StrikeForGPUCacheInterface,
    ) -> Result<
        Option<std::sync::Arc<dyn crate::src::text::StrikeForGPU::StrikeForGPU>>,
        crate::RasterError,
    > {
        unsafe { cache.findOrCreateScopedStrike(self) }
    }
}
