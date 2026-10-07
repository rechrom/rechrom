//! RGBA/BGRA bitmap subset; tight stride and exclusive Rust ownership adapt SkBitmap.

// Migrated unchanged in behavior from tiny-skia-0.12.0/src/pixmap.rs.

// Copyright 2006 The Android Open Source Project
// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use alloc::vec;
use alloc::vec::Vec;

use core::convert::TryFrom;
use core::num::NonZeroUsize;

use crate::path::IntSize;

use crate::raster::{Color, IntRect};

use crate::raster::color::PremultipliedColorU8;
use crate::raster::geom::{IntSizeExt, ScreenIntRect};

#[cfg(feature = "png-format")]
use crate::raster::color::{premultiply_u8, ALPHA_U8_OPAQUE};

/// Number of bytes per pixel.
pub const BYTES_PER_PIXEL: usize = 4;

use crate::src::core::SkPixmap::{data_len_for_size, PixmapMut, PixmapRef};
/// Owns pixels with explicit RGBA/BGRA layout or opaque host BGRX encoding.
///
/// The data is not aligned, therefore width == stride.
#[derive(Clone, PartialEq)]
pub struct SkBitmap {
    pub(crate) data: crate::raster::PixelStorage,
    pub(crate) size: IntSize,
    pub(crate) format: crate::raster::PixelFormat,
}

impl SkBitmap {
    /// Allocates a new pixmap.
    ///
    /// A pixmap is filled with transparent black by default, aka (0, 0, 0, 0).
    ///
    /// Zero size in an error.
    ///
    /// SkBitmap's width is limited by i32::MAX/4.
    pub fn new(width: u32, height: u32) -> Option<Self> {
        let size = IntSize::from_wh(width, height)?;
        let data_len = data_len_for_size(size)?;

        // We cannot check that allocation was successful yet.
        // We have to wait for https://github.com/rust-lang/rust/issues/48043

        Some(SkBitmap {
            data: crate::raster::PixelStorage::owned(vec![0; data_len]),
            size,
            format: crate::raster::PixelFormat::Rgba8888,
        })
    }

    /// Creates a new pixmap by taking ownership over an image buffer
    /// (premultiplied RGBA pixels).
    ///
    /// The size needs to match the data provided.
    ///
    /// SkBitmap's width is limited by i32::MAX/4.
    pub fn from_vec(data: Vec<u8>, size: IntSize) -> Option<Self> {
        let data_len = data_len_for_size(size)?;
        if data.len() != data_len {
            return None;
        }

        Some(SkBitmap {
            data: crate::raster::PixelStorage::owned(data),
            size,
            format: crate::raster::PixelFormat::Rgba8888,
        })
    }

    /// Restricted SkBitmap::installPixels: attach an exclusively owned mapping.
    /// This default entry point installs tightly packed premultiplied RGBA8888.
    pub fn install_pixels(
        data: crate::raster::PixelStorage,
        width: u32,
        height: u32,
        row_bytes: usize,
    ) -> Option<Self> {
        Self::install_pixels_with_format(
            data,
            width,
            height,
            row_bytes,
            crate::raster::PixelFormat::Rgba8888,
        )
    }

    /// Color-layout-aware installPixels subset. BGRX has opaque semantic alpha
    /// and a zero unused byte required by the host; RGBA/BGRA are premultiplied.
    pub fn install_pixels_with_format(
        data: crate::raster::PixelStorage,
        width: u32,
        height: u32,
        row_bytes: usize,
        format: crate::raster::PixelFormat,
    ) -> Option<Self> {
        let size = IntSize::from_wh(width, height)?;
        let data_len = data_len_for_size(size)?;
        if row_bytes != (width as usize).checked_mul(BYTES_PER_PIXEL)? || data.len() != data_len {
            return None;
        }
        Some(Self { data, size, format })
    }

    /// Return the exact target ownership, without readback or alpha conversion.
    pub fn take_storage(self) -> crate::raster::PixelStorage {
        self.data
    }

