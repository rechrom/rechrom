// Migrated unchanged in behavior from tiny-skia-0.12.0/src/edge_builder.rs.

// Copyright 2011 Google Inc.
// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use alloc::vec::Vec;

use crate::path::PathVerb;

use crate::raster::{Path, Point};

use crate::raster::edge::{SkCubicEdge, SkEdge, SkEdgeKind, SkQuadraticEdge};
use crate::raster::edge_clipper::EdgeClipperIter;
use crate::raster::geom::ScreenIntRect;
use crate::raster::path_geometry;

#[derive(Copy, Clone, PartialEq, Debug)]
enum Combine {
    No,
    Partial,
    Total,
}

#[derive(Copy, Clone, Debug)]
pub struct ShiftedIntRect {
    shifted: ScreenIntRect,
    shift: i32,
}

impl ShiftedIntRect {
    pub fn new(rect: &ScreenIntRect, shift: i32) -> Option<Self> {
        let shifted = ScreenIntRect::from_xywh(
            rect.x() << shift,
            rect.y() << shift,
            rect.width() << shift,
            rect.height() << shift,
        )?;
        Some(ShiftedIntRect { shifted, shift })
    }

    pub fn shifted(&self) -> &ScreenIntRect {
        &self.shifted
    }

    pub fn recover(&self) -> ScreenIntRect {
        ScreenIntRect::from_xywh(
            self.shifted.x() >> self.shift,
            self.shifted.y() >> self.shift,
            self.shifted.width() >> self.shift,
            self.shifted.height() >> self.shift,
        )
        .unwrap() // cannot fail, because the original rect was valid
    }
}

pub struct SkBasicEdgeBuilder {
    edges: Vec<SkEdgeKind>,
    clip_shift: i32,
}

impl SkBasicEdgeBuilder {
    pub fn new(clip_shift: i32) -> Self {
        SkBasicEdgeBuilder {
            edges: Vec::with_capacity(64), // TODO: stack array + fallback
            clip_shift,
        }
    }

    // Skia returns a linked list here, but it's a nightmare to use in Rust,
    // so we're mimicking it with Vec.
    pub fn build_edges(
        path: &Path,
        clip: Option<&ShiftedIntRect>,
        clip_shift: i32,
    ) -> Option<Vec<SkEdgeKind>> {
        // If we're convex, then we need both edges, even if the right edge is past the clip.
        // let can_cull_to_the_right = !path.isConvex();
        let can_cull_to_the_right = false; // TODO: this

        let mut builder = SkBasicEdgeBuilder::new(clip_shift);
        if !builder.build(path, clip, can_cull_to_the_right) {
            log::warn!("infinite or NaN segments detected during edges building");
            return None;
        }

        if builder.edges.len() < 2 {
            return None;
        }

        Some(builder.edges)
    }

    // TODO: build_poly
    pub fn build(
        &mut self,
        path: &Path,
        clip: Option<&ShiftedIntRect>,
        can_cull_to_the_right: bool,
    ) -> bool {
        if let Some(clip) = clip {
            let clip = clip.recover().to_rect();
            for edges in EdgeClipperIter::new(path, clip, can_cull_to_the_right) {
                for edge in edges {
                    match edge {
                        PathEdge::LineTo(p0, p1) => {
                            if !p0.is_finite() || !p1.is_finite() {
                                return false;
                            }

                            self.push_line(&[p0, p1])
                        }
                        PathEdge::QuadTo(p0, p1, p2) => {
                            if !p0.is_finite() || !p1.is_finite() || !p2.is_finite() {
                                return false;
                            }

                            self.push_quad(&[p0, p1, p2])
                        }
                        PathEdge::CubicTo(p0, p1, p2, p3) => {
                            if !p0.is_finite()
                                || !p1.is_finite()
                                || !p2.is_finite()
                                || !p3.is_finite()
                            {
                                return false;
                            }

                            self.push_cubic(&[p0, p1, p2, p3])
                        }
                    }
                }
            }
        } else {
            for edge in edge_iter(path) {
                match edge {
                    PathEdge::LineTo(p0, p1) => {
                        self.push_line(&[p0, p1]);
                    }
                    PathEdge::QuadTo(p0, p1, p2) => {
                        let points = [p0, p1, p2];
                        let mut mono_x = [Point::zero(); 5];
                        let n = path_geometry::chop_quad_at_y_extrema(&points, &mut mono_x);
                        for i in 0..=n {
                            self.push_quad(&mono_x[i * 2..]);
                        }
                    }
                    PathEdge::CubicTo(p0, p1, p2, p3) => {
                        let points = [p0, p1, p2, p3];
                        let mut mono_y = [Point::zero(); 10];
                        let n = path_geometry::chop_cubic_at_y_extrema(&points, &mut mono_y);
                        for i in 0..=n {
                            self.push_cubic(&mono_y[i * 3..]);
                        }
                    }
                }
            }
        }

        true
    }

