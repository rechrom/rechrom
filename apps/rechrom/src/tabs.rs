//! The embedding owns tabs. A tab keeps its document and session history while
//! another tab is visible; Page and the rendering crates do not own windows.
use navigation_history::{
    FrameLoadType, HistoryItemIdentitySource, HistoryItemSnapshot, NativeHistoryItemIdentitySource,
    NavigationRequestMetadata, WebNavigationType,
};
use std::{collections::BTreeMap, sync::Arc};

/// NavigationController's pending entry is separate from committed history.
/// A destination index exists only for traversal/reload, never a new URL.
pub struct PendingNavigation<P> {
    pub generation: u64,
    pub history_index: Option<usize>,
    pub page: Option<P>,
    pub previous_location: String,
    pub previous_title: String,
    pub previous_loading: bool,
}

pub struct Tab<P> {
    pub id: u64,
    pub page: Option<P>,
    pub pending_navigation: Option<PendingNavigation<P>>,
    /// Browsing-context name survives document navigation.
    pub name: String,
    /// Immutable host policy of the latest admitted renderer navigation.
    pub navigation_host: Option<browser::page::NavigationHostMetadata>,
    pub location: String,
    pub title: String,
    pub loading: bool,
    pub history: Vec<Arc<HistoryItemSnapshot>>,
    /// Browser-owned about: pages load data: backing documents. Their public
    /// address is UI metadata, separate from the loader's canonical item URL.
    history_display_urls: BTreeMap<i64, String>,
    pub history_index: usize,
    /// The document that last published the current entry. A new pending Page
    /// must not receive a same-document key belonging to its predecessor.
    pub history_document_generation: Option<u64>,
    pub document_generation: u64,
    pub viewport: Option<crate::engine::Viewport>,
    pub title_node_id: Option<u64>,
    pub metadata_nodes: usize,
    pub metadata_sequence: u64,
}
impl<P> Tab<P> {
    /// Source: NavigationControllerImpl::GetCurrentEntryIndex/GetIndexForOffset.
    /// A pending new URL does not fabricate an entry after the committed one.
    pub fn current_history_index(&self) -> usize {
        self.pending_navigation
            .as_ref()
            .and_then(|pending| pending.history_index)
            .unwrap_or(self.history_index)
    }

    pub fn history_index_for_offset(&self, delta: isize) -> Option<usize> {
        self.current_history_index()
            .checked_add_signed(delta)
            .filter(|index| *index < self.history.len())
    }

    pub fn begin_navigation(&mut self, generation: u64, request: &url_loader::URLRequest) {
        let history_index = request.navigation.as_ref().and_then(|metadata| {
            (metadata.load_type.IsTraversal() || metadata.load_type.IsReload())
                .then_some(metadata.history_item.as_ref())
                .flatten()
                .and_then(|item| {
                    self.history.iter().position(|entry| {
                        item.session_entry_id.map_or(
                            entry.item_sequence_number == item.item_sequence_number,
                            |id| entry.session_entry_id == Some(id),
                        )
                    })
                })
        });
        let previous = self.pending_navigation.take();
        let (previous_location, previous_title, previous_loading) = match previous {
            Some(previous) => (
                previous.previous_location,
                previous.previous_title,
                previous.previous_loading,
            ),
            None => (self.location.clone(), self.title.clone(), self.loading),
        };
        self.pending_navigation = Some(PendingNavigation {
            generation,
            history_index,
            page: None,
            previous_location,
            previous_title,
            previous_loading,
        });
        self.document_generation = generation;
    }

    pub fn commit_pending_navigation(&mut self, generation: u64) -> Option<P> {
        if !self
            .pending_navigation
            .as_ref()
            .is_some_and(|pending| pending.generation == generation)
        {
            return None;
        }
        let pending = self.pending_navigation.take().unwrap();
        pending.page.and_then(|page| self.page.replace(page))
    }

