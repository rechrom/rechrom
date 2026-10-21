use crate::include::core::SkFontTypes::SkFontHinting;
use crate::src::core::SkFontPriv::SkFontPriv;

use crate::src::core::SkTypeface::{SkTypeface, SkTypefaceRef};

// Source: include/core/SkFont.h:43-47
// A scoped enum without an explicit underlying type has native int storage.
// Retain every underlying value, including unnamed values, without invalid
// Rust enum discriminants. Font's actual stored byte remains separate below.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Edging(pub core::ffi::c_int);
impl Edging {
    pub const kAlias: Self = Self(0);
    pub const kAntiAlias: Self = Self(1);
    pub const kSubpixelAntiAlias: Self = Self(2);
}
impl SkFont {
    // Source: include/core/SkFont.h:105-105
    pub fn isForceAutoHinting(&self) -> bool {
        self.fFlags & Self::kForceAutoHinting_PrivFlag != 0
    }
}
impl SkFont {
    // Source: include/core/SkFont.h:111-111
    pub fn isEmbeddedBitmaps(&self) -> bool {
        self.fFlags & Self::kEmbeddedBitmaps_PrivFlag != 0
    }
}
impl SkFont {
    // Source: include/core/SkFont.h:117-117
    pub fn isSubpixel(&self) -> bool {
        self.fFlags & Self::kSubpixel_PrivFlag != 0
    }
}
impl SkFont {
    // Source: include/core/SkFont.h:123-123
    pub fn isLinearMetrics(&self) -> bool {
        self.fFlags & Self::kLinearMetrics_PrivFlag != 0
    }
}
impl SkFont {
    // Source: include/core/SkFont.h:130-130
    pub fn isEmbolden(&self) -> bool {
        self.fFlags & Self::kEmbolden_PrivFlag != 0
    }
}
impl SkFont {
    // Source: include/core/SkFont.h:137-137
    pub fn isBaselineSnap(&self) -> bool {
        self.fFlags & Self::kBaselineSnap_PrivFlag != 0
    }
}
impl SkFont {
    // Source: include/core/SkFont.h:184-184
    pub fn getEdging(&self) -> Edging {
        Edging(self.fEdging as core::ffi::c_int)
    }
}
impl SkFont {
    // Source: include/core/SkFont.h:198-198
    pub fn getHinting(&self) -> SkFontHinting {
        SkFontHinting(self.fHinting as core::ffi::c_int)
    }
}
impl SkFont {
    // Source: include/core/SkFont.h:212-215
    pub fn getTypeface(&self) -> &SkTypeface {
        self.fTypeface.get().expect("non-null typeface")
    }
}
impl SkFont {
    // Source: include/core/SkFont.h:222-222
    pub fn getSize(&self) -> f32 {
        self.fSize
    }
}
impl SkFont {
    // Source: include/core/SkFont.h:229-229
    pub fn getScaleX(&self) -> f32 {
        self.fScaleX
    }
}
impl SkFont {
    // Source: include/core/SkFont.h:236-236
    pub fn getSkewX(&self) -> f32 {
        self.fSkewX
    }
}
impl SkFont {
    // Source: include/core/SkFont.h:242-245
    pub fn refTypeface(&self) -> SkTypefaceRef {
        crate::SkASSERT!(@source b"include/core/SkFont.h\0".as_ptr(),243; @condition b"fTypeface\0".as_ptr(); self.fTypeface.get().is_some());
        self.fTypeface.clone()
    }
}
impl SkFont {
    // Source: include/core/SkFont.h:328-330
    pub unsafe fn countText(
        &self,
        text: *const std::ffi::c_void,
        byte_length: usize,
        encoding: crate::include::core::SkFontTypes::SkTextEncoding,
    ) -> usize {
        self.textToGlyphs(text, byte_length, encoding, &mut [])
    }
}
impl SkFont {
    // Source: include/core/SkFont.h:341-344
    /// Safety: retain measureText's native text/encoding lease and transferable
    /// typeface/backend graph. The native omitted paint argument is null.
    pub unsafe fn measureText_without_paint(
        &self,
        text: *const std::ffi::c_void,
        byte_length: usize,
        encoding: crate::include::core::SkFontTypes::SkTextEncoding,
        bounds: Option<&mut crate::src::core::SkRect::SkRect>,
    ) -> Result<f32, crate::RasterError> {
        unsafe { self.measureText(text, byte_length, encoding, bounds, None) }
    }
}
impl SkFont {
    // Source: include/core/SkFont.h:392-394
    pub unsafe fn getWidths(
        &self,
        ids: &[u16],
        widths: &mut [f32],
    ) -> Result<(), crate::RasterError> {
        unsafe { self.getWidthsBounds(ids, widths, &mut [], None) }
    }
    // Source: include/core/SkFont.h:395-399
    pub unsafe fn getWidth(&self, id: u16) -> Result<f32, crate::RasterError> {
        let mut width = 0.0;
        unsafe {
            self.getWidthsBounds(&[id], std::slice::from_mut(&mut width), &mut [], None)?;
        }
        Ok(width)
    }
    // Source: include/core/SkFont.h:409-412
    pub unsafe fn getBounds(
        &self,
        ids: &[u16],
        bounds: &mut [crate::src::core::SkRect::SkRect],
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<(), crate::RasterError> {
        unsafe { self.getWidthsBounds(ids, &mut [], bounds, paint) }
    }
    // Source: include/core/SkFont.h:413-417
    pub unsafe fn getGlyphBounds(
        &self,
        id: u16,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<crate::src::core::SkRect::SkRect, crate::RasterError> {
        let mut bounds = crate::src::core::SkRect::SkRect::MakeEmpty();
        unsafe {
            self.getBounds(&[id], std::slice::from_mut(&mut bounds), paint)?;
        }
        Ok(bounds)
    }
}

impl SkFont {
    // Source: include/core/SkFont.h:491-491
    /// Safety: retain getMetrics's actual transferable backend/cache graph.
    pub unsafe fn getSpacing(&self) -> Result<f32, crate::RasterError> {
        unsafe { self.getMetrics(None) }
    }
}

impl SkFont {
    // Source: include/core/SkFont.h:501-508
    // Source: include/core/SkFont.h:510-515
    pub(crate) const kForceAutoHinting_PrivFlag: u8 = 1 << 0;
}
impl SkFont {
    pub(crate) const kEmbeddedBitmaps_PrivFlag: u8 = 1 << 1;
}
impl SkFont {
    pub(crate) const kSubpixel_PrivFlag: u8 = 1 << 2;
}
impl SkFont {
    pub(crate) const kLinearMetrics_PrivFlag: u8 = 1 << 3;
}
impl SkFont {
    pub(crate) const kEmbolden_PrivFlag: u8 = 1 << 4;
}
impl SkFont {
    pub(crate) const kBaselineSnap_PrivFlag: u8 = 1 << 5;
}
impl SkFont {
    pub(crate) const kAllFlags: u8 = Self::kForceAutoHinting_PrivFlag
        | Self::kEmbeddedBitmaps_PrivFlag
        | Self::kSubpixel_PrivFlag
        | Self::kLinearMetrics_PrivFlag
        | Self::kEmbolden_PrivFlag
        | Self::kBaselineSnap_PrivFlag;
}
// Source: include/core/SkFont.h:517-523
#[derive(Clone)]
pub struct SkFont {
    pub(crate) fTypeface: SkTypefaceRef,
    pub(crate) fSize: f32,
    pub(crate) fScaleX: f32,
    pub(crate) fSkewX: f32,
    pub(crate) fFlags: u8,
    pub(crate) fEdging: u8,
    pub(crate) fHinting: u8,
}

const DEFAULT_FLAGS: u8 = SkFont::kBaselineSnap_PrivFlag;

const DEFAULT_EDGING: Edging = Edging::kAntiAlias;

const DEFAULT_HINTING: SkFontHinting = crate::src::core::SkPaintDefaults::SkPaintDefaults_Hinting;
// Source: src/core/SkFont.cpp:43-46
const DEFAULT_SIZE: f32 = crate::src::core::SkPaintDefaults::SkPaintDefaults_TextSize;
// Source: src/core/SkFont.cpp:48-50
fn valid_size(size: f32) -> f32 {
    if 0. < size {
        size
    } else {
        0.
    }
}
// Source: src/core/SkFont.cpp:52-63
impl SkFont {
    pub fn new(face: Option<SkTypefaceRef>, size: f32, scale_x: f32, skew_x: f32) -> Self {
        let mut font = Self {
            fTypeface: face.unwrap_or_default(),
            fSize: valid_size(size),
            fScaleX: scale_x,
            fSkewX: skew_x,
            fFlags: DEFAULT_FLAGS,
            fEdging: DEFAULT_EDGING.0 as u8,
            fHinting: DEFAULT_HINTING.0 as u8,
        };
        if font.fTypeface.get().is_none() {
            font.fTypeface = SkTypeface::MakeEmpty();
        }
        font
    }
}
impl SkFont {
    // Source: src/core/SkFont.cpp:65-65
    pub fn with_size(face: Option<SkTypefaceRef>, size: f32) -> Self {
        Self::new(face, size, 1., 0.)
    }
}
impl SkFont {
    // Source: src/core/SkFont.cpp:67-67
    pub fn from_typeface(face: Option<SkTypefaceRef>) -> Self {
        Self::new(face, DEFAULT_SIZE, 1., 0.)
    }
}
// Source: src/core/SkFont.cpp:69-69
impl Default for SkFont {
    fn default() -> Self {
        Self::with_size(None, DEFAULT_SIZE)
    }
}
// Source: include/core/SkFont.h:98-98
// Existing Rust PartialEq default ne is !eq, preserving native inline !=.
// Source: src/core/SkFont.cpp:71-79
impl PartialEq for SkFont {
    fn eq(&self, b: &Self) -> bool {
        let same_face = self.fTypeface.as_ptr() == b.fTypeface.as_ptr();
        same_face
            && self.fSize == b.fSize
            && self.fScaleX == b.fScaleX
            && self.fSkewX == b.fSkewX
            && self.fFlags == b.fFlags
            && self.fEdging == b.fEdging
            && self.fHinting == b.fHinting
    }
}
// Source: src/core/SkFont.cpp:81-89
// Direct native logging calls; temporary/typeface backing lives through each call.
impl SkFont {
    // Source: include/core/SkFont.h:496-496
    pub fn dump(&self) {
        use core::ffi::c_void;
        let typeface = self
            .fTypeface
            .as_ptr()
            .map_or(core::ptr::null(), |face| face.as_ptr().cast::<c_void>());
        unsafe {
            crate::SkDebugf!(@source b"src/core/SkFont.cpp\0".as_ptr(),82; b"typeface %p\n\0".as_ptr(),typeface);
            crate::SkDebugf!(@source b"src/core/SkFont.cpp\0".as_ptr(),83; b"size %g\n\0".as_ptr(),self.fSize as f64);
            crate::SkDebugf!(@source b"src/core/SkFont.cpp\0".as_ptr(),84; b"skewx %g\n\0".as_ptr(),self.fSkewX as f64);
            crate::SkDebugf!(@source b"src/core/SkFont.cpp\0".as_ptr(),85; b"scalex %g\n\0".as_ptr(),self.fScaleX as f64);
            crate::SkDebugf!(@source b"src/core/SkFont.cpp\0".as_ptr(),86; b"flags 0x%X\n\0".as_ptr(),self.fFlags as i32);
            crate::SkDebugf!(@source b"src/core/SkFont.cpp\0".as_ptr(),87; b"edging %u\n\0".as_ptr(),self.fEdging as u32);
            crate::SkDebugf!(@source b"src/core/SkFont.cpp\0".as_ptr(),88; b"hinting %u\n\0".as_ptr(),self.fHinting as u32);
        }
    }
}
// Source: src/core/SkFont.cpp:91-96
impl SkFont {
    pub fn setTypeface(&mut self, face: Option<SkTypefaceRef>) {
        // Native sk_sp::reset publishes its replacement before releasing the old owner.
        drop(std::mem::replace(
            &mut self.fTypeface,
            face.unwrap_or_default(),
        ));
        if self.fTypeface.get().is_none() {
            self.fTypeface = SkTypeface::MakeEmpty();
        }
    }
}
// Source: src/core/SkFont.cpp:100-102
fn set_clear_mask(bits: u32, cond: bool, mask: u32) -> u32 {
    if cond {
        bits | mask
    } else {
        bits & !mask
    }
}
impl SkFont {
    // Source: src/core/SkFont.cpp:104-106
    pub fn setForceAutoHinting(&mut self, predicate: bool) {
        self.fFlags = set_clear_mask(
            self.fFlags as u32,
            predicate,
            Self::kForceAutoHinting_PrivFlag as u32,
        ) as u8;
    }
}
impl SkFont {
    // Source: src/core/SkFont.cpp:107-109
    pub fn setEmbeddedBitmaps(&mut self, predicate: bool) {
        self.fFlags = set_clear_mask(
            self.fFlags as u32,
            predicate,
            Self::kEmbeddedBitmaps_PrivFlag as u32,
        ) as u8;
    }
}
impl SkFont {
    // Source: src/core/SkFont.cpp:110-112
    pub fn setSubpixel(&mut self, predicate: bool) {
        self.fFlags = set_clear_mask(
            self.fFlags as u32,
            predicate,
            Self::kSubpixel_PrivFlag as u32,
        ) as u8;
    }
}
impl SkFont {
    // Source: src/core/SkFont.cpp:113-115
    pub fn setLinearMetrics(&mut self, predicate: bool) {
        self.fFlags = set_clear_mask(
            self.fFlags as u32,
            predicate,
            Self::kLinearMetrics_PrivFlag as u32,
        ) as u8;
    }
}
impl SkFont {
    // Source: src/core/SkFont.cpp:116-118
    pub fn setEmbolden(&mut self, predicate: bool) {
        self.fFlags = set_clear_mask(
            self.fFlags as u32,
            predicate,
            Self::kEmbolden_PrivFlag as u32,
        ) as u8;
    }
}
impl SkFont {
    // Source: src/core/SkFont.cpp:119-121
    pub fn setBaselineSnap(&mut self, predicate: bool) {
        self.fFlags = set_clear_mask(
            self.fFlags as u32,
            predicate,
            Self::kBaselineSnap_PrivFlag as u32,
        ) as u8;
    }
}
impl SkFont {
    // Source: src/core/SkFont.cpp:122-124
    pub fn setEdging(&mut self, edging: Edging) {
        self.fEdging = edging.0 as u8;
    }
}
impl SkFont {
    // Source: src/core/SkFont.cpp:126-128
    pub fn setHinting(&mut self, hinting: SkFontHinting) {
        self.fHinting = hinting.0 as u8;
    }
}
impl SkFont {
    // Source: src/core/SkFont.cpp:130-138
    pub fn setSize(&mut self, size: f32) {
        self.fSize = valid_size(size);
    }
}
impl SkFont {
    pub fn setScaleX(&mut self, scale: f32) {
        self.fScaleX = scale;
    }
}
impl SkFont {
    pub fn setSkewX(&mut self, skew: f32) {
        self.fSkewX = skew;
    }
}
impl SkFont {
    // Source: src/core/SkFont.cpp:140-144
    pub fn makeWithSize(&self, new_size: f32) -> Self {
        let mut font = self.clone();
        font.setSize(new_size);
        font
    }
}
impl SkFont {
    // Source: src/core/SkFont.cpp:148-150
    // Source: src/core/SkFont.cpp:152-153
    // Source: src/core/SkFont.cpp:155-157
    // Source: src/core/SkFont.cpp:159-166
    pub(crate) fn setupForAsPaths(
        &mut self,
        paint: Option<&mut crate::src::core::SkPaint::SkPaint>,
    ) -> f32 {
        let flags_to_ignore = Self::kEmbeddedBitmaps_PrivFlag | Self::kForceAutoHinting_PrivFlag;
        self.fFlags = (self.fFlags & !flags_to_ignore) | Self::kSubpixel_PrivFlag;
        self.setHinting(SkFontHinting::kNone);
        if self.getEdging() == Edging::kSubpixelAntiAlias {
            self.setEdging(Edging::kAntiAlias)
        }
        if let Some(paint) = paint {
            paint.setStyle(crate::src::core::SkPaint::Style::kFill_Style);
            paint.setPathEffect(crate::include::core::SkRefCnt::sk_sp::default());
        }
        let text_size = self.fSize;
        self.setSize(crate::src::core::SkFontPriv::SkFontPriv::kCanonicalTextSizeForPaths as f32);
        text_size / crate::src::core::SkFontPriv::SkFontPriv::kCanonicalTextSizeForPaths as f32
    }
}
impl SkFont {
    // Source: src/core/SkFont.cpp:168-172
    pub(crate) fn hasSomeAntiAliasing(&self) -> bool {
        let edging = self.getEdging();
        edging == Edging::kAntiAlias || edging == Edging::kSubpixelAntiAlias
    }
}
impl SkFont {
    // Source: src/core/SkFont.cpp:174-176
    // Source: src/core/SkFont.cpp:178-180
    pub fn unicharToGlyph(&self, uni: i32) -> u16 {
        self.getTypeface().unicharToGlyph(uni)
    }
}
impl SkFont {
    pub fn unicharsToGlyphs(&self, unis: &[i32], glyphs: &mut [u16]) {
        self.getTypeface().unicharsToGlyphs(unis, glyphs);
    }
}
impl SkFont {
    // Source: src/core/SkFont.cpp:182-185
    pub unsafe fn textToGlyphs(
        &self,
        text: *const std::ffi::c_void,
        byte_length: usize,
        encoding: crate::include::core::SkFontTypes::SkTextEncoding,
        glyphs: &mut [u16],
    ) -> usize {
        self.getTypeface()
            .textToGlyphs(text, byte_length, encoding, glyphs)
    }
}
impl SkFont {
    // Source: src/core/SkFont.cpp:187-231
    /// Safety: text has its native encoding/alignment lease; the complete retained
    /// typeface/effect/backend graph is transferable for the process-global strike cache.
    pub unsafe fn measureText(
        &self,
        text: *const std::ffi::c_void,
        length: usize,
        encoding: crate::include::core::SkFontTypes::SkTextEncoding,
        mut bounds: Option<&mut crate::src::core::SkRect::SkRect>,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<f32, crate::RasterError> {
        let atg = unsafe {
            crate::src::core::SkFontPriv::SkAutoToGlyphs::new(self, text, length, encoding)
        };
        let ids = atg.glyphs();
        if ids.is_empty() {
            if let Some(bounds) = bounds {
                bounds.setEmpty();
            }
            return Ok(0.0);
        }
        let (spec, scale) =
            crate::src::core::SkStrikeSpec::SkStrikeSpec::MakeCanonicalized(self, paint)?;
        let mut metrics = unsafe { crate::src::core::SkStrikeSpec::SkBulkGlyphMetrics::new(&spec) };
        Ok(unsafe {
            metrics.withGlyphs(ids, |glyphs| {
                let mut width = 0.0;
                if let Some(bounds) = bounds.as_deref_mut() {
                    *bounds = glyphs[0].as_ref().rect();
                    width = glyphs[0].as_ref().advanceX();
                    for glyph in &glyphs[1..] {
                        let glyph = glyph.as_ref();
                        let mut r = glyph.rect();
                        r.offset(width, 0.0);
                        bounds.join(&r);
                        width += glyph.advanceX();
                    }
                } else {
                    for glyph in glyphs {
                        width += glyph.as_ref().advanceX();
                    }
                }
                if scale != 1.0 {
                    width *= scale;
                    if let Some(bounds) = bounds {
                        bounds.fLeft *= scale;
                        bounds.fTop *= scale;
                        bounds.fRight *= scale;
                        bounds.fBottom *= scale;
                    }
                }
                width
            })
        })
    }
}

// Source: src/core/SkFont.cpp:233-238
fn scale_pos(r: crate::src::core::SkRect::SkRect, s: f32) -> crate::src::core::SkRect::SkRect {
    debug_assert!(s >= 0.);
    crate::src::core::SkRect::SkRect::MakeLTRB(r.fLeft * s, r.fTop * s, r.fRight * s, r.fBottom * s)
}
impl SkFont {
    // Source: src/core/SkFont.cpp:245-266
    /// Safety: retain the actual transferable cache/backend graph and output leases.
    pub unsafe fn getWidthsBounds(
        &self,
        ids: &[u16],
        widths: &mut [f32],
        bounds: &mut [crate::src::core::SkRect::SkRect],
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<(), crate::RasterError> {
        let (spec, scale) =
            crate::src::core::SkStrikeSpec::SkStrikeSpec::MakeCanonicalized(self, paint)?;
        let mut metrics = unsafe { crate::src::core::SkStrikeSpec::SkBulkGlyphMetrics::new(&spec) };
        unsafe {
            metrics.withGlyphs(ids, |glyphs| {
                if !bounds.is_empty() {
                    for (bound, glyph) in bounds.iter_mut().zip(glyphs) {
                        *bound = scale_pos(glyph.as_ref().rect(), scale);
                    }
                }
                if !widths.is_empty() {
                    for (width, glyph) in widths.iter_mut().zip(glyphs) {
                        *width = glyph.as_ref().advanceX() * scale;
                    }
                }
            });
        }
        Ok(())
    }
    // Source: src/core/SkFont.cpp:268-279
    /// Safety: retain the actual transferable cache/backend graph and position leases.
    pub unsafe fn getPos(
        &self,
        ids: &[u16],
        positions: &mut [crate::src::core::SkPoint::SkPoint],
        origin: crate::src::core::SkPoint::SkPoint,
    ) -> Result<(), crate::RasterError> {
        let (spec, scale) =
            crate::src::core::SkStrikeSpec::SkStrikeSpec::MakeCanonicalized(self, None)?;
        let mut metrics = unsafe { crate::src::core::SkStrikeSpec::SkBulkGlyphMetrics::new(&spec) };
        unsafe {
            metrics.withGlyphs(ids, |glyphs| {
                let mut sum = origin;
                for (position, glyph) in positions.iter_mut().zip(glyphs) {
                    *position = sum;
                    sum += glyph.as_ref().advanceVector() * scale;
                }
            });
        }
        Ok(())
    }
    // Source: src/core/SkFont.cpp:281-292
    /// Safety: retain the same real graph and destination leases as getPos.
    pub unsafe fn getXPos(
        &self,
        ids: &[u16],
        positions: &mut [f32],
        origin: f32,
    ) -> Result<(), crate::RasterError> {
        let (spec, scale) =
            crate::src::core::SkStrikeSpec::SkStrikeSpec::MakeCanonicalized(self, None)?;
        let mut metrics = unsafe { crate::src::core::SkStrikeSpec::SkBulkGlyphMetrics::new(&spec) };
        unsafe {
            metrics.withGlyphs(ids, |glyphs| {
                let mut loc = origin;
                for (position, glyph) in positions.iter_mut().zip(glyphs) {
                    *position = loc;
                    loc += glyph.as_ref().advanceX() * scale;
                }
            });
        }
        Ok(())
    }
    // Source: src/core/SkFont.cpp:294-307
    /// Safety: retain actual native glyph/path leases throughout each callback;
    /// callbacks do not race incompatible sidecar mutation on this retained strike.
    pub unsafe fn getPaths<
        F: FnMut(Option<&crate::src::core::SkPath::SkPath>, &crate::src::core::SkMatrix::SkMatrix),
    >(
        &self,
        ids: &[u16],
        mut proc: F,
    ) -> Result<(), crate::RasterError> {
        let mut font = self.clone();
        let scale = font.setupForAsPaths(None);
        let matrix = crate::src::core::SkMatrix::SkMatrix::Scale(scale, scale);
        let spec = crate::src::core::SkStrikeSpec::SkStrikeSpec::MakeWithNoDevice(
            &font,
            None,
            crate::src::core::SkScalerContext::SkScalerContextFlags::NONE,
        )?;
        let mut paths =
            unsafe { crate::src::core::SkStrikeSpec::SkBulkGlyphMetricsAndPaths::new(&spec) };
        for glyph in unsafe { paths.glyphs(ids) } {
            proc(unsafe { glyph.as_ref().path() }, &matrix);
        }
        Ok(())
    }
    // Source: src/core/SkFont.cpp:309-320
    /// Safety: same real native path/backend leases as getPaths.
    pub unsafe fn getPath(
        &self,
        id: u16,
    ) -> Result<Option<crate::src::core::SkPath::SkPath>, crate::RasterError> {
        let mut result = None;
        unsafe {
            self.getPaths(&[id], |path, matrix| {
                if let Some(path) = path {
                    result = path.tryMakeTransform(matrix);
                }
            })?;
        }
        Ok(result)
    }
    // Source: src/core/SkFont.cpp:322-338
    /// Safety: retain the transferable native backend graph for the global cache.
    pub unsafe fn getMetrics(
        &self,
        metrics: Option<&mut crate::include::core::SkFontMetrics::SkFontMetrics>,
    ) -> Result<f32, crate::RasterError> {
        let (spec, scale) =
            crate::src::core::SkStrikeSpec::SkStrikeSpec::MakeCanonicalized(self, None)?;
        // Native storage is unspecified before the cache copy. Initialize Rust's
        // temporary only to satisfy typed reference validity; no default metrics escape.
        let mut storage =
            unsafe { std::mem::zeroed::<crate::include::core::SkFontMetrics::SkFontMetrics>() };
        let metrics = metrics.unwrap_or(&mut storage);
        let cache = unsafe { spec.findOrCreateStrike() };
        *metrics = *cache.getFontMetrics();
        if scale != 1.0 {
            SkFontPriv::ScaleFontMetrics(metrics, scale);
        }
        Ok(metrics.fDescent - metrics.fAscent + metrics.fLeading)
    }
}

// Source: src/core/SkFont.cpp:342-358
impl crate::src::core::SkFontPriv::SkFontPriv {
    pub fn ScaleFontMetrics(
        metrics: &mut crate::include::core::SkFontMetrics::SkFontMetrics,
        scale: f32,
    ) {
        metrics.fTop *= scale;
        metrics.fAscent *= scale;
        metrics.fDescent *= scale;
        metrics.fBottom *= scale;
        metrics.fLeading *= scale;
        metrics.fAvgCharWidth *= scale;
        metrics.fMaxCharWidth *= scale;
        metrics.fXMin *= scale;
        metrics.fXMax *= scale;
        metrics.fXHeight *= scale;
        metrics.fCapHeight *= scale;
        metrics.fUnderlineThickness *= scale;
        metrics.fUnderlinePosition *= scale;
        metrics.fStrikeoutThickness *= scale;
        metrics.fStrikeoutPosition *= scale;
    }
}
impl SkFontPriv {
    // Source: src/core/SkFont.cpp:360-370
    pub fn GetFontBounds(font: &SkFont) -> crate::src::core::SkRect::SkRect {
        let mut m = crate::src::core::SkMatrix::SkMatrix::default();
        m.setScale(font.getSize() * font.getScaleX(), font.getSize());
        m.postSkew(font.getSkewX(), 0.0);
        let typeface = &font.fTypeface;
        let mut bounds = crate::src::core::SkRect::SkRect::MakeEmpty();
        m.mapRect(&mut bounds, &typeface.getBounds());
        bounds
    }
}

// Source: src/core/SkFont.cpp:372-376
// Source: src/core/SkFont.cpp:378-385
impl crate::src::core::SkFontPriv::SkFontPriv {
    pub fn ApproximateTransformedTextSize(
        font: &SkFont,
        matrix: &crate::src::core::SkMatrix::SkMatrix,
        text_location: &crate::src::core::SkPoint::SkPoint,
    ) -> f32 {
        if !matrix.hasPerspective() {
            font.getSize() * matrix.getMaxScale()
        } else {
            let max_scale_sq = crate::src::core::SkMatrixPriv::SkMatrixPriv::DifferentialAreaScale(
                matrix,
                text_location,
            );
            if crate::src::core::SkFloatingPoint::SkIsFinite(&[max_scale_sq])
                && !crate::include::core::SkScalar::SkScalarNearlyZero(
                    max_scale_sq,
                    crate::include::core::SkScalar::SK_ScalarNearlyZero,
                )
            {
                font.getSize() * max_scale_sq.sqrt()
            } else {
                -font.getSize()
            }
        }
    }
}
impl SkFontPriv {
    // Source: src/core/SkFont.cpp:387-400
    pub unsafe fn CountTextElements(
        text: *const std::ffi::c_void,
        byte_length: usize,
        encoding: crate::include::core::SkFontTypes::SkTextEncoding,
    ) -> usize {
        use crate::include::core::SkFontTypes::SkTextEncoding;
        use crate::src::core::SkUTF;
        match encoding {
            SkTextEncoding::kUTF8 => SkUTF::CountUTF8(text.cast(), byte_length) as usize,
            SkTextEncoding::kUTF16 => SkUTF::CountUTF16(text.cast(), byte_length) as usize,
            SkTextEncoding::kUTF32 => byte_length >> 2,
            SkTextEncoding::kGlyphID => byte_length >> 1,
            _ => {
                crate::SkASSERT!(@source b"src/core/SkFont.cpp\0".as_ptr(),398; @condition b"false\0".as_ptr(); false);
                0
            }
        }
    }
}
// Source: src/core/SkFont.cpp:402-406
// Source: src/core/SkFont.cpp:408-411
// Source: src/core/SkFont.cpp:413-417
impl crate::src::core::SkFontPriv::SkFontPriv {
    pub fn GlyphsToUnichars(font: &SkFont, glyphs: &[u16], count: i32, text: &mut [i32]) {
        if count <= 0 {
            return;
        }
        let typeface = font.getTypeface();
        let num_glyphs_in_typeface = typeface.countGlyphs() as u32;
        // Rust initializes POD storage; the original virtual contract still requires every
        // glyph entry to be filled by the typeface. This allocation is not a mapping fallback.
        let mut unichars = vec![0; num_glyphs_in_typeface as usize];
        typeface.virtuals.getGlyphToUnicodeMap(&mut unichars);
        for i in 0..count as usize {
            let id = glyphs[i] as u32;
            text[i] = if id < num_glyphs_in_typeface {
                unichars[id as usize]
            } else {
                0xfffd
            };
        }
    }
}

// Source: include/core/SkFont.h:372
// Source: src/core/SkFont.cpp:240-243
impl SkFont {
    /// Safety: font's entire retained typeface/effect/backend graph obeys the native
    /// strike-cache transfer and synchronous execution contract for the returned
    /// handle's complete lifetime, including all copies and thread transfers.
    pub unsafe fn makeStrikeRef(
        &self,
    ) -> Result<super::SkStrikeRef::SkStrikeRef, crate::RasterError> {
        let (spec, scale) = super::SkStrikeSpec::SkStrikeSpec::MakeCanonicalized(self, None)?;
        Ok(super::SkStrikeRef::SkStrikeRef::from_strike(
            Some(spec.findOrCreateStrike()),
            scale,
        ))
    }
}
