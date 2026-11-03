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

    pub fn OffsetFromOverscrollContainer(
        &self,
        _container: *const LayoutObject,
        _mode: MapCoordinatesFlags,
    ) -> PhysicalOffset {
        PhysicalOffset::default()
    }

    #[inline(never)]
    pub fn OffsetFromAncestor(&self, ancestor: *const LayoutObject) -> PhysicalOffset {
        self.CheckIsNotDestroyed();
        if std::ptr::eq(self, ancestor) {
            return PhysicalOffset::default();
        }

        let mut offset = PhysicalOffset::default();
        let mut current = self as *const LayoutObject;
        let mut skip_info = AncestorSkipInfo::new(ancestor);
        loop {
            let next = unsafe { &*current }.ContainerWithSkipInfo(&mut skip_info);
            assert!(!next.is_null(), "ancestor is not in the container chain");
            if next.is_null() {
                break;
            }
            debug_assert!(!unsafe { &*current }.HasTransformRelatedProperty());
            offset += unsafe { &*current }.OffsetFromContainerDefault(next);
            current = next;
            if current == ancestor || skip_info.AncestorSkipped() {
                break;
            }
        }
        if skip_info.AncestorSkipped() {
            debug_assert!(!ancestor.is_null());
            offset -= unsafe { &*ancestor }.OffsetFromAncestor(current);
        }
        offset
    }

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
    pub fn OffsetFromContainerInternal(
        &self,
        container: *const LayoutObject,
        mode: MapCoordinatesFlags,
    ) -> PhysicalOffset {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectOffsetFromContainerInternal(self, container, mode) }
    }

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

    #[inline(never)]
    pub fn LocalToAncestorRect(
        &self,
        rect: &PhysicalRect,
        ancestor: *const LayoutBoxModelObject,
        mode: MapCoordinatesFlags,
    ) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        let quad = gfx::QuadF::from(gfx::RectF::from(*rect));
        let mut state = TransformState::new(TransformState::kApplyTransformDirection, quad);
        self.MapLocalToAncestorForGeometry(ancestor, &mut state, mode);
        state.Flatten();
        PhysicalRect::EnclosingRect(&state.LastPlanarQuad().BoundingBox())
    }

    fn MapLocalToAncestorForGeometry(
        &self,
        ancestor: *const LayoutBoxModelObject,
        state: &mut TransformState,
        mode: MapCoordinatesFlags,
    ) {
        debug_assert_eq!(state.Direction(), TransformState::kApplyTransformDirection);
        let ancestor_object = ancestor.cast::<LayoutObject>();
        let mut current = self as *const LayoutObject;
        while current != ancestor_object {
            let current_object = unsafe { &*current };
            let canvas = current_object.CanvasForDrawingLayoutObject();
            if !canvas.is_null() {
                let use_transforms = !mode.contains(MapCoordinatesMode::kIgnoreTransforms);
                let preserve_3d = use_transforms && current_object.StyleRef().Preserves3D();
                let accumulation = if preserve_3d {
                    TransformState::kAccumulateTransform
                } else {
                    TransformState::kFlattenTransform
                };
                if use_transforms && current_object.ShouldUseTransformFromContainer(canvas) {
                    let mut transform = gfx::Transform::default();
                    current_object.GetTransformFromContainer(
                        canvas,
                        PhysicalOffset::default(),
                        &mut transform,
                        std::ptr::null(),
                        None,
                    );
                    state.ApplyTransform(&transform, accumulation);
                }
                if canvas.cast_const() == ancestor_object {
                    return;
                }
                current = canvas;
                continue;
            }

            let mut skip_info = AncestorSkipInfo::new(ancestor_object);
            let container = current_object.ContainerWithSkipInfo(&mut skip_info);
            if container.is_null() {
                return;
            }

            let container_offset = current_object.OffsetFromContainer(container, mode);
            let use_transforms = !mode.contains(MapCoordinatesMode::kIgnoreTransforms);
            let container_object = unsafe { &*container };
            let container_preserves_3d =
                container_object.StyleRef().Preserves3D() && !container_object.IsText();
            let path_preserves_3d = container == current_object.NearestAncestorForElement();
            let preserve_3d = use_transforms && container_preserves_3d && path_preserves_3d;
            let accumulation = if preserve_3d {
                TransformState::kAccumulateTransform
            } else {
                TransformState::kFlattenTransform
            };

            if use_transforms && current_object.ShouldUseTransformFromContainer(container) {
                let mut transform = gfx::Transform::default();
                current_object.GetTransformFromContainer(
                    container,
                    container_offset,
                    &mut transform,
                    std::ptr::null(),
                    None,
                );
                state.ApplyTransform(&transform, accumulation);
            } else {
                state.Move(container_offset, accumulation);
            }

            if skip_info.AncestorSkipped() {
                debug_assert!(!ancestor_object.is_null());
                state.Move(
                    -unsafe { &*ancestor_object }.OffsetFromAncestor(container),
                    accumulation,
                );
                return;
            }
            current = container;
        }
    }

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

    #[inline(never)]
    pub fn LocalToAncestorQuad(
        &self,
        quad: &gfx::QuadF,
        ancestor: *const LayoutBoxModelObject,
        mode: MapCoordinatesFlags,
    ) -> gfx::QuadF {
        self.CheckIsNotDestroyed();
        // Chromium keeps the quad in floating coordinates while it walks the
        // container chain, then flattens it at the requested ancestor.  Use
        // the same geometry path as LocalToAncestorRect; enclosing it in a
        // PhysicalRect here would lose subpixel information needed by scroll
        // anchoring and hit testing.
        let mut state = TransformState::new(TransformState::kApplyTransformDirection, *quad);
        self.MapLocalToAncestorForGeometry(ancestor, &mut state, mode);
        state.Flatten();
        state.LastPlanarQuad()
    }

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

    #[inline(never)]
    pub fn LocalToAncestorTransform(
        &self,
        ancestor: *const LayoutBoxModelObject,
        mode: MapCoordinatesFlags,
    ) -> gfx::Transform {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectLocalToAncestorTransformProvider(self, ancestor, mode) }
    }

    pub fn LocalToAbsoluteTransform(&self, mode: MapCoordinatesFlags) -> gfx::Transform {
        self.CheckIsNotDestroyed();
        self.LocalToAncestorTransform(std::ptr::null(), mode)
    }

    pub fn LocalToAbsoluteRect(
        &self,
        rect: &PhysicalRect,
        mode: MapCoordinatesFlags,
    ) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        self.LocalToAncestorRect(rect, std::ptr::null(), mode)
    }

    pub fn LocalRectToAbsoluteQuad(
        &self,
        rect: &PhysicalRect,
        mode: MapCoordinatesFlags,
    ) -> gfx::QuadF {
        self.CheckIsNotDestroyed();
        self.LocalRectToAncestorQuad(rect, std::ptr::null(), mode)
    }

    pub fn LocalToAbsoluteQuad(&self, quad: &gfx::QuadF, mode: MapCoordinatesFlags) -> gfx::QuadF {
        self.CheckIsNotDestroyed();
        self.LocalToAncestorQuad(quad, std::ptr::null(), mode)
    }

    pub fn LocalToAbsolutePointPhysical(
        &self,
        point: &PhysicalOffset,
        mode: MapCoordinatesFlags,
    ) -> PhysicalOffset {
        self.CheckIsNotDestroyed();
        self.LocalToAncestorPointPhysical(point, std::ptr::null(), mode)
    }

    pub fn LocalToAbsolutePointF(
        &self,
        point: &gfx::PointF,
        mode: MapCoordinatesFlags,
    ) -> gfx::PointF {
        self.CheckIsNotDestroyed();
        self.LocalToAncestorPoint(point, std::ptr::null(), mode)
    }

    pub fn AbsoluteToLocalRect(
        &self,
        rect: &PhysicalRect,
        mode: MapCoordinatesFlags,
    ) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        self.AncestorToLocalRect(std::ptr::null(), rect, mode)
    }

    pub fn AbsoluteToLocalQuad(&self, quad: &gfx::QuadF, mode: MapCoordinatesFlags) -> gfx::QuadF {
        self.CheckIsNotDestroyed();
        self.AncestorToLocalQuad(std::ptr::null(), quad, mode)
    }

    pub fn AbsoluteToLocalPointPhysical(
        &self,
        point: &PhysicalOffset,
        mode: MapCoordinatesFlags,
    ) -> PhysicalOffset {
        self.CheckIsNotDestroyed();
        self.AncestorToLocalPointPhysical(std::ptr::null(), point, mode)
    }

    pub fn AbsoluteToLocalPointF(
        &self,
        point: &gfx::PointF,
        mode: MapCoordinatesFlags,
    ) -> gfx::PointF {
        self.CheckIsNotDestroyed();
        self.AncestorToLocalPointF(std::ptr::null(), point, mode)
    }
}
