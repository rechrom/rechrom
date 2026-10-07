// Copyright 2008 The Android Open Source Project
// Copyright 2020 Yevhenii Reizner
// BSD-3-Clause; see LICENSE and provenance in translation_map.json.
//! Cap and join construction from src/core/SkStrokerPriv.{h,cpp}.
//! MiterClip is retained as a Rust compatibility extension to the upstream joins.
use super::SkStroke::{CapProc, JoinProc, LineCap, LineJoin, SwappableBuilders};
use crate::path::path_builder::{PathBuilder, PathDirection};
use crate::path::path_geometry;
use crate::path::scalar::{Scalar, SCALAR_NEARLY_ZERO, SCALAR_ROOT_2_OVER_2};
#[cfg(all(not(feature = "std"), feature = "no-std-float"))]
use crate::path::NoStdFloat;
use crate::path::{Point, Transform};
pub(crate) fn CapFactory(cap: LineCap) -> CapProc {
    match cap {
        LineCap::Butt => butt_capper,
        LineCap::Round => round_capper,
        LineCap::Square => square_capper,
    }
}

pub(crate) fn butt_capper(
    _: Point,
    _: Point,
    stop: Point,
    _: Option<&PathBuilder>,
    path: &mut PathBuilder,
) {
    path.line_to(stop.x, stop.y);
}

fn round_capper(
    pivot: Point,
    normal: Point,
    stop: Point,
    _: Option<&PathBuilder>,
    path: &mut PathBuilder,
) {
    let mut parallel = normal;
    parallel.rotate_cw();

    let projected_center = pivot + parallel;

    path.conic_points_to(
        projected_center + normal,
        projected_center,
        SCALAR_ROOT_2_OVER_2,
    );
    path.conic_points_to(projected_center - normal, stop, SCALAR_ROOT_2_OVER_2);
}

fn square_capper(
    pivot: Point,
    normal: Point,
    stop: Point,
    other_path: Option<&PathBuilder>,
    path: &mut PathBuilder,
) {
    let mut parallel = normal;
    parallel.rotate_cw();

    if other_path.is_some() {
        path.set_last_point(Point::from_xy(
            pivot.x + normal.x + parallel.x,
            pivot.y + normal.y + parallel.y,
        ));
        path.line_to(
            pivot.x - normal.x + parallel.x,
            pivot.y - normal.y + parallel.y,
        );
    } else {
        path.line_to(
            pivot.x + normal.x + parallel.x,
            pivot.y + normal.y + parallel.y,
        );
        path.line_to(
            pivot.x - normal.x + parallel.x,
            pivot.y - normal.y + parallel.y,
        );
        path.line_to(stop.x, stop.y);
    }
}

pub(crate) fn JoinFactory(join: LineJoin) -> JoinProc {
    match join {
        LineJoin::Miter => miter_joiner,
        LineJoin::MiterClip => miter_clip_joiner,
        LineJoin::Round => round_joiner,
        LineJoin::Bevel => bevel_joiner,
    }
}

