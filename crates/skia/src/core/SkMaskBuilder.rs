//! A8 mask builder with bounds/row-byte addressing corresponding to SkMask.h.
//! Logical-device storage and PNG/path convenience APIs are local Rust adapters,
//! not a complete translation of the C++ SkMaskBuilder API.

// Dense storage and path methods migrated from tiny-skia-0.12.0/src/mask.rs.
// Bounded A8 storage follows SkMask.h's bounds-relative getAddr8 / fRowBytes
// addressing. Logical device dimensions and lazy dense compatibility are local
// Rust adapters, not a complete translation of the C++ SkMaskBuilder API.

// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#[cfg(all(not(feature = "std"), feature = "no-std-float"))]
use crate::path::NoStdFloat;

use alloc::vec;
use alloc::vec::Vec;
// This crate currently uses std even with its `std` feature disabled (lib.rs
// has no no_std attribute). OnceLock keeps immutable dense fallback Send + Sync.
use std::sync::OnceLock;

use crate::path::{IntRect, IntSize, Path, Scalar, Transform};

use crate::raster::geom::IntSizeExt;
use crate::raster::painter::DrawTiler;
use crate::raster::pipeline::RasterPipelineBlitter;
use crate::raster::pixmap::SubPixmapMut;
use crate::raster::scan;
use crate::raster::{FillRule, PixmapRef};

/// A mask type.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MaskType {
    /// Transfers only the Alpha channel from `Pixmap` to `SkMaskBuilder`.
    Alpha,
    /// Transfers RGB channels as luminance from `Pixmap` to `SkMaskBuilder`.
    ///
    /// Formula: `Y = 0.2126 * R + 0.7152 * G + 0.0722 * B`
    Luminance,
}

/// A mask.
///
/// During drawing over `Pixmap`, mask's black (0) "pixels" would block rendering
/// and white (255) will allow it.
/// Anything in between is used for gradual masking and anti-aliasing.
///
/// This subset implements A8 coverage. Public byte/path APIs retain a dense
/// logical-device view; internal raster clips may own bounds-relative rows.
#[derive(Clone)]
pub struct SkMaskBuilder {
    data: Vec<u8>,
    size: IntSize,
    bounded: Option<BoundedStorage>,
}

#[derive(Clone)]
struct BoundedStorage {
    bounds: IntRect,
    // Read-only legacy APIs may require full device indexing. Mutating packed
    // rows invalidates this cache; mutable legacy APIs promote it to storage.
    dense: OnceLock<Vec<u8>>,
}

fn image_len(width: u32, height: u32) -> Option<usize> {
    let len = usize::try_from(width)
        .ok()?
        .checked_mul(usize::try_from(height).ok()?)?;
    (len <= isize::MAX as usize).then_some(len)
}

impl PartialEq for SkMaskBuilder {
    fn eq(&self, other: &Self) -> bool {
        if self.size != other.size {
            return false;
        }
        if self.bounded.as_ref().map(|b| b.bounds) == other.bounded.as_ref().map(|b| b.bounds) {
            return self.data == other.data;
        }
        // Compare logical coverage, independent of packed bounds or whether a
        // read-only dense cache has been materialized. Avoid allocating caches.
        (0..self.height())
            .all(|y| (0..self.width()).all(|x| self.alpha_at(x, y) == other.alpha_at(x, y)))
    }
}

impl SkMaskBuilder {
    /// Creates a new mask, allocating a buffer of the given size.
    pub fn new(width: u32, height: u32) -> Option<Self> {
        let size = IntSize::from_wh(width, height)?;
        Some(SkMaskBuilder {
            data: vec![0; image_len(width, height)?],
            size,
            bounded: None,
        })
    }

