//! Native output-value allocation and diagnostics for canonical draw/surfaces.
//! No layer, tree, surface or Host is retained by this independent capability.
use super::{frame::CompositorRenderPass, native_services::NativeEventSink};
use crate::{
    adapter::{
        layer_tree_impl::draw_host::LayerTreeDrawServices, property_tree::PropertyTreeFeatures,
        render_surface_impl::RenderSurfaceServices,
    },
    NativeCompositorTypes,
};
use std::sync::{
    atomic::{AtomicI32, Ordering},
    Arc,
};

/// Source: cc/layers/layer_impl.cc:102-105. The same process-wide native atomic
/// sequence supplies both LayerImpl and RenderSurface stable identities. The
/// canonical LayerImpl helper remains responsible for the source +1 mapping.
pub(crate) fn NextStableIdSequenceValue() -> i32 {
    static SEQUENCE: AtomicI32 = AtomicI32::new(0);
    let next = SEQUENCE.fetch_add(1, Ordering::Relaxed);
    assert!(next < i32::MAX, "native stable sequence exhausted");
    next
}
#[derive(Clone, Copy)]
pub(crate) struct NativeRenderConfig {
    pub size_oopif_effect_surfaces_at_external_scale: bool,
    pub allow_undamaged_nonroot_render_pass_to_skip: bool,
    pub view_transitions_new_rounding_change: bool,
    pub trace_enabled: bool,
}
#[derive(Clone)]
pub(crate) struct NativeRenderServices {
    config: NativeRenderConfig,
    events: Arc<dyn NativeEventSink>,
}
impl NativeRenderServices {
    pub(crate) fn new(config: NativeRenderConfig, events: Arc<dyn NativeEventSink>) -> Self {
        Self { config, events }
    }
}
impl PropertyTreeFeatures for NativeRenderServices {
    fn SizeOopifEffectSurfacesAtExternalScale(&self) -> bool {
        self.config.size_oopif_effect_surfaces_at_external_scale
    }
}
impl LayerTreeDrawServices for NativeRenderServices {
    fn SizeOopifEffectSurfacesAtExternalScale(&self) -> bool {
        self.config.size_oopif_effect_surfaces_at_external_scale
    }
    fn AllowUndamagedNonrootRenderPassToSkip(&self) -> bool {
        self.config.allow_undamaged_nonroot_render_pass_to_skip
    }
    fn ViewTransitionsNewRoundingChange(&self) -> bool {
        self.config.view_transitions_new_rounding_change
    }
    fn LogDoubleBlur(&self, first: &str, second: &str) {
        self.events
            .Emit("cc", format!("double blur {first} {second}"));
    }
    fn LogMaxTextureExceeded(&self, maximum: i32, width: i32, height: i32) {
        self.events.Emit(
            "cc",
            format!("render surface {width}x{height} exceeds {maximum}"),
        );
    }
    fn TraceCategoryEnabled(&self) -> bool {
        self.config.trace_enabled
    }
    fn TraceRenderSurfaceReasonCount(&self, name: &str, count: usize) {
        if self.config.trace_enabled {
            self.events
                .Emit("cc", format!("surface reason {name} count={count}"));
        }
    }
    fn TraceScope(&self, name: &str, count: Option<usize>) {
        if self.config.trace_enabled {
            self.events.Emit("cc", format!("{name} {count:?}"));
        }
    }
}
impl RenderSurfaceServices<NativeCompositorTypes> for NativeRenderServices {
    fn NextStableIdSequenceValue(&self) -> i32 {
        NextStableIdSequenceValue()
    }
    fn TraceMaskLayerGpuMemoryUsage(&self, bytes: usize) {
        if self.config.trace_enabled {
            self.events.Emit("cc", format!("mask gpu bytes={bytes}"));
        }
    }
    fn LogMaxTextureExceeded(&self, maximum: i32, width: i32, height: i32) {
        LayerTreeDrawServices::LogMaxTextureExceeded(self, maximum, width, height);
    }
    fn CreateRenderPass(
        &mut self,
        num_contributors: i32,
    ) -> Box<CompositorRenderPass<NativeCompositorTypes>> {
        // Source: components/viz/common/quads/compositor_render_pass.cc:41-44,54-55.
        // Source: components/viz/common/quads/render_pass_internal.cc:21-37.
        // Create reserves output arrays; it does not decide content or paint.
        assert!(num_contributors >= 0);
        let mut pass = CompositorRenderPass::default();
        pass.shared_quad_state_list
            .reserve(num_contributors as usize);
        pass.quad_list.reserve(128);
        Box::new(pass)
    }
}
