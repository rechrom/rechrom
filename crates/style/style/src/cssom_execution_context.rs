//! Current Document -> CSSOM/ElementSheet ownership, without a retained DOM
//! borrow or a host callback. The parser reads the original shared counters.
//! Chromium Document.cc:5069-5073,5335-5338; CSSParserContext.cc:85-136;
//! InlineCSSStyleDeclaration.cc:57-60; PropertySetCSSStyleDeclaration.h:40-45.
#![allow(non_snake_case)]
use crate::{
    document_style_engine::DocumentStyleEngine,
    parser::{css_parser_context::*, css_parser_impl::*, css_parser_mode::CSSParserMode},
    production_style_sheet::{Backend, Feature, SourceURL},
    style_sheet_contents::StyleSheetContents,
};
use foundation::{CSSPropertyID, String};
use std::rc::{Rc, Weak};
use style_parser::production_css_execution_context::{
    CSSReferrerPolicy, ProductionCSSDOMWrapperWorld, ProductionCSSExecutionContext,
};

struct ParserDocument {
    execution: Rc<std::cell::RefCell<Weak<ProductionCSSExecutionContext>>>,
    execution_attached: Rc<std::cell::Cell<bool>>,
    counters: Rc<foundation::css_use_counter::CSSUseCounter>,
    is_for_markup_sanitization: bool,
}
impl CSSParserContextDocument<Backend> for ParserDocument {
    fn CountUse(&self, feature: Feature) {
        self.counters.Count(feature.CounterFeature());
    }
    fn CountWebDXFeature(&self, feature: Feature) {
        self.counters.CountWebDX(feature.CounterFeature());
    }
    fn CountProperty(&self, property: CSSPropertyID) {
        self.counters.CountProperty(property);
    }
    fn GetExecutionContext(&self) -> Option<Rc<ProductionCSSExecutionContext>> {
        if !self.execution_attached.get() {
            return None;
        }
        self.execution
            .borrow()
            .upgrade()
            .filter(|context| !context.IsContextDestroyed())
    }
    fn IsForMarkupSanitization(&self) -> bool {
        self.is_for_markup_sanitization
    }
}
/// The Style-owned, single-Document CSSOM source shared with native bindings.
/// Its narrow operation borrows current DOM values; no Page callback is stored.
#[derive(Default)]
pub struct DocumentCSSOMSource {
    detached: std::cell::Cell<bool>,
    retained: std::cell::RefCell<Option<DocumentCSSOMOwner>>,
    referrer_policy:
        std::cell::RefCell<Option<foundation::referrer_policy::ReferrerPolicyReadHandle>>,
}
pub(crate) struct DocumentCSSOMOwner {
    identity: dom::persistent_document::DOMOwnerHandle,
    execution: Rc<std::cell::RefCell<Weak<ProductionCSSExecutionContext>>>,
    active_execution: Option<Rc<ProductionCSSExecutionContext>>,
    execution_attached: Rc<std::cell::Cell<bool>>,
    document: DocumentHandle<Backend>,
    element_sheet: Option<(u64, Rc<StyleSheetContents<Backend>>)>,
}
/// A synchronous lease of the source ElementSheet and current Window context.
/// Cloning pins their original owners, never a declaration or stylesheet copy.
#[derive(Clone)]
pub struct DocumentCSSOMContext {
    document_identity: u128,
    execution: Option<Rc<ProductionCSSExecutionContext>>,
    document: DocumentHandle<Backend>,
    element_sheet: Rc<StyleSheetContents<Backend>>,
    is_html_element: bool,
    in_quirks_mode: bool,
}
impl DocumentCSSOMContext {
    pub fn ExecutionContext(&self) -> Option<Rc<ProductionCSSExecutionContext>> {
        self.execution.clone()
    }
    pub fn DocumentIdentity(&self) -> u128 {
        self.document_identity
    }
    pub(crate) fn ParserContext(&self) -> &Rc<CSSParserContext<Backend>> {
        self.element_sheet.ParserContext()
    }
    pub(crate) fn InlineMode(&self) -> CSSParserMode {
        if self.is_html_element && !self.in_quirks_mode {
            CSSParserMode::kHTMLStandardMode
        } else {
            CSSParserMode::kHTMLQuirksMode
        }
    }
    pub(crate) fn ParseInline(
        &self,
        text: &String,
    ) -> Rc<
        crate::css_property_value_set::CSSPropertyValueSet<
            crate::production_css_value::ProductionCSSValueDispatch,
        >,
    > {
        CSSParserImpl::<Backend>::ParseInlineStyleDeclarationForElement(
            text,
            &mut InlineElement {
                source: self.clone(),
                sheet: self.element_sheet.clone(),
            },
        )
    }
    pub fn EnsurePropertySetDeclaration(
        &self,
        properties: &Rc<
            std::cell::RefCell<
                crate::css_property_value_set::MutableCSSPropertyValueSet<
                    crate::production_css_value::ProductionCSSValueDispatch,
                >,
            >,
        >,
    ) -> Rc<crate::production_css_value::ProductionCSSStyleDeclaration> {
        crate::css_property_value_set::MutableCSSPropertyValueSet::EnsureCSSStyleDeclaration(
            properties,
            self.execution.as_ref(),
        )
    }
    /// A PropertySetCSSStyleDeclaration has no ParentStyleSheet. Source
    /// ParseAndSetProperty forwards secure mode, while the absent context sheet
    /// selects CSSParser.cc:165-167's parser-mode context (not ElementSheet).
    pub fn SetProperty(
        &self,
        declaration: &crate::production_css_value::ProductionCSSStyleDeclaration,
        id: CSSPropertyID,
        value: &String,
        important: bool,
    ) -> crate::css_property_value_set::SetResult {
        use crate::css_property_value_set::SetResult;
        assert_ne!(
            id,
            CSSPropertyID::kVariable,
            "custom properties require their actual name"
        );
        let owner = declaration.PropertySetOwner();
        let mut set = owner.borrow_mut();
        if value.empty() {
            return if set.RemoveProperty(crate::css_property_names::ResolveCSSPropertyID(id), None)
            {
                SetResult::kChangedPropertySet
            } else {
                SetResult::kUnchanged
            };
        }
        if let (Some(original), Some(current)) =
            (declaration.GetExecutionContext(), self.execution.as_ref())
        {
            assert!(
                Rc::ptr_eq(&original, current),
                "property declaration retains its original ExecutionContext"
            );
        }
        let context = CSSParserContext::<Backend>::FromMode(
            set.CssParserMode(),
            self.execution
                .as_ref()
                .map_or(SecureContextMode::kInsecureContext, |context| {
                    context.GetSecureContextMode()
                }),
            None,
        );
        CSSParserImpl::<Backend>::ParseValue(
            &mut set,
            id,
            foundation::StringView::from(value),
            important,
            Some(&context),
        )
    }
}
pub struct InlineElement {
    source: DocumentCSSOMContext,
    sheet: Rc<StyleSheetContents<Backend>>,
}
impl CSSParserImplInlineElementBackend for Backend {
    type Element = InlineElement;
    fn ElementDocument(element: &InlineElement) -> DocumentHandle<Self> {
        element.source.document.clone()
    }
    fn ElementSheetParserContext(element: &InlineElement) -> Rc<CSSParserContext<Self>> {
        element.sheet.ParserContext().clone()
    }
    fn IsHTMLElement(element: &InlineElement) -> bool {
        element.source.is_html_element
    }
    fn ElementDocumentInQuirksMode(element: &InlineElement) -> bool {
        element.source.in_quirks_mode
    }
    fn ElementSheetContents(element: &mut InlineElement) -> &mut Rc<StyleSheetContents<Self>> {
        &mut element.sheet
    }
}
impl DocumentStyleEngine {
    pub fn CSSOMSource(&self) -> Rc<DocumentCSSOMSource> {
        self.cssom_document.clone()
    }
    pub fn CSSOMContextForElement(
        &self,
        document: &dom::Document,
        element: usize,
        policy: CSSReferrerPolicy,
        world: Option<Rc<ProductionCSSDOMWrapperWorld>>,
    ) -> DocumentCSSOMContext {
        self.cssom_document
            .CSSOMContextForElement(document, element, policy, world)
    }
}
impl DocumentCSSOMSource {
    pub(crate) fn DidDetach(&self) {
        // Document::elem_sheet_ is not a StyleResolver member. Keep that
        // original compiled owner for later CSSOM on this detached Document.
        // Only the active Window/Style lease and fetcher binding are released.
        if let Some(owner) = self.retained.borrow_mut().as_mut() {
            owner.execution_attached.set(false);
            owner.active_execution.take();
        }
        self.referrer_policy.borrow_mut().take();
        self.detached.set(true);
    }
    /// Bind the current fetcher's original slot, without installing an updater
    /// or a second copy of the execution-context referrer policy.
    pub fn BindReferrerPolicy(
        &self,
        policy: foundation::referrer_policy::ReferrerPolicyReadHandle,
    ) {
        assert!(
            !self.detached.get(),
            "detached Style owner cannot reconnect a fetcher"
        );
        *self.referrer_policy.borrow_mut() = Some(policy);
    }
    pub(crate) fn ContextForCurrentElement(
        &self,
        document: &dom::Document,
        element: usize,
    ) -> Option<DocumentCSSOMContext> {
        if self.detached.get() || !document.HasDOMWindow() {
            return None;
        }
        let policy = self.referrer_policy.borrow().as_ref()?.Get();
        Some(self.CSSOMContextForElement(document, element, policy, None))
    }
    /// Called with the live Document before a CSSOM operation. Policy and world
    /// are typed current inputs; Page makes no parser or ownership decisions.
    pub fn CSSOMContextForElement(
        &self,
        document: &dom::Document,
        element: usize,
        policy: CSSReferrerPolicy,
        world: Option<Rc<ProductionCSSDOMWrapperWorld>>,
    ) -> DocumentCSSOMContext {
        // Pure CSSOM is also valid on an inactive Document. This entry never
        // attaches a resolver or enters the style/layout lifecycle.
        let has_execution = !self.detached.get() && document.HasDOMWindow();
        let identity = document.RootHandle();
        let secure = if document.SecurityContext().IsSecureContext() {
            SecureContextMode::kSecureContext
        } else {
            SecureContextMode::kInsecureContext
        };
        let mut retained = self.retained.borrow_mut();
        if retained
            .as_ref()
            .is_none_or(|owner| owner.identity != identity)
        {
            let active_execution = has_execution.then(|| {
                ProductionCSSExecutionContext::new(
                    identity.Identity(),
                    document.NativeExecutionContextLifetime(),
                    secure,
                    document.CSSUseCounter(),
                )
            });
            let execution = Rc::new(std::cell::RefCell::new(
                active_execution
                    .as_ref()
                    .map_or_else(Weak::new, Rc::downgrade),
            ));
            let execution_attached = Rc::new(std::cell::Cell::new(has_execution));
            let parser_document: DocumentHandle<Backend> = Rc::new(ParserDocument {
                execution: execution.clone(),
                execution_attached: execution_attached.clone(),
                counters: document.CSSUseCounter(),
                is_for_markup_sanitization: document.IsForMarkupSanitization(),
            });
            *retained = Some(DocumentCSSOMOwner {
                identity,
                execution,
                active_execution,
                execution_attached,
                document: parser_document,
                element_sheet: None,
            });
        }
        let owner = retained.as_mut().unwrap();
        if has_execution && owner.active_execution.is_none() {
            let execution = ProductionCSSExecutionContext::new(
                identity.Identity(),
                document.NativeExecutionContextLifetime(),
                secure,
                document.CSSUseCounter(),
            );
            *owner.execution.borrow_mut() = Rc::downgrade(&execution);
            owner.execution_attached.set(true);
            owner.active_execution = Some(execution);
        }
        if let Some(execution) = owner.active_execution.as_ref() {
            execution.SynchronizeSecureContextMode(secure);
        }
        let outgoing = if document.SecurityContext().Origin().IsOpaque() {
            String::from("")
        } else {
            StrippedReferrer(document.DocumentURL())
        };
        let snapshot = DocumentSnapshot::<Backend> {
            document: owner.document.clone(),
            base_url: SourceURL::Value(String::from(document.BaseURL().AsStr())),
            in_quirks_mode: document.GetCompatibilityMode()
                == dom::persistent_document::CompatibilityMode::kQuirks,
            is_html_document: document.DocumentType(document.Root())
                == dom::persistent_document::DOMDocumentType::kHTML,
            referrer_policy: policy,
            execution_context: has_execution.then_some(DocumentExecutionContextSnapshot {
                outgoing_referrer: outgoing,
                secure_context_mode: secure,
                world,
            }),
        };
        let current = Rc::new(CSSParserContext::FromDocument(&snapshot));
        let revision = document.BaseURLRevision();
        if owner
            .element_sheet
            .as_ref()
            .is_none_or(|(old, _)| *old != revision)
        {
            owner.element_sheet = Some((
                revision,
                StyleSheetContents::<Backend>::new(current.clone(), String::new(), None),
            ));
        }
        DocumentCSSOMContext {
            document_identity: identity.Identity(),
            execution: if has_execution {
                owner.execution.borrow().upgrade()
            } else {
                None
            },
            document: owner.document.clone(),
            element_sheet: owner.element_sheet.as_ref().unwrap().1.clone(),
            is_html_element: document.Node(element).Namespace()
                == dom::persistent_document::DOMNamespace::kHTML,
            in_quirks_mode: snapshot.in_quirks_mode,
        }
    }
}
/// LocalDOMWindow.cc:583-608, KURL::StrippedForUseAsReferrer. The SecurityOrigin
/// gate above is the original Window gate, independent of the mutable URL.
pub(crate) fn StrippedReferrer(value: &str) -> String {
    let Ok(mut url) = url::Url::parse(value) else {
        return String::from("");
    };
    if !matches!(url.scheme(), "http" | "https") {
        return String::from("");
    }
    let _ = url.set_username("");
    let _ = url.set_password(None);
    url.set_fragment(None);
    String::from(url.as_str())
}

