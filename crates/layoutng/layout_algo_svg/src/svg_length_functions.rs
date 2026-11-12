#![allow(non_snake_case, non_camel_case_types)]

use foundation::gfx::{SizeF, Vector2dF};
use foundation::length_functions::FloatValueForLength;
use foundation::{Length, RuntimeEnabledFeatures};
use layoutng_style::style::computed_style::ComputedStyle;

// cpp: layoutng_svg/svg_length_functions.h:39-39
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SVGLengthMode {
    kWidth,
    kHeight,
    kOther,
}

// cpp: layoutng_svg/svg_length_functions.cc:33-43
pub fn ValueForLength(length: &Length, zoom: f32, dimension: f32) -> f32 {
    debug_assert_ne!(zoom, 0.0);
    if !length.HasOnlyFixedAndPercent() {
        return 0.0;
    }
    if RuntimeEnabledFeatures::SvgNewZoomEnabled() {
        return FloatValueForLength(length, dimension);
    }
    FloatValueForLength(length, dimension * zoom) / zoom
}

// core/svg/svg_length_functions.cc:126-143. StrokeWidth's canonical
// UnzoomedLength follows a distinct zoom rule from ordinary SVG lengths.
pub fn ValueForUnzoomedLength(
    length: &layoutng_style::style::unzoomed_length::UnzoomedLength,
    zoom: f32,
    dimension: f32,
) -> f32 {
    let length = length.length();
    if RuntimeEnabledFeatures::SvgNewZoomEnabled() {
        if !length.HasOnlyFixedAndPercent() {
            return 0.;
        }
        return FloatValueForLength(length, dimension / zoom) * zoom;
    }
    ValueForLength(length, 1., dimension)
}

// cpp: layoutng_svg/svg_length_functions.cc:45-49
pub fn ValueForLengthWithStyle(length: &Length, style: &ComputedStyle, dimension: f32) -> f32 {
    ValueForLength(length, style.EffectiveZoom(), dimension)
}

// cpp: layoutng_svg/svg_length_functions.cc:51-68
pub fn VectorForLengthPair(
    x_length: &Length,
    y_length: &Length,
    zoom: f32,
    viewport_size: &SizeF,
) -> Vector2dF {
    let mut viewport_size_considering_auto = *viewport_size;
    if x_length.IsAuto() {
        viewport_size_considering_auto.set_width(0.0);
    }
    if y_length.IsAuto() {
        viewport_size_considering_auto.set_height(0.0);
    }
    Vector2dF::new(
        ValueForLength(x_length, zoom, viewport_size_considering_auto.width()),
        ValueForLength(y_length, zoom, viewport_size_considering_auto.height()),
    )
}
