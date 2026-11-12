//! Chromium 6c1d401f: core/loader/render_blocking_resource_manager.h/cc and
//! render_blocking_element_link_map.h/cc. Document owns this retained state;
//! resource clients provide typed original identities and Page routes effects.
use std::{
    collections::{HashMap, HashSet, VecDeque},
    rc::Rc,
    time::{Duration, Instant},
};
macro_rules! identity {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub struct $name(pub u64);
    };
}
identity!(StylesheetOwner);
identity!(ScriptOwner);
/// Immutable ScriptLoader::ScriptTypeAtPrepare / IsParserInserted facts at
/// original pending-script admission (not recomputed from mutable type attrs).
#[derive(Clone, Copy, Debug)]
pub enum ScriptRenderBlockingKind {
    ParserInsertedClassic,
    Other,
}
identity!(FontPreloadOwner);
identity!(FontFaceOwner);
identity!(ExpectLinkOwner);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderBlockingLevel {
    None,
    Block,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontTimer {
    MaxBlocking,
    MaxFCPDelay,
}
#[derive(Clone, Debug)]
pub enum RenderBlockingEffect {
    StartFontTimer {
        kind: FontTimer,
        generation: u64,
        deadline: Instant,
    },
    RenderBlockingResourcesLoadedMetric,
    MissingExpectedElement {
        link: ExpectLinkOwner,
        href: String,
    },
    ExpiredFontsHistogram {
        had_pending_fonts: bool,
    },
}
/// Exact downstream LocalFrameView request. This queue remains pending until
/// the owning frame-view lifecycle implementation consumes it; ordinary Page
/// scheduling is not a substitute for BeginLifecycleUpdates.
#[derive(Clone, Copy, Debug)]
pub enum RenderBlockingFrameEffect {
    BeginLifecycleUpdates,
}
/// Shared DocumentLoader load timing, never a second navigation record.
pub struct DocumentLoadTiming {
    pub navigation_start: std::cell::Cell<Option<Instant>>,
    pub unload_event: std::cell::Cell<Option<navigation_history::UnloadEventTiming>>,
    pub commit_navigation_end: std::cell::Cell<Option<Instant>>,
    now: Rc<dyn Fn() -> Instant>,
    wall_now: Rc<dyn Fn() -> std::time::SystemTime>,
    reference_monotonic_time: std::cell::Cell<Option<Instant>>,
    reference_wall_time_us: std::cell::Cell<Option<i64>>,
}
pub type RenderBlockingLoadTiming = DocumentLoadTiming;
impl DocumentLoadTiming {
    pub fn new(now: Rc<dyn Fn() -> Instant>) -> Rc<Self> {
        Self::WithClocks(now, Rc::new(std::time::SystemTime::now))
    }
    pub fn WithClocks(
        now: Rc<dyn Fn() -> Instant>,
        wall_now: Rc<dyn Fn() -> std::time::SystemTime>,
    ) -> Rc<Self> {
        Rc::new(Self {
            navigation_start: Default::default(),
            unload_event: Default::default(),
            commit_navigation_end: Default::default(),
            now,
            wall_now,
            reference_monotonic_time: Default::default(),
            reference_wall_time_us: Default::default(),
        })
    }
    pub fn Now(&self) -> Instant {
        (self.now)()
    }
    /// DocumentLoadTiming.cc:62-105. Capture the platform pair once, before
    /// reading either navigation-relative or pseudo-wall reporting timestamps.
    fn EnsureReferenceTimesSet(&self) {
        if self.reference_wall_time_us.get().is_none() {
            let wall = (self.wall_now)();
            let micros = match wall.duration_since(std::time::UNIX_EPOCH) {
                Ok(delta) => i64::try_from(delta.as_micros()).expect("source wall clock range"),
                Err(error) => {
                    -i64::try_from(error.duration().as_micros()).expect("source wall clock range")
                }
            };
            self.reference_wall_time_us.set(Some(micros));
        }
        if self.reference_monotonic_time.get().is_none() {
            self.reference_monotonic_time.set(Some(self.Now()));
        }
    }
    pub fn MarkNavigationStart(&self) {
        if self.navigation_start.get().is_some() {
            return;
        }
        self.EnsureReferenceTimesSet();
        self.navigation_start
            .set(self.reference_monotonic_time.get());
    }
    pub fn ReferenceMonotonicTime(&self) -> Option<Instant> {
        self.reference_monotonic_time.get()
    }
    pub fn MonotonicTimeToPseudoWallMicroseconds(&self, time: Option<Instant>) -> i64 {
        let (Some(time), Some(reference)) = (time, self.reference_monotonic_time.get()) else {
            return 0;
        };
        let delta = if time >= reference {
            i128::try_from(time.duration_since(reference).as_micros()).expect("source clock range")
        } else {
            -i128::try_from(reference.duration_since(time).as_micros()).expect("source clock range")
        };
        i64::try_from(
            delta
                + i128::from(
                    self.reference_wall_time_us
                        .get()
                        .expect("paired source clocks"),
                ),
        )
        .expect("source pseudo-wall range")
    }
    /// .cc:126-144. Adjust both reference samples, preserving their mapping;
    /// this is an embedder navigation-start override, not a new clock sample.
    pub fn SetNavigationStart(&self, start: Instant) {
        self.EnsureReferenceTimesSet();
        let wall = self.MonotonicTimeToPseudoWallMicroseconds(Some(start));
        self.reference_wall_time_us.set(Some(wall));
        self.reference_monotonic_time.set(Some(start));
        self.navigation_start.set(Some(start));
    }
    /// DocumentLoader.cc3908-3920; features.cc1934-1946.
    pub fn RemainingFontMaxBlockingTime(&self, now: Instant) -> Duration {
        let deadline = self
            .navigation_start
            .get()
            .expect("font blocking begins after navigation start")
            + Duration::from_millis(1500);
        deadline.saturating_duration_since(now)
    }
}
#[derive(Default)]
pub struct RenderBlockingElementLinkMap {
    levels: HashMap<RenderBlockingLevel, HashMap<Rc<[u16]>, HashSet<ExpectLinkOwner>>>,
}
impl RenderBlockingElementLinkMap {
    pub fn HasElement(&self, level: RenderBlockingLevel) -> bool {
        self.levels.get(&level).is_some_and(|ids| !ids.is_empty())
    }
    pub fn AddLinkWithTargetElement(
        &mut self,
        id: Rc<[u16]>,
        link: ExpectLinkOwner,
        level: RenderBlockingLevel,
    ) {
        if id.is_empty() {
            return;
        }
        self.levels
            .entry(level)
            .or_default()
            .entry(id)
            .or_default()
            .insert(link);
    }
    fn RemoveTargetElement(&mut self, id: &[u16]) -> Vec<RenderBlockingLevel> {
        if id.is_empty() {
            return Vec::new();
        }
        let escaped =
            foundation::url_escape::EncodeWithUrlEscapeSequences(&String::from_utf16_lossy(id))
                .encode_utf16()
                .collect::<Vec<_>>();
        let mut emptied = Vec::new();
        for (&level, ids) in &mut self.levels {
            if ids.is_empty() {
                continue;
            }
            ids.remove(id);
            ids.remove(escaped.as_slice());
            if ids.is_empty() {
                emptied.push(level);
            }
        }
        emptied
    }
    fn RemoveLinkWithTargetElement(
        &mut self,
        id: &[u16],
        link: ExpectLinkOwner,
    ) -> Vec<RenderBlockingLevel> {
        if id.is_empty() {
            return Vec::new();
        }
        let mut emptied = Vec::new();
        for (&level, ids) in &mut self.levels {
            let Some(links) = ids.get_mut(id) else {
                continue;
            };
            links.remove(&link);
            if links.is_empty() {
                ids.remove(id);
            }
            if ids.is_empty() {
                emptied.push(level);
            }
        }
        emptied
    }
    fn ForEach(&self, mut callback: impl FnMut(RenderBlockingLevel, ExpectLinkOwner)) {
        for (&level, ids) in &self.levels {
            for links in ids.values() {
                for &link in links {
                    callback(level, link);
                }
            }
        }
    }
    fn Clear(&mut self) -> [RenderBlockingLevel; 1] {
        self.levels.clear();
        [RenderBlockingLevel::Block]
    }
}
pub struct RenderBlockingResourceManager {
    pending_stylesheet_owner_nodes: HashSet<StylesheetOwner>,
    pending_scripts: HashMap<ScriptOwner, ScriptRenderBlockingKind>,
    pending_font_preloads: HashSet<FontPreloadOwner>,
    element_render_blocking_links: RenderBlockingElementLinkMap,
    imperative_font_loading_count: u32,
    max_blocking_timer: Option<(u64, Instant)>,
    max_fcp_timer: Option<(u64, Instant)>,
    next_timer_generation: u64,
    font_preload_timeout: Duration,
    font_preload_timer_has_fired: bool,
    effects: VecDeque<RenderBlockingEffect>,
    frame_effects: VecDeque<RenderBlockingFrameEffect>,
    imperative_font_subscriptions: VecDeque<FontFaceOwner>,
    timing: Rc<RenderBlockingLoadTiming>,
    has_expect_links: Rc<std::cell::Cell<bool>>,
    lifecycle: Rc<crate::document_lifecycle::DocumentLifecycle>,
    rendering_has_begun: Rc<std::cell::Cell<bool>>,
    document_has_body: Rc<std::cell::Cell<bool>>,
}
impl RenderBlockingResourceManager {
    pub fn new(
        timing: Rc<RenderBlockingLoadTiming>,
        has_expect_links: Rc<std::cell::Cell<bool>>,
        document_has_body: Rc<std::cell::Cell<bool>>,
        lifecycle: Rc<crate::document_lifecycle::DocumentLifecycle>,
        rendering_has_begun: Rc<std::cell::Cell<bool>>,
    ) -> Self {
        Self {
            pending_stylesheet_owner_nodes: HashSet::new(),
            pending_scripts: HashMap::new(),
            pending_font_preloads: HashSet::new(),
            element_render_blocking_links: Default::default(),
            imperative_font_loading_count: 0,
            max_blocking_timer: None,
            max_fcp_timer: None,
            next_timer_generation: 0,
            font_preload_timeout: Duration::from_millis(50),
            font_preload_timer_has_fired: false,
            effects: VecDeque::new(),
            frame_effects: VecDeque::new(),
            imperative_font_subscriptions: VecDeque::new(),
            timing,
            has_expect_links,
            lifecycle,
            rendering_has_begun,
            document_has_body,
        }
    }
    pub fn SetLoadTiming(&mut self, timing: Rc<RenderBlockingLoadTiming>) {
        self.timing = timing;
    }
    pub fn DocumentHasBody(&self) -> bool {
        self.document_has_body.get()
    }
    pub fn HasRenderBlockingResources(&self) -> bool {
        self.HasNonFontRenderBlockingResources() || self.HasRenderBlockingFonts()
    }
    pub fn HasNonFontRenderBlockingResources(&self) -> bool {
        !self.pending_stylesheet_owner_nodes.is_empty()
            || !self.pending_scripts.is_empty()
            || self
                .element_render_blocking_links
                .HasElement(RenderBlockingLevel::Block)
    }
    pub fn HasRenderBlockingFonts(&self) -> bool {
        !self.pending_font_preloads.is_empty() || self.imperative_font_loading_count != 0
    }
    pub fn DebugPendingCounts(&self) -> (usize, usize, usize, u32, bool) {
        (
            self.pending_stylesheet_owner_nodes.len(),
            self.pending_scripts.len(),
            self.pending_font_preloads.len(),
            self.imperative_font_loading_count,
            self.element_render_blocking_links
                .HasElement(RenderBlockingLevel::Block),
        )
    }
    pub fn HasPendingStylesheets(&self) -> bool {
        !self.pending_stylesheet_owner_nodes.is_empty()
    }
    pub fn AddPendingStylesheet(&mut self, owner: StylesheetOwner, has_body: bool) -> bool {
        if has_body {
            return false;
        }
        assert!(
            self.pending_stylesheet_owner_nodes.insert(owner),
            "one source stylesheet owner registration"
        );
        true
    }
    pub fn RemovePendingStylesheet(&mut self, owner: StylesheetOwner, has_body: bool) -> bool {
        if !self.pending_stylesheet_owner_nodes.remove(&owner) {
            return false;
        }
        self.RenderBlockingResourceUnblocked(has_body);
        true
    }
    pub fn AddPendingScript(
        &mut self,
        script: ScriptOwner,
        has_body: bool,
        kind: ScriptRenderBlockingKind,
    ) {
        if !has_body {
            self.pending_scripts.insert(script, kind);
        }
    }
    /// HTMLScriptElement.cc115-121: removing explicit render blocking still
    /// leaves a parser-inserted classic script potentially blocking when its
    /// current async/defer attributes are absent.
    pub fn ScriptBlockingAttributeChanged(
        &mut self,
        script: ScriptOwner,
        has_render_token: bool,
        asynchronous: bool,
        deferred: bool,
        has_body: bool,
    ) {
        let implicit = matches!(
            self.pending_scripts.get(&script),
            Some(ScriptRenderBlockingKind::ParserInsertedClassic)
        ) && !asynchronous
            && !deferred;
        if !has_render_token && !implicit {
            self.RemovePendingScript(script, has_body);
        }
    }
    pub fn RemovePendingScript(&mut self, script: ScriptOwner, has_body: bool) {
        if self.pending_scripts.remove(&script).is_some() {
            self.RenderBlockingResourceUnblocked(has_body);
        }
    }
    pub fn AddPendingFontPreload(&mut self, link: FontPreloadOwner, has_body: bool) {
        if self.font_preload_timer_has_fired || has_body {
            return;
        }
        self.pending_font_preloads.insert(link);
        self.EnsureStartFontPreloadMaxBlockingTimer();
    }
    pub fn RemovePendingFontPreload(&mut self, link: FontPreloadOwner, has_body: bool) {
        if self.pending_font_preloads.remove(&link) {
            self.RenderBlockingResourceUnblocked(has_body);
        }
    }
    pub fn AddImperativeFontLoading(
        &mut self,
        font: FontFaceOwner,
        is_loading: bool,
        has_body: bool,
    ) {
        if !is_loading || self.font_preload_timer_has_fired || has_body {
            return;
        }
        self.imperative_font_subscriptions.push_back(font);
        self.imperative_font_loading_count = self
            .imperative_font_loading_count
            .checked_add(1)
            .expect("source font loading counter");
        self.EnsureStartFontPreloadMaxBlockingTimer();
    }
    pub fn RemoveImperativeFontLoading(&mut self, has_body: bool) {
        if self.font_preload_timer_has_fired {
            return;
        }
        assert_ne!(self.imperative_font_loading_count, 0);
        self.imperative_font_loading_count -= 1;
        self.RenderBlockingResourceUnblocked(has_body);
    }
    fn StartTimer(&mut self, kind: FontTimer, delay: Duration, now: Instant) {
        self.next_timer_generation = self
            .next_timer_generation
            .checked_add(1)
            .expect("source timer generations");
        let generation = self.next_timer_generation;
        let deadline = now + delay;
        match kind {
            FontTimer::MaxBlocking => self.max_blocking_timer = Some((generation, deadline)),
            FontTimer::MaxFCPDelay => self.max_fcp_timer = Some((generation, deadline)),
        }
        self.effects
            .push_back(RenderBlockingEffect::StartFontTimer {
                kind,
                generation,
                deadline,
            });
    }
    pub fn EnsureStartFontPreloadMaxBlockingTimer(&mut self) {
        if self.font_preload_timer_has_fired || self.max_blocking_timer.is_some() {
            return;
        }
        let now = self.timing.Now();
        // features::kRenderBlockingFonts is enabled by default, features.cc1934.
        let timeout = if foundation::blink_features::RenderBlockingFontsEnabled() {
            self.timing.RemainingFontMaxBlockingTime(now)
        } else {
            self.font_preload_timeout
        };
        self.StartTimer(FontTimer::MaxBlocking, timeout, now);
    }
    pub fn EnsureStartFontPreloadMaxFCPDelayTimer(&mut self) {
        if self.font_preload_timer_has_fired || self.max_fcp_timer.is_some() {
            return;
        }
        self.StartTimer(
            FontTimer::MaxFCPDelay,
            Duration::from_millis(100),
            self.timing.Now(),
        );
    }
    pub fn FontPreloadingTimerFired(&mut self, kind: FontTimer, generation: u64) {
        let timer = match kind {
            FontTimer::MaxBlocking => &mut self.max_blocking_timer,
            FontTimer::MaxFCPDelay => &mut self.max_fcp_timer,
        };
        if timer.as_ref().map(|(current, _)| *current) != Some(generation) {
            return;
        }
        *timer = None;
        if self.font_preload_timer_has_fired {
            return;
        }
        self.effects
            .push_back(RenderBlockingEffect::ExpiredFontsHistogram {
                had_pending_fonts: self.HasRenderBlockingFonts(),
            });
        self.font_preload_timer_has_fired = true;
        self.pending_font_preloads.clear();
        self.imperative_font_loading_count = 0;
        if self.document_has_body.get() {
            self.BeginLifecycleUpdatesIfRenderingReady();
        }
    }
    pub fn NextFontTimerDeadline(&self) -> Option<Instant> {
        [self.max_blocking_timer, self.max_fcp_timer]
            .into_iter()
            .flatten()
            .map(|(_, deadline)| deadline)
            .min()
    }
    pub fn FireDueFontTimers(&mut self, now: Instant) {
        let due = [
            (FontTimer::MaxBlocking, self.max_blocking_timer),
            (FontTimer::MaxFCPDelay, self.max_fcp_timer),
        ]
        .into_iter()
        .filter_map(|(kind, timer)| {
            timer
                .filter(|(_, deadline)| *deadline <= now)
                .map(|(generation, _)| (kind, generation))
        })
        .collect::<Vec<_>>();
        for (kind, generation) in due {
            self.FontPreloadingTimerFired(kind, generation);
        }
    }
    /// Document.cc8166-8185, executed synchronously on its retained source
    /// lifecycle and rendering_has_begun_ owners, before emitting the frame effect.
    pub fn BeginLifecycleUpdatesIfRenderingReady(&mut self) {
        if !self.lifecycle.IsActive() || self.HasRenderBlockingResources() {
            return;
        }
        if !self.rendering_has_begun.replace(true) {
            self.effects
                .push_back(RenderBlockingEffect::RenderBlockingResourcesLoadedMetric);
        }
        self.frame_effects
            .push_back(RenderBlockingFrameEffect::BeginLifecycleUpdates);
    }
    pub fn WillInsertDocumentBody(&mut self) {
        if foundation::blink_features::RenderBlockingFontsEnabled()
            && !self.HasNonFontRenderBlockingResources()
            && self.HasRenderBlockingFonts()
        {
            self.EnsureStartFontPreloadMaxFCPDelayTimer();
        }
    }
    fn RenderBlockingResourceUnblocked(&mut self, has_body: bool) {
        if has_body {
            self.BeginLifecycleUpdatesIfRenderingReady();
        }
        if foundation::blink_features::RenderBlockingFontsEnabled()
            && !self.HasNonFontRenderBlockingResources()
            && self.HasRenderBlockingFonts()
            && has_body
        {
            self.EnsureStartFontPreloadMaxFCPDelayTimer();
        }
    }
    pub fn AddPendingParsingElementLink(
        &mut self,
        id: Rc<[u16]>,
        link: ExpectLinkOwner,
        level: RenderBlockingLevel,
        has_body: bool,
    ) {
        if has_body || id.is_empty() {
            return;
        }
        self.element_render_blocking_links
            .AddLinkWithTargetElement(id, link, level);
        if level == RenderBlockingLevel::Block {
            self.has_expect_links.set(true);
        }
    }
    pub fn RemovePendingParsingElement(
        &mut self,
        id: &[u16],
        in_shadow_tree: bool,
        is_connected: bool,
        has_body: bool,
    ) {
        if in_shadow_tree || !is_connected {
            return;
        }
        let levels = self.element_render_blocking_links.RemoveTargetElement(id);
        self.OnEmptyLevels(levels, has_body);
    }
    pub fn RemovePendingParsingElementLink(
        &mut self,
        id: &[u16],
        link: ExpectLinkOwner,
        has_body: bool,
    ) {
        let levels = self
            .element_render_blocking_links
            .RemoveLinkWithTargetElement(id, link);
        self.OnEmptyLevels(levels, has_body);
    }
    pub fn ClearPendingParsingElements(
        &mut self,
        has_body: bool,
        mut href_for_link: impl FnMut(ExpectLinkOwner) -> String,
    ) {
        if !self
            .element_render_blocking_links
            .HasElement(RenderBlockingLevel::Block)
        {
            return;
        }
        self.element_render_blocking_links.ForEach(|_, link| {
            self.effects
                .push_back(RenderBlockingEffect::MissingExpectedElement {
                    link,
                    href: href_for_link(link),
                })
        });
        let levels = self.element_render_blocking_links.Clear();
        self.OnEmptyLevels(levels, has_body);
    }
    fn OnEmptyLevels(
        &mut self,
        levels: impl IntoIterator<Item = RenderBlockingLevel>,
        has_body: bool,
    ) {
        for level in levels {
            if level == RenderBlockingLevel::Block {
                self.has_expect_links.set(false);
                self.RenderBlockingResourceUnblocked(has_body);
            }
        }
    }
    /// FontFace::AddCallback consumer must bind the original live FontFace;
    /// this is intentionally separate from transport/timer effects.
    pub fn TakeImperativeFontSubscriptions(&mut self) -> VecDeque<FontFaceOwner> {
        std::mem::take(&mut self.imperative_font_subscriptions)
    }
    pub fn TakeFrameEffects(&mut self) -> VecDeque<RenderBlockingFrameEffect> {
        std::mem::take(&mut self.frame_effects)
    }
    pub fn TakeEffects(&mut self) -> VecDeque<RenderBlockingEffect> {
        std::mem::take(&mut self.effects)
    }
}

/// LinkStyle/StyleElement pending-sheet lifetime. All import/parse stages share
/// this registration, including cancellation and failure; it is not keyed by URL.
pub struct StylesheetRenderBlockingRegistration {
    manager: std::rc::Weak<std::cell::RefCell<RenderBlockingResourceManager>>,
    owner: StylesheetOwner,
}
impl StylesheetRenderBlockingRegistration {
    pub fn Register(
        manager: &Rc<std::cell::RefCell<RenderBlockingResourceManager>>,
        owner: StylesheetOwner,
    ) -> Option<Rc<Self>> {
        let body = manager.borrow().DocumentHasBody();
        if !manager.borrow_mut().AddPendingStylesheet(owner, body) {
            return None;
        }
        Some(Rc::new(Self {
            manager: Rc::downgrade(manager),
            owner,
        }))
    }
}
impl Drop for StylesheetRenderBlockingRegistration {
    fn drop(&mut self) {
        if let Some(manager) = self.manager.upgrade() {
            let body = manager.borrow().DocumentHasBody();
            manager
                .borrow_mut()
                .RemovePendingStylesheet(self.owner, body);
        }
    }
}

/// BlockingAttribute.cc25-28: token matching is case sensitive and uses HTML
/// space characters (vertical tab is not an HTML space).
pub fn HasRenderBlockingToken(value: &str) -> bool {
    value
        .split(|c| matches!(c, ' ' | '\t' | '\n' | '\r' | '\u{000c}'))
        .any(|token| token == "render")
}

/// PendingScript.cc223-231. The pending script owns this registration until
/// immediately before ExecuteScriptBlock (including its error branch).
pub struct ScriptRenderBlockingRegistration {
    manager: std::rc::Weak<std::cell::RefCell<RenderBlockingResourceManager>>,
    owner: ScriptOwner,
}
impl ScriptRenderBlockingRegistration {
    pub fn Register(
        manager: &Rc<std::cell::RefCell<RenderBlockingResourceManager>>,
        owner: ScriptOwner,
        kind: ScriptRenderBlockingKind,
    ) -> Option<Rc<Self>> {
        let body = manager.borrow().DocumentHasBody();
        if body {
            return None;
        }
        manager.borrow_mut().AddPendingScript(owner, body, kind);
        Some(Rc::new(Self {
            manager: Rc::downgrade(manager),
            owner,
        }))
    }
}
impl Drop for ScriptRenderBlockingRegistration {
    fn drop(&mut self) {
        if let Some(manager) = self.manager.upgrade() {
            let body = manager.borrow().DocumentHasBody();
            manager.borrow_mut().RemovePendingScript(self.owner, body);
        }
    }
}