    /// Creates a new mask from a `PixmapRef`.
    pub fn from_pixmap(pixmap: PixmapRef, mask_type: MaskType) -> Self {
        let data_len = pixmap.width() as usize * pixmap.height() as usize;
        let mut mask = SkMaskBuilder {
            data: vec![0; data_len],
            size: pixmap.size(),
            bounded: None,
        };

        // TODO: optimize
        match mask_type {
            MaskType::Alpha => {
                for (p, a) in pixmap.pixels().iter().zip(mask.data.as_mut_slice()) {
                    *a = p.alpha();
                }
            }
            MaskType::Luminance => {
                for (p, ma) in pixmap.pixels().iter().zip(mask.data.as_mut_slice()) {
                    // Normalize.
                    let mut r = f32::from(p.red()) / 255.0;
                    let mut g = f32::from(p.green()) / 255.0;
                    let mut b = f32::from(p.blue()) / 255.0;
                    let a = f32::from(p.alpha()) / 255.0;

                    // Demultiply.
                    if p.alpha() != 0 {
                        r /= a;
                        g /= a;
                        b /= a;
                    }

                    let luma = r * 0.2126 + g * 0.7152 + b * 0.0722;
                    *ma = ((luma * a) * 255.0).clamp(0.0, 255.0).ceil() as u8;
                }
            }
        }

        mask
    }

    /// Creates a new mask by taking ownership over a mask buffer.
    ///
    /// The size needs to match the data provided.
    pub fn from_vec(data: Vec<u8>, size: IntSize) -> Option<Self> {
        let data_len = image_len(size.width(), size.height())?;
        if data.len() != data_len {
            return None;
        }

        Some(SkMaskBuilder {
            data,
            size,
            bounded: None,
        })
    }

    /// Returns mask's width.
    #[inline]
    pub fn width(&self) -> u32 {
        self.size.width()
    }

    /// Returns mask's height.
    #[inline]
    pub fn height(&self) -> u32 {
        self.size.height()
    }

    /// Returns mask's size.
    #[allow(dead_code)]
    pub(crate) fn size(&self) -> IntSize {
        self.size
    }

    /// Creates packed A8 storage for nonempty bounds inside the logical device.
    ///
    /// Addressing uses (x - left) + (y - top) * bounds.width(), corresponding
    /// to SkMask::getAddr8 with a tightly packed fRowBytes. Coverage elsewhere
    /// in the device is implicitly zero until a mutable dense API is used.
    pub(crate) fn new_bounded(width: u32, height: u32, bounds: IntRect) -> Option<Self> {
        let size = IntSize::from_wh(width, height)?;
        // Also validate the device length: legacy APIs must be able to expose it.
        image_len(width, height)?;
        let device = IntRect::from_xywh(0, 0, width, height)?;
        if !device.contains(&bounds) {
            return None;
        }
        Some(Self {
            data: vec![0; image_len(bounds.width(), bounds.height())?],
            size,
            bounded: Some(BoundedStorage {
                bounds,
                dense: OnceLock::new(),
            }),
        })
    }

    /// Physical primary storage bounds; read-only dense caches do not change it.
    pub(crate) fn storage_bounds(&self) -> IntRect {
        self.bounded
            .as_ref()
            .map_or_else(|| self.size.to_int_rect(0, 0), |b| b.bounds)
    }

    /// Bytes owned by primary storage and any lazily materialized dense cache.
    pub(crate) fn allocated_bytes(&self) -> usize {
        self.data.capacity()
            + self
                .bounded
                .as_ref()
                .and_then(|b| b.dense.get())
                .map_or(0, Vec::capacity)
    }

    fn packed_range(&self, y: u32, x_start: u32, x_end: u32) -> Option<core::ops::Range<usize>> {
        let b = self.bounded.as_ref()?.bounds;
        if y < b.top() as u32
            || y >= b.bottom() as u32
            || x_start < b.left() as u32
            || x_end > b.right() as u32
        {
            return None;
        }
        let start = (y - b.top() as u32) as usize * b.width() as usize
            + (x_start - b.left() as u32) as usize;
        Some(start..start + (x_end - x_start) as usize)
    }

    fn check_row_range(&self, y: u32, x_start: u32, x_end: u32) {
        assert!(y < self.height(), "mask row outside device");
        assert!(
            x_start <= x_end && x_end <= self.width(),
            "mask range outside device"
        );
    }

    /// Borrows packed coverage if possible, otherwise zero-extends the device.
    pub(crate) fn row_range(&self, y: u32, x_start: u32, x_end: u32) -> &[u8] {
        self.check_row_range(y, x_start, x_end);
        // Empty ranges never require materializing a dense fallback.
        if x_start == x_end {
            return &self.data[..0];
        }
        if let Some(range) = self.packed_range(y, x_start, x_end) {
            return &self.data[range];
        }
        let row = y as usize * self.width() as usize;
        &self.data()[row + x_start as usize..row + x_end as usize]
    }

