//! HTMLMediaElement's single DOM state owner. Source h:845-950; paused/muted
//! read/mutations cc:2915-2930,3434-3458 at the fixed Chromium revision.
use super::*;
use foundation::media_element::*;
use std::cell::Cell;

#[derive(Clone, Copy, Debug)]
pub struct DOMMediaPlayerLayoutFacts {
    pub natural_size: foundation::gfx::Size,
    pub has_available_video_frame: bool,
    pub has_compositor_layer: bool,
    pub would_taint_origin: bool,
}
/// HTMLMediaElement retains this alias to its actual platform player. The
/// reader contains no Page, Layout or copied playback state.
pub trait DOMMediaPlayerLayoutSource {
    fn LayoutFacts(&self) -> DOMMediaPlayerLayoutFacts;
}
/// The optional VideoVisibilityTracker is installed only by the original
/// Auto-PiP/CDM creation lifecycle; construction leaves it absent.
pub trait DOMVideoVisibilityTracker {
    fn UpdateVisibilityTrackerState(&self);
}
pub trait DOMMediaStreamDescriptor {}
pub trait DOMMediaSourceHandle {
    fn InternalBlobURL(&self) -> Rc<url::Url>;
    fn TakeAttachment(&self) -> Option<Rc<dyn DOMMediaSourceAttachment>>;
}
/// MediaSourceAttachment's existing identity, distinct from srcObject's handle.
pub trait DOMMediaSourceAttachment {
    fn OnElementTimeUpdated(&self, current_time: f64);
    fn Seekable(&self) -> foundation::web_time_ranges::WebTimeRanges;
    fn StartAttachingToMediaElement(&self, element: DOMOwnerHandle) -> bool;
    fn Close(&self);
    fn OnElementError(&self);
}
#[derive(Clone)]
pub enum DOMMediaSrcObject {
    Stream(Option<Rc<dyn DOMMediaStreamDescriptor>>),
    Source(Option<Rc<dyn DOMMediaSourceHandle>>),
}
pub struct HTMLMediaElementState {
    pub video_timing: RefCell<Option<Rc<crate::video_timing::VideoTiming>>>,
    pub has_received_first_frame: Cell<bool>,
    pub player_layout: RefCell<Option<Rc<dyn DOMMediaPlayerLayoutSource>>>,
    pub default_poster_url: Rc<str>,
    pub initial_src_attribute: Cell<bool>,
    pub initial_poster_attribute: Cell<bool>,
    pub poster_deferred_for_lazy_load: Cell<bool>,
    pub video_visibility_tracker: RefCell<Option<Rc<dyn DOMVideoVisibilityTracker>>>,
    pub autoplay: Rc<MediaAutoplayState>,
    pub autoplay_visibility_observer: RefCell<Option<Rc<dyn MediaAutoplayVisibilityObserver>>>,
    pub play_promises: Rc<MediaPlayPromiseState>,
    pub current_src_after_redirects: RefCell<Option<Rc<url::Url>>>,
    pub media_source_attachment: RefCell<Option<Rc<dyn DOMMediaSourceAttachment>>>,
    pub current_src_internal: RefCell<Option<Rc<url::Url>>>,
    pub load_state: Cell<MediaLoadState>,
    pub deferred_load_state: Cell<MediaDeferredLoadState>,
    pub lazy_load_state: Cell<MediaLazyLoadState>,
    pub current_source_node: Cell<Option<u64>>,
    pub next_child_node_to_consider: Cell<Option<u64>>,
    pub player_lazy_load_observer: RefCell<Option<Rc<dyn MediaAutoplayVisibilityObserver>>>,
    pub current_src: RefCell<Option<Rc<url::Url>>>,
    pub src_object: RefCell<DOMMediaSrcObject>,
    pub network: Cell<MediaNetworkState>,
    pub pending_media_events: Cell<u32>,
    pub media_event_generation: Cell<u64>,
    pub next_media_event_id: Cell<u64>,
    pub media_event_tasks: RefCell<std::collections::HashMap<u64, MediaElementTaskToken>>,
    pub should_delay_load_event: Cell<bool>,
    pub load_event_delay_document: Cell<Option<DOMOwnerHandle>>,
    pub ready: Cell<MediaReadyState>,
    pub ready_maximum: Cell<MediaReadyState>,
    pub network_maximum: Cell<MediaNetworkState>,
    pub error_message: RefCell<String>,
    pub error: Cell<Option<MediaErrorCode>>,
    pub is_encrypted: Cell<bool>,
    pub is_auto_picture_in_picture: Cell<bool>,
    pub can_autoplay: Cell<bool>,
    pub muted_is_default: Cell<bool>,
    pub muted_default_state_enabled: Cell<bool>,
    pub seeking: Cell<bool>,
    pub sent_stalled_event: Cell<bool>,
    pub muted: Cell<bool>,
    pub paused: Cell<bool>,
    pub playing: Cell<bool>,
    pub playback_rate: Cell<f64>,
    pub default_playback_rate: Cell<f64>,
    pub have_fired_loaded_data: Cell<bool>,
    pub finished_parsing_children: Cell<bool>,
    pub last_seek_time: Cell<f64>,
    pub last_timeupdate_event_media_time: Cell<f64>,
    pub official_playback_position: Cell<f64>,
    pub official_playback_position_needs_update: Cell<bool>,
    pub default_playback_start_position: Cell<f64>,
    pub fragment_end_time: Cell<f64>,
    pub played_time_ranges: RefCell<foundation::web_time_ranges::WebTimeRanges>,
    pub show_poster: Cell<bool>,
    pub was_always_muted: Cell<bool>,
    pub ignore_preload_none: Cell<bool>,
    pub volume: Cell<f64>,
    pub duration: Cell<f64>,
    pub user_wants_controls_visible: Cell<Option<bool>>,
    pub previous_progress_time_micros: Cell<Option<i64>>,
    pub video_has_played: Cell<bool>,
}
impl HTMLMediaElementState {
    fn new(parser_muted_attribute: bool, src: bool, poster: bool) -> Self {
        Self {
            video_timing: RefCell::new(None),
            has_received_first_frame: Cell::new(false),
            // settings.json5:1012-1015. This document's media construction
            // profile currently admits the source NoUserGestureRequired setting.
            player_layout: RefCell::new(None),
            // HTMLVideoElement constructor cc:138-141 reads Settings' initial
            // defaultVideoPosterURL (settings.json5:422), an empty AtomicString.
            default_poster_url: Rc::from(""),
            initial_src_attribute: Cell::new(src),
            initial_poster_attribute: Cell::new(poster),
            poster_deferred_for_lazy_load: Cell::new(false),
            video_visibility_tracker: RefCell::new(None),
            autoplay_visibility_observer: RefCell::new(None),
            autoplay: Rc::new(MediaAutoplayState::new(
                MediaAutoplayPolicyType::NoUserGestureRequired,
            )),
            play_promises: Rc::new(MediaPlayPromiseState::default()),
            current_src_after_redirects: RefCell::new(None),
            media_source_attachment: RefCell::new(None),
            current_src_internal: RefCell::new(None),
            load_state: Cell::new(MediaLoadState::WaitingForSource),
            deferred_load_state: Cell::new(MediaDeferredLoadState::NotDeferred),
            lazy_load_state: Cell::new(MediaLazyLoadState::None),
            current_source_node: Cell::new(None),
            next_child_node_to_consider: Cell::new(None),
            player_lazy_load_observer: RefCell::new(None),
            current_src: RefCell::new(None),
            src_object: RefCell::new(DOMMediaSrcObject::Stream(None)),
            network: Cell::new(MediaNetworkState::Empty),
            pending_media_events: Cell::new(0),
            media_event_generation: Cell::new(0),
            next_media_event_id: Cell::new(0),
            media_event_tasks: RefCell::new(std::collections::HashMap::new()),
            should_delay_load_event: Cell::new(false),
            load_event_delay_document: Cell::new(None),
            ready: Cell::new(MediaReadyState::HaveNothing),
            ready_maximum: Cell::new(MediaReadyState::HaveNothing),
            network_maximum: Cell::new(MediaNetworkState::Empty),
            error_message: RefCell::new(String::new()),
            error: Cell::new(None),
            is_encrypted: Cell::new(false),
            is_auto_picture_in_picture: Cell::new(false),
            can_autoplay: Cell::new(true),
            muted_is_default: Cell::new(true),
            muted_default_state_enabled: Cell::new(false),
            seeking: Cell::new(false),
            sent_stalled_event: Cell::new(false),
            muted: Cell::new(parser_muted_attribute),
            paused: Cell::new(true),
            playing: Cell::new(false),
            playback_rate: Cell::new(1.),
            default_playback_rate: Cell::new(1.),
            have_fired_loaded_data: Cell::new(false),
            finished_parsing_children: Cell::new(true),
            last_seek_time: Cell::new(0.),
            last_timeupdate_event_media_time: Cell::new(f64::NAN),
            official_playback_position: Cell::new(0.),
            official_playback_position_needs_update: Cell::new(true),
            default_playback_start_position: Cell::new(0.),
            fragment_end_time: Cell::new(f64::NAN),
            played_time_ranges: RefCell::new(Default::default()),
            show_poster: Cell::new(true),
            was_always_muted: Cell::new(true),
            ignore_preload_none: Cell::new(false),
            volume: Cell::new(1.),
            duration: Cell::new(f64::NAN),
            user_wants_controls_visible: Cell::new(None),
            previous_progress_time_micros: Cell::new(None),
            video_has_played: Cell::new(false),
        }
    }
    pub fn Muted(&self, default_state_enabled: bool, muted_attribute: bool) -> bool {
        if default_state_enabled && self.muted_is_default.get() {
            muted_attribute
        } else {
            self.muted.get()
        }
    }
}
impl PersistentDocument {
    /// The independent Page attaches its LocalFrame only to this root
    /// Document. Script-created secondary Documents have no active frame.
    pub fn IsNodeOwnerDocumentActive(&self, node: usize) -> bool {
        self.IsActive()
            && (self.Node(node).OwnerDocumentNode() == Some(self.RootHandle())
                || (self.Node(node).Type() == DOMNodeType::kDocument
                    && self.NodeHandle(node) == self.RootHandle()))
    }
    /// HTMLMediaElement::SetNetworkState, cc:4884-4897, and stalled pseudo
    /// transitions cc:2572-2589. Mutations retain the same element state owner.
    pub fn SetMediaNetworkState(&mut self, node: usize, state: MediaNetworkState) {
        let source = self.MediaElementState(node);
        if source.network.get() == state {
            return;
        }
        let was = self.MediaMatchesBufferingPseudo(node);
        source.network.set(state);
        if was != self.MediaMatchesBufferingPseudo(node) {
            for pseudo in [
                crate::style_state::ControlPseudoState::Buffering,
                crate::style_state::ControlPseudoState::Stalled,
            ] {
                self.RecordStyleChange(crate::style_state::StyleChange::ControlPseudo(
                    node, pseudo,
                ));
            }
        }
        let rank = |state| match state {
            MediaNetworkState::Empty => 0,
            MediaNetworkState::Idle => 1,
            MediaNetworkState::Loading => 2,
            MediaNetworkState::NoSource => 3,
        };
        if rank(state) > rank(source.network_maximum.get()) {
            source.network_maximum.set(state);
        }
    }
    pub fn SetMediaSentStalledEvent(&mut self, node: usize, stalled: bool) {
        if self
            .MediaElementState(node)
            .sent_stalled_event
            .replace(stalled)
            != stalled
        {
            self.RecordStyleChange(crate::style_state::StyleChange::ControlPseudo(
                node,
                crate::style_state::ControlPseudoState::Stalled,
            ));
        }
    }

