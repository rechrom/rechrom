//! Platform-independent Chromium touch recognition and acknowledgement filtering.
pub mod filtered_gesture_provider;
pub mod gesture_configuration;
mod gesture_detector;
mod gesture_event_data_packet;
pub mod gesture_provider;
mod scale_gesture_detector;
pub mod touch_action_filter;
pub mod touch_disposition_gesture_filter;
pub mod velocity_tracker;
