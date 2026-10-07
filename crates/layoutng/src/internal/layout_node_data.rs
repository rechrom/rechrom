#![allow(non_snake_case)]

use super::algorithm_forward::CustomLayoutChild;
use foundation::{GCedHeapVector, LayoutUnit, Member, PhysicalRect, Visitor};
use layoutng_geometry::geometry::axis::{
    kPhysicalAxesBoth, kPhysicalAxesHorizontal, kPhysicalAxesNone, kPhysicalAxesVertical,
    PhysicalAxes,
};

use super::gap::gap_geometry::GapGeometry;
use super::layout_block::LayoutBlock;
use super::layout_block_flow::LayoutBlockFlow;
use super::layout_box::LayoutBox;
use super::layout_box_model_object::LayoutBoxModelObject;
use super::scroll_layout_scope::ScrollbarFreezeState;

// This record is declared by layout_box.h; the constructor and tracer live in
// layout_node_data.cc. Its owning LayoutBox field is mapped with that header.
// cpp: layoutng/internal/layout_box.h:82-110
pub struct LayoutBoxRareData {
    pub(crate) has_override_containing_block_content_logical_width_: bool,
    pub(crate) has_previous_content_box_rect_: bool,
    pub(crate) override_containing_block_content_logical_width_: LayoutUnit,
    pub(crate) previous_physical_content_box_rect_: PhysicalRect,
    pub(crate) layout_child_: Member<CustomLayoutChild>,
    pub(crate) clear_layout_child_: Option<fn(*mut CustomLayoutChild)>,
    pub(crate) previous_gap_geometries_: Member<GCedHeapVector<Member<GapGeometry>>>,
}

impl LayoutBoxRareData {
    // cpp: layoutng/internal/layout_node_data.cc:234-237
    pub fn new() -> Self {
        Self {
            has_override_containing_block_content_logical_width_: false,
            has_previous_content_box_rect_: false,
            override_containing_block_content_logical_width_: LayoutUnit::default(),
            previous_physical_content_box_rect_: PhysicalRect::default(),
            layout_child_: Member::default(),
            clear_layout_child_: None,
            previous_gap_geometries_: Member::default(),
        }
    }

    // cpp: layoutng/internal/layout_node_data.cc:265-268
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.layout_child_);
        visitor.Trace(&self.previous_gap_geometries_);
    }
}

impl Default for LayoutBoxRareData {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutBox {
    // cpp: layoutng/internal/layout_node_data.cc:57-59
    pub fn GetScrollbarFreezeState(&self) -> *mut ScrollbarFreezeState {
        self.CheckIsNotDestroyed();
        if self.GetScrollableArea().is_null() {
            std::ptr::null_mut()
        } else {
            std::ptr::addr_of!(self.scrollbar_freeze_state_).cast_mut()
        }
    }

    // The base call is deliberately non-virtual after tracing LayoutBox fields.
    // cpp: layoutng/internal/layout_node_data.cc:296-303
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.min_max_sizes_cache_);
        visitor.Trace(&self.measure_cache_);
        visitor.Trace(&self.layout_results_);
        visitor.Trace(&self.overflow_);
        visitor.Trace(&self.rare_data_);
        let model = unsafe { &*(self as *const LayoutBox).cast::<LayoutBoxModelObject>() };
        model.Trace(visitor);
    }

    // cpp: layoutng/internal/layout_node_data.cc:391-415
    pub fn ScrollableAxesForLayout(&self) -> PhysicalAxes {
        if !self.IsScrollContainer() {
            return kPhysicalAxesNone;
        }
        if self.IsLayoutView() {
            return kPhysicalAxesBoth;
        }
        let mut axes = kPhysicalAxesNone;
        let style = self.StyleRef();
        if style.IsOverflowValueScrollableX() {
            axes |= kPhysicalAxesHorizontal;
        }
        if style.IsOverflowValueScrollableY() {
            axes |= kPhysicalAxesVertical;
        }
        axes
    }
}

impl LayoutBlock {
    // cpp: layoutng/internal/layout_block.h:99-99
    // cpp: layoutng/internal/layout_node_data.cc:329-332
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.children_);
        let box_ = unsafe { &*(self as *const LayoutBlock).cast::<LayoutBox>() };
        box_.Trace(visitor);
    }
}

impl LayoutBlockFlow {
    // cpp: layoutng/internal/layout_block_flow.h:65-65
    // cpp: layoutng/internal/layout_node_data.cc:338-341
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.inline_node_data_);
        let block = unsafe { &*(self as *const LayoutBlockFlow).cast::<LayoutBlock>() };
        block.Trace(visitor);
    }
}
