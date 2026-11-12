// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/public/common/frame/user_activation_state.h:16-25,106-164
// Source: third_party/blink/common/frame/user_activation_state.cc:17-83
// The embedding supplies TimeTicks in its monotonic clock domain. Expiry is
// evaluated when queried; no timer, OS clock or event-loop owner is retained.

use crate::base::time::time::TimeTicks;
pub use crate::public::mojom::frame::user_activation_notification_type::UserActivationNotificationType;

pub const kActivationLifespanMicroseconds: i64 = 5_000_000;

#[derive(Default)]
pub struct UserActivationState {
    has_been_active_: bool,
    transient_state_expiry_time_: TimeTicks,
    last_activation_was_restricted_: bool,
}

impl UserActivationState {
    // Source: user_activation_state.cc:17-26,31-36,77-79.
    pub fn Activate(&mut self, notification: UserActivationNotificationType, now: TimeTicks) {
        self.has_been_active_ = true;
        self.last_activation_was_restricted_ = matches!(
            notification,
            UserActivationNotificationType::kExtensionMessagingBothPrivileged
                | UserActivationNotificationType::kExtensionMessagingSenderPrivileged
                | UserActivationNotificationType::kExtensionMessagingReceiverPrivileged
                | UserActivationNotificationType::kExtensionMessagingNeitherPrivileged
        );
        self.transient_state_expiry_time_ = TimeTicks::FromInternalValue(
            now.ToInternalValue()
                .saturating_add(kActivationLifespanMicroseconds),
        );
    }

    // Source: user_activation_state.cc:38-40.
    pub fn SetHasBeenActive(&mut self) {
        self.has_been_active_ = true;
    }

    // Source: user_activation_state.cc:42-46.
    pub fn Clear(&mut self) {
        self.has_been_active_ = false;
        self.last_activation_was_restricted_ = false;
        self.DeactivateTransientState();
    }

    // Source: user_activation_state.cc:48-58.
    pub fn HasBeenActive(&self) -> bool {
        self.has_been_active_
    }
    pub fn IsActive(&self, now: TimeTicks) -> bool {
        now <= self.transient_state_expiry_time_
    }

    // Source: user_activation_state.cc:60-65.
    pub fn ConsumeIfActive(&mut self, now: TimeTicks) -> bool {
        if !self.IsActive(now) {
            return false;
        }
        self.DeactivateTransientState();
        true
    }

    // Source: user_activation_state.cc:67-69.
    pub fn LastActivationWasRestricted(&self) -> bool {
        self.last_activation_was_restricted_
    }

    // Source: user_activation_state.cc:81-83.
    fn DeactivateTransientState(&mut self) {
        self.transient_state_expiry_time_ = TimeTicks::new();
    }
}
