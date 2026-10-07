// C++: font_engine/fonts/font_services.cc. The remaining methods in this
// production file are still recorded as blocked in translation status.
#![allow(non_snake_case)]

use foundation::{AtomicString, LayoutUnit, RuntimeEnabledFeatures};

use super::font::Font;
use super::font_baseline::FontBaseline;

// cpp: font_engine/fonts/font_services.cc:118-135
#[unsafe(no_mangle)]
pub extern "Rust" fn FontEmphasisMarkHeight(font: &Font, mark: &AtomicString) -> LayoutUnit {
    let primary = font.PrimaryFont();
    if mark.empty() || primary.is_null() {
        return LayoutUnit::default();
    }
    let mut character = u32::from(mark.at(0));
    if (0xd800..=0xdbff).contains(&character) && mark.length() > 1 {
        let trail = u32::from(mark.at(1));
        if (0xdc00..=0xdfff).contains(&trail) {
            character = 0x10000 + ((character - 0xd800) << 10) + (trail - 0xdc00);
        }
    }
    let primary = unsafe { &*primary };
    if primary.GlyphForCharacter(character) == 0 {
        return LayoutUnit::default();
    }
    if RuntimeEnabledFeatures::TextEmphasisAsRubyEnabled() {
        return primary
            .NormalizedTypoAscentAndDescent(FontBaseline::kAlphabeticBaseline)
            .LineHeight();
    }
    LayoutUnit::FromFloatRound(primary.GetFontMetrics().FloatHeight())
}