fn is_clockwise(before: Point, after: Point) -> bool {
    before.x * after.y > before.y * after.x
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum AngleType {
    Nearly180,
    Sharp,
    Shallow,
    NearlyLine,
}

fn dot_to_angle_type(dot: f32) -> AngleType {
    if dot >= 0.0 {
        // shallow or line
        if (1.0 - dot).is_nearly_zero() {
            AngleType::NearlyLine
        } else {
            AngleType::Shallow
        }
    } else {
        // sharp or 180
        if (1.0 + dot).is_nearly_zero() {
            AngleType::Nearly180
        } else {
            AngleType::Sharp
        }
    }
}

fn handle_inner_join(pivot: Point, after: Point, inner: &mut PathBuilder) {
    // In the degenerate case that the stroke radius is larger than our segments
    // just connecting the two inner segments may "show through" as a funny
    // diagonal. To pseudo-fix this, we go through the pivot point. This adds
    // an extra point/edge, but I can't see a cheap way to know when this is
    // not needed :(
    inner.line_to(pivot.x, pivot.y);

    inner.line_to(pivot.x - after.x, pivot.y - after.y);
}

fn bevel_joiner(
    before_unit_normal: Point,
    pivot: Point,
    after_unit_normal: Point,
    radius: f32,
    _: f32,
    _: bool,
    _: bool,
    mut builders: SwappableBuilders,
) {
    let mut after = after_unit_normal.scaled(radius);

    if !is_clockwise(before_unit_normal, after_unit_normal) {
        builders.swap();
        after = -after;
    }

    builders.outer.line_to(pivot.x + after.x, pivot.y + after.y);
    handle_inner_join(pivot, after, builders.inner);
}

pub(crate) fn round_joiner(
    before_unit_normal: Point,
    pivot: Point,
    after_unit_normal: Point,
    radius: f32,
    _: f32,
    _: bool,
    _: bool,
    mut builders: SwappableBuilders,
) {
    let dot_prod = before_unit_normal.dot(after_unit_normal);
    let angle_type = dot_to_angle_type(dot_prod);

    if angle_type == AngleType::NearlyLine {
        return;
    }

    let mut before = before_unit_normal;
    let mut after = after_unit_normal;
    let mut dir = PathDirection::CW;

    if !is_clockwise(before, after) {
        builders.swap();
        before = -before;
        after = -after;
        dir = PathDirection::CCW;
    }

    let ts = Transform::from_row(radius, 0.0, 0.0, radius, pivot.x, pivot.y);

    let mut conics = [path_geometry::Conic::default(); 5];
    let conics = path_geometry::Conic::build_unit_arc(before, after, dir, ts, &mut conics);
    if let Some(conics) = conics {
        for conic in conics {
            builders
                .outer
                .conic_points_to(conic.points[1], conic.points[2], conic.weight);
        }

        after.scale(radius);
        handle_inner_join(pivot, after, builders.inner);
    }
}

#[inline]
pub(crate) fn miter_joiner(
    before_unit_normal: Point,
    pivot: Point,
    after_unit_normal: Point,
    radius: f32,
    inv_miter_limit: f32,
    prev_is_line: bool,
    curr_is_line: bool,
    builders: SwappableBuilders,
) {
    miter_joiner_inner(
        before_unit_normal,
        pivot,
        after_unit_normal,
        radius,
        inv_miter_limit,
        false,
        prev_is_line,
        curr_is_line,
        builders,
    );
}

#[inline]
fn miter_clip_joiner(
    before_unit_normal: Point,
    pivot: Point,
    after_unit_normal: Point,
    radius: f32,
    inv_miter_limit: f32,
    prev_is_line: bool,
    curr_is_line: bool,
    builders: SwappableBuilders,
) {
    miter_joiner_inner(
        before_unit_normal,
        pivot,
        after_unit_normal,
        radius,
        inv_miter_limit,
        true,
        prev_is_line,
        curr_is_line,
        builders,
    );
}

fn miter_joiner_inner(
    before_unit_normal: Point,
    pivot: Point,
    after_unit_normal: Point,
    radius: f32,
    inv_miter_limit: f32,
    miter_clip: bool,
    prev_is_line: bool,
    mut curr_is_line: bool,
    mut builders: SwappableBuilders,
) {
    fn do_blunt_or_clipped(
        builders: SwappableBuilders,
        pivot: Point,
        radius: f32,
        prev_is_line: bool,
        curr_is_line: bool,
        mut before: Point,
        mut mid: Point,
        mut after: Point,
        inv_miter_limit: f32,
        miter_clip: bool,
    ) {
        after.scale(radius);

        if miter_clip {
            mid.normalize();

            let cos_beta = before.dot(mid);
            let sin_beta = before.cross(mid);

            let x = if sin_beta.abs() <= SCALAR_NEARLY_ZERO {
                1.0 / inv_miter_limit
            } else {
                ((1.0 / inv_miter_limit) - cos_beta) / sin_beta
            };

            before.scale(radius);

            let mut before_tangent = before;
            before_tangent.rotate_cw();

            let mut after_tangent = after;
            after_tangent.rotate_ccw();

            let c1 = pivot + before + before_tangent.scaled(x);
            let c2 = pivot + after + after_tangent.scaled(x);

            if prev_is_line {
                builders.outer.set_last_point(c1);
            } else {
                builders.outer.line_to(c1.x, c1.y);
            }

            builders.outer.line_to(c2.x, c2.y);
        }

        if !curr_is_line {
            builders.outer.line_to(pivot.x + after.x, pivot.y + after.y);
        }

        handle_inner_join(pivot, after, builders.inner);
    }

    fn do_miter(
        builders: SwappableBuilders,
        pivot: Point,
        radius: f32,
        prev_is_line: bool,
        curr_is_line: bool,
        mid: Point,
        mut after: Point,
    ) {
        after.scale(radius);

        if prev_is_line {
            builders
                .outer
                .set_last_point(Point::from_xy(pivot.x + mid.x, pivot.y + mid.y));
        } else {
            builders.outer.line_to(pivot.x + mid.x, pivot.y + mid.y);
        }

        if !curr_is_line {
            builders.outer.line_to(pivot.x + after.x, pivot.y + after.y);
        }

        handle_inner_join(pivot, after, builders.inner);
    }

    // negate the dot since we're using normals instead of tangents
    let dot_prod = before_unit_normal.dot(after_unit_normal);
    let angle_type = dot_to_angle_type(dot_prod);
    let mut before = before_unit_normal;
    let mut after = after_unit_normal;
    let mut mid;

    if angle_type == AngleType::NearlyLine {
        return;
    }

    if angle_type == AngleType::Nearly180 {
        curr_is_line = false;
        mid = (after - before).scaled(radius / 2.0);
        do_blunt_or_clipped(
            builders,
            pivot,
            radius,
            prev_is_line,
            curr_is_line,
            before,
            mid,
            after,
            inv_miter_limit,
            miter_clip,
        );
        return;
    }

    let ccw = !is_clockwise(before, after);
    if ccw {
        builders.swap();
        before = -before;
        after = -after;
    }

    // Before we enter the world of square-roots and divides,
    // check if we're trying to join an upright right angle
    // (common case for stroking rectangles). If so, special case
    // that (for speed an accuracy).
    // Note: we only need to check one normal if dot==0
    if dot_prod == 0.0 && inv_miter_limit <= SCALAR_ROOT_2_OVER_2 {
        mid = (before + after).scaled(radius);
        do_miter(
            builders,
            pivot,
            radius,
            prev_is_line,
            curr_is_line,
            mid,
            after,
        );
        return;
    }

    // choose the most accurate way to form the initial mid-vector
    if angle_type == AngleType::Sharp {
        mid = Point::from_xy(after.y - before.y, before.x - after.x);
        if ccw {
            mid = -mid;
        }
    } else {
        mid = Point::from_xy(before.x + after.x, before.y + after.y);
    }

    // midLength = radius / sinHalfAngle
    // if (midLength > miterLimit * radius) abort
    // if (radius / sinHalf > miterLimit * radius) abort
    // if (1 / sinHalf > miterLimit) abort
    // if (1 / miterLimit > sinHalf) abort
    // My dotProd is opposite sign, since it is built from normals and not tangents
    // hence 1 + dot instead of 1 - dot in the formula
    let sin_half_angle = (1.0 + dot_prod).half().sqrt();
    if sin_half_angle < inv_miter_limit {
        curr_is_line = false;
        do_blunt_or_clipped(
            builders,
            pivot,
            radius,
            prev_is_line,
            curr_is_line,
            before,
            mid,
            after,
            inv_miter_limit,
            miter_clip,
        );
        return;
    }

    mid.set_length(radius / sin_half_angle);
    do_miter(
        builders,
        pivot,
        radius,
        prev_is_line,
        curr_is_line,
        mid,
        after,
    );
}
