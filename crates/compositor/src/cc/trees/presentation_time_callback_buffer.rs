// Translation status: complete
// Copyright 2019 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/trees/presentation_time_callback_buffer.h:1-165;
//         cc/trees/presentation_time_callback_buffer.cc:1-12.
use crate::adapter::presentation::{
    Callback, SequenceChecker, SuccessfulCallback, SuccessfulCallbackWithDetails,
};
use std::collections::VecDeque;
// Source: cc/trees/presentation_time_callback_buffer.h:82-107;
//         cc/trees/presentation_time_callback_buffer.cc:80-86.
// Rust moves implement the defaulted move operations. Owning fields are listed
// in reverse C++ declaration order so Rust drops captured callback resources in
// the same order as the source defaulted destructor. Copy remains unavailable.
pub struct PendingCallbacks<F, D> {
    // Source: cc/trees/presentation_time_callback_buffer.h:103-105.
    pub compositor_successful_callbacks: Vec<SuccessfulCallback>,
    // Source: cc/trees/presentation_time_callback_buffer.h:99-101.
    pub main_successful_callbacks: Vec<SuccessfulCallbackWithDetails<D>>,
    // Source: cc/trees/presentation_time_callback_buffer.h:95-97.
    pub main_callbacks: Vec<Callback<F>>,
}
impl<F, D> Default for PendingCallbacks<F, D> {
    fn default() -> Self {
        Self {
            main_callbacks: Vec::new(),
            main_successful_callbacks: Vec::new(),
            compositor_successful_callbacks: Vec::new(),
        }
    }
}
// Source: cc/trees/presentation_time_callback_buffer.h:128-150;
//         cc/trees/presentation_time_callback_buffer.cc:30-38.
struct FrameTokenInfo<F, D> {
    // Source: cc/trees/presentation_time_callback_buffer.h:145-146.
    compositor_successful_callbacks: Vec<SuccessfulCallback>,
    // Source: cc/trees/presentation_time_callback_buffer.h:142-143.
    main_successful_callbacks: Vec<SuccessfulCallbackWithDetails<D>>,
    // Source: cc/trees/presentation_time_callback_buffer.h:139-140.
    main_callbacks: Vec<Callback<F>>,
    // Source: cc/trees/presentation_time_callback_buffer.h:135-137.
    token: u32,
}
impl<F, D> FrameTokenInfo<F, D> {
    fn new(token: u32) -> Self {
        Self {
            token,
            main_callbacks: Vec::new(),
            main_successful_callbacks: Vec::new(),
            compositor_successful_callbacks: Vec::new(),
        }
    }
}
// Source: cc/trees/presentation_time_callback_buffer.h:38-160.
pub struct PresentationTimeCallbackBuffer<F, D, Q: SequenceChecker> {
    frame_token_infos_: VecDeque<FrameTokenInfo<F, D>>,
    sequence_checker_: Q,
}
impl<F, D, Q: SequenceChecker> PresentationTimeCallbackBuffer<F, D, Q> {
    // Source: cc/trees/presentation_time_callback_buffer.h:43.
    pub const kMaxBufferSize: usize = 60;
    // Source: cc/trees/presentation_time_callback_buffer.cc:14,28.
    pub fn new(sequence_checker: Q) -> Self {
        Self {
            frame_token_infos_: VecDeque::new(),
            sequence_checker_: sequence_checker,
        }
    }
    // Source: cc/trees/presentation_time_callback_buffer.cc:16-18.
    // The source move constructor creates a fresh sequence checker.
    pub fn MoveFrom(other: &mut Self, sequence_checker: Q) -> Self {
        Self {
            frame_token_infos_: std::mem::take(&mut other.frame_token_infos_),
            sequence_checker_: sequence_checker,
        }
    }
    // Source: cc/trees/presentation_time_callback_buffer.cc:20-26.
    pub fn MoveAssign(&mut self, other: &mut Self) -> &mut Self {
        if !std::ptr::eq(self, other) {
            // Source dependency: base/containers/circular_deque.h:556-568.
            // Move assignment first destroys the destination registrations,
            // then consumes the source buffer, preserving callback cancellation
            // before the moved-from source becomes empty.
            self.frame_token_infos_.clear();
            self.frame_token_infos_ = std::mem::take(&mut other.frame_token_infos_);
        }
        self
    }
    // Source: cc/trees/presentation_time_callback_buffer.cc:40-51.
    pub fn RegisterMainThreadCallbacks(
        &mut self,
        frame_token: u32,
        mut callbacks: Vec<Callback<F>>,
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
    // Source: cc/trees/presentation_time_callback_buffer.cc:53-64.
    pub fn RegisterMainThreadSuccessfulCallbacks(
        &mut self,
        frame_token: u32,
        mut callbacks: Vec<SuccessfulCallbackWithDetails<D>>,
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
    // Source: cc/trees/presentation_time_callback_buffer.cc:66-79.
    pub fn RegisterCompositorThreadSuccessfulCallbacks(
        &mut self,
        frame_token: u32,
        mut callbacks: Vec<SuccessfulCallback>,
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
    // Source: cc/trees/presentation_time_callback_buffer.cc:88-126.
    pub fn PopPendingCallbacks(
        &mut self,
        frame_token: u32,
        presentation_failed: bool,
    ) -> PendingCallbacks<F, D> {
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
    // Source: cc/trees/presentation_time_callback_buffer.cc:128-142.
    fn GetOrMakeRegistration(&mut self, frame_token: u32) -> &mut FrameTokenInfo<F, D> {
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
