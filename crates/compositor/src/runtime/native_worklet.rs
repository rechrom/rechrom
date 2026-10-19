//! External animation/worklet signals. Native Host algorithms and the successful
//! first-scroll continuation remain in the translated owner, never in closures.
use super::{
    engine_factory::NativeFactoryCapabilities, execution::EngineTask,
    host_types::UnboundedClientIdentity,
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
    NativeCompositorTypes,
};
use std::sync::Arc;
pub(crate) struct NativeWorkletServices {
    native: Arc<NativeFactoryCapabilities>,
}
impl NativeWorkletServices {
    pub(crate) fn new(native: Arc<NativeFactoryCapabilities>) -> Self {
        Self { native }
    }
}
impl HostImplJServices<NativeCompositorTypes> for NativeWorkletServices {
    fn NotifyAnimationWorkletStateChange(
        &mut self,
        delegate: HostImplDelegateId,
        state: AnimationWorkletMutationState,
        list: ElementListType,
    ) {
        self.native
            .commands
            .Push(EngineTask::AnimationWorkletState {
                delegate,
                state,
                list,
            });
    }
    fn SetNeedsImplSideInvalidation(&mut self, delegate: HostImplDelegateId, first: bool) {
        self.native
            .lifecycle
            .SetNeedsImplSideInvalidation(delegate, first);
    }
    fn IsInSynchronousComposite(&self, _: HostImplDelegateId) -> bool {
        // Source: cc/trees/proxy_impl.cc:381-383. This native constructor does
        // not install a synchronous-renderer compositor or enter that source scope.
        false
    }
    fn IsReadyToActivate(&self, _: HostImplDelegateId) -> bool {
        self.native.scheduler_queries.IsReadyToActivate()
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
