//! cc/raster service interfaces and values. The cc impl side selects raster
//! work and owns its completion accounting; the injected raster service
//! implements these interfaces. Both name them here, so the raster service
//! depends only on cc/viz value roots.

// Chromium identifiers keep their source spelling.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub mod raster_buffer_provider;
pub mod raster_source;
pub mod ui_resource;
