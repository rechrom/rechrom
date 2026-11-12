// Copyright 2014 The Chromium Authors. BSD-style license.
// Chromium: ui/events/velocity_tracker/velocity_tracker.cc:267-375,488-668.
// Source LSQ2 strategy. This owner stores movement history, never input tasks.
use crate::public::common::input::web_input_event::Type;
use crate::public::common::input::web_touch_event::WebTouchEvent;
use std::collections::{HashMap, VecDeque};

#[derive(Default)]
pub(super) struct VelocityTracker {
    samples: HashMap<i32, VecDeque<(i64, f32, f32)>>,
    last_time: i64,
}
impl VelocityTracker {
    pub fn Clear(&mut self) {
        self.samples.clear();
    }
    pub fn AddMovement(&mut self, e: &WebTouchEvent) {
        let t = e.TimeStamp().ToInternalValue();
        match e.GetType() {
            Type::kTouchStart if e.IsTouchSequenceStart() => self.Clear(),
            Type::kTouchStart => {
                for p in &e.touches[..e.touches_length] {
                    if p.state == super::gesture_provider::TouchState::kStatePressed {
                        self.samples.remove(&p.pointer_properties.id);
                    }
                }
            }
            Type::kTouchMove => {
                if t - self.last_time >= 40_000 {
                    self.Clear();
                }
            }
            Type::kTouchEnd => {
                if t - self.last_time >= 80_000 {
                    self.Clear();
                }
                return;
            }
            _ => return,
        }
        self.last_time = t;
        for p in &e.touches[..e.touches_length] {
            let xy = p.pointer_properties.PositionInWidget();
            let samples = self.samples.entry(p.pointer_properties.id).or_default();
            if samples.back().is_some_and(|p| p.0 == t) {
                samples.pop_back();
            }
            samples.push_back((t, xy.x(), xy.y()));
            while samples.len() > 20 {
                samples.pop_front();
            }
        }
    }
    pub fn Velocity(&self, id: i32, max: f32) -> (f32, f32) {
        let Some(samples) = self.samples.get(&id) else {
            return (0., 0.);
        };
        let Some(newest) = samples.back() else {
            return (0., 0.);
        };
        let samples: Vec<_> = samples
            .iter()
            .rev()
            .take_while(|s| newest.0 - s.0 <= 100_000)
            .collect();
        if samples.len() < 2 {
            return (0., 0.);
        }
        let t: Vec<f32> = samples
            .iter()
            .map(|s| (s.0 - newest.0) as f32 / 1_000_000.)
            .collect();
        let x: Vec<f32> = samples.iter().map(|s| s.1).collect();
        let y: Vec<f32> = samples.iter().map(|s| s.2).collect();
        let n = 3.min(samples.len());
        let solve = |values: &[f32]| -> Option<f32> {
            let m = t.len();
            let mut a = vec![vec![0.; m]; n];
            for h in 0..m {
                a[0][h] = 1.;
                for i in 1..n {
                    a[i][h] = a[i - 1][h] * t[h];
                }
            }
            let mut q = a.clone();
            let mut r = vec![vec![0.; n]; n];
            for j in 0..n {
                for i in 0..j {
                    let dot: f32 = (0..m).map(|h| q[i][h] * q[j][h]).sum();
                    for h in 0..m {
                        q[j][h] -= dot * q[i][h];
                    }
                }
                let norm = q[j].iter().map(|v| v * v).sum::<f32>().sqrt();
                if norm < 0.000001 {
                    return None;
                }
                for h in 0..m {
                    q[j][h] /= norm;
                }
                for i in j..n {
                    r[j][i] = (0..m).map(|h| q[j][h] * a[i][h]).sum();
                }
            }
            let mut b = vec![0.; n];
            for i in (0..n).rev() {
                b[i] = (0..m).map(|h| q[i][h] * values[h]).sum();
                for j in ((i + 1)..n).rev() {
                    b[i] -= r[i][j] * b[j];
                }
                b[i] /= r[i][i];
            }
            Some(b[1].clamp(-max, max))
        };
        (solve(&x).unwrap_or(0.), solve(&y).unwrap_or(0.))
    }
}
