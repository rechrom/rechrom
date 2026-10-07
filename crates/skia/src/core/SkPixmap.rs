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
use crate::src::core::SkBitmap::SkBitmap as Pixmap;

use crate::raster::color::PremultipliedColorU8;
use crate::raster::geom::{IntSizeExt, ScreenIntRect};

#[cfg(feature = "png-format")]
use crate::raster::color::{premultiply_u8, ALPHA_U8_OPAQUE};

/// Number of bytes per pixel.
pub const BYTES_PER_PIXEL: usize = 4;

/// A container that references premultiplied RGBA pixels.
///
/// Can be created from `Pixmap` or from a user provided data.
///
/// The data is not aligned, therefore width == stride.
#[derive(Clone, Copy, PartialEq)]
pub struct SkPixmap<'a> {
    pub(crate) data: &'a [u8],
    pub(crate) size: IntSize,
    pub(crate) format: crate::raster::PixelFormat,
}

impl<'a> SkPixmap<'a> {
    /// Creates a new `SkPixmap` from bytes.
    ///
    /// The size must be at least `size.width() * size.height() * BYTES_PER_PIXEL`.
    /// Zero size in an error. Width is limited by i32::MAX/4.
    ///
    /// The `data` is assumed to have premultiplied RGBA pixels (byteorder: RGBA).
    pub fn from_bytes(data: &'a [u8], width: u32, height: u32) -> Option<Self> {
        let size = IntSize::from_wh(width, height)?;
        let data_len = data_len_for_size(size)?;
        if data.len() < data_len {
            return None;
        }

        Some(SkPixmap {
            data,
            size,
            format: crate::raster::PixelFormat::Rgba8888,
        })
    }

    /// Creates a new `Pixmap` from the current data.
    ///
    /// Clones the underlying data.
    pub fn to_owned(&self) -> Pixmap {
        Pixmap {
            data: crate::raster::PixelStorage::owned(self.data.to_vec()),
            size: self.size,
            format: self.format,
        }
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
    pub(crate) fn size(&self) -> IntSize {
        self.size
    }

    /// Returns pixmap's rect.
    pub(crate) fn rect(&self) -> ScreenIntRect {
        self.size.to_screen_int_rect(0, 0)
    }

    /// Returns the internal data.
    ///
    /// Byteorder: RGBA
    pub fn data(&self) -> &'a [u8] {
        self.data
    }

    /// True when every pixel in this view has alpha 255.
    ///
    /// Translates the RGBA8888 branch of upstream SkPixmap.cpp::computeIsOpaque:
    /// AND packed pixel alpha bits across each row and reject a nonopaque row.
    /// Extra bytes allowed by from_bytes() are outside the view and ignored.
    #[allow(non_snake_case)]
    pub fn computeIsOpaque(&self) -> bool {
        if self.format == crate::raster::PixelFormat::Bgrx8888 {
            return true;
        }
        #[cfg(all(feature = "simd", target_arch = "aarch64"))]
        #[inline(always)]
        unsafe fn row_is_opaque(row: &[u8]) -> bool {
            // Equivalent packed AND reduction for official computeIsOpaque.
            // Each packed field loads eight initialized bytes without requiring
            // alignment; unrolling avoids DEBUG calls per NEON instruction.

            #[repr(C, packed)]
            struct Block {
                w0: u64,
                w1: u64,
                w2: u64,
                w3: u64,
                w4: u64,
                w5: u64,
                w6: u64,
                w7: u64,
                w8: u64,
                w9: u64,
                w10: u64,
                w11: u64,
                w12: u64,
                w13: u64,
                w14: u64,
                w15: u64,
                w16: u64,
                w17: u64,
                w18: u64,
                w19: u64,
                w20: u64,
                w21: u64,
                w22: u64,
                w23: u64,
                w24: u64,
                w25: u64,
                w26: u64,
                w27: u64,
                w28: u64,
                w29: u64,
                w30: u64,
                w31: u64,
            }
            let base = row.as_ptr();
            let mut p = base;
            let end = base.add(row.len() / 256 * 256);
            let mut a = u64::MAX;
            let mut b = a;
            let mut c = a;
            let mut d = a;
            while p < end {
                let block = p as *const Block;
                a &= (*block).w0
                    & (*block).w1
                    & (*block).w2
                    & (*block).w3
                    & (*block).w4
                    & (*block).w5
                    & (*block).w6
                    & (*block).w7;
                b &= (*block).w8
                    & (*block).w9
                    & (*block).w10
                    & (*block).w11
                    & (*block).w12
                    & (*block).w13
                    & (*block).w14
                    & (*block).w15;
                c &= (*block).w16
                    & (*block).w17
                    & (*block).w18
                    & (*block).w19
                    & (*block).w20
                    & (*block).w21
                    & (*block).w22
                    & (*block).w23;
                d &= (*block).w24
                    & (*block).w25
                    & (*block).w26
                    & (*block).w27
                    & (*block).w28
                    & (*block).w29
                    & (*block).w30
                    & (*block).w31;
                p = p.add(256);
            }
            let mut alpha = (a & b & c & d) & 0xff000000ff000000;
            for rgba in row[row.len() / 256 * 256..].chunks_exact(4) {
                alpha &= (rgba[3] as u64) << 24 | (rgba[3] as u64) << 56;
            }
            alpha == 0xff000000ff000000
        }

        #[cfg(not(all(feature = "simd", target_arch = "aarch64")))]
        #[inline(always)]
        fn row_is_opaque(row: &[u8]) -> bool {
            let mut alpha = 255u8;
            for pixel in row.chunks_exact(4) {
                alpha &= pixel[3];
            }
            alpha == 255
        }
        let row_bytes = self.width() as usize * BYTES_PER_PIXEL;
        let view_bytes = row_bytes * self.height() as usize;
        for row in self.data[..view_bytes].chunks_exact(row_bytes) {
            #[cfg(all(feature = "simd", target_arch = "aarch64"))]
            let opaque = unsafe { row_is_opaque(row) };
            #[cfg(not(all(feature = "simd", target_arch = "aarch64")))]
            let opaque = row_is_opaque(row);
            if !opaque {
                return false;
            }
        }
        true
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

    /// Returns a slice of pixels.
    pub fn pixels(&self) -> &'a [PremultipliedColorU8] {
        bytemuck::cast_slice(self.data())
    }

