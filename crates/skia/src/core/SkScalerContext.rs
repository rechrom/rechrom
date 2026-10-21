use crate::src::core::SkMask::Format;

use crate::src::core::SkMaskGamma::SkTMaskGamma;

impl SkScalerContextFlags {
    pub const NONE: Self = Self(0);
}

impl SkScalerContextFlags {
    pub const FAKE_GAMMA: Self = Self(1 << 0);
}

impl SkScalerContextFlags {
    pub const BOOST_CONTRAST: Self = Self(1 << 1);
}

impl SkScalerContextFlags {
    pub const FAKE_GAMMA_AND_BOOST_CONTRAST: Self =
        Self(Self::FAKE_GAMMA.0 | Self::BOOST_CONTRAST.0);
}

impl std::ops::BitOr for SkScalerContextFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for SkScalerContextFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::Not for SkScalerContextFlags {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0)
    }
}

impl std::ops::BitXor for SkScalerContextFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitOrAssign for SkScalerContextFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAndAssign for SkScalerContextFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl std::ops::BitXorAssign for SkScalerContextFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl SkScalerContextRec {
    pub const fn internal_gamma_from_external(g: f32) -> u8 {
        (g * (1 << 6) as f32) as u8
    }
}

impl SkScalerContextRec {
    pub const fn external_contrast_from_internal(c: u8) -> f32 {
        c as f32 / ((1 << 8) - 1) as f32
    }
}

impl SkScalerContextRec {
    pub const fn internal_contrast_from_external(c: f32) -> u8 {
        (c * ((1 << 8) - 1) as f32 + 0.5) as u8
    }
}

impl SkScalerContextRec {
    pub fn set_contrast(&mut self, c: f32) {
        use crate::include::core::SkSurfaceProps::SkSurfaceProps;
        debug_assert!(
            SkSurfaceProps::kMinContrastInclusive <= c
                && c <= SkSurfaceProps::kMaxContrastInclusive
        );
        self.fContrast = Self::internal_contrast_from_external(c);
    }
}

impl SkScalerContextRec {
    pub fn ignore_pre_blend(&mut self) {
        self.ignore_gamma();
        self.set_contrast(0.);
    }
}

impl SkScalerContextRec {
    pub fn get_luminance_color(&self) -> u32 {
        self.fLumBits
    }
}

pub const K_EFFECTS_SK_DESCRIPTOR_TAG: u32 = u32::from_be_bytes(*b"efct");

impl SkScalerContextRec {
    pub fn get_single_matrix(&self) -> crate::src::core::SkMatrix::SkMatrix {
        let mut matrix = self.get_local_matrix();
        matrix.postConcat(&self.get_matrix_from_2x2());
        matrix
    }
}

// Source: src/core/SkScalerContext.h:51-51
pub type SkMaskGamma = SkTMaskGamma<3, 3, 3>;

// Source: src/core/SkScalerContext.h:53-59
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct SkScalerContextFlags(pub u32);

// Source: src/core/SkScalerContext.h:69-73
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SkScalerContextRec {
    pub fTypefaceID: u32,
    pub fTextSize: f32,
    pub fPreScaleX: f32,
    pub fPreSkewX: f32,
    pub fPost2x2: [[f32; 2]; 2],
    pub fFrameWidth: f32,
    pub fMiterLimit: f32,
    pub fForegroundColor: u32,
    // Source: src/core/SkScalerContext.h:82-86
    fLumBits: u32,
    fDeviceGamma: u8,
    fReservedAlign2: u8,
    fContrast: u8,
    fReservedAlign: u8,
    // Adapted source: src/core/SkScalerContext.h:143-143: later native mask/bitfield/flags storage is interleaved with inline bodies; retain actual dense field order rather than invent storage types
    // Adapted source: src/core/SkScalerContext.h:145-147: later native mask/bitfield/flags storage is interleaved with inline bodies; retain actual dense field order rather than invent storage types
    // Adapted source: src/core/SkScalerContext.h:149-150: later native mask/bitfield/flags storage is interleaved with inline bodies; retain actual dense field order rather than invent storage types
    pub fMaskFormat: Format,
    // Two C++ nibble bitfields share this byte; no ABI identity claimed.
    stroke_join_cap: u8,
    pub fFlags: u16,
}

impl SkScalerContextRec {
    // Source: src/core/SkScalerContext.h:88-91
    // Source: src/core/SkScalerContext.h:93-98
    // Source: src/core/SkScalerContext.h:100-101
    pub const fn external_gamma_from_internal(g: u8) -> f32 {
        g as f32 / (1 << 6) as f32
    }
}

// Source: src/core/SkScalerContext.h:103-108
// Source: src/core/SkScalerContext.h:110-115
impl SkScalerContextRec {
    pub fn set_device_gamma(&mut self, g: f32) {
        use crate::include::core::SkSurfaceProps::SkSurfaceProps;
        debug_assert!(
            SkSurfaceProps::kMinGammaInclusive <= g && g < SkSurfaceProps::kMaxGammaExclusive
        );
        self.fDeviceGamma = Self::internal_gamma_from_external(g);
    }
}

// Source: src/core/SkScalerContext.h:126-129
// Source: src/core/SkScalerContext.h:135-138
impl SkScalerContextRec {
    pub fn ignore_gamma(&mut self) {
        self.set_luminance_color(0);
        self.set_device_gamma(1.);
    }
}

// Source: src/core/SkScalerContext.h:176-180
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreMatrixScale {
    Full,
    Vertical,
    VerticalInteger,
}

// Source: src/core/SkScalerContext.h:225-227
// Source: src/core/SkScalerContext.h:229-231
impl SkScalerContextRec {
    pub fn get_format(&self) -> Format {
        self.fMaskFormat
    }
}

// Source: src/core/SkScalerContext.h:245-245
impl Default for SkScalerContextEffects<'_> {
    fn default() -> Self {
        Self {
            fPathEffect: None,
            fMaskFilter: None,
        }
    }
}

impl<'a> SkScalerContextEffects<'a> {
    // Source: src/core/SkScalerContext.h:246-247
    pub fn new(
        path_effect: Option<&'a dyn crate::src::core::SkPathEffect::SkPathEffect>,
        mask_filter: Option<&'a dyn crate::src::core::SkMaskFilterBase::SkMaskFilterBase>,
    ) -> Self {
        Self {
            fPathEffect: path_effect,
            fMaskFilter: mask_filter,
        }
    }
}

impl<'a> SkScalerContextEffects<'a> {
    // Source: src/core/SkScalerContext.h:248-250
    pub fn from_paint(paint: &'a crate::src::core::SkPaint::SkPaint) -> Self {
        Self {
            fPathEffect: paint.getPathEffect(),
            fMaskFilter: paint.getMaskFilter(),
        }
    }
}

// Adapted source: src/core/SkScalerContext.h:244-244: Rust effects storage declaration is separate from preceding inline constructors
// Source: src/core/SkScalerContext.h:252-254
pub struct SkScalerContextEffects<'a> {
    pub fPathEffect: Option<&'a dyn crate::src::core::SkPathEffect::SkPathEffect>,
    pub fMaskFilter: Option<&'a dyn crate::src::core::SkMaskFilterBase::SkMaskFilterBase>,
}

// Source: src/core/SkScalerContext.h:258-264
// Source: src/core/SkScalerContext.h:268-270
// Source: src/core/SkScalerContext.h:274-275
// Source: src/core/SkScalerContext.h:279-281
// Source: src/core/SkScalerContext.h:283-284
// Source: src/core/SkScalerContext.h:287-289
// Rust module constants replace the nested C++ flag enums without an empty context object.
pub mod flags {
    pub const FRAME_AND_FILL: u16 = 0x0001;
    pub const UNUSED: u16 = 0x0002;
    pub const EMBEDDED_BITMAP_TEXT: u16 = 0x0004;
    pub const EMBOLDEN: u16 = 0x0008;
    pub const SUBPIXEL_POSITIONING: u16 = 0x0010;
    pub const FORCE_AUTOHINTING: u16 = 0x0020;
    pub const HINTING_SHIFT: u32 = 7;
    pub const HINTING_BIT1: u16 = 0x0080;
    pub const HINTING_BIT2: u16 = 0x0100;
    pub const LCD_VERTICAL: u16 = 0x0200;
    pub const LCD_BGR_ORDER: u16 = 0x0400;
    pub const GEN_A8_FROM_LCD: u16 = 0x0800;
    pub const LINEAR_METRICS: u16 = 0x1000;
    pub const BASELINE_SNAP: u16 = 0x2000;
    pub const NEEDS_FOREGROUND_COLOR: u16 = 0x4000;
    pub const HINTING_MASK: u16 = HINTING_BIT1 | HINTING_BIT2;
}

