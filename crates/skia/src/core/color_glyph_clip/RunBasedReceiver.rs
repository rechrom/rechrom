//! Independent unadopted original receiver/accumulator candidate.
//! Copyright 2006, 2016 The Android Open Source Project (BSD-3-Clause).
//! Local scalar port of SkAlphaRuns and RunBased/SafeRLEAdditiveBlitter.
//! Native valid-input assertions apply only to this source candidate.
#[path = "ClipReceiver.rs"]
pub(crate) mod receiver;
use receiver::Receiver;
// Receiver's storage module is re-exported below only for standalone fixtures.
use receiver::clip::{Bounds, Encoding};

pub(crate) struct AlphaRuns {
    runs: Vec<i16>,
    alpha: Vec<u8>,
    width: usize,
}
impl AlphaRuns {
    pub(crate) fn new(width: usize) -> Self {
        assert!(width > 0 && width <= i16::MAX as usize);
        let mut out = Self {
            runs: vec![0; width + 1],
            alpha: vec![0; width + 1],
            width,
        };
        out.reset();
        out
    }
    fn reset(&mut self) {
        self.runs[0] = self.width as i16;
        self.runs[self.width] = 0;
        self.alpha[0] = 0;
    }
    fn empty(&self) -> bool {
        self.alpha[0] == 0 && self.runs[self.runs[0] as usize] == 0
    }
    /// Equivalent native BreakAt, called at both endpoints of native Break.
    /// Does not combine neighbouring equal-alpha partitions.
    fn break_at(&mut self, base: usize, x: usize) {
        let mut head = base;
        let target = base + x;
        assert!(target <= self.width);
        while head < target {
            let n = self.runs[head] as usize;
            assert!(n > 0);
            if target < head + n {
                self.alpha[target] = self.alpha[head];
                self.runs[head] = (target - head) as i16;
                self.runs[target] = (head + n - target) as i16;
                break;
            }
            head += n;
        }
    }
    fn break_range(&mut self, base: usize, x: usize, count: usize) {
        assert!(count > 0);
        self.break_at(base, x);
        self.break_at(base + x, count);
    }
    fn catch_overflow(value: u16) -> u8 {
        assert!(value <= 256);
        (value - (value >> 8)) as u8
    }
    pub(crate) fn add(
        &mut self,
        x: usize,
        start: u8,
        mut middle: usize,
        stop: u8,
        max: u8,
        offset: usize,
    ) -> usize {
        assert!(
            x >= offset
                && x + usize::from(start != 0) + middle + usize::from(stop != 0) <= self.width
        );
        let mut cursor = offset;
        let mut local_x = x - offset;
        let mut last_alpha = offset;
        if start != 0 {
            self.break_range(cursor, local_x, 1);
            let target = cursor + local_x;
            self.alpha[target] =
                Self::catch_overflow(u16::from(self.alpha[target]) + u16::from(start));
            cursor = target + 1;
            local_x = 0;
        }
        if middle > 0 {
            self.break_range(cursor, local_x, middle);
            cursor += local_x;
            local_x = 0;
            while middle > 0 {
                self.alpha[cursor] =
                    Self::catch_overflow(u16::from(self.alpha[cursor]) + u16::from(max));
                let n = self.runs[cursor] as usize;
                assert!(n > 0 && n <= middle);
                cursor += n;
                middle -= n;
            }
            last_alpha = cursor;
        }
        if stop != 0 {
            self.break_range(cursor, local_x, 1);
            let target = cursor + local_x;
            self.alpha[target] = self.alpha[target].wrapping_add(stop);
            last_alpha = target;
        }
        last_alpha
    }
    fn force_pixels(&mut self, x: usize, count: usize) {
        let mut i = x;
        while i < x + count {
            let n = self.runs[i] as usize;
            assert!(n > 0 && i + n <= x + count);
            for j in 1..n {
                self.runs[i + j] = 1;
                self.alpha[i + j] = self.alpha[i];
            }
            self.runs[i] = 1;
            i += n;
        }
    }
    fn spans_iter(&self) -> impl Iterator<Item = (i32, u8)> + '_ {
        let mut x = 0;
        std::iter::from_fn(move || {
            let n = self.runs[x] as usize;
            if n == 0 {
                assert_eq!(x, self.width);
                return None;
            }
            assert!(x + n <= self.width);
            let span = (n as i32, self.alpha[x]);
            x += n;
            Some(span)
        })
    }
    #[cfg(test)]
    fn spans(&self) -> Vec<(i32, u8)> {
        let mut out = Vec::new();
        let mut x = 0;
        while self.runs[x] != 0 {
            let n = self.runs[x] as usize;
            assert!(n > 0 && x + n <= self.width);
            out.push((n as i32, self.alpha[x]));
            x += n;
        }
        assert_eq!(x, self.width);
        out
    }
    fn snap(&mut self) {
        let mut x = 0;
        while self.runs[x] != 0 {
            self.alpha[x] = match self.alpha[x] {
                0..=7 => 0,
                248..=255 => 255,
                a => a,
            };
            x += self.runs[x] as usize;
        }
    }
}

