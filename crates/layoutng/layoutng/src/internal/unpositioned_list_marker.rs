#![allow(non_snake_case)]

use font_engine::FontBaseline;
use foundation::{LayoutUnit, Member, Visitor};
use layoutng_fragment_tree::box_fragment_builder::BoxFragmentBuilder;
use layoutng_fragment_tree::layout_result::LayoutResult;
use layoutng_fragment_tree::physical_fragment::PhysicalFragment;
use layoutng_geometry::geometry::box_strut::BoxStrut;
use layoutng_style::style::computed_style::ComputedStyle;

use super::block_node::BlockNode;
use super::constraint_space::ConstraintSpace;
use super::layout_box::LayoutBox;

// cpp: layoutng/internal/unpositioned_list_marker.h:50-51,102-109
// The C++ class is stack-only. Rust callers construct it by value; its single
// traced member retains the original data layout and ownership boundary.
#[repr(C)]
pub struct UnpositionedListMarker {
    marker_layout_object_: Member<LayoutBox>,
}

// The source's implicit copy constructor copies its single Member handle.
impl Clone for UnpositionedListMarker {
    fn clone(&self) -> Self {
        Self {
            marker_layout_object_: self.marker_layout_object_.clone(),
        }
    }
}

impl Default for UnpositionedListMarker {
    // cpp: layoutng/internal/unpositioned_list_marker.h:54
    fn default() -> Self {
        Self {
            marker_layout_object_: Member::from_ptr(std::ptr::null_mut()),
        }
    }
}

impl PartialEq for UnpositionedListMarker {
    // cpp: layoutng/internal/unpositioned_list_marker.h:88-90
    fn eq(&self, other: &Self) -> bool {
        self.marker_layout_object_.Get() == other.marker_layout_object_.Get()
    }
}

impl Eq for UnpositionedListMarker {}

#[allow(non_snake_case)]
impl UnpositionedListMarker {
    // cpp: layoutng/internal/unpositioned_list_marker.h:55
    // The constructor body belongs to //src/layoutng_block.
    pub fn new(node: &BlockNode) -> Self {
        let marker = unsafe { UnpositionedListMarkerConstructFromBlock(node) };
        Self {
            marker_layout_object_: Member::from_ptr(marker),
        }
    }

    // cpp: layoutng/internal/unpositioned_list_marker.h:57
    pub fn is_non_null(&self) -> bool {
        !self.marker_layout_object_.Get().is_null()
    }

    pub fn is_present(&self) -> bool {
        self.is_non_null()
    }

    // Cross-crate provider access to the private C++ member. This is the only
    // additional bridge required for the block package's method bodies.
    pub fn MarkerLayoutObject(&self) -> *mut LayoutBox {
        self.marker_layout_object_.Get()
    }

