//! Per-frame count of LayoutObjects created since the last reset.
//!
//! Source: `third_party/blink/renderer/core/paint/layout_object_counter.h`.

#![allow(non_snake_case)]

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LayoutObjectCounter {
    count_: u32,
}

impl LayoutObjectCounter {
    pub fn Reset(&mut self) {
        self.count_ = 0;
    }

    pub fn Increment(&mut self) {
        // Chromium stores an unsigned count, whose overflow is defined to
        // wrap. Preserve that behavior in debug as well as release builds.
        self.count_ = self.count_.wrapping_add(1);
    }

    pub fn Count(&self) -> u32 {
        self.count_
    }
}

#[cfg(test)]
mod tests {
    use super::LayoutObjectCounter;

    #[test]
    fn increment_and_reset_match_the_source_counter() {
        let mut counter = LayoutObjectCounter::default();
        counter.Increment();
        counter.Increment();
        assert_eq!(counter.Count(), 2);
        counter.Reset();
        assert_eq!(counter.Count(), 0);
    }
}
