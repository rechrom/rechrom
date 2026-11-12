//! Same-Page native LocalFrame owners. This is Frame/FrameTree assembly, not a
//! second Page or a projection of DOM/Layout. Source frame.cc:520-560,591-641;
//! web_local_frame_impl.cc:2391-2517; local_frame.cc:425-507.
use super::*;
use dom::{
    content_frames::{ContentFrameSource, ContentFrameState},
    frame_lifecycle::FrameLifecycleState,
    frame_owner_loading::{FrameOwnerLoadError, FrameOwnerPolicy},
    persistent_document::DOMOwnerHandle,
};
use interaction::hit_test::{
    core::layout::{
        hit_test_location::HitTestLocation,
        hit_test_result::HitTestResultRequest,
        layout_embedded_content_hit_test::{
            ChildFrameHitTestResult, LocalChildFrameFacts, LocalChildFrameHitTest,
        },
        native_hit_test::NativeHitTestRequiredPort,
    },
    resident::{
        child_frame_query::{
            LocalChildFrameIdentity, PreparedLocalChildFrameQueries, PreparedLocalChildLayoutView,
        },
        layout_view_query::{PreparedLayoutViewLifecycle, PreparedResidentLayoutView},
        query_services::{ResidentQueryServices, ResidentQueryViewport},
    },
};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_FRAME_TOKEN: AtomicU64 = AtomicU64::new(1);
fn NewFrameToken() -> u64 {
    NEXT_FRAME_TOKEN.fetch_add(1, Ordering::Relaxed)
}

