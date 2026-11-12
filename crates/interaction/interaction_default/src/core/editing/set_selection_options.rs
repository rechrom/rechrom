// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license in Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/editing/set_selection_options.h:14-56
// Rust struct-update syntax supplies the source Builder's mechanical setters.
use super::text_granularity::TextGranularity;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CursorAlignOnScroll {
    #[default]
    kIfNeeded,
    kAlways,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SetSelectionBy {
    #[default]
    kSystem = 0,
    kUser = 1,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct SetSelectionOptions {
    pub cursor_align_on_scroll: CursorAlignOnScroll,
    pub do_not_clear_strategy: bool,
    pub do_not_set_focus: bool,
    pub granularity: TextGranularity,
    pub set_selection_by: SetSelectionBy,
    pub should_clear_typing_style: bool,
    pub should_close_typing: bool,
    pub should_show_handle: bool,
    pub should_shrink_next_tap: bool,
    pub is_directional: bool,
    pub should_notify_selection_controller_of_unchanged_selection: bool,
}
