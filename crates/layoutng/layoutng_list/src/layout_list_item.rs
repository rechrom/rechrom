#![allow(non_snake_case)]

use foundation::{To, Traceable, Visitor};
use layoutng_assembly::internal::layout_block_flow::LayoutBlockFlow;
use layoutng_assembly::internal::layout_node_metadata::Element;
use layoutng_assembly::internal::layout_object::LayoutObject;
use layoutng_assembly::internal::layout_object::{LayoutObjectClass, StyleChangeContext};
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::style_difference::StyleDifference;

use crate::layout_inline_list_item::LayoutInlineListItem;
use crate::list_item_ordinal::ListItemOrdinal;
use crate::list_marker::ListMarker;

// cpp: layoutng_list/layout_list_item.h:15-18,60-61
#[repr(C)]
pub struct LayoutListItem {
    flow: LayoutBlockFlow,
    ordinal: ListItemOrdinal,
}

impl LayoutListItem {
    // cpp: layoutng_list/layout_list_item.cc:12-14
    pub fn new(element: *mut Element) -> Self {
        let mut flow = LayoutBlockFlow::new(element.cast());
        flow.SetRuntimeClass(LayoutObjectClass::ListItem);
        flow.SetInline(false);
        Self {
            flow,
            ordinal: ListItemOrdinal::new(),
        }
    }

    // cpp: layoutng_list/layout_list_item.h:20-23
    pub fn Ordinal(&mut self) -> &mut ListItemOrdinal {
        &mut self.ordinal
    }

    // cpp: layoutng_list/layout_list_item.h:27-31
    pub fn Marker(&self) -> *mut LayoutObject {
        let node = self.GetNode();
        debug_assert!(!node.is_null());
        let list_item = To::<Element>(node);
        unsafe { &*list_item }.InputListMarkerLayoutObject()
    }

    // cpp: layoutng_list/layout_list_item.cc:17-20
    pub fn WillBeDestroyed(&mut self) {
        self.flow.WillBeDestroyed();
    }

    // cpp: layoutng_list/layout_list_item.cc:22-25
    pub fn InsertedIntoTree(&mut self) {
        self.flow.InsertedIntoTree();
    }

    // cpp: layoutng_list/layout_list_item.cc:27-30
    pub fn WillBeRemovedFromTree(&mut self) {
        self.flow.WillBeRemovedFromTree();
    }

    // cpp: layoutng_list/layout_list_item.cc:32-40
    pub fn StyleDidChange(
        &mut self,
        diff: StyleDifference,
        old_style: *const ComputedStyle,
        new_style: &ComputedStyle,
        style_change_context: &StyleChangeContext,
    ) {
        self.flow
            .StyleDidChange(diff, old_style, new_style, style_change_context);
    }

    // cpp: layoutng_list/layout_list_item.cc:42-46
    pub fn OrdinalValueChanged(&mut self) {
        let marker = self.Marker();
        let list_marker = ListMarker::GetMut(marker);
        if !list_marker.is_null() {
            unsafe { &mut *list_marker }.OrdinalValueChanged(unsafe { &mut *marker });
        }
    }

    // cpp: layoutng_list/layout_list_item.cc:48-49
    pub fn SubtreeDidChange(&mut self) {}

    // cpp: layoutng_list/layout_list_item.h:36-36
    // cpp: layoutng_list/layout_list_item.cc:51-53
    pub fn WillCollectInlines(&mut self) {
        self.UpdateMarkerTextIfNeeded();
    }

    // cpp: layoutng_list/layout_list_item.h:33-33
    // cpp: layoutng_list/layout_list_item.cc:55-59
    pub fn UpdateMarkerTextIfNeeded(&mut self) {
        let marker = self.Marker();
        let list_marker = ListMarker::GetMut(marker);
        if !list_marker.is_null() {
            unsafe { &mut *list_marker }.UpdateMarkerTextIfNeeded(unsafe { &mut *marker });
        }
    }

    // cpp: layoutng_list/layout_list_item.h:25-25
    // cpp: layoutng_list/layout_list_item.cc:61-64
    pub fn Value(&self) -> i32 {
        let node = self.GetNode();
        debug_assert!(!node.is_null());
        self.ordinal.Value(unsafe { &*node })
    }

    // cpp: layoutng_list/layout_list_item.h:38-38
    // cpp: layoutng_list/layout_list_item.cc:66-91
    pub fn FindSymbolMarkerLayoutText(object: *const LayoutObject) -> *const LayoutObject {
        if object.is_null() {
            return std::ptr::null();
        }
        let list_marker = ListMarker::Get(object);
        if !list_marker.is_null() {
            return unsafe { &*list_marker }.SymbolMarkerLayoutText(unsafe { &*object });
        }
        let object_ref = unsafe { &*object };
        if object_ref.IsLayoutListItem() {
            return Self::FindSymbolMarkerLayoutText(
                unsafe { &*object.cast::<LayoutListItem>() }.Marker(),
            );
        }
        if object_ref.IsInlineListItem() {
            return Self::FindSymbolMarkerLayoutText(
                unsafe { &*object.cast::<LayoutInlineListItem>() }.Marker(),
            );
        }
        if object_ref.IsAnonymousBlockFlow() || object_ref.IsLayoutTextCombine() {
            return Self::FindSymbolMarkerLayoutText(object_ref.Parent());
        }
        std::ptr::null()
    }

    // cpp: layoutng_list/layout_list_item.h:40-43
    pub fn GetName(&self) -> &'static str {
        "LayoutListItem"
    }
}

impl std::ops::Deref for LayoutListItem {
    type Target = LayoutBlockFlow;
    fn deref(&self) -> &Self::Target {
        &self.flow
    }
}

impl std::ops::DerefMut for LayoutListItem {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.flow
    }
}

impl Traceable for LayoutListItem {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.flow.Trace(visitor);
    }
}
