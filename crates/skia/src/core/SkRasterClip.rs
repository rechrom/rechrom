// Copyright 2010 Google Inc.
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Metadata subset of upstream SkRasterClip.h / SkRasterClip.cpp.
//!
//! The dense u8 mask remains owned by the caller. `from_mask` is a local Rust
//! adapter, not a line-by-line translation: it derives the exact nonzero bounds
//! and hard-edged rectangle classification in one scan. This type does not
//! implement SkRegion, SkAAClip storage, clip operations, or AA compression.
//! Empty bounds use Option because the existing Rust IntRect cannot be empty.
use crate::raster::IntRect;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SkRasterClip {
    bounds: Option<IntRect>,
    is_rect: bool,
}

// Local dense coverage driver: the rectangle interior and zero margins are
// normally uniform. Skip initialized eight-byte equal-alpha runs; tails and
// nonuniform AA boundary bytes retain the exact scalar metadata decisions.
fn alpha_run_end(row: &[u8], start: usize, value: u8) -> usize {
    crate::compat::analytic_masks::equal_byte_run_end(row, start, value)
}

#[allow(non_snake_case)]
impl SkRasterClip {
    /// Computes metadata for width*height densely packed u8 coverage bytes.
    /// Empty/malformed dimensions, unrepresentable bounds, or a short slice
    /// return None. Bytes after the specified mask dimensions are ignored.
    pub fn from_mask(width: u32, height: u32, data: &[u8]) -> Option<Self> {
        if width == 0 || height == 0 || width > i32::MAX as u32 || height > i32::MAX as u32 {
            return None;
        }
        Self::from_mask_with_bounds(width, height, data, IntRect::from_xywh(0, 0, width, height))
    }

    /// Local dense-mask adapter for SkRasterClip's bounded intersection.
    /// `support` must contain every nonzero byte; bytes outside it are known
    /// zero and are intentionally not read. The returned bounds and rectangle
    /// classification are exact, including holes and partial coverage.
    pub(crate) fn from_mask_with_bounds(
        width: u32,
        height: u32,
        data: &[u8],
        support: Option<IntRect>,
    ) -> Option<Self> {
        let device = IntRect::from_xywh(0, 0, width, height)?;
        let stride = usize::try_from(width).ok()?;
        let length = stride.checked_mul(height as usize)?;
        let data = data.get(..length)?;
        Self::from_rows(device, support, |y, left, right| {
            &data[y as usize * stride + left as usize..y as usize * stride + right as usize]
        })
    }

    /// Local bounds/rowBytes adapter. Packed masks retain their storage;
    /// metadata scanning never requests the legacy full-device byte slice.
    pub(crate) fn from_mask_builder(
        mask: &crate::raster::Mask,
        support: Option<IntRect>,
    ) -> Option<Self> {
        let device = IntRect::from_xywh(0, 0, mask.width(), mask.height())?;
        let support = support.and_then(|b| b.intersect(&mask.storage_bounds()));
        Self::from_rows(device, support, |y, left, right| {
            mask.row_range(y, left, right)
        })
    }

