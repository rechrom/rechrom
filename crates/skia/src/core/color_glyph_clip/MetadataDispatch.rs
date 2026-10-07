//! Source-only original final-stream dispatch. No dense reconstruction.
#[path = "RunBasedReceiver.rs"]
pub(crate) mod runs;
pub(crate) use runs::receiver::clip::{Bounds, Encoding, Row, Run};
use runs::RunBasedReceiver;

pub(crate) struct MetadataDispatch {
    stream: RunBasedReceiver,
}
fn partial(a: u8, full: u8) -> u8 {
    ((u32::from(a) * u32::from(full)) >> 8) as u8
}
fn fixed_alpha(a: i32) -> u8 {
    ((255_i64 * i64::from(a) + 32768) >> 16) as u8
}
fn fixed_mul(a: i32, b: i32) -> i32 {
    ((i64::from(a) * i64::from(b)) >> 16) as i32
}
fn fixed_ceil(a: i32) -> i32 {
    (a + 65535) >> 16
}
impl MetadataDispatch {
    pub(crate) fn new(snug_path_clip_bounds: Bounds, concave: bool) -> Self {
        Self {
            stream: RunBasedReceiver::new(snug_path_clip_bounds, concave),
        }
    }
    pub(crate) fn single(&mut self, x: i32, y: i32, a: u8, full: u8, native_no_real: bool) {
        if full == 255 && !native_no_real {
            self.stream.direct_v(x, y, 1, a);
        } else {
            self.stream.accumulated_single(x, y, partial(a, full));
        }
    }
    pub(crate) fn two(&mut self, x: i32, y: i32, a: u8, b: u8, full: u8, native_no_real: bool) {
        if full == 255 && !native_no_real {
            self.stream.direct_anti_h(x, y, &[(1, a), (1, b)]);
        } else {
            // Original blit_two_alphas already receives weighted alphas.
            self.stream.accumulated_single(x, y, a);
            self.stream.accumulated_single(x + 1, y, b);
        }
    }
    pub(crate) fn full(&mut self, x: i32, y: i32, width: i32, full: u8, native_no_real: bool) {
        if width <= 0 {
            return;
        }
        if full == 255 && !native_no_real {
            self.stream.direct_h(x, y, width);
        } else {
            self.stream.accumulated_span(x, y, width, full);
        }
    }
    pub(crate) fn array(&mut self, x: i32, y: i32, alphas: &[u8], full: u8, native_no_real: bool) {
        if full == 255 && !native_no_real {
            // Original blit_aaa_trapezoid_row initializes every run to1.
            self.stream
                .direct_anti_h_iter(x, y, alphas.iter().map(|&a| (1, a)));
        } else {
            self.stream.accumulated_array(x, y, alphas);
        }
    }
    /// The complete native convex zero-dX band must be recorded once before
    /// dense output chooses its local per-row/real-rectangle implementation.
    pub(crate) fn vertical_band(&mut self, left: i32, right: i32, y: i32, bottom: i32) {
        let full_left = fixed_ceil(left);
        let full_right = right >> 16;
        let partial_left = (full_left << 16) - left;
        let partial_right = right - (full_right << 16);
        let full_top = fixed_ceil(y);
        let full_bot = bottom >> 16;
        let mut partial_top = (full_top << 16) - y;
        let mut partial_bot = bottom - (full_bot << 16);
        if full_top > full_bot {
            partial_top -= 65536 - partial_bot;
            partial_bot = 0;
        }
        if full_right >= full_left {
            if partial_top > 0 {
                if partial_left > 0 {
                    self.stream.accumulated_single(
                        full_left - 1,
                        full_top - 1,
                        fixed_alpha(fixed_mul(partial_top, partial_left)),
                    );
                }
                self.stream.accumulated_span(
                    full_left,
                    full_top - 1,
                    full_right - full_left,
                    fixed_alpha(partial_top),
                );
                if partial_right > 0 {
                    self.stream.accumulated_single(
                        full_right,
                        full_top - 1,
                        fixed_alpha(fixed_mul(partial_top, partial_right)),
                    );
                }
                self.stream.flush_if_y_changed(y, y + partial_top);
            }
            if full_bot > full_top
                && (full_right > full_left
                    || fixed_alpha(partial_left) > 0
                    || fixed_alpha(partial_right) > 0)
            {
                self.stream.direct_anti_rect(
                    full_left - 1,
                    full_top,
                    full_right - full_left,
                    full_bot - full_top,
                    fixed_alpha(partial_left),
                    fixed_alpha(partial_right),
                );
            }
            if partial_bot > 0 {
                if partial_left > 0 {
                    self.stream.accumulated_single(
                        full_left - 1,
                        full_bot,
                        fixed_alpha(fixed_mul(partial_bot, partial_left)),
                    );
                }
                self.stream.accumulated_span(
                    full_left,
                    full_bot,
                    full_right - full_left,
                    fixed_alpha(partial_bot),
                );
                if partial_right > 0 {
                    self.stream.accumulated_single(
                        full_right,
                        full_bot,
                        fixed_alpha(fixed_mul(partial_bot, partial_right)),
                    );
                }
            }
        } else {
            let width = right - left;
            if width > 0 {
                if partial_top > 0 {
                    self.stream.accumulated_span(
                        full_left - 1,
                        full_top - 1,
                        1,
                        fixed_alpha(fixed_mul(partial_top, width)),
                    );
                    self.stream.flush_if_y_changed(y, y + partial_top);
                }
                if full_bot > full_top {
                    self.stream.direct_v(
                        full_left - 1,
                        full_top,
                        full_bot - full_top,
                        fixed_alpha(width),
                    );
                }
                if partial_bot > 0 {
                    self.stream.accumulated_span(
                        full_left - 1,
                        full_bot,
                        1,
                        fixed_alpha(fixed_mul(partial_bot, width)),
                    );
                }
            }
        }
    }
    /// General active-edge direct rectangle shortcut, at source's own branch.
    pub(crate) fn direct_rect(&mut self, x: i32, y: i32, w: i32, h: i32) {
        self.stream.direct_rect(x, y, w, h);
    }
    pub(crate) fn flush_if_y_changed(&mut self, y: i32, next: i32) {
        self.stream.flush_if_y_changed(y, next);
    }
    pub(crate) fn finish(self) -> Option<Encoding> {
        self.stream.finish()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn real_single_and_rle_single_have_distinct_original_alpha_stages() {
        let b = Bounds([0, 0, 4, 4]);
        let mut direct = MetadataDispatch::new(b, false);
        direct.single(1, 0, 128, 255, false);
        let mut accumulated = MetadataDispatch::new(b, false);
        accumulated.single(1, 0, 128, 255, true);
        assert_eq!(direct.finish().unwrap().rows[0].runs[0].alpha, 128);
        assert_eq!(accumulated.finish().unwrap().rows[0].runs[0].alpha, 127);
    }
    #[test]
    fn two_alphas_are_not_scaled_twice_when_full_is_partial() {
        let mut d = MetadataDispatch::new(Bounds([0, 0, 4, 4]), false);
        d.two(1, 0, 100, 150, 128, true);
        let e = d.finish().unwrap();
        assert_eq!(
            e.rows[0].runs.iter().map(|r| r.alpha).collect::<Vec<_>>(),
            [100, 150]
        );
    }
    #[test]
    fn vertical_fractional_band_retains_partial_full_partial_y_records() {
        let mut d = MetadataDispatch::new(Bounds([0, 0, 8, 8]), false);
        d.vertical_band(
            2 * 65536 + 16384,
            5 * 65536 + 32768,
            65536 + 16384,
            5 * 65536 + 49152,
        );
        let e = d.finish().unwrap();
        assert_eq!(e.bounds, Bounds([2, 1, 6, 6]));
        assert_eq!(e.rows.len(), 3);
        assert_eq!(
            e.rows.iter().map(|r| r.last_y).collect::<Vec<_>>(),
            [0, 3, 4]
        );
    }
}