    /// Copies a logical device row to bounded caller scratch, padding only the
    /// requested interval with zero. This keeps legacy row consumers from
    /// materializing a whole device when their interval crosses mask bounds.
    pub(crate) fn copy_row_range(&self, y: u32, x_start: u32, destination: &mut [u8]) {
        let x_end = x_start
            .checked_add(u32::try_from(destination.len()).expect("mask row length"))
            .expect("mask row extent");
        self.check_row_range(y, x_start, x_end);
        destination.fill(0);
        let b = self.storage_bounds();
        if y < b.top() as u32 || y >= b.bottom() as u32 {
            return;
        }
        let left = x_start.max(b.left() as u32);
        let right = x_end.min(b.right() as u32);
        if left < right {
            destination[(left - x_start) as usize..(right - x_start) as usize]
                .copy_from_slice(self.row_range(y, left, right));
        }
    }

    /// Borrows a packed row, or promotes to dense storage for an outside write.
    pub(crate) fn row_range_mut(&mut self, y: u32, x_start: u32, x_end: u32) -> &mut [u8] {
        self.check_row_range(y, x_start, x_end);
        if x_start == x_end {
            return &mut self.data[..0];
        }
        if let Some(range) = self.packed_range(y, x_start, x_end) {
            self.bounded.as_mut().unwrap().dense.take();
            return &mut self.data[range];
        }
        let row = y as usize * self.width() as usize;
        &mut self.data_mut()[row + x_start as usize..row + x_end as usize]
    }

    /// Reads alpha in device coordinates; outside primary bounds is zero.
    #[inline]
    pub(crate) fn alpha_at(&self, x: u32, y: u32) -> u8 {
        if x >= self.width() || y >= self.height() {
            return 0;
        }
        if self.bounded.is_some() {
            return self
                .packed_range(y, x, x + 1)
                .map_or(0, |r| self.data[r.start]);
        }
        self.data[y as usize * self.width() as usize + x as usize]
    }

    fn make_dense(&self) -> Vec<u8> {
        let bounds = self.bounded.as_ref().unwrap().bounds;
        let mut dense = vec![0; image_len(self.width(), self.height()).unwrap()];
        for (row, source) in self.data.chunks_exact(bounds.width() as usize).enumerate() {
            let start =
                (row + bounds.top() as usize) * self.width() as usize + bounds.left() as usize;
            dense[start..start + source.len()].copy_from_slice(source);
        }
        dense
    }

    fn ensure_dense(&mut self) {
        if self.bounded.is_some() {
            let dense = self
                .bounded
                .as_mut()
                .unwrap()
                .dense
                .take()
                .unwrap_or_else(|| self.make_dense());
            self.data = dense;
            self.bounded = None;
        }
    }

    /// Returns full device data, materializing zero margins on first access.
    pub fn data(&self) -> &[u8] {
        match &self.bounded {
            Some(b) => b.dense.get_or_init(|| self.make_dense()).as_slice(),
            None => self.data.as_slice(),
        }
    }

    /// Returns mutable full device data, promoting bounded storage if needed.
    pub fn data_mut(&mut self) -> &mut [u8] {
        self.ensure_dense();
        self.data.as_mut_slice()
    }

    /// Consumes the mask and returns full device data, including zero margins.
    pub fn take(mut self) -> Vec<u8> {
        self.ensure_dense();
        self.data
    }

