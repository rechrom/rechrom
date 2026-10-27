#![allow(non_snake_case)]

use foundation::{LayoutUnit, MinimumValueForLength};
use layoutng_geometry::geometry::box_strut::PhysicalBoxStrut;

use super::layout_box_model_object::LayoutBoxModelObject;
use super::layout_node_metadata::{ContainerNode, Node};
use super::layout_object::LayoutObject;

#[path = "../../../layout_invalidation/src/native_layout_box_model_tree_mutation.rs"]
mod layout_invalidation_native_layout_box_model_tree_mutation;

// The derived owner calls this when constructing its LayoutObject base.
// Its own fields belong to the pending layout_box_model_object.h record.
// cpp: layoutng/internal/layout_box_model_tree.cc:33-35
pub fn LayoutBoxModelObjectBaseForNode(node: *mut ContainerNode) -> LayoutObject {
    LayoutObject::new_base(node as *mut Node)
}

// cpp: layoutng/internal/layout_box_model_object.h:327-355
unsafe extern "Rust" {
    fn DispatchLayoutBoxModelObjectCanMergeWith(
        owner: *const LayoutBoxModelObject,
        other: &LayoutBoxModelObject,
    ) -> bool;
    fn DispatchLayoutBoxModelObjectContainingBlockLogicalWidthForContent(
        owner: *const LayoutBoxModelObject,
    ) -> LayoutUnit;
}

impl LayoutBoxModelObject {
    // cpp: layoutng/internal/layout_box_model_object.h:350-353
    pub fn CanMergeWithBase(&self, _other: &LayoutBoxModelObject) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn CanMergeWith(&self, other: &LayoutBoxModelObject) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutBoxModelObjectCanMergeWith(self, other) }
    }

    pub fn ContainingBlockLogicalWidthForContent(&self) -> LayoutUnit {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutBoxModelObjectContainingBlockLogicalWidthForContent(self) }
    }

    // cpp: layoutng/internal/layout_box_model_tree.cc:181-198
    pub fn ComputedPaddingOutsets(&self) -> PhysicalBoxStrut {
        self.CheckIsNotDestroyed();
        let style = self.StyleRef();
        if !style.MayHavePadding() {
            return PhysicalBoxStrut::default();
        }
        let percentage_size = if style.PaddingTop().HasPercent()
            || style.PaddingRight().HasPercent()
            || style.PaddingBottom().HasPercent()
            || style.PaddingLeft().HasPercent()
        {
            self.ContainingBlockLogicalWidthForContent()
        } else {
            LayoutUnit::default()
        };
        PhysicalBoxStrut::new(
            MinimumValueForLength(style.PaddingTop(), percentage_size),
            MinimumValueForLength(style.PaddingRight(), percentage_size),
            MinimumValueForLength(style.PaddingBottom(), percentage_size),
            MinimumValueForLength(style.PaddingLeft(), percentage_size),
        )
    }

    // cpp: layoutng/internal/layout_box_model_tree.cc:200-203
    pub fn ContainingBlockLogicalWidthForContentBase(&self) -> LayoutUnit {
        self.CheckIsNotDestroyed();
        unsafe { &*self.ContainingBlock() }.ContentLogicalWidth()
    }
}
