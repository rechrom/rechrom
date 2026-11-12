#![allow(non_snake_case)]
//! Source Page resource state. Page supplies mutation and event delivery, so
//! decoding and polling do not replace the source DOM/dirty/task lifecycle.
#[path = "font_face_resources.rs"]
mod font_face_resources;
pub use font_face_resources::FontFaceSourceInput;
#[path = "raw_resource_fetcher.rs"]
mod raw_resource_fetcher;
use crate::{
    resource_loader::{RequireResponse, ResourceLoadResult, ResourceLoader},
    text_decode::DecodeText,
    url_reference::ResolveUrl,
};
use cssom::{CSSFontFaceRule, CSSStyleSheet};
use decode::DecodeEffect;
use dom::{Document, DOM};
use image_resource::{
    ContainerKey, DocumentImageMutation, DocumentPaintContent, PaintImageContent,
};
use layoutng_assembly::internal::layout_input::{ConstraintSpace, FontFace, PaintImage};
use page_mutation::{
    DocumentImageFrameChanged, DocumentImageIntrinsicSizeChanged, FontResourceReady,
    ImageResourceReady, ResourceKind, ResourceLoadFailed, ResourceMutation, VideoFrameChanged,
};
pub use raw_resource_fetcher::RawResourceFetchOptions;
use resource::{DecodedImageResource, ResourceEngine};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    io,
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};
use url_loader::{ReferrerPolicy, RequestDestination, URLRequest};

/// These calls are Page::ApplyMutation and Page's native image-event delivery.
/// They are required; absent callbacks cannot silently complete a resource.
pub trait ResourceFetcherClient {
    fn ApplyResourceMutation(&mut self, mutation: ResourceMutation);
    fn DispatchImageEvent(&mut self, id: u64, loaded: bool);
    fn DispatchStyleSheetEvent(&mut self, id: u64, signature: String, loaded: bool);
    fn HasJavaScript(&self) -> bool;
}
// cpp: browser/browser.cc:1104-1112
struct PendingFontFace {
    metadata: font_face_resources::FontFaceTransportMetadata,
    owner: Rc<dom::font_loading::FontFace>,
    sources: Vec<font_face_resources::FaceSource>,
    next_url: usize,
    selected: bool,
    resource: Option<Rc<crate::font_resource::FontResource>>,
    decoded: Option<font_engine::SharedFontBytes>,
    published: bool,
    dirty: bool,
    local_face: Option<FontFace>,
    compiled: Option<Rc<style_parser::font_face_descriptors::CompiledFontFaceDescriptors>>,
}
// cpp: browser/browser.cc:1241-1245
pub struct ImageResourceRequestFacts {
    pub url: Option<Rc<url::Url>>,
    pub is_automatic_upgrade: bool,
    /// The same immutable ResourceBuffer passed into its owning decoder.
    pub response: Option<Rc<ImageResourceResponseFacts>>,
}
pub struct ImageResourceResponseFacts {
    pub bytes: Arc<[u8]>,
    pub mime_type: String,
    pub content_disposition: String,
}
struct PendingImage {
    source: String,
    url: String,
    resource: ResourceLoader,
    content: Rc<dom::persistent_document::DOMImageResourceContent>,
    decode: Option<decode::ImageDecodeJob>,
    is_multipart_response: Option<bool>,
    is_cors_same_origin: Option<bool>,
    content_dpr: Option<f64>,
    blocks_load: bool,
}

struct PendingVideo {
    node_id: u64,
    url: String,
    resource: ResourceLoader,
    decode: Option<decode::VideoDecodeJob>,
    is_cors_same_origin: Option<bool>,
}

// cpp: third_party/blink/renderer/core/html/link_style.h;
//      third_party/blink/renderer/core/loader/resource/css_style_sheet_resource.h
struct PendingStyleSheet {
    owner_node_id: u64,
    signature: String,
    url: String,
    resource: Option<ResourceLoader>,
    base: String,
    sheet: Option<CSSStyleSheet>,
    imports: Vec<PendingStyleSheetImport>,
    import_failed: bool,
    alternate: bool,
    title: String,
    media: String,
}

struct PendingStyleSheetImport {
    request: style::CSSStyleSheetImportRequest,
    url: String,
    resource: ResourceLoader,
}

/// Outstanding child resources for one parser-owned stylesheet. The parent
/// sheet remains with the parser; each request retains the original typed
/// `@import` rule and attaches the loaded child directly to that rule.
pub struct StyleSheetImportLoad {
    imports: Vec<PendingStyleSheetImport>,
}

pub struct StyleSheetImportFailure {
    pub url: String,
    pub error: String,
}

pub struct StyleSheetImportPoll {
    pub complete: bool,
    pub failures: Vec<StyleSheetImportFailure>,
}

struct DeferredImage {
    request: Option<URLRequest>,
    source: String,
    url: String,
    referrer_policy: Option<ReferrerPolicy>,
}

struct ReferencedImageOwners {
    key: Option<(dom::persistent_document::DOMOwnerHandle, u64)>,
    // Resource loading observes shadow-including document order. Cache only
    // owner positions and HTMLImageElement membership; current selections and
    // resolved-style dependencies are still read on every lifecycle pass.
    order: HashMap<usize, usize>,
    image_nodes: Vec<usize>,
}

fn SVGImageViewSpecification(source: &str) -> &str {
    source.rsplit_once('#').map_or("", |(_, fragment)| fragment)
}

/// HTML image attributes in document order. The parser's speculative scanner
/// can supply these without creating DOM nodes or borrowing Style internals.
#[derive(Clone, Debug, Default)]
pub struct ResponsiveImageSourceInput {
    pub srcset: Option<String>,
    pub sizes: Option<String>,
    pub media: Option<String>,
    pub image_type: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct ResponsiveImageInput {
    pub src: Option<String>,
    pub srcset: Option<String>,
    pub sizes: Option<String>,
    /// Only source siblings preceding the image in its parent picture.
    pub picture_sources: Vec<ResponsiveImageSourceInput>,
    pub allows_auto_sizes: bool,
    /// Last rendered content-box width after absolute-zoom adjustment. A
    /// speculative image has no rendered width and leaves this unset.
    pub auto_size: Option<f64>,
    pub lazy: bool,
}

pub type PreloadImageSelector =
    dyn Fn(
        &ResponsiveImageInput,
        &ConstraintSpace,
        &dyn Fn(&str) -> bool,
    ) -> Option<dom::persistent_document::ResponsiveImageSelection>;

pub type ResponsiveImageSelector =
    dyn Fn(
        &Document,
        usize,
        &ConstraintSpace,
        &dyn Fn(&str) -> bool,
    ) -> Option<dom::persistent_document::ResponsiveImageSelection>;

// Rust separates the resource-owned fields of Page::Impl while retaining the
// same document and constraint handles, not serialized/copied DOM data.
// cpp: browser/browser.cc:1774-1798
pub struct ResourceFetcher {
    raw_fetch: raw_resource_fetcher::RawFetchContext,
    html_images: html_image_loader::HTMLImageFetchContext,
    archive: Option<Rc<dyn std::any::Any>>,
    style_sheet_receiver: RefCell<Option<Rc<dyn Fn(&mut Document, CSSStyleSheet)>>>,
    engine: Rc<ResourceEngine>,
    allow_stale_resources: std::cell::Cell<bool>,
    document: Rc<RefCell<DOM>>,
    constraints: Rc<RefCell<ConstraintSpace>>,
    base_url: RefCell<Rc<dom::document_base_url::DocumentURLValue>>,
    image_request_urls: RefCell<HashMap<String, Rc<ImageResourceRequestFacts>>>,
    style_sheet_signatures: RefCell<HashMap<u64, String>>,
    pending_style_sheets: RefCell<Vec<PendingStyleSheet>>,
    loaded_image_sources: RefCell<HashSet<String>>,
    referenced_image_owners: RefCell<Option<ReferencedImageOwners>>,
    pending_images: RefCell<Vec<PendingImage>>,
    pending_videos: RefCell<Vec<PendingVideo>>,
    attempted_video_nodes: RefCell<HashSet<u64>>,
    deferred_images: RefCell<Vec<DeferredImage>>,
    visible_image_sources: RefCell<HashSet<String>>,
    document_image_sources: RefCell<HashMap<u64, String>>,
    responsive_image_selector: RefCell<Option<Rc<ResponsiveImageSelector>>>,
    preload_image_selector: RefCell<Option<Rc<PreloadImageSelector>>>,
    preload_media_matcher: RefCell<Option<Rc<dyn Fn(&str, &ConstraintSpace) -> bool>>>,
    pending_fonts: RefCell<Vec<PendingFontFace>>,
    font_loading: font_face_resources::FontFetchContext,
}
/// ResourceFetcher.h:776-799. The guard owns the exact fetcher and restores
/// its previous policy across listener reentry and nested clipboard commands.
pub struct ResourceCacheValidationSuppressor {
    fetcher: Rc<ResourceFetcher>,
    previous: bool,
}
impl Drop for ResourceCacheValidationSuppressor {
    fn drop(&mut self) {
        self.fetcher.allow_stale_resources.set(self.previous);
    }
}
impl ResourceFetcher {
    /// Document::Shutdown / ResourceFetcher::ClearContext. The original
    /// resource owner performs synchronous cancellation before the Window's
    /// task sources are retired; its keepalive transfer remains independent.
    pub fn DetachDocumentFetchContext(&self) -> io::Result<()> {
        self.engine.StopFetching()?;
        self.engine.ClearFetchContext()
    }
    /// A native child Document gets its own FetchContext/URL/referrer and
    /// cancellation owner, sharing only the original transport/decode service
    /// and immutable Page resource namespace through ResourceEngine::ForDocument.
    pub fn ForChildDocument(
        &self,
        document: Rc<RefCell<DOM>>,
        constraints: Rc<RefCell<ConstraintSpace>>,
        base_url: String,
    ) -> (Rc<Self>, Rc<ResourceEngine>) {
        let engine = self.engine.ForDocument(base_url.clone());
        let fetcher = Rc::new(Self::WithEngine(engine.clone(), document, constraints));
        // Share stateless selection services; each receiving FetchContext still
        // reads its own Document, constraints, policy and pending requests.
        *fetcher.responsive_image_selector.borrow_mut() =
            self.responsive_image_selector.borrow().clone();
        *fetcher.preload_image_selector.borrow_mut() = self.preload_image_selector.borrow().clone();
        *fetcher.preload_media_matcher.borrow_mut() = self.preload_media_matcher.borrow().clone();
        (fetcher, engine)
    }
    pub fn SuppressCacheValidation(self: &Rc<Self>) -> ResourceCacheValidationSuppressor {
        let previous = self.allow_stale_resources.replace(true);
        ResourceCacheValidationSuppressor {
            fetcher: self.clone(),
            previous,
        }
    }
    pub fn HasArchive(&self) -> bool {
        self.archive.is_some()
    }

