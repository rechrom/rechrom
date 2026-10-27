//! Native nine-piece image resolution for fragment paint payloads.
//!
//! Border, mask-box and reflection image resource lookup is fragment
//! publication work and retains a package-qualified sampling entry here.

#![allow(non_snake_case)]

use super::*;

#[inline(never)]
pub(super) fn ExportNativeNinePieceImages(
    style: &NativeComputedStyle,
    size: Size,
    resources: &PaintResources,
    paint: &mut PaintProperties,
) {
    let borders = paint.border;
    let export = |native: &layoutng_style::style::nine_piece_image::NinePieceImage,
                  image_area_size: Size| {
        let resource_id = unsafe { native.GetImage().as_ref() }.and_then(
            crate::internal::boundary::native_paint_style::FetchedNinePieceImageResourceId,
        );
        let resource =
            resource_id.and_then(|id| resources.images.iter().find(|image| image.id == id));
        crate::internal::boundary::native_paint_style::ExportNinePieceImageForFragment(
            style,
            native,
            image_area_size,
            borders,
            resource,
        )
    };
    paint.style.border_image = export(style.BorderImage(), size);
    paint.style.mask_box_image = export(style.MaskBoxImage(), size);
    if let Some(reflection) = unsafe { style.BoxReflect().as_ref() } {
        if reflection.Mask().HasImage() {
            if let Some(filter) = paint.style.filters.iter_mut().find(|filter| {
                filter.r#type == crate::internal::paint_input::PaintFilterType::kBoxReflect
            }) {
                let stitched_size = paint
                    .stitched_decoration
                    .as_ref()
                    .map_or(size, |stitched| stitched.size);
                filter.reflection_mask_image = export(reflection.Mask(), stitched_size);
            }
        }
    }
}
