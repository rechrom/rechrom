//! Narrow retained PermissionsPolicy declarations consumed by LNA.
//! Source permissions_policy_parser.cc:216-368,398-545,669-710;
//! network permissions_policy.cc:199-253,579-636; features.json5:21-33,406-431.
//! Structured syntax uses Chromium sfv 0.15's RFC8941/lenient configuration.
pub use crate::permissions_policy_features::PermissionsPolicyFeature as NetworkPolicyFeature;
use crate::permissions_policy_features::{
    feature_default_all, feature_descriptors, is_client_hint,
};
use crate::{
    content_security_policy::PermissionsOriginPattern, persistent_document::SecurityOrigin,
};
use foundation::security::OriginKey;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};
#[derive(Clone, Copy, Debug)]
pub enum PolicyMessageLevel {
    Error,
    Warning,
}
pub struct PolicyParsingMessage {
    pub level: PolicyMessageLevel,
    pub message: String,
    pub location: foundation::content_security_policy::CSPSourceLocation,
}
#[derive(Clone)]
struct Allowlist {
    all: bool,
    self_origin: Option<OriginKey>,
    origins: Vec<PermissionsOriginPattern>,
    reporting_endpoint: Option<String>,
    matches_opaque_src: bool,
}
pub struct NetworkPermissionsPolicy {
    origin: OriginKey,
    declarations: RefCell<HashMap<NetworkPolicyFeature, Allowlist>>,
    messages: RefCell<Vec<PolicyParsingMessage>>,
    inherited: Option<HashSet<NetworkPolicyFeature>>,
}
fn feature(name: &str) -> Option<NetworkPolicyFeature> {
    crate::permissions_policy_features::descriptor(name)
        .filter(|entry| !entry.isolated_only)
        .map(|entry| entry.feature)
}
fn message(messages: &mut Vec<PolicyParsingMessage>, level: PolicyMessageLevel, message: String) {
    messages.push(PolicyParsingMessage {
        level,
        message,
        location: foundation::content_security_policy::CSPSourceLocation::Unknown,
    });
}
impl NetworkPermissionsPolicy {
    pub fn new(origin: OriginKey) -> Self {
        Self {
            origin,
            declarations: RefCell::new(HashMap::new()),
            messages: RefCell::new(Vec::new()),
            inherited: None,
        }
    }
    pub fn SetHeaderPoliciesForOutermostDocument(
        &self,
        legacy: &str,
        modern: &str,
        origin: &SecurityOrigin,
        response_url: &str,
    ) {
        let mut declarations = HashMap::new();
        let mut messages = Vec::new();
        let length = modern.encode_utf16().count();
        if length > 65536 {
            message(
                &mut messages,
                PolicyMessageLevel::Error,
                format!("Permissions policy declaration exceeds size limit({length}>65536)"),
            );
        } else {
            let dictionary = sfv::Parser::new(modern.as_bytes())
                .with_version(sfv::Version::Rfc8941)
                .with_lenient_mode(true)
                .parse::<sfv::Dictionary>();
            match dictionary {
                Err(_)=>message(&mut messages,PolicyMessageLevel::Error,"Parse of permissions policy failed because of errors reported by structured header parser.".into()),
                Ok(dictionary)=>{
                    for (name,value) in dictionary {
                        let Some(feature)=feature(name.as_str())else{continue;};
                        let (items,params)=match value {
                            sfv::ListEntry::Item(item)=>(vec![sfv::Item{bare_item:item.bare_item,params:Default::default()}],item.params),
                            sfv::ListEntry::InnerList(list)=>(list.items,list.params),
                        };
                        let endpoint=params.get("report-to").and_then(|value|match value{sfv::BareItem::Token(value)=>Some(value.as_str().to_owned()),_=>None});
                        let mut tokens=Vec::new();
                        for item in items {
                            if !item.params.is_empty(){message(&mut messages,PolicyMessageLevel::Warning,format!("Feature {}'s parameters are ignored.",name.as_str()));}
                            match item.bare_item {
                                sfv::BareItem::Token(token)if token.as_str()=="*"=>tokens.push("*".into()),
                                sfv::BareItem::Token(token)if token.as_str()=="self"=>tokens.push("'self'".into()),
                                sfv::BareItem::String(value)=>tokens.push(value.as_str().to_owned()),
                                _=>message(&mut messages,PolicyMessageLevel::Warning,format!("Invalid allowlist item for feature {}. Allowlist item must be *, self, or quoted url.",name.as_str())),
                            }
                        }
                        // Empty structured lists mean 'none'. Empty legacy
                        // declarations, by contrast, mean 'self'.
                        let mut allowlist=parse_allowlist(&tokens,origin,false,None,&mut messages);
                        allowlist.reporting_endpoint=endpoint;
                        declarations.insert(feature,allowlist);
                    }
                }
            }
        }
        let length = legacy.encode_utf16().count();
        if length > 65536 {
            message(
                &mut messages,
                PolicyMessageLevel::Error,
                format!("Feature policy declaration exceeds size limit({length}>65536)"),
            );
        } else {
            for entry in legacy.split([',', ';']).filter(|entry| !entry.is_empty()) {
                let mut words = entry.split_ascii_whitespace();
                let Some(name) = words.next() else {
                    continue;
                };
                let Some(feature) = feature(name) else {
                    continue;
                };
                // ParseHeader merges legacy declarations only for features
                // absent from Permissions-Policy; first legacy entry wins.
                if declarations.contains_key(&feature) {
                    continue;
                }
                let tokens = words.map(str::to_owned).collect::<Vec<_>>();
                declarations.insert(
                    feature,
                    parse_allowlist(&tokens, origin, true, None, &mut messages),
                );
            }
        }
        *self.declarations.borrow_mut() = declarations;
        for message in &mut messages {
            message.location =
                foundation::content_security_policy::CSPSourceLocation::ResponseHeader {
                    url: response_url.to_owned(),
                };
        }
        self.messages.borrow_mut().extend(messages);
    }
    /// Current native Pages are independent outermost frames: their inherited
    /// feature set is enabled. Declared policy intersects it; absent local /
    /// loopback declarations use EnableForSelf against this same origin.
    pub fn IsFeatureEnabled(&self, feature: NetworkPolicyFeature, origin: &SecurityOrigin) -> bool {
        if self
            .inherited
            .as_ref()
            .is_some_and(|inherited| !inherited.contains(&feature))
        {
            return false;
        }
        let declarations = self.declarations.borrow();
        let Some(allowlist) = declarations.get(&feature) else {
            return feature_default_all(feature) || self.origin == origin.Key();
        };
        let key = origin.Key();
        allowlist.all
            || allowlist.self_origin.as_ref() == Some(&key)
            || allowlist
                .origins
                .iter()
                .any(|pattern| pattern.Matches(&key))
    }
    pub fn ReportingEndpoint(&self, feature: NetworkPolicyFeature) -> Option<String> {
        self.declarations
            .borrow()
            .get(&feature)
            .and_then(|allowlist| allowlist.reporting_endpoint.clone())
    }
    pub fn TakeParsingMessages(&self) -> Vec<PolicyParsingMessage> {
        std::mem::take(&mut *self.messages.borrow_mut())
    }
}
fn parse_allowlist(
    tokens: &[String],
    origin: &SecurityOrigin,
    legacy: bool,
    src: Option<&SecurityOrigin>,
    messages: &mut Vec<PolicyParsingMessage>,
) -> Allowlist {
    let mut result = Allowlist {
        all: false,
        self_origin: None,
        origins: Vec::new(),
        reporting_endpoint: None,
        matches_opaque_src: false,
    };
    if tokens.is_empty() && legacy {
        if let Some(src) = src {
            if src.IsOpaque() {
                result.matches_opaque_src = true;
            } else if let Some(pattern) = PermissionsOriginPattern::FromOrigin(&src.Key()) {
                result.origins.push(pattern);
            }
        } else {
            result.self_origin = Some(origin.Key());
        }
    }
    for token in tokens {
        if !token.is_ascii() {
            message(
                messages,
                PolicyMessageLevel::Warning,
                "Non-ASCII characters in origin.".into(),
            );
            continue;
        }
        if token.eq_ignore_ascii_case("'self'") {
            result.self_origin = Some(origin.Key());
        } else if token.eq_ignore_ascii_case("'src'") && src.is_some() {
            let src = src.unwrap();
            if src.IsOpaque() {
                result.matches_opaque_src = true;
            } else if let Some(pattern) = PermissionsOriginPattern::FromOrigin(&src.Key()) {
                result.origins.push(pattern);
            }
        } else if token == "*" {
            result.all = true;
            result.matches_opaque_src = true;
        } else if token.eq_ignore_ascii_case("'none'") {
        } else if let Some(pattern) = PermissionsOriginPattern::Parse(token, !legacy) {
            result.origins.push(pattern);
        } else {
            message(
                messages,
                PolicyMessageLevel::Warning,
                format!("Unrecognized origin: '{}'.", token),
            );
        }
    }
    if result.all {
        result.origins.clear();
    }
    result.origins.sort();
    result
}

