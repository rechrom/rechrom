// C++: foundation/graphics_types/graphics/dynamic_range_limit.h.
// ComputeEffectiveHdrHeadroom and ToString are declared here but have no
// definitions in the supplied source tree; this mapping is partial.

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
}

pub type DynamicRangeLimit = DynamicRangeLimitMixture;
