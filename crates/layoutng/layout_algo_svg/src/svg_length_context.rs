// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Native SVGLengthContext over the retained CSS primitive and live style/font.
//! Source: svg_length_context.cc:99-118,147-221; svg_length.cc:165-178,190-200.
use foundation::{FloatValueForLength, RuntimeEnabledFeatures};
use layoutng_assembly::internal::{
    layout_object::LayoutObject, layout_pass_scope::LayoutPassScope, layout_view::LayoutView,
};
use layoutng_style::css::{
    css_math_expression_node::MathError,
    css_numeric_literal_value::CSSNumericLiteralValue,
    css_primitive_value::{ClampToCSSLengthRange, UnitType},
    css_to_length_conversion_data::{CSSToLengthConversionData, ContainerSizes, ViewportSize},
};

// SVGLength::IsRelative deliberately returns false for calculations (the
// source TODO about calc percentage invalidation is preserved).
pub(crate) fn IsRelative(value: &CSSNumericLiteralValue) -> bool {
    use UnitType::*;
    value.MathValue().is_none()
        && !matches!(
            value.GetType(),
            kNumber
                | kInteger
                | kUserUnits
                | kPixels
                | kCentimeters
                | kMillimeters
                | kQuarterMillimeters
                | kInches
                | kPoints
                | kPicas
        )
}

pub(crate) fn Value(
    object: &LayoutObject,
    value: &CSSNumericLiteralValue,
    width: bool,
    containers: Option<ContainerSizes>,
) -> f32 {
    use UnitType::*;
    let style = object.StyleRef();
    let zoom = if RuntimeEnabledFeatures::SvgNewZoomEnabled() {
        style.EffectiveZoom()
    } else {
        1.
    };
    let viewport = crate::svg_viewport::ResolveViewport(object);
    let dimension = if width {
        viewport.width()
    } else {
        viewport.height()
    };
    if value.MathValue().is_none() {
        let numeric = value.DoubleValue() as f32;
        let absolute = match value.GetType() {
            kNumber | kInteger | kUserUnits | kPixels => Some(zoom as f64 * numeric as f64),
            kCentimeters => Some(zoom as f64 * numeric as f64 * (96. / 2.54)),
            kMillimeters => Some(zoom as f64 * numeric as f64 * (96. / 25.4)),
            kQuarterMillimeters => Some(zoom as f64 * numeric as f64 * (96. / 101.6)),
            kInches => Some(zoom as f64 * numeric as f64 * 96.),
            kPoints => Some(zoom as f64 * numeric as f64 * (96. / 72.)),
            kPicas => Some(zoom as f64 * numeric as f64 * 16.),
            kPercentage => Some((numeric * dimension / 100.) as f64),
            _ => None,
        };
        if let Some(value) = absolute {
            return ClampToCSSLengthRange(value);
        }
    }
    // The Style owner supplies source ContainerQueryEvaluator results. Missing
    // context is distinct from a completed selection without an eligible axis.
    if containers.is_none()
        && [
            kContainerWidth,
            kContainerHeight,
            kContainerInlineSize,
            kContainerBlockSize,
            kContainerMin,
            kContainerMax,
        ]
        .iter()
        .any(|unit| value.UsesUnit(*unit))
    {
        crate::svg_viewport::Unsupported("SVG length container-query environment required");
    }
    let resolved = LayoutPassScope::WithSVGResourceTree(|tree| {
        let root = tree.Root();
        let root_style = (root.GetNode() != object.GetNode()).then(|| root.StyleRef());
        let viewport = if root.IsLayoutView() {
            let view = unsafe { &*(root as *const LayoutObject).cast::<LayoutView>() };
            let physical =
                |size: foundation::gfx::SizeF| [size.width() as f64, size.height() as f64];
            ViewportSize {
                small: physical(view.SmallViewportSizeForViewportUnits()),
                large: physical(view.LargeViewportSizeForViewportUnits()),
                dynamic: physical(view.DynamicViewportSizeForViewportUnits()),
            }
        } else {
            ViewportSize::default()
        };
        let mut ignored_flags = 0;
        let mut conversion = CSSToLengthConversionData::new(
            style,
            Some(style),
            root_style,
            viewport,
            containers.unwrap_or_default(),
            zoom,
            &mut ignored_flags,
        );
        if let Some(math) = value.MathValue() {
            let length = math.ConvertToLength(&mut conversion)?;
            Ok(FloatValueForLength(&length, dimension))
        } else {
            conversion
                .ZoomedComputedPixels(value.DoubleValue() as f32 as f64, value.GetType())
                .map(ClampToCSSLengthRange)
        }
    })
    .unwrap_or(Err(MathError::MissingLengthContext));
    resolved.unwrap_or_else(|_| {
        crate::svg_viewport::Unsupported("native SVG length conversion context required")
    })
}
