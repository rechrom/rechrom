//! Standalone source candidate, not compiled or adopted.
//! Replays the *real final* SkAAClip::Builder::Blitter event stream.
//! Internal additive trapezoids or canonicalized dense rows are not this input.
#[path = "Encoding.rs"]
pub(crate) mod clip;
use clip::{Bounds, Encoding, Row, Run};

pub(crate) struct Receiver {
    bounds: Bounds,
    rows: Vec<Row>,
    current_y: Option<i32>,
    row_width: i32,
    last_y: Option<i32>,
    min_y: Option<i32>,
}
impl Receiver {
    pub(crate) fn new(bounds: Bounds) -> Self {
        Self {
            bounds,
            rows: Vec::new(),
            current_y: None,
            row_width: 0,
            last_y: None,
            min_y: None,
        }
    }
    fn width(&self) -> i32 {
        self.bounds.0[2] - self.bounds.0[0]
    }
    fn append(&mut self, alpha: u8, mut count: i32) {
        while count > 0 {
            let n = count.min(255) as u8;
            self.rows
                .last_mut()
                .unwrap()
                .runs
                .push(Run { count: n, alpha });
            count -= i32::from(n);
        }
    }
    fn flush_h(&mut self) {
        if self.current_y.is_some() {
            let remaining = self.width() - self.row_width;
            self.append(0, remaining);
            self.row_width += remaining;
        }
    }
    fn add_run(&mut self, x: i32, y: i32, alpha: u8, count: i32) {
        assert!(count > 0 && x >= self.bounds.0[0] && x + count <= self.bounds.0[2]);
        assert!(y >= self.bounds.0[1] && y < self.bounds.0[3]);
        if self.current_y != Some(y) {
            if let Some(last) = self.rows.last() {
                assert!(y > last.last_y);
            }
            self.flush_h();
            self.rows.push(Row {
                last_y: y,
                runs: Vec::new(),
            });
            self.current_y = Some(y);
            self.row_width = 0;
        }
        let local_x = x - self.bounds.0[0];
        assert!(self.row_width <= local_x);
        let gap = local_x - self.row_width;
        self.append(0, gap);
        self.append(alpha, count);
        self.row_width = local_x + count;
    }
    fn record_min_y(&mut self, y: i32) {
        self.min_y = Some(self.min_y.map_or(y, |old| old.min(y)));
    }
    fn check_y_gap(&mut self, y: i32) {
        if let Some(last) = self.last_y {
            assert!(y >= last);
            if y - last > 1 {
                self.add_run(self.bounds.0[0], y - 1, 0, self.width());
            }
        }
        self.last_y = Some(y);
    }
    pub(crate) fn h(&mut self, x: i32, y: i32, width: i32) {
        self.record_min_y(y);
        self.check_y_gap(y);
        self.add_run(x, y, 255, width);
    }
    /// Already emitted count/alpha spans, before AppendRun's count255 split.
    /// Adjacent equal-alpha spans remain separate, including alpha0 spans.
    /// This is a decoded view of native sparse-alpha *receiver* runs, not
    /// runs discovered by scanning the completed dense destination.
    pub(crate) fn anti_h(&mut self, x: i32, y: i32, spans: &[(i32, u8)]) {
        self.anti_h_iter(x, y, spans.iter().copied());
    }
    pub(crate) fn anti_h_iter(
        &mut self,
        mut x: i32,
        y: i32,
        spans: impl IntoIterator<Item = (i32, u8)>,
    ) {
        self.record_min_y(y);
        self.check_y_gap(y);
        for (count, alpha) in spans {
            assert!(count > 0);
            let right = x + count;
            let local_x = x.max(self.bounds.0[0]);
            let local_right = right.min(self.bounds.0[2]);
            if local_x != x || local_right != right {
                assert!(alpha < 16);
            }
            if local_right > local_x {
                self.add_run(local_x, y, alpha, local_right - local_x);
            }
            x = right;
        }
    }
    pub(crate) fn rect(&mut self, x: i32, y: i32, width: i32, height: i32) {
        assert!(height > 0 && y + height <= self.bounds.0[3]);
        self.record_min_y(y);
        self.check_y_gap(y);
        self.add_run(x, y, 255, width);
        self.flush_h();
        self.rows.last_mut().unwrap().last_y = y + height - 1;
        self.last_y = Some(y + height - 1);
    }
    pub(crate) fn anti_rect(
        &mut self,
        mut x: i32,
        y: i32,
        mut width: i32,
        height: i32,
        left: u8,
        right: u8,
    ) {
        assert!(width >= 0 && height > 0 && y + height <= self.bounds.0[3]);
        self.record_min_y(y);
        self.check_y_gap(y);
        if left == 255 {
            width += 1;
        } else if left > 0 {
            self.add_run(x, y, left, 1);
            x += 1;
        } else {
            x += 1;
        }
        if right == 255 {
            width += 1;
        }
        if width > 0 {
            self.add_run(x, y, 255, width);
        }
        if right > 0 && right < 255 {
            self.add_run(x + width, y, right, 1);
        }
        // Exactly as native: a fully empty AntiRect need not create a row.
        // This condition must not be rewritten as dense zero output.
        if self.current_y.is_some() {
            assert_eq!(self.current_y, Some(y));
            self.flush_h();
            self.rows.last_mut().unwrap().last_y = y + height - 1;
        }
        self.last_y = Some(y + height - 1);
    }
    pub(crate) fn v(&mut self, x: i32, y: i32, height: i32, alpha: u8) {
        assert!(height > 0 && y + height <= self.bounds.0[3]);
        if height == 1 {
            self.anti_h(x, y, &[(1, alpha)]);
        } else {
            // Native blitV(height>1) deliberately omits checkForYGap.
            self.record_min_y(y);
            self.add_run(x, y, alpha, 1);
            self.flush_h();
            self.rows.last_mut().unwrap().last_y = y + height - 1;
            self.last_y = Some(y + height - 1);
        }
    }
    pub(crate) fn finish(mut self) -> Option<Encoding> {
        self.flush_h();
        let mut bounds = self.bounds;
        bounds.0[1] = self.min_y?;
        Encoding::finish(bounds, self.rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn receiver_keeps_explicit_y_holes_and_min_y() {
        let mut r = Receiver::new(Bounds([0, 0, 8, 9]));
        r.h(2, 2, 3);
        r.h(2, 6, 3);
        let e = r.finish().unwrap();
        assert_eq!(e.bounds, Bounds([2, 2, 5, 7]));
        assert_eq!(
            e.rows.iter().map(|row| row.last_y).collect::<Vec<_>>(),
            [0, 3, 4]
        );
        assert_eq!(e.rows[1].runs, [Run { count: 3, alpha: 0 }]);
    }
    #[test]
    fn count255_split_is_anchored_to_original_event() {
        let mut r = Receiver::new(Bounds([0, 0, 700, 3]));
        r.anti_h(0, 0, &[(260, 255), (440, 255)]);
        r.anti_h(0, 1, &[(700, 255)]);
        let e = r.finish().unwrap();
        assert_eq!(e.rows.len(), 2);
        assert_eq!(
            e.rows[0]
                .runs
                .iter()
                .map(|run| run.count)
                .collect::<Vec<_>>(),
            [255, 5, 255, 185]
        );
        assert_eq!(
            e.rows[1]
                .runs
                .iter()
                .map(|run| run.count)
                .collect::<Vec<_>>(),
            [255, 255, 190]
        );
    }
    #[test]
    fn anti_rect_zero_left_and_full_right_follow_native_merging() {
        let mut r = Receiver::new(Bounds([0, 0, 8, 8]));
        r.anti_rect(1, 2, 2, 3, 0, 255);
        let e = r.finish().unwrap();
        assert_eq!(e.bounds, Bounds([2, 2, 5, 5]));
        assert_eq!(
            e.rows.as_slice(),
            &[Row {
                last_y: 2,
                runs: vec![Run {
                    count: 3,
                    alpha: 255
                }]
            }]
        );
    }
    #[test]
    fn vertical_single_pixel_and_multiline_retain_same_receiver_rules() {
        let mut r = Receiver::new(Bounds([0, 0, 8, 8]));
        r.v(2, 2, 1, 128);
        r.v(2, 3, 3, 128);
        let e = r.finish().unwrap();
        assert_eq!(e.bounds, Bounds([2, 2, 3, 6]));
        assert_eq!(
            e.rows.as_slice(),
            &[Row {
                last_y: 3,
                runs: vec![Run {
                    count: 1,
                    alpha: 128
                }]
            }]
        );
    }
}