    pub fn SetMediaSeeking(&mut self, node: usize, value: bool) {
        if self.MediaElementState(node).seeking.replace(value) != value {
            self.RecordStyleChange(crate::style_state::StyleChange::ControlPseudo(
                node,
                crate::style_state::ControlPseudoState::Seeking,
            ));
        }
    }

    /// The existing allocation registry, not a DOM scan. Once modules admit
    /// their UA loader, CSSDefaultStyleSheets lazily selects it for media.
    pub fn HasMediaElementsForStyle(&self) -> bool {
        !self.media_elements.borrow().is_empty()
    }
    pub fn MediaElementState(&self, node: usize) -> Rc<HTMLMediaElementState> {
        assert!(self.IsMediaElement(node));
        self.media_elements
            .borrow()
            .get(&self.Node(node).Id())
            .expect("media constructor's canonical state")
            .clone()
    }
    pub(super) fn ConstructMediaElementState(&self, node: usize) {
        // Allocate's initial attributes are parser/clone construction input;
        // later setAttribute(muted) never changes the non-default IDL state.
        self.media_elements.borrow_mut().insert(
            self.Node(node).Id(),
            Rc::new(HTMLMediaElementState::new(
                self.Node(node).FindAttribute("muted").is_some(),
                self.Node(node).FindAttribute("src").is_some(),
                self.Node(node).FindAttribute("poster").is_some(),
            )),
        );
    }
    pub fn SetMediaPaused(&mut self, node: usize, paused: bool) {
        let state = self.MediaElementState(node);
        if state.paused.replace(paused) == paused {
            return;
        }
        for pseudo in [
            crate::style_state::ControlPseudoState::Paused,
            crate::style_state::ControlPseudoState::Playing,
            crate::style_state::ControlPseudoState::Buffering,
            crate::style_state::ControlPseudoState::Stalled,
        ] {
            self.RecordStyleChange(crate::style_state::StyleChange::ControlPseudo(node, pseudo));
        }
    }
    pub fn MediaMutedPseudoChanged(&mut self, node: usize) {
        self.RecordStyleChange(crate::style_state::StyleChange::ControlPseudo(
            node,
            crate::style_state::ControlPseudoState::Muted,
        ));
    }
    pub fn SetMediaShowPoster(&self, node: usize, value: bool) -> bool {
        self.MediaElementState(node).show_poster.replace(value) != value
    }
    pub fn MediaMuted(&self, node: usize) -> bool {
        let state = self.MediaElementState(node);
        state.Muted(
            state.muted_default_state_enabled.get(),
            self.Node(node).FindAttribute("muted").is_some(),
        )
    }
    pub fn MediaMatchesBufferingPseudo(&self, node: usize) -> bool {
        let state = self.MediaElementState(node);
        !state.paused.get()
            && state.network.get() == MediaNetworkState::Loading
            && state.ready.get() <= MediaReadyState::HaveCurrentData
    }
    pub fn MediaMatchesStalledPseudo(&self, node: usize) -> bool {
        self.MediaMatchesBufferingPseudo(node)
            && self.MediaElementState(node).sent_stalled_event.get()
    }
}

