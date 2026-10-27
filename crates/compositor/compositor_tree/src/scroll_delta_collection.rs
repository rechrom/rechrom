//! Active-tree scroll delta collection for main-thread commits.

use crate::{
    geometry::{PointF, Vector2dF},
    synced_property::{AdditionGroup, SyncedProperty},
    ElementId,
};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

pub type SyncedScrollOffset = SyncedProperty<AdditionGroup<PointF, Vector2dF>>;
pub type SyncedScrollOffsetMap = BTreeMap<ElementId, Rc<RefCell<SyncedScrollOffset>>>;

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollDeltaUpdate<Snap> {
    pub element_id: ElementId,
    pub scroll_delta: Vector2dF,
    pub snap_target: Option<Snap>,
    pub is_inner_viewport: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScrollDeltaCollectionStats {
    pub offsets_visited: usize,
    pub interrupted_animations: usize,
    pub emitted_updates: usize,
    pub inner_viewport_updates: usize,
    pub snap_updates: usize,
}

fn PullDeltaForMainThread(
    offset: &mut SyncedScrollOffset,
    fractional: bool,
    next_bmf: bool,
) -> Vector2dF {
    if fractional {
        return offset.PullDeltaForMainThread(next_bmf);
    }
    let current = offset.Current(true);
    let rounded = PointF::new(current.x().round(), current.y().round());
    let active = offset.ActiveBase();
    let diff = active - PointF::new(active.x().round(), active.y().round());
    offset.SetCurrent(rounded + diff);
    let delta = offset.PullDeltaForMainThread(next_bmf);
    offset.SetCurrent(current);
    delta
}

/// Pulls unsent deltas in stable element-id order and decides which updates
/// belong in the commit. Interrupted animations intentionally clobber without
/// pulling their delta, while snap-only updates are still emitted.
pub fn CollectScrollDeltas<Snap: Clone>(
    offsets: &SyncedScrollOffsetMap,
    inner_viewport: ElementId,
    fractional: bool,
    snapped: &BTreeMap<ElementId, Snap>,
    mut animation_interrupted: impl FnMut(ElementId) -> bool,
    next_bmf: bool,
) -> (Vec<ScrollDeltaUpdate<Snap>>, ScrollDeltaCollectionStats) {
    let mut trace = browser_tracing::span("compositor.tree", "Compositor.Tree.CollectScrollDeltas");
    let mut updates = Vec::new();
    let mut stats = ScrollDeltaCollectionStats::default();
    for (element_id, offset) in offsets {
        stats.offsets_visited += 1;
        let interrupted = animation_interrupted(*element_id);
        stats.interrupted_animations += usize::from(interrupted);
        let delta = if interrupted {
            Vector2dF::default()
        } else {
            PullDeltaForMainThread(&mut offset.borrow_mut(), fractional, next_bmf)
        };
        let snap_target = snapped.get(element_id).cloned();
        if delta.IsZero() && snap_target.is_none() {
            continue;
        }
        let is_inner_viewport = *element_id == inner_viewport;
        stats.emitted_updates += 1;
        stats.inner_viewport_updates += usize::from(is_inner_viewport);
        stats.snap_updates += usize::from(snap_target.is_some());
        updates.push(ScrollDeltaUpdate {
            element_id: *element_id,
            scroll_delta: delta,
            snap_target,
            is_inner_viewport,
        });
    }
    trace.set("offsets_visited", stats.offsets_visited as f64);
    trace.set(
        "interrupted_animations",
        stats.interrupted_animations as f64,
    );
    trace.set("emitted_updates", stats.emitted_updates as f64);
    trace.set(
        "inner_viewport_updates",
        stats.inner_viewport_updates as f64,
    );
    trace.set("snap_updates", stats.snap_updates as f64);
    (updates, stats)
}

/// Test-only equivalent of the native collection helper: pull and mark every
/// offset without producing commit records.
pub fn PullAllScrollDeltasForTesting(offsets: &SyncedScrollOffsetMap, fractional: bool) {
    for offset in offsets.values() {
        PullDeltaForMainThread(&mut offset.borrow_mut(), fractional, false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn active_offset(current: PointF) -> Rc<RefCell<SyncedScrollOffset>> {
        let mut offset = SyncedScrollOffset::default();
        offset.PushMainToPending(PointF::default());
        offset.PushPendingToActive();
        offset.SetCurrent(current);
        Rc::new(RefCell::new(offset))
    }

    #[test]
    fn collects_in_id_order_and_keeps_snap_only_interrupted_update() {
        let first = ElementId::new(2);
        let inner = ElementId::new(4);
        let mut offsets = SyncedScrollOffsetMap::new();
        offsets.insert(inner, active_offset(PointF::new(2.25, 3.75)));
        offsets.insert(first, active_offset(PointF::new(1.25, 0.0)));
        let snapped = BTreeMap::from([(inner, 99)]);
        let (updates, stats) =
            CollectScrollDeltas(&offsets, inner, true, &snapped, |id| id == inner, false);

        assert_eq!(updates.len(), 2);
        assert_eq!(updates[0].element_id, first);
        assert_eq!(updates[0].scroll_delta, Vector2dF::new(1.25, 0.0));
        assert_eq!(updates[1].element_id, inner);
        assert_eq!(updates[1].scroll_delta, Vector2dF::default());
        assert_eq!(updates[1].snap_target, Some(99));
        assert!(updates[1].is_inner_viewport);
        assert_eq!(stats.interrupted_animations, 1);
        assert_eq!(stats.snap_updates, 1);
    }

    #[test]
    fn rounds_non_fractional_delta_without_changing_current_value() {
        let id = ElementId::new(2);
        let offset = active_offset(PointF::new(1.25, 2.75));
        let offsets = BTreeMap::from([(id, Rc::clone(&offset))]);
        let (updates, _) = CollectScrollDeltas::<()>(
            &offsets,
            ElementId::default(),
            false,
            &BTreeMap::new(),
            |_| false,
            false,
        );
        assert_eq!(updates[0].scroll_delta, Vector2dF::new(1.0, 3.0));
        assert_eq!(offset.borrow().Current(true), PointF::new(1.25, 2.75));
    }
}
