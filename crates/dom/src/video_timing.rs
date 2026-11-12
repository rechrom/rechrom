//! Chromium core/loader/resource/video_timing.h:19-93; retained by the
//! HTMLVideoElement first-frame lifecycle, html_video_element.cc:770-798.
use image_resource::media_timing::{MediaTiming, RequestPriority, ResourceLoadTimings};
use std::{cell::Cell, time::Instant};
pub struct VideoTiming {
    url: String,
    is_loaded: Cell<bool>,
    first_frame_time: Cell<Option<Instant>>,
    content_size: Cell<u64>,
}
impl VideoTiming {
    pub fn new(url: String) -> Self {
        Self {
            url,
            is_loaded: Cell::new(false),
            first_frame_time: Cell::new(None),
            content_size: Cell::new(0),
        }
    }
    pub fn SetFirstVideoFrameTime(&self, time: Instant) {
        self.first_frame_time.set(Some(time));
    }
    pub fn SetIsSufficientContentLoadedForPaint(&self) {
        self.is_loaded.set(true);
    }
    pub fn SetContentSizeForEntropy(&self, bytes_to_first_frame: u64) {
        self.content_size.set(bytes_to_first_frame);
    }
}
impl MediaTiming for VideoTiming {
    fn Url(&self) -> String {
        self.url.clone()
    }
    fn MediaType(&self) -> String {
        "video".to_owned()
    }
    fn RequestPriority(&self) -> Option<RequestPriority> {
        None
    }
    fn IsDataUrl(&self) -> bool {
        self.url
            .get(..5)
            .is_some_and(|scheme| scheme.eq_ignore_ascii_case("data:"))
    }
    fn IsPaintedFirstFrame(&self) -> bool {
        self.first_frame_time.get().is_some()
    }
    fn IsSufficientContentLoadedForPaint(&self) -> bool {
        self.is_loaded.get()
    }
    fn GetFirstVideoFrameTime(&self) -> Option<Instant> {
        self.first_frame_time.get()
    }
    fn ContentSizeForEntropy(&self) -> u64 {
        self.content_size.get()
    }
    // The pinned VideoTiming explicitly returns null for all three timings;
    // h:68-84 TODO(crbug.com/40058142), unlike ImageResourceInfo's live timing.
    fn ResourceLoadTimings(&self) -> ResourceLoadTimings {
        ResourceLoadTimings::default()
    }
}
