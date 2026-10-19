// Translation status: complete
// Copyright 2021 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/trees/throttle_decider.h:1-54; cc/trees/throttle_decider.cc:1-14.
use crate::adapter::{geometry::RectF, throttle::*};
use std::collections::{BTreeMap, BTreeSet};
// Source: cc/trees/throttle_decider.h:20-49; cc/trees/throttle_decider.cc:16-18.
pub struct ThrottleDecider<C: ThrottleTypes> {
    id_to_pass_map_: BTreeMap<C::PassId, C::PassHandle>,
    ids_: BTreeSet<C::FrameSinkId>,
    last_ids_: BTreeSet<C::FrameSinkId>,
}
impl<C: ThrottleTypes> Default for ThrottleDecider<C> {
    fn default() -> Self {
        Self {
            id_to_pass_map_: BTreeMap::new(),
            ids_: BTreeSet::new(),
            last_ids_: BTreeSet::new(),
        }
    }
}
impl<C: ThrottleTypes> ThrottleDecider<C> {
    // Source: cc/trees/throttle_decider.cc:20-24.
    pub fn Prepare(&mut self) {
        std::mem::swap(&mut self.last_ids_, &mut self.ids_);
        self.id_to_pass_map_.clear();
        self.ids_.clear();
    }
    // Source: cc/trees/throttle_decider.cc:26-88.
    pub fn ProcessRenderPass<D>(&mut self, context: &D, identity: C::PassHandle)
    where
        D: ThrottleContext<
            Transform = C::Transform,
            PassId = C::PassId,
            FrameSinkId = C::FrameSinkId,
            PassHandle = C::PassHandle,
            Pass = C::Pass,
        >,
    {
        let pass = context.pass(identity);
        let foreground = pass.filters_have_blur();
        let mut blur_bounds_list = Vec::new();
        for quad in pass.quads() {
            if let Some(rpdq) = quad.render_pass_quad() {
                let Some(&child_id) = self.id_to_pass_map_.get(&rpdq.render_pass_id()) else {
                    continue;
                };
                let child = context.pass(child_id);
                if child.backdrop_filters_have_blur() && rpdq.resource_id().0 == 0 {
                    let mut bounds = RectF::from(child.output_rect());
                    if let Some(filter_bounds) = child.backdrop_filter_bounds() {
                        bounds.Intersect(filter_bounds);
                    }
                    bounds = quad.quad_to_target_transform().MapRect(bounds);
                    if let Some(clip) = quad.clip_rect() {
                        bounds.Intersect(RectF::from(clip));
                    }
                    blur_bounds_list.push(bounds);
                }
            } else if let Some(surface) = quad.surface_quad() {
                let mut inside = false;
                if !foreground && !blur_bounds_list.is_empty() {
                    let mut rect = quad
                        .quad_to_target_transform()
                        .MapRect(RectF::from(quad.visible_rect()));
                    if let Some(clip) = quad.clip_rect() {
                        rect.Intersect(RectF::from(clip));
                    }
                    for bounds in &blur_bounds_list {
                        if bounds.Contains(rect) {
                            inside = true;
                            break;
                        }
                    }
                }
                let range = surface.surface_range();
                debug_assert!(range.IsValid());
                if foreground || inside {
                    self.ids_.insert(range.end_frame_sink_id());
                }
            }
        }
        self.id_to_pass_map_.entry(pass.id()).or_insert(identity);
    }
    // Source: cc/trees/throttle_decider.cc:90-96.
    pub fn ProcessLayerNotToDraw(&mut self, layer: &impl ThrottleLayer<C::FrameSinkId>) {
        if let Some(range) = layer.surface_range() {
            if range.IsValid() {
                self.ids_.insert(range.end_frame_sink_id());
            }
        }
    }
    // Source: cc/trees/throttle_decider.cc:98-100.
    pub fn HasThrottlingChanged(&self) -> bool {
        self.ids_ != self.last_ids_
    }
    // Source: cc/trees/throttle_decider.h:39.
    pub fn ids(&self) -> &BTreeSet<C::FrameSinkId> {
        &self.ids_
    }
}
