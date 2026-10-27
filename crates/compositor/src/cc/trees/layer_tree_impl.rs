use crate::adapter::{geometry::TracedValue as _, property_types::PropertyTrace as _};
use compositor_frame::swap_promise::{
    BreakSwapPromises as BreakSwapPromisesPolicy, ClearSwapPromises as ClearSwapPromisesPolicy,
    FinishSwapPromises as FinishSwapPromisesPolicy, PassSwapPromises as PassSwapPromisesPolicy,
};
// Translation status: complete
// Copyright 2011 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/trees/layer_tree_impl.h:1-69;
//         cc/trees/layer_tree_impl.cc:1-73.
// Includes/namespaces/export guards become Rust imports. External live service
// types retain their native owner; no FramePlan or copied tree state is used.
use super::{
    browser_controls_params::BrowserControlsParams,
    clip_node::ClipNode,
    damage_reason::DamageReason,
    property_ids::{kContentsRootPropertyNodeId, kInvalidPropertyNodeId, kRootPropertyNodeId},
    property_tree::{PropertyTrees, ScrollTree},
    scroll_node::ScrollNode,
    transform_node::TransformNode,
    viewport_property_ids::ViewportPropertyIds,
};
use crate::adapter::input_types::ElementId;
use crate::adapter::{
    begin_frame::BeginFrameArgs,
    geometry::{PointF, Rect, RectF, Size, SizeF, Vector2dF},
    layer_tree_impl::*,
    property_tree::{
        PropertyTreeRenderSurfaceFactory, PropertyTreeScrollLayerTree, PropertyTreesOwnerId,
    },
};
use crate::cc::{
    base::{
        region::Region,
        synced_property::{AdditionGroup, ScaleGroup, SyncedProperty},
    },
    input::{
        event_listener_properties::{
            kEventListenerClassCount, EventListenerClass, EventListenerProperties,
        },
        layer_selection_bound::LayerSelection,
        overscroll_behavior::OverscrollBehavior,
        scrollbar::ScrollbarOrientation,
    },
};
use std::{
    cell::{Ref, RefCell, RefMut},
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

// Source: cc/trees/layer_tree_impl.h:70-71.
pub type SyncedScale = SyncedProperty<ScaleGroup>;
pub type SyncedBrowserControls = SyncedProperty<AdditionGroup<f32>>;
// Source: cc/trees/layer_tree_impl.h:73-100.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum LifecycleState {
    #[default]
    kNotSyncing,
    kBeginningSync,
    kSyncedPropertyTrees,
    kSyncedLayerProperties,
}
impl LifecycleState {
    pub const kLastSyncState: Self = Self::kSyncedLayerProperties;
}
#[derive(Default)]
pub struct LayerTreeLifecycle {
    state_: LifecycleState,
}
impl LayerTreeLifecycle {
    // Source: cc/trees/layer_tree_impl.cc:138-151.
    pub fn AdvanceTo(&mut self, next: LifecycleState) {
        match next {
            LifecycleState::kNotSyncing => {
                debug_assert_eq!(self.state_, LifecycleState::kLastSyncState)
            }
            LifecycleState::kBeginningSync
            | LifecycleState::kSyncedPropertyTrees
            | LifecycleState::kSyncedLayerProperties => {
                debug_assert_eq!(self.state_ as u8 + 1, next as u8)
            }
        }
        self.state_ = next;
    }
    // Source: cc/trees/layer_tree_impl.h:91-96.
    pub fn AllowsPropertyTreeAccess(&self) -> bool {
        self.state_ == LifecycleState::kNotSyncing
            || self.state_ >= LifecycleState::kSyncedPropertyTrees
    }
    pub fn AllowsLayerPropertyAccess(&self) -> bool {
        self.state_ == LifecycleState::kNotSyncing
            || self.state_ >= LifecycleState::kSyncedLayerProperties
    }
}
// Source: cc/trees/layer_tree_impl.h:278-283.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropertyMutation {
    kTransform = 0,
    kOpacity = 1,
    kFilter = 2,
    kBackdropFilter = 3,
}
impl PropertyMutation {
    pub const kMaxValue: Self = Self::kBackdropFilter;
}
// Source: cc/trees/layer_tree_impl.h:966-969.
#[derive(Clone, Copy, Debug)]
pub struct ScrollbarLayerIds {
    pub horizontal: i32,
    pub vertical: i32,
}
impl Default for ScrollbarLayerIds {
    fn default() -> Self {
        Self {
            horizontal: -1,
            vertical: -1,
        }
    }
}

/// Live non-owning host access. Canonical host owns active/pending/recycle trees;
/// this borrow only exposes the source host services during a method call.
pub struct LayerTreeContext<'a, B: LayerTreeTypes> {
    pub host: &'a mut dyn LayerTreeHost<B>,
}
// Source: cc/trees/layer_tree_impl.h:895-1064.
pub struct LayerTreeImpl<B: LayerTreeTypes> {
    pub id: LayerTreeId,
    host_impl_: LayerTreeHostId,
    pub last_frame_begin_frame_args: BeginFrameArgs,
    created_begin_frame_args_: BeginFrameArgs,
    source_frame_number_: i32,
    trace_id_: crate::adapter::commit_types::BeginMainFrameTraceId,
    hud_layer_: Option<i32>,
    property_trees_: PropertyTreesFor<B>,
    background_color_: Color4f,
    viewport_property_ids_: ViewportPropertyIds,
    selection_: LayerSelection,
    page_scale_factor_: Rc<RefCell<SyncedScale>>,
    page_scale_factor_limits_set_: bool,
    min_page_scale_factor_: f32,
    max_page_scale_factor_: f32,
    external_page_scale_factor_: f32,
    device_scale_factor_: f32,
    painted_device_scale_factor_: f32,
    display_color_spaces_: B::DisplayColorSpaces,
    local_surface_id_from_parent_: B::LocalSurfaceId,
    new_local_surface_id_request_: bool,
    needs_update_draw_properties_: bool,
    needs_update_tiles_: bool,
    needs_full_tree_sync_: bool,
    needs_surface_ranges_sync_: bool,
    next_activation_forces_redraw_: bool,
    handle_visibility_changed_: bool,
    have_scroll_event_handlers_: bool,
    did_raster_inducing_scroll_: bool,
    device_viewport_rect_changed_: bool,
    force_send_metadata_request_: bool,
    is_animating_hud_contents_: bool,
    property_change_forces_commit_criteria_: PropertyChangeForcesCommitCriteria,
    device_viewport_rect_: Rect,
    max_safe_area_inset_bottom_: f32,
    layer_list_: B::LayerList,
    element_id_to_scrollbar_layer_ids_: BTreeMap<ElementId, ScrollbarLayerIds>,
    show_scrollbar_requests_: BTreeSet<ElementId>,
    surface_layer_ranges_: BTreeSet<B::SurfaceRange>,
    render_surface_list_: Vec<i32>,
    unoccluded_screen_space_region_: Region<B::RegionBackend>,
    swap_promise_list_: Vec<Option<Box<B::SwapPromise>>>,
    pinned_swap_promise_list_: Vec<Option<Box<B::SwapPromise>>>,
    ui_resource_request_queue_: Vec<B::UIResourceRequest>,
    event_listener_properties_: [EventListenerProperties; kEventListenerClassCount],
    browser_controls_params_: BrowserControlsParams,
    overscroll_behavior_: OverscrollBehavior,
    top_controls_shown_ratio_: Rc<RefCell<SyncedBrowserControls>>,
    bottom_controls_shown_ratio_: Rc<RefCell<SyncedBrowserControls>>,
    pending_page_scale_animation_:
        Option<Box<crate::cc::input::page_scale_animation::PendingPageScaleAnimation>>,
    lifecycle_: LayerTreeLifecycle,
    display_transform_hint_: OverlayTransform,
    // Rust's forward field drop preserves C++ reverse-member callback
    // cancellation by listing this owning pair in reverse source order.
    // Source: cc/trees/layer_tree_impl.h:1044-1045.
    successful_presentation_callbacks_: Vec<B::SuccessfulPresentationCallback>,
    // Source: cc/trees/layer_tree_impl.h:1043.
    presentation_callbacks_: Vec<B::PresentationCallback>,
    events_metrics_from_main_thread_: Vec<B::EventMetrics>,
    event_metrics_from_raster_thread_: Vec<B::EventMetrics>,
    delegated_ink_metadata_: Option<Box<B::DelegatedInkMetadata>>,
    view_transition_requests_: Vec<Box<B::ViewTransitionRequest>>,
    screenshot_destination_: UnguessableToken,
    primary_main_frame_item_sequence_number_: i64,
    discardable_image_map_updater_: Option<DiscardableImageMapUpdaterMarker>,
}
impl<B: LayerTreeTypes> LayerTreeImpl<B> {
    // Source: cc/trees/layer_tree_impl.cc:154-174; cc/trees/layer_tree_impl.h:895-1064.
    pub fn new(
        id: LayerTreeId,
        property_owner: PropertyTreesOwnerId,
        host: LayerTreeHostId,
        begin_frame_args: BeginFrameArgs,
        page_scale_factor: Rc<RefCell<SyncedScale>>,
        top_controls_shown_ratio: Rc<RefCell<SyncedBrowserControls>>,
        bottom_controls_shown_ratio: Rc<RefCell<SyncedBrowserControls>>,
    ) -> Self {
        let mut tree = Self {
            id,
            host_impl_: host,
            last_frame_begin_frame_args: BeginFrameArgs::default(),
            created_begin_frame_args_: begin_frame_args,
            source_frame_number_: -1,
            trace_id_: crate::adapter::commit_types::BeginMainFrameTraceId(0),
            hud_layer_: None,
            property_trees_: PropertyTrees::new(property_owner),
            background_color_: Color4f::default(),
            viewport_property_ids_: ViewportPropertyIds::default(),
            selection_: LayerSelection::default(),
            page_scale_factor_: page_scale_factor,
            page_scale_factor_limits_set_: false,
            min_page_scale_factor_: 1.0,
            max_page_scale_factor_: 1.0,
            external_page_scale_factor_: 1.0,
            device_scale_factor_: 1.0,
            painted_device_scale_factor_: 1.0,
            display_color_spaces_: B::DisplayColorSpaces::default(),
            local_surface_id_from_parent_: B::LocalSurfaceId::default(),
            new_local_surface_id_request_: false,
            needs_update_draw_properties_: true,
            needs_update_tiles_: false,
            needs_full_tree_sync_: true,
            needs_surface_ranges_sync_: false,
            next_activation_forces_redraw_: false,
            handle_visibility_changed_: false,
            have_scroll_event_handlers_: false,
            did_raster_inducing_scroll_: false,
            device_viewport_rect_changed_: false,
            force_send_metadata_request_: false,
            is_animating_hud_contents_: false,
            property_change_forces_commit_criteria_: PropertyChangeForcesCommitCriteria::kNone,
            device_viewport_rect_: Rect::default(),
            max_safe_area_inset_bottom_: 0.0,
            layer_list_: B::LayerList::default(),
            element_id_to_scrollbar_layer_ids_: BTreeMap::new(),
            show_scrollbar_requests_: BTreeSet::new(),
            surface_layer_ranges_: BTreeSet::new(),
            render_surface_list_: Vec::new(),
            unoccluded_screen_space_region_: Region::default(),
            swap_promise_list_: Vec::new(),
            pinned_swap_promise_list_: Vec::new(),
            ui_resource_request_queue_: Vec::new(),
            event_listener_properties_: [EventListenerProperties::kNone; kEventListenerClassCount],
            browser_controls_params_: BrowserControlsParams::default(),
            overscroll_behavior_: OverscrollBehavior::default(),
            top_controls_shown_ratio_: top_controls_shown_ratio,
            bottom_controls_shown_ratio_: bottom_controls_shown_ratio,
            pending_page_scale_animation_: None,
            lifecycle_: LayerTreeLifecycle::default(),
            display_transform_hint_: OverlayTransform::None,
            presentation_callbacks_: Vec::new(),
            successful_presentation_callbacks_: Vec::new(),
            events_metrics_from_main_thread_: Vec::new(),
            event_metrics_from_raster_thread_: Vec::new(),
            delegated_ink_metadata_: None,
            view_transition_requests_: Vec::new(),
            screenshot_destination_: UnguessableToken::default(),
            primary_main_frame_item_sequence_number_: -1,
            discardable_image_map_updater_: None,
        };
        tree.property_trees_.set_is_main_thread(false);
        tree
    }
    // Source: cc/trees/layer_tree_impl.cc:182-187.
    pub fn Shutdown(
        &mut self,
        context: &mut LayerTreeContext<'_, B>,
        destroy_layers: impl FnOnce(B::LayerList),
    ) {
        self.ShutdownWithHost(
            context.host.identity(),
            |id| context.host.IsActiveTree(id),
            || context.host.MonotonicTimeNow(),
            destroy_layers,
        );
    }
    // Source: cc/trees/layer_tree_impl.cc:182-187.
    // Rust borrow split: destruction only needs owner identity, the current
    // active-allocation query and live clock, not the whole host interface.
    pub(crate) fn ShutdownWithHost(
        &mut self,
        owner: LayerTreeHostId,
        is_active: impl FnOnce(LayerTreeId) -> bool,
        now: impl FnOnce() -> crate::adapter::time::TimeTicks,
        destroy_layers: impl FnOnce(B::LayerList),
    ) {
        // DetachLayers' returned unique owners are destroyed here in C++.
        // Rust supplies the call-scoped most-derived destructor dispatcher.
        destroy_layers(self.DetachLayers());
        self.FinishShutdownWithHost(owner, is_active, now);
    }
    // Source: cc/trees/layer_tree_impl.cc:184-187. Shared continuation after
    // the returned detached layer owners have completed their destructors.
    pub(crate) fn FinishShutdownWithHost(
        &mut self,
        owner: LayerTreeHostId,
        is_active: impl FnOnce(LayerTreeId) -> bool,
        now: impl FnOnce() -> crate::adapter::time::TimeTicks,
    ) {
        debug_assert_eq!(self.host_impl_, owner);
        let reason = if is_active(self.id) {
            DidNotSwapReason::SWAP_FAILS
        } else {
            DidNotSwapReason::ACTIVATION_FAILS
        };
        self.BreakSwapPromisesAt(reason, now());
        debug_assert!(self.LayerListIsEmpty());
    }
    // Source: cc/trees/layer_tree_impl.cc:189-215.
    pub fn ReleaseResources(&mut self, context: &mut LayerTreeContext<'_, B>) {
        self.ForEachLayerWithContext(false, context, |layer, context| {
            layer.ReleaseResources(context)
        });
    }
    pub fn OnPurgeMemory(&mut self, context: &mut LayerTreeContext<'_, B>) {
        self.ForEachLayerWithContext(false, context, |layer, context| {
            layer.OnPurgeMemory(context)
        });
    }
    pub fn ReleaseTileResources(&mut self, context: &mut LayerTreeContext<'_, B>) {
        self.ForEachLayerWithContext(false, context, |layer, context| {
            layer.ReleaseTileResources(context)
        });
    }
    pub fn RecreateTileResources(&mut self, context: &mut LayerTreeContext<'_, B>) {
        self.ForEachLayerWithContext(false, context, |layer, context| {
            layer.RecreateTileResources(context)
        });
    }
    pub fn SetVisible(&mut self, visible: bool) {
        if !visible {
            for layer in self.layer_list_.iter_mut() {
                layer.SetInInvisibleLayerTree();
            }
        }
    }
    // Source: cc/trees/layer_tree_impl.h:222-269.
    // Native IteratorAdapter is expressed by the backend's borrowed iterator GATs;
    // iteration dereferences the owned layer handles, without allocating a list.
    pub fn root_layer(&self) -> Option<&B::Layer> {
        self.layer_list_.iter().next()
    }
    /// Borrow the actual resident owner table for a render-pass call scope.
    /// This is the source layer_list_ pointer's Rust borrow projection.
    pub(crate) fn LayersForRead(&self) -> &B::LayerList {
        &self.layer_list_
    }
    /// Source layer_list_ pointees are mutable during native resource traversal.
    /// This lends the same owner for the call scope and copies no index or layer.
    pub(crate) fn LayersForMutable(&mut self) -> &mut B::LayerList {
        &mut self.layer_list_
    }
    pub fn num_layers(&self) -> usize {
        self.layer_list_.len()
    }
    pub fn begin(&self) -> <B::LayerList as LayerTreeLayerList<B>>::Iter<'_> {
        self.layer_list_.iter()
    }
    // Source: cc/trees/layer_tree_impl.h:261 (iterator over mutable LayerImpl pointees).
    // Pure resident layer-list reborrow; unlike RenderPassState this is also
    // valid before pending-tree draw properties have been updated.
    pub(crate) fn begin_mut(&mut self) -> <B::LayerList as LayerTreeLayerList<B>>::IterMut<'_> {
        self.layer_list_.iter_mut()
    }
    pub fn rbegin(&self) -> std::iter::Rev<<B::LayerList as LayerTreeLayerList<B>>::Iter<'_>> {
        self.layer_list_.iter().rev()
    }
    // Source iterator semantics: cc/layers/effect_tree_layer_list_iterator.cc.
    // Rust borrow bridge: retain only the reverse offset, then reborrow the
    // resident layer in O(1) without holding or copying the whole tree.
    pub(crate) fn LayerAtReverseIndex(&self, index: usize) -> Option<&B::Layer> {
        let len = self.layer_list_.len();
        (index < len).then(|| self.layer_list_.LayerAt(len - index - 1))
    }
    // Rust live-borrow bridge for cc/trees/layer_tree_host_impl.cc:1108-1585.
    // The source iterator holds pointers into these same resident fields; no
    // layer, property tree, surface list, or region is copied here.
    pub(crate) fn RenderPassState(
        &mut self,
    ) -> crate::adapter::layer_tree_host_impl::RenderPassState<'_, B> {
        debug_assert!(!self.needs_update_draw_properties_);
        crate::adapter::layer_tree_host_impl::RenderPassState {
            tree_id: self.id,
            layer_list: &mut self.layer_list_,
            property_trees: &mut self.property_trees_,
            render_surface_list: &self.render_surface_list_,
            unoccluded_screen_space_region: &self.unoccluded_screen_space_region_,
        }
    }
    // Source scoped_refptr getters used by ResetTreesForTesting. These clone
    // only the shared source value, never a tree or a property snapshot.
    pub(crate) fn SharedPageScaleFactor(&self) -> Rc<RefCell<SyncedScale>> {
        Rc::clone(&self.page_scale_factor_)
    }
    pub(crate) fn SharedTopControlsShownRatio(&self) -> Rc<RefCell<SyncedBrowserControls>> {
        Rc::clone(&self.top_controls_shown_ratio_)
    }
    pub(crate) fn SharedBottomControlsShownRatio(&self) -> Rc<RefCell<SyncedBrowserControls>> {
        Rc::clone(&self.bottom_controls_shown_ratio_)
    }
    // Source: cc/trees/layer_tree_impl.h:216-224.
    pub fn property_trees(&self) -> &PropertyTreesFor<B> {
        &self.property_trees_
    }
    pub fn property_trees_mut(&mut self) -> &mut PropertyTreesFor<B> {
        &mut self.property_trees_
    }
}
// Source: cc/trees/layer_tree_impl.cc:176-180.
impl<B: LayerTreeTypes> Drop for LayerTreeImpl<B> {
    fn drop(&mut self) {
        debug_assert!(self.LayerListIsEmpty());
    }
}

// Source: cc/trees/layer_tree_impl.cc:126-134.
fn GetVisibleSelectionEndPoints(rect: &RectF, top: &PointF, bottom: &PointF) -> (PointF, PointF) {
    let start = PointF::new(
        top.x().clamp(rect.x(), rect.right()),
        top.y().clamp(rect.y(), rect.bottom()),
    );
    let end = start + (*bottom - *top);
    (start, end)
}

