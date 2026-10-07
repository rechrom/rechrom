// Migrated unchanged in behavior from tiny-skia-path-0.12.0/src/path.rs.

// Copyright 2006 The Android Open Source Project
// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use alloc::vec::Vec;

use crate::path::path_builder::PathBuilder;
use crate::path::transform::Transform;
use crate::path::{Point, Rect};

#[cfg(all(not(feature = "std"), feature = "no-std-float"))]
use crate::path::NoStdFloat;

/// A path verb.
#[allow(missing_docs)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub enum PathVerb {
    Move,
    Line,
    Quad,
    Cubic,
    Close,
}

/// A Bezier path.
///
/// Can be created via [`PathBuilder`].
/// Where [`PathBuilder`] can be created from the [`SkPath`] using [`clear`] to reuse the allocation.
///
/// SkPath is immutable and uses compact storage, where segment types and numbers are stored
/// separately. Use can access path segments via [`SkPath::verbs`] and [`SkPath::points`],
/// or via [`SkPath::segments`]
///
/// # Guarantees
///
/// - Has a valid, precomputed bounds.
/// - All points are finite.
/// - Has at least two segments.
/// - Each contour starts with a MoveTo.
/// - No duplicated Move.
/// - No duplicated Close.
/// - Zero-length contours are allowed.
///
/// [`PathBuilder`]: struct.PathBuilder.html
/// [`clear`]: struct.SkPath.html#method.clear
#[derive(Clone, PartialEq)]
pub struct SkPath {
    pub(crate) verbs: Vec<PathVerb>,
    pub(crate) points: Vec<Point>,
    pub(crate) bounds: Rect,
}

impl SkPath {
    /// Returns the number of segments in the path.
    pub fn len(&self) -> usize {
        self.verbs.len()
    }

    /// Return if the path is empty.
    pub fn is_empty(&self) -> bool {
        self.verbs.is_empty()
    }

    /// Returns the bounds of the path's points.
    ///
    /// The value is already calculated.
    pub fn bounds(&self) -> Rect {
        self.bounds
    }

    /// Calculates path's tight bounds.
    ///
    /// This operation can be expensive.
    pub fn compute_tight_bounds(&self) -> Option<Rect> {
        // big enough to hold worst-case curve type (cubic) extremas + 1
        let mut extremas = [Point::zero(); 5];

        let mut min = self.points[0];
        let mut max = self.points[0];
        let mut iter = self.segments();
        let mut last_point = Point::zero();
        while let Some(segment) = iter.next() {
            let mut count = 0;
            match segment {
                PathSegment::MoveTo(p) => {
                    extremas[0] = p;
                    count = 1;
                }
                PathSegment::LineTo(p) => {
                    extremas[0] = p;
                    count = 1;
                }
                PathSegment::QuadTo(p0, p1) => {
                    count = compute_quad_extremas(last_point, p0, p1, &mut extremas);
                }
                PathSegment::CubicTo(p0, p1, p2) => {
                    count = compute_cubic_extremas(last_point, p0, p1, p2, &mut extremas);
                }
                PathSegment::Close => {}
            }

            last_point = iter.last_point;
            for tmp in &extremas[0..count] {
                min.x = min.x.min(tmp.x);
                min.y = min.y.min(tmp.y);
                max.x = max.x.max(tmp.x);
                max.y = max.y.max(tmp.y);
            }
        }

        Rect::from_ltrb(min.x, min.y, max.x, max.y)
    }

    /// Returns an internal vector of verbs.
    pub fn verbs(&self) -> &[PathVerb] {
        &self.verbs
    }

    /// Returns an internal vector of points.
    pub fn points(&self) -> &[Point] {
        &self.points
    }

    /// Returns a transformed in-place path.
    ///
    /// Some points may become NaN/inf therefore this method can fail.
    pub fn transform(mut self, ts: Transform) -> Option<Self> {
        if ts.is_identity() {
            return Some(self);
        }

        ts.map_points(&mut self.points);

        // Update bounds.
        self.bounds = Rect::from_points(&self.points)?;

        Some(self)
    }

    /// Returns an iterator over path's segments.
    pub fn segments(&self) -> PathSegmentsIter<'_> {
        PathSegmentsIter {
            path: self,
            verb_index: 0,
            points_index: 0,
            is_auto_close: false,
            last_move_to: Point::zero(),
            last_point: Point::zero(),
        }
    }

    /// Clears the path and returns a `PathBuilder` that will reuse an allocated memory.
    pub fn clear(mut self) -> PathBuilder {
        self.verbs.clear();
        self.points.clear();

        PathBuilder {
            verbs: self.verbs,
            points: self.points,
            last_move_to_index: 0,
            move_to_required: true,
        }
    }
}

impl core::fmt::Debug for SkPath {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use core::fmt::Write;

