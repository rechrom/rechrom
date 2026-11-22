//! Rechrom's ComputedStyle -> paint-value projection.
//!
//! Chromium painters read `ComputedStyle` directly; there is no Chromium
//! "export" step. Rechrom's immutable Layout -> Paint boundary carries plain
//! `paint_input_types` values instead, and this package owns the pure
//! conversion from the canonical `ComputedStyle` to those values, together
//! with the style-based scrollbar thickness facts StyleCascade needs for
//! viewport units (core/paint/paint_layer_scrollable_area.cc:1698-1738).
//! It is a shared root of the Layout engine (like `layout_style`, which owns
//! `ComputedStyle`): Style names it without depending on Layout internals.
//! Layout keeps the `PaintStyle` publication wrappers and re-exports these items from
//! their established module paths.

#![allow(non_snake_case)]

// Link the layout facade (and layout_assembly's untranslated-symbol stubs) into the unit-test binary
// (layout_style declares extern functions MSVC link.exe must resolve).
#[cfg(test)]
extern crate layout as _;

pub mod paint_style_export;
pub mod scrollbar_metrics;
