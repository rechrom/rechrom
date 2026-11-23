//! Compositor-local facets of the external gfx transform value. The value
//! type and its shared facets live in layerize::commit::transform (moved
//! there because CommitTypes::Transform names it in both halves).
use crate::adapter::{
    geometry::{Point3F, PointF, RRectF, Rect, RectF, Vector2dF},
    property_types::MaskFilterInfo,
};
pub use layerize::commit::transform::{RoundedRectTransformBackend, Transform};

impl<R: RoundedRectTransformBackend> crate::adapter::draw_property::DrawTransform for Transform<R> {
    fn InverseOrIdentity(&self) -> Self {
        Self::FromSource(self.as_source().InverseOrIdentity())
    }
    fn MapRectInt(&self, rect: Rect) -> Rect {
        crate::adapter::geometry::ToEnclosingRect(Transform::MapRect(self, RectF::from(rect)))
    }
    fn ApplyMaskFilterInfo(&self, info: &mut MaskFilterInfo) {
        // Source: ui/gfx/geometry/mask_filter_info.cc:15-47
        if info.rounded_corner_bounds.rect.IsEmpty() {
            return;
        }
        let mut rounded = self.FlattenedSkMatrix();
        for index in [0, 1, 3, 4] {
            if rounded[index].abs() < f32::EPSILON {
                rounded[index] = 0.;
            }
        }
        let mut new_rect = RRectF::default();
        if !R::TransformRoundedRect(rounded, &info.rounded_corner_bounds, &mut new_rect)
            || !R::IsRoundedRectValid(&new_rect)
        {
            info.rounded_corner_bounds = RRectF::default();
            return;
        }
        info.rounded_corner_bounds = new_rect;
        if let Some(gradient) = info.gradient_mask.as_mut().filter(|g| !g.steps.is_empty()) {
            // Source: ui/gfx/geometry/linear_gradient.cc:52-64
            if self.as_source().IsIdentityOrTranslation() {
                return;
            }
            let radian = (gradient.angle as f32).to_radians();
            let y = -radian.sin();
            let x = radian.cos();
            let origin = self.MapPoint(PointF::default());
            let end = self.MapPoint(PointF::new(x, y));
            let new_angle = (end.y() - origin.y())
                .atan2(end.x() - origin.x())
                .to_degrees();
            gradient.angle = -(new_angle.round() as i16);
        }
    }
}
impl<R: RoundedRectTransformBackend> crate::adapter::layer_tree_impl::LayerTreeTransform
    for Transform<R>
{
    fn MapPoint3D(&self, point: Point3F) -> Point3F {
        let p = self.as_source().MapPoint3d([point.x, point.y, point.z]);
        Point3F {
            x: p[0],
            y: p[1],
            z: p[2],
        }
    }
}
impl<R: RoundedRectTransformBackend> crate::adapter::layer_impl::LayerImplTransform
    for Transform<R>
{
    fn IsIdentity(&self) -> bool {
        self.as_source().IsIdentity()
    }
}
impl<R: RoundedRectTransformBackend> crate::adapter::paint_chunks_to_cc_layer::ConversionTransform
    for Transform<R>
{
    fn IsIdentity(&self) -> bool {
        self.as_source().IsIdentity()
    }
    fn IsIdentityOr2dTranslation(&self) -> bool {
        self.as_source().IsIdentityOrTranslation() && self.as_source().rc(2, 3) == 0.
    }
    fn To2dTranslation(&self) -> Vector2dF {
        let p = self.as_source().To2dTranslation();
        Vector2dF::new(p.x(), p.y())
    }
    fn MapPoint(&self, point: PointF) -> PointF {
        self.MapPoint(point)
    }
}
