// Migrated unchanged in behavior from tiny-skia-path-0.12.0/src/stroker.rs.

// Copyright 2008 The Android Open Source Project
// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Based on SkStroke.cpp

use super::SkStrokerPriv::{
    butt_capper, miter_joiner, round_joiner, CapFactory as cap_factory, JoinFactory as join_factory,
};
use crate::path::{Path, Point, Transform};

use crate::path::dash::StrokeDash;
use crate::path::floating_point::{NonZeroPositiveF32, NormalizedF32, NormalizedF32Exclusive};
use crate::path::path::{PathSegment, PathSegmentsIter};
use crate::path::path_builder::PathBuilder;
use crate::path::path_geometry;
use crate::path::scalar::{Scalar, SCALAR_NEARLY_ZERO};

#[cfg(all(not(feature = "std"), feature = "no-std-float"))]
use crate::path::NoStdFloat;

pub(crate) struct SwappableBuilders<'a> {
    pub(crate) inner: &'a mut PathBuilder,
    pub(crate) outer: &'a mut PathBuilder,
}

impl SwappableBuilders<'_> {
    pub(crate) fn swap(&mut self) {
        // Skia swaps pointers to inner and outer builders during joining,
        // but not builders itself. So a simple `core::mem::swap` will produce invalid results.
        // And if we try to use use `core::mem::swap` on references, like below,
        // borrow checker will be unhappy.
        // That's why we need this wrapper. Maybe there is a better solution.
        core::mem::swap(&mut self.inner, &mut self.outer);
    }
}

/// Stroke properties.
#[derive(Clone, PartialEq, Debug)]
pub struct Stroke {
    /// A stroke thickness.
    ///
    /// Must be >= 0.
    ///
    /// When set to 0, a hairline stroking will be used.
    ///
    /// Default: 1.0
    pub width: f32,

    /// The limit at which a sharp corner is drawn beveled.
    ///
    /// Default: 4.0
    pub miter_limit: f32,

    /// A stroke line cap.
    ///
    /// Default: Butt
    pub line_cap: LineCap,

    /// A stroke line join.
    ///
    /// Default: Miter
    pub line_join: LineJoin,

    /// A stroke dashing properties.
    ///
    /// Default: None
    pub dash: Option<StrokeDash>,
}

impl Default for Stroke {
    fn default() -> Self {
        Stroke {
            width: 1.0,
            miter_limit: 4.0,
            line_cap: LineCap::default(),
            line_join: LineJoin::default(),
            dash: None,
        }
    }
}

/// Draws at the beginning and end of an open path contour.
#[derive(Copy, Clone, Default, PartialEq, Debug)]
pub enum LineCap {
    /// No stroke extension.
    #[default]
    Butt,
    /// Adds circle.
    Round,
    /// Adds square.
    Square,
}

/// Specifies how corners are drawn when a shape is stroked.
///
/// Join affects the four corners of a stroked rectangle, and the connected segments in a
/// stroked path.
///
/// Choose miter join to draw sharp corners. Choose round join to draw a circle with a
/// radius equal to the stroke width on top of the corner. Choose bevel join to minimally
/// connect the thick strokes.
///
/// The fill path constructed to describe the stroked path respects the join setting but may
/// not contain the actual join. For instance, a fill path constructed with round joins does
/// not necessarily include circles at each connected segment.
#[derive(Copy, Clone, Default, PartialEq, Debug)]
pub enum LineJoin {
    /// Extends to miter limit, then switches to bevel.
    #[default]
    Miter,
    /// Extends to miter limit, then clips the corner.
    MiterClip,
    /// Adds circle.
    Round,
    /// Connects outside edges.
    Bevel,
}

const QUAD_RECURSIVE_LIMIT: usize = 3;

// quads with extreme widths (e.g. (0,1) (1,6) (0,3) width=5e7) recurse to point of failure
// largest seen for normal cubics: 5, 26
// largest seen for normal quads: 11
const RECURSIVE_LIMITS: [i32; 4] = [5 * 3, 26 * 3, 11 * 3, 11 * 3]; // 3x limits seen in practice

pub(crate) type CapProc = fn(
    pivot: Point,
    normal: Point,
    stop: Point,
    other_path: Option<&PathBuilder>,
    path: &mut PathBuilder,
);

