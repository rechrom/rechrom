//! The allocation/identity part of media UA DOM subclasses. This does not
//! create widgets or admit their observer/timer/control lifecycle. Those source
//! constructors must call these allocators when their owning path is translated.
//! Chromium: MediaControlsImpl cc:391-392,460-465; TextTrackContainer cc:70-74;
//! MediaRemotingInterstitial cc:29-37; PictureInPictureInterstitial cc:60-71.
use super::{editing_node::DOMNodeClass, DOMNamespace, DOMOwnerHandle, PersistentDocument};
pub use layoutng_assembly::internal::layout_input::MediaChildKind;

/// Alias of the canonical ControlsImpl's own source state. DOM never builds a
/// second controls state machine to freeze an event path.
pub trait MediaControlsEventState: std::fmt::Debug {
    fn ShouldShowAudioControls(&self) -> bool;
}
#[derive(Clone, Debug)]
pub(super) struct MediaControls {
    pub media: DOMOwnerHandle,
    event_state: std::rc::Rc<std::cell::OnceCell<std::rc::Rc<dyn MediaControlsEventState>>>,
}
#[derive(Clone, Debug)]
pub(super) struct TextTrackContainer {
    pub media: DOMOwnerHandle,
}
#[derive(Clone, Debug)]
pub(super) struct MediaRemotingInterstitial {
    pub video: DOMOwnerHandle,
}
#[derive(Clone, Debug)]
pub(super) struct PictureInPictureInterstitial {
    pub video: DOMOwnerHandle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaControlInputClass {
    PlayButton,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaControlDivClass {
    PanelEnclosure,
    OverlayEnclosure,
    ButtonPanel,
    VolumeContainer,
    CurrentTime,
    RemainingTime,
}
#[derive(Clone, Debug)]
pub(super) struct MediaControlPanel {
    pub controls: DOMOwnerHandle,
}

impl PersistentDocument {
    pub fn SetMediaControlShadowPseudoId(&mut self, media: usize, child: usize, pseudo: &str) {
        // ShadowPseudoId belongs to the child. Remoting and PiP intentionally
        // share a pseudo name and must not overwrite a host->single-child map.
        self.shadow_pseudo_ids.insert(child, pseudo.into());
        self.ultimate_originating_elements.insert(child, media);
    }

    /// Native class identity selected at allocation, independent of tag,
    /// pseudo name, attachment state, or the host's current attributes.
    pub fn MediaChildKindForLayout(&self, child: usize) -> Option<MediaChildKind> {
        Some(match &self.arena[child].class {
            DOMNodeClass::MediaControls(_) => MediaChildKind::Controls,
            DOMNodeClass::TextTrackContainer(_) => MediaChildKind::TextTrackContainer,
            DOMNodeClass::MediaRemotingInterstitial(_) => MediaChildKind::RemotingInterstitial,
            DOMNodeClass::PictureInPictureInterstitial(_) => {
                MediaChildKind::PictureInPictureInterstitial
            }
            _ => return None,
        })
    }
    pub fn MediaChildOwner(&self, child: usize) -> Option<DOMOwnerHandle> {
        Some(match &self.arena[child].class {
            DOMNodeClass::MediaControls(owner) => owner.media,
            DOMNodeClass::TextTrackContainer(owner) => owner.media,
            DOMNodeClass::MediaRemotingInterstitial(owner) => owner.video,
            DOMNodeClass::PictureInPictureInterstitial(owner) => owner.video,
            _ => return None,
        })
    }
    pub fn AllocateMediaControlPlayButton(&mut self, controls: usize) -> usize {
        assert_eq!(
            self.MediaChildKindForLayout(controls),
            Some(MediaChildKind::Controls)
        );
        let owner = self.NodeHandle(controls);
        let child = self.CreateElementDefault(DOMNamespace::kHTML, "input".into());
        self.arena[child].class = DOMNodeClass::MediaControlInput {
            controls: owner,
            kind: MediaControlInputClass::PlayButton,
        };
        child
    }
    pub fn MediaControlInputClass(&self, node: usize) -> Option<MediaControlInputClass> {
        if let DOMNodeClass::MediaControlInput { kind, .. } = self.arena[node].class {
            Some(kind)
        } else {
            None
        }
    }
    pub fn AllocateMediaControlPanel(&mut self, controls: usize) -> usize {
        assert_eq!(
            self.MediaChildKindForLayout(controls),
            Some(MediaChildKind::Controls)
        );
        let owner = self.NodeHandle(controls);
        let media = self
            .MediaChildOwner(controls)
            .expect("controls media owner");
        let media_index = self
            .FindNodeById(media.Id())
            .expect("original media element");
        assert_eq!(self.NodeHandle(media_index), media);
        let child = self.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        self.arena[child].class =
            DOMNodeClass::MediaControlPanel(MediaControlPanel { controls: owner });
        self.SetMediaControlShadowPseudoId(media_index, child, "-webkit-media-controls-panel");
        child
    }
    // These are deliberately the base-allocation subset, not complete widget
    // constructors: no ordinary HTML div is ever promoted by tag/pseudo lookup.
    pub(crate) fn AllocateMediaControlsBase(&mut self, media: usize) -> usize {
        assert!(self.IsMediaElement(media));
        let owner = self.NodeHandle(media);
        let child = self.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        self.arena[child].class = DOMNodeClass::MediaControls(MediaControls {
            media: owner,
            event_state: std::rc::Rc::new(std::cell::OnceCell::new()),
        });
        self.SetMediaControlShadowPseudoId(media, child, "-webkit-media-controls");
        child
    }
    pub(crate) fn AllocateTextTrackContainerBase(&mut self, media: usize) -> usize {
        assert!(self.IsMediaElement(media));
        let owner = self.NodeHandle(media);
        let child = self.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        self.arena[child].class =
            DOMNodeClass::TextTrackContainer(TextTrackContainer { media: owner });
        self.SetMediaControlShadowPseudoId(media, child, "-webkit-media-text-track-container");
        child
    }
    pub(crate) fn AllocateMediaRemotingInterstitialBase(&mut self, video: usize) -> usize {
        assert!(self.IsVideoElement(video));
        let owner = self.NodeHandle(video);
        let child = self.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        self.arena[child].class =
            DOMNodeClass::MediaRemotingInterstitial(MediaRemotingInterstitial { video: owner });
        self.SetMediaControlShadowPseudoId(video, child, "-internal-media-interstitial");
        child
    }
    pub(crate) fn AllocatePictureInPictureInterstitialBase(&mut self, video: usize) -> usize {
        assert!(self.IsVideoElement(video));
        let owner = self.NodeHandle(video);
        let child = self.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        self.arena[child].class =
            DOMNodeClass::PictureInPictureInterstitial(PictureInPictureInterstitial {
                video: owner,
            });
        self.SetMediaControlShadowPseudoId(video, child, "-internal-media-interstitial");
        child
    }
}

impl PersistentDocument {
    /// HTMLMediaElement::EnsureUserAgentShadowRoot and
    /// EnsureTextTrackContainer, html_media_element.cc:4541-4570.
    pub fn EnsureMediaTextTrackContainer(&mut self, media: usize) -> (usize, bool) {
        assert!(self.IsMediaElement(media));
        let root = self.ShadowRoot(media).unwrap_or_else(|| {
            self.AttachShadowRootWithSlotAssignment(
                media,
                crate::persistent_document::DOMShadowRootMode::kUserAgent,
                crate::persistent_document::DOMSlotAssignmentMode::kNamed,
            )
        });
        assert_eq!(
            self.ShadowRootMode(root),
            Some(crate::persistent_document::DOMShadowRootMode::kUserAgent)
        );
        let first = self.Node(root).Children().first().copied();
        if let Some(first) = first {
            if self.MediaChildKindForLayout(first) == Some(MediaChildKind::TextTrackContainer) {
                return (first, false);
            }
        }
        let before = if first.is_some_and(|first| {
            matches!(
                self.MediaChildKindForLayout(first),
                Some(
                    MediaChildKind::RemotingInterstitial
                        | MediaChildKind::PictureInPictureInterstitial
                )
            )
        }) {
            first.and_then(|first| self.Node(first).NextSibling())
        } else {
            first
        };
        if let Some(before) = before {
            if self.MediaChildKindForLayout(before) == Some(MediaChildKind::TextTrackContainer) {
                return (before, false);
            }
        }
        let container = self.AllocateTextTrackContainerBase(media);
        if let Some(before) = before {
            self.InsertBefore(root, container, before);
        } else {
            self.AppendChild(root, container);
        }
        (container, true)
    }
    pub fn ClearTextTrackContainer(&mut self, container: usize) {
        assert_eq!(
            self.MediaChildKindForLayout(container),
            Some(MediaChildKind::TextTrackContainer)
        );
        while let Some(child) = self.Node(container).Children().first().copied() {
            self.Remove(child);
        }
    }
}

impl PersistentDocument {
    pub fn BindMediaControlsEventState(
        &self,
        controls: usize,
        state: std::rc::Rc<dyn MediaControlsEventState>,
    ) {
        let DOMNodeClass::MediaControls(controls) = &self.arena[controls].class else {
            unreachable!("actual MediaControlsImpl native class");
        };
        assert!(
            controls.event_state.set(state).is_ok(),
            "one original ControlsImpl source owner"
        );
    }
    pub(crate) fn KeepMediaControlEventInNode(
        &self,
        node: usize,
        event: crate::event_path::DOMEventPathEvent<'_>,
    ) -> bool {
        match &self.arena[node].class {
            // MediaControlPanelElement::KeepEventInNode cc:139-142.
            DOMNodeClass::MediaControlPanel(panel) => {
                let controls = self
                    .FindNodeById(panel.controls.Id())
                    .expect("original controls node");
                assert_eq!(self.NodeHandle(controls), panel.controls);
                let DOMNodeClass::MediaControls(controls) = &self.arena[controls].class else {
                    unreachable!("panel native parent source class");
                };
                controls
                    .event_state
                    .get()
                    .expect("native ControlsImpl bound before panel insertion")
                    .ShouldShowAudioControls()
                    && event.IsUserInteraction()
            }
            DOMNodeClass::MediaControlDiv {
                kind: MediaControlDivClass::ButtonPanel,
                ..
            } => event.IsUserInteraction(),
            // Node::KeepEventInNode's inherited false branch. No source
            // subclass is selected from a tag or pseudo attribute here.
            _ => false,
        }
    }
}

impl PersistentDocument {
    pub fn AllocateMediaControlDiv(
        &mut self,
        controls: usize,
        kind: MediaControlDivClass,
    ) -> usize {
        assert_eq!(
            self.MediaChildKindForLayout(controls),
            Some(MediaChildKind::Controls)
        );
        let owner = self.NodeHandle(controls);
        let child = self.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        self.arena[child].class = DOMNodeClass::MediaControlDiv {
            controls: owner,
            kind,
        };
        child
    }
    pub fn MediaControlDivClass(&self, node: usize) -> Option<MediaControlDivClass> {
        match &self.arena[node].class {
            DOMNodeClass::MediaControlDiv { kind, .. } => Some(*kind),
            _ => None,
        }
    }
}