// Source: src/core/SkScalerContext.h:337-344
// Adapted lifetime/signature: the actual temporary default Paint owns no effects; the
// returned empty borrowed effect view therefore captures no reference to that Paint.
pub fn MakeRecAndEffectsFromFont(
    font: &crate::src::core::SkFont::SkFont,
) -> Result<
    (SkScalerContextRec, SkScalerContextEffects<'static>),
    crate::src::effects::colorfilters::SkColorFilterBase::ColorFilterEvaluationError,
> {
    let paint = crate::src::core::SkPaint::SkPaint::default();
    let props = crate::include::core::SkSurfaceProps::SkSurfaceProps::default();
    let (rec, effects) = MakeRecAndEffects(
        font,
        &paint,
        &props,
        SkScalerContextFlags::NONE,
        &crate::src::core::SkMatrix::SkMatrix::I(),
    )?;
    debug_assert!(effects.fPathEffect.is_none() && effects.fMaskFilter.is_none());
    Ok((rec, SkScalerContextEffects::default()))
}

// Source: src/core/SkScalerContext.h:387-390
pub struct GeneratedPath {
    pub path: crate::src::core::SkPath::SkPath,
    pub modified: bool,
}

// Source: src/core/SkScalerContext.h:391-398
pub struct GlyphMetrics {
    pub advance: crate::src::core::SkPoint::SkPoint,
    pub bounds: crate::src::core::SkRect::SkRect,
    pub maskFormat: Format,
    pub extraBits: u16,
    pub neverRequestPath: bool,
    pub computeFromPath: bool,
    pub generatedPath: Option<GeneratedPath>,
}

// Source: src/core/SkScalerContext.h:399-407
impl GlyphMetrics {
    pub fn new(format: Format) -> Self {
        Self {
            advance: crate::src::core::SkPoint::SkPoint::make(0.0, 0.0),
            bounds: crate::src::core::SkRect::SkRect::MakeLTRB(0.0, 0.0, 0.0, 0.0),
            maskFormat: format,
            extraBits: 0,
            neverRequestPath: false,
            computeFromPath: false,
            generatedPath: None,
        }
    }
}

// Source: src/core/SkScalerContext.h:483-484
pub const K_REC_SK_DESCRIPTOR_TAG: u32 = u32::from_be_bytes(*b"srec");

// Source: src/core/SkScalerContext.h:488-492
impl SkScalerContextRec {
    pub fn get_hinting(&self) -> crate::include::core::SkFontTypes::SkFontHinting {
        let hint = (self.fFlags & flags::HINTING_MASK) >> flags::HINTING_SHIFT;
        crate::include::core::SkFontTypes::SkFontHinting(hint as core::ffi::c_int)
    }
}

impl SkScalerContextRec {
    // Source: src/core/SkScalerContext.h:494-497
    pub fn set_hinting(&mut self, hinting: crate::include::core::SkFontTypes::SkFontHinting) {
        self.fFlags =
            (self.fFlags & !flags::HINTING_MASK) | ((hinting.0 as u16) << flags::HINTING_SHIFT);
    }
}

// Source: src/core/SkScalerContext.cpp:55-55
const kSkShowTextBlitCoverage: bool = false;

// Source: src/core/SkScalerContext.cpp:59-63
// Source: src/core/SkScalerContext.cpp:66-66
// Source: src/core/SkScalerContext.cpp:68-68
// Source: src/core/SkScalerContext.cpp:73-74
// Source: src/core/SkScalerContext.cpp:76-76
// Source: src/core/SkScalerContext.cpp:78-83
// Source: src/core/SkScalerContext.cpp:86-86
// Source: src/core/SkScalerContext.cpp:88-89
/// Safety: the descriptor's record entry must contain an initialized native-layout
/// SkScalerContextRec, including a valid mask-format enum value.
pub unsafe fn preprocess_rec(
    typeface: &crate::src::core::SkTypeface::SkTypeface,
    effects: &SkScalerContextEffects<'_>,
    desc: &crate::src::core::SkDescriptor::SkDescriptor,
) -> SkScalerContextRec {
    let bytes = desc
        .find_entry(K_REC_SK_DESCRIPTOR_TAG)
        .expect("native scaler record required");
    let mut rec = std::ptr::read(bytes.as_ptr().cast::<SkScalerContextRec>());
    typeface.filterRec(&mut rec);
    if effects.fMaskFilter.is_some() {
        rec.ignore_pre_blend()
    }
    let mut lum_color = rec.get_luminance_color();
    if rec.fMaskFormat == Format::kA8_Format {
        use crate::src::core::SkColor::{SkColorGetB, SkColorGetG, SkColorGetR, SkColorSetRGB};
        let lum = crate::src::core::SkColorData::SkComputeLuminance(
            SkColorGetR(lum_color),
            SkColorGetG(lum_color),
            SkColorGetB(lum_color),
        );
        lum_color = SkColorSetRGB(lum, lum, lum);
    }
    rec.set_luminance_color(lum_color);
    rec
}

// Source: src/core/SkScalerContext.h:381-381
// Source: src/core/SkScalerContext.h:459-471
// Source: src/core/SkScalerContext.h:480-480
// Exact native field order; explicit guarded teardown retains preblend, mask, path.
// Typeface remains a native nonowning pointer; it must remain live for operations,
// but may already have died when this context is destroyed.
pub struct SkScalerContext {
    // Preserve native field-group order: h385,464,467,468,472,480.
    fRec: SkScalerContextRec,
    fTypeface: core::ptr::NonNull<crate::src::core::SkTypeface::SkTypeface>,
    fPathEffect:
        crate::include::core::SkRefCnt::sk_sp<dyn crate::src::core::SkPathEffect::SkPathEffect>,
    fMaskFilter: Option<std::sync::Arc<dyn crate::src::core::SkMaskFilter::SkMaskFilter>>,
    fGenerateImageFromPath: bool,
    pub(crate) fPreBlend:
        core::mem::ManuallyDrop<crate::src::core::SkMaskGamma::SkTMaskPreBlend<3, 3, 3>>,
}
// Rust-only owner adaptation; no added native-line credit. The preblend's
// canonical parent Arc is a real cache lease, so release it before effects.
impl Drop for SkScalerContext {
    fn drop(&mut self) {
        struct Owner<'a, T>(&'a mut core::mem::ManuallyDrop<T>);
        impl<T> Drop for Owner<'_, T> {
            fn drop(&mut self) {
                unsafe {
                    core::mem::ManuallyDrop::drop(self.0);
                }
            }
        }
        let owners = (
            Owner(&mut self.fPreBlend),
            self.fMaskFilter.take(),
            crate::include::core::SkRefCnt::sk_sp::move_from(&mut self.fPathEffect),
        );
        drop(owners);
    }
}
impl SkScalerContext {
    // Source: src/core/SkScalerContext.cpp:91-107
    /// Safety: typeface and descriptor record are valid for preprocessing. Canonical
    /// effect handles designate exactly the borrowed effects; native scalar/glyph
    /// operation leases keep the typeface alive. Context destruction does not dereference it.
    pub unsafe fn new(
        typeface: core::ptr::NonNull<crate::src::core::SkTypeface::SkTypeface>,
        effects: &SkScalerContextEffects<'_>,
        desc: &crate::src::core::SkDescriptor::SkDescriptor,
        path_owner: &crate::include::core::SkRefCnt::sk_sp<
            dyn crate::src::core::SkPathEffect::SkPathEffect,
        >,
        mask_owner: Option<&std::sync::Arc<dyn crate::src::core::SkMaskFilter::SkMaskFilter>>,
    ) -> Self {
        // Arc cannot be recovered from an arbitrary borrowed trait pointer. These
        // explicit canonical handles perform the same two native ref increments.
        debug_assert!(match (effects.fPathEffect, path_owner.get()) {
            (Some(a), Some(b)) => std::ptr::addr_eq(a, b),
            (None, None) => true,
            _ => false,
        });
        debug_assert!(match (effects.fMaskFilter, mask_owner) {
            (Some(a), Some(b)) => std::ptr::addr_eq(a, b.as_ref()),
            (None, None) => true,
            _ => false,
        });
        let rec = unsafe { preprocess_rec(typeface.as_ref(), effects, desc) };
        let path = path_owner.clone();
        let mask = mask_owner.cloned();
        let generate_from_path = rec.fFrameWidth >= 0.0 || path.get().is_some();
        let preblend = if mask.is_some() {
            crate::src::core::SkMaskGamma::SkTMaskPreBlend::new()
        } else {
            get_mask_pre_blend(&rec)
        };
        Self {
            fRec: rec,
            fTypeface: typeface,
            fPathEffect: path,
            fMaskFilter: mask,
            fGenerateImageFromPath: generate_from_path,
            fPreBlend: core::mem::ManuallyDrop::new(preblend),
        }
    }
    // Source: src/core/SkScalerContext.h:294-294
    pub fn getTypeface(&self) -> *mut crate::src::core::SkTypeface::SkTypeface {
        self.fTypeface.as_ptr()
    }
    // Source: src/core/SkScalerContext.h:296-298
    pub fn getMaskFormat(&self) -> Format {
        self.fRec.fMaskFormat
    }
    // Source: src/core/SkScalerContext.h:300-302
    pub fn isSubpixel(&self) -> bool {
        self.fRec.fFlags & flags::SUBPIXEL_POSITIONING != 0
    }
    // Source: src/core/SkScalerContext.h:304-306
    pub fn isLinearMetrics(&self) -> bool {
        self.fRec.fFlags & flags::LINEAR_METRICS != 0
    }
    // Source: src/core/SkScalerContext.h:309-309
    pub fn isVertical(&self) -> bool {
        false
    }
    // Source: src/core/SkScalerContext.h:366-366
    pub fn getRec(&self) -> &SkScalerContextRec {
        &self.fRec
    }
    // Source: src/core/SkScalerContext.h:368-370
    pub fn getEffects(&self) -> SkScalerContextEffects<'_> {
        SkScalerContextEffects::new(self.fPathEffect.get(), self.fMaskFilter.as_deref())
    }
}

// Source: src/core/SkScalerContext.h:411-411
// Source: src/core/SkScalerContext.h:431-431
// Source: src/core/SkScalerContext.h:445-445
// Source: src/core/SkScalerContext.h:455-455
// Real required backend operations replace C++ virtual dispatch. No synthetic
// generator or metrics is supplied by the base class.
pub trait SkScalerContextVirtual {
    fn base(&self) -> &SkScalerContext;
    fn base_mut(&mut self) -> &mut SkScalerContext;
    unsafe fn makeGlyph(
        &mut self,
        packed_id: crate::src::core::SkGlyph::SkPackedGlyphID,
        alloc: &mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    ) -> crate::src::core::SkGlyph::SkGlyph {
        unsafe { make_glyph(self, packed_id, alloc) }
    }

    unsafe fn generateMetrics(
        &mut self,
        glyph: &crate::src::core::SkGlyph::SkGlyph,
        alloc: &mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    ) -> GlyphMetrics;
    // Source: src/core/SkScalerContext.h:431-431
    /// Safety: glyph has its real prepared path; buffer is exclusively writable
    /// for the glyph image size and retained through the raster operation.
    unsafe fn generateImageFromPath(
        &self,
        glyph: &crate::src::core::SkGlyph::SkGlyph,
        buffer: *mut std::ffi::c_void,
    ) -> Result<(), crate::RasterError> {
        unsafe { self.base().generateImageFromPath(glyph, buffer) }
    }
    unsafe fn generateImage(
        &mut self,
        glyph: &crate::src::core::SkGlyph::SkGlyph,
        buffer: *mut std::ffi::c_void,
    );
    unsafe fn generatePath(
        &mut self,
        glyph: &crate::src::core::SkGlyph::SkGlyph,
    ) -> Option<GeneratedPath>;
    unsafe fn generateFontMetrics(
        &mut self,
        metrics: &mut crate::include::core::SkFontMetrics::SkFontMetrics,
    );
    unsafe fn getImage(
        &mut self,
        glyph: &crate::src::core::SkGlyph::SkGlyph,
    ) -> Result<(), crate::RasterError> {
        unsafe { get_image(self, glyph) }
    }
    unsafe fn getPath(
        &mut self,
        glyph: &mut crate::src::core::SkGlyph::SkGlyph,
        alloc: &mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    ) {
        unsafe { get_path(self, glyph, alloc) }
    }
    unsafe fn getDrawable(
        &mut self,
        glyph: &crate::src::core::SkGlyph::SkGlyph,
    ) -> crate::include::core::SkRefCnt::sk_sp<dyn crate::src::core::SkDrawable::SkDrawable> {
        unsafe { get_drawable(self, glyph) }
    }
    unsafe fn generateDrawable(
        &mut self,
        glyph: &crate::src::core::SkGlyph::SkGlyph,
    ) -> crate::include::core::SkRefCnt::sk_sp<dyn crate::src::core::SkDrawable::SkDrawable> {
        generate_drawable(glyph)
    }
    /// Safety: backend/scaler/font resources retain their required synchronous lease.
    unsafe fn getFontMetrics(
        &mut self,
        metrics: &mut crate::include::core::SkFontMetrics::SkFontMetrics,
    ) {
        unsafe {
            get_font_metrics(self, metrics);
        }
    }
}
// Source: src/core/SkScalerContext.cpp:117-120
fn mask_gamma_cache_mutex() -> &'static std::sync::Mutex<MaskGammaCache> {
    static CACHE: std::sync::OnceLock<std::sync::Mutex<MaskGammaCache>> =
        std::sync::OnceLock::new();
    CACHE.get_or_init(|| {
        std::sync::Mutex::new(MaskGammaCache {
            default: None,
            current: None,
            contrast: 0,
            gamma: 0,
        })
    })
}

// Source: src/core/SkScalerContext.cpp:117-120
// Rust expresses the native caller-held mutex obligation as an opaque guard.
// Its sole constructor locks the original cache; it cannot designate another cache.
pub struct SkMaskGammaCacheGuard<'lock> {
    cache: std::sync::MutexGuard<'lock, MaskGammaCache>,
}
pub fn lock_mask_gamma_cache() -> SkMaskGammaCacheGuard<'static> {
    SkMaskGammaCacheGuard {
        cache: mask_gamma_cache_mutex()
            .lock()
            .unwrap_or_else(|e| e.into_inner()),
    }
}

// Source: src/core/SkScalerContext.cpp:122-125
fn linear_gamma() -> &'static std::sync::Arc<SkMaskGamma> {
    static LINEAR: std::sync::OnceLock<std::sync::Arc<SkMaskGamma>> = std::sync::OnceLock::new();
    LINEAR.get_or_init(|| std::sync::Arc::new(SkMaskGamma::linear()))
}

// Source: src/core/SkScalerContext.cpp:127-130
// Shared mutable globals become the state protected by the original cache mutex.
struct MaskGammaCache {
    default: Option<std::sync::Arc<SkMaskGamma>>,
    current: Option<std::sync::Arc<SkMaskGamma>>,
    contrast: u8,
    gamma: u8,
}

