#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use foundation::{To, Traceable, Visitor};
use layoutng_assembly::internal::layout_inline::LayoutInline;
use layoutng_assembly::internal::layout_node_metadata::Element;
use layoutng_assembly::internal::layout_object::{
    LayoutObject, LayoutObjectClass, StyleChangeContext,
};
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::style_difference::StyleDifference;

use crate::list_item_ordinal::ListItemOrdinal;
use crate::list_marker::ListMarker;

// cpp: layoutng_list/layout_inline_list_item.h:13-44
#[repr(C)]
pub struct LayoutInlineListItem {
    inline_: LayoutInline,
    ordinal_: ListItemOrdinal,
}

impl LayoutInlineListItem {
    // cpp: layoutng_list/layout_inline_list_item.h:16-16
    // cpp: layoutng_list/layout_inline_list_item.cc:13-18
    pub fn new(element: *mut Element) -> Self {
        let mut inline_ = LayoutInline::new(element);
        inline_.SetRuntimeClass(LayoutObjectClass::InlineListItem);
        inline_.SetConsumesSubtreeChangeNotification();
        inline_.RegisterSubtreeChangeListenerOnDescendants(true);
        let view = inline_.View();
        unsafe { &mut *view }.AddLayoutListItem();
        Self {
            inline_,
            ordinal_: ListItemOrdinal::new(),
        }
    }

    // cpp: layoutng_list/layout_inline_list_item.h:18-21
    pub fn Ordinal(&mut self) -> &mut ListItemOrdinal {
        &mut self.ordinal_
    }

    // cpp: layoutng_list/layout_inline_list_item.h:29-29
    // cpp: layoutng_list/layout_inline_list_item.cc:20-26
    pub fn WillBeDestroyed(&mut self) {
        let view = self.View();
        if !view.is_null() {
            unsafe { &mut *view }.RemoveLayoutListItem();
        }
        self.inline_.WillBeDestroyed();
    }

    // cpp: layoutng_list/layout_inline_list_item.h:30-30
    // cpp: layoutng_list/layout_inline_list_item.cc:28-31
    pub fn GetName(&self) -> &'static str {
        "LayoutInlineListItem"
    }

    // cpp: layoutng_list/layout_inline_list_item.h:35-35
    // cpp: layoutng_list/layout_inline_list_item.cc:33-36
    pub fn InsertedIntoTree(&mut self) {
        self.inline_.InsertedIntoTree();
        ListItemOrdinal::ItemInsertedOrRemoved(self as *const Self as *const LayoutObject);
    }

    // cpp: layoutng_list/layout_inline_list_item.h:36-36
    // cpp: layoutng_list/layout_inline_list_item.cc:38-41
    pub fn WillBeRemovedFromTree(&mut self) {
        self.inline_.WillBeRemovedFromTree();
        ListItemOrdinal::ItemInsertedOrRemoved(self as *const Self as *const LayoutObject);
    }

    // cpp: layoutng_list/layout_inline_list_item.h:25-25
    // cpp: layoutng_list/layout_inline_list_item.cc:43-46
    pub fn Marker(&self) -> *mut LayoutObject {
        let element = To::<Element>(self.GetNode());
        unsafe { &*element }.InputListMarkerLayoutObject()
    }

    // cpp: layoutng_list/layout_inline_list_item.h:26-26
    // cpp: layoutng_list/layout_inline_list_item.cc:48-53
    pub fn UpdateMarkerTextIfNeeded(&mut self) {
        let marker = self.Marker();
        let list_marker = ListMarker::GetMut(marker);
        if !list_marker.is_null() {
            unsafe { &mut *list_marker }.UpdateMarkerTextIfNeeded(unsafe { &mut *marker });
        }
    }

    // cpp: layoutng_list/layout_inline_list_item.h:37-40
    // cpp: layoutng_list/layout_inline_list_item.cc:55-80
    pub fn StyleDidChange(
        &mut self,
        diff: StyleDifference,
        old_style: *const ComputedStyle,
        new_style: &ComputedStyle,
        style_change_context: &StyleChangeContext,
    ) {
        self.inline_
            .StyleDidChange(diff, old_style, new_style, style_change_context);
        let marker = self.Marker();
        let list_marker = ListMarker::GetMut(marker);
        if list_marker.is_null() {
            return;
        }
        unsafe { &mut *list_marker }.UpdateMarkerContentIfNeeded(unsafe { &mut *marker });
        if !old_style.is_null() {
            let old_list_style_type = unsafe { &*old_style }.ListStyleType().Get();
            let new_list_style_type = new_style.ListStyleType().Get();
            if old_list_style_type != new_list_style_type
                && (old_list_style_type.is_null()
                    || new_list_style_type.is_null()
                    || unsafe { &*old_list_style_type } != unsafe { &*new_list_style_type })
            {
                unsafe { &mut *list_marker }.ListStyleTypeChanged(unsafe { &mut *marker });
                self.SetNeedsCollectInlines();
            }
        }
    }

    // cpp: layoutng_list/layout_inline_list_item.h:22-22
    // cpp: layoutng_list/layout_inline_list_item.cc:82-85
    pub fn Value(&self) -> i32 {
        let node = self.GetNode();
        debug_assert!(!node.is_null());
        self.ordinal_.Value(unsafe { &*node })
    }

    // cpp: layoutng_list/layout_inline_list_item.h:23-23
    // cpp: layoutng_list/layout_inline_list_item.cc:87-94
    pub fn OrdinalValueChanged(&mut self) {
        let marker = self.Marker();
        let list_marker = ListMarker::GetMut(marker);
        if !list_marker.is_null() {
            unsafe { &mut *list_marker }.OrdinalValueChanged(unsafe { &mut *marker });
            unsafe { &mut *marker }.SetNeedsCollectInlines();
        }
    }

    // cpp: layoutng_list/layout_inline_list_item.h:41-41
    // cpp: layoutng_list/layout_inline_list_item.cc:96-104
    pub fn SubtreeDidChange(&mut self) {
        let marker = self.Marker();
        let list_marker = ListMarker::GetMut(marker);
        if list_marker.is_null() {
            return;
        }
        debug_assert!(unsafe { &*marker }.IsLayoutInsideListMarker());
        unsafe { &mut *list_marker }.UpdateMarkerContentIfNeeded(unsafe { &mut *marker });
    }
}

impl Deref for LayoutInlineListItem {
    type Target = LayoutInline;
    fn deref(&self) -> &Self::Target {
        &self.inline_
    }
}
impl DerefMut for LayoutInlineListItem {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inline_
    }
}
impl Traceable for LayoutInlineListItem {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.inline_.Trace(visitor);
    }
}
