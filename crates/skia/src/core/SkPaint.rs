// Source: include/core/SkPaint.h:162-164
impl SkPaint {
    pub fn isAntiAlias(&self) -> bool {
        self.fBitfieldsUInt & 1 != 0
    }
}
// Source: include/core/SkPaint.h:170-170
impl SkPaint {
    pub fn setAntiAlias(&mut self, aa: bool) {
        self.fBitfieldsUInt = (self.fBitfieldsUInt & !1) | u32::from(aa);
    }
}
// Source: include/core/SkPaint.h:175-177
impl SkPaint {
    pub fn isDither(&self) -> bool {
        self.fBitfieldsUInt & 2 != 0
    }
}
// Source: include/core/SkPaint.h:182-182
impl SkPaint {
    pub fn setDither(&mut self, dither: bool) {
        self.fBitfieldsUInt = (self.fBitfieldsUInt & !2) | (u32::from(dither) << 1);
    }
}
// Source: include/core/SkPaint.h:192-200
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Style {
    kFill_Style = 0,
    kStroke_Style = 1,
    kStrokeAndFill_Style = 2,
}
pub const kStyleCount: i32 = Style::kStrokeAndFill_Style as i32 + 1;
pub use Style::{kFill_Style, kStrokeAndFill_Style, kStroke_Style};
// Source: include/core/SkPaint.h:204-204
impl SkPaint {
    pub fn getStyle(&self) -> Style {
        match (self.fBitfieldsUInt >> 6) & 3 {
            0 => Style::kFill_Style,
            1 => Style::kStroke_Style,
            2 => Style::kStrokeAndFill_Style,
            _ => unreachable!("invalid private paint style"),
        }
    }
}
// Source: include/core/SkPaint.h:225-225
impl SkPaint {
    pub fn getColor(&self) -> SkColor {
        self.fColor4f.toSkColor()
    }
}
// Source: include/core/SkPaint.h:232-232
impl SkPaint {
    pub fn getColor4f(&self) -> SkColor4f {
        self.fColor4f
    }
}
// Source: include/core/SkPaint.h:261-261
impl SkPaint {
    pub fn getAlphaf(&self) -> f32 {
        self.fColor4f.fA
    }
}
// Source: include/core/SkPaint.h:264-266
impl SkPaint {
    pub fn getAlpha(&self) -> u8 {
        crate::src::core::SkFloatingPoint::sk_float_round2int(self.getAlphaf() * 255.) as u8
    }
}
// Source: include/core/SkPaint.h:279-281
impl SkPaint {
    pub fn setAlpha(&mut self, a: u32) {
        self.setAlphaf(a as f32 * (1. / 255.));
    }
}
// Source: include/core/SkPaint.h:300-300
impl SkPaint {
    pub fn getStrokeWidth(&self) -> f32 {
        self.fWidth
    }
}
// Source: include/core/SkPaint.h:318-318
impl SkPaint {
    pub fn getStrokeMiter(&self) -> f32 {
        self.fMiterLimit
    }
}
// Source: include/core/SkPaint.h:336-346
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Cap {
    kButt_Cap = 0,
    kRound_Cap = 1,
    kSquare_Cap = 2,
}
impl Cap {
    pub const kLast_Cap: Self = Self::kSquare_Cap;
    pub const kDefault_Cap: Self = Self::kButt_Cap;
}
pub const kCapCount: i32 = Cap::kLast_Cap as i32 + 1;
pub const kDefault_Cap: Cap = Cap::kDefault_Cap;
pub const kLast_Cap: Cap = Cap::kLast_Cap;
pub use Cap::{kButt_Cap, kRound_Cap, kSquare_Cap};
// Source: include/core/SkPaint.h:361-371
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Join {
    kMiter_Join = 0,
    kRound_Join = 1,
    kBevel_Join = 2,
}
impl Join {
    pub const kLast_Join: Self = Self::kBevel_Join;
    pub const kDefault_Join: Self = Self::kMiter_Join;
}
pub const kJoinCount: i32 = Join::kLast_Join as i32 + 1;
pub const kDefault_Join: Join = Join::kDefault_Join;
pub const kLast_Join: Join = Join::kLast_Join;
pub use Join::{kBevel_Join, kMiter_Join, kRound_Join};

