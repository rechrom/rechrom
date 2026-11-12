//! Chromium services/network/public/cpp/cors/origin_access_entry.{h:38-99,cc:35-159}
//! and origin_access_list.{h:33-145,cc:25-94,143-207}.
use crate::security::{DomainAndRegistry, OriginKey};
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{Arc, OnceLock, RwLock},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DomainMatchMode {
    AllowSubdomains,
    AllowRegistrableDomains,
    DisallowSubdomains,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortMatchMode {
    AllowAnyPort,
    AllowOnlySpecifiedPort,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MatchPriority {
    NoMatchingOrigin,
    Default,
    Low,
    Medium,
    High,
    Max,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchResult {
    MatchesOrigin,
    MatchesOriginButIsPublicSuffix,
    DoesNotMatchOrigin,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessState {
    Allowed,
    Blocked,
    NotListed,
}
#[derive(Clone, Debug)]
pub struct OriginAccessEntry {
    protocol: String,
    host: String,
    port: u16,
    domain_mode: DomainMatchMode,
    port_mode: PortMatchMode,
    priority: MatchPriority,
    host_is_ip: bool,
    registrable_domain: Option<String>,
    host_is_public_suffix: bool,
}
fn subdomain(domain: &str, host: &str) -> bool {
    domain.len() > host.len()
        && domain.ends_with(host)
        && domain.as_bytes()[domain.len() - host.len() - 1] == b'.'
}
fn registry(host: &str) -> Option<&str> {
    if host.trim_matches(['[', ']']).parse::<IpAddr>().is_ok() {
        return None;
    }
    if let Some(domain) = DomainAndRegistry(host) {
        domain.split_once('.').map(|(_, suffix)| suffix)
    } else {
        None
    }
}
impl OriginAccessEntry {
    pub fn new(
        protocol: String,
        host: String,
        port: u16,
        domain_mode: DomainMatchMode,
        port_mode: PortMatchMode,
        priority: MatchPriority,
    ) -> Self {
        let host_is_ip = host.trim_matches(['[', ']']).parse::<IpAddr>().is_ok();
        let suffix_len = registry(&host).map_or(host.len(), str::len);
        let host_is_public_suffix = !host_is_ip && host.len() <= suffix_len + 1;
        let registrable_domain = (!host_is_ip
            && !host_is_public_suffix
            && domain_mode == DomainMatchMode::AllowRegistrableDomains)
            .then(|| DomainAndRegistry(&host).map(str::to_owned))
            .flatten();
        Self {
            protocol,
            host,
            port,
            domain_mode,
            port_mode,
            priority,
            host_is_ip,
            registrable_domain,
            host_is_public_suffix,
        }
    }
    pub fn MatchesOrigin(&self, origin: &OriginKey) -> MatchResult {
        let (scheme, host, port) = match origin {
            OriginKey::Tuple { scheme, host, port } => (scheme.as_str(), host.as_str(), *port),
            OriginKey::Opaque(_) => ("", "", 0),
        };
        if self.protocol.as_str() != scheme
            || (self.port_mode == PortMatchMode::AllowOnlySpecifiedPort && self.port != port)
        {
            return MatchResult::DoesNotMatchOrigin;
        }
        self.MatchesDomain(host)
    }
    pub fn MatchesDomain(&self, domain: &str) -> MatchResult {
        use MatchResult::*;
        if self.domain_mode != DomainMatchMode::DisallowSubdomains && self.host.is_empty() {
            return MatchesOrigin;
        }
        if self.host == domain {
            return MatchesOrigin;
        }
        if self.host_is_ip {
            return DoesNotMatchOrigin;
        }
        let matches = match self.domain_mode {
            DomainMatchMode::DisallowSubdomains => false,
            DomainMatchMode::AllowSubdomains => subdomain(domain, &self.host),
            DomainMatchMode::AllowRegistrableDomains => {
                self.registrable_domain.as_ref().map_or_else(
                    || subdomain(domain, &self.host),
                    |base| base == domain || subdomain(domain, base),
                )
            }
        };
        if !matches {
            return DoesNotMatchOrigin;
        }
        if self.host_is_public_suffix
            || registry(domain).is_some_and(|suffix| subdomain(suffix, &self.host))
        {
            MatchesOriginButIsPublicSuffix
        } else {
            MatchesOrigin
        }
    }
}
#[derive(Default, Debug)]
struct Patterns {
    allow: Vec<OriginAccessEntry>,
    block: Vec<OriginAccessEntry>,
}
#[derive(Default, Debug)]
pub struct OriginAccessList {
    map: RwLock<HashMap<OriginKey, Patterns>>,
}
impl OriginAccessList {
    pub fn Global() -> Arc<Self> {
        static INSTANCE: OnceLock<Arc<OriginAccessList>> = OnceLock::new();
        INSTANCE.get_or_init(|| Arc::new(Self::default())).clone()
    }
    pub fn SetForOrigin(
        &self,
        source: OriginKey,
        allow: Vec<OriginAccessEntry>,
        block: Vec<OriginAccessEntry>,
    ) {
        assert!(!matches!(source, OriginKey::Opaque(_)));
        let mut map = self.map.write().expect("origin access registry");
        if allow.is_empty() && block.is_empty() {
            map.remove(&source);
        } else {
            map.insert(source, Patterns { allow, block });
        }
    }
    pub fn AddAllowListEntry(&self, source: OriginKey, entry: OriginAccessEntry) {
        assert!(!matches!(source, OriginKey::Opaque(_)));
        self.map
            .write()
            .expect("origin access registry")
            .entry(source)
            .or_default()
            .allow
            .push(entry);
    }
    pub fn AddBlockListEntry(&self, source: OriginKey, entry: OriginAccessEntry) {
        assert!(!matches!(source, OriginKey::Opaque(_)));
        self.map
            .write()
            .expect("origin access registry")
            .entry(source)
            .or_default()
            .block
            .push(entry);
    }
    pub fn ClearForOrigin(&self, source: &OriginKey) {
        self.map
            .write()
            .expect("origin access registry")
            .remove(source);
    }
    pub fn CheckAccessState(&self, source: &OriginKey, target: &OriginKey) -> AccessState {
        if matches!(source, OriginKey::Opaque(_)) {
            return AccessState::Blocked;
        }
        let map = self.map.read().expect("origin access registry");
        let Some(patterns) = map.get(source) else {
            return AccessState::NotListed;
        };
        let priority = |patterns: &Vec<OriginAccessEntry>| {
            patterns
                .iter()
                .filter(|entry| entry.MatchesOrigin(target) != MatchResult::DoesNotMatchOrigin)
                .map(|entry| entry.priority)
                .max()
                .unwrap_or(MatchPriority::NoMatchingOrigin)
        };
        let allow = priority(&patterns.allow);
        let block = priority(&patterns.block);
        if block == MatchPriority::NoMatchingOrigin {
            if allow == MatchPriority::NoMatchingOrigin {
                AccessState::NotListed
            } else {
                AccessState::Allowed
            }
        } else if allow > block {
            AccessState::Allowed
        } else {
            AccessState::Blocked
        }
    }
}
