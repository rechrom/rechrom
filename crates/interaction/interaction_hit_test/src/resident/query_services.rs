//! Borrowed production services for the translated PaintLayer/fragment walk.
//! Algorithms remain with their source owners; an unavailable virtual port
//! invalidates the entire query before its tentative result can be published.
use super::*;
use crate::core::layout::native_hit_test::{NativeHitTestRequiredPort, NativeHitTestResult};
use foundation::{layout_hit_test::LayoutHitTestKind as Kind, DynamicTo};
use layoutng_assembly::{
    internal::{
        layout_box_model_object::LayoutBoxModelObject, layout_node_metadata::Node,
        layout_scrollable_area::OverflowControlHit,
    },
    layout_engine::NativeHitTestCursor,
};
use std::cell::Cell;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResidentQueryError {
    Native(NativeHitTestRequiredPort),
    ClipPath,
    OverflowControl,
    TextCombine,
    Geometry(ResidentCursorGeometryError),
    MissingLayerOrder,
    MissingRootPaintLayer,
    MissingReplacedPaintLayer,
}

/// DOM supplies identities and transition participation from its live owners.
pub trait ResidentQueryNodes {
    fn EmbeddedChildFrameView(
        &self,
        owner: &Node,
    ) -> Result<
        crate::core::layout::layout_embedded_content_hit_test::ChildFrameView<'_>,
        NativeHitTestRequiredPort,
    >;
    fn ResolveNode(&self, id: u64) -> Option<NodeRef>;
    fn ResolveNativeNode(&self, node: &Node) -> Option<NodeRef>;
    fn DocumentElement(&self) -> Option<NodeRef>;
    fn TransitionFacts(&self, cursor: PaintFragmentCursor<'_>) -> (bool, bool);
}
pub trait ResidentQueryViewport {
    fn RootFrameToVisualViewport(&self, point: gfx::Point) -> gfx::Point;
    fn LocalFramePointToRootFrame(&self, point: gfx::Point) -> gfx::Point {
        point
    }
    fn LocalFrameRectToRootFrame(&self, rect: gfx::Rect) -> gfx::Rect {
        rect
    }
}

pub struct ResidentQueryServices<'a> {
    pub paint: &'a PaintEngine,
    pub root: PaintFragmentCursor<'a>,
    pub nodes: &'a dyn ResidentQueryNodes,
    pub viewport: &'a dyn ResidentQueryViewport,
    pub geometry: ResidentCursorGeometry<'a>,
    error: Cell<Option<ResidentQueryError>>,
}
impl<'a> ResidentQueryServices<'a> {
    pub fn new(
        paint: &'a PaintEngine,
        root: PaintFragmentCursor<'a>,
        nodes: &'a dyn ResidentQueryNodes,
        viewport: &'a dyn ResidentQueryViewport,
        paths: &'a dyn paint::path_hit_test::PaintPathHitTestFactory,
    ) -> Self {
        Self {
            paint,
            root,
            nodes,
            viewport,
            geometry: ResidentCursorGeometry { paths },
            error: Cell::new(None),
        }
    }
    fn Reject(&self, error: ResidentQueryError) {
        if self.error.get().is_none() {
            self.error.set(Some(error));
        }
    }
    fn Result(&self, result: NativeHitTestResult) -> bool {
        result.unwrap_or_else(|error| {
            self.Reject(ResidentQueryError::Native(error));
            false
        })
    }
    fn Object<'b>(&self, cursor: PaintFragmentCursor<'b>) -> &'b LayoutObject {
        let object = match cursor.Cursor() {
            FragmentCursor::Physical { fragment, .. } => fragment.GetSelfOrContainerLayoutObject(),
            FragmentCursor::Inline { item, .. } => item.GetLayoutObject(),
        };
        unsafe { object.as_ref() }.expect("resident fragment retains LayoutObject")
    }
    fn Layer<'b>(&'b self, cursor: PaintFragmentCursor<'b>) -> Option<ResidentLayer<'b>> {
        ResidentLayer::new(self.paint, self.root, cursor, self)
    }
    fn Path(
        &self,
        cursor: PaintFragmentCursor<'_>,
        rect: PhysicalRect,
    ) -> Box<dyn PathHitTest + '_> {
        self.geometry
            .BorderShapeOuterPath(cursor, rect)
            .unwrap_or_else(|error| {
                self.Reject(ResidentQueryError::Geometry(error));
                Box::new(RejectedPath)
            })
    }
}
// A failed geometry operation is not a no-hit: QueryError prevents publication.
struct RejectedPath;
impl PathHitTest for RejectedPath {
    fn CountVerbs(&self) -> i32 {
        1
    }
    fn Intersects(&self, _: &gfx::QuadF) -> bool {
        false
    }
    fn Contains(&self, _: gfx::PointF) -> bool {
        false
    }
    fn IntersectsWithWindRule(&self, _: &gfx::QuadF, _: foundation::WindRule) -> bool {
        false
    }
    fn ContainsWithWindRule(&self, _: gfx::PointF, _: foundation::WindRule) -> bool {
        false
    }
}
struct ResidentInlineRegion(Box<dyn paint::path_hit_test::PaintHitTestRegion>);
struct ResidentScrollbar(foundation::Persistent<layoutng_assembly::internal::scrollbar::Scrollbar>);
impl crate::core::layout::hit_test_result::HitTestScrollbar for ResidentScrollbar {
    fn NativeScrollbarOwner(
        &self,
    ) -> Option<&foundation::Persistent<layoutng_assembly::internal::scrollbar::Scrollbar>> {
        Some(&self.0)
    }
}
impl crate::core::layout::hit_test_result::HitTestRegion for ResidentInlineRegion {
    fn Contains(&self, rect: &gfx::Rect) -> bool {
        self.0.contains(rect)
    }
}
impl crate::core::layout::layout_inline::InlineHitTestRegion for ResidentInlineRegion {
    fn Union(&mut self, rect: gfx::Rect) {
        self.0.union(rect);
    }
    fn AsRegion(&self) -> &dyn crate::core::layout::hit_test_result::HitTestRegion {
        self
    }
}
impl ResidentHitTestServices<()> for ResidentQueryServices<'_> {
    fn RecordNativeHitTestResult(&self, result: NativeHitTestResult) -> bool {
        self.Result(result)
    }
    fn NativeClipPathHitTest(
        &self,
        source: NativeHitTestCursor<'_>,
        location: &HitTestLocation,
    ) -> bool {
        let geometry = super::svg_clip::ResidentClipGeometry {
            source,
            root: self.root,
            services: self,
        };
        paint::clip_path_clipper::HitTest(&geometry, location, self.geometry.paths).unwrap_or_else(
            |_| {
                self.Reject(ResidentQueryError::ClipPath);
                false
            },
        )
    }
    fn SvgForeignObjectLayerHitTest(
        &self,
        source: NativeHitTestCursor<'_>,
        result: &mut HitTestResult<'_, ()>,
        location: &HitTestLocation,
    ) -> NativeHitTestResult {
        let cursor = self.root.ForPaintObject(source.PaintIdentity()).ok_or(
            NativeHitTestRequiredPort::MissingPaintFragment {
                layout_identity: source.Identity(),
            },
        )?;
        let layer = self
            .Layer(cursor)
            .ok_or(NativeHitTestRequiredPort::SvgForeignObjectLayer {
                layout_identity: source.Identity(),
            })?;
        Ok(crate::core::paint::paint_layer_hit_test::HitTest(
            &layer,
            location,
            result,
            &PhysicalRect::from(foundation::InfiniteIntRect()),
        ))
    }
    fn EmbeddedChildFrameView(
        &self,
        owner: &Node,
    ) -> Result<
        crate::core::layout::layout_embedded_content_hit_test::ChildFrameView<'_>,
        NativeHitTestRequiredPort,
    > {
        self.nodes.EmbeddedChildFrameView(owner)
    }
    fn NewInlineHitTestRegion(
        &self,
    ) -> Box<dyn crate::core::layout::layout_inline::InlineHitTestRegion> {
        Box::new(ResidentInlineRegion(self.geometry.paths.new_region()))
    }
    fn MayIntersect(
        &self,
        cursor: PaintFragmentCursor<'_>,
        result: &HitTestResult<'_, ()>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
    ) -> bool {
        let object = self.Object(cursor);
        let source = NativeHitTestCursor::ForObject(object);
        let port = match source.Kind() {
            Kind::Block
            | Kind::Replaced
            | Kind::Image
            | Kind::EmbeddedContent
            | Kind::TextControlSingleLine
            | Kind::TextControlMultiLine
            | Kind::Base
            | Kind::Hidden
            | Kind::Inline
            | Kind::SvgRoot
            | Kind::SvgShape
            | Kind::SvgText
            | Kind::SvgContainer
            | Kind::SvgViewportContainer => None,
            kind => Some(NativeHitTestRequiredPort::Special(kind)),
        };
        if let Some(port) = port {
            self.Reject(ResidentQueryError::Native(port));
            return false;
        }
        // PhysicalBoxFragment::MayIntersect only delegates for a LayoutBox.
        !object.IsBox() || native_dispatch::MayIntersect(source, result, location, offset)
    }
    fn ShouldPaintReplacedNormalFlowInline(&self, cursor: PaintFragmentCursor<'_>) -> bool {
        let object = self.Object(cursor);
        // BoxFragmentPainter asks this question for every self-painting inline
        // child before deciding whether it should be delegated to PaintLayer.
        // Most of those children are ordinary controls and deliberately have
        // no fragment-local resident layer node. Chromium's decision is made
        // from LayoutObject scalar facts; do not require a PaintLayer merely
        // to discover that the special replaced-inline path is disabled.
        ShouldPaintReplacedNormalFlowInlineLayout(
            ReplacedNormalFlowInlineFacts {
                svg_foreign_object: object.IsSVGForeignObject(),
                replaced_normal_flow_stacking_context: object
                    .IsReplacedNormalFlowStackingContext(object.StyleRef()),
                floating: object.IsFloating(),
                fragmented: object.IsFragmented(),
            },
            LayerHitTestFeatures {
                unbounded_element: foundation::RuntimeEnabledFeatures::UnboundedElementEnabled(),
                border_radius_for_stacking_context: foundation::RuntimeEnabledFeatures::HitTestBorderRadiusForStackingContextEnabled(),
                container_transform_for_preserve_3d: foundation::RuntimeEnabledFeatures::HitTestContainerTransformStateForPreserve3dEnabled(),
                stacking_context_is_not_stacked: foundation::RuntimeEnabledFeatures::StackingContextIsNotStackedEnabled(),
                replaced_normal_flow_stacking_inline_paint: foundation::RuntimeEnabledFeatures::ReplacedNormalFlowStackingInlinePaintEnabled(),
                canvas_draw_element: object.CanvasDrawElementEnabledForLayout(),
            },
        )
    }
    fn VisualOverflowRectIncludingFilters(&self, cursor: PaintFragmentCursor<'_>) -> PhysicalRect {
        NativeHitTestCursor::ForObject(self.Object(cursor)).VisualOverflowRectIncludingFilters()
    }
    fn ResolveNode(&self, id: u64) -> Option<NodeRef> {
        self.nodes.ResolveNode(id)
    }
    fn ResolveNativeNode(&self, node: &Node) -> Option<NodeRef> {
        self.nodes.ResolveNativeNode(node)
    }
    fn DocumentElement(&self) -> Option<NodeRef> {
        self.nodes.DocumentElement()
    }
    fn ObjectBoundingBox(&self, cursor: PaintFragmentCursor<'_>) -> gfx::RectF {
        self.Reject(ResidentQueryError::Native(
            NativeHitTestRequiredPort::Special(
                NativeHitTestCursor::ForObject(self.Object(cursor)).Kind(),
            ),
        ));
        gfx::RectF::default()
    }
    fn ContouredBorder(
        &self,
        cursor: PaintFragmentCursor<'_>,
        rect: PhysicalRect,
        inner: bool,
        sides: bool,
    ) -> Box<dyn ContouredRectHitTest + '_> {
        self.geometry.ContouredBorder(cursor, rect, inner, sides)
    }
    fn BorderShapeOuterPath(
        &self,
        cursor: PaintFragmentCursor<'_>,
        rect: PhysicalRect,
    ) -> Box<dyn PathHitTest + '_> {
        self.Path(cursor, rect)
    }
    fn ApplyFiltersToRect(
        &self,
        cursor: PaintFragmentCursor<'_>,
        rect: PhysicalRect,
    ) -> PhysicalRect {
        unsafe { DynamicTo::<LayoutBoxModelObject>(self.Object(cursor)).as_ref() }
            .expect("filtered fragment has a box model owner")
            .ApplyFiltersToRect(&rect)
    }
    fn TextCombineBounds(
        &self,
        cursor: PaintFragmentCursor<'_>,
        _: &dyn FragmentItem<()>,
        _: PhysicalOffset,
    ) -> Option<PhysicalRect> {
        if self.Object(cursor).IsLayoutTextCombine() {
            self.Reject(ResidentQueryError::TextCombine);
        }
        None
    }
    fn LegacyNodeAtPoint(
        &self,
        cursor: PaintFragmentCursor<'_>,
        result: &mut HitTestResult<'_, ()>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
        phase: HitTestPhase,
    ) -> bool {
        self.Result(native_dispatch::LegacyNodeAtPoint(
            cursor, self, result, location, offset, phase,
        ))
    }
    fn LegacyHitTestAllPhases(
        &self,
        cursor: PaintFragmentCursor<'_>,
        result: &mut HitTestResult<'_, ()>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
    ) -> bool {
        self.Result(native_dispatch::LegacyHitTestAllPhases(
            cursor, self, result, location, offset,
        ))
    }
    fn HitTestReplacedNormalFlowInline(
        &self,
        cursor: PaintFragmentCursor<'_>,
        result: &mut HitTestResult<'_, ()>,
        location: &HitTestLocation,
    ) -> bool {
        let Some(layer) = self.Layer(cursor) else {
            self.Reject(ResidentQueryError::MissingReplacedPaintLayer);
            return false;
        };
        crate::core::paint::paint_layer_hit_test::HitTestReplacedNormalFlowInline(
            &layer, result, location,
        )
    }
    fn HitTestOverflowControl(
        &self,
        cursor: PaintFragmentCursor<'_>,
        result: &mut HitTestResult<'_, ()>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
    ) -> bool {
        if !NativeHitTestCursor::ForObject(self.Object(cursor))
            .VisibleToHitTestRequest(result.GetHitTestRequest().IgnorePointerEventsNone())
        {
            return false;
        }
        let Some(box_) =
            (unsafe { DynamicTo::<LayoutBoxModelObject>(self.Object(cursor)).as_ref() })
        else {
            return false;
        };
        let Some(area) = (unsafe { box_.GetScrollableArea().as_ref() }) else {
            return false;
        };
        // Source paint_layer_scrollable_area.cc:2243-2280. Preserve the
        // concrete source control owner instead of discarding its identity.
        match area
            .HitTestOverflowControls(&foundation::ToRoundedPoint(&(*location.Point() - offset)))
        {
            OverflowControlHit::None => return false,
            OverflowControlHit::Resizer => result.SetIsOverResizer(true),
            OverflowControlHit::Corner => result.SetIsOverScrollCorner(true),
            OverflowControlHit::Scrollbar(scrollbar) => {
                result.SetScrollbar(Some(std::rc::Rc::new(ResidentScrollbar(
                    foundation::Persistent::from_ptr(scrollbar),
                ))))
            }
        }
        true
    }
    fn HitTestCulledInline(
        &self,
        cursor: PaintFragmentCursor<'_>,
        parent: &dyn CulledInlineLayout,
        result: &mut HitTestResult<'_, ()>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
        inline_cursor: InlineCursor<'_, ()>,
    ) -> bool {
        self.Result(native_dispatch::HitTestCulledInline(
            cursor,
            self,
            parent,
            result,
            location,
            offset,
            inline_cursor,
        ))
    }
}
impl ResidentLayerServices for ResidentQueryServices<'_> {
    fn QueryError(&self) -> Option<ResidentQueryError> {
        self.error.get()
    }
    fn RootFrameToVisualViewport(&self, point: gfx::Point) -> gfx::Point {
        self.viewport.RootFrameToVisualViewport(point)
    }
    fn ClipPathHitTest(&self, cursor: PaintFragmentCursor<'_>, location: &HitTestLocation) -> bool {
        self.NativeClipPathHitTest(
            NativeHitTestCursor::ForObject(self.Object(cursor)),
            location,
        )
    }
    fn TransitionFacts(&self, cursor: PaintFragmentCursor<'_>) -> (bool, bool) {
        self.nodes.TransitionFacts(cursor)
    }
    fn BorderShapePath(
        &self,
        cursor: PaintFragmentCursor<'_>,
        rect: PhysicalRect,
    ) -> Option<Box<dyn PathHitTest + '_>> {
        Some(self.Path(cursor, rect))
    }
}
