//! Ordered effect-tree walk that assigns layer and surface occlusion.

use crate::occlusion_context::{
    EffectTreeLayerListIteratorPosition, EffectTreeLayerListIteratorState,
};

#[derive(Clone, Copy, Debug)]
pub struct OcclusionWalkEntry<Surface: Copy + Eq, Layer: Copy + Eq> {
    pub position: EffectTreeLayerListIteratorPosition<Surface, Layer>,
    pub current_render_surface: Option<Surface>,
}

pub trait OcclusionWalkContext {
    type Surface: Copy + Eq;
    type Layer: Copy + Eq;

    fn current_entry(&mut self) -> Option<OcclusionWalkEntry<Self::Surface, Self::Layer>>;
    fn enter(&mut self, position: &EffectTreeLayerListIteratorPosition<Self::Surface, Self::Layer>);
    fn assign_layer_occlusion(&mut self, layer: Self::Layer);
    fn assign_contributing_surface_occlusion(&mut self, surface: Self::Surface);
    fn leave(&mut self, position: &EffectTreeLayerListIteratorPosition<Self::Surface, Self::Layer>);
    fn advance(&mut self);
    fn finish_visible_region(&mut self);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OcclusionWalkStats {
    pub positions: usize,
    pub layers: usize,
    pub target_surfaces: usize,
    pub contributing_surfaces: usize,
}

pub fn RunOcclusionWalk<C: OcclusionWalkContext>(context: &mut C) -> OcclusionWalkStats {
    let mut trace = browser_tracing::span("compositor.draw", "Compositor.Draw.OcclusionWalk");
    let mut stats = OcclusionWalkStats::default();
    while let Some(entry) = context.current_entry() {
        stats.positions += 1;
        context.enter(&entry.position);
        match entry.position.state {
            EffectTreeLayerListIteratorState::kLayer => {
                stats.layers += 1;
                context.assign_layer_occlusion(
                    entry.position.current_layer.expect("layer walk position"),
                );
            }
            EffectTreeLayerListIteratorState::kTargetSurface => {
                stats.target_surfaces += 1;
            }
            EffectTreeLayerListIteratorState::kContributingSurface => {
                stats.contributing_surfaces += 1;
                context.assign_contributing_surface_occlusion(
                    entry
                        .current_render_surface
                        .expect("contributing surface walk position"),
                );
            }
        }
        context.leave(&entry.position);
        context.advance();
    }
    context.finish_visible_region();
    trace.set("positions", stats.positions as f64);
    trace.set("layers", stats.layers as f64);
    trace.set("target_surfaces", stats.target_surfaces as f64);
    trace.set("contributing_surfaces", stats.contributing_surfaces as f64);
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Context {
        entries: Vec<OcclusionWalkEntry<i32, i32>>,
        index: usize,
        events: Vec<String>,
    }

    impl OcclusionWalkContext for Context {
        type Surface = i32;
        type Layer = i32;

        fn current_entry(&mut self) -> Option<OcclusionWalkEntry<i32, i32>> {
            self.entries.get(self.index).copied()
        }
        fn enter(&mut self, _: &EffectTreeLayerListIteratorPosition<i32, i32>) {
            self.events.push("enter".into());
        }
        fn assign_layer_occlusion(&mut self, layer: i32) {
            self.events.push(format!("layer-{layer}"));
        }
        fn assign_contributing_surface_occlusion(&mut self, surface: i32) {
            self.events.push(format!("surface-{surface}"));
        }
        fn leave(&mut self, _: &EffectTreeLayerListIteratorPosition<i32, i32>) {
            self.events.push("leave".into());
        }
        fn advance(&mut self) {
            self.index += 1;
        }
        fn finish_visible_region(&mut self) {
            self.events.push("finish".into());
        }
    }

    #[test]
    fn owns_position_branching_and_visit_order() {
        let position = |state, layer| EffectTreeLayerListIteratorPosition {
            state,
            target_render_surface: 9,
            current_layer: layer,
        };
        let mut context = Context {
            entries: vec![
                OcclusionWalkEntry {
                    position: position(EffectTreeLayerListIteratorState::kLayer, Some(4)),
                    current_render_surface: None,
                },
                OcclusionWalkEntry {
                    position: position(
                        EffectTreeLayerListIteratorState::kContributingSurface,
                        None,
                    ),
                    current_render_surface: Some(7),
                },
                OcclusionWalkEntry {
                    position: position(EffectTreeLayerListIteratorState::kTargetSurface, None),
                    current_render_surface: Some(9),
                },
            ],
            index: 0,
            events: Vec::new(),
        };
        let stats = RunOcclusionWalk(&mut context);
        assert_eq!(
            context.events,
            [
                "enter",
                "layer-4",
                "leave",
                "enter",
                "surface-7",
                "leave",
                "enter",
                "leave",
                "finish",
            ]
        );
        assert_eq!(
            stats,
            OcclusionWalkStats {
                positions: 3,
                layers: 1,
                target_surfaces: 1,
                contributing_surfaces: 1,
            }
        );
    }
}
