//! LayoutObject container, ancestor, and absolute coordinate conversion.
//!
//! Provider-backed operations may walk container or transform ancestry and
//! retain fragment-owned sampling frames. Thin type/absolute wrappers remain
//! inlineable so common point conversions do not accumulate artificial calls.

use super::*;
use crate::internal::map_coordinates_flags::MapCoordinatesMode;
use foundation::{LayoutUnit, To};

impl LayoutObject {
    // Base virtual implementation.
    // cpp: layoutng/internal/layout_object_core_services.cc:436-446
    pub fn OffsetFromContainerInternalBase(
        &self,
        container: *const LayoutObject,
        mode: MapCoordinatesFlags,
    ) -> PhysicalOffset {
        self.CheckIsNotDestroyed();
        assert!(!container.is_null());
        debug_assert_eq!(container, self.Container() as *const LayoutObject);
        if unsafe { &*container }.IsScrollContainer() {
            return self.OffsetFromScrollableContainer(container, mode);
        }
        PhysicalOffset::default()
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:447-461
    #[inline(never)]
    pub fn OffsetFromScrollableContainer(
        &self,
        container: *const LayoutObject,
        mode: MapCoordinatesFlags,
    ) -> PhysicalOffset {
        self.CheckIsNotDestroyed();
        assert!(!container.is_null());
        debug_assert!(unsafe { &*container }.IsScrollContainer());
        if self.IsFixedPositioned() && unsafe { &*container }.IsLayoutView() {
            return PhysicalOffset::default();
        }
        if mode.contains(MapCoordinatesMode::kIgnoreScrollOriginAndOffset) {
            return PhysicalOffset::default();
        }
        let box_ = To::<LayoutBox>(container);
        if !mode.contains(MapCoordinatesMode::kIgnoreScrollOffset) {
            return -unsafe { &*box_ }.ScrolledContentOffset();
        }
        let area = unsafe { &*box_ }.GetScrollableArea();
        assert!(!area.is_null());
        let origin = unsafe { &*area }.ScrollOrigin();
        PhysicalOffset::new(
            LayoutUnit::from_signed(origin.x()),
            LayoutUnit::from_signed(origin.y()),
        )
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:463-469
    pub fn OffsetFromOverscrollContainer(
        &self,
        _container: *const LayoutObject,
        _mode: MapCoordinatesFlags,
    ) -> PhysicalOffset {
        PhysicalOffset::default()
    }

    // cpp: layoutng/internal/layout_object.h:2410-2410
    #[inline(never)]
    pub fn OffsetFromAncestor(&self, ancestor: *const LayoutObject) -> PhysicalOffset {
        unsafe { LayoutObjectOffsetFromAncestorProvider(self, ancestor) }
    }

    // cpp: layoutng/internal/layout_object.h:2401-2405
    pub fn OffsetFromContainer(
        &self,
        container: *const LayoutObject,
        mode: MapCoordinatesFlags,
    ) -> PhysicalOffset {
        self.CheckIsNotDestroyed();
        self.OffsetFromContainerInternal(container, mode)
    }

    pub fn OffsetFromContainerDefault(&self, container: *const LayoutObject) -> PhysicalOffset {
        self.OffsetFromContainer(container, MapCoordinatesFlags::default())
    }

    // This virtual offset is commonly called once per ancestor step, so keep
    // the wrapper inlineable and let the enclosing conversion own the frame.
    // cpp: layoutng/internal/layout_object.h:3459-3461
    pub fn OffsetFromContainerInternal(
        &self,
        container: *const LayoutObject,
        mode: MapCoordinatesFlags,
    ) -> PhysicalOffset {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectOffsetFromContainerInternal(self, container, mode) }
    }

    // cpp: layoutng/internal/layout_object.h:2287-2294
    pub fn AncestorToLocalRect(
        &self,
        ancestor: *const LayoutBoxModelObject,
        rect: &PhysicalRect,
        mode: MapCoordinatesFlags,
    ) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        let quad = gfx::QuadF::from(gfx::RectF::from(*rect));
        PhysicalRect::EnclosingRect(
            &self
                .AncestorToLocalQuad(ancestor, &quad, mode)
                .BoundingBox(),
        )
    }

