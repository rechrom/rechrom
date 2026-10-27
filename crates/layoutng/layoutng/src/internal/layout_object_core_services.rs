#![allow(non_snake_case)]

use foundation::blink_geometry::transforms::affine_transform::AffineTransform;
use foundation::graphics_types;
use foundation::{
    gfx, DynamicTo, EDisplay, ETextTransform, IsHorizontalTypographicMode, LayoutUnit,
    PhysicalOffset, String as BlinkString, StringBuilder, To, UnsupportedLayout, ValuesEquivalent,
};
use graphics_types::graphics::dom_node_id::{kInvalidDOMNodeId, DOMNodeId};
use layoutng_geometry::geometry::axis::PhysicalAxis;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::style_difference::StyleDifference;

use super::form_control_types::FormControlType;
use super::form_node_metadata::HTMLFormControlElement;
use super::layout_box::LayoutBox;
use super::layout_box_model_object::LayoutBoxModelObject;
use super::layout_node_metadata::Element;
use super::layout_object::{
    ApplyStyleChanges, LayoutObject, RecalcScrollableOverflowResult, StyleChangeContext,
};
use super::layout_pass_scope::LayoutObjectFactoryScope;
use super::layout_pass_scope::LayoutPassScope;
use super::layout_text::LayoutText;
use super::map_coordinates_flags::{MapCoordinatesFlags, MapCoordinatesMode};
use super::svg_layout_info::{SVGLayoutInfo, SVGLayoutResult};

#[path = "../../../layout_invalidation/src/native_layout_object_style_update.rs"]
mod layout_invalidation_native_layout_object_style_update;

#[path = "../../../layout_invalidation/src/native_layout_object_tree_lifecycle.rs"]
mod layout_invalidation_native_layout_object_tree_lifecycle;

#[path = "../../../layout_invalidation/src/native_layout_object_child_mutation.rs"]
mod layout_invalidation_native_layout_object_child_mutation;

#[path = "../../../layout_fragment/src/native_layout_object_overflow_recalc.rs"]
mod layout_fragment_native_layout_object_overflow_recalc;

#[path = "../../../layout_fragment/src/native_layout_object_geometry_queries.rs"]
mod layout_fragment_native_layout_object_geometry_queries;

#[path = "../../../layout_reuse/src/native_layout_object_font_validity.rs"]
mod layout_reuse_native_layout_object_font_validity;

#[path = "../../../layout_algo_svg/src/native_layout_object_svg_dispatch.rs"]
mod layout_algo_svg_native_layout_object_svg_dispatch;

// The concrete hierarchy must supply each virtual dispatch entry.
// cpp: layoutng/internal/layout_object_core_services.cc:107-162
unsafe extern "Rust" {
    fn DispatchLayoutObjectAddChild(
        object: *mut LayoutObject,
        new_child: *mut LayoutObject,
        before_child: *mut LayoutObject,
    );
    fn DispatchLayoutObjectRemoveChild(object: *mut LayoutObject, old_child: *mut LayoutObject);
    fn DispatchLayoutObjectInsertedIntoTree(object: *mut LayoutObject);
    fn DispatchLayoutObjectWillBeRemovedFromTree(object: *mut LayoutObject);
}

// cpp: layoutng/internal/layout_object_core_services.cc:164-203
unsafe extern "Rust" {
    fn DispatchLayoutObjectStyleWillChange(
        object: *mut LayoutObject,
        difference: StyleDifference,
        old_style: *const ComputedStyle,
        new_style: &ComputedStyle,
        context: &mut StyleChangeContext,
    );
    fn DispatchLayoutObjectStyleDidChange(
        object: *mut LayoutObject,
        difference: StyleDifference,
        old_style: *const ComputedStyle,
        new_style: &ComputedStyle,
        context: &StyleChangeContext,
    );
}

// cpp: layoutng/internal/layout_object_core_services.cc:280-300
unsafe extern "Rust" {
    fn DispatchLayoutObjectRecalcScrollableOverflow(
        object: *mut LayoutObject,
    ) -> RecalcScrollableOverflowResult;
    fn DispatchLayoutObjectRecalcVisualOverflow(object: *mut LayoutObject);
}

