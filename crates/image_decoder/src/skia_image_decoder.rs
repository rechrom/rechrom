#![allow(non_snake_case)]

use std::borrow::Cow;
use std::io::{self, Cursor};

use image::{AnimationDecoder as _, ImageDecoder as _};

use crate::image_decoder::{DecodedImage, DecodedImageFrame, ImageDecodeInput, ImageDecoder};

// cpp: image_decoder/skia_image_decoder.h:9-13
#[derive(Default)]
pub struct SkiaImageDecoder;

// cpp: image_decoder/skia_image_decoder.cc:37-40
fn IsJpeg(input: &ImageDecodeInput<'_>) -> bool {
    input.bytes.len() >= 2 && input.bytes[0] == 0xff && input.bytes[1] == 0xd8
}

// SkGifDecoder accepts palette-less transparent GIFs used as tracking and
// layout placeholders.  The image crate rejects them before decoding LZW even
// when every referenced pixel is the transparent index.  Supply the two-entry
// black global palette that Skia effectively makes available to the decoder;
// the GIF transparency extension still clears the decoded alpha channel.
//
// Source: third_party/blink/renderer/platform/image-decoders/gif/
// gif_image_decoder.cc:45-55 (SkGifDecoder owns GIF acceptance and pixels).
fn GifBytesWithFallbackColorTable(bytes: &[u8]) -> Cow<'_, [u8]> {
    if bytes.len() < 13 || !matches!(&bytes[..6], b"GIF87a" | b"GIF89a") || bytes[10] & 0x80 != 0 {
        return Cow::Borrowed(bytes);
    }
    let mut normalized = Vec::with_capacity(bytes.len() + 6);
    normalized.extend_from_slice(&bytes[..10]);
    // Set the global-color-table flag and select its minimum two-entry size.
    normalized.push((bytes[10] | 0x80) & !0x07);
    normalized.extend_from_slice(&bytes[11..13]);
    normalized.extend_from_slice(&[0; 6]);
    normalized.extend_from_slice(&bytes[13..]);
    Cow::Owned(normalized)
}

