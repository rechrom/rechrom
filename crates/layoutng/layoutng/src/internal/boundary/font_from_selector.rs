//! Selector-backed font construction and family-chain projection.
//!
//! The layout boundary projects `FontDescription` into the neutral resolver
//! contract. FontEngine owns selection and caching.

#![allow(non_snake_case)]

use font_engine::{Font, FontDescription, FontFamily, FontSelector};

use crate::internal::layout_font_resolver::NativeFontRequest;
use crate::internal::layout_input::WritingMode;

use super::native_input::CurrentNativeFontResolver;

// C++ defines Font::Font(description, selector) here. A free Rust constructor
// keeps that cross-crate implementation in its Bazel source owner.
// cpp: layoutng/internal/boundary/font_from_selector.cc:12-28
#[inline(never)]
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
    )
    .WithBaseDescription(description);
    let resolver = CurrentNativeFontResolver();
    let resolved = unsafe { &mut *resolver }.Resolve(&request);
    // Retain the engine-owned lazy fallback source. Projecting the catalog
    // through OrderedFontData would eagerly admit all fallback faces.
    resolved.CloneWithDescription(description.clone())
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
