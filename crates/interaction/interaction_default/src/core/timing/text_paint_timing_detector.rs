//! Chromium 6c1d401f paint/timing/text_paint_timing_detector.h:38-107,
//! .cc:33-278. The native Paint walk supplies one aggregated observation in
//! its original completion order. This owner alone keeps recorded/allow-repaint
//! state and presentation queues; it does not rescan DOM or Layout.
use super::{paint_timing_record::*, soft_navigation_heuristics::SoftNavigationHeuristics};
use foundation::gfx;
use interaction_dispatch::core::dom::events::event::EventTargetRef;
use interaction_input::base::time::time::TimeTicks;
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, VecDeque},
    rc::{Rc, Weak},
};

pub struct AggregatedTextPaint {
    pub node: EventTargetRef,
    pub layout_identity: PaintTimingLayoutIdentity,
    pub frame_visual_rect: gfx::Rect,
    pub root_visual_rect: gfx::RectF,
    pub element_timing_rect: gfx::RectF,
    pub color_is_fully_transparent: bool,
    pub has_text_shadow: bool,
    pub has_text_stroke: bool,
    pub ignore_depth: i32,
    pub document_element_is_invisible: bool,
}
/// Source integrations are native owning objects, not callbacks that choose
/// candidates. Hard-LCP tracking, element/container timing and HUD effects
/// remain explicit so an enabled producer cannot silently drop a client.
pub trait TextPaintTimingEnvironment: PaintTimingNodeContext {
    fn HasLargestContentfulPaintManager(&self) -> bool;
    fn InitializeLargestContentfulPaintTracking(&self, record: &Rc<PaintTimingRecord>);
    fn MaybeUpdateLargestIgnoredText(
        &self,
        record: &Rc<PaintTimingRecord>,
        ignore_depth: i32,
        document_element_is_invisible: bool,
    );
    fn TakeLargestIgnoredText(&self) -> Option<Rc<PaintTimingRecord>>;
    fn SoftNavigationHeuristics(&self) -> Option<Rc<SoftNavigationHeuristics>>;
    fn TextElementTimingNeeded(&self, node: &EventTargetRef) -> bool;
    fn MarkFirstContentfulPaint(&self);
    fn EmitTextDebuggingRect(&self, record: &Rc<PaintTimingRecord>);
    fn EmitContentfulPaintHudRect(&self, record: &Rc<PaintTimingRecord>, interaction: bool);
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum TextPaintStatus {
    Painted,
    AllowRepaint,
}
pub struct TextPaintTimingDetector {
    self_: Weak<Self>,
    environment: Rc<dyn TextPaintTimingEnvironment>,
    recorded: RefCell<HashMap<PaintTimingLayoutIdentity, TextPaintStatus>>,
    queued: RefCell<VecDeque<Rc<PaintTimingRecord>>>,
    frame_index: Cell<u32>,
    added_entry_in_latest_frame: Cell<bool>,
}
pub struct TextPaintTimingCallback {
    detector: Weak<TextPaintTimingDetector>,
    frame_index: u32,
}
impl TextPaintTimingCallback {
    pub fn OnFramePresented(
        self,
        timestamp: TimeTicks,
        info: DOMPaintTimingInfo,
        settled: &mut Vec<Rc<PaintTimingRecord>>,
    ) {
        if let Some(detector) = self.detector.upgrade() {
            detector.AssignPaintTimeToQueuedRecords(self.frame_index, timestamp, info, settled);
        }
    }
}
impl TextPaintTimingDetector {
    pub fn new(environment: Rc<dyn TextPaintTimingEnvironment>) -> Rc<Self> {
        Rc::new_cyclic(|self_| Self {
            self_: self_.clone(),
            environment,
            recorded: RefCell::new(HashMap::new()),
            queued: RefCell::new(VecDeque::new()),
            frame_index: Cell::new(1),
            added_entry_in_latest_frame: Cell::new(false),
        })
    }
    /// Native execution-context teardown releases Oilpan-style retained records;
    /// it does not synthesize presentation or notify a stopped manager.
    pub fn ContextDestroyed(&self) {
        self.queued.borrow_mut().clear();
        self.recorded.borrow_mut().clear();
        self.added_entry_in_latest_frame.set(false);
    }
    pub fn ShouldWalkObject(&self, identity: PaintTimingLayoutIdentity, has_node: bool) -> bool {
        has_node
            && self
                .recorded
                .borrow()
                .get(&identity)
                .is_none_or(|status| *status == TextPaintStatus::AllowRepaint)
    }
    pub fn ResetPaintTrackingOnInteraction(&self, identity: PaintTimingLayoutIdentity) {
        if let Some(status) = self.recorded.borrow_mut().get_mut(&identity) {
            *status = TextPaintStatus::AllowRepaint;
        }
    }
    /// Called by the canonical LayoutObject teardown route for weak-key cleanup.
    pub fn LayoutObjectDestroyed(&self, identity: PaintTimingLayoutIdentity) {
        self.recorded.borrow_mut().remove(&identity);
    }
    pub fn RecordAggregatedText(&self, paint: AggregatedTextPaint) {
        if paint.color_is_fully_transparent && !paint.has_text_shadow && !paint.has_text_stroke {
            return;
        }
        if !self.ShouldWalkObject(paint.layout_identity, true) {
            return;
        }
        assert!(!paint.frame_visual_rect.IsEmpty());
        let size = (paint.root_visual_rect.width() * paint.root_visual_rect.height()) as u64;
        let record = if size == 0 {
            PaintTimingRecord::NewText(
                &paint.node,
                paint.layout_identity,
                0,
                gfx::RectF::default(),
                gfx::Rect::default(),
                gfx::RectF::default(),
            )
        } else {
            PaintTimingRecord::NewText(
                &paint.node,
                paint.layout_identity,
                size,
                paint.element_timing_rect,
                paint.frame_visual_rect,
                paint.root_visual_rect,
            )
        };
        if paint.ignore_depth != 0 {
            if self.environment.HasLargestContentfulPaintManager() {
                self.environment.MaybeUpdateLargestIgnoredText(
                    &record,
                    paint.ignore_depth,
                    paint.document_element_is_invisible,
                );
            }
            return;
        }
        let repaint = self
            .recorded
            .borrow_mut()
            .insert(paint.layout_identity, TextPaintStatus::Painted)
            .is_some();
        if self.environment.HasLargestContentfulPaintManager() {
            self.environment
                .InitializeLargestContentfulPaintTracking(&record);
        }
        if let Some(heuristics) = self.environment.SoftNavigationHeuristics() {
            heuristics.InitializePaintTracking(&record);
            if let Some(context) = record.GetSoftNavigationContext() {
                context.AddPaintedArea(&record);
            }
        }
        record.SetIsNeededForElementTiming(
            !repaint && self.environment.TextElementTimingNeeded(&paint.node),
        );
        if record.IsNeededForLargestContentfulPaint()
            || record.IsNeededForInteractionContentfulPaint()
            || record.IsNeededForElementTiming()
        {
            self.QueueToMeasurePaintTime(record.clone());
        }
        self.environment.EmitTextDebuggingRect(&record);
    }
    fn QueueToMeasurePaintTime(&self, record: Rc<PaintTimingRecord>) {
        record.SetFrameIndex(self.frame_index.get());
        self.queued.borrow_mut().push_back(record);
        self.added_entry_in_latest_frame.set(true);
    }
    pub fn IsRecordingLargestTextPaint(&self) -> bool {
        self.environment.HasLargestContentfulPaintManager()
    }
    pub fn ReportLargestIgnoredText(&self) {
        if !self.environment.HasLargestContentfulPaintManager() {
            return;
        }
        let Some(record) = self.environment.TakeLargestIgnoredText() else {
            return;
        };
        self.environment.MarkFirstContentfulPaint();
        // TakeLargestIgnoredText already rejects removed/replaced native objects.
        let node = record
            .GetNode()
            .expect("retained largest ignored text Node");
        let identity = self
            .environment
            .CurrentLayoutIdentity(&node)
            .expect("largest ignored text LayoutObject");
        self.recorded
            .borrow_mut()
            .insert(identity, TextPaintStatus::Painted);
        self.QueueToMeasurePaintTime(record);
    }
    pub fn TakePaintTimingCallback(&self) -> Option<TextPaintTimingCallback> {
        if !self.added_entry_in_latest_frame.replace(false) {
            return None;
        }
        let frame = self.frame_index.get();
        let recording = self.IsRecordingLargestTextPaint();
        let records: Vec<_> = self
            .queued
            .borrow()
            .iter()
            .filter(|record| record.FrameIndex() == frame)
            .cloned()
            .collect();
        for record in records {
            if record.GetSoftNavigationContext().is_some() {
                self.environment.EmitContentfulPaintHudRect(&record, true);
            } else if recording {
                self.environment.EmitContentfulPaintHudRect(&record, false);
            }
        }
        self.frame_index
            .set(frame.checked_add(1).expect("text paint frame index"));
        Some(TextPaintTimingCallback {
            detector: self.self_.clone(),
            frame_index: frame,
        })
    }
    fn AssignPaintTimeToQueuedRecords(
        &self,
        frame: u32,
        timestamp: TimeTicks,
        info: DOMPaintTimingInfo,
        settled: &mut Vec<Rc<PaintTimingRecord>>,
    ) {
        loop {
            let record = {
                let mut queue = self.queued.borrow_mut();
                if queue
                    .front()
                    .is_none_or(|record| record.FrameIndex() > frame)
                {
                    break;
                }
                queue.pop_front().expect("front was checked")
            };
            assert!(!record.HasPaintTime());
            record.SetPaintTime(timestamp, info);
            settled.push(record);
        }
    }
}