/// One local-root Frame registry; entries are the original native owners.
/// The sibling links below are FrameTree's links, never a copied DOM tree.
pub(crate) struct LocalFrameDomain {
    pub(super) page: LocalFramePageOwner,
    pub root: u64,
    pub(super) focus_identity: Rc<()>,
    pub(super) form_state_tasks:
        RefCell<std::collections::VecDeque<(Weak<LocalFrameRuntime>, Box<dyn FnOnce()>)>>,
    pub(super) focus_frames: RefCell<HashMap<u64, Rc<super::local_frame_focus::NativeFocusFrame>>>,
    pub(super) focus_controller:
        RefCell<Option<Rc<interaction::default::core::page::focus_controller::FocusController>>>,
    pub(super) navigation_lifecycle: Rc<
        interaction::default::core::loader::frame_navigation_lifecycle::FrameNavigationLifecycle,
    >,
    pub(super) frames: RefCell<HashMap<u64, Rc<LocalFrame>>>,
    pub(super) root_view: Rc<LocalFrameView>,
    pub(super) root_history_restore: RefCell<Option<Arc<navigation_history::HistoryItemSnapshot>>>,
    pub(super) session_entry_id: Cell<Option<i64>>,
    pub(super) session_history_state: Cell<navigation_history::SessionHistoryState>,
    pub(super) root_navigation: RefCell<Weak<super::navigation_api::NavigationWindowEnvironment>>,
    pub(super) root_anchor_navigation: std::cell::OnceCell<
        Rc<super::navigation_api::anchor_navigation::NativeAnchorNavigationHost>,
    >,
    first_child: Cell<Option<u64>>,
    last_child: Cell<Option<u64>>,
    pub(super) dirty: Cell<bool>,
    pub(super) script_factory:
        std::cell::OnceCell<Rc<super::local_frame_navigation::LocalFrameScriptFactory>>,
    pub(super) intersection_tasks:
        RefCell<std::collections::VecDeque<(Weak<DocumentEngine>, Box<dyn FnOnce()>)>>,
    pub(super) intersection_error: RefCell<Option<FrameOwnerLoadError>>,
    pub(super) intersection_deadline: Cell<Option<std::time::Instant>>,
    pub(super) effective_connection_type:
        Cell<foundation::network_connection_type::WebEffectiveConnectionType>,
    pub(super) loading_wake: RefCell<Option<url_loader::URLWakeCallback>>,
}
/// The Frame tree survives a root Document swap. Its current assembly root
/// changes atomically when DocumentLoader publishes the replacement owner.
#[derive(Clone)]
pub(super) struct LocalFramePageOwner(Rc<RefCell<Weak<PageState>>>);
impl LocalFramePageOwner {
    pub(super) fn upgrade(&self) -> Option<Rc<PageState>> {
        self.0.borrow().upgrade()
    }
    pub(super) fn Downgrade(&self) -> Weak<PageState> {
        self.0.borrow().clone()
    }
    pub(super) fn Rebind(&self, page: Weak<PageState>) {
        *self.0.borrow_mut() = page;
    }
}
pub(super) struct LocalFrame {
    pub(super) state: ContentFrameState,
    pub(super) domain: Weak<LocalFrameDomain>,
    pub(super) parent: u64,
    pub(super) navigation_lifecycle: Rc<
        interaction::default::core::loader::frame_navigation_lifecycle::FrameNavigationLifecycle,
    >,
    pub(super) owner_document: Weak<DocumentEngine>,
    previous: Cell<Option<u64>>,
    next: Cell<Option<u64>>,
    first_child: Cell<Option<u64>>,
    last_child: Cell<Option<u64>>,
    pub name: RefCell<Rc<[u16]>>,
    pub(super) unique_name: RefCell<navigation_history::unique_name::UniqueNameHelper>,
    pub(super) generation: Cell<u64>,
    pub(super) runtime: RefCell<Option<Rc<LocalFrameRuntime>>>,
    pub(super) pending_policy: RefCell<Option<FrameOwnerPolicy>>,
    pub(super) loading_client:
        std::cell::OnceCell<Rc<dyn dom::frame_owner_loading::FrameOwnerLoadingClient>>,
    pub(super) navigation: RefCell<super::local_frame_navigation::LocalFrameNavigation>,
    pub(super) anchor_navigation: std::cell::OnceCell<
        Rc<super::navigation_api::anchor_navigation::NativeAnchorNavigationHost>,
    >,
    view: std::cell::OnceCell<Rc<LocalFrameView>>,
    pub(super) window_performance_timing:
        RefCell<Rc<webapi::window_bindings::WindowPerformanceTiming>>,
}
/// LocalFrame's actual Document, FrameLoader and LocalFrameView owners. The
/// local root alone owns PaintController/Compositor; children own native Layout.
pub(super) struct LocalFrameRuntime {
    pub document: Rc<DocumentEngine>,
    pub(super) window_credentialless: bool,
    pub loader: Option<RefCell<document_loader::DocumentLoader>>,
    pub(super) navigation_state: std::cell::OnceCell<Rc<document_loader::DocumentLoaderNavigationState>>,
    pub resources: Rc<ResourceFetcher>,
    pub style: RefCell<style::StyleEngine>,
    pub layout: RefCell<layoutng_assembly::layout_engine::LayoutEngine>,
    pub constraints: Rc<RefCell<ConstraintSpace>>,
    pub view: Rc<LocalFrameView>,
    hit_test: interaction::hit_test::core::layout::layout_view::LayoutViewHitTestState,
    pub editing: Rc<layoutng_assembly::editing_state::LayoutEditingState>,
    pub interaction_state: Rc<RefCell<UserInteractionState>>,
    pub(super) interaction_scroll: RefCell<Option<super::local_frame_scroll::ChildInteractionScroll>>,
    pub(super) compositor_scroll_pending: Cell<bool>,
    pub(super) blocking_wheel_registry_dirty: Cell<bool>,
    pub(super) programmatic_scroll_animators: std::cell::OnceCell<interaction::scroll::core::scroll::programmatic_scroll_animator::ProgrammaticScrollAnimatorRegistry>,
    pub(super) frame_interaction:
        std::cell::OnceCell<super::local_frame_focus::FrameInteraction>,
    pub pending_text_overflow_layout_invalidations: RefCell<Vec<u64>>,
    pub(super) positions: std::cell::OnceCell<Rc<ChildResidentEditingPositions>>,
}
pub(super) struct LocalFrameView {
    pub size: Cell<foundation::gfx::Size>,
    pub revision: Cell<u64>,
    pub should_throttle: Cell<bool>,
    // LocalFrameView::first_layout_ and lifecycle_updates_throttled_ are
    // distinct from empty layout output and rendering throttling.
    pub first_layout: Cell<bool>,
    pub lifecycle_updates_throttled: Cell<bool>,
    pub needs_occlusion_tracking: Cell<bool>,
    pub rect_in_parent: Cell<foundation::PhysicalRect>,
    pub rect_in_parent_for_iov2: Cell<foundation::PhysicalRect>,
    pub rect_in_parent_stable_since: Cell<i64>,
    pub rect_in_parent_stable_since_for_iov2: Cell<i64>,
}
impl LocalFrameView {
    pub(super) fn new() -> Self {
        Self {
            size: Cell::new(foundation::gfx::Size::default()),
            revision: Cell::new(1),
            should_throttle: Cell::new(false),
            first_layout: Cell::new(true),
            lifecycle_updates_throttled: Cell::new(false),
            needs_occlusion_tracking: Cell::new(false),
            rect_in_parent: Default::default(),
            rect_in_parent_for_iov2: Default::default(),
            rect_in_parent_stable_since: Cell::new(0),
            rect_in_parent_stable_since_for_iov2: Cell::new(0),
        }
    }
    pub(super) fn DidFirstLayout(&self) -> bool {
        !self.first_layout.get()
    }
    pub(super) fn LifecycleUpdatesActive(&self) -> bool {
        !self.lifecycle_updates_throttled.get()
    }
    pub(super) fn DidLayout(&self) {
        self.first_layout.set(false);
    }
    /// FrameView::UpdateViewportIntersection, frame_view.cc:125-168. The
    /// default feature parameters are distance_factor=DBL_MAX, time_ms=0;
    /// IOv2 always retains its distinct 30 DIP / 500 ms policy.
    pub(super) fn UpdateRectInParent(&self, rect: foundation::PhysicalRect, now: i64) {
        let old = self.rect_in_parent.get();
        if old.size != rect.size {
            self.rect_in_parent.set(rect);
            self.rect_in_parent_stable_since.set(now);
        }
        let old = self.rect_in_parent_for_iov2.get();
        if old.size != rect.size
            || (rect.X() - old.X()).ToDouble().abs() + (rect.Y() - old.Y()).ToDouble().abs() > 30.0
        {
            self.rect_in_parent_for_iov2.set(rect);
            self.rect_in_parent_stable_since_for_iov2.set(now);
        }
    }
}
pub(super) struct LocalFramePaint {
    frame: Rc<LocalFrame>,
    pub runtime: Rc<LocalFrameRuntime>,
    pub output: Option<LayoutPaintOutput>,
    lifecycle: PagePaintLifecycle,
    parent_index: Option<usize>,
    is_cross_origin: bool,
    blocking_wheel_targets: Arc<[u64]>,
}
pub(super) struct LocalFrameQueries<'a> {
    pub domain: &'a LocalFrameDomain,
    pub paint: &'a paint::PaintEngine,
}
struct CurrentLocalFrameQuery<'a> {
    queries: &'a LocalFrameQueries<'a>,
    frame: Rc<LocalFrame>,
    generation: u64,
}
impl PreparedLocalChildFrameQueries for LocalFrameQueries<'_> {
    fn ForFrame(
        &self,
        frame_identity: u64,
    ) -> Result<Box<dyn LocalChildFrameHitTest + '_>, NativeHitTestRequiredPort> {
        let frame = self
            .domain
            .Frame(frame_identity)
            .ok_or(NativeHitTestRequiredPort::LocalChildFrame { frame_identity })?;
        let generation = frame.generation.get();
        Ok(Box::new(CurrentLocalFrameQuery {
            queries: self,
            frame,
            generation,
        }))
    }
}
impl CurrentLocalFrameQuery<'_> {
    fn Required(&self) -> NativeHitTestRequiredPort {
        NativeHitTestRequiredPort::LocalChildFrame {
            frame_identity: self.frame.state.Identity(),
        }
    }
    fn Validate(&self) -> Result<Rc<LocalFrameRuntime>, NativeHitTestRequiredPort> {
        if self.frame.generation.get() != self.generation
            || self.frame.state.Lifecycle().GetState() != FrameLifecycleState::Attached
        {
            return Err(self.Required());
        }
        self.frame
            .runtime
            .borrow()
            .clone()
            .ok_or_else(|| self.Required())
    }
}
impl LocalChildFrameHitTest for CurrentLocalFrameQuery<'_> {
    fn Facts(&self) -> Result<LocalChildFrameFacts, NativeHitTestRequiredPort> {
        let runtime = self.Validate()?;
        let doc = runtime.document.borrow();
        let state = doc.GetDocument().Lifecycle().GetState();
        let facts = LocalChildFrameFacts {
            should_throttle_rendering: runtime.view.should_throttle.get(),
            has_document: true,
            is_prepaint_clean: state >= dom::document_lifecycle::LifecycleState::PrePaintClean
                && state < dom::document_lifecycle::LifecycleState::Stopping,
            has_layout_view: runtime.layout.borrow().GetLayoutTree().is_some(),
        };
        Ok(facts)
    }
    fn HitTestNoLifecycleUpdate<'r>(
        &self,
        location: &HitTestLocation,
        request: &HitTestResultRequest<'r, ()>,
    ) -> Result<ChildFrameHitTestResult<'r>, NativeHitTestRequiredPort> {
        let runtime = self.Validate()?;
        let layout = runtime.layout.borrow();
        let result = layout.GetLayoutResult().ok_or_else(|| self.Required())?;
        let cursor = PaintFragmentCursor::FromHandle(result);
        let view = unsafe {
            foundation::DynamicTo::<layoutng_assembly::internal::layout_view::LayoutView>(
                layout
                    .GetLayoutTree()
                    .ok_or_else(|| self.Required())?
                    .Root(),
            )
            .as_ref()
        }
        .ok_or_else(|| self.Required())?;
        let output = layout.Output().ok_or_else(|| self.Required())?;
        let nodes = interaction::adapter::page_cursor::ResidentDOMCursorNodes::new(
            runtime.document.Handle(),
            output,
            cursor,
            self.queries,
            runtime
                .positions
                .get()
                .expect("child Frame editing query owner")
                .clone(),
            Arc::new(paint_skia::svg_shape_path::SkiaNativePathFactory),
        );
        let services = ResidentQueryServices::new(
            self.queries.paint,
            cursor,
            &nodes,
            self,
            &paint_skia::path_hit_test::SkiaPathHitTestFactory,
        );
        let lifecycle = CurrentLocalFrameView {
            query: self,
            runtime: &runtime,
        };
        let query = PreparedResidentLayoutView::new(
            view,
            cursor,
            &services,
            &lifecycle,
            &runtime.hit_test,
        )?;
        let result = query.HitTestNoLifecycleUpdate(location, request)?;
        nodes.CheckQuery().map_err(|_| self.Required())?;
        self.Validate()?;
        Ok(result)
    }
}
impl ResidentQueryViewport for CurrentLocalFrameQuery<'_> {
    fn RootFrameToVisualViewport(&self, point: foundation::gfx::Point) -> foundation::gfx::Point {
        point
    }
}
struct CurrentLocalFrameView<'a> {
    query: &'a CurrentLocalFrameQuery<'a>,
    runtime: &'a LocalFrameRuntime,
}
pub(super) struct ChildResidentEditingPositions {
    frame: Weak<LocalFrame>,
    runtime: Weak<LocalFrameRuntime>,
}
impl ChildResidentEditingPositions {
    fn Owners(&self) -> (Rc<LocalFrame>, Rc<LocalFrameRuntime>, Rc<LocalFrameDomain>) {
        let frame = self.frame.upgrade().expect("live child Frame query");
        let runtime = self.runtime.upgrade().expect("live child Document query");
        let domain = frame.domain.upgrade().expect("live child local root");
        (frame, runtime, domain)
    }
    fn Settings(
        domain: &LocalFrameDomain,
    ) -> interaction::adapter::page_pointer_hit_test::InteractionTextSettings {
        domain
            .page
            .upgrade()
            .expect("live child local root")
            .interaction_text_settings
            .borrow()
            .clone()
            .expect("embedding interaction text settings")
    }
}
impl interaction::adapter::resident_editing_positions::ResidentEditingPositionQueries
    for ChildResidentEditingPositions
{
    fn VisibleContentRectExcludingScrollbars(&self) -> foundation::gfx::RectF {
        let (_, runtime, _) = self.Owners();
        let result = runtime
            .layout
            .borrow()
            .Output()
            .expect("prepared child LayoutView")
            .CursorVisibleContentRectExcludingScrollbars()
            .expect("child LocalFrameView viewport");
        result
    }
    fn UpdateCursorStyleAndLayout(&self) {
        let (_, runtime, _) = self.Owners();
        assert!(
            runtime.layout.borrow().Output().is_some()
                && runtime
                    .document
                    .borrow()
                    .GetDocument()
                    .Lifecycle()
                    .GetState()
                    >= dom::document_lifecycle::LifecycleState::PrePaintClean,
            "child cursor requires its current prepared lifecycle"
        );
    }
    fn NodeOwner(
        &self,
        node: u64,
    ) -> Option<foundation::Persistent<layoutng_assembly::internal::layout_node_metadata::Node>>
    {
        let runtime = self.runtime.upgrade()?;
        let result = runtime
            .layout
            .borrow()
            .CurrentLayoutNode(node)?
            .NativeNodeOwner();
        result
    }
    fn PositionForPoint(
        &self,
        node: &foundation::Persistent<layoutng_assembly::internal::layout_node_metadata::Node>,
        point: &foundation::PhysicalOffset,
    ) -> Option<interaction::hit_test::core::layout::hit_test_result::HitTestPositionData> {
        let (_, runtime, domain) = self.Owners();
        let settings = Self::Settings(&domain);
        let document = runtime.document.borrow();
        let layout = runtime.layout.borrow();
        interaction::adapter::resident_editing_positions::QueryPositionForPoint(
            document.GetDocument(),
            layout.Output()?,
            unsafe { node.Get().as_ref() }?,
            point,
            layoutng_assembly::EditingPositionOptions {
                move_caret_to_horizontal_boundary_when_past_top_or_bottom: settings
                    .editing_behavior
                    .ShouldMoveCaretToHorizontalBoundaryWhenPastTopOrBottom(),
                ..Default::default()
            },
            &settings.ui_locale,
        )
    }
    fn CaretPosition(
        &self,
        node: u64,
        query: interaction::hit_test::core::layout::hit_test_result::CaretPositionQuery,
    ) -> Option<interaction::hit_test::core::layout::hit_test_result::HitTestPositionData> {
        let (_, runtime, domain) = self.Owners();
        let settings = Self::Settings(&domain);
        let document = runtime.document.borrow();
        let layout = runtime.layout.borrow();
        interaction::adapter::resident_editing_positions::QueryCaretPosition(
            document.GetDocument(),
            layout.Output()?,
            node,
            query,
            &settings.ui_locale,
        )
    }
    fn SelectionContains(&self, point: &foundation::PhysicalOffset) -> bool {
        let (frame, runtime, domain) = self.Owners();
        let settings = Self::Settings(&domain);
        let document = runtime.document.borrow();
        let layout = runtime.layout.borrow();
        let Some(output) = layout.Output() else {
            return false;
        };
        interaction::adapter::resident_editing_positions::QuerySelectionContains(
            document.GetDocument(),
            output,
            &runtime.editing.selections,
            &settings.ui_locale,
            point,
            || {
                use interaction::hit_test::core::layout::{
                    hit_test_request::RequestType,
                    hit_test_result::{HitTestPositionAnchor as A, HitTestTextAffinity as F},
                };
                let page = domain.page.upgrade().expect("child local root");
                let paint = page.paint_engine.borrow();
                let queries = LocalFrameQueries {
                    domain: &domain,
                    paint: &paint,
                };
                let query = CurrentLocalFrameQuery {
                    queries: &queries,
                    frame: frame.clone(),
                    generation: frame.generation.get(),
                };
                let location = HitTestLocation::FromPhysicalOffset(*point);
                let request = HitTestResultRequest::<()>::new(
                    RequestType::kReadOnly as u32 | RequestType::kActive as u32,
                    None,
                    None,
                );
                let result = query
                    .HitTestNoLifecycleUpdate(&location, &request)
                    .expect("prepared child FrameSelection active hit query");
                let p = result
                    .Result()
                    .GetPositionForInnerNodeOrImageMapImage()?
                    .Value();
                if p.document != document.GetDocument().RootHandle().Identity() {
                    return None;
                }
                Some(layoutng_assembly::EditingPositionWithAffinity {
                    position: layoutng_assembly::EditingPosition::WithAnchor(
                        p.node_id?,
                        p.offset,
                        match p.anchor {
                            A::OffsetInAnchor => {
                                layoutng_assembly::EditingPositionAnchorType::OffsetInAnchor
                            }
                            A::BeforeAnchor => {
                                layoutng_assembly::EditingPositionAnchorType::BeforeAnchor
                            }
                            A::AfterAnchor => {
                                layoutng_assembly::EditingPositionAnchorType::AfterAnchor
                            }
                            A::AfterChildren => {
                                layoutng_assembly::EditingPositionAnchorType::AfterChildren
                            }
                        },
                    ),
                    affinity: match p.affinity {
                        F::Downstream => layoutng_assembly::EditingTextAffinity::Downstream,
                        F::Upstream => layoutng_assembly::EditingTextAffinity::Upstream,
                    },
                })
            },
        )
    }
    fn UpdateSelectionStyleTree(&self, _node: u64) {
        let (_, runtime, _) = self.Owners();
        let document = runtime.document.borrow();
        assert!(
            document.GetDocument().Lifecycle().GetState()
                >= dom::document_lifecycle::LifecycleState::PrePaintClean,
            "child cursor admitted only after its own lifecycle preparation"
        );
    }
    fn ContentLocalToRootAncestorPoint(
        &self,
        point: foundation::gfx::PointF,
    ) -> foundation::gfx::PointF {
        let (frame, runtime, domain) = self.Owners();
        let point = runtime
            .layout
            .borrow()
            .Output()
            .expect("prepared child LayoutView")
            .CursorContentLocalToRootAncestorPoint(point);
        domain.MapChildPoint(&frame, point)
    }
    fn MapContentRectToRootAncestorSpace(
        &self,
        rect: foundation::gfx::RectF,
    ) -> foundation::gfx::RectF {
        let (frame, runtime, domain) = self.Owners();
        let rect = runtime
            .layout
            .borrow()
            .Output()
            .expect("prepared child LayoutView")
            .CursorMapContentRectToRootAncestorSpace(rect);
        domain.MapChildRect(&frame, rect)
    }
}
impl PreparedLayoutViewLifecycle for CurrentLocalFrameView<'_> {
    fn Identity(&self) -> LocalChildFrameIdentity {
        LocalChildFrameIdentity {
            frame: self.query.frame.state.Identity(),
            local_root: self.query.queries.domain.root,
            document_generation: self.query.frame.generation.get(),
        }
    }
    fn IsPrePaintClean(&self) -> bool {
        let doc = self.runtime.document.borrow();
        let state = doc.GetDocument().Lifecycle().GetState();
        state >= dom::document_lifecycle::LifecycleState::PrePaintClean
            && state < dom::document_lifecycle::LifecycleState::Stopping
    }
    fn PreparedRevision(&self) -> u64 {
        self.runtime.view.revision.get()
    }
    fn DomTreeVersion(&self) -> u64 {
        self.runtime
            .document
            .borrow()
            .GetDocument()
            .DomTreeVersion()
    }
    fn FrameSize(&self) -> Option<foundation::gfx::Size> {
        Some(self.runtime.view.size.get())
    }
    fn DocumentToFrameOffset(&self) -> foundation::PhysicalOffset {
        let layout = self.runtime.layout.borrow();
        let root = layout
            .GetLayoutTree()
            .expect("prepared child LayoutView")
            .Root();
        let box_ = unsafe {
            foundation::DynamicTo::<layoutng_assembly::internal::layout_box::LayoutBox>(root)
                .as_ref()
        }
        .expect("LayoutView base");
        -box_.ScrolledContentOffset()
    }
    fn HasActiveUnboundedElements(&self) -> bool {
        // There is no unbounded activation producer in this embedding. Its
        // source feature is disabled, so LayoutView never enters this branch.
        assert!(!foundation::RuntimeEnabledFeatures::UnboundedElementEnabled());
        false
    }
}
impl LocalFrameDomain {
    pub(super) fn WithParentLayout<T>(
        &self,
        frame: &LocalFrame,
        read: impl FnOnce(&layoutng_assembly::layout_engine::LayoutEngine) -> T,
    ) -> T {
        if frame.parent == self.root {
            let page = self.page.upgrade().expect("live local root");
            let value = read(&page.layout_engine.borrow());
            value
        } else {
            let parent = self.Frame(frame.parent).expect("native frame parent");
            let runtime = parent.runtime.borrow().clone().expect("parent Document");
            let value = read(&runtime.layout.borrow());
            value
        }
    }
    pub(super) fn MapChildPoint(
        &self,
        frame: &LocalFrame,
        point: foundation::gfx::PointF,
    ) -> foundation::gfx::PointF {
        let point = self.WithParentLayout(frame, |layout| {
            layout
                .Output()
                .expect("prepared parent LayoutView")
                .Node(frame.state.Owner().unwrap().Id())
                .expect("native embedded owner")
                .EmbeddedContentPointToDocument(point)
        });
        if frame.parent == self.root {
            point
        } else {
            self.MapChildPoint(&self.Frame(frame.parent).unwrap(), point)
        }
    }
    pub(super) fn MapChildRect(
        &self,
        frame: &LocalFrame,
        rect: foundation::gfx::RectF,
    ) -> foundation::gfx::RectF {
        let rect = self.WithParentLayout(frame, |layout| {
            layout
                .Output()
                .expect("prepared parent LayoutView")
                .Node(frame.state.Owner().unwrap().Id())
                .expect("native embedded owner")
                .EmbeddedContentRectToDocument(rect)
        });
        if frame.parent == self.root {
            rect
        } else {
            self.MapChildRect(&self.Frame(frame.parent).unwrap(), rect)
        }
    }
    /// LocalFrameView updates the original child documents after the parent
    /// layout has supplied their viewport geometry (local_frame_view.cc:4220-
    /// 4254). Only source owners/receipts travel to the local-root Paint pass.
    pub fn PreparePaint(
        &self,
        root: &layoutng_assembly::layout_engine::LayoutEngine,
    ) -> io::Result<Vec<LocalFramePaint>> {
        let page = self
            .page
            .upgrade()
            .ok_or_else(|| io::Error::other("local root detached"))?;
        let mut prepared = Vec::new();
        self.PrepareChildren(self.root, root, None, &page, &mut prepared)?;
        Ok(prepared)
    }
    fn PrepareChildren(
        &self,
        parent: u64,
        parent_layout: &layoutng_assembly::layout_engine::LayoutEngine,
        parent_index: Option<usize>,
        page: &PageState,
        prepared: &mut Vec<LocalFramePaint>,
    ) -> io::Result<()> {
        for frame in self.Children(parent) {
            let Some(runtime) = frame.runtime.borrow().clone() else {
                continue;
            };
            runtime.view.lifecycle_updates_throttled.set(
                !runtime.view.DidFirstLayout()
                    && !runtime
                        .document
                        .borrow()
                        .GetDocument()
                        .HaveRenderBlockingResourcesLoaded(),
            );
            if !runtime.view.LifecycleUpdatesActive() {
                continue;
            }
            let Some(owner) = frame.state.Owner() else {
                continue;
            };
            let Some(size) = parent_layout
                .Output()
                .and_then(|output| output.Node(owner.Id()))
                .and_then(|node| node.EmbeddedChildFrameSize())
            else {
                continue;
            };
            if runtime.view.should_throttle.get() {
                continue;
            }
            // FrameView owns the movement history. Use the same resident
            // IntersectionGeometry path as FrameView::UpdateViewportIntersection,
            // rather than substituting an embedded element's bounding box.
            let parent_document = self
                .Document(parent)
                .ok_or_else(|| io::Error::other("child parent Document detached"))?;
            let standardized_zoom = parent_document
                .borrow()
                .GetDocument()
                .StandardizedBrowserZoomEnabled();
            let geometry = parent_layout
                .ComputeIntersectionGeometryForNode(
                    owner.Id(),
                    None,
                    standardized_zoom,
                    intersection_observer::OcclusionState::Unknown,
                    &intersection_observer::IntersectionObserverParams {
                        thresholds: vec![f32::MIN_POSITIVE],
                        ..Default::default()
                    },
                    intersection_observer::geometry_flags::ROOT_IMPLICIT
                        | intersection_observer::geometry_flags::FRAME_VIEWPORT,
                    &mut None,
                    &mut Default::default(),
                )
                .map_err(|error| {
                    io::Error::other(format!("child FrameView intersection: {error:?}"))
                })?;
            let rect = geometry.intersection_rect;
            let scale = page.constraints.borrow().device_pixel_ratio as f32;
            let rect =
                foundation::PhysicalRect::FastAndLossyFromRectF(&foundation::gfx::RectF::new(
                    foundation::gfx::PointF::new(rect.x() / scale, rect.y() / scale),
                    foundation::gfx::SizeF::new(rect.width() / scale, rect.height() / scale),
                ));
            runtime
                .view
                .UpdateRectInParent(rect, page.InteractionEventClock().Now().ToInternalValue());
            if let Some(interaction) = self.FocusInteraction(frame.state.Identity()) {
                interaction::adapter::page_form_validation::RefreshControlValidity(
                    &runtime.document.Handle(),
                    &interaction.FormControlState(),
                )
                .map_err(|error| {
                    io::Error::other(format!(
                        "child constraint-validation service required: {error:?}"
                    ))
                })?;
            }
            runtime.UpdateFocusLayout(size, page)?;
            let mut output = runtime
                .layout
                .borrow_mut()
                .TakePaintOutput()
                .ok_or_else(|| io::Error::other("child LayoutView result required"))?;
            if runtime.blocking_wheel_registry_dirty.get() {
                runtime
                    .layout
                    .borrow_mut()
                    .RefreshOutputPrePaintInputs(&mut output);
            }
            let parent_document = self
                .Document(parent)
                .ok_or_else(|| io::Error::other("child parent document detached"))?;
            let is_cross_origin = !parent_document
                .borrow()
                .GetDocument()
                .SecurityContext()
                .Origin()
                .CanAccess(
                    runtime
                        .document
                        .borrow()
                        .GetDocument()
                        .SecurityContext()
                        .Origin()
                        .as_ref(),
                );
            let index = prepared.len();
            let lifecycle = runtime.document.borrow().GetDocument().Lifecycle().clone();
            prepared.push(LocalFramePaint {
                frame: frame.clone(),
                runtime: runtime.clone(),
                output: Some(output),
                lifecycle: PagePaintLifecycle(lifecycle),
                parent_index,
                is_cross_origin,
                blocking_wheel_targets: super::local_frame_scroll::InteractionScrollOwner::Child {
                    frame: Rc::downgrade(&frame),
                    runtime: Rc::downgrade(&runtime),
                }
                .BlockingWheelTargets(),
            });
            self.PrepareChildren(
                frame.state.Identity(),
                &runtime.layout.borrow(),
                Some(index),
                page,
                prepared,
            )?;
        }
        Ok(())
    }
    pub fn PaintInputs<'a>(
        root: &'a LayoutPaintOutput,
        root_layout: &'a layoutng_assembly::layout_engine::LayoutEngine,
        prepared: &'a [LocalFramePaint],
        layouts: &'a [std::cell::Ref<'_, layoutng_assembly::layout_engine::LayoutEngine>],
    ) -> Vec<paint::LocalFrameViewPaintInput<'a>> {
        prepared
            .iter()
            .map(|child| {
                let (output, layout) = child.parent_index.map_or((root, root_layout), |index| {
                    (prepared[index].output.as_ref().unwrap(), &*layouts[index])
                });
                let node = layout
                    .Output()
                    .unwrap()
                    .Node(child.frame.state.Owner().unwrap().Id())
                    .unwrap();
                paint::LocalFrameViewPaintInput {
                    owner: output
                        .PaintCursor()
                        .ForPaintObject(node.PaintObjectIdentity())
                        .expect("original embedded owner fragment"),
                    layout: child.output.as_ref().unwrap(),
                    is_cross_origin_to_parent: child.is_cross_origin,
                    blocking_wheel_targets: child.blocking_wheel_targets.clone(),
                    lifecycle: &child.lifecycle,
                }
            })
            .collect()
    }
    pub fn CommitPaint(
        &self,
        prepared: Vec<LocalFramePaint>,
        receipt: &paint::paint_engine::PaintOutput,
    ) {
        for mut child in prepared {
            child.runtime.layout.borrow_mut().CommitPaint(
                child.output.take().unwrap(),
                receipt.clone(),
                None,
            );
            child.runtime.document.borrow_mut().DidCommitPaint();
            if let Some(performance) = child
                .frame
                .window_performance_timing
                .borrow()
                .ExistingEventTimingSource()
            {
                performance.OnPaintFinished();
            }
            child.runtime.blocking_wheel_registry_dirty.set(false);
        }
        self.dirty.set(false);
    }
    pub fn ReturnPrePaint(prepared: Vec<LocalFramePaint>) {
        for mut child in prepared {
            child
                .runtime
                .layout
                .borrow_mut()
                .ReturnPrePaintOutput(child.output.take().unwrap());
        }
    }
    pub fn new(page: Weak<PageState>) -> Rc<Self> {
        Rc::new(Self {
            page: LocalFramePageOwner(Rc::new(RefCell::new(page))),
            root: NewFrameToken(),
            focus_identity: Rc::new(()),
            focus_frames: Default::default(),
            form_state_tasks: Default::default(),
            focus_controller: Default::default(),
            navigation_lifecycle: Default::default(),
            frames: Default::default(),
            root_view: Rc::new(LocalFrameView::new()),
            root_history_restore: Default::default(),
            session_entry_id: Cell::new(None),
            session_history_state: Default::default(),
            root_navigation: Default::default(),
            root_anchor_navigation: Default::default(),
            dirty: Cell::new(false),
            script_factory: Default::default(),
            intersection_tasks: Default::default(),
            intersection_error: Default::default(),
            intersection_deadline: Cell::new(None),
            effective_connection_type: Cell::new(
                foundation::network_connection_type::WebEffectiveConnectionType::Unknown,
            ),
            loading_wake: Default::default(),
            first_child: Cell::new(None),
            last_child: Cell::new(None),
        })
    }
    pub fn Frame(&self, token: u64) -> Option<Rc<LocalFrame>> {
        self.frames.borrow().get(&token).cloned()
    }
    pub fn IsVisible(&self) -> bool {
        self.page
            .upgrade()
            .is_some_and(|page| page.performance_visible.get())
    }
    pub fn NeedsLifecycle(&self) -> bool {
        self.dirty.get()
    }
    pub fn SubframeCount(&self) -> usize {
        self.frames.borrow().len()
    }
    pub fn DetachChildren(&self) {
        for child in self.Children(self.root) {
            child.DetachForRemove();
        }
    }
    pub fn RouteEffect(
        &self,
        effect: &dom::frame_owner_loading::FrameOwnerLoadEffect,
    ) -> Result<(), FrameOwnerLoadError> {
        use dom::frame_owner_loading::FrameOwnerLoadEffect as Effect;
        match effect {
            Effect::DidChangeAttributes {
                owner,
                frame_identity,
            } => {
                let frame = self
                    .Frame(*frame_identity)
                    .ok_or(FrameOwnerLoadError::UnexpectedCreatedFrame)?;
                if frame.state.Owner() != Some(*owner) {
                    return Err(FrameOwnerLoadError::UnexpectedCreatedFrame);
                }
                let doc = frame
                    .owner_document
                    .upgrade()
                    .ok_or(FrameOwnerLoadError::MissingOwner)?;
                let dom = doc.borrow();
                let doc = dom.GetDocument();
                let index = doc
                    .FindNodeById(owner.Id())
                    .ok_or(FrameOwnerLoadError::MissingOwner)?;
                let name: Rc<[u16]> = Rc::from(
                    doc.Node(index)
                        .FindAttribute("name")
                        .map_or_else(Vec::new, |a| a.value.encode_utf16().collect::<Vec<_>>()),
                );
                if *frame.name.borrow() != name {
                    *frame.name.borrow_mut() = name;
                    frame.UpdateUniqueName();
                }
                Ok(())
            }
            Effect::DidChangeFramePolicy {
                frame_identity,
                policy,
            } => {
                let frame = self
                    .Frame(*frame_identity)
                    .ok_or(FrameOwnerLoadError::UnexpectedCreatedFrame)?;
                *frame.pending_policy.borrow_mut() = Some(policy.clone());
                Ok(())
            }
            Effect::Navigate { .. }
            | Effect::StartNavigation { .. }
            | Effect::SrcDocChanged { .. } => self.RouteNavigationEffect(effect),
            _ => Err(FrameOwnerLoadError::HostEffect),
        }
    }
    pub fn Document(&self, token: u64) -> Option<Rc<DocumentEngine>> {
        if token == self.root {
            return Some(self.page.upgrade()?.document.clone());
        }
        self.Frame(token)?
            .runtime
            .borrow()
            .as_ref()
            .map(|r| r.document.clone())
    }
    pub fn Resources(&self, token: u64) -> Option<Rc<ResourceFetcher>> {
        if token == self.root {
            return self.page.upgrade()?.resources.borrow().clone();
        }
        self.Frame(token)?
            .runtime
            .borrow()
            .as_ref()
            .map(|r| r.resources.clone())
    }
    pub fn LoadType(&self, token: u64) -> Option<navigation_history::FrameLoadType> {
        if token == self.root {
            return self
                .page
                .upgrade()?
                .document_loader_owners
                .borrow()
                .as_ref()?
                .navigation
                .LoadType();
        }
        let frame = self.Frame(token)?;
        if let Some(load_type) = frame.navigation.borrow().LoadType() {
            return Some(load_type);
        }
        let load_type = frame
            .runtime
            .borrow()
            .as_ref()?
            .loader
            .as_ref()?
            .borrow()
            .NavigationState()
            .LoadType();
        load_type
    }
    fn AppendChild(&self, frame: &Rc<LocalFrame>) {
        let parent = self.Frame(frame.parent);
        let (first, last) = parent
            .as_ref()
            .map_or((&self.first_child, &self.last_child), |p| {
                (&p.first_child, &p.last_child)
            });
        frame.previous.set(last.get());
        if let Some(previous) = last.get().and_then(|id| self.Frame(id)) {
            previous.next.set(Some(frame.state.Identity()));
        } else {
            first.set(Some(frame.state.Identity()));
        }
        last.set(Some(frame.state.Identity()));
    }
    /// WebLocalFrameImpl::InitializeCoreFrameInternal: bind Frame::owner before
    /// Frame::Init/FrameLoader::Init installs the initial empty Document.
    pub fn Create(
        self: &Rc<Self>,
        parent: u64,
        owner: DOMOwnerHandle,
        name: &[u16],
        is_created_by_script: bool,
    ) -> Result<Rc<LocalFrame>, FrameOwnerLoadError> {
        let page = self
            .page
            .upgrade()
            .ok_or(FrameOwnerLoadError::MissingClient)?;
        let parent_document = self
            .Document(parent)
            .ok_or(FrameOwnerLoadError::ParentFrameFacts)?;
        let resources = self
            .Resources(parent)
            .ok_or(FrameOwnerLoadError::CreateLocalFrame)?;
        let policy = {
            let dom = parent_document.borrow();
            let doc = dom.GetDocument();
            let index = doc
                .FindNodeById(owner.Id())
                .ok_or(FrameOwnerLoadError::MissingOwner)?;
            if doc.NodeHandle(index) != owner {
                return Err(FrameOwnerLoadError::WrongOwnerDocument);
            }
            doc.FrameOwnerPolicy(index)
                .ok_or(FrameOwnerLoadError::ContainerPolicy)?
        };
        let frame = Rc::new(LocalFrame {
            state: ContentFrameState::new(NewFrameToken(), Some(owner)),
            domain: Rc::downgrade(self),
            parent,
            navigation_lifecycle: Default::default(),
            owner_document: Rc::downgrade(&parent_document),
            previous: Cell::new(None),
            next: Cell::new(None),
            first_child: Cell::new(None),
            last_child: Cell::new(None),
            name: RefCell::new(Rc::from(name)),
            unique_name: RefCell::new(self.GenerateUniqueName(parent, name, is_created_by_script)),
            generation: Cell::new(1),
            runtime: Default::default(),
            pending_policy: RefCell::new(Some(policy.clone())),
            loading_client: Default::default(),
            navigation: Default::default(),
            anchor_navigation: Default::default(),
            view: Default::default(),
            window_performance_timing: RefCell::new(Rc::new(
                webapi::window_bindings::WindowPerformanceTiming::new(),
            )),
        });
        parent_document
            .borrow_mut()
            .GetDocumentMut()
            .InitializeFrame(frame.clone())
            .map_err(|_| FrameOwnerLoadError::UnexpectedCreatedFrame)?;
        self.frames
            .borrow_mut()
            .insert(frame.state.Identity(), frame.clone());
        self.AppendChild(&frame);
        self.MatchChildHistory(&frame);
        if let Err(error) = frame.Init(&page, &parent_document, &resources, &policy) {
            frame.DetachForRemove();
            return Err(error);
        }
        page.dirty.set(true);
        self.dirty.set(true);
        page.scroll_only.set(false);
        Ok(frame)
    }
    pub fn Children(&self, parent: u64) -> Vec<Rc<LocalFrame>> {
        let mut next = if parent == self.root {
            self.first_child.get()
        } else {
            self.Frame(parent).and_then(|p| p.first_child.get())
        };
        let mut children = Vec::new();
        while let Some(frame) = next.and_then(|id| self.Frame(id)) {
            next = frame.next.get();
            children.push(frame);
        }
        children
    }
    fn Remove(&self, frame: &LocalFrame) {
        let parent = self.Frame(frame.parent);
        let (first, last) = parent
            .as_ref()
            .map_or((&self.first_child, &self.last_child), |p| {
                (&p.first_child, &p.last_child)
            });
        if let Some(previous) = frame.previous.get().and_then(|id| self.Frame(id)) {
            previous.next.set(frame.next.get());
        } else {
            first.set(frame.next.get());
        }
        if let Some(next) = frame.next.get().and_then(|id| self.Frame(id)) {
            next.previous.set(frame.previous.get());
        } else {
            last.set(frame.previous.get());
        }
        frame.previous.set(None);
        frame.next.set(None);
        self.frames.borrow_mut().remove(&frame.state.Identity());
        self.focus_frames
            .borrow_mut()
            .remove(&frame.state.Identity());
    }
}
impl LocalFrame {
    fn Init(
        &self,
        page: &Rc<PageState>,
        parent: &DocumentEngine,
        parent_resources: &ResourceFetcher,
        policy: &FrameOwnerPolicy,
    ) -> Result<(), FrameOwnerLoadError> {
        let document = Rc::new(DocumentEngine::new());
        document
            .borrow_mut()
            .GetDocumentMut()
            .InitializeInitialLocalFrameDocument(parent.borrow().GetDocument(), policy);
        let mut constraints = page.constraints.borrow().clone();
        // LocalFrameView's child constructor is unsized until its owning
        // LayoutEmbeddedContent supplies the physical content-box dimensions.
        constraints.available_size = layoutng_assembly::internal::layout_input::Size {
            width: 0.0,
            height: 0.0,
        };
        constraints.viewport = None;
        let constraints = Rc::new(RefCell::new(constraints));
        let (resources, resource_engine) = parent_resources.ForChildDocument(
            document.Handle(),
            constraints.clone(),
            document.borrow().GetDocument().BaseURL().AsStr().into(),
        );
        let mut loader = document_loader::DocumentLoader::ForInitialLocalFrame(resource_engine);
        loader.BindRequiredDocumentPolicy(policy.required_document.clone());
        loader
            .InitializeLocalFrameDocument(document.borrow_mut().GetDocumentMut())
            .map_err(|_| FrameOwnerLoadError::InitialLocalFrameDocument)?;
        self.InstallDocument(document, Some(loader), resources, constraints)
    }
    pub(super) fn InstallDocument(
        &self,
        document: Rc<DocumentEngine>,
        loader: Option<document_loader::DocumentLoader>,
        resources: Rc<ResourceFetcher>,
        constraints: Rc<RefCell<ConstraintSpace>>,
    ) -> Result<(), FrameOwnerLoadError> {
        let runtime = self.PrepareDocument(document, loader, resources, constraints)?;
        self.InstallPreparedDocument(runtime);
        Ok(())
    }
    pub(super) fn PrepareDocument(
        &self,
        document: Rc<DocumentEngine>,
        loader: Option<document_loader::DocumentLoader>,
        resources: Rc<ResourceFetcher>,
        constraints: Rc<RefCell<ConstraintSpace>>,
    ) -> Result<Rc<LocalFrameRuntime>, FrameOwnerLoadError> {
        let style = style::StyleEngine::new(&document.borrow());
        let client = self.loading_client.get_or_init(|| {
            let client: Rc<dyn dom::frame_owner_loading::FrameOwnerLoadingClient> =
                Rc::new(super::frame_owner_loading::PageFrameOwnerLoadingClient {
                    domain: self.domain.clone(),
                    parent: self.state.Identity(),
                });
            client
        });
        dom::frame_owner_loading::BindFrameOwnerLoadingClient(&document.Handle(), client);
        let navigation_state = std::cell::OnceCell::new();
        if let Some(loader) = &loader {
            let _ = navigation_state.set(loader.NavigationState());
        }
        let runtime = Rc::new(LocalFrameRuntime {
            document,
            window_credentialless: super::local_frame_window::FrameCredentialless(self),
            loader: loader.map(RefCell::new),
            navigation_state,
            resources,
            style: RefCell::new(style),
            layout: RefCell::new(layoutng_assembly::layout_engine::LayoutEngine::new(
                &crate::CreateLayoutAssembly(),
            )),
            constraints,
            view: self
                .view
                .get_or_init(|| Rc::new(LocalFrameView::new()))
                .clone(),
            hit_test: Default::default(),
            editing: Rc::new(Default::default()),
            interaction_state: Default::default(),
            interaction_scroll: Default::default(),
            compositor_scroll_pending: Default::default(),
            blocking_wheel_registry_dirty: Default::default(),
            programmatic_scroll_animators: Default::default(),
            frame_interaction: Default::default(),
            pending_text_overflow_layout_invalidations: Default::default(),
            positions: Default::default(),
        });
        let domain = self
            .domain
            .upgrade()
            .ok_or(FrameOwnerLoadError::MissingClient)?;
        let frame = domain
            .Frame(self.state.Identity())
            .ok_or(FrameOwnerLoadError::UnexpectedCreatedFrame)?;
        runtime.InstallProgrammaticScroll(&frame);
        assert!(runtime
            .positions
            .set(Rc::new(ChildResidentEditingPositions {
                frame: Rc::downgrade(&frame),
                runtime: Rc::downgrade(&runtime)
            }))
            .is_ok());
        runtime.InstallFrameInteraction(&frame);
        Ok(runtime)
    }
    pub(super) fn InstallPreparedDocument(&self, runtime: Rc<LocalFrameRuntime>) {
        // DocumentLoader::ShouldReuseDefaultView is independent of whether
        // ScriptController has materialized a realm for this Window.
        let reuse_window = self.runtime.borrow().as_ref().is_none_or(|old| {
            let old_document = old.document.borrow();
            let next_document = runtime.document.borrow();
            let old_document = old_document.GetDocument();
            let next_document = next_document.GetDocument();
            old_document.IsInitialEmptyDocument()
                && old.window_credentialless == runtime.window_credentialless
                && old_document
                    .SecurityContext()
                    .Origin()
                    .CanAccess(next_document.SecurityContext().Origin().as_ref())
                && old_document
                    .ExecutionContextIsolation()
                    .CrossOriginIsolatedCapability()
                    == next_document
                        .ExecutionContextIsolation()
                        .CrossOriginIsolatedCapability()
        });
        if !reuse_window {
            *self.window_performance_timing.borrow_mut() =
                Rc::new(webapi::window_bindings::WindowPerformanceTiming::new());
        }
        if let Some(page) = self
            .domain
            .upgrade()
            .and_then(|domain| domain.page.upgrade())
        {
            let timing = self.window_performance_timing.borrow().clone();
            page.BindWindowEventTiming(self.state.Identity(), &runtime.document, &timing);
            runtime
                .frame_interaction
                .get()
                .expect("Document input owner")
                .engine
                .BindWindowPerformance(timing.EventTimingSource());
        }
        if let Some(page) = self
            .domain
            .upgrade()
            .and_then(|domain| domain.page.upgrade())
        {
            page.ValidationMessageFrameDetached(self.state.Identity());
            page.native_event_handler_host
                .layout_shift_trackers
                .borrow_mut()
                .remove(&self.state.Identity());
            page.native_event_handler_host
                .accessibility_caches
                .borrow_mut()
                .remove(&self.state.Identity());
        }

        if let Some(domain) = self.domain.upgrade() {
            domain.StopMiddleClickAutoscrollForFrame(self.state.Identity());
        }
        let old = self.runtime.borrow_mut().take();
        if let Some(old) = old {
            if let Some(domain) = self.domain.upgrade() {
                for child in domain.Children(self.state.Identity()) {
                    child.DetachForRemove();
                }
            }
            if let Some(loader) = old.loader.as_ref() {
                loader.borrow_mut().StopLoading();
            }
            crate::persistent_layout::DetachDocumentLayoutTreeWithEngine(
                &mut old.layout.borrow_mut(),
                &old.document.borrow(),
            );
            old.document
                .borrow()
                .GetDocument()
                .SetLifecycleActive(false);
            self.generation.set(self.generation.get().wrapping_add(1));
        }
        runtime
            .view
            .revision
            .set(runtime.view.revision.get().wrapping_add(1));
        runtime.view.first_layout.set(true);
        runtime.view.lifecycle_updates_throttled.set(false);
        runtime.view.needs_occlusion_tracking.set(false);
        runtime.view.rect_in_parent.set(Default::default());
        runtime.view.rect_in_parent_for_iov2.set(Default::default());
        runtime.view.rect_in_parent_stable_since.set(0);
        runtime.view.rect_in_parent_stable_since_for_iov2.set(0);
        *self.runtime.borrow_mut() = Some(runtime);
        if let Some(domain) = self.domain.upgrade() {
            domain.BindChildEventHandler(self.state.Identity());
            domain.dirty.set(true);
            if let Some(page) = domain.page.upgrade() {
                page.dirty.set(true);
                page.scroll_only.set(false);
            }
        }
    }
}
impl ContentFrameSource for LocalFrame {
    fn State(&self) -> &ContentFrameState {
        &self.state
    }
    fn IsRemoteFrame(&self) -> bool {
        false
    }
    fn DetachForRemove(&self) {
        if self.state.Lifecycle().GetState() == FrameLifecycleState::Detached {
            return;
        }
        let Some(domain) = self.domain.upgrade() else {
            return;
        };
        let Some(owner) = domain.Frame(self.state.Identity()) else {
            return;
        };
        if let Some(page) = domain.page.upgrade() {
            page.ValidationMessageFrameDetached(self.state.Identity());
            page.native_event_handler_host
                .layout_shift_trackers
                .borrow_mut()
                .remove(&self.state.Identity());
            page.native_event_handler_host
                .accessibility_caches
                .borrow_mut()
                .remove(&self.state.Identity());
        }
        domain.StopMiddleClickAutoscrollForFrame(self.state.Identity());
        self.state
            .Lifecycle()
            .AdvanceTo(FrameLifecycleState::Detaching);
        self.DetachFocus();
        self.StopNavigation();
        for child in domain.Children(self.state.Identity()) {
            child.DetachForRemove();
        }
        let runtime = self.runtime.borrow().clone();
        if let Some(runtime) = runtime {
            if let Some(loader) = runtime.loader.as_ref() {
                loader.borrow_mut().StopLoading();
            }
            crate::persistent_layout::DetachDocumentLayoutTreeWithEngine(
                &mut runtime.layout.borrow_mut(),
                &runtime.document.borrow(),
            );
            runtime
                .document
                .borrow()
                .GetDocument()
                .SetLifecycleActive(false);
        }
        self.generation.set(self.generation.get().wrapping_add(1));
        self.state.SetIsLoading(false);
        self.state
            .Lifecycle()
            .AdvanceTo(FrameLifecycleState::Detached);
        if let Some(document) = self.owner_document.upgrade() {
            let source: Rc<dyn ContentFrameSource> = owner;
            let _ = document
                .borrow_mut()
                .GetDocumentMut()
                .DisconnectFrameOwner(&source);
        }
        domain.Remove(self);
        domain.dirty.set(true);
    }
}

