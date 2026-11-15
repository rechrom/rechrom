// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Chromium 6c1d401f public/platform/web_time_range.h:42-77 and
//! platform/exported/web_time_range.cc:40-91,148-181. Canonical interval owner.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WebTimeRange {
    pub start: f64,
    pub end: f64,
}
impl WebTimeRange {
    fn IsPointInRange(self, point: f64) -> bool {
        self.start <= point && point < self.end
    }
    fn IsOverlappingRange(self, range: Self) -> bool {
        self.IsPointInRange(range.start)
            || self.IsPointInRange(range.end)
            || range.IsPointInRange(self.start)
    }
    fn IsContiguousWithRange(self, range: Self) -> bool {
        range.start == self.end || range.end == self.start
    }
    fn IsBeforeRange(self, range: Self) -> bool {
        range.start >= self.end
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WebTimeRanges {
    pub ranges: Vec<WebTimeRange>,
}
impl WebTimeRanges {
    pub fn Add(&mut self, start: f64, end: f64) {
        debug_assert!(start <= end);
        let mut added = WebTimeRange { start, end };
        let mut index = 0;
        while index < self.ranges.len() {
            let range = self.ranges[index];
            if added.IsOverlappingRange(range) || added.IsContiguousWithRange(range) {
                added = WebTimeRange {
                    start: added.start.min(range.start),
                    end: added.end.max(range.end),
                };
                self.ranges.remove(index);
                continue;
            }
            if (index == 0 && added.IsBeforeRange(self.ranges[0]))
                || (index != 0
                    && self.ranges[index - 1].IsBeforeRange(added)
                    && added.IsBeforeRange(range))
            {
                break;
            }
            index += 1;
        }
        self.ranges.insert(index, added);
    }
    pub fn Contain(&self, time: f64) -> bool {
        self.ranges
            .iter()
            .any(|range| time >= range.start && time <= range.end)
    }
    pub fn Nearest(&self, position: f64, current: f64) -> f64 {
        let mut best_match: f64 = 0.;
        let mut best_delta = f64::INFINITY;
        for range in &self.ranges {
            if position >= range.start && position <= range.end {
                return position;
            }
            let (delta, value) = if position < range.start {
                (range.start - position, range.start)
            } else {
                (position - range.end, range.end)
            };
            if delta < best_delta
                || (delta == best_delta && (current - value).abs() < (current - best_match).abs())
            {
                best_delta = delta;
                best_match = value;
            }
        }
        best_match
    }
}
