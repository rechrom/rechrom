//! Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! Retained Document CSP, fetch/form/base consumers and JavaScript URL checks.
//! Source: content_security_policy.cc:102-124,827-893,949-1065;
//! Inline: content_security_policy.cc:607-686; csp_directive_list.cc:78-125,
//! 304-325,484-568,773-816,1229-1242; source_list_directive.cc:146-179.
//! csp_directive_list.cc:692-738; csp_source.cc:20-227;
//! source_list_directive.cc:47-87; network CSP parser cc:398-630,747-797,
//! 859-1006,1028-1221,1384-1401,1427-1507.
use crate::persistent_document::PersistentDocument;
use base64::{
    alphabet,
    engine::DecodePaddingMode,
    engine::{GeneralPurpose, GeneralPurposeConfig},
    Engine as _,
};
use foundation::content_security_policy::*;
use sha2::{Digest, Sha256, Sha384, Sha512};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    sync::{Arc, Mutex, RwLock},
};
use url::Url;

#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Source {
    scheme: String,
    host: String,
    port: Option<i32>,
    path: String,
    host_wildcard: bool,
    port_wildcard: bool,
}
#[derive(Default)]
struct SourceList {
    allow_self: bool,
    allow_star: bool,
    sources: Vec<Source>,
    allow_inline: bool,
    allow_dynamic: bool,
    allow_unsafe_hashes: bool,
    report_sample: bool,
    nonces: Vec<String>,
    hashes: Vec<(HashAlgorithm, Vec<u8>)>,
}
#[derive(Clone, Copy)]
enum HashAlgorithm {
    Sha256,
    Sha384,
    Sha512,
    Ed25519,
}
#[path = "content_security_policy_trusted_types.rs"]
mod trusted_types;
pub use trusted_types::TrustedTypePolicyDetails;

struct Policy {
    original: String,
    disposition: CSPHeaderType,
    self_source: Option<Source>,
    default_src: Option<(String, SourceList)>,
    connect_src: Option<(String, SourceList)>,
    media_src: Option<(String, SourceList)>,
    img_src: Option<(String, SourceList)>,
    font_src: Option<(String, SourceList)>,
    form_action: Option<(String, SourceList)>,
    base_uri: Option<(String, SourceList)>,
    script_src: Option<(String, SourceList)>,
    script_src_elem: Option<(String, SourceList)>,
    require_trusted_types_for_script: bool,
    trusted_types: Option<trusted_types::TrustedTypesDirective>,
    endpoints: Vec<String>,
    reporting_api: bool,
    upgrade_insecure_requests: bool,
    block_all_mixed_content: bool,
    sandbox: foundation::security::SandboxFlags,
}
#[derive(Clone, Default)]
struct CompiledPolicies {
    document_url: Arc<str>,
    document_node_id: Option<u64>,
    main: Arc<Vec<Arc<Policy>>>,
    isolated: Arc<BTreeMap<u64, Arc<Vec<Arc<Policy>>>>>,
    bypass_schemes: Arc<HashSet<String>>,
    trust_registry: crate::persistent_document::SecurityOriginSchemeRegistry,
    protected_origin: CSPProtectedOrigin,
    status_code: Option<u16>,
    referrer: Option<String>,
    report_hashes: Arc<Mutex<HashMap<CSPWorld, HashSet<u64>>>>,
}
/// This owner is shared by Document and ResourceFetcher. Mutations compile
/// once; a fetch snapshot shares these exact immutable policy ASTs with the
/// browser's keepalive redirect checker after renderer/document teardown.
#[derive(Default)]
pub struct DocumentContentSecurityPolicy {
    compiled: Arc<RwLock<CompiledPolicies>>,
    parsing_errors: Mutex<Vec<CSPParsingError>>,
    local_violations: Mutex<Vec<CSPViolation>>,
}
#[derive(Clone)]
struct PolicySnapshot(CompiledPolicies);
#[derive(Clone)]
struct PolicyContainerView(Arc<RwLock<CompiledPolicies>>);
impl PolicyContainerView {
    fn Current(&self) -> PolicySnapshot {
        PolicySnapshot(self.0.read().expect("retained policy container").clone())
    }
}
impl CSPRequestPolicy for PolicyContainerView {
    fn PrepareViolationDelivery(&self, violation: CSPViolation) -> Vec<CSPDeliveryEffect> {
        self.Current().PrepareViolationDelivery(violation)
    }

    fn PrepareRequestURL(&self, url: &str) -> CSPRequestPreparation {
        self.Current().PrepareRequestURL(url)
    }
    fn IsURLPotentiallyTrustworthy(&self, url: &str) -> bool {
        self.Current().IsURLPotentiallyTrustworthy(url)
    }
    fn AllowRequest(&self, request: &CSPRequest<'_>) -> CSPRequestVerdict {
        self.Current().AllowRequest(request)
    }
    fn AllowMedia(&self, request: &CSPSourceRequest<'_>) -> CSPRequestVerdict {
        self.Current().AllowMedia(request)
    }
    fn AllowFormAction(&self, request: &CSPSourceRequest<'_>) -> CSPRequestVerdict {
        self.Current().AllowFormAction(request)
    }
    fn AllowBaseURI(&self, request: &CSPSourceRequest<'_>) -> CSPRequestVerdict {
        self.Current().AllowBaseURI(request)
    }
    fn AllowInlineNavigation(
        &self,
        decoded_url: &str,
        world: CSPWorld,
        source_location: CSPSourceLocation,
    ) -> CSPRequestVerdict {
        self.Current()
            .AllowInlineNavigation(decoded_url, world, source_location)
    }
    fn RequiresTrustedTypesForScript(&self, world: CSPWorld) -> bool {
        self.Current().RequiresTrustedTypesForScript(world)
    }
    fn Snapshot(&self) -> Arc<dyn CSPRequestPolicy> {
        Arc::new(self.clone())
    }
}