pub(crate) type JoinProc = fn(
    before_unit_normal: Point,
    pivot: Point,
    after_unit_normal: Point,
    radius: f32,
    inv_miter_limit: f32,
    prev_is_line: bool,
    curr_is_line: bool,
    builders: SwappableBuilders,
);

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
enum ReductionType {
    Point,       // all curve points are practically identical
    Line,        // the control point is on the line between the ends
    Quad,        // the control point is outside the line between the ends
    Degenerate,  // the control point is on the line but outside the ends
    Degenerate2, // two control points are on the line but outside ends (cubic)
    Degenerate3, // three areas of max curvature found (for cubic)
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum StrokeType {
    Outer = 1, // use sign-opposite values later to flip perpendicular axis
    Inner = -1,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum ResultType {
    Split,      // the caller should split the quad stroke in two
    Degenerate, // the caller should add a line
    Quad,       // the caller should (continue to try to) add a quad stroke
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum IntersectRayType {
    CtrlPt,
    ResultType,
}

impl Path {
    /// Returns a stoked path.
    ///
    /// `resolution_scale` can be obtained via
    /// [`compute_resolution_scale`](SkPathStroker::compute_resolution_scale).
    ///
    /// If you plan stroking multiple paths, you can try using [`SkPathStroker`]
    /// which will preserve temporary allocations required during stroking.
    /// This might improve performance a bit.
    pub fn stroke(&self, stroke: &Stroke, resolution_scale: f32) -> Option<Path> {
        SkPathStroker::new().stroke(self, stroke, resolution_scale)
    }
}

/// A path stroker.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct SkPathStroker {
    radius: f32,
    inv_miter_limit: f32,
    res_scale: f32,
    inv_res_scale: f32,
    inv_res_scale_squared: f32,

    first_normal: Point,
    prev_normal: Point,
    first_unit_normal: Point,
    prev_unit_normal: Point,

    // on original path
    first_pt: Point,
    prev_pt: Point,

    first_outer_pt: Point,
    first_outer_pt_index_in_contour: usize,
    segment_count: i32,
    prev_is_line: bool,

    capper: CapProc,
    joiner: JoinProc,

    // outer is our working answer, inner is temp
    inner: PathBuilder,
    outer: PathBuilder,
    cusper: PathBuilder,

    stroke_type: StrokeType,

    recursion_depth: i32, // track stack depth to abort if numerics run amok
    found_tangents: bool, // do less work until tangents meet (cubic)
    join_completed: bool, // previous join was not degenerate
}

impl Default for SkPathStroker {
    fn default() -> Self {
        SkPathStroker::new()
    }
}

impl SkPathStroker {
    /// Creates a new SkPathStroker.
    pub fn new() -> Self {
        SkPathStroker {
            radius: 0.0,
            inv_miter_limit: 0.0,
            res_scale: 1.0,
            inv_res_scale: 1.0,
            inv_res_scale_squared: 1.0,

            first_normal: Point::zero(),
            prev_normal: Point::zero(),
            first_unit_normal: Point::zero(),
            prev_unit_normal: Point::zero(),

            first_pt: Point::zero(),
            prev_pt: Point::zero(),

            first_outer_pt: Point::zero(),
            first_outer_pt_index_in_contour: 0,
            segment_count: -1,
            prev_is_line: false,

            capper: butt_capper,
            joiner: miter_joiner,

            inner: PathBuilder::new(),
            outer: PathBuilder::new(),
            cusper: PathBuilder::new(),

            stroke_type: StrokeType::Outer,

            recursion_depth: 0,
            found_tangents: false,
            join_completed: false,
        }
    }

    /// Computes a resolution scale.
    ///
    /// Resolution scale is the "intended" resolution for the output. Default is 1.0.
    ///
    /// Larger values (res > 1) indicate that the result should be more precise, since it will
    /// be zoomed up, and small errors will be magnified.
    ///
    /// Smaller values (0 < res < 1) indicate that the result can be less precise, since it will
    /// be zoomed down, and small errors may be invisible.
    pub fn compute_resolution_scale(ts: &Transform) -> f32 {
        let sx = Point::from_xy(ts.sx, ts.kx).length();
        let sy = Point::from_xy(ts.ky, ts.sy).length();
        if sx.is_finite() && sy.is_finite() {
            let scale = sx.max(sy);
            if scale > 0.0 {
                return scale;
            }
        }

        1.0
    }

    /// Stokes the path.
    ///
    /// Can be called multiple times to reuse allocated buffers.
    ///
    /// `resolution_scale` can be obtained via
    /// [`compute_resolution_scale`](Self::compute_resolution_scale).
    pub fn stroke(&mut self, path: &Path, stroke: &Stroke, resolution_scale: f32) -> Option<Path> {
        let width = NonZeroPositiveF32::new(stroke.width)?;
        self.stroke_inner(
            path,
            width,
            stroke.miter_limit,
            stroke.line_cap,
            stroke.line_join,
            resolution_scale,
        )
    }

    fn stroke_inner(
        &mut self,
        path: &Path,
        width: NonZeroPositiveF32,
        miter_limit: f32,
        line_cap: LineCap,
        mut line_join: LineJoin,
        res_scale: f32,
    ) -> Option<Path> {
        // TODO: stroke_rect optimization

        let mut inv_miter_limit = 0.0;

        if line_join == LineJoin::Miter {
            if miter_limit <= 1.0 {
                line_join = LineJoin::Bevel;
            } else {
                inv_miter_limit = miter_limit.invert();
            }
        }

        if line_join == LineJoin::MiterClip {
            inv_miter_limit = miter_limit.invert();
        }

        self.res_scale = res_scale;
        // The '4' below matches the fill scan converter's error term.
        self.inv_res_scale = (res_scale * 4.0).invert();
        self.inv_res_scale_squared = self.inv_res_scale.sqr();

        self.radius = width.get().half();
        self.inv_miter_limit = inv_miter_limit;

        self.first_normal = Point::zero();
        self.prev_normal = Point::zero();
        self.first_unit_normal = Point::zero();
        self.prev_unit_normal = Point::zero();

        self.first_pt = Point::zero();
        self.prev_pt = Point::zero();

        self.first_outer_pt = Point::zero();
        self.first_outer_pt_index_in_contour = 0;
        self.segment_count = -1;
        self.prev_is_line = false;

        self.capper = cap_factory(line_cap);
        self.joiner = join_factory(line_join);

        // Need some estimate of how large our final result (fOuter)
        // and our per-contour temp (fInner) will be, so we don't spend
        // extra time repeatedly growing these arrays.
        //
        // 1x for inner == 'wag' (worst contour length would be better guess)
        self.inner.clear();
        self.inner.reserve(path.verbs.len(), path.points.len());

        // 3x for result == inner + outer + join (swag)
        self.outer.clear();
        self.outer
            .reserve(path.verbs.len() * 3, path.points.len() * 3);

        self.cusper.clear();

        self.stroke_type = StrokeType::Outer;

        self.recursion_depth = 0;
        self.found_tangents = false;
        self.join_completed = false;

        let mut last_segment_is_line = false;
        let mut iter = path.segments();
        iter.set_auto_close(true);
        while let Some(segment) = iter.next() {
            match segment {
                PathSegment::MoveTo(p) => {
                    self.move_to(p);
                }
                PathSegment::LineTo(p) => {
                    self.line_to(p, Some(&iter));
                    last_segment_is_line = true;
                }
                PathSegment::QuadTo(p1, p2) => {
                    self.quad_to(p1, p2);
                    last_segment_is_line = false;
                }
                PathSegment::CubicTo(p1, p2, p3) => {
                    self.cubic_to(p1, p2, p3);
                    last_segment_is_line = false;
                }
                PathSegment::Close => {
                    if line_cap != LineCap::Butt {
                        // If the stroke consists of a moveTo followed by a close, treat it
                        // as if it were followed by a zero-length line. Lines without length
                        // can have square and round end caps.
                        if self.has_only_move_to() {
                            self.line_to(self.move_to_pt(), None);
                            last_segment_is_line = true;
                            continue;
                        }

                        // If the stroke consists of a moveTo followed by one or more zero-length
                        // verbs, then followed by a close, treat is as if it were followed by a
                        // zero-length line. Lines without length can have square & round end caps.
                        if self.is_current_contour_empty() {
                            last_segment_is_line = true;
                            continue;
                        }
                    }

                    self.close(last_segment_is_line);
                }
            }
        }

        self.finish(last_segment_is_line)
    }

    fn builders(&mut self) -> SwappableBuilders<'_> {
        SwappableBuilders {
            inner: &mut self.inner,
            outer: &mut self.outer,
        }
    }

    fn move_to_pt(&self) -> Point {
        self.first_pt
    }

    fn move_to(&mut self, p: Point) {
        if self.segment_count > 0 {
            self.finish_contour(false, false);
        }

        self.segment_count = 0;
        self.first_pt = p;
        self.prev_pt = p;
        self.join_completed = false;
    }

    fn line_to(&mut self, p: Point, iter: Option<&PathSegmentsIter>) {
        let teeny_line = self
            .prev_pt
            .equals_within_tolerance(p, SCALAR_NEARLY_ZERO * self.inv_res_scale);
        if fn_ptr_eq(self.capper, butt_capper) && teeny_line {
            return;
        }

        if teeny_line && (self.join_completed || iter.map(|i| i.has_valid_tangent()) == Some(true))
        {
            return;
        }

        let mut normal = Point::zero();
        let mut unit_normal = Point::zero();
        if !self.pre_join_to(p, true, &mut normal, &mut unit_normal) {
            return;
        }

        self.outer.line_to(p.x + normal.x, p.y + normal.y);
        self.inner.line_to(p.x - normal.x, p.y - normal.y);

        self.post_join_to(p, normal, unit_normal);
    }

    fn quad_to(&mut self, p1: Point, p2: Point) {
        let quad = [self.prev_pt, p1, p2];
        let (reduction, reduction_type) = check_quad_linear(&quad);
        if reduction_type == ReductionType::Point {
            // If the stroke consists of a moveTo followed by a degenerate curve, treat it
            // as if it were followed by a zero-length line. Lines without length
            // can have square and round end caps.
            self.line_to(p2, None);
            return;
        }

        if reduction_type == ReductionType::Line {
            self.line_to(p2, None);
            return;
        }

        if reduction_type == ReductionType::Degenerate {
            self.line_to(reduction, None);
            let save_joiner = self.joiner;
            self.joiner = round_joiner;
            self.line_to(p2, None);
            self.joiner = save_joiner;
            return;
        }

        debug_assert_eq!(reduction_type, ReductionType::Quad);

        let mut normal_ab = Point::zero();
        let mut unit_ab = Point::zero();
        let mut normal_bc = Point::zero();
        let mut unit_bc = Point::zero();
        if !self.pre_join_to(p1, false, &mut normal_ab, &mut unit_ab) {
            self.line_to(p2, None);
            return;
        }

        let mut quad_points = SkQuadConstruct::default();
        self.init_quad(
            StrokeType::Outer,
            NormalizedF32::ZERO,
            NormalizedF32::ONE,
            &mut quad_points,
        );
        self.quad_stroke(&quad, &mut quad_points);
        self.init_quad(
            StrokeType::Inner,
            NormalizedF32::ZERO,
            NormalizedF32::ONE,
            &mut quad_points,
        );
        self.quad_stroke(&quad, &mut quad_points);

        let ok = set_normal_unit_normal(
            quad[1],
            quad[2],
            self.res_scale,
            self.radius,
            &mut normal_bc,
            &mut unit_bc,
        );
        if !ok {
            normal_bc = normal_ab;
            unit_bc = unit_ab;
        }

        self.post_join_to(p2, normal_bc, unit_bc);
    }

    fn cubic_to(&mut self, pt1: Point, pt2: Point, pt3: Point) {
        let cubic = [self.prev_pt, pt1, pt2, pt3];
        let mut reduction = [Point::zero(); 3];
        let mut tangent_pt = Point::zero();
        let reduction_type = check_cubic_linear(&cubic, &mut reduction, Some(&mut tangent_pt));
        if reduction_type == ReductionType::Point {
            // If the stroke consists of a moveTo followed by a degenerate curve, treat it
            // as if it were followed by a zero-length line. Lines without length
            // can have square and round end caps.
            self.line_to(pt3, None);
            return;
        }

        if reduction_type == ReductionType::Line {
            self.line_to(pt3, None);
            return;
        }

        if ReductionType::Degenerate <= reduction_type
            && ReductionType::Degenerate3 >= reduction_type
        {
            self.line_to(reduction[0], None);
            let save_joiner = self.joiner;
            self.joiner = round_joiner;
            if ReductionType::Degenerate2 <= reduction_type {
                self.line_to(reduction[1], None);
            }

            if ReductionType::Degenerate3 == reduction_type {
                self.line_to(reduction[2], None);
            }

            self.line_to(pt3, None);
            self.joiner = save_joiner;
            return;
        }

        debug_assert_eq!(reduction_type, ReductionType::Quad);
        let mut normal_ab = Point::zero();
        let mut unit_ab = Point::zero();
        let mut normal_cd = Point::zero();
        let mut unit_cd = Point::zero();
        if !self.pre_join_to(tangent_pt, false, &mut normal_ab, &mut unit_ab) {
            self.line_to(pt3, None);
            return;
        }

        let mut t_values = path_geometry::new_t_values();
        let t_values = path_geometry::find_cubic_inflections(&cubic, &mut t_values);
        let mut last_t = NormalizedF32::ZERO;
        for index in 0..=t_values.len() {
            let next_t = t_values
                .get(index)
                .cloned()
                .map(|n| n.to_normalized())
                .unwrap_or(NormalizedF32::ONE);

            let mut quad_points = SkQuadConstruct::default();
            self.init_quad(StrokeType::Outer, last_t, next_t, &mut quad_points);
            self.cubic_stroke(&cubic, &mut quad_points);
            self.init_quad(StrokeType::Inner, last_t, next_t, &mut quad_points);
            self.cubic_stroke(&cubic, &mut quad_points);
            last_t = next_t;
        }

        if let Some(cusp) = path_geometry::find_cubic_cusp(&cubic) {
            let cusp_loc = path_geometry::eval_cubic_pos_at(&cubic, cusp.to_normalized());
            self.cusper.push_circle(cusp_loc.x, cusp_loc.y, self.radius);
        }

        // emit the join even if one stroke succeeded but the last one failed
        // this avoids reversing an inner stroke with a partial path followed by another moveto
        self.set_cubic_end_normal(&cubic, normal_ab, unit_ab, &mut normal_cd, &mut unit_cd);

        self.post_join_to(pt3, normal_cd, unit_cd);
    }

    fn cubic_stroke(&mut self, cubic: &[Point; 4], quad_points: &mut SkQuadConstruct) -> bool {
        if !self.found_tangents {
            let result_type = self.tangents_meet(cubic, quad_points);
            if result_type != ResultType::Quad {
                let ok = points_within_dist(
                    quad_points.quad[0],
                    quad_points.quad[2],
                    self.inv_res_scale,
                );
                if (result_type == ResultType::Degenerate || ok)
                    && self.cubic_mid_on_line(cubic, quad_points)
                {
                    self.add_degenerate_line(quad_points);
                    return true;
                }
            } else {
                self.found_tangents = true;
            }
        }

        if self.found_tangents {
            let result_type = self.compare_quad_cubic(cubic, quad_points);
            if result_type == ResultType::Quad {
                let stroke = &quad_points.quad;
                if self.stroke_type == StrokeType::Outer {
                    self.outer
                        .quad_to(stroke[1].x, stroke[1].y, stroke[2].x, stroke[2].y);
                } else {
                    self.inner
                        .quad_to(stroke[1].x, stroke[1].y, stroke[2].x, stroke[2].y);
                }

                return true;
            }

            if result_type == ResultType::Degenerate {
                if !quad_points.opposite_tangents {
                    self.add_degenerate_line(quad_points);
                    return true;
                }
            }
        }

        if !quad_points.quad[2].x.is_finite() || !quad_points.quad[2].x.is_finite() {
            return false; // just abort if projected quad isn't representable
        }

        self.recursion_depth += 1;
        if self.recursion_depth > RECURSIVE_LIMITS[self.found_tangents as usize] {
            return false; // just abort if projected quad isn't representable
        }

        let mut half = SkQuadConstruct::default();
        if !half.init_with_start(quad_points) {
            self.add_degenerate_line(quad_points);
            self.recursion_depth -= 1;
            return true;
        }

        if !self.cubic_stroke(cubic, &mut half) {
            return false;
        }

        if !half.init_with_end(quad_points) {
            self.add_degenerate_line(quad_points);
            self.recursion_depth -= 1;
            return true;
        }

        if !self.cubic_stroke(cubic, &mut half) {
            return false;
        }

        self.recursion_depth -= 1;
        true
    }

    fn cubic_mid_on_line(&self, cubic: &[Point; 4], quad_points: &mut SkQuadConstruct) -> bool {
        let mut stroke_mid = Point::zero();
        self.cubic_quad_mid(cubic, quad_points, &mut stroke_mid);
        let dist = pt_to_line(stroke_mid, quad_points.quad[0], quad_points.quad[2]);
        dist < self.inv_res_scale_squared
    }

    fn cubic_quad_mid(
        &self,
        cubic: &[Point; 4],
        quad_points: &mut SkQuadConstruct,
        mid: &mut Point,
    ) {
        let mut cubic_mid_pt = Point::zero();
        self.cubic_perp_ray(cubic, quad_points.mid_t, &mut cubic_mid_pt, mid, None);
    }

    // Given a cubic and t, return the point on curve,
    // its perpendicular, and the perpendicular tangent.
    fn cubic_perp_ray(
        &self,
        cubic: &[Point; 4],
        t: NormalizedF32,
        t_pt: &mut Point,
        on_pt: &mut Point,
        tangent: Option<&mut Point>,
    ) {
        *t_pt = path_geometry::eval_cubic_pos_at(cubic, t);
        let mut dxy = path_geometry::eval_cubic_tangent_at(cubic, t);

        let mut chopped = [Point::zero(); 7];
        if dxy.x == 0.0 && dxy.y == 0.0 {
            let mut c_points: &[Point] = cubic;
            if t.get().is_nearly_zero() {
                dxy = cubic[2] - cubic[0];
            } else if (1.0 - t.get()).is_nearly_zero() {
                dxy = cubic[3] - cubic[1];
            } else {
                // If the cubic inflection falls on the cusp, subdivide the cubic
                // to find the tangent at that point.
                //
                // Unwrap never fails, because we already checked that `t` is not 0/1,
                let t = NormalizedF32Exclusive::new(t.get()).unwrap();
                path_geometry::chop_cubic_at2(cubic, t, &mut chopped);
                dxy = chopped[3] - chopped[2];
                if dxy.x == 0.0 && dxy.y == 0.0 {
                    dxy = chopped[3] - chopped[1];
                    c_points = &chopped;
                }
            }

            if dxy.x == 0.0 && dxy.y == 0.0 {
                dxy = c_points[3] - c_points[0];
            }
        }

        self.set_ray_points(*t_pt, &mut dxy, on_pt, tangent);
    }

    fn set_cubic_end_normal(
        &mut self,
        cubic: &[Point; 4],
        normal_ab: Point,
        unit_normal_ab: Point,
        normal_cd: &mut Point,
        unit_normal_cd: &mut Point,
    ) {
        let mut ab = cubic[1] - cubic[0];
        let mut cd = cubic[3] - cubic[2];

        let mut degenerate_ab = degenerate_vector(ab);
        let mut degenerate_cb = degenerate_vector(cd);

        if degenerate_ab && degenerate_cb {
            *normal_cd = normal_ab;
            *unit_normal_cd = unit_normal_ab;
            return;
        }

        if degenerate_ab {
            ab = cubic[2] - cubic[0];
            degenerate_ab = degenerate_vector(ab);
        }

        if degenerate_cb {
            cd = cubic[3] - cubic[1];
            degenerate_cb = degenerate_vector(cd);
        }

        if degenerate_ab || degenerate_cb {
            *normal_cd = normal_ab;
            *unit_normal_cd = unit_normal_ab;
            return;
        }

        let res = set_normal_unit_normal2(cd, self.radius, normal_cd, unit_normal_cd);
        debug_assert!(res);
    }

    fn compare_quad_cubic(
        &self,
        cubic: &[Point; 4],
        quad_points: &mut SkQuadConstruct,
    ) -> ResultType {
        // get the quadratic approximation of the stroke
        self.cubic_quad_ends(cubic, quad_points);
        let result_type = self.intersect_ray(IntersectRayType::CtrlPt, quad_points);
        if result_type != ResultType::Quad {
            return result_type;
        }

        // project a ray from the curve to the stroke
        // points near midpoint on quad, midpoint on cubic
        let mut ray0 = Point::zero();
        let mut ray1 = Point::zero();
        self.cubic_perp_ray(cubic, quad_points.mid_t, &mut ray1, &mut ray0, None);
        self.stroke_close_enough(&quad_points.quad.clone(), &[ray0, ray1], quad_points)
    }

    // Given a cubic and a t range, find the start and end if they haven't been found already.
    fn cubic_quad_ends(&self, cubic: &[Point; 4], quad_points: &mut SkQuadConstruct) {
        if !quad_points.start_set {
            let mut cubic_start_pt = Point::zero();
            self.cubic_perp_ray(
                cubic,
                quad_points.start_t,
                &mut cubic_start_pt,
                &mut quad_points.quad[0],
                Some(&mut quad_points.tangent_start),
            );
            quad_points.start_set = true;
        }

        if !quad_points.end_set {
            let mut cubic_end_pt = Point::zero();
            self.cubic_perp_ray(
                cubic,
                quad_points.end_t,
                &mut cubic_end_pt,
                &mut quad_points.quad[2],
                Some(&mut quad_points.tangent_end),
            );
            quad_points.end_set = true;
        }
    }

    fn close(&mut self, is_line: bool) {
        self.finish_contour(true, is_line);
    }

    fn finish_contour(&mut self, close: bool, curr_is_line: bool) {
        if self.segment_count > 0 {
            if close {
                (self.joiner)(
                    self.prev_unit_normal,
                    self.prev_pt,
                    self.first_unit_normal,
                    self.radius,
                    self.inv_miter_limit,
                    self.prev_is_line,
                    curr_is_line,
                    self.builders(),
                );
                self.outer.close();

                // now add inner as its own contour
                let pt = self.inner.last_point().unwrap_or_default();
                self.outer.move_to(pt.x, pt.y);
                self.outer.reverse_path_to(&self.inner);
                self.outer.close();
            } else {
                // add caps to start and end

                // cap the end
                let pt = self.inner.last_point().unwrap_or_default();
                let other_path = if curr_is_line {
                    Some(&self.inner)
                } else {
                    None
                };
                (self.capper)(
                    self.prev_pt,
                    self.prev_normal,
                    pt,
                    other_path,
                    &mut self.outer,
                );
                self.outer.reverse_path_to(&self.inner);

                // cap the start
                let other_path = if self.prev_is_line {
                    Some(&self.inner)
                } else {
                    None
                };
                (self.capper)(
                    self.first_pt,
                    -self.first_normal,
                    self.first_outer_pt,
                    other_path,
                    &mut self.outer,
                );
                self.outer.close();
            }

            if !self.cusper.is_empty() {
                self.outer.push_path_builder(&self.cusper);
                self.cusper.clear();
            }
        }

        // since we may re-use `inner`, we rewind instead of reset, to save on
        // reallocating its internal storage.
        self.inner.clear();
        self.segment_count = -1;
        self.first_outer_pt_index_in_contour = self.outer.points.len();
    }

    fn pre_join_to(
        &mut self,
        p: Point,
        curr_is_line: bool,
        normal: &mut Point,
        unit_normal: &mut Point,
    ) -> bool {
        debug_assert!(self.segment_count >= 0);

        let prev_x = self.prev_pt.x;
        let prev_y = self.prev_pt.y;

        let normal_set = set_normal_unit_normal(
            self.prev_pt,
            p,
            self.res_scale,
            self.radius,
            normal,
            unit_normal,
        );
        if !normal_set {
            if fn_ptr_eq(self.capper, butt_capper) {
                return false;
            }

            // Square caps and round caps draw even if the segment length is zero.
            // Since the zero length segment has no direction, set the orientation
            // to upright as the default orientation.
            *normal = Point::from_xy(self.radius, 0.0);
            *unit_normal = Point::from_xy(1.0, 0.0);
        }

        if self.segment_count == 0 {
            self.first_normal = *normal;
            self.first_unit_normal = *unit_normal;
            self.first_outer_pt = Point::from_xy(prev_x + normal.x, prev_y + normal.y);

            self.outer
                .move_to(self.first_outer_pt.x, self.first_outer_pt.y);
            self.inner.move_to(prev_x - normal.x, prev_y - normal.y);
        } else {
            // we have a previous segment
            (self.joiner)(
                self.prev_unit_normal,
                self.prev_pt,
                *unit_normal,
                self.radius,
                self.inv_miter_limit,
                self.prev_is_line,
                curr_is_line,
                self.builders(),
            );
        }
        self.prev_is_line = curr_is_line;
        true
    }

    fn post_join_to(&mut self, p: Point, normal: Point, unit_normal: Point) {
        self.join_completed = true;
        self.prev_pt = p;
        self.prev_unit_normal = unit_normal;
        self.prev_normal = normal;
        self.segment_count += 1;
    }

    fn init_quad(
        &mut self,
        stroke_type: StrokeType,
        start: NormalizedF32,
        end: NormalizedF32,
        quad_points: &mut SkQuadConstruct,
    ) {
        self.stroke_type = stroke_type;
        self.found_tangents = false;
        quad_points.init(start, end);
    }

    fn quad_stroke(&mut self, quad: &[Point; 3], quad_points: &mut SkQuadConstruct) -> bool {
        let result_type = self.compare_quad_quad(quad, quad_points);
        if result_type == ResultType::Quad {
            let path = if self.stroke_type == StrokeType::Outer {
                &mut self.outer
            } else {
                &mut self.inner
            };

            path.quad_to(
                quad_points.quad[1].x,
                quad_points.quad[1].y,
                quad_points.quad[2].x,
                quad_points.quad[2].y,
            );

            return true;
        }

        if result_type == ResultType::Degenerate {
            self.add_degenerate_line(quad_points);
            return true;
        }

        self.recursion_depth += 1;
        if self.recursion_depth > RECURSIVE_LIMITS[QUAD_RECURSIVE_LIMIT] {
            return false; // just abort if projected quad isn't representable
        }

        let mut half = SkQuadConstruct::default();
        half.init_with_start(quad_points);
        if !self.quad_stroke(quad, &mut half) {
            return false;
        }

        half.init_with_end(quad_points);
        if !self.quad_stroke(quad, &mut half) {
            return false;
        }

        self.recursion_depth -= 1;
        true
    }

    fn compare_quad_quad(
        &mut self,
        quad: &[Point; 3],
        quad_points: &mut SkQuadConstruct,
    ) -> ResultType {
        // get the quadratic approximation of the stroke
        if !quad_points.start_set {
            let mut quad_start_pt = Point::zero();
            self.quad_perp_ray(
                quad,
                quad_points.start_t,
                &mut quad_start_pt,
                &mut quad_points.quad[0],
                Some(&mut quad_points.tangent_start),
            );
            quad_points.start_set = true;
        }

        if !quad_points.end_set {
            let mut quad_end_pt = Point::zero();
            self.quad_perp_ray(
                quad,
                quad_points.end_t,
                &mut quad_end_pt,
                &mut quad_points.quad[2],
                Some(&mut quad_points.tangent_end),
            );
            quad_points.end_set = true;
        }

        let result_type = self.intersect_ray(IntersectRayType::CtrlPt, quad_points);
        if result_type != ResultType::Quad {
            return result_type;
        }

        // project a ray from the curve to the stroke
        let mut ray0 = Point::zero();
        let mut ray1 = Point::zero();
        self.quad_perp_ray(quad, quad_points.mid_t, &mut ray1, &mut ray0, None);
        self.stroke_close_enough(&quad_points.quad.clone(), &[ray0, ray1], quad_points)
    }

    // Given a point on the curve and its derivative, scale the derivative by the radius, and
    // compute the perpendicular point and its tangent.
    fn set_ray_points(
        &self,
        tp: Point,
        dxy: &mut Point,
        on_p: &mut Point,
        mut tangent: Option<&mut Point>,
    ) {
        if !dxy.set_length(self.radius) {
            *dxy = Point::from_xy(self.radius, 0.0);
        }

        let axis_flip = self.stroke_type as i32 as f32; // go opposite ways for outer, inner
        on_p.x = tp.x + axis_flip * dxy.y;
        on_p.y = tp.y - axis_flip * dxy.x;

        if let Some(ref mut tangent) = tangent {
            tangent.x = on_p.x + dxy.x;
            tangent.y = on_p.y + dxy.y;
        }
    }

    // Given a quad and t, return the point on curve,
    // its perpendicular, and the perpendicular tangent.
    fn quad_perp_ray(
        &self,
        quad: &[Point; 3],
        t: NormalizedF32,
        tp: &mut Point,
        on_p: &mut Point,
        tangent: Option<&mut Point>,
    ) {
        *tp = path_geometry::eval_quad_at(quad, t);
        let mut dxy = path_geometry::eval_quad_tangent_at(quad, t);

        if dxy.is_zero() {
            dxy = quad[2] - quad[0];
        }

        self.set_ray_points(*tp, &mut dxy, on_p, tangent);
    }

    fn add_degenerate_line(&mut self, quad_points: &SkQuadConstruct) {
        if self.stroke_type == StrokeType::Outer {
            self.outer
                .line_to(quad_points.quad[2].x, quad_points.quad[2].y);
        } else {
            self.inner
                .line_to(quad_points.quad[2].x, quad_points.quad[2].y);
        }
    }

    fn stroke_close_enough(
        &self,
        stroke: &[Point; 3],
        ray: &[Point; 2],
        quad_points: &mut SkQuadConstruct,
    ) -> ResultType {
        let half = NormalizedF32::new_clamped(0.5);
        let stroke_mid = path_geometry::eval_quad_at(stroke, half);
        // measure the distance from the curve to the quad-stroke midpoint, compare to radius
        if points_within_dist(ray[0], stroke_mid, self.inv_res_scale) {
            // if the difference is small
            if sharp_angle(&quad_points.quad) {
                return ResultType::Split;
            }

            return ResultType::Quad;
        }

        // measure the distance to quad's bounds (quick reject)
        // an alternative : look for point in triangle
        if !pt_in_quad_bounds(stroke, ray[0], self.inv_res_scale) {
            // if far, subdivide
            return ResultType::Split;
        }

        // measure the curve ray distance to the quad-stroke
        let mut roots = path_geometry::new_t_values();
        let roots = intersect_quad_ray(ray, stroke, &mut roots);
        if roots.len() != 1 {
            return ResultType::Split;
        }

        let quad_pt = path_geometry::eval_quad_at(stroke, roots[0].to_normalized());
        let error = self.inv_res_scale * (1.0 - (roots[0].get() - 0.5).abs() * 2.0);
        if points_within_dist(ray[0], quad_pt, error) {
            // if the difference is small, we're done
            if sharp_angle(&quad_points.quad) {
                return ResultType::Split;
            }

            return ResultType::Quad;
        }

        // otherwise, subdivide
        ResultType::Split
    }

    // Find the intersection of the stroke tangents to construct a stroke quad.
    // Return whether the stroke is a degenerate (a line), a quad, or must be split.
    // Optionally compute the quad's control point.
    fn intersect_ray(
        &self,
        intersect_ray_type: IntersectRayType,
        quad_points: &mut SkQuadConstruct,
    ) -> ResultType {
        let start = quad_points.quad[0];
        let end = quad_points.quad[2];
        let a_len = quad_points.tangent_start - start;
        let b_len = quad_points.tangent_end - end;

        // Slopes match when denom goes to zero:
        //                   axLen / ayLen ==                   bxLen / byLen
        // (ayLen * byLen) * axLen / ayLen == (ayLen * byLen) * bxLen / byLen
        //          byLen  * axLen         ==  ayLen          * bxLen
        //          byLen  * axLen         -   ayLen          * bxLen         ( == denom )
        let denom = a_len.cross(b_len);
        if denom == 0.0 || !denom.is_finite() {
            quad_points.opposite_tangents = a_len.dot(b_len) < 0.0;
            return ResultType::Degenerate;
        }

        quad_points.opposite_tangents = false;
        let ab0 = start - end;
        let mut numer_a = b_len.cross(ab0);
        let numer_b = a_len.cross(ab0);
        if (numer_a >= 0.0) == (numer_b >= 0.0) {
            // if the control point is outside the quad ends

            // if the perpendicular distances from the quad points to the opposite tangent line
            // are small, a straight line is good enough
            let dist1 = pt_to_line(start, end, quad_points.tangent_end);
            let dist2 = pt_to_line(end, start, quad_points.tangent_start);
            if dist1.max(dist2) <= self.inv_res_scale_squared {
                return ResultType::Degenerate;
            }

            return ResultType::Split;
        }

        // check to see if the denominator is teeny relative to the numerator
        // if the offset by one will be lost, the ratio is too large
        numer_a /= denom;
        let valid_divide = numer_a > numer_a - 1.0;
        if valid_divide {
            if intersect_ray_type == IntersectRayType::CtrlPt {
                // the intersection of the tangents need not be on the tangent segment
                // so 0 <= numer_a <= 1 is not necessarily true
                quad_points.quad[1].x =
                    start.x * (1.0 - numer_a) + quad_points.tangent_start.x * numer_a;
                quad_points.quad[1].y =
                    start.y * (1.0 - numer_a) + quad_points.tangent_start.y * numer_a;
            }

            return ResultType::Quad;
        }

        quad_points.opposite_tangents = a_len.dot(b_len) < 0.0;

        // if the lines are parallel, straight line is good enough
        ResultType::Degenerate
    }

    // Given a cubic and a t-range, determine if the stroke can be described by a quadratic.
    fn tangents_meet(&self, cubic: &[Point; 4], quad_points: &mut SkQuadConstruct) -> ResultType {
        self.cubic_quad_ends(cubic, quad_points);
        self.intersect_ray(IntersectRayType::ResultType, quad_points)
    }

    fn finish(&mut self, is_line: bool) -> Option<Path> {
        self.finish_contour(false, is_line);

        // Swap out the outer builder.
        let mut buf = PathBuilder::new();
        core::mem::swap(&mut self.outer, &mut buf);

        buf.finish()
    }

    fn has_only_move_to(&self) -> bool {
        self.segment_count == 0
    }

    fn is_current_contour_empty(&self) -> bool {
        self.inner.is_zero_length_since_point(0)
            && self
                .outer
                .is_zero_length_since_point(self.first_outer_pt_index_in_contour)
    }
}

fn set_normal_unit_normal(
    before: Point,
    after: Point,
    scale: f32,
    radius: f32,
    normal: &mut Point,
    unit_normal: &mut Point,
) -> bool {
    if !unit_normal.set_normalize((after.x - before.x) * scale, (after.y - before.y) * scale) {
        return false;
    }

    unit_normal.rotate_ccw();
    *normal = unit_normal.scaled(radius);
    true
}

fn set_normal_unit_normal2(
    vec: Point,
    radius: f32,
    normal: &mut Point,
    unit_normal: &mut Point,
) -> bool {
    if !unit_normal.set_normalize(vec.x, vec.y) {
        return false;
    }

    unit_normal.rotate_ccw();
    *normal = unit_normal.scaled(radius);
    true
}

fn fn_ptr_eq(f1: CapProc, f2: CapProc) -> bool {
    core::ptr::eq(f1 as *const (), f2 as *const ())
}

#[derive(Debug)]
struct SkQuadConstruct {
    // The state of the quad stroke under construction.
    quad: [Point; 3],       // the stroked quad parallel to the original curve
    tangent_start: Point,   // a point tangent to quad[0]
    tangent_end: Point,     // a point tangent to quad[2]
    start_t: NormalizedF32, // a segment of the original curve
    mid_t: NormalizedF32,
    end_t: NormalizedF32,
    start_set: bool, // state to share common points across structs
    end_set: bool,
    opposite_tangents: bool, // set if coincident tangents have opposite directions
}

impl Default for SkQuadConstruct {
    fn default() -> Self {
        Self {
            quad: Default::default(),
            tangent_start: Point::default(),
            tangent_end: Point::default(),
            start_t: NormalizedF32::ZERO,
            mid_t: NormalizedF32::ZERO,
            end_t: NormalizedF32::ZERO,
            start_set: false,
            end_set: false,
            opposite_tangents: false,
        }
    }
}

impl SkQuadConstruct {
    // return false if start and end are too close to have a unique middle
    fn init(&mut self, start: NormalizedF32, end: NormalizedF32) -> bool {
        self.start_t = start;
        self.mid_t = NormalizedF32::new_clamped((start.get() + end.get()).half());
        self.end_t = end;
        self.start_set = false;
        self.end_set = false;
        self.start_t < self.mid_t && self.mid_t < self.end_t
    }

