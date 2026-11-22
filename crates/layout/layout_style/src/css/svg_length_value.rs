// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! SVG animated length payloads share CSS numeric/math ownership. Parsing
//! remains in style_parser; native SVG resolves these values against live font
//! and layout data. Source: svg_length.h:68-95; svg_marker_element.cc:45-76.
use super::css_numeric_literal_value::CSSNumericLiteralValue;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct SVGMarkerLengthValues {
    pub width: Arc<CSSNumericLiteralValue>,
    pub height: Arc<CSSNumericLiteralValue>,
    pub ref_x: Arc<CSSNumericLiteralValue>,
    pub ref_y: Arc<CSSNumericLiteralValue>,
}

impl SVGMarkerLengthValues {
    #[allow(non_snake_case)]
    pub fn UsesUnit(&self, unit: super::css_primitive_value::UnitType) -> bool {
        [&self.width, &self.height, &self.ref_x, &self.ref_y]
            .into_iter()
            .any(|value| value.UsesUnit(unit))
    }
}

impl PartialEq for SVGMarkerLengthValues {
    fn eq(&self, other: &Self) -> bool {
        fn same(a: &Arc<CSSNumericLiteralValue>, b: &Arc<CSSNumericLiteralValue>) -> bool {
            Arc::ptr_eq(a, b) || **a == **b
        }
        same(&self.width, &other.width)
            && same(&self.height, &other.height)
            && same(&self.ref_x, &other.ref_x)
            && same(&self.ref_y, &other.ref_y)
    }
}
