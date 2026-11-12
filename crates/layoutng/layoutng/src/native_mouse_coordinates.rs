//! MouseEvent's source LayoutObject coordinate queries (mouse_event.cc:63-76,
//! 475-522). The handle roots the original object; it contains no geometry copy.
use crate::{
    internal::{
        layout_box_model_object::LayoutBoxModelObject, layout_object::LayoutObject,
        map_coordinates_flags::MapCoordinatesFlags,
    },
    layout_engine::LayoutEngine,
};
use foundation::{DynamicTo, Persistent, PhysicalOffset, gfx};
/// DraggedNodeImageBuilder's geometry, computed by the original Layout
/// ancestry/coordinate owner. Paint reads its own FirstFragment offset/slots.
pub struct NativeDragResourceImageFacts {
    pub resource_id: Option<u64>,
    pub image_rendering: foundation::EImageRendering,
    pub respect_orientation: bool,
}
pub struct NativeDragNodeImageGeometry {
    pub layer_client_id: u64,
    pub bounding_box: gfx::RectF,
}
pub struct NativeMouseCoordinateLayout {
    object: Persistent<LayoutObject>,
}
impl NativeMouseCoordinateLayout {
    fn new(object: *mut LayoutObject) -> Option<Self> {
        (!object.is_null()).then(|| Self {
            object: Persistent::from_ptr(object),
        })
    }
    fn object(&self) -> &LayoutObject {
        unsafe { &*self.object.Get() }
    }
    /// HTMLMediaElement::UpdateLayoutObject, cc:2535-2538. The handle
    /// invokes the actual native virtual; no LayoutObject escapes this API.
    pub fn UpdateMediaFromElement(&self, engine: &LayoutEngine) {
        let _scope = engine.NativeScrollPassScope();
        let object = unsafe { &mut *self.object.Get() };
        assert!(object.IsMedia());
        object.UpdateFromElement();
    }
    /// TextTrackContainer::UpdateDefaultFontSize reads LayoutVideo's actual
    /// content box (cc:112-128); generated-content replacement is not a video.
    pub fn VideoContentBoxSize(&self, engine: &LayoutEngine) -> Option<foundation::PhysicalSize> {
        let _scope = engine.NativeScrollPassScope();
        if !self.object().IsVideo() {
            return None;
        }
        let box_ = DynamicTo::<crate::internal::layout_box::LayoutBox>(self.object.Get());
        (!box_.is_null()).then(|| unsafe { (&*box_).PhysicalContentBoxRect().size })
    }
    /// SimulatedEventUtil::PopulateMouseEventInitCoordinates, cc:55-60.
    /// Borrow the original LayoutBox and its coordinate owner; no AX or
    /// document-wide geometry projection is needed for an accessible click.
    pub fn SimulatedClickRootFrameCenter(
        &self,
        engine: &LayoutEngine,
    ) -> Option<(PhysicalOffset, f32)> {
        let _scope = engine.NativeScrollPassScope();
        let box_ = unsafe {
            DynamicTo::<crate::internal::layout_box::LayoutBox>(self.object.Get()).as_ref()
        }?;
        let center = box_.PhysicalBorderBoxRect().Center();
        let root_center = self.object().LocalToAncestorPointPhysical(
            &center,
            std::ptr::null(),
            MapCoordinatesFlags::from_mode(
                crate::internal::map_coordinates_flags::MapCoordinatesMode::kTraverseDocumentBoundaries,
            ),
        );
        Some((root_center, self.object().StyleRef().EffectiveZoom()))
    }
    pub fn NonPseudoNodeId(&self) -> Option<u64> {
        Some(unsafe { self.object().NonPseudoNode().as_ref()? }.InputId())
    }
    pub fn UserDrag(&self) -> foundation::EUserDrag {
        self.object().StyleRef().UserDrag()
    }
    pub fn IsImage(&self) -> bool {
        self.object().IsImage()
    }
    /// LayoutImageResource's live resource edge and the same ComputedStyle.
    /// The resource owner applies ForceOrientationIfNecessary, not Page.
    pub fn DragImageComputedStyleOwner(
        &self,
    ) -> Option<foundation::Persistent<layoutng_style::style::computed_style::ComputedStyle>> {
        self.object().IsImage().then(|| {
            foundation::Persistent::from_ptr(
                (self.object().StyleRef()
                    as *const layoutng_style::style::computed_style::ComputedStyle)
                    .cast_mut(),
            )
        })
    }
    pub fn DragResourceImageFacts(&self) -> Option<NativeDragResourceImageFacts> {
        self.object()
            .IsImage()
            .then(|| NativeDragResourceImageFacts {
                resource_id: self
                    .object()
                    .InputOwnerForLayout()
                    .InputStyleResources()
                    .image_resource_id,
                image_rendering: self.object().StyleRef().ImageRendering(),
                respect_orientation: self.object().StyleRef().ImageOrientation()
                    == foundation::RespectImageOrientationEnum::kRespectImageOrientation,
            })
    }