    /// Same execution-context policy used by every outgoing request. HTTP and
    /// meta-referrer producers update this ResourceEngine owner in place.
    pub fn ReferrerPolicyReadHandle(
        &self,
    ) -> foundation::referrer_policy::ReferrerPolicyReadHandle {
        self.engine.ReferrerPolicyReadHandle()
    }
    pub fn ReferrerPolicy(&self) -> ReferrerPolicy {
        self.engine.ReferrerPolicy()
    }
    pub fn MemoryDiagnostics(&self) -> resource::ResourceMemoryDiagnostics {
        self.engine.MemoryDiagnostics()
    }
    pub fn WithEngine(
        engine: Rc<ResourceEngine>,
        document: Rc<RefCell<DOM>>,
        constraints: Rc<RefCell<ConstraintSpace>>,
    ) -> Self {
        let base_url = document.borrow().GetDocument().BaseURL();
        let raw_fetch = raw_resource_fetcher::RawFetchContext::new(
            document.borrow().GetDocument(),
            engine.FetchContextDetachedState(),
        );
        Self {
            allow_stale_resources: std::cell::Cell::new(false),
            raw_fetch,
            html_images: Default::default(),
            archive: None,
            style_sheet_receiver: RefCell::new(None),
            engine,
            document,
            constraints,
            base_url: RefCell::new(base_url),
            image_request_urls: RefCell::new(HashMap::new()),
            style_sheet_signatures: RefCell::new(HashMap::new()),
            pending_style_sheets: RefCell::new(Vec::new()),
            loaded_image_sources: RefCell::new(HashSet::new()),
            referenced_image_owners: RefCell::new(None),
            pending_images: RefCell::new(Vec::new()),
            pending_videos: RefCell::new(Vec::new()),
            attempted_video_nodes: RefCell::new(HashSet::new()),
            deferred_images: RefCell::new(Vec::new()),
            visible_image_sources: RefCell::new(HashSet::new()),
            document_image_sources: RefCell::new(HashMap::new()),
            responsive_image_selector: RefCell::new(None),
            preload_image_selector: RefCell::new(None),
            preload_media_matcher: RefCell::new(None),
            pending_fonts: RefCell::new(Vec::new()),
            font_loading: font_face_resources::FontFetchContext::default(),
        }
    }
    pub fn ImageResourceRequest(&self, source: &str) -> Option<Rc<ImageResourceRequestFacts>> {
        self.image_request_urls.borrow().get(source).cloned()
    }
    fn RetainImageResponseURL(
        &self,
        source: &str,
        url: &str,
        bytes: Arc<[u8]>,
        mime_type: String,
        content_disposition: String,
    ) {
        let previous = self
            .image_request_urls
            .borrow()
            .get(source)
            .cloned()
            .expect("image request precedes response");
        self.image_request_urls.borrow_mut().insert(
            source.to_owned(),
            Rc::new(ImageResourceRequestFacts {
                url: url::Url::parse(url).ok().map(Rc::new),
                is_automatic_upgrade: previous.is_automatic_upgrade,
                response: Some(Rc::new(ImageResourceResponseFacts {
                    bytes,
                    mime_type,
                    content_disposition,
                })),
            }),
        );
    }
    fn StartImageRequest(
        &self,
        source: &str,
        request: URLRequest,
    ) -> (
        ResourceLoader,
        Rc<dom::persistent_document::DOMImageResourceContent>,
    ) {
        self.StartImageRequestWithReload(source, request, false)
    }
    fn StartImageRequestWithReload(
        &self,
        source: &str,
        request: URLRequest,
        forced_reload: bool,
    ) -> (
        ResourceLoader,
        Rc<dom::persistent_document::DOMImageResourceContent>,
    ) {
        self.StartImageRequestWithReloadAndPolicy(
            source,
            request,
            forced_reload,
            resource::ResourceLoadEventPolicy::Regular,
        )
    }
    fn StartImageRequestWithReloadAndPolicy(
        &self,
        source: &str,
        request: URLRequest,
        forced_reload: bool,
        policy: resource::ResourceLoadEventPolicy,
    ) -> (
        ResourceLoader,
        Rc<dom::persistent_document::DOMImageResourceContent>,
    ) {
        self.image_request_urls.borrow_mut().insert(
            source.to_owned(),
            Rc::new(ImageResourceRequestFacts {
                url: url::Url::parse(&request.url).ok().map(Rc::new),
                is_automatic_upgrade: request.is_automatic_upgrade,
                response: None,
            }),
        );
        let content = self.ImageContentForRequest(source, &request);
        content.StartDeferred();
        let resource = if forced_reload && !self.allow_stale_resources.get() {
            self.engine.ReloadImageWithLoadEventPolicy(request, policy)
        } else {
            self.engine.StartWithLoadEventPolicy(request, policy)
        };
        if let Some(info) = resource.MediaResourceInfo() {
            content.SetResourceInfo(info);
        }
        (resource, content)
    }
    pub fn BaseURL(&self) -> String {
        self.base_url.borrow().AsStr().to_owned()
    }
    fn IsLoadedImageSource(&self, source: &str) -> bool {
        let Ok(url) = ResolveUrl(&self.base_url.borrow(), source) else {
            return false;
        };
        self.engine.HasLoaded(URLRequest {
            url,
            destination: RequestDestination::kImage,
            ..Default::default()
        })
    }
    pub fn SetResponsiveImageSelector(&self, selector: Rc<ResponsiveImageSelector>) {
        *self.responsive_image_selector.borrow_mut() = Some(selector);
    }
    pub fn SetPreloadImageSelector(&self, selector: Rc<PreloadImageSelector>) {
        *self.preload_image_selector.borrow_mut() = Some(selector);
    }
    pub fn SetPreloadMediaMatcher(&self, matcher: Rc<dyn Fn(&str, &ConstraintSpace) -> bool>) {
        *self.preload_media_matcher.borrow_mut() = Some(matcher);
    }
    /// HTMLLinkElement's original media evaluation, using this loading owner's
    /// existing Style matcher and viewport, before the DOM transaction reenters JS.
    pub fn ProcessRenderBlockingDocumentMutations(&self, document: &Document) {
        self.ProcessFontPreloadMutations(document);
        for request in document.TakeRenderBlockingDocumentMutations() {
            let matches = request
                .Media()
                .is_none_or(|media| self.PreloadMediaMatches(media));
            document.ApplyRenderBlockingDocumentMutation(request, matches);
        }
    }
    pub fn PreloadMediaMatches(&self, query: &str) -> bool {
        if query.trim().is_empty() {
            return true;
        }
        self.preload_media_matcher
            .borrow()
            .as_ref()
            .is_some_and(|matcher| matcher(query, &self.constraints.borrow()))
    }
    /// Select using the scanner's predicted base without borrowing the DOM:
    /// the authoritative parser may currently own its arena. Cache reuse must
    /// resolve against this same base as the eventual speculative request.
    pub fn SelectPreloadImage(
        &self,
        input: &ResponsiveImageInput,
        base_url_for_cache: &str,
    ) -> Option<dom::persistent_document::ResponsiveImageSelection> {
        let selector = self.preload_image_selector.borrow().clone()?;
        let constraints = self.constraints.borrow();
        let is_loaded = |source: &str| {
            let Ok(url) = ResolveUrl(base_url_for_cache, source) else {
                return false;
            };
            self.engine.HasLoaded(URLRequest {
                url,
                destination: RequestDestination::kImage,
                ..Default::default()
            })
        };
        selector(input, &constraints, &is_loaded)
    }
    /// The parser lends its authoritative arena while handling the element;
    /// selection therefore must not borrow ResourceFetcher's DOM facade.
    pub fn SelectResponsiveImageInDocument(
        &self,
        document: &Document,
        node: usize,
    ) -> Option<dom::persistent_document::ResponsiveImageSelection> {
        if !document.Node(node).IsHTMLElement("img") {
            return None;
        }
        let Some(selector) = self.responsive_image_selector.borrow().clone() else {
            return document
                .Node(node)
                .FindAttribute("src")
                .filter(|attribute| !attribute.value.is_empty())
                .map(
                    |attribute| dom::persistent_document::ResponsiveImageSelection {
                        source: attribute.value.clone(),
                        density: None,
                    },
                );
        };
        let constraints = self.constraints.borrow();
        let is_loaded = |source: &str| {
            document.ImageResourceFor(source).is_some() || self.IsLoadedImageSource(source)
        };
        selector(document, node, &constraints, &is_loaded)
    }
    /// Run HTMLImageElement's responsive selector for one connected element.
    /// Parser/DOM mutation discovery calls this before starting the request so
    /// `currentSrc` and the queued URL are never temporarily pinned to `src`.
    pub fn UpdateResponsiveImageSelection(&self, node: usize) -> Option<String> {
        let selection = {
            let owner = self.document.borrow();
            if !owner.GetDocument().Node(node).IsHTMLElement("img") {
                return None;
            }
            self.SelectResponsiveImageInDocument(owner.GetDocument(), node)
        };
        let changed = self
            .document
            .borrow_mut()
            .GetDocumentMut()
            .SetResponsiveImageSelection(node, selection.clone());
        selection.map(|selection| selection.source)
    }
    /// Page routes prepared rules through its resident CSSOM. The parser can
    /// lend its temporarily owned arena without re-borrowing the DOM facade.
    pub fn SetStyleSheetReceiver(&self, receive: Rc<dyn Fn(&mut Document, CSSStyleSheet)>) {
        *self.style_sheet_receiver.borrow_mut() = Some(receive);
    }

