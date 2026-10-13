// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! SVG presentation hints shared by the production resolver and its fallback.
//! Source: Chromium 78ef49f74feba3d0dde852881feef92bbf86aad7,
//! core/svg/svg_element.cc:319-412,579-602 and
//! core/svg/svg_transformable_element.cc:45-49.
use crate::{
    css_property_names::{kNotKnownExposedPropertyBit, FindProperty},
    parser::{
        css_parser_mode::CSSParserMode,
        production_property_metadata::UnresolvedPropertyIDFromInteger,
        production_property_parser::ParseProperty,
    },
    production_css_value::PropertyValue,
};
use dom::persistent_document::{DOMNamespace, DOMNode};

pub(crate) fn CollectPresentationAttributes(node: &DOMNode) -> Vec<PropertyValue> {
    if node.Namespace() != DOMNamespace::kSVG {
        return Vec::new();
    }
    // SVGElement's CSS-facing attribute map. Geometry attributes remain with
    // the existing DOM/Layout owner; transform is the animated-property
    // mapping supplied by SVGTransformableElement.
    const PROPERTIES: &[&str] = &[
        "alignment-baseline",
        "baseline-shift",
        "buffered-rendering",
        "clip",
        "clip-path",
        "clip-rule",
        "color",
        "color-interpolation",
        "color-interpolation-filters",
        "color-rendering",
        "cursor",
        "direction",
        "display",
        "dominant-baseline",
        "fill",
        "fill-opacity",
        "fill-rule",
        "filter",
        "flood-color",
        "flood-opacity",
        "font-family",
        "font-size",
        "font-stretch",
        "font-style",
        "font-variant",
        "font-weight",
        "image-rendering",
        "letter-spacing",
        "lighting-color",
        "marker-end",
        "marker-mid",
        "marker-start",
        "mask",
        "mask-type",
        "opacity",
        "overflow",
        "paint-order",
        "pointer-events",
        "shape-rendering",
        "stop-color",
        "stop-opacity",
        "stroke",
        "stroke-dasharray",
        "stroke-dashoffset",
        "stroke-linecap",
        "stroke-linejoin",
        "stroke-miterlimit",
        "stroke-opacity",
        "stroke-width",
        "text-anchor",
        "text-decoration",
        "text-rendering",
        "transform-origin",
        "unicode-bidi",
        "vector-effect",
        "visibility",
        "word-spacing",
        "writing-mode",
        "transform",
    ];
    let mut properties = Vec::new();
    for attribute in node.Attributes() {
        if !attribute.namespace_uri.is_empty()
            || !PROPERTIES.contains(&attribute.local_name.as_str())
        {
            continue;
        }
        let Some(property) = FindProperty(attribute.local_name.as_bytes()) else {
            continue;
        };
        let id = UnresolvedPropertyIDFromInteger(
            property.id_and_exposed_bit & !kNotKnownExposedPropertyBit,
        );
        let value = if attribute.local_name == "transform" {
            NormalizeSVGTransform(&attribute.value)
        } else {
            attribute.value.clone()
        };
        // Parse one value, never a declaration list. SVG accepts user-unit
        // lengths, and a malformed attribute cannot inject another property.
        if let Ok(parsed) = ParseProperty(
            id,
            &foundation::String::from(value),
            false,
            CSSParserMode::kSVGAttributeMode,
        ) {
            properties.extend(parsed);
        }
    }
    properties
}

fn NormalizeSVGTransform(value: &str) -> String {
    // SVGTransformList has SVG-specific argument grammar (including optional
    // rotate centers). Chromium exposes the concatenated AffineTransform to
    // style. Lowering to matrix() preserves that exact boundary and avoids
    // pretending the attribute itself used CSS transform-function syntax.
    type Matrix = [f64; 6];
    fn multiply(left: Matrix, right: Matrix) -> Matrix {
        [
            left[0] * right[0] + left[2] * right[1],
            left[1] * right[0] + left[3] * right[1],
            left[0] * right[2] + left[2] * right[3],
            left[1] * right[2] + left[3] * right[3],
            left[0] * right[4] + left[2] * right[5] + left[4],
            left[1] * right[4] + left[3] * right[5] + left[5],
        ]
    }
    fn operation(name: &str, arguments: &[f64]) -> Option<Matrix> {
        match name.to_ascii_lowercase().as_str() {
            "matrix" if arguments.len() == 6 => arguments.try_into().ok(),
            "translate" if (1..=2).contains(&arguments.len()) => Some([
                1.0,
                0.0,
                0.0,
                1.0,
                arguments[0],
                arguments.get(1).copied().unwrap_or(0.0),
            ]),
            "scale" if (1..=2).contains(&arguments.len()) => Some([
                arguments[0],
                0.0,
                0.0,
                arguments.get(1).copied().unwrap_or(arguments[0]),
                0.0,
                0.0,
            ]),
            "rotate" if arguments.len() == 1 || arguments.len() == 3 => {
                let radians = arguments[0].to_radians();
                let rotation = [
                    radians.cos(),
                    radians.sin(),
                    -radians.sin(),
                    radians.cos(),
                    0.0,
                    0.0,
                ];
                if arguments.len() == 1 {
                    Some(rotation)
                } else {
                    let to_center = [1.0, 0.0, 0.0, 1.0, arguments[1], arguments[2]];
                    let from_center = [1.0, 0.0, 0.0, 1.0, -arguments[1], -arguments[2]];
                    Some(multiply(multiply(to_center, rotation), from_center))
                }
            }
            "skewx" if arguments.len() == 1 => {
                Some([1.0, 0.0, arguments[0].to_radians().tan(), 1.0, 0.0, 0.0])
            }
            "skewy" if arguments.len() == 1 => {
                Some([1.0, arguments[0].to_radians().tan(), 0.0, 1.0, 0.0, 0.0])
            }
            _ => None,
        }
    }

    let bytes = value.as_bytes();
    let mut cursor = 0;
    let mut result = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
    let mut count = 0;
    while cursor < bytes.len() {
        while cursor < bytes.len() && (bytes[cursor].is_ascii_whitespace() || bytes[cursor] == b',')
        {
            cursor += 1;
        }
        let name_start = cursor;
        while cursor < bytes.len() && bytes[cursor].is_ascii_alphabetic() {
            cursor += 1;
        }
        let name = &value[name_start..cursor];
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if name.is_empty() || bytes.get(cursor) != Some(&b'(') {
            return value.to_owned();
        }
        cursor += 1;
        let arguments_start = cursor;
        while cursor < bytes.len() && bytes[cursor] != b')' {
            cursor += 1;
        }
        if cursor == bytes.len() {
            return value.to_owned();
        }
        let arguments: Option<Vec<f64>> = value[arguments_start..cursor]
            .split(|character: char| character == ',' || character.is_ascii_whitespace())
            .filter(|argument| !argument.is_empty())
            .map(|argument| argument.parse().ok())
            .collect();
        let Some(matrix) = arguments.and_then(|arguments| operation(name, &arguments)) else {
            return value.to_owned();
        };
        result = multiply(result, matrix);
        count += 1;
        cursor += 1;
    }
    if count == 0 {
        return value.to_owned();
    }
    format!(
        "matrix({},{},{},{},{},{})",
        result[0], result[1], result[2], result[3], result[4], result[5]
    )
}
