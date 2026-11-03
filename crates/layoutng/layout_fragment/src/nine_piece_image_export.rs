//! Chromium nine-piece image resolution for fragment paint payloads.
//!
//! Border, mask-box and reflection image resource lookup is fragment
//! publication work and retains a package-qualified sampling entry here.

#![allow(non_snake_case)]

use super::*;

#[inline(never)]
pub(super) fn ExportNinePieceImages(
    style: &StyleComputedStyle,
    size: Size,
    resources: &PaintResources,
    paint: &mut PaintProperties,
) {
    let borders = paint.border;
    let export = |source: &layoutng_style::style::nine_piece_image::NinePieceImage,
                  image_area_size: Size| {
        let resource_id = unsafe { source.GetImage().as_ref() }.and_then(
            crate::internal::boundary::paint_style_export::FetchedNinePieceImageResourceId,
        );
        let resource =
            resource_id.and_then(|id| resources.images.iter().find(|image| image.id == id));
        crate::internal::boundary::paint_style_export::ExportNinePieceImageForFragment(
            style,
            source,
            image_area_size,
            borders,
            resource,
        )
    };
    paint.style.border_image = export(style.BorderImage(), size);
    paint.style.mask_box_image = export(style.MaskBoxImage(), size);
    if let Some(reflection) = unsafe { style.BoxReflect().as_ref() } {
        if reflection.Mask().HasImage() {
            let stitched_size = paint
                .stitched_decoration
                .as_ref()
                .map_or(size, |stitched| stitched.size);
            paint.style.box_reflection_mask_image = export(reflection.Mask(), stitched_size);
        }
    }
}
