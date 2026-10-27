// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! SVG presentation hints shared by the production resolver and its fallback.
//! Source: Chromium 78ef49f74feba3d0dde852881feef92bbf86aad7,
//! core/svg/svg_element.cc:319-412,579-602 and
//! core/svg/svg_transformable_element.cc:45-49.
//! Geometry presentation mappings: Chromium
//! 6c1d401fcca5e1b0030563a90c2f2bba168e0c15, core/svg/svg_{rect,circle,
//! ellipse,use,viewport_container}_element.cc constructors and
//! SVGSVGElement::CollectStyleForPresentationAttribute.
use crate::{
    css_property_names::{kNotKnownExposedPropertyBit, FindProperty},
    parser::{
        css_parser_mode::CSSParserMode,
        production_property_metadata::UnresolvedPropertyIDFromInteger,
        production_property_parser::ParseProperty,
    },
    production_css_value::PropertyValue,
};
use dom::persistent_document::{DOMNamespace, PersistentDocument};

pub(crate) fn CollectPresentationAttributes(
    document: &PersistentDocument,
    index: usize,
) -> Vec<PropertyValue> {
    let node = document.Node(index);
    if node.Namespace() != DOMNamespace::kSVG {
        return Vec::new();
    }
    // SVGElement's common CSS-facing attribute map. Element-specific animated
    // geometry properties join these hints below ordinary author declarations;
    // the native cascade, including explicit auto/zero, owns the final value.
    // Transform is supplied by SVGTransformableElement.
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
            || !(PROPERTIES.contains(&attribute.local_name.as_str())
                || IsGeometryPresentationAttribute(document, index, &attribute.local_name))
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

// SVGAnimatedLength carries a CSSPropertyID only on these element-specific
// properties. Do not turn similarly named attributes on arbitrary SVG elements
// (or <use> width/height) into CSS declarations.
fn IsGeometryPresentationAttribute(
    document: &PersistentDocument,
    index: usize,
    name: &str,
) -> bool {
    match document.Node(index).Name() {
        "rect" => matches!(name, "x" | "y" | "width" | "height" | "rx" | "ry"),
        "circle" => matches!(name, "cx" | "cy" | "r"),
        "ellipse" => matches!(name, "cx" | "cy" | "rx" | "ry"),
        "use" => matches!(name, "x" | "y"),
        "svg" => {
            matches!(name, "x" | "y")
                || (matches!(name, "width" | "height") && IsOutermostSVG(document, index))
        }
        _ => false,
    }
}

// cpp: SVGElement::IsOutermostSVGSVGElement. This uses the DOM parent, not
// flat-tree attachment: an SVG below foreignObject is a new outermost root,
// while the synthetic SVG of a <use> instance remains an inner viewport.
fn IsOutermostSVG(document: &PersistentDocument, index: usize) -> bool {
    let Some(parent) = document.Node(index).Parent() else {
        return true;
    };
    if document
        .Node(parent)
        .IsElement("foreignObject", DOMNamespace::kSVG)
    {
        return true;
    }
    let in_use_shadow_tree = document
        .ShadowHost(document.TreeScopeRoot(index))
        .is_some_and(|host| document.Node(host).IsElement("use", DOMNamespace::kSVG));
    if in_use_shadow_tree
        && document
            .ParentOrShadowHostElement(index)
            .is_some_and(|host| document.Node(host).Namespace() == DOMNamespace::kSVG)
    {
        return false;
    }
    document.Node(parent).Namespace() != DOMNamespace::kSVG
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

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::{CSSPropertyID as P, Length};
    use layoutng_style::style::computed_style::ComputedStyle;

    fn node(document: &PersistentDocument, id: &str) -> usize {
        (0..document.NodeCount())
            .find(|&index| {
                document
                    .Node(index)
                    .FindAttribute("id")
                    .is_some_and(|attribute| attribute.value == id)
            })
            .unwrap()
    }

    fn native<'a>(owner: &'a dom::DOM, id: &str) -> &'a ComputedStyle {
        let document = owner.GetDocument();
        unsafe {
            &*document
                .ResolvedStyleFor(node(document, id))
                .unwrap()
                .native_style
                .Get()
        }
    }

    #[test]
    fn svg_geometry_hints_are_element_specific_and_outermost_aware() {
        let owner = html::html_parser::ParseHTML(
            "<svg id=root x=1 y=2 width=100 height=80>\
             <svg id=inner x=3 y=4 width=50 height=40></svg>\
             <rect id=rect x=1 y=2 width=20 height=30 rx=4 ry=5 cx=99 />\
             <circle id=circle cx=6 cy=7 r=8 width=99 />\
             <ellipse id=ellipse cx=9 cy=10 rx=11 ry=12 r=99 />\
             <use id=use x=13 y=14 width=99 height=99 />\
             <g id=group x=99 y=99 width=99 height=99 />\
             <foreignObject><svg id=foreign width=70 height=60></svg></foreignObject>\
             </svg>",
        );
        let document = owner.GetDocument();
        let ids = |id| {
            CollectPresentationAttributes(document, node(document, id))
                .iter()
                .map(PropertyValue::PropertyID)
                .collect::<Vec<_>>()
        };
        assert_eq!(ids("root"), vec![P::kX, P::kY, P::kWidth, P::kHeight]);
        assert_eq!(ids("inner"), vec![P::kX, P::kY]);
        assert_eq!(
            ids("rect"),
            vec![P::kX, P::kY, P::kWidth, P::kHeight, P::kRx, P::kRy]
        );
        assert_eq!(ids("circle"), vec![P::kCx, P::kCy, P::kR]);
        assert_eq!(ids("ellipse"), vec![P::kCx, P::kCy, P::kRx, P::kRy]);
        assert_eq!(ids("use"), vec![P::kX, P::kY]);
        assert!(ids("group").is_empty());
        assert_eq!(ids("foreign"), vec![P::kWidth, P::kHeight]);
    }

    #[test]
    fn svg_geometry_hints_reach_native_cascade_without_overriding_css_auto_or_zero() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut owner = html::html_parser::ParseHTML(
            "<svg id=root width=100 height=80>\
             <rect id=rect x=3 y=4 width=20 height=30 rx=5 ry=6 />\
             <rect id=auto width=20 height=30 rx=5 style='width:auto;height:0;rx:auto'/>\
             <rect id=zero x=3 y=4 width=20 height=30 />\
             <circle id=circle cx=25% cy=7 r=8 />\
             <ellipse id=ellipse cx=9 cy=10 rx=11 ry=12 />\
             <use id=use x=13 y=14 width=99 height=99 />\
             </svg>",
        );
        owner
            .GetDocumentMut()
            .AppendStyleSheet(crate::ParseCSS("#zero { x:0; y:0; width:0; height:auto }"));
        let media = crate::media_queries::MediaValuesCachedData {
            media_type: foundation::String::from("screen"),
            viewport_width: 800.,
            viewport_height: 600.,
            em_size: 16.,
            ..Default::default()
        };
        let mut engine = crate::StyleEngine::new(&owner);
        engine.Update(&mut owner, &media, &[]).unwrap();
        assert!(
            engine.Diagnostics().is_empty(),
            "{:?}",
            engine.Diagnostics()
        );
        assert_eq!(native(&owner, "root").Width(), &Length::Fixed(100));
        assert_eq!(native(&owner, "root").Height(), &Length::Fixed(80));
        let rect = native(&owner, "rect");
        assert_eq!(rect.X(), &Length::Fixed(3));
        assert_eq!(rect.Y(), &Length::Fixed(4));
        assert_eq!(rect.Width(), &Length::Fixed(20));
        assert_eq!(rect.Height(), &Length::Fixed(30));
        assert_eq!(rect.Rx(), &Length::Fixed(5));
        assert_eq!(rect.Ry(), &Length::Fixed(6));
        let auto = native(&owner, "auto");
        assert!(auto.Width().IsAuto());
        assert_eq!(auto.Height(), &Length::Fixed(0));
        assert!(auto.Rx().IsAuto());
        let zero = native(&owner, "zero");
        assert_eq!(zero.X(), &Length::Fixed(0));
        assert_eq!(zero.Y(), &Length::Fixed(0));
        assert_eq!(zero.Width(), &Length::Fixed(0));
        assert!(zero.Height().IsAuto());
        assert_eq!(native(&owner, "circle").Cx(), &Length::Percent(25.));
        assert_eq!(native(&owner, "circle").Cy(), &Length::Fixed(7));
        assert_eq!(native(&owner, "circle").R(), &Length::Fixed(8));
        assert_eq!(native(&owner, "ellipse").Cx(), &Length::Fixed(9));
        assert_eq!(native(&owner, "ellipse").Cy(), &Length::Fixed(10));
        assert_eq!(native(&owner, "ellipse").Rx(), &Length::Fixed(11));
        assert_eq!(native(&owner, "ellipse").Ry(), &Length::Fixed(12));
        assert_eq!(native(&owner, "use").X(), &Length::Fixed(13));
        assert_eq!(native(&owner, "use").Y(), &Length::Fixed(14));
        assert!(native(&owner, "use").Width().IsAuto());
    }
}
