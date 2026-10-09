use super::longhand_dispatch::*;
use foundation::{CSSPropertyID, CSSValueID, EPosition, EVisibility};
use layoutng_style::style::computed_style::{ComputedStyle, ComputedStyleBuilder};

fn apply_declarations(
    builder: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    css: &str,
) {
    let declarations = crate::parser::production_property_parser::ParseDeclarationList(
        &foundation::String::from(css),
        crate::parser::css_parser_mode::CSSParserMode::kHTMLStandardMode,
    );
    assert!(declarations.errors.is_empty(), "{:?}", declarations.errors);
    assert!(!declarations.properties.is_empty());
    for property in declarations.properties {
        crate::resolver::production_style_builder::Apply(
            property.PropertyID(),
            builder,
            parent,
            property.Value(),
            16.0,
            &crate::media_queries::MediaValuesCachedData::default(),
        )
        .unwrap();
    }
}

#[test]
fn typed_alignment_values_set_native_content_self_and_legacy_data() {
    use layoutng_style::style::computed_style_constants::*;
    let _heap = foundation::LayoutHeapScope::new();
    let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
    let mut builder = ComputedStyleBuilder::from_style(initial);
    apply_declarations(&mut builder, None, "align-content:space-evenly;justify-content:safe right;align-items:last baseline;align-self:unsafe self-end;justify-items:legacy left;justify-self:anchor-center");
    assert_eq!(
        builder.AlignContent().Distribution(),
        ContentDistributionType::kSpaceEvenly
    );
    assert_eq!(
        builder.AlignContent().GetPosition(),
        ContentPosition::kNormal
    );
    assert_eq!(
        builder.JustifyContent().GetPosition(),
        ContentPosition::kRight
    );
    assert_eq!(
        builder.JustifyContent().Overflow(),
        OverflowAlignment::kSafe
    );
    assert_eq!(
        builder.AlignItems().GetPosition(),
        ItemPosition::kLastBaseline
    );
    assert_eq!(builder.AlignSelf().GetPosition(), ItemPosition::kSelfEnd);
    assert_eq!(builder.AlignSelf().Overflow(), OverflowAlignment::kUnsafe);
    assert_eq!(
        builder.JustifyItems().PositionType(),
        ItemPositionType::kLegacy
    );
    assert_eq!(builder.JustifyItems().GetPosition(), ItemPosition::kLeft);
    assert_eq!(
        builder.JustifySelf().GetPosition(),
        ItemPosition::kAnchorCenter
    );
    let parent = unsafe { &*builder.TakeStyle() };
    let mut inherited = ComputedStyleBuilder::from_style(initial);
    apply_declarations(
        &mut inherited,
        Some(parent),
        "place-content:inherit;place-items:inherit;place-self:inherit",
    );
    assert!(inherited.HasExplicitInheritance());
    assert_eq!(inherited.AlignContent(), parent.AlignContent());
    assert_eq!(inherited.JustifyContent(), parent.JustifyContent());
    assert_eq!(inherited.AlignItems(), parent.AlignItems());
    assert_eq!(inherited.JustifyItems(), parent.JustifyItems());
    assert_eq!(inherited.AlignSelf(), parent.AlignSelf());
    assert_eq!(inherited.JustifySelf(), parent.JustifySelf());
    apply_declarations(
        &mut inherited,
        Some(parent),
        "place-content:initial;place-items:unset;place-self:initial",
    );
    assert_eq!(inherited.AlignContent(), initial.AlignContent());
    assert_eq!(inherited.JustifyContent(), initial.JustifyContent());
    assert_eq!(inherited.AlignItems(), initial.AlignItems());
    assert_eq!(inherited.JustifyItems(), initial.JustifyItems());
    assert_eq!(inherited.AlignSelf(), initial.AlignSelf());
    assert_eq!(inherited.JustifySelf(), initial.JustifySelf());
}

