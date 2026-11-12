//! Chromium 6c1d401f paint/timing/largest_contentful_paint_manager.h:32-127,
//! .cc:22-259. Hard navigation owns this manager; each soft navigation instead
//! owns its own calculator. Stopping one never stops the other.
use super::{
    largest_contentful_paint_calculator::*, paint_timing_record::*,
    window_performance::WindowPerformance,
};
use foundation::AtomicString;
use interaction_dispatch::core::dom::events::event::EventTargetRef;
use interaction_input::base::time::time::TimeTicks;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
pub trait LargestContentfulPaintManagerEnvironment: LcpEnvironment {
    fn SetLargestContentfulPaintDetailsForMetrics(&self, details: &LargestContentfulPaintDetails);
    fn NotifyLoaderPerformanceTimingChanged(&self);
    fn OnLargestContentfulPaint(&self, presentation_time: TimeTicks);
    fn RecordContainsFullViewportImage(&self, value: bool);
    fn NowTicks(&self) -> TimeTicks;
}
pub struct LargestContentfulPaintManager {
    performance: Rc<WindowPerformance>,
    environment: Rc<dyn LargestContentfulPaintManagerEnvironment>,
    calculator: RefCell<Option<Rc<LargestContentfulPaintCalculator>>>,
    largest_ignored_text: RefCell<Option<Rc<PaintTimingRecord>>>,
    largest_ignored_image: RefCell<Option<Rc<PaintTimingRecord>>>,
    contains_full_viewport_image: Cell<bool>,
}
impl LargestContentfulPaintManager {
    pub fn new(
        performance: Rc<WindowPerformance>,
        environment: Rc<dyn LargestContentfulPaintManagerEnvironment>,
        features: LcpFeatures,
    ) -> Rc<Self> {
        let manager = Rc::new(Self {
            performance: performance.clone(),
            environment: environment.clone(),
            calculator: RefCell::new(None),
            largest_ignored_text: RefCell::new(None),
            largest_ignored_image: RefCell::new(None),
            contains_full_viewport_image: Cell::new(false),
        });
        let delegate: Rc<dyn LcpDelegate> = manager.clone();
        *manager.calculator.borrow_mut() = Some(LargestContentfulPaintCalculator::new(
            performance,
            Rc::downgrade(&delegate),
            environment,
            features,
        ));
        manager
    }
    pub fn ContextDestroyed(&self) {
        self.calculator.borrow_mut().take();
        self.largest_ignored_text.borrow_mut().take();
        self.largest_ignored_image.borrow_mut().take();
    }
    pub fn IsRecording(&self) -> bool {
        self.calculator.borrow().is_some()
    }
    pub fn Calculator(&self) -> Option<Rc<LargestContentfulPaintCalculator>> {
        self.calculator.borrow().clone()
    }
    pub fn OnFirstInputOrScroll(&self) {
        assert!(self.calculator.borrow_mut().take().is_some());
        if self.environment.IsOutermostMainFrame() {
            self.environment
                .RecordContainsFullViewportImage(self.contains_full_viewport_image.get());
        }
    }
    pub fn InitializePaintTracking(&self, record: &Rc<PaintTimingRecord>, ignore_depth: i32) {
        let calculator = self.Calculator().expect("hard LCP has not stopped");
        if record.IsImageRecord() {
            self.contains_full_viewport_image.set(
                self.contains_full_viewport_image.get()
                    || record.GetEffectiveVisualSizeResult().is_viewport_covered,
            );
            if calculator.ShouldTrackForPaintTiming(record) {
                record.SetIsNeededForLargestContentfulPaint(true);
                if ignore_depth == 0 {
                    calculator.OnImageFirstPaint(record.clone());
                }
            }
        } else if calculator.IsEligibleForLcp(record) {
            record.SetIsNeededForLargestContentfulPaint(true);
        }
    }
    pub fn OnFramePresented(
        &self,
        images: &[Rc<PaintTimingRecord>],
        texts: &[Rc<PaintTimingRecord>],
    ) {
        if let Some(calculator) = self.Calculator() {
            calculator.OnRecordsPresented(images, texts);
        }
    }
    fn LargestIgnoredText(&self) -> Option<Rc<PaintTimingRecord>> {
        self.largest_ignored_text
            .borrow()
            .as_ref()
            .filter(|record| !record.WasNodeRemoved(self.environment.as_ref()))
            .cloned()
    }
    fn LargestIgnoredImage(&self) -> Option<Rc<PaintTimingRecord>> {
        self.largest_ignored_image
            .borrow()
            .as_ref()
            .filter(|record| {
                !record.WasNodeRemoved(self.environment.as_ref())
                    && record.GetMediaTiming().is_some()
            })
            .cloned()
    }
    pub fn TakeLargestIgnoredText(&self) -> Option<Rc<PaintTimingRecord>> {
        assert!(self.IsRecording());
        let record = self.LargestIgnoredText();
        self.largest_ignored_text.borrow_mut().take();
        record
    }
    pub fn TakeLargestIgnoredImage(&self) -> Option<Rc<PaintTimingRecord>> {
        let calculator = self.Calculator().expect("hard LCP has not stopped");
        let record = self.LargestIgnoredImage();
        if let Some(record) = &record {
            calculator.OnImageFirstPaint(record.clone());
        }
        self.largest_ignored_image.borrow_mut().take();
        record
    }
    pub fn MaybeUpdateLargestIgnoredText(
        &self,
        record: &Rc<PaintTimingRecord>,
        ignore_depth: i32,
        document_element_is_invisible: bool,
    ) {
        assert!(self.IsRecording());
        if ignore_depth != 1 || !document_element_is_invisible {
            return;
        }
        self.InitializePaintTracking(record, ignore_depth);
        if record.IsNeededForLargestContentfulPaint()
            && record.IsEffectiveSizeLargerThan(self.LargestIgnoredText().as_deref())
        {
            *self.largest_ignored_text.borrow_mut() = Some(record.clone());
        }
    }
    pub fn MaybeUpdateLargestIgnoredImage(
        &self,
        record: &Rc<PaintTimingRecord>,
        ignore_depth: i32,
        document_element_is_invisible: bool,
    ) {
        assert!(self.IsRecording());
        if ignore_depth != 1 || !document_element_is_invisible {
            return;
        }
        let media = record
            .GetMediaTiming()
            .expect("ignored ImageRecord has source media");
        if !media.IsSufficientContentLoadedForPaint() {
            return;
        }
        record.SetLoadTime(self.environment.NowTicks());
        record.SetIsSufficientlyLoadedForReporting();
        self.InitializePaintTracking(record, ignore_depth);
        if record.IsNeededForLargestContentfulPaint()
            && record.IsEffectiveSizeLargerThan(self.largest_ignored_image.borrow().as_deref())
        {
            *self.largest_ignored_image.borrow_mut() = Some(record.clone());
        }
    }
    pub fn OnImageRemoved(
        &self,
        pending: Option<&Rc<PaintTimingRecord>>,
        layout: PaintTimingLayoutIdentity,
        media: Option<&Rc<dyn MediaTiming>>,
    ) {
        let calculator = self.Calculator().expect("hard LCP has not stopped");
        if let Some(record) = pending.filter(|record| record.IsNeededForLargestContentfulPaint()) {
            calculator.OnPendingImageRemoved(record);
        }
        let ignored = self.LargestIgnoredImage();
        let same = ignored.as_ref().is_some_and(|record| {
            record.LayoutIdentity() == layout
                && match (record.GetMediaTiming(), media) {
                    (Some(a), Some(b)) => Rc::ptr_eq(&a, b),
                    _ => false,
                }
        });
        if ignored.is_none() || same {
            self.largest_ignored_image.borrow_mut().take();
        }
    }
}
impl LcpDelegate for LargestContentfulPaintManager {
    fn IsHardNavigation(&self) -> bool {
        true
    }
    fn EmitLcpPerformanceEntry(
        &self,
        info: DOMPaintTimingInfo,
        size: u64,
        load: TimeTicks,
        id: AtomicString,
        url: String,
        element: Option<EventTargetRef>,
    ) {
        if !self.IsRecording() {
            return;
        }
        self.performance
            .OnLargestContentfulPaintUpdated(info, size, load, id, url, element);
    }
    fn OnLcpMetricsForReportingChanged(&self) {
        let Some(calculator) = self.Calculator() else {
            return;
        };
        let details = calculator.LatestLcpDetails();
        self.environment
            .SetLargestContentfulPaintDetailsForMetrics(&details);
        self.environment.NotifyLoaderPerformanceTimingChanged();
        let text = details.largest_text_paint_size > details.largest_image_paint_size
            || (details.largest_text_paint_size == details.largest_image_paint_size
                && details.largest_text_paint_time < details.largest_image_paint_time);
        let time = if text {
            details.largest_text_paint_time
        } else {
            details.largest_image_paint_time
        };
        drop(details);
        self.environment.OnLargestContentfulPaint(time);
    }
}
