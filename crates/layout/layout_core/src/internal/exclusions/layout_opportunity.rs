//! Float layout opportunities: the data type is fragment data
//! (`layout_fragment`); the shape-aware line bounds read each shaped float's
//! LayoutBox and are this base extension trait.

#![allow(non_snake_case)]

use foundation::{LayoutUnit, TextDirection};

use crate::internal::constraint_space::ConstraintSpace;
use crate::internal::layout_box::LayoutBox;
use crate::internal::shapes::shape::LineSegment;

use super::exclusion_area::ExclusionArea;
use super::line_layout_opportunity::LineLayoutOpportunity;

pub use layout_fragment::internal::exclusions::layout_opportunity::*;

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
#[inline(never)]
fn excluded_segment(
    exclusion: &ExclusionArea,
    bfc_block_offset: LayoutUnit,
    line_block_size: LayoutUnit,
) -> LineSegment {
    let shape_data_ptr = exclusion.shape_data.Get();
    debug_assert!(!shape_data_ptr.is_null());
    let shape_data = unsafe { &*shape_data_ptr };
    let layout_box = unsafe { &*shape_data.layout_box.Get().cast::<LayoutBox>() };
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

// cpp: layoutng/internal/exclusions/layout_opportunity.h:61-63
// cpp: layoutng/internal/exclusions/layout_opportunity.cc:99-129
#[inline(never)]
fn compute_line_left_offset(
    opportunity: &LayoutOpportunity,
    space: &ConstraintSpace,
    line_block_size: LayoutUnit,
    block_delta: LayoutUnit,
) -> LayoutUnit {
    let shapes_ptr = opportunity.shape_exclusions.Get();
    if shapes_ptr.is_null() || unsafe { &*shapes_ptr }.line_left_shapes.is_empty() {
        return opportunity.rect.LineStartOffset();
    }

    let bfc_block_offset = opportunity.rect.BlockStartOffset() + block_delta;
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
    min_layout(line_left, opportunity.rect.LineEndOffset())
}

// cpp: layoutng/internal/exclusions/layout_opportunity.h:64-66
// cpp: layoutng/internal/exclusions/layout_opportunity.cc:131-163
#[inline(never)]
fn compute_line_right_offset(
    opportunity: &LayoutOpportunity,
    space: &ConstraintSpace,
    line_block_size: LayoutUnit,
    block_delta: LayoutUnit,
) -> LayoutUnit {
    let shapes_ptr = opportunity.shape_exclusions.Get();
    if shapes_ptr.is_null() || unsafe { &*shapes_ptr }.line_right_shapes.is_empty() {
        return opportunity.rect.LineEndOffset();
    }

    let bfc_block_offset = opportunity.rect.BlockStartOffset() + block_delta;
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
    max_layout(line_right, opportunity.rect.LineStartOffset())
}

/// LayoutOpportunity behaviour that reads a shaped float's LayoutBox.
pub trait LayoutOpportunityExt {
    fn ComputeLineLayoutOpportunity(
        &self,
        space: &ConstraintSpace,
        line_block_size: LayoutUnit,
        block_delta: LayoutUnit,
    ) -> LineLayoutOpportunity;
}

impl LayoutOpportunityExt for LayoutOpportunity {
    // cpp: layoutng/internal/exclusions/layout_opportunity.h:51-56
    // cpp: layoutng/internal/exclusions/layout_opportunity.cc:165-190
    #[inline]
    fn ComputeLineLayoutOpportunity(
        &self,
        space: &ConstraintSpace,
        line_block_size: LayoutUnit,
        block_delta: LayoutUnit,
    ) -> LineLayoutOpportunity {
        let mut line_left = compute_line_left_offset(self, space, line_block_size, block_delta);
        let mut line_right = compute_line_right_offset(self, space, line_block_size, block_delta);

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
