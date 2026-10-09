// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! core/style/style_ray.h:25-72 and style_ray.cc:17-157.
#![allow(non_snake_case)]
use super::basic_shapes::{BasicShape, ShapeType};
use foundation::{gfx, LengthPoint, Path, PointAndTangent, Traceable, Visitor};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RaySize {
    kClosestSide,
    kClosestCorner,
    kFarthestSide,
    kFarthestCorner,
    kSides,
}
pub struct StyleRay {
    angle_: f32,
    size_: RaySize,
    contain_: bool,
    center_: LengthPoint,
    has_explicit_center_: bool,
}
impl StyleRay {
    pub fn new(
        angle: f32,
        size: RaySize,
        contain: bool,
        center: &LengthPoint,
        has_explicit_center: bool,
    ) -> Self {
        Self {
            angle_: angle,
            size_: size,
            contain_: contain,
            center_: center.clone(),
            has_explicit_center_: has_explicit_center,
        }
    }
    pub fn Angle(&self) -> f32 {
        self.angle_
    }
    pub fn Size(&self) -> RaySize {
        self.size_
    }
    pub fn Contain(&self) -> bool {
        self.contain_
    }
    pub fn Center(&self) -> &LengthPoint {
        &self.center_
    }
    pub fn HasExplicitCenter(&self) -> bool {
        self.has_explicit_center_
    }
    // style_ray.cc:49-143. Source geometry is independent of the Path owner.
    pub fn CalculateRayPathLength(&self, p: &gfx::PointF, b: &gfx::SizeF) -> f32 {
        let closest = matches!(self.Size(), RaySize::kClosestSide | RaySize::kClosestCorner);
        let values = match self.Size() {
            RaySize::kClosestSide | RaySize::kFarthestSide => [
                p.x().abs(),
                (p.x() - b.width()).abs(),
                p.y().abs(),
                (p.y() - b.height()).abs(),
            ],
            RaySize::kClosestCorner | RaySize::kFarthestCorner => [
                p.x().hypot(p.y()),
                (p.x() - b.width()).hypot(p.y()),
                (p.x() - b.width()).hypot(p.y() - b.height()),
                p.x().hypot(p.y() - b.height()),
            ],
            RaySize::kSides => {
                if p.x() < 0.0 || p.x() > b.width() || p.y() < 0.0 || p.y() > b.height() {
                    return 0.0;
                }
                let theta = self.Angle().to_radians();
                let mut cos = theta.cos();
                let mut sin = theta.sin();
                let vertical = if cos >= 0.0 {
                    p.y()
                } else {
                    b.height() - p.y()
                };
                let horizontal = if sin >= 0.0 { b.width() - p.x() } else { p.x() };
                cos = cos.abs();
                sin = sin.abs();
                return if vertical * sin > horizontal * cos {
                    horizontal / sin
                } else {
                    vertical / cos
                };
            }
        };
        values
            .into_iter()
            .reduce(if closest { f32::min } else { f32::max })
            .unwrap()
    }
    // style_ray.cc:145-154.
    pub fn PointAndNormalAtLength(&self, p: &gfx::PointF, length: f32) -> PointAndTangent {
        let angle = self.Angle() - 90.0;
        let rad = angle.to_radians();
        PointAndTangent {
            point: gfx::PointF::new(p.x() + length * rad.cos(), p.y() + length * rad.sin()),
            tangent_in_degrees: angle,
        }
    }
}
impl BasicShape for StyleRay {
    fn GetPath(&self, _: &gfx::RectF, _: f32, _: f32) -> Path {
        unreachable!("Chromium StyleRay::GetPath: rays use independent motion geometry")
    }
    fn GetType(&self) -> ShapeType {
        ShapeType::kStyleRayType
    }
    fn IsEqualAssumingSameType(&self, other: &dyn BasicShape) -> bool {
        debug_assert!(self.IsSameType(other));
        let other = unsafe { &*(other as *const dyn BasicShape as *const Self) };
        self.angle_ == other.angle_
            && self.size_ == other.size_
            && self.contain_ == other.contain_
            && self.center_ == other.center_
            && self.has_explicit_center_ == other.has_explicit_center_
    }
}
impl Traceable for StyleRay {
    fn Trace(&self, _: &mut Visitor<'_>) {}
}
