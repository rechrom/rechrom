// Translation status: complete
// Copyright 2024 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/layers/scroll_hit_test_rect.h:1-23.

use crate::adapter::{geometry::Rect, input_types::ElementId};

// Source: cc/layers/scroll_hit_test_rect.h:10-19.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScrollHitTestRect {
    pub scroll_element_id: ElementId,
    pub hit_test_rect: Rect,
}

// Source: cc/layers/scroll_hit_test_rect.h:21-23.
// Namespace and include-guard closure map to the Rust module boundary.
