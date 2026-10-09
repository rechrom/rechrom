//! Compatibility re-export while DocumentEngine extraction moves callers to
//! the page-independent resource boundary.
pub use resource::{
    AwaitResource, LoadResponse, RequireResponse, ResourceLoadResult, ResourceLoadStatus,
    ResourceLoader, StartResource,
};
