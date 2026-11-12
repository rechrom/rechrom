//! intersection_observer.cc:332-496; intersection_observation.cc:32-255.
use crate::*;
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
    time::Duration,
};
pub trait IntersectionObserverTarget {
    fn Data(&self) -> Rc<ElementIntersectionObserverData>;
    fn Controller(&self) -> Option<Rc<IntersectionObserverController>>;
    fn DocumentIdentity(&self) -> u128;
    fn NodeId(&self) -> u64;
    fn IsDocumentRoot(&self) -> bool;
    fn IsConnected(&self) -> bool;
    fn HasFrame(&self) -> bool;
    fn OcclusionState(&self) -> OcclusionState;
    fn ScheduleRequiredUpdate(&self);
    fn ComputeGeometry(
        &self,
        root: Option<&dyn IntersectionObserverTarget>,
        params: &IntersectionObserverParams,
        flags: u32,
        root_geometry: &mut Option<RootGeometry>,
        cache: &mut CachedRects,
    ) -> Result<IntersectionGeometry, RequiredIntersectionMechanism>;
}
pub trait IntersectionObserverDelegate {
    fn Deliver(&self, entries: &[IntersectionObserverEntry], observer: &Rc<IntersectionObserver>);
}
#[derive(Default)]
pub struct ElementIntersectionObserverData {
    observations: RefCell<Vec<(usize, Rc<IntersectionObservation>)>>,
    root_observers: RefCell<Vec<Weak<IntersectionObserver>>>,
}
impl ElementIntersectionObserverData {
    pub fn TrackWithController(&self, controller: &Rc<IntersectionObserverController>) {
        let observations = self
            .observations
            .borrow()
            .iter()
            .map(|(_, o)| o.clone())
            .collect::<Vec<_>>();
        for observation in observations {
            controller.AddTrackedObservation(observation);
        }
        let observers = self
            .root_observers
            .borrow()
            .iter()
            .filter_map(Weak::upgrade)
            .collect::<Vec<_>>();
        for observer in observers {
            controller.AddTrackedObserver(observer);
        }
    }
    pub fn NeedsOcclusionTracking(&self) -> bool {
        self.observations
            .borrow()
            .iter()
            .any(|(_, o)| o.Observer().is_some_and(|o| o.params.track_visibility))
    }
    pub fn IsEmpty(&self) -> bool {
        self.observations.borrow().is_empty()
            && self
                .root_observers
                .borrow()
                .iter()
                .all(|o| o.strong_count() == 0)
    }
}
pub struct IntersectionObserver {
    pub(crate) environment: Rc<dyn IntersectionObserverEnvironment>,
    delegate: Rc<dyn IntersectionObserverDelegate>,
    pub params: IntersectionObserverParams,
    root: Option<Weak<dyn IntersectionObserverTarget>>,
    root_implicit: bool,
    observations: RefCell<Vec<Weak<IntersectionObservation>>>,
    active: RefCell<Vec<Rc<IntersectionObservation>>>,
}
impl IntersectionObserver {
    pub fn Create(
        environment: Rc<dyn IntersectionObserverEnvironment>,
        delegate: Rc<dyn IntersectionObserverDelegate>,
        root: Option<Rc<dyn IntersectionObserverTarget>>,
        params: IntersectionObserverParams,
    ) -> Result<Rc<Self>, ObserverOptionsError> {
        let observer = Rc::new(Self {
            environment,
            delegate,
            params: params.Normalize()?,
            root_implicit: root.is_none(),
            root: root.as_ref().map(Rc::downgrade),
            observations: RefCell::new(vec![]),
            active: RefCell::new(vec![]),
        });
        if let Some(root) = root {
            root.Data()
                .root_observers
                .borrow_mut()
                .push(Rc::downgrade(&observer));
        }
        Ok(observer)
    }
    pub fn RootIsImplicit(&self) -> bool {
        self.root_implicit
    }
    pub fn Root(&self) -> Option<Rc<dyn IntersectionObserverTarget>> {
        self.root.as_ref().and_then(Weak::upgrade)
    }
    pub fn RootIsValid(&self) -> bool {
        self.root_implicit || self.Root().is_some()
    }
    pub fn HasObservations(&self) -> bool {
        self.observations
            .borrow()
            .iter()
            .any(|o| o.strong_count() != 0)
    }
    pub fn Observations(&self) -> Vec<Rc<IntersectionObservation>> {
        self.observations
            .borrow()
            .iter()
            .filter_map(Weak::upgrade)
            .collect()
    }
    pub(crate) fn id(&self) -> usize {
        self as *const Self as usize
    }
    pub fn Observe(self: &Rc<Self>, target: Rc<dyn IntersectionObserverTarget>) {
        if !self.RootIsValid() {
            return;
        }
        let data = target.Data();
        if data
            .observations
            .borrow()
            .iter()
            .any(|(id, _)| *id == self.id())
        {
            return;
        }
        let observation = Rc::new(IntersectionObservation {
            observer: RefCell::new(Some(self.clone())),
            target: Rc::downgrade(&target),
            entries: RefCell::new(vec![]),
            last_run: Cell::new(None),
            cache: RefCell::new(CachedRects::default()),
            last_threshold: Cell::new(usize::MAX),
            last_visible: Cell::new(false),
            needs_update: Cell::new(true),
        });
        data.observations
            .borrow_mut()
            .push((self.id(), observation.clone()));
        self.observations
            .borrow_mut()
            .push(Rc::downgrade(&observation));
        if let Some(root) = self.Root() {
            if let Some(controller) = root.Controller() {
                controller.AddTrackedObserver(self.clone());
            }
        }
        if let Some(controller) = target.Controller() {
            controller.AddTrackedObservation(observation);
        }
        target.ScheduleRequiredUpdate();
    }
    pub fn Unobserve(&self, target: &dyn IntersectionObserverTarget) {
        let observation = target
            .Data()
            .observations
            .borrow()
            .iter()
            .find(|(id, _)| *id == self.id())
            .map(|(_, o)| o.clone());
        if let Some(observation) = observation {
            observation.Disconnect();
            self.observations
                .borrow_mut()
                .retain(|o| o.upgrade().is_some_and(|o| !Rc::ptr_eq(&o, &observation)));
            self.active
                .borrow_mut()
                .retain(|o| !Rc::ptr_eq(&o, &observation));
        }
        if !self.HasObservations() {
            if let Some(root) = self.Root() {
                if let Some(controller) = root.Controller() {
                    controller.RemoveTrackedObserver(self);
                }
            }
        }
    }
    pub fn Disconnect(&self) {
        for observation in self.Observations() {
            observation.Disconnect();
        }
        self.observations.borrow_mut().clear();
        self.active.borrow_mut().clear();
        if let Some(root) = self.Root() {
            if let Some(controller) = root.Controller() {
                controller.RemoveTrackedObserver(self);
            }
        }
    }
    pub fn TakeRecords(&self) -> Vec<IntersectionObserverEntry> {
        let mut entries = vec![];
        for observation in self.Observations() {
            observation.TakeRecords(&mut entries);
        }
        self.active.borrow_mut().clear();
        entries
    }
    fn ReportUpdates(self: &Rc<Self>, observation: &Rc<IntersectionObservation>) {
        let schedule = self.active.borrow().is_empty();
        if !self
            .active
            .borrow()
            .iter()
            .any(|o| Rc::ptr_eq(o, observation))
        {
            self.active.borrow_mut().push(observation.clone());
        }
        if schedule {
            self.environment
                .Controller()
                .ScheduleForDelivery(self.clone());
        }
    }
    pub fn NeedsDelivery(&self) -> bool {
        !self.active.borrow().is_empty()
    }
    pub fn Deliver(self: &Rc<Self>) {
        if !self.NeedsDelivery() {
            return;
        }
        let entries = self.TakeRecords();
        if !entries.is_empty() {
            self.delegate.Deliver(&entries, self);
        }
    }
    pub(crate) fn ClearActive(&self) {
        self.active.borrow_mut().clear();
        for observation in self.Observations() {
            observation.entries.borrow_mut().clear();
        }
    }
}
pub struct IntersectionObservation {
    observer: RefCell<Option<Rc<IntersectionObserver>>>,
    target: Weak<dyn IntersectionObserverTarget>,
    entries: RefCell<Vec<IntersectionObserverEntry>>,
    last_run: Cell<Option<Duration>>,
    cache: RefCell<CachedRects>,
    last_threshold: Cell<usize>,
    last_visible: Cell<bool>,
    needs_update: Cell<bool>,
}
impl IntersectionObservation {
    pub fn Observer(&self) -> Option<Rc<IntersectionObserver>> {
        self.observer.borrow().clone()
    }
    pub fn Target(&self) -> Option<Rc<dyn IntersectionObserverTarget>> {
        self.target.upgrade()
    }
    pub fn CanCompute(&self) -> bool {
        self.Target().is_some()
            && self
                .Observer()
                .is_some_and(|o| o.RootIsValid() && o.environment.ExecutionContextAlive())
    }
    pub fn Disconnect(&self) {
        let Some(observer) = self.Observer() else {
            return;
        };
        if let Some(target) = self.Target() {
            target
                .Data()
                .observations
                .borrow_mut()
                .retain(|(id, _)| *id != observer.id());
            if let Some(controller) = target.Controller() {
                controller.RemoveTrackedObservation(self);
            }
        }
        self.entries.borrow_mut().clear();
        self.observer.borrow_mut().take();
    }
    pub fn TakeRecords(&self, entries: &mut Vec<IntersectionObserverEntry>) {
        entries.append(&mut *self.entries.borrow_mut());
    }
    pub fn MinScrollDeltaToUpdate(&self) -> [f32; 2] {
        let cache = self.cache.borrow();
        if cache.valid {
            cache.min_scroll_delta_to_update
        } else {
            [0.0; 2]
        }
    }
    pub fn ComputeIntersection(
        self: &Rc<Self>,
        flags: u32,
        mut scroll_delta: [f32; 2],
        context: &mut ComputeIntersectionsContext,
    ) -> Result<i64, RequiredIntersectionMechanism> {
        use compute_flags as F;
        if !self.CanCompute() {
            return Ok(0);
        }
        if flags & F::CONSUME_SCROLL_DELTA != 0 {
            let mut cache = self.cache.borrow_mut();
            for i in 0..2 {
                cache.min_scroll_delta_to_update[i] -= scroll_delta[i];
            }
            scroll_delta = [0.0; 2];
        }
        let observer = self.Observer().expect("computed observer");
        if (flags & F::POST_LAYOUT_ONLY != 0)
            != (observer.params.behavior == DeliveryBehavior::PostLayout)
        {
            return Ok(0);
        }
        let had_pending_update = self.needs_update.get();
        if flags
            & if observer.RootIsImplicit() {
                F::IMPLICIT_ROOT_NEEDS_UPDATE
            } else {
                F::EXPLICIT_ROOT_NEEDS_UPDATE
            }
            != 0
        {
            self.needs_update.set(true);
        }
        if !self.CanCompute() || !self.needs_update.get() {
            return Ok(0);
        }
        let target = self.Target().expect("computed target");
        if target.IsConnected()
            && target.HasFrame()
            && observer.params.track_visibility
            && !self.last_visible.get()
            && target.OcclusionState() == OcclusionState::Unknown
        {
            return Ok(0);
        }
        if flags & F::IGNORE_DELAY == 0 {
            if let Some(last) = self.last_run.get() {
                let remaining = observer
                    .params
                    .delay
                    .saturating_sub(context.monotonic_time.saturating_sub(last));
                if !remaining.is_zero() {
                    context.UpdateNextRunDelay(remaining);
                    return Ok(0);
                }
            }
        }
        self.last_run.set(Some(context.monotonic_time));
        self.needs_update.set(false);
        if !had_pending_update && flags & F::SCROLL_AND_VISIBILITY_ONLY != 0 {
            let min = self.cache.borrow().min_scroll_delta_to_update;
            if min[0] > scroll_delta[0] && min[1] > scroll_delta[1] {
                return Ok(1);
            }
        }
        let mut geometry_flags = geometry_flags::CONVERT_TO_CSS_PIXELS;
        if observer.params.always_report_root_bounds
            || flags & F::REPORT_IMPLICIT_ROOT_BOUNDS != 0
            || !observer.RootIsImplicit()
        {
            geometry_flags |= geometry_flags::REPORT_ROOT_BOUNDS;
        }
        if observer.params.track_visibility {
            geometry_flags |= geometry_flags::COMPUTE_VISIBILITY;
        }
        if observer.params.expose_occluder_id {
            geometry_flags |= geometry_flags::EXPOSE_OCCLUDER_ID;
        }
        if observer.params.fraction_of_root {
            geometry_flags |= geometry_flags::FRACTION_OF_ROOT;
        }
        if observer.params.use_overflow_clip_edge {
            geometry_flags |= geometry_flags::OVERFLOW_CLIP_EDGE;
        }
        if observer.params.internal {
            geometry_flags |= geometry_flags::RESPECT_FILTERS;
        }
        if flags & F::SCROLL_AND_VISIBILITY_ONLY != 0 {
            geometry_flags |= geometry_flags::SCROLL_AND_VISIBILITY_ONLY;
        }
        let root = observer.Root();
        let root_geometry = context.RootGeometry(&observer, flags);
        let geometry = match target.ComputeGeometry(
            root.as_deref(),
            &observer.params,
            geometry_flags,
            root_geometry,
            &mut self.cache.borrow_mut(),
        ) {
            Ok(geometry) => geometry,
            Err(required) => {
                self.needs_update.set(true);
                return Err(required);
            }
        };
        let did_compute = geometry.did_compute_geometry;
        if self.last_threshold.get() != geometry.threshold_index
            || self.last_visible.get() != geometry.is_visible
        {
            let threshold = geometry.threshold_index;
            let visible = geometry.is_visible;
            self.entries.borrow_mut().push(IntersectionObserverEntry {
                geometry,
                timestamp: observer.environment.TimeStamp(context.monotonic_time),
                target,
            });
            observer.ReportUpdates(self);
            self.last_threshold.set(threshold);
            self.last_visible.set(visible);
        }
        Ok(i64::from(did_compute))
    }
}
