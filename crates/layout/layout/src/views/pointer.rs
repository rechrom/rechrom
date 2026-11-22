//! Pointer view: mouse-coordinate sources, pointer scrollbar/resizer queries
//! and narrow resident-tree facts.
//!
//! `LayoutEngine::Pointer()` borrows the core engine for one query sequence.
//! Every method forwards to exactly the core call the former facade method
//! made; they stay `#[inline]` and allocation-free (apart from what the core
//! call itself returns) because pointer input is latency-sensitive. Pointer
//! commands (overlay fade timer, resize mode) are `LayoutMutation::Pointer`.

use foundation::{gfx, PhysicalOffset};
use layout_base::internal::layout_box::LayoutBox;
use layout_base::internal::layout_object::LayoutObject;
use layout_base::internal::layout_object_builder::LayoutObjectTree;
use layout_base::internal::layout_pass_scope::SVGResourceQueryScope;
use layout_base::internal::layout_view::LayoutView;
use layout_base::internal::scrollbar::Scrollbar;
use layout_base::layout_engine::LayoutEngine as CoreLayoutEngine;
use layout_base::engine::pointer::mouse_coordinates::{
    NativeDragNodeImageGeometry, NativeMouseCoordinateLayout, NativeRegisteredScrollableArea,
};
use layout_base::engine::pointer::pointer_scroll_controls::{
    NativeMacScrollbarAnimationUpdate, NativeOverlayVisibilityUpdate, NativePointerResizeArea,
    NativeResizePlan, NativeScrollbarInteraction,
};

use crate::LayoutEngine;

/// Opaque lifetime guard for Layout's `SVGResourceQueryScope`: while it is
/// alive, Paint/PrePaint resolve native SVG resource owners from the resident
/// tree. Consumers only hold and drop it; the scope type stays private.
#[must_use = "the SVG resource scope ends when the guard is dropped"]
pub struct SVGResourceQueryGuard<'a> {
    _scope: SVGResourceQueryScope<'a>,
}

/// Borrowed pointer and mouse-coordinate queries over the resident layout.
#[derive(Clone, Copy)]
pub struct PointerQueries<'a> {
    engine: &'a CoreLayoutEngine,
}

impl LayoutEngine {
    /// Pointer, mouse-coordinate and resident-tree queries.
    #[inline]
    pub fn Pointer(&self) -> PointerQueries<'_> {
        PointerQueries {
            engine: &self.engine,
        }
    }
}

/// Resident-tree facts.
impl<'a> PointerQueries<'a> {
    /// Whether a LayoutView (resident LayoutObject tree) exists.
    #[inline]
    pub fn HasLayoutView(&self) -> bool {
        self.engine.GetLayoutTree().is_some()
    }

    /// Whether `node_id` currently has an attached LayoutObject.
    #[inline]
    pub fn HasLayoutObjectForNode(&self, node_id: u64) -> bool {
        self.engine
            .GetLayoutTree()
            .and_then(|tree| tree.GetLayoutObjectForNode(node_id))
            .is_some()
    }

    /// The resident LayoutView root object.
    #[inline]
    pub fn LayoutViewRoot(&self) -> Option<&'a LayoutObject> {
        let engine: &'a CoreLayoutEngine = self.engine;
        engine.GetLayoutTree().map(LayoutObjectTree::Root)
    }

    /// The resident root downcast to its LayoutView, for hit-test queries
    /// that enter the LayoutView directly (`LayoutView::HitTest*`). `None`
    /// when no tree exists or the root is not a LayoutView.
    #[inline]
    pub fn ResidentLayoutView(&self) -> Option<&'a LayoutView> {
        let root = self.LayoutViewRoot()?;
        // SAFETY: the downcast target is the root object, borrowed for `'a`.
        unsafe { foundation::DynamicTo::<LayoutView>(root).as_ref() }
    }

    /// `LayoutBox::ScrolledContentOffset()` of the resident LayoutView.
    /// `None` when no tree exists or the root is not a LayoutBox.
    #[inline]
    pub fn LayoutViewScrolledContentOffset(&self) -> Option<PhysicalOffset> {
        let root = self.LayoutViewRoot()?;
        // SAFETY: the downcast target is the root object, borrowed for `'a`.
        let box_ = unsafe { foundation::DynamicTo::<LayoutBox>(root).as_ref() }?;
        Some(box_.ScrolledContentOffset())
    }

    /// Scope that resolves native SVG resource owners while Paint/PrePaint
    /// walk this resident tree. `None` when no tree exists. Hold the guard
    /// for the whole Paint/PrePaint call; dropping it leaves the scope.
    #[inline]
    pub fn SVGResourceQueryScope(&self) -> Option<SVGResourceQueryGuard<'a>> {
        let engine: &'a CoreLayoutEngine = self.engine;
        engine.GetLayoutTree().map(|tree| SVGResourceQueryGuard {
            _scope: SVGResourceQueryScope::new(tree),
        })
    }

    /// The whole resident LayoutObject tree. Test fixtures and diagnostics
    /// only; production consumers use the narrow facts above.
    #[inline]
    pub fn ResidentLayoutTree(&self) -> Option<&'a LayoutObjectTree> {
        let engine: &'a CoreLayoutEngine = self.engine;
        engine.GetLayoutTree()
    }
}

