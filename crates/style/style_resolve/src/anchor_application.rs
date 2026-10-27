// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Anchor positioning native fields and real Chromium converters.
#![allow(non_snake_case)]
use super::*;
use crate::production_position_area as area;
use foundation::{
    EPositionTryOrder, HeapVector, MakeGarbageCollected, Member, ScopedCSSName, ScopedCSSNameList,
    StyleNameScope, StyleNameScopeType,
};
use layoutng_style::style::{
    computed_style_constants::{PositionVisibility, TryTactic},
    position_area::{PositionArea, PositionAreaRegion as R},
    position_try_fallbacks::{kNoTryTactics, PositionTryFallback, PositionTryFallbacks},
    style_position_anchor::{StylePositionAnchor, Type},
};
pub(super) fn IsAnchorProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kAnchorName
            | kAnchorScope
            | kPositionAnchor
            | kPositionArea
            | kPositionVisibility
            | kPositionTryFallbacks
            | kPositionTryOrder
    )
}
// converter.cc:2287-2294. The existing production owner operates in the null
// document-root scope domain. Preserve native scoped names, never plain strings.
// Shadow/non-null scope and StyleResolverState::HasTreeScopedReference need an
// actual owner boundary and remain partial in the source ledger.
pub(super) fn Name(
    id: CSSPropertyID,
    v: &Value,
) -> std::result::Result<Member<ScopedCSSName>, LonghandApplicationError> {
    let CSSValuePayload::kCustomIdentClass(name) = v.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    if name.property != CSSPropertyID::kInvalid || name.name.IsNull() {
        return Err(LonghandApplicationError::InvalidValue(id));
    }
    Ok(Member::from_ptr(MakeGarbageCollected(ScopedCSSName::new(
        &name.name,
        std::ptr::null(),
    ))))
}
fn Names(
    id: CSSPropertyID,
    v: &Value,
) -> std::result::Result<Member<ScopedCSSNameList>, LonghandApplicationError> {
    let CSSValuePayload::kValueListClass(list) = v.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    if list.separator != crate::production_css_value::ListSeparator::Comma || list.values.is_empty()
    {
        return Err(LonghandApplicationError::InvalidValue(id));
    }
    let names = list
        .values
        .iter()
        .map(|v| Name(id, v))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(Member::from_ptr(MakeGarbageCollected(
        ScopedCSSNameList::new(HeapVector::from(names)),
    )))
}
// converter.cc:2366-2390. Reuse this native scope owner for TriggerScope.
pub(super) fn ConvertNameScope(
    id: CSSPropertyID,
    v: &Value,
) -> std::result::Result<StyleNameScope, LonghandApplicationError> {
    if matches!(v.Payload(), CSSValuePayload::kIdentifierClass(k) if k.0 == CSSValueID::kNone) {
        return Ok(StyleNameScope::default());
    }
    if let CSSValuePayload::kScopedKeywordClass(keyword) = v.Payload() {
        if keyword.GetValueID() != CSSValueID::kAll {
            return Err(LonghandApplicationError::InvalidValue(id));
        }
        if !keyword.GetTreeScope().is_null() {
            return Err(LonghandApplicationError::Unsupported(id));
        }
        let populated = if v.IsScopedValue() {
            keyword.clone()
        } else {
            keyword.PopulateForDocumentRoot()
        };
        return Ok(StyleNameScope::new(
            StyleNameScopeType::kAll,
            populated.GetPopulatedTreeScope(),
            std::ptr::null(),
        ));
    }
    let names = Names(id, v)?;
    Ok(StyleNameScope::new(
        StyleNameScopeType::kNames,
        std::ptr::null(),
        names.Get(),
    ))
}
// converter.cc:3999-4217. Single-value repetition and the default second span
// are intentionally distinct; physical resolution belongs to native PositionArea.
fn Area(
    id: CSSPropertyID,
    v: &Value,
) -> std::result::Result<PositionArea, LonghandApplicationError> {
    let span = |k| area::ConvertSpan(k, false).ok_or(LonghandApplicationError::InvalidValue(id));
    if let CSSValuePayload::kValuePairClass(pair) = v.Payload() {
        let a = span(Identifier(id, &pair.first)?)?;
        let b = span(Identifier(id, &pair.second)?)?;
        return Ok(PositionArea::new(a.0, a.1, b.0, b.1));
    }
    let k = Identifier(id, v)?;
    if k == CSSValueID::kNone {
        return Ok(PositionArea::default());
    }
    let a = span(k)?;
    let b = if area::IsRepeated(k) {
        a
    } else {
        (R::kAll, R::kAll)
    };
    Ok(PositionArea::new(a.0, a.1, b.0, b.1))
}
fn Fallback(
    id: CSSPropertyID,
    v: &Value,
) -> std::result::Result<PositionTryFallback, LonghandApplicationError> {
    if matches!(
        v.Payload(),
        CSSValuePayload::kValuePairClass(_) | CSSValuePayload::kIdentifierClass(_)
    ) {
        return Ok(PositionTryFallback::from_position_area(Area(id, v)?));
    }
    let CSSValuePayload::kValueListClass(list) = v.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    if list.separator != crate::production_css_value::ListSeparator::Space || list.values.is_empty()
    {
        return Err(LonghandApplicationError::InvalidValue(id));
    }
    let mut name = Member::default();
    let mut tactics = kNoTryTactics;
    let mut count = 0;
    // converter.cc:4219-4247; native TryTacticList has capacity 3, while the
    // source parser recognizes 5 flips. Keep the unsupported capacity explicit.
    for v in &list.values {
        if matches!(v.Payload(), CSSValuePayload::kCustomIdentClass(_)) {
            name = Name(id, v)?;
            continue;
        }
        if count == tactics.len() {
            return Err(LonghandApplicationError::Unsupported(id));
        }
        tactics[count] = match Identifier(id, v)? {
            CSSValueID::kFlipBlock => TryTactic::kFlipBlock,
            CSSValueID::kFlipInline => TryTactic::kFlipInline,
            CSSValueID::kFlipStart => TryTactic::kFlipStart,
            CSSValueID::kFlipX => TryTactic::kFlipX,
            CSSValueID::kFlipY => TryTactic::kFlipY,
            _ => return Err(LonghandApplicationError::InvalidValue(id)),
        };
        count += 1;
    }
    Ok(PositionTryFallback::from_name(name, tactics))
}
pub(super) fn Apply(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
) -> Result {
    use CSSPropertyID::*;
    let inherit = v.IsInheritedValue() || v.IsUnsetValue() && CSSProperty::Get(id).IsInherited();
    let initial = v.IsInitialValue() || v.IsUnsetValue() && !inherit || inherit && parent.is_none();
    let inherited = parent.filter(|_| inherit && !initial);
    let none = matches!(v.Payload(),CSSValuePayload::kIdentifierClass(k) if k.0==CSSValueID::kNone);
    match id {
        kAnchorName => {
            let names = if initial || none {
                Member::default()
            } else if let Some(p) = inherited {
                p.AnchorName().clone()
            } else {
                Names(id, v)?
            };
            b.SetAnchorNameOwned(names);
        }
        kAnchorScope => {
            let scope = if initial || none {
                StyleNameScope::default()
            } else if let Some(p) = inherited {
                p.AnchorScope().clone()
            } else {
                ConvertNameScope(id, v)?
            };
            b.SetAnchorScopeOwned(scope);
        }
        // longhands_custom.cc:252-265; state.cc:383-394. Native data is real;
        // the AnchorEvaluator / conversion AnchorData owner remains a collaborator.
        kPositionAnchor => {
            let anchor = if initial {
                StylePositionAnchor::Initial()
            } else if let Some(p) = inherited {
                p.PositionAnchor().clone()
            } else if matches!(v.Payload(), CSSValuePayload::kIdentifierClass(_)) {
                StylePositionAnchor::new_type(match Identifier(id, v)? {
                    CSSValueID::kAuto => Type::kAuto,
                    CSSValueID::kNone => Type::kNone,
                    CSSValueID::kNormal => Type::kNormal,
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                })
            } else {
                StylePositionAnchor::new_name(Name(id, v)?)
            };
            b.SetPositionAnchorOwned(anchor);
        }
        // state.cc:396-429: a changed non-none area marks HasAnchorFunctions.
        // No AnchorEvaluator means no fabricated layout offsets or AnchorData.
        kPositionArea => {
            let val = if initial {
                PositionArea::default()
            } else if let Some(p) = inherited {
                *p.GetPositionArea()
            } else {
                Area(id, v)?
            };
            if b.GetPositionArea() != &val {
                b.SetPositionArea(&val);
                if !val.IsNone() {
                    b.SetHasAnchorFunctions();
                }
            }
        }
        kPositionVisibility => {
            let flags = if initial {
                PositionVisibility::kAnchorsVisible
            } else if let Some(p) = inherited {
                p.GetPositionVisibility()
            } else {
                let mut flags = PositionVisibility::kAlways;
                let mut process = |v: &Value| -> Result {
                    flags |= match Identifier(id, v)? {
                        CSSValueID::kAlways => PositionVisibility::kAlways,
                        CSSValueID::kAnchorsVisible => PositionVisibility::kAnchorsVisible,
                        CSSValueID::kNoOverflow => PositionVisibility::kNoOverflow,
                        _ => return Err(LonghandApplicationError::InvalidValue(id)),
                    };
                    Ok(())
                };
                if let CSSValuePayload::kValueListClass(list) = v.Payload() {
                    for v in &list.values {
                        process(v)?;
                    }
                } else {
                    process(v)?;
                }
                flags
            };
            b.SetPositionVisibility(flags);
        }
        kPositionTryFallbacks => {
            let fallbacks = if initial || none {
                Member::default()
            } else if let Some(p) = inherited {
                p.GetPositionTryFallbacks().clone()
            } else {
                let CSSValuePayload::kValueListClass(list) = v.Payload() else {
                    return Err(LonghandApplicationError::InvalidValue(id));
                };
                if list.separator != crate::production_css_value::ListSeparator::Comma
                    || list.values.is_empty()
                {
                    return Err(LonghandApplicationError::InvalidValue(id));
                }
                let values = list
                    .values
                    .iter()
                    .map(|v| Fallback(id, v))
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                Member::from_ptr(MakeGarbageCollected(PositionTryFallbacks::new(
                    HeapVector::from(values),
                )))
            };
            b.SetPositionTryFallbacksOwned(fallbacks);
        }
        kPositionTryOrder => {
            let order = if initial {
                EPositionTryOrder::kNormal
            } else if let Some(p) = inherited {
                p.PositionTryOrder()
            } else {
                match Identifier(id, v)? {
                    CSSValueID::kNormal => EPositionTryOrder::kNormal,
                    CSSValueID::kMostWidth => EPositionTryOrder::kMostWidth,
                    CSSValueID::kMostHeight => EPositionTryOrder::kMostHeight,
                    CSSValueID::kMostBlockSize => EPositionTryOrder::kMostBlockSize,
                    CSSValueID::kMostInlineSize => EPositionTryOrder::kMostInlineSize,
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                }
            };
            b.SetPositionTryOrder(order);
        }
        _ => return Err(LonghandApplicationError::Unsupported(id)),
    }
    if inherit && !initial {
        b.SetHasExplicitInheritance();
        parent.unwrap().SetChildHasExplicitInheritance();
    }
    Ok(())
}
