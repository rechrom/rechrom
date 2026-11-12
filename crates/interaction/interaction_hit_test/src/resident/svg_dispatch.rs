//! Native SVG virtual adapters. Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! layout_svg_shape.cc:530-602; layout_svg_container.cc:227-277;
//! layout_svg_text.cc:355-374; layout_svg_root.cc:505-525;
//! layout_svg_viewport_container.cc:94-113; svg_content_container.cc:181-198;
//! layout_svg_foreign_object.cc:207-254. Every cursor borrows its original owner.
use super::{native_dispatch, ResidentFragment, ResidentHitTestServices};
use crate::core::{
    layout::{
        hit_test_location::HitTestLocation,
        hit_test_phase::HitTestPhase,
        hit_test_request::ListBasedHitTestBehavior,
        hit_test_result::{HitTestResult, NodeRef},
        native_hit_test::{NativeHitTestRequiredPort, NativeHitTestResult},
        svg::{
            layout_svg_container::{self, SVGContainer},
            layout_svg_shape::{self, SVGClipShape, SVGShape},
            layout_svg_text::{self, SVGClipText},
            transformed_hit_test_location::TransformedHitTestLocation,
        },
    },
    paint::box_fragment_painter::PhysicalFragment,
};
use foundation::{
    blink_geometry::transforms::affine_transform::AffineTransform,
    gfx,
    layout_hit_test::LayoutHitTestKind as Kind,
    native_path::{NativePath, ShapeFillGeometry, ShapeStrokeGeometry},
    EPointerEvents, EVisibility, PhysicalOffset, PhysicalRect, WindRule,
};
use layoutng_assembly::layout_engine::{NativeHitTestCursor, PaintFragmentCursor};