// This is a split borrow of the canonical tree, not an additional tree/domain.
// Scroll callbacks borrow its other fields while the resident ScrollTree is
// executing; its source algorithms remain below in this translated file.
struct LayerTreeScrollFacade<'a, B: LayerTreeTypes, H: mutation::LayerTreeScrollHost<B> + ?Sized> {
    id: LayerTreeId,
    host: &'a mut H,
    layer_list: &'a mut dyn LayerTreeScrollStore<B>,
    scrollbar_ids: &'a BTreeMap<ElementId, ScrollbarLayerIds>,
    viewport: &'a ViewportPropertyIds,
    page_scale: &'a Rc<RefCell<SyncedScale>>,
    needs_update_draw_properties: &'a mut bool,
    show_requests: &'a mut BTreeSet<ElementId>,
}
impl<'a, B: LayerTreeTypes, H: mutation::LayerTreeScrollHost<B> + ?Sized>
    LayerTreeScrollFacade<'a, B, H>
{
    // Source: cc/trees/layer_tree_impl.cc:217-293.
    fn DidUpdateScrollOffsetCore(
        &mut self,
        id: ElementId,
        pushed: bool,
        scroll_tree: &ScrollTree,
        context: &mut super::property_tree::ScrollTreeMutationContext<'_, B::Transform>,
    ) {
        let active = self.host.IsActiveTree(self.id);
        let Some(node) = scroll_tree.FindNodeFromElementId(id) else {
            debug_assert!(!active);
            return;
        };
        let active_realize = scroll_tree.CanRealizeScrollsOnActiveTree(node);
        let pending_realize = scroll_tree.CanRealizeScrollsOnPendingTree(node);
        let mut realize_now = active_realize;
        if !active {
            realize_now |= pending_realize;
        }
        if realize_now {
            assert_ne!(node.transform_id, kInvalidPropertyNodeId);
            let offset = scroll_tree.current_scroll_offset(id, context.state);
            let transform_node = context.transform_tree.MutableNode(node.transform_id);
            if *transform_node.scroll_offset() != offset {
                transform_node.SetScrollOffset(&offset, DamageReason::kCompositorScroll);
                transform_node.needs_local_transform_update = true;
                let mut transform_context = super::property_tree::TransformTreeContext {
                    owner: context.owner,
                    scroll_tree,
                    clip_tree: context.clip_tree,
                    state: context.state,
                    cached_data: context.cached_data,
                };
                context
                    .transform_tree
                    .set_needs_update(true, &mut transform_context);
            }
            context
                .transform_tree
                .MutableNode(node.transform_id)
                .SetTransformChanged(DamageReason::kCompositorScroll);
            context.state.set_changed(true);
            *self.needs_update_draw_properties = true;
        }
        if pending_realize {
            self.host
                .RequestImplSideInvalidationForRasterInducingScroll(id);
        }
        if active {
            if self.host.HasPendingTree() && !pushed {
                self.host.PendingTreeDidUpdateScrollOffset(id, pushed);
            }
            if self.host.HasRecycleTree() {
                self.host.RecycleTreeDidUpdateScrollOffset(id, pushed);
            }
            if !pushed {
                if node.id == self.viewport.inner_scroll || node.id == self.viewport.outer_scroll {
                    self.UpdateViewportScrollbarGeometries(scroll_tree, context.state);
                } else {
                    self.UpdateScrollbarGeometries(node, scroll_tree, context.state);
                }
            }
        }
    }
    // Source: cc/trees/layer_tree_impl.cc:311-321.
    fn UpdateViewportScrollbarGeometries(
        &mut self,
        scroll_tree: &ScrollTree,
        state: &super::property_tree::PropertyTreesState,
    ) {
        if !self.host.IsActiveTree(self.id) {
            return;
        }
        if self.viewport.inner_scroll != kInvalidPropertyNodeId {
            self.UpdateScrollbarGeometries(
                scroll_tree.Node(self.viewport.inner_scroll),
                scroll_tree,
                state,
            );
        }
        if self.viewport.outer_scroll != kInvalidPropertyNodeId {
            self.UpdateScrollbarGeometries(
                scroll_tree.Node(self.viewport.outer_scroll),
                scroll_tree,
                state,
            );
        }
    }
    // Source: cc/trees/layer_tree_impl.cc:323-380.
    fn UpdateScrollbarGeometries(
        &mut self,
        node: &ScrollNode,
        scroll_tree: &ScrollTree,
        state: &super::property_tree::PropertyTreesState,
    ) {
        debug_assert!(self.host.IsActiveTree(self.id));
        let mut offset = scroll_tree.current_scroll_offset(node.element_id, state);
        let mut scrolling_size =
            SizeF::new(node.bounds.width() as f32, node.bounds.height() as f32);
        let mut bounds = scroll_tree.container_bounds(node.id, state);
        let viewport =
            node.id == self.viewport.inner_scroll || node.id == self.viewport.outer_scroll;
        if viewport {
            offset = PointF::default();
            if self.viewport.inner_scroll != kInvalidPropertyNodeId
                && self.viewport.outer_scroll != kInvalidPropertyNodeId
            {
                offset = PointF::new(0.0, 0.0)
                    + scroll_tree
                        .current_scroll_offset(
                            scroll_tree.Node(self.viewport.inner_scroll).element_id,
                            state,
                        )
                        .OffsetFromOrigin()
                    + scroll_tree
                        .current_scroll_offset(
                            scroll_tree.Node(self.viewport.outer_scroll).element_id,
                            state,
                        )
                        .OffsetFromOrigin();
            }
            let mut viewport_bounds = SizeF::new(bounds.width() as f32, bounds.height() as f32);
            let page_scale = self
                .page_scale
                .borrow()
                .Current(self.host.IsActiveTree(self.id));
            if node.id == self.viewport.inner_scroll {
                if self.viewport.outer_scroll == kInvalidPropertyNodeId {
                    return;
                }
                let outer = scroll_tree.Node(self.viewport.outer_scroll);
                let outer_bounds = scroll_tree.container_bounds(outer.id, state);
                viewport_bounds = SizeF::new(
                    (viewport_bounds.width() / page_scale).min(outer_bounds.width() as f32),
                    (viewport_bounds.height() / page_scale).min(outer_bounds.height() as f32),
                );
                scrolling_size =
                    SizeF::new(outer.bounds.width() as f32, outer.bounds.height() as f32);
            } else {
                debug_assert_eq!(node.id, self.viewport.outer_scroll);
                if self.viewport.inner_scroll == kInvalidPropertyNodeId {
                    return;
                }
                let inner = scroll_tree.Node(self.viewport.inner_scroll);
                let inner_bounds = scroll_tree.container_bounds(inner.id, state);
                viewport_bounds = SizeF::new(
                    viewport_bounds
                        .width()
                        .min(inner_bounds.width() as f32 / page_scale),
                    viewport_bounds
                        .height()
                        .min(inner_bounds.height() as f32 / page_scale),
                );
            }
            bounds = Size::new(
                viewport_bounds.width().ceil() as i32,
                viewport_bounds.height().ceil() as i32,
            );
        }
        if let Some(ids) = self.scrollbar_ids.get(&node.element_id) {
            for id in [ids.horizontal, ids.vertical] {
                if id == -1 {
                    continue;
                }
                let tree_id = self.id;
                let host = &mut *self.host;
                let needs_draw = &mut *self.needs_update_draw_properties;
                let show_requests = &mut *self.show_requests;
                self.layer_list
                    .WithScrollbarGeometry(id, &mut |scrollbar, push| {
                        let mut context = LayerTreeGeometryContext {
                            id: tree_id,
                            host,
                            push,
                            needs_draw,
                            show_requests,
                            _types: std::marker::PhantomData,
                        };
                        if scrollbar.orientation() == ScrollbarOrientation::kHorizontal {
                            scrollbar.SetCurrentPos(offset.x(), &mut context);
                            scrollbar.SetClipLayerLength(bounds.width() as f32, &mut context);
                            scrollbar.SetScrollLayerLength(scrolling_size.width(), &mut context);
                        } else {
                            scrollbar.SetCurrentPos(offset.y(), &mut context);
                            scrollbar.SetClipLayerLength(bounds.height() as f32, &mut context);
                            scrollbar.SetScrollLayerLength(scrolling_size.height(), &mut context);
                        }
                        if viewport {
                            scrollbar.SetVerticalAdjust(
                                state.inner_viewport_container_bounds_delta().y(),
                                &mut context,
                            );
                        }
                    });
            }
        }
    }
}
impl<B: LayerTreeTypes, H: mutation::LayerTreeScrollHost<B> + ?Sized>
    PropertyTreeScrollLayerTree<B::Transform> for LayerTreeScrollFacade<'_, B, H>
{
    fn DidUpdateScrollOffset(
        &mut self,
        id: ElementId,
        pushed: bool,
        scroll_tree: &ScrollTree,
        context: &mut super::property_tree::ScrollTreeMutationContext<'_, B::Transform>,
    ) {
        self.DidUpdateScrollOffsetCore(id, pushed, scroll_tree, context);
    }
    fn DidAnimateScrollOffset(&mut self) {
        self.host.DidAnimateScrollOffset();
    }
}
impl<B: LayerTreeTypes> LayerTreeImpl<B> {
    // Source: cc/trees/layer_tree_impl.cc:217-293.
    pub fn DidUpdateScrollOffset(
        &mut self,
        id: ElementId,
        pushed: bool,
        context: &mut LayerTreeContext<'_, B>,
    ) {
        self.DidUpdateScrollOffsetWithHost(id, pushed, context.host)
    }
    // Source: cc/trees/layer_tree_impl.cc:217-293.
    pub(crate) fn DidUpdateScrollOffsetWithHost<H: mutation::LayerTreeScrollHost<B> + ?Sized>(
        &mut self,
        id: ElementId,
        pushed: bool,
        host: &mut H,
    ) {
        let (layers, mut fields) = self.SplitLayerTreeFields();
        fields.DidUpdateScrollOffsetWithStore(
            id,
            pushed,
            host,
            &mut LayerTreeWholeScrollStore::<B>(layers),
        );
    }
    // Source: cc/trees/layer_tree_impl.cc:295-309.
    fn UpdateAllScrollbarGeometries(&mut self, context: &mut LayerTreeContext<'_, B>) {
        if !self.IsActiveTree(context) {
            return;
        }
        debug_assert!(self.lifecycle_.AllowsPropertyTreeAccess());
        debug_assert!(self.lifecycle_.AllowsLayerPropertyAccess());
        let (scroll_tree, _, state) = self.property_trees_.scroll_tree_read_context();
        let mut facade = LayerTreeScrollFacade {
            id: self.id,
            host: context.host,
            layer_list: &mut LayerTreeWholeScrollStore::<B>(&mut self.layer_list_),
            scrollbar_ids: &self.element_id_to_scrollbar_layer_ids_,
            viewport: &self.viewport_property_ids_,
            page_scale: &self.page_scale_factor_,
            needs_update_draw_properties: &mut self.needs_update_draw_properties_,
            show_requests: &mut self.show_scrollbar_requests_,
        };
        for (id, _) in &self.element_id_to_scrollbar_layer_ids_ {
            if let Some(node) = scroll_tree.FindNodeFromElementId(*id) {
                facade.UpdateScrollbarGeometries(node, scroll_tree, state);
            }
        }
    }
    // Source: cc/trees/layer_tree_impl.cc:311-321.
    fn UpdateViewportScrollbarGeometries(&mut self, context: &mut LayerTreeContext<'_, B>) {
        self.UpdateViewportScrollbarGeometriesWithHost(context.host)
    }
    // Source: cc/trees/layer_tree_impl.cc:311-321.
    pub(crate) fn UpdateViewportScrollbarGeometriesWithHost<
        H: mutation::LayerTreeScrollHost<B> + ?Sized,
    >(
        &mut self,
        host: &mut H,
    ) {
        let (scroll_tree, _, state) = self.property_trees_.scroll_tree_read_context();
        let mut facade = LayerTreeScrollFacade {
            id: self.id,
            host,
            layer_list: &mut LayerTreeWholeScrollStore::<B>(&mut self.layer_list_),
            scrollbar_ids: &self.element_id_to_scrollbar_layer_ids_,
            viewport: &self.viewport_property_ids_,
            page_scale: &self.page_scale_factor_,
            needs_update_draw_properties: &mut self.needs_update_draw_properties_,
            show_requests: &mut self.show_scrollbar_requests_,
        };
        facade.UpdateViewportScrollbarGeometries(scroll_tree, state);
    }
    // Source: cc/trees/layer_tree_impl.cc:382-400.
    pub fn RootRenderSurface(&self) -> Option<&B::RenderSurface> {
        Self::RootRenderSurfaceForPropertyTrees(&self.property_trees_)
    }
    // Same source body; permits the scoped occlusion context to query this
    // owner's actual tree without retaining a reverse LayerTreeImpl pointer.
    pub(crate) fn RootRenderSurfaceForPropertyTrees(
        trees: &PropertyTreesFor<B>,
    ) -> Option<&B::RenderSurface> {
        let effects = trees.effect_tree();
        if effects.size() > kContentsRootPropertyNodeId as usize
            && (kContentsRootPropertyNodeId as usize) < effects.num_render_surfaces()
        {
            effects.GetRenderSurface(kContentsRootPropertyNodeId)
        } else {
            None
        }
    }
    // Source: cc/trees/layer_tree_impl.cc:402-404.
    pub fn LayerListIsEmpty(&self) -> bool {
        self.layer_list_.is_empty()
    }
    // Source: cc/trees/layer_tree_impl.cc:406-414.
    pub fn SetRootLayerForTesting(
        &mut self,
        layer: Option<Box<B::Layer>>,
        context: &mut LayerTreeContext<'_, B>,
    ) {
        self.DetachLayers();
        if let Some(layer) = layer {
            let id = layer.id();
            self.AddLayer(layer);
            self.layer_list_.AddLayerShouldPushProperties(id);
        }
        context.host.OnCanDrawStateChangedForTree(self);
    }
    // Source: cc/trees/layer_tree_impl.cc:416-418.
    pub fn OnCanDrawStateChangedForTree(&mut self, context: &mut LayerTreeContext<'_, B>) {
        context.host.OnCanDrawStateChangedForTree(self);
    }
    // Source: cc/trees/layer_tree_impl.cc:420-453.
    pub fn InvalidateRegionForImages(
        &mut self,
        images: &BTreeSet<B::PaintImageId>,
        context: &mut LayerTreeContext<'_, B>,
    ) {
        debug_assert!(self.IsSyncTree(context));
        let (mut no_images_count, mut no_invalidation_count, mut invalidated_count) =
            (0usize, 0usize, 0usize);
        if !images.is_empty() {
            B::WithPictureLayersContext(self, context.host, |layers, layer_context| {
                let _ = layers.ForEachPictureLayer(&mut |layer| {
                    match layer.InvalidateRegionForImages(images, layer_context) {
                        ImageInvalidationResult::kNoImages => no_images_count += 1,
                        ImageInvalidationResult::kNoInvalidation => no_invalidation_count += 1,
                        ImageInvalidationResult::kInvalidated => invalidated_count += 1,
                    }
                    std::ops::ControlFlow::Continue(())
                });
            });
        }
        context.host.TraceImageInvalidationCounts(
            no_images_count,
            no_invalidation_count,
            invalidated_count,
        );
    }
    // Source: cc/trees/layer_tree_impl.cc:455-466.
    pub fn InvalidateRasterInducingScrolls(
        &mut self,
        scrolls: &BTreeSet<ElementId>,
        context: &mut LayerTreeContext<'_, B>,
    ) {
        let is_sync_tree = self.IsSyncTree(context);
        self.InvalidateRasterInducingScrollsWithPictureHost(scrolls, is_sync_tree, context.host)
    }
    // Same source body with only its actual Picture factory dependency. The
    // client commit caller separately lends its resident pending scroll set.
    pub(crate) fn InvalidateRasterInducingScrollsWithPictureHost(
        &mut self,
        scrolls: &BTreeSet<ElementId>,
        is_sync_tree: bool,
        host: &mut dyn crate::adapter::picture_layer_impl::PictureLayerContextHost<B>,
    ) {
        debug_assert!(is_sync_tree);
        if scrolls.is_empty() {
            self.did_raster_inducing_scroll_ = false;
            return;
        }
        self.did_raster_inducing_scroll_ = true;
        B::WithPictureLayersContext(self, host, |layers, layer_context| {
            let _ = layers.ForEachPictureLayer(&mut |layer| {
                layer.InvalidateRasterInducingScrolls(scrolls, layer_context);
                std::ops::ControlFlow::Continue(())
            });
        });
    }
}

// Source: cc/trees/layer_tree_impl.cc:75-123.
// Non-owning node pointers are stable node identities for this synchronous scope.
struct ViewportAnchor {
    inner_: i32,
    outer_: Option<i32>,
    viewport_in_content_coordinates_: Vector2dF,
}
impl ViewportAnchor {
    fn new<B: LayerTreeTypes>(inner: i32, outer: Option<i32>, tree: &LayerTreeImpl<B>) -> Self {
        let (scroll, _, state) = tree.property_trees_.scroll_tree_read_context();
        let mut point = scroll
            .current_scroll_offset(scroll.Node(inner).element_id, state)
            .OffsetFromOrigin();
        if let Some(outer) = outer {
            point += scroll
                .current_scroll_offset(scroll.Node(outer).element_id, state)
                .OffsetFromOrigin();
        }
        Self {
            inner_: inner,
            outer_: outer,
            viewport_in_content_coordinates_: point,
        }
    }
    fn ResetViewportToAnchoredPosition<
        B: LayerTreeTypes,
        H: mutation::LayerTreeScrollHost<B> + ?Sized,
    >(
        &self,
        tree: &mut LayerTreeImpl<B>,
        host: &mut H,
    ) {
        let outer = self.outer_.expect("source DCHECK outer viewport");
        tree.ClampScrollNodeToMaxScrollOffsetWithHost(self.inner_, host);
        tree.ClampScrollNodeToMaxScrollOffsetWithHost(outer, host);
        let (scroll, _, state) = tree.property_trees_.scroll_tree_read_context();
        let location = scroll
            .current_scroll_offset(scroll.Node(self.inner_).element_id, state)
            .OffsetFromOrigin()
            + scroll
                .current_scroll_offset(scroll.Node(outer).element_id, state)
                .OffsetFromOrigin();
        let delta = self.viewport_in_content_coordinates_ - location;
        let delta = tree.ScrollNodeByWithHost(self.inner_, &delta, host);
        tree.ScrollNodeByWithHost(outer, &delta, host);
    }
}
impl<B: LayerTreeTypes> LayerTreeImpl<B> {
    pub(crate) fn ScrollNodeBy(
        &mut self,
        id: i32,
        scroll: &Vector2dF,
        context: &mut LayerTreeContext<'_, B>,
    ) -> Vector2dF {
        self.ScrollNodeByWithHost(id, scroll, context.host)
    }
    // Borrow-only capability entry; the source body is shared with the context entry.
    // Source: cc/trees/property_tree.cc:2371-2395.
    pub(crate) fn ScrollNodeByWithHost<H: mutation::LayerTreeScrollHost<B> + ?Sized>(
        &mut self,
        id: i32,
        scroll: &Vector2dF,
        host: &mut H,
    ) -> Vector2dF {
        let (layers, mut fields) = self.SplitLayerTreeFields();
        fields.ScrollNodeByWithStore(
            id,
            scroll,
            host,
            &mut LayerTreeWholeScrollStore::<B>(layers),
        )
    }
    pub(crate) fn ClampScrollNodeToMaxScrollOffset(
        &mut self,
        id: i32,
        context: &mut LayerTreeContext<'_, B>,
    ) -> Vector2dF {
        self.ClampScrollNodeToMaxScrollOffsetWithHost(id, context.host)
    }
    // Borrow-only capability entry; the source body is shared with the context entry.
    // Source: cc/trees/property_tree.cc:1946-1956.
    pub(crate) fn ClampScrollNodeToMaxScrollOffsetWithHost<
        H: mutation::LayerTreeScrollHost<B> + ?Sized,
    >(
        &mut self,
        id: i32,
        host: &mut H,
    ) -> Vector2dF {
        let (tree, mut mutation) = self.property_trees_.scroll_tree_mutation_context();
        let mut facade = LayerTreeScrollFacade {
            id: self.id,
            host: host,
            layer_list: &mut LayerTreeWholeScrollStore::<B>(&mut self.layer_list_),
            scrollbar_ids: &self.element_id_to_scrollbar_layer_ids_,
            viewport: &self.viewport_property_ids_,
            page_scale: &self.page_scale_factor_,
            needs_update_draw_properties: &mut self.needs_update_draw_properties_,
            show_requests: &mut self.show_scrollbar_requests_,
        };
        tree.ClampScrollToMaxScrollOffsetById(id, &mut facade, &mut mutation)
    }
    // Source caller: cc/trees/layer_tree_host_impl.cc:5996-6012.
    // Source algorithm: cc/trees/property_tree.cc:1867-1885.
    // Rust borrow split preserves the resident ScrollTree and sibling mutation
    // context; no owner/tree reference or independent scroll state is retained.
    pub(crate) fn OnScrollOffsetAnimated(
        &mut self,
        id: ElementId,
        node_id: i32,
        offset: &PointF,
        context: &mut LayerTreeContext<'_, B>,
    ) {
        debug_assert_eq!(self.host_impl_, context.host.identity());
        let (tree, mut mutation) = self.property_trees_.scroll_tree_mutation_context();
        debug_assert_eq!(tree.Node(node_id).element_id, id);
        let mut facade = LayerTreeScrollFacade {
            id: self.id,
            host: context.host,
            layer_list: &mut LayerTreeWholeScrollStore::<B>(&mut self.layer_list_),
            scrollbar_ids: &self.element_id_to_scrollbar_layer_ids_,
            viewport: &self.viewport_property_ids_,
            page_scale: &self.page_scale_factor_,
            needs_update_draw_properties: &mut self.needs_update_draw_properties_,
            show_requests: &mut self.show_scrollbar_requests_,
        };
        tree.OnScrollOffsetAnimated(id, node_id, offset, &mut facade, &mut mutation);
    }
    // Source: cc/trees/layer_tree_impl.cc:468-581.
    pub fn UpdateViewportContainerSizes(&mut self, context: &mut LayerTreeContext<'_, B>) {
        self.UpdateViewportContainerSizesWithHost(context.host)
    }
    // Borrow-only capability entry; the source body is shared with the context entry.
    // Source: cc/trees/layer_tree_impl.cc:468-581.
    pub fn UpdateViewportContainerSizesWithHost<H: mutation::LayerTreeScaleHost<B> + ?Sized>(
        &mut self,
        host: &mut H,
    ) {
        let ids = self.viewport_property_ids_;
        if ids.inner_scroll == kInvalidPropertyNodeId || ids.outer_scroll == kInvalidPropertyNodeId
        {
            return;
        }
        let anchor = ViewportAnchor::new(ids.inner_scroll, Some(ids.outer_scroll), self);
        let active = host.IsActiveTree(self.id);
        let top_ratio = self.top_controls_shown_ratio_.borrow().Current(active);
        let bottom_ratio = self.bottom_controls_shown_ratio_.borrow().Current(active);
        let top_layout = if self.browser_controls_shrink_blink_size() {
            self.top_controls_height()
        } else {
            self.top_controls_min_height()
        };
        let top_offset = if self.top_controls_height() > 0.0 {
            self.top_controls_height() * top_ratio
        } else {
            0.0
        };
        let mut delta = top_layout - top_offset;
        let bottom_layout = if self.browser_controls_shrink_blink_size() {
            self.bottom_controls_height()
        } else {
            self.bottom_controls_min_height()
        };
        let bottom_offset = if self.bottom_controls_height() > 0.0 {
            self.bottom_controls_height() * bottom_ratio
        } else {
            0.0
        };
        delta += bottom_layout - bottom_offset;
        let blink_offset = bottom_offset
            - self.bottom_controls_height() * self.bottom_controls_shown_ratio_.borrow().Delta();
        let real_saib = (self.max_safe_area_inset_bottom() - bottom_offset).max(0.0);
        let blink_saib = (self.max_safe_area_inset_bottom() - blink_offset).max(0.0);
        let mut safe_delta = -(real_saib - blink_saib);
        debug_assert!(self.min_page_scale_factor() > 0.0);
        safe_delta /= self.min_page_scale_factor();
        if self
            .property_trees_
            .transform_delta_by_safe_area_inset_bottom()
            != safe_delta
        {
            self.property_trees_
                .SetTransformDeltaBySafeAreaInsetBottom(safe_delta);
        }
        let bounds_delta = Vector2dF::new(0.0, delta);
        if self.property_trees_.inner_viewport_container_bounds_delta() == bounds_delta {
            return;
        }
        self.property_trees_
            .SetInnerViewportContainerBoundsDelta(bounds_delta);
        let mut scaled_delta = bounds_delta;
        if self.min_page_scale_factor() > 0.0 {
            scaled_delta.Scale(1.0 / self.min_page_scale_factor());
        }
        self.property_trees_
            .SetOuterViewportContainerBoundsDelta(scaled_delta);
        debug_assert_eq!(
            scaled_delta,
            self.property_trees_.inner_viewport_scroll_bounds_delta()
        );
        if ids.outer_clip != kInvalidPropertyNodeId {
            let height = self
                .property_trees_
                .scroll_tree()
                .Node(ids.outer_scroll)
                .container_bounds
                .height() as f32
                + scaled_delta.y();
            let clips = self.property_trees_.clip_tree_mutable();
            clips.MutableNode(ids.outer_clip).clip.set_height(height);
            let mut ancestor = clips.Node(ids.outer_clip).parent_id;
            while ancestor != kInvalidPropertyNodeId && ancestor != kRootPropertyNodeId {
                let clip = clips.Node(ids.outer_clip).clip;
                clips.MutableNode(ancestor).clip.Union(clip);
                ancestor = clips.Node(ancestor).parent_id;
            }
        }
        anchor.ResetViewportToAnchoredPosition(self, host);
        self.property_trees_
            .clip_tree_mutable()
            .set_needs_update(true);
        self.property_trees_.set_full_tree_damaged(true);
        self.set_needs_update_draw_properties();
        self.UpdateViewportScrollbarGeometriesWithHost(host);
    }
    // Source: cc/trees/layer_tree_impl.cc:583-585.
    pub fn IsRootLayer(&self, layer: &B::Layer) -> bool {
        self.root_layer()
            .is_some_and(|root| std::ptr::eq(root, layer))
    }
    // Source: cc/trees/layer_tree_impl.cc:587-612.
    pub fn TotalScrollOffset(&self) -> PointF {
        let mut offset = Vector2dF::default();
        let (scroll, _, state) = self.property_trees_.scroll_tree_read_context();
        if let (Some(inner), Some(outer)) = (
            self.InnerViewportScrollNode(),
            self.OuterViewportScrollNode(),
        ) {
            offset += scroll
                .current_scroll_offset(inner.element_id, state)
                .OffsetFromOrigin()
                + scroll
                    .current_scroll_offset(outer.element_id, state)
                    .OffsetFromOrigin();
        }
        PointF::new(0.0, 0.0) + offset
    }
    // Source: cc/trees/layer_tree_impl.cc:587-612.
    pub fn TotalScrollOffsetForElement(&self, id: ElementId) -> PointF {
        let mut offset = Vector2dF::default();
        let (scroll, _, state) = self.property_trees_.scroll_tree_read_context();
        if let Some(node) = scroll.FindNodeFromElementId(id) {
            offset = scroll
                .current_scroll_offset(node.element_id, state)
                .OffsetFromOrigin();
        }
        PointF::new(0.0, 0.0) + offset
    }
    // Source: cc/trees/layer_tree_impl.cc:614-641.
    pub fn TotalMaxScrollOffset(&self) -> PointF {
        let mut offset = Vector2dF::default();
        let (scroll, transforms, state) = self.property_trees_.scroll_tree_read_context();
        if self.viewport_property_ids_.inner_scroll != kInvalidPropertyNodeId {
            offset += scroll
                .MaxScrollOffset(self.viewport_property_ids_.inner_scroll, transforms, state)
                .OffsetFromOrigin();
        }
        if self.viewport_property_ids_.outer_scroll != kInvalidPropertyNodeId {
            offset += scroll
                .MaxScrollOffset(self.viewport_property_ids_.outer_scroll, transforms, state)
                .OffsetFromOrigin();
        }
        PointF::new(0.0, 0.0) + offset
    }
    // Source: cc/trees/layer_tree_impl.cc:614-641.
    pub fn TotalMaxScrollOffsetForElement(&self, id: ElementId) -> PointF {
        let mut offset = Vector2dF::default();
        let (scroll, transforms, state) = self.property_trees_.scroll_tree_read_context();
        if let Some(node) = scroll.FindNodeFromElementId(id) {
            offset = scroll
                .MaxScrollOffset(node.id, transforms, state)
                .OffsetFromOrigin();
        }
        PointF::new(0.0, 0.0) + offset
    }
    // Source: cc/trees/layer_tree_impl.cc:643-656.
    pub fn DetachLayers(&mut self) -> B::LayerList {
        self.ClearLayersThatShouldPushProperties();
        self.render_surface_list_.clear();
        self.set_needs_update_draw_properties();
        self.hud_layer_ = None;
        let result = std::mem::take(&mut self.layer_list_);
        assert_eq!(self.layer_list_.len(), 0);
        result
    }
    /// Destroy owners that TreeSynchronizer did not reuse. In Chromium the
    /// local vector of `unique_ptr<LayerImpl>` performs these most-derived
    /// destructors when it leaves scope; Rust must run the same callbacks
    /// explicitly before dropping the boxes.
    pub fn DestroyDetachedLayers(
        &mut self,
        mut layers: B::LayerList,
        context: &mut LayerTreeContext<'_, B>,
    ) {
        // TreeSynchronizer leaves holes for owners transferred into the new
        // list. Walk only the unique_ptr-equivalent owners that remain.
        for id in layers.RemainingLayerIds() {
            let layer = layers
                .LayerByIdMut(id)
                .expect("remaining detached layer owner");
            if let Some(picture) = layer.AsPictureLayerMut() {
                let identity = picture.IdentityForDestruction();
                if let Some(twin) = picture.TwinForDestruction() {
                    let reverse = context.host.DisconnectPictureTwin(twin);
                    assert_eq!(reverse, identity, "native symmetric PictureLayerImpl twin");
                    assert_eq!(picture.ClearTwinForDestruction(), twin);
                }
            }
            B::WithDetachedLayerContext(self, context, layer, |layer, layer_context| {
                layer.Destroy(layer_context)
            });
        }
        drop(layers);
    }
    // Source: cc/trees/layer_tree_impl.cc:658-663.
    pub fn DetachLayersKeepingRootLayerForTesting(
        &mut self,
        context: &mut LayerTreeContext<'_, B>,
    ) -> B::LayerList {
        let mut layers = self.DetachLayers();
        let id = layers.LayerAt(0).id();
        self.SetRootLayerForTesting(Some(layers.ReleaseLayerForTesting(id)), context);
        layers
    }
    // Source: cc/trees/layer_tree_impl.cc:665-670.
    pub fn SwapLayers(&mut self, new_layers: B::LayerList) -> B::LayerList {
        let result = self.DetachLayers();
        self.layer_list_ = new_layers;
        self.set_needs_update_draw_properties();
        result
    }
    // Source: cc/trees/layer_tree_impl.cc:672-737.
    pub fn SetPropertyTrees(
        &mut self,
        trees: &mut PropertyTreesFor<B>,
        viewport: &ViewportPropertyIds,
        preserve: bool,
        context: &mut LayerTreeContext<'_, B>,
    ) {
        if preserve {
            self.property_trees_.CollectChangeState();
            trees.ApplyChangeStateFrom(&mut self.property_trees_);
        }
        let active = self.IsActiveTree(context);
        let scrolling_element = if active {
            self.CurrentlyScrollingNode()
                .map_or(ElementId::default(), |node| node.element_id)
        } else {
            ElementId::default()
        };
        let mut old_surfaces = Vec::new();
        self.property_trees_
            .effect_tree_mutable()
            .TakeRenderSurfaces(&mut old_surfaces);
        self.property_trees_.AssignFrom(trees);
        self.property_trees_.ApplyChangedNodes(
            trees.changed_effect_nodes(),
            trees.changed_transform_nodes(),
        );
        self.property_trees_.set_changed(trees.changed());
        self.property_trees_
            .set_needs_rebuild(trees.needs_rebuild());
        self.property_trees_
            .set_full_tree_damaged(trees.full_tree_damaged());
        let changed = self
            .property_trees_
            .effect_tree_mutable()
            .CreateOrReuseRenderSurfaces(
                &mut old_surfaces,
                &mut LayerTreeSurfaceFactory {
                    id: self.id,
                    host: context.host,
                },
            );
        self.property_trees_
            .effect_tree_mutable()
            .ApplyRenderSurfaceChangedFlags(trees.surface_property_changed_flags());
        if changed {
            self.set_needs_update_draw_properties();
        }
        let mut requests = trees.TakeCopyRequests();
        self.property_trees_
            .effect_tree_mutable()
            .PullCopyRequestsFrom(&mut requests);
        self.property_trees_.set_is_main_thread(false);
        self.property_trees_.set_is_active(active);
        if active {
            self.property_trees_
                .effect_tree_mutable()
                .set_needs_update(true);
        }
        let scrolling_node = if scrolling_element != ElementId::default() {
            self.property_trees_
                .scroll_tree()
                .FindNodeFromElementId(scrolling_element)
                .map(|node| node.id)
        } else {
            None
        };
        self.SetCurrentlyScrollingNodeById(scrolling_node, context);
        self.SetViewportPropertyIds(viewport);
    }
}
struct LayerTreeSurfaceFactory<'a, B: LayerTreeTypes> {
    id: LayerTreeId,
    host: &'a mut dyn LayerTreeHost<B>,
}
impl<B: LayerTreeTypes> PropertyTreeRenderSurfaceFactory<B::RenderSurface>
    for LayerTreeSurfaceFactory<'_, B>
{
    fn CreateRenderSurface(&mut self, id: ElementId) -> Box<B::RenderSurface> {
        self.host.CreateRenderSurface(self.id, id)
    }
}

struct TreeCommitSyncAdapter<'a, 'context, 'main, 'main_context, B>
where
    B: LayerTreeTypes + 'main + 'main_context,
    'main_context: 'main,
{
    tree: &'a mut LayerTreeImpl<B>,
    commit: &'a mut B::CommitState,
    unsafe_state: &'a B::UnsafeCommitState,
    context: &'a mut LayerTreeContext<'context, B>,
    main_context:
        &'a mut <B::Synchronizer as LayerTreeSynchronizer<B>>::MainContext<'main, 'main_context>,
}

