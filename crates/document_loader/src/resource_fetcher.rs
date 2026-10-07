#![allow(non_snake_case)]
//! Source Page resource state. Page supplies mutation and event delivery, so
//! decoding and polling do not replace the source DOM/dirty/task lifecycle.
use crate::{
    resource_loader::{ResourceLoadResult, ResourceLoader, StartResource},
    url_reference::{ResolveCSSURLs, ResolveUrl},
};
use cssom::{CSSFontFaceRule, CSSStyleSheet};
use dom::{Document, DOM};
use image_decoder::image_decoder::{ImageDecodeInput, ImageDecoder};
use image_resource::{
    ContainerKey, DocumentImage, DocumentImageDecoder, DocumentImageEffect, DocumentImageMutation,
    PaintImageContent,
};
use layoutng_assembly::internal::layout_input::{
    ComputedStyle, ConstraintSpace, FontFace, PaintImage,
};
use page_mutation::{
    DocumentImageFrameChanged, DocumentImageIntrinsicSizeChanged, FontResourceReady,
    ImageResourceReady, ResourceKind, ResourceLoadFailed, ResourceMutation,
};
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    io,
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};
use url_loader::{RequestDestination, URLLoader, URLRequest};

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
    resource: Option<ResourceLoader>,
}
// cpp: browser/browser.cc:1241-1245
struct PendingImage {
    source: String,
    url: String,
    resource: ResourceLoader,
}

struct AnimatedImage {
    source: String,
    id: u64,
    first_frame_time: Option<Instant>,
    begin_frame_requested: bool,
    image: Box<dyn DocumentImage>,
}

struct ImageEventSubtrees {
    key: (dom::persistent_document::DOMOwnerHandle, u64),
    // Include detached trees: an earlier listener can detach a node that is
    // still present in the original traversal's already captured children.
    ancestors: HashMap<String, HashSet<usize>>,
}

// Rust separates the resource-owned fields of Page::Impl while retaining the
// same document and constraint handles, not serialized/copied DOM data.
// cpp: browser/browser.cc:1774-1798
pub struct ResourceFetcher {
    style_sheet_receiver: RefCell<Option<Rc<dyn Fn(&mut Document, CSSStyleSheet)>>>,
    loader: Rc<RefCell<dyn URLLoader>>,
    images: Rc<RefCell<dyn ImageDecoder>>,
    document_images: Rc<RefCell<dyn DocumentImageDecoder>>,
    document: Rc<RefCell<DOM>>,
    constraints: Rc<RefCell<ConstraintSpace>>,
    current_url: RefCell<String>,
    base_url: RefCell<String>,
    next_image_id: Cell<u64>,
    loaded_image_sources: RefCell<HashSet<String>>,
    image_event_sources: RefCell<HashMap<u64, String>>,
    image_event_subtrees: RefCell<Option<ImageEventSubtrees>>,
    pending_images: RefCell<Vec<PendingImage>>,
    animated_images: RefCell<Vec<AnimatedImage>>,
    loaded_font_urls: RefCell<HashSet<String>>,
    pending_fonts: RefCell<Vec<PendingFontFace>>,
}
impl ResourceFetcher {
    pub fn new(
        loader: Rc<RefCell<dyn URLLoader>>,
        images: Rc<RefCell<dyn ImageDecoder>>,
        document_images: Rc<RefCell<dyn DocumentImageDecoder>>,
        document: Rc<RefCell<DOM>>,
        constraints: Rc<RefCell<ConstraintSpace>>,
        current_url: String,
    ) -> Self {
        Self {
            style_sheet_receiver: RefCell::new(None),
            loader,
            images,
            document_images,
            document,
            constraints,
            base_url: RefCell::new(current_url.clone()),
            current_url: RefCell::new(current_url),
            next_image_id: Cell::new(1),
            loaded_image_sources: RefCell::new(HashSet::new()),
            image_event_sources: RefCell::new(HashMap::new()),
            image_event_subtrees: RefCell::new(None),
            pending_images: RefCell::new(Vec::new()),
            animated_images: RefCell::new(Vec::new()),
            loaded_font_urls: RefCell::new(HashSet::new()),
            pending_fonts: RefCell::new(Vec::new()),
        }
    }
    pub fn BaseURL(&self) -> String {
        self.base_url.borrow().clone()
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
        *self.current_url.borrow_mut() = url.clone();
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
        self.next_image_id
            .set(self.next_image_id.get().max(image.id.wrapping_add(1)));
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
        for rule in &sheet.font_faces {
            self.QueueFontFace(rule, base)?;
        }
        for rule in &mut sheet.rules {
            for declaration in &mut rule.declarations {
                ResolveCSSURLs(&mut declaration.value, base)?;
            }
        }
        self.CommitStyleSheet(d, sheet);
        Ok(())
    }