/// Source HTMLMediaElement::IsHLSURL, cc:406-415. Query the original completed
/// URL's parsed components; no DOM URL is reparsed by context-menu assembly.
pub fn MediaDownloadFactsForURL(
    url: &url::Url,
) -> foundation::media_element::MediaDownloadURLFacts {
    let is_local_file = url.scheme() == "file";
    foundation::media_element::MediaDownloadURLFacts {
        is_local_file,
        is_content_scheme: url.scheme() == "content",
        is_hls: (is_local_file || matches!(url.scheme(), "http" | "https"))
            && url.path().ends_with(".m3u8"),
    }
}

/// PictureInPictureController is a Document supplement. These identities are
/// set by the successful Enter/Exit lifecycle, not queried from attributes.
/// Source core/frame/picture_in_picture_controller.cc:20-46;
/// modules/document_picture_in_picture/picture_in_picture_controller_impl.h.
#[derive(Default)]
pub struct DOMPictureInPictureState {
    pub element: Cell<Option<DOMOwnerHandle>>,
    pub is_document_picture_in_picture_window: Cell<bool>,
}
impl PersistentDocument {
    pub fn PictureInPictureState(&self) -> Rc<DOMPictureInPictureState> {
        self.picture_in_picture_state
            .get_or_init(|| Rc::new(DOMPictureInPictureState::default()))
            .clone()
    }
}

