#![allow(non_snake_case, non_camel_case_types)]
//! Document loading, resident parsing, and document resource lifetimes.
//! Page owns script execution, render readiness, layout and frame scheduling.
#[cfg(test)]
extern crate layoutng_replaced;

mod diagnostic_read_ahead;
mod document_loader;
mod document_policy;
mod navigation;
pub use navigation::{
    DocumentLoaderNavigationState, HistoryCommitType, SameDocumentCommit,
    SameDocumentCommitNotification, SameDocumentCommitStage, SameDocumentNavigationFacts,
    SameDocumentNavigationParams, SameDocumentNavigationType, StaticDocumentCommit,
    StaticDocumentWindowChoice, StaticDocumentWindowDisposition, StaticDocumentWindowReuseFacts,
    UserNavigationInvolvement,
};
mod import_map;
mod module_resources;
mod resource_fetcher;
mod resource_loader;
mod text_decode;
mod url_reference;

pub use document_loader::{
    DocumentLoadBudget, DocumentLoadProgress, DocumentLoader, DocumentLoaderStatus,
    InitialScrollState,
};
pub use module_resources::{ModuleLoadError, ModuleResources};
pub use resource_fetcher::{
    FontFaceSourceInput, ImageResourceRequestFacts, ImageResourceResponseFacts, ResourceFetcher,
    ResourceFetcherClient, ResponsiveImageInput, ResponsiveImageSourceInput,
    StyleSheetImportFailure, StyleSheetImportLoad, StyleSheetImportPoll,
};
pub use resource_loader::{
    AwaitResource, LoadResponse, RequireResponse, ResourceLoadResult, ResourceLoadStatus,
    ResourceLoader, StartResource,
};
pub use text_decode::DecodeText;
pub use url_reference::{ResolveCSSStyleSheetURLs, ResolveCSSURLs, ResolveURLUTF16, ResolveUrl};

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

pub use resource_fetcher::RawResourceFetchOptions;

pub use fetch_policy::{
    DeriveRawLocalNetworkClientSecurityState, RawFetchAddressSpace, RawFetchAddressSpaceOverrides,
    RawFetchContentSettingsClient, RawFetchFrameSecurity, RawFetchFrameSettings,
    RawFetchLocalNetworkAccessPolicy, RawFetchSecurityContext, RawKeepaliveRedirectFrame,
    RawLocalNetworkAccessEffect, RawLocalNetworkClientSecurityState,
    RawLocalNetworkDocumentPermissions, RawLocalNetworkPermission,
    RawLocalNetworkPermissionContinuation, RawLocalNetworkPermissionController,
    RawLocalNetworkPermissionPrompt, RawLocalNetworkPermissionResult,
    RawLocalNetworkPermissionStatus, RawLocalNetworkRequestPolicy, RawMixedContentEffect,
    RawMixedContentRequest,
};

pub use resource_fetcher::ResourceCacheValidationSuppressor;

pub mod font_resource;

pub mod document_load_timing;
pub use document_load_timing::DocumentLoadTiming;
