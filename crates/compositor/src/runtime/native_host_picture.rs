//! Native value and platform facets consumed by canonical PictureLayerImpl.
//!
//! Chromium correspondence:
//! - cc/layers/layer_impl.cc:96-103,576-584,999-1027
//! - cc/layers/picture_layer_impl.cc:280-296,1814-1835
//! - ui/gfx/geometry/size.cc:77-86
//! - ui/gfx/geometry/transform_util.cc:124-181
//! - components/viz/common/traced_value.cc:12-52
//!
//! These objects contain only explicit feature values and native diagnostic/
//! resource capabilities. They retain no Host, tree, layer, tile or Page.

use super::{
    client_resources::RasterClientResources,
    host::{NativeTraceValue, TraceValue},
    native_host_lifecycle::NativeHostLifecycle,
    native_services::NativeEventSink,
    tile_execution::TileExecution,
    types::{CpuCompositorTypes, CpuSkiaBackend, DeletionTrace},
};
use crate::adapter::{
    geometry::{Size, TracedValue, Vector2dF},
    host_picture::{
        HostPictureMutationServices, HostPictureReadEnvironment, HostPictureTileServices,
    },
    input_types::ElementId,
    layer_impl::{LayerImplDeletionTrace, LayerImplServices, LayerImplTraceValue},
    layer_tree_host_impl::{
        HostColorContextServices, HostImplContextProviderId, HostImplDelegateId,
        HostImplFrameSinkId, HostTraceValue,
    },
    layer_tree_impl::ResourceId,
    picture_layer_impl::PictureLayerImplServices,
    property_types::{PropertyTrace, TracedValueJSON},
    settings::LayerTreeSettingsFeatures,
    tile::{PictureLayerTilingId, TileTraceId},
    tile_based_layer_impl::TileBasedTrace,
};
use std::sync::{
    atomic::{AtomicI32, AtomicU64, Ordering},
    Arc, Mutex,
};

/// The embedder's live scheduler/frame-sink queries. An implementation may
/// share the scheduler's own atomics, but may not retain a compositor owner.
pub(crate) trait NativeHostPictureState {
    fn FrameSinkContextProvider(
        &self,
        sink: HostImplFrameSinkId,
    ) -> Option<HostImplContextProviderId>;
    fn IsReadyToActivate(&self, delegate: HostImplDelegateId) -> bool;
}

/// Explicit process feature values. There is no `Default`: Engine assembly
/// must pass the actual configured values used by this Page's compositor.
#[derive(Clone, Debug)]
pub(crate) struct NativeHostPictureFeatures {
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
    pub low_end_device: bool,
    pub cc_slimming: bool,
    pub compute_raster_translate_for_external_scale: bool,
    pub raw_draw: bool,
    pub raw_draw_tile_size_factor: f64,
}

