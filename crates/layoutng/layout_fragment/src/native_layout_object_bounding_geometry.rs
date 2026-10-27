//! LayoutObject quad enumeration and absolute bounding geometry.

use super::*;

impl LayoutObject {
    // cpp: layoutng/internal/layout_object.h:786-786
    #[inline(never)]
    pub fn AddAbsoluteRectForLayer(&mut self, result: &mut gfx::Rect) {
        unsafe { LayoutObjectAddAbsoluteRectForLayerProvider(self, result) }
    }

    // cpp: layoutng/internal/layout_object.h:2438-2453,3470-3472
    pub fn QuadsInAncestorInternal(
        &self,
        _quads: &mut Vector<gfx::QuadF>,
        _ancestor: *const LayoutBoxModelObject,
        _mode: MapCoordinatesFlags,
        _box_type: BoxQuadType,
    ) {
        self.CheckIsNotDestroyed();
    }

    pub fn QuadsInAncestor(
        &self,
        quads: &mut Vector<gfx::QuadF>,
        ancestor: *const LayoutBoxModelObject,
        mode: MapCoordinatesFlags,
        box_type: BoxQuadType,
    ) {
        self.CheckIsNotDestroyed();
        self.QuadsInAncestorInternal(quads, ancestor, mode, box_type);
    }

    pub fn AbsoluteQuads(
        &self,
        quads: &mut Vector<gfx::QuadF>,
        mode: MapCoordinatesFlags,
        box_type: BoxQuadType,
    ) {
        self.CheckIsNotDestroyed();
        self.QuadsInAncestor(quads, std::ptr::null(), mode, box_type);
    }

    // cpp: layoutng/internal/layout_object.h:2412-2412
    #[inline(never)]
    pub fn AbsoluteBoundingBoxRectF(&self, mode: MapCoordinatesFlags) -> gfx::RectF {
        unsafe { LayoutObjectAbsoluteBoundingBoxRectFProvider(self, mode) }
    }

    pub fn AbsoluteBoundingBoxRectFDefault(&self) -> gfx::RectF {
        self.AbsoluteBoundingBoxRectF(MapCoordinatesFlags::default())
    }

    // cpp: layoutng/internal/layout_object.h:2416-2416
    #[inline(never)]
    pub fn AbsoluteBoundingBoxRect(&self, mode: MapCoordinatesFlags) -> gfx::Rect {
        unsafe { LayoutObjectAbsoluteBoundingBoxRectProvider(self, mode) }
    }

    pub fn AbsoluteBoundingBoxRectDefault(&self) -> gfx::Rect {
        self.AbsoluteBoundingBoxRect(MapCoordinatesFlags::default())
    }

    // cpp: layoutng/internal/layout_object.h:2422-2422
    #[inline(never)]
    pub fn AbsoluteBoundingBoxRectForUnboundedElement(&self) -> gfx::Rect {
        unsafe { LayoutObjectAbsoluteBoundingBoxRectForUnboundedElementProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2430-2431
    #[inline(never)]
    pub fn AbsoluteBoundingBoxRectHandlingEmptyInline(
        &self,
        flags: MapCoordinatesFlags,
    ) -> PhysicalRect {
        unsafe { LayoutObjectAbsoluteBoundingBoxRectHandlingEmptyInlineProvider(self, flags) }
    }

    pub fn AbsoluteBoundingBoxRectHandlingEmptyInlineDefault(&self) -> PhysicalRect {
        self.AbsoluteBoundingBoxRectHandlingEmptyInline(MapCoordinatesFlags::default())
    }

    // cpp: layoutng/internal/layout_object.h:2434-2434
    #[inline(never)]
    pub fn AbsoluteBoundingBoxRectForScrollIntoView(&self) -> PhysicalRect {
        unsafe { LayoutObjectAbsoluteBoundingBoxRectForScrollIntoViewProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2456-2456
    #[inline(never)]
    pub fn AbsoluteBoundingBoxRectIncludingDescendants(&self) -> gfx::Rect {
        unsafe { LayoutObjectAbsoluteBoundingBoxRectIncludingDescendantsProvider(self) }
    }
}
