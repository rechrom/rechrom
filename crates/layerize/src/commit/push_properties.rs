//! The push target of the main/impl commit: the LayerImpl operations that
//! main Layer::PushPropertiesTo performs during LayerTreeHostImpl::FinishCommit
//! and the values it pushes. Moved from compositor/src/adapter/main_layer.rs
//! and compositor/src/cc/layers/layer.rs, which re-export them.
//! Source: cc/layers/layer.cc:1510-1622 (PushPropertiesTo, PushDirtyProperties,
//!         CreateLayerImpl); cc/layers/layer.h:63-80 (LayerDebugInfo).
use super::{
    commit_state::CommitTypes,
    geometry::{Rect, Size, Vector2dF},
};
use compositor_draw::Color4f;
use compositor_input::{
    hit_test_opaqueness::HitTestOpaqueness, input_types::ElementId,
    scroll_hit_test_rect::ScrollHitTestRect, touch_action_region::TouchActionRegion,
};
use compositor_tree::{region::Region, region_backend::SkRegionBackend};

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

/// Default is the native empty singleton value (capture/tracked rects); it
/// never copies a nonempty resident payload.
pub trait LayerValue: Clone + Default + PartialEq + Send + Sync {
    fn IsEmpty(&self) -> bool;
}

/// The concrete types named by the LayerImpl setters that
/// Layer::PushPropertiesTo calls: the push target (`CommitTypes::Layer`), the
/// impl-tree borrow it writes through and the pushed rare-input values.
pub trait PushPropertiesTypes:
    CommitTypes<Layer: MainLayerImpl<Self>, RegionBackend: SkRegionBackend + Send + Sync + 'static>
{
    /// Native cc::NodeId, with kInvalidNodeId as Default.
    type NodeId: Copy + Default + Send + Sync + 'static;
    type RegionCaptureBounds: LayerValue + 'static;
    type TrackedElementRects: LayerValue + 'static;
    /// Native PaintFlags::FilterQuality, with kLow as Default.
    type FilterQuality: Copy + Default + PartialEq + Send + Sync + 'static;
    /// Native DynamicRangeLimitMixture, with kHigh as Default.
    type DynamicRangeLimit: Copy + Default + PartialEq + Send + Sync + 'static;
    /// Current impl-side tree/services borrow. Native Layer::PushDirtyProperties
    /// receives the actual target tree; an implementation must split its layer
    /// owner from the remaining resident tree fields before constructing this.
    /// Never retained by either main Layer or the target LayerImpl.
    type ImplContext<'a>: MainLayerImplTree<Self>
    where
        Self: 'a;
}

/// Exact live LayerImpl operations consumed by cc/layers/layer.cc:1510-1622.
/// This boundary owns no LayerImpl state and makes no push decisions.
/// Scoped owner facet for the target resident LayerImpl. The implementing
/// borrow excludes that target layer; no whole tree/backpointer is retained.
pub trait MainLayerImplTree<B: PushPropertiesTypes> {
    fn source_frame_number(&self) -> i32;
}
pub trait MainLayerImpl<B: PushPropertiesTypes>: Sized {
    fn SetTransformTreeIndex(&mut self, index: i32, impl_tree: &mut B::ImplContext<'_>);
    fn SetHasTransformNode(&mut self, value: bool, impl_tree: &mut B::ImplContext<'_>);
    fn SetEffectTreeIndex(&mut self, index: i32, impl_tree: &mut B::ImplContext<'_>);
    fn SetClipTreeIndex(&mut self, index: i32, impl_tree: &mut B::ImplContext<'_>);
    fn SetScrollTreeIndex(&mut self, index: i32, impl_tree: &mut B::ImplContext<'_>);
    fn SetElementId(&mut self, element: ElementId, impl_tree: &mut B::ImplContext<'_>);
    fn SetBackgroundColor(&mut self, color: Color4f, impl_tree: &mut B::ImplContext<'_>);
    fn SetSafeOpaqueBackgroundColor(&mut self, color: Color4f, impl_tree: &mut B::ImplContext<'_>);
    fn SetBounds(&mut self, bounds: &Size, impl_tree: &mut B::ImplContext<'_>);
    fn SetOffsetToTransformParent(&mut self, offset: Vector2dF, impl_tree: &mut B::ImplContext<'_>);
    fn SetDrawsContent(&mut self, value: bool, impl_tree: &mut B::ImplContext<'_>);
    fn SetHitTestOpaqueness(&mut self, value: HitTestOpaqueness, impl_tree: &mut B::ImplContext<'_>);
    fn NoteLayerPropertyChanged(&mut self, impl_tree: &mut B::ImplContext<'_>);
    fn SetTouchActionRegion(
        &mut self,
        value: &TouchActionRegion<Region<B::RegionBackend>>,
        impl_tree: &mut B::ImplContext<'_>,
    );
    fn SetContentsOpaque(&mut self, value: bool, impl_tree: &mut B::ImplContext<'_>);
    fn SetContentsOpaqueForText(&mut self, value: bool, impl_tree: &mut B::ImplContext<'_>);
    fn SetShouldCheckBackfaceVisibility(&mut self, value: bool, impl_tree: &mut B::ImplContext<'_>);
    fn LifecycleAllowsPropertyTreeAccess(&self, impl_tree: &mut B::ImplContext<'_>) -> bool;
    fn UnionUpdateRect(&mut self, rect: &Rect, impl_tree: &mut B::ImplContext<'_>);
    fn UpdateDebugInfo(
        &mut self,
        info: Option<&mut LayerDebugInfo<B::NodeId>>,
        impl_tree: &mut B::ImplContext<'_>,
    );
    fn SetFilterQuality(&mut self, value: B::FilterQuality, impl_tree: &mut B::ImplContext<'_>);
    fn SetDynamicRangeLimit(
        &mut self,
        value: B::DynamicRangeLimit,
        impl_tree: &mut B::ImplContext<'_>,
    );
    fn SetMainThreadScrollHitTestRegion(
        &mut self,
        value: &Region<B::RegionBackend>,
        impl_tree: &mut B::ImplContext<'_>,
    );
    fn SetNonCompositedScrollHitTestRects(
        &mut self,
        value: &[ScrollHitTestRect],
        impl_tree: &mut B::ImplContext<'_>,
    );
    fn SetCaptureBounds(
        &mut self,
        value: &B::RegionCaptureBounds,
        impl_tree: &mut B::ImplContext<'_>,
    );
    fn SetTrackedElementRects(
        &mut self,
        value: &B::TrackedElementRects,
        impl_tree: &mut B::ImplContext<'_>,
    );
    fn SetWheelEventHandlerRegion(
        &mut self,
        value: &Region<B::RegionBackend>,
        impl_tree: &mut B::ImplContext<'_>,
    );
    fn SetCanvasChildId(&mut self, value: ElementId, impl_tree: &mut B::ImplContext<'_>);
    fn ResetRareProperties(&mut self, impl_tree: &mut B::ImplContext<'_>);
    fn SetNeedsPushProperties(&mut self, value: u8, impl_tree: &mut B::ImplContext<'_>);
}