impl LayerTreeSettingsFeatures for NativeHostPictureFeatures {
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

pub(crate) struct NativeHostPictureEnvironment {
    features: Arc<NativeHostPictureFeatures>,
    state: Arc<dyn NativeHostPictureState>,
}

impl NativeHostPictureEnvironment {
    pub(crate) fn new(
        features: Arc<NativeHostPictureFeatures>,
        state: Arc<dyn NativeHostPictureState>,
    ) -> Self {
        Self { features, state }
    }
}

impl HostColorContextServices for NativeHostPictureEnvironment {
    fn FrameSinkContextProvider(
        &self,
        sink: HostImplFrameSinkId,
    ) -> Option<HostImplContextProviderId> {
        self.state.FrameSinkContextProvider(sink)
    }
}

impl HostPictureReadEnvironment for NativeHostPictureEnvironment {
    fn SettingsFeatures(&self) -> &dyn LayerTreeSettingsFeatures {
        self.features.as_ref()
    }
    fn IsLowEndDevice(&self) -> bool {
        self.features.low_end_device
    }
    fn IsCCSlimmingEnabled(&self) -> bool {
        self.features.cc_slimming
    }
    fn IsReadyToActivate(&self, delegate: HostImplDelegateId) -> bool {
        self.state.IsReadyToActivate(delegate)
    }
}

struct NativeLayerTraceValue {
    value: TracedValueJSON,
}

impl NativeLayerTraceValue {
    fn new() -> Self {
        Self {
            value: TracedValueJSON::default(),
        }
    }
}

impl TracedValue for NativeLayerTraceValue {
    fn BeginDictionary(&mut self, name: &str) {
        self.value.BeginDictionary(name)
    }
    fn EndDictionary(&mut self) {
        self.value.EndDictionary()
    }
    fn BeginArray(&mut self, name: &str) {
        self.value.BeginArray(name)
    }
    fn EndArray(&mut self) {
        self.value.EndArray()
    }
    fn SetDouble(&mut self, name: &str, value: f64) {
        self.value.SetDouble(name, value)
    }
    fn SetInteger(&mut self, name: &str, value: i32) {
        self.value.SetInteger(name, value)
    }
    fn AppendDouble(&mut self, value: f64) {
        self.value.AppendDouble(value)
    }
    fn AppendInteger(&mut self, value: i32) {
        self.value.AppendInteger(value)
    }
}

impl PropertyTrace for NativeLayerTraceValue {
    fn BeginDictionaryUnnamed(&mut self) {
        self.value.BeginDictionaryUnnamed()
    }
    fn SetBoolean(&mut self, name: &str, value: bool) {
        self.value.SetBoolean(name, value)
    }
    fn SetString(&mut self, name: &str, value: &str) {
        self.value.SetString(name, value)
    }
    fn AppendString(&mut self, value: &str) {
        self.value.AppendString(value)
    }
}

impl LayerImplTraceValue for NativeLayerTraceValue {
    fn MakeDictIntoImplicitSnapshotWithCategory(
        &mut self,
        category: &str,
        base_type: &str,
        layer_type: &str,
        stable_id: i32,
    ) {
        // Source: components/viz/common/traced_value.cc:35-52.
        self.SetString("cat", category);
        self.SetString("base_type", base_type);
        self.SetString("id", &format!("{layer_type}/0x{:x}", stable_id as u32));
    }
    fn ElementIdInto(&mut self, element: ElementId) {
        crate::adapter::property_types::AddElementIdToTracedValue(element, self)
    }
    fn ToFormattedJSON(&self) -> String {
        self.value.ToFormattedJSON()
    }
}

impl HostTraceValue for NativeLayerTraceValue {
    fn ToFormattedJSON(&self) -> String {
        self.value.ToFormattedJSON()
    }
}

pub(crate) fn boxed_trace_value() -> TraceValue {
    let value: Box<dyn NativeTraceValue> = Box::new(NativeLayerTraceValue::new());
    TraceValue(value)
}

struct NativeLayerDeletionTrace {
    events: Arc<dyn NativeEventSink>,
}
impl LayerImplDeletionTrace for NativeLayerDeletionTrace {
    fn LayerDeleted(&self, layer_id: i32, stable_id: i32) {
        self.events.Emit(
            "compositor.layer",
            format!("deleted layer={layer_id} flow=0x{:x}", stable_id as u32),
        )
    }
}

static NEXT_STABLE_LAYER_ID: AtomicI32 = AtomicI32::new(0);
static NEXT_TILING_SET_ID: AtomicU64 = AtomicU64::new(1);

/// Process-scoped identities and tracing for resident PictureLayerTilings.
/// The object owns no tiling or TileManager and therefore cannot become a
/// second layer/tile graph. Source callers keep the actual resident owner.
pub(crate) struct NativeHostPictureTileServices {
    events: Arc<dyn NativeEventSink>,
    next_tiling_id: AtomicU64,
}

impl NativeHostPictureTileServices {
    pub(crate) fn new(events: Arc<dyn NativeEventSink>) -> Self {
        Self {
            events,
            // Source AtomicSequenceNumber values are incremented before use.
            next_tiling_id: AtomicU64::new(0),
        }
    }
}

impl HostPictureTileServices for NativeHostPictureTileServices {
    fn AllocateTilingId(&mut self) -> PictureLayerTilingId {
        let id = self.next_tiling_id.fetch_add(1, Ordering::Relaxed) + 1;
        assert_ne!(id, 0, "picture tiling identity overflow");
        PictureLayerTilingId(id)
    }

    fn TraceTileDeleted(&self, category: &str, event: &str, id: TileTraceId, flow: &str) {
        self.events.Emit(
            "compositor.tile",
            format!("category={category} event={event} id={} flow={flow}", id.0),
        )
    }

    fn MakeTileTrace(
        &self,
        category: &str,
        value: &mut dyn PropertyTrace,
        name: &str,
        id: TileTraceId,
    ) {
        // Source: components/viz/common/traced_value.cc:35-52. The implicit
        // snapshot is tracing metadata only; it does not copy compositor state.
        value.BeginDictionary(name);
        value.SetString("cat", category);
        value.SetString("id", &format!("Tile/0x{:x}", id.0));
        value.EndDictionary();
    }
}

/// External effects used by PictureLayerImpl while the actual Picture, tiling
/// and TileManager remain borrowed by the translated host call.
pub(crate) struct NativeHostPictureMutationServices<R: CpuSkiaBackend> {
    lifecycle: Arc<NativeHostLifecycle>,
    tile_execution: Arc<Mutex<TileExecution<R>>>,
}

impl<R: CpuSkiaBackend> NativeHostPictureMutationServices<R> {
    pub(crate) fn new(
        lifecycle: Arc<NativeHostLifecycle>,
        tile_execution: Arc<Mutex<TileExecution<R>>>,
    ) -> Self {
        Self {
            lifecycle,
            tile_execution,
        }
    }
}

impl<R: CpuSkiaBackend> HostPictureMutationServices for NativeHostPictureMutationServices<R> {
    fn RequestImplSideInvalidation(
        &mut self,
        delegate: HostImplDelegateId,
        needs_first_draw_on_activation: bool,
    ) {
        self.lifecycle
            .SetNeedsImplSideInvalidation(delegate, needs_first_draw_on_activation)
    }

