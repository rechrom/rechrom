//! One scan supplies the real mask bytes and its final receiver encoding.
//! AA fallback captures actual sparse SkBlitter events; it never reconstructs
//! AA partitions from a completed mask. Its native equivalence is a test gate.
use super::super::SkColorGlyphClip::{Bounds, BwRegion, CanvasClipOwner, Receiver, ReceiverKind};
use crate::compat::commands::{PaintPathCommand, PaintPathVerb};
use crate::raster::alpha_runs::AlphaRun;
use crate::raster::blitter::{Blitter, Mask as BlitMask};
use crate::raster::geom::{IntRectExt, ScreenIntRect};
use crate::raster::{FillRule, Mask, Path, Transform};
use crate::src::core::SkRasterPipelineBlitter::SkRasterPipelineBlitter;

/// SkPath::RRect default clockwise start6; retain rational conics until the
/// producer maps them to device coordinates, exactly like original SkDraw.
pub(crate) fn rounded_commands(
    bounds: crate::compat::commands::PaintRect,
    radii: crate::compat::commands::PaintCornerRadii,
) -> Option<Vec<PaintPathCommand>> {
    let rect = crate::compat::geometry::rect(bounds)?;
    let (x, y, w, h) = (rect.x(), rect.y(), rect.width(), rect.height());
    let mut r = [
        (radii.top_left.x as f32, radii.top_left.y as f32),
        (radii.top_right.x as f32, radii.top_right.y as f32),
        (radii.bottom_right.x as f32, radii.bottom_right.y as f32),
        (radii.bottom_left.x as f32, radii.bottom_left.y as f32),
    ];
    if r.iter().any(|&(rx, ry)| !rx.is_finite() || !ry.is_finite()) {
        r.fill((0.0, 0.0));
    }
    for (rx, ry) in &mut r {
        if *rx <= 0.0 || *ry <= 0.0 {
            *rx = 0.0;
            *ry = 0.0;
        }
    }
    // The bridge chooses setRectXY only for four identical circular radii.
    // All other corners use setRectRadii's double scale and ULP correction.
    let circular = radii.top_left == radii.top_right
        && radii.top_left == radii.bottom_right
        && radii.top_left == radii.bottom_left
        && radii.top_left.x == radii.top_left.y;
    let sides = [
        (0, 1, 0, f64::from(w)),
        (1, 2, 1, f64::from(h)),
        (2, 3, 0, f64::from(w)),
        (3, 0, 1, f64::from(h)),
    ];
    let get = |r: &[(f32, f32); 4], i: usize, axis: usize| if axis == 0 { r[i].0 } else { r[i].1 };
    let set = |r: &mut [(f32, f32); 4], i: usize, axis: usize, a: f32| {
        if axis == 0 {
            r[i].0 = a;
        } else {
            r[i].1 = a;
        }
    };
    if circular {
        let (mut rx, mut ry) = r[0];
        if w < rx + rx || h < ry + ry {
            let scale = (w / (rx + rx)).min(h / (ry + ry));
            rx *= scale;
            ry *= scale;
        }
        r.fill((rx, ry));
    } else {
        let mut scale = 1.0_f64;
        for &(a, b, axis, limit) in &sides {
            let sum = f64::from(get(&r, a, axis)) + f64::from(get(&r, b, axis));
            if sum > limit {
                scale = scale.min(limit / sum);
            }
        }
        // SkRRect::flush_to_zero precedes AdjustRadii.
        for &(a, b, axis, _) in &sides {
            let (ra, rb) = (get(&r, a, axis), get(&r, b, axis));
            if ra + rb == ra {
                set(&mut r, b, axis, 0.0);
            } else if ra + rb == rb {
                set(&mut r, a, axis, 0.0);
            }
        }
        if scale < 1.0 {
            for &(a, b, axis, limit) in &sides {
                let mut ra = (f64::from(get(&r, a, axis)) * scale) as f32;
                let mut rb = (f64::from(get(&r, b, axis)) * scale) as f32;
                if f64::from(ra + rb) > limit {
                    let (small, large) = if ra > rb {
                        (&mut rb, &mut ra)
                    } else {
                        (&mut ra, &mut rb)
                    };
                    *large = (limit - f64::from(*small)) as f32;
                    while f64::from(*large + *small) > limit {
                        // positive finite nextafterf(value, 0): one ULP down.
                        *large = f32::from_bits(large.to_bits() - 1);
                    }
                }
                set(&mut r, a, axis, ra);
                set(&mut r, b, axis, rb);
            }
        }
        for pair in &mut r {
            if pair.0 <= 0.0 || pair.1 <= 0.0 {
                *pair = (0.0, 0.0);
            }
        }
    }
    // SkRRect validity also constrains radii relative to translated bounds.
    let valid = r.iter().all(|&(rx, ry)| {
        rx >= 0.0
            && rx <= w
            && x + rx <= rect.right()
            && rect.right() - rx >= x
            && ry >= 0.0
            && ry <= h
            && y + ry <= rect.bottom()
            && rect.bottom() - ry >= y
    });
    if !valid {
        r.fill((0.0, 0.0));
    }
    if r.iter().all(|&(rx, ry)| rx == 0.0 || ry == 0.0) {
        // SimplifyRRect(start6) => rect start3, with no degenerate conics.
        let offset = |x, y| crate::compat::commands::Offset {
            x: f64::from(x),
            y: f64::from(y),
        };
        let mut commands: Vec<_> = [
            (x, rect.bottom()),
            (x, y),
            (rect.right(), y),
            (rect.right(), rect.bottom()),
        ]
        .into_iter()
        .enumerate()
        .map(|(i, (x, y))| PaintPathCommand {
            verb: if i == 0 {
                PaintPathVerb::kMoveTo
            } else {
                PaintPathVerb::kLineTo
            },
            point: offset(x, y),
            ..Default::default()
        })
        .collect();
        commands.push(PaintPathCommand {
            verb: PaintPathVerb::kClose,
            ..Default::default()
        });
        return Some(commands);
    }
    if r.iter().all(|&p| p == r[0]) && r[0].0 >= w / 2.0 && r[0].1 >= h / 2.0 {
        // SimplifyRRect(start6) => oval start3. Keep four conics only.
        let (cx, cy) = ((x + rect.right()) * 0.5, (y + rect.bottom()) * 0.5);
        let point = |x, y| crate::compat::commands::Offset {
            x: f64::from(x),
            y: f64::from(y),
        };
        let mut commands = vec![PaintPathCommand {
            verb: PaintPathVerb::kMoveTo,
            point: point(x, cy),
            ..Default::default()
        }];
        for (control, end) in [
            ((x, y), (cx, y)),
            ((rect.right(), y), (rect.right(), cy)),
            ((rect.right(), rect.bottom()), (cx, rect.bottom())),
            ((x, rect.bottom()), (x, cy)),
        ] {
            commands.push(PaintPathCommand {
                verb: PaintPathVerb::kConicTo,
                control1: point(control.0, control.1),
                point: point(end.0, end.1),
                conic_weight: f64::from(std::f32::consts::FRAC_1_SQRT_2),
                ..Default::default()
            });
        }
        commands.push(PaintPathCommand {
            verb: PaintPathVerb::kClose,
            ..Default::default()
        });
        return Some(commands);
    }
    let points = [
        (x + r[0].0, y),
        (x + w - r[1].0, y),
        (x + w, y + r[1].1),
        (x + w, y + h - r[2].1),
        (x + w - r[2].0, y + h),
        (x + r[3].0, y + h),
        (x, y + h - r[3].1),
        (x, y + r[0].1),
    ];
    let controls = [(x, y), (x + w, y), (x + w, y + h), (x, y + h)];
    let offset = |(x, y): (f32, f32)| crate::compat::commands::Offset {
        x: f64::from(x),
        y: f64::from(y),
    };
    let mut output = Vec::with_capacity(10);
    output.push(PaintPathCommand {
        verb: PaintPathVerb::kMoveTo,
        point: offset(points[6]),
        ..Default::default()
    });
    for i in [7, 1, 3, 5] {
        output.push(PaintPathCommand {
            verb: PaintPathVerb::kLineTo,
            point: offset(points[i]),
            ..Default::default()
        });
        let corner = ((i + 1) % 8) / 2;
        output.push(PaintPathCommand {
            verb: PaintPathVerb::kConicTo,
            point: offset(points[(i + 1) % 8]),
            control1: offset(controls[corner]),
            conic_weight: f64::from(std::f32::consts::FRAC_1_SQRT_2),
            ..Default::default()
        });
    }
    output.push(PaintPathCommand {
        verb: PaintPathVerb::kClose,
        ..Default::default()
    });
    Some(output)
}

