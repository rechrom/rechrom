// Translation status: complete
// Copyright 2015 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Source: cc/input/scroll_state.h:1-21,103-109;
//         cc/input/scroll_state.cc:1-18,33-34.
// Module imports replace includes/namespace/export macros. Clone is C++'s
// defaulted copy constructor, and Rust field destruction is the defaulted dtor.
use crate::adapter::input_geometry::Vector2dF;
use crate::adapter::input_types::{ElementId, MainThreadHitTestReasons, ScrollGranularity};
use crate::cc::input::scroll_state_data::ScrollStateData;

// Source: cc/input/scroll_state.h:14-20,103-105.
// ScrollState is based on the proposal for scroll customization in blink, found
// here: https://goo.gl/1ipTpP.
#[derive(Clone, Debug)]
pub struct ScrollState {
    data_: ScrollStateData,
}

impl ScrollState {
    // Source: cc/input/scroll_state.h:18; cc/input/scroll_state.cc:13.
    pub fn new(data: ScrollStateData) -> Self {
        Self { data_: data }
    }

    // Source: cc/input/scroll_state.h:22-23; cc/input/scroll_state.cc:19-25.
    // Reduce deltas by x, y.
    pub fn ConsumeDelta(&mut self, x: f64, y: f64) {
        self.data_.delta_x -= x;
        self.data_.delta_y -= y;
        if x != 0.0 || y != 0.0 {
            self.data_.delta_consumed_for_scroll_sequence = true;
        }
    }

    // Source: cc/input/scroll_state.h:24-25.
    // Positive when scrolling right.
    pub fn delta_x(&self) -> f64 {
        self.data_.delta_x
    }

    // Source: cc/input/scroll_state.h:26-27.
    // Positive when scrolling down.
    pub fn delta_y(&self) -> f64 {
        self.data_.delta_y
    }

    // Source: cc/input/scroll_state.h:28-29.
    // Positive when scrolling right.
    pub fn delta_x_unconstrained(&self) -> f64 {
        self.data_.delta_x_unconstrained
    }

    // Source: cc/input/scroll_state.h:30-31.
    // Positive when scrolling down.
    pub fn delta_y_unconstrained(&self) -> f64 {
        self.data_.delta_y_unconstrained
    }

    // Source: cc/input/scroll_state.h:32-33.
    // Positive when scrolling right.
    pub fn delta_x_hint(&self) -> f64 {
        self.data_.delta_x_hint
    }

    // Source: cc/input/scroll_state.h:34-35.
    // Positive when scrolling down.
    pub fn delta_y_hint(&self) -> f64 {
        self.data_.delta_y_hint
    }

    // Source: cc/input/scroll_state.h:36-38.
    // The location associated with this scroll update. For touch, this is the
    // position of the finger. For mouse, the location of the cursor.
    pub fn position_x(&self) -> i32 {
        self.data_.position_x
    }

    // Source: cc/input/scroll_state.h:39-40.
    pub fn position_y(&self) -> i32 {
        self.data_.position_y
    }

    // Source: cc/input/scroll_state.h:41.
    pub fn is_beginning(&self) -> bool {
        self.data_.is_beginning
    }
    // Source: cc/input/scroll_state.h:42-44.
    pub fn set_is_beginning(&mut self, is_beginning: bool) {
        self.data_.is_beginning = is_beginning;
    }
    // Source: cc/input/scroll_state.h:45.
    pub fn is_in_inertial_phase(&self) -> bool {
        self.data_.is_in_inertial_phase
    }
    // Source: cc/input/scroll_state.h:46-48.
    pub fn set_is_in_inertial_phase(&mut self, is_in_inertial_phase: bool) {
        self.data_.is_in_inertial_phase = is_in_inertial_phase;
    }
    // Source: cc/input/scroll_state.h:49.
    pub fn is_ending(&self) -> bool {
        self.data_.is_ending
    }
    // Source: cc/input/scroll_state.h:50-51.
    pub fn set_is_ending(&mut self, is_ending: bool) {
        self.data_.is_ending = is_ending;
    }

    // Source: cc/input/scroll_state.h:52-53.
    // True if the user interacts directly with the screen, e.g., via touch.
    pub fn is_direct_manipulation(&self) -> bool {
        self.data_.is_direct_manipulation
    }
    // Source: cc/input/scroll_state.h:54-57.
    pub fn set_is_direct_manipulation(&mut self, is_direct_manipulation: bool) {
        self.data_.is_direct_manipulation = is_direct_manipulation;
    }

    // Source: cc/input/scroll_state.h:58-61.
    // True if the user interacts with the scrollbar.
    pub fn is_scrollbar_interaction(&self) -> bool {
        self.data_.is_scrollbar_interaction
    }
    // Source: cc/input/scroll_state.h:62-65.
    pub fn set_is_scrollbar_interaction(&mut self, is_scrollbar_interaction: bool) {
        self.data_.is_scrollbar_interaction = is_scrollbar_interaction;
    }

    // Source: cc/input/scroll_state.h:66-68.
    pub fn delta_consumed_for_scroll_sequence(&self) -> bool {
        self.data_.delta_consumed_for_scroll_sequence
    }
    // Source: cc/input/scroll_state.h:69-72.
    pub fn set_delta_consumed_for_scroll_sequence(&mut self, delta_consumed: bool) {
        self.data_.delta_consumed_for_scroll_sequence = delta_consumed;
    }

    // Source: cc/input/scroll_state.h:73-77.
    pub fn set_caused_scroll(&mut self, x: bool, y: bool) {
        self.data_.caused_scroll_x |= x;
        self.data_.caused_scroll_y |= y;
    }
    // Source: cc/input/scroll_state.h:78.
    pub fn caused_scroll_x(&self) -> bool {
        self.data_.caused_scroll_x
    }
    // Source: cc/input/scroll_state.h:79-80.
    pub fn caused_scroll_y(&self) -> bool {
        self.data_.caused_scroll_y
    }

    // Source: cc/input/scroll_state.h:81-82.
    pub fn set_is_scroll_chain_cut(&mut self, cut: bool) {
        self.data_.is_scroll_chain_cut = cut;
    }
    // Source: cc/input/scroll_state.h:83-84.
    pub fn is_scroll_chain_cut(&self) -> bool {
        self.data_.is_scroll_chain_cut
    }

    // Source: cc/input/scroll_state.h:85-88.
    pub fn delta_granularity(&self) -> ScrollGranularity {
        self.data_.delta_granularity
    }

    // Source: cc/input/scroll_state.h:89-92; cc/input/scroll_state.cc:26-32.
    // Returns the delta hints if this is a scroll begin or the real delta if
    // it's a scroll update.
    pub fn DeltaOrHint(&self) -> Vector2dF {
        if self.is_beginning() {
            return Vector2dF::new(self.delta_x_hint() as f32, self.delta_y_hint() as f32);
        }
        Vector2dF::new(self.delta_x() as f32, self.delta_y() as f32)
    }

    // Source: cc/input/scroll_state.h:93-96.
    pub fn target_element_id(&self) -> ElementId {
        self.data_.current_native_scrolling_element()
    }
    // Source: cc/input/scroll_state.h:97-100.
    pub fn main_thread_hit_tested_reasons(&self) -> MainThreadHitTestReasons {
        self.data_.main_thread_hit_tested_reasons
    }
    // Source: cc/input/scroll_state.h:101-102.
    pub fn data(&mut self) -> &mut ScrollStateData {
        &mut self.data_
    }
}