impl PersistentDocument {
    pub fn ContentSecurityPolicy(&self) -> Arc<DocumentContentSecurityPolicy> {
        self.content_security_policy
            .BindDocumentNodeId(self.RootHandle().Id());
        self.content_security_policy.clone()
    }
    pub fn AddResponseContentSecurityPolicies(
        &self,
        base_url: &str,
        enforce: &[String],
        report_only: &[String],
    ) {
        self.content_security_policy
            .BindDocumentNodeId(self.RootHandle().Id());
        self.content_security_policy.AddResponsePolicies(
            base_url,
            enforce,
            report_only,
            self.SecurityContext().Origin(),
        );
    }
}
impl DocumentContentSecurityPolicy {
    /// Main Document's own CSP; an isolated world's separate policy does not
    /// make the main policy active. ContentSecurityPolicy::IsActive.
    pub fn BindDocumentNodeId(&self, id: u64) {
        self.compiled
            .write()
            .expect("Document CSP owner")
            .document_node_id = Some(id);
    }
    pub fn IsActive(&self) -> bool {
        !self
            .compiled
            .read()
            .expect("Document CSP owner")
            .main
            .is_empty()
    }
    pub fn BindReportContext(&self, status_code: Option<u16>, referrer: Option<String>) {
        let mut state = self.compiled.write().expect("Document CSP owner");
        state.status_code = status_code;
        state.referrer = referrer;
    }
    pub fn BindSecurityOrigin(&self, origin: &crate::persistent_document::SecurityOrigin) {
        let mut state = self.compiled.write().expect("Document CSP owner");
        state.trust_registry = origin.SchemeRegistryForRequests();
        state.protected_origin = protected_origin(origin);
    }
    pub fn UpdateDocumentURL(&self, url: &str) {
        self.compiled
            .write()
            .expect("Document CSP owner")
            .document_url = Arc::from(url);
    }
    /// Source response headers are independent policies whose decisions are
    /// intersected. No policy is discarded merely because it reports only.
    pub fn AddResponsePolicies(
        &self,
        base_url: &str,
        enforce: &[String],
        report_only: &[String],
        origin: &crate::persistent_document::SecurityOrigin,
    ) {
        let mut errors = Vec::new();
        let mut parsed = Vec::new();
        for (headers, kind) in [
            (enforce, CSPHeaderType::Enforce),
            (report_only, CSPHeaderType::ReportOnly),
        ] {
            for header in headers {
                for value in header.split(',').map(trim_csp).filter(|v| !v.is_empty()) {
                    parsed.push(Arc::new(parse_policy(
                        value,
                        kind,
                        base_url,
                        origin,
                        &mut errors,
                    )));
                }
            }
        }
        for error in &mut errors {
            error.source_location = CSPSourceLocation::ResponseHeader {
                url: base_url.to_owned(),
            };
        }
        let mut compiled = self.compiled.write().expect("Document CSP owner");
        compiled.document_url = Arc::from(base_url);
        compiled.trust_registry = origin.SchemeRegistryForRequests();
        compiled.protected_origin = protected_origin(origin);
        Arc::make_mut(&mut compiled.main).extend(parsed);
        self.parsing_errors
            .lock()
            .expect("CSP parse effects")
            .extend(errors);
    }
    /// LocalDOMWindow::GetContentSecurityPolicyForWorld (cc:412-432). Worlds
    /// without an installed isolated policy use the main world's same owner.
    pub fn SetIsolatedWorldPolicies(
        &self,
        world: u64,
        base_url: &str,
        enforce: &[String],
        report_only: &[String],
        origin: &crate::persistent_document::SecurityOrigin,
    ) {
        let mut parsed = Vec::new();
        let mut errors = Vec::new();
        for (headers, kind) in [
            (enforce, CSPHeaderType::Enforce),
            (report_only, CSPHeaderType::ReportOnly),
        ] {
            for header in headers {
                for value in header.split(',').map(trim_csp).filter(|v| !v.is_empty()) {
                    parsed.push(Arc::new(parse_policy(
                        value,
                        kind,
                        base_url,
                        origin,
                        &mut errors,
                    )));
                }
            }
        }
        Arc::make_mut(&mut self.compiled.write().expect("Document CSP owner").isolated)
            .insert(world, Arc::new(parsed));
        self.parsing_errors
            .lock()
            .expect("CSP parse effects")
            .extend(errors);
    }
    pub fn RemoveIsolatedWorldPolicy(&self, world: u64) {
        Arc::make_mut(&mut self.compiled.write().expect("Document CSP owner").isolated)
            .remove(&world);
    }
    /// Source SchemeRegistry's explicitly registered policy-area-all schemes.
    /// The current product starts with none; it does not infer privileged
    /// schemes from URL strings or a document's origin.
    pub fn RegisterBypassingScheme(&self, scheme: &str) {
        assert!(valid_scheme(scheme));
        Arc::make_mut(
            &mut self
                .compiled
                .write()
                .expect("Document CSP owner")
                .bypass_schemes,
        )
        .insert(scheme.to_ascii_lowercase());
    }
    /// Renderer-side DOM policy consumers enqueue their original violations
    /// while borrowing Document. The Page drains after that borrow is released.
    pub fn QueueLocalViolations(&self, violations: Vec<CSPViolation>) {
        self.local_violations
            .lock()
            .expect("Document CSP violations")
            .extend(violations);
    }
    pub fn TakeLocalViolationDeliveryEffects(&self) -> Vec<CSPDeliveryEffect> {
        let violations = std::mem::take(
            &mut *self
                .local_violations
                .lock()
                .expect("Document CSP violations"),
        );
        violations
            .into_iter()
            .flat_map(|violation| self.PrepareViolationDelivery(violation))
            .collect()
    }
    pub fn TakeParsingErrors(&self) -> Vec<CSPParsingError> {
        std::mem::take(&mut *self.parsing_errors.lock().expect("CSP parse effects"))
    }
}
impl CSPRequestPolicy for DocumentContentSecurityPolicy {
    fn PrepareViolationDelivery(&self, violation: CSPViolation) -> Vec<CSPDeliveryEffect> {
        self.Snapshot().PrepareViolationDelivery(violation)
    }

    fn PrepareRequestURL(&self, url: &str) -> CSPRequestPreparation {
        self.Snapshot().PrepareRequestURL(url)
    }
    fn IsURLPotentiallyTrustworthy(&self, url: &str) -> bool {
        self.compiled
            .read()
            .expect("Document CSP owner")
            .trust_registry
            .IsURLPotentiallyTrustworthy(url)
    }
    fn AllowRequest(&self, request: &CSPRequest<'_>) -> CSPRequestVerdict {
        self.Snapshot().AllowRequest(request)
    }
    fn AllowMedia(&self, request: &CSPSourceRequest<'_>) -> CSPRequestVerdict {
        self.Snapshot().AllowMedia(request)
    }
    fn AllowFormAction(&self, request: &CSPSourceRequest<'_>) -> CSPRequestVerdict {
        self.Snapshot().AllowFormAction(request)
    }
    fn AllowBaseURI(&self, request: &CSPSourceRequest<'_>) -> CSPRequestVerdict {
        self.Snapshot().AllowBaseURI(request)
    }
    fn AllowInlineNavigation(
        &self,
        decoded_url: &str,
        world: CSPWorld,
        source_location: CSPSourceLocation,
    ) -> CSPRequestVerdict {
        self.Snapshot()
            .AllowInlineNavigation(decoded_url, world, source_location)
    }
    fn RequiresTrustedTypesForScript(&self, world: CSPWorld) -> bool {
        self.Snapshot().RequiresTrustedTypesForScript(world)
    }
    fn Snapshot(&self) -> Arc<dyn CSPRequestPolicy> {
        // KeepAliveURLLoaderService.cc:284-288 retains the SAME policy host;
        // keep_alive_url_loader.cc:1201 reads its current policies on redirect.
        Arc::new(PolicyContainerView(self.compiled.clone()))
    }
}
impl CSPRequestPolicy for PolicySnapshot {
    fn PrepareViolationDelivery(&self, violation: CSPViolation) -> Vec<CSPDeliveryEffect> {
        prepare_delivery(&self.0, violation)
    }

