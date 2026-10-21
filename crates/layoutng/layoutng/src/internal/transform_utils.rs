#![allow(non_snake_case)]

use foundation::{
    gfx, ETransformBox, PhysicalOffset, PhysicalRect, PhysicalSize, To, TransformState,
};
use layoutng_fragment_tree::physical_box_fragment::PhysicalBoxFragment;
use layoutng_fragment_tree::physical_fragment::PhysicalFragment;
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
use layoutng_style::style::computed_style_constants::CoordBox;
use layoutng_style::style::style_transform::{
    ApplyIndependentTransformProperties, ApplyMotionPath, ApplyTransformOperations,
    ApplyTransformOrigin, MotionPathBox, TransformBoxContext,
};

use super::layout_box::LayoutBox;
use super::layout_box_model_object::LayoutBoxModelObject;
use super::layout_object::LayoutObject;
use super::map_coordinates_flags::MapCoordinatesMode;

// cpp: layoutng_style/style/computed_style.cc:1506-1551
// This is the narrow LayoutNG implementation of the geometry interface used
// by ComputedStyle's translated motion-path algorithm.
#[allow(non_snake_case)]
impl MotionPathBox for LayoutBox {
    fn MotionPathReferenceBox(&self, coord_box: CoordBox) -> gfx::RectF {
        let containing = self.ContainingBlock();
        if containing.is_null() {
            return gfx::RectF::default();
        }
        let containing = unsafe { &*containing };
        // The current SVG layout bridge does not yet expose
        // SVGViewportResolver.  Its CSS box still provides the correct
        // fallback used while native SVG geometry is unavailable.
        let rect = match coord_box {
            CoordBox::kFillBox | CoordBox::kContentBox => containing.PhysicalContentBoxRect(),
            CoordBox::kPaddingBox => containing.PhysicalPaddingBoxRect(),
            CoordBox::kViewBox | CoordBox::kStrokeBox | CoordBox::kBorderBox => {
                containing.PhysicalBorderBoxRect()
            }
        };
        gfx::RectF::new(
            gfx::PointF::new(rect.offset.left.ToFloat(), rect.offset.top.ToFloat()),
            gfx::SizeF::new(rect.size.width.ToFloat(), rect.size.height.ToFloat()),
        )
    }

    fn MotionPathOffsetFromContainingBlock(&self) -> gfx::PointF {
        let containing = self.ContainingBlock();
        if containing.is_null() {
            return gfx::PointF::default();
        }
        self.LocalToAncestorPoint(
            &gfx::PointF::default(),
            containing as *const LayoutBoxModelObject,
            MapCoordinatesMode::kIgnoreTransforms.into(),
        )
    }

    fn MotionPathBorderBoxSize(&self) -> gfx::SizeF {
        let rect = self.PhysicalBorderBoxRect();
        gfx::SizeF::new(rect.size.width.ToFloat(), rect.size.height.ToFloat())
    }

    fn MotionPathHasContainingBlock(&self) -> bool {
        !self.ContainingBlock().is_null()
    }

    fn MotionPathPointAndTangent(
        &self,
        shape: &dyn layoutng_style::style::basic_shapes::BasicShape,
        starting_point: &gfx::PointF,
        reference_size: &gfx::SizeF,
        distance: &foundation::Length,
        effective_zoom: f32,
    ) -> Option<foundation::PointAndTangent> {
        super::motion_path_geometry::PointAndTangentForShape(
            shape,
            starting_point,
            reference_size,
            distance,
            effective_zoom,
        )
    }

    fn MotionPathCoordBoxPointAndTangent(
        &self,
        reference_size: &gfx::SizeF,
        distance: &foundation::Length,
    ) -> Option<foundation::PointAndTangent> {
        let containing = self.ContainingBlock();
        if containing.is_null() {
            return None;
        }
        let style = unsafe { &*containing }.StyleRef();
        let mut inset = layoutng_style::style::basic_shapes::BasicShapeInset::default();
        inset.SetTopLeftRadius(style.BorderTopLeftRadius());
        inset.SetTopRightRadius(style.BorderTopRightRadius());
        inset.SetBottomRightRadius(style.BorderBottomRightRadius());
        inset.SetBottomLeftRadius(style.BorderBottomLeftRadius());
        super::motion_path_geometry::PointAndTangentForShape(
            &inset,
            &gfx::PointF::default(),
            reference_size,
            distance,
            1.0,
        )
    }
}

