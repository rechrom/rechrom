//! Canonical Document class and decoding metadata.
//! Chromium document.h:303-320,647-667,1550-1556; document_encoding_data.cc:38-43;
//! decoded_data_document_parser.cc:90-104; document.cc:1924-1926,7621-7640.
use crate::persistent_document::{DOMDocumentType, DOMNodeType, PersistentDocument};

/// The source class flags are queried independently of element namespaces or
/// MIME labels. The document constructor supplies HTML, XML or SVG explicitly;
/// an unsupported specialized document never acquires its class bit.
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum DocumentClass {
    HTML,
    XHTML,
    Image,
    Plugin,
    Media,
    SVG,
    XML,
    Text,
}
#[derive(Clone, Copy)]
pub struct DocumentClassFlags(u8);
impl DocumentClassFlags {
    pub fn Has(self, class: DocumentClass) -> bool {
        self.0 & (1 << class as u8) != 0
    }
}
/// Actual supported TextDecoder selections, not the unnormalized HTTP label.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DocumentTextEncoding {
    #[default]
    UTF8,
    Windows1252,
}
impl DocumentTextEncoding {
    pub fn Name(self) -> &'static str {
        match self {
            Self::UTF8 => "UTF-8",
            Self::Windows1252 => "windows-1252",
        }
    }
}
#[derive(Default)]
pub(crate) struct DocumentMetadata {
    pub(crate) frame_owner_loading: crate::frame_owner_loading::FrameOwnerLoadingDocument,
    pub(crate) runtime_features: crate::document_policy::DocumentRuntimeFeatureFacts,
    pub(crate) intersection_observer_controller:
        std::cell::OnceCell<std::rc::Rc<intersection_observer::IntersectionObserverController>>,
    pub(crate) font_loading: std::rc::Rc<crate::font_loading::FontLoadingDocument>,
    pub(crate) required_document_policy:
        std::rc::Rc<crate::document_policy::DocumentPolicyFeatureState>,
    pub(crate) iframe_document_policies: std::cell::RefCell<
        std::collections::HashMap<u64, crate::document_policy::IFrameDocumentPolicy>,
    >,
    pub(crate) document_policy_messages:
        std::cell::RefCell<Vec<crate::document_policy::DocumentPolicyMessage>>,
    // DocumentInit::is_initial_empty_document_ is false for regular documents.
    is_initial_empty_document: bool,
    deferred_compositor_commit_is_allowed: bool,
    xml_declaration: Option<XMLDeclaration>,
    encoding: DocumentTextEncoding,
    loader_attached: bool,
    has_dom_window: bool,
    loader_load_timing:
        Option<std::rc::Rc<crate::render_blocking_resource_manager::RenderBlockingLoadTiming>>,
    render_blocking_resources: Option<
        std::rc::Rc<
            std::cell::RefCell<
                crate::render_blocking_resource_manager::RenderBlockingResourceManager,
            >,
        >,
    >,
    pub(crate) render_blocking_mutations: std::cell::RefCell<
        std::collections::VecDeque<crate::render_blocking_elements::RenderBlockingDocumentMutation>,
    >,
    pub(crate) parsing_element_children: std::cell::RefCell<std::collections::HashSet<u64>>,
    has_render_blocking_expect_links: std::rc::Rc<std::cell::Cell<bool>>,
    render_blocking_body_presence: std::rc::Rc<std::cell::Cell<bool>>,
    rendering_has_begun: std::rc::Rc<std::cell::Cell<bool>>,
    content_language: Option<std::rc::Rc<[u16]>>,
}
/// Document's XML prologue fields, independent from decoded transport encoding.
/// Source document.h:580-598; xml_document_parser_rs.cc:319-353.
#[derive(Clone)]
pub struct XMLDeclaration {
    pub version: std::rc::Rc<[u16]>,
    pub encoding: std::rc::Rc<[u16]>,
    pub standalone: Option<bool>,
}
impl PersistentDocument {
    pub fn IsInitialEmptyDocument(&self) -> bool {
        self.document_metadata
            .get(&self.Node(self.Root()).Id())
            .is_some_and(|data| data.is_initial_empty_document)
    }
    pub fn SetIsInitialEmptyDocument(&mut self, value: bool) {
        let id = self.Node(self.Root()).Id();
        self.document_metadata
            .entry(id)
            .or_default()
            .is_initial_empty_document = value;
    }
    pub fn SetDeferredCompositorCommitIsAllowed(&mut self, value: bool) {
        let id = self.Node(self.Root()).Id();
        self.document_metadata
            .entry(id)
            .or_default()
            .deferred_compositor_commit_is_allowed = value;
    }
    pub fn DeferredCompositorCommitIsAllowed(&self) -> bool {
        // Document.cc:10087-10094: the actual document transition excludes holding.
        self.GetViewTransitionsIfExists()
            .is_none_or(|owner| !owner.HasDocumentTransition())
            && self
                .document_metadata
                .get(&self.Node(self.Root()).Id())
                .is_some_and(|data| data.deferred_compositor_commit_is_allowed)
    }
    pub fn HasFinishedParsing(&self) -> bool {
        self.ParsingState() == crate::document_parser_lifecycle::DocumentParsingState::Finished
    }
    pub fn DocumentClasses(&self, document: usize) -> DocumentClassFlags {
        DocumentClassFlags(match self.DocumentType(document) {
            DOMDocumentType::kHTML => 1 << DocumentClass::HTML as u8,
            DOMDocumentType::kXML => 1 << DocumentClass::XML as u8,
            // DocumentInit::CreateDocument cc:335-336 -> XMLDocument::CreateSVG.
            DOMDocumentType::kSVG => {
                (1 << DocumentClass::XML as u8) | (1 << DocumentClass::SVG as u8)
            }
        })
    }
    pub fn SetDocumentEncoding(&mut self, encoding: DocumentTextEncoding) {
        let id = self.Node(self.Root()).Id();
        self.document_metadata.entry(id).or_default().encoding = encoding;
    }
    pub fn EncodingName(&self) -> &'static str {
        self.document_metadata
            .get(&self.Node(self.Root()).Id())
            .map_or(DocumentTextEncoding::default(), |data| data.encoding)
            .Name()
    }
    /// DocumentLoader's attachment is a document lifecycle fact; the owning
    /// loader installs it when it binds the parser to this exact resident arena.
    pub fn AttachDocumentLoader(&mut self) {
        assert_eq!(self.Node(self.Root()).Type(), DOMNodeType::kDocument);
        let id = self.Node(self.Root()).Id();
        self.document_metadata
            .entry(id)
            .or_default()
            .loader_attached = true;
    }
    pub fn HasDocumentLoader(&self) -> bool {
        self.document_metadata
            .get(&self.Node(self.Root()).Id())
            .is_some_and(|data| data.loader_attached)
    }
}

