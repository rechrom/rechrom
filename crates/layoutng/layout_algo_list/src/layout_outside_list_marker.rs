#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use foundation::{DynamicTo, Traceable, Visitor};
use layoutng_assembly::internal::layout_block_flow::LayoutBlockFlow;
use layoutng_assembly::internal::layout_node_metadata::Element;
use layoutng_assembly::internal::layout_object::{LayoutObject, LayoutObjectClass};

use crate::list_marker::ListMarker;

// cpp: layoutng_list/layout_outside_list_marker.h:14-45
#[repr(C)]
pub struct LayoutOutsideListMarker {
    flow: LayoutBlockFlow,
    list_marker_: ListMarker,
}

impl LayoutOutsideListMarker {
    // cpp: layoutng_list/layout_outside_list_marker.h:17-17
    // cpp: layoutng_list/layout_outside_list_marker.cc:12-13
    pub fn new(element: *mut Element) -> Self {
        let flow = LayoutBlockFlow::new(element.cast());
        flow.SetRuntimeClass(LayoutObjectClass::OutsideListMarker);
        Self {
            flow,
            list_marker_: ListMarker::default(),
        }
    }

    // cpp: layoutng_list/layout_outside_list_marker.h:19-19
    // cpp: layoutng_list/layout_outside_list_marker.cc:15-17
    pub fn WillCollectInlines(&mut self) {
        let marker = &mut self.list_marker_ as *mut ListMarker;
        let object = self as *mut Self as *mut LayoutObject;
        unsafe { &mut *marker }.UpdateMarkerTextIfNeeded(unsafe { &mut *object });
    }

    // cpp: layoutng_list/layout_outside_list_marker.h:21-24
    pub fn GetName(&self) -> &'static str {
        "LayoutOutsideListMarker"
    }

    // cpp: layoutng_list/layout_outside_list_marker.h:28-35
    pub fn Marker(&self) -> &ListMarker {
        &self.list_marker_
    }
    pub fn MarkerMut(&mut self) -> &mut ListMarker {
        &mut self.list_marker_
    }

    // cpp: layoutng_list/layout_outside_list_marker.h:37-37
    // cpp: layoutng_list/layout_outside_list_marker.cc:19-21
    pub fn IsMonolithic(&self) -> bool {
        true
    }

    // cpp: layoutng_list/layout_outside_list_marker.h:26-26
    // cpp: layoutng_list/layout_outside_list_marker.cc:23-39
    pub fn NeedsOccupyWholeLine(&self) -> bool {
        if !self.InQuirksModeForLayout() {
            return false;
        }
        let next_sibling = self.NextSibling();
        if !next_sibling.is_null() {
            let next = unsafe { &*next_sibling };
            if !next.IsInline()
                && !next.IsFloatingOrOutOfFlowPositioned()
                && !next.GetNode().is_null()
            {
                let element = DynamicTo::<Element>(next.GetNode());
                if !element.is_null() {
                    if unsafe { &*element }
                        .InputElementData()
                        .as_ref()
                        .is_some_and(|data| data.html_ordered_or_unordered_list)
                    {
                        return true;
                    }
                }
            }
        }
        false
    }
}

impl Deref for LayoutOutsideListMarker {
    type Target = LayoutBlockFlow;
    fn deref(&self) -> &Self::Target {
        &self.flow
    }
}
impl DerefMut for LayoutOutsideListMarker {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.flow
    }
}
impl Traceable for LayoutOutsideListMarker {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.flow.Trace(visitor);
    }
}
