#![allow(non_snake_case, non_camel_case_types)]
//! Document loading, resident parsing, and document resource lifetimes.
//! Page owns script execution, render readiness, layout and frame scheduling.
#[cfg(test)]
extern crate layoutng_replaced;

mod document_loader;
mod import_map;
mod module_resources;
mod resource_fetcher;
mod resource_loader;
mod text_decode;
mod url_reference;
mod web_fonts;

pub use document_loader::{
    DocumentLoadBudget, DocumentLoadProgress, DocumentLoader, DocumentLoaderStatus,
};
pub use module_resources::{ModuleLoadError, ModuleResources};
pub use resource_fetcher::{ResourceFetcher, ResourceFetcherClient};
pub use resource_loader::{
    AwaitResource, LoadResponse, RequireResponse, ResourceLoadResult, ResourceLoadStatus,
    ResourceLoader, StartResource,
};
pub use text_decode::DecodeText;
pub use url_reference::{ResolveCSSURLs, ResolveUrl};
pub use web_fonts::LoadUsedFontFaces;

pub fn ParseImportMap(
    source: &[u8],
) -> Result<std::collections::HashMap<Vec<u8>, Vec<u8>>, &'static str> {
    import_map::JSONCursor::new(source).ParseImportMap()
}
pub fn ResolveModuleReference(base: &[u8], reference: &[u8]) -> Vec<u8> {
    import_map::ResolveReferenceBytes(base, reference)
}

#[cfg(test)]
mod loading_tests;

#[cfg(test)]
mod streaming_tests;

#[cfg(test)]
mod module_resources_tests;
