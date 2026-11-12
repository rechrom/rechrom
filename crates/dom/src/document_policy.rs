//! Chromium 6c1d401f: common/permissions_policy/document_policy.cc:20-232,
//! policy_value.cc:18-159; core/permissions_policy/document_policy_parser.cc:
//! 35-258 and document_policy_features.json5:54-109. This policy is distinct
//! from PermissionsPolicy's origin allowlists and is shared with its Loader.
use std::{collections::BTreeMap, rc::Rc};
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum DocumentPolicyFeature {
    Default = 0,
    ForceLoadAtTop = 3,
    JSProfiling = 11,
    SyncXHR = 12,
    IncludeJSCallStacksInCrashReports = 14,
    ExpectNoLinkedResources = 15,
    NetworkEfficiencyGuardrails = 16,
    JSProfilingMode = 17,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PolicyValue {
    Bool(bool),
    DecDouble(f64),
    Enum(i32),
}
impl PolicyValue {
    pub fn IsCompatibleWith(self, required: Self) -> bool {
        match (self, required) {
            (Self::Bool(value), Self::Bool(required)) => !value || required,
            (Self::DecDouble(value), Self::DecDouble(required)) => value <= required,
            (Self::Enum(value), Self::Enum(required)) => value == required,
            _ => panic!("same DocumentPolicy feature has one source value type"),
        }
    }
}
pub type DocumentPolicyFeatureState = BTreeMap<DocumentPolicyFeature, PolicyValue>;
#[derive(Clone, Default, Debug)]
pub struct ParsedDocumentPolicy {
    pub feature_state: DocumentPolicyFeatureState,
    pub endpoint_map: BTreeMap<DocumentPolicyFeature, String>,
}
#[derive(Clone, Copy, Debug)]
pub enum PolicyMessageLevel {
    Warning,
    Error,
}
#[derive(Clone, Debug)]
pub struct DocumentPolicyMessage {
    pub level: PolicyMessageLevel,
    pub message: String,
}
const FEATURES: [DocumentPolicyFeature; 8] = [
    DocumentPolicyFeature::Default,
    DocumentPolicyFeature::ForceLoadAtTop,
    DocumentPolicyFeature::JSProfiling,
    DocumentPolicyFeature::SyncXHR,
    DocumentPolicyFeature::IncludeJSCallStacksInCrashReports,
    DocumentPolicyFeature::ExpectNoLinkedResources,
    DocumentPolicyFeature::NetworkEfficiencyGuardrails,
    DocumentPolicyFeature::JSProfilingMode,
];
impl DocumentPolicyFeature {
    pub fn Name(self) -> &'static str {
        match self {
            Self::Default => "*",
            Self::ForceLoadAtTop => "force-load-at-top",
            Self::JSProfiling => "js-profiling",
            Self::SyncXHR => "sync-xhr",
            Self::IncludeJSCallStacksInCrashReports => "include-js-call-stacks-in-crash-reports",
            Self::ExpectNoLinkedResources => "expect-no-linked-resources",
            Self::NetworkEfficiencyGuardrails => "network-efficiency-guardrails",
            Self::JSProfilingMode => "js-profiling-mode",
        }
    }
    pub fn DefaultValue(self) -> PolicyValue {
        match self {
            Self::Default | Self::SyncXHR => PolicyValue::Bool(true),
            Self::JSProfilingMode => PolicyValue::Enum(0),
            _ => PolicyValue::Bool(false),
        }
    }
    pub fn Available(self) -> bool {
        use foundation::RuntimeEnabledFeatures as F;
        match self {
            Self::SyncXHR => F::DocumentPolicySyncXHREnabled(),
            Self::ExpectNoLinkedResources => F::DocumentPolicyExpectNoLinkedResourcesEnabled(),
            Self::NetworkEfficiencyGuardrails => {
                F::DocumentPolicyNetworkEfficiencyGuardrailsEnabled()
            }
            Self::JSProfilingMode => F::DocumentPolicyJSProfilingModeEnabled(),
            _ => true,
        }
    }
}
fn message(
    messages: &mut Vec<DocumentPolicyMessage>,
    level: PolicyMessageLevel,
    prefix: &str,
    text: String,
) {
    messages.push(DocumentPolicyMessage {
        level,
        message: format!("{prefix}{text}"),
    });
}
pub fn ParseDocumentPolicy(
    header: &str,
    prefix: &str,
    messages: &mut Vec<DocumentPolicyMessage>,
) -> Option<ParsedDocumentPolicy> {
    if header.is_empty() {
        return Some(ParsedDocumentPolicy::default());
    }
    // DocumentPolicyParser passes String::Ascii(), including the per-code-unit
    // '?' replacement for non-ASCII iframe attribute text.
    let ascii = header
        .encode_utf16()
        .map(|c| if c <= 0x7f { c as u8 } else { b'?' })
        .collect::<Vec<_>>();
    let dictionary = match sfv::Parser::new(&ascii)
        .with_version(sfv::Version::Rfc8941)
        .with_lenient_mode(true)
        .parse::<sfv::Dictionary>()
    {
        Ok(value) => value,
        Err(_) => {
            message(messages,PolicyMessageLevel::Error,prefix,"Parse of document policy failed because of errors reported by structured header parser.".into());
            return None;
        }
    };
    let mut parsed = ParsedDocumentPolicy::default();
    let mut default_endpoint = String::new();
    for (name, entry) in dictionary {
        let item = match entry {
            sfv::ListEntry::Item(item) => item,
            sfv::ListEntry::InnerList(list) => {
                message(messages,PolicyMessageLevel::Warning,prefix,format!("Parameter for feature {} should be single item, but get list of items(length={}).",name.as_str(),list.items.len()));
                continue;
            }
        };
        let Some(feature) = FEATURES
            .iter()
            .copied()
            .find(|feature| feature.Name() == name.as_str())
        else {
            message(
                messages,
                PolicyMessageLevel::Warning,
                prefix,
                format!(
                    "Unrecognized document policy feature name {}.",
                    name.as_str()
                ),
            );
            continue;
        };
        let value = match (feature.DefaultValue(), &item.bare_item) {
            (PolicyValue::Bool(_), sfv::BareItem::Boolean(value)) => {
                Some(PolicyValue::Bool(*value))
            }
            (PolicyValue::Enum(_), sfv::BareItem::Token(token)) => match token.as_str() {
                "eager" => Some(PolicyValue::Enum(1)),
                "lazy" => Some(PolicyValue::Enum(2)),
                _ => None,
            },
            _ => None,
        };
        let Some(value) = value else {
            message(
                messages,
                PolicyMessageLevel::Warning,
                prefix,
                format!(
                    "Parameter for feature {} has an incompatible policy value type.",
                    name.as_str()
                ),
            );
            continue;
        };
        let mut endpoint = None;
        let mut invalid = false;
        for (param, value) in item.params {
            if param.as_str() == "report-to" {
                if let sfv::BareItem::Token(value) = value {
                    endpoint = Some(value.as_str().to_owned());
                } else {
                    message(
                        messages,
                        PolicyMessageLevel::Warning,
                        prefix,
                        format!(
                            "\"report-to\" parameter should be a token in feature {}.",
                            name.as_str()
                        ),
                    );
                    invalid = true;
                    break;
                }
            } else {
                message(
                    messages,
                    PolicyMessageLevel::Warning,
                    prefix,
                    format!(
                        "Unrecognized parameter name {} for feature {}.",
                        param.as_str(),
                        name.as_str()
                    ),
                );
            }
        }
        if invalid {
            continue;
        }
        if feature == DocumentPolicyFeature::Default {
            if let Some(endpoint) = endpoint {
                default_endpoint = endpoint;
            }
            continue;
        }
        if !feature.Available() {
            continue;
        }
        parsed.feature_state.entry(feature).or_insert(value);
        if let Some(endpoint) = endpoint {
            parsed.endpoint_map.entry(feature).or_insert(endpoint);
        }
    }
    if !default_endpoint.is_empty() {
        for feature in parsed.feature_state.keys() {
            parsed
                .endpoint_map
                .entry(*feature)
                .or_insert_with(|| default_endpoint.clone());
        }
    }
    parsed.endpoint_map.retain(|_, endpoint| endpoint != "none");
    Some(parsed)
}
pub fn MergeFeatureState(
    base: &DocumentPolicyFeatureState,
    overrides: &DocumentPolicyFeatureState,
) -> DocumentPolicyFeatureState {
    let mut result = base.clone();
    for (feature, value) in overrides {
        result
            .entry(*feature)
            .and_modify(|base| {
                if !base.IsCompatibleWith(*value) {
                    *base = *value;
                }
            })
            .or_insert(*value);
    }
    result
}
pub fn IsPolicyCompatible(
    required: &DocumentPolicyFeatureState,
    incoming: &DocumentPolicyFeatureState,
) -> bool {
    required.iter().all(|(feature, required)| {
        incoming
            .get(feature)
            .copied()
            .unwrap_or_else(|| feature.DefaultValue())
            .IsCompatibleWith(*required)
    })
}
pub struct DocumentPolicy {
    state: DocumentPolicyFeatureState,
    endpoints: BTreeMap<DocumentPolicyFeature, String>,
}
impl DocumentPolicy {
    pub fn CreateWithHeaderPolicy(parsed: &ParsedDocumentPolicy) -> Rc<Self> {
        let mut state = FEATURES
            .into_iter()
            .map(|feature| (feature, feature.DefaultValue()))
            .collect::<DocumentPolicyFeatureState>();
        state.extend(parsed.feature_state.iter().map(|(k, v)| (*k, *v)));
        Rc::new(Self {
            state,
            endpoints: parsed.endpoint_map.clone(),
        })
    }
    pub fn GetFeatureValue(&self, feature: DocumentPolicyFeature) -> PolicyValue {
        self.state[&feature]
    }
    pub fn IsFeatureEnabled(&self, feature: DocumentPolicyFeature) -> bool {
        match self.GetFeatureValue(feature) {
            PolicyValue::Bool(value) => value,
            _ => panic!("enum DocumentPolicy feature requires an explicit threshold"),
        }
    }
    pub fn IsFeatureEnabledAt(
        &self,
        feature: DocumentPolicyFeature,
        threshold: PolicyValue,
    ) -> bool {
        threshold.IsCompatibleWith(self.GetFeatureValue(feature))
    }
    pub fn GetFeatureEndpoint(&self, feature: DocumentPolicyFeature) -> Option<&str> {
        self.endpoints.get(&feature).map(String::as_str)
    }
}

