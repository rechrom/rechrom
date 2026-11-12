//! Chromium core/css/font_face.{h,cc}:153-163,568-667,1078-1088 and
//! font_face_set.cc:24-62,158-184,272-325; font_face_set_document.cc:68-171.
//! Font resource work stays in document_loader. These are the retained face,
//! callback and set owners, shared by CSS demand and the imperative API.
use crate::render_blocking_resource_manager::{FontFaceOwner, RenderBlockingResourceManager};
use foundation::{AtomicString, String as BlinkString};
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, VecDeque},
    rc::{Rc, Weak},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
};
use style_parser::{at_rule_descriptors::AtRuleDescriptorID as D, production_css_value::Value};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontFaceLoadStatus {
    Unloaded,
    Loading,
    Loaded,
    Error,
}
impl FontFaceLoadStatus {
    pub fn Name(self) -> &'static str {
        match self {
            Self::Unloaded => "unloaded",
            Self::Loading => "loading",
            Self::Loaded => "loaded",
            Self::Error => "error",
        }
    }
}
#[derive(Clone, Debug)]
pub struct FontFaceError {
    pub name: &'static str,
    pub message: BlinkString,
}
pub trait LoadFontCallback {
    fn NotifyLoaded(&self, face: &Rc<FontFace>);
    fn NotifyError(&self, face: &Rc<FontFace>);
}
/// The original CSSFontFace::Load collaborator, constructed internally by the
/// resource owner and borrowed only during a synchronous load call.
pub trait FontFaceLoadSource {
    fn Load(&self, face: &Rc<FontFace>);
}
pub trait FontFaceLoadedPromise {
    fn Resolve(&self, face: Rc<FontFace>);
    fn Reject(&self, error: Rc<FontFaceError>);
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontFaceSetEventKind {
    Loading,
    LoadingDone,
    LoadingError,
}
pub enum FontLoadingEffect {
    /// Source task types: kDOMManipulation=1, kFontLoading=56. Native routing
    /// supplies the original document task source, not a private font pump.
    Task {
        task_type: u8,
        run: Box<dyn FnOnce()>,
    },
    SetEvent {
        kind: FontFaceSetEventKind,
        faces: Vec<Rc<FontFace>>,
    },
    LinkPreloadEvent {
        node: u64,
        loaded: bool,
    },
    ConsoleWarning(String),
    SetReady {
        generation: u64,
    },
    /// FontFaceSetDocument FireDoneEventIfPossible must synchronously update
    /// style/layout, then recheck the same owner through CompleteLayoutCheck.
    CheckStyleAndLayout {
        set: Rc<FontFaceSet>,
    },
    FaceDescriptorChanged {
        face: Rc<FontFace>,
    },
}
static NEXT_FACE: AtomicU64 = AtomicU64::new(1);
pub struct FontFace {
    owner: FontFaceOwner,
    world: foundation::content_security_policy::CSPWorld,
    context: Weak<FontLoadingDocument>,
    family: RefCell<AtomicString>,
    descriptors: RefCell<HashMap<u16, Rc<Value>>>,
    status: Cell<FontFaceLoadStatus>,
    error: RefCell<Option<Rc<FontFaceError>>>,
    callbacks: RefCell<Vec<Rc<dyn LoadFontCallback>>>,
    loaded_property: RefCell<Option<Rc<dyn FontFaceLoadedPromise>>>,
}
impl FontFace {
    pub fn new(
        context: &Rc<FontLoadingDocument>,
        family: AtomicString,
        world: foundation::content_security_policy::CSPWorld,
    ) -> Rc<Self> {
        let face = Rc::new(Self {
            owner: FontFaceOwner(NEXT_FACE.fetch_add(1, Ordering::Relaxed)),
            world,
            context: Rc::downgrade(context),
            family: RefCell::new(family),
            descriptors: RefCell::new(HashMap::new()),
            status: Cell::new(FontFaceLoadStatus::Unloaded),
            error: RefCell::new(None),
            callbacks: RefCell::new(Vec::new()),
            loaded_property: RefCell::new(None),
        });
        context
            .faces
            .borrow_mut()
            .insert(face.owner, Rc::downgrade(&face));
        face
    }
    pub fn World(&self) -> foundation::content_security_policy::CSPWorld {
        self.world
    }
    pub fn Context(&self) -> Option<Rc<FontLoadingDocument>> {
        self.context.upgrade()
    }
    pub fn Owner(&self) -> FontFaceOwner {
        self.owner
    }
    pub fn Family(&self) -> AtomicString {
        self.family.borrow().clone()
    }
    pub fn SetFamily(self: &Rc<Self>, family: AtomicString) {
        if *self.family.borrow() == family {
            return;
        }
        *self.family.borrow_mut() = family;
        if let Some(context) = self.context.upgrade() {
            context.Push(FontLoadingEffect::FaceDescriptorChanged { face: self.clone() });
        }
    }
    pub fn Descriptor(&self, id: D) -> Option<Rc<Value>> {
        self.descriptors.borrow().get(&(id as u16)).cloned()
    }
    pub fn SetDescriptor(self: &Rc<Self>, id: D, value: Rc<Value>) {
        self.descriptors.borrow_mut().insert(id as u16, value);
        if let Some(context) = self.context.upgrade() {
            context.Push(FontLoadingEffect::FaceDescriptorChanged { face: self.clone() });
        }
    }
    pub fn LoadStatus(&self) -> FontFaceLoadStatus {
        self.status.get()
    }
    pub fn GetError(&self) -> Option<Rc<FontFaceError>> {
        self.error.borrow().clone()
    }
    pub fn HasPendingActivity(&self) -> bool {
        self.status.get() == FontFaceLoadStatus::Loading
            && self
                .context
                .upgrade()
                .is_some_and(|c| c.HasExecutionContext())
    }
    pub fn SetLoadStatus(self: &Rc<Self>, status: FontFaceLoadStatus) {
        self.status.set(status);
        debug_assert!(status != FontFaceLoadStatus::Error || self.error.borrow().is_some());
        let Some(context) = self.context.upgrade().filter(|c| c.HasExecutionContext()) else {
            return;
        };
        if matches!(
            status,
            FontFaceLoadStatus::Loaded | FontFaceLoadStatus::Error
        ) {
            let promise = self.loaded_property.borrow().clone();
            if let Some(promise) = promise {
                let face = self.clone();
                let error = self.GetError();
                context.Push(FontLoadingEffect::Task {
                    task_type: 1,
                    run: Box::new(move || {
                        if status == FontFaceLoadStatus::Loaded {
                            promise.Resolve(face);
                        } else {
                            promise.Reject(error.expect("source FontFace error"));
                        }
                    }),
                });
            }
            let face = self.clone();
            context.Push(FontLoadingEffect::Task {
                task_type: 1,
                run: Box::new(move || face.RunCallbacks()),
            });
        }
    }
    pub fn SetError(self: &Rc<Self>, error: Option<FontFaceError>) {
        if self.error.borrow().is_none() {
            *self.error.borrow_mut() = Some(Rc::new(error.unwrap_or(FontFaceError {
                name: "NetworkError",
                message: BlinkString::default(),
            })));
        }
        self.SetLoadStatus(FontFaceLoadStatus::Error);
    }
    fn RunCallbacks(self: &Rc<Self>) {
        let callbacks = std::mem::take(&mut *self.callbacks.borrow_mut());
        for callback in callbacks {
            if self.status.get() == FontFaceLoadStatus::Loaded {
                callback.NotifyLoaded(self);
            } else {
                callback.NotifyError(self);
            }
        }
    }
    pub fn FontStatusPromise(
        self: &Rc<Self>,
        create: impl FnOnce() -> Rc<dyn FontFaceLoadedPromise>,
    ) -> Rc<dyn FontFaceLoadedPromise> {
        if let Some(promise) = self.loaded_property.borrow().clone() {
            return promise;
        }
        let promise = create();
        *self.loaded_property.borrow_mut() = Some(promise.clone());
        match self.status.get() {
            FontFaceLoadStatus::Loaded => promise.Resolve(self.clone()),
            FontFaceLoadStatus::Error => promise.Reject(self.GetError().expect("source error")),
            _ => {}
        }
        promise
    }
    pub fn Load(self: &Rc<Self>, source: &dyn FontFaceLoadSource) {
        if self.status.get() == FontFaceLoadStatus::Unloaded {
            source.Load(self);
        }
        if let Some(context) = self.context.upgrade() {
            context.AddImperativeFontLoading(self);
        }
    }
    pub fn LoadWithCallback(
        self: &Rc<Self>,
        source: &dyn FontFaceLoadSource,
        callback: Rc<dyn LoadFontCallback>,
    ) {
        if self.status.get() == FontFaceLoadStatus::Unloaded {
            source.Load(self);
        }
        self.AddCallback(callback);
    }
    pub fn AddCallback(self: &Rc<Self>, callback: Rc<dyn LoadFontCallback>) {
        match self.status.get() {
            FontFaceLoadStatus::Loaded => callback.NotifyLoaded(self),
            FontFaceLoadStatus::Error => callback.NotifyError(self),
            _ => self.callbacks.borrow_mut().push(callback),
        }
    }
}
pub struct FontLoadingDocument {
    faces: RefCell<HashMap<FontFaceOwner, Weak<FontFace>>>,
    preloads: RefCell<VecDeque<crate::font_preload::FontPreloadMutation>>,
    manager: RefCell<Option<Rc<RefCell<RenderBlockingResourceManager>>>>,
    lifetime: RefCell<Arc<AtomicBool>>,
    has_window: Cell<bool>,
    effects: RefCell<VecDeque<FontLoadingEffect>>,
    notifier: RefCell<Option<Rc<dyn Fn()>>>,
    font_set: RefCell<Option<Rc<FontFaceSet>>>,
}
impl Default for FontLoadingDocument {
    fn default() -> Self {
        Self {
            faces: RefCell::new(HashMap::new()),
            preloads: RefCell::new(VecDeque::new()),
            manager: RefCell::new(None),
            lifetime: RefCell::new(Arc::new(AtomicBool::new(false))),
            has_window: Cell::new(false),
            effects: RefCell::new(VecDeque::new()),
            notifier: RefCell::new(None),
            font_set: RefCell::new(None),
        }
    }
}
impl FontLoadingDocument {
    pub(crate) fn RecordPreload(&self, mutation: crate::font_preload::FontPreloadMutation) {
        self.preloads.borrow_mut().push_back(mutation);
    }
    pub fn TakePreloads(&self) -> VecDeque<crate::font_preload::FontPreloadMutation> {
        std::mem::take(&mut *self.preloads.borrow_mut())
    }
    pub fn BindDocument(
        &self,
        lifetime: Arc<AtomicBool>,
        manager: Option<Rc<RefCell<RenderBlockingResourceManager>>>,
        has_window: bool,
    ) {
        *self.lifetime.borrow_mut() = lifetime;
        *self.manager.borrow_mut() = manager;
        self.has_window.set(has_window);
    }
    pub fn HasExecutionContext(&self) -> bool {
        self.has_window.get() && self.lifetime.borrow().load(Ordering::Acquire)
    }
    pub fn BindEffectNotifier(&self, notifier: Rc<dyn Fn()>) {
        *self.notifier.borrow_mut() = Some(notifier);
    }
    pub fn Push(&self, effect: FontLoadingEffect) {
        self.effects.borrow_mut().push_back(effect);
        let notify = self.notifier.borrow().clone();
        if let Some(notify) = notify {
            notify();
        }
    }
    pub fn QueueTask(&self, task_type: u8, run: Box<dyn FnOnce()>) {
        self.Push(FontLoadingEffect::Task { task_type, run });
    }
    pub fn TakeEffects(&self) -> VecDeque<FontLoadingEffect> {
        std::mem::take(&mut *self.effects.borrow_mut())
    }
    pub fn Face(&self, id: FontFaceOwner) -> Option<Rc<FontFace>> {
        self.faces.borrow().get(&id).and_then(Weak::upgrade)
    }
    pub fn FontFaceSet(self: &Rc<Self>) -> Rc<FontFaceSet> {
        self.font_set
            .borrow_mut()
            .get_or_insert_with(|| FontFaceSet::new(self))
            .clone()
    }
    pub fn AddImperativeFontLoading(&self, face: &Rc<FontFace>) {
        let Some(manager) = self.manager.borrow().clone() else {
            return;
        };
        let subscriptions = {
            let mut source = manager.borrow_mut();
            let body = source.DocumentHasBody();
            source.AddImperativeFontLoading(
                face.Owner(),
                face.LoadStatus() == FontFaceLoadStatus::Loading,
                body,
            );
            source.TakeImperativeFontSubscriptions()
        };
        for id in subscriptions {
            let face = self.Face(id).expect("same registered FontFace owner");
            face.AddCallback(Rc::new(ImperativeFontLoadFinishedCallback {
                manager: manager.clone(),
            }));
        }
    }
}
struct ImperativeFontLoadFinishedCallback {
    manager: Rc<RefCell<RenderBlockingResourceManager>>,
}
impl ImperativeFontLoadFinishedCallback {
    fn Finish(&self) {
        let mut manager = self.manager.borrow_mut();
        let body = manager.DocumentHasBody();
        manager.RemoveImperativeFontLoading(body);
    }
}
impl LoadFontCallback for ImperativeFontLoadFinishedCallback {
    fn NotifyLoaded(&self, _: &Rc<FontFace>) {
        self.Finish();
    }
    fn NotifyError(&self, _: &Rc<FontFace>) {
        self.Finish();
    }
}
pub struct FontFaceSet {
    context: Weak<FontLoadingDocument>,
    is_loading: Cell<bool>,
    should_fire_loading_event: Cell<bool>,
    pending_task_queued: Cell<bool>,
    ready_pending: Cell<bool>,
    ready_generation: Cell<u64>,
    loading: RefCell<Vec<Rc<FontFace>>>,
    loaded: RefCell<Vec<Rc<FontFace>>>,
    failed: RefCell<Vec<Rc<FontFace>>>,
    css_connected: RefCell<Vec<Rc<FontFace>>>,
    non_css_connected: RefCell<Vec<Rc<FontFace>>>,
}
impl FontFaceSet {
    fn new(context: &Rc<FontLoadingDocument>) -> Rc<Self> {
        Rc::new(Self {
            context: Rc::downgrade(context),
            is_loading: Cell::new(false),
            should_fire_loading_event: Cell::new(false),
            pending_task_queued: Cell::new(false),
            ready_pending: Cell::new(true),
            ready_generation: Cell::new(0),
            loading: RefCell::new(Vec::new()),
            loaded: RefCell::new(Vec::new()),
            failed: RefCell::new(Vec::new()),
            css_connected: RefCell::new(Vec::new()),
            non_css_connected: RefCell::new(Vec::new()),
        })
    }
    pub fn IsLoading(&self) -> bool {
        self.is_loading.get()
    }
    pub fn ReadyPending(&self) -> bool {
        self.ready_pending.get()
    }
    pub fn NonCSSConnected(&self) -> Vec<Rc<FontFace>> {
        self.non_css_connected.borrow().clone()
    }
    pub fn ReadyGeneration(&self) -> u64 {
        self.ready_generation.get()
    }
    pub fn Has(&self, face: &Rc<FontFace>) -> bool {
        self.css_connected
            .borrow()
            .iter()
            .chain(self.non_css_connected.borrow().iter())
            .any(|f| Rc::ptr_eq(f, face))
    }
    pub fn IsCSSConnected(&self, face: &Rc<FontFace>) -> bool {
        self.css_connected
            .borrow()
            .iter()
            .any(|f| Rc::ptr_eq(f, face))
    }
    pub fn Snapshot(&self) -> Vec<Rc<FontFace>> {
        self.css_connected
            .borrow()
            .iter()
            .chain(self.non_css_connected.borrow().iter())
            .cloned()
            .collect()
    }
    pub fn AddCSSConnected(self: &Rc<Self>, face: Rc<FontFace>) {
        if !self.IsCSSConnected(&face) {
            self.css_connected.borrow_mut().push(face.clone());
            if face.LoadStatus() == FontFaceLoadStatus::Loading {
                self.BeginFontLoading(face);
            }
        }
    }
    pub fn Add(self: &Rc<Self>, face: Rc<FontFace>) -> bool {
        if self.Has(&face) {
            return false;
        }
        self.non_css_connected.borrow_mut().push(face.clone());
        if face.LoadStatus() == FontFaceLoadStatus::Loading {
            self.BeginFontLoading(face);
        }
        true
    }
    pub fn Delete(self: &Rc<Self>, face: &Rc<FontFace>) -> bool {
        let index = self
            .non_css_connected
            .borrow()
            .iter()
            .position(|f| Rc::ptr_eq(f, face));
        let Some(index) = index else {
            return false;
        };
        self.non_css_connected.borrow_mut().remove(index);
        if face.LoadStatus() == FontFaceLoadStatus::Loading {
            self.RemoveFromLoadingFonts(face);
        }
        true
    }
    pub fn Clear(self: &Rc<Self>) {
        let faces = std::mem::take(&mut *self.non_css_connected.borrow_mut());
        for face in faces {
            if face.LoadStatus() == FontFaceLoadStatus::Loading {
                self.RemoveFromLoadingFonts(&face);
            }
        }
    }
    pub fn BeginFontLoading(self: &Rc<Self>, face: Rc<FontFace>) {
        if !self.is_loading.replace(true) {
            self.should_fire_loading_event.set(true);
            if !self.ready_pending.replace(true) {
                self.ready_generation.set(self.ready_generation.get() + 1);
            }
            self.HandleSoon();
        }
        if !self.loading.borrow().iter().any(|f| Rc::ptr_eq(f, &face)) {
            self.loading.borrow_mut().push(face.clone());
        }
        face.AddCallback(Rc::new(FontFaceSetCallback {
            set: Rc::downgrade(self),
        }));
    }
    fn RemoveFromLoadingFonts(self: &Rc<Self>, face: &Rc<FontFace>) {
        self.loading.borrow_mut().retain(|f| !Rc::ptr_eq(f, face));
        if self.loading.borrow().is_empty() {
            self.HandleSoon();
        }
    }
    fn HandleSoon(self: &Rc<Self>) {
        if self.pending_task_queued.get() {
            return;
        }
        let Some(context) = self.context.upgrade().filter(|c| c.HasExecutionContext()) else {
            return;
        };
        self.pending_task_queued.set(true);
        let set = self.clone();
        context.Push(FontLoadingEffect::Task {
            task_type: 56,
            run: Box::new(move || set.HandlePendingEventsAndPromises()),
        });
    }
    fn HandlePendingEventsAndPromises(self: &Rc<Self>) {
        self.pending_task_queued.set(false);
        let Some(context) = self.context.upgrade().filter(|c| c.HasExecutionContext()) else {
            return;
        };
        if self.should_fire_loading_event.replace(false) {
            context.Push(FontLoadingEffect::SetEvent {
                kind: FontFaceSetEventKind::Loading,
                faces: Vec::new(),
            });
        }
        if self.ShouldSignalReady() {
            context.Push(FontLoadingEffect::CheckStyleAndLayout { set: self.clone() });
        }
    }
    pub fn DidLayout(self: &Rc<Self>) {
        if self.ShouldSignalReady() {
            self.HandleSoon();
        }
    }
    fn ShouldSignalReady(&self) -> bool {
        self.loading.borrow().is_empty() && (self.is_loading.get() || self.ready_pending.get())
    }
    pub fn CompleteLayoutCheck(self: &Rc<Self>) {
        if self.should_fire_loading_event.get() || !self.ShouldSignalReady() {
            return;
        }
        let Some(context) = self.context.upgrade().filter(|c| c.HasExecutionContext()) else {
            return;
        };
        if self.is_loading.replace(false) {
            let loaded = std::mem::take(&mut *self.loaded.borrow_mut());
            let failed = std::mem::take(&mut *self.failed.borrow_mut());
            context.Push(FontLoadingEffect::SetEvent {
                kind: FontFaceSetEventKind::LoadingDone,
                faces: loaded,
            });
            if !failed.is_empty() {
                context.Push(FontLoadingEffect::SetEvent {
                    kind: FontFaceSetEventKind::LoadingError,
                    faces: failed,
                });
            }
        }
        if self.ready_pending.replace(false) {
            context.Push(FontLoadingEffect::SetReady {
                generation: self.ready_generation.get(),
            });
        }
    }
}
struct FontFaceSetCallback {
    set: Weak<FontFaceSet>,
}
impl LoadFontCallback for FontFaceSetCallback {
    fn NotifyLoaded(&self, face: &Rc<FontFace>) {
        if let Some(set) = self.set.upgrade() {
            set.loaded.borrow_mut().push(face.clone());
            set.RemoveFromLoadingFonts(face);
        }
    }
    fn NotifyError(&self, face: &Rc<FontFace>) {
        if let Some(set) = self.set.upgrade() {
            set.failed.borrow_mut().push(face.clone());
            set.RemoveFromLoadingFonts(face);
        }
    }
}

impl crate::persistent_document::PersistentDocument {
    pub fn FontLoading(&self) -> Rc<FontLoadingDocument> {
        let owner = self
            .document_metadata
            .get(&self.Node(self.Root()).Id())
            .expect("Document font loading supplement")
            .font_loading
            .clone();
        owner.BindDocument(
            self.NativeExecutionContextLifetime(),
            self.GetRenderBlockingResourceManager(),
            self.HasDOMWindow(),
        );
        owner
    }
}

/// FontFaceDescriptors IDL defaults in source order (stable variation flag).
pub const FONT_FACE_DESCRIPTORS: &[(D, &str, &str)] = &[
    (D::FontStyle, "style", "normal"),
    (D::FontWeight, "weight", "normal"),
    (D::FontStretch, "stretch", "normal"),
    (D::UnicodeRange, "unicodeRange", "U+0-10FFFF"),
    (D::FontVariant, "variant", "normal"),
    (D::FontFeatureSettings, "featureSettings", "normal"),
    (D::FontVariationSettings, "variationSettings", "normal"),
    (D::FontDisplay, "display", "auto"),
    (D::AscentOverride, "ascentOverride", "normal"),
    (D::DescentOverride, "descentOverride", "normal"),
    (D::LineGapOverride, "lineGapOverride", "normal"),
    (D::SizeAdjust, "sizeAdjust", "100%"),
];
impl FontFace {
    pub fn InitializeDescriptor(&self, id: D, value: Rc<Value>) {
        self.descriptors.borrow_mut().insert(id as u16, value);
    }
    pub fn InitializeFromStrings<
        P: style_parser::css_parser_context::CSSParserContextPlatform<
            URL = style_parser::production_css_value::ProductionURL,
        >,
    >(
        context: &Rc<FontLoadingDocument>,
        family: AtomicString,
        descriptors: &[(D, BlinkString)],
        world: foundation::content_security_policy::CSPWorld,
        parser: &style_parser::css_parser_context::CSSParserContext<P>,
    ) -> Rc<Self> {
        let face = Self::new(context, family, world);
        for &(id, _, default) in FONT_FACE_DESCRIPTORS {
            let spelling = descriptors
                .iter()
                .find(|(key, _)| *key == id)
                .map(|(_, value)| value.clone())
                .unwrap_or_else(|| BlinkString::from(default));
            match style_parser::font_face_descriptors::ParseFontFaceDescriptor(
                id, &spelling, parser,
            ) {
                Ok(value) => face.InitializeDescriptor(id, value),
                Err(_) => face.SetError(Some(FontFaceError {
                    name: "SyntaxError",
                    message: BlinkString::from(format!(
                        "Failed to set '{}' as a property value.",
                        spelling.Utf8()
                    )),
                })),
            }
        }
        face
    }
    pub fn InitializeSource<
        P: style_parser::css_parser_context::CSSParserContextPlatform<
            URL = style_parser::production_css_value::ProductionURL,
        >,
    >(
        self: &Rc<Self>,
        source: &BlinkString,
        parser: &style_parser::css_parser_context::CSSParserContext<P>,
    ) {
        match style_parser::font_face_descriptors::ParseFontFaceDescriptor(D::Src, source, parser) {
            Ok(value) => self.InitializeDescriptor(D::Src, value),
            Err(_) => self.SetError(Some(FontFaceError {
                name: "SyntaxError",
                message: BlinkString::from(format!(
                    "The source provided ('{}') could not be parsed as a value list.",
                    source.Utf8()
                )),
            })),
        }
    }
    pub fn FromCompiled(
        context: &Rc<FontLoadingDocument>,
        compiled: &style_parser::font_face_descriptors::CompiledFontFaceDescriptors,
    ) -> Option<Rc<Self>> {
        use style_parser::css_value::CSSValuePayload as V;
        let family = compiled.Get(D::FontFamily)?;
        let V::kFontFamilyClass(family) = family.Payload() else {
            return None;
        };
        let source = compiled.Get(D::Src)?;
        if !matches!(source.Payload(), V::kValueListClass(_)) {
            return None;
        }
        let units = (0..family.0.length())
            .map(|i| family.0.CodeUnitAt(i as u32))
            .collect::<Vec<_>>();
        let face = Self::new(
            context,
            AtomicString::from_utf16(&units),
            foundation::content_security_policy::CSPWorld::Main,
        );
        for (id, value) in &compiled.0 {
            face.InitializeDescriptor(*id, value.clone());
        }
        Some(face)
    }
    pub fn DescriptorText(&self, id: D) -> BlinkString {
        self.Descriptor(id)
            .map(|value| value.CssText())
            .unwrap_or_else(|| {
                BlinkString::from(
                    FONT_FACE_DESCRIPTORS
                        .iter()
                        .find(|(key, _, _)| *key == id)
                        .map_or("", |(_, _, default)| *default),
                )
            })
    }
    pub fn SetDescriptorFromString<
        P: style_parser::css_parser_context::CSSParserContextPlatform<
            URL = style_parser::production_css_value::ProductionURL,
        >,
    >(
        self: &Rc<Self>,
        id: D,
        text: &BlinkString,
        parser: &style_parser::css_parser_context::CSSParserContext<P>,
    ) -> Result<(), FontFaceError> {
        let value = style_parser::font_face_descriptors::ParseFontFaceDescriptor(id, text, parser)
            .map_err(|_| FontFaceError {
                name: "SyntaxError",
                message: BlinkString::from(format!(
                    "Failed to set '{}' as a property value.",
                    text.Utf8()
                )),
            })?;
        let old = self.Descriptor(id);
        self.InitializeDescriptor(id, value.clone());
        if matches!(
            id,
            D::FontStyle
                | D::FontWeight
                | D::FontStretch
                | D::UnicodeRange
                | D::FontFeatureSettings
                | D::FontVariationSettings
        ) && old.as_ref().is_none_or(|old| !Rc::ptr_eq(old, &value))
        {
            if let Some(context) = self.context.upgrade() {
                context.Push(FontLoadingEffect::FaceDescriptorChanged { face: self.clone() });
            }
        }
        Ok(())
    }
}

/// FontFaceSet::LoadFontPromiseResolver, font_face_set.cc:178-233 and
/// font_face_set.h:125-151. Holds the original matched array and one callback
/// counter; it never composes per-face JavaScript promises.
pub trait FontFaceArrayPromise {
    fn Resolve(&self, faces: &[Rc<FontFace>]);
    fn Reject(&self, error: Rc<FontFaceError>);
}
pub struct LoadFontPromiseResolver {
    faces: Vec<Rc<FontFace>>,
    num_loading: Cell<usize>,
    error_occurred: Cell<bool>,
    promise: Rc<dyn FontFaceArrayPromise>,
}
impl LoadFontPromiseResolver {
    pub fn new(faces: Vec<Rc<FontFace>>, promise: Rc<dyn FontFaceArrayPromise>) -> Rc<Self> {
        Rc::new(Self {
            num_loading: Cell::new(faces.len()),
            faces,
            error_occurred: Cell::new(false),
            promise,
        })
    }
    pub fn LoadFonts(self: &Rc<Self>, source: &dyn FontFaceLoadSource) {
        if self.num_loading.get() == 0 {
            self.promise.Resolve(&self.faces);
            return;
        }
        for face in &self.faces {
            face.LoadWithCallback(source, self.clone());
            if let Some(context) = face.Context() {
                context.AddImperativeFontLoading(face);
            }
        }
    }
}
impl LoadFontCallback for LoadFontPromiseResolver {
    fn NotifyLoaded(&self, _: &Rc<FontFace>) {
        self.num_loading.set(
            self.num_loading
                .get()
                .checked_sub(1)
                .expect("one source face callback"),
        );
        if self.num_loading.get() == 0 && !self.error_occurred.get() {
            self.promise.Resolve(&self.faces);
        }
    }
    fn NotifyError(&self, face: &Rc<FontFace>) {
        self.num_loading.set(
            self.num_loading
                .get()
                .checked_sub(1)
                .expect("one source face callback"),
        );
        if !self.error_occurred.replace(true) {
            self.promise.Reject(
                face.GetError()
                    .expect("source failed FontFace has an error"),
            );
        }
    }
}
