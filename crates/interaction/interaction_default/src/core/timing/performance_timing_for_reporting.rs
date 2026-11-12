//! Chromium 6c1d401f performance_timing_for_reporting.h:113-145,
//! .cc:29-90,463-505; web_performance_metrics_for_reporting.h:27-51.
//! This is the LCP reporting owner. It maps original ticks through the live
//! DocumentLoader timing, independently of the web performance time origin.
use super::{
    largest_contentful_paint_calculator::*, paint_timing_record::RequestPriority,
    window_performance::WindowPerformance,
};
use interaction_input::base::time::time::TimeTicks;
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};
#[derive(Clone, Debug, Default)]
pub struct ResourceLoadTimingsForReporting {
    pub discovery_time: Option<i64>,
    pub load_start: Option<i64>,
    pub load_end: Option<i64>,
}
/// Pseudo-wall resource times are signed microseconds. Paint times are source
/// integer milliseconds expressed in seconds; merged ticks remain unclamped.
#[derive(Clone, Debug)]
pub struct LargestContentfulPaintDetailsForReporting {
    pub image_paint_time: f64,
    pub image_paint_size: u64,
    pub resource_load_timings: ResourceLoadTimingsForReporting,
    pub type_: LargestContentfulPaintType,
    pub image_bpp: f64,
    pub text_paint_time: f64,
    pub text_paint_size: u64,
    pub image_request_priority: Option<RequestPriority>,
    pub merged_unclamped_paint_time: Option<TimeTicks>,
    pub performance_timeline_navigation_id: u64,
}
impl Default for LargestContentfulPaintDetailsForReporting {
    fn default() -> Self {
        Self {
            image_paint_time: 0.0,
            image_paint_size: 0,
            resource_load_timings: Default::default(),
            type_: LargestContentfulPaintType::NONE,
            image_bpp: 0.0,
            text_paint_time: 0.0,
            text_paint_size: 0,
            image_request_priority: None,
            merged_unclamped_paint_time: None,
            performance_timeline_navigation_id: 1,
        }
    }
}
pub struct PerformanceTimingForReporting {
    performance: Weak<WindowPerformance>,
    largest_contentful_paint_details_for_metrics:
        RefCell<LargestContentfulPaintDetailsForReporting>,
    first_input_or_scroll_notified_timestamp: Cell<u64>,
}
impl PerformanceTimingForReporting {
    pub(crate) fn new(performance: Weak<WindowPerformance>) -> Rc<Self> {
        Rc::new(Self {
            performance,
            largest_contentful_paint_details_for_metrics: Default::default(),
            first_input_or_scroll_notified_timestamp: Cell::new(0),
        })
    }
    fn MonotonicTimeToIntegerMilliseconds(&self, time: TimeTicks) -> u64 {
        let Some(performance) = self.performance.upgrade() else {
            return 0;
        };
        let Some(microseconds) = performance
            .Environment()
            .MonotonicTimeToPseudoWallMicroseconds(time)
        else {
            return 0;
        };
        (performance.ClampTimeResolutionMicroseconds(microseconds) as f64 / 1000.0) as u64
    }
    pub fn PopulateLargestContentfulPaintDetailsForReporting(
        &self,
        timing: &LargestContentfulPaintDetails,
    ) -> LargestContentfulPaintDetailsForReporting {
        let resource = &timing.resource_load_timings;
        let environment = self
            .performance
            .upgrade()
            .map(|performance| performance.Environment());
        let pseudo_wall = |time| {
            environment
                .as_ref()
                .and_then(|environment| environment.ResourceTimeToPseudoWallMicroseconds(time))
        };
        let text = timing.largest_text_paint_size;
        let image = timing.largest_image_paint_size;
        let merged = if text == 0 && image == 0 {
            None
        } else if text == image {
            Some(
                timing
                    .largest_text_paint_time
                    .min(timing.largest_image_paint_time),
            )
        } else if text > image {
            Some(timing.largest_text_paint_time)
        } else {
            Some(timing.largest_image_paint_time)
        };
        LargestContentfulPaintDetailsForReporting {
            image_paint_time: self
                .MonotonicTimeToIntegerMilliseconds(timing.largest_image_paint_time)
                as f64
                / 1000.0,
            image_paint_size: image,
            resource_load_timings: ResourceLoadTimingsForReporting {
                discovery_time: resource.discovery_time.and_then(pseudo_wall),
                load_start: resource.load_start.and_then(pseudo_wall),
                load_end: resource.load_end.and_then(pseudo_wall),
            },
            type_: timing.largest_contentful_paint_type,
            image_bpp: timing.largest_contentful_paint_image_bpp,
            text_paint_time: self.MonotonicTimeToIntegerMilliseconds(timing.largest_text_paint_time)
                as f64
                / 1000.0,
            text_paint_size: text,
            image_request_priority: timing.largest_contentful_paint_image_request_priority,
            merged_unclamped_paint_time: merged,
            performance_timeline_navigation_id: 1,
        }
    }
    pub fn SetLargestContentfulPaintDetailsForMetrics(
        &self,
        details: &LargestContentfulPaintDetails,
    ) {
        *self
            .largest_contentful_paint_details_for_metrics
            .borrow_mut() = self.PopulateLargestContentfulPaintDetailsForReporting(details);
    }
    pub fn LargestContentfulPaintDetailsForMetrics(
        &self,
    ) -> LargestContentfulPaintDetailsForReporting {
        self.largest_contentful_paint_details_for_metrics
            .borrow()
            .clone()
    }
    pub fn SetFirstInputOrScrollNotifiedTimestamp(&self, time: TimeTicks) {
        assert_eq!(self.first_input_or_scroll_notified_timestamp.get(), 0);
        self.first_input_or_scroll_notified_timestamp
            .set(self.MonotonicTimeToIntegerMilliseconds(time));
    }
    pub fn FirstInputOrScrollNotifiedTimestamp(&self) -> u64 {
        self.first_input_or_scroll_notified_timestamp.get()
    }
}
