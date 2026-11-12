//! CSSFontFace/RemoteFontFaceSource loading over the canonical FontResource.
//! Chromium css_font_face.cc:242-299, remote_font_face_source.cc, FontFace
//! construction cc164-245,956-1033. Font selection descriptors retain the
//! original CSSValue owners; transport and OTS decode are never duplicated.
use super::*;
use crate::font_resource::{FontResource, PendingLinkPreload};
use dom::font_loading::{
    FontFace as DOMFontFace, FontFaceError, FontFaceLoadSource, FontFaceLoadStatus,
    FontLoadingDocument,
};
use foundation::content_security_policy::{CSPRequestContext, CSPWorld};
use foundation::{AtomicString, String as BlinkString, StringView};
use style_parser::{
    at_rule_descriptors::{AsAtRuleDescriptorID, AtRuleDescriptorID as D},
    css_parser_context::{CSSParserContext, SecureContextMode},
    css_parser_mode::CSSParserMode,
    css_value::CSSValuePayload as V,
    font_face_descriptors::{CompiledFontFaceDescriptors, ParseFontFaceDescriptor},
    production_css_value::{self as values, FontFaceResource},
    production_property_parser::ProductionParserPlatform,
};
pub(super) struct FontFaceTransportMetadata {
    pub family: String,
    pub urls: Vec<String>,
    pub weight: f64,
    pub italic: bool,
    pub font_feature_settings: Vec<font_engine::FontFeatureValue>,
    pub unicode_ranges: Vec<font_engine::FontUnicodeRange>,
}
#[derive(Clone)]
pub(super) enum FaceSource {
    Local(AtomicString),
    Remote(String),
}
#[derive(Default)]
pub(super) struct FontFetchContext {
    pub(super) security:
        RefCell<Option<Rc<dyn Fn(&Document) -> Arc<fetch_policy::RawFetchSecurityContext>>>>,
    resources: RefCell<HashMap<u64, Rc<FontResource>>>,
    pub(super) local_resolver: RefCell<Option<Rc<dyn Fn(&str) -> io::Result<Option<FontFace>>>>>,
    pub(super) preloads: RefCell<HashMap<u64, (String, Rc<PendingLinkPreload>)>>,
    pub(super) ready: RefCell<Vec<ResourceMutation>>,
}
pub enum FontFaceSourceInput {
    CSS(BlinkString),
    Binary(Vec<u8>),
}
fn ParserContext(base: &str, secure: bool) -> CSSParserContext<ProductionParserPlatform> {
    CSSParserContext::new(
        values::ProductionURL::FromString(base),
        true,
        None,
        CSSParserMode::kCSSFontFaceRuleMode,
        BlinkString::from(""),
        false,
        if secure {
            SecureContextMode::kSecureContext
        } else {
            SecureContextMode::kInsecureContext
        },
        None,
        None,
        style_parser::css_resource_fetch_restriction::ResourceFetchRestriction::kNone,
    )
}
fn FirstNumber(value: &values::Value) -> Option<f64> {
    match value.Payload() {
        V::kNumericLiteralClass(n) => Some(n.DoubleValue()),
        V::kValueListClass(list) => list.values.first().and_then(|v| FirstNumber(v)),
        _ => None,
    }
}
fn Metadata(face: &Rc<DOMFontFace>, sources: &[FaceSource]) -> FontFaceTransportMetadata {
    let weight = face
        .Descriptor(D::FontWeight)
        .as_ref()
        .and_then(|value| FirstNumber(value))
        .unwrap_or_else(
            || match face.Descriptor(D::FontWeight).as_ref().map(|v| v.Payload()) {
                Some(V::kIdentifierClass(v)) if v.0 == foundation::CSSValueID::kBold => 700.0,
                _ => 400.0,
            },
        );
    let italic=face.Descriptor(D::FontStyle).is_some_and(|v|!matches!(v.Payload(),V::kIdentifierClass(v)if matches!(v.0,foundation::CSSValueID::kNormal|foundation::CSSValueID::kAuto)));
    let mut ranges = Vec::new();
    if let Some(value) = face.Descriptor(D::UnicodeRange) {
        if let V::kValueListClass(list) = value.Payload() {
            for value in &list.values {
                if let V::kUnicodeRangeClass(range) = value.Payload() {
                    ranges.push(font_engine::FontUnicodeRange {
                        start: range.from,
                        end: range.to,
                    });
                }
            }
        }
    }
    let mut features = Vec::new();
    if let Some(value) = face.Descriptor(D::FontFeatureSettings) {
        if let V::kValueListClass(list) = value.Payload() {
            for value in &list.values {
                if let V::kFontFeatureClass(feature) = value.Payload() {
                    if let Some(value) = FirstNumber(&feature.value) {
                        let bytes = feature.tag.Ascii().into_bytes();
                        if bytes.len() == 4 {
                            features.push(font_engine::FontFeatureValue {
                                tag: u32::from_be_bytes(bytes.try_into().unwrap()),
                                value: value as u32,
                            });
                        }
                    }
                }
            }
        }
    }
    FontFaceTransportMetadata {
        family: face.Family().Utf8(),
        urls: sources
            .iter()
            .map(|source| match source {
                FaceSource::Remote(url) => url.clone(),
                FaceSource::Local(name) => name.Utf8(),
            })
            .collect(),
        weight,
        italic,
        font_feature_settings: features,
        unicode_ranges: ranges,
    }
}
fn Sources(face: &Rc<DOMFontFace>) -> Vec<FaceSource> {
    let Some(value) = face.Descriptor(D::Src) else {
        return Vec::new();
    };
    let V::kValueListClass(list) = value.Payload() else {
        return Vec::new();
    };
    list.values
        .iter()
        .filter_map(|value| {
            let V::kFontFaceSrcClass(src) = value.Payload() else {
                return None;
            };
            Some(match &src.resource {
                FontFaceResource::Local(name) => FaceSource::Local(AtomicString::from_utf16(
                    &(0..name.length())
                        .map(|i| name.CodeUnitAt(i as u32))
                        .collect::<Vec<_>>(),
                )),
                FontFaceResource::Url(url) => FaceSource::Remote(url.resolved_url.Utf8()),
            })
        })
        .collect()
}
struct LoadSource<'a>(&'a ResourceFetcher);
impl FontFaceLoadSource for LoadSource<'_> {
    fn Load(&self, face: &Rc<DOMFontFace>) {
        self.0.StartFontFace(face);
    }
}
impl ResourceFetcher {
    pub fn SetFontFetchSecuritySource(
        &self,
        security: Rc<dyn Fn(&Document) -> Arc<fetch_policy::RawFetchSecurityContext>>,
    ) {
        *self.font_loading.security.borrow_mut() = Some(security);
    }
    pub fn FontLoadingDocument(&self) -> Rc<FontLoadingDocument> {
        self.document.borrow().GetDocument().FontLoading()
    }
    pub fn SetLocalFontResolver(&self, resolver: Rc<dyn Fn(&str) -> io::Result<Option<FontFace>>>) {
        *self.font_loading.local_resolver.borrow_mut() = Some(resolver);
    }
    pub fn CreateFontFace(
        &self,
        family: AtomicString,
        source: FontFaceSourceInput,
        descriptors: &[(D, BlinkString)],
        world: CSPWorld,
    ) -> Rc<DOMFontFace> {
        let (context, parser) = {
            let document = self.document.borrow();
            let document = document.GetDocument();
            (
                document.FontLoading(),
                ParserContext(
                    &self.BaseURL(),
                    document.SecurityContext().IsSecureContext(),
                ),
            )
        };
        let face =
            DOMFontFace::InitializeFromStrings(&context, family, descriptors, world, &parser);
        match source {
            FontFaceSourceInput::CSS(text) => {
                face.InitializeSource(&text, &parser);
                self.RegisterPendingFontFace(face.clone(), false, None);
            }
            FontFaceSourceInput::Binary(bytes) => {
                let result = self
                    .engine
                    .GetDecodeEngine()
                    .GetWebFontDecoder()
                    .Decode(bytes)
                    .and_then(font_engine::AcceptDecodedWebFont);
                let mut pending = self.MakePendingFontFace(face.clone(), Vec::new());
                match result {
                    Ok(bytes) => {
                        pending.decoded = Some(bytes.into());
                        face.SetLoadStatus(FontFaceLoadStatus::Loaded);
                    }
                    Err(error) => face.SetError(Some(FontFaceError {
                        name: "SyntaxError",
                        message: BlinkString::from(format!(
                            "Invalid font data in ArrayBuffer. {}",
                            error
                        )),
                    })),
                }
                self.pending_fonts.borrow_mut().push(pending);
            }
        }
        face
    }
    pub fn ImperativeLoadFontFace(&self, face: &Rc<DOMFontFace>) {
        self.document.borrow().GetDocument().FontLoading();
        face.Load(&LoadSource(self));
    }
    pub fn LoadFontFaceWithCallback(
        &self,
        face: &Rc<DOMFontFace>,
        callback: Rc<dyn dom::font_loading::LoadFontCallback>,
    ) {
        face.LoadWithCallback(&LoadSource(self), callback);
    }
    pub fn LoadFontFaceArray(
        &self,
        faces: Vec<Rc<DOMFontFace>>,
        promise: Rc<dyn dom::font_loading::FontFaceArrayPromise>,
    ) {
        dom::font_loading::LoadFontPromiseResolver::new(faces, promise)
            .LoadFonts(&LoadSource(self));
    }
    pub fn SetFontFaceDescriptor(
        &self,
        face: &Rc<DOMFontFace>,
        name: &str,
        value: &BlinkString,
    ) -> Result<(), FontFaceError> {
        let id = dom::font_loading::FONT_FACE_DESCRIPTORS
            .iter()
            .find(|(_, key, _)| *key == name)
            .map(|(id, _, _)| *id)
            .expect("IDL FontFace descriptor");
        let document = self.document.borrow();
        let parser = ParserContext(
            &self.BaseURL(),
            document.GetDocument().SecurityContext().IsSecureContext(),
        );
        face.SetDescriptorFromString(id, value, &parser)
    }
    fn MakePendingFontFace(
        &self,
        face: Rc<DOMFontFace>,
        sources: Vec<FaceSource>,
    ) -> PendingFontFace {
        PendingFontFace {
            metadata: Metadata(&face, &sources),
            owner: face,
            sources,
            next_url: 0,
            selected: false,
            resource: None,
            decoded: None,
            published: false,
            dirty: true,
            local_face: None,
            compiled: None,
        }
    }
    fn RegisterPendingFontFace(
        &self,
        face: Rc<DOMFontFace>,
        css_connected: bool,
        context: Option<&Rc<FontLoadingDocument>>,
    ) {
        let sources = Sources(&face);
        let pending = self.MakePendingFontFace(face.clone(), sources);
        if css_connected {
            context
                .expect("CSS-connected font face retains its parser Document context")
                .FontFaceSet()
                .AddCSSConnected(face);
        }
        self.pending_fonts.borrow_mut().push(pending);
    }
    pub(super) fn QueueCompiledFontFace(
        &self,
        rule: &CSSFontFaceRule,
        base: &str,
    ) -> io::Result<()> {
        let document = self.document.borrow();
        self.QueueCompiledFontFaceInDocument(rule, base, document.GetDocument())
    }
    pub(super) fn QueueCompiledFontFaceInDocument(
        &self,
        rule: &CSSFontFaceRule,
        base: &str,
        document: &Document,
    ) -> io::Result<()> {
        let context = document.FontLoading();
        let compiled = if let Some(compiled) = rule
            .compiled_descriptors
            .as_ref()
            .and_then(|owner| owner.Get::<CompiledFontFaceDescriptors>())
        {
            compiled
        } else {
            // Raw CSSOM/test construction is initial parse input. Production
            // Style sheets always carry the compiled owner above.
            let secure = document.SecurityContext().IsSecureContext();
            let parser = ParserContext(base, secure);
            let mut compiled = Vec::new();
            for declaration in &rule.declarations {
                let id = AsAtRuleDescriptorID(StringView::from(declaration.property.as_str()));
                if let Ok(value) = ParseFontFaceDescriptor(
                    id,
                    &BlinkString::from(declaration.value.as_str()),
                    &parser,
                ) {
                    compiled.push((id, value));
                }
            }
            Rc::new(CompiledFontFaceDescriptors(compiled))
        };
        let Some(face) = DOMFontFace::FromCompiled(&context, &compiled) else {
            return Ok(());
        };
        // CSSFontFaceCache tracks the original style rule identity. The raw
        // CSSOM facade's cloned compiled owner preserves that same identity.
        if self.pending_fonts.borrow().iter().any(|pending| {
            pending
                .compiled
                .as_ref()
                .is_some_and(|owner| Rc::ptr_eq(owner, &compiled))
        }) {
            return Ok(());
        }
        self.RegisterPendingFontFace(face.clone(), true, Some(&context));
        self.pending_fonts.borrow_mut().last_mut().unwrap().compiled = Some(compiled);
        Ok(())
    }
    fn FontResource(
        &self,
        mut request: URLRequest,
        world: CSPWorld,
        mode: url_loader::FetchMode,
        credentials: url_loader::FetchCredentialsMode,
    ) -> io::Result<Rc<FontResource>> {
        let document = self.document.borrow();
        self.FontResourceInDocument(
            request,
            world,
            mode,
            credentials,
            document.GetDocument(),
            resource::ResourceLoadEventPolicy::Regular,
        )
    }
    fn FontResourceInDocument(
        &self,
        mut request: URLRequest,
        world: CSPWorld,
        mode: url_loader::FetchMode,
        credentials: url_loader::FetchCredentialsMode,
        document: &Document,
        policy: resource::ResourceLoadEventPolicy,
    ) -> io::Result<Rc<FontResource>> {
        let origin = document.SecurityContext().Origin();
        let origin = match origin.Tuple() {
            Some(tuple) => url_loader::FetchOrigin::Tuple {
                scheme: tuple.scheme.clone(),
                host: tuple.host.clone(),
                port: tuple.port,
            },
            None => {
                url_loader::FetchOrigin::Opaque(origin.OpaqueNonce().expect("source opaque origin"))
            }
        };
        let security = self.font_loading.security.borrow().clone().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Unsupported,
                "font fetch requires the canonical frame security owner",
            )
        })?;
        let security = security(document);
        self.ConfigureSubresourceNetworkRequest(
            &mut request,
            security,
            mode,
            credentials,
            CSPRequestContext::Font,
            origin,
            world,
        )
        .map_err(io::Error::other)?;
        let loader = self.engine.StartWithLoadEventPolicy(request, policy);
        let identity = loader.Identity();
        if let Some(resource) = self.font_loading.resources.borrow().get(&identity).cloned() {
            return Ok(resource);
        }
        let resource = FontResource::new(self.engine.clone(), loader, &document.FontLoading());
        self.font_loading
            .resources
            .borrow_mut()
            .insert(identity, resource.clone());
        Ok(resource)
    }
    fn StartFontFace(&self, owner: &Rc<DOMFontFace>) {
        if owner.LoadStatus() == FontFaceLoadStatus::Unloaded {
            owner.SetLoadStatus(FontFaceLoadStatus::Loading);
            let set = owner
                .Context()
                .expect("original font document")
                .FontFaceSet();
            if set.Has(owner) {
                set.BeginFontLoading(owner.clone());
            }
        }
        let index = self
            .pending_fonts
            .borrow()
            .iter()
            .position(|pending| Rc::ptr_eq(&pending.owner, owner));
        let Some(index) = index else {
            owner.SetError(None);
            return;
        };
        self.pending_fonts.borrow_mut()[index].selected = true;
        self.StartFontAt(index);
    }
    pub(super) fn StartFontAt(&self, index: usize) {
        loop {
            let (owner, source) = {
                let pending = self.pending_fonts.borrow();
                let face = &pending[index];
                if !face.selected
                    || face.resource.is_some()
                    || matches!(
                        face.owner.LoadStatus(),
                        FontFaceLoadStatus::Loaded | FontFaceLoadStatus::Error
                    )
                {
                    return;
                }
                (face.owner.clone(), face.sources.get(face.next_url).cloned())
            };
            if owner.LoadStatus() == FontFaceLoadStatus::Unloaded {
                owner.SetLoadStatus(FontFaceLoadStatus::Loading);
                let set = owner
                    .Context()
                    .expect("original font document")
                    .FontFaceSet();
                if set.Has(&owner) {
                    set.BeginFontLoading(owner.clone());
                }
            }
            let Some(source) = source else {
                owner.SetError(None);
                return;
            };
            match source {
                FaceSource::Remote(url) => {
                    match self.FontResource(
                        URLRequest {
                            url,
                            destination: RequestDestination::kFont,
                            ..Default::default()
                        },
                        owner.World(),
                        url_loader::FetchMode::Cors,
                        url_loader::FetchCredentialsMode::SameOrigin,
                    ) {
                        Ok(resource) => {
                            resource.RequestDecode();
                            let ready = resource.Decoded();
                            self.pending_fonts.borrow_mut()[index].resource = Some(resource);
                            if let Some(result) = ready {
                                self.FinishFontAt(index, result);
                            }
                            return;
                        }
                        Err(_) => self.pending_fonts.borrow_mut()[index].next_url += 1,
                    }
                }
                FaceSource::Local(name) => {
                    let resolver = self.font_loading.local_resolver.borrow().clone();
                    let result = resolver
                        .ok_or_else(|| {
                            io::Error::new(
                                io::ErrorKind::Unsupported,
                                "local font requires the canonical system font provider",
                            )
                        })
                        .and_then(|resolver| resolver(&name.Utf8()));
                    match result {
                        Ok(Some(font)) => {
                            {
                                let mut pending = self.pending_fonts.borrow_mut();
                                pending[index].decoded = Some(font.bytes.clone());
                                pending[index].local_face = Some(font);
                            }
                            owner.SetLoadStatus(FontFaceLoadStatus::Loaded);
                            return;
                        }
                        Ok(None) => self.pending_fonts.borrow_mut()[index].next_url += 1,
                        Err(error) => {
                            owner.SetError(Some(FontFaceError {
                                name: "NetworkError",
                                message: BlinkString::from(error.to_string()),
                            }));
                            return;
                        }
                    }
                }
            }
        }
    }
    pub(super) fn FinishFontAt(
        &self,
        index: usize,
        result: Result<font_engine::SharedFontBytes, String>,
    ) {
        let owner = {
            let mut pending = self.pending_fonts.borrow_mut();
            let face = &mut pending[index];
            face.resource = None;
            face.next_url += 1;
            match result {
                Ok(bytes) => {
                    face.decoded = Some(bytes);
                    Some(face.owner.clone())
                }
                Err(_) => None,
            }
        };
        if let Some(owner) = owner {
            owner.SetLoadStatus(FontFaceLoadStatus::Loaded);
        } else {
            let terminal = {
                let pending = self.pending_fonts.borrow();
                let face = &pending[index];
                (face.next_url == face.sources.len()).then(|| face.owner.clone())
            };
            if let Some(owner) = terminal {
                owner.SetError(None);
            }
        }
    }
    pub fn TakeFontResourceMutations(&self) -> Vec<ResourceMutation> {
        std::mem::take(&mut *self.font_loading.ready.borrow_mut())
    }
    pub fn FontFaceMembershipChanged(&self, face: &Rc<DOMFontFace>) {
        let mut pending = self.pending_fonts.borrow_mut();
        if let Some(pending) = pending
            .iter_mut()
            .find(|pending| Rc::ptr_eq(&pending.owner, face))
        {
            pending.metadata = Metadata(face, &pending.sources);
            pending.dirty = true;
        }
    }
}

