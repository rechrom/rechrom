//! Chromium paint_timing.cc:235-387,391-457. Per-frame detector callbacks,
//! raw presentation stamps and coarsened PerformanceTimeline tasks retain the
//! original records. Widget transport supplies no paint/candidate algorithm.
use super::{
    paint_timing_record::{DOMPaintTimingInfo, PaintTimingRecord},
    window_performance::{PresentationTimingDetails, WindowPerformance},
};
use interaction_input::base::time::time::TimeTicks;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::{Rc, Weak},
};
pub type PaintTimingDetectorCallback =
    Box<dyn FnOnce(TimeTicks, DOMPaintTimingInfo, &mut Vec<Rc<PaintTimingRecord>>)>;
pub trait PaintTimingPresentationClient {
    fn OnPaintPresentation(&self, request: u64, details: PresentationTimingDetails);
}
pub trait PaintTimingPresentationEnvironment {
    fn Performance(&self) -> Option<Rc<WindowPerformance>>;
    fn RegisterPresentationTime(
        &self,
        client: Weak<dyn PaintTimingPresentationClient>,
        request: u64,
    );
    fn PostPerformanceTimelineTaskAt(&self, time: TimeTicks, task: Box<dyn FnOnce()>);
    fn OnRecordsPresented(&self, images: &[Rc<PaintTimingRecord>], texts: &[Rc<PaintTimingRecord>]);
}
struct PendingFrame {
    rendering_update_end: TimeTicks,
    images: Option<PaintTimingDetectorCallback>,
    texts: Option<PaintTimingDetectorCallback>,
}
pub struct PaintTimingPresentation {
    self_: Weak<Self>,
    environment: Rc<dyn PaintTimingPresentationEnvironment>,
    next: Cell<u64>,
    pending: RefCell<HashMap<u64, PendingFrame>>,
}
impl PaintTimingPresentation {
    pub fn new(environment: Rc<dyn PaintTimingPresentationEnvironment>) -> Rc<Self> {
        Rc::new_cyclic(|self_| Self {
            self_: self_.clone(),
            environment,
            next: Cell::new(0),
            pending: RefCell::new(HashMap::new()),
        })
    }
    pub fn MarkPaintTiming(
        &self,
        rendering_update_end: TimeTicks,
        images: Option<PaintTimingDetectorCallback>,
        texts: Option<PaintTimingDetectorCallback>,
    ) {
        if images.is_none() && texts.is_none() {
            return;
        }
        let request = self
            .next
            .get()
            .checked_add(1)
            .expect("paint presentation request");
        self.next.set(request);
        self.pending.borrow_mut().insert(
            request,
            PendingFrame {
                rendering_update_end,
                images,
                texts,
            },
        );
        let client: Rc<dyn PaintTimingPresentationClient> =
            self.self_.upgrade().expect("live paint timing owner");
        self.environment
            .RegisterPresentationTime(Rc::downgrade(&client), request);
    }
    pub fn ContextDestroyed(&self) {
        self.pending.borrow_mut().clear();
    }
    fn Flush(&self, frame: PendingFrame, raw: TimeTicks, info: DOMPaintTimingInfo) {
        if self.environment.Performance().is_none() {
            return;
        }
        let mut images = Vec::new();
        let mut texts = Vec::new();
        if let Some(callback) = frame.images {
            callback(raw, info, &mut images);
        }
        if let Some(callback) = frame.texts {
            callback(raw, info, &mut texts);
        }
        self.environment.OnRecordsPresented(&images, &texts);
    }
}
impl PaintTimingPresentationClient for PaintTimingPresentation {
    fn OnPaintPresentation(&self, request: u64, details: PresentationTimingDetails) {
        let Some(frame) = self.pending.borrow_mut().remove(&request) else {
            return;
        };
        let Some(performance) = self.environment.Performance() else {
            return;
        };
        let raw = details.timestamp;
        let mut info = DOMPaintTimingInfo {
            paint_time: performance.MonotonicTimeToDOMHighResTimeStamp(frame.rendering_update_end),
            presentation_time: performance.MonotonicTimeToDOMHighResTimeStamp(raw),
        };
        if !performance.CrossOriginIsolatedCapability() {
            let origin = performance.GetTimeOriginInternal().ToInternalValue();
            let delta = raw.ToInternalValue() - origin;
            let rounded =
                delta.div_euclid(4000) * 4000 + if delta.rem_euclid(4000) != 0 { 4000 } else { 0 };
            info.presentation_time = rounded as f64 / 1000.0;
            let target = TimeTicks::FromInternalValue(
                origin
                    .checked_add(rounded)
                    .expect("source presentation target"),
            );
            let owner = self.self_.clone();
            self.environment.PostPerformanceTimelineTaskAt(
                target,
                Box::new(move || {
                    if let Some(owner) = owner.upgrade() {
                        owner.Flush(frame, raw, info);
                    }
                }),
            );
        } else {
            self.Flush(frame, raw, info);
        }
    }
}
