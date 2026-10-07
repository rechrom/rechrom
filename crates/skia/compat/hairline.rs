//! Local surface, transform, and command adapters for Skia AA hairline scanning.
use crate::compat::commands::{Color, PaintPathCommand, PaintPathVerb as V, SvgStrokeLineCap};
use crate::raster::{Mask, Pixmap, Transform};
use crate::src::core::SkAnalyticEdge::Pt;
use crate::src::core::SkScan_Antihair::{HairlineClip as Clip, HairlineCoverageBlitter};
use crate::src::core::SkScan_Hairline::{cubic_points, quad_points};
pub(crate) fn bounds(points: &[Pt]) -> [f32; 4] {
    points.iter().fold(
        [
            f32::INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
        ],
        |a, p| [a[0].min(p.0), a[1].min(p.1), a[2].max(p.0), a[3].max(p.1)],
    )
}

pub(crate) fn draw_with_surface(
    pixmap: &mut Pixmap,
    surface: Option<&mut crate::cpu::f16_surface::Surface>,
    mask: Option<&Mask>,
    aa_clip: bool,
    transform: Transform,
    path: &[PaintPathCommand],
    color: Color,
    width: f32,
) -> bool {
    draw_stroke_with_surface(
        pixmap,
        surface,
        mask,
        aa_clip,
        transform,
        path,
        color,
        width,
        SvgStrokeLineCap::kButt,
    )
}

pub(crate) fn draw_with_clip(
    pixmap: &mut Pixmap,
    surface: Option<&mut crate::cpu::f16_surface::Surface>,
    mask: Option<&Mask>,
    aa_clip: bool,
    raster_clip: Option<&crate::src::core::SkRasterClip::SkRasterClip>,
    transform: Transform,
    path: &[PaintPathCommand],
    color: Color,
    width: f32,
) -> bool {
    draw_stroke_with_clip(
        pixmap,
        surface,
        mask,
        aa_clip,
        raster_clip,
        transform,
        path,
        color,
        width,
        SvgStrokeLineCap::kButt,
    )
}

pub(crate) fn draw_stroke_with_surface(
    pixmap: &mut Pixmap,
    surface: Option<&mut crate::cpu::f16_surface::Surface>,
    mask: Option<&Mask>,
    aa_clip: bool,
    transform: Transform,
    path: &[PaintPathCommand],
    color: Color,
    width: f32,
    cap: SvgStrokeLineCap,
) -> bool {
    draw_stroke_with_clip(
        pixmap, surface, mask, aa_clip, None, transform, path, color, width, cap,
    )
}

// SkDraw::DrawTreatAAStrokeAsHairline evaluates the transformed width before asking
// the mask-only hairline blitter to materialize a BW clip. Wider strokes use
// their existing analytic outline path and can retain the rectangular region.
pub(crate) fn device_hairline_width(transform: Transform, width: f32) -> Option<f32> {
    if width < 0.0 || !width.is_finite() {
        return None;
    }
    if width == 0.0 {
        return Some(0.0);
    }
    let fast_len = |x: f32, y: f32| x.abs().max(y.abs()) + x.abs().min(y.abs()) * 0.5;
    let a = fast_len(transform.sx * width, transform.ky * width);
    let b = fast_len(transform.kx * width, transform.sy * width);
    if a > 1.0 || b > 1.0 || !a.is_finite() || !b.is_finite() {
        None
    } else {
        Some((a + b) * 0.5)
    }
}

