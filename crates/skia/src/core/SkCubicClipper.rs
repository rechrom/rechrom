// CPU implementation moved from renderer/analytic_aa/cubic_clip.rs.

// SkEdgeClipper::clipCubic/clipMonoCubic, SkChopMonoCubicAtX/Y.
// Copyright 2009 The Android Open Source Project (BSD-3-Clause).
// License: ../glyphs/SKIA_LICENSE.
use crate::src::core::SkAnalyticEdge::{Pt, SkAnalyticEdge as AnalyticEdge};
use crate::src::core::SkEdgeBuilder::add_clipped_line;
use crate::src::core::SkEdgeClipper::ClipBox;
use crate::src::core::SkGeometry::chop_y_extrema;
fn coord(p: Pt, axis: usize) -> f32 {
    if axis == 0 {
        p.0
    } else {
        p.1
    }
}
fn chop(p: [Pt; 4], axis: usize, target: f32) -> [Pt; 7] {
    if let Some(t) = crate::src::core::SkGeometry::first_axis_intersection(
        p.map(|p| coord(p, axis) as f64),
        target as f64,
    ) {
        // SkBezierCubic::Subdivide uses doubles, then converts each point to float.
        let p = p.map(|p| (p.0 as f64, p.1 as f64));
        let mix = |a: (f64, f64), b: (f64, f64)| (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
        let a = mix(p[0], p[1]);
        let b = mix(p[1], p[2]);
        let c = mix(p[2], p[3]);
        let ab = mix(a, b);
        let bc = mix(b, c);
        return [p[0], a, ab, mix(ab, bc), bc, c, p[3]].map(|p| (p.0 as f32, p.1 as f32));
    }
    // mono_cubic_closestT / float SkChopCubicAt fallback.
    let [d, b, c, a] = p.map(|p| coord(p, axis));
    let aa = a + 3.0 * (b - c) - d;
    let bb = 3.0 * (c - b - b + d);
    let cc = 3.0 * (b - d);
    let target = target - d;
    let mut t = 0.5;
    let mut step = 0.25;
    let mut best = t;
    let mut closest = f32::MAX;
    loop {
        let loc = ((aa * t + bb) * t + cc) * t;
        let dist = (loc - target).abs();
        if dist < closest {
            closest = dist;
            best = t;
        }
        let last = t;
        t += if loc < target { step } else { -step };
        step *= 0.5;
        if closest <= 0.25 || last == t {
            break;
        }
    }
    let mix = |a: Pt, b: Pt| ((b.0 - a.0) * best + a.0, (b.1 - a.1) * best + a.1);
    let a = mix(p[0], p[1]);
    let b = mix(p[1], p[2]);
    let c = mix(p[2], p[3]);
    let ab = mix(a, b);
    let bc = mix(b, c);
    [p[0], a, ab, mix(ab, bc), bc, c, p[3]]
}
fn add(mut p: [Pt; 4], reverse: bool, out: &mut Vec<AnalyticEdge>) {
    if reverse {
        p.reverse();
    }
    if let Some(e) = AnalyticEdge::setCubic(p) {
        out.push(e);
    }
}
fn mono(mut p: [Pt; 4], c: ClipBox, cull_right: bool, out: &mut Vec<AnalyticEdge>) {
    let mut reverse = p[0].1 > p[3].1;
    if reverse {
        p.reverse();
    }
    if p[3].1 <= c.top || p[0].1 >= c.bottom {
        return;
    }
    if p[0].1 < c.top {
        let mut q = chop(p, 1, c.top);
        if q[3].1 < c.top && q[4].1 < c.top && q[5].1 < c.top {
            q = chop([q[3], q[4], q[5], q[6]], 1, c.top);
        }
        q[3].1 = c.top;
        q[4].1 = q[4].1.max(c.top);
        p[0] = q[3];
        p[1] = q[4];
        p[2] = q[5];
    }
    if p[3].1 > c.bottom {
        let mut q = chop(p, 1, c.bottom);
        q[3].1 = c.bottom;
        q[2].1 = q[2].1.min(c.bottom);
        p[1] = q[1];
        p[2] = q[2];
        p[3] = q[3];
    }
    if p[0].0 > p[3].0 {
        p.reverse();
        reverse = !reverse;
    }
    if p[3].0 <= c.left {
        add_clipped_line((c.left, p[0].1), (c.left, p[3].1), reverse, out);
        return;
    }
    if p[0].0 >= c.right {
        if cull_right {
            return;
        }
        add_clipped_line((c.right, p[0].1), (c.right, p[3].1), reverse, out);
        return;
    }
    if p[0].0 < c.left {
        let mut q = chop(p, 0, c.left);
        add_clipped_line((c.left, q[0].1), (c.left, q[3].1), reverse, out);
        q[3].0 = c.left;
        q[4].0 = q[4].0.max(c.left);
        p[0] = q[3];
        p[1] = q[4];
        p[2] = q[5];
    }
    if p[3].0 > c.right {
        let mut q = chop(p, 0, c.right);
        q[3].0 = c.right;
        q[2].0 = q[2].0.min(c.right);
        add([q[0], q[1], q[2], q[3]], reverse, out);
        add_clipped_line((c.right, q[3].1), (c.right, q[6].1), reverse, out);
    } else {
        add(p, reverse, out);
    }
}
pub(crate) fn clipped(p: [Pt; 4], c: ClipBox, cull_right: bool, out: &mut Vec<AnalyticEdge>) {
    if p.iter().map(|p| p.1).fold(f32::NEG_INFINITY, f32::max) <= c.top
        || p.iter().map(|p| p.1).fold(f32::INFINITY, f32::min) >= c.bottom
    {
        return;
    }
    for y in chop_y_extrema(p) {
        for x in chop_y_extrema(y.map(|p| (p.1, p.0))) {
            mono(x.map(|p| (p.1, p.0)), c, cull_right, out);
        }
    }
}
// SkLineClipper::ClipLine, retaining both side boundaries for winding fill.
pub(crate) fn line(mut p: [Pt; 2], c: ClipBox, cull_right: bool, out: &mut Vec<AnalyticEdge>) {
    let (lo, hi) = if p[0].1 < p[1].1 { (0, 1) } else { (1, 0) };
    if p[hi].1 <= c.top || p[lo].1 >= c.bottom {
        return;
    }
    fn intersect(p: [Pt; 2], axis: usize, target: f32) -> f32 {
        let a = coord(p[0], axis);
        let b = coord(p[1], axis);
        let u = coord(p[0], 1 - axis);
        let v = coord(p[1], 1 - axis);
        if (b - a).abs() <= 1.0 / 4096.0 {
            return (u + v) * 0.5;
        }
        let value =
            u as f64 + (target as f64 - a as f64) * (v as f64 - u as f64) / (b as f64 - a as f64);
        value.clamp(u.min(v) as f64, u.max(v) as f64) as f32
    }
    let original = p;
    if p[lo].1 < c.top {
        p[lo] = (intersect(original, 1, c.top), c.top);
    }
    if p[hi].1 > c.bottom {
        p[hi] = (intersect(original, 1, c.bottom), c.bottom);
    }
    let reverse = original[0].0 >= original[1].0;
    let (lo, hi) = if reverse { (1, 0) } else { (0, 1) };
    if p[hi].0 <= c.left {
        crate::src::core::SkEdgeBuilder::add_line((c.left, p[0].1), (c.left, p[1].1), out);
        return;
    }
    if p[lo].0 >= c.right {
        if cull_right {
            return;
        }
        crate::src::core::SkEdgeBuilder::add_line((c.right, p[0].1), (c.right, p[1].1), out);
        return;
    }
    let mut points = Vec::new();
    if p[lo].0 < c.left {
        points.push((c.left, p[lo].1));
        points.push((c.left, intersect(p, 0, c.left)));
    } else {
        points.push(p[lo]);
    }
    if p[hi].0 > c.right {
        points.push((c.right, intersect(p, 0, c.right)));
        points.push((c.right, p[hi].1));
    } else {
        points.push(p[hi]);
    }
    if reverse {
        points.reverse();
    }
    for pair in points.windows(2) {
        crate::src::core::SkEdgeBuilder::add_line(pair[0], pair[1], out);
    }
}