impl<'main, 'main_context, B> compositor_tree::commit_sync::CommitSyncContext
    for TreeCommitSyncAdapter<'_, '_, 'main, 'main_context, B>
where
    B: LayerTreeTypes + 'main + 'main_context,
    'main_context: 'main,
{
    fn begin_sync(&mut self) {
        self.tree
            .lifecycle_
            .AdvanceTo(LifecycleState::kBeginningSync);
    }

    fn commit_forces_redraw(&self) -> bool {
        self.commit.next_commit_forces_redraw()
    }

    fn force_redraw_next_activation(&mut self) {
        self.tree.ForceRedrawNextActivation();
    }

    fn commit_forces_raster_scale_recalculation(&self) -> bool {
        self.commit.next_commit_forces_recalculate_raster_scales()
    }

    fn force_raster_scale_recalculation(&mut self) {
        self.tree.ForceRecalculateRasterScales();
    }

    fn transfer_presentation_callbacks(&mut self) {
        self.tree
            .AddPresentationCallbacks(self.commit.TakePresentationCallbacks());
    }

    fn transfer_successful_presentation_callbacks(&mut self) {
        self.tree
            .AddSuccessfulPresentationCallbacks(self.commit.TakeSuccessfulPresentationCallbacks());
    }

    fn needs_full_tree_sync(&self) -> bool {
        self.commit.needs_full_tree_sync()
    }

    fn synchronize_full_tree(&mut self) {
        B::Synchronizer::SynchronizeTrees(
            self.commit,
            self.unsafe_state,
            self.tree,
            self.context,
            self.main_context,
        );
    }

    fn should_clear_caches(&self) -> bool {
        self.commit.clear_caches_on_next_commit()
    }

    fn clear_history_and_caches(&mut self) {
        self.context.host.ClearHistory();
        self.context.host.ClearCaches();
    }

    fn pull_property_trees(&mut self) {
        self.tree
            .PullPropertyTreesFrom(self.commit, self.unsafe_state, self.context);
    }

    fn mark_property_trees_synced(&mut self) {
        self.tree
            .lifecycle_
            .AdvanceTo(LifecycleState::kSyncedPropertyTrees);
    }

    fn needs_surface_ranges_sync(&self) -> bool {
        self.commit.needs_surface_ranges_sync()
    }

    fn synchronize_surface_ranges(&mut self) {
        self.tree.ClearSurfaceRanges();
        self.tree.SetSurfaceRanges(self.commit.SurfaceRanges());
    }

    fn push_layer_properties_and_scroll_updates(&mut self) {
        let mut updater = DiscardableImageMapUpdater::new(self.tree, self.context);
        let (tree, updates, context) = updater.split();
        B::Synchronizer::PushLayerProperties(
            self.commit,
            self.unsafe_state,
            tree,
            context,
            self.main_context,
            updates,
        );
        tree.lifecycle_
            .AdvanceTo(LifecycleState::kSyncedLayerProperties);
        for id in self.commit.scrollers_clobbering_active_value() {
            let (scroll, _, state) = tree.property_trees_.scroll_tree_read_context();
            scroll.SetScrollOffsetClobberActiveValue(*id, state);
        }
        B::WithPictureLayersContext(tree, context.host, |layers, layer_context| {
            for id in self.commit.picture_layer_ids_with_new_raster_source() {
                layers
                    .LayerByIdMut(*id)
                    .unwrap()
                    .AsPictureLayerMut()
                    .unwrap()
                    .CommitPendingRasterSource(updates, layer_context);
            }
        });
        let fractional = context.host.settings().commit_fractional_scroll_deltas();
        let (scroll, mut mutation) = tree.property_trees_.scroll_tree_mutation_context();
        let mut facade = LayerTreeScrollFacade {
            id: tree.id,
            host: context.host,
            layer_list: &mut LayerTreeWholeScrollStore::<B>(&mut tree.layer_list_),
            scrollbar_ids: &tree.element_id_to_scrollbar_layer_ids_,
            viewport: &tree.viewport_property_ids_,
            page_scale: &tree.page_scale_factor_,
            needs_update_draw_properties: &mut tree.needs_update_draw_properties_,
            show_requests: &mut tree.show_scrollbar_requests_,
        };
        scroll.PushScrollUpdatesFromMainThread(
            self.commit.property_trees().scroll_tree(),
            &mut facade,
            fractional,
            &mut mutation,
        );
    }

    fn pull_layer_tree_properties(&mut self) {
        self.tree
            .PullLayerTreePropertiesFrom(self.commit, self.context);
    }

    fn transfer_swap_promises(&mut self) {
        self.tree
            .PassSwapPromises(self.commit.TakeSwapPromises(), self.context);
    }

    fn transfer_event_metrics(&mut self) {
        self.tree
            .AppendEventsMetricsFromMainThread(self.commit.TakeEventMetrics());
    }

    fn transfer_ui_resource_requests(&mut self) {
        self.tree
            .set_ui_resource_request_queue(self.commit.ui_resource_request_queue(), self.context);
    }

    fn update_property_tree_animation(&mut self) {
        self.tree
            .UpdatePropertyTreeAnimationFromMainThread(self.context);
    }

    fn push_mutator_properties(&mut self) {
        self.context
            .host
            .PushMutatorPropertiesFrom(self.unsafe_state, self.commit.property_trees());
    }

    fn move_change_tracking_to_layers(&mut self) {
        self.tree.MoveChangeTrackingToLayers(self.context);
    }

    fn update_scrollbar_geometries(&mut self) {
        self.tree.UpdateAllScrollbarGeometries(self.context);
    }

    fn finish_sync(&mut self) {
        self.tree.lifecycle_.AdvanceTo(LifecycleState::kNotSyncing);
    }
}

struct TreeCommitPropertyAdapter<'a, 'context, B: LayerTreeTypes> {
    tree: &'a mut LayerTreeImpl<B>,
    commit: &'a mut B::CommitState,
    context: &'a mut LayerTreeContext<'context, B>,
}

impl<B: LayerTreeTypes> compositor_tree::commit_properties::CommitPropertyContext
    for TreeCommitPropertyAdapter<'_, '_, B>
{
    fn trees_in_viz_client_process(&self) -> bool {
        self.context.host.TreesInVizInClientProcess()
    }

    fn current_needs_full_tree_sync(&self) -> bool {
        self.tree.needs_full_tree_sync_
    }

    fn commit_needs_full_tree_sync(&self) -> bool {
        self.commit.needs_full_tree_sync()
    }

    fn set_needs_full_tree_sync(&mut self, value: bool) {
        self.tree.set_needs_full_tree_sync(value);
    }

    fn update_hud_layer(&mut self) {
        self.tree.hud_layer_ = if self.commit.hud_layer_id() != -1 {
            self.tree
                .LayerById(self.commit.hud_layer_id())
                .map(|layer| layer.id())
        } else {
            None
        };
    }

    fn pull_background_color(&mut self) {
        self.tree
            .set_background_color(self.commit.background_color());
    }

    fn pull_scroll_event_handler_state(&mut self) {
        self.tree
            .set_have_scroll_event_handlers(self.commit.have_scroll_event_handlers());
    }

    fn event_listener_class_count(&self) -> usize {
        3
    }

    fn pull_event_listener_properties(&mut self, index: usize) {
        let class = [
            EventListenerClass::kTouchStartOrMove,
            EventListenerClass::kMouseWheel,
            EventListenerClass::kTouchEndOrCancel,
        ][index];
        self.tree
            .set_event_listener_properties(class, self.commit.GetEventListenerProperties(class));
    }

    fn pull_property_change_commit_criteria(&mut self) {
        self.tree.property_change_forces_commit_criteria_ =
            self.commit.property_change_forces_commit_criteria();
    }

    fn pull_viewport_property_ids(&mut self) {
        self.tree
            .SetViewportPropertyIds(&self.commit.viewport_property_ids());
    }

    fn pull_selection(&mut self) {
        self.tree.RegisterSelection(self.commit.selection());
    }

    fn pull_page_scale(&mut self) {
        self.tree.PushPageScaleFromMainThread(
            self.commit.page_scale_factor(),
            self.commit.min_page_scale_factor(),
            self.commit.max_page_scale_factor(),
            self.commit.page_scale_factor_limits_set(),
            self.context,
        );
    }

    fn pull_browser_controls_params(&mut self) {
        self.tree
            .SetBrowserControlsParams(self.commit.browser_controls_params(), self.context);
    }

    fn pull_overscroll_behavior(&mut self) {
        self.tree
            .set_overscroll_behavior(&self.commit.overscroll_behavior());
    }

    fn pull_browser_controls_ratios(&mut self) {
        self.tree.PushBrowserControlsFromMainThread(
            self.commit.top_controls_shown_ratio(),
            self.commit.bottom_controls_shown_ratio(),
            self.context,
        );
    }

    fn pull_display_color_spaces(&mut self) {
        self.tree
            .SetDisplayColorSpaces(self.commit.display_color_spaces());
    }

    fn pull_external_page_scale(&mut self) {
        self.tree
            .SetExternalPageScaleFactor(self.commit.external_page_scale_factor(), self.context);
    }

    fn pull_painted_device_scale(&mut self) {
        self.tree
            .set_painted_device_scale_factor(self.commit.painted_device_scale_factor());
    }

    fn pull_device_scale(&mut self) {
        self.tree
            .SetDeviceScaleFactor(self.commit.device_scale_factor(), self.context);
    }

    fn pull_device_viewport(&mut self) {
        self.tree
            .SetDeviceViewportRect(&self.commit.device_viewport_rect(), self.context);
    }

    fn pull_max_safe_area_inset(&mut self) {
        self.tree
            .SetMaxSafeAreaInsetBottom(self.commit.max_safe_area_inset_bottom());
    }

    fn commit_requests_new_local_surface_id(&self) -> bool {
        self.commit.new_local_surface_id_request()
    }

    fn request_new_local_surface_id(&mut self) {
        self.tree.RequestNewLocalSurfaceId();
    }

    fn commit_has_screenshot_destination(&self) -> bool {
        !self.commit.screenshot_destination_token().is_empty()
    }

    fn pull_screenshot_destination(&mut self) {
        self.tree
            .SetScreenshotDestinationToken(self.commit.screenshot_destination_token());
    }

    fn pull_primary_main_frame_sequence(&mut self) {
        self.tree.set_primary_main_frame_item_sequence_number(
            self.commit.primary_main_frame_item_sequence_number(),
        );
    }

    fn pull_parent_local_surface_id(&mut self) {
        self.tree
            .SetLocalSurfaceIdFromParent(self.commit.local_surface_id_from_parent());
    }

    fn pull_pending_page_scale_animation(&mut self) -> bool {
        let Some(animation) = self.commit.TakePendingPageScaleAnimation() else {
            return false;
        };
        self.tree.SetPendingPageScaleAnimation(Some(animation));
        true
    }

    fn commit_requests_force_send_metadata(&self) -> bool {
        self.commit.force_send_metadata_request()
    }

    fn request_force_send_metadata(&mut self) {
        self.tree.RequestForceSendMetadata();
    }

    fn pull_display_transform_hint(&mut self) {
        self.tree
            .set_display_transform_hint(self.commit.display_transform_hint());
    }

    fn pull_delegated_ink_metadata(&mut self) {
        self.tree.delegated_ink_metadata_ = self.commit.TakeDelegatedInkMetadata();
    }

    fn view_transition_request_count(&mut self) -> usize {
        self.commit.ViewTransitionRequests().len()
    }

    fn pull_view_transition_request(&mut self, index: usize) {
        let request = self.commit.ViewTransitionRequests()[index].take().unwrap();
        self.tree.AddViewTransitionRequest(request, self.context);
    }
}

impl<B: LayerTreeTypes> LayerTreeImpl<B> {
    // Source: cc/trees/layer_tree_impl.cc:739-828.
    pub fn PullPropertiesFrom(
        &mut self,
        commit: &mut B::CommitState,
        unsafe_state: &B::UnsafeCommitState,
        context: &mut LayerTreeContext<'_, B>,
        main_context: &mut <B::Synchronizer as LayerTreeSynchronizer<B>>::MainContext<'_, '_>,
    ) {
        compositor_tree::commit_sync::PullCommitIntoSyncTree(&mut TreeCommitSyncAdapter {
            tree: self,
            commit,
            unsafe_state,
            context,
            main_context,
        });
    }
    // Source: cc/trees/layer_tree_impl.cc:830-849.
    pub fn PullPropertyTreesFrom(
        &mut self,
        commit: &mut B::CommitState,
        unsafe_state: &B::UnsafeCommitState,
        context: &mut LayerTreeContext<'_, B>,
    ) {
        let mut preserve = false;
        if unsafe_state.HasRootLayer()
            && self.IsActiveTree(context)
            && self.property_trees_.changed()
        {
            if commit.property_trees().sequence_number() == self.property_trees_.sequence_number() {
                preserve = true;
            } else {
                self.MoveChangeTrackingToLayers(context);
            }
        }
        let ids = commit.viewport_property_ids();
        self.SetPropertyTrees(commit.property_trees_mut(), &ids, preserve, context);
    }
    // Source: cc/trees/layer_tree_impl.cc:851-937.
    pub fn PullLayerTreePropertiesFrom(
        &mut self,
        commit: &mut B::CommitState,
        context: &mut LayerTreeContext<'_, B>,
    ) {
        compositor_tree::commit_properties::PullCommitProperties(&mut TreeCommitPropertyAdapter {
            tree: self,
            commit,
            context,
        });
    }
    // Source: cc/trees/layer_tree_impl.cc:939-978.
    pub fn PushPropertyTreesTo(
        &mut self,
        target: &mut Self,
        context: &mut LayerTreeContext<'_, B>,
    ) {
        let mut preserve = false;
        if target.property_trees_.changed() {
            if self.property_trees_.sequence_number() == target.property_trees_.sequence_number() {
                preserve = true;
            } else {
                target.MoveChangeTrackingToLayers(context);
            }
        }
        target.SetPropertyTrees(
            &mut self.property_trees_,
            &self.viewport_property_ids_,
            preserve,
            context,
        );
        target.AppendEventsMetricsFromMainThread(std::mem::take(
            &mut self.events_metrics_from_main_thread_,
        ));
        target.AppendEventMetricsFromRasterThread(std::mem::take(
            &mut self.event_metrics_from_raster_thread_,
        ));
    }
    // Source: cc/trees/layer_tree_impl.cc:980-987.
    pub fn PushSurfaceRangesTo(&mut self, target: &mut Self) {
        if self.needs_surface_ranges_sync() {
            target.ClearSurfaceRanges();
            target.SetSurfaceRanges(self.SurfaceRanges().clone());
            self.set_needs_surface_ranges_sync(false);
        }
    }
}

// Source: cc/trees/layer_tree_impl.h:844-864;
//         cc/trees/layer_tree_impl.cc:3384-3398.
// A unique scoped marker replaces the source non-owning updater pointer. The
// guard holds the tree's live borrow, with queued layer identities held exactly
// as the source scoped helper does; Drop performs the source regeneration loop.
pub(crate) struct DiscardableImageMapUpdaterMarker;
pub struct DiscardableImageMapUpdater<'a, 'c, B: LayerTreeTypes> {
    tree: &'a mut LayerTreeImpl<B>,
    context: &'a mut LayerTreeContext<'c, B>,
    layers_needing_update_: Vec<i32>,
}
impl<'a, 'c, B: LayerTreeTypes> DiscardableImageMapUpdater<'a, 'c, B> {
    fn new(tree: &'a mut LayerTreeImpl<B>, context: &'a mut LayerTreeContext<'c, B>) -> Self {
        assert!(tree.IsSyncTree(context));
        assert!(tree.discardable_image_map_updater_.is_none());
        tree.discardable_image_map_updater_ = Some(DiscardableImageMapUpdaterMarker);
        Self {
            tree,
            context,
            layers_needing_update_: Vec::new(),
        }
    }
    fn split(
        &mut self,
    ) -> (
        &mut LayerTreeImpl<B>,
        &mut Vec<i32>,
        &mut LayerTreeContext<'c, B>,
    ) {
        (self.tree, &mut self.layers_needing_update_, self.context)
    }
    pub fn AddLayerNeedingUpdate(&mut self, id: i32) {
        self.layers_needing_update_.push(id);
    }
}
impl<B: LayerTreeTypes> Drop for DiscardableImageMapUpdater<'_, '_, B> {
    fn drop(&mut self) {
        debug_assert!(self.tree.discardable_image_map_updater_.is_some());
        B::WithPictureLayersContext(self.tree, self.context.host, |layers, layer_context| {
            for id in &self.layers_needing_update_ {
                layers
                    .LayerByIdMut(*id)
                    .unwrap()
                    .AsPictureLayerMut()
                    .unwrap()
                    .RegenerateDiscardableImageMap(layer_context);
            }
        });
        self.tree.discardable_image_map_updater_ = None;
    }
}

/// Resident-object adapter for the Tree-owned pending-to-active property push
/// pipeline. Each method performs one named transfer and contains no
/// cross-stage ordering.
struct TreePropertyPushAdapter<'a, 'services, B: LayerTreeTypes> {
    source: &'a mut LayerTreeImpl<B>,
    target: &'a mut LayerTreeImpl<B>,
    context: &'a mut LayerTreeContext<'services, B>,
}

impl<B: LayerTreeTypes> compositor_tree::property_push::PropertyPushContext
    for TreePropertyPushAdapter<'_, '_, B>
{
    type ViewTransitionRequest = Box<B::ViewTransitionRequest>;

    fn assert_source_ui_resource_queue_empty(&self) {
        debug_assert!(self.source.ui_resource_request_queue_.is_empty());
    }

    fn push_surface_ranges(&mut self) {
        self.source.PushSurfaceRangesTo(self.target);
    }

    fn push_pending_scroll_updates(&mut self) {
        let (scroll, mut mutation) = self.target.property_trees_.scroll_tree_mutation_context();
        let mut facade = LayerTreeScrollFacade {
            id: self.target.id,
            host: self.context.host,
            layer_list: &mut LayerTreeWholeScrollStore::<B>(&mut self.target.layer_list_),
            scrollbar_ids: &self.target.element_id_to_scrollbar_layer_ids_,
            viewport: &self.target.viewport_property_ids_,
            page_scale: &self.target.page_scale_factor_,
            needs_update_draw_properties: &mut self.target.needs_update_draw_properties_,
            show_requests: &mut self.target.show_scrollbar_requests_,
        };
        scroll.PushScrollUpdatesFromPendingTree(
            self.source.property_trees_.scroll_tree(),
            self.source.property_trees_.state(),
            &mut facade,
            &mut mutation,
        );
    }

    fn take_force_redraw(&mut self) -> bool {
        let value = self.source.next_activation_forces_redraw_;
        self.source.next_activation_forces_redraw_ = false;
        value
    }

    fn force_target_redraw(&mut self) {
        self.target.ForceRedrawNextActivation();
    }

    fn transfer_swap_promises(&mut self) {
        self.target.PassSwapPromises(
            std::mem::take(&mut self.source.swap_promise_list_)
                .into_iter()
                .map(|promise| promise.expect("non-null swap promise"))
                .collect(),
            self.context,
        );
        self.source.swap_promise_list_.clear();
    }

    fn transfer_viewport_property_ids(&mut self) {
        self.target
            .SetViewportPropertyIds(&self.source.viewport_property_ids_);
    }

    fn transfer_page_scale_and_limits(&mut self) {
        let min_scale = self.source.min_page_scale_factor();
        let max_scale = self.source.max_page_scale_factor();
        let limits_set = self.source.page_scale_factor_limits_set_;
        self.context
            .host
            .WithBorrowedTree(self.source, &mut |host| {
                self.target.PushPageScaleFactorAndLimits(
                    None,
                    min_scale,
                    max_scale,
                    limits_set,
                    &mut LayerTreeContext { host },
                );
            });
    }

    fn transfer_external_page_scale(&mut self) {
        self.target
            .SetExternalPageScaleFactor(self.source.external_page_scale_factor_, self.context);
    }

    fn transfer_browser_controls_params(&mut self) {
        let controls_params = self.source.browser_controls_params_;
        self.context
            .host
            .WithBorrowedTree(self.source, &mut |host| {
                self.target
                    .SetBrowserControlsParams(&controls_params, &mut LayerTreeContext { host });
            });
    }

    fn transfer_browser_controls_values(&mut self) {
        self.context
            .host
            .WithBorrowedTree(self.source, &mut |host| {
                self.target
                    .PushBrowserControls(None, None, &mut LayerTreeContext { host });
            });
    }

    fn transfer_overscroll_behavior(&mut self) {
        self.target
            .set_overscroll_behavior(&self.source.overscroll_behavior_);
    }

    fn transfer_display_color_spaces(&mut self) {
        self.target
            .SetDisplayColorSpaces(&self.source.display_color_spaces_);
    }

    fn transfer_painted_device_scale(&mut self) {
        self.target
            .set_painted_device_scale_factor(self.source.painted_device_scale_factor());
    }

    fn transfer_device_scale(&mut self) {
        self.target
            .SetDeviceScaleFactor(self.source.device_scale_factor(), self.context);
    }

    fn transfer_device_viewport(&mut self) {
        self.target
            .SetDeviceViewportRect(&self.source.device_viewport_rect_, self.context);
    }

    fn transfer_max_safe_area_inset(&mut self) {
        self.target
            .SetMaxSafeAreaInsetBottom(self.source.max_safe_area_inset_bottom_);
    }

    fn take_new_local_surface_id_request(&mut self) -> bool {
        self.source.TakeNewLocalSurfaceIdRequest()
    }

    fn request_target_local_surface_id(&mut self) {
        self.target.RequestNewLocalSurfaceId();
    }

    fn transfer_parent_local_surface_id(&mut self) {
        self.target
            .SetLocalSurfaceIdFromParent(self.source.local_surface_id_from_parent());
    }

    fn transfer_screenshot_destination(&mut self) -> bool {
        let token = self.source.TakeScreenshotDestinationToken();
        if token.is_empty() {
            return false;
        }
        self.target.SetScreenshotDestinationToken(token);
        true
    }

    fn transfer_primary_item_sequence(&mut self) {
        self.target.set_primary_main_frame_item_sequence_number(
            self.source.primary_main_frame_item_sequence_number(),
        );
    }

    fn transfer_pending_page_scale_animation(&mut self) -> bool {
        let animation = self.source.pending_page_scale_animation_.take();
        let transferred = animation.is_some();
        self.target.pending_page_scale_animation_ = animation;
        transferred
    }

    fn take_force_send_metadata_request(&mut self) -> bool {
        self.source.TakeForceSendMetadataRequest()
    }

    fn request_target_force_send_metadata(&mut self) {
        self.target.RequestForceSendMetadata();
    }

    fn transfer_selection(&mut self) {
        self.target.RegisterSelection(&self.source.selection_);
    }

    fn transfer_frame_identity(&mut self) {
        self.target
            .set_source_frame_number(self.source.source_frame_number());
        self.target.set_trace_id(self.source.trace_id());
    }

    fn transfer_background_and_event_state(&mut self) {
        self.target
            .set_background_color(self.source.background_color());
        self.target
            .set_have_scroll_event_handlers(self.source.have_scroll_event_handlers());
        self.target.did_raster_inducing_scroll_ = self.source.did_raster_inducing_scroll_;
    }

    fn transfer_event_listener_properties(&mut self) {
        for class in [
            EventListenerClass::kTouchStartOrMove,
            EventListenerClass::kMouseWheel,
            EventListenerClass::kTouchEndOrCancel,
        ] {
            self.target
                .set_event_listener_properties(class, self.source.event_listener_properties(class));
        }
    }

    fn transfer_hud_layer(&mut self) {
        self.target.hud_layer_ = self
            .source
            .hud_layer_
            .and_then(|id| self.target.LayerById(id).map(|layer| layer.id()));
    }

    fn update_tickmark_visibility(&mut self) {
        self.target.HandleTickmarksVisibilityChange(self.context);
    }

    fn update_scrollbar_geometries(&mut self) {
        self.target.UpdateAllScrollbarGeometries(self.context);
    }

    fn handle_scrollbar_show_requests(&mut self) {
        self.target.HandleScrollbarShowRequests(self.context);
    }

    fn transfer_presentation_callbacks(&mut self) {
        self.target
            .AddPresentationCallbacks(std::mem::take(&mut self.source.presentation_callbacks_));
        self.source.presentation_callbacks_.clear();
    }

    fn transfer_successful_presentation_callbacks(&mut self) {
        self.target
            .AddSuccessfulPresentationCallbacks(std::mem::take(
                &mut self.source.successful_presentation_callbacks_,
            ));
        self.source.successful_presentation_callbacks_.clear();
    }

    fn transfer_delegated_ink_metadata(&mut self) -> bool {
        if let Some(metadata) = self.source.delegated_ink_metadata_.take() {
            self.context.host.TraceDelegatedInkMetadataPush(&metadata);
            self.target.set_delegated_ink_metadata(Some(metadata));
            true
        } else {
            if self.target.delegated_ink_metadata().is_some() {
                self.target.clear_delegated_ink_metadata();
            }
            false
        }
    }

    fn take_view_transition_requests(&mut self) -> Vec<Self::ViewTransitionRequest> {
        self.source.TakeViewTransitionRequests(true)
    }

    fn push_view_transition_request(&mut self, request: Self::ViewTransitionRequest) {
        let mut request = Some(request);
        self.context
            .host
            .WithBorrowedTree(self.source, &mut |host| {
                self.target.AddViewTransitionRequest(
                    request.take().unwrap(),
                    &mut LayerTreeContext { host },
                );
            });
    }

    fn trees_in_viz_in_client_process(&self) -> bool {
        self.context.host.TreesInVizInClientProcess()
    }

    fn move_change_tracking_to_layers(&mut self) {
        self.target.MoveChangeTrackingToLayers(self.context);
    }

    fn update_property_tree_change_tracking(&mut self) {
        self.target.property_trees_.UpdateChangeTracking();
    }

    fn transfer_created_begin_frame_args(&mut self) {
        self.target
            .SetCreatedBeginFrameArgs(std::mem::take(&mut self.source.created_begin_frame_args_));
    }
}

struct DrawOcclusionWalkAdapter<'a, B: LayerTreeTypes> {
    tree: &'a mut LayerTreeImpl<B>,
    iterator: crate::cc::layers::effect_tree_layer_list_iterator::EffectTreeLayerListIterator,
    tracker: super::occlusion_tracker::OcclusionTracker<B::Transform, i32, i32>,
}

impl<'a, B: LayerTreeTypes> DrawOcclusionWalkAdapter<'a, B> {
    fn new(tree: &'a mut LayerTreeImpl<B>, minimum_tracking_size: Size) -> Self {
        use crate::adapter::draw_property::DrawSurface;
        let content_rect = DrawSurface::content_rect(tree.RootRenderSurface().unwrap());
        let iterator =
            crate::cc::layers::effect_tree_layer_list_iterator::EffectTreeLayerListIterator::new(
                &*tree,
            );
        let mut tracker =
            super::occlusion_tracker::OcclusionTracker::<B::Transform, i32, i32>::new(content_rect);
        tracker.set_minimum_tracking_size(minimum_tracking_size);
        Self {
            tree,
            iterator,
            tracker,
        }
    }
}

