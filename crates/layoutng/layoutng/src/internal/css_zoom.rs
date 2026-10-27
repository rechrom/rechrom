#![allow(non_snake_case)]
//! Fast-path facade for CSS-unit conversion used by layout and paint.

use super::layout_input::{ComputedStyle, Edges, GridTrackBreadthKind, GridTrackListInput};
use super::paint_input::PaintStyleData;
use std::borrow::Cow;

#[path = "../../../layout_invalidation/src/native_css_zoom.rs"]
mod layout_invalidation_native_css_zoom;
use layout_invalidation_native_css_zoom::ZoomedStyleNonUnit;

/// Preserve the overwhelmingly common unit-zoom path as a borrowed style.
/// Clone-and-scale work is owned by layout_invalidation so sampling profiles
/// do not charge variable-size input conversion to the core facade.
#[inline]
pub fn ZoomedStyle(style: &ComputedStyle) -> Cow<'_, ComputedStyle> {
    let zoom = style.extended.as_ref().map_or(1.0, |e| e.effective_zoom) as f64;
    if zoom == 1.0 {
        return Cow::Borrowed(style);
    }
    Cow::Owned(ZoomedStyleNonUnit(style, zoom))
}
