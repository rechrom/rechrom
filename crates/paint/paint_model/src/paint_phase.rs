// cpp: paint/paint_engine.h:15-31
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
