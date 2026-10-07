#![allow(non_snake_case)]

use std::ffi::c_void;

use foundation::DynamicTo;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::style_difference::StyleDifference;
use layoutng_style::style::style_position_anchor::Type as AnchorType;

use super::layout_box_model_object::LayoutBoxModelObject;
use super::layout_node_metadata::Element;
use super::layout_object::{LayoutObject, StyleChangeContext};
use super::loader::resource::image_resource_observer::CanDeferInvalidation;

// C++'s unqualified call in LayoutBoxModelObject::StyleDidChange is virtual.
// cpp: layoutng/internal/layout_box_model_lifecycle.cc:63
unsafe extern "Rust" {
    fn DispatchLayoutBoxModelUpdateFromStyle(object: *mut LayoutBoxModelObject);
}

// cpp: layoutng/internal/layout_box_model_lifecycle.cc:18-37
fn NeedsAnchorPositionScrollData(element: &Element, style: &ComputedStyle) -> bool {
    if !style.HasOutOfFlowPosition() {
        return false;
    }
    match style.GetDefaultAnchorData().GetType() {
        AnchorType::kNone => false,
        AnchorType::kAuto => !element.ImplicitAnchorElement().is_null(),
        AnchorType::kName => true,
        AnchorType::kNormal => std::process::abort(),
    }
}

impl LayoutBoxModelObject {
    // The pending Rust owner must retain its sole LayoutObject base at offset
    // zero. This static base call must not redispatch to the derived override.
    fn object_base_mut(&mut self) -> &mut LayoutObject {
        unsafe { &mut *(self as *mut LayoutBoxModelObject).cast::<LayoutObject>() }
    }

    // cpp: layoutng/internal/layout_box_model_lifecycle.cc:39-43
    pub fn WillBeDestroyed(&mut self) {
        self.CheckIsNotDestroyed();
        self.DisposeScrollableAreaForLayout();
        if self.HasLayer() {
            self.DestroyLayer();
        }
        self.object_base_mut().WillBeDestroyed();
    }

    // cpp: layoutng/internal/layout_box_model_lifecycle.cc:45-53
    pub fn StyleWillChange(
        &mut self,
        diff: StyleDifference,
        old_style: *const ComputedStyle,
        new_style: &ComputedStyle,
        style_change_context: &mut StyleChangeContext,
    ) {
        self.CheckIsNotDestroyed();
        self.object_base_mut()
            .StyleWillChange(diff, old_style, new_style, style_change_context);
    }

    // cpp: layoutng/internal/layout_box_model_lifecycle.cc:55-75
    pub fn StyleDidChange(
        &mut self,
        diff: StyleDifference,
        old_style: *const ComputedStyle,
        new_style: &ComputedStyle,
        style_change_context: &StyleChangeContext,
    ) {
        self.CheckIsNotDestroyed();
        self.object_base_mut()
            .StyleDidChange(diff, old_style, new_style, style_change_context);
        unsafe { DispatchLayoutBoxModelUpdateFromStyle(self) };
        self.UpdateTransformAfterStyleChangeForLayout(diff, old_style, new_style);
        self.UpdateLayerAfterStyleChange();
        let element = DynamicTo::<Element>(self.GetNode());
        if !element.is_null() {
            if NeedsAnchorPositionScrollData(unsafe { &*element }, new_style) {
                unsafe { &mut *element }.EnsureAnchorPositionScrollData();
            } else {
                unsafe { &mut *element }.RemoveAnchorPositionScrollData();
            }
        }
    }

    // cpp: layoutng/internal/layout_box_model_lifecycle.cc:77-80
    pub fn RecalcVisualOverflow(&mut self) {
        self.CheckIsNotDestroyed();
        self.object_base_mut().RecalcVisualOverflow();
    }

    // cpp: layoutng/internal/layout_box_model_lifecycle.cc:82-87
    pub fn ImageChanged(&mut self, _image: *const c_void, _defer: CanDeferInvalidation) {
        std::process::abort();
    }
}