    // TODO: add rows() iterator

    /// Returns a copy of the pixmap that intersects the `rect`.
    ///
    /// Returns `None` when `Pixmap`'s rect doesn't contain `rect`.
    pub fn clone_rect(&self, rect: IntRect) -> Option<Pixmap> {
        // TODO: to ScreenIntRect?

        let rect = self.rect().to_int_rect().intersect(&rect)?;
        let mut new = Pixmap::new(rect.width(), rect.height())?;
        let src_stride = self.width() as usize * BYTES_PER_PIXEL;
        let dst_stride = rect.width() as usize * BYTES_PER_PIXEL;
        let first = rect.y() as usize * src_stride + rect.x() as usize * BYTES_PER_PIXEL;
        // SkPixmap::readPixels -> SkConvertPixels::rect_memcpy copies rows
        // directly when format/alpha/color space agree. This owned subset
        // adapter keeps canonical RGBA output and converts other formats.
        for (y, dst) in new.data_mut().chunks_exact_mut(dst_stride).enumerate() {
            let offset = first + y * src_stride;
            let src = &self.data[offset..offset + dst_stride];
            if self.format == crate::raster::PixelFormat::Rgba8888 {
                dst.copy_from_slice(src);
            } else {
                for (dst, src) in dst.chunks_exact_mut(4).zip(src.chunks_exact(4)) {
                    dst.copy_from_slice(&self.format.decode(src.try_into().unwrap()));
                }
            }
        }

        Some(new)
    }

    /// Encodes pixmap into a PNG data.
    #[cfg(feature = "png-format")]
    pub fn encode_png(&self) -> Result<Vec<u8>, png::EncodingError> {
        // Skia uses skcms here, which is somewhat similar to RasterPipeline.

        // Sadly, we have to copy the pixmap here, because of demultiplication.
        // Not sure how to avoid this.
        // TODO: remove allocation
        let demultiplied_data = self.to_owned().take_demultiplied();

        let mut data = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut data, self.width(), self.height());
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header()?;
            writer.write_image_data(&demultiplied_data)?;
        }

        Ok(data)
    }

    /// Saves pixmap as a PNG file.
    #[cfg(feature = "png-format")]
    pub fn save_png<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), png::EncodingError> {
        let data = self.encode_png()?;
        std::fs::write(path, data)?;
        Ok(())
    }
}

impl core::fmt::Debug for SkPixmap<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SkPixmap")
            .field("data", &"...")
            .field("width", &self.size.width())
            .field("height", &self.size.height())
            .finish()
    }
}

/// A container that references mutable premultiplied RGBA pixels.
///
/// Can be created from `Pixmap` or from a user provided data.
///
/// The data is not aligned, therefore width == stride.
#[derive(PartialEq)]
pub struct SkPixmapMut<'a> {
    pub(crate) data: &'a mut [u8],
    pub(crate) size: IntSize,
    pub(crate) format: crate::raster::PixelFormat,
}

