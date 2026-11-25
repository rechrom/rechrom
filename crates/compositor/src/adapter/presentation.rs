//! Platform sequence affinity and callback argument types stay with the owner.
//! The queue never executes callbacks and does not create or schedule threads.
pub use compositor_frame::presentation_callback_buffer::SequenceChecker;
// The callback types are shared with the main LayerTreeHost
// (cc/trees/presentation_time_callback_buffer.h).
pub use layerize::commit::presentation_time_callback::{Callback, SuccessfulCallbackWithDetails};
/// Source OnceCallback payload. The first-scroll binding carries only weak
/// identity and scalar event data, and executes in the current Host call scope.
pub enum SuccessfulCallback {
    External(Box<dyn FnOnce(super::time::TimeTicks) + Send>),
    FirstScroll {
        target: super::layer_tree_host_impl::HostImplWeak,
        payload: super::layer_tree_host_impl::FirstScrollTrackingPayload,
    },
}
impl From<Box<dyn FnOnce(super::time::TimeTicks) + Send>> for SuccessfulCallback {
    fn from(callback: Box<dyn FnOnce(super::time::TimeTicks) + Send>) -> Self {
        Self::External(callback)
    }
}