impl PersistentDocument {
    pub fn SetXMLDeclaration(
        &mut self,
        version: &str,
        encoding: &str,
        standalone: Option<bool>,
        has_declaration: bool,
    ) {
        let id = self.Node(self.Root()).Id();
        self.document_metadata
            .entry(id)
            .or_default()
            .xml_declaration = has_declaration.then(|| XMLDeclaration {
            // Source silently processes XML 1.x as XML 1.0.
            version: "1.0".encode_utf16().collect::<Vec<_>>().into(),
            encoding: encoding.encode_utf16().collect::<Vec<_>>().into(),
            standalone,
        });
        let _ = version;
    }
    pub fn XMLDeclaration(&self) -> Option<&XMLDeclaration> {
        self.document_metadata
            .get(&self.Node(self.Root()).Id())?
            .xml_declaration
            .as_ref()
    }
}

impl PersistentDocument {
    /// Document.h ContentLanguage/SetContentLanguage, set by DocumentLoader
    /// from the first HTTP Content-Language value (loader.cc2312-2322).
    pub fn ContentLanguage(&self) -> std::rc::Rc<[u16]> {
        self.document_metadata
            .get(&self.Node(self.Root()).Id())
            .and_then(|m| m.content_language.clone())
            .unwrap_or_else(|| std::rc::Rc::from([]))
    }
    pub fn SetContentLanguage(&mut self, language: &[u16]) {
        let id = self.Node(self.Root()).Id();
        if self
            .document_metadata
            .get(&id)
            .and_then(|metadata| metadata.content_language.as_deref())
            == Some(language)
        {
            return;
        }
        self.document_metadata
            .entry(id)
            .or_default()
            .content_language = Some(std::rc::Rc::from(language));
        // Source Document.cc1928-1936: viewport/root inherited language and
        // every element style depend on this value. Use the existing full
        // document style/lifecycle producer, consumed at the next recalc.
        self.InvalidateAllStyles();
    }
}

