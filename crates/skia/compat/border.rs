//! Browser drawing-command adapter for a uniform rounded border.
use crate::compat::commands::{PaintPathCommand, PaintPathVerb as V};
type Pt = (f32, f32);
// skia_renderer.cc::CanStrokeDoubleRoundedRect, then SkRRect::inset.
pub(crate) fn uniform_border_path(
    item: &crate::compat::commands::DrawCommand,
) -> Option<(Vec<PaintPathCommand>, f32)> {
    use crate::compat::commands::Offset;
    let near = |a: f64, b: f64| (a - b).abs() <= 1e-5;
    let w = item.inner_rect.x - item.rect.x;
    if w <= 0.0
        || w > 1.0
        || !near(w, item.inner_rect.y - item.rect.y)
        || !near(
            w,
            item.rect.x + item.rect.width - item.inner_rect.x - item.inner_rect.width,
        )
        || !near(
            w,
            item.rect.y + item.rect.height - item.inner_rect.y - item.inner_rect.height,
        )
    {
        return None;
    }
    let r = item.corner_radii;
    let i = item.inner_corner_radii;
    let radii = [r.top_left, r.top_right, r.bottom_right, r.bottom_left];
    let inner = [i.top_left, i.top_right, i.bottom_right, i.bottom_left];
    for (r, i) in radii.into_iter().zip(inner) {
        if r.x == 0.0 && r.y == 0.0 && i.x == 0.0 && i.y == 0.0 {
            continue;
        }
        if !near(r.x, r.y) || !near(i.x, i.y) || !near(r.x, i.x + w) {
            return None;
        }
    }
    let half = w as f32 * 0.5;
    let x = item.rect.x as f32 + half;
    let y = item.rect.y as f32 + half;
    let right = (item.rect.x + item.rect.width) as f32 - half;
    let bottom = (item.rect.y + item.rect.height) as f32 - half;
    let r = radii.map(|r| (r.x as f32 - half).max(0.0));
    if right <= x
        || bottom <= y
        || r[0] + r[1] > right - x
        || r[2] + r[3] > right - x
        || r[0] + r[3] > bottom - y
        || r[1] + r[2] > bottom - y
    {
        return None;
    }
    let points = [
        (x + r[0], y),
        (right - r[1], y),
        (right, y + r[1]),
        (right, bottom - r[2]),
        (right - r[2], bottom),
        (x + r[3], bottom),
        (x, bottom - r[3]),
        (x, y + r[0]),
    ];
    let controls = [(right, y), (right, bottom), (x, bottom), (x, y)];
    let offset = |p: Pt| Offset {
        x: p.0 as f64,
        y: p.1 as f64,
    };
    // SkPath::RRect defaults to CW start index 6 (left bottom).
    let mut path = vec![PaintPathCommand {
        verb: V::kMoveTo,
        point: offset(points[6]),
        ..Default::default()
    }];
    for j in 0..4 {
        let corner = (3 + j) % 4;
        let k = 2 * corner;
        path.push(PaintPathCommand {
            verb: V::kLineTo,
            point: offset(points[k + 1]),
            ..Default::default()
        });
        path.push(PaintPathCommand {
            verb: V::kConicTo,
            control1: offset(controls[corner]),
            point: offset(points[(k + 2) % 8]),
            conic_weight: std::f32::consts::FRAC_1_SQRT_2 as f64,
            ..Default::default()
        });
    }
    path.push(PaintPathCommand {
        verb: V::kClose,
        ..Default::default()
    });
    Some((path, w as f32))
}