// cpp: layoutng/internal/transform_utils.cc:17-34
fn ComputeReferenceBoxInternal(
    fragment: &PhysicalBoxFragment,
    border_box_rect: PhysicalRect,
) -> PhysicalRect {
    let mut fragment_reference_box = border_box_rect;
    match fragment
        .Style()
        .UsedTransformBox(TransformBoxContext::kLayoutBox)
    {
        ETransformBox::kContentBox => {
            fragment_reference_box.Contract(&(fragment.Borders() + fragment.Padding()));
            fragment_reference_box.size.ClampNegativeToZero();
        }
        ETransformBox::kBorderBox => {}
        ETransformBox::kFillBox | ETransformBox::kStrokeBox | ETransformBox::kViewBox => {
            panic!("unreachable transform box for a layout box")
        }
    }
    fragment_reference_box
}

// cpp: layoutng/internal/transform_utils.h:23-23
// cpp: layoutng/internal/transform_utils.cc:38-40
pub fn ComputeReferenceBoxForFragment(fragment: &PhysicalBoxFragment) -> PhysicalRect {
    ComputeReferenceBoxInternal(fragment, fragment.LocalRect())
}

// cpp: layoutng/internal/transform_utils.h:24-24
// cpp: layoutng/internal/transform_utils.cc:42-49
pub fn ComputeReferenceBoxForLayoutBox(box_: &LayoutBox) -> PhysicalRect {
    if box_.PhysicalFragmentCount() == 0 {
        return PhysicalRect::default();
    }
    let fragment = box_.GetPhysicalFragment(0);
    debug_assert!(!fragment.is_null());
    ComputeReferenceBoxInternal(unsafe { &*fragment }, box_.PhysicalBorderBoxRect())
}

// cpp: layoutng/internal/transform_utils.h:27-30
// cpp: layoutng/internal/transform_utils.cc:51-84
pub fn GetTransformForChildFragment(
    child_fragment: &PhysicalBoxFragment,
    container_object: &LayoutObject,
    container_size: PhysicalSize,
) -> Option<gfx::Transform> {
    let child_layout_object = child_fragment.GetLayoutObject();
    debug_assert!(!child_layout_object.is_null());
    let child_layout_object = unsafe { &*child_layout_object };

    if !child_layout_object.ShouldUseTransformFromContainer(container_object) {
        return None;
    }

    let mut fragment_transform = None;
    if !child_fragment.IsOnlyForNode() {
        let mut computed_transform = gfx::Transform::default();
        computed_transform.MakeIdentity();
        let reference_box = ComputeReferenceBoxForFragment(child_fragment);
        let container_box = To::<LayoutBox>(container_object as *const LayoutObject);
        child_fragment.Style().ApplyTransformPhysicalRect(
            &mut computed_transform,
            Some(unsafe { &*container_box }),
            &reference_box,
            ApplyTransformOperations::kIncludeTransformOperations,
            ApplyTransformOrigin::kIncludeTransformOrigin,
            ApplyMotionPath::kIncludeMotionPath,
            ApplyIndependentTransformProperties::kIncludeIndependentTransformProperties,
        );
        fragment_transform = Some(computed_transform);
    }

    let mut transform = gfx::Transform::default();
    child_layout_object.GetTransformFromContainer(
        container_object,
        PhysicalOffset::default(),
        &mut transform,
        &container_size,
        fragment_transform.as_ref(),
    );
    Some(transform)
}

// cpp: layoutng/internal/transform_utils.h:34-38
// cpp: layoutng/internal/transform_utils.cc:86-108
pub fn UpdateTransformState(
    child_fragment: &PhysicalFragment,
    child_offset: PhysicalOffset,
    container_object: &LayoutObject,
    container_size: PhysicalSize,
    transform_state: &mut TransformState,
) {
    let accumulation = if container_object.StyleRef().Preserves3D() {
        TransformState::kAccumulateTransform
    } else {
        TransformState::kFlattenTransform
    };

    if child_fragment.IsCSSBox() {
        let child_box =
            unsafe { &*To::<PhysicalBoxFragment>(child_fragment as *const PhysicalFragment) };
        if let Some(transform) =
            GetTransformForChildFragment(child_box, container_object, container_size)
        {
            let child_object = child_fragment.GetLayoutObject();
            if !child_object.is_null()
                && unsafe { &*child_object }.ShouldUseTransformFromContainer(container_object)
            {
                transform_state.ApplyTransform(&transform, accumulation);
            }
        }
    }
    transform_state.Move(child_offset, accumulation);
}
