//! Concrete external gfx value mapping. Foundation owns matrix arithmetic;
//! the injected Skia backend owns SkRRect operations. No trees are stored here.
use crate::adapter::{
    geometry::{Point3F, PointF, RRectF, Rect, RectF, Vector2dF},
    property_types::MaskFilterInfo,
};
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
    native: foundation::gfx::Transform,
    backend: PhantomData<fn() -> R>,
}
impl<R: RoundedRectTransformBackend> Clone for Transform<R> {
    fn clone(&self) -> Self {
        Self {
            native: self.native.clone(),
            backend: PhantomData,
        }
    }
}
impl<R: RoundedRectTransformBackend> Default for Transform<R> {
    fn default() -> Self {
        Self {
            native: Default::default(),
            backend: PhantomData,
        }
    }
}
impl<R: RoundedRectTransformBackend> PartialEq for Transform<R> {
    fn eq(&self, rhs: &Self) -> bool {
        self.native == rhs.native
    }
}
impl<R: RoundedRectTransformBackend> std::fmt::Debug for Transform<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.native.fmt(f)
    }
}
impl<R: RoundedRectTransformBackend> Transform<R> {
    pub fn FromColMajor(matrix: &[f64; 16]) -> Self {
        Self {
            native: foundation::gfx::Transform::ColMajor(matrix),
            backend: PhantomData,
        }
    }
    pub fn native(&self) -> &foundation::gfx::Transform {
        &self.native
    }
    pub fn native_mut(&mut self) -> &mut foundation::gfx::Transform {
        &mut self.native
    }
    fn GetInverseNative(&self, result: &mut Self) -> bool {
        self.native.GetInverse(&mut result.native)
    }
    fn MapPointNative(&self, point: PointF) -> PointF {
        let p = self
            .native
            .MapPoint(foundation::gfx::PointF::new(point.x(), point.y()));
        PointF::new(p.x(), p.y())
    }
    fn MapRectNative(&self, rect: RectF) -> RectF {
        let r = self.native.MapRect(foundation::gfx::RectF::new(
            foundation::gfx::PointF::new(rect.x(), rect.y()),
            foundation::gfx::SizeF::new(rect.width(), rect.height()),
        ));
        RectF::new(r.x(), r.y(), r.width(), r.height())
    }
    fn FlattenedSkMatrix(&self) -> [f32; 9] {
        // Source: ui/gfx/geometry/skia_conversions.cc:122-128
        let m = &self.native;
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
impl<R: RoundedRectTransformBackend> crate::adapter::geometry::Transform for Transform<R> {
    fn rc(&self, row: usize, column: usize) -> f64 {
        self.native.rc(row, column)
    }
    fn TransformVector4(&self, vector: &mut [f32; 4]) {
        self.native.TransformVector4(vector);
    }
    fn IsIdentityOrIntegerTranslation(&self) -> bool {
        self.native.IsIdentityOrIntegerTranslation()
    }
    fn IsIdentityOrTranslation(&self) -> bool {
        self.native.IsIdentityOrTranslation()
    }
    fn IsScaleOrTranslation(&self) -> bool {
        self.native.IsScaleOrTranslation()
    }
    fn IsScale2d(&self) -> bool {
        self.native.IsScale2d()
    }
    fn Preserves2dAxisAlignment(&self) -> bool {
        self.native.Preserves2dAxisAlignment()
    }
    fn IsFlat(&self) -> bool {
        self.native.IsFlat()
    }
    fn GetCheckedInverse(&self) -> Self {
        Self {
            native: self.native.GetCheckedInverse(),
            backend: PhantomData,
        }
    }
    fn To2dTranslation(&self) -> Vector2dF {
        let v = self.native.To2dTranslation();
        Vector2dF::new(v.x(), v.y())
    }
    fn To2dScale(&self) -> Vector2dF {
        let v = self.native.To2dScale();
        Vector2dF::new(v.x(), v.y())
    }
}
impl<R: RoundedRectTransformBackend> crate::adapter::property_types::PropertyTransform
    for Transform<R>
{
    fn IsInvertible(&self) -> bool {
        self.native.IsInvertible()
    }
}
impl<R: RoundedRectTransformBackend> crate::adapter::property_tree::PropertyTreeTransform
    for Transform<R>
{
    fn PostConcat(&mut self, other: &Self) {
        self.native.PostConcat(&other.native);
    }
    fn PreConcat(&mut self, other: &Self) {
        self.native.PreConcat(&other.native);
    }
    fn MultiplyAssign(&mut self, other: &Self) {
        self.native.PreConcat(&other.native);
    }
    fn Flatten(&mut self) {
        self.native.Flatten();
    }
    fn GetInverse(&self, result: &mut Self) -> bool {
        self.GetInverseNative(result)
    }
    fn Translate(&mut self, x: f32, y: f32) {
        self.native.Translate(x, y);
    }
    fn Translate3d(&mut self, x: f32, y: f32, z: f32) {
        self.native.Translate3d(x, y, z);
    }
    fn Scale(&mut self, x: f32, y: f32) {
        self.native.Scale(x, y);
    }
    fn PostTranslate(&mut self, offset: Vector2dF) {
        self.native.PostTranslate(offset.x(), offset.y());
    }
    fn MapPoint(&self, point: PointF) -> PointF {
        self.MapPointNative(point)
    }
    fn Round2dTranslationComponents(&mut self) {
        self.native.Round2dTranslationComponents();
    }
    fn IsApproximatelyIdentityOrTranslation(&self, tolerance: f32) -> bool {
        self.native
            .IsApproximatelyIdentityOrTranslation(tolerance as f64)
    }
    fn IsApproximatelyIdentityOrIntegerTranslation(&self, tolerance: f32) -> bool {
        self.native
            .IsApproximatelyIdentityOrIntegerTranslation(tolerance as f64)
    }
    fn RoundToIdentityOrIntegerTranslation(&mut self) {
        self.native.RoundToIdentityOrIntegerTranslation();
    }
    fn IsBackFaceVisible(&self) -> bool {
        self.native.IsBackFaceVisible()
    }
    fn ComputeTransform2dScaleComponents(&self, fallback: f32) -> Vector2dF {
        let v = self.native.ComputeTransform2dScaleComponents(fallback);
        Vector2dF::new(v.x(), v.y())
    }
    fn IsPositiveScaleOrTranslation(&self) -> bool {
        self.native.IsPositiveScaleOrTranslation()
    }
    fn MapRect(&self, rect: RectF) -> RectF {
        self.MapRectNative(rect)
    }
    fn HasPerspective(&self) -> bool {
        self.native.HasPerspective()
    }
    fn PostScale(&mut self, x: f32, y: f32) {
        self.native.PostScale(x, y);
    }
}
impl<R: RoundedRectTransformBackend> crate::adapter::occlusion::OcclusionTransform
    for Transform<R>
{
    fn NonDegeneratePreserves2dAxisAlignment(&self) -> bool {
        self.native.NonDegeneratePreserves2dAxisAlignment()
    }
    fn Multiply(&self, other: &Self) -> Self {
        let mut result = self.clone();
        result.native.PreConcat(&other.native);
        result
    }
    fn ToString(&self) -> String {
        format!("{:?}", self.native)
    }
}
impl<R: RoundedRectTransformBackend> crate::adapter::scrollbar::ScrollbarTransform
    for Transform<R>
{
    fn GetInverse(&self, result: &mut Self) -> bool {
        self.GetInverseNative(result)
    }
}
impl<R: RoundedRectTransformBackend> crate::adapter::damage_context::DamageTransform
    for Transform<R>
{
    fn ToFlattenedSkMatrix(&self) -> crate::adapter::occlusion::SkMatrix {
        crate::adapter::occlusion::SkMatrix {
            values: self.FlattenedSkMatrix(),
        }
    }
}
impl<R: RoundedRectTransformBackend> crate::adapter::draw_property::DrawTransform for Transform<R> {
    fn InverseOrIdentity(&self) -> Self {
        Self {
            native: self.native.InverseOrIdentity(),
            backend: PhantomData,
        }
    }
    fn MapRectInt(&self, rect: Rect) -> Rect {
        crate::adapter::geometry::ToEnclosingRect(self.MapRectNative(RectF::from(rect)))
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
            if self.native.IsIdentityOrTranslation() {
                return;
            }
            let radian = (gradient.angle as f32).to_radians();
            let y = -radian.sin();
            let x = radian.cos();
            let origin = self.MapPointNative(PointF::default());
            let end = self.MapPointNative(PointF::new(x, y));
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
        let p = self.native.MapPoint3d([point.x, point.y, point.z]);
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
        self.native.IsIdentity()
    }
}
impl<R: RoundedRectTransformBackend> crate::adapter::paint_chunks_to_cc_layer::ConversionTransform
    for Transform<R>
{
    fn IsIdentity(&self) -> bool {
        self.native.IsIdentity()
    }
    fn IsIdentityOr2dTranslation(&self) -> bool {
        self.native.IsIdentityOrTranslation() && self.native.rc(2, 3) == 0.
    }
    fn To2dTranslation(&self) -> Vector2dF {
        let p = self.native.To2dTranslation();
        Vector2dF::new(p.x(), p.y())
    }
    fn MapPoint(&self, point: PointF) -> PointF {
        self.MapPointNative(point)
    }
}