impl MaskGammaCache {
    // Source: src/core/SkScalerContext.cpp:136-137
    // Source: src/core/SkScalerContext.cpp:139-161
    fn cached_mask_gamma(&mut self, contrast: u8, gamma: u8) -> &std::sync::Arc<SkMaskGamma> {
        use crate::include::core::SkTypes::{SK_GAMMA_CONTRAST, SK_GAMMA_EXPONENT};
        use SkScalerContextRec as Rec;
        let contrast0 = Rec::internal_contrast_from_external(0.);
        let gamma1 = Rec::internal_gamma_from_external(1.);
        if contrast0 == contrast && gamma1 == gamma {
            return linear_gamma();
        }
        let default_contrast = Rec::internal_contrast_from_external(SK_GAMMA_CONTRAST);
        let default_gamma = Rec::internal_gamma_from_external(SK_GAMMA_EXPONENT);
        if default_contrast == contrast && default_gamma == gamma {
            if self.default.is_none() {
                self.default = Some(std::sync::Arc::new(SkMaskGamma::new(
                    Rec::external_contrast_from_internal(contrast),
                    Rec::external_gamma_from_internal(gamma),
                )));
            }
            return self.default.as_ref().unwrap();
        }
        if self.current.is_none() || self.contrast != contrast || self.gamma != gamma {
            // Drop the cache's old reference before allocating its replacement, as SkSafeUnref does.
            self.current = None;
            self.current = Some(std::sync::Arc::new(SkMaskGamma::new(
                Rec::external_contrast_from_internal(contrast),
                Rec::external_gamma_from_internal(gamma),
            )));
            self.contrast = contrast;
            self.gamma = gamma;
        }
        self.current.as_ref().unwrap()
    }
}

impl SkScalerContextRec {
    // Source: src/core/SkScalerContext.h:117-117
    // The returned Gamma is the cache's existing object, without a retain/copy.
    // A mutable borrow of the same guard excludes replacement while it is used.
    pub fn CachedMaskGamma<'guard>(
        guard: &'guard mut SkMaskGammaCacheGuard<'_>,
        contrast: u8,
        gamma: u8,
    ) -> std::sync::Arc<SkMaskGamma> {
        guard.cache.cached_mask_gamma(contrast, gamma).clone()
    }
    // Source: src/core/SkScalerContext.h:118-120
    pub fn cachedMaskGamma<'guard>(
        &self,
        guard: &'guard mut SkMaskGammaCacheGuard<'_>,
    ) -> std::sync::Arc<SkMaskGamma> {
        Self::CachedMaskGamma(guard, self.fContrast, self.fDeviceGamma)
    }
}

// Source: src/core/SkScalerContext.cpp:166-167
// Source: src/core/SkScalerContext.cpp:169-169
// Source: src/core/SkScalerContext.cpp:172-173
pub fn get_mask_pre_blend(
    rec: &SkScalerContextRec,
) -> crate::src::core::SkMaskGamma::SkTMaskPreBlend<3, 3, 3> {
    let mut cache = lock_mask_gamma_cache();
    let mask_gamma = rec.cachedMaskGamma(&mut cache);
    mask_gamma.pre_blend(rec.fLumBits)
}

// Source: src/core/SkScalerContext.cpp:175-183
pub fn get_gamma_lut_size(contrast: f32, device_gamma: f32) -> (usize, usize, usize) {
    let mut cache = lock_mask_gamma_cache();
    let mask_gamma = SkScalerContextRec::CachedMaskGamma(
        &mut cache,
        SkScalerContextRec::internal_contrast_from_external(contrast),
        SkScalerContextRec::internal_gamma_from_external(device_gamma),
    );
    let (width, height) = mask_gamma.gamma_table_dimensions();
    (mask_gamma.gamma_table_size_in_bytes(), width, height)
}

// Source: src/core/SkScalerContext.cpp:185-193
// Source: src/core/SkScalerContext.cpp:195-197
pub fn get_gamma_lut_data(contrast: f32, device_gamma: f32, data: &mut [u8]) -> bool {
    let mut cache = lock_mask_gamma_cache();
    let mask_gamma = SkScalerContextRec::CachedMaskGamma(
        &mut cache,
        SkScalerContextRec::internal_contrast_from_external(contrast),
        SkScalerContextRec::internal_gamma_from_external(device_gamma),
    );
    let Some(tables) = mask_gamma.gamma_tables() else {
        return false;
    };
    data[..mask_gamma.gamma_table_size_in_bytes()].copy_from_slice(tables);
    true
}

// Source: src/core/SkScalerContext.cpp:199-201
/// Safety: retain actual backend/typeface leases and the glyph arena until all glyph uses end.
pub unsafe fn make_glyph<T: SkScalerContextVirtual + ?Sized>(
    context: &mut T,
    packed_id: crate::src::core::SkGlyph::SkPackedGlyphID,
    alloc: &mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
) -> crate::src::core::SkGlyph::SkGlyph {
    let format = context.base().fRec.fMaskFormat;
    unsafe { internal_make_glyph(context, packed_id, format, alloc) }
}

// Source: src/core/SkScalerContext.cpp:204-208
macro_rules! sk_saturate_cast {
    ($value:expr, $destination:ty, $source:ty) => {{
        let mut s: $source = $value;
        s = if s < <$destination>::MAX as $source {
            s
        } else {
            <$destination>::MAX as $source
        };
        s = if s > <$destination>::MIN as $source {
            s
        } else {
            <$destination>::MIN as $source
        };
        s as $destination
    }};
}

// Source: src/core/SkScalerContext.cpp:210-216
pub fn SaturateGlyphBounds(
    glyph: &mut crate::src::core::SkGlyph::SkGlyph,
    mut r: crate::src::core::SkRect::SkRect,
) {
    let unrounded = r;
    unrounded.roundOutRectTo(&mut r);
    glyph.fLeft = sk_saturate_cast!(r.fLeft, i16, f32);
    glyph.fTop = sk_saturate_cast!(r.fTop, i16, f32);
    glyph.fWidth = sk_saturate_cast!(r.width(), u16, f32);
    glyph.fHeight = sk_saturate_cast!(r.height(), u16, f32);
}

// Source: src/core/SkScalerContext.cpp:217-222
pub fn SaturateGlyphBoundsIRect(
    glyph: &mut crate::src::core::SkGlyph::SkGlyph,
    r: &crate::src::core::SkRect::SkIRect,
) {
    glyph.fLeft = sk_saturate_cast!(r.fLeft, i16, i32);
    glyph.fTop = sk_saturate_cast!(r.fTop, i16, i32);
    glyph.fWidth = sk_saturate_cast!(r.width64(), u16, i64);
    glyph.fHeight = sk_saturate_cast!(r.height64(), u16, i64);
}

// Source: src/core/SkScalerContext.cpp:224-253
pub fn GenerateMetricsFromPath(
    glyph: &mut crate::src::core::SkGlyph::SkGlyph,
    dev_path: &crate::src::core::SkPath::SkPath,
    _format: Format,
    vertical_lcd: bool,
    a8_from_lcd: bool,
    hairline: bool,
) {
    if glyph.fMaskFormat != Format::kBW_Format
        && glyph.fMaskFormat != Format::kA8_Format
        && glyph.fMaskFormat != Format::kLCD16_Format
    {
        glyph.fMaskFormat = Format::kA8_Format;
    }
    let mut bounds = *dev_path.getBounds();
    if !bounds.isEmpty() {
        let from_lcd = glyph.fMaskFormat == Format::kLCD16_Format
            || (glyph.fMaskFormat == Format::kA8_Format && a8_from_lcd);
        let need_extra_width = (from_lcd && !vertical_lcd) || hairline;
        let need_extra_height = (from_lcd && vertical_lcd) || hairline;
        if need_extra_width {
            let unrounded = bounds;
            unrounded.roundOutRectTo(&mut bounds);
            bounds.outset(1.0, 0.0);
        }
        if need_extra_height {
            let unrounded = bounds;
            unrounded.roundOutRectTo(&mut bounds);
            bounds.outset(0.0, 1.0);
        }
    }
    SaturateGlyphBounds(glyph, bounds);
}

// Source: src/core/SkScalerContext.cpp:255-313
/// Safety: the real backend and arena obey their native synchronous allocation leases.
pub unsafe fn internal_make_glyph<T: SkScalerContextVirtual + ?Sized>(
    context: &mut T,
    packed_id: crate::src::core::SkGlyph::SkPackedGlyphID,
    format: Format,
    alloc: &mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
) -> crate::src::core::SkGlyph::SkGlyph {
    use crate::src::core::SkGlyph::SkGlyph;
    let zero_bounds = |glyph: &mut SkGlyph| {
        glyph.fLeft = 0;
        glyph.fTop = 0;
        glyph.fWidth = 0;
        glyph.fHeight = 0;
    };
    let mut glyph = SkGlyph::new(packed_id);
    glyph.fMaskFormat = format;
    let mut mx = unsafe { context.generateMetrics(&glyph, alloc) };
    crate::SkASSERT!(@source b"src/core/SkScalerContext.cpp\0".as_ptr(),266; @condition b"!mx.neverRequestPath || !mx.computeFromPath\0".as_ptr(); !mx.neverRequestPath || !mx.computeFromPath);
    glyph.fAdvanceX = mx.advance.fX;
    glyph.fAdvanceY = mx.advance.fY;
    glyph.fMaskFormat = mx.maskFormat;
    glyph.fScalerContextBits = mx.extraBits;
    if mx.computeFromPath || (context.base().fGenerateImageFromPath && !mx.neverRequestPath) {
        #[cfg(sk_debug)]
        {
            glyph.fAdvancesBoundsFormatAndInitialPathDone = true;
        }
        unsafe {
            internal_get_path(context, &mut glyph, alloc, mx.generatedPath.take());
        }
        // Path data is a distinct stable arena allocation, not a field in glyph.
        // Retain the exact native borrow without adding a temporary owning path clone.
        if let Some(path) = unsafe {
            glyph
                .path()
                .map(|path| path as *const crate::src::core::SkPath::SkPath)
        } {
            let flags = context.base().fRec.fFlags;
            let hairline = unsafe { glyph.pathIsHairline() };
            GenerateMetricsFromPath(
                &mut glyph,
                unsafe { &*path },
                format,
                flags & self::flags::LCD_VERTICAL != 0,
                flags & self::flags::GEN_A8_FROM_LCD != 0,
                hairline,
            );
        }
    } else {
        SaturateGlyphBounds(&mut glyph, mx.bounds);
        if mx.neverRequestPath {
            unsafe {
                glyph.setPath(alloc, None, false, false);
            }
        }
    }
    #[cfg(sk_debug)]
    {
        glyph.fAdvancesBoundsFormatAndInitialPathDone = true;
    }
    if glyph.fWidth == 0 || glyph.fHeight == 0 {
        zero_bounds(&mut glyph);
        return glyph;
    }
    if let Some(filter) = context.base().fMaskFilter.as_deref() {
        use crate::src::core::SkMask::{SkMask, SkMaskBuilder};
        let src = SkMask::new(
            std::ptr::null(),
            glyph.iRect(),
            glyph.rowBytes() as u32,
            glyph.maskFormat(),
        );
        let mut dst = SkMaskBuilder::new();
        if unsafe {
            filter.filterMask(
                &mut dst,
                &src,
                &context.base().fRec.get_matrix_from_2x2(),
                None,
            )
        }
        .unwrap_or(false)
        {
            if dst.fBounds.is_empty() {
                zero_bounds(&mut glyph);
                return glyph;
            }
            crate::SkASSERT!(@source b"src/core/SkScalerContext.cpp\0".as_ptr(),307; @condition b"dst.fImage == nullptr\0".as_ptr(); dst.fImage.is_null());
            SaturateGlyphBoundsIRect(&mut glyph, &dst.fBounds);
            glyph.fMaskFormat = dst.fFormat;
        }
    }
    glyph
}

