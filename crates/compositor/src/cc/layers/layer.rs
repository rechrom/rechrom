// Translation status: complete
// Copyright 2010 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/layers/layer.h:1-62; cc/layers/layer.cc:1-46.
// Includes, export guards, forward declarations and namespaces map to modules.
use crate::{
    adapter::{
        geometry::{LinearGradient, Point3F, PointF, Rect, Size, Vector2dF},
        input_types::ElementId,
        layer_tree_impl::Color4f,
        main_layer::*,
        occlusion::SkBlendMode,
        property_types::SubtreeCaptureId,
    },
    cc::{
        base::{protected_sequence_synchronizer::*, region::Region},
        input::hit_test_opaqueness::HitTestOpaqueness,
        layers::{scroll_hit_test_rect::ScrollHitTestRect, touch_action_region::TouchActionRegion},
        trees::property_ids::kInvalidPropertyNodeId,
    },
};
use std::sync::{
    atomic::{AtomicU8, AtomicUsize, Ordering},
    Arc, RwLock, RwLockReadGuard, Weak,
};

// Source: cc/layers/layer.h:63-80; cc/layers/layer.cc:84-86.
#[derive(Clone)]
pub struct LayerDebugInfo<N: Copy + Default> {
    pub name: String,
    pub owner_node_id: N,
    pub paint_count: i32,
    pub compositing_reasons: Vec<&'static str>,
    pub compositing_reason_ids: Vec<&'static str>,
    pub invalidations: Vec<Invalidation>,
}
// Source: cc/layers/layer.h:73-78.
#[derive(Clone)]
pub struct Invalidation {
    pub rect: Rect,
    pub reason: &'static str,
    pub client: String,
}
impl<N: Copy + Default> Default for LayerDebugInfo<N> {
    fn default() -> Self {
        Self {
            name: String::new(),
            owner_node_id: N::default(),
            paint_count: 0,
            compositing_reasons: Vec::new(),
            compositing_reason_ids: Vec::new(),
            invalidations: Vec::new(),
        }
    }
}
// Source: cc/layers/layer.h:1044-1064; cc/layers/layer.cc:88-89.
struct RareInputs<B: MainLayerTypes> {
    capture_bounds: B::RegionCaptureBounds,
    tracked_element_rects: B::TrackedElementRects,
    main_thread_scroll_hit_test_region: Region<B::RegionBackend>,
    non_composited_scroll_hit_test_rects: Vec<ScrollHitTestRect>,
    wheel_event_region: Region<B::RegionBackend>,
    #[cfg(target_os = "android")]
    xr_hit_test_order: Vec<ElementId>,
    canvas_child_id: ElementId,
    filter_quality: B::FilterQuality,
    dynamic_range_limit: B::DynamicRangeLimit,
}
impl<B: MainLayerTypes> Default for RareInputs<B> {
    fn default() -> Self {
        Self {
            capture_bounds: Default::default(),
            tracked_element_rects: Default::default(),
            main_thread_scroll_hit_test_region: Default::default(),
            non_composited_scroll_hit_test_rects: Vec::new(),
            wheel_event_region: Default::default(),
            #[cfg(target_os = "android")]
            xr_hit_test_order: Vec::new(),
            canvas_child_id: Default::default(),
            filter_quality: Default::default(),
            dynamic_range_limit: Default::default(),
        }
    }
}
// Source: cc/layers/layer.h:1073-1098; cc/layers/layer.cc:91-92.
struct Inputs<B: MainLayerTypes> {
    children: Vec<LayerRef<B>>,
    bounds: Size,
    hit_test_opaqueness: HitTestOpaqueness,
    contents_opaque: bool,
    contents_opaque_for_text: bool,
    is_drawable: bool,
    double_sided: bool,
    background_color: Color4f,
    touch_action_region: TouchActionRegion<B::RegionBackend>,
    element_id: ElementId,
    rare_inputs: Option<Box<RareInputs<B>>>,
}
impl<B: MainLayerTypes> Default for Inputs<B> {
    fn default() -> Self {
        Self {
            children: Vec::new(),
            bounds: Default::default(),
            hit_test_opaqueness: HitTestOpaqueness::kTransparent,
            contents_opaque: false,
            contents_opaque_for_text: false,
            is_drawable: false,
            double_sided: true,
            background_color: Color4f::default(),
            touch_action_region: Default::default(),
            element_id: Default::default(),
            rare_inputs: None,
        }
    }
}
// Source: cc/layers/layer.h:1100-1167; cc/layers/layer.cc:94-95.
struct LayerTreeInputs<B: MainLayerTypes> {
    clip_rect: Rect,
    mask_layer: Option<Weak<dyn LayerObject<B>>>,
    opacity: f32,
    blend_mode: SkBlendMode,
    masks_to_bounds: bool,
    is_fast_rounded_corner: bool,
    trilinear_filtering: bool,
    hide_layer_and_subtree: bool,
    scrollable: bool,
    position: PointF,
    transform: B::Transform,
    transform_origin: Point3F,
    subtree_capture_id: SubtreeCaptureId,
    safe_opaque_background_color: Color4f,
    filters: B::Filters,
    backdrop_filters: B::Filters,
    backdrop_filter_bounds: Option<B::Path>,
    backdrop_filter_quality: f32,
    mirror_count: i32,
    scroll_offset: PointF,
    scroll_container_bounds: Size,
    corner_radii: RoundedCornersF,
    gradient_mask: LinearGradient,
    did_scroll_callback: B::DidScrollCallback,
    copy_requests: Vec<Option<Box<B::CopyOutputRequest>>>,
}
impl<B: MainLayerTypes> Default for LayerTreeInputs<B> {
    fn default() -> Self {
        Self {
            clip_rect: Default::default(),
            mask_layer: None,
            opacity: 1.,
            blend_mode: SkBlendMode::kSrcOver,
            masks_to_bounds: false,
            is_fast_rounded_corner: false,
            trilinear_filtering: false,
            hide_layer_and_subtree: false,
            scrollable: false,
            position: Default::default(),
            transform: Default::default(),
            transform_origin: Default::default(),
            subtree_capture_id: Default::default(),
            safe_opaque_background_color: Default::default(),
            filters: Default::default(),
            backdrop_filters: Default::default(),
            backdrop_filter_bounds: None,
            backdrop_filter_quality: 1.,
            mirror_count: 0,
            scroll_offset: Default::default(),
            scroll_container_bounds: Default::default(),
            corner_radii: Default::default(),
            gradient_mask: Default::default(),
            did_scroll_callback: Default::default(),
            copy_requests: Vec::new(),
        }
    }
}

// Source: cc/layers/layer.h:82-95,1176-1234.
// Arc is the source's strong per-Layer ownership; Weak is the parent backlink.
// Every protected field has its own canonical guard, never a whole-tree lock.
pub struct Layer<B: MainLayerTypes> {
    // Native base::RefCounted lifecycle, distinct from temporary Rust pointer
    // liveness. 0=live, 1=derived destruction, 2=base destruction, 3=dead.
    native_owners_: AtomicUsize,
    native_lifecycle_: AtomicU8,
    parent_: ProtectedSequenceReadable<Option<Weak<dyn LayerObject<B>>>>,
    layer_tree_host_: RwLock<Option<MainLayerHostId>>,
    layer_tree_inputs_: ProtectedSequenceReadableBox<LayerTreeInputs<B>>,
    debug_info_: ProtectedSequenceWritableBox<LayerDebugInfo<B::NodeId>>,
    inputs_: ProtectedSequenceReadable<Inputs<B>>,
    update_rect_: RwLock<Rect>,
    layer_id_: i32,
    num_descendants_that_draw_content_: ProtectedSequenceReadable<i32>,
    transform_tree_index_: ProtectedSequenceReadable<i32>,
    effect_tree_index_: ProtectedSequenceReadable<i32>,
    clip_tree_index_: ProtectedSequenceReadable<i32>,
    scroll_tree_index_: ProtectedSequenceReadable<i32>,
    property_tree_sequence_number_: ProtectedSequenceReadable<i32>,
    offset_to_transform_parent_: ProtectedSequenceReadable<Vector2dF>,
    ignore_set_needs_commit_for_test_: RwLock<bool>,
    subtree_property_changed_: ProtectedSequenceWritable<bool>,
    #[cfg(debug_assertions)]
    allow_remove_for_readd_: RwLock<bool>,
    bitflags_: ProtectedSequenceReadable<u8>,
    changed_properties_: ProtectedSequenceWritable<u8>,
}
// Source dependency: base/memory/scoped_refptr.h. This strong handle owns only
// this resident Layer, and never stores a host/tree/context capability.
// Source: cc/layers/layer.h virtual interface. Derived canonical layer owners
// compose this resident base and override their actual native virtual methods.
// This Rust vtable carries no host/service/backpointer or cloned base state.
pub trait LayerObject<B: MainLayerTypes>: Send + Sync {
    fn base(&self) -> &Layer<B>;
    fn AsScrollbarLayer(
        &self,
    ) -> Option<&dyn super::scrollbar_layer_base::ScrollbarLayerObject<B>> {
        None
    }
    fn AsPictureLayer(&self) -> Option<&dyn PictureLayerObject<B>> {
        None
    }
    fn HasDrawableContent(&self, c: &LayerContext<'_, B>) -> bool {
        self.base().HasDrawableContent(c)
    }
    // Rust destructor dispatch for the native most-derived destructor chain.
    // Derived owners release their own members and invoke the canonical base.
    fn NativeShutdown(&self, c: &mut LayerContext<'_, B>) {
        self.base().Shutdown(c);
    }
    // Source: cc/layers/layer.h:795. Virtual dispatch only; canonical base
    // lifecycle remains in Layer::SetLayerTreeHost.
    fn SetLayerTreeHost(
        &self,
        allocation: &LayerBorrow<B>,
        host: Option<MainLayerHostId>,
        c: &mut LayerContext<'_, B>,
    ) {
        self.base().SetLayerTreeHost(allocation, host, c);
    }
    // Source: cc/layers/layer.h:182,270,290,342,488,726-727,733,735,746,751-752,766,770,775,782,855,908,918-920,934.
    // Native virtual slots; each default calls the single canonical base body.
    fn SetBackgroundColor(&self, value: Color4f, c: &mut LayerContext<'_, B>) {
        self.base().SetBackgroundColor(value, c);
    }
    fn SetOpacity(&self, value: f32, c: &mut LayerContext<'_, B>) {
        self.base().SetOpacity(value, c);
    }
    fn SetNeedsDisplayRect(&self, rect: &Rect, c: &mut LayerContext<'_, B>) {
        self.base().SetNeedsDisplayRect(self, rect, c);
    }
    fn RequiresSetNeedsDisplayOnHdrHeadroomChange(&self, _c: &LayerContext<'_, B>) -> bool {
        self.base().RequiresSetNeedsDisplayOnHdrHeadroomChange()
    }
    fn IsScrollbarLayerForTesting(&self) -> bool {
        self.base().IsScrollbarLayerForTesting()
    }
    fn IsSolidColorLayerForTesting(&self) -> bool {
        self.base().IsSolidColorLayerForTesting()
    }
    fn OpacityCanAnimateOnImplThread(&self) -> bool {
        self.base().OpacityCanAnimateOnImplThread()
    }
    fn CaptureContent(
        &self,
        rect: &Rect,
        content: &mut Vec<B::NodeInfo>,
        _c: &mut LayerContext<'_, B>,
    ) {
        self.base().CaptureContent(*rect, content);
    }
    fn GetPicture(&self, _c: &mut LayerContext<'_, B>) -> Option<Arc<B::Picture>> {
        self.base().GetPicture()
    }
    fn RunMicroBenchmark(&self, benchmark: &mut B::MicroBenchmark, _c: &mut LayerContext<'_, B>) {
        self.base().RunMicroBenchmark(benchmark);
    }
    fn CreateLayerImpl(
        &self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        tree_context: &mut crate::cc::trees::layer_tree_impl::LayerTreeContext<'_, B>,
        _c: &mut LayerContext<'_, B>,
    ) -> Box<B::Layer> {
        self.base().CreateLayerImpl(tree, tree_context)
    }
    fn PushDirtyPropertiesTo(
        &self,
        target: &mut B::Layer,
        dirty: u8,
        commit: &mut crate::cc::trees::commit_state::CommitState<B>,
        c: &mut LayerContext<'_, B>,
        impl_tree: &mut B::ImplContext<'_>,
    ) {
        self.base()
            .PushDirtyPropertiesTo(target, dirty, commit, c, impl_tree);
    }
    fn Update(&self, c: &mut LayerContext<'_, B>) -> bool {
        self.base().Update(c)
    }
    fn MayUpdateAfterPaintEvent(&self) -> bool {
        self.base().MayUpdateAfterPaintEvent()
    }
    fn UpdateAfterPaintEvent(&self, c: &mut LayerContext<'_, B>) -> bool {
        self.base().UpdateAfterPaintEvent(c)
    }
    fn IsSnappedToPixelGridInTarget(&self) -> bool {
        self.base().IsSnappedToPixelGridInTarget()
    }
    fn ViewTransitionResourceId(
        &self,
    ) -> crate::adapter::property_types::ViewTransitionElementResourceId {
        self.base().ViewTransitionResourceId()
    }
}
impl<B: MainLayerTypes> LayerObject<B> for Layer<B> {
    fn base(&self) -> &Layer<B> {
        self
    }
}
pub struct LayerRef<B: MainLayerTypes>(Arc<dyn LayerObject<B>>);
impl<B: MainLayerTypes> Clone for LayerRef<B> {
    fn clone(&self) -> Self {
        self.0
            .base()
            .native_owners_
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                if count == 0 {
                    None
                } else {
                    count.checked_add(1)
                }
            })
            .expect("cannot clone a destroyed Layer or overflow its native owner count");
        Self(Arc::clone(&self.0))
    }
}
impl<B: MainLayerTypes> std::ops::Deref for LayerRef<B> {
    type Target = Layer<B>;
    fn deref(&self) -> &Layer<B> {
        assert_eq!(
            self.0.base().native_lifecycle_.load(Ordering::Acquire),
            0,
            "native Layer owner points to a destroyed object"
        );
        self.0.base()
    }
}
impl<B: MainLayerTypes> PartialEq for LayerRef<B> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl<B: MainLayerTypes> Eq for LayerRef<B> {}
impl<B: MainLayerTypes> Layer<B> {
    // Source: cc/layers/layer.h:93-96.
    pub const INVALID_ID: i32 = -1;
    // Source: cc/layers/layer.h:959-963.
    pub const kChangedPropertyTreeIndex: u8 = 1 << 0;
    pub const kChangedGeneralProperty: u8 = 1 << 1;
    pub const kChangedAllProperties: u8 =
        Self::kChangedPropertyTreeIndex | Self::kChangedGeneralProperty;
    // Source: cc/layers/layer.h:1224-1232.
    const kDrawsContentFlagMask: u8 = 1 << 0;
    const kShouldCheckBackfaceVisibilityFlagMask: u8 = 1 << 1;
    const kCacheRenderSurfaceFlagMask: u8 = 1 << 2;
    const kForceRenderSurfaceForTestingFlagMask: u8 = 1 << 3;
    const kHasTransformNodeFlagMask: u8 = 1 << 5;
    const kHasClipNodeFlagMask: u8 = 1 << 6;
    const kSubtreeHasCopyRequestFlagMask: u8 = 1 << 7;
    // Source: cc/layers/layer.cc:97-100. Atomic generation is supplied by the owner.
    pub fn GetNextLayerId(services: &dyn MainLayerServices) -> i32 {
        services.NextLayerIdSequenceValue() + 1
    }
    // Source: cc/layers/layer.cc:102-104.
    pub fn Create(services: &dyn MainLayerServices) -> LayerRef<B> {
        LayerRef(Arc::new(Self::new(services)))
    }
    // Source: cc/layers/layer.cc:106-119.
    pub(crate) fn new(services: &dyn MainLayerServices) -> Self {
        Self {
            native_owners_: AtomicUsize::new(1),
            native_lifecycle_: AtomicU8::new(0),
            parent_: ProtectedSequenceReadable::new(None),
            layer_tree_host_: RwLock::new(None),
            layer_tree_inputs_: ProtectedSequenceReadableBox::new(None),
            debug_info_: ProtectedSequenceWritableBox::new(None),
            inputs_: ProtectedSequenceReadable::new(Inputs::default()),
            update_rect_: RwLock::new(Rect::default()),
            layer_id_: Self::GetNextLayerId(services),
            num_descendants_that_draw_content_: ProtectedSequenceReadable::new(0),
            transform_tree_index_: ProtectedSequenceReadable::new(kInvalidPropertyNodeId),
            effect_tree_index_: ProtectedSequenceReadable::new(kInvalidPropertyNodeId),
            clip_tree_index_: ProtectedSequenceReadable::new(kInvalidPropertyNodeId),
            scroll_tree_index_: ProtectedSequenceReadable::new(kInvalidPropertyNodeId),
            property_tree_sequence_number_: ProtectedSequenceReadable::new(-1),
            offset_to_transform_parent_: ProtectedSequenceReadable::new(Default::default()),
            ignore_set_needs_commit_for_test_: RwLock::new(false),
            subtree_property_changed_: ProtectedSequenceWritable::new(false),
            #[cfg(debug_assertions)]
            allow_remove_for_readd_: RwLock::new(false),
            bitflags_: ProtectedSequenceReadable::new(0),
            changed_properties_: ProtectedSequenceWritable::new(0),
        }
    }
    // Source: cc/layers/layer.h:113-113.
    pub fn id(&self) -> i32 {
        self.layer_id_
    }
    // No host backlink: only the native weak host identity is resident.
    pub(crate) fn host_id(&self) -> Option<MainLayerHostId> {
        *self
            .layer_tree_host_
            .read()
            .unwrap_or_else(|error| error.into_inner())
    }
    fn sync<'a, 'c>(&self, c: &'a LayerContext<'c, B>) -> LayerSynchronization<'a, 'c, B> {
        LayerSynchronization {
            host: self.host_id(),
            context: c,
        }
    }
    // Source: cc/layers/layer.h:150-150.
    pub fn IsAttached(&self) -> bool {
        self.host_id().is_some()
    }
}

