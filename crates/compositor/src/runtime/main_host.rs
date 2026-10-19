//! The resident layer-list main-host subset required by PAC and TreeSynchronizer.
//! Source algorithms are identified individually; this is not a Proxy/platform
//! implementation. Request/sequence effects are independent injected capabilities.
use super::{
    host_owners::{ViewTransitionCaptureCallback, ViewTransitionRequest},
    paint::CpuPaintServices,
    types::{CpuCompositorTypes, CpuSkiaBackend, MainMutatorHostIdentity, TaskRunner},
};
use crate::{
    adapter::{
        geometry::Rect,
        input_types::ElementId,
        layer_tree_host_impl::{
            host_impl_chunk_c::HostViewTransitionElementResourceRects,
            HostMetadataTransitionRequest,
        },
        layer_tree_impl::{LayerTreeViewTransitionRequest, PropertyTreesFor},
        main_layer::{
            LayerContext, MainLayerHost, MainLayerHostId, MainLayerHostRead, MainLayerHosts,
            MainLayerReadHosts, MainLayerServices,
        },
        property_tree::PropertyTreeFeatures,
        property_tree::PropertyTreesOwnerId,
    },
    cc::{
        base::protected_sequence_synchronizer::ProtectedSequenceSynchronizer,
        debug::layer_tree_debug_state::LayerTreeDebugState,
        input::layer_selection_bound::LayerSelection,
        layers::{
            layer::{Layer, LayerBorrow, LayerRef},
            layer_list_iterator::LayerListConstIterator,
        },
        trees::{
            commit_state::{CommitState, ThreadUnsafeCommitState},
            compositor_commit_data::{CompositorCommitData, ScrollUpdateInfo},
            damage_reason::DamageReason,
            layer_tree_settings::LayerTreeSettings,
            property_ids::kInvalidPropertyNodeId,
            scroll_source_type::ScrollSourceType,
            viewport_property_ids::ViewportPropertyIds,
        },
    },
};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};
type B<R> = CpuCompositorTypes<R>;
/// Source effects at LayerTreeHost.cc:321-337,520-547,835-849,877-880.
/// The endpoint schedules actual embedding work and preserves monitor/metrics
/// notifications. It never owns a Layer, PAC, or this host.
pub trait CpuMainHostEndpoint: ProtectedSequenceSynchronizer {
    fn IsMainThread(&self) -> bool;
    fn CommitRequested(&self) -> bool;
    fn SetNeedsUpdateLayers(&self);
    fn SetNeedsCommit(&self, urgent: bool);
    fn NotifyLatencyInfoSwapPromiseMonitors(&self);
    fn SaveActiveEventMetrics(&self);
    fn TraceCommitRequest(&self, urgent: bool);
    fn SetTargetLocalSurfaceId(&self, id: super::types::LocalSurfaceId);
    fn IsRenderingPaused(&self) -> bool;
    fn NotifyNewLocalSurfaceIdExpectedWhilePaused(&self);
    fn UpdateDeferMainFrameUpdate(&self);
    fn IsElementAnimating(&self, id: ElementId) -> bool;
    fn MainThreadTaskRunner(&self) -> TaskRunner;
}
pub struct CpuMainLayerHost<R: CpuSkiaBackend> {
    // Source: cc/trees/layer_tree_host.h:1180-1188. Rust fields drop forward,
    // matching the callback-before-token-map C++ reverse declaration order.
    view_transition_callbacks: BTreeMap<u32, Option<ViewTransitionCaptureCallback>>,
    view_transition_needs_new_lsid: BTreeMap<[u64; 2], bool>,
    // Source: cc/trees/layer_tree_host.h:1093-1095,1132,1146-1150.
    property_trees: PropertyTreesFor<B<R>>,
    pending: CommitState<B<R>>,
    unsafe_state: ThreadUnsafeCommitState<B<R>>,
    element_layers: HashMap<ElementId, LayerBorrow<B<R>>>,
    recording_scale_factor: f32,
    in_paint_layer_contents: bool,
    inside_will_commit: bool,
    // Source: cc/trees/layer_tree_host.h:1097-1100,1129; source external Proxy/
    // task-runner effects are lent via one independent typed capability.
    settings: LayerTreeSettings,
    id: MainLayerHostId,
    threaded: bool,
    has_copy_request: bool,
    endpoint: Arc<dyn CpuMainHostEndpoint>,
}
impl<R: CpuSkiaBackend> CpuMainLayerHost<R> {
    // Source: cc/trees/layer_tree_host.cc:143-182. The caller supplies the
    // source global-sequence ID and actual executor mode; no thread is created.
    pub fn new(
        id: MainLayerHostId,
        owner: PropertyTreesOwnerId,
        mutator: MainMutatorHostIdentity,
        settings: LayerTreeSettings,
        threaded: bool,
        endpoint: Arc<dyn CpuMainHostEndpoint>,
    ) -> Self {
        assert!(
            settings.use_layer_lists,
            "PAC main host requires native layer-list mode"
        );
        assert!(id.0 <= i32::MAX as u64, "native LayerTreeHost ID is int");
        let mut pending = CommitState::new(owner);
        pending.needs_gpu_rasterization_histogram = threaded;
        pending.needs_full_tree_sync = true;
        pending.debug_state = settings.initial_debug_state.clone();
        Self {
            view_transition_callbacks: BTreeMap::new(),
            view_transition_needs_new_lsid: BTreeMap::new(),
            property_trees: PropertyTreesFor::<B<R>>::new(owner),
            pending,
            unsafe_state: ThreadUnsafeCommitState::new(mutator),
            element_layers: HashMap::new(),
            recording_scale_factor: 1.,
            in_paint_layer_contents: false,
            inside_will_commit: false,
            settings,
            id,
            threaded,
            has_copy_request: false,
            endpoint,
        }
    }
    pub fn identity(&self) -> MainLayerHostId {
        self.id
    }

