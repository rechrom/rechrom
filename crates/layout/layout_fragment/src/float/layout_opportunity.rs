//! Float-exclusion layout opportunities.
//!
//! The shape-aware line bounds
//! (`LayoutOpportunity::ComputeLineLayoutOpportunity`,
//! layout_opportunity.cc:16-190) read each shaped float's LayoutBox
//! ShapeOutsideInfo, so they are the base extension trait
//! `LayoutOpportunityExt` (`layout_core/src/internal/exclusions/layout_opportunity.rs`).

use std::fmt;

use foundation::{LayoutUnit, Member, Visitor};
use layout_geometry::geometry::bfc_offset::BfcOffset;
use layout_geometry::geometry::bfc_rect::BfcRect;

use super::super::shape_exclusions::ShapeExclusions;

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
    #[inline(never)]
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