    fn UpdateCheckerImageDecodingHints(
        &mut self,
        checker: crate::adapter::tile_manager::CheckerImageTrackerId,
        modes: crate::adapter::picture_layer_impl::PictureImageDecodingModeMap,
    ) {
        self.tile_execution
            .lock()
            .expect("native tile execution")
            .images
            .UpdateCheckerImageDecodingHints(checker, modes)
    }
}

pub(crate) struct NativePictureLayerServices<R: CpuSkiaBackend> {
    features: Arc<NativeHostPictureFeatures>,
    events: Arc<dyn NativeEventSink>,
    resources: RasterClientResources,
    marker: std::marker::PhantomData<fn() -> R>,
}

impl<R: CpuSkiaBackend> NativePictureLayerServices<R> {
    pub(crate) fn new(
        features: Arc<NativeHostPictureFeatures>,
        events: Arc<dyn NativeEventSink>,
        resources: RasterClientResources,
    ) -> Self {
        Self {
            features,
            events,
            resources,
            marker: std::marker::PhantomData,
        }
    }
}

impl<R: CpuSkiaBackend> TileBasedTrace for NativePictureLayerServices<R> {
    fn CheckerboardTrace(&self, missing_tile_count: i32) {
        self.events.HistogramCount(
            "Compositing.Renderer.CheckerboardedTiles",
            missing_tile_count as f32,
        )
    }
}

impl<R: CpuSkiaBackend> LayerImplServices<CpuCompositorTypes<R>> for NativePictureLayerServices<R> {
    fn NextStableIdSequenceValue(&self) -> i32 {
        // Source AtomicSequenceNumber starts at zero; source adds one.
        NEXT_STABLE_LAYER_ID.fetch_add(1, Ordering::Relaxed) + 1
    }
    fn DeletionTrace(&self) -> DeletionTrace {
        DeletionTrace(Arc::new(NativeLayerDeletionTrace {
            events: self.events.clone(),
        }))
    }
    fn TraceElement(&self, event: &str, element: ElementId) {
        self.events.Emit(
            "compositor.layer",
            format!("{event} element={}", element.GetInternalValue()),
        )
    }
    fn ValidateResource(&self, resource: ResourceId) {
        self.resources.ValidateResource(resource)
    }
    fn NewTracedValueJSON(&self) -> TraceValue {
        boxed_trace_value()
    }
    fn ScaleToCeiledSize(&self, size: Size, scale: f32) -> Size {
        // Source: ui/gfx/geometry/size.cc:77-86 and size_conversions.cc:15-17.
        if scale == 1.0 {
            return size;
        }
        let clamp_ceil = |value: f32| {
            if value.is_nan() {
                0
            } else {
                value.ceil().clamp(i32::MIN as f32, i32::MAX as f32) as i32
            }
        };
        Size::new(
            clamp_ceil(size.width() as f32 * scale),
            clamp_ceil(size.height() as f32 * scale),
        )
    }
    fn TryComputeTransform2dScaleComponents(
        &self,
        transform: &super::geometry::Transform<R>,
    ) -> Option<Vector2dF> {
        transform
            .native()
            .TryComputeTransform2dScaleComponents()
            .map(|value| Vector2dF::new(value.x(), value.y()))
    }
    fn ComputeApproximateMaxScale(&self, transform: &super::geometry::Transform<R>) -> f32 {
        // Source: ui/gfx/geometry/transform_util.cc:181-184.
        let unit = transform.native().MapRect(foundation::gfx::RectF::new(
            foundation::gfx::PointF::new(0.0, 0.0),
            foundation::gfx::SizeF::new(1.0, 1.0),
        ));
        unit.width().max(unit.height())
    }
}

impl<R: CpuSkiaBackend> PictureLayerImplServices<CpuCompositorTypes<R>>
    for NativePictureLayerServices<R>
{
    fn AllocateTilingSetId(
        &mut self,
    ) -> crate::cc::tiles::raster_tile_priority_queue::PictureLayerTilingSetId {
        let id = NEXT_TILING_SET_ID.fetch_add(1, Ordering::Relaxed);
        assert_ne!(id, 0, "picture tiling-set identity overflow");
        crate::cc::tiles::raster_tile_priority_queue::PictureLayerTilingSetId(id)
    }
    fn ComputeRasterTranslateForExternalScale(&self) -> bool {
        self.features.compute_raster_translate_for_external_scale
    }
    fn MissedLCPCandidateImageDeadline(&self, missed: bool) {
        self.events.HistogramCount(
            "Compositing.DecodeLCPCandidateImage.MissedDeadline",
            if missed { 1.0 } else { 0.0 },
        )
    }
    fn SetTileIDRef(&self, tile: TileTraceId, value: &mut TraceValue, name: &str) {
        // Source: components/viz/common/traced_value.cc:20-27.
        value.BeginDictionary(name);
        value.SetString("id_ref", &format!("0x{:x}", tile.0));
        value.EndDictionary();
    }
    fn IsUsingRawDraw(&self) -> bool {
        self.features.raw_draw
    }
    fn RawDrawTileSizeFactor(&self) -> f64 {
        self.features.raw_draw_tile_size_factor
    }
}
