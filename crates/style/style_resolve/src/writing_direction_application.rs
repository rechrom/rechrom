// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
#![allow(non_snake_case)]
use super::*;
use foundation::{TextDirection, WritingMode};

// cpp: generated longhands.cc:297-303; longhands_custom.cc:3610-3615,
// 12635-12648; css_value_id_mappings.h:124-138.
pub(super) fn Apply(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
) -> Result {
    let inherit = v.IsInheritedValue() || v.IsUnsetValue();
    let initial = v.IsInitialValue() || inherit && parent.is_none();
    let keyword = || {
        if let CSSValuePayload::kIdentifierClass(value) = v.Payload() {
            Ok(value.0)
        } else {
            Err(LonghandApplicationError::InvalidValue(id))
        }
    };
    if id == CSSPropertyID::kDirection {
        let direction = if initial {
            ComputedStyleInitialValues::InitialDirection()
        } else if inherit {
            parent.unwrap().Direction()
        } else {
            match keyword()? {
                CSSValueID::kLtr => TextDirection::kLtr,
                CSSValueID::kRtl => TextDirection::kRtl,
                _ => return Err(LonghandApplicationError::InvalidValue(id)),
            }
        };
        b.SetDirection(direction);
        return Ok(());
    }
    let mode = if initial {
        ComputedStyleInitialValues::InitialWritingMode()
    } else if inherit {
        parent.unwrap().GetWritingMode()
    } else {
        match keyword()? {
            CSSValueID::kHorizontalTb
            | CSSValueID::kLr
            | CSSValueID::kLrTb
            | CSSValueID::kRl
            | CSSValueID::kRlTb => WritingMode::kHorizontalTb,
            CSSValueID::kVerticalRl | CSSValueID::kTb | CSSValueID::kTbRl => {
                WritingMode::kVerticalRl
            }
            CSSValueID::kVerticalLr => WritingMode::kVerticalLr,
            CSSValueID::kSidewaysRl => WritingMode::kSidewaysRl,
            CSSValueID::kSidewaysLr => WritingMode::kSidewaysLr,
            _ => return Err(LonghandApplicationError::InvalidValue(id)),
        }
    };
    // StyleResolverState::SetWritingMode:347-353 invalidates conversion data
    // and marks the FontBuilder orientation update. Production length data is
    // computed per value, and the native font description is staged here.
    b.SetWritingMode(mode);
    // cpp: font_builder.cc:597-604; computed_style.cc:3115-3130.
    use font_engine::fonts::font_orientation::FontOrientation;
    let orientation = if foundation::IsHorizontalTypographicMode(mode) {
        FontOrientation::kHorizontal
    } else {
        match b.GetTextOrientation() {
            foundation::ETextOrientation::kMixed => FontOrientation::kVerticalMixed,
            foundation::ETextOrientation::kUpright => FontOrientation::kVerticalUpright,
            foundation::ETextOrientation::kSideways => FontOrientation::kVerticalRotated,
            _ => return Err(LonghandApplicationError::Unsupported(id)),
        }
    };
    let mut font = b.GetFontDescription().clone();
    font.SetOrientation(orientation);
    StageFontDescription(b, &font);
    Ok(())
}
