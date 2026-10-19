//! Bounded still-image AVIF adapter for the existing owned RGBA8 contract.

use std::{io, num::NonZeroU32};

use crabby_avif::{
    decoder::{Decoder, Settings, Source, Strictness, StrictnessFlag},
    image::Image,
    AvifError, CodecChoice, ColorPrimaries, MatrixCoefficients, PixelFormat,
    TransferCharacteristics,
};
use image_resource::ImageOrientation;

use crate::image_decoder::DecodedImage;

#[derive(Clone, Copy)]
struct Limits {
    input_bytes: usize,
    pixels: NonZeroU32,
    dimension: NonZeroU32,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            input_bytes: 64 * 1024 * 1024,
            // A host resource budget, not a change to AVIF format semantics.
            pixels: NonZeroU32::new(16 * 1024 * 1024).unwrap(),
            dimension: NonZeroU32::new(32768).unwrap(),
        }
    }
}

pub(super) fn invalid(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, format!("AVIF: {error}"))
}
pub(super) fn unsupported(reason: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, format!("AVIF: {reason}"))
}
pub(super) fn codec_error(error: AvifError) -> io::Error {
    if matches!(
        error,
        AvifError::NotImplemented | AvifError::UnsupportedDepth | AvifError::NoCodecAvailable
    ) {
        unsupported(&format!("{error:?}"))
    } else {
        invalid(format!("{error:?}"))
    }
}

fn new_decoder(bytes: &[u8], limits: Limits) -> io::Result<Decoder> {
    if bytes.is_empty() || bytes.len() > limits.input_bytes {
        return Err(invalid("empty input or encoded-byte budget exceeded"));
    }
    // AVIFImageDecoder::UpdateDemuxer, Chromium 6c1d401f, lines 725-782.
    let mut decoder = Decoder::default();
    decoder.settings = Settings {
        source: Source::PrimaryItem,
        max_threads: 2,
        ignore_exif: true,
        ignore_xmp: true,
        allow_progressive: false, // The bytes-to-frame boundary supplies all data.
        strictness: Strictness::SpecificExclude(vec![
            StrictnessFlag::ClapValid,
            StrictnessFlag::PixiRequired,
        ]),
        codec_choice: CodecChoice::Rav1d,
        image_size_limit: Some(limits.pixels),
        image_dimension_limit: Some(limits.dimension),
        image_count_limit: NonZeroU32::new(1),
        ..Settings::default()
    };
    let mut owned = Vec::new();
    owned.try_reserve_exact(bytes.len()).map_err(invalid)?;
    owned.extend_from_slice(bytes);
    decoder.set_io_vec(owned);
    decoder.parse().map_err(codec_error)?;
    Ok(decoder)
}

