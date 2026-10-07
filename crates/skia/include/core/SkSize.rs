// Migrated unchanged in behavior from tiny-skia-path-0.12.0/src/size.rs.

// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use strict_num::NonZeroPositiveF32;

use crate::path::{IntRect, LengthU32, NonZeroRect, Rect};

#[cfg(all(not(feature = "std"), feature = "no-std-float"))]
use crate::path::NoStdFloat;

/// An integer size.
///
/// # Guarantees
///
/// - Width and height are positive and non-zero.
#[derive(Copy, Clone, PartialEq, Debug, Hash, Eq)]
pub struct SkISize {
    width: LengthU32,
    height: LengthU32,
}

impl SkISize {
    /// Creates a new `SkISize` from width and height.
    pub fn from_wh(width: u32, height: u32) -> Option<Self> {
        Some(SkISize {
            width: LengthU32::new(width)?,
            height: LengthU32::new(height)?,
        })
    }

    pub(crate) fn from_wh_safe(width: LengthU32, height: LengthU32) -> Self {
        SkISize { width, height }
    }

    /// Returns width.
    pub fn width(&self) -> u32 {
        self.width.get()
    }

    /// Returns height.
    pub fn height(&self) -> u32 {
        self.height.get()
    }

    /// Returns width and height as a tuple.
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width(), self.height())
    }

    /// Scales current size by the specified factor.
    #[inline]
    pub fn scale_by(&self, factor: f32) -> Option<Self> {
        Self::from_wh(
            (self.width() as f32 * factor).round() as u32,
            (self.height() as f32 * factor).round() as u32,
        )
    }

    /// Scales current size to the specified size.
    #[inline]
    pub fn scale_to(&self, to: Self) -> Self {
        size_scale(*self, to, false)
    }

    /// Scales current size to the specified width.
    #[inline]
    pub fn scale_to_width(&self, new_width: u32) -> Option<Self> {
        let new_height = (new_width as f32 * self.height() as f32 / self.width() as f32).ceil();
        Self::from_wh(new_width, new_height as u32)
    }

    /// Scales current size to the specified height.
    #[inline]
    pub fn scale_to_height(&self, new_height: u32) -> Option<Self> {
        let new_width = (new_height as f32 * self.width() as f32 / self.height() as f32).ceil();
        Self::from_wh(new_width as u32, new_height)
    }

    /// Converts into [`SkSize`].
    pub fn to_size(&self) -> SkSize {
        SkSize::from_wh(self.width() as f32, self.height() as f32).unwrap()
    }

    /// Converts into [`IntRect`] at the provided position.
    pub fn to_int_rect(&self, x: i32, y: i32) -> IntRect {
        IntRect::from_xywh(x, y, self.width(), self.height()).unwrap()
    }
}

fn size_scale(s1: SkISize, s2: SkISize, expand: bool) -> SkISize {
    let rw = (s2.height() as f32 * s1.width() as f32 / s1.height() as f32).ceil() as u32;
    let with_h = if expand {
        rw <= s2.width()
    } else {
        rw >= s2.width()
    };

    if !with_h {
        SkISize::from_wh(rw, s2.height()).unwrap()
    } else {
        let h = (s2.width() as f32 * s1.height() as f32 / s1.width() as f32).ceil() as u32;
        SkISize::from_wh(s2.width(), h).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn int_size_tests() {
        assert_eq!(SkISize::from_wh(0, 0), None);
        assert_eq!(SkISize::from_wh(1, 0), None);
        assert_eq!(SkISize::from_wh(0, 1), None);

        let size = SkISize::from_wh(3, 4).unwrap();
        assert_eq!(
            size.to_int_rect(1, 2),
            IntRect::from_xywh(1, 2, 3, 4).unwrap()
        );
    }
}

/// A size.
///
/// # Guarantees
///
/// - Width and height are positive, non-zero and finite.
#[derive(Copy, Clone, PartialEq, Debug, Hash, Eq)]
pub struct SkSize {
    width: NonZeroPositiveF32,
    height: NonZeroPositiveF32,
}

impl SkSize {
    /// Creates a new `SkSize` from width and height.
    pub fn from_wh(width: f32, height: f32) -> Option<Self> {
        Some(SkSize {
            width: NonZeroPositiveF32::new(width)?,
            height: NonZeroPositiveF32::new(height)?,
        })
    }

    /// Returns width.
    pub fn width(&self) -> f32 {
        self.width.get()
    }

    /// Returns height.
    pub fn height(&self) -> f32 {
        self.height.get()
    }

    /// Scales current size to specified size.
    pub fn scale_to(&self, to: Self) -> Self {
        size_scale_f64(*self, to, false)
    }

    /// Expands current size to specified size.
    pub fn expand_to(&self, to: Self) -> Self {
        size_scale_f64(*self, to, true)
    }

    /// Scales current size by the specified factor.
    #[inline]
    pub fn scale_by(&self, factor: f32) -> Option<Self> {
        Self::from_wh(self.width() * factor, self.height() * factor)
    }

    /// Scales current size to the specified width.
    #[inline]
    pub fn scale_to_width(&self, new_width: f32) -> Option<Self> {
        let new_height = new_width * self.height() / self.width();
        Self::from_wh(new_width, new_height)
    }

    /// Scales current size to the specified height.
    #[inline]
    pub fn scale_to_height(&self, new_height: f32) -> Option<Self> {
        let new_width = new_height * self.width() / self.height();
        Self::from_wh(new_width, new_height)
    }

    /// Converts into [`SkISize`].
    pub fn to_int_size(&self) -> SkISize {
        SkISize::from_wh(
            core::cmp::max(1, self.width().round() as u32),
            core::cmp::max(1, self.height().round() as u32),
        )
        .unwrap()
    }

    /// Converts the current size to `Rect` at provided position.
    pub fn to_rect(&self, x: f32, y: f32) -> Option<Rect> {
        Rect::from_xywh(x, y, self.width.get(), self.height.get())
    }

    /// Converts the current size to `NonZeroRect` at provided position.
    pub fn to_non_zero_rect(&self, x: f32, y: f32) -> NonZeroRect {
        NonZeroRect::from_xywh(x, y, self.width.get(), self.height.get()).unwrap()
    }
}

fn size_scale_f64(s1: SkSize, s2: SkSize, expand: bool) -> SkSize {
    let rw = s2.height.get() * s1.width.get() / s1.height.get();
    let with_h = if expand {
        rw <= s2.width.get()
    } else {
        rw >= s2.width.get()
    };
    if !with_h {
        SkSize::from_wh(rw, s2.height.get()).unwrap()
    } else {
        let h = s2.width.get() * s1.height.get() / s1.width.get();
        SkSize::from_wh(s2.width.get(), h).unwrap()
    }
}

/// Legacy Rust API alias; canonical implementation name follows Skia.
pub use SkSize as Size;

/// Legacy Rust API alias; canonical implementation name follows Skia.
pub use SkISize as IntSize;