    fn init_with_start(&mut self, parent: &Self) -> bool {
        if !self.init(parent.start_t, parent.mid_t) {
            return false;
        }

        self.quad[0] = parent.quad[0];
        self.tangent_start = parent.tangent_start;
        self.start_set = true;
        true
    }

    fn init_with_end(&mut self, parent: &Self) -> bool {
        if !self.init(parent.mid_t, parent.end_t) {
            return false;
        }

        self.quad[2] = parent.quad[2];
        self.tangent_end = parent.tangent_end;
        self.end_set = true;
        true
    }
}

fn check_quad_linear(quad: &[Point; 3]) -> (Point, ReductionType) {
    let degenerate_ab = degenerate_vector(quad[1] - quad[0]);
    let degenerate_bc = degenerate_vector(quad[2] - quad[1]);
    if degenerate_ab & degenerate_bc {
        return (Point::zero(), ReductionType::Point);
    }

    if degenerate_ab | degenerate_bc {
        return (Point::zero(), ReductionType::Line);
    }

    if !quad_in_line(quad) {
        return (Point::zero(), ReductionType::Quad);
    }

    let t = path_geometry::find_quad_max_curvature(quad);
    if t == NormalizedF32::ZERO || t == NormalizedF32::ONE {
        return (Point::zero(), ReductionType::Line);
    }

    (
        path_geometry::eval_quad_at(quad, t),
        ReductionType::Degenerate,
    )
}

fn degenerate_vector(v: Point) -> bool {
    !v.can_normalize()
}

/// Given quad, see if all there points are in a line.
/// Return true if the inside point is close to a line connecting the outermost points.
///
/// Find the outermost point by looking for the largest difference in X or Y.
/// Since the XOR of the indices is 3  (0 ^ 1 ^ 2)
/// the missing index equals: outer_1 ^ outer_2 ^ 3.
fn quad_in_line(quad: &[Point; 3]) -> bool {
    let mut pt_max = -1.0;
    let mut outer1 = 0;
    let mut outer2 = 0;
    for index in 0..2 {
        for inner in index + 1..3 {
            let test_diff = quad[inner] - quad[index];
            let test_max = test_diff.x.abs().max(test_diff.y.abs());
            if pt_max < test_max {
                outer1 = index;
                outer2 = inner;
                pt_max = test_max;
            }
        }
    }

    debug_assert!(outer1 <= 1);
    debug_assert!(outer2 >= 1 && outer2 <= 2);
    debug_assert!(outer1 < outer2);

    let mid = outer1 ^ outer2 ^ 3;
    const CURVATURE_SLOP: f32 = 0.000005; // this multiplier is pulled out of the air
    let line_slop = pt_max * pt_max * CURVATURE_SLOP;
    pt_to_line(quad[mid], quad[outer1], quad[outer2]) <= line_slop
}

// returns the distance squared from the point to the line
fn pt_to_line(pt: Point, line_start: Point, line_end: Point) -> f32 {
    let dxy = line_end - line_start;
    let ab0 = pt - line_start;
    let numer = dxy.dot(ab0);
    let denom = dxy.dot(dxy);
    let t = numer / denom;
    if t >= 0.0 && t <= 1.0 {
        let hit = Point::from_xy(
            line_start.x * (1.0 - t) + line_end.x * t,
            line_start.y * (1.0 - t) + line_end.y * t,
        );
        hit.distance_to_sqd(pt)
    } else {
        pt.distance_to_sqd(line_start)
    }
}

// Intersect the line with the quad and return the t values on the quad where the line crosses.
fn intersect_quad_ray<'a>(
    line: &[Point; 2],
    quad: &[Point; 3],
    roots: &'a mut [NormalizedF32Exclusive; 3],
) -> &'a [NormalizedF32Exclusive] {
    let vec = line[1] - line[0];
    let mut r = [0.0; 3];
    for n in 0..3 {
        r[n] = (quad[n].y - line[0].y) * vec.x - (quad[n].x - line[0].x) * vec.y;
    }
    let mut a = r[2];
    let mut b = r[1];
    let c = r[0];
    a += c - 2.0 * b; // A = a - 2*b + c
    b -= c; // B = -(b - c)

    let len = path_geometry::find_unit_quad_roots(a, 2.0 * b, c, roots);
    &roots[0..len]
}

