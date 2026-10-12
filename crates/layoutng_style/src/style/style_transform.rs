use foundation::{
    gfx, ETransformBox, EVectorEffect, FloatValueForLength, Length, PhysicalRect, PointAndTangent,
    TransformOperationType,
};

use super::basic_shapes::BasicShape;

use super::computed_style::ComputedStyle;
use super::computed_style_constants::CoordBox;

/// Geometry which motion-path needs from the owning layout box.
///
/// Chromium's ComputedStyle algorithm receives a LayoutBox directly.  Keep
/// layoutng_style independent of LayoutNG by carrying only the three immutable
/// geometry queries used by that algorithm across the package boundary.
pub trait MotionPathBox {
    fn MotionPathReferenceBox(&self, _coord_box: CoordBox) -> gfx::RectF {
        gfx::RectF::default()
    }

    fn MotionPathOffsetFromContainingBlock(&self) -> gfx::PointF {
        gfx::PointF::default()
    }

    fn MotionPathBorderBoxSize(&self) -> gfx::SizeF {
        gfx::SizeF::default()
    }

    fn MotionPathHasContainingBlock(&self) -> bool {
        false
    }

    /// The concrete vector-path package lives on the Layout side of this
    /// boundary.  Style keeps the canonical typed shape and asks Layout to
    /// measure it only when a transform is materialized.
    fn MotionPathPointAndTangent(
        &self,
        _shape: &dyn BasicShape,
        _starting_point: &gfx::PointF,
        _reference_size: &gfx::SizeF,
        _distance: &Length,
        _effective_zoom: f32,
    ) -> Option<PointAndTangent> {
        None
    }

    /// Resolves the inset synthesized by a bare `<coord-box>`, including the
    /// containing block's border radii.
    fn MotionPathCoordBoxPointAndTangent(
        &self,
        _reference_size: &gfx::SizeF,
        _distance: &Length,
    ) -> Option<PointAndTangent> {
        None
    }
}

// Callers which explicitly exclude motion-path do not own a LayoutBox (for
// example standalone SVG clip-path transform projection).
impl MotionPathBox for () {}