// Read-only synchronization view of the same current host capability. It keeps
// no copied flags and cannot outlive the calling method's context borrow.
struct LayerSynchronization<'a, 'c, B: MainLayerTypes> {
    host: Option<MainLayerHostId>,
    context: &'a LayerContext<'c, B>,
}
impl<B: MainLayerTypes> ProtectedSequenceSynchronizer for LayerSynchronization<'_, '_, B> {
    // Source: cc/layers/layer.cc:1662-1664.
    fn IsOwnerThread(&self) -> bool {
        self.host
            .is_none_or(|id| self.context.Host(id).IsOwnerThread())
    }
    // Source: cc/layers/layer.cc:1670-1672.
    fn InProtectedSequence(&self) -> bool {
        self.host
            .is_some_and(|id| self.context.Host(id).InProtectedSequence())
    }
    // Source: cc/layers/layer.cc:1674-1677.
    fn WaitForProtectedSequenceCompletion(&self) {
        if let Some(id) = self.host {
            self.context.Host(id).WaitForProtectedSequenceCompletion();
        }
    }
}

impl<B: MainLayerTypes> Layer<B> {
    // Source: cc/layers/layer.cc:133-139.
    fn EnsureLayerTreeInputsAccess(
        &self,
        c: &LayerContext<'_, B>,
    ) -> WriteAccess<'_, Option<Box<LayerTreeInputs<B>>>> {
        debug_assert!(!self.IsAttached() || !self.IsUsingLayerLists(c));
        let inputs = self.layer_tree_inputs_.WriteAccess(&self.sync(c));
        {
            let mut value = inputs.Write();
            if value.is_none() {
                *value = Some(Box::new(LayerTreeInputs::default()));
            }
        }
        inputs
    }
    // Guard expression of the same source T&; allocation/control flow exists
    // only in EnsureLayerTreeInputsAccess, never duplicated here.
    fn EnsureLayerTreeInputs(
        &self,
        c: &LayerContext<'_, B>,
    ) -> LiveFieldWrite<
        std::sync::RwLockWriteGuard<'_, Option<Box<LayerTreeInputs<B>>>>,
        LayerTreeInputs<B>,
    > {
        LiveFieldWrite::new(
            self.EnsureLayerTreeInputsAccess(c).Write(),
            |value| value.as_deref().unwrap(),
            |value| value.as_deref_mut().unwrap(),
        )
    }
    // Source: cc/layers/layer.cc:141-146; cc/layers/layer.h:974-980.
    fn layer_tree_inputs(
        &self,
        c: &LayerContext<'_, B>,
    ) -> Option<
        LiveFieldRead<RwLockReadGuard<'_, Option<Box<LayerTreeInputs<B>>>>, LayerTreeInputs<B>>,
    > {
        #[cfg(debug_assertions)]
        debug_assert!(!self.IsAttached() || !self.IsUsingLayerLists(c));
        let value = self.layer_tree_inputs_.Read(&self.sync(c));
        if value.is_some() {
            Some(LiveFieldRead::new(value, |value| value.as_deref().unwrap()))
        } else {
            None
        }
    }
    // Source: cc/layers/layer.cc:148-199.
    fn SetLayerTreeHost(
        &self,
        allocation: &LayerBorrow<B>,
        host: Option<MainLayerHostId>,
        c: &mut LayerContext<'_, B>,
    ) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        if self.host_id() == host {
            return;
        }
        let mut property_tree_indices_invalid = false;
        let element_id = self.inputs_.Write(&self.sync(c)).element_id;
        if let Some(old) = self.host_id() {
            debug_assert!(self.sync(c).IsOwnerThread());
            c.Host(old).WaitForProtectedSequenceCompletion();
            c.HostMut(old).UnregisterLayer(self);
            if element_id.IsValid() {
                c.HostMut(old).UnregisterElement(element_id, self);
            }
            if !self.IsUsingLayerLists(c) {
                c.HostMut(old).property_trees_mut().set_needs_rebuild(true);
                property_tree_indices_invalid = true;
            }
        }
        if let Some(host) = host {
            c.HostMut(host).RegisterLayer(self, allocation.clone());
            if element_id.IsValid() {
                c.HostMut(host).RegisterElement(element_id, self);
            }
            if !c.Host(host).IsUsingLayerLists() {
                c.HostMut(host).property_trees_mut().set_needs_rebuild(true);
                property_tree_indices_invalid = true;
            }
        }
        *self
            .layer_tree_host_
            .write()
            .unwrap_or_else(|error| error.into_inner()) = host;
        if let Some(host) = host {
            *self.changed_properties_.Write(&self.sync(c)) |= Self::kChangedAllProperties;
            c.HostMut(host).AddLayerShouldPushProperties(self);
        }
        if property_tree_indices_invalid {
            self.InvalidatePropertyTreesIndices(c);
        }
        // Native `auto child` copies one scoped_refptr per iteration, not a list.
        let count = self.inputs_.Read(&self.sync(c)).children.len();
        for index in 0..count {
            let child = self.inputs_.Read(&self.sync(c)).children[index].clone();
            child.0.SetLayerTreeHost(&child.borrow(), host, c);
        }
        if let Some(host) = host {
            if !c.Host(host).IsUsingLayerLists() && c.Host(host).IsElementAnimating(element_id) {
                c.HostMut(host).SetNeedsCommit();
            }
        }
    }
    // Source: cc/layers/layer.cc:201-212.
    pub fn SetNeedsCommit(&self, c: &mut LayerContext<'_, B>) {
        let Some(host) = self.host_id() else {
            return;
        };
        self.SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
        if *self
            .ignore_set_needs_commit_for_test_
            .read()
            .unwrap_or_else(|error| error.into_inner())
        {
            return;
        }
        c.HostMut(host).SetNeedsCommit();
    }
    // Source: cc/layers/layer.cc:214-218.
    pub fn SetDebugName(&self, name: String, c: &mut LayerContext<'_, B>) {
        if !name.is_empty() || self.debug_info_.Read(&self.sync(c)).is_some() {
            self.EnsureDebugInfo(c).name = name;
        }
    }
    // Source: cc/layers/layer.cc:220-222.
    pub fn ViewTransitionResourceId(
        &self,
    ) -> crate::adapter::property_types::ViewTransitionElementResourceId {
        Default::default()
    }
    // Source: cc/layers/layer.cc:224-226.
    pub fn IsSolidColorLayerForTesting(&self) -> bool {
        false
    }
    // Source: cc/layers/layer.cc:228-233.
    fn SetNeedsFullTreeSync(&self, c: &mut LayerContext<'_, B>) {
        if let Some(host) = self.host_id() {
            c.HostMut(host).SetNeedsFullTreeSync();
        }
    }
    // Source: cc/layers/layer.cc:235-246.
    pub fn SetNeedsPushProperties(&self, mut changed_props: u8, c: &mut LayerContext<'_, B>) {
        let changed = self.changed_properties_.WriteAccess(&self.sync(c));
        if !c.Services().IsCCSlimmingEnabled() {
            changed_props = Self::kChangedAllProperties;
        }
        if (*changed.Read() & changed_props) != changed_props {
            if *changed.Read() == 0 {
                if let Some(host) = self.host_id() {
                    c.HostMut(host).AddLayerShouldPushProperties(self);
                }
            }
            *changed.Write() |= changed_props;
        }
    }

    // Source: cc/layers/layer.cc:248-254.
    pub fn IsPropertyChangeAllowed(&self, c: &LayerContext<'_, B>) -> bool {
        let Some(host) = self.host_id() else {
            return true;
        };
        debug_assert!(self.IsMainThread(c));
        !c.Host(host).in_paint_layer_contents()
    }
    // Source: cc/layers/layer.cc:256-257.
    pub fn CaptureContent(&self, _rect: Rect, _content: &mut Vec<B::NodeInfo>) {}
    // Source: cc/layers/layer.cc:259-261.
    pub fn GetPicture(&self) -> Option<Arc<B::Picture>> {
        None
    }
    // Source: cc/layers/layer.cc:1666-1668.
    pub fn IsMainThread(&self, c: &LayerContext<'_, B>) -> bool {
        self.host_id().is_some_and(|id| c.Host(id).IsMainThread())
    }
    // Source: cc/layers/layer.cc:1679-1681.
    pub fn IsUsingLayerLists(&self, c: &LayerContext<'_, B>) -> bool {
        self.host_id()
            .is_some_and(|id| c.Host(id).IsUsingLayerLists())
    }
}
impl<B: MainLayerTypes> Layer<B> {
    // Source: cc/layers/layer.cc:342-344.
    fn GetBitFlag(&self, mask: u8, c: &LayerContext<'_, B>) -> bool {
        *self.bitflags_.Read(&self.sync(c)) & mask != 0
    }
    // Source: cc/layers/layer.cc:346-363.
    fn SetBitFlag(
        &self,
        value: bool,
        mask: u8,
        invalidate: bool,
        needs_push: bool,
        c: &mut LayerContext<'_, B>,
    ) -> bool {
        if self.GetBitFlag(mask, c) == value {
            return false;
        }
        if value {
            *self.bitflags_.Write(&self.sync(c)) |= mask;
        } else {
            *self.bitflags_.Write(&self.sync(c)) &= !mask;
        }
        if invalidate {
            self.SetPropertyTreesNeedRebuild(c);
            self.SetNeedsCommit(c);
        }
        if needs_push {
            self.SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
        }
        true
    }
    // Source: cc/layers/layer.h:689-691.
    pub fn SetHasTransformNode(&self, value: bool, c: &mut LayerContext<'_, B>) {
        self.SetBitFlag(value, Self::kHasTransformNodeFlagMask, false, false, c);
    }
    // Source: cc/layers/layer.cc:1283-1290.
    pub fn SetTransformTreeIndex(&self, index: i32, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        if *self.transform_tree_index_.Read(&self.sync(c)) == index {
            return;
        }
        self.SetHasTransformNode(index != kInvalidPropertyNodeId, c);
        *self.transform_tree_index_.Write(&self.sync(c)) = index;
        self.SetNeedsPushProperties(Self::kChangedPropertyTreeIndex, c);
    }
    // Source: cc/layers/layer.cc:1313-1319.
    pub fn SetClipTreeIndex(&self, index: i32, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        if *self.clip_tree_index_.Read(&self.sync(c)) == index {
            return;
        }
        *self.clip_tree_index_.Write(&self.sync(c)) = index;
        self.SetNeedsPushProperties(Self::kChangedPropertyTreeIndex, c);
    }
    // Source: cc/layers/layer.cc:1342-1348.
    pub fn SetEffectTreeIndex(&self, index: i32, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        if *self.effect_tree_index_.Read(&self.sync(c)) == index {
            return;
        }
        *self.effect_tree_index_.Write(&self.sync(c)) = index;
        self.SetNeedsPushProperties(Self::kChangedPropertyTreeIndex, c);
    }
    // Source: cc/layers/layer.cc:1371-1377.
    pub fn SetScrollTreeIndex(&self, index: i32, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        if *self.scroll_tree_index_.Read(&self.sync(c)) == index {
            return;
        }
        *self.scroll_tree_index_.Write(&self.sync(c)) = index;
        self.SetNeedsPushProperties(Self::kChangedPropertyTreeIndex, c);
    }
    // Source: cc/layers/layer.cc:1408-1413.
    fn InvalidatePropertyTreesIndices(&self, c: &mut LayerContext<'_, B>) {
        self.SetTransformTreeIndex(kInvalidPropertyNodeId, c);
        self.SetClipTreeIndex(kInvalidPropertyNodeId, c);
        self.SetEffectTreeIndex(kInvalidPropertyNodeId, c);
        self.SetScrollTreeIndex(kInvalidPropertyNodeId, c);
    }
    // Source: cc/layers/layer.cc:1415-1418.
    fn SetPropertyTreesNeedRebuild(&self, c: &mut LayerContext<'_, B>) {
        if let Some(host) = self.host_id() {
            c.HostMut(host).property_trees_mut().set_needs_rebuild(true);
        }
    }
    // Source: cc/layers/layer.cc:1420-1430.
    pub fn EnsureDebugInfo(
        &self,
        c: &mut LayerContext<'_, B>,
    ) -> LiveFieldWrite<
        std::sync::RwLockWriteGuard<'_, Option<Box<LayerDebugInfo<B::NodeId>>>>,
        LayerDebugInfo<B::NodeId>,
    > {
        let info = self.debug_info_.WriteAccess(&self.sync(c));
        if info.Read().is_none() {
            *info.Write() = Some(Box::new(LayerDebugInfo::default()));
            self.SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
        }
        LiveFieldWrite::new(
            info.Write(),
            |value| value.as_deref().unwrap(),
            |value| value.as_deref_mut().unwrap(),
        )
    }
}

