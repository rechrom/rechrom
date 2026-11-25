//! The presentation callbacks the main LayerTreeHost queues
//! (LayerTreeHost::RequestPresentationTimeForNextFrame /
//! RequestSuccessfulPresentationTimeForNextFrame, carried as
//! CommitState::pending_presentation_callbacks and
//! pending_successful_presentation_callbacks, cc/trees/commit_state.h) and the
//! impl PresentationTimeCallbackBuffer
//! (cc/trees/presentation_time_callback_buffer.h:26-44) runs on the main
//! thread after presentation.
pub use viz_common::frame_timing_details::{FrameTimingDetails, PresentationFeedback};

// `Send`: main-thread presentation callbacks are posted back from the impl
// thread (cc/trees/proxy_main.cc DidPresentCompositorFrame).
pub type Callback<F> = Box<dyn FnOnce(&F) + Send>;
pub type SuccessfulCallbackWithDetails<D> = Box<dyn FnOnce(&D) + Send>;

/// PresentationTimeCallbackBuffer::Callback (base::OnceCallback<void(const
/// gfx::PresentationFeedback&)>).
pub type PresentationCallback = Callback<PresentationFeedback>;
/// PresentationTimeCallbackBuffer::SuccessfulCallbackWithDetails
/// (base::OnceCallback<void(const viz::FrameTimingDetails&)>).
pub type SuccessfulPresentationCallback = SuccessfulCallbackWithDetails<FrameTimingDetails>;