impl NetworkPermissionsPolicy {
    /// network PermissionsPolicy.cc145-160: clone compiled declarations,
    /// leaving the new context's parser/report message queue empty.
    pub(crate) fn CopyStateFrom(&self) -> std::rc::Rc<Self> {
        std::rc::Rc::new(Self {
            origin: self.origin.clone(),
            declarations: RefCell::new(self.declarations.borrow().clone()),
            messages: RefCell::new(Vec::new()),
            inherited: self.inherited.clone(),
        })
    }
}

/// Chromium ParsedPermissionsPolicyDeclaration, permissions_policy_declaration.h:20-50.
/// The compiled allowlist remains private to this policy owner.
#[derive(Clone)]
pub struct ParsedPermissionsPolicyDeclaration {
    pub feature: NetworkPolicyFeature,
    allowlist: Allowlist,
}
/// One compiled container policy retained by HTMLFrameOwnerElement and passed
/// unchanged to FrameLoader. No serialized allow attribute crosses that port.
pub struct ParsedPermissionsPolicy {
    declarations: Vec<ParsedPermissionsPolicyDeclaration>,
    src_origin: Rc<SecurityOrigin>,
}
impl ParsedPermissionsPolicy {
    pub fn SourceOrigin(&self) -> &Rc<SecurityOrigin> {
        &self.src_origin
    }
    pub fn Declarations(&self) -> &[ParsedPermissionsPolicyDeclaration] {
        &self.declarations
    }
    /// PermissionsPolicyParser.cc:216-368,398-456,724-734. This shares the
    /// existing header parser's compiled allowlist and native origin matcher.
    pub fn ParseAttribute(
        value: &str,
        self_origin: &SecurityOrigin,
        src_origin: Rc<SecurityOrigin>,
        isolated_context: bool,
        messages: &mut Vec<PolicyParsingMessage>,
    ) -> Self {
        let mut declarations = Vec::new();
        let mut observed = HashSet::new();
        if value.encode_utf16().count() > 65536 {
            message(
                messages,
                PolicyMessageLevel::Error,
                format!(
                    "Feature policy declaration exceeds size limit({}>65536)",
                    value.encode_utf16().count()
                ),
            );
        } else {
            for entry in value.split(';').filter(|entry| !entry.is_empty()) {
                let mut tokens = entry
                    .split(|c| matches!(c, '\t' | '\n' | '\x0c' | '\r' | ' '))
                    .filter(|token| !token.is_empty());
                let Some(name) = tokens.next() else {
                    continue;
                };
                let Some(descriptor) = crate::permissions_policy_features::descriptor(name)
                    .filter(|entry| !entry.isolated_only || isolated_context)
                else {
                    message(
                        messages,
                        PolicyMessageLevel::Warning,
                        format!("Unrecognized feature: '{name}'."),
                    );
                    continue;
                };
                let tokens = tokens.map(str::to_owned).collect::<Vec<_>>();
                // Source parses the allowlist before discarding duplicate
                // feature declarations, so diagnostics keep their source order.
                let allowlist = parse_allowlist(
                    &tokens,
                    self_origin,
                    true,
                    Some(src_origin.as_ref()),
                    messages,
                );
                if observed.insert(descriptor.feature) {
                    declarations.push(ParsedPermissionsPolicyDeclaration {
                        feature: descriptor.feature,
                        allowlist,
                    });
                }
            }
        }
        Self {
            declarations,
            src_origin,
        }
    }
    fn AllowEverywhereIfNotPresent(&mut self, feature: NetworkPolicyFeature) -> bool {
        if self
            .declarations
            .iter()
            .any(|declaration| declaration.feature == feature)
        {
            return false;
        }
        self.declarations.push(ParsedPermissionsPolicyDeclaration {
            feature,
            allowlist: Allowlist {
                all: true,
                self_origin: None,
                origins: vec![],
                reporting_endpoint: None,
                matches_opaque_src: true,
            },
        });
        true
    }
    /// client_hints_util.cc:96-165: merge only explicit parent allowlists.
    /// Defaults are not promoted into declarations.
    fn ApplyClientHintDelegation(&mut self, parent: &NetworkPermissionsPolicy) {
        for (feature, parent_allowlist) in parent.declarations.borrow().iter() {
            if !is_client_hint(*feature) {
                continue;
            }
            let first = self
                .declarations
                .iter()
                .find(|declaration| declaration.feature == *feature)
                .cloned();
            self.declarations
                .retain(|declaration| declaration.feature != *feature);
            let mut allowlist =
                first
                    .map(|declaration| declaration.allowlist)
                    .unwrap_or(Allowlist {
                        all: false,
                        self_origin: None,
                        origins: vec![],
                        reporting_endpoint: None,
                        matches_opaque_src: false,
                    });
            allowlist.self_origin = parent_allowlist.self_origin.clone();
            allowlist.all |= parent_allowlist.all;
            allowlist.matches_opaque_src |= parent_allowlist.matches_opaque_src;
            if allowlist.all {
                allowlist.origins.clear();
            } else {
                allowlist
                    .origins
                    .extend(parent_allowlist.origins.iter().cloned());
                allowlist.origins.sort();
                allowlist.origins.dedup();
            }
            self.declarations.push(ParsedPermissionsPolicyDeclaration {
                feature: *feature,
                allowlist,
            });
        }
    }
}
impl Allowlist {
    fn Contains(&self, origin: &OriginKey) -> bool {
        self.self_origin.as_ref() == Some(origin)
            || self.origins.iter().any(|pattern| pattern.Matches(origin))
            || if matches!(origin, OriginKey::Opaque(_)) {
                self.matches_opaque_src
            } else {
                self.all
            }
    }
}
impl NetworkPermissionsPolicy {
    /// PermissionsPolicy.cc:579-697, preserving the parent's inherited and
    /// declared owner state. The receiving child owns its resulting bitset.
    pub fn CreateForChild(
        &self,
        container: &ParsedPermissionsPolicy,
        origin: OriginKey,
    ) -> Rc<Self> {
        let mut inherited = HashSet::new();
        for descriptor in feature_descriptors() {
            let feature = descriptor.feature;
            let parent_admits = |target: &OriginKey| {
                !self
                    .inherited
                    .as_ref()
                    .is_some_and(|inherited| !inherited.contains(&feature))
                    && self
                        .declarations
                        .borrow()
                        .get(&feature)
                        .is_none_or(|allowlist| allowlist.Contains(target))
            };
            if !parent_admits(&self.origin) || !parent_admits(&origin) {
                continue;
            }
            let enabled = container
                .declarations
                .iter()
                .find(|declaration| declaration.feature == feature)
                .map_or_else(
                    || descriptor.default_all || self.origin == origin,
                    |declaration| declaration.allowlist.Contains(&origin),
                );
            if enabled {
                inherited.insert(feature);
                if feature == NetworkPolicyFeature::LocalNetworkAccess {
                    inherited.insert(NetworkPolicyFeature::LocalNetwork);
                    inherited.insert(NetworkPolicyFeature::LoopbackNetwork);
                }
            }
        }
        Rc::new(Self {
            origin,
            declarations: RefCell::new(HashMap::new()),
            messages: RefCell::new(vec![]),
            inherited: Some(inherited),
        })
    }
}