// cpp: layoutng_style/style/computed_style.h:2098-2102
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplyTransformOrigin {
    kIncludeTransformOrigin,
    kExcludeTransformOrigin,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplyMotionPath {
    kIncludeMotionPath,
    kExcludeMotionPath,
}

// cpp: layoutng_style/style/computed_style.h:2103-2110
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplyIndependentTransformProperties {
    kIncludeIndependentTransformProperties,
    kExcludeIndependentTransformProperties,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplyTransformOperations {
    kIncludeTransformOperations,
    kExcludeTransformOperations,
}

// cpp: layoutng_style/style/computed_style.h:2126-2129
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransformBoxContext {
    kLayoutBox,
    kSvg,
}

#[allow(non_snake_case)]
impl ComputedStyle {
    // cpp: layoutng_style/style/computed_style.h:2452-2453
    // cpp: layoutng_style/style/style_transform.cc:36-62
    pub fn RequireTransformOrigin(
        &self,
        apply_origin: ApplyTransformOrigin,
        apply_motion_path: ApplyMotionPath,
    ) -> bool {
        if apply_origin != ApplyTransformOrigin::kIncludeTransformOrigin {
            return false;
        }

        if apply_motion_path == ApplyMotionPath::kIncludeMotionPath {
            return true;
        }

        for operation in self.Transform().Operations() {
            let operation = operation.GetNonNull().expect("transform operation is null");
            let type_ = unsafe { operation.as_ref().GetType() };
            if type_ != TransformOperationType::kTranslateX
                && type_ != TransformOperationType::kTranslateY
                && type_ != TransformOperationType::kTranslate
                && type_ != TransformOperationType::kTranslateZ
                && type_ != TransformOperationType::kTranslate3D
            {
                return true;
            }
        }

        !self.Scale().is_null() || !self.Rotate().is_null()
    }

    // cpp: layoutng_style/style/style_transform.cc:64-105
    // cpp: layoutng_style/style/computed_style.h:2130
    pub fn UsedTransformBox(&self, box_context: TransformBoxContext) -> ETransformBox {
        let mut transform_box = self.TransformBox();
        if box_context == TransformBoxContext::kSvg {
            transform_box = match transform_box {
                ETransformBox::kContentBox => ETransformBox::kFillBox,
                ETransformBox::kBorderBox => ETransformBox::kStrokeBox,
                ETransformBox::kFillBox | ETransformBox::kStrokeBox | ETransformBox::kViewBox => {
                    transform_box
                }
            };
            if transform_box == ETransformBox::kStrokeBox
                && self.VectorEffect() == EVectorEffect::kNonScalingStroke
            {
                transform_box = ETransformBox::kFillBox;
            }
        } else {
            transform_box = match transform_box {
                ETransformBox::kContentBox | ETransformBox::kBorderBox => transform_box,
                ETransformBox::kFillBox => ETransformBox::kContentBox,
                ETransformBox::kStrokeBox | ETransformBox::kViewBox => ETransformBox::kBorderBox,
            };
        }
        transform_box
    }

    // cpp: layoutng_style/style/computed_style.h:2111-2118
    // cpp: layoutng_style/style/style_transform.cc:107-119
    pub fn ApplyTransformPhysicalRect<B: MotionPathBox>(
        &self,
        result: &mut gfx::Transform,
        box_: Option<&B>,
        reference_box: &PhysicalRect,
        apply_operations: ApplyTransformOperations,
        apply_origin: ApplyTransformOrigin,
        apply_motion_path: ApplyMotionPath,
        apply_independent_transform_properties: ApplyIndependentTransformProperties,
    ) {
        let bounding_box = gfx::RectF::new(
            gfx::PointF::new(
                reference_box.offset.left.ToFloat(),
                reference_box.offset.top.ToFloat(),
            ),
            gfx::SizeF::new(
                reference_box.size.width.ToFloat(),
                reference_box.size.height.ToFloat(),
            ),
        );
        self.ApplyTransformRectF(
            result,
            box_,
            &bounding_box,
            apply_operations,
            apply_origin,
            apply_motion_path,
            apply_independent_transform_properties,
        );
    }

    // cpp: layoutng_style/style/computed_style.h:2119-2125
    // cpp: layoutng_style/style/style_transform.cc:121-183
    pub fn ApplyTransformRectF<B: MotionPathBox>(
        &self,
        result: &mut gfx::Transform,
        box_: Option<&B>,
        bounding_box: &gfx::RectF,
        apply_operations: ApplyTransformOperations,
        apply_origin: ApplyTransformOrigin,
        mut apply_motion_path: ApplyMotionPath,
        apply_independent_transform_properties: ApplyIndependentTransformProperties,
    ) {
        if !self.HasOffset() {
            apply_motion_path = ApplyMotionPath::kExcludeMotionPath;
        }
        let apply_transform_origin = self.RequireTransformOrigin(apply_origin, apply_motion_path);

        let mut origin_x = 0.0_f32;
        let mut origin_y = 0.0_f32;
        let mut origin_z = 0.0_f32;

        let box_size = bounding_box.size();
        if apply_transform_origin || apply_motion_path == ApplyMotionPath::kIncludeMotionPath {
            origin_x = FloatValueForLength(self.GetTransformOrigin().X(), box_size.width())
                + bounding_box.x();
            origin_y = FloatValueForLength(self.GetTransformOrigin().Y(), box_size.height())
                + bounding_box.y();
            if apply_transform_origin {
                origin_z = self.GetTransformOrigin().Z();
                result.Translate3d(origin_x, origin_y, origin_z);
            }
        }

        if apply_independent_transform_properties
            == ApplyIndependentTransformProperties::kIncludeIndependentTransformProperties
        {
            let translate = self.Translate();
            if !translate.is_null() {
                unsafe { &*translate }.Apply(result, &box_size);
            }

            let rotate = self.Rotate();
            if !rotate.is_null() {
                unsafe { &*rotate }.Apply(result, &box_size);
            }

            let scale = self.Scale();
            if !scale.is_null() {
                unsafe { &*scale }.Apply(result, &box_size);
            }
        }

        if apply_motion_path == ApplyMotionPath::kIncludeMotionPath {
            self.ApplyMotionPathTransform(origin_x, origin_y, box_, bounding_box, result);
        }

        if apply_operations == ApplyTransformOperations::kIncludeTransformOperations {
            for operation in self.Transform().Operations() {
                let operation = operation.GetNonNull().expect("transform operation is null");
                unsafe { operation.as_ref().Apply(result, &box_size) };
            }
        }

        if apply_transform_origin {
            result.Translate3d(-origin_x, -origin_y, -origin_z);
        }
    }
}
