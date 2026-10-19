//! Concrete value storage and injected execution boundaries for the Page façade.
//! Canonical compositor owners/algorithms remain in the source-relative modules; these modules
//! implement external owned output values, never an alternative layer graph.
pub mod cpu_raster;
pub mod frame;

pub mod metadata;

pub mod layer;

pub mod main_host;
pub mod paint;
pub mod paint_services;

pub mod display_item_list;

pub mod geometry;

pub mod types;

pub mod skia;

pub mod surface;

pub mod layer_list;

pub mod draw;

pub mod host_layer;

pub mod input;

pub mod scheduler;

pub mod resources;

pub mod debug;

pub mod host;

pub mod host_resources;

pub mod sync;

pub mod frame_sink;

pub mod host_owners;

pub mod host_metrics;

pub mod host_services;

pub mod layer_context;

pub mod host_types;

pub mod native_services;

pub(crate) mod native_animation;
pub(crate) mod native_core_services;
pub(crate) mod native_frame_sink;
pub(crate) mod native_host_action;
pub(crate) mod native_host_core;
pub(crate) mod native_host_frame_sink;
pub(crate) mod native_host_lifecycle;
pub(crate) mod native_host_metadata;
pub(crate) mod native_host_picture;
pub(crate) mod native_images;
pub(crate) mod native_latency;
pub mod native_metrics;
pub(crate) mod native_pac_scroll;
pub(crate) mod native_scheduler;
pub mod owner_types;
pub mod picture_types;

pub mod host_picture;

pub(crate) mod execution;

pub(crate) mod raster_resources;

pub(crate) mod client_resources;

pub(crate) mod pool_services;

pub(crate) mod tile_execution;

pub(crate) mod main_endpoint;

pub(crate) mod engine_core;

pub(crate) mod native_render;

pub(crate) mod engine_factory;

pub(crate) mod engine_queries;

pub(crate) mod engine_assembly;

pub(crate) mod native_begin_frame;

pub(crate) mod native_resource_events;
pub(crate) mod native_resources;

pub(crate) mod native_animation_services;

pub(crate) mod native_client;

pub(crate) mod native_worklet;

pub(crate) mod native_frame_build;
pub mod native_ui;

pub(crate) mod native_tree_environment;

pub(crate) mod native_activation;

pub(crate) mod native_scrollbar;

pub(crate) mod native_software_images;
