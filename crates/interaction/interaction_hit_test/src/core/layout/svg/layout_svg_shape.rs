//! Source: core/layout/svg/layout_svg_shape.cc:530-565,571-602.
//! Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! SVG clip requests force PointerEventsHitRules to kFill; no stroke or
//! bounding-box fallback participates in this source specialization.
use super::transformed_hit_test_location::TransformedHitTestLocation;
use crate::core::layout::hit_test_location::HitTestLocation;
use crate::platform::geometry::path::PathHitTest;
use foundation::{
    blink_geometry::transforms::affine_transform::AffineTransform,
    native_path::{NativePath, ShapeFillGeometry},
    WindRule,
};

/// Borrow the same native Path/Style/clip identity used by the Layout object.
/// The nested clip query resolves the child's own canonical operation.
pub trait SVGClipShape {
    fn GetPath(&self) -> Option<&dyn NativePath>;
    fn IsShapeEmpty(&self) -> bool;
    fn FillGeometry(&self) -> ShapeFillGeometry;
    fn LocalToSVGParentTransform(&self) -> AffineTransform;
    fn ClipRule(&self) -> WindRule;
    fn HasClipPath(&self) -> bool;
    fn ClipPathHitTest(&self, local: &HitTestLocation) -> bool;
}
struct BorrowedPath<'a>(&'a dyn NativePath);
impl PathHitTest for BorrowedPath<'_> {
    fn CountVerbs(&self) -> i32 {
        self.0.CountVerbs()
    }
    fn Contains(&self, point: foundation::gfx::PointF) -> bool {
        self.0.Contains(point, WindRule::RULE_NONZERO)
    }
    fn Intersects(&self, quad: &foundation::gfx::QuadF) -> bool {
        self.0.Intersects(quad, WindRule::RULE_NONZERO)
    }
    fn ContainsWithWindRule(&self, point: foundation::gfx::PointF, rule: WindRule) -> bool {
        self.0.Contains(point, rule)
    }
    fn IntersectsWithWindRule(&self, quad: &foundation::gfx::QuadF, rule: WindRule) -> bool {
        self.0.Intersects(quad, rule)
    }
}
/// NodeAtPoint with kSVGClipContent, zero offset and kForeground. The caller
/// only consumes the hit boolean, so no result-node projection is required.
pub fn NodeAtPointForSvgClipContent(shape: &dyn SVGClipShape, location: &HitTestLocation) -> bool {
    if shape.IsShapeEmpty() {
        return false;
    }
    let Some(path) = shape.GetPath() else {
        return false;
    };
    let local = TransformedHitTestLocation::new(location, &shape.LocalToSVGParentTransform());
    if !local.IsValid() {
        return false;
    }
    let local = local.Location();
    if shape.HasClipPath() && !shape.ClipPathHitTest(local) {
        return false;
    }
    if !local.Intersects(&path.BoundingRect()) {
        return false;
    }
    match shape.FillGeometry() {
        ShapeFillGeometry::Path => {
            local.IntersectsPathWithWindRule(&BorrowedPath(path), shape.ClipRule())
        }
        ShapeFillGeometry::Rectangle(rect) => local.Intersects(&rect),
        ShapeFillGeometry::Ellipse { center, radii } => local.IntersectsEllipse(center, radii),
    }
}