#[test]
fn place_shorthands_preserve_baseline_and_overflow_native_semantics() {
    use layoutng_style::style::computed_style_constants::*;
    let _heap = foundation::LayoutHeapScope::new();
    let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
    let mut builder = ComputedStyleBuilder::from_style(initial);
    apply_declarations(
        &mut builder,
        None,
        "place-content:first baseline;place-items:safe center;place-self:last baseline end",
    );
    assert_eq!(
        builder.AlignContent().GetPosition(),
        ContentPosition::kBaseline
    );
    assert_eq!(
        builder.JustifyContent().GetPosition(),
        ContentPosition::kStart
    );
    assert_eq!(builder.AlignItems().GetPosition(), ItemPosition::kCenter);
    assert_eq!(builder.AlignItems().Overflow(), OverflowAlignment::kSafe);
    assert_eq!(builder.JustifyItems().GetPosition(), ItemPosition::kCenter);
    assert_eq!(builder.JustifyItems().Overflow(), OverflowAlignment::kSafe);
    assert_eq!(
        builder.AlignSelf().GetPosition(),
        ItemPosition::kLastBaseline
    );
    assert_eq!(builder.JustifySelf().GetPosition(), ItemPosition::kEnd);
    apply_declarations(
        &mut builder,
        None,
        "place-content:space-around safe center;place-items:normal legacy right;place-self:auto",
    );
    assert_eq!(
        builder.AlignContent().Distribution(),
        ContentDistributionType::kSpaceAround
    );
    assert_eq!(
        builder.JustifyContent().GetPosition(),
        ContentPosition::kCenter
    );
    assert_eq!(
        builder.JustifyContent().Overflow(),
        OverflowAlignment::kSafe
    );
    assert_eq!(builder.AlignItems().GetPosition(), ItemPosition::kNormal);
    assert_eq!(
        builder.JustifyItems().PositionType(),
        ItemPositionType::kLegacy
    );
    assert_eq!(builder.JustifyItems().GetPosition(), ItemPosition::kRight);
    assert_eq!(builder.AlignSelf().GetPosition(), ItemPosition::kAuto);
    assert_eq!(builder.JustifySelf().GetPosition(), ItemPosition::kAuto);
}

#[test]
fn wrapping_and_flex_flow_shorthands_apply_native_values_and_reset_omitted_longhands() {
    use foundation::{TextWrapMode, TextWrapStyle};
    use layoutng_style::css::white_space::WhiteSpaceCollapse;
    use layoutng_style::style::computed_style_constants::*;
    let _heap = foundation::LayoutHeapScope::new();
    let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
    let mut builder = ComputedStyleBuilder::from_style(initial);
    apply_declarations(
        &mut builder,
        None,
        "white-space:pre;flex-flow:column wrap-reverse balance",
    );
    assert_eq!(
        builder.GetWhiteSpaceCollapse(),
        WhiteSpaceCollapse::kPreserve
    );
    assert_eq!(builder.GetTextWrapMode(), TextWrapMode::kNowrap);
    assert_eq!(builder.FlexDirection(), foundation::EFlexDirection::kColumn);
    assert_eq!(builder.FlexWrap().GetWrapMode(), FlexWrapMode::kWrapReverse);
    assert!(builder.FlexWrap().IsBalanced());
    apply_declarations(&mut builder, None, "text-wrap:balance");
    assert_eq!(builder.GetTextWrapMode(), TextWrapMode::kWrap);
    assert_eq!(builder.GetTextWrapStyle(), TextWrapStyle::kBalance);
    assert_eq!(
        builder.GetWhiteSpaceCollapse(),
        WhiteSpaceCollapse::kPreserve
    );
    apply_declarations(
        &mut builder,
        None,
        "white-space:preserve-breaks nowrap;text-wrap:nowrap;flex-flow:wrap",
    );
    assert_eq!(
        builder.GetWhiteSpaceCollapse(),
        WhiteSpaceCollapse::kPreserveBreaks
    );
    assert_eq!(builder.GetTextWrapMode(), TextWrapMode::kNowrap);
    assert_eq!(builder.GetTextWrapStyle(), TextWrapStyle::kAuto);
    assert_eq!(builder.FlexDirection(), foundation::EFlexDirection::kRow);
    assert_eq!(builder.FlexWrap().GetWrapMode(), FlexWrapMode::kWrap);
    assert!(!builder.FlexWrap().IsBalanced());
    apply_declarations(&mut builder, None, "flex-wrap:balance");
    assert_eq!(builder.FlexWrap().GetWrapMode(), FlexWrapMode::kWrap);
    assert!(builder.FlexWrap().IsBalanced());
    let parent = unsafe { &*builder.TakeStyle() };
    let mut inherited = ComputedStyleBuilder::from_style(initial);
    apply_declarations(
        &mut inherited,
        Some(parent),
        "white-space:inherit;text-wrap:inherit;flex-flow:inherit",
    );
    assert_eq!(
        inherited.GetWhiteSpaceCollapse(),
        parent.GetWhiteSpaceCollapse()
    );
    assert_eq!(inherited.GetTextWrapMode(), parent.GetTextWrapMode());
    assert_eq!(inherited.GetTextWrapStyle(), parent.GetTextWrapStyle());
    assert_eq!(inherited.FlexDirection(), parent.FlexDirection());
    assert_eq!(inherited.FlexWrap(), parent.FlexWrap());
}

