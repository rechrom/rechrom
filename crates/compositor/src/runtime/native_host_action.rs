//! External actions for the translated LayerTreeHostImpl A facet.
//! The facade owns no Host or tree; monitor lifetime is represented by the
//! canonical registry identity and event metrics stay in their actual manager.

use super::{
    engine_factory::NativeFactoryCapabilities,
    host_owners::EventsMetricsManager,
    input::{
        EventMetricsDoneCallback, EventMetricsScopedMonitor, LatencyInfo,
        LatencyInfoSwapPromiseMonitor,
    },
};
use crate::{
    adapter::layer_tree_host_impl::{HostImplAServices, HostImplDelegateId, HostImplWeak},
    NativeCompositorTypes,
};
use std::sync::Arc;

pub(crate) struct NativeHostActionServices {
    native: Arc<NativeFactoryCapabilities>,
}

impl NativeHostActionServices {
    pub(crate) fn new(native: Arc<NativeFactoryCapabilities>) -> Self {
        Self { native }
    }
}

impl HostImplAServices<NativeCompositorTypes> for NativeHostActionServices {
    fn TraceHostImplAEvent(&self, name: &'static str) {
        self.native.events.Emit("cc", name.into());
    }

    fn DelegateDidPrepareTiles(&mut self, delegate: HostImplDelegateId) {
        self.native.lifecycle.DidPrepareTiles(delegate);
    }

    fn CreateLatencyInfoSwapPromiseMonitor(
        &mut self,
        latency: Option<&mut LatencyInfo>,
        owner: HostImplWeak,
    ) -> Box<dyn LatencyInfoSwapPromiseMonitor> {
        self.native
            .latency_monitors
            .Create(latency, owner, self.native.commands.clone())
    }

    fn GetScopedEventMetricsMonitor(
        &mut self,
        manager: &mut EventsMetricsManager,
        callback: EventMetricsDoneCallback,
    ) -> Box<dyn EventMetricsScopedMonitor> {
        manager.0.GetScopedMonitor(callback)
    }
}
