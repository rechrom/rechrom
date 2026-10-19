// Translation status: complete
// Copyright 2013 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/trees/sticky_position_constraint.h:1-99
// Source: cc/trees/sticky_position_constraint.cc:1-231
use crate::adapter::{
    geometry::{cpp_max, cpp_min, PointF, RectF, Vector2dF},
    input_types::ElementId,
    snap_geometry::RangeF,
};

// Source: cc/trees/sticky_position_constraint.h:18-68
// Source: cc/trees/sticky_position_constraint.cc:97-102,228-229
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StickyPositionConstraint {
    // Source: cc/trees/sticky_position_constraint.h:23-26
    pub is_anchored_left: bool,
    pub is_anchored_right: bool,
    pub is_anchored_top: bool,
    pub is_anchored_bottom: bool,
    // Source: cc/trees/sticky_position_constraint.h:28-33
    pub left_offset: f32,
    pub right_offset: f32,
    pub top_offset: f32,
    pub bottom_offset: f32,
    // Source: cc/trees/sticky_position_constraint.h:35-37
    pub constraint_box_rect: RectF,
    // Source: cc/trees/sticky_position_constraint.h:39-43
    pub scroll_container_relative_sticky_box_rect: RectF,
    // Source: cc/trees/sticky_position_constraint.h:45-48
    pub scroll_container_relative_containing_block_rect: RectF,
    // Source: cc/trees/sticky_position_constraint.h:50-53
    pub pixel_snap_offset: Vector2dF,
    // Source: cc/trees/sticky_position_constraint.h:55-58
    pub x_scroll_ancestor_element_id: ElementId,
    pub y_scroll_ancestor_element_id: ElementId,
    // Source: cc/trees/sticky_position_constraint.h:60-68
    pub nearest_element_shifting_sticky_box: ElementId,
    pub nearest_element_shifting_containing_block: ElementId,
}
// Source: cc/trees/sticky_position_constraint.h:76-80
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanMergeResult {
    kCannotMerge,
    kCanAlwaysMerge,
    kCanMergeWithinScrollRange,
}

