// Translation status: complete
// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
use compositor_tree::target_property::TargetProperties;
// Source: cc/trees/property_animation_state.h:1-39;
//         cc/trees/property_animation_state.cc:1-26.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PropertyAnimationState {
    pub currently_running: TargetProperties,
    pub potentially_animating: TargetProperties,
}
// Source: cc/trees/property_animation_state.cc:28-49.
impl std::ops::BitOrAssign for PropertyAnimationState {
    fn bitor_assign(&mut self, rhs: Self) {
        self.currently_running = self.currently_running | rhs.currently_running;
        self.potentially_animating = self.potentially_animating | rhs.potentially_animating;
    }
}
impl std::ops::BitXorAssign for PropertyAnimationState {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.currently_running = self.currently_running ^ rhs.currently_running;
        self.potentially_animating = self.potentially_animating ^ rhs.potentially_animating;
    }
}
impl std::ops::BitAndAssign for PropertyAnimationState {
    fn bitand_assign(&mut self, rhs: Self) {
        self.currently_running = self.currently_running & rhs.currently_running;
        self.potentially_animating = self.potentially_animating & rhs.potentially_animating;
    }
}
// Source: cc/trees/property_animation_state.cc:51-57.
impl std::ops::BitXor for PropertyAnimationState {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self {
        let mut result = self;
        result ^= rhs;
        result
    }
}
impl PropertyAnimationState {
    // Source: cc/trees/property_animation_state.cc:59-64.
    pub fn IsValid(&self) -> bool {
        (self.potentially_animating | !self.currently_running).all()
    }
    // Source: cc/trees/property_animation_state.cc:66-69.
    pub fn Clear(&mut self) {
        self.currently_running.reset();
        self.potentially_animating.reset();
    }
}
