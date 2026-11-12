// Copyright 2014 The Chromium Authors. BSD-style license.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: ui/events/gesture_detection/gesture_configuration.h:206-275.
// The existing provider configuration remains the package input value.
#[derive(Clone, Debug)]
pub struct GestureProviderConfig {
    pub touch_slop: f32,
    pub stylus_slop: f32,
    pub double_tap_slop: f32,
    pub showpress_timeout_us: i64,
    pub shortpress_timeout_us: i64,
    pub longpress_timeout_us: i64,
    pub double_tap_timeout_us: i64,
    pub double_tap_min_time_us: i64,
    pub minimum_fling_velocity: f32,
    pub maximum_fling_velocity: f32,
    pub double_tap_enabled: bool,
    pub single_tap_repeat_interval: i32,
    pub span_slop: f32,
    pub min_scaling_span: f32,
    pub min_pinch_update_span_delta: f32,
}
impl Default for GestureProviderConfig {
    // gesture_configuration.h:206-275: generic desktop configuration.
    fn default() -> Self {
        Self {
            touch_slop: 15.,
            stylus_slop: 20.,
            double_tap_slop: 20.,
            showpress_timeout_us: 150_000,
            shortpress_timeout_us: 400_000,
            longpress_timeout_us: 500_000,
            double_tap_timeout_us: 300_000,
            double_tap_min_time_us: 40_000,
            minimum_fling_velocity: 30.,
            maximum_fling_velocity: 17000.,
            double_tap_enabled: false,
            single_tap_repeat_interval: 3,
            span_slop: 30.,
            min_scaling_span: 125.,
            min_pinch_update_span_delta: 0.,
        }
    }
}
