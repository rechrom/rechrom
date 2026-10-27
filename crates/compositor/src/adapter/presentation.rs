//! Platform sequence affinity and callback argument types stay with the owner.
//! The queue never executes callbacks and does not create or schedule threads.
pub use compositor_frame::presentation_callback_buffer::SequenceChecker;
pub type Callback<F> = Box<dyn FnOnce(&F)>;
/// Source OnceCallback payload. The first-scroll binding carries only weak
/// identity and scalar event data, and executes in the current Host call scope.
pub enum SuccessfulCallback {
    External(Box<dyn FnOnce(super::time::TimeTicks)>),
    FirstScroll {
        target: super::layer_tree_host_impl::HostImplWeak,
        payload: super::layer_tree_host_impl::FirstScrollTrackingPayload,
    },
}
impl From<Box<dyn FnOnce(super::time::TimeTicks)>> for SuccessfulCallback {
    fn from(callback: Box<dyn FnOnce(super::time::TimeTicks)>) -> Self {
        Self::External(callback)
    }
}
pub type SuccessfulCallbackWithDetails<D> = Box<dyn FnOnce(&D)>;
