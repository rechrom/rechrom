#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

// C++ base::EnumSet stores the listed enum values in a small bitset.
// cpp: foundation/graphics_types/graphics/visual_rect_flags.h:13-56
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VisualRectFlag {
    kEdgeInclusive = 0,
    kIgnoreFilters,
    kUseGeometryMapper,
    kDontApplyMainFrameOverflowClip,
    kIgnoreLocalClipPath,
    kApplyRemoteViewportTransform,
    kUsePreciseClipPath,
    kSkipAncestorAndViewportClips,
}

impl VisualRectFlag {
    pub const kMinValue: Self = Self::kEdgeInclusive;
    pub const kMaxValue: Self = Self::kSkipAncestorAndViewportClips;
}

// cpp: foundation/graphics_types/graphics/visual_rect_flags.h:58-58
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct VisualRectFlags(u8);

impl VisualRectFlags {
    pub const fn Has(self, flag: VisualRectFlag) -> bool {
        self.0 & (1 << flag as u8) != 0
    }

    pub fn Add(&mut self, flag: VisualRectFlag) {
        self.0 |= 1 << flag as u8;
    }

    pub fn Remove(&mut self, flag: VisualRectFlag) {
        self.0 &= !(1 << flag as u8);
    }
}

impl From<VisualRectFlag> for VisualRectFlags {
    fn from(flag: VisualRectFlag) -> Self {
        Self(1 << flag as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_keep_independent_bits() {
        let mut flags = VisualRectFlags::default();
        flags.Add(VisualRectFlag::kEdgeInclusive);
        flags.Add(VisualRectFlag::kSkipAncestorAndViewportClips);
        assert!(flags.Has(VisualRectFlag::kEdgeInclusive));
        assert!(flags.Has(VisualRectFlag::kSkipAncestorAndViewportClips));
        flags.Remove(VisualRectFlag::kEdgeInclusive);
        assert!(!flags.Has(VisualRectFlag::kEdgeInclusive));
    }
}
