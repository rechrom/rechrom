//! Chromium ResizeObserver/Controller/Observation, revision
//! 6c1d401fcca5e1b0030563a90c2f2bba168e0c15. DOM retains Element data;
//! resident Layout supplies only the source-derived geometry values.
#![allow(non_snake_case)]
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResizeObserverBoxOptions {
    ContentBox,
    BorderBox,
    DevicePixelContentBox,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ResizeObserverSize {
    pub inline_size: f64,
    pub block_size: f64,
}
/// Source LogicalSize in LayoutUnit raw (1/64 CSS pixel) representation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ObservationSize {
    pub inline_size: i32,
    pub block_size: i32,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ResizeObserverGeometry {
    pub content_rect: [f64; 4],
    pub content_box: ResizeObserverSize,
    pub border_box: ResizeObserverSize,
    pub device_pixel_content_box: ResizeObserverSize,
    pub observation_sizes: [ObservationSize; 3],
}
impl ResizeObserverGeometry {
    pub fn ObservationSize(self, box_: ResizeObserverBoxOptions) -> ObservationSize {
        self.observation_sizes[match box_ {
            ResizeObserverBoxOptions::ContentBox => 0,
            ResizeObserverBoxOptions::BorderBox => 1,
            ResizeObserverBoxOptions::DevicePixelContentBox => 2,
        }]
    }
}
/// This object is retained by the canonical Element. An observation weakly
/// references it; observing a detached Element does not create another tree.
pub trait ResizeObserverTarget {
    fn Data(&self) -> Rc<ResizeObserverData>;
    fn Geometry(&self) -> ResizeObserverGeometry;
    fn TargetDepth(&self) -> usize;
    fn IsInLockedSubtree(&self) -> bool;
    fn IsNonAtomicInline(&self) -> bool;
    fn IsContextDestroyed(&self) -> bool;
    fn ScheduleAnimation(&self);
    fn NodeId(&self) -> u64;
}
/// Element::ResizeObserverData is the sole strong observation map.
#[derive(Default)]
pub struct ResizeObserverData(RefCell<Vec<(usize, Rc<ResizeObservation>)>>);
#[derive(Clone)]
pub struct ResizeObserverEntry {
    pub target: Rc<dyn ResizeObserverTarget>,
    pub geometry: ResizeObserverGeometry,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeliveryTime {
    InsertionOrder,
    BeforeOthers,
}
pub trait ResizeObserverDelegate {
    fn OnResize(&self, entries: &[ResizeObserverEntry]);
    // resize_observer.h:50-53 source base delegate virtuals.
    fn Delivery(&self) -> DeliveryTime {
        DeliveryTime::InsertionOrder
    }
    fn SkipNonAtomicInlineObservations(&self) -> bool {
        false
    }
}
pub struct ResizeObservation {
    target: Weak<dyn ResizeObserverTarget>,
    observer: Rc<ResizeObserver>,
    observation_size: Cell<Option<ObservationSize>>,
    observed_box: ResizeObserverBoxOptions,
}
impl ResizeObservation {
    fn ComputeTargetSize(&self) -> ObservationSize {
        self.target
            .upgrade()
            .map_or(ObservationSize::default(), |target| {
                target.Geometry().ObservationSize(self.observed_box)
            })
    }
    // resize_observation.cc:75-102: size comparison precedes lock/inline gates.
    fn ObservationSizeOutOfSync(&self) -> bool {
        if self.observation_size.get() == Some(self.ComputeTargetSize()) {
            return false;
        }
        let Some(target) = self.target.upgrade() else {
            return false;
        };
        if target.IsInLockedSubtree() {
            return false;
        }
        if self.observer.delegate.SkipNonAtomicInlineObservations() && target.IsNonAtomicInline() {
            return false;
        }
        true
    }
}
pub struct ResizeObserver {
    delegate: Rc<dyn ResizeObserverDelegate>,
    observations: RefCell<Vec<Weak<ResizeObservation>>>,
    active: RefCell<Vec<Rc<ResizeObservation>>>,
    skipped: Cell<bool>,
    _controller: Weak<ResizeObserverController>,
}
impl ResizeObserver {
    pub fn Create(
        controller: &Rc<ResizeObserverController>,
        delegate: Rc<dyn ResizeObserverDelegate>,
    ) -> Rc<Self> {
        let observer = Rc::new(Self {
            delegate,
            observations: RefCell::new(vec![]),
            active: RefCell::new(vec![]),
            skipped: Cell::new(false),
            _controller: Rc::downgrade(controller),
        });
        controller.AddObserver(&observer);
        observer
    }
    fn id(&self) -> usize {
        self as *const Self as usize
    }
    // resize_observer.cc:86-120: changing the observed box replaces the
    // observation, removes the previous active entry and schedules animation.
    pub fn Observe(
        self: &Rc<Self>,
        target: Rc<dyn ResizeObserverTarget>,
        box_: ResizeObserverBoxOptions,
    ) {
        let data = target.Data();
        let previous = data
            .0
            .borrow()
            .iter()
            .find(|(id, _)| *id == self.id())
            .map(|(_, o)| o.clone());
        if let Some(previous) = previous {
            if previous.observed_box == box_ {
                return;
            }
            self.RemoveObservation(&previous);
            data.0.borrow_mut().retain(|(id, _)| *id != self.id());
        }
        let observation = Rc::new(ResizeObservation {
            target: Rc::downgrade(&target),
            observer: self.clone(),
            observation_size: Cell::new(None),
            observed_box: box_,
        });
        self.observations
            .borrow_mut()
            .push(Rc::downgrade(&observation));
        data.0.borrow_mut().push((self.id(), observation));
        target.ScheduleAnimation();
    }
    fn RemoveObservation(&self, previous: &Rc<ResizeObservation>) {
        self.observations
            .borrow_mut()
            .retain(|o| o.upgrade().is_some_and(|o| !Rc::ptr_eq(&o, previous)));
        self.active
            .borrow_mut()
            .retain(|o| !Rc::ptr_eq(o, previous));
    }
    pub fn Unobserve(&self, target: &dyn ResizeObserverTarget) {
        let data = target.Data();
        let previous = data
            .0
            .borrow()
            .iter()
            .find(|(id, _)| *id == self.id())
            .map(|(_, o)| o.clone());
        if let Some(previous) = previous {
            self.RemoveObservation(&previous);
            data.0.borrow_mut().retain(|(id, _)| *id != self.id());
        }
    }
    pub fn Disconnect(&self) {
        let observations = std::mem::take(&mut *self.observations.borrow_mut());
        for observation in observations.into_iter().filter_map(|o| o.upgrade()) {
            if let Some(target) = observation.target.upgrade() {
                target
                    .Data()
                    .0
                    .borrow_mut()
                    .retain(|(id, _)| *id != self.id());
            }
        }
        self.ClearObservations();
    }
    pub fn GatherObservations(&self, deeper_than: usize) -> usize {
        debug_assert!(self.active.borrow().is_empty());
        let mut shallowest = ResizeObserverController::DEPTH_BOTTOM;
        let observations = self
            .observations
            .borrow()
            .iter()
            .filter_map(Weak::upgrade)
            .collect::<Vec<_>>();
        for observation in observations {
            if !observation.ObservationSizeOutOfSync() {
                continue;
            }
            let depth = observation.target.upgrade().map_or(0, |t| t.TargetDepth());
            if depth > deeper_than {
                self.active.borrow_mut().push(observation);
                shallowest = shallowest.min(depth);
            } else {
                self.skipped.set(true);
            }
        }
        shallowest
    }
    pub fn DeliverObservations(&self) {
        let active = self.active.borrow().clone();
        if active.is_empty() {
            return;
        }
        let mut entries = Vec::new();
        for observation in active {
            let Some(target) = observation.target.upgrade() else {
                continue;
            };
            if target.IsContextDestroyed() {
                continue;
            }
            observation
                .observation_size
                .set(Some(observation.ComputeTargetSize()));
            entries.push(ResizeObserverEntry {
                geometry: target.Geometry(),
                target,
            });
        }
        if !entries.is_empty() {
            self.delegate.OnResize(&entries);
        }
        self.ClearObservations();
    }
    pub fn ClearObservations(&self) {
        self.active.borrow_mut().clear();
        self.skipped.set(false);
    }
    pub fn HasPendingActivity(&self) -> bool {
        !self.active.borrow().is_empty()
    }
    pub fn SkippedObservations(&self) -> bool {
        self.skipped.get()
    }
}
#[derive(Default)]
pub struct ResizeObserverController {
    observers: RefCell<Vec<Weak<ResizeObserver>>>,
    min_depth: Cell<usize>,
    loop_limit_error_dispatched: Cell<bool>,
}
impl ResizeObserverController {
    pub const DEPTH_BOTTOM: usize = 4096;
    fn Observers(&self) -> Vec<Rc<ResizeObserver>> {
        let mut observers = self.observers.borrow_mut();
        observers.retain(|o| o.strong_count() != 0);
        observers.iter().filter_map(Weak::upgrade).collect()
    }
    fn AddObserver(&self, observer: &Rc<ResizeObserver>) {
        let mut observers = self.observers.borrow_mut();
        observers
            .retain(|o| o.strong_count() != 0 && !std::ptr::eq(o.as_ptr(), Rc::as_ptr(observer)));
        match observer.delegate.Delivery() {
            DeliveryTime::InsertionOrder => observers.push(Rc::downgrade(observer)),
            DeliveryTime::BeforeOthers => observers.insert(0, Rc::downgrade(observer)),
        }
    }
    pub fn GatherObservations(&self) -> usize {
        let mut shallowest = Self::DEPTH_BOTTOM;
        for observer in self.Observers() {
            shallowest = shallowest.min(observer.GatherObservations(self.min_depth.get()));
        }
        self.min_depth.set(shallowest);
        shallowest
    }
    pub fn SkippedObservations(&self) -> bool {
        self.Observers().iter().any(|o| o.SkippedObservations())
    }
    pub fn DeliverObservations(&self) {
        for observer in self.Observers() {
            observer.DeliverObservations();
        }
    }
    pub fn ClearObservations(&self) {
        for observer in self.Observers() {
            observer.ClearObservations();
        }
    }
    pub fn ClearMinDepth(&self) {
        self.min_depth.set(0);
    }
    pub fn IsLoopLimitErrorDispatched(&self) -> bool {
        self.loop_limit_error_dispatched.get()
    }
    pub fn SetLoopLimitErrorDispatched(&self, value: bool) {
        self.loop_limit_error_dispatched.set(value);
    }
}

impl Drop for ResizeObserverController {
    fn drop(&mut self) {
        // Release Rust's temporary active-observation strong cycle when the
        // Window supplement retires. Element maps retain the source lifetime.
        self.ClearObservations();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Target {
        data: Rc<ResizeObserverData>,
        width: Cell<i32>,
        depth: usize,
        locked: Cell<bool>,
        dead: Cell<bool>,
    }
    impl Target {
        fn new(depth: usize) -> Rc<Self> {
            Rc::new(Self {
                data: Rc::default(),
                width: Cell::new(0),
                depth,
                locked: Cell::new(false),
                dead: Cell::new(false),
            })
        }
    }
    impl ResizeObserverTarget for Target {
        fn Data(&self) -> Rc<ResizeObserverData> {
            self.data.clone()
        }
        fn Geometry(&self) -> ResizeObserverGeometry {
            ResizeObserverGeometry {
                observation_sizes: [ObservationSize {
                    inline_size: self.width.get(),
                    block_size: 0,
                }; 3],
                ..Default::default()
            }
        }
        fn TargetDepth(&self) -> usize {
            self.depth
        }
        fn IsInLockedSubtree(&self) -> bool {
            self.locked.get()
        }
        fn IsNonAtomicInline(&self) -> bool {
            false
        }
        fn IsContextDestroyed(&self) -> bool {
            self.dead.get()
        }
        fn ScheduleAnimation(&self) {}
        fn NodeId(&self) -> u64 {
            self.depth as u64
        }
    }
    struct Delegate {
        name: u8,
        order: Rc<RefCell<Vec<u8>>>,
        before: bool,
        on_resize: RefCell<Option<Box<dyn Fn()>>>,
    }
    impl ResizeObserverDelegate for Delegate {
        fn OnResize(&self, entries: &[ResizeObserverEntry]) {
            assert!(!entries.is_empty());
            self.order.borrow_mut().push(self.name);
            if let Some(callback) = self.on_resize.borrow_mut().take() {
                callback();
            }
        }
        fn Delivery(&self) -> DeliveryTime {
            if self.before {
                DeliveryTime::BeforeOthers
            } else {
                DeliveryTime::InsertionOrder
            }
        }
    }
    fn delegate(name: u8, order: &Rc<RefCell<Vec<u8>>>, before: bool) -> Rc<Delegate> {
        Rc::new(Delegate {
            name,
            order: order.clone(),
            before,
            on_resize: RefCell::new(None),
        })
    }
    #[test]
    fn source_initial_zero_depth_loop_and_disconnect() {
        let controller = Rc::new(ResizeObserverController::default());
        let order = Rc::default();
        let target = Target::new(2);
        let observer = ResizeObserver::Create(&controller, delegate(1, &order, false));
        observer.Observe(target.clone(), ResizeObserverBoxOptions::ContentBox);
        assert_eq!(controller.GatherObservations(), 2);
        controller.DeliverObservations();
        assert_eq!(*order.borrow(), [1]);
        assert_eq!(
            controller.GatherObservations(),
            ResizeObserverController::DEPTH_BOTTOM
        );
        target.width.set(64);
        assert_eq!(
            controller.GatherObservations(),
            ResizeObserverController::DEPTH_BOTTOM
        );
        assert!(controller.SkippedObservations());
        controller.ClearObservations();
        controller.ClearMinDepth();
        assert_eq!(controller.GatherObservations(), 2);
        controller.DeliverObservations();
        observer.Disconnect();
        assert!(target.data.0.borrow().is_empty());
        controller.ClearMinDepth();
        assert_eq!(
            controller.GatherObservations(),
            ResizeObserverController::DEPTH_BOTTOM
        );
    }
    #[test]
    fn source_delivery_snapshot_reentry_and_destroyed_context() {
        let controller = Rc::new(ResizeObserverController::default());
        let order = Rc::default();
        let first_delegate = delegate(1, &order, false);
        let target = Target::new(3);
        let first = ResizeObserver::Create(&controller, first_delegate.clone());
        first.Observe(target.clone(), ResizeObserverBoxOptions::ContentBox);
        let before = ResizeObserver::Create(&controller, delegate(0, &order, true));
        before.Observe(target.clone(), ResizeObserverBoxOptions::BorderBox);
        let weak = Rc::downgrade(&first);
        *first_delegate.on_resize.borrow_mut() =
            Some(Box::new(move || weak.upgrade().unwrap().Disconnect()));
        assert_eq!(controller.GatherObservations(), 3);
        controller.DeliverObservations();
        assert_eq!(*order.borrow(), [0, 1]);
        assert!(!first.HasPendingActivity());
        assert_eq!(target.data.0.borrow().len(), 1);
        controller.ClearMinDepth();
        target.width.set(64);
        target.dead.set(true);
        assert_eq!(controller.GatherObservations(), 3);
        controller.DeliverObservations();
        assert_eq!(*order.borrow(), [0, 1]);
        before.Disconnect();
        assert!(target.data.0.borrow().is_empty());
    }
}