pub(crate) struct PathProduct {
    pub(crate) mask: Mask,
    pub(crate) owner: CanvasClipOwner,
}
fn empty(kind: ReceiverKind, width: u32, height: u32) -> PathProduct {
    PathProduct {
        mask: Mask::new(width, height).expect("valid clip mask"),
        owner: match kind {
            ReceiverKind::AaClip => CanvasClipOwner::Aa(None),
            ReceiverKind::BwRegion => CanvasClipOwner::Bw(BwRegion::from_bw_rects(&[])),
        },
    }
}
fn map_commands(commands: &[PaintPathCommand], t: Transform) -> Vec<PaintPathCommand> {
    let map = |p: crate::compat::commands::Offset| {
        let (x, y) = (p.x as f32, p.y as f32);
        crate::compat::commands::Offset {
            x: f64::from((x * t.sx + y * t.kx) + t.tx),
            y: f64::from((y * t.sy + x * t.ky) + t.ty),
        }
    };
    commands
        .iter()
        .map(|c| {
            let mut c = c.clone();
            c.point = map(c.point);
            if matches!(
                c.verb,
                PaintPathVerb::kQuadraticTo | PaintPathVerb::kConicTo | PaintPathVerb::kCubicTo
            ) {
                c.control1 = map(c.control1);
            }
            if c.verb == PaintPathVerb::kCubicTo {
                c.control2 = map(c.control2);
            }
            c
        })
        .collect()
}
pub(crate) fn produce_path(
    path: &Path,
    source_commands: Option<&[PaintPathCommand]>,
    rule: FillRule,
    do_aa: bool,
    transform: Transform,
    width: u32,
    height: u32,
    bounds: Bounds,
    kind: ReceiverKind,
) -> PathProduct {
    let owned;
    let commands = if let Some(c) = source_commands {
        c
    } else {
        owned = crate::compat::geometry::commands_from_path(path);
        &owned
    };
    // AntiFillPath tests clippedIR = roundOut(devPath.bounds) ∩ clip,
    // rather than the whole clip/device. A wide device still antialiases a
    // small in-range path. Include raw conic controls, before decomposition.
    let map_point = |p: crate::compat::commands::Offset| {
        let (x, y) = (p.x as f32, p.y as f32);
        (
            (x * transform.sx + y * transform.kx) + transform.tx,
            (y * transform.sy + x * transform.ky) + transform.ty,
        )
    };
    let mut extent = [
        f32::INFINITY,
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::NEG_INFINITY,
    ];
    let mut include = |p| {
        let (x, y) = map_point(p);
        extent[0] = extent[0].min(x);
        extent[1] = extent[1].min(y);
        extent[2] = extent[2].max(x);
        extent[3] = extent[3].max(y);
    };
    for c in commands {
        if c.verb != PaintPathVerb::kClose {
            include(c.point);
        }
        if matches!(
            c.verb,
            PaintPathVerb::kQuadraticTo | PaintPathVerb::kConicTo | PaintPathVerb::kCubicTo
        ) {
            include(c.control1);
        }
        if c.verb == PaintPathVerb::kCubicTo {
            include(c.control2);
        }
    }
    let clipped_ir = Bounds([
        bounds.0[0].max(extent[0].floor() as i32),
        bounds.0[1].max(extent[1].floor() as i32),
        bounds.0[2].min(extent[2].ceil() as i32),
        bounds.0[3].min(extent[3].ceil() as i32),
    ]);
    let native_aa_range = clipped_ir.0.iter().all(|&v| (0..8192).contains(&v));
    if kind == ReceiverKind::AaClip && do_aa && native_aa_range {
        if let Some((mask, encoding)) = crate::compat::analytic_masks::path_clip_mask_with_encoding(
            commands,
            rule == FillRule::EvenOdd,
            transform,
            width,
            height,
            (0, 0),
            bounds,
        ) {
            return PathProduct {
                mask: mask.mask,
                owner: CanvasClipOwner::Aa(encoding),
            };
        }
    }
    // Original conic decomposition belongs after mapping to device space.
    // Retain raw commands when available rather than approximating a conic
    // in source coordinates and scaling its already chopped quadratics.
    let mapped = map_commands(commands, transform);
    let Some(device) = crate::compat::geometry::path_from_commands(&mapped) else {
        return empty(kind, width, height);
    };
    if crate::raster::painter::is_too_big_for_math(&device) {
        return empty(kind, width, height); // same existing mask scanner rejection
    }
    let snug = if kind == ReceiverKind::AaClip {
        clipped_ir
    } else {
        bounds
    };
    let Some(rect) = crate::raster::IntRect::from_ltrb(snug.0[0], snug.0[1], snug.0[2], snug.0[3])
    else {
        return empty(kind, width, height);
    };
    let Some(clip) = rect.to_screen_int_rect() else {
        return empty(kind, width, height);
    };
    let mut mask = Mask::new(width, height).expect("valid clip mask");
    let owner = {
        let mut pixels = mask.as_subpixmap();
        let mut real = SkRasterPipelineBlitter::new_mask(&mut pixels).expect("valid mask blitter");
        let capture = match kind {
            ReceiverKind::AaClip => Capture::Aa(Receiver::new(snug)),
            ReceiverKind::BwRegion => Capture::Bw(Vec::new()),
        };
        let mut target = CaptureBlitter {
            real: &mut real,
            capture,
        };
        if do_aa && native_aa_range {
            crate::raster::scan::path_aa::fill_path(&device, rule, &clip, &mut target);
        } else {
            // Original AntiFillPath clippedIR short overflow falls to BW
            // while preserving the caller's actual AA Builder receiver.
            crate::raster::scan::path::fill_path(&device, rule, &clip, &mut target);
        }
        target.finish()
    };
    PathProduct { mask, owner }
}
enum Capture {
    Aa(Receiver),
    Bw(Vec<Bounds>),
}
struct CaptureBlitter<'a> {
    real: &'a mut dyn Blitter,
    capture: Capture,
}
fn append_bw_rect(rects: &mut Vec<Bounds>, rect: Bounds) {
    // Consecutive opaque rectangles with identical X and touching Y have
    // exactly the same SkRegion union. General overlap/order still uses the
    // canonical BW builder. Never called on the AA receiver.
    if let Some(last) = rects
        .last_mut()
        .filter(|r| r.0[0] == rect.0[0] && r.0[2] == rect.0[2] && r.0[3] == rect.0[1])
    {
        last.0[3] = rect.0[3];
    } else {
        rects.push(rect);
    }
}
impl CaptureBlitter<'_> {
    fn finish(self) -> CanvasClipOwner {
        match self.capture {
            Capture::Aa(r) => CanvasClipOwner::Aa(r.finish()),
            Capture::Bw(rects) => CanvasClipOwner::Bw(BwRegion::from_bw_rects(&rects)),
        }
    }
    fn h(&mut self, x: u32, y: u32, w: u32) {
        match &mut self.capture {
            Capture::Aa(r) => r.h(x as i32, y as i32, w as i32),
            Capture::Bw(rects) => append_bw_rect(
                rects,
                Bounds([x as i32, y as i32, (x + w) as i32, (y + 1) as i32]),
            ),
        }
    }
    fn anti_h(&mut self, x: u32, y: u32, spans: &[(i32, u8)]) {
        self.anti_h_iter(x, y, spans.iter().copied());
    }
    fn anti_h_iter(&mut self, x: u32, y: u32, spans: impl IntoIterator<Item = (i32, u8)>) {
        match &mut self.capture {
            Capture::Aa(r) => r.anti_h_iter(x as i32, y as i32, spans),
            Capture::Bw(_) => {
                let mut x = x;
                for (w, a) in spans {
                    debug_assert!(a == 0 || a == 255);
                    if a == 255 {
                        self.h(x, y, w as u32);
                    }
                    x += w as u32;
                }
            }
        }
    }
}
impl Blitter for CaptureBlitter<'_> {
    fn blit_h(&mut self, x: u32, y: u32, w: core::num::NonZeroU32) {
        self.h(x, y, w.get());
        self.real.blit_h(x, y, w);
    }
    fn blit_anti_h(&mut self, x: u32, y: u32, aa: &mut [u8], runs: &mut [AlphaRun]) {
        let mut offset = 0;
        let spans = std::iter::from_fn(|| {
            let run = runs[offset]?;
            let span = (i32::from(run.get()), aa[offset]);
            offset += usize::from(run.get());
            Some(span)
        });
        self.anti_h_iter(x, y, spans);
        self.real.blit_anti_h(x, y, aa, runs);
    }
    fn blit_rect(&mut self, b: &ScreenIntRect) {
        match &mut self.capture {
            Capture::Aa(r) => r.rect(
                b.x() as i32,
                b.y() as i32,
                b.width() as i32,
                b.height() as i32,
            ),
            Capture::Bw(rects) => append_bw_rect(
                rects,
                Bounds([
                    b.x() as i32,
                    b.y() as i32,
                    b.right() as i32,
                    b.bottom() as i32,
                ]),
            ),
        }
        self.real.blit_rect(b);
    }
    fn blit_v(&mut self, x: u32, y: u32, height: core::num::NonZeroU32, alpha: u8) {
        match &mut self.capture {
            Capture::Aa(r) => r.v(x as i32, y as i32, height.get() as i32, alpha),
            Capture::Bw(_) => {
                for y in y..y + height.get() {
                    if alpha != 0 {
                        debug_assert_eq!(alpha, 255);
                        self.h(x, y, 1);
                    }
                }
            }
        }
        self.real.blit_v(x, y, height, alpha);
    }
    fn blit_anti_h2(&mut self, x: u32, y: u32, a: u8, b: u8) {
        self.anti_h(x, y, &[(1, a), (1, b)]);
        self.real.blit_anti_h2(x, y, a, b);
    }
    fn blit_anti_v2(&mut self, x: u32, y: u32, a: u8, b: u8) {
        self.anti_h(x, y, &[(1, a)]);
        self.anti_h(x, y + 1, &[(1, b)]);
        self.real.blit_anti_v2(x, y, a, b);
    }
    fn blit_mask(&mut self, mask: &BlitMask, clip: &ScreenIntRect) {
        // Current source Mask is a two-pixel borrowed primitive, not a
        // completed raster frame. Preserve each actual primitive row.
        for y in clip.y()..clip.bottom() {
            let spans = (clip.x()..clip.right()).map(|x| {
                let i = (y - mask.bounds.y()) as usize * mask.row_bytes as usize
                    + (x - mask.bounds.x()) as usize;
                (1, mask.image[i])
            });
            self.anti_h_iter(clip.x(), y, spans);
        }
        self.real.blit_mask(mask, clip);
    }
}

#[path = "ProducerCache.rs"]
pub(crate) mod cache;
