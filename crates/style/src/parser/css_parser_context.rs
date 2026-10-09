// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// cpp: third_party/blink/renderer/core/css/parser/css_parser_context.h:31-180
// cpp: third_party/blink/renderer/core/css/parser/css_parser_context.cc:25-280
// Browser objects and KURL/TextEncoding/Referrer operations are explicit input
// snapshots/interfaces below; no URL parser or browser defaults are invented.
// Oilpan Trace/Member storage is expressed by strong world and weak document
// ownership. Production adapters for those interfaces remain external work.

#![allow(non_camel_case_types, non_snake_case)]

use super::css_parser_mode::{CSSParserMode, IsUseCounterEnabledForMode};
use crate::css_resource_fetch_restriction::ResourceFetchRestriction;
use foundation::{CSSPropertyID, String};
use std::any::{Any, TypeId};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};

// cpp: third_party/blink/renderer/core/execution_context/security_context.h:58
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SecureContextMode {
    kInsecureContext,
    kSecureContext,
}

// Required platform operations. URL results and encoding/referrer equality are
// supplied by real platform adapters rather than approximated in this module.
pub trait CSSParserContextPlatform: 'static {
    type URL: Clone + Eq;
    type TextEncoding: Clone + Eq;
    type Referrer: Clone + Eq;
    type ReferrerPolicy: Copy;
    type DOMWrapperWorld: 'static;
    type ExecutionContext: 'static;
    type WebFeature: Copy;
    type WebDXFeature: Copy;

    fn NullURL() -> Self::URL;
    fn EmptyURL() -> Self::URL;
    fn EmptyTextEncoding() -> Self::TextEncoding;
    fn IsEncodingValid(encoding: &Self::TextEncoding) -> bool;
    fn EmptyReferrer() -> Self::Referrer;
    fn MakeReferrer(referrer: String, policy: Self::ReferrerPolicy) -> Self::Referrer;
    fn StrippedForUseAsReferrer(url: &Self::URL) -> String;
    fn ResolveURL(
        base: &Self::URL,
        url: &String,
        encoding: Option<&Self::TextEncoding>,
    ) -> Self::URL;
    fn CSSParserIgnoreCharsetForURLsEnabled() -> bool;
    fn CountDeprecation(context: Option<&Rc<Self::ExecutionContext>>, feature: Self::WebFeature);
}

// Exact calls made to the weak Document handle; no DOM dependency is required.
pub trait CSSParserContextDocument<P: CSSParserContextPlatform>: 'static {
    fn CountUse(&self, feature: P::WebFeature);
    fn CountWebDXFeature(&self, feature: P::WebDXFeature);
    fn CountProperty(&self, property: CSSPropertyID);
    fn GetExecutionContext(&self) -> Option<Rc<P::ExecutionContext>>;
    fn IsForMarkupSanitization(&self) -> bool;
}

pub type DocumentHandle<P> = Rc<dyn CSSParserContextDocument<P>>;
pub type WeakDocumentHandle<P> = Weak<dyn CSSParserContextDocument<P>>;

// The sheet adapter implements the actual SingleOwnerDocument operation. Both
// CSSStyleSheet and StyleSheetContents constructor overloads use this boundary.
pub trait SingleOwnerDocument<P: CSSParserContextPlatform> {
    fn SingleOwnerDocument(&self) -> Option<DocumentHandle<P>>;
}

// Values read from document.GetExecutionContext() during construction.
pub struct DocumentExecutionContextSnapshot<P: CSSParserContextPlatform> {
    pub outgoing_referrer: String,
    pub secure_context_mode: SecureContextMode,
    pub world: Option<Rc<P::DOMWrapperWorld>>,
}

// All document construction inputs are explicit, including the test-only
// absence of its execution context. The handle is separately retained weakly.
pub struct DocumentSnapshot<P: CSSParserContextPlatform> {
    pub document: DocumentHandle<P>,
    pub base_url: P::URL,
    pub in_quirks_mode: bool,
    pub is_html_document: bool,
    pub referrer_policy: P::ReferrerPolicy,
    pub execution_context: Option<DocumentExecutionContextSnapshot<P>>,
}

