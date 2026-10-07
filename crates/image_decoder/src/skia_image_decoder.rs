#![allow(non_snake_case)]

use std::io;

use crate::image_decoder::{DecodedImage, ImageDecodeInput, ImageDecoder};

// cpp: image_decoder/skia_image_decoder.h:9-13
#[derive(Default)]
pub struct SkiaImageDecoder;

// cpp: image_decoder/skia_image_decoder.cc:37-40
fn IsJpeg(input: &ImageDecodeInput<'_>) -> bool {
    input.bytes.len() >= 2 && input.bytes[0] == 0xff && input.bytes[1] == 0xd8
}

// cpp: image_decoder/skia_image_decoder.cc:42-111
// The decoder and its pixel buffers are owned by Rust; no native codec is linked.
fn DecodeJpeg(input: &ImageDecodeInput<'_>) -> io::Result<DecodedImage> {
    let decoded = image::load_from_memory_with_format(input.bytes, image::ImageFormat::Jpeg)
        .map_err(|_| io::Error::other("JPEG image could not be decoded"))?;
    IntoDecodedImage(decoded)
}

fn IntoDecodedImage(decoded: image::DynamicImage) -> io::Result<DecodedImage> {
    let (width, height) = (decoded.width(), decoded.height());
    if width == 0 || height == 0 {
        return Err(io::Error::other("decoded image has invalid dimensions"));
    }
    (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or_else(|| io::Error::other("decoded image dimensions overflow RGBA storage"))?;
    Ok(DecodedImage {
        width,
        height,
        rgba8: decoded.into_rgba8().into_raw(),
    })
}

// cpp: image_decoder/skia_image_decoder.cc:180-229
impl ImageDecoder for SkiaImageDecoder {
    fn Decode(&mut self, input: &ImageDecodeInput<'_>) -> io::Result<DecodedImage> {
        if input.bytes.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "cannot decode an empty image",
            ));
        }
        if IsJpeg(input) {
            return DecodeJpeg(input);
        }
        let decoded = image::load_from_memory(input.bytes).map_err(|error| {
            io::Error::other(format!(
                "image could not be decoded ({}): {error}",
                if input.mime_type.is_empty() {
                    "unknown image format"
                } else {
                    input.mime_type
                }
            ))
        })?;
        IntoDecodedImage(decoded)
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
        assert_eq!(result.rgba8, [20, 40, 60, 128, 1, 2, 3, 255]);
    }

    #[test]
    fn jpeg_integer_transform_fancy_upsampling_and_last_row_match_native() {
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
    fn jpeg_even_width_right_edge_matches_native() {
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
