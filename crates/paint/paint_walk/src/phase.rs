use paint_model::PaintPhase;

pub const FOREGROUND_PHASES: [PaintPhase; 5] = [
    PaintPhase::kDescendantBlockBackgroundsOnly,
    PaintPhase::kFloat,
    PaintPhase::kForeground,
    PaintPhase::kDescendantOutlinesOnly,
    PaintPhase::kOverlayOverflowControls,
];

pub const ATOMIC_PHASES: [PaintPhase; 4] = [
    PaintPhase::kBlockBackground,
    PaintPhase::kFloat,
    PaintPhase::kForeground,
    PaintPhase::kOutline,
];

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
        PaintPhase::kDescendantBlockBackgroundsOnly | PaintPhase::kBlockBackground => {
            if !is_float && !is_atomic {
                ChildPhaseAction::Recurse(PaintPhase::kBlockBackground)
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
        PaintPhase::kOverlayOverflowControls => ChildPhaseAction::Recurse(parent_phase),
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
    }
}