impl<B: LayerTreeTypes> compositor_draw::occlusion_walk::OcclusionWalkContext
    for DrawOcclusionWalkAdapter<'_, B>
{
    type Surface = i32;
    type Layer = i32;

    fn current_entry(
        &mut self,
    ) -> Option<compositor_draw::occlusion_walk::OcclusionWalkEntry<i32, i32>> {
        use crate::adapter::occlusion::EffectTreeLayerListIteratorState as DrawState;
        use crate::cc::layers::effect_tree_layer_list_iterator::State;
        if self.iterator.state() == State::kEnd {
            return None;
        }
        let native = self.iterator.position(&*self.tree);
        Some(compositor_draw::occlusion_walk::OcclusionWalkEntry {
            position: crate::adapter::occlusion::EffectTreeLayerListIteratorPosition {
                state: match native.state {
                    State::kLayer => DrawState::kLayer,
                    State::kTargetSurface => DrawState::kTargetSurface,
                    State::kContributingSurface => DrawState::kContributingSurface,
                    State::kEnd => unreachable!(),
                },
                target_render_surface: native.target_render_surface.unwrap(),
                current_layer: native.current_layer,
            },
            current_render_surface: native.current_render_surface,
        })
    }

    fn enter(
        &mut self,
        position: &crate::adapter::occlusion::EffectTreeLayerListIteratorPosition<i32, i32>,
    ) {
        let occlusion = B::MakeOcclusionContext(
            &self.tree.layer_list_,
            &self.tree.property_trees_,
            &self.tree.render_surface_list_,
        );
        self.tracker.EnterLayer(position, &occlusion);
    }

    fn assign_layer_occlusion(&mut self, layer_id: i32) {
        let transform = self
            .tree
            .layer_list_
            .LayerById(layer_id)
            .unwrap()
            .DrawTransform(&self.tree.property_trees_);
        let occlusion = self.tracker.GetCurrentOcclusionForLayer(&transform);
        self.tree
            .layer_list_
            .LayerByIdMut(layer_id)
            .unwrap()
            .SetOcclusionInContentSpace(occlusion);
    }

    fn assign_contributing_surface_occlusion(&mut self, surface_id: i32) {
        use crate::adapter::draw_property::DrawSurface;
        let surface = self
            .tree
            .property_trees_
            .effect_tree()
            .GetRenderSurface(surface_id)
            .unwrap();
        let mut transform = B::Transform::default();
        if let Some(occlusion_id) = self.tracker.OcclusionSurfaceForContributingSurface() {
            let target = self
                .tree
                .property_trees_
                .effect_tree()
                .GetRenderSurface(occlusion_id)
                .unwrap();
            self.tree.property_trees_.GetToTarget(
                DrawSurface::TransformTreeIndex(surface, &self.tree.property_trees_),
                DrawSurface::EffectTreeIndex(target),
                &mut transform,
            );
            let node = self
                .tree
                .property_trees_
                .effect_tree()
                .Node(DrawSurface::EffectTreeIndex(surface));
            crate::cc::trees::draw_property_utils::ConcatInverseSurfaceContentsScale::<
                B::DrawContext<'_>,
            >(node, &mut transform);
            crate::adapter::property_tree::PropertyTreeTransform::PostTranslate(
                &mut transform,
                DrawSurface::pixel_alignment_offset(target),
            );
        }
        let occlusion = self
            .tracker
            .GetCurrentOcclusionForContributingSurface(&transform);
        self.tree
            .property_trees_
            .effect_tree_mutable()
            .GetRenderSurfaceMutable(surface_id)
            .unwrap()
            .SetOcclusionInContentSpace(occlusion);
    }

    fn leave(
        &mut self,
        position: &crate::adapter::occlusion::EffectTreeLayerListIteratorPosition<i32, i32>,
    ) {
        let occlusion = B::MakeOcclusionContext(
            &self.tree.layer_list_,
            &self.tree.property_trees_,
            &self.tree.render_surface_list_,
        );
        self.tracker.LeaveLayer(position, &occlusion);
    }

    fn advance(&mut self) {
        self.iterator.Increment(&*self.tree);
    }

    fn finish_visible_region(&mut self) {
        let occlusion = B::MakeOcclusionContext(
            &self.tree.layer_list_,
            &self.tree.property_trees_,
            &self.tree.render_surface_list_,
        );
        self.tree.unoccluded_screen_space_region_ = self
            .tracker
            .ComputeVisibleRegionInScreen::<B::RegionBackend>(&occlusion);
    }
}

struct DrawTreeUpdateAdapter<'a, B: LayerTreeTypes> {
    tree: &'a mut LayerTreeImpl<B>,
    host: &'a mut dyn draw_host::LayerTreeDrawHost<B>,
    output_updates: Option<&'a mut Vec<i32>>,
}

impl<B: LayerTreeTypes> compositor_draw::tree_draw_update::TreeDrawUpdateContext
    for DrawTreeUpdateAdapter<'_, B>
{
    fn needs_draw_property_update(&self) -> bool {
        self.tree.needs_update_draw_properties_
    }

    fn begin_draw_property_update(&mut self) {
        self.tree.needs_update_draw_properties_ = false;
        self.tree.needs_update_tiles_ = false;
    }

    fn has_frame_sink(&self) -> bool {
        self.host.HasLayerTreeFrameSink()
    }

    fn clear_render_surface_list(&mut self) {
        self.tree.render_surface_list_.clear();
    }

    fn layer_list_is_empty(&self) -> bool {
        self.tree.layer_list_.is_empty()
    }

    fn calculate_draw_properties(&mut self) {
        let tokens = self.tree.GetCaptureViewTransitionTokensWithHost(self.host);
        let mut draw = B::MakeDrawContext(
            LayerTreeDrawState {
                layers: &mut self.tree.layer_list_,
                viewport: &self.tree.viewport_property_ids_,
                page_scale: &self.tree.page_scale_factor_,
                device_scale: &self.tree.device_scale_factor_,
                external_scale: &self.tree.external_page_scale_factor_,
                device_viewport: &self.tree.device_viewport_rect_,
                capture_tokens: &tokens,
                tree_id: self.tree.id,
                view_transition_requests: &self.tree.view_transition_requests_,
            },
            self.host,
        );
        crate::cc::trees::draw_property_utils::CalculateDrawProperties(
            &mut draw,
            &mut self.tree.property_trees_,
            &mut self.tree.render_surface_list_,
            self.output_updates.take(),
        );
    }

    fn run_occlusion_walk(&mut self) {
        compositor_draw::occlusion_walk::RunOcclusionWalk(&mut DrawOcclusionWalkAdapter::new(
            self.tree,
            self.host.settings().minimum_occlusion_tracking_size(),
        ));
    }

    fn is_resourceless_software_draw(&self) -> bool {
        self.host.IsResourcelessSoftwareDraw()
    }

    fn mark_tiles_need_update(&mut self) {
        self.tree.needs_update_tiles_ = true;
    }

    fn update_tiles(&mut self) -> bool {
        self.tree.UpdateTilesWithHost(self.host)
    }

    fn did_modify_tile_priorities(&mut self, deferred: bool) {
        self.host.DidModifyTilePriorities(deferred);
    }

    fn has_image_animation_controller(&self) -> bool {
        self.host.HasImageAnimationController()
    }

    fn update_image_animation_state(&mut self) {
        assert!(!self.host.settings().trees_in_viz_in_viz_process());
        self.host.UpdateImageAnimationStateFromDrivers(self.tree);
    }

    fn finish_draw_property_update(&mut self) {
        self.tree.device_viewport_rect_changed_ = false;
    }

    fn assert_draw_properties_clean(&self) {
        debug_assert!(
            !self.tree.needs_update_draw_properties_,
            "CalcDrawProperties should not set_needs_update_draw_properties()"
        );
    }
}

struct ResidentPictureLayerUpdateAdapter<'a, 'context, B>
where
    B: LayerTreeTypes + 'context,
{
    layer: &'a mut B::PictureLayer,
    context: &'a mut B::PictureLayerContext<'context>,
}

impl<B: LayerTreeTypes> compositor_tile::picture_layer_update::PictureLayerUpdate
    for ResidentPictureLayerUpdateAdapter<'_, '_, B>
{
    fn has_valid_tile_priorities(&self) -> bool {
        self.layer.HasValidTilePriorities(self.context)
    }

    fn release_resources(&mut self) {
        self.layer.ReleaseResources(self.context);
    }

    fn update_tiles(&mut self) -> bool {
        self.layer.UpdateTiles(self.context)
    }
}

struct TilePictureLayerUpdateAdapter<'a, B: LayerTreeTypes> {
    tree: &'a mut LayerTreeImpl<B>,
    host: &'a mut dyn draw_host::LayerTreeDrawHost<B>,
}

impl<B: LayerTreeTypes> compositor_tile::picture_layer_update::PictureLayerUpdateContext
    for TilePictureLayerUpdateAdapter<'_, B>
{
    fn needs_tile_update(&self) -> bool {
        self.tree.needs_update_tiles_
    }

    fn begin_tile_update(&mut self) {
        self.tree.needs_update_tiles_ = false;
    }

    fn release_resources_for_hidden_layers(&self) -> bool {
        self.host
            .settings()
            .release_tile_resources_for_hidden_layers()
    }

    fn visit_picture_layers(
        &mut self,
        visitor: &mut dyn FnMut(&mut dyn compositor_tile::picture_layer_update::PictureLayerUpdate),
    ) {
        B::WithPictureLayersContext(self.tree, self.host, |layers, layer_context| {
            let _ = layers.ForEachPictureLayer(&mut |layer| {
                visitor(&mut ResidentPictureLayerUpdateAdapter::<B> {
                    layer,
                    context: layer_context,
                });
                std::ops::ControlFlow::Continue(())
            });
        });
    }

    fn report_layers_updated(&mut self, count: usize) {
        self.host.TraceLayersUpdatedCount(count);
    }
}

impl<B: LayerTreeTypes> LayerTreeImpl<B> {
    // Source: cc/trees/layer_tree_impl.cc:989-1121.
    pub fn PushPropertiesTo(&mut self, target: &mut Self, context: &mut LayerTreeContext<'_, B>) {
        compositor_tree::property_push::PushPropertiesToActive(&mut TreePropertyPushAdapter {
            source: self,
            target,
            context,
        });
    }
    // Source: cc/trees/layer_tree_impl.cc:1123-1146.
    pub fn HandleTickmarksVisibilityChange(&mut self, context: &mut LayerTreeContext<'_, B>) {
        let Some(id) = context
            .host
            .OuterViewportScrollNode(self)
            .map(|node| node.element_id)
        else {
            return;
        };
        context
            .host
            .WithScrollbarAnimationController(self, id, &mut |controller, client| {
                let Some(controller) = controller else {
                    return;
                };
                for scrollbar in controller.Scrollbars(client) {
                    if client.Scrollbar_orientation(&scrollbar) != ScrollbarOrientation::kVertical {
                        continue;
                    }
                    if !client.Scrollbar_UsesAuraOverlayAnimator(&scrollbar) {
                        debug_assert!(!client.Scrollbar_has_find_in_page_tickmarks(&scrollbar));
                    }
                    let has_tickmarks = client.Scrollbar_has_find_in_page_tickmarks(&scrollbar);
                    controller.UpdateTickmarksVisibility(client, has_tickmarks);
                }
            });
    }
    // Source: cc/trees/layer_tree_impl.cc:1148-1152.
    pub fn RequestShowScrollbars(&mut self, id: ElementId, context: &LayerTreeContext<'_, B>) {
        RequestShowScrollbarsCore(
            self.id,
            id,
            context.host,
            &mut self.show_scrollbar_requests_,
        );
    }
    // Source: cc/trees/layer_tree_impl.cc:1154-1163.
    pub fn HandleScrollbarShowRequests(&mut self, context: &mut LayerTreeContext<'_, B>) {
        // The source set stays resident during synchronous callbacks. Copy only
        // the current native ElementId, then reborrow the next live set entry.
        let mut previous = None;
        loop {
            let id = match previous {
                None => self.show_scrollbar_requests_.iter().next().copied(),
                Some(previous) => self
                    .show_scrollbar_requests_
                    .range((
                        std::ops::Bound::Excluded(previous),
                        std::ops::Bound::Unbounded,
                    ))
                    .next()
                    .copied(),
            };
            let Some(id) = id else {
                break;
            };
            previous = Some(id);
            context
                .host
                .WithScrollbarAnimationController(self, id, &mut |controller, client| {
                    if let Some(controller) = controller {
                        controller.DidRequestShow(client);
                    }
                });
        }
        self.show_scrollbar_requests_.clear();
    }
    // Source: cc/trees/layer_tree_impl.cc:1165-1183.
    pub fn MoveChangeTrackingToLayers(&mut self, context: &mut LayerTreeContext<'_, B>) {
        self.property_trees_.UpdateChangeTracking();
        self.ForEachLayerWithContext(false, context, |layer, layer_context| {
            if layer.LayerPropertyChanged(layer_context.PropertyTrees()) {
                if layer.LayerPropertyChangedFromPropertyTrees(layer_context.PropertyTrees()) {
                    layer.NoteLayerPropertyChangedFromPropertyTrees(layer_context);
                }
            }
        });
        for id in kContentsRootPropertyNodeId..self.property_trees_.effect_tree().size() as i32 {
            // Source cc1176–1181: finish the same surface's tree query before
            // uniquely borrowing its mutable flag; no tree/layer copy is made.
            if self
                .property_trees_
                .effect_tree()
                .GetRenderSurface(id)
                .is_some_and(|surface| surface.AncestorPropertyChanged(&self.property_trees_))
            {
                self.property_trees_
                    .effect_tree_mutable()
                    .GetRenderSurfaceMutable(id)
                    .expect("source existing surface")
                    .NoteAncestorPropertyChanged();
            }
        }
    }
    // Source: cc/trees/layer_tree_impl.cc:1185-1189.
    pub fn ForceRecalculateRasterScales(&mut self) {
        let _ = self.layer_list_.ForEachPictureLayer(&mut |layer| {
            layer.ResetRasterScale();
            std::ops::ControlFlow::Continue(())
        });
    }
    // Source: cc/trees/layer_tree_impl.cc:1191-1197.
    pub fn IsElementInPropertyTree(&self, id: ElementId) -> bool {
        self.property_trees_.HasElement(id)
    }
    fn GetElementTypeForAnimation(&self, context: &LayerTreeContext<'_, B>) -> ElementListType {
        if self.IsActiveTree(context) {
            ElementListType::ACTIVE
        } else {
            ElementListType::PENDING
        }
    }
    // Source: cc/trees/layer_tree_impl.cc:1199-1208.
    pub fn ValidateEffectTreeMapping(
        &self,
        id: ElementId,
        mutation: PropertyMutation,
        context: &LayerTreeContext<'_, B>,
    ) {
        let count = usize::from(
            self.property_trees_
                .effect_tree()
                .element_id_to_node_index()
                .contains_key(&id),
        );
        if count != 1 {
            context.host.MissingPropertyNodeForElementId(mutation);
        }
    }
    // Source: cc/trees/layer_tree_impl.cc:1210-1235.
    pub fn RequestCommitForPropertyMutationIfNeeded(
        &self,
        mutation: PropertyMutation,
        context: &mut LayerTreeContext<'_, B>,
    ) {
        if !self.IsPendingTree(context) && !self.IsRecycleTree(context) {
            return;
        }
        let needs_commit = match mutation {
            PropertyMutation::kTransform => {
                self.property_change_forces_commit_criteria_
                    != PropertyChangeForcesCommitCriteria::kNone
            }
            PropertyMutation::kOpacity
            | PropertyMutation::kFilter
            | PropertyMutation::kBackdropFilter => {
                self.property_change_forces_commit_criteria_
                    == PropertyChangeForcesCommitCriteria::kAny
            }
        };
        if needs_commit {
            context.host.SetNeedsCommit();
        }
    }
    // Source: cc/trees/layer_tree_impl.cc:1237-1245.
    pub fn SetTransformMutated(
        &mut self,
        id: ElementId,
        transform: &B::Transform,
        context: &mut LayerTreeContext<'_, B>,
    ) {
        self.ValidateEffectTreeMapping(id, PropertyMutation::kTransform, context);
        let (tree, mut mutation) = self.property_trees_.transform_tree_with_context();
        if tree.OnTransformAnimated(id, transform, &mut mutation) {
            self.set_needs_update_draw_properties();
            self.RequestCommitForPropertyMutationIfNeeded(PropertyMutation::kTransform, context);
        }
    }
    // Source: cc/trees/layer_tree_impl.cc:1247-1275.
    pub fn SetOpacityMutated(
        &mut self,
        id: ElementId,
        opacity: f32,
        context: &mut LayerTreeContext<'_, B>,
    ) {
        self.ValidateEffectTreeMapping(id, PropertyMutation::kOpacity, context);
        let (tree, state) = self.property_trees_.effect_tree_with_state();
        if tree.OnOpacityAnimated(id, opacity, state) {
            self.set_needs_update_draw_properties();
            self.RequestCommitForPropertyMutationIfNeeded(PropertyMutation::kOpacity, context);
        }
    }
    pub fn SetFilterMutated(
        &mut self,
        id: ElementId,
        filters: &B::Filters,
        context: &mut LayerTreeContext<'_, B>,
    ) {
        self.ValidateEffectTreeMapping(id, PropertyMutation::kFilter, context);
        let (tree, state) = self.property_trees_.effect_tree_with_state();
        if tree.OnFilterAnimated(id, filters, state) {
            self.set_needs_update_draw_properties();
            self.RequestCommitForPropertyMutationIfNeeded(PropertyMutation::kFilter, context);
        }
    }
    pub fn SetBackdropFilterMutated(
        &mut self,
        id: ElementId,
        filters: &B::Filters,
        context: &mut LayerTreeContext<'_, B>,
    ) {
        self.ValidateEffectTreeMapping(id, PropertyMutation::kBackdropFilter, context);
        let (tree, state) = self.property_trees_.effect_tree_with_state();
        if tree.OnBackdropFilterAnimated(id, filters, state) {
            self.set_needs_update_draw_properties();
            self.RequestCommitForPropertyMutationIfNeeded(
                PropertyMutation::kBackdropFilter,
                context,
            );
        }
    }
    // Source: cc/trees/layer_tree_impl.cc:1277-1302.
    pub fn AddPresentationCallbacks(&mut self, mut callbacks: Vec<B::PresentationCallback>) {
        self.presentation_callbacks_.append(&mut callbacks);
    }
    pub fn TakePresentationCallbacks(&mut self) -> Vec<B::PresentationCallback> {
        std::mem::take(&mut self.presentation_callbacks_)
    }
    pub fn AddSuccessfulPresentationCallbacks(
        &mut self,
        mut callbacks: Vec<B::SuccessfulPresentationCallback>,
    ) {
        self.successful_presentation_callbacks_
            .append(&mut callbacks);
    }
    pub fn TakeSuccessfulPresentationCallbacks(
        &mut self,
    ) -> Vec<B::SuccessfulPresentationCallback> {
        std::mem::take(&mut self.successful_presentation_callbacks_)
    }
    // Source: cc/trees/layer_tree_impl.cc:1304-1314.
    pub fn InnerViewportScrollLayerForTesting(&self) -> Option<&B::Layer> {
        self.InnerViewportScrollNode()
            .and_then(|node| self.LayerByElementId(node.element_id))
    }
    pub fn OuterViewportScrollLayerForTesting(&self) -> Option<&B::Layer> {
        self.OuterViewportScrollNode()
            .and_then(|node| self.LayerByElementId(node.element_id))
    }
    // Source: cc/trees/layer_tree_impl.cc:1316-1340.
    pub fn CurrentlyScrollingNode(&self) -> Option<&ScrollNode> {
        self.property_trees_.scroll_tree().CurrentlyScrollingNode()
    }
    pub fn CurrentlyScrollingNodeMutable(
        &mut self,
        context: &LayerTreeContext<'_, B>,
    ) -> Option<&mut ScrollNode> {
        debug_assert!(self.IsActiveTree(context));
        self.property_trees_
            .scroll_tree_mutable()
            .CurrentlyScrollingNodeMutable()
    }
    pub fn SetCurrentlyScrollingNodeById(
        &mut self,
        node: Option<i32>,
        _context: &LayerTreeContext<'_, B>,
    ) {
        let tree = self.property_trees_.scroll_tree_mutable();
        let old = tree
            .CurrentlyScrollingNode()
            .map_or(ElementId::default(), |node| node.element_id);
        let new = node.map_or(ElementId::default(), |id| tree.Node(id).element_id);
        if old == new {
            return;
        }
        tree.set_currently_scrolling_node(node.unwrap_or(kInvalidPropertyNodeId));
    }
    pub fn ClearCurrentlyScrollingNode(&mut self, context: &LayerTreeContext<'_, B>) {
        self.SetCurrentlyScrollingNodeById(None, context);
    }
    // Source: cc/trees/layer_tree_impl.cc:1342-1354.
    fn ClampPageScaleFactorToLimits(&self, scale: f32) -> f32 {
        if !self.page_scale_factor_limits_set_ {
            return scale;
        }
        if scale < self.min_page_scale_factor_ {
            return self.min_page_scale_factor_;
        }
        if scale > self.max_page_scale_factor_ {
            return self.max_page_scale_factor_;
        }
        scale
    }
}