impl ResourceFetcher {
    pub(super) fn FontResourcesSnapshot(&self) -> Vec<Rc<FontResource>> {
        self.font_loading
            .resources
            .borrow()
            .values()
            .cloned()
            .collect()
    }
    pub(super) fn CompleteFontResources(&self) {
        let results = self
            .pending_fonts
            .borrow()
            .iter()
            .enumerate()
            .filter_map(|(index, face)| {
                face.resource
                    .as_ref()
                    .and_then(|resource| resource.Decoded())
                    .map(|result| (index, result))
            })
            .collect::<Vec<_>>();
        for (index, result) in results {
            self.FinishFontAt(index, result);
        }
        let set = self.FontLoadingDocument().FontFaceSet();
        let mut pending = self.pending_fonts.borrow_mut();
        for face in pending.iter_mut() {
            let connected = set.Has(&face.owner);
            if !connected && face.published {
                self.font_loading.ready.borrow_mut().push(
                    ResourceMutation::FontFaceResourceChanged {
                        owner: face.owner.Owner().0,
                        font: None,
                    },
                );
                face.published = false;
            }
            if !connected || (face.published && !face.dirty) {
                continue;
            }
            let Some(bytes) = face.decoded.clone() else {
                continue;
            };
            self.font_loading
                .ready
                .borrow_mut()
                .push(ResourceMutation::FontFaceResourceChanged {
                    owner: face.owner.Owner().0,
                    font: Some(FontFace {
                        family: face.metadata.family.clone(),
                        weight: face.metadata.weight,
                        italic: face.metadata.italic,
                        bytes,
                        font_feature_settings: face.metadata.font_feature_settings.clone(),
                        unicode_ranges: face.metadata.unicode_ranges.clone(),
                        // local() keeps the provider's exact TTC index,
                        // variation coordinates and opened immutable bytes.
                        ..face.local_face.clone().unwrap_or_default()
                    }),
                });
            face.published = true;
            face.dirty = false;
        }
    }
}

