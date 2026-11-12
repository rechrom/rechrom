//! Source ClipPathClipper hit testing and LayoutSVGResourceClipper traversal.
//! Source: core/paint/clip_path_clipper.cc:237-257,364-400,461-532;
//! core/layout/svg/layout_svg_resource_clipper.cc:250-281.
//! Views borrow compiled Style and the existing resource/fragment owners. The
//! source resolver retains loading, TreeScope, cycle and display-lock policy.
use foundation::{blink_geometry::transforms::affine_transform::AffineTransform, gfx::RectF};
use layoutng_style::style::{
    clip_path_operation::{ClipPathOperation, OperationType},
    computed_style_constants::GeometryBox,
    geometry_box_clip_path_operation::GeometryBoxClipPathOperation,
    reference_clip_path_operation::ReferenceClipPathOperation,
    shape_clip_path_operation::ShapeClipPathOperation,
};
use paint_model::{
    basic_shape_path::InvalidBasicShape,
    path_hit_test::{PaintPathHitTest, PaintPathHitTestFactory},
};

/// Interaction lends its existing HitTestLocation projection. Transforming
/// uses its source TransformedHitTestLocation, preserving rect-based queries.
pub trait ClipPathLocation: Clone {
    fn IntersectsPath(&self, path: &dyn PaintPathHitTest) -> bool;
    fn IntersectsContouredRect(
        &self,
        rect: &paint_model::contoured_rect::ContouredRect,
        factory: &dyn PaintPathHitTestFactory,
    ) -> bool;
    fn TransformedToLocal(&self, transform: &AffineTransform) -> Option<Self>;
}
pub enum ClipPathReference<'a, L: ClipPathLocation> {
    Loading,
    Missing,
    Resource(Box<dyn ClipPathResource<L> + 'a>),
}
/// Only live reference-box facts and the canonical reference-resource query
/// cross this boundary. Path construction/traversal remain in Paint.
pub trait ClipPathGeometry<L: ClipPathLocation> {
    fn Operation(&self) -> Option<&dyn ClipPathOperation>;
    fn IsSvgChild(&self) -> bool;
    fn IsSvgForeignObject(&self) -> bool;
    fn EffectiveZoom(&self) -> f32;
    /// Exact unzoomed SVGResources::ReferenceBoxForEffects, foreign-object
    /// quirk disabled. SVG geometry stays with its native geometry owner.
    fn SvgReferenceBoxForEffects(&self, geometry_box: GeometryBox) -> RectF;
    /// Native BorderBoxRect plus GeometryBoxUtils used-value outsets.
    fn NonSvgReferenceBox(&self, geometry_box: GeometryBox) -> RectF;
    fn ContouredBorderReferenceBox(
        &self,
        geometry_box: GeometryBox,
    ) -> paint_model::contoured_rect::ContouredRect;
    /// Source ResolveElementReference clears the resource invalidation mask,
    /// then rejects a layout-locked resource. No path/bounds substitute.
    fn ResolveReference(&self, operation: &ReferenceClipPathOperation) -> ClipPathReference<'_, L>;
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ClipContentKind {
    Shape,
    Text,
    Other,
}
/// Source element/layout identity facts; Style is the same borrowed owner.
pub trait ClipPathContent<L: ClipPathLocation> {
    fn IsUseElement(&self) -> bool;
    fn IsGraphicsElement(&self) -> bool;
    fn LayoutStyle(&self) -> Option<&layoutng_style::style::computed_style::ComputedStyle>;
    fn LockedAncestorPreventingLayout(&self) -> bool;
    fn LayoutKind(&self) -> ClipContentKind;
    fn VisibleTargetGraphicsElementForClipping(&self) -> Option<Box<dyn ClipPathContent<L> + '_>>;