impl<B: LayerTreeTypes> LayerTreeImpl<B> {
    // Source: cc/trees/layer_tree_impl.cc:1356-1376.
    pub fn UpdatePropertyTreeAnimationFromMainThread(
        &mut self,
        context: &mut LayerTreeContext<'_, B>,
    ) {
        if self.layer_list_.is_empty() {
            return;
        }
        debug_assert!(self.IsSyncTree(context));
        // Copy only the source loop's current {ElementId,index} pair. The map remains
        // resident, and each subsequent entry is looked up after the preceding call.
        let mut previous = None;
        loop {
            let entry = match previous {
                None => self
                    .property_trees_
                    .transform_tree()
                    .element_id_to_node_index()
                    .iter()
                    .next()
                    .map(|(id, index)| (*id, *index)),
                Some(previous) => self
                    .property_trees_
                    .transform_tree()
                    .element_id_to_node_index()
                    .range((
                        std::ops::Bound::Excluded(previous),
                        std::ops::Bound::Unbounded,
                    ))
                    .next()
                    .map(|(id, index)| (*id, *index)),
            };
            let Some((id, index)) = entry else {
                break;
            };
            self.UpdateTransformAnimation(id, index, context);
            previous = Some(id);
        }
    }
    // Source: cc/trees/layer_tree_impl.cc:1378-1422.
    fn UpdateTransformAnimation(
        &mut self,
        id: ElementId,
        index: i32,
        context: &mut LayerTreeContext<'_, B>,
    ) {
        use super::target_property::Type::{ROTATE, SCALE, TRANSFORM, TRANSLATE};
        #[cfg(debug_assertions)]
        let mut property_count = 0u32;
        for property in [TRANSFORM, SCALE, ROTATE, TRANSLATE] {
            if context
                .host
                .mutator_host()
                .HasAnyAnimationTargetingProperty(id, property)
            {
                #[cfg(debug_assertions)]
                {
                    property_count += 1;
                }
                if index != kInvalidPropertyNodeId {
                    let list = self.GetElementTypeForAnimation(context);
                    let potential = context
                        .host
                        .mutator_host()
                        .HasPotentiallyRunningAnimationForProperty(id, list, property);
                    let node = self
                        .property_trees_
                        .transform_tree_mutable()
                        .MutableNode(index);
                    if node.has_potential_animation != potential {
                        node.has_potential_animation = potential;
                        node.maximum_animation_scale =
                            context.host.mutator_host().MaximumScale(id, list);
                        let (tree, mut mutation) =
                            self.property_trees_.transform_tree_with_context();
                        tree.set_needs_update(true, &mut mutation);
                        self.set_needs_update_draw_properties();
                    }
                }
            }
        }
        #[cfg(debug_assertions)]
        debug_assert!(property_count <= 1);
    }
    // Source: cc/trees/layer_tree_impl.cc:1424-1431.
    fn UpdatePageScaleNode(&mut self, context: &LayerTreeContext<'_, B>) {
        self.UpdatePageScaleNodeWithHost(&*context.host)
    }
    // Borrow-only capability entry; the source body is shared with the context entry.
    // Source: cc/trees/layer_tree_impl.cc:1424-1431.
    pub(crate) fn UpdatePageScaleNodeWithHost<H: mutation::LayerTreeScaleHost<B> + ?Sized>(
        &mut self,
        host: &H,
    ) {
        if self.PageScaleTransformNode().is_none() {
            debug_assert!(
                self.layer_list_.is_empty()
                    || self.current_page_scale_factor_for_tree(host.IsActiveTree(self.id)) == 1.0
            );
            return;
        }
        let factor = self.current_page_scale_factor_for_tree(host.IsActiveTree(self.id));
        crate::cc::trees::draw_property_utils::UpdatePageScaleFactor::<B::DrawContext<'_>>(
            &mut self.property_trees_,
            Some(self.viewport_property_ids_.page_scale_transform),
            factor,
        );
    }
    // Source: cc/trees/layer_tree_impl.cc:1433-1450.
    pub fn SetPageScaleOnActiveTree(&mut self, scale: f32, context: &mut LayerTreeContext<'_, B>) {
        self.SetPageScaleOnActiveTreeWithHost(scale, context.host)
    }
    // Borrow-only capability entry; the source body is shared with the context entry.
    // Source: cc/trees/layer_tree_impl.cc:1433-1450.
    pub fn SetPageScaleOnActiveTreeWithHost<H: mutation::LayerTreeScaleHost<B> + ?Sized>(
        &mut self,
        scale: f32,
        host: &mut H,
    ) {
        debug_assert!(host.IsActiveTree(self.id));
        debug_assert!(self.lifecycle_.AllowsPropertyTreeAccess());
        let clamped = self.ClampPageScaleFactorToLimits(scale);
        if !host.settings().is_for_scalable_page() && clamped != 1.0 {
            host.ReportUnexpectedOopifPageScaleOnce(clamped);
        }
        let changed = self.page_scale_factor_.borrow_mut().SetCurrent(clamped);
        if changed {
            self.DidUpdatePageScaleWithHost(host);
        }
    }
    // Source: cc/trees/layer_tree_impl.cc:1452-1458.
    pub fn PushPageScaleFromMainThread(
        &mut self,
        scale: f32,
        min: f32,
        max: f32,
        limits_set: bool,
        context: &mut LayerTreeContext<'_, B>,
    ) {
        self.PushPageScaleFactorAndLimits(Some(scale), min, max, limits_set, context);
    }
    // Source: cc/trees/layer_tree_impl.cc:1460-1472.
    pub fn SetPageScaleFactorAndLimitsForDisplayTree(
        &mut self,
        scale: f32,
        min: f32,
        max: f32,
        context: &mut LayerTreeContext<'_, B>,
    ) {
        debug_assert!(context.host.settings().trees_in_viz_in_viz_process());
        let mut changed = self.page_scale_factor_.borrow_mut().SetCurrent(scale);
        changed |= self.SetPageScaleFactorLimits(min, max, true);
        if changed {
            self.DidUpdatePageScale(context);
        }
    }
    // Source: cc/trees/layer_tree_impl.cc:1474-1496.
    fn PushPageScaleFactorAndLimits(
        &mut self,
        scale: Option<f32>,
        min: f32,
        max: f32,
        limits_set: bool,
        context: &mut LayerTreeContext<'_, B>,
    ) {
        debug_assert!(scale.is_some() || self.IsActiveTree(context));
        let mut changed = false;
        changed |= self.SetPageScaleFactorLimits(min, max, limits_set);
        if let Some(scale) = scale {
            debug_assert!(!self.IsActiveTree(context) || !context.host.HasPendingTree());
            changed |= self
                .page_scale_factor_
                .borrow_mut()
                .PushMainToPending(scale);
        }
        if self.IsActiveTree(context) {
            changed |= self.page_scale_factor_.borrow_mut().PushPendingToActive();
        }
        if changed {
            self.DidUpdatePageScale(context);
        }
    }
    // Source: cc/trees/layer_tree_impl.cc:1498-1519.
    pub fn SetBrowserControlsParams(
        &mut self,
        params: &BrowserControlsParams,
        context: &mut LayerTreeContext<'_, B>,
    ) {
        if self.browser_controls_params_ == *params {
            return;
        }
        self.browser_controls_params_ = *params;
        self.UpdateViewportContainerSizes(context);
        if self.IsActiveTree(context) {
            let can_animate = !context.host.settings().trees_in_viz_in_viz_process()
                || context.host.TreeAnimationsInVizInVizProcess();
            context.host.OnBrowserControlsParamsChanged(
                self,
                params.animate_browser_controls_height_changes && can_animate,
            );
        }
    }
    // Source: cc/trees/layer_tree_impl.cc:1521-1524.
    pub fn set_overscroll_behavior(&mut self, behavior: &OverscrollBehavior) {
        self.overscroll_behavior_ = *behavior;
    }
    // Source: cc/trees/layer_tree_impl.cc:1526-1537.
    fn ClampTopControlsShownRatio(&mut self, context: &LayerTreeContext<'_, B>) -> bool {
        let id = self.id;
        self.ClampTopControlsShownRatioWithRange(|| {
            if context.host.IsActiveTree(id) {
                context.host.TopControlsShownRatioRange()
            } else {
                (0., 1.)
            }
        })
    }
    // Source: cc/trees/layer_tree_impl.cc:1526-1537.
    fn ClampTopControlsShownRatioWithRange(
        &mut self,
        get_range: impl FnOnce() -> (f32, f32),
    ) -> bool {
        let ratio = self.top_controls_shown_ratio_.borrow().Current(true);
        let range = get_range();
        self.top_controls_shown_ratio_
            .borrow_mut()
            .SetCurrent(ratio.clamp(range.0, range.1))
    }
    // Source: cc/trees/layer_tree_impl.cc:1539-1550.
    fn ClampBottomControlsShownRatio(&mut self, context: &LayerTreeContext<'_, B>) -> bool {
        let id = self.id;
        self.ClampBottomControlsShownRatioWithRange(|| {
            if context.host.IsActiveTree(id) {
                context.host.BottomControlsShownRatioRange()
            } else {
                (0., 1.)
            }
        })
    }
    // Source: cc/trees/layer_tree_impl.cc:1539-1550.
    fn ClampBottomControlsShownRatioWithRange(
        &mut self,
        get_range: impl FnOnce() -> (f32, f32),
    ) -> bool {
        let ratio = self.bottom_controls_shown_ratio_.borrow().Current(true);
        let range = get_range();
        self.bottom_controls_shown_ratio_
            .borrow_mut()
            .SetCurrent(ratio.clamp(range.0, range.1))
    }
    // Source: cc/trees/layer_tree_impl.cc:1552-1561.
    pub fn SetCurrentBrowserControlsShownRatio(
        &mut self,
        top: f32,
        bottom: f32,
        context: &LayerTreeContext<'_, B>,
    ) -> bool {
        let active = self.IsActiveTree(context);
        self.SetCurrentBrowserControlsShownRatioWithRanges(
            top,
            bottom,
            || {
                if active {
                    context.host.TopControlsShownRatioRange()
                } else {
                    (0., 1.)
                }
            },
            || {
                if active {
                    context.host.BottomControlsShownRatioRange()
                } else {
                    (0., 1.)
                }
            },
        )
    }
    // Source: cc/trees/layer_tree_impl.cc:1526-1561.
    // Rust borrow split: manager ranges are queried at their source sequence
    // points after each SetCurrent, without retaining/copying manager state.
    pub(crate) fn SetCurrentBrowserControlsShownRatioWithRanges(
        &mut self,
        top: f32,
        bottom: f32,
        top_range: impl FnOnce() -> (f32, f32),
        bottom_range: impl FnOnce() -> (f32, f32),
    ) -> bool {
        let mut changed = self.top_controls_shown_ratio_.borrow_mut().SetCurrent(top);
        changed |= self.ClampTopControlsShownRatioWithRange(top_range);
        changed |= self
            .bottom_controls_shown_ratio_
            .borrow_mut()
            .SetCurrent(bottom);
        changed |= self.ClampBottomControlsShownRatioWithRange(bottom_range);
        changed
    }
    // Source: cc/trees/layer_tree_impl.cc:1563-1567.
    pub fn PushBrowserControlsFromMainThread(
        &mut self,
        top: f32,
        bottom: f32,
        context: &mut LayerTreeContext<'_, B>,
    ) {
        self.PushBrowserControls(Some(top), Some(bottom), context);
    }
    // Source: cc/trees/layer_tree_impl.cc:1569-1593.
    fn PushBrowserControls(
        &mut self,
        top: Option<f32>,
        bottom: Option<f32>,
        context: &mut LayerTreeContext<'_, B>,
    ) {
        debug_assert!(top.is_some() || bottom.is_some() || self.IsActiveTree(context));
        debug_assert!(top.is_none() || bottom.is_some());
        debug_assert!(top.is_some() || self.IsActiveTree(context));
        if let Some(top) = top {
            debug_assert!(!self.IsActiveTree(context) || !context.host.HasPendingTree());
            self.top_controls_shown_ratio_
                .borrow_mut()
                .PushMainToPending(top);
            self.bottom_controls_shown_ratio_
                .borrow_mut()
                .PushMainToPending(bottom.unwrap());
            if !self.IsActiveTree(context) {
                self.UpdateViewportContainerSizes(context);
            }
        }
        if self.IsActiveTree(context) {
            let mut changed = self
                .top_controls_shown_ratio_
                .borrow_mut()
                .PushPendingToActive();
            changed |= self.ClampTopControlsShownRatio(context);
            changed |= self
                .bottom_controls_shown_ratio_
                .borrow_mut()
                .PushPendingToActive();
            changed |= self.ClampBottomControlsShownRatio(context);
            if changed {
                context.host.DidChangeBrowserControlsPosition(self);
            }
        }
    }
    // Source: cc/trees/layer_tree_impl.cc:1595-1612.
    fn SetPageScaleFactorLimits(&mut self, min: f32, max: f32, limits_set: bool) -> bool {
        debug_assert!(min > 0.0);
        debug_assert!(max >= min);
        if self.page_scale_factor_limits_set_ == limits_set
            && min == self.min_page_scale_factor_
            && max == self.max_page_scale_factor_
        {
            return false;
        }
        self.page_scale_factor_limits_set_ = limits_set;
        self.min_page_scale_factor_ = min;
        self.max_page_scale_factor_ = max;
        true
    }
    // Source: cc/trees/layer_tree_impl.cc:1614-1643.
    fn DidUpdatePageScale(&mut self, context: &mut LayerTreeContext<'_, B>) {
        self.DidUpdatePageScaleWithHost(context.host)
    }
    // Borrow-only capability entry; the source body is shared with the context entry.
    // Source: cc/trees/layer_tree_impl.cc:1614-1643.
    pub(crate) fn DidUpdatePageScaleWithHost<H: mutation::LayerTreeScaleHost<B> + ?Sized>(
        &mut self,
        host: &mut H,
    ) {
        if host.IsActiveTree(self.id) {
            let clamped = self.ClampPageScaleFactorToLimits(
                self.current_page_scale_factor_for_tree(host.IsActiveTree(self.id)),
            );
            self.page_scale_factor_.borrow_mut().SetCurrent(clamped);
            if host.HasPendingTree() {
                host.PendingTreeDidUpdatePageScale();
            }
            if host.HasRecycleTree() {
                host.RecycleTreeDidUpdatePageScale();
            }
            if host.settings().scrollbar_flash_once_after_scroll_update()
                || host.settings().scrollbar_flash_after_any_scroll_update()
            {
                host.OnPageScaleUpdated(self);
            } else if let Some(id) = host
                .OuterViewportScrollNode(self)
                .map(|node| node.element_id)
            {
                host.WithScrollbarAnimationController(self, id, &mut |controller, client| {
                    if let Some(controller) = controller {
                        controller.DidScrollUpdate(client);
                    }
                });
            }
        }
        debug_assert!(self.lifecycle_.AllowsPropertyTreeAccess());
        self.UpdatePageScaleNodeWithHost(host);
        self.set_needs_update_draw_properties();
        self.UpdateViewportScrollbarGeometriesWithHost(host);
    }
    // Source: cc/trees/layer_tree_impl.cc:1645-1653.
    pub fn SetDeviceScaleFactor(&mut self, factor: f32, context: &mut LayerTreeContext<'_, B>) {
        if factor == self.device_scale_factor_ {
            return;
        }
        self.device_scale_factor_ = factor;
        self.set_needs_update_draw_properties();
        if self.IsActiveTree(context) {
            context
                .host
                .SetViewportDamage(self.GetDeviceViewport(context));
        }
    }
    // Source: cc/trees/layer_tree_impl.cc:1655-1679.
    pub fn SetLocalSurfaceIdFromParent(&mut self, id: &B::LocalSurfaceId) {
        self.local_surface_id_from_parent_ = id.clone();
    }
    pub fn RequestNewLocalSurfaceId(&mut self) {
        self.new_local_surface_id_request_ = true;
    }
    pub fn TakeNewLocalSurfaceIdRequest(&mut self) -> bool {
        let request = self.new_local_surface_id_request_;
        self.new_local_surface_id_request_ = false;
        request
    }
    pub fn SetScreenshotDestinationToken(&mut self, token: UnguessableToken) {
        self.screenshot_destination_ = token;
    }
    pub fn TakeScreenshotDestinationToken(&mut self) -> UnguessableToken {
        std::mem::take(&mut self.screenshot_destination_)
    }
    // Source: cc/trees/layer_tree_impl.cc:1681-1695.
    pub fn SetDeviceViewportRect(&mut self, rect: &Rect, context: &mut LayerTreeContext<'_, B>) {
        if *rect == self.device_viewport_rect_ {
            return;
        }
        self.device_viewport_rect_ = *rect;
        self.device_viewport_rect_changed_ = true;
        self.set_needs_update_draw_properties();
        if !self.IsActiveTree(context) {
            return;
        }
        self.UpdateViewportContainerSizes(context);
        context.host.OnCanDrawStateChangedForTree(self);
        context
            .host
            .SetViewportDamage(self.GetDeviceViewport(context));
    }
    // Source: cc/trees/layer_tree_impl.cc:1697-1725.
    pub fn GetDeviceViewport(&self, context: &LayerTreeContext<'_, B>) -> Rect {
        self.GetDeviceViewportWithExternal(context.host.external_viewport())
    }
    // Source: cc/trees/layer_tree_impl.cc:1697-1702.
    // Rust borrow split: only the current external rect is required here.
    pub(crate) fn GetDeviceViewportWithExternal(&self, external: Rect) -> Rect {
        Self::GetDeviceViewportForRects(self.device_viewport_rect_, external)
    }
    pub fn SetMaxSafeAreaInsetBottom(&mut self, value: f32) {
        self.max_safe_area_inset_bottom_ = value;
    }
    pub fn SetDisplayColorSpaces(&mut self, value: &B::DisplayColorSpaces) {
        if self.display_color_spaces_ == *value {
            return;
        }
        self.display_color_spaces_ = value.clone();
    }
    pub fn SetExternalPageScaleFactor(
        &mut self,
        factor: f32,
        context: &mut LayerTreeContext<'_, B>,
    ) {
        if self.external_page_scale_factor_ == factor {
            return;
        }
        self.external_page_scale_factor_ = factor;
        self.DidUpdatePageScale(context);
    }
    // Source: cc/trees/layer_tree_impl.cc:1727-1733.
    pub fn page_scale_factor(&self) -> Ref<'_, SyncedScale> {
        self.page_scale_factor_.borrow()
    }
    pub fn page_scale_factor_mut(&mut self) -> RefMut<'_, SyncedScale> {
        self.page_scale_factor_.borrow_mut()
    }
    // Source: cc/trees/layer_tree_impl.cc:1735-1743.
    pub fn ScrollableViewportSize(&self, context: &LayerTreeContext<'_, B>) -> SizeF {
        if self.InnerViewportScrollNode().is_none() {
            return SizeF::default();
        }
        let (scroll, _, state) = self.property_trees_.scroll_tree_read_context();
        let bounds = scroll.container_bounds(self.viewport_property_ids_.inner_scroll, state);
        let scale = self.page_scale_factor_for_scroll(context);
        SizeF::new(
            bounds.width() as f32 / scale,
            bounds.height() as f32 / scale,
        )
    }
    // Source: cc/trees/layer_tree_impl.cc:1745-1755.
    pub fn RootScrollLayerDeviceViewportBounds(&self) -> Rect {
        let Some(node) = self.OuterViewportScrollNode() else {
            debug_assert!(self.InnerViewportScrollNode().is_none());
            return Rect::default();
        };
        crate::cc::base::math_util::MathUtil::MapEnclosingClippedRect(
            self.property_trees_
                .transform_tree()
                .ToScreen(node.transform_id),
            Rect::new(0, 0, node.bounds.width(), node.bounds.height()),
        )
    }
    // Source: cc/trees/layer_tree_impl.cc:1757-1774.
    pub fn ApplySentScrollAndScaleDeltasFromAbortedCommit(
        &mut self,
        next_bmf: bool,
        applied: bool,
        context: &LayerTreeContext<'_, B>,
    ) {
        debug_assert!(self.IsActiveTree(context));
        self.page_scale_factor_
            .borrow_mut()
            .AbortCommit(next_bmf, applied);
        self.top_controls_shown_ratio_
            .borrow_mut()
            .AbortCommit(next_bmf, applied);
        self.bottom_controls_shown_ratio_
            .borrow_mut()
            .AbortCommit(next_bmf, applied);
        if self.layer_list_.is_empty() {
            return;
        }
        let (scroll, _, state) = self.property_trees_.scroll_tree_read_context();
        scroll.ApplySentScrollDeltasFromAbortedCommit(next_bmf, applied, state);
    }
    // Source: cc/trees/layer_tree_impl.cc:1776-1787.
    pub fn SetViewportPropertyIds(&mut self, ids: &ViewportPropertyIds) {
        self.viewport_property_ids_ = *ids;
        debug_assert!(
            ids.inner_scroll != kInvalidPropertyNodeId
                || (ids.outer_scroll == kInvalidPropertyNodeId
                    && ids.outer_clip == kInvalidPropertyNodeId)
        );
        if let Some(element) = self.InnerViewportScrollNode().map(|node| node.element_id) {
            if let Some(layer) = self
                .layer_list_
                .iter_mut()
                .rev()
                .find(|layer| layer.element_id() == element)
            {
                layer.set_is_inner_viewport_scroll_layer();
            }
        }
    }
    // Source: cc/trees/layer_tree_impl.cc:1789-1832; cc/trees/layer_tree_impl.h:343-378.
    pub fn OverscrollElasticityTransformNode(&self) -> Option<&TransformNode<B::Transform>> {
        let id = self.viewport_property_ids_.overscroll_elasticity_transform;
        if id == kInvalidPropertyNodeId {
            None
        } else {
            Some(self.property_trees_.transform_tree().Node(id))
        }
    }
    pub fn PageScaleTransformNode(&self) -> Option<&TransformNode<B::Transform>> {
        let id = self.viewport_property_ids_.page_scale_transform;
        if id == kInvalidPropertyNodeId {
            None
        } else {
            Some(self.property_trees_.transform_tree().Node(id))
        }
    }
    pub fn InnerViewportScrollNode(&self) -> Option<&ScrollNode> {
        let id = self.viewport_property_ids_.inner_scroll;
        if id == kInvalidPropertyNodeId {
            None
        } else {
            Some(self.property_trees_.scroll_tree().Node(id))
        }
    }
    pub fn OuterViewportClipNode(&self) -> Option<&ClipNode> {
        let id = self.viewport_property_ids_.outer_clip;
        if id == kInvalidPropertyNodeId {
            None
        } else {
            Some(self.property_trees_.clip_tree().Node(id))
        }
    }
    pub fn OuterViewportScrollNode(&self) -> Option<&ScrollNode> {
        let id = self.viewport_property_ids_.outer_scroll;
        if id == kInvalidPropertyNodeId {
            None
        } else {
            Some(self.property_trees_.scroll_tree().Node(id))
        }
    }
}

impl<B: LayerTreeTypes> LayerTreeImpl<B> {
    // Source: cc/trees/layer_tree_impl.cc:1835-1840.
    pub fn SetElementIdsForTesting(&mut self, context: &mut LayerTreeContext<'_, B>) {
        self.ForEachLayerWithContext(false, context, |layer, layer_context| {
            if layer.element_id() == ElementId::default() {
                layer.SetElementId(
                    ElementId::new((i32::MAX - layer.id()) as u64),
                    layer_context,
                );
            }
        });
    }
    // Source: cc/trees/layer_tree_impl.cc:1842-1958.
    pub fn UpdateDrawProperties(
        &mut self,
        update_tiles: bool,
        update_images: bool,
        output_updates: Option<&mut Vec<i32>>,
        context: &mut LayerTreeContext<'_, B>,
    ) -> bool {
        self.UpdateDrawPropertiesWithHost(
            update_tiles,
            update_images,
            output_updates,
            &mut draw_host::LayerTreeDrawHostRef(context.host),
        )
    }
    // Borrow-only entry; the canonical source body is shared with the full host.
    pub fn UpdateDrawPropertiesWithHost(
        &mut self,
        update_tiles: bool,
        update_images: bool,
        output_updates: Option<&mut Vec<i32>>,
        host: &mut dyn draw_host::LayerTreeDrawHost<B>,
    ) -> bool {
        compositor_draw::tree_draw_update::UpdateTreeDrawProperties(
            &mut DrawTreeUpdateAdapter {
                tree: self,
                host,
                output_updates,
            },
            update_tiles,
            update_images,
        )
        .success
    }
    // Source: cc/trees/layer_tree_impl.cc:1960-1985.
    pub fn UpdateTiles(&mut self, context: &mut LayerTreeContext<'_, B>) -> bool {
        self.UpdateTilesWithHost(&mut draw_host::LayerTreeDrawHostRef(context.host))
    }
    // Same source body; the host factory only splits resident fields/services.
    pub fn UpdateTilesWithHost(&mut self, host: &mut dyn draw_host::LayerTreeDrawHost<B>) -> bool {
        compositor_tile::picture_layer_update::UpdatePictureLayerTiles(
            &mut TilePictureLayerUpdateAdapter { tree: self, host },
        )
        .changed
    }
    // Source: cc/trees/layer_tree_impl.cc:1987-2006.
    pub fn GetRenderSurfaceList(&self) -> &Vec<i32> {
        debug_assert!(!self.needs_update_draw_properties_);
        &self.render_surface_list_
    }
    pub fn GetRenderSurface(&self, id: i32) -> Option<&B::RenderSurface> {
        self.property_trees_.effect_tree().GetRenderSurface(id)
    }
    pub fn GetRenderSurfaceMutable(&mut self, id: i32) -> Option<&mut B::RenderSurface> {
        self.property_trees_
            .effect_tree_mutable()
            .GetRenderSurfaceMutable(id)
    }
    pub fn UnoccludedScreenSpaceRegion(&self) -> &Region<B::RegionBackend> {
        debug_assert!(!self.needs_update_draw_properties_);
        &self.unoccluded_screen_space_region_
    }
    // Source: cc/trees/layer_tree_impl.cc:2008-2018.
    pub fn ScrollableSize(&self) -> SizeF {
        let Some(node) = self.OuterViewportScrollNode() else {
            debug_assert!(self.InnerViewportScrollNode().is_none());
            return SizeF::default();
        };
        let (scroll, _, state) = self.property_trees_.scroll_tree_read_context();
        let size = scroll.scroll_bounds(node.id, state);
        let bounds = scroll.container_bounds(node.id, state);
        SizeF::new(
            size.width().max(bounds.width() as f32),
            size.height().max(bounds.height() as f32),
        )
    }
    // Source: cc/trees/layer_tree_impl.cc:2020-2033.
    pub fn LayerById(&self, id: i32) -> Option<&B::Layer> {
        self.layer_list_.LayerById(id)
    }
    pub fn LayerByElementId(&self, id: ElementId) -> Option<&B::Layer> {
        self.layer_list_
            .iter()
            .rev()
            .find(|layer| layer.element_id() == id)
    }
    // Source: cc/trees/layer_tree_impl.cc:2035-2071.
    pub fn SetSurfaceRanges(&mut self, ranges: BTreeSet<B::SurfaceRange>) {
        debug_assert!(self.surface_layer_ranges_.is_empty());
        self.surface_layer_ranges_ = ranges;
        self.needs_surface_ranges_sync_ = true;
    }
    pub fn SurfaceRanges(&self) -> &BTreeSet<B::SurfaceRange> {
        &self.surface_layer_ranges_
    }
    pub fn ClearSurfaceRanges(&mut self) {
        self.surface_layer_ranges_.clear();
        self.needs_surface_ranges_sync_ = true;
    }
    pub fn AddLayerShouldPushProperties(&mut self, layer: &B::Layer) {
        self.layer_list_.AddLayerShouldPushProperties(layer.id());
    }
    pub fn ClearLayersThatShouldPushProperties(&mut self) {
        self.layer_list_.ClearLayersThatShouldPushProperties();
    }
    pub fn ReserveLayers(&mut self, count: usize) {
        self.layer_list_.Reserve(count);
    }
    pub fn AddLayer(&mut self, layer: Box<B::Layer>) {
        self.layer_list_.AddLayer(layer);
        self.set_needs_update_draw_properties();
    }
    pub fn NumLayers(&self) -> usize {
        self.layer_list_.len()
    }
}

