// Translation status: complete
// Copyright 2025 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/layers/append_quads_context.h:1-44.
// Source: cc/layers/append_quads_context.cc:1-23.
use super::draw_mode::DrawMode;
use crate::adapter::draw_property::ViewTransitionToken;
use std::collections::BTreeSet;
// Source: cc/layers/append_quads_context.h:15-40.
// Source: cc/layers/append_quads_context.cc:11,21.
// Clone implements the source implicit value copy, including the token set.
// Rust's default drop destroys that owned set; no owner/tree is retained.
#[derive(Clone, Debug, Default)]
pub struct AppendQuadsContext {
    pub draw_mode: DrawMode,
    pub capture_view_transition_tokens: BTreeSet<ViewTransitionToken>,
    pub for_view_transition_capture: bool,
}
impl AppendQuadsContext {
    // Source: cc/layers/append_quads_context.cc:13-19.
    pub fn new(
        draw_mode: DrawMode,
        capture_view_transition_tokens: BTreeSet<ViewTransitionToken>,
        for_view_transition_capture: bool,
    ) -> Self {
        Self {
            draw_mode,
            capture_view_transition_tokens,
            for_view_transition_capture,
        }
    }
}
