// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Native animation name/composition and reset-only timeline/range application.
#![allow(non_snake_case)]
use super::*;
use crate::production_css_value::ListSeparator;
use foundation::{AtomicString, MakeGarbageCollected, Member, ScopedCSSName};
use layoutng_style::style::css_timing_data::{CompositeOperation, StyleTimeline};

pub(super) fn IsAnimationProperty(id: CSSPropertyID) -> bool {
    matches!(
        id,
        CSSPropertyID::kAnimationName
            | CSSPropertyID::kAnimationComposition
            | CSSPropertyID::kAnimationTimeline
            | CSSPropertyID::kAnimationRangeStart
            | CSSPropertyID::kAnimationRangeEnd
    )
}

// cpp: css_to_style_map.cc:319-334. The document-root cascade retains native
// ScopedCSSName value identity. A shadow owner/TreeScope population and
// SetHasTreeScopedReference are still missing collaborators of that cascade.
fn Name(
    id: CSSPropertyID,
    value: &Value,
) -> std::result::Result<Member<ScopedCSSName>, LonghandApplicationError> {
    let name = match value.Payload() {
        CSSValuePayload::kIdentifierClass(keyword) if keyword.0 == CSSValueID::kNone => {
            return Ok(Member::default())
        }
        CSSValuePayload::kCustomIdentClass(name) if name.property == CSSPropertyID::kInvalid => {
            name.name.clone()
        }
        CSSValuePayload::kStringClass(name) => {
            AtomicString::from_utf16(name.0.Span16().unwrap_or_default())
        }
        _ => return Err(LonghandApplicationError::InvalidValue(id)),
    };
    Ok(Member::from_ptr(MakeGarbageCollected(ScopedCSSName::new(
        &name,
        std::ptr::null(),
    ))))
}