pub(crate) fn draw_stroke_with_clip(
    pixmap: &mut Pixmap,
    mut surface: Option<&mut crate::cpu::f16_surface::Surface>,
    mask: Option<&Mask>,
    aa_clip: bool,
    raster_clip: Option<&crate::src::core::SkRasterClip::SkRasterClip>,
    transform: Transform,
    path: &[PaintPathCommand],
    mut color: Color,
    width: f32,
    cap: SvgStrokeLineCap,
) -> bool {
    let Some(width) = device_hairline_width(transform, width) else {
        return false;
    };
    if cap != SvgStrokeLineCap::kButt
        && path
            .iter()
            .any(|c| !matches!(c.verb, V::kMoveTo | V::kLineTo | V::kClose))
    {
        return false;
    }

    if width > 0.0 && width < 1.0 {
        let alpha = (color.alpha * 255.0 + 0.5).floor() as u32;
        let scale = (width * 256.0) as u32;
        color.alpha = ((alpha * scale) >> 8) as f32 / 255.0;
    }
    if path.iter().any(|cmd| {
        [cmd.point, cmd.control1, cmd.control2].iter().any(|p| {
            !p.x.is_finite() || !p.y.is_finite() || p.x.abs() >= 32760.0 || p.y.abs() >= 32760.0
        })
    }) {
        return false;
    }
    let sw = pixmap.width();
    let sh = pixmap.height();
    let mut bbox = Clip {
        l: 0,
        t: 0,
        r: sw as i32,
        b: sh as i32,
    };
    // SkRasterClip already carries the exact nonzero bounds. BW clips
    // are rectangles, so their per-pixel alpha is uniformly full inside it.
    let mask = mask.filter(|_| !raster_clip.is_some_and(|c| c.isRect()));
    if let Some(clip) = raster_clip {
        let Some(bounds) = clip.getBounds() else {
            return true;
        };
        bbox = Clip {
            l: bounds.left(),
            t: bounds.top(),
            r: bounds.right(),
            b: bounds.bottom(),
        };
    } else if let Some(m) = mask {
        bbox = Clip {
            l: sw as i32,
            t: sh as i32,
            r: 0,
            b: 0,
        };
        let storage = m.storage_bounds();
        for yy in storage.top() as u32..storage.bottom() as u32 {
            for (xx, &a) in m
                .row_range(yy, storage.left() as u32, storage.right() as u32)
                .iter()
                .enumerate()
            {
                let i = yy as usize * sw as usize + storage.left() as usize + xx;
                if a > 0 {
                    let x = (i % sw as usize) as i32;
                    let y = (i / sw as usize) as i32;
                    bbox.l = bbox.l.min(x);
                    bbox.t = bbox.t.min(y);
                    bbox.r = bbox.r.max(x + 1);
                    bbox.b = bbox.b.max(y + 1);
                }
            }
        }
    }
    // Prepare every tile before blending, preserving the source transform and
    // local conic subdivision. No reference pixels or native renderer calls.
    let mut tiles = Vec::new();
    let mut oy = 0;
    while oy < sh {
        let ly = u32::from(oy != 0);
        let ny = (255 - ly).min(sh - oy);
        let sy = oy - ly;
        let th = ly + ny + u32::from(oy + ny < sh);
        let mut ox = 0;
        while ox < sw {
            let lx = u32::from(ox != 0);
            let nx = (255 - lx).min(sw - ox);
            let sx = ox - lx;
            let tw = lx + nx + u32::from(ox + nx < sw);
            let c = Clip {
                l: (bbox.l - sx as i32).max(0),
                t: (bbox.t - sy as i32).max(0),
                r: (bbox.r - sx as i32).min(tw as i32),
                b: (bbox.b - sy as i32).min(th as i32),
            };
            if c.l < c.r && c.t < c.b {
                let map = |v: crate::compat::commands::Offset| {
                    (
                        (transform.sx * v.x as f32 + transform.kx * v.y as f32)
                            + (transform.tx - sx as f32),
                        (transform.ky * v.x as f32 + transform.sy * v.y as f32)
                            + (transform.ty - sy as f32),
                    )
                };
                let mut polylines = Vec::new();
                let mut prev = (0.0, 0.0);
                let mut start = prev;
                let mut closed = false;
                for (index, cmd) in path.iter().enumerate() {
                    let p = map(cmd.point);
                    if ![p.0, p.1]
                        .iter()
                        .all(|v| v.is_finite() && v.abs() < 32760.0)
                    {
                        return false;
                    }
                    match cmd.verb {
                        V::kMoveTo => {
                            start = p;
                            prev = p;
                            closed = path[index + 1..]
                                .iter()
                                .take_while(|c| c.verb != V::kMoveTo)
                                .any(|c| c.verb == V::kClose);
                            continue;
                        }
                        V::kLineTo => {
                            let mut points = [prev, p];
                            if cap != SvgStrokeLineCap::kButt && (!closed || prev == p) {
                                let outset = if cap == SvgStrokeLineCap::kSquare {
                                    0.5
                                } else {
                                    std::f32::consts::PI / 8.0
                                };
                                let normalize = |x: f32, y: f32, degenerate: f32| {
                                    if x == 0.0 && y == 0.0 {
                                        (degenerate, 0.0)
                                    } else {
                                        let scale = 1.0
                                            / ((x as f64) * (x as f64) + (y as f64) * (y as f64))
                                                .sqrt();
                                        ((x as f64 * scale) as f32, (y as f64 * scale) as f32)
                                    }
                                };
                                if index > 0 && path[index - 1].verb == V::kMoveTo {
                                    let t = normalize(prev.0 - p.0, prev.1 - p.1, 1.0);
                                    points[0] = (prev.0 + t.0 * outset, prev.1 + t.1 * outset);
                                }
                                if path
                                    .get(index + 1)
                                    .is_none_or(|c| matches!(c.verb, V::kMoveTo | V::kClose))
                                {
                                    let t = normalize(p.0 - points[0].0, p.1 - points[0].1, -1.0);
                                    points[1] = (p.0 + t.0 * outset, p.1 + t.1 * outset);
                                }
                            }
                            polylines.push(points.to_vec());
                        }
                        V::kQuadraticTo => {
                            polylines.push(quad_points([prev, map(cmd.control1), p]))
                        }
                        V::kConicTo => {
                            let w = cmd.conic_weight as f32;
                            if !w.is_finite() || w <= 0.0 {
                                return false;
                            }
                            for q in crate::src::core::SkGeometry::conic_to_quads(
                                [prev, map(cmd.control1), p],
                                w,
                            ) {
                                polylines.push(quad_points(q));
                            }
                        }
                        V::kCubicTo => {
                            let Some(q) =
                                cubic_points([prev, map(cmd.control1), map(cmd.control2), p])
                            else {
                                return false;
                            };
                            polylines.push(q);
                        }
                        V::kClose => {
                            polylines.push(vec![prev, start]);
                            prev = start;
                            continue;
                        }
                    }
                    prev = p;
                }
                tiles.push((ox, oy, nx, ny, sx, sy, c, polylines));
            }
            ox += nx;
        }
        oy += ny;
    }
    for (ox, oy, nx, ny, sx, sy, clip, lines) in tiles {
        let mut b = HairlineCoverageBlitter {
            pixmap,
            surface: surface.as_deref_mut(),
            mask,
            color,
            aa_clip,
            clip,
            ox,
            oy,
            nx,
            ny,
            sx,
            sy,
        };
        for line in lines {
            for points in line.windows(2) {
                b.line([points[0], points[1]]);
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compat::commands::Offset;
    use crate::raster::{IntRect, PixelFormat};
    use crate::src::core::SkRasterClip::SkRasterClip;

    #[test]
    fn raster_clip_bounds_match_dense_hairline_mask() {
        for (w, h) in [(41u32, 33u32), (260, 258)] {
            let bounds = IntRect::from_ltrb(7, 5, w as i32 - 8, h as i32 - 6).unwrap();
            for aa in [false, true] {
                let mut mask = Mask::new(w, h).unwrap();
                for y in bounds.top()..bounds.bottom() {
                    for x in bounds.left()..bounds.right() {
                        mask.data_mut()[y as usize * w as usize + x as usize] =
                            if aa && (x == bounds.left() || y == bounds.top()) {
                                123
                            } else {
                                255
                            };
                    }
                }
                let clip =
                    SkRasterClip::from_mask_with_bounds(w, h, mask.data(), Some(bounds)).unwrap();
                let p = |x, y| Offset { x, y };
                let paths = [
                    vec![
                        PaintPathCommand {
                            point: p(-4.0, 8.5),
                            ..Default::default()
                        },
                        PaintPathCommand {
                            verb: V::kLineTo,
                            point: p(w as f64 + 2.0, h as f64 - 8.25),
                            ..Default::default()
                        },
                    ],
                    vec![
                        PaintPathCommand {
                            point: p(4.0, 4.0),
                            ..Default::default()
                        },
                        PaintPathCommand {
                            verb: V::kCubicTo,
                            control1: p(38.5, -7.0),
                            control2: p(-5.0, h as f64),
                            point: p(w as f64 - 2.0, h as f64 - 3.0),
                            ..Default::default()
                        },
                    ],
                ];
                for format in [
                    PixelFormat::Rgba8888,
                    PixelFormat::Bgra8888,
                    PixelFormat::Bgrx8888,
                ] {
                    for width in [0.0, 0.25, 0.75, 1.0, 2.0] {
                        for path in &paths {
                            let mut old = Pixmap::new(w, h).unwrap();
                            let mut new = Pixmap::new(w, h).unwrap();
                            old.format = format;
                            new.format = format;
                            old.data_mut().fill(31);
                            new.data_mut().fill(31);
                            let color = Color {
                                red: 0.8,
                                green: 0.2,
                                blue: 0.7,
                                alpha: 0.6,
                            };
                            let expected = draw_stroke_with_surface(
                                &mut old,
                                None,
                                Some(&mask),
                                aa,
                                Transform::identity(),
                                path,
                                color,
                                width,
                                SvgStrokeLineCap::kButt,
                            );
                            let actual = draw_stroke_with_clip(
                                &mut new,
                                None,
                                if aa { Some(&mask) } else { None },
                                aa,
                                Some(&clip),
                                Transform::identity(),
                                path,
                                color,
                                width,
                                SvgStrokeLineCap::kButt,
                            );
                            assert_eq!(actual, expected);
                            assert_eq!(
                                new.data(),
                                old.data(),
                                "{w}x{h} aa={aa} format={format:?} width={width}"
                            );
                        }
                    }
                }
            }
        }
    }
}
