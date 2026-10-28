#![allow(non_snake_case)]

//! Cache and retained-state reuse decisions.
//!
//! Reuse compares old and new producer state and returns a decision. It does
//! not traverse layout, emit paint operations, mutate chunks, or publish an
//! artifact.

pub mod pre_paint_state_compare;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheDecision {
    Reuse,
    Record,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PaintReuseMetrics {
    pub candidates: usize,
    pub attempts: usize,
    pub reused: usize,
    pub rejected_uncacheable: usize,
    pub rejected_just_created: usize,
    pub rejected_invalid: usize,
    pub rejected_geometry: usize,
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

pub fn DecideDisplayItemReuse(
    has_previous: bool,
    is_cacheable: bool,
    client_is_just_created: bool,
    invalidated: bool,
    geometry_matches: bool,
) -> CacheDecision {
    if has_previous && is_cacheable && !client_is_just_created && !invalidated && geometry_matches {
        CacheDecision::Reuse
    } else {
        CacheDecision::Record
    }
}
