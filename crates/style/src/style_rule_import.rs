/*
 * (C) 1999-2003 Lars Knoll (knoll@kde.org)
 * (C) 2002-2003 Dirk Mueller (mueller@kde.org)
 * Copyright (C) 2002, 2005, 2006, 2008, 2009, 2010, 2012 Apple Inc.
 * GNU Library General Public License, version 2 or later. See COPYING.LIB.
 */
// Source ledger: effective means nonblank, comment-stripped source lines after
// the exact boilerplate/debug-only exclusions below; braces count as lines.
// style_rule_import.h: physical 153; effective 58; mapped 58; remaining 0.
// style_rule_import.cc: physical 248; effective 157; mapped 157; remaining 0.
// Mapped .h:44-51,54-86,99-100,102-105,115,118-141,146-148.
// Omitted .h:1-43,52,88-98,101,107-114,116-117,142-145,149-153:
// comments, includes/class wrappers, destructor, tracing and debug name.
// Omitted debug-only .h:58; .cc:227.
// Mapped .cc:44-67,71-73,84-246. Omitted .cc:1-43,69,75-82,248:
// includes/namespace wrappers, destructor, and Oilpan tracing.
// External class bodies (Document, ResourceClient, CSSStyleSheetResource,
// ResourceRequest/FetchParameters, integrity and StyleSheetContents) are
// mandatory typed dependencies below, with no default or synthetic behavior.
// This source pair stores MediaQuerySet; it contains no media evaluation or
// result flags. CSS initiator/world/position/referrer reach the real Fetch;
// resource timing remains the loader's operation, as in Chromium.

#![allow(non_snake_case, non_camel_case_types)]

use crate::css_origin_clean::OriginClean;
use crate::parser::css_parser_context::{
    CSSParserContext, CSSParserContextPlatform, DocumentHandle, SecureContextMode,
    StrictCSSParserContext,
};
use crate::style_rule::{LayerName, LayerNameAsString, RuleType};
use crate::style_sheet_contents::RenderBlockingBehavior;
use foundation::String;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

// platform/loader/fetch/cross_origin_attribute_value.h:16-20.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrossOriginAttributeValue {
    kCrossOriginAttributeNotSet,
    kCrossOriginAttributeAnonymous,
    kCrossOriginAttributeUseCredentials,
}

// css_url_data.h:55-58. Preserve null versus empty integrity strings.
pub struct CSSUrlRequestModifiers<P: CSSParserContextPlatform> {
    pub cross_origin: CrossOriginAttributeValue,
    pub integrity: String,
    pub referrer_policy: Option<P::ReferrerPolicy>,
}
impl<P: CSSParserContextPlatform> Clone for CSSUrlRequestModifiers<P> {
    fn clone(&self) -> Self {
        Self {
            cross_origin: self.cross_origin,
            integrity: self.integrity.clone(),
            referrer_policy: self.referrer_policy,
        }
    }
}
impl<P: CSSParserContextPlatform> Default for CSSUrlRequestModifiers<P> {
    fn default() -> Self {
        Self {
            cross_origin: CrossOriginAttributeValue::kCrossOriginAttributeNotSet,
            integrity: String::default(),
            referrer_policy: None,
        }
    }
}

/// Parser-owned portion of StyleRuleImport (.h:54-86,122-141;
/// .cc:44-67). The loader-bound class below retains its full lifecycle; this
/// detached representation exposes a typed loading effect to its caller.
/// It does not pretend to own a ResourceClient, Document or loaded child sheet.
pub struct ParsedStyleRuleImport<P: CSSParserContextPlatform, S, M> {
    pub href: String,
    pub layer: LayerName,
    pub scope: Option<Rc<S>>,
    pub supported: bool,
    pub supports_string: String,
    pub media_queries: Rc<M>,
    pub origin_clean: OriginClean,
    pub modifiers: CSSUrlRequestModifiers<P>,
}
impl<P: CSSParserContextPlatform, S, M> ParsedStyleRuleImport<P, S, M> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        href: String,
        layer: LayerName,
        scope: Option<Rc<S>>,
        supported: bool,
        supports_string: String,
        media_queries: Rc<M>,
        origin_clean: OriginClean,
        modifiers: &CSSUrlRequestModifiers<P>,
    ) -> Self {
        Self {
            href,
            layer,
            scope,
            supported,
            supports_string,
            media_queries,
            origin_clean,
            modifiers: modifiers.clone(),
        }
    }
    pub fn GetType(&self) -> RuleType {
        RuleType::kImport
    }
    pub fn Href(&self) -> String {
        self.href.clone()
    }
    pub fn GetLayerName(&self) -> &LayerName {
        &self.layer
    }
    pub fn IsSupported(&self) -> bool {
        self.supported
    }
    pub fn GetSupportsString(&self) -> String {
        self.supports_string.clone()
    }
    pub fn MediaQueries(&self) -> Rc<M> {
        self.media_queries.clone()
    }
}

/// Exact sheet operations this source uses. An adapter may implement this for
/// the existing StyleSheetContents; no rule-set, parser or browser is replaced.
pub trait StyleRuleImportSheet<B: StyleRuleImportBackend>: 'static {
    fn SingleOwnerDocument(&self) -> Option<Rc<B::Document>>;
    fn ParserContext(&self) -> Rc<CSSParserContext<B::Platform>>;
    fn BaseURL(&self) -> <B::Platform as CSSParserContextPlatform>::URL;
    fn OriginalURL(&self) -> String;
    fn ParentStyleSheet(&self) -> Option<Rc<Self>>;
    fn Charset(&self) -> <B::Platform as CSSParserContextPlatform>::TextEncoding;
    fn ClearOwnerRule(&self);
    fn ParseAuthorStyleSheet(&self, resource: &B::Resource);
    fn IsLoading(&self) -> bool;
    fn NotifyLoadedSheet(&self, resource: &B::Resource);
    fn CheckLoaded(&self);
    fn LoadCompleted(&self) -> bool;
    fn SetToPendingState(&self);
    fn GetRenderBlockingBehavior(&self) -> RenderBlockingBehavior;
}

