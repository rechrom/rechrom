#![allow(non_snake_case, non_camel_case_types)]

//! Page-independent resource loading, identity and decoding.
//!
//! Each `ResourceEngine` belongs to one document/open operation and owns its
//! request context over shared transport and decoder services. It does not know
//! about DOM, CSSOM, layout, paint, tiles, windows or threads.

mod engine;
mod xml_http_request_registry;
mod xml_http_request_transport;
pub use xml_http_request_registry::{RegisteredXMLHttpRequestOperation, XMLHttpRequestRegistry};
pub use xml_http_request_transport::*;
mod fetcher_lifecycle;
pub use fetcher_lifecycle::{
    FetcherCancellableTask, FetcherTaskRunner, FetcherWakeTask, ResourceCancellationClient,
    ResourceLoadEventPolicy,
};
mod raw_resource;
mod text_track_resource;
pub use text_track_resource::{TextTrackResource, TextTrackResourceClient};
mod resource_loader;
pub use raw_resource::{RawResource, RawResourceObserver};

pub use decode::{DecodeEffect, DecodeMutation, DecodedImageResource, DecodedVideoFrame};
pub use engine::{ResourceEngine, ResourceMemoryDiagnostics};
pub use resource_loader::{
    AwaitResource, LoadResponse, RequireResponse, ResourceLoadResult, ResourceLoadStatus,
    ResourceLoader, StartResource,
};

pub mod xml_http_request;

pub mod media_resource_info;