fn points_within_dist(near_pt: Point, far_pt: Point, limit: f32) -> bool {
    near_pt.distance_to_sqd(far_pt) <= limit * limit
}

fn sharp_angle(quad: &[Point; 3]) -> bool {
    let mut smaller = quad[1] - quad[0];
    let mut larger = quad[1] - quad[2];
    let smaller_len = smaller.length_sqd();
    let mut larger_len = larger.length_sqd();
    if smaller_len > larger_len {
        core::mem::swap(&mut smaller, &mut larger);
        larger_len = smaller_len;
    }

    if !smaller.set_length(larger_len) {
        return false;
    }

    let dot = smaller.dot(larger);
    dot > 0.0
}

// Return true if the point is close to the bounds of the quad. This is used as a quick reject.
fn pt_in_quad_bounds(quad: &[Point; 3], pt: Point, inv_res_scale: f32) -> bool {
    let x_min = quad[0].x.min(quad[1].x).min(quad[2].x);
    if pt.x + inv_res_scale < x_min {
        return false;
    }

    let x_max = quad[0].x.max(quad[1].x).max(quad[2].x);
    if pt.x - inv_res_scale > x_max {
        return false;
    }

    let y_min = quad[0].y.min(quad[1].y).min(quad[2].y);
    if pt.y + inv_res_scale < y_min {
        return false;
    }

    let y_max = quad[0].y.max(quad[1].y).max(quad[2].y);
    if pt.y - inv_res_scale > y_max {
        return false;
    }

    true
}