    pub(crate) fn as_submask(&self) -> SubMaskRef<'_> {
        SubMaskRef {
            size: self.size,
            real_width: self.size.width(),
            data: self.data(),
        }
    }

    pub(crate) fn submask(&self, rect: IntRect) -> Option<SubMaskRef<'_>> {
        let rect = self.size.to_int_rect(0, 0).intersect(&rect)?;
        let row_bytes = self.width() as usize;
        let offset = rect.top() as usize * row_bytes + rect.left() as usize;

        Some(SubMaskRef {
            size: rect.size(),
            real_width: self.size.width(),
            data: &self.data()[offset..],
        })
    }

    pub(crate) fn as_subpixmap(&mut self) -> SubPixmapMut<'_> {
        self.ensure_dense();
        SubPixmapMut {
            format: crate::raster::PixelFormat::Rgba8888,
            size: self.size,
            real_width: self.size.width() as usize,
            data: &mut self.data,
        }
    }

    pub(crate) fn subpixmap(&mut self, rect: IntRect) -> Option<SubPixmapMut<'_>> {
        let rect = self.size.to_int_rect(0, 0).intersect(&rect)?;
        self.ensure_dense();
        let row_bytes = self.width() as usize;
        let offset = rect.top() as usize * row_bytes + rect.left() as usize;

        Some(SubPixmapMut {
            format: crate::raster::PixelFormat::Rgba8888,
            size: rect.size(),
            real_width: self.size.width() as usize,
            data: &mut self.data[offset..],
        })
    }

    /// Loads a PNG file into a `SkMaskBuilder`.
    ///
    /// Only grayscale images are supported.
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

        if info.color_type != png::ColorType::Grayscale {
            return Err(make_custom_png_error("only grayscale masks are supported"));
        }

        let size = IntSize::from_wh(info.width, info.height)
            .ok_or_else(|| make_custom_png_error("invalid image size"))?;

        SkMaskBuilder::from_vec(img_data, size)
            .ok_or_else(|| make_custom_png_error("failed to create a mask"))
    }

    /// Loads a PNG file into a `SkMaskBuilder`.
    ///
    /// Only grayscale images are supported.
    #[cfg(feature = "png-format")]
    pub fn load_png<P: AsRef<std::path::Path>>(path: P) -> Result<Self, png::DecodingError> {
        // `png::Decoder` is generic over input, which means that it will instance
        // two copies: one for `&[]` and one for `File`. Which will simply bloat the code.
        // Therefore we're using only one type for input.
        let data = std::fs::read(path)?;
        Self::decode_png(&data)
    }

    /// Encodes mask into a PNG data.
    #[cfg(feature = "png-format")]
    pub fn encode_png(&self) -> Result<Vec<u8>, png::EncodingError> {
        let mut data = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut data, self.width(), self.height());
            encoder.set_color(png::ColorType::Grayscale);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header()?;
            writer.write_image_data(self.data())?;
        }

        Ok(data)
    }

    /// Saves mask as a PNG file.
    #[cfg(feature = "png-format")]
    pub fn save_png<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), png::EncodingError> {
        let data = self.encode_png()?;
        std::fs::write(path, data)?;
        Ok(())
    }

    // Almost a direct copy of PixmapMut::fill_path
    /// Draws a filled path onto the mask.
    ///
    /// In terms of RGB (no alpha) image, draws a white path on top of black mask.
    ///
    /// Doesn't reset the existing mask content and draws the path on top of existing data.
    ///
    /// If the above behavior is undesired, [`SkMaskBuilder::clear()`] should be called first.
    ///
    /// This method is intended to be used for simple cases. For more complex masks
    /// prefer [`SkMaskBuilder::from_pixmap()`].
    pub fn fill_path(
        &mut self,
        path: &Path,
        fill_rule: FillRule,
        anti_alias: bool,
        transform: Transform,
    ) {
        if transform.is_identity() {
            // This is sort of similar to SkDraw::drawPath

            // Skip empty paths and horizontal/vertical lines.
            let path_bounds = path.bounds();
            if path_bounds.width().is_nearly_zero() || path_bounds.height().is_nearly_zero() {
                log::warn!("empty paths and horizontal/vertical lines cannot be filled");
                return;
            }

            if crate::raster::painter::is_too_big_for_math(path) {
                log::warn!("path coordinates are too big");
                return;
            }

            // TODO: ignore paths outside the pixmap

            if let Some(tiler) = DrawTiler::new(self.width(), self.height()) {
                let mut path = path.clone(); // TODO: avoid cloning

                for tile in tiler {
                    let ts = Transform::from_translate(-(tile.x() as f32), -(tile.y() as f32));
                    path = match path.transform(ts) {
                        Some(v) => v,
                        None => {
                            log::warn!("path transformation failed");
                            return;
                        }
                    };

                    let clip_rect = tile.size().to_screen_int_rect(0, 0);
                    let mut subpix = match self.subpixmap(tile.to_int_rect()) {
                        Some(v) => v,
                        None => continue, // technically unreachable
                    };

                    let mut blitter = match RasterPipelineBlitter::new_mask(&mut subpix) {
                        Some(v) => v,
                        None => continue, // nothing to do, all good
                    };

                    // We're ignoring "errors" here, because `fill_path` will return `None`
                    // when rendering a tile that doesn't have a path on it.
                    // Which is not an error in this case.
                    if anti_alias {
                        scan::path_aa::fill_path(&path, fill_rule, &clip_rect, &mut blitter);
                    } else {
                        scan::path::fill_path(&path, fill_rule, &clip_rect, &mut blitter);
                    }

                    let ts = Transform::from_translate(tile.x() as f32, tile.y() as f32);
                    path = match path.transform(ts) {
                        Some(v) => v,
                        None => return, // technically unreachable
                    };
                }
            } else {
                let clip_rect = self.size().to_screen_int_rect(0, 0);
                let mut subpix = self.as_subpixmap();
                let mut blitter = match RasterPipelineBlitter::new_mask(&mut subpix) {
                    Some(v) => v,
                    None => return, // nothing to do, all good
                };

                if anti_alias {
                    scan::path_aa::fill_path(path, fill_rule, &clip_rect, &mut blitter);
                } else {
                    scan::path::fill_path(path, fill_rule, &clip_rect, &mut blitter);
                }
            }
        } else {
            let path = match path.clone().transform(transform) {
                Some(v) => v,
                None => {
                    log::warn!("path transformation failed");
                    return;
                }
            };

            self.fill_path(&path, fill_rule, anti_alias, Transform::identity());
        }
    }

    /// Intersects the provided path with the current clipping path.
    ///
    /// A temporary mask with the same size as the current one will be created.
    pub fn intersect_path(
        &mut self,
        path: &Path,
        fill_rule: FillRule,
        anti_alias: bool,
        transform: Transform,
    ) {
        let mut submask = SkMaskBuilder::new(self.width(), self.height()).unwrap();
        submask.fill_path(path, fill_rule, anti_alias, transform);

        for (a, b) in self.data_mut().iter_mut().zip(submask.data.iter()) {
            *a = crate::raster::color::premultiply_u8(*a, *b);
        }
    }

    /// Inverts the mask.
    pub fn invert(&mut self) {
        self.data_mut().iter_mut().for_each(|a| *a = 255 - *a);
    }

    /// Clears the mask.
    ///
    /// Zero-fills the internal data buffer.
    pub fn clear(&mut self) {
        if let Some(b) = &mut self.bounded {
            b.dense.take();
        }
        self.data.fill(0);
    }
}

