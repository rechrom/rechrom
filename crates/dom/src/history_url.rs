//! Chromium frame/history_util.cc:15-102. Query on the canonical Document URL
//! and SecurityOrigin, including registered standard schemes and opaque origin.
use crate::persistent_document::PersistentDocument;
use url::Url;
impl PersistentDocument {
    pub fn CanChangeToURLForHistoryApi(&self, target: &str) -> bool {
        let (Ok(url), Ok(document_url)) = (Url::parse(target), Url::parse(self.DocumentURL()))
        else {
            return false;
        };
        if url.scheme() != document_url.scheme()
            || url.username() != document_url.username()
            || url.password().unwrap_or_default() != document_url.password().unwrap_or_default()
            || url.host_str() != document_url.host_str()
            || url.port() != document_url.port()
        {
            return false;
        }
        if matches!(url.scheme(), "http" | "https") {
            return true;
        }
        let differ_in_path = url.path() != document_url.path();
        let differ_in_query =
            url.query().unwrap_or_default() != document_url.query().unwrap_or_default();
        if url.scheme() == "file" && differ_in_path {
            return false;
        }
        let origin = self.SecurityContext().Origin();
        if origin.IsOpaque() {
            return !differ_in_path;
        }
        if origin.IsStandardURLScheme(url.scheme()) {
            return true;
        }
        !differ_in_path && !differ_in_query
    }
}