    // The block package supplies the Trace body, but this Member is owned by
    // the layoutng header. Keep the GC visit on the actual member field.
    pub fn TraceMarkerMember(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.marker_layout_object_);
    }

    // cpp: layoutng/internal/unpositioned_list_marker.h:64-67
    pub fn ContentAlignmentBaseline(
        &self,
        space: &ConstraintSpace,
        baseline: FontBaseline,
        content: &PhysicalFragment,
    ) -> Option<LayoutUnit> {
        unsafe {
            UnpositionedListMarkerContentAlignmentBaselineFromBlock(self, space, baseline, content)
        }
    }

    // cpp: layoutng/internal/unpositioned_list_marker.h:69-76
    pub fn AddToBox(
        &self,
        space: &ConstraintSpace,
        baseline: FontBaseline,
        content: &PhysicalFragment,
        border_scrollbar_padding: &BoxStrut,
        marker_layout_result: &LayoutResult,
        content_baseline: LayoutUnit,
        block_offset: &mut LayoutUnit,
        builder: *mut BoxFragmentBuilder,
    ) {
        unsafe {
            UnpositionedListMarkerAddToBoxFromBlock(
                self,
                space,
                baseline,
                content,
                border_scrollbar_padding,
                marker_layout_result,
                content_baseline,
                block_offset,
                builder,
            )
        }
    }

    // cpp: layoutng/internal/unpositioned_list_marker.h:81-86
    pub fn AddToBoxWithoutLineBoxes(
        &self,
        space: &ConstraintSpace,
        baseline: FontBaseline,
        marker_layout_result: &LayoutResult,
        builder: *mut BoxFragmentBuilder,
        intrinsic_block_size: &mut LayoutUnit,
    ) {
        unsafe {
            UnpositionedListMarkerAddToBoxWithoutLineBoxesFromBlock(
                self,
                space,
                baseline,
                marker_layout_result,
                builder,
                intrinsic_block_size,
            )
        }
    }

    pub fn InlineOffset(&self, marker_inline_size: LayoutUnit) -> LayoutUnit {
        unsafe { UnpositionedListMarkerInlineOffsetFromBlock(self, marker_inline_size) }
    }

    // cpp: layoutng/internal/unpositioned_list_marker.h:92-94
    pub fn Layout(
        &self,
        parent_space: &ConstraintSpace,
        parent_style: &ComputedStyle,
        baseline: FontBaseline,
    ) -> *const LayoutResult {
        unsafe { UnpositionedListMarkerLayoutFromBlock(self, parent_space, parent_style, baseline) }
    }

    // cpp: layoutng/internal/unpositioned_list_marker.h:96-100
    #[cfg(debug_assertions)]
    pub fn CheckMargin(&self) {
        unsafe { UnpositionedListMarkerCheckMarginFromBlock(self) }
    }

    pub fn Trace(&self, visitor: &mut Visitor) {
        unsafe { UnpositionedListMarkerTraceFromBlock(self, visitor) }
    }

    // cpp: layoutng/internal/unpositioned_list_marker.h:103-106
    #[doc(hidden)]
    pub fn ComputeIntrudedFloatOffset(
        &self,
        space: &ConstraintSpace,
        builder: *const BoxFragmentBuilder,
        border_scrollbar_padding: &BoxStrut,
        marker_block_offset: LayoutUnit,
    ) -> LayoutUnit {
        unsafe {
            UnpositionedListMarkerComputeIntrudedFloatOffsetFromBlock(
                self,
                space,
                builder,
                border_scrollbar_padding,
                marker_block_offset,
            )
        }
    }
}

// Rust cannot attach inherent methods to this owner from //src/layoutng_block.
// These declarations are typed links, not substitute implementations.
unsafe extern "Rust" {
    fn UnpositionedListMarkerConstructFromBlock(node: &BlockNode) -> *mut LayoutBox;
    fn UnpositionedListMarkerContentAlignmentBaselineFromBlock(
        marker: &UnpositionedListMarker,
        space: &ConstraintSpace,
        baseline: FontBaseline,
        content: &PhysicalFragment,
    ) -> Option<LayoutUnit>;
    fn UnpositionedListMarkerAddToBoxFromBlock(
        marker: &UnpositionedListMarker,
        space: &ConstraintSpace,
        baseline: FontBaseline,
        content: &PhysicalFragment,
        border_scrollbar_padding: &BoxStrut,
        marker_layout_result: &LayoutResult,
        content_baseline: LayoutUnit,
        block_offset: &mut LayoutUnit,
        builder: *mut BoxFragmentBuilder,
    );
    fn UnpositionedListMarkerAddToBoxWithoutLineBoxesFromBlock(
        marker: &UnpositionedListMarker,
        space: &ConstraintSpace,
        baseline: FontBaseline,
        marker_layout_result: &LayoutResult,
        builder: *mut BoxFragmentBuilder,
        intrinsic_block_size: &mut LayoutUnit,
    );
    fn UnpositionedListMarkerInlineOffsetFromBlock(
        marker: &UnpositionedListMarker,
        marker_inline_size: LayoutUnit,
    ) -> LayoutUnit;
    fn UnpositionedListMarkerLayoutFromBlock(
        marker: &UnpositionedListMarker,
        parent_space: &ConstraintSpace,
        parent_style: &ComputedStyle,
        baseline: FontBaseline,
    ) -> *const LayoutResult;
    #[cfg(debug_assertions)]
    fn UnpositionedListMarkerCheckMarginFromBlock(marker: &UnpositionedListMarker);
    fn UnpositionedListMarkerTraceFromBlock(marker: &UnpositionedListMarker, visitor: &mut Visitor);
    fn UnpositionedListMarkerComputeIntrudedFloatOffsetFromBlock(
        marker: &UnpositionedListMarker,
        space: &ConstraintSpace,
        builder: *const BoxFragmentBuilder,
        border_scrollbar_padding: &BoxStrut,
        marker_block_offset: LayoutUnit,
    ) -> LayoutUnit;
}