    fn from_rows<'a>(
        device: IntRect,
        support: Option<IntRect>,
        mut row_at: impl FnMut(u32, u32, u32) -> &'a [u8],
    ) -> Option<Self> {
        let Some(support) = support.and_then(|bounds| bounds.intersect(&device)) else {
            return Some(Self::default());
        };
        let mut left = support.right() as usize;
        let mut top = support.bottom() as usize;
        let mut right = 0usize;
        let mut bottom = 0usize;
        let mut nonzero_count = 0usize;
        let mut all_nonzero_are_opaque = true;
        for y in support.top() as usize..support.bottom() as usize {
            let row = row_at(y as u32, support.left() as u32, support.right() as u32);
            let mut row_left = support.right() as usize;
            let mut row_right = 0usize;
            let mut offset = 0;
            while offset < row.len() {
                let coverage = row[offset];
                let end = alpha_run_end(row, offset, coverage);
                if coverage != 0 {
                    let x = support.left() as usize + offset;
                    row_left = row_left.min(x);
                    row_right = support.left() as usize + end;
                    nonzero_count += end - offset;
                    all_nonzero_are_opaque &= coverage == 255;
                }
                offset = end;
            }
            if row_right != 0 {
                left = left.min(row_left);
                right = right.max(row_right);
                top = top.min(y);
                bottom = y + 1;
            }
        }
        if nonzero_count == 0 {
            return Some(Self::default());
        }
        let bounds = IntRect::from_xywh(
            left as i32,
            top as i32,
            (right - left) as u32,
            (bottom - top) as u32,
        )?;
        // A finite grid set fills its bounding rectangle exactly iff its
        // cardinality equals the rectangle's area. Together with opaque alpha
        // this catches holes, missing rows, and shifted scanline intervals.
        let rectangle_area = (right - left).checked_mul(bottom - top)?;
        Some(Self {
            bounds: Some(bounds),
            is_rect: all_nonzero_are_opaque && nonzero_count == rectangle_area,
        })
    }

    /// Exact nonzero coverage bounds, or None for the empty clip.
    pub fn getBounds(&self) -> Option<IntRect> {
        self.bounds
    }

    /// True only for a nonempty, completely opaque rectangular clip.
    pub fn isRect(&self) -> bool {
        self.is_rect
    }

    /// Replaces metadata with a hard-edged rectangle, or an empty clip.
    /// Mirrors the nonempty boolean returned by upstream setRect.
    pub fn setRect(&mut self, bounds: Option<IntRect>) -> bool {
        self.bounds = bounds;
        self.is_rect = bounds.is_some();
        self.is_rect
    }

    /// Guaranteed rejection by bounds. False does not prove actual coverage
    /// intersects the rectangle, exactly as upstream quickReject documents.
    pub fn quickReject(&self, bounds: IntRect) -> bool {
        self.bounds.map_or(true, |clip| {
            clip.left() >= bounds.right()
                || bounds.left() >= clip.right()
                || clip.top() >= bounds.bottom()
                || bounds.top() >= clip.bottom()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: i32, y: i32, width: u32, height: u32) -> IntRect {
        IntRect::from_xywh(x, y, width, height).unwrap()
    }

    #[test]
    fn empty_and_full_masks_have_exact_metadata() {
        let empty = SkRasterClip::from_mask(4, 3, &[0; 12]).unwrap();
        assert_eq!(empty, SkRasterClip::default());
        assert_eq!(empty.getBounds(), None);
        assert!(!empty.isRect());
        assert!(empty.quickReject(rect(0, 0, 1, 1)));
        let full = SkRasterClip::from_mask(4, 3, &[255; 12]).unwrap();
        assert_eq!(full.getBounds(), Some(rect(0, 0, 4, 3)));
        assert!(full.isRect());
    }

    #[test]
    fn inset_opaque_rectangle_and_single_pixel_are_recognized() {
        let mask = [
            0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 0, 0, 0, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0,
        ];
        let clip = SkRasterClip::from_mask(6, 4, &mask).unwrap();
        assert_eq!(clip.getBounds(), Some(rect(2, 1, 3, 2)));
        assert!(clip.isRect());
        let clip = SkRasterClip::from_mask(3, 2, &[0, 0, 0, 0, 0, 255]).unwrap();
        assert_eq!(clip.getBounds(), Some(rect(2, 1, 1, 1)));
        assert!(clip.isRect());
    }

    #[test]
    fn holes_partial_alpha_missing_rows_and_shifted_rows_are_not_rectangles() {
        let cases: &[(u32, u32, &[u8], IntRect)] = &[
            (
                3,
                3,
                &[255, 255, 255, 255, 0, 255, 255, 255, 255],
                rect(0, 0, 3, 3),
            ),
            (3, 2, &[255, 1, 255, 255, 254, 255], rect(0, 0, 3, 2)),
            (
                3,
                3,
                &[255, 255, 255, 0, 0, 0, 255, 255, 255],
                rect(0, 0, 3, 3),
            ),
            (4, 2, &[255, 255, 0, 0, 0, 255, 255, 0], rect(0, 0, 3, 2)),
            (3, 3, &[0, 0, 0, 0, 127, 0, 0, 0, 0], rect(1, 1, 1, 1)),
        ];
        for &(width, height, data, bounds) in cases {
            let clip = SkRasterClip::from_mask(width, height, data).unwrap();
            assert_eq!(clip.getBounds(), Some(bounds));
            assert!(!clip.isRect());
        }
    }

    #[test]
    fn touching_edges_reject_but_overlapping_bounds_and_holes_do_not() {
        let mut clip = SkRasterClip::default();
        assert!(clip.setRect(Some(rect(2, 3, 4, 5))));
        for outside in [
            rect(0, 3, 2, 5),
            rect(6, 3, 2, 5),
            rect(2, 0, 4, 3),
            rect(2, 8, 4, 2),
            rect(-10, -10, 1, 1),
        ] {
            assert!(clip.quickReject(outside));
        }
        for inside in [
            rect(2, 3, 1, 1),
            rect(5, 7, 1, 1),
            rect(0, 0, 3, 4),
            rect(1, 2, 8, 9),
        ] {
            assert!(!clip.quickReject(inside));
        }
        let hole =
            SkRasterClip::from_mask(3, 3, &[255, 255, 255, 255, 0, 255, 255, 255, 255]).unwrap();
        assert!(
            !hole.quickReject(rect(1, 1, 1, 1)),
            "bounds test cannot reject interior holes"
        );
        assert!(!clip.setRect(None));
        assert!(!clip.isRect());
        assert!(clip.quickReject(rect(2, 3, 4, 5)));
        assert_eq!(clip.getBounds(), None);
    }

    #[test]
    fn malformed_masks_are_rejected_and_extra_storage_is_ignored() {
        for (width, height, data) in [
            (0, 0, &[][..]),
            (0, 1, &[][..]),
            (1, 0, &[][..]),
            (2, 2, &[255, 255, 255][..]),
            (u32::MAX, 1, &[][..]),
            (1, u32::MAX, &[][..]),
            (i32::MAX as u32, i32::MAX as u32, &[][..]),
        ] {
            assert!(SkRasterClip::from_mask(width, height, data).is_none());
        }
        let opaque = SkRasterClip::from_mask(2, 1, &[255, 255, 0, 128, 0]).unwrap();
        assert_eq!(opaque.getBounds(), Some(rect(0, 0, 2, 1)));
        assert!(opaque.isRect());
        let empty = SkRasterClip::from_mask(2, 1, &[0, 0, 255, 255]).unwrap();
        assert_eq!(empty, SkRasterClip::default());
    }

    #[test]
    fn bounded_metadata_matches_full_scan_for_partial_alpha_holes_and_empty() {
        let (width, height) = (19, 13);
        for support in [None, Some(rect(4, 3, 7, 6)), Some(rect(-4, 3, 11, 8))] {
            for seed in 0..17u32 {
                let mut data = vec![0u8; (width * height) as usize];
                if let Some(bounds) = support.and_then(|b| b.intersect(&rect(0, 0, width, height)))
                {
                    for y in bounds.top()..bounds.bottom() {
                        for x in bounds.left()..bounds.right() {
                            data[y as usize * width as usize + x as usize] =
                                match (x as u32 * 3 + y as u32 * 7 + seed) % 5 {
                                    0 => 0,
                                    1 => 127,
                                    _ => 255,
                                };
                        }
                    }
                }
                assert_eq!(
                    SkRasterClip::from_mask_with_bounds(width, height, &data, support),
                    SkRasterClip::from_mask(width, height, &data),
                );
            }
        }
    }

    #[test]
    fn all_small_binary_masks_match_independent_coverage_oracle() {
        // Exhaust all 4x3 masks; compute the reference with an independent
        // second-pass containment test to validate the one-scan adapter.
        for bits in 0..(1u32 << 12) {
            let mask: Vec<u8> = (0..12)
                .map(|i| if bits & (1 << i) == 0 { 0 } else { 255 })
                .collect();
            let clip = SkRasterClip::from_mask(4, 3, &mask).unwrap();
            let points: Vec<(usize, usize)> = (0..12)
                .filter(|&i| mask[i] != 0)
                .map(|i| (i % 4, i / 4))
                .collect();
            let expected = if points.is_empty() {
                None
            } else {
                let left = points.iter().map(|p| p.0).min().unwrap();
                let right = points.iter().map(|p| p.0).max().unwrap() + 1;
                let top = points.iter().map(|p| p.1).min().unwrap();
                let bottom = points.iter().map(|p| p.1).max().unwrap() + 1;
                Some(rect(
                    left as i32,
                    top as i32,
                    (right - left) as u32,
                    (bottom - top) as u32,
                ))
            };
            assert_eq!(clip.getBounds(), expected);
            let rectangle = expected.is_some_and(|r| {
                (r.top()..r.bottom()).all(|y| {
                    (r.left()..r.right()).all(|x| mask[y as usize * 4 + x as usize] == 255)
                })
            });
            assert_eq!(clip.isRect(), rectangle);
        }
    }
}
