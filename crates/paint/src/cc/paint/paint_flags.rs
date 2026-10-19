//! Canonical cc PaintFlags state and decisions. External native objects are
//! reference-counted values, never services pointing back at a layer or tree.
#![allow(non_snake_case, non_camel_case_types)]
use skia::{
    include::core::{
        SkColor::SkColor4f,
        SkSamplingOptions::{SkCubicResampler, SkFilterMode, SkMipmapMode, SkSamplingOptions},
    },
    BlendMode,
};
use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};
// Source: cc/paint/paint_flags.h:35-39,75-95,159-183.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    kFill_Style,
    kStroke_Style,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cap {
    kButt_Cap,
    kRound_Cap,
    kSquare_Cap,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Join {
    kMiter_Join,
    kRound_Join,
    kBevel_Join,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FilterQuality {
    #[default]
    kNone,
    kLow,
    kMedium,
    kHigh,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScalingOperation {
    kDefault,
    kUnknown,
    kUpscale,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DynamicRangeLimit {
    kStandard,
    kHigh,
    kConstrainedHigh,
}
// Source: cc/paint/paint_flags.h:96-133.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DynamicRangeLimitMixture {
    pub standard_mix: f32,
    pub constrained_high_mix: f32,
}
impl DynamicRangeLimitMixture {
    pub fn FromLimit(limit: DynamicRangeLimit) -> Self {
        let mut result = Self::default();
        match limit {
            DynamicRangeLimit::kStandard => result.standard_mix = 1.,
            DynamicRangeLimit::kConstrainedHigh => result.constrained_high_mix = 1.,
            DynamicRangeLimit::kHigh => {}
        }
        result
    }
    pub fn new(standard_mix: f32, constrained_high_mix: f32) -> Self {
        Self {
            standard_mix,
            constrained_high_mix,
        }
    }
    // Source: cc/paint/paint_flags.cc:255-263.
    pub fn ComputeEffectiveHdrHeadroom(&self, target: f32) -> f32 {
        let high = 1. - self.constrained_high_mix - self.standard_mix;
        self.constrained_high_mix * if target < 1. { target } else { 1. } + high * target
    }
    // Source: cc/paint/paint_flags.cc:265-267.
    pub fn ToString(&self) -> String {
        format!("{:.6}, {:.6}", self.standard_mix, self.constrained_high_mix).to_lowercase()
    }
}
// Source: cc/paint/paint_flags.h:185-220. Bit fields are represented by one
// actual u32, including native truncation and equality of the complete word.
#[derive(Clone, Copy, Debug)]
pub struct CorePaintFlags {
    color_: SkColor4f,
    width_: f32,
    miter_limit_: f32,
    bitfields_uint_: u32,
}
impl Default for CorePaintFlags {
    // Source: cc/paint/paint_flags.cc:35-50; paint_flags.h:193-195.
    fn default() -> Self {
        let mut value = Self {
            color_: SkColor4f::new(0., 0., 0., 1.),
            width_: 0.,
            miter_limit_: 4.,
            bitfields_uint_: 0,
        };
        value.setStrokeCap(Cap::kButt_Cap);
        value.setStrokeJoin(Join::kMiter_Join);
        value.setStyle(Style::kFill_Style);
        value.setBlendMode(BlendMode::kSrcOver);
        value.setFilterQuality(FilterQuality::kNone);
        value
    }
}
impl PartialEq for CorePaintFlags {
    // Source: cc/paint/paint_flags.cc:52-56.
    fn eq(&self, other: &Self) -> bool {
        self.color_ == other.color_
            && self.width_ == other.width_
            && self.miter_limit_ == other.miter_limit_
            && self.bitfields_uint_ == other.bitfields_uint_
    }
}
impl CorePaintFlags {
    fn bits(&self, shift: u32, width: u32) -> u32 {
        (self.bitfields_uint_ >> shift) & ((1 << width) - 1)
    }
    fn set_bits(&mut self, shift: u32, width: u32, value: u32) {
        let mask = (1 << width) - 1;
        self.bitfields_uint_ = (self.bitfields_uint_ & !(mask << shift)) | ((value & mask) << shift)
    }
    // Source: cc/paint/paint_flags.h:40-73.
    pub fn getStyle(&self) -> Style {
        match self.bits(6, 2) {
            0 => Style::kFill_Style,
            1 => Style::kStroke_Style,
            _ => unreachable!("native validated style"),
        }
    }
    pub fn setStyle(&mut self, value: Style) {
        self.set_bits(6, 2, value as u32)
    }
    pub fn getColor(&self) -> u32 {
        self.color_.toSkColor()
    }
    pub fn getColor4f(&self) -> &SkColor4f {
        &self.color_
    }
    pub fn setColor(&mut self, value: SkColor4f) {
        self.color_ = value
    }
    pub fn setPackedColor(&mut self, value: u32) {
        self.color_ = SkColor4f::FromColor(value)
    }
    pub fn getAlphaf(&self) -> f32 {
        self.color_.fA
    }
    pub fn isFullyTransparent(&self) -> bool {
        self.color_.fA == 0.
    }
    pub fn isOpaque(&self) -> bool {
        self.color_.fA >= 1.
    }
    pub fn setAlphaf(&mut self, value: f32) {
        self.color_.fA = value
    }
    pub fn setBlendMode(&mut self, value: BlendMode) {
        self.set_bits(8, 5, value as u32)
    }
    pub fn getBlendMode(&self) -> BlendMode {
        BLEND_MODES[self.bits(8, 5) as usize]
    }
    pub fn isAntiAlias(&self) -> bool {
        self.bits(0, 1) != 0
    }
    pub fn setAntiAlias(&mut self, v: bool) {
        self.set_bits(0, 1, v as u32)
    }
    pub fn isDither(&self) -> bool {
        self.bits(1, 1) != 0
    }
    pub fn setDither(&mut self, v: bool) {
        self.set_bits(1, 1, v as u32)
    }
    pub fn setArcClosed(&mut self, v: bool) {
        self.set_bits(30, 1, v as u32)
    }
    pub fn isArcClosed(&self) -> bool {
        self.bits(30, 1) != 0
    }
    // Source: cc/paint/paint_flags.h:96-158.
    pub fn setFilterQuality(&mut self, v: FilterQuality) {
        self.set_bits(13, 2, v as u32)
    }
    pub fn getFilterQuality(&self) -> FilterQuality {
        [
            FilterQuality::kNone,
            FilterQuality::kLow,
            FilterQuality::kMedium,
            FilterQuality::kHigh,
        ][self.bits(13, 2) as usize]
    }
    pub fn setDynamicRangeLimit(&mut self, v: DynamicRangeLimitMixture) {
        self.set_bits(15, 7, (0.5 + 127. * v.standard_mix) as u32);
        self.set_bits(22, 7, (0.5 + 127. * v.constrained_high_mix) as u32)
    }
    pub fn getDynamicRangeLimit(&self) -> DynamicRangeLimitMixture {
        DynamicRangeLimitMixture::new(
            (1. / 127.) * self.bits(15, 7) as f32,
            (1. / 127.) * self.bits(22, 7) as f32,
        )
    }
    pub fn useDarkModeForImage(&self) -> bool {
        self.bits(29, 1) != 0
    }
    pub fn setUseDarkModeForImage(&mut self, v: bool) {
        self.set_bits(29, 1, v as u32)
    }
    pub fn getStrokeWidth(&self) -> f32 {
        self.width_
    }
    pub fn setStrokeWidth(&mut self, v: f32) {
        self.width_ = v
    }
    pub fn getStrokeMiter(&self) -> f32 {
        self.miter_limit_
    }
    pub fn setStrokeMiter(&mut self, v: f32) {
        self.miter_limit_ = v
    }
    // Source: cc/paint/paint_flags.h:159-183.
    pub fn getStrokeCap(&self) -> Cap {
        [Cap::kButt_Cap, Cap::kRound_Cap, Cap::kSquare_Cap][self.bits(2, 2) as usize]
    }
    pub fn setStrokeCap(&mut self, v: Cap) {
        self.set_bits(2, 2, v as u32)
    }
    pub fn getStrokeJoin(&self) -> Join {
        [Join::kMiter_Join, Join::kRound_Join, Join::kBevel_Join][self.bits(4, 2) as usize]
    }
    pub fn setStrokeJoin(&mut self, v: Join) {
        self.set_bits(4, 2, v as u32)
    }
    // Source: cc/paint/paint_flags.cc:209-211; paint_op.h:216-219.
    pub fn IsValid(&self) -> bool {
        self.bits(8, 5) < BLEND_MODES.len() as u32
    }
}
const BLEND_MODES: [BlendMode; 29] = [
    BlendMode::kClear,
    BlendMode::kSrc,
    BlendMode::kDst,
    BlendMode::kSrcOver,
    BlendMode::kDstOver,
    BlendMode::kSrcIn,
    BlendMode::kDstIn,
    BlendMode::kSrcOut,
    BlendMode::kDstOut,
    BlendMode::kSrcATop,
    BlendMode::kDstATop,
    BlendMode::kXor,
    BlendMode::kPlus,
    BlendMode::kModulate,
    BlendMode::kScreen,
    BlendMode::kOverlay,
    BlendMode::kDarken,
    BlendMode::kLighten,
    BlendMode::kColorDodge,
    BlendMode::kColorBurn,
    BlendMode::kHardLight,
    BlendMode::kSoftLight,
    BlendMode::kDifference,
    BlendMode::kExclusion,
    BlendMode::kMultiply,
    BlendMode::kHue,
    BlendMode::kSaturation,
    BlendMode::kColor,
    BlendMode::kLuminosity,
];

pub trait PaintValueEquals {
    fn EqualsForTesting(&self, other: &Self) -> bool;
}
pub trait PaintShaderValue<C>: PaintValueEquals {
    fn IsOpaque(&self) -> bool;
    fn HasDiscardableImages(&self, usage: Option<&mut C>) -> bool;
}
pub trait PaintFilterValue<C>: PaintValueEquals {
    fn has_discardable_images(&self) -> bool;
    fn GetContentColorUsage(&self) -> C;
}
pub trait PaintFlagsTypes {
    type ContentColorUsage: Copy + Ord;
    type Shader: PaintShaderValue<Self::ContentColorUsage>;
    type ColorFilter: PaintValueEquals;
    type PathEffect: PaintValueEquals;
    type DrawLooper: PaintValueEquals;
    type ImageFilter: PaintFilterValue<Self::ContentColorUsage>;
}
// Native Skia values and operations only. No classification, buffer traversal,
// Paint policy, owner lookup or layer/tree capability is admitted here.
pub trait NativePaintValue<B: PaintFlagsTypes + ?Sized> {
    fn setPathEffect(&mut self, value: &B::PathEffect);
    fn setShader(&mut self, value: &B::Shader, quality: FilterQuality);
    fn setColorFilter(&mut self, value: &B::ColorFilter);
    fn setImageFilter(&mut self, value: &B::ImageFilter);
    fn setColor(&mut self, value: SkColor4f);
    fn setStrokeWidth(&mut self, value: f32);
    fn setStrokeMiter(&mut self, value: f32);
    fn setBlendMode(&mut self, value: BlendMode);
    fn setAntiAlias(&mut self, value: bool);
    fn setDither(&mut self, value: bool);
    fn setStrokeCap(&mut self, value: Cap);
    fn setStrokeJoin(&mut self, value: Join);
    fn setStyle(&mut self, value: Style);
}
pub trait PaintFlagsRasterTypes: PaintFlagsTypes + Sized {
    type NativePaint: NativePaintValue<Self> + Default;
    type Canvas;
    type Path;
    type Rect;
    fn FillPathWithPaint(
        src: &Self::Path,
        paint: Self::NativePaint,
        dst: &mut Self::Path,
        cull: Option<&Self::Rect>,
        scale: f32,
    ) -> bool;
    fn ApplyLooper(
        looper: &Self::DrawLooper,
        canvas: &mut Self::Canvas,
        paint: Self::NativePaint,
        proc: &mut dyn FnMut(&mut Self::Canvas, Self::NativePaint),
    );
}
// Source: cc/paint/paint_flags.h:222-348.
pub struct PaintFlags<B: PaintFlagsTypes> {
    core: CorePaintFlags,
    targeted_hdr_headroom_: f32,
    path_effect_: Option<Arc<B::PathEffect>>,
    shader_: Option<Arc<B::Shader>>,
    color_filter_: Option<Arc<B::ColorFilter>>,
    draw_looper_: Option<Arc<B::DrawLooper>>,
    image_filter_: Option<Arc<B::ImageFilter>>,
}
pub struct TargetedHdrHeadroom;
impl TargetedHdrHeadroom {
    pub const FROM_PLAYBACK_PARAMS: f32 = -1.;
    pub const DISABLE_EVERYTHING: f32 = -2.;
}
impl<B: PaintFlagsTypes> Default for PaintFlags<B> {
    // Source: cc/paint/paint_flags.cc:58; paint_flags.h:337-343.
    fn default() -> Self {
        Self {
            core: CorePaintFlags::default(),
            targeted_hdr_headroom_: -1.,
            path_effect_: None,
            shader_: None,
            color_filter_: None,
            draw_looper_: None,
            image_filter_: None,
        }
    }
}
impl<B: PaintFlagsTypes> Clone for PaintFlags<B> {
    // Source: cc/paint/paint_flags.cc:60-70. Arc moves/clones preserve native
    // sk_sp ownership; normal Rust field drop matches cc:72.
    fn clone(&self) -> Self {
        Self {
            core: self.core,
            targeted_hdr_headroom_: self.targeted_hdr_headroom_,
            path_effect_: self.path_effect_.clone(),
            shader_: self.shader_.clone(),
            color_filter_: self.color_filter_.clone(),
            draw_looper_: self.draw_looper_.clone(),
            image_filter_: self.image_filter_.clone(),
        }
    }
}
impl<B: PaintFlagsTypes> Deref for PaintFlags<B> {
    type Target = CorePaintFlags;
    fn deref(&self) -> &CorePaintFlags {
        &self.core
    }
}
impl<B: PaintFlagsTypes> DerefMut for PaintFlags<B> {
    fn deref_mut(&mut self) -> &mut CorePaintFlags {
        &mut self.core
    }
}
impl<B: PaintFlagsTypes> PaintFlags<B> {
    pub fn FromCore(core: CorePaintFlags) -> Self {
        Self {
            core,
            ..Self::default()
        }
    }
    // Source: cc/paint/paint_flags.cc:74-83.
    pub fn CanConvertToCorePaintFlags(&self) -> bool {
        self.IsValid()
            && self.path_effect_.is_none()
            && self.shader_.is_none()
            && self.color_filter_.is_none()
            && self.draw_looper_.is_none()
            && self.image_filter_.is_none()
    }
    pub fn ToCorePaintFlags(&self) -> CorePaintFlags {
        debug_assert!(self.CanConvertToCorePaintFlags());
        self.core
    }
    // Source: cc/paint/paint_flags.h:245-296; cc:85-91.
    pub fn getColorFilter(&self) -> &Option<Arc<B::ColorFilter>> {
        &self.color_filter_
    }
    pub fn setColorFilter(&mut self, v: Option<Arc<B::ColorFilter>>) {
        self.color_filter_ = v
    }
    pub fn getShader(&self) -> Option<&B::Shader> {
        self.shader_.as_deref()
    }
    pub fn HasShader(&self) -> bool {
        self.shader_.is_some()
    }
    pub fn ShaderIsOpaque(&self) -> bool {
        self.shader_
            .as_ref()
            .expect("source HasShader precondition")
            .IsOpaque()
    }
    pub fn setShader(&mut self, v: Option<Arc<B::Shader>>) {
        self.shader_ = v
    }
    pub fn getPathEffect(&self) -> &Option<Arc<B::PathEffect>> {
        &self.path_effect_
    }
    pub fn setPathEffect(&mut self, v: Option<Arc<B::PathEffect>>) {
        self.path_effect_ = v
    }
    pub fn getImageFilter(&self) -> &Option<Arc<B::ImageFilter>> {
        &self.image_filter_
    }
    pub fn setImageFilter(&mut self, v: Option<Arc<B::ImageFilter>>) {
        self.image_filter_ = v
    }
    pub fn getLooper(&self) -> &Option<Arc<B::DrawLooper>> {
        &self.draw_looper_
    }
    pub fn setLooper(&mut self, v: Option<Arc<B::DrawLooper>>) {
        self.draw_looper_ = v
    }
    pub fn setTargetedHdrHeadroom(&mut self, v: f32) {
        self.targeted_hdr_headroom_ = v
    }
    pub fn getTargetedHdrHeadroom(&self) -> f32 {
        self.targeted_hdr_headroom_
    }
    // Source: cc/paint/paint_flags.cc:93-115.
    pub fn nothingToDraw(&self) -> bool {
        if self.getLooper().is_some() {
            return false;
        }
        match self.getBlendMode() {
            BlendMode::kSrcOver
            | BlendMode::kSrcATop
            | BlendMode::kDstOut
            | BlendMode::kDstOver
            | BlendMode::kPlus => {
                if self.isFullyTransparent() {
                    return self.color_filter_.is_none() && self.image_filter_.is_none();
                }
            }
            BlendMode::kDst => return true,
            _ => {}
        }
        false
    }
    // Source: cc/paint/paint_flags.cc:130-145.
    pub fn SupportsFoldingAlpha(&self) -> bool {
        if self.getBlendMode() != BlendMode::kSrcOver {
            return false;
        }
        if self.getColorFilter().is_some() {
            return false;
        }
        if self.getImageFilter().is_some() {
            return false;
        }
        if self.getLooper().is_some() {
            return false;
        }
        true
    }
    // Source: cc/paint/paint_flags.cc:174-207.
    pub fn FilterQualityToSkSamplingOptions(quality: FilterQuality) -> SkSamplingOptions {
        Self::FilterQualityToSkSamplingOptionsForScaling(quality, ScalingOperation::kDefault)
    }
    pub fn FilterQualityToSkSamplingOptionsForScaling(
        quality: FilterQuality,
        scaling: ScalingOperation,
    ) -> SkSamplingOptions {
        use FilterQuality::*;
        use ScalingOperation::*;
        match quality {
            kHigh => match scaling {
                kDefault => SkSamplingOptions::Cubic(SkCubicResampler::CatmullRom()),
                kUnknown => SkSamplingOptions::Filter(SkFilterMode::kLinear, SkMipmapMode::kLinear),
                kUpscale => SkSamplingOptions::Cubic(SkCubicResampler::Mitchell()),
            },
            kMedium => match scaling {
                kDefault => {
                    SkSamplingOptions::Filter(SkFilterMode::kLinear, SkMipmapMode::kNearest)
                }
                kUnknown | kUpscale => {
                    SkSamplingOptions::Filter(SkFilterMode::kLinear, SkMipmapMode::kLinear)
                }
            },
            kLow => SkSamplingOptions::Filter(SkFilterMode::kLinear, SkMipmapMode::kNone),
            kNone => SkSamplingOptions::Filter(SkFilterMode::kNearest, SkMipmapMode::kNone),
        }
    }
    // Source: cc/paint/paint_flags.cc:213-236.
    pub fn EqualsForTesting(&self, other: &Self) -> bool {
        self.getColor() == other.getColor()
            && self.getStrokeWidth() == other.getStrokeWidth()
            && self.getStrokeMiter() == other.getStrokeMiter()
            && self.getBlendMode() == other.getBlendMode()
            && self.getStrokeCap() == other.getStrokeCap()
            && self.getStrokeJoin() == other.getStrokeJoin()
            && self.getStyle() == other.getStyle()
            && self.getFilterQuality() == other.getFilterQuality()
            && self.getDynamicRangeLimit() == other.getDynamicRangeLimit()
            && self.isArcClosed() == other.isArcClosed()
            && values_equal(&self.path_effect_, &other.path_effect_)
            && values_equal(&self.color_filter_, &other.color_filter_)
            && values_equal(&self.draw_looper_, &other.draw_looper_)
            && values_equal(&self.image_filter_, &other.image_filter_)
            && values_equal(&self.shader_, &other.shader_)
    }
    // Source: cc/paint/paint_flags.cc:238-253.
    pub fn HasDiscardableImages(&self, mut usage: Option<&mut B::ContentColorUsage>) -> bool {
        let mut has = false;
        if let Some(shader) = &self.shader_ {
            has = shader.HasDiscardableImages(usage.as_deref_mut())
        }
        if let Some(filter) = &self.image_filter_ {
            if filter.has_discardable_images() {
                if let Some(usage) = usage {
                    *usage = (*usage).max(filter.GetContentColorUsage());
                }
                has = true
            }
        }
        has
    }
}
impl<B: PaintFlagsRasterTypes> PaintFlags<B> {
    // Source: cc/paint/paint_flags.cc:117-128. The native Skia primitive owns
    // the temporary PathBuilder and detach; no Paint decision is delegated.
    pub fn getFillPath(
        &self,
        src: &B::Path,
        dst: &mut B::Path,
        cull: Option<&B::Rect>,
        scale: f32,
    ) -> bool {
        B::FillPathWithPaint(src, self.ToSkPaint(), dst, cull, scale)
    }
    // Source: cc/paint/paint_flags.cc:147-172.
    pub fn ToSkPaint(&self) -> B::NativePaint {
        let mut paint = B::NativePaint::default();
        if let Some(v) = &self.path_effect_ {
            paint.setPathEffect(v)
        }
        if let Some(v) = &self.shader_ {
            paint.setShader(v, self.getFilterQuality())
        }
        if let Some(v) = &self.color_filter_ {
            paint.setColorFilter(v)
        }
        if let Some(v) = &self.image_filter_ {
            paint.setImageFilter(v)
        }
        paint.setColor(*self.getColor4f());
        paint.setStrokeWidth(self.getStrokeWidth());
        paint.setStrokeMiter(self.getStrokeMiter());
        paint.setBlendMode(self.getBlendMode());
        paint.setAntiAlias(self.isAntiAlias());
        paint.setDither(self.isDither());
        paint.setStrokeCap(self.getStrokeCap());
        paint.setStrokeJoin(self.getStrokeJoin());
        paint.setStyle(self.getStyle());
        paint
    }
    // Source: cc/paint/paint_flags.h:307-318.
    pub fn DrawToSk(
        &self,
        canvas: &mut B::Canvas,
        mut proc: impl FnMut(&mut B::Canvas, B::NativePaint),
    ) {
        let paint = self.ToSkPaint();
        if let Some(looper) = self.getLooper() {
            B::ApplyLooper(looper, canvas, paint, &mut proc)
        } else {
            proc(canvas, paint)
        }
    }
}
// Source: cc/paint/paint_flags.cc:23-30.
fn values_equal<T: PaintValueEquals>(a: &Option<Arc<T>>, b: &Option<Arc<T>>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => Arc::ptr_eq(a, b) || a.EqualsForTesting(b),
        _ => false,
    }
}
// Source scaffolding: cc/paint/paint_flags.h:1-34,349 guard; cc:1-34,268 namespace.
// Translation status: complete