impl core::fmt::Debug for SkMaskBuilder {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SkMaskBuilder")
            .field("data", &"...")
            .field("width", &self.size.width())
            .field("height", &self.size.height())
            .finish()
    }
}

#[derive(Clone, Copy)]
pub struct SubMaskRef<'a> {
    pub data: &'a [u8],
    pub size: IntSize,
    pub real_width: u32,
}

impl<'a> SubMaskRef<'a> {
    pub(crate) fn mask_ctx(&self) -> crate::raster::pipeline::MaskCtx<'a> {
        crate::raster::pipeline::MaskCtx {
            data: self.data,
            real_width: self.real_width,
        }
    }
}

// Compatibility name for the former tiny-skia API.
pub use SkMaskBuilder as Mask;

#[cfg(test)]
mod bounded_storage_tests {
    use super::*;

    fn bounds() -> IntRect {
        IntRect::from_xywh(2, 1, 3, 2).unwrap()
    }

    fn bounded() -> Mask {
        let mut mask = Mask::new_bounded(7, 5, bounds()).unwrap();
        mask.row_range_mut(1, 2, 5).copy_from_slice(&[10, 20, 30]);
        mask.row_range_mut(2, 2, 5).copy_from_slice(&[40, 50, 60]);
        mask
    }

    fn dense() -> Mask {
        let mut mask = Mask::new(7, 5).unwrap();
        mask.data_mut()[9..12].copy_from_slice(&[10, 20, 30]);
        mask.data_mut()[16..19].copy_from_slice(&[40, 50, 60]);
        mask
    }

