//! LayoutObject quad enumeration and absolute bounding geometry.

use super::*;
use crate::internal::layout_inline::LayoutInline;
use crate::internal::map_coordinates_flags::MapCoordinatesMode;

impl LayoutObject {
    #[inline(never)]
    pub fn AddAbsoluteRectForLayer(&mut self, result: &mut gfx::Rect) {
        // Source: core/layout/layout_object.cc:2225-2234.
        self.CheckIsNotDestroyed();
        if self.HasLayer() {
            result.Union(self.AbsoluteBoundingBoxRectDefault());
        }
        let mut child = self.SlowFirstChild();
        while let Some(current) = unsafe { child.as_mut() } {
            current.AddAbsoluteRectForLayer(result);
            child = current.NextSibling();
        }
    }

    // Source: core/layout/layout_object.h:3628-3634.
    pub fn QuadsInAncestorInternalBase(
        &self,
        _quads: &mut Vector<gfx::QuadF>,
        _ancestor: *const LayoutBoxModelObject,
        _mode: MapCoordinatesFlags,
        _box_type: BoxQuadType,
    ) {
        self.CheckIsNotDestroyed();
    }

    // Preserve the source virtual call before selecting the concrete native
    // fragment owner; the base implementation intentionally emits no quad.
    pub fn QuadsInAncestorInternal(
        &self,
        quads: &mut Vector<gfx::QuadF>,
        ancestor: *const LayoutBoxModelObject,
        mode: MapCoordinatesFlags,
        box_type: BoxQuadType,
    ) {
        unsafe {
            DispatchLayoutObjectQuadsInAncestorInternal(self, quads, ancestor, mode, box_type)
        }
    }

    // Source: core/layout/layout_object.h:2496-2510.
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

    #[inline(never)]
    pub fn AbsoluteBoundingBoxRectF(&self, mode: MapCoordinatesFlags) -> gfx::RectF {
        // Source: core/layout/layout_object.cc:2123-2141.
        self.CheckIsNotDestroyed();
        debug_assert!(!mode.contains(MapCoordinatesMode::kIgnoreTransforms));
        let mut quads = Vector::new();
        self.AbsoluteQuads(&mut quads, mode, BoxQuadType::kBorder);
        let mut quads = quads.iter();
        let Some(first) = quads.next() else {
            return gfx::RectF::default();
        };
        let mut bounds = first.BoundingBox();
        for quad in quads {
            bounds.Union(quad.BoundingBox());
        }
        bounds
    }

    pub fn AbsoluteBoundingBoxRectFDefault(&self) -> gfx::RectF {
        self.AbsoluteBoundingBoxRectF(MapCoordinatesFlags::default())
    }

    #[inline(never)]
    pub fn AbsoluteBoundingBoxRect(&self, mode: MapCoordinatesFlags) -> gfx::Rect {
        // Source: core/layout/layout_object.cc:2143-2160.
        self.CheckIsNotDestroyed();
        debug_assert!(!mode.contains(MapCoordinatesMode::kIgnoreTransforms));
        let mut quads = Vector::new();
        self.AbsoluteQuads(&mut quads, mode, BoxQuadType::kBorder);
        let mut bounds = gfx::RectF::default();
        for quad in quads.iter() {
            bounds.Union(quad.BoundingBox());
        }
        gfx::ToEnclosingRect(bounds)
    }

    pub fn AbsoluteBoundingBoxRectDefault(&self) -> gfx::Rect {
        self.AbsoluteBoundingBoxRect(MapCoordinatesFlags::default())
    }

    #[inline(never)]
    pub fn AbsoluteBoundingBoxRectForUnboundedElement(&self) -> gfx::Rect {
        unsafe { LayoutObjectAbsoluteBoundingBoxRectForUnboundedElementProvider(self) }
    }

    #[inline(never)]
    pub fn AbsoluteBoundingBoxRectHandlingEmptyInline(
        &self,
        flags: MapCoordinatesFlags,
    ) -> PhysicalRect {
        // Source: core/layout/layout_object.cc:2178-2182 and the LayoutInline
        // final override, layout_inline.cc:542-552.
        self.CheckIsNotDestroyed();
        if let Some(inline) = unsafe { DynamicTo::<LayoutInline>(self as *const _).as_ref() } {
            return inline.AbsoluteBoundingBoxRectHandlingEmptyInline(flags);
        }
        PhysicalRect::EnclosingRect(&self.AbsoluteBoundingBoxRectF(flags))
    }

    pub fn AbsoluteBoundingBoxRectHandlingEmptyInlineDefault(&self) -> PhysicalRect {
        self.AbsoluteBoundingBoxRectHandlingEmptyInline(MapCoordinatesFlags::default())
    }

    #[inline(never)]
    pub fn AbsoluteBoundingBoxRectForScrollIntoView(&self) -> PhysicalRect {
        // Source: core/layout/layout_object.cc:2184-2222. Indexed ::column
        // pseudos are not admitted by this DOM allocator, so every resident
        // scroll marker follows Chromium's ordinary originating-element path.
        self.CheckIsNotDestroyed();
        if self.StyleRef().StyleType() == PseudoId::kPseudoIdScrollMarker {
            let node = self.GetNode();
            assert!(!node.is_null(), "scroll marker requires a node");
            let origin = unsafe { &*node }.UltimateOriginatingElement();
            assert!(
                !origin.is_null(),
                "scroll marker requires its ultimate originating element"
            );
            let origin_object = unsafe { &*origin }.GetLayoutObject();
            assert!(
                !origin_object.is_null(),
                "scroll marker origin requires a layout object"
            );
            return unsafe { &*origin_object }.AbsoluteBoundingBoxRectForScrollIntoView();
        }
        let flags = if RuntimeEnabledFeatures::CSSPositionStickyStaticScrollPositionEnabled() {
            MapCoordinatesFlags::from_mode(MapCoordinatesMode::kIgnoreStickyOffset)
        } else {
            MapCoordinatesFlags::default()
        };
        self.AbsoluteBoundingBoxRectHandlingEmptyInline(flags)
    }

    #[inline(never)]
    pub fn AbsoluteBoundingBoxRectIncludingDescendants(&self) -> gfx::Rect {
        // Source: core/layout/layout_object.cc:2236-2245. Only descendant
        // layers expand this object's own native absolute bounds.
        self.CheckIsNotDestroyed();
        let mut bounds = self.AbsoluteBoundingBoxRectDefault();
        let mut child = self.SlowFirstChild();
        while let Some(current) = unsafe { child.as_mut() } {
            current.AddAbsoluteRectForLayer(&mut bounds);
            child = current.NextSibling();
        }
        bounds
    }
}