    fn PrepareRequestURL(&self, url: &str) -> CSPRequestPreparation {
        let mut prepared = CSPRequestPreparation {
            url: url.to_owned(),
            upgrade_if_insecure: self.0.main.iter().any(|p| p.upgrade_insecure_requests),
            did_upgrade_insecure_request: false,
            block_all_mixed_content: self
                .0
                .main
                .iter()
                .any(|p| p.disposition == CSPHeaderType::Enforce && p.block_all_mixed_content),
            protected_origin: self.0.protected_origin.clone(),
        };
        if prepared.upgrade_if_insecure {
            if let Ok(mut parsed) = Url::parse(url) {
                // network CSP cc:1579-1596. Localhost/loopback and registered
                // trustworthy origins stay HTTP; only source HTTP is upgraded.
                if parsed.scheme() == "http" && !self.IsURLPotentiallyTrustworthy(url) {
                    parsed
                        .set_scheme("https")
                        .expect("HTTP to HTTPS scheme replacement");
                    prepared.url = parsed.to_string();
                    prepared.did_upgrade_insecure_request = true;
                }
            }
        }
        prepared
    }
    fn IsURLPotentiallyTrustworthy(&self, url: &str) -> bool {
        self.0.trust_registry.IsURLPotentiallyTrustworthy(url)
    }
    fn Snapshot(&self) -> Arc<dyn CSPRequestPolicy> {
        Arc::new(self.clone())
    }
    fn AllowRequest(&self, request: &CSPRequest<'_>) -> CSPRequestVerdict {
        if request.context == CSPRequestContext::CSPReport {
            return allowed_verdict();
        }
        self.AllowFromSource(
            &CSPSourceRequest::from(request),
            if matches!(
                request.context,
                CSPRequestContext::Image | CSPRequestContext::ImageSet
            ) {
                CSPDirective::ImgSrc
            } else if request.context == CSPRequestContext::Font {
                CSPDirective::FontSrc
            } else if request.context == CSPRequestContext::Track {
                CSPDirective::MediaSrc
            } else {
                CSPDirective::ConnectSrc
            },
        )
    }
    fn AllowMedia(&self, request: &CSPSourceRequest<'_>) -> CSPRequestVerdict {
        self.AllowFromSource(request, CSPDirective::MediaSrc)
    }
    fn AllowFormAction(&self, request: &CSPSourceRequest<'_>) -> CSPRequestVerdict {
        self.AllowFromSource(request, CSPDirective::FormAction)
    }
    fn AllowBaseURI(&self, request: &CSPSourceRequest<'_>) -> CSPRequestVerdict {
        self.AllowFromSource(request, CSPDirective::BaseURI)
    }
    fn AllowInlineNavigation(
        &self,
        decoded_url: &str,
        world: CSPWorld,
        source_location: CSPSourceLocation,
    ) -> CSPRequestVerdict {
        self.CheckInlineNavigation(decoded_url, world, source_location)
    }
    fn RequiresTrustedTypesForScript(&self, world: CSPWorld) -> bool {
        self.Policies(world)
            .iter()
            .any(|policy| policy.require_trusted_types_for_script)
    }
}
fn allowed_verdict() -> CSPRequestVerdict {
    CSPRequestVerdict {
        allowed: true,
        blocked_reason: None,
        violations: Vec::new(),
        wildcard_matches_websocket: false,
    }
}
impl PolicySnapshot {
    fn Policies(&self, world: CSPWorld) -> &[Arc<Policy>] {
        match world {
            CSPWorld::Main => &self.0.main,
            CSPWorld::Isolated(world) => self.0.isolated.get(&world).unwrap_or(&self.0.main),
        }
    }
    /// content_security_policy.cc607-686; csp_directive_list.cc78-125,
    /// 304-325,484-568,773-816,1229-1242. Navigation has no nonce or
    /// HTMLScriptElement; unsafe-hashes gates its full decoded URL hash.
    fn CheckInlineNavigation(
        &self,
        content: &str,
        world: CSPWorld,
        location: CSPSourceLocation,
    ) -> CSPRequestVerdict {
        let mut result = allowed_verdict();
        let (mut sha256, mut sha384, mut sha512) = (None, None, None);
        for (index, policy) in self.Policies(world).iter().enumerate() {
            let operative = policy
                .script_src_elem
                .as_ref()
                .map(|v| (CSPDirective::ScriptSrcElem, v))
                .or_else(|| {
                    policy
                        .script_src
                        .as_ref()
                        .map(|v| (CSPDirective::ScriptSrc, v))
                })
                .or_else(|| {
                    policy
                        .default_src
                        .as_ref()
                        .map(|v| (CSPDirective::DefaultSrc, v))
                });
            let Some((directive, (raw, list))) = operative else {
                continue;
            };
            let has_hash_or_nonce = !list.nonces.is_empty() || !list.hashes.is_empty();
            if list.allow_inline && !has_hash_or_nonce && !list.allow_dynamic {
                continue;
            }
            if list.allow_unsafe_hashes
                && list.hashes.iter().any(|(algorithm, expected)| {
                    let actual = match algorithm {
                        HashAlgorithm::Sha256 => sha256
                            .get_or_insert_with(|| Sha256::digest(content.as_bytes()).to_vec()),
                        HashAlgorithm::Sha384 => sha384
                            .get_or_insert_with(|| Sha384::digest(content.as_bytes()).to_vec()),
                        HashAlgorithm::Sha512 => sha512
                            .get_or_insert_with(|| Sha512::digest(content.as_bytes()).to_vec()),
                        HashAlgorithm::Ed25519 => return false,
                    };
                    actual.as_slice() == expected.as_slice()
                })
            {
                continue;
            }
            let reason = (policy.disposition == CSPHeaderType::Enforce).then_some(
                CSPBlockedReason::ContentSecurityPolicy {
                    policy_index: index,
                    directive,
                },
            );
            if reason.is_some() {
                result.allowed = false;
                if result.blocked_reason.is_none() {
                    result.blocked_reason = reason.clone();
                }
            }
            let directive_text = if raw.is_empty() {
                directive.Name().to_owned()
            } else {
                format!("{} {}", directive.Name(), raw)
            };
            let mut message = format!("Running the JavaScript URL violates the following Content Security Policy directive '{}'.", directive_text);
            if list.allow_inline && has_hash_or_nonce {
                message.push_str(" Note that 'unsafe-inline' is ignored if either a hash or nonce value is present in the source list.");
            } else {
                message.push_str(" Either the 'unsafe-inline' keyword, a hash ('sha256-...'), or a nonce ('nonce-...') is required to enable inline execution.");
                if !list.allow_unsafe_hashes {
                    message.push_str(" Note that hashes do not apply to event handlers, style attributes and javascript: navigations unless the 'unsafe-hashes' keyword is present.");
                }
                if directive == CSPDirective::DefaultSrc {
                    message.push_str(" Note also that 'script-src' was not explicitly set, so 'default-src' is used as a fallback.");
                }
            }
            message.push_str(if policy.disposition == CSPHeaderType::ReportOnly {
                " The policy is report-only, so the violation has been logged but no further action has been taken."
            } else { " The action has been blocked." });
            result.violations.push(CSPViolation {
                disposition: policy.disposition,
                effective_directive: CSPDirective::ScriptSrcElem,
                violated_directive: directive,
                directive_text,
                original_policy: policy.original.clone(),
                document_url: strip_url_for_report(&self.0.document_url),
                report_url: self.0.document_url.to_string(),
                blocked_url: "inline".into(),
                url_before_redirects: String::new(),
                report_endpoints: policy.endpoints.clone(),
                use_reporting_api: policy.reporting_api,
                console_message: message,
                console_level: if policy.disposition == CSPHeaderType::ReportOnly {
                    CSPConsoleLevel::Info
                } else {
                    CSPConsoleLevel::Error
                },
                source_location: location.clone(),
                status_code: self.0.status_code,
                referrer: self.0.referrer.clone(),
                world,
                blocked_reason: reason,
                // ReportViolation cc1374+: strip whitespace, truncate 40 WTF
                // UTF-16 units, retaining a final unpaired high surrogate.
                sample: if list.report_sample {
                    content.trim().encode_utf16().take(40).collect()
                } else {
                    Vec::new()
                },
            });
        }
        result
    }
    /// ContentSecurityPolicy::AllowFromSource, cc:949-1065. Operative source
    /// lists and reports stay on this same compiled policy/container owner.
    fn AllowFromSource(
        &self,
        request: &CSPSourceRequest<'_>,
        effective: CSPDirective,
    ) -> CSPRequestVerdict {
        let mut result = allowed_verdict();
        let url = Url::parse(request.url).ok();
        if let Some(url) = url.as_ref() {
            let scheme = if matches!(url.scheme(), "blob" | "filesystem") {
                Url::parse(url.path()).ok().map(|u| u.scheme().to_owned())
            } else {
                Some(url.scheme().to_owned())
            };
            if scheme
                .as_ref()
                .is_some_and(|s| self.0.bypass_schemes.contains(s))
            {
                return result;
            }
        }
        let policies = match request.world {
            CSPWorld::Main => &self.0.main,
            CSPWorld::Isolated(world) => self.0.isolated.get(&world).unwrap_or(&self.0.main),
        };
        for (index, policy) in policies.iter().enumerate() {
            if !check_header(request, policy.disposition) {
                continue;
            }
            let source_list = match effective {
                CSPDirective::ConnectSrc => &policy.connect_src,
                CSPDirective::MediaSrc => &policy.media_src,
                CSPDirective::ImgSrc => &policy.img_src,
                CSPDirective::FontSrc => &policy.font_src,
                CSPDirective::FormAction => &policy.form_action,
                CSPDirective::BaseURI => &policy.base_uri,
                CSPDirective::DefaultSrc
                | CSPDirective::ScriptSrc
                | CSPDirective::ScriptSrcElem
                | CSPDirective::TrustedTypes
                | CSPDirective::RequireTrustedTypesFor => {
                    unreachable!("explicit URL source directive")
                }
            };
            let operative = source_list.as_ref().map(|s| (effective, s)).or_else(|| {
                // form-action is not a fetch directive and has no default-src
                // fallback (CSPDirectiveList::OperativeDirective source).
                if matches!(effective, CSPDirective::FormAction | CSPDirective::BaseURI) {
                    None
                } else {
                    policy
                        .default_src
                        .as_ref()
                        .map(|s| (CSPDirective::DefaultSrc, s))
                }
            });
            let Some((directive, (raw, list))) = operative else {
                continue;
            };
            let (allowed, wildcard_ws) = url.as_ref().map_or((false, false), |url| {
                list_allows(list, policy.self_source.as_ref(), url, request.redirect)
            });
            result.wildcard_matches_websocket |=
                effective == CSPDirective::ConnectSrc && wildcard_ws;
            if allowed {
                continue;
            }
            if policy.disposition == CSPHeaderType::Enforce {
                result.allowed = false;
                if result.blocked_reason.is_none() {
                    result.blocked_reason = Some(CSPBlockedReason::ContentSecurityPolicy {
                        policy_index: index,
                        directive,
                    });
                }
            }
            if request.reporting == CSPReportingDisposition::Report {
                let directive_text = if raw.is_empty() {
                    directive.Name().to_owned()
                } else {
                    format!("{} {}", directive.Name(), raw)
                };
                let prefix = match effective {
                    CSPDirective::ConnectSrc => "Connecting to '",
                    CSPDirective::MediaSrc => "Loading media from  '",
                    CSPDirective::ImgSrc => "Loading the image '",
                    CSPDirective::FontSrc => "Loading the font '",
                    CSPDirective::FormAction => "Sending form data to '",
                    CSPDirective::BaseURI => "Setting the document's base URI to '",
                    CSPDirective::DefaultSrc
                    | CSPDirective::ScriptSrc
                    | CSPDirective::ScriptSrcElem
                    | CSPDirective::TrustedTypes
                    | CSPDirective::RequireTrustedTypesFor => unreachable!(
                        "explicit URL source directive; Trusted Types uses its policy/sink checker"
                    ),
                };
                let mut message = format!(
                    "{prefix}{}' violates the following Content Security Policy directive: \"{}\".",
                    request.url, directive_text
                );
                if directive == CSPDirective::DefaultSrc {
                    message.push_str(&format!(" Note that '{}' was not explicitly set, so 'default-src' is used as a fallback.", effective.Name()));
                }
                if policy.disposition == CSPHeaderType::ReportOnly {
                    message.push_str(" The policy is report-only, so the violation has been logged but no further action has been taken.");
                } else {
                    message.push_str(" The action has been blocked.");
                }
                result.violations.push(CSPViolation {
                    disposition: policy.disposition,
                    effective_directive: effective,
                    violated_directive: directive,
                    directive_text,
                    original_policy: policy.original.clone(),
                    document_url: strip_url_for_report(&self.0.document_url),
                    report_url: self.0.document_url.to_string(),
                    blocked_url: strip_url_for_report(request.url_before_redirects),
                    url_before_redirects: request.url_before_redirects.to_owned(),
                    report_endpoints: policy.endpoints.clone(),
                    use_reporting_api: policy.reporting_api,
                    console_message: message,
                    console_level: if policy.disposition == CSPHeaderType::ReportOnly {
                        CSPConsoleLevel::Info
                    } else {
                        CSPConsoleLevel::Error
                    },
                    source_location: CSPSourceLocation::Unknown,
                    sample: Vec::new(),
                    status_code: self.0.status_code,
                    referrer: self.0.referrer.clone(),
                    world: request.world,
                    blocked_reason: if policy.disposition == CSPHeaderType::Enforce {
                        Some(CSPBlockedReason::ContentSecurityPolicy {
                            policy_index: index,
                            directive,
                        })
                    } else {
                        None
                    },
                });
            }
        }
        result
    }
}
fn check_header(request: &CSPSourceRequest<'_>, kind: CSPHeaderType) -> bool {
    if request.reporting == CSPReportingDisposition::SuppressReporting
        && kind == CSPHeaderType::ReportOnly
    {
        return false;
    }
    match request.check_header_type {
        CSPCheckHeaderType::All => true,
        CSPCheckHeaderType::Enforce => kind == CSPHeaderType::Enforce,
        CSPCheckHeaderType::ReportOnly => kind == CSPHeaderType::ReportOnly,
    }
}
fn trim_csp(value: &str) -> &str {
    value.trim_matches(|c| matches!(c, '\t' | '\n' | '\x0c' | '\r' | ' '))
}
fn words(value: &str) -> impl Iterator<Item = &str> {
    value
        .split(|c| matches!(c, '\t' | '\n' | '\x0c' | '\r' | ' '))
        .filter(|w| !w.is_empty())
}
fn error(errors: &mut Vec<CSPParsingError>, header: &str, message: String) {
    errors.push(CSPParsingError {
        original_policy: header.to_owned(),
        message,
        source_location: CSPSourceLocation::Unknown,
    });
}
fn parse_policy(
    header: &str,
    disposition: CSPHeaderType,
    base_url: &str,
    origin: &crate::persistent_document::SecurityOrigin,
    errors: &mut Vec<CSPParsingError>,
) -> Policy {
    let base = Url::parse(base_url).ok();
    let self_source = base.as_ref().map(|url| Source {
        scheme: url.scheme().into(),
        host: if url.scheme() == "file" {
            String::new()
        } else {
            url.host_str().unwrap_or("").into()
        },
        port: if url.scheme() == "file" {
            None
        } else {
            url.port_or_known_default().map(i32::from)
        },
        ..Source::default()
    });
    let mut policy = Policy {
        original: header.into(),
        disposition,
        self_source,
        default_src: None,
        connect_src: None,
        media_src: None,
        img_src: None,
        font_src: None,
        form_action: None,
        base_uri: None,
        script_src: None,
        script_src_elem: None,
        require_trusted_types_for_script: false,
        trusted_types: None,
        endpoints: Vec::new(),
        reporting_api: false,
        upgrade_insecure_requests: false,
        block_all_mixed_content: false,
        sandbox: foundation::security::SandboxFlags::default(),
    };
    let mut seen = HashSet::new();
    let mut report_uri = None;
    let mut report_to = None;
    for raw in header.split(';').map(trim_csp).filter(|s| !s.is_empty()) {
        let end = raw
            .find(|c| matches!(c, '\t' | '\n' | '\x0c' | '\r' | ' '))
            .unwrap_or(raw.len());
        let name = raw[..end].to_ascii_lowercase();
        let value = trim_csp(&raw[end..]);
        if !name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-') {
            error(
                errors,
                header,
                format!("Invalid Content-Security-Policy directive name '{name}'."),
            );
            continue;
        }
        if !seen.insert(name.clone()) {
            error(
                errors,
                header,
                format!("Ignoring duplicate Content-Security-Policy directive '{name}'."),
            );
            continue;
        }
        if !value.bytes().all(|c| {
            (0x21..=0x7e).contains(&c) || matches!(c, b'\t' | b'\n' | b'\x0c' | b'\r' | b' ')
        }) {
            error(
                errors,
                header,
                format!("Invalid characters in Content-Security-Policy directive '{name}'."),
            );
            continue;
        }
        match name.as_str() {
            "script-src" => {
                policy.script_src = Some((value.into(), parse_list(value, header, errors)))
            }
            "script-src-elem" => {
                policy.script_src_elem = Some((value.into(), parse_list(value, header, errors)))
            }
            "trusted-types" => {
                policy.trusted_types = Some(trusted_types::Parse(value, header, errors));
            }
            "require-trusted-types-for" => {
                for token in words(value) {
                    if token == "'script'" {
                        policy.require_trusted_types_for_script = true;
                    } else {
                        error(errors, header, format!("Invalid expression in 'require-trusted-types-for' Content Security Policy directive: {token}."));
                    }
                }
                if !policy.require_trusted_types_for_script {
                    error(errors, header, "The 'require-trusted-types-for' Content Security Policy directive is empty; it has no effect.".into());
                }
            }
            "connect-src" => {
                policy.connect_src = Some((value.into(), parse_list(value, header, errors)))
            }
            "base-uri" => {
                policy.base_uri = Some((value.into(), parse_list(value, header, errors)));
            }
            "form-action" => {
                policy.form_action = Some((value.into(), parse_list(value, header, errors)))
            }
            "media-src" => {
                policy.media_src = Some((value.into(), parse_list(value, header, errors)))
            }
            "font-src" => policy.font_src = Some((value.into(), parse_list(value, header, errors))),
            "img-src" => {
                policy.img_src = Some((value.into(), parse_list(value, header, errors)));
            }
            "default-src" => {
                policy.default_src = Some((value.into(), parse_list(value, header, errors)))
            }
            "upgrade-insecure-requests" => {
                if disposition == CSPHeaderType::Enforce {
                    policy.upgrade_insecure_requests = true;
                } else {
                    error(errors,header,"The 'upgrade-insecure-requests' directive is ignored in a report-only policy.".into());
                }
            }
            "sandbox" => {
                if disposition == CSPHeaderType::Enforce {
                    let (flags, invalid) = foundation::security::ParseSandboxPolicy(value);
                    policy.sandbox = flags;
                    if !invalid.is_empty() {
                        error(errors,header,format!("Error while parsing the 'sandbox' Content Security Policy directive: '{}' {} invalid sandbox {}.",invalid.join("', '"),if invalid.len()>1{"are"}else{"is an"},if invalid.len()>1{"flags"}else{"flag"}));
                    }
                } else {
                    error(errors,header,"The 'sandbox' directive is ignored when delivered in a report-only policy.".into());
                }
            }
            "block-all-mixed-content" => policy.block_all_mixed_content = true,
            "report-uri" => report_uri = Some(value),
            "report-to" => report_to = Some(value),
            // These source directives are outside these fetch consumers.
            // They remain inert here; this owner is not a script/style CSP API.
            "script-src-v2"
            | "script-src-attr"
            | "style-src"
            | "style-src-attr"
            | "style-src-elem"
            | "object-src"
            | "frame-src"
            | "frame-ancestors"
            | "child-src"
            | "worker-src"
            | "manifest-src"
            | "fenced-frame-src"
            | "treat-as-public-address" => {}
            _ => error(
                errors,
                header,
                format!("Unrecognized Content-Security-Policy directive '{name}'."),
            ),
        }
    }
    if let Some(value) = report_to {
        policy.reporting_api = true;
        if let Some(endpoint) = words(value).next() {
            policy.endpoints.push(endpoint.into());
        }
        if words(value).count() > 1 {
            error(errors,header,"The 'report-to' directive contains more than one endpoint; only the first is used.".into());
        }
    } else if let (Some(value), Some(base)) = (report_uri, base) {
        for endpoint in words(value) {
            match base.join(endpoint){
            Ok(url) if base.scheme()!="https" || origin.IsURLPotentiallyTrustworthy(url.as_str())=>policy.endpoints.push(url.to_string()),
            _=>error(errors,header,format!("The 'report-uri' endpoint '{endpoint}' is invalid or violates Mixed Content policy.")),
        }
        }
    }
    policy
}
fn parse_list(value: &str, header: &str, errors: &mut Vec<CSPParsingError>) -> SourceList {
    let mut list = SourceList::default();
    if value.eq_ignore_ascii_case("'none'") {
        return list;
    }
    let mut has_none = false;
    for token in words(value) {
        if token.eq_ignore_ascii_case("'none'") {
            has_none = true;
            continue;
        }
        if token.eq_ignore_ascii_case("'self'") {
            list.allow_self = true;
            continue;
        }
        if token == "*" {
            list.allow_star = true;
            continue;
        }
        if let Some(source) = parse_source(token) {
            if token.contains(['?', '#']) {
                error(errors, header, format!("The source list contains a source with an invalid path: '{token}'. Its query or fragment component is ignored."));
            }
            list.sources.push(source);
            continue;
        }
        let lower = token.to_ascii_lowercase();
        match lower.as_str() {
            "'unsafe-inline'" => {
                list.allow_inline = true;
                continue;
            }
            "'strict-dynamic'" => {
                list.allow_dynamic = true;
                continue;
            }
            "'unsafe-hashes'" => {
                list.allow_unsafe_hashes = true;
                continue;
            }
            "'report-sample'" => {
                list.report_sample = true;
                continue;
            }
            _ => {}
        }
        if let Some(nonce) = parse_nonce(token) {
            list.nonces.push(nonce);
            continue;
        }
        if let Some(hash) = parse_hash(token) {
            list.hashes.push(hash);
            continue;
        }
        // Execution keywords/nonces/hashes never authorize connect requests.
        // They are valid source-list syntax and therefore produce no URL entry.
        if matches!(
            lower.as_str(),
            "'unsafe-inline'"
                | "'unsafe-eval'"
                | "'wasm-unsafe-eval'"
                | "'wasm-eval'"
                | "'strict-dynamic'"
                | "'unsafe-hashes'"
                | "'report-sample'"
                | "'inline-speculation-rules'"
                | "'trusted-types-eval'"
                | "'report-sha256'"
                | "'report-sha384'"
                | "'report-sha512'"
        ) {
            continue;
        }
        error(
            errors,
            header,
            format!("Invalid Content-Security-Policy source expression '{token}' is ignored."),
        );
    }
    if has_none
        && words(value).any(|token| {
            !token.eq_ignore_ascii_case("'none'") && !token.eq_ignore_ascii_case("'report-sample'")
        })
    {
        error(
            errors,
            header,
            "The 'none' source expression is ignored when combined with other expressions.".into(),
        );
    }
    list
}
// network CSP parser cc542-630: nonce syntax is not digest decoding.
fn valid_base64(value: &str) -> bool {
    let body = value.trim_end_matches('=');
    !body.is_empty()
        && value.len() - body.len() <= 2
        && body
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'+' | b'/' | b'-' | b'_'))
}
fn parse_nonce(token: &str) -> Option<String> {
    if token.len() <= 8
        || !token
            .get(..7)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("'nonce-"))
        || !token.ends_with('\'')
    {
        return None;
    }
    let value = &token[7..token.len() - 1];
    valid_base64(value).then(|| value.to_owned())
}
fn parse_hash(token: &str) -> Option<(HashAlgorithm, Vec<u8>)> {
    let value = token.strip_prefix('\'')?.strip_suffix('\'')?;
    for (prefix, algorithm) in [
        ("sha256-", HashAlgorithm::Sha256),
        ("sha384-", HashAlgorithm::Sha384),
        ("sha512-", HashAlgorithm::Sha512),
        ("sha-256-", HashAlgorithm::Sha256),
        ("sha-384-", HashAlgorithm::Sha384),
        ("sha-512-", HashAlgorithm::Sha512),
        ("ed25519-", HashAlgorithm::Ed25519),
    ] {
        if value
            .get(..prefix.len())
            .is_some_and(|s| s.eq_ignore_ascii_case(prefix))
        {
            let encoded = &value[prefix.len()..];
            if !valid_base64(encoded) {
                return None;
            }
            let normalized = encoded.replace('+', "-").replace('/', "_");
            let decoder = GeneralPurpose::new(
                &alphabet::URL_SAFE,
                GeneralPurposeConfig::new()
                    .with_decode_padding_mode(DecodePaddingMode::Indifferent),
            );
            return Some((algorithm, decoder.decode(normalized).ok()?));
        }
    }
    None
}
fn valid_scheme(value: &str) -> bool {
    value
        .as_bytes()
        .first()
        .is_some_and(u8::is_ascii_alphabetic)
        && value
            .bytes()
            .skip(1)
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'+' | b'-' | b'.'))
}
fn parse_source(token: &str) -> Option<Source> {
    let mut source = Source::default();
    let mut rest = token;
    if let Some(index) = rest.find([':', '/']) {
        if rest.as_bytes()[index] == b':' {
            if index + 1 == rest.len() {
                if !valid_scheme(&rest[..index]) {
                    return None;
                }
                source.scheme = rest[..index].to_ascii_lowercase();
                return Some(source);
            }
            if rest.as_bytes()[index + 1] == b'/' {
                if !rest[index + 1..].starts_with("//") || !valid_scheme(&rest[..index]) {
                    return None;
                }
                source.scheme = rest[..index].to_ascii_lowercase();
                rest = &rest[index + 3..];
            }
        }
    }
    let index = rest.find([':', '/']).unwrap_or(rest.len());
    let mut host = &rest[..index];
    if host.starts_with('*') {
        source.host_wildcard = true;
        if host != "*" {
            host = host.strip_prefix("*.")?;
            if host.is_empty() {
                return None;
            }
        } else {
            host = "";
        }
    }
    if host.is_empty() && !source.host_wildcard {
        return None;
    }
    let pieces: Vec<_> = host.split('.').collect();
    if pieces.iter().enumerate().any(|(i, p)| {
        (p.is_empty() && i + 1 < pieces.len())
            || !p.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
    }) {
        return None;
    }
    source.host = host.to_ascii_lowercase();
    rest = &rest[index..];
    if let Some(port_and_path) = rest.strip_prefix(':') {
        let end = port_and_path.find('/').unwrap_or(port_and_path.len());
        let port = &port_and_path[..end];
        if port == "*" {
            source.port_wildcard = true;
        } else {
            if port.is_empty() || !port.bytes().all(|c| c.is_ascii_digit()) {
                return None;
            }
            source.port = Some(port.parse::<i32>().ok()?);
        }
        rest = &port_and_path[end..];
    }
    if !rest.is_empty() {
        if !rest.starts_with('/') {
            return None;
        }
        source.path = decode_path(rest.split(['?', '#']).next().unwrap());
    }
    Some(source)
}
#[derive(PartialEq, Eq)]
enum Match {
    No,
    Exact,
    Wildcard,
    Upgrade,
}
fn default_port(scheme: &str) -> Option<i32> {
    match scheme {
        "http" | "ws" => Some(80),
        "https" | "wss" => Some(443),
        "ftp" => Some(21),
        _ => None,
    }
}
fn scheme_match(source: &Source, self_scheme: &str, url: &Url) -> Match {
    let scheme = if source.scheme.is_empty() {
        self_scheme
    } else {
        &source.scheme
    };
    if scheme == url.scheme() {
        Match::Exact
    } else if matches!((scheme, url.scheme()), ("http", "https") | ("ws", "wss")) {
        Match::Upgrade
    } else {
        Match::No
    }
}
fn host_match(source: &Source, url: &Url) -> bool {
    let host = if matches!(
        url.scheme(),
        "http" | "https" | "ws" | "wss" | "ftp" | "file"
    ) {
        url.host_str().unwrap_or("")
    } else {
        ""
    };
    if source.host_wildcard {
        source.host.is_empty() || host.ends_with(&format!(".{}", source.host))
    } else {
        source.host == host
    }
}
fn port_match(source: &Source, self_scheme: &str, url: &Url) -> Match {
    if source.port_wildcard {
        return Match::Wildcard;
    }
    let port = url.port().map(i32::from);
    if port == source.port {
        return if port.is_none() {
            Match::Wildcard
        } else {
            Match::Exact
        };
    }
    let http = if source.scheme.is_empty() {
        self_scheme == "http"
    } else {
        source.scheme == "http"
    };
    if (source.port == Some(80) || ((source.port.is_none() || source.port == Some(443)) && http))
        && (port == Some(443) || (port.is_none() && default_port(url.scheme()) == Some(443)))
    {
        return Match::Upgrade;
    }
    if port.is_none() {
        if source.port == default_port(url.scheme()) {
            Match::Exact
        } else {
            Match::No
        }
    } else if source.port.is_none() {
        if port == default_port(url.scheme()) {
            Match::Exact
        } else {
            Match::No
        }
    } else {
        Match::No
    }
}
fn source_matches(
    source: &Source,
    self_scheme: &str,
    url: &Url,
    redirect: CSPRedirectStatus,
) -> bool {
    let scheme = scheme_match(source, self_scheme, url);
    if scheme == Match::No {
        return false;
    }
    if source.host.is_empty() && !source.host_wildcard {
        return true;
    }
    let port = port_match(source, self_scheme, url);
    if (scheme == Match::Upgrade || port == Match::Upgrade)
        && (!(scheme == Match::Upgrade) || !matches!(port, Match::Upgrade | Match::Wildcard))
    {
        return false;
    }
    if port == Match::No || !host_match(source, url) {
        return false;
    }
    if redirect == CSPRedirectStatus::FollowedRedirect
        || source.path.is_empty()
        || (source.path == "/" && url.path().is_empty())
    {
        return true;
    }
    let path = decode_path(url.path());
    if source.path.ends_with('/') {
        path.starts_with(&source.path)
    } else {
        path == source.path
    }
}
fn self_matches(source: &Source, url: &Url) -> bool {
    let scheme = scheme_match(source, &source.scheme, url);
    if url.scheme() == "file" && scheme == Match::Exact {
        return true;
    }
    let host = host_match(source, url);
    let port = port_match(source, &source.scheme, url);
    if scheme == Match::Exact && host && matches!(port, Match::Exact | Match::Wildcard) {
        return true;
    }
    let defaults = port == Match::Exact
        || ((source.port == default_port(&source.scheme) || source.port.is_none())
            && (url.port().is_none() || url.port().map(i32::from) == default_port(url.scheme())));
    host && defaults && (matches!(url.scheme(), "https" | "wss") || source.scheme == "http")
}
fn list_allows(
    list: &SourceList,
    self_source: Option<&Source>,
    url: &Url,
    redirect: CSPRedirectStatus,
) -> (bool, bool) {
    let self_scheme = self_source.map_or("", |s| s.scheme.as_str());
    if list.allow_star
        && (matches!(url.scheme(), "http" | "https")
            || (!url.scheme().is_empty() && url.scheme().eq_ignore_ascii_case(self_scheme)))
    {
        return (true, false);
    }
    if list.allow_self && self_source.is_some_and(|s| self_matches(s, url)) {
        return (true, false);
    }
    if list
        .sources
        .iter()
        .any(|s| source_matches(s, self_scheme, url, redirect))
    {
        return (true, false);
    }
    if list.allow_star && matches!(url.scheme(), "ws" | "wss") {
        return (true, true);
    }
    (false, false)
}
fn decode_path(value: &str) -> String {
    let mut bytes = Vec::with_capacity(value.len());
    let mut rest = value.as_bytes();
    while !rest.is_empty() {
        if rest.len() >= 3 && rest[0] == b'%' {
            if let (Some(a), Some(b)) = (
                (rest[1] as char).to_digit(16),
                (rest[2] as char).to_digit(16),
            ) {
                bytes.push((a * 16 + b) as u8);
                rest = &rest[3..];
                continue;
            }
        }
        bytes.push(rest[0]);
        rest = &rest[1..];
    }
    match String::from_utf8(bytes) {
        Ok(s) => s,
        Err(error) => error.into_bytes().into_iter().map(char::from).collect(),
    }
}
fn strip_url_for_report(value: &str) -> String {
    let Ok(mut url) = Url::parse(value) else {
        return String::new();
    };
    if !matches!(url.scheme(), "http" | "https" | "ws" | "wss") {
        return url.scheme().into();
    }
    let _ = url.set_username("");
    let _ = url.set_password(None);
    url.set_fragment(None);
    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistent_document::{SecurityOrigin, SecurityOriginSchemeRegistry};
    use std::rc::Rc;
    fn install(enforce: &[&str], report: &[&str]) -> DocumentContentSecurityPolicy {
        let owner = DocumentContentSecurityPolicy::default();
        let origin = SecurityOrigin::CreateForNavigation(
            "https://example.test/page",
            None,
            Rc::new(SecurityOriginSchemeRegistry::default()),
        );
        owner.AddResponsePolicies(
            "https://example.test/page",
            &enforce.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            &report.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            &origin,
        );
        owner
    }
    fn request(url: &str) -> CSPRequest<'_> {
        CSPRequest {
            url,
            url_before_redirects: url,
            context: CSPRequestContext::Ping,
            world: CSPWorld::Main,
            redirect: CSPRedirectStatus::NoRedirect,
            reporting: CSPReportingDisposition::Report,
            check_header_type: CSPCheckHeaderType::All,
        }
    }
    // Source CSPSource/CSPSourceList/ContentSecurityPolicy fixtures exercise
    // actual network policy behavior, including protocol/port upgrade pairing.
    #[test]
    fn report_delivery_dedup_keeps_every_violation_event() {
        let owner = install(&["connect-src 'none'; report-uri /csp"], &[]);
        let violation = owner
            .AllowRequest(&request("https://blocked.test/"))
            .violations
            .remove(0);
        let first = owner.PrepareViolationDelivery(violation.clone());
        assert!(matches!(
            first.first(),
            Some(CSPDeliveryEffect::Console { .. })
        ));
        assert_eq!(
            first
                .iter()
                .filter(|effect| matches!(effect, CSPDeliveryEffect::QueueObservedReport { .. }))
                .count(),
            1
        );
        assert_eq!(
            first
                .iter()
                .filter(|effect| matches!(effect, CSPDeliveryEffect::SendLegacyReport { .. }))
                .count(),
            1
        );
        assert!(matches!(
            first.last(),
            Some(CSPDeliveryEffect::QueueViolationEvent { .. })
        ));
        let duplicate = owner.PrepareViolationDelivery(violation);
        assert_eq!(duplicate.len(), 2);
        assert!(matches!(
            duplicate.last(),
            Some(CSPDeliveryEffect::QueueViolationEvent { .. })
        ));
        let report = install(&[], &["connect-src 'none'; report-to csp-group"]);
        let violation = report
            .AllowRequest(&request("https://blocked.test/"))
            .violations
            .remove(0);
        let effects = report.PrepareViolationDelivery(violation);
        assert!(effects.iter().any(|effect|matches!(effect,CSPDeliveryEffect::QueueObservedReport{endpoint_groups,..} if endpoint_groups==&["csp-group"])));
        assert!(!effects
            .iter()
            .any(|effect| matches!(effect, CSPDeliveryEffect::SendLegacyReport { .. })));
    }
    #[test]
    fn connect_policy_source_matching() {
        let cases = [
            ("connect-src 'none'", "https://example.test/ping", false),
            ("default-src 'none'", "https://example.test/ping", false),
            (
                "default-src 'none'; connect-src 'self'",
                "https://example.test/ping",
                true,
            ),
            (
                "connect-src 'none'; connect-src *",
                "https://other.test/",
                false,
            ),
            (
                "connect-src *.example.test",
                "https://sub.example.test/",
                true,
            ),
            ("connect-src *.example.test", "https://example.test/", false),
            (
                "connect-src http://example.test:80",
                "https://example.test/",
                true,
            ),
            (
                "connect-src http://example.test:90",
                "https://example.test:90/",
                false,
            ),
            (
                "connect-src https://example.test/foo/",
                "https://example.test/foo/bar",
                true,
            ),
            (
                "connect-src https://example.test/foo/",
                "https://example.test/foobar",
                false,
            ),
            ("connect-src 'none' https:", "https://other.test/", true),
            ("connect-src *", "data:text/plain,ping", false),
        ];
        for (policy, url, expected) in cases {
            assert_eq!(
                install(&[policy], &[]).AllowRequest(&request(url)).allowed,
                expected,
                "{policy} against {url}"
            );
        }
    }
    #[test]
    fn independent_policies_intersect_and_report_only_never_blocks() {
        let owner = install(
            &["connect-src *", "connect-src 'self'"],
            &["connect-src 'none'"],
        );
        let verdict = owner.AllowRequest(&request("https://other.test/"));
        assert!(!verdict.allowed);
        assert_eq!(verdict.violations.len(), 2);
        assert_eq!(verdict.violations[1].disposition, CSPHeaderType::ReportOnly);
        let only_report = install(&[], &["connect-src 'none'"]);
        assert!(
            only_report
                .AllowRequest(&request("https://other.test/"))
                .allowed
        );
        let mut suppressed = request("https://other.test/");
        suppressed.reporting = CSPReportingDisposition::SuppressReporting;
        assert!(only_report.AllowRequest(&suppressed).violations.is_empty());
    }
    #[test]
    fn redirect_ignores_source_path_but_reports_original_url() {
        let owner = install(&["connect-src https://example.test/allowed/"], &[]);
        let mut redirect = request("https://example.test/other/");
        assert!(!owner.AllowRequest(&redirect).allowed);
        redirect.redirect = CSPRedirectStatus::FollowedRedirect;
        assert!(owner.AllowRequest(&redirect).allowed);
        redirect.url = "https://blocked.test/";
        redirect.url_before_redirects = "https://user:password@example.test/start#private";
        assert_eq!(
            owner.AllowRequest(&redirect).violations[0].blocked_url,
            "https://example.test/start"
        );
    }
    #[test]
    fn keepalive_retains_same_policy_host_and_observes_later_policy_additions() {
        let owner = install(&[], &[]);
        let snapshot = owner.Snapshot();
        let origin = SecurityOrigin::CreateForNavigation(
            "https://example.test",
            None,
            Rc::new(SecurityOriginSchemeRegistry::default()),
        );
        owner.AddResponsePolicies(
            "https://example.test/",
            &["connect-src 'none'".into()],
            &[],
            &origin,
        );
        assert!(
            !owner
                .AllowRequest(&request("https://example.test/"))
                .allowed
        );
        assert!(
            !snapshot
                .AllowRequest(&request("https://example.test/"))
                .allowed
        );
    }
    #[test]
    fn upgrade_preparation_precedes_connect_verdict_and_preserves_local_trust() {
        let owner = install(
            &["connect-src https:; upgrade-insecure-requests; block-all-mixed-content"],
            &[],
        );
        let prepared = owner.PrepareRequestURL("http://remote.test/path");
        assert_eq!(prepared.url, "https://remote.test/path");
        assert!(prepared.did_upgrade_insecure_request && prepared.block_all_mixed_content);
        assert!(owner.AllowRequest(&request(&prepared.url)).allowed);
        let local = owner.PrepareRequestURL("http://127.0.0.1/path");
        assert!(!local.did_upgrade_insecure_request);
        assert_eq!(local.url, "http://127.0.0.1/path");
        let report = install(&[], &["upgrade-insecure-requests; block-all-mixed-content"]);
        let prepared = report.PrepareRequestURL("http://remote.test/");
        assert!(!prepared.did_upgrade_insecure_request && !prepared.block_all_mixed_content);
    }
    #[test]
    fn report_context_world_fallback_and_websocket_wildcard() {
        let owner = install(&["connect-src 'none'"], &[]);
        let mut report = request("https://report.test/");
        report.context = CSPRequestContext::CSPReport;
        let verdict = owner.AllowRequest(&report);
        assert!(verdict.allowed && verdict.violations.is_empty());
        report.context = CSPRequestContext::Beacon;
        report.world = CSPWorld::Isolated(7);
        assert!(!owner.AllowRequest(&report).allowed);
        let verdict = install(&["connect-src *"], &[]).AllowRequest(&request("wss://socket.test/"));
        assert!(verdict.allowed && verdict.wildcard_matches_websocket);
    }
}

