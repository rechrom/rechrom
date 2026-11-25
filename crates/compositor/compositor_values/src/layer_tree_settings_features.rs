//! Live feature services read by canonical LayerTreeSettings. The embedder
//! supplies process feature values; the settings type retains every source field.
pub trait LayerTreeSettingsFeatures {
    fn UseLayerListsByDefault(&self) -> bool;
    fn TreesInViz(&self) -> bool;
    fn TreeAnimationsInViz(&self) -> bool;
    fn DeferImplInvalidation(&self) -> bool;
    fn DeferImplInvalidationFrames(&self) -> i32;
    fn ScrollEventDispatchMode(&self) -> &str;
    fn WaitForLateScrollEvents(&self) -> bool;
    fn WaitForLateScrollEventsDeadlineRatio(&self) -> f64;
    fn ImageDecodeWorkingSetBytes(&self, for_renderer: bool) -> usize;
}

/// The process feature values LayerTreeSettings reads (cc/base/features.h:
/// kUseLayerListsByDefault, kTreesInViz, kTreeAnimationsInViz,
/// kDeferImplInvalidation, kScrollEventDispatchMode,
/// kWaitForLateScrollEvents, the image decode working-set budgets), read on
/// both threads like Chromium's base::Feature state. Moved from the impl
/// HostPictureFeatures (compositor/src/runtime/host_picture_services.rs),
/// which keeps the impl-only picture features beside them.
#[derive(Clone, Debug)]
pub struct LayerTreeSettingsFeatureValues {
    pub use_layer_lists_by_default: bool,
    pub trees_in_viz: bool,
    pub tree_animations_in_viz: bool,
    pub defer_impl_invalidation: bool,
    pub defer_impl_invalidation_frames: i32,
    pub scroll_event_dispatch_mode: String,
    pub wait_for_late_scroll_events: bool,
    pub wait_for_late_scroll_events_deadline_ratio: f64,
    pub renderer_image_decode_working_set_bytes: usize,
    pub browser_image_decode_working_set_bytes: usize,
}
impl LayerTreeSettingsFeatureValues {
    /// The desktop CPU embedding's feature state. Moved from
    /// compositor/src/runtime/engine_factory.rs FactoryOptions::cpu_default.
    pub fn CpuDesktop() -> Self {
        Self {
            use_layer_lists_by_default: false,
            trees_in_viz: false,
            tree_animations_in_viz: false,
            defer_impl_invalidation: true,
            defer_impl_invalidation_frames: 4,
            scroll_event_dispatch_mode: "DispatchScrollEventsUntilDeadline".into(),
            // QueuedInputHandlerClient is the embedding's CTEQ boundary. It
            // reports queued precision-wheel updates and InputHandler drains
            // them from WillBeginImplFrame before the deadline decision.
            wait_for_late_scroll_events: true,
            wait_for_late_scroll_events_deadline_ratio: 0.333,
            renderer_image_decode_working_set_bytes: 128 * 1024 * 1024,
            browser_image_decode_working_set_bytes: 128 * 1024 * 1024,
        }
    }
}
impl LayerTreeSettingsFeatures for LayerTreeSettingsFeatureValues {
    fn UseLayerListsByDefault(&self) -> bool {
        self.use_layer_lists_by_default
    }
    fn TreesInViz(&self) -> bool {
        self.trees_in_viz
    }
    fn TreeAnimationsInViz(&self) -> bool {
        self.tree_animations_in_viz
    }
    fn DeferImplInvalidation(&self) -> bool {
        self.defer_impl_invalidation
    }
    fn DeferImplInvalidationFrames(&self) -> i32 {
        self.defer_impl_invalidation_frames
    }
    fn ScrollEventDispatchMode(&self) -> &str {
        &self.scroll_event_dispatch_mode
    }
    fn WaitForLateScrollEvents(&self) -> bool {
        self.wait_for_late_scroll_events
    }
    fn WaitForLateScrollEventsDeadlineRatio(&self) -> f64 {
        self.wait_for_late_scroll_events_deadline_ratio
    }
    fn ImageDecodeWorkingSetBytes(&self, for_renderer: bool) -> usize {
        if for_renderer {
            self.renderer_image_decode_working_set_bytes
        } else {
            self.browser_image_decode_working_set_bytes
        }
    }
}