    #[test]
    fn bounded_rows_use_bounds_origin_and_stride() {
        let mut mask = bounded();
        assert_eq!((mask.width(), mask.height()), (7, 5));
        assert_eq!(mask.storage_bounds(), bounds());
        assert_eq!(mask.allocated_bytes(), 6);
        assert_eq!(mask.row_range(1, 3, 5), &[20, 30]);
        assert_eq!(mask.row_range(2, 2, 4), &[40, 50]);
        assert_eq!(mask.row_range(4, 7, 7), &[]);
        assert_eq!(mask.row_range_mut(0, 0, 0), &[]);
        assert_eq!(mask.allocated_bytes(), 6);
        for y in 0..6 {
            for x in 0..8 {
                let expected = if y < 5 && x < 7 {
                    dense().data()[y * 7 + x]
                } else {
                    0
                };
                assert_eq!(mask.alpha_at(x as u32, y as u32), expected);
            }
        }
        assert_eq!(mask.alpha_at(u32::MAX, u32::MAX), 0);
        assert_eq!(mask.allocated_bytes(), 6);
    }

    #[test]
    fn bounded_dense_read_materializes_zero_margins_and_reuses_cache() {
        let mask = bounded();
        assert_eq!(mask.row_range(1, 0, 7), &[0, 0, 10, 20, 30, 0, 0]);
        assert_eq!(mask.row_range(0, 0, 7), &[0; 7]);
        assert_eq!(mask.data(), dense().data());
        assert_eq!(mask.data().as_ptr(), mask.data().as_ptr());
        assert_eq!(mask.storage_bounds(), bounds());
        assert_eq!(mask.allocated_bytes(), 6 + 35);
        assert_eq!(mask.take(), dense().take());
    }

    #[test]
    fn bounded_mutation_invalidates_dense_read_cache() {
        let mut mask = bounded();
        assert_eq!(mask.data()[10], 20);
        mask.row_range_mut(1, 3, 4)[0] = 90;
        assert_eq!(mask.allocated_bytes(), 6);
        assert_eq!(mask.alpha_at(3, 1), 90);
        assert_eq!(mask.data()[10], 90);
        mask.clear();
        assert_eq!(mask.allocated_bytes(), 6);
        assert_eq!(mask.storage_bounds(), bounds());
        assert!(mask.data().iter().all(|a| *a == 0));
    }

    #[test]
    fn bounded_mutable_fallback_preserves_coverage_and_allows_margin_writes() {
        let mut mask = bounded();
        let expected = dense().take();
        let _ = mask.data();
        mask.row_range_mut(0, 0, 2).copy_from_slice(&[70, 80]);
        assert_eq!(
            mask.storage_bounds(),
            IntRect::from_xywh(0, 0, 7, 5).unwrap()
        );
        assert_eq!(mask.allocated_bytes(), 35);
        assert_eq!(mask.alpha_at(0, 0), 70);
        assert_eq!(&mask.data()[2..], &expected[2..]);
        let mut mask = bounded();
        mask.data_mut()[34] = 255;
        assert_eq!(mask.alpha_at(6, 4), 255);
        assert_eq!(&mask.data()[..34], &expected[..34]);
    }

    #[test]
    fn bounded_clone_equality_are_logical_and_independent() {
        let mask = bounded();
        assert_eq!(mask, dense());
        assert_eq!(mask.allocated_bytes(), 6);
        let mut clone = mask.clone();
        clone.row_range_mut(1, 2, 3)[0] = 11;
        assert_ne!(mask, clone);
        assert_eq!(mask.alpha_at(2, 1), 10);
        let _ = mask.data();
        assert_eq!(mask, dense());
        assert_eq!(mask.clone().take(), dense().take());
        // Different packed extents still compare equal when all coverage is zero.
        let a = Mask::new_bounded(7, 5, bounds()).unwrap();
        let b = Mask::new_bounded(7, 5, IntRect::from_xywh(0, 0, 1, 1).unwrap()).unwrap();
        assert_eq!(a, b);
        assert_ne!(a, Mask::new(5, 7).unwrap());
        assert_eq!(a.allocated_bytes(), 6);
        assert_eq!(b.allocated_bytes(), 1);
    }