    /// Source: NavigationControllerImpl::DiscardPendingEntry. Cancelling an
    /// uncommitted producer leaves the last committed Page and entries intact.
    pub fn discard_pending_navigation(&mut self, generation: u64) -> bool {
        if !self
            .pending_navigation
            .as_ref()
            .is_some_and(|pending| pending.generation == generation)
        {
            return false;
        }
        let pending = self.pending_navigation.take().unwrap();
        self.location = pending.previous_location;
        self.title = pending.previous_title;
        self.loading = pending.previous_loading;
        true
    }

    /// Commit notifications carry values only; no host operation reaches back
    /// into a Page's loader or Window to reconstruct session history.
    pub fn commit_history(&mut self, item: Arc<HistoryItemSnapshot>, load: FrameLoadType) {
        let mut item = item;
        if self.location.starts_with("about:") && item.url.starts_with("data:") {
            self.history_display_urls
                .insert(item.item_sequence_number, self.location.clone());
        }
        let existing = if load.IsTraversal() {
            item.session_entry_id
                .and_then(|id| {
                    self.history.iter().position(|entry| {
                        entry.session_entry_id == Some(id)
                            && entry.item_sequence_number == item.item_sequence_number
                    })
                })
                .or_else(|| {
                    self.history
                        .iter()
                        .position(|entry| entry.item_sequence_number == item.item_sequence_number)
                })
        } else {
            self.history
                .get(self.history_index)
                .filter(|entry| entry.item_sequence_number == item.item_sequence_number)
                .map(|_| self.history_index)
        };
        if let Some(index) = existing {
            Arc::make_mut(&mut item).session_entry_id = self.history[index].session_entry_id;
            self.history_index = index;
            self.history[index] = item;
        } else if !self.history.is_empty()
            && (load == FrameLoadType::ReplaceCurrentItem || load.IsReload())
        {
            Arc::make_mut(&mut item).session_entry_id =
                self.history[self.history_index].session_entry_id;
            self.history_display_urls
                .remove(&self.history[self.history_index].item_sequence_number);
            self.history[self.history_index] = item;
        } else {
            Arc::make_mut(&mut item).session_entry_id =
                Some(NativeHistoryItemIdentitySource.NextSequenceNumber());
            if !self.history.is_empty() {
                for entry in &self.history[self.history_index + 1..] {
                    self.history_display_urls
                        .remove(&entry.item_sequence_number);
                }
                self.history.truncate(self.history_index + 1);
            }
            self.history.push(item);
            self.history_index = self.history.len() - 1;
        }
    }

    /// NavigationControllerImpl::RendererDidNavigateNewSubframe clones the
    /// current entry and replaces its one frame entry; the immutable recursive
    /// snapshot already carries that native frame's updated item. Auto-subframe
    /// and history commits update/select an existing browser entry.
    pub fn commit_child_history(
        &mut self,
        mut item: Arc<HistoryItemSnapshot>,
        load: FrameLoadType,
    ) {
        if self.history.is_empty() {
            return;
        }
        if load == FrameLoadType::Standard {
            self.history.truncate(self.history_index + 1);
            Arc::make_mut(&mut item).session_entry_id =
                Some(NativeHistoryItemIdentitySource.NextSequenceNumber());
            if let Some(display) = self
                .history_display_urls
                .get(&self.history[self.history_index].item_sequence_number)
                .cloned()
            {
                self.history_display_urls
                    .insert(item.item_sequence_number, display);
            }
            self.history.push(item);
            self.history_index = self.history.len() - 1;
        } else {
            let index = if load.IsTraversal() {
                item.session_entry_id
                    .and_then(|id| {
                        self.history
                            .iter()
                            .position(|entry| entry.session_entry_id == Some(id))
                    })
                    .unwrap_or(self.history_index)
            } else {
                self.history_index
            };
            Arc::make_mut(&mut item).session_entry_id = self.history[index].session_entry_id;
            self.history[index] = item;
            self.history_index = index;
        }
    }

