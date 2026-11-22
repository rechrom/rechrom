#![allow(non_snake_case)]

use foundation::{Member, Visitor};

use super::super::layout_box::LayoutBox;

// cpp: layoutng/internal/unpositioned_list_marker.h:50-51,102-109
// The C++ class is stack-only. Rust callers construct it by value; its single
// traced member retains the original data layout and ownership boundary.
//
// The constructor and the layout members (ContentAlignmentBaseline, AddToBox,
// AddToBoxWithoutLineBoxes, InlineOffset, Layout, CheckMargin,
// ComputeIntrudedFloatOffset) are Chromium's block-layout code
// (layoutng_block/unpositioned_list_marker.cc) and only the block algorithm
// calls them: they are `layout_algo_block::unpositioned_list_marker::
// UnpositionedListMarkerAlgorithms`, an extension trait on this type.
#[repr(C)]
pub struct UnpositionedListMarker {
    marker_layout_object_: Member<LayoutBox>,
}

// The source's implicit copy constructor copies its single Member handle.
impl Clone for UnpositionedListMarker {
    fn clone(&self) -> Self {
        Self {
            marker_layout_object_: self.marker_layout_object_.clone(),
        }
    }
}

impl Default for UnpositionedListMarker {
    // cpp: layoutng/internal/unpositioned_list_marker.h:54
    fn default() -> Self {
        Self {
            marker_layout_object_: Member::from_ptr(std::ptr::null_mut()),
        }
    }
}

impl PartialEq for UnpositionedListMarker {
    // cpp: layoutng/internal/unpositioned_list_marker.h:88-90
    fn eq(&self, other: &Self) -> bool {
        self.marker_layout_object_.Get() == other.marker_layout_object_.Get()
    }
}

impl Eq for UnpositionedListMarker {}

#[allow(non_snake_case)]
impl UnpositionedListMarker {
    // cpp: layoutng/internal/unpositioned_list_marker.h:55
    // Member initialisation of the block package's constructor
    // (layoutng_block/unpositioned_list_marker.cc:20-23), which resolves the
    // marker box and passes it here.
    pub fn FromMarkerLayoutObject(marker: *mut LayoutBox) -> Self {
        Self {
            marker_layout_object_: Member::from_ptr(marker),
        }
    }

    // cpp: layoutng/internal/unpositioned_list_marker.h:57
    pub fn is_non_null(&self) -> bool {
        !self.marker_layout_object_.Get().is_null()
    }

    pub fn is_present(&self) -> bool {
        self.is_non_null()
    }

    // Cross-crate access to the private C++ member for the block package's
    // method bodies.
    pub fn MarkerLayoutObject(&self) -> *mut LayoutBox {
        self.marker_layout_object_.Get()
    }

    // Keep the GC visit on the actual member field.
    pub fn TraceMarkerMember(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.marker_layout_object_);
    }

    // cpp: layoutng_block/unpositioned_list_marker.cc:216-218
    // A record member: base's trace bindings call it, so it stays with the type.
    pub fn Trace(&self, visitor: &mut Visitor) {
        self.TraceMarkerMember(visitor);
    }
}
