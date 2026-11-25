//! Concrete external gfx value mapping (ui/gfx/geometry/transform.h).
//! Foundation owns matrix arithmetic; the injected Skia backend owns SkRRect
//! operations. No trees are stored here. CommitTypes::Transform names this
//! type in both halves, so it lives in the shared root with the impls of the
//! shared gfx, property-tree and geometry-mapper facets. The impl packages
//! implement their own facets (occlusion, damage, scrollbar) for it;
//! `layerize::commit::transform` re-exports it.
use compositor_tree::geometry::{self, PointF, RRectF, RectF, Vector2dF};
use std::marker::PhantomData;
/// Native SkRRect value operations used by gfx::MaskFilterInfo. This capability
/// is separate from SkRegionBackend and has no fallback/approximation. A single
/// public Skia backend type can implement both external abilities.
pub trait RoundedRectTransformBackend {
    /// Exactly SkRRect(source).transform(SkMatrix::MakeAll(matrix), &output).
    fn TransformRoundedRect(matrix: [f32; 9], source: &RRectF, output: &mut RRectF) -> bool;
    /// Exactly SkRRect(value).isValid().
    fn IsRoundedRectValid(value: &RRectF) -> bool;
}
pub struct Transform<R: RoundedRectTransformBackend> {
    source: foundation::gfx::Transform,
    backend: PhantomData<fn() -> R>,
}
impl<R: RoundedRectTransformBackend> Clone for Transform<R> {
    fn clone(&self) -> Self {
        Self {
            source: self.source.clone(),
            backend: PhantomData,
        }
    }
}
impl<R: RoundedRectTransformBackend> Default for Transform<R> {
    fn default() -> Self {
        Self {
            source: Default::default(),
            backend: PhantomData,
        }
    }
}
impl<R: RoundedRectTransformBackend> PartialEq for Transform<R> {
    fn eq(&self, rhs: &Self) -> bool {
        self.source == rhs.source
    }
}
impl<R: RoundedRectTransformBackend> std::fmt::Debug for Transform<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.source.fmt(f)
    }
}
impl<R: RoundedRectTransformBackend> Transform<R> {
    pub fn FromColMajor(matrix: &[f64; 16]) -> Self {
        Self {
            source: foundation::gfx::Transform::ColMajor(matrix),
            backend: PhantomData,
        }
    }
    pub fn FromSource(source: foundation::gfx::Transform) -> Self {
        Self {
            source,
            backend: PhantomData,
        }
    }
    pub fn as_source(&self) -> &foundation::gfx::Transform {
        &self.source
    }
    pub fn transform_mut(&mut self) -> &mut foundation::gfx::Transform {
        &mut self.source
    }
    pub fn GetInverse(&self, result: &mut Self) -> bool {
        self.source.GetInverse(&mut result.source)
    }
    // Source: ui/gfx/geometry/transform.h NonDegeneratePreserves2dAxisAlignment
    // (read by cc occlusion and the viz OcclusionCuller).
    pub fn NonDegeneratePreserves2dAxisAlignment(&self) -> bool {
        self.source.NonDegeneratePreserves2dAxisAlignment()
    }
    pub fn MapPoint(&self, point: PointF) -> PointF {
        let p = self
            .source
            .MapPoint(foundation::gfx::PointF::new(point.x(), point.y()));
        PointF::new(p.x(), p.y())
    }
    pub fn MapRect(&self, rect: RectF) -> RectF {
        let r = self.source.MapRect(foundation::gfx::RectF::new(
            foundation::gfx::PointF::new(rect.x(), rect.y()),
            foundation::gfx::SizeF::new(rect.width(), rect.height()),
        ));
        RectF::new(r.x(), r.y(), r.width(), r.height())
    }
    pub fn FlattenedSkMatrix(&self) -> [f32; 9] {
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
impl<R: RoundedRectTransformBackend> geometry::Transform for Transform<R> {
    fn FromColMajor(matrix: &[f64; 16]) -> Self {
        Self::FromColMajor(matrix)
    }
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
            backend: PhantomData,
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
impl<R: RoundedRectTransformBackend> compositor_tree::property_types::PropertyTransform
    for Transform<R>
{
    fn IsInvertible(&self) -> bool {
        self.source.IsInvertible()
    }
}
impl<R: RoundedRectTransformBackend> compositor_tree::transform::PropertyTreeTransform
    for Transform<R>
{
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
impl<R: RoundedRectTransformBackend>
    paint_model::geometry_mapper_source::matrix::GeometryMapperMatrix for Transform<R>
{
    fn ToGeometryMapperMatrix(
        &self,
    ) -> paint_model::geometry_mapper_source::matrix::SharedGeometryTransform {
        paint_model::geometry_mapper_source::matrix::SharedGeometryTransform::FromSource(
            self.source.clone(),
        )
    }
    fn FromGeometryMapperMatrix(
        value: paint_model::geometry_mapper_source::matrix::SharedGeometryTransform,
    ) -> Self {
        Self {
            source: value.into_source(),
            backend: PhantomData,
        }
    }
}
