//! Source-aligned elastic overscroll state machine.
//!
//! Source:
//! third_party/blink/renderer/platform/widget/input/elastic_overscroll_controller.{h,cc}
//! third_party/blink/renderer/platform/widget/input/elastic_overscroll_controller_exponential.cc

use crate::{
    input_handler_types::InputHandlerScrollResult, overscroll_behavior::OverscrollBehavior,
};
use compositor_scheduler::{TimeDelta, TimeTicks};
use compositor_tree::{geometry::Vector2dF, ElementId};
use std::collections::BTreeMap;

const SCROLL_VELOCITY_ZEROING_TIMEOUT_SECONDS: f64 = 0.10;
const MINIMUM_DELTA_BEFORE_STRETCH: f32 = 10.0;
const RUBBERBAND_STIFFNESS: f32 = 20.0;
const RUBBERBAND_AMPLITUDE: f32 = 0.31;
const RUBBERBAND_PERIOD: f32 = 1.6;

pub trait ElasticOverscrollContext {
    fn constrain_overscroll_delta(&mut self, id: ElementId, delta: Vector2dF) -> Vector2dF;
    fn is_user_overscrollable(&mut self, id: ElementId) -> bool;
    fn stretch_amount(&mut self, id: ElementId) -> Vector2dF;
    fn set_stretch_amount(&mut self, id: ElementId, amount: Vector2dF);
    fn scroll_offset(&mut self, id: ElementId) -> compositor_tree::geometry::PointF;
    fn max_scroll_offset(&mut self, id: ElementId) -> compositor_tree::geometry::PointF;
    fn scroll_by(&mut self, id: ElementId, delta: Vector2dF);
    fn request_one_begin_frame(&mut self);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum State {
    #[default]
    Inactive,
    ActiveScroll,
    MomentumScroll,
    MomentumAnimated,
}

#[derive(Clone, Debug)]
struct OverscrollEntry {
    target_scroller_id: ElementId,
    momentum_animation_start_time: TimeTicks,
    state: State,
    pending_overscroll_delta: Vector2dF,
    scroll_velocity: Vector2dF,
    last_scroll_event_timestamp: TimeTicks,
    stretch_scroll_force: Vector2dF,
    received_overscroll_update: bool,
    overscroll_behavior: OverscrollBehavior,
    momentum_animation_initial_stretch: Vector2dF,
    momentum_animation_initial_velocity: Vector2dF,
    is_in_momentum_phase: bool,
}

impl OverscrollEntry {
    fn new(target_scroller_id: ElementId) -> Self {
        Self {
            target_scroller_id,
            momentum_animation_start_time: TimeTicks::default(),
            state: State::Inactive,
            pending_overscroll_delta: Vector2dF::default(),
            scroll_velocity: Vector2dF::default(),
            last_scroll_event_timestamp: TimeTicks::default(),
            stretch_scroll_force: Vector2dF::default(),
            received_overscroll_update: false,
            overscroll_behavior: OverscrollBehavior::default(),
            momentum_animation_initial_stretch: Vector2dF::default(),
            momentum_animation_initial_velocity: Vector2dF::default(),
            is_in_momentum_phase: false,
        }
    }
}

#[derive(Default)]
pub struct ElasticOverscrollController {
    entries: BTreeMap<ElementId, OverscrollEntry>,
}

impl ElasticOverscrollController {
    pub fn can_overscroll<C: ElasticOverscrollContext>(
        &self,
        context: &mut C,
        id: ElementId,
    ) -> bool {
        id.IsValid() && context.is_user_overscrollable(id)
    }

    pub fn observe_scroll_begin<C: ElasticOverscrollContext>(
        &mut self,
        context: &mut C,
        id: ElementId,
        enter_momentum: bool,
        leave_momentum: bool,
    ) {
        if !self.can_overscroll(context, id) {
            return;
        }
        let entry = self
            .entries
            .entry(id)
            .or_insert_with(|| OverscrollEntry::new(id));
        entry.received_overscroll_update = false;
        entry.overscroll_behavior = OverscrollBehavior::default();
        if enter_momentum {
            if entry.state == State::Inactive {
                entry.state = State::MomentumScroll;
            }
        } else if leave_momentum {
            entry.scroll_velocity = Vector2dF::default();
            entry.last_scroll_event_timestamp = TimeTicks::default();
            entry.state = State::ActiveScroll;
            entry.pending_overscroll_delta = Vector2dF::default();
        }
    }

