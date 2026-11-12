//! Chromium 6c1d401f: net/base/schemeful_site.cc:26-152;
//! registry_controlled_domains/registry_controlled_domain.cc:181-335;
//! blink/common/storage_key/storage_key.cc:512-542, storage_key.h:217-230,308-314.
//! Immutable boundary values, never a second SecurityContext/Origin owner.
use std::{collections::HashSet, net::IpAddr, sync::OnceLock};
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum OriginKey {
    Tuple {
        scheme: String,
        host: String,
        port: u16,
    },
    Opaque(u64),
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SchemefulSite {
    Tuple { scheme: String, host: String },
    Opaque(u64),
}
impl SchemefulSite {
    pub fn FromOrigin(origin: &OriginKey) -> Self {
        match origin {
            OriginKey::Opaque(nonce) => Self::Opaque(*nonce),
            OriginKey::Tuple { scheme, host, .. } => {
                let registrable =
                    if matches!(scheme.as_str(), "http" | "https" | "ws" | "wss" | "ftp") {
                        DomainAndRegistry(host)
                    } else {
                        None
                    };
                Self::Tuple {
                    scheme: scheme.clone(),
                    host: registrable.unwrap_or(host).into(),
                }
            }
        }
    }
}
struct SuffixRules {
    exact: HashSet<String>,
    wildcard: HashSet<String>,
    exception: HashSet<String>,
}
fn rules() -> &'static SuffixRules {
    static RULES: OnceLock<SuffixRules> = OnceLock::new();
    RULES.get_or_init(|| {
        let mut rules = SuffixRules {
            exact: HashSet::new(),
            wildcard: HashSet::new(),
            exception: HashSet::new(),
        };
        // The fixed-revision PSL is canonicalized once, including PRIVATE rules.
        // All lookups subsequently borrow the retained compiled rule sets.
        for line in include_str!("security/effective_tld_names.dat").lines() {
            let rule = line.split("//").next().unwrap_or("").trim();
            if rule.is_empty() {
                continue;
            }
            let (target, name) = if let Some(name) = rule.strip_prefix('!') {
                (&mut rules.exception, name)
            } else if let Some(name) = rule.strip_prefix("*.") {
                (&mut rules.wildcard, name)
            } else {
                (&mut rules.exact, rule)
            };
            let normalized = url::Host::parse(name)
                .expect("fixed Chromium PSL host")
                .to_string();
            target.insert(normalized);
        }
        rules
    })
}
/// INCLUDE_PRIVATE_REGISTRIES + INCLUDE_UNKNOWN_REGISTRIES, on a canonical host.
/// The borrowed result includes the source trailing dot and never allocates.
pub fn DomainAndRegistry(host: &str) -> Option<&str> {
    if host.is_empty() || host.trim_matches(['[', ']']).parse::<IpAddr>().is_ok() {
        return None;
    }
    let trimmed = host
        .trim_start_matches('.')
        .strip_suffix('.')
        .unwrap_or_else(|| host.trim_start_matches('.'));
    if trimmed.is_empty() {
        return None;
    }
    let rules = rules();
    let starts = std::iter::once(0)
        .chain(trimmed.match_indices('.').map(|(i, _)| i + 1))
        .collect::<Vec<_>>();
    let mut registry_labels = 1;
    let labels = starts.len();
    for (i, &start) in starts.iter().enumerate() {
        let suffix = &trimmed[start..];
        if rules.exception.contains(suffix) {
            registry_labels = labels - i - 1;
            break;
        }
        if rules.exact.contains(suffix) {
            registry_labels = registry_labels.max(labels - i);
        }
        if rules.wildcard.contains(suffix) {
            registry_labels = registry_labels.max(labels - i + usize::from(i > 0));
        }
    }
    if labels <= registry_labels {
        return None;
    }
    let start = host.len() - trimmed.len() - usize::from(host.ends_with('.'))
        + starts[labels - registry_labels - 1];
    Some(&host[start..])
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AncestorChainBit {
    SameSite,
    CrossSite,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StorageKey {
    origin: OriginKey,
    top_level_site: SchemefulSite,
    nonce: Option<[u64; 2]>,
    ancestor_chain: AncestorChainBit,
}
impl StorageKey {
    pub fn CreateFirstParty(origin: OriginKey) -> Self {
        let ancestor_chain = if matches!(origin, OriginKey::Opaque(_)) {
            AncestorChainBit::CrossSite
        } else {
            AncestorChainBit::SameSite
        };
        Self {
            top_level_site: SchemefulSite::FromOrigin(&origin),
            origin,
            nonce: None,
            ancestor_chain,
        }
    }
    pub fn Create(
        origin: OriginKey,
        top_level_site: SchemefulSite,
        ancestor_chain: AncestorChainBit,
        partitioning: bool,
    ) -> Self {
        assert!(
            ancestor_chain == AncestorChainBit::CrossSite
                || (!matches!(origin, OriginKey::Opaque(_))
                    && SchemefulSite::FromOrigin(&origin) == top_level_site)
        );
        if !partitioning {
            return Self::CreateFirstParty(origin);
        }
        Self {
            origin,
            top_level_site,
            ancestor_chain,
            nonce: None,
        }
    }
    pub fn CreateWithNonce(origin: OriginKey, nonce: [u64; 2]) -> Self {
        assert_ne!(nonce, [0, 0]);
        Self {
            top_level_site: SchemefulSite::FromOrigin(&origin),
            origin,
            nonce: Some(nonce),
            ancestor_chain: AncestorChainBit::CrossSite,
        }
    }
    pub fn Origin(&self) -> &OriginKey {
        &self.origin
    }
    pub fn TopLevelSite(&self) -> &SchemefulSite {
        &self.top_level_site
    }
    pub fn IsFirstPartyContext(&self) -> bool {
        self.ancestor_chain == AncestorChainBit::SameSite
    }
}
/// security_context.cc:160-168; services/network/public/mojom/web_sandbox_flags.mojom:20-54.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SandboxFlags(pub u32);
impl SandboxFlags {
    pub const NAVIGATION: Self = Self(1);
    pub const ORIGIN: Self = Self(4);
    pub const FORMS: Self = Self(8);
    pub const SCRIPTS: Self = Self(16);
    pub const TOP_NAVIGATION: Self = Self(32);
    pub const POPUPS: Self = Self(64);
    pub const PROPAGATES_TO_AUXILIARY: Self = Self(2048);
    pub const TOP_NAVIGATION_BY_USER_ACTIVATION: Self = Self(16384);
    pub const DOWNLOADS: Self = Self(32768);
    pub fn Contains(self, mask: Self) -> bool {
        self.0 & mask.0 != 0
    }
}

/// services/network/public/cpp/web_sandbox_flags.cc:24-98. Flags denote
/// restrictions, so every recognized allow-token removes its source mask.
pub fn ParseSandboxPolicy(input: &str) -> (SandboxFlags, Vec<String>) {
    let mut flags = u32::MAX;
    let mut errors = Vec::new();
    for token in input
        .split(|c| matches!(c, ' ' | '\n' | '\t' | '\r' | '\x0c'))
        .filter(|s| !s.is_empty())
    {
        let mask = match token.to_ascii_lowercase().as_str() {
            "allow-downloads" => 32768,
            "allow-forms" => 8,
            "allow-modals" => 4096,
            "allow-orientation-lock" => 1024,
            "allow-pointer-lock" => 256,
            "allow-popups" => 64 | 131072,
            "allow-popups-to-escape-sandbox" => 2048,
            "allow-presentation" => 8192,
            "allow-same-origin" => 4,
            "allow-scripts" => 128 | 16,
            "allow-storage-access-by-user-activation" => 65536,
            "allow-top-navigation" => 32 | 131072,
            "allow-top-navigation-by-user-activation" => 16384,
            "allow-top-navigation-to-custom-protocols" => 131072,
            "allow-same-site-none-cookies" => 262144,
            _ => 0,
        };
        flags &= !mask;
        if mask == 0 && !errors.iter().any(|old| old == token) {
            errors.push(token.to_owned());
        }
    }
    (SandboxFlags(flags), errors)
}
/// SecurityOrigin access state carried by an opaque browsing-context proxy.
/// It is produced by that context's canonical owner, never reconstructed from
/// its URL. A lease generation update replaces the immutable proxy facts.
#[derive(Clone, Debug)]
pub struct OriginAccess {
    pub origin: OriginKey,
    pub domain_override: Option<String>,
    pub is_local: bool,
    pub blocks_local_access: bool,
    pub universal_access: bool,
    pub cross_agent_cluster_access: bool,
    pub agent_cluster: Option<[u64; 2]>,
}
impl OriginAccess {
    /// security_origin.cc:369-392,646-712. The `this==other` shortcut is
    /// handled by the owning caller before crossing this value boundary.
    pub fn CanAccess(&self, other: &Self) -> bool {
        if self.universal_access {
            return true;
        }
        let same = match (&self.origin, &other.origin) {
            (OriginKey::Opaque(a), OriginKey::Opaque(b)) => a == b,
            (
                OriginKey::Tuple {
                    scheme: a,
                    host: ah,
                    port: ap,
                },
                OriginKey::Tuple {
                    scheme: b,
                    host: bh,
                    port: bp,
                },
            ) => {
                a == b
                    && match (&self.domain_override, &other.domain_override) {
                        (None, None) => ah == bh && ap == bp,
                        (Some(a), Some(b)) => a == b,
                        _ => false,
                    }
                    && (!self.is_local || !self.blocks_local_access && !other.blocks_local_access)
            }
            _ => false,
        };
        same && (self.cross_agent_cluster_access
            || self.agent_cluster.is_none()
            || other.agent_cluster.is_none()
            || self.agent_cluster == other.agent_cluster)
    }
}
impl OriginAccess {
    /// SecurityOrigin::ToString / BuildRawString cc:527-565, for opaque proxy
    /// diagnostics. The original identity is serialized, never reparsed.
    pub fn Serialize(&self) -> String {
        match &self.origin {
            OriginKey::Opaque(_) => "null".into(),
            OriginKey::Tuple { scheme, host, port } => {
                if self.is_local && self.blocks_local_access {
                    return "null".into();
                }
                if scheme == "file" {
                    return "file://".into();
                }
                let default = match scheme.as_str() {
                    "http" | "ws" => 80,
                    "https" | "wss" => 443,
                    "ftp" => 21,
                    _ => 0,
                };
                let mut value = format!("{}://{}", scheme, host);
                if *port != 0 && *port != default {
                    value.push(':');
                    value.push_str(&port.to_string());
                }
                value
            }
        }
    }
}

/// Retained original SecurityOrigin at a synchronous package boundary. Facts
/// are read from that owner, including file-access mutations and opaque nonce.
/// SecurityOrigin.cc:592-616; identity preserves its this==other shortcut.
pub struct SameOriginFacts {
    pub origin: OriginKey,
    pub is_local: bool,
    pub blocks_local_access: bool,
}
pub trait SecurityOriginOwner {
    fn SecurityOriginIdentity(&self) -> usize;
    fn SecurityOriginFacts(&self) -> SameOriginFacts;
    fn SecurityOriginAccess(&self) -> OriginAccess;
}
pub fn IsSameSecurityOrigin(a: &dyn SecurityOriginOwner, b: &dyn SecurityOriginOwner) -> bool {
    if a.SecurityOriginIdentity() == b.SecurityOriginIdentity() {
        return true;
    }
    let a = a.SecurityOriginFacts();
    let b = b.SecurityOriginFacts();
    a.origin == b.origin && (!a.is_local || (!a.blocks_local_access && !b.blocks_local_access))
}

/// SecurityOrigin::CanAccess's original shared-owner pointer shortcut.
pub fn CanAccessSecurityOrigin(a: &dyn SecurityOriginOwner, b: &dyn SecurityOriginOwner) -> bool {
    a.SecurityOriginIdentity() == b.SecurityOriginIdentity()
        || a.SecurityOriginAccess()
            .CanAccess(&b.SecurityOriginAccess())
}