#[test]
fn invalid_typed_alignment_and_flex_values_leave_native_style_unchanged() {
    use crate::production_css_value as values;
    let _heap = foundation::LayoutHeapScope::new();
    let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
    let mut builder = ComputedStyleBuilder::from_style(initial);
    for property in [
        CSSPropertyID::kAlignContent,
        CSSPropertyID::kAlignSelf,
        CSSPropertyID::kFlexWrap,
    ] {
        assert_eq!(
            crate::resolver::production_style_builder::Apply(
                property,
                &mut builder,
                None,
                &values::identifier(CSSValueID::kBlock),
                16.0,
                &crate::media_queries::MediaValuesCachedData::default()
            ),
            Err(LonghandApplicationError::InvalidValue(property))
        );
    }
    assert_eq!(builder.AlignContent(), initial.AlignContent());
    assert_eq!(builder.AlignSelf(), initial.AlignSelf());
    assert_eq!(builder.FlexWrap(), initial.FlexWrap());
}

#[test]
fn flex_basis_inherits_native_length_and_requires_cross_zoom_capability() {
    let _heap = foundation::LayoutHeapScope::new();
    let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
    let mut parent = ComputedStyleBuilder::from_style(initial);
    apply_declarations(&mut parent, None, "flex-basis:48px;flex-line-count:3");
    let parent = unsafe { &*parent.TakeStyle() };
    let mut builder = ComputedStyleBuilder::from_style(initial);
    apply_declarations(
        &mut builder,
        Some(parent),
        "flex-basis:inherit;flex-line-count:inherit",
    );
    assert_eq!(builder.FlexBasis().Pixels(), 48.0);
    assert_eq!(builder.FlexLineCount(), 3);
    assert!(builder.HasExplicitInheritance());
    builder.SetEffectiveZoom(2.0);
    assert_eq!(
        ApplyInherit(CSSPropertyID::kFlexBasis, &mut builder, parent),
        Err(LonghandApplicationError::Unsupported(
            CSSPropertyID::kFlexBasis
        ))
    );
    assert_eq!(builder.FlexBasis().Pixels(), 48.0);
    apply_declarations(
        &mut builder,
        Some(parent),
        "flex-basis:initial;flex-line-count:65536",
    );
    assert_eq!(builder.FlexBasis(), initial.FlexBasis());
    assert_eq!(builder.FlexLineCount(), u16::MAX);
    ApplyInitial(CSSPropertyID::kFlexLineCount, &mut builder).unwrap();
    assert_eq!(builder.FlexLineCount(), initial.FlexLineCount());
}

