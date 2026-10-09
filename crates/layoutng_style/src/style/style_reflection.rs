use foundation::{Length, Visitor};

use super::nine_piece_image::NinePieceImage;
use crate::css::css_reflection_direction::CSSReflectionDirection;

// cpp: layoutng_style/style/style_reflection.h:36-59
pub struct StyleReflection {
    direction_: CSSReflectionDirection,
    offset_: Length,
    mask_: NinePieceImage,
}

// cpp: layoutng_style/style/style_reflection.h:38-39
// cpp: layoutng_style/style/style_reflection.h:56
impl Default for StyleReflection {
    fn default() -> Self {
        Self {
            direction_: CSSReflectionDirection::kReflectionBelow,
            offset_: Length::Fixed(0),
            mask_: NinePieceImage::MaskDefaults(),
        }
    }
}

#[allow(non_snake_case)]
impl StyleReflection {
    // cpp: layoutng_style/style/style_reflection.h:46
    pub fn Direction(&self) -> CSSReflectionDirection {
        self.direction_
    }

    // cpp: layoutng_style/style/style_reflection.h:47
    pub fn Offset(&self) -> &Length {
        &self.offset_
    }

    // cpp: layoutng_style/style/style_reflection.h:48
    pub fn Mask(&self) -> &NinePieceImage {
        &self.mask_
    }

    // cpp: layoutng_style/style/style_reflection.h:50
    pub fn SetDirection(&mut self, dir: CSSReflectionDirection) {
        self.direction_ = dir;
    }

    // cpp: layoutng_style/style/style_reflection.h:51
    pub fn SetOffset(&mut self, length: &Length) {
        self.offset_ = length.clone();
    }

    // cpp: layoutng_style/style/style_reflection.h:52
    pub fn SetMask(&mut self, image: &NinePieceImage) {
        self.mask_ = image.clone();
    }

    // cpp: layoutng_style/style/style_reflection.h:53
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.mask_);
    }
}

// cpp: layoutng_style/style/style_reflection.h:41-44
impl PartialEq for StyleReflection {
    fn eq(&self, other: &Self) -> bool {
        self.direction_ == other.direction_
            && self.offset_ == other.offset_
            && self.mask_ == other.mask_
    }
}

// cpp: style_reflection.h:53; GC traces the native NinePieceImage mask.
impl foundation::Traceable for StyleReflection {
    fn Trace(&self, visitor: &mut Visitor) { StyleReflection::Trace(self,visitor); }
}
