use paint_model::PaintPhase;

// Source: core/paint/box_fragment_painter.cc:1805-1836.
pub const ATOMIC_PHASES: [PaintPhase; 5] = [
    PaintPhase::kBlockBackground,
    PaintPhase::kForcedColorsModeBackplate,
    PaintPhase::kFloat,
    PaintPhase::kForeground,
    PaintPhase::kOutline,
];

// Source: core/paint/box_fragment_painter.cc:451-455.
pub fn FloatPaintPhase(phase: PaintPhase) -> PaintPhase {
    if phase == PaintPhase::kFloat {
        PaintPhase::kForeground
    } else {
        phase
    }
}

pub fn ShouldSkipPhase(
    phase: PaintPhase,
    needs_float: bool,
    needs_outline: bool,
    needs_overflow_controls: bool,
) -> bool {
    matches!(phase, PaintPhase::kFloat) && !needs_float
        || matches!(
            phase,
            PaintPhase::kOutline | PaintPhase::kDescendantOutlinesOnly
        ) && !needs_outline
        || matches!(phase, PaintPhase::kOverlayOverflowControls) && !needs_overflow_controls
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChildPhaseAction {
    Skip,
    Recurse(PaintPhase),
    PaintAtomically,
}

pub fn ClassifyChildPhase(
    parent_phase: PaintPhase,
    is_float: bool,
    is_atomic: bool,
) -> ChildPhaseAction {
    match parent_phase {
        PaintPhase::kDescendantBlockBackgroundsOnly
        | PaintPhase::kBlockBackground
        | PaintPhase::kForcedColorsModeBackplate => {
            if !is_float && !is_atomic {
                ChildPhaseAction::Recurse(
                    if parent_phase == PaintPhase::kForcedColorsModeBackplate {
                        parent_phase
                    } else {
                        PaintPhase::kBlockBackground
                    },
                )
            } else {
                ChildPhaseAction::Skip
            }
        }
        PaintPhase::kFloat => {
            if is_float {
                ChildPhaseAction::PaintAtomically
            } else if !is_atomic {
                ChildPhaseAction::Recurse(parent_phase)
            } else {
                ChildPhaseAction::Skip
            }
        }
        PaintPhase::kForeground => {
            if !is_float && is_atomic {
                ChildPhaseAction::PaintAtomically
            } else if !is_float {
                ChildPhaseAction::Recurse(parent_phase)
            } else {
                ChildPhaseAction::Skip
            }
        }
        PaintPhase::kDescendantOutlinesOnly | PaintPhase::kOutline => {
            if !is_atomic {
                ChildPhaseAction::Recurse(PaintPhase::kOutline)
            } else {
                ChildPhaseAction::Skip
            }
        }
        PaintPhase::kSelectionDragImage | PaintPhase::kTextClip => {
            if is_atomic {
                ChildPhaseAction::PaintAtomically
            } else {
                ChildPhaseAction::Recurse(parent_phase)
            }
        }
        // Source: core/paint/box_fragment_painter.cc:756-763. This phase
        // belongs to the current self-painting layer. Its descendant layers
        // receive their own overlay-control pass, while non-layer descendants
        // paint overlay controls during their foreground pass.
        PaintPhase::kOverlayOverflowControls => ChildPhaseAction::Skip,
        _ => ChildPhaseAction::Skip,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_policy_keeps_atomic_and_float_ownership_disjoint() {
        assert_eq!(
            ClassifyChildPhase(PaintPhase::kFloat, true, false),
            ChildPhaseAction::PaintAtomically
        );
        assert_eq!(
            ClassifyChildPhase(PaintPhase::kForeground, false, true),
            ChildPhaseAction::PaintAtomically
        );
        assert_eq!(
            ClassifyChildPhase(PaintPhase::kForeground, true, true),
            ChildPhaseAction::Skip
        );
        assert_eq!(
            ATOMIC_PHASES,
            [
                PaintPhase::kBlockBackground,
                PaintPhase::kForcedColorsModeBackplate,
                PaintPhase::kFloat,
                PaintPhase::kForeground,
                PaintPhase::kOutline,
            ]
        );
        assert_eq!(
            ClassifyChildPhase(PaintPhase::kOverlayOverflowControls, false, false),
            ChildPhaseAction::Skip
        );
        assert_eq!(FloatPaintPhase(PaintPhase::kFloat), PaintPhase::kForeground);
        assert_eq!(
            FloatPaintPhase(PaintPhase::kTextClip),
            PaintPhase::kTextClip
        );
        assert_eq!(
            ClassifyChildPhase(PaintPhase::kTextClip, false, true),
            ChildPhaseAction::PaintAtomically
        );
    }
}
