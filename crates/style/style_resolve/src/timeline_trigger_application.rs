// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! css_to_style_map.cc:740-800; generated longhands.cc:16994-17300.
#![allow(non_snake_case)]
use super::*;
use layoutng_style::style::css_timing_data::{
    CSSAnimationData, StyleTimeline, TimelineNamedRange, TimelineOffset, TimelineOffsetOrAuto,
};
pub(super) fn IsProperty(id: CSSPropertyID) -> bool {
    matches!(
        id,
        CSSPropertyID::kTimelineTriggerName
            | CSSPropertyID::kTimelineTriggerSource
            | CSSPropertyID::kTimelineTriggerActivationRangeStart
            | CSSPropertyID::kTimelineTriggerActivationRangeEnd
            | CSSPropertyID::kTimelineTriggerActiveRangeStart
            | CSSPropertyID::kTimelineTriggerActiveRangeEnd
    )
}
// cpp: css_to_style_map.cc:403-429. Shared animation range typed/native bridge.
pub(super) fn Range(
    id: CSSPropertyID,
    v: &Value,
    b: &ComputedStyleBuilder,
    root: f32,
    media: &MediaValuesCachedData,
    default: f64,
) -> std::result::Result<Option<TimelineOffset>, LonghandApplicationError> {
    if matches!(v.Payload(),CSSValuePayload::kIdentifierClass(k) if k.0==CSSValueID::kNormal)
        || v.IsInitialValue()
    {
        return Ok(None);
    }
    let CSSValuePayload::kValueListClass(l) = v.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    if l.values.is_empty()
        || l.values.len() > 2
        || l.separator != crate::production_css_value::ListSeparator::Space
    {
        return Err(LonghandApplicationError::InvalidValue(id));
    }
    let mut name = TimelineNamedRange::kNone;
    let mut offset = Length::Percent(default);
    let mut numeric = Some(l.values[0].as_ref());
    if let CSSValuePayload::kIdentifierClass(k) = l.values[0].Payload() {
        use CSSValueID::*;
        name = match k.0 {
            kCover => TimelineNamedRange::kCover,
            kContain => TimelineNamedRange::kContain,
            kEntry => TimelineNamedRange::kEntry,
            kEntryCrossing => TimelineNamedRange::kEntryCrossing,
            kExit => TimelineNamedRange::kExit,
            kExitCrossing => TimelineNamedRange::kExitCrossing,
            kScroll => TimelineNamedRange::kScroll,
            _ => return Err(LonghandApplicationError::InvalidValue(id)),
        };
        numeric = l.values.get(1).map(|v| v.as_ref());
    } else if l.values.len() != 1 {
        return Err(LonghandApplicationError::InvalidValue(id));
    }
    if let Some(v) = numeric {
        offset =
            crate::resolver::position_repeat_application::ConvertLength(id, v, b, root, media)?;
    }
    Ok(Some(TimelineOffset {
        name,
        offset,
        style_dependent_offset: None,
        zoom: None,
    }))
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
    let initial = v.IsInitialValue() || v.IsUnsetValue() || inherit && parent.is_none();
    let parent_data = parent.map(|p| p.Animations());
    let parent_data = parent_data
        .as_ref()
        .filter(|d| !d.Get().is_null())
        .map(|d| unsafe { &*d.Get() });
    if (initial || inherit && parent_data.is_none()) && b.Animations().Get().is_null() {
        return Ok(());
    }
    if inherit && !initial {
        b.SetHasExplicitInheritance();
        parent.unwrap().SetChildHasExplicitInheritance();
        if !matches!(id, kTimelineTriggerName | kTimelineTriggerSource)
            && parent.unwrap().EffectiveZoom() != b.EffectiveZoom()
        {
            return Err(LonghandApplicationError::Unsupported(id));
        }
    }
    let reset = initial || inherit && parent_data.is_none();
    let items = if reset || inherit {
        None
    } else {
        let CSSValuePayload::kValueListClass(l) = v.Payload() else {
            return Err(LonghandApplicationError::InvalidValue(id));
        };
        if l.separator != crate::production_css_value::ListSeparator::Comma || l.values.is_empty() {
            return Err(LonghandApplicationError::InvalidValue(id));
        }
        Some(&l.values)
    };
    macro_rules! list {
        ($get:ident,$set:ident,$default:ident,$convert:expr) => {{
            let values = if reset {
                vec![CSSAnimationData::$default()]
            } else if inherit {
                parent_data
                    .unwrap()
                    .$get()
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
            } else {
                items
                    .unwrap()
                    .iter()
                    .map($convert)
                    .collect::<std::result::Result<Vec<_>, LonghandApplicationError>>()?
            };
            *b.AccessAnimations().$set() = values.into();
            Ok(())
        }};
    }
    match id {
        kTimelineTriggerName => list!(
            TimelineTriggerNameList,
            TimelineTriggerNameListMut,
            InitialTimelineTriggerName,
            |item: &std::rc::Rc<Value>| {
                if item.IsInitialValue()
                    || matches!(item.Payload(),CSSValuePayload::kIdentifierClass(k) if k.0==CSSValueID::kNone)
                {
                    Ok(foundation::Member::default())
                } else {
                    super::anchor_application::Name(id, item)
                }
            }
        ),
        kTimelineTriggerSource => list!(
            TimelineTriggerSourceList,
            TimelineTriggerSourceListMut,
            InitialTimelineTriggerSource,
            |item: &std::rc::Rc<Value>| {
                if item.IsInitialValue() {
                    return Ok(CSSAnimationData::InitialTimelineTriggerSource());
                }
                match item.Payload() {
                    CSSValuePayload::kIdentifierClass(k)
                        if matches!(k.0, CSSValueID::kNone | CSSValueID::kAuto) =>
                    {
                        Ok(StyleTimeline::Keyword(k.0))
                    }
                    CSSValuePayload::kCustomIdentClass(n)
                        if n.property == CSSPropertyID::kInvalid =>
                    {
                        Ok(StyleTimeline::Name(n.name.clone()))
                    }
                    _ => Err(LonghandApplicationError::Unsupported(id)),
                }
            }
        ),
        kTimelineTriggerActivationRangeStart => list!(
            TimelineTriggerActivationRangeStartList,
            TimelineTriggerActivationRangeStartListMut,
            InitialTimelineTriggerActivationRangeStart,
            |item: &std::rc::Rc<Value>| Range(id, item, b, root, media, 0.0)
        ),
        kTimelineTriggerActivationRangeEnd => list!(
            TimelineTriggerActivationRangeEndList,
            TimelineTriggerActivationRangeEndListMut,
            InitialTimelineTriggerActivationRangeEnd,
            |item: &std::rc::Rc<Value>| Range(id, item, b, root, media, 100.0)
        ),
        kTimelineTriggerActiveRangeStart => list!(
            TimelineTriggerActiveRangeStartList,
            TimelineTriggerActiveRangeStartListMut,
            InitialTimelineTriggerActiveRangeStart,
            |item: &std::rc::Rc<Value>| Active(id, item, b, root, media, 0.0)
        ),
        kTimelineTriggerActiveRangeEnd => list!(
            TimelineTriggerActiveRangeEndList,
            TimelineTriggerActiveRangeEndListMut,
            InitialTimelineTriggerActiveRangeEnd,
            |item: &std::rc::Rc<Value>| Active(id, item, b, root, media, 100.0)
        ),
        _ => Err(LonghandApplicationError::Unsupported(id)),
    }
}
fn Active(
    id: CSSPropertyID,
    v: &Value,
    b: &ComputedStyleBuilder,
    root: f32,
    media: &MediaValuesCachedData,
    default: f64,
) -> std::result::Result<TimelineOffsetOrAuto, LonghandApplicationError> {
    if v.IsInitialValue()
        || matches!(v.Payload(),CSSValuePayload::kIdentifierClass(k) if k.0==CSSValueID::kAuto)
    {
        return Ok(TimelineOffsetOrAuto::default());
    }
    Ok(TimelineOffsetOrAuto::new(Range(
        id, v, b, root, media, default,
    )?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{
        css_parser_mode::CSSParserMode,
        production_property_parser::{ParseDeclarationList, ParseProperty, PropertyParseErrorKind},
    };
    fn initial() -> &'static ComputedStyle {
        unsafe { &*ComputedStyle::GetInitialStyleSingleton() }
    }
    fn apply(b: &mut ComputedStyleBuilder, p: Option<&ComputedStyle>, css: &str) {
        let d = ParseDeclarationList(
            &foundation::String::from(css),
            CSSParserMode::kHTMLStandardMode,
        );
        assert!(d.errors.is_empty(), "{:?}", d.errors);
        for v in d.properties {
            super::super::Apply(
                v.PropertyID(),
                b,
                p,
                v.Value(),
                16.0,
                &MediaValuesCachedData::default(),
            )
            .unwrap();
        }
    }
    fn data(b: &ComputedStyleBuilder) -> &CSSAnimationData {
        unsafe { &*b.Animations().Get() }
    }
    #[test]
    fn production_timeline_trigger_native_lists_repetition_and_recalc() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(&mut b,None,"timeline-trigger-name:--a,--b,--c;timeline-trigger-source:--scroll,none;timeline-trigger-activation-range-start:entry 20%,calc(10px + 5%);timeline-trigger-activation-range-end:exit;timeline-trigger-active-range-start:auto,normal;timeline-trigger-active-range-end:contain 75%");
        let d = data(&b);
        assert_eq!(d.TimelineTriggerNameList().len(), 3);
        let n = unsafe { &*d.TimelineTriggerNameList()[0].Get() };
        assert!(n.GetName() == "--a");
        assert!(n.GetTreeScope().is_null());
        assert!(matches!(
            d.GetTimelineTriggerSource(2),
            StyleTimeline::Name(_)
        ));
        let repeated = layoutng_style::style::css_timing_data::CSSTimingData::GetRepeated(
            d.TimelineTriggerActivationRangeStartList(),
            2,
        )
        .as_ref()
        .unwrap();
        assert_eq!(repeated.name, TimelineNamedRange::kEntry);
        assert_eq!(repeated.offset, Length::Percent(20.0));
        assert_eq!(
            d.TimelineTriggerActivationRangeEndList()[0]
                .as_ref()
                .unwrap()
                .offset,
            Length::Percent(100.0)
        );
        assert!(d.TimelineTriggerActiveRangeStartList()[0].IsAuto());
        assert!(!d.TimelineTriggerActiveRangeStartList()[1].IsAuto());
        let mut copy = d.clone();
        assert!(copy.TriggersMatchForStyleRecalc(d));
        *copy.TimelineTriggerSourceListMut() = vec![StyleTimeline::Keyword(CSSValueID::kAuto)];
        assert!(!copy.AnimationsMatchForStyleRecalc(d));
        assert!(CSSAnimationData::TimelineTriggerDataChanged(
            Some(d),
            Some(&copy)
        ));
        assert!(CSSAnimationData::TimelineTriggerDataChanged(None, Some(d)));
        assert!(!CSSAnimationData::TimelineTriggerDataChanged(
            None,
            Some(&CSSAnimationData::default())
        ));
    }
    #[test]
    fn production_timeline_trigger_shorthand_resets_six_fields_and_implies_named_end() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(&mut b,None,"animation-duration:2s;timeline-trigger:--a --scroll entry 20% / contain 25%,--b auto 10% 80% / normal");
        let d = data(&b);
        assert_eq!(d.TimelineTriggerNameList().len(), 2);
        assert_eq!(
            d.TimelineTriggerActivationRangeEndList()[0]
                .as_ref()
                .unwrap()
                .offset,
            Length::Percent(100.0)
        );
        assert_eq!(
            d.TimelineTriggerActiveRangeEndList()[0]
                .GetTimelineOffset()
                .unwrap()
                .name,
            TimelineNamedRange::kContain
        );
        assert!(d.TimelineTriggerActiveRangeEndList()[1].IsAuto());
        apply(&mut b, None, "timeline-trigger:none");
        let d = data(&b);
        assert!(d.TimelineTriggerNameList()[0].Get().is_null());
        assert!(matches!(
            d.GetTimelineTriggerSource(0),
            StyleTimeline::Keyword(CSSValueID::kAuto)
        ));
        assert_eq!(d.TimelineTriggerActivationRangeStartList(), &[None]);
        assert!(d.TimelineTriggerActiveRangeStartList()[0].IsAuto());
        assert_eq!(d.DurationList(), &[Some(2.0)]);
        apply(&mut b,None,"timeline-trigger-activation-range:entry 30%,10%;timeline-trigger-active-range:normal,cover 15%");
        let d = data(&b);
        assert_eq!(d.TimelineTriggerActivationRangeEndList().len(), 2);
        assert!(d.TimelineTriggerActivationRangeEndList()[1].is_none());
        assert!(d.TimelineTriggerActiveRangeEndList()[0].IsAuto());
        assert_eq!(
            d.TimelineTriggerActiveRangeEndList()[1]
                .GetTimelineOffset()
                .unwrap()
                .offset,
            Length::Percent(100.0)
        );
    }
    #[test]
    fn production_timeline_trigger_css_wide_cow_and_same_zoom_boundary() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut p = ComputedStyleBuilder::from_style(initial());
        apply(
            &mut p,
            None,
            "timeline-trigger:--a --scroll entry 20% / normal 80%",
        );
        let p = unsafe { &*p.TakeStyle() };
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(&mut b, Some(p), "timeline-trigger:inherit");
        assert!(data(&b).AnimationsMatchForStyleRecalc(unsafe { &*p.Animations().Get() }));
        assert!(b.HasExplicitInheritance());
        apply(&mut b, Some(p), "timeline-trigger:unset");
        assert!(data(&b).TimelineTriggerNameList()[0].Get().is_null());
        assert!(
            !unsafe { &*p.Animations().Get() }.TimelineTriggerNameList()[0]
                .Get()
                .is_null()
        );
        let mut empty = ComputedStyleBuilder::from_style(initial());
        apply(&mut empty, None, "timeline-trigger:initial");
        assert!(empty.Animations().Get().is_null());
        b.SetEffectiveZoom(2.0);
        let v = ParseProperty(
            CSSPropertyID::kTimelineTriggerActivationRangeStart,
            &foundation::String::from("inherit"),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap();
        assert_eq!(
            super::super::Apply(
                v[0].PropertyID(),
                &mut b,
                Some(p),
                v[0].Value(),
                16.0,
                &MediaValuesCachedData::default()
            ),
            Err(LonghandApplicationError::Unsupported(
                CSSPropertyID::kTimelineTriggerActivationRangeStart
            ))
        );
    }
    #[test]
    fn production_timeline_trigger_stable_exposure_invalid_and_typed_collaborators() {
        use CSSPropertyID::*;
        for (id, text) in [
            (kTimelineTriggerName, "plain"),
            (kTimelineTriggerName, "auto"),
            (kTimelineTriggerActivationRangeStart, "auto"),
            (kTimelineTriggerSource, "ordinary"),
            (kTimelineTrigger, "--a /"),
            (kTimelineTriggerActiveRange, "auto nonsense"),
            (kTimelineTriggerName, "--a,"),
        ] {
            assert!(
                ParseProperty(
                    id,
                    &foundation::String::from(text),
                    false,
                    CSSParserMode::kHTMLStandardMode
                )
                .is_err(),
                "{id:?}: {text}"
            );
        }
        for (id, text) in [
            (kTimelineTriggerSource, "scroll()"),
            (kTimelineTriggerSource, "view()"),
            (kTimelineTriggerName, "ident(--a)"),
        ] {
            assert_eq!(
                ParseProperty(
                    id,
                    &foundation::String::from(text),
                    false,
                    CSSParserMode::kHTMLStandardMode
                )
                .err()
                .unwrap()
                .kind,
                PropertyParseErrorKind::Unsupported
            );
        }
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(&mut b, None, "timeline-trigger:--a auto normal / auto");
        assert!(matches!(
            data(&b).GetTimelineTriggerSource(0),
            StyleTimeline::Keyword(CSSValueID::kAuto)
        ));
    }
}
