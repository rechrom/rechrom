#![allow(non_snake_case)]

//! Frame-aligned CSS animation sample routing.
//!
//! The engine owns animation-frame ordering and paint-invalidation
//! classification. It does not read a clock, mutate a document, run style or
//! paint, or know which thread hosts those engines.

use compositor_animation::keyframed_animation_curve::{
    CanInterpolateTransformOperations, TransformOperation, TransformOperations,
};
use cssom::CSSDeclaration;
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

/// Cascade bucket that owns an already-sampled effect value.
///
/// This is the package-boundary equivalent of Blink's two retained
/// `CSSAnimationUpdate` maps. Sampling stays with the producer; Style uses the
/// origin only to establish animation-versus-transition cascade priority.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AnimationCascadeOrigin {
    #[default]
    Animation,
    Transition,
}

#[derive(Clone, Default)]
pub struct AnimationStyleSample {
    pub node_id: u64,
    pub effect_id: u64,
    pub cascade_origin: AnimationCascadeOrigin,
    pub declarations: Vec<CSSDeclaration>,
}

/// Stable CSS animation identity. Chromium keys CSSAnimationUpdate entries by
/// the originating element and animation-list position; the animation name is
/// data and changing it replaces the effect at this key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CSSAnimationKey {
    pub node_id: u64,
    pub list_index: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSSPlaybackDirection {
    Normal,
    Reverse,
    Alternate,
    AlternateReverse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSSFillMode {
    None,
    Forwards,
    Backwards,
    Both,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSSPlayState {
    Running,
    Paused,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CSSTimingFunction {
    Linear,
    CubicBezier([f64; 4]),
    Steps { count: u32, jump_at_start: bool },
}

impl Default for CSSTimingFunction {
    fn default() -> Self {
        Self::Linear
    }
}

fn SameTimingNumber(left: f64, right: f64) -> bool {
    left == right || (left.is_nan() && right.is_nan())
}

fn SameTimingFunction(left: &CSSTimingFunction, right: &CSSTimingFunction) -> bool {
    match (left, right) {
        (CSSTimingFunction::Linear, CSSTimingFunction::Linear) => true,
        (CSSTimingFunction::CubicBezier(left), CSSTimingFunction::CubicBezier(right)) => left
            .iter()
            .zip(right)
            .all(|(left, right)| SameTimingNumber(*left, *right)),
        (
            CSSTimingFunction::Steps {
                count: left_count,
                jump_at_start: left_jump,
            },
            CSSTimingFunction::Steps {
                count: right_count,
                jump_at_start: right_jump,
            },
        ) => left_count == right_count && left_jump == right_jump,
        _ => false,
    }
}

fn SameAnimationTiming(left: &CSSAnimationTiming, right: &CSSAnimationTiming) -> bool {
    SameTimingNumber(left.delay, right.delay)
        && SameTimingNumber(left.duration, right.duration)
        && SameTimingNumber(left.iterations, right.iterations)
        && left.direction == right.direction
        && left.fill == right.fill
        && left.play_state == right.play_state
        && SameTimingFunction(&left.easing, &right.easing)
}

fn AnimationTimingDifferenceMask(left: &CSSAnimationTiming, right: &CSSAnimationTiming) -> u32 {
    u32::from(!SameTimingNumber(left.delay, right.delay))
        | (u32::from(!SameTimingNumber(left.duration, right.duration)) << 1)
        | (u32::from(!SameTimingNumber(left.iterations, right.iterations)) << 2)
        | (u32::from(left.direction != right.direction) << 3)
        | (u32::from(left.fill != right.fill) << 4)
        | (u32::from(left.play_state != right.play_state) << 5)
        | (u32::from(!SameTimingFunction(&left.easing, &right.easing)) << 6)
}

#[derive(Clone, Debug, PartialEq)]
pub struct CSSAnimationTiming {
    pub delay: f64,
    pub duration: f64,
    pub iterations: f64,
    pub direction: CSSPlaybackDirection,
    pub fill: CSSFillMode,
    pub play_state: CSSPlayState,
    pub easing: CSSTimingFunction,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CSSKeyframe {
    pub offset: f64,
    pub declarations: Vec<CSSDeclaration>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CSSAnimationDefinition {
    pub key: CSSAnimationKey,
    pub name: String,
    pub timing: CSSAnimationTiming,
    pub keyframes: Vec<CSSKeyframe>,
}

#[derive(Clone, Debug)]
pub enum CompositorPropertyCurve {
    Opacity(Vec<(f64, f32)>),
    Transform(Vec<(f64, TransformOperations)>),
}

#[derive(Clone, Debug)]
pub struct CompositorAnimationCandidate {
    pub key: CSSAnimationKey,
    pub effect_id: u64,
    /// Changes whenever an installed compositor model must be replaced even
    /// though the CSS animation-list slot retained its source identity.
    pub generation: u64,
    pub timing: CSSAnimationTiming,
    pub local_time: f64,
    pub sample_time: Option<Instant>,
    pub curves: Vec<CompositorPropertyCurve>,
}

/// Typed marker for declarations that the current vertical slice cannot
/// interpolate without inventing a property-specific animation type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnsupportedInterpolation {
    pub node_id: u64,
    pub effect_id: u64,
    pub property: String,
}

#[derive(Clone, Default)]
pub struct AnimationTick {
    pub monotonic_time: f64,
    pub begin_frame_source_id: u64,
    pub begin_frame_sequence: u64,
    pub samples: Vec<AnimationStyleSample>,
    pub has_active_animations: bool,
}

pub enum AnimationMutation {
    BeginFrame {
        source_id: u64,
        begin_frame_sequence: u64,
        frame_time: Instant,
    },
    Tick(AnimationTick),
    SynchronizeCSSAnimations(Vec<CSSAnimationDefinition>),
    InvalidatePaintBatch,
    CommitPaint,
}

#[derive(Clone)]
pub enum DocumentMutation {
    ApplyStyleBatch(Vec<AnimationStyleSample>),
    SetPropertyFlags(Vec<AnimationPropertyFlags>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimationPropertyFlags {
    pub node_id: u64,
    pub transform: bool,
    pub opacity: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PaintMutation {
    SetTargetedProperties { node_ids: Vec<u64> },
    Commit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationEffect {
    RequestBeginFrame,
}

#[derive(Default)]
pub struct AnimationOutput {
    pub document: Vec<DocumentMutation>,
    pub paint: Vec<PaintMutation>,
    pub effects: Vec<AnimationEffect>,
    pub unsupported: Vec<UnsupportedInterpolation>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationError {
    InvalidTime,
    OutOfOrderFrame,
}

pub struct AnimationEngine {
    last_time: f64,
    last_begin_frame: Option<(u64, u64)>,
    last_frame_time: Option<Instant>,
    targeted_nodes: BTreeSet<u64>,
    targeted_batch_compatible: bool,
    css_timeline_origin: Option<Instant>,
    css_time: f64,
    next_css_effect_id: u64,
    css_animations: BTreeMap<CSSAnimationKey, RetainedCSSAnimation>,
    published_property_flags: Vec<AnimationPropertyFlags>,
}

struct RetainedCSSAnimation {
    effect_id: u64,
    compositor_generation: u64,
    definition: CSSAnimationDefinition,
    start_time: f64,
    hold_time: f64,
    last_sample: Vec<CSSDeclaration>,
    compositor_driven: BTreeSet<String>,
    compositor_eligible: BTreeSet<String>,
}

impl Default for AnimationEngine {
    fn default() -> Self {
        Self {
            last_time: 0.0,
            last_begin_frame: None,
            last_frame_time: None,
            targeted_nodes: BTreeSet::new(),
            targeted_batch_compatible: true,
            css_timeline_origin: None,
            css_time: 0.0,
            next_css_effect_id: 1u64 << 63,
            css_animations: BTreeMap::new(),
            published_property_flags: Vec::new(),
        }
    }
}

impl AnimationEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn LastTime(&self) -> f64 {
        self.last_time
    }

    fn SynchronizeCSSAnimations(
        &mut self,
        definitions: Vec<CSSAnimationDefinition>,
    ) -> AnimationOutput {
        let mut trace = browser_tracing::span("animation", "Animation.SynchronizeCSSAnimations");
        trace.set("existing", self.css_animations.len() as f64);
        trace.set("incoming", definitions.len() as f64);
        let mut incoming = definitions
            .into_iter()
            .map(|definition| (definition.key, definition))
            .collect::<BTreeMap<_, _>>();
        let removed = self
            .css_animations
            .keys()
            .filter(|key| !incoming.contains_key(key))
            .copied()
            .collect::<Vec<_>>();
        trace.set("removed", removed.len() as f64);
        let mut output = AnimationOutput::default();
        for key in removed {
            if let Some(effect) = self.css_animations.remove(&key) {
                if !effect.last_sample.is_empty() {
                    output.document.push(DocumentMutation::ApplyStyleBatch(vec![
                        AnimationStyleSample {
                            node_id: key.node_id,
                            effect_id: effect.effect_id,
                            cascade_origin: AnimationCascadeOrigin::Animation,
                            declarations: Vec::new(),
                        },
                    ]));
                }
            }
        }
        let mut retained = 0usize;
        let mut timing_changed = 0usize;
        let mut timing_change_mask = 0u32;
        let mut replaced = 0usize;
        for (key, definition) in incoming.iter_mut() {
            let replacing = self.css_animations.contains_key(key);
            if let Some(current) = self.css_animations.get_mut(key) {
                if current.definition.name == definition.name
                    && current.definition.keyframes == definition.keyframes
                {
                    retained += 1;
                    let was_paused = current.definition.timing.play_state == CSSPlayState::Paused;
                    let is_paused = definition.timing.play_state == CSSPlayState::Paused;
                    if was_paused != is_paused {
                        if is_paused {
                            current.hold_time = (self.css_time - current.start_time).max(0.0);
                        } else {
                            current.start_time = self.css_time - current.hold_time;
                        }
                    }
                    if !SameAnimationTiming(&current.definition.timing, &definition.timing) {
                        timing_changed += 1;
                        let difference = AnimationTimingDifferenceMask(
                            &current.definition.timing,
                            &definition.timing,
                        );
                        timing_change_mask |= difference;
                        browser_tracing::instant(
                            "animation",
                            "Animation.CSSAnimationTimingChanged",
                            &[
                                ("node_id", key.node_id as f64),
                                ("list_index", key.list_index as f64),
                                ("difference_mask", difference as f64),
                                (
                                    "old_play_state",
                                    (current.definition.timing.play_state == CSSPlayState::Paused)
                                        as u8 as f64,
                                ),
                                (
                                    "new_play_state",
                                    (definition.timing.play_state == CSSPlayState::Paused) as u8
                                        as f64,
                                ),
                                (
                                    "compositor_owned_properties",
                                    current.compositor_driven.len() as f64,
                                ),
                                (
                                    "compositor_curve_count",
                                    CompositorCurves(definition).map_or(0, |curves| curves.len())
                                        as f64,
                                ),
                                ("generation", current.compositor_generation as f64),
                            ],
                        );
                        // Chromium updates the existing KeyframeModel when CSS
                        // timing changes. Rechrom replaces that model through
                        // the same stable animation identity and retained
                        // property-tree element. Ownership stays on the
                        // compositor so pause/resume cannot create a one-frame
                        // document-sequence fallback.
                        current.compositor_generation =
                            current.compositor_generation.wrapping_add(1).max(1);
                    }
                    current.definition = definition.clone();
                    continue;
                }
            }
            replaced += usize::from(replacing);
            let effect_id = self.next_css_effect_id;
            self.next_css_effect_id = self.next_css_effect_id.wrapping_add(1).max(1u64 << 63);
            self.css_animations.insert(
                *key,
                RetainedCSSAnimation {
                    effect_id,
                    compositor_generation: 1,
                    definition: definition.clone(),
                    start_time: self.css_time,
                    hold_time: 0.0,
                    last_sample: Vec::new(),
                    compositor_driven: BTreeSet::new(),
                    compositor_eligible: CompositorCurves(definition)
                        .into_iter()
                        .flatten()
                        .map(|curve| match curve {
                            CompositorPropertyCurve::Transform(_) => "transform".to_owned(),
                            CompositorPropertyCurve::Opacity(_) => "opacity".to_owned(),
                        })
                        .collect(),
                },
            );
        }
        trace.set("retained", retained as f64);
        trace.set("timing_changed", timing_changed as f64);
        trace.set("timing_change_mask", timing_change_mask as f64);
        trace.set("replaced", replaced as f64);
        trace.set(
            "compositor_owned",
            self.css_animations
                .values()
                .map(|effect| effect.compositor_driven.len())
                .sum::<usize>() as f64,
        );
        let mut flags = BTreeMap::<u64, (bool, bool)>::new();
        for effect in self.css_animations.values() {
            for declaration in effect
                .definition
                .keyframes
                .iter()
                .flat_map(|keyframe| &keyframe.declarations)
            {
                let entry = flags.entry(effect.definition.key.node_id).or_default();
                match declaration.property.as_str() {
                    "transform" => entry.0 = true,
                    "opacity" => entry.1 = true,
                    _ => {}
                }
            }
        }
        let flags = flags
            .into_iter()
            .filter_map(|(node_id, (transform, opacity))| {
                (transform || opacity).then_some(AnimationPropertyFlags {
                    node_id,
                    transform,
                    opacity,
                })
            })
            .collect::<Vec<_>>();
        if flags != self.published_property_flags {
            self.published_property_flags = flags.clone();
            output
                .document
                .push(DocumentMutation::SetPropertyFlags(flags));
        }
        output.Merge(self.SampleCSSAnimations());
        output
    }

    pub fn CompositorCandidates(&self) -> Vec<CompositorAnimationCandidate> {
        self.css_animations
            .values()
            .filter_map(|effect| {
                let curves = CompositorCurves(&effect.definition)?;
                let local_time = if effect.definition.timing.play_state == CSSPlayState::Paused {
                    effect.hold_time
                } else {
                    self.css_time - effect.start_time
                };
                Some(CompositorAnimationCandidate {
                    key: effect.definition.key,
                    effect_id: effect.effect_id,
                    generation: effect.compositor_generation,
                    timing: effect.definition.timing.clone(),
                    local_time,
                    sample_time: self.last_frame_time,
                    curves,
                })
            })
            .collect()
    }

    pub fn SetCompositorDriven(
        &mut self,
        key: CSSAnimationKey,
        properties: impl IntoIterator<Item = String>,
    ) {
        if let Some(effect) = self.css_animations.get_mut(&key) {
            effect.compositor_driven.extend(properties);
        }
    }

    fn SampleCSSAnimations(&mut self) -> AnimationOutput {
        let mut output = AnimationOutput::default();
        let mut active = false;
        let mut changed_samples = Vec::new();
        for effect in self.css_animations.values_mut() {
            let local_time = if effect.definition.timing.play_state == CSSPlayState::Paused {
                effect.hold_time
            } else {
                self.css_time - effect.start_time
            };
            let mut sample = SampleCSSAnimation(effect, local_time, &mut output.unsupported);
            // A compositor-owned property keeps the computed-style value from
            // the handoff commit. Chromium stops main-thread interpolation;
            // it does not remove the animation-origin declaration or the
            // active-animation bit which owns the paint property node. Keep
            // that base value stable while continuing to sample any sibling
            // properties which remain on the document sequence.
            for declaration in &mut sample.declarations {
                if effect.compositor_driven.contains(&declaration.property) {
                    if let Some(retained) = effect
                        .last_sample
                        .iter()
                        .find(|retained| retained.property == declaration.property)
                    {
                        *declaration = retained.clone();
                    }
                }
            }
            let has_main_thread_property = sample
                .declarations
                .iter()
                .any(|declaration| !effect.compositor_driven.contains(&declaration.property));
            active |= sample.active
                && effect.definition.timing.play_state == CSSPlayState::Running
                && has_main_thread_property;
            if sample.declarations != effect.last_sample {
                effect.last_sample = sample.declarations.clone();
                let sample = AnimationStyleSample {
                    node_id: effect.definition.key.node_id,
                    effect_id: effect.effect_id,
                    cascade_origin: AnimationCascadeOrigin::Animation,
                    declarations: sample.declarations,
                };
                changed_samples.push(sample.clone());
                output
                    .document
                    .push(DocumentMutation::ApplyStyleBatch(vec![sample]));
            }
        }
        self.RecordTargetedProperties(&changed_samples, &mut output);
        if active {
            output.effects.push(AnimationEffect::RequestBeginFrame);
        }
        output
    }

    /// Blink keeps transform/opacity animation samples on the paint-property
    /// update path. CSS animations produced by this engine must use the same
    /// classification as externally supplied animation ticks; otherwise the
    /// identical declarations fall back to a full document paint walk.
    fn RecordTargetedProperties(
        &mut self,
        samples: &[AnimationStyleSample],
        output: &mut AnimationOutput,
    ) {
        if samples.is_empty() {
            return;
        }
        let targeted = samples.iter().all(|sample| {
            sample.declarations.iter().all(|declaration| {
                matches!(
                    declaration.property.as_str(),
                    "opacity" | "transform" | "transform-origin"
                )
            })
        }) && self.targeted_batch_compatible;

        if targeted {
            self.targeted_nodes
                .extend(samples.iter().map(|sample| sample.node_id));
        } else {
            self.targeted_nodes.clear();
            self.targeted_batch_compatible = false;
        }
        output.paint.push(PaintMutation::SetTargetedProperties {
            node_ids: self.targeted_nodes.iter().copied().collect(),
        });
    }

    pub fn ApplyMutation(
        &mut self,
        mutation: AnimationMutation,
    ) -> Result<AnimationOutput, AnimationError> {
        match mutation {
            AnimationMutation::BeginFrame {
                source_id,
                begin_frame_sequence,
                frame_time,
            } => {
                if self
                    .last_begin_frame
                    .is_some_and(|(last_source, last_sequence)| {
                        last_source == source_id && begin_frame_sequence < last_sequence
                    })
                    || self
                        .last_frame_time
                        .is_some_and(|last_time| frame_time < last_time)
                {
                    return Err(AnimationError::OutOfOrderFrame);
                }
                self.last_begin_frame = Some((source_id, begin_frame_sequence));
                self.last_frame_time = Some(frame_time);
                let origin = *self.css_timeline_origin.get_or_insert(frame_time);
                self.css_time = frame_time.saturating_duration_since(origin).as_secs_f64();
                Ok(self.SampleCSSAnimations())
            }
            AnimationMutation::Tick(tick) => self.ApplyTick(tick),
            AnimationMutation::SynchronizeCSSAnimations(definitions) => {
                Ok(self.SynchronizeCSSAnimations(definitions))
            }
            AnimationMutation::InvalidatePaintBatch => {
                self.targeted_nodes.clear();
                self.targeted_batch_compatible = false;
                Ok(AnimationOutput {
                    paint: vec![PaintMutation::SetTargetedProperties {
                        node_ids: Vec::new(),
                    }],
                    ..Default::default()
                })
            }
            AnimationMutation::CommitPaint => {
                self.targeted_nodes.clear();
                self.targeted_batch_compatible = true;
                Ok(AnimationOutput {
                    paint: vec![PaintMutation::Commit],
                    ..Default::default()
                })
            }
        }
    }

    fn ApplyTick(&mut self, tick: AnimationTick) -> Result<AnimationOutput, AnimationError> {
        if !tick.monotonic_time.is_finite() || tick.monotonic_time < self.last_time {
            return Err(AnimationError::InvalidTime);
        }
        if tick.begin_frame_source_id != 0 && tick.begin_frame_sequence != 0 {
            if self
                .last_begin_frame
                .is_some_and(|(last_source, last_sequence)| {
                    last_source == tick.begin_frame_source_id
                        && tick.begin_frame_sequence < last_sequence
                })
            {
                return Err(AnimationError::OutOfOrderFrame);
            }
            self.last_begin_frame = Some((tick.begin_frame_source_id, tick.begin_frame_sequence));
        }
        self.last_time = tick.monotonic_time;

        let mut output = AnimationOutput::default();
        if !tick.samples.is_empty() {
            output
                .document
                .push(DocumentMutation::ApplyStyleBatch(tick.samples.clone()));
            self.RecordTargetedProperties(&tick.samples, &mut output);
        }
        if tick.has_active_animations {
            output.effects.push(AnimationEffect::RequestBeginFrame);
        }
        Ok(output)
    }
}

impl AnimationOutput {
    fn Merge(&mut self, mut other: Self) {
        self.document.append(&mut other.document);
        self.paint.append(&mut other.paint);
        self.effects.append(&mut other.effects);
        self.unsupported.append(&mut other.unsupported);
    }
}

struct CSSSample {
    declarations: Vec<CSSDeclaration>,
    active: bool,
}

fn ApplyEasing(function: &CSSTimingFunction, value: f64) -> f64 {
    let value = value.clamp(0.0, 1.0);
    match function {
        CSSTimingFunction::Linear => value,
        CSSTimingFunction::Steps {
            count,
            jump_at_start,
        } => {
            let count = (*count).max(1) as f64;
            let step = (value * count).floor() + u8::from(*jump_at_start) as f64;
            (step / count).clamp(0.0, 1.0)
        }
        CSSTimingFunction::CubicBezier([x1, y1, x2, y2]) => {
            let curve = |t: f64, a: f64, b: f64| {
                3.0 * (1.0 - t) * (1.0 - t) * t * a + 3.0 * (1.0 - t) * t * t * b + t * t * t
            };
            let (mut low, mut high) = (0.0, 1.0);
            for _ in 0..24 {
                let middle = (low + high) * 0.5;
                if curve(middle, *x1, *x2) < value {
                    low = middle;
                } else {
                    high = middle;
                }
            }
            curve((low + high) * 0.5, *y1, *y2)
        }
    }
}

fn DirectedProgress(progress: f64, iteration: u64, direction: CSSPlaybackDirection) -> f64 {
    let reverse = match direction {
        CSSPlaybackDirection::Normal => false,
        CSSPlaybackDirection::Reverse => true,
        CSSPlaybackDirection::Alternate => iteration % 2 == 1,
        CSSPlaybackDirection::AlternateReverse => iteration % 2 == 0,
    };
    if reverse {
        1.0 - progress
    } else {
        progress
    }
}

fn SampleCSSAnimation(
    effect: &RetainedCSSAnimation,
    local_time: f64,
    unsupported: &mut Vec<UnsupportedInterpolation>,
) -> CSSSample {
    let timing = &effect.definition.timing;
    let duration = timing.duration.max(0.0);
    let active_duration = if duration == 0.0 {
        0.0
    } else {
        duration * timing.iterations.max(0.0)
    };
    let active_time = local_time - timing.delay;
    let (progress, active) = if active_time < 0.0 {
        if matches!(timing.fill, CSSFillMode::Backwards | CSSFillMode::Both) {
            (Some(DirectedProgress(0.0, 0, timing.direction)), true)
        } else {
            (None, true)
        }
    } else if active_time < active_duration || active_duration.is_infinite() {
        if duration == 0.0 {
            (Some(DirectedProgress(1.0, 0, timing.direction)), false)
        } else {
            let overall = active_time / duration;
            let iteration = overall.floor().max(0.0) as u64;
            (
                Some(DirectedProgress(
                    overall.fract(),
                    iteration,
                    timing.direction,
                )),
                true,
            )
        }
    } else if matches!(timing.fill, CSSFillMode::Forwards | CSSFillMode::Both) {
        let final_iteration = timing.iterations.ceil().max(1.0) as u64 - 1;
        (
            Some(DirectedProgress(1.0, final_iteration, timing.direction)),
            false,
        )
    } else {
        (None, false)
    };
    let Some(progress) = progress else {
        return CSSSample {
            declarations: Vec::new(),
            active,
        };
    };
    let progress = ApplyEasing(&timing.easing, progress);
    CSSSample {
        declarations: SampleKeyframes(effect, progress, unsupported),
        active,
    }
}

fn SampleKeyframes(
    effect: &RetainedCSSAnimation,
    progress: f64,
    unsupported: &mut Vec<UnsupportedInterpolation>,
) -> Vec<CSSDeclaration> {
    let mut properties = BTreeSet::new();
    for frame in &effect.definition.keyframes {
        properties.extend(
            frame
                .declarations
                .iter()
                .map(|value| value.property.clone()),
        );
    }
    let mut result = Vec::new();
    for property in properties {
        let values = effect
            .definition
            .keyframes
            .iter()
            .filter_map(|frame| {
                frame
                    .declarations
                    .iter()
                    .rev()
                    .find(|value| value.property == property && !value.important)
                    .map(|value| (frame.offset, value.value.as_str()))
            })
            .collect::<Vec<_>>();
        if values.first().is_none_or(|value| value.0 > 0.0)
            || values.last().is_none_or(|value| value.0 < 1.0)
        {
            unsupported.push(UnsupportedInterpolation {
                node_id: effect.definition.key.node_id,
                effect_id: effect.effect_id,
                property,
            });
            continue;
        }
        let upper = values
            .iter()
            .position(|value| value.0 >= progress)
            .unwrap_or(values.len() - 1);
        let lower = upper.saturating_sub(1);
        let (start_offset, start) = values[lower];
        let (end_offset, end) = values[upper];
        let interval = if end_offset == start_offset {
            1.0
        } else {
            (progress - start_offset) / (end_offset - start_offset)
        };
        let Some(value) = InterpolateCSSValue(start, end, interval.clamp(0.0, 1.0)) else {
            unsupported.push(UnsupportedInterpolation {
                node_id: effect.definition.key.node_id,
                effect_id: effect.effect_id,
                property,
            });
            continue;
        };
        result.push(CSSDeclaration {
            property,
            value,
            important: false,
        });
    }
    result
}

fn ParseNumber(value: &str) -> Option<f64> {
    value
        .trim()
        .parse()
        .ok()
        .filter(|value: &f64| value.is_finite())
}

fn ParseLengthPx(value: &str) -> Option<f64> {
    let value = value.trim();
    if value == "0" {
        return Some(0.0);
    }
    ParseNumber(value.strip_suffix("px")?)
}

fn ParseAngleDegrees(value: &str) -> Option<f64> {
    let value = value.trim();
    if let Some(value) = value.strip_suffix("deg") {
        ParseNumber(value)
    } else if let Some(value) = value.strip_suffix("turn") {
        Some(ParseNumber(value)? * 360.0)
    } else if value == "0" {
        Some(0.0)
    } else {
        None
    }
}

fn ParseTransformOperations(value: &str) -> Option<TransformOperations> {
    let value = value.trim();
    if value == "none" {
        return Some(Vec::new());
    }
    let mut operations = Vec::new();
    let mut rest = value;
    while !rest.trim_start().is_empty() {
        rest = rest.trim_start();
        let open = rest.find('(')?;
        let close = rest[open + 1..].find(')')? + open + 1;
        let name = rest[..open].trim();
        let arguments = rest[open + 1..close]
            .split(',')
            .flat_map(|part| part.split_ascii_whitespace())
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();
        let operation = match name {
            "translateY" if arguments.len() == 1 => TransformOperation::Translate {
                x: 0.0,
                y: ParseLengthPx(arguments[0])?,
                z: 0.0,
            },
            "translateX" if arguments.len() == 1 => TransformOperation::Translate {
                x: ParseLengthPx(arguments[0])?,
                y: 0.0,
                z: 0.0,
            },
            "translate" if (1..=2).contains(&arguments.len()) => TransformOperation::Translate {
                x: ParseLengthPx(arguments[0])?,
                y: arguments
                    .get(1)
                    .map_or(Some(0.0), |value| ParseLengthPx(value))?,
                z: 0.0,
            },
            "translate3d" if arguments.len() == 3 => TransformOperation::Translate {
                x: ParseLengthPx(arguments[0])?,
                y: ParseLengthPx(arguments[1])?,
                z: ParseLengthPx(arguments[2])?,
            },
            "scale" if (1..=2).contains(&arguments.len()) => {
                let x = ParseNumber(arguments[0])?;
                TransformOperation::Scale {
                    x,
                    y: arguments
                        .get(1)
                        .map_or(Some(x), |value| ParseNumber(value))?,
                    z: 1.0,
                }
            }
            "scaleX" if arguments.len() == 1 => TransformOperation::Scale {
                x: ParseNumber(arguments[0])?,
                y: 1.0,
                z: 1.0,
            },
            "scaleY" if arguments.len() == 1 => TransformOperation::Scale {
                x: 1.0,
                y: ParseNumber(arguments[0])?,
                z: 1.0,
            },
            "rotate" | "rotateZ" if arguments.len() == 1 => TransformOperation::RotateZ {
                degrees: ParseAngleDegrees(arguments[0])?,
            },
            _ => return None,
        };
        operations.push(operation);
        rest = &rest[close + 1..];
    }
    Some(operations)
}

fn CompatibleTransformKeyframes(values: &[(f64, TransformOperations)]) -> bool {
    if values.is_empty() {
        return false;
    }
    values
        .windows(2)
        .all(|pair| CanInterpolateTransformOperations(&pair[0].1, &pair[1].1))
}

fn TraceCompositorCurveRejection(
    definition: &CSSAnimationDefinition,
    reason: u8,
    property: u8,
    parsed_keyframes: usize,
    left_operations: usize,
    right_operations: usize,
) {
    browser_tracing::instant(
        "animation",
        "Animation.CompositorCurveRejected",
        &[
            ("node_id", definition.key.node_id as f64),
            ("list_index", definition.key.list_index as f64),
            ("reason", reason as f64),
            ("property", property as f64),
            ("source_keyframes", definition.keyframes.len() as f64),
            ("parsed_keyframes", parsed_keyframes as f64),
            ("left_operations", left_operations as f64),
            ("right_operations", right_operations as f64),
        ],
    );
}

fn CompositorCurves(definition: &CSSAnimationDefinition) -> Option<Vec<CompositorPropertyCurve>> {
    let mut properties = BTreeSet::new();
    for frame in &definition.keyframes {
        properties.extend(
            frame
                .declarations
                .iter()
                .map(|value| value.property.as_str()),
        );
    }
    if properties.is_empty() {
        TraceCompositorCurveRejection(definition, 1, 0, 0, 0, 0);
        return None;
    }
    if properties
        .iter()
        .any(|property| !matches!(*property, "opacity" | "transform"))
    {
        TraceCompositorCurveRejection(definition, 2, 0, 0, 0, 0);
        return None;
    }
    let mut curves = Vec::new();
    for property in properties {
        if property == "opacity" {
            let values = definition
                .keyframes
                .iter()
                .filter_map(|frame| {
                    frame
                        .declarations
                        .iter()
                        .rev()
                        .find(|value| value.property == property && !value.important)
                        .and_then(|value| value.value.parse::<f32>().ok())
                        .map(|value| (frame.offset, value))
                })
                .collect::<Vec<_>>();
            if values.first().is_none_or(|value| value.0 != 0.0)
                || values.last().is_none_or(|value| value.0 != 1.0)
            {
                TraceCompositorCurveRejection(definition, 3, 1, values.len(), 0, 0);
                return None;
            }
            curves.push(CompositorPropertyCurve::Opacity(values));
        } else {
            let values = definition
                .keyframes
                .iter()
                .filter_map(|frame| {
                    frame
                        .declarations
                        .iter()
                        .rev()
                        .find(|value| value.property == property && !value.important)
                        .and_then(|value| ParseTransformOperations(&value.value))
                        .map(|value| (frame.offset, value))
                })
                .collect::<Vec<_>>();
            if values.first().is_none_or(|value| value.0 != 0.0)
                || values.last().is_none_or(|value| value.0 != 1.0)
            {
                TraceCompositorCurveRejection(definition, 4, 2, values.len(), 0, 0);
                return None;
            }
            if !CompatibleTransformKeyframes(&values) {
                let incompatible = values
                    .windows(2)
                    .find(|pair| !CanInterpolateTransformOperations(&pair[0].1, &pair[1].1));
                let (left, right) =
                    incompatible.map_or((0, 0), |pair| (pair[0].1.len(), pair[1].1.len()));
                TraceCompositorCurveRejection(definition, 5, 2, values.len(), left, right);
                return None;
            }
            curves.push(CompositorPropertyCurve::Transform(values));
        }
    }
    Some(curves)
}

fn InterpolateCSSValue(start: &str, end: &str, progress: f64) -> Option<String> {
    if start == end {
        return Some(start.to_owned());
    }
    fn parts(value: &str) -> Option<(Vec<&str>, Vec<f64>)> {
        let bytes = value.as_bytes();
        let mut literals = Vec::new();
        let mut numbers = Vec::new();
        let mut literal_start = 0;
        let mut index = 0;
        while index < bytes.len() {
            let start = index;
            if matches!(bytes[index], b'+' | b'-') {
                index += 1;
            }
            let number_start = index;
            while index < bytes.len() && bytes[index].is_ascii_digit() {
                index += 1;
            }
            if index < bytes.len() && bytes[index] == b'.' {
                index += 1;
                while index < bytes.len() && bytes[index].is_ascii_digit() {
                    index += 1;
                }
            }
            if index == number_start || (index == number_start + 1 && bytes[number_start] == b'.') {
                index = start + 1;
                continue;
            }
            if index < bytes.len() && matches!(bytes[index], b'e' | b'E') {
                let exponent = index;
                index += 1;
                if index < bytes.len() && matches!(bytes[index], b'+' | b'-') {
                    index += 1;
                }
                let digits = index;
                while index < bytes.len() && bytes[index].is_ascii_digit() {
                    index += 1;
                }
                if digits == index {
                    index = exponent;
                }
            }
            literals.push(&value[literal_start..start]);
            numbers.push(value[start..index].parse().ok()?);
            literal_start = index;
        }
        literals.push(&value[literal_start..]);
        (!numbers.is_empty()).then_some((literals, numbers))
    }
    let (start_literals, start_numbers) = parts(start)?;
    let (end_literals, end_numbers) = parts(end)?;
    if start_literals != end_literals || start_numbers.len() != end_numbers.len() {
        return None;
    }
    let mut output = String::new();
    for index in 0..start_numbers.len() {
        output.push_str(start_literals[index]);
        let value = start_numbers[index] + (end_numbers[index] - start_numbers[index]) * progress;
        output.push_str(&value.to_string());
    }
    output.push_str(start_literals.last().copied().unwrap_or_default());
    Some(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accumulates_targeted_properties_until_commit() {
        let mut engine = AnimationEngine::new();
        for (node_id, property) in [(7, "opacity"), (9, "transform")] {
            let output = engine
                .ApplyMutation(AnimationMutation::Tick(AnimationTick {
                    monotonic_time: 1.0,
                    samples: vec![AnimationStyleSample {
                        node_id,
                        effect_id: node_id,
                        declarations: vec![CSSDeclaration {
                            property: property.into(),
                            value: "1".into(),
                            important: false,
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                }))
                .unwrap();
            let PaintMutation::SetTargetedProperties { node_ids } = &output.paint[0] else {
                panic!("expected targeted paint mutation")
            };
            assert!(node_ids.contains(&node_id));
        }
        engine
            .ApplyMutation(AnimationMutation::CommitPaint)
            .unwrap();
        let output = engine
            .ApplyMutation(AnimationMutation::Tick(AnimationTick {
                monotonic_time: 2.0,
                samples: vec![AnimationStyleSample {
                    node_id: 11,
                    effect_id: 11,
                    declarations: vec![CSSDeclaration {
                        property: "width".into(),
                        value: "1px".into(),
                        important: false,
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }))
            .unwrap();
        assert_eq!(
            output.paint,
            vec![PaintMutation::SetTargetedProperties {
                node_ids: Vec::new()
            }]
        );
    }

    #[test]
    fn begin_frame_order_is_scoped_to_the_source() {
        let mut engine = AnimationEngine::new();
        let start = Instant::now();
        engine
            .ApplyMutation(AnimationMutation::BeginFrame {
                source_id: 3,
                begin_frame_sequence: 90,
                frame_time: start,
            })
            .unwrap();
        engine
            .ApplyMutation(AnimationMutation::BeginFrame {
                source_id: 4,
                begin_frame_sequence: 1,
                frame_time: start,
            })
            .unwrap();
        assert!(matches!(
            engine.ApplyMutation(AnimationMutation::BeginFrame {
                source_id: 4,
                begin_frame_sequence: 0,
                frame_time: start,
            }),
            Err(AnimationError::OutOfOrderFrame)
        ));
    }
}
