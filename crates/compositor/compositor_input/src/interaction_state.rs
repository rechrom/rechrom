//! Input interaction-state classification kept separate from host/tree access.

use crate::{
    actively_scrolling_type::ActivelyScrollingType, input_handler_types::SnapAnimationData,
    scroll_snap_data::TargetSnapAreaElementIds,
};

pub fn HasAnimatingSnapTargets<'a>(
    animations: impl IntoIterator<Item = &'a SnapAnimationData>,
) -> bool {
    animations
        .into_iter()
        .any(|data| data.animating_snap_target_ids_ != TargetSnapAreaElementIds::default())
}

pub fn IsCurrentlyScrolling(
    has_current_scroll_node: bool,
    has_animating_snap_targets: impl FnOnce() -> bool,
) -> bool {
    has_current_scroll_node || has_animating_snap_targets()
}

/// Classifies the active scroll without eagerly reading settings or scanning
/// snap animations. These inputs can require separate host/tree borrows.
pub fn GetActivelyScrollingType(
    has_current_scroll_node: bool,
    has_last_scroll_update: bool,
    delta_consumed_for_scroll_gesture: bool,
    should_animate_scroll: impl FnOnce() -> bool,
    has_animating_snap_targets: impl FnOnce() -> bool,
) -> ActivelyScrollingType {
    if has_current_scroll_node && has_last_scroll_update && delta_consumed_for_scroll_gesture {
        if should_animate_scroll() {
            return ActivelyScrollingType::kAnimated;
        }
        return ActivelyScrollingType::kPrecise;
    }
    if has_animating_snap_targets() {
        return ActivelyScrollingType::kAnimated;
    }
    ActivelyScrollingType::kNone
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn current_precise_scroll_does_not_scan_snap_animations() {
        let snap_scanned = Cell::new(false);
        let result = GetActivelyScrollingType(
            true,
            true,
            true,
            || false,
            || {
                snap_scanned.set(true);
                true
            },
        );
        assert_eq!(result, ActivelyScrollingType::kPrecise);
        assert!(!snap_scanned.get());
    }

    #[test]
    fn snap_animation_is_active_without_current_scroll_node() {
        assert_eq!(
            GetActivelyScrollingType(false, false, false, || unreachable!(), || true),
            ActivelyScrollingType::kAnimated
        );
    }
}