// Source: src/core/SkScalerContext.cpp:315-317
// Source: src/core/SkScalerContext.cpp:319-325
/// Safety: image rows must be writable for bounds.width() bytes and lut readable for256 bytes.
pub(crate) unsafe fn apply_lut_to_a8_mask(
    mask: &mut crate::src::core::SkMask::SkMaskBuilder,
    lut: *const u8,
) {
    let mut dst = *mask.image();
    let row_bytes = mask.fRowBytes as usize;
    let mut y = mask.fBounds.height() - 1;
    while y >= 0 {
        let mut x = mask.fBounds.width() - 1;
        while x >= 0 {
            let p = dst.add(x as usize);
            *p = *lut.add(*p as usize);
            x -= 1
        }
        dst = dst.add(row_bytes);
        y -= 1
    }
}

// Source: src/core/SkScalerContext.cpp:327-332
// Source: src/core/SkScalerContext.cpp:334-335
// Source: src/core/SkScalerContext.cpp:338-344
// Source: src/core/SkScalerContext.cpp:346-347
// Source: src/core/SkScalerContext.cpp:349-350
// Source: src/core/SkScalerContext.cpp:372-372
// Source: src/core/SkScalerContext.cpp:374-374
// Source: src/core/SkScalerContext.cpp:376-376
// Source: src/core/SkScalerContext.cpp:378-379
// Source: src/core/SkScalerContext.cpp:381-391
// Source: src/core/SkScalerContext.cpp:393-393
// Source: src/core/SkScalerContext.cpp:397-411
// Source: src/core/SkScalerContext.cpp:413-445
/// Safety: source and destination rows must be allocated for the dimensions/strides,
/// and LCD16 destination pixels must have u16 alignment.
pub(crate) unsafe fn pack_4xh_to_mask(
    src: &crate::src::core::SkPixmap::SkPixmap,
    dst: &mut crate::src::core::SkMask::SkMaskBuilder,
    pre_blend: &crate::src::core::SkMaskGamma::SkTMaskPreBlend<3, 3, 3>,
    do_bgr: bool,
    do_vert: bool,
) {
    const SAMPLES_PER_PIXEL: usize = 4;
    const LCD_PER_PIXEL: usize = 3;
    debug_assert_eq!(
        src.colorType(),
        crate::include::core::SkColorType::SkColorType::kAlpha_8_SkColorType
    );
    let to_a8 = dst.fFormat == Format::kA8_Format;
    debug_assert!(dst.fFormat == Format::kLCD16_Format || to_a8);
    if do_vert {
        crate::SkASSERT!(@source b"src/core/SkScalerContext.cpp\0".as_ptr(),339; @condition b"src.width() == (dst.fBounds.height() - 2) * 4\0".as_ptr(); (src.width())==((dst.fBounds.height() - 2) * 4));
        crate::SkASSERT!(@source b"src/core/SkScalerContext.cpp\0".as_ptr(),340; @condition b"src.height() == dst.fBounds.width()\0".as_ptr(); (src.height())==(dst.fBounds.width()))
    } else {
        crate::SkASSERT!(@source b"src/core/SkScalerContext.cpp\0".as_ptr(),342; @condition b"src.width() == (dst.fBounds.width() - 2) * 4\0".as_ptr(); (src.width())==((dst.fBounds.width() - 2) * 4));
        crate::SkASSERT!(@source b"src/core/SkScalerContext.cpp\0".as_ptr(),343; @condition b"src.height() == dst.fBounds.height()\0".as_ptr(); (src.height())==(dst.fBounds.height()))
    }
    let sample_width = src.width();
    let height = src.height();
    let dst_image = *dst.image();
    let dst_rb = dst.fRowBytes as usize;
    const COEFFICIENTS: [[u32; SAMPLES_PER_PIXEL * 3]; LCD_PER_PIXEL] = [
        [
            0x03, 0x0b, 0x1c, 0x33, 0x40, 0x39, 0x24, 0x10, 0x05, 0x01, 0, 0,
        ],
        [
            0, 0x02, 0x08, 0x16, 0x2b, 0x3d, 0x3d, 0x2b, 0x16, 0x08, 0x02, 0,
        ],
        [
            0, 0, 0x01, 0x05, 0x10, 0x24, 0x39, 0x40, 0x33, 0x1c, 0x0b, 0x03,
        ],
    ];
    let dst_pb = if to_a8 { 1 } else { 2 };
    for y in 0..height {
        let (mut dst_p, dst_delta) = if do_vert {
            (dst_image.add(y as usize * dst_pb), dst_rb)
        } else {
            (dst_image.add(y as usize * dst_rb), dst_pb)
        };
        let src_p = src.addr().add(y as usize * src.rowBytes());
        let mut sample_x = -4;
        while sample_x < sample_width + 4 {
            let mut fir = [0i32; LCD_PER_PIXEL];
            let mut sample_index = 0.max(sample_x - 4);
            let mut coeff_index = sample_index - (sample_x - 4);
            while sample_index < (sample_x + 8).min(sample_width) {
                let value = *src_p.add(sample_index as usize) as u32;
                for subpixel in 0..LCD_PER_PIXEL {
                    fir[subpixel] += (COEFFICIENTS[subpixel][coeff_index as usize] * value) as i32
                }
                sample_index += 1;
                coeff_index += 1;
            }
            for subpixel in 0..LCD_PER_PIXEL {
                fir[subpixel] /= 0x100;
                fir[subpixel] = fir[subpixel].min(255)
            }
            let (mut r, mut g, mut b) = if do_bgr {
                (fir[2] as u32, fir[1] as u32, fir[0] as u32)
            } else {
                (fir[0] as u32, fir[1] as u32, fir[2] as u32)
            };
            if kSkShowTextBlitCoverage {
                r = r.max(10);
                g = g.max(10);
                b = b.max(10)
            }
            if to_a8 {
                let mut a = (r + g + b) / 3;
                if pre_blend.is_applicable() {
                    a = pre_blend.channel(1).unwrap()[a as usize] as u32
                }
                *dst_p = a as u8
            } else {
                if pre_blend.is_applicable() {
                    r = pre_blend.channel(0).unwrap()[r as usize] as u32;
                    g = pre_blend.channel(1).unwrap()[g as usize] as u32;
                    b = pre_blend.channel(2).unwrap()[b as usize] as u32
                }
                *dst_p.cast::<u16>() =
                    crate::src::core::SkColorData::SkPack888ToRGB16(r, g, b) as u16;
            }
            dst_p = dst_p.add(dst_delta);
            sample_x += 4;
        }
    }
}

// Source: src/core/SkScalerContext.cpp:447-450
fn convert_8_to_1(byte: u32) -> u32 {
    crate::SkASSERT!(@source b"src/core/SkScalerContext.cpp\0".as_ptr(),448; @condition b"byte <= 0xFF\0".as_ptr(); byte <= 0xFF);
    byte >> 7
}

// Source: src/core/SkScalerContext.cpp:452-459
unsafe fn pack_8_to_1(alpha: *const u8) -> u8 {
    let mut bits = 0u32;
    for i in 0..8 {
        bits <<= 1;
        bits |= convert_8_to_1(*alpha.add(i) as u32)
    }
    bits as u8
}

// Source: src/core/SkScalerContext.cpp:461-465
// Source: src/core/SkScalerContext.cpp:467-469
// Source: src/core/SkScalerContext.cpp:471-473
// Source: src/core/SkScalerContext.cpp:475-491
/// Safety: readable source rows and writable destination rows must satisfy native bounds/stride.
pub(crate) unsafe fn pack_a8_to_a1(
    dst_mask: &mut crate::src::core::SkMask::SkMaskBuilder,
    mut src: *const u8,
    src_rb: usize,
) {
    let height = dst_mask.fBounds.height();
    let width = dst_mask.fBounds.width();
    let octs = width >> 3;
    let leftover = width & 7;
    let mut dst = *dst_mask.image();
    let dst_pad = (dst_mask
        .fRowBytes
        .wrapping_sub(((width.wrapping_add(7) & !7) / 8) as u32)) as i32;
    crate::SkASSERT!(@source b"src/core/SkScalerContext.cpp\0".as_ptr(),469; @condition b"dstPad >= 0\0".as_ptr(); dst_pad >= 0);
    crate::SkASSERT!(@source b"src/core/SkScalerContext.cpp\0".as_ptr(),471; @condition b"width >= 0\0".as_ptr(); width >= 0);
    debug_assert!(src_rb >= width as usize);
    let src_pad = src_rb - width as usize;
    for _ in 0..height {
        for _ in 0..octs {
            *dst = pack_8_to_1(src);
            dst = dst.add(1);
            src = src.add(8)
        }
        if leftover > 0 {
            let mut bits = 0u32;
            let mut shift = 7;
            for _ in 0..leftover {
                bits |= convert_8_to_1(*src as u32) << shift;
                src = src.add(1);
                shift -= 1
            }
            *dst = bits as u8;
            dst = dst.add(1)
        }
        src = src.add(src_pad);
        dst = dst.offset(dst_pad as isize);
    }
}

