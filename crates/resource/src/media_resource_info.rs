//! ImageResource::ImageResourceInfoImpl, image_resource.cc:159-205,211-217.
//! The request and its response timing are retained once by the Resource;
//! ImageResourceContent holds an alias, not a second resource snapshot.
#![allow(non_snake_case)]
use image_resource::media_timing::{ImageResourceInfo, RequestPriority, ResourceLoadTimings};
use std::{cell::Cell, sync::Arc, time::Instant};
use url_loader::{RequestDestination, URLRequest, URLResponse};
pub struct MediaResourceInfo {
    request: Arc<URLRequest>,
    base_priority: RequestPriority,
    priority: Cell<RequestPriority>,
    timings: Cell<ResourceLoadTimings>,
    expected_content_length: Cell<Option<u64>>,
}
impl MediaResourceInfo {
    pub(crate) fn new(request: Arc<URLRequest>) -> Self {
        // ResourceFetcher::TypeToPriority; keep Blink's priority before the
        // HTTP backend's three-level scheduling projection.
        let base_priority = match request.destination {
            RequestDestination::kDocument
            | RequestDestination::kStyleSheet
            | RequestDestination::kFont => RequestPriority::VeryHigh,
            RequestDestination::kScript
                if request.script_type == url_loader::ScriptType::Module
                    || request.defer == url_loader::RequestDefer::None =>
            {
                RequestPriority::High
            }
            RequestDestination::kFetch | RequestDestination::kReport => RequestPriority::High,
            _ => RequestPriority::Low,
        };
        Self {
            request,
            base_priority,
            priority: Cell::new(base_priority),
            timings: Cell::new(ResourceLoadTimings {
                discovery_time: Some(Instant::now()),
                ..Default::default()
            }),
            expected_content_length: Cell::new(None),
        }
    }
    pub(crate) fn ResponseReceived(&self, response: &URLResponse) {
        let mut timing = self.timings.get();
        timing.load_start = response.send_start;
        timing.load_end = response.response_end;
        self.timings.set(timing);
        self.expected_content_length
            .set(response.expected_content_length);
    }
    pub(crate) fn SetPriority(&self, priority: url_loader::ResourceLoadPriority) {
        self.priority.set(match priority {
            url_loader::ResourceLoadPriority::Normal => self.base_priority,
            url_loader::ResourceLoadPriority::High => RequestPriority::High,
            url_loader::ResourceLoadPriority::VeryHigh => RequestPriority::VeryHigh,
        });
    }
}
impl ImageResourceInfo for MediaResourceInfo {
    fn IsAutomaticUpgrade(&self) -> bool {
        self.request.is_automatic_upgrade
    }
    fn Url(&self) -> String {
        self.request.url.clone()
    }
    fn RequestPriority(&self) -> Option<RequestPriority> {
        Some(self.priority.get())
    }
    fn ResourceLoadTimings(&self) -> ResourceLoadTimings {
        self.timings.get()
    }
    fn ExpectedContentLength(&self) -> Option<u64> {
        self.expected_content_length.get()
    }
}