impl<B: LayerTreeTypes> LayerTreeImpl<B> {
    // Source: cc/trees/layer_tree_impl.cc:2073-2092.
    pub fn DidBecomeActive(&mut self, context: &mut LayerTreeContext<'_, B>) {
        if self.next_activation_forces_redraw_ {
            context
                .host
                .SetViewportDamage(self.GetDeviceViewport(context));
            self.next_activation_forces_redraw_ = false;
        }
        context.host.ResetRequiresHighResToDraw();
        self.ForEachLayerWithContext(false, context, |layer, layer_context| {
            layer.DidBecomeActive(layer_context)
        });
        for promise in &mut self.swap_promise_list_ {
            promise
                .as_mut()
                .expect("non-null swap promise")
                .DidActivate();
        }
        if !context.host.settings().is_layer_tree_for_ui() {
            context
                .host
                .TraceDidActivateLayerTree(self.source_frame_number_);
        }
    }
    // Source: cc/trees/layer_tree_impl.cc:2094-2190; cc/trees/layer_tree_impl.h:137-147.
    pub fn RequiresHighResToDraw(&self, c: &LayerTreeContext<'_, B>) -> bool {
        c.host.RequiresHighResToDraw()
    }
    pub fn task_runner_provider<'a>(
        &self,
        c: &'a LayerTreeContext<'_, B>,
    ) -> &'a B::TaskRunnerProvider {
        c.host.task_runner_provider()
    }
    pub fn WithLayerTreeFrameSink(
        &self,
        c: &LayerTreeContext<'_, B>,
        callback: &mut dyn FnMut(Option<&B::LayerTreeFrameSink>),
    ) {
        c.host.WithLayerTreeFrameSink(callback)
    }
    pub fn max_texture_size(&self, c: &LayerTreeContext<'_, B>) -> i32 {
        c.host.max_texture_size()
    }
    pub fn settings<'a>(&self, c: &'a LayerTreeContext<'_, B>) -> &'a B::Settings {
        c.host.settings()
    }
    pub fn debug_state<'a>(&self, c: &'a LayerTreeContext<'_, B>) -> &'a B::DebugState {
        c.host.debug_state()
    }
    pub fn context_provider<'a>(
        &self,
        c: &'a LayerTreeContext<'_, B>,
    ) -> Option<&'a B::RasterContextProvider> {
        c.host.context_provider()
    }
    pub fn resource_provider<'a>(
        &self,
        c: &'a LayerTreeContext<'_, B>,
    ) -> Option<&'a B::ClientResourceProvider> {
        c.host.resource_provider()
    }
    pub fn tile_manager<'a>(&self, c: &'a LayerTreeContext<'_, B>) -> &'a B::TileManager {
        c.host.tile_manager()
    }
    pub fn image_decode_cache<'a>(
        &self,
        c: &'a LayerTreeContext<'_, B>,
    ) -> Option<&'a B::ImageDecodeCache> {
        c.host.GetImageDecodeCache()
    }
    pub fn image_animation_controller<'a>(
        &self,
        c: &'a LayerTreeContext<'_, B>,
    ) -> Option<&'a B::ImageAnimationController> {
        c.host.image_animation_controller()
    }
    pub fn frame_sorter<'a>(&self, c: &'a LayerTreeContext<'_, B>) -> &'a B::FrameSorter {
        c.host.frame_sorter()
    }
    pub fn memory_history<'a>(&self, c: &'a LayerTreeContext<'_, B>) -> &'a B::MemoryHistory {
        c.host.memory_history()
    }
    pub fn debug_rect_history<'a>(
        &self,
        c: &'a LayerTreeContext<'_, B>,
    ) -> &'a B::DebugRectHistory {
        c.host.debug_rect_history()
    }
    pub fn IsActiveTree(&self, c: &LayerTreeContext<'_, B>) -> bool {
        debug_assert_eq!(self.host_impl_, c.host.identity());
        c.host.IsActiveTree(self.id)
    }
    pub fn IsPendingTree(&self, c: &LayerTreeContext<'_, B>) -> bool {
        c.host.IsPendingTree(self.id)
    }
    pub fn IsRecycleTree(&self, c: &LayerTreeContext<'_, B>) -> bool {
        c.host.IsRecycleTree(self.id)
    }
    pub fn IsSyncTree(&self, c: &LayerTreeContext<'_, B>) -> bool {
        c.host.IsSyncTree(self.id)
    }
    pub fn HasPendingTree(&self, c: &LayerTreeContext<'_, B>) -> bool {
        c.host.HasPendingTree()
    }
    pub fn FindActiveTreeLayerById<'a>(
        &'a self,
        id: i32,
        c: &'a LayerTreeContext<'_, B>,
    ) -> Option<&'a B::Layer> {
        c.host.ActiveTreeLayerById(self, id)
    }
    pub fn FindPendingTreeLayerById<'a>(
        &'a self,
        id: i32,
        c: &'a LayerTreeContext<'_, B>,
    ) -> Option<&'a B::Layer> {
        c.host.PendingTreeLayerById(self, id)
    }
    pub fn PinchGestureActive(&self, c: &LayerTreeContext<'_, B>) -> bool {
        c.host.IsPinchGestureActive()
    }
    pub fn CurrentBeginFrameArgs<'a>(&self, c: &'a LayerTreeContext<'_, B>) -> &'a BeginFrameArgs {
        c.host.CurrentBeginFrameArgs()
    }
    // Source: cc/trees/layer_tree_impl.cc:2192-2210.
    pub fn CreatedBeginFrameArgs(&self) -> &BeginFrameArgs {
        &self.created_begin_frame_args_
    }
    pub fn SetCreatedBeginFrameArgs(&mut self, args: BeginFrameArgs) {
        self.created_begin_frame_args_ = args;
    }
    pub fn CurrentBeginFrameInterval(
        &self,
        c: &LayerTreeContext<'_, B>,
    ) -> crate::adapter::time::TimeDelta {
        c.host.CurrentBeginFrameInterval()
    }
    pub fn ViewportRectForTilePriority(&self, c: &LayerTreeContext<'_, B>) -> Rect {
        let rect = c.host.viewport_rect_for_tile_priority();
        if rect.IsEmpty() {
            self.GetDeviceViewport(c)
        } else {
            rect
        }
    }
    // Source: cc/trees/layer_tree_impl.cc:2213-2237.
    pub fn CreateScrollbarAnimationController(
        &mut self,
        id: ElementId,
        opacity: f32,
        c: &mut LayerTreeContext<'_, B>,
    ) -> Option<Box<crate::cc::input::scrollbar_animation_controller::ScrollbarAnimationController>>
    {
        c.host.CreateScrollbarAnimationController(self, id, opacity)
    }
    // Source: cc/trees/layer_tree_impl.cc:2212-2237.
    // Rust borrow split: source creation reads only immutable settings and
    // borrows the current host client; it does not access tree-owned fields.
    pub(crate) fn CreateScrollbarAnimationControllerWithClient(
        settings: &B::Settings,
        id: ElementId,
        opacity: f32,
        client: &mut dyn crate::adapter::scrollbar::ScrollbarInitializationClient,
    ) -> Option<Box<crate::cc::input::scrollbar_animation_controller::ScrollbarAnimationController>>
    {
        use crate::cc::input::scrollbar_animation_controller::ScrollbarAnimationController as Controller;
        let delay = settings.scrollbar_fade_delay();
        let duration = settings.scrollbar_fade_duration();
        match settings.scrollbar_animator() {
            ScrollbarAnimator::ANDROID_OVERLAY => {
                Some(Controller::CreateScrollbarAnimationControllerAndroid(
                    id, client, delay, duration, opacity,
                ))
            }
            ScrollbarAnimator::AURA_OVERLAY => {
                let thinning = settings.scrollbar_thinning_duration();
                let idle = settings.idle_thickness_scale();
                Some(Controller::CreateScrollbarAnimationControllerAuraOverlay(
                    id, client, delay, duration, thinning, opacity, idle,
                ))
            }
            ScrollbarAnimator::NO_ANIMATOR => {
                unreachable!("NO_ANIMATOR cannot create a controller");
            }
        }
    }
    // Source: cc/trees/layer_tree_impl.cc:2239-2250.
    pub fn DidAnimateScrollOffset(&mut self, c: &mut LayerTreeContext<'_, B>) {
        c.host.DidAnimateScrollOffset();
    }
    pub fn raster_caps<'a>(
        &self,
        c: &'a LayerTreeContext<'_, B>,
    ) -> &'a super::raster_capabilities::RasterCapabilities {
        c.host.raster_caps()
    }
    pub fn SetNeedsRedraw(&mut self, c: &mut LayerTreeContext<'_, B>) {
        c.host.SetNeedsRedraw(self, false, false);
    }
    // Source: cc/trees/layer_tree_impl.cc:2252-2259.
    pub fn GetAllPrioritizedTilesForTracing(
        &mut self,
        out: &mut Vec<B::PrioritizedTile>,
        context: &mut LayerTreeContext<'_, B>,
    ) {
        self.ForEachLayerWithContext(true, context, |layer, layer_context| {
            if !layer.contributes_to_drawn_render_surface() {
                return;
            }
            layer.GetAllPrioritizedTilesForTracing(out, layer_context);
        });
    }
    // Source: cc/trees/layer_tree_impl.cc:2261-2291.
    pub fn AsValueInto(
        &mut self,
        value: &mut B::LayerTraceValue,
        context: &mut LayerTreeContext<'_, B>,
    ) {
        MakeDictIntoImplicitSnapshot(value, "cc::LayerTreeImpl", self.id.0 as usize);
        value.SetInteger("source_frame_number", self.source_frame_number_);
        value.BeginArray("render_surface_layer_list");
        for layer in self.layer_list_.iter().rev() {
            if layer.contributes_to_drawn_render_surface() {
                continue;
            }
            AppendIDRef(value, layer as *const B::Layer as usize);
        }
        value.EndArray();
        value.BeginArray("swap_promise_trace_ids");
        for promise in &self.swap_promise_list_ {
            value.AppendDouble(
                promise
                    .as_ref()
                    .expect("non-null swap promise")
                    .GetTraceId() as f64,
            );
        }
        value.EndArray();
        value.BeginArray("pinned_swap_promise_trace_ids");
        for promise in &self.pinned_swap_promise_list_ {
            value.AppendDouble(
                promise
                    .as_ref()
                    .expect("non-null swap promise")
                    .GetTraceId() as f64,
            );
        }
        value.EndArray();
        value.BeginArray("layers");
        self.ForEachLayerWithContext(false, context, |layer, layer_context| {
            value.BeginDictionaryUnnamed();
            layer.AsValueInto(value, layer_context);
            value.EndDictionary();
        });
        value.EndArray();
    }
    // Source: cc/trees/layer_tree_impl.cc:2293-2326.
    pub fn QueueSwapPromise(&mut self, promise: Box<B::SwapPromise>) {
        self.swap_promise_list_.push(Some(promise));
    }
    pub fn QueuePinnedSwapPromise(
        &mut self,
        promise: Box<B::SwapPromise>,
        c: &LayerTreeContext<'_, B>,
    ) {
        debug_assert!(self.IsActiveTree(c));
        self.QueuePinnedSwapPromiseFromActiveHost(promise);
    }
    /// Host-side half of the same source method. Callers possess the resident
    /// active-tree slot, so no synthetic LayerTreeContext or owner lookup is
    /// needed merely to append the move-only promise.
    pub(crate) fn QueuePinnedSwapPromiseFromActiveHost(&mut self, promise: Box<B::SwapPromise>) {
        self.pinned_swap_promise_list_.push(Some(promise));
    }
    pub fn PassSwapPromises(&mut self, new: Vec<Box<B::SwapPromise>>, c: &LayerTreeContext<'_, B>) {
        let timestamp = c.host.MonotonicTimeNow();
        PassSwapPromisesPolicy::<B::SwapPromise, B::CompositorFrameMetadata>(
            &mut self.swap_promise_list_,
            new,
            timestamp,
        );
    }
    pub fn AppendSwapPromises(&mut self, new: Vec<Box<B::SwapPromise>>) {
        self.swap_promise_list_.extend(new.into_iter().map(Some));
    }
    // Source: cc/trees/layer_tree_impl.cc:2328-2370.
    pub fn FinishSwapPromises(&mut self, metadata: &mut B::CompositorFrameMetadata) {
        FinishSwapPromisesPolicy(
            &mut self.swap_promise_list_,
            &mut self.pinned_swap_promise_list_,
            metadata,
        );
    }
    pub fn ClearSwapPromises(&mut self) {
        ClearSwapPromisesPolicy::<B::SwapPromise, B::CompositorFrameMetadata>(
            &mut self.swap_promise_list_,
            &mut self.pinned_swap_promise_list_,
        );
    }
    pub fn BreakSwapPromises(&mut self, reason: DidNotSwapReason, c: &LayerTreeContext<'_, B>) {
        self.BreakSwapPromisesAt(reason, c.host.MonotonicTimeNow());
    }
    // Source: cc/trees/layer_tree_impl.cc:2344-2370.
    // Single canonical body shared by ordinary live-context calls and owner
    // destruction, whose field split supplies the timestamp directly.
    fn BreakSwapPromisesAt(
        &mut self,
        reason: DidNotSwapReason,
        timestamp: crate::adapter::time::TimeTicks,
    ) {
        BreakSwapPromisesPolicy::<B::SwapPromise, B::CompositorFrameMetadata>(
            &mut self.swap_promise_list_,
            &mut self.pinned_swap_promise_list_,
            reason,
            timestamp,
        );
    }
    // Source: cc/trees/layer_tree_impl.cc:2372-2412.
    pub fn DidModifyTilePriorities(&mut self, pending: bool, c: &mut LayerTreeContext<'_, B>) {
        c.host.DidModifyTilePriorities(pending);
    }
    pub fn set_ui_resource_request_queue(
        &mut self,
        queue: Vec<B::UIResourceRequest>,
        _c: &LayerTreeContext<'_, B>,
    ) {
        self.ui_resource_request_queue_ = queue;
    }
    pub fn ResourceIdForUIResource(
        &self,
        id: UIResourceId,
        c: &LayerTreeContext<'_, B>,
    ) -> ResourceId {
        c.host.ResourceIdForUIResource(id)
    }
    pub fn GetUIResourceSize(&self, id: UIResourceId, c: &LayerTreeContext<'_, B>) -> Size {
        c.host.GetUIResourceSize(id)
    }
    pub fn IsUIResourceOpaque(&self, id: UIResourceId, c: &LayerTreeContext<'_, B>) -> bool {
        c.host.IsUIResourceOpaque(id)
    }
    pub fn ProcessUIResourceRequestQueue(&mut self, c: &mut LayerTreeContext<'_, B>) {
        for request in &self.ui_resource_request_queue_ {
            match request.GetType() {
                UIResourceRequestType::kCreate => {
                    c.host
                        .CreateUIResource(self, request.GetId(), request.GetBitmap())
                }
                UIResourceRequestType::kDelete => c.host.DeleteUIResource(self, request.GetId()),
            }
        }
        self.ui_resource_request_queue_.clear();
        if c.host.EvictedUIResourcesExist() {
            c.host.SetNeedsCommit();
        }
    }
}

impl<B: LayerTreeTypes> LayerTreeImpl<B> {
    // Source: cc/trees/layer_tree_impl.cc:2414-2443.
    pub fn NotifyLayerHasAnimatedImagesChanged(&mut self, id: i32, has: bool) {
        if has {
            self.layer_list_.SetPictureLayerWithAnimatedImages(id);
        } else {
            self.layer_list_.RemovePictureLayerWithAnimatedImages(id);
        }
        debug_assert_eq!(has, self.layer_list_.HasPictureLayerWithAnimatedImages(id));
    }
    pub fn AnnotateAnimatedImages(
        &self,
        map: &mut B::AnimatedImageDriverMap,
        context: &dyn crate::adapter::picture_layer_impl::PictureLayerReadContext,
    ) {
        for layer in self.layer_list_.PictureLayersWithAnimatedImages() {
            layer.AnnotateAnimatedImages(map, context);
        }
    }
    pub fn NotifyLayerHasPaintWorkletsChanged(&mut self, id: i32, has: bool) {
        if has {
            self.layer_list_.SetPictureLayerWithWorklet(id);
        } else {
            self.layer_list_.RemovePictureLayerWithWorklet(id);
        }
        debug_assert_eq!(has, self.layer_list_.HasPictureLayerWithWorklet(id));
    }
    // Source: cc/trees/layer_tree_impl.cc:2445-2492. The native current
    // pointer is lent with its remaining resident storage, not looked up twice.
    pub fn RegisterScrollbar(&mut self, id: i32, c: &mut LayerTreeContext<'_, B>) {
        B::WithLayerContext(self, c, id, |layer, context| {
            context.RegisterScrollbar(
                layer
                    .AsScrollbarLayerMut()
                    .expect("native scrollbar subtype"),
            );
        });
    }
    // Source: cc/trees/layer_tree_impl.cc:2494-2516.
    pub fn UnregisterScrollbar(&mut self, id: i32, c: &mut LayerTreeContext<'_, B>) {
        B::WithLayerContext(self, c, id, |layer, context| {
            context.UnregisterScrollbar(
                layer
                    .AsScrollbarLayerMut()
                    .expect("native scrollbar subtype"),
            );
        });
    }
    // Source: cc/trees/layer_tree_impl.cc:2518-2535.
    pub fn ScrollbarsFor(&self, element: ElementId) -> ScrollbarSet {
        let mut result = ScrollbarSet::new();
        if let Some(ids) = self.element_id_to_scrollbar_layer_ids_.get(&element) {
            for id in [ids.horizontal, ids.vertical] {
                if id != -1 {
                    if let Some(layer) = self.LayerById(id) {
                        result.insert(ScrollbarLayerHandle {
                            object_identity: layer as *const B::Layer as usize,
                            layer_id: id,
                        });
                    }
                }
            }
        }
        result
    }
    // Source: cc/trees/layer_tree_impl.cc:2576-2610.
    fn PointIsClippedByAncestorClipNode(&self, point: &PointF, layer: &B::Layer) -> bool {
        use crate::adapter::geometry::ToEnclosingRect;
        let clips = self.property_trees_.clip_tree();
        let transforms = self.property_trees_.transform_tree();
        let mut clip = ToEnclosingRect(clips.Node(1).clip);
        if !PointHitsRect::<B::Transform>(point, &B::Transform::default(), &clip, None) {
            return true;
        }
        let mut id = layer.clip_tree_index();
        while id > super::property_ids::kViewportPropertyNodeId {
            let node = clips.Node(id);
            if node.AppliesLocalClip() {
                clip = ToEnclosingRect(node.clip);
                if !PointHitsRect(point, transforms.ToScreen(node.transform_id), &clip, None) {
                    return true;
                }
            }
            id = node.parent_id;
        }
        false
    }
    // Source: cc/trees/layer_tree_impl.cc:2612-2618.
    fn PointIsClippedBySurfaceOrClipRect(&self, point: &PointF, layer: &B::Layer) -> bool {
        self.PointIsClippedByAncestorClipNode(point, layer)
    }
    // Source: cc/trees/layer_tree_impl.cc:2620-2661.
    fn PointHitsRegion(
        &self,
        touch: &RectF,
        transform: &B::Transform,
        region: &Region<B::RegionBackend>,
        layer: Option<&B::Layer>,
    ) -> bool {
        use crate::adapter::{geometry::QuadF, property_tree::PropertyTreeTransform};
        use crate::cc::base::math_util::MathUtil;
        if region.IsEmpty() {
            return false;
        }
        let mut inverse = B::Transform::default();
        if !PropertyTreeTransform::GetInverse(transform, &mut inverse) {
            return false;
        }
        let mut clipped = false;
        let rect = MathUtil::MapQuad(&inverse, QuadF::FromRect(*touch), &mut clipped).BoundingBox();
        if clipped {
            return false;
        }
        if layer.is_some_and(|layer| {
            self.PointIsClippedBySurfaceOrClipRect(&touch.CenterPoint(), layer)
        }) {
            return false;
        }
        let hit_rect = Rect::new(
            rect.x().round() as i32,
            rect.y().round() as i32,
            (rect.width().round() as i32).max(1),
            (rect.height().round() as i32).max(1),
        );
        region.Intersects(&hit_rect)
    }
    // Source: cc/trees/layer_tree_impl.cc:2663-2683.
    fn PointHitsLayer(&self, layer: &B::Layer, point: &PointF, distance: Option<&mut f32>) -> bool {
        let bounds = layer.bounds(&self.property_trees_);
        let rect = Rect::new(0, 0, bounds.width(), bounds.height());
        if !PointHitsRect(
            point,
            &layer.ScreenSpaceTransform(&self.property_trees_),
            &rect,
            distance,
        ) {
            return false;
        }
        if self.PointIsClippedBySurfaceOrClipRect(point, layer) {
            return false;
        }
        if self.hud_layer_ == Some(layer.id()) {
            return false;
        }
        true
    }
    // Source: cc/trees/layer_tree_impl.cc:2699-2740.
    fn FindClosestMatchingLayer<'a>(
        &'a self,
        touch: &RectF,
        func: impl Fn(&B::Layer) -> bool,
        state: &mut FindClosestMatchingLayerState<'a, B::Layer>,
    ) {
        assert!(!self.layer_list_.is_empty());
        for layer in self.layer_list_.iter().rev() {
            if !func(layer) {
                continue;
            }
            let mut distance = 0.0;
            let hit = if layer.Is3dSorted(&self.property_trees_) {
                self.PointHitsLayer(layer, &touch.CenterPoint(), Some(&mut distance))
            } else {
                self.PointHitsLayer(layer, &touch.CenterPoint(), None)
            };
            if !hit {
                continue;
            }
            let in_front = state.closest_match.is_some_and(|previous| {
                layer.GetSortingContextId(&self.property_trees_)
                    == previous.GetSortingContextId(&self.property_trees_)
                    && distance > state.closest_distance + f32::EPSILON
            });
            if state.closest_match.is_none() || in_front {
                state.closest_distance = distance;
                state.closest_match = Some(layer);
            }
        }
    }
    // Source: cc/trees/layer_tree_impl.cc:2746-2760.
    pub fn FindLayerThatIsHitByPoint(
        &mut self,
        point: &PointF,
        c: &mut LayerTreeContext<'_, B>,
    ) -> Option<&B::Layer> {
        self.FindLayerThatIsHitByPointWithHost(point, &mut draw_host::LayerTreeDrawHostRef(c.host))
    }
    // Borrow-only entry; the canonical source body is shared with the full host.
    pub fn FindLayerThatIsHitByPointWithHost(
        &mut self,
        point: &PointF,
        host: &mut dyn draw_host::LayerTreeDrawHost<B>,
    ) -> Option<&B::Layer> {
        if self.layer_list_.is_empty() {
            return None;
        }
        let update_tiles = !host.IsCCSlimmingEnabled();
        if !self.UpdateDrawPropertiesWithHost(update_tiles, true, None, host) {
            return None;
        }
        let mut state = FindClosestMatchingLayerState::default();
        self.FindClosestMatchingLayer(
            &RectF::FromOriginSize(*point, SizeF::default()),
            |layer| layer.HitTestable(&self.property_trees_),
            &mut state,
        );
        state.closest_match
    }
    // Source: cc/trees/layer_tree_impl.cc:2783-2818.
    fn FindLayerThatIsHitByPointInEventHandlerRegion(
        &mut self,
        touch: &RectF,
        kind: EventHandlerRegion,
        c: &mut LayerTreeContext<'_, B>,
    ) -> Option<&B::Layer> {
        self.FindLayerThatIsHitByPointInEventHandlerRegionWithHost(
            touch,
            kind,
            &mut draw_host::LayerTreeDrawHostRef(c.host),
        )
    }
    // Borrow-only entry; the canonical source body is shared with the full host.
    pub(crate) fn FindLayerThatIsHitByPointInEventHandlerRegionWithHost(
        &mut self,
        touch: &RectF,
        kind: EventHandlerRegion,
        host: &mut dyn draw_host::LayerTreeDrawHost<B>,
    ) -> Option<&B::Layer> {
        if self.layer_list_.is_empty() {
            return None;
        }
        let update_tiles = !host.IsCCSlimmingEnabled();
        if !self.UpdateDrawPropertiesWithHost(update_tiles, true, None, host) {
            return None;
        }
        let mut state = FindClosestMatchingLayerState::default();
        self.FindClosestMatchingLayer(
            touch,
            |layer| match kind {
                EventHandlerRegion::Touch => {
                    layer.has_touch_action_regions()
                        && self.PointHitsRegion(
                            touch,
                            &layer.ScreenSpaceTransform(&self.property_trees_),
                            layer.GetAllTouchActionRegions(),
                            Some(layer),
                        )
                }
                EventHandlerRegion::Wheel => self.PointHitsRegion(
                    touch,
                    &layer.ScreenSpaceTransform(&self.property_trees_),
                    &layer.wheel_event_handler_region(),
                    Some(layer),
                ),
            },
            &mut state,
        );
        state.closest_match
    }
    pub fn FindLayerThatIsHitByPointInTouchHandlerRegion(
        &mut self,
        touch: &RectF,
        c: &mut LayerTreeContext<'_, B>,
    ) -> Option<&B::Layer> {
        self.FindLayerThatIsHitByPointInTouchHandlerRegionWithHost(
            touch,
            &mut draw_host::LayerTreeDrawHostRef(c.host),
        )
    }
    // Borrow-only entry; the canonical source body is shared with the full host.
    pub fn FindLayerThatIsHitByPointInTouchHandlerRegionWithHost(
        &mut self,
        touch: &RectF,
        host: &mut dyn draw_host::LayerTreeDrawHost<B>,
    ) -> Option<&B::Layer> {
        self.FindLayerThatIsHitByPointInEventHandlerRegionWithHost(
            touch,
            EventHandlerRegion::Touch,
            host,
        )
    }
    pub fn FindLayerThatIsHitByPointInTouchHandlerRegionAtPoint(
        &mut self,
        point: &PointF,
        c: &mut LayerTreeContext<'_, B>,
    ) -> Option<&B::Layer> {
        self.FindLayerThatIsHitByPointInTouchHandlerRegion(
            &RectF::FromOriginSize(*point, SizeF::default()),
            c,
        )
    }
    pub fn FindLayerThatIsHitByPointInWheelEventHandlerRegion(
        &mut self,
        point: &PointF,
        c: &mut LayerTreeContext<'_, B>,
    ) -> Option<&B::Layer> {
        self.FindLayerThatIsHitByPointInWheelEventHandlerRegionWithHost(
            point,
            &mut draw_host::LayerTreeDrawHostRef(c.host),
        )
    }
    // Borrow-only entry; the canonical source body is shared with the full host.
    pub fn FindLayerThatIsHitByPointInWheelEventHandlerRegionWithHost(
        &mut self,
        point: &PointF,
        host: &mut dyn draw_host::LayerTreeDrawHost<B>,
    ) -> Option<&B::Layer> {
        self.FindLayerThatIsHitByPointInEventHandlerRegionWithHost(
            &RectF::FromOriginSize(*point, SizeF::default()),
            EventHandlerRegion::Wheel,
            host,
        )
    }
    // Source: cc/trees/layer_tree_impl.cc:2821-2925.
    pub fn FindLayersUpToFirstScrollableOrOpaqueToHitTest(
        &mut self,
        point: &PointF,
        c: &mut LayerTreeContext<'_, B>,
    ) -> Vec<&B::Layer> {
        self.FindLayersUpToFirstScrollableOrOpaqueToHitTestWithHost(
            point,
            &mut draw_host::LayerTreeDrawHostRef(c.host),
        )
    }
    // Borrow-only entry; the canonical source body is shared with the full host.
    pub fn FindLayersUpToFirstScrollableOrOpaqueToHitTestWithHost(
        &mut self,
        point: &PointF,
        host: &mut dyn draw_host::LayerTreeDrawHost<B>,
    ) -> Vec<&B::Layer> {
        let mut layers = Vec::new();
        if self.layer_list_.is_empty() {
            return layers;
        }
        let update_tiles = !host.IsCCSlimmingEnabled();
        let updated = self.UpdateDrawPropertiesWithHost(update_tiles, true, None, host);
        if !updated {
            return layers;
        }
        let mut layers_3d = Vec::new();
        let mut first: Option<&B::Layer> = None;
        for layer in self.layer_list_.iter().rev() {
            let hit_testable = layer.HitTestable(&self.property_trees_);
            if !hit_testable {
                continue;
            }
            if first.is_some_and(|first| {
                layer.Is3dSorted(&self.property_trees_)
                    && layer.GetSortingContextId(&self.property_trees_)
                        != first.GetSortingContextId(&self.property_trees_)
            }) {
                continue;
            }
            let mut distance = 0.0;
            let hit = if layer.Is3dSorted(&self.property_trees_) {
                self.PointHitsLayer(layer, point, Some(&mut distance))
            } else {
                self.PointHitsLayer(layer, point, None)
            };
            if !hit {
                continue;
            }
            let first = *first.get_or_insert(layer);
            if first.Is3dSorted(&self.property_trees_) && layer.Is3dSorted(&self.property_trees_) {
                layers_3d.push((layer, distance));
            } else {
                layers.push(layer);
                if if host.settings().enable_hit_test_opaqueness() {
                    layer.OpaqueToHitTest(&self.property_trees_)
                } else {
                    layer.IsScrollerOrScrollbar(&self.property_trees_)
                } {
                    break;
                }
            }
        }
        let Some(first) = first else {
            debug_assert!(layers.is_empty());
            debug_assert!(layers_3d.is_empty());
            return layers;
        };
        if first.Is3dSorted(&self.property_trees_) {
            let mut result = Vec::new();
            debug_assert!(!layers_3d.is_empty());
            layers_3d.sort_unstable_by(|a, b| {
                b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
            });
            for (layer, _) in layers_3d {
                result.push(layer);
                if if host.settings().enable_hit_test_opaqueness() {
                    layer.OpaqueToHitTest(&self.property_trees_)
                } else {
                    layer.IsScrollerOrScrollbar(&self.property_trees_)
                } {
                    return result;
                }
            }
            result.extend(layers);
            return result;
        } else {
            debug_assert!(!layers.is_empty());
            debug_assert!(layers_3d.is_empty());
        }
        layers
    }
    // Source: cc/trees/layer_tree_impl.cc:2927-2957.
    pub fn PointHitsMainThreadScrollHitTestRegion(&self, point: &PointF, layer: &B::Layer) -> bool {
        debug_assert!(self.PointHitsLayer(layer, point, None));
        if layer.main_thread_scroll_hit_test_region().IsEmpty() {
            return false;
        }
        self.PointHitsRegion(
            &RectF::FromOriginSize(*point, SizeF::default()),
            &layer.ScreenSpaceTransform(&self.property_trees_),
            &layer.main_thread_scroll_hit_test_region(),
            Some(layer),
        )
    }
    pub fn PointHitsNonCompositedScroll(&self, point: &PointF, layer: &B::Layer) -> ElementId {
        let Some(rects) = layer.non_composited_scroll_hit_test_rects() else {
            return ElementId::default();
        };
        for rect in rects.iter().rev() {
            if PointHitsRect(
                point,
                &layer.ScreenSpaceTransform(&self.property_trees_),
                &rect.hit_test_rect,
                None,
            ) {
                return rect.scroll_element_id;
            }
        }
        ElementId::default()
    }
    // Source: cc/trees/layer_tree_impl.cc:2959-2971.
    fn GetFrameElementIdForLayer(&self, layer: &B::Layer) -> ElementId {
        let transforms = self.property_trees_.transform_tree();
        let mut id = layer.transform_tree_index();
        while id != kInvalidPropertyNodeId {
            let node = transforms.Node(id);
            if node.visible_frame_element_id != ElementId::default() {
                return node.visible_frame_element_id;
            }
            id = node.parent_frame_id;
        }
        ElementId::default()
    }
    // Source: cc/trees/layer_tree_impl.cc:2973-3037.
    fn FindClosestMatchingLayerForAttribution<'a>(
        &'a self,
        point: &PointF,
        state: &mut FindClosestMatchingLayerState<'a, B::Layer>,
    ) {
        let mut frames = std::collections::HashSet::new();
        for layer in self.layer_list_.iter().rev() {
            if !layer.HitTestable(&self.property_trees_) {
                continue;
            }
            let mut distance = 0.0;
            let hit = if layer.Is3dSorted(&self.property_trees_) {
                self.PointHitsLayer(layer, point, Some(&mut distance))
            } else {
                self.PointHitsLayer(layer, point, None)
            };
            if !hit {
                continue;
            }
            let in_front = state.closest_match.is_some_and(|previous| {
                layer.GetSortingContextId(&self.property_trees_)
                    == previous.GetSortingContextId(&self.property_trees_)
                    && distance > state.closest_distance + f32::EPSILON
            });
            if state.closest_match.is_none() || in_front {
                state.closest_distance = distance;
                state.closest_match = Some(layer);
            }
            frames.insert(self.GetFrameElementIdForLayer(layer));
        }
        if let Some(layer) = state.closest_match {
            let transforms = self.property_trees_.transform_tree();
            let mut id = layer.transform_tree_index();
            while id != kInvalidPropertyNodeId {
                let node = transforms.Node(id);
                frames.remove(&node.visible_frame_element_id);
                if frames.is_empty() {
                    break;
                }
                id = node.parent_frame_id;
            }
            if !frames.is_empty() {
                state.closest_distance = 0.0;
                state.closest_match = None;
            }
        }
    }
    // Source: cc/trees/layer_tree_impl.cc:3039-3065.
    pub fn FindFrameElementIdAtPoint(
        &mut self,
        point: &PointF,
        c: &mut LayerTreeContext<'_, B>,
    ) -> ElementId {
        self.FindFrameElementIdAtPointWithHost(point, &mut draw_host::LayerTreeDrawHostRef(c.host))
    }
    // Borrow-only entry; the canonical source body is shared with the full host.
    pub fn FindFrameElementIdAtPointWithHost(
        &mut self,
        point: &PointF,
        host: &mut dyn draw_host::LayerTreeDrawHost<B>,
    ) -> ElementId {
        if self.layer_list_.is_empty() {
            return ElementId::default();
        }
        let update_tiles = !host.IsCCSlimmingEnabled();
        if !self.UpdateDrawPropertiesWithHost(update_tiles, true, None, host) {
            return ElementId::default();
        }
        let mut state = FindClosestMatchingLayerState::default();
        self.FindClosestMatchingLayerForAttribution(point, &mut state);
        if let Some(layer) = state.closest_match {
            let masked = self
                .property_trees_
                .effect_tree()
                .HitTestMayBeAffectedByMask(layer.effect_tree_index());
            if !masked {
                return self.GetFrameElementIdForLayer(layer);
            }
        }
        ElementId::default()
    }
}
// Source: cc/trees/layer_tree_impl.cc:2537-2574.
fn PointHitsRect<T: LayerTreeTransform>(
    point: &PointF,
    transform: &T,
    rect: &Rect,
    distance: Option<&mut f32>,
) -> bool {
    use crate::{
        adapter::{
            geometry::{Point3F, RectF},
            property_tree::PropertyTreeTransform,
        },
        cc::base::math_util::MathUtil,
    };
    let mut inverse = T::default();
    if !PropertyTreeTransform::GetInverse(transform, &mut inverse) {
        return false;
    }
    let mut clipped = false;
    let local = MathUtil::ProjectPoint(&inverse, *point, &mut clipped);
    if clipped {
        return false;
    }
    if !crate::adapter::scrollbar::RectContainsPoint(RectF::from(*rect), local) {
        return false;
    }
    if let Some(distance) = distance {
        *distance = transform
            .MapPoint3D(Point3F::new(local.x(), local.y(), 0.0))
            .z();
    }
    true
}
// Source: cc/trees/layer_tree_impl.cc:2685-2696.
struct FindClosestMatchingLayerState<'a, L> {
    closest_match: Option<&'a L>,
    closest_distance: f32,
}
impl<L> Default for FindClosestMatchingLayerState<'_, L> {
    fn default() -> Self {
        Self {
            closest_match: None,
            closest_distance: f32::NEG_INFINITY,
        }
    }
}
#[derive(Clone, Copy)]
enum EventHandlerRegion {
    Touch,
    Wheel,
}