    fn push_line(&mut self, points: &[Point; 2]) {
        if let Some(edge) = SkEdge::new(points[0], points[1], self.clip_shift) {
            let combine = if edge.is_vertical() && !self.edges.is_empty() {
                if let Some(SkEdgeKind::Line(last)) = self.edges.last_mut() {
                    combine_vertical(&edge, last)
                } else {
                    Combine::No
                }
            } else {
                Combine::No
            };

            match combine {
                Combine::Total => {
                    self.edges.pop();
                }
                Combine::Partial => {}
                Combine::No => self.edges.push(SkEdgeKind::Line(edge)),
            }
        }
    }

    fn push_quad(&mut self, points: &[Point]) {
        if let Some(edge) = SkQuadraticEdge::new(points, self.clip_shift) {
            self.edges.push(SkEdgeKind::Quadratic(edge));
        }
    }

    fn push_cubic(&mut self, points: &[Point]) {
        if let Some(edge) = SkCubicEdge::new(points, self.clip_shift) {
            self.edges.push(SkEdgeKind::Cubic(edge));
        }
    }
}

fn combine_vertical(edge: &SkEdge, last: &mut SkEdge) -> Combine {
    if last.dx != 0 || edge.x != last.x {
        return Combine::No;
    }

    if edge.winding == last.winding {
        return if edge.last_y + 1 == last.first_y {
            last.first_y = edge.first_y;
            Combine::Partial
        } else if edge.first_y == last.last_y + 1 {
            last.last_y = edge.last_y;
            Combine::Partial
        } else {
            Combine::No
        };
    }

    if edge.first_y == last.first_y {
        return if edge.last_y == last.last_y {
            Combine::Total
        } else if edge.last_y < last.last_y {
            last.first_y = edge.last_y + 1;
            Combine::Partial
        } else {
            last.first_y = last.last_y + 1;
            last.last_y = edge.last_y;
            last.winding = edge.winding;
            Combine::Partial
        };
    }

    if edge.last_y == last.last_y {
        if edge.first_y > last.first_y {
            last.last_y = edge.first_y - 1;
        } else {
            last.last_y = last.first_y - 1;
            last.first_y = edge.first_y;
            last.winding = edge.winding;
        }

        return Combine::Partial;
    }

    Combine::No
}

pub fn edge_iter(path: &Path) -> PathEdgeIter<'_> {
    PathEdgeIter {
        path,
        verb_index: 0,
        points_index: 0,
        move_to: Point::zero(),
        needs_close_line: false,
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PathEdge {
    LineTo(Point, Point),
    QuadTo(Point, Point, Point),
    CubicTo(Point, Point, Point, Point),
}

/// Lightweight variant of PathIter that only returns segments (e.g. lines/quads).
///
/// Does not return Move or Close. Always "auto-closes" each contour.
pub struct PathEdgeIter<'a> {
    path: &'a Path,
    verb_index: usize,
    points_index: usize,
    move_to: Point,
    needs_close_line: bool,
}

impl PathEdgeIter<'_> {
    fn close_line(&mut self) -> Option<PathEdge> {
        self.needs_close_line = false;

        let edge = PathEdge::LineTo(self.path.points()[self.points_index - 1], self.move_to);
        Some(edge)
    }
}