// cpp: layoutng/internal/layout_object.h:321-321
unsafe extern "Rust" {
    fn DispatchLayoutObjectGetName(object: *const LayoutObject) -> &'static str;
}

impl LayoutObject {
    // cpp: layoutng/internal/layout_object_core_services.cc:315-318
    pub fn IsInlineRuby(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.IsLayoutInline() && self.StyleRef().Display() == EDisplay::kRuby
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:320-323
    pub fn IsInlineRubyText(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.IsLayoutInline() && self.StyleRef().Display() == EDisplay::kRubyText
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:325-343
    pub fn IsButtonOrInputButton(&self) -> bool {
        self.CheckIsNotDestroyed();
        let control = DynamicTo::<HTMLFormControlElement>(self.GetNode());
        if control.is_null() {
            return false;
        }
        matches!(
            unsafe { &*control }.FormControlType(),
            FormControlType::kButtonButton
                | FormControlType::kButtonSubmit
                | FormControlType::kButtonReset
                | FormControlType::kButtonPopover
                | FormControlType::kInputButton
                | FormControlType::kInputImage
                | FormControlType::kInputReset
                | FormControlType::kInputSubmit
        )
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:362-372
    pub fn IsScrollMarkerGroup(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsInTopOrViewTransitionLayer(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:402-408
    pub fn IsRelayoutBoundary(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    // GetName() is pure virtual in the source header.
    // cpp: layoutng/internal/layout_object.h:321-321
    pub fn GetName(&self) -> &'static str {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectGetName(self) }
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:489-499
    pub fn DebugName(&self) -> BlinkString {
        self.CheckIsNotDestroyed();
        let mut result = StringBuilder::default();
        result.Append(self.GetName());
        let node = self.GetNode();
        if !node.is_null() {
            let debug_name = unsafe { &*node }.InputDebugName();
            if !debug_name.is_empty() {
                result.Append(" ");
                result.Append(debug_name);
            }
        }
        result.ToString()
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:501-505
    pub fn IsStyleGenerated(&self) -> bool {
        self.CheckIsNotDestroyed();
        let node = self.GetNode();
        !node.is_null() && unsafe { &*node }.InputIsStyleGenerated()
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:507-511
    pub fn ToString(&self) -> BlinkString {
        self.CheckIsNotDestroyed();
        self.DebugName()
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:513-522
    pub fn OwnerNodeId(&self, _is_internal_content: bool) -> DOMNodeId {
        self.CheckIsNotDestroyed();
        let node = self.GetNode();
        if node.is_null()
            || unsafe { &*node }.InputId() == 0
            || unsafe { &*node }.InputId() > DOMNodeId::MAX as u64
        {
            return kInvalidDOMNodeId;
        }
        unsafe { &*node }.InputId() as DOMNodeId
    }

    // cpp: layoutng/internal/layout_object.h:351-351
    pub fn OwnerNodeIdDefault(&self) -> DOMNodeId {
        self.OwnerNodeId(false)
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:525-529
    #[cfg(debug_assertions)]
    pub fn ShowLayoutTreeForThis(&self) {
        self.CheckIsNotDestroyed();
        eprintln!("{:p}:{}", self, self.DebugName());
    }
}

// cpp: layoutng/internal/layout_object_core_services.cc:531-534
impl std::fmt::Display for LayoutObject {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(out, "{:p}:{}", self, self.DebugName())
    }
}

// Rust's orphan rule prevents implementing Display for a raw pointer.
// cpp: layoutng/internal/layout_object_core_services.cc:536-538
pub struct LayoutObjectPointerDisplay(pub *const LayoutObject);

impl std::fmt::Display for LayoutObjectPointerDisplay {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_null() {
            out.write_str("<null>")
        } else {
            write!(out, "{}", unsafe { &*self.0 })
        }
    }
}
