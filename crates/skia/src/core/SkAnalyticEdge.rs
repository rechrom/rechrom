//! Analytic line, quadratic and cubic edges.
//! Source: src/core/SkAnalyticEdge.h/.cpp; tagged curve storage is a Rust representation.
const ONE: i32 = 65536;
pub(crate) type Pt = (f32, f32);
fn mul(a: i32, b: i32) -> i32 {
    (i64::from(a) * i64::from(b) >> 16) as i32
}
fn ceil(v: i32) -> i32 {
    (v + ONE - 1) >> 16
}
pub(crate) fn snap_y(y: i32) -> i32 {
    (y + 8192) & !16383
}
fn inverse(v: i32) -> i32 {
    // Skia's quick_inverse table has a zero entry. A nonzero curve slope
    // can truncate to zero when converted from Fixed to FDot6 in updateLine.
    if v == 0 {
        0
    } else {
        4_194_304 / v
    }
}
fn quick_div(a: i32, b: i32) -> i32 {
    if b.abs() >= 8 && b.abs() < 1024 && a.abs() < 4096 {
        (a * inverse(b)) >> 6
    } else {
        ((i64::from(a) << 16) / i64::from(b)).clamp(i32::MIN as i64, i32::MAX as i64) as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_tile_shift_preserves_all_line_quad_cubic_segments_and_derivatives() {
        let cases = [
            SkAnalyticEdge::setLine((2499.125, 600.25), (2534.875, 850.75)).unwrap(),
            SkAnalyticEdge::setQuadratic([
                (2499.125, 600.25),
                (2580.375, 730.5),
                (2534.875, 850.75),
            ])
            .unwrap(),
            SkAnalyticEdge::setCubic([
                (2499.125, 600.25),
                (2580.375, 655.5),
                (2477.625, 810.25),
                (2534.875, 850.75),
            ])
            .unwrap(),
        ];
        for source in cases {
            for (tx, ty) in [(-254, -254), (-1016, -508), (-2286, -762), (0, 0)] {
                let (dx, dy) = (tx << 16, ty << 16);
                let mut global = source.clone();
                let mut tile = source.clone();
                tile.translate_fixed(dx, dy);
                loop {
                    assert_eq!(
                        (tile.x, tile.upper_x, tile.y, tile.upper_y, tile.lower_y),
                        (
                            global.x + dx,
                            global.upper_x + dx,
                            global.y + dy,
                            global.upper_y + dy,
                            global.lower_y + dy
                        )
                    );
                    assert_eq!(
                        (tile.dx, tile.dy, tile.winding),
                        (global.dx, global.dy, global.winding)
                    );
                    if let (Some(a), Some(b)) = (&global.quad, &tile.quad) {
                        assert_eq!(
                            (b.x, b.y, b.last_x, b.last_y, b.snapped_x, b.snapped_y),
                            (
                                a.x + dx,
                                a.y + dy,
                                a.last_x + dx,
                                a.last_y + dy,
                                a.snapped_x + dx,
                                a.snapped_y + dy
                            )
                        );
                        assert_eq!(
                            (b.dx, b.dy, b.ddx, b.ddy, b.shift, b.count),
                            (a.dx, a.dy, a.ddx, a.ddy, a.shift, a.count)
                        );
                    }
                    if let (Some(a), Some(b)) = (&global.cubic, &tile.cubic) {
                        assert_eq!(
                            (b.x, b.y, b.last_x, b.last_y, b.snapped_y),
                            (
                                a.x + dx,
                                a.y + dy,
                                a.last_x + dx,
                                a.last_y + dy,
                                a.snapped_y + dy
                            )
                        );
                        assert_eq!(
                            (
                                b.dx,
                                b.dy,
                                b.ddx,
                                b.ddy,
                                b.dddx,
                                b.dddy,
                                b.shift,
                                b.fixed_shift,
                                b.count
                            ),
                            (
                                a.dx,
                                a.dy,
                                a.ddx,
                                a.ddy,
                                a.dddx,
                                a.dddy,
                                a.shift,
                                a.fixed_shift,
                                a.count
                            )
                        );
                    }
                    let a = global.update();
                    let b = tile.update();
                    assert_eq!(a, b);
                    if !a {
                        break;
                    }
                }
            }
        }
    }

    #[test]
    fn nearly_vertical_curves_use_skia_zero_inverse_entry() {
        let mut quadratic =
            SkAnalyticEdge::setQuadratic([(0.0, 0.0), (0.125, 50.0), (0.25, 100.0)]).unwrap();
        let mut cubic =
            SkAnalyticEdge::setCubic([(0.0, 0.0), (0.125, 32.0), (0.25, 64.0), (0.375, 100.0)])
                .unwrap();
        for edge in [&mut quadratic, &mut cubic] {
            let mut zero_inverse_segments = 0;
            loop {
                if edge.dx != 0 && (edge.dx >> 10) == 0 {
                    assert_eq!(edge.dy, 0);
                    zero_inverse_segments += 1;
                }
                if !edge.update() {
                    break;
                }
            }
            assert!(zero_inverse_segments > 0);
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct SkAnalyticQuadraticEdge {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) dx: i32,
    pub(crate) dy: i32,
    pub(crate) ddx: i32,
    pub(crate) ddy: i32,
    pub(crate) last_x: i32,
    pub(crate) last_y: i32,
    pub(crate) snapped_x: i32,
    pub(crate) snapped_y: i32,
    pub(crate) shift: u32,
    pub(crate) count: i32,
}
#[derive(Clone)]
pub(crate) struct SkAnalyticEdge {
    pub(crate) x: i32,
    pub(crate) dx: i32,
    pub(crate) upper_x: i32,
    pub(crate) y: i32,
    pub(crate) upper_y: i32,
    pub(crate) lower_y: i32,
    pub(crate) dy: i32,
    pub(crate) quad: Option<SkAnalyticQuadraticEdge>,
    pub(crate) cubic: Option<SkAnalyticCubicEdge>,
    pub(crate) winding: i8,
}
impl SkAnalyticEdge {
    /// Translate already-quantized device edges into an integer-offset output
    /// tile. Curve derivatives and stepping counts are translation invariant.
    /// Shifting every current/end/snapped position avoids re-quantization.
    pub(crate) fn translate_fixed(&mut self, dx: i32, dy: i32) {
        debug_assert_eq!(dx & 65535, 0);
        debug_assert_eq!(dy & 65535, 0);
        self.x += dx;
        self.upper_x += dx;
        self.y += dy;
        self.upper_y += dy;
        self.lower_y += dy;
        if let Some(q) = self.quad.as_mut() {
            q.x += dx;
            q.last_x += dx;
            q.snapped_x += dx;
            q.y += dy;
            q.last_y += dy;
            q.snapped_y += dy;
        }
        if let Some(c) = self.cubic.as_mut() {
            c.x += dx;
            c.last_x += dx;
            c.y += dy;
            c.last_y += dy;
            c.snapped_y += dy;
        }
    }
    pub(crate) fn setLine(a: Pt, b: Pt) -> Option<Self> {
        let mut x0 = ((a.0 * 256.0) as i32) << 8;
        let mut x1 = ((b.0 * 256.0) as i32) << 8;
        let mut y0 = snap_y(((a.1 * 256.0) as i32) << 8);
        let mut y1 = snap_y(((b.1 * 256.0) as i32) << 8);
        if y0 > y1 {
            std::mem::swap(&mut x0, &mut x1);
            std::mem::swap(&mut y0, &mut y1);
        }
        let dy = (y1 - y0) >> 10;
        if dy == 0 {
            return None;
        }
        let dx = (x1 - x0) >> 10;
        let slope = quick_div(dx, dy);
        let reciprocal = if dx == 0 || slope == 0 {
            i32::MAX
        } else if slope.abs() < 1024 {
            inverse(slope.abs())
        } else {
            quick_div(dy, dx).abs()
        };
        Some(Self {
            x: x0,
            dx: slope,
            upper_x: x0,
            y: y0,
            upper_y: y0,
            lower_y: y1,
            dy: reciprocal,
            quad: None,
            cubic: None,
            winding: if a.1 > b.1 { -1 } else { 1 },
        })
    }
    pub(crate) fn setQuadratic(mut p: [Pt; 3]) -> Option<Self> {
        let winding = if p[0].1 > p[2].1 { -1 } else { 1 };
        if p[0].1 > p[2].1 {
            p.swap(0, 2);
        }
        let x = p.map(|v| (v.0 * 256.0) as i32);
        let y = p.map(|v| (v.1 * 256.0) as i32);
        if (y[0] + 32) >> 6 == (y[2] + 32) >> 6 {
            return None;
        }
        let ax = ((2 * x[1] - x[0] - x[2]) >> 2).abs();
        let ay = ((2 * y[1] - y[0] - y[2]) >> 2).abs();
        let dist = (ax.max(ay) + (ax.min(ay) >> 1) + 16) >> 5;
        let shift = ((32 - dist.leading_zeros()) >> 1).clamp(1, 6);
        let coefficients = |v: [i32; 3]| {
            let a = (v[0] - 2 * v[1] + v[2]) << 9;
            let b = (v[1] - v[0]) << 10;
            (
                (v[0] << 10) >> 2,
                (b + (a >> shift)) >> 2,
                (a >> (shift - 1)) >> 2,
                (v[2] << 10) >> 2,
            )
        };
        let (qx, qdx, qddx, last_x) = coefficients(x);
        let (qy, qdy, qddy, last_y) = coefficients(y);
        let qy = snap_y(qy);
        let mut edge = Self {
            x: qx,
            dx: 0,
            upper_x: qx,
            y: qy,
            upper_y: qy,
            lower_y: qy,
            dy: 0,
            winding,
            cubic: None,
            quad: Some(SkAnalyticQuadraticEdge {
                x: qx,
                y: qy,
                dx: qdx,
                dy: qdy,
                ddx: qddx,
                ddy: qddy,
                last_x,
                last_y: snap_y(last_y),
                snapped_x: qx,
                snapped_y: qy,
                shift: shift - 1,
                count: 1 << shift,
            }),
        };
        if edge.update() {
            Some(edge)
        } else {
            None
        }
    }
    pub(crate) fn setCubic(p: [Pt; 4]) -> Option<Self> {
        let c = SkAnalyticCubicEdge::new(p)?;
        let mut e = Self {
            x: 0,
            dx: 0,
            upper_x: 0,
            y: 0,
            upper_y: 0,
            lower_y: 0,
            dy: 0,
            quad: None,
            cubic: Some(c),
            winding: if p[0].1 > p[3].1 { -1 } else { 1 },
        };
        e.update().then_some(e)
    }
    pub(crate) fn update(&mut self) -> bool {
        if let Some(c) = self.cubic.as_mut() {
            if let Some((mut x0, mut y0, mut x1, mut y1, slope)) = c.next() {
                if y0 > y1 {
                    std::mem::swap(&mut x0, &mut x1);
                    std::mem::swap(&mut y0, &mut y1);
                    self.winding = -self.winding;
                }
                let dx = (x1 - x0) >> 10;
                let dy = (y1 - y0) >> 10;
                let abs_slope = (slope >> 10).abs();
                self.x = x0;
                self.dx = slope;
                self.upper_x = x0;
                self.y = y0;
                self.upper_y = y0;
                self.lower_y = y1;
                self.dy = if dx == 0 || slope == 0 {
                    i32::MAX
                } else if abs_slope < 1024 {
                    inverse(abs_slope)
                } else {
                    quick_div(dy, dx).abs()
                };
                return true;
            }
            return false;
        }
        let Some(q) = self.quad.as_mut() else {
            return false;
        };
        while q.count > 0 {
            q.count -= 1;
            let (nx, ny, sx, sy, slope);
            if q.count > 0 {
                nx = q.x + (q.dx >> q.shift);
                ny = q.y + (q.dy >> q.shift);
                if (q.dy >> q.shift).abs() >= ONE * 2
                    && (i64::from(q.dy.abs()) << 6) > i64::from(q.dx.abs())
                {
                    let dy = (ny - q.snapped_y) >> 10;
                    slope = if dy != 0 {
                        quick_div((nx - q.snapped_x) >> 10, dy)
                    } else {
                        i32::MAX
                    };
                    sy = q.last_y.min((ny + 32768) & !65535);
                    sx = nx - mul(slope, ny - sy);
                } else {
                    sy = q.last_y.min(snap_y(ny));
                    sx = nx;
                    let dy = (sy - q.snapped_y) >> 10;
                    slope = if dy != 0 {
                        quick_div((nx - q.snapped_x) >> 10, dy)
                    } else {
                        i32::MAX
                    };
                }
                q.dx += q.ddx;
                q.dy += q.ddy;
            } else {
                nx = q.last_x;
                ny = q.last_y;
                sx = nx;
                sy = ny;
                let dy = (ny - q.snapped_y) >> 10;
                slope = if dy != 0 {
                    quick_div((nx - q.snapped_x) >> 10, dy)
                } else {
                    i32::MAX
                };
            }
            let x0 = q.snapped_x;
            let y0 = q.snapped_y;
            q.x = nx;
            q.y = ny;
            // updateQuadratic keeps the original snapped start while it skips
            // zero-height segments; advancing it early changes partial coverage.
            if slope < i32::MAX && sy > y0 {
                q.snapped_x = sx;
                q.snapped_y = sy;
                self.x = x0;
                self.upper_x = x0;
                self.y = y0;
                self.upper_y = y0;
                self.lower_y = sy;
                self.dx = slope;
                let dx = (sx - x0) >> 10;
                let dy = (sy - y0) >> 10;
                let abs_slope = (slope >> 10).abs();
                self.dy = if dx == 0 || slope == 0 {
                    i32::MAX
                } else if abs_slope < 1024 {
                    inverse(abs_slope)
                } else {
                    quick_div(dy, dx).abs()
                };
                return true;
            }
            if q.count == 0 {
                q.snapped_x = sx;
                q.snapped_y = sy;
            }
        }
        false
    }
    pub(crate) fn go_y(&mut self, y: i32) {
        if y == self.y + ONE {
            self.x += self.dx;
        } else if y != self.y {
            self.x = self.upper_x + mul(self.dx, y - self.upper_y);
        }
        self.y = y;
    }
    pub(crate) fn smooth(&self, next: &Self) -> bool {
        if let Some(c) = &self.cubic {
            if c.count < 0 {
                return (c.dx.abs() >> 1) >= (c.ddx.abs() >> c.shift)
                    && (c.dy.abs() >> 1) >= (c.ddy.abs() >> c.shift)
                    && (c.dy - (c.ddy >> c.shift)) >> c.fixed_shift >= ONE;
            }
        }
        if let Some(q) = &self.quad {
            if q.count > 0 {
                return (q.dx.abs() >> 1) >= q.ddx.abs()
                    && (q.dy.abs() >> 1) >= q.ddy.abs()
                    && (q.dy - q.ddy) >> q.shift >= ONE;
            }
        }
        next.dx.saturating_sub(self.dx).abs() <= ONE && next.lower_y - next.upper_y >= ONE
    }
}

// CPU implementation moved from renderer/analytic_aa/cubic_edge.rs.

// SkAnalyticCubicEdge::setCubic/updateCubic. BSD license: ../glyphs/SKIA_LICENSE.

#[derive(Clone)]
pub(crate) struct SkAnalyticCubicEdge {
    pub x: i32,
    pub y: i32,
    pub dx: i32,
    pub dy: i32,
    pub ddx: i32,
    pub ddy: i32,
    dddx: i32,
    dddy: i32,
    last_x: i32,
    last_y: i32,
    snapped_y: i32,
    pub shift: u32,
    pub fixed_shift: u32,
    pub count: i32,
}
impl SkAnalyticCubicEdge {
    pub fn new(mut p: [Pt; 4]) -> Option<Self> {
        if p[0].1 > p[3].1 {
            p.reverse();
        }
        let x = p.map(|v| (v.0 * 256.0) as i32);
        let y = p.map(|v| (v.1 * 256.0) as i32);
        if (y[0] + 32) >> 6 == (y[3] + 32) >> 6 {
            return None;
        }
        let delta = |v: [i32; 4]| {
            let a = (v[0] * 8 - v[1] * 15 + 6 * v[2] + v[3]) * 19 >> 9;
            let b = (v[0] + 6 * v[1] - v[2] * 15 + v[3] * 8) * 19 >> 9;
            a.abs().max(b.abs())
        };
        let ax = delta(x);
        let ay = delta(y);
        let dist = (ax.max(ay) + (ax.min(ay) >> 1) + 16) >> 5;
        let shift = (((32 - dist.leading_zeros()) >> 1) + 1).min(6);
        let mut up_shift = 6;
        let mut fixed_shift = shift as i32 + up_shift - 10;
        if fixed_shift < 0 {
            fixed_shift = 0;
            up_shift = 10 - shift as i32;
        }
        let coefficients = |v: [i32; 4]| {
            let b = 3 * (v[1] - v[0]) << up_shift;
            let c = 3 * (v[0] - v[1] - v[1] + v[2]) << up_shift;
            let d = (v[3] + 3 * (v[1] - v[2]) - v[0]) << up_shift;
            (
                (v[0] << 10) >> 2,
                (b + (c >> shift) + (d >> (2 * shift))) >> 2,
                ((2 * i64::from(c) + ((3 * i64::from(d)) >> (shift - 1))) as i32) >> 2,
                (((3 * i64::from(d)) >> (shift - 1)) as i32) >> 2,
                (v[3] << 10) >> 2,
            )
        };
        let (x, dx, ddx, dddx, last_x) = coefficients(x);
        let (y, dy, ddy, dddy, last_y) = coefficients(y);
        let y = snap_y(y);
        Some(Self {
            x,
            y,
            dx,
            dy,
            ddx,
            ddy,
            dddx,
            dddy,
            last_x,
            last_y: snap_y(last_y),
            snapped_y: y,
            shift,
            fixed_shift: fixed_shift as u32,
            count: -(1 << shift),
        })
    }
    pub fn keep_continuous(&mut self, x: i32, y: i32) {
        self.x = x;
        self.snapped_y = y;
    }
    pub fn next(&mut self) -> Option<(i32, i32, i32, i32, i32)> {
        while self.count < 0 {
            self.count += 1;
            let ox = self.x;
            let oy = self.y;
            let sy = self.snapped_y;
            let (nx, mut ny) = if self.count < 0 {
                let x = ox + (self.dx >> self.fixed_shift);
                self.dx += self.ddx >> self.shift;
                self.ddx += self.dddx;
                let y = oy + (self.dy >> self.fixed_shift);
                self.dy += self.ddy >> self.shift;
                self.ddy += self.dddy;
                (x, y)
            } else {
                (self.last_x, self.last_y)
            };
            ny = ny.max(oy);
            let mut snapped = snap_y(ny);
            if snapped > self.last_y {
                snapped = self.last_y;
                self.count = 0;
            }
            let dx = (nx - ox) >> 10;
            let dy = (snapped - sy) >> 10;
            let slope = if dy == 0 {
                i32::MAX
            } else {
                ((i64::from(dx) << 16) / i64::from(dy)).clamp(i32::MIN as i64, i32::MAX as i64)
                    as i32
            };
            self.x = nx;
            self.y = ny;
            self.snapped_y = snapped;
            if dy != 0 {
                return Some((ox, sy, nx, snapped, slope));
            }
        }
        None
    }
}
