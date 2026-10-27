//! Native shape-image lookup for float/shape layout.
//!
//! The resolver validates bitmap inputs at the algorithm boundary, keeping
//! shape-image failures and lookup samples under `layout_algo_float`.

#![allow(non_snake_case)]

use super::*;

// cpp: layoutng/internal/boundary/layout_boundary.cc:113-134
pub(super) struct InputShapeImageResolver {
    images_: Vec<PaintImage>,
}

impl InputShapeImageResolver {
    pub(super) fn new(images: &[PaintImage]) -> Self {
        // The C++ environment borrows ConstraintSpace::images. Owning the
        // immutable snapshot gives the Rust TLS resolver a stable allocation.
        Self {
            images_: images.to_vec(),
        }
    }
}

impl NativeShapeImageResolver for InputShapeImageResolver {
    fn Resolve(&self, resource_id: u64) -> *const PaintImage {
        let Some(image) = self.images_.iter().find(|image| image.id == resource_id) else {
            return std::ptr::null();
        };
        let expected_pixels = (image.width as usize)
            .checked_mul(image.height as usize)
            .and_then(|pixels| pixels.checked_mul(4));
        if image.width == 0
            || image.height == 0
            || !image.resolution_scale.is_finite()
            || image.resolution_scale <= 0.0
            || image
                .BitmapPixels()
                .is_some_and(|pixels| expected_pixels != Some(pixels.len()))
        {
            panic!("shape image requires dimensions, scale and RGBA pixels");
        }
        image as *const PaintImage
    }
}
