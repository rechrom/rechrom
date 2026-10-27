//! LayoutObject paint, outline, and diagnostic geometry dispatch.

use super::*;

impl LayoutObject {
    // These ancestry predicates are called recursively. Keep them physically
    // paint-owned but inlineable so a deep canvas subtree does not produce one
    // forced frame for every ancestor.
    // cpp: third_party/blink/renderer/core/layout/layout_object.cc:5738-5743
    pub fn IsCanvasOrInCanvasSubtree(&self) -> bool {
        self.IsCanvas() || self.IsInCanvasSubtree()
    }

    // cpp: third_party/blink/renderer/core/layout/layout_object.cc:5745-5756
    pub fn IsInCanvasSubtree(&self) -> bool {
        let parent = self.Parent();
        !parent.is_null() && unsafe { &*parent }.IsCanvasOrInCanvasSubtree()
    }

    // cpp: layoutng/internal/layout_object.h:2696-2698
    #[inline(never)]
    pub fn CollectOutlineRectsAndAdvance(
        &self,
        outline_type: OutlineType,
        iterator: &mut AccompaniedFragmentIterator,
    ) -> Vector<PhysicalRect> {
        unsafe { LayoutObjectCollectOutlineRectsAndAdvanceProvider(self, outline_type, iterator) }
    }

    // cpp: layoutng/internal/layout_object.h:2704-2706
    #[inline(never)]
    pub fn OutlineRects(
        &self,
        info: *mut LayoutOutlineInfo,
        additional_offset: &PhysicalOffset,
        outline_type: OutlineType,
    ) -> Vector<PhysicalRect> {
        unsafe { LayoutObjectOutlineRectsProvider(self, info, additional_offset, outline_type) }
    }

    // Paint is invoked for many individual objects. Physical ownership is
    // useful, but the tiny dispatch wrapper stays inlineable to avoid a forced
    // frame on every painted object.
    // cpp: layoutng/internal/layout_object.h:2102-2102
    pub fn Paint(&self, info: &PaintInfo) {
        if self.IsSVGHiddenContainer() {
            return;
        }
        unsafe { LayoutObjectPaintProvider(self, info) }
    }

    // cpp: layoutng/internal/layout_object.h:2157-2157
    pub fn AddDraggableRegions(&mut self, regions: &mut Vector<DraggableRegionValue>) {
        unsafe { LayoutObjectAddDraggableRegionsProvider(self, regions) }
    }

    // cpp: layoutng/internal/layout_object.h:2160-2160
    pub fn CanHaveAdditionalCompositingReasons(&self) -> bool {
        unsafe { LayoutObjectCanHaveAdditionalCompositingReasonsProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2161-2161
    pub fn AdditionalCompositingReasons(&self) -> CompositingReasons {
        unsafe { LayoutObjectAdditionalCompositingReasonsProvider(self) }
    }

    // Resolving the drawing canvas may walk out of the current object. Keep a
    // paint-domain frame so ancestor lookup cost does not accrue to core.
    // cpp: layoutng/internal/layout_object.h:2674-2674
    #[inline(never)]
    pub fn CanvasForDrawingLayoutObject(&self) -> *mut LayoutObject {
        unsafe { LayoutObjectCanvasForDrawingLayoutObjectProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2712-2717
    pub fn AddOutlineRectsBase(
        &self,
        _collector: &mut dyn OutlineRectCollector,
        _info: *mut LayoutOutlineInfo,
        _additional_offset: &PhysicalOffset,
        _outline_type: OutlineType,
    ) {
        self.CheckIsNotDestroyed();
    }

    // Outline collection may enumerate multiple retained fragments, so keep a
    // single package-level sampling frame around the virtual dispatch.
    // cpp: layoutng/internal/layout_object.h:2712-2717
    #[inline(never)]
    pub fn AddOutlineRects(
        &self,
        collector: &mut dyn OutlineRectCollector,
        info: *mut LayoutOutlineInfo,
        additional_offset: &PhysicalOffset,
        outline_type: OutlineType,
    ) {
        self.CheckIsNotDestroyed();
        unsafe {
            DispatchLayoutObjectAddOutlineRects(
                self,
                collector,
                info,
                additional_offset,
                outline_type,
            )
        }
    }

    // cpp: layoutng/internal/layout_object.h:2861-2861
    pub fn DebugRect(&self) -> PhysicalRect {
        unsafe { LayoutObjectDebugRectProvider(self) }
    }
}
