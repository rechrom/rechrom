#![allow(non_snake_case)]

use foundation::{HeapHashMap, Member, PhysicalRect, PhysicalSize, Traceable, Visitor};
use layoutng_fragment_tree::box_fragment_builder::BoxFragmentBuilder;
use layoutng_geometry::geometry::logical_offset::LogicalOffset;

use super::layout_box::LayoutBox;
use super::layout_object::LayoutObject;

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
pub struct InlineContainingBlockUtils {
    _private: (),
}

impl InlineContainingBlockUtils {
    // cpp: layoutng/internal/inline_containing_block_utils.h:44-50
    pub fn ComputeInlineContainerGeometry(
        inline_containing_block_map: *mut InlineContainingBlockMap,
        container_builder: *mut BoxFragmentBuilder,
    ) {
        unsafe {
            ComputeInlineContainerGeometryProvider(inline_containing_block_map, container_builder)
        }
    }

    // cpp: layoutng/internal/inline_containing_block_utils.h:52-61
    pub fn ComputeInlineContainerGeometryForFragmentainer(
        box_: *const LayoutBox,
        accumulated_containing_block_size: PhysicalSize,
        inline_containing_block_map: *mut InlineContainingBlockMap,
    ) {
        unsafe {
            ComputeInlineContainerGeometryForFragmentainerProvider(
                box_,
                accumulated_containing_block_size,
                inline_containing_block_map,
            )
        }
    }
}

// Implementations belong to //src/layoutng_out_of_flow.
unsafe extern "Rust" {
    fn ComputeInlineContainerGeometryProvider(
        inline_containing_block_map: *mut InlineContainingBlockMap,
        container_builder: *mut BoxFragmentBuilder,
    );

    fn ComputeInlineContainerGeometryForFragmentainerProvider(
        box_: *const LayoutBox,
        accumulated_containing_block_size: PhysicalSize,
        inline_containing_block_map: *mut InlineContainingBlockMap,
    );
}