impl LocalFrame {
    pub(super) fn CaretSelectionForContentsPoint(
        &self,
        runtime: &Rc<LocalFrameRuntime>,
        point: (i32, i32),
    ) -> Option<layoutng_assembly::editing_state::DOMSelection> {
        use interaction::hit_test::core::layout::hit_test_request::RequestType;
        let domain = self.domain.upgrade()?;
        let frame = domain.Frame(self.state.Identity())?;
        let page = domain.page.upgrade()?;
        let settings = page.interaction_text_settings.borrow().clone()?;
        let paint = page.paint_engine.borrow();
        let queries = LocalFrameQueries {
            domain: &domain,
            paint: &paint,
        };
        let query = CurrentLocalFrameQuery {
            queries: &queries,
            frame,
            generation: self.generation.get(),
        };
        let location = HitTestLocation::FromPhysicalOffset(foundation::PhysicalOffset::new(
            foundation::LayoutUnit::FromDoubleRound(f64::from(point.0)),
            foundation::LayoutUnit::FromDoubleRound(f64::from(point.1)),
        ));
        let request = HitTestResultRequest::<()>::new(
            RequestType::kMove as u32
                | RequestType::kReadOnly as u32
                | RequestType::kActive as u32
                | RequestType::kIgnoreClipping as u32,
            None,
            None,
        );
        let hit = query.HitTestNoLifecycleUpdate(&location, &request).ok()?;
        let target = hit.Result().InnerNode()?.DOMNodeId()?;
        let position = hit.Result().GetPosition().map(|position| position.Value());
        let layout = runtime.layout.borrow();
        let owner = runtime.document.borrow();
        let selection = interaction::adapter::page_frame_selection::CurrentDOMSelectionForFocus(
            owner.GetDocument(),
            &runtime.editing.selections,
        )?;
        interaction::adapter::page_frame_selection::CaretSelectionForContentsHit(
            owner.GetDocument(),
            layout.Output()?,
            selection,
            target,
            position,
            *hit.Result().LocalPoint(),
            layoutng_assembly::EditingPositionOptions {
                move_caret_to_horizontal_boundary_when_past_top_or_bottom: settings
                    .editing_behavior
                    .ShouldMoveCaretToHorizontalBoundaryWhenPastTopOrBottom(),
                ..Default::default()
            },
            &settings.ui_locale,
        )
    }
}