impl PersistentDocument {
    /// Document.h:1659,3066; Document.cc:8856-8882. Each native Document node
    /// owns an initially-zero counter, including secondary documents in arena.
    pub fn LoadEventDelayCount(&self) -> u32 {
        self.LoadEventDelayCountForDocument(self.RootHandle())
    }
    pub fn LoadEventDelayCounterForDocument(
        &self,
        document: DOMOwnerHandle,
    ) -> Rc<crate::load_event_delay_count::LoadEventDelayCounter> {
        let index = self
            .FindNodeById(document.Id())
            .expect("load delay document");
        assert_eq!(self.NodeHandle(index), document);
        assert_eq!(self.Node(index).node_type, DOMNodeType::kDocument);
        self.load_event_delay_counts
            .borrow_mut()
            .entry(document)
            .or_default()
            .clone()
    }
    pub fn LoadEventDelayCountForDocument(&self, document: DOMOwnerHandle) -> u32 {
        self.load_event_delay_counts
            .borrow()
            .get(&document)
            .map_or(0, |count| count.Count())
    }
    pub fn IncrementLoadEventDelayCountForDocument(&self, document: DOMOwnerHandle) {
        self.LoadEventDelayCounterForDocument(document).Increment();
    }
    pub fn DecrementLoadEventDelayCountForDocument(&self, document: DOMOwnerHandle) -> bool {
        self.LoadEventDelayCounterForDocument(document).Decrement()
    }
    /// HTMLMediaElement.cc:4666-4689. Some(document) is the exact deferred
    /// decrement to post to kMediaElementEvent *after* currently queued media
    /// events. It is not a deferred change to the element's source flag.
    pub fn SetMediaShouldDelayLoadEvent(
        &self,
        node: usize,
        should_delay: bool,
    ) -> Option<DOMOwnerHandle> {
        let source = self.MediaElementState(node);
        if source.should_delay_load_event.replace(should_delay) == should_delay {
            return None;
        }
        if should_delay {
            let document = self
                .Node(node)
                .OwnerDocumentNode()
                .expect("media owning Document");
            source.load_event_delay_document.set(Some(document));
            self.IncrementLoadEventDelayCountForDocument(document);
            None
        } else {
            let document = source
                .load_event_delay_document
                .take()
                .expect("media balanced load delay document");
            if source.pending_media_events.get() != 0 {
                Some(document)
            } else {
                self.DecrementLoadEventDelayCountForDocument(document);
                None
            }
        }
    }
}

