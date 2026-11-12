use std::{env, fs, io, sync::Arc};

use skia::{
    include::core::{
        SkAlphaType::SkAlphaType,
        SkColorType::SkColorType,
        SkSamplingOptions::{SkFilterMode, SkMipmapMode, SkSamplingOptions},
    },
    raster_target::{premultiply_n32_rgba8888, PixelFormat, RasterTarget},
    src::{
        core::{SkImageInfo::SkImageInfo, SkMatrix::SkMatrix, SkPixmap::SkPixmap, SkRect::SkRect},
        image::{SkImage::SkImage, SkImage_RasterFactories::RasterFromPixmapCopy},
    },
};

fn main() -> io::Result<()> {
    let mut args = env::args().skip(1);
    let input = args.next().expect("input RGBA path");
    let source_width: i32 = args.next().expect("source width").parse().unwrap();
    let source_height: i32 = args.next().expect("source height").parse().unwrap();
    let destination_width: i32 = args.next().expect("destination width").parse().unwrap();
    let destination_height: i32 = args.next().expect("destination height").parse().unwrap();
    let output = args.next().expect("output RGBA path");
    let mode = args.next().expect("sampling mode");
    let draw_width: f32 = args
        .next()
        .map_or(destination_width as f32, |value| value.parse().unwrap());
    let draw_height: f32 = args
        .next()
        .map_or(destination_height as f32, |value| value.parse().unwrap());
    let draw_x: f32 = args.next().map_or(0.0, |value| value.parse().unwrap());
    let draw_y: f32 = args.next().map_or(0.0, |value| value.parse().unwrap());

    let straight = fs::read(input)?;
    let premultiplied =
        premultiply_n32_rgba8888(source_width as u32, source_height as u32, &straight)
            .expect("valid source pixels");
    let source_info = SkImageInfo::Make(
        source_width,
        source_height,
        SkColorType::kRGBA_8888_SkColorType,
        SkAlphaType::kPremul_SkAlphaType,
    );
    let pixmap = SkPixmap::new(
        &source_info,
        premultiplied.as_ptr(),
        source_width as usize * 4,
    );
    if mode == "scale-pixels-medium" {
        let mut pixels = vec![0; destination_width as usize * destination_height as usize * 4];
        let target_info = SkImageInfo::Make(
            destination_width,
            destination_height,
            SkColorType::kRGBA_8888_SkColorType,
            SkAlphaType::kPremul_SkAlphaType,
        );
        let target_pixmap = SkPixmap::new(
            &target_info,
            pixels.as_mut_ptr(),
            destination_width as usize * 4,
        );
        unsafe {
            pixmap.scalePixels(
                &target_pixmap,
                &SkSamplingOptions::with_filter_mipmap(
                    SkFilterMode::kLinear,
                    SkMipmapMode::kNearest,
                ),
            )
        }
        .map_err(io::Error::other)?;
        fs::write(output, pixels)?;
        return Ok(());
    }
    let image: Arc<dyn SkImage> = unsafe { RasterFromPixmapCopy(&pixmap) }
        .ok_or_else(|| io::Error::other("source image creation failed"))?;
    let (image, sampling) = match mode.as_str() {
        "nearest" => (
            image,
            SkSamplingOptions::with_filter(SkFilterMode::kNearest),
        ),
        "linear" => (image, SkSamplingOptions::with_filter(SkFilterMode::kLinear)),
        "mip-nearest" => (
            unsafe { image.withDefaultMipmaps() }.map_err(io::Error::other)?,
            SkSamplingOptions::with_filter_mipmap(SkFilterMode::kLinear, SkMipmapMode::kNearest),
        ),
        "mip-linear" => (
            unsafe { image.withDefaultMipmaps() }.map_err(io::Error::other)?,
            SkSamplingOptions::with_filter_mipmap(SkFilterMode::kLinear, SkMipmapMode::kLinear),
        ),
        _ => panic!("unknown sampling mode"),
    };

    let mut pixels = vec![0; destination_width as usize * destination_height as usize * 4];
    {
        let mut target = RasterTarget::new(
            &mut pixels,
            destination_width,
            destination_height,
            destination_width as usize * 4,
            PixelFormat::Rgba8888,
            None,
        )
        .map_err(|error| io::Error::other(format!("target creation failed: {error:?}")))?;
        let source = SkRect::MakeWH(source_width as f32, source_height as f32);
        let destination = SkRect::MakeXYWH(draw_x, draw_y, draw_width, draw_height);
        target.concat(&SkMatrix::RectToRect(
            &source,
            &destination,
            skia::src::core::SkMatrix::ScaleToFit::kFill_ScaleToFit,
        ));
        unsafe {
            target.draw_image_rect(
                &image,
                None,
                &source,
                &sampling,
                None,
                skia::src::core::SkCanvas::SrcRectConstraint::kFast_SrcRectConstraint,
            )
        }
        .map_err(io::Error::other)?;
        target.finish().map_err(io::Error::other)?;
    }
    fs::write(output, pixels)
}