impl PersistentDocument {
    /// Document.cc1055-1058. The loader binds its same timing owner after the
    /// concrete HTML/SVG document constructor has been selected.
    /// DocumentInit's actual LocalDOMWindow attachment, supplied by the Page
    /// constructor rather than inferred from a loader, URL or HTML root.
    pub fn HasDOMWindow(&self) -> bool {
        self.document_metadata
            .get(&self.Node(self.Root()).Id())
            .is_some_and(|m| m.has_dom_window)
    }
    pub fn UnbindDOMWindow(&mut self) {
        let id = self.Node(self.Root()).Id();
        self.document_metadata.entry(id).or_default().has_dom_window = false;
    }
    pub fn BindDOMWindow(&mut self) {
        let id = self.Node(self.Root()).Id();
        self.document_metadata.entry(id).or_default().has_dom_window = true;
    }
    pub fn DocumentLoaderLoadTiming(
        &self,
    ) -> Option<std::rc::Rc<crate::render_blocking_resource_manager::RenderBlockingLoadTiming>>
    {
        self.document_metadata
            .get(&self.Node(self.Root()).Id())
            .and_then(|m| m.loader_load_timing.clone())
    }
    pub fn BindRenderBlockingLoadTiming(
        &mut self,
        timing: std::rc::Rc<crate::render_blocking_resource_manager::RenderBlockingLoadTiming>,
    ) {
        self.document_metadata
            .entry(self.Node(self.Root()).Id())
            .or_default()
            .loader_load_timing = Some(timing.clone());
        if self.DocumentType(self.Root()) != DOMDocumentType::kHTML
            || !self
                .document_metadata
                .get(&self.Node(self.Root()).Id())
                .is_some_and(|m| m.has_dom_window)
        {
            return;
        }
        let lifecycle = self.Lifecycle().clone();
        let id = self.Node(self.Root()).Id();
        let metadata = self.document_metadata.entry(id).or_default();
        if let Some(manager) = &metadata.render_blocking_resources {
            manager.borrow_mut().SetLoadTiming(timing);
            return;
        }
        let body = metadata.render_blocking_body_presence.clone();
        metadata.render_blocking_resources = Some(std::rc::Rc::new(std::cell::RefCell::new(
            crate::render_blocking_resource_manager::RenderBlockingResourceManager::new(
                timing,
                metadata.has_render_blocking_expect_links.clone(),
                body,
                lifecycle,
                metadata.rendering_has_begun.clone(),
            ),
        )));
        self.SyncRenderBlockingBodyPresence();
    }
    pub(crate) fn SyncRenderBlockingBodyPresence(&self) {
        if let Some(metadata) = self.document_metadata.get(&self.Node(self.Root()).Id()) {
            metadata
                .render_blocking_body_presence
                .set(self.BodyElement().is_some());
        }
    }
    /// HTMLConstructionSite::InsertHTMLBodyElement, cc1051-1052.
    pub fn WillInsertBody(&self) {
        self.QueueRenderBlockingWillInsertBody();
    }
    pub fn GetRenderBlockingResourceManager(
        &self,
    ) -> Option<
        std::rc::Rc<
            std::cell::RefCell<
                crate::render_blocking_resource_manager::RenderBlockingResourceManager,
            >,
        >,
    > {
        self.document_metadata
            .get(&self.Node(self.Root()).Id())
            .and_then(|metadata| metadata.render_blocking_resources.clone())
    }
    /// Document.cc9106-9114. Font/preload/expect/script identities participate;
    /// the stylesheet queue is not a rendering readiness approximation.
    pub fn HaveRenderBlockingResourcesLoaded(&self) -> bool {
        self.GetRenderBlockingResourceManager()
            .is_none_or(|manager| !manager.borrow().HasRenderBlockingResources())
    }
    pub fn HaveRenderBlockingStylesheetsLoaded(&self) -> bool {
        self.GetRenderBlockingResourceManager()
            .is_none_or(|manager| !manager.borrow().HasPendingStylesheets())
    }
    pub fn HasRenderBlockingExpectLinkElements(&self) -> bool {
        self.document_metadata
            .get(&self.Node(self.Root()).Id())
            .is_some_and(|metadata| metadata.has_render_blocking_expect_links.get())
    }
    pub fn RenderingHasBegun(&self) -> bool {
        self.document_metadata
            .get(&self.Node(self.Root()).Id())
            .is_some_and(|m| m.rendering_has_begun.get())
    }
    pub fn TakeRenderBlockingFrameEffects(
        &self,
    ) -> std::collections::VecDeque<
        crate::render_blocking_resource_manager::RenderBlockingFrameEffect,
    > {
        self.GetRenderBlockingResourceManager()
            .map(|manager| manager.borrow_mut().TakeFrameEffects())
            .unwrap_or_default()
    }
    pub fn TakeRenderBlockingEffects(
        &self,
    ) -> std::collections::VecDeque<crate::render_blocking_resource_manager::RenderBlockingEffect>
    {
        self.GetRenderBlockingResourceManager()
            .map(|manager| manager.borrow_mut().TakeEffects())
            .unwrap_or_default()
    }
    pub fn NextRenderBlockingTimerDeadline(&self) -> Option<std::time::Instant> {
        self.GetRenderBlockingResourceManager()
            .and_then(|manager| manager.borrow().NextFontTimerDeadline())
    }
    pub fn FireDueRenderBlockingTimers(&self, now: std::time::Instant) {
        if let Some(manager) = self.GetRenderBlockingResourceManager() {
            manager.borrow_mut().FireDueFontTimers(now);
        }
    }
}
