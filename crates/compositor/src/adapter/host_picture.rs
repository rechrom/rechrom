//! Borrow-only constructor inputs for the CPU type family's short Picture
//! contexts. No private host facade or owner backpointer crosses this boundary.
use super::{
    layer_tree_host_impl::{
        HostImplDelegateId, HostImplFrameSinkId, HostImplServices, HostImplTypes,
    },
    layer_tree_impl::{LayerTreeHostId, LayerTreeId, LayerTreeTypes},
};
use crate::cc::{
    debug::layer_tree_debug_state::LayerTreeDebugState,
    input::input_handler::InputHandlerView,
    scheduler::begin_frame_tracker::BeginFrameTracker,
    trees::{
        animated_paint_worklet_tracker::AnimatedPaintWorkletTracker,
        image_animation_controller::ImageAnimationController, layer_tree_impl::LayerTreeImpl,
        layer_tree_settings::LayerTreeSettings, raster_capabilities::RasterCapabilities,
    },
};
/// Source raw tree pointers retain either their actual resident slot/receiver,
/// or the selected identity whose resident fields the caller is already lending.
/// Selected is never interpreted as a missing tree.
pub enum HostPictureTreeSlot<'a, B: HostImplTypes> {
    Resident(&'a mut Option<Box<LayerTreeImpl<B>>>),
    Direct(&'a mut LayerTreeImpl<B>),
    Selected(LayerTreeId),
}
pub struct HostPictureFields<'a, B: HostImplTypes> {
    pub owner_id: LayerTreeHostId,
    pub selected_id: LayerTreeId,
    pub active: HostPictureTreeSlot<'a, B>,
    pub pending: HostPictureTreeSlot<'a, B>,
    pub recycle: HostPictureTreeSlot<'a, B>,
    pub settings: &'a LayerTreeSettings,
    pub debug_state: &'a LayerTreeDebugState,
    pub raster_caps: &'a RasterCapabilities,
    pub global_tile_state: &'a mut B::GlobalTileState,
    pub resource_pool: &'a mut Option<Box<B::ResourcePool>>,
    pub resource_provider: &'a mut Option<Box<B::ClientResourceProvider>>,
    pub layer_context: &'a mut Option<Box<<B as HostImplTypes>::LayerContext>>,
    pub latency_monitors:
        &'a std::collections::BTreeSet<super::layer_tree_host_impl::HostImplLatencyMonitorId>,
    pub events_metrics: &'a mut B::EventsMetricsManager,
    pub tile_manager: &'a mut B::TileManager,
    pub image_animation_controller: &'a mut Option<Box<ImageAnimationController>>,
    pub paint_worklet_tracker: &'a mut AnimatedPaintWorkletTracker<B>,
    pub requires_high_res_to_draw: &'a bool,
    pub external_viewport: &'a super::geometry::Rect,
    pub viewport_rect_for_tile_priority: &'a super::geometry::Rect,
    pub layer_tree_frame_sink: &'a Option<HostImplFrameSinkId>,
    pub current_begin_frame_tracker: &'a BeginFrameTracker,
    pub delegate: HostImplDelegateId,
    /// None is the source's genuinely absent input_delegate_ before creation;
    /// input-triggered draw always supplies Some(actual borrowed View).
    pub input_view: Option<InputHandlerView<'a>>,
    pub services: &'a mut dyn HostImplServices<B>,
}

/// Independent immutable platform/feature capability. It contains no cc owner,
/// tree, layer or manager; all decisions remain in the source host helpers.
pub trait HostPictureReadEnvironment:
    super::layer_tree_host_impl::HostColorContextServices
{
    fn SettingsFeatures(&self) -> &dyn super::settings::LayerTreeSettingsFeatures;
    fn IsLowEndDevice(&self) -> bool;
    fn IsCCSlimmingEnabled(&self) -> bool;
    fn IsReadyToActivate(&self, delegate: HostImplDelegateId) -> bool;
}

/// IDs and tracing are independent process capabilities. This interface cannot
/// find a compositor owner, allocate a tile, release a resource, or schedule.
pub trait HostPictureTileServices {
    fn AllocateTilingId(&mut self) -> super::tile::PictureLayerTilingId;
    fn TraceTileDeleted(
        &self,
        category: &str,
        event: &str,
        id: super::tile::TileTraceId,
        flow: &str,
    );
    fn MakeTileTrace(
        &self,
        category: &str,
        value: &mut dyn super::property_types::PropertyTrace,
        name: &str,
        id: super::tile::TileTraceId,
    );
}