    pub fn OwnsDocument(&self, document: &Rc<RefCell<DOM>>) -> bool {
        Rc::ptr_eq(&self.document, document)
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
        }) {
            return Ok(());
        }
        self.pending_fonts.borrow_mut().push(PendingFontFace {
            metadata,
            next_url: 0,
            loaded: false,
            resource: None,
        });
        Ok(())
    }
    // cpp: browser/browser.cc:1174-1196
    pub fn DiscardUnusedFontFaces(&self) {
        if self.pending_fonts.borrow().is_empty() {
            return;
        }
        let mut used = HashSet::new();
        fn style(s: &ComputedStyle, used: &mut HashSet<String>) {
            if let Some(e) = &s.extended {
                for family in &e.font_families {
                    used.insert(family.to_ascii_lowercase());
                }
            }
        }
        fn node(d: &Document, i: usize, used: &mut HashSet<String>) {
            if let Some(r) = d.ResolvedStyleFor(i) {
                style(&r.style, used);
                for pseudo in [&r.before, &r.after, &r.first_letter].into_iter().flatten() {
                    style(&pseudo.style, used);
                }
            }
            for &c in d.Node(i).Children() {
                node(d, c, used);
            }
        }
        let owner = self.document.borrow();
        let d = owner.GetDocument();
        node(d, d.Root(), &mut used);
        self.pending_fonts
            .borrow_mut()
            .retain(|f| used.contains(&f.metadata.family.to_ascii_lowercase()));
    }
    // cpp: browser/browser.cc:1198-1208
    pub fn StartPendingFonts(&self) {
        for face in &mut *self.pending_fonts.borrow_mut() {
            if face.resource.is_some() {
                continue;
            }
            while !face.loaded
                && face.next_url < face.metadata.urls.len()
                && self
                    .loaded_font_urls
                    .borrow()
                    .contains(&face.metadata.urls[face.next_url])
            {
                face.next_url += 1;
            }
            if face.loaded || face.next_url == face.metadata.urls.len() {
                continue;
            }
            face.resource = Some(StartResource(
                &mut *self.loader.borrow_mut(),
                &URLRequest {
                    url: face.metadata.urls[face.next_url].clone(),
                    referrer: self.current_url.borrow().clone(),
                    destination: RequestDestination::kFont,
                    ..Default::default()
                },
            ));
        }
    }
    // cpp: browser/browser.cc:1210-1239
    fn ApplyFontResult(
        face: &mut PendingFontFace,
        result: ResourceLoadResult,
        loaded: &RefCell<HashSet<String>>,
        client: &mut dyn ResourceFetcherClient,
    ) {
        let url = face.metadata.urls[face.next_url].clone();
        face.next_url += 1;
        let fail = |message: String, client: &mut dyn ResourceFetcherClient| {
            client.ApplyResourceMutation(ResourceMutation::ResourceLoadFailed(ResourceLoadFailed {
                kind: ResourceKind::kFont,
                url: url.clone(),
                error: message,
            }))
        };
        let Some(response) = result.response else {
            fail(result.error, client);
            return;
        };
        let decoded = (|| -> io::Result<Vec<u8>> {
            if response.body.is_empty() {
                return Err(io::Error::other("empty font resource"));
            }
            let bytes = web_font::DecodeWebFont(response.body)?;
            // The owning native OpenType API maps its C++ constructor errors
            // to panic payloads. Preserve the source resource-failure boundary.
            let validated =
                std::panic::catch_unwind(|| font_engine::OpenTypeFont::new(&bytes, 0, 1.0, &[]));
            match validated {
                Ok(_) => Ok(bytes),
                Err(e) => {
                    let message = e
                        .downcast_ref::<String>()
                        .map(String::as_str)
                        .or_else(|| e.downcast_ref::<&str>().copied());
                    match message {
                        Some(
                            message @ ("invalid OpenType font request"
                            | "invalid OpenType font or face index"
                            | "font contains no glyphs"),
                        ) => Err(io::Error::other(message.to_owned())),
                        Some("HarfBuzz allocation failed") => {
                            Err(io::Error::other("std::bad_alloc"))
                        }
                        _ => std::panic::resume_unwind(e),
                    }
                }
            }
        })();
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
                loaded.borrow_mut().insert(url);
                face.loaded = true;
            }
            Err(e) => fail(e.to_string(), client),
        }
    }
    // cpp: browser/browser.cc:1247-1258
    pub fn QueueImage(&self, source: &str, resolved: Option<&str>) -> io::Result<()> {
        let url = match resolved {
            Some(url) => url.into(),
            None => ResolveUrl(&self.base_url.borrow(), source)?,
        };
        if !self.loaded_image_sources.borrow_mut().insert(source.into()) {
            return Ok(());
        }
        let resource = StartResource(
            &mut *self.loader.borrow_mut(),
            &URLRequest {
                url: url.clone(),
                referrer: self.current_url.borrow().clone(),
                destination: RequestDestination::kImage,
                ..Default::default()
            },
        );
        self.pending_images.borrow_mut().push(PendingImage {
            source: source.into(),
            url,
            resource,
        });
        Ok(())
    }
    // cpp: browser/browser.cc:1260-1291
    fn ApplyImageResult(
        &self,
        image: PendingImage,
        result: ResourceLoadResult,
        client: &mut dyn ResourceFetcherClient,
    ) {
        let Some(response) = result.response else {
            client.ApplyResourceMutation(ResourceMutation::ResourceLoadFailed(
                ResourceLoadFailed {
                    kind: ResourceKind::kImage,
                    url: image.url,
                    error: result.error,
                },
            ));
            self.DispatchImageEvents(&image.source, false, client);
            return;
        };
        let decoded = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
            || -> io::Result<_> {
                let profile =
                    std::env::var_os("BROWSER_PROFILE_INPUT").map(|_| std::time::Instant::now());
                let input = ImageDecodeInput {
                    bytes: &response.body,
                    mime_type: &response.mime_type,
                };
                if self.next_image_id.get() == 0 {
                    return Err(io::Error::other("image resource id space exhausted"));
                }
                let id = self.next_image_id.get();
                self.next_image_id
                    .set(self.next_image_id.get().wrapping_add(1));
                let is_document = self
                    .document_images
                    .borrow()
                    .can_decode(input.bytes, input.mime_type);
                let (decoded, retained_document) = if is_document {
                    let constraints = self.constraints.borrow();
                    let container = ContainerKey::new(
                        // The resource is decoded before any particular paint
                        // client has a resolved replaced-content box. Zero asks
                        // document_image to use the SVG intrinsic viewport;
                        // a later typed SetContainer mutation selects a
                        // client-specific responsive recording.
                        0,
                        0,
                        1.0,
                        constraints.device_pixel_ratio,
                    );
                    drop(constraints);
                    let created = self.document_images.borrow_mut().create(
                        id,
                        Arc::from(response.body.clone()),
                        &response.mime_type,
                        &container,
                    )?;
                    let begin_frame_requested = created
                        .effects
                        .iter()
                        .any(|effect| matches!(effect, DocumentImageEffect::RequestBeginFrame));
                    let frame = created.initial_frame;
                    let paint_image = PaintImage {
                        id,
                        revision: frame.revision,
                        width: frame.intrinsic_size.width,
                        height: frame.intrinsic_size.height,
                        content: PaintImageContent::Document(frame.record.clone()),
                        ..Default::default()
                    };
                    let retained =
                        begin_frame_requested.then_some((created.image, begin_frame_requested));
                    (paint_image, retained)
                } else {
                    let bitmap = self.images.borrow_mut().Decode(&input)?;
                    (
                        PaintImage {
                            id,
                            revision: 1,
                            width: bitmap.width,
                            height: bitmap.height,
                            content: PaintImageContent::Bitmap(bitmap.rgba8.into()),
                            ..Default::default()
                        },
                        None,
                    )
                };
                let decode_done = profile.map(|start| start.elapsed());
                // Source catches validation while committing the decoded image,
                // as well as decoder failures. Keep the consumed id and continue
                // loading other resources after a rejected image.
                self.CommitImage(image.source.clone(), decoded, client);
                if let Some((document, begin_frame_requested)) = retained_document {
                    self.animated_images.borrow_mut().push(AnimatedImage {
                        source: image.source.clone(),
                        id,
                        first_frame_time: None,
                        begin_frame_requested,
                        image: document,
                    });
                }
                let commit_done = profile.map(|start| start.elapsed());
                self.DispatchImageEvents(&image.source, true, client);
                if let Some(start) = profile {
                    if start.elapsed() > std::time::Duration::from_millis(5) {
                        eprintln!("image-completion-profile bytes={} mime={} decode_ms={:.3} commit_ms={:.3} events_ms={:.3}",
                            response.body.len(), response.mime_type,
                            decode_done.unwrap().as_secs_f64()*1000.0,
                            (commit_done.unwrap()-decode_done.unwrap()).as_secs_f64()*1000.0,
                            (start.elapsed()-commit_done.unwrap()).as_secs_f64()*1000.0);
                    }
                }
                Ok(())
            },
        ));
        // Source's resource exception boundary catches known engine errors.
        // Internal Rust invariants remain visible rather than being relabeled
        // as a broken network image and silently discarded.
        let decoded = match decoded {
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
                        layoutng_assembly::layout_engine::LayoutEngineError::RuntimeError(
                            message,
                        ) => Err(io::Error::other(*message)),
                    }
                } else {
                    std::panic::resume_unwind(error)
                }
            }
        };
        match decoded {
            Ok(()) => {}
            Err(e) => {
                client.ApplyResourceMutation(ResourceMutation::ResourceLoadFailed(
                    ResourceLoadFailed {
                        kind: ResourceKind::kImage,
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
            d.Node(i)
                .FindAttribute("src")
                .map_or(String::new(), |a| a.value.clone())
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
                let Some(attribute) = n.FindAttribute("src") else {
                    continue;
                };
                if attribute.value.is_empty() {
                    continue;
                }
                let nodes = ancestors.entry(attribute.value.clone()).or_default();
                let mut ancestor = Some(node);
                while let Some(a) = ancestor {
                    // An already indexed ancestor's entire path is present.
                    if !nodes.insert(a) {
                        break;
                    }
                    ancestor = d.Node(a).Parent();
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
                    && n.FindAttribute("src").is_some_and(|a| a.value == source)
                {
                    Some(n.Id())
                } else {
                    None
                }
            };
            if let Some(id) = id {
                resources.DispatchImageEvent(id, loaded, client);
            }
            let children = resources
                .document
                .borrow()
                .GetDocument()
                .Node(i)
                .Children()
                .to_vec();
            for c in children {
                visit(resources, c, source, loaded, client);
            }
        }
        let root = self.document.borrow().GetDocument().Root();
        visit(self, root, source, loaded, client);
    }

    // cpp: browser/browser.cc:1359-1393
    pub fn QueueReferencedImages(&self) -> io::Result<()> {
        let mut requests = Vec::new();
        fn request<'a>(
            source: &'a str,
            resolve: bool,
            admitted: &HashSet<String>,
            seen: &mut HashSet<&'a str>,
            requests: &mut Vec<(String, bool)>,
        ) {
            // QueueImage uses this exact source key for admission. Keep the
            // first occurrence, without cloning/resolving every repeated owner.
            // CSS sources already carry their stylesheet base via ResolveCSSURLs;
            // distinct DOM/CSS source strings remain distinct requests.
            if !admitted.contains(source) && seen.insert(source) {
                requests.push((source.to_owned(), resolve));
            }
        }
        fn style<'a>(
            s: &'a ComputedStyle,
            admitted: &HashSet<String>,
            seen: &mut HashSet<&'a str>,
            requests: &mut Vec<(String, bool)>,
        ) {
            let mut layer =
                |l: &'a layoutng_assembly::internal::paint_input::BackgroundImageLayer| {
                    if l.shader.is_none() && l.resource_id == 0 && !l.source_url.is_empty() {
                        request(&l.source_url, true, admitted, seen, requests);
                    }
                };
            for l in &s.paint.background_images {
                layer(l);
            }
            for m in &s.paint.mask_images {
                layer(&m.image);
            }
        }
        fn node<'a>(
            d: &'a Document,
            i: usize,
            admitted: &HashSet<String>,
            seen: &mut HashSet<&'a str>,
            requests: &mut Vec<(String, bool)>,
        ) {
            let n = d.Node(i);
            if n.IsHTMLElement("img") {
                if let Some(a) = n.FindAttribute("src") {
                    if !a.value.is_empty() && d.ImageResourceFor(&a.value).is_none() {
                        request(&a.value, false, admitted, seen, requests);
                    }
                }
            }
            if let Some(r) = d.ResolvedStyleFor(i) {
                if r.generates_box {
                    style(&r.style, admitted, seen, requests);
                }
                for p in [&r.before, &r.after].into_iter().flatten() {
                    if !p.display_contents {
                        style(&p.style, admitted, seen, requests);
                    }
                }
                if let Some(p) = &r.first_letter {
                    style(&p.style, admitted, seen, requests);
                }
            }
            for &c in n.Children() {
                node(d, c, admitted, seen, requests);
            }
        }
        {
            let owner = self.document.borrow();
            let d = owner.GetDocument();
            let admitted = self.loaded_image_sources.borrow();
            let mut seen = HashSet::new();
            node(d, d.Root(), &admitted, &mut seen, &mut requests);
        }
        for (source, resolve) in requests {
            let url = if resolve {
                Some(ResolveUrl(&self.base_url.borrow(), &source)?)
            } else {
                None
            };
            self.QueueImage(&source, url.as_deref())?;
        }
        Ok(())
    }
    pub fn HasPendingImages(&self) -> bool {
        !self.pending_images.borrow().is_empty()
    }
    pub fn HasAnimatedImages(&self) -> bool {
        self.animated_images
            .borrow()
            .iter()
            .any(|image| image.begin_frame_requested)
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
        for image in &mut *self.animated_images.borrow_mut() {
            if !image.begin_frame_requested {
                continue;
            }
            image.begin_frame_requested = false;
            let origin = *image.first_frame_time.get_or_insert(frame_time);
            let elapsed = frame_time.saturating_duration_since(origin);
            for effect in image
                .image
                .apply_mutation(DocumentImageMutation::AdvanceTimeline {
                    frame_time: elapsed,
                    begin_frame_sequence,
                })?
            {
                match effect {
                    DocumentImageEffect::FrameChanged { frame, .. } => {
                        if frame.resource_id != image.id {
                            return Err(io::Error::other(
                                "document image changed resource identity",
                            ));
                        }
                        client.ApplyResourceMutation(ResourceMutation::DocumentImageFrameChanged(
                            DocumentImageFrameChanged {
                                source: image.source.clone(),
                                frame,
                            },
                        ));
                        sampled += 1;
                    }
                    DocumentImageEffect::IntrinsicSizeChanged {
                        resource_id,
                        revision,
                        size,
                        ..
                    } => {
                        client.ApplyResourceMutation(
                            ResourceMutation::DocumentImageIntrinsicSizeChanged(
                                DocumentImageIntrinsicSizeChanged {
                                    source: image.source.clone(),
                                    resource_id,
                                    revision,
                                    size,
                                },
                            ),
                        );
                    }
                    DocumentImageEffect::RequestBeginFrame => {
                        image.begin_frame_requested = true;
                    }
                }
            }
        }
        Ok(sampled)
    }
    pub fn HasPendingFonts(&self) -> bool {
        self.pending_fonts.borrow().iter().any(|face| {
            face.resource.is_some() || (!face.loaded && face.next_url < face.metadata.urls.len())
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
        let mut index = 0;
        for _ in 0..count {
            if completed >= max_completions {
                break;
            }
            let image = {
                let mut images = self.pending_images.borrow_mut();
                if index >= images.len() {
                    break;
                }
                if !images[index].resource.Poll() {
                    index += 1;
                    continue;
                }
                let mut image = images.remove(index);
                let result = image.resource.TakeResult();
                (image, result)
            };
            self.ApplyImageResult(image.0, image.1, client);
            completed += 1;
        }
        if std::env::var_os("BROWSER_TRACE_LOADING_STATE").is_some() {
            eprintln!(
                "image-loading-state completed={completed} pending={:?}",
                self.pending_images
                    .borrow()
                    .iter()
                    .map(|image| image.url.as_str())
                    .collect::<Vec<_>>()
            );
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
        let mut index = 0;
        for _ in 0..count {
            if completed >= max_completions {
                break;
            }
            let face = {
                let mut faces = self.pending_fonts.borrow_mut();
                if index >= faces.len() {
                    break;
                }
                let Some(resource) = faces[index].resource.as_mut() else {
                    index += 1;
                    continue;
                };
                if !resource.Poll() {
                    index += 1;
                    continue;
                }
                let result = resource.TakeResult();
                let mut face = faces.remove(index);
                face.resource = None;
                (face, result)
            };
            let mut face = face;
            Self::ApplyFontResult(&mut face.0, face.1, &self.loaded_font_urls, client);
            self.pending_fonts.borrow_mut().insert(index, face.0);
            index += 1;
            completed += 1;
        }
        if std::env::var_os("BROWSER_TRACE_LOADING_STATE").is_some() {
            eprintln!(
                "font-loading-state completed={completed} pending={}",
                self.pending_fonts
                    .borrow()
                    .iter()
                    .filter(|face| face.resource.is_some()
                        || (!face.loaded && face.next_url < face.metadata.urls.len()))
                    .count()
            );
        }
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
        self.pending_fonts.borrow_mut().clear();
    }
}
