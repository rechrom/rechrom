use foundation::Visitor;

use crate::css::style_color::StyleColor;

// cpp: layoutng_style/style/style_scrollbar_color.h:14-34
#[derive(Clone)]
pub struct StyleScrollbarColor {
    thumb_color_: StyleColor,
    track_color_: StyleColor,
}

#[allow(non_snake_case)]
impl StyleScrollbarColor {
    // cpp: layoutng_style/style/style_scrollbar_color.h:17
    // No constructor definition exists in the supplied package.
    pub fn new(thumb_color: StyleColor, track_color: StyleColor) -> Self {
        unsafe { StyleScrollbarColorConstruct(thumb_color, track_color) }
    }

    // cpp: layoutng_style/style/style_scrollbar_color.h:19-22
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.thumb_color_);
        visitor.Trace(&self.track_color_);
    }

    // cpp: layoutng_style/style/style_scrollbar_color.h:24-25
    pub fn GetThumbColor(&self) -> StyleColor {
        self.thumb_color_.clone()
    }
    pub fn GetTrackColor(&self) -> StyleColor {
        self.track_color_.clone()
    }
}

// cpp: layoutng_style/style/style_scrollbar_color.h:27-29
impl PartialEq for StyleScrollbarColor {
    fn eq(&self, other: &Self) -> bool {
        self.thumb_color_ == other.thumb_color_ && self.track_color_ == other.track_color_
    }
}

unsafe extern "Rust" {
    fn StyleScrollbarColorConstruct(
        thumb_color: StyleColor,
        track_color: StyleColor,
    ) -> StyleScrollbarColor;
}
