// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Direct numeric/length computed-style virtuals from fixed Chromium
//! longhands_custom.cc. The original computed fields and zoom are borrowed.
use crate::{
    computed_style_utils as utils,
    production_css_value::{self as values, Value},
};
use foundation::CSSPropertyID as P;
use layoutng_style::style::computed_style::ComputedStyle;
use std::rc::Rc;
use style_model::css_primitive_value::UnitType;
pub fn ScalarValue(property: P, style: &ComputedStyle) -> Option<Rc<Value>> {
    Some(match property {
        // longhands_custom.cc:3536-3542.
        P::kCx => utils::ZoomAdjustedPixelValueForLength(style.Cx(), style),
        // longhands_custom.cc:3552-3558.
        P::kCy => utils::ZoomAdjustedPixelValueForLength(style.Cy(), style),
        // longhands_custom.cc:4145-4152.
        P::kFillOpacity => values::numeric(style.FillOpacity() as f64, UnitType::kNumber),
        // longhands_custom.cc:4201-4208.
        P::kFlexBasis => utils::ZoomAdjustedPixelValueForLength(style.FlexBasis(), style),
        // longhands_custom.cc:4231-4238.
        P::kFlexGrow => values::numeric(style.FlexGrow() as f64, UnitType::kNumber),
        // longhands_custom.cc:4249-4256.
        P::kFlexLineCount => values::numeric(style.FlexLineCount() as f64, UnitType::kNumber),
        // longhands_custom.cc:4267-4274.
        P::kFlexShrink => values::numeric(style.FlexShrink() as f64, UnitType::kNumber),
        // longhands_custom.cc:4393-4400.
        P::kFloodOpacity => values::numeric(style.FloodOpacity() as f64, UnitType::kNumber),
        // longhands_custom.cc:7061-7068.
        P::kMathDepth => values::numeric(style.MathDepth() as f64, UnitType::kNumber),
        // longhands_custom.cc:7429-7436.
        P::kOpacity => values::numeric(style.Opacity() as f64, UnitType::kNumber),
        // longhands_custom.cc:7438-7445.
        P::kOrder => values::numeric(style.Order() as f64, UnitType::kNumber),
        // longhands_custom.cc:7455-7462.
        P::kOrphans => values::numeric(style.Orphans() as f64, UnitType::kNumber),
        // longhands_custom.cc:7546-7552.
        P::kOutlineOffset => utils::ZoomAdjustedPixelValue(*style.OutlineOffset() as f64, style),
        // longhands_custom.cc:7614-7620.
        P::kOutlineWidth => utils::ZoomAdjustedPixelValue(*style.OutlineWidth() as f64, style),
        // longhands_custom.cc:8447-8453.
        P::kR => utils::ZoomAdjustedPixelValueForLength(style.R(), style),
        // longhands_custom.cc:8463-8470.
        P::kReadingOrder => values::numeric(style.ReadingOrder() as f64, UnitType::kNumber),
        // longhands_custom.cc:8654-8660.
        P::kRx => utils::ZoomAdjustedPixelValueForLength(style.Rx(), style),
        // longhands_custom.cc:8674-8680.
        P::kRy => utils::ZoomAdjustedPixelValueForLength(style.Ry(), style),
        // longhands_custom.cc:8973-8979.
        P::kScrollMarginBottom => {
            utils::ZoomAdjustedPixelValue(style.ScrollMarginBottom() as f64, style)
        }
        // longhands_custom.cc:9008-9014.
        P::kScrollMarginLeft => {
            utils::ZoomAdjustedPixelValue(style.ScrollMarginLeft() as f64, style)
        }
        // longhands_custom.cc:9025-9031.
        P::kScrollMarginRight => {
            utils::ZoomAdjustedPixelValue(style.ScrollMarginRight() as f64, style)
        }
        // longhands_custom.cc:9042-9048.
        P::kScrollMarginTop => utils::ZoomAdjustedPixelValue(style.ScrollMarginTop() as f64, style),
        // longhands_custom.cc:9240-9247.
        P::kShapeImageThreshold => {
            values::numeric(style.ShapeImageThreshold() as f64, UnitType::kNumber)
        }
        // longhands_custom.cc:9522-9529.
        P::kStopOpacity => values::numeric(style.StopOpacity() as f64, UnitType::kNumber),
        // longhands_custom.cc:9649-9656.
        P::kStrokeMiterlimit => values::numeric(style.StrokeMiterLimit() as f64, UnitType::kNumber),
        // longhands_custom.cc:9658-9665.
        P::kStrokeOpacity => values::numeric(style.StrokeOpacity() as f64, UnitType::kNumber),
        // longhands_custom.cc:11078-11084.
        P::kWebkitBorderVerticalSpacing => {
            utils::ZoomAdjustedPixelValue(style.VerticalBorderSpacing() as f64, style)
        }
        // longhands_custom.cc:11119-11126.
        P::kWebkitBoxFlex => values::numeric(style.BoxFlex() as f64, UnitType::kNumber),
        // longhands_custom.cc:11128-11135.
        P::kWebkitBoxOrdinalGroup => {
            values::numeric(style.BoxOrdinalGroup() as f64, UnitType::kNumber)
        }
        // longhands_custom.cc:12203-12209.
        P::kWebkitTextStrokeWidth => {
            utils::ZoomAdjustedPixelValue(style.TextStrokeWidth() as f64, style)
        }
        // longhands_custom.cc:12395-12402.
        P::kWidows => values::numeric(style.Widows() as f64, UnitType::kNumber),
        // longhands_custom.cc:12673-12679.
        P::kX => utils::ZoomAdjustedPixelValueForLength(style.X(), style),
        // longhands_custom.cc:12689-12695.
        P::kY => utils::ZoomAdjustedPixelValueForLength(style.Y(), style),
        // longhands_custom.cc:12752-12759.
        P::kZoom => values::numeric(style.Zoom() as f64, UnitType::kNumber),
        _ => return None,
    })
}
