//! Chromium 6c1d401f paint/timing/largest_contentful_paint_calculator.h:38-331,
//! .cc:53-758. Candidate, pending-image, reporting and metric state share the
//! original Rc<Record>; presentation never reconstructs a candidate from bounds.
use super::{paint_timing_record::*, window_performance::WindowPerformance};
use foundation::AtomicString;
use interaction_dispatch::core::dom::events::event::EventTargetRef;
use interaction_input::base::time::time::TimeTicks;
use std::{
    cell::{Cell, Ref, RefCell},
    rc::{Rc, Weak},
};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LargestContentfulPaintType(pub u32);
impl LargestContentfulPaintType {
    pub const NONE: Self = Self(0);
    pub const IMAGE: Self = Self(1);
    pub const TEXT: Self = Self(2);
    pub const ANIMATED_IMAGE: Self = Self(4);
    pub const VIDEO: Self = Self(8);
    pub const DATA_URI: Self = Self(16);
    pub const PNG: Self = Self(32);
    pub const JPG: Self = Self(64);
    pub const WEBP: Self = Self(128);
    pub const SVG: Self = Self(256);
    pub const GIF: Self = Self(512);
    pub const AVIF: Self = Self(1024);
    pub const CROSS_ORIGIN: Self = Self(8192);
}
#[derive(Clone, Debug, Default)]
pub struct LargestContentfulPaintDetails {
    pub largest_image_paint_time: TimeTicks,
    pub largest_image_paint_size: u64,
    pub resource_load_timings: ResourceLoadTimings,
    pub largest_contentful_paint_type: LargestContentfulPaintType,
    pub largest_contentful_paint_image_bpp: f64,
    pub largest_text_paint_time: TimeTicks,
    pub largest_text_paint_size: u64,
    pub largest_contentful_paint_image_request_priority: Option<RequestPriority>,
}
#[derive(Clone, Copy, Debug)]
pub struct LcpFeatures {
    pub use_largest_painted_image_for_lcp_candidate: bool,
    pub entropy_ignored_for_first_video_frame_lcp: bool,
    pub soft_navigation_trace_events: bool,
}
// BASE_FEATURE(kSoftNavigationTraceEvents), calculator.cc:32.
static SOFT_NAVIGATION_TRACE_EVENTS: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(true);
impl LcpFeatures {
    pub fn FromSourceFeatures() -> Self {
        Self {
            use_largest_painted_image_for_lcp_candidate:
                foundation::RuntimeEnabledFeatures::UseLargestPaintedImageForLCPCandidateEnabled(),
            entropy_ignored_for_first_video_frame_lcp:
                foundation::RuntimeEnabledFeatures::EntropyIgnoredForFirstVideoFrameLCPEnabled(),
            soft_navigation_trace_events: SOFT_NAVIGATION_TRACE_EVENTS
                .load(std::sync::atomic::Ordering::Relaxed),
        }
    }
    pub fn SetSoftNavigationTraceEventsEnabled(value: bool) {
        SOFT_NAVIGATION_TRACE_EVENTS.store(value, std::sync::atomic::Ordering::Relaxed);
    }
}
pub trait LcpDelegate {
    fn EmitLcpPerformanceEntry(
        &self,
        info: DOMPaintTimingInfo,
        paint_size: u64,
        load_time: TimeTicks,
        id: AtomicString,
        url: String,
        element: Option<EventTargetRef>,
    );
    fn OnLcpMetricsForReportingChanged(&self);
    fn IsHardNavigation(&self) -> bool;
}
pub struct LcpNodeInfo {
    pub id: AtomicString,
    pub element: Option<EventTargetRef>,
    pub is_in_shadow_tree: bool,
    pub node_id: i32,
    pub node_name: String,
    pub loading_attribute: Option<AtomicString>,
}
#[derive(Clone)]
pub struct LcpFrameTraceInfo {
    pub is_main_frame: bool,
    pub is_outermost_main_frame: bool,
    pub is_embedded_frame: bool,
    pub loader_id: String,
    pub frame_id: String,
    pub performance_timeline_navigation_id: u64,
}
#[derive(Clone, Copy)]
pub enum LcpTraceKind {
    WebExposedText,
    WebExposedImage,
    MetricsText,
    MetricsImage,
    NoMetricsImage,
}
pub struct LcpTrace {
    pub kind: LcpTraceKind,
    pub record: Option<Rc<PaintTimingRecord>>,
    pub timestamp: TimeTicks,
    pub candidate_index: u32,
    pub hard_navigation: bool,
    pub frame: LcpFrameTraceInfo,
}
pub trait LcpEnvironment: PaintTimingNodeContext {
    fn NodeInfo(&self, node: &EventTargetRef) -> LcpNodeInfo;
    fn IsOutermostMainFrame(&self) -> bool;
    fn IsSameOriginWithWindow(&self, media: &dyn MediaTiming) -> bool;
    fn CountRemovedCandidateWhilePaintTimePending(&self);
    fn IsTracing(&self) -> bool;
    fn FrameTraceInfo(&self, performance: &Rc<WindowPerformance>) -> Option<LcpFrameTraceInfo>;
    fn EmitTrace(&self, trace: LcpTrace);
}
#[derive(Default)]
pub struct LcpCandidates {
    image: RefCell<Option<Rc<PaintTimingRecord>>>,
    text: RefCell<Option<Rc<PaintTimingRecord>>>,
}
impl LcpCandidates {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn MaybeUpdateImage(&self, record: Rc<PaintTimingRecord>) {
        assert!(record.IsImageRecord());
        Self::Update(&self.image, record);
    }
    pub fn MaybeUpdateText(&self, record: Rc<PaintTimingRecord>) {
        assert!(record.IsTextRecord());
        Self::Update(&self.text, record);
    }
    fn Update(slot: &RefCell<Option<Rc<PaintTimingRecord>>>, record: Rc<PaintTimingRecord>) {
        let mut slot = slot.borrow_mut();
        if record.IsEffectiveSizeLargerThan(slot.as_deref()) {
            *slot = Some(record);
        }
    }
    pub fn CandidateImage(&self) -> Option<Rc<PaintTimingRecord>> {
        self.image.borrow().clone()
    }
    pub fn CandidateText(&self) -> Option<Rc<PaintTimingRecord>> {
        self.text.borrow().clone()
    }
}
pub struct LargestContentfulPaintCalculator {
    performance: Rc<WindowPerformance>,
    delegate: Weak<dyn LcpDelegate>,
    environment: Rc<dyn LcpEnvironment>,
    features: LcpFeatures,
    largest_reported_size: Cell<u64>,
    web_exposed_candidate_count: Cell<u32>,
    ukm_image_candidate_count: Cell<u32>,
    ukm_text_candidate_count: Cell<u32>,
    latest: RefCell<LargestContentfulPaintDetails>,
    largest_text: RefCell<Option<Rc<PaintTimingRecord>>>,
    largest_painted_image: RefCell<Option<Rc<PaintTimingRecord>>>,
    largest_pending_image: RefCell<Option<Rc<PaintTimingRecord>>>,
}
impl LargestContentfulPaintCalculator {
    pub fn new(
        performance: Rc<WindowPerformance>,
        delegate: Weak<dyn LcpDelegate>,
        environment: Rc<dyn LcpEnvironment>,
        features: LcpFeatures,
    ) -> Rc<Self> {
        Rc::new(Self {
            performance,
            delegate,
            environment,
            features,
            largest_reported_size: Cell::new(0),
            web_exposed_candidate_count: Cell::new(0),
            ukm_image_candidate_count: Cell::new(0),
            ukm_text_candidate_count: Cell::new(0),
            latest: RefCell::new(Default::default()),
            largest_text: RefCell::new(None),
            largest_painted_image: RefCell::new(None),
            largest_pending_image: RefCell::new(None),
        })
    }
    fn Delegate(&self) -> Rc<dyn LcpDelegate> {
        self.delegate
            .upgrade()
            .expect("calculator belongs to its retained navigation context")
    }
    pub fn LatestLcpDetails(&self) -> Ref<'_, LargestContentfulPaintDetails> {
        self.latest.borrow()
    }
    pub fn IsEligibleForLcp(&self, record: &PaintTimingRecord) -> bool {
        if record.IsTextRecord() {
            return record.EffectiveVisualSize() > 0;
        }
        assert!(record.EffectiveVisualSize() > 0);
        if record.GetEffectiveVisualSizeResult().is_viewport_covered {
            return false;
        }
        let media = record
            .GetMediaTiming()
            .expect("image eligibility is queried while its source media lives");
        if media.GetFirstVideoFrameTime().is_some()
            && (!self.Delegate().IsHardNavigation()
                || self.features.entropy_ignored_for_first_video_frame_lcp)
        {
            return true;
        }
        record.GetEffectiveVisualSizeResult().is_min_entropy_met
    }
    pub fn ShouldTrackForPaintTiming(&self, record: &PaintTimingRecord) -> bool {
        assert!(record.IsImageRecord());
        self.IsEligibleForLcp(record)
            && record.IsEffectiveSizeLargerThan(self.largest_painted_image.borrow().as_deref())
    }
    pub fn OnImageFirstPaint(&self, record: Rc<PaintTimingRecord>) {
        assert!(record.IsImageRecord());
        LcpCandidates::Update(&self.largest_pending_image, record);
    }
    pub fn OnPendingImageRemoved(&self, record: &Rc<PaintTimingRecord>) {
        if self
            .largest_pending_image
            .borrow()
            .as_ref()
            .is_some_and(|pending| Rc::ptr_eq(pending, record))
        {
            self.largest_pending_image.borrow_mut().take();
        }
    }
    pub fn LargestPaintedOrPendingImage(&self) -> Option<Rc<PaintTimingRecord>> {
        let painted = self.largest_painted_image.borrow().clone();
        let pending = self.largest_pending_image.borrow().clone();
        if painted.is_none()
            || pending
                .as_ref()
                .is_some_and(|pending| pending.IsEffectiveSizeLargerThan(painted.as_deref()))
        {
            pending
        } else {
            painted
        }
    }
    pub fn OnRecordsPresented(
        &self,
        images: &[Rc<PaintTimingRecord>],
        texts: &[Rc<PaintTimingRecord>],
    ) {
        assert!(self.Delegate().IsHardNavigation());
        let candidates = LcpCandidates::new();
        self.ProcessLcpCandidates(images, true, &candidates);
        self.ProcessLcpCandidates(texts, false, &candidates);
        self.OnFramePresented(&candidates);
    }
    fn ProcessLcpCandidates(
        &self,
        records: &[Rc<PaintTimingRecord>],
        image: bool,
        candidates: &LcpCandidates,
    ) {
        let mut largest_removed: Option<Rc<PaintTimingRecord>> = None;
        for record in records {
            assert_eq!(record.IsImageRecord(), image);
            if !record.IsNeededForLargestContentfulPaint() {
                continue;
            }
            if record.WasNodeRemoved(self.environment.as_ref()) {
                if record.IsEffectiveSizeLargerThan(largest_removed.as_deref()) {
                    largest_removed = Some(record.clone());
                }
                continue;
            }
            if image {
                candidates.MaybeUpdateImage(record.clone());
            } else {
                candidates.MaybeUpdateText(record.clone());
            }
        }
        let candidate = if image {
            candidates.CandidateImage()
        } else {
            candidates.CandidateText()
        };
        if let Some(removed) = largest_removed {
            if removed.IsEffectiveSizeLargerThan(candidate.as_deref()) {
                let largest = if image {
                    self.LargestPaintedOrPendingImage()
                } else {
                    self.largest_text.borrow().clone()
                };
                if removed.IsEffectiveSizeLargerThan(largest.as_deref()) {
                    self.environment
                        .CountRemovedCandidateWhilePaintTimePending();
                }
            }
        }
    }
    pub fn OnFramePresented(&self, candidates: &LcpCandidates) {
        if let Some(candidate) = candidates.CandidateImage() {
            LcpCandidates::Update(&self.largest_painted_image, candidate);
        }
        if let Some(candidate) = candidates.CandidateText() {
            LcpCandidates::Update(&self.largest_text, candidate);
        }
        self.MaybeFlushCandidates();
    }
    pub fn MaybeFlushCandidates(&self) {
        let image = self.UpdateMetricsIfLargestImagePaintChanged();
        let text = self.UpdateMetricsIfLargestTextPaintChanged();
        if image || text {
            self.Delegate().OnLcpMetricsForReportingChanged();
        }
        self.UpdateWebExposedLargestContentfulPaintIfNeeded();
    }
    fn UpdateLatestLcpDetailsTypeIfNeeded(&self) {
        let mut latest = self.latest.borrow_mut();
        if latest.largest_text_paint_size < latest.largest_image_paint_size
            || (latest.largest_text_paint_size == latest.largest_image_paint_size
                && latest.largest_text_paint_time >= latest.largest_image_paint_time)
        {
            return;
        }
        latest.largest_contentful_paint_type = LargestContentfulPaintType::TEXT;
    }
    fn UpdateMetricsIfLargestImagePaintChanged(&self) -> bool {
        let Some(record) = self.LargestPaintedOrPendingImage() else {
            return false;
        };
        let hard = self.Delegate().IsHardNavigation();
        let time = if hard && record.HasFirstAnimatedFrameTime() {
            record.FirstAnimatedFrameTime()
        } else {
            record.PaintTime()
        };
        if !hard && time.is_null() {
            return false;
        }
        {
            let latest = self.latest.borrow();
            if time == latest.largest_image_paint_time
                && record.EffectiveVisualSize() == latest.largest_image_paint_size
            {
                return false;
            }
        }
        let mut kind = LargestContentfulPaintType::NONE;
        let media = record.GetMediaTiming();
        if let Some(media) = &media {
            if media.GetFirstVideoFrameTime().is_some() {
                kind.0 |= LargestContentfulPaintType::VIDEO.0;
            } else if media.IsPaintedFirstFrame() {
                kind.0 |= LargestContentfulPaintType::ANIMATED_IMAGE.0;
            }
            kind.0 |= LargestContentfulPaintType::IMAGE.0 | ImageType(&media.MediaType()).0;
            if media.IsDataUrl() {
                kind.0 |= LargestContentfulPaintType::DATA_URI.0;
            }
            let url = media.Url();
            let http =
                url::Url::parse(&url).is_ok_and(|url| matches!(url.scheme(), "http" | "https"));
            if !url.is_empty()
                && http
                && self.environment.IsOutermostMainFrame()
                && !self.environment.IsSameOriginWithWindow(media.as_ref())
            {
                kind.0 |= LargestContentfulPaintType::CROSS_ORIGIN.0;
            }
        }
        {
            let mut latest = self.latest.borrow_mut();
            latest.largest_contentful_paint_type = kind;
            if let Some(media) = &media {
                latest.resource_load_timings = media.ResourceLoadTimings();
            }
            latest.largest_image_paint_time = time;
            latest.largest_image_paint_size = record.EffectiveVisualSize();
            latest.largest_contentful_paint_image_bpp = record.EntropyForLCP();
            latest.largest_contentful_paint_image_request_priority = record.RequestPriority();
        }
        self.UpdateLatestLcpDetailsTypeIfNeeded();
        if hard && self.environment.IsTracing() {
            if !time.is_null() && record.IsSufficientlyLoadedForReporting() {
                self.TraceCandidate(LcpTraceKind::MetricsImage, Some(record), time);
            } else {
                self.TraceCandidate(LcpTraceKind::NoMetricsImage, None, TimeTicks::new());
            }
        }
        true
    }
    fn UpdateMetricsIfLargestTextPaintChanged(&self) -> bool {
        let Some(record) = self.largest_text.borrow().clone() else {
            return false;
        };
        if !record.HasPaintTime() {
            assert!(!self.Delegate().IsHardNavigation());
            return false;
        }
        {
            let latest = self.latest.borrow();
            if latest.largest_text_paint_time == record.PaintTime()
                && latest.largest_text_paint_size == record.EffectiveVisualSize()
            {
                return false;
            }
        }
        {
            let mut latest = self.latest.borrow_mut();
            latest.largest_text_paint_time = record.PaintTime();
            latest.largest_text_paint_size = record.EffectiveVisualSize();
        }
        self.UpdateLatestLcpDetailsTypeIfNeeded();
        if self.Delegate().IsHardNavigation() && self.environment.IsTracing() {
            let time = record.PaintTime();
            self.TraceCandidate(LcpTraceKind::MetricsText, Some(record), time);
        }
        true
    }
    fn UpdateWebExposedLargestContentfulPaintIfNeeded(&self) {
        let image = if self.features.use_largest_painted_image_for_lcp_candidate {
            self.largest_painted_image.borrow().clone()
        } else {
            self.LargestPaintedOrPendingImage()
        };
        let text = self.largest_text.borrow().clone();
        let image_size = image.as_ref().map_or(0, |r| r.EffectiveVisualSize());
        let text_size = text.as_ref().map_or(0, |r| r.EffectiveVisualSize());
        if image_size > text_size {
            if image_size > self.largest_reported_size.get() {
                let record = image.expect("positive image size");
                if record.HasPaintTime() {
                    self.EmitImage(record);
                }
            }
        } else if text_size > self.largest_reported_size.get() {
            let record = text.expect("positive text size");
            if record.HasPaintTime() {
                self.EmitText(record);
            }
        }
    }
    fn EmitImage(&self, record: Rc<PaintTimingRecord>) {
        let (Some(media), Some(node)) = (record.GetMediaTiming(), record.GetNode()) else {
            return;
        };
        self.largest_reported_size.set(record.EffectiveVisualSize());
        let mut url = media.Url();
        if media.IsDataUrl() {
            url = url.chars().take(100).collect();
        }
        let info = self.environment.NodeInfo(&node);
        let element = if info.is_in_shadow_tree {
            None
        } else {
            info.element
        };
        let id = if element.is_some() {
            info.id
        } else {
            AtomicString::default()
        };
        self.Delegate().EmitLcpPerformanceEntry(
            record.PaintTimingInfo(),
            record.EffectiveVisualSize(),
            record.LoadTime(),
            id,
            url,
            element,
        );
        if self.Delegate().IsHardNavigation() || self.features.soft_navigation_trace_events {
            let time = record.PaintTime();
            self.TraceCandidate(LcpTraceKind::WebExposedImage, Some(record), time);
        }
    }
    fn EmitText(&self, record: Rc<PaintTimingRecord>) {
        let Some(node) = record.GetNode() else {
            return;
        };
        self.largest_reported_size.set(record.EffectiveVisualSize());
        let info = self.environment.NodeInfo(&node);
        let element = if info.is_in_shadow_tree {
            None
        } else {
            info.element
        };
        let id = if element.is_some() {
            info.id
        } else {
            AtomicString::default()
        };
        self.Delegate().EmitLcpPerformanceEntry(
            record.PaintTimingInfo(),
            record.EffectiveVisualSize(),
            TimeTicks::new(),
            id,
            String::new(),
            element,
        );
        let time = record.PaintTime();
        self.TraceCandidate(LcpTraceKind::WebExposedText, Some(record), time);
    }
    fn TraceCandidate(
        &self,
        kind: LcpTraceKind,
        record: Option<Rc<PaintTimingRecord>>,
        timestamp: TimeTicks,
    ) {
        if !self.environment.IsTracing() {
            return;
        }
        let Some(frame) = self.environment.FrameTraceInfo(&self.performance) else {
            return;
        };
        let counter = match kind {
            LcpTraceKind::WebExposedText | LcpTraceKind::WebExposedImage => {
                &self.web_exposed_candidate_count
            }
            LcpTraceKind::MetricsText => &self.ukm_text_candidate_count,
            _ => &self.ukm_image_candidate_count,
        };
        let index = counter
            .get()
            .checked_add(1)
            .expect("LCP trace candidate index");
        counter.set(index);
        self.environment.EmitTrace(LcpTrace {
            kind,
            record,
            timestamp,
            candidate_index: index,
            hard_navigation: self.Delegate().IsHardNavigation(),
            frame,
        });
    }
    /// .cc:681-758. Input rectangles have already traversed GeometryMapper and
    /// BlinkSpaceToDIPs in the original Paint owner, before this pure algorithm.
    pub fn ComputeEffectiveVisualSize(
        media: &dyn MediaTiming,
        mapped: &foundation::gfx::RectF,
        bounding_client: &foundation::gfx::RectF,
        intrinsic_area: u64,
        viewport_area: u64,
    ) -> EffectiveVisualSizeResult {
        let mut result = EffectiveVisualSizeResult::default();
        let mut size = (mapped.width() * mapped.height()) as u64;
        if size >= viewport_area {
            result.size = size;
            result.is_viewport_covered = true;
            return result;
        }
        if intrinsic_area == 0 {
            return result;
        }
        let bounding = (bounding_client.width() * bounding_client.height()) as u64;
        if bounding > intrinsic_area {
            size = (size as f64 * intrinsic_area as f64 / bounding as f64) as u64;
        }
        result.entropy = media.ContentSizeForEntropy() as f64 * 8.0 / size as f64;
        result.is_min_entropy_met = result.entropy >= 0.05;
        result.size = size;
        result
    }
}
fn ImageType(name: &str) -> LargestContentfulPaintType {
    match name {
        "svg" => LargestContentfulPaintType::SVG,
        "gif" => LargestContentfulPaintType::GIF,
        "png" => LargestContentfulPaintType::PNG,
        "jpg" => LargestContentfulPaintType::JPG,
        "avif" => LargestContentfulPaintType::AVIF,
        "webp" => LargestContentfulPaintType::WEBP,
        _ => LargestContentfulPaintType::NONE,
    }
}