// A raw/Weak source pointer is an allocation borrow, not a scoped_refptr owner.
// It cannot revive a source object after its last native owner was released.
pub struct LayerBorrow<B: MainLayerTypes>(Arc<dyn LayerObject<B>>);
impl<B: MainLayerTypes> LayerBorrow<B> {
    /// Call-scoped raw allocation dispatch; acquiring this borrow does not add
    /// a native scoped_refptr owner or discover a containing most-derived type.
    pub fn SetLayerTreeHost(&self, host: Option<MainLayerHostId>, c: &mut LayerContext<'_, B>) {
        self.object(c).SetLayerTreeHost(self, host, c);
    }
    pub fn id(&self) -> i32 {
        self.0.base().id()
    }
    pub fn Update(&self, c: &mut LayerContext<'_, B>) -> bool {
        self.object(c).Update(c)
    }
    pub fn MatchesBase(&self, base: &Layer<B>) -> bool {
        std::ptr::eq(self.0.base(), base)
    }
    pub fn IsAlive(&self) -> bool {
        self.0.base().native_lifecycle_.load(Ordering::Acquire) == 0
            && self.0.base().native_owners_.load(Ordering::Acquire) > 0
    }
    fn object(&self, c: &LayerContext<'_, B>) -> &dyn LayerObject<B> {
        let base = self.0.base();
        let state = base.native_lifecycle_.load(Ordering::Acquire);
        assert!(
            self.IsAlive()
                || ((state == 1 || state == 2) && c.native_destruction_layer == Some(base.id())),
            "raw Layer borrow used after native Shutdown"
        );
        self.0.as_ref()
    }
    fn layer(&self, c: &LayerContext<'_, B>) -> &Layer<B> {
        self.object(c).base()
    }
}
impl<B: MainLayerTypes> LayerRef<B> {
    pub fn borrow(&self) -> LayerBorrow<B> {
        LayerBorrow(Arc::clone(&self.0))
    }
    fn weak(&self) -> Weak<dyn LayerObject<B>> {
        Arc::downgrade(&self.0)
    }
}
impl<B: MainLayerTypes> Drop for LayerRef<B> {
    fn drop(&mut self) {
        let base = self.0.base();
        let previous = base
            .native_owners_
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                count.checked_sub(1)
            })
            .expect("native Layer owner-count underflow");
        if previous == 1 {
            base.native_lifecycle_
                .compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire)
                .expect("native Layer Shutdown executed twice");
            let mut context = LayerContext::DetachedForDestruction(base.id());
            self.0.NativeShutdown(&mut context);
            assert_eq!(
                base.native_lifecycle_.load(Ordering::Acquire),
                2,
                "most-derived shutdown must execute its unique native base destructor"
            );
            base.native_lifecycle_.store(3, Ordering::Release);
        }
    }
}
// Source: cc/layers/layer.h:995-998.
#[derive(Clone, Copy, PartialEq, Eq)]
enum RemovalReason {
    kNormal,
    kForReadd,
}
impl<B: MainLayerTypes> Layer<B> {
    // Source: cc/layers/layer.h:120-120. Raw parent queries never add native owners.
    pub fn parent(&self, c: &LayerContext<'_, B>) -> Option<LayerBorrow<B>> {
        self.parent_
            .Read(&self.sync(c))
            .as_ref()
            .and_then(Weak::upgrade)
            .map(LayerBorrow)
    }
    // Source: cc/layers/layer.h:654-654.
    pub fn draws_content(&self, c: &LayerContext<'_, B>) -> bool {
        self.GetBitFlag(Self::kDrawsContentFlagMask, c)
    }
    // Source: cc/layers/layer.cc:1637-1639.
    pub fn NumDescendantsThatDrawContent(&self, c: &LayerContext<'_, B>) -> i32 {
        *self.num_descendants_that_draw_content_.Read(&self.sync(c))
    }
    // Source: cc/layers/layer.cc:1655-1660.
    pub fn SetSubtreePropertyChanged(&self, c: &mut LayerContext<'_, B>) {
        if *self.subtree_property_changed_.Read(&self.sync(c)) {
            return;
        }
        *self.subtree_property_changed_.Write(&self.sync(c)) = true;
        self.SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
    }
    // Source: cc/layers/layer.cc:1742-1751.
    fn AddDrawableDescendants(&self, num: i32, c: &mut LayerContext<'_, B>) {
        debug_assert!(*self.num_descendants_that_draw_content_.Read(&self.sync(c)) >= 0);
        debug_assert!(*self.num_descendants_that_draw_content_.Read(&self.sync(c)) + num >= 0);
        if num == 0 {
            return;
        }
        *self.num_descendants_that_draw_content_.Write(&self.sync(c)) += num;
        self.SetNeedsCommit(c);
        if let Some(parent) = self.parent(c) {
            parent.layer(c).AddDrawableDescendants(num, c);
        }
    }
    // Source: cc/layers/layer.cc:263-282.
    fn SetParent(
        &self,
        receiver: &LayerBorrow<B>,
        layer: Option<&LayerRef<B>>,
        reason: RemovalReason,
        c: &mut LayerContext<'_, B>,
    ) {
        debug_assert!(layer.is_none_or(|layer| !layer.HasAncestor(self, c)));
        debug_assert!(reason == RemovalReason::kNormal || layer.is_none());
        *self.parent_.Write(&self.sync(c)) = layer.map(LayerRef::weak);
        if reason == RemovalReason::kForReadd {
            #[cfg(debug_assertions)]
            debug_assert!(*self.allow_remove_for_readd_.read().unwrap());
        } else {
            receiver.object(c).SetLayerTreeHost(
                receiver,
                layer.and_then(|parent| parent.host_id()),
                c,
            );
        }
        self.SetPropertyTreesNeedRebuild(c);
    }
    // Source: cc/layers/layer.cc:284-286.
    pub fn AddChild(
        &self,
        receiver: &LayerRef<B>,
        child: LayerRef<B>,
        c: &mut LayerContext<'_, B>,
    ) {
        let len = self.inputs_.Read(&self.sync(c)).children.len();
        self.InsertChild(receiver, child.clone(), len, c);
    }
    // Source: cc/layers/layer.cc:288-308.
    pub fn InsertChild(
        &self,
        receiver: &LayerRef<B>,
        child: LayerRef<B>,
        mut index: usize,
        c: &mut LayerContext<'_, B>,
    ) {
        debug_assert!(std::ptr::eq(self, receiver.0.base()));
        debug_assert!(self.IsPropertyChangeAllowed(c));
        let allow = AllowRemoveForReadd::new(&child, c);
        child.RemoveFromParentForReadd(c);
        self.AddDrawableDescendants(
            child.NumDescendantsThatDrawContent(c) + i32::from(child.draws_content(c)),
            c,
        );
        child.SetParent(Some(receiver), RemovalReason::kNormal, c);
        child.SetSubtreePropertyChanged(c);
        let mut inputs = self.inputs_.Write(&self.sync(c));
        index = index.min(inputs.children.len());
        let tree_inputs = self.layer_tree_inputs_.Read(&self.sync(c));
        if tree_inputs
            .as_ref()
            .is_some_and(|value| value.mask_layer.is_some())
            && index != 0
            && index == inputs.children.len()
        {
            debug_assert!(Weak::ptr_eq(
                tree_inputs.as_ref().unwrap().mask_layer.as_ref().unwrap(),
                &inputs.children.last().unwrap().weak()
            ));
            index -= 1;
        }
        inputs.children.insert(index, child.clone());
        drop(tree_inputs);
        drop(inputs);
        self.SetNeedsFullTreeSync(c);
        allow.Finish(c);
    }
    // Source: cc/layers/layer.cc:310-314.
    pub fn RemoveFromParent(&self, receiver: &LayerBorrow<B>, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        if self.parent_.Read(&self.sync(c)).is_some() {
            let parent = self
                .parent_
                .Write(&self.sync(c))
                .as_ref()
                .and_then(Weak::upgrade)
                .map(LayerBorrow)
                .unwrap();
            parent
                .layer(c)
                .RemoveChild(receiver, RemovalReason::kNormal, c);
        }
    }
    // Source: cc/layers/layer.cc:316-320.
    fn RemoveFromParentForReadd(&self, receiver: &LayerBorrow<B>, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        if self.parent_.Read(&self.sync(c)).is_some() {
            let parent = self
                .parent_
                .Write(&self.sync(c))
                .as_ref()
                .and_then(Weak::upgrade)
                .map(LayerBorrow)
                .unwrap();
            parent
                .layer(c)
                .RemoveChild(receiver, RemovalReason::kForReadd, c);
        }
    }
    // Source: cc/layers/layer.cc:322-340.
    fn RemoveChild(
        &self,
        child_object: &LayerBorrow<B>,
        reason: RemovalReason,
        c: &mut LayerContext<'_, B>,
    ) {
        let child = child_object.layer(c);
        let has_mask = self
            .layer_tree_inputs_
            .Read(&self.sync(c))
            .as_ref()
            .and_then(|value| value.mask_layer.as_ref())
            .and_then(Weak::upgrade)
            .is_some_and(|mask| std::ptr::eq(mask.base(), child));
        if has_mask {
            self.layer_tree_inputs_
                .Write(&self.sync(c))
                .as_mut()
                .unwrap()
                .mask_layer = None;
        }
        let inputs = self.inputs_.WriteAccess(&self.sync(c));
        // Rust borrow split: the source inputs reference is reentrant. End
        // the count query's read guard before the loop may erase a child from
        // this same resident vector; a for-range temporary otherwise lives
        // through the entire loop and blocks its synchronous write.
        let child_count = inputs.Read().children.len();
        for index in 0..child_count {
            if !std::ptr::eq(inputs.Read().children[index].0.base(), child) {
                continue;
            }
            child.SetParent(child_object, None, reason, c);
            self.AddDrawableDescendants(
                -child.NumDescendantsThatDrawContent(c) - i32::from(child.draws_content(c)),
                c,
            );
            let removed = inputs.Write().children.remove(index);
            drop(removed);
            self.SetNeedsFullTreeSync(c);
            return;
        }
    }
    // Source: cc/layers/layer.cc:365-385.
    pub fn ReorderChildren(
        &self,
        new_children_order: &mut Vec<LayerRef<B>>,
        c: &mut LayerContext<'_, B>,
    ) {
        let mut inputs = self.inputs_.Write(&self.sync(c));
        #[cfg(debug_assertions)]
        {
            let mut children_set = std::collections::BTreeSet::new();
            for child in new_children_order.iter() {
                debug_assert!(child
                    .parent(c)
                    .is_some_and(|parent| std::ptr::eq(parent.layer(c), self)));
                children_set.insert(child.id());
            }
            for child in &inputs.children {
                debug_assert!(children_set.contains(&child.id()));
            }
        }
        inputs.children = std::mem::take(new_children_order);
        for child in &inputs.children {
            *child.subtree_property_changed_.Write(&child.sync(c)) = true;
        }
        drop(inputs);
        self.SetNeedsFullTreeSync(c);
    }
    // Source: cc/layers/layer.cc:387-406.
    pub fn ReplaceChild(
        &self,
        receiver: &LayerRef<B>,
        reference_object: &LayerBorrow<B>,
        new_layer: Option<LayerRef<B>>,
        c: &mut LayerContext<'_, B>,
    ) {
        let reference = reference_object.layer(c);
        debug_assert!(reference
            .parent(c)
            .is_some_and(|parent| std::ptr::eq(parent.layer(c), self)));
        debug_assert!(self.IsPropertyChangeAllowed(c));
        if new_layer
            .as_ref()
            .is_some_and(|value| std::ptr::eq(value.0.base(), reference))
        {
            return;
        }
        let reference_index = self
            .inputs_
            .Write(&self.sync(c))
            .children
            .iter()
            .position(|value| std::ptr::eq(value.0.base(), reference))
            .expect("native reference child not found");
        reference.RemoveFromParent(reference_object, c);
        if let Some(layer) = new_layer {
            self.InsertChild(receiver, layer, reference_index, c);
        }
    }
}
// Source: cc/layers/layer.h:1236-1270. The guard keeps only the source raw
// layer pointer's allocation borrow, without changing native ownership.
struct AllowRemoveForReadd<'a, B: MainLayerTypes> {
    layer: &'a Layer<B>,
    finished: bool,
}
impl<'a, B: MainLayerTypes> AllowRemoveForReadd<'a, B> {
    fn new(layer: &'a Layer<B>, _c: &LayerContext<'_, B>) -> Self {
        #[cfg(debug_assertions)]
        {
            let mut allow = layer.allow_remove_for_readd_.write().unwrap();
            debug_assert!(!*allow);
            *allow = true;
        }
        Self {
            layer,
            finished: false,
        }
    }
    // The lexical source destructor runs before its current per-call context
    // expires; no host/context is stored in the guard or resident Layer.
    fn Finish(mut self, c: &LayerContext<'_, B>) {
        #[cfg(debug_assertions)]
        {
            debug_assert!(self.layer.parent(c).is_some());
            let mut allow = self.layer.allow_remove_for_readd_.write().unwrap();
            debug_assert!(*allow);
            *allow = false;
        }
        self.finished = true;
    }
}
impl<B: MainLayerTypes> Drop for AllowRemoveForReadd<'_, B> {
    fn drop(&mut self) {
        debug_assert!(
            self.finished || std::thread::panicking(),
            "native readd scope must finish before its live context expires"
        );
    }
}
impl<B: MainLayerTypes> Layer<B> {
    // Source: cc/layers/layer.cc:121-131. Native destructor executes synchronously
    // at the last LayerRef release while raw Rust borrows may still keep allocation.
    pub(crate) fn Shutdown(&self, c: &mut LayerContext<'_, B>) {
        assert_eq!(
            c.native_destruction_layer,
            Some(self.id()),
            "native destructor requires its scoped allocation identity"
        );
        assert_eq!(
            self.native_owners_.load(Ordering::Acquire),
            0,
            "native destructor requires the final owner release"
        );
        self.native_lifecycle_
            .compare_exchange(1, 2, Ordering::AcqRel, Ordering::Acquire)
            .expect("native Layer base destructor executed twice");
        debug_assert!(self.parent(c).is_none());
        debug_assert!(!self.IsAttached());
        self.RemoveAllChildren(c);
        // Source defaulted member destruction after cc:130, in reverse declaration
        // order (layer.h:1196,1194,1190-1191). Only owned payload members have effects;
        // all other fields are scalar/weak lifecycle metadata. These moves release
        // source values now even when a raw Rust allocation borrow still exists.
        drop(std::mem::replace(
            &mut *self.inputs_.Write(&self.sync(c)),
            Inputs::default(),
        ));
        drop(self.debug_info_.Write(&self.sync(c)).take());
        drop(self.layer_tree_inputs_.Write(&self.sync(c)).take());
    }
    // Source: cc/layers/layer.cc:446-451.
    pub fn RootLayer(&self, receiver: &LayerRef<B>, c: &LayerContext<'_, B>) -> LayerBorrow<B> {
        debug_assert!(std::ptr::eq(self, receiver.0.base()));
        let mut layer = receiver.borrow();
        while let Some(parent) = layer.layer(c).parent(c) {
            layer = parent;
        }
        layer
    }
    // Source: cc/layers/layer.cc:453-461.
    pub fn RemoveAllChildren(&self, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        while !self.inputs_.Read(&self.sync(c)).children.is_empty() {
            let layer = self.inputs_.Read(&self.sync(c)).children[0].borrow();
            debug_assert!(layer
                .layer(c)
                .parent(c)
                .is_some_and(|parent| std::ptr::eq(parent.layer(c), self)));
            layer.RemoveFromParent(c);
        }
    }
    // Source: cc/layers/layer.cc:463-513.
    pub fn SetChildLayerList(
        &self,
        receiver: &LayerRef<B>,
        new_children: Vec<LayerRef<B>>,
        c: &mut LayerContext<'_, B>,
    ) {
        debug_assert!(self.IsUsingLayerLists(c));
        if self.inputs_.Read(&self.sync(c)).children == new_children {
            return;
        }
        {
            // Source unordered_set holds raw pointers: map ids only to allocation borrows
            // and never create additional native scoped_refptr owners.
            let mut children_to_remove = std::collections::HashMap::new();
            for child in &self.inputs_.Read(&self.sync(c)).children {
                children_to_remove.insert(child.id(), child.borrow());
            }
            for child in &new_children {
                children_to_remove.remove(&child.id());
            }
            for child in children_to_remove.values() {
                child
                    .layer(c)
                    .SetParent(child, None, RemovalReason::kNormal, c);
                self.AddDrawableDescendants(
                    -child.layer(c).NumDescendantsThatDrawContent(c)
                        - i32::from(child.layer(c).draws_content(c)),
                    c,
                );
            }
        }
        let mut existing_child_index = 0;
        for child in &new_children {
            if child
                .parent(c)
                .is_some_and(|parent| std::ptr::eq(parent.layer(c), self))
            {
                let children = self.inputs_.Read(&self.sync(c));
                while existing_child_index < children.children.len()
                    && children.children[existing_child_index] != *child
                {
                    existing_child_index += 1;
                }
                if existing_child_index == children.children.len() {
                    child.SetSubtreePropertyChanged(c);
                }
            }
        }
        for child in &new_children {
            if !child
                .parent(c)
                .is_some_and(|parent| std::ptr::eq(parent.layer(c), self))
            {
                let allow = AllowRemoveForReadd::new(child, c);
                child.RemoveFromParentForReadd(c);
                self.AddDrawableDescendants(
                    child.NumDescendantsThatDrawContent(c) + i32::from(child.draws_content(c)),
                    c,
                );
                child.SetParent(Some(receiver), RemovalReason::kNormal, c);
                child.SetSubtreePropertyChanged(c);
                allow.Finish(c);
            }
        }
        let old = std::mem::replace(
            &mut self.inputs_.Write(&self.sync(c)).children,
            new_children,
        );
        drop(old);
        c.HostMut(self.host_id().unwrap()).SetNeedsFullTreeSync();
    }
    // Source: cc/layers/layer.cc:515-521.
    pub fn HasAncestor(&self, ancestor: &Layer<B>, c: &LayerContext<'_, B>) -> bool {
        let mut layer = self.parent(c);
        while let Some(current) = layer {
            if std::ptr::eq(current.layer(c), ancestor) {
                return true;
            }
            layer = current.layer(c).parent(c);
        }
        false
    }
}
impl<B: MainLayerTypes> Layer<B> {
    // Source: cc/layers/layer.h:230-230.
    pub fn bounds(&self, c: &LayerContext<'_, B>) -> impl std::ops::Deref<Target = Size> + '_ {
        LiveFieldRead::new(self.inputs_.Read(&self.sync(c)), |inputs| &inputs.bounds)
    }
    // Source: cc/layers/layer.h:236-238.
    pub fn masks_to_bounds(&self, c: &LayerContext<'_, B>) -> bool {
        self.layer_tree_inputs(c)
            .is_some_and(|inputs| inputs.masks_to_bounds)
    }
    // Source: cc/layers/layer.h:258-266. Both native const/mutable raw accessors
    // resolve one resident allocation borrow; they never create a native owner.
    pub fn mask_layer(&self, c: &LayerContext<'_, B>) -> Option<LayerBorrow<B>> {
        self.layer_tree_inputs(c).and_then(|inputs| {
            inputs
                .mask_layer
                .as_ref()
                .and_then(Weak::upgrade)
                .map(LayerBorrow)
        })
    }
    // Source: cc/layers/layer.h:480-482.
    pub fn scrollable(&self, c: &LayerContext<'_, B>) -> bool {
        self.layer_tree_inputs(c)
            .is_some_and(|inputs| inputs.scrollable)
    }
    // Source: cc/layers/layer.cc:408-444.
    pub fn SetBounds(&self, size: Size, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        if *self.bounds(c) == size {
            return;
        }
        self.inputs_.Write(&self.sync(c)).bounds = size;
        let Some(host) = self.host_id() else {
            return;
        };
        if !self.IsUsingLayerLists(c) {
            if self.subtree_capture_id(c).is_valid()
                || self.masks_to_bounds(c)
                || self.mask_layer(c).is_some()
                || self.HasMaskFilter(c)
            {
                self.SetSubtreePropertyChanged(c);
                self.SetPropertyTreesNeedRebuild(c);
            }
            if self.scrollable(c) {
                let index = *self.scroll_tree_index_.Read(&self.sync(c));
                if index != kInvalidPropertyNodeId {
                    c.HostMut(host)
                        .property_trees_mut()
                        .scroll_tree_mutable()
                        .MutableNode(index)
                        .bounds = *self.bounds(c);
                } else {
                    self.SetPropertyTreesNeedRebuild(c);
                }
            }
        }
        self.SetNeedsCommit(c);
    }
    // Source: cc/layers/layer.cc:523-543.
    pub fn RequestCopyOfOutput(
        &self,
        request: Box<B::CopyOutputRequest>,
        c: &mut LayerContext<'_, B>,
    ) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        let inputs = self.EnsureLayerTreeInputsAccess(c);
        if request.has_source() {
            let source = request.source();
            if let Some(index) = inputs
                .Read()
                .as_ref()
                .unwrap()
                .copy_requests
                .iter()
                .position(|value| {
                    value
                        .as_ref()
                        .is_some_and(|value| value.has_source() && value.source() == source)
                })
            {
                let removed = inputs.Write().as_mut().unwrap().copy_requests.remove(index);
                drop(removed);
            }
        }
        inputs
            .Write()
            .as_mut()
            .unwrap()
            .copy_requests
            .push(Some(request));
        self.SetSubtreePropertyChanged(c);
        self.SetPropertyTreesNeedRebuild(c);
        self.SetNeedsCommit(c);
        if let Some(host) = self.host_id() {
            c.HostMut(host).SetHasCopyRequest(true);
        }
    }
}
// Each entry preserves the actual most-derived receiver in the lexical base
// operation; these wrappers do not create another native owning reference.
impl<B: MainLayerTypes> LayerRef<B> {
    pub fn SetLayerTreeHost(&self, host: Option<MainLayerHostId>, c: &mut LayerContext<'_, B>) {
        self.0.SetLayerTreeHost(&self.borrow(), host, c);
    }
    fn SetParent(
        &self,
        parent: Option<&LayerRef<B>>,
        reason: RemovalReason,
        c: &mut LayerContext<'_, B>,
    ) {
        self.0.base().SetParent(&self.borrow(), parent, reason, c);
    }
    pub fn RemoveFromParent(&self, c: &mut LayerContext<'_, B>) {
        self.0.base().RemoveFromParent(&self.borrow(), c);
    }
    fn RemoveFromParentForReadd(&self, c: &mut LayerContext<'_, B>) {
        self.0.base().RemoveFromParentForReadd(&self.borrow(), c);
    }
    pub fn AddChild(&self, child: LayerRef<B>, c: &mut LayerContext<'_, B>) {
        self.0.base().AddChild(self, child, c);
    }
    pub fn InsertChild(&self, child: LayerRef<B>, index: usize, c: &mut LayerContext<'_, B>) {
        self.0.base().InsertChild(self, child, index, c);
    }
    pub fn ReplaceChild(
        &self,
        reference: &LayerRef<B>,
        new_layer: Option<LayerRef<B>>,
        c: &mut LayerContext<'_, B>,
    ) {
        self.0
            .base()
            .ReplaceChild(self, &reference.borrow(), new_layer, c);
    }
    pub fn SetChildLayerList(&self, children: Vec<LayerRef<B>>, c: &mut LayerContext<'_, B>) {
        self.0.base().SetChildLayerList(self, children, c);
    }
    pub fn RootLayer(&self, c: &LayerContext<'_, B>) -> LayerBorrow<B> {
        self.0.base().RootLayer(self, c)
    }
}
impl<B: MainLayerTypes> LayerBorrow<B> {
    fn RemoveFromParent(&self, c: &mut LayerContext<'_, B>) {
        self.object(c).base().RemoveFromParent(self, c);
    }
}

