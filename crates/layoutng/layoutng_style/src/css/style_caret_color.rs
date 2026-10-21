use foundation::Visitor;

use super::style_auto_color::StyleAutoColor;
use super::style_color::StyleColor;

// cpp: layoutng_style/css/style_caret_color.h:18-50
#[derive(Clone)]
pub struct StyleCaretColor {
    fill_color_: StyleAutoColor,
    text_color_: StyleAutoColor,
}

// cpp: layoutng_style/css/style_caret_color.h:22-24
impl Default for StyleCaretColor {
    fn default() -> Self {
        Self {
            fill_color_: StyleAutoColor::AutoColor(),
            text_color_: StyleAutoColor::AutoColor(),
        }
    }
}

#[allow(non_snake_case)]
impl StyleCaretColor {
    // cpp: layoutng_style/css/style_caret_color.h:26-28
    pub fn new(fill_color: StyleAutoColor, text_color: StyleAutoColor) -> Self {
        Self {
            fill_color_: fill_color,
            text_color_: text_color,
        }
    }

    // cpp: layoutng_style/css/style_caret_color.h:30-33
    pub fn IsAutoColor(&self) -> bool {
        self.fill_color_.IsAutoColor()
    }
    pub fn IsCurrentColor(&self) -> bool {
        self.fill_color_.IsCurrentColor()
    }
    pub fn ToStyleColor(&self) -> &StyleColor {
        self.fill_color_.ToStyleColor()
    }

    // cpp: layoutng_style/css/style_caret_color.h:34-37
    pub fn DependsOnCurrentColor(&self) -> bool {
        self.fill_color_.DependsOnCurrentColor() || self.text_color_.DependsOnCurrentColor()
    }

    // cpp: layoutng_style/css/style_caret_color.h:39-40
    pub fn FillColor(&self) -> &StyleAutoColor {
        &self.fill_color_
    }
    pub fn TextColor(&self) -> &StyleAutoColor {
        &self.text_color_
    }

    // cpp: layoutng_style/css/style_caret_color.h:42-45
    pub fn Trace(&self, visitor: &mut Visitor) {
        self.fill_color_.Trace(visitor);
        self.text_color_.Trace(visitor);
    }
}

// cpp: layoutng_style/css/style_caret_color.h:52-54
impl PartialEq for StyleCaretColor {
    fn eq(&self, other: &Self) -> bool {
        self.FillColor() == other.FillColor() && self.TextColor() == other.TextColor()
    }
}