impl PersistentDocument {
    /// HTMLVideoElement::ImageSourceURL, cc:391-397. Parsing/creation selects
    /// the video dynamic class once; read queries never infer it from a tag.
    pub fn VideoImageSourceURL(&self, node: usize) -> String {
        assert!(self.IsVideoElement(node));
        let poster = self
            .Node(node)
            .FindAttribute("poster")
            .map(|attr| attr.value.as_str())
            .unwrap_or("");
        if !poster
            .trim_matches(|c| matches!(c, ' ' | '\t' | '\n' | '\r' | '\u{000c}'))
            .is_empty()
        {
            poster.to_owned()
        } else {
            self.MediaElementState(node).default_poster_url.to_string()
        }
    }
    pub fn VideoPosterContent(&self, node: usize) -> Option<Rc<DOMImageResourceContent>> {
        assert!(self.IsVideoElement(node));
        self.ExistingHTMLImageLoader(node)
            .and_then(|loader| loader.GetContent())
    }
    pub fn VideoLayoutStateForElement(
        &self,
        node: usize,
    ) -> Option<layoutng_assembly::internal::layout_input::VideoLayoutState> {
        if !self.IsVideoElement(node) {
            return None;
        }
        let source = self.MediaElementState(node);
        let player = source
            .player_layout
            .borrow()
            .clone()
            .map(|player| player.LayoutFacts());
        let image_source = self.VideoImageSourceURL(node);
        Some(
            layoutng_assembly::internal::layout_input::VideoLayoutState {
                show_poster: source.show_poster.get(),
                has_available_video_frame: player
                    .is_some_and(|player| player.has_available_video_frame),
                poster_url_present: !image_source
                    .trim_matches(|c| matches!(c, ' ' | '\t' | '\n' | '\r' | '\u{000c}'))
                    .is_empty(),
                is_default_poster: image_source.as_str() == source.default_poster_url.as_ref(),
                player_natural_size: player.map(|player| player.natural_size),
                in_active_document: self.IsNodeOwnerDocumentActive(node)
                    && self.IsConnectedToDocument(node),
                // The current ordinary Document constructor is not MediaDocument.
                // That owning subclass must supply its identity when admitted.
                is_media_document: false,
                has_compositor_layer: player.is_some_and(|player| player.has_compositor_layer),
                would_taint_origin: player.is_some_and(|player| player.would_taint_origin),
                has_controls: self.Node(node).FindAttribute("controls").is_some(),
                is_fullscreen: self.FullscreenFlag(node),
            },
        )
    }
}