    /// Decodes a PNG data into a `SkBitmap`.
    ///
    /// Only 8-bit images are supported.
    /// Index PNGs are not supported.
    #[cfg(feature = "png-format")]
    pub fn decode_png(data: &[u8]) -> Result<Self, png::DecodingError> {
        fn make_custom_png_error(msg: &str) -> png::DecodingError {
            std::io::Error::new(std::io::ErrorKind::Other, msg).into()
        }

        let mut decoder = png::Decoder::new(std::io::BufReader::new(std::io::Cursor::new(data)));
        decoder.set_transformations(png::Transformations::normalize_to_color8());
        let mut reader = decoder.read_info()?;
        let output_buffer_size = reader
            .output_buffer_size()
            .ok_or(png::DecodingError::LimitsExceeded)?;
        let mut img_data = vec![0; output_buffer_size];
        let info = reader.next_frame(&mut img_data)?;

        if info.bit_depth != png::BitDepth::Eight {
            return Err(make_custom_png_error("unsupported bit depth"));
        }

        let size = IntSize::from_wh(info.width, info.height)
            .ok_or_else(|| make_custom_png_error("invalid image size"))?;
        let data_len =
            data_len_for_size(size).ok_or_else(|| make_custom_png_error("image is too big"))?;

        img_data = match info.color_type {
            png::ColorType::Rgb => {
                let mut rgba_data = Vec::with_capacity(data_len);
                for rgb in img_data.chunks(3) {
                    rgba_data.push(rgb[0]);
                    rgba_data.push(rgb[1]);
                    rgba_data.push(rgb[2]);
                    rgba_data.push(ALPHA_U8_OPAQUE);
                }

                rgba_data
            }
            png::ColorType::Rgba => img_data,
            png::ColorType::Grayscale => {
                let mut rgba_data = Vec::with_capacity(data_len);
                for gray in img_data {
                    rgba_data.push(gray);
                    rgba_data.push(gray);
                    rgba_data.push(gray);
                    rgba_data.push(ALPHA_U8_OPAQUE);
                }

                rgba_data
            }
            png::ColorType::GrayscaleAlpha => {
                let mut rgba_data = Vec::with_capacity(data_len);
                for slice in img_data.chunks(2) {
                    let gray = slice[0];
                    let alpha = slice[1];
                    rgba_data.push(gray);
                    rgba_data.push(gray);
                    rgba_data.push(gray);
                    rgba_data.push(alpha);
                }

                rgba_data
            }
            png::ColorType::Indexed => {
                return Err(make_custom_png_error("indexed PNG is not supported"));
            }
        };

        // Premultiply alpha.
        //
        // We cannon use RasterPipeline here, which is faster,
        // because it produces slightly different results.
        // Seems like Skia does the same.
        //
        // Also, in our tests unsafe version (no bound checking)
        // had roughly the same performance. So we keep the safe one.
        for pixel in img_data.as_mut_slice().chunks_mut(BYTES_PER_PIXEL) {
            let a = pixel[3];
            pixel[0] = premultiply_u8(pixel[0], a);
            pixel[1] = premultiply_u8(pixel[1], a);
            pixel[2] = premultiply_u8(pixel[2], a);
        }

        SkBitmap::from_vec(img_data, size)
            .ok_or_else(|| make_custom_png_error("failed to create a pixmap"))
    }

    /// Loads a PNG file into a `SkBitmap`.
    ///
    /// Only 8-bit images are supported.
    /// Index PNGs are not supported.
    #[cfg(feature = "png-format")]
    pub fn load_png<P: AsRef<std::path::Path>>(path: P) -> Result<Self, png::DecodingError> {
        // `png::Decoder` is generic over input, which means that it will instance
        // two copies: one for `&[]` and one for `File`. Which will simply bloat the code.
        // Therefore we're using only one type for input.
        let data = std::fs::read(path)?;
        Self::decode_png(&data)
    }

    /// Encodes pixmap into a PNG data.
    #[cfg(feature = "png-format")]
    pub fn encode_png(&self) -> Result<Vec<u8>, png::EncodingError> {
        self.as_ref().encode_png()
    }

