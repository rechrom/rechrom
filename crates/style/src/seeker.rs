#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]
// Copyright 2024 The Chromium Authors. BSD-style license; see Chromium LICENSE.

// cpp: third_party/blink/renderer/core/css/rule_set.h:597-608
// Borrowed references replace Oilpan Member<T>, including its null value.
pub struct Interval<'a, T> {
    pub start_position: u32,
    pub value: Option<&'a T>,
}

// cpp: third_party/blink/renderer/core/css/seeker.h:18-47
pub struct Seeker<'a, T> {
    intervals: &'a [Interval<'a, T>],
    iter: usize,
    last_rule_position: u32,
}
impl<'a, T> Seeker<'a, T> {
    // cpp: third_party/blink/renderer/core/css/seeker.h:23-24
    pub fn new(intervals: &'a [Interval<'a, T>]) -> Self {
        Self {
            intervals,
            iter: 0,
            last_rule_position: 0,
        }
    }
    // cpp: third_party/blink/renderer/core/css/seeker.h:26-39
    pub fn Seek(&mut self, rule_position: u32) -> Option<&'a T> {
        debug_assert!(rule_position >= self.last_rule_position);
        self.last_rule_position = rule_position;
        while self.iter < self.intervals.len()
            && self.intervals[self.iter].start_position <= rule_position
        {
            self.iter += 1;
        }
        if self.iter == 0 {
            return None;
        }
        self.intervals[self.iter - 1].value
    }
}

#[cfg(test)]
mod tests {
    use crate::seeker;
    #[test]
    fn null_and_equal_position_intervals() {
        let a = 10;
        let b = 20;
        let intervals = [
            seeker::Interval {
                start_position: 2,
                value: Some(&a),
            },
            seeker::Interval {
                start_position: 4,
                value: None,
            },
            seeker::Interval {
                start_position: 4,
                value: Some(&b),
            },
            seeker::Interval {
                start_position: 8,
                value: None,
            },
        ];
        let mut s = seeker::Seeker::new(&intervals);
        assert_eq!(s.Seek(0), None);
        assert_eq!(s.Seek(2), Some(&10));
        assert_eq!(s.Seek(4), Some(&20));
        assert_eq!(s.Seek(7), Some(&20));
        assert_eq!(s.Seek(8), None);
        assert_eq!(s.Seek(100), None);
    }
}
