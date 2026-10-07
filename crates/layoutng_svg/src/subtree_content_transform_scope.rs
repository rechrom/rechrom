#![allow(non_snake_case)]

use foundation::blink_geometry::transforms::affine_transform::AffineTransform;
use std::sync::Mutex;

// cpp: layoutng_svg/subtree_content_transform_scope.cc:28-28
static CURRENT_CONTENT_TRANSFORMATION: Mutex<AffineTransform> =
    Mutex::new(AffineTransform::new(1.0, 0.0, 0.0, 1.0, 0.0, 0.0));

// cpp: layoutng_svg/svg_layout_support.h:112-126
pub struct SubtreeContentTransformScope {
    saved_content_transformation: AffineTransform,
}

impl SubtreeContentTransformScope {
    // cpp: layoutng_svg/subtree_content_transform_scope.cc:30-34
    pub fn new(subtree_content_transformation: AffineTransform) -> Self {
        let mut current = CURRENT_CONTENT_TRANSFORMATION.lock().unwrap();
        let saved_content_transformation = *current;
        current.PostConcat(subtree_content_transformation);
        Self {
            saved_content_transformation,
        }
    }

    // cpp: layoutng_svg/svg_layout_support.h:119-121
    pub fn CurrentContentTransformation() -> AffineTransform {
        *CURRENT_CONTENT_TRANSFORMATION.lock().unwrap()
    }
}

impl Drop for SubtreeContentTransformScope {
    // cpp: layoutng_svg/subtree_content_transform_scope.cc:36-38
    fn drop(&mut self) {
        *CURRENT_CONTENT_TRANSFORMATION.lock().unwrap() = self.saved_content_transformation;
    }
}
