use foundation::{EClear, EFloat, LayoutUnit, TextDirection};
use layoutng_fragment_tree::block_break_token::BlockBreakToken;
use layoutng_fragment_tree::layout_result::LayoutResult;
use layoutng_geometry::geometry::bfc_offset::BfcOffset;
use layoutng_geometry::geometry::box_strut::BoxStrut;
use layoutng_geometry::geometry::logical_size::LogicalSize;
use layoutng_style::style::computed_style::ComputedStyle;

use super::block_node::BlockNode;
use super::constraint_space::ConstraintSpace;

// cpp: layoutng/internal/unpositioned_float.h:23-64
pub struct UnpositionedFloat<'a> {
    pub node: BlockNode,
    pub token: *const BlockBreakToken,
    pub available_size: LogicalSize,
    pub percentage_size: LogicalSize,
    pub origin_bfc_offset: BfcOffset,
    pub parent_space: &'a ConstraintSpace,
    pub parent_style: &'a ComputedStyle,
    pub fragmentainer_block_size: LayoutUnit,
    pub fragmentainer_block_offset: LayoutUnit,
    pub is_hidden_for_paint: bool,
    pub layout_result: *const LayoutResult,
    pub margins: BoxStrut,
}

#[allow(non_snake_case)]
impl<'a> UnpositionedFloat<'a> {
    // cpp: layoutng/internal/unpositioned_float.h:28-47
    pub fn new(
        node: BlockNode,
        token: *const BlockBreakToken,
        available_size: LogicalSize,
        percentage_size: LogicalSize,
        origin_bfc_offset: &BfcOffset,
        parent_space: &'a ConstraintSpace,
        parent_style: &'a ComputedStyle,
        fragmentainer_block_size: LayoutUnit,
        fragmentainer_block_offset: LayoutUnit,
        is_hidden_for_paint: bool,
    ) -> Self {
        Self {
            node,
            token,
            available_size,
            percentage_size,
            origin_bfc_offset: *origin_bfc_offset,
            parent_space,
            parent_style,
            fragmentainer_block_size,
            fragmentainer_block_offset,
            is_hidden_for_paint,
            layout_result: std::ptr::null(),
            margins: BoxStrut::default(),
        }
    }

    // cpp: layoutng/internal/unpositioned_float.h:66-74
    pub fn IsLineLeft(&self, cb_direction: TextDirection) -> bool {
        self.node.Style().FloatingWithDirection(cb_direction) == EFloat::kLeft
    }

    pub fn IsLineRight(&self, cb_direction: TextDirection) -> bool {
        self.node.Style().FloatingWithDirection(cb_direction) == EFloat::kRight
    }

    pub fn ClearType(&self, cb_direction: TextDirection) -> EClear {
        self.node.Style().ClearWithDirection(cb_direction)
    }

    // cpp: layoutng/internal/unpositioned_float.h:76-80
    pub fn FragmentainerSpaceLeft(&self) -> LayoutUnit {
        let space = self.fragmentainer_block_size - self.fragmentainer_block_offset;
        space.ClampNegativeToZero()
    }

    // cpp: layoutng/internal/unpositioned_float.h:82-85
    pub fn FragmentainerOffsetAtBfc(&self) -> LayoutUnit {
        self.fragmentainer_block_offset - self.parent_space.ExpectedBfcBlockOffset()
    }
}
