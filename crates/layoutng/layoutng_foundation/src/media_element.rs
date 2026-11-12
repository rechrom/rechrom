//! Platform-neutral HTMLMediaElement state/effect vocabulary; Chromium revision
//! 6c1d401fcca5e1b0030563a90c2f2bba168e0c15 html_media_element.h.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaNetworkState {
    Empty,
    Idle,
    Loading,
    NoSource,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MediaReadyState {
    HaveNothing,
    HaveMetadata,
    HaveCurrentData,
    HaveFutureData,
    HaveEnoughData,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaErrorCode {
    Aborted,
    Network,
    Decode,
    SourceNotSupported,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaPlayError {
    NotAllowed,
    NotSupported,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaPauseReason {
    PauseCalled,
    EndOfPlayback,
    RemovedFromDocument,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaEvent {
    Play,
    Waiting,
    Pause,
    VolumeChange,
    Playing,
    Timeupdate,
    Error,
    Seeking,
    Seeked,
    Loadstart,
    Suspend,
    Abort,
    Emptied,
    Progress,
    Stalled,
    Ratechange,
}
#[derive(Clone, Copy, Debug)]
pub struct MediaDownloadURLFacts {
    pub is_local_file: bool,
    pub is_content_scheme: bool,
    pub is_hls: bool,
}

impl MediaEvent {
    pub fn Name(self) -> &'static str {
        match self {
            Self::Play => "play",
            Self::Waiting => "waiting",
            Self::Pause => "pause",
            Self::VolumeChange => "volumechange",
            Self::Playing => "playing",
            Self::Timeupdate => "timeupdate",
            Self::Error => "error",
            Self::Seeking => "seeking",
            Self::Seeked => "seeked",
            Self::Loadstart => "loadstart",
            Self::Suspend => "suspend",
            Self::Abort => "abort",
            Self::Emptied => "emptied",
            Self::Progress => "progress",
            Self::Stalled => "stalled",
            Self::Ratechange => "ratechange",
        }
    }
}
/// Script-free ownership of HTMLMediaElement's original promise resolver lists.
/// WebAPI supplies a resolver identity; command-only play() adds no resolver.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaPlayPromiseError {
    NotSupported,
    Interrupted(MediaPauseReason),
    NewLoadRequest,
}
pub trait MediaPlayPromiseResolver {
    fn Resolve(&self);
    fn Reject(&self, error: MediaPlayPromiseError);
}
/// Opaque token in the element's existing kMediaElementEvent task runner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MediaElementTaskToken(pub u64);
pub struct MediaPlayPromiseState {
    pub pending: std::cell::RefCell<Vec<std::rc::Rc<dyn MediaPlayPromiseResolver>>>,
    pub resolving: std::cell::RefCell<Vec<std::rc::Rc<dyn MediaPlayPromiseResolver>>>,
    pub rejecting: std::cell::RefCell<Vec<std::rc::Rc<dyn MediaPlayPromiseResolver>>>,
    pub resolve_task: std::cell::Cell<Option<MediaElementTaskToken>>,
    pub reject_task: std::cell::Cell<Option<MediaElementTaskToken>>,
    pub resolve_task_active: std::cell::Cell<bool>,
    pub reject_task_active: std::cell::Cell<bool>,
    pub reject_error: std::cell::Cell<MediaPlayPromiseError>,
}
impl Default for MediaPlayPromiseState {
    fn default() -> Self {
        Self {
            pending: Default::default(),
            resolving: Default::default(),
            rejecting: Default::default(),
            resolve_task: std::cell::Cell::new(None),
            reject_task: std::cell::Cell::new(None),
            resolve_task_active: std::cell::Cell::new(false),
            reject_task_active: std::cell::Cell::new(false),
            reject_error: std::cell::Cell::new(MediaPlayPromiseError::NotSupported),
        }
    }
}

/// autoplay_policy.h:31-35; settings.json5:1012-1015.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaAutoplayPolicyType {
    NoUserGestureRequired,
    UserGestureRequired,
    DocumentUserActivationRequired,
}
pub struct MediaAutoplayState {
    pub locked_pending_user_gesture: std::cell::Cell<bool>,
    pub initiated: std::cell::Cell<Option<bool>>,
}
impl MediaAutoplayState {
    pub fn new(policy: MediaAutoplayPolicyType) -> Self {
        Self {
            locked_pending_user_gesture: std::cell::Cell::new(
                policy == MediaAutoplayPolicyType::UserGestureRequired,
            ),
            initiated: std::cell::Cell::new(None),
        }
    }
}

/// Original autoplay IntersectionObserver identity. Its producer is attribute
/// autoplay; play()/pause()/unmute only disconnect an existing observer.
pub trait MediaAutoplayVisibilityObserver {
    fn Disconnect(&self);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaLoadState {
    WaitingForSource,
    LoadingFromSrcAttribute,
    LoadingFromSrcObject,
    LoadingFromSourceElement,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaDeferredLoadState {
    NotDeferred,
    WaitingForStopDelayingLoadEventTask,
    WaitingForTrigger,
    ExecuteOnStopDelayingLoadEventTask,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaLazyLoadState {
    None,
    Deferred,
    FullMedia,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaPreload {
    None,
    Metadata,
    Auto,
}

/// text_track_kind_user_preference.h:12-16; settings.json5:870-873.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextTrackKindUserPreference {
    Default,
    Subtitles,
    Captions,
}
