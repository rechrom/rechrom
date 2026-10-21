// C++: foundation/style_values/style/text_decoration_thickness.h/.cc.
use crate::{CSSValueID, Length};

// cpp: foundation/style_values/style/text_decoration_thickness.h:14-36
#[repr(C)]
#[derive(Clone, Debug)]
pub struct TextDecorationThickness {
    thickness_: Length,
    thickness_from_font_: bool,
}

// cpp: foundation/style_values/style/text_decoration_thickness.cc:10-11
impl Default for TextDecorationThickness {
    fn default() -> Self {
        Self {
            thickness_: Length::Auto().clone(),
            thickness_from_font_: false,
        }
    }
}

#[allow(non_snake_case)]
impl TextDecorationThickness {
    // cpp: foundation/style_values/style/text_decoration_thickness.cc:13-14
    pub fn new(length: &Length) -> Self {
        Self {
            thickness_: length.clone(),
            thickness_from_font_: false,
        }
    }

    // cpp: foundation/style_values/style/text_decoration_thickness.cc:16-19
    pub fn from_keyword(keyword: CSSValueID) -> Self {
        assert_eq!(keyword, CSSValueID::kFromFont);
        Self {
            thickness_: Length::default(),
            thickness_from_font_: true,
        }
    }

    // cpp: foundation/style_values/style/text_decoration_thickness.h:25-29
    pub fn IsFromFont(&self) -> bool {
        self.thickness_from_font_
    }
    pub fn Thickness(&self) -> &Length {
        assert!(!self.thickness_from_font_);
        &self.thickness_
    }
    pub fn IsAuto(&self) -> bool {
        !self.thickness_from_font_ && self.thickness_.IsAuto()
    }
}

// cpp: foundation/style_values/style/text_decoration_thickness.cc:21-25
impl PartialEq for TextDecorationThickness {
    fn eq(&self, other: &Self) -> bool {
        self.thickness_from_font_ == other.thickness_from_font_
            && self.thickness_ == other.thickness_
    }
}