fn protected_origin(origin: &crate::persistent_document::SecurityOrigin) -> CSPProtectedOrigin {
    origin.ProtectedOriginForRequest()
}

// Source ContentSecurityPolicy::ReportViolation/PostViolationReport cc1374-1527.
// Source details are assembled once; event/report source defaults are applied
// only by their WebIDL getters/serializers, preserving explicit Unknown input.
fn prepare_delivery(state: &CompiledPolicies, violation: CSPViolation) -> Vec<CSPDeliveryEffect> {
    let mut effects = vec![CSPDeliveryEffect::Console {
        level: violation.console_level,
        message: violation.console_message.clone(),
        location: violation.source_location.clone(),
    }];
    let location = match violation.source_location.clone() {
        CSPSourceLocation::Script { url, line, column } if line > 0 => {
            if let Ok(mut parsed) = Url::parse(&url) {
                if state.bypass_schemes.contains(parsed.scheme()) {
                    return effects;
                }
                parsed.set_query(None);
                CSPSourceLocation::Script {
                    url: strip_url_for_report(parsed.as_str()),
                    line,
                    column,
                }
            } else {
                CSPSourceLocation::Unknown
            }
        }
        _ => CSPSourceLocation::Unknown,
    };
    let data = CSPViolationEventData {
        document_uri: violation.document_url.clone(),
        referrer: violation.referrer.clone(),
        blocked_uri: violation.blocked_url.clone(),
        violated_directive: violation.effective_directive.Name().into(),
        effective_directive: violation.effective_directive.Name().into(),
        original_policy: violation.original_policy.clone(),
        disposition: violation.disposition,
        source_location: location,
        status_code: violation.status_code,
        sample: violation.sample.clone(),
        document_node_id: state.document_node_id,
        eval_hash: String::new(),
        url_hash: String::new(),
        blocked_reason: violation.blocked_reason.clone(),
    };
    let body = data.LegacyReportJSON();
    use std::hash::{Hash, Hasher};
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    body.hash(&mut hash);
    let report_world = match violation.world {
        CSPWorld::Isolated(world) if state.isolated.contains_key(&world) => violation.world,
        _ => CSPWorld::Main,
    };
    if state
        .report_hashes
        .lock()
        .expect("same CSP report cache")
        .entry(report_world)
        .or_default()
        .insert(hash.finish())
    {
        effects.push(CSPDeliveryEffect::QueueObservedReport {
            url: violation.report_url,
            data: data.clone(),
            endpoint_groups: if violation.use_reporting_api {
                violation.report_endpoints.clone()
            } else {
                Vec::new()
            },
        });
        if !violation.use_reporting_api {
            for url in violation.report_endpoints {
                effects.push(CSPDeliveryEffect::SendLegacyReport {
                    url,
                    body: body.clone(),
                    world: violation.world,
                });
            }
        }
    }
    effects.push(CSPDeliveryEffect::QueueViolationEvent { data });
    effects
}

