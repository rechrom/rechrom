//! Source: blink/platform/widget/input/input_handler_proxy.cc:61-120,
//! 883-925,1176-1410. The proxy owns sequence disposition; cc owns scrolling.
use super::*;
use crate::{
    adapter::{
        geometry::Point,
        input_handler::ScrollInputType,
        input_types::{ElementId, MainThreadHitTestReasons},
        time::TimeDelta,
    },
    cc::input::{
        input_handler::{ScrollBeginThreadState, ScrollThread, ScrollVector},
        scroll_state::ScrollState,
        scroll_state_data::ScrollStateData,
    },
    engine::ScrollPhase,
};

impl EngineAssembly {
    pub(super) fn GestureInput(
        &mut self,
        event: CompositorInput,
        hit_test: Option<(ElementId, MainThreadHitTestReasons)>,
    ) -> Result<CompositorOutput, CompositorError> {
        if self.waiting_wheel_hit_test.is_some() {
            self.deferred_wheel_inputs.push_back((event, true));
            // InputHandlerProxy retains the original callback until the
            // pending main-thread hit test and this deferred dispatch finish.
            return Ok(CompositorOutput {
                frame: None,
                frame_matches_latest_commit: false,
                main_thread_effects: std::mem::take(&mut self.core.effects),
                needs_begin_frames: self.needs_begin_frames,
                work_pending: false,
                main_commit_pending: self.core.HasReadyCommit()
                    || self.scroll_animation_commit.is_some(),
            });
        }
        let dispatch_id = match event {
            CompositorInput::ScrollGesture(gesture) => gesture.dispatch_id,
            CompositorInput::PinchGesture { dispatch_id, .. } => dispatch_id,
            _ => unreachable!(),
        };
        let query = self
            .core
            .scheduler_queries
            .Enter(self.scheduler.IsReadyToActivate());
        let host = self
            .core
            .host
            .as_mut()
            .expect("resident Host")
            .BaseMutable();
        let mut context = HostImplContext {
            services: &mut self.core.services,
        };
        host.WithInputHandler(&mut context, |handler, runtime| {
            let is_fling = matches!(event, CompositorInput::ScrollGesture(gesture) if gesture.is_inertial && gesture.phase != ScrollPhase::Begin);
            handler.expect("source bound InputHandler").NotifyInputEvent(runtime, is_fling);
        });
        let mut hit_reasons = None;
        let mut active = self.active_scroll_gesture;
        let mut pinch = self.gesture_pinch_in_progress;
        let boundary = matches!(event, CompositorInput::ScrollGesture(gesture) if gesture.phase != ScrollPhase::Update)
            || matches!(event, CompositorInput::PinchGesture { .. });
        // A phase boundary cannot overtake BeginImplFrame-aligned updates.
        if boundary {
            let pending = self.input_client.TakeQueued();
            host.WithInputHandler(&mut context, |handler, runtime| {
                let handler = handler.expect("source bound InputHandler");
                for update in pending {
                    handler.ScrollUpdateWithElasticOverscroll(
                        runtime,
                        ScrollState::new(update.data),
                        TimeDelta::default(),
                        update.event_timestamp,
                    );
                }
            });
        }
        let handled = match event {
            CompositorInput::ScrollGesture(mut gesture) => {
                if !gesture.position.x().is_finite()
                    || !gesture.position.y().is_finite()
                    || !gesture.delta.x().is_finite()
                    || !gesture.delta.y().is_finite()
                    || !gesture.unconstrained_delta.x().is_finite()
                    || !gesture.unconstrained_delta.y().is_finite()
                {
                    return Err(Self::WheelContinuationError("nonfinite gesture value"));
                }
                if let Some((id, reasons)) = hit_test {
                    gesture.scroll_element_id = id;
                    gesture.main_thread_hit_tested_reasons = reasons;
                }
                let mut data = ScrollStateData::default();
                data.is_direct_manipulation = gesture.input_type == ScrollInputType::kTouchscreen;
                data.is_scrollbar_interaction = gesture.input_type == ScrollInputType::kScrollbar;
                data.is_in_inertial_phase = gesture.is_inertial;
                data.delta_granularity = gesture.granularity;
                if gesture.phase == ScrollPhase::Begin {
                    data.position_x = gesture.position.x() as i32;
                    data.position_y = gesture.position.y() as i32;
                    data.delta_x_hint = -f64::from(gesture.delta.x());
                    data.delta_y_hint = -f64::from(gesture.delta.y());
                    data.is_beginning = true;
                    data.set_current_native_scrolling_element(gesture.scroll_element_id);
                    data.main_thread_hit_tested_reasons = gesture.main_thread_hit_tested_reasons;
                    self.input_client.BeginGestureForInput(gesture.input_type);
                    host.WithInputHandler(&mut context, |handler, runtime| {
                        let handler = handler.expect("source bound InputHandler");
                        if let Some(previous) = active.take() {
                            handler.RecordScrollEnd(runtime, previous);
                            handler.ScrollEnd(runtime, true, None);
                        } else if self.wheel_gesture_in_progress || self.scrollbar_gesture_active {
                            handler.RecordScrollEnd(
                                runtime,
                                if self.scrollbar_gesture_active {
                                    ScrollInputType::kScrollbar
                                } else {
                                    ScrollInputType::kWheel
                                },
                            );
                            handler.ScrollEnd(runtime, true, None);
                        }
                        let mut state = ScrollState::new(data);
                        let status = if gesture.target_viewport {
                            handler.RootScrollBegin(runtime, &mut state, gesture.input_type)
                        } else {
                            handler.ScrollBeginWithElasticOverscroll(
                                runtime,
                                &mut state,
                                gesture.input_type,
                                gesture.synthetic,
                            )
                        };
                        if !status.main_thread_hit_test_reasons.empty() {
                            assert!(
                                hit_test.is_none(),
                                "reinjected begin cannot request another hit test"
                            );
                            hit_reasons = Some(status.main_thread_hit_test_reasons);
                            return false;
                        }
                        if status.thread != ScrollThread::kScrollIgnored {
                            let blocked = !gesture.main_thread_hit_tested_reasons.empty();
                            let state = if status.main_thread_repaint_reasons.empty() {
                                if blocked {
                                    ScrollBeginThreadState::kScrollingOnCompositorBlockedOnMain
                                } else {
                                    ScrollBeginThreadState::kScrollingOnCompositor
                                }
                            } else if status.raster_inducing {
                                if blocked {
                                    ScrollBeginThreadState::kRasterInducingScrollBlockedOnMain
                                } else {
                                    ScrollBeginThreadState::kRasterInducingScroll
                                }
                            } else {
                                ScrollBeginThreadState::kScrollingOnMain
                            };
                            handler.RecordScrollBegin(runtime, gesture.input_type, state);
                        }
                        if status.thread == ScrollThread::kScrollOnImplThread {
                            active = Some(gesture.input_type);
                            !status.viewport_cannot_scroll
                        } else {
                            false
                        }
                    })
                } else if gesture.phase == ScrollPhase::Update
                    && (active == Some(gesture.input_type) || pinch)
                {
                    data.delta_x = -f64::from(gesture.delta.x());
                    data.delta_y = -f64::from(gesture.delta.y());
                    data.delta_x_unconstrained = -f64::from(gesture.unconstrained_delta.x());
                    data.delta_y_unconstrained = -f64::from(gesture.unconstrained_delta.y());
                    let route = self
                        .input_client
                        .RouteUpdate(data.clone(), gesture.event_timestamp);
                    if route != super::super::input_client::ScrollUpdateRoute::Dispatch {
                        if route == super::super::input_client::ScrollUpdateRoute::EnqueueAndRequestAnimation {
                            host.WithInputHandler(&mut context, |handler, runtime| handler.expect("source bound InputHandler").SetNeedsAnimateInput(runtime));
                        }
                        true
                    } else {
                        host.WithInputHandler(&mut context, |handler, runtime| {
                            let handler = handler.expect("source bound InputHandler");
                            let result = handler.ScrollUpdateWithElasticOverscroll(
                                runtime,
                                ScrollState::new(data),
                                TimeDelta::default(),
                                gesture.event_timestamp,
                            );
                            if result.hit_snap_constraint {
                                handler.ScrollEnd(runtime, true, None);
                                active = None;
                            }
                            result.did_scroll
                        })
                    }
                } else if gesture.phase == ScrollPhase::End && active == Some(gesture.input_type) {
                    host.WithInputHandler(&mut context, |handler, runtime| {
                        let handler = handler.expect("source bound InputHandler");
                        handler.RecordScrollEnd(runtime, gesture.input_type);
                        handler.ScrollEndWithElasticOverscroll(
                            runtime,
                            true,
                            Some(ScrollVector {
                                scroll_delta: crate::adapter::geometry::Vector2dF::new(
                                    -gesture.delta.x(),
                                    -gesture.delta.y(),
                                ),
                                granularity: gesture.granularity,
                            }),
                            gesture.event_timestamp,
                            gesture.synthetic,
                        );
                    });
                    active = None;
                    true
                } else {
                    false
                }
            }
            CompositorInput::PinchGesture {
                dispatch_id: _,
                anchor,
                scale,
                input_type,
                phase,
            } => {
                if !anchor.x().is_finite()
                    || !anchor.y().is_finite()
                    || !scale.is_finite()
                    || scale <= 0.
                {
                    return Err(Self::WheelContinuationError("invalid pinch value"));
                }
                let point = Point::new(anchor.x().floor() as i32, anchor.y().floor() as i32);
                host.WithInputHandler(&mut context, |handler, runtime| {
                    let handler = handler.expect("source bound InputHandler");
                    match phase {
                        ScrollPhase::Begin if !pinch => {
                            handler.PinchGestureBegin(runtime, point, input_type);
                            pinch = true;
                            true
                        }
                        ScrollPhase::Update if pinch => {
                            handler.PinchGestureUpdate(runtime, scale, point);
                            true
                        }
                        ScrollPhase::End if pinch => {
                            handler.PinchGestureEnd(runtime, point);
                            pinch = false;
                            true
                        }
                        _ => false,
                    }
                })
            }
            _ => unreachable!(),
        };
        self.active_scroll_gesture = active;
        self.gesture_pinch_in_progress = pinch;
        if matches!(event, CompositorInput::ScrollGesture(gesture) if gesture.phase == ScrollPhase::Begin)
        {
            self.wheel_gesture_in_progress = false;
            self.scrollbar_gesture_active = false;
        }
        drop(query);
        if let Some(reasons) = hit_reasons {
            let CompositorInput::ScrollGesture(gesture) = event else {
                unreachable!()
            };
            let request =
                self.RetainWheelInput(event, PendingWheelStage::MainThreadHitTest(reasons));
            self.waiting_wheel_hit_test = Some(request);
            self.core
                .effects
                .push(CompositorMainThreadEffect::WheelRequiresMainThreadHitTest {
                    request,
                    position: gesture.position,
                });
        } else {
            self.core
                .effects
                .push(CompositorMainThreadEffect::GestureDisposition {
                    dispatch_id,
                    handled,
                });
        }
        while let Some(effect) = self.lifecycle.Pop() {
            self.Lifecycle(effect)?;
        }
        if let Some(error) = self.core.failure.take() {
            return Err(error);
        }
        let mut effects = std::mem::take(&mut self.core.effects);
        self.core.DrainScrollEffects(&mut effects);
        Ok(CompositorOutput {
            frame: None,
            frame_matches_latest_commit: false,
            main_thread_effects: effects,
            needs_begin_frames: true,
            work_pending: true,
            main_commit_pending: self.core.HasReadyCommit()
                || self.scroll_animation_commit.is_some(),
        })
    }
}
