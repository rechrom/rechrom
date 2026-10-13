//! Additive cc::SyncedProperty, pinned Chromium
//! 6c1d401fcca5e1b0030563a90c2f2bba168e0c15 cc/base/synced_property.h.
//! Explicit batch IDs preserve reflected main/pending deltas across this
//! embedding's asynchronous messages, including stale/deferred raster jobs.
use super::ScrollNode;
use crate::MainThreadScrollDelta;
use layoutng_assembly::internal::layout_input::Offset;
use std::collections::VecDeque;

#[derive(Default)]
pub(crate) struct SyncedScrollOffset {
    root: Option<ScrollNode>,
    // cc active_delta_. Includes wheel changes AND later clamping corrections.
    active_delta: Offset,
    // Deltas sent to main but not yet reflected in the activated main base.
    // NEVER rewrite an old packet after it has been sent.
    sent: VecDeque<MainThreadScrollDelta>,
    last_sent_sequence: u64,
    last_activated_sequence: u64,
}

fn add(a: Offset, b: Offset) -> Offset {
    Offset {
        x: a.x + b.x,
        y: a.y + b.y,
    }
}
fn sub(a: Offset, b: Offset) -> Offset {
    Offset {
        x: a.x - b.x,
        y: a.y - b.y,
    }
}
fn clamp(value: Offset, root: ScrollNode) -> Offset {
    Offset {
        x: value.x.clamp(root.minimum.x, root.maximum.x),
        y: value.y.clamp(root.minimum.y, root.maximum.y),
    }
}

impl SyncedScrollOffset {
    pub fn has_unacknowledged_batch(&self, acknowledged: u64) -> bool {
        self.sent
            .back()
            .is_some_and(|batch| batch.sequence > acknowledged)
    }
    pub fn root(&self) -> Option<ScrollNode> {
        self.root
    }
    pub fn pending_delta(&self) -> Offset {
        self.active_delta
    }
    pub fn current_position(&self) -> Option<(u64, Offset)> {
        let root = self.root?;
        // activate/set_current establish the bounds invariant, but retaining
        // this clamp also protects callers supplying a refreshed ScrollNode.
        Some((root.id, clamp(add(root.committed, self.active_delta), root)))
    }
    pub fn visual_offset_for(&self, root: ScrollNode) -> Offset {
        let delta = if self
            .root
            .is_some_and(|active| active.owner_client_id == root.owner_client_id)
        {
            self.active_delta
        } else {
            Offset::default()
        };
        clamp(add(root.committed, delta), root)
    }

    /// cc::SyncedProperty::SetCurrent. The caller applies real scroll limits.
    /// Clamping corrections use this exact entry, just like ordinary input.
    pub fn set_current(&mut self, current: Offset) -> bool {
        let Some(root) = self.root else {
            return false;
        };
        assert!(current.x.is_finite() && current.y.is_finite());
        let delta = sub(current, root.committed);
        if delta == self.active_delta {
            return false;
        }
        self.active_delta = delta;
        true
    }

    /// cc::PullDeltaForMainThread / UnsentDelta. Call before producing each
    /// BeginMainFrame effect. Delivery must be reliable even without Page
    /// render demand; no packet can be silently dropped or applied twice.
    pub fn pull_delta_for_main(&mut self) -> Option<MainThreadScrollDelta> {
        self.root?;
        // main_frame_before_activation_enabled=false: a later delta batch
        // cannot race the DOM clamp/activation of the preceding main commit.
        if !self.sent.is_empty() {
            return None;
        }
        let reflected = self
            .sent
            .iter()
            .fold(Offset::default(), |sum, b| add(sum, b.delta));
        let delta = sub(self.active_delta, reflected);
        if delta == Offset::default() {
            return None;
        }
        self.last_sent_sequence = self
            .last_sent_sequence
            .checked_add(1)
            .expect("scroll synchronization sequence exhausted");
        let packet = MainThreadScrollDelta {
            sequence: self.last_sent_sequence,
            owner_client_id: self.root.expect("root checked above").owner_client_id,
            delta,
        };
        self.sent.push_back(packet);
        Some(packet)
    }

    /// cc PendingDelta = active_delta - reflected_delta_in_pending_tree.
    /// Here the immutable commit carries the last batch applied on main.
    fn delta_after_ack(&self, acknowledged: u64) -> Offset {
        debug_assert!(acknowledged <= self.last_sent_sequence);
        let reflected = self
            .sent
            .iter()
            .filter(|batch| batch.sequence <= acknowledged)
            .fold(Offset::default(), |sum, batch| add(sum, batch.delta));
        sub(self.active_delta, reflected)
    }

    /// Tile coverage must use the SAME intended position as activate. This is
    /// read-only: deferred/stale raster results must not consume acknowledgments.
    pub fn activation_position(&self, root: ScrollNode, acknowledged: u64) -> Offset {
        let delta = if self
            .root
            .is_some_and(|active| active.owner_client_id == root.owner_client_id)
        {
            self.delta_after_ack(acknowledged)
        } else {
            Offset::default()
        };
        clamp(add(root.committed, delta), root)
    }

