//! Module assembly for source-relative Chromium translations.
//!
//! A same-stem Chromium `.h` and `.cc` pair maps to one Rust `.rs` file
//! under `cc/<directory>/<stem>.rs`.

pub mod animation {
    pub mod animation;
    pub mod keyframe_effect;
    pub mod keyframe_model;
    pub mod animation_id_provider;
    pub mod animation_events;
    pub mod animation_timeline;
    pub mod scroll_offset_animation_curve;
    pub mod scroll_offset_animation_curve_factory;
    pub mod scroll_offset_animations_impl;
}
pub mod paint {
    pub use layerize::commit::paint::display_item_list;
}
pub mod benchmarks {
    pub mod micro_benchmark_controller_impl;
}
pub mod debug {
    pub mod debug_colors;
    pub mod layer_tree_debug_state;
}
pub mod metrics {
    pub mod compositor_timing_history;
    pub mod submit_info;
}
pub mod resources {
    pub mod resource_pool;
    pub mod memory_history;
}
pub mod raster {
    pub mod task_graph_work_queue;
    pub mod synchronous_task_graph_runner;
    pub mod task_graph_runner;
    pub mod tile_task;
    pub mod task;
    pub mod task_category;
}
pub mod tiles {
    pub mod picture_layer_tiling;
    pub mod picture_layer_tiling_set;
    pub mod tiling_set_coverage_iterator;
    pub mod eviction_tile_priority_queue;
    pub mod raster_tile_priority_queue_all;
    pub mod tiling_set_eviction_queue;
    pub mod tiling_set_raster_queue_all;
    pub mod tiling_set_raster_queue_required;
    pub mod raster_tile_priority_queue_required;
    pub mod raster_tile_priority_queue;
    pub mod tile_task_manager;
    pub mod prioritized_tile;
    pub mod tile;
    pub mod tile_draw_info;
    pub mod tiles_with_resource_iterator;
    pub mod tile_manager;
    pub mod tile_manager_client;
    pub mod tiling_coverage_iterator;
    pub mod tiling_internal;
    pub mod mipmap_util;
    pub mod tile_index;
    pub mod tile_priority;
    pub mod tile_manager_settings;
}
pub mod base {
    pub mod rolling_time_delta_history;
    pub use layerize::commit::rtree;
    pub mod container_util;
    pub mod base_export;
    pub mod delayed_unique_notifier;
    pub mod invalidation_region;
    pub mod tiling_data;
    pub mod index_rect;
    pub mod math_util;
    pub mod region;
    pub mod reverse_spiral_iterator;
    pub mod spiral_iterator;
    pub mod simple_enclosed_region;
    pub mod synced_property;
    pub mod unique_notifier;
    pub mod protected_sequence_synchronizer;
}
pub mod input {
    pub mod hit_test_opaqueness;

    pub mod actively_scrolling_type;
    pub mod browser_controls_state;
    pub mod browser_controls_offset_tag_modifications;
    pub mod browser_controls_offset_tags;
    pub mod browser_controls_offset_manager;
    pub mod browser_controls_offset_manager_client;
    pub mod compositor_input_interfaces;
    pub mod event_listener_properties;
    pub mod input_handler;
    pub mod layer_selection_bound;
    pub mod scroll_elasticity_helper;
    pub mod main_thread_scrolling_reason;
    pub mod overscroll_behavior;
    pub mod page_scale_animation;
    pub mod scroll_state_data;
    pub mod scroll_state;
    pub mod scroll_utils;
    pub mod scroll_velocity_tracker;
    pub mod scrollbar;
    pub mod scrollbar_controller;
    pub mod scrollbar_animation_controller;
    pub mod single_scrollbar_animation_controller_thinning;
    pub mod scroll_snap_data;
    pub mod snap_selection_strategy;
    pub mod touch_action;
}
pub mod layers {
    pub mod render_surface_impl;
    pub mod layer;
    pub mod layer_impl;
    pub mod layer_list_iterator;
    pub mod picture_layer;
    pub mod solid_color_layer;
    pub mod solid_color_layer_impl;
    pub mod surface_layer_impl;
    pub mod surface_layer;
    pub mod scrollbar_layer_base;
    pub mod scrollbar_layer_impl_base;
    pub mod painted_scrollbar_layer;
    pub mod painted_scrollbar_layer_impl;
    pub mod solid_color_scrollbar_layer;
    pub mod solid_color_scrollbar_layer_impl;
    pub mod picture_layer_impl;
    pub mod tile_based_layer_impl;
    pub mod recording_source;
    pub mod layer_collections;
    pub mod scroll_hit_test_rect;
    pub mod touch_action_region;
    pub mod draw_properties;
    pub mod append_quads_context;
    pub mod append_quads_data;
    pub mod draw_mode;
    pub mod effect_tree_layer_list_iterator;
    pub mod viewport;
    pub mod tile_size_calculator;
}
pub mod scheduler {
    pub mod commit_earlyout_reason;
    pub mod draw_result;
    pub mod scheduler_settings;
    pub mod begin_frame_tracker;
    pub mod scheduler_state_machine;
    pub mod scheduler;
    pub mod headless_scheduler_state_machine;
    pub mod slim_scheduler_state_machine;
    pub mod video_frame_controller;
    pub mod webview_scheduler_state_machine;
}
pub mod trees {
    pub mod animated_paint_worklet_tracker;
    pub mod begin_main_frame_trace_id;
    pub mod browser_controls_params;
    pub mod clip_node;
    pub mod compositor_commit_data;
    pub mod commit_state;
    pub mod debug_rect_history;
    pub mod layer_tree_settings;
    pub mod image_animation_controller;
    pub mod layer_tree_impl;
    pub mod client_layer_tree_host_impl;
    pub mod layer_tree_host_impl;
    pub mod damage_tracker;
    pub mod draw_property_utils;
    pub mod property_animation_state;
    pub mod presentation_time_callback_buffer;
    pub mod property_tree;
    pub mod property_ids;
    pub mod damage_reason;
    pub mod effect_node;
    pub mod frame_data;
    pub mod managed_memory_policy;
    pub mod mobile_optimized_viewport_util;
    pub mod occlusion;
    pub mod occlusion_tracker;
    pub mod raster_capabilities;
    pub mod render_frame_metadata;
    pub mod scroll_source_type;
    pub mod sticky_position_constraint;
    pub mod swap_promise;
    pub mod scroll_node;
    pub mod scroll_elasticity_utils;
    pub mod target_property;
    pub mod throttle_decider;
    pub mod tree_synchronizer;
    pub mod unbounded_frame_sink_handler;
    pub mod transform_node;
    pub mod viewport_property_ids;
    pub mod proxy_common;
}
