//! The construction values of one Page compositor's main half: Blink's
//! LayerTreeView::Initialize inputs (GenerateLayerTreeSettings,
//! third_party/blink/renderer/platform/widget/compositing/
//! layer_tree_settings.cc; the main AnimationHost::CreateMainInstance,
//! cc/animation/animation_host.cc:55-57) and the Blink paint feature state of
//! its PaintArtifactCompositor. These are plain values: the Page constructs
//! the main LayerTreeHost (`LayerizeEngine`) and its ProxyMain from them
//! itself (LayerTreeHost::CreateThreaded, cc/trees/layer_tree_host.cc), and
//! ProxyMain::Start posts the `ProxyImplInit` made from them with Initialize
//! (cc/trees/proxy_main.cc:80-89,898-914). Moved from compositor/src/engine.rs
//! and compositor/src/runtime/engine_factory.rs (MainHostParamsFor): the
//! main half reads only main-side values, never the impl factory's options.
use crate::commit::{
    animation_host::AnimationHostOwner,
    layer_tree_settings::{GenerateLayerTreeSettings, LayerTreeSettings},
    main_paint_config::{CpuPaintFeatures, SkiaPaintDiagnostics},
    property_tree_services::PropertyTreeFeatureValues,
};
use compositor_values::{
    color4f::Color4f, compositor_config::CompositorConfig,
    layer_tree_settings_features::LayerTreeSettingsFeatureValues,
};
use std::sync::Arc;

pub struct MainHostParams {
    /// LayerTreeHost::id_; also the PropertyTrees owner and the ProxyImpl
    /// identity. This constructor owns identity generation; Page never
    /// invents IDs.
    pub id: u64,
    /// The first property-tree owner value ProxyMain allocates (the identity
    /// reserved after `id`).
    pub first_property_owner: u64,
    /// LayerTreeHost::InitParams::settings.
    pub settings: LayerTreeSettings,
    pub paint_features: CpuPaintFeatures,
    pub paint_diagnostics: SkiaPaintDiagnostics,
    /// LayerTreeHost::InitParams::mutator_host: the main AnimationHost
    /// instance. ProxyImpl's impl instance is created from it
    /// (`ProxyImplInit::new`).
    pub animation_host: AnimationHostOwner,
    /// Chromium's WebLayerTreeView forwards the embedder background color to
    /// LayerTreeHost. An alpha-zero root keeps popup Pages compositable over
    /// the document.
    pub background_color: Option<Color4f>,
    /// The feature values LayerTreeHost::UpdateLayers' property-tree
    /// computation reads.
    pub property_tree_features: PropertyTreeFeatureValues,
    /// The `compositor.main` SetNeedsCommit trace of the ProxyMain endpoint.
    pub commit_request_trace: Arc<dyn Fn(bool) + Send + Sync>,
}

/// The main-half construction values of one Page compositor for `config`.
pub fn CreateMainHostParams(config: &CompositorConfig) -> std::io::Result<MainHostParams> {
    if config.thread {
        return Err(std::io::Error::other(
            "CompositorConfig::thread is not a supported mode",
        ));
    }
    // One identity per main half; the ProxyImpl receives it in its init.
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let owner = NEXT.fetch_add(2, std::sync::atomic::Ordering::Relaxed);
    assert!(
        owner < i32::MAX as u64,
        "capabilities engine identity exhausted"
    );
    Ok(MainHostParams {
        id: owner,
        first_property_owner: owner + 1,
        settings: GenerateLayerTreeSettings(config, &LayerTreeSettingsFeatureValues::CpuDesktop()),
        paint_features: DefaultPaintFeatures(),
        paint_diagnostics: DefaultPaintDiagnostics(),
        // Source: cc/animation/animation_host.cc:55-57. The main instance
        // is owned by the main half (LayerizeEngine).
        animation_host: AnimationHostOwner::CreateMainInstance(),
        background_color: config.transparent_background.then_some(Color4f {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 0.0,
        }),
        property_tree_features: PropertyTreeFeatureValues::CpuDesktop(),
        // The CPU embedding's compositor event sink is the no-op sink
        // (the impl factory's NoopEventSink): the trace records nothing.
        commit_request_trace: Arc::new(|_urgent| {}),
    })
}

/// The Blink paint feature state of the main half's PAC/conversion owner.
fn DefaultPaintFeatures() -> CpuPaintFeatures {
    CpuPaintFeatures {
        cc_slimming: true,
        singular_transform_fix: true,
        merge_fixed_layers: true,
        merge_sticky_layers: true,
        raster_inducing_scroll: cfg!(any(target_os = "linux", target_os = "windows")),
        scrolling_contents_cull_rect: true,
        remove_scroll_node_workaround: true,
        selection_edge_visibility_uses_full_edge: true,
        canvas_draw_element: false,
        expensive_dchecks: cfg!(debug_assertions),
        paint_under_invalidation_checking: false,
        layer_merge_distance_limit: 0x10000000,
    }
}
fn DefaultPaintDiagnostics() -> SkiaPaintDiagnostics {
    SkiaPaintDiagnostics {
        trace: std::env::var_os("RECHROM_DIAG_STACKING").is_some(),
        verbose_level: 0,
        histogram: false,
    }
}
