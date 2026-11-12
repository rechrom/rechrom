//! The one backend-neutral gfx matrix value used by native property caches.
//! Math stays in Foundation. Engine-specific wrappers project only this value.
use super::traits::FloatClipRectTransform;
use crate::source_geometry::{PointF, RectF, Vector2dF};

pub trait GeometryMapperMatrix: crate::source_transform::PropertyTreeTransform {
    fn ToGeometryMapperMatrix(&self) -> SharedGeometryTransform;
    fn FromGeometryMapperMatrix(value: SharedGeometryTransform) -> Self;
}
pub struct SharedGeometryTransform {
    source: foundation::gfx::Transform,
}
impl Clone for SharedGeometryTransform {
    fn clone(&self) -> Self {
        Self {
            source: self.source.clone(),
        }
    }
}
impl Default for SharedGeometryTransform {
    fn default() -> Self {
        Self {
            source: Default::default(),
        }
    }
}
impl PartialEq for SharedGeometryTransform {
    fn eq(&self, rhs: &Self) -> bool {
        self.source == rhs.source
    }
}
impl std::fmt::Debug for SharedGeometryTransform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.source.fmt(f)
    }
}
impl SharedGeometryTransform {
    pub fn FromColMajor(matrix: &[f64; 16]) -> Self {
        Self {
            source: foundation::gfx::Transform::ColMajor(matrix),
        }
    }
    pub fn as_source(&self) -> &foundation::gfx::Transform {
        &self.source
    }
    pub fn transform_mut(&mut self) -> &mut foundation::gfx::Transform {
        &mut self.source
    }
    fn GetInverse(&self, result: &mut Self) -> bool {
        self.source.GetInverse(&mut result.source)
    }
    fn MapPoint(&self, point: PointF) -> PointF {
        let p = self
            .source
            .MapPoint(foundation::gfx::PointF::new(point.x(), point.y()));
        PointF::new(p.x(), p.y())
    }
    fn MapRect(&self, rect: RectF) -> RectF {
        let r = self.source.MapRect(foundation::gfx::RectF::new(
            foundation::gfx::PointF::new(rect.x(), rect.y()),
            foundation::gfx::SizeF::new(rect.width(), rect.height()),
        ));
        RectF::new(r.x(), r.y(), r.width(), r.height())
    }
    fn FlattenedSkMatrix(&self) -> [f32; 9] {
        // Source: ui/gfx/geometry/skia_conversions.cc:122-128
        let m = &self.source;
        [
            m.rc(0, 0) as f32,
            m.rc(0, 1) as f32,
            m.rc(0, 3) as f32,
            m.rc(1, 0) as f32,
            m.rc(1, 1) as f32,
            m.rc(1, 3) as f32,
            m.rc(3, 0) as f32,
            m.rc(3, 1) as f32,
            m.rc(3, 3) as f32,
        ]
    }
}
impl crate::source_geometry::Transform for SharedGeometryTransform {
    fn rc(&self, row: usize, column: usize) -> f64 {
        self.source.rc(row, column)
    }
    fn TransformVector4(&self, vector: &mut [f32; 4]) {
        self.source.TransformVector4(vector);
    }
    fn IsIdentityOrIntegerTranslation(&self) -> bool {
        self.source.IsIdentityOrIntegerTranslation()
    }
    fn IsIdentityOrTranslation(&self) -> bool {
        self.source.IsIdentityOrTranslation()
    }
    fn IsScaleOrTranslation(&self) -> bool {
        self.source.IsScaleOrTranslation()
    }
    fn IsScale2d(&self) -> bool {
        self.source.IsScale2d()
    }
    fn Preserves2dAxisAlignment(&self) -> bool {
        self.source.Preserves2dAxisAlignment()
    }
    fn IsFlat(&self) -> bool {
        self.source.IsFlat()
    }
    fn GetCheckedInverse(&self) -> Self {
        Self {
            source: self.source.GetCheckedInverse(),
        }
    }
    fn To2dTranslation(&self) -> Vector2dF {
        let v = self.source.To2dTranslation();
        Vector2dF::new(v.x(), v.y())
    }
    fn To2dScale(&self) -> Vector2dF {
        let v = self.source.To2dScale();
        Vector2dF::new(v.x(), v.y())
    }
}
impl crate::source_transform::PropertyTransform for SharedGeometryTransform {
    fn IsInvertible(&self) -> bool {
        self.source.IsInvertible()
    }
}
impl crate::source_transform::PropertyTreeTransform for SharedGeometryTransform {
    fn PostConcat(&mut self, other: &Self) {
        self.source.PostConcat(&other.source);
    }
    fn PreConcat(&mut self, other: &Self) {
        self.source.PreConcat(&other.source);
    }
    fn MultiplyAssign(&mut self, other: &Self) {
        self.source.PreConcat(&other.source);
    }
    fn Flatten(&mut self) {
        self.source.Flatten();
    }
    fn GetInverse(&self, result: &mut Self) -> bool {
        self.GetInverse(result)
    }
    fn Translate(&mut self, x: f32, y: f32) {
        self.source.Translate(x, y);
    }
    fn Translate3d(&mut self, x: f32, y: f32, z: f32) {
        self.source.Translate3d(x, y, z);
    }
    fn Scale(&mut self, x: f32, y: f32) {
        self.source.Scale(x, y);
    }
    fn PostTranslate(&mut self, offset: Vector2dF) {
        self.source.PostTranslate(offset.x(), offset.y());
    }
    fn MapPoint(&self, point: PointF) -> PointF {
        self.MapPoint(point)
    }
    fn Round2dTranslationComponents(&mut self) {
        self.source.Round2dTranslationComponents();
    }
    fn IsApproximatelyIdentityOrTranslation(&self, tolerance: f32) -> bool {
        self.source
            .IsApproximatelyIdentityOrTranslation(tolerance as f64)
    }
    fn IsApproximatelyIdentityOrIntegerTranslation(&self, tolerance: f32) -> bool {
        self.source
            .IsApproximatelyIdentityOrIntegerTranslation(tolerance as f64)
    }
    fn RoundToIdentityOrIntegerTranslation(&mut self) {
        self.source.RoundToIdentityOrIntegerTranslation();
    }
    fn IsBackFaceVisible(&self) -> bool {
        self.source.IsBackFaceVisible()
    }
    fn ComputeTransform2dScaleComponents(&self, fallback: f32) -> Vector2dF {
        let v = self.source.ComputeTransform2dScaleComponents(fallback);
        Vector2dF::new(v.x(), v.y())
    }
    fn IsPositiveScaleOrTranslation(&self) -> bool {
        self.source.IsPositiveScaleOrTranslation()
    }
    fn MapRect(&self, rect: RectF) -> RectF {
        self.MapRect(rect)
    }
    fn HasPerspective(&self) -> bool {
        self.source.HasPerspective()
    }
    fn PostScale(&mut self, x: f32, y: f32) {
        self.source.PostScale(x, y);
    }
}

impl SharedGeometryTransform {
    pub fn FromSource(source: foundation::gfx::Transform) -> Self {
        Self { source }
    }
    pub fn into_source(self) -> foundation::gfx::Transform {
        self.source
    }
}
impl GeometryMapperMatrix for SharedGeometryTransform {
    fn ToGeometryMapperMatrix(&self) -> Self {
        self.clone()
    }
    fn FromGeometryMapperMatrix(value: Self) -> Self {
        value
    }
}
impl FloatClipRectTransform for SharedGeometryTransform {
    fn IsIdentityOr2dTranslation(&self) -> bool {
        self.source.IsIdentityOrTranslation() && self.source.rc(2, 3) == 0.0
    }
    fn To2dTranslation(&self) -> Vector2dF {
        let v = self.source.To2dTranslation();
        Vector2dF::new(v.x(), v.y())
    }
    fn MapRect(&self, rect: RectF) -> RectF {
        self.MapRect(rect)
    }
}
