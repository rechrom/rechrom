//! URL-based byte loading for HTTP(S), file:, data: and registered memory: URLs.
//! Document parsing, resource scheduling and render blocking belong upstream.
mod data;
mod default_url_loader;
mod file;
mod http;
mod memory;
mod metadata;
mod ready;
mod url_loader;

pub use default_url_loader::{DefaultURLLoader, DefaultURLLoaderOptions};
pub use http_transport::{IsSameOrigin, ReferrerPolicy};
pub use url_loader::{
    RequestDefer, RequestDestination, ResourceLoadPriority, ScriptType, URLLoadOperation,
    URLLoader, URLRequest, URLResponse,
};

mod stream;
pub use stream::{URLLoadEvent, URLResponseHead, URLStreamOperation, URLWakeCallback};

mod file_stream;

pub use http_transport::{
    FetchCredentialsMode, FetchKeepaliveMetrics, FetchKeepaliveMetricsSnapshot,
    FetchKeepaliveRequestState, FetchMode, FetchOrigin, FetchRedirectMode, NetworkFetchOptions,
};

pub use http_transport::IPAddressSpace;