/// Independent external side effects consumed by native scrollbar registration.
/// The capability owns no cc tree, layer, controller or mutator state.
pub trait HostPictureRegistrationServices<B: HostImplTypes> {
    fn PostDelayedAutoscrollTask(
        &mut self,
        owner: super::layer_tree_host_impl::HostImplWeak,
        task: super::scrollbar::AutoscrollTask,
        delay: super::time::TimeDelta,
    );
    fn ScrollAnimationAbort(
        &mut self,
        mutator: &mut <B as LayerTreeTypes>::MutatorHost,
        id: super::input_types::ElementId,
    );
}
/// Actual owner fields needed by HostImpl's registration source bodies. The
/// InputHandler controller is split from its simultaneous readonly InputView.
pub struct HostPictureRegistrationFields<'a, B: HostImplTypes> {
    pub owner: super::layer_tree_host_impl::HostImplWeak,
    pub settings: &'a LayerTreeSettings,
    pub input_controller:
        Option<&'a mut crate::cc::input::scrollbar_controller::ScrollbarController>,
    pub controllers: &'a mut std::collections::HashMap<
        super::input_types::ElementId,
        Option<Box<crate::cc::input::scrollbar_animation_controller::ScrollbarAnimationController>>,
    >,
    pub flashed: &'a mut std::collections::HashSet<super::input_types::ElementId>,
    pub previously_visible: &'a mut std::collections::BTreeSet<super::input_types::ElementId>,
    pub accumulated: &'a mut std::collections::HashMap<
        super::input_types::ElementId,
        super::geometry::Vector2dF,
    >,
    pub mutator: &'a mut <B as LayerTreeTypes>::MutatorHost,
    pub services: &'a mut dyn HostPictureRegistrationServices<B>,
}

/// Same resident registration fields before the external services are lent.
pub struct HostPictureRegistrationOwnerFields<'a, B: HostImplTypes> {
    pub owner: super::layer_tree_host_impl::HostImplWeak,
    pub settings: &'a LayerTreeSettings,
    pub input_controller:
        Option<&'a mut crate::cc::input::scrollbar_controller::ScrollbarController>,
    pub controllers: &'a mut std::collections::HashMap<
        super::input_types::ElementId,
        Option<Box<crate::cc::input::scrollbar_animation_controller::ScrollbarAnimationController>>,
    >,
    pub flashed: &'a mut std::collections::HashSet<super::input_types::ElementId>,
    pub previously_visible: &'a mut std::collections::BTreeSet<super::input_types::ElementId>,
    pub accumulated: &'a mut std::collections::HashMap<
        super::input_types::ElementId,
        super::geometry::Vector2dF,
    >,
    pub mutator: &'a mut <B as LayerTreeTypes>::MutatorHost,
}

/// External checker/invalidator calls only, with actual native identities. All
/// scheduling and invalidation decisions remain in translated source methods.
pub trait HostPictureMutationServices {
    fn RequestImplSideInvalidation(
        &mut self,
        delegate: HostImplDelegateId,
        needs_first_draw_on_activation: bool,
    );
    fn UpdateCheckerImageDecodingHints(
        &mut self,
        checker: super::tile_manager::CheckerImageTrackerId,
        modes: super::picture_layer_impl::PictureImageDecodingModeMap,
    );
}

/// Native external redraw and Viz primitives. Each owner argument is borrowed
/// only for this call; implementations must not retain compositor state.
pub trait HostPictureRedrawServices<B: HostImplTypes> {
    fn SettingsFeatures(&self) -> &dyn super::settings::LayerTreeSettingsFeatures;
    fn DelegateIsInsideDraw(&self, delegate: HostImplDelegateId) -> bool;
    fn NotifyLatencyInfoSwapPromiseMonitor(
        &mut self,
        id: super::layer_tree_host_impl::HostImplLatencyMonitorId,
    ) -> Option<Box<B::SwapPromise>>;
    fn SaveActiveEventMetrics(&mut self, events: &mut B::EventsMetricsManager);
    fn DelegateSetNeedsRedrawOnImplThread(&mut self, delegate: HostImplDelegateId);
}
pub trait HostPictureReleaseServices<
    B: HostImplTypes + super::picture_layer_impl::PictureLayerImplTypes,
>: HostPictureRedrawServices<B>
{
    fn TraceTileStateNotification(&mut self);
    fn UpdateDisplayTile(
        &mut self,
        context: &mut <B as HostImplTypes>::LayerContext,
        layer: &dyn super::layer_tree_host_impl::PictureLayerDisplayRead,
        tiling: &crate::cc::tiles::picture_layer_tiling::PictureLayerTilingState<B>,
        tile: &crate::cc::tiles::tile::Tile<B>,
        provider: &mut B::ClientResourceProvider,
        sink: HostImplFrameSinkId,
        update_damage: bool,
    );
}