impl<'a> SkPixmapMut<'a> {
    /// Creates a new `SkPixmapMut` from bytes.
    ///
    /// The size must be at least `size.width() * size.height() * BYTES_PER_PIXEL`.
    /// Zero size in an error. Width is limited by i32::MAX/4.
    ///
    /// The `data` is assumed to have premultiplied RGBA pixels (byteorder: RGBA).
    pub fn from_bytes(data: &'a mut [u8], width: u32, height: u32) -> Option<Self> {
        let size = IntSize::from_wh(width, height)?;
        let data_len = data_len_for_size(size)?;
        if data.len() < data_len {
            return None;
        }

        Some(SkPixmapMut {
            data,
            size,
            format: crate::raster::PixelFormat::Rgba8888,
        })
    }

    /// Creates a new `Pixmap` from the current data.
    ///
    /// Clones the underlying data.
    pub fn to_owned(&self) -> Pixmap {
        Pixmap {
            data: crate::raster::PixelStorage::owned(self.data.to_vec()),
            size: self.size,
            format: self.format,
        }
    }

    /// Returns a container that references Pixmap's data.
    pub fn as_ref(&self) -> SkPixmap<'_> {
        SkPixmap {
            data: self.data,
            size: self.size,
            format: self.format,
        }
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
    pub(crate) fn size(&self) -> IntSize {
        self.size
    }

    /// Fills the entire pixmap with a specified color.
    pub fn fill(&mut self, color: Color) {
        let c = color.premultiply().to_color_u8();
        let c = self
            .format
            .encode([c.red(), c.green(), c.blue(), c.alpha()]);
        let c = PremultipliedColorU8::from_rgba_unchecked(c[0], c[1], c[2], c[3]);
        for p in self.pixels_mut() {
            *p = c;
        }
    }

    /// Returns the mutable internal data.
    ///
    /// Byteorder: RGBA
    pub fn data_mut(&mut self) -> &mut [u8] {
        self.data
    }

    /// Returns a mutable slice of pixels.
    pub fn pixels_mut(&mut self) -> &mut [PremultipliedColorU8] {
        bytemuck::cast_slice_mut(self.data_mut())
    }

    /// Creates `SubPixmapMut` that contains the whole `SkPixmapMut`.
    pub(crate) fn as_subpixmap(&mut self) -> SubPixmapMut<'_> {
        SubPixmapMut {
            size: self.size(),
            real_width: self.width() as usize,
            format: self.format,
            data: self.data,
        }
    }

    /// Returns a mutable reference to the pixmap region that intersects the `rect`.
    ///
    /// Returns `None` when `Pixmap`'s rect doesn't contain `rect`.
    pub(crate) fn subpixmap(&mut self, rect: IntRect) -> Option<SubPixmapMut<'_>> {
        let rect = self.size.to_int_rect(0, 0).intersect(&rect)?;
        let row_bytes = self.width() as usize * BYTES_PER_PIXEL;
        let offset = rect.top() as usize * row_bytes + rect.left() as usize * BYTES_PER_PIXEL;

        Some(SubPixmapMut {
            size: rect.size(),
            real_width: self.width() as usize,
            format: self.format,
            data: &mut self.data[offset..],
        })
    }
}

impl core::fmt::Debug for SkPixmapMut<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SkPixmapMut")
            .field("data", &"...")
            .field("width", &self.size.width())
            .field("height", &self.size.height())
            .finish()
    }
}

/// A `SkPixmapMut` subregion.
///
/// Unlike `SkPixmapMut`, contains `real_width` which references the parent `SkPixmapMut` width.
/// This way we can operate on a `SkPixmapMut` subregion without reallocations.
/// Primarily required because of `DrawTiler`.
///
/// We cannot implement it in `SkPixmapMut` directly, because it will brake `fill`, `data_mut`
/// `pixels_mut` and other similar methods.
/// This is because `SubPixmapMut.data` references more "data" than it actually allowed to access.
/// On the other hand, `SkPixmapMut.data` can access all it's data and it's stored linearly.
pub struct SubPixmapMut<'a> {
    pub data: &'a mut [u8],
    pub size: IntSize,
    pub real_width: usize,
    pub(crate) format: crate::raster::PixelFormat,
}

impl SubPixmapMut<'_> {
    /// Returns a mutable slice of pixels.
    pub fn pixels_mut(&mut self) -> &mut [PremultipliedColorU8] {
        bytemuck::cast_slice_mut(self.data)
    }
}

