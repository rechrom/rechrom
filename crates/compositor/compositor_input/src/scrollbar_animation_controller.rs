// Translation status: complete
// Source: cc/input/scrollbar_animation_controller.h:1-179
// Source: cc/input/scrollbar_animation_controller.cc:1-409
// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
use super::{
    scrollbar::ScrollbarOrientation,
    single_scrollbar_animation_controller_thinning::SingleScrollbarAnimationControllerThinning,
};
use crate::{
    input_types::{ElementId, InputLayerId},
    scrollbar_animation::ScrollbarAnimationTask,
    scrollbar_controller_boundary::{ScrollbarInitializationClient, ScrollbarLayerAccess},
};
use compositor_scheduler::{TimeDelta, TimeTicks};
use compositor_tree::geometry::{cpp_clamp, cpp_max, cpp_min, PointF};
use std::sync::Arc;

// Source: cc/input/scrollbar_animation_controller.h:22-34
pub trait ScrollbarAnimationControllerClient: ScrollbarLayerAccess {
    /// Queue after this call stack; deliver via RunDelayedAnimationTask if alive.
    fn PostDelayedScrollbarAnimationTask(&mut self, task: ScrollbarAnimationTask, delay: TimeDelta);
    fn SetNeedsRedrawForScrollbarAnimation(&mut self);
    fn SetNeedsAnimateForScrollbarAnimation(&mut self);
    fn DidChangeScrollbarVisibility(&mut self);
    fn ScrollbarsFor(&mut self, element: ElementId) -> Vec<InputLayerId>;
    fn IsFluentOverlayScrollbar(&mut self) -> bool;
    /// Adapter identity for routing weak callbacks, not a second animation domain.
    fn animation_controller_id(&self, element: ElementId) -> u64;
}
// Source: cc/input/scrollbar_animation_controller.h:105
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AnimationChange {
    kNone,
    kFadeIn,
    kFadeOut,
}
// Source: cc/input/scrollbar_animation_controller.h:141-174
pub struct ScrollbarAnimationController {
    last_awaken_time_: TimeTicks,
    fade_delay_: TimeDelta,
    fade_duration_: TimeDelta,
    need_trigger_scrollbar_fade_in_: bool,
    is_animating_: bool,
    animation_change_: AnimationChange,
    scroll_element_id_: ElementId,
    delayed_scrollbar_animation_: Option<ScrollbarAnimationTask>,
    opacity_: f32,
    show_scrollbars_on_scroll_gesture_: bool,
    need_thinning_animation_: bool,
    is_mouse_down_: bool,
    tickmarks_showing_: bool,
    visibility_changed_: bool,
    vertical_controller_: Option<Box<SingleScrollbarAnimationControllerThinning>>,
    horizontal_controller_: Option<Box<SingleScrollbarAnimationControllerThinning>>,
    weak_factory_: Option<Arc<()>>,
    task_generation_: u64,
}
impl ScrollbarAnimationController {
    // Source: cc/input/scrollbar_animation_controller.cc:18-26
    pub fn CreateScrollbarAnimationControllerAndroid<C: ScrollbarInitializationClient + ?Sized>(
        element: ElementId,
        client: &mut C,
        fade_delay: TimeDelta,
        fade_duration: TimeDelta,
        opacity: f32,
    ) -> Box<Self> {
        Box::new(Self::new_android(
            element,
            client,
            fade_delay,
            fade_duration,
            opacity,
        ))
    }
    // Source: cc/input/scrollbar_animation_controller.cc:29-40
    pub fn CreateScrollbarAnimationControllerAuraOverlay<
        C: ScrollbarInitializationClient + ?Sized,
    >(
        element: ElementId,
        client: &mut C,
        fade_delay: TimeDelta,
        fade_duration: TimeDelta,
        thinning_duration: TimeDelta,
        opacity: f32,
        idle_scale: f32,
    ) -> Box<Self> {
        Box::new(Self::new_aura(
            element,
            client,
            fade_delay,
            fade_duration,
            thinning_duration,
            opacity,
            idle_scale,
        ))
    }
    // Source: cc/input/scrollbar_animation_controller.cc:42-59
    fn new_android<C: ScrollbarInitializationClient + ?Sized>(
        element: ElementId,
        _client: &mut C,
        fade_delay: TimeDelta,
        fade_duration: TimeDelta,
        opacity: f32,
    ) -> Self {
        Self {
            last_awaken_time_: TimeTicks::default(),
            fade_delay_: fade_delay,
            fade_duration_: fade_duration,
            need_trigger_scrollbar_fade_in_: false,
            is_animating_: false,
            animation_change_: AnimationChange::kNone,
            scroll_element_id_: element,
            delayed_scrollbar_animation_: None,
            opacity_: opacity,
            show_scrollbars_on_scroll_gesture_: false,
            need_thinning_animation_: false,
            is_mouse_down_: false,
            tickmarks_showing_: false,
            visibility_changed_: false,
            vertical_controller_: None,
            horizontal_controller_: None,
            weak_factory_: Some(Arc::new(())),
            task_generation_: 0,
        }
    }
    // Source: cc/input/scrollbar_animation_controller.cc:61-87
    fn new_aura<C: ScrollbarInitializationClient + ?Sized>(
        element: ElementId,
        client: &mut C,
        fade_delay: TimeDelta,
        fade_duration: TimeDelta,
        thinning_duration: TimeDelta,
        opacity: f32,
        idle_scale: f32,
    ) -> Self {
        let mut result = Self {
            last_awaken_time_: TimeTicks::default(),
            fade_delay_: fade_delay,
            fade_duration_: fade_duration,
            need_trigger_scrollbar_fade_in_: false,
            is_animating_: false,
            animation_change_: AnimationChange::kNone,
            scroll_element_id_: element,
            delayed_scrollbar_animation_: None,
            opacity_: opacity,
            show_scrollbars_on_scroll_gesture_: true,
            need_thinning_animation_: true,
            is_mouse_down_: false,
            tickmarks_showing_: false,
            visibility_changed_: false,
            vertical_controller_: None,
            horizontal_controller_: None,
            weak_factory_: Some(Arc::new(())),
            task_generation_: 0,
        };
        result.vertical_controller_ = Some(SingleScrollbarAnimationControllerThinning::Create(
            element,
            ScrollbarOrientation::kVertical,
            client,
            thinning_duration,
            idle_scale,
        ));
        result.horizontal_controller_ = Some(SingleScrollbarAnimationControllerThinning::Create(
            element,
            ScrollbarOrientation::kHorizontal,
            client,
            thinning_duration,
            idle_scale,
        ));
        result
    }
    // Source: cc/input/scrollbar_animation_controller.h:69-70
    pub fn visibility_changed(&self) -> bool {
        self.visibility_changed_
    }
    pub fn ClearVisibilityChanged(&mut self) {
        self.visibility_changed_ = false;
    }
    // Source: cc/input/scrollbar_animation_controller.cc:91-93
    pub fn Scrollbars<C: ScrollbarAnimationControllerClient + ?Sized>(
        &self,
        client: &mut C,
    ) -> Vec<InputLayerId> {
        client.ScrollbarsFor(self.scroll_element_id_)
    }
    // Source: cc/input/scrollbar_animation_controller.cc:96-104
    pub fn GetScrollbarAnimationController(
        &self,
        orientation: ScrollbarOrientation,
    ) -> &SingleScrollbarAnimationControllerThinning {
        debug_assert!(self.need_thinning_animation_);
        if orientation == ScrollbarOrientation::kVertical {
            self.vertical_controller_.as_deref().unwrap()
        } else {
            self.horizontal_controller_.as_deref().unwrap()
        }
    }
    // C++ returns a mutable reference from the same const method. Rust makes
    // the mutable borrow explicit, with the identical orientation selection.
    pub fn GetScrollbarAnimationControllerMut(
        &mut self,
        orientation: ScrollbarOrientation,
    ) -> &mut SingleScrollbarAnimationControllerThinning {
        debug_assert!(self.need_thinning_animation_);
        if orientation == ScrollbarOrientation::kVertical {
            self.vertical_controller_.as_deref_mut().unwrap()
        } else {
            self.horizontal_controller_.as_deref_mut().unwrap()
        }
    }
    // Source: cc/input/scrollbar_animation_controller.cc:106-113
    fn StartAnimation<C: ScrollbarAnimationControllerClient + ?Sized>(&mut self, client: &mut C) {
        debug_assert_ne!(self.animation_change_, AnimationChange::kNone);
        self.CancelDelayedAnimation();
        self.need_trigger_scrollbar_fade_in_ = false;
        self.is_animating_ = true;
        self.last_awaken_time_ = TimeTicks::default();
        client.SetNeedsAnimateForScrollbarAnimation();
    }
    // Source: cc/input/scrollbar_animation_controller.cc:115-120
    fn StopAnimation(&mut self) {
        self.CancelDelayedAnimation();
        self.need_trigger_scrollbar_fade_in_ = false;
        self.is_animating_ = false;
        self.animation_change_ = AnimationChange::kNone;
    }
    // Source: cc/input/scrollbar_animation_controller.cc:122-136
    fn PostDelayedAnimation<C: ScrollbarAnimationControllerClient + ?Sized>(
        &mut self,
        client: &mut C,
        change: AnimationChange,
    ) {
        if self.fade_delay_ == TimeDelta::Max() {
            return;
        }
        self.animation_change_ = change;
        self.CancelDelayedAnimation();
        self.task_generation_ = self.task_generation_.wrapping_add(1);
        let task = ScrollbarAnimationTask::new(
            client.animation_controller_id(self.scroll_element_id_),
            self.task_generation_,
            self.weak_factory_.as_ref().unwrap(),
        );
        self.delayed_scrollbar_animation_ = Some(task.clone());
        client.PostDelayedScrollbarAnimationTask(task, self.fade_delay_);
    }
    // Source: cc/input/scrollbar_animation_controller.cc:138-167
    pub fn Animate<C: ScrollbarAnimationControllerClient + ?Sized>(
        &mut self,
        client: &mut C,
        now: TimeTicks,
    ) -> bool {
        let mut animated = false;
        for scrollbar in self.Scrollbars(client) {
            if !client.Scrollbar_CanScrollOrientation(&scrollbar) {
                client.Scrollbar_SetOverlayScrollbarLayerOpacityAnimated(&scrollbar, 0., false);
            }
        }
        if self.is_animating_ {
            debug_assert_ne!(self.animation_change_, AnimationChange::kNone);
            if self.last_awaken_time_.is_null() {
                self.last_awaken_time_ = now;
            }
            let progress = self.AnimationProgressAtTime(now);
            self.RunAnimationFrame(client, progress);
            if self.is_animating_ {
                client.SetNeedsAnimateForScrollbarAnimation();
            }
            animated = true;
        }
        if self.need_thinning_animation_ {
            animated |= self
                .vertical_controller_
                .as_mut()
                .unwrap()
                .Animate(client, now);
            animated |= self
                .horizontal_controller_
                .as_mut()
                .unwrap()
                .Animate(client, now);
        }
        animated
    }
    // Source: cc/input/scrollbar_animation_controller.cc:169-173
    fn AnimationProgressAtTime(&self, now: TimeTicks) -> f32 {
        let delta = now - self.last_awaken_time_;
        cpp_clamp((delta / self.fade_duration_) as f32, 0., 1.)
    }
    // Source: cc/input/scrollbar_animation_controller.cc:175-188
    fn RunAnimationFrame<C: ScrollbarAnimationControllerClient + ?Sized>(
        &mut self,
        client: &mut C,
        progress: f32,
    ) {
        debug_assert_ne!(self.animation_change_, AnimationChange::kNone);
        let opacity;
        if self.animation_change_ == AnimationChange::kFadeIn {
            opacity = cpp_max(progress, self.opacity_);
        } else {
            opacity = cpp_min(1. - progress, self.opacity_);
        }
        self.ApplyOpacityToScrollbars(client, opacity);
        if progress == 1. {
            self.StopAnimation();
        }
    }
    // Source: cc/input/scrollbar_animation_controller.cc:190-196
    pub fn DidScrollUpdate<C: ScrollbarAnimationControllerClient + ?Sized>(
        &mut self,
        client: &mut C,
    ) {
        self.UpdateScrollbarState(client);
        if self.need_thinning_animation_ {
            self.vertical_controller_
                .as_mut()
                .unwrap()
                .DidScrollUpdate(client);
            self.horizontal_controller_
                .as_mut()
                .unwrap()
                .DidScrollUpdate(client);
        }
    }
    // Source: cc/input/scrollbar_animation_controller.cc:198-214
    fn UpdateScrollbarState<C: ScrollbarAnimationControllerClient + ?Sized>(
        &mut self,
        client: &mut C,
    ) {
        if self.need_thinning_animation_ && self.Captured() {
            return;
        }
        self.StopAnimation();
        self.Show(client);
        if self.need_thinning_animation_ {
            if !self.MouseIsNearAnyScrollbar() && !self.tickmarks_showing_ {
                self.PostDelayedAnimation(client, AnimationChange::kFadeOut);
            }
        } else {
            self.PostDelayedAnimation(client, AnimationChange::kFadeOut);
        }
    }
    // Source: cc/input/scrollbar_animation_controller.cc:216-224
    pub fn WillUpdateScroll<C: ScrollbarAnimationControllerClient + ?Sized>(
        &mut self,
        client: &mut C,
    ) {
        if self.show_scrollbars_on_scroll_gesture_ {
            self.UpdateScrollbarState(client);
            if self.need_thinning_animation_ {
                self.vertical_controller_
                    .as_mut()
                    .unwrap()
                    .DidRequestShow(client);
                self.horizontal_controller_
                    .as_mut()
                    .unwrap()
                    .DidRequestShow(client);
            }
        }
    }
    // Source: cc/input/scrollbar_animation_controller.cc:226-232
    pub fn DidRequestShow<C: ScrollbarAnimationControllerClient + ?Sized>(
        &mut self,
        client: &mut C,
    ) {
        self.UpdateScrollbarState(client);
        if self.need_thinning_animation_ {
            self.vertical_controller_
                .as_mut()
                .unwrap()
                .DidRequestShow(client);
            self.horizontal_controller_
                .as_mut()
                .unwrap()
                .DidRequestShow(client);
        }
    }
    // Source: cc/input/scrollbar_animation_controller.cc:234-244
    pub fn UpdateTickmarksVisibility<C: ScrollbarAnimationControllerClient + ?Sized>(
        &mut self,
        client: &mut C,
        show: bool,
    ) {
        if !self.need_thinning_animation_ {
            return;
        }
        if self.tickmarks_showing_ == show {
            return;
        }
        self.tickmarks_showing_ = show;
        self.vertical_controller_
            .as_mut()
            .unwrap()
            .UpdateTickmarksVisibility(client, show);
        self.UpdateScrollbarState(client);
    }
    // Source: cc/input/scrollbar_animation_controller.cc:246-262
    pub fn DidMouseDown<C: ScrollbarAnimationControllerClient + ?Sized>(&mut self, client: &mut C) {
        if !self.need_thinning_animation_ {
            return;
        }
        self.is_mouse_down_ = true;
        if self.ScrollbarsHidden() {
            if self.need_trigger_scrollbar_fade_in_ {
                self.CancelDelayedAnimation();
                self.need_trigger_scrollbar_fade_in_ = false;
            }
            return;
        }
        self.vertical_controller_
            .as_mut()
            .unwrap()
            .DidMouseDown(client);
        self.horizontal_controller_
            .as_mut()
            .unwrap()
            .DidMouseDown(client);
    }
    // Source: cc/input/scrollbar_animation_controller.cc:264-283
    pub fn DidMouseUp<C: ScrollbarAnimationControllerClient + ?Sized>(&mut self, client: &mut C) {
        if !self.need_thinning_animation_ {
            return;
        }
        self.is_mouse_down_ = false;
        if !self.Captured() {
            if self.MouseIsNearAnyScrollbar() && self.ScrollbarsHidden() {
                self.PostDelayedAnimation(client, AnimationChange::kFadeIn);
                self.need_trigger_scrollbar_fade_in_ = true;
            }
            return;
        }
        self.vertical_controller_
            .as_mut()
            .unwrap()
            .DidMouseUp(client);
        self.horizontal_controller_
            .as_mut()
            .unwrap()
            .DidMouseUp(client);
        if !self.MouseIsNearAnyScrollbar() && !self.ScrollbarsHidden() && !self.tickmarks_showing_ {
            self.PostDelayedAnimation(client, AnimationChange::kFadeOut);
        }
    }
    // Source: cc/input/scrollbar_animation_controller.cc:285-299
    pub fn DidMouseLeave<C: ScrollbarAnimationControllerClient + ?Sized>(
        &mut self,
        client: &mut C,
    ) {
        if !self.need_thinning_animation_ {
            return;
        }
        self.vertical_controller_
            .as_mut()
            .unwrap()
            .DidMouseLeave(client);
        self.horizontal_controller_
            .as_mut()
            .unwrap()
            .DidMouseLeave(client);
        self.CancelDelayedAnimation();
        self.need_trigger_scrollbar_fade_in_ = false;
        if self.ScrollbarsHidden() || self.Captured() || self.tickmarks_showing_ {
            return;
        }
        self.PostDelayedAnimation(client, AnimationChange::kFadeOut);
    }
    // Source: cc/input/scrollbar_animation_controller.cc:301-340
    pub fn DidMouseMove<C: ScrollbarAnimationControllerClient + ?Sized>(
        &mut self,
        client: &mut C,
        point: PointF,
    ) {
        if !self.need_thinning_animation_ {
            return;
        }
        let need_before = self.need_trigger_scrollbar_fade_in_;
        self.vertical_controller_
            .as_mut()
            .unwrap()
            .DidMouseMove(client, point);
        self.horizontal_controller_
            .as_mut()
            .unwrap()
            .DidMouseMove(client, point);
        if self.Captured() || self.tickmarks_showing_ {
            debug_assert!(!self.ScrollbarsHidden());
            return;
        }
        if self.ScrollbarsHidden() {
            if self.is_mouse_down_ || client.IsFluentOverlayScrollbar() {
                return;
            }
            self.need_trigger_scrollbar_fade_in_ = self.MouseIsNearAnyScrollbar();
            if need_before != self.need_trigger_scrollbar_fade_in_ {
                if self.need_trigger_scrollbar_fade_in_ {
                    self.PostDelayedAnimation(client, AnimationChange::kFadeIn);
                } else {
                    self.CancelDelayedAnimation();
                }
            }
        } else {
            if self.MouseIsNearAnyScrollbar() {
                self.Show(client);
                self.StopAnimation();
            } else if !self.is_animating_ || client.IsFluentOverlayScrollbar() {
                self.PostDelayedAnimation(client, AnimationChange::kFadeOut);
            }
        }
    }
    // Source: cc/input/scrollbar_animation_controller.cc:342-347
    pub fn MouseIsOverScrollbarThumb(&self, o: ScrollbarOrientation) -> bool {
        debug_assert!(self.need_thinning_animation_);
        self.GetScrollbarAnimationController(o)
            .mouse_is_over_scrollbar_thumb()
    }
    // Source: cc/input/scrollbar_animation_controller.cc:349-354
    pub fn MouseIsNearScrollbarThumb(&self, o: ScrollbarOrientation) -> bool {
        debug_assert!(self.need_thinning_animation_);
        self.GetScrollbarAnimationController(o)
            .mouse_is_near_scrollbar_thumb()
    }
    // Source: cc/input/scrollbar_animation_controller.cc:356-360
    pub fn MouseIsNearScrollbar(&self, o: ScrollbarOrientation) -> bool {
        debug_assert!(self.need_thinning_animation_);
        self.GetScrollbarAnimationController(o)
            .mouse_is_near_scrollbar()
    }
    // Source: cc/input/scrollbar_animation_controller.cc:362-366
    pub fn MouseIsNearAnyScrollbar(&self) -> bool {
        debug_assert!(self.need_thinning_animation_);
        self.vertical_controller_
            .as_ref()
            .unwrap()
            .mouse_is_near_scrollbar()
            || self
                .horizontal_controller_
                .as_ref()
                .unwrap()
                .mouse_is_near_scrollbar()
    }
    // Source: cc/input/scrollbar_animation_controller.cc:368-370
    pub fn ScrollbarsHidden(&self) -> bool {
        self.opacity_ == 0.
    }
    // Source: cc/input/scrollbar_animation_controller.cc:372-378
    fn Captured(&self) -> bool {
        debug_assert!(self.need_thinning_animation_);
        self.GetScrollbarAnimationController(ScrollbarOrientation::kVertical)
            .captured()
            || self
                .GetScrollbarAnimationController(ScrollbarOrientation::kHorizontal)
                .captured()
    }
    // Source: cc/input/scrollbar_animation_controller.cc:380-383
    fn Show<C: ScrollbarAnimationControllerClient + ?Sized>(&mut self, client: &mut C) {
        self.CancelDelayedAnimation();
        self.ApplyOpacityToScrollbars(client, 1.);
    }
    // Source: cc/input/scrollbar_animation_controller.cc:385-407
    fn ApplyOpacityToScrollbars<C: ScrollbarAnimationControllerClient + ?Sized>(
        &mut self,
        client: &mut C,
        opacity: f32,
    ) {
        for scrollbar in self.Scrollbars(client) {
            debug_assert!(client.Scrollbar_is_overlay_scrollbar(&scrollbar));
            let effective = if client.Scrollbar_CanScrollOrientation(&scrollbar) {
                opacity
            } else {
                0.
            };
            client.Scrollbar_SetOverlayScrollbarLayerOpacityAnimated(
                &scrollbar,
                effective,
                self.animation_change_ == AnimationChange::kFadeOut,
            );
        }
        let previously_visible = self.opacity_ > 0.;
        let currently_visible = opacity > 0.;
        if self.opacity_ != opacity {
            client.SetNeedsRedrawForScrollbarAnimation();
        }
        self.opacity_ = opacity;
        if previously_visible != currently_visible {
            client.DidChangeScrollbarVisibility();
            self.visibility_changed_ = true;
        }
    }
    // Rust equivalent of CancelableOnceClosure's pending callback invalidation.
    fn CancelDelayedAnimation(&mut self) {
        if let Some(task) = self.delayed_scrollbar_animation_.take() {
            drop(task);
        }
    }
    /// Typed callback delivery preserves the source weak pointer + once closure.
    pub fn RunDelayedAnimationTask<C: ScrollbarAnimationControllerClient + ?Sized>(
        &mut self,
        client: &mut C,
        task: ScrollbarAnimationTask,
    ) {
        if !task.IsAlive()
            || !self
                .delayed_scrollbar_animation_
                .as_ref()
                .is_some_and(|pending| pending.matches(&task))
        {
            return;
        }
        self.delayed_scrollbar_animation_ = None;
        self.StartAnimation(client);
    }
}
// Source: cc/input/scrollbar_animation_controller.cc:89-89
// CancelableOnceClosure and WeakPtrFactory invalidate callbacks on destruction.
impl Drop for ScrollbarAnimationController {
    fn drop(&mut self) {
        // C++ destroys the last-declared WeakPtrFactory before other members.
        self.weak_factory_.take();
        self.CancelDelayedAnimation();
    }
}
