// Translation status: complete
// Source: cc/trees/image_animation_controller.h:1-326;
//         cc/trees/image_animation_controller.cc:1-693.
// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/trees/image_animation_controller.h:1-48,322-326;
//         cc/trees/image_animation_controller.cc:1-21,691-693.
// Rust imports replace includes/namespace/exports. Per-call client facets are
// the source non-owning delegate; the injected runner capability owns no tree.
use crate::adapter::{
    begin_frame::BeginFrameArgs,
    image_animation::*,
    input_types::ElementId,
    time::{TimeDelta, TimeTicks},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
// Source: cc/trees/image_animation_controller.cc:23-26.
const kAnimationResyncCutoff: TimeDelta = TimeDelta::Seconds(5 * 60);
// Source: cc/trees/image_animation_controller.cc:28-40.
fn SnappedTickTimeFromFrameTime(args: &BeginFrameArgs, desired_frame_time: TimeTicks) -> TimeTicks {
    let mut snapped_tick_time =
        SnappedToNextTick(desired_frame_time, args.frame_time, args.interval);
    debug_assert!(snapped_tick_time >= desired_frame_time);
    if snapped_tick_time != desired_frame_time {
        snapped_tick_time -= args.interval;
    }
    snapped_tick_time
}
// Source: cc/trees/image_animation_controller.h:49-62.
// Delegate is included in the live ImageAnimationClient facet; no owner copy.
pub use crate::adapter::image_animation::ImageAnimationClient as Delegate;
// Source: cc/trees/image_animation_controller.h:303-322.
// C++ members destruct in reverse declaration order, Rust in forward order.
// Declare the fields in destruction order so scheduler weak cancellation and
// callback/runner capability release precede destruction of retained states.
pub struct ImageAnimationController {
    did_navigate_: bool,
    enable_image_animation_resync_: bool,
    scheduler_: InvalidationScheduler,
    advanced_animation_clients_: BTreeSet<ElementId>,
    images_animated_on_sync_tree_: PaintImageIdFlatSet,
    animation_state_map_: BTreeMap<PaintImageId, AnimationState>,
}
// Source: cc/trees/image_animation_controller.h:122.
pub type NowCallback = Box<dyn FnMut() -> TimeTicks>;
// Source: cc/trees/image_animation_controller.h:129-132.
pub struct ConsistentFrameDuration {
    pub frame_duration: TimeDelta,
    pub num_images: u32,
}
impl ImageAnimationController {
    // Source: cc/trees/image_animation_controller.h:64-71;
    //         cc/trees/image_animation_controller.cc:44-51.
    pub fn new(
        owner: ImageAnimationControllerId,
        task_runner: Box<dyn ImageAnimationTaskRunner>,
        enable_image_animation_resync: bool,
    ) -> Self {
        Self {
            animation_state_map_: BTreeMap::new(),
            images_animated_on_sync_tree_: BTreeSet::new(),
            advanced_animation_clients_: BTreeSet::new(),
            scheduler_: InvalidationScheduler::new(owner, task_runner),
            enable_image_animation_resync_: enable_image_animation_resync,
            did_navigate_: false,
        }
    }
    // Source: cc/trees/image_animation_controller.cc:53-57.
    pub fn UpdateAnimatedImage(&mut self, data: &AnimatedImageMetadata) {
        // C++ map[] inserts this image before UpdateMetadata, including self-targets.
        self.animation_state_map_
            .entry(data.paint_image_id)
            .or_default();
        // Only the active scalar selected by native find() crosses this split borrow.
        let target_active_index = self
            .animation_state_map_
            .get(&data.sync_animation_target_id)
            .map(AnimationState::active_index);
        self.animation_state_map_
            .get_mut(&data.paint_image_id)
            .unwrap()
            .UpdateMetadata(data, target_active_index);
    }
    // Source: cc/trees/image_animation_controller.cc:59-61.
    pub fn IsRegistered(&self, id: PaintImageId) -> bool {
        self.animation_state_map_.contains_key(&id)
    }
    // Source: cc/trees/image_animation_controller.cc:63-121.
    pub fn AnimateForSyncTree(
        &mut self,
        args: &BeginFrameArgs,
        driver_map: &AnimatedImageDriverMap,
        client: &mut dyn ImageAnimationClient,
    ) -> &PaintImageIdFlatSet {
        if client.TraceEnabled("ImageAnimationController::AnimateImagesForSyncTree") {
            client.Trace(
                "ImageAnimationController::AnimateImagesForSyncTree",
                ImageAnimationTraceValue::Duration(client.MonotonicTimeNow() - args.frame_time),
            );
        }
        debug_assert!(self.images_animated_on_sync_tree_.is_empty());
        self.scheduler_.WillAnimate();
        let mut next_invalidation_time: Option<TimeTicks> = None;
        for (&image_id, state) in &mut self.animation_state_map_ {
            let driver = driver_map.get(&image_id);
            state.UpdateStateFromDrivers(driver);
            if !state.ShouldAnimate() {
                client.Trace("ShouldAnimate - early out", ImageAnimationTraceValue::None);
                continue;
            }
            if state.AdvanceFrame(args, self.enable_image_animation_resync_, client) {
                self.images_animated_on_sync_tree_.insert(image_id);
                self.advanced_animation_clients_
                    .extend(driver.unwrap().1.iter().copied());
            }
            if client.TraceEnabled("AnimationState") {
                client.Trace(
                    "AnimationState",
                    ImageAnimationTraceValue::State(&state.ToString()),
                );
            }
            if !state.ShouldAnimate() {
                continue;
            }
            debug_assert!(state.next_desired_tick_time() > args.frame_time);
            next_invalidation_time = Some(match next_invalidation_time {
                None => state.next_desired_tick_time(),
                Some(t) => state.next_desired_tick_time().min(t),
            });
        }
        if let Some(t) = next_invalidation_time {
            self.scheduler_.Schedule(t, client);
        } else {
            self.scheduler_.Cancel();
        }
        &self.images_animated_on_sync_tree_
    }
    // Source: cc/trees/image_animation_controller.cc:123-155.
    pub fn UpdateStateFromDrivers(
        &mut self,
        map: &AnimatedImageDriverMap,
        client: &mut dyn ImageAnimationClient,
    ) {
        client.Trace(
            "ImageAnimationController::UpdateState",
            ImageAnimationTraceValue::None,
        );
        let mut next: Option<TimeTicks> = None;
        for (&id, state) in &mut self.animation_state_map_ {
            state.UpdateStateFromDrivers(map.get(&id));
            if !state.ShouldAnimate() {
                continue;
            }
            next = Some(match next {
                None => state.next_desired_tick_time(),
                Some(t) => t.min(state.next_desired_tick_time()),
            });
        }
        if let Some(t) = next {
            self.scheduler_.Schedule(t, client);
        } else {
            self.scheduler_.Cancel();
        }
    }
    // Source: cc/trees/image_animation_controller.cc:157-176.
    pub fn DidActivate(&mut self, client: &mut dyn ImageAnimationClient) {
        client.Trace(
            "ImageAnimationController::WillActivate",
            ImageAnimationTraceValue::None,
        );
        for id in &self.images_animated_on_sync_tree_ {
            self.animation_state_map_
                .get_mut(id)
                .expect("CHECK image exists")
                .PushPendingToActive();
        }
        self.images_animated_on_sync_tree_.clear();
        if self.did_navigate_ {
            self.animation_state_map_
                .retain(|_, state| state.has_drivers());
            self.did_navigate_ = false;
        }
    }
    // Source: cc/trees/image_animation_controller.cc:178-185.
    pub fn GetFrameIndexForImage(&self, id: PaintImageId, tree: WhichTree) -> usize {
        let state = self
            .animation_state_map_
            .get(&id)
            .expect("CHECK image exists");
        if tree == WhichTree::PENDING_TREE {
            state.pending_index()
        } else {
            state.active_index()
        }
    }
    // Source: cc/trees/image_animation_controller.cc:187-190.
    pub fn TakeAdvancedAnimationClients(&mut self) -> BTreeSet<ElementId> {
        std::mem::take(&mut self.advanced_animation_clients_)
    }
    // Source: cc/trees/image_animation_controller.h:104-106.
    pub fn HasAdvancedAnimationClients(&self) -> bool {
        !self.advanced_animation_clients_.is_empty()
    }
    // Source: cc/trees/image_animation_controller.cc:192-199.
    // This explicitly source-owned immutable refcounted index map is retained.
    pub fn GatherFrameIndexes(&self) -> Arc<AnimatedImageFrameIndexMap> {
        Arc::new(
            self.animation_state_map_
                .iter()
                .map(|(&id, s)| (id, s.pending_index()))
                .collect(),
        )
    }
    // Source: cc/trees/image_animation_controller.cc:201-204.
    pub fn WillBeginImplFrame(
        &mut self,
        args: &BeginFrameArgs,
        client: &mut dyn ImageAnimationClient,
    ) {
        self.scheduler_.WillBeginImplFrame(args, client);
    }
    // Source: cc/trees/image_animation_controller.h:112-126.
    pub fn did_navigate(&self) -> bool {
        self.did_navigate_
    }
    pub fn set_did_navigate(&mut self) {
        self.did_navigate_ = true;
    }
    pub fn animation_state_map_size_for_testing(&self) -> usize {
        self.animation_state_map_.len()
    }
    pub fn set_now_callback_for_testing(&mut self, cb: Option<NowCallback>) {
        self.scheduler_.set_now_callback_for_testing(cb);
    }
    // Source: cc/trees/image_animation_controller.cc:206-211.
    pub fn GetLastNumOfFramesSkippedForTesting(&self, id: PaintImageId) -> usize {
        self.animation_state_map_
            .get(&id)
            .expect("CHECK image exists")
            .last_num_frames_skipped_for_testing()
    }
    // Source: cc/trees/image_animation_controller.cc:213-240.
    pub fn GetConsistentContentFrameDuration(&mut self) -> Option<ConsistentFrameDuration> {
        if self.animation_state_map_.is_empty() {
            return None;
        }
        let mut duration = None;
        let mut num_images = 0;
        for state in self.animation_state_map_.values_mut() {
            if !state.ShouldAnimate() {
                continue;
            }
            let image_duration = state.GetConsistentContentFrameDuration()?;
            if duration.is_some_and(|d| d != image_duration) {
                return None;
            }
            duration = Some(image_duration);
            num_images += 1;
        }
        Some(ConsistentFrameDuration {
            frame_duration: duration?,
            num_images,
        })
    }
    // Source: cc/trees/image_animation_controller.cc:612-616 (weak BindOnce target).
    // The host returns queued tasks here; no closure takes ownership of `self`.
    pub fn RunDelayedTask(
        &mut self,
        task: &ImageAnimationTask,
        client: &mut dyn ImageAnimationClient,
    ) {
        if task.matches(self.scheduler_.owner_, &self.scheduler_.weak_factory_) {
            self.scheduler_.RequestBeginFrame(client);
        }
    }
}
// Source: cc/trees/image_animation_controller.h:139-260.
struct AnimationState {
    paint_image_id_: PaintImageId,
    frames_: Vec<FrameMetadata>,
    requested_repetitions_: i32,
    current_state_: AnimationAdvancementState,
    has_drivers_: bool,
    active_index_: usize,
    cached_consistent_frame_duration_: TimeDelta,
    cached_has_consistent_frame_duration_: bool,
    cached_consistent_frame_duration_valid_: bool,
    should_animate_from_drivers_: bool,
    animation_started_: bool,
    animation_started_time_: TimeTicks,
    reset_animation_sequence_id_: u32,
    sync_animation_sequence_id_: u32,
    completion_state_: CompletionState,
    last_num_frames_skipped_: usize,
    clients_: Vec<ElementId>,
}
// Source: cc/trees/image_animation_controller.h:174-193.
#[derive(Clone, Copy, Default)]
struct AnimationAdvancementState {
    pending_index: usize,
    next_desired_frame_time: TimeTicks,
    next_desired_tick_time: TimeTicks,
    repetitions_completed: i32,
    num_of_frames_advanced: usize,
}
// Source: cc/trees/image_animation_controller.cc:242-251 (defaulted move/dtor).
// Rust move/destruction are native ownership; AnimationState deliberately !Clone.
// cc/trees/image_animation_controller.cc:51 defaulted controller destructor uses
// the explicit Rust field destruction order declared above.
impl Default for AnimationState {
    fn default() -> Self {
        Self {
            paint_image_id_: kInvalidId,
            frames_: Vec::new(),
            requested_repetitions_: kAnimationNone,
            current_state_: AnimationAdvancementState::default(),
            has_drivers_: false,
            active_index_: kDefaultFrameIndex,
            cached_consistent_frame_duration_: TimeDelta::default(),
            cached_has_consistent_frame_duration_: false,
            cached_consistent_frame_duration_valid_: false,
            should_animate_from_drivers_: false,
            animation_started_: false,
            animation_started_time_: TimeTicks::default(),
            reset_animation_sequence_id_: 0,
            sync_animation_sequence_id_: 0,
            completion_state_: CompletionState::kPartiallyDone,
            last_num_frames_skipped_: 0,
            clients_: Vec::new(),
        }
    }
}
impl AnimationState {
    // Source: cc/trees/image_animation_controller.cc:253-297.
    fn ShouldAnimate(&self) -> bool {
        self.ShouldAnimateAt(
            self.current_state_.repetitions_completed,
            self.current_state_.pending_index,
        )
    }
    fn ShouldAnimateAt(&self, repetitions_completed: i32, pending_index: usize) -> bool {
        debug_assert!(self.current_state_.repetitions_completed == 0 || self.is_complete());
        if !self.should_animate_from_drivers_ {
            return false;
        }
        match self.requested_repetitions_ {
            kAnimationLoopOnce => {
                if repetitions_completed >= 1 {
                    return false;
                }
            }
            kAnimationPaused => return false,
            kAnimationNone => unreachable!("We shouldn't be tracking kAnimationNone images"),
            kAnimationLoopInfinite => {}
            n => {
                if n <= repetitions_completed {
                    return false;
                }
            }
        }
        if !self.frames_[self.NextFrameIndex(pending_index)].complete {
            return false;
        }
        let last_frame_index = self.frames_.len() - 1;
        if self.completion_state_ != CompletionState::kDone && pending_index == last_frame_index {
            return false;
        }
        true
    }
    // Source: cc/trees/image_animation_controller.cc:309-363.
    fn AdvanceFrame(
        &mut self,
        args: &BeginFrameArgs,
        resync: bool,
        client: &mut dyn ImageAnimationClient,
    ) -> bool {
        debug_assert!(self.ShouldAnimateAt(
            self.current_state_.repetitions_completed,
            self.current_state_.pending_index
        ));
        let next_tick_time = args.frame_time + args.interval;
        if !self.animation_started_ {
            debug_assert_eq!(self.current_state_.pending_index, 0);
            self.animation_started_time_ = args.frame_time;
            self.current_state_.next_desired_frame_time =
                args.frame_time + self.frames_[0].duration;
            self.current_state_.next_desired_tick_time =
                SnappedTickTimeFromFrameTime(args, self.current_state_.next_desired_frame_time);
            self.animation_started_ = true;
        }
        if args.frame_time < self.current_state_.next_desired_tick_time {
            return self.needs_invalidation();
        }
        if resync
            && args.frame_time - self.current_state_.next_desired_frame_time
                > kAnimationResyncCutoff
        {
            client.Trace("Resync - early out", ImageAnimationTraceValue::None);
            debug_assert_eq!(self.current_state_.pending_index, self.active_index_);
            self.current_state_.next_desired_frame_time =
                args.frame_time + self.frames_[self.current_state_.pending_index].duration;
            self.current_state_.next_desired_tick_time =
                SnappedTickTimeFromFrameTime(args, self.current_state_.next_desired_frame_time)
                    .max(next_tick_time);
            return self.needs_invalidation();
        }
        self.current_state_.num_of_frames_advanced = 0;
        self.current_state_ = self.AdvanceAnimationState(
            self.current_state_,
            args,
            self.current_state_.next_desired_tick_time,
            resync,
            client,
        );
        debug_assert!(self.current_state_.num_of_frames_advanced >= 1);
        self.last_num_frames_skipped_ = self.current_state_.num_of_frames_advanced - 1;
        self.needs_invalidation()
    }
    // Source: cc/trees/image_animation_controller.cc:365-431.
    fn AdvanceAnimationState(
        &self,
        mut state: AnimationAdvancementState,
        args: &BeginFrameArgs,
        start: TimeTicks,
        resync: bool,
        client: &mut dyn ImageAnimationClient,
    ) -> AnimationAdvancementState {
        let end = args.frame_time + args.interval;
        let mut elapsed_time = start;
        while elapsed_time < end
            && self.ShouldAnimateAt(state.repetitions_completed, state.pending_index)
        {
            state.num_of_frames_advanced += 1;
            let next_frame_index = self.NextFrameIndex(state.pending_index);
            elapsed_time += self.frames_[next_frame_index].duration;
            client.Trace(
                "FrameDurationIndex",
                ImageAnimationTraceValue::Frame {
                    index: next_frame_index,
                    duration: self.frames_[next_frame_index].duration,
                },
            );
            let next_desired_frame_time =
                state.next_desired_frame_time + self.frames_[next_frame_index].duration;
            if resync
                && next_frame_index == 0
                && state.repetitions_completed == 1
                && next_desired_frame_time <= args.frame_time
            {
                state.pending_index = 0;
                state.next_desired_frame_time = args.frame_time + self.frames_[0].duration;
                state.next_desired_tick_time =
                    SnappedTickTimeFromFrameTime(args, state.next_desired_frame_time).max(end);
                state.repetitions_completed = 0;
                break;
            }
            state.pending_index = next_frame_index;
            state.next_desired_frame_time = next_desired_frame_time;
            state.next_desired_tick_time =
                SnappedTickTimeFromFrameTime(args, state.next_desired_frame_time);
            if state.pending_index == self.frames_.len() - 1 && self.is_complete() {
                state.repetitions_completed += 1;
            }
        }
        debug_assert!(state.num_of_frames_advanced > 0);
        state
    }
    // Source: cc/trees/image_animation_controller.cc:433-486.
    fn UpdateMetadata(&mut self, data: &AnimatedImageMetadata, target_active_index: Option<usize>) {
        self.paint_image_id_ = data.paint_image_id;
        debug_assert_ne!(data.repetition_count, kAnimationNone);
        self.requested_repetitions_ = data.repetition_count;
        debug_assert!(
            self.frames_.len() <= data.frames.len(),
            "Updated recordings can only append frames"
        );
        self.frames_ = data.frames.clone();
        self.cached_consistent_frame_duration_valid_ = false;
        debug_assert!(self.frames_.len() > 1);
        debug_assert!(
            self.completion_state_ != CompletionState::kDone
                || data.completion_state == CompletionState::kDone,
            "Complete images cannot become incomplete"
        );
        self.completion_state_ = data.completion_state;
        let last = self.frames_.len() - 1;
        if self.current_state_.pending_index == last
            && self.is_complete()
            && self.current_state_.repetitions_completed == 0
        {
            self.current_state_.repetitions_completed += 1;
        }
        if data.repetition_count == kAnimationPaused {
            self.current_state_.next_desired_frame_time = TimeTicks::default();
        }
        if self.sync_animation_sequence_id_ != data.sync_animation_sequence_id {
            self.current_state_.next_desired_frame_time = TimeTicks::default();
            if data.sync_animation_target_id != kInvalidId {
                if let Some(index) = target_active_index {
                    self.current_state_.pending_index = index;
                    self.animation_started_ = true;
                }
            } else {
                self.current_state_.pending_index = 0;
            }
            self.sync_animation_sequence_id_ = data.sync_animation_sequence_id;
            self.PushPendingToActive();
        }
        if self.reset_animation_sequence_id_ < data.reset_animation_sequence_id {
            self.reset_animation_sequence_id_ = data.reset_animation_sequence_id;
            self.ResetAnimation();
        }
    }
    // Source: cc/trees/image_animation_controller.cc:488-490.
    fn PushPendingToActive(&mut self) {
        self.active_index_ = self.current_state_.pending_index;
    }
    // Source: cc/trees/image_animation_controller.cc:492-518.
    fn GetConsistentContentFrameDuration(&mut self) -> Option<TimeDelta> {
        if !self.cached_consistent_frame_duration_valid_ {
            self.ComputeConsistentContentFrameDuration();
        }
        self.cached_consistent_frame_duration_valid_ = true;
        if !self.cached_has_consistent_frame_duration_ {
            return None;
        }
        Some(self.cached_consistent_frame_duration_)
    }
    fn ComputeConsistentContentFrameDuration(&mut self) {
        self.cached_has_consistent_frame_duration_ = false;
        let mut frame_duration = None;
        for metadata in &self.frames_ {
            if frame_duration.is_some_and(|d| d != metadata.duration) {
                return;
            }
            frame_duration = Some(metadata.duration);
        }
        if let Some(d) = frame_duration {
            self.cached_has_consistent_frame_duration_ = true;
            self.cached_consistent_frame_duration_ = d;
        }
    }
    // Source: cc/trees/image_animation_controller.cc:520-531.
    fn UpdateStateFromDrivers(&mut self, driver: Option<&AnimatedImageDriverState>) {
        if let Some(driver) = driver {
            self.has_drivers_ = true;
            self.should_animate_from_drivers_ = driver.0;
            self.clients_ = driver.1.clone();
        } else {
            self.has_drivers_ = false;
            self.should_animate_from_drivers_ = false;
            self.clients_.clear();
        }
    }
    // Source: cc/trees/image_animation_controller.cc:533-539.
    fn ResetAnimation(&mut self) {
        self.animation_started_ = false;
        self.current_state_.next_desired_frame_time = TimeTicks::default();
        self.current_state_.repetitions_completed = 0;
        self.current_state_.pending_index = 0;
    }
    // Source: cc/trees/image_animation_controller.cc:541-556.
    fn ToString(&self) -> String {
        // Keep the source diagnostic's requested_repetitions value in its
        // repetitions_completed label, even though it is surprising.
        format!("paint_image_id[{}]\nrequested_repetitions[{}]\nrepetitions_completed[{}]\nactive_index[{}]\npending_index[{}]\nnext_desired_frame_time[{}]\nnext_desired_tick_time[{}]\nshould_animate_from_drivers[{}]\ncompletion_state[{}]",self.paint_image_id_.0,self.requested_repetitions_,self.requested_repetitions_,self.active_index_,self.current_state_.pending_index,(self.current_state_.next_desired_frame_time-self.animation_started_time_).InMicrosecondsF()/1000.,(self.current_state_.next_desired_tick_time-self.animation_started_time_).InMicrosecondsF()/1000.,i32::from(self.should_animate_from_drivers_),self.completion_state_ as i32)
    }
    // Source: cc/trees/image_animation_controller.cc:558-563.
    fn NextFrameIndex(&self, index: usize) -> usize {
        if !self.animation_started_ {
            return 0;
        }
        (index + 1) % self.frames_.len()
    }
    // Source: cc/trees/image_animation_controller.h:160-170,202-207.
    fn has_drivers(&self) -> bool {
        self.has_drivers_
    }
    fn pending_index(&self) -> usize {
        self.current_state_.pending_index
    }
    fn active_index(&self) -> usize {
        self.active_index_
    }
    fn next_desired_tick_time(&self) -> TimeTicks {
        self.current_state_.next_desired_tick_time
    }
    fn last_num_frames_skipped_for_testing(&self) -> usize {
        self.last_num_frames_skipped_
    }
    fn is_complete(&self) -> bool {
        self.completion_state_ == CompletionState::kDone
    }
    fn needs_invalidation(&self) -> bool {
        self.current_state_.pending_index != self.active_index_
    }
}
// Source: cc/trees/image_animation_controller.h:262-301.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum InvalidationState {
    kIdle,
    kPendingRequestBeginFrame,
    kPendingImplFrame,
    kPendingInvalidation,
}
// As above, weak_factory_ drops before callbacks and the runner capability.
struct InvalidationScheduler {
    weak_factory_: Arc<()>,
    next_animation_time_: TimeTicks,
    state_: InvalidationState,
    now_callback_for_testing_: Option<NowCallback>,
    task_runner_: Box<dyn ImageAnimationTaskRunner>,
    owner_: ImageAnimationControllerId,
}
impl InvalidationScheduler {
    // Source: cc/trees/image_animation_controller.h:272-274.
    fn set_now_callback_for_testing(&mut self, cb: Option<NowCallback>) {
        self.now_callback_for_testing_ = cb;
    }
    // Source: cc/trees/image_animation_controller.cc:565-570.
    fn new(owner: ImageAnimationControllerId, runner: Box<dyn ImageAnimationTaskRunner>) -> Self {
        debug_assert!(runner.BelongsToCurrentThread());
        Self {
            task_runner_: runner,
            owner_: owner,
            now_callback_for_testing_: None,
            state_: InvalidationState::kIdle,
            next_animation_time_: TimeTicks::default(),
            weak_factory_: Arc::new(()),
        }
    }
    // Source: cc/trees/image_animation_controller.cc:576-617.
    fn Schedule(&mut self, mut animation_time: TimeTicks, client: &mut dyn ImageAnimationClient) {
        let now = if let Some(cb) = &mut self.now_callback_for_testing_ {
            cb()
        } else {
            client.MonotonicTimeNow()
        };
        if matches!(
            self.state_,
            InvalidationState::kPendingInvalidation | InvalidationState::kPendingImplFrame
        ) {
            return;
        }
        if animation_time < now {
            animation_time = now;
        }
        if self.state_ == InvalidationState::kPendingRequestBeginFrame
            && animation_time == self.next_animation_time_
        {
            return;
        }
        self.Cancel();
        let delta = animation_time - now;
        client.Trace(
            "ScheduleFrameForImageAnimation",
            ImageAnimationTraceValue::Duration(delta),
        );
        self.state_ = InvalidationState::kPendingRequestBeginFrame;
        self.next_animation_time_ = animation_time;
        self.task_runner_.PostDelayedTask(
            ImageAnimationTask::new(self.owner_, &self.weak_factory_),
            delta,
        );
    }
    // Source: cc/trees/image_animation_controller.cc:619-622.
    fn Cancel(&mut self) {
        self.state_ = InvalidationState::kIdle;
        self.weak_factory_ = Arc::new(());
    }
    // Source: cc/trees/image_animation_controller.cc:624-632.
    fn RequestBeginFrame(&mut self, client: &mut dyn ImageAnimationClient) {
        client.Trace(
            "ImageAnimationController::InvalidationScheduler::RequestBeginFrame",
            ImageAnimationTraceValue::None,
        );
        debug_assert_eq!(self.state_, InvalidationState::kPendingRequestBeginFrame);
        self.state_ = InvalidationState::kPendingImplFrame;
        client.RequestBeginFrameForAnimatedImages();
    }
    // Source: cc/trees/image_animation_controller.cc:634-642.
    fn WillAnimate(&mut self) {
        if self.state_ != InvalidationState::kPendingInvalidation {
            return;
        }
        self.state_ = InvalidationState::kIdle;
        self.next_animation_time_ = TimeTicks::default();
    }
    // Source: cc/trees/image_animation_controller.cc:644-675.
    fn WillBeginImplFrame(&mut self, args: &BeginFrameArgs, client: &mut dyn ImageAnimationClient) {
        match self.state_ {
            InvalidationState::kIdle | InvalidationState::kPendingInvalidation => {}
            InvalidationState::kPendingRequestBeginFrame => {
                if args.frame_time >= self.next_animation_time_ {
                    self.RequestInvalidation(client);
                }
            }
            InvalidationState::kPendingImplFrame => {
                if args.frame_time >= self.next_animation_time_ {
                    self.RequestInvalidation(client);
                } else {
                    client.RequestBeginFrameForAnimatedImages();
                }
            }
        }
    }
    // Source: cc/trees/image_animation_controller.cc:677-691.
    fn RequestInvalidation(&mut self, client: &mut dyn ImageAnimationClient) {
        client.Trace(
            "ImageAnimationController::InvalidationScheduler::RequestInvalidation",
            ImageAnimationTraceValue::None,
        );
        debug_assert_ne!(self.state_, InvalidationState::kIdle);
        debug_assert_ne!(self.state_, InvalidationState::kPendingInvalidation);
        self.Cancel();
        self.state_ = InvalidationState::kPendingInvalidation;
        client.RequestInvalidationForAnimatedImages();
    }
}
// Source: cc/trees/image_animation_controller.cc:572-574.
impl Drop for InvalidationScheduler {
    fn drop(&mut self) {
        debug_assert!(self.task_runner_.BelongsToCurrentThread());
        // weak_factory_ is the first dropped field, as in native reverse order.
    }
}