// cpp: image_decoder/skia_image_decoder.cc:42-111
// The decoder and its pixel buffers are owned by this package.
fn DecodeImage(
    input: &ImageDecodeInput<'_>,
    format: Option<image::ImageFormat>,
) -> io::Result<DecodedImage> {
    let mut reader = image::ImageReader::new(Cursor::new(input.bytes));
    if let Some(format) = format {
        reader.set_format(format);
    } else {
        reader = reader.with_guessed_format().map_err(|error| {
            io::Error::other(format!("image format could not be read: {error}"))
        })?;
    }
    // Chromium's GIFImageDecoder publishes kAnimationNone for a one-frame GIF.
    // For a multi-frame GIF, BitmapImage retains a PaintImageGenerator whose
    // GetFrameMetadata/GetPixels methods are consumed by cc's image animation
    // controller. This eager decoder retains the equivalent complete frames
    // and timing facts in the immutable Rust PaintImage producer.
    //
    // Source: third_party/blink/renderer/platform/image-decoders/gif/
    // gif_image_decoder.cc; platform/graphics/bitmap_image.cc:636-680.
    if reader.format() == Some(image::ImageFormat::Gif) {
        let gif_bytes = GifBytesWithFallbackColorTable(input.bytes);
        let decoder = image::codecs::gif::GifDecoder::new(Cursor::new(gif_bytes.as_ref()))
            .map_err(io::Error::other)?;
        let loop_count = decoder.loop_count();
        let frames = decoder
            .into_frames()
            .collect_frames()
            .map_err(io::Error::other)?;
        let first = frames
            .first()
            .ok_or_else(|| io::Error::other("GIF image contains no frames"))?;
        if frames.len() > 1 {
            let width = first.buffer().width();
            let height = first.buffer().height();
            let repetition_count = match loop_count {
                image::metadata::LoopCount::Infinite => -1,
                image::metadata::LoopCount::Finite(count) => {
                    // BitmapImage::RepetitionCount converts the GIF decoder's
                    // repeat count to the total number of animation loops.
                    count.get().saturating_add(1).min(i32::MAX as u32) as i32
                }
            };
            let animation_frames = frames
                .into_iter()
                .map(|frame| {
                    let (numerator, denominator) = frame.delay().numer_denom_ms();
                    let nanos =
                        (u128::from(numerator) * 1_000_000) / u128::from(denominator.max(1));
                    DecodedImageFrame {
                        duration: std::time::Duration::from_nanos(
                            nanos.min(u128::from(u64::MAX)) as u64
                        ),
                        rgba8: frame.into_buffer().into_raw(),
                    }
                })
                .collect::<Vec<_>>();
            return Ok(DecodedImage {
                width,
                height,
                orientation: image_resource::ImageOrientation::kDefault,
                alpha_type: image_resource::PaintImageAlphaType::Unpremultiplied,
                animation: image_resource::PaintImageAnimationMetadata::Known(
                    image_resource::PaintImageAnimationInfo {
                        animation_type: image_resource::PaintImageAnimationType::Animated,
                        repetition_count,
                        frame_count: animation_frames.len(),
                    },
                ),
                rgba8: animation_frames[0].rgba8.clone(),
                animation_frames,
            });
        }
        return IntoDecodedImage(
            image::DynamicImage::ImageRgba8(
                frames
                    .into_iter()
                    .next()
                    .expect("checked non-empty GIF frame set")
                    .into_buffer(),
            ),
            image::metadata::Orientation::NoTransforms,
            image_resource::PaintImageAnimationMetadata::Known(
                image_resource::PaintImageAnimationInfo::confirmed_static(),
            ),
        );
    }
    // Keep the codec's actual animation classification before erasing its
    // concrete type. The current pixel ABI cannot retain animated frame sets.
    let format = reader.format();
    let (mut decoder, animation): (Box<dyn image::ImageDecoder + '_>, _) = match format {
        Some(image::ImageFormat::Png) => {
            let mut decoder = image::codecs::png::PngDecoder::with_limits(
                Cursor::new(input.bytes),
                image::Limits::default(),
            )
            .map_err(io::Error::other)?;
            decoder
                .set_limits(image::Limits::default())
                .map_err(io::Error::other)?;
            let animation = if decoder.is_apng().map_err(io::Error::other)? {
                image_resource::PaintImageAnimationMetadata::Unavailable(
                    image_resource::PaintImageMetadataUnavailable::EncodedAnimationTimeline {
                        format: "APNG",
                    },
                )
            } else {
                image_resource::PaintImageAnimationMetadata::Known(
                    image_resource::PaintImageAnimationInfo::confirmed_static(),
                )
            };
            (Box::new(decoder), animation)
        }
        Some(image::ImageFormat::WebP) => {
            let mut decoder = image::codecs::webp::WebPDecoder::new(Cursor::new(input.bytes))
                .map_err(io::Error::other)?;
            decoder
                .set_limits(image::Limits::default())
                .map_err(io::Error::other)?;
            let animation = if decoder.has_animation() {
                image_resource::PaintImageAnimationMetadata::Unavailable(
                    image_resource::PaintImageMetadataUnavailable::EncodedAnimationTimeline {
                        format: "animated WebP",
                    },
                )
            } else {
                image_resource::PaintImageAnimationMetadata::Known(
                    image_resource::PaintImageAnimationInfo::confirmed_static(),
                )
            };
            (Box::new(decoder), animation)
        }
        format => {
            let animation = match format {
                Some(image::ImageFormat::Gif) => {
                    image_resource::PaintImageAnimationMetadata::Unavailable(
                        image_resource::PaintImageMetadataUnavailable::EncodedAnimationTimeline {
                            format: "GIF",
                        },
                    )
                }
                Some(_) => image_resource::PaintImageAnimationMetadata::Known(
                    image_resource::PaintImageAnimationInfo::confirmed_static(),
                ),
                None => image_resource::PaintImageAnimationMetadata::default(),
            };
            let decoder = reader.into_decoder().map_err(|error| {
                io::Error::other(format!(
                    "image could not be decoded ({}): {error}",
                    if input.mime_type.is_empty() {
                        "unknown image format"
                    } else {
                        input.mime_type
                    }
                ))
            })?;
            (Box::new(decoder), animation)
        }
    };
    let orientation = decoder
        .orientation()
        .unwrap_or(image::metadata::Orientation::NoTransforms);
    let decoded = image::DynamicImage::from_decoder(decoder)
        .map_err(|error| io::Error::other(format!("image pixels could not be decoded: {error}")))?;
    let alpha_type = if format == Some(image::ImageFormat::WebP) {
        image_resource::PaintImageAlphaType::Premultiplied
    } else {
        image_resource::PaintImageAlphaType::Unpremultiplied
    };
    IntoDecodedImageWithAlphaType(decoded, orientation, animation, alpha_type)
}

