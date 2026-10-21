#![allow(non_snake_case)]

use foundation::{gfx, DynamicTo, PhysicalOffset, PhysicalSize, PointForLengthPoint, To};
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::style_difference::StyleDifference;
use layoutng_style::style::style_transform::{
    ApplyIndependentTransformProperties, ApplyMotionPath, ApplyTransformOperations,
    ApplyTransformOrigin,
};

use super::layout_box::LayoutBox;
use super::layout_box_model_object::LayoutBoxModelObject;
use super::layout_node_metadata::Element;
use super::layout_object::LayoutObject;
use super::transform_utils::ComputeReferenceBoxForLayoutBox;

impl LayoutBoxModelObject {
    // cpp: layoutng/internal/layout_transform.cc:53-79
    pub fn UpdateTransformForLayout(&mut self) {
        if !self.HasTransform() {
            self.transform_ = None;
            return;
        }
        if self.transform_.is_none() {
            self.transform_ = Some(Box::new(gfx::Transform::default()));
        } else {
            self.transform_.as_mut().unwrap().MakeIdentity();
        }
        let element = DynamicTo::<Element>(self.GetNode());
        if !element.is_null() {
            if let Some(canvas_transform) = unsafe { &*element }.GetUsedCanvasTransform() {
                self.transform_
                    .as_mut()
                    .unwrap()
                    .PreConcat(canvas_transform);
            }
        }
        let box_ = DynamicTo::<LayoutBox>(self as *const LayoutBoxModelObject);
        if !box_.is_null() {
            let box_ref = unsafe { &*box_ };
            let reference_box = ComputeReferenceBoxForLayoutBox(box_ref);
            box_ref.StyleRef().ApplyTransformPhysicalRect(
                self.transform_.as_mut().unwrap(),
                Some(box_ref),
                &reference_box,
                ApplyTransformOperations::kIncludeTransformOperations,
                ApplyTransformOrigin::kIncludeTransformOrigin,
                ApplyMotionPath::kIncludeMotionPath,
                ApplyIndependentTransformProperties::kIncludeIndependentTransformProperties,
            );
        }
    }

    // cpp: layoutng/internal/layout_transform.cc:81-96
    pub fn UpdateTransformAfterStyleChangeForLayout(
        &mut self,
        diff: StyleDifference,
        old_style: *const ComputedStyle,
        _new_style: &ComputedStyle,
    ) {
        let had_transform = !self.TransformForLayout().is_null();
        let has_transform = self.HasTransform();
        if had_transform == has_transform && !old_style.is_null() && !diff.transform_data_changed()
        {
            return;
        }
        self.UpdateTransformForLayout();
    }

    // cpp: layoutng/internal/layout_transform.cc:98-102
    pub fn CurrentTransformForLayout(&self) -> gfx::Transform {
        let transform = self.TransformForLayout();
        if !transform.is_null() {
            return unsafe { &*transform }.clone();
        }
        gfx::Transform::default()
    }
}

impl LayoutObject {
    // cpp: layoutng/internal/layout_transform.cc:104-114
    pub fn HasTransform(&self) -> bool {
        self.CheckIsNotDestroyed();
        if self.IsBoxModelObject() {
            let element = DynamicTo::<Element>(self.GetNode());
            if !element.is_null() && unsafe { &*element }.GetUsedCanvasTransform().is_some() {
                return true;
            }
        }
        self.HasTransformRelatedProperty() && self.StyleRef().HasTransform()
    }

    // cpp: layoutng/internal/layout_transform.cc:116-123
    pub fn ShouldUseTransformFromContainer(&self, container_object: *const LayoutObject) -> bool {
        self.CheckIsNotDestroyed();
        (self.IsBoxModelObject() && {
            let model = To::<LayoutBoxModelObject>(self as *const LayoutObject);
            !unsafe { &*model }.TransformForLayout().is_null()
        }) || (!container_object.is_null() && unsafe { &*container_object }.HasPerspective())
    }

    // cpp: layoutng/internal/layout_transform.cc:125-164
    pub fn GetTransformFromContainer(
        &self,
        container_object: *const LayoutObject,
        offset_in_container: PhysicalOffset,
        transform: &mut gfx::Transform,
        size: *const PhysicalSize,
        fragment_transform: Option<&gfx::Transform>,
    ) {
        self.CheckIsNotDestroyed();
        transform.MakeIdentity();
        if let Some(fragment_transform) = fragment_transform {
            transform.PreConcat(fragment_transform);
        } else {
            let box_model = DynamicTo::<LayoutBoxModelObject>(self as *const LayoutObject);
            if !box_model.is_null() && !unsafe { &*box_model }.TransformForLayout().is_null() {
                let current = unsafe { &*box_model }.CurrentTransformForLayout();
                transform.PreConcat(&current);
            }
        }

        transform.PostTranslate(
            offset_in_container.left.ToFloat(),
            offset_in_container.top.ToFloat(),
        );

        let mut has_perspective =
            !container_object.is_null() && unsafe { &*container_object }.HasPerspective();
        if has_perspective && container_object != self.NearestAncestorForElement() {
            has_perspective = false;
        }
        if has_perspective {
            let mut perspective_origin = gfx::PointF::default();
            let container_box = DynamicTo::<LayoutBox>(container_object);
            if !container_box.is_null() {
                perspective_origin = unsafe { &*container_box }.PerspectiveOrigin(size);
            }
            let mut perspective_matrix = gfx::Transform::default();
            perspective_matrix.ApplyPerspectiveDepth(
                unsafe { &*container_object }.StyleRef().UsedPerspective() as f64,
            );
            perspective_matrix.ApplyTransformOrigin(
                perspective_origin.x(),
                perspective_origin.y(),
                0.0,
            );
            *transform = perspective_matrix * transform.clone();
        }
    }

    // cpp: layoutng/internal/layout_transform.cc:166-173
    pub fn NearestAncestorForElement(&self) -> *mut LayoutObject {
        self.CheckIsNotDestroyed();
        let mut ancestor = self.Parent();
        while !ancestor.is_null() && unsafe { &*ancestor }.IsAnonymous() {
            ancestor = unsafe { &*ancestor }.Parent();
        }
        ancestor
    }
}

impl LayoutBox {
    // cpp: layoutng/internal/layout_transform.cc:175-184
    pub fn PerspectiveOrigin(&self, size: *const PhysicalSize) -> gfx::PointF {
        self.CheckIsNotDestroyed();
        if !self.HasTransformRelatedProperty() {
            return gfx::PointF::default();
        }
        let float_size = if !size.is_null() {
            gfx::SizeF::from(unsafe { *size })
        } else {
            gfx::SizeF::from(self.StitchedSize())
        };
        PointForLengthPoint(self.StyleRef().PerspectiveOrigin(), &float_size)
    }
}
