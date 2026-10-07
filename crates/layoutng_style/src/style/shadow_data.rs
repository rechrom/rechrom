use foundation::{gfx, Visitor};

use crate::css::style_color::StyleColor;

impl foundation::Traceable for ShadowData {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        ShadowData::Trace(self, visitor);
    }
}

// cpp: layoutng_style/style/shadow_data.h:37
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadowStyle {
    kNormal,
    kInset,
}

// cpp: layoutng_style/style/shadow_data.h:41-109
#[derive(Clone, PartialEq)]
pub struct ShadowData {
    offset_: gfx::Vector2dF,
    blur_: gfx::SizeF,
    spread_: f32,
    color_: StyleColor,
    style_: ShadowStyle,
    opacity_: f32,
}

#[allow(non_snake_case)]
impl ShadowData {
    // cpp: layoutng_style/style/shadow_data.h:45-56
    pub fn new(
        offset: gfx::Vector2dF,
        blur: f32,
        spread: f32,
        style: ShadowStyle,
        color: StyleColor,
        opacity: f32,
    ) -> Self {
        Self {
            offset_: offset,
            blur_: gfx::SizeF::new(blur, blur),
            spread_: spread,
            color_: color,
            style_: style,
            opacity_: opacity,
        }
    }

    // cpp: layoutng_style/style/shadow_data.h:58-69
    pub fn new_with_blur_xy(
        offset: gfx::Vector2dF,
        blur: gfx::SizeF,
        spread: f32,
        style: ShadowStyle,
        color: StyleColor,
        opacity: f32,
    ) -> Self {
        Self {
            offset_: offset,
            blur_: blur,
            spread_: spread,
            color_: color,
            style_: style,
            opacity_: opacity,
        }
    }

    // cpp: layoutng_style/style/shadow_data.h:71
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.color_);
    }

    // cpp: layoutng_style/style/shadow_data.h:75
    // No definition is supplied in this package.
    pub fn NeutralValue() -> Self {
        unsafe { ShadowDataNeutralValue() }
    }

    // cpp: layoutng_style/style/shadow_data.h:76-82
    pub fn BlurRadiusToStdDev(radius: f32) -> f32 {
        debug_assert!(radius >= 0.0);
        radius * 0.5
    }

    // cpp: layoutng_style/style/shadow_data.h:84-96
    pub fn X(&self) -> f32 {
        self.offset_.x()
    }
    pub fn Y(&self) -> f32 {
        self.offset_.y()
    }
    pub fn Offset(&self) -> gfx::Vector2dF {
        self.offset_
    }
    pub fn BlurRadius(&self) -> f32 {
        self.BlurValue()
    }
    pub fn BlurAsSigma(&self) -> f32 {
        Self::BlurRadiusToStdDev(self.BlurValue())
    }
    pub fn BlurValue(&self) -> f32 {
        self.blur_.width()
    }
    pub fn BlurValueXY(&self) -> gfx::SizeF {
        self.blur_
    }
    pub fn Spread(&self) -> f32 {
        self.spread_
    }
    pub fn Style(&self) -> ShadowStyle {
        self.style_
    }
    pub fn GetColor(&self) -> &StyleColor {
        &self.color_
    }
    pub fn Opacity(&self) -> f32 {
        self.opacity_
    }

    // cpp: layoutng_style/style/shadow_data.h:98-100
    // No definition is supplied in this package.
    pub fn RectOutsets(&self) -> gfx::OutsetsF {
        unsafe { ShadowDataRectOutsets(self) }
    }
}

unsafe extern "Rust" {
    fn ShadowDataNeutralValue() -> ShadowData;
    fn ShadowDataRectOutsets(value: &ShadowData) -> gfx::OutsetsF;
}
