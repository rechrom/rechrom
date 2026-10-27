//! Ordered retained clip-tree cache updates.

pub trait ClipTreeUpdateContext {
    fn needs_update(&self) -> bool;
    fn first_node_id(&self) -> i32;
    fn node_count(&self) -> i32;
    fn clear_node_cache(&mut self, id: i32);
    fn initialize_viewport_clip(&mut self, id: i32);
    fn update_accumulated_clip(&mut self, id: i32) -> bool;
    fn clear_needs_update(&mut self);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClipTreeUpdateStats {
    pub updated: bool,
    pub node_visits: usize,
    pub mapped_nodes: usize,
    pub failed_mappings: usize,
}

/// Rebuilds clip caches parent-before-child. The viewport node seeds the walk;
/// every later node consumes its parent's accumulated screen-space clip.
pub fn UpdateClipTree<C: ClipTreeUpdateContext>(context: &mut C) -> ClipTreeUpdateStats {
    if !context.needs_update() {
        return ClipTreeUpdateStats::default();
    }
    let mut trace = browser_tracing::span("compositor.tree", "Compositor.Tree.UpdateClips");
    let first = context.first_node_id();
    let end = context.node_count();
    let mut stats = ClipTreeUpdateStats {
        updated: true,
        ..Default::default()
    };
    for id in first..end {
        stats.node_visits += 1;
        context.clear_node_cache(id);
        if id == first {
            context.initialize_viewport_clip(id);
        } else if context.update_accumulated_clip(id) {
            stats.mapped_nodes += 1;
        } else {
            stats.failed_mappings += 1;
        }
    }
    context.clear_needs_update();
    trace.set("node_visits", stats.node_visits as f64);
    trace.set("mapped_nodes", stats.mapped_nodes as f64);
    trace.set("failed_mappings", stats.failed_mappings as f64);
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Context {
        dirty: bool,
        events: Vec<String>,
    }

    impl ClipTreeUpdateContext for Context {
        fn needs_update(&self) -> bool {
            self.dirty
        }
        fn first_node_id(&self) -> i32 {
            1
        }
        fn node_count(&self) -> i32 {
            4
        }
        fn clear_node_cache(&mut self, id: i32) {
            self.events.push(format!("clear:{id}"));
        }
        fn initialize_viewport_clip(&mut self, id: i32) {
            self.events.push(format!("viewport:{id}"));
        }
        fn update_accumulated_clip(&mut self, id: i32) -> bool {
            self.events.push(format!("map:{id}"));
            id != 3
        }
        fn clear_needs_update(&mut self) {
            self.dirty = false;
            self.events.push("clean".into());
        }
    }

    #[test]
    fn owns_viewport_seed_parent_first_walk_and_dirty_clear() {
        let mut context = Context {
            dirty: true,
            events: Vec::new(),
        };
        let stats = UpdateClipTree(&mut context);
        assert_eq!(
            context.events,
            [
                "clear:1",
                "viewport:1",
                "clear:2",
                "map:2",
                "clear:3",
                "map:3",
                "clean",
            ]
        );
        assert_eq!(stats.node_visits, 3);
        assert_eq!(stats.mapped_nodes, 1);
        assert_eq!(stats.failed_mappings, 1);
        assert!(!context.dirty);
    }

    #[test]
    fn clean_tree_does_no_work() {
        let mut context = Context {
            dirty: false,
            events: Vec::new(),
        };
        assert_eq!(UpdateClipTree(&mut context), ClipTreeUpdateStats::default());
        assert!(context.events.is_empty());
    }
}