    fn CommitStyleSheet(&self, document: &mut Document, sheet: CSSStyleSheet) {
        if let Some(receive) = self.style_sheet_receiver.borrow().as_ref() {
            receive(document, sheet);
        } else {
            document.AppendStyleSheet(sheet);
        }
    }
    // cpp: browser/browser.cc:775-779
    pub fn SetDocumentURL(&self, url: String) {
        self.engine.SetDocumentURL(url.clone());
        let mut owner = self.document.borrow_mut();
        owner.GetDocumentMut().SetDocumentURL(url);
        self.ConsumeBaseURLChanges(owner.GetDocumentMut());
    }
    pub fn SetBaseURL(&self, base: Rc<dom::document_base_url::DocumentURLValue>) {
        *self.base_url.borrow_mut() = base;
    }
    pub fn ConsumeBaseURLChanges(&self, document: &mut Document) {
        for change in document.TakeBaseURLChanges() {
            self.SetBaseURL(change.value);
        }
    }
    pub fn SynchronizeDocumentBaseURL(&self) {
        self.ConsumeBaseURLChanges(self.document.borrow_mut().GetDocumentMut());
    }
    /// Complete against the same retained resource/base URL owner, never a
    /// second DOM base-element scan or a parser in the hit-test adapter.
    pub fn CompleteURL(&self, value: &str) -> Option<url::Url> {
        self.base_url.borrow().CompleteURL(value)
    }
    // cpp: browser/browser.cc:766-772
    pub fn AddInitialImage(
        &self,
        source: String,
        image: PaintImage,
        client: &mut dyn ResourceFetcherClient,
    ) {
        self.loaded_image_sources
            .borrow_mut()
            .insert(source.clone());
        let url = self
            .CompleteURL(&source)
            .map_or_else(|| source.clone(), |url| url.to_string());
        let content = self.ImageContentForRequest(
            &source,
            &URLRequest {
                url,
                destination: RequestDestination::kImage,
                ..Default::default()
            },
        );
        self.CommitImage(source, image, Some(content), client);
    }
    // cpp: browser/browser.cc:1046-1067
    fn CommitImage(
        &self,
        source: String,
        image: PaintImage,
        content: Option<Rc<dom::persistent_document::DOMImageResourceContent>>,
        client: &mut dyn ResourceFetcherClient,
    ) {
        if !self.image_request_urls.borrow().contains_key(&source) {
            self.image_request_urls.borrow_mut().insert(
                source.clone(),
                Rc::new(ImageResourceRequestFacts {
                    url: self.CompleteURL(&source).map(Rc::new),
                    is_automatic_upgrade: false,
                    response: None,
                }),
            );
        }
        self.engine.ObserveImageId(image.id);
        client.ApplyResourceMutation(ResourceMutation::ImageResourceReady(ImageResourceReady {
            source,
            image,
            content,
        }));
    }
    // cpp: browser/browser.cc:1081-1102
    pub fn AddParsedStyleSheet(&self, sheet: CSSStyleSheet, base: &str) -> io::Result<()> {
        self.AddParsedStyleSheetToDocument(self.document.borrow_mut().GetDocumentMut(), sheet, base)
    }

    /// Commit a connected `<style>` immediately when it has no imports, or
    /// retain that exact parsed owner until its original `@import` rules have
    /// their child contents attached.
    pub fn AddParsedStyleSheetWithImports(
        &self,
        sheet: CSSStyleSheet,
        base: &str,
    ) -> io::Result<bool> {
        let imports = self.StartStyleSheetImports(&sheet, base)?;
        if imports.is_empty() {
            self.AddParsedStyleSheet(sheet, base)?;
            return Ok(true);
        }
        self.pending_style_sheets
            .borrow_mut()
            .push(PendingStyleSheet {
                owner_node_id: sheet.owner_node_id,
                signature: String::new(),
                url: base.to_owned(),
                resource: None,
                base: base.to_owned(),
                sheet: Some(sheet),
                imports,
                import_failed: false,
                alternate: false,
                title: String::new(),
                media: String::new(),
            });
        Ok(false)
    }

    // The parser temporarily owns the same arena; use its borrow directly.
    // Re-borrowing DOM here would target the swapped-out document.
    pub fn AddParsedStyleSheetToDocument(
        &self,
        d: &mut Document,
        mut sheet: CSSStyleSheet,
        base: &str,
    ) -> io::Result<()> {
        let detached = d
            .FindNodeById(sheet.owner_node_id)
            .is_some_and(|i| d.Node(i).OwnerDocumentNode() != Some(d.RootHandle()));
        if detached {
            self.CommitStyleSheet(d, sheet);
            return Ok(());
        }
        if d.StyleSheetCanBeActivated(&sheet) {
            for rule in &sheet.font_faces {
                self.QueueCompiledFontFaceInDocument(rule, base, d)?;
            }
        }
        crate::ResolveCSSStyleSheetURLs(&mut sheet, base)?;
        self.CommitStyleSheet(d, sheet);
        Ok(())
    }

    pub fn OwnsDocument(&self, document: &Rc<RefCell<DOM>>) -> bool {
        Rc::ptr_eq(&self.document, document)
    }

    /// Discover stylesheet links created or changed after the HTML parser's
    /// insertion callback. Blink gives every connected HTMLLinkElement a
    /// LinkStyle loader; the document-wide lifecycle walk here is the Rust
    /// ownership equivalent and deliberately shares ResourceEngine with the
    /// parser so URL requests remain coalesced.
    pub fn QueueReferencedStyleSheets(&self) -> io::Result<usize> {
        self.QueueStyleSheetCandidates(None)
    }
    pub fn QueueStyleSheetForNode(&self, node_id: u64) -> io::Result<usize> {
        self.QueueStyleSheetCandidates(Some(node_id))
    }
    fn QueueStyleSheetCandidates(&self, candidate: Option<u64>) -> io::Result<usize> {
        struct Request {
            owner_node_id: u64,
            signature: String,
            href: String,
            referrer_policy: Option<ReferrerPolicy>,
            alternate: bool,
            title: String,
            media: String,
        }

        let mut requests = Vec::new();
        let mut metadata_updates = Vec::new();
        {
            let owner = self.document.borrow();
            let document = owner.GetDocument();
            let candidates = if let Some(id) = candidate {
                let Some(node) = document.FindNodeById(id) else {
                    return Ok(0);
                };
                node..node + 1
            } else {
                0..document.NodeCount()
            };
            for i in candidates {
                if !document.IsConnectedToDocument(i) {
                    continue;
                }
                let node = document.Node(i);
                if !node.IsHTMLElement("link") {
                    continue;
                }
                let attr = |name| node.FindAttribute(name).map_or("", |a| a.value.as_str());
                let disabled = node.FindAttribute("disabled").is_some();
                let disposition =
                    html::link_style::ClassifyStyleSheetLink(attr("rel"), attr("type"), disabled);
                let signature = format!(
                    "{}\u{0}{}\u{0}{}\u{0}{}\u{0}{}\u{0}{}",
                    attr("href"),
                    attr("rel"),
                    attr("type"),
                    disabled,
                    attr("title"),
                    attr("media")
                );
                let existing = document
                    .StyleSheets(None)
                    .iter()
                    .find(|sheet| sheet.owner_node_id == node.Id());
                if self.style_sheet_signatures.borrow().get(&node.Id()) == Some(&signature) {
                    continue;
                }
                if !disposition.is_style_sheet || disabled || attr("href").is_empty() {
                    if let Some(sheet) = existing {
                        if !sheet.disabled {
                            let mut sheet = sheet.clone();
                            sheet.disabled = true;
                            metadata_updates.push(sheet);
                        }
                    }
                    self.style_sheet_signatures
                        .borrow_mut()
                        .insert(node.Id(), signature);
                    continue;
                }
                // Parser-owned links have already completed before Page's
                // ordinary discovery pass. Seed their signature without
                // issuing a second request; ResourceEngine coalescing alone
                // would not avoid a second CSS parse and owner replacement.
                if existing.is_some()
                    && !self
                        .style_sheet_signatures
                        .borrow()
                        .contains_key(&node.Id())
                {
                    self.style_sheet_signatures
                        .borrow_mut()
                        .insert(node.Id(), signature);
                    continue;
                }
                requests.push(Request {
                    owner_node_id: node.Id(),
                    signature,
                    href: attr("href").to_owned(),
                    referrer_policy: ReferrerPolicy::FromString(attr("referrerpolicy")),
                    alternate: disposition.alternate,
                    title: attr("title").to_owned(),
                    media: attr("media").to_owned(),
                });
            }
        }
        for sheet in metadata_updates {
            self.CommitStyleSheet(self.document.borrow_mut().GetDocumentMut(), sheet);
        }
        let current_url = self.engine.DocumentURL();
        let base_url = self.base_url.borrow().clone();
        let mut queued = 0;
        for request in requests {
            let url = ResolveUrl(&base_url, &request.href)?;
            let resource = self.engine.Start(URLRequest {
                url: url.clone(),
                referrer: current_url.clone(),
                referrer_policy: request.referrer_policy,
                destination: RequestDestination::kStyleSheet,
                ..Default::default()
            });
            self.style_sheet_signatures
                .borrow_mut()
                .insert(request.owner_node_id, request.signature.clone());
            self.pending_style_sheets
                .borrow_mut()
                .push(PendingStyleSheet {
                    owner_node_id: request.owner_node_id,
                    signature: request.signature,
                    url,
                    resource: Some(resource),
                    base: String::new(),
                    sheet: None,
                    imports: Vec::new(),
                    import_failed: false,
                    alternate: request.alternate,
                    title: request.title,
                    media: request.media,
                });
            queued += 1;
        }
        Ok(queued)
    }