impl<B: LayerTreeTypes> LayerTreeImpl<B> {
    // Source: cc/trees/layer_tree_impl.cc:3067-3077.
    pub fn RegisterSelection(&mut self, selection: &LayerSelection) {
        if self.selection_ == *selection {
            return;
        }
        self.handle_visibility_changed_ = true;
        self.selection_ = selection.clone();
    }
    pub fn ResetHandleVisibilityChanged(&mut self) {
        self.handle_visibility_changed_ = false;
    }
    // Source: cc/trees/layer_tree_impl.cc:3079-3179.
    fn ComputeViewportSelectionBound(
        &self,
        bound: &crate::cc::input::layer_selection_bound::LayerSelectionBound,
        layer: Option<&B::Layer>,
        device_scale: f32,
        c: &LayerTreeContext<'_, B>,
    ) -> B::SelectionBound {
        use crate::{adapter::geometry::BoundingRect, cc::base::math_util::MathUtil};
        let mut viewport = B::SelectionBound::default();
        viewport.set_type(bound.r#type);
        let Some(layer) = layer else {
            return viewport;
        };
        if bound.r#type == SelectionBoundType::EMPTY {
            return viewport;
        }
        let start = PointF::new(bound.edge_start.x() as f32, bound.edge_start.y() as f32);
        let end = PointF::new(bound.edge_end.x() as f32, bound.edge_end.y() as f32);
        let transform = layer.ScreenSpaceTransform(&self.property_trees_);
        let mut clipped = false;
        let screen_start = MathUtil::MapPoint(&transform, start, &mut clipped);
        let screen_end = MathUtil::MapPoint(&transform, end, &mut clipped);
        let inv = 1.0 / device_scale;
        viewport.SetEdgeStart(PointF::new(screen_start.x() * inv, screen_start.y() * inv));
        viewport.SetEdgeEnd(PointF::new(screen_end.x() * inv, screen_end.y() * inv));
        if bound.hidden {
            viewport.set_visible(false);
        } else {
            let full = c.host.SelectionEdgeVisibilityUsesFullEdge();
            let is_visible = |point: PointF| {
                let mut clipped = false;
                let point = MathUtil::MapPoint(&transform, point, &mut clipped);
                self.PointHitsLayer(layer, &point, None)
            };
            let mut visible = false;
            if full {
                visible = is_visible(end) || is_visible(start);
                if !visible {
                    let mut rect = BoundingRect(start, end);
                    let bounds = layer.bounds(&self.property_trees_);
                    if InclusiveIntersectRectF(
                        &mut rect,
                        RectF::new(0.0, 0.0, bounds.width() as f32, bounds.height() as f32),
                    ) {
                        visible = is_visible(rect.CenterPoint());
                    }
                }
            } else {
                let mut offset = start - end;
                offset.Scale(device_scale / offset.Length());
                let mut point = end + offset;
                if point.x() < 0.0 {
                    point.set_x(point.x() + device_scale);
                }
                visible = is_visible(point);
            }
            viewport.set_visible(visible);
        }
        if viewport.visible() {
            viewport.SetVisibleEdge(viewport.edge_start(), viewport.edge_end());
        } else {
            let visible_rect = RectF::from(layer.visible_layer_rect());
            let (mut visible_start, mut visible_end) = (start, end);
            if !crate::adapter::scrollbar::RectContainsPoint(visible_rect, visible_start)
                && !crate::adapter::scrollbar::RectContainsPoint(visible_rect, visible_end)
            {
                (visible_start, visible_end) =
                    GetVisibleSelectionEndPoints(&visible_rect, &start, &end);
            }
            let visible_start = MathUtil::MapPoint(&transform, visible_start, &mut clipped);
            let visible_end = MathUtil::MapPoint(&transform, visible_end, &mut clipped);
            viewport.SetVisibleEdgeStart(PointF::new(
                visible_start.x() * inv,
                visible_start.y() * inv,
            ));
            viewport.SetVisibleEdgeEnd(PointF::new(visible_end.x() * inv, visible_end.y() * inv));
        }
        viewport
    }
    // Source: cc/trees/layer_tree_impl.cc:3181-3200.
    pub fn GetViewportSelection(
        &self,
        selection: &mut Selection<B::SelectionBound>,
        c: &LayerTreeContext<'_, B>,
    ) {
        let start = if self.selection_.start.layer_id != 0 {
            self.LayerById(self.selection_.start.layer_id)
        } else {
            None
        };
        let scale = self.device_scale_factor() * self.painted_device_scale_factor();
        selection.start =
            self.ComputeViewportSelectionBound(&self.selection_.start, start, scale, c);
        if matches!(
            selection.start.r#type(),
            SelectionBoundType::CENTER | SelectionBoundType::HIDDEN | SelectionBoundType::EMPTY
        ) {
            selection.end = selection.start.clone();
        } else {
            let end = if self.selection_.end.layer_id != 0 {
                self.LayerById(self.selection_.end.layer_id)
            } else {
                None
            };
            selection.end = self.ComputeViewportSelectionBound(&self.selection_.end, end, scale, c);
        }
    }
    // Source: cc/trees/layer_tree_impl.cc:3202-3225.
    pub fn SmoothnessTakesPriority(&self, c: &LayerTreeContext<'_, B>) -> bool {
        c.host.GetTreePriority()
            == crate::adapter::scheduler::TreePriority::SMOOTHNESS_TAKES_PRIORITY
    }
    pub fn GetVideoFrameControllerClient<'a>(
        &self,
        c: &'a LayerTreeContext<'_, B>,
    ) -> &'a B::VideoFrameControllerClient {
        c.host.GetVideoFrameControllerClient()
    }
    pub fn UpdateImageDecodingHints(
        &mut self,
        map: BTreeMap<B::PaintImageId, B::DecodingMode>,
        c: &mut LayerTreeContext<'_, B>,
    ) {
        c.host.UpdateImageDecodingHints(map);
    }
    pub fn GetMSAASampleCountForRaster(
        &self,
        list: &B::DisplayItemList,
        c: &LayerTreeContext<'_, B>,
    ) -> i32 {
        c.host.GetMSAASampleCountForRaster(self, list)
    }
    pub fn GetTargetColorParams(
        &self,
        usage: B::ContentColorUsage,
        c: &LayerTreeContext<'_, B>,
    ) -> B::TargetColorParams {
        c.host.GetTargetColorParams(self, usage)
    }
    // Source: cc/trees/layer_tree_impl.cc:3227-3273.
    pub fn SetPendingPageScaleAnimation(
        &mut self,
        value: Option<Box<crate::cc::input::page_scale_animation::PendingPageScaleAnimation>>,
    ) {
        self.pending_page_scale_animation_ = value;
    }
    pub fn TakePendingPageScaleAnimation(
        &mut self,
    ) -> Option<Box<crate::cc::input::page_scale_animation::PendingPageScaleAnimation>> {
        self.pending_page_scale_animation_.take()
    }
    pub fn AppendEventsMetricsFromMainThread(&mut self, mut events: Vec<B::EventMetrics>) {
        self.events_metrics_from_main_thread_.reserve(events.len());
        self.events_metrics_from_main_thread_.append(&mut events);
    }
    pub fn AppendEventMetricsFromRasterThread(&mut self, mut events: Vec<B::EventMetrics>) {
        self.event_metrics_from_raster_thread_.reserve(events.len());
        self.event_metrics_from_raster_thread_.append(&mut events);
    }
    pub fn TakeEventsMetrics(&mut self) -> Vec<B::EventMetrics> {
        std::mem::take(&mut self.events_metrics_from_main_thread_)
    }
    pub fn TakeRasterEventsMetrics(&mut self) -> Vec<B::EventMetrics> {
        std::mem::take(&mut self.event_metrics_from_raster_thread_)
    }
    pub fn TakeForceSendMetadataRequest(&mut self) -> bool {
        let request = self.force_send_metadata_request_;
        self.force_send_metadata_request_ = false;
        request
    }
    // Source: cc/trees/layer_tree_impl.cc:3275-3292.
    pub fn ResetAllChangeTracking(&mut self) {
        for layer in self.layer_list_.iter_mut() {
            layer.ResetChangeTracking();
        }
        self.property_trees_.ResetAllChangeTracking();
    }
    pub fn LayerListAsJson(&mut self, context: &mut LayerTreeContext<'_, B>) -> String {
        use crate::adapter::{
            geometry::TracedValue,
            property_types::{PropertyTrace, TracedValueJSON},
        };
        let mut value = context.host.NewLayerTraceValue();
        value.BeginArray("LayerTreeImpl");
        self.ForEachLayerWithContext(false, context, |layer, layer_context| {
            value.BeginDictionaryUnnamed();
            layer.AsValueInto(&mut value, layer_context);
            value.EndDictionary();
        });
        value.EndArray();
        crate::adapter::layer_impl::LayerImplTraceValue::ToFormattedJSON(&value)
    }
    // Source: cc/trees/layer_tree_impl.cc:3294-3307.
    pub fn AddViewTransitionRequest(
        &mut self,
        request: Box<B::ViewTransitionRequest>,
        c: &mut LayerTreeContext<'_, B>,
    ) {
        if self.IsActiveTree(c) && request.IsSave() {
            c.host.ResetBrowserControlsAnimations(self);
        }
        self.view_transition_requests_.push(request);
        self.SetNeedsRedraw(c);
    }
    // Source: cc/trees/layer_tree_impl.cc:3310-3336.
    pub fn TakeViewTransitionRequests(
        &mut self,
        update: bool,
    ) -> Vec<Box<B::ViewTransitionRequest>> {
        if self.HasViewTransitionRequests() && update {
            self.set_needs_update_draw_properties();
        }
        std::mem::take(&mut self.view_transition_requests_)
    }
    pub fn HasViewTransitionRequests(&self) -> bool {
        !self.view_transition_requests_.is_empty()
    }
    pub fn HasViewTransitionSaveRequest(&self) -> bool {
        for request in &self.view_transition_requests_ {
            if request.IsSave() {
                return true;
            }
        }
        false
    }
    pub fn IsAnimatingHUDContents(&self, c: &LayerTreeContext<'_, B>) -> bool {
        if c.host.settings().trees_in_viz_in_viz_process() {
            return self.is_animating_hud_contents_;
        }
        self.hud_layer_
            .and_then(|id| self.LayerById(id))
            .is_some_and(|layer| {
                layer
                    .AsHeadsUpDisplayLayer()
                    .expect("source hud_layer subtype")
                    .IsAnimatingHUDContents()
            })
    }
    // Source: cc/trees/layer_tree_impl.cc:3339-3354.
    pub fn GetCaptureViewTransitionTokens(
        &self,
        c: &LayerTreeContext<'_, B>,
    ) -> BTreeSet<crate::adapter::draw_property::ViewTransitionToken> {
        self.GetCaptureViewTransitionTokensWithFeature(|| c.host.ViewTransitionCaptureAndDisplay())
    }
    // Borrow-only entry; the canonical source body is shared with the full host.
    pub fn GetCaptureViewTransitionTokensWithHost(
        &self,
        host: &dyn draw_host::LayerTreeDrawHost<B>,
    ) -> BTreeSet<crate::adapter::draw_property::ViewTransitionToken> {
        self.GetCaptureViewTransitionTokensWithFeature(|| host.ViewTransitionCaptureAndDisplay())
    }
    // Source: cc/trees/layer_tree_impl.cc:3339-3354. A lazy external feature
    // query is the sole owner capability required by this native loop.
    fn GetCaptureViewTransitionTokensWithFeature(
        &self,
        enabled: impl FnOnce() -> bool,
    ) -> BTreeSet<crate::adapter::draw_property::ViewTransitionToken> {
        let mut result = BTreeSet::new();
        if !enabled() {
            return result;
        }
        for request in &self.view_transition_requests_ {
            if request.IsSave() {
                result.insert(request.token());
            }
        }
        result
    }
    // Source: cc/trees/layer_tree_impl.cc:3356-3368.
    pub fn SetViewTransitionContentRect(
        &self,
        id: &crate::adapter::property_types::ViewTransitionElementResourceId,
        rect: &RectF,
        c: &mut LayerTreeContext<'_, B>,
    ) {
        Self::SetViewTransitionContentRectForRequests(
            &self.view_transition_requests_,
            id,
            rect,
            &mut draw_host::LayerTreeDrawHostRef(c.host),
        );
    }
    // Source: cc/trees/layer_tree_impl.cc:3355-3368. The request slice and host
    // map are unique live borrows; all native filtering stays in this body.
    pub fn SetViewTransitionContentRectForRequests(
        requests: &[Box<B::ViewTransitionRequest>],
        id: &crate::adapter::property_types::ViewTransitionElementResourceId,
        rect: &RectF,
        host: &mut dyn draw_host::LayerTreeDrawHost<B>,
    ) {
        for request in requests {
            if request.IsSave() && Some(request.token()) == id.transition_token {
                host.SetViewTransitionContentRect(request.sequence_id(), id, rect);
            }
        }
    }
    // Source: cc/trees/layer_tree_impl.cc:3370-3382.
    pub fn IsReadyToActivate(&self, c: &LayerTreeContext<'_, B>) -> bool {
        c.host.IsReadyToActivate()
    }
    pub fn RequestImplSideInvalidationForRerasterTiling(
        &mut self,
        c: &mut LayerTreeContext<'_, B>,
    ) {
        c.host.RequestImplSideInvalidationForRerasterTiling();
    }
    pub fn AddLayerNeedingUpdateDiscardableImageMap(&self, id: i32, updates: &mut Vec<i32>) {
        assert!(self.discardable_image_map_updater_.is_some());
        updates.push(id);
    }
}

impl<B: LayerTreeTypes> LayerTreeImpl<B> {
    // Source: cc/trees/layer_tree_impl.h:297-337,402-411,429-487,516-550,724-741,778-784.
    pub fn source_frame_number(&self) -> i32 {
        self.source_frame_number_
    }
    pub fn trace_id(&self) -> crate::adapter::commit_types::BeginMainFrameTraceId {
        self.trace_id_
    }
    pub fn background_color(&self) -> Color4f {
        self.background_color_
    }
    pub fn display_transform_hint(&self) -> OverlayTransform {
        self.display_transform_hint_
    }
    pub fn primary_main_frame_item_sequence_number(&self) -> i64 {
        self.primary_main_frame_item_sequence_number_
    }
    pub fn device_scale_factor(&self) -> f32 {
        self.device_scale_factor_
    }
    pub fn painted_device_scale_factor(&self) -> f32 {
        self.painted_device_scale_factor_
    }
    pub fn external_page_scale_factor(&self) -> f32 {
        self.external_page_scale_factor_
    }
    pub fn min_page_scale_factor(&self) -> f32 {
        self.min_page_scale_factor_
    }
    pub fn max_page_scale_factor(&self) -> f32 {
        self.max_page_scale_factor_
    }
    pub fn max_safe_area_inset_bottom(&self) -> f32 {
        self.max_safe_area_inset_bottom_
    }
    pub fn needs_full_tree_sync(&self) -> bool {
        self.needs_full_tree_sync_
    }
    pub fn needs_surface_ranges_sync(&self) -> bool {
        self.needs_surface_ranges_sync_
    }
    pub fn needs_update_draw_properties(&self) -> bool {
        self.needs_update_draw_properties_
    }
    pub fn have_scroll_event_handlers(&self) -> bool {
        self.have_scroll_event_handlers_
    }
    pub fn did_raster_inducing_scroll(&self) -> bool {
        self.did_raster_inducing_scroll_
    }
    pub fn device_viewport_rect_changed(&self) -> bool {
        self.device_viewport_rect_changed_
    }
    pub fn property_change_forces_commit_criteria(&self) -> PropertyChangeForcesCommitCriteria {
        self.property_change_forces_commit_criteria_
    }
    pub fn overscroll_behavior(&self) -> OverscrollBehavior {
        self.overscroll_behavior_
    }
    pub fn set_source_frame_number(&mut self, value: i32) {
        self.source_frame_number_ = value;
    }
    pub fn set_trace_id(&mut self, value: crate::adapter::commit_types::BeginMainFrameTraceId) {
        self.trace_id_ = value;
    }
    pub fn set_background_color(&mut self, value: Color4f) {
        self.background_color_ = value;
    }
    pub fn set_display_transform_hint(&mut self, value: OverlayTransform) {
        self.display_transform_hint_ = value;
    }
    pub fn set_primary_main_frame_item_sequence_number(&mut self, value: i64) {
        self.primary_main_frame_item_sequence_number_ = value;
    }
    pub fn set_painted_device_scale_factor(&mut self, value: f32) {
        self.painted_device_scale_factor_ = value;
    }
    pub fn set_needs_full_tree_sync(&mut self, value: bool) {
        self.needs_full_tree_sync_ = value;
    }
    pub fn set_needs_surface_ranges_sync(&mut self, value: bool) {
        self.needs_surface_ranges_sync_ = value;
    }
    pub fn set_have_scroll_event_handlers(&mut self, value: bool) {
        self.have_scroll_event_handlers_ = value;
    }

    // Source: cc/trees/layer_tree_impl.h:143-146,182-183,307-313,333-337,389-396,414-416,440-443,475-500,610-615,665-702,745,749-760,788-793,815-820,830-835.
    pub fn global_tile_state<'a>(&self, c: &'a LayerTreeContext<'_, B>) -> &'a B::GlobalTileState {
        c.host.global_tile_state()
    }
    pub fn mutator_host<'a>(&self, c: &'a LayerTreeContext<'_, B>) -> &'a B::MutatorHost {
        c.host.mutator_host()
    }
    pub fn hud_layer(&self) -> Option<&B::Layer> {
        self.hud_layer_.and_then(|id| self.LayerById(id))
    }
    pub fn set_hud_layer(&mut self, id: Option<i32>) {
        self.hud_layer_ = id;
    }
    pub fn ViewportPropertyIdsForTesting(&self) -> ViewportPropertyIds {
        self.viewport_property_ids_
    }
    pub fn SetCurrentlyScrollingNode(
        &mut self,
        node: Option<&ScrollNode>,
        c: &LayerTreeContext<'_, B>,
    ) {
        self.SetCurrentlyScrollingNodeById(node.map(|node| node.id), c);
    }
    pub fn current_page_scale_factor(&self, c: &LayerTreeContext<'_, B>) -> f32 {
        self.current_page_scale_factor_for_tree(self.IsActiveTree(c))
    }
    // Source: cc/trees/layer_tree_impl.h:390-392.
    // Rust borrow split: source active_tree pointer identity is supplied live.
    pub(crate) fn current_page_scale_factor_for_tree(&self, active: bool) -> f32 {
        self.page_scale_factor_.borrow().Current(active)
    }
    pub fn page_scale_delta(&self) -> f32 {
        self.page_scale_factor_.borrow().Delta()
    }
    pub fn selection(&self) -> &LayerSelection {
        &self.selection_
    }
    pub fn local_surface_id_from_parent(&self) -> &B::LocalSurfaceId {
        &self.local_surface_id_from_parent_
    }
    pub fn new_local_surface_id_request_for_testing(&self) -> bool {
        self.new_local_surface_id_request_
    }
    pub fn internal_device_viewport(&self) -> Rect {
        self.device_viewport_rect_
    }
    pub fn display_color_spaces(&self) -> &B::DisplayColorSpaces {
        &self.display_color_spaces_
    }
    pub fn page_scale_factor_for_scroll(&self, c: &LayerTreeContext<'_, B>) -> f32 {
        self.page_scale_factor_for_scroll_for_tree(
            self.IsActiveTree(c),
            c.host.settings().is_for_scalable_page(),
        )
    }
    // Source: cc/trees/layer_tree_impl.h:463-468.
    // Rust borrow split of live identity and immutable source settings.
    pub(crate) fn page_scale_factor_for_scroll_for_tree(
        &self,
        active: bool,
        scalable_page: bool,
    ) -> f32 {
        debug_assert!(
            self.external_page_scale_factor_ == 1.0
                || self.current_page_scale_factor_for_tree(active) == 1.0
                || scalable_page
        );
        self.external_page_scale_factor_ * self.current_page_scale_factor_for_tree(active)
    }