    // Source: cc/trees/property_tree_layer_list_delegate.cc:53-110.
    // The native runtime always uses Chromium's layer-list delegate. Keep the
    // operation on the resident main PropertyTrees instead of exporting a
    // copied scroll snapshot through the Engine boundary.
    fn UpdateScrollOffsetFromImpl(&mut self, update: &ScrollUpdateInfo, source: ScrollSourceType) {
        let id = update.element_id;
        let (current, transform_id, realize_on_main) = {
            let (scroll_tree, _, state) = self.property_trees.scroll_tree_read_context();
            let current = scroll_tree.current_scroll_offset(id, state);
            let Some(node) = scroll_tree.FindNodeFromElementId(id) else {
                scroll_tree.NotifyDidCompositorScroll(
                    id,
                    &(current + update.scroll_delta),
                    source,
                    &update.snap_target_element_ids,
                    state,
                );
                return;
            };
            (
                current,
                node.transform_id,
                scroll_tree.ShouldRealizeScrollsOnMain(node),
            )
        };
        let new_offset = current + update.scroll_delta;
        if !new_offset.x().is_finite() || !new_offset.y().is_finite() {
            return;
        }
        let changed = {
            let (scroll_tree, _, state) = self.property_trees.scroll_tree_with_context();
            scroll_tree.SetScrollOffset(id, &new_offset, state)
        };
        if changed && transform_id != kInvalidPropertyNodeId {
            let (transform_tree, mut context) = self.property_trees.transform_tree_with_context();
            let transform = transform_tree.MutableNode(transform_id);
            if *transform.scroll_offset() != new_offset {
                transform.SetScrollOffset(&new_offset, DamageReason::kUntracked);
                transform.needs_local_transform_update = true;
                transform.SetTransformChanged(DamageReason::kUntracked);
                transform_tree.set_needs_update(true, &mut context);
            }
        }
        if changed {
            if realize_on_main {
                <Self as MainLayerHost<B<R>>>::SetNeedsCommit(self);
            }
            <Self as MainLayerHost<B<R>>>::SetNeedsUpdateLayers(self);
        }
        let (scroll_tree, _, state) = self.property_trees.scroll_tree_read_context();
        scroll_tree.NotifyDidCompositorScroll(
            id,
            &new_offset,
            source,
            &update.snap_target_element_ids,
            state,
        );
    }

