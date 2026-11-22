//! Native HTMLFrameOwnerElement content-frame identity and Node's original
//! connected-subframe count. Source html_frame_owner_element.cc282-381;
//! child_frame_disconnector.cc20-77. No iframe tag creates a Frame.
use super::{DOMNodeClass, DOMOwnerHandle, PersistentDocument};
use crate::frame_lifecycle::{FrameLifecycle, FrameLifecycleState};
use std::{cell::Cell, rc::Rc};

/// Retained Frame base fields, not a child Document or a second frame tree.
/// Source frame.cc:520-550; frame.h:212-217. Only the actual Frame constructor
/// supplies its identity/owner. An HTML element does not construct this state.
pub struct ContentFrameState {
    identity: u64,
    owner: Cell<Option<DOMOwnerHandle>>,
    lifecycle: FrameLifecycle,
    is_loading: Cell<bool>,
}
impl ContentFrameState {
    pub fn new(identity: u64, owner: Option<DOMOwnerHandle>) -> Self {
        Self {
            identity,
            owner: Cell::new(owner),
            lifecycle: FrameLifecycle::default(),
            is_loading: Cell::new(false),
        }
    }
    pub fn Identity(&self) -> u64 {
        self.identity
    }
    pub fn Owner(&self) -> Option<DOMOwnerHandle> {
        self.owner.get()
    }
    pub fn Lifecycle(&self) -> &FrameLifecycle {
        &self.lifecycle
    }
    pub fn IsLoading(&self) -> bool {
        self.is_loading.get()
    }
    pub fn SetIsLoading(&self, loading: bool) {
        self.is_loading.set(loading);
    }
}
/// Original native child Frame virtuals. Only the native frame constructor may
/// bind this owner; foreign Page domains are never synchronously invoked here.
pub trait ContentFrameSource {
    fn State(&self) -> &ContentFrameState;
    fn IsRemoteFrame(&self) -> bool;
    fn DetachForRemove(&self);
}
impl dyn ContentFrameSource + '_ {
    pub fn Identity(&self) -> u64 {
        self.State().Identity()
    }
    pub fn IsLoading(&self) -> bool {
        self.State().IsLoading()
    }
}

