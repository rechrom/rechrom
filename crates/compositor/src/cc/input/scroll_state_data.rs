// Translation status: complete
// Copyright 2015 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Source: cc/input/scroll_state_data.h:1-18,83,91-95;
//         cc/input/scroll_state_data.cc:1-9,25.
// Header guards/export macros and namespace cc become Rust module scope. Source
// includes are dependency imports; the unused scroll_node.h include needs no
// Rust import. Rust Clone/clone_from implements the defaulted C++ copy members.
use crate::adapter::input_types::{ElementId, MainThreadHitTestReasons, ScrollGranularity};

// Source: cc/input/scroll_state_data.h:17-21;
//         cc/input/scroll_state_data.cc:12-14.
#[derive(Clone, Debug)]
pub struct ScrollStateData {
    // Source: cc/input/scroll_state_data.h:22-25.
    // Scroll delta in viewport coordinates (DIP).
    pub delta_x: f64,
    pub delta_y: f64,

    // Source: cc/input/scroll_state_data.h:26-30.
    // The raw, unconstrained scroll deltas before any axis locking (railing)
    // or snapping constraints are applied by the browser. Used when
    // scroll-axis-lock: none is active to allow diagonal scrolling.
    pub delta_x_unconstrained: f64,
    pub delta_y_unconstrained: f64,

    // Source: cc/input/scroll_state_data.h:31-35.
    // Scroll delta hint in viewport coordinates (DIP).
    // Delta hints are equal to deltas of the first gesture scroll update event in
    // a scroll sequence and are used for hittesting.
    pub delta_x_hint: f64,
    pub delta_y_hint: f64,

    // Source: cc/input/scroll_state_data.h:36-38.
    // Pointer (i.e. cursor/touch point) position in viewport coordinates (DIP).
    pub position_x: i32,
    pub position_y: i32,

    // Source: cc/input/scroll_state_data.h:39-44.
    pub is_beginning: bool,
    pub is_in_inertial_phase: bool,
    pub is_ending: bool,
    pub from_user_input: bool,

    // Source: cc/input/scroll_state_data.h:45-48.
    // Whether the scroll sequence has had any delta consumed, in the
    // current frame, or any child frames.
    pub delta_consumed_for_scroll_sequence: bool,

    // Source: cc/input/scroll_state_data.h:49-51.
    // True if the user interacts directly with the display, e.g., via touch.
    pub is_direct_manipulation: bool,

    // Source: cc/input/scroll_state_data.h:52-54.
    // True if the scroll is the result of a scrollbar interaction.
    pub is_scrollbar_interaction: bool,

    // Source: cc/input/scroll_state_data.h:55-58.
    // Granularity units for the scroll delta.
    pub delta_granularity: ScrollGranularity,

    // Source: cc/input/scroll_state_data.h:59-66.
    // TODO(tdresser): ScrollState shouldn't need to keep track of whether or not
    // this ScrollState object has caused a scroll. Ideally, any native scroller
    // consuming delta has caused a scroll. Currently, there are some cases where
    // we consume delta without scrolling, such as in |Viewport::AdjustOverscroll|.
    // Once these cases are fixed, remove |caused_scroll_*_|. crbug.com/510045.
    pub caused_scroll_x: bool,
    pub caused_scroll_y: bool,

    // Source: cc/input/scroll_state_data.h:67-73.
    // Track if the scroll_chain has been cut by overscroll_behavior, in
    // order to properly handle overscroll-effects.
    // TODO(sunyunjia): overscroll should be handled at the top of scroll_chain,
    // as implemented at blink side. Remove this once resolved. crbug.com/755164.
    pub is_scroll_chain_cut: bool,

    // Source: cc/input/scroll_state_data.h:77-82.
    // Used in scroll unification to specify that a scroll state has been hit
    // tested on the main thread. If this is non-empty, the hit test result will
    // be placed in the current_native_scrolling_element_.
    pub main_thread_hit_tested_reasons: MainThreadHitTestReasons,

    // Source: cc/input/scroll_state_data.h:84-90.
    // The id of the last native element to respond to a scroll, or 0 if none
    // exists.
    // TODO(bokan): In the compositor, this is now only used as an override to
    // scroller targeting. I.e. latch scrolling to the specified element_id.
    // Rename to a better name (target_element_id?) when main thread is converted.
    current_native_scrolling_element_: ElementId,
}

// Source: cc/input/scroll_state_data.h:19,23-73,81,90;
//         cc/input/scroll_state_data.cc:10.
// C++ in-class initializers execute in the defaulted constructor. ElementId and
// the EnumSet dependency are default constructed rather than assigned raw zero.
impl Default for ScrollStateData {
    fn default() -> Self {
        Self {
            delta_x: 0.0,
            delta_y: 0.0,
            delta_x_unconstrained: 0.0,
            delta_y_unconstrained: 0.0,
            delta_x_hint: 0.0,
            delta_y_hint: 0.0,
            position_x: 0,
            position_y: 0,
            is_beginning: false,
            is_in_inertial_phase: false,
            is_ending: false,
            from_user_input: false,
            delta_consumed_for_scroll_sequence: false,
            is_direct_manipulation: false,
            is_scrollbar_interaction: false,
            delta_granularity: ScrollGranularity::kScrollByPrecisePixel,
            caused_scroll_x: false,
            caused_scroll_y: false,
            is_scroll_chain_cut: false,
            main_thread_hit_tested_reasons: MainThreadHitTestReasons::default(),
            current_native_scrolling_element_: ElementId::default(),
        }
    }
}

impl ScrollStateData {
    // Source: cc/input/scroll_state_data.h:75;
    //         cc/input/scroll_state_data.cc:15-18.
    pub fn current_native_scrolling_element(&self) -> ElementId {
        self.current_native_scrolling_element_
    }

    // Source: cc/input/scroll_state_data.h:74,76;
    //         cc/input/scroll_state_data.cc:19-24.
    pub fn set_current_native_scrolling_element(&mut self, element_id: ElementId) {
        self.current_native_scrolling_element_ = element_id;
    }
}
