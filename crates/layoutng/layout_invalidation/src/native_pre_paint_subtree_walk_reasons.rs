#![allow(non_upper_case_globals)]

// cpp: layoutng/internal/pre_paint_subtree_walk_reasons.h:12-19
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrePaintSubtreeWalkReason {
    kEffectiveAllowedTouchAction = 0,
    kBlockingWheelEventHandler = 1,
    kSoftNavigationContext = 2,
    kContainerTimingContext = 3,
}
impl PrePaintSubtreeWalkReason {
    pub const kMinValue: Self = Self::kEffectiveAllowedTouchAction;
    pub const kMaxValue: Self = Self::kContainerTimingContext;
}

// cpp: layoutng/internal/pre_paint_subtree_walk_reasons.h:21-24
pub const kPrePaintSubtreeWalkReasonBits: u32 = PrePaintSubtreeWalkReason::kMaxValue as u32 + 1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PrePaintSubtreeWalkReasons(u8);
impl PrePaintSubtreeWalkReasons {
    pub const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn from_reason(reason: PrePaintSubtreeWalkReason) -> Self {
        Self(1 << reason as u8)
    }

    pub const fn contains(self, reason: PrePaintSubtreeWalkReason) -> bool {
        (self.0 & (1 << reason as u8)) != 0
    }

    pub const fn bits(self) -> u8 {
        self.0
    }
}
impl std::ops::BitOr for PrePaintSubtreeWalkReasons {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

// cpp: layoutng/internal/pre_paint_subtree_walk_reasons.h:26-34
#[allow(non_snake_case)]
pub const fn CrossFramePrePaintSubtreeWalkReasons(
    reasons: PrePaintSubtreeWalkReasons,
) -> PrePaintSubtreeWalkReasons {
    PrePaintSubtreeWalkReasons(
        reasons.0
            & ((1 << PrePaintSubtreeWalkReason::kEffectiveAllowedTouchAction as u8)
                | (1 << PrePaintSubtreeWalkReason::kBlockingWheelEventHandler as u8)),
    )
}