    pub fn observe_scroll_update<C: ElasticOverscrollContext>(
        &mut self,
        context: &mut C,
        id: ElementId,
        event_delta: Vector2dF,
        result: &InputHandlerScrollResult,
        event_timestamp: TimeTicks,
        has_momentum: bool,
    ) {
        if !self.can_overscroll(context, id) {
            return;
        }
        let entry = self
            .entries
            .entry(id)
            .or_insert_with(|| OverscrollEntry::new(id));
        if entry.state == State::MomentumAnimated || entry.state == State::Inactive {
            return;
        }
        if !entry.received_overscroll_update && !result.unused_scroll_delta.IsZero() {
            entry.overscroll_behavior = result.overscroll_behavior;
            entry.received_overscroll_update = true;
        }
        // Source: ElasticOverscrollController::ObserveScrollUpdate. macOS can
        // deliver the first momentum sample only a few milliseconds after the
        // last finger sample with a much larger delta. Treating that phase
        // boundary as an ordinary velocity sample gives the rubber-band a
        // bogus launch velocity and makes the page snap at a scroll edge.
        if !has_momentum || entry.is_in_momentum_phase {
            Self::update_velocity(context, entry, event_delta, event_timestamp);
        }
        entry.is_in_momentum_phase = has_momentum;
        Self::overscroll(context, entry, result.unused_scroll_delta, event_delta);
        if has_momentum && !context.stretch_amount(id).IsZero() {
            Self::enter_momentum_animated(context, entry, event_timestamp);
        }
    }

    pub fn observe_scroll_end<C: ElasticOverscrollContext>(
        &mut self,
        context: &mut C,
        id: ElementId,
        event_timestamp: TimeTicks,
    ) {
        if !self.can_overscroll(context, id) {
            return;
        }
        let Some(entry) = self.entries.get_mut(&id) else {
            return;
        };
        if entry.state == State::MomentumAnimated || entry.state == State::Inactive {
            return;
        }
        if context.stretch_amount(id).IsZero() {
            Self::enter_inactive(entry);
        } else {
            Self::enter_momentum_animated(context, entry, event_timestamp);
        }
    }

    pub fn animate<C: ElasticOverscrollContext>(
        &mut self,
        context: &mut C,
        time: TimeTicks,
    ) -> Vec<ElementId> {
        let mut is_animating = false;
        let mut finished = Vec::new();
        for (&id, entry) in &mut self.entries {
            if entry.state != State::MomentumAnimated {
                continue;
            }
            let elapsed = time - entry.momentum_animation_start_time;
            let elapsed = if elapsed.is_negative() {
                TimeDelta::default()
            } else {
                elapsed
            };
            let new_stretch = Self::stretch_amount_for_time(entry, elapsed);
            if new_stretch.x().abs() < 1.0 && new_stretch.y().abs() < 1.0 {
                context.set_stretch_amount(id, Vector2dF::default());
                finished.push(id);
                Self::enter_inactive(entry);
                continue;
            }
            entry.stretch_scroll_force = Self::accumulated_overscroll(new_stretch);
            context.set_stretch_amount(id, new_stretch);
            is_animating = true;
        }
        self.entries
            .retain(|_, entry| entry.state != State::Inactive);
        if is_animating {
            context.request_one_begin_frame();
        }
        finished
    }

    pub fn reconcile_stretch_and_scroll<C: ElasticOverscrollContext>(&mut self, context: &mut C) {
        for (&id, entry) in &mut self.entries {
            let stretch = context.stretch_amount(id);
            if stretch.IsZero() {
                continue;
            }
            let offset = context.scroll_offset(id);
            let max = context.max_scroll_offset(id);
            let mut adjustment = Vector2dF::default();
            if stretch.x() < 0.0 && offset.x() > 0.0 {
                adjustment.set_x((-stretch.x()).min(offset.x()));
            }
            if stretch.x() > 0.0 && offset.x() < max.x() {
                adjustment.set_x((-stretch.x()).max(offset.x() - max.x()));
            }
            if stretch.y() < 0.0 && offset.y() > 0.0 {
                adjustment.set_y((-stretch.y()).min(offset.y()));
            }
            if stretch.y() > 0.0 && offset.y() < max.y() {
                adjustment.set_y((-stretch.y()).max(offset.y() - max.y()));
            }
            if adjustment.IsZero() {
                continue;
            }
            let new_stretch = stretch + adjustment;
            context.scroll_by(id, Self::scale(adjustment, -1.0));
            context.set_stretch_amount(id, new_stretch);
            if entry.state == State::ActiveScroll {
                entry.stretch_scroll_force = Self::accumulated_overscroll(new_stretch);
            }
        }
    }

    fn update_velocity<C: ElasticOverscrollContext>(
        context: &mut C,
        entry: &mut OverscrollEntry,
        event_delta: Vector2dF,
        timestamp: TimeTicks,
    ) {
        let adjusted = context.constrain_overscroll_delta(entry.target_scroller_id, event_delta);
        let seconds = (timestamp - entry.last_scroll_event_timestamp).InSecondsF();
        entry.scroll_velocity =
            if seconds > 0.0 && seconds < SCROLL_VELOCITY_ZEROING_TIMEOUT_SECONDS {
                Vector2dF::new(adjusted.x() / seconds as f32, adjusted.y() / seconds as f32)
            } else {
                Vector2dF::default()
            };
        entry.last_scroll_event_timestamp = timestamp;
    }

