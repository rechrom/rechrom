//! Resident ports for Paint's source ClipPathClipper, without a clip graph.
//! Chromium clip_path_clipper.cc:237-257,364-400,461-532;
//! layout_svg_resource_clipper.cc:46-104,237-281; svg_use_element.cc:545-560.
use super::{svg_dispatch, ResidentHitTestServices};
use crate::{
    core::layout::{
        hit_test_location::HitTestLocation,
        hit_test_phase::HitTestPhase,
        hit_test_request::HitTestRequest,
        hit_test_result::{HitTestResult, HitTestResultRequest},
        svg::transformed_hit_test_location::TransformedHitTestLocation,
    },
    platform::geometry::path::PathHitTest,
};
use foundation::{
    blink_geometry::transforms::affine_transform::AffineTransform, gfx,
    layout_hit_test::LayoutHitTestKind as Kind,
};
use layoutng_assembly::layout_engine::{NativeHitTestCursor, PaintFragmentCursor};
use layoutng_style::style::{
    clip_path_operation::ClipPathOperation, computed_style::ComputedStyle,
    computed_style_constants::GeometryBox,
    reference_clip_path_operation::ReferenceClipPathOperation,
};
use paint::{
    clip_path_clipper::{
        ClipContentKind, ClipPathContent, ClipPathGeometry, ClipPathLocation, ClipPathReference,
        ClipPathResource,
    },
    path_hit_test::{PaintPathHitTest, PaintPathHitTestFactory},
};

