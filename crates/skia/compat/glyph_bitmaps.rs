//! Embedded bitmap adapter for SkScalerContext's image (neverRequestPath) branch.
//!
//! SkFontHost_FreeType.cpp selects a strike at/above the requested ppem, falling
//! back to the largest strike, then scales its bearings and pixels. ttf-parser
//! supplies that table selection and normalizes the bearing to a bottom origin.
//! PNG decoding and image resampling stay in the existing Rust CPU backend.
use crate::raster::{IntSize, Pixmap};
use ttf_parser::{RasterGlyphImage, RasterImageFormat};

pub(crate) struct GlyphBitmap {
    pub pixels: Pixmap,
    pub left: f32,
    pub top: f32,
    pub pixels_per_em: u16,
    pub color: bool,
}

impl GlyphBitmap {
    pub fn heap_bytes(&self) -> usize {
        self.pixels.data().len()
    }

    pub fn decode(image: RasterGlyphImage<'_>) -> Option<Self> {
        if image.pixels_per_em == 0 {
            return None;
        }
        let size = IntSize::from_wh(u32::from(image.width), u32::from(image.height))?;
        // A font-supplied strike must not cause an unbounded image allocation.
        if u64::from(size.width()) * u64::from(size.height()) > 4 * 1024 * 1024 {
            return None;
        }
        let color = matches!(
            image.format,
            RasterImageFormat::PNG | RasterImageFormat::BitmapPremulBgra32
        );
        let pixels = match image.format {
            RasterImageFormat::PNG => {
                #[cfg(feature = "png-format")]
                {
                    // Validate dimensions before the decoder allocates its
                    // output, not just against the decoded result afterwards.
                    if image.data.get(..8)? != b"\x89PNG\r\n\x1a\n"
                        || image.data.get(12..16)? != b"IHDR"
                        || u32::from_be_bytes(image.data.get(16..20)?.try_into().ok()?)
                            != size.width()
                        || u32::from_be_bytes(image.data.get(20..24)?.try_into().ok()?)
                            != size.height()
                    {
                        return None;
                    }
                    let pixels = Pixmap::decode_png(image.data).ok()?;
                    // Bitmap metrics and encoded image dimensions must agree.
                    if pixels.width() != size.width() || pixels.height() != size.height() {
                        return None;
                    }
                    pixels
                }
                #[cfg(not(feature = "png-format"))]
                return None;
            }
            RasterImageFormat::BitmapPremulBgra32 => {
                let len = size.width() as usize * size.height() as usize * 4;
                let mut data = image.data.get(..len)?.to_vec();
                for pixel in data.chunks_exact_mut(4) {
                    pixel.swap(0, 2);
                    if pixel[..3].iter().any(|&channel| channel > pixel[3]) {
                        return None;
                    }
                }
                Pixmap::from_vec(data, size)?
            }
            format => {
                let (bits, packed) = match format {
                    RasterImageFormat::BitmapMono => (1, false),
                    RasterImageFormat::BitmapMonoPacked => (1, true),
                    RasterImageFormat::BitmapGray2 => (2, false),
                    RasterImageFormat::BitmapGray2Packed => (2, true),
                    RasterImageFormat::BitmapGray4 => (4, false),
                    RasterImageFormat::BitmapGray4Packed => (4, true),
                    RasterImageFormat::BitmapGray8 => (8, false),
                    _ => unreachable!(),
                };
                let width = size.width() as usize;
                let height = size.height() as usize;
                let row_bits = if packed {
                    width * bits
                } else {
                    (width * bits).div_ceil(8) * 8
                };
                let mut pixels = Pixmap::new(size.width(), size.height())?;
                for y in 0..height {
                    for x in 0..width {
                        let bit = y * row_bits + x * bits;
                        let value = (image.data.get(bit / 8)? >> (8 - bits - bit % 8))
                            & ((1u16 << bits) - 1) as u8;
                        let alpha = (u16::from(value) * 255 / ((1 << bits) - 1)) as u8;
                        pixels.data_mut()[(y * width + x) * 4..(y * width + x + 1) * 4].fill(alpha);
                    }
                }
                pixels
            }
        };
        Some(Self {
            pixels,
            left: f32::from(image.x),
            top: -f32::from(image.y) - f32::from(image.height),
            pixels_per_em: image.pixels_per_em,
            color,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(format: RasterImageFormat, data: &[u8]) -> RasterGlyphImage<'_> {
        RasterGlyphImage {
            x: -1,
            y: -2,
            width: 3,
            height: 2,
            pixels_per_em: 16,
            format,
            data,
        }
    }

    #[test]
    fn bitmap_masks_preserve_row_padding_and_bearings() {
        let padded =
            GlyphBitmap::decode(image(RasterImageFormat::BitmapMono, &[0xa0, 0x40])).unwrap();
        let packed =
            GlyphBitmap::decode(image(RasterImageFormat::BitmapMonoPacked, &[0xa8])).unwrap();
        assert_eq!(padded.pixels.data(), packed.pixels.data());
        assert_eq!(
            padded
                .pixels
                .data()
                .chunks_exact(4)
                .map(|p| p[3])
                .collect::<Vec<_>>(),
            [255, 0, 255, 0, 255, 0]
        );
        assert_eq!((padded.left, padded.top), (-1.0, 0.0));
        assert!(!padded.color);
        assert!(GlyphBitmap::decode(image(RasterImageFormat::BitmapMono, &[0xa0])).is_none());
    }

    #[test]
    fn embedded_bgra_is_premultiplied_and_not_tinted() {
        let mut source = image(RasterImageFormat::BitmapPremulBgra32, &[10, 20, 30, 40]);
        source.width = 1;
        source.height = 1;
        let decoded = GlyphBitmap::decode(source).unwrap();
        assert_eq!(decoded.pixels.data(), [30, 20, 10, 40]);
        assert!(decoded.color);
    }
}

/// Deterministic bitmap-only sfnt using the existing licensed test font's
/// metadata. Glyph 1 has a 2x2 opaque red sbix strike; other glyphs are empty.
#[cfg(all(test, feature = "png-format"))]
pub(crate) fn test_font() -> Vec<u8> {
    let face = ttf_parser::Face::parse(
        include_bytes!("../../../assets/fonts/Roboto-Regular.ttf"),
        0,
    )
    .unwrap();
    let count = u32::from(face.number_of_glyphs());
    let png = crate::compat::png::EncodeRgbaPng(&[255, 0, 0, 255].repeat(4), 2, 2);
    let mut sbix = Vec::new();
    sbix.extend_from_slice(&1u16.to_be_bytes());
    sbix.extend_from_slice(&1u16.to_be_bytes());
    sbix.extend_from_slice(&1u32.to_be_bytes());
    sbix.extend_from_slice(&12u32.to_be_bytes());
    sbix.extend_from_slice(&16u16.to_be_bytes());
    sbix.extend_from_slice(&72u16.to_be_bytes());
    let header = 4 + (count + 1) * 4;
    for glyph in 0..=count {
        let offset = header + if glyph > 1 { 8 + png.len() as u32 } else { 0 };
        sbix.extend_from_slice(&offset.to_be_bytes());
    }
    sbix.extend_from_slice(&0i16.to_be_bytes());
    sbix.extend_from_slice(&0i16.to_be_bytes());
    sbix.extend_from_slice(b"png ");
    sbix.extend_from_slice(&png);
    let raw = face.raw_face();
    let mut tables: Vec<([u8; 4], Vec<u8>)> = raw
        .table_records
        .into_iter()
        .filter(|record| !matches!(&record.tag.to_bytes(), b"glyf" | b"loca" | b"sbix"))
        .map(|record| {
            (
                record.tag.to_bytes(),
                raw.table(record.tag).unwrap().to_vec(),
            )
        })
        .collect();
    tables.push((*b"sbix", sbix));
    tables.sort_by_key(|table| table.0);
    let mut bytes = vec![0; 12 + tables.len() * 16];
    bytes[0..4].copy_from_slice(&0x00010000u32.to_be_bytes());
    bytes[4..6].copy_from_slice(&(tables.len() as u16).to_be_bytes());
    for (i, (tag, data)) in tables.iter().enumerate() {
        while bytes.len() % 4 != 0 {
            bytes.push(0);
        }
        let offset = bytes.len() as u32;
        let record = 12 + i * 16;
        bytes[record..record + 4].copy_from_slice(tag);
        bytes[record + 8..record + 12].copy_from_slice(&offset.to_be_bytes());
        bytes[record + 12..record + 16].copy_from_slice(&(data.len() as u32).to_be_bytes());
        bytes.extend_from_slice(data);
    }
    bytes
}
