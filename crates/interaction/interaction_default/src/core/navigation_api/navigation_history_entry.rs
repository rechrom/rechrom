//! Chromium navigation_history_entry.cc:23-98. Entry identity and serialized
//! state belong to this native EventTarget; the Window lends its current loader.
use super::navigation_api::{
    HistoryItem, NavigationApi, NavigationHistoryEntry, SerializedScriptValue,
};
use crate::bindings::core::v8::script_promise::ScriptNativeObject;
use interaction_dispatch::core::dom::events::event_target::EventTarget;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

pub trait NavigationHistoryEntryWindow {
    fn HasWindow(&self) -> bool;
    fn NavigationApi(&self) -> Rc<NavigationApi>;
    fn CurrentHistoryItem(&self) -> Rc<HistoryItem>;
    fn NotifyCurrentHistoryItemChanged(&self);
    fn EntryWindow(&self, identity: u64) -> Rc<dyn NavigationHistoryEntryWindow>;
    fn CreateEntryEventTarget(&self) -> Rc<EventTarget>;
}
pub struct NavigationHistoryEntryOwner {
    window: Weak<dyn NavigationHistoryEntryWindow>,
    history_item: Rc<HistoryItem>,
    key: String,
    id: String,
    url: String,
    document_sequence_number: i64,
    state: RefCell<Option<Rc<dyn SerializedScriptValue>>>,
    event_target: Rc<EventTarget>,
}
impl NavigationHistoryEntryOwner {
    pub fn new(window: Rc<dyn NavigationHistoryEntryWindow>, item: &Rc<HistoryItem>) -> Rc<Self> {
        Rc::new(Self {
            event_target: window.CreateEntryEventTarget(),
            window: Rc::downgrade(&window),
            history_item: item.clone(),
            key: item.NavigationApiKey(),
            id: item.NavigationApiId(),
            url: item.URL(),
            document_sequence_number: item.DocumentSequenceNumber(),
            state: RefCell::new(item.NavigationApiState()),
        })
    }
    pub fn HistoryItem(&self) -> Rc<HistoryItem> {
        self.history_item.clone()
    }
    fn Window(&self) -> Option<Rc<dyn NavigationHistoryEntryWindow>> {
        self.window.upgrade().filter(|window| window.HasWindow())
    }
}
impl NavigationHistoryEntry for NavigationHistoryEntryOwner {
    fn Index(&self) -> i32 {
        self.Window().map_or(-1, |window| {
            window
                .NavigationApi()
                .GetIndexForIdentity(&self.key, self.ScriptIdentity())
        })
    }
    fn Url(&self) -> Option<String> {
        self.Window()
            .map(|_| self.url.clone())
            .filter(|url| !url.is_empty())
    }
    fn SameDocument(&self) -> bool {
        self.Window().is_some_and(|window| {
            window.CurrentHistoryItem().DocumentSequenceNumber() == self.document_sequence_number
        })
    }
    fn SerializedState(&self) -> Option<Rc<dyn SerializedScriptValue>> {
        self.Window().and_then(|_| self.state.borrow().clone())
    }
    fn SetAndSaveState(&self, state: Option<Rc<dyn SerializedScriptValue>>) {
        *self.state.borrow_mut() = state.clone();
        let Some(window) = self.Window() else {
            return;
        };
        if window
            .NavigationApi()
            .CurrentEntry()
            .is_some_and(|entry| entry.ScriptIdentity() == self.ScriptIdentity())
        {
            window.CurrentHistoryItem().SetNavigationApiState(state);
            window.NotifyCurrentHistoryItemChanged();
        }
    }
    fn Key(&self) -> String {
        self.Window().map_or_else(String::new, |_| self.key.clone())
    }
    fn Id(&self) -> String {
        self.Window().map_or_else(String::new, |_| self.id.clone())
    }
    fn CloneForWindow(&self, identity: u64) -> Rc<dyn NavigationHistoryEntry> {
        let window = self
            .window
            .upgrade()
            .expect("entry clone has live source Window")
            .EntryWindow(identity);
        Rc::new(Self {
            event_target: window.CreateEntryEventTarget(),
            window: Rc::downgrade(&window),
            history_item: self.history_item.clone(),
            key: self.key.clone(),
            id: self.id.clone(),
            url: self.url.clone(),
            document_sequence_number: self.document_sequence_number,
            state: RefCell::new(self.state.borrow().clone()),
        })
    }
}
impl ScriptNativeObject for NavigationHistoryEntryOwner {
    fn AsAny(&self) -> &dyn std::any::Any {
        self
    }
    fn ScriptInterfaceName(&self) -> &'static str {
        "NavigationHistoryEntry"
    }
    fn EventTarget(&self) -> Option<Rc<EventTarget>> {
        Some(self.event_target.clone())
    }
}