pub(super) fn Apply(
    id: CSSPropertyID,
    builder: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    value: &Value,
    inherit: bool,
    initial: bool,
) -> Result {
    if initial {
        return ApplyInitial(id, builder);
    }
    if inherit {
        if value.IsInheritedValue() {
            builder.SetHasExplicitInheritance();
            parent.unwrap().SetChildHasExplicitInheritance();
        }
        return ApplyInherit(id, builder, parent.unwrap());
    }
    let CSSValuePayload::kValueListClass(list) = value.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    if list.separator != ListSeparator::Comma || list.values.is_empty() {
        return Err(LonghandApplicationError::InvalidValue(id));
    }
    use CSSPropertyID::*;
    use CSSValueID::*;
    match id {
        kAnimationName => {
            ApplyConvertedNameList(id, builder, &MapList(id, value, |item| Name(id, item))?)
        }
        // cpp: css_to_style_map.cc:439-451.
        kAnimationComposition => ApplyConvertedCompositionList(
            id,
            builder,
            &MapList(id, value, |item| {
                let CSSValuePayload::kIdentifierClass(keyword) = item.Payload() else {
                    return Err(LonghandApplicationError::InvalidValue(id));
                };
                Ok(match keyword.0 {
                    kReplace => CompositeOperation::kCompositeReplace,
                    kAdd => CompositeOperation::kCompositeAdd,
                    kAccumulate => CompositeOperation::kCompositeAccumulate,
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                })
            })?,
        ),
        // cpp: css_to_style_map.cc:348-386. Native auto/none are real values;
        // named/scroll/view timelines retain the typed conversion boundary.
        kAnimationTimeline => ApplyConvertedTimelineList(
            id,
            builder,
            &MapList(id, value, |item| match item.Payload() {
                CSSValuePayload::kIdentifierClass(keyword)
                    if matches!(keyword.0, kAuto | kNone) =>
                {
                    Ok(StyleTimeline::Keyword(keyword.0))
                }
                _ => Err(LonghandApplicationError::Unsupported(id)),
            })?,
        ),
        // cpp: css_to_style_map.cc:403-433. `normal` resets actual native lists.
        kAnimationRangeStart | kAnimationRangeEnd => {
            let values = MapList(id, value, |item| match item.Payload() {
                CSSValuePayload::kIdentifierClass(keyword) if keyword.0 == kNormal => Ok(None),
                _ => Err(LonghandApplicationError::Unsupported(id)),
            })?;
            if id == kAnimationRangeStart {
                ApplyConvertedRangeStartList(id, builder, &values)
            } else {
                ApplyConvertedRangeEndList(id, builder, &values)
            }
        }
        _ => Err(LonghandApplicationError::Unsupported(id)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{
        css_parser_mode::CSSParserMode,
        production_property_parser::{ParseDeclarationList, ParseProperty, PropertyParseErrorKind},
    };
    use layoutng_style::style::computed_style_constants::{TimelineAxis, TimelineScroller};
    use layoutng_style::style::css_timing_data::{
        CSSAnimationData, TimelineNamedRange, TimelineOffset,
    };
    fn initial() -> &'static ComputedStyle {
        unsafe { &*ComputedStyle::GetInitialStyleSingleton() }
    }
    fn apply(b: &mut ComputedStyleBuilder, parent: Option<&ComputedStyle>, css: &str) {
        let parsed = ParseDeclarationList(
            &foundation::String::from(css),
            CSSParserMode::kHTMLStandardMode,
        );
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
        assert!(!parsed.properties.is_empty());
        for p in parsed.properties {
            super::super::Apply(
                p.PropertyID(),
                b,
                parent,
                p.Value(),
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
    fn production_animation_native_names_repeated_lists_and_typed_easing() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(
            &mut b,
            None,
            r#"animation-name:Fade,none,"none",Fade;animation-duration:calc(1s + 1s),500ms;animation-delay:-250ms;animation-iteration-count:2,infinite;animation-direction:alternate,reverse;animation-fill-mode:both,forwards,none;animation-play-state:paused;animation-composition:add,accumulate;animation-timing-function:steps(calc(1.4 + 1.4),jump-none),cubic-bezier(calc(.1 + .1),0,1,1),linear(calc(0) calc(20%),1 calc(80%))"#,
        );
        let d = data(&b);
        assert_eq!(d.NameList().len(), 4);
        assert!(d.NameList()[1].Get().is_null());
        assert_eq!(unsafe { &*d.NameList()[2].Get() }.GetName().Utf8(), "none");
        let sheet = crate::production_style_sheet::ParseStyleSheet(
            &foundation::String::from("@keyframes F\\61 de{from{opacity:0}to{opacity:1}}"),
            CSSParserMode::kHTMLStandardMode,
        );
        assert_eq!(
            unsafe { &*d.NameList()[0].Get() }.GetName(),
            &sheet.keyframes[0].rule.name
        );
        assert!(unsafe { &*d.NameList()[0].Get() }.GetTreeScope().is_null());
        assert_eq!(d.DurationList(), &[Some(2.0), Some(0.5)]);
        let timing = d.ConvertToTiming(3);
        assert_eq!(timing.iteration_duration, Some(0.5));
        assert_eq!(timing.start_delay.time_delay, -0.25);
        assert_eq!(timing.iteration_count, f64::INFINITY);
        assert_eq!(timing.direction, PlaybackDirection::REVERSE);
        assert_eq!(timing.fill_mode, FillMode::BOTH);
        assert!(matches!(
            &*timing.timing_function,
            TimingFunction::Steps {
                number_of_steps: 3,
                step_position: StepPosition::JUMP_NONE
            }
        ));
        assert_eq!(
            d.GetComposition(3),
            CompositeOperation::kCompositeAccumulate
        );
        assert!(
            matches!(&**d.TimingFunctionList().get(1).unwrap(),TimingFunction::CubicBezier(v) if v[0]==0.2)
        );
        assert!(
            matches!(&**d.TimingFunctionList().get(2).unwrap(),TimingFunction::Linear(v) if v.len()==2 && v[0].input==20.0 && v[1].input==80.0)
        );
        assert!(matches!(
            d.GetTimeline(3),
            StyleTimeline::Keyword(CSSValueID::kAuto)
        ));
    }
    #[test]
    fn production_animation_shorthand_resets_real_timeline_range_and_preserves_composition() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(&mut b, None, "animation-composition:add,accumulate");
        *b.AccessAnimations().TimelineListMut() = vec![StyleTimeline::Scroll {
            axis: TimelineAxis::kX,
            scroller: TimelineScroller::kRoot,
        }];
        let offset = TimelineOffset {
            name: TimelineNamedRange::kEntry,
            offset: Length::Percent(20.0),
            style_dependent_offset: None,
            zoom: Some(1.0),
        };
        *b.AccessAnimations().RangeStartListMut() = vec![Some(offset.clone())];
        *b.AccessAnimations().RangeEndListMut() = vec![Some(offset)];
        apply(&mut b,None,"animation:slide calc(2s) ease-in calc(-250ms) calc(2) alternate both paused,1s linear none");
        let d = data(&b);
        assert_eq!(d.NameList().len(), 2);
        assert_eq!(d.DurationList(), &[Some(2.0), Some(1.0)]);
        assert_eq!(d.IterationCountList(), &[2.0, 1.0]);
        assert_eq!(
            d.PlayStateList(),
            &[EAnimPlayState::kPaused, EAnimPlayState::kPlaying]
        );
        assert_eq!(
            d.TimelineList(),
            &[StyleTimeline::Keyword(CSSValueID::kAuto)]
        );
        assert!(d.RangeStartList().len() == 1 && d.RangeStartList()[0].is_none());
        assert!(d.RangeEndList().len() == 1 && d.RangeEndList()[0].is_none());
        assert_eq!(
            d.CompositionList(),
            &[
                CompositeOperation::kCompositeAdd,
                CompositeOperation::kCompositeAccumulate
            ]
        );
    }
    #[test]
    fn production_animation_css_wide_inheritance_and_copy_on_write() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut p = ComputedStyleBuilder::from_style(initial());
        apply(
            &mut p,
            None,
            "animation:slide 2s ease 1s 3 reverse both paused;animation-composition:accumulate",
        );
        let p = unsafe { &*p.TakeStyle() };
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(
            &mut b,
            Some(p),
            "animation:inherit;animation-composition:inherit",
        );
        assert!(b.HasExplicitInheritance());
        assert!(data(&b).NamesMatch(unsafe { &*p.Animations().Get() }));
        assert_eq!(
            data(&b).GetComposition(0),
            CompositeOperation::kCompositeAccumulate
        );
        assert_eq!(data(&b).ConvertToTiming(0).iteration_count, 3.0);
        apply(
            &mut b,
            Some(p),
            "animation:unset;animation-composition:initial",
        );
        assert!(data(&b).NameList()[0].Get().is_null());
        assert_eq!(data(&b).DurationList(), &[None]);
        assert_eq!(data(&b).DelayStartList()[0].time_delay, 0.0);
        assert_eq!(data(&b).IterationCountList(), &[1.0]);
        assert_eq!(
            data(&b).GetComposition(0),
            CompositeOperation::kCompositeReplace
        );
        assert_eq!(
            unsafe { &*p.Animations().Get() }.DurationList(),
            &[Some(2.0)]
        );
        b.SetEffectiveZoom(2.0);
        let value = ParseProperty(
            CSSPropertyID::kAnimationRangeStart,
            &foundation::String::from("inherit"),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap();
        assert_eq!(
            super::super::Apply(
                CSSPropertyID::kAnimationRangeStart,
                &mut b,
                Some(p),
                value[0].Value(),
                16.0,
                &MediaValuesCachedData::default()
            ),
            Err(LonghandApplicationError::Unsupported(
                CSSPropertyID::kAnimationRangeStart
            ))
        );
        let mut empty = ComputedStyleBuilder::from_style(initial());
        apply(
            &mut empty,
            None,
            "animation:initial;animation-composition:unset",
        );
        assert!(empty.Animations().Get().is_null());
    }
    #[test]
    fn production_animation_invalid_and_uncovered_functions_remain_typed() {
        for (id, css, kind) in [
            (
                CSSPropertyID::kAnimation,
                "slide -1s -2s",
                PropertyParseErrorKind::Invalid,
            ),
            (
                CSSPropertyID::kAnimationComposition,
                "add replace",
                PropertyParseErrorKind::Invalid,
            ),
            (
                CSSPropertyID::kAnimationTimingFunction,
                "steps(calc(1),jump-none)",
                PropertyParseErrorKind::Invalid,
            ),
            (
                CSSPropertyID::kAnimationTimingFunction,
                "linear(0 calc(20%) calc(30%) calc(40%),1)",
                PropertyParseErrorKind::Invalid,
            ),
            (
                CSSPropertyID::kAnimationTimingFunction,
                "cubic-bezier(calc(2),0,1,1)",
                PropertyParseErrorKind::Invalid,
            ),
            (
                CSSPropertyID::kAnimationDuration,
                "round(1s,1s)",
                PropertyParseErrorKind::Unsupported,
            ),
            (
                CSSPropertyID::kAnimationName,
                "ident(foo)",
                PropertyParseErrorKind::Unsupported,
            ),
        ] {
            let error = ParseProperty(
                id,
                &foundation::String::from(css),
                false,
                CSSParserMode::kHTMLStandardMode,
            )
            .err()
            .unwrap_or_else(|| panic!("accepted {css}"));
            assert_eq!(error.kind, kind, "{css}");
        }
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        for (id, css) in [
            (CSSPropertyID::kAnimationTimeline, "--named"),
            (CSSPropertyID::kAnimationRangeStart, "entry 20%"),
        ] {
            let value = ParseProperty(
                id,
                &foundation::String::from(css),
                false,
                CSSParserMode::kHTMLStandardMode,
            )
            .unwrap();
            assert_eq!(
                super::super::Apply(
                    id,
                    &mut b,
                    None,
                    value[0].Value(),
                    16.0,
                    &MediaValuesCachedData::default()
                ),
                Err(LonghandApplicationError::Unsupported(id))
            );
        }
        assert!(b.Animations().Get().is_null());
    }
}