fn check_cubic_linear(
    cubic: &[Point; 4],
    reduction: &mut [Point; 3],
    tangent_pt: Option<&mut Point>,
) -> ReductionType {
    let degenerate_ab = degenerate_vector(cubic[1] - cubic[0]);
    let degenerate_bc = degenerate_vector(cubic[2] - cubic[1]);
    let degenerate_cd = degenerate_vector(cubic[3] - cubic[2]);
    if degenerate_ab & degenerate_bc & degenerate_cd {
        return ReductionType::Point;
    }

    if degenerate_ab as i32 + degenerate_bc as i32 + degenerate_cd as i32 == 2 {
        return ReductionType::Line;
    }

    if !cubic_in_line(cubic) {
        if let Some(tangent_pt) = tangent_pt {
            *tangent_pt = if degenerate_ab { cubic[2] } else { cubic[1] };
        }

        return ReductionType::Quad;
    }

    let mut t_values = [NormalizedF32::ZERO; 3];
    let t_values = path_geometry::find_cubic_max_curvature(cubic, &mut t_values);
    let mut r_count = 0;
    // Now loop over the t-values, and reject any that evaluate to either end-point
    for t in t_values {
        if 0.0 >= t.get() || t.get() >= 1.0 {
            continue;
        }

        reduction[r_count] = path_geometry::eval_cubic_pos_at(cubic, *t);
        if reduction[r_count] != cubic[0] && reduction[r_count] != cubic[3] {
            r_count += 1;
        }
    }

    match r_count {
        0 => ReductionType::Line,
        1 => ReductionType::Degenerate,
        2 => ReductionType::Degenerate2,
        3 => ReductionType::Degenerate3,
        _ => unreachable!(),
    }
}

/// Given a cubic, determine if all four points are in a line.
///
/// Return true if the inner points is close to a line connecting the outermost points.
///
/// Find the outermost point by looking for the largest difference in X or Y.
/// Given the indices of the outermost points, and that outer_1 is greater than outer_2,
/// this table shows the index of the smaller of the remaining points:
///
/// ```text
///                   outer_2
///               0    1    2    3
///   outer_1     ----------------
///      0     |  -    2    1    1
///      1     |  -    -    0    0
///      2     |  -    -    -    0
///      3     |  -    -    -    -
/// ```
///
/// If outer_1 == 0 and outer_2 == 1, the smaller of the remaining indices (2 and 3) is 2.
///
/// This table can be collapsed to: (1 + (2 >> outer_2)) >> outer_1
///
/// Given three indices (outer_1 outer_2 mid_1) from 0..3, the remaining index is:
///
/// ```text
/// mid_2 == (outer_1 ^ outer_2 ^ mid_1)
/// ```
fn cubic_in_line(cubic: &[Point; 4]) -> bool {
    let mut pt_max = -1.0;
    let mut outer1 = 0;
    let mut outer2 = 0;
    for index in 0..3 {
        for inner in index + 1..4 {
            let test_diff = cubic[inner] - cubic[index];
            let test_max = test_diff.x.abs().max(test_diff.y.abs());
            if pt_max < test_max {
                outer1 = index;
                outer2 = inner;
                pt_max = test_max;
            }
        }
    }
    debug_assert!(outer1 <= 2);
    debug_assert!(outer2 >= 1 && outer2 <= 3);
    debug_assert!(outer1 < outer2);
    let mid1 = (1 + (2 >> outer2)) >> outer1;
    debug_assert!(mid1 <= 2);
    debug_assert!(outer1 != mid1 && outer2 != mid1);
    let mid2 = outer1 ^ outer2 ^ mid1;
    debug_assert!(mid2 >= 1 && mid2 <= 3);
    debug_assert!(mid2 != outer1 && mid2 != outer2 && mid2 != mid1);
    debug_assert!(((1 << outer1) | (1 << outer2) | (1 << mid1) | (1 << mid2)) == 0x0f);
    let line_slop = pt_max * pt_max * 0.00001; // this multiplier is pulled out of the air

    pt_to_line(cubic[mid1], cubic[outer1], cubic[outer2]) <= line_slop
        && pt_to_line(cubic[mid2], cubic[outer1], cubic[outer2]) <= line_slop
}

