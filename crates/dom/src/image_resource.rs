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
    AddImageResourceWithContent(owner, constraints, source, image, None);
}
pub fn AddImageResourceWithContent(
    owner: &mut DOM,
    constraints: &mut ConstraintSpace,
    source: String,
    image: PaintImage,
    content: Option<std::rc::Rc<crate::persistent_document::DOMImageResourceContent>>,
) {
    if source.is_empty() {
        invalid_argument("image source must not be empty");
    }
    if image.id == 0
        || image.width == 0
        || image.height == 0
        || !image.resolution_scale.is_finite()
        || image.resolution_scale <= 0.0
        || image
            .content_dpr
            .is_some_and(|value| !value.is_finite() || value <= 0.0)
        || image.BitmapPixels().is_some_and(|pixels| {
            pixels.len()
                != (image.width as usize)
                    .wrapping_mul(image.height as usize)
                    .wrapping_mul(4)
        })
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
    let is_document_image = image.IsDocumentImage();
    let maybe_animated = image.MaybeAnimated();
    let known_to_be_opaque = image
        .BitmapPixels()
        .is_some_and(|pixels| pixels.chunks_exact(4).all(|pixel| pixel[3] == 255));
    owner
        .GetDocumentMut()
        .SetImageResourceIsDocument(image.id, is_document_image);
    owner
        .GetDocumentMut()
        .SetImageResourceMaybeAnimated(image.id, maybe_animated);
    owner
        .GetDocumentMut()
        .SetImageResourceContentDPR(image.id, image.content_dpr);
    // Populate the same canonical content before synchronous completion
    // notification; Style and input retain this immutable decoder value.
    let content = content
        .or_else(|| owner.GetDocument().ImageResourceContentFor(&source))
        .unwrap_or_else(crate::persistent_document::DOMImageResourceContent::Pending);
    content.SetIsSVGImage(is_document_image);
    content.SetImageSnapshot(std::sync::Arc::new(image.clone()));
    owner.GetDocumentMut().SetImageResourceWithContent(
        source,
        ImageResourceMetadata {
            id: image.id,
            natural_width,
            natural_height,
            resolution_scale: image.resolution_scale,
            orientation: image.orientation,
            known_to_be_opaque,
            is_cors_same_origin: image.is_cors_same_origin,
        },
        Some(content),
    );
    constraints.images.push(image);
}
