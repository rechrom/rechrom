#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use foundation::{Traceable, Visitor};
use layoutng_assembly::internal::layout_inline::LayoutInline;
use layoutng_assembly::internal::layout_node_metadata::Element;
use layoutng_assembly::internal::layout_object::{LayoutObject, LayoutObjectClass};

use crate::list_marker::ListMarker;

// cpp: layoutng_list/layout_inside_list_marker.h:15-49
#[repr(C)]
pub struct LayoutInsideListMarker {
    inline_: LayoutInline,
    list_marker_: ListMarker,
}

impl LayoutInsideListMarker {
    // cpp: layoutng_list/layout_inside_list_marker.h:18-18
    // cpp: layoutng_list/layout_inside_list_marker.cc:9-10
    pub fn new(element: *mut Element) -> Self {
        let inline_ = LayoutInline::new(element);
        inline_.SetRuntimeClass(LayoutObjectClass::InsideListMarker);
        Self {
            inline_,
            list_marker_: ListMarker::default(),
        }
    }

    // cpp: layoutng_list/layout_inside_list_marker.h:20-23
    pub fn GetName(&self) -> &'static str {
        "LayoutInsideListMarker"
    }

    // cpp: layoutng_list/layout_inside_list_marker.h:25-32
    pub fn Marker(&self) -> &ListMarker {
        &self.list_marker_
    }
    pub fn MarkerMut(&mut self) -> &mut ListMarker {
        &mut self.list_marker_
    }

    // cpp: layoutng_list/layout_inside_list_marker.h:34-41
    #[cfg(debug_assertions)]
    pub fn AddChild(&mut self, new_child: *mut LayoutObject, before_child: *mut LayoutObject) {
        debug_assert!(!self.StyleRef().ContentBehavesAsNormal() || self.FirstChild().is_null());
        self.inline_.AddChild(new_child, before_child);
    }
}

impl Deref for LayoutInsideListMarker {
    type Target = LayoutInline;
    fn deref(&self) -> &Self::Target {
        &self.inline_
    }
}
impl DerefMut for LayoutInsideListMarker {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inline_
    }
}
impl Traceable for LayoutInsideListMarker {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.inline_.Trace(visitor);
    }
}