struct BorrowedPaintPath<'a>(&'a dyn PaintPathHitTest);
impl PathHitTest for BorrowedPaintPath<'_> {
    fn CountVerbs(&self) -> i32 {
        self.0.count_verbs()
    }
    fn Contains(&self, point: gfx::PointF) -> bool {
        self.0.contains(point, None)
    }
    fn Intersects(&self, quad: &gfx::QuadF) -> bool {
        self.0.intersects(quad, None)
    }
    fn IntersectsWithWindRule(&self, quad: &gfx::QuadF, rule: foundation::WindRule) -> bool {
        self.0.intersects(quad, Some(super::PathFill(rule)))
    }
    fn ContainsWithWindRule(&self, point: gfx::PointF, rule: foundation::WindRule) -> bool {
        self.0.contains(point, Some(super::PathFill(rule)))
    }
}
impl ClipPathLocation for HitTestLocation {
    fn IntersectsPath(&self, path: &dyn PaintPathHitTest) -> bool {
        HitTestLocation::IntersectsPath(self, &BorrowedPaintPath(path))
    }
    fn IntersectsContouredRect(
        &self,
        rect: &paint::contoured_rect::ContouredRect,
        factory: &dyn PaintPathHitTestFactory,
    ) -> bool {
        HitTestLocation::IntersectsContouredRect(
            self,
            &super::ResidentContour {
                contour: *rect,
                paths: factory,
            },
        )
    }
    fn TransformedToLocal(&self, transform: &AffineTransform) -> Option<Self> {
        let local = TransformedHitTestLocation::new(self, transform);
        local.IsValid().then(|| local.Location().clone())
    }
}
#[derive(Clone, Copy)]
pub struct ResidentClipGeometry<'a> {
    pub source: NativeHitTestCursor<'a>,
    pub root: PaintFragmentCursor<'a>,
    pub services: &'a dyn ResidentHitTestServices<()>,
}
impl ClipPathGeometry<HitTestLocation> for ResidentClipGeometry<'_> {
    fn Operation(&self) -> Option<&dyn ClipPathOperation> {
        self.source
            .Style()
            .ClipPath()
            .and_then(|operation| unsafe { operation.as_ref() })
    }
    fn IsSvgChild(&self) -> bool {
        self.source.IsSvgChild()
    }
    fn IsSvgForeignObject(&self) -> bool {
        self.source.IsSvgForeignObject()
    }
    fn EffectiveZoom(&self) -> f32 {
        self.source.Style().EffectiveZoom()
    }
    fn SvgReferenceBoxForEffects(&self, geometry: GeometryBox) -> gfx::RectF {
        self.source.SvgReferenceBox(geometry)
    }
    fn NonSvgReferenceBox(&self, geometry: GeometryBox) -> gfx::RectF {
        gfx::RectF::from(self.source.PaintIdentity().ClipPathReferenceBox(geometry))
    }
    fn ContouredBorderReferenceBox(
        &self,
        geometry: GeometryBox,
    ) -> paint::contoured_rect::ContouredRect {
        let owner = self.source.PaintIdentity();
        let outsets = owner.ReferenceBoxBorderBoxOutsets(geometry);
        paint::contoured_rect::border::GeometryBoxReference(
            self.source.Style(),
            owner.ClipPathBorderBox(),
            geometry,
            gfx::OutsetsF::new(
                outsets.top.ToFloat(),
                outsets.right.ToFloat(),
                outsets.bottom.ToFloat(),
                outsets.left.ToFloat(),
            ),
        )
    }
    fn ResolveReference(
        &self,
        operation: &ReferenceClipPathOperation,
    ) -> ClipPathReference<'_, HitTestLocation> {
        match self.source.SvgResolveClip(operation) {
            Some(source) if !source.LockedAncestorPreventingLayout() => {
                ClipPathReference::Resource(Box::new(ResidentClipGeometry { source, ..*self }))
            }
            _ => ClipPathReference::Missing,
        }
    }
}
impl ClipPathResource<HitTestLocation> for ResidentClipGeometry<'_> {
    fn Geometry(&self) -> &dyn ClipPathGeometry<HitTestLocation> {
        self
    }
    fn UsesObjectBoundingBoxUnits(&self) -> bool {
        self.source.SvgClipUnits()
    }
    fn ElementTransformIncludingMotion(&self) -> AffineTransform {
        self.source.SvgClipTransform()
    }
    fn Children(
        &self,
    ) -> Box<dyn Iterator<Item = Box<dyn ClipPathContent<HitTestLocation> + '_>> + '_> {
        Box::new(
            std::iter::successors(self.source.FirstChild(), |child| child.NextSibling()).map(
                |source| {
                    Box::new(ResidentClipGeometry { source, ..*self })
                        as Box<dyn ClipPathContent<HitTestLocation>>
                },
            ),
        )
    }
}
impl ClipPathContent<HitTestLocation> for ResidentClipGeometry<'_> {
    fn IsUseElement(&self) -> bool {
        self.source.SvgIsUse()
    }
    fn IsGraphicsElement(&self) -> bool {
        self.source.IsSvgChild()
    }
    fn LayoutStyle(&self) -> Option<&ComputedStyle> {
        Some(self.source.Style())
    }
    fn LockedAncestorPreventingLayout(&self) -> bool {
        self.source.LockedAncestorPreventingLayout()
    }
    fn LayoutKind(&self) -> ClipContentKind {
        match self.source.Kind() {
            Kind::SvgShape => ClipContentKind::Shape,
            Kind::SvgText => ClipContentKind::Text,
            _ => ClipContentKind::Other,
        }
    }
    fn VisibleTargetGraphicsElementForClipping(
        &self,
    ) -> Option<Box<dyn ClipPathContent<HitTestLocation> + '_>> {
        self.source.SvgUseClipTarget().map(|source| {
            Box::new(ResidentClipGeometry { source, ..*self })
                as Box<dyn ClipPathContent<HitTestLocation>>
        })
    }
    fn NodeAtPointForSvgClipContent(&self, location: &HitTestLocation) -> bool {
        let request: HitTestResultRequest<'_, ()> =
            HitTestRequest::new(HitTestResultRequest::<()>::kSVGClipContent, None, None);
        let mut result = HitTestResult::new(&request, location);
        self.services
            .RecordNativeHitTestResult(svg_dispatch::NodeAtPoint(
                self.source,
                self.root,
                self.services,
                &mut result,
                location,
                HitTestPhase::kForeground,
            ))
    }
}
