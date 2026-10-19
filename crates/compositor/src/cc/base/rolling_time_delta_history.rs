// Translation status: complete
// Source: cc/base/rolling_time_delta_history.h:1-54
// Source: cc/base/rolling_time_delta_history.cc:1-86
use crate::adapter::time::TimeDelta;
use std::collections::{BTreeSet, VecDeque};

// Source: cc/base/rolling_time_delta_history.h:22-50
pub struct RollingTimeDeltaHistory {
    // A multiset iterator's stable identity is represented by its unique serial.
    // Equal duration samples remain distinct and the deque erases that exact entry.
    sample_set_: BTreeSet<(TimeDelta, u64)>,
    chronological_sample_deque_: VecDeque<(TimeDelta, u64)>,
    max_size_: usize,
    percentile_cache_: Vec<(f64, TimeDelta)>,
    next_serial: u64,
}
impl RollingTimeDeltaHistory {
    // Source: cc/base/rolling_time_delta_history.cc:14-15
    pub fn new(max_size: usize) -> Self {
        Self {
            sample_set_: BTreeSet::new(),
            chronological_sample_deque_: VecDeque::new(),
            max_size_: max_size,
            percentile_cache_: Vec::new(),
            next_serial: 0,
        }
    }
    // Source: cc/base/rolling_time_delta_history.cc:19-31
    pub fn InsertSample(&mut self, time: TimeDelta) {
        if self.max_size_ == 0 {
            return;
        }
        if self.sample_set_.len() == self.max_size_ {
            let key = self.chronological_sample_deque_.pop_front().unwrap();
            self.sample_set_.remove(&key);
        }
        let key = (time, self.next_serial);
        self.next_serial = self
            .next_serial
            .checked_add(1)
            .expect("native multiset iterator identity exhausted");
        self.sample_set_.insert(key);
        self.chronological_sample_deque_.push_back(key);
        self.percentile_cache_.clear();
    }
    // Source: cc/base/rolling_time_delta_history.cc:33-38
    pub fn RemoveOldestSample(&mut self) {
        if !self.sample_set_.is_empty() {
            let key = self.chronological_sample_deque_.pop_front().unwrap();
            self.sample_set_.remove(&key);
        }
        // Source intentionally does not invalidate the percentile cache here.
    }
    // Source: cc/base/rolling_time_delta_history.h:32
    pub fn sample_count(&self) -> usize {
        self.sample_set_.len()
    }
    // Source: cc/base/rolling_time_delta_history.cc:40-44
    pub fn Clear(&mut self) {
        self.chronological_sample_deque_.clear();
        self.sample_set_.clear();
        self.percentile_cache_.clear();
    }
    // Source: cc/base/rolling_time_delta_history.cc:46-54
    pub fn Percentile(&mut self, percent: f64) -> TimeDelta {
        let index = self
            .percentile_cache_
            .partition_point(|(key, _)| *key < percent);
        if index == self.percentile_cache_.len() || percent < self.percentile_cache_[index].0 {
            let result = self.ComputePercentile(percent);
            self.percentile_cache_.insert(index, (percent, result));
        }
        self.percentile_cache_[index].1
    }
    // Source: cc/base/rolling_time_delta_history.cc:56-84
    fn ComputePercentile(&self, percent: f64) -> TimeDelta {
        if self.sample_set_.is_empty() {
            return TimeDelta::default();
        }
        let fraction = percent / 100.0;
        if fraction <= 0.0 {
            return self.sample_set_.first().unwrap().0;
        }
        if fraction >= 1.0 {
            return self.sample_set_.last().unwrap().0;
        }
        let num_smaller_samples = (fraction * self.sample_set_.len() as f64).ceil() as usize - 1;
        if num_smaller_samples > self.sample_set_.len() / 2 {
            let num_larger_samples = self.sample_set_.len() - num_smaller_samples - 1;
            let mut iter = self.sample_set_.iter().rev();
            for _ in 0..num_larger_samples {
                iter.next();
            }
            return iter.next().unwrap().0;
        }
        let mut iter = self.sample_set_.iter();
        for _ in 0..num_smaller_samples {
            iter.next();
        }
        iter.next().unwrap().0
    }
}
