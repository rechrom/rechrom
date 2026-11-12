//! Canonical Window SecurityContext and immutable SecurityOrigin identity.
//! Source security_origin.cc:104-165,225-254,474-483,756-766;
//! security_context.cc:92-145; net/base/is_potentially_trustworthy.cc:271-345.
//! Loader commit constructs the origin once. Input/event queries never infer
//! security from the current URL, which may change without changing origin.
use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    net::{IpAddr, Ipv6Addr},
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SecurityOriginTuple {
    pub scheme: String,
    pub host: String,
    pub port: u16,
}
#[derive(Clone, Debug)]
pub struct SecurityOriginSchemeRegistry {
    data: std::sync::Arc<SecurityOriginSchemeRegistryData>,
    access_list: std::sync::Arc<foundation::origin_access_list::OriginAccessList>,
}
#[derive(Clone, Debug)]
struct SecurityOriginSchemeRegistryData {
    standard: HashSet<String>,
    secure: HashSet<String>,
    local: HashSet<String>,
    no_access: HashSet<String>,
    display_isolated: HashSet<String>,
    bypass: HashSet<String>,
    allowlisted: HashSet<SecurityOriginTuple>,
}
impl Default for SecurityOriginSchemeRegistry {
    fn default() -> Self {
        Self {
            access_list: foundation::origin_access_list::OriginAccessList::Global(),
            data: std::sync::Arc::new(SecurityOriginSchemeRegistryData {
                standard: ["http", "https", "ws", "wss", "ftp", "file"]
                    .map(str::to_owned)
                    .into(),
                secure: ["https", "wss"].map(str::to_owned).into(),
                local: ["file"].map(str::to_owned).into(),
                no_access: ["about", "data", "javascript"].map(str::to_owned).into(),
                display_isolated: HashSet::new(),
                bypass: HashSet::new(),
                allowlisted: HashSet::new(),
            }),
        }
    }
}
impl SecurityOriginSchemeRegistry {
    /// Source url::AddStandardScheme/AddSecureScheme/AddLocalScheme and
    /// SchemeRegistry's bypass list. Registration precedes loader admission.
    pub fn RegisterScheme(
        &mut self,
        scheme: &str,
        standard: bool,
        secure: bool,
        local: bool,
        bypass_secure_context_check: bool,
    ) {
        let data = std::sync::Arc::make_mut(&mut self.data);
        if standard {
            data.standard.insert(scheme.into());
        }
        if secure {
            data.secure.insert(scheme.into());
        }
        if local {
            data.local.insert(scheme.into());
        }
        if bypass_secure_context_check {
            data.bypass.insert(scheme.into());
        }
    }
    // SchemeRegistry.cc:145-156. Registration belongs to product initialization.
    pub fn RegisterURLSchemeAsDisplayIsolated(&mut self, scheme: &str) {
        assert_eq!(scheme, scheme.to_ascii_lowercase());
        std::sync::Arc::make_mut(&mut self.data)
            .display_isolated
            .insert(scheme.into());
    }
    pub fn AddTrustworthyOrigin(&mut self, origin: SecurityOriginTuple) {
        std::sync::Arc::make_mut(&mut self.data)
            .allowlisted
            .insert(origin);
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum OriginIdentity {
    Tuple(SecurityOriginTuple),
    Opaque(u64),
}
static NEXT_OPAQUE_ORIGIN: AtomicU64 = AtomicU64::new(1);
#[derive(Clone, Debug)]
pub struct SecurityOrigin {
    identity: OriginIdentity,
    precursor: Option<Rc<SecurityOrigin>>,
    opaque_potentially_trustworthy: Cell<bool>,
    registry: Rc<SecurityOriginSchemeRegistry>,
    block_local_access_from_local_origin: Cell<bool>,
    universal_access: Cell<bool>,
    can_load_local_resources: Cell<bool>,
    cross_agent_cluster_access: Cell<bool>,
    domain_override: RefCell<Option<String>>,
    agent_cluster: Cell<Option<[u64; 2]>>,
}
impl SecurityOrigin {
    pub fn CreateUniqueOpaque(registry: Rc<SecurityOriginSchemeRegistry>) -> Rc<Self> {
        Rc::new(Self {
            identity: OriginIdentity::Opaque(NEXT_OPAQUE_ORIGIN.fetch_add(1, Ordering::Relaxed)),
            precursor: None,
            opaque_potentially_trustworthy: Cell::new(false),
            registry,
            block_local_access_from_local_origin: Cell::new(false),
            universal_access: Cell::new(false),
            can_load_local_resources: Cell::new(false),
            cross_agent_cluster_access: Cell::new(false),
            domain_override: RefCell::new(None),
            agent_cluster: Cell::new(None),
        })
    }
    pub fn DeriveNewOpaqueOrigin(self: &Rc<Self>) -> Rc<Self> {
        Rc::new(Self {
            identity: OriginIdentity::Opaque(NEXT_OPAQUE_ORIGIN.fetch_add(1, Ordering::Relaxed)),
            // security_origin.cc:184-185,756-768: the new opaque constructor
            // records the original precursor and keeps the source false flag.
            precursor: Some(self.precursor.clone().unwrap_or_else(|| self.clone())),
            opaque_potentially_trustworthy: Cell::new(false),
            registry: self.registry.clone(),
            block_local_access_from_local_origin: Cell::new(false),
            universal_access: Cell::new(false),
            can_load_local_resources: Cell::new(false),
            cross_agent_cluster_access: Cell::new(false),
            domain_override: RefCell::new(None),
            agent_cluster: Cell::new(None),
        })
    }
    pub fn CreateForNavigation(
        url: &str,
        reference: Option<&Rc<Self>>,
        registry: Rc<SecurityOriginSchemeRegistry>,
    ) -> Rc<Self> {
        let Ok(mut url) = url::Url::parse(url) else {
            return reference.map_or_else(
                || Self::CreateUniqueOpaque(registry),
                |origin| origin.DeriveNewOpaqueOrigin(),
            );
        };
        if url.scheme() == "about" && matches!(url.path(), "blank" | "srcdoc") {
            return reference
                .cloned()
                .unwrap_or_else(|| Self::CreateUniqueOpaque(registry));
        }
        // Blob/filesystem origins use the one inner URL, never nested wrappers.
        if matches!(url.scheme(), "blob" | "filesystem") {
            let Ok(inner) = url::Url::parse(url.path()) else {
                return reference.map_or_else(
                    || Self::CreateUniqueOpaque(registry),
                    |origin| origin.DeriveNewOpaqueOrigin(),
                );
            };
            if matches!(inner.scheme(), "blob" | "filesystem") {
                return reference.map_or_else(
                    || Self::CreateUniqueOpaque(registry),
                    |origin| origin.DeriveNewOpaqueOrigin(),
                );
            }
            url = inner;
        }
        if registry.data.no_access.contains(url.scheme())
            || (!registry.data.standard.contains(url.scheme())
                && !registry.data.local.contains(url.scheme()))
        {
            return reference.map_or_else(
                || Self::CreateUniqueOpaque(registry),
                |origin| origin.DeriveNewOpaqueOrigin(),
            );
        }
        let tuple = SecurityOriginTuple {
            scheme: url.scheme().into(),
            host: url.host_str().unwrap_or("").into(),
            port: url.port_or_known_default().unwrap_or(0),
        };
        let can_load_local_resources = registry.data.local.contains(&tuple.scheme);
        Rc::new(Self {
            identity: OriginIdentity::Tuple(tuple),
            precursor: None,
            opaque_potentially_trustworthy: Cell::new(false),
            registry,
            block_local_access_from_local_origin: Cell::new(false),
            universal_access: Cell::new(false),
            can_load_local_resources: Cell::new(can_load_local_resources),
            cross_agent_cluster_access: Cell::new(false),
            domain_override: RefCell::new(None),
            agent_cluster: Cell::new(None),
        })
    }
    /// Immutable opaque identity at a request boundary, never a URL-derived origin.
    pub fn OpaqueNonce(&self) -> Option<u64> {
        match self.identity {
            OriginIdentity::Opaque(nonce) => Some(nonce),
            _ => None,
        }
    }
    pub fn IsOpaque(&self) -> bool {
        matches!(self.identity, OriginIdentity::Opaque(_))
    }
    pub fn Tuple(&self) -> Option<&SecurityOriginTuple> {
        match &self.identity {
            OriginIdentity::Tuple(tuple) => Some(tuple),
            OriginIdentity::Opaque(_) => None,
        }
    }
    /// Source security_origin.cc:511-513,820-827.
    pub fn BlockLocalAccessFromLocalOrigin(&self) {
        assert!(self.IsLocal());
        self.block_local_access_from_local_origin.set(true);
    }
    pub fn SerializesAsNull(&self) -> bool {
        self.IsOpaque() || (self.IsLocal() && self.block_local_access_from_local_origin.get())
    }
    /// Source security_origin.cc:516-518.
    pub fn IsLocal(&self) -> bool {
        self.Tuple()
            .is_some_and(|tuple| self.registry.data.local.contains(&tuple.scheme))
    }
    /// Source security_origin.cc:527-531,545-565; known_ports.cc:65-75.
    /// Serialize the original canonical origin, independently of document URL.
    /// url::IsStandard, as used by history_util.cc:81-90. The source's
    /// filesystem URL type is standard even though its origin is inner-URL based.
    pub fn IsStandardURLScheme(&self, scheme: &str) -> bool {
        scheme == "filesystem" || self.registry.data.standard.contains(scheme)
    }
    pub fn ToString(&self) -> String {
        if self.SerializesAsNull() {
            return "null".into();
        }
        let tuple = self.Tuple().expect("nonopaque origin tuple");
        if tuple.scheme == "file" {
            return "file://".into();
        }
        let mut value = format!("{}://{}", tuple.scheme, tuple.host);
        let default_port = match tuple.scheme.as_str() {
            "http" | "ws" => 80,
            "https" | "wss" => 443,
            "ftp" => 21,
            _ => 0,
        };
        if default_port != 0 && tuple.port != default_port {
            value.push(':');
            value.push_str(&tuple.port.to_string());
        }
        value
    }
    /// security_origin.cc:592-616: same-origin excludes document.domain and
    /// universal access, retaining the original local-file access flags.
    pub fn IsSameOriginWith(&self, other: &Self) -> bool {
        if std::ptr::eq(self, other) {
            return true;
        }
        if self.identity != other.identity {
            return false;
        }
        !self.IsLocal()
            || (!self.block_local_access_from_local_origin.get()
                && !other.block_local_access_from_local_origin.get())
    }
    pub fn CanAccess(&self, other: &Self) -> bool {
        std::ptr::eq(self, other) || self.AccessFacts().CanAccess(&other.AccessFacts())
    }
    pub fn IsLocalhost(&self) -> bool {
        let Some(tuple) = self.Tuple() else {
            return false;
        };
        let host = tuple.host.trim_start_matches('[').trim_end_matches(']');
        if let Ok(ip) = host.parse::<IpAddr>() {
            return match ip {
                IpAddr::V4(ip) => ip.octets()[0] == 127,
                IpAddr::V6(ip) => ip == Ipv6Addr::LOCALHOST,
            };
        }
        let host = host.strip_suffix('.').unwrap_or(host);
        host.eq_ignore_ascii_case("localhost") || host.to_ascii_lowercase().ends_with(".localhost")
    }
    pub fn IsPotentiallyTrustworthy(&self) -> bool {
        let Some(tuple) = self.Tuple() else {
            return self.opaque_potentially_trustworthy.get();
        };
        matches!(tuple.scheme.as_str(), "https" | "wss")
            || self.IsLocalhost()
            || tuple.scheme == "file"
            || self.registry.data.secure.contains(&tuple.scheme)
            || self.registry.data.local.contains(&tuple.scheme)
            || self.registry.data.allowlisted.contains(tuple)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SecureContextMode {
    Insecure,
    Secure,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SecureContextExplanation {
    InsecureScheme,
    InsecureAncestor,
    SecureLocalhost,
    Secure,
}
pub struct SecurityContext {
    origin: Rc<SecurityOrigin>,
    mode: SecureContextMode,
    explanation: SecureContextExplanation,
    address_space: foundation::network_address_space::IPAddressSpace,
    sandbox_flags: foundation::security::SandboxFlags,
    public_urls: Rc<crate::public_url_manager::PublicURLManager>,
    has_insecure_ancestor: bool,
    can_navigate_top_without_user_gesture: Cell<bool>,
    network_permissions_policy: Rc<crate::network_permissions_policy::NetworkPermissionsPolicy>,
    document_policy: RefCell<Option<Rc<crate::document_policy::DocumentPolicy>>>,
    report_only_document_policy: RefCell<Option<Rc<crate::document_policy::DocumentPolicy>>>,
}
impl SecurityContext {
    /// WebLocalFrameImpl::CreateChildFrame cc:2482-2517 and
    /// SecurityContextInit's initial-empty inheritance. Native Frame creation
    /// supplies the actual owner policy; retain the parent's original origin.
    fn ForInitialChild(
        parent: &Self,
        policy: &crate::frame_owner_loading::FrameOwnerPolicy,
    ) -> Self {
        let sandbox = foundation::security::SandboxFlags(parent.sandbox_flags.0 | policy.sandbox.0);
        let origin = if sandbox.Contains(foundation::security::SandboxFlags::ORIGIN) {
            let origin = parent.origin.DeriveNewOpaqueOrigin();
            if parent.origin.IsPotentiallyTrustworthy() {
                origin.SetOpaqueOriginIsPotentiallyTrustworthy(true);
            }
            if parent.origin.can_load_local_resources.get() {
                origin.GrantLoadLocalResources();
            }
            origin
        } else {
            parent.origin.clone()
        };
        let mut result = Self::ForNavigationInStoragePartition(
            origin.clone(),
            parent.has_insecure_ancestor || !parent.IsSecureContext(),
            parent.public_urls.Registry(),
        );
        result.public_urls = parent.public_urls.CopyForNavigation(origin.clone());
        result.address_space = parent.address_space;
        result.sandbox_flags = sandbox;
        result
            .can_navigate_top_without_user_gesture
            .set(parent.can_navigate_top_without_user_gesture.get());
        result.network_permissions_policy = parent
            .network_permissions_policy
            .CreateForChild(&policy.container, origin.Key());
        result
    }
    pub fn NewInitial(registry: Rc<SecurityOriginSchemeRegistry>) -> Self {
        let origin = SecurityOrigin::CreateUniqueOpaque(registry);
        let network_policy =
            Rc::new(crate::network_permissions_policy::NetworkPermissionsPolicy::new(origin.Key()));
        Self {
            public_urls: crate::public_url_manager::PublicURLManager::new(
                origin.clone(),
                foundation::security::StorageKey::CreateFirstParty(origin.Key()),
                std::sync::Arc::new(fileapi::blob_url_registry::BlobURLRegistry::default()),
            ),
            origin,
            mode: SecureContextMode::Insecure,
            explanation: SecureContextExplanation::InsecureScheme,
            address_space: foundation::network_address_space::IPAddressSpace::Unknown,
            sandbox_flags: foundation::security::SandboxFlags::default(),
            has_insecure_ancestor: false,
            can_navigate_top_without_user_gesture: Cell::new(true),
            network_permissions_policy: network_policy,
            document_policy: Default::default(),
            report_only_document_policy: Default::default(),
        }
    }
    /// New loader/Window construction, not history's URL mutation. The
    /// independent outermost Page has no insecure ancestor; subframe admission
    /// must carry its actual ancestor result across its immutable boundary.
    pub fn ForNavigation(origin: Rc<SecurityOrigin>, has_insecure_ancestor: bool) -> Self {
        Self::ForNavigationInStoragePartition(
            origin,
            has_insecure_ancestor,
            std::sync::Arc::new(fileapi::blob_url_registry::BlobURLRegistry::default()),
        )
    }
    pub fn ForNavigationInStoragePartition(
        origin: Rc<SecurityOrigin>,
        has_insecure_ancestor: bool,
        registry: std::sync::Arc<fileapi::blob_url_registry::BlobURLRegistry>,
    ) -> Self {
        let (mode, explanation) = if !origin.IsPotentiallyTrustworthy() {
            (
                SecureContextMode::Insecure,
                SecureContextExplanation::InsecureScheme,
            )
        } else if origin
            .Tuple()
            .is_some_and(|tuple| origin.registry.data.bypass.contains(&tuple.scheme))
        {
            (SecureContextMode::Secure, SecureContextExplanation::Secure)
        } else if has_insecure_ancestor {
            (
                SecureContextMode::Insecure,
                SecureContextExplanation::InsecureAncestor,
            )
        } else {
            (
                SecureContextMode::Secure,
                if origin.IsLocalhost() {
                    SecureContextExplanation::SecureLocalhost
                } else {
                    SecureContextExplanation::Secure
                },
            )
        };
        let network_policy =
            Rc::new(crate::network_permissions_policy::NetworkPermissionsPolicy::new(origin.Key()));
        Self {
            public_urls: crate::public_url_manager::PublicURLManager::new(
                origin.clone(),
                foundation::security::StorageKey::CreateFirstParty(origin.Key()),
                registry,
            ),
            origin,
            mode,
            explanation,
            address_space: foundation::network_address_space::IPAddressSpace::Unknown,
            sandbox_flags: foundation::security::SandboxFlags::default(),
            has_insecure_ancestor,
            can_navigate_top_without_user_gesture: Cell::new(true),
            network_permissions_policy: network_policy,
            document_policy: Default::default(),
            report_only_document_policy: Default::default(),
        }
    }
    /// PolicyContainerPolicies default (policy_container_host.h:171,
    /// policy_container_host.cc:198). A committed policy updates this original
    /// retained value; it is independent of current URL and sandbox flags.
    pub fn CanNavigateTopWithoutUserGesture(&self) -> bool {
        self.can_navigate_top_without_user_gesture.get()
    }
    pub fn SetCanNavigateTopWithoutUserGesture(&self, allowed: bool) {
        self.can_navigate_top_without_user_gesture.set(allowed);
    }
    pub fn AddressSpace(&self) -> foundation::network_address_space::IPAddressSpace {
        self.address_space
    }
    pub(crate) fn SetAddressSpace(
        &mut self,
        space: foundation::network_address_space::IPAddressSpace,
    ) {
        self.address_space = space;
    }
    pub fn Origin(&self) -> &Rc<SecurityOrigin> {
        &self.origin
    }
    pub fn IsSecureContext(&self) -> bool {
        self.mode == SecureContextMode::Secure
    }
    pub fn Explanation(&self) -> SecureContextExplanation {
        self.explanation
    }
}

impl SecurityOrigin {
    /// Source net::IsUrlPotentiallyTrustworthy via the original scheme registry.
    /// Used while compiling CSP report endpoints, never on a request thread.
    pub fn IsURLPotentiallyTrustworthy(&self, url: &str) -> bool {
        self.registry.IsURLPotentiallyTrustworthy(url)
    }
}

impl SecurityOrigin {
    /// Immutable registry data is shared with policy-container request
    /// snapshots. The original Rc SecurityOrigin remains document-only.
    pub fn SchemeRegistryForRequests(&self) -> SecurityOriginSchemeRegistry {
        self.registry.as_ref().clone()
    }
}
impl SecurityOriginSchemeRegistry {
    /// Source net/base/is_potentially_trustworthy.cc:348-376. URL trust has
    /// about/data exceptions that the opaque SecurityOrigin query does not.
    pub fn IsURLPotentiallyTrustworthy(&self, url: &str) -> bool {
        let Ok(parsed) = url::Url::parse(url) else {
            return false;
        };
        if (parsed.scheme() == "about" && matches!(parsed.path(), "blank" | "srcdoc"))
            || parsed.scheme() == "data"
            || self.data.secure.contains(parsed.scheme())
            || self.data.local.contains(parsed.scheme())
        {
            return true;
        }
        SecurityOrigin::CreateForNavigation(url, None, Rc::new(self.clone()))
            .IsPotentiallyTrustworthy()
    }
}

impl SecurityOrigin {
    /// MixedContentChecker.cc:328-342 checks the original origin or its
    /// flattened precursor for an opaque origin, never the document URL.
    pub fn MixedContentOriginScheme(&self) -> Option<&str> {
        self.Tuple()
            .or_else(|| self.precursor.as_ref().and_then(|origin| origin.Tuple()))
            .map(|tuple| tuple.scheme.as_str())
    }
}

/// LocalDOMWindow::CrossOriginIsolatedCapability, cc:2697-2712, and
/// Agent::IsCrossOriginIsolated, agent.cc:84-94. This belongs to the retained
/// execution context and is initialized at the loader's agent-cluster commit.
/// Rechrom currently admits nonisolated agent clusters only: neither COOP/COEP
/// nor DocumentIsolationPolicy establishes a concrete isolation key.
pub struct ExecutionContextIsolation {
    cross_origin_isolated_capability: bool,
    // ShouldReuseDOMWindow compares concrete AgentCluster COI keys. The
    // non-isolated constructor establishes absence; capability alone does
    // not prove it for a cluster inherited through an incomplete boundary.
    cross_origin_isolation_key_known_absent: bool,
}
impl ExecutionContextIsolation {
    pub(crate) fn ForNonIsolatedAgentCluster() -> Self {
        Self {
            cross_origin_isolated_capability: false,
            cross_origin_isolation_key_known_absent: true,
        }
    }
    pub(crate) fn ForInheritedAgentCluster(capability: bool) -> Self {
        Self {
            cross_origin_isolated_capability: capability,
            cross_origin_isolation_key_known_absent: false,
        }
    }
    pub fn CrossOriginIsolatedCapability(&self) -> bool {
        self.cross_origin_isolated_capability
    }
    pub fn CrossOriginIsolationKeyKnownAbsent(&self) -> bool {
        self.cross_origin_isolation_key_known_absent
    }
}

impl SecurityOrigin {
    /// SecurityOrigin::ToUrlOrigin, cc:295-309: pure immutable boundary identity.
    pub fn Key(&self) -> foundation::security::OriginKey {
        match &self.identity {
            OriginIdentity::Tuple(t) => foundation::security::OriginKey::Tuple {
                scheme: t.scheme.clone(),
                host: t.host.clone(),
                port: t.port,
            },
            OriginIdentity::Opaque(n) => foundation::security::OriginKey::Opaque(*n),
        }
    }
    /// Only the *destination* is parsed, as SecurityOrigin::CanRequest does at
    /// cc:418-432. The current Document origin always comes from this owner.
    pub fn URLKey(&self, url: &str) -> Option<foundation::security::OriginKey> {
        let target = SecurityOrigin::CreateForNavigation(url, None, self.registry.clone());
        (!target.IsOpaque()).then(|| target.Key())
    }
    pub fn CanRequest(
        &self,
        url: &str,
        registered_blob_origin: Option<&foundation::security::OriginKey>,
    ) -> bool {
        if self.universal_access.get() {
            return true;
        }
        if self.SerializesAsNull() {
            return url.starts_with("blob:null/") && registered_blob_origin == Some(&self.Key());
        }
        self.URLKey(url).is_some_and(|target| {
            target == self.Key()
                || self
                    .registry
                    .access_list
                    .CheckAccessState(&self.Key(), &target)
                    == foundation::origin_access_list::AccessState::Allowed
        })
    }
    /// SecurityOrigin.cc:448-473. Preserve display isolation and local-resource
    /// privilege independently of same-origin request/read permission.
    pub fn CanDisplay(
        &self,
        url: &url::Url,
        registered_blob_origin: Option<&foundation::security::OriginKey>,
    ) -> bool {
        if self.universal_access.get() || url.scheme() == "data" {
            return true;
        }
        if matches!(url.scheme(), "blob" | "filesystem") {
            return self.CanRequest(url.as_str(), registered_blob_origin);
        }
        let access_allowed = || {
            self.URLKey(url.as_str()).is_some_and(|target| {
                self.registry
                    .access_list
                    .CheckAccessState(&self.Key(), &target)
                    == foundation::origin_access_list::AccessState::Allowed
            })
        };
        if self.registry.data.display_isolated.contains(url.scheme()) {
            return self
                .Tuple()
                .is_some_and(|origin| origin.scheme == url.scheme())
                || access_allowed();
        }
        if self.registry.data.local.contains(url.scheme()) {
            return self.can_load_local_resources.get() || access_allowed();
        }
        true
    }
    pub fn CanReadContent(
        &self,
        url: &str,
        registered_blob_origin: Option<&foundation::security::OriginKey>,
    ) -> bool {
        self.CanRequest(url, registered_blob_origin)
            || url::Url::parse(url).is_ok_and(|url| url.scheme() == "data")
    }
}
impl SecurityContext {
    pub fn SandboxFlags(&self) -> foundation::security::SandboxFlags {
        self.sandbox_flags
    }
    pub fn IsSandboxed(&self, mask: foundation::security::SandboxFlags) -> bool {
        self.sandbox_flags.Contains(mask)
    }
    pub fn SetSandboxFlags(&mut self, flags: foundation::security::SandboxFlags) {
        self.sandbox_flags = flags;
    }
    pub fn PublicURLManager(&self) -> &Rc<crate::public_url_manager::PublicURLManager> {
        &self.public_urls
    }
    pub fn CanReadContent(&self, url: &str) -> bool {
        self.origin
            .CanReadContent(url, self.public_urls.OriginForURL(url).as_ref())
    }
    pub fn CanDisplay(&self, url: &url::Url) -> bool {
        self.origin
            .CanDisplay(url, self.public_urls.OriginForURL(url.as_str()).as_ref())
    }
    pub fn CanRequest(&self, url: &str) -> bool {
        self.origin
            .CanRequest(url, self.public_urls.OriginForURL(url).as_ref())
    }
}

impl SecurityOrigin {
    // security_origin.cc:503-509; source constructor defaults .h:484,489.
    // SecurityOrigin.cc:498-501.
    pub fn GrantLoadLocalResources(&self) {
        self.can_load_local_resources.set(true);
    }
    pub fn GrantUniversalAccess(&self) {
        self.universal_access.set(true);
    }
    pub fn GrantCrossAgentClusterAccess(&self) {
        self.cross_agent_cluster_access.set(true);
    }
    pub fn IsGrantedCrossAgentClusterAccess(&self) -> bool {
        self.cross_agent_cluster_access.get()
    }
}

impl SecurityOrigin {
    pub fn AccessFacts(&self) -> foundation::security::OriginAccess {
        foundation::security::OriginAccess {
            origin: self.Key(),
            domain_override: self.domain_override.borrow().clone(),
            is_local: self.IsLocal(),
            blocks_local_access: self.block_local_access_from_local_origin.get(),
            universal_access: self.universal_access.get(),
            cross_agent_cluster_access: self.cross_agent_cluster_access.get(),
            agent_cluster: self.agent_cluster.get(),
        }
    }
}
impl SecurityContext {
    pub fn DestinationOriginAccess(&self, url: &str) -> Option<foundation::security::OriginAccess> {
        if let Some(origin) = self.public_urls.OriginForURL(url) {
            return Some(foundation::security::OriginAccess {
                origin,
                domain_override: None,
                is_local: false,
                blocks_local_access: false,
                universal_access: false,
                cross_agent_cluster_access: false,
                agent_cluster: None,
            });
        }
        let origin = SecurityOrigin::CreateForNavigation(url, None, self.origin.registry.clone());
        (!origin.IsOpaque()).then(|| origin.AccessFacts())
    }
}

impl SecurityOrigin {
    /// Source SetDomainFromDOM security_origin.cc:499-501; callers own the
    /// Document.domain admission checks before changing this original owner.
    pub fn SetDomainFromDOM(&self, domain: String) {
        *self.domain_override.borrow_mut() = Some(domain);
    }
    pub fn SetAgentClusterId(&self, id: [u64; 2]) {
        assert_ne!(id, [0, 0]);
        self.agent_cluster.set(Some(id));
    }
}

impl SecurityContext {
    /// PublicURLManager is an ExecutionContext supplement. ClearForReuse
    /// retains that same Window owner, including outstanding registrations.
    pub(crate) fn ReuseWindowPublicURLManager(&mut self, previous: &SecurityContext) {
        self.public_urls = previous.public_urls.clone();
    }
    pub(crate) fn InstallBlobURLRegistry(
        &mut self,
        registry: std::sync::Arc<fileapi::blob_url_registry::BlobURLRegistry>,
    ) {
        assert!(self.public_urls.ReadyForStoragePartitionCommit());
        let key = self.public_urls.StorageKey().clone();
        self.public_urls =
            crate::public_url_manager::PublicURLManager::new(self.origin.clone(), key, registry);
    }
}

impl SecurityOrigin {
    /// security_origin.cc:749-753; DocumentLoader::CalculateOrigin cc:2629-2634.
    pub fn SetOpaqueOriginIsPotentiallyTrustworthy(&self, value: bool) {
        assert!(self.IsOpaque());
        self.opaque_potentially_trustworthy.set(value);
    }
}
impl SecurityContext {
    pub(crate) fn HasInsecureAncestor(&self) -> bool {
        self.has_insecure_ancestor
    }
}

impl SecurityOrigin {
    pub fn Domain(&self) -> String {
        self.domain_override.borrow().clone().unwrap_or_else(|| {
            self.Tuple()
                .map_or_else(String::new, |tuple| tuple.host.clone())
        })
    }
}

impl SecurityOriginSchemeRegistry {
    pub fn OriginAccessList(
        &self,
    ) -> &std::sync::Arc<foundation::origin_access_list::OriginAccessList> {
        &self.access_list
    }
}
impl SecurityOrigin {
    pub fn ProtectedOriginForRequest(
        &self,
    ) -> foundation::content_security_policy::CSPProtectedOrigin {
        use foundation::content_security_policy::CSPProtectedOrigin;
        let potentially_trustworthy = self.IsPotentiallyTrustworthy();
        match self.Tuple() {
            Some(tuple) => CSPProtectedOrigin::Tuple {
                scheme: tuple.scheme.clone(),
                host: tuple.host.clone(),
                port: tuple.port,
                potentially_trustworthy,
            },
            None => CSPProtectedOrigin::Opaque {
                potentially_trustworthy,
                precursor_scheme: self.MixedContentOriginScheme().map(str::to_owned),
            },
        }
    }
}

impl SecurityContext {
    pub fn NetworkPermissionsPolicy(
        &self,
    ) -> &Rc<crate::network_permissions_policy::NetworkPermissionsPolicy> {
        &self.network_permissions_policy
    }
}

impl SecurityContext {
    pub fn DocumentPolicy(&self) -> Option<Rc<crate::document_policy::DocumentPolicy>> {
        self.document_policy.borrow().clone()
    }
    pub fn ReportOnlyDocumentPolicy(&self) -> Option<Rc<crate::document_policy::DocumentPolicy>> {
        self.report_only_document_policy.borrow().clone()
    }
    pub fn SetDocumentPolicy(&self, policy: Rc<crate::document_policy::DocumentPolicy>) {
        *self.document_policy.borrow_mut() = Some(policy);
    }
    pub fn SetReportOnlyDocumentPolicy(
        &self,
        policy: Option<Rc<crate::document_policy::DocumentPolicy>>,
    ) {
        *self.report_only_document_policy.borrow_mut() = policy;
    }
}

impl SecurityOrigin {
    /// security_origin.cc188-205,342-344: copy mutable domain/access bits,
    /// preserve an opaque nonce, and recursively isolate the precursor.
    pub fn IsolatedCopy(&self) -> Rc<Self> {
        let mut copy = self.clone();
        copy.precursor = self.precursor.as_ref().map(|origin| origin.IsolatedCopy());
        Rc::new(copy)
    }
}
/// Original Window security inputs held through its unload dispatch. In
/// particular the origin remains live until InitializeWindow isolates it.
pub struct JavaScriptURLSecuritySnapshot {
    origin: Rc<SecurityOrigin>,
    mode: SecureContextMode,
    explanation: SecureContextExplanation,
    address_space: foundation::network_address_space::IPAddressSpace,
    sandbox_flags: foundation::security::SandboxFlags,
    public_urls: Rc<crate::public_url_manager::PublicURLManager>,
    has_insecure_ancestor: bool,
    can_navigate_top_without_user_gesture: bool,
    permissions_policy: Rc<crate::network_permissions_policy::NetworkPermissionsPolicy>,
    document_policy: Option<Rc<crate::document_policy::DocumentPolicy>>,
    report_only_document_policy: Option<Rc<crate::document_policy::DocumentPolicy>>,
}
impl SecurityContext {
    pub fn CaptureForJavaScriptURLCommit(&self) -> JavaScriptURLSecuritySnapshot {
        JavaScriptURLSecuritySnapshot {
            origin: self.origin.clone(),
            mode: self.mode,
            explanation: self.explanation,
            address_space: self.address_space,
            sandbox_flags: self.sandbox_flags,
            public_urls: self.public_urls.clone(),
            has_insecure_ancestor: self.has_insecure_ancestor,
            can_navigate_top_without_user_gesture: self.can_navigate_top_without_user_gesture.get(),
            permissions_policy: self.network_permissions_policy.clone(),
            document_policy: self.DocumentPolicy(),
            report_only_document_policy: self.ReportOnlyDocumentPolicy(),
        }
    }
}
impl crate::persistent_document::PersistentDocument {
    /// DocumentInit/InstallNewDocument for FrameLoader::Init. The child owns a
    /// new Document lifetime and shares the parent's immutable compiled CSP.
    /// DocumentLoader::CalculateOrigin/InitializeWindow for a container-initiated
    /// local-frame navigation. The Window receives policy from its native Frame
    /// owner and the live parent execution context before parser entry.
    pub fn InitializeLocalFrameNavigationDocument(
        &mut self,
        parent: &Self,
        policy: &crate::frame_owner_loading::FrameOwnerPolicy,
        url: &str,
    ) {
        let inherited = url.is_empty() || url.starts_with("about:");
        self.security_context = if inherited {
            SecurityContext::ForInitialChild(parent.SecurityContext(), policy)
        } else {
            let parent_context = parent.SecurityContext();
            let sandbox = foundation::security::SandboxFlags(
                parent_context.SandboxFlags().0 | policy.sandbox.0,
            );
            let mut origin = SecurityOrigin::CreateForNavigation(
                url,
                None,
                parent_context.Origin().registry.clone(),
            );
            if sandbox.Contains(foundation::security::SandboxFlags::ORIGIN) {
                let trustworthy = origin.IsPotentiallyTrustworthy();
                origin = origin.DeriveNewOpaqueOrigin();
                if trustworthy {
                    origin.SetOpaqueOriginIsPotentiallyTrustworthy(true);
                }
            }
            let mut context = SecurityContext::ForNavigationInStoragePartition(
                origin.clone(),
                parent_context.HasInsecureAncestor() || !parent_context.IsSecureContext(),
                parent_context.PublicURLManager().Registry(),
            );
            context.SetSandboxFlags(sandbox);
            context.network_permissions_policy = parent_context
                .network_permissions_policy
                .CreateForChild(&policy.container, origin.Key());
            context
        };
        if inherited {
            self.content_security_policy =
                parent.ContentSecurityPolicy().CopyPoliciesForNavigation();
            self.SetBaseURLOverride(parent.BaseURL().AsStr());
        }
        self.content_security_policy
            .BindSecurityOrigin(self.security_context.Origin());
        self.execution_context_isolation = if inherited {
            ExecutionContextIsolation::ForInheritedAgentCluster(
                parent
                    .ExecutionContextIsolation()
                    .CrossOriginIsolatedCapability(),
            )
        } else {
            ExecutionContextIsolation::ForNonIsolatedAgentCluster()
        };
        self.navigation_security_initialized = true;
        self.SetDocumentURL(url.to_owned());
        self.SetRequiredDocumentPolicy(policy.required_document.clone());
        self.SetIsInitialEmptyDocument(false);
        self.SetScriptingEnabled(parent.ScriptingEnabled());
        self.BindDOMWindow();
        self.SetLifecycleActive(true);
    }
    pub fn InitializeInitialLocalFrameDocument(
        &mut self,
        parent: &Self,
        policy: &crate::frame_owner_loading::FrameOwnerPolicy,
    ) {
        assert!(
            !self.IsActive(),
            "initial Frame document is not attached yet"
        );
        self.security_context = SecurityContext::ForInitialChild(parent.SecurityContext(), policy);
        self.content_security_policy = parent.ContentSecurityPolicy().CopyPoliciesForNavigation();
        self.content_security_policy
            .BindSecurityOrigin(self.security_context.Origin());
        self.execution_context_isolation = ExecutionContextIsolation::ForInheritedAgentCluster(
            parent
                .ExecutionContextIsolation()
                .CrossOriginIsolatedCapability(),
        );
        self.navigation_security_initialized = true;
        self.SetDocumentURL(String::new());
        self.SetBaseURLOverride(parent.BaseURL().AsStr());
        self.SetRequiredDocumentPolicy(policy.required_document.clone());
        self.SetIsInitialEmptyDocument(true);
        self.SetScriptingEnabled(parent.ScriptingEnabled());
        self.BindDOMWindow();
        self.SetLifecycleActive(true);
    }
}
impl JavaScriptURLSecuritySnapshot {
    /// DocumentLoader.cc2790-2807,3144-3172; SecurityContextInit.cc323-338.
    pub(crate) fn CreateContext(&self) -> SecurityContext {
        let origin = self.origin.IsolatedCopy();
        SecurityContext {
            public_urls: self.public_urls.CopyForNavigation(origin.clone()),
            origin,
            mode: self.mode,
            explanation: self.explanation,
            address_space: self.address_space,
            sandbox_flags: self.sandbox_flags,
            has_insecure_ancestor: self.has_insecure_ancestor,
            can_navigate_top_without_user_gesture: Cell::new(
                self.can_navigate_top_without_user_gesture,
            ),
            network_permissions_policy: self.permissions_policy.CopyStateFrom(),
            document_policy: RefCell::new(
                self.document_policy
                    .as_ref()
                    .map(|policy| policy.CopyStateFrom()),
            ),
            report_only_document_policy: RefCell::new(
                self.report_only_document_policy
                    .as_ref()
                    .map(|policy| policy.CopyStateFrom()),
            ),
        }
    }
}

impl foundation::security::SecurityOriginOwner for SecurityOrigin {
    fn SecurityOriginAccess(&self) -> foundation::security::OriginAccess {
        self.AccessFacts()
    }
    fn SecurityOriginIdentity(&self) -> usize {
        self as *const Self as usize
    }
    fn SecurityOriginFacts(&self) -> foundation::security::SameOriginFacts {
        foundation::security::SameOriginFacts {
            origin: self.Key(),
            is_local: self.IsLocal(),
            blocks_local_access: self.block_local_access_from_local_origin.get(),
        }
    }
}
