//! Minimal typed fetch boundary for the retained Document CSP owner.
//! Chromium content_security_policy.{h,cc}: AllowRequest/AllowFromSource;
//! request contexts are the RawResource keepalive consumers of connect-src.
use std::sync::Arc;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSPRequestContext {
    Ping,
    Beacon,
    CSPReport,
    Image,
    ImageSet,
    Font,
    Track,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CSPWorld {
    Main,
    Isolated(u64),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSPRedirectStatus {
    NoRedirect,
    FollowedRedirect,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSPReportingDisposition {
    Report,
    SuppressReporting,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSPCheckHeaderType {
    All,
    Enforce,
    ReportOnly,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSPHeaderType {
    Enforce,
    ReportOnly,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSPDirective {
    ConnectSrc,
    MediaSrc,
    FormAction,
    BaseURI,
    ImgSrc,
    DefaultSrc,
    ScriptSrc,
    ScriptSrcElem,
    FontSrc,
    TrustedTypes,
    RequireTrustedTypesFor,
}
impl CSPDirective {
    pub fn Name(self) -> &'static str {
        match self {
            Self::ConnectSrc => "connect-src",
            Self::MediaSrc => "media-src",
            Self::FormAction => "form-action",
            Self::BaseURI => "base-uri",
            Self::ImgSrc => "img-src",
            Self::DefaultSrc => "default-src",
            Self::ScriptSrc => "script-src",
            Self::ScriptSrcElem => "script-src-elem",
            Self::FontSrc => "font-src",
            Self::TrustedTypes => "trusted-types",
            Self::RequireTrustedTypesFor => "require-trusted-types-for",
        }
    }
}
pub struct CSPRequest<'a> {
    pub url: &'a str,
    pub url_before_redirects: &'a str,
    pub context: CSPRequestContext,
    pub world: CSPWorld,
    pub redirect: CSPRedirectStatus,
    pub reporting: CSPReportingDisposition,
    pub check_header_type: CSPCheckHeaderType,
}
/// AllowFromSource inputs shared by explicit fetch directives. Media is not a
/// RawResource keepalive request context; it retains its own source directive.
pub struct CSPSourceRequest<'a> {
    pub url: &'a str,
    pub url_before_redirects: &'a str,
    pub world: CSPWorld,
    pub redirect: CSPRedirectStatus,
    pub reporting: CSPReportingDisposition,
    pub check_header_type: CSPCheckHeaderType,
}
impl<'a> From<&CSPRequest<'a>> for CSPSourceRequest<'a> {
    fn from(request: &CSPRequest<'a>) -> Self {
        Self {
            url: request.url,
            url_before_redirects: request.url_before_redirects,
            world: request.world,
            redirect: request.redirect,
            reporting: request.reporting,
            check_header_type: request.check_header_type,
        }
    }
}
/// One source ReportViolation effect. The embedding routes console, violation
/// event and reporting delivery; the policy never calls DOM or a network client.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSPConsoleLevel {
    Info,
    Error,
}
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum CSPSourceLocation {
    #[default]
    Unknown,
    ResponseHeader {
        url: String,
    },
    Script {
        url: String,
        line: u32,
        column: u32,
    },
    /// Explicit WebIDL initializer values, not an inferred runtime stack.
    WebIDL {
        url: String,
        line: i32,
        column: i32,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CSPViolation {
    pub disposition: CSPHeaderType,
    pub effective_directive: CSPDirective,
    pub violated_directive: CSPDirective,
    pub directive_text: String,
    pub original_policy: String,
    pub document_url: String,
    pub report_url: String,
    pub blocked_url: String,
    pub url_before_redirects: String,
    pub report_endpoints: Vec<String>,
    pub use_reporting_api: bool,
    pub console_message: String,
    pub console_level: CSPConsoleLevel,
    pub source_location: CSPSourceLocation,
    pub status_code: Option<u16>,
    pub referrer: Option<String>,
    pub world: CSPWorld,
    pub blocked_reason: Option<CSPBlockedReason>,
    /// Source truncation counts UTF-16 code units, including a split surrogate.
    pub sample: Vec<u16>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CSPBlockedReason {
    ContentSecurityPolicy {
        policy_index: usize,
        directive: CSPDirective,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CSPRequestVerdict {
    pub allowed: bool,
    pub blocked_reason: Option<CSPBlockedReason>,
    pub violations: Vec<CSPViolation>,
    /// Source metric, not another blocking decision. CSP permits wildcard WS
    /// matches in this revision while counting the narrower future behavior.
    pub wildcard_matches_websocket: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CSPProtectedOrigin {
    Tuple {
        scheme: String,
        host: String,
        port: u16,
        potentially_trustworthy: bool,
    },
    Opaque {
        potentially_trustworthy: bool,
        precursor_scheme: Option<String>,
    },
}
impl Default for CSPProtectedOrigin {
    fn default() -> Self {
        Self::Opaque {
            potentially_trustworthy: false,
            precursor_scheme: None,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CSPRequestPreparation {
    pub url: String,
    /// ResourceRequest::UpgradeIfInsecure survives an initially HTTPS URL.
    pub upgrade_if_insecure: bool,
    /// This preparation actually upgraded HTTP; not IsAutomaticUpgrade, which
    /// belongs only to MixedContentChecker's default image-upgrade branch.
    pub did_upgrade_insecure_request: bool,
    pub block_all_mixed_content: bool,
    pub protected_origin: CSPProtectedOrigin,
}
pub trait CSPRequestPolicy: Send + Sync {
    /// PopulateResourceRequest's source upgrade stage precedes AllowRequest.
    fn PrepareRequestURL(&self, url: &str) -> CSPRequestPreparation;
    /// MixedContentChecker uses the source registry snapshot, never a live DOM
    /// callback or a hardcoded HTTP-only trust approximation.
    fn IsURLPotentiallyTrustworthy(&self, url: &str) -> bool;

    fn AllowRequest(&self, request: &CSPRequest<'_>) -> CSPRequestVerdict;
    fn AllowMedia(&self, request: &CSPSourceRequest<'_>) -> CSPRequestVerdict;
    fn AllowFormAction(&self, request: &CSPSourceRequest<'_>) -> CSPRequestVerdict;
    fn AllowBaseURI(&self, request: &CSPSourceRequest<'_>) -> CSPRequestVerdict;
    /// InlineType::Navigation hashes the entire decoded javascript: URL. It
    /// has no nonce or script element and uses the original calling world.
    fn AllowInlineNavigation(
        &self,
        decoded_url: &str,
        world: CSPWorld,
        source_location: CSPSourceLocation,
    ) -> CSPRequestVerdict;
    fn RequiresTrustedTypesForScript(&self, world: CSPWorld) -> bool;
    /// Reporting decisions/dedup stay on this original policy owner; the Page
    /// executes the resulting typed effects in their source order.
    fn PrepareViolationDelivery(&self, violation: CSPViolation) -> Vec<CSPDeliveryEffect>;
    /// Retain the original PolicyContainer host for keepalive redirects. Its
    /// compiled ASTs are immutable/shared; subsequent policy additions remain
    /// visible through the same host. No DOM callback or policy reparse.
    fn Snapshot(&self) -> Arc<dyn CSPRequestPolicy>;
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CSPParsingError {
    pub original_policy: String,
    pub message: String,
    pub source_location: CSPSourceLocation,
}

#[derive(Clone, Debug)]
pub struct CSPViolationEventData {
    /// Original Document event target, preserved while its parser owns the arena.
    /// Absent only on an explicit script-created event initializer.
    pub document_node_id: Option<u64>,
    pub document_uri: String,
    pub referrer: Option<String>,
    pub blocked_uri: String,
    pub violated_directive: String,
    pub effective_directive: String,
    pub original_policy: String,
    pub disposition: CSPHeaderType,
    pub source_location: CSPSourceLocation,
    pub status_code: Option<u16>,
    pub sample: Vec<u16>,
    pub eval_hash: String,
    pub url_hash: String,
    /// Retained native diagnostic; this is not an invented WebIDL property.
    pub blocked_reason: Option<CSPBlockedReason>,
}
impl CSPViolationEventData {
    pub fn sourceFile(&self) -> &str {
        match &self.source_location {
            CSPSourceLocation::Script { url, line, .. } if *line > 0 => url,
            CSPSourceLocation::WebIDL { url, .. } => url,
            _ => "",
        }
    }
    pub fn lineNumber(&self) -> i32 {
        match &self.source_location {
            CSPSourceLocation::Script { line, .. } => *line as i32,
            CSPSourceLocation::WebIDL { line, .. } => *line,
            _ => 0,
        }
    }
    pub fn columnNumber(&self) -> i32 {
        match &self.source_location {
            CSPSourceLocation::Script { line, column, .. } if *line > 0 => *column as i32,
            CSPSourceLocation::WebIDL { column, .. } => *column,
            _ => 0,
        }
    }
    pub fn statusCode(&self) -> u16 {
        self.status_code.unwrap_or(0)
    }
    pub fn disposition(&self) -> &'static str {
        match self.disposition {
            CSPHeaderType::Enforce => "enforce",
            CSPHeaderType::ReportOnly => "report",
        }
    }
    /// PostViolationReport's exact JSON field order is retained for the same
    /// policy's duplicate-report hash. Unknown source stays absent, not a URL
    /// inferred from the document and a fabricated line number.
    pub fn LegacyReportJSON(&self) -> String {
        let mut fields = vec![
            ("document-uri", quote(&self.document_uri)),
            ("referrer", quote(self.referrer.as_deref().unwrap_or(""))),
            ("violated-directive", quote(&self.violated_directive)),
            ("effective-directive", quote(&self.effective_directive)),
            ("original-policy", quote(&self.original_policy)),
            ("disposition", quote(self.disposition())),
            ("blocked-uri", quote(&self.blocked_uri)),
        ];
        if self.lineNumber() != 0 {
            fields.push(("line-number", self.lineNumber().to_string()));
        }
        if self.columnNumber() != 0 {
            fields.push(("column-number", self.columnNumber().to_string()));
        }
        if !self.sourceFile().is_empty() {
            fields.push(("source-file", quote(self.sourceFile())));
        }
        fields.push(("status-code", self.statusCode().to_string()));
        fields.push(("script-sample", quote_utf16(&self.sample)));
        if !self.eval_hash.is_empty() {
            fields.push(("eval-hash", quote(&self.eval_hash)));
        }
        if !self.url_hash.is_empty() {
            fields.push(("url-hash", quote(&self.url_hash)));
        }
        format!(
            "{{\"csp-report\":{{{}}}}}",
            fields
                .into_iter()
                .map(|(key, value)| format!("{}:{}", quote(key), value))
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}
fn quote(value: &str) -> String {
    let mut out = String::from("\"");
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if c < ' ' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn quote_utf16(value: &[u16]) -> String {
    let mut out = String::from("\"");
    for decoded in char::decode_utf16(value.iter().copied()) {
        match decoded {
            Ok(c) => {
                let quoted = quote(c.encode_utf8(&mut [0; 4]));
                out.push_str(&quoted[1..quoted.len() - 1]);
            }
            Err(surrogate) => out.push_str(&format!("\\u{:04x}", surrogate.unpaired_surrogate())),
        }
    }
    out.push('"');
    out
}

#[derive(Clone, Debug)]
pub enum CSPDeliveryEffect {
    Console {
        level: CSPConsoleLevel,
        message: String,
        location: CSPSourceLocation,
    },
    QueueObservedReport {
        url: String,
        data: CSPViolationEventData,
        endpoint_groups: Vec<String>,
    },
    SendLegacyReport {
        url: String,
        body: String,
        world: CSPWorld,
    },
    QueueViolationEvent {
        data: CSPViolationEventData,
    },
}