/// The source HTMLVideoElement action, computed by its owning DOM class before
/// the embedding releases its document borrow. No URL/resource cache is copied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VideoPosterUpdate {
    None,
    EnqueueMicrotask,
    UpdateImageLoader,
    StartLazyMonitoring,
}
impl PersistentDocument {
    fn VideoHasLazyLoadingAttribute(&self, node: usize) -> bool {
        self.Node(node)
            .FindAttribute("loading")
            .is_some_and(|attr| attr.value.eq_ignore_ascii_case("lazy"))
    }
    pub fn MediaHasSources(&self, node: usize) -> bool {
        let state = self.MediaElementState(node);
        let has_object = match &*state.src_object.borrow() {
            DOMMediaSrcObject::Stream(value) => value.is_some(),
            DOMMediaSrcObject::Source(value) => value.is_some(),
        };
        has_object
            || self.Node(node).FindAttribute("src").is_some()
            || self
                .Node(node)
                .Children()
                .iter()
                .any(|child| self.IsHTMLSourceElement(*child))
    }
    /// HTMLVideoElement::UpdatePosterImage, html_video_element.cc:257-280.
    pub fn VideoUpdatePosterImage(&self, node: usize) -> VideoPosterUpdate {
        assert!(self.IsVideoElement(node));
        let source = self.MediaElementState(node);
        if foundation::RuntimeEnabledFeatures::LazyLoadVideoAndAudioEnabled() {
            if source.poster_deferred_for_lazy_load.get() {
                return VideoPosterUpdate::None;
            }
            if source.lazy_load_state.get() == MediaLazyLoadState::Deferred
                || self.VideoHasLazyLoadingAttribute(node)
            {
                source.poster_deferred_for_lazy_load.set(true);
                return VideoPosterUpdate::None;
            }
            if self.ExistingHTMLImageLoader(node).is_none() {
                return VideoPosterUpdate::EnqueueMicrotask;
            }
        }
        self.EnsureHTMLImageLoader(node);
        VideoPosterUpdate::UpdateImageLoader
    }
    /// The original Agent microtask rechecks all attributes, cc:282-302.
    pub fn VideoUpdatePosterImageInternal(&self, node: usize) -> VideoPosterUpdate {
        assert!(self.IsVideoElement(node));
        let source = self.MediaElementState(node);
        if foundation::RuntimeEnabledFeatures::LazyLoadVideoAndAudioEnabled()
            && (source.poster_deferred_for_lazy_load.get()
                || source.lazy_load_state.get() == MediaLazyLoadState::Deferred
                || self.VideoHasLazyLoadingAttribute(node))
        {
            source.poster_deferred_for_lazy_load.set(true);
            if !self.MediaHasSources(node) && self.IsNodeOwnerDocumentActive(node) {
                source.lazy_load_state.set(MediaLazyLoadState::Deferred);
                return VideoPosterUpdate::StartLazyMonitoring;
            }
            return VideoPosterUpdate::None;
        }
        self.EnsureHTMLImageLoader(node);
        VideoPosterUpdate::UpdateImageLoader
    }
    /// HTMLVideoElement::OnLazyLoadResumed, cc:303-313. Called only by the
    /// actual media lazy-loading observer's original resumption lifecycle.
    pub fn VideoPosterOnLazyLoadResumed(&self, node: usize) -> VideoPosterUpdate {
        assert!(self.IsVideoElement(node));
        if self
            .MediaElementState(node)
            .poster_deferred_for_lazy_load
            .replace(false)
        {
            self.EnsureHTMLImageLoader(node);
            VideoPosterUpdate::UpdateImageLoader
        } else {
            VideoPosterUpdate::None
        }
    }
}