    /// UpdateState does not commit or select an entry. In particular an
    /// outgoing document can finish saving while a new producer is pending.
    pub fn update_history_state(&mut self, item: Arc<HistoryItemSnapshot>) {
        let current = self
            .history
            .get(self.history_index)
            .is_some_and(|entry| entry.SameItemTree(&item));
        let entry = if current {
            self.history.get_mut(self.history_index)
        } else {
            self.history.iter_mut().find(|entry| {
                entry.item_sequence_number == item.item_sequence_number
                    && entry.document_sequence_number == item.document_sequence_number
                    && item
                        .session_entry_id
                        .is_some_and(|id| entry.session_entry_id == Some(id))
            })
        };
        if let Some(entry) = entry {
            let mut item = item;
            Arc::make_mut(&mut item).session_entry_id = entry.session_entry_id;
            *entry = item;
        }
    }

    pub fn history_url<'a>(&'a self, item: &'a HistoryItemSnapshot) -> &'a str {
        self.history_display_urls
            .get(&item.item_sequence_number)
            .map(String::as_str)
            .unwrap_or(&item.url)
    }

    /// The browser owns the joint session-history list. New Pages receive
    /// immutable snapshots, retaining identities without a source-realm owner.
    pub fn prepare_navigation(
        &self,
        request: &mut url_loader::URLRequest,
        fallback_load: FrameLoadType,
    ) {
        let metadata = request
            .navigation
            .get_or_insert_with(|| NavigationRequestMetadata {
                load_type: fallback_load,
                history_item: None,
                back_entries: Arc::from([]),
                forward_entries: Arc::from([]),
                navigation_api_state: None,
                navigation_type: WebNavigationType::Determine(fallback_load, false, false),
                is_browser_initiated: true,
                had_transient_user_activation: false,
                requestor_origin: None,
                propagated_text_fragment_token: None,
                internal_scroll_to_text_fragment: None,
            });
        let destination = metadata.history_item.as_ref().and_then(|item| {
            self.history.iter().position(|entry| {
                item.session_entry_id.map_or(
                    entry.item_sequence_number == item.item_sequence_number,
                    |id| entry.session_entry_id == Some(id),
                )
            })
        });
        if let Some(index) = destination {
            // A live outgoing renderer may have refreshed this destination
            // after the host queued Reload/traversal preparation.
            metadata.history_item = Some(self.history[index].clone());
        }
        let current = destination.unwrap_or(self.history_index);
        let retained_current = metadata.load_type != FrameLoadType::Standard;
        if retained_current {
            if metadata.history_item.is_none() && metadata.load_type.IsReload() {
                metadata.history_item = self.history.get(current).cloned();
            }
            metadata.back_entries = self.history[..current.min(self.history.len())].into();
            metadata.forward_entries = self.history[(current + 1).min(self.history.len())..].into();
        } else {
            metadata.back_entries = self.history[..(current + 1).min(self.history.len())].into();
            metadata.forward_entries = Arc::from([]);
        }
        if metadata.is_browser_initiated {
            metadata.navigation_type = WebNavigationType::Determine(
                metadata.load_type,
                metadata
                    .history_item
                    .as_ref()
                    .is_some_and(|item| item.form_data.is_some()),
                false,
            );
            metadata.requestor_origin = metadata
                .history_item
                .as_ref()
                .and_then(|item| item.requestor_origin.clone());
        }
    }