// Source: src/core/SkScalerContext.cpp:507-607
/// Safety: dst_mask describes live exclusive writable rows of its declared format;
/// no drawing callback unwinds, and its image storage outlives the complete draw/pack call.
pub unsafe fn GenerateImageFromPath(
    dst_mask: &mut crate::src::core::SkMask::SkMaskBuilder,
    path: &crate::src::core::SkPath::SkPath,
    pre_blend: &crate::src::core::SkMaskGamma::SkTMaskPreBlend<3, 3, 3>,
    do_bgr: bool,
    vertical_lcd: bool,
    a8_from_lcd: bool,
    hairline: bool,
) -> Result<(), crate::RasterError> {
    use crate::src::core::{
        SkAutoPixmapStorage::SkAutoPixmapStorage,
        SkDraw::Draw,
        SkImageInfo::SkImageInfo,
        SkMatrix::SkMatrix,
        SkPaint::{Cap, Join, SkPaint, Style},
        SkPath::SkPath,
        SkPathBuilder::SkPathBuilder,
        SkRasterClip::SkRasterClip,
        SkRect::SkIRect,
        SkStrokeRec::{InitStyle, SkStrokeRec},
    };
    debug_assert!(matches!(
        dst_mask.fFormat,
        Format::kBW_Format | Format::kA8_Format | Format::kLCD16_Format
    ));
    let mut paint = SkPaint::default();
    let mut stroke_path: SkPath;
    let mut path_to_use = path;
    let src_w = dst_mask.fBounds.width();
    let src_h = dst_mask.fBounds.height();
    let mut dst_w = src_w;
    let mut dst_h = src_h;
    let mut matrix = SkMatrix::default();
    matrix.setTranslate(
        -(dst_mask.fBounds.fLeft as f32),
        -(dst_mask.fBounds.fTop as f32),
    );
    paint.setStroke(hairline);
    paint.setAntiAlias(dst_mask.fFormat != Format::kBW_Format);
    let from_lcd = dst_mask.fFormat == Format::kLCD16_Format
        || (dst_mask.fFormat == Format::kA8_Format && a8_from_lcd);
    let intermediate_dst = from_lcd || dst_mask.fFormat == Format::kBW_Format;
    if from_lcd {
        if vertical_lcd {
            dst_w = 4 * dst_h - 8;
            dst_h = src_w;
            matrix.setAll(
                0.0,
                4.0,
                -((dst_mask.fBounds.fTop + 1) as f32) * 4.0,
                1.0,
                0.0,
                -(dst_mask.fBounds.fLeft as f32),
                0.0,
                0.0,
                1.0,
            );
        } else {
            dst_w = 4 * dst_w - 8;
            matrix.setAll(
                4.0,
                0.0,
                -((dst_mask.fBounds.fLeft + 1) as f32) * 4.0,
                0.0,
                1.0,
                -(dst_mask.fBounds.fTop as f32),
                0.0,
                0.0,
                1.0,
            );
        }
        let mut rec = SkStrokeRec::new(InitStyle::kFill_InitStyle);
        if hairline {
            rec.setStrokeStyle(1.0, false);
            rec.setStrokeParams(Cap::kButt_Cap, Join::kRound_Join, 0.0);
        }
        let mut builder = SkPathBuilder::new();
        if rec.needToApply() && rec.applyToPath(&mut builder, path) {
            stroke_path = builder.detach(None);
            path_to_use = &stroke_path;
            paint.setStyle(Style::kFill_Style);
        }
    }
    let mut clip = SkRasterClip::new();
    clip.set_rect(SkIRect::make_wh(dst_w, dst_h));
    let info = SkImageInfo::MakeA8(dst_w, dst_h);
    let mut dst = SkAutoPixmapStorage::new();
    if intermediate_dst {
        if !dst.tryAlloc(&info) {
            let size = dst_mask.computeImageSize();
            if size != 0 {
                core::ptr::write_bytes(*dst_mask.image(), 0, size);
            }
            return Ok(());
        }
        let size = dst.computeByteSize();
        if size != 0 {
            core::ptr::write_bytes(dst.pixmap().writable_addr(), 0, size);
        }
    } else {
        dst.reset_with(&info, *dst_mask.image(), dst_mask.fRowBytes as usize);
    }
    let size = dst_mask.computeImageSize();
    if size != 0 {
        core::ptr::write_bytes(*dst_mask.image(), 0, size);
    }

    // Rust-only error-protocol forwarding: the native A8 chooser's actual stage error
    // becomes RasterError; no bool/None swallowing, extra owner or alternate chooser.
    unsafe fn choose_a8<'arena>(
        dst: &crate::src::core::SkPixmap::SkPixmap,
        ctm: &SkMatrix,
        paint: &SkPaint,
        alloc: core::pin::Pin<&'arena mut crate::src::core::SkArenaAlloc::SkArenaAlloc>,
        coverage: crate::src::core::SkDrawTypes::SkDrawCoverage,
        clip_shader: Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>,
        props: &crate::include::core::SkSurfaceProps::SkSurfaceProps,
        bounds: &crate::src::core::SkRect::SkRect,
    ) -> Result<
        Option<core::ptr::NonNull<dyn crate::src::core::SkBlitter::SkBlitter + 'arena>>,
        crate::RasterError,
    > {
        Ok(crate::src::core::SkBlitter_A8::SkA8Blitter_Choose(
            dst,
            ctm,
            paint,
            alloc,
            coverage,
            clip_shader,
            props,
            bounds,
        )?)
    }
    let mut draw = Draw::new();
    draw.fBlitterChooser = choose_a8;
    draw.fDst = dst.pixmap().clone();
    draw.fRC = Some(&clip);
    draw.fCTM = Some(&matrix);
    draw.drawPath(path_to_use, &paint, None)?;
    match dst_mask.fFormat {
        Format::kBW_Format => pack_a8_to_a1(dst_mask, dst.pixmap().addr8_xy(0, 0), dst.rowBytes()),
        Format::kA8_Format => {
            if from_lcd {
                pack_4xh_to_mask(dst.pixmap(), dst_mask, pre_blend, do_bgr, vertical_lcd);
            } else if pre_blend.is_applicable() {
                apply_lut_to_a8_mask(dst_mask, pre_blend.channel(1).unwrap().as_ptr());
            }
        }
        Format::kLCD16_Format => {
            pack_4xh_to_mask(dst.pixmap(), dst_mask, pre_blend, do_bgr, vertical_lcd)
        }
        _ => crate::SkUNREACHABLE!(@source b"src/core/SkScalerContext.cpp\0".as_ptr(),604;),
    }
    Ok(())
}

impl SkScalerContext {
    // Source: src/core/SkScalerContext.cpp:493-506
    /// Safety: glyph path and image backing retain their native operation lease;
    /// image_buffer covers glyph.imageSize() writable bytes without aliases.
    pub unsafe fn generateImageFromPath(
        &self,
        glyph: &crate::src::core::SkGlyph::SkGlyph,
        image_buffer: *mut std::ffi::c_void,
    ) -> Result<(), crate::RasterError> {
        crate::SkASSERT!(@source b"src/core/SkScalerContext.cpp\0".as_ptr(),494; @condition b"glyph.setPathHasBeenCalled()\0".as_ptr(); glyph.setPathHasBeenCalled());
        let path = unsafe { glyph.path() }.expect("native SkASSERT_RELEASE(devPath)");
        let mut mask = crate::src::core::SkMask::SkMaskBuilder::from_image(
            image_buffer.cast(),
            glyph.iRect(),
            glyph.rowBytes() as u32,
            glyph.maskFormat(),
        );
        debug_assert!(mask.fFormat != Format::kARGB32_Format);
        let do_bgr = self.fRec.fFlags & flags::LCD_BGR_ORDER != 0;
        let do_vert = self.fRec.fFlags & flags::LCD_VERTICAL != 0;
        let a8_lcd = self.fRec.fFlags & flags::GEN_A8_FROM_LCD != 0;
        let hairline = unsafe { glyph.pathIsHairline() };
        unsafe {
            GenerateImageFromPath(
                &mut mask,
                path,
                &self.fPreBlend,
                do_bgr,
                do_vert,
                a8_lcd,
                hairline,
            )
        }
    }
}

// Source: src/core/SkScalerContext.cpp:609-756
/// Safety: orig_glyph's allocated pixels are exclusively writable for its image
/// size; real backend/effect callbacks do not unwind and retain all backing leases.
/// On Err the caller must not consume incomplete pixel data.
pub unsafe fn get_image<T: SkScalerContextVirtual + ?Sized>(
    context: &mut T,
    orig_glyph: &crate::src::core::SkGlyph::SkGlyph,
) -> Result<(), crate::RasterError> {
    use crate::src::core::{
        SkArenaAlloc::SkSTArenaAlloc,
        SkAutoMalloc::{OnShrink, SkAutoMalloc},
        SkGlyph::{PathData, SkGlyph},
        SkMask::{SkAutoMaskFreeImage, SkMaskBuilder},
    };
    #[cfg(sk_debug)]
    crate::SkASSERT!(@source b"src/core/SkScalerContext.cpp\0".as_ptr(),610; @condition b"origGlyph.fAdvancesBoundsFormatAndInitialPathDone\0".as_ptr(); orig_glyph.fAdvancesBoundsFormatAndInitialPathDone);
    let mut unfiltered_glyph = orig_glyph as *const SkGlyph;
    let mut tmp_image = SkAutoMalloc::new(0);
    let mut tmp_glyph = SkGlyph::default();
    let mut tmp_path_storage = std::pin::pin!(
        SkSTArenaAlloc::<{ std::mem::size_of::<PathData>() }>::new(0)
    );
    if context.base().fMaskFilter.is_some() {
        // Keep the original owner while the actual scaler computes unfiltered
        // bounds. Raw self projection permits the native callback, but retains no
        // exclusive base borrow across it. Restore on normal return and unwind.
        struct RestoreMaskFilter {
            base: *mut SkScalerContext,
            filter: Option<std::sync::Arc<dyn crate::src::core::SkMaskFilter::SkMaskFilter>>,
        }
        impl Drop for RestoreMaskFilter {
            fn drop(&mut self) {
                unsafe {
                    (*self.base).fMaskFilter = self.filter.take();
                }
            }
        }
        let base = context.base_mut();
        let restore = RestoreMaskFilter {
            base: base as *mut SkScalerContext,
            filter: base.fMaskFilter.take(),
        };
        tmp_glyph = unsafe {
            context.makeGlyph(orig_glyph.getPackedID(), tmp_path_storage.as_mut().arena())
        };
        drop(restore);
        if tmp_glyph.fMaskFormat == orig_glyph.fMaskFormat
            && tmp_glyph.imageSize() <= orig_glyph.imageSize()
        {
            tmp_glyph.fImage = orig_glyph.fImage;
        } else {
            tmp_image.reset(tmp_glyph.imageSize(), OnShrink::kAlloc_OnShrink);
            tmp_glyph.fImage = tmp_image.get();
        }
        unfiltered_glyph = &tmp_glyph;
    }
    let unfiltered = unsafe { &*unfiltered_glyph };
    if !context.base().fGenerateImageFromPath {
        unsafe {
            context.generateImage(unfiltered, unfiltered.fImage.cast());
        }
    } else {
        crate::SkASSERT!(@source b"src/core/SkScalerContext.cpp\0".as_ptr(),639; @condition b"origGlyph.setPathHasBeenCalled()\0".as_ptr(); orig_glyph.setPathHasBeenCalled());
        if let Some(path) = unsafe { orig_glyph.path() } {
            let mut mask = SkMaskBuilder::from_image(
                unfiltered.fImage,
                unfiltered.iRect(),
                unfiltered.rowBytes() as u32,
                unfiltered.maskFormat(),
            );
            debug_assert!(orig_glyph.fMaskFormat != Format::kARGB32_Format);
            debug_assert!(mask.fFormat != Format::kARGB32_Format);
            let rec = &context.base().fRec;
            let (bgr, vertical, a8) = (
                rec.fFlags & flags::LCD_BGR_ORDER != 0,
                rec.fFlags & flags::LCD_VERTICAL != 0,
                rec.fFlags & flags::GEN_A8_FROM_LCD != 0,
            );
            let hairline = unsafe { orig_glyph.pathIsHairline() };
            unsafe {
                GenerateImageFromPath(
                    &mut mask,
                    path,
                    &context.base().fPreBlend,
                    bgr,
                    vertical,
                    a8,
                    hairline,
                )?;
            }
        } else {
            unsafe {
                context.generateImage(unfiltered, unfiltered.fImage.cast());
            }
        }
    }
    if let Some(filter) = context.base().fMaskFilter.as_deref() {
        debug_assert!(unfiltered.fMaskFormat != Format::k3D_Format);
        let mut src = SkMaskBuilder::new();
        let filtered = unsafe {
            filter.filterMask(
                &mut src,
                &unfiltered.mask(),
                &context.base().fRec.get_matrix_from_2x2(),
                None,
            )
        }?;
        // This guard owns only a successful filter allocation, never an arena or
        // temporary allocation. It drops before the original temporary storage.
        let mut _src_owned = unsafe { SkAutoMaskFreeImage::from_raw(std::ptr::null_mut()) };
        if filtered {
            _src_owned = unsafe { SkAutoMaskFreeImage::from_raw(src.fImage.cast_mut()) };
        } else if unfiltered.fImage == tmp_image.get() {
            src = SkMaskBuilder::from_image(
                unfiltered.fImage,
                unfiltered.iRect(),
                unfiltered.rowBytes() as u32,
                unfiltered.maskFormat(),
            );
        } else if orig_glyph.iRect() == unfiltered.iRect() {
            return Ok(());
        } else {
            src = SkMaskBuilder::from_image(
                unfiltered.fImage,
                unfiltered.iRect(),
                unfiltered.rowBytes() as u32,
                unfiltered.maskFormat(),
            );
            let image_size = unfiltered.imageSize();
            tmp_image.reset(image_size, OnShrink::kAlloc_OnShrink);
            src.mask.fImage = tmp_image.get();
            unsafe {
                std::ptr::copy_nonoverlapping(unfiltered.fImage, tmp_image.get(), image_size);
            }
        }
        assert_eq!(src.fFormat, orig_glyph.fMaskFormat);
        let mut dst = SkMaskBuilder::from_image(
            orig_glyph.fImage,
            orig_glyph.iRect(),
            orig_glyph.rowBytes() as u32,
            orig_glyph.maskFormat(),
        );
        let orig_bounds = dst.fBounds;
        if src.fBounds.fTop < dst.fBounds.fTop {
            let diff = dst.fBounds.fTop.wrapping_sub(src.fBounds.fTop);
            src.mask.fImage = unsafe {
                src.fImage
                    .add(src.fRowBytes.wrapping_mul(diff as u32) as usize)
            };
            src.mask.fBounds.fTop = dst.fBounds.fTop;
        }
        if dst.fBounds.fTop < src.fBounds.fTop {
            let diff = src.fBounds.fTop.wrapping_sub(dst.fBounds.fTop);
            dst.mask.fImage = unsafe {
                dst.fImage
                    .add(dst.fRowBytes.wrapping_mul(diff as u32) as usize)
            };
            dst.mask.fBounds.fTop = src.fBounds.fTop;
        }
        if src.fBounds.fLeft < dst.fBounds.fLeft {
            let diff = dst.fBounds.fLeft.wrapping_sub(src.fBounds.fLeft);
            src.mask.fImage = unsafe { src.fImage.add(diff as usize) };
            src.mask.fBounds.fLeft = dst.fBounds.fLeft;
        }
        if dst.fBounds.fLeft < src.fBounds.fLeft {
            let diff = src.fBounds.fLeft.wrapping_sub(dst.fBounds.fLeft);
            dst.mask.fImage = unsafe { dst.fImage.add(diff as usize) };
            dst.mask.fBounds.fLeft = src.fBounds.fLeft;
        }
        if src.fBounds.fBottom < dst.fBounds.fBottom {
            dst.mask.fBounds.fBottom = src.fBounds.fBottom;
        }
        if dst.fBounds.fBottom < src.fBounds.fBottom {
            src.mask.fBounds.fBottom = dst.fBounds.fBottom;
        }
        if src.fBounds.fRight < dst.fBounds.fRight {
            dst.mask.fBounds.fRight = src.fBounds.fRight;
        }
        if dst.fBounds.fRight < src.fBounds.fRight {
            src.mask.fBounds.fRight = dst.fBounds.fRight;
        }
        debug_assert_eq!(src.fBounds, dst.fBounds);
        let width = src.fBounds.width();
        let mut height = src.fBounds.height();
        let (dst_row, src_row) = (dst.fRowBytes as i32, src.fRowBytes as i32);
        let mut src_ptr = src.fImage;
        let mut dst_ptr = dst.fImage.cast_mut();
        if src.fFormat == Format::k3D_Format {
            height = height.wrapping_mul(3);
        }
        if dst.fBounds != orig_bounds {
            unsafe {
                std::ptr::write_bytes(
                    orig_glyph.fImage,
                    0,
                    orig_glyph.fHeight as usize * orig_glyph.rowBytes(),
                );
            }
        }
        loop {
            height = height.wrapping_sub(1);
            if height < 0 {
                break;
            }
            unsafe {
                std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, width as usize);
                src_ptr = src_ptr.offset(src_row as isize);
                dst_ptr = dst_ptr.offset(dst_row as isize);
            }
        }
    }
    Ok(())
}