/// These methods are calls into external Chromium classes, not optional hooks.
/// Fetch must perform the real CSSStyleSheetResource::Fetch, register its
/// resource via client.SetResource, and may call NotifyFinished synchronously.
/// Request and option constructors retain their native defaults (including the
/// below-range initiator position); setters modify exactly the named field.
pub trait StyleRuleImportBackend: Sized + 'static {
    type Platform: CSSParserContextPlatform;
    type StyleSheet: StyleRuleImportSheet<Self>;
    type StyleScope: 'static;
    type MediaQuerySet: 'static;
    type Document: 'static;
    type ResourceFetcher;
    type Resource: 'static;
    type ResourceLoaderOptions;
    type ResourceRequest;
    type FetchParameters;
    type IntegrityMetadataSet;
    type TextPosition: Copy;
    type DevToolsId;

    fn CreateEmptyMediaQuerySet() -> Rc<Self::MediaQuerySet>;
    fn ParserDocument(document: &Rc<Self::Document>) -> DocumentHandle<Self::Platform>;
    fn Fetcher(document: &Self::Document) -> Option<Rc<Self::ResourceFetcher>>;
    fn CompleteURL(
        document: &Self::Document,
        relative: &String,
    ) -> <Self::Platform as CSSParserContextPlatform>::URL;
    fn IsNullURL(url: &<Self::Platform as CSSParserContextPlatform>::URL) -> bool;
    fn EqualIgnoringFragmentIdentifier(
        a: &<Self::Platform as CSSParserContextPlatform>::URL,
        b: &<Self::Platform as CSSParserContextPlatform>::URL,
    ) -> bool;
    fn URLString(url: &<Self::Platform as CSSParserContextPlatform>::URL) -> String;
    fn ReferrerString(referrer: &<Self::Platform as CSSParserContextPlatform>::Referrer) -> String;
    fn ReferrerPolicy(
        referrer: &<Self::Platform as CSSParserContextPlatform>::Referrer,
    ) -> <Self::Platform as CSSParserContextPlatform>::ReferrerPolicy;

    fn NewResourceLoaderOptions(
        world: Option<&Rc<<Self::Platform as CSSParserContextPlatform>::DOMWrapperWorld>>,
    ) -> Self::ResourceLoaderOptions;
    fn SetCSSInitiatorName(options: &mut Self::ResourceLoaderOptions);
    fn SetInitiatorPosition(
        options: &mut Self::ResourceLoaderOptions,
        position: Self::TextPosition,
    );
    fn SetInitiatorReferrer(options: &mut Self::ResourceLoaderOptions, referrer: String);
    fn NewResourceRequest(
        url: <Self::Platform as CSSParserContextPlatform>::URL,
    ) -> Self::ResourceRequest;
    fn SetReferrerPolicy(
        request: &mut Self::ResourceRequest,
        policy: <Self::Platform as CSSParserContextPlatform>::ReferrerPolicy,
    );
    fn SetReferrerString(request: &mut Self::ResourceRequest, referrer: String);
    fn SetIsAdResource(request: &mut Self::ResourceRequest);
    fn NewFetchParameters(
        request: Self::ResourceRequest,
        options: Self::ResourceLoaderOptions,
    ) -> Self::FetchParameters;
    fn SetCharset(
        params: &mut Self::FetchParameters,
        charset: <Self::Platform as CSSParserContextPlatform>::TextEncoding,
    );
    fn SetFromOriginDirtyStyleSheet(params: &mut Self::FetchParameters, dirty: bool);
    // Uses document.GetExecutionContext().GetSecurityOrigin(), exactly as the
    // source call; the style crate does not derive a security origin from a URL.
    fn SetCrossOriginAccessControl(
        params: &mut Self::FetchParameters,
        document: &Self::Document,
        cross_origin: CrossOriginAttributeValue,
    );
    fn ParseIntegrityAttribute(
        integrity: &String,
        document: &Self::Document,
    ) -> Self::IntegrityMetadataSet;
    fn SetIntegrityMetadata(
        params: &mut Self::FetchParameters,
        metadata: Self::IntegrityMetadataSet,
    );
    fn SetFetchIntegrity(
        params: &mut Self::FetchParameters,
        integrity: &String,
        document: &Self::Document,
    );
    fn SetRenderBlockingBehavior(
        params: &mut Self::FetchParameters,
        behavior: RenderBlockingBehavior,
    );
    fn Fetch(
        params: Self::FetchParameters,
        fetcher: &Self::ResourceFetcher,
        client: &Rc<ImportedStyleSheetClient<Self>>,
    );
    // ResourceClient::ClearResource must detach the client from the resource.
    fn DetachResourceClient(resource: &Self::Resource, client: &ImportedStyleSheetClient<Self>);

    fn LoadFailedOrCanceled(resource: &Self::Resource) -> bool;
    fn ResourceURL(resource: &Self::Resource) -> <Self::Platform as CSSParserContextPlatform>::URL;
    fn LastRequestDevToolsId(resource: &Self::Resource) -> Self::DevToolsId;
    fn ResourceInitiatorPosition(resource: &Self::Resource) -> Self::TextPosition;
    fn LocalizedErrorDescription(resource: &Self::Resource) -> String;
    fn ReportStylesheetLoadingRequestFailedIssue(
        document: &Self::Document,
        url: <Self::Platform as CSSParserContextPlatform>::URL,
        devtools_id: Self::DevToolsId,
        parent_base_url: <Self::Platform as CSSParserContextPlatform>::URL,
        position: Self::TextPosition,
        description: String,
    );
    fn CSSResourceIntegrityEnforcementEnabled() -> bool;
    fn IntegrityMetadataIsEmpty(resource: &Self::Resource) -> bool;
    fn PassedIntegrityChecks(resource: &Self::Resource) -> bool;
    fn ErrorOccurred(resource: &Self::Resource) -> bool;
    fn SetLoadErrorStatus(resource: &Self::Resource);
    fn SendIntegrityReports(resource: &Self::Resource, document: &Self::Document);
    fn ResponseURL(resource: &Self::Resource) -> <Self::Platform as CSSParserContextPlatform>::URL;
    fn ResponseIsCorsSameOrigin(resource: &Self::Resource) -> bool;
    fn ResourceReferrerPolicy(
        resource: &Self::Resource,
    ) -> <Self::Platform as CSSParserContextPlatform>::ReferrerPolicy;
    fn ResourceEncoding(
        resource: &Self::Resource,
    ) -> <Self::Platform as CSSParserContextPlatform>::TextEncoding;
    fn ResourceRequestIsAdResource(resource: &Self::Resource) -> bool;
    fn NewStyleSheetContents(
        context: Rc<CSSParserContext<Self::Platform>>,
        original_url: <Self::Platform as CSSParserContextPlatform>::URL,
        owner: &Rc<StyleRuleImport<Self>>,
    ) -> Rc<Self::StyleSheet>;
}

