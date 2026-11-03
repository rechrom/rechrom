//! External actions for the translated LayerTreeHostImpl A facet.
//! The facade owns no Host or tree; monitor lifetime is represented by the
//! canonical registry identity and event metrics stay in their actual manager.

use super::{
    engine_factory::FactoryCapabilities,
    host_owners::EventsMetricsManager,
    input::{
        EventMetricsDoneCallback, EventMetricsScopedMonitor, LatencyInfo,
        LatencyInfoSwapPromiseMonitor,
    },
};
use crate::{
    adapter::layer_tree_host_impl::{HostImplAServices, HostImplDelegateId, HostImplWeak},
    CompositorTypes,
};
use std::sync::Arc;

pub(crate) struct HostActionServices {
    capabilities: Arc<FactoryCapabilities>,
}

impl HostActionServices {
    pub(crate) fn new(capabilities: Arc<FactoryCapabilities>) -> Self {
        Self { capabilities }
    }
}

impl HostImplAServices<CompositorTypes> for HostActionServices {
    fn TraceHostImplAEvent(&self, name: &'static str) {
        self.capabilities.events.Emit("cc", name.into());
    }

    fn DelegateDidPrepareTiles(&mut self, delegate: HostImplDelegateId) {
        self.capabilities.lifecycle.DidPrepareTiles(delegate);
    }

    fn CreateLatencyInfoSwapPromiseMonitor(
        &mut self,
        latency: Option<&mut LatencyInfo>,
        owner: HostImplWeak,
    ) -> Box<dyn LatencyInfoSwapPromiseMonitor> {
        self.capabilities.latency_monitors.Create(
            latency,
            owner,
            self.capabilities.commands.clone(),
        )
    }

    fn GetScopedEventMetricsMonitor(
        &mut self,
        manager: &mut EventsMetricsManager,
        callback: EventMetricsDoneCallback,
    ) -> Box<dyn EventMetricsScopedMonitor> {
        manager.0.GetScopedMonitor(callback)
    }
}
