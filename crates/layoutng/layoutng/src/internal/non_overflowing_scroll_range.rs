use super::layout_node_metadata::Element;
use foundation::{Member, PhysicalOffset, Visitor};
use layoutng_geometry::geometry::scroll_offset_range::PhysicalScrollRange;

// cpp: layoutng/internal/non_overflowing_scroll_range.h:20-42
#[derive(Clone, Default)]
pub struct NonOverflowingScrollRange {
    pub containing_block_range: PhysicalScrollRange,
    pub anchor_element: Member<Element>,
}

#[allow(non_snake_case)]
impl NonOverflowingScrollRange {
    pub fn Contains(&self, anchor_scroll_offset: &PhysicalOffset) -> bool {
        self.containing_block_range.Contains(anchor_scroll_offset)
    }

    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.anchor_element);
    }
}

impl PartialEq for NonOverflowingScrollRange {
    fn eq(&self, other: &Self) -> bool {
        self.containing_block_range == other.containing_block_range
    }
}

// cpp: layoutng/internal/non_overflowing_scroll_range.h:43-43
// ToString is declared here but has no definition in the supplied source tree.
