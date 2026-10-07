#![allow(non_snake_case)]

//! Native PaintLayer identity and ownership. Compositing and paint-property
//! construction are owned by the paint package; this source subset supplies
//! the persistent DisplayItemClient used by PaintLayerPainter chunk scopes.

use foundation::graphics_types::graphics::paint::display_item_client::DisplayItemClient;
use foundation::{Member, Traceable, Visitor};

use super::layout_box_model_object::{LayoutBoxModelObject, PaintLayerType};

// The GarbageCollected base has no data. Preserve the DisplayItemClient base
// as a real subobject, independently of the owning LayoutObject's client.
// cpp: core/paint/paint_layer.h:168-171,688-738
#[repr(C)]
pub struct PaintLayer {
    display_item_client_: DisplayItemClient,
    layout_object_: Member<LayoutBoxModelObject>,
    is_self_painting_layer_: bool,
    is_destroyed_: bool,
}

impl PaintLayer {
    // cpp: core/paint/paint_layer.cc:178-217
    pub fn new(layout_object: *mut LayoutBoxModelObject) -> Self {
        assert!(!layout_object.is_null());
        Self {
            display_item_client_: DisplayItemClient::default(),
            layout_object_: Member::from_ptr(layout_object),
            is_self_painting_layer_: unsafe { &*layout_object }.LayerTypeRequired()
                == PaintLayerType::kNormalPaintLayer,
            is_destroyed_: false,
        }
    }

    // cpp: core/paint/paint_layer.h:185-190
    pub fn GetLayoutObject(&self) -> &LayoutBoxModelObject {
        debug_assert!(!self.is_destroyed_);
        unsafe { &*self.layout_object_.Get() }
    }

    pub fn DisplayItemClient(&self) -> &DisplayItemClient {
        &self.display_item_client_
    }

    // cpp: platform/graphics/paint/display_item_client.h:33-35,71-82
    pub fn Id(&self) -> u64 {
        self.display_item_client_.Id() as u64
    }
    pub fn IsCacheable(&self) -> bool {
        self.display_item_client_.IsCacheable()
    }
    pub fn IsJustCreated(&self) -> bool {
        self.display_item_client_.IsJustCreated()
    }

    // cpp: core/paint/paint_layer.h:210
    pub fn IsSelfPaintingLayer(&self) -> bool {
        debug_assert!(!self.is_destroyed_);
        self.is_self_painting_layer_
    }

    // cpp: core/paint/paint_layer.cc:2298-2307
    pub fn UpdateSelfPaintingLayer(&mut self) {
        self.is_self_painting_layer_ =
            self.GetLayoutObject().LayerTypeRequired() == PaintLayerType::kNormalPaintLayer;
    }

    // Resource subscriptions/compositor trees are absent from this native
    // subset. Their owners remain outside this identity/lifetime translation.
    // cpp: core/paint/paint_layer.cc:225-257
    pub fn Destroy(&mut self) {
        debug_assert!(!self.is_destroyed_);
        self.is_destroyed_ = true;
    }
}

// cpp: core/paint/paint_layer.cc:2881-2892
impl Traceable for PaintLayer {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.layout_object_);
        self.display_item_client_.Trace(visitor);
    }
}