#[derive(Default)]
pub(crate) struct IFrameDocumentPolicy {
    source: String,
    own: Rc<DocumentPolicyFeatureState>,
    required: Rc<DocumentPolicyFeatureState>,
}
impl crate::persistent_document::PersistentDocument {
    pub fn RequiredDocumentPolicy(&self) -> Rc<DocumentPolicyFeatureState> {
        self.document_metadata
            .get(&self.Node(self.Root()).Id())
            .map_or_else(
                || Rc::new(BTreeMap::new()),
                |data| data.required_document_policy.clone(),
            )
    }
    pub fn SetRequiredDocumentPolicy(&mut self, required: Rc<DocumentPolicyFeatureState>) {
        let id = self.Node(self.Root()).Id();
        self.document_metadata
            .entry(id)
            .or_default()
            .required_document_policy = required;
    }
    pub fn QueueDocumentPolicyMessages(
        &mut self,
        messages: impl IntoIterator<Item = DocumentPolicyMessage>,
    ) {
        let id = self.Node(self.Root()).Id();
        self.document_metadata
            .entry(id)
            .or_default()
            .document_policy_messages
            .borrow_mut()
            .extend(messages);
    }
    pub fn TakeDocumentPolicyMessages(&self) -> Vec<DocumentPolicyMessage> {
        self.document_metadata
            .get(&self.Node(self.Root()).Id())
            .map_or_else(Vec::new, |data| {
                data.document_policy_messages
                    .borrow_mut()
                    .drain(..)
                    .collect()
            })
    }
    /// HTMLIFrameElement.cc:400-404,452-489; HTMLFrameOwnerElement.cc460-479.
    /// Parse the element's original attribute once per changed source string;
    /// retain the compiled policy and derive only the small parent merge.
    pub(crate) fn IFrameDocumentPolicyAttributeChanged(
        &mut self,
        node: usize,
        name: &str,
        namespace: &str,
    ) {
        if !namespace.is_empty()
            || name != "policy"
            || !self.Node(node).IsHTMLElement("iframe")
            || !foundation::RuntimeEnabledFeatures::DocumentPolicyNegotiationEnabled()
        {
            return;
        }
        self.UpdateIFrameRequiredDocumentPolicy(node);
    }
    pub fn UpdateIFrameRequiredDocumentPolicy(&mut self, node: usize) {
        assert!(self.Node(node).IsHTMLElement("iframe"));
        if !foundation::RuntimeEnabledFeatures::DocumentPolicyNegotiationEnabled() {
            return;
        }
        let source = self
            .Node(node)
            .FindAttribute("policy")
            .map_or_else(String::new, |attr| attr.value.clone());
        let node_id = self.Node(node).Id();
        let document = self.NodeDocumentHandle(node);
        let document_index = self.DocumentPolicyDocumentIndex(document);
        let root = self.Node(document_index).Id();
        let parent = self.document_metadata.get(&root).map_or_else(
            || Rc::new(BTreeMap::new()),
            |data| data.required_document_policy.clone(),
        );
        let metadata = self.document_metadata.entry(root).or_default();
        let mut policies = metadata.iframe_document_policies.borrow_mut();
        let policy = policies.entry(node_id).or_default();
        if policy.source != source {
            let mut messages = vec![];
            let parsed = ParseDocumentPolicy(&source, "", &mut messages).unwrap_or_default();
            if !parsed.endpoint_map.is_empty() {
                messages.push(DocumentPolicyMessage {
                    level: PolicyMessageLevel::Warning,
                    message: "Iframe policy attribute cannot specify reporting endpoint.".into(),
                });
            }
            policy.source = source;
            policy.own = Rc::new(parsed.feature_state);
            metadata
                .document_policy_messages
                .borrow_mut()
                .extend(messages);
        }
        policy.required = Rc::new(MergeFeatureState(&policy.own, &parent));
    }
    pub fn IFrameRequiredDocumentPolicy(&self, node: usize) -> Rc<DocumentPolicyFeatureState> {
        assert!(self.Node(node).IsHTMLElement("iframe"));
        self.document_metadata
            .get(
                &self
                    .Node(self.DocumentPolicyDocumentIndex(self.NodeDocumentHandle(node)))
                    .Id(),
            )
            .and_then(|data| {
                data.iframe_document_policies
                    .borrow()
                    .get(&self.Node(node).Id())
                    .map(|policy| policy.required.clone())
            })
            .unwrap_or_else(|| Rc::new(BTreeMap::new()))
    }
}

