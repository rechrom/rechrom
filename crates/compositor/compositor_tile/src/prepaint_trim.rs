//! Reclamation walk for old prepaint tiles.

use crate::{
    tile_manager_policy::{
        DecideTrimPrepaintAction, TilePriorityViolatesMemoryPolicy, TrimPrepaintAction,
    },
    tile_priority::TilePriority,
    TileMemoryLimitPolicy,
};

pub trait PrepaintTrimContext {
    fn queue_is_empty(&self) -> bool;
    fn current_priority(&self) -> TilePriority;
    fn current_was_used(&self) -> bool;
    fn free_current_resource(&mut self) -> bool;
    fn clear_current_used(&mut self);
    fn pop_current(&mut self);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PrepaintTrimOutcome {
    pub candidates: usize,
    pub freed_tiles: usize,
    pub deferred_used_tiles: usize,
    pub freed_required_for_draw_tile: bool,
    pub retry_for_deferred_tiles: bool,
}

pub fn TrimPrepaintTiles<C: PrepaintTrimContext>(
    context: &mut C,
    memory_limit_policy: TileMemoryLimitPolicy,
) -> PrepaintTrimOutcome {
    let mut trace = browser_tracing::span("compositor.tile", "Compositor.Tile.TrimPrepaint");
    let mut outcome = PrepaintTrimOutcome::default();
    while !context.queue_is_empty() {
        outcome.candidates += 1;
        let priority = context.current_priority();
        match DecideTrimPrepaintAction(
            &priority,
            TilePriorityViolatesMemoryPolicy(memory_limit_policy, &priority),
            context.current_was_used(),
        ) {
            TrimPrepaintAction::Skip => {}
            TrimPrepaintAction::FreeResource => {
                outcome.freed_tiles += 1;
                outcome.freed_required_for_draw_tile |= context.free_current_resource();
            }
            TrimPrepaintAction::ClearUsedAndRetryLater => {
                outcome.deferred_used_tiles += 1;
                outcome.retry_for_deferred_tiles = true;
                context.clear_current_used();
            }
        }
        context.pop_current();
    }
    trace.set("candidates", outcome.candidates as f64);
    trace.set("freed_tiles", outcome.freed_tiles as f64);
    trace.set("deferred_used_tiles", outcome.deferred_used_tiles as f64);
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile_priority::{PriorityBin, TileResolution};

    struct Context {
        priorities: Vec<TilePriority>,
        used: Vec<bool>,
        index: usize,
        freed: Vec<usize>,
    }

    impl PrepaintTrimContext for Context {
        fn queue_is_empty(&self) -> bool {
            self.index == self.priorities.len()
        }
        fn current_priority(&self) -> TilePriority {
            self.priorities[self.index]
        }
        fn current_was_used(&self) -> bool {
            self.used[self.index]
        }
        fn free_current_resource(&mut self) -> bool {
            self.freed.push(self.index);
            self.index == 2
        }
        fn clear_current_used(&mut self) {
            self.used[self.index] = false;
        }
        fn pop_current(&mut self) {
            self.index += 1;
        }
    }

    #[test]
    fn skips_visible_defers_used_and_frees_old_prepaint() {
        let make = |bin| TilePriority::new(TileResolution::HIGH_RESOLUTION, bin, 5.0);
        let mut context = Context {
            priorities: vec![
                make(PriorityBin::NOW),
                make(PriorityBin::EVENTUALLY),
                make(PriorityBin::EVENTUALLY),
            ],
            used: vec![false, true, false],
            index: 0,
            freed: Vec::new(),
        };
        let outcome =
            TrimPrepaintTiles(&mut context, TileMemoryLimitPolicy::ALLOW_ABSOLUTE_MINIMUM);
        assert_eq!(context.freed, vec![2]);
        assert!(!context.used[1]);
        assert_eq!(outcome.candidates, 3);
        assert_eq!(outcome.freed_tiles, 1);
        assert_eq!(outcome.deferred_used_tiles, 1);
        assert!(outcome.freed_required_for_draw_tile);
        assert!(outcome.retry_for_deferred_tiles);
    }
}