    fn new(id: u64) -> Self {
        Self {
            id,
            page: None,
            pending_navigation: None,
            name: String::new(),
            navigation_host: None,
            location: "about:home".into(),
            title: "New tab".into(),
            loading: false,
            history: Vec::new(),
            history_display_urls: BTreeMap::new(),
            history_index: 0,
            history_document_generation: None,
            document_generation: 0,
            viewport: None,
            title_node_id: None,
            metadata_nodes: 0,
            metadata_sequence: 0,
        }
    }
}
pub struct Tabs<P> {
    pub active: Tab<P>,
    pub background: Vec<Tab<P>>,
    next_id: u64,
}
impl<P> Tabs<P> {
    pub fn new() -> Self {
        Self {
            active: Tab::new(1),
            background: Vec::new(),
            next_id: 2,
        }
    }
    pub fn add(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let old = std::mem::replace(&mut self.active, Tab::new(id));
        self.background.push(old);
        id
    }
    pub fn add_background(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.background.push(Tab::new(id));
        id
    }
    pub fn activate(&mut self, id: u64) -> bool {
        let Some(index) = self.background.iter().position(|tab| tab.id == id) else {
            return false;
        };
        std::mem::swap(&mut self.active, &mut self.background[index]);
        true
    }
    pub fn is_only(&self, id: u64) -> bool {
        self.active.id == id && self.background.is_empty()
    }
    // Return true when the visible tab changed. The native host owns the
    // final-tab/window decision, so the collection never fabricates a
    // replacement tab when the active tab is the last one.
    pub fn close(&mut self, id: u64) -> bool {
        if id != self.active.id {
            if let Some(index) = self.background.iter().position(|tab| tab.id == id) {
                self.background.remove(index);
            }
            return false;
        }
        if self.background.is_empty() {
            return false;
        }
        if let Some(index) = self
            .background
            .iter()
            .enumerate()
            .min_by_key(|(_, tab)| (tab.id.abs_diff(id), tab.id))
            .map(|(i, _)| i)
        {
            self.active = self.background.remove(index);
        }
        true
    }
    pub fn ordered(&self) -> Vec<&Tab<P>> {
        let mut tabs: Vec<_> = self
            .background
            .iter()
            .chain(std::iter::once(&self.active))
            .collect();
        tabs.sort_by_key(|tab| tab.id);
        tabs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn child_commits_keep_distinct_joint_entries_with_one_root_frame_item() {
        let frame = |url: &str| {
            let item = navigation_history::HistoryItemOwner::Create();
            item.SetURL(url.into());
            item.Snapshot()
        };
        let mut first = frame("https://root.test/");
        Arc::make_mut(&mut first).children = vec![navigation_history::ChildHistoryItemSnapshot {
            unique_name: "child".into(),
            item: frame("https://child.test/first"),
        }]
        .into();
        let mut tab = Tab::<()>::new(1);
        tab.commit_history(first, FrameLoadType::Standard);
        let first = tab.history[0].clone();
        let mut second = first.clone();
        Arc::make_mut(&mut second).children = vec![navigation_history::ChildHistoryItemSnapshot {
            unique_name: "child".into(),
            item: frame("https://child.test/second"),
        }]
        .into();
        tab.commit_child_history(second, FrameLoadType::Standard);
        let second = tab.history[1].clone();
        assert_eq!(first.item_sequence_number, second.item_sequence_number);
        assert_ne!(first.session_entry_id, second.session_entry_id);
        assert_eq!((tab.history_index, tab.history.len()), (1, 2));

        let mut request = url_loader::URLRequest::default();
        tab.prepare_navigation(&mut request, FrameLoadType::BackForward);
        request.navigation.as_mut().unwrap().history_item = Some(first.clone());
        tab.prepare_navigation(&mut request, FrameLoadType::BackForward);
        assert!(request.navigation.as_ref().unwrap().back_entries.is_empty());
        assert_eq!(
            request.navigation.as_ref().unwrap().forward_entries[0].session_entry_id,
            second.session_entry_id
        );
        tab.begin_navigation(2, &request);
        assert_eq!(tab.current_history_index(), 0);
        tab.discard_pending_navigation(2);

        tab.commit_child_history(first.clone(), FrameLoadType::BackForward);
        assert_eq!(tab.history_index, 0);
        tab.commit_child_history(first.clone(), FrameLoadType::ReplaceCurrentItem);
        assert_eq!((tab.history_index, tab.history.len()), (0, 2));
        assert_eq!(tab.history[0].session_entry_id, first.session_entry_id);
        tab.commit_child_history(second, FrameLoadType::BackForward);
        assert_eq!(tab.history_index, 1);
        tab.commit_child_history(first.clone(), FrameLoadType::BackForward);
        let mut replacement = first.clone();
        Arc::make_mut(&mut replacement).children =
            vec![navigation_history::ChildHistoryItemSnapshot {
                unique_name: "child".into(),
                item: frame("https://child.test/new"),
            }]
            .into();
        tab.commit_child_history(replacement, FrameLoadType::Standard);
        assert_eq!((tab.history_index, tab.history.len()), (1, 2));
        assert_eq!(
            tab.history[0].children[0].item.url,
            "https://child.test/first"
        );
        assert_eq!(
            tab.history[1].children[0].item.url,
            "https://child.test/new"
        );
    }
    #[test]
    fn committed_snapshots_preserve_identity_across_navigation_and_traversal() {
        let snapshot = |url: &str| {
            let owner = navigation_history::HistoryItemOwner::Create();
            owner.SetURL(url.into());
            owner.Snapshot()
        };
        let first = snapshot("https://first.test");
        let second = snapshot("https://second.test");
        let mut tab = Tab::<Box<String>>::new(1);
        tab.commit_history(first.clone(), FrameLoadType::Standard);
        let first = tab.history[0].clone();
        tab.commit_history(second.clone(), FrameLoadType::Standard);
        let second = tab.history[1].clone();

        let mut request = url_loader::URLRequest::default();
        tab.prepare_navigation(&mut request, FrameLoadType::Standard);
        let metadata = request.navigation.as_ref().unwrap();
        assert!(Arc::ptr_eq(&metadata.back_entries[0], &first));
        assert!(Arc::ptr_eq(&metadata.back_entries[1], &second));
        assert!(metadata.forward_entries.is_empty());

        let mut traversal = url_loader::URLRequest::default();
        tab.prepare_navigation(&mut traversal, FrameLoadType::BackForward);
        traversal.navigation.as_mut().unwrap().history_item = Some(first.clone());
        tab.prepare_navigation(&mut traversal, FrameLoadType::BackForward);
        let metadata = traversal.navigation.as_ref().unwrap();
        assert!(metadata.back_entries.is_empty());
        assert!(Arc::ptr_eq(&metadata.forward_entries[0], &second));
        tab.commit_history(first.clone(), FrameLoadType::BackForward);
        assert_eq!(tab.history_index, 0);

        let replacement = navigation_history::HistoryItemOwner::Restore(&first);
        replacement.SetURL("https://first.test#replacement".into());
        let replacement = replacement.Snapshot();
        tab.commit_history(replacement.clone(), FrameLoadType::ReplaceCurrentItem);
        let replacement = tab.history[0].clone();
        assert_eq!(first.url, "https://first.test");
        assert!(Arc::ptr_eq(&tab.history[0], &replacement));
        assert_eq!(tab.history.len(), 2);

        let third = snapshot("https://third.test");
        tab.commit_history(third.clone(), FrameLoadType::Standard);
        let third = tab.history[1].clone();
        assert_eq!(tab.history.len(), 2);
        assert!(Arc::ptr_eq(&tab.history[1], &third));
        let mut reload = url_loader::URLRequest::default();
        tab.prepare_navigation(&mut reload, FrameLoadType::Reload);
        let metadata = reload.navigation.as_ref().unwrap();
        assert!(Arc::ptr_eq(metadata.history_item.as_ref().unwrap(), &third));
        assert!(Arc::ptr_eq(&metadata.back_entries[0], &replacement));
        assert!(metadata.forward_entries.is_empty());

        let refreshed = navigation_history::HistoryItemOwner::Restore(&third);
        refreshed.SetDocumentState(vec![std::rc::Rc::<[u16]>::from([65u16])]);
        let refreshed = refreshed.Snapshot();
        tab.update_history_state(refreshed.clone());
        let refreshed = tab.history[1].clone();
        assert_eq!(tab.history_index, 1);
        assert_eq!(tab.history.len(), 2);
        tab.prepare_navigation(&mut reload, FrameLoadType::Reload);
        assert!(Arc::ptr_eq(
            reload
                .navigation
                .as_ref()
                .unwrap()
                .history_item
                .as_ref()
                .unwrap(),
            &refreshed,
        ));

        tab.page = Some(Box::new("third committed document".into()));
        let committed_identity = tab.page.as_deref().unwrap() as *const String;
        tab.location = third.url.clone();
        tab.title = "Third document".into();
        let mut new_navigation = url_loader::URLRequest {
            url: "https://slow.test".into(),
            ..Default::default()
        };
        tab.prepare_navigation(&mut new_navigation, FrameLoadType::Standard);
        tab.begin_navigation(1, &new_navigation);
        tab.pending_navigation.as_mut().unwrap().page =
            Some(Box::new("uncommitted new document".into()));
        tab.location = new_navigation.url;
        tab.loading = true;
        assert_eq!(tab.history.len(), 2);
        assert_eq!(tab.current_history_index(), 1);
        assert_eq!(tab.history_index_for_offset(-1), Some(0));
        assert_eq!(tab.history_index_for_offset(1), None);

        // Replacing a pending new URL with Back uses an existing entry index,
        // while retaining the original committed Page for cancellation.
        tab.begin_navigation(2, &traversal);
        tab.pending_navigation.as_mut().unwrap().page =
            Some(Box::new("uncommitted traversal document".into()));
        tab.update_history_state(refreshed);
        assert_eq!(tab.pending_navigation.as_ref().unwrap().generation, 2);
        assert_eq!(tab.history_index, 1);
        assert_eq!(tab.current_history_index(), 0);
        assert_eq!(tab.history_index_for_offset(-1), None);
        assert_eq!(tab.history_index_for_offset(1), Some(1));
        assert!(!tab.discard_pending_navigation(1));
        assert!(tab.commit_pending_navigation(1).is_none());
        assert_eq!(
            tab.page.as_deref().unwrap() as *const String,
            committed_identity
        );
        assert!(tab.discard_pending_navigation(2));
        assert_eq!(tab.location, third.url);
        assert_eq!(tab.title, "Third document");
        assert!(!tab.loading);
        assert_eq!(tab.current_history_index(), 1);
        assert_eq!(tab.document_generation, 2);
        assert_eq!(
            tab.page.as_deref().unwrap() as *const String,
            committed_identity
        );
        assert_eq!(tab.history.len(), 2);

        tab.begin_navigation(3, &traversal);
        tab.pending_navigation.as_mut().unwrap().page =
            Some(Box::new("first committed document".into()));
        let previous = tab.commit_pending_navigation(3).unwrap();
        assert_eq!(previous.as_ref() as *const String, committed_identity);
        tab.commit_history(replacement, FrameLoadType::BackForward);
        assert_eq!(tab.history_index, 0);
        assert_eq!(tab.history.len(), 2);
        assert!(tab.pending_navigation.is_none());
    }

    #[test]
    fn inactive_tab_retains_owned_document_history_and_order() {
        let mut tabs = Tabs::<Box<String>>::new();
        tabs.active.page = Some(Box::new("first document".into()));
        let identity = tabs.active.page.as_ref().unwrap().as_ref() as *const String;
        let item = navigation_history::HistoryItemOwner::Create();
        item.SetURL("https://first.test".into());
        tabs.active.history = vec![item.Snapshot()];
        tabs.active.location = "https://first.test".into();
        let second = tabs.add();
        tabs.active.page = Some(Box::new("second document".into()));
        assert!(tabs.activate(1));
        assert_eq!(
            tabs.active.page.as_ref().unwrap().as_ref() as *const String,
            identity
        );
        assert_eq!(tabs.active.history[0].url, "https://first.test");
        assert_eq!(tabs.active.location, "https://first.test");
        assert_eq!(
            tabs.ordered().iter().map(|tab| tab.id).collect::<Vec<_>>(),
            [1, second]
        );
        assert!(!tabs.activate(999));
        assert_eq!(tabs.active.id, 1);
        assert!(tabs.close(1));
        assert_eq!(tabs.active.id, second);
        assert_eq!(
            tabs.active.page.as_deref().map(String::as_str),
            Some("second document")
        );
        assert!(tabs.is_only(second));
        assert!(!tabs.close(second));
        assert_eq!(tabs.active.id, second);
        assert_eq!(
            tabs.active.page.as_deref().map(String::as_str),
            Some("second document")
        );
        assert!(tabs.background.is_empty());
    }
}
