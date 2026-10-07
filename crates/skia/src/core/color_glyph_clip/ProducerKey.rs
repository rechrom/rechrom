use crate::compat::commands::PaintPathCommand;
use crate::raster::{FillRule, Path, Transform};
use crate::src::core::SkColorGlyphClip::{Bounds, ReceiverKind};
pub(super) fn exact_key(
    path: &Path,
    commands: Option<&[PaintPathCommand]>,
    rule: FillRule,
    do_aa: bool,
    t: Transform,
    width: u32,
    height: u32,
    bounds: Bounds,
    kind: ReceiverKind,
) -> Option<Vec<u64>> {
    use crate::raster::PathSegment as P;
    let segments = path.segments().count();
    let count = 24usize
        .checked_add(segments.checked_mul(7)?)?
        .checked_add(commands.map_or(0, |c| c.len()).checked_mul(8)?)?;
    let mut key = Vec::new();
    key.try_reserve_exact(count).ok()?;
    // Namespace/version,W/H,LTRB,rule,AA,receiver,6localCTMbits,4Pathbounds.
    key.extend([0x434c_5052_4f44_0001, width as u64, height as u64]);
    key.extend(bounds.0.into_iter().map(|v| v as u32 as u64));
    key.extend([
        u64::from(rule == FillRule::EvenOdd),
        u64::from(do_aa),
        match kind {
            ReceiverKind::BwRegion => 0,
            ReceiverKind::AaClip => 1,
        },
    ]);
    key.extend([t.sx, t.sy, t.kx, t.ky, t.tx, t.ty].map(|v| v.to_bits() as u64));
    let b = path.bounds();
    key.extend([b.left(), b.top(), b.right(), b.bottom()].map(|v| v.to_bits() as u64));
    key.push(segments as u64);
    let point = |key: &mut Vec<u64>, p: crate::raster::Point| {
        key.extend([p.x.to_bits() as u64, p.y.to_bits() as u64]);
    };
    for segment in path.segments() {
        match segment {
            P::MoveTo(p) => {
                key.push(0);
                point(&mut key, p);
            }
            P::LineTo(p) => {
                key.push(1);
                point(&mut key, p);
            }
            P::QuadTo(a, p) => {
                key.push(2);
                point(&mut key, a);
                point(&mut key, p);
            }
            P::CubicTo(a, b, p) => {
                key.push(3);
                point(&mut key, a);
                point(&mut key, b);
                point(&mut key, p);
            }
            P::Close => key.push(4),
        }
    }
    match commands {
        None => key.extend([0, 0]),
        Some(commands) => {
            key.extend([1, commands.len() as u64]);
            for c in commands {
                key.push(c.verb as i32 as u32 as u64);
                key.extend(
                    [
                        c.point.x,
                        c.point.y,
                        c.control1.x,
                        c.control1.y,
                        c.control2.x,
                        c.control2.y,
                        c.conic_weight,
                    ]
                    .map(f64::to_bits),
                );
            }
        }
    }
    debug_assert!(key.len() <= count);
    Some(key)
}