    pub fn viewport_property_ids(&self) -> &ViewportPropertyIds {
        &self.viewport_property_ids_
    }
    pub fn top_controls_shown_ratio(&self) -> Ref<'_, SyncedBrowserControls> {
        self.top_controls_shown_ratio_.borrow()
    }
    pub fn top_controls_shown_ratio_mut(&mut self) -> RefMut<'_, SyncedBrowserControls> {
        self.top_controls_shown_ratio_.borrow_mut()
    }
    pub fn bottom_controls_shown_ratio(&self) -> Ref<'_, SyncedBrowserControls> {
        self.bottom_controls_shown_ratio_.borrow()
    }
    pub fn bottom_controls_shown_ratio_mut(&mut self) -> RefMut<'_, SyncedBrowserControls> {
        self.bottom_controls_shown_ratio_.borrow_mut()
    }
    pub fn set_needs_update_draw_properties(&mut self) {
        self.needs_update_draw_properties_ = true;
    }
    pub fn clear_needs_update_draw_properties_for_testing(&mut self) {
        self.needs_update_draw_properties_ = false;
    }
    pub fn is_in_resourceless_software_draw_mode(&self, c: &LayerTreeContext<'_, B>) -> bool {
        c.host.IsResourcelessSoftwareDraw()
    }
    pub fn ForceRedrawNextActivation(&mut self) {
        self.next_activation_forces_redraw_ = true;
    }
    pub fn LayersThatShouldPushProperties(
        &self,
    ) -> <B::LayerList as LayerTreeLayerList<B>>::Iter<'_> {
        self.layer_list_.LayersThatShouldPushProperties()
    }
    pub fn picture_layers(&self) -> <B::LayerList as LayerTreeLayerList<B>>::PictureIter<'_> {
        self.layer_list_.PictureLayers()
    }
    pub fn picture_layers_with_paint_worklets(
        &self,
    ) -> <B::LayerList as LayerTreeLayerList<B>>::PictureIter<'_> {
        self.layer_list_.PictureLayersWithWorklets()
    }
    pub fn HandleVisibilityChanged(&self) -> bool {
        self.handle_visibility_changed_
    }
    pub fn browser_controls_params(&self) -> &BrowserControlsParams {
        &self.browser_controls_params_
    }
    pub fn browser_controls_shrink_blink_size(&self) -> bool {
        self.browser_controls_params_
            .browser_controls_shrink_blink_size
    }
    pub fn CurrentTopControlsShownRatio(&self, c: &LayerTreeContext<'_, B>) -> f32 {
        self.top_controls_shown_ratio_
            .borrow()
            .Current(self.IsActiveTree(c))
    }
    pub fn CurrentBottomControlsShownRatio(&self, c: &LayerTreeContext<'_, B>) -> f32 {
        self.bottom_controls_shown_ratio_
            .borrow()
            .Current(self.IsActiveTree(c))
    }
    pub fn top_controls_height(&self) -> f32 {
        self.browser_controls_params_.top_controls_height
    }
    pub fn top_controls_min_height(&self) -> f32 {
        self.browser_controls_params_.top_controls_min_height
    }
    pub fn bottom_controls_height(&self) -> f32 {
        self.browser_controls_params_.bottom_controls_height
    }
    pub fn bottom_controls_min_height(&self) -> f32 {
        self.browser_controls_params_.bottom_controls_min_height
    }
    pub fn only_expand_top_controls_at_page_top(&self) -> bool {
        self.browser_controls_params_
            .only_expand_top_controls_at_page_top
    }
    pub fn RequestForceSendMetadata(&mut self) {
        self.force_send_metadata_request_ = true;
    }
    pub fn event_listener_properties(&self, class: EventListenerClass) -> EventListenerProperties {
        self.event_listener_properties_[class as usize]
    }
    pub fn set_event_listener_properties(
        &mut self,
        class: EventListenerClass,
        value: EventListenerProperties,
    ) {
        self.event_listener_properties_[class as usize] = value;
    }
    pub fn lifecycle(&mut self) -> &mut LayerTreeLifecycle {
        &mut self.lifecycle_
    }
    pub fn paint_worklet_tracker<'a>(
        &self,
        c: &'a mut LayerTreeContext<'_, B>,
    ) -> &'a mut B::PaintWorkletTracker {
        c.host.paint_worklet_tracker()
    }
    pub fn DrawTransform<'a>(&self, c: &'a LayerTreeContext<'_, B>) -> &'a B::Transform {
        c.host.DrawTransform()
    }
    pub fn IsInSynchronousComposite(&self, c: &LayerTreeContext<'_, B>) -> bool {
        c.host.IsInSynchronousComposite()
    }
    pub fn GetActivelyScrollingType(
        &self,
        c: &LayerTreeContext<'_, B>,
    ) -> crate::cc::input::actively_scrolling_type::ActivelyScrollingType {
        c.host.GetActivelyScrollingType(self)
    }
    pub fn set_delegated_ink_metadata(&mut self, metadata: Option<Box<B::DelegatedInkMetadata>>) {
        self.delegated_ink_metadata_ = metadata;
    }
    pub fn delegated_ink_metadata(&self) -> Option<&B::DelegatedInkMetadata> {
        self.delegated_ink_metadata_.as_deref()
    }
    pub fn clear_delegated_ink_metadata(&mut self) {
        self.delegated_ink_metadata_ = None;
    }
    pub fn events_metrics_from_main_thread_count_for_testing(&self) -> usize {
        self.events_metrics_from_main_thread_.len()
    }
    pub fn events_metrics_from_raster_thread_count_for_testing(&self) -> usize {
        self.event_metrics_from_raster_thread_.len()
    }
    pub fn viewport_mobile_optimized(&self, c: &LayerTreeContext<'_, B>) -> bool {
        c.host.viewport_mobile_optimized()
    }
    pub fn set_is_animating_hud_contents(&mut self, value: bool, c: &LayerTreeContext<'_, B>) {
        debug_assert!(c.host.settings().trees_in_viz_in_viz_process());
        self.is_animating_hud_contents_ = value;
    }
    pub fn view_transition_requests(&self) -> &Vec<Box<B::ViewTransitionRequest>> {
        &self.view_transition_requests_
    }
    pub fn UpdateAllScrollbarGeometriesForTesting(&mut self, c: &mut LayerTreeContext<'_, B>) {
        self.UpdateAllScrollbarGeometries(c);
    }
    pub fn host_impl<'a>(
        &self,
        c: &'a mut LayerTreeContext<'_, B>,
    ) -> &'a mut dyn LayerTreeHost<B> {
        c.host
    }
}

impl<B: LayerTreeTypes> LayerTreeImpl<B> {
    // Source: cc/trees/layer_tree_impl.h:307-313,344-378.
    // Rust names distinguish the C++ const/non-const overloads; all return the
    // resident object and retain exactly the source invalid-id/null convention.
    pub fn hud_layer_mut(&mut self) -> Option<&mut B::Layer> {
        let id = self.hud_layer_?;
        self.layer_list_.LayerByIdMut(id)
    }
    pub fn OverscrollElasticityTransformNodeMutable(
        &mut self,
    ) -> Option<&mut TransformNode<B::Transform>> {
        let id = self.viewport_property_ids_.overscroll_elasticity_transform;
        if id == kInvalidPropertyNodeId {
            None
        } else {
            Some(
                self.property_trees_
                    .transform_tree_mutable()
                    .MutableNode(id),
            )
        }
    }
    pub fn PageScaleTransformNodeMutable(&mut self) -> Option<&mut TransformNode<B::Transform>> {
        let id = self.viewport_property_ids_.page_scale_transform;
        if id == kInvalidPropertyNodeId {
            None
        } else {
            Some(
                self.property_trees_
                    .transform_tree_mutable()
                    .MutableNode(id),
            )
        }
    }
    pub fn InnerViewportScrollNodeMutable(&mut self) -> Option<&mut ScrollNode> {
        let id = self.viewport_property_ids_.inner_scroll;
        if id == kInvalidPropertyNodeId {
            None
        } else {
            Some(self.property_trees_.scroll_tree_mutable().MutableNode(id))
        }
    }
    pub fn OuterViewportClipNodeMutable(&mut self) -> Option<&mut ClipNode> {
        let id = self.viewport_property_ids_.outer_clip;
        if id == kInvalidPropertyNodeId {
            None
        } else {
            Some(self.property_trees_.clip_tree_mutable().MutableNode(id))
        }
    }
    pub fn OuterViewportScrollNodeMutable(&mut self) -> Option<&mut ScrollNode> {
        let id = self.viewport_property_ids_.outer_scroll;
        if id == kInvalidPropertyNodeId {
            None
        } else {
            Some(self.property_trees_.scroll_tree_mutable().MutableNode(id))
        }
    }
}

impl<B: LayerTreeTypes> LayerTreeImpl<B> {
    // Source: cc/trees/layer_tree_impl.h:542; cc/trees/layer_tree_impl.cc:2020-2022.
    // Pure live borrow for the translated TreeSynchronizer's source mutations.
    pub fn LayerByIdMutable(&mut self, id: i32) -> Option<&mut B::Layer> {
        self.layer_list_.LayerByIdMut(id)
    }
}

impl<B: LayerTreeTypes> LayerTreeImpl<B> {
    // Source: cc/trees/layer_tree_impl.h:1034-1039; cc/input/scroll_elasticity_helper.cc:174-290.
    // Borrow-only sibling split. The effects live only inside this invocation.
    pub(crate) fn WithElasticityMutationParts<R>(
        &mut self,
        visit: impl FnOnce(
            &mut ScrollTree,
            super::property_tree::ScrollTreeMutationContext<'_, B::Transform>,
            &mut bool,
            &Rect,
        ) -> R,
    ) -> R {
        let (scroll, context) = self.property_trees_.scroll_tree_mutation_context();
        visit(
            scroll,
            context,
            &mut self.needs_update_draw_properties_,
            &self.device_viewport_rect_,
        )
    }
    // Source: cc/trees/layer_tree_impl.cc:1697-1702.
    pub(crate) fn GetDeviceViewportForRects(internal: Rect, external: Rect) -> Rect {
        if external.IsEmpty() {
            internal
        } else {
            external
        }
    }
}

impl<B: LayerTreeTypes> LayerTreeImpl<B> {
    /// The source vector range visits the same resident slots in the same
    /// direction. Only one identity scalar crosses a borrow boundary; no list
    /// of ids/state is materialized and no reference survives a callback.
    fn ForEachLayerWithContext(
        &mut self,
        reverse: bool,
        context: &mut LayerTreeContext<'_, B>,
        mut callback: impl FnMut(&mut B::Layer, &mut B::LayerContext<'_>),
    ) {
        let count = self.layer_list_.len();
        for position in 0..count {
            let index = if reverse {
                count - 1 - position
            } else {
                position
            };
            let id = self.layer_list_.LayerAt(index).id();
            B::WithLayerContext(self, context, id, |layer, layer_context| {
                callback(layer, layer_context)
            });
        }
    }
}

struct LayerTreeWholeScrollStore<'a, B: LayerTreeTypes>(&'a mut B::LayerList);
impl<B: LayerTreeTypes> LayerTreeScrollStore<B> for LayerTreeWholeScrollStore<'_, B> {
    fn WithScrollbarGeometry(
        &mut self,
        id: i32,
        callback: &mut dyn FnMut(&mut dyn LayerTreeScrollbarGeometry<B>, &mut BTreeSet<i32>),
    ) {
        self.0.WithScrollbarGeometry(id, callback)
    }
}
struct LayerTreeGeometryContext<'a, B: LayerTreeTypes, H: mutation::LayerTreeScrollHost<B> + ?Sized>
{
    _types: std::marker::PhantomData<fn() -> B>,
    id: LayerTreeId,
    host: &'a mut H,
    push: &'a mut BTreeSet<i32>,
    needs_draw: &'a mut bool,
    show_requests: &'a mut BTreeSet<ElementId>,
}
impl<B: LayerTreeTypes, H: mutation::LayerTreeScrollHost<B> + ?Sized>
    crate::adapter::layer_impl::LayerImplPushContext for LayerTreeGeometryContext<'_, B, H>
{
    type Types = B;
    fn id(&self) -> LayerTreeId {
        self.id
    }
    fn IsActiveTree(&self) -> bool {
        self.host.IsActiveTree(self.id)
    }
    fn TreesInVizInClientProcess(&self) -> bool {
        self.host.TreesInVizInClientProcess()
    }
    fn IsCCSlimmingEnabled(&self) -> bool {
        self.host.IsCCSlimmingEnabled()
    }
    fn AddLayerShouldPushProperties(&mut self, id: i32) {
        self.push.insert(id);
    }
    fn set_needs_update_draw_properties(&mut self) {
        *self.needs_draw = true;
    }
}
impl<B: LayerTreeTypes, H: mutation::LayerTreeScrollHost<B> + ?Sized>
    crate::adapter::scrollbar_layer_impl_base::ScrollbarGeometryStyle
    for LayerTreeGeometryContext<'_, B, H>
{
    fn ScrollbarAnimator(&self) -> ScrollbarAnimator {
        self.host.settings().scrollbar_animator()
    }
    fn IsFluentOverlayScrollbarEnabled(&self) -> bool {
        self.host.settings().enable_fluent_overlay_scrollbar()
    }
}
impl<B: LayerTreeTypes, H: mutation::LayerTreeScrollHost<B> + ?Sized>
    crate::adapter::scrollbar_layer_impl_base::ScrollbarGeometryContext<B>
    for LayerTreeGeometryContext<'_, B, H>
{
    fn RequestShowScrollbars(&mut self, id: ElementId) {
        RequestShowScrollbarsCore(self.id, id, self.host, self.show_requests);
    }
}
struct LayerTreeHostScrollbarStyle<'a, B: LayerTreeTypes>(&'a dyn LayerTreeHost<B>);
impl<B: LayerTreeTypes> crate::adapter::scrollbar_layer_impl_base::ScrollbarGeometryStyle
    for LayerTreeHostScrollbarStyle<'_, B>
{
    fn ScrollbarAnimator(&self) -> ScrollbarAnimator {
        self.0.settings().scrollbar_animator()
    }
    fn IsFluentOverlayScrollbarEnabled(&self) -> bool {
        self.0.settings().enable_fluent_overlay_scrollbar()
    }
}
// Source: cc/trees/layer_tree_impl.cc:1148-1152. Same source body shared by
// the resident-tree entry and the disjoint scroll callback fields.
fn RequestShowScrollbarsCore<B: LayerTreeTypes, H: mutation::LayerTreeScrollHost<B> + ?Sized>(
    tree: LayerTreeId,
    id: ElementId,
    host: &H,
    requests: &mut BTreeSet<ElementId>,
) {
    if host.IsActiveTree(tree) {
        requests.insert(id);
    }
}

/// Only the original resident fields needed while a layer owns its selected
/// vector slot. Every non-identity value remains a reference to its source field.
/// This view cannot change vector membership or cache a tree/Layer reference.
pub struct LayerTreeLayerFields<'a, B: LayerTreeTypes> {
    pub id: LayerTreeId,
    pub source_frame_number: &'a i32,
    pub(crate) discardable_image_map_updater: &'a mut Option<DiscardableImageMapUpdaterMarker>,
    pub property_trees: &'a mut PropertyTreesFor<B>,
    pub needs_update_draw_properties: &'a mut bool,
    pub lifecycle: &'a LayerTreeLifecycle,
    pub device_scale_factor: &'a f32,
    pub display_color_spaces: &'a B::DisplayColorSpaces,
    pub page_scale_factor: &'a Rc<RefCell<SyncedScale>>,
    pub external_page_scale_factor: &'a f32,
    pub device_viewport_rect: &'a Rect,
    pub device_viewport_rect_changed: &'a bool,
    pub viewport_property_ids: &'a ViewportPropertyIds,
    pub scrollbar_ids: &'a mut BTreeMap<ElementId, ScrollbarLayerIds>,
    pub show_scrollbar_requests: &'a mut BTreeSet<ElementId>,
}
impl<B: LayerTreeTypes> LayerTreeImpl<B> {
    /// Pure split of the resident layer owner from its siblings. No owner moves,
    /// map rebuilds, aggregate copies or source callbacks occur in this projection.
    pub fn SplitLayerTreeFields(&mut self) -> (&mut B::LayerList, LayerTreeLayerFields<'_, B>) {
        (
            &mut self.layer_list_,
            LayerTreeLayerFields {
                id: self.id,
                source_frame_number: &self.source_frame_number_,
                discardable_image_map_updater: &mut self.discardable_image_map_updater_,
                property_trees: &mut self.property_trees_,
                needs_update_draw_properties: &mut self.needs_update_draw_properties_,
                lifecycle: &self.lifecycle_,
                device_scale_factor: &self.device_scale_factor_,
                display_color_spaces: &self.display_color_spaces_,
                page_scale_factor: &self.page_scale_factor_,
                external_page_scale_factor: &self.external_page_scale_factor_,
                device_viewport_rect: &self.device_viewport_rect_,
                device_viewport_rect_changed: &self.device_viewport_rect_changed_,
                viewport_property_ids: &self.viewport_property_ids_,
                scrollbar_ids: &mut self.element_id_to_scrollbar_layer_ids_,
                show_scrollbar_requests: &mut self.show_scrollbar_requests_,
            },
        )
    }
}
impl<B: LayerTreeTypes> LayerTreeLayerFields<'_, B> {
    // Source: cc/trees/property_tree.cc:2371-2395. One source ScrollBy body,
    // shared by the ordinary tree entry and an explicit selected-layer callback.
    pub fn ScrollNodeByWithStore<H: mutation::LayerTreeScrollHost<B> + ?Sized>(
        &mut self,
        id: i32,
        scroll: &Vector2dF,
        host: &mut H,
        storage: &mut dyn LayerTreeScrollStore<B>,
    ) -> Vector2dF {
        let (tree, mut mutation) = self.property_trees.scroll_tree_mutation_context();
        let mut facade = LayerTreeScrollFacade {
            id: self.id,
            host,
            layer_list: storage,
            scrollbar_ids: self.scrollbar_ids,
            viewport: self.viewport_property_ids,
            page_scale: self.page_scale_factor,
            needs_update_draw_properties: self.needs_update_draw_properties,
            show_requests: self.show_scrollbar_requests,
        };
        tree.ScrollByNodeId(id, scroll, &mut facade, &mut mutation)
    }
    // Source: cc/trees/layer_tree_impl.cc:217-293. Same synchronous source
    // callback body; the selected id resolves only through its current receiver.
    pub fn DidUpdateScrollOffsetWithStore<H: mutation::LayerTreeScrollHost<B> + ?Sized>(
        &mut self,
        id: ElementId,
        pushed: bool,
        host: &mut H,
        storage: &mut dyn LayerTreeScrollStore<B>,
    ) {
        debug_assert!(self.lifecycle.AllowsPropertyTreeAccess());
        let (scroll_tree, mut mutation) = self.property_trees.scroll_tree_mutation_context();
        let mut facade = LayerTreeScrollFacade {
            id: self.id,
            host,
            layer_list: storage,
            scrollbar_ids: self.scrollbar_ids,
            viewport: self.viewport_property_ids,
            page_scale: self.page_scale_factor,
            needs_update_draw_properties: self.needs_update_draw_properties,
            show_requests: self.show_scrollbar_requests,
        };
        facade.DidUpdateScrollOffsetCore(id, pushed, scroll_tree, &mut mutation);
    }
}

// Source: cc/trees/layer_tree_impl.cc:3384-3400 (scoped updater lifecycle).
// The test caller supplies its actual current Picture as an action/lookup; no
// layer registry, snapshot or retained callback is introduced by this mapping.
pub(crate) fn WithScopedPictureImageMapUpdater<
    B: crate::adapter::picture_layer_impl::PictureLayerImplTypes,
>(
    tree: &mut dyn crate::adapter::picture_layer_impl::PictureLayerImplTree<B>,
    action: &mut dyn crate::adapter::picture_layer_impl::DiscardableImageMapUpdateAction<B>,
) {
    tree.BeginDiscardableImageMapUpdate();
    struct Guard<'a, B: crate::adapter::picture_layer_impl::PictureLayerImplTypes> {
        tree: &'a mut dyn crate::adapter::picture_layer_impl::PictureLayerImplTree<B>,
        action: &'a mut dyn crate::adapter::picture_layer_impl::DiscardableImageMapUpdateAction<B>,
        updates: Vec<i32>,
    }
    impl<B: crate::adapter::picture_layer_impl::PictureLayerImplTypes> Drop for Guard<'_, B> {
        fn drop(&mut self) {
            for id in &self.updates {
                self.action.Regenerate(*id, self.tree);
            }
            self.tree.EndDiscardableImageMapUpdate();
        }
    }
    let mut guard = Guard {
        tree,
        action,
        updates: Vec::new(),
    };
    guard.action.Run(guard.tree, &mut guard.updates);
}

impl<B: LayerTreeTypes> LayerTreeLayerFields<'_, B> {
    pub fn reborrow(&mut self) -> LayerTreeLayerFields<'_, B> {
        LayerTreeLayerFields {
            id: self.id,
            source_frame_number: self.source_frame_number,
            discardable_image_map_updater: &mut *self.discardable_image_map_updater,
            property_trees: &mut *self.property_trees,
            needs_update_draw_properties: &mut *self.needs_update_draw_properties,
            lifecycle: self.lifecycle,
            device_scale_factor: self.device_scale_factor,
            display_color_spaces: self.display_color_spaces,
            page_scale_factor: self.page_scale_factor,
            external_page_scale_factor: self.external_page_scale_factor,
            device_viewport_rect: self.device_viewport_rect,
            device_viewport_rect_changed: self.device_viewport_rect_changed,
            viewport_property_ids: self.viewport_property_ids,
            scrollbar_ids: &mut *self.scrollbar_ids,
            show_scrollbar_requests: &mut *self.show_scrollbar_requests,
        }
    }
    // Source: cc/trees/layer_tree_impl.cc:3384-3390.
    pub(crate) fn BeginDiscardableImageMapUpdate(&mut self, is_sync_tree: bool) {
        assert!(is_sync_tree);
        assert!(self.discardable_image_map_updater.is_none());
        *self.discardable_image_map_updater = Some(DiscardableImageMapUpdaterMarker);
    }
    // Source: cc/trees/layer_tree_impl.cc:3392-3400. End follows synchronous
    // regeneration performed by the same canonical scoped updater guard.
    pub(crate) fn EndDiscardableImageMapUpdate(&mut self) {
        *self.discardable_image_map_updater = None;
    }
}

impl<B: LayerTreeTypes> LayerTreeImpl<B> {
    /// Same resident lifecycle query used by a readonly native twin-layer call.
    pub(crate) fn LifecycleAllowsPropertyTreeAccess(&self) -> bool {
        self.lifecycle_.AllowsPropertyTreeAccess()
    }
}

impl<B: LayerTreeTypes> ScrollbarControllerTree for LayerTreeImpl<B> {
    fn InnerViewportScrollNode(&self) -> Option<&ScrollNode> {
        self.InnerViewportScrollNode()
    }
    fn OuterViewportScrollNode(&self) -> Option<&ScrollNode> {
        self.OuterViewportScrollNode()
    }
}

// Source: cc/trees/layer_tree_impl.cc:2445-2492. The source current scrollbar
// is explicit; conflicting orientation unregisters this same current pointer.
pub(crate) fn RegisterScrollbarWithContext<B: crate::adapter::layer_impl::LayerImplTypes>(
    scrollbar: &mut crate::cc::layers::scrollbar_layer_impl_base::ScrollbarLayerImplBase<B>,
    context: &mut dyn crate::adapter::scrollbar_layer_impl_base::ScrollbarRegistrationContext<B>,
) {
    let element = scrollbar.scroll_element_id();
    if element == ElementId::default() {
        return;
    }
    let orientation = scrollbar.orientation();
    let ids = context.RegistryMut().entry(element).or_default();
    let old = if orientation == ScrollbarOrientation::kHorizontal {
        ids.horizontal
    } else {
        ids.vertical
    };
    if old != -1 {
        UnregisterScrollbarWithContext(scrollbar, context);
    }
    let ids = context.RegistryMut().entry(element).or_default();
    if orientation == ScrollbarOrientation::kHorizontal {
        ids.horizontal = scrollbar.base().id();
    } else {
        ids.vertical = scrollbar.base().id();
    }
    if context.IsActiveTree() {
        context.DidRegisterScrollbarLayer(element, orientation);
    }
    if context.IsActiveTree()
        && scrollbar.is_overlay_scrollbar()
        && scrollbar.GetScrollbarAnimatorWithStyle(context) != ScrollbarAnimator::NO_ANIMATOR
    {
        if scrollbar.IsFluentOverlayScrollbarEnabledWithStyle(context) {
            scrollbar.SetOverlayScrollbarLayerOpacityAnimatedWithContext(0.0, false, context);
        }
        let opacity = scrollbar
            .base()
            .OpacityForPropertyTrees(context.PropertyTrees());
        context.RegisterScrollbarAnimationController(scrollbar, opacity);
    }
}
// Source: cc/trees/layer_tree_impl.cc:2494-2516.
pub(crate) fn UnregisterScrollbarWithContext<B: crate::adapter::layer_impl::LayerImplTypes>(
    scrollbar: &mut crate::cc::layers::scrollbar_layer_impl_base::ScrollbarLayerImplBase<B>,
    context: &mut dyn crate::adapter::scrollbar_layer_impl_base::ScrollbarRegistrationContext<B>,
) {
    let element = scrollbar.scroll_element_id();
    if element == ElementId::default() {
        return;
    }
    let orientation = scrollbar.orientation();
    let ids = context.RegistryMut().entry(element).or_default();
    if orientation == ScrollbarOrientation::kHorizontal {
        ids.horizontal = -1;
    } else {
        ids.vertical = -1;
    }
    if ids.horizontal == -1 && ids.vertical == -1 {
        context.RegistryMut().remove(&element);
    }
    if context.IsActiveTree() {
        context.DidUnregisterScrollbarLayer(scrollbar);
    }
}

/// Rust expression of native ScrollbarSet and its scoped current pointer.
/// The two ids come from the original registry; ordering uses the original
/// allocation identity just as native std::set<ScrollbarLayerImplBase*> does.
pub(crate) struct CurrentScrollbarInitializationClient<
    'a,
    B: crate::adapter::layer_impl::LayerImplTypes,
> {
    pub tree: LayerTreeId,
    pub current_identity: usize,
    pub current: &'a mut crate::cc::layers::scrollbar_layer_impl_base::ScrollbarLayerImplBase<B>,
    pub remaining: &'a mut dyn LayerTreeRemainingStorage<B>,
    pub registry: &'a BTreeMap<ElementId, ScrollbarLayerIds>,
    pub geometry:
        &'a mut dyn crate::adapter::scrollbar_layer_impl_base::ScrollbarGeometryContext<B>,
}
impl<B: crate::adapter::layer_impl::LayerImplTypes> CurrentScrollbarInitializationClient<'_, B> {
    fn WithScrollbar<Output>(
        &mut self,
        id: &crate::adapter::input_handler::InputLayerId,
        callback: impl FnOnce(&mut dyn LayerTreeScrollbarGeometry<B>) -> Output,
    ) -> Output {
        assert_eq!(
            id.tree, self.tree,
            "source scrollbar belongs to current tree"
        );
        if id.id == self.current.base().id() {
            callback(self.current)
        } else {
            let mut callback = Some(callback);
            let mut output = None;
            self.remaining
                .WithScrollbarGeometry(id.id, &mut |scrollbar| {
                    output = Some(callback.take().expect("one source pointer resolution")(
                        scrollbar,
                    ));
                });
            output.expect("registered scrollbar exists")
        }
    }
}
impl<B: crate::adapter::layer_impl::LayerImplTypes>
    crate::adapter::scrollbar::ScrollbarInitializationClient
    for CurrentScrollbarInitializationClient<'_, B>
{
    // Source: cc/trees/layer_tree_impl.cc:2518-2535.
    fn ScrollbarsFor(
        &mut self,
        element: ElementId,
    ) -> Vec<crate::adapter::input_handler::InputLayerId> {
        let mut result = ScrollbarSet::new();
        if let Some(ids) = self.registry.get(&element) {
            for id in [ids.horizontal, ids.vertical] {
                if id != -1 {
                    let identity = if id == self.current.base().id() {
                        Some(self.current_identity)
                    } else {
                        self.remaining.LayerObjectIdentity(id)
                    };
                    if let Some(object_identity) = identity {
                        result.insert(ScrollbarLayerHandle {
                            object_identity,
                            layer_id: id,
                        });
                    }
                }
            }
        }
        result
            .into_iter()
            .map(|layer| crate::adapter::input_handler::InputLayerId {
                tree: self.tree,
                id: layer.layer_id,
            })
            .collect()
    }
    fn Scrollbar_orientation(
        &mut self,
        id: &crate::adapter::input_handler::InputLayerId,
    ) -> ScrollbarOrientation {
        self.WithScrollbar(id, |layer| layer.orientation())
    }
    fn Scrollbar_is_overlay_scrollbar(
        &mut self,
        id: &crate::adapter::input_handler::InputLayerId,
    ) -> bool {
        assert_eq!(id.tree, self.tree);
        if id.id == self.current.base().id() {
            self.current.is_overlay_scrollbar()
        } else {
            // The geometry projection includes the native overlay flag; no alternate
            // subtype owner or cached registration metadata is needed.
            self.WithScrollbar(id, |layer| layer.is_overlay_scrollbar())
        }
    }
    fn Scrollbar_thumb_thickness_scale_factor(
        &mut self,
        id: &crate::adapter::input_handler::InputLayerId,
    ) -> f32 {
        self.WithScrollbar(id, |layer| layer.thumb_thickness_scale_factor())
    }
    fn Scrollbar_SetThumbThicknessScaleFactor(
        &mut self,
        id: &crate::adapter::input_handler::InputLayerId,
        value: f32,
    ) {
        assert_eq!(id.tree, self.tree);
        if id.id == self.current.base().id() {
            let (base, mut fields) = self.current.SplitGeometryFields();
            fields.SetThumbThicknessScaleFactor(base, value, self.geometry);
        } else {
            let geometry = &mut *self.geometry;
            self.remaining.WithScrollbarGeometry(id.id, &mut |layer| {
                layer.SetThumbThicknessScaleFactor(value, geometry)
            });
        }
    }
}