impl Iterator for PathEdgeIter<'_> {
    type Item = PathEdge;

    fn next(&mut self) -> Option<Self::Item> {
        if self.verb_index < self.path.verbs().len() {
            let verb = self.path.verbs()[self.verb_index];
            self.verb_index += 1;

            match verb {
                PathVerb::Move => {
                    if self.needs_close_line {
                        let res = self.close_line();
                        self.move_to = self.path.points()[self.points_index];
                        self.points_index += 1;
                        return res;
                    }

                    self.move_to = self.path.points()[self.points_index];
                    self.points_index += 1;
                    self.next()
                }
                PathVerb::Close => {
                    if self.needs_close_line {
                        return self.close_line();
                    }

                    self.next()
                }
                _ => {
                    // Actual edge.
                    self.needs_close_line = true;

                    let edge;
                    match verb {
                        PathVerb::Line => {
                            edge = PathEdge::LineTo(
                                self.path.points()[self.points_index - 1],
                                self.path.points()[self.points_index + 0],
                            );
                            self.points_index += 1;
                        }
                        PathVerb::Quad => {
                            edge = PathEdge::QuadTo(
                                self.path.points()[self.points_index - 1],
                                self.path.points()[self.points_index + 0],
                                self.path.points()[self.points_index + 1],
                            );
                            self.points_index += 2;
                        }
                        PathVerb::Cubic => {
                            edge = PathEdge::CubicTo(
                                self.path.points()[self.points_index - 1],
                                self.path.points()[self.points_index + 0],
                                self.path.points()[self.points_index + 1],
                                self.path.points()[self.points_index + 2],
                            );
                            self.points_index += 3;
                        }
                        _ => unreachable!(),
                    };

                    Some(edge)
                }
            }
        } else if self.needs_close_line {
            self.close_line()
        } else {
            None
        }
    }
}

// Rust compatibility names for the migrated raster API.
pub use SkBasicEdgeBuilder as BasicEdgeBuilder;

// Analytic edge accumulation; the Vec is the Rust equivalent of SkAnalyticEdgeBuilder storage.
use crate::src::core::SkAnalyticEdge::{Pt, SkAnalyticEdge};
use crate::src::core::SkEdgeClipper::{clipped_quad, ClipBox};
use crate::src::core::SkGeometry::conic_to_quads;
pub(crate) fn add_line(a: Pt, b: Pt, out: &mut Vec<SkAnalyticEdge>) {
    let Some(e) = SkAnalyticEdge::setLine(a, b) else {
        return;
    };
    if e.dx == 0 {
        if let Some(last) = out
            .last_mut()
            .filter(|v| v.quad.is_none() && v.cubic.is_none() && v.dx == 0 && v.x == e.x)
        {
            let eq = |a: i32, b: i32| (a - b).abs() < 256;
            if e.winding == last.winding {
                if e.lower_y == last.upper_y {
                    last.upper_y = e.upper_y;
                    last.y = last.upper_y;
                    return;
                }
                if eq(e.upper_y, last.lower_y) {
                    last.lower_y = e.lower_y;
                    return;
                }
            } else if eq(e.upper_y, last.upper_y) {
                if eq(e.lower_y, last.lower_y) {
                    out.pop();
                    return;
                }
                if e.lower_y < last.lower_y {
                    last.upper_y = e.lower_y;
                    last.y = last.upper_y;
                    return;
                }
                last.upper_y = last.lower_y;
                last.y = last.upper_y;
                last.lower_y = e.lower_y;
                last.winding = e.winding;
                return;
            } else if eq(e.lower_y, last.lower_y) {
                if e.upper_y > last.upper_y {
                    last.lower_y = e.upper_y;
                    return;
                }
                last.lower_y = last.upper_y;
                last.upper_y = e.upper_y;
                last.y = last.upper_y;
                last.winding = e.winding;
                return;
            }
        }
    }
    out.push(e);
}

pub(crate) fn add_clipped_line(a: Pt, b: Pt, reverse: bool, out: &mut Vec<SkAnalyticEdge>) {
    if reverse {
        add_line(b, a, out);
    } else {
        add_line(a, b, out);
    }
}

pub(crate) fn add_quad(p: [Pt; 3], out: &mut Vec<SkAnalyticEdge>) {
    if let Some(e) = SkAnalyticEdge::setQuadratic(p) {
        out.push(e);
    }
}

pub(crate) fn add_conic(p: [Pt; 3], clip: Option<ClipBox>, edges: &mut Vec<SkAnalyticEdge>) {
    for q in conic_to_quads(p, std::f32::consts::FRAC_1_SQRT_2) {
        if let Some(c) = clip {
            clipped_quad(q, c, edges);
        } else {
            add_quad(q, edges);
        }
    }
}
