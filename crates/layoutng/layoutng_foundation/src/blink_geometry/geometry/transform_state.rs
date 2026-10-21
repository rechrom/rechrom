#![allow(non_snake_case, non_upper_case_globals)]

use crate::blink_geometry::transforms::affine_transform::AffineTransform;
use crate::{gfx, PhysicalOffset};

// cpp: foundation/blink_geometry/geometry/transform_state.h:48-53
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

// This is the source's offset and 2D-affine mapping path. A non-2D gfx
// transform is rejected explicitly until the 4x4 projection path is mapped.
// cpp: foundation/blink_geometry/geometry/transform_state.h:46-140
#[derive(Clone, Debug)]
pub struct TransformState {
    last_planar_point_: gfx::PointF,
    last_planar_quad_: gfx::QuadF,
    accumulated_transform_: Option<AffineTransform>,
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

    // cpp: foundation/blink_geometry/geometry/transform_state.h:55-86
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
        state.accumulated_transform_ = Some(AffineTransform::default());
        state.force_accumulating_transform_ = true;
        state.map_point_ = false;
        state
    }

    // cpp: foundation/blink_geometry/geometry/transform_state.h:96-106
    pub fn SetQuad(&mut self, quad: gfx::QuadF) {
        debug_assert!(!self.map_point_);
        debug_assert!(self
            .accumulated_transform_
            .as_ref()
            .is_none_or(AffineTransform::IsIdentity));
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

    // cpp: foundation/blink_geometry/geometry/transform_state.cc:51-59
    fn translate_transform(&mut self, offset: PhysicalOffset) {
        let translation = AffineTransform::new(
            1.0,
            0.0,
            0.0,
            1.0,
            offset.left.ToDouble(),
            offset.top.ToDouble(),
        );
        let transform = self
            .accumulated_transform_
            .as_mut()
            .expect("accumulated transform");
        if self.direction_ == Self::kApplyTransformDirection {
            transform.PostConcat(translation);
        } else {
            transform.PreConcat(translation);
        }
    }

    // cpp: foundation/blink_geometry/geometry/transform_state.cc:70-88
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

    // cpp: foundation/blink_geometry/geometry/transform_state.cc:90-101
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

    // cpp: foundation/blink_geometry/geometry/transform_state.cc:105-147
    pub fn ApplyTransform(
        &mut self,
        transform: &gfx::Transform,
        accumulation: TransformAccumulation,
    ) {
        let matrix = transform.GetColMajor();
        assert!(
            matrix[2] == 0.0
                && matrix[3] == 0.0
                && matrix[6] == 0.0
                && matrix[7] == 0.0
                && matrix[8] == 0.0
                && matrix[9] == 0.0
                && matrix[10] == 1.0
                && matrix[11] == 0.0
                && matrix[14] == 0.0
                && matrix[15] == 1.0,
            "3D TransformState mapping is not implemented"
        );
        let incoming = AffineTransform::FromTransform(transform);
        if incoming.A() == 1.0
            && incoming.B() == 0.0
            && incoming.C() == 0.0
            && incoming.D() == 1.0
            && incoming.E().fract() == 0.0
            && incoming.F().fract() == 0.0
        {
            self.Move(
                PhysicalOffset::new(
                    crate::LayoutUnit::FromFloatRound(incoming.E() as f32),
                    crate::LayoutUnit::FromFloatRound(incoming.F() as f32),
                ),
                accumulation,
            );
            return;
        }
        self.apply_accumulated_offset();
        if let Some(current) = self.accumulated_transform_.as_mut() {
            if self.direction_ == Self::kApplyTransformDirection {
                current.PostConcat(incoming);
            } else {
                current.PreConcat(incoming);
            }
        } else if accumulation == Self::kAccumulateTransform {
            self.accumulated_transform_ = Some(incoming);
        }
        if accumulation == Self::kFlattenTransform && !self.force_accumulating_transform_ {
            self.flatten_with_transform(self.accumulated_transform_.unwrap_or(incoming));
        }
    }

    // cpp: foundation/blink_geometry/geometry/transform_state.cc:149-159
    pub fn Flatten(&mut self) {
        assert!(!self.force_accumulating_transform_);
        self.apply_accumulated_offset();
        if let Some(transform) = self.accumulated_transform_ {
            self.flatten_with_transform(transform);
        }
    }

    // cpp: foundation/blink_geometry/geometry/transform_state.cc:194-215
    fn flatten_with_transform(&mut self, transform: AffineTransform) {
        let mapping = if self.direction_ == Self::kApplyTransformDirection {
            transform
        } else {
            transform.Inverse()
        };
        if self.map_point_ {
            self.last_planar_point_ = mapping.MapPoint(&self.last_planar_point_);
        }
        if self.map_quad_ {
            self.last_planar_quad_ = mapping.MapQuad(self.last_planar_quad_);
        }
        if self.accumulated_transform_.is_some() {
            self.accumulated_transform_ = Some(AffineTransform::default());
        }
    }

    // cpp: foundation/blink_geometry/geometry/transform_state.h:115-125
    pub fn LastPlanarPoint(&self) -> gfx::PointF {
        self.last_planar_point_
    }
    pub fn LastPlanarQuad(&self) -> gfx::QuadF {
        self.last_planar_quad_
    }

    // cpp: foundation/blink_geometry/geometry/transform_state.cc:161-192
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
        if let Some(transform) = self.accumulated_transform_ {
            point = if self.direction_ == Self::kApplyTransformDirection {
                transform.MapPoint(&point)
            } else {
                transform.Inverse().MapPoint(&point)
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
        match self.accumulated_transform_ {
            Some(transform) if self.direction_ == Self::kApplyTransformDirection => {
                transform.MapQuad(translated)
            }
            Some(transform) => transform.Inverse().MapQuad(translated),
            None => translated,
        }
    }

    pub fn Direction(&self) -> TransformDirection {
        self.direction_
    }
    pub fn AccumulatedTransform(&self) -> gfx::Transform {
        assert!(self.force_accumulating_transform_);
        self.accumulated_transform_
            .expect("accumulating transform")
            .ToTransform()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
