//! Platform-independent frame time and demand. Native frame production stays
//! with the embedder; Page consumes these values on its existing owner thread.
//! Corresponding roles: viz::BeginFrameArgs and BeginFrameSource's demand path.
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub struct BeginFrameArgs {
    pub source_id: u64,
    pub sequence_number: u64,
    pub frame_time: Instant,
    pub deadline: Instant,
    pub interval: Duration,
}

/// Request a forthcoming frame. Requests before that frame coalesce. A frame
/// callback never calls Page across threads; the embedder forwards its args.
pub trait BeginFrameSource: Send + Sync {
    fn request_begin_frame(&self);

    /// Rebind a native source when its window changes display. Timer and
    /// platform-independent sources have no display binding.
    fn set_display(&self, _display_id: u32) {}
}
