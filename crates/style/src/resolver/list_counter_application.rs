// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Native counter directive/list, QuotesData and ListStyleTypeData application.
#![allow(non_snake_case)]
use super::*;
use foundation::{Member, QuotesData, ScopedRefPtr};
use layoutng_style::style::{
    counter_directives::{CounterPropertyEntry, CounterPropertyList},
    list_style_type_data::ListStyleTypeData,
};

pub(super) fn IsListCounterProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kCounterIncrement
            | kCounterReset
            | kCounterSet
            | kQuotes
            | kListStyleType
            | kListStyleImage
            | kListStylePosition
    )
}
fn SetList(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    list: Option<Box<CounterPropertyList>>,
) {
    match id {
        CSSPropertyID::kCounterIncrement => b.SetCounterIncrementList(list),
        CSSPropertyID::kCounterReset => b.SetCounterResetList(list),
        _ => b.SetCounterSetList(list),
    }
}
fn Clear(id: CSSPropertyID, b: &mut ComputedStyleBuilder) {
    match id {
        CSSPropertyID::kCounterIncrement => b.ClearIncrementDirectives(),
        CSSPropertyID::kCounterReset => b.ClearResetDirectives(),
        _ => b.ClearSetDirectives(),
    }
}
fn Number(
    id: CSSPropertyID,
    b: &ComputedStyleBuilder,
    value: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> std::result::Result<f64, LonghandApplicationError> {
    match value.Payload() {
        CSSValuePayload::kNumericLiteralClass(n)
            if matches!(n.GetType(), UnitType::kInteger | UnitType::kNumber) =>
        {
            Ok(n.DoubleValue())
        }
        CSSValuePayload::kMathFunctionClass(math)
            if math.Category()
                == crate::css_math_expression_node::CalculationResultCategory::Number =>
        {
            math.ComputeValue(
                &mut super::MathLengthResolver(
                    id,
                    b.GetFontDescription().ComputedSize(),
                    root,
                    b.EffectiveZoom(),
                    media,
                ),
                None,
            )
            .map_err(|_| LonghandApplicationError::Unsupported(id))
        }
        _ => Err(LonghandApplicationError::InvalidValue(id)),
    }
}
// cpp: generated longhands.cc:7303-7357,7378-7437,7458-7512.
fn Counters(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    value: &Value,
    root: f32,
    media: &MediaValuesCachedData,
    inherit: bool,
    initial: bool,
) -> Result {
    if initial {
        Clear(id, b);
        SetList(id, b, None);
        return Ok(());
    }
    if inherit {
        let parent = parent.unwrap();
        Clear(id, b);
        if let Some(parent_map) = unsafe { parent.GetCounterDirectivesMap().as_ref() } {
            let map = b.AccessCounterDirectives();
            for (name, directives) in parent_map.iter() {
                let target = map.entry(name.clone()).or_default();
                match id {
                    CSSPropertyID::kCounterIncrement => target.InheritIncrement(directives),
                    CSSPropertyID::kCounterReset => target.InheritReset(directives),
                    _ => target.InheritSet(directives),
                }
            }
        }
        let list = match id {
            CSSPropertyID::kCounterIncrement => parent.CounterIncrementList(),
            CSSPropertyID::kCounterReset => parent.CounterResetList(),
            _ => parent.CounterSetList(),
        };
        SetList(
            id,
            b,
            unsafe { list.as_ref() }.map(CounterPropertyList::Clone),
        );
        b.SetHasExplicitInheritance();
        parent.SetChildHasExplicitInheritance();
        return Ok(());
    }
    if matches!(value.Payload(), CSSValuePayload::kIdentifierClass(k) if k.0 == CSSValueID::kNone) {
        Clear(id, b);
        SetList(id, b, None);
        return Ok(());
    }
    let CSSValuePayload::kValueListClass(list) = value.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    if list.separator != crate::production_css_value::ListSeparator::Space || list.values.is_empty()
    {
        return Err(LonghandApplicationError::InvalidValue(id));
    }
    // Resolve the typed entries before changing native state.
    let mut entries = Vec::new();
    for item in &list.values {
        let CSSValuePayload::kCounterClass(counter) = item.Payload() else {
            return Err(LonghandApplicationError::InvalidValue(id));
        };
        let CSSValuePayload::kCustomIdentClass(name) = counter.identifier.Payload() else {
            return Err(LonghandApplicationError::InvalidValue(id));
        };
        if counter.is_reversed {
            return Err(if id == CSSPropertyID::kCounterReset {
                LonghandApplicationError::Unsupported(id)
            } else {
                LonghandApplicationError::InvalidValue(id)
            });
        }
        let number = counter
            .value
            .as_ref()
            .map(|value| Number(id, b, value, root, media))
            .transpose()?;
        entries.push((name.name.clone(), number));
    }
    Clear(id, b);
    let map = b.AccessCounterDirectives();
    let mut native_list = CounterPropertyList::default();
    for (name, number) in entries {
        let target = map.entry(name.clone()).or_default();
        let mut entry = CounterPropertyEntry {
            name,
            ..Default::default()
        };
        if let Some(number) = number {
            match id {
                CSSPropertyID::kCounterIncrement => {
                    let integer = number as i32;
                    target.AddIncrementValue(integer);
                    entry.value = Some(integer);
                }
                CSSPropertyID::kCounterReset => {
                    target.SetResetValue(number as i64);
                    entry.value = target.ResetValue();
                }
                _ => {
                    let integer = number as i32;
                    target.SetSetValue(integer);
                    entry.value = Some(integer);
                }
            }
        } else {
            entry.value = Some(if id == CSSPropertyID::kCounterIncrement {
                1
            } else {
                0
            });
        }
        native_list.push(entry);
    }
    SetList(id, b, Some(Box::new(native_list)));
    Ok(())
}
// cpp: StyleBuilderConverter::ConvertQuotes, style_builder_converter.cc:2554-2571.
fn Quotes(
    value: &Value,
) -> std::result::Result<ScopedRefPtr<QuotesData>, LonghandApplicationError> {
    let invalid = || LonghandApplicationError::InvalidValue(CSSPropertyID::kQuotes);
    match value.Payload() {
        CSSValuePayload::kIdentifierClass(k) if k.0 == CSSValueID::kAuto => {
            Ok(ScopedRefPtr::default())
        }
        CSSValuePayload::kIdentifierClass(k) if k.0 == CSSValueID::kNone => {
            Ok(QuotesData::Create())
        }
        CSSValuePayload::kValueListClass(list)
            if list.separator == crate::production_css_value::ListSeparator::Space
                && !list.values.is_empty()
                && list.values.len() % 2 == 0 =>
        {
            let mut quotes = QuotesData::default();
            for pair in list.values.chunks_exact(2) {
                let (CSSValuePayload::kStringClass(open), CSSValuePayload::kStringClass(close)) =
                    (pair[0].Payload(), pair[1].Payload())
                else {
                    return Err(invalid());
                };
                quotes.AddPair((open.0.clone(), close.0.clone()));
            }
            Ok(ScopedRefPtr::new(quotes))
        }
        _ => Err(invalid()),
    }
}
pub(super) fn Apply(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    value: &Value,
    root: f32,
    media: &MediaValuesCachedData,
    images: Option<&dyn URLImageResolver>,
) -> Result {
    let inherit =
        value.IsInheritedValue() || value.IsUnsetValue() && CSSProperty::Get(id).IsInherited();
    let initial =
        value.IsInitialValue() || value.IsUnsetValue() && !inherit || inherit && parent.is_none();
    use CSSPropertyID::*;
    if matches!(id, kCounterIncrement | kCounterReset | kCounterSet) {
        return Counters(id, b, parent, value, root, media, inherit, initial);
    }
    match id {
        // generated longhands.cc:13490-13498.
        kQuotes => b.SetQuotes(if initial {
            ScopedRefPtr::default()
        } else if inherit {
            parent.unwrap().Quotes().clone()
        } else {
            Quotes(value)?
        }),
        // longhands_custom.cc:6654-6695; scope-dependent names retain Unsupported.
        kListStyleType => {
            let pointer = if initial {
                ComputedStyleInitialValues::InitialListStyleType().unwrap_or(std::ptr::null_mut())
            } else if inherit {
                parent.unwrap().ListStyleType().Get()
            } else {
                match value.Payload() {
                    CSSValuePayload::kIdentifierClass(k) if k.0 == CSSValueID::kNone => {
                        std::ptr::null_mut()
                    }
                    CSSValuePayload::kStringClass(text) => ListStyleTypeData::CreateString(
                        &foundation::AtomicString::from_utf16(text.0.Span16().unwrap_or_default()),
                    ),
                    CSSValuePayload::kCustomIdentClass(name) => {
                        let name_text = StringFromAtomic(&name.name);
                        if !matches!(
                            name_text.as_str(),
                            "decimal"
                                | "disc"
                                | "square"
                                | "circle"
                                | "disclosure-open"
                                | "disclosure-closed"
                        ) {
                            return Err(LonghandApplicationError::Unsupported(id));
                        }
                        ListStyleTypeData::CreateCounterStyle(&name.name, std::ptr::null())
                    }
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                }
            };
            b.SetListStyleType(&Member::from_ptr(pointer));
        }
        // generated longhands.cc:10165-10175; custom:6589-6594.
        kListStyleImage => {
            if inherit && !initial && b.EffectiveZoom() != parent.unwrap().EffectiveZoom() {
                return Err(LonghandApplicationError::Unsupported(id));
            }
            let pointer = if initial {
                std::ptr::null_mut()
            } else if inherit {
                parent.unwrap().ListStyleImage().Get()
            } else {
                super::ResolveStyleImage(id, value, images)?
            };
            b.SetListStyleImage(&Member::from_ptr(pointer));
        }
        // generated longhands.cc:10196-10207.
        kListStylePosition => {
            use foundation::EListStylePosition;
            let position = if initial {
                EListStylePosition::kOutside
            } else if inherit {
                parent.unwrap().ListStylePosition()
            } else {
                match value.Payload() {
                    CSSValuePayload::kIdentifierClass(k) if k.0 == CSSValueID::kInside => {
                        EListStylePosition::kInside
                    }
                    CSSValuePayload::kIdentifierClass(k) if k.0 == CSSValueID::kOutside => {
                        EListStylePosition::kOutside
                    }
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                }
            };
            b.SetListStylePosition(position);
            b.SetListStylePositionIsInherited(inherit && !initial);
        }
        _ => return Err(LonghandApplicationError::Unsupported(id)),
    }
    if inherit && !initial {
        b.SetHasExplicitInheritance();
        parent.unwrap().SetChildHasExplicitInheritance();
    }
    Ok(())
}
fn StringFromAtomic(value: &foundation::AtomicString) -> std::string::String {
    foundation::String::from_utf16(value.utf16_units().unwrap_or_default()).Utf8()
}
