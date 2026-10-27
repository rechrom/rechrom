//! Cross-tree update ordering for retained compositor property trees.

pub trait PropertyTreeUpdateContext {
    fn transform_tree_needs_update(&self) -> bool;
    fn mark_clip_tree_needs_update(&mut self);
    fn mark_effect_tree_needs_update(&mut self);
    fn update_render_targets(&mut self);
    fn update_transforms(&mut self);
    fn update_effects(&mut self);
    fn update_clips(&mut self);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PropertyTreeUpdateStats {
    pub propagated_transform_dirty: bool,
    pub updated_render_targets: bool,
}

/// Updates dependent property trees in their required order. Render-target
/// topology is optional because the native main-thread path updates properties
/// without creating impl-side surfaces.
pub fn UpdatePropertyTrees<C: PropertyTreeUpdateContext>(
    context: &mut C,
    update_render_targets: bool,
) -> PropertyTreeUpdateStats {
    let mut trace = browser_tracing::span("compositor.tree", "Compositor.Tree.UpdatePropertyTrees");
    let propagate = context.transform_tree_needs_update();
    if propagate {
        context.mark_clip_tree_needs_update();
        context.mark_effect_tree_needs_update();
    }
    if update_render_targets {
        context.update_render_targets();
    }
    context.update_transforms();
    context.update_effects();
    context.update_clips();
    trace.set("propagated_transform_dirty", propagate as u8 as f64);
    trace.set("updated_render_targets", update_render_targets as u8 as f64);
    PropertyTreeUpdateStats {
        propagated_transform_dirty: propagate,
        updated_render_targets: update_render_targets,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Context {
        transform_dirty: bool,
        events: Vec<&'static str>,
    }

    impl PropertyTreeUpdateContext for Context {
        fn transform_tree_needs_update(&self) -> bool {
            self.transform_dirty
        }
        fn mark_clip_tree_needs_update(&mut self) {
            self.events.push("dirty-clip");
        }
        fn mark_effect_tree_needs_update(&mut self) {
            self.events.push("dirty-effect");
        }
        fn update_render_targets(&mut self) {
            self.events.push("targets");
        }
        fn update_transforms(&mut self) {
            self.events.push("transforms");
        }
        fn update_effects(&mut self) {
            self.events.push("effects");
        }
        fn update_clips(&mut self) {
            self.events.push("clips");
        }
    }

    #[test]
    fn propagates_transform_dirty_and_preserves_dependency_order() {
        let mut context = Context {
            transform_dirty: true,
            events: Vec::new(),
        };
        UpdatePropertyTrees(&mut context, true);
        assert_eq!(
            context.events,
            [
                "dirty-clip",
                "dirty-effect",
                "targets",
                "transforms",
                "effects",
                "clips",
            ]
        );
    }

    #[test]
    fn main_thread_path_omits_render_target_update() {
        let mut context = Context {
            transform_dirty: false,
            events: Vec::new(),
        };
        UpdatePropertyTrees(&mut context, false);
        assert_eq!(context.events, ["transforms", "effects", "clips"]);
    }
}