#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use super::*;

    impl PathSegment {
        fn new_move_to(x: f32, y: f32) -> Self {
            PathSegment::MoveTo(Point::from_xy(x, y))
        }

        fn new_line_to(x: f32, y: f32) -> Self {
            PathSegment::LineTo(Point::from_xy(x, y))
        }

        // fn new_quad_to(x1: f32, y1: f32, x: f32, y: f32) -> Self {
        //     PathSegment::QuadTo(Point::from_xy(x1, y1), Point::from_xy(x, y))
        // }

        // fn new_cubic_to(x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) -> Self {
        //     PathSegment::CubicTo(Point::from_xy(x1, y1), Point::from_xy(x2, y2), Point::from_xy(x, y))
        // }

        fn new_close() -> Self {
            PathSegment::Close
        }
    }

    // Make sure that subpath auto-closing is enabled.
    #[test]
    fn auto_close() {
        // A triangle.
        let mut pb = PathBuilder::new();
        pb.move_to(10.0, 10.0);
        pb.line_to(20.0, 50.0);
        pb.line_to(30.0, 10.0);
        pb.close();
        let path = pb.finish().unwrap();

        let stroke = Stroke::default();
        let stroke_path = SkPathStroker::new().stroke(&path, &stroke, 1.0).unwrap();

        let mut iter = stroke_path.segments();
        iter.set_auto_close(true);

        assert_eq!(iter.next().unwrap(), PathSegment::new_move_to(10.485071, 9.878732));
        assert_eq!(iter.next().unwrap(), PathSegment::new_line_to(20.485071, 49.878731));
        assert_eq!(iter.next().unwrap(), PathSegment::new_line_to(20.0, 50.0));
        assert_eq!(iter.next().unwrap(), PathSegment::new_line_to(19.514929, 49.878731));
        assert_eq!(iter.next().unwrap(), PathSegment::new_line_to(29.514929, 9.878732));
        assert_eq!(iter.next().unwrap(), PathSegment::new_line_to(30.0, 10.0));
        assert_eq!(iter.next().unwrap(), PathSegment::new_line_to(30.0, 10.5));
        assert_eq!(iter.next().unwrap(), PathSegment::new_line_to(10.0, 10.5));
        assert_eq!(iter.next().unwrap(), PathSegment::new_line_to(10.0, 10.0));
        assert_eq!(iter.next().unwrap(), PathSegment::new_line_to(10.485071, 9.878732));
        assert_eq!(iter.next().unwrap(), PathSegment::new_close());
        assert_eq!(iter.next().unwrap(), PathSegment::new_move_to(9.3596115, 9.5));
        assert_eq!(iter.next().unwrap(), PathSegment::new_line_to(30.640388, 9.5));
        assert_eq!(iter.next().unwrap(), PathSegment::new_line_to(20.485071, 50.121269));
        assert_eq!(iter.next().unwrap(), PathSegment::new_line_to(19.514929, 50.121269));
        assert_eq!(iter.next().unwrap(), PathSegment::new_line_to(9.514929, 10.121268));
        assert_eq!(iter.next().unwrap(), PathSegment::new_line_to(9.3596115, 9.5));
        assert_eq!(iter.next().unwrap(), PathSegment::new_close());
    }

    // From skia/tests/StrokeTest.cpp
    #[test]
    fn cubic_1() {
        let mut pb = PathBuilder::new();
        pb.move_to(51.0161362, 1511.52478);
        pb.cubic_to(
            51.0161362, 1511.52478,
            51.0161362, 1511.52478,
            51.0161362, 1511.52478,
        );
        let path = pb.finish().unwrap();

        let mut stroke = Stroke::default();
        stroke.width = 0.394537568;

        assert!(SkPathStroker::new().stroke(&path, &stroke, 1.0).is_none());
    }

    // From skia/tests/StrokeTest.cpp
    #[test]
    fn cubic_2() {
        let mut pb = PathBuilder::new();
        pb.move_to(f32::from_bits(0x424c1086), f32::from_bits(0x44bcf0cb)); // 51.0161362, 1511.52478
        pb.cubic_to(
            f32::from_bits(0x424c107c), f32::from_bits(0x44bcf0cb), // 51.0160980, 1511.52478
            f32::from_bits(0x424c10c2), f32::from_bits(0x44bcf0cb), // 51.0163651, 1511.52478
            f32::from_bits(0x424c1119), f32::from_bits(0x44bcf0ca), // 51.0166969, 1511.52466
        );
        let path = pb.finish().unwrap();

        let mut stroke = Stroke::default();
        stroke.width = 0.394537568;

        assert!(SkPathStroker::new().stroke(&path, &stroke, 1.0).is_some());
    }

    // From skia/tests/StrokeTest.cpp
    // From skbug.com/6491. The large stroke width can cause numerical instabilities.
    #[test]
    fn big() {
        // Skia uses `kStrokeAndFill_Style` here, but we do not support it.

        let mut pb = PathBuilder::new();
        pb.move_to(f32::from_bits(0x46380000), f32::from_bits(0xc6380000)); // 11776, -11776
        pb.line_to(f32::from_bits(0x46a00000), f32::from_bits(0xc6a00000)); // 20480, -20480
        pb.line_to(f32::from_bits(0x468c0000), f32::from_bits(0xc68c0000)); // 17920, -17920
        pb.line_to(f32::from_bits(0x46100000), f32::from_bits(0xc6100000)); // 9216, -9216
        pb.line_to(f32::from_bits(0x46380000), f32::from_bits(0xc6380000)); // 11776, -11776
        pb.close();
        let path = pb.finish().unwrap();

        let mut stroke = Stroke::default();
        stroke.width = 1.49679073e+10;

        assert!(SkPathStroker::new().stroke(&path, &stroke, 1.0).is_some());
    }

    // From skia/tests/StrokerTest.cpp
    #[test]
    fn quad_stroker_one_off() {
        let mut pb = PathBuilder::new();
        pb.move_to(f32::from_bits(0x43c99223), f32::from_bits(0x42b7417e));
        pb.quad_to(
            f32::from_bits(0x4285d839), f32::from_bits(0x43ed6645),
            f32::from_bits(0x43c941c8), f32::from_bits(0x42b3ace3),
        );
        let path = pb.finish().unwrap();

        let mut stroke = Stroke::default();
        stroke.width = 164.683548;

        assert!(SkPathStroker::new().stroke(&path, &stroke, 1.0).is_some());
    }

    // From skia/tests/StrokerTest.cpp
    #[test]
    fn cubic_stroker_one_off() {
        let mut pb = PathBuilder::new();
        pb.move_to(f32::from_bits(0x433f5370), f32::from_bits(0x43d1f4b3));
        pb.cubic_to(
            f32::from_bits(0x4331cb76), f32::from_bits(0x43ea3340),
            f32::from_bits(0x4388f498), f32::from_bits(0x42f7f08d),
            f32::from_bits(0x43f1cd32), f32::from_bits(0x42802ec1),
        );
        let path = pb.finish().unwrap();

        let mut stroke = Stroke::default();
        stroke.width = 42.835968;

        assert!(SkPathStroker::new().stroke(&path, &stroke, 1.0).is_some());
    }
}

// CPU implementation moved from renderer/stroke.rs.