    #[test]
    fn bounded_submask_and_path_apis_keep_device_coordinates() {
        let mask = bounded();
        let submask = mask
            .submask(IntRect::from_xywh(1, 1, 5, 2).unwrap())
            .unwrap();
        assert_eq!(submask.real_width, 7);
        assert_eq!(submask.size.dimensions(), (5, 2));
        assert_eq!(&submask.data[..5], &[0, 10, 20, 30, 0]);
        assert_eq!(&submask.data[7..12], &[0, 40, 50, 60, 0]);
        assert_eq!(mask.as_submask().data, dense().data());
        assert!(mask
            .submask(IntRect::from_xywh(7, 0, 1, 1).unwrap())
            .is_none());
        let path = crate::path::PathBuilder::from_rect(
            crate::path::Rect::from_xywh(0.0, 0.0, 4.0, 3.0).unwrap(),
        );
        for anti_alias in [false, true] {
            let mut bounded = bounded();
            let mut dense = dense();
            bounded.fill_path(&path, FillRule::Winding, anti_alias, Transform::identity());
            dense.fill_path(&path, FillRule::Winding, anti_alias, Transform::identity());
            assert_eq!(bounded, dense);
            assert_eq!(bounded.alpha_at(0, 0), 255);
            bounded.intersect_path(&path, FillRule::Winding, anti_alias, Transform::identity());
            dense.intersect_path(&path, FillRule::Winding, anti_alias, Transform::identity());
            assert_eq!(bounded, dense);
        }
        let mut mask = bounded();
        let mut expected = dense();
        mask.invert();
        expected.invert();
        assert_eq!(mask, expected);
        assert_eq!(mask.alpha_at(0, 0), 255);
    }

    #[test]
    fn bounded_constructor_rejects_invalid_device_or_storage_bounds() {
        assert!(Mask::new_bounded(0, 5, bounds()).is_none());
        assert!(Mask::new_bounded(7, 0, bounds()).is_none());
        assert!(Mask::new_bounded(u32::MAX, u32::MAX, bounds()).is_none());
        assert!(Mask::new_bounded(7, 5, IntRect::from_xywh(-1, 0, 2, 1).unwrap()).is_none());
        assert!(Mask::new_bounded(7, 5, IntRect::from_xywh(0, -1, 1, 2).unwrap()).is_none());
        assert!(Mask::new_bounded(7, 5, IntRect::from_xywh(6, 0, 2, 1).unwrap()).is_none());
        assert!(Mask::new_bounded(7, 5, IntRect::from_xywh(0, 4, 1, 2).unwrap()).is_none());
        assert_eq!(Mask::new(7, 5).unwrap().allocated_bytes(), 35);
        assert_eq!(
            Mask::from_vec(vec![0; 35], IntSize::from_wh(7, 5).unwrap())
                .unwrap()
                .allocated_bytes(),
            35
        );
    }

    #[test]
    fn bounded_row_ranges_guard_logical_device_limits() {
        for (y, start, end) in [(5, 0, 1), (0, 0, 8), (0, 4, 3), (0, 8, 8), (u32::MAX, 0, 0)] {
            let mask = bounded();
            assert!(std::panic::catch_unwind(|| mask.row_range(y, start, end)).is_err());
            assert_eq!(mask.allocated_bytes(), 6);
            let mut mask = bounded();
            assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _ = mask.row_range_mut(y, start, end);
            }))
            .is_err());
            assert_eq!(mask.allocated_bytes(), 6);
        }
    }

    #[test]
    fn bounded_mask_remains_send_sync_with_thread_safe_dense_cache() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Mask>();
        let mask = std::sync::Arc::new(bounded());
        let threads: Vec<_> = (0..4)
            .map(|_| {
                let mask = mask.clone();
                std::thread::spawn(move || (mask.data()[10], mask.data().as_ptr() as usize))
            })
            .collect();
        let results: Vec<_> = threads.into_iter().map(|t| t.join().unwrap()).collect();
        assert!(results.iter().all(|r| r.0 == 20 && r.1 == results[0].1));
        assert_eq!(mask.allocated_bytes(), 41);
    }
}

// Strict pristine producer cache adapters: never request the dense view.
impl SkMaskBuilder {
    pub(crate) fn pristine_clip_product_allocation(&self) -> Option<(usize, usize)> {
        if self
            .bounded
            .as_ref()
            .is_some_and(|b| b.dense.get().is_some())
        {
            return None;
        }
        Some((self.data.capacity(), self.data.len()))
    }
    pub(crate) fn try_clone_pristine_clip_product(&self) -> Option<Self> {
        self.pristine_clip_product_allocation()?;
        let mut data = Vec::new();
        data.try_reserve_exact(self.data.len()).ok()?;
        data.extend_from_slice(&self.data);
        Some(Self {
            data,
            size: self.size,
            bounded: self.bounded.as_ref().map(|b| BoundedStorage {
                bounds: b.bounds,
                dense: OnceLock::new(),
            }),
        })
    }
}
