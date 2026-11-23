//! Per-call live host/paint capabilities for the canonical main-thread Layer.
//! No host object or service pointer is stored in a Layer; native weak host
//! pointers are stable identities resolved only through the current context.
use super::{
    geometry::{LinearGradient, PointF, Rect, Vector2dF},
    input_types::ElementId,
    layer_tree_impl::{LayerTreeTypes, PropertyTreesFor},
};
use crate::cc::{
    base::protected_sequence_synchronizer::ProtectedSequenceSynchronizer, layers::layer::Layer,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MainLayerHostId(pub u64);
pub trait MainLayerTypes:
    'static
    + super::commit_state::CommitStateTypes
    + PushPropertiesTypes
    + layerize::commit::CommitTypes<
        Transform: Send + Sync + 'static,
        Filters: super::draw_property::DrawFilters + Send + Sync + 'static,
        Path: MainLayerPath + Send + Sync + 'static,
        CopyOutputRequest: MainLayerCopyOutputRequest<TaskRunner = Self::TaskRunner>
                               + Send
                               + Sync
                               + 'static,
    >
{
    type DidScrollCallback: LayerDidScrollCallback + Default + Send + Sync + 'static;
    type NodeInfo;
    type Picture;
    type LayerSelection;
    type MicroBenchmark;
    type TaskRunner: Clone;
    type DerivedServices: ?Sized;
}
pub trait LayerDidScrollCallback: Clone {
    fn is_null(&self) -> bool;
    fn Run(&self, offset: PointF, element: ElementId);
}
type SelfViewTransitionRequest<B> = <B as layerize::commit::CommitTypes>::ViewTransitionRequest;
pub trait MainLayerHostRead<B: MainLayerTypes>: ProtectedSequenceSynchronizer {
    fn device_scale_factor(&self) -> f32;
    fn device_viewport_rect(&self) -> Rect;
    fn IsMainThread(&self) -> bool;
    fn CommitRequested(&self) -> bool;
    fn SourceFrameNumber(&self) -> i32;
    fn GetId(&self) -> i32;
    fn recording_scale_factor(&self) -> f32;
    fn GetSettings(&self) -> &crate::cc::trees::layer_tree_settings::LayerTreeSettings;
    fn GetDebugState(&self) -> &crate::cc::debug::layer_tree_debug_state::LayerTreeDebugState;
    fn OuterViewportScrollElementId(&self) -> ElementId;
    fn LayerByElementId(&self, id: ElementId) -> Option<crate::cc::layers::layer::LayerBorrow<B>>;

    /// Source cc/trees/layer_tree_host.h:710. Actual resident layer lookup;
    /// the returned raw-pointer-equivalent cannot acquire native ownership.
    fn LayerById(&self, id: i32) -> Option<crate::cc::layers::layer::LayerBorrow<B>>;
    fn IsUsingLayerLists(&self) -> bool;
    fn in_paint_layer_contents(&self) -> bool;
    fn in_will_commit(&self) -> bool;
    fn MainThreadTaskRunner(&self) -> B::TaskRunner;
    fn property_trees(&self) -> &PropertyTreesFor<B>;
    fn IsElementAnimating(&self, element: ElementId) -> bool;
}
pub trait MainLayerHost<B: MainLayerTypes>: MainLayerHostRead<B> {
    fn CreateUIResource(
        &mut self,
        bitmap: <B as layerize::commit::CommitTypes>::UIResourceBitmap,
    ) -> super::layer_tree_impl::UIResourceId;
    fn DeleteUIResource(&mut self, id: super::layer_tree_impl::UIResourceId);
    fn AddViewTransitionRequest(&mut self, request: Box<SelfViewTransitionRequest<B>>);
    fn RegisterSelection(&mut self, selection: B::LayerSelection);
    fn RegisterViewportPropertyIds(
        &mut self,
        ids: crate::cc::trees::viewport_property_ids::ViewportPropertyIds,
    );
    fn SetNeedsUpdateLayers(&mut self);
    fn property_trees_mut(&mut self) -> &mut PropertyTreesFor<B>;
    fn RegisterLayer(
        &mut self,
        layer: &Layer<B>,
        allocation: crate::cc::layers::layer::LayerBorrow<B>,
    );
    fn UnregisterLayer(&mut self, layer: &Layer<B>);
    fn RegisterElement(&mut self, element: ElementId, layer: &Layer<B>);
    fn UnregisterElement(&mut self, element: ElementId, layer: &Layer<B>);
    fn AddLayerShouldPushProperties(&mut self, layer: &Layer<B>);
    fn SetNeedsCommit(&mut self);
    fn SetNeedsFullTreeSync(&mut self);
    fn SetHasCopyRequest(&mut self, value: bool);
    fn DropActiveScrollDeltaNextCommit(&mut self, element: ElementId);
}
pub trait MainLayerReadHosts<B: MainLayerTypes> {
    fn Host(&self, id: MainLayerHostId) -> &dyn MainLayerHostRead<B>;
}
pub trait MainLayerHosts<B: MainLayerTypes>: MainLayerReadHosts<B> {
    fn HostMut(&mut self, id: MainLayerHostId) -> &mut dyn MainLayerHost<B>;
}
pub trait MainLayerServices {
    fn TraceLayer(&self, event: &str, element: Option<ElementId>);
    /// Native gfx::Vector2dF::ToString/printf %g formatting, not compositor policy.
    fn Vector2dFToString(&self, value: Vector2dF) -> String;
    fn NextLayerIdSequenceValue(&self) -> i32;
    fn IsCCSlimmingEnabled(&self) -> bool;
    fn RasterInducingScrollEnabled(&self) -> bool;
    fn ScrollingContentsCullRectOnScrollNodeEnabled(&self) -> bool;
}
pub struct LayerContext<'a, B: MainLayerTypes> {
    hosts: Option<LayerHosts<'a, B>>,
    pub services: Option<&'a dyn MainLayerServices>,
    pub derived_services: Option<&'a B::DerivedServices>,
    pub(crate) destruction_layer: Option<i32>,
}
pub enum LayerHosts<'a, B: MainLayerTypes> {
    Read(&'a dyn MainLayerReadHosts<B>),
    Write(&'a mut dyn MainLayerHosts<B>),
}
impl<'a, B: MainLayerTypes> LayerContext<'a, B> {
    pub fn new(hosts: &'a mut dyn MainLayerHosts<B>, services: &'a dyn MainLayerServices) -> Self {
        Self {
            hosts: Some(LayerHosts::Write(hosts)),
            services: Some(services),
            derived_services: None,
            destruction_layer: None,
        }
    }
    pub fn new_read_only(
        hosts: &'a dyn MainLayerReadHosts<B>,
        services: &'a dyn MainLayerServices,
    ) -> Self {
        Self {
            hosts: Some(LayerHosts::Read(hosts)),
            services: Some(services),
            derived_services: None,
            destruction_layer: None,
        }
    }
    pub(crate) fn DetachedForDestruction(id: i32) -> Self {
        Self {
            hosts: None,
            services: None,
            derived_services: None,
            destruction_layer: Some(id),
        }
    }
    pub fn Host(&self, id: MainLayerHostId) -> &dyn MainLayerHostRead<B> {
        match self
            .hosts
            .as_ref()
            .expect("attached Layer requires a live current host capability")
        {
            LayerHosts::Read(hosts) => hosts.Host(id),
            LayerHosts::Write(hosts) => hosts.Host(id),
        }
    }
    pub fn HostMut(&mut self, id: MainLayerHostId) -> &mut dyn MainLayerHost<B> {
        match self
            .hosts
            .as_mut()
            .expect("attached Layer requires a live current host capability")
        {
            LayerHosts::Write(hosts) => hosts.HostMut(id),
            LayerHosts::Read(_) => {
                panic!("commit synchronization receives a read-only main Layer host")
            }
        }
    }
    pub fn WithDerivedServices(mut self, services: &'a B::DerivedServices) -> Self {
        self.derived_services = Some(services);
        self
    }
    pub fn DerivedServices(&self) -> &B::DerivedServices {
        self.derived_services
            .as_deref()
            .expect("derived source branch requires its injected live capability")
    }
    pub fn Services(&self) -> &dyn MainLayerServices {
        self.services
            .expect("this native Layer branch requires injected services")
    }
}
// External gfx value: ui/gfx/geometry/rounded_corners_f.h:18-96.
// Native constructor clamp and four scalar radii; no compositor decisions.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RoundedCornersF([f32; 4]);
impl RoundedCornersF {
    pub fn new(upper_left: f32, upper_right: f32, lower_right: f32, lower_left: f32) -> Self {
        Self(
            [upper_left, upper_right, lower_right, lower_left].map(|value| {
                if value > 8. * f32::EPSILON {
                    value
                } else {
                    0.
                }
            }),
        )
    }
    pub fn IsEmpty(&self) -> bool {
        self.0.iter().all(|radius| *radius == 0.)
    }
    pub fn radii(&self) -> [f32; 4] {
        self.0
    }
}

// Rust-only field borrow projections. The original guard stays held and both
// projections resolve the same field; neither projection allocates or copies it.
pub struct LiveFieldRead<G: std::ops::Deref, T: ?Sized> {
    guard: G,
    project: fn(&G::Target) -> &T,
}
impl<G: std::ops::Deref, T: ?Sized> LiveFieldRead<G, T> {
    pub fn new(guard: G, project: fn(&G::Target) -> &T) -> Self {
        Self { guard, project }
    }
}
impl<G: std::ops::Deref, T: ?Sized> std::ops::Deref for LiveFieldRead<G, T> {
    type Target = T;
    fn deref(&self) -> &T {
        (self.project)(&self.guard)
    }
}
pub struct LiveFieldWrite<G: std::ops::DerefMut, T: ?Sized> {
    guard: G,
    read: fn(&G::Target) -> &T,
    write: fn(&mut G::Target) -> &mut T,
}
impl<G: std::ops::DerefMut, T: ?Sized> LiveFieldWrite<G, T> {
    pub fn new(guard: G, read: fn(&G::Target) -> &T, write: fn(&mut G::Target) -> &mut T) -> Self {
        Self { guard, read, write }
    }
}
impl<G: std::ops::DerefMut, T: ?Sized> std::ops::Deref for LiveFieldWrite<G, T> {
    type Target = T;
    fn deref(&self) -> &T {
        (self.read)(&self.guard)
    }
}
impl<G: std::ops::DerefMut, T: ?Sized> std::ops::DerefMut for LiveFieldWrite<G, T> {
    fn deref_mut(&mut self) -> &mut T {
        (self.write)(&mut self.guard)
    }
}

pub use layerize::commit::layer_value_backend::{MainLayerCopyOutputRequest, MainLayerPath};

// External gfx::MaskFilterInfo value constructor (ui/gfx/geometry/mask_filter_info.h).
// This constructs the requested owned value; source decisions remain above.
pub fn MaskFilterInfoForLayer(
    rect: crate::adapter::geometry::RectF,
    corners: RoundedCornersF,
    gradient: &LinearGradient,
) -> crate::adapter::property_types::MaskFilterInfo {
    crate::adapter::property_types::MaskFilterInfo {
        rounded_corner_bounds: crate::adapter::geometry::RRectF {
            rect,
            corner_radii: corners.radii().map(|r| Vector2dF::new(r, r)),
        },
        gradient_mask: if gradient.steps.is_empty() {
            None
        } else {
            Some(gradient.clone())
        },
        clip_id: None,
    }
}
pub use layerize::commit::layer_impl_factory::LayerImplFactory;
pub use layerize::commit::push_properties::{LayerValue, MainLayerImpl, MainLayerImplTree, PushPropertiesTypes};