// Source: src/core/SkScalerContext.cpp:758-760
pub unsafe fn get_path<T: SkScalerContextVirtual + ?Sized>(
    context: &mut T,
    glyph: &mut crate::src::core::SkGlyph::SkGlyph,
    alloc: &mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
) {
    unsafe { internal_get_path(context, glyph, alloc, None) }
}
// Source: src/core/SkScalerContext.cpp:762-764
pub unsafe fn get_drawable<T: SkScalerContextVirtual + ?Sized>(
    context: &mut T,
    glyph: &crate::src::core::SkGlyph::SkGlyph,
) -> crate::include::core::SkRefCnt::sk_sp<dyn crate::src::core::SkDrawable::SkDrawable> {
    unsafe { context.generateDrawable(glyph) }
}
// Source: src/core/SkScalerContext.cpp:766-768
pub fn generate_drawable(
    _glyph: &crate::src::core::SkGlyph::SkGlyph,
) -> crate::include::core::SkRefCnt::sk_sp<dyn crate::src::core::SkDrawable::SkDrawable> {
    crate::include::core::SkRefCnt::sk_sp::default()
}

// Source: src/core/SkScalerContext.cpp:770-773
/// Safety: backend/scaler/font resources retain their required synchronous lease.
pub unsafe fn get_font_metrics<T: SkScalerContextVirtual + ?Sized>(
    context: &mut T,
    metrics: &mut crate::include::core::SkFontMetrics::SkFontMetrics,
) {
    unsafe {
        context.generateFontMetrics(metrics);
    }
}

// Source: src/core/SkScalerContext.cpp:777-857
/// Safety: the actual backend, path-effect owner and arena retain native operation leases.
pub unsafe fn internal_get_path<T: SkScalerContextVirtual + ?Sized>(
    context: &mut T,
    glyph: &mut crate::src::core::SkGlyph::SkGlyph,
    alloc: &mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    mut generated_path: Option<GeneratedPath>,
) {
    #[cfg(sk_debug)]
    crate::SkASSERT!(@source b"src/core/SkScalerContext.cpp\0".as_ptr(),779; @condition b"glyph.fAdvancesBoundsFormatAndInitialPathDone\0".as_ptr(); glyph.fAdvancesBoundsFormatAndInitialPathDone);
    if glyph.setPathHasBeenCalled() {
        return;
    }
    if generated_path.is_none() {
        generated_path = unsafe { context.generatePath(glyph) };
    }
    let Some(generated_path) = generated_path else {
        unsafe {
            glyph.setPath(alloc, None, false, false);
        }
        return;
    };
    let mut path = generated_path.path;
    let mut path_modified = generated_path.modified;
    let base = context.base();
    if base.fRec.fFlags & flags::SUBPIXEL_POSITIONING != 0 {
        let id = glyph.getPackedID();
        let dx = id.sub_x_fixed();
        let dy = id.sub_y_fixed();
        if dx | dy != 0 {
            path_modified = true;
            path = path.makeOffset(dx as f32 / 65536.0, dy as f32 / 65536.0);
        }
    }
    if base.fRec.fFrameWidth < 0.0 && base.fPathEffect.get().is_none() {
        unsafe {
            glyph.setPath(alloc, Some(&path), false, path_modified);
        }
        return;
    }
    path_modified = true;
    let matrix = base.fRec.get_matrix_from_2x2();
    let Some(inverse) = matrix.invert() else {
        let empty = crate::src::core::SkPath::SkPath::default();
        unsafe {
            glyph.setPath(alloc, Some(&empty), false, path_modified);
        }
        return;
    };
    let mut local_path = path.makeTransform(&inverse);
    use crate::src::core::{
        SkPathBuilder::SkPathBuilder,
        SkStrokeRec::{InitStyle, SkStrokeRec},
    };
    let mut rec = SkStrokeRec::new(InitStyle::kFill_InitStyle);
    if base.fRec.fFrameWidth >= 0.0 {
        rec.setStrokeStyle(
            base.fRec.fFrameWidth,
            base.fRec.fFlags & flags::FRAME_AND_FILL != 0,
        );
        use crate::src::core::SkPaint::{Cap, Join};
        let cap = match base.fRec.stroke_join_cap >> 4 {
            0 => Cap::kButt_Cap,
            1 => Cap::kRound_Cap,
            2 => Cap::kSquare_Cap,
            _ => unreachable!("native valid cap"),
        };
        let join = match base.fRec.stroke_join_cap & 15 {
            0 => Join::kMiter_Join,
            1 => Join::kRound_Join,
            2 => Join::kBevel_Join,
            _ => unreachable!("native valid join"),
        };
        rec.setStrokeParams(cap, join, base.fRec.fMiterLimit);
    }
    if let Some(effect) = base.fPathEffect.get() {
        let mut builder = SkPathBuilder::new();
        if effect.filterPath(&mut builder, &local_path, &mut rec, None, &matrix) {
            local_path = builder.detach(None);
        }
    }
    if rec.needToApply() {
        let mut builder = SkPathBuilder::new();
        if rec.applyToPath(&mut builder, &local_path) {
            local_path = builder.detach(None);
        }
    }
    let dev_path = local_path.makeTransform(&matrix);
    unsafe {
        glyph.setPath(alloc, Some(&dev_path), rec.isHairlineStyle(), path_modified);
    }
}

impl SkScalerContextRec {
    // Source: src/core/SkScalerContext.cpp:860-864
    pub fn get_matrix_from_2x2(&self) -> crate::src::core::SkMatrix::SkMatrix {
        crate::src::core::SkMatrix::SkMatrix::MakeAll(
            self.fPost2x2[0][0],
            self.fPost2x2[0][1],
            0.,
            self.fPost2x2[1][0],
            self.fPost2x2[1][1],
            0.,
            0.,
            0.,
            1.,
        )
    }
}

impl SkScalerContextRec {
    // Source: src/core/SkScalerContext.cpp:866-868
    // Source: src/core/SkScalerContext.cpp:870-872
    pub fn get_local_matrix(&self) -> crate::src::core::SkMatrix::SkMatrix {
        crate::src::core::SkFontPriv::SkFontPriv::MakeTextMatrix(
            self.fTextSize,
            self.fPreScaleX,
            self.fPreSkewX,
        )
    }
}

impl SkScalerContextRec {
    // Source: src/core/SkScalerContext.cpp:874-876
    // Source: src/core/SkScalerContext.cpp:878-878
    // Source: src/core/SkScalerContext.cpp:881-883
    // Source: src/core/SkScalerContext.cpp:886-888
    // Source: src/core/SkScalerContext.cpp:891-891
    // Source: src/core/SkScalerContext.cpp:894-895
    // Source: src/core/SkScalerContext.cpp:897-898
    // Source: src/core/SkScalerContext.cpp:901-912
    // Source: src/core/SkScalerContext.cpp:919-933
    // Source: src/core/SkScalerContext.cpp:936-957
    // Source: src/core/SkScalerContext.cpp:960-963
    // Source: src/core/SkScalerContext.cpp:966-967
    // Source: src/core/SkScalerContext.cpp:969-971
    // Source: src/core/SkScalerContext.cpp:973-975
    // Source: src/core/SkScalerContext.cpp:978-979
    // Source: src/core/SkScalerContext.cpp:981-982
    // Source: src/core/SkScalerContext.cpp:984-985
    pub fn compute_matrices(
        &self,
        pre_matrix_scale: PreMatrixScale,
        s: &mut crate::src::core::SkPoint::SkPoint,
        sa: &mut crate::src::core::SkMatrix::SkMatrix,
        mut gsa: Option<&mut crate::src::core::SkMatrix::SkMatrix>,
        mut g_inv: Option<&mut crate::src::core::SkMatrix::SkMatrix>,
        a_out: Option<&mut crate::src::core::SkMatrix::SkMatrix>,
    ) -> bool {
        use crate::src::core::SkMatrix::SkMatrix;
        use crate::src::core::SkPoint::SkPoint;
        let a = self.get_single_matrix();
        if let Some(out) = a_out {
            *out = a.clone();
        }
        let mut ga;
        let skewed_or_flipped =
            a.getSkewX() != 0. || a.getSkewY() != 0. || a.getScaleX() < 0. || a.getScaleY() < 0.;
        if skewed_or_flipped {
            let h = a.mapPoint(SkPoint::make(1., 0.));
            let mut g = SkMatrix::default();
            crate::src::utils::SkMatrix22::SkComputeGivensRotation(&h, &mut g);
            ga = g.clone();
            ga.preConcat(&a);
            if let Some(inv) = g_inv.as_deref_mut() {
                inv.setAll(
                    g.get(SkMatrix::kMScaleX),
                    -g.get(SkMatrix::kMSkewX),
                    g.get(SkMatrix::kMTransX),
                    -g.get(SkMatrix::kMSkewY),
                    g.get(SkMatrix::kMScaleY),
                    g.get(SkMatrix::kMTransY),
                    g.get(SkMatrix::kMPersp0),
                    g.get(SkMatrix::kMPersp1),
                    g.get(SkMatrix::kMPersp2),
                );
            }
        } else {
            ga = a.clone();
            if let Some(inv) = g_inv.as_deref_mut() {
                inv.reset();
            }
        }
        let nearly_zero = crate::include::core::SkScalar::SK_ScalarNearlyZero;
        if ga.get(SkMatrix::kMScaleX).abs() <= nearly_zero
            || ga.get(SkMatrix::kMScaleY).abs() <= nearly_zero
            || !ga.isFinite()
        {
            s.fX = 1.;
            s.fY = 1.;
            sa.setScale(0., 0.);
            if let Some(out) = gsa.as_deref_mut() {
                out.setScale(0., 0.);
            }
            if let Some(inv) = g_inv.as_deref_mut() {
                inv.reset();
            }
            return false;
        }
        match pre_matrix_scale {
            PreMatrixScale::Full => {
                s.fX = ga.get(SkMatrix::kMScaleX).abs();
                s.fY = ga.get(SkMatrix::kMScaleY).abs();
            }
            PreMatrixScale::Vertical => {
                let y_scale = ga.get(SkMatrix::kMScaleY).abs();
                s.fX = y_scale;
                s.fY = y_scale;
            }
            PreMatrixScale::VerticalInteger => {
                let real_y_scale = ga.get(SkMatrix::kMScaleY).abs();
                let mut int_y_scale =
                    crate::src::core::SkFloatingPoint::sk_float_round(real_y_scale);
                if int_y_scale == 0. {
                    int_y_scale = 1.;
                }
                s.fX = int_y_scale;
                s.fY = int_y_scale;
            }
        }
        if !skewed_or_flipped
            && (pre_matrix_scale == PreMatrixScale::Full
                || (pre_matrix_scale == PreMatrixScale::Vertical && a.getScaleX() == a.getScaleY()))
        {
            sa.reset();
        } else if !skewed_or_flipped && pre_matrix_scale == PreMatrixScale::Vertical {
            sa.reset();
            sa.setScaleX(a.getScaleX() / s.fY);
        } else {
            *sa = a.clone();
            sa.preScale(1. / s.fX, 1. / s.fY);
        }
        if let Some(out) = gsa.as_deref_mut() {
            *out = ga;
            out.preScale(1. / s.fX, 1. / s.fY);
        }
        true
    }
}

