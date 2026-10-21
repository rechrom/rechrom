use super::SkImage::{SkImage, SkImages::BitDepth};
use super::SkImage_Picture::SkImage_Picture;
use crate::include::core::{SkRefCnt::sk_sp, SkSize::SkISize, SkSurfaceProps::SkSurfaceProps};
use crate::src::core::{
    SkColorSpace::SkColorSpace, SkMatrix::SkMatrix, SkPaint::SkPaint, SkPicture::SkPicture,
};
use std::sync::Arc;
// Source: src/image/SkImage_LazyFactories.cpp:23-31
pub fn DeferredFromPicture(
    picture: sk_sp<dyn SkPicture>,
    dimensions: &SkISize,
    matrix: Option<&SkMatrix>,
    paint: Option<&SkPaint>,
    bitDepth: BitDepth,
    colorSpace: Option<Arc<SkColorSpace>>,
) -> Option<Arc<dyn SkImage>> {
    SkImage_Picture::Make(
        picture,
        dimensions,
        matrix,
        paint,
        bitDepth,
        colorSpace,
        SkSurfaceProps::default(),
    )
}
// Source: src/image/SkImage_LazyFactories.cpp:33-42
// Rust selects the explicit native overload by name, preserving its props value.
pub fn DeferredFromPicture_with_props(
    picture: sk_sp<dyn SkPicture>,
    dimensions: &SkISize,
    matrix: Option<&SkMatrix>,
    paint: Option<&SkPaint>,
    bitDepth: BitDepth,
    colorSpace: Option<Arc<SkColorSpace>>,
    props: SkSurfaceProps,
) -> Option<Arc<dyn SkImage>> {
    SkImage_Picture::Make(
        picture, dimensions, matrix, paint, bitDepth, colorSpace, props,
    )
}