impl<B: MainLayerTypes> Layer<B> {
    // Source: cc/layers/layer.h:183-185.
    pub fn background_color(&self, c: &LayerContext<'_, B>) -> Color4f {
        self.inputs_.Read(&self.sync(c)).background_color
    }
    // Source: cc/layers/layer.h:409-409.
    pub fn contents_opaque(&self, c: &LayerContext<'_, B>) -> bool {
        self.inputs_.Read(&self.sync(c)).contents_opaque
    }
    // Source: cc/layers/layer.h:245-247.
    pub fn clip_rect(&self, c: &LayerContext<'_, B>) -> Rect {
        self.layer_tree_inputs(c)
            .map_or_else(Rect::default, |inputs| inputs.clip_rect)
    }
    // Source: cc/layers/layer.h:830-832.
    pub fn offset_to_transform_parent(&self, c: &LayerContext<'_, B>) -> Vector2dF {
        *self.offset_to_transform_parent_.Read(&self.sync(c))
    }
    // Source: cc/layers/layer.cc:545-553.
    pub fn SetBackgroundColor(&self, color: Color4f, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        let mut inputs = self.inputs_.Write(&self.sync(c));
        if inputs.background_color == color {
            return;
        }
        inputs.background_color = color;
        drop(inputs);
        self.SetPropertyTreesNeedRebuild(c);
        self.SetNeedsCommit(c);
    }
    // Source: cc/layers/layer.cc:555-563.
    pub fn SetSafeOpaqueBackgroundColor(&self, color: Color4f, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        let mut opaque = color;
        opaque.alpha = 1.;
        let mut inputs = self.EnsureLayerTreeInputs(c);
        if inputs.safe_opaque_background_color == opaque {
            return;
        }
        inputs.safe_opaque_background_color = opaque;
        drop(inputs);
        self.SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
    }
    // Source: cc/layers/layer.cc:565-587.
    pub fn SafeOpaqueBackgroundColor(&self, c: &LayerContext<'_, B>) -> Color4f {
        if self.contents_opaque(c) {
            if !self.IsUsingLayerLists(c) {
                let inputs = self.layer_tree_inputs(c).expect("source layer-tree inputs");
                debug_assert!(inputs.safe_opaque_background_color.alpha == 1.);
                return inputs.safe_opaque_background_color;
            }
            let mut color = self.background_color(c);
            color.alpha = 1.;
            return color;
        }
        if self.background_color(c).alpha == 1. {
            return Color4f::default();
        }
        self.background_color(c)
    }
    // Source: cc/layers/layer.cc:589-598.
    pub fn SetMasksToBounds(&self, value: bool, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        let mut inputs = self.EnsureLayerTreeInputs(c);
        if inputs.masks_to_bounds == value {
            return;
        }
        inputs.masks_to_bounds = value;
        drop(inputs);
        self.SetNeedsCommit(c);
        self.SetPropertyTreesNeedRebuild(c);
        self.SetSubtreePropertyChanged(c);
    }
    // Source: cc/layers/layer.cc:600-634.
    pub fn SetClipRect(&self, rect: Rect, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        let inputs = self.EnsureLayerTreeInputsAccess(c);
        {
            let mut value = inputs.Write();
            let value = value.as_mut().unwrap();
            if value.clip_rect == rect {
                return;
            }
            value.clip_rect = rect;
        }
        let force_rebuild = rect.IsEmpty() || !self.has_clip_node(c);
        self.SetSubtreePropertyChanged(c);
        if self.clip_tree_index(c) != kInvalidPropertyNodeId && !force_rebuild {
            let effective = self.EffectiveClipRect(c);
            let clip_id = self.clip_tree_index(c);
            if clip_id != kInvalidPropertyNodeId {
                let offset = self.offset_to_transform_parent(c);
                let trees = c.HostMut(self.host_id().unwrap()).property_trees_mut();
                let node = trees.clip_tree_mutable().MutableNode(clip_id);
                node.clip = effective;
                node.clip.Offset(offset.x(), offset.y());
                trees.clip_tree_mutable().set_needs_update(true);
            }
            let effect_id = self.effect_tree_index(c);
            if self.HasMaskFilter(c) && effect_id != kInvalidPropertyNodeId {
                let inputs = inputs.Read();
                let inputs = inputs.as_ref().unwrap();
                let mask =
                    MaskFilterInfoForLayer(effective, inputs.corner_radii, &inputs.gradient_mask);
                let trees = c.HostMut(self.host_id().unwrap()).property_trees_mut();
                let node = trees.effect_tree_mutable().MutableNode(effect_id);
                node.mask_filter_info = mask;
                node.effect_changed = true;
                trees.effect_tree_mutable().set_needs_update(true);
            }
        } else {
            self.SetPropertyTreesNeedRebuild(c);
        }
        self.SetNeedsCommit(c);
    }
    // Source: cc/layers/layer.cc:636-652.
    pub fn EffectiveClipRect(&self, c: &LayerContext<'_, B>) -> crate::adapter::geometry::RectF {
        use crate::adapter::draw_property::DrawFilters;
        let size = *self.bounds(c);
        let bounds = crate::adapter::geometry::RectF::FromSize(
            crate::adapter::geometry::SizeF::new(size.width() as f32, size.height() as f32),
        );
        let clip = self.clip_rect(c);
        if clip.IsEmpty() {
            return bounds;
        }
        let clip = crate::adapter::geometry::RectF::new(
            clip.x as f32,
            clip.y as f32,
            clip.width as f32,
            clip.height as f32,
        );
        if self.masks_to_bounds(c)
            || self.mask_layer(c).is_some()
            || self.filters(c).HasFilterThatMovesPixels()
        {
            let mut value = bounds;
            value.Intersect(clip);
            return value;
        }
        clip
    }
    // Source: cc/layers/layer.cc:681-690.
    pub fn SetFilters(&self, filters: &B::Filters, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        let mut inputs = self.EnsureLayerTreeInputs(c);
        if inputs.filters == *filters {
            return;
        }
        inputs.filters = filters.clone();
        drop(inputs);
        self.SetSubtreePropertyChanged(c);
        self.SetPropertyTreesNeedRebuild(c);
        self.SetNeedsCommit(c);
    }
    // Source: cc/layers/layer.cc:692-702.
    pub fn SetBackdropFilters(&self, filters: &B::Filters, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        let mut inputs = self.EnsureLayerTreeInputs(c);
        if inputs.backdrop_filters == *filters {
            return;
        }
        inputs.backdrop_filters = filters.clone();
        drop(inputs);
        self.SetSubtreePropertyChanged(c);
        self.SetPropertyTreesNeedRebuild(c);
        self.SetNeedsCommit(c);
    }
    // Source: cc/layers/layer.cc:704-706.
    pub fn SetBackdropFilterBounds(&self, bounds: &B::Path, c: &LayerContext<'_, B>) {
        self.EnsureLayerTreeInputs(c).backdrop_filter_bounds = Some(bounds.clone());
    }
    // Source: cc/layers/layer.cc:708-711.
    pub fn ClearBackdropFilterBounds(&self, c: &LayerContext<'_, B>) {
        if self.layer_tree_inputs(c).is_some() {
            self.layer_tree_inputs_
                .Write(&self.sync(c))
                .as_mut()
                .unwrap()
                .backdrop_filter_bounds = None;
        }
    }
    // Source: cc/layers/layer.cc:713-715.
    pub fn SetBackdropFilterQuality(&self, quality: f32, c: &LayerContext<'_, B>) {
        self.EnsureLayerTreeInputs(c).backdrop_filter_quality = quality;
    }
    // Source: cc/layers/layer.cc:717-745.
    fn UpdateMaskFilterInfo(
        &self,
        corners: Option<RoundedCornersF>,
        gradient: Option<&LinearGradient>,
        c: &mut LayerContext<'_, B>,
    ) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        let inputs = self.EnsureLayerTreeInputsAccess(c);
        if let Some(corners) = corners {
            inputs.Write().as_mut().unwrap().corner_radii = corners;
        }
        if let Some(gradient) = gradient {
            inputs.Write().as_mut().unwrap().gradient_mask = gradient.clone();
        }
        self.SetSubtreePropertyChanged(c);
        self.SetNeedsCommit(c);
        let effect_id = self.effect_tree_index(c);
        if self.IsAttached() && effect_id != kInvalidPropertyNodeId {
            let mut effective = self.EffectiveClipRect(c);
            let offset = self.offset_to_transform_parent(c);
            effective.Offset(offset.x(), offset.y());
            let inputs = inputs.Read();
            let inputs = inputs.as_ref().unwrap();
            let mask =
                MaskFilterInfoForLayer(effective, inputs.corner_radii, &inputs.gradient_mask);
            let trees = c.HostMut(self.host_id().unwrap()).property_trees_mut();
            let node = trees.effect_tree_mutable().MutableNode(effect_id);
            node.mask_filter_info = mask;
            node.effect_changed = true;
            trees.effect_tree_mutable().set_needs_update(true);
        } else {
            self.SetPropertyTreesNeedRebuild(c);
        }
    }
    // Source: cc/layers/layer.cc:747-752.
    pub fn SetRoundedCorner(&self, corners: RoundedCornersF, c: &mut LayerContext<'_, B>) {
        if self.EnsureLayerTreeInputs(c).corner_radii == corners {
            return;
        }
        self.UpdateMaskFilterInfo(Some(corners), None, c);
    }
    // Source: cc/layers/layer.cc:754-759.
    pub fn SetGradientMask(&self, gradient: &LinearGradient, c: &mut LayerContext<'_, B>) {
        if self.EnsureLayerTreeInputs(c).gradient_mask == *gradient {
            return;
        }
        self.UpdateMaskFilterInfo(None, Some(gradient), c);
    }
    // Source: cc/layers/layer.cc:761-776.
    pub fn SetIsFastRoundedCorner(&self, enable: bool, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        let mut inputs = self.EnsureLayerTreeInputs(c);
        if inputs.is_fast_rounded_corner == enable {
            return;
        }
        inputs.is_fast_rounded_corner = enable;
        drop(inputs);
        if !self.HasRoundedCorner(c) {
            return;
        }
        self.SetSubtreePropertyChanged(c);
        self.SetNeedsCommit(c);
        self.SetPropertyTreesNeedRebuild(c);
    }
    // Source: cc/layers/layer.cc:778-809.
    pub fn SetOpacity(&self, opacity: f32, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        debug_assert!(opacity >= 0.);
        debug_assert!(opacity <= 1.);
        let mut inputs = self.EnsureLayerTreeInputs(c);
        if inputs.opacity == opacity {
            return;
        }
        let force_rebuild = opacity == 1. || inputs.opacity == 1.;
        inputs.opacity = opacity;
        drop(inputs);
        self.SetSubtreePropertyChanged(c);
        if let Some(host) = self.host_id() {
            if !force_rebuild {
                let id = self.effect_tree_index(c);
                if id != kInvalidPropertyNodeId {
                    let trees = c.HostMut(host).property_trees_mut();
                    let node = trees.effect_tree_mutable().MutableNode(id);
                    node.opacity = opacity;
                    node.effect_changed = true;
                    trees.effect_tree_mutable().set_needs_update(true);
                }
            } else {
                self.SetPropertyTreesNeedRebuild(c);
            }
        }
        self.SetNeedsCommit(c);
    }
    // Source: cc/layers/layer.cc:811-817.
    pub fn EffectiveOpacity(&self, c: &LayerContext<'_, B>) -> f32 {
        let Some(inputs) = self.layer_tree_inputs(c) else {
            return 1.;
        };
        if inputs.hide_layer_and_subtree {
            0.
        } else {
            inputs.opacity
        }
    }
    // Source: cc/layers/layer.cc:819-821.
    pub fn OpacityCanAnimateOnImplThread(&self) -> bool {
        false
    }
    // Source: cc/layers/layer.cc:875-884.
    pub fn SetHitTestOpaqueness(&self, value: HitTestOpaqueness, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        let mut inputs = self.inputs_.Write(&self.sync(c));
        if inputs.hit_test_opaqueness == value {
            return;
        }
        inputs.hit_test_opaqueness = value;
        drop(inputs);
        self.SetPropertyTreesNeedRebuild(c);
        self.SetNeedsCommit(c);
    }
    // Source: cc/layers/layer.cc:886-889.
    pub fn SetHitTestable(&self, value: bool, c: &mut LayerContext<'_, B>) {
        self.SetHitTestOpaqueness(
            if value {
                HitTestOpaqueness::kMixed
            } else {
                HitTestOpaqueness::kTransparent
            },
            c,
        );
    }
    // Source: cc/layers/layer.cc:891-901.
    pub fn SetContentsOpaque(&self, value: bool, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        if self.inputs_.Read(&self.sync(c)).contents_opaque == value {
            return;
        }
        let mut inputs = self.inputs_.Write(&self.sync(c));
        inputs.contents_opaque = value;
        inputs.contents_opaque_for_text = value;
        drop(inputs);
        self.SetNeedsCommit(c);
        self.SetSubtreePropertyChanged(c);
        self.SetPropertyTreesNeedRebuild(c);
    }
}

