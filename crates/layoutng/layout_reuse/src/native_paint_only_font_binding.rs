//! Reuse-owned preservation of native fonts across paint-only style updates.

use foundation::Member;
use layoutng_style::style::computed_style::{
    ComputedStyle as NativeComputedStyle, ComputedStyleBuilder,
};

// Chromium's ComputedStyle already owns the Font selected for its document
// environment. Preserve that platform binding when a paint-only replacement
// installs StyleEngine's unresolved canonical style.
#[inline(never)]
pub(super) fn PreservePaintOnlyFontBinding(
    style: *const NativeComputedStyle,
    previous: &NativeComputedStyle,
) -> *const NativeComputedStyle {
    let font = previous.GetFont();
    let container_font = previous.ContainerFont();
    if font.is_null() && container_font.is_null() {
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