/// Legacy Rust API alias.
pub use SkPathStroker as PathStroker;
/// Command-stream adapter for SkPathStroker; not an upstream namespace.
pub mod command_stroker {
    //! SkStroke.cpp curve-parallel quadratic construction and contour finishing.
    //! Copyright 2008 The Android Open Source Project. BSD-3-Clause; glyphs/SKIA_LICENSE.
    //! Keep tangent vectors, not tangent endpoints: the current Skia representation
    //! avoids subtracting large coordinates during ray intersection.
    use crate::compat::commands::{Offset, PaintPathCommand, PaintPathVerb as V};
    use crate::compat::commands::{SvgStrokeLineCap, SvgStrokeLineJoin};
    use std::ops::{Add, Mul, Sub};
    #[derive(Clone, Copy, Default, PartialEq)]
    struct P(f32, f32);
    impl Add for P {
        type Output = Self;
        fn add(self, b: Self) -> Self {
            Self(self.0 + b.0, self.1 + b.1)
        }
    }
    impl Sub for P {
        type Output = Self;
        fn sub(self, b: Self) -> Self {
            Self(self.0 - b.0, self.1 - b.1)
        }
    }
    impl Mul<f32> for P {
        type Output = Self;
        fn mul(self, s: f32) -> Self {
            Self(self.0 * s, self.1 * s)
        }
    }
    impl P {
        fn dot(self, b: Self) -> f32 {
            self.0 * b.0 + self.1 * b.1
        }
        fn cross(self, b: Self) -> f32 {
            self.0 * b.1 - self.1 * b.0
        }
        fn squared(self) -> f32 {
            self.dot(self)
        }
        fn finite(self) -> bool {
            self.0.is_finite() && self.1.is_finite()
        }
        fn length(self, radius: f32) -> Option<Self> {
            // SkPoint::setLength computes in double even for finite small vectors.
            let x = f64::from(self.0);
            let y = f64::from(self.1);
            let scale = f64::from(radius) / (x * x + y * y).sqrt();
            let p = Self((x * scale) as f32, (y * scale) as f32);
            (p.finite() && p != Self::default()).then_some(p)
        }
        fn normal(self) -> Self {
            Self(self.1, -self.0)
        }
        fn offset(self) -> Offset {
            Offset {
                x: f64::from(self.0),
                y: f64::from(self.1),
            }
        }
    }
    fn point(p: Offset) -> P {
        P(p.x as f32, p.y as f32)
    }
    fn command(verb: V, p: P, control: P) -> PaintPathCommand {
        PaintPathCommand {
            verb,
            point: p.offset(),
            control1: control.offset(),
            ..Default::default()
        }
    }
    fn eval_quad(q: [P; 3], t: f32) -> P {
        let a = q[2] - q[1] * 2.0 + q[0];
        let b = (q[1] - q[0]) * 2.0;
        (a * t + b) * t + q[0]
    }
    #[derive(Clone, Copy)]
    enum Curve {
        Line([P; 2]),
        Quad([P; 3]),
        Conic([P; 3], f32),
        Cubic([P; 4]),
    }
    impl Curve {
        fn ends(self) -> (P, P) {
            match self {
                Self::Line(p) => (p[0], p[1]),
                Self::Quad(p) | Self::Conic(p, _) => (p[0], p[2]),
                Self::Cubic(p) => (p[0], p[3]),
            }
        }
        fn eval(self, t: f32) -> (P, P) {
            match self {
                Self::Line(p) => (p[0] + (p[1] - p[0]) * t, p[1] - p[0]),
                Self::Quad(p) => {
                    let pos = eval_quad(p, t);
                    let tangent = if (t == 0.0 && p[0] == p[1]) || (t == 1.0 && p[1] == p[2]) {
                        p[2] - p[0]
                    } else {
                        let b = p[1] - p[0];
                        let a = p[2] - p[1] - b;
                        (a * t + b) * 2.0
                    };
                    (pos, tangent)
                }
                Self::Conic(p, w) => {
                    // SkConicCoeff / SkConic::evalTangentAt. Preserve the
                    // polynomial evaluation and subtraction order in f32.
                    let weighted = p[1] * w;
                    let a = p[2] - weighted * 2.0 + p[0];
                    let b = (weighted - p[0]) * 2.0;
                    let db = 2.0 * (w - 1.0);
                    let denominator = (-db * t + db) * t + 1.0;
                    let numerator = (a * t + b) * t + p[0];
                    let pos = P(numerator.0 / denominator, numerator.1 / denominator);
                    let p20 = p[2] - p[0];
                    let c = (p[1] - p[0]) * w;
                    let a = p20 * w - p20;
                    let b = p20 - c - c;
                    let tangent = (a * t + b) * t + c;
                    (
                        pos,
                        if tangent == P::default() {
                            p20
                        } else {
                            tangent
                        },
                    )
                }
                Self::Cubic(p) => {
                    let a = p[3] + (p[1] - p[2]) * 3.0 - p[0];
                    let b = (p[2] - p[1] * 2.0 + p[0]) * 3.0;
                    let c = (p[1] - p[0]) * 3.0;
                    let pos = ((a * t + b) * t + c) * t + p[0];
                    let tangent = if (t == 0.0 && p[0] == p[1]) || (t == 1.0 && p[2] == p[3]) {
                        let d = if t == 0.0 { p[2] - p[0] } else { p[3] - p[1] };
                        if d == P::default() {
                            p[3] - p[0]
                        } else {
                            d
                        }
                    } else {
                        // SkQuadCoeff evaluates with C = P1 - P0 computed first.
                        // Adding P1 then subtracting P0 loses endpoint tangents.
                        (a * t + (p[2] - p[1] * 2.0 + p[0]) * 2.0) * t + (p[1] - p[0])
                    };
                    (pos, tangent)
                }
            }
        }
        fn ray(self, t: f32, radius: f32, side: f32) -> Option<(P, P, P)> {
            let (pos, d) = self.eval(t);
            let tangent = d.length(radius)?;
            Some((pos, pos + tangent.normal() * side, tangent))
        }
        fn inflections(self) -> Vec<f32> {
            match self {
                Self::Cubic(p) => {
                    let a = p[1] - p[0];
                    let b = p[2] - p[1] * 2.0 + p[0];
                    let c = p[3] + (p[1] - p[2]) * 3.0 - p[0];
                    crate::src::core::SkGeometry::unit_quad_roots(
                        b.cross(c),
                        a.cross(c),
                        a.cross(b),
                    )
                }
                _ => vec![],
            }
        }
    }
    #[derive(Clone, Copy)]
    struct QuadConstruct {
        quad: [P; 3],
        tangent_start: P,
        tangent_end: P,
        start: f32,
        mid: f32,
        end: f32,
        opposite: bool,
    }
    impl QuadConstruct {
        fn new(curve: Curve, start: f32, end: f32, radius: f32, side: f32) -> Option<Self> {
            let mid = (start + end) * 0.5;
            if start >= mid || mid >= end {
                return None;
            }
            let (_, a, ta) = curve.ray(start, radius, side)?;
            let (_, b, tb) = curve.ray(end, radius, side)?;
            Some(Self {
                quad: [a, P::default(), b],
                tangent_start: ta,
                tangent_end: tb,
                start,
                mid,
                end,
                opposite: false,
            })
        }
    }
    #[derive(PartialEq)]
    enum ResultType {
        Degenerate,
        Quad,
        Split,
    }
    fn pt_to_line(p: P, start: P, end: P) -> f32 {
        let d = end - start;
        let t = d.dot(p - start) / d.squared();
        let hit = if (0.0..=1.0).contains(&t) {
            start * (1.0 - t) + end * t
        } else {
            start
        };
        (hit - p).squared()
    }
    fn pt_to_tangent_line(p: P, start: P, tangent: P) -> f32 {
        let t = tangent.dot(p - start) / tangent.squared();
        let hit = if (0.0..=1.0).contains(&t) {
            start + tangent * t
        } else {
            start
        };
        (hit - p).squared()
    }
    fn intersect_ray(q: &mut QuadConstruct, control: bool, tolerance: f32) -> ResultType {
        let a = q.tangent_start;
        let b = q.tangent_end;
        let denominator = a.cross(b);
        if denominator == 0.0 || !denominator.is_finite() {
            q.opposite = a.dot(b) < 0.0;
            return ResultType::Degenerate;
        }
        q.opposite = false;
        let ab = q.quad[0] - q.quad[2];
        let mut numerator = b.cross(ab);
        let numerator_b = a.cross(ab);
        if (numerator >= 0.0) == (numerator_b >= 0.0) {
            if pt_to_tangent_line(q.quad[0], q.quad[2], b)
                .max(pt_to_tangent_line(q.quad[2], q.quad[0], a))
                <= tolerance * tolerance
            {
                return ResultType::Degenerate;
            }
            return ResultType::Split;
        }
        numerator /= denominator;
        if numerator > numerator - 1.0 {
            if control {
                q.quad[1] = q.quad[0] + a * numerator;
            }
            ResultType::Quad
        } else {
            q.opposite = a.dot(b) < 0.0;
            ResultType::Degenerate
        }
    }
    fn sharp_angle(q: [P; 3]) -> bool {
        let mut smaller = q[1] - q[0];
        let mut larger = q[1] - q[2];
        let mut large_len = larger.squared();
        let small_len = smaller.squared();
        if small_len > large_len {
            std::mem::swap(&mut smaller, &mut larger);
            large_len = small_len;
        }
        smaller
            .length(large_len)
            .is_some_and(|v| v.dot(larger) > 0.0)
    }
    fn close_enough(q: [P; 3], ray: (P, P), tolerance: f32) -> bool {
        if (ray.0 - eval_quad(q, 0.5)).squared() <= tolerance * tolerance {
            return !sharp_angle(q);
        }
        let min_x = q.iter().map(|p| p.0).fold(f32::INFINITY, f32::min);
        let max_x = q.iter().map(|p| p.0).fold(f32::NEG_INFINITY, f32::max);
        let min_y = q.iter().map(|p| p.1).fold(f32::INFINITY, f32::min);
        let max_y = q.iter().map(|p| p.1).fold(f32::NEG_INFINITY, f32::max);
        if ray.0 .0 + tolerance < min_x
            || ray.0 .0 - tolerance > max_x
            || ray.0 .1 + tolerance < min_y
            || ray.0 .1 - tolerance > max_y
        {
            return false;
        }
        let vector = ray.1 - ray.0;
        let r = q.map(|p| vector.cross(p - ray.0));
        let a = r[2] + (r[0] - 2.0 * r[1]);
        let b = r[1] - r[0];
        let roots = crate::src::core::SkGeometry::unit_quad_roots(a, 2.0 * b, r[0]);
        if roots.len() != 1 {
            return false;
        }
        let t = roots[0];
        let error = tolerance * (1.0 - (t - 0.5).abs() * 2.0);
        (ray.0 - eval_quad(q, t)).squared() <= error * error && !sharp_angle(q)
    }
    fn curve_stroke(
        curve: Curve,
        q: &mut QuadConstruct,
        radius: f32,
        side: f32,
        found: &mut bool,
        depth: u32,
        tolerance: f32,
        out: &mut Vec<PaintPathCommand>,
    ) -> Option<()> {
        let cubic = matches!(curve, Curve::Cubic(_));
        if cubic && !*found {
            let result = intersect_ray(q, false, tolerance);
            if result == ResultType::Quad {
                *found = true;
            } else {
                let (_, mid, _) = curve.ray(q.mid, radius, side)?;
                if (result == ResultType::Degenerate
                    || (q.quad[0] - q.quad[2]).squared() <= tolerance * tolerance)
                    && pt_to_line(mid, q.quad[0], q.quad[2]) < tolerance * tolerance
                {
                    out.push(command(V::kLineTo, q.quad[2], P::default()));
                    return Some(());
                }
            }
        }
        if !cubic || *found {
            let result = intersect_ray(q, true, tolerance);
            if result == ResultType::Quad {
                let (pos, mid, _) = curve.ray(q.mid, radius, side)?;
                if close_enough(q.quad, (mid, pos), tolerance) {
                    out.push(command(V::kQuadraticTo, q.quad[2], q.quad[1]));
                    return Some(());
                }
            } else if result == ResultType::Degenerate && (!cubic || !q.opposite) {
                out.push(command(V::kLineTo, q.quad[2], P::default()));
                return Some(());
            }
        }
        if depth + 1
            > if !cubic {
                33
            } else if *found {
                24
            } else {
                15
            }
        {
            out.push(command(V::kLineTo, q.quad[2], P::default()));
            return Some(());
        }
        let mut first = QuadConstruct::new(curve, q.start, q.mid, radius, side)?;
        first.quad[0] = q.quad[0];
        first.tangent_start = q.tangent_start;
        curve_stroke(
            curve,
            &mut first,
            radius,
            side,
            found,
            depth + 1,
            tolerance,
            out,
        )?;
        let mut second = QuadConstruct::new(curve, q.mid, q.end, radius, side)?;
        second.quad[2] = q.quad[2];
        second.tangent_end = q.tangent_end;
        curve_stroke(
            curve,
            &mut second,
            radius,
            side,
            found,
            depth + 1,
            tolerance,
            out,
        )
    }
    fn reverse_contour(path: &[PaintPathCommand], out: &mut Vec<PaintPathCommand>) {
        let mut start = point(path[0].point);
        let mut pieces = Vec::new();
        for c in &path[1..] {
            pieces.push((start, *c));
            start = point(c.point);
        }
        out.push(command(V::kMoveTo, start, P::default()));
        for (start, c) in pieces.into_iter().rev() {
            let mut reversed = command(c.verb, start, point(c.control1));
            reversed.conic_weight = c.conic_weight;
            out.push(reversed);
        }
        out.push(command(V::kClose, P::default(), P::default()));
    }
    fn cap(out: &mut Vec<PaintPathCommand>, pivot: P, normal: P, stop: P, style: SvgStrokeLineCap) {
        if style == SvgStrokeLineCap::kRound {
            // SkStrokerPriv::RoundCapper: two rational quarter-circle arcs.
            let center = pivot + P(-normal.1, normal.0);
            for (control, point) in [(center + normal, center), (center - normal, stop)] {
                let mut c = command(V::kConicTo, point, control);
                c.conic_weight = f64::from(std::f32::consts::FRAC_1_SQRT_2);
                out.push(c);
            }
        } else {
            out.push(command(V::kLineTo, stop, P::default()));
        }
    }
    // SkConic::BuildUnitArc and SkStrokerPriv::RoundJoiner. Construct each
    // rational quadrant before applying the start-vector rotation and radius.
    fn round_join(
        outer: &mut Vec<PaintPathCommand>,
        inner: &mut Vec<PaintPathCommand>,
        mut before: P,
        pivot: P,
        mut after: P,
        radius: f32,
    ) -> Option<()> {
        // SkStrokerPriv::Dot2AngleType skips nearly collinear joins before
        // choosing an arc direction, including a tiny negative cross product.
        if (1.0 - before.dot(after)).abs() <= 1.0 / 4096.0 {
            return Some(());
        }
        let ccw = before.0 * after.1 <= before.1 * after.0;
        let (outer, inner) = if ccw {
            before = before * -1.0;
            after = after * -1.0;
            (inner, outer)
        } else {
            (outer, inner)
        };
        let x = before.dot(after);
        let cross = before.cross(after);
        if cross.abs() <= 1.0 / 4096.0
            && x > 0.0
            && ((!ccw && cross >= 0.0) || (ccw && cross <= 0.0))
        {
            return Some(());
        }
        let y = if ccw { -cross } else { cross };
        let quadrant = if y == 0.0 {
            2
        } else if x == 0.0 {
            if y > 0.0 {
                1
            } else {
                3
            }
        } else {
            usize::from(y < 0.0) * 2 + usize::from((x < 0.0) != (y < 0.0))
        };
        let points = [
            P(1.0, 0.0),
            P(1.0, 1.0),
            P(0.0, 1.0),
            P(-1.0, 1.0),
            P(-1.0, 0.0),
            P(-1.0, -1.0),
            P(0.0, -1.0),
            P(1.0, -1.0),
        ];
        // SkMatrix::postConcat computes the scale/rotation coefficients first.
        let sx = before.0 * radius;
        let ky = before.1 * radius;
        let kx = if ccw { ky } else { -ky };
        let sy = if ccw { -sx } else { sx };
        let map = |p: P| {
            P(
                (sx * p.0 + kx * p.1) + pivot.0,
                (ky * p.0 + sy * p.1) + pivot.1,
            )
        };
        let mut count = 0;
        for i in 0..quadrant {
            let mut c = command(V::kConicTo, map(points[i * 2 + 2]), map(points[i * 2 + 1]));
            c.conic_weight = f64::from(std::f32::consts::FRAC_1_SQRT_2);
            outer.push(c);
            count += 1;
        }
        let last = points[quadrant * 2];
        let final_p = P(x, y);
        let dot = last.dot(final_p);
        if dot.is_nan() {
            return None;
        }
        if dot < 1.0 {
            let weight = ((1.0 + dot) / 2.0).sqrt();
            let control = (last + final_p).length(1.0 / weight)?;
            if (last.0 - control.0).abs() > 1.0 / 4096.0
                || (last.1 - control.1).abs() > 1.0 / 4096.0
            {
                let mut c = command(V::kConicTo, map(final_p), map(control));
                c.conic_weight = f64::from(weight);
                outer.push(c);
                count += 1;
            }
        }
        if count > 0 {
            inner.push(command(V::kLineTo, pivot, P::default()));
            inner.push(command(V::kLineTo, pivot - after * radius, P::default()));
        }
        Some(())
    }
    // SkStrokerPriv::MiterJoiner / BluntJoiner / HandleInnerJoin.
    fn join(
        outer: &mut Vec<PaintPathCommand>,
        inner: &mut Vec<PaintPathCommand>,
        before: P,
        pivot: P,
        after: P,
        radius: f32,
        miter_limit: f32,
        style: SvgStrokeLineJoin,
        previous_line: bool,
        mut current_line: bool,
    ) -> Option<()> {
        let dot = before.dot(after);
        if style != SvgStrokeLineJoin::kBevel && dot >= 0.0 && (1.0 - dot).abs() <= 1.0 / 4096.0 {
            return Some(());
        }
        if style == SvgStrokeLineJoin::kRound {
            return round_join(outer, inner, before, pivot, after, radius);
        }
        let bevel = style == SvgStrokeLineJoin::kBevel || miter_limit <= 1.0;
        let nearly_opposite = !bevel && dot < 0.0 && (1.0 + dot).abs() <= 1.0 / 4096.0;
        let ccw = before.0 * after.1 <= before.1 * after.0;
        let (outer, inner, before, after) = if ccw && !nearly_opposite {
            (inner, outer, before * -1.0, after * -1.0)
        } else {
            (outer, inner, before, after)
        };
        if !bevel && !nearly_opposite {
            let inverse = 1.0 / miter_limit;
            let mid = if dot == 0.0 && inverse <= std::f32::consts::FRAC_1_SQRT_2 {
                Some((before + after) * radius)
            } else {
                let half = ((1.0 + dot) / 2.0).sqrt();
                if half < inverse {
                    None
                } else {
                    let mid = if dot < 0.0 {
                        let p = P(after.1 - before.1, before.0 - after.0);
                        if ccw {
                            p * -1.0
                        } else {
                            p
                        }
                    } else {
                        before + after
                    };
                    mid.length(radius / half)
                }
            };
            if let Some(mid) = mid {
                if previous_line {
                    outer.last_mut()?.point = (pivot + mid).offset();
                } else {
                    outer.push(command(V::kLineTo, pivot + mid, P::default()));
                }
            } else {
                current_line = false;
            }
        } else if nearly_opposite {
            current_line = false;
        }
        let after = after * radius;
        if bevel || !current_line {
            outer.push(command(V::kLineTo, pivot + after, P::default()));
        }
        inner.push(command(V::kLineTo, pivot, P::default()));
        inner.push(command(V::kLineTo, pivot - after, P::default()));
        Some(())
    }