use crate::include::core::SkBlender::SkBlender;
use crate::include::core::SkRefCnt::sk_sp;
use crate::src::core::SkColor::{SkColor, SkColor4f};
use crate::src::core::SkColorFilter::SkColorFilter;
use crate::src::core::SkImageFilter::SkImageFilter;
use crate::src::core::SkMaskFilter::SkMaskFilter;
use crate::src::core::SkPathEffect::SkPathEffect;
use crate::src::shaders::SkShader::SkShader;
use std::sync::Arc;

// Source: include/core/SkPaint.h:375-375
impl SkPaint {
    pub fn getStrokeCap(&self) -> Cap {
        match (self.fBitfieldsUInt >> 2) & 3 {
            0 => Cap::kButt_Cap,
            1 => Cap::kRound_Cap,
            2 => Cap::kSquare_Cap,
            _ => unreachable!("invalid private paint cap"),
        }
    }
}
// Source: include/core/SkPaint.h:386-386
impl SkPaint {
    pub fn getStrokeJoin(&self) -> Join {
        match (self.fBitfieldsUInt >> 4) & 3 {
            0 => Join::kMiter_Join,
            1 => Join::kRound_Join,
            2 => Join::kBevel_Join,
            _ => unreachable!("invalid private paint join"),
        }
    }
}
// Source: include/core/SkPaint.h:400-400
impl SkPaint {
    pub fn getShader(&self) -> Option<&dyn SkShader> {
        self.fShader.as_deref()
    }
}
// Source: include/core/SkPaint.h:429-429
impl SkPaint {
    pub fn getColorFilter(&self) -> Option<&dyn SkColorFilter> {
        self.fColorFilter.as_deref()
    }
}
// Source: include/core/SkPaint.h:483-483
impl SkPaint {
    pub fn getBlender(&self) -> Option<&dyn SkBlender> {
        self.fBlender.get()
    }
}
// Source: include/core/SkPaint.h:509-509
impl SkPaint {
    pub fn getPathEffect(&self) -> Option<&dyn SkPathEffect> {
        self.fPathEffect.get()
    }
}
// Source: include/core/SkPaint.h:537-537
impl SkPaint {
    pub fn getMaskFilter(&self) -> Option<&dyn SkMaskFilter> {
        self.fMaskFilter.as_deref()
    }
}
// Source: include/core/SkPaint.h:567-567
impl SkPaint {
    pub fn getImageFilter(&self) -> Option<&dyn SkImageFilter> {
        self.fImageFilter.as_deref()
    }
    // Rust ownership-token projection of the same native borrowed pointer.
    // Source: include/core/SkPaint.h:567-567
    pub(crate) fn imageFilterOwner(&self) -> Option<&Arc<dyn SkImageFilter>> {
        self.fImageFilter.as_ref()
    }
}