        let mut s = alloc::string::String::new();
        for segment in self.segments() {
            match segment {
                PathSegment::MoveTo(p) => s.write_fmt(format_args!("M {} {} ", p.x, p.y))?,
                PathSegment::LineTo(p) => s.write_fmt(format_args!("L {} {} ", p.x, p.y))?,
                PathSegment::QuadTo(p0, p1) => {
                    s.write_fmt(format_args!("Q {} {} {} {} ", p0.x, p0.y, p1.x, p1.y))?
                }
                PathSegment::CubicTo(p0, p1, p2) => s.write_fmt(format_args!(
                    "C {} {} {} {} {} {} ",
                    p0.x, p0.y, p1.x, p1.y, p2.x, p2.y
                ))?,
                PathSegment::Close => s.write_fmt(format_args!("Z "))?,
            }
        }

        s.pop(); // ' '

        f.debug_struct("SkPath")
            .field("segments", &s)
            .field("bounds", &self.bounds)
            .finish()
    }
}

fn compute_quad_extremas(p0: Point, p1: Point, p2: Point, extremas: &mut [Point; 5]) -> usize {
    use crate::path::path_geometry;

    let src = [p0, p1, p2];
    let mut extrema_idx = 0;
    if let Some(t) = path_geometry::find_quad_extrema(p0.x, p1.x, p2.x) {
        extremas[extrema_idx] = path_geometry::eval_quad_at(&src, t.to_normalized());
        extrema_idx += 1;
    }
    if let Some(t) = path_geometry::find_quad_extrema(p0.y, p1.y, p2.y) {
        extremas[extrema_idx] = path_geometry::eval_quad_at(&src, t.to_normalized());
        extrema_idx += 1;
    }
    extremas[extrema_idx] = p2;
    extrema_idx + 1
}

fn compute_cubic_extremas(
    p0: Point,
    p1: Point,
    p2: Point,
    p3: Point,
    extremas: &mut [Point; 5],
) -> usize {
    use crate::path::path_geometry;

    let mut ts0 = path_geometry::new_t_values();
    let mut ts1 = path_geometry::new_t_values();
    let n0 = path_geometry::find_cubic_extrema(p0.x, p1.x, p2.x, p3.x, &mut ts0);
    let n1 = path_geometry::find_cubic_extrema(p0.y, p1.y, p2.y, p3.y, &mut ts1);
    let total_len = n0 + n1;
    debug_assert!(total_len <= 4);

    let src = [p0, p1, p2, p3];
    let mut extrema_idx = 0;
    for t in &ts0[0..n0] {
        extremas[extrema_idx] = path_geometry::eval_cubic_pos_at(&src, t.to_normalized());
        extrema_idx += 1;
    }
    for t in &ts1[0..n1] {
        extremas[extrema_idx] = path_geometry::eval_cubic_pos_at(&src, t.to_normalized());
        extrema_idx += 1;
    }
    extremas[total_len] = p3;
    total_len + 1
}

/// A path segment.
#[allow(missing_docs)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PathSegment {
    MoveTo(Point),
    LineTo(Point),
    QuadTo(Point, Point),
    CubicTo(Point, Point, Point),
    Close,
}

/// A path segments iterator.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct PathSegmentsIter<'a> {
    path: &'a SkPath,
    verb_index: usize,
    points_index: usize,

    is_auto_close: bool,
    last_move_to: Point,
    last_point: Point,
}

impl PathSegmentsIter<'_> {
    /// Sets the auto closing mode. Off by default.
    ///
    /// When enabled, emits an additional `PathSegment::Line` from the current position
    /// to the previous `PathSegment::Move`. And only then emits `PathSegment::Close`.
    pub fn set_auto_close(&mut self, flag: bool) {
        self.is_auto_close = flag;
    }

    pub(crate) fn auto_close(&mut self) -> PathSegment {
        if self.is_auto_close && self.last_point != self.last_move_to {
            self.verb_index -= 1;
            PathSegment::LineTo(self.last_move_to)
        } else {
            PathSegment::Close
        }
    }

    pub(crate) fn has_valid_tangent(&self) -> bool {
        let mut iter = self.clone();
        while let Some(segment) = iter.next() {
            match segment {
                PathSegment::MoveTo(_) => {
                    return false;
                }
                PathSegment::LineTo(p) => {
                    if iter.last_point == p {
                        continue;
                    }

                    return true;
                }
                PathSegment::QuadTo(p1, p2) => {
                    if iter.last_point == p1 && iter.last_point == p2 {
                        continue;
                    }

                    return true;
                }
                PathSegment::CubicTo(p1, p2, p3) => {
                    if iter.last_point == p1 && iter.last_point == p2 && iter.last_point == p3 {
                        continue;
                    }

                    return true;
                }
                PathSegment::Close => {
                    return false;
                }
            }
        }

        false
    }

    /// Returns the current verb.
    pub fn curr_verb(&self) -> PathVerb {
        self.path.verbs[self.verb_index - 1]
    }

    /// Returns the next verb.
    pub fn next_verb(&self) -> Option<PathVerb> {
        self.path.verbs.get(self.verb_index).cloned()
    }
}

