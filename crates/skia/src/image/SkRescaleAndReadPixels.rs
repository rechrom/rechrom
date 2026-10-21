// Pinned native implementation; unverified as instructed.
use core::mem::MaybeUninit;
use std::sync::Arc;
// Source: src/image/SkRescaleAndReadPixels.h:11-14
use super::{
    SkImage::{
        AsyncReadResult, ReadPixelsCallback, ReadPixelsContext, RescaleGamma, RescaleMode, SkImage,
    },
    SkSurface, SkSurface_Raster,
};
use crate::{
    include::core::{
        SkColorType::SkColorType,
        SkRefCnt::sk_sp,
        SkSamplingOptions::{SkCubicResampler, SkFilterMode, SkSamplingOptions},
    },
    src::core::{
        SkBitmap::SkBitmap,
        SkBlendMode::SkBlendMode,
        SkCanvas::{SkCanvas, SrcRectConstraint},
        SkImageInfo::SkImageInfo,
        SkPaint::SkPaint,
        SkPixmap::SkPixmap,
        SkRect::{SkIRect, SkRect},
    },
    RasterError,
};
// Source: src/image/SkRescaleAndReadPixels.h:17-23
// Source: src/image/SkRescaleAndReadPixels.cpp:28-166
/// Safety: bmp's initialized pixels and every nonowning pixmap retain their real
/// backing lease through callback completion. Valid resultInfo/srcRect follow
/// the public Image/Surface validation. Callback/context obey the same synchronous
/// native lease; backend errors propagate explicitly without a fake failed result.
pub unsafe fn SkRescaleAndReadPixels(
    bmp: SkBitmap,
    resultInfo: &SkImageInfo,
    srcRect: &SkIRect,
    rescaleGamma: RescaleGamma,
    mut rescaleMode: RescaleMode,
    callback: ReadPixelsCallback,
    context: ReadPixelsContext,
) -> Result<(), RasterError> {
    let mut srcW = srcRect.width();
    let mut srcH = srcRect.height();
    let sx = resultInfo.width() as f32 / srcW as f32;
    let sy = resultInfo.height() as f32 / srcH as f32;
    let (mut stepsX, mut stepsY) = if rescaleMode != RescaleMode::kNearest {
        (
            if sx > 1.0 {
                sx.log2().ceil() as i32
            } else {
                sx.log2().floor() as i32
            },
            if sy > 1.0 {
                sy.log2().ceil() as i32
            } else {
                sy.log2().floor() as i32
            },
        )
    } else {
        (i32::from(sx != 1.0), i32::from(sy != 1.0))
    };
    let mut paint = SkPaint::default();
    paint.setBlendMode(SkBlendMode::kSrc);
    if stepsX < 0 || stepsY < 0 {
        if rescaleMode != RescaleMode::kNearest {
            rescaleMode = RescaleMode::kRepeatedLinear;
        }
    }
    let sampling = match rescaleMode {
        RescaleMode::kRepeatedLinear => SkSamplingOptions::with_filter(SkFilterMode::kLinear),
        RescaleMode::kRepeatedCubic => SkSamplingOptions::with_cubic(SkCubicResampler {
            B: 1.0f32 / 3.0,
            C: 1.0f32 / 3.0,
        }),
        _ => SkSamplingOptions::default(),
    };
    // Preserve the native Surface retention across snapshot reads and all scale steps.
    let mut tempSurf: Option<sk_sp<dyn SkSurface::SkSurface<'static>>> = None;
    let mut srcImage: Arc<dyn SkImage>;
    let mut srcX = srcRect.fLeft;
    let mut srcY = srcRect.fTop;
    let mut constraint = SrcRectConstraint::kStrict_SrcRectConstraint;
    if rescaleGamma == RescaleGamma::kLinear
        && bmp
            .info()
            .colorSpace()
            .is_some_and(|cs| !cs.gammaIsLinear())
    {
        let cs = bmp
            .info()
            .refColorSpace()
            .expect("native nonnull colorspace")
            .makeLinearGamma();
        let info = SkImageInfo::MakeWithColorSpace(
            srcW,
            srcH,
            SkColorType::kRGBA_F16_SkColorType,
            bmp.info().alphaType(),
            cs,
        );
        let Some(linearSurf) = SkSurface_Raster::Raster(&info, 0, None) else {
            callback(context, None);
            return Ok(());
        };
        let owner = linearSurf.as_ptr().expect("native allocated surface");
        let canvas = SkSurface::getCanvas_raw(owner).expect("native raster canvas");
        {
            let image = bmp.asImage();
            SkCanvas::drawImage_raw(
                canvas,
                image.as_ref(),
                -srcX as f32,
                -srcY as f32,
                &sampling,
                Some(&paint),
            )?;
        }
        tempSurf = Some(linearSurf);
        srcImage = SkSurface::makeImageSnapshot_raw(tempSurf.as_ref().unwrap().as_ptr().unwrap())?
            .expect("native raster snapshot");
        srcX = 0;
        srcY = 0;
        constraint = SrcRectConstraint::kFast_SrcRectConstraint;
    } else {
        srcImage = super::SkImage_RasterFactories::RasterFromPixmap(
            bmp.pixmap(),
            None,
            core::ptr::null_mut(),
        )
        .expect("native valid source pixmap");
    }
    while stepsX != 0 || stepsY != 0 {
        let mut nextW = resultInfo.width();
        let mut nextH = resultInfo.height();
        if stepsX < 0 {
            nextW = resultInfo.width() << (-stepsX - 1);
            stepsX += 1;
        } else if stepsX != 0 {
            if stepsX > 1 {
                nextW = srcW * 2;
            }
            stepsX -= 1;
        }
        if stepsY < 0 {
            nextH = resultInfo.height() << (-stepsY - 1);
            stepsY += 1;
        } else if stepsY != 0 {
            if stepsY > 1 {
                nextH = srcH * 2;
            }
            stepsY -= 1;
        }
        let mut info = srcImage.imageInfo().makeWH(nextW, nextH);
        if stepsX == 0 && stepsY == 0 {
            info = resultInfo.clone();
        }
        let Some(next) = SkSurface_Raster::Raster(&info, 0, None) else {
            callback(context, None);
            return Ok(());
        };
        let canvas = SkSurface::getCanvas_raw(next.as_ptr().expect("native allocated surface"))
            .expect("native raster canvas");
        SkCanvas::drawImageRect_raw(
            canvas,
            &srcImage,
            Some(&SkRect::Make(&SkIRect::make_xywh(srcX, srcY, srcW, srcH))),
            &SkRect::MakeIWH(nextW, nextH),
            &sampling,
            Some(&paint),
            constraint,
        )?;
        // C++ move-assignment installs the new Surface before releasing the old owner.
        drop(core::mem::replace(&mut tempSurf, Some(next)));
        let snapshot =
            SkSurface::makeImageSnapshot_raw(tempSurf.as_ref().unwrap().as_ptr().unwrap())?
                .expect("native raster snapshot");
        drop(core::mem::replace(&mut srcImage, snapshot));
        srcX = 0;
        srcY = 0;
        srcW = nextW;
        srcH = nextH;
        constraint = SrcRectConstraint::kFast_SrcRectConstraint;
    }
    let rowBytes = resultInfo.minRowBytes();
    // Native new char[] is uninitialized storage; no zero fill or fabricated pixels.
    let mut data =
        Box::<[u8]>::new_uninit_slice((resultInfo.height() as usize).wrapping_mul(rowBytes));
    let pm = SkPixmap::new(resultInfo, data.as_mut_ptr().cast(), rowBytes);
    if srcImage.readPixels_default(&pm, srcX, srcY)? {
        // Source: src/image/SkRescaleAndReadPixels.cpp:150-161
        struct ReadResult {
            fData: Box<[MaybeUninit<u8>]>,
            fRowBytes: usize,
            // Real inherited empty base: no extra counter or allocation.
            base: super::SkImage::AsyncReadResultBase,
        }
        impl AsyncReadResult for ReadResult {
            fn async_base(&self) -> &super::SkImage::AsyncReadResultBase {
                &self.base
            }
            fn count(&self) -> i32 {
                1
            }
            fn data(&self, _i: i32) -> *const core::ffi::c_void {
                self.fData.as_ptr().cast()
            }
            fn rowBytes(&self, _i: i32) -> usize {
                self.fRowBytes
            }
        }
        let result: core::pin::Pin<Box<dyn AsyncReadResult>> = Box::pin(ReadResult {
            fData: data,
            fRowBytes: rowBytes,
            base: super::SkImage::AsyncReadResultBase::new(),
        });
        callback(context, Some(result));
    } else {
        callback(context, None);
    }
    Ok(())
}
