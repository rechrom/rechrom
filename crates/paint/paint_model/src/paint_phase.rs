#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PaintPhase {
    kBlockBackground,
    kSelfBlockBackgroundOnly,
    kDescendantBlockBackgroundsOnly,
    kForcedColorsModeBackplate,
    kFloat,
    kForeground,
    kOutline,
    kSelfOutlineOnly,
    kDescendantOutlinesOnly,
    kOverlayOverflowControls,
    kSelectionDragImage,
    kTextClip,
    kMask,
}

impl PaintPhase {
    pub const kMax: Self = Self::kMask;
}

pub fn ShouldPaintSelfBlockBackground(phase: PaintPhase) -> bool {
    matches!(
        phase,
        PaintPhase::kBlockBackground | PaintPhase::kSelfBlockBackgroundOnly
    )
}

pub fn ShouldPaintSelfOutline(phase: PaintPhase) -> bool {
    matches!(phase, PaintPhase::kOutline | PaintPhase::kSelfOutlineOnly)
}

pub fn ShouldPaintDescendantBlockBackgrounds(phase: PaintPhase) -> bool {
    matches!(
        phase,
        PaintPhase::kBlockBackground | PaintPhase::kDescendantBlockBackgroundsOnly
    )
}

pub fn ShouldPaintDescendantOutlines(phase: PaintPhase) -> bool {
    matches!(
        phase,
        PaintPhase::kOutline | PaintPhase::kDescendantOutlinesOnly
    )
}

const _: () = assert!(
    PaintPhase::kMax as u8
        == crate::display_item_id::DisplayItemIdType::kDrawingPaintPhaseLast.0
            - crate::display_item_id::DisplayItemIdType::kDrawingPaintPhaseFirst.0
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_helpers_keep_self_and_descendant_scopes_distinct() {
        assert!(ShouldPaintSelfBlockBackground(PaintPhase::kBlockBackground));
        assert!(ShouldPaintSelfBlockBackground(
            PaintPhase::kSelfBlockBackgroundOnly
        ));
        assert!(!ShouldPaintSelfBlockBackground(
            PaintPhase::kDescendantBlockBackgroundsOnly
        ));
        assert!(ShouldPaintDescendantBlockBackgrounds(
            PaintPhase::kDescendantBlockBackgroundsOnly
        ));
        assert!(ShouldPaintSelfOutline(PaintPhase::kSelfOutlineOnly));
        assert!(ShouldPaintDescendantOutlines(
            PaintPhase::kDescendantOutlinesOnly
        ));
    }
}
