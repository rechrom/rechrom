#![allow(non_snake_case)]
//! Source Page resource state. Page supplies mutation and event delivery, so
//! decoding and polling do not replace the source DOM/dirty/task lifecycle.
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
    ImageResourceReady, ResourceKind, ResourceLoadFailed, ResourceMutation,
};
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
    fn HasJavaScript(&self) -> bool;
}
// cpp: browser/browser.cc:1104-1112
struct PendingFontFace {
    metadata: crate::web_fonts::PendingFontFace,
    next_url: usize,
    loaded: bool,
    selected: bool,
    resource: Option<ResourceLoader>,
    decode: Option<decode::DecodeJob<Vec<u8>>>,
}
// cpp: browser/browser.cc:1241-1245
struct PendingImage {
    source: String,
    url: String,
    resource: ResourceLoader,
    decode: Option<decode::ImageDecodeJob>,
    is_multipart_response: Option<bool>,
    blocks_load: bool,
}

// cpp: third_party/blink/renderer/core/html/link_style.h;
//      third_party/blink/renderer/core/loader/resource/css_style_sheet_resource.h
struct PendingStyleSheet {
    owner_node_id: u64,
    signature: String,
    url: String,
    resource: ResourceLoader,
    alternate: bool,
    title: String,
    media: String,
}

struct DeferredImage {
    source: String,
    url: String,
    referrer_policy: Option<ReferrerPolicy>,
}

