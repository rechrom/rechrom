#![allow(non_snake_case)]

use foundation::blink_geometry::transforms::affine_transform::AffineTransform;
use foundation::DynamicTo;
use layoutng_assembly::internal::layout_box_model_object::LayoutBoxModelObject;
use layoutng_assembly::internal::layout_object::LayoutObject;

use crate::subtree_content_transform_scope::SubtreeContentTransformScope;

// cpp: layoutng_svg/svg_text_scale.cc:31-53
fn DeprecatedCalculateTransformToLayer(object: *const LayoutObject) -> AffineTransform {
    let mut transform = AffineTransform::default();
    let mut object = object;
    while !object.is_null() {
        let object_ref = unsafe { &*object };
        let mut local = object_ref.LocalToSVGParentTransform();
        local.PreConcat(transform);
        transform = local;
        if object_ref.IsSVGRoot() {
            break;
        }
        object = object_ref.Parent();
    }
    while !object.is_null() {
        let box_model = DynamicTo::<LayoutBoxModelObject>(object);
        if !box_model.is_null() {
            let layer_transform = unsafe { &*box_model }.TransformForLayout();
            if !layer_transform.is_null() {
                let mut layer = AffineTransform::FromTransform(unsafe { &*layer_transform });
                layer.PreConcat(transform);
                transform = layer;
            }
        }
        object = unsafe { &*object }.Parent();
    }
    transform
}

// cpp: layoutng_svg/svg_text_scale.cc:55-67
pub fn CalculateScreenFontSizeScalingFactor(object: &LayoutObject) -> f32 {
    let mut ctm = DeprecatedCalculateTransformToLayer(object);
    ctm.PreConcat(SubtreeContentTransformScope::CurrentContentTransformation());
    let x_scale_squared = ctm.A() * ctm.A() + ctm.B() * ctm.B();
    let y_scale_squared = ctm.C() * ctm.C() + ctm.D() * ctm.D();
    let scale = ((x_scale_squared + y_scale_squared) / 2.0).sqrt();
    scale.min(f32::MAX as f64) as f32
}
