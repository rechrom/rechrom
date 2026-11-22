use font_engine::Font;
use foundation::MakeGarbageCollected;

use super::computed_style_initial_values::ComputedStyleInitialValues;
use super::theme_defaults::layout_style_defaults;
use crate::css::style_color::StyleColor;

#[allow(non_snake_case)]
impl ComputedStyleInitialValues {
    // cpp: layoutng_style/style/computed_style_initial_values.h:1513
    // cpp: layoutng_style/style/computed_style_initial_values_providers.cc:9-11
    pub fn InitialTapHighlightColor() -> StyleColor {
        StyleColor::from_color(layout_style_defaults::kDefaultTapHighlightColor)
    }

    // cpp: layoutng_style/style/computed_style_initial_values.h:1651
    // cpp: layoutng_style/style/computed_style_initial_values_providers.cc:12-14
    pub fn InitialFont() -> *mut Font {
        MakeGarbageCollected(Font::default())
    }
}
