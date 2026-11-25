//! The std::unique_ptr<cc::SwapPromise> the commit carries
//! (cc/trees/commit_state.h swap_promises; LayerTreeHost::QueueSwapPromise,
//! cc/trees/layer_tree_host.cc) and BeginMainFrameAbortedOnImpl returns
//! (cc/trees/proxy_impl.cc). The interface is
//! `compositor_values::swap_promise::SwapPromise` (cc/trees/swap_promise.h);
//! this owner holds the concrete promise the main thread created and
//! forwards each call to it, exactly as the C++ virtual call does.
pub use compositor_values::swap_promise::{DidNotSwapAction, DidNotSwapReason};
use compositor_values::swap_promise::SwapPromise as SwapPromiseInterface;
use scroll_animation::TimeTicks;

/// viz::CompositorFrameMetadata, the argument of SwapPromise::WillSwap.
pub type SwapPromiseMetadata = viz_common::CompositorFrameMetadata;

pub struct SwapPromise(pub Box<dyn SwapPromiseInterface<SwapPromiseMetadata>>);

impl compositor_tree::commit_types::SwapPromise for SwapPromise {}

impl SwapPromiseInterface<SwapPromiseMetadata> for SwapPromise {
    fn DidActivate(&mut self) {
        self.0.DidActivate()
    }
    fn WillSwap(&mut self, metadata: &mut SwapPromiseMetadata) {
        self.0.WillSwap(metadata)
    }
    fn DidSwap(&mut self) {
        self.0.DidSwap()
    }
    fn DidNotSwap(&mut self, reason: DidNotSwapReason, time: TimeTicks) -> DidNotSwapAction {
        self.0.DidNotSwap(reason, time)
    }
    fn OnCommit(&mut self) {
        self.0.OnCommit()
    }
    fn GetTraceId(&self) -> i64 {
        self.0.GetTraceId()
    }
}