// Source: cc/trees/sticky_position_constraint.cc:16-38
#[derive(Clone, Copy, Debug, PartialEq)]
struct StickyTransitionPoints {
    left: RangeF,
    top: RangeF,
    right: RangeF,
    bottom: RangeF,
}
impl Default for StickyTransitionPoints {
    // Source: cc/trees/sticky_position_constraint.cc:23-28
    fn default() -> Self {
        Self {
            left: RangeF::InvalidRange(),
            top: RangeF::InvalidRange(),
            right: RangeF::InvalidRange(),
            bottom: RangeF::InvalidRange(),
        }
    }
}
impl StickyTransitionPoints {
    // Source: cc/trees/sticky_position_constraint.cc:30-37
    fn ClipByScrollRange(&mut self, range: RectF) {
        let range_x = RangeF::new(range.x(), range.right());
        self.left = self.left.Intersect(range_x);
        self.right = self.right.Intersect(range_x);
        let range_y = RangeF::new(range.y(), range.bottom());
        self.top = self.top.Intersect(range_y);
        self.bottom = self.bottom.Intersect(range_y);
    }
}
// Source: cc/trees/sticky_position_constraint.cc:40-93
fn GetStickyTransitionPoints(constraint: &StickyPositionConstraint) -> StickyTransitionPoints {
    debug_assert!(!constraint.nearest_element_shifting_sticky_box.IsValid());
    debug_assert!(!constraint
        .nearest_element_shifting_containing_block
        .IsValid());
    let sticky_box_rect = &constraint.scroll_container_relative_sticky_box_rect;
    let containing_block_rect = &constraint.scroll_container_relative_containing_block_rect;
    let mut points = StickyTransitionPoints::default();
    if constraint.is_anchored_left {
        let available_space = containing_block_rect.right() - sticky_box_rect.right();
        if available_space > 0.0 {
            points.left.set_start(
                sticky_box_rect.x() - constraint.constraint_box_rect.x() - constraint.left_offset,
            );
            points.left.set_end(points.left.start() + available_space);
        }
    }
    if constraint.is_anchored_right {
        let available_space = containing_block_rect.x() - sticky_box_rect.x();
        if available_space < 0.0 {
            points.right.set_end(
                sticky_box_rect.right() + constraint.right_offset
                    - constraint.constraint_box_rect.right(),
            );
            points.right.set_start(points.right.end() + available_space);
        }
    }
    if constraint.is_anchored_top {
        let available_space = containing_block_rect.bottom() - sticky_box_rect.bottom();
        if available_space > 0.0 {
            points.top.set_start(
                sticky_box_rect.y() - constraint.constraint_box_rect.y() - constraint.top_offset,
            );
            points.top.set_end(points.top.start() + available_space);
        }
    }
    if constraint.is_anchored_bottom {
        let available_space = containing_block_rect.y() - sticky_box_rect.y();
        if available_space < 0.0 {
            points.bottom.set_end(
                sticky_box_rect.bottom() + constraint.bottom_offset
                    - constraint.constraint_box_rect.bottom(),
            );
            points
                .bottom
                .set_start(points.bottom.end() + available_space);
        }
    }
    points
}
impl StickyPositionConstraint {
    // Source: cc/trees/sticky_position_constraint.h:69-83
    // Source: cc/trees/sticky_position_constraint.cc:104-146
    pub fn CanMerge(&self, other: &Self, scroll_range: Option<RectF>) -> CanMergeResult {
        if self == other {
            return CanMergeResult::kCanAlwaysMerge;
        }
        if self.x_scroll_ancestor_element_id != other.x_scroll_ancestor_element_id
            || self.y_scroll_ancestor_element_id != other.y_scroll_ancestor_element_id
        {
            return CanMergeResult::kCannotMerge;
        }
        if self.nearest_element_shifting_sticky_box.IsValid()
            || other.nearest_element_shifting_sticky_box.IsValid()
            || self.nearest_element_shifting_containing_block.IsValid()
            || other.nearest_element_shifting_containing_block.IsValid()
        {
            return CanMergeResult::kCannotMerge;
        }
        let mut points1 = GetStickyTransitionPoints(self);
        let mut points2 = GetStickyTransitionPoints(other);
        if points1 == points2 {
            return CanMergeResult::kCanAlwaysMerge;
        }
        if let Some(scroll_range) = scroll_range {
            points1.ClipByScrollRange(scroll_range);
            points2.ClipByScrollRange(scroll_range);
            if points1 == points2 {
                return CanMergeResult::kCanMergeWithinScrollRange;
            }
        }
        CanMergeResult::kCannotMerge
    }
    // Source: cc/trees/sticky_position_constraint.h:85-92
    // Source: cc/trees/sticky_position_constraint.cc:148-226
    pub fn StickyPositionOffset(
        &self,
        scroll_position: PointF,
        constraint_box_expansion: Vector2dF,
        ancestor_sticky_box_offset: Vector2dF,
        ancestor_containing_block_offset: Vector2dF,
    ) -> Vector2dF {
        let mut clip = self.constraint_box_rect;
        clip.Offset(scroll_position.x(), scroll_position.y());
        clip.set_width(clip.width() + constraint_box_expansion.x());
        clip.set_height(clip.height() + constraint_box_expansion.y());
        let sticky_box_rect = self.scroll_container_relative_sticky_box_rect
            + ancestor_sticky_box_offset
            + ancestor_containing_block_offset;
        let containing_block_rect =
            self.scroll_container_relative_containing_block_rect + ancestor_containing_block_offset;
        let mut sticky_offset = Vector2dF::default();
        if self.is_anchored_right {
            let right_limit = clip.right() - self.right_offset;
            let mut right_delta = cpp_min(0.0, right_limit - sticky_box_rect.right());
            let available_space = cpp_min(0.0, containing_block_rect.x() - sticky_box_rect.x());
            if right_delta < available_space {
                right_delta = available_space;
            }
            sticky_offset.set_x(sticky_offset.x() + right_delta);
        }
        if self.is_anchored_left {
            let left_limit = clip.x() + self.left_offset;
            let mut left_delta = cpp_max(0.0, left_limit - sticky_box_rect.x());
            let available_space =
                cpp_max(0.0, containing_block_rect.right() - sticky_box_rect.right());
            if left_delta > available_space {
                left_delta = available_space;
            }
            sticky_offset.set_x(sticky_offset.x() + left_delta);
        }
        if self.is_anchored_bottom {
            let bottom_limit = clip.bottom() - self.bottom_offset;
            let mut bottom_delta = cpp_min(0.0, bottom_limit - sticky_box_rect.bottom());
            let available_space = cpp_min(0.0, containing_block_rect.y() - sticky_box_rect.y());
            if bottom_delta < available_space {
                bottom_delta = available_space;
            }
            sticky_offset.set_y(sticky_offset.y() + bottom_delta);
        }
        if self.is_anchored_top {
            let top_limit = clip.y() + self.top_offset;
            let mut top_delta = cpp_max(0.0, top_limit - sticky_box_rect.y());
            let available_space = cpp_max(
                0.0,
                containing_block_rect.bottom() - sticky_box_rect.bottom(),
            );
            if top_delta > available_space {
                top_delta = available_space;
            }
            sticky_offset.set_y(sticky_offset.y() + top_delta);
        }
        sticky_offset
    }
}
