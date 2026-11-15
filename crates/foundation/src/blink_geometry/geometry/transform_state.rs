#![allow(non_snake_case, non_upper_case_globals)]

use crate::blink_geometry::transforms::affine_transform::AffineTransform;
use crate::{gfx, PhysicalOffset};

// cpp: core/layout/geometry/transform_state.h:48-53
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransformDirection {
    kApplyTransformDirection,
    kUnapplyInverseTransformDirection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransformAccumulation {
    kFlattenTransform,
    kAccumulateTransform,
}

pub enum TransformStateInput {
    Point(gfx::PointF),
    Quad(gfx::QuadF),
}

impl From<gfx::PointF> for TransformStateInput {
    fn from(point: gfx::PointF) -> Self {
        Self::Point(point)
    }
}
impl From<gfx::QuadF> for TransformStateInput {
    fn from(quad: gfx::QuadF) -> Self {
        Self::Quad(quad)
    }
}

// Source core/layout/geometry/transform_state.{h,cc}. One accumulation owner
// retains the existing affine fast path until a real 3D transform promotes it.
#[derive(Clone, Debug)]
enum AccumulatedTransform {
    Affine(AffineTransform),
    Full(Box<gfx::Transform>),
}
impl AccumulatedTransform {
    fn FromTransform(transform: &gfx::Transform) -> Self {
        if transform.Is2dTransform() {
            Self::Affine(AffineTransform::FromTransform(transform))
        } else {
            Self::Full(Box::new(transform.clone()))
        }
    }
    fn ToTransform(&self) -> gfx::Transform {
        match self {
            Self::Affine(t) => t.ToTransform(),
            Self::Full(t) => (**t).clone(),
        }
    }
    fn IsIdentity(&self) -> bool {
        match self {
            Self::Affine(t) => t.IsIdentity(),
            Self::Full(t) => t.IsIdentity(),
        }
    }
    fn MakeIdentity(&mut self) {
        match self {
            Self::Affine(t) => t.MakeIdentity(),
            Self::Full(t) => t.MakeIdentity(),
        }
    }
    fn Flatten(&mut self) {
        if let Self::Full(t) = self {
            t.Flatten();
        }
    }
    fn Concat(&mut self, incoming: &gfx::Transform, direction: TransformDirection) {
        if let Self::Affine(current) = self {
            if incoming.Is2dTransform() {
                let incoming = AffineTransform::FromTransform(incoming);
                if direction == TransformDirection::kApplyTransformDirection {
                    current.PostConcat(incoming);
                } else {
                    current.PreConcat(incoming);
                }
                return;
            }
        }
        if let Self::Full(current) = self {
            if direction == TransformDirection::kApplyTransformDirection {
                current.PostConcat(incoming);
            } else {
                current.PreConcat(incoming);
            }
            return;
        }
        // Promote this same accumulated owner once; never flatten a 3D chain
        // merely to fit the affine representation.
        let mut current = self.ToTransform();
        if direction == TransformDirection::kApplyTransformDirection {
            current.PostConcat(incoming);
        } else {
            current.PreConcat(incoming);
        }
        *self = Self::Full(Box::new(current));
    }
    fn Translate(&mut self, offset: PhysicalOffset, direction: TransformDirection) {
        match self {
            Self::Affine(current) => {
                let translation = AffineTransform::new(
                    1.0,
                    0.0,
                    0.0,
                    1.0,
                    offset.left.ToDouble(),
                    offset.top.ToDouble(),
                );
                if direction == TransformDirection::kApplyTransformDirection {
                    current.PostConcat(translation);
                } else {
                    current.PreConcat(translation);
                }
            }
            Self::Full(current) => {
                if direction == TransformDirection::kApplyTransformDirection {
                    current.PostTranslate(offset.left.ToFloat(), offset.top.ToFloat());
                } else {
                    current.Translate(offset.left.ToFloat(), offset.top.ToFloat());
                }
            }
        }
    }
    fn MapPoint(&self, point: gfx::PointF) -> gfx::PointF {
        match self {
            Self::Affine(t) => t.MapPoint(&point),
            Self::Full(t) => t.MapPoint(point),
        }
    }
    fn MapQuad(&self, quad: gfx::QuadF) -> gfx::QuadF {
        match self {
            Self::Affine(t) => t.MapQuad(quad),
            Self::Full(t) => t.MapQuad(quad),
        }
    }
    fn InverseProjectPoint(&self, point: gfx::PointF) -> gfx::PointF {
        match self {
            Self::Affine(t) => t.Inverse().MapPoint(&point),
            Self::Full(t) => t.InverseOrIdentity().ProjectPoint(point),
        }
    }
    fn InverseProjectQuad(&self, quad: gfx::QuadF) -> gfx::QuadF {
        match self {
            Self::Affine(t) => t.Inverse().MapQuad(quad),
            Self::Full(t) => t.InverseOrIdentity().ProjectQuad(quad),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TransformState {
    last_planar_point_: gfx::PointF,
    last_planar_quad_: gfx::QuadF,
    accumulated_transform_: Option<AccumulatedTransform>,
    accumulated_offset_: PhysicalOffset,
    force_accumulating_transform_: bool,
    map_point_: bool,
    map_quad_: bool,
    direction_: TransformDirection,
}

impl TransformState {
    pub const kApplyTransformDirection: TransformDirection =
        TransformDirection::kApplyTransformDirection;
    pub const kUnapplyInverseTransformDirection: TransformDirection =
        TransformDirection::kUnapplyInverseTransformDirection;
    pub const kFlattenTransform: TransformAccumulation = TransformAccumulation::kFlattenTransform;
    pub const kAccumulateTransform: TransformAccumulation =
        TransformAccumulation::kAccumulateTransform;

    // cpp: core/layout/geometry/transform_state.h:55-86
    pub fn new(direction: TransformDirection, input: impl Into<TransformStateInput>) -> Self {
        let (point, quad, map_point, map_quad) = match input.into() {
            TransformStateInput::Point(point) => (point, gfx::QuadF::default(), true, false),
            TransformStateInput::Quad(quad) => (gfx::PointF::default(), quad, false, true),
        };
        Self {
            last_planar_point_: point,
            last_planar_quad_: quad,
            accumulated_transform_: None,
            accumulated_offset_: PhysicalOffset::default(),
            force_accumulating_transform_: false,
            map_point_: map_point,
            map_quad_: map_quad,
            direction_: direction,
        }
    }

    pub fn accumulating(direction: TransformDirection) -> Self {
        let mut state = Self::new(direction, gfx::PointF::default());
        state.accumulated_transform_ =
            Some(AccumulatedTransform::Affine(AffineTransform::default()));
        state.force_accumulating_transform_ = true;
        state.map_point_ = false;
        state
    }

    // cpp: core/layout/geometry/transform_state.h:96-106
    pub fn SetQuad(&mut self, quad: gfx::QuadF) {
        debug_assert!(!self.map_point_);
        debug_assert!(self
            .accumulated_transform_
            .as_ref()
            .is_none_or(AccumulatedTransform::IsIdentity));
        self.accumulated_offset_ = PhysicalOffset::default();
        self.last_planar_quad_ = quad;
    }

    fn translate_mapped_coordinates(&mut self, offset: PhysicalOffset) {
        let sign = if self.direction_ == Self::kApplyTransformDirection {
            1.0
        } else {
            -1.0
        };
        let x = sign * offset.left.ToFloat();
        let y = sign * offset.top.ToFloat();
        let move_point = |p: &gfx::PointF| gfx::PointF::new(p.x() + x, p.y() + y);
        if self.map_point_ {
            self.last_planar_point_ = move_point(&self.last_planar_point_);
        }
        if self.map_quad_ {
            let quad = self.last_planar_quad_;
            self.last_planar_quad_ = gfx::QuadF::new(
                move_point(quad.p1()),
                move_point(quad.p2()),
                move_point(quad.p3()),
                move_point(quad.p4()),
            );
        }
    }

    // cpp: core/layout/geometry/transform_state.cc:51-59
    fn translate_transform(&mut self, offset: PhysicalOffset) {
        self.accumulated_transform_
            .as_mut()
            .expect("accumulated transform")
            .Translate(offset, self.direction_);
    }

    // cpp: core/layout/geometry/transform_state.cc:70-88
    pub fn Move(&mut self, offset: PhysicalOffset, mut accumulation: TransformAccumulation) {
        if self.force_accumulating_transform_ {
            accumulation = Self::kAccumulateTransform;
        }
        if accumulation == Self::kFlattenTransform || self.accumulated_transform_.is_none() {
            self.accumulated_offset_ += offset;
        } else {
            self.apply_accumulated_offset();
            if self.accumulated_transform_.is_some() {
                self.translate_transform(offset);
            } else {
                self.translate_mapped_coordinates(offset);
            }
        }
    }

    // cpp: core/layout/geometry/transform_state.cc:90-101
    fn apply_accumulated_offset(&mut self) {
        let offset = std::mem::take(&mut self.accumulated_offset_);
        if offset.IsZero() {
            return;
        }
        if self.accumulated_transform_.is_some() {
            self.translate_transform(offset);
            self.Flatten();
        } else {
            self.translate_mapped_coordinates(offset);
        }
    }

    // cpp: core/layout/geometry/transform_state.cc:105-147
    pub fn ApplyTransform(
        &mut self,
        transform: &gfx::Transform,
        accumulation: TransformAccumulation,
    ) {
        if transform.Is2dTransform() && transform.IsIdentityOrIntegerTranslation() {
            let translation = transform.To2dTranslation();
            self.Move(
                PhysicalOffset::new(
                    crate::LayoutUnit::FromFloatRound(translation.x()),
                    crate::LayoutUnit::FromFloatRound(translation.y()),
                ),
                accumulation,
            );
            return;
        }
        self.apply_accumulated_offset();
        if let Some(current) = self.accumulated_transform_.as_mut() {
            current.Concat(transform, self.direction_);
        } else if accumulation == Self::kAccumulateTransform {
            self.accumulated_transform_ = Some(AccumulatedTransform::FromTransform(transform));
        }
        if accumulation == Self::kFlattenTransform {
            if self.force_accumulating_transform_ {
                self.accumulated_transform_
                    .as_mut()
                    .expect("forced accumulated owner")
                    .Flatten();
            } else {
                if self.accumulated_transform_.is_some() {
                    self.flatten_accumulated_transform();
                } else if transform.Is2dTransform() {
                    self.flatten_with_transform(&AccumulatedTransform::Affine(
                        AffineTransform::FromTransform(transform),
                    ));
                } else {
                    self.flatten_with_gfx_transform(transform);
                }
            }
        }
    }

    // cpp: core/layout/geometry/transform_state.cc:149-159
    pub fn Flatten(&mut self) {
        assert!(!self.force_accumulating_transform_);
        self.apply_accumulated_offset();
        self.flatten_accumulated_transform();
    }

    // cpp: core/layout/geometry/transform_state.cc:194-215
    // Move the unique owner temporarily while mapping planar fields. This
    // avoids allocating/copying a 4x4 owner just to satisfy a mutable borrow.
    fn flatten_accumulated_transform(&mut self) {
        if let Some(mut transform) = self.accumulated_transform_.take() {
            self.flatten_with_transform(&transform);
            transform.MakeIdentity();
            self.accumulated_transform_ = Some(transform);
        }
    }
    fn flatten_with_gfx_transform(&mut self, transform: &gfx::Transform) {
        if self.direction_ == Self::kApplyTransformDirection {
            if self.map_point_ {
                self.last_planar_point_ = transform.MapPoint(self.last_planar_point_);
            }
            if self.map_quad_ {
                self.last_planar_quad_ = transform.MapQuad(self.last_planar_quad_);
            }
        } else {
            let mut inverse = gfx::Transform::default();
            if transform.GetInverse(&mut inverse) {
                if self.map_point_ {
                    self.last_planar_point_ = inverse.ProjectPoint(self.last_planar_point_);
                }
                if self.map_quad_ {
                    self.last_planar_quad_ = inverse.ProjectQuad(self.last_planar_quad_);
                }
            }
        }
    }
    fn flatten_with_transform(&mut self, transform: &AccumulatedTransform) {
        // Source FlattenWithTransform keeps planar geometry unchanged when
        // inverse mapping is singular; MappedPoint/Quad use identity fallback.
        if self.direction_ == Self::kApplyTransformDirection {
            if self.map_point_ {
                self.last_planar_point_ = transform.MapPoint(self.last_planar_point_);
            }
            if self.map_quad_ {
                self.last_planar_quad_ = transform.MapQuad(self.last_planar_quad_);
            }
        } else {
            match transform {
                AccumulatedTransform::Affine(t) if t.IsInvertible() => {
                    let inverse = t.Inverse();
                    if self.map_point_ {
                        self.last_planar_point_ = inverse.MapPoint(&self.last_planar_point_);
                    }
                    if self.map_quad_ {
                        self.last_planar_quad_ = inverse.MapQuad(self.last_planar_quad_);
                    }
                }
                AccumulatedTransform::Full(t) => {
                    self.flatten_with_gfx_transform(t);
                }

                _ => {}
            }
        }
        if let Some(current) = self.accumulated_transform_.as_mut() {
            current.MakeIdentity();
        }
    }

    // cpp: core/layout/geometry/transform_state.h:115-125
    pub fn LastPlanarPoint(&self) -> gfx::PointF {
        self.last_planar_point_
    }
    pub fn LastPlanarQuad(&self) -> gfx::QuadF {
        self.last_planar_quad_
    }

    // cpp: core/layout/geometry/transform_state.cc:161-192
    pub fn MappedPoint(&self) -> PhysicalOffset {
        let mut point = self.last_planar_point_;
        let sign = if self.direction_ == Self::kApplyTransformDirection {
            1.0
        } else {
            -1.0
        };
        point = gfx::PointF::new(
            point.x() + sign * self.accumulated_offset_.left.ToFloat(),
            point.y() + sign * self.accumulated_offset_.top.ToFloat(),
        );
        if let Some(transform) = &self.accumulated_transform_ {
            point = if self.direction_ == Self::kApplyTransformDirection {
                transform.MapPoint(point)
            } else {
                transform.InverseProjectPoint(point)
            };
        }
        PhysicalOffset::FromPointFRound(&point)
    }

    pub fn MappedQuad(&self) -> gfx::QuadF {
        let quad = self.last_planar_quad_;
        let sign = if self.direction_ == Self::kApplyTransformDirection {
            1.0
        } else {
            -1.0
        };
        let x = sign * self.accumulated_offset_.left.ToFloat();
        let y = sign * self.accumulated_offset_.top.ToFloat();
        let moved = |p: &gfx::PointF| gfx::PointF::new(p.x() + x, p.y() + y);
        let translated = gfx::QuadF::new(
            moved(quad.p1()),
            moved(quad.p2()),
            moved(quad.p3()),
            moved(quad.p4()),
        );
        match &self.accumulated_transform_ {
            Some(transform) if self.direction_ == Self::kApplyTransformDirection => {
                transform.MapQuad(translated)
            }
            Some(transform) => transform.InverseProjectQuad(translated),
            None => translated,
        }
    }

    pub fn Direction(&self) -> TransformDirection {
        self.direction_
    }
    pub fn AccumulatedTransform(&self) -> gfx::Transform {
        assert!(self.force_accumulating_transform_);
        self.accumulated_transform_
            .as_ref()
            .expect("accumulating transform")
            .ToTransform()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserve_3d_accumulates_and_inverse_projects_to_the_plane() {
        let mut rotation = gfx::Transform::default();
        rotation.RotateAboutYAxis(60.0);
        let mut undo = gfx::Transform::default();
        undo.RotateAboutYAxis(-60.0);
        let original = gfx::PointF::new(2.0, 3.0);
        let mut apply = TransformState::new(TransformState::kApplyTransformDirection, original);
        apply.ApplyTransform(&rotation, TransformState::kAccumulateTransform);
        apply.ApplyTransform(&undo, TransformState::kAccumulateTransform);
        apply.Flatten();
        assert!((apply.LastPlanarPoint().x() - 2.0).abs() < 0.00001);
        assert!((apply.LastPlanarPoint().y() - 3.0).abs() < 0.00001);
        // Inverse MapPoint would return x=0.5. Source ProjectPoint intersects
        // the transformed plane and correctly recovers the original x=2.
        let mut inverse = TransformState::new(
            TransformState::kUnapplyInverseTransformDirection,
            gfx::PointF::new(1.0, 3.0),
        );
        inverse.ApplyTransform(&rotation, TransformState::kFlattenTransform);
        assert!((inverse.LastPlanarPoint().x() - 2.0).abs() < 0.00001);
        assert!((inverse.LastPlanarPoint().y() - 3.0).abs() < 0.00001);
    }

    #[test]
    fn move_and_flatten_map_a_block_rect() {
        let quad = gfx::QuadF::from(gfx::RectF::new(
            gfx::PointF::default(),
            gfx::SizeF::new(20.0, 10.0),
        ));
        let mut state = TransformState::new(TransformState::kApplyTransformDirection, quad);
        state.Move(
            PhysicalOffset::new(
                crate::LayoutUnit::from_signed(5),
                crate::LayoutUnit::from_signed(7),
            ),
            TransformState::kFlattenTransform,
        );
        assert_eq!(
            state.MappedQuad().BoundingBox().origin(),
            gfx::PointF::new(5.0, 7.0)
        );
        state.Flatten();
        assert_eq!(
            state.LastPlanarQuad().BoundingBox().origin(),
            gfx::PointF::new(5.0, 7.0)
        );
    }

    #[test]
    fn accumulated_affine_transform_maps_and_unmaps() {
        let scale = AffineTransform::new(2.0, 0.0, 0.0, 3.0, 0.0, 0.0).ToTransform();
        let mut apply = TransformState::new(
            TransformState::kApplyTransformDirection,
            gfx::PointF::new(4.0, 5.0),
        );
        apply.ApplyTransform(&scale, TransformState::kAccumulateTransform);
        assert_eq!(
            apply.MappedPoint(),
            PhysicalOffset::new(
                crate::LayoutUnit::from_signed(8),
                crate::LayoutUnit::from_signed(15)
            )
        );
        let mut unapply = TransformState::new(
            TransformState::kUnapplyInverseTransformDirection,
            gfx::PointF::new(8.0, 15.0),
        );
        unapply.ApplyTransform(&scale, TransformState::kAccumulateTransform);
        assert_eq!(
            unapply.MappedPoint(),
            PhysicalOffset::new(
                crate::LayoutUnit::from_signed(4),
                crate::LayoutUnit::from_signed(5)
            )
        );
    }
}