    // SkPathStroker::preJoinTo/postJoinTo/finishContour, including miter, bevel,
    // and rational round joins. Unsupported cusp/square-cap contours use the
    // existing stroker until those source branches are translated.
    pub(crate) fn outline(
        commands: &[PaintPathCommand],
        width: f32,
        cap_style: SvgStrokeLineCap,
    ) -> Option<Vec<PaintPathCommand>> {
        outline_with_scale(
            commands,
            width,
            cap_style,
            SvgStrokeLineJoin::kMiter,
            4.0,
            1.0,
        )
    }

    // SkPathUtils::FillPathWithPaint and SkPathStroker::fInvResScale.
    pub(crate) fn outline_with_scale(
        commands: &[PaintPathCommand],
        width: f32,
        cap_style: SvgStrokeLineCap,
        join_style: SvgStrokeLineJoin,
        miter_limit: f32,
        resolution_scale: f32,
    ) -> Option<Vec<PaintPathCommand>> {
        if !resolution_scale.is_finite() || resolution_scale <= 0.0 {
            return None;
        }
        let tolerance = 1.0 / (resolution_scale * 4.0);
        if width <= 0.0 || !width.is_finite() || cap_style == SvgStrokeLineCap::kSquare {
            return None;
        }
        let radius = width * 0.5;
        let mut contours = Vec::<(Vec<Curve>, bool)>::new();
        let mut curves = Vec::new();
        let mut previous = P::default();
        let mut start = P::default();
        let mut open = false;
        for c in commands {
            let p = point(c.point);
            match c.verb {
                V::kMoveTo => {
                    if open {
                        if curves.is_empty() {
                            return None;
                        }
                        contours.push((std::mem::take(&mut curves), false));
                    }
                    open = true;
                    start = p;
                    previous = p;
                }
                V::kLineTo => {
                    if !open {
                        return None;
                    }
                    if previous != p {
                        curves.push(Curve::Line([previous, p]));
                    }
                    previous = p;
                }
                V::kQuadraticTo => {
                    if !open {
                        return None;
                    }
                    curves.push(Curve::Quad([previous, point(c.control1), p]));
                    previous = p;
                }
                V::kConicTo => {
                    let w = c.conic_weight as f32;
                    if !open || !w.is_finite() || w <= 0.0 {
                        return None;
                    }
                    curves.push(Curve::Conic([previous, point(c.control1), p], w));
                    previous = p;
                }
                V::kCubicTo => {
                    if !open {
                        return None;
                    }
                    let cubic = [previous, point(c.control1), point(c.control2), p];
                    // SkFindCubicCusp first checks whether the two endpoint
                    // control segments cross. Defer potential cusp contours to
                    // the existing stroker until the cusper circle is translated.
                    let same_side = |test: usize, line: usize| {
                        let vector = cubic[line + 1] - cubic[line];
                        vector.cross(cubic[test] - cubic[line])
                            * vector.cross(cubic[test + 1] - cubic[line])
                            >= 0.0
                    };
                    if cubic[0] != cubic[1]
                        && cubic[2] != cubic[3]
                        && !same_side(0, 2)
                        && !same_side(2, 0)
                    {
                        return None;
                    }
                    curves.push(Curve::Cubic(cubic));
                    previous = p;
                }
                V::kClose => {
                    if !open {
                        return None;
                    }
                    if previous != start {
                        curves.push(Curve::Line([previous, start]));
                    }
                    if curves.is_empty() {
                        return None;
                    }
                    contours.push((std::mem::take(&mut curves), true));
                    open = false;
                }
            }
        }
        if open {
            if curves.is_empty() {
                return None;
            }
            contours.push((curves, false));
        }
        if contours.is_empty() {
            return None;
        }
        let mut result = Vec::new();
        for (curves, closed) in contours {
            let normals: Vec<_> = curves
                .iter()
                .map(|&c| Some((c.eval(0.0).1.length(1.0)?, c.eval(1.0).1.length(1.0)?)))
                .collect::<Option<_>>()?;
            let first = curves[0].ends().0;
            let normal = normals[0].0.normal() * radius;
            let mut outer = vec![command(V::kMoveTo, first + normal, P::default())];
            let mut inner = vec![command(V::kMoveTo, first - normal, P::default())];
            for (i, &curve) in curves.iter().enumerate() {
                if i > 0 {
                    join(
                        &mut outer,
                        &mut inner,
                        normals[i - 1].1.normal(),
                        curve.ends().0,
                        normals[i].0.normal(),
                        radius,
                        miter_limit,
                        join_style,
                        matches!(curves[i - 1], Curve::Line(_)),
                        matches!(curve, Curve::Line(_)),
                    )?;
                }
                if let Curve::Line(p) = curve {
                    let normal = normals[i].0.normal() * radius;
                    outer.push(command(V::kLineTo, p[1] + normal, P::default()));
                    inner.push(command(V::kLineTo, p[1] - normal, P::default()));
                } else {
                    let mut ts = curve.inflections();
                    ts.insert(0, 0.0);
                    ts.push(1.0);
                    for interval in ts.windows(2) {
                        for (side, sink) in [(1.0, &mut outer), (-1.0, &mut inner)] {
                            let mut q =
                                QuadConstruct::new(curve, interval[0], interval[1], radius, side)?;
                            let mut found = false;
                            curve_stroke(
                                curve, &mut q, radius, side, &mut found, 0, tolerance, sink,
                            )?;
                        }
                    }
                }
            }
            if outer
                .iter()
                .chain(inner.iter())
                .any(|c| !point(c.point).finite() || !point(c.control1).finite())
            {
                return None;
            }
            if closed {
                join(
                    &mut outer,
                    &mut inner,
                    normals.last()?.1.normal(),
                    first,
                    normals[0].0.normal(),
                    radius,
                    miter_limit,
                    join_style,
                    matches!(curves.last()?, Curve::Line(_)),
                    matches!(curves[0], Curve::Line(_)),
                )?;
                outer.push(command(V::kClose, P::default(), P::default()));
                result.extend(outer);
                reverse_contour(&inner, &mut result);
            } else {
                let last = curves.last()?.ends().1;
                let end_normal = normals.last()?.1.normal() * radius;
                cap(
                    &mut outer,
                    last,
                    end_normal,
                    point(inner.last()?.point),
                    cap_style,
                );
                let mut reverse = Vec::new();
                reverse_contour(&inner, &mut reverse);
                outer.extend_from_slice(&reverse[1..reverse.len() - 1]);
                let stop = point(outer[0].point);
                cap(&mut outer, first, normal * -1.0, stop, cap_style);
                outer.push(command(V::kClose, P::default(), P::default()));
                result.extend(outer);
            }
        }
        Some(result)
    }
}
