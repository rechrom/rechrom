//! Synchronous source delegate query bridge. No Scheduler/Host pointer is held.
//! Source: cc/trees/layer_tree_host_impl.cc:6241-6243.
use super::native_frame_sink::NativeFrameSinkEndpoint;
use crate::adapter::layer_tree_host_impl::{
    HostImplContextProviderId, HostImplDelegateId, HostImplFrameSinkId,
};
use std::sync::{Arc, Mutex};
#[derive(Default)]
pub(crate) struct CallScopedSchedulerQueries {
    next: std::sync::atomic::AtomicU64,
    current: Mutex<Option<(u64, bool)>>,
}
impl CallScopedSchedulerQueries {
    pub(crate) fn Enter(self: &Arc<Self>, ready_to_activate: bool) -> SchedulerQueryScope {
        let epoch = self.next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut current = self.current.lock().expect("source scheduler call scope");
        assert!(current.is_none(), "overlapping source Host query scope");
        *current = Some((epoch, ready_to_activate));
        SchedulerQueryScope {
            queries: self.clone(),
            epoch,
        }
    }
    pub(crate) fn IsReadyToActivate(&self) -> bool {
        self.current
            .lock()
            .expect("source scheduler call scope")
            .as_ref()
            .expect("Scheduler readiness is only valid during its current Host call")
            .1
    }
}
pub(crate) struct SchedulerQueryScope {
    queries: Arc<CallScopedSchedulerQueries>,
    epoch: u64,
}
impl Drop for SchedulerQueryScope {
    fn drop(&mut self) {
        let mut current = self
            .queries
            .current
            .lock()
            .expect("source scheduler call scope");
        assert_eq!(current.as_ref().map(|value| value.0), Some(self.epoch));
        *current = None;
    }
}
pub(crate) struct NativeFactoryPictureState {
    sink: Arc<Mutex<NativeFrameSinkEndpoint>>,
    queries: Arc<CallScopedSchedulerQueries>,
    delegate: HostImplDelegateId,
}
impl NativeFactoryPictureState {
    pub(crate) fn new(
        sink: Arc<Mutex<NativeFrameSinkEndpoint>>,
        queries: Arc<CallScopedSchedulerQueries>,
        delegate: HostImplDelegateId,
    ) -> Self {
        Self {
            sink,
            queries,
            delegate,
        }
    }
}
impl super::native_host_picture::NativeHostPictureState for NativeFactoryPictureState {
    fn FrameSinkContextProvider(
        &self,
        sink: HostImplFrameSinkId,
    ) -> Option<HostImplContextProviderId> {
        assert_eq!(sink, self.sink.lock().expect("CPU frame sink").id());
        // This actual native CPU sink owns no GPU context. GPU endpoints use
        // their own native capability, rather than a fabricated context handle.
        None
    }
    fn IsReadyToActivate(&self, delegate: HostImplDelegateId) -> bool {
        assert_eq!(delegate, self.delegate);
        self.queries.IsReadyToActivate()
    }
}
