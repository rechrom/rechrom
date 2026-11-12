pub mod http_request;
pub mod http_transport_options;
mod local_network_access;
mod proxy_connector;
pub use foundation::network_address_space::IPAddressSpace;
mod referrer_policy;
mod request_trace;
mod static_get;
pub use referrer_policy::{IsSameOrigin, ReferrerPolicy};

pub use http_request::{
    HTTPRequestData, HTTPRequestOperation, HTTPRequestPriority, HTTPRequestTransport,
    HTTPResponseData, StartHTTPRequest,
};
pub use http_transport_options::HTTPTransportOptions;
pub use static_get::{Get, GetResponse, Response};

mod http_stream;
pub use http_stream::{HTTPStreamEvent, HTTPStreamOperation, HTTPWakeCallback};

mod fetch_request;
pub use fetch_request::{
    FetchCredentialsMode, FetchKeepaliveMetrics, FetchKeepaliveMetricsSnapshot,
    FetchKeepaliveRequestState, FetchMode, FetchOrigin, FetchRedirectMode, NetworkFetchOptions,
};