pub struct ResidentSvgSource<'a> {
    pub source: NativeHitTestCursor<'a>,
    pub root: PaintFragmentCursor<'a>,
    pub services: &'a dyn ResidentHitTestServices<()>,
    text_fragment: Option<ResidentFragment<'a, ()>>,
}
impl<'a> ResidentSvgSource<'a> {
    pub fn new(
        source: NativeHitTestCursor<'a>,
        root: PaintFragmentCursor<'a>,
        services: &'a dyn ResidentHitTestServices<()>,
    ) -> Result<Self, NativeHitTestRequiredPort> {
        // LayoutSVGBlock delegates to the first resident physical fragment.
        let text_fragment = if source.Kind() == Kind::SvgText {
            let fragment = source.FirstPhysicalFragment().ok_or(
                NativeHitTestRequiredPort::MissingPaintFragment {
                    layout_identity: source.Identity(),
                },
            )?;
            Some(ResidentFragment::new(
                root.ForPhysicalFragment(fragment),
                services,
            ))
        } else {
            None
        };
        Ok(Self {
            source,
            root,
            services,
            text_fragment,
        })
    }
    fn NodeValue(&self) -> Option<NodeRef> {
        self.source
            .NodeForHitTestId()
            .and_then(|id| self.services.ResolveNode(id))
    }
}
impl SVGClipShape for ResidentSvgSource<'_> {
    fn GetPath(&self) -> Option<&dyn NativePath> {
        self.source.SvgPath()
    }
    fn IsShapeEmpty(&self) -> bool {
        self.source.SvgShapeEmpty()
    }
    fn FillGeometry(&self) -> ShapeFillGeometry {
        self.source.SvgFillGeometry()
    }
    fn LocalToSVGParentTransform(&self) -> AffineTransform {
        self.source.SvgLocalToParentTransform()
    }
    fn ClipRule(&self) -> WindRule {
        self.source.Style().ClipRule()
    }
    fn HasClipPath(&self) -> bool {
        self.source.HasClipPath()
    }
    fn ClipPathHitTest(&self, local: &HitTestLocation) -> bool {
        self.services.NativeClipPathHitTest(self.source, local)
    }
}
impl SVGShape for ResidentSvgSource<'_> {
    fn PointerEvents(&self) -> EPointerEvents {
        self.source.Style().UsedPointerEvents()
    }
    fn Visible(&self) -> bool {
        self.source.Style().Visibility() == EVisibility::kVisible
    }
    fn HasFill(&self) -> bool {
        self.source.Style().HasFill()
    }
    fn HasStroke(&self) -> bool {
        self.source.Style().HasStroke()
    }
    fn StrokeWidthIsZero(&self) -> bool {
        self.source.SvgStrokeWidth() == 0.
    }
    fn HasPaintServer(&self, stroke: bool) -> bool {
        self.source.SvgHasPaintServer(stroke)
    }
    fn FillRule(&self) -> WindRule {
        self.source.Style().FillRule()
    }
    fn ObjectBoundingBox(&self) -> gfx::RectF {
        self.source.SvgObjectBoundingBox()
    }
    fn DecoratedBoundingBox(&self) -> gfx::RectF {
        self.source.SvgDecoratedBoundingBox()
    }
    fn FilteredDecoratedBoundingBox(&self) -> gfx::RectF {
        self.source.SvgFilteredDecoratedBoundingBox()
    }
    fn HitTestStrokeBoundingBox(&self) -> gfx::RectF {
        self.source.SvgHitTestStrokeBounds()
    }
    fn StrokeGeometry(&self) -> ShapeStrokeGeometry {
        self.source.SvgStrokeGeometry()
    }
    fn Node(&self) -> Option<NodeRef> {
        self.NodeValue()
    }
}
impl SVGContainer for ResidentSvgSource<'_> {
    fn LocalToSVGParentTransform(&self) -> AffineTransform {
        self.source.SvgLocalToParentTransform()
    }
    fn HasClipPath(&self) -> bool {
        self.source.HasClipPath()
    }
    fn ClipPathHitTest(&self, local: &HitTestLocation) -> bool {
        self.services.NativeClipPathHitTest(self.source, local)
    }
    fn ChildPaintBlockedByDisplayLock(&self) -> bool {
        self.source.ChildPaintBlockedByDisplayLock()
    }
    fn PointerEventsBoundingBox(&self) -> bool {
        self.source.Style().UsedPointerEvents() == EPointerEvents::kBoundingBox
    }
    fn ObjectBoundingBox(&self) -> gfx::RectF {
        self.source.SvgObjectBoundingBox()
    }
    fn IsObjectBoundingBoxValid(&self) -> bool {
        self.source.SvgObjectBoundsValid()
    }
    fn FilteredDecoratedBoundingBox(&self) -> gfx::RectF {
        self.source.SvgFilteredDecoratedBoundingBox()
    }
    fn Node(&self) -> Option<NodeRef> {
        self.NodeValue()
    }
    fn HitTestChildren<'r>(
        &self,
        result: &mut HitTestResult<'r, ()>,
        location: &HitTestLocation,
        phase: HitTestPhase,
    ) -> bool {
        // Errors are retained by query services before any result is published.
        self.services.RecordNativeHitTestResult(HitTestChildren(
            self.source,
            self.root,
            self.services,
            result,
            location,
            phase,
        ))
    }
}
impl SVGClipText<()> for ResidentSvgSource<'_> {
    fn LocalToSVGParentTransform(&self) -> AffineTransform {
        self.source.SvgLocalToParentTransform()
    }
    fn HasClipPath(&self) -> bool {
        self.source.HasClipPath()
    }
    fn ClipPathHitTest(&self, local: &HitTestLocation) -> bool {
        self.services.NativeClipPathHitTest(self.source, local)
    }
    fn PhysicalFragments(&self) -> Box<dyn Iterator<Item = &dyn PhysicalFragment<()>> + '_> {
        Box::new(
            self.text_fragment
                .iter()
                .map(|fragment| fragment as &dyn PhysicalFragment<()>),
        )
    }
}
pub fn NodeAtPoint(
    source: NativeHitTestCursor<'_>,
    root: PaintFragmentCursor<'_>,
    services: &dyn ResidentHitTestServices<()>,
    result: &mut HitTestResult<'_, ()>,
    location: &HitTestLocation,
    phase: HitTestPhase,
) -> NativeHitTestResult {
    let adapter = ResidentSvgSource::new(source, root, services)?;
    Ok(match source.Kind() {
        Kind::SvgShape => layout_svg_shape::NodeAtPoint(&adapter, result, location, phase),
        Kind::SvgText => layout_svg_text::NodeAtPoint(&adapter, result, location, phase),
        Kind::SvgContainer | Kind::SvgViewportContainer => {
            if source.Kind() == Kind::SvgViewportContainer
                && source.SvgOverflowHidden()
                && !result.GetHitTestRequest().IsHitTestVisualOverflow()
            {
                let (viewport, transform) = source.SvgViewport();
                let local = TransformedHitTestLocation::new(location, &transform);
                if !local.IsValid() || !local.Location().Intersects(&viewport) {
                    return Ok(false);
                }
            }
            layout_svg_container::NodeAtPoint(&adapter, result, location, phase)
        }
        kind => return Err(NativeHitTestRequiredPort::Special(kind)),
    })
}
pub fn HitTestChildren(
    source: NativeHitTestCursor<'_>,
    root: PaintFragmentCursor<'_>,
    services: &dyn ResidentHitTestServices<()>,
    result: &mut HitTestResult<'_, ()>,
    location: &HitTestLocation,
    phase: HitTestPhase,
) -> NativeHitTestResult {
    for child in source.ReverseChildren() {
        let hit = if child.IsSvgForeignObject() {
            ForeignObjectNodeAtPoint(child, services, result, location)?
        } else {
            native_dispatch::DispatchNativeNodeAtPoint(
                root,
                child,
                services,
                result,
                location,
                PhysicalOffset::default(),
                phase,
            )?
        };
        if hit {
            return Ok(true);
        }
    }
    Ok(false)
}
pub fn RootHitTestChildren(
    source: NativeHitTestCursor<'_>,
    root: PaintFragmentCursor<'_>,
    services: &dyn ResidentHitTestServices<()>,
    result: &mut HitTestResult<'_, ()>,
    location: &HitTestLocation,
    offset: PhysicalOffset,
    phase: HitTestPhase,
) -> NativeHitTestResult {
    let border_box = HitTestLocation::WithOffset(location, -offset);
    let local = TransformedHitTestLocation::new(&border_box, &source.SvgRootContentTransform());
    if !local.IsValid() {
        return Ok(false);
    }
    HitTestChildren(source, root, services, result, local.Location(), phase)
}
fn ForeignObjectNodeAtPoint(
    source: NativeHitTestCursor<'_>,
    services: &dyn ResidentHitTestServices<()>,
    result: &mut HitTestResult<'_, ()>,
    location: &HitTestLocation,
) -> NativeHitTestResult {
    let local = TransformedHitTestLocation::new(location, &source.SvgLocalToParentTransform());
    if !local.IsValid() {
        return Ok(false);
    }
    let local = local.Location();
    if result.GetHitTestRequest().IsHitTestVisualOverflow()
        && local.Intersects(&source.SvgFilteredDecoratedBoundingBox())
    {
        let node = source
            .NodeForHitTestId()
            .and_then(|id| services.ResolveNode(id));
        if result.InnerNode().is_none() {
            result.SetNodeAndPosition(
                node.clone(),
                PhysicalOffset::FromPointFRound(&local.TransformedPoint()),
            );
        }
        if result.AddNodeToListBasedTestResult(node, local, &PhysicalRect::default())
            == ListBasedHitTestBehavior::kStopHitTesting
        {
            return Ok(true);
        }
    }
    let local = HitTestLocation::WithOffset(local, -source.PhysicalLocation().unwrap_or_default());
    let mut layer_result = HitTestResult::new(result.GetHitTestRequest(), &local);
    let hit = services.SvgForeignObjectLayerHitTest(source, &mut layer_result, &local)?;
    let original = *result.PointInInnerNodeFrame();
    if result.GetHitTestRequest().ListBased() {
        result.Append(&layer_result);
    } else {
        result.Assign(&layer_result);
    }
    result.SetPointInInnerNodeFrame(original);
    Ok(hit)
}