// .h:99-105,115. Weak owner is the Rust back-pointer; the rule owns its client.
// The loader retains this client while attached. Callbacks use the same rule
// allocation, including synchronous cache hits inside Fetch.
pub struct ImportedStyleSheetClient<B: StyleRuleImportBackend> {
    owner_rule: Weak<StyleRuleImport<B>>,
    resource: RefCell<Option<Rc<B::Resource>>>,
}
impl<B: StyleRuleImportBackend> ImportedStyleSheetClient<B> {
    pub fn NotifyFinished(&self, resource: &B::Resource) {
        if let Some(owner) = self.owner_rule.upgrade() {
            owner.NotifyFinished(resource);
        }
    }
    /// Called by the resource backend when ResourceClient is associated. The
    /// backend retains its native AddClient/NotifyFinished ordering.
    pub fn SetResource(&self, resource: Rc<B::Resource>) {
        self.Dispose();
        *self.resource.borrow_mut() = Some(resource);
    }
    pub fn GetResource(&self) -> Option<Rc<B::Resource>> {
        self.resource.borrow().clone()
    }
    pub fn Dispose(&self) {
        let resource = self.resource.borrow_mut().take();
        if let Some(resource) = resource {
            B::DetachResourceClient(&resource, self);
        }
    }
}

