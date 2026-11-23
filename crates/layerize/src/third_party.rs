//! Module assembly for source-relative Chromium translations.
//!
//! A same-stem Chromium `.h` and `.cc` pair maps to one Rust `.rs` file
//! under `third_party/<directory>/<stem>.rs`.

pub mod blink {
    pub mod renderer {
        pub mod platform {
            pub mod graphics {
                pub mod paint {
                    pub mod raster_invalidator;
                    pub mod display_item_raster_invalidator;
                    pub mod geometry_mapper;
                    pub mod geometry_mapper_transform_cache;
                    pub mod geometry_mapper_clip_cache;
                    pub mod float_clip_rect;
                    pub mod raster_invalidation_tracking;
                }
                pub mod compositing {
                    pub mod layers_as_json;
                    pub mod paint_artifact_compositor;
                    pub mod paint_chunks_to_cc_layer;
                    pub mod content_layer_client_impl;
                    pub mod chunk_to_layer_mapper;
                    pub mod adjust_mask_layer_geometry;
                    pub mod pending_layer;
                    pub mod property_tree_manager;
                }
            }
        }
    }
}
