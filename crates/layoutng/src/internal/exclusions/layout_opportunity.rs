use std::fmt;

use foundation::{LayoutUnit, Member, TextDirection, Visitor};
use layoutng_geometry::geometry::bfc_offset::BfcOffset;
use layoutng_geometry::geometry::bfc_rect::BfcRect;

use crate::internal::constraint_space::ConstraintSpace;
use crate::internal::shapes::shape::LineSegment;

use super::exclusion_area::ExclusionArea;
use super::line_layout_opportunity::LineLayoutOpportunity;
use super::shape_exclusions::ShapeExclusions;

fn min_layout(a: LayoutUnit, b: LayoutUnit) -> LayoutUnit {
    if b < a {
        b
    } else {
        a
    }
}

fn max_layout(a: LayoutUnit, b: LayoutUnit) -> LayoutUnit {
    if a < b {
        b
    } else {
        a
    }
}

fn clamp_layout(value: LayoutUnit, low: LayoutUnit, high: LayoutUnit) -> LayoutUnit {
    if value < low {
        low
    } else if high < value {
        high
    } else {
        value
    }
}

// cpp: layoutng/internal/exclusions/layout_opportunity.cc:16-68
fn excluded_segment(
    exclusion: &ExclusionArea,
    bfc_block_offset: LayoutUnit,
    line_block_size: LayoutUnit,
) -> LineSegment {
    let shape_data_ptr = exclusion.shape_data.Get();
    debug_assert!(!shape_data_ptr.is_null());
    let shape_data = unsafe { &*shape_data_ptr };
    let layout_box = unsafe { &*shape_data.layout_box.Get() };
    let shape_info = unsafe { &*layout_box.GetShapeOutsideInfo() };
    let shape = shape_info.ComputedShape();

    let shape_relative_block_offset = bfc_block_offset
        - (exclusion.rect.BlockStartOffset()
            + shape_data.margins.block_start
            + shape_data.shape_insets.block_start);

    if !shape.LineOverlapsShapeMarginBounds(shape_relative_block_offset, line_block_size) {
        return LineSegment::default();
    }

    let clamped_line_block_size = min_layout(
        line_block_size,
        exclusion.rect.BlockSize()
            - shape_data.shape_insets.BlockSum()
            - shape_data.margins.BlockSum(),
    );

    let mut segment =
        shape.GetExcludedInterval(shape_relative_block_offset, clamped_line_block_size);
    let margin_delta = shape_data.margins.LineLeft(TextDirection::kLtr)
        + shape_data.shape_insets.LineLeft(TextDirection::kLtr);
    segment.logical_left += margin_delta;
    segment.logical_right += margin_delta;

    segment.logical_left = clamp_layout(
        segment.logical_left,
        LayoutUnit::default(),
        exclusion.rect.InlineSize(),
    );
    segment.logical_right = clamp_layout(
        segment.logical_right,
        LayoutUnit::default(),
        exclusion.rect.InlineSize(),
    );

    segment.logical_left += exclusion.rect.LineStartOffset();
    segment.logical_right += exclusion.rect.LineStartOffset();
    segment
}

// cpp: layoutng/internal/exclusions/layout_opportunity.cc:70-77
fn intersects_exclusion(
    exclusion: &ExclusionArea,
    bfc_block_offset: LayoutUnit,
    line_block_size: LayoutUnit,
) -> bool {
    bfc_block_offset < exclusion.rect.BlockEndOffset()
        && bfc_block_offset + line_block_size > exclusion.rect.BlockStartOffset()
}

// cpp: layoutng/internal/exclusions/layout_opportunity.h:17-67
pub struct LayoutOpportunity {
    pub rect: BfcRect,
    pub shape_exclusions: Member<ShapeExclusions>,
}

impl Default for LayoutOpportunity {
    // cpp: layoutng/internal/exclusions/layout_opportunity.h:26-28
    fn default() -> Self {
        Self {
            rect: BfcRect::new(
                BfcOffset::new(LayoutUnit::Min(), LayoutUnit::Min()),
                BfcOffset::new(LayoutUnit::Max(), LayoutUnit::Max()),
            ),
            shape_exclusions: Member::default(),
        }
    }
}

#[allow(non_snake_case)]
impl LayoutOpportunity {
    // cpp: layoutng/internal/exclusions/layout_opportunity.h:29-31
    pub fn new(rect: &BfcRect, shape_exclusions: *const ShapeExclusions) -> Self {
        Self {
            rect: *rect,
            shape_exclusions: Member::from_ptr(shape_exclusions as *mut ShapeExclusions),
        }
    }

    pub fn without_shapes(rect: &BfcRect) -> Self {
        Self::new(rect, std::ptr::null())
    }