    // Source: cc/trees/layer_tree_host.cc:1207-1244. Viewport scale/control
    // deltas are applied by their respective translated paths; this resident
    // layer-list host owns the ordinary scroll and scrollbar portions.
    pub fn ApplyCompositorChanges(&mut self, data: &mut CompositorCommitData) {
        if self.unsafe_state.root_layer.is_some() {
            // Source: cc/trees/layer_tree_host.cc:1129-1201. The translated
            // commit value already carries the inner viewport element id, so
            // the layer-list host can apply it through the same delegate path
            // even when Blink did not register a separate outer viewport.
            if data.inner_viewport_scroll.element_id.IsValid() {
                self.UpdateScrollOffsetFromImpl(
                    &data.inner_viewport_scroll,
                    ScrollSourceType::kNone,
                );
            }
            for scroll in &data.scrolls {
                let source = if scroll.element_id == data.scroll_latched_element_id {
                    data.scroll_type
                } else {
                    ScrollSourceType::kNone
                };
                self.UpdateScrollOffsetFromImpl(scroll, source);
            }
            let (scroll_tree, _, state) = self.property_trees.scroll_tree_read_context();
            for scrollbar in &data.scrollbars {
                scroll_tree.NotifyDidChangeScrollbarsHidden(
                    scrollbar.element_id,
                    scrollbar.hidden,
                    state,
                );
            }
        }
    }
    pub fn pending_commit_state(&self) -> &CommitState<B<R>> {
        &self.pending
    }
    pub fn pending_commit_state_mut(&mut self) -> &mut CommitState<B<R>> {
        &mut self.pending
    }
    pub fn thread_unsafe_commit_state(&self) -> &ThreadUnsafeCommitState<B<R>> {
        &self.unsafe_state
    }
    pub fn root_layer(&self) -> Option<&LayerRef<B<R>>> {
        self.unsafe_state.root_layer.as_ref()
    }
    fn with_layer_context<T>(
        &mut self,
        services: &dyn MainLayerServices,
        derived: &(dyn CpuPaintServices<B<R>> + 'static),
        f: impl FnOnce(&mut LayerContext<'_, B<R>>) -> T,
    ) -> T {
        let mut hosts = CpuMainLayerHosts { host: self };
        let mut context = LayerContext::new(&mut hosts, services).WithDerivedServices(derived);
        f(&mut context)
    }
    /// Lends the same resident main-thread state to Chromium's commit walk.
    /// `unsafe_state` is never cloned: the synchronization context resolves
    /// every main Layer through a read-only view over this host's disjoint
    /// fields for the duration of the call.
    pub fn WithCommitContext<T>(
        &mut self,
        services: &dyn MainLayerServices,
        derived: &(dyn CpuPaintServices<B<R>> + 'static),
        trace: &dyn crate::adapter::tree_sync::SynchronizationTrace,
        f: impl FnOnce(
            &ThreadUnsafeCommitState<B<R>>,
            &mut super::sync::MainSynchronizationContext<'_, '_, B<R>>,
        ) -> T,
    ) -> T {
        let Self {
            property_trees,
            pending,
            unsafe_state,
            element_layers,
            recording_scale_factor,
            in_paint_layer_contents,
            inside_will_commit,
            settings,
            id,
            threaded,
            endpoint,
            ..
        } = self;
        let hosts = CpuMainLayerReadHosts {
            host: CpuMainLayerReadHost {
                property_trees,
                pending,
                unsafe_state,
                element_layers,
                recording_scale_factor: *recording_scale_factor,
                in_paint_layer_contents: *in_paint_layer_contents,
                inside_will_commit: *inside_will_commit,
                settings,
                id: *id,
                threaded: *threaded,
                endpoint,
            },
        };
        let mut layer = LayerContext::new_read_only(&hosts, services).WithDerivedServices(derived);
        let mut main = super::sync::MainSynchronizationContext {
            main: &mut layer,
            host: *id,
            trace,
        };
        f(unsafe_state, &mut main)
    }
    // Source: cc/trees/layer_tree_host.cc:1393-1407,1417. HUD ownership and
    // Proxy lifecycle are outside this explicitly selected layer-list owner.
    pub fn SetRootLayer(
        &mut self,
        root: Option<LayerRef<B<R>>>,
        services: &dyn MainLayerServices,
        derived: &(dyn CpuPaintServices<B<R>> + 'static),
    ) {
        if self.unsafe_state.root_layer == root {
            return;
        }
        if let Some(old) = self.unsafe_state.root_layer.as_ref().map(LayerRef::borrow) {
            self.WaitForProtectedSequenceCompletion();
            self.with_layer_context(services, derived, |c| old.SetLayerTreeHost(None, c));
        }
        self.unsafe_state.root_layer = root;
        if let Some(root) = self.unsafe_state.root_layer.as_ref().map(LayerRef::borrow) {
            let id = self.id;
            self.with_layer_context(services, derived, |c| {
                root.WithSourceRef(c, |object, c| {
                    debug_assert!(object.base().parent(c).is_none())
                });
                root.SetLayerTreeHost(Some(id), c);
            });
        }
        self.SetNeedsFullTreeSync();
    }
    // Source: cc/trees/layer_tree_host.cc:504-517. This is the source's explicit
    // commit transfer: property-tree assignment, take change state, reset, then
    // move the commit and retain only its selective persistent constructor.
    pub fn ActivateCommitState(&mut self, owner: PropertyTreesOwnerId) -> CommitState<B<R>> {
        debug_assert!(self.IsMainThread());
        self.pending.property_trees.AssignFrom(&self.property_trees);
        self.pending
            .property_trees
            .TakeChangeStateFrom(&mut self.property_trees);
        self.property_trees.ResetAllChangeTracking();
        let persistent = self.pending.CopyPersistent(owner);
        std::mem::replace(&mut self.pending, persistent)
    }

    // Source: cc/trees/layer_tree_host.cc:421-441. Raster invalidation lives
    // on the resident main-thread Layer until WillCommit snapshots it into the
    // move-only CommitState consumed by the impl tree.
    pub fn WillCommit(
        &mut self,
        services: &dyn MainLayerServices,
        derived: &(dyn CpuPaintServices<B<R>> + 'static),
    ) {
        let Some(root) = self.unsafe_state.root_layer.as_ref().map(LayerRef::borrow) else {
            self.pending.layer_update_rects.clear();
            return;
        };
        let mut update_rects = Vec::new();
        self.with_layer_context(services, derived, |context| {
            let mut iterator = LayerListConstIterator::new(Some(root), context);
            while let Some(layer) = iterator.Dereference().cloned() {
                layer.WithSource(context, |object, context| {
                    let layer = object.base();
                    let rect = *layer.update_rect(context);
                    if rect.IsEmpty() {
                        return;
                    }
                    layer.SetNeedsPushProperties(Layer::<B<R>>::kChangedGeneralProperty, context);
                    update_rects.push((layer.id(), rect));
                    layer.ResetUpdateRect(context);
                });
                iterator.Increment(context);
            }
        });
        update_rects.sort_by_key(|(id, _)| *id);
        self.pending.layer_update_rects = update_rects.into_iter().collect();
    }
    // Source: cc/trees/layer_tree_host.cc:1477-1530,1719-1771. Surface identity
    // remains an Engine-owned Viz protocol value; Page supplies no Host/tree
    // object. All Proxy side effects are emitted through the injected endpoint.
    pub fn SetViewportRectAndScale(
        &mut self,
        device_viewport_rect: Rect,
        device_scale_factor: f32,
        local_surface_id_from_parent: super::types::LocalSurfaceId,
    ) {
        use crate::adapter::unbounded::LocalSurfaceId as _;

        let previous_local_surface_id = self.pending.local_surface_id_from_parent;
        self.SetLocalSurfaceIdFromParent(local_surface_id_from_parent);

        let device_viewport_rect_changed =
            self.pending.device_viewport_rect != device_viewport_rect;
        if device_viewport_rect_changed {
            self.pending.device_viewport_rect = device_viewport_rect;
        }

        let mut painted_device_scale_factor_changed = false;
        let mut device_scale_factor_changed = false;
        if self.settings.use_painted_device_scale_factor {
            debug_assert_eq!(self.pending.device_scale_factor, 1.0);
            if self.pending.painted_device_scale_factor != device_scale_factor {
                self.pending.painted_device_scale_factor = device_scale_factor;
                painted_device_scale_factor_changed = true;
            }
        } else {
            debug_assert_eq!(self.pending.painted_device_scale_factor, 1.0);
            if self.pending.device_scale_factor != device_scale_factor {
                self.pending.device_scale_factor = device_scale_factor;
                device_scale_factor_changed = true;
            }
        }

        if local_surface_id_from_parent.is_valid()
            && previous_local_surface_id != local_surface_id_from_parent
            && device_viewport_rect_changed
        {
            self.endpoint
                .SetTargetLocalSurfaceId(local_surface_id_from_parent);
        }

        if device_viewport_rect_changed
            || painted_device_scale_factor_changed
            || device_scale_factor_changed
        {
            self.SetPropertyTreesNeedRebuild();
            self.SetNeedsCommitUrgent();
        }
    }

    // Source: cc/trees/layer_tree_host.cc:1592-1625.
    pub fn SetPageScaleFactorAndLimits(
        &mut self,
        mut page_scale_factor: f32,
        mut min_page_scale_factor: f32,
        max_page_scale_factor: f32,
    ) {
        if self.pending.page_scale_factor_limits_set
            && self.pending.page_scale_factor == page_scale_factor
            && self.pending.min_page_scale_factor == min_page_scale_factor
            && self.pending.max_page_scale_factor == max_page_scale_factor
        {
            return;
        }
        if page_scale_factor <= 0.0 {
            page_scale_factor = 1.0;
        }
        if min_page_scale_factor <= 0.0 {
            min_page_scale_factor = page_scale_factor;
        }
        debug_assert!(max_page_scale_factor >= min_page_scale_factor);
        debug_assert!(page_scale_factor >= min_page_scale_factor);
        debug_assert!(page_scale_factor <= max_page_scale_factor);
        debug_assert!(
            self.settings.is_for_scalable_page
                || page_scale_factor == self.pending.page_scale_factor
        );

        self.pending.page_scale_factor_limits_set = true;
        self.pending.page_scale_factor = page_scale_factor;
        self.pending.min_page_scale_factor = min_page_scale_factor;
        self.pending.max_page_scale_factor = max_page_scale_factor;
        self.SetPropertyTreesNeedRebuild();
        self.SetNeedsCommit();
    }

    // Source: cc/trees/layer_tree_host.cc:1719-1771.
    fn SetLocalSurfaceIdFromParent(&mut self, id: super::types::LocalSurfaceId) {
        if self.pending.local_surface_id_from_parent == id {
            return;
        }
        let current = self.pending.local_surface_id_from_parent;
        self.pending.local_surface_id_from_parent = id;
        if self.endpoint.IsRenderingPaused() {
            self.endpoint.NotifyNewLocalSurfaceIdExpectedWhilePaused();
        }
        if current.embed_token == id.embed_token
            && current.parent_sequence_number == id.parent_sequence_number
        {
            return;
        }
        self.endpoint.UpdateDeferMainFrameUpdate();
        self.SetNeedsCommit();
    }

    // Source: cc/trees/layer_tree_host.cc:1919-1922.
    fn SetPropertyTreesNeedRebuild(&mut self) {
        self.property_trees.set_needs_rebuild(true);
        self.SetNeedsUpdateLayers();
    }

    fn SetNeedsCommitUrgent(&mut self) {
        self.endpoint.TraceCommitRequest(true);
        debug_assert!(self.IsMainThread());
        self.endpoint.SetNeedsCommit(true);
        self.endpoint.NotifyLatencyInfoSwapPromiseMonitors();
        self.endpoint.SaveActiveEventMetrics();
    }
    // Source: cc/trees/layer_tree_host.cc:958-978,1093-1125,1830-1837.
    // HUD/client/benchmark timing hooks are embedding effects and surround this
    // method in EngineAssembly; all tree selection and Layer::Update dispatch
    // remains on the same resident main host here.
    pub fn UpdateLayers(
        &mut self,
        services: &dyn MainLayerServices,
        derived: &(dyn CpuPaintServices<B<R>> + 'static),
        features: &dyn PropertyTreeFeatures,
    ) -> bool {
        if self.unsafe_state.root_layer.is_none() {
            self.property_trees.clear();
            self.pending.viewport_property_ids = ViewportPropertyIds::default();
            return false;
        }

        // Source draw_property_utils.cc:1590-1605. The runtime DrawContext type
        // is used only as the canonical CPU type family for ComputeClips; no
        // impl-side tree, layer, or host object is constructed or queried.
        if self.property_trees.transform_tree().needs_update() {
            self.property_trees
                .clip_tree_mutable()
                .set_needs_update(true);
            self.property_trees
                .effect_tree_mutable()
                .set_needs_update(true);
        }
        {
            let ids = self.pending.viewport_property_ids;
            let (tree, mut context) = self.property_trees.transform_tree_with_context();
            crate::cc::trees::draw_property_utils::ComputeTransforms(tree, &ids, &mut context);
        }
        {
            let (tree, mut context) = self.property_trees.effect_tree_with_context(features);
            if tree.needs_update() {
                for id in
                    crate::cc::trees::property_ids::kContentsRootPropertyNodeId..tree.size() as i32
                {
                    tree.UpdateEffects(id, &mut context);
                }
                tree.set_needs_update(false);
            }
        }
        crate::cc::trees::draw_property_utils::ComputeClips::<super::draw::DrawContext<'_, B<R>>>(
            &mut self.property_trees,
        );

        let root = self
            .unsafe_state
            .root_layer
            .as_ref()
            .expect("source root Layer")
            .borrow();
        let root_id = root.id();
        let host_id = self.id;
        let mut updates = Vec::new();
        self.with_layer_context(services, derived, |context| {
            let mut iterator = LayerListConstIterator::new(Some(root), context);
            while let Some(layer) = iterator.Dereference().cloned() {
                layer.WithSourceRef(context, |object, context| {
                    let layer = object.base();
                    let trees = context.Host(host_id).property_trees();
                    let effect_id = layer.effect_tree_index(context);
                    let transform_id = layer.transform_tree_index(context);
                    let clip_id = layer.clip_tree_index(context);
                    let scroll_id = layer.scroll_tree_index(context);
                    trees.effect_tree().Node(effect_id);
                    trees.transform_tree().Node(transform_id);
                    trees.clip_tree().Node(clip_id);
                    trees.scroll_tree().Node(scroll_id);
                    let effect = trees.effect_tree().Node(effect_id);
                    let skip = layer.id() != root_id
                        && !(effect.HasRenderSurface() && effect.subtree_has_copy_request)
                        && (!trees
                            .transform_tree()
                            .Node(transform_id)
                            .node_and_ancestors_are_animated_or_invertible
                            || !effect.is_drawn);
                    if skip || !effect.is_drawn || !layer.draws_content(context) {
                        return;
                    }
                    if layer.bounds(context).IsEmpty() && !effect.filters.HasReferenceFilter() {
                        return;
                    }
                    // Source IsLayerBackFaceVisible returns false unconditionally
                    // for the main-thread path (draw_property_utils.cc:465-469).
                    updates.push(layer.id());
                });
                iterator.Increment(context);
            }
        });

        self.in_paint_layer_contents = true;
        let mut updated = false;
        self.with_layer_context(services, derived, |context| {
            for id in updates {
                let layer = context
                    .Host(host_id)
                    .LayerById(id)
                    .expect("update list retains a live source Layer");
                updated |= layer.Update(context);
            }
        });
        self.in_paint_layer_contents = false;
        updated
    }
    // Source: cc/trees/layer_tree_host.cc:589-604. BindOnce copies only the
    // explicit callback map argument, exactly as the source posted task does.
    pub fn NotifyTransitionRequestsFinished(
        &mut self,
        id: u32,
        rects: &HostViewTransitionElementResourceRects,
    ) {
        debug_assert!(self.IsMainThread());
        if let Some(callback) = self
            .view_transition_callbacks
            .get_mut(&id)
            .and_then(Option::take)
        {
            let rects = rects.clone();
            self.endpoint
                .MainThreadTaskRunner()
                .PostTask(Box::new(move || callback(&rects)));
            self.view_transition_callbacks.remove(&id);
        }
    }
    // Source: cc/trees/layer_tree_host.cc:1800-1806.
    fn RequestNewLocalSurfaceId(&mut self) {
        if self.pending.new_local_surface_id_request {
            return;
        }
        self.pending.new_local_surface_id_request = true;
        self.SetNeedsCommit();
    }
}
impl<R: CpuSkiaBackend> ProtectedSequenceSynchronizer for CpuMainLayerHost<R> {
    // Source: cc/trees/layer_tree_host.cc:329-337,520-524.
    fn IsOwnerThread(&self) -> bool {
        self.endpoint.IsOwnerThread()
    }
    fn InProtectedSequence(&self) -> bool {
        self.endpoint.InProtectedSequence()
    }
    fn WaitForProtectedSequenceCompletion(&self) {
        if self.threaded {
            self.endpoint.WaitForProtectedSequenceCompletion();
        }
    }
}
impl<R: CpuSkiaBackend> MainLayerHostRead<B<R>> for CpuMainLayerHost<R> {
    // Source: cc/trees/layer_tree_host.h:205-237,695-696,713-718,928-931;
    // cc/trees/layer_tree_host.cc:304-310,321-324,356-358,399-401,548-550.
    fn device_scale_factor(&self) -> f32 {
        self.pending.device_scale_factor
    }
    fn device_viewport_rect(&self) -> Rect {
        self.pending.device_viewport_rect
    }
    fn IsMainThread(&self) -> bool {
        self.endpoint.IsMainThread()
    }
    fn CommitRequested(&self) -> bool {
        debug_assert!(self.IsMainThread());
        self.endpoint.CommitRequested()
    }
    fn SourceFrameNumber(&self) -> i32 {
        self.pending.source_frame_number
    }
    fn GetId(&self) -> i32 {
        self.id.0 as i32
    }
    fn recording_scale_factor(&self) -> f32 {
        self.recording_scale_factor
    }
    fn GetSettings(&self) -> &LayerTreeSettings {
        &self.settings
    }
    fn GetDebugState(&self) -> &LayerTreeDebugState {
        &self.pending.debug_state
    }
    fn OuterViewportScrollElementId(&self) -> ElementId {
        let id = self.pending.viewport_property_ids.outer_scroll;
        if id != kInvalidPropertyNodeId {
            self.property_trees.scroll_tree().Node(id).element_id
        } else {
            ElementId::default()
        }
    }
    fn LayerByElementId(&self, id: ElementId) -> Option<LayerBorrow<B<R>>> {
        debug_assert!(self.IsMainThread());
        self.element_layers.get(&id).cloned()
    }
    fn LayerById(&self, id: i32) -> Option<LayerBorrow<B<R>>> {
        debug_assert!(self.IsMainThread());
        self.unsafe_state.LayerById(id)
    }
    fn IsUsingLayerLists(&self) -> bool {
        self.settings.use_layer_lists
    }
    fn in_paint_layer_contents(&self) -> bool {
        debug_assert!(self.IsMainThread());
        self.in_paint_layer_contents
    }
    fn in_will_commit(&self) -> bool {
        self.inside_will_commit
    }
    fn MainThreadTaskRunner(&self) -> TaskRunner {
        self.endpoint.MainThreadTaskRunner()
    }
    fn property_trees(&self) -> &PropertyTreesFor<B<R>> {
        &self.property_trees
    }
    fn IsElementAnimating(&self, id: ElementId) -> bool {
        self.endpoint.IsElementAnimating(id)
    }
}
impl<R: CpuSkiaBackend> MainLayerHost<B<R>> for CpuMainLayerHost<R> {
    // Source: cc/trees/layer_tree_host.cc:1044-1090.
    fn AddViewTransitionRequest(&mut self, mut request: Box<ViewTransitionRequest>) {
        debug_assert!(!self
            .view_transition_callbacks
            .contains_key(&request.sequence_id()));
        if let Some(callback) = request.0.TakeFinishedCallback() {
            self.view_transition_callbacks
                .insert(request.sequence_id(), Some(callback));
        }
        if request.0.maybe_cross_frame_sink() {
            let token = request.token();
            if request.IsSave()
                && !self.view_transition_needs_new_lsid.contains_key(&token)
                && self.view_transition_needs_new_lsid.len() < 100
            {
                self.view_transition_needs_new_lsid.insert(token, true);
            }
            if request.IsAnimateRenderer()
                && self.view_transition_needs_new_lsid.contains_key(&token)
            {
                if self.view_transition_needs_new_lsid[&token] {
                    self.RequestNewLocalSurfaceId();
                }
                self.view_transition_needs_new_lsid.remove(&token);
            }
            assert_ne!(self.view_transition_needs_new_lsid.len(), 100);
            if self.view_transition_needs_new_lsid.len() == 100 {
                self.view_transition_needs_new_lsid.clear();
            }
        }
        self.pending.view_transition_requests.push(Some(request));
        self.SetNeedsCommit();
    }
    // Source: cc/trees/layer_tree_host.cc:1451-1457.
    fn RegisterSelection(&mut self, selection: LayerSelection) {
        if self.pending.selection == selection {
            return;
        }
        self.pending.selection = selection;
        self.SetNeedsCommit();
    }
    // Source: cc/trees/property_tree_layer_list_delegate.cc:112-119;
    // cc/trees/layer_tree_host.cc:1420-1427. The delegate is the fixed native
    // layer-list variant, so it owns no separate host backpointer.
    fn RegisterViewportPropertyIds(&mut self, ids: ViewportPropertyIds) {
        self.pending.viewport_property_ids = ids;
        debug_assert!(
            ids.inner_scroll != kInvalidPropertyNodeId
                || (ids.outer_scroll == kInvalidPropertyNodeId
                    && ids.outer_clip == kInvalidPropertyNodeId)
        );
    }
    // Source: cc/trees/layer_tree_host.cc:835-849.
    fn SetNeedsUpdateLayers(&mut self) {
        debug_assert!(self.IsMainThread());
        self.endpoint.SetNeedsUpdateLayers();
        self.endpoint.NotifyLatencyInfoSwapPromiseMonitors();
        self.endpoint.SaveActiveEventMetrics();
    }
    fn SetNeedsCommit(&mut self) {
        self.endpoint.TraceCommitRequest(false);
        debug_assert!(self.IsMainThread());
        self.endpoint.SetNeedsCommit(false);
        self.endpoint.NotifyLatencyInfoSwapPromiseMonitors();
        self.endpoint.SaveActiveEventMetrics();
    }
    fn property_trees_mut(&mut self) -> &mut PropertyTreesFor<B<R>> {
        &mut self.property_trees
    }
    // Source: cc/trees/layer_tree_host.cc:1809-1823. allocation is the actual
    // most-derived raw pointer equivalence, not a reconstructed base owner.
    fn RegisterLayer(&mut self, layer: &Layer<B<R>>, allocation: LayerBorrow<B<R>>) {
        debug_assert!(self.IsMainThread());
        debug_assert!(!self.unsafe_state.layer_id_map.contains_key(&layer.id()));
        debug_assert!(!self.in_paint_layer_contents);
        assert!(allocation.MatchesBase(layer));
        self.unsafe_state
            .layer_id_map
            .insert(layer.id(), allocation);
    }
    fn UnregisterLayer(&mut self, layer: &Layer<B<R>>) {
        debug_assert!(self.IsMainThread());
        debug_assert!(self.unsafe_state.layer_id_map.contains_key(&layer.id()));
        debug_assert!(!self.in_paint_layer_contents);
        self.pending
            .layer_ids_that_should_push_properties
            .remove(&layer.id());
        self.unsafe_state.layer_id_map.remove(&layer.id());
    }
    // Source: cc/trees/layer_tree_host.cc:1936-1965.
    fn RegisterElement(&mut self, id: ElementId, layer: &Layer<B<R>>) {
        debug_assert!(self.IsMainThread());
        if let Some(old) = self.element_layers.get(&id) {
            if old.MatchesBase(layer) {
                return;
            }
            self.element_layers.remove(&id);
        }
        let allocation = self
            .unsafe_state
            .layer_id_map
            .get(&layer.id())
            .expect("source registered Layer allocation");
        assert!(allocation.MatchesBase(layer));
        self.element_layers.insert(id, allocation.clone());
    }
    fn UnregisterElement(&mut self, id: ElementId, layer: &Layer<B<R>>) {
        debug_assert!(self.IsMainThread());
        if !self
            .element_layers
            .get(&id)
            .is_some_and(|old| old.MatchesBase(layer))
        {
            return;
        } /* Source: property_tree_layer_list_delegate.cc:121-123 native no-op. */
        self.element_layers.remove(&id);
    }
    // Source: cc/trees/layer_tree_host.cc:1860-1863,1909-1913,2132-2135,2186-2189.
    fn AddLayerShouldPushProperties(&mut self, layer: &Layer<B<R>>) {
        self.pending
            .layer_ids_that_should_push_properties
            .insert(layer.id());
    }
    fn SetNeedsFullTreeSync(&mut self) {
        self.pending.needs_full_tree_sync = true;
        self.property_trees.set_needs_rebuild(true);
        self.SetNeedsCommit();
    }
    fn SetHasCopyRequest(&mut self, value: bool) {
        debug_assert!(self.IsMainThread());
        self.has_copy_request = value;
    }
    fn DropActiveScrollDeltaNextCommit(&mut self, id: ElementId) {
        self.pending.scrollers_clobbering_active_value.insert(id);
        self.SetNeedsCommit();
    }
}

/// Read-only phase view used only while the impl tree pulls a commit. The
/// fields are direct borrows of `CpuMainLayerHost`; this is a capability view,
/// not a copied host or a state snapshot.
struct CpuMainLayerReadHost<'a, R: CpuSkiaBackend> {
    property_trees: &'a PropertyTreesFor<B<R>>,
    pending: &'a CommitState<B<R>>,
    unsafe_state: &'a ThreadUnsafeCommitState<B<R>>,
    element_layers: &'a HashMap<ElementId, LayerBorrow<B<R>>>,
    recording_scale_factor: f32,
    in_paint_layer_contents: bool,
    inside_will_commit: bool,
    settings: &'a LayerTreeSettings,
    id: MainLayerHostId,
    threaded: bool,
    endpoint: &'a Arc<dyn CpuMainHostEndpoint>,
}
impl<R: CpuSkiaBackend> ProtectedSequenceSynchronizer for CpuMainLayerReadHost<'_, R> {
    fn IsOwnerThread(&self) -> bool {
        self.endpoint.IsOwnerThread()
    }
    fn InProtectedSequence(&self) -> bool {
        self.endpoint.InProtectedSequence()
    }
    fn WaitForProtectedSequenceCompletion(&self) {
        if self.threaded {
            self.endpoint.WaitForProtectedSequenceCompletion();
        }
    }
}
impl<R: CpuSkiaBackend> MainLayerHostRead<B<R>> for CpuMainLayerReadHost<'_, R> {
    fn device_scale_factor(&self) -> f32 {
        self.pending.device_scale_factor
    }
    fn device_viewport_rect(&self) -> Rect {
        self.pending.device_viewport_rect
    }
    fn IsMainThread(&self) -> bool {
        self.endpoint.IsMainThread()
    }
    fn CommitRequested(&self) -> bool {
        debug_assert!(self.IsMainThread());
        self.endpoint.CommitRequested()
    }
    fn SourceFrameNumber(&self) -> i32 {
        self.pending.source_frame_number
    }
    fn GetId(&self) -> i32 {
        self.id.0 as i32
    }
    fn recording_scale_factor(&self) -> f32 {
        self.recording_scale_factor
    }
    fn GetSettings(&self) -> &LayerTreeSettings {
        self.settings
    }
    fn GetDebugState(&self) -> &LayerTreeDebugState {
        &self.pending.debug_state
    }
    fn OuterViewportScrollElementId(&self) -> ElementId {
        let id = self.pending.viewport_property_ids.outer_scroll;
        if id != kInvalidPropertyNodeId {
            self.property_trees.scroll_tree().Node(id).element_id
        } else {
            ElementId::default()
        }
    }
    fn LayerByElementId(&self, id: ElementId) -> Option<LayerBorrow<B<R>>> {
        debug_assert!(self.IsMainThread());
        self.element_layers.get(&id).cloned()
    }
    fn LayerById(&self, id: i32) -> Option<LayerBorrow<B<R>>> {
        debug_assert!(self.IsMainThread());
        self.unsafe_state.LayerById(id)
    }
    fn IsUsingLayerLists(&self) -> bool {
        self.settings.use_layer_lists
    }
    fn in_paint_layer_contents(&self) -> bool {
        debug_assert!(self.IsMainThread());
        self.in_paint_layer_contents
    }
    fn in_will_commit(&self) -> bool {
        self.inside_will_commit
    }
    fn MainThreadTaskRunner(&self) -> TaskRunner {
        self.endpoint.MainThreadTaskRunner()
    }
    fn property_trees(&self) -> &PropertyTreesFor<B<R>> {
        self.property_trees
    }
    fn IsElementAnimating(&self, id: ElementId) -> bool {
        self.endpoint.IsElementAnimating(id)
    }
}
struct CpuMainLayerReadHosts<'a, R: CpuSkiaBackend> {
    host: CpuMainLayerReadHost<'a, R>,
}
impl<R: CpuSkiaBackend> MainLayerReadHosts<B<R>> for CpuMainLayerReadHosts<'_, R> {
    fn Host(&self, id: MainLayerHostId) -> &dyn MainLayerHostRead<B<R>> {
        assert_eq!(id, self.host.id);
        &self.host
    }
}

pub struct CpuMainLayerHosts<'a, R: CpuSkiaBackend> {
    pub host: &'a mut CpuMainLayerHost<R>,
}
impl<R: CpuSkiaBackend> MainLayerReadHosts<B<R>> for CpuMainLayerHosts<'_, R> {
    fn Host(&self, id: MainLayerHostId) -> &dyn MainLayerHostRead<B<R>> {
        assert_eq!(id, self.host.id);
        self.host
    }
}
impl<R: CpuSkiaBackend> MainLayerHosts<B<R>> for CpuMainLayerHosts<'_, R> {
    fn HostMut(&mut self, id: MainLayerHostId) -> &mut dyn MainLayerHost<B<R>> {
        assert_eq!(id, self.host.id);
        self.host
    }
}
impl<R: CpuSkiaBackend> Drop for CpuMainLayerHost<R> {
    fn drop(&mut self) {
        assert!(
            self.unsafe_state.root_layer.is_none(),
            "detach the native root with SetRootLayer(None, live context) before main-host destruction"
        );
    }
}