struct ImageEventSubtrees {
    key: (dom::persistent_document::DOMOwnerHandle, u64),
    // Include detached trees: an earlier listener can detach a node that is
    // still present in the original traversal's already captured children.
    ancestors: HashMap<String, HashSet<usize>>,
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
    style_sheet_receiver: RefCell<Option<Rc<dyn Fn(&mut Document, CSSStyleSheet)>>>,
    engine: Rc<ResourceEngine>,
    document: Rc<RefCell<DOM>>,
    constraints: Rc<RefCell<ConstraintSpace>>,
    base_url: RefCell<String>,
    style_sheet_signatures: RefCell<HashMap<u64, String>>,
    pending_style_sheets: RefCell<Vec<PendingStyleSheet>>,
    loaded_image_sources: RefCell<HashSet<String>>,
    image_event_sources: RefCell<HashMap<u64, String>>,
    cached_image_event_sources: RefCell<HashSet<String>>,
    image_event_subtrees: RefCell<Option<ImageEventSubtrees>>,
    referenced_image_owners: RefCell<Option<ReferencedImageOwners>>,
    pending_images: RefCell<Vec<PendingImage>>,
    deferred_images: RefCell<Vec<DeferredImage>>,
    document_image_sources: RefCell<HashMap<u64, String>>,
    responsive_image_selector: RefCell<Option<Rc<ResponsiveImageSelector>>>,
    preload_image_selector: RefCell<Option<Rc<PreloadImageSelector>>>,
    preload_media_matcher: RefCell<Option<Rc<dyn Fn(&str, &ConstraintSpace) -> bool>>>,
    pending_fonts: RefCell<Vec<PendingFontFace>>,
}
impl ResourceFetcher {
    pub fn MemoryDiagnostics(&self) -> resource::ResourceMemoryDiagnostics {
        self.engine.MemoryDiagnostics()
    }
    pub fn WithEngine(
        engine: Rc<ResourceEngine>,
        document: Rc<RefCell<DOM>>,
        constraints: Rc<RefCell<ConstraintSpace>>,
    ) -> Self {
        let current_url = engine.DocumentURL();
        Self {
            style_sheet_receiver: RefCell::new(None),
            engine,
            document,
            constraints,
            base_url: RefCell::new(current_url.clone()),
            style_sheet_signatures: RefCell::new(HashMap::new()),
            pending_style_sheets: RefCell::new(Vec::new()),
            loaded_image_sources: RefCell::new(HashSet::new()),
            image_event_sources: RefCell::new(HashMap::new()),
            cached_image_event_sources: RefCell::new(HashSet::new()),
            image_event_subtrees: RefCell::new(None),
            referenced_image_owners: RefCell::new(None),
            pending_images: RefCell::new(Vec::new()),
            deferred_images: RefCell::new(Vec::new()),
            document_image_sources: RefCell::new(HashMap::new()),
            responsive_image_selector: RefCell::new(None),
            preload_image_selector: RefCell::new(None),
            preload_media_matcher: RefCell::new(None),
            pending_fonts: RefCell::new(Vec::new()),
        }
    }
    pub fn BaseURL(&self) -> String {
        self.base_url.borrow().clone()
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
        if changed {
            self.image_event_subtrees.borrow_mut().take();
        }
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
        self.SetBaseURL(url);
    }
    pub fn SetBaseURL(&self, base: String) {
        *self.base_url.borrow_mut() = base;
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
        self.CommitImage(source, image, client);
    }
    // cpp: browser/browser.cc:1046-1067
    fn CommitImage(
        &self,
        source: String,
        image: PaintImage,
        client: &mut dyn ResourceFetcherClient,
    ) {
        self.engine.ObserveImageId(image.id);
        client.ApplyResourceMutation(ResourceMutation::ImageResourceReady(ImageResourceReady {
            source,
            image,
        }));
    }
    // cpp: browser/browser.cc:1081-1102
    pub fn AddParsedStyleSheet(&self, sheet: CSSStyleSheet, base: &str) -> io::Result<()> {
        self.AddParsedStyleSheetToDocument(self.document.borrow_mut().GetDocumentMut(), sheet, base)
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
                self.QueueFontFace(rule, base)?;
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
            for i in 0..document.NodeCount() {
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
                    resource,
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
            if completed >= max_completions || !pending.resource.Poll() {
                self.pending_style_sheets.borrow_mut().insert(0, pending);
                continue;
            }
            completed += 1;
            let result = RequireResponse(pending.resource.TakeResult()).and_then(|response| {
                let base = if response.final_url.is_empty() {
                    pending.url.clone()
                } else {
                    response.final_url.clone()
                };
                let css = DecodeText(&response)?;
                let mut sheet = style::ParseCSSWithBase(&css, &base);
                sheet.SetLinkMetadata(
                    pending.owner_node_id,
                    pending.alternate,
                    false,
                    pending.title.clone(),
                    pending.media.clone(),
                );
                self.AddParsedStyleSheet(sheet, &base)
            });
            if let Err(error) = result {
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
        let Some(mut metadata) = crate::web_fonts::QueueFontFace(rule) else {
            return Ok(());
        };
        for url in &mut metadata.urls {
            *url = ResolveUrl(base, url)?;
        }
        if self.pending_fonts.borrow().iter().any(|f| {
            f.metadata.family.eq_ignore_ascii_case(&metadata.family)
                && f.metadata.weight == metadata.weight
                && f.metadata.italic == metadata.italic
                && f.metadata.urls == metadata.urls
                && f.metadata.unicode_ranges == metadata.unicode_ranges
        }) {
            return Ok(());
        }
        self.pending_fonts.borrow_mut().push(PendingFontFace {
            metadata,
            next_url: 0,
            loaded: false,
            // Fetch only after FontEngine reports actual text demand.
            selected: false,
            resource: None,
            decode: None,
        });
        Ok(())
    }
    // Register immutable face descriptors with FontEngine before LayoutText
    // and shaping report actual text demand. Keep dormant definitions resident
    // so later text/style mutations can activate them without another sheet.
    // cpp: core/css/font_face_set_document.cc; core/css/css_font_selector.cc
    pub fn PrepareFontFaces(&self) {
        let faces = self.pending_fonts.borrow();
        let metadata = faces
            .iter()
            .enumerate()
            .map(|(index, face)| font_engine::WebFontFace {
                rule_id: index as u64,
                family: face.metadata.family.clone(),
                weight: face.metadata.weight,
                italic: face.metadata.italic,
                unicode_ranges: face.metadata.unicode_ranges.clone(),
                loaded: face.loaded,
                failed: !face.loaded
                    && face.resource.is_none()
                    && face.decode.is_none()
                    && face.next_url == face.metadata.urls.len(),
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
            let mut faces = self.pending_fonts.borrow_mut();
            for id in requested {
                if let Some(face) = faces.get_mut(id as usize) {
                    face.selected = true;
                }
            }
        }
        for face in &mut *self.pending_fonts.borrow_mut() {
            if face.resource.is_some() || face.decode.is_some() || !face.selected {
                continue;
            }
            if face.loaded || face.next_url == face.metadata.urls.len() {
                continue;
            }
            face.resource = Some(self.engine.Start(URLRequest {
                url: face.metadata.urls[face.next_url].clone(),
                destination: RequestDestination::kFont,
                ..Default::default()
            }));
        }
    }
    // cpp: browser/browser.cc:1210-1239
    fn ApplyFontResult(
        &self,
        face: &mut PendingFontFace,
        decoded: io::Result<Vec<u8>>,
        client: &mut dyn ResourceFetcherClient,
    ) {
        let url = face.metadata.urls[face.next_url].clone();
        face.next_url += 1;
        let fail = |message: String, client: &mut dyn ResourceFetcherClient| {
            client.ApplyResourceMutation(ResourceMutation::ResourceLoadFailed(ResourceLoadFailed {
                kind: ResourceKind::kFont,
                source: None,
                url: url.clone(),
                error: message,
            }))
        };
        let decoded = decoded.and_then(font_engine::AcceptDecodedWebFont);
        match decoded {
            Ok(bytes) => {
                client.ApplyResourceMutation(ResourceMutation::FontResourceReady(
                    FontResourceReady {
                        font: FontFace {
                            family: face.metadata.family.clone(),
                            weight: face.metadata.weight,
                            italic: face.metadata.italic,
                            bytes: bytes.into(),
                            unicode_ranges: face.metadata.unicode_ranges.clone(),
                            ..Default::default()
                        },
                    },
                ));
                face.loaded = true;
            }
            Err(e) => fail(e.to_string(), client),
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
                    let resource = self.engine.Start(URLRequest {
                        url: image.url.clone(),
                        referrer_policy: image.referrer_policy,
                        destination: RequestDestination::kImage,
                        ..Default::default()
                    });
                    self.pending_images.borrow_mut().push(PendingImage {
                        source: image.source,
                        url: image.url,
                        resource,
                        decode: None,
                        is_multipart_response: None,
                        blocks_load: true,
                    });
                }
            }
            return Ok(());
        }
        if !blocks_load {
            self.deferred_images.borrow_mut().push(DeferredImage {
                source: source.into(),
                url,
                referrer_policy,
            });
            return Ok(());
        }
        let resource = self.engine.Start(URLRequest {
            url: url.clone(),
            referrer_policy,
            destination: RequestDestination::kImage,
            ..Default::default()
        });
        self.pending_images.borrow_mut().push(PendingImage {
            source: source.into(),
            url,
            resource,
            decode: None,
            is_multipart_response: None,
            blocks_load,
        });
        Ok(())
    }

    /// Admit a bounded number of deferred lazy requests after window load.
    /// The cap preserves transport capacity for scripts and other high-priority
    /// resources while still allowing below-the-fold content to make progress.
    pub fn StartDeferredImages(&self, max_active: usize) -> usize {
        let active = self.pending_images.borrow().len();
        let count = max_active
            .saturating_sub(active)
            .min(self.deferred_images.borrow().len());
        for _ in 0..count {
            let image = self.deferred_images.borrow_mut().remove(0);
            let resource = self.engine.Start(URLRequest {
                url: image.url.clone(),
                referrer_policy: image.referrer_policy,
                destination: RequestDestination::kImage,
                ..Default::default()
            });
            self.pending_images.borrow_mut().push(PendingImage {
                source: image.source,
                url: image.url,
                resource,
                decode: None,
                is_multipart_response: None,
                blocks_load: false,
            });
        }
        count
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
        ImageResultBoundary(|| {
            let mut container =
                ContainerKey::new(0, 0, 1.0, self.constraints.borrow().device_pixel_ratio);
            container.view_specification = SVGImageViewSpecification(&image.source).to_owned();
            self.engine
                .BeginImageDecode(Arc::from(response.body), &response.mime_type, &container)
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
            let (id, decoded, document_image) = match decoded_resource {
                DecodedImageResource::Document { resource_id, frame } => {
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
                } => {
                    // BitmapImage retains a concrete decoder/frame owner for
                    // animated formats before it publishes a PaintImage. This
                    // CPU decoder currently exposes only one immutable pixel
                    // buffer, so reject that individual resource at the image
                    // loading boundary. Letting incomplete animation facts
                    // enter PaintArtifact would turn a recoverable image
                    // decode failure into a terminal compositor/Page error.
                    //
                    // Source: third_party/blink/renderer/platform/graphics/
                    // bitmap_image.cc:155-166; image.cc:217-234.
                    if let image_resource::PaintImageAnimationMetadata::Unavailable(error) =
                        &bitmap.animation
                    {
                        return Err(io::Error::new(io::ErrorKind::Unsupported, error.clone()));
                    }
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
                            orientation: bitmap.orientation,
                            content: PaintImageContent::Bitmap(bitmap.rgba8.into()),
                            ..Default::default()
                        },
                        false,
                    )
                }
            };
            // Source catches validation while committing the decoded image,
            // as well as decoder failures. Keep the consumed id and continue
            // loading other resources after a rejected image.
            self.CommitImage(image.source.clone(), decoded, client);
            if document_image {
                self.document_image_sources
                    .borrow_mut()
                    .insert(id, image.source.clone());
            }
            self.DispatchImageEvents(&image.source, true, client);
            Ok(())
        });
        match decoded {
            Ok(()) => {}
            Err(e) => {
                client.ApplyResourceMutation(ResourceMutation::ResourceLoadFailed(
                    ResourceLoadFailed {
                        kind: ResourceKind::kImage,
                        source: Some(image.source.clone()),
                        url: image.url,
                        error: e.to_string(),
                    },
                ));
                self.DispatchImageEvents(&image.source, false, client);
            }
        }
    }
    /// A changed img request may revisit its previous URL before a queued DOM
    /// event runs. Deduplication must follow the request, not the URL forever.
    pub fn ResetImageEventSource(&self, id: u64) {
        self.image_event_sources.borrow_mut().remove(&id);
    }
    // cpp: browser/browser.cc:1293-1310
    pub fn DispatchImageEvent(
        &self,
        id: u64,
        loaded: bool,
        client: &mut dyn ResourceFetcherClient,
    ) {
        if !client.HasJavaScript() {
            return;
        }
        let source = {
            let owner = self.document.borrow();
            let d = owner.GetDocument();
            let Some(i) = d.FindNodeById(id) else { return };
            d.CurrentImageSource(i).unwrap_or("").to_owned()
        };
        if source.is_empty() || self.image_event_sources.borrow().get(&id) == Some(&source) {
            return;
        }
        self.image_event_sources.borrow_mut().insert(id, source);
        client.DispatchImageEvent(id, loaded);
    }
    // cpp: browser/browser.cc:1312-1319
    fn MayHaveImageEvent(&self, i: usize, source: &str) -> bool {
        let owner = self.document.borrow();
        let d = owner.GetDocument();
        let Some(key) = d.CollectionMembershipKey() else {
            // Revision exhaustion disables reuse, retaining the original walk.
            return true;
        };
        let mut cached = self.image_event_subtrees.borrow_mut();
        if cached.as_ref().is_none_or(|index| index.key != key) {
            let mut ancestors: HashMap<String, HashSet<usize>> = HashMap::new();
            for node in 0..d.NodeCount() {
                let n = d.Node(node);
                if !n.IsHTMLElement("img") {
                    continue;
                }
                let Some(source) = d.CurrentImageSource(node) else {
                    continue;
                };
                let nodes = ancestors.entry(source.to_owned()).or_default();
                let mut ancestor = Some(node);
                while let Some(a) = ancestor {
                    // An already indexed ancestor's entire path is present.
                    if !nodes.insert(a) {
                        break;
                    }
                    ancestor = d.Node(a).Parent().or_else(|| d.ShadowHost(a));
                }
            }
            *cached = Some(ImageEventSubtrees { key, ancestors });
        }
        cached
            .as_ref()
            .unwrap()
            .ancestors
            .get(source)
            .is_some_and(|nodes| nodes.contains(&i))
    }

    fn DispatchImageEvents(
        &self,
        source: &str,
        loaded: bool,
        client: &mut dyn ResourceFetcherClient,
    ) {
        if !client.HasJavaScript() {
            return;
        }
        fn visit(
            resources: &ResourceFetcher,
            i: usize,
            source: &str,
            loaded: bool,
            client: &mut dyn ResourceFetcherClient,
        ) {
            // Recheck after every possible listener/checkpoint mutation. Skip
            // only subtrees with no matching img; keep the original callback
            // order and the post-callback children snapshot below.
            if !resources.MayHaveImageEvent(i, source) {
                return;
            }
            let id = {
                let owner = resources.document.borrow();
                let d = owner.GetDocument();
                let n = d.Node(i);
                if n.IsHTMLElement("img")
                    && d.CurrentImageSource(i)
                        .is_some_and(|current| current == source)
                {
                    Some(n.Id())
                } else {
                    None
                }
            };
            if let Some(id) = id {
                resources.DispatchImageEvent(id, loaded, client);
            }
            let children = {
                let owner = resources.document.borrow();
                let document = owner.GetDocument();
                // Shadow-including order visits the shadow subtree before the
                // host's light children. Capture after the callback, as above.
                document
                    .ShadowRoot(i)
                    .into_iter()
                    .chain(document.Node(i).Children().iter().copied())
                    .collect::<Vec<_>>()
            };
            for c in children {
                visit(resources, c, source, loaded, client);
            }
        }
        let root = self.document.borrow().GetDocument().Root();
        visit(self, root, source, loaded, client);
        // HTMLImageElement starts fetching while detached (`new Image()` is
        // the common preload path), and Blink delivers its load/error event
        // without requiring insertion into the document. The rooted walk
        // above preserves shadow-including document order; finish with the
        // matching detached image owners in arena creation order.
        //
        // Source: third_party/blink/renderer/core/html/html_image_element.cc
        // ImageLoader::UpdatedHasPendingEvent / DispatchPendingEvent.
        let detached = {
            let owner = self.document.borrow();
            let document = owner.GetDocument();
            (0..document.NodeCount())
                .filter_map(|i| {
                    let node = document.Node(i);
                    (!document.IsConnectedToDocument(i)
                        && node.IsHTMLElement("img")
                        && document
                            .CurrentImageSource(i)
                            .is_some_and(|current| current == source))
                    .then_some(node.Id())
                })
                .collect::<Vec<_>>()
        };
        for id in detached {
            self.DispatchImageEvent(id, loaded, client);
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
            debug_assert!(n.IsHTMLElement("img"));
            let selection = if let Some(selector) = selector {
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
            if let Some(selection) = selection.as_ref() {
                if d.ImageResourceFor(&selection.source).is_none() {
                    let lazy = n
                        .FindAttribute("loading")
                        .is_some_and(|loading| loading.value.eq_ignore_ascii_case("lazy"));
                    let policy = n
                        .FindAttribute("referrerpolicy")
                        .and_then(|attribute| ReferrerPolicy::FromString(&attribute.value));
                    request(
                        &selection.source,
                        false,
                        !lazy,
                        policy,
                        admitted,
                        seen,
                        requests,
                    );
                }
            }
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
                    if d.Node(node).IsHTMLElement("img") {
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
                    if !d.IsConnectedToDocument(node) && d.Node(node).IsHTMLElement("img") {
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
        {
            let owner = self.document.borrow();
            let document = owner.GetDocument();
            let mut cached = self.cached_image_event_sources.borrow_mut();
            for (_, selection) in &selections {
                if let Some(selection) = selection {
                    if document.ImageResourceFor(&selection.source).is_some() {
                        cached.insert(selection.source.clone());
                    }
                }
            }
        }
        let selection_changed = {
            let mut owner = self.document.borrow_mut();
            let document = owner.GetDocumentMut();
            let mut selection_changed = false;
            for (node, selection) in selections {
                selection_changed |= document.SetResponsiveImageSelection(node, selection);
            }
            if selection_changed {
                self.image_event_subtrees.borrow_mut().take();
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
        Ok(selection_changed)
    }
    pub fn HasPendingImages(&self) -> bool {
        !self.pending_images.borrow().is_empty()
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
        mut container: ContainerKey,
        preferred_color_scheme: u8,
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
        container.view_specification = SVGImageViewSpecification(&source).to_owned();
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
            face.resource.is_some()
                || face.decode.is_some()
                || (face.selected && !face.loaded && face.next_url < face.metadata.urls.len())
        })
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
        let started = std::env::var_os("BROWSER_PROFILE_INPUT")
            .is_some()
            .then(std::time::Instant::now);
        let count = self.pending_images.borrow().len();
        let mut completed = 0;
        let mut admitted = 0;
        // A cache hit still queues a load event for each ImageLoader owner.
        // Discovery records sources without calling script; this ordinary
        // resource turn performs delivery with the same per-element dedupe as
        // a fresh decode completion.
        let cached_sources = {
            let mut pending = self.cached_image_event_sources.borrow_mut();
            let take = pending.len().min(max_completions);
            let sources = pending.iter().take(take).cloned().collect::<Vec<_>>();
            for source in &sources {
                pending.remove(source);
            }
            sources
        };
        for source in cached_sources {
            self.DispatchImageEvents(&source, true, client);
            completed += 1;
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
        if let Some(start) = started {
            if start.elapsed().as_millis() >= 50 {
                eprintln!(
                    "image-poll-profile initial={count} completed={completed} pending={} ms={:.3}",
                    self.pending_images.borrow().len(),
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
        let count = self.pending_fonts.borrow().len();
        let mut completed = 0;
        let mut admitted = 0;
        let mut index = 0;
        for _ in 0..count {
            if completed + admitted >= max_completions {
                break;
            }
            let mut face = {
                let mut faces = self.pending_fonts.borrow_mut();
                if index >= faces.len() {
                    break;
                }
                faces.remove(index)
            };
            let result = if let Some(job) = &mut face.decode {
                job.poll()
            } else if face
                .resource
                .as_mut()
                .is_some_and(|resource| resource.Poll())
            {
                let result = face.resource.take().unwrap().TakeResult();
                if let Some(response) = result.response {
                    let mut job = self.engine.GetDecodeEngine().BeginFontDecode(response.body);
                    let result = job.poll();
                    if result.is_none() {
                        admitted += 1;
                    }
                    face.decode = Some(job);
                    result
                } else {
                    Some(Err(io::Error::other(result.error)))
                }
            } else {
                None
            };
            if let Some(result) = result {
                face.decode = None;
                self.ApplyFontResult(&mut face, result, client);
                completed += 1;
            }
            self.pending_fonts.borrow_mut().insert(index, face);
            index += 1;
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
