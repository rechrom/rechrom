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

    #[inline(never)]
    pub fn CollectOutlineRectsAndAdvance(
        &self,
        outline_type: OutlineType,
        iterator: &mut AccompaniedFragmentIterator,
    ) -> Vector<PhysicalRect> {
        unsafe { LayoutObjectCollectOutlineRectsAndAdvanceProvider(self, outline_type, iterator) }
    }

    #[inline(never)]
    pub fn OutlineRects(
        &self,
        info: *mut LayoutOutlineInfo,
        additional_offset: &PhysicalOffset,
        outline_type: OutlineType,
    ) -> Vector<PhysicalRect> {
        self.CheckIsNotDestroyed();
        let mut collector =
            crate::internal::outline_rect_collector::VectorOutlineRectCollector::default();
        self.AddOutlineRects(&mut collector, info, additional_offset, outline_type);
        collector.TakeRects()
    }

    // Paint is invoked for many individual objects. Physical ownership is
    // useful, but the tiny dispatch wrapper stays inlineable to avoid a forced
    // frame on every painted object.
    pub fn Paint(&self, info: &PaintInfo) {
        if self.IsSVGHiddenContainer() {
            return;
        }
        unsafe { LayoutObjectPaintProvider(self, info) }
    }

    pub fn AddDraggableRegions(&mut self, regions: &mut Vector<DraggableRegionValue>) {
        unsafe { LayoutObjectAddDraggableRegionsProvider(self, regions) }
    }

    pub fn CanHaveAdditionalCompositingReasonsBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn CanHaveAdditionalCompositingReasons(&self) -> bool {
        unsafe { LayoutObjectCanHaveAdditionalCompositingReasonsProvider(self) }
    }

    pub fn AdditionalCompositingReasonsBase(&self) -> CompositingReasons {
        self.CheckIsNotDestroyed();
        CompositingReasons::default()
    }

    pub fn AdditionalCompositingReasons(&self) -> CompositingReasons {
        unsafe { LayoutObjectAdditionalCompositingReasonsProvider(self) }
    }

    // Resolving the drawing canvas may walk out of the current object. Keep a
    // paint-domain frame so ancestor lookup cost does not accrue to core.
    // cpp: third_party/blink/renderer/core/layout/layout_object.cc:3792-3803
    #[inline(never)]
    pub fn CanvasForDrawingLayoutObject(&self) -> *mut LayoutObject {
        self.CanvasForDrawingLayoutObjectInfo()
            .unwrap_or(std::ptr::null_mut())
    }
    /// Some(nullptr) retains an eligible canvas with no LayoutObject (for
    /// example display:contents), distinct from no CanvasForDrawing owner.
    pub fn CanvasForDrawingLayoutObjectInfo(&self) -> Option<*mut LayoutObject> {
        self.CheckIsNotDestroyed();
        if !self.IsBoxModelObject() {
            return None;
        }
        let node = unsafe { self.GetNode().as_ref()? };
        if !node.InputCanvasDrawElementEnabled() || node.InputIsStyleGenerated() {
            return None;
        }
        let mut ancestor = node.parentNode().cast::<Node>();
        let parent = unsafe { ancestor.as_ref()? };
        if parent.InputIsHTMLCanvasElement() {
            return parent
                .InputCanvasLayoutSubtree()
                .then(|| parent.GetLayoutObject());
        }
        if !node.InputHasDrawableAttribute() {
            return None;
        }
        while let Some(parent) = unsafe { ancestor.as_ref() } {
            if parent.InputIsHTMLCanvasElement() {
                return parent
                    .InputCanvasLayoutSubtree()
                    .then(|| parent.GetLayoutObject());
            }
            ancestor = parent.parentNode().cast::<Node>();
        }
        None
    }

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

    pub fn DebugRect(&self) -> PhysicalRect {
        unsafe { LayoutObjectDebugRectProvider(self) }
    }
}