/// Mouse-coordinate sources.
impl<'a> PointerQueries<'a> {
    #[inline]
    pub fn MouseCoordinateLayout(&self, id: u64) -> Option<NativeMouseCoordinateLayout> {
        self.engine.MouseCoordinateLayout(id)
    }

    #[inline]
    pub fn MouseLayoutViewportScrollOffset(&self) -> Option<(f32, f32)> {
        self.engine.MouseLayoutViewportScrollOffset()
    }

    #[inline]
    pub fn MouseLayoutViewportScrollOffsetInt(&self) -> Option<(i32, i32)> {
        self.engine.MouseLayoutViewportScrollOffsetInt()
    }

    #[inline]
    pub fn RegisteredScrollableAreas(&self) -> Vec<NativeRegisteredScrollableArea> {
        self.engine.RegisteredScrollableAreas()
    }

    /// `NativeMouseCoordinateLayout::SimulatedClickRootFrameCenter`.
    #[inline]
    pub fn SimulatedClickRootFrameCenter(
        &self,
        object: &NativeMouseCoordinateLayout,
    ) -> Option<(PhysicalOffset, f32)> {
        object.SimulatedClickRootFrameCenter(self.engine)
    }

    /// `NativeMouseCoordinateLayout::AbsoluteBoundingBoxIncludingDescendants`.
    #[inline]
    pub fn AbsoluteBoundingBoxIncludingDescendants(
        &self,
        object: &NativeMouseCoordinateLayout,
    ) -> gfx::Rect {
        object.AbsoluteBoundingBoxIncludingDescendants(self.engine)
    }

    /// `NativeMouseCoordinateLayout::DragNodeImageGeometry`.
    #[inline]
    pub fn DragNodeImageGeometry(
        &self,
        object: &NativeMouseCoordinateLayout,
    ) -> Option<NativeDragNodeImageGeometry> {
        object.DragNodeImageGeometry(self.engine)
    }
}

/// Pointer scrollbar and resizer controls.
impl<'a> PointerQueries<'a> {
    #[inline]
    pub fn PointerScrollbarInteraction(
        &self,
        scrollbar: &Scrollbar,
        interaction: NativeScrollbarInteraction,
        is_popup: bool,
    ) -> Option<(NativePointerResizeArea, NativeOverlayVisibilityUpdate)> {
        self.engine
            .PointerScrollbarInteraction(scrollbar, interaction, is_popup)
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn PublishPointerMacScrollbarAnimation(
        &self,
        scrollbar: &Scrollbar,
        thumb: f32,
        track: f32,
        width: i32,
        thumb_dirty: bool,
        track_dirty: bool,
        hidden: Option<bool>,
    ) -> Option<NativeMacScrollbarAnimationUpdate> {
        self.engine.PublishPointerMacScrollbarAnimation(
            scrollbar,
            thumb,
            track,
            width,
            thumb_dirty,
            track_dirty,
            hidden,
        )
    }

    #[inline]
    pub fn PointerScrollbarTargetThumbGeometry(
        &self,
        scrollbar: &Scrollbar,
        target: gfx::Vector2dF,
    ) -> (i32, i32) {
        self.engine
            .PointerScrollbarTargetThumbGeometry(scrollbar, target)
    }

    #[inline]
    pub fn PointerScrollbarPointFromRootFrame(
        &self,
        scrollbar: &Scrollbar,
        point_in_containing_frame: gfx::Point,
    ) -> gfx::Point {
        self.engine
            .PointerScrollbarPointFromRootFrame(scrollbar, point_in_containing_frame)
    }

    #[inline]
    pub fn PointerResizeAreaForNode(&self, node: u64) -> Option<NativePointerResizeArea> {
        self.engine.PointerResizeAreaForNode(node)
    }

    #[inline]
    pub fn PointerResizeAreaInResizeMode(&self, area: &NativePointerResizeArea) -> bool {
        self.engine.PointerResizeAreaInResizeMode(area)
    }

    #[inline]
    pub fn PointerResizeAreaHasLayerAndLayoutBox(&self, area: &NativePointerResizeArea) -> bool {
        self.engine.PointerResizeAreaHasLayerAndLayoutBox(area)
    }

    #[inline]
    pub fn PointerResizeAreaContainsTouchPoint(
        &self,
        area: &NativePointerResizeArea,
        point: gfx::Point,
    ) -> bool {
        self.engine.PointerResizeAreaContainsTouchPoint(area, point)
    }

    #[inline]
    pub fn InitializePointerResizeTransform(
        &self,
        area: &NativePointerResizeArea,
        point: gfx::Point,
    ) -> gfx::Transform {
        self.engine.InitializePointerResizeTransform(area, point)
    }

    #[inline]
    pub fn PlanPointerResize(
        &self,
        area: &NativePointerResizeArea,
        point_in_containing_frame: gfx::Point,
        transform: &gfx::Transform,
    ) -> Option<NativeResizePlan> {
        self.engine
            .PlanPointerResize(area, point_in_containing_frame, transform)
    }
}
