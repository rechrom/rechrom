//! Transform operations shared by property-tree consumers.
//!
//! The matrix representation stays in the embedding compositor. Keeping this
//! contract in the tree package lets draw, input, and animation algorithms
//! depend on tree semantics without depending on the compositor facade.

use crate::geometry::{PointF, RectF, Transform, Vector2dF};

pub trait PropertyTransform: Transform + Clone + Default + PartialEq {
    fn IsInvertible(&self) -> bool;
}

pub trait PropertyTreeTransform: PropertyTransform {
    fn PostConcat(&mut self, transform: &Self);
    fn PreConcat(&mut self, transform: &Self);
    fn MultiplyAssign(&mut self, transform: &Self);
    fn Flatten(&mut self);
    fn GetInverse(&self, inverse: &mut Self) -> bool;
    fn Translate(&mut self, x: f32, y: f32);
    fn Translate3d(&mut self, x: f32, y: f32, z: f32);
    fn Scale(&mut self, x: f32, y: f32);
    fn PostTranslate(&mut self, offset: Vector2dF);
    fn MapPoint(&self, point: PointF) -> PointF;
    fn Round2dTranslationComponents(&mut self);
    fn IsApproximatelyIdentityOrTranslation(&self, tolerance: f32) -> bool;
    fn IsApproximatelyIdentityOrIntegerTranslation(&self, tolerance: f32) -> bool;
    fn RoundToIdentityOrIntegerTranslation(&mut self);
    fn IsBackFaceVisible(&self) -> bool;
    fn ComputeTransform2dScaleComponents(&self, fallback: f32) -> Vector2dF;
    fn IsPositiveScaleOrTranslation(&self) -> bool;
    fn MapRect(&self, rect: RectF) -> RectF;
    fn HasPerspective(&self) -> bool;
    fn PostScale(&mut self, x: f32, y: f32);
}
