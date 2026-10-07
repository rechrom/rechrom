#![allow(non_snake_case)]

use layoutng_assembly::internal::layout_input::{ConstraintSpace, PaintImage};

use crate::error::invalid_argument;
use crate::{persistent_document::DOM, ImageResourceMetadata};

// cpp: dom/image_resource.h:15-18
// cpp: dom/image_resource.cc:14-39
pub fn AddImageResource(
    owner: &mut DOM,
    constraints: &mut ConstraintSpace,
    source: String,
    image: PaintImage,
) {
    if source.is_empty() {
        invalid_argument("image source must not be empty");
    }
    if image.id == 0
        || image.width == 0
        || image.height == 0
        || !image.resolution_scale.is_finite()
        || image.resolution_scale <= 0.0
        || image.rgba8.len()
            != (image.width as usize)
                .wrapping_mul(image.height as usize)
                .wrapping_mul(4)
    {
        invalid_argument("image resource requires an id, dimensions, scale and RGBA pixels");
    }
    let duplicate_id = constraints
        .images
        .iter()
        .any(|existing| existing.id == image.id);
    if duplicate_id {
        invalid_argument("image resource id must be unique");
    }

    let natural_width = image.width as f64 / image.resolution_scale;
    let natural_height = image.height as f64 / image.resolution_scale;
    owner.GetDocumentMut().SetImageResource(
        source,
        ImageResourceMetadata {
            id: image.id,
            natural_width,
            natural_height,
            resolution_scale: image.resolution_scale,
        },
    );
    constraints.images.push(image);
}