fn admit_still_image(image: &Image, gainmap: bool) -> io::Result<()> {
    if image.image_sequence_track_present {
        return Err(unsupported(
            "animation/sequence requires the frame timeline contract",
        ));
    }
    if !matches!(
        image.progressive_state,
        crabby_avif::decoder::ProgressiveState::Unavailable
    ) {
        return Err(unsupported(
            "progressive/layered still-image admission is deferred",
        ));
    }
    if image.depth != 8 {
        return Err(unsupported(
            "10/12-bit output and HDR color management are deferred",
        ));
    }
    if image.clap.is_some() {
        return Err(unsupported("clean-aperture policy has not been integrated"));
    }
    if image
        .pasp
        .is_some_and(|p| p.h_spacing == 0 || p.h_spacing != p.v_spacing)
    {
        return Err(unsupported("non-square pixel aspect ratio is deferred"));
    }
    if gainmap || image.clli.is_some() {
        return Err(unsupported("gain maps and HDR metadata are deferred"));
    }
    if !image.icc.is_empty()
        || !matches!(
            image.color_primaries,
            ColorPrimaries::Srgb | ColorPrimaries::Unspecified
        )
        || !matches!(
            image.transfer_characteristics,
            TransferCharacteristics::Srgb
                | TransferCharacteristics::Unspecified
                | TransferCharacteristics::Bt709
                | TransferCharacteristics::Bt601
                | TransferCharacteristics::Bt2020_10bit
                | TransferCharacteristics::Bt2020_12bit
        )
    {
        return Err(unsupported(
            "ICC/CICP color transformation is not integrated",
        ));
    }
    if !matches!(
        image.matrix_coefficients,
        MatrixCoefficients::Identity
            | MatrixCoefficients::Bt709
            | MatrixCoefficients::Unspecified
            | MatrixCoefficients::Bt470bg
            | MatrixCoefficients::Bt601
    ) || !matches!(
        image.yuv_format,
        PixelFormat::Yuv400 | PixelFormat::Yuv420 | PixelFormat::Yuv422 | PixelFormat::Yuv444
    ) {
        return Err(unsupported(
            "color matrix or chroma format outside the candidate slice",
        ));
    }
    if image.matrix_coefficients == MatrixCoefficients::Identity
        && !(image.yuv_format == PixelFormat::Yuv400
            || (image.yuv_format == PixelFormat::Yuv444
                && image.yuv_range == crabby_avif::image::YuvRange::Full))
    {
        return Err(unsupported(
            "identity matrix requires full-range 444 or monochrome",
        ));
    }
    if image.alpha_premultiplied {
        return Err(unsupported(
            "encoded premultiplied-alpha reconstruction is deferred",
        ));
    }
    Ok(())
}

fn orientation(angle: Option<u8>, axis: Option<u8>) -> io::Result<ImageOrientation> {
    // Exact kAxisAngleToOrientation table in Chromium's AVIFImageDecoder, 903-938.
    // Pixels stay in encoded storage order; image_resource retains orientation.
    const EXIF: [[u8; 4]; 3] = [[1, 8, 3, 6], [4, 5, 2, 7], [2, 7, 4, 5]];
    let angle = angle.unwrap_or(0) as usize;
    let axis = axis.map_or(0, |x| x as usize + 1);
    EXIF.get(axis)
        .and_then(|row| row.get(angle))
        .copied()
        .map(ImageOrientation::from_exif)
        .ok_or_else(|| invalid("invalid irot/imir"))
}

pub(crate) fn decode(bytes: &[u8]) -> io::Result<DecodedImage> {
    decode_with_limits(bytes, Limits::default())
}

fn decode_with_limits(bytes: &[u8], limits: Limits) -> io::Result<DecodedImage> {
    let mut decoder = new_decoder(bytes, limits)?;
    let image = decoder
        .image()
        .ok_or_else(|| invalid("missing container image"))?;
    admit_still_image(image, decoder.gainmap_present())?;
    let expected = (image.width, image.height, image.depth, image.yuv_format);
    let expected_alpha = image.alpha_present;
    let orientation = orientation(image.irot_angle, image.imir_axis)?;
    decoder.next_image().map_err(codec_error)?;
    let image = decoder
        .image()
        .ok_or_else(|| invalid("missing decoded image"))?;
    if (image.width, image.height, image.depth, image.yuv_format) != expected {
        return Err(invalid(
            "container and AV1 dimensions/depth/chroma format disagree",
        ));
    }
    if image.has_alpha() != expected_alpha {
        return Err(invalid("container and decoded alpha presence disagree"));
    }
    admit_still_image(image, decoder.gainmap_present())?;
    // All conversion completes while the decoder owns its referenced pictures.
    let rgba8 = super::avif_rgb8::convert(image)?;
    Ok(DecodedImage {
        width: image.width,
        height: image.height,
        orientation,
        // admit_still_image above rejects sequence tracks before publication.
        animation: image_resource::PaintImageAnimationMetadata::Known(
            image_resource::PaintImageAnimationInfo::confirmed_static(),
        ),
        rgba8,
    })
}

#[cfg(test)]
#[path = "avif_backend_tests.rs"]
mod tests;