/// DocumentPolicy.cc:88-124: feature names determine dictionary order. These
/// source feature descriptors contain only booleans and the profiler enum.
pub fn SerializeDocumentPolicy(state: &DocumentPolicyFeatureState) -> Option<String> {
    let mut entries = state.iter().collect::<Vec<_>>();
    entries.sort_by_key(|(feature, _)| feature.Name());
    let mut result = Vec::with_capacity(entries.len());
    for (feature, value) in entries {
        let value = match value {
            PolicyValue::Bool(true) => feature.Name().to_owned(),
            PolicyValue::Bool(false) => format!("{}=?0", feature.Name()),
            PolicyValue::Enum(0) => continue,
            PolicyValue::Enum(1) => format!("{}=eager", feature.Name()),
            PolicyValue::Enum(2) => format!("{}=lazy", feature.Name()),
            _ => return None,
        };
        result.push(value);
    }
    Some(result.join(", "))
}

impl crate::persistent_document::PersistentDocument {
    // The metadata belongs to a resident Document, not to an arena index or
    // an Id which can collide with an adopted foreign document.
    fn DocumentPolicyDocumentIndex(
        &self,
        document: crate::persistent_document::DOMOwnerHandle,
    ) -> usize {
        let index = self
            .FindNodeById(document.Id())
            .expect("policy owner document");
        assert_eq!(self.NodeHandle(index), document);
        assert_eq!(
            self.Node(index).Type(),
            crate::persistent_document::DOMNodeType::kDocument
        );
        index
    }
    pub(crate) fn IFrameDocumentPolicyOwnerChanged(
        &mut self,
        node: usize,
        previous_document: Option<crate::persistent_document::DOMOwnerHandle>,
    ) {
        if !foundation::RuntimeEnabledFeatures::DocumentPolicyNegotiationEnabled() {
            return;
        }
        let id = self.Node(node).Id();
        let previous = self
            .Node(self.DocumentPolicyDocumentIndex(
                previous_document.expect("iframe has an owner document"),
            ))
            .Id();
        let record = self
            .document_metadata
            .get(&previous)
            .and_then(|metadata| metadata.iframe_document_policies.borrow_mut().remove(&id));
        let document = self.NodeDocumentHandle(node);
        let root = self.Node(self.DocumentPolicyDocumentIndex(document)).Id();
        if let Some(record) = record {
            self.document_metadata
                .entry(root)
                .or_default()
                .iframe_document_policies
                .borrow_mut()
                .insert(id, record);
        }
        self.UpdateIFrameRequiredDocumentPolicy(node);
    }
}

