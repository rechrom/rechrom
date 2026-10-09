// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Native computed animation/transition timing lists. Timeline-dependent timing
//! and effect execution remain outside this style-storage slice.
#![allow(non_snake_case, non_camel_case_types)]
use super::computed_style_constants::{EAnimPlayState, TimelineAxis, TimelineScroller};
use super::timeline_inset::TimelineInset;
use foundation::{
    AtomicString, CSSPropertyID, CSSValueID, HeapVector, Length, Member, ScopedCSSName,
    ScopedRefPtr, Traceable, ValuesEquivalent, Visitor,
};

// cpp: core/animation/timing.h:101-126. Time delays are stored in seconds;
// relative timeline delays retain their distinct optional field.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TimingDelay {
    pub time_delay: f64,
    pub relative_delay: Option<f64>,
}
// cpp: cc/animation/keyframe_model.h Direction/FillMode, timing.h:128-129.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaybackDirection {
    NORMAL,
    REVERSE,
    ALTERNATE_NORMAL,
    ALTERNATE_REVERSE,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FillMode {
    AUTO,
    NONE,
    FORWARDS,
    BACKWARDS,
    BOTH,
}

// cpp: platform/animation/timing_function.h; gfx/animation/keyframe/timing_function.h.
// The enum is the source base-class type discriminator with owning parameters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EaseType {
    EASE,
    EASE_IN,
    EASE_OUT,
    EASE_IN_OUT,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepPosition {
    START,
    END,
    JUMP_BOTH,
    JUMP_END,
    JUMP_NONE,
    JUMP_START,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LinearEasingPoint {
    pub input: f64,
    pub output: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub enum TimingFunction {
    Linear(Vec<LinearEasingPoint>),
    CubicBezierPreset(EaseType),
    CubicBezier([f64; 4]),
    Steps {
        number_of_steps: i32,
        step_position: StepPosition,
    },
}

// cpp: core/animation/effect_model.h:55-59.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompositeOperation {
    kCompositeReplace,
    kCompositeAdd,
    kCompositeAccumulate,
}

// cpp: core/style/style_timeline.h:21-106. Preserve the source variant and
// parameter types, including values set by other native style owners.
#[derive(Clone, Debug, PartialEq)]
pub enum StyleTimeline {
    Keyword(CSSValueID),
    Name(AtomicString),
    Scroll {
        axis: TimelineAxis,
        scroller: TimelineScroller,
    },
    View {
        axis: TimelineAxis,
        inset: TimelineInset,
    },
}
// cpp: bindings/core/v8/v8_timeline_range.h:27; animation/timeline_offset.h:22-46.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimelineNamedRange {
    kNone,
    kCover,
    kContain,
    kEntry,
    kEntryCrossing,
    kExit,
    kExitCrossing,
    kScroll,
}
#[derive(Clone, Debug)]
pub struct TimelineOffset {
    pub name: TimelineNamedRange,
    pub offset: Length,
    pub style_dependent_offset: Option<foundation::String>,
    pub zoom: Option<f32>,
}
impl PartialEq for TimelineOffset {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.offset == other.offset
            && self.style_dependent_offset == other.style_dependent_offset
    }
}

// cpp: core/animation/timeline_offset.h:67-89. Keep auto distinct from normal,
// including the source equality's optional-offset comparison.
#[derive(Clone, Debug)]
pub struct TimelineOffsetOrAuto {
    is_auto: bool,
    timeline_offset: Option<TimelineOffset>,
}
impl Default for TimelineOffsetOrAuto {
    fn default() -> Self {
        Self {
            is_auto: true,
            timeline_offset: None,
        }
    }
}
impl TimelineOffsetOrAuto {
    pub fn new(offset: Option<TimelineOffset>) -> Self {
        Self {
            is_auto: false,
            timeline_offset: offset,
        }
    }
    pub fn IsAuto(&self) -> bool {
        self.is_auto
    }
    pub fn GetTimelineOffset(&self) -> Option<TimelineOffset> {
        self.timeline_offset.clone()
    }
}
impl PartialEq for TimelineOffsetOrAuto {
    fn eq(&self, other: &Self) -> bool {
        self.IsAuto() && other.IsAuto() || self.GetTimelineOffset() == other.GetTimelineOffset()
    }
}

