//! Source-aligned compositor-thread LatencyInfo swap-promise monitor.
//!
//! The registry owns only the external latency value and stable monitor ids.
//! Notification returns the real pinned SwapPromise to the current Host call;
//! it never retains a Host, tree, Page, or callback into compositor state.

use super::{
    execution::{EngineTask, EngineTaskQueue},
    host_owners::SwapPromise,
    input::{LatencyInfo, LatencyInfoSwapPromiseMonitor},
    skia::SkRegion,
};
use crate::{
    adapter::{
        layer_tree_host_impl::{HostImplLatencyMonitorId, HostImplWeak},
        time::TimeTicks,
    },
    cc::trees::swap_promise::{
        DidNotSwapAction, DidNotSwapReason, SwapPromise as SwapPromiseContract,
    },
    NativeCompositorTypes,
};
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

#[derive(Clone, Default)]
pub(crate) struct NativeLatencyMonitorRegistry {
    state: Arc<Mutex<BTreeMap<HostImplLatencyMonitorId, Option<LatencyInfo>>>>,
}
impl NativeLatencyMonitorRegistry {
    pub(crate) fn Create(
        &self,
        latency: Option<&mut LatencyInfo>,
        owner: HostImplWeak,
        commands: EngineTaskQueue,
    ) -> Box<dyn LatencyInfoSwapPromiseMonitor> {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        let raw = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        assert_ne!(raw, 0, "latency monitor identity exhausted");
        let id = HostImplLatencyMonitorId(raw);
        assert!(self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(id, latency.map(|value| value.clone()))
            .is_none());
        Box::new(NativeLatencyMonitor {
            id,
            owner,
            commands,
        })
    }

    pub(crate) fn Notify(
        &self,
        id: HostImplLatencyMonitorId,
    ) -> Option<Box<SwapPromise<SkRegion>>> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let latency = state
            .get_mut(&id)
            .unwrap_or_else(|| panic!("unknown live latency monitor {}", id.0))
            .as_mut()?;
        if !latency.AddRenderingScheduledImpl() {
            return None;
        }
        let promise = NativeLatencyInfoSwapPromise {
            latency: Some(latency.CloneForSwapPromise()),
        };
        Some(Box::new(SwapPromise(Box::new(promise))))
    }

    pub(crate) fn Remove(&self, id: HostImplLatencyMonitorId) {
        assert!(self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&id)
            .is_some());
    }
}

struct NativeLatencyMonitor {
    id: HostImplLatencyMonitorId,
    owner: HostImplWeak,
    commands: EngineTaskQueue,
}
impl crate::adapter::layer_tree_host_impl::HostLatencyInfoSwapPromiseMonitor
    for NativeLatencyMonitor
{
    fn identity(&self) -> HostImplLatencyMonitorId {
        self.id
    }
}
impl LatencyInfoSwapPromiseMonitor for NativeLatencyMonitor {}
impl Drop for NativeLatencyMonitor {
    fn drop(&mut self) {
        self.commands.Push(EngineTask::RemoveLatencyMonitor {
            owner: self.owner.clone(),
            id: self.id,
        });
    }
}

struct NativeLatencyInfoSwapPromise {
    latency: Option<LatencyInfo>,
}
impl SwapPromiseContract<super::metadata::CompositorFrameMetadata<NativeCompositorTypes>>
    for NativeLatencyInfoSwapPromise
{
    fn DidActivate(&mut self) {}

    fn WillSwap(
        &mut self,
        metadata: &mut super::metadata::CompositorFrameMetadata<NativeCompositorTypes>,
    ) {
        metadata.latency_info.push(
            self.latency
                .take()
                .expect("LatencyInfo WillSwap is single-shot"),
        );
    }

    fn DidSwap(&mut self) {}

    fn DidNotSwap(&mut self, _reason: DidNotSwapReason, _timestamp: TimeTicks) -> DidNotSwapAction {
        if let Some(latency) = self.latency.as_mut() {
            latency.Terminate();
        }
        DidNotSwapAction::BREAK_PROMISE
    }

    fn OnCommit(&mut self) {
        if let Some(latency) = self.latency.as_ref() {
            latency.TraceMainCommit();
        }
    }

    fn GetTraceId(&self) -> i64 {
        self.latency.as_ref().map_or(0, LatencyInfo::TraceId)
    }
}
