// Copyright 2022 The Chromium Authors. BSD license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/public/common/frame/history_user_activation_state.h
use crate::base::time::time::TimeTicks;

#[derive(Default)]
pub struct HistoryUserActivationState {
    user_activation_time_: TimeTicks,
    last_used_user_activation_time_: TimeTicks,
}
impl HistoryUserActivationState {
    pub fn Activate(&mut self, now: TimeTicks) {
        self.user_activation_time_ = now;
    }
    pub fn Consume(&mut self) {
        self.last_used_user_activation_time_ = self.user_activation_time_;
    }
    pub fn IsActive(&self) -> bool {
        self.last_used_user_activation_time_ != self.user_activation_time_
    }
    pub fn Clear(&mut self) {
        self.Consume();
    }
}