impl SkScalerContext {
    // Source: src/core/SkScalerContext.h:376-376
    // Source: src/core/SkScalerContext.cpp:987-989
    pub fn computeAxisAlignmentForHText(&self) -> crate::src::core::SkGlyph::SkAxisAlignment {
        self.fRec.compute_axis_alignment_for_h_text()
    }
}

impl SkScalerContextRec {
    // Source: src/core/SkScalerContext.cpp:991-991
    // Source: src/core/SkScalerContext.cpp:999-1001
    // Source: src/core/SkScalerContext.cpp:1003-1003
    // Source: src/core/SkScalerContext.cpp:1005-1007
    // Source: src/core/SkScalerContext.cpp:1009-1012
    pub fn compute_axis_alignment_for_h_text(&self) -> crate::src::core::SkGlyph::SkAxisAlignment {
        use crate::src::core::SkGlyph::SkAxisAlignment;
        if self.fFlags & flags::BASELINE_SNAP == 0 {
            return SkAxisAlignment::None;
        }
        if self.fPost2x2[1][0] == 0. {
            return SkAxisAlignment::X;
        }
        if self.fPost2x2[0][0] == 0. {
            return SkAxisAlignment::Y;
        }
        SkAxisAlignment::None
    }
}

impl SkScalerContextRec {
    // Source: src/core/SkScalerContext.cpp:1014-1017
    pub fn set_luminance_color(&mut self, color: u32) {
        self.fLumBits = SkMaskGamma::canonical_color(color | 0xff000000);
    }
}

impl SkScalerContextRec {
    // Source: src/core/SkScalerContext.h:141-141
    // Source: src/core/SkScalerContext.cpp:1019-1041
    pub fn useStrokeForFakeBold(&mut self) {
        use crate::src::core::SkTextFormatParams::*;
        if self.fFlags & flags::EMBOLDEN == 0 {
            return;
        }
        self.fFlags &= !flags::EMBOLDEN;
        let scale = crate::src::core::SkScalar::SkFloatInterpFunc(
            self.fTextSize,
            &kStdFakeBoldInterpKeys,
            &kStdFakeBoldInterpValues,
            kStdFakeBoldInterpLength,
        );
        let extra = self.fTextSize * scale;
        if self.fFrameWidth >= 0.0 {
            self.fFrameWidth += extra;
        } else {
            self.fFlags |= flags::FRAME_AND_FILL;
            self.fFrameWidth = extra;
            let paint = crate::src::core::SkPaint::SkPaint::default();
            self.fMiterLimit = paint.getStrokeMiter();
            self.stroke_join_cap =
                ((paint.getStrokeJoin() as u8) & 0x0f) | ((paint.getStrokeCap() as u8) << 4);
        }
    }
}

// Source: src/core/SkScalerContext.cpp:1048-1051
pub(crate) fn sk_relax(x: f32) -> f32 {
    let n = crate::include::core::SkScalar::SkScalarRoundToScalar(x * 1024.);
    n / 1024.0
}

// Source: src/core/SkScalerContext.cpp:1053-1064
pub(crate) fn compute_mask_format(font: &crate::src::core::SkFont::SkFont) -> Format {
    use crate::src::core::SkFont::Edging;
    match font.getEdging() {
        Edging::kAlias => Format::kBW_Format,
        Edging::kAntiAlias => Format::kA8_Format,
        Edging::kSubpixelAntiAlias => Format::kLCD16_Format,
        _ => {
            // Source: src/core/SkScalerContext.cpp:1062-1063
            crate::SkASSERT!(@source b"src/core/SkScalerContext.cpp\0".as_ptr(),1062; @condition b"false\0".as_ptr(); false);
            Format::kA8_Format
        }
    }
}

// Source: src/core/SkScalerContext.cpp:1069-1069
const SK_MAX_SIZE_FOR_LCDTEXT: f32 = 48.;

// Source: src/core/SkScalerContext.cpp:1072-1072
pub const gMaxSize2ForLCDText: f32 = SK_MAX_SIZE_FOR_LCDTEXT * SK_MAX_SIZE_FOR_LCDTEXT;

// Source: src/core/SkScalerContext.cpp:1074-1083
pub(crate) fn too_big_for_lcd(rec: &SkScalerContextRec, check_post_2x2: bool) -> bool {
    if check_post_2x2 {
        let mut area =
            rec.fPost2x2[0][0] * rec.fPost2x2[1][1] - rec.fPost2x2[1][0] * rec.fPost2x2[0][1];
        area *= rec.fTextSize * rec.fTextSize;
        area > gMaxSize2ForLCDText
    } else {
        rec.fTextSize > SK_MAX_SIZE_FOR_LCDTEXT
    }
}

// Source: src/core/SkScalerContext.cpp:1087-1093
// Source: src/core/SkScalerContext.cpp:1095-1095
// Source: src/core/SkScalerContext.cpp:1097-1097
// Source: src/core/SkScalerContext.cpp:1099-1102
// Source: src/core/SkScalerContext.cpp:1104-1104
// Source: src/core/SkScalerContext.cpp:1106-1120
// Source: src/core/SkScalerContext.cpp:1122-1123
// Source: src/core/SkScalerContext.cpp:1125-1125
// Source: src/core/SkScalerContext.cpp:1127-1129
// Source: src/core/SkScalerContext.cpp:1131-1135
// Source: src/core/SkScalerContext.cpp:1137-1145
// Source: src/core/SkScalerContext.cpp:1147-1147
// Source: src/core/SkScalerContext.cpp:1151-1156
// Source: src/core/SkScalerContext.cpp:1158-1159
// Source: src/core/SkScalerContext.cpp:1161-1164
// Source: src/core/SkScalerContext.cpp:1166-1179
// Source: src/core/SkScalerContext.cpp:1181-1200
// Source: src/core/SkScalerContext.cpp:1203-1204
// Source: src/core/SkScalerContext.cpp:1211-1212
// Source: src/core/SkScalerContext.cpp:1214-1219
// Source: src/core/SkScalerContext.cpp:1221-1222
// Adapted signature: propagate the real canonical color-filter executor error; ordinary
// successful record construction keeps the native assignment/flag/effect order.
pub fn MakeRecAndEffects<'a>(
    font: &crate::src::core::SkFont::SkFont,
    paint: &'a crate::src::core::SkPaint::SkPaint,
    surface_props: &crate::include::core::SkSurfaceProps::SkSurfaceProps,
    scaler_context_flags: SkScalerContextFlags,
    device_matrix: &crate::src::core::SkMatrix::SkMatrix,
) -> Result<
    (SkScalerContextRec, SkScalerContextEffects<'a>),
    crate::src::effects::colorfilters::SkColorFilterBase::ColorFilterEvaluationError,
> {
    use crate::include::core::SkSurfaceProps::SkPixelGeometry::*;
    use crate::src::core::SkMatrix::SkMatrix;
    use crate::src::core::SkPaint::Style;
    crate::SkASSERT!(@source b"src/core/SkScalerContext.cpp\0".as_ptr(),1093; @condition b"!deviceMatrix.hasPerspective()\0".as_ptr(); !device_matrix.hasPerspective());
    // Native sk_bzero: all scalar/bitfield values admit zero, including BW mask format.
    let mut rec: SkScalerContextRec = unsafe { std::mem::zeroed() };
    let typeface = font.getTypeface();
    rec.fTypefaceID = typeface.uniqueID();
    rec.fTextSize = font.getSize();
    rec.fPreScaleX = font.getScaleX();
    rec.fPreSkewX = font.getSkewX();
    let mut check_post_2x2 = false;
    let mask = device_matrix.getType();
    if mask & SkMatrix::kScale_Mask != 0 {
        rec.fPost2x2[0][0] = sk_relax(device_matrix.getScaleX());
        rec.fPost2x2[1][1] = sk_relax(device_matrix.getScaleY());
        check_post_2x2 = true;
    } else {
        rec.fPost2x2[0][0] = 1.;
        rec.fPost2x2[1][1] = 1.;
    }
    if mask & SkMatrix::kAffine_Mask != 0 {
        rec.fPost2x2[0][1] = sk_relax(device_matrix.getSkewX());
        rec.fPost2x2[1][0] = sk_relax(device_matrix.getSkewY());
        check_post_2x2 = true;
    } else {
        rec.fPost2x2[0][1] = 0.;
        rec.fPost2x2[1][0] = 0.;
    }
    let style = paint.getStyle();
    let stroke_width = paint.getStrokeWidth();
    let mut record_flags = 0u16;
    if font.isEmbolden() {
        record_flags |= flags::EMBOLDEN;
    }
    if style != Style::kFill_Style && stroke_width >= 0. {
        rec.fFrameWidth = stroke_width;
        rec.fMiterLimit = paint.getStrokeMiter();
        rec.stroke_join_cap =
            ((paint.getStrokeJoin() as u8) & 0x0f) | ((paint.getStrokeCap() as u8) << 4);
        if style == Style::kStrokeAndFill_Style {
            record_flags |= flags::FRAME_AND_FILL;
        }
    } else {
        rec.fFrameWidth = -1.;
        rec.fMiterLimit = 0.;
        rec.stroke_join_cap = 0;
    }
    rec.fMaskFormat = compute_mask_format(font);
    if rec.fMaskFormat == Format::kLCD16_Format {
        if too_big_for_lcd(&rec, check_post_2x2) {
            rec.fMaskFormat = Format::kA8_Format;
            record_flags |= flags::GEN_A8_FROM_LCD;
        } else {
            match surface_props.pixelGeometry() {
                kUnknown_SkPixelGeometry => {
                    rec.fMaskFormat = Format::kA8_Format;
                    record_flags |= flags::GEN_A8_FROM_LCD;
                }
                kRGB_H_SkPixelGeometry => {}
                kBGR_H_SkPixelGeometry => record_flags |= flags::LCD_BGR_ORDER,
                kRGB_V_SkPixelGeometry => record_flags |= flags::LCD_VERTICAL,
                kBGR_V_SkPixelGeometry => {
                    record_flags |= flags::LCD_VERTICAL | flags::LCD_BGR_ORDER
                }
            }
        }
    }
    if font.isEmbeddedBitmaps() {
        record_flags |= flags::EMBEDDED_BITMAP_TEXT;
    }
    if font.isSubpixel() {
        record_flags |= flags::SUBPIXEL_POSITIONING;
    }
    if font.isForceAutoHinting() {
        record_flags |= flags::FORCE_AUTOHINTING;
    }
    if font.isLinearMetrics() {
        record_flags |= flags::LINEAR_METRICS;
    }
    if font.isBaselineSnap() {
        record_flags |= flags::BASELINE_SNAP;
    }
    if typeface.glyphMaskNeedsCurrentColor() {
        record_flags |= flags::NEEDS_FOREGROUND_COLOR;
        rec.fForegroundColor = paint.getColor();
    }
    rec.fFlags = record_flags;
    rec.set_hinting(font.getHinting());
    rec.set_luminance_color(crate::src::core::SkPaintPriv::ComputeLuminanceColor(paint)?);
    rec.set_device_gamma(surface_props.textGamma());
    rec.set_contrast(surface_props.textContrast());
    if scaler_context_flags.0 & SkScalerContextFlags::FAKE_GAMMA.0 == 0 {
        rec.ignore_gamma();
    }
    if scaler_context_flags.0 & SkScalerContextFlags::BOOST_CONTRAST.0 == 0 {
        rec.set_contrast(0.);
    }
    let effects = SkScalerContextEffects::from_paint(paint);
    Ok((rec, effects))
}

// Source: src/core/SkScalerContext.cpp:1224-1232
pub fn CreateDescriptorAndEffectsUsingPaint<'a>(
    font: &crate::src::core::SkFont::SkFont,
    paint: &'a crate::src::core::SkPaint::SkPaint,
    props: &crate::include::core::SkSurfaceProps::SkSurfaceProps,
    flags: SkScalerContextFlags,
    matrix: &crate::src::core::SkMatrix::SkMatrix,
) -> Result<
    (
        crate::src::core::SkDescriptor::SkAutoDescriptor,
        SkScalerContextEffects<'a>,
    ),
    crate::RasterError,
