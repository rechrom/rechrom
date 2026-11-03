//! External animation/worklet signals. Native Host algorithms and the successful
//! first-scroll continuation remain in the translated owner, never in closures.
use super::{
    engine_factory::FactoryCapabilities, execution::EngineTask, host_types::UnboundedClientIdentity,
};
use crate::{
    adapter::{
        layer_tree_host_impl::{
            AnimationWorkletMutationState, FirstScrollTrackingPayload, HostImplDelegateId,
            HostImplJServices, HostImplWeak,
        },
        layer_tree_impl::{ElementListType, LayerTreeHostId},
        presentation::SuccessfulCallback,
    },
    CompositorTypes,
};
use std::sync::Arc;
pub(crate) struct WorkletServices {
    capabilities: Arc<FactoryCapabilities>,
}
impl WorkletServices {
    pub(crate) fn new(capabilities: Arc<FactoryCapabilities>) -> Self {
        Self { capabilities }
    }
}
impl HostImplJServices<CompositorTypes> for WorkletServices {
    fn NotifyAnimationWorkletStateChange(
        &mut self,
        delegate: HostImplDelegateId,
        state: AnimationWorkletMutationState,
        list: ElementListType,
    ) {
        self.capabilities
            .commands
            .Push(EngineTask::AnimationWorkletState {
                delegate,
                state,
                list,
            });
    }
    fn SetNeedsImplSideInvalidation(&mut self, delegate: HostImplDelegateId, first: bool) {
        self.capabilities
            .lifecycle
            .SetNeedsImplSideInvalidation(delegate, first);
    }
    fn IsInSynchronousComposite(&self, _: HostImplDelegateId) -> bool {
        // Source: cc/trees/proxy_impl.cc:381-383. This native constructor does
        // not install a synchronous-renderer compositor or enter that source scope.
        false
    }
    fn IsReadyToActivate(&self, _: HostImplDelegateId) -> bool {
        self.capabilities.scheduler_queries.IsReadyToActivate()
    }
    fn RouteFirstScrollCallback(
        &mut self,
        target: HostImplWeak,
        payload: FirstScrollTrackingPayload,
    ) -> SuccessfulCallback {
        SuccessfulCallback::FirstScroll { target, payload }
    }
    fn UnboundedClientIdentity(&mut self, owner: LayerTreeHostId) -> UnboundedClientIdentity {
        UnboundedClientIdentity(owner.0)
    }
}