impl ResourceFetcher {
    /// LinkLoader::LoadLink cc111-143; PreloadHelper::PreloadIfNeeded
    /// cc430-598. Called per original DOM producer, including parser insertion
    /// before the following token can insert body or execute a script.
    pub fn ProcessFontPreloadMutations(&self, document: &Document) {
        use dom::font_preload::FontPreloadReason;
        let context = document.FontLoading();
        for params in context.TakePreloads() {
            let previous = self.font_loading.preloads.borrow_mut().remove(&params.node);
            let is_font = params.active
                && params
                    .rel
                    .split_ascii_whitespace()
                    .any(|rel| rel.eq_ignore_ascii_case("preload"))
                && !params
                    .rel
                    .split_ascii_whitespace()
                    .any(|rel| rel.eq_ignore_ascii_case("expect"))
                && params.as_.eq_ignore_ascii_case("font");
            if !is_font {
                if let Some((_, previous)) = previous {
                    previous.Dispose();
                }
                continue;
            }
            let pending = match previous {
                Some((_, previous))
                    if params.reason == FontPreloadReason::MediaChange
                        && previous.MatchesMedia() =>
                {
                    previous
                }
                previous => {
                    if let Some((_, previous)) = previous {
                        previous.Dispose();
                    }
                    let original = Rc::downgrade(&context);
                    let node = params.node;
                    PendingLinkPreload::new(
                        document.GetRenderBlockingResourceManager(),
                        Rc::new(move |loaded| {
                            if let Some(original) = original
                                .upgrade()
                                .filter(|context| context.HasExecutionContext())
                            {
                                original.Push(
                                    dom::font_loading::FontLoadingEffect::LinkPreloadEvent {
                                        node,
                                        loaded,
                                    },
                                );
                            }
                        }),
                    )
                }
            };
            let key = params.href.clone().unwrap_or_default();
            self.font_loading
                .preloads
                .borrow_mut()
                .insert(params.node, (key, pending.clone()));
            let Some(url) = params.href else {
                context.Push(dom::font_loading::FontLoadingEffect::ConsoleWarning(
                    "<link rel=preload> has an invalid `href` value".into(),
                ));
                continue;
            };
            let matches = self.PreloadMediaMatches(&params.media);
            if params.reason == FontPreloadReason::MediaChange {
                if !matches {
                    pending.Dispose();
                } else if pending.MatchesMedia() {
                    continue;
                }
            }
            pending.SetMatchesMedia(matches);
            if !matches {
                continue;
            }
            let supported_type = params.type_.is_empty()
                || params.type_.split_once('/').is_some_and(|(kind, subtype)| {
                    kind.eq_ignore_ascii_case("font")
                        && ["woff", "woff2", "otf", "ttf", "sfnt"]
                            .iter()
                            .any(|known| subtype.eq_ignore_ascii_case(known))
                });
            if !supported_type {
                context.Push(dom::font_loading::FontLoadingEffect::ConsoleWarning(
                    "<link rel=preload> has an unsupported `type` value".into(),
                ));
                continue;
            }
            pending.SetIntegrity(params.integrity.clone());
            pending.RegisterRenderBlocking(params.has_body);
            let (mode, credentials) = match params.cross_origin.as_deref() {
                None => (
                    url_loader::FetchMode::NoCors,
                    url_loader::FetchCredentialsMode::Include,
                ),
                Some(value) if value.eq_ignore_ascii_case("use-credentials") => (
                    url_loader::FetchMode::Cors,
                    url_loader::FetchCredentialsMode::Include,
                ),
                Some(_) => (
                    url_loader::FetchMode::Cors,
                    url_loader::FetchCredentialsMode::SameOrigin,
                ),
            };
            let request = URLRequest {
                url,
                referrer_policy: ReferrerPolicy::FromLegacyAttribute(&params.referrer_policy),
                destination: RequestDestination::kFont,
                ..Default::default()
            };
            // Source registers before StartPreload. A blocked/null resource
            // retains its exact token until dispose or the manager timeout.
            match self.FontResourceInDocument(
                request,
                CSPWorld::Main,
                mode,
                credentials,
                document,
                resource::ResourceLoadEventPolicy::LinkPreload,
            ) {
                Ok(resource) => pending.AddResource(resource),
                Err(error) => context.Push(dom::font_loading::FontLoadingEffect::ConsoleWarning(
                    error.to_string(),
                )),
            }
        }
    }
}