/// Source Frame::Initialize's ownerless branch is routed to Page::SetMainFrame
/// by the embedding; DOM never owns or invokes that Page.
pub enum FrameInitialization {
    ContentFrameBound,
    SetMainFrame(Rc<dyn ContentFrameSource>),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameOwnerError {
    WrongDocument,
    MissingOwner,
    DisconnectedOwner,
    NotFrameOwner,
    AlreadyBound,
    NotAttached,
    NotDetached,
}
/// The actual LazyLoadFrameObserver, if a frame-owner loading path created one.
/// Absence is independent of an HTML loading attribute.
pub trait LazyLoadFrameObserverSource {
    fn CancelPendingLazyLoad(&self);
    fn IsLazyLoadPending(&self) -> bool;
    fn LoadImmediately(&self) -> Result<(), crate::frame_owner_loading::FrameOwnerLoadError>;
    fn DeferLoadUntilNearViewport(
        &self,
        request: crate::frame_owner_loading::FrameOwnerResourceRequest,
        load_type: navigation_history::FrameLoadType,
    ) -> Result<(), crate::frame_owner_loading::FrameOwnerLoadError>;
}
#[derive(Clone)]
pub struct ContentFrameOwner {
    pub owner: DOMOwnerHandle,
    pub frame: Rc<dyn ContentFrameSource>,
}
impl PersistentDocument {
    fn FrameOwnerIndex(&self, owner: DOMOwnerHandle) -> Result<usize, FrameOwnerError> {
        if owner.arena_id != self.arena_id {
            return Err(FrameOwnerError::WrongDocument);
        }
        self.FindNodeById(owner.node_id)
            .ok_or(FrameOwnerError::MissingOwner)
    }
    /// Frame::Initialize, frame.cc:552-560. Creation supplies the real Frame;
    /// only initialization links it to its owner, preserving source identity.
    pub fn InitializeFrame(
        &mut self,
        frame: Rc<dyn ContentFrameSource>,
    ) -> Result<FrameInitialization, FrameOwnerError> {
        if frame.State().Lifecycle().GetState() != FrameLifecycleState::Attached {
            return Err(FrameOwnerError::NotAttached);
        }
        let Some(owner) = frame.State().Owner() else {
            return Ok(FrameInitialization::SetMainFrame(frame));
        };
        let index = self.FrameOwnerIndex(owner)?;
        if !self.IsFrameOwnerElement(index) {
            return Err(FrameOwnerError::NotFrameOwner);
        }
        if !self.IsConnectedToDocument(index) {
            return Err(FrameOwnerError::DisconnectedOwner);
        }
        if self.ContentFrame(index).is_some() {
            return Err(FrameOwnerError::AlreadyBound);
        }
        self.SetContentFrame(index, frame);
        Ok(FrameInitialization::ContentFrameBound)
    }
    /// Source Frame::DisconnectOwnerElement, frame.cc:190-201, after the
    /// client's detach callback and kDetached transition. A provisional Frame
    /// must not clear another Frame's binding to the same original owner.
    pub fn DisconnectFrameOwner(
        &mut self,
        frame: &Rc<dyn ContentFrameSource>,
    ) -> Result<(), FrameOwnerError> {
        if frame.State().Lifecycle().GetState() != FrameLifecycleState::Detached {
            return Err(FrameOwnerError::NotDetached);
        }
        let Some(owner) = frame.State().Owner() else {
            return Ok(());
        };
        let index = self.FrameOwnerIndex(owner)?;
        if self
            .ContentFrame(index)
            .is_some_and(|current| Rc::ptr_eq(&current, frame))
        {
            self.ClearContentFrame(index);
        }
        frame.State().owner.set(None);
        Ok(())
    }
    pub fn IsFrameOwnerElement(&self, node: usize) -> bool {
        matches!(self.Node(node).class, DOMNodeClass::HTMLFrameElementBase)
    }
    pub fn ContentFrame(&self, owner: usize) -> Option<Rc<dyn ContentFrameSource>> {
        self.Node(owner).content_frame.clone()
    }
    pub fn FrameLazyLoadObserver(
        &self,
        owner: usize,
    ) -> Option<Rc<dyn LazyLoadFrameObserverSource>> {
        self.Node(owner).lazy_frame_observer.clone()
    }
    pub fn BindFrameLazyLoadObserver(
        &mut self,
        owner: usize,
        observer: Rc<dyn LazyLoadFrameObserverSource>,
    ) {
        assert!(self.IsFrameOwnerElement(owner));
        self.NodeMut(owner).lazy_frame_observer = Some(observer);
    }
    pub fn SetContentFrame(&mut self, owner: usize, frame: Rc<dyn ContentFrameSource>) {
        assert!(self.IsFrameOwnerElement(owner));
        assert!(self.IsConnectedToDocument(owner));
        assert_eq!(frame.State().Owner(), Some(self.NodeHandle(owner)));
        assert_eq!(
            frame.State().Lifecycle().GetState(),
            FrameLifecycleState::Attached,
        );
        assert!(
            self.Node(owner).content_frame.is_none(),
            "one native Frame per original owner"
        );
        self.NodeMut(owner).content_frame = Some(frame);
        self.AdjustConnectedSubframeCount(owner, true);
        self.InvalidateNodeStyle(owner);
    }
    /// Frame::Detach's owner notification, after its actual detach callbacks.
    pub fn ClearContentFrame(&mut self, owner: usize) {
        // Source html_frame_owner_element.cc:328-333: cancellation also runs
        // when Frame::Detach bypassed DisconnectContentFrame.
        if self.Node(owner).content_frame.is_some() {
            if let Some(observer) = self.FrameLazyLoadObserver(owner) {
                observer.CancelPendingLazyLoad();
            }
        }
        if self.NodeMut(owner).content_frame.take().is_some() {
            self.AdjustConnectedSubframeCount(owner, false);
        }
    }
    fn AdjustConnectedSubframeCount(&mut self, owner: usize, increment: bool) {
        let mut next = Some(owner);
        while let Some(node) = next {
            let value = &mut self.NodeMut(node).connected_subframe_count;
            *value = if increment {
                value.checked_add(1).expect("Node connected subframe count")
            } else {
                value
                    .checked_sub(1)
                    .expect("balanced Node connected subframe count")
            };
            next = self.Node(node).Parent().or_else(|| self.ShadowHost(node));
        }
    }
    pub fn ConnectedSubframeCount(&self, root: usize) -> u32 {
        self.Node(root).connected_subframe_count
    }
    /// ChildFrameDisconnector::CollectFrameOwners; copies only the retained
    /// owner identities, not a Frame/DOM tree. Script runs after this borrow.
    pub fn CollectChildFrameOwners(&self, root: usize) -> Vec<ContentFrameOwner> {
        let mut owners = Vec::new();
        let mut pending = vec![root];
        while let Some(node) = pending.pop() {
            if self.ConnectedSubframeCount(node) == 0 {
                continue;
            }
            if self.IsFrameOwnerElement(node) {
                if let Some(frame) = self.ContentFrame(node) {
                    owners.push(ContentFrameOwner {
                        owner: self.NodeHandle(node),
                        frame,
                    });
                }
            }
            // Source visits light descendants before a retained shadow root.
            if let Some(shadow) = self.ShadowRoot(node) {
                pending.push(shadow);
            }
            pending.extend(self.Node(node).Children().iter().rev().copied());
        }
        owners
    }
}
