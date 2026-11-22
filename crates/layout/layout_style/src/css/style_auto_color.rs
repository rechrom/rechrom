use foundation::{CSSValueID, Visitor};

use super::style_color::StyleColor;

// cpp: layoutng_style/css/style_auto_color.h:13-29
#[derive(Clone)]
pub struct StyleAutoColor {
    color_: StyleColor,
}

#[allow(non_snake_case)]
impl StyleAutoColor {
    // cpp: layoutng_style/css/style_auto_color.h:17
    pub fn new(color: StyleColor) -> Self {
        Self { color_: color }
    }

    // cpp: layoutng_style/css/style_auto_color.h:19-21
    pub fn AutoColor() -> Self {
        Self::new(StyleColor::from_keyword(CSSValueID::kAuto))
    }

    // cpp: layoutng_style/css/style_auto_color.h:23
    pub fn IsAutoColor(&self) -> bool {
        self.color_.color_keyword_ == CSSValueID::kAuto
    }

    // cpp: layoutng_style/css/style_auto_color.h:25-28
    pub fn ToStyleColor(&self) -> &StyleColor {
        debug_assert!(!self.IsAutoColor());
        &self.color_
    }

    pub fn IsCurrentColor(&self) -> bool {
        self.color_.IsCurrentColor()
    }

    pub fn DependsOnCurrentColor(&self) -> bool {
        self.color_.DependsOnCurrentColor()
    }

    pub fn Trace(&self, visitor: &mut Visitor) {
        self.color_.Trace(visitor);
    }
}

// cpp: layoutng_style/css/style_auto_color.h:31-36
impl PartialEq for StyleAutoColor {
    fn eq(&self, other: &Self) -> bool {
        if self.IsAutoColor() || other.IsAutoColor() {
            return self.IsAutoColor() && other.IsAutoColor();
        }
        self.ToStyleColor() == other.ToStyleColor()
    }
}
