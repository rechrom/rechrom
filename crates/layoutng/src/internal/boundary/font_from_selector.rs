#![allow(non_snake_case)]

use font_engine::{Font, FontDescription, FontFamily, FontSelector, SimpleFontData};

use crate::internal::layout_font_resolver::NativeFontRequest;
use crate::internal::layout_input::WritingMode;

use super::native_input::CurrentNativeFontResolver;

// The resolved-data constructor is owned by //src/font_engine/fonts/font_data.cc.
// It initializes Font's private description, primary, and ordered fallback
// fields; this package owns only the selector-to-resolver path below.
unsafe extern "Rust" {
    pub(crate) fn FontNewFromResolvedData(
        description: &FontDescription,
        ordered_fonts: &[*const SimpleFontData],
        primary_font: *const SimpleFontData,
    ) -> Font;
}

// C++ defines Font::Font(description, selector) here. A free Rust constructor
// keeps that cross-crate implementation in its Bazel source owner.
// cpp: layoutng/internal/boundary/font_from_selector.cc:12-28
pub fn FontFromSelector(description: &FontDescription, _selector: *mut FontSelector) -> Font {
    let vertical = description.IsVerticalAnyUpright();
    let mut families = Vec::<String>::new();
    let mut family: *const FontFamily = description.Family();
    while !family.is_null() && !unsafe { &*family }.FamilyName().IsNull() {
        let current = unsafe { &*family };
        families.push(current.FamilyName().Utf8());
        family = current.Next();
    }

    let request = NativeFontRequest::new_auto(
        description.ComputedSize() as f64,
        description.LetterSpacing() as f64,
        description.WordSpacing() as f64,
        if vertical {
            WritingMode::kVerticalRl
        } else {
            WritingMode::kHorizontalTb
        },
        "",
        &families,
        description.Weight().RawValue() as f64 / 4.0,
        description.Style().RawValue() >= 14 * 4,
        description.Orientation(),
    );
    let resolver = CurrentNativeFontResolver();
    let resolved = unsafe { &mut *resolver }.Resolve(&request);
    let primary_font = resolved.PrimaryFont();
    assert!(!primary_font.is_null());
    let ordered_fonts: Vec<*const SimpleFontData> = resolved
        .OrderedFontData()
        .iter()
        .map(|font| font.Get() as *const SimpleFontData)
        .collect();
    unsafe { FontNewFromResolvedData(description, &ordered_fonts, primary_font) }
}

// The constructor declaration lives in font_engine; this package supplies
// its selector-backed definition at the shared layout boundary.
// cpp: layoutng/internal/boundary/font_from_selector.cc:12-28
#[unsafe(no_mangle)]
pub extern "Rust" fn FontNewWithSelector(
    description: FontDescription,
    selector: *mut FontSelector,
) -> Font {
    FontFromSelector(&description, selector)
}