// cpp: core/animation/timing.h:177-187. Seconds are the existing native
// style-storage representation of AnimationTimeDelta, including auto duration.
#[derive(Clone)]
pub struct Timing {
    pub start_delay: TimingDelay,
    pub end_delay: TimingDelay,
    pub fill_mode: FillMode,
    pub iteration_start: f64,
    pub iteration_count: f64,
    pub iteration_duration: Option<f64>,
    pub direction: PlaybackDirection,
    pub timing_function: ScopedRefPtr<TimingFunction>,
}
impl Timing {
    // cpp: core/animation/timing.h:139-148.
    pub fn AssertValid(&self) {
        debug_assert!(!self.start_delay.time_delay.is_infinite());
        debug_assert!(!self.end_delay.time_delay.is_infinite());
        debug_assert!(self.iteration_start.is_finite() && self.iteration_start >= 0.0);
        debug_assert!(self.iteration_count >= 0.0);
        debug_assert!(self
            .iteration_duration
            .is_none_or(|duration| duration >= 0.0));
    }
}
impl Default for Timing {
    fn default() -> Self {
        Self {
            start_delay: TimingDelay::default(),
            end_delay: TimingDelay::default(),
            fill_mode: FillMode::AUTO,
            iteration_start: 0.0,
            iteration_count: 1.0,
            iteration_duration: None,
            direction: PlaybackDirection::NORMAL,
            timing_function: ScopedRefPtr::new(TimingFunction::Linear(vec![])),
        }
    }
}

// cpp: core/animation/css/css_timing_data.h:26-94; .cc:9-21,43-64.
#[derive(Clone)]
pub struct CSSTimingData {
    delay_start_list: Vec<TimingDelay>,
    delay_end_list: Vec<TimingDelay>,
    duration_list: Vec<Option<f64>>,
    timing_function_list: Vec<ScopedRefPtr<TimingFunction>>,
}
macro_rules! list_accessors {
    ($getter:ident, $mutable:ident, $field:ident, $ty:ty) => {
        pub fn $getter(&self) -> &Vec<$ty> {
            &self.$field
        }
        pub fn $mutable(&mut self) -> &mut Vec<$ty> {
            &mut self.$field
        }
    };
}
impl CSSTimingData {
    pub fn new(initial_duration: Option<f64>) -> Self {
        Self {
            delay_start_list: vec![TimingDelay::default()],
            delay_end_list: vec![TimingDelay::default()],
            duration_list: vec![initial_duration],
            timing_function_list: vec![Self::InitialTimingFunction()],
        }
    }
    pub fn InitialDelayStart() -> TimingDelay {
        TimingDelay::default()
    }
    // cpp: css_timing_data.h:59-62; css_timing_data.cc:18-32.
    pub fn GetRepeated<T>(list: &[T], index: usize) -> &T {
        &list[index % list.len()]
    }
    pub fn ConvertToTiming(&self, index: usize) -> Timing {
        let timing = Timing {
            start_delay: *Self::GetRepeated(&self.delay_start_list, index),
            end_delay: *Self::GetRepeated(&self.delay_end_list, index),
            iteration_duration: *Self::GetRepeated(&self.duration_list, index),
            timing_function: Self::GetRepeated(&self.timing_function_list, index).clone(),
            ..Timing::default()
        };
        timing.AssertValid();
        timing
    }
    list_accessors!(DelayEndList, DelayEndListMut, delay_end_list, TimingDelay);
    pub fn InitialTimingFunction() -> ScopedRefPtr<TimingFunction> {
        ScopedRefPtr::new(TimingFunction::CubicBezierPreset(EaseType::EASE))
    }
    list_accessors!(
        DelayStartList,
        DelayStartListMut,
        delay_start_list,
        TimingDelay
    );
    list_accessors!(DurationList, DurationListMut, duration_list, Option<f64>);
    list_accessors!(
        TimingFunctionList,
        TimingFunctionListMut,
        timing_function_list,
        ScopedRefPtr<TimingFunction>
    );
}
impl PartialEq for CSSTimingData {
    fn eq(&self, other: &Self) -> bool {
        self.delay_start_list == other.delay_start_list
            && self.delay_end_list == other.delay_end_list
            && self.duration_list == other.duration_list
            && self.timing_function_list.len() == other.timing_function_list.len()
            && self
                .timing_function_list
                .iter()
                .zip(&other.timing_function_list)
                .all(|(a, b)| **a == **b)
    }
}