// local_dom_window_document is populated only for a LocalDOMWindow context;
// worker contexts pass None, exactly as the source's IsA<LocalDOMWindow> branch.
pub struct ExecutionContextSnapshot<P: CSSParserContextPlatform> {
    pub url: P::URL,
    pub referrer_policy: P::ReferrerPolicy,
    pub secure_context_mode: SecureContextMode,
    pub world: Option<Rc<P::DOMWrapperWorld>>,
    pub local_dom_window_document: Option<DocumentHandle<P>>,
}

// cpp: css_parser_context.h:153-177
pub struct CSSParserContext<P: CSSParserContextPlatform> {
    base_url: P::URL,
    world: Option<Rc<P::DOMWrapperWorld>>,
    origin_clean: bool,
    mode: Cell<CSSParserMode>,
    referrer: P::Referrer,
    is_ad_related: Cell<bool>,
    is_html_document: bool,
    secure_context_mode: SecureContextMode,
    charset: P::TextEncoding,
    document: Option<WeakDocumentHandle<P>>,
    resource_fetch_restriction: ResourceFetchRestriction,
}

impl<P: CSSParserContextPlatform> CSSParserContext<P> {
    // cpp: css_parser_context.cc:138-161
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        base_url: P::URL,
        origin_clean: bool,
        charset: P::TextEncoding,
        mode: CSSParserMode,
        referrer: P::Referrer,
        is_html_document: bool,
        secure_context_mode: SecureContextMode,
        world: Option<Rc<P::DOMWrapperWorld>>,
        document: Option<&DocumentHandle<P>>,
        resource_fetch_restriction: ResourceFetchRestriction,
    ) -> Self {
        let charset = if P::CSSParserIgnoreCharsetForURLsEnabled() {
            P::EmptyTextEncoding()
        } else {
            charset
        };
        Self {
            base_url,
            world,
            origin_clean,
            mode: Cell::new(mode),
            referrer,
            is_ad_related: Cell::new(false),
            is_html_document,
            secure_context_mode,
            charset,
            document: document.map(Rc::downgrade),
            resource_fetch_restriction,
        }
    }
    // cpp: css_parser_context.cc:25-35
    pub fn FromStyleSheetOwner(other: &Self, style_sheet: &impl SingleOwnerDocument<P>) -> Self {
        Self::CopyWithDocument(other, style_sheet.SingleOwnerDocument().as_ref())
    }
    // cpp: css_parser_context.cc:37-50
    pub fn CopyWithDocument(other: &Self, document: Option<&DocumentHandle<P>>) -> Self {
        let copy = Self::new(
            other.base_url.clone(),
            other.origin_clean,
            other.charset.clone(),
            other.Mode(),
            other.referrer.clone(),
            other.is_html_document,
            other.secure_context_mode,
            other.world.clone(),
            document,
            other.resource_fetch_restriction,
        );
        copy.is_ad_related.set(other.IsAdRelated());
        copy
    }
    // cpp: css_parser_context.cc:52-69
    pub fn CopyForImport(
        other: &Self,
        base_url: P::URL,
        origin_clean: bool,
        referrer: P::Referrer,
        charset: P::TextEncoding,
        document: Option<&DocumentHandle<P>>,
    ) -> Self {
        let copy = Self::new(
            base_url,
            origin_clean,
            charset,
            other.Mode(),
            referrer,
            other.is_html_document,
            other.secure_context_mode,
            other.world.clone(),
            document,
            other.resource_fetch_restriction,
        );
        copy.is_ad_related.set(other.IsAdRelated());
        copy
    }
    // cpp: css_parser_context.cc:71-83
    pub fn FromMode(
        mode: CSSParserMode,
        secure: SecureContextMode,
        document: Option<&DocumentHandle<P>>,
    ) -> Self {
        Self::new(
            P::NullURL(),
            true,
            P::EmptyTextEncoding(),
            mode,
            P::EmptyReferrer(),
            false,
            secure,
            None,
            document,
            ResourceFetchRestriction::kNone,
        )
    }
    // cpp: css_parser_context.cc:85-98
    pub fn FromDocument(document: &DocumentSnapshot<P>) -> Self {
        Self::FromDocumentWithBaseURL(document, document.base_url.clone())
    }
    pub fn FromDocumentWithBaseURL(document: &DocumentSnapshot<P>, base_url: P::URL) -> Self {
        let outgoing = document
            .execution_context
            .as_ref()
            .map_or_else(String::default, |context| context.outgoing_referrer.clone());
        let referrer = P::MakeReferrer(outgoing, document.referrer_policy);
        Self::FromDocumentWithOptions(
            document,
            base_url,
            true,
            referrer,
            P::EmptyTextEncoding(),
            ResourceFetchRestriction::kNone,
        )
    }
    // cpp: css_parser_context.cc:100-121
    pub fn FromDocumentWithOptions(
        document: &DocumentSnapshot<P>,
        base_url: P::URL,
        origin_clean: bool,
        referrer: P::Referrer,
        charset: P::TextEncoding,
        restriction: ResourceFetchRestriction,
    ) -> Self {
        let (secure, world) = match &document.execution_context {
            Some(context) => (context.secure_context_mode, context.world.clone()),
            None => (SecureContextMode::kInsecureContext, None),
        };
        Self::new(
            base_url,
            origin_clean,
            charset,
            if document.in_quirks_mode {
                CSSParserMode::kHTMLQuirksMode
            } else {
                CSSParserMode::kHTMLStandardMode
            },
            referrer,
            document.is_html_document,
            secure,
            world,
            Some(&document.document),
            restriction,
        )
    }
    // cpp: css_parser_context.cc:123-136
    pub fn FromExecutionContext(context: &ExecutionContextSnapshot<P>) -> Self {
        Self::new(
            context.url.clone(),
            true,
            P::EmptyTextEncoding(),
            CSSParserMode::kHTMLStandardMode,
            P::MakeReferrer(
                P::StrippedForUseAsReferrer(&context.url),
                context.referrer_policy,
            ),
            true,
            context.secure_context_mode,
            context.world.clone(),
            context.local_dom_window_document.as_ref(),
            ResourceFetchRestriction::kNone,
        )
    }
    // cpp: css_parser_context.h:81-109
    pub fn Mode(&self) -> CSSParserMode {
        self.mode.get()
    }
    pub fn BaseURL(&self) -> &P::URL {
        &self.base_url
    }
    pub fn Charset(&self) -> &P::TextEncoding {
        &self.charset
    }
    pub fn GetReferrer(&self) -> &P::Referrer {
        &self.referrer
    }
    pub fn IsAdRelated(&self) -> bool {
        self.is_ad_related.get()
    }
    pub fn IsHTMLDocument(&self) -> bool {
        self.is_html_document
    }
    pub fn ResourceFetchRestriction(&self) -> ResourceFetchRestriction {
        self.resource_fetch_restriction
    }
    pub fn SetMode(&self, mode: CSSParserMode) {
        self.mode.set(mode);
    }
    pub fn GetMode(&self) -> CSSParserMode {
        self.Mode()
    }
    pub fn SetIsAdRelated(&self) {
        self.is_ad_related.set(true);
    }
    pub fn GetSecureContextMode(&self) -> SecureContextMode {
        self.secure_context_mode
    }
    // cpp: css_parser_context.cc:195-201
    pub fn IsOriginClean(&self) -> bool {
        self.origin_clean
    }
    pub fn IsSecureContext(&self) -> bool {
        self.secure_context_mode == SecureContextMode::kSecureContext
    }
    // cpp: css_parser_context.cc:203-218
    pub fn CompleteURL(&self, url: &String) -> P::URL {
        if url.IsNull() {
            return P::NullURL();
        }
        if !P::IsEncodingValid(self.Charset()) {
            return P::ResolveURL(self.BaseURL(), url, None);
        }
        P::ResolveURL(self.BaseURL(), url, Some(self.Charset()))
    }
    pub fn CompleteNonEmptyURL(&self, url: &String) -> P::URL {
        if url.empty() && !url.IsNull() {
            return P::EmptyURL();
        }
        self.CompleteURL(url)
    }
    // cpp: css_parser_context.h:115-117
    pub fn IsUseCounterRecordingEnabled(&self) -> bool {
        self.GetDocument().is_some() && IsUseCounterEnabledForMode(self.Mode())
    }
    // cpp: css_parser_context.cc:220-242
    pub fn CountWebFeature(&self, feature: P::WebFeature) {
        if self.IsUseCounterRecordingEnabled() {
            if let Some(document) = self.GetDocument() {
                document.CountUse(feature);
            }
        }
    }
    pub fn CountWebDXFeature(&self, feature: P::WebDXFeature) {
        if self.IsUseCounterRecordingEnabled() {
            if let Some(document) = self.GetDocument() {
                document.CountWebDXFeature(feature);
            }
        }
    }
    pub fn CountDeprecation(&self, feature: P::WebFeature) {
        if self.IsUseCounterRecordingEnabled() {
            P::CountDeprecation(self.GetExecutionContext().as_ref(), feature);
        }
    }
    pub fn CountProperty(&self, property: CSSPropertyID) {
        if self.IsUseCounterRecordingEnabled() {
            if let Some(document) = self.GetDocument() {
                document.CountProperty(property);
            }
        }
    }
    // cpp: css_parser_context.cc:244-260
    pub fn IsDocumentHandleEqual(&self, other: Option<&DocumentHandle<P>>) -> bool {
        match (self.GetDocument().as_ref(), other) {
            (None, None) => true,
            (Some(a), Some(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
    pub fn GetDocument(&self) -> Option<DocumentHandle<P>> {
        self.document.as_ref().and_then(Weak::upgrade)
    }
    pub fn GetExecutionContext(&self) -> Option<Rc<P::ExecutionContext>> {
        self.GetDocument()
            .and_then(|document| document.GetExecutionContext())
    }
    pub fn IsForMarkupSanitization(&self) -> bool {
        self.GetDocument()
            .is_some_and(|document| document.IsForMarkupSanitization())
    }
    // cpp: css_parser_context.h:123
    pub fn JavascriptWorld(&self) -> Option<&Rc<P::DOMWrapperWorld>> {
        self.world.as_ref()
    }
    // cpp: css_parser_context.cc:262-280
    pub fn InElementContext(&self) -> bool {
        use CSSParserMode::*;
        match self.Mode() {
            kCSSFontFaceRuleMode
            | kCSSPropertyRuleMode
            | kCSSFontPaletteValuesRuleMode
            | kCSSCounterStyleRuleMode => false,
            kHTMLStandardMode
            | kHTMLQuirksMode
            | kSVGAttributeMode
            | kCSSKeyframeRuleMode
            | kCSSPositionTryRuleMode
            | kCSSFunctionDescriptorsMode
            | kUASheetMode => true,
            kNumCSSParserModes => panic!("NOTREACHED: parser mode sentinel"),
        }
    }
}

// cpp: css_parser_context.cc:163-171
impl<P: CSSParserContextPlatform> PartialEq for CSSParserContext<P> {
    fn eq(&self, other: &Self) -> bool {
        let world_equal = match (&self.world, &other.world) {
            (None, None) => true,
            (Some(a), Some(b)) => Rc::ptr_eq(a, b),
            _ => false,
        };
        self.base_url == other.base_url
            && self.origin_clean == other.origin_clean
            && self.charset == other.charset
            && self.Mode() == other.Mode()
            && self.IsAdRelated() == other.IsAdRelated()
            && self.is_html_document == other.is_html_document
            && self.secure_context_mode == other.secure_context_mode
            && world_equal
            && self.referrer == other.referrer
            && self.resource_fetch_restriction == other.resource_fetch_restriction
    }
}
impl<P: CSSParserContextPlatform> Eq for CSSParserContext<P> {}

// cpp: css_parser_context.h:136-146
pub struct ParserModeOverridingScope<'a, P: CSSParserContextPlatform> {
    context: &'a CSSParserContext<P>,
    previous: CSSParserMode,
}
impl<'a, P: CSSParserContextPlatform> ParserModeOverridingScope<'a, P> {
    pub fn new(context: &'a CSSParserContext<P>, mode: CSSParserMode) -> Self {
        let previous = context.mode.replace(mode);
        Self { context, previous }
    }
}
impl<P: CSSParserContextPlatform> Drop for ParserModeOverridingScope<'_, P> {
    fn drop(&mut self) {
        self.context.mode.set(self.previous);
    }
}

// cpp: css_parser_context.cc:175-193
thread_local! {
    static STRICT_CONTEXT_POOL: RefCell<HashMap<(TypeId, bool), Box<dyn Any>>> = RefCell::new(HashMap::new());
}
pub fn StrictCSSParserContext<P: CSSParserContextPlatform>(
    secure: SecureContextMode,
) -> Rc<CSSParserContext<P>> {
    STRICT_CONTEXT_POOL.with(|pool| {
        let mut pool = pool.borrow_mut();
        let entry = pool
            .entry((
                TypeId::of::<P>(),
                secure == SecureContextMode::kSecureContext,
            ))
            .or_insert_with(|| {
                Box::new(Rc::new(CSSParserContext::<P>::FromMode(
                    CSSParserMode::kHTMLStandardMode,
                    secure,
                    None,
                )))
            });
        entry
            .downcast_ref::<Rc<CSSParserContext<P>>>()
            .expect("platform-specific strict context")
            .clone()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Clone, Debug, PartialEq, Eq)]
    enum URL {
        Null,
        Empty,
        Base(&'static str),
        Resolve(Box<URL>, String, Option<&'static str>),
    }
    struct Platform;
    thread_local! {
        static IGNORE_CHARSET: Cell<bool> = const { Cell::new(false) };
        static DEPRECATIONS: RefCell<Vec<(Option<u32>, u32)>> = const { RefCell::new(Vec::new()) };
    }
    impl CSSParserContextPlatform for Platform {
        type URL = URL;
        type TextEncoding = Option<&'static str>;
        type Referrer = (String, u8);
        type ReferrerPolicy = u8;
        type DOMWrapperWorld = u32;
        type ExecutionContext = u32;
        type WebFeature = u32;
        type WebDXFeature = u32;
        fn NullURL() -> URL {
            URL::Null
        }
        fn EmptyURL() -> URL {
            URL::Empty
        }
        fn EmptyTextEncoding() -> Self::TextEncoding {
            None
        }
        fn IsEncodingValid(encoding: &Self::TextEncoding) -> bool {
            encoding.is_some()
        }
        fn EmptyReferrer() -> Self::Referrer {
            (String::default(), 0)
        }
        fn MakeReferrer(value: String, policy: u8) -> Self::Referrer {
            (value, policy)
        }
        fn StrippedForUseAsReferrer(_: &URL) -> String {
            String::from("explicitly stripped input")
        }
        // This test adapter records dispatch arguments; it does not parse URLs.
        fn ResolveURL(base: &URL, url: &String, encoding: Option<&Self::TextEncoding>) -> URL {
            URL::Resolve(
                Box::new(base.clone()),
                url.clone(),
                encoding.copied().flatten(),
            )
        }
        fn CSSParserIgnoreCharsetForURLsEnabled() -> bool {
            IGNORE_CHARSET.with(Cell::get)
        }
        fn CountDeprecation(context: Option<&Rc<u32>>, feature: u32) {
            DEPRECATIONS.with(|events| events.borrow_mut().push((context.map(|c| **c), feature)));
        }
    }
    #[derive(Debug, PartialEq, Eq)]
    enum Event {
        Web(u32),
        DX(u32),
        Property(CSSPropertyID),
    }
    struct Document {
        events: RefCell<Vec<Event>>,
        execution: Option<Rc<u32>>,
        sanitization: bool,
    }
    impl CSSParserContextDocument<Platform> for Document {
        fn CountUse(&self, feature: u32) {
            self.events.borrow_mut().push(Event::Web(feature));
        }
        fn CountWebDXFeature(&self, feature: u32) {
            self.events.borrow_mut().push(Event::DX(feature));
        }
        fn CountProperty(&self, property: CSSPropertyID) {
            self.events.borrow_mut().push(Event::Property(property));
        }
        fn GetExecutionContext(&self) -> Option<Rc<u32>> {
            self.execution.clone()
        }
        fn IsForMarkupSanitization(&self) -> bool {
            self.sanitization
        }
    }
    fn document() -> Rc<Document> {
        Rc::new(Document {
            events: RefCell::new(vec![]),
            execution: Some(Rc::new(7)),
            sanitization: true,
        })
    }
    fn context(
        document: Option<&DocumentHandle<Platform>>,
        world: Option<Rc<u32>>,
    ) -> CSSParserContext<Platform> {
        CSSParserContext::new(
            URL::Base("base"),
            false,
            Some("legacy"),
            CSSParserMode::kHTMLQuirksMode,
            (String::from("referrer"), 2),
            true,
            SecureContextMode::kSecureContext,
            world,
            document,
            ResourceFetchRestriction::kOnlyDataUrls,
        )
    }
    #[test]
    fn copy_equality_weak_document_and_world_identity() {
        let document = document();
        let handle: DocumentHandle<Platform> = document.clone();
        let world = Rc::new(3);
        let source = context(Some(&handle), Some(world.clone()));
        source.SetIsAdRelated();
        let copy = CSSParserContext::CopyWithDocument(&source, None);
        assert!(source == copy); // The source equality deliberately excludes document_.
        assert!(copy.IsAdRelated());
        assert!(source.IsDocumentHandleEqual(Some(&handle)));
        assert!(!copy.IsDocumentHandleEqual(Some(&handle)));
        assert!(copy.IsDocumentHandleEqual(None));
        let different_world = context(Some(&handle), Some(Rc::new(3)));
        different_world.SetIsAdRelated();
        assert!(source != different_world); // Same contents do not mean the same world.
        assert!(source.IsForMarkupSanitization());
        assert_eq!(*source.GetExecutionContext().unwrap(), 7);
        let import = CSSParserContext::CopyForImport(
            &source,
            URL::Base("import"),
            true,
            (String::from("new referrer"), 9),
            Some("import encoding"),
            Some(&handle),
        );
        assert_eq!(import.BaseURL(), &URL::Base("import"));
        assert!(import.IsOriginClean());
        assert_eq!(import.Charset(), &Some("import encoding"));
        assert_eq!(import.Mode(), source.Mode());
        assert!(import.IsAdRelated());
        assert_eq!(
            import.ResourceFetchRestriction(),
            ResourceFetchRestriction::kOnlyDataUrls
        );
        drop(handle);
        drop(document);
        assert!(source.GetDocument().is_none());
        assert!(!source.IsUseCounterRecordingEnabled());
        assert!(!source.IsForMarkupSanitization());
        assert!(source.IsDocumentHandleEqual(None));
    }
    #[test]
    fn url_dispatch_distinguishes_null_empty_and_charset_feature() {
        IGNORE_CHARSET.with(|flag| flag.set(false));
        let context = context(None, None);
        let null = String::default();
        let empty = String::from("");
        assert_eq!(context.CompleteURL(&null), URL::Null);
        assert_eq!(context.CompleteNonEmptyURL(&null), URL::Null);
        assert_eq!(context.CompleteNonEmptyURL(&empty), URL::Empty);
        assert_eq!(
            context.CompleteURL(&empty),
            URL::Resolve(Box::new(URL::Base("base")), empty, Some("legacy"))
        );
        let input = String::from("relative");
        assert_eq!(
            context.CompleteURL(&input),
            URL::Resolve(Box::new(URL::Base("base")), input.clone(), Some("legacy"))
        );
        IGNORE_CHARSET.with(|flag| flag.set(true));
        let ignored = CSSParserContext::CopyWithDocument(&context, None);
        assert_eq!(ignored.Charset(), &None);
        assert_eq!(
            ignored.CompleteURL(&input),
            URL::Resolve(Box::new(URL::Base("base")), input, None)
        );
        IGNORE_CHARSET.with(|flag| flag.set(false));
    }
    #[test]
    fn counters_obey_mode_document_and_nested_mode_restore() {
        let document = document();
        let handle: DocumentHandle<Platform> = document.clone();
        let context = context(Some(&handle), None);
        context.CountWebFeature(1);
        context.CountWebDXFeature(2);
        context.CountProperty(CSSPropertyID::kColor);
        context.CountDeprecation(3);
        assert_eq!(
            &*document.events.borrow(),
            &[
                Event::Web(1),
                Event::DX(2),
                Event::Property(CSSPropertyID::kColor)
            ]
        );
        DEPRECATIONS.with(|events| assert_eq!(events.borrow().last(), Some(&(Some(7), 3))));
        {
            let _scope = ParserModeOverridingScope::new(&context, CSSParserMode::kUASheetMode);
            assert!(!context.IsUseCounterRecordingEnabled());
            context.CountWebFeature(4);
            context.CountWebDXFeature(5);
            context.CountProperty(CSSPropertyID::kWidth);
            context.CountDeprecation(6);
            {
                let _nested =
                    ParserModeOverridingScope::new(&context, CSSParserMode::kCSSFontFaceRuleMode);
                assert!(context.IsUseCounterRecordingEnabled());
                assert!(!context.InElementContext());
            }
            assert_eq!(context.Mode(), CSSParserMode::kUASheetMode);
        }
        assert_eq!(context.Mode(), CSSParserMode::kHTMLQuirksMode);
        assert_eq!(document.events.borrow().len(), 3);
        let documentless = CSSParserContext::CopyWithDocument(&context, None);
        documentless.CountWebFeature(8);
        documentless.CountDeprecation(9);
        assert_eq!(document.events.borrow().len(), 3);
    }
    #[test]
    fn constructors_and_strict_pool_preserve_source_inputs() {
        let document = document();
        let handle: DocumentHandle<Platform> = document.clone();
        let snapshot = DocumentSnapshot {
            document: handle.clone(),
            base_url: URL::Base("document"),
            in_quirks_mode: true,
            is_html_document: false,
            referrer_policy: 8,
            execution_context: None,
        };
        let context = CSSParserContext::FromDocument(&snapshot);
        assert_eq!(context.Mode(), CSSParserMode::kHTMLQuirksMode);
        assert!(!context.IsHTMLDocument());
        assert!(!context.IsSecureContext());
        assert!(context.GetReferrer().0.IsNull());
        assert_eq!(context.GetReferrer().1, 8);
        assert!(context.JavascriptWorld().is_none());
        let execution = ExecutionContextSnapshot::<Platform> {
            url: URL::Base("worker"),
            referrer_policy: 4,
            secure_context_mode: SecureContextMode::kSecureContext,
            world: Some(Rc::new(9)),
            local_dom_window_document: None,
        };
        let worker = CSSParserContext::FromExecutionContext(&execution);
        assert!(worker.IsHTMLDocument());
        assert!(worker.IsSecureContext());
        assert!(worker.GetDocument().is_none());
        assert_eq!(
            worker.GetReferrer(),
            &(String::from("explicitly stripped input"), 4)
        );
        let secure = StrictCSSParserContext::<Platform>(SecureContextMode::kSecureContext);
        let same = StrictCSSParserContext::<Platform>(SecureContextMode::kSecureContext);
        let insecure = StrictCSSParserContext::<Platform>(SecureContextMode::kInsecureContext);
        assert!(Rc::ptr_eq(&secure, &same));
        assert!(!Rc::ptr_eq(&secure, &insecure));
        assert!(secure.IsSecureContext());
        assert!(!insecure.IsSecureContext());
        assert_eq!(secure.Mode(), CSSParserMode::kHTMLStandardMode);
    }
}
