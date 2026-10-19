// Translation status: complete
// Source: cc/input/scroll_velocity_tracker.h:1-54
// Source: cc/input/scroll_velocity_tracker.cc:1-72
// Copyright 2026 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/input/scroll_velocity_tracker.h:1-17,42,50-54;
//         cc/input/scroll_velocity_tracker.cc:1-18,72.
// Rust module/imports replace headers/exports/namespaces. VecDeque supplies the
// circular-deque dependency; automatic destruction is the defaulted destructor.
use crate::adapter::browser_controls::InMillisecondsF;
use crate::adapter::input_geometry::Vector2dF;
use crate::adapter::time::{TimeDelta, TimeTicks};
use std::collections::VecDeque;

// Source: cc/input/scroll_velocity_tracker.h:43-46.
struct Sample {
    timestamp: TimeTicks,
    scroll_delta: Vector2dF,
}
// Source: cc/input/scroll_velocity_tracker.h:15-16,48-49.
// A moving time-window scroll velocity tracker.
pub struct ScrollVelocityTracker {
    samples_: VecDeque<Sample>,
    window_delta_: TimeDelta,
}
impl ScrollVelocityTracker {
    // Source: cc/input/scroll_velocity_tracker.h:18-22;
    //         cc/input/scroll_velocity_tracker.cc:14-18.
    pub fn new(window_delta: TimeDelta) -> Self {
        Self {
            samples_: VecDeque::new(),
            window_delta_: window_delta,
        }
    }
    // Source: cc/input/scroll_velocity_tracker.h:23-32;
    //         cc/input/scroll_velocity_tracker.cc:19-44.
    pub fn CurrentVelocity(&self) -> Vector2dF {
        if self.samples_.is_empty() {
            return Vector2dF::default();
        }
        // Smallest duration encompassing samples; one sample uses the window.
        let mut duration_ms = InMillisecondsF(self.window_delta_) as f32;
        if self.samples_.len() > 1 {
            let oldest_sample = self.samples_.front().unwrap();
            let latest_sample = self.samples_.back().unwrap();
            duration_ms = InMillisecondsF(latest_sample.timestamp - oldest_sample.timestamp) as f32;
        }
        assert!(duration_ms > 0.0);
        let mut total_delta = Vector2dF::default();
        for sample in &self.samples_ {
            total_delta = total_delta + sample.scroll_delta;
        }
        // gfx::Vector2dF::InvScale divides independently, without reciprocal rounding.
        Vector2dF::new(total_delta.x() / duration_ms, total_delta.y() / duration_ms)
    }
    // Source: cc/input/scroll_velocity_tracker.h:33-36;
    //         cc/input/scroll_velocity_tracker.cc:45-67.
    pub fn AddSample(&mut self, timestamp: TimeTicks, scroll_delta: &Vector2dF) {
        if self.window_delta_ != TimeDelta::Max() {
            self.samples_
                .retain(|sample| (timestamp - sample.timestamp) <= self.window_delta_);
        }
        if let Some(last) = self.samples_.back_mut() {
            assert!(last.timestamp <= timestamp);
            if last.timestamp == timestamp {
                last.scroll_delta = last.scroll_delta + *scroll_delta;
                return;
            }
        }
        self.samples_.push_back(Sample {
            timestamp,
            scroll_delta: *scroll_delta,
        });
    }
    // Source: cc/input/scroll_velocity_tracker.h:37-39;
    //         cc/input/scroll_velocity_tracker.cc:68-71.
    pub fn Reset(&mut self) {
        self.samples_.clear();
    }
    // Source: cc/input/scroll_velocity_tracker.h:40-41.
    pub fn num_samples(&self) -> usize {
        self.samples_.len()
    }
}