// cpp: core/animation/css/css_animation_data.h; .cc:32-59,78-88.
// Trigger attachments remain outside this native storage slice; timeline trigger
// declaration lists are real CSSAnimationData fields.
#[derive(Clone)]
pub struct CSSAnimationData {
    timing: CSSTimingData,
    name_list: HeapVector<Member<ScopedCSSName>>,
    timeline_list: Vec<StyleTimeline>,
    range_start_list: Vec<Option<TimelineOffset>>,
    range_end_list: Vec<Option<TimelineOffset>>,
    timeline_trigger_name_list: HeapVector<Member<ScopedCSSName>>,
    timeline_trigger_source_list: Vec<StyleTimeline>,
    timeline_trigger_activation_range_start_list: Vec<Option<TimelineOffset>>,
    timeline_trigger_activation_range_end_list: Vec<Option<TimelineOffset>>,
    timeline_trigger_active_range_start_list: Vec<TimelineOffsetOrAuto>,
    timeline_trigger_active_range_end_list: Vec<TimelineOffsetOrAuto>,
    composition_list: Vec<CompositeOperation>,
    iteration_count_list: Vec<f64>,
    direction_list: Vec<PlaybackDirection>,
    fill_mode_list: Vec<FillMode>,
    play_state_list: Vec<EAnimPlayState>,
}
impl Default for CSSAnimationData {
    fn default() -> Self {
        Self {
            timing: CSSTimingData::new(Self::InitialDuration()),
            name_list: vec![Self::InitialName()].into(),
            timeline_list: vec![Self::InitialTimeline()],
            range_start_list: vec![Self::InitialRangeStart()],
            range_end_list: vec![Self::InitialRangeEnd()],
            timeline_trigger_name_list: vec![Self::InitialTimelineTriggerName()].into(),
            timeline_trigger_source_list: vec![Self::InitialTimelineTriggerSource()],
            timeline_trigger_activation_range_start_list: vec![
                Self::InitialTimelineTriggerActivationRangeStart(),
            ],
            timeline_trigger_activation_range_end_list: vec![
                Self::InitialTimelineTriggerActivationRangeEnd(),
            ],
            timeline_trigger_active_range_start_list: vec![
                Self::InitialTimelineTriggerActiveRangeStart(),
            ],
            timeline_trigger_active_range_end_list: vec![
                Self::InitialTimelineTriggerActiveRangeEnd(),
            ],
            composition_list: vec![Self::InitialComposition()],
            iteration_count_list: vec![Self::InitialIterationCount()],
            direction_list: vec![Self::InitialDirection()],
            fill_mode_list: vec![Self::InitialFillMode()],
            play_state_list: vec![Self::InitialPlayState()],
        }
    }
}
impl CSSAnimationData {
    pub fn InitialTimelineTriggerName() -> Member<ScopedCSSName> {
        Member::default()
    }
    pub fn TimelineTriggerNameList(&self) -> &HeapVector<Member<ScopedCSSName>> {
        &self.timeline_trigger_name_list
    }
    pub fn TimelineTriggerNameListMut(&mut self) -> &mut HeapVector<Member<ScopedCSSName>> {
        &mut self.timeline_trigger_name_list
    }
    pub fn InitialTimelineTriggerSource() -> StyleTimeline {
        StyleTimeline::Keyword(CSSValueID::kAuto)
    }
    list_accessors!(
        TimelineTriggerSourceList,
        TimelineTriggerSourceListMut,
        timeline_trigger_source_list,
        StyleTimeline
    );
    pub fn InitialTimelineTriggerActivationRangeStart() -> Option<TimelineOffset> {
        None
    }
    list_accessors!(
        TimelineTriggerActivationRangeStartList,
        TimelineTriggerActivationRangeStartListMut,
        timeline_trigger_activation_range_start_list,
        Option<TimelineOffset>
    );
    pub fn InitialTimelineTriggerActivationRangeEnd() -> Option<TimelineOffset> {
        None
    }
    list_accessors!(
        TimelineTriggerActivationRangeEndList,
        TimelineTriggerActivationRangeEndListMut,
        timeline_trigger_activation_range_end_list,
        Option<TimelineOffset>
    );
    pub fn InitialTimelineTriggerActiveRangeStart() -> TimelineOffsetOrAuto {
        TimelineOffsetOrAuto::default()
    }
    list_accessors!(
        TimelineTriggerActiveRangeStartList,
        TimelineTriggerActiveRangeStartListMut,
        timeline_trigger_active_range_start_list,
        TimelineOffsetOrAuto
    );
    pub fn InitialTimelineTriggerActiveRangeEnd() -> TimelineOffsetOrAuto {
        TimelineOffsetOrAuto::default()
    }
    list_accessors!(
        TimelineTriggerActiveRangeEndList,
        TimelineTriggerActiveRangeEndListMut,
        timeline_trigger_active_range_end_list,
        TimelineOffsetOrAuto
    );
    // cpp: css_animation_data.cc:13-30,106-110,122-151.
    pub fn GetTimelineTriggerSource(&self, index: usize) -> &StyleTimeline {
        debug_assert!(index < self.timeline_trigger_name_list.len());
        CSSTimingData::GetRepeated(&self.timeline_trigger_source_list, index)
    }
    pub fn TimelineTriggerNamesMatch(&self, other: &Self) -> bool {
        self.timeline_trigger_name_list.len() == other.timeline_trigger_name_list.len()
            && self
                .timeline_trigger_name_list
                .iter()
                .zip(other.timeline_trigger_name_list.iter())
                .all(|(a, b)| ValuesEquivalent(a, b))
    }
    pub fn TriggersMatchForStyleRecalc(&self, other: &Self) -> bool {
        self.TimelineTriggerNamesMatch(other)
            && self.timeline_trigger_source_list == other.timeline_trigger_source_list
            && self.timeline_trigger_activation_range_start_list
                == other.timeline_trigger_activation_range_start_list
            && self.timeline_trigger_activation_range_end_list
                == other.timeline_trigger_activation_range_end_list
            && self.timeline_trigger_active_range_start_list
                == other.timeline_trigger_active_range_start_list
            && self.timeline_trigger_active_range_end_list
                == other.timeline_trigger_active_range_end_list
    }
    pub fn TimelineTriggerDataChanged(old: Option<&Self>, new: Option<&Self>) -> bool {
        match (old, new) {
            (Some(a), Some(b)) => !a.TriggersMatchForStyleRecalc(b),
            (Some(d), None) | (None, Some(d)) => d
                .TimelineTriggerNameList()
                .iter()
                .any(|n| !n.Get().is_null()),
            (None, None) => false,
        }
    }
    pub fn InitialName() -> Member<ScopedCSSName> {
        Member::default()
    }
    pub fn InitialTimeline() -> StyleTimeline {
        StyleTimeline::Keyword(CSSValueID::kAuto)
    }
    pub fn InitialRangeStart() -> Option<TimelineOffset> {
        None
    }
    pub fn InitialRangeEnd() -> Option<TimelineOffset> {
        None
    }
    pub fn InitialComposition() -> CompositeOperation {
        CompositeOperation::kCompositeReplace
    }
    pub fn NameList(&self) -> &HeapVector<Member<ScopedCSSName>> {
        &self.name_list
    }
    pub fn NameListMut(&mut self) -> &mut HeapVector<Member<ScopedCSSName>> {
        &mut self.name_list
    }
    list_accessors!(TimelineList, TimelineListMut, timeline_list, StyleTimeline);
    list_accessors!(
        RangeStartList,
        RangeStartListMut,
        range_start_list,
        Option<TimelineOffset>
    );
    list_accessors!(
        RangeEndList,
        RangeEndListMut,
        range_end_list,
        Option<TimelineOffset>
    );
    list_accessors!(
        CompositionList,
        CompositionListMut,
        composition_list,
        CompositeOperation
    );
    // cpp: css_animation_data.cc:61-66,78-104; .h:108-115.
    pub fn NamesMatch(&self, other: &Self) -> bool {
        self.name_list.len() == other.name_list.len()
            && self
                .name_list
                .iter()
                .zip(other.name_list.iter())
                .all(|(a, b)| ValuesEquivalent(a, b))
    }
    pub fn AnimationsMatchForStyleRecalc(&self, other: &Self) -> bool {
        self.NamesMatch(other)
            && self.timeline_list == other.timeline_list
            && self.play_state_list == other.play_state_list
            && self.iteration_count_list == other.iteration_count_list
            && self.direction_list == other.direction_list
            && self.fill_mode_list == other.fill_mode_list
            && self.range_start_list == other.range_start_list
            && self.range_end_list == other.range_end_list
            && self.timing == other.timing
            && self.TriggersMatchForStyleRecalc(other)
    }
    pub fn ConvertToTiming(&self, index: usize) -> Timing {
        debug_assert!(index < self.name_list.len());
        let mut timing = self.timing.ConvertToTiming(index);
        timing.iteration_count = *CSSTimingData::GetRepeated(&self.iteration_count_list, index);
        timing.direction = *CSSTimingData::GetRepeated(&self.direction_list, index);
        timing.fill_mode = *CSSTimingData::GetRepeated(&self.fill_mode_list, index);
        timing.AssertValid();
        timing
    }
    pub fn GetTimeline(&self, index: usize) -> &StyleTimeline {
        debug_assert!(index < self.name_list.len());
        CSSTimingData::GetRepeated(&self.timeline_list, index)
    }
    pub fn GetComposition(&self, index: usize) -> CompositeOperation {
        if self.composition_list.is_empty() {
            Self::InitialComposition()
        } else {
            *CSSTimingData::GetRepeated(&self.composition_list, index)
        }
    }
    pub fn InitialDelayStart() -> TimingDelay {
        CSSTimingData::InitialDelayStart()
    }
    pub fn InitialTimingFunction() -> ScopedRefPtr<TimingFunction> {
        CSSTimingData::InitialTimingFunction()
    }
    pub fn InitialDuration() -> Option<f64> {
        None
    }
    pub fn InitialIterationCount() -> f64 {
        1.0
    }
    pub fn InitialDirection() -> PlaybackDirection {
        PlaybackDirection::NORMAL
    }
    pub fn InitialFillMode() -> FillMode {
        FillMode::NONE
    }
    pub fn InitialPlayState() -> EAnimPlayState {
        EAnimPlayState::kPlaying
    }
    list_accessors!(
        IterationCountList,
        IterationCountListMut,
        iteration_count_list,
        f64
    );
    list_accessors!(
        DirectionList,
        DirectionListMut,
        direction_list,
        PlaybackDirection
    );
    list_accessors!(FillModeList, FillModeListMut, fill_mode_list, FillMode);
    list_accessors!(
        PlayStateList,
        PlayStateListMut,
        play_state_list,
        EAnimPlayState
    );
}
impl std::ops::Deref for CSSAnimationData {
    type Target = CSSTimingData;
    fn deref(&self) -> &Self::Target {
        &self.timing
    }
}
impl std::ops::DerefMut for CSSAnimationData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.timing
    }
}
impl PartialEq for CSSAnimationData {
    fn eq(&self, other: &Self) -> bool {
        self.AnimationsMatchForStyleRecalc(other)
    }
}
impl Traceable for CSSAnimationData {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.name_list);
        visitor.Trace(&self.timeline_trigger_name_list);
    }
}

