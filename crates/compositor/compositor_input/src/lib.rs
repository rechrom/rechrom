//! Platform-neutral impl-input algorithms (cc InputHandler side).
//!
//! Host events are adapted by the `compositor` facade. This package owns only
//! compositor input decisions, never a Page or native event loop. The cc
//! input values both sides name live in the shared root `compositor_values`
//! and are re-exported here under their old paths.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub mod browser_controls;
pub mod browser_controls_offset_manager;
pub mod browser_controls_offset_manager_client;
pub mod elastic_overscroll_controller;
pub mod hit_test_list;
pub mod input_handler_types;
pub mod interaction_state;
pub mod mobile_optimized_viewport;
pub mod offset_tag;
pub mod performance;
pub mod scroll_begin;
pub mod scroll_end;
pub mod scroll_state;
pub mod scroll_state_data;
pub mod scroll_update;
pub mod scroll_utils;
pub mod scroll_velocity_tracker;
pub mod scrollbar_animation;
pub mod scrollbar_animation_controller;
pub mod scrollbar_controller;
pub mod scrollbar_controller_boundary;
pub mod scrollbar_flash;
pub mod single_scrollbar_animation_controller_thinning;
pub mod snap_geometry;
pub mod viewport;

pub use compositor_values::{
    actively_scrolling_type, browser_controls_params, browser_controls_state,
    compositor_commit_data, event_listener_properties, hit_test_opaqueness, input_types,
    layer_selection_bound, main_thread_scrolling_reason, overscroll_behavior,
    page_scale_animation, scroll_decision, scroll_hit_test_rect, scroll_snap_data,
    scroll_source_type, scrollbar, snap_selection_strategy, touch_action, touch_action_region,
    ScrollGranularity,
};
