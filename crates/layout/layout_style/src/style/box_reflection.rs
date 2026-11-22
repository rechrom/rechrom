#![allow(non_snake_case)]

use foundation::{gfx, RectF};

// cpp: platform/graphics/box_reflection.h:28-34
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReflectionDirection {
    kVerticalReflection,
    kHorizontalReflection,
}

// PaintRecord is owned by Paint rather than Style in Rechrom. The immutable
// style-side value retains the one bit its source equality contract observes:
// whether the record is empty.
// cpp: platform/graphics/box_reflection.h:36-63
#[derive(Clone, Debug)]
pub struct BoxReflection {
    direction_: ReflectionDirection,
    offset_: f32,
    mask_is_empty_: bool,
    mask_bounds_: RectF,
}

impl BoxReflection {
    // cpp: platform/graphics/box_reflection.cc:16-17
    pub fn new(direction: ReflectionDirection, offset: f32) -> Self {
        Self::new_with_mask_state(direction, offset, true, RectF::default())
    }

    // cpp: platform/graphics/box_reflection.cc:19-30
    pub fn new_with_mask_state(
        direction: ReflectionDirection,
        offset: f32,
        mask_is_empty: bool,
        mask_bounds: RectF,
    ) -> Self {
        Self {
            direction_: direction,
            offset_: offset,
            mask_is_empty_: mask_is_empty,
            mask_bounds_: mask_bounds,
        }
    }

    pub fn Direction(&self) -> ReflectionDirection {
        self.direction_
    }

    pub fn Offset(&self) -> f32 {
        self.offset_
    }

    pub fn MaskIsEmpty(&self) -> bool {
        self.mask_is_empty_
    }

    pub fn MaskBounds(&self) -> RectF {
        self.mask_bounds_
    }

    // cpp: platform/graphics/box_reflection.cc:32-53
    pub fn MapRect(&self, rect: &RectF) -> RectF {
        let reflected = match self.direction_ {
            ReflectionDirection::kVerticalReflection => RectF::new(
                gfx::PointF::new(rect.x(), self.offset_ - rect.bottom()),
                rect.size(),
            ),
            ReflectionDirection::kHorizontalReflection => RectF::new(
                gfx::PointF::new(self.offset_ - rect.right(), rect.y()),
                rect.size(),
            ),
        };
        let mut result = *rect;
        result.Union(reflected);
        result
    }
}

// cpp: platform/graphics/box_reflection.h:66-71. False negatives are
// intentional: two nonempty PaintRecords are never considered equal.
impl PartialEq for BoxReflection {
    fn eq(&self, other: &Self) -> bool {
        self.direction_ == other.direction_
            && self.offset_ == other.offset_
            && self.mask_is_empty_
            && other.mask_is_empty_
            && self.mask_bounds_ == other.mask_bounds_
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equality_and_mapping_follow_box_reflection_source_contract() {
        let bounds = RectF::new(gfx::PointF::new(1.0, 2.0), gfx::SizeF::new(3.0, 4.0));
        let empty = BoxReflection::new_with_mask_state(
            ReflectionDirection::kVerticalReflection,
            100.0,
            true,
            bounds,
        );
        assert_eq!(empty, empty.clone());

        let nonempty = BoxReflection::new_with_mask_state(
            ReflectionDirection::kVerticalReflection,
            100.0,
            false,
            bounds,
        );
        assert_ne!(nonempty, nonempty.clone());

        let rect = RectF::new(gfx::PointF::new(10.0, 20.0), gfx::SizeF::new(30.0, 40.0));
        assert_eq!(
            empty.MapRect(&rect),
            RectF::new(gfx::PointF::new(10.0, 20.0), gfx::SizeF::new(30.0, 60.0))
        );
    }
}
