use super::SkImage::SkImages::BitDepth;
use crate::include::core::SkAlphaType::SkAlphaType::kPremul_SkAlphaType;
use crate::include::core::SkColorType::{kN32_SkColorType, SkColorType::kRGBA_F16_SkColorType};
use crate::include::core::{SkRefCnt::sk_sp, SkSize::SkISize, SkSurfaceProps::SkSurfaceProps};
use crate::src::core::{
    SkCanvas::SkCanvas,
    SkColorSpace::SkColorSpace,
    SkImageGenerator::{Options, SkImageGenerator, SkImageGeneratorState},
    SkImageInfo::SkImageInfo,
    SkMatrix::SkMatrix,
    SkPaint::SkPaint,
    SkPicture::SkPicture,
};
use std::ptr::NonNull;
use std::sync::Arc;

// Source: src/image/SkPictureImageGenerator.h:22-25
// Source: src/image/SkPictureImageGenerator.h:31-37
// Contiguous native fields retain declaration order. Base state is embedded in
// the same unique generator allocation. Visibility maps the native image friend.
pub struct SkPictureImageGenerator {
    pub(super) fPicture: sk_sp<dyn SkPicture>,
    pub(super) fMatrix: SkMatrix,
    pub(super) fPaint: Option<SkPaint>,
    pub(super) fProps: SkSurfaceProps,
    base: SkImageGeneratorState,
}
// The optional Paint and Picture are the only nontrivial derived field owners.
// Release them in native reverse declaration order before the actual base state.
impl Drop for SkPictureImageGenerator {
    fn drop(&mut self) {
        drop(self.fPaint.take());
        drop(std::mem::take(&mut self.fPicture));
    }
}
// Source: src/image/SkPictureImageGenerator.cpp:28-36
// Rust gives the six-argument native overload an explicit name.
pub fn MakeFromPicture_default(
    size: &SkISize,
    picture: sk_sp<dyn SkPicture>,
    matrix: Option<&SkMatrix>,
    paint: Option<&SkPaint>,
    bitDepth: BitDepth,
    colorSpace: Option<Arc<SkColorSpace>>,
) -> Option<Box<dyn SkImageGenerator>> {
    MakeFromPicture(
        size,
        picture.clone(),
        matrix,
        paint,
        bitDepth,
        colorSpace.clone(),
        SkSurfaceProps::default(),
    )
}
// Source: src/image/SkPictureImageGenerator.cpp:38-58
pub fn MakeFromPicture(
    size: &SkISize,
    picture: sk_sp<dyn SkPicture>,
    matrix: Option<&SkMatrix>,
    paint: Option<&SkPaint>,
    bitDepth: BitDepth,
    colorSpace: Option<Arc<SkColorSpace>>,
    props: SkSurfaceProps,
) -> Option<Box<dyn SkImageGenerator>> {
    if picture.is_none() || colorSpace.is_none() || size.isEmpty() {
        return None;
    }
    let mut colorType = kN32_SkColorType;
    if bitDepth == BitDepth::kF16 {
        colorType = kRGBA_F16_SkColorType;
    }
    let info = SkImageInfo::MakeDimensionsWithColorSpace(
        *size,
        colorType,
        kPremul_SkAlphaType,
        colorSpace,
    );
    Some(Box::new(SkPictureImageGenerator::new(
        &info, picture, matrix, paint, &props,
    )))
}
impl SkPictureImageGenerator {
    // Source: src/image/SkPictureImageGenerator.cpp:63-79
    pub fn new(
        info: &SkImageInfo,
        picture: sk_sp<dyn SkPicture>,
        matrix: Option<&SkMatrix>,
        paint: Option<&SkPaint>,
        props: &SkSurfaceProps,
    ) -> Self {
        let mut out = Self {
            base: SkImageGeneratorState::new(info, 0),
            fPicture: picture,
            fMatrix: SkMatrix::default(),
            fPaint: None,
            fProps: *props,
        };
        if let Some(matrix) = matrix {
            out.fMatrix = matrix.clone();
        } else {
            out.fMatrix.reset();
        }
        if let Some(paint) = paint {
            out.fPaint = Some(paint.clone());
        }
        out
    }
    // Source: src/image/SkPictureImageGenerator.h:28-28
    // Source: src/image/SkPictureImageGenerator.cpp:81-90
    /// Safety: pixels are the caller's exclusively leased writable allocation,
    /// valid for this layout until the actual direct raster Canvas is destroyed.
    unsafe fn onGetPixels_impl(
        &self,
        info: &SkImageInfo,
        pixels: *mut u8,
        rowBytes: usize,
        _opts: &Options,
    ) -> bool {
        let Some(mut canvas) =
            SkCanvas::MakeRasterDirect(info, pixels.cast(), rowBytes, Some(&self.fProps))
        else {
            return false;
        };
        let canvasPtr = NonNull::from(canvas.as_mut().get_unchecked_mut());
        // Adapted cpp87-89: Canvas playback has a real fallible Rust contract,
        // while the native generator virtual returns bool. Actual draw failure
        // becomes generator failure; never report successful pixels on error.
        if SkCanvas::clear_raw(canvasPtr, 0).is_err() {
            return false;
        }
        if SkCanvas::drawPicture_sp_raw(
            canvasPtr,
            &self.fPicture,
            Some(&self.fMatrix),
            self.fPaint.as_ref(),
        )
        .is_err()
        {
            return false;
        }
        true
    }
}
// Exact virtual forwarding to the actual unique concrete decoder state.
impl SkImageGenerator for SkPictureImageGenerator {
    fn state(&self) -> &SkImageGeneratorState {
        &self.base
    }
    unsafe fn onGetPixels(
        &self,
        info: &SkImageInfo,
        pixels: *mut u8,
        rowBytes: usize,
        opts: &Options,
    ) -> bool {
        unsafe { self.onGetPixels_impl(info, pixels, rowBytes, opts) }
    }
}
