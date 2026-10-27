#![allow(non_snake_case)]

use foundation::UnsupportedLayout;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::style_difference::StyleDifference;

use super::layout_box::LayoutBox;
use super::layout_box_model_object::{LayoutBoxModelObject, PaintLayerType};
use super::layout_invalidation_reason;
use super::layout_object::StyleChangeContext;
use super::layout_pass_scope::LayoutObjectFactoryScope;
use super::shapes::shape_outside_info::ShapeOutsideInfo;

#[path = "../../../layout_invalidation/src/native_layout_box_style_change.rs"]
mod layout_invalidation_native_layout_box_style_change;
use layout_invalidation_native_layout_box_style_change::{
    LayoutBoxStyleDidChange, LayoutBoxStyleWillChange,
};

#[path = "../../../layout_invalidation/src/native_layout_box_tree_lifecycle.rs"]
mod layout_invalidation_native_layout_box_tree_lifecycle;
use layout_invalidation_native_layout_box_tree_lifecycle::{
    LayoutBoxInsertedIntoTree, LayoutBoxWillBeDestroyed, LayoutBoxWillBeRemovedFromTree,
};

impl LayoutBox {
    // LayoutBox has one C++ base, LayoutBoxModelObject, at offset zero. Its
    // pending Rust owner must retain that first-field layout for static base
    // calls to reach the same subobject without virtual redispatch.
    fn model_object_base_mut(&mut self) -> &mut LayoutBoxModelObject {
        unsafe { &mut *(self as *mut LayoutBox).cast::<LayoutBoxModelObject>() }
    }

    // cpp: layoutng/internal/layout_box_lifecycle.cc:22-31
    pub fn LayerTypeRequired(&self) -> PaintLayerType {
        self.CheckIsNotDestroyed();
        if self.IsStacked() || self.HasHiddenBackface() {
            return PaintLayerType::kNormalPaintLayer;
        }
        if self.HasNonVisibleOverflow() && !self.IsLayoutReplaced() {
            return PaintLayerType::kOverflowClipPaintLayer;
        }
        if self.IsOverscrollContainer() {
            return PaintLayerType::kForcedPaintLayer;
        }
        PaintLayerType::kNoPaintLayer
    }

    // cpp: layoutng/internal/layout_box_lifecycle.cc:33-38
    pub fn WillBeDestroyed(&mut self) {
        LayoutBoxWillBeDestroyed(self)
    }

    // cpp: layoutng/internal/layout_box_lifecycle.cc:40-44
    pub fn InsertedIntoTree(&mut self) {
        LayoutBoxInsertedIntoTree(self)
    }

    // cpp: layoutng/internal/layout_box_lifecycle.cc:46-50
    pub fn WillBeRemovedFromTree(&mut self) {
        LayoutBoxWillBeRemovedFromTree(self)
    }

    // cpp: layoutng/internal/layout_box_lifecycle.cc:52-68
    pub fn StyleWillChange(
        &mut self,
        diff: StyleDifference,
        old_style: *const ComputedStyle,
        new_style: &ComputedStyle,
        style_change_context: &mut StyleChangeContext,
    ) {
        LayoutBoxStyleWillChange(self, diff, old_style, new_style, style_change_context)
    }

    // cpp: layoutng/internal/layout_box_lifecycle.cc:70-99
    pub fn StyleDidChange(
        &mut self,
        diff: StyleDifference,
        old_style: *const ComputedStyle,
        new_style: &ComputedStyle,
        style_change_context: &StyleChangeContext,
    ) {
        LayoutBoxStyleDidChange(self, diff, old_style, new_style, style_change_context)
    }
}
