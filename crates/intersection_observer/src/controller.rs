//! intersection_observer_controller.cc:27-335. Existing document task sources
//! and monotonic clock are supplied by the native execution-context adapter.
use crate::*;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
    time::Duration,
};
pub trait IntersectionObserverEnvironment {
    fn Controller(&self) -> Rc<IntersectionObserverController>;
    fn ExecutionContextAlive(&self) -> bool;
    fn DocumentIdentity(&self) -> u128;
    fn MonotonicTime(&self) -> Duration;
    fn TimeStamp(&self, monotonic_time: Duration) -> f64;
    /// Must enqueue on the existing InternalIntersectionObserver task source.
    fn PostTask(&self, task: Box<dyn FnOnce()>);
    fn ScheduleDelayedIntersection(&self, delay: Duration);
    fn SetNeedsOcclusionTracking(&self);
}
pub struct ComputeIntersectionsContext {
    pub monotonic_time: Duration,
    next_run_delay: Option<Duration>,
    implicit_observer: usize,
    explicit_observer: usize,
    implicit_root_geometry: Option<RootGeometry>,
    explicit_root_geometry: Option<RootGeometry>,
}
impl ComputeIntersectionsContext {
    pub fn new(monotonic_time: Duration) -> Self {
        Self {
            monotonic_time,
            next_run_delay: None,
            implicit_observer: 0,
            explicit_observer: 0,
            implicit_root_geometry: None,
            explicit_root_geometry: None,
        }
    }
    pub fn UpdateNextRunDelay(&mut self, delay: Duration) {
        assert!(!delay.is_zero());
        self.next_run_delay = Some(self.next_run_delay.map_or(delay, |old| old.min(delay)));
    }
    pub fn GetAndResetNextRunDelay(&mut self) -> Option<Duration> {
        self.next_run_delay.take()
    }
    pub fn RootGeometry(
        &mut self,
        observer: &IntersectionObserver,
        flags: u32,
    ) -> &mut Option<RootGeometry> {
        if observer.RootIsImplicit() {
            if self.implicit_observer != observer.id() {
                self.implicit_observer = observer.id();
                if let Some(geometry) = self.implicit_root_geometry.as_mut() {
                    geometry.UpdateMargin(
                        if flags & compute_flags::REPORT_IMPLICIT_ROOT_BOUNDS != 0 {
                            observer.params.RootMargin()
                        } else {
                            &[]
                        },
                    );
                }
            }
            &mut self.implicit_root_geometry
        } else {
            if self.explicit_observer != observer.id() {
                self.explicit_observer = observer.id();
                self.explicit_root_geometry = None;
            }
            &mut self.explicit_root_geometry
        }
    }
}
#[derive(Clone, Copy, Default, Debug)]
pub struct IntersectionComputeCounts {
    pub internal: i64,
    pub javascript: i64,
}
pub struct IntersectionObserverController {
    environment: Rc<dyn IntersectionObserverEnvironment>,
    explicit: RefCell<HashMap<usize, Rc<IntersectionObserver>>>,
    implicit: RefCell<HashMap<usize, (Rc<IntersectionObserver>, Vec<Rc<IntersectionObservation>>)>>,
    pending: RefCell<HashMap<usize, Rc<IntersectionObserver>>>,
    needs_occlusion_tracking: Cell<bool>,
    has_active_observations: Cell<bool>,
}
impl IntersectionObserverController {
    pub fn Create(environment: Rc<dyn IntersectionObserverEnvironment>) -> Rc<Self> {
        Rc::new(Self {
            environment,
            explicit: RefCell::new(HashMap::new()),
            implicit: RefCell::new(HashMap::new()),
            pending: RefCell::new(HashMap::new()),
            needs_occlusion_tracking: Cell::new(false),
            has_active_observations: Cell::new(false),
        })
    }
    pub fn Environment(&self) -> Rc<dyn IntersectionObserverEnvironment> {
        self.environment.clone()
    }
    pub fn ScheduleForDelivery(self: &Rc<Self>, observer: Rc<IntersectionObserver>) {
        let behavior = observer.params.behavior;
        self.pending.borrow_mut().insert(observer.id(), observer);
        if behavior == DeliveryBehavior::PostTask {
            let weak = Rc::downgrade(self);
            self.environment.PostTask(Box::new(move || {
                if let Some(controller) = weak.upgrade() {
                    controller.DeliverNotifications(DeliveryBehavior::PostTask);
                }
            }));
        }
    }
    pub fn DeliverNotifications(&self, behavior: DeliveryBehavior) {
        if !self.environment.ExecutionContextAlive() {
            self.pending.borrow_mut().clear();
            return;
        }
        let invoked = self
            .pending
            .borrow()
            .values()
            .filter(|o| o.params.behavior == behavior)
            .cloned()
            .collect::<Vec<_>>();
        for observer in invoked {
            self.pending.borrow_mut().remove(&observer.id());
            observer.Deliver();
        }
    }
    pub fn AddTrackedObserver(&self, observer: Rc<IntersectionObserver>) {
        if observer.RootIsImplicit() || !observer.HasObservations() {
            return;
        }
        if observer.params.track_visibility {
            self.environment.SetNeedsOcclusionTracking();
        }
        self.explicit.borrow_mut().insert(observer.id(), observer);
    }
    pub fn RemoveTrackedObserver(&self, observer: &IntersectionObserver) {
        if !observer.RootIsImplicit() {
            self.explicit.borrow_mut().remove(&observer.id());
        }
    }
    pub fn AddTrackedObservation(&self, observation: Rc<IntersectionObservation>) {
        let Some(observer) = observation.Observer() else {
            return;
        };
        if !observer.RootIsImplicit() {
            return;
        }
        if observer.params.track_visibility {
            self.environment.SetNeedsOcclusionTracking();
        }
        let mut tracked = self.implicit.borrow_mut();
        let entry = tracked
            .entry(observer.id())
            .or_insert_with(|| (observer, vec![]));
        if !entry.1.iter().any(|o| Rc::ptr_eq(o, &observation)) {
            entry.1.push(observation);
        }
    }
    pub fn RemoveTrackedObservation(&self, observation: &IntersectionObservation) {
        let Some(observer) = observation.Observer() else {
            return;
        };
        if !observer.RootIsImplicit() {
            return;
        }
        let mut tracked = self.implicit.borrow_mut();
        if let Some(entry) = tracked.get_mut(&observer.id()) {
            entry.1.retain(|o| !std::ptr::eq(o.as_ref(), observation));
            if entry.1.is_empty() {
                tracked.remove(&observer.id());
            }
        }
    }
    pub fn UpdateIntersectionObserverStatus(&self) {
        self.needs_occlusion_tracking.set(false);
        self.has_active_observations.set(false);
        if !self.environment.ExecutionContextAlive() {
            return;
        }
        for observer in self.explicit.borrow().values() {
            for observation in observer.Observations() {
                self.needs_occlusion_tracking
                    .set(self.needs_occlusion_tracking.get() || observer.params.track_visibility);
                self.has_active_observations
                    .set(self.has_active_observations.get() || observation.CanCompute());
            }
        }
        for (observer, observations) in self.implicit.borrow().values() {
            for observation in observations {
                self.needs_occlusion_tracking
                    .set(self.needs_occlusion_tracking.get() || observer.params.track_visibility);
                self.has_active_observations
                    .set(self.has_active_observations.get() || observation.CanCompute());
            }
        }
    }
    pub fn NeedsOcclusionTracking(&self) -> bool {
        self.needs_occlusion_tracking.get()
    }
    pub fn HasActiveObservations(&self) -> bool {
        self.has_active_observations.get()
    }
    pub fn ComputeIntersections(
        &self,
        flags: u32,
        scroll_delta: [f32; 2],
        context: &mut ComputeIntersectionsContext,
    ) -> Result<IntersectionComputeCounts, RequiredIntersectionMechanism> {
        if !self.environment.ExecutionContextAlive() {
            return Ok(IntersectionComputeCounts::default());
        }
        let update = flags & compute_flags::UPDATE_TRACKING != 0;
        let document = self.environment.DocumentIdentity();
        let mut counts = IntersectionComputeCounts::default();
        let mut compute = |observer: &IntersectionObserver,
                           observations: &[Rc<IntersectionObservation>]|
         -> Result<(), RequiredIntersectionMechanism> {
            let mut count = 0;
            for observation in observations {
                count += observation.ComputeIntersection(flags, scroll_delta, context)?;
            }
            if observer.params.internal {
                counts.internal += count;
            } else {
                counts.javascript += count;
            }
            Ok(())
        };
        let mut retired = vec![];
        {
            let mut explicit = self.explicit.borrow_mut();
            if update {
                explicit.retain(|_, observer| {
                    let keep = observer.HasObservations()
                        && observer
                            .Root()
                            .is_some_and(|r| r.DocumentIdentity() == document);
                    if !observer.RootIsValid() {
                        retired.push(observer.clone());
                    }
                    keep
                });
            }
            for observer in explicit.values() {
                compute(observer, &observer.Observations())?;
            }
            if update {
                explicit.retain(|_, observer| observer.Root().is_some_and(|r| r.IsConnected()));
            }
        }
        for observer in retired {
            observer.Disconnect();
        }
        {
            let mut implicit = self.implicit.borrow_mut();
            for (observer, observations) in implicit.values_mut() {
                if update {
                    observations
                        .retain(|o| o.Target().is_some_and(|t| t.DocumentIdentity() == document));
                }
                compute(observer, observations)?;
                if update {
                    observations.retain(|o| o.Target().is_some_and(|t| t.IsConnected()));
                }
            }
            if update {
                implicit.retain(|_, (_, observations)| !observations.is_empty());
            }
        }
        if let Some(delay) = context.GetAndResetNextRunDelay() {
            self.environment.ScheduleDelayedIntersection(delay);
        }
        Ok(counts)
    }
}
impl Drop for IntersectionObserverController {
    fn drop(&mut self) {
        // Release Rust temporary entry/active cycles at execution-context GC.
        for observer in self.explicit.get_mut().values() {
            observer.ClearActive();
        }
        for (observer, _) in self.implicit.get_mut().values() {
            observer.ClearActive();
        }
        for observer in self.pending.get_mut().values() {
            observer.ClearActive();
        }
    }
}