    /// cc PushPendingToActive followed by the actual active-tree limit clamp.
    /// A clamp changes active_delta, so its correction is sent in the next
    /// pull_delta_for_main instead of mutating previously acknowledged packets.
    pub fn activate(&mut self, root: Option<ScrollNode>, acknowledged: u64) {
        let same_root = self.root.map(|r| r.owner_client_id) == root.map(|r| r.owner_client_id);
        let position = root.map(|r| self.activation_position(r, acknowledged));
        if same_root {
            debug_assert!(acknowledged >= self.last_activated_sequence);
            self.sent.retain(|batch| batch.sequence > acknowledged);
            self.last_activated_sequence = acknowledged;
        } else {
            self.sent.clear();
            // Keep the monotonically increasing packet sequence across root
            // slot replacement within a document. A root-identity guard in
            // packet routing must reject packets belonging to removed roots.
            self.last_activated_sequence = acknowledged;
        }
        self.root = root;
        self.active_delta = match (root, position) {
            (Some(root), Some(position)) => sub(position, root.committed),
            _ => Offset::default(),
        };
    }

    /// Use only when the document identity changes and old packets cannot
    /// reach the replacement Page. Gesture state must be reset by its owner.
    pub fn reset_document(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn y(y: f64) -> Offset {
        Offset { x: 0.0, y }
    }
    fn root(base: f64, maximum: f64) -> ScrollNode {
        ScrollNode {
            id: 7,
            owner_client_id: 70,
            parent_id: Some(0),
            container_rect: Default::default(),
            committed: y(base),
            minimum: y(0.0),
            maximum: y(maximum),
            user_scrollable_horizontal: false,
            user_scrollable_vertical: true,
        }
    }
    fn current(sync: &SyncedScrollOffset) -> f64 {
        sync.current_position().unwrap().1.y
    }

    #[test]
    fn authoritative_main_scroll_without_impl_delta() {
        let mut s = SyncedScrollOffset::default();
        s.activate(Some(root(0.0, 1000.0)), 0);
        s.activate(Some(root(180.0, 1000.0)), 0);
        assert_eq!(current(&s), 180.0);
        assert!(s.pull_delta_for_main().is_none());
    }
    #[test]
    fn main_acknowledges_sent_delta_but_preserves_input_during_raster() {
        let mut s = SyncedScrollOffset::default();
        s.activate(Some(root(0.0, 1000.0)), 0);
        s.set_current(y(30.0));
        let first = s.pull_delta_for_main().unwrap();
        assert_eq!(first.delta, y(30.0));
        // Main applied30, then script replaced its offset with200.
        // Meanwhile another10 pixels arrived on impl while raster was pending.
        s.set_current(y(40.0));
        assert_eq!(
            s.activation_position(root(200.0, 1000.0), first.sequence),
            y(210.0)
        );
        s.activate(Some(root(200.0, 1000.0)), first.sequence);
        assert_eq!(current(&s), 210.0);
        assert_eq!(s.pull_delta_for_main().unwrap().delta, y(10.0));
    }
    #[test]
    fn delta_batches_wait_for_preceding_commit_activation() {
        let mut s = SyncedScrollOffset::default();
        s.activate(Some(root(0., 1000.)), 0);
        s.set_current(y(20.));
        let a = s.pull_delta_for_main().unwrap();
        s.set_current(y(35.));
        assert!(s.pull_delta_for_main().is_none());
        assert!(s.has_unacknowledged_batch(0));
        assert!(!s.has_unacknowledged_batch(a.sequence));
        s.activate(Some(root(20., 1000.)), a.sequence);
        assert_eq!(current(&s), 35.);
        let b = s.pull_delta_for_main().unwrap();
        assert_eq!(b.delta, y(15.));
        s.activate(Some(root(35., 1000.)), b.sequence);
        assert_eq!(s.pending_delta(), y(0.));
        s.activate(Some(root(35., 1000.)), b.sequence);
        assert_eq!(current(&s), 35.);
    }
    #[test]
    fn range_clamp_waits_for_reflected_batch_and_does_not_replay_old_input() {
        let mut s = SyncedScrollOffset::default();
        s.activate(Some(root(0., 100.)), 0);
        s.set_current(y(80.));
        let a = s.pull_delta_for_main().unwrap();
        // The engine defers any older-ack commit until main applies this delta.
        assert!(s.has_unacknowledged_batch(0));
        assert!(s.pull_delta_for_main().is_none());
        s.activate(Some(root(30., 30.)), a.sequence);
        assert_eq!(current(&s), 30.);
        assert!(s.pull_delta_for_main().is_none());
        s.set_current(y(20.));
        s.activate(Some(root(30., 30.)), a.sequence);
        assert_eq!(current(&s), 20.);
        assert_eq!(s.pull_delta_for_main().unwrap().delta, y(-10.));
    }
    #[test]
    fn clamped_unsent_delta_does_not_resurrect_on_range_expansion() {
        let mut s = SyncedScrollOffset::default();
        s.activate(Some(root(0.0, 100.0)), 0);
        s.set_current(y(80.0));
        s.activate(Some(root(0.0, 30.0)), 0);
        s.set_current(y(20.0));
        s.activate(Some(root(0.0, 100.0)), 0);
        assert_eq!(current(&s), 20.0);
        assert_eq!(s.pull_delta_for_main().unwrap().delta, y(20.0));
    }
    #[test]
    fn native_owner_survives_property_id_rebuild_and_drops_replacement() {
        let mut s = SyncedScrollOffset::default();
        s.activate(Some(root(0., 100.)), 0);
        s.set_current(y(20.));
        let first = s.pull_delta_for_main().unwrap();
        s.set_current(y(30.));
        let rebuilt = ScrollNode {
            id: 77,
            ..root(20., 100.)
        };
        s.activate(Some(rebuilt), first.sequence);
        assert_eq!(current(&s), 30.);
        let replaced = ScrollNode {
            owner_client_id: 99,
            ..rebuilt
        };
        s.activate(Some(replaced), first.sequence);
        assert_eq!(current(&s), 20.);
        assert!(s.pull_delta_for_main().is_none());
    }
}
