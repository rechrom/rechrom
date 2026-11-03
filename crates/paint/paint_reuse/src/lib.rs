#![allow(non_snake_case)]

//! Retained-state comparison and reuse diagnostics.
//!
//! This package does not traverse layout, emit paint operations, mutate
//! chunks, or publish an artifact.

pub mod pre_paint_state_compare;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PaintReuseMetrics {
    pub candidates: usize,
    pub attempts: usize,
    pub reused: usize,
    pub rejected_uncacheable: usize,
    pub rejected_just_created: usize,
    pub rejected_invalid: usize,
    pub rejected_missing_id: usize,
    pub rejected_missing_previous: usize,
}

impl PaintReuseMetrics {
    pub fn hit_rate(self) -> f64 {
        if self.attempts == 0 {
            0.0
        } else {
            self.reused as f64 / self.attempts as f64
        }
    }
}
