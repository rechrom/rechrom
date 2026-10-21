use foundation::EScrollbarWidth;

use super::computed_style::ComputedStyle;

#[allow(non_snake_case)]
impl ComputedStyle {
    // cpp: layoutng_style/style/computed_style.h:725
    // cpp: layoutng_style/style/scrollbar_style_data.cc:26-33
    pub fn UsedScrollbarWidth(&self) -> EScrollbarWidth {
        if self.PrefersDefaultScrollbarStyles() && self.ScrollbarWidth() != EScrollbarWidth::kNone {
            return EScrollbarWidth::kAuto;
        }
        self.ScrollbarWidth()
    }
}