impl Iterator for PathSegmentsIter<'_> {
    type Item = PathSegment;

    fn next(&mut self) -> Option<Self::Item> {
        if self.verb_index < self.path.verbs.len() {
            let verb = self.path.verbs[self.verb_index];
            self.verb_index += 1;

            match verb {
                PathVerb::Move => {
                    self.points_index += 1;
                    self.last_move_to = self.path.points[self.points_index - 1];
                    self.last_point = self.last_move_to;
                    Some(PathSegment::MoveTo(self.last_move_to))
                }
                PathVerb::Line => {
                    self.points_index += 1;
                    self.last_point = self.path.points[self.points_index - 1];
                    Some(PathSegment::LineTo(self.last_point))
                }
                PathVerb::Quad => {
                    self.points_index += 2;
                    self.last_point = self.path.points[self.points_index - 1];
                    Some(PathSegment::QuadTo(
                        self.path.points[self.points_index - 2],
                        self.last_point,
                    ))
                }
                PathVerb::Cubic => {
                    self.points_index += 3;
                    self.last_point = self.path.points[self.points_index - 1];
                    Some(PathSegment::CubicTo(
                        self.path.points[self.points_index - 3],
                        self.path.points[self.points_index - 2],
                        self.last_point,
                    ))
                }
                PathVerb::Close => {
                    let seg = self.auto_close();
                    self.last_point = self.last_move_to;
                    Some(seg)
                }
            }
        } else {
            None
        }
    }
}

// CPU implementation moved from renderer/analytic_aa/path_geometry.rs.
/// Rust adapter for SkPath.cpp Convexicator; not an upstream namespace.
pub mod convexity {
    //! SkPathPriv.cpp::Convexicator, applied after the device-space transform.
    //! BSD license: ../glyphs/SKIA_LICENSE.
    use crate::src::core::SkAnalyticEdge::Pt;
    fn vector(a: Pt, b: Pt) -> Pt {
        (b.0 - a.0, b.1 - a.1)
    }
    fn zero(p: Pt) -> bool {
        p.0 == 0.0 && p.1 == 0.0
    }
    pub(crate) fn is_convex(points: &[Pt], single_contour: bool) -> bool {
        if !single_contour || points.is_empty() {
            return false;
        }
        // Convexicator::IsConcaveBySign rejects polygons winding repeatedly even
        // when all their local turns have the same sign.
        if points.len() > 3 {
            let mut previous = points[0];
            let mut sx = 2;
            let mut sy = 2;
            let mut dx = 0;
            let mut dy = 0;
            for &p in points[1..].iter().chain(std::iter::once(&points[0])) {
                let v = vector(previous, p);
                if !zero(v) {
                    if !v.0.is_finite() || !v.1.is_finite() {
                        return false;
                    }
                    let x = i32::from(v.0 < 0.0);
                    let y = i32::from(v.1 < 0.0);
                    dx += i32::from(x != sx);
                    dy += i32::from(y != sy);
                    if dx > 3 || dy > 3 {
                        return false;
                    }
                    sx = x;
                    sy = y;
                }
                previous = p;
            }
        }
        let mut previous = points[0];
        let mut first = None;
        let mut expected = 0;
        let mut reversals = 0;
        // Keep first-vector initialization separate from the mutable state used
        // by add_vector, matching setMovePt/addPt/close.
        let mut vectors = Vec::new();
        for &p in points[1..].iter().chain(std::iter::once(&points[0])) {
            if previous != p {
                let v = vector(previous, p);
                if first.is_none() {
                    first = Some(v);
                }
                vectors.push(v);
                previous = p;
            }
        }
        let Some(first) = first else {
            return true;
        };
        // The first vector has no preceding direction change.
        let mut last = first;
        let mut add_vector = |v: Pt| {
            let cross = last.0 * v.1 - last.1 * v.0;
            if !cross.is_finite() {
                return false;
            }
            if cross == 0.0 {
                if last.0 * v.0 + last.1 * v.1 < 0.0 {
                    last = v;
                    reversals += 1;
                    return reversals < 3;
                }
            } else {
                let direction = if cross > 0.0 { 1 } else { -1 };
                if expected == 0 {
                    expected = direction;
                } else if direction != expected {
                    return false;
                }
                last = v;
            }
            true
        };
        vectors[1..]
            .iter()
            .copied()
            .chain(std::iter::once(first))
            .all(&mut add_vector)
    }
}

/// Legacy Rust API alias; canonical implementation name follows Skia.
pub use SkPath as Path;