    pub fn HasPendingStyleSheets(&self) -> bool {
        !self.pending_style_sheets.borrow().is_empty()
    }

    fn StartStyleSheetImports(
        &self,
        sheet: &CSSStyleSheet,
        base: &str,
    ) -> io::Result<Vec<PendingStyleSheetImport>> {
        let referrer = self.engine.DocumentURL();
        style::CSSStyleSheetImportRequests(sheet)
            .into_iter()
            .map(|request| {
                let url = ResolveUrl(base, request.Href())?;
                let resource = self.engine.Start(URLRequest {
                    url: url.clone(),
                    referrer: referrer.clone(),
                    destination: RequestDestination::kStyleSheet,
                    ..Default::default()
                });
                Ok(PendingStyleSheetImport {
                    request,
                    url,
                    resource,
                })
            })
            .collect()
    }

    pub fn StartStyleSheetImportLoad(
        &self,
        sheet: &CSSStyleSheet,
        base: &str,
    ) -> io::Result<StyleSheetImportLoad> {
        Ok(StyleSheetImportLoad {
            imports: self.StartStyleSheetImports(sheet, base)?,
        })
    }

    /// Advance ready `@import` resources without blocking the parser task.
    /// Newly discovered imports are appended to this same load owner.
    pub fn PollStyleSheetImportLoad(
        &self,
        load: &mut StyleSheetImportLoad,
    ) -> StyleSheetImportPoll {
        let mut failures = Vec::new();
        let ready = load
            .imports
            .iter_mut()
            .position(|import| import.resource.Poll());
        if let Some(position) = ready {
            let mut import = load.imports.swap_remove(position);
            let result = RequireResponse(import.resource.TakeResult()).and_then(|response| {
                let base = if response.final_url.is_empty() {
                    import.url.clone()
                } else {
                    response.final_url.clone()
                };
                let css = DecodeText(&response)?;
                let sheet = style::ParseCSSWithBase(&css, &base);
                if !import.request.Attach(&sheet) {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "@import child lost its translated stylesheet owner",
                    ));
                }
                self.StartStyleSheetImports(&sheet, &base)
            });
            match result {
                Ok(imports) => load.imports.extend(imports),
                Err(error) => failures.push(StyleSheetImportFailure {
                    url: import.url,
                    error: error.to_string(),
                }),
            }
        }
        StyleSheetImportPoll {
            complete: load.imports.is_empty(),
            failures,
        }
    }

    pub fn PollPendingStyleSheetsWithBudget(
        &self,
        client: &mut dyn ResourceFetcherClient,
        max_completions: usize,
    ) -> usize {
        if max_completions == 0 {
            return 0;
        }
        let count = self.pending_style_sheets.borrow().len();
        let mut completed = 0;
        for _ in 0..count {
            let mut pending = self
                .pending_style_sheets
                .borrow_mut()
                .pop()
                .expect("stylesheet count changed during poll");
            if completed >= max_completions {
                self.pending_style_sheets.borrow_mut().insert(0, pending);
                continue;
            }

            if pending.sheet.is_none() {
                let ready = pending.resource.as_mut().is_some_and(ResourceLoader::Poll);
                if !ready {
                    self.pending_style_sheets.borrow_mut().insert(0, pending);
                    continue;
                }
                completed += 1;
                let result = RequireResponse(
                    pending
                        .resource
                        .as_mut()
                        .expect("ready stylesheet resource exists")
                        .TakeResult(),
                )
                .and_then(|response| {
                    let base = if response.final_url.is_empty() {
                        pending.url.clone()
                    } else {
                        response.final_url.clone()
                    };
                    let css = DecodeText(&response)?;
                    let sheet = style::ParseCSSWithBase(&css, &base);
                    let imports = self.StartStyleSheetImports(&sheet, &base)?;
                    Ok((sheet, base, imports))
                });
                pending.resource = None;
                match result {
                    Ok((sheet, base, imports)) => {
                        pending.sheet = Some(sheet);
                        pending.base = base;
                        pending.imports = imports;
                    }
                    Err(error) => {
                        // A changed href may retry. Do not pin the failed signature as
                        // a successful LinkStyle state.
                        if self
                            .style_sheet_signatures
                            .borrow()
                            .get(&pending.owner_node_id)
                            == Some(&pending.signature)
                        {
                            self.style_sheet_signatures
                                .borrow_mut()
                                .remove(&pending.owner_node_id);
                        }
                        client.ApplyResourceMutation(ResourceMutation::ResourceLoadFailed(
                            ResourceLoadFailed {
                                kind: ResourceKind::kStyleSheet,
                                source: None,
                                url: pending.url,
                                error: error.to_string(),
                            },
                        ));
                        if client.HasJavaScript() {
                            client.DispatchStyleSheetEvent(
                                pending.owner_node_id,
                                pending.signature,
                                false,
                            );
                        }
                        continue;
                    }
                }
            } else if let Some(position) = pending
                .imports
                .iter_mut()
                .position(|import| import.resource.Poll())
            {
                completed += 1;
                let mut import = pending.imports.swap_remove(position);
                let result = RequireResponse(import.resource.TakeResult()).and_then(|response| {
                    let base = if response.final_url.is_empty() {
                        import.url.clone()
                    } else {
                        response.final_url.clone()
                    };
                    let css = DecodeText(&response)?;
                    let sheet = style::ParseCSSWithBase(&css, &base);
                    if !import.request.Attach(&sheet) {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "@import child lost its translated stylesheet owner",
                        ));
                    }
                    let imports = self.StartStyleSheetImports(&sheet, &base)?;
                    Ok(imports)
                });
                match result {
                    Ok(imports) => pending.imports.extend(imports),
                    Err(error) => {
                        pending.import_failed = true;
                        client.ApplyResourceMutation(ResourceMutation::ResourceLoadFailed(
                            ResourceLoadFailed {
                                kind: ResourceKind::kStyleSheet,
                                source: None,
                                url: import.url,
                                error: error.to_string(),
                            },
                        ));
                    }
                }
            }

            if pending.sheet.is_some() && pending.imports.is_empty() {
                let mut sheet = pending.sheet.take().expect("checked stylesheet owner");
                sheet.SetLinkMetadata(
                    pending.owner_node_id,
                    pending.alternate,
                    false,
                    pending.title.clone(),
                    pending.media.clone(),
                );
                let loaded = !pending.import_failed;
                match self.AddParsedStyleSheet(sheet, &pending.base) {
                    Ok(()) => {
                        if client.HasJavaScript() {
                            client.DispatchStyleSheetEvent(
                                pending.owner_node_id,
                                pending.signature,
                                loaded,
                            );
                        }
                    }
                    Err(error) => {
                        client.ApplyResourceMutation(ResourceMutation::ResourceLoadFailed(
                            ResourceLoadFailed {
                                kind: ResourceKind::kStyleSheet,
                                source: None,
                                url: pending.url,
                                error: error.to_string(),
                            },
                        ));
                    }
                }
            } else {
                self.pending_style_sheets.borrow_mut().insert(0, pending);
            }
        }
        completed
    }

    pub fn LoadPendingStyleSheets(&self, client: &mut dyn ResourceFetcherClient) {
        while self.HasPendingStyleSheets() {
            if self.PollPendingStyleSheetsWithBudget(client, usize::MAX) == 0 {
                std::thread::sleep(Duration::from_millis(1));
            }
        }
    }
    // cpp: browser/browser.cc:1130-1172
    pub fn QueueFontFace(&self, rule: &CSSFontFaceRule, base: &str) -> io::Result<()> {
        self.QueueCompiledFontFace(rule, base)
    }
    // Register immutable face descriptors with FontEngine before LayoutText
    // and shaping report actual text demand. Keep dormant definitions resident
    // so later text/style mutations can activate them without another sheet.
    // cpp: core/css/font_face_set_document.cc; core/css/css_font_selector.cc
    pub fn PrepareFontFaces(&self) {
        let set = self.FontLoadingDocument().FontFaceSet();
        let faces = self.pending_fonts.borrow();
        let metadata = faces
            .iter()
            .enumerate()
            .filter(|(_, face)| set.Has(&face.owner))
            .map(|(index, face)| font_engine::WebFontFace {
                rule_id: index as u64,
                family: face.metadata.family.clone(),
                weight: face.metadata.weight,
                italic: face.metadata.italic,
                unicode_ranges: face.metadata.unicode_ranges.clone(),
                loaded: face.owner.LoadStatus() == dom::font_loading::FontFaceLoadStatus::Loaded,
                failed: face.owner.LoadStatus() == dom::font_loading::FontFaceLoadStatus::Error,
            })
            .collect();
        self.constraints
            .borrow_mut()
            .font_queries
            .get_or_insert_with(Default::default)
            .RegisterWebFonts(metadata);
    }
    // cpp: browser/browser.cc:1198-1208
    pub fn StartPendingFonts(&self) {
        self.PrepareFontFaces();
        if let Some(queries) = &self.constraints.borrow().font_queries {
            let requested = queries.DrainWebFontRequests();
            if std::env::var_os("RECHROM_TRACE_FONT_BINDING").is_some() && !requested.is_empty() {
                eprintln!("FONT_FACE_DEMAND ids={requested:?}");
            }
            let mut faces = self.pending_fonts.borrow_mut();
            for id in requested {
                if let Some(face) = faces.get_mut(id as usize) {
                    face.selected = true;
                }
            }
        }
        let count = self.pending_fonts.borrow().len();
        for index in 0..count {
            self.StartFontAt(index);
        }
    }
    // cpp: browser/browser.cc:1247-1258
    pub fn QueueImage(&self, source: &str, resolved: Option<&str>) -> io::Result<()> {
        self.QueueImageWithLoadBlocking(source, resolved, true)
    }

    /// Chromium's ImageLoader neither starts a deferred lazy image immediately
    /// nor holds the document load event for it.  Keep the request dormant
    /// until the document has loaded; an eager consumer of the same source
    /// promotes it immediately without creating a duplicate request.
    pub fn QueueImageWithLoadBlocking(
        &self,
        source: &str,
        resolved: Option<&str>,
        blocks_load: bool,
    ) -> io::Result<()> {
        self.QueueImageWithPolicy(source, resolved, blocks_load, None)
    }

    /// Keep the element's explicit policy through deferred admission so the
    /// authoritative request can coalesce with the scanner's same image.
    pub fn QueueImageWithPolicy(
        &self,
        source: &str,
        resolved: Option<&str>,
        blocks_load: bool,
        referrer_policy: Option<ReferrerPolicy>,
    ) -> io::Result<()> {
        let url = match resolved {
            Some(url) => url.into(),
            None => ResolveUrl(&self.base_url.borrow(), source)?,
        };
        if !self.loaded_image_sources.borrow_mut().insert(source.into()) {
            if blocks_load {
                if let Some(image) = self
                    .pending_images
                    .borrow_mut()
                    .iter_mut()
                    .find(|image| image.source == source)
                {
                    image.blocks_load = true;
                    return Ok(());
                }
                let deferred = self
                    .deferred_images
                    .borrow()
                    .iter()
                    .position(|image| image.source == source);
                if let Some(index) = deferred {
                    let image = self.deferred_images.borrow_mut().remove(index);
                    let (resource, content) = self.StartImageRequest(
                        &image.source,
                        image.request.unwrap_or_else(|| URLRequest {
                            url: image.url.clone(),
                            referrer_policy: image.referrer_policy,
                            destination: RequestDestination::kImage,
                            ..Default::default()
                        }),
                    );
                    self.pending_images.borrow_mut().push(PendingImage {
                        source: image.source,
                        url: image.url,
                        resource,
                        content,
                        decode: None,
                        is_multipart_response: None,
                        is_cors_same_origin: None,
                        content_dpr: None,
                        blocks_load: true,
                    });
                }
            }
            return Ok(());
        }
        if !blocks_load {
            self.deferred_images.borrow_mut().push(DeferredImage {
                request: None,
                source: source.into(),
                url,
                referrer_policy,
            });
            return Ok(());
        }
        let (resource, content) = self.StartImageRequest(
            source,
            URLRequest {
                url: url.clone(),
                referrer_policy,
                destination: RequestDestination::kImage,
                ..Default::default()
            },
        );
        self.pending_images.borrow_mut().push(PendingImage {
            source: source.into(),
            url,
            resource,
            content,
            decode: None,
            is_multipart_response: None,
            is_cors_same_origin: None,
            content_dpr: None,
            blocks_load,
        });
        Ok(())
    }

    /// Promote lazy images whose layout boxes have entered the viewport.
    ///
    /// A visible image must not wait behind every earlier below-the-fold image
    /// in a long document; Blink's lazy-load observer gives these requests
    /// immediate high-priority admission when their intersection state changes.
    pub fn StartDeferredImageSources(&self, sources: &HashSet<String>) -> (usize, usize) {
        if sources.is_empty() {
            return (0, 0);
        }
        let mut promoted = Vec::new();
        {
            let mut deferred = self.deferred_images.borrow_mut();
            let mut index = 0;
            while index < deferred.len() {
                if sources.contains(&deferred[index].source) {
                    promoted.push(deferred.remove(index));
                } else {
                    index += 1;
                }
            }
        }
        let count = promoted.len();
        let mut prioritized = 0;
        for image in self.pending_images.borrow_mut().iter_mut() {
            if sources.contains(&image.source) {
                image
                    .resource
                    .SetPriority(url_loader::ResourceLoadPriority::High);
                prioritized += 1;
            }
        }
        for image in promoted {
            let (mut resource, content) = self.StartImageRequest(
                &image.source,
                image.request.unwrap_or_else(|| URLRequest {
                    url: image.url.clone(),
                    referrer_policy: image.referrer_policy,
                    destination: RequestDestination::kImage,
                    ..Default::default()
                }),
            );
            resource.SetPriority(url_loader::ResourceLoadPriority::High);
            self.pending_images.borrow_mut().push(PendingImage {
                source: image.source,
                url: image.url,
                resource,
                content,
                decode: None,
                is_multipart_response: None,
                is_cors_same_origin: None,
                content_dpr: None,
                blocks_load: false,
            });
        }
        // A transport wake identifies that some resource progressed, while
        // this fetcher owns the pending-image collection and polls it with a
        // bounded completion budget. Keep the current viewport at the front
        // so unrelated offscreen/cache completions cannot consume every wake
        // before a just-promoted visible image is observed. Stable ordering
        // preserves completion/event order within each visibility class.
        self.pending_images
            .borrow_mut()
            .sort_by_key(|image| !sources.contains(&image.source));
        (count, prioritized)
    }

    /// Ensure every currently visible image dependency has a request, then
    /// raise queued work above ordinary offscreen transfers. Style resolution
    /// normally admits CSS images before Layout, but a newly attached or
    /// incrementally styled owner can become visible in the frame which first
    /// exposes its dependency. The visibility transaction is therefore also a
    /// lossless admission fallback, not only a lazy-image promotion pass.
    pub fn StartVisibleImageSources(
        &self,
        sources: &HashSet<String>,
    ) -> io::Result<(usize, usize)> {
        let previous = self.visible_image_sources.replace(sources.clone());
        if previous != *sources {
            for image in self.pending_images.borrow_mut().iter_mut() {
                if previous.contains(&image.source) && !sources.contains(&image.source) {
                    // Dynamic viewport priority is reversible. Keeping an
                    // offscreen transfer in the critical class lets the old
                    // viewport monopolize its reserved connections after a
                    // long scroll, starving the resources visible now.
                    image
                        .resource
                        .SetPriority(url_loader::ResourceLoadPriority::Normal);
                }
            }
        }
        for source in sources {
            if !self.loaded_image_sources.borrow().contains(source) {
                // Visibility discovery happens after the document's load
                // accounting snapshot. Admit a missing request without
                // retroactively holding the load event, then promote it below.
                self.QueueImageWithPolicy(source, None, false, None)?;
            }
        }
        Ok(self.StartDeferredImageSources(sources))
    }
    // cpp: browser/browser.cc:1260-1291
    fn BeginImageResult(
        &self,
        image: &mut PendingImage,
        result: ResourceLoadResult,
    ) -> io::Result<decode::ImageDecodeJob> {
        let Some(response) = result.response else {
            return Err(io::Error::other(result.error));
        };
        // Source: platform/graphics/image.h is_multipart_; image.cc:357.
        // The transport publishes one completed response here; preserve the
        // actual MIME classification instead of guessing in the compositor.
        image.is_multipart_response = Some(
            response
                .mime_type
                .split(';')
                .next()
                .expect("MIME first component")
                .trim()
                .eq_ignore_ascii_case("multipart/x-mixed-replace"),
        );
        let encoded_data: Arc<[u8]> = Arc::from(response.body);
        image
            .content
            .SetEncodedResource(encoded_data.clone(), response.content_disposition.clone());
        self.RetainImageResponseURL(
            &image.source,
            &response.final_url,
            encoded_data.clone(),
            response.mime_type.clone(),
            response.content_disposition.clone(),
        );
        image.is_cors_same_origin = Some(
            image.content.CORSEnabled()
                || self.engine.IsSameOriginWithDocument(&response.final_url),
        );
        image.content_dpr = response.content_dpr;
        ImageResultBoundary(|| {
            let mut container =
                ContainerKey::new(0, 0, 1.0, self.constraints.borrow().device_pixel_ratio);
            container.view_specification = SVGImageViewSpecification(&image.source).to_owned();
            self.engine
                .BeginImageDecode(encoded_data, &response.mime_type, &container)
        })
    }
    fn ApplyImageResult(
        &self,
        image: PendingImage,
        result: io::Result<DecodedImageResource>,
        client: &mut dyn ResourceFetcherClient,
    ) {
        let decoded = ImageResultBoundary(|| -> io::Result<_> {
            let decoded_resource = result?;
            let is_multipart = image.is_multipart_response.ok_or_else(|| {
                io::Error::other("image transport did not publish its response classification")
            })?;
            let is_cors_same_origin = image.is_cors_same_origin.ok_or_else(|| {
                io::Error::other("image transport did not publish its origin provenance")
            })?;
            let (id, decoded, document_image) = match decoded_resource {
                DecodedImageResource::Document { resource_id, frame } => {
                    // SVGImage::FilenameExtension is source "svg"; network
                    // response size is retained on the same resource info.
                    image.content.SetDecodedMediaFacts(
                        Some("svg"),
                        image.content.EncodedResourceDataSize(),
                        frame.animation_info.animation_type
                            == image_resource::PaintImageAnimationType::Animated,
                        frame.completion_state == image_resource::PaintImageCompletionState::Done,
                    );
                    let paint_image = PaintImage {
                        id: resource_id,
                        paint_image_id: image_resource::PaintImageId::allocate(),
                        compositor_metadata: Ok(image_resource::PaintImageMetadata {
                            animation: image_resource::PaintImageAnimationMetadata::Known(
                                frame.animation_info,
                            ),
                            completion_state: frame.completion_state,
                            is_multipart,
                        }),
                        revision: frame.revision,
                        width: frame.intrinsic_size.width,
                        height: frame.intrinsic_size.height,
                        content_dpr: image.content_dpr,
                        is_cors_same_origin,
                        document_container_key: Some(frame.container_key.clone()),
                        content: PaintImageContent::Document(Arc::new(DocumentPaintContent::new(
                            frame.container_key.clone(),
                            frame.record.clone(),
                        ))),
                        ..Default::default()
                    };
                    (resource_id, paint_image, true)
                }
                DecodedImageResource::Bitmap {
                    resource_id,
                    image: bitmap,
                    encoded_data,
                } => {
                    // BitmapImage retains a concrete decoder/frame owner for
                    // animated formats before it publishes a PaintImage. The
                    // decoded frame set below is that producer-owned value;
                    // formats which have not published their complete frame
                    // set remain recoverable resource errors at this boundary.
                    //
                    // Source: third_party/blink/renderer/platform/graphics/
                    // bitmap_image.cc:155-166; image.cc:217-234.
                    if let image_resource::PaintImageAnimationMetadata::Unavailable(error) =
                        &bitmap.animation
                    {
                        return Err(io::Error::new(io::ErrorKind::Unsupported, error.clone()));
                    }
                    let animated = matches!(&bitmap.animation, image_resource::PaintImageAnimationMetadata::Known(info) if info.animation_type == image_resource::PaintImageAnimationType::Animated);
                    image.content.SetDecodedMediaFacts(
                        bitmap.filename_extension,
                        encoded_data.len() as u64,
                        animated,
                        true,
                    );
                    let animation_frames = (!bitmap.animation_frames.is_empty()).then(|| {
                        Arc::new(
                            bitmap
                                .animation_frames
                                .into_iter()
                                .map(|frame| image_resource::PaintImageFrame {
                                    duration: frame.duration,
                                    pixels: Arc::new(frame.rgba8),
                                })
                                .collect(),
                        )
                    });
                    (
                        resource_id,
                        PaintImage {
                            id: resource_id,
                            paint_image_id: image_resource::PaintImageId::allocate(),
                            // Source: platform/graphics/bitmap_image.cc:155-166.
                            // Only full-response successful decode reaches this
                            // point; incremental/partial image publication is a
                            // separate contract that this loader does not emit.
                            compositor_metadata: Ok(image_resource::PaintImageMetadata {
                                animation: bitmap.animation,
                                completion_state: image_resource::PaintImageCompletionState::Done,
                                is_multipart,
                            }),
                            revision: 1,
                            width: bitmap.width,
                            height: bitmap.height,
                            content_dpr: image.content_dpr,
                            is_cors_same_origin,
                            orientation: bitmap.orientation,
                            hot_spot: bitmap.hot_spot,
                            alpha_type: bitmap.alpha_type,
                            // BitmapImage retains its encoded-data generator
                            // identity after the first frame is available.
                            // SoftwareImageDecodeCache uses that producer fact
                            // to create Chromium's mip-sized decode allocation;
                            // the decoded RGBA bytes remain the immutable
                            // candidate when the codec cannot decode to scale.
                            is_lazy_generated: true,
                            encoded_data: Some(encoded_data),
                            content: PaintImageContent::Bitmap(bitmap.rgba8.into()),
                            animation_frames,
                            ..Default::default()
                        },
                        false,
                    )
                }
            };
            image
                .content
                .SetBitmapProvider(self.engine.ImageBitmapProvider(id));
            // Source catches validation while committing the decoded image,
            // as well as decoder failures. Keep the consumed id and continue
            // loading other resources after a rejected image.
            let metadata = dom::ImageResourceMetadata {
                id: decoded.id,
                natural_width: decoded.width as f64 / decoded.resolution_scale,
                natural_height: decoded.height as f64 / decoded.resolution_scale,
                resolution_scale: decoded.resolution_scale,
                orientation: decoded.orientation,
                known_to_be_opaque: decoded
                    .BitmapPixels()
                    .is_some_and(|pixels| pixels.chunks_exact(4).all(|pixel| pixel[3] == 255)),
                is_cors_same_origin: decoded.is_cors_same_origin,
            };
            self.CommitImage(
                image.source.clone(),
                decoded,
                Some(image.content.clone()),
                client,
            );
            self.CompleteHTMLImageContent(&image.content, Some(metadata), false);
            if document_image {
                self.document_image_sources
                    .borrow_mut()
                    .insert(id, image.source.clone());
            }
            Ok(())
        });
        match decoded {
            Ok(()) => {}
            Err(e) => {
                self.engine.RemoveImageFromMemoryCache(&image.resource);
                self.CompleteHTMLImageContent(
                    &image.content,
                    None,
                    image.resource.AccessCheckFailed(),
                );
                client.ApplyResourceMutation(ResourceMutation::ResourceLoadFailed(
                    ResourceLoadFailed {
                        kind: ResourceKind::kImage,
                        source: Some(image.source.clone()),
                        url: image.url,
                        error: e.to_string(),
                    },
                ));
            }
        }
    }
    fn BeginVideoResult(
        &self,
        video: &mut PendingVideo,
        result: ResourceLoadResult,
    ) -> io::Result<decode::VideoDecodeJob> {
        let Some(response) = result.response else {
            return Err(io::Error::other(result.error));
        };
        video.is_cors_same_origin = Some(self.engine.IsSameOriginWithDocument(&response.final_url));
        let declared = response.mime_type.split(';').next().unwrap_or("").trim();
        let mime_type = if matches!(declared, "video/mp4" | "video/quicktime") {
            declared
        } else if video.url.to_ascii_lowercase().ends_with(".mov") {
            "video/quicktime"
        } else {
            "video/mp4"
        };
        self.engine
            .BeginVideoFrameDecode(Arc::from(response.body), mime_type)
    }
    fn ApplyVideoResult(
        &self,
        video: PendingVideo,
        result: io::Result<resource::DecodedVideoFrame>,
        client: &mut dyn ResourceFetcherClient,
    ) {
        match result {
            Ok(decoded) => {
                let bitmap = decoded.image;
                client.ApplyResourceMutation(ResourceMutation::VideoFrameChanged(
                    VideoFrameChanged {
                        node_id: video.node_id,
                        frame: PaintImage {
                            id: decoded.resource_id,
                            paint_image_id: image_resource::PaintImageId::allocate(),
                            compositor_metadata: Ok(image_resource::PaintImageMetadata {
                                animation: image_resource::PaintImageAnimationMetadata::Known(
                                    image_resource::PaintImageAnimationInfo {
                                        animation_type:
                                            image_resource::PaintImageAnimationType::Video,
                                        repetition_count: image_resource::ANIMATION_NONE,
                                        frame_count: 1,
                                    },
                                ),
                                completion_state: image_resource::PaintImageCompletionState::Done,
                                is_multipart: false,
                            }),
                            revision: 1,
                            width: bitmap.width,
                            height: bitmap.height,
                            is_cors_same_origin: video.is_cors_same_origin.unwrap_or(false),
                            orientation: bitmap.orientation,
                            hot_spot: bitmap.hot_spot,
                            alpha_type: bitmap.alpha_type,
                            content: PaintImageContent::Bitmap(Arc::new(bitmap.rgba8)),
                            ..Default::default()
                        },
                        resource: decoded.resource,
                    },
                ));
            }
            Err(error) => {
                client.ApplyResourceMutation(ResourceMutation::ResourceLoadFailed(
                    ResourceLoadFailed {
                        kind: ResourceKind::kMedia,
                        source: None,
                        url: video.url,
                        error: error.to_string(),
                    },
                ));
            }
        }
    }
    /// A changed img request may revisit its previous URL before a queued DOM
    /// event runs. Deduplication must follow the request, not the URL forever.
    pub fn DispatchImageEvent(
        &self,
        id: u64,
        loaded: bool,
        client: &mut dyn ResourceFetcherClient,
    ) {
        if !client.HasJavaScript() {
            return;
        }
        let ticket = {
            let owner = self.document.borrow();
            let d = owner.GetDocument();
            d.FindNodeById(id)
                .and_then(|node| d.ExistingHTMLImageLoader(node))
                .and_then(|loader| loader.EventTicket())
        };
        if ticket.is_some_and(|ticket| {
            (ticket.event == dom::persistent_document::ImageLoaderEvent::Load) == loaded
        }) {
            client.DispatchImageEvent(id, loaded);
        }
    }
    // cpp: browser/browser.cc:1359-1393
    pub fn QueueReferencedImages(&self) -> io::Result<bool> {
        let mut requests = Vec::new();
        let mut selections = Vec::new();
        let selector = self.responsive_image_selector.borrow().clone();
        fn request(
            source: &str,
            resolve: bool,
            blocks_load: bool,
            referrer_policy: Option<ReferrerPolicy>,
            admitted: &HashSet<String>,
            seen: &mut HashSet<String>,
            requests: &mut Vec<(String, bool, bool, Option<ReferrerPolicy>)>,
        ) {
            // QueueImage uses this exact source key for admission. Keep the
            // first occurrence, without cloning/resolving every repeated owner.
            // CSS sources already carry their stylesheet base via ResolveCSSURLs;
            // distinct DOM/CSS source strings remain distinct requests.
            if !admitted.contains(source) && seen.insert(source.to_owned()) {
                requests.push((source.to_owned(), resolve, blocks_load, referrer_policy));
            } else if blocks_load {
                if let Some(request) = requests.iter_mut().find(|request| request.0 == source) {
                    request.2 = true;
                }
            }
        }
        fn image_node(
            d: &Document,
            i: usize,
            constraints: &ConstraintSpace,
            selector: Option<&ResponsiveImageSelector>,
            is_loaded: &dyn Fn(&str) -> bool,
            admitted: &HashSet<String>,
            seen: &mut HashSet<String>,
            requests: &mut Vec<(String, bool, bool, Option<ReferrerPolicy>)>,
            selections: &mut Vec<(
                usize,
                Option<dom::persistent_document::ResponsiveImageSelection>,
            )>,
        ) {
            let n = d.Node(i);
            debug_assert!(
                n.IsHTMLElement("img")
                    || d.InputType(i) == Some(dom::persistent_document::DOMInputType::kImage)
            );
            let selection = if let Some(selector) = selector.filter(|_| n.IsHTMLElement("img")) {
                selector(d, i, constraints, is_loaded)
            } else {
                n.FindAttribute("src")
                    .filter(|attribute| !attribute.value.is_empty())
                    .map(
                        |attribute| dom::persistent_document::ResponsiveImageSelection {
                            source: attribute.value.clone(),
                            density: None,
                        },
                    )
            };
            selections.push((i, selection));
        }
        {
            let owner = self.document.borrow();
            let d = owner.GetDocument();
            let admitted = self.loaded_image_sources.borrow();
            let constraints = self.constraints.borrow();
            let mut seen = HashSet::new();
            let is_loaded = |source: &str| {
                d.ImageResourceFor(source).is_some() || self.IsLoadedImageSource(source)
            };
            let key = d.CollectionMembershipKey();
            let mut cached = self.referenced_image_owners.borrow_mut();
            if key.is_none() || cached.as_ref().is_none_or(|owners| owners.key != key) {
                fn visit(
                    d: &Document,
                    node: usize,
                    order: &mut HashMap<usize, usize>,
                    image_nodes: &mut Vec<usize>,
                ) {
                    order.insert(node, order.len());
                    if d.Node(node).IsHTMLElement("img")
                        || d.InputType(node) == Some(dom::persistent_document::DOMInputType::kImage)
                    {
                        image_nodes.push(node);
                    }
                    // Blink resolves pending resources for shadow descendants
                    // before light children, including unassigned light DOM.
                    if let Some(shadow_root) = d.ShadowRoot(node) {
                        visit(d, shadow_root, order, image_nodes);
                    }
                    for &child in d.Node(node).Children() {
                        visit(d, child, order, image_nodes);
                    }
                }
                let mut order = HashMap::new();
                let mut image_nodes = Vec::new();
                visit(d, d.Root(), &mut order, &mut image_nodes);
                // ImageLoader belongs to HTMLImageElement and also operates
                // while detached. Preserve the previous arena creation order.
                for node in 0..d.NodeCount() {
                    if !d.IsConnectedToDocument(node)
                        && (d.Node(node).IsHTMLElement("img")
                            || d.InputType(node)
                                == Some(dom::persistent_document::DOMInputType::kImage))
                    {
                        order.insert(node, order.len());
                        image_nodes.push(node);
                    }
                }
                *cached = Some(ReferencedImageOwners {
                    key,
                    order,
                    image_nodes,
                });
            }
            let owners = cached.as_ref().unwrap();
            let mut style_owners = d
                .ReferencedStyleImageConsumers()
                .into_iter()
                .filter_map(|(node, sources)| {
                    owners
                        .order
                        .get(&node)
                        .copied()
                        .map(|order| (order, sources))
                })
                .collect::<Vec<_>>();
            style_owners.sort_unstable_by_key(|(order, _)| *order);
            let mut style = style_owners.into_iter().peekable();
            for &node in &owners.image_nodes {
                let node_order = owners.order[&node];
                while style.peek().is_some_and(|(order, _)| *order < node_order) {
                    for source in style.next().unwrap().1.iter() {
                        if !source.is_empty() {
                            request(
                                source,
                                true,
                                true,
                                None,
                                &admitted,
                                &mut seen,
                                &mut requests,
                            );
                        }
                    }
                }
                image_node(
                    d,
                    node,
                    &constraints,
                    selector.as_deref(),
                    &is_loaded,
                    &admitted,
                    &mut seen,
                    &mut requests,
                    &mut selections,
                );
                if style.peek().is_some_and(|(order, _)| *order == node_order) {
                    for source in style.next().unwrap().1.iter() {
                        if !source.is_empty() {
                            request(
                                source,
                                true,
                                true,
                                None,
                                &admitted,
                                &mut seen,
                                &mut requests,
                            );
                        }
                    }
                }
            }
            for (_, sources) in style {
                for source in sources.iter() {
                    if !source.is_empty() {
                        request(
                            source,
                            true,
                            true,
                            None,
                            &admitted,
                            &mut seen,
                            &mut requests,
                        );
                    }
                }
            }
            if key.is_none() {
                cached.take();
            }
        }
        let loader_nodes = selections.iter().map(|(node, _)| *node).collect::<Vec<_>>();
        let selection_changed = {
            let mut owner = self.document.borrow_mut();
            let document = owner.GetDocumentMut();
            let mut selection_changed = false;
            for (node, selection) in selections {
                selection_changed |= document.SetResponsiveImageSelection(node, selection);
            }
            selection_changed
        };
        for (source, resolve, blocks_load, referrer_policy) in requests {
            let url = if resolve {
                Some(ResolveUrl(&self.base_url.borrow(), &source)?)
            } else {
                None
            };
            self.QueueImageWithPolicy(&source, url.as_deref(), blocks_load, referrer_policy)?;
        }
        for node in loader_nodes {
            self.UpdateHTMLImageLoader(
                node,
                dom::persistent_document::ImageLoaderUpdate::Normal,
                false,
            )?;
        }
        self.QueueReferencedVideos()?;
        Ok(selection_changed)
    }
    // cpp: core/html/media/html_media_element.cc:1309-1405
    fn QueueReferencedVideos(&self) -> io::Result<()> {
        let requests = {
            let owner = self.document.borrow();
            let document = owner.GetDocument();
            let attempted = self.attempted_video_nodes.borrow();
            (0..document.NodeCount())
                .filter_map(|index| {
                    let video = document.Node(index);
                    if !document.IsConnectedToDocument(index)
                        || !video.IsHTMLElement("video")
                        || attempted.contains(&video.Id())
                        || document.VideoFrameMetadataFor(video.Id()).is_some()
                    {
                        return None;
                    }
                    let direct = video
                        .FindAttribute("src")
                        .filter(|attribute| !attribute.value.is_empty())
                        .map(|attribute| attribute.value.clone());
                    let child_source = video.Children().iter().find_map(|&child| {
                        let source = document.Node(child);
                        if !source.IsHTMLElement("source")
                            || source.FindAttribute("type").is_some_and(|kind| {
                                !matches!(
                                    kind.value.split(';').next().unwrap_or("").trim(),
                                    "video/mp4" | "video/quicktime" | ""
                                )
                            })
                        {
                            return None;
                        }
                        source
                            .FindAttribute("src")
                            .filter(|attribute| !attribute.value.is_empty())
                            .map(|attribute| attribute.value.clone())
                    });
                    direct.or(child_source).map(|source| (video.Id(), source))
                })
                .collect::<Vec<_>>()
        };
        for (node_id, source) in requests {
            let url = ResolveUrl(&self.base_url.borrow(), &source)?;
            let resource = self.engine.Start(URLRequest {
                url: url.clone(),
                destination: RequestDestination::kVideo,
                ..Default::default()
            });
            self.attempted_video_nodes.borrow_mut().insert(node_id);
            self.pending_videos.borrow_mut().push(PendingVideo {
                node_id,
                url,
                resource,
                decode: None,
                is_cors_same_origin: None,
            });
        }
        Ok(())
    }
    pub fn HasPendingImages(&self) -> bool {
        !self.pending_images.borrow().is_empty()
            || !self.pending_videos.borrow().is_empty()
            || !self.html_images.tasks.borrow().is_empty()
            || !self.html_images.ready_events.borrow().is_empty()
    }
    /// Viewport observation is useful only while an image can still be
    /// promoted or reprioritized. Loaded resources must not keep forcing a
    /// lifecycle geometry pass on every animation/scroll frame.
    pub fn HasViewportImageWork(&self) -> bool {
        !self.deferred_images.borrow().is_empty() || !self.pending_images.borrow().is_empty()
    }
    pub fn HasPendingLoadBlockingImages(&self) -> bool {
        self.pending_images
            .borrow()
            .iter()
            .any(|image| image.blocks_load)
    }
    pub fn HasAnimatedImages(&self) -> bool {
        self.engine.HasAnimatedImages()
    }
    /// Publish the concrete environment for one SVG image use. Chromium's
    /// SVGImageForContainer owns this value per paint client. Rechrom's
    /// records each returned frame as a ContainerKey variant; callers may
    /// therefore configure multiple simultaneous uses without overwriting an
    /// earlier client's recording.
    ///
    pub fn ConfigureDocumentImageContainer(
        &self,
        resource_id: image_resource::ImageId,
        container: ContainerKey,
        preferred_color_scheme: u8,
        client: &mut dyn ResourceFetcherClient,
    ) -> io::Result<bool> {
        self.ConfigureDocumentImageContainerWithView(
            resource_id,
            container,
            preferred_color_scheme,
            None,
            client,
        )
    }

    /// An explicit view preserves the caller's SVGImageViewInfo decision.
    /// `Some("")` denotes SVGImageForContainer::Create's null viewinfo;
    /// ordinary image uses resolve the resource URL fragment with `None`.
    pub fn ConfigureDocumentImageContainerWithView(
        &self,
        resource_id: image_resource::ImageId,
        mut container: ContainerKey,
        preferred_color_scheme: u8,
        view_specification: Option<&str>,
        client: &mut dyn ResourceFetcherClient,
    ) -> io::Result<bool> {
        let source = self
            .document_image_sources
            .borrow()
            .get(&resource_id)
            .cloned()
            .ok_or_else(|| io::Error::other("document image source is missing"))?;
        // The resource owner, rather than a paint caller's previous current
        // frame, supplies the view. This prevents one use from inheriting a
        // last-recorded fragment and keeps ContainerKey variant lookup exact.
        container.view_specification = view_specification
            .unwrap_or_else(|| SVGImageViewSpecification(&source))
            .to_owned();
        if !container.preferred_color_scheme_is_explicit {
            container.preferred_color_scheme = preferred_color_scheme;
            container.preferred_color_scheme_is_explicit = true;
        }
        for effect in self.engine.MutateDocumentImage(
            resource_id,
            DocumentImageMutation::SetContainer { container },
        )? {
            match effect {
                DecodeEffect::DocumentImageFrameChanged { frame } => client.ApplyResourceMutation(
                    ResourceMutation::DocumentImageFrameChanged(DocumentImageFrameChanged {
                        source: source.clone(),
                        frame,
                    }),
                ),
                DecodeEffect::DocumentImageIntrinsicSizeChanged {
                    resource_id,
                    revision,
                    size,
                } => client.ApplyResourceMutation(
                    ResourceMutation::DocumentImageIntrinsicSizeChanged(
                        DocumentImageIntrinsicSizeChanged {
                            source: source.clone(),
                            resource_id,
                            revision,
                            size,
                        },
                    ),
                ),
                DecodeEffect::RequestBeginFrame { .. } => {}
                DecodeEffect::ImageReady(_) => {
                    return Err(io::Error::other(
                        "container mutation produced a non-container decode effect",
                    ));
                }
            }
        }
        Ok(true)
    }
    /// Sample nested document-image timelines at the embedder's rendering
    /// opportunity.  This emits ordinary immutable image frames; Page and
    /// layout never retain or inspect the SVG animation implementation.
    pub fn SampleAnimatedImages(
        &self,
        frame_time: Instant,
        begin_frame_sequence: u64,
        client: &mut dyn ResourceFetcherClient,
    ) -> io::Result<usize> {
        let mut sampled = 0;
        for effect in self
            .engine
            .AdvanceDocumentImages(frame_time, begin_frame_sequence)?
        {
            match effect {
                DecodeEffect::DocumentImageFrameChanged { frame } => {
                    let source = self
                        .document_image_sources
                        .borrow()
                        .get(&frame.resource_id)
                        .cloned()
                        .ok_or_else(|| io::Error::other("document image source is missing"))?;
                    client.ApplyResourceMutation(ResourceMutation::DocumentImageFrameChanged(
                        DocumentImageFrameChanged { source, frame },
                    ));
                    sampled += 1;
                }
                DecodeEffect::DocumentImageIntrinsicSizeChanged {
                    resource_id,
                    revision,
                    size,
                } => {
                    let source = self
                        .document_image_sources
                        .borrow()
                        .get(&resource_id)
                        .cloned()
                        .ok_or_else(|| io::Error::other("document image source is missing"))?;
                    client.ApplyResourceMutation(
                        ResourceMutation::DocumentImageIntrinsicSizeChanged(
                            DocumentImageIntrinsicSizeChanged {
                                source,
                                resource_id,
                                revision,
                                size,
                            },
                        ),
                    );
                }
                DecodeEffect::RequestBeginFrame { .. } => {}
                DecodeEffect::ImageReady(_) => {
                    return Err(io::Error::other(
                        "timeline mutation produced a non-timeline decode effect",
                    ));
                }
            }
        }
        Ok(sampled)
    }
    pub fn HasPendingFonts(&self) -> bool {
        self.pending_fonts.borrow().iter().any(|face| {
            face.selected
                && face.owner.LoadStatus() == dom::font_loading::FontFaceLoadStatus::Loading
        }) || self
            .font_loading
            .preloads
            .borrow()
            .values()
            .any(|(_, preload)| preload.Finished().is_none() && preload.HasResource())
    }
    /// Poll the current image set once. Page receives mutations/events only for
    /// completed requests; outstanding IO never causes a wait here.
    pub fn PollPendingImages(&self, client: &mut dyn ResourceFetcherClient) -> usize {
        self.PollPendingImagesWithBudget(client, usize::MAX)
    }
    /// Apply at most max_completions responses. Page combines this allowance
    /// with its other ready work; synchronous adapters keep the unbounded poll.
    pub fn PollPendingImagesWithBudget(
        &self,
        client: &mut dyn ResourceFetcherClient,
        max_completions: usize,
    ) -> usize {
        self.RunPendingHTMLImageLoaderTasks();
        let started = std::env::var_os("BROWSER_PROFILE_INPUT")
            .is_some()
            .then(std::time::Instant::now);
        let count = self.pending_images.borrow().len();
        let mut completed = 0;
        let mut admitted = 0;
        let events = {
            let mut events = self.html_images.ready_events.borrow_mut();
            let count = events.len().min(max_completions);
            events.drain(..count).collect::<Vec<_>>()
        };
        for ticket in events {
            if let Some(_delay) = ticket.loader.BeginEventDispatch(&ticket) {
                if client.HasJavaScript() {
                    client.DispatchImageEvent(
                        ticket.loader.Element().Id(),
                        ticket.event == dom::persistent_document::ImageLoaderEvent::Load,
                    );
                }
                completed += 1;
            }
        }
        let mut index = 0;
        for _ in 0..count {
            if completed + admitted >= max_completions {
                break;
            }
            let mut image = {
                let mut images = self.pending_images.borrow_mut();
                if index >= images.len() {
                    break;
                }
                images.remove(index)
            };
            let result = if let Some(job) = &mut image.decode {
                job.poll()
            } else if image.resource.Poll() {
                let response = image.resource.TakeResult();
                match self.BeginImageResult(&mut image, response) {
                    Ok(mut job) => {
                        let result = job.poll();
                        if result.is_none() {
                            admitted += 1;
                        }
                        image.decode = Some(job);
                        result
                    }
                    Err(error) => Some(Err(error)),
                }
            } else {
                None
            };
            let Some(result) = result else {
                self.pending_images.borrow_mut().insert(index, image);
                index += 1;
                continue;
            };
            self.ApplyImageResult(image, result, client);
            completed += 1;
        }
        let video_count = self.pending_videos.borrow().len();
        let mut video_index = 0;
        for _ in 0..video_count {
            if completed + admitted >= max_completions {
                break;
            }
            let mut video = {
                let mut videos = self.pending_videos.borrow_mut();
                if video_index >= videos.len() {
                    break;
                }
                videos.remove(video_index)
            };
            let result = if let Some(job) = &mut video.decode {
                job.poll()
            } else if video.resource.Poll() {
                let response = video.resource.TakeResult();
                match self.BeginVideoResult(&mut video, response) {
                    Ok(mut job) => {
                        let result = job.poll();
                        if result.is_none() {
                            admitted += 1;
                        }
                        video.decode = Some(job);
                        result
                    }
                    Err(error) => Some(Err(error)),
                }
            } else {
                None
            };
            let Some(result) = result else {
                self.pending_videos.borrow_mut().insert(video_index, video);
                video_index += 1;
                continue;
            };
            self.ApplyVideoResult(video, result, client);
            completed += 1;
        }
        if let Some(start) = started {
            if start.elapsed().as_millis() >= 50 {
                eprintln!(
                    "image-poll-profile initial={count} completed={completed} pending={} ms={:.3}",
                    self.pending_images.borrow().len() + self.pending_videos.borrow().len(),
                    start.elapsed().as_secs_f64() * 1000.0
                );
            }
        }
        completed
    }
    /// One font admission/completion pass. A failed source's fallback is
    /// admitted on a later pass, preserving source order without a busy wait.
    pub fn PollPendingFonts(&self, client: &mut dyn ResourceFetcherClient) -> usize {
        self.PollPendingFontsWithBudget(client, usize::MAX)
    }
    pub fn PollPendingFontsWithBudget(
        &self,
        client: &mut dyn ResourceFetcherClient,
        max_completions: usize,
    ) -> usize {
        self.StartPendingFonts();
        let resources = self.FontResourcesSnapshot();
        let mut completed = 0;
        for resource in resources {
            if completed >= max_completions {
                break;
            }
            if resource.Poll() {
                completed += 1;
            }
        }
        self.CompleteFontResources();
        for mutation in self.TakeFontResourceMutations() {
            client.ApplyResourceMutation(mutation);
        }
        self.PrepareFontFaces();
        completed
    }
    // cpp: browser/browser.cc:1395-1407
    /// Existing synchronous adapter, used until Page's frame loop polls loads.
    pub fn LoadPendingImages(&self, client: &mut dyn ResourceFetcherClient) {
        while self.HasPendingImages() {
            if self.PollPendingImages(client) == 0 {
                std::thread::sleep(Duration::from_millis(1));
            }
        }
    }
    // cpp: browser/browser.cc:1409-1425
    pub fn LoadPendingFonts(&self, client: &mut dyn ResourceFetcherClient) {
        while self.HasPendingFonts() {
            self.PollPendingFonts(client);
            if self.HasPendingFonts() {
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        // Keep dormant and completed faces available for later text demand.
        // Resource handles are released by PollPendingFonts at completion.
    }
}

// Preserve the existing known resource-error boundary for document images and
// owner-side commit. Unknown invariant failures remain Page failures.
fn ImageResultBoundary<T>(work: impl FnOnce() -> io::Result<T>) -> io::Result<T> {
    let decoded = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work));
    match decoded {
        Ok(result) => result,
        Err(error) => {
            if let Some(error) = error.downcast_ref::<dom::error::DOMException>() {
                Err(io::Error::other(error.to_string()))
            } else if let Some(error) = error.downcast_ref::<foundation::UnsupportedLayout>() {
                Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    error.to_string(),
                ))
            } else if let Some(error) =
                error.downcast_ref::<layoutng_assembly::layout_engine::LayoutEngineError>()
            {
                match error {
                    layoutng_assembly::layout_engine::LayoutEngineError::InvalidArgument(
                        message,
                    ) => Err(io::Error::new(io::ErrorKind::InvalidInput, *message)),
                    layoutng_assembly::layout_engine::LayoutEngineError::RuntimeError(message) => {
                        Err(io::Error::other(*message))
                    }
                }
            } else {
                std::panic::resume_unwind(error)
            }
        }
    }
}

#[path = "html_image_loader.rs"]
mod html_image_loader;
