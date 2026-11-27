//! Reuse-owned preservation of source fonts across paint-only style updates.

use foundation::Member;
use layout_style::style::computed_style::{
    ComputedStyle as StyleComputedStyle, ComputedStyleBuilder,
};

// Chromium's ComputedStyle already owns the Font selected for its document
// environment. Preserve that platform binding when a paint-only replacement
// installs StyleEngine's unresolved canonical style.
#[inline(never)]
pub(super) fn PreservePaintOnlyFontBinding(
    style: *const StyleComputedStyle,
    previous: &StyleComputedStyle,
) -> *const StyleComputedStyle {
    let font = previous.GetFont();
    let container_font = previous.ContainerFont();
    if font.is_null() && container_font.is_null() {
        return style;
    }
    // The incoming style already carries the bound Fonts (for example the
    // replacement was derived from the previous resident style): installing
    // them again would copy an identical ComputedStyle.
    let incoming = unsafe { &*style };
    if (font.is_null() || std::ptr::eq(incoming.GetFont(), font))
        && (container_font.is_null() || std::ptr::eq(incoming.ContainerFont(), container_font))
    {
        return style;
    }
    let mut builder = ComputedStyleBuilder::from_style(unsafe { &*style });
    if !font.is_null() {
        builder.SetFont(Member::from_ptr(font));
    }
    if !container_font.is_null() {
        builder.SetContainerFont(Member::from_ptr(container_font));
    }
    builder.TakeStyle()
}

/// Whether the platform Font resolved for `previous` also serves `current`:
/// the boundary's FontRequest (layout_invalidation_prepare_tree::rebind_font)
/// reads only the FontDescription, writing mode, text orientation and zoom,
/// plus the parent's font for a MathML depth. Chromium's FontBuilder likewise
/// keeps the existing Font when no font property changed
/// (core/css/resolver/font_builder.cc FontBuilder::CreateFont).
#[inline(never)]
pub(super) fn FontRequestInputsEqual(
    previous: &StyleComputedStyle,
    current: &StyleComputedStyle,
) -> bool {
    previous.MathDepth() == 0
        && current.MathDepth() == 0
        && !previous.GetFont().is_null()
        && !current.GetFont().is_null()
        && previous.GetWritingMode() == current.GetWritingMode()
        && previous.GetTextOrientation() == current.GetTextOrientation()
        && previous.EffectiveZoom() == current.EffectiveZoom()
        && previous.GetFontDescription() == current.GetFontDescription()
}