> {
    let (rec, effects) = MakeRecAndEffects(font, paint, props, flags, matrix)?;
    let descriptor = AutoDescriptorGivenRecAndEffects(&rec, &effects);
    Ok((descriptor, effects))
}

// Source: src/core/SkScalerContext.cpp:1234-1249
fn calculate_size_and_flatten(
    rec: &SkScalerContextRec,
    effects: &SkScalerContextEffects<'_>,
    buffer: &mut crate::src::core::SkWriteBuffer::SkBinaryWriteBuffer,
) -> usize {
    use crate::src::core::SkWriteBuffer::SkWriteBuffer;
    let mut size = core::mem::size_of_val(rec);
    let mut count = 1;
    if effects.fPathEffect.is_some() || effects.fMaskFilter.is_some() {
        if let Some(effect) = effects.fPathEffect {
            buffer.writeFlattenable(Some(effect));
        }
        if let Some(filter) = effects.fMaskFilter {
            buffer.writeFlattenable(Some(filter));
        }
        count += 1;
        size += buffer.bytesWritten();
    }
    size += crate::src::core::SkDescriptor::SkDescriptor::compute_overhead(count);
    size
}

// Source: src/core/SkScalerContext.cpp:1251-1263
// SkScalerContextRec has no padding in its initialized 56-byte native record layout.
fn generate_descriptor(
    rec: &SkScalerContextRec,
    effect_buffer: &crate::src::core::SkWriteBuffer::SkBinaryWriteBuffer,
    desc: &mut crate::src::core::SkDescriptor::SkDescriptor,
) {
    const _: () = assert!(std::mem::size_of::<SkScalerContextRec>() == 56);
    let rec_bytes = unsafe {
        std::slice::from_raw_parts(
            (rec as *const SkScalerContextRec).cast::<u8>(),
            std::mem::size_of::<SkScalerContextRec>(),
        )
    };
    desc.add_entry(K_REC_SK_DESCRIPTOR_TAG, rec_bytes.len(), Some(rec_bytes));
    if effect_buffer.bytesWritten() > 0 {
        let dst = desc.add_entry(
            K_EFFECTS_SK_DESCRIPTOR_TAG,
            effect_buffer.bytesWritten(),
            None,
        );
        unsafe { effect_buffer.writeToMemory(dst.as_mut_ptr()) };
    }
    desc.compute_checksum();
}

// Source: src/core/SkScalerContext.cpp:1265-1276
pub fn AutoDescriptorGivenRecAndEffects(
    rec: &SkScalerContextRec,
    effects: &SkScalerContextEffects<'_>,
) -> crate::src::core::SkDescriptor::SkAutoDescriptor {
    let mut buffer = crate::src::core::SkWriteBuffer::SkBinaryWriteBuffer::new(Default::default());
    let mut ad = crate::src::core::SkDescriptor::SkAutoDescriptor::with_size(
        calculate_size_and_flatten(rec, effects, &mut buffer),
    );
    generate_descriptor(rec, &buffer, ad.get_desc_mut());
    ad
}

// Source: src/core/SkScalerContext.cpp:1278-1288
pub fn DescriptorGivenRecAndEffects(
    rec: &SkScalerContextRec,
    effects: &SkScalerContextEffects<'_>,
) -> crate::src::core::SkDescriptor::SkDescriptor {
    let mut buffer = crate::src::core::SkWriteBuffer::SkBinaryWriteBuffer::new(Default::default());
    let mut desc = crate::src::core::SkDescriptor::SkDescriptor::alloc(calculate_size_and_flatten(
        rec,
        effects,
        &mut buffer,
    ));
    generate_descriptor(rec, &buffer, &mut desc);
    desc
}

// Source: src/core/SkScalerContext.cpp:1290-1292
pub fn DescriptorBufferGiveRec(
    rec: &SkScalerContextRec,
    buffer: &mut crate::src::core::SkDescriptor::SkDescriptor,
) {
    let effect_buffer =
        crate::src::core::SkWriteBuffer::SkBinaryWriteBuffer::new(Default::default());
    generate_descriptor(rec, &effect_buffer, buffer);
}

// Source: src/core/SkScalerContext.h:360-362
// Source: src/core/SkScalerContext.cpp:1294-1299
pub fn CheckBufferSizeForRec(
    rec: &SkScalerContextRec,
    effects: &SkScalerContextEffects<'_>,
    size: usize,
) -> bool {
    let mut buffer = crate::src::core::SkWriteBuffer::SkBinaryWriteBuffer::new(Default::default());
    size >= calculate_size_and_flatten(rec, effects, &mut buffer)
}

// Source: src/core/SkScalerContext.cpp:1301-1326
// This is the actual native explicit Empty scaler. It is never selected here
// as a fallback for an unavailable platform scaler or for a nonempty typeface.
impl SkScalerContext {
    /// Safety: retain the same concrete typeface/effect leases as new().
    pub unsafe fn MakeEmpty(
        typeface: core::ptr::NonNull<crate::src::core::SkTypeface::SkTypeface>,
        effects: &SkScalerContextEffects<'_>,
        desc: &crate::src::core::SkDescriptor::SkDescriptor,
        path_owner: &crate::include::core::SkRefCnt::sk_sp<
            dyn crate::src::core::SkPathEffect::SkPathEffect,
        >,
        mask_owner: Option<&std::sync::Arc<dyn crate::src::core::SkMaskFilter::SkMaskFilter>>,
    ) -> Box<dyn SkScalerContextVirtual> {
        struct SkScalerContext_Empty {
            base: SkScalerContext,
        }
        impl SkScalerContextVirtual for SkScalerContext_Empty {
            fn base(&self) -> &SkScalerContext {
                &self.base
            }
            fn base_mut(&mut self) -> &mut SkScalerContext {
                &mut self.base
            }
            unsafe fn generateMetrics(
                &mut self,
                glyph: &crate::src::core::SkGlyph::SkGlyph,
                _alloc: &mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
            ) -> GlyphMetrics {
                GlyphMetrics::new(glyph.maskFormat())
            }
            // Native Empty has no image payload; this exact override does not write it.
            unsafe fn generateImage(
                &mut self,
                _glyph: &crate::src::core::SkGlyph::SkGlyph,
                _buffer: *mut std::ffi::c_void,
            ) {
            }
            unsafe fn generatePath(
                &mut self,
                _glyph: &crate::src::core::SkGlyph::SkGlyph,
            ) -> Option<GeneratedPath> {
                None
            }
            unsafe fn generateFontMetrics(
                &mut self,
                metrics: &mut crate::include::core::SkFontMetrics::SkFontMetrics,
            ) {
                // SkFontMetrics consists exclusively of initialized u32/f32 fields.
                unsafe {
                    std::ptr::write_bytes(metrics, 0, 1);
                }
            }
        }
        Box::new(SkScalerContext_Empty {
            base: unsafe { Self::new(typeface, effects, desc, path_owner, mask_owner) },
        })
    }
}

// Source: src/core/SkScalerContext.h:157-169
impl SkScalerContextRec {
    pub fn dump(&self) -> crate::src::core::SkString::SkString {
        let mut msg = crate::src::core::SkString::SkString::new();
        unsafe {
            msg.appendf(b"    Rec\n\0".as_ptr());
            msg.appendf(
                b"      textsize %a prescale %a preskew %a post [%a %a %a %a]\n\0".as_ptr(),
                self.fTextSize as f64,
                self.fPreScaleX as f64,
                self.fPreSkewX as f64,
                self.fPost2x2[0][0] as f64,
                self.fPost2x2[0][1] as f64,
                self.fPost2x2[1][0] as f64,
                self.fPost2x2[1][1] as f64,
            );
            msg.appendf(
                b"      frame %g miter %g format %d join %d cap %d flags %#hx\n\0".as_ptr(),
                self.fFrameWidth as f64,
                self.fMiterLimit as f64,
                self.fMaskFormat as i32,
                (self.stroke_join_cap & 15) as i32,
                (self.stroke_join_cap >> 4) as i32,
                self.fFlags as i32,
            );
            msg.appendf(
                b"      lum bits %x, device gamma %d, contrast %d\n\0".as_ptr(),
                self.fLumBits,
                self.fDeviceGamma as i32,
                self.fContrast as i32,
            );
            msg.appendf(
                b"      foreground color %x\n\0".as_ptr(),
                self.fForegroundColor,
            );
        }
        msg
    }
}
// Source: src/core/SkScalerContext.h:78-78
impl SkScalerContextRec {
    /// Native implicit default construction initializes only this in-class member;
    /// the remaining bytes stay uninitialized until their actual producer writes them.
    pub fn new_uninit() -> core::mem::MaybeUninit<Self> {
        let mut rec = core::mem::MaybeUninit::<Self>::uninit();
        unsafe {
            core::ptr::addr_of_mut!((*rec.as_mut_ptr()).fForegroundColor).write(0xFF000000);
        }
        rec
    }
}
// Source: src/core/SkScalerContext.cpp:56-56
// Structural source: src/core/SkScalerContext.cpp:102-107: native if constexpr(false) diagnostic statements produce no executable body.
const kSkScalerContextDumpRec: bool = false;
