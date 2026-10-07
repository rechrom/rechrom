// Migrated unchanged in behavior from tiny-skia-path-0.12.0/src/dash.rs.

// Copyright 2014 Google Inc.
// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// This module is a mix of SkDashPath, SkDashPathEffect, SkContourMeasure and SkPathMeasure.

use alloc::vec::Vec;

use crate::src::core::SkContourMeasure::SkContourMeasureIter;

use crate::path::Path;

use crate::path::floating_point::{FiniteF32, NonZeroPositiveF32};

use crate::path::path_builder::PathBuilder;

#[cfg(all(not(feature = "std"), feature = "no-std-float"))]
use crate::path::NoStdFloat;

/// A stroke dashing properties.
///
/// Contains an array of pairs, where the first number indicates an "on" interval
/// and the second one indicates an "off" interval;
/// a dash offset value and internal properties.
///
/// # Guarantees
///
/// - The dash array always have an even number of values.
/// - All dash array values are finite and >= 0.
/// - There is at least two dash array values.
/// - The sum of all dash array values is positive and finite.
/// - Dash offset is finite.
#[derive(Clone, PartialEq, Debug)]
pub struct StrokeDash {
    array: Vec<f32>,
    offset: f32,
    interval_len: NonZeroPositiveF32,
    first_len: f32, // TODO: PositiveF32
    first_index: usize,
}

