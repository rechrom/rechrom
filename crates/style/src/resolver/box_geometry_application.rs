// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Generated physical box length Apply*, including logical aliases resolved upstream.
#![allow(non_snake_case)]
use super::*;
pub(super) fn IsProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kWidth
            | kHeight
            | kMinWidth
            | kMinHeight
            | kMaxWidth
            | kMaxHeight
            | kMarginTop
            | kMarginRight
            | kMarginBottom
            | kMarginLeft
            | kPaddingTop
            | kPaddingRight
            | kPaddingBottom
            | kPaddingLeft
            | kTop
            | kRight
            | kBottom
            | kLeft
            | kShapeMargin
    )
}
pub(super) fn Apply(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> Result {
    use CSSPropertyID::*;
    let inherit = v.IsInheritedValue();
    if v.IsInitialValue() || v.IsUnsetValue() || inherit && parent.is_none() {
        return ApplyInitial(id, b);
    }
    // generated longhands.cc, physical lengths: the parent value is copied
    // after ApplyParentValueIfZoomChanged. That branch needs document policy.
    let length = if inherit {
        let p = parent.unwrap();
        if p.EffectiveZoom() != b.EffectiveZoom() {
            return Err(LonghandApplicationError::Unsupported(id));
        }
        match id {
            kWidth => p.Width(),
            kHeight => p.Height(),
            kMinWidth => p.MinWidth(),
            kMinHeight => p.MinHeight(),
            kMaxWidth => p.MaxWidth(),
            kMaxHeight => p.MaxHeight(),
            kMarginTop => p.MarginTop(),
            kMarginRight => p.MarginRight(),
            kMarginBottom => p.MarginBottom(),
            kMarginLeft => p.MarginLeft(),
            kPaddingTop => p.PaddingTop(),
            kPaddingRight => p.PaddingRight(),
            kPaddingBottom => p.PaddingBottom(),
            kPaddingLeft => p.PaddingLeft(),
            kTop => p.Top(),
            kRight => p.Right(),
            kBottom => p.Bottom(),
            kLeft => p.Left(),
            kShapeMargin => p.ShapeMargin(),
            _ => return Err(LonghandApplicationError::Unsupported(id)),
        }
        .clone()
    } else if let CSSValuePayload::kIdentifierClass(k) = v.Payload() {
        // ConvertLengthSizing / ConvertLengthMaxSizing / ConvertLengthOrAuto.
        match k.0 {
            CSSValueID::kAuto => Length::Auto().clone(),
            CSSValueID::kNone => Length::None(),
            CSSValueID::kMinContent => Length::MinContent().clone(),
            CSSValueID::kMaxContent => Length::MaxContent().clone(),
            CSSValueID::kFitContent => Length::from_type(LengthType::kFitContent),
            CSSValueID::kStretch | CSSValueID::kWebkitFillAvailable => {
                Length::from_type(LengthType::kStretch)
            }
            _ => return Err(LonghandApplicationError::InvalidValue(id)),
        }
    } else {
        text_application::ConvertLength(id, b, v, root, media)?
    };
    ApplyConvertedLength(id, b, &length)?;
    if inherit {
        b.SetHasExplicitInheritance();
        parent.unwrap().SetChildHasExplicitInheritance();
    }
    Ok(())
}