fn IntoDecodedImage(
    decoded: image::DynamicImage,
    orientation: image::metadata::Orientation,
    animation: image_resource::PaintImageAnimationMetadata,
) -> io::Result<DecodedImage> {
    IntoDecodedImageWithAlphaType(
        decoded,
        orientation,
        animation,
        image_resource::PaintImageAlphaType::Unpremultiplied,
    )
}

fn IntoDecodedImageWithAlphaType(
    decoded: image::DynamicImage,
    orientation: image::metadata::Orientation,
    animation: image_resource::PaintImageAnimationMetadata,
    alpha_type: image_resource::PaintImageAlphaType,
) -> io::Result<DecodedImage> {
    let (width, height) = (decoded.width(), decoded.height());
    if width == 0 || height == 0 {
        return Err(io::Error::other("decoded image has invalid dimensions"));
    }
    (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or_else(|| io::Error::other("decoded image dimensions overflow RGBA storage"))?;
    let mut rgba8 = decoded.into_rgba8().into_raw();
    if alpha_type == image_resource::PaintImageAlphaType::Premultiplied {
        // Source: libwebp's MODE_rgbA/MODE_bgrA output selected by
        // WEBPImageDecoder::RGBOutputMode. WebP's premultiplied modes use
        // truncating component * alpha / 255, not Skia's rounded conversion.
        for pixel in rgba8.chunks_exact_mut(4) {
            let alpha = u16::from(pixel[3]);
            pixel[0] = (u16::from(pixel[0]) * alpha / 255) as u8;
            pixel[1] = (u16::from(pixel[1]) * alpha / 255) as u8;
            pixel[2] = (u16::from(pixel[2]) * alpha / 255) as u8;
        }
    }
    Ok(DecodedImage {
        width,
        height,
        orientation: image_resource::ImageOrientation::from_exif(orientation.to_exif()),
        alpha_type,
        animation,
        animation_frames: Vec::new(),
        rgba8,
    })
}

// cpp: image_decoder/skia_image_decoder.cc:180-229
impl ImageDecoder for SkiaImageDecoder {
    fn BackgroundFactory(
        &self,
    ) -> Option<std::sync::Arc<dyn Fn() -> Box<dyn ImageDecoder> + Send + Sync>> {
        Some(std::sync::Arc::new(|| Box::new(SkiaImageDecoder)))
    }
    fn Decode(&mut self, input: &ImageDecodeInput<'_>) -> io::Result<DecodedImage> {
        if input.bytes.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "cannot decode an empty image",
            ));
        }
        // Blink AVIFImageDecoder::MatchesAVIFSignature uses the pinned parser
        // and at most 144 bytes, independently of server MIME or URL suffix.
        #[cfg(feature = "avif-backend")]
        if crabby_avif::decoder::Decoder::peek_compatible_file_type(
            &input.bytes[..input.bytes.len().min(144)],
        ) {
            return crate::avif_backend::decode(input.bytes);
        }
        if IsJpeg(input) {
            return DecodeImage(input, Some(image::ImageFormat::Jpeg));
        }
        DecodeImage(input, None)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn png_keeps_straight_alpha_and_rgba_channels() {
        let pixels = image::RgbaImage::from_raw(2, 1, vec![20, 40, 60, 128, 1, 2, 3, 255])
            .expect("valid RGBA fixture");
        let mut encoded = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(pixels)
            .write_to(&mut encoded, image::ImageFormat::Png)
            .expect("encode PNG fixture");
        let result = SkiaImageDecoder
            .Decode(&ImageDecodeInput {
                bytes: encoded.get_ref(),
                mime_type: "image/png",
            })
            .expect("decode PNG");
        assert_eq!((result.width, result.height), (2, 1));
        assert_eq!(
            result.alpha_type,
            image_resource::PaintImageAlphaType::Unpremultiplied
        );
        assert_eq!(result.rgba8, [20, 40, 60, 128, 1, 2, 3, 255]);
    }

    #[test]
    fn webp_publishes_libwebp_premultiplied_frame_bytes() {
        let pixels = image::RgbaImage::from_raw(2, 1, vec![13, 40, 24, 19, 200, 100, 50, 255])
            .expect("valid RGBA fixture");
        let mut encoded = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(pixels)
            .write_to(&mut encoded, image::ImageFormat::WebP)
            .expect("encode lossless WebP fixture");
        let result = SkiaImageDecoder
            .Decode(&ImageDecodeInput {
                bytes: encoded.get_ref(),
                mime_type: "image/webp",
            })
            .expect("decode WebP");
        assert_eq!(
            result.alpha_type,
            image_resource::PaintImageAlphaType::Premultiplied
        );
        // WEBPImageDecoder::RGBOutputMode selects MODE_rgbA/MODE_bgrA.
        // libwebp truncates each color component * alpha / 255.
        assert_eq!(result.rgba8, [0, 2, 1, 19, 200, 100, 50, 255]);
    }

    #[test]
    fn gif_retains_complete_frame_timeline() {
        let mut encoded = Vec::new();
        {
            let mut encoder = image::codecs::gif::GifEncoder::new(&mut encoded);
            encoder
                .set_repeat(image::codecs::gif::Repeat::Infinite)
                .unwrap();
            let frames = [
                image::Frame::from_parts(
                    image::RgbaImage::from_raw(1, 1, vec![255, 0, 0, 255]).unwrap(),
                    0,
                    0,
                    image::Delay::from_numer_denom_ms(20, 1),
                ),
                image::Frame::from_parts(
                    image::RgbaImage::from_raw(1, 1, vec![0, 0, 255, 255]).unwrap(),
                    0,
                    0,
                    image::Delay::from_numer_denom_ms(30, 1),
                ),
            ];
            encoder.encode_frames(frames).unwrap();
        }
        let decoded = SkiaImageDecoder
            .Decode(&ImageDecodeInput {
                bytes: &encoded,
                mime_type: "image/gif",
            })
            .unwrap();
        assert_eq!(decoded.animation_frames.len(), 2);
        assert_eq!(decoded.animation_frames[0].duration.as_millis(), 20);
        assert_eq!(decoded.animation_frames[1].duration.as_millis(), 30);
        assert_eq!(decoded.rgba8, decoded.animation_frames[0].rgba8);
        assert_ne!(
            decoded.animation_frames[0].rgba8,
            decoded.animation_frames[1].rgba8
        );
        assert_eq!(
            decoded.animation,
            image_resource::PaintImageAnimationMetadata::Known(
                image_resource::PaintImageAnimationInfo {
                    animation_type: image_resource::PaintImageAnimationType::Animated,
                    repetition_count: -1,
                    frame_count: 2,
                }
            )
        );
    }

    #[test]
    fn palette_less_transparent_gif_matches_skia_acceptance() {
        // A common 1x1 placeholder accepted by Chrome's SkGifDecoder.  It has
        // neither a global nor a local color table; the sole pixel is the
        // transparent index declared by its graphics-control extension.
        let encoded = [
            0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x70, 0x00, 0x00, 0x21,
            0xf9, 0x04, 0x01, 0x00, 0x00, 0x00, 0x00, 0x2c, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00,
            0x01, 0x00, 0x00, 0x02, 0x02, 0x44, 0x01, 0x00, 0x3b,
        ];
        let decoded = SkiaImageDecoder
            .Decode(&ImageDecodeInput {
                bytes: &encoded,
                mime_type: "image/gif",
            })
            .expect("decode palette-less transparent GIF");
        assert_eq!((decoded.width, decoded.height), (1, 1));
        assert_eq!(decoded.rgba8, [0, 0, 0, 0]);
    }

    #[test]
    fn jpeg_integer_transform_fancy_upsampling_and_last_row_match_source() {
        macro_rules! fixture {
            ($name:literal) => {
                (
                    include_bytes!(concat!("../tests/fixtures/jpeg-source/", $name, ".jpg"))
                        .as_slice(),
                    include_bytes!(concat!("../tests/fixtures/jpeg-source/", $name, ".rgba"))
                        .as_slice(),
                    $name,
                )
            };
        }
        for (encoded, expected, name) in [
            fixture!("sampling-0-baseline"),
            fixture!("sampling-0-progressive"),
            fixture!("sampling-1-baseline"),
            fixture!("sampling-1-progressive"),
            fixture!("sampling-2-baseline"),
            fixture!("sampling-2-progressive"),
        ] {
            let decoded = SkiaImageDecoder
                .Decode(&ImageDecodeInput {
                    bytes: encoded,
                    mime_type: "image/jpeg",
                })
                .unwrap();
            assert_eq!((decoded.width, decoded.height), (35, 19), "{name}");
            let differences = decoded
                .rgba8
                .chunks_exact(4)
                .zip(expected.chunks_exact(4))
                .filter(|(a, b)| a != b)
                .count();
            assert_eq!(differences, 0, "{name}: native RGBA differs");
        }
    }

    #[test]
    fn jpeg_even_width_right_edge_matches_source() {
        macro_rules! fixture {
            ($name:literal) => {
                (
                    include_bytes!(concat!("../tests/fixtures/jpeg-source/", $name, ".jpg"))
                        .as_slice(),
                    include_bytes!(concat!("../tests/fixtures/jpeg-source/", $name, ".rgba"))
                        .as_slice(),
                    $name,
                )
            };
        }
        for (encoded, expected, name) in [
            fixture!("right-edge-1-baseline"),
            fixture!("right-edge-1-progressive"),
            fixture!("right-edge-2-baseline"),
            fixture!("right-edge-2-progressive"),
        ] {
            let decoded = SkiaImageDecoder
                .Decode(&ImageDecodeInput {
                    bytes: encoded,
                    mime_type: "image/jpeg",
                })
                .unwrap();
            assert_eq!((decoded.width, decoded.height), (38, 19), "{name}");
            assert_eq!(
                decoded.rgba8.as_slice(),
                expected,
                "{name}: native RGBA differs"
            );
        }
    }

    #[test]
    fn jpeg_produces_opaque_rgba() {
        let pixels = image::RgbImage::from_raw(2, 1, vec![12, 34, 56, 70, 90, 110])
            .expect("valid RGB fixture");
        let mut encoded = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgb8(pixels)
            .write_to(&mut encoded, image::ImageFormat::Jpeg)
            .expect("encode JPEG fixture");
        let result = SkiaImageDecoder
            .Decode(&ImageDecodeInput {
                bytes: encoded.get_ref(),
                mime_type: "image/jpeg",
            })
            .expect("decode JPEG");
        assert_eq!((result.width, result.height), (2, 1));
        assert_eq!(result.rgba8.len(), 8);
        assert!(result.rgba8.chunks_exact(4).all(|pixel| pixel[3] == 255));
    }
}