#[test]
fn animation_and_transition_lists_have_native_storage_and_copy_on_write() {
    use layoutng_style::style::computed_style_constants::EAnimPlayState;
    use layoutng_style::style::css_timing_data::*;
    let _heap = foundation::LayoutHeapScope::new();
    let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
    let mut parent = ComputedStyleBuilder::from_style(initial);
    // Initial/unset on a style with no data must preserve nullable native storage.
    apply_declarations(
        &mut parent,
        None,
        "animation-duration:initial;transition-delay:unset",
    );
    assert!(parent.Animations().Get().is_null());
    assert!(parent.Transitions().Get().is_null());
    let css = "animation-duration:auto,200ms;animation-delay:-500ms,1s;animation-iteration-count:2.5,infinite;animation-direction:alternate-reverse,normal;animation-fill-mode:both,backwards;animation-play-state:paused,running;animation-timing-function:cubic-bezier(.1,.2,.7,.8),steps(3,jump-none);transition-duration:1s,200ms;transition-delay:-300ms;transition-property:opacity,--progress,mystery;transition-behavior:allow-discrete,normal;transition-timing-function:linear(0, .5 40%, 1),ease-in";
    apply_declarations(&mut parent, None, css);
    let parent = unsafe { &*parent.TakeStyle() };
    let animation = unsafe { &*parent.Animations().Get() };
    assert_eq!(animation.DurationList(), &[None, Some(0.2)]);
    assert_eq!(animation.DelayStartList()[0].time_delay, -0.5);
    assert_eq!(animation.IterationCountList(), &[2.5, f64::INFINITY]);
    assert_eq!(
        animation.DirectionList(),
        &[
            PlaybackDirection::ALTERNATE_REVERSE,
            PlaybackDirection::NORMAL
        ]
    );
    assert_eq!(
        animation.FillModeList(),
        &[FillMode::BOTH, FillMode::BACKWARDS]
    );
    assert_eq!(
        animation.PlayStateList(),
        &[EAnimPlayState::kPaused, EAnimPlayState::kPlaying]
    );
    assert_eq!(
        **animation.TimingFunctionList().first().unwrap(),
        TimingFunction::CubicBezier([0.1, 0.2, 0.7, 0.8])
    );
    assert_eq!(
        *animation.TimingFunctionList()[1],
        TimingFunction::Steps {
            number_of_steps: 3,
            step_position: StepPosition::JUMP_NONE
        }
    );
    let transition = unsafe { &*parent.Transitions().Get() };
    assert_eq!(transition.DurationList(), &[Some(1.0), Some(0.2)]);
    assert_eq!(transition.DelayStartList()[0].time_delay, -0.3);
    assert_eq!(
        transition.PropertyList(),
        &[
            TransitionProperty::Known(CSSPropertyID::kOpacity),
            TransitionProperty::Unknown(foundation::AtomicString::from_str("--progress")),
            TransitionProperty::Unknown(foundation::AtomicString::from_str("mystery"))
        ]
    );
    assert_eq!(
        transition.BehaviorList(),
        &[
            TransitionBehavior::kAllowDiscrete,
            TransitionBehavior::kNormal
        ]
    );
    assert_eq!(
        *transition.TimingFunctionList()[0],
        TimingFunction::Linear(vec![
            LinearEasingPoint {
                input: 0.0,
                output: 0.0
            },
            LinearEasingPoint {
                input: 40.0,
                output: 0.5
            },
            LinearEasingPoint {
                input: 100.0,
                output: 1.0
            }
        ])
    );
    let mut independent = ComputedStyleBuilder::from_style(initial);
    apply_declarations(&mut independent, None, css);
    let independent = unsafe { &*independent.TakeStyle() };
    assert!(independent == parent); // deep equality includes distinct timing refs.
    let mut child = ComputedStyleBuilder::from_style(parent);
    apply_declarations(
        &mut child,
        Some(parent),
        "animation-duration:500ms;transition-property:none",
    );
    assert_ne!(child.Animations().Get(), parent.Animations().Get());
    assert_ne!(child.Transitions().Get(), parent.Transitions().Get());
    assert_eq!(
        unsafe { &*child.Animations().Get() }.DurationList(),
        &[Some(0.5)]
    );
    assert_eq!(
        unsafe { &*child.Transitions().Get() }.PropertyList(),
        &[TransitionProperty::None]
    );
    assert_eq!(animation.DurationList(), &[None, Some(0.2)]);
    assert_eq!(transition.PropertyList().len(), 3);
    apply_declarations(&mut child, Some(parent), "animation-duration:inherit;animation-delay:initial;transition-property:inherit;transition-duration:initial");
    assert_eq!(
        unsafe { &*child.Animations().Get() }.DurationList(),
        animation.DurationList()
    );
    assert_eq!(
        unsafe { &*child.Animations().Get() }.DelayStartList(),
        &[TimingDelay::default()]
    );
    assert_eq!(
        unsafe { &*child.Transitions().Get() }.PropertyList(),
        transition.PropertyList()
    );
    assert_eq!(
        unsafe { &*child.Transitions().Get() }.DurationList(),
        &[Some(0.0)]
    );
}

