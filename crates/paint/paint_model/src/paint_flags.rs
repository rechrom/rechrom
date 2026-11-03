#![allow(non_snake_case, non_upper_case_globals)]

/// Combination of bits in Blink's `core/paint/paint_flags.h`.
pub type PaintFlags = u32;

/// Source: `core/paint/paint_flags.h:11-38`.
pub mod PaintFlag {
    use super::PaintFlags;

    pub const kNoFlag: PaintFlags = 0;
    pub const kSelectionDragImageOnly: PaintFlags = 1 << 0;
    pub const kOmitCompositingInfo: PaintFlags = 1 << 1;
    pub const kAddUrlMetadata: PaintFlags = 1 << 2;
    pub const kPaintingClipPathAsMask: PaintFlags = 1 << 3;
    pub const kPaintingResourceSubtree: PaintFlags = 1 << 4;
    pub const kPaintingSVGMask: PaintFlags = 1 << 5;
    pub const kPrivacyPreserving: PaintFlags = 1 << 6;
}

#[cfg(test)]
mod tests {
    use super::{PaintFlag, PaintFlags};

    #[test]
    fn chromium_flags_are_independent_bits() {
        let all: PaintFlags = PaintFlag::kSelectionDragImageOnly
            | PaintFlag::kOmitCompositingInfo
            | PaintFlag::kAddUrlMetadata
            | PaintFlag::kPaintingClipPathAsMask
            | PaintFlag::kPaintingResourceSubtree
            | PaintFlag::kPaintingSVGMask
            | PaintFlag::kPrivacyPreserving;
        assert_eq!(all, 0x7f);
    }
}