    fn overscroll<C: ElasticOverscrollContext>(
        context: &mut C,
        entry: &mut OverscrollEntry,
        overscroll_delta: Vector2dF,
        event_delta: Vector2dF,
    ) {
        let id = entry.target_scroller_id;
        let mut adjusted = context.constrain_overscroll_delta(id, overscroll_delta);
        if !context.is_user_overscrollable(id) {
            adjusted = Vector2dF::default();
        }
        if adjusted.IsZero() {
            return;
        }
        adjusted += entry.pending_overscroll_delta;
        entry.pending_overscroll_delta = Vector2dF::default();
        if event_delta.y().abs() >= event_delta.x().abs() {
            adjusted.set_x(0.0);
        } else {
            adjusted.set_y(0.0);
        }
        let offset = context.scroll_offset(id);
        let max = context.max_scroll_offset(id);
        if !Self::pinned(offset.x(), max.x(), adjusted.x()) {
            adjusted.set_x(0.0);
        }
        if !Self::pinned(offset.y(), max.y(), adjusted.y()) {
            adjusted.set_y(0.0);
        }
        if !entry.overscroll_behavior.HasXLocalBorderEffects() {
            adjusted.set_x(0.0);
        }
        if !entry.overscroll_behavior.HasYLocalBorderEffects() {
            adjusted.set_y(0.0);
        }
        let old_stretch = context.stretch_amount(id);
        let mut force_delta = Vector2dF::default();
        if old_stretch.x() != 0.0 || adjusted.x().abs() >= MINIMUM_DELTA_BEFORE_STRETCH {
            force_delta.set_x(adjusted.x());
        } else {
            entry.pending_overscroll_delta.set_x(adjusted.x());
        }
        if old_stretch.y() != 0.0 || adjusted.y().abs() >= MINIMUM_DELTA_BEFORE_STRETCH {
            force_delta.set_y(adjusted.y());
        } else {
            entry.pending_overscroll_delta.set_y(adjusted.y());
        }
        if force_delta.IsZero() {
            return;
        }
        entry.stretch_scroll_force += force_delta;
        context.set_stretch_amount(
            id,
            Self::stretch_for_accumulated(entry.stretch_scroll_force),
        );
    }

    fn enter_inactive(entry: &mut OverscrollEntry) {
        entry.state = State::Inactive;
        entry.stretch_scroll_force = Vector2dF::default();
    }

    fn enter_momentum_animated<C: ElasticOverscrollContext>(
        context: &mut C,
        entry: &mut OverscrollEntry,
        timestamp: TimeTicks,
    ) {
        entry.state = State::MomentumAnimated;
        let stretch = context.stretch_amount(entry.target_scroller_id);
        if stretch.IsZero() {
            return;
        }
        entry.momentum_animation_start_time = timestamp;
        entry.momentum_animation_initial_stretch = stretch;
        entry.momentum_animation_initial_velocity = entry.scroll_velocity;
        if entry.momentum_animation_initial_velocity.y().abs()
            >= entry.momentum_animation_initial_velocity.x().abs()
        {
            entry.momentum_animation_initial_velocity.set_x(0.0);
        }
        let max = context.max_scroll_offset(entry.target_scroller_id);
        if max.x() <= 0.0 {
            entry.momentum_animation_initial_velocity.set_x(0.0);
        }
        if max.y() <= 0.0 {
            entry.momentum_animation_initial_velocity.set_y(0.0);
        }
        context.request_one_begin_frame();
    }

    fn pinned(offset: f32, max: f32, direction: f32) -> bool {
        if direction < 0.0 {
            offset <= 0.0
        } else if direction > 0.0 {
            offset >= max
        } else {
            false
        }
    }

    fn stretch_amount_for_time(entry: &OverscrollEntry, elapsed: TimeDelta) -> Vector2dF {
        let seconds = elapsed.InSecondsF() as f32;
        let damping = ((-seconds * RUBBERBAND_STIFFNESS) / RUBBERBAND_PERIOD).exp();
        let velocity = Self::scale(
            entry.momentum_animation_initial_velocity,
            seconds * RUBBERBAND_AMPLITUDE,
        );
        Self::round_vector(Self::scale(
            entry.momentum_animation_initial_stretch + velocity,
            damping,
        ))
    }

    fn stretch_for_accumulated(force: Vector2dF) -> Vector2dF {
        Self::round_vector(Self::scale(force, 1.0 / RUBBERBAND_STIFFNESS.max(1.0)))
    }

    fn accumulated_overscroll(stretch: Vector2dF) -> Vector2dF {
        Self::round_vector(Self::scale(stretch, RUBBERBAND_STIFFNESS))
    }

    fn round_vector(value: Vector2dF) -> Vector2dF {
        Vector2dF::new(value.x().round(), value.y().round())
    }

    fn scale(value: Vector2dF, factor: f32) -> Vector2dF {
        Vector2dF::new(value.x() * factor, value.y() * factor)
    }
}