impl crate::persistent_document::PersistentDocument {
    /// HTMLIFrameElement.cc:492-558 and HTMLFrameElementBase.cc:155-171.
    pub(crate) fn ConstructFrameContainerPolicy(
        &self,
        node: usize,
        url: &crate::document_base_url::DocumentURLValue,
        sandbox: foundation::security::SandboxFlags,
    ) -> Rc<ParsedPermissionsPolicy> {
        let origin = self.SecurityContext().Origin();
        let registry = Rc::new(origin.SchemeRegistryForRequests());
        let src_origin = if sandbox.Contains(foundation::security::SandboxFlags::ORIGIN) {
            SecurityOrigin::CreateUniqueOpaque(registry)
        } else {
            let inherits = url.IsEmpty()
                || url.AsStr().eq_ignore_ascii_case("about:blank")
                || url.AsStr().eq_ignore_ascii_case("about:srcdoc");
            SecurityOrigin::CreateForNavigation(url.AsStr(), inherits.then_some(origin), registry)
        };
        let mut messages = Vec::new();
        let allow = self
            .Node(node)
            .FindAttribute("allow")
            .map_or("", |attribute| attribute.value.as_str());
        let mut policy = ParsedPermissionsPolicy::ParseAttribute(
            allow,
            origin,
            src_origin,
            false,
            &mut messages,
        );
        for (attribute, feature) in [
            ("allowfullscreen", NetworkPolicyFeature::Fullscreen),
            ("allowpaymentrequest", NetworkPolicyFeature::Payment),
        ] {
            if self.Node(node).FindAttribute(attribute).is_some()
                && !policy.AllowEverywhereIfNotPresent(feature)
            {
                message(
                    &mut messages,
                    PolicyMessageLevel::Warning,
                    format!("Allow attribute will take precedence over '{attribute}'."),
                );
            }
        }
        policy.ApplyClientHintDelegation(self.SecurityContext().NetworkPermissionsPolicy());
        self.SecurityContext()
            .NetworkPermissionsPolicy()
            .messages
            .borrow_mut()
            .extend(messages);
        Rc::new(policy)
    }
}