impl DocumentStyleEngine {
    /// StyleElement::CreateSheet / CSSParserContext(document): keep original
    /// UTF-16 and the Document's retained parser/execution/counter owner.
    /// In particular IsForMarkupSanitization prohibits @import in the parser.
    pub fn ParseStyleElementSheet(
        &self,
        document: &dom::Document,
        element: usize,
        text: &[u16],
    ) -> Option<cssom::CSSStyleSheet> {
        // StyleElement.cc:60–62,244–246: only empty/text/css is a classic
        // stylesheet. The module tag runtime feature is not admitted here.
        let node = document.Node(element);
        let kind = node.AttributeValueUTF16("type");
        if kind.as_deref().is_some_and(|value| {
            !value.is_empty()
                && !value.iter().copied().eq("text/css".bytes().map(u16::from))
                && !value
                    .iter()
                    .copied()
                    .map(|c| {
                        if c <= 127 {
                            (c as u8).to_ascii_lowercase() as u16
                        } else {
                            c
                        }
                    })
                    .eq("text/css".bytes().map(u16::from))
        }) {
            return None;
        }
        let context = self.CSSOMContextForElement(
            document,
            element,
            foundation::referrer_policy::ReferrerPolicy::default(),
            None,
        );
        let parsed = Rc::new(crate::production_style_sheet::ParseStyleSheetWithContext(
            &String::from_utf16(text),
            context.ParserContext().clone(),
            false,
        ));
        let mut sheet = crate::production_style_sheet_projection::ProjectStyleSheet(&parsed);
        sheet.AttachParsedBacking(&parsed.source.Utf8(), parsed.clone());
        let media = node
            .FindAttribute("media")
            .map_or_else(std::string::String::new, |a| a.value.clone());
        // StyleElement::CreateSheet/CSSStyleSheet::SetMediaQueries: keep the
        // owning element's outer media query on the canonical CSSOM sheet and
        // project it onto every parsed rule through the existing metadata API.
        sheet.SetLinkMetadata(node.Id(), false, false, std::string::String::new(), media);
        Some(sheet)
    }
}
