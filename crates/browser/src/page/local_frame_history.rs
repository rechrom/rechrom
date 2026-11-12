//! RenderFrameImpl's UniqueNameHelper adapter and NavigationEntry's recursive
//! FrameNavigationEntry boundary. Only immutable history values leave a Page;
//! live child Documents and their lazy FormController owners stay in Frames.
use super::local_frame::{LocalFrame, LocalFrameDomain};
use super::*;
use navigation_history::{
    unique_name::{FrameAdapter, IsFramePath, UniqueNameHelper},
    ChildHistoryItemSnapshot, FrameLoadType, HistoryItemOwner, HistoryItemSnapshot,
};

struct FrameNameAdapter<'a> {
    domain: &'a LocalFrameDomain,
    parent: u64,
    current: Option<u64>,
}
impl FrameAdapter for FrameNameAdapter<'_> {
    fn IsCandidateUnique(&self, name: &str) -> bool {
        !self.domain.frames.borrow().values().any(|frame| {
            Some(frame.state.Identity()) != self.current
                && frame.unique_name.borrow().Value() == name
        })
    }
    fn SiblingCount(&self) -> usize {
        self.domain.Children(self.parent).len() - usize::from(self.current.is_some())
    }
    fn AncestorNames(&self) -> Vec<String> {
        let mut names = Vec::new();
        let mut parent = self.parent;
        loop {
            if parent == self.domain.root {
                names.push(String::new());
                break;
            }
            let frame = self.domain.Frame(parent).expect("live frame ancestor");
            let name = frame.unique_name.borrow().Value().to_owned();
            let stop = IsFramePath(&name);
            names.push(name);
            if stop {
                break;
            }
            parent = frame.parent;
        }
        names
    }
    fn FramePosition(&self) -> Vec<usize> {
        let mut positions = Vec::new();
        let mut parent = self.parent;
        let mut child = self.current;
        loop {
            let children = self.domain.Children(parent);
            positions.push(child.map_or(children.len(), |child| {
                children
                    .iter()
                    .position(|frame| frame.state.Identity() == child)
                    .expect("live frame sibling")
            }));
            if parent == self.domain.root {
                break;
            }
            child = Some(parent);
            parent = self
                .domain
                .Frame(parent)
                .expect("live frame ancestor")
                .parent;
        }
        positions
    }
}
impl LocalFrameDomain {
    pub(super) fn SetMirrorHistoryState(&self, state: navigation_history::SessionHistoryState) {
        self.session_history_state.set(state);
        if let Some(root) = self.root_navigation.borrow().upgrade() {
            root.SetSessionHistoryState(state);
        }
        for frame in self.frames.borrow().values() {
            if let Some(window) = frame.navigation.borrow().window.clone() {
                window.navigation.SetSessionHistoryState(state);
            }
        }
    }
    fn MatchesItemTree(&self, frame: u64, expected: &HistoryItemSnapshot) -> bool {
        let current = if frame == self.root {
            self.page
                .upgrade()
                .and_then(|page| page.DocumentLoaderNavigation().CurrentHistoryItem())
        } else {
            self.Frame(frame).and_then(|frame| {
                frame.runtime.borrow().as_ref().and_then(|runtime| {
                    runtime
                        .navigation_state
                        .get()
                        .and_then(|loader| loader.CurrentHistoryItem())
                })
            })
        };
        if !current.is_some_and(|item| {
            item.ItemSequenceNumber() == expected.item_sequence_number
                && item.DocumentSequenceNumber() == expected.document_sequence_number
        }) {
            return false;
        }
        self.Children(frame).iter().all(|child| {
            let Some(saved) = expected
                .children
                .iter()
                .find(|saved| saved.unique_name == child.unique_name.borrow().Value())
            else {
                return false;
            };
            if let Some(pending) = child.navigation.borrow().pending_history_restore.clone() {
                pending.SameItemTree(&saved.item)
            } else {
                self.MatchesItemTree(child.state.Identity(), &saved.item)
            }
        })
    }
    fn CommitSameDocumentWithoutScript(&self, frame: u64, snapshot: &HistoryItemSnapshot) {
        use document_loader::{
            SameDocumentCommitStage, SameDocumentNavigationFacts, SameDocumentNavigationParams,
            SameDocumentNavigationType,
        };
        let document = self.Document(frame).expect("live history document");
        let loader = if frame == self.root {
            self.page
                .upgrade()
                .expect("live Page")
                .DocumentLoaderNavigation()
        } else {
            self.Frame(frame)
                .expect("live child")
                .runtime
                .borrow()
                .as_ref()
                .expect("child document")
                .navigation_state
                .get()
                .expect("committed child loader")
                .clone()
        };
        let facts = {
            let owner = document.borrow();
            let doc = owner.GetDocument();
            SameDocumentNavigationFacts {
                document_url: doc.DocumentURL().to_owned(),
                is_on_initial_empty_document: doc.IsInitialEmptyDocument(),
                was_loading: false,
                has_transient_user_activation: false,
                document_origin: doc.SecurityContext().Origin().clone(),
            }
        };
        let mut commit = loader.RunURLAndHistoryUpdateSteps(
            SameDocumentNavigationParams {
                new_url: snapshot.url.clone(),
                history_item: Some(HistoryItemOwner::Restore(snapshot)),
                navigation_type: SameDocumentNavigationType::Fragment,
                state: None,
                load_type: FrameLoadType::BackForward,
                fire_popstate: false,
                initiator_origin: None,
                is_browser_initiated: true,
                is_synchronously_committed: false,
                has_ua_visual_transition: false,
                should_skip_screenshot: false,
                involvement: navigation_history::UserNavigationInvolvement::BrowserUI,
                interaction_id: None,
            },
            facts,
        );
        loop {
            match commit.Next(true) {
                SameDocumentCommitStage::UpdateDocument { .. } => {
                    let state = document.borrow().GetDocument().FormElementsState();
                    commit.ApplyDocumentUpdate(document.borrow_mut().GetDocumentMut(), state);
                }
                SameDocumentCommitStage::Done => break,
                _ => {} // This Document has no script realm/listeners.
            }
        }
        if let Some(child) = self.Frame(frame) {
            child.navigation.borrow_mut().pending_history_restore = None;
            child.MarkNavigationDirty();
        }
        if let Some(page) = self.page.upgrade() {
            if frame == self.root {
                let snapshot = self.SnapshotHistoryItem(
                    &loader.CurrentHistoryItem().expect("committed history"),
                    self.root,
                );
                page.client
                    .borrow_mut()
                    .DidCommitHistory(snapshot, FrameLoadType::BackForward);
            } else {
                page.PublishChildHistory(FrameLoadType::BackForward);
            }
        }
    }
    fn TraverseChildren(&self, parent: u64, destination: &HistoryItemSnapshot) -> bool {
        let mut changed = false;
        for frame in self.Children(parent) {
            let Some(saved) = destination
                .children
                .iter()
                .find(|child| child.unique_name == frame.unique_name.borrow().Value())
            else {
                continue;
            };
            let current = frame.runtime.borrow().as_ref().and_then(|runtime| {
                runtime
                    .navigation_state
                    .get()
                    .and_then(|loader| loader.CurrentHistoryItem())
            });
            let Some(current) = current else {
                continue;
            };
            if current.ItemSequenceNumber() == saved.item.item_sequence_number {
                changed |= self.TraverseChildren(frame.state.Identity(), &saved.item);
                continue;
            }
            changed = true;
            frame.navigation.borrow_mut().pending_history_restore = Some(saved.item.clone());
            if current.DocumentSequenceNumber() == saved.item.document_sequence_number {
                let window = frame.navigation.borrow().window.clone();
                if let Some(window) = window {
                    window.navigation.TraverseToHistoryItem(saved.item.clone());
                } else {
                    self.CommitSameDocumentWithoutScript(frame.state.Identity(), &saved.item);
                }
                changed |= self.TraverseChildren(frame.state.Identity(), &saved.item);
            } else {
                let mut request = URLRequest {
                    url: saved.item.url.clone(),
                    ..Default::default()
                };
                super::navigation_api::RestoreHistoryRequest(&mut request, &saved.item);
                request.navigation = Some(navigation_history::NavigationRequestMetadata {
                    load_type: FrameLoadType::BackForward,
                    history_item: Some(saved.item.clone()),
                    back_entries: Arc::from([]),
                    forward_entries: Arc::from([]),
                    navigation_api_state: None,
                    navigation_type: navigation_history::WebNavigationType::BackForward,
                    is_browser_initiated: true,
                    had_transient_user_activation: false,
                    requestor_origin: saved.item.requestor_origin.clone(),
                    propagated_text_fragment_token: None,
                    internal_scroll_to_text_fragment: None,
                });
                frame.RequestScriptNavigationRequest(request, FrameLoadType::BackForward);
            }
        }
        changed
    }
    pub(super) fn GenerateUniqueName(
        &self,
        parent: u64,
        name: &[u16],
        is_created_by_script: bool,
    ) -> UniqueNameHelper {
        UniqueNameHelper::GenerateForNewChild(
            &FrameNameAdapter {
                domain: self,
                parent,
                current: None,
            },
            &String::from_utf16_lossy(name),
            is_created_by_script,
        )
    }
    /// Materialize each original Document's lazy state on its owning sequence.
    /// No control/tree projection and no renderer token is persisted.
    pub(super) fn SnapshotHistoryItem(
        &self,
        item: &HistoryItemOwner,
        frame: u64,
    ) -> Arc<HistoryItemSnapshot> {
        let mut snapshot = item.Snapshot();
        let data = Arc::get_mut(&mut snapshot).expect("new history snapshot");
        data.session_entry_id = self.session_entry_id.get();
        if let Some(document) = self.Document(frame) {
            data.committed_origin = Some(super::navigation_api::RequestorOriginFor(
                document.borrow().GetDocument().SecurityContext().Origin(),
            ));
        }
        let mut children: Vec<_> = self
            .Children(frame)
            .iter()
            .filter_map(|child| {
                if let Some(item) = child.navigation.borrow().pending_history_restore.clone() {
                    return Some(ChildHistoryItemSnapshot {
                        unique_name: child.unique_name.borrow().Value().to_owned(),
                        item,
                    });
                }
                let runtime = child.runtime.borrow().clone()?;
                let item = runtime.navigation_state.get()?.CurrentHistoryItem()?;
                Some(ChildHistoryItemSnapshot {
                    unique_name: child.unique_name.borrow().Value().to_owned(),
                    item: self.SnapshotHistoryItem(&item, child.state.Identity()),
                })
            })
            .collect();
        let restore = if frame == self.root {
            self.root_history_restore.borrow().clone()
        } else {
            self.Frame(frame)
                .and_then(|frame| frame.navigation.borrow().committed_history_restore.clone())
        };
        if let Some(restore) = restore {
            for child in restore.children.iter() {
                if !children
                    .iter()
                    .any(|live| live.unique_name == child.unique_name)
                {
                    children.push(child.clone());
                }
            }
        }
        data.children = children.into();
        snapshot
    }
    /// Install the browser-provided frame tree before the parser creates its
    /// children. Matching happens at each new owner, by its stable unique name.
    pub(super) fn InstallHistoryRestore(
        &self,
        frame: u64,
        loader: &document_loader::DocumentLoaderNavigationState,
    ) {
        let snapshot = loader
            .CrossDocumentNavigationMetadata()
            .filter(|metadata| metadata.load_type.IsTraversal())
            .and_then(|metadata| metadata.history_item);
        if frame == self.root {
            if let Some(metadata) = loader.CrossDocumentNavigationMetadata() {
                let offset = metadata.back_entries.len() as i32;
                self.SetMirrorHistoryState(navigation_history::SessionHistoryState {
                    offset,
                    length: offset + 1 + metadata.forward_entries.len() as i32,
                });
            } else {
                self.SetMirrorHistoryState(navigation_history::SessionHistoryState {
                    offset: 0,
                    length: 1,
                });
            }
            self.session_entry_id.set(
                loader
                    .CrossDocumentNavigationMetadata()
                    .and_then(|metadata| metadata.history_item)
                    .and_then(|item| item.session_entry_id),
            );
            *self.root_history_restore.borrow_mut() = snapshot;
        } else if let Some(frame) = self.Frame(frame) {
            frame.navigation.borrow_mut().committed_history_restore = snapshot;
        }
    }
    pub(super) fn MatchChildHistory(&self, frame: &LocalFrame) {
        let parent = if frame.parent == self.root {
            self.root_history_restore.borrow().clone()
        } else {
            self.Frame(frame.parent)
                .and_then(|parent| parent.navigation.borrow().committed_history_restore.clone())
        };
        let Some(parent) = parent else {
            return;
        };
        let item = parent
            .children
            .iter()
            .find(|child| child.unique_name == frame.unique_name.borrow().Value())
            .map(|child| child.item.clone());
        frame.navigation.borrow_mut().pending_history_restore = item;
    }
    pub(super) fn RestoreChildNavigation(
        &self,
        frame: &LocalFrame,
        navigation: &mut dom::frame_owner_loading::FrameOwnerNavigation,
    ) {
        if navigation.reason
            != dom::frame_owner_loading::FrameOwnerNavigationReason::InitialFrameNavigation
        {
            return;
        }
        let Some(item) = frame.navigation.borrow().pending_history_restore.clone() else {
            return;
        };
        let load_type = self
            .LoadType(frame.parent)
            .filter(|load| load.IsTraversal())
            .unwrap_or(FrameLoadType::BackForward);
        let mut request = URLRequest {
            url: item.url.clone(),
            ..Default::default()
        };
        super::navigation_api::RestoreHistoryRequest(&mut request, &item);
        navigation.request.url = self
            .Document(frame.state.Identity())
            .expect("live child Document")
            .borrow()
            .GetDocument()
            .CompleteURLValue(&item.url);
        navigation.request.method = request.method;
        navigation.request.referrer = request.referrer;
        navigation.request.referrer_policy = item.referrer_policy;
        navigation.request.headers = request.headers;
        navigation.request.body = request.body;
        navigation.request.encoded_body = request.encoded_body;
        navigation.request.navigation = Some(navigation_history::NavigationRequestMetadata {
            load_type,
            history_item: Some(item),
            back_entries: Arc::from([]),
            forward_entries: Arc::from([]),
            navigation_api_state: None,
            navigation_type: navigation_history::WebNavigationType::Determine(
                load_type, false, false,
            ),
            is_browser_initiated: true,
            had_transient_user_activation: false,
            requestor_origin: None,
            propagated_text_fragment_token: None,
            internal_scroll_to_text_fragment: None,
        });
        navigation.load_type = load_type;
    }
}
impl Page {
    /// Browser-provided joint entry; FrameLoader compares each resident frame's
    /// item/document sequence before choosing same- or different-document work.
    pub fn TraverseSessionHistory(
        &self,
        destination: Arc<HistoryItemSnapshot>,
        state: navigation_history::SessionHistoryState,
    ) -> bool {
        let Some(domain) = self.state.local_frames.get() else {
            return false;
        };
        let Some(current) = self.state.DocumentLoaderNavigation().CurrentHistoryItem() else {
            return false;
        };
        if current.DocumentSequenceNumber() != destination.document_sequence_number {
            return false;
        }
        self.state.SaveHistoryState();
        domain.session_entry_id.set(destination.session_entry_id);
        domain.SetMirrorHistoryState(state);
        *domain.root_history_restore.borrow_mut() = Some(destination.clone());
        let children_changed = domain.TraverseChildren(domain.root, &destination);
        if current.ItemSequenceNumber() != destination.item_sequence_number {
            if let Some(scripts) = self.scripts.as_ref() {
                return scripts.TraverseToHistoryItem(destination);
            }
            domain.CommitSameDocumentWithoutScript(domain.root, &destination);
        } else if !children_changed {
            self.state
                .client
                .borrow_mut()
                .DidCommitHistory(destination, FrameLoadType::BackForward);
        }
        true
    }
    pub fn SetSessionHistoryState(
        &self,
        acknowledged: Arc<HistoryItemSnapshot>,
        state: navigation_history::SessionHistoryState,
    ) {
        let Some(domain) = self.state.local_frames.get() else {
            return;
        };
        if !domain.MatchesItemTree(domain.root, &acknowledged) {
            return;
        }
        domain.session_entry_id.set(acknowledged.session_entry_id);
        domain.SetMirrorHistoryState(state);
    }
}
impl LocalFrame {
    pub(super) fn UpdateUniqueName(&self) {
        let Some(domain) = self.domain.upgrade() else {
            return;
        };
        if self.unique_name.borrow().IsFrozen() {
            return;
        }
        self.unique_name.borrow_mut().UpdateName(
            &FrameNameAdapter {
                domain: &domain,
                parent: self.parent,
                current: Some(self.state.Identity()),
            },
            &String::from_utf16_lossy(&self.name.borrow()),
        );
        domain.MatchChildHistory(self);
    }
}