    // cpp: layoutng/internal/layout_object.h:2295-2297
    #[inline(never)]
    pub fn AncestorToLocalQuad(
        &self,
        ancestor: *const LayoutBoxModelObject,
        quad: &gfx::QuadF,
        mode: MapCoordinatesFlags,
    ) -> gfx::QuadF {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectAncestorToLocalQuad(self, ancestor, quad, mode) }
    }

    // cpp: layoutng/internal/layout_object.h:2298-2304
    pub fn AncestorToLocalPointPhysical(
        &self,
        ancestor: *const LayoutBoxModelObject,
        point: &PhysicalOffset,
        mode: MapCoordinatesFlags,
    ) -> PhysicalOffset {
        self.CheckIsNotDestroyed();
        let float_point = gfx::PointF::new(point.left.ToFloat(), point.top.ToFloat());
        PhysicalOffset::FromPointFRound(&self.AncestorToLocalPointF(ancestor, &float_point, mode))
    }

    // cpp: layoutng/internal/layout_object.h:2305-2307
    #[inline(never)]
    pub fn AncestorToLocalPointF(
        &self,
        ancestor: *const LayoutBoxModelObject,
        point: &gfx::PointF,
        mode: MapCoordinatesFlags,
    ) -> gfx::PointF {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectAncestorToLocalPointFProvider(self, ancestor, point, mode) }
    }

    // cpp: layoutng/internal/layout_object.h:2315-2317
    #[inline(never)]
    pub fn LocalToAncestorRect(
        &self,
        rect: &PhysicalRect,
        ancestor: *const LayoutBoxModelObject,
        mode: MapCoordinatesFlags,
    ) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectLocalToAncestorRectProvider(self, rect, ancestor, mode) }
    }

    // cpp: layoutng/internal/layout_object.h:2318-2323
    pub fn LocalRectToAncestorQuad(
        &self,
        rect: &PhysicalRect,
        ancestor: *const LayoutBoxModelObject,
        mode: MapCoordinatesFlags,
    ) -> gfx::QuadF {
        self.CheckIsNotDestroyed();
        let quad = gfx::QuadF::from(gfx::RectF::from(*rect));
        self.LocalToAncestorQuad(&quad, ancestor, mode)
    }

    // cpp: layoutng/internal/layout_object.h:2324-2326
    #[inline(never)]
    pub fn LocalToAncestorQuad(
        &self,
        quad: &gfx::QuadF,
        ancestor: *const LayoutBoxModelObject,
        mode: MapCoordinatesFlags,
    ) -> gfx::QuadF {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectLocalToAncestorQuad(self, quad, ancestor, mode) }
    }

    // cpp: layoutng/internal/layout_object.h:2327-2333
    pub fn LocalToAncestorPointPhysical(
        &self,
        point: &PhysicalOffset,
        ancestor: *const LayoutBoxModelObject,
        mode: MapCoordinatesFlags,
    ) -> PhysicalOffset {
        self.CheckIsNotDestroyed();
        let float_point = gfx::PointF::new(point.left.ToFloat(), point.top.ToFloat());
        PhysicalOffset::FromPointFRound(&self.LocalToAncestorPoint(&float_point, ancestor, mode))
    }

    // cpp: layoutng/internal/layout_object.h:2342-2343
    #[inline(never)]
    pub fn LocalToAncestorTransform(
        &self,
        ancestor: *const LayoutBoxModelObject,
        mode: MapCoordinatesFlags,
    ) -> gfx::Transform {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectLocalToAncestorTransformProvider(self, ancestor, mode) }
    }

    // cpp: layoutng/internal/layout_object.h:2344-2347
    pub fn LocalToAbsoluteTransform(&self, mode: MapCoordinatesFlags) -> gfx::Transform {
        self.CheckIsNotDestroyed();
        self.LocalToAncestorTransform(std::ptr::null(), mode)
    }

    // cpp: layoutng/internal/layout_object.h:2352-2356
    pub fn LocalToAbsoluteRect(
        &self,
        rect: &PhysicalRect,
        mode: MapCoordinatesFlags,
    ) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        self.LocalToAncestorRect(rect, std::ptr::null(), mode)
    }

    // cpp: layoutng/internal/layout_object.h:2357-2361
    pub fn LocalRectToAbsoluteQuad(
        &self,
        rect: &PhysicalRect,
        mode: MapCoordinatesFlags,
    ) -> gfx::QuadF {
        self.CheckIsNotDestroyed();
        self.LocalRectToAncestorQuad(rect, std::ptr::null(), mode)
    }

    // cpp: layoutng/internal/layout_object.h:2362-2366
    pub fn LocalToAbsoluteQuad(&self, quad: &gfx::QuadF, mode: MapCoordinatesFlags) -> gfx::QuadF {
        self.CheckIsNotDestroyed();
        self.LocalToAncestorQuad(quad, std::ptr::null(), mode)
    }

    // cpp: layoutng/internal/layout_object.h:2367-2371
    pub fn LocalToAbsolutePointPhysical(
        &self,
        point: &PhysicalOffset,
        mode: MapCoordinatesFlags,
    ) -> PhysicalOffset {
        self.CheckIsNotDestroyed();
        self.LocalToAncestorPointPhysical(point, std::ptr::null(), mode)
    }

    // cpp: layoutng/internal/layout_object.h:2372-2376
    pub fn LocalToAbsolutePointF(
        &self,
        point: &gfx::PointF,
        mode: MapCoordinatesFlags,
    ) -> gfx::PointF {
        self.CheckIsNotDestroyed();
        self.LocalToAncestorPoint(point, std::ptr::null(), mode)
    }

    // cpp: layoutng/internal/layout_object.h:2377-2381
    pub fn AbsoluteToLocalRect(
        &self,
        rect: &PhysicalRect,
        mode: MapCoordinatesFlags,
    ) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        self.AncestorToLocalRect(std::ptr::null(), rect, mode)
    }

    // cpp: layoutng/internal/layout_object.h:2382-2386
    pub fn AbsoluteToLocalQuad(&self, quad: &gfx::QuadF, mode: MapCoordinatesFlags) -> gfx::QuadF {
        self.CheckIsNotDestroyed();
        self.AncestorToLocalQuad(std::ptr::null(), quad, mode)
    }

    // cpp: layoutng/internal/layout_object.h:2387-2391
    pub fn AbsoluteToLocalPointPhysical(
        &self,
        point: &PhysicalOffset,
        mode: MapCoordinatesFlags,
    ) -> PhysicalOffset {
        self.CheckIsNotDestroyed();
        self.AncestorToLocalPointPhysical(std::ptr::null(), point, mode)
    }

    // cpp: layoutng/internal/layout_object.h:2392-2396
    pub fn AbsoluteToLocalPointF(
        &self,
        point: &gfx::PointF,
        mode: MapCoordinatesFlags,
    ) -> gfx::PointF {
        self.CheckIsNotDestroyed();
        self.AncestorToLocalPointF(std::ptr::null(), point, mode)
    }
}
