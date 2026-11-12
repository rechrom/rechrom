//! Document URL owner and HTMLBaseElement lifecycle. Chromium revision
//! 6c1d401fcca5e1b0030563a90c2f2bba168e0c15, document.cc:5043-5088,5164-5230;
//! html_base_element.cc:35-56. Base target is a separate source member.
use crate::{
    persistent_document::{DOMNamespace, PersistentDocument},
    WebFeature,
};
use foundation::content_security_policy::*;
use std::{ops::Deref, rc::Rc};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentURLValue {
    serialized: Rc<str>,
    parsed: Option<url::Url>,
}
impl DocumentURLValue {
    fn Empty() -> Rc<Self> {
        Rc::new(Self {
            serialized: Rc::from(""),
            parsed: None,
        })
    }
    fn Parsed(url: url::Url) -> Rc<Self> {
        Rc::new(Self {
            serialized: Rc::from(url.as_str()),
            parsed: Some(url),
        })
    }
    fn Absolute(value: &str) -> Rc<Self> {
        match url::Url::parse(value) {
            Ok(url) => Self::Parsed(url),
            Err(_) => Rc::new(Self {
                serialized: Rc::from(value),
                parsed: None,
            }),
        }
    }
    pub(crate) fn Resolve(base: &Self, value: &str) -> Rc<Self> {
        match base.CompleteURL(value) {
            Some(url) => Self::Parsed(url),
            None => Self::Absolute(value),
        }
    }
    pub fn AsStr(&self) -> &str {
        &self.serialized
    }
    pub fn IsValid(&self) -> bool {
        self.parsed.is_some()
    }
    pub fn ParsedURL(&self) -> Option<&url::Url> {
        self.parsed.as_ref()
    }
    pub fn IsEmpty(&self) -> bool {
        self.serialized.is_empty()
    }
    pub fn CompleteURL(&self, value: &str) -> Option<url::Url> {
        self.parsed.as_ref().map_or_else(
            || url::Url::parse(value).ok(),
            |base| {
                if value.is_empty() {
                    Some(base.clone())
                } else {
                    base.join(value).ok()
                }
            },
        )
    }
    fn IsDisallowedBaseScheme(&self) -> Option<&'static str> {
        match self.parsed.as_ref()?.scheme() {
            "data" => Some("data"),
            "javascript" => Some("javascript"),
            _ => None,
        }
    }
}
impl Deref for DocumentURLValue {
    type Target = str;
    fn deref(&self) -> &str {
        self.AsStr()
    }
}
#[derive(Clone, Debug)]
pub struct BaseURLChanged {
    pub value: Rc<DocumentURLValue>,
}
#[derive(Default)]
pub(super) struct DocumentBaseURL {
    revision: u64,
    has_href: bool,
    pub base: Option<Rc<DocumentURLValue>>,
    element: Option<Rc<DocumentURLValue>>,
    override_: Option<Rc<DocumentURLValue>>,
    changes: Vec<BaseURLChanged>,
    disallowed_schemes: Vec<&'static str>,
}
impl PersistentDocument {
    /// Element.cc:11644-11657. URL parsing crosses from WTF-16 into the URL
    /// Standard's scalar-value input; the attribute's original UTF-16 owner is
    /// retained unchanged on Element.
    pub fn GetNonEmptyURLAttribute(&self, node: usize, name: &str) -> Rc<DocumentURLValue> {
        let Some(value) = self.Node(node).AttributeValueUTF16(name) else {
            return DocumentURLValue::Empty();
        };
        let space = |unit: &u16| matches!(*unit, 9 | 10 | 12 | 13 | 32);
        let start = value
            .iter()
            .position(|unit| !space(unit))
            .unwrap_or(value.len());
        let end = value
            .iter()
            .rposition(|unit| !space(unit))
            .map_or(start, |index| index + 1);
        if start == end {
            return DocumentURLValue::Empty();
        }
        let scalar_value: String = char::decode_utf16(value[start..end].iter().copied())
            .map(|unit| unit.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect();
        DocumentURLValue::Resolve(&self.BaseURL(), &scalar_value)
    }
    /// Source Document::CompleteURL returns an immutable KURL value, including
    /// an invalid non-null value. Consumers retain the original URL owner.
    pub fn CompleteURLValue(&self, value: &str) -> Rc<DocumentURLValue> {
        DocumentURLValue::Resolve(&self.BaseURL(), value)
    }
    pub fn HasBaseElementHref(&self) -> bool {
        self.base_url.has_href
    }
    pub fn BaseURLRevision(&self) -> u64 {
        self.base_url.revision
    }
    pub fn BaseURL(&self) -> Rc<DocumentURLValue> {
        self.base_url
            .base
            .clone()
            .unwrap_or_else(DocumentURLValue::Empty)
    }
    pub fn ValidBaseElementURL(&self) -> Option<Rc<DocumentURLValue>> {
        self.base_url
            .element
            .as_ref()
            .filter(|url| url.IsValid())
            .cloned()
    }
    pub fn TakeBaseURLChanges(&mut self) -> Vec<BaseURLChanged> {
        std::mem::take(&mut self.base_url.changes)
    }
    pub fn TakeDisallowedBaseURLSchemes(&mut self) -> Vec<&'static str> {
        std::mem::take(&mut self.base_url.disallowed_schemes)
    }
    /// Document::SetBaseURLOverride, document.cc:5158-5161.
    pub fn SetBaseURLOverride(&mut self, value: &str) {
        self.base_url.override_ = (!value.is_empty()).then(|| DocumentURLValue::Absolute(value));
        self.UpdateBaseURL();
    }
    pub(super) fn UpdateBaseURL(&mut self) {
        let value = self
            .base_url
            .element
            .as_ref()
            .filter(|v| !v.IsEmpty())
            .or(self.base_url.override_.as_ref())
            .cloned()
            .unwrap_or_else(|| DocumentURLValue::Absolute(self.DocumentURL()));
        let value = if value.IsValid() {
            value
        } else {
            DocumentURLValue::Empty()
        };
        self.base_url.revision = self
            .base_url
            .revision
            .checked_add(1)
            .expect("Document base URL revision");
        self.base_url.base = Some(value.clone());
        self.base_url.changes.push(BaseURLChanged { value });
    }
    /// Process only the URL member of ProcessBaseElement. Traversal is the
    /// ordinary document tree; disconnected and shadow base elements are absent.
    pub fn ProcessBaseElementURL(&mut self) {
        self.CountUse(WebFeature::kBaseElement);
        let mut current = self.NextFormStateElement(self.Root());
        let mut href = None;
        while let Some(node) = current {
            if self.Node(node).IsElement("base", DOMNamespace::kHTML) {
                if self.ContentSecurityPolicy().IsActive() {
                    self.CountUse(WebFeature::kContentSecurityPolicyWithBaseElement);
                }
                if let Some(attribute) = self.Node(node).FindAttribute("href") {
                    href = Some(attribute.value.clone());
                    break;
                }
            }
            current = self.NextFormStateElement(node);
        }
        self.base_url.has_href = href.is_some();
        let fallback = DocumentURLValue::Absolute(self.DocumentURL());
        let candidate = href
            .as_deref()
            .map(|href| href.trim_matches(|c| matches!(c, '\t' | '\n' | '\x0c' | '\r' | ' ')))
            .filter(|href| !href.is_empty())
            .map_or_else(DocumentURLValue::Empty, |href| {
                DocumentURLValue::Resolve(&fallback, href)
            });
        if !candidate.IsEmpty() {
            if let Some(scheme) = candidate.IsDisallowedBaseScheme() {
                self.CountUse(WebFeature::kBaseWithDataHref);
                self.base_url.disallowed_schemes.push(scheme);
            }
            if !self.SecurityContext().CanRequest(candidate.AsStr()) {
                self.CountUse(WebFeature::kBaseWithCrossOriginHref);
            }
        }
        let previous = self
            .base_url
            .element
            .clone()
            .unwrap_or_else(DocumentURLValue::Empty);
        if previous != candidate {
            let accepted = candidate.IsDisallowedBaseScheme().is_none() && {
                let policy = self.ContentSecurityPolicy();
                let verdict = policy.AllowBaseURI(&CSPSourceRequest {
                    url: candidate.AsStr(),
                    url_before_redirects: candidate.AsStr(),
                    world: CSPWorld::Main,
                    redirect: CSPRedirectStatus::NoRedirect,
                    reporting: CSPReportingDisposition::Report,
                    check_header_type: CSPCheckHeaderType::All,
                });
                policy.QueueLocalViolations(verdict.violations);
                verdict.allowed
            };
            self.base_url.element = Some(if accepted { candidate } else { fallback });
            self.UpdateBaseURL();
        }
    }
    pub(super) fn BaseElementURLAttributeChanged(
        &mut self,
        node: usize,
        name: &str,
        namespace: &str,
    ) {
        if namespace.is_empty()
            && matches!(name, "href" | "target")
            && self.Node(node).IsElement("base", DOMNamespace::kHTML)
        {
            self.ProcessBaseElementURL();
        }
    }
}
