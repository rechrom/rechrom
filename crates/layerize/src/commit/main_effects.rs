//! Values the main half reports to its client from the commit protocol
//! (LayerTreeHostClient::UpdateCompositorScrollState and the
//! cc::ScrollCallbacks notifications PAC installs on the scroll tree).
//! Source: cc/trees/layer_tree_host_client.h; cc/trees/property_tree.h:583-598.
//! `CompositorScrollState` moved from compositor/src/engine.rs and
//! `PacScrollEffect` from compositor/src/runtime/pac_scroll.rs.
use super::{
    geometry::PointF, input_types::ElementId, scroll_snap_data::TargetSnapAreaElementIds,
    scroll_source_type::ScrollSourceType,
};

/// LayerTreeHostClient::UpdateCompositorScrollState, from the same commit
/// as the preceding scroll deltas. Element/source identities remain native.
#[derive(Clone, Debug)]
pub struct CompositorScrollState {
    pub is_scroll_active: bool,
    pub scroll_latched_element_id: ElementId,
    pub source_type: ScrollSourceType,
    pub done_containers: Vec<ElementId>,
}

#[derive(Clone, Debug)]
pub enum PacScrollEffect {
    DidUpdateCompositorScrollState(CompositorScrollState),
    DidCompositorScroll {
        sequence: u64,
        element: ElementId,
        offset: PointF,
        source: ScrollSourceType,
        snap: Option<TargetSnapAreaElementIds>,
    },
    DidChangeScrollbarsHidden {
        element: ElementId,
        hidden: bool,
    },
}

/// viz::LocalSurfaceId, carried by the LayerTreeHost -> Proxy requests below.
pub use compositor_values::surface_id::LocalSurfaceId;

/// The first parent LocalSurfaceId of a Page widget (Blink's
/// ParentLocalSurfaceIdAllocator::GenerateId with a new embed token,
/// components/viz/common/surfaces/parent_local_surface_id_allocator.cc),
/// which LayerTreeHost::SetViewportRectAndScale carries. Moved from
/// compositor/src/engine.rs.
pub fn NewParentLocalSurfaceId() -> LocalSurfaceId {
    LocalSurfaceId {
        parent_sequence_number: 1,
        child_sequence_number: 1,
        embed_token: super::values::CreateUnguessableToken().0,
    }
}

/// Proxy calls from LayerTreeHost (cc/trees/layer_tree_host.cc:321-337,
/// 520-547,835-849,877-880), not copied commit/tree state. They are posted in
/// source order to the impl half (ProxyMain -> ProxyImpl).
/// Moved from compositor/src/runtime/main_endpoint.rs.
pub enum MainHostEffect {
    NeedsUpdateLayers,
    NeedsCommit { urgent: bool },
    SaveActiveEventMetrics,
    SetTargetLocalSurfaceId(LocalSurfaceId),
    NewLocalSurfaceIdExpectedWhilePaused,
    UpdateDeferMainFrameUpdate,
}

/// The embedding side of the main host's Proxy endpoint: the sequence the
/// main half runs on, the posted Proxy effects and the main task runner (the
/// main AnimationHost is `MainHostInit::mutator`, S1b-5). No Host, Layer or Page reference can
/// be recovered from it. `Send + Sync`: the endpoint that holds it is read
/// from the impl thread inside the protected sequence.
pub trait MainHostEffectSink: Send + Sync {
    fn RunsTasksInCurrentSequence(&self) -> bool;
    fn PostEffect(&self, effect: MainHostEffect);
    fn TraceCommitRequest(&self, urgent: bool);
    fn MainThreadTaskRunner(&self) -> super::main_callbacks::TaskRunner;
}

/// Construction values of the main LayerTreeHost
/// (cc/trees/layer_tree_host.cc:143-182, LayerTreeHost::InitParams): the
/// global-sequence id, property-tree owner, main MutatorHost identity,
/// settings, the actual executor mode and the embedder background color
/// (WebLayerTreeView forwards it to LayerTreeHost::set_background_color).
pub struct MainHostInit<M> {
    pub id: u64,
    pub owner: super::property_tree_services::PropertyTreesOwnerId,
    pub mutator: M,
    pub settings: super::layer_tree_settings::LayerTreeSettings,
    pub threaded: bool,
    pub background_color: Option<compositor_values::color4f::Color4f>,
    /// The impl half runs on the same sequence as this host (in-process
    /// Proxy transport): protected-sequence waits
    /// return immediately, as for CompositorMode::SINGLE_THREADED.
    pub impl_on_owner_sequence: bool,
    pub sink: std::sync::Arc<dyn MainHostEffectSink>,
}