impl StrokeDash {
    /// Creates a new stroke dashing object.
    pub fn new(dash_array: Vec<f32>, dash_offset: f32) -> Option<Self> {
        let dash_offset = FiniteF32::new(dash_offset)?;

        if dash_array.len() < 2 || dash_array.len() % 2 != 0 {
            return None;
        }

        if dash_array.iter().any(|n| *n < 0.0) {
            return None;
        }

        let interval_len: f32 = dash_array.iter().sum();
        let interval_len = NonZeroPositiveF32::new(interval_len)?;

        let dash_offset = adjust_dash_offset(dash_offset.get(), interval_len.get());
        debug_assert!(dash_offset >= 0.0);
        debug_assert!(dash_offset < interval_len.get());

        let (first_len, first_index) = find_first_interval(&dash_array, dash_offset);
        debug_assert!(first_len >= 0.0);
        debug_assert!(first_index < dash_array.len());

        Some(StrokeDash {
            array: dash_array,
            offset: dash_offset,
            interval_len,
            first_len,
            first_index,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test() {
        assert_eq!(StrokeDash::new(vec![], 0.0), None);
        assert_eq!(StrokeDash::new(vec![1.0], 0.0), None);
        assert_eq!(StrokeDash::new(vec![1.0, 2.0, 3.0], 0.0), None);
        assert_eq!(StrokeDash::new(vec![1.0, -2.0], 0.0), None);
        assert_eq!(StrokeDash::new(vec![0.0, 0.0], 0.0), None);
        assert_eq!(StrokeDash::new(vec![1.0, -1.0], 0.0), None);
        assert_eq!(StrokeDash::new(vec![1.0, 1.0], f32::INFINITY), None);
        assert_eq!(StrokeDash::new(vec![1.0, f32::INFINITY], 0.0), None);
    }

    #[test]
    fn bug_26() {
        let mut pb = PathBuilder::new();
        pb.move_to(665.54, 287.3);
        pb.line_to(675.67, 273.04);
        pb.line_to(675.52, 271.32);
        pb.line_to(674.79, 269.61);
        pb.line_to(674.05, 268.04);
        pb.line_to(672.88, 266.47);
        pb.line_to(671.27, 264.9);
        let path = pb.finish().unwrap();

        let stroke_dash = StrokeDash::new(vec![6.0, 4.5], 0.0).unwrap();

        assert!(path.dash(&stroke_dash, 1.0).is_some());
    }
}

// Adjust phase to be between 0 and len, "flipping" phase if negative.
// e.g., if len is 100, then phase of -20 (or -120) is equivalent to 80.
fn adjust_dash_offset(mut offset: f32, len: f32) -> f32 {
    if offset < 0.0 {
        offset = -offset;
        if offset > len {
            offset %= len;
        }

        offset = len - offset;

        // Due to finite precision, it's possible that phase == len,
        // even after the subtract (if len >>> phase), so fix that here.
        debug_assert!(offset <= len);
        if offset == len {
            offset = 0.0;
        }

        offset
    } else if offset >= len {
        offset % len
    } else {
        offset
    }
}

fn find_first_interval(dash_array: &[f32], mut dash_offset: f32) -> (f32, usize) {
    for (i, gap) in dash_array.iter().copied().enumerate() {
        if dash_offset > gap || (dash_offset == gap && gap != 0.0) {
            dash_offset -= gap;
        } else {
            return (gap - dash_offset, i);
        }
    }

    // If we get here, phase "appears" to be larger than our length. This
    // shouldn't happen with perfect precision, but we can accumulate errors
    // during the initial length computation (rounding can make our sum be too
    // big or too small. In that event, we just have to eat the error here.
    (dash_array[0], 0)
}

impl Path {
    /// Converts the current path into a dashed one.
    ///
    /// `resolution_scale` can be obtained via
    /// [`compute_resolution_scale`](crate::path::PathStroker::compute_resolution_scale).
    ///
    /// Returns `None` when more than 1_000_000 dashes had to be produced
    /// or when the final path has an invalid bounding box.
    pub fn dash(&self, dash: &StrokeDash, resolution_scale: f32) -> Option<Path> {
        dash_impl(self, dash, resolution_scale)
    }
}

fn dash_impl(src: &Path, dash: &StrokeDash, res_scale: f32) -> Option<Path> {
    // We do not support the `cull_path` branch here.
    // Skia has a lot of code for cases when a path contains only a single zero-length line
    // or when a path is a rect. Not sure why.
    // We simply ignoring it for the sake of simplicity.

    // We also doesn't support the `SpecialLineRec` case.
    // I have no idea what the point in it.

    fn is_even(x: usize) -> bool {
        x % 2 == 0
    }

    let mut pb = PathBuilder::new();
    let mut dash_count = 0.0;
    for contour in SkContourMeasureIter::new(src, res_scale) {
        let mut skip_first_segment = contour.is_closed;
        let mut added_segment = false;
        let length = contour.length;
        let mut index = dash.first_index;

        // Since the path length / dash length ratio may be arbitrarily large, we can exert
        // significant memory pressure while attempting to build the filtered path. To avoid this,
        // we simply give up dashing beyond a certain threshold.
        //
        // The original bug report (http://crbug.com/165432) is based on a path yielding more than
        // 90 million dash segments and crashing the memory allocator. A limit of 1 million
        // segments seems reasonable: at 2 verbs per segment * 9 bytes per verb, this caps the
        // maximum dash memory overhead at roughly 17MB per path.
        const MAX_DASH_COUNT: usize = 1000000;
        dash_count += length * (dash.array.len() >> 1) as f32 / dash.interval_len.get();
        if dash_count > MAX_DASH_COUNT as f32 {
            return None;
        }

        // Using double precision to avoid looping indefinitely due to single precision rounding
        // (for extreme path_length/dash_length ratios). See test_infinite_dash() unittest.
        let mut distance = 0.0;
        let mut d_len = dash.first_len;

        while distance < length {
            debug_assert!(d_len >= 0.0);
            added_segment = false;
            if is_even(index) && !skip_first_segment {
                added_segment = true;
                contour.push_segment(distance, distance + d_len, true, &mut pb);
            }

            distance += d_len;

            // clear this so we only respect it the first time around
            skip_first_segment = false;

            // wrap around our intervals array if necessary
            index += 1;
            debug_assert!(index <= dash.array.len());
            if index == dash.array.len() {
                index = 0;
            }

            // fetch our next d_len
            d_len = dash.array[index];
        }

        // extend if we ended on a segment and we need to join up with the (skipped) initial segment
        if contour.is_closed && is_even(dash.first_index) && dash.first_len >= 0.0 {
            contour.push_segment(0.0, dash.first_len, !added_segment, &mut pb);
        }
    }

    pb.finish()
}
