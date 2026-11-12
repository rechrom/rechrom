//! MediaTiming's platform-neutral resource facts. Chromium 6c1d401f:
//! platform/loader/fetch/media_timing.h:18-69. Instants retain the native
//! monotonic domain; None represents a null base::TimeTicks.
#![allow(non_snake_case)]
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestPriority {
    Unresolved,
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ResourceLoadTimings {
    pub load_start: Option<Instant>,
    pub load_end: Option<Instant>,
    pub discovery_time: Option<Instant>,
}
pub trait MediaTiming {
    fn Url(&self) -> String;
    fn MediaType(&self) -> String;
    fn RequestPriority(&self) -> Option<RequestPriority>;
    fn IsDataUrl(&self) -> bool;
    fn IsPaintedFirstFrame(&self) -> bool;
    fn IsSufficientContentLoadedForPaint(&self) -> bool;
    fn GetFirstVideoFrameTime(&self) -> Option<Instant>;
    fn ContentSizeForEntropy(&self) -> u64;
    fn ResourceLoadTimings(&self) -> ResourceLoadTimings;
}
/// ImageResourceInfo's required timing/response portion. Both Resource and
/// ImageResourceContent retain this same owner (image_resource.cc:159-205).
pub trait ImageResourceInfo {
    fn Url(&self) -> String;
    fn IsAutomaticUpgrade(&self) -> bool;
    fn RequestPriority(&self) -> Option<RequestPriority>;
    fn ResourceLoadTimings(&self) -> ResourceLoadTimings;
    fn ExpectedContentLength(&self) -> Option<u64>;
}
