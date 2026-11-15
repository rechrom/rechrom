// C++: foundation/graphics_types/graphics/dynamic_range_limit.h.
// CSS owns the normalized mixture value; cc/paint supplies the effective-HDR
// headroom calculation used by image playback.

// cpp: foundation/graphics_types/graphics/dynamic_range_limit.h:8-13
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DynamicRangeLimitKind {
    kStandard,
    kHigh,
    kConstrainedHigh,
}

impl DynamicRangeLimitKind {
    pub const kLast: Self = Self::kConstrainedHigh;
}

// cpp: foundation/graphics_types/graphics/dynamic_range_limit.h:17-43
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DynamicRangeLimitMixture {
    pub standard_mix: f32,
    pub constrained_high_mix: f32,
}

impl DynamicRangeLimitMixture {
    pub fn new(limit: DynamicRangeLimitKind) -> Self {
        let mut result = Self::default();
        match limit {
            DynamicRangeLimitKind::kStandard => result.standard_mix = 1.0,
            DynamicRangeLimitKind::kConstrainedHigh => result.constrained_high_mix = 1.0,
            DynamicRangeLimitKind::kHigh => {}
        }
        result
    }

    pub fn from_mix(standard_mix: f32, constrained_high_mix: f32) -> Self {
        Self {
            standard_mix,
            constrained_high_mix,
        }
    }

    /// Source: cc/paint/paint_flags.cc:254-263. CSS uses the same normalized
    /// standard/constrained/high mixture as cc::PaintFlags.
    pub fn ComputeEffectiveHdrHeadroom(self, target: f32) -> f32 {
        let high_mix = 1.0 - self.standard_mix - self.constrained_high_mix;
        self.constrained_high_mix * target.min(1.0) + high_mix * target
    }
}

pub type DynamicRangeLimit = DynamicRangeLimitMixture;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn css_mixture_matches_cc_effective_headroom() {
        assert_eq!(
            DynamicRangeLimit::from_mix(0.25, 0.5).ComputeEffectiveHdrHeadroom(4.0),
            1.5
        );
    }
}
