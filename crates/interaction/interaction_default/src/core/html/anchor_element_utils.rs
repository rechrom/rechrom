// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/html/anchor_element_utils.h:30-39; .cc:197-230.
// Shared pure parser; canonical DOM anchor classes retain the result.
pub use foundation::anchor_element_utils::ParseRelAttribute;

// Source .cc:43-97,120-194,233-277; PingLoader.cc:132-169.
use crate::bindings::core::v8::script_promise::ScriptInvocation;
use crate::core::loader::resource_request::*;
use interaction_dispatch::core::dom::events::event::EventTargetRef;
use std::{collections::HashSet, rc::Rc};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigateDispatchResult {
    Continue,
    Abort,
    Intercept,
}
/// Precise required NavigationApi entrypoints, never an automatic Continue.
/// Owner translation: core/navigation_api/navigation_api.cc:715-1003,1043-1110
/// DispatchNavigateEvent and InformAboutCanceledNavigation. Script event/promise
/// and HistoryItem owners must be admitted together with this capability.
pub trait AnchorNavigationApi {
    fn DispatchNavigateEvent(
        &self,
        invocation: &dyn ScriptInvocation,
        params: DownloadNavigateEventParams,
    ) -> NavigateDispatchResult;
    fn InformAboutCanceledNavigationDropped(&self, invocation: &dyn ScriptInvocation);
}
pub struct DownloadNavigateEventParams {
    pub url: String,
    pub source_element: EventTargetRef,
    pub download_filename: String,
    /// The source event type is CrossDocument, load type Standard; activation is
    /// present only for a trusted triggering event.
    pub activation: bool,
}
pub trait AnchorElementUtilsEnvironment {
    fn HasTransientUserActivation(&self) -> bool;
    fn IsAdFrame(&self) -> bool;
    fn IsAdScriptInStack(&self) -> bool;
    fn SandboxedDownloads(&self) -> bool;
    fn RequestorOrigin(&self) -> RequestorOrigin;
    fn DocumentReferrerPolicy(&self) -> ReferrerPolicy;
    fn OutgoingReferrer(&self) -> String;
    fn GenerateReferrer(&self, policy: ReferrerPolicy, url: &str, source: &str) -> String;
    fn NavigationApi(&self) -> &dyn AnchorNavigationApi;
    fn DownloadURL(&self, request: Rc<ResourceRequest>);
    fn HyperlinkAuditingEnabled(&self) -> bool;
    fn HasArchive(&self) -> bool;
    fn CompleteURL(&self, url: &str) -> String;
    fn DocumentURL(&self) -> String;
    fn IsHTTPFamily(&self, url: &str) -> bool;
    fn IsHTTP(&self, url: &str) -> bool;
    fn CanAccessPingOrigin(&self, url: &str) -> bool;
    fn FetchLinkAuditPing(
        &self,
        invocation: Option<&dyn ScriptInvocation>,
        request: Rc<ResourceRequest>,
    );
    fn CountUse(&self, feature: &'static str);
    fn CountDeprecation(&self, feature: &'static str);
    fn DownloadAttributeTooLong(&self, length: usize);
}
pub fn ShouldInterveneDownloadByFramePolicy(e: &dyn AnchorElementUtilsEnvironment) -> bool {
    e.CountUse("DownloadPrePolicyCheck");
    let has_gesture = e.HasTransientUserActivation();
    if !has_gesture {
        e.CountUse("DownloadWithoutUserGesture");
    }
    let mut intervene = false;
    if e.IsAdFrame() {
        e.CountUse("DownloadInAdFrame");
        if !has_gesture {
            e.CountUse("DownloadInAdFrameWithoutUserGesture");
            intervene = true;
        }
    } else if e.IsAdScriptInStack() {
        e.CountUse("DownloadFromAdScript");
    }
    if e.SandboxedDownloads() {
        e.CountUse("DownloadInSandbox");
        intervene = true;
    }
    if !intervene {
        e.CountUse("DownloadPostPolicyCheck");
    }
    intervene
}
pub fn HandleDownloadAttribute(
    invocation: &dyn ScriptInvocation,
    e: &dyn AnchorElementUtilsEnvironment,
    source: EventTargetRef,
    download: &str,
    url: &str,
    trusted: bool,
    request: Rc<ResourceRequest>,
) {
    HandleDownloadAttributeWithInvocation(
        Some(invocation),
        e,
        source,
        download,
        url,
        trusted,
        request,
    );
}
pub fn HandleDownloadAttributeWithInvocation(
    invocation: Option<&dyn ScriptInvocation>,
    e: &dyn AnchorElementUtilsEnvironment,
    source: EventTargetRef,
    download: &str,
    url: &str,
    trusted: bool,
    request: Rc<ResourceRequest>,
) {
    if ShouldInterveneDownloadByFramePolicy(e) {
        return;
    }
    let length = download.encode_utf16().count();
    if length > 1_000_000 {
        e.DownloadAttributeTooLong(length);
        return;
    }
    if let Some(invocation) = invocation {
        if e.NavigationApi().DispatchNavigateEvent(
            invocation,
            DownloadNavigateEventParams {
                url: url.into(),
                source_element: source,
                download_filename: download.into(),
                activation: trusted,
            },
        ) != NavigateDispatchResult::Continue
        {
            return;
        }
        e.NavigationApi()
            .InformAboutCanceledNavigationDropped(invocation);
    }
    let mut policy = request.Read(|r| r.referrer_policy);
    if policy == ReferrerPolicy::Default {
        policy = e.DocumentReferrerPolicy();
    }
    let referrer = e.GenerateReferrer(policy, url, &e.OutgoingReferrer());
    request.Mutate(|r| {
        r.suggested_filename = Some(download.into());
        r.context = RequestContext::Download;
        r.requestor_origin = Some(e.RequestorOrigin());
        r.has_user_gesture = e.HasTransientUserActivation();
        r.referrer = referrer;
        r.referrer_policy = policy;
    });
    e.DownloadURL(request);
}
pub fn ReferrerPolicyFromStringLegacy(value: &str) -> Option<ReferrerPolicy> {
    // SecurityPolicy.cc:245-301: no whitespace trimming, legacy aliases admitted.
    Some(match value.to_ascii_lowercase().as_str() {
        "no-referrer" | "never" | "none" => ReferrerPolicy::Never,
        "unsafe-url" | "always" => ReferrerPolicy::Always,
        "origin" => ReferrerPolicy::Origin,
        "origin-when-cross-origin" | "origin-when-crossorigin" => {
            ReferrerPolicy::OriginWhenCrossOrigin
        }
        "same-origin" => ReferrerPolicy::SameOrigin,
        "strict-origin" => ReferrerPolicy::StrictOrigin,
        "strict-origin-when-cross-origin" => ReferrerPolicy::StrictOriginWhenCrossOrigin,
        "no-referrer-when-downgrade" => ReferrerPolicy::NoReferrerWhenDowngrade,
        "default" => ReferrerPolicy::StrictOriginWhenCrossOrigin,
        _ => return None,
    })
}
pub fn HandleReferrerPolicyAttribute(
    e: &dyn AnchorElementUtilsEnvironment,
    request: &ResourceRequest,
    value: &str,
    relations: u32,
) {
    if let Some(policy) = ReferrerPolicyForAttribute(value, relations) {
        e.CountUse("HTMLAnchorElementReferrerPolicyAttribute");
        request.Mutate(|r| r.referrer_policy = policy);
    }
}
/// Source .cc:233-244. The policy decision is shared by the native DOM adapter
/// and the instrumented host projection, without a second attribute parser.
pub fn ReferrerPolicyForAttribute(value: &str, relations: u32) -> Option<ReferrerPolicy> {
    if value.is_empty() || relations & 0x1000 != 0 {
        return None;
    }
    ReferrerPolicyFromStringLegacy(value)
}
pub fn SendPings(
    e: &dyn AnchorElementUtilsEnvironment,
    invocation: Option<&dyn ScriptInvocation>,
    destination: &str,
    ping: Option<&str>,
) {
    let Some(ping) = ping else {
        return;
    };
    if !e.HyperlinkAuditingEnabled() || e.HasArchive() {
        return;
    }
    if ping.contains(['\n', '\r', '\t']) && ping.contains('<') {
        e.CountDeprecation("CanRequestURLHTTPContainingNewline");
        return;
    }
    e.CountUse("HTMLAnchorElementPingAttribute");
    let mut seen = HashSet::new();
    for value in ping
        .split(|c| matches!(c, '\t' | '\n' | '\u{000c}' | '\r' | ' '))
        .filter(|v| !v.is_empty())
    {
        if !seen.insert(value) {
            continue;
        }
        SendLinkAuditPing(e, invocation, &e.CompleteURL(value), destination);
    }
}
pub fn SendLinkAuditPing(
    e: &dyn AnchorElementUtilsEnvironment,
    invocation: Option<&dyn ScriptInvocation>,
    ping: &str,
    destination: &str,
) {
    if !e.IsHTTPFamily(ping) {
        return;
    }
    let request = ResourceRequest::new(ping);
    let document = e.DocumentURL();
    request.Mutate(|r| {
        r.method = "POST".into();
        r.body = Some(
            interaction_dispatch::core::fileapi::encoded_form_data::EncodedFormData::FromBytes(
                std::sync::Arc::<[u8]>::from(&b"PING"[..]),
            ),
        );
        r.headers = vec![
            ("Content-Type".into(), "text/ping".into()),
            ("Cache-Control".into(), "max-age=0".into()),
            ("Ping-To".into(), destination.into()),
        ];
        if e.IsHTTP(&document) || e.CanAccessPingOrigin(ping) {
            r.headers.push(("Ping-From".into(), document));
        }
        r.keepalive = true;
        r.referrer.clear();
        r.referrer_policy = ReferrerPolicy::Never;
        r.context = RequestContext::Ping;
    });
    e.FetchLinkAuditPing(invocation, request);
}