pub(crate) struct RunBasedReceiver {
    real: Receiver,
    runs: AlphaRuns,
    left: i32,
    top: i32,
    current_y: Option<i32>,
    offset_x: usize,
    safe: bool,
}
impl RunBasedReceiver {
    /// bounds is native intersect(path.roundOut, actual clip bounds), without
    /// the dense destination's conservative padding or output-tile gutters.
    pub(crate) fn new(bounds: Bounds, safe: bool) -> Self {
        Self {
            real: Receiver::new(bounds),
            runs: AlphaRuns::new((bounds.0[2] - bounds.0[0]) as usize),
            left: bounds.0[0],
            top: bounds.0[1],
            current_y: None,
            offset_x: 0,
            safe,
        }
    }
    fn check_y(&mut self, y: i32) {
        if self.current_y != Some(y) {
            self.flush();
            self.current_y = Some(y);
        }
    }
    pub(crate) fn flush(&mut self) {
        if let Some(y) = self.current_y.filter(|&y| y >= self.top) {
            self.runs.snap();
            if !self.runs.empty() {
                self.real.anti_h_iter(self.left, y, self.runs.spans_iter());
                // requestRowsPreserved buffers can be replaced by one owned
                // buffer here: Receiver copies every segment synchronously.
                self.runs.reset();
                self.offset_x = 0;
            }
        }
        self.current_y = None;
    }
    pub(crate) fn flush_if_y_changed(&mut self, fixed_y: i32, next_y: i32) {
        if fixed_y >> 16 != next_y >> 16 {
            self.flush();
        }
    }
    pub(crate) fn accumulated_span(&mut self, x: i32, y: i32, width: i32, alpha: u8) {
        self.check_y(y);
        let x = x - self.left;
        if x < self.offset_x as i32 {
            self.offset_x = 0;
        }
        if x < 0 || width < 0 || x + width > self.runs.width as i32 {
            return;
        }
        let x = x as usize;
        let width = width as usize;
        self.offset_x = self.runs.add(
            x,
            0,
            width,
            0,
            if self.safe { 0 } else { alpha },
            self.offset_x,
        );
        if self.safe {
            let mut i = x;
            while i < x + width {
                self.runs.alpha[i] = self.runs.alpha[i].saturating_add(alpha);
                i += self.runs.runs[i] as usize;
            }
        }
    }
    pub(crate) fn accumulated_single(&mut self, x: i32, y: i32, alpha: u8) {
        self.accumulated_span(x, y, 1, alpha);
    }
    pub(crate) fn accumulated_array(&mut self, x: i32, y: i32, alphas: &[u8]) {
        self.check_y(y);
        let local = x - self.left;
        let skip = (-local).max(0) as usize;
        let start = local.max(0) as usize;
        let count = alphas
            .len()
            .saturating_sub(skip)
            .min(self.runs.width.saturating_sub(start));
        if start < self.offset_x {
            self.offset_x = 0;
        }
        if count == 0 {
            return;
        }
        self.offset_x = self.runs.add(start, 0, count, 0, 0, self.offset_x);
        self.runs.force_pixels(start, count);
        for (i, &delta) in alphas[skip..skip + count].iter().enumerate() {
            let target = &mut self.runs.alpha[start + i];
            *target = if self.safe {
                target.saturating_add(delta)
            } else {
                AlphaRuns::catch_overflow(u16::from(*target) + u16::from(delta))
            };
        }
    }
    // getRealBlitter() does NOT flush the additive receiver implicitly.
    pub(crate) fn direct_h(&mut self, x: i32, y: i32, width: i32) {
        self.real.h(x, y, width);
    }
    pub(crate) fn direct_v(&mut self, x: i32, y: i32, height: i32, alpha: u8) {
        self.real.v(x, y, height, alpha);
    }
    pub(crate) fn direct_anti_h(&mut self, x: i32, y: i32, spans: &[(i32, u8)]) {
        self.real.anti_h(x, y, spans);
    }
    pub(crate) fn direct_anti_h_iter(
        &mut self,
        x: i32,
        y: i32,
        spans: impl IntoIterator<Item = (i32, u8)>,
    ) {
        self.real.anti_h_iter(x, y, spans);
    }
    pub(crate) fn direct_rect(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.real.rect(x, y, width, height);
    }
    pub(crate) fn direct_anti_rect(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        left: u8,
        right: u8,
    ) {
        self.real.anti_rect(x, y, width, height, left, right);
    }
    pub(crate) fn finish(mut self) -> Option<Encoding> {
        self.flush();
        self.real.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn overlapping_equal_alphas_keep_break_partitions() {
        let mut runs = AlphaRuns::new(8);
        runs.add(0, 0, 8, 0, 100, 0);
        runs.add(2, 0, 4, 0, 0, 0);
        assert_eq!(runs.spans(), [(2, 100), (4, 100), (2, 100)]);
        assert!(!runs.empty());
    }
    #[test]
    fn zero_split_is_not_native_empty() {
        let mut runs = AlphaRuns::new(8);
        assert!(runs.empty());
        runs.add(2, 0, 4, 0, 0, 0);
        assert!(!runs.empty());
        assert_eq!(runs.spans(), [(2, 0), (4, 0), (2, 0)]);
    }
    #[test]
    fn overflow256_and_alpha_snap_thresholds_are_exact() {
        let mut runs = AlphaRuns::new(4);
        for (x, a) in [7, 8, 247, 248].into_iter().enumerate() {
            runs.add(x, 0, 1, 0, a, 0);
        }
        runs.snap();
        assert_eq!(runs.spans(), [(1, 0), (1, 8), (1, 247), (1, 255)]);
        assert_eq!(AlphaRuns::catch_overflow(256), 255);
    }
    #[test]
    fn array_form_preserves_one_pixel_runs_and_true_width_anchor() {
        let mut r = RunBasedReceiver::new(Bounds([10, 4, 16, 8]), false);
        r.accumulated_array(11, 4, &[100, 100, 100, 100]);
        r.flush();
        let e = r.finish().unwrap();
        assert_eq!(e.bounds, Bounds([11, 4, 15, 5]));
        assert_eq!(
            e.rows[0].runs.iter().map(|r| r.count).collect::<Vec<_>>(),
            [1, 1, 1, 1]
        );
    }
    #[test]
    fn safe_concave_overlap_saturates_without_collapsing_runs() {
        let mut r = RunBasedReceiver::new(Bounds([0, 0, 8, 4]), true);
        r.accumulated_span(0, 0, 8, 200);
        r.accumulated_span(2, 0, 4, 200);
        let e = r.finish().unwrap();
        assert_eq!(
            e.rows[0]
                .runs
                .iter()
                .map(|r| (r.count, r.alpha))
                .collect::<Vec<_>>(),
            [(2, 200), (4, 255), (2, 200)]
        );
    }
    #[test]
    fn fixed_y_flush_keeps_unmerged_pretrim_row_partitions() {
        let mut r = RunBasedReceiver::new(Bounds([0, 0, 8, 4]), false);
        r.accumulated_span(0, 0, 8, 100);
        r.flush_if_y_changed(0, 65536);
        r.accumulated_array(0, 1, &[100; 8]);
        let e = r.finish().unwrap();
        assert_eq!(e.rows.len(), 2);
        assert_eq!(e.rows[0].runs.len(), 1);
        assert_eq!(e.rows[1].runs.len(), 8);
    }
}
