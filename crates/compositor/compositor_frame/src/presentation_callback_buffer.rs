//! Frame-token ordered presentation callback storage.
//!
//! Callback payloads remain opaque owner types. This package owns only the
//! ordering, retry, cancellation, and bounded-buffer behavior.

use std::collections::VecDeque;

pub trait SequenceChecker {
    fn Check(&self);
}

pub struct PendingCallbacks<MainCallback, MainSuccessful, CompositorSuccessful> {
    pub compositor_successful_callbacks: Vec<CompositorSuccessful>,
    pub main_successful_callbacks: Vec<MainSuccessful>,
    pub main_callbacks: Vec<MainCallback>,
}

impl<MainCallback, MainSuccessful, CompositorSuccessful> Default
    for PendingCallbacks<MainCallback, MainSuccessful, CompositorSuccessful>
{
    fn default() -> Self {
        Self {
            main_callbacks: Vec::new(),
            main_successful_callbacks: Vec::new(),
            compositor_successful_callbacks: Vec::new(),
        }
    }
}

struct FrameTokenInfo<MainCallback, MainSuccessful, CompositorSuccessful> {
    compositor_successful_callbacks: Vec<CompositorSuccessful>,
    main_successful_callbacks: Vec<MainSuccessful>,
    main_callbacks: Vec<MainCallback>,
    token: u32,
}

impl<MainCallback, MainSuccessful, CompositorSuccessful>
    FrameTokenInfo<MainCallback, MainSuccessful, CompositorSuccessful>
{
    fn new(token: u32) -> Self {
        Self {
            token,
            main_callbacks: Vec::new(),
            main_successful_callbacks: Vec::new(),
            compositor_successful_callbacks: Vec::new(),
        }
    }
}

pub struct PresentationTimeCallbackBuffer<
    MainCallback,
    MainSuccessful,
    CompositorSuccessful,
    Q: SequenceChecker,
> {
    frame_token_infos_:
        VecDeque<FrameTokenInfo<MainCallback, MainSuccessful, CompositorSuccessful>>,
    sequence_checker_: Q,
}

impl<MainCallback, MainSuccessful, CompositorSuccessful, Q: SequenceChecker>
    PresentationTimeCallbackBuffer<MainCallback, MainSuccessful, CompositorSuccessful, Q>
{
    pub const kMaxBufferSize: usize = 60;

    pub fn new(sequence_checker: Q) -> Self {
        Self {
            frame_token_infos_: VecDeque::new(),
            sequence_checker_: sequence_checker,
        }
    }

    pub fn MoveFrom(other: &mut Self, sequence_checker: Q) -> Self {
        Self {
            frame_token_infos_: std::mem::take(&mut other.frame_token_infos_),
            sequence_checker_: sequence_checker,
        }
    }

    pub fn MoveAssign(&mut self, other: &mut Self) -> &mut Self {
        if !std::ptr::eq(self, other) {
            self.frame_token_infos_.clear();
            self.frame_token_infos_ = std::mem::take(&mut other.frame_token_infos_);
        }
        self
    }

    pub fn RegisterMainThreadCallbacks(
        &mut self,
        frame_token: u32,
        mut callbacks: Vec<MainCallback>,
    ) {
        #[cfg(debug_assertions)]
        self.sequence_checker_.Check();
        if callbacks.is_empty() {
            return;
        }
        let sink = &mut self.GetOrMakeRegistration(frame_token).main_callbacks;
        sink.reserve(callbacks.len());
        sink.append(&mut callbacks);
    }

    pub fn RegisterMainThreadSuccessfulCallbacks(
        &mut self,
        frame_token: u32,
        mut callbacks: Vec<MainSuccessful>,
    ) {
        #[cfg(debug_assertions)]
        self.sequence_checker_.Check();
        if callbacks.is_empty() {
            return;
        }
        let sink = &mut self
            .GetOrMakeRegistration(frame_token)
            .main_successful_callbacks;
        sink.reserve(callbacks.len());
        sink.append(&mut callbacks);
    }

    pub fn RegisterCompositorThreadSuccessfulCallbacks(
        &mut self,
        frame_token: u32,
        mut callbacks: Vec<CompositorSuccessful>,
    ) {
        if callbacks.is_empty() {
            return;
        }
        let sink = &mut self
            .GetOrMakeRegistration(frame_token)
            .compositor_successful_callbacks;
        sink.reserve(callbacks.len());
        sink.append(&mut callbacks);
    }

    pub fn PopPendingCallbacks(
        &mut self,
        frame_token: u32,
        presentation_failed: bool,
    ) -> PendingCallbacks<MainCallback, MainSuccessful, CompositorSuccessful> {
        #[cfg(debug_assertions)]
        self.sequence_checker_.Check();
        let mut result = PendingCallbacks::default();
        let mut index = 0;
        while index < self.frame_token_infos_.len() {
            let info = &mut self.frame_token_infos_[index];
            if info.token > frame_token {
                break;
            }
            result.main_callbacks.append(&mut info.main_callbacks);
            info.main_callbacks.clear();
            let keep = presentation_failed
                && (!info.main_successful_callbacks.is_empty()
                    || !info.compositor_successful_callbacks.is_empty());
            if keep {
                index += 1;
            } else {
                result
                    .main_successful_callbacks
                    .append(&mut info.main_successful_callbacks);
                result
                    .compositor_successful_callbacks
                    .append(&mut info.compositor_successful_callbacks);
                self.frame_token_infos_.remove(index);
            }
        }
        result
    }

    fn GetOrMakeRegistration(
        &mut self,
        frame_token: u32,
    ) -> &mut FrameTokenInfo<MainCallback, MainSuccessful, CompositorSuccessful> {
        if self.frame_token_infos_.is_empty()
            || frame_token > self.frame_token_infos_.back().unwrap().token
        {
            self.frame_token_infos_
                .push_back(FrameTokenInfo::new(frame_token));
            debug_assert!(self.frame_token_infos_.len() <= Self::kMaxBufferSize);
        }
        debug_assert_eq!(self.frame_token_infos_.back().unwrap().token, frame_token);
        self.frame_token_infos_.back_mut().unwrap()
    }
}