impl<B: MainLayerTypes> Layer<B> {
    // Source: cc/layers/layer.cc:1292-1298.
    fn transform_tree_index_for_trees(
        &self,
        trees: &crate::adapter::layer_tree_impl::PropertyTreesFor<B>,
        c: &LayerContext<'_, B>,
    ) -> i32 {
        if trees.sequence_number() != *self.property_tree_sequence_number_.Read(&self.sync(c)) {
            return kInvalidPropertyNodeId;
        }
        *self.transform_tree_index_.Read(&self.sync(c))
    }
    // Source: cc/layers/layer.cc:1300-1305.
    pub fn transform_tree_index_is_valid(
        &self,
        trees: &crate::adapter::layer_tree_impl::PropertyTreesFor<B>,
        c: &LayerContext<'_, B>,
    ) -> bool {
        *self.transform_tree_index_.Read(&self.sync(c)) != kInvalidPropertyNodeId
            && trees.sequence_number() == *self.property_tree_sequence_number_.Read(&self.sync(c))
    }
    // Source: cc/layers/layer.cc:1307-1311.
    pub fn transform_tree_index(&self, c: &LayerContext<'_, B>) -> i32 {
        let Some(host) = self.host_id() else {
            return kInvalidPropertyNodeId;
        };
        self.transform_tree_index_for_trees(c.Host(host).property_trees(), c)
    }
    // Source: cc/layers/layer.cc:1321-1327.
    fn clip_tree_index_for_trees(
        &self,
        trees: &crate::adapter::layer_tree_impl::PropertyTreesFor<B>,
        c: &LayerContext<'_, B>,
    ) -> i32 {
        if trees.sequence_number() != *self.property_tree_sequence_number_.Read(&self.sync(c)) {
            return kInvalidPropertyNodeId;
        }
        *self.clip_tree_index_.Read(&self.sync(c))
    }
    // Source: cc/layers/layer.cc:1329-1334.
    pub fn clip_tree_index_is_valid(
        &self,
        trees: &crate::adapter::layer_tree_impl::PropertyTreesFor<B>,
        c: &LayerContext<'_, B>,
    ) -> bool {
        *self.clip_tree_index_.Read(&self.sync(c)) != kInvalidPropertyNodeId
            && trees.sequence_number() == *self.property_tree_sequence_number_.Read(&self.sync(c))
    }
    // Source: cc/layers/layer.cc:1336-1340.
    pub fn clip_tree_index(&self, c: &LayerContext<'_, B>) -> i32 {
        let Some(host) = self.host_id() else {
            return kInvalidPropertyNodeId;
        };
        self.clip_tree_index_for_trees(c.Host(host).property_trees(), c)
    }
    // Source: cc/layers/layer.cc:1350-1356.
    fn effect_tree_index_for_trees(
        &self,
        trees: &crate::adapter::layer_tree_impl::PropertyTreesFor<B>,
        c: &LayerContext<'_, B>,
    ) -> i32 {
        if trees.sequence_number() != *self.property_tree_sequence_number_.Read(&self.sync(c)) {
            return kInvalidPropertyNodeId;
        }
        *self.effect_tree_index_.Read(&self.sync(c))
    }
    // Source: cc/layers/layer.cc:1358-1363.
    pub fn effect_tree_index_is_valid(
        &self,
        trees: &crate::adapter::layer_tree_impl::PropertyTreesFor<B>,
        c: &LayerContext<'_, B>,
    ) -> bool {
        *self.effect_tree_index_.Read(&self.sync(c)) != kInvalidPropertyNodeId
            && trees.sequence_number() == *self.property_tree_sequence_number_.Read(&self.sync(c))
    }
    // Source: cc/layers/layer.cc:1365-1369.
    pub fn effect_tree_index(&self, c: &LayerContext<'_, B>) -> i32 {
        let Some(host) = self.host_id() else {
            return kInvalidPropertyNodeId;
        };
        self.effect_tree_index_for_trees(c.Host(host).property_trees(), c)
    }
    // Source: cc/layers/layer.cc:1379-1385.
    fn scroll_tree_index_for_trees(
        &self,
        trees: &crate::adapter::layer_tree_impl::PropertyTreesFor<B>,
        c: &LayerContext<'_, B>,
    ) -> i32 {
        if trees.sequence_number() != *self.property_tree_sequence_number_.Read(&self.sync(c)) {
            return kInvalidPropertyNodeId;
        }
        *self.scroll_tree_index_.Read(&self.sync(c))
    }
    // Source: cc/layers/layer.cc:1387-1392.
    pub fn scroll_tree_index_is_valid(
        &self,
        trees: &crate::adapter::layer_tree_impl::PropertyTreesFor<B>,
        c: &LayerContext<'_, B>,
    ) -> bool {
        *self.scroll_tree_index_.Read(&self.sync(c)) != kInvalidPropertyNodeId
            && trees.sequence_number() == *self.property_tree_sequence_number_.Read(&self.sync(c))
    }
    // Source: cc/layers/layer.cc:1394-1398.
    pub fn scroll_tree_index(&self, c: &LayerContext<'_, B>) -> i32 {
        let Some(host) = self.host_id() else {
            return kInvalidPropertyNodeId;
        };
        self.scroll_tree_index_for_trees(c.Host(host).property_trees(), c)
    }
}
// Typed canonical subclass vtable: cc/layers/layer.h:55 and
// cc/layers/picture_layer.h (SetIsBackdropFilterMask). No PictureLayer state or
// algorithm is manufactured here; the real composed owner implements this.
pub trait PictureLayerObject<B: MainLayerTypes>: LayerObject<B> {
    /// Typed native subclass projection. It borrows the same most-derived
    /// allocation; no state copy or runtime downcast is introduced.
    fn picture(&self) -> &super::picture_layer::PictureLayer<B>
    where
        B: crate::adapter::picture_layer::PictureLayerTypes;
    fn SetIsBackdropFilterMask(&self, value: bool, c: &mut LayerContext<'_, B>);
}
pub struct PictureLayerRef<B: MainLayerTypes> {
    layer: LayerRef<B>,
}
impl<B: MainLayerTypes> Clone for PictureLayerRef<B> {
    fn clone(&self) -> Self {
        Self {
            layer: self.layer.clone(),
        }
    }
}
impl<B: crate::adapter::picture_layer::PictureLayerTypes> PictureLayerRef<B> {
    /// Call-scoped source PictureLayer pointer access, retaining the native
    /// strong owner throughout and preventing its concrete borrow escaping.
    pub fn WithPictureLayer<R>(
        &self,
        c: &mut LayerContext<'_, B>,
        f: impl for<'s> FnOnce(
            &'s super::picture_layer::PictureLayer<B>,
            &'s mut LayerContext<'_, B>,
        ) -> R,
    ) -> R {
        let _ = &*self.layer;
        f(
            self.layer
                .0
                .AsPictureLayer()
                .expect("typed PictureLayer owner")
                .picture(),
            c,
        )
    }
    pub fn LayerRef(&self) -> &LayerRef<B> {
        &self.layer
    }
}
impl<B: MainLayerTypes> LayerRef<B> {
    pub fn IntoPicture(self) -> Result<PictureLayerRef<B>, Self> {
        if self.0.AsPictureLayer().is_some() {
            Ok(PictureLayerRef { layer: self })
        } else {
            Err(self)
        }
    }
    // Source static upcast of a native most-derived allocation; no owner clone.
    pub(crate) fn FromObject(object: impl LayerObject<B> + 'static) -> Self {
        Self(Arc::new(object))
    }
    pub fn SetMaskLayer(&self, mask: Option<PictureLayerRef<B>>, c: &mut LayerContext<'_, B>) {
        self.0.base().SetMaskLayer(self, mask, c);
    }
    pub fn SetIsDrawable(&self, value: bool, c: &mut LayerContext<'_, B>) {
        self.0.base().SetIsDrawable(self.0.as_ref(), value, c);
    }
}
impl<B: MainLayerTypes> Layer<B> {
    // Source: cc/layers/layer.cc:654-679.
    pub fn SetMaskLayer(
        &self,
        receiver: &LayerRef<B>,
        mask: Option<PictureLayerRef<B>>,
        c: &mut LayerContext<'_, B>,
    ) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        let inputs = self.EnsureLayerTreeInputsAccess(c);
        let old = inputs
            .Read()
            .as_ref()
            .unwrap()
            .mask_layer
            .as_ref()
            .and_then(Weak::upgrade)
            .map(LayerBorrow);
        if match (&old, &mask) {
            (None, None) => true,
            (Some(old), Some(mask)) => Arc::ptr_eq(&old.0, &mask.layer.0),
            _ => false,
        } {
            return;
        }
        if let Some(old) = old {
            debug_assert!(old
                .layer(c)
                .parent(c)
                .is_some_and(|parent| std::ptr::eq(parent.layer(c), self)));
            old.RemoveFromParent(c);
        }
        inputs.Write().as_mut().unwrap().mask_layer = None;
        if let Some(mask) = &mask {
            debug_assert!(mask
                .layer
                .inputs_
                .Read(&mask.layer.sync(c))
                .children
                .is_empty());
            mask.layer.EnsureLayerTreeInputs(c).position = PointF::default();
            mask.layer.SetIsDrawable(true, c);
            mask.layer.SetBlendMode(SkBlendMode::kDstIn, c);
            mask.layer
                .0
                .AsPictureLayer()
                .expect("typed PictureLayerRef must retain its actual subclass")
                .SetIsBackdropFilterMask(false, c);
            self.AddChild(receiver, mask.layer.clone(), c);
        }
        inputs.Write().as_mut().unwrap().mask_layer = mask.as_ref().map(|mask| mask.layer.weak());
        self.SetSubtreePropertyChanged(c);
    }
    // Source: cc/layers/layer.cc:823-873.
    pub fn SetBlendMode(&self, mode: SkBlendMode, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        let mut inputs = self.EnsureLayerTreeInputs(c);
        if inputs.blend_mode == mode {
            return;
        }
        match mode {
            SkBlendMode::kSrcOver
            | SkBlendMode::kDstIn
            | SkBlendMode::kScreen
            | SkBlendMode::kOverlay
            | SkBlendMode::kDarken
            | SkBlendMode::kLighten
            | SkBlendMode::kColorDodge
            | SkBlendMode::kColorBurn
            | SkBlendMode::kHardLight
            | SkBlendMode::kSoftLight
            | SkBlendMode::kDifference
            | SkBlendMode::kExclusion
            | SkBlendMode::kMultiply
            | SkBlendMode::kHue
            | SkBlendMode::kSaturation
            | SkBlendMode::kColor
            | SkBlendMode::kLuminosity => {}
            SkBlendMode::kClear
            | SkBlendMode::kSrc
            | SkBlendMode::kDst
            | SkBlendMode::kDstOver
            | SkBlendMode::kSrcIn
            | SkBlendMode::kSrcOut
            | SkBlendMode::kDstOut
            | SkBlendMode::kSrcATop
            | SkBlendMode::kDstATop
            | SkBlendMode::kXor
            | SkBlendMode::kPlus
            | SkBlendMode::kModulate => {
                unreachable!("source unsupported Porter Duff compositing operator")
            }
        }
        inputs.blend_mode = mode;
        drop(inputs);
        self.SetNeedsCommit(c);
        self.SetSubtreePropertyChanged(c);
        self.SetPropertyTreesNeedRebuild(c);
    }
    // Source: cc/layers/layer.cc:1464-1471.
    pub fn SetIsDrawable(
        &self,
        receiver: &dyn LayerObject<B>,
        value: bool,
        c: &mut LayerContext<'_, B>,
    ) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        if self.inputs_.Read(&self.sync(c)).is_drawable == value {
            return;
        }
        self.inputs_.Write(&self.sync(c)).is_drawable = value;
        self.UpdateDrawsContent(receiver, c);
    }
    // Source: cc/layers/layer.cc:1624-1626.
    pub fn HasDrawableContent(&self, c: &LayerContext<'_, B>) -> bool {
        self.inputs_.Read(&self.sync(c)).is_drawable
    }
    // Source: cc/layers/layer.cc:1628-1635.
    pub fn UpdateDrawsContent(&self, receiver: &dyn LayerObject<B>, c: &mut LayerContext<'_, B>) {
        let value = receiver.HasDrawableContent(c);
        debug_assert!(self.inputs_.Read(&self.sync(c)).is_drawable || !value);
        if !self.SetBitFlag(value, Self::kDrawsContentFlagMask, true, false, c) {
            return;
        }
        if let Some(parent) = self.parent(c) {
            parent
                .layer(c)
                .AddDrawableDescendants(if value { 1 } else { -1 }, c);
        }
    }
}
// Source: cc/layers/layer.cc:956-969.
pub fn Are2dAxisAligned<T: crate::adapter::layer_tree_impl::LayerTreeTransform>(
    a: &T,
    b: &T,
) -> bool {
    use crate::adapter::property_tree::PropertyTreeTransform;
    if a.IsScaleOrTranslation() && b.IsScaleOrTranslation() {
        return true;
    }
    let mut inverse = T::default();
    if PropertyTreeTransform::GetInverse(b, &mut inverse) {
        inverse.MultiplyAssign(a);
        inverse.Preserves2dAxisAlignment()
    } else {
        a.Preserves2dAxisAlignment()
    }
}
impl<B: MainLayerTypes> Layer<B> {
    // Source: cc/layers/layer.h:692-694.
    pub fn has_transform_node(&self, c: &LayerContext<'_, B>) -> bool {
        self.GetBitFlag(Self::kHasTransformNodeFlagMask, c)
    }
    // Source: cc/layers/layer.h:213-215.
    pub fn position(&self, c: &LayerContext<'_, B>) -> PointF {
        self.layer_tree_inputs(c)
            .map_or_else(PointF::default, |inputs| inputs.position)
    }
    // Source: cc/layers/layer.h:464-467.
    pub fn scroll_offset(&self, c: &LayerContext<'_, B>) -> PointF {
        self.layer_tree_inputs(c)
            .map_or_else(PointF::default, |inputs| inputs.scroll_offset)
    }
    // Source: cc/layers/layer.h:702-702.
    pub fn element_id(&self, c: &LayerContext<'_, B>) -> ElementId {
        self.inputs_.Read(&self.sync(c)).element_id
    }
    // Source: cc/layers/layer.cc:903-910.
    pub fn SetContentsOpaqueForText(&self, value: bool, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        if self.inputs_.Read(&self.sync(c)).contents_opaque_for_text == value {
            return;
        }
        debug_assert!(!self.contents_opaque(c) || value);
        self.inputs_.Write(&self.sync(c)).contents_opaque_for_text = value;
        self.SetNeedsCommit(c);
    }
    // Source: cc/layers/layer.cc:912-954.
    pub fn SetPosition(&self, position: PointF, c: &mut LayerContext<'_, B>) {
        debug_assert!(!self.IsAttached() || !self.IsUsingLayerLists(c));
        if let Some(parent) = self.parent(c) {
            if parent
                .layer(c)
                .mask_layer(c)
                .is_some_and(|mask| std::ptr::eq(mask.layer(c), self))
            {
                debug_assert!(self.position(c) == PointF::default());
                return;
            }
        }
        debug_assert!(self.IsPropertyChangeAllowed(c));
        let inputs = self.EnsureLayerTreeInputsAccess(c);
        {
            let mut value = inputs.Write();
            let value = value.as_mut().unwrap();
            if value.position == position {
                return;
            }
            value.position = position;
        }
        let Some(host) = self.host_id() else {
            return;
        };
        self.SetSubtreePropertyChanged(c);
        if self.has_transform_node(c) {
            let index = *self.transform_tree_index_.Read(&self.sync(c));
            let parent = self
                .parent(c)
                .expect("source root layer position must stay zero");
            let offset =
                position.OffsetFromOrigin() + parent.layer(c).offset_to_transform_parent(c);
            let trees = c.HostMut(host).property_trees_mut();
            let node = trees.transform_tree_mutable().MutableNode(index);
            node.post_translation = offset;
            node.needs_local_transform_update = true;
            node.SetTransformChanged(
                crate::cc::trees::damage_reason::DamageReason::kUntracked,
            );
            {
                let (tree, mut context) = trees.transform_tree_with_context();
                tree.set_needs_update(true, &mut context);
            }
        } else {
            self.SetPropertyTreesNeedRebuild(c);
        }
        self.SetNeedsCommit(c);
    }
    // Source: cc/layers/layer.cc:971-1007.
    pub fn SetTransform(&self, transform: &B::Transform, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        let inputs = self.EnsureLayerTreeInputsAccess(c);
        if inputs.Read().as_ref().unwrap().transform == *transform {
            return;
        }
        self.SetSubtreePropertyChanged(c);
        if let Some(host) = self.host_id() {
            if self.has_transform_node(c) {
                let index = *self.transform_tree_index_.Read(&self.sync(c));
                let expected = self.transform_tree_index(c);
                let old = inputs.Read();
                let old = &old.as_ref().unwrap().transform;
                let trees = c.HostMut(host).property_trees_mut();
                let node = trees.transform_tree_mutable().MutableNode(index);
                debug_assert_eq!(expected, node.id);
                let preserves = Are2dAxisAligned(old, transform);
                node.local = transform.clone();
                node.needs_local_transform_update = true;
                node.SetTransformChanged(
                    crate::cc::trees::damage_reason::DamageReason::kUntracked,
                );
                {
                    let (tree, mut context) = trees.transform_tree_with_context();
                    tree.set_needs_update(true, &mut context);
                }
                if !preserves {
                    self.SetPropertyTreesNeedRebuild(c);
                }
            } else {
                self.SetPropertyTreesNeedRebuild(c);
            }
        }
        inputs.Write().as_mut().unwrap().transform = transform.clone();
        self.SetNeedsCommit(c);
    }
    // Source: cc/layers/layer.cc:1009-1040.
    pub fn SetTransformOrigin(&self, origin: Point3F, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        let inputs = self.EnsureLayerTreeInputsAccess(c);
        {
            let mut value = inputs.Write();
            let value = value.as_mut().unwrap();
            if value.transform_origin == origin {
                return;
            }
            value.transform_origin = origin;
        }
        let Some(host) = self.host_id() else {
            return;
        };
        self.SetSubtreePropertyChanged(c);
        if self.has_transform_node(c) {
            let index = *self.transform_tree_index_.Read(&self.sync(c));
            let expected = self.transform_tree_index(c);
            let trees = c.HostMut(host).property_trees_mut();
            let node = trees.transform_tree_mutable().MutableNode(index);
            debug_assert_eq!(expected, node.id);
            node.origin = origin;
            node.needs_local_transform_update = true;
            node.SetTransformChanged(
                crate::cc::trees::damage_reason::DamageReason::kUntracked,
            );
            {
                let (tree, mut context) = trees.transform_tree_with_context();
                tree.set_needs_update(true, &mut context);
            }
        } else {
            self.SetPropertyTreesNeedRebuild(c);
        }
        self.SetNeedsCommit(c);
    }
    // Source: cc/layers/layer.cc:1042-1059.
    pub fn SetScrollOffset(&self, offset: PointF, c: &mut LayerContext<'_, B>) {
        if !offset.x().is_finite() || !offset.y().is_finite() {
            return;
        }
        debug_assert!(self.IsPropertyChangeAllowed(c));
        let inputs = self.EnsureLayerTreeInputsAccess(c);
        {
            let mut value = inputs.Write();
            let value = value.as_mut().unwrap();
            if value.scroll_offset == offset {
                return;
            }
            value.scroll_offset = offset;
        }
        if !self.IsAttached() {
            return;
        }
        self.UpdatePropertyTreeScrollOffset(c);
        self.SetNeedsCommit(c);
    }
    // Source: cc/layers/layer.cc:1061-1082.
    pub fn SetScrollOffsetFromImplSide(&self, offset: PointF, c: &mut LayerContext<'_, B>) {
        if !offset.x().is_finite() || !offset.y().is_finite() {
            return;
        }
        debug_assert!(self.IsPropertyChangeAllowed(c));
        debug_assert!(self.IsAttached() && c.Host(self.host_id().unwrap()).CommitRequested());
        let inputs = self.EnsureLayerTreeInputsAccess(c);
        {
            let mut value = inputs.Write();
            let value = value.as_mut().unwrap();
            if value.scroll_offset == offset {
                return;
            }
            value.scroll_offset = offset;
        }
        self.UpdatePropertyTreeScrollOffset(c);
        // RepeatingCallback::Run retains its native bind-state before invocation.
        // Clone only that shared callback capability, then release the field guard;
        // the source explicitly permits this callback to destroy/restructure Layer.
        let callback = inputs.Read().as_ref().unwrap().did_scroll_callback.clone();
        if !callback.is_null() {
            callback.Run(offset, self.element_id(c));
        }
        // No Layer access after the native callback.
    }
    // Source: cc/layers/layer.cc:1084-1107.
    fn UpdatePropertyTreeScrollOffset(&self, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.scrollable(c));
        debug_assert!(!self.IsUsingLayerLists(c));
        if self.scroll_tree_index(c) == kInvalidPropertyNodeId {
            debug_assert!(c
                .Host(self.host_id().unwrap())
                .property_trees()
                .needs_rebuild());
            return;
        }
        debug_assert!(self.transform_tree_index(c) != kInvalidPropertyNodeId);
        let element = self.element_id(c);
        let offset = self.scroll_offset(c);
        let index = self.transform_tree_index(c);
        let trees = c.HostMut(self.host_id().unwrap()).property_trees_mut();
        {
            let (scroll, _, state) = trees.scroll_tree_with_context();
            scroll.SetScrollOffset(element, &offset, state);
        }
        let node = trees.transform_tree_mutable().MutableNode(index);
        debug_assert_eq!(index, node.id);
        node.SetScrollOffset(
            &offset,
            crate::cc::trees::damage_reason::DamageReason::kUntracked,
        );
        node.needs_local_transform_update = true;
        {
            let (tree, mut context) = trees.transform_tree_with_context();
            tree.set_needs_update(true, &mut context);
        }
    }
    // Source: cc/layers/layer.cc:1109-1113.
    pub fn SetDidScrollCallback(&self, callback: B::DidScrollCallback, c: &LayerContext<'_, B>) {
        self.EnsureLayerTreeInputs(c).did_scroll_callback = callback;
    }
    // Source: cc/layers/layer.cc:1115-1129.
    pub fn SetSubtreeCaptureId(&self, id: SubtreeCaptureId, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        let mut inputs = self.EnsureLayerTreeInputs(c);
        if inputs.subtree_capture_id == id {
            return;
        }
        debug_assert!(
            !inputs.subtree_capture_id.is_valid() || !id.is_valid(),
            "cannot change a valid in-use subtree capture id"
        );
        inputs.subtree_capture_id = id;
        drop(inputs);
        self.SetPropertyTreesNeedRebuild(c);
        self.SetNeedsCommit(c);
    }
    // Source: cc/layers/layer.cc:1131-1154.
    pub fn SetScrollable(&self, bounds: Size, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        let mut inputs = self.EnsureLayerTreeInputs(c);
        if inputs.scrollable && inputs.scroll_container_bounds == bounds {
            return;
        }
        let was_scrollable = inputs.scrollable;
        inputs.scrollable = true;
        inputs.scroll_container_bounds = bounds;
        drop(inputs);
        let Some(host) = self.host_id() else {
            return;
        };
        let id = *self.scroll_tree_index_.Read(&self.sync(c));
        if was_scrollable && id != kInvalidPropertyNodeId {
            c.HostMut(host)
                .property_trees_mut()
                .scroll_tree_mutable()
                .MutableNode(id)
                .container_bounds = bounds;
        } else {
            self.SetPropertyTreesNeedRebuild(c);
        }
        self.SetNeedsCommit(c);
    }
    // Source: cc/layers/layer.cc:1156-1158.
    pub fn IsScrollbarLayerForTesting(&self) -> bool {
        false
    }
}
impl<B: MainLayerTypes> Layer<B> {
    // Source: cc/layers/layer.h:1066-1071.
    fn EnsureRareInputs(
        &self,
        c: &LayerContext<'_, B>,
    ) -> LiveFieldWrite<std::sync::RwLockWriteGuard<'_, Inputs<B>>, RareInputs<B>> {
        let mut inputs = self.inputs_.Write(&self.sync(c));
        if inputs.rare_inputs.is_none() {
            inputs.rare_inputs = Some(Box::new(RareInputs::default()));
        }
        LiveFieldWrite::new(
            inputs,
            |v| v.rare_inputs.as_deref().unwrap(),
            |v| v.rare_inputs.as_deref_mut().unwrap(),
        )
    }
    // Source: cc/layers/layer.cc:1160-1172.
    pub fn SetMainThreadScrollHitTestRegion(
        &self,
        region: &Region<B::RegionBackend>,
        c: &mut LayerContext<'_, B>,
    ) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        {
            let inputs = self.inputs_.Read(&self.sync(c));
            let rare = &inputs.rare_inputs;
            if rare.is_none() && region.IsEmpty() {
                return;
            }
            if rare
                .as_ref()
                .is_some_and(|v| v.main_thread_scroll_hit_test_region == *region)
            {
                return;
            }
        }
        self.EnsureRareInputs(c).main_thread_scroll_hit_test_region = region.clone();
        self.SetPropertyTreesNeedRebuild(c);
        self.SetNeedsCommit(c);
    }
    // Source: cc/layers/layer.cc:1174-1187.
    pub fn SetNonCompositedScrollHitTestRects(
        &self,
        rects: Vec<ScrollHitTestRect>,
        c: &mut LayerContext<'_, B>,
    ) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        {
            let inputs = self.inputs_.Read(&self.sync(c));
            let rare = &inputs.rare_inputs;
            if rare.is_none() && rects.is_empty() {
                return;
            }
            if rare
                .as_ref()
                .is_some_and(|v| v.non_composited_scroll_hit_test_rects == rects)
            {
                return;
            }
        }
        self.EnsureRareInputs(c)
            .non_composited_scroll_hit_test_rects = rects;
        self.SetNeedsCommit(c);
    }
    // Source: cc/layers/layer.cc:1189-1197.
    pub fn SetTouchActionRegion(
        &self,
        region: TouchActionRegion<B::RegionBackend>,
        c: &mut LayerContext<'_, B>,
    ) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        if self.inputs_.Read(&self.sync(c)).touch_action_region == region {
            return;
        }
        self.inputs_.Write(&self.sync(c)).touch_action_region = region;
        self.SetPropertyTreesNeedRebuild(c);
        self.SetNeedsCommit(c);
    }
    // Source: cc/layers/layer.cc:1199-1210.
    pub fn SetCaptureBounds(&self, bounds: B::RegionCaptureBounds, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        {
            let inputs = self.inputs_.Read(&self.sync(c));
            let rare = &inputs.rare_inputs;
            if rare.is_none() && bounds.IsEmpty() {
                return;
            }
            if rare.as_ref().is_some_and(|v| v.capture_bounds == bounds) {
                return;
            }
        }
        self.EnsureRareInputs(c).capture_bounds = bounds;
        self.SetPropertyTreesNeedRebuild(c);
        self.SetNeedsCommit(c);
        self.SetSubtreePropertyChanged(c);
    }
    // Source: cc/layers/layer.cc:1212-1225.
    pub fn SetTrackedElementRects(
        &self,
        rects: B::TrackedElementRects,
        c: &mut LayerContext<'_, B>,
    ) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        {
            let inputs = self.inputs_.Read(&self.sync(c));
            let rare = &inputs.rare_inputs;
            if rare.is_none() && rects.IsEmpty() {
                return;
            }
            if rare
                .as_ref()
                .is_some_and(|v| v.tracked_element_rects == rects)
            {
                return;
            }
        }
        self.EnsureRareInputs(c).tracked_element_rects = rects;
        self.SetPropertyTreesNeedRebuild(c);
        self.SetNeedsCommit(c);
        self.SetSubtreePropertyChanged(c);
    }
    // Source: cc/layers/layer.cc:1227-1236.
    pub fn SetWheelEventRegion(
        &self,
        region: Region<B::RegionBackend>,
        c: &mut LayerContext<'_, B>,
    ) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        {
            let inputs = self.inputs_.Read(&self.sync(c));
            let rare = &inputs.rare_inputs;
            if rare.is_none() && region.IsEmpty() {
                return;
            }
            if rare
                .as_ref()
                .is_some_and(|v| v.wheel_event_region == region)
            {
                return;
            }
        }
        self.EnsureRareInputs(c).wheel_event_region = region;
        self.SetNeedsCommit(c);
    }
    // Source: cc/layers/layer.cc:1238-1250 (BUILDFLAG(IS_ANDROID)).
    #[cfg(target_os = "android")]
    pub fn SetXrHitTestOrder(&self, order: Vec<ElementId>, c: &LayerContext<'_, B>) {
        assert!(self.IsPropertyChangeAllowed(c));
        {
            let inputs = self.inputs_.Read(&self.sync(c));
            let rare = &inputs.rare_inputs;
            if rare.is_none() && order.is_empty() {
                return;
            }
            if rare.as_ref().is_some_and(|v| v.xr_hit_test_order == order) {
                return;
            }
        }
        self.EnsureRareInputs(c).xr_hit_test_order = order;
    }
    // Source: cc/layers/layer.cc:1252-1263.
    pub fn SetCanvasChildId(&self, id: ElementId, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        {
            let inputs = self.inputs_.Read(&self.sync(c));
            let rare = &inputs.rare_inputs;
            if rare.is_none() && !id.IsValid() {
                return;
            }
            if rare.as_ref().is_some_and(|v| v.canvas_child_id == id) {
                return;
            }
        }
        self.EnsureRareInputs(c).canvas_child_id = id;
        self.SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
    }
    // Source: cc/layers/layer.cc:1265-1281.
    pub fn GetRenderSurfaceReason(
        &self,
        c: &LayerContext<'_, B>,
    ) -> crate::cc::trees::effect_node::RenderSurfaceReason {
        use crate::cc::trees::effect_node::RenderSurfaceReason;
        let Some(host) = self.host_id() else {
            return RenderSurfaceReason::kNone;
        };
        let trees = c.Host(host).property_trees();
        debug_assert!(!trees.needs_rebuild());
        let index = self.effect_tree_index(c);
        if index == kInvalidPropertyNodeId
            || self
                .parent(c)
                .is_some_and(|parent| index == parent.layer(c).effect_tree_index(c))
        {
            return RenderSurfaceReason::kNone;
        }
        trees.effect_tree().Node(index).render_surface_reason
    }
    // Source: cc/layers/layer.cc:1400-1406.
    pub fn SetOffsetToTransformParent(&self, offset: Vector2dF, c: &mut LayerContext<'_, B>) {
        if *self.offset_to_transform_parent_.Read(&self.sync(c)) == offset {
            return;
        }
        *self.offset_to_transform_parent_.Write(&self.sync(c)) = offset;
        self.SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
        self.SetSubtreePropertyChanged(c);
    }
    // Source: cc/layers/layer.cc:1432-1438.
    pub fn ClearDebugInfo(&self, c: &mut LayerContext<'_, B>) {
        if self.debug_info_.Read(&self.sync(c)).is_none() {
            return;
        }
        self.debug_info_.Write(&self.sync(c)).take();
        self.SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
    }
    // Source: cc/layers/layer.cc:1440-1443.
    pub fn DebugName(&self, c: &LayerContext<'_, B>) -> String {
        self.debug_info_
            .Read(&self.sync(c))
            .as_ref()
            .map_or_else(String::new, |info| info.name.clone())
    }
    // Source: cc/layers/layer.cc:1473-1483.
    pub fn SetHideLayerAndSubtree(&self, hide: bool, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        let inputs = self.EnsureLayerTreeInputsAccess(c);
        if inputs.Read().as_ref().unwrap().hide_layer_and_subtree == hide {
            return;
        }
        inputs.Write().as_mut().unwrap().hide_layer_and_subtree = hide;
        self.SetNeedsCommit(c);
        self.SetPropertyTreesNeedRebuild(c);
        self.SetSubtreePropertyChanged(c);
    }
    // Source: cc/layers/layer.cc:1502-1504.
    pub fn RequiresSetNeedsDisplayOnHdrHeadroomChange(&self) -> bool {
        false
    }
    // Source: cc/layers/layer.cc:1506-1508.
    pub fn IsSnappedToPixelGridInTarget(&self) -> bool {
        false
    }
    // Source: cc/layers/layer.cc:1641-1644.
    pub fn Update(&self, _c: &LayerContext<'_, B>) -> bool {
        debug_assert!(self.IsAttached());
        false
    }
    // Source: cc/layers/layer.cc:1646-1648.
    pub fn MayUpdateAfterPaintEvent(&self) -> bool {
        false
    }
    // Source: cc/layers/layer.cc:1650-1653.
    pub fn UpdateAfterPaintEvent(&self, _c: &LayerContext<'_, B>) -> bool {
        debug_assert!(self.IsAttached());
        false
    }
    // Source: cc/layers/layer.cc:1687-1689.
    pub fn OnFilterAnimated(&self, filters: &B::Filters, c: &LayerContext<'_, B>) {
        self.EnsureLayerTreeInputs(c).filters = filters.clone();
    }
    // Source: cc/layers/layer.cc:1691-1693.
    pub fn OnBackdropFilterAnimated(&self, filters: &B::Filters, c: &LayerContext<'_, B>) {
        self.EnsureLayerTreeInputs(c).backdrop_filters = filters.clone();
    }
    // Source: cc/layers/layer.cc:1695-1697.
    pub fn OnOpacityAnimated(&self, opacity: f32, c: &LayerContext<'_, B>) {
        self.EnsureLayerTreeInputs(c).opacity = opacity;
    }
    // Source: cc/layers/layer.cc:1699-1701.
    pub fn OnTransformAnimated(&self, transform: &B::Transform, c: &LayerContext<'_, B>) {
        self.EnsureLayerTreeInputs(c).transform = transform.clone();
    }
    // Source: cc/layers/layer.cc:1703-1714.
    pub fn SetTrilinearFiltering(&self, value: bool, c: &mut LayerContext<'_, B>) {
        let inputs = self.EnsureLayerTreeInputsAccess(c);
        if inputs.Read().as_ref().unwrap().trilinear_filtering == value {
            return;
        }
        inputs.Write().as_mut().unwrap().trilinear_filtering = value;
        self.SetPropertyTreesNeedRebuild(c);
        self.SetSubtreePropertyChanged(c);
        self.SetNeedsCommit(c);
    }
    // Source: cc/layers/layer.h:720-722.
    pub fn mirror_count(&self, c: &LayerContext<'_, B>) -> i32 {
        self.layer_tree_inputs(c).map_or(0, |v| v.mirror_count)
    }
    // Source: cc/layers/layer.cc:1716-1718.
    pub fn IncrementMirrorCount(&self, c: &mut LayerContext<'_, B>) {
        self.SetMirrorCount(self.mirror_count(c) + 1, c);
    }
    // Source: cc/layers/layer.cc:1720-1722.
    pub fn DecrementMirrorCount(&self, c: &mut LayerContext<'_, B>) {
        self.SetMirrorCount(self.mirror_count(c) - 1, c);
    }
    // Source: cc/layers/layer.cc:1724-1736.
    fn SetMirrorCount(&self, count: i32, c: &mut LayerContext<'_, B>) {
        let inputs = self.EnsureLayerTreeInputsAccess(c);
        let old = inputs.Read().as_ref().unwrap().mirror_count;
        if old == count {
            return;
        }
        debug_assert!(count >= 0);
        let was = old > 0;
        inputs.Write().as_mut().unwrap().mirror_count = count;
        let is = count > 0;
        if was != is {
            self.SetPropertyTreesNeedRebuild(c);
        }
        self.SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
    }
    // Source: cc/layers/layer.cc:1738-1740.
    pub fn GetElementTypeForAnimation(&self) -> crate::adapter::layer_tree_impl::ElementListType {
        crate::adapter::layer_tree_impl::ElementListType::ACTIVE
    }
    // Source: cc/layers/layer.cc:1753.
    pub fn RunMicroBenchmark(&self, _benchmark: &mut B::MicroBenchmark) {}
    // Source: cc/layers/layer.cc:1779-1788.
    pub fn SetFilterQuality(&self, quality: B::FilterQuality, c: &mut LayerContext<'_, B>) {
        let old = self
            .inputs_
            .Read(&self.sync(c))
            .rare_inputs
            .as_ref()
            .map_or_else(B::FilterQuality::default, |v| v.filter_quality);
        if old == quality {
            return;
        }
        self.EnsureRareInputs(c).filter_quality = quality;
        self.SetNeedsCommit(c);
    }
    // Source: cc/layers/layer.cc:1790-1802.
    pub fn SetDynamicRangeLimit(&self, limit: B::DynamicRangeLimit, c: &mut LayerContext<'_, B>) {
        let old = self
            .inputs_
            .Read(&self.sync(c))
            .rare_inputs
            .as_ref()
            .map_or_else(B::DynamicRangeLimit::default, |v| v.dynamic_range_limit);
        if old == limit {
            return;
        }
        self.EnsureRareInputs(c).dynamic_range_limit = limit;
        self.SetNeedsCommit(c);
    }
}
impl<B: MainLayerTypes> Layer<B> {
    // Source: cc/layers/layer.cc:1485-1500.
    pub fn SetNeedsDisplayRect(
        &self,
        receiver: &(impl LayerObject<B> + ?Sized),
        rect: &Rect,
        c: &mut LayerContext<'_, B>,
    ) {
        if rect.IsEmpty() {
            return;
        }
        self.update_rect_.write().unwrap().Union(*rect);
        if self.IsAttached()
            && c.Host(self.host_id().unwrap()).in_will_commit()
            && receiver.MayUpdateAfterPaintEvent()
        {
            return;
        }
        if self.draws_content(c)
            && self.IsAttached()
            && !*self.ignore_set_needs_commit_for_test_.read().unwrap()
        {
            c.HostMut(self.host_id().unwrap()).SetNeedsUpdateLayers();
        }
    }
    // Source: cc/layers/layer.cc:1510-1581.
    pub fn PushDirtyPropertiesTo(
        &self,
        layer: &mut B::Layer,
        dirty: u8,
        commit: &mut crate::cc::trees::commit_state::CommitState<B>,
        c: &mut LayerContext<'_, B>,
        impl_tree: &mut B::ImplContext<'_>,
    ) {
        let trees = &commit.property_trees;
        if dirty & Self::kChangedPropertyTreeIndex != 0 {
            layer.SetTransformTreeIndex(self.transform_tree_index_for_trees(trees, c), impl_tree);
            layer.SetHasTransformNode(self.has_transform_node(c), impl_tree);
            layer.SetEffectTreeIndex(self.effect_tree_index_for_trees(trees, c), impl_tree);
            layer.SetClipTreeIndex(self.clip_tree_index_for_trees(trees, c), impl_tree);
            layer.SetScrollTreeIndex(self.scroll_tree_index_for_trees(trees, c), impl_tree);
        }
        if dirty & Self::kChangedGeneralProperty != 0 {
            let inputs = self.inputs_.Read(&self.sync(c));
            layer.SetElementId(inputs.element_id, impl_tree);
            layer.SetBackgroundColor(inputs.background_color, impl_tree);
            layer.SetSafeOpaqueBackgroundColor(self.SafeOpaqueBackgroundColor(c), impl_tree);
            layer.SetBounds(&inputs.bounds, impl_tree);
            layer.SetOffsetToTransformParent(
                *self.offset_to_transform_parent_.Read(&self.sync(c)),
                impl_tree,
            );
            let has_canvas_child = inputs
                .rare_inputs
                .as_ref()
                .is_some_and(|v| v.canvas_child_id.IsValid());
            layer.SetDrawsContent(self.draws_content(c) && !has_canvas_child, impl_tree);
            layer.SetHitTestOpaqueness(inputs.hit_test_opaqueness, impl_tree);
            if *self.subtree_property_changed_.Read(&self.sync(c)) {
                layer.NoteLayerPropertyChanged(impl_tree);
            }
            layer.SetTouchActionRegion(&inputs.touch_action_region, impl_tree);
            layer.SetContentsOpaque(inputs.contents_opaque, impl_tree);
            layer.SetContentsOpaqueForText(inputs.contents_opaque_for_text, impl_tree);
            layer.SetShouldCheckBackfaceVisibility(
                self.should_check_backface_visibility(c),
                impl_tree,
            );
            debug_assert!(layer.LifecycleAllowsPropertyTreeAccess(impl_tree));
            if let Some(rect) = commit.layer_update_rects.get(&self.id()) {
                layer.UnionUpdateRect(rect, impl_tree);
            }
            layer.UpdateDebugInfo(
                self.debug_info_.Write(&self.sync(c)).as_deref_mut(),
                impl_tree,
            );
            if let Some(rare) = &inputs.rare_inputs {
                layer.SetFilterQuality(rare.filter_quality, impl_tree);
                layer.SetDynamicRangeLimit(rare.dynamic_range_limit, impl_tree);
                layer.SetMainThreadScrollHitTestRegion(
                    &rare.main_thread_scroll_hit_test_region,
                    impl_tree,
                );
                layer.SetNonCompositedScrollHitTestRects(
                    &rare.non_composited_scroll_hit_test_rects,
                    impl_tree,
                );
                layer.SetCaptureBounds(&rare.capture_bounds, impl_tree);
                layer.SetTrackedElementRects(&rare.tracked_element_rects, impl_tree);
                layer.SetWheelEventHandlerRegion(&rare.wheel_event_region, impl_tree);
                layer.SetCanvasChildId(rare.canvas_child_id, impl_tree);
            } else {
                layer.ResetRareProperties(impl_tree);
            }
            *self.subtree_property_changed_.Write(&self.sync(c)) = false;
        }
        layer.SetNeedsPushProperties(dirty, impl_tree);
    }
    // Source: cc/layers/layer.cc:1583-1594.
    pub fn PushPropertiesTo(
        &self,
        receiver: &(impl LayerObject<B> + ?Sized),
        layer: &mut B::Layer,
        commit: &mut crate::cc::trees::commit_state::CommitState<B>,
        c: &mut LayerContext<'_, B>,
        impl_tree: &mut B::ImplContext<'_>,
    ) {
        c.Services().TraceLayer("Layer::PushPropertiesTo", None);
        debug_assert!(self.IsAttached());
        let changed = *self.changed_properties_.Read(&self.sync(c));
        receiver.PushDirtyPropertiesTo(layer, changed, commit, c, impl_tree);
        *self.changed_properties_.Write(&self.sync(c)) = 0;
    }
    // Source: cc/layers/layer.cc:1596-1617.
    pub fn TakeCopyRequests(
        &self,
        requests: &mut Vec<Option<Box<B::CopyOutputRequest>>>,
        c: &LayerContext<'_, B>,
    ) {
        if self.layer_tree_inputs(c).is_none() {
            return;
        }
        let mut inputs = self.layer_tree_inputs_.Write(&self.sync(c));
        let inputs = inputs.as_mut().unwrap();
        for request in &mut inputs.copy_requests {
            let value = request
                .as_mut()
                .expect("native copy request slot must be nonnull before move");
            if !value.has_result_task_runner() {
                value
                    .set_result_task_runner(c.Host(self.host_id().unwrap()).MainThreadTaskRunner());
            }
            if value.has_area() {
                let mut area = *value.area();
                {
                    let bounds = self.bounds(c);
                    area.Intersect(Rect::new(0, 0, bounds.width(), bounds.height()));
                }
                value.set_area(area);
            }
            requests.push(request.take());
        }
        inputs.copy_requests.clear();
    }
    // Source: cc/layers/layer.cc:1619-1622.
    pub fn CreateLayerImpl(
        &self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        tree_context: &mut crate::cc::trees::layer_tree_impl::LayerTreeContext<'_, B>,
    ) -> Box<B::Layer> {
        <B::Layer as MainLayerImpl<B>>::Create(tree, self.id(), tree_context)
    }
    // Source: cc/layers/layer.cc:1755-1771.
    pub fn SetElementId(&self, id: ElementId, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        if self.inputs_.Read(&self.sync(c)).element_id == id {
            return;
        }
        c.Services().TraceLayer("Layer::SetElementId", Some(id));
        let inputs = self.inputs_.WriteAccess(&self.sync(c));
        let old = inputs.Read().element_id;
        if let Some(host) = self.host_id() {
            if old.IsValid() {
                c.HostMut(host).UnregisterElement(old, self);
            }
        }
        inputs.Write().element_id = id;
        if let Some(host) = self.host_id() {
            if inputs.Read().element_id.IsValid() {
                c.HostMut(host)
                    .RegisterElement(inputs.Read().element_id, self);
            }
        }
        self.SetNeedsCommit(c);
    }
    // Source: cc/layers/layer.cc:1773-1777.
    pub fn ScreenSpaceTransform(&self, c: &LayerContext<'_, B>) -> B::Transform {
        debug_assert_ne!(
            *self.transform_tree_index_.Read(&self.sync(c)),
            kInvalidPropertyNodeId
        );
        crate::cc::trees::draw_property_utils::ScreenSpaceTransformForLayer(
            self.offset_to_transform_parent(c),
            self.transform_tree_index(c),
            c.Host(self.host_id().unwrap())
                .property_trees()
                .transform_tree(),
        )
    }
}
impl<B: MainLayerTypes> Layer<B> {
    // Source: cc/layers/layer.h:146-146.
    pub fn children(
        &self,
        c: &LayerContext<'_, B>,
    ) -> impl std::ops::Deref<Target = Vec<LayerRef<B>>> + '_ {
        LiveFieldRead::new(self.inputs_.Read(&self.sync(c)), |v| &v.children)
    }
    // Source: cc/layers/layer.h:162-165. Native raw_ptr is resolved per call.
    pub fn layer_tree_host<'a>(
        &self,
        c: &'a LayerContext<'_, B>,
    ) -> Option<&'a dyn MainLayerHostRead<B>> {
        debug_assert!(!self.IsAttached() || self.IsMainThread(c));
        self.host_id().map(|id| c.Host(id))
    }
    // Source: cc/layers/layer.h:175-177.
    pub fn HasCopyRequest(&self, c: &LayerContext<'_, B>) -> bool {
        self.layer_tree_inputs(c)
            .is_some_and(|v| !v.copy_requests.is_empty())
    }
    // Source: cc/layers/layer.h:279-282.
    pub fn update_rect(&self, c: &LayerContext<'_, B>) -> impl std::ops::Deref<Target = Rect> + '_ {
        debug_assert!(!self.IsAttached() || self.IsMainThread(c));
        self.update_rect_.read().unwrap()
    }
    // Source: cc/layers/layer.h:283-286.
    pub fn ResetUpdateRect(&self, c: &LayerContext<'_, B>) {
        debug_assert!(!self.IsAttached() || self.IsMainThread(c));
        *self.update_rect_.write().unwrap() = Rect::default();
    }
    // Source: cc/layers/layer.h:302-305.
    pub fn corner_radii(
        &self,
        c: &LayerContext<'_, B>,
    ) -> impl std::ops::Deref<Target = RoundedCornersF> + '_ {
        match self.layer_tree_inputs(c) {
            Some(v) => ReadOrDefault::Read(LiveFieldRead::new(v, |v| &v.corner_radii)),
            None => ReadOrDefault::Default(RoundedCornersF::default()),
        }
    }
    // Source: cc/layers/layer.h:307-307.
    pub fn HasRoundedCorner(&self, c: &LayerContext<'_, B>) -> bool {
        !self.corner_radii(c).IsEmpty()
    }
    // Source: cc/layers/layer.h:315-317.
    pub fn is_fast_rounded_corner(&self, c: &LayerContext<'_, B>) -> bool {
        self.layer_tree_inputs(c)
            .is_some_and(|v| v.is_fast_rounded_corner)
    }
    // Source: cc/layers/layer.h:328-331.
    pub fn gradient_mask(
        &self,
        c: &LayerContext<'_, B>,
    ) -> impl std::ops::Deref<Target = LinearGradient> + '_ {
        match self.layer_tree_inputs(c) {
            Some(v) => ReadOrDefault::Read(LiveFieldRead::new(v, |v| &v.gradient_mask)),
            None => ReadOrDefault::Default(LinearGradient::default()),
        }
    }
    // Source: cc/layers/layer.h:332-332.
    pub fn HasGradientMask(&self, c: &LayerContext<'_, B>) -> bool {
        !self.gradient_mask(c).steps.is_empty()
    }
    // Source: cc/layers/layer.h:334-334.
    pub fn HasMaskFilter(&self, c: &LayerContext<'_, B>) -> bool {
        self.HasRoundedCorner(c) || self.HasGradientMask(c)
    }
    // Source: cc/layers/layer.h:343-345.
    pub fn opacity(&self, c: &LayerContext<'_, B>) -> f32 {
        self.layer_tree_inputs(c).map_or(1., |v| v.opacity)
    }
    // Source: cc/layers/layer.h:357-360.
    pub fn blend_mode(&self, c: &LayerContext<'_, B>) -> SkBlendMode {
        self.layer_tree_inputs(c)
            .map_or(SkBlendMode::kSrcOver, |v| v.blend_mode)
    }
    // Source: cc/layers/layer.h:367-370. Native getter returns a value copy.
    pub fn filters(&self, c: &LayerContext<'_, B>) -> B::Filters {
        self.layer_tree_inputs(c)
            .map_or_else(B::Filters::default, |v| v.filters.clone())
    }
    // Source: cc/layers/layer.h:377-380. Native getter returns a value copy.
    pub fn backdrop_filters(&self, c: &LayerContext<'_, B>) -> B::Filters {
        self.layer_tree_inputs(c)
            .map_or_else(B::Filters::default, |v| v.backdrop_filters.clone())
    }
    // Source: cc/layers/layer.h:388-391. Native getter returns an optional value copy.
    pub fn backdrop_filter_bounds(&self, c: &LayerContext<'_, B>) -> Option<B::Path> {
        self.layer_tree_inputs(c)
            .and_then(|v| v.backdrop_filter_bounds.clone())
    }
    // Source: cc/layers/layer.h:395-398.
    pub fn backdrop_filter_quality(&self, c: &LayerContext<'_, B>) -> f32 {
        self.layer_tree_inputs(c)
            .map_or(1., |v| v.backdrop_filter_quality)
    }
    // Source: cc/layers/layer.h:419-421.
    pub fn contents_opaque_for_text(&self, c: &LayerContext<'_, B>) -> bool {
        self.inputs_.Read(&self.sync(c)).contents_opaque_for_text
    }
    // Source: cc/layers/layer.h:426-428.
    pub fn hit_test_opaqueness(&self, c: &LayerContext<'_, B>) -> HitTestOpaqueness {
        self.inputs_.Read(&self.sync(c)).hit_test_opaqueness
    }
    // Source: cc/layers/layer.h:434-437.
    pub fn transform(
        &self,
        c: &LayerContext<'_, B>,
    ) -> impl std::ops::Deref<Target = B::Transform> + '_ {
        match self.layer_tree_inputs(c) {
            Some(v) => ReadOrDefault::Read(LiveFieldRead::new(v, |v| &v.transform)),
            None => ReadOrDefault::Default(B::Transform::default()),
        }
    }
    // Source: cc/layers/layer.h:453-456.
    pub fn transform_origin(&self, c: &LayerContext<'_, B>) -> Point3F {
        self.layer_tree_inputs(c)
            .map_or_else(Point3F::default, |v| v.transform_origin)
    }
    // Source: cc/layers/layer.h:483-486.
    pub fn scroll_container_bounds(&self, c: &LayerContext<'_, B>) -> Size {
        self.layer_tree_inputs(c)
            .map_or_else(Size::default, |v| v.scroll_container_bounds)
    }
    // Source: cc/layers/layer.h:497-501.
    pub fn main_thread_scroll_hit_test_region(
        &self,
        c: &LayerContext<'_, B>,
    ) -> impl std::ops::Deref<Target = Region<B::RegionBackend>> + '_ {
        let inputs = self.inputs_.Read(&self.sync(c));
        if inputs.rare_inputs.is_some() {
            ReadOrDefault::Read(LiveFieldRead::new(inputs, |v| {
                &v.rare_inputs
                    .as_ref()
                    .unwrap()
                    .main_thread_scroll_hit_test_region
            }))
        } else {
            ReadOrDefault::Default(Region::default())
        }
    }
    // Source: cc/layers/layer.h:508-514.
    pub fn non_composited_scroll_hit_test_rects(
        &self,
        c: &LayerContext<'_, B>,
    ) -> Option<impl std::ops::Deref<Target = Vec<ScrollHitTestRect>> + '_> {
        let inputs = self.inputs_.Read(&self.sync(c));
        if inputs.rare_inputs.is_some() {
            Some(LiveFieldRead::new(inputs, |v| {
                &v.rare_inputs
                    .as_ref()
                    .unwrap()
                    .non_composited_scroll_hit_test_rects
            }))
        } else {
            None
        }
    }
    // Source: cc/layers/layer.h:522-524.
    pub fn touch_action_region(
        &self,
        c: &LayerContext<'_, B>,
    ) -> impl std::ops::Deref<Target = TouchActionRegion<B::RegionBackend>> + '_ {
        LiveFieldRead::new(self.inputs_.Read(&self.sync(c)), |v| &v.touch_action_region)
    }
    // Source: cc/layers/layer.h:528-532.
    pub fn capture_bounds(
        &self,
        c: &LayerContext<'_, B>,
    ) -> impl std::ops::Deref<Target = B::RegionCaptureBounds> + '_ {
        let inputs = self.inputs_.Read(&self.sync(c));
        if inputs.rare_inputs.is_some() {
            ReadOrDefault::Read(LiveFieldRead::new(inputs, |v| {
                &v.rare_inputs.as_ref().unwrap().capture_bounds
            }))
        } else {
            ReadOrDefault::Default(B::RegionCaptureBounds::default())
        }
    }
    // Source: cc/layers/layer.h:537-542.
    pub fn tracked_element_rects(
        &self,
        c: &LayerContext<'_, B>,
    ) -> impl std::ops::Deref<Target = B::TrackedElementRects> + '_ {
        let inputs = self.inputs_.Read(&self.sync(c));
        if inputs.rare_inputs.is_some() {
            ReadOrDefault::Read(LiveFieldRead::new(inputs, |v| {
                &v.rare_inputs.as_ref().unwrap().tracked_element_rects
            }))
        } else {
            ReadOrDefault::Default(B::TrackedElementRects::default())
        }
    }
    // Source: cc/layers/layer.h:550-554.
    pub fn wheel_event_region(
        &self,
        c: &LayerContext<'_, B>,
    ) -> impl std::ops::Deref<Target = Region<B::RegionBackend>> + '_ {
        let inputs = self.inputs_.Read(&self.sync(c));
        if inputs.rare_inputs.is_some() {
            ReadOrDefault::Read(LiveFieldRead::new(inputs, |v| {
                &v.rare_inputs.as_ref().unwrap().wheel_event_region
            }))
        } else {
            ReadOrDefault::Default(Region::default())
        }
    }
    // Source: cc/layers/layer.h:596-604 (BUILDFLAG(IS_ANDROID)).
    #[cfg(target_os = "android")]
    pub fn xr_hit_test_order(
        &self,
        c: &LayerContext<'_, B>,
    ) -> Option<impl std::ops::Deref<Target = Vec<ElementId>> + '_> {
        let inputs = self.inputs_.Read(&self.sync(c));
        if inputs.rare_inputs.is_some() {
            Some(LiveFieldRead::new(inputs, |v| {
                &v.rare_inputs.as_ref().unwrap().xr_hit_test_order
            }))
        } else {
            None
        }
    }
    // Source: cc/layers/layer.h:569-574.
    pub fn canvas_child_id(&self, c: &LayerContext<'_, B>) -> ElementId {
        self.inputs_
            .Read(&self.sync(c))
            .rare_inputs
            .as_ref()
            .map_or_else(ElementId::default, |v| v.canvas_child_id)
    }
    // Source: cc/layers/layer.h:602-606.
    pub fn subtree_capture_id(&self, c: &LayerContext<'_, B>) -> SubtreeCaptureId {
        self.layer_tree_inputs(c)
            .map_or_else(SubtreeCaptureId::default, |v| v.subtree_capture_id)
    }
    // Source: cc/layers/layer.h:618-621.
    pub fn SetCacheRenderSurface(&self, value: bool, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        self.SetBitFlag(value, Self::kCacheRenderSurfaceFlagMask, true, false, c);
    }
    // Source: cc/layers/layer.h:622-624.
    pub fn cache_render_surface(&self, c: &LayerContext<'_, B>) -> bool {
        self.GetBitFlag(Self::kCacheRenderSurfaceFlagMask, c)
    }
    // Source: cc/layers/layer.h:635-639.
    pub fn SetForceRenderSurfaceForTesting(&self, value: bool, c: &mut LayerContext<'_, B>) {
        debug_assert!(self.IsPropertyChangeAllowed(c));
        self.SetBitFlag(
            value,
            Self::kForceRenderSurfaceForTestingFlagMask,
            true,
            false,
            c,
        );
    }
    // Source: cc/layers/layer.h:640-642.
    pub fn force_render_surface_for_testing(&self, c: &LayerContext<'_, B>) -> bool {
        self.GetBitFlag(Self::kForceRenderSurfaceForTestingFlagMask, c)
    }
    // Source: cc/layers/layer.h:668-670.
    pub fn hide_layer_and_subtree(&self, c: &LayerContext<'_, B>) -> bool {
        self.layer_tree_inputs(c)
            .is_some_and(|v| v.hide_layer_and_subtree)
    }
    // Source: cc/layers/layer.h:697-697.
    pub fn SetHasClipNode(&self, value: bool, c: &mut LayerContext<'_, B>) {
        self.SetBitFlag(value, Self::kHasClipNodeFlagMask, false, false, c);
    }
    // Source: cc/layers/layer.h:698-698.
    pub fn has_clip_node(&self, c: &LayerContext<'_, B>) -> bool {
        self.GetBitFlag(Self::kHasClipNodeFlagMask, c)
    }
    // Source: cc/layers/layer.h:712-714.
    pub fn trilinear_filtering(&self, c: &LayerContext<'_, B>) -> bool {
        self.layer_tree_inputs(c)
            .is_some_and(|v| v.trilinear_filtering)
    }
    // Source: cc/layers/layer.h:737-737.
    pub fn debug_info(
        &self,
        c: &LayerContext<'_, B>,
    ) -> Option<impl std::ops::Deref<Target = LayerDebugInfo<B::NodeId>> + '_> {
        let info = self.debug_info_.Read(&self.sync(c));
        if info.is_some() {
            Some(LiveFieldRead::new(info, |v| v.as_ref().unwrap().as_ref()))
        } else {
            None
        }
    }
    // Source: cc/layers/layer.h:804-806.
    pub fn ClearChangedPushPropertiesForTesting(&self, c: &LayerContext<'_, B>) {
        *self.changed_properties_.Write(&self.sync(c)) = 0;
    }
    // Source: cc/layers/layer.h:839-841.
    pub fn ClearSubtreePropertyChangedForTesting(&self, c: &LayerContext<'_, B>) {
        *self.subtree_property_changed_.Write(&self.sync(c)) = false;
    }
    // Source: cc/layers/layer.h:842-844.
    pub fn subtree_property_changed(&self, c: &LayerContext<'_, B>) -> bool {
        *self.subtree_property_changed_.Read(&self.sync(c))
    }
    // Source: cc/layers/layer.h:861-863.
    pub fn SetSubtreeHasCopyRequest(&self, value: bool, c: &mut LayerContext<'_, B>) {
        self.SetBitFlag(value, Self::kSubtreeHasCopyRequestFlagMask, false, false, c);
    }
    // Source: cc/layers/layer.h:864-866.
    pub fn subtree_has_copy_request(&self, c: &LayerContext<'_, B>) -> bool {
        self.GetBitFlag(Self::kSubtreeHasCopyRequestFlagMask, c)
    }
    // Source: cc/layers/layer.h:875-878.
    pub fn SetShouldCheckBackfaceVisibility(&self, value: bool, c: &mut LayerContext<'_, B>) {
        self.SetBitFlag(
            value,
            Self::kShouldCheckBackfaceVisibilityFlagMask,
            false,
            true,
            c,
        );
    }
    // Source: cc/layers/layer.h:879-881.
    pub fn should_check_backface_visibility(&self, c: &LayerContext<'_, B>) -> bool {
        self.GetBitFlag(Self::kShouldCheckBackfaceVisibilityFlagMask, c)
    }
    // Source: cc/layers/layer.h:950-953.
    pub fn IncreasePaintCount(&self, c: &LayerContext<'_, B>) {
        if self.debug_info_.Read(&self.sync(c)).is_some() {
            self.debug_info_
                .Write(&self.sync(c))
                .as_mut()
                .unwrap()
                .paint_count += 1;
        }
    }
    // Source: cc/layers/layer.h:955-957.
    pub fn IgnoreSetNeedsCommitForTest(&self) -> IgnoreSetNeedsCommit<'_> {
        let mut value = self.ignore_set_needs_commit_for_test_.write().unwrap();
        let old = *value;
        *value = true;
        IgnoreSetNeedsCommit {
            value: &self.ignore_set_needs_commit_for_test_,
            old,
        }
    }
}
// Rust-only analogue of a const reference into optional native storage. Only the
// absent branch owns a canonical empty/identity value; present state is borrowed.
enum ReadOrDefault<G: std::ops::Deref<Target = T>, T> {
    Read(G),
    Default(T),
}
impl<G: std::ops::Deref<Target = T>, T> std::ops::Deref for ReadOrDefault<G, T> {
    type Target = T;
    fn deref(&self) -> &T {
        match self {
            Self::Read(v) => v,
            Self::Default(v) => v,
        }
    }
}
// Source: cc/layers/layer.h:956-958. AutoReset stores only this field's old bool.
pub struct IgnoreSetNeedsCommit<'a> {
    value: &'a RwLock<bool>,
    old: bool,
}
impl Drop for IgnoreSetNeedsCommit<'_> {
    fn drop(&mut self) {
        *self.value.write().unwrap() = self.old;
    }
}
impl<B: MainLayerTypes> Layer<B> {
    // Source: cc/layers/layer.cc:1445-1462. Native diagnostic formatting.
    pub fn ToString(&self, c: &LayerContext<'_, B>) -> String {
        let bounds = self.bounds(c);
        let offset = self.offset_to_transform_parent(c);
        format!(
            "layer_id: {}\n  name: {}\n  Bounds: {}x{}\n  ElementId: {}\n  HitTestOpaqueness: {}\n  OffsetToTransformParent: {}\n  clip_tree_index: {}\n  effect_tree_index: {}\n  scroll_tree_index: {}\n  transform_tree_index: {}\n",
            self.id(),
            self.DebugName(c),
            bounds.width(),
            bounds.height(),
            self.element_id(c).ToString(),
            crate::cc::input::hit_test_opaqueness::HitTestOpaquenessToString(
                self.hit_test_opaqueness(c)
            ),
            c.Services().Vector2dFToString(offset),
            self.clip_tree_index(c),
            self.effect_tree_index(c),
            self.scroll_tree_index(c),
            self.transform_tree_index(c)
        )
    }
    /// Rust-only scoped synchronization view, not a cached thread/host state.
    pub fn Synchronization<'a, 'c>(
        &self,
        c: &'a LayerContext<'c, B>,
    ) -> impl ProtectedSequenceSynchronizer + 'a + use<'a, 'c, B> {
        self.sync(c)
    }
}
impl<B: MainLayerTypes> LayerRef<B> {
    // Source: cc/layers/layer.h:276-276. Dispatch the most-derived native slot.
    pub fn SetNeedsDisplay(&self, c: &mut LayerContext<'_, B>) {
        let bounds = self.bounds(c);
        let rect = Rect::new(0, 0, bounds.width(), bounds.height());
        drop(bounds);
        self.0.SetNeedsDisplayRect(&rect, c);
    }
    pub fn SetNeedsDisplayRect(&self, rect: &Rect, c: &mut LayerContext<'_, B>) {
        self.0.SetNeedsDisplayRect(rect, c);
    }
    pub fn SetBackgroundColor(&self, value: Color4f, c: &mut LayerContext<'_, B>) {
        self.0.SetBackgroundColor(value, c);
    }
    pub fn SetOpacity(&self, value: f32, c: &mut LayerContext<'_, B>) {
        self.0.SetOpacity(value, c);
    }
    pub fn RequiresSetNeedsDisplayOnHdrHeadroomChange(&self, c: &LayerContext<'_, B>) -> bool {
        self.0.RequiresSetNeedsDisplayOnHdrHeadroomChange(c)
    }
    pub fn GetPicture(&self, c: &mut LayerContext<'_, B>) -> Option<Arc<B::Picture>> {
        self.0.GetPicture(c)
    }
    pub fn Update(&self, c: &mut LayerContext<'_, B>) -> bool {
        self.0.Update(c)
    }
    pub fn MayUpdateAfterPaintEvent(&self) -> bool {
        self.0.MayUpdateAfterPaintEvent()
    }
    pub fn UpdateAfterPaintEvent(&self, c: &mut LayerContext<'_, B>) -> bool {
        self.0.UpdateAfterPaintEvent(c)
    }
    pub fn IsSnappedToPixelGridInTarget(&self) -> bool {
        self.0.IsSnappedToPixelGridInTarget()
    }
    pub fn IsScrollbarLayerForTesting(&self) -> bool {
        self.0.IsScrollbarLayerForTesting()
    }
    pub fn IsSolidColorLayerForTesting(&self) -> bool {
        self.0.IsSolidColorLayerForTesting()
    }
    pub fn OpacityCanAnimateOnImplThread(&self) -> bool {
        self.0.OpacityCanAnimateOnImplThread()
    }
    pub fn ViewTransitionResourceId(
        &self,
    ) -> crate::adapter::property_types::ViewTransitionElementResourceId {
        self.0.ViewTransitionResourceId()
    }
    pub fn CaptureContent(
        &self,
        rect: &Rect,
        content: &mut Vec<B::NodeInfo>,
        c: &mut LayerContext<'_, B>,
    ) {
        self.0.CaptureContent(rect, content, c);
    }
    pub fn RunMicroBenchmark(
        &self,
        benchmark: &mut B::MicroBenchmark,
        c: &mut LayerContext<'_, B>,
    ) {
        self.0.RunMicroBenchmark(benchmark, c);
    }
    pub fn CreateLayerImpl(
        &self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        tree_context: &mut crate::cc::trees::layer_tree_impl::LayerTreeContext<'_, B>,
        c: &mut LayerContext<'_, B>,
    ) -> Box<B::Layer> {
        self.0.CreateLayerImpl(tree, tree_context, c)
    }
    pub fn PushDirtyPropertiesTo(
        &self,
        target: &mut B::Layer,
        dirty: u8,
        commit: &mut crate::cc::trees::commit_state::CommitState<B>,
        c: &mut LayerContext<'_, B>,
        impl_tree: &mut B::ImplContext<'_>,
    ) {
        self.0
            .PushDirtyPropertiesTo(target, dirty, commit, c, impl_tree);
    }
    pub fn PushPropertiesTo(
        &self,
        target: &mut B::Layer,
        commit: &mut crate::cc::trees::commit_state::CommitState<B>,
        c: &mut LayerContext<'_, B>,
        impl_tree: &mut B::ImplContext<'_>,
    ) {
        self.0
            .base()
            .PushPropertiesTo(self.0.as_ref(), target, commit, c, impl_tree);
    }
}

