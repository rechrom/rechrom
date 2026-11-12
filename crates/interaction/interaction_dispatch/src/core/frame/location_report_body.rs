//! Chromium `third_party/blink/renderer/core/frame/location_report_body.h` and
//! `location_report_body.cc`: retained source location and extension filtering.

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReportLocation {
    pub source_file: Option<String>,
    pub line_number: Option<u32>,
    pub column_number: Option<u32>,
}
impl ReportLocation {
    /// location_report_body.cc:55-66: parse the captured source URL once at this
    /// query, then consult the original product scheme registry.
    pub fn IsExtensionSource(&self) -> bool {
        self.source_file.as_deref().is_some_and(IsExtensionSource)
    }
}

// Shared by payloads that inherit LocationReportBody in Chromium. Borrow the
// existing source string rather than constructing a second location owner.
pub(super) fn IsExtensionSource(source_file: &str) -> bool {
    url::Url::parse(source_file).is_ok_and(|url| {
        foundation::common_scheme_registry::CommonSchemeRegistry::IsExtensionScheme(url.scheme())
    })
}
