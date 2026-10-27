// Compatibility aliases for frame-owned presentation callback ordering.

use crate::adapter::presentation::{Callback, SuccessfulCallback, SuccessfulCallbackWithDetails};

pub type PendingCallbacks<F, D> = compositor_frame::presentation_callback_buffer::PendingCallbacks<
    Callback<F>,
    SuccessfulCallbackWithDetails<D>,
    SuccessfulCallback,
>;

pub type PresentationTimeCallbackBuffer<F, D, Q> =
    compositor_frame::presentation_callback_buffer::PresentationTimeCallbackBuffer<
        Callback<F>,
        SuccessfulCallbackWithDetails<D>,
        SuccessfulCallback,
        Q,
    >;