// .h:122-141.
pub struct StyleRuleImport<B: StyleRuleImportBackend> {
    parent_style_sheet: RefCell<Option<Rc<B::StyleSheet>>>,
    style_sheet_client: Rc<ImportedStyleSheetClient<B>>,
    str_href: String,
    layer: LayerName,
    scope: Option<Rc<B::StyleScope>>,
    supports_string: String,
    media_queries: RefCell<Option<Rc<B::MediaQuerySet>>>,
    style_sheet: RefCell<Option<Rc<B::StyleSheet>>>,
    loading: Cell<bool>,
    supported: bool,
    origin_clean: OriginClean,
    position_hint: Cell<Option<B::TextPosition>>,
    modifiers: CSSUrlRequestModifiers<B::Platform>,
}
impl<B: StyleRuleImportBackend> StyleRuleImport<B> {
    // .cc:44-67.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        href: String,
        layer: LayerName,
        scope: Option<Rc<B::StyleScope>>,
        supported: bool,
        supports_string: String,
        media: Option<Rc<B::MediaQuerySet>>,
        origin_clean: OriginClean,
        modifiers: &CSSUrlRequestModifiers<B::Platform>,
    ) -> Rc<Self> {
        Rc::new_cyclic(|owner| Self {
            parent_style_sheet: RefCell::new(None),
            style_sheet_client: Rc::new(ImportedStyleSheetClient {
                owner_rule: owner.clone(),
                resource: RefCell::new(None),
            }),
            str_href: href,
            layer,
            scope,
            supports_string,
            media_queries: RefCell::new(Some(media.unwrap_or_else(B::CreateEmptyMediaQuerySet))),
            style_sheet: RefCell::new(None),
            loading: Cell::new(false),
            supported,
            origin_clean,
            position_hint: Cell::new(None),
            modifiers: modifiers.clone(),
        })
    }
    // .h:54-86,146-148; .cc:52.
    pub fn GetType(&self) -> RuleType {
        RuleType::kImport
    }
    pub fn IsImportRule(&self) -> bool {
        true
    }
    pub fn ParentStyleSheet(&self) -> Option<Rc<B::StyleSheet>> {
        self.parent_style_sheet.borrow().clone()
    }
    pub fn SetParentStyleSheet(&self, sheet: &Rc<B::StyleSheet>) {
        *self.parent_style_sheet.borrow_mut() = Some(sheet.clone());
    }
    pub fn ClearParentStyleSheet(&self) {
        self.parent_style_sheet.borrow_mut().take();
    }
    pub fn Href(&self) -> String {
        self.str_href.clone()
    }
    pub fn GetStyleSheet(&self) -> Option<Rc<B::StyleSheet>> {
        self.style_sheet.borrow().clone()
    }
    pub fn MediaQueries(&self) -> Option<Rc<B::MediaQuerySet>> {
        self.media_queries.borrow().clone()
    }
    pub fn SetMediaQueries(&self, media_queries: Option<Rc<B::MediaQuerySet>>) {
        *self.media_queries.borrow_mut() = media_queries;
    }
    pub fn SetPositionHint(&self, position_hint: B::TextPosition) {
        self.position_hint.set(Some(position_hint));
    }
    pub fn IsLayered(&self) -> bool {
        !self.layer.is_empty()
    }
    pub fn GetLayerName(&self) -> &LayerName {
        &self.layer
    }
    pub fn GetScope(&self) -> Option<Rc<B::StyleScope>> {
        self.scope.clone()
    }
    pub fn IsSupported(&self) -> bool {
        self.supported
    }
    pub fn GetModifiers(&self) -> &CSSUrlRequestModifiers<B::Platform> {
        &self.modifiers
    }
    pub fn GetSupportsString(&self) -> String {
        self.supports_string.clone()
    }
    // .cc:71-73.
    pub fn Dispose(&self) {
        self.style_sheet_client.Dispose();
    }
    // .cc:84-148.
    fn NotifyFinished(self: &Rc<Self>, resource: &B::Resource) {
        if let Some(sheet) = self.GetStyleSheet() {
            sheet.ClearOwnerRule();
        }
        let mut document = None;
        let mut parent_context =
            StrictCSSParserContext::<B::Platform>(SecureContextMode::kInsecureContext);
        if let Some(parent) = self.ParentStyleSheet() {
            document = parent.SingleOwnerDocument();
            parent_context = parent.ParserContext();
            if B::LoadFailedOrCanceled(resource) {
                if let Some(document) = document.as_ref() {
                    B::ReportStylesheetLoadingRequestFailedIssue(
                        document,
                        B::ResourceURL(resource),
                        B::LastRequestDevToolsId(resource),
                        parent.BaseURL(),
                        B::ResourceInitiatorPosition(resource),
                        B::LocalizedErrorDescription(resource),
                    );
                }
            }
        }
        let integrity_failed = B::CSSResourceIntegrityEnforcementEnabled()
            && !B::IntegrityMetadataIsEmpty(resource)
            && !B::PassedIntegrityChecks(resource);
        if integrity_failed {
            if !B::ErrorOccurred(resource) {
                B::SetLoadErrorStatus(resource);
            }
            if let Some(document) = document.as_ref() {
                B::SendIntegrityReports(resource, document);
            }
        } else {
            let response_url = B::ResponseURL(resource);
            let parser_document = document.as_ref().map(B::ParserDocument);
            let context = Rc::new(CSSParserContext::CopyForImport(
                &parent_context,
                response_url.clone(),
                B::ResponseIsCorsSameOrigin(resource),
                <B::Platform as CSSParserContextPlatform>::MakeReferrer(
                    B::URLString(&response_url),
                    B::ResourceReferrerPolicy(resource),
                ),
                B::ResourceEncoding(resource),
                parser_document.as_ref(),
            ));
            if B::ResourceRequestIsAdResource(resource) {
                context.SetIsAdRelated();
            }
            let sheet = B::NewStyleSheetContents(context, B::ResourceURL(resource), self);
            // Publish before parsing: nested imports observe their owner chain.
            *self.style_sheet.borrow_mut() = Some(sheet.clone());
            sheet.ParseAuthorStyleSheet(resource);
        }
        self.loading.set(false);
        if let Some(parent) = self.ParentStyleSheet() {
            parent.NotifyLoadedSheet(resource);
            parent.CheckLoaded();
        }
    }
    // .cc:150-152.
    pub fn IsLoading(&self) -> bool {
        self.loading.get() || self.GetStyleSheet().is_some_and(|sheet| sheet.IsLoading())
    }
    // .cc:154-242.
    pub fn RequestStyleSheet(&self) {
        let Some(parent) = self.ParentStyleSheet() else {
            return;
        };
        let Some(document) = parent.SingleOwnerDocument() else {
            return;
        };
        let Some(fetcher) = B::Fetcher(&document) else {
            return;
        };
        let base_url = parent.BaseURL();
        let abs_url = if !B::IsNullURL(&base_url) {
            <B::Platform as CSSParserContextPlatform>::ResolveURL(&base_url, &self.str_href, None)
        } else {
            B::CompleteURL(&document, &self.str_href)
        };
        let mut root_sheet = parent.clone();
        let mut sheet = Some(parent.clone());
        while let Some(current) = sheet {
            if B::EqualIgnoringFragmentIdentifier(&abs_url, &current.BaseURL())
                || B::EqualIgnoringFragmentIdentifier(
                    &abs_url,
                    &B::CompleteURL(&document, &current.OriginalURL()),
                )
            {
                return;
            }
            sheet = current.ParentStyleSheet();
            root_sheet = current;
        }
        let parser_context = parent.ParserContext();
        let referrer = parser_context.GetReferrer();
        let mut options = B::NewResourceLoaderOptions(parser_context.JavascriptWorld());
        B::SetCSSInitiatorName(&mut options);
        if let Some(position) = self.position_hint.get() {
            B::SetInitiatorPosition(&mut options, position);
        }
        B::SetInitiatorReferrer(&mut options, B::ReferrerString(referrer));
        let mut request = B::NewResourceRequest(abs_url);
        B::SetReferrerPolicy(
            &mut request,
            self.modifiers
                .referrer_policy
                .unwrap_or_else(|| B::ReferrerPolicy(referrer)),
        );
        B::SetReferrerString(&mut request, B::ReferrerString(referrer));
        if parser_context.IsAdRelated() {
            B::SetIsAdResource(&mut request);
        }
        let mut params = B::NewFetchParameters(request, options);
        B::SetCharset(&mut params, parent.Charset());
        B::SetFromOriginDirtyStyleSheet(&mut params, self.origin_clean != OriginClean::kTrue);
        if self.modifiers.cross_origin != CrossOriginAttributeValue::kCrossOriginAttributeNotSet {
            B::SetCrossOriginAccessControl(&mut params, &document, self.modifiers.cross_origin);
        }
        if !self.modifiers.integrity.IsNull() {
            let metadata = B::ParseIntegrityAttribute(&self.modifiers.integrity, &document);
            B::SetIntegrityMetadata(&mut params, metadata);
            B::SetFetchIntegrity(&mut params, &self.modifiers.integrity, &document);
        }
        self.loading.set(true);
        B::SetRenderBlockingBehavior(&mut params, root_sheet.GetRenderBlockingBehavior());
        B::Fetch(params, &fetcher, &self.style_sheet_client);
        if self.loading.get() {
            if let Some(parent) = self.ParentStyleSheet() {
                if parent.LoadCompleted() && Rc::ptr_eq(&root_sheet, &parent) {
                    parent.SetToPendingState();
                }
            }
        }
    }
    // .cc:244-246.
    pub fn GetLayerNameAsString(&self) -> String {
        LayerNameAsString(&self.layer)
    }
}
impl<B: StyleRuleImportBackend> Drop for StyleRuleImport<B> {
    fn drop(&mut self) {
        self.Dispose();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css_resource_fetch_restriction::ResourceFetchRestriction;
    use crate::parser::css_parser_context::CSSParserContextDocument;
    use crate::parser::css_parser_mode::CSSParserMode;
    use foundation::{AtomicString, CSSPropertyID};

    // Test doubles record external calls; production has no default backend.
    struct Platform;
    impl CSSParserContextPlatform for Platform {
        type URL = String;
        type TextEncoding = u8;
        type Referrer = (String, u8);
        type ReferrerPolicy = u8;
        type DOMWrapperWorld = u8;
        type ExecutionContext = ();
        type WebFeature = ();
        type WebDXFeature = ();
        fn NullURL() -> String {
            String::default()
        }
        fn EmptyURL() -> String {
            String::from("")
        }
        fn EmptyTextEncoding() -> u8 {
            0
        }
        fn IsEncodingValid(encoding: &u8) -> bool {
            *encoding != 0
        }
        fn EmptyReferrer() -> (String, u8) {
            (String::default(), 0)
        }
        fn MakeReferrer(value: String, policy: u8) -> (String, u8) {
            (value, policy)
        }
        fn StrippedForUseAsReferrer(url: &String) -> String {
            url.clone()
        }
        fn ResolveURL(base: &String, relative: &String, encoding: Option<&u8>) -> String {
            assert!(encoding.is_none());
            resolve(base, relative)
        }
        fn CSSParserIgnoreCharsetForURLsEnabled() -> bool {
            false
        }
        fn CountDeprecation(_: Option<&Rc<()>>, _: ()) {}
    }
    fn resolve(base: &String, relative: &String) -> String {
        if relative.as_str().starts_with("https://") {
            return relative.clone();
        }
        String::from(format!(
            "{}{relative}",
            base.as_str().rsplit_once('/').unwrap().0.to_owned() + "/",
            relative = relative.as_str()
        ))
    }
    struct Document {
        fetcher: Cell<bool>,
    }
    impl CSSParserContextDocument<Platform> for Document {
        fn CountUse(&self, _: ()) {}
        fn CountWebDXFeature(&self, _: ()) {}
        fn CountProperty(&self, _: CSSPropertyID) {}
        fn GetExecutionContext(&self) -> Option<Rc<()>> {
            Some(Rc::new(()))
        }
        fn IsForMarkupSanitization(&self) -> bool {
            false
        }
    }
    type Log = Rc<RefCell<Vec<&'static str>>>;
    struct Sheet {
        context: Rc<CSSParserContext<Platform>>,
        original: String,
        document: RefCell<Option<Rc<Document>>>,
        parent: RefCell<Option<Rc<Sheet>>>,
        completed: Cell<bool>,
        pending: Cell<usize>,
        loading: Cell<bool>,
        behavior: RenderBlockingBehavior,
        owner: RefCell<Weak<StyleRuleImport<Backend>>>,
        log: Log,
    }
    impl StyleRuleImportSheet<Backend> for Sheet {
        fn SingleOwnerDocument(&self) -> Option<Rc<Document>> {
            self.document.borrow().clone()
        }
        fn ParserContext(&self) -> Rc<CSSParserContext<Platform>> {
            self.context.clone()
        }
        fn BaseURL(&self) -> String {
            self.context.BaseURL().clone()
        }
        fn OriginalURL(&self) -> String {
            self.original.clone()
        }
        fn ParentStyleSheet(&self) -> Option<Rc<Self>> {
            self.parent.borrow().clone()
        }
        fn Charset(&self) -> u8 {
            *self.context.Charset()
        }
        fn ClearOwnerRule(&self) {
            self.log.borrow_mut().push("clear-owner");
            *self.owner.borrow_mut() = Weak::new();
        }
        fn ParseAuthorStyleSheet(&self, _: &Resource) {
            // New contents must already be visible while nested imports parse.
            let owner = self.owner.borrow().upgrade().unwrap();
            assert!(std::ptr::eq(owner.GetStyleSheet().unwrap().as_ref(), self));
            self.log.borrow_mut().push("parse");
        }
        fn IsLoading(&self) -> bool {
            self.loading.get()
        }
        fn NotifyLoadedSheet(&self, _: &Resource) {
            self.log.borrow_mut().push("notify");
        }
        fn CheckLoaded(&self) {
            self.log.borrow_mut().push("check");
        }
        fn LoadCompleted(&self) -> bool {
            self.completed.get()
        }
        fn SetToPendingState(&self) {
            self.pending.set(self.pending.get() + 1);
        }
        fn GetRenderBlockingBehavior(&self) -> RenderBlockingBehavior {
            self.behavior
        }
    }
    struct Resource {
        error: Cell<bool>,
        failed: Cell<bool>,
        metadata: Cell<bool>,
        passed: Cell<bool>,
        ad: Cell<bool>,
        log: Log,
    }
    struct Options {
        world: Option<u8>,
        css: bool,
        position: (i32, i32),
        referrer: String,
    }
    struct Request {
        url: String,
        referrer: String,
        policy: u8,
        ad: bool,
    }
    struct Params {
        request: Request,
        options: Options,
        charset: u8,
        dirty: bool,
        cross_origin: Option<CrossOriginAttributeValue>,
        metadata: Option<String>,
        fetch_integrity: Option<String>,
        behavior: RenderBlockingBehavior,
    }
    struct State {
        last_params: Option<Params>,
        client: Option<Rc<ImportedStyleSheetClient<Backend>>>,
        resource: Rc<Resource>,
        synchronous: bool,
        enforce: bool,
    }
    impl State {
        fn new() -> Self {
            Self {
                last_params: None,
                client: None,
                resource: Rc::new(Resource {
                    error: Cell::new(false),
                    failed: Cell::new(false),
                    metadata: Cell::new(false),
                    passed: Cell::new(true),
                    ad: Cell::new(false),
                    log: Rc::default(),
                }),
                synchronous: false,
                enforce: true,
            }
        }
    }
    thread_local! { static STATE: RefCell<State> = RefCell::new(State::new()); }
    struct Backend;
    impl StyleRuleImportBackend for Backend {
        type Platform = Platform;
        type StyleSheet = Sheet;
        type StyleScope = ();
        type MediaQuerySet = ();
        type Document = Document;
        type ResourceFetcher = ();
        type Resource = Resource;
        type ResourceLoaderOptions = Options;
        type ResourceRequest = Request;
        type FetchParameters = Params;
        type IntegrityMetadataSet = String;
        type TextPosition = (i32, i32);
        type DevToolsId = u32;
        fn CreateEmptyMediaQuerySet() -> Rc<()> {
            Rc::new(())
        }
        fn ParserDocument(document: &Rc<Document>) -> DocumentHandle<Platform> {
            document.clone()
        }
        fn Fetcher(document: &Document) -> Option<Rc<()>> {
            document.fetcher.get().then(|| Rc::new(()))
        }
        fn CompleteURL(_: &Document, relative: &String) -> String {
            resolve(&String::from("https://test/document"), relative)
        }
        fn IsNullURL(url: &String) -> bool {
            url.IsNull()
        }
        fn EqualIgnoringFragmentIdentifier(a: &String, b: &String) -> bool {
            !a.IsNull()
                && !b.IsNull()
                && a.as_str().split('#').next() == b.as_str().split('#').next()
        }
        fn URLString(url: &String) -> String {
            url.clone()
        }
        fn ReferrerString(referrer: &(String, u8)) -> String {
            referrer.0.clone()
        }
        fn ReferrerPolicy(referrer: &(String, u8)) -> u8 {
            referrer.1
        }
        fn NewResourceLoaderOptions(world: Option<&Rc<u8>>) -> Options {
            Options {
                world: world.map(|w| **w),
                css: false,
                position: (-1, -1),
                referrer: String::default(),
            }
        }
        fn SetCSSInitiatorName(options: &mut Options) {
            options.css = true;
        }
        fn SetInitiatorPosition(options: &mut Options, position: (i32, i32)) {
            options.position = position;
        }
        fn SetInitiatorReferrer(options: &mut Options, referrer: String) {
            options.referrer = referrer;
        }
        fn NewResourceRequest(url: String) -> Request {
            Request {
                url,
                referrer: String::default(),
                policy: 0,
                ad: false,
            }
        }
        fn SetReferrerPolicy(request: &mut Request, policy: u8) {
            request.policy = policy;
        }
        fn SetReferrerString(request: &mut Request, referrer: String) {
            request.referrer = referrer;
        }
        fn SetIsAdResource(request: &mut Request) {
            request.ad = true;
        }
        fn NewFetchParameters(request: Request, options: Options) -> Params {
            Params {
                request,
                options,
                charset: 0,
                dirty: false,
                cross_origin: None,
                metadata: None,
                fetch_integrity: None,
                behavior: RenderBlockingBehavior::kUnset,
            }
        }
        fn SetCharset(params: &mut Params, charset: u8) {
            params.charset = charset;
        }
        fn SetFromOriginDirtyStyleSheet(params: &mut Params, dirty: bool) {
            params.dirty = dirty;
        }
        fn SetCrossOriginAccessControl(
            params: &mut Params,
            _: &Document,
            cross_origin: CrossOriginAttributeValue,
        ) {
            params.cross_origin = Some(cross_origin);
        }
        fn ParseIntegrityAttribute(integrity: &String, _: &Document) -> String {
            integrity.clone()
        }
        fn SetIntegrityMetadata(params: &mut Params, metadata: String) {
            params.metadata = Some(metadata);
        }
        fn SetFetchIntegrity(params: &mut Params, integrity: &String, _: &Document) {
            params.fetch_integrity = Some(integrity.clone());
        }
        fn SetRenderBlockingBehavior(params: &mut Params, behavior: RenderBlockingBehavior) {
            params.behavior = behavior;
        }
        fn Fetch(params: Params, _: &(), client: &Rc<ImportedStyleSheetClient<Self>>) {
            let (resource, synchronous) = STATE.with(|state| {
                let mut state = state.borrow_mut();
                state.last_params = Some(params);
                state.client = Some(client.clone());
                (state.resource.clone(), state.synchronous)
            });
            client.SetResource(resource.clone());
            if synchronous {
                client.NotifyFinished(&resource);
            }
        }
        fn DetachResourceClient(resource: &Resource, _: &ImportedStyleSheetClient<Self>) {
            resource.log.borrow_mut().push("detach");
        }
        fn LoadFailedOrCanceled(resource: &Resource) -> bool {
            resource.failed.get()
        }
        fn ResourceURL(_: &Resource) -> String {
            String::from("https://test/request.css")
        }
        fn LastRequestDevToolsId(_: &Resource) -> u32 {
            42
        }
        fn ResourceInitiatorPosition(_: &Resource) -> (i32, i32) {
            (3, 4)
        }
        fn LocalizedErrorDescription(_: &Resource) -> String {
            String::from("failed")
        }
        fn ReportStylesheetLoadingRequestFailedIssue(
            _: &Document,
            url: String,
            id: u32,
            base: String,
            position: (i32, i32),
            description: String,
        ) {
            assert_eq!(url.as_str(), "https://test/request.css");
            assert_eq!(id, 42);
            assert_eq!(base.as_str(), "https://test/parent.css");
            assert_eq!(position, (3, 4));
            assert_eq!(description.as_str(), "failed");
            STATE.with(|state| state.borrow().resource.log.borrow_mut().push("audit"));
        }
        fn CSSResourceIntegrityEnforcementEnabled() -> bool {
            STATE.with(|state| state.borrow().enforce)
        }
        fn IntegrityMetadataIsEmpty(resource: &Resource) -> bool {
            !resource.metadata.get()
        }
        fn PassedIntegrityChecks(resource: &Resource) -> bool {
            resource.passed.get()
        }
        fn ErrorOccurred(resource: &Resource) -> bool {
            resource.error.get()
        }
        fn SetLoadErrorStatus(resource: &Resource) {
            resource.error.set(true);
            resource.log.borrow_mut().push("load-error");
        }
        fn SendIntegrityReports(resource: &Resource, _: &Document) {
            resource.log.borrow_mut().push("integrity-report");
        }
        fn ResponseURL(_: &Resource) -> String {
            String::from("https://cdn/response.css")
        }
        fn ResponseIsCorsSameOrigin(_: &Resource) -> bool {
            false
        }
        fn ResourceReferrerPolicy(_: &Resource) -> u8 {
            7
        }
        fn ResourceEncoding(_: &Resource) -> u8 {
            2
        }
        fn ResourceRequestIsAdResource(resource: &Resource) -> bool {
            resource.ad.get()
        }
        fn NewStyleSheetContents(
            context: Rc<CSSParserContext<Platform>>,
            url: String,
            owner: &Rc<StyleRuleImport<Self>>,
        ) -> Rc<Sheet> {
            let log = STATE.with(|state| state.borrow().resource.log.clone());
            Rc::new(Sheet {
                context,
                original: url,
                document: RefCell::new(None),
                parent: RefCell::new(owner.ParentStyleSheet()),
                completed: Cell::new(false),
                pending: Cell::new(0),
                loading: Cell::new(false),
                behavior: RenderBlockingBehavior::kUnset,
                owner: RefCell::new(Rc::downgrade(owner)),
                log,
            })
        }
    }
    fn setup() -> (Rc<Sheet>, Rc<Resource>) {
        STATE.with(|state| *state.borrow_mut() = State::new());
        let resource = STATE.with(|state| state.borrow().resource.clone());
        let document = Rc::new(Document {
            fetcher: Cell::new(true),
        });
        let parser_document: DocumentHandle<Platform> = document.clone();
        let context = Rc::new(CSSParserContext::new(
            String::from("https://test/parent.css"),
            true,
            1,
            CSSParserMode::kHTMLStandardMode,
            (String::from("parent-referrer"), 5),
            true,
            SecureContextMode::kSecureContext,
            Some(Rc::new(9)),
            Some(&parser_document),
            ResourceFetchRestriction::kNone,
        ));
        (
            Rc::new(Sheet {
                context,
                original: String::from("https://test/original.css"),
                document: RefCell::new(Some(document)),
                parent: RefCell::new(None),
                completed: Cell::new(true),
                pending: Cell::new(0),
                loading: Cell::new(false),
                behavior: RenderBlockingBehavior::kBlocking,
                owner: RefCell::new(Weak::new()),
                log: resource.log.clone(),
            }),
            resource,
        )
    }
    fn rule(
        href: &str,
        modifiers: &CSSUrlRequestModifiers<Platform>,
    ) -> Rc<StyleRuleImport<Backend>> {
        StyleRuleImport::new(
            String::from(href),
            vec![
                AtomicString::from_str("outer"),
                AtomicString::from_str("inner"),
            ],
            None,
            false,
            String::from("supports(display: grid)"),
            None,
            OriginClean::kFalse,
            modifiers,
        )
    }
    #[test]
    fn import_cycles_and_request_context_preserve_pending_semantics() {
        let (parent, resource) = setup();
        for href in ["parent.css#cycle", "original.css#cycle"] {
            let rule = rule(href, &CSSUrlRequestModifiers::default());
            rule.SetParentStyleSheet(&parent);
            rule.RequestStyleSheet();
            assert!(!rule.IsLoading());
            STATE.with(|state| assert!(state.borrow().last_params.is_none()));
        }
        let modifiers = CSSUrlRequestModifiers {
            cross_origin: CrossOriginAttributeValue::kCrossOriginAttributeUseCredentials,
            integrity: String::from(""),
            referrer_policy: Some(8),
        };
        parent.context.SetIsAdRelated();
        let rule = rule("child.css", &modifiers);
        assert!(rule.MediaQueries().is_some());
        assert!(!rule.IsSupported());
        assert_eq!(rule.GetLayerNameAsString().as_str(), "outer.inner");
        rule.SetParentStyleSheet(&parent);
        rule.SetPositionHint((3, 4));
        rule.RequestStyleSheet();
        assert!(rule.IsLoading());
        assert_eq!(parent.pending.get(), 1);
        STATE.with(|state| {
            let state = state.borrow();
            let params = state.last_params.as_ref().unwrap();
            assert_eq!(params.request.url.as_str(), "https://test/child.css");
            assert_eq!(params.request.policy, 8);
            assert!(params.request.ad);
            assert_eq!(params.request.referrer.as_str(), "parent-referrer");
            assert_eq!(params.options.referrer.as_str(), "parent-referrer");
            assert_eq!(params.options.world, Some(9));
            assert!(params.options.css);
            assert_eq!(params.options.position, (3, 4));
            assert_eq!(params.charset, 1);
            assert!(params.dirty);
            assert_eq!(
                params.cross_origin,
                Some(CrossOriginAttributeValue::kCrossOriginAttributeUseCredentials)
            );
            assert!(!params.metadata.as_ref().unwrap().IsNull());
            assert!(!params.fetch_integrity.as_ref().unwrap().IsNull());
            assert_eq!(params.behavior, RenderBlockingBehavior::kBlocking);
        });
        // An async nested import inherits the root's behavior and does not
        // transition an already completed nested sheet back to pending.
        let nested = Rc::new(Sheet {
            context: parent.context.clone(),
            original: String::from("https://test/nested.css"),
            document: RefCell::new(parent.SingleOwnerDocument()),
            parent: RefCell::new(Some(parent.clone())),
            completed: Cell::new(true),
            pending: Cell::new(0),
            loading: Cell::new(false),
            behavior: RenderBlockingBehavior::kNonBlocking,
            owner: RefCell::new(Weak::new()),
            log: resource.log.clone(),
        });
        let nested_rule = self::rule("other.css", &CSSUrlRequestModifiers::default());
        nested_rule.SetParentStyleSheet(&nested);
        nested_rule.RequestStyleSheet();
        assert_eq!(nested.pending.get(), 0);
        STATE.with(|state| {
            let state = state.borrow();
            let params = state.last_params.as_ref().unwrap();
            assert_eq!(params.behavior, RenderBlockingBehavior::kBlocking);
            assert!(params.metadata.is_none());
            assert!(params.fetch_integrity.is_none());
            assert_eq!(params.request.policy, 5);
            assert_eq!(params.options.position, (-1, -1));
        });
        rule.Dispose();
        assert!(rule.style_sheet_client.GetResource().is_none());
    }
    #[test]
    fn synchronous_completion_integrity_failure_and_detachment_keep_callback_order() {
        let (parent, resource) = setup();
        STATE.with(|state| state.borrow_mut().synchronous = true);
        let rule = rule("child.css", &CSSUrlRequestModifiers::default());
        rule.SetParentStyleSheet(&parent);
        rule.RequestStyleSheet();
        assert!(!rule.IsLoading());
        assert_eq!(parent.pending.get(), 0);
        assert_eq!(&*resource.log.borrow(), &["parse", "notify", "check"]);
        let old_sheet = rule.GetStyleSheet().unwrap();
        assert_eq!(old_sheet.BaseURL().as_str(), "https://cdn/response.css");
        assert_eq!(old_sheet.OriginalURL().as_str(), "https://test/request.css");
        assert!(!old_sheet.context.IsOriginClean());
        assert_eq!(
            old_sheet.context.GetReferrer().0.as_str(),
            "https://cdn/response.css"
        );
        assert_eq!(old_sheet.context.GetReferrer().1, 7);
        assert_eq!(*old_sheet.context.Charset(), 2);
        assert_eq!(
            old_sheet.context.GetSecureContextMode(),
            SecureContextMode::kSecureContext
        );
        old_sheet.loading.set(true);
        assert!(rule.IsLoading());
        old_sheet.loading.set(false);
        resource.log.borrow_mut().clear();
        resource.failed.set(true);
        resource.metadata.set(true);
        resource.passed.set(false);
        rule.style_sheet_client.NotifyFinished(&resource);
        assert_eq!(
            &*resource.log.borrow(),
            &[
                "clear-owner",
                "audit",
                "load-error",
                "integrity-report",
                "notify",
                "check"
            ]
        );
        assert!(Rc::ptr_eq(&old_sheet, &rule.GetStyleSheet().unwrap()));
        assert!(old_sheet.owner.borrow().upgrade().is_none());
        resource.log.borrow_mut().clear();
        rule.style_sheet_client.NotifyFinished(&resource);
        assert_eq!(
            &*resource.log.borrow(),
            &[
                "clear-owner",
                "audit",
                "integrity-report",
                "notify",
                "check"
            ]
        );
        // A detached rule still finishes with the strict insecure parser; it
        // has no document audit/report or parent completion callbacks.
        rule.ClearParentStyleSheet();
        resource.log.borrow_mut().clear();
        STATE.with(|state| state.borrow_mut().enforce = false);
        resource.ad.set(true);
        rule.style_sheet_client.NotifyFinished(&resource);
        assert_eq!(&*resource.log.borrow(), &["clear-owner", "parse"]);
        let detached = rule.GetStyleSheet().unwrap();
        assert_eq!(
            detached.context.GetSecureContextMode(),
            SecureContextMode::kInsecureContext
        );
        assert!(detached.context.IsAdRelated());
        assert!(!rule.IsLoading());
        rule.Dispose();
        assert_eq!(resource.log.borrow().last(), Some(&"detach"));
    }
}