    /// Saves pixmap as a PNG file.
    #[cfg(feature = "png-format")]
    pub fn save_png<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), png::EncodingError> {
        self.as_ref().save_png(path)
    }

    /// Returns a container that references SkBitmap's data.
    pub fn as_ref(&self) -> PixmapRef<'_> {
        PixmapRef {
            data: &self.data,
            size: self.size,
            format: self.format,
        }
    }

    /// Returns a container that references SkBitmap's data.
    pub fn as_mut(&mut self) -> PixmapMut<'_> {
        PixmapMut {
            data: &mut self.data,
            size: self.size,
            format: self.format,
        }
    }

    pub fn pixel_format(&self) -> crate::raster::PixelFormat {
        self.format
    }

    /// Returns pixmap's width.
    #[inline]
    pub fn width(&self) -> u32 {
        self.size.width()
    }

    /// Returns pixmap's height.
    #[inline]
    pub fn height(&self) -> u32 {
        self.size.height()
    }

    /// Returns pixmap's size.
    #[allow(dead_code)]
    pub(crate) fn size(&self) -> IntSize {
        self.size
    }

    /// Fills the entire pixmap with a specified color.
    pub fn fill(&mut self, color: Color) {
        let c = color.premultiply().to_color_u8();
        // Uniform N32 erase follows SkPixmap::erase / SkOpts::rect_memset32.
        // Retain this adapter's color conversion, then fill contiguous bytes
        // through the owned constant-span implementation rather than walking
        // one Rust pixel at a time in DEBUG builds.
        let c = self
            .format
            .encode([c.red(), c.green(), c.blue(), c.alpha()]);
        crate::src::core::SkBlitRow_D32::fill_solid(self.data_mut(), c);
    }

    /// Returns encoded internal data in the bitmap's pixel_format().
    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    /// Returns mutable encoded data in the bitmap's pixel_format().
    pub fn data_mut(&mut self) -> &mut [u8] {
        self.data.as_mut_slice()
    }

    /// Returns a pixel color.
    ///
    /// Returns `None` when position is out of bounds.
    pub fn pixel(&self, x: u32, y: u32) -> Option<PremultipliedColorU8> {
        let idx = self.width().checked_mul(y)?.checked_add(x)?;
        self.pixels().get(idx as usize).map(|p| {
            let c = self
                .format
                .decode([p.red(), p.green(), p.blue(), p.alpha()]);
            PremultipliedColorU8::from_rgba_unchecked(c[0], c[1], c[2], c[3])
        })
    }

    /// Returns raw encoded pixels; use pixel_format() to interpret channels.
    pub fn pixels_mut(&mut self) -> &mut [PremultipliedColorU8] {
        bytemuck::cast_slice_mut(self.data_mut())
    }

    /// Returns raw encoded pixels; use pixel_format() to interpret channels.
    pub fn pixels(&self) -> &[PremultipliedColorU8] {
        bytemuck::cast_slice(self.data())
    }

    /// Consumes the internal data.
    ///
    /// Byteorder: RGBA
    pub fn take(self) -> Vec<u8> {
        let format = self.format;
        let mut data = self.data.into_vec();
        if format != crate::raster::PixelFormat::Rgba8888 {
            for pixel in data.chunks_exact_mut(4) {
                let decoded = format.decode((&*pixel).try_into().unwrap());
                pixel.copy_from_slice(&decoded);
            }
        }
        data
    }

    /// Consumes the pixmap and returns the internal data as demultiplied RGBA bytes.
    ///
    /// Byteorder: RGBA
    pub fn take_demultiplied(mut self) -> Vec<u8> {
        if self.format != crate::raster::PixelFormat::Rgba8888 {
            let size = self.size;
            return Self::from_vec(self.take(), size)
                .unwrap()
                .take_demultiplied();
        }
        // SkConvertPixels skips alpha conversion for opaque pixels. Detect that
        // case with SkPixmap's alpha reduction before the legacy per-pixel path.
        // This consumes the existing allocation; translucent rounding is unchanged.
        if self.as_ref().computeIsOpaque() {
            return self.data.into_vec();
        }
        // Demultiply alpha.
        //
        // RasterPipeline is 15% faster here, but produces slightly different results
        // due to rounding. So we stick with this method for now.
        for pixel in self.pixels_mut() {
            let c = pixel.demultiply();
            *pixel =
                PremultipliedColorU8::from_rgba_unchecked(c.red(), c.green(), c.blue(), c.alpha());
        }
        self.data.into_vec()
    }

    /// Returns a copy of the pixmap that intersects the `rect`.
    ///
    /// Returns `None` when `SkBitmap`'s rect doesn't contain `rect`.
    pub fn clone_rect(&self, rect: IntRect) -> Option<SkBitmap> {
        self.as_ref().clone_rect(rect)
    }
}

impl core::fmt::Debug for SkBitmap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SkBitmap")
            .field("data", &"...")
            .field("width", &self.size.width())
            .field("height", &self.size.height())
            .finish()
    }
}

pub use SkBitmap as Pixmap;
