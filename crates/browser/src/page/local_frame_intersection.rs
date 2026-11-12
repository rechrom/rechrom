//! Native Document/Element bindings for the original IntersectionObserver
//! owners. Geometry is calculated by LayoutNG's resident source translation;
//! delivery stays after released-borrow local-root lifecycle completion.
use super::local_frame::LocalFrameDomain;
use super::*;
use dom::{frame_owner_loading::FrameOwnerLoadError, persistent_document::DOMOwnerHandle};
use intersection_observer::*;
use std::{
    cell::OnceCell,
    collections::VecDeque,
    time::{Duration, Instant},
};

struct FrameIntersectionEnvironment {
    domain: Weak<LocalFrameDomain>,
    frame: u64,
    document: Weak<DocumentEngine>,
    identity: u128,
    clock: Rc<webapi::window_bindings::WindowPerformanceTiming>,
    controller: OnceCell<Weak<IntersectionObserverController>>,
    needs_occlusion_tracking: Cell<bool>,
}
impl IntersectionObserverEnvironment for FrameIntersectionEnvironment {
    fn Controller(&self) -> Rc<IntersectionObserverController> {
        self.controller
            .get()
            .and_then(Weak::upgrade)
            .expect("Document intersection controller")
    }
    fn ExecutionContextAlive(&self) -> bool {
        self.document
            .upgrade()
            .is_some_and(|d| d.borrow().GetDocument().IsActive())
    }
    fn DocumentIdentity(&self) -> u128 {
        self.identity
    }
    fn MonotonicTime(&self) -> Duration {
        self.clock.MonotonicElapsed()
    }
    fn TimeStamp(&self, time: Duration) -> f64 {
        self.clock
            .MonotonicTimeToDOMHighResTimeStamp(self.clock.MonotonicOrigin() + time)
    }
    fn PostTask(&self, task: Box<dyn FnOnce()>) {
        if let Some(domain) = self.domain.upgrade() {
            domain
                .intersection_tasks
                .borrow_mut()
                .push_back((self.document.clone(), task));
            domain.intersection_deadline.set(Some(Instant::now()));
        }
    }
    fn ScheduleDelayedIntersection(&self, delay: Duration) {
        if let Some(domain) = self.domain.upgrade() {
            let deadline = Instant::now() + delay;
            domain.intersection_deadline.set(Some(
                domain
                    .intersection_deadline
                    .get()
                    .map_or(deadline, |old| old.min(deadline)),
            ));
        }
    }
    fn SetNeedsOcclusionTracking(&self) {
        self.needs_occlusion_tracking.set(true);
        if let Some(domain) = self.domain.upgrade() {
            if self.frame == domain.root {
                domain.root_view.needs_occlusion_tracking.set(true);
            } else if let Some(frame) = domain.Frame(self.frame) {
                if let Some(runtime) = frame.runtime.borrow().as_ref() {
                    runtime.view.needs_occlusion_tracking.set(true);
                }
            }
        }
    }
}
struct FrameIntersectionTarget {
    document: Weak<DocumentEngine>,
    domain: Weak<LocalFrameDomain>,
    frame: u64,
    owner: DOMOwnerHandle,
    data: Rc<ElementIntersectionObserverData>,
}
impl IntersectionObserverTarget for FrameIntersectionTarget {
    fn Data(&self) -> Rc<ElementIntersectionObserverData> {
        self.data.clone()
    }
    fn Controller(&self) -> Option<Rc<IntersectionObserverController>> {
        let document = self.document.upgrade()?;
        let dom = document.borrow();
        let doc = dom.GetDocument();
        doc.IntersectionController(doc.RootHandle())
    }
    fn DocumentIdentity(&self) -> u128 {
        let document = self.document.upgrade().expect("live observer document");
        let id = document.borrow().GetDocument().RootHandle().Identity();
        id
    }
    fn NodeId(&self) -> u64 {
        self.owner.Id()
    }
    fn IsDocumentRoot(&self) -> bool {
        self.document
            .upgrade()
            .is_some_and(|d| d.borrow().GetDocument().RootHandle() == self.owner)
    }
    fn IsConnected(&self) -> bool {
        self.document.upgrade().is_some_and(|d| {
            let owner = d.borrow();
            let doc = owner.GetDocument();
            doc.FindNodeById(self.owner.Id()).is_some_and(|index| {
                doc.NodeHandle(index) == self.owner && doc.IsConnectedToDocument(index)
            })
        })
    }
    fn HasFrame(&self) -> bool {
        self.domain
            .upgrade()
            .is_some_and(|d| self.frame == d.root || d.Frame(self.frame).is_some())
    }
    fn OcclusionState(&self) -> OcclusionState {
        OcclusionState::Unknown
    }
    fn ScheduleRequiredUpdate(&self) {
        if let Some(domain) = self.domain.upgrade() {
            domain.dirty.set(true);
            if let Some(page) = domain.page.upgrade() {
                page.dirty.set(true);
                page.scroll_only.set(false);
            }
        }
    }
    fn ComputeGeometry(
        &self,
        root: Option<&dyn IntersectionObserverTarget>,
        params: &IntersectionObserverParams,
        mut flags: u32,
        root_geometry: &mut Option<RootGeometry>,
        cache: &mut CachedRects,
    ) -> Result<IntersectionGeometry, RequiredIntersectionMechanism> {
        let Some(domain) = self.domain.upgrade() else {
            return Ok(Default::default());
        };
        let Some(document) = self.document.upgrade() else {
            return Ok(Default::default());
        };
        let dom = document.borrow();
        let doc = dom.GetDocument();
        if !doc.IsActive() || !self.IsConnected() {
            cache.valid = false;
            return Ok(Default::default());
        }
        let root = match root {
            None => {
                if self.frame != domain.root {
                    return Err(RequiredIntersectionMechanism::CrossFrameGeometry);
                }
                flags |= geometry_flags::ROOT_IMPLICIT;
                None
            }
            Some(root) => {
                if root.DocumentIdentity() != doc.RootHandle().Identity() {
                    return Err(RequiredIntersectionMechanism::CrossFrameGeometry);
                }
                Some(if root.IsDocumentRoot() {
                    layoutng_assembly::layout_engine::NativeIntersectionRoot::Document
                } else {
                    layoutng_assembly::layout_engine::NativeIntersectionRoot::Element(root.NodeId())
                })
            }
        };
        domain
            .WithLayout(self.frame, |layout| {
                layout.ComputeIntersectionGeometryForNode(
                    self.owner.Id(),
                    root,
                    doc.StandardizedBrowserZoomEnabled(),
                    OcclusionState::Unknown,
                    params,
                    flags,
                    root_geometry,
                    cache,
                )
            })
            .unwrap_or_else(|| {
                cache.valid = false;
                Ok(Default::default())
            })
    }
}
struct LazyFrameDelegate {
    owner: DOMOwnerHandle,
    observer: Weak<dom::lazy_load_frame_observer::LazyLoadFrameObserver>,
    domain: Weak<LocalFrameDomain>,
}
impl IntersectionObserverDelegate for LazyFrameDelegate {
    fn Deliver(&self, entries: &[IntersectionObserverEntry], _: &Rc<IntersectionObserver>) {
        let entry = entries.last().expect("source lazy frame observation entry");
        assert_eq!(entry.target.NodeId(), self.owner.Id());
        if let Some(observer) = self.observer.upgrade() {
            if let Err(error) =
                observer.LoadIfNearViewport(self.owner, entry.geometry.IsIntersecting())
            {
                if let Some(domain) = self.domain.upgrade() {
                    *domain.intersection_error.borrow_mut() = Some(error);
                }
            }
        }
    }
}
struct LazyFrameSubscription(Rc<IntersectionObserver>);
impl dom::lazy_load_frame_observer::LazyFrameIntersectionSubscription for LazyFrameSubscription {
    fn Disconnect(&self) {
        self.0.Disconnect();
    }
}
impl LocalFrameDomain {
    pub(super) fn WithLayout<T>(
        &self,
        frame: u64,
        read: impl FnOnce(&layoutng_assembly::layout_engine::LayoutEngine) -> T,
    ) -> Option<T> {
        if frame == self.root {
            let page = self.page.upgrade()?;
            let result = read(&page.layout_engine.borrow());
            Some(result)
        } else {
            let frame = self.Frame(frame)?;
            let runtime = frame.runtime.borrow().clone()?;
            let result = read(&runtime.layout.borrow());
            Some(result)
        }
    }
    pub(super) fn EnsureIntersectionTarget(
        self: &Rc<Self>,
        frame: u64,
        document: &Rc<DocumentEngine>,
        doc: &dom::Document,
        owner: DOMOwnerHandle,
    ) -> Result<Rc<dyn IntersectionObserverTarget>, FrameOwnerLoadError> {
        let index = doc
            .FindNodeById(owner.Id())
            .ok_or(FrameOwnerLoadError::MissingOwner)?;
        if doc.NodeHandle(index) != owner {
            return Err(FrameOwnerLoadError::WrongOwnerDocument);
        }
        if doc.IntersectionController(doc.RootHandle()).is_none() {
            let page = self
                .page
                .upgrade()
                .ok_or(FrameOwnerLoadError::MissingClient)?;
            let clock = if frame == self.root {
                page.window_performance_timing.clone()
            } else {
                self.Frame(frame)
                    .and_then(|frame| frame.WindowPerformanceTiming())
                    .ok_or(FrameOwnerLoadError::LazyLoadObserver)?
            };
            let environment = Rc::new(FrameIntersectionEnvironment {
                domain: Rc::downgrade(self),
                frame,
                document: Rc::downgrade(document),
                identity: doc.RootHandle().Identity(),
                clock,
                controller: Default::default(),
                needs_occlusion_tracking: Cell::new(false),
            });
            let controller = IntersectionObserverController::Create(environment.clone());
            assert!(environment
                .controller
                .set(Rc::downgrade(&controller))
                .is_ok());
            doc.SetIntersectionController(doc.RootHandle(), controller);
        }
        if let Some(target) = doc.IntersectionObserverTarget(index) {
            return Ok(target);
        }
        let target: Rc<dyn IntersectionObserverTarget> = Rc::new(FrameIntersectionTarget {
            document: Rc::downgrade(document),
            domain: Rc::downgrade(self),
            frame,
            owner,
            data: doc.EnsureIntersectionObserverData(index),
        });
        doc.SetIntersectionObserverTarget(index, target.clone());
        Ok(target)
    }
    pub(super) fn ObserveLazyFrame(
        self: &Rc<Self>,
        owner: DOMOwnerHandle,
        target: Rc<dyn IntersectionObserverTarget>,
        observer: Weak<dom::lazy_load_frame_observer::LazyLoadFrameObserver>,
    ) -> Result<
        Rc<dyn dom::lazy_load_frame_observer::LazyFrameIntersectionSubscription>,
        FrameOwnerLoadError,
    > {
        let controller = target
            .Controller()
            .ok_or(FrameOwnerLoadError::LazyLoadObserver)?;
        let params =
            dom::lazy_load_frame_observer::IntersectionParams(self.effective_connection_type.get());
        let observer = IntersectionObserver::Create(
            controller.Environment(),
            Rc::new(LazyFrameDelegate {
                owner,
                observer,
                domain: Rc::downgrade(self),
            }),
            None,
            params,
        )
        .map_err(|_| FrameOwnerLoadError::LazyLoadObserver)?;
        observer.Observe(target);
        Ok(Rc::new(LazyFrameSubscription(observer)))
    }
    pub(super) fn RunIntersectionTask(&self) {
        let next = self.intersection_tasks.borrow_mut().pop_front();
        if let Some((document, task)) = next {
            if document
                .upgrade()
                .is_some_and(|d| d.borrow().GetDocument().IsActive())
            {
                task();
            }
        }
    }
    pub(super) fn UpdateIntersections(&self) -> io::Result<()> {
        let mut frames = VecDeque::from([self.root]);
        let mut controllers = Vec::new();
        while let Some(frame) = frames.pop_front() {
            frames.extend(
                self.Children(frame)
                    .iter()
                    .map(|frame| frame.state.Identity()),
            );
            let Some(document) = self.Document(frame) else {
                continue;
            };
            let dom = document.borrow();
            let doc = dom.GetDocument();
            let state = doc.Lifecycle().GetState();
            if state < dom::document_lifecycle::LifecycleState::LayoutClean
                || state >= dom::document_lifecycle::LifecycleState::Stopping
            {
                continue;
            }
            if let Some(controller) = doc.IntersectionController(doc.RootHandle()) {
                controllers.push(controller);
            }
        }
        self.intersection_deadline.set(None);
        for controller in &controllers {
            controller.UpdateIntersectionObserverStatus();
            let mut context =
                ComputeIntersectionsContext::new(controller.Environment().MonotonicTime());
            controller
                .ComputeIntersections(
                    compute_flags::IMPLICIT_ROOT_NEEDS_UPDATE
                        | compute_flags::EXPLICIT_ROOT_NEEDS_UPDATE
                        | compute_flags::REPORT_IMPLICIT_ROOT_BOUNDS
                        | compute_flags::UPDATE_TRACKING,
                    [0.0; 2],
                    &mut context,
                )
                .map_err(|error| {
                    io::Error::other(format!("native intersection geometry required: {error:?}"))
                })?;
        }
        for controller in controllers {
            controller.DeliverNotifications(DeliveryBehavior::PostLifecycle);
        }
        if !self.intersection_tasks.borrow().is_empty() {
            self.intersection_deadline.set(Some(Instant::now()));
        }
        if let Some(error) = self.intersection_error.borrow_mut().take() {
            return Err(io::Error::other(format!(
                "lazy frame navigation required: {error:?}"
            )));
        }
        Ok(())
    }
}