// cpp: core/animation/css/css_transition_data.h:26-102.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransitionProperty {
    None,
    Known(CSSPropertyID),
    Unknown(AtomicString),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionBehavior {
    kNormal,
    kAllowDiscrete,
}
#[derive(Clone)]
pub struct CSSTransitionData {
    timing: CSSTimingData,
    property_list: Vec<TransitionProperty>,
    behavior_list: Vec<TransitionBehavior>,
}
impl Default for CSSTransitionData {
    fn default() -> Self {
        Self {
            timing: CSSTimingData::new(Self::InitialDuration()),
            property_list: vec![Self::InitialProperty()],
            behavior_list: vec![Self::InitialBehavior()],
        }
    }
}
impl CSSTransitionData {
    pub fn InitialDelayStart() -> TimingDelay {
        CSSTimingData::InitialDelayStart()
    }
    pub fn InitialTimingFunction() -> ScopedRefPtr<TimingFunction> {
        CSSTimingData::InitialTimingFunction()
    }
    pub fn InitialDuration() -> Option<f64> {
        Some(0.0)
    }
    pub fn InitialProperty() -> TransitionProperty {
        TransitionProperty::Known(CSSPropertyID::kAll)
    }
    pub fn InitialBehavior() -> TransitionBehavior {
        TransitionBehavior::kNormal
    }
    list_accessors!(
        PropertyList,
        PropertyListMut,
        property_list,
        TransitionProperty
    );
    list_accessors!(
        BehaviorList,
        BehaviorListMut,
        behavior_list,
        TransitionBehavior
    );
}
impl PartialEq for CSSTransitionData {
    fn eq(&self, other: &Self) -> bool {
        // cpp: css_transition_data.cc:22-26. The source's comparison does not
        // include behavior_list_; preserve that boundary independently of storage.
        self.property_list == other.property_list && self.timing == other.timing
    }
}
impl std::ops::Deref for CSSTransitionData {
    type Target = CSSTimingData;
    fn deref(&self) -> &Self::Target {
        &self.timing
    }
}
impl std::ops::DerefMut for CSSTransitionData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.timing
    }
}
impl Traceable for CSSTransitionData {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        for property in &self.property_list {
            if let TransitionProperty::Unknown(name) = property {
                visitor.Trace(name);
            }
        }
    }
}
