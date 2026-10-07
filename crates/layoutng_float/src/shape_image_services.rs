#![allow(non_snake_case)]

use foundation::UnsupportedLayout;
use layoutng_assembly::internal::layout_input::PaintImage;
use layoutng_assembly::internal::shapes::shape_image_services::CurrentShapeImageResolver;

// cpp: layoutng_float/shape_image_services.cc:8-19
#[unsafe(no_mangle)]
pub extern "Rust" fn CurrentShapeImage(resource_id: u64) -> *const PaintImage {
    let resolver = CurrentShapeImageResolver().unwrap_or_else(|| {
        std::panic::panic_any(UnsupportedLayout::new(
            "shape image lookup requires a Layout ConstraintSpace",
        ))
    });
    let image = unsafe { &*resolver }.Resolve(resource_id);
    if image.is_null() {
        std::panic::panic_any(UnsupportedLayout::new(
            "shape-outside image is missing from ConstraintSpace::images",
        ));
    }
    image
}
