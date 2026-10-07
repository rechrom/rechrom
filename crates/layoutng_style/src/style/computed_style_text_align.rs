use foundation::{ETextAlign, ETextAlignLast};

use super::computed_style::ComputedStyle;

#[allow(non_snake_case)]
impl ComputedStyle {
    // cpp: layoutng_style/style/computed_style.h:1228
    // cpp: layoutng_style/style/computed_style_text_align.cc:14-39
    pub fn GetTextAlignForLine(&self, is_last_line: bool) -> ETextAlign {
        if !is_last_line {
            return self.GetTextAlign();
        }
        match self.TextAlignLast() {
            ETextAlignLast::kStart => ETextAlign::kStart,
            ETextAlignLast::kEnd => ETextAlign::kEnd,
            ETextAlignLast::kLeft => ETextAlign::kLeft,
            ETextAlignLast::kRight => ETextAlign::kRight,
            ETextAlignLast::kCenter => ETextAlign::kCenter,
            ETextAlignLast::kJustify => ETextAlign::kJustify,
            ETextAlignLast::kMatchParent => ETextAlign::kMatchParent,
            ETextAlignLast::kAuto => {
                let text_align = self.GetTextAlign();
                if text_align == ETextAlign::kJustify {
                    ETextAlign::kStart
                } else {
                    text_align
                }
            }
        }
    }
}