#[test]
fn invalid_timing_list_conversion_does_not_allocate_or_mutate_native_storage() {
    use crate::production_css_value as values;
    let _heap = foundation::LayoutHeapScope::new();
    let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
    let mut builder = ComputedStyleBuilder::from_style(initial);
    let value = values::list(
        vec![
            values::numeric(1.0, crate::css_primitive_value::UnitType::kSeconds),
            values::identifier(CSSValueID::kBlock),
        ],
        values::ListSeparator::Comma,
    );
    assert_eq!(
        crate::resolver::production_style_builder::Apply(
            CSSPropertyID::kAnimationDelay,
            &mut builder,
            None,
            &value,
            16.0,
            &crate::media_queries::MediaValuesCachedData::default()
        ),
        Err(LonghandApplicationError::InvalidValue(
            CSSPropertyID::kAnimationDelay
        ))
    );
    assert!(builder.Animations().Get().is_null());
}

#[test]
fn generated_longhand_application_changes_real_computed_style() {
    let _heap = foundation::LayoutHeapScope::new();
    let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
    let mut builder = ComputedStyleBuilder::from_style(initial);
    ApplyIdentifier(
        CSSPropertyID::kPosition,
        &mut builder,
        CSSValueID::kAbsolute,
    )
    .unwrap();
    ApplyNumber(CSSPropertyID::kOpacity, &mut builder, 0.25).unwrap();
    assert_eq!(builder.GetPosition(), EPosition::kAbsolute);
    assert_eq!(builder.Opacity(), 0.25);
    ApplyInitial(CSSPropertyID::kPosition, &mut builder).unwrap();
    assert_eq!(builder.GetPosition(), EPosition::kStatic);
    let mut parent = ComputedStyleBuilder::from_style(initial);
    ApplyIdentifier(CSSPropertyID::kVisibility, &mut parent, CSSValueID::kHidden).unwrap();
    let parent = unsafe { &*parent.TakeStyle() };
    ApplyInherit(CSSPropertyID::kVisibility, &mut builder, parent).unwrap();
    assert_eq!(builder.Visibility(), EVisibility::kHidden);
    assert!(builder.VisibilityIsInherited());
    ApplyIdentifier(
        CSSPropertyID::kVisibility,
        &mut builder,
        CSSValueID::kVisible,
    )
    .unwrap();
    assert_eq!(builder.Visibility(), EVisibility::kVisible);
    assert!(!builder.VisibilityIsInherited());
}

#[test]
fn missing_converter_and_invalid_keyword_are_explicit_errors() {
    let _heap = foundation::LayoutHeapScope::new();
    let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
    let mut builder = ComputedStyleBuilder::from_style(initial);
    assert_eq!(
        ApplyIdentifier(CSSPropertyID::kWidth, &mut builder, CSSValueID::kAuto),
        Err(LonghandApplicationError::Unsupported(CSSPropertyID::kWidth))
    );
    assert_eq!(
        ApplyIdentifier(CSSPropertyID::kPosition, &mut builder, CSSValueID::kBlock),
        Err(LonghandApplicationError::InvalidValue(
            CSSPropertyID::kPosition
        ))
    );
    assert_eq!(builder.GetPosition(), EPosition::kStatic);
}
