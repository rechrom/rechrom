// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Generated Apply and native value owners for three stable properties.
#![allow(non_snake_case)]
use super::*;
use foundation::{MakeGarbageCollected, Member, StyleInitialLetter, StyleNameScope};
use layoutng_style::style::scroll_marker_group::{
    ScrollMarkerGroup, ScrollMarkerMode, ScrollMarkerPosition,
};

pub(super) fn IsProperty(id: CSSPropertyID) -> bool {
    matches!(
        id,
        CSSPropertyID::kInitialLetter
            | CSSPropertyID::kScrollMarkerGroup
            | CSSPropertyID::kTriggerScope
    )
}

// CSSPrimitiveValue::ComputeNumber:377-384; CSSMathFunctionValue:99-114.
// Size narrows double to float in converter.cc:2408-2409; do not introduce
// ClampTo<float>, which this source conversion does not use.
fn Number(id: CSSPropertyID, value: &Value) -> std::result::Result<f64, LonghandApplicationError> {
    let n = match value.Payload() {
        CSSValuePayload::kNumericLiteralClass(n) if n.IsNumber() => n.DoubleValue(),
        CSSValuePayload::kMathFunctionClass(math)
            if math.Category()
                == crate::css_math_expression_node::CalculationResultCategory::Number =>
        {
            math.ComputeValue(
                &mut |_, _| Err(crate::css_math_expression_node::MathError::MissingLengthContext),
                None,
            )
            .map_err(|_| LonghandApplicationError::Unsupported(id))?
        }
        _ => return Err(LonghandApplicationError::InvalidValue(id)),
    };
    Ok(crate::css_value_clamping_utils::CSSValueClampingUtils::ClampDouble(n))
}

// style_builder_converter.cc:2398-2433, native style_initial_letter.cc:14-39.
fn InitialLetter(
    id: CSSPropertyID,
    v: &Value,
) -> std::result::Result<StyleInitialLetter, LonghandApplicationError> {
    if let CSSValuePayload::kIdentifierClass(k) = v.Payload() {
        return if k.0 == CSSValueID::kNormal {
            Ok(StyleInitialLetter::Normal())
        } else {
            Err(LonghandApplicationError::InvalidValue(id))
        };
    }
    let CSSValuePayload::kValueListClass(list) = v.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    if list.separator != crate::production_css_value::ListSeparator::Space
        || !(1..=2).contains(&list.values.len())
    {
        return Err(LonghandApplicationError::InvalidValue(id));
    }
    let size = Number(id, &list.values[0])? as f32;
    // Source parser accepts calc(<1) but the native owner DCHECKs >=1.
    // Keep that boundary explicit instead of inventing a minimum-one clamp.
    if size < 1. {
        return Err(LonghandApplicationError::Unsupported(id));
    }
    if list.values.len() == 1 {
        return Ok(StyleInitialLetter::with_size(size));
    }
    let second = &list.values[1];
    if let CSSValuePayload::kIdentifierClass(k) = second.Payload() {
        return match k.0 {
            CSSValueID::kDrop => Ok(StyleInitialLetter::Drop(size)),
            CSSValueID::kRaise => Ok(StyleInitialLetter::Raise(size)),
            _ => Err(LonghandApplicationError::InvalidValue(id)),
        };
    }
    let sink = Number(id, second)?;
    // The explicit sink uses C++ double->int, not saturated_cast. Undefined
    // out-of-int-range values cannot be fabricated as a saturated native sink.
    if sink < 1. || sink > i32::MAX as f64 {
        return Err(LonghandApplicationError::Unsupported(id));
    }
    Ok(StyleInitialLetter::with_sink(size, sink as i32))
}

fn Marker(
    id: CSSPropertyID,
    v: &Value,
) -> std::result::Result<Member<ScrollMarkerGroup>, LonghandApplicationError> {
    let position = |v: &Value| match Identifier(id, v)? {
        CSSValueID::kBefore => Ok(ScrollMarkerPosition::kBefore),
        CSSValueID::kAfter => Ok(ScrollMarkerPosition::kAfter),
        _ => Err(LonghandApplicationError::InvalidValue(id)),
    };
    let group = match v.Payload() {
        CSSValuePayload::kIdentifierClass(k) if k.0 == CSSValueID::kNone => {
            return Ok(Member::default())
        }
        CSSValuePayload::kIdentifierClass(_) => ScrollMarkerGroup::new_links(position(v)?),
        CSSValuePayload::kValuePairClass(pair) => {
            let mode = match Identifier(id, &pair.second)? {
                CSSValueID::kLinks => ScrollMarkerMode::kLinks,
                CSSValueID::kTabs => ScrollMarkerMode::kTabs,
                _ => return Err(LonghandApplicationError::InvalidValue(id)),
            };
            ScrollMarkerGroup::new(position(&pair.first)?, mode)
        }
        _ => return Err(LonghandApplicationError::InvalidValue(id)),
    };
    Ok(Member::from_ptr(MakeGarbageCollected(group)))
}

pub(super) fn Apply(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
) -> Result {
    let inherit = v.IsInheritedValue() || v.IsUnsetValue() && CSSProperty::Get(id).IsInherited();
    let initial = v.IsInitialValue() || v.IsUnsetValue() && !inherit || inherit && parent.is_none();
    let inherited = parent.filter(|_| inherit && !initial);
    if v.IsInheritedValue() && inherited.is_some() && !CSSProperty::Get(id).IsInherited() {
        b.SetHasExplicitInheritance();
        inherited.unwrap().SetChildHasExplicitInheritance();
    }
    match id {
        CSSPropertyID::kInitialLetter => b.SetInitialLetterOwned(if initial {
            ComputedStyleInitialValues::InitialInitialLetter()
        } else if let Some(p) = inherited {
            *p.InitialLetter()
        } else {
            InitialLetter(id, v)?
        }),
        CSSPropertyID::kScrollMarkerGroup => b.SetScrollMarkerGroup(if initial {
            Member::default()
        } else if let Some(p) = inherited {
            Member::from_ptr(p.GetScrollMarkerGroup())
        } else {
            Marker(id, v)?
        }),
        CSSPropertyID::kTriggerScope => b.SetTriggerScopeOwned(if initial {
            StyleNameScope::default()
        } else if let Some(p) = inherited {
            p.TriggerScope().clone()
        } else {
            anchor_application::ConvertNameScope(id, v)?
        }),
        _ => return Err(LonghandApplicationError::Unsupported(id)),
    }
    Ok(())
}
