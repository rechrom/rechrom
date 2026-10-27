//! Rechrom-owned boundary types used where Chromium core refers to services
//! owned by another package. Nothing in this module counts as translated
//! Chromium source.

pub mod animation_host;
pub mod begin_frame;
pub mod browser_controls;
pub mod commit_state;
pub mod commit_types;
pub mod cubic_bezier;
pub mod damage_context;
pub mod debug_rect;
pub mod display_item_list;
pub mod draw_property;
pub mod elasticity_context;
pub mod enum_set;
pub mod frame_data;
pub mod geometry;
pub mod history;
pub mod image_animation;
pub mod input_geometry;
pub mod input_handler;
pub mod input_services;
pub mod input_types;
pub mod layer_impl;
pub mod layer_tree_host_impl;
pub mod layer_tree_impl;
pub mod main_layer;
pub mod micro_benchmark;
pub mod occlusion;
pub mod paint_geometry;
pub mod paint_worklet;
pub mod pending_layer;
pub mod picture_layer;
pub mod picture_layer_impl;
pub mod presentation;
pub mod prioritized_tile;
pub mod property_tree;
pub mod property_tree_manager;
pub mod property_types;
pub mod raster_task;
pub mod recording_source;
pub mod region_backend;
pub mod render_metadata;
pub mod resource_pool;
pub mod resource_types;
pub mod scheduler;
pub mod scheduler_host;
pub mod scroll_elasticity;
pub mod scrollbar;
pub mod scrollbar_animation;
pub mod scrollbar_layer_base;
pub mod scrollbar_layer_impl_base;
pub mod settings;
pub mod snap_geometry;
pub mod solid_color_layer;
pub mod solid_color_layer_impl;
pub mod solid_color_scrollbar_layer;
pub mod solid_color_scrollbar_layer_impl;
pub mod subnormal_float;
pub mod task_runner;
pub mod throttle;
pub mod tile;
pub mod tile_based_layer_impl;
pub mod time;
pub mod trace_sink;
pub mod tree_sync;
pub mod unbounded;
pub mod viewport;

/// Compatibility export for the platform-neutral structured trace value now
/// owned by the scheduler package.
pub use compositor_scheduler::TraceValue;
pub mod content_layer_client_impl;
pub mod picture_layer_tiling;
pub mod tile_manager;

pub mod paint_artifact_compositor;
pub mod paint_chunks_to_cc_layer;

pub mod render_surface_impl;

pub mod host_picture;

pub mod raster_invalidator;

pub mod client_layer_tree_host_impl;
