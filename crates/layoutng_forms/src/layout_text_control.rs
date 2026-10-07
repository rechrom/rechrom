#![allow(non_snake_case)]

use std::collections::HashSet;
use std::sync::OnceLock;

use font_engine::fonts::font::Font;
use font_engine::fonts::shaping::text_width::ComputeTextWidth;
use foundation::{AtomicString, StringView};
use layoutng_assembly::internal::layout_box::LayoutBox;
use layoutng_assembly::internal::scrollbar_theme_metrics::ScrollbarThemeThickness;
use layoutng_style::style::computed_style::ComputedStyle;

// cpp: layoutng_forms/layout_text_control.h:34-34
// cpp: layoutng_forms/layout_text_control.cc:34-37
pub fn ScrollbarThickness(box_: &LayoutBox) -> i32 {
    ScrollbarThemeThickness(
        box_.ScrollbarThemeForLayout(),
        box_.StyleRef().UsedScrollbarWidth(),
    )
}

// cpp: layoutng_forms/layout_text_control.cc:39-74
const K_FONT_FAMILIES_WITH_INVALID_CHAR_WIDTH: &[&str] = &[
    "American Typewriter",
    "Arial Hebrew",
    "Chalkboard",
    "Cochin",
    "Corsiva Hebrew",
    "Courier",
    "Euphemia UCAS",
    "Geneva",
    "Gill Sans",
    "Hei",
    "Helvetica",
    "Hoefler Text",
    "InaiMathi",
    "Kai",
    "Lucida Grande",
    "Marker Felt",
    "Monaco",
    "Mshtakan",
    "New Peninim MT",
    "Osaka",
    "Raanana",
    "STHeiti",
    "Symbol",
    "Times",
    "Apple Braille",
    "Apple LiGothic",
    "Apple LiSung",
    "Apple Symbols",
    "AppleGothic",
    "AppleMyungjo",
    "#GungSeo",
    "#HeadLineA",
    "#PCMyungjo",
    "#PilGi",
];

// cpp: layoutng_forms/layout_text_control.h:36-36
// cpp: layoutng_forms/layout_text_control.cc:76-113
pub fn HasValidAvgCharWidth(font: &Font) -> bool {
    let font_data = font.PrimaryFont();
    debug_assert!(!font_data.is_null());
    if font_data.is_null() {
        return false;
    }

    let font_data = unsafe { &*font_data };
    let metrics = font_data.GetFontMetrics();
    if metrics.HasZeroWidth() && font_data.AvgCharWidth() > metrics.ZeroWidth() * 1.7 {
        return false;
    }

    let family = font.GetFontDescription().Family().FamilyName();
    if family.empty() {
        return false;
    }

    // C++ initializes a process-wide HashSet on first use. OnceLock keeps
    // that lazy lifetime without an unsynchronized mutable global pointer.
    static INVALID_FAMILIES: OnceLock<HashSet<AtomicString>> = OnceLock::new();
    let invalid_families = INVALID_FAMILIES.get_or_init(|| {
        let mut families = HashSet::with_capacity(K_FONT_FAMILIES_WITH_INVALID_CHAR_WIDTH.len());
        for font_family in K_FONT_FAMILIES_WITH_INVALID_CHAR_WIDTH {
            families.insert(AtomicString::from_str(font_family));
        }
        families
    });
    !invalid_families.contains(family)
}

// cpp: layoutng_forms/layout_text_control.h:35-35
// cpp: layoutng_forms/layout_text_control.cc:115-130
pub fn GetAvgCharWidth(style: &ComputedStyle) -> f32 {
    let font = unsafe { &*style.GetFont() };
    let primary_font = font.PrimaryFont();
    if !primary_font.is_null() && HasValidAvgCharWidth(font) {
        let width = unsafe { &*primary_font }.AvgCharWidth();
        return width.max(width.round());
    }

    // StringView::from(&str) stores UTF-16 units, matching the one-UChar
    // source span rather than a Latin-1 byte view.
    const K_CH: &str = "0";
    ComputeTextWidth(&StringView::from(K_CH), style.GetFont())
}