/// Returns minimum bytes per row as usize.
///
/// Pixmap's maximum value for row bytes must fit in 31 bits.
fn min_row_bytes(size: IntSize) -> Option<NonZeroUsize> {
    let w = i32::try_from(size.width()).ok()?;
    let w = w.checked_mul(BYTES_PER_PIXEL as i32)?;
    NonZeroUsize::new(w as usize)
}

/// Returns storage size required by pixel array.
fn compute_data_len(size: IntSize, row_bytes: usize) -> Option<usize> {
    let h = size.height().checked_sub(1)?;
    let h = (h as usize).checked_mul(row_bytes)?;

    let w = (size.width() as usize).checked_mul(BYTES_PER_PIXEL)?;

    h.checked_add(w)
}

pub(crate) fn data_len_for_size(size: IntSize) -> Option<usize> {
    let row_bytes = min_row_bytes(size)?;
    compute_data_len(size, row_bytes.get())
}

pub use SkPixmap as PixmapRef;
pub use SkPixmapMut as PixmapMut;

#[cfg(test)]
mod opacity_tests {
    use super::SkPixmap;

    #[test]
    fn subset_rows_preserve_intersection_channels_and_unaligned_views() {
        use crate::raster::{IntRect, PixelFormat};
        for format in [
            PixelFormat::Rgba8888,
            PixelFormat::Bgra8888,
            PixelFormat::Bgrx8888,
        ] {
            for offset in 0..16 {
                let bytes: Vec<u8> = (0..offset + 9 * 7 * 4 + 13)
                    .map(|i| (i * 31) as u8)
                    .collect();
                let mut view = SkPixmap::from_bytes(&bytes[offset..], 9, 7).unwrap();
                view.format = format;
                for (x, y, w, h) in [
                    (0, 0, 9, 7),
                    (2, 1, 5, 4),
                    (-3, -2, 6, 5),
                    (7, 5, 8, 8),
                    (15, 0, 2, 2),
                ] {
                    let requested = IntRect::from_xywh(x, y, w, h).unwrap();
                    let result = view.clone_rect(requested);
                    let intersection = view.rect().to_int_rect().intersect(&requested);
                    assert_eq!(result.is_some(), intersection.is_some());
                    if let (Some(result), Some(rect)) = (result, intersection) {
                        assert_eq!(
                            (result.width(), result.height()),
                            (rect.width(), rect.height())
                        );
                        let expected: Vec<u8> = (rect.y()..rect.bottom())
                            .flat_map(|y| {
                                (rect.x()..rect.right()).flat_map(move |x| {
                                    let i = (y as usize * 9 + x as usize) * 4;
                                    format.decode(view.data()[i..i + 4].try_into().unwrap())
                                })
                            })
                            .collect();
                        assert_eq!(result.data(), expected, "format={format:?} offset={offset}");
                    }
                }
            }
        }
    }
    #[test]
    fn every_alpha_in_every_vector_and_tail_position_is_checked() {
        for width in [
            1usize, 3, 4, 5, 7, 8, 9, 15, 16, 17, 31, 32, 33, 63, 64, 65, 127, 128, 129,
        ] {
            let mut bytes = vec![255u8; width * 2 * 4];
            for alpha in 0..=255u8 {
                for pixel in 0..width * 2 {
                    bytes[pixel * 4 + 3] = alpha;
                    let view = SkPixmap::from_bytes(&bytes, width as u32, 2).unwrap();
                    assert_eq!(
                        view.computeIsOpaque(),
                        alpha == 255,
                        "width={width}, pixel={pixel}, alpha={alpha}"
                    );
                    bytes[pixel * 4 + 3] = 255;
                }
            }
        }
    }

    #[test]
    fn unaligned_views_ignore_rgb_and_bytes_outside_dimensions() {
        for offset in 0..=15 {
            for width in (1..=35u32).chain([63, 64, 65, 127, 128, 129, 255, 256, 257]) {
                let view_bytes = width as usize * 3 * 4;
                let mut storage = vec![0u8; offset + view_bytes + 17];
                for (pixel, rgba) in storage[offset..offset + view_bytes]
                    .chunks_exact_mut(4)
                    .enumerate()
                {
                    rgba.copy_from_slice(&[pixel as u8, 0, 123, 255]);
                }
                let view = SkPixmap::from_bytes(&storage[offset..], width, 3).unwrap();
                assert!(view.computeIsOpaque(), "offset={offset}, width={width}");
                storage[offset + view_bytes - 1] = 254;
                let view = SkPixmap::from_bytes(&storage[offset..], width, 3).unwrap();
                assert!(!view.computeIsOpaque(), "offset={offset}, width={width}");
            }
        }
    }
}
