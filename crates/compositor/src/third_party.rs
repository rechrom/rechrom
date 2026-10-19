//! Module assembly for source-relative Chromium translations.
//!
//! A same-stem Chromium `.h` and `.cc` pair maps to one Rust `.rs` file
//! under `third_party/<directory>/<stem>.rs`.

pub mod blink {
    pub mod renderer {
        pub mod platform {
            pub mod graphics {
                pub mod paint {
                    pub mod raster_invalidator {
                        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/third_party/blink/renderer/platform/graphics/paint/raster_invalidator.rs"));
                    }
                    pub mod display_item_raster_invalidator {
                        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/third_party/blink/renderer/platform/graphics/paint/display_item_raster_invalidator.rs"));
                    }
                    pub mod geometry_mapper {
                        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/third_party/blink/renderer/platform/graphics/paint/geometry_mapper.rs"));
                    }
                    pub mod geometry_mapper_transform_cache {
                        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/third_party/blink/renderer/platform/graphics/paint/geometry_mapper_transform_cache.rs"));
                    }
                    pub mod geometry_mapper_clip_cache {
                        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/third_party/blink/renderer/platform/graphics/paint/geometry_mapper_clip_cache.rs"));
                    }
                    pub mod float_clip_rect {
                        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/third_party/blink/renderer/platform/graphics/paint/float_clip_rect.rs"));
                    }
                    pub mod raster_invalidation_tracking {
                        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/third_party/blink/renderer/platform/graphics/paint/raster_invalidation_tracking.rs"));
                    }
                }
                pub mod compositing {
                    pub mod layers_as_json {
                        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/third_party/blink/renderer/platform/graphics/compositing/layers_as_json.rs"));
                    }
                    pub mod paint_artifact_compositor {
                        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.rs"));
                    }
                    pub mod paint_chunks_to_cc_layer {
                        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.rs"));
                    }
                    pub mod content_layer_client_impl {
                        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/third_party/blink/renderer/platform/graphics/compositing/content_layer_client_impl.rs"));
                    }
                    pub mod chunk_to_layer_mapper {
                        include!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/src/third_party/blink/renderer/platform/graphics/compositing/chunk_to_layer_mapper.rs"
                ));
                    }
                    pub mod adjust_mask_layer_geometry {
                        include!(concat!(
                            env!("CARGO_MANIFEST_DIR"),
                            "/src/third_party/blink/renderer/platform/graphics/compositing/adjust_mask_layer_geometry.rs"
                        ));
                    }
                    pub mod pending_layer {
                        include!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/src/third_party/blink/renderer/platform/graphics/compositing/pending_layer.rs"
                ));
                    }
                    pub mod property_tree_manager {
                        include!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/src/third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.rs"
                ));
                    }
                }
            }
        }
    }
}