/// Execution-context runtime feature facts retained with the canonical
/// Document. These are separate from Document-Policy header feature values.
/// Source document.cc:2132-2136; runtime_enabled_features.json5:5941-5952.
pub(crate) struct DocumentRuntimeFeatureFacts {
    standardized_browser_zoom: bool,
    standardized_browser_zoom_opt_out: std::cell::Cell<bool>,
}
impl Default for DocumentRuntimeFeatureFacts {
    fn default() -> Self {
        Self {
            // Pinned feature metadata: StandardizedBrowserZoom is stable;
            // an origin trial is inactive until its real validator enables it.
            standardized_browser_zoom: true,
            standardized_browser_zoom_opt_out: std::cell::Cell::new(false),
        }
    }
}
impl crate::persistent_document::PersistentDocument {
    pub fn StandardizedBrowserZoomEnabled(&self) -> bool {
        let facts = &self
            .document_metadata
            .get(&self.Node(self.Root()).Id())
            .expect("runtime feature facts allocated with Document")
            .runtime_features;
        facts.standardized_browser_zoom && !facts.standardized_browser_zoom_opt_out.get()
    }
    // Only the DOM's validated OriginTrial feature producer may apply this
    // fact. CSS, URL strings, native embedding inputs and IO cannot set it.
    pub(crate) fn ApplyValidatedOriginTrialStandardizedBrowserZoomOptOut(&self, enabled: bool) {
        self.document_metadata
            .get(&self.Node(self.Root()).Id())
            .expect("same Document runtime feature owner")
            .runtime_features
            .standardized_browser_zoom_opt_out
            .set(enabled);
    }
}

impl DocumentPolicy {
    /// common/permissions_policy/document_policy.cc25-36.
    pub(crate) fn CopyStateFrom(&self) -> Rc<Self> {
        Rc::new(Self {
            state: self.state.clone(),
            endpoints: self.endpoints.clone(),
        })
    }
}
