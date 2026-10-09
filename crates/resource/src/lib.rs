#![allow(non_snake_case, non_camel_case_types)]

//! Page-independent resource loading, identity and decoding.
//!
//! Each `ResourceEngine` belongs to one document/open operation and owns its
//! request context over shared transport and decoder services. It does not know
//! about DOM, CSSOM, layout, paint, tiles, windows or threads.

mod engine;
mod resource_loader;

pub use decode::{DecodeEffect, DecodeMutation, DecodedImageResource};
pub use engine::ResourceEngine;
pub use resource_loader::{
    AwaitResource, LoadResponse, RequireResponse, ResourceLoadResult, ResourceLoadStatus,
    ResourceLoader, StartResource,
};