impl DocumentContentSecurityPolicy {
    /// ContentSecurityPolicy::GetSandboxMask h:473-475; EnforceSandboxFlags cc:1211-1214, ApplyPolicySideEffects
    /// cc:506-520. Report-only policy never contributes execution restrictions.
    pub fn SandboxMask(&self) -> foundation::security::SandboxFlags {
        let state = self.compiled.read().expect("Document CSP owner");
        foundation::security::SandboxFlags(
            state
                .main
                .iter()
                .filter(|policy| policy.disposition == CSPHeaderType::Enforce)
                .fold(0, |flags, policy| flags | policy.sandbox.0),
        )
    }
}

/// network OriginWithPossibleWildcards.cc52-94 reuses the compiled source parser
/// and the PermissionsPolicy matching context (which forbids upgrades).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PermissionsOriginPattern(Source);
impl PermissionsOriginPattern {
    /// OriginWithPossibleWildcards::FromOrigin, source.cc:27-35. Serialize the
    /// original immutable url::Origin value, then reuse this same CSP parser.
    pub(crate) fn FromOrigin(origin: &foundation::security::OriginKey) -> Option<Self> {
        let foundation::security::OriginKey::Tuple { scheme, host, port } = origin else {
            return None;
        };
        let host = if host.contains(':') {
            format!("[{host}]")
        } else {
            host.clone()
        };
        let mut value = format!("{scheme}://{host}");
        if Some(i32::from(*port)) != default_port(scheme) && *port != 0 {
            value.push(':');
            value.push_str(&port.to_string());
        }
        Self::Parse(&value, true)
    }
    pub(crate) fn Parse(input: &str, allow_wildcards: bool) -> Option<Self> {
        let mut source = parse_source(input)?;
        if source.scheme.is_empty()
            || (!allow_wildcards
                && (source.host.is_empty() || source.host_wildcard || source.port_wildcard))
        {
            return None;
        }
        source.path.clear();
        Some(Self(source))
    }
    pub(crate) fn Matches(&self, origin: &foundation::security::OriginKey) -> bool {
        let foundation::security::OriginKey::Tuple { scheme, host, port } = origin else {
            return false;
        };
        let source = &self.0;
        if source.host.is_empty() && !source.host_wildcard {
            return source.scheme == *scheme
                || matches!(
                    (source.scheme.as_str(), scheme.as_str()),
                    ("http", "https") | ("ws", "wss")
                );
        }
        if source.scheme != *scheme {
            return false;
        }
        if !(if source.host_wildcard {
            source.host.is_empty() || host.ends_with(&format!(".{}", source.host))
        } else {
            source.host == *host
        }) {
            return false;
        }
        source.port_wildcard
            || source.port.or_else(|| default_port(&source.scheme)) == Some(i32::from(*port))
    }
}

impl DocumentContentSecurityPolicy {
    pub fn BlocksAllMixedContent(&self) -> bool {
        self.compiled
            .read()
            .expect("Document CSP owner")
            .main
            .iter()
            .any(|policy| {
                policy.disposition == CSPHeaderType::Enforce && policy.block_all_mixed_content
            })
    }
}

impl DocumentContentSecurityPolicy {
    /// DocumentLoader::CreateCSP cc4034-4050: a fresh CSP delegate shares the
    /// immutable policy ASTs from the previous PolicyContainer. It has its own
    /// report hashes, violations and document binding; no header is reparsed.
    pub(crate) fn CopyPoliciesForNavigation(&self) -> Arc<Self> {
        let old = self.compiled.read().expect("original policy container");
        let compiled = CompiledPolicies {
            main: old.main.clone(),
            bypass_schemes: old.bypass_schemes.clone(),
            trust_registry: old.trust_registry.clone(),
            protected_origin: old.protected_origin.clone(),
            ..Default::default()
        };
        Arc::new(Self {
            compiled: Arc::new(RwLock::new(compiled)),
            parsing_errors: Mutex::new(Vec::new()),
            local_violations: Mutex::new(Vec::new()),
        })
    }
}