    pub fn AbsoluteBoundingBoxIncludingDescendants(&self, engine: &LayoutEngine) -> gfx::Rect {
        let _scope = engine.NativeScrollPassScope();
        self.object().AbsoluteBoundingBoxRectIncludingDescendants()
    }
    /// Source data_transfer.cc:109-134. The caller updates lifecycle before
    /// obtaining this handle; no native PaintOffset is projected here because
    /// its canonical committed value belongs to PrePaint's FragmentData slot.
    pub fn DragNodeImageGeometry(
        &self,
        engine: &LayoutEngine,
    ) -> Option<NativeDragNodeImageGeometry> {
        if self.object().WasDestroyedForTreeUpdate() {
            return None;
        }
        let _scope = engine.NativeScrollPassScope();
        let mut layer = unsafe { self.object().EnclosingLayer().as_ref() }
            .expect("source laid-out node has its enclosing PaintLayer");
        if !layer.GetLayoutObject().IsStackingContext() {
            layer = unsafe { layer.AncestorStackingContext().as_ref() }
                .expect("source enclosing layer has a root stacking context");
        }
        let bounds = self.object().AbsoluteBoundingBoxRectIncludingDescendants();
        let bounds = gfx::Rect::new(
            gfx::Point::new(bounds.x(), bounds.y()),
            gfx::Size::new(bounds.width().min(8192), bounds.height().min(8192)),
        );
        let bounds = gfx::RectF::new(
            gfx::PointF::new(bounds.x() as f32, bounds.y() as f32),
            gfx::SizeF::new(bounds.width() as f32, bounds.height() as f32),
        );
        let bounds = layer
            .GetLayoutObject()
            .AbsoluteToLocalQuad(&gfx::QuadF::from(bounds), MapCoordinatesFlags::empty())
            .BoundingBox();
        Some(NativeDragNodeImageGeometry {
            layer_client_id: layer.Id(),
            bounding_box: bounds,
        })
    }
    pub fn IsSVG(&self) -> bool {
        self.object().IsSVG()
    }
    pub fn IsSVGRoot(&self) -> bool {
        self.object().IsSVGRoot()
    }
    pub fn Parent(&self) -> Option<Self> {
        Self::new(self.object().Parent())
    }
    pub fn NodeId(&self) -> Option<u64> {
        Some(unsafe { self.object().GetNode().as_ref()? }.InputId())
    }
    pub fn AbsoluteToLocalPoint(&self, engine: &LayoutEngine, point: gfx::PointF) -> gfx::PointF {
        let _scope = engine.NativeScrollPassScope();
        self.object()
            .AbsoluteToLocalPointF(&point, MapCoordinatesFlags::empty())
    }
    pub fn BorderOffset(&self) -> Option<(f32, f32)> {
        let box_ = unsafe { DynamicTo::<LayoutBoxModelObject>(self.object.Get()).as_ref() }?;
        let outsets = box_.BorderOutsets();
        Some((outsets.left.ToFloat(), outsets.top.ToFloat()))
    }
    pub fn EnclosingSelfPaintingLayerAbsoluteOrigin(&self, engine: &LayoutEngine) -> (f32, f32) {
        let _scope = engine.NativeScrollPassScope();
        let layer = unsafe { self.object().EnclosingLayer().as_ref() }
            .expect("source laid-out object has enclosing layer");
        let layer = unsafe { layer.EnclosingSelfPaintingLayer().as_ref() }
            .expect("source root self-painting layer");
        let p = layer
            .GetLayoutObject()
            .LocalToAbsolutePointPhysical(&PhysicalOffset::default(), MapCoordinatesFlags::empty());
        (p.left.ToFloat(), p.top.ToFloat())
    }
}
impl LayoutEngine {
    pub fn MouseLayoutViewportScrollOffset(&self) -> Option<(f32, f32)> {
        let root = self.GetLayoutTree()?.Root();
        let root = unsafe {
            DynamicTo::<crate::internal::layout_box::LayoutBox>(root as *const _).as_ref()
        }?;
        let area = unsafe { root.GetScrollableArea().as_ref() }?;
        let offset = area.GetScrollOffset();
        Some((offset.x() as f32, offset.y() as f32))
    }