    /// Exact source NodeAtPoint: kSVGClipContent request, zero accumulated
    /// offset, kForeground phase. The child keeps its own Style/clip owners.
    fn NodeAtPointForSvgClipContent(&self, location: &L) -> bool;
}
pub trait ClipPathResource<L: ClipPathLocation> {
    fn Geometry(&self) -> &dyn ClipPathGeometry<L>;
    fn UsesObjectBoundingBoxUnits(&self) -> bool;
    fn ElementTransformIncludingMotion(&self) -> AffineTransform;
    fn Children(&self) -> Box<dyn Iterator<Item = Box<dyn ClipPathContent<L> + '_>> + '_>;
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum ClipStrategy {
    None,
    Mask,
    Path,
}
fn modify_for_clip_path(
    style: &layoutng_style::style::computed_style::ComputedStyle,
    strategy: ClipStrategy,
) -> ClipStrategy {
    if strategy == ClipStrategy::Path && style.HasClipPath() {
        ClipStrategy::Mask
    } else {
        strategy
    }
}
// Source: layout_svg_resource_clipper.cc:46-104. A path clipped again is a
// mask strategy, but hit testing still enters its native NodeAtPoint path.
fn graphics_strategy<L: ClipPathLocation>(element: &dyn ClipPathContent<L>) -> ClipStrategy {
    let Some(style) = element.LayoutStyle() else {
        return ClipStrategy::None;
    };
    if element.LockedAncestorPreventingLayout()
        || style.Display() == foundation::EDisplay::kNone
        || style.Visibility() != foundation::EVisibility::kVisible
    {
        return ClipStrategy::None;
    }
    modify_for_clip_path(
        style,
        match element.LayoutKind() {
            ClipContentKind::Shape => ClipStrategy::Path,
            ClipContentKind::Text => ClipStrategy::Mask,
            ClipContentKind::Other => ClipStrategy::None,
        },
    )
}
fn ContributesToClip<L: ClipPathLocation>(element: &dyn ClipPathContent<L>) -> bool {
    let strategy = if element.IsUseElement() {
        let Some(style) = element.LayoutStyle() else {
            return false;
        };
        if element.LockedAncestorPreventingLayout()
            || style.Display() == foundation::EDisplay::kNone
        {
            return false;
        }
        let Some(target) = element.VisibleTargetGraphicsElementForClipping() else {
            return false;
        };
        modify_for_clip_path(style, graphics_strategy(target.as_ref()))
    } else if element.IsGraphicsElement() {
        graphics_strategy(element)
    } else {
        ClipStrategy::None
    };
    strategy != ClipStrategy::None
}
// Source: layout_svg_resource_clipper.cc:237-247. Preserve the element's
// compiled transform including its motion transform, then the units mapping.
fn CalculateClipTransform<L: ClipPathLocation>(
    resource: &dyn ClipPathResource<L>,
    reference: RectF,
) -> AffineTransform {
    let mut transform = resource.ElementTransformIncludingMotion();
    if resource.UsesObjectBoundingBoxUnits() {
        transform.Translate(reference.x() as f64, reference.y() as f64);
        transform.Scale(reference.width() as f64, reference.height() as f64);
    }
    transform
}
fn uses_zoomed<L: ClipPathLocation>(geometry: &dyn ClipPathGeometry<L>) -> bool {
    !geometry.IsSvgChild() || geometry.IsSvgForeignObject()
}
fn scaled(rect: RectF, scale: f32) -> RectF {
    RectF::new(
        foundation::gfx::PointF::new(rect.x() * scale, rect.y() * scale),
        foundation::gfx::SizeF::new(rect.width() * scale, rect.height() * scale),
    )
}
fn geometry_box(operation: &dyn ClipPathOperation) -> GeometryBox {
    match operation.GetType() {
        OperationType::kShape => unsafe {
            &*(operation as *const dyn ClipPathOperation as *const ShapeClipPathOperation)
        }
        .GetGeometryBox(),
        OperationType::kGeometryBox => unsafe {
            &*(operation as *const dyn ClipPathOperation as *const GeometryBoxClipPathOperation)
        }
        .GetGeometryBox(),
        OperationType::kReference => GeometryBox::kBorderBox,
    }
}
// Source: ClipPathClipper::CalcLocalReferenceBox/LocalReferenceBox,
// clip_path_clipper.cc:364-400.
fn LocalReferenceBox<L: ClipPathLocation>(geometry: &dyn ClipPathGeometry<L>) -> RectF {
    let operation = geometry.Operation().expect("clip-path required");
    let mut box_ = geometry_box(operation);
    if geometry.IsSvgChild() {
        if operation.GetType() == OperationType::kReference {
            box_ = GeometryBox::kFillBox;
        }
        let rect = geometry.SvgReferenceBoxForEffects(box_);
        if uses_zoomed(geometry) {
            scaled(rect, geometry.EffectiveZoom())
        } else {
            rect
        }
    } else {
        geometry.NonSvgReferenceBox(box_)
    }
}
fn RoundedReferenceBox<L: ClipPathLocation>(
    geometry: &dyn ClipPathGeometry<L>,
    box_: GeometryBox,
) -> paint_model::contoured_rect::ContouredRect {
    if geometry.IsSvgChild() {
        paint_model::contoured_rect::ContouredRect::new(
            paint_model::contoured_rect::FloatRoundedRect::new(
                LocalReferenceBox(geometry),
                Default::default(),
            ),
        )
    } else {
        geometry.ContouredBorderReferenceBox(box_)
    }
}
pub fn HitTest<L: ClipPathLocation>(
    geometry: &dyn ClipPathGeometry<L>,
    location: &L,
    factory: &dyn PaintPathHitTestFactory,
) -> Result<bool, InvalidBasicShape> {
    let reference = LocalReferenceBox(geometry);
    HitTestWithReferenceBox(geometry, reference, geometry, location, factory)
}
pub fn HitTestWithReferenceBox<L: ClipPathLocation>(
    owner: &dyn ClipPathGeometry<L>,
    reference: RectF,
    reference_owner: &dyn ClipPathGeometry<L>,
    location: &L,
    factory: &dyn PaintPathHitTestFactory,
) -> Result<bool, InvalidBasicShape> {
    let operation = owner
        .Operation()
        .expect("source ClipPathClipper requires clip-path");
    match operation.GetType() {
        OperationType::kShape => {
            let shape = unsafe {
                &*(operation as *const dyn ClipPathOperation as *const ShapeClipPathOperation)
            };
            let zoom = reference_owner.EffectiveZoom();
            let zoomed = uses_zoomed(reference_owner);
            let path = factory.create_basic_shape(
                shape.GetBasicShape(),
                if zoomed {
                    reference
                } else {
                    scaled(reference, zoom)
                },
                zoom,
                if zoomed { 1. } else { 1. / zoom },
            )?;
            Ok(location.IntersectsPath(path.as_ref()))
        }
        OperationType::kGeometryBox => {
            let operation = unsafe {
                &*(operation as *const dyn ClipPathOperation as *const GeometryBoxClipPathOperation)
            };
            Ok(location.IntersectsContouredRect(
                &RoundedReferenceBox(reference_owner, operation.GetGeometryBox()),
                factory,
            ))
        }
        OperationType::kReference => {
            let operation = unsafe {
                &*(operation as *const dyn ClipPathOperation as *const ReferenceClipPathOperation)
            };
            let resource = match owner.ResolveReference(operation) {
                ClipPathReference::Loading => return Ok(false),
                ClipPathReference::Missing => return Ok(true),
                ClipPathReference::Resource(resource) => resource,
            };
            let mut transform = AffineTransform::default();
            if uses_zoomed(reference_owner) {
                if !resource.UsesObjectBoundingBoxUnits() {
                    transform.Translate(reference.x() as f64, reference.y() as f64);
                }
                transform.Scale(
                    reference_owner.EffectiveZoom() as f64,
                    reference_owner.EffectiveZoom() as f64,
                );
            }
            let Some(location) = location.TransformedToLocal(&transform) else {
                return Ok(false);
            };
            let reference = if uses_zoomed(reference_owner) {
                scaled(reference, 1. / reference_owner.EffectiveZoom())
            } else {
                reference
            };
            HitTestClipContent(
                resource.as_ref(),
                reference,
                reference_owner,
                &location,
                factory,
            )
        }
    }
}
pub fn HitTestClipContent<L: ClipPathLocation>(
    resource: &dyn ClipPathResource<L>,
    reference: RectF,
    reference_owner: &dyn ClipPathGeometry<L>,
    location: &L,
    factory: &dyn PaintPathHitTestFactory,
) -> Result<bool, InvalidBasicShape> {
    // The resource's own clip-path is tested first, with the original
    // consumer reference box/zoom owner, before its clipPathUnits transform.
    if resource.Geometry().Operation().is_some()
        && !HitTestWithReferenceBox(
            resource.Geometry(),
            reference,
            reference_owner,
            location,
            factory,
        )?
    {
        return Ok(false);
    }
    let Some(local) = location.TransformedToLocal(&CalculateClipTransform(resource, reference))
    else {
        return Ok(false);
    };
    for child in resource.Children() {
        if ContributesToClip(child.as_ref()) && child.NodeAtPointForSvgClipContent(&local) {
            return Ok(true);
        }
    }
    Ok(false)
}
