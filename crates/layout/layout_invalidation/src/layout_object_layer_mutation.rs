//! LayoutObject paint-layer tree mutation and insertion-point search.

use super::*;

impl LayoutObject {
    // cpp: layoutng/internal/layout_object.h:449-449
    #[inline(never)]
    pub fn AddLayers(&mut self, parent_layer: *mut PaintLayer) {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectAddLayersProvider(self, parent_layer) }
    }

    // cpp: layoutng/internal/layout_object.h:450-450
    #[inline(never)]
    pub fn RemoveLayers(&mut self, parent_layer: *mut PaintLayer) {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectRemoveLayersProvider(self, parent_layer) }
    }

    // cpp: layoutng/internal/layout_object.h:451-451
    #[inline(never)]
    pub fn MoveLayers(&mut self, old_parent: *mut PaintLayer, new_parent: *mut PaintLayer) {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectMoveLayersProvider(self, old_parent, new_parent) }
    }

    // cpp: layoutng/internal/layout_object.h:452-454
    #[inline(never)]
    pub fn FindNextLayer(
        &mut self,
        parent_layer: *mut PaintLayer,
        start_point: *mut LayoutObject,
        check_parent: bool,
    ) -> *mut PaintLayer {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectFindNextLayerProvider(self, parent_layer, start_point, check_parent) }
    }

    // cpp: layoutng/internal/layout_object.h:452-454
    pub fn FindNextLayerDefault(
        &mut self,
        parent_layer: *mut PaintLayer,
        start_point: *mut LayoutObject,
    ) -> *mut PaintLayer {
        self.FindNextLayer(parent_layer, start_point, true)
    }
}