impl<B: MainLayerTypes> Layer<B> {
    // Source: cc/layers/layer.h:119-119. Writable raw parent query has the source wait.
    pub fn mutable_parent(&self, c: &LayerContext<'_, B>) -> Option<LayerBorrow<B>> {
        self.parent_
            .Write(&self.sync(c))
            .as_ref()
            .and_then(Weak::upgrade)
            .map(LayerBorrow)
    }
    // Source: cc/layers/layer.h:812-814.
    pub fn set_property_tree_sequence_number(&self, sequence: i32, c: &LayerContext<'_, B>) {
        *self.property_tree_sequence_number_.Write(&self.sync(c)) = sequence;
    }
    // Source: cc/layers/layer.h:815-817.
    pub fn property_tree_sequence_number(&self, c: &LayerContext<'_, B>) -> i32 {
        *self.property_tree_sequence_number_.Read(&self.sync(c))
    }
    // Source: cc/layers/layer.h:384-386. Rust overload name preserves its gfx input.
    pub fn SetBackdropFilterBoundsRRect(
        &self,
        bounds: &crate::adapter::geometry::RRectF,
        c: &LayerContext<'_, B>,
    ) {
        self.SetBackdropFilterBounds(&B::Path::FromRRect(bounds), c);
    }
}
impl<B: MainLayerTypes> LayerBorrow<B> {
    /// The native raw pointer is usable only within this synchronous source
    /// scope. The higher-ranked lifetime prevents a resident reference escaping.
    pub fn WithSource<R>(
        &self,
        c: &mut LayerContext<'_, B>,
        f: impl for<'s> FnOnce(&'s dyn LayerObject<B>, &'s mut LayerContext<'_, B>) -> R,
    ) -> R {
        f(self.object(c), c)
    }
}
// Source: cc/layers/layer.h:988-989. AddClipChild/RemoveClipChild are declarations
// with no definitions in this Chromium stem and no call sites; no Rust body is
// invented for these unused C++ declarations.
// Source: cc/layers/layer.cc:47-83; cc/layers/layer.h:1271-1272.
// SameSizeAsLayer is a C++ ABI-only diagnostic (including raw_ptr/unique_ptr and
// bitfield padding), not a second runtime Layer. Rust's scoped guards/vtable and
// per-field synchronization have a different ABI, so that C++ sizeof assertion
// has no Rust representation. g_next_layer_id remains the injected atomic
// sequence capability; kIdentityTransform/kNoRoundedCornersF and external Empty
// constants are the absent ReadOrDefault branches. Their native identity/zero
// defaults are value semantics; present fields always stay borrowed resident.
impl<B: MainLayerTypes> Clone for LayerBorrow<B> {
    // Native raw pointer copy; deliberately leaves the native owner count alone.
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
impl<B: MainLayerTypes> PartialEq for LayerBorrow<B> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl<B: MainLayerTypes> Eq for LayerBorrow<B> {}
impl<B: MainLayerTypes> LayerBorrow<B> {
    pub fn WithSourceRef<R>(
        &self,
        c: &LayerContext<'_, B>,
        f: impl for<'s> FnOnce(&'s dyn LayerObject<B>, &'s LayerContext<'_, B>) -> R,
    ) -> R {
        f(self.object(c), c)
    }
}

impl<B: MainLayerTypes> LayerBorrow<B> {
    /// Native raw Layer* -> scoped_refptr assignment. Only this actual live
    /// allocation gains one native owner; a borrow cannot resurrect Shutdown.
    pub fn ToSourceRef(&self, c: &LayerContext<'_, B>) -> LayerRef<B> {
        self.object(c)
            .base()
            .native_owners_
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| {
                if n == 0 {
                    None
                } else {
                    n.checked_add(1)
                }
            })
            .expect("cannot acquire an owner of a destroyed Layer");
        LayerRef(Arc::clone(&self.0))
    }
}