/// General LayoutSVGShape source branch (cc:530-602). These inputs borrow
/// the native Layout shape; only StrokeGeometry may populate its Path cache.
pub trait SVGShape: SVGClipShape {
    fn PointerEvents(&self) -> foundation::EPointerEvents;
    fn Visible(&self) -> bool;
    fn HasFill(&self) -> bool;
    fn HasStroke(&self) -> bool;
    fn StrokeWidthIsZero(&self) -> bool;
    fn HasPaintServer(&self, stroke: bool) -> bool;
    fn FillRule(&self) -> WindRule;
    fn ObjectBoundingBox(&self) -> foundation::gfx::RectF;
    fn DecoratedBoundingBox(&self) -> foundation::gfx::RectF;
    fn FilteredDecoratedBoundingBox(&self) -> foundation::gfx::RectF;
    fn HitTestStrokeBoundingBox(&self) -> foundation::gfx::RectF;
    fn StrokeGeometry(&self) -> foundation::native_path::ShapeStrokeGeometry;
    fn Node(&self) -> Option<super::super::hit_test_result::NodeRef>;
}
pub fn NodeAtPoint<'r>(
    shape: &dyn SVGShape,
    result: &mut super::super::hit_test_result::HitTestResult<'r, ()>,
    location: &HitTestLocation,
    phase: super::super::hit_test_phase::HitTestPhase,
) -> bool {
    use super::super::{
        hit_test_phase::HitTestPhase,
        pointer_events_hit_rules::{EHitTesting, PointerEventsHitRules},
    };
    if phase != HitTestPhase::kForeground || shape.IsShapeEmpty() {
        return false;
    }
    let request = result.GetHitTestRequest();
    let rules = PointerEventsHitRules::new(
        EHitTesting::kSvgGeometryHitTesting,
        request,
        shape.PointerEvents(),
    );
    if rules.require_visible && !shape.Visible() {
        return false;
    }
    let local = TransformedHitTestLocation::new(location, &shape.LocalToSVGParentTransform());
    if !local.IsValid() {
        return false;
    }
    let local = local.Location();
    if shape.HasClipPath() && !shape.ClipPathHitTest(local) {
        return false;
    }
    let hit = if request.IsHitTestVisualOverflow() {
        local.Intersects(&shape.FilteredDecoratedBoundingBox())
    } else {
        (rules.can_hit_bounding_box && local.Intersects(&shape.ObjectBoundingBox()))
            || (rules.can_hit_stroke
                && (shape.HasStroke() || !rules.require_stroke)
                && StrokeContains(shape, local, rules.require_stroke))
            || (rules.can_hit_fill
                && (shape.HasFill() || !rules.require_fill)
                && FillContains(
                    shape,
                    local,
                    rules.require_fill,
                    if request.SvgClipContent() {
                        shape.ClipRule()
                    } else {
                        shape.FillRule()
                    },
                ))
    };
    if !hit {
        return false;
    }
    if result.InnerNode().is_none() {
        result.SetNodeAndPosition(
            shape.Node(),
            foundation::PhysicalOffset::FromPointFRound(&local.TransformedPoint()),
        );
    }
    result.AddNodeToListBasedTestResult(shape.Node(), local, &foundation::PhysicalRect::default())
        == super::super::hit_test_request::ListBasedHitTestBehavior::kStopHitTesting
}
fn FillContains(
    shape: &dyn SVGShape,
    local: &HitTestLocation,
    requires_fill: bool,
    rule: WindRule,
) -> bool {
    if !local.Intersects(&shape.ObjectBoundingBox())
        || (requires_fill && !shape.HasPaintServer(false))
    {
        return false;
    }
    match shape.FillGeometry() {
        ShapeFillGeometry::Rectangle(rect) => local.Intersects(&rect),
        ShapeFillGeometry::Ellipse { center, radii } => local.IntersectsEllipse(center, radii),
        ShapeFillGeometry::Path => shape
            .GetPath()
            .is_some_and(|path| local.IntersectsPathWithWindRule(&BorrowedPath(path), rule)),
    }
}
fn StrokeContains(shape: &dyn SVGShape, local: &HitTestLocation, requires_stroke: bool) -> bool {
    use foundation::native_path::ShapeStrokeGeometry;
    if shape.StrokeWidthIsZero() {
        return false;
    }
    if requires_stroke {
        if !local.Intersects(&shape.DecoratedBoundingBox()) || !shape.HasPaintServer(true) {
            return false;
        }
    } else if !local.Intersects(&shape.HitTestStrokeBoundingBox()) {
        return false;
    }
    match shape.StrokeGeometry() {
        ShapeStrokeGeometry::Rectangle { rect, width } => local.IntersectsStroke(&rect, width),
        ShapeStrokeGeometry::Circle {
            center,
            radius,
            width,
        } => local.IntersectsCircleStroke(center, radius, width),
        ShapeStrokeGeometry::Path {
            path,
            host_transform,
        } => {
            let transformed = TransformedHitTestLocation::WithInverseTag(
                local,
                &host_transform,
                super::transformed_hit_test_location::InverseTag::kDontComputeInverse,
            );
            transformed.IsValid()
                && transformed
                    .Location()
                    .IntersectsPath(&BorrowedPath(path.as_ref()))
        }
    }
}