    pub fn MouseLayoutViewportScrollOffsetInt(&self) -> Option<(i32, i32)> {
        let root = self.GetLayoutTree()?.Root();
        let root = unsafe {
            DynamicTo::<crate::internal::layout_box::LayoutBox>(root as *const _).as_ref()
        }?;
        let area = unsafe { root.GetScrollableArea().as_ref() }?;
        let offset = area.ScrollOffsetInt();
        Some((offset.x(), offset.y()))
    }
    pub fn MouseCoordinateLayout(&self, id: u64) -> Option<NativeMouseCoordinateLayout> {
        NativeMouseCoordinateLayout::new(
            self.GetLayoutTree()?.OutputNode(id)? as *const _ as *mut LayoutObject
        )
    }
}

pub struct NativeRegisteredScrollableArea {
    source: Persistent<crate::internal::layout_scrollable_area::PaintLayerScrollableArea>,
}
impl NativeRegisteredScrollableArea {
    /// The same ScrollableArea identity used by the user animator registry.
    pub fn Identity(&self) -> usize {
        self.source.Get() as usize
    }
    pub fn ScrollsOverflow(&self) -> bool {
        let area = unsafe { &*self.source.Get() };
        !area.HasBeenDisposed() && area.ScrollsOverflow()
    }
    pub fn ScrollElementId(&self) -> foundation::CompositorElementId {
        unsafe { &*self.source.Get() }.GetScrollElementId()
    }
}
impl LayoutEngine {
    pub fn RegisteredScrollableAreas(&self) -> Vec<NativeRegisteredScrollableArea> {
        let Some(tree) = self.GetLayoutTree() else {
            return Vec::new();
        };
        let _scope = self.NativeScrollPassScope();
        let view = unsafe {
            DynamicTo::<crate::internal::layout_view::LayoutView>(tree.Root() as *const _).as_ref()
        }
        .expect("source LayoutView root");
        view.RegisteredScrollableAreas()
            .into_iter()
            .map(|source| NativeRegisteredScrollableArea { source })
            .collect()
    }
}

impl LayoutEngine {
    /// LocalFrameView::ViewportWidth (cc4716-4719), reading its resident
    /// LayoutView viewport input. The caller performs source absolute-zoom
    /// integer adjustment, preserving its original rounding.
    pub fn MobileMetricsViewportWidthAndZoom(&self) -> Option<(i32, f32)> {
        let tree = self.GetLayoutTree()?;
        let _scope = self.NativeScrollPassScope();
        let root = tree.Root();
        let view = unsafe {
            DynamicTo::<crate::internal::layout_view::LayoutView>(root as *const _).as_ref()
        }?;
        Some((
            view.GetLayoutSize(
                crate::internal::scroll_types::IncludeScrollbarsInRect::kIncludeScrollbars,
            )
            .width(),
            root.StyleRef().EffectiveZoom(),
        ))
    }
}
