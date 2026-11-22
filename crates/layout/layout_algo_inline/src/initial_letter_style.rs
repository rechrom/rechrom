//! Chromium initial-letter font fitting used while preparing inline layout.
//!
//! Tree preparation invokes this algorithm, but the cap-height fitting policy
//! belongs with the rest of initial-letter layout.

#![allow(non_snake_case)]

use super::*;

// cpp: layoutng/internal/boundary/layout_boundary.cc:2545-2605
#[inline(never)]
pub(super) fn InitialLetterTextStyle(
    initial_letter_box: &mut LayoutObject,
) -> *const StyleComputedStyle {
    let box_style = initial_letter_box.StyleRef();
    let paragraph = initial_letter_box.ContainingBlock();
    let box_font = box_style.GetFont();
    if paragraph.is_null() || box_style.InitialLetter().IsNormal() || box_font.is_null() {
        return box_style;
    }
    let paragraph_style = unsafe { &*paragraph }.StyleRef();
    let paragraph_font = paragraph_style.GetFont();
    if paragraph_font.is_null() {
        return box_style;
    }
    let box_primary = (unsafe { &*box_font }).PrimaryFont();
    if box_primary.is_null() {
        return box_style;
    }
    let paragraph_primary = (unsafe { &*paragraph_font }).PrimaryFont();
    if paragraph_primary.is_null() {
        return box_style;
    }
    let cap_height = unsafe { &*box_primary }.GetFontMetrics().CapHeight();
    let paragraph_cap_height = unsafe { &*paragraph_primary }.GetFontMetrics().CapHeight();
    if !(cap_height > 0.0) || !(paragraph_cap_height > 0.0) {
        return box_style;
    }
    let desired_cap_height = paragraph_style.ComputedLineHeight()
        * (box_style.InitialLetter().Size() - 1.0)
        + paragraph_cap_height;
    let source_size = box_style.ComputedFontSize();
    let mut adjusted_size = desired_cap_height * source_size / cap_height;
    if !adjusted_size.is_finite() || adjusted_size <= 1.0 {
        return box_style;
    }
    let node = initial_letter_box.GetNode();
    if node.is_null() {
        return box_style;
    }
    let description = box_style.GetFontDescription();
    let mut families = Vec::new();
    let mut family: *const font_engine::FontFamily = description.Family();
    while !family.is_null() && !unsafe { &*family }.FamilyName().IsNull() {
        let current = unsafe { &*family };
        families.push(current.FamilyName().Utf8());
        family = current.Next();
    }
    let language = description.LocaleOrDefault().Ascii();
    let mut adjusted_font = std::ptr::null_mut();
    while adjusted_size > 1.0 {
        let mut request = FontRequest::new_auto(
            adjusted_size as f64,
            description.LetterSpacing() as f64,
            description.WordSpacing() as f64,
            PublicWritingMode(box_style.GetWritingMode()),
            &language,
            &families,
            description.Weight().RawValue() as f64 / 4.0,
            description.Style().RawValue() >= 14 * 4,
            description.Orientation(),
        )
        .WithBaseDescription(description);
        request.specified_size = adjusted_size as f64 / f64::from(box_style.EffectiveZoom());
        let candidate = unsafe { &mut *CurrentFontResolver() }.Resolve(&request);
        let primary = candidate.PrimaryFont();
        if primary.is_null()
            || unsafe { &*primary }.GetFontMetrics().CapHeight() <= desired_cap_height
        {
            adjusted_font = candidate;
            break;
        }
        adjusted_size -= 1.0;
    }
    if adjusted_font.is_null() || unsafe { &*adjusted_font }.PrimaryFont().is_null() {
        return box_style;
    }
    let mut builder = ComputedStyleBuilder::from_style(box_style);
    builder.SetFont(Member::from_ptr(adjusted_font));
    let font_height = builder.FontHeight();
    builder.SetLineHeight(&Length::Fixed(font_height));
    builder.SetVerticalAlign(EVerticalAlign::kBaseline);
    builder.TakeStyle()
}
