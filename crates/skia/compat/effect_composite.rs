//! Shared isolated-effect restore for CPU compositor and Canvas saveLayer.
//!
//! Source pixels are premultiplied RGBA8888. The destination may use any N32
//! storage order. Non-normal modes go through the same migrated Skia raster
//! pipeline as ordinary paint; optional A8 coverage keeps saveLayer's parent
//! clip on the blend result rather than approximating it as source opacity.
use crate::compat::commands::PaintBlendMode;
use crate::raster::{
    FilterQuality, Mask, PixelFormat, PixmapMut, PixmapPaint, PixmapRef, Transform,
};

pub fn raster_blend_mode(mode: PaintBlendMode) -> crate::BlendMode {
    match mode {
        PaintBlendMode::kNormal => crate::BlendMode::kSrcOver,
        PaintBlendMode::kMultiply => crate::BlendMode::kMultiply,
        PaintBlendMode::kScreen => crate::BlendMode::kScreen,
        PaintBlendMode::kOverlay => crate::BlendMode::kOverlay,
        PaintBlendMode::kDarken => crate::BlendMode::kDarken,
        PaintBlendMode::kLighten => crate::BlendMode::kLighten,
        PaintBlendMode::kColorDodge => crate::BlendMode::kColorDodge,
        PaintBlendMode::kColorBurn => crate::BlendMode::kColorBurn,
        PaintBlendMode::kHardLight => crate::BlendMode::kHardLight,
        PaintBlendMode::kSoftLight => crate::BlendMode::kSoftLight,
        PaintBlendMode::kDifference => crate::BlendMode::kDifference,
        PaintBlendMode::kExclusion => crate::BlendMode::kExclusion,
        PaintBlendMode::kHue => crate::BlendMode::kHue,
        PaintBlendMode::kSaturation => crate::BlendMode::kSaturation,
        PaintBlendMode::kColor => crate::BlendMode::kColor,
        PaintBlendMode::kLuminosity => crate::BlendMode::kLuminosity,
        PaintBlendMode::kPlusLighter => crate::BlendMode::kPlus,
    }
}

fn swap_red_blue(word: u32) -> u32 {
    (word & 0xff00_ff00) | ((word & 0x0000_00ff) << 16) | ((word & 0x00ff_0000) >> 16)
}

/// Restore one isolated RGBA source row into an N32 destination row.
/// `coverage`, when present, has one byte per pixel and clips the completed
/// blend. This is deliberately a rare-mode path; normal SrcOver keeps the
/// backend's vectorized row implementation.
pub fn blend_effect_row(
    target: &mut [u32],
    source: &[u32],
    format: PixelFormat,
    mode: PaintBlendMode,
    opacity: f32,
    coverage: Option<&[u8]>,
) {
    assert_eq!(target.len(), source.len());
    assert!(coverage.is_none_or(|coverage| coverage.len() == target.len()));
    if target.is_empty() || opacity <= 0.0 {
        return;
    }
    for (chunk_index, (destination, source)) in
        target.chunks_mut(256).zip(source.chunks(256)).enumerate()
    {
        let mut rgba_destination = [0u32; 256];
        for (output, &input) in rgba_destination.iter_mut().zip(destination.iter()) {
            *output = match format {
                PixelFormat::Rgba8888 => input,
                PixelFormat::Bgra8888 => swap_red_blue(input),
                PixelFormat::Bgrx8888 => swap_red_blue(input) | 0xff00_0000,
            };
        }
        let destination_bytes = unsafe {
            std::slice::from_raw_parts_mut(
                rgba_destination.as_mut_ptr().cast::<u8>(),
                destination.len() * 4,
            )
        };
        let source_bytes =
            unsafe { std::slice::from_raw_parts(source.as_ptr().cast::<u8>(), source.len() * 4) };
        let mut pixmap = PixmapMut::from_bytes(destination_bytes, destination.len() as u32, 1)
            .expect("bounded non-empty effect row");
        let source = PixmapRef::from_bytes(source_bytes, source.len() as u32, 1)
            .expect("bounded non-empty effect source row");
        let mask = coverage.map(|coverage| {
            let start = chunk_index * 256;
            let coverage = &coverage[start..start + destination.len()];
            let mut mask =
                Mask::new(destination.len() as u32, 1).expect("bounded non-empty effect coverage");
            mask.data_mut().copy_from_slice(coverage);
            mask
        });
        pixmap.draw_pixmap(
            0,
            0,
            source,
            &PixmapPaint {
                opacity: opacity.clamp(0.0, 1.0),
                blend_mode: raster_blend_mode(mode),
                quality: FilterQuality::Nearest,
            },
            Transform::identity(),
            mask.as_ref(),
        );
        for (output, &input) in destination.iter_mut().zip(&rgba_destination) {
            *output = match format {
                PixelFormat::Rgba8888 => input,
                PixelFormat::Bgra8888 => swap_red_blue(input),
                PixelFormat::Bgrx8888 => swap_red_blue(input) & 0x00ff_ffff,
            };
        }
    }
}
