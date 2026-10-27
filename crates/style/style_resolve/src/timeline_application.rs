// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Native ComputedStyle timeline definition lists, independent of animation data.
#![allow(non_snake_case)]
use super::*;
use foundation::AtomicString;
use layoutng_style::style::{
    computed_style_constants::TimelineAxis,
    style_timeline_scope::{StyleTimelineScope, StyleTimelineScopeType},
    timeline_inset::TimelineInset,
};

pub(super) fn IsProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kScrollTimelineName
            | kScrollTimelineAxis
            | kViewTimelineName
            | kViewTimelineAxis
            | kViewTimelineInset
            | kTimelineScope
    )
}

// ConvertViewTimelineName and ConvertTimelineScope deliberately convert
// unscoped AtomicString; ScopedCSSName belongs to timeline consumption.
fn Name(
    id: CSSPropertyID,
    v: &Value,
    none: bool,
) -> std::result::Result<AtomicString, LonghandApplicationError> {
    match v.Payload() {
        CSSValuePayload::kIdentifierClass(k) if none && k.0 == CSSValueID::kNone => {
            Ok(AtomicString::default())
        }
        CSSValuePayload::kCustomIdentClass(name) if name.property == CSSPropertyID::kInvalid => {
            Ok(name.name.clone())
        }
        _ => Err(LonghandApplicationError::InvalidValue(id)),
    }
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
    let inherit = v.IsInheritedValue() || v.IsUnsetValue() && CSSProperty::Get(id).IsInherited();
    let initial = v.IsInitialValue()
        || v.IsUnsetValue() && !CSSProperty::Get(id).IsInherited()
        || inherit && parent.is_none();
    if inherit && parent.is_some() && v.IsInheritedValue() {
        b.SetHasExplicitInheritance();
        parent.unwrap().SetChildHasExplicitInheritance();
    }
    if initial {
        match id {
            kScrollTimelineName => {
                b.SetScrollTimelineName(&ComputedStyleInitialValues::InitialScrollTimelineName())
            }
            kScrollTimelineAxis => {
                b.SetScrollTimelineAxis(&ComputedStyleInitialValues::InitialScrollTimelineAxis())
            }
            kViewTimelineName => {
                b.SetViewTimelineName(&ComputedStyleInitialValues::InitialViewTimelineName())
            }
            kViewTimelineAxis => {
                b.SetViewTimelineAxis(&ComputedStyleInitialValues::InitialViewTimelineAxis())
            }
            kViewTimelineInset => {
                b.SetViewTimelineInset(&ComputedStyleInitialValues::InitialViewTimelineInset())
            }
            kTimelineScope => {
                b.SetTimelineScope(&ComputedStyleInitialValues::InitialTimelineScope())
            }
            _ => return Err(LonghandApplicationError::Unsupported(id)),
        }
        return Ok(());
    }
    if inherit {
        let p = parent.unwrap();
        match id {
            kScrollTimelineName => b.SetScrollTimelineName(p.ScrollTimelineName()),
            kScrollTimelineAxis => b.SetScrollTimelineAxis(p.ScrollTimelineAxis()),
            kViewTimelineName => b.SetViewTimelineName(p.ViewTimelineName()),
            kViewTimelineAxis => b.SetViewTimelineAxis(p.ViewTimelineAxis()),
            kViewTimelineInset => {
                if b.EffectiveZoom() != p.EffectiveZoom() {
                    return Err(LonghandApplicationError::Unsupported(id));
                }
                b.SetViewTimelineInset(p.ViewTimelineInset());
            }
            kTimelineScope => b.SetTimelineScope(p.TimelineScope()),
            _ => return Err(LonghandApplicationError::Unsupported(id)),
        }
        return Ok(());
    }
    if id == kTimelineScope {
        let scope = match Identifier(id, v).ok() {
            Some(CSSValueID::kNone) => {
                StyleTimelineScope::new(StyleTimelineScopeType::kNone, Vec::new())
            }
            // Converter supports this typed branch; production stable parser
            // rejects `all` because CSSTimelineScopeAll is experimental.
            Some(CSSValueID::kAll) => {
                StyleTimelineScope::new(StyleTimelineScopeType::kAll, Vec::new())
            }
            Some(_) => return Err(LonghandApplicationError::InvalidValue(id)),
            None => StyleTimelineScope::new(
                StyleTimelineScopeType::kNames,
                MapList(id, v, |x| Name(id, x, false))?,
            ),
        };
        b.SetTimelineScope(&scope);
        return Ok(());
    }
    match id {
        kScrollTimelineName | kViewTimelineName => {
            let names = MapList(id, v, |x| Name(id, x, true))?;
            if id == kScrollTimelineName {
                b.SetScrollTimelineName(&names);
            } else {
                b.SetViewTimelineName(&names);
            }
        }
        kScrollTimelineAxis | kViewTimelineAxis => {
            let axes = MapList(id, v, |x| {
                Ok(match Identifier(id, x).ok() {
                    Some(CSSValueID::kBlock) => TimelineAxis::kBlock,
                    Some(CSSValueID::kInline) => TimelineAxis::kInline,
                    Some(CSSValueID::kX) => TimelineAxis::kX,
                    Some(CSSValueID::kY) => TimelineAxis::kY,
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                })
            })?;
            if id == kScrollTimelineAxis {
                b.SetScrollTimelineAxis(&axes);
            } else {
                b.SetViewTimelineAxis(&axes);
            }
        }
        kViewTimelineInset => {
            let insets = MapList(id, v, |x| {
                let CSSValuePayload::kValuePairClass(pair) = x.Payload() else {
                    return Err(LonghandApplicationError::InvalidValue(id));
                };
                let side = |v: &Value| {
                    if Identifier(id, v).ok() == Some(CSSValueID::kAuto) {
                        Ok(Length::Auto().clone())
                    } else {
                        svg_application::ConvertLength(id, b, v, root, media)
                    }
                };
                Ok(TimelineInset::new(
                    &side(&pair.first)?,
                    &side(&pair.second)?,
                ))
            })?;
            b.SetViewTimelineInset(&insets);
        }
        _ => return Err(LonghandApplicationError::Unsupported(id)),
    }
    Ok(())
}
