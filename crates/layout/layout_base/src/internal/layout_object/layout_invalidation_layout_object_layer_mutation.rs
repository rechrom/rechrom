//! LayoutObject paint-layer tree mutation and insertion-point search.
//!
//! Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
//! Source: third_party/blink/renderer/core/layout/layout_object.cc:1158-1264
//!
//! Rechrom derives PaintLayer parent/child/sibling links from the
//! authoritative LayoutObject tree (PaintLayer::Parent/FirstChild), so these
//! walks keep Chromium's call structure and drive PaintLayer::AddChild /
//! RemoveChild for their invalidation (z-order lists, descendant-dependent
//! flags, repaint and cull-rect bits) instead of splicing a stored list.

use super::*;

fn LayerOf(object: *mut LayoutObject) -> *mut PaintLayer {
    debug_assert!(unsafe { &*object }.HasLayer());
    let box_model = foundation::DynamicTo::<LayoutBoxModelObject>(object);
    debug_assert!(!box_model.is_null());
    unsafe { &*box_model }.Layer()
}

// Source: core/layout/layout_object.cc:1158-1179. The source also computes
// the `before_child` sibling through FindNextLayer() once per call so
// AddChild() can splice the stored sibling list; the derived links already
// place the layer at its LayoutObject tree position, so only the AddChild()
// invalidation remains.
fn AddLayersRecursive(object: *mut LayoutObject, parent_layer: *mut PaintLayer) {
    let object_ref = unsafe { &*object };
    if object_ref.HasLayer() {
        unsafe { &mut *parent_layer }.AddChild(LayerOf(object));
        return;
    }

    let mut curr = object_ref.SlowFirstChild();
    while !curr.is_null() {
        AddLayersRecursive(curr, parent_layer);
        curr = unsafe { &*curr }.NextSibling();
    }
}

impl LayoutObject {
    // Source: core/layout/layout_object.cc:1181-1189.
    #[inline(never)]
    pub fn AddLayers(&mut self, parent_layer: *mut PaintLayer) {
        self.CheckIsNotDestroyed();
        if parent_layer.is_null() {
            return;
        }
        AddLayersRecursive(self, parent_layer);
    }

    // Source: core/layout/layout_object.cc:1191-1203. Called before the
    // LayoutObject is unlinked, while the derived layer links still name
    // `parent_layer` as the parent.
    #[inline(never)]
    pub fn RemoveLayers(&mut self, parent_layer: *mut PaintLayer) {
        self.CheckIsNotDestroyed();
        if parent_layer.is_null() {
            return;
        }

        if self.HasLayer() {
            unsafe { &mut *parent_layer }.RemoveChild(LayerOf(self));
            return;
        }

        let mut curr = self.SlowFirstChild();
        while !curr.is_null() {
            unsafe { &mut *curr }.RemoveLayers(parent_layer);
            curr = unsafe { &*curr }.NextSibling();
        }
    }

    // Source: core/layout/layout_object.cc:1205-1221. The source DCHECKs
    // `old_parent == layer->Parent()` before relinking; here the derived link
    // already names `new_parent` (a layer was just created between them), so
    // RemoveChild(old_parent) / AddChild(new_parent) carry the invalidation of
    // both sides of the move.
    #[inline(never)]
    pub fn MoveLayers(&mut self, old_parent: *mut PaintLayer, new_parent: *mut PaintLayer) {
        self.CheckIsNotDestroyed();
        if new_parent.is_null() {
            return;
        }

        if self.HasLayer() {
            let layer = LayerOf(self);
            debug_assert_eq!(unsafe { &*layer }.Parent(), new_parent);
            if !old_parent.is_null() {
                unsafe { &mut *old_parent }.RemoveChild(layer);
            }
            unsafe { &mut *new_parent }.AddChild(layer);
            return;
        }

        let mut curr = self.SlowFirstChild();
        while !curr.is_null() {
            unsafe { &mut *curr }.MoveLayers(old_parent, new_parent);
            curr = unsafe { &*curr }.NextSibling();
        }
    }

    // Source: core/layout/layout_object.cc:1223-1264.
    #[inline(never)]
    pub fn FindNextLayer(
        &mut self,
        parent_layer: *mut PaintLayer,
        start_point: *mut LayoutObject,
        check_parent: bool,
    ) -> *mut PaintLayer {
        self.CheckIsNotDestroyed();
        // Error check the parent layer passed in. If it's null, we can't find
        // anything.
        if parent_layer.is_null() {
            return std::ptr::null_mut();
        }

        // Step 1: If our layer is a child of the desired parent, then return
        // our layer.
        let our_layer = if self.HasLayer() {
            LayerOf(self)
        } else {
            std::ptr::null_mut()
        };
        if !our_layer.is_null() && unsafe { &*our_layer }.Parent() == parent_layer {
            return our_layer;
        }

        // Step 2: If we don't have a layer, or our layer is the desired
        // parent, then descend into our siblings trying to find the next layer
        // whose parent is the desired parent.
        if our_layer.is_null() || our_layer == parent_layer {
            let mut curr = if start_point.is_null() {
                self.SlowFirstChild()
            } else {
                unsafe { &*start_point }.NextSibling()
            };
            while !curr.is_null() {
                let next_layer =
                    unsafe { &mut *curr }.FindNextLayer(parent_layer, std::ptr::null_mut(), false);
                if !next_layer.is_null() {
                    return next_layer;
                }
                curr = unsafe { &*curr }.NextSibling();
            }
        }

        // Step 3: If our layer is the desired parent layer, then we're
        // finished. We didn't find anything.
        if parent_layer == our_layer {
            return std::ptr::null_mut();
        }

        // Step 4: If |checkParent| is set, climb up to our parent and check
        // its siblings that follow us to see if we can locate a layer.
        let this = self as *mut LayoutObject;
        let parent = self.Parent();
        if check_parent && !parent.is_null() {
            return unsafe { &mut *parent }.FindNextLayer(parent_layer, this, true);
        }

        std::ptr::null_mut()
    }

    // Source: core/layout/layout_object.h:452-454 (check_parent = true).
    pub fn FindNextLayerDefault(
        &mut self,
        parent_layer: *mut PaintLayer,
        start_point: *mut LayoutObject,
    ) -> *mut PaintLayer {
        self.FindNextLayer(parent_layer, start_point, true)
    }
}
