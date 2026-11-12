//! Source MixedContentChecker's RawResource and native image request branches.
//! Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15:
//! core/loader/mixed_content_checker.cc:251-264,278-285,328-417,497-687,
//! 921-1025,1139-1260;
//! platform/loader/mixed_content.cc:58-90; scheme_registry.cc:159-162;
//! services/network/public/cpp/ip_address_space_util.cc:249-320,470-490.
//! Input contains two original frame facts and explicit optional capabilities;
//! no Document, Frame tree or engine callback is retained by network requests.
#![allow(non_snake_case)]
use foundation::content_security_policy::{
    CSPProtectedOrigin, CSPRedirectStatus, CSPRequestContext, CSPRequestPolicy,
};
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    sync::Arc,
};
use url::Url;
#[derive(Clone, Debug)]
pub struct RawFetchFrameSettings {
    pub strict_mixed_content_checking: bool,
    pub strictly_block_blockable_mixed_content: bool,
    pub allow_running_insecure_content: bool,
}
#[derive(Clone, Debug)]
pub struct RawFetchFrameSecurity {
    pub origin: CSPProtectedOrigin,
    pub settings: RawFetchFrameSettings,
    pub block_all_mixed_content: bool,
}
/// Source WebContentSettingsClient. None in the context means there is no
/// client override; it does not grant permission to run insecure content.
pub trait RawFetchContentSettingsClient: Send + Sync {
    fn ShouldAutoupgradeMixedContent(&self) -> bool;
    fn AllowRunningInsecureContent(
        &self,
        allowed_per_settings: bool,
        request: &RawMixedContentRequest<'_>,
    ) -> bool;
}
pub use foundation::network_address_space::IPAddressSpace as RawFetchAddressSpace;
mod local_network_access;
pub use local_network_access::*;
pub struct RawMixedContentRequest<'a> {
    pub url: &'a str,
    pub url_before_redirects: &'a str,
    pub context: CSPRequestContext,
    pub redirect: CSPRedirectStatus,
    pub current_frame_origin: &'a CSPProtectedOrigin,
    pub target_address_space: RawFetchAddressSpace,
}
/// Optional network AddressSpaceOverrides native owner. None uses the source's
/// original no-overrides mapping; overrides never come from DOM/URL guesses.
pub trait RawFetchAddressSpaceOverrides: Send + Sync {
    fn ForEndpoint(&self, address: IpAddr, port: u16) -> Option<RawFetchAddressSpace>;
}
#[derive(Clone)]
pub struct RawFetchSecurityContext {
    pub current_frame: Option<RawFetchFrameSecurity>,
    pub top_frame: Option<RawFetchFrameSecurity>,
    pub current_frame_is_top: bool,
    pub content_settings_client: Option<Arc<dyn RawFetchContentSettingsClient>>,
    /// blink::features::kMixedContentAutoupgrade, independent of CSP UIR.
    pub mixed_content_autoupgrade: bool,
    pub local_network_access_checks: bool,
    pub target_address_space: RawFetchAddressSpace,
    pub local_network_access_client_security_state: Option<RawLocalNetworkClientSecurityState>,
    pub block_local_request: bool,
    pub local_network_access_policy: Option<Arc<dyn RawFetchLocalNetworkAccessPolicy>>,
    pub keepalive_redirect_frame: Option<RawKeepaliveRedirectFrame>,
    pub address_space_overrides: Option<Arc<dyn RawFetchAddressSpaceOverrides>>,
}
impl std::fmt::Debug for RawFetchSecurityContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawFetchSecurityContext")
            .field("current_frame", &self.current_frame)
            .field("top_frame", &self.top_frame)
            .field("current_frame_is_top", &self.current_frame_is_top)
            .field(
                "has_content_settings_client",
                &self.content_settings_client.is_some(),
            )
            .field(
                "local_network_access_checks",
                &self.local_network_access_checks,
            )
            .field("target_address_space", &self.target_address_space)
            .field(
                "has_local_network_access_policy",
                &self.local_network_access_policy.is_some(),
            )
            .finish_non_exhaustive()
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawMixedContentEffect {
    pub url: String,
    pub url_before_redirects: String,
    pub context: CSPRequestContext,
    pub allowed: bool,
    pub mixed_with_top_frame: bool,
    pub cancel_prerendering: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RawMixedContentBlockedReason {
    MixedContent,
    LocalNetworkAccess,
    MissingLocalNetworkAccessPolicy,
    MissingTopFrameFacts,
    MissingKeepaliveRedirectFrameFacts,
}
pub struct RawMixedContentResult {
    pub blocked: Option<RawMixedContentBlockedReason>,
    pub effect: Option<RawMixedContentEffect>,
}
fn protocol(origin: &CSPProtectedOrigin) -> Option<&str> {
    match origin {
        CSPProtectedOrigin::Tuple { scheme, .. } => Some(scheme),
        CSPProtectedOrigin::Opaque {
            precursor_scheme, ..
        } => precursor_scheme.as_deref(),
    }
}
/// MixedContentChecker.cc:921-1025,1163-1203. The stable runtime feature
/// consults the original top/current frame origins, including opaque precursors.
/// The returned bit is retained on ResourceRequest even when its URL is HTTPS.
pub fn ShouldAutoupgradeImage(
    security: &RawFetchSecurityContext,
    context: CSPRequestContext,
    url: &str,
) -> bool {
    if !security.mixed_content_autoupgrade || context != CSPRequestContext::Image {
        return false;
    }
    let restricted = [&security.top_frame, &security.current_frame]
        .into_iter()
        .flatten()
        .any(|frame| protocol(&frame.origin) == Some("https"));
    if !restricted
        || security
            .content_settings_client
            .as_ref()
            .is_some_and(|client| !client.ShouldAutoupgradeMixedContent())
    {
        return false;
    }
    let Ok(url) = Url::parse(url) else {
        return false;
    };
    if matches!(url.host(), Some(url::Host::Ipv4(_) | url::Host::Ipv6(_))) {
        return false;
    }
    !(security.local_network_access_checks && IsLocalTarget(security, &url))
}
/// ResourceRequest's upgrade-if-insecure handling; the original HTTP default
/// port becomes HTTPS's default port. Trustworthy loopback remains HTTP.
pub fn UpgradeInsecureURL(policy: &dyn CSPRequestPolicy, url: &str) -> Option<String> {
    let mut parsed = Url::parse(url).ok()?;
    if parsed.scheme() != "http" || policy.IsURLPotentiallyTrustworthy(url) {
        return None;
    }
    parsed.set_scheme("https").ok()?;
    Some(parsed.to_string())
}
pub fn CheckRawMixedContent(
    security: &RawFetchSecurityContext,
    policy: &dyn CSPRequestPolicy,
    url: &str,
    before: &str,
    context: CSPRequestContext,
    redirect: CSPRedirectStatus,
    current_block_all: bool,
) -> RawMixedContentResult {
    let Some(current) = security.current_frame.as_ref() else {
        return RawMixedContentResult {
            blocked: None,
            effect: None,
        };
    };
    // Source checks top first, then current; trustworthy URLs are never mixed.
    if policy.IsURLPotentiallyTrustworthy(url) {
        return RawMixedContentResult {
            blocked: None,
            effect: None,
        };
    }
    let Some(top) = security.top_frame.as_ref() else {
        return RawMixedContentResult {
            blocked: Some(RawMixedContentBlockedReason::MissingTopFrameFacts),
            effect: None,
        };
    };
    let top_mixed = protocol(&top.origin) == Some("https");
    let mixed = if top_mixed {
        top
    } else if protocol(&current.origin) == Some("https") {
        current
    } else {
        return RawMixedContentResult {
            blocked: None,
            effect: None,
        };
    };
    let Ok(parsed) = Url::parse(url) else {
        return RawMixedContentResult {
            blocked: Some(RawMixedContentBlockedReason::MixedContent),
            effect: None,
        };
    };
    // This producer admits only HTTP(S), so the source's CORS-enabled-scheme
    // branch is exact. Other schemes remain the URL loader's explicit boundary.
    if !matches!(parsed.scheme(), "http" | "https") {
        return RawMixedContentResult {
            blocked: None,
            effect: None,
        };
    }
    let strict = mixed.block_all_mixed_content
        || (!top_mixed && current_block_all)
        || (security.current_frame_is_top && current_block_all)
        || mixed.settings.strict_mixed_content_checking;
    let request = RawMixedContentRequest {
        url,
        url_before_redirects: before,
        context,
        redirect,
        current_frame_origin: &current.origin,
        target_address_space: security.target_address_space,
    };
    // MixedContentChecker.cc:554-579: raw IMAGE is optionally blockable;
    // IMAGE_SET stays blockable. Insecure IP literals cannot use this branch.
    let mut allowed = false;
    if context == CSPRequestContext::Image {
        allowed = !strict
            && parsed
                .host_str()
                .is_some_and(|host| host.parse::<IpAddr>().is_err());
    } else if !mixed.settings.allow_running_insecure_content
        && !security.current_frame_is_top
        && protocol(&current.origin) == Some("https")
    {
        // Strictly block the source's cross-origin subframe subresource case.
    } else if !strict
        && (!mixed.settings.strictly_block_blockable_mixed_content
            || mixed.settings.allow_running_insecure_content)
    {
        allowed = mixed.settings.allow_running_insecure_content;
        if let Some(client) = &security.content_settings_client {
            allowed = client.AllowRunningInsecureContent(allowed, &request);
        }
    }
    // Source MixedContentChecker grants the exemption here; the network's
    // separate LNA permission owner gets the final decision afterwards.
    let mut blocked = None;
    if security.local_network_access_checks && IsLocalTarget(security, &parsed) {
        allowed = true;
    }
    if !allowed && blocked.is_none() {
        blocked = Some(RawMixedContentBlockedReason::MixedContent);
    }
    RawMixedContentResult {
        blocked,
        effect: Some(RawMixedContentEffect {
            url: url.into(),
            url_before_redirects: before.into(),
            context,
            allowed,
            mixed_with_top_frame: top_mixed,
            cancel_prerendering: false,
        }),
    }
}
/// Browser KeepAliveURLLoader's live initiator-frame capability. None is
/// explicitly missing facts, not proof that the frame is outside fenced or
/// prerendering trees. Delegate None is source WebContentsImpl's actual no-client
/// branch, returning allowed_per_prefs (web_contents_impl.cc:8685-8696).
#[derive(Clone)]
pub struct RawKeepaliveRedirectFrame {
    pub is_in_fenced_frame_tree: bool,
    pub is_prerendering: bool,
    pub cancel_prerendering_on_mixed_content: bool,
    pub delegate: Option<Arc<dyn RawFetchContentSettingsClient>>,
}
/// content/browser/renderer_host/mixed_content_checker.cc:79-109,239-375,443-468.
/// This is deliberately distinct from the renderer subresource branch above.
pub fn CheckRawKeepaliveRedirectMixedContent(
    security: &RawFetchSecurityContext,
    policy: &dyn CSPRequestPolicy,
    url: &str,
    before: &str,
    context: CSPRequestContext,
    current_block_all: bool,
) -> RawMixedContentResult {
    let none = || RawMixedContentResult {
        blocked: None,
        effect: None,
    };
    let Some(current) = security.current_frame.as_ref() else {
        return none();
    };
    if policy.IsURLPotentiallyTrustworthy(url) {
        return none();
    }
    let Some(top) = security.top_frame.as_ref() else {
        return RawMixedContentResult {
            blocked: Some(RawMixedContentBlockedReason::MissingTopFrameFacts),
            effect: None,
        };
    };
    let top_mixed = protocol(&top.origin) == Some("https");
    let mixed = if top_mixed {
        top
    } else if protocol(&current.origin) == Some("https") {
        current
    } else {
        return none();
    };
    let Ok(parsed) = Url::parse(url) else {
        return RawMixedContentResult {
            blocked: Some(RawMixedContentBlockedReason::MixedContent),
            effect: None,
        };
    };
    if !matches!(parsed.scheme(), "http" | "https") {
        return none();
    }
    let Some(frame) = &security.keepalive_redirect_frame else {
        return RawMixedContentResult {
            blocked: Some(RawMixedContentBlockedReason::MissingKeepaliveRedirectFrameFacts),
            effect: None,
        };
    };
    let cancel = frame.is_prerendering && frame.cancel_prerendering_on_mixed_content;
    let strict = mixed.settings.strict_mixed_content_checking
        || mixed.block_all_mixed_content
        || ((!top_mixed || security.current_frame_is_top) && current_block_all)
        || frame.is_in_fenced_frame_tree;
    let mut allowed = false;
    if !cancel
        && !strict
        && (!mixed.settings.strictly_block_blockable_mixed_content
            || mixed.settings.allow_running_insecure_content)
    {
        allowed = mixed.settings.allow_running_insecure_content;
        if let Some(delegate) = &frame.delegate {
            allowed = delegate.AllowRunningInsecureContent(
                allowed,
                &RawMixedContentRequest {
                    url,
                    url_before_redirects: before,
                    context,
                    redirect: CSPRedirectStatus::FollowedRedirect,
                    current_frame_origin: &mixed.origin,
                    target_address_space: security.target_address_space,
                },
            );
        }
    }
    // Browser branch recognizes the URL's local address space; unlike the
    // renderer it does not use required_ip_address_space for this exemption.
    if !cancel
        && security.local_network_access_checks
        && matches!(
            AddressSpaceFromURL(&parsed, security.address_space_overrides.as_deref()),
            Some(RawFetchAddressSpace::Local | RawFetchAddressSpace::Loopback)
        )
    {
        allowed = true;
    }
    RawMixedContentResult {
        blocked: (!allowed).then_some(RawMixedContentBlockedReason::MixedContent),
        effect: Some(RawMixedContentEffect {
            url: url.into(),
            url_before_redirects: before.into(),
            context,
            allowed,
            mixed_with_top_frame: top_mixed,
            cancel_prerendering: cancel,
        }),
    }
}
fn IsLocalTarget(security: &RawFetchSecurityContext, url: &Url) -> bool {
    matches!(
        security.target_address_space,
        RawFetchAddressSpace::Local | RawFetchAddressSpace::Loopback
    ) || matches!(
        AddressSpaceFromURL(url, security.address_space_overrides.as_deref()),
        Some(RawFetchAddressSpace::Local | RawFetchAddressSpace::Loopback)
    )
}

fn AddressSpaceFromURL(
    url: &Url,
    overrides: Option<&dyn RawFetchAddressSpaceOverrides>,
) -> Option<RawFetchAddressSpace> {
    let host = url.host_str()?.trim_matches(['[', ']']);
    let host = host.strip_suffix('.').unwrap_or(host);
    if host == "local" || host.ends_with(".local") {
        return Some(RawFetchAddressSpace::Local);
    }
    let ip = if host == "localhost" || host.ends_with(".localhost") {
        IpAddr::V4(Ipv4Addr::LOCALHOST)
    } else {
        host.parse::<IpAddr>().ok()?
    };
    if let Some(space) =
        overrides.and_then(|o| o.ForEndpoint(ip, url.port_or_known_default().unwrap_or(0)))
    {
        return Some(space);
    }
    Some(AddressSpaceFromIP(ip))
}
pub fn AddressSpaceFromIP(ip: IpAddr) -> RawFetchAddressSpace {
    use RawFetchAddressSpace::*;
    match ip {
        IpAddr::V4(ip) => {
            let [a, b, c, d] = ip.octets();
            if a == 127 || (a, b, c, d) == (0, 0, 0, 0) {
                Loopback
            } else if a == 0
                || a == 10
                || a == 172 && (16..32).contains(&b)
                || a == 192 && b == 168
                || a == 169 && b == 254
                || a == 100 && (64..128).contains(&b)
            {
                Local
            } else {
                Public
            }
        }
        IpAddr::V6(ip) => {
            if let Some(v4) = ip.to_ipv4_mapped() {
                return AddressSpaceFromIP(IpAddr::V4(v4));
            }
            let v = ip.octets();
            if ip == Ipv6Addr::LOCALHOST || ip.is_unspecified() {
                Loopback
            } else if v[0] & 0xfe == 0xfc
                || v[0] == 0xfe && matches!(v[1] & 0xc0, 0x80 | 0xc0)
                || v[0..4] == [0x20, 0x01, 0x0d, 0xb8]
                || v[0] == 0x3f && v[1] == 0xff && v[2] & 0xf0 == 0
            {
                Local
            } else {
                Public
            }
        }
    }
}