    // cpp: layoutng/internal/exclusions/layout_opportunity.h:33-33
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.shape_exclusions);
    }

    // cpp: layoutng/internal/exclusions/layout_opportunity.h:43-45
    pub fn HasShapeExclusions(&self) -> bool {
        !self.shape_exclusions.Get().is_null()
    }

    // cpp: layoutng/internal/exclusions/layout_opportunity.h:47-49
    // cpp: layoutng/internal/exclusions/layout_opportunity.cc:81-97
    pub fn IsBlockDeltaBelowShapes(&self, block_delta: LayoutUnit) -> bool {
        let shapes_ptr = self.shape_exclusions.Get();
        debug_assert!(!shapes_ptr.is_null());
        let shapes = unsafe { &*shapes_ptr };
        for exclusion in &shapes.line_left_shapes {
            if self.rect.BlockStartOffset() + block_delta
                < unsafe { &*exclusion.Get() }.rect.BlockEndOffset()
            {
                return false;
            }
        }
        for exclusion in &shapes.line_right_shapes {
            if self.rect.BlockStartOffset() + block_delta
                < unsafe { &*exclusion.Get() }.rect.BlockEndOffset()
            {
                return false;
            }
        }
        true
    }

    // cpp: layoutng/internal/exclusions/layout_opportunity.h:61-63
    // cpp: layoutng/internal/exclusions/layout_opportunity.cc:99-129
    fn compute_line_left_offset(
        &self,
        space: &ConstraintSpace,
        line_block_size: LayoutUnit,
        block_delta: LayoutUnit,
    ) -> LayoutUnit {
        let shapes_ptr = self.shape_exclusions.Get();
        if shapes_ptr.is_null() || unsafe { &*shapes_ptr }.line_left_shapes.is_empty() {
            return self.rect.LineStartOffset();
        }

        let bfc_block_offset = self.rect.BlockStartOffset() + block_delta;
        let mut line_left = space.GetBfcOffset().line_offset;
        for exclusion in &unsafe { &*shapes_ptr }.line_left_shapes {
            let exclusion = unsafe { &*exclusion.Get() };
            if !intersects_exclusion(exclusion, bfc_block_offset, line_block_size) {
                continue;
            }
            if !exclusion.shape_data.Get().is_null() {
                let segment = excluded_segment(exclusion, bfc_block_offset, line_block_size);
                if segment.is_valid {
                    line_left = max_layout(line_left, segment.logical_right);
                }
            } else {
                line_left = max_layout(line_left, exclusion.rect.LineEndOffset());
            }
        }
        min_layout(line_left, self.rect.LineEndOffset())
    }

    // cpp: layoutng/internal/exclusions/layout_opportunity.h:64-66
    // cpp: layoutng/internal/exclusions/layout_opportunity.cc:131-163
    fn compute_line_right_offset(
        &self,
        space: &ConstraintSpace,
        line_block_size: LayoutUnit,
        block_delta: LayoutUnit,
    ) -> LayoutUnit {
        let shapes_ptr = self.shape_exclusions.Get();
        if shapes_ptr.is_null() || unsafe { &*shapes_ptr }.line_right_shapes.is_empty() {
            return self.rect.LineEndOffset();
        }

        let bfc_block_offset = self.rect.BlockStartOffset() + block_delta;
        let mut line_right = space.GetBfcOffset().line_offset + space.AvailableSize().inline_size;
        for exclusion in &unsafe { &*shapes_ptr }.line_right_shapes {
            let exclusion = unsafe { &*exclusion.Get() };
            if !intersects_exclusion(exclusion, bfc_block_offset, line_block_size) {
                continue;
            }
            if !exclusion.shape_data.Get().is_null() {
                let segment = excluded_segment(exclusion, bfc_block_offset, line_block_size);
                if segment.is_valid {
                    line_right = min_layout(line_right, segment.logical_left);
                }
            } else {
                line_right = min_layout(line_right, exclusion.rect.LineStartOffset());
            }
        }
        max_layout(line_right, self.rect.LineStartOffset())
    }

    // cpp: layoutng/internal/exclusions/layout_opportunity.h:51-56
    // cpp: layoutng/internal/exclusions/layout_opportunity.cc:165-190
    pub fn ComputeLineLayoutOpportunity(
        &self,
        space: &ConstraintSpace,
        line_block_size: LayoutUnit,
        block_delta: LayoutUnit,
    ) -> LineLayoutOpportunity {
        let mut line_left = self.compute_line_left_offset(space, line_block_size, block_delta);
        let mut line_right = self.compute_line_right_offset(space, line_block_size, block_delta);

        if space.Direction() == TextDirection::kLtr {
            let available_line_right =
                space.GetBfcOffset().line_offset + space.AvailableSize().inline_size;
            line_right = max_layout(min_layout(line_right, available_line_right), line_left);
        } else {
            let available_line_left = space.GetBfcOffset().line_offset;
            line_left = min_layout(max_layout(line_left, available_line_left), line_right);
        }

        LineLayoutOpportunity::new(
            line_left,
            line_right,
            self.rect.LineStartOffset(),
            self.rect.LineEndOffset(),
            self.rect.BlockStartOffset() + block_delta,
            line_block_size,
        )
    }
}

// cpp: layoutng/internal/exclusions/layout_opportunity.h:58-58
// cpp: layoutng/internal/exclusions/layout_opportunity_data.cc:7-9
impl PartialEq for LayoutOpportunity {
    fn eq(&self, other: &Self) -> bool {
        self.rect == other.rect && self.shape_exclusions.Get() == other.shape_exclusions.Get()
    }
}

// cpp: layoutng/internal/exclusions/layout_opportunity.h:69-70
// cpp: layoutng/internal/exclusions/layout_opportunity.cc:193-198
impl fmt::Display for LayoutOpportunity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.HasShapeExclusions() {
            f.write_str("ShapeExclusion@")
        } else {
            write!(f, "{}", self.rect)
        }
    }
}
