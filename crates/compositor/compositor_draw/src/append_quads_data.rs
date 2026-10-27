// Translation status: complete
// Copyright 2012 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/layers/append_quads_data.h:1-50.
// Source: cc/layers/append_quads_data.cc:1-13.
// Viz SurfaceId is external typed output; this value owns no layer/tree state.
// Source: cc/layers/append_quads_data.h:19-47.
// Source: cc/layers/append_quads_data.cc:9,11.
#[derive(Clone, Debug)]
pub struct AppendQuadsData<SurfaceId> {
    pub num_missing_tiles: i32,
    pub visible_layer_area: i64,
    pub approximated_visible_content_area: i64,
    pub deadline_in_frames: Option<u32>,
    pub use_default_lower_bound_deadline: bool,
    pub activation_dependencies: Vec<SurfaceId>,
    pub has_shared_element_resources: bool,
}
impl<SurfaceId> Default for AppendQuadsData<SurfaceId> {
    // Source: cc/layers/append_quads_data.h:24-45; .cc:9.
    fn default() -> Self {
        Self {
            num_missing_tiles: 0,
            visible_layer_area: 0,
            approximated_visible_content_area: 0,
            deadline_in_frames: None,
            use_default_lower_bound_deadline: false,
            activation_dependencies: Vec::new(),
            has_shared_element_resources: false,
        }
    }
}
