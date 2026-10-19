//! External scheduler/delegate endpoint for the resident impl Host.
//!
//! Chromium correspondence:
//! - cc/trees/layer_tree_host_impl.cc delegate calls in PrepareTiles,
//!   NotifyReadyToActivate, NotifyReadyToDraw and draw scheduling
//! - cc/trees/layer_tree_host_impl.h LayerTreeHostImpl::Delegate contract
//!
//! This endpoint records only typed calls to the Page-local Engine scheduler.
//! It cannot recover a Host, tree, TileManager or Page. The Engine drains the
//! FIFO after the current canonical Host call has released all live borrows.

use super::input::BeginMainFrameReason;
use crate::{
    adapter::layer_tree_host_impl::{HostImplDelegateId, HostImplVideoFrameControllerId},
    cc::tiles::tile_priority::TreePriority,
    runtime::types::FrameSinkId,
};
use std::{
    collections::{BTreeSet, VecDeque},
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub(crate) enum HostLifecycleEffect {
    NotifyReadyToActivate {
        delegate: HostImplDelegateId,
    },
    NotifyReadyToDraw {
        delegate: HostImplDelegateId,
    },
    DidPrepareTiles {
        delegate: HostImplDelegateId,
    },
    SetNeedsPrepareTiles {
        delegate: HostImplDelegateId,
    },
    SetNeedsCommit {
        delegate: HostImplDelegateId,
        reason: BeginMainFrameReason,
    },
    RenewTreePriority {
        delegate: HostImplDelegateId,
    },
    SetHasActiveThreadedScroll {
        delegate: HostImplDelegateId,
        active: bool,
    },
    SetWaitingForScrollEvent {
        delegate: HostImplDelegateId,
        waiting: bool,
    },
    SetDeferBeginMainFrameFromImpl {
        delegate: HostImplDelegateId,
        defer: bool,
    },
    SetNeedsOneBeginImplFrame {
        delegate: HostImplDelegateId,
    },
    SetNeedsRedraw {
        delegate: HostImplDelegateId,
    },
    SetNeedsImplSideInvalidation {
        delegate: HostImplDelegateId,
        needs_first_draw_on_activation: bool,
    },
    DidReceiveCompositorFrameAck {
        delegate: HostImplDelegateId,
    },
    OnDrawForLayerTreeFrameSink {
        delegate: HostImplDelegateId,
        resourceless: bool,
        skip: bool,
    },
    FrameSinksToThrottleUpdated {
        delegate: HostImplDelegateId,
        ids: BTreeSet<FrameSinkId>,
    },
    VideoFrameControllerDidDrawFrame {
        controller: HostImplVideoFrameControllerId,
    },
    SetBeginFrameSource {
        delegate: HostImplDelegateId,
        source: Option<std::sync::Arc<dyn crate::adapter::scheduler_host::BeginFrameSource>>,
    },
    ImageDecodeRequestFinished {
        delegate: HostImplDelegateId,
        request_id: i32,
        speculative: bool,
        succeeded: bool,
    },
    ClearHistory {
        delegate: HostImplDelegateId,
    },
    FirstScrollDelay {
        delegate: HostImplDelegateId,
        frame: i32,
        delay: crate::adapter::time::TimeDelta,
        timestamp: crate::adapter::time::TimeTicks,
    },
    ReturnResource {
        delegate: HostImplDelegateId,
        resource: crate::runtime::resources::CpuReturnedResource,
    },
    NewLocalSurfaceIdExpectedWhilePaused {
        sink: crate::adapter::layer_tree_host_impl::HostImplFrameSinkId,
    },
}

#[derive(Clone, Default)]
pub(crate) struct HostLifecycleEffects(Arc<Mutex<VecDeque<HostLifecycleEffect>>>);
impl HostLifecycleEffects {
    fn Push(&self, effect: HostLifecycleEffect) {
        self.0
            .lock()
            .expect("impl lifecycle effect queue")
            .push_back(effect);
    }

    pub(crate) fn Pop(&self) -> Option<HostLifecycleEffect> {
        self.0
            .lock()
            .expect("impl lifecycle effect queue")
            .pop_front()
    }

    pub(crate) fn IsEmpty(&self) -> bool {
        self.0
            .lock()
            .expect("impl lifecycle effect queue")
            .is_empty()
    }
}

struct HostLifecycleState {
    inside_draw: bool,
    needs_prepare_tiles: bool,
    needs_commit: bool,
    needs_redraw: bool,
    needs_one_begin_impl_frame: bool,
    has_active_threaded_scroll: bool,
    waiting_for_scroll_event: bool,
    defer_begin_main_frame_from_impl: bool,
    tree_priority: TreePriority,
}

pub(crate) struct NativeHostLifecycle {
    effects: HostLifecycleEffects,
    state: Mutex<HostLifecycleState>,
}

impl NativeHostLifecycle {
    pub(crate) fn new(effects: HostLifecycleEffects) -> Self {
        Self {
            effects,
            state: Mutex::new(HostLifecycleState {
                inside_draw: false,
                needs_prepare_tiles: false,
                needs_commit: false,
                needs_redraw: false,
                needs_one_begin_impl_frame: false,
                has_active_threaded_scroll: false,
                waiting_for_scroll_event: false,
                defer_begin_main_frame_from_impl: false,
                tree_priority: TreePriority::SAME_PRIORITY_FOR_BOTH_TREES,
            }),
        }
    }

    pub(crate) fn effects(&self) -> HostLifecycleEffects {
        self.effects.clone()
    }

    pub(crate) fn NotifyReadyToActivate(&self, delegate: HostImplDelegateId) {
        self.effects
            .Push(HostLifecycleEffect::NotifyReadyToActivate { delegate });
    }

    pub(crate) fn NotifyReadyToDraw(&self, delegate: HostImplDelegateId) {
        self.effects
            .Push(HostLifecycleEffect::NotifyReadyToDraw { delegate });
    }

    pub(crate) fn DidPrepareTiles(&self, delegate: HostImplDelegateId) {
        self.state
            .lock()
            .expect("impl lifecycle state")
            .needs_prepare_tiles = false;
        self.effects
            .Push(HostLifecycleEffect::DidPrepareTiles { delegate });
    }

    pub(crate) fn SetNeedsPrepareTiles(&self, delegate: HostImplDelegateId) {
        self.state
            .lock()
            .expect("impl lifecycle state")
            .needs_prepare_tiles = true;
        self.effects
            .Push(HostLifecycleEffect::SetNeedsPrepareTiles { delegate });
    }

    pub(crate) fn SetNeedsCommit(
        &self,
        delegate: HostImplDelegateId,
        reason: BeginMainFrameReason,
    ) {
        self.state
            .lock()
            .expect("impl lifecycle state")
            .needs_commit = true;
        self.effects
            .Push(HostLifecycleEffect::SetNeedsCommit { delegate, reason });
    }

    pub(crate) fn DidBeginMainFrame(&self) {
        self.state
            .lock()
            .expect("impl lifecycle state")
            .needs_commit = false;
    }

    pub(crate) fn RenewTreePriority(&self, delegate: HostImplDelegateId) {
        self.effects
            .Push(HostLifecycleEffect::RenewTreePriority { delegate });
    }

    pub(crate) fn SetTreePriority(&self, priority: TreePriority) {
        self.state
            .lock()
            .expect("impl lifecycle state")
            .tree_priority = priority;
    }

    pub(crate) fn tree_priority(&self) -> TreePriority {
        self.state
            .lock()
            .expect("impl lifecycle state")
            .tree_priority
    }

    pub(crate) fn SetHasActiveThreadedScroll(&self, delegate: HostImplDelegateId, active: bool) {
        self.state
            .lock()
            .expect("impl lifecycle state")
            .has_active_threaded_scroll = active;
        self.effects
            .Push(HostLifecycleEffect::SetHasActiveThreadedScroll { delegate, active });
    }

    pub(crate) fn SetWaitingForScrollEvent(&self, delegate: HostImplDelegateId, waiting: bool) {
        self.state
            .lock()
            .expect("impl lifecycle state")
            .waiting_for_scroll_event = waiting;
        self.effects
            .Push(HostLifecycleEffect::SetWaitingForScrollEvent { delegate, waiting });
    }

    pub(crate) fn SetDeferBeginMainFrameFromImpl(&self, delegate: HostImplDelegateId, defer: bool) {
        self.state
            .lock()
            .expect("impl lifecycle state")
            .defer_begin_main_frame_from_impl = defer;
        self.effects
            .Push(HostLifecycleEffect::SetDeferBeginMainFrameFromImpl { delegate, defer });
    }

    pub(crate) fn SetNeedsOneBeginImplFrame(&self, delegate: HostImplDelegateId) {
        self.state
            .lock()
            .expect("impl lifecycle state")
            .needs_one_begin_impl_frame = true;
        self.effects
            .Push(HostLifecycleEffect::SetNeedsOneBeginImplFrame { delegate });
    }

    pub(crate) fn DidBeginImplFrame(&self) {
        self.state
            .lock()
            .expect("impl lifecycle state")
            .needs_one_begin_impl_frame = false;
    }

    pub(crate) fn SetNeedsRedraw(&self, delegate: HostImplDelegateId) {
        self.state
            .lock()
            .expect("impl lifecycle state")
            .needs_redraw = true;
        self.effects
            .Push(HostLifecycleEffect::SetNeedsRedraw { delegate });
    }

    pub(crate) fn SetNeedsImplSideInvalidation(
        &self,
        delegate: HostImplDelegateId,
        needs_first_draw_on_activation: bool,
    ) {
        self.effects
            .Push(HostLifecycleEffect::SetNeedsImplSideInvalidation {
                delegate,
                needs_first_draw_on_activation,
            });
    }

    pub(crate) fn FrameSinksToThrottleUpdated(
        &self,
        delegate: HostImplDelegateId,
        ids: &BTreeSet<FrameSinkId>,
    ) {
        self.effects
            .Push(HostLifecycleEffect::FrameSinksToThrottleUpdated {
                delegate,
                ids: ids.clone(),
            });
    }

    pub(crate) fn VideoFrameControllerDidDrawFrame(
        &self,
        controller: HostImplVideoFrameControllerId,
    ) {
        self.effects
            .Push(HostLifecycleEffect::VideoFrameControllerDidDrawFrame { controller });
    }

    pub(crate) fn SetBeginFrameSource(
        &self,
        delegate: HostImplDelegateId,
        source: Option<&std::sync::Arc<dyn crate::adapter::scheduler_host::BeginFrameSource>>,
    ) {
        self.effects.Push(HostLifecycleEffect::SetBeginFrameSource {
            delegate,
            source: source.cloned(),
        });
    }

    pub(crate) fn ImageDecodeRequestFinished(
        &self,
        delegate: HostImplDelegateId,
        request_id: i32,
        speculative: bool,
        succeeded: bool,
    ) {
        self.effects
            .Push(HostLifecycleEffect::ImageDecodeRequestFinished {
                delegate,
                request_id,
                speculative,
                succeeded,
            });
    }

    pub(crate) fn ClearHistory(&self, delegate: HostImplDelegateId) {
        self.effects
            .Push(HostLifecycleEffect::ClearHistory { delegate });
    }

    pub(crate) fn FirstScrollDelay(
        &self,
        delegate: HostImplDelegateId,
        frame: i32,
        delay: crate::adapter::time::TimeDelta,
        timestamp: crate::adapter::time::TimeTicks,
    ) {
        self.effects.Push(HostLifecycleEffect::FirstScrollDelay {
            delegate,
            frame,
            delay,
            timestamp,
        });
    }

    pub(crate) fn ReturnResource(
        &self,
        delegate: HostImplDelegateId,
        resource: crate::runtime::resources::CpuReturnedResource,
    ) {
        self.effects
            .Push(HostLifecycleEffect::ReturnResource { delegate, resource });
    }

    pub(crate) fn NewLocalSurfaceIdExpectedWhilePaused(
        &self,
        sink: crate::adapter::layer_tree_host_impl::HostImplFrameSinkId,
    ) {
        self.effects
            .Push(HostLifecycleEffect::NewLocalSurfaceIdExpectedWhilePaused { sink });
    }

    pub(crate) fn DidDraw(&self) {
        self.state
            .lock()
            .expect("impl lifecycle state")
            .needs_redraw = false;
    }

    pub(crate) fn EnterDraw(&self) -> NativeHostDrawScope<'_> {
        let mut state = self.state.lock().expect("impl lifecycle state");
        assert!(!state.inside_draw, "nested HostImpl draw");
        state.inside_draw = true;
        NativeHostDrawScope(self)
    }

    pub(crate) fn IsInsideDraw(&self) -> bool {
        self.state.lock().expect("impl lifecycle state").inside_draw
    }

    pub(crate) fn DidReceiveCompositorFrameAck(&self, delegate: HostImplDelegateId) {
        self.effects
            .Push(HostLifecycleEffect::DidReceiveCompositorFrameAck { delegate });
    }

    pub(crate) fn OnDrawForLayerTreeFrameSink(
        &self,
        delegate: HostImplDelegateId,
        resourceless: bool,
        skip: bool,
    ) {
        self.effects
            .Push(HostLifecycleEffect::OnDrawForLayerTreeFrameSink {
                delegate,
                resourceless,
                skip,
            });
    }
}

pub(crate) struct NativeHostDrawScope<'a>(&'a NativeHostLifecycle);
impl Drop for NativeHostDrawScope<'_> {
    fn drop(&mut self) {
        let mut state = self.0.state.lock().expect("impl lifecycle state");
        assert!(state.inside_draw, "HostImpl draw scope state");
        state.inside_draw = false;
    }
}
