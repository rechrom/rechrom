//! Draw-property layer selection and ordered visible-layer passes.

pub trait LayerUpdateListContext {
    type Layer: Copy;

    fn layer_count(&self) -> usize;
    fn layer_at(&self, index: usize) -> Self::Layer;
    fn ensure_valid_property_tree_indices(&self, layer: Self::Layer);
    fn is_root_layer(&self, layer: Self::Layer, main_thread: bool) -> bool;
    fn should_skip_layer(&self, layer: Self::Layer, main_thread: bool) -> bool;
    fn layer_needs_update(&self, layer: Self::Layer, main_thread: bool) -> bool;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LayerUpdateListStats {
    pub candidates: usize,
    pub skipped: usize,
    pub updates: usize,
}

pub fn FindLayersThatNeedUpdates<C: LayerUpdateListContext>(
    context: &C,
    main_thread: bool,
) -> (Vec<C::Layer>, LayerUpdateListStats) {
    let operation = if main_thread {
        "Compositor.Draw.FindMainLayersThatNeedUpdates"
    } else {
        "Compositor.Draw.FindLayersThatNeedUpdates"
    };
    let mut trace = browser_tracing::span("compositor.draw", operation);
    let mut result = Vec::new();
    let mut stats = LayerUpdateListStats {
        candidates: context.layer_count(),
        ..Default::default()
    };
    for index in 0..context.layer_count() {
        let layer = context.layer_at(index);
        if !main_thread {
            context.ensure_valid_property_tree_indices(layer);
        }
        if !context.is_root_layer(layer, main_thread)
            && context.should_skip_layer(layer, main_thread)
        {
            stats.skipped += 1;
            continue;
        }
        if context.layer_needs_update(layer, main_thread) {
            result.push(layer);
        }
    }
    stats.updates = result.len();
    trace.set("candidates", stats.candidates as f64);
    trace.set("skipped", stats.skipped as f64);
    trace.set("updates", stats.updates as f64);
    (result, stats)
}

pub trait VisibleLayerDrawContext {
    type Layer: Copy;

    fn compute_draw_transforms(&mut self, layer: Self::Layer);
    fn clear_render_surface_common_ancestor_clip(&mut self, layer: Self::Layer);
    fn compute_draw_opacity(&mut self, layer: Self::Layer);
    fn update_render_surface_common_ancestor_clip(&mut self, layer: Self::Layer);
    fn compute_clip_and_visible_rect(&mut self, layer: Self::Layer);
    fn compute_drawable_content_rect(&mut self, layer: Self::Layer);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VisibleLayerDrawStats {
    pub layers: usize,
    pub pass_visits: usize,
}

/// Runs the four ordered passes required by inter-layer clip ancestry.
pub fn ComputeDrawPropertiesOfVisibleLayers<C: VisibleLayerDrawContext>(
    context: &mut C,
    layers: &[C::Layer],
) -> VisibleLayerDrawStats {
    let mut trace = browser_tracing::span(
        "compositor.draw",
        "Compositor.Draw.ComputeVisibleLayerProperties",
    );
    for &layer in layers {
        context.compute_draw_transforms(layer);
        context.clear_render_surface_common_ancestor_clip(layer);
    }
    for &layer in layers {
        context.compute_draw_opacity(layer);
        context.update_render_surface_common_ancestor_clip(layer);
    }
    for &layer in layers {
        context.compute_clip_and_visible_rect(layer);
    }
    for &layer in layers {
        context.compute_drawable_content_rect(layer);
    }
    let stats = VisibleLayerDrawStats {
        layers: layers.len(),
        pass_visits: layers.len().saturating_mul(4),
    };
    trace.set("layers", stats.layers as f64);
    trace.set("pass_visits", stats.pass_visits as f64);
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    struct UpdateContext;

    impl LayerUpdateListContext for UpdateContext {
        type Layer = usize;
        fn layer_count(&self) -> usize {
            4
        }
        fn layer_at(&self, index: usize) -> Self::Layer {
            index
        }
        fn ensure_valid_property_tree_indices(&self, _layer: Self::Layer) {}
        fn is_root_layer(&self, layer: Self::Layer, _main_thread: bool) -> bool {
            layer == 0
        }
        fn should_skip_layer(&self, layer: Self::Layer, _main_thread: bool) -> bool {
            layer == 1
        }
        fn layer_needs_update(&self, layer: Self::Layer, _main_thread: bool) -> bool {
            layer != 2
        }
    }

    #[test]
    fn update_selection_keeps_root_and_counts_skips_separately() {
        let (layers, stats) = FindLayersThatNeedUpdates(&UpdateContext, false);
        assert_eq!(layers, vec![0, 3]);
        assert_eq!(stats.candidates, 4);
        assert_eq!(stats.skipped, 1);
        assert_eq!(stats.updates, 2);
    }

    #[derive(Default)]
    struct VisibleContext(Vec<String>);

    impl VisibleLayerDrawContext for VisibleContext {
        type Layer = i32;
        fn compute_draw_transforms(&mut self, layer: Self::Layer) {
            self.0.push(format!("transform:{layer}"));
        }
        fn clear_render_surface_common_ancestor_clip(&mut self, layer: Self::Layer) {
            self.0.push(format!("clear:{layer}"));
        }
        fn compute_draw_opacity(&mut self, layer: Self::Layer) {
            self.0.push(format!("opacity:{layer}"));
        }
        fn update_render_surface_common_ancestor_clip(&mut self, layer: Self::Layer) {
            self.0.push(format!("ancestor:{layer}"));
        }
        fn compute_clip_and_visible_rect(&mut self, layer: Self::Layer) {
            self.0.push(format!("clip:{layer}"));
        }
        fn compute_drawable_content_rect(&mut self, layer: Self::Layer) {
            self.0.push(format!("drawable:{layer}"));
        }
    }

    #[test]
    fn visible_properties_preserve_cross_layer_pass_order() {
        let mut context = VisibleContext::default();
        let stats = ComputeDrawPropertiesOfVisibleLayers(&mut context, &[4, 5]);
        assert_eq!(
            context.0,
            vec![
                "transform:4",
                "clear:4",
                "transform:5",
                "clear:5",
                "opacity:4",
                "ancestor:4",
                "opacity:5",
                "ancestor:5",
                "clip:4",
                "clip:5",
                "drawable:4",
                "drawable:5",
            ]
        );
        assert_eq!(stats.pass_visits, 8);
    }
}
