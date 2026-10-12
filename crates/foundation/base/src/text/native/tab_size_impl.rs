// C++: src/foundation/text/native/tab_size.cc

use crate::layout_features::runtime_enabled_features::RuntimeEnabledFeatures;

use super::tab_size::TabSize;

impl TabSize {
    // C++: tab_size.cc, GetPixelSize.
    pub fn GetPixelSize(&self, space_width: f32, letter_spacing: f32, word_spacing: f32) -> f32 {
        if !RuntimeEnabledFeatures::TabSizeWithSpacingEnabled() {
            return if self.IsSpaces() {
                self.float_value_ * space_width
            } else {
                self.float_value_
            };
        }
        if self.IsSpaces() {
            self.float_value_ * (space_width + letter_spacing + word_spacing)
        } else {
            self.float_value_
        }
    }
}
