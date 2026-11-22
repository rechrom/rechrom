#![allow(non_snake_case)]

use foundation::{HeapHashMap, Member, PhysicalRect, Traceable, Visitor};
use layout_geometry::geometry::logical_offset::LogicalOffset;

use super::super::layout_object::LayoutObject;

// cpp: layoutng/internal/inline_containing_block_utils.h:25-37
#[derive(Default)]
pub struct InlineContainingBlockGeometry {
    pub start_fragment_union_rect: PhysicalRect,
    pub end_fragment_union_rect: PhysicalRect,
    pub relative_offset: LogicalOffset,
    pub is_hidden_for_paint: bool,
}

impl Traceable for InlineContainingBlockGeometry {
    fn Trace(&self, _visitor: &mut Visitor<'_>) {}
}

// The source key is Member<const LayoutObject>. Rust Member retains the same
// traced object; callers receive only a borrowed key from this map.
// cpp: layoutng/internal/inline_containing_block_utils.h:39-42
pub type InlineContainingBlockMap =
    HeapHashMap<Member<LayoutObject>, Option<InlineContainingBlockGeometry>>;

// cpp: layoutng/internal/inline_containing_block_utils.h:21-24
// The static members ComputeInlineContainerGeometry(ForFragmentainer) are
// out-of-flow algorithms (layoutng_out_of_flow/inline_containing_block_utils.cc)
// that only the out-of-flow package calls: the extension trait
// `layout_algo_out_of_flow::inline_containing_block_utils::
// InlineContainingBlockUtilsAlgorithms`.
pub struct InlineContainingBlockUtils {
    _private: (),
}