// Arc pointer identity mirrors the upstream owning pointer comparison.
fn arc_same<T: ?Sized>(a: &Option<Arc<T>>, b: &Option<Arc<T>>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => Arc::ptr_eq(a, b),
        _ => false,
    }
}
// Source: include/core/SkPaint.h:643-646
impl SkPaint {
    pub fn computeFastStrokeBounds<'a>(
        &self,
        orig: &crate::src::core::SkRect::SkRect,
        storage: &'a mut crate::src::core::SkRect::SkRect,
    ) -> Result<&'a crate::src::core::SkRect::SkRect, crate::RasterError> {
        self.doComputeFastBounds(orig, storage, Style::kStroke_Style)
    }
    // Rust alias-safe overload for the source's explicitly permitted orig==storage.
    // Copy the input before taking the destination's exclusive borrow.
    pub fn computeFastBoundsInPlace(
        &self,
        bounds: &mut crate::src::core::SkRect::SkRect,
    ) -> Result<(), crate::RasterError> {
        let orig = *bounds;
        let result = *self.computeFastBounds(&orig, bounds)?;
        *bounds = result;
        Ok(())
    }
}
// Source: include/core/SkPaint.h:664-684
// Ownership adaptation: four canonical Arc trait owners remain pending native
// intrusive migration; these fields and related lifecycle blocks are adapted.
// ARM little-endian bitfield union occupies its original u32. All 24 padding
// bits remain zero and equality compares the entire word, as native does.
pub struct SkPaint {
    fPathEffect: sk_sp<dyn SkPathEffect>,
    fShader: Option<Arc<dyn SkShader>>,
    fMaskFilter: Option<Arc<dyn SkMaskFilter>>,
    fColorFilter: Option<Arc<dyn SkColorFilter>>,
    fImageFilter: Option<Arc<dyn SkImageFilter>>,
    fBlender: sk_sp<dyn SkBlender>,
    fColor4f: SkColor4f,
    fWidth: f32,
    fMiterLimit: f32,
    fBitfieldsUInt: u32,
}
// Source: src/core/SkPaint.cpp:38-50
impl Default for SkPaint {
    fn default() -> Self {
        Self {
            fPathEffect: sk_sp::default(),
            fShader: None,
            fMaskFilter: None,
            fColorFilter: None,
            fImageFilter: None,
            fBlender: sk_sp::default(),
            fColor4f: SkColor4f {
                fR: 0.,
                fG: 0.,
                fB: 0.,
                fA: 1.,
            },
            fWidth: 0.,
            fMiterLimit: crate::src::core::SkPaintDefaults::SkPaintDefaults_MiterLimit,
            fBitfieldsUInt: ((kDefault_Cap as u32) << 2)
                | ((kDefault_Join as u32) << 4)
                | ((Style::kFill_Style as u32) << 6),
        }
    }
}
// Source: src/core/SkPaint.cpp:52-54
impl SkPaint {
    pub fn from_color4f(
        color: &SkColor4f,
        color_space: Option<&crate::src::core::SkColorSpace::SkColorSpace>,
    ) -> Self {
        let mut paint = Self::default();
        paint.setColor4f(color, color_space);
        paint
    }
}
// Source: src/core/SkPaint.cpp:56-56
// Arc canonical owners retain their current actual ownership; native migration pending.
impl Clone for SkPaint {
    fn clone(&self) -> Self {
        Self {
            fPathEffect: self.fPathEffect.clone(),
            fShader: self.fShader.clone(),
            fMaskFilter: self.fMaskFilter.clone(),
            fColorFilter: self.fColorFilter.clone(),
            fImageFilter: self.fImageFilter.clone(),
            fBlender: self.fBlender.clone(),
            fColor4f: self.fColor4f,
            fWidth: self.fWidth,
            fMiterLimit: self.fMiterLimit,
            fBitfieldsUInt: self.fBitfieldsUInt,
        }
    }
}
// Source: src/core/SkPaint.cpp:58-58
impl SkPaint {
    // Default C++ memberwise move: scalar values remain in the original source,
    // while each existing pointer owner transfers without another retain.
    pub(crate) fn take_from(source: &mut Self) -> Self {
        Self {
            fPathEffect: core::mem::take(&mut source.fPathEffect),
            fShader: source.fShader.take(),
            fMaskFilter: source.fMaskFilter.take(),
            fColorFilter: source.fColorFilter.take(),
            fImageFilter: source.fImageFilter.take(),
            fBlender: core::mem::take(&mut source.fBlender),
            fColor4f: source.fColor4f,
            fWidth: source.fWidth,
            fMiterLimit: source.fMiterLimit,
            fBitfieldsUInt: source.fBitfieldsUInt,
        }
    }
}
// Source: src/core/SkPaint.cpp:60-60
impl Drop for SkPaint {
    fn drop(&mut self) {
        // C++ default member destruction runs in reverse declaration order.
        // Reset explicitly before Rust's subsequent forward member destruction.
        self.fBlender.reset_to_null();
        drop(self.fImageFilter.take());
        drop(self.fColorFilter.take());
        drop(self.fMaskFilter.take());
        drop(self.fShader.take());
        self.fPathEffect.reset_to_null();
    }
}
// Source: src/core/SkPaint.cpp:62-62
impl SkPaint {
    pub fn assign(&mut self, other: &Self) -> &mut Self {
        // Default C++ copy assignment retains/releases one member at a time;
        // clone-whole-then-replace would change ref/destructor ordering.
        self.fPathEffect.assign(&other.fPathEffect);
        drop(core::mem::replace(&mut self.fShader, other.fShader.clone()));
        drop(core::mem::replace(
            &mut self.fMaskFilter,
            other.fMaskFilter.clone(),
        ));
        drop(core::mem::replace(
            &mut self.fColorFilter,
            other.fColorFilter.clone(),
        ));
        drop(core::mem::replace(
            &mut self.fImageFilter,
            other.fImageFilter.clone(),
        ));
        self.fBlender.assign(&other.fBlender);
        self.fColor4f = other.fColor4f;
        self.fWidth = other.fWidth;
        self.fMiterLimit = other.fMiterLimit;
        self.fBitfieldsUInt = other.fBitfieldsUInt;
        self
    }
}
// Source: src/core/SkPaint.cpp:64-64
impl SkPaint {
    pub(crate) fn assign_move(&mut self, source: &mut Self) {
        self.fPathEffect = core::mem::take(&mut source.fPathEffect);
        self.fShader = source.fShader.take();
        self.fMaskFilter = source.fMaskFilter.take();
        self.fColorFilter = source.fColorFilter.take();
        self.fImageFilter = source.fImageFilter.take();
        self.fBlender = core::mem::take(&mut source.fBlender);
        self.fColor4f = source.fColor4f;
        self.fWidth = source.fWidth;
        self.fMiterLimit = source.fMiterLimit;
        self.fBitfieldsUInt = source.fBitfieldsUInt;
    }
}
// Source: src/core/SkPaint.cpp:66-80
impl PartialEq for SkPaint {
    fn eq(&self, other: &Self) -> bool {
        self.fPathEffect == other.fPathEffect
            && arc_same(&self.fShader, &other.fShader)
            && arc_same(&self.fMaskFilter, &other.fMaskFilter)
            && arc_same(&self.fColorFilter, &other.fColorFilter)
            && self.fBlender == other.fBlender
            && arc_same(&self.fImageFilter, &other.fImageFilter)
            && self.fColor4f == other.fColor4f
            && self.fWidth == other.fWidth
            && self.fMiterLimit == other.fMiterLimit
            && self.fBitfieldsUInt == other.fBitfieldsUInt
    }
}
// Source: src/core/SkPaint.cpp:82-90
impl SkPaint {
    pub fn refColorFilter(&self) -> Option<Arc<dyn SkColorFilter>> {
        self.fColorFilter.clone()
    }
    pub fn refBlender(&self) -> sk_sp<dyn SkBlender> {
        self.fBlender.clone()
    }
    pub fn refImageFilter(&self) -> Option<Arc<dyn SkImageFilter>> {
        self.fImageFilter.clone()
    }
    pub fn refMaskFilter(&self) -> Option<Arc<dyn SkMaskFilter>> {
        self.fMaskFilter.clone()
    }
    pub fn refPathEffect(&self) -> sk_sp<dyn SkPathEffect> {
        self.fPathEffect.clone()
    }
    pub fn refShader(&self) -> Option<Arc<dyn SkShader>> {
        self.fShader.clone()
    }
}
// Source: src/core/SkPaint.cpp:92-99
impl SkPaint {
    pub fn setColorFilter(&mut self, mut value: Option<Arc<dyn SkColorFilter>>) {
        drop(core::mem::replace(&mut self.fColorFilter, value));
    }
    pub fn setImageFilter(&mut self, mut value: Option<Arc<dyn SkImageFilter>>) {
        drop(core::mem::replace(&mut self.fImageFilter, value));
    }
    pub fn setMaskFilter(&mut self, mut value: Option<Arc<dyn SkMaskFilter>>) {
        drop(core::mem::replace(&mut self.fMaskFilter, value));
    }
    pub fn setPathEffect(&mut self, mut value: sk_sp<dyn SkPathEffect>) {
        unsafe {
            self.fPathEffect.reset(value.release());
        }
    }
    pub fn setShader(&mut self, mut value: Option<Arc<dyn SkShader>>) {
        drop(core::mem::replace(&mut self.fShader, value));
    }
}
// Source: src/core/SkPaint.cpp:103-103
impl SkPaint {
    pub fn reset(&mut self) {
        let initial = Self::default();
        self.assign(&initial);
    }
}
// Source: src/core/SkPaint.cpp:105-113
impl SkPaint {
    pub fn setStyle(&mut self, style: Style) {
        if (style as u32) < kStyleCount as u32 {
            self.fBitfieldsUInt = (self.fBitfieldsUInt & !(3 << 6)) | ((style as u32) << 6);
        }
    }
}
// Source: src/core/SkPaint.cpp:115-117
impl SkPaint {
    pub fn setStroke(&mut self, stroke: bool) {
        self.fBitfieldsUInt = (self.fBitfieldsUInt & !(3 << 6))
            | ((if stroke {
                Style::kStroke_Style
            } else {
                Style::kFill_Style
            } as u32)
                << 6);
    }
}
// Source: src/core/SkPaint.cpp:119-121
impl SkPaint {
    pub fn setColor(&mut self, color: SkColor) {
        self.fColor4f = SkColor4f::FromColor(color);
    }
}
// Source: src/core/SkPaint.cpp:123-128
// Current canonical MakeSRGB returns Arc rather than the native borrowed singleton.
impl SkPaint {
    pub fn setColor4f(
        &mut self,
        color: &SkColor4f,
        color_space: Option<&crate::src::core::SkColorSpace::SkColorSpace>,
    ) {
        use crate::include::core::SkAlphaType::SkAlphaType::kUnpremul_SkAlphaType;
        let srgb = crate::src::core::SkColorSpace::SkColorSpace::MakeSRGB();
        let steps = crate::src::core::SkColorSpaceXformSteps::SkColorSpaceXformSteps::new(
            color_space,
            kUnpremul_SkAlphaType,
            Some(&srgb),
            kUnpremul_SkAlphaType,
        );
        self.fColor4f = color.pinAlpha();
        let mut rgba = [
            self.fColor4f.fR,
            self.fColor4f.fG,
            self.fColor4f.fB,
            self.fColor4f.fA,
        ];
        steps.apply(&mut rgba);
        self.fColor4f = SkColor4f {
            fR: rgba[0],
            fG: rgba[1],
            fB: rgba[2],
            fA: rgba[3],
        };
    }
}
// Source: src/core/SkPaint.cpp:130-132
impl SkPaint {
    pub fn setAlphaf(&mut self, a: f32) {
        let inner = if 1.0 < a { 1.0 } else { a };
        self.fColor4f.fA = if 0.0 < inner { inner } else { 0.0 };
    }
}
// Source: src/core/SkPaint.cpp:134-136
impl SkPaint {
    pub fn setARGB(&mut self, a: u32, r: u32, g: u32, b: u32) {
        self.setColor(crate::src::core::SkColor::SkColorSetARGB(a, r, g, b));
    }
}
// Source: src/core/SkPaint.cpp:138-153
impl SkPaint {
    pub fn asBlendMode(&self) -> Option<crate::src::core::SkBlendMode::SkBlendMode> {
        self.fBlender.get().map_or(
            Some(crate::src::core::SkBlendMode::SkBlendMode::kSrcOver),
            |b| b.asBlendMode(),
        )
    }
    pub fn getBlendMode_or(
        &self,
        default_mode: crate::src::core::SkBlendMode::SkBlendMode,
    ) -> crate::src::core::SkBlendMode::SkBlendMode {
        self.asBlendMode().unwrap_or(default_mode)
    }
    pub fn isSrcOver(&self) -> bool {
        self.fBlender.get().is_none_or(|b| {
            b.asBlendMode() == Some(crate::src::core::SkBlendMode::SkBlendMode::kSrcOver)
        })
    }
    pub fn setBlendMode(&mut self, mode: crate::src::core::SkBlendMode::SkBlendMode) {
        self.setBlender(
            if mode == crate::src::core::SkBlendMode::SkBlendMode::kSrcOver {
                sk_sp::default()
            } else {
                <dyn SkBlender>::Mode(mode)
            },
        );
    }
}
// Source: src/core/SkPaint.cpp:155-157
impl SkPaint {
    pub fn setBlender(&mut self, mut blender: sk_sp<dyn SkBlender>) {
        unsafe {
            self.fBlender.reset(blender.release());
        }
    }
}
// Source: src/core/SkPaint.cpp:159-167
impl SkPaint {
    pub fn setStrokeWidth(&mut self, width: f32) {
        if width >= 0. {
            self.fWidth = width;
        }
    }
}
// Source: src/core/SkPaint.cpp:169-177
impl SkPaint {
    pub fn setStrokeMiter(&mut self, limit: f32) {
        if limit >= 0. {
            self.fMiterLimit = limit;
        }
    }
}
// Source: src/core/SkPaint.cpp:179-187
impl SkPaint {
    pub fn setStrokeCap(&mut self, cap: Cap) {
        if (cap as u32) < kCapCount as u32 {
            self.fBitfieldsUInt = (self.fBitfieldsUInt & !(3 << 2)) | ((cap as u32) << 2);
        }
    }
}
// Source: src/core/SkPaint.cpp:189-197
impl SkPaint {
    pub fn setStrokeJoin(&mut self, join: Join) {
        if (join as u32) < kJoinCount as u32 {
            self.fBitfieldsUInt = (self.fBitfieldsUInt & !(3 << 4)) | ((join as u32) << 4);
        }
    }
}
// Source: src/core/SkPaint.cpp:201-211
impl SkPaint {
    pub fn canComputeFastBounds(&self) -> Result<bool, crate::RasterError> {
        if let Some(filter) = self.getImageFilter() {
            if !filter.canComputeFastBounds()? {
                return Ok(false);
            }
        }
        if self
            .getPathEffect()
            .is_some_and(|effect| !effect.computeFastBounds(None))
        {
            return Ok(false);
        }
        Ok(true)
    }
}
// Source: src/core/SkPaint.cpp:213-229
impl SkPaint {
    pub fn computeFastBounds<'a>(
        &self,
        orig: &'a crate::src::core::SkRect::SkRect,
        storage: &'a mut crate::src::core::SkRect::SkRect,
    ) -> Result<&'a crate::src::core::SkRect::SkRect, crate::RasterError> {
        crate::SkASSERT!(@source b"src/core/SkPaint.cpp\0".as_ptr(), 215; @condition b"orig.isSorted()\0".as_ptr(); orig.isSorted());
        let style = self.getStyle();
        if style == Style::kFill_Style
            && self.getMaskFilter().is_none()
            && self.getPathEffect().is_none()
            && self.getImageFilter().is_none()
        {
            return Ok(orig);
        }
        self.doComputeFastBounds(orig, storage, style)
    }
}
// Source: src/core/SkPaint.cpp:231-257
impl SkPaint {
    pub fn doComputeFastBounds<'a>(
        &self,
        orig: &crate::src::core::SkRect::SkRect,
        storage: &'a mut crate::src::core::SkRect::SkRect,
        style: Style,
    ) -> Result<&'a crate::src::core::SkRect::SkRect, crate::RasterError> {
        let mut tmp;
        let src = if let Some(effect) = self.getPathEffect() {
            tmp = *orig;
            let success = effect.computeFastBounds(Some(&mut tmp));
            crate::SkAssertResult!(@source b"src/core/SkPaint.cpp\0".as_ptr(), 241; @condition b"as_PEB(this->getPathEffect())->computeFastBounds(&tmpSrc)\0".as_ptr(); success);
            &tmp
        } else {
            orig
        };
        let radius =
            crate::src::core::SkStrokeRec::SkStrokeRec::GetInflationRadiusForPaint(self, style);
        *storage = src.makeOutset(radius, radius);
        if let Some(filter) = self.getMaskFilter() {
            filter.computeFastBounds(*storage, storage)?;
        }
        if let Some(filter) = self.getImageFilter() {
            *storage = filter.computeFastBounds(storage)?;
        }
        Ok(storage)
    }
}
// Source: src/core/SkPaint.cpp:262-264
fn affects_color_alpha(filter: Option<&dyn SkColorFilter>) -> bool {
    filter.is_some_and(|f| !f.isAlphaUnchanged())
}
// Source: src/core/SkPaint.cpp:267-271
fn affects_image_alpha(filter: Option<&dyn SkImageFilter>) -> bool {
    filter.is_some()
}
// Source: src/core/SkPaint.cpp:273-294
impl SkPaint {
    pub fn nothingToDraw(&self) -> bool {
        use crate::src::core::SkBlendMode::SkBlendMode::*;
        let Some(mode) = self.asBlendMode() else {
            return false;
        };
        match mode {
            kSrcOver | kSrcATop | kDstOut | kDstOver | kPlus => {
                if self.getAlpha() == 0 {
                    return !affects_color_alpha(self.getColorFilter())
                        && !affects_image_alpha(self.getImageFilter());
                }
            }
            kDst => return true,
            _ => {}
        }
        false
    }
}
