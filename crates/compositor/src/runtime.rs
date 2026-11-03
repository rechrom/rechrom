//! Concrete value storage and injected execution boundaries for the Page façade.
//! Canonical compositor owners/algorithms remain in the source-relative modules; these modules
//! implement external owned output values, never an alternative layer graph.
pub mod frame;
pub mod raster_execution;

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

pub mod services;

pub(crate) mod animation_host;
pub(crate) mod core_services;
pub(crate) mod host_action;
pub(crate) mod host_core;
pub(crate) mod host_frame_sink;
pub(crate) mod host_lifecycle;
pub(crate) mod host_metadata;
pub(crate) mod host_picture_services;
pub(crate) mod images;
pub(crate) mod input_client;
pub(crate) mod latency_info_swap_promise_monitor;
pub(crate) mod layer_tree_frame_sink_endpoint;
pub mod metrics;
pub mod owner_types;
pub(crate) mod pac_scroll;
pub mod picture_types;
pub(crate) mod scheduler_services;

pub mod host_picture;

pub(crate) mod execution;

pub(crate) mod raster_resources;

pub(crate) mod client_resources;

pub(crate) mod pool_services;

pub(crate) mod tile_execution;

pub(crate) mod main_endpoint;

pub(crate) mod engine_core;

pub(crate) mod render;

pub(crate) mod engine_factory;

pub(crate) mod engine_queries;

pub(crate) mod engine_assembly;

pub(crate) mod begin_frame;

pub(crate) mod resource_events;
pub(crate) mod resource_services;

pub(crate) mod animation_services;

pub(crate) mod client;

pub(crate) mod worklet;

pub(crate) mod frame_build;
pub mod ui;

pub(crate) mod tree_environment;

pub(crate) mod activation;

pub(crate) mod scrollbar;

pub(crate) mod software_image_decode_cache;