impl PersistentDocument {
    /// HTMLVideoElement::InsertedInto, cc:217-228. Only this existing
    /// insertion lifecycle may begin observing a deferred poster-only video.
    pub fn VideoPosterInsertedInto(&self, node: usize) -> VideoPosterUpdate {
        assert!(self.IsVideoElement(node));
        let source = self.MediaElementState(node);
        if foundation::RuntimeEnabledFeatures::LazyLoadVideoAndAudioEnabled()
            && source.poster_deferred_for_lazy_load.get()
            && source.lazy_load_state.get() == MediaLazyLoadState::None
            && !self.MediaHasSources(node)
            && self.IsNodeOwnerDocumentActive(node)
        {
            source.lazy_load_state.set(MediaLazyLoadState::Deferred);
            VideoPosterUpdate::StartLazyMonitoring
        } else {
            VideoPosterUpdate::None
        }
    }
}

impl PersistentDocument {
    pub fn RegisterMediaSourceAttachment(
        &self,
        url: String,
        attachment: Rc<dyn DOMMediaSourceAttachment>,
    ) {
        self.media_source_attachments
            .borrow_mut()
            .insert(url, attachment);
    }
    pub fn RevokeMediaSourceAttachment(&self, url: &str) {
        self.media_source_attachments.borrow_mut().remove(url);
    }
    pub fn LookupMediaSourceAttachment(
        &self,
        url: &str,
    ) -> Option<Rc<dyn DOMMediaSourceAttachment>> {
        self.media_source_attachments.borrow().get(url).cloned()
    }
}

impl PersistentDocument {
    /// OnFirstFrame cc:770-798. The caller supplies the VideoFrameCompositor's
    /// actual first-frame callback and PipelineStatistics::video_bytes_decoded,
    /// after testing the original LayoutObject border-box properties. No size
    /// is inferred from the response buffer or decoded frame dimensions.
    pub fn RecordVideoFirstFrameTiming(
        &self,
        node: usize,
        frame_time: std::time::Instant,
        bytes_to_first_frame: u64,
    ) -> Rc<crate::video_timing::VideoTiming> {
        assert!(self.arena[node].IsHTMLElement("video"));
        let state = self.MediaElementState(node);
        state.has_received_first_frame.set(true);
        let url = state
            .current_src
            .borrow()
            .as_ref()
            .map_or_else(String::new, |url| url.to_string());
        let timing = Rc::new(crate::video_timing::VideoTiming::new(url));
        timing.SetFirstVideoFrameTime(frame_time);
        timing.SetIsSufficientContentLoadedForPaint();
        timing.SetContentSizeForEntropy(bytes_to_first_frame);
        // html_video_element.cc:117: kKeepVideoTimingAlive enabled by default.
        *state.video_timing.borrow_mut() = Some(timing.clone());
        timing
    }
    pub fn VideoTimingFor(&self, node: usize) -> Option<Rc<crate::video_timing::VideoTiming>> {
        let state = self.MediaElementState(node);
        let timing = state.video_timing.borrow().clone();
        timing
    }
}
