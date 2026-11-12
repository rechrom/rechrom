// Copyright The Chromium Authors. BSD license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/dom/popover_data.h:25-154; html_dialog_element.h:42-148;
// closewatcher/close_watcher.h:36-119; document.h popover/dialog stack members.
// Canonical DOM rare data. Interaction's source lifecycle mutates these exact
// owners through narrow typed primitives, releasing DOM borrows before events.
use foundation::popup_types::*;
use std::collections::{HashMap, HashSet};
#[derive(Default)]
pub struct PopoverData {
    pub visibility_state: PopoverVisibilityState,
    pub popover_type: PopoverValueType,
    pub invoker: Option<u64>,
    pub previously_focused_element: Option<u64>,
    pub pending_toggle_event_task: Option<PopupTaskId>,
    pub pending_toggle_event_started_closed: bool,
    pub hiding_this_popover: bool,
    pub hover_show_tasks: HashMap<u64, PopupTaskId>,
    pub hover_hide_task: Option<PopupTaskId>,
    pub implicit_anchor: Option<u64>,
    pub close_watcher: Option<CloseWatcherId>,
}
pub struct DialogData {
    pub is_modal: bool,
    pub is_closing: bool,
    pub return_value: Vec<u16>,
    pub request_close_return_value: Option<Vec<u16>>,
    pub request_close_source_element: Option<u64>,
    pub previously_focused_element: Option<u64>,
    pub close_watcher: Option<CloseWatcherId>,
    pub pending_toggle_event: Option<PopupToggleEvent>,
    pub pending_toggle_event_task: Option<PopupTaskId>,
}
impl Default for DialogData {
    fn default() -> Self {
        Self {
            is_modal: false,
            is_closing: false,
            return_value: Vec::new(),
            request_close_return_value: Some(Vec::new()),
            request_close_source_element: None,
            previously_focused_element: None,
            close_watcher: None,
            pending_toggle_event: None,
            pending_toggle_event_task: None,
        }
    }
}
pub struct CloseWatcherData {
    pub script_owner: Option<std::rc::Rc<dyn std::any::Any>>,
    pub closed: bool,
    pub dispatching_cancel: bool,
    pub enabled: bool,
    pub abort_algorithm: Option<u64>,
    pub owner: CloseWatcherOwner,
}
impl CloseWatcherData {
    pub fn new(owner: CloseWatcherOwner) -> Self {
        Self {
            script_owner: None,
            closed: false,
            dispatching_cancel: false,
            enabled: true,
            abort_algorithm: None,
            owner,
        }
    }
}
pub struct CloseWatcherStackData {
    pub watcher_groups: Vec<Vec<CloseWatcherId>>,
    pub next_user_interaction_creates_a_new_allowed_group: bool,
    pub allowed_groups: usize,
    pub close_listener: Option<std::rc::Rc<dyn Fn()>>,
    pub close_signal_host: Option<std::rc::Rc<dyn Fn()>>,
}
impl Default for CloseWatcherStackData {
    fn default() -> Self {
        Self {
            watcher_groups: Vec::new(),
            next_user_interaction_creates_a_new_allowed_group: true,
            allowed_groups: 1,
            close_listener: None,
            close_signal_host: None,
        }
    }
}
#[derive(Default)]
pub struct PopupLifecycleState {
    /// Embedding-injected task/lifecycle service handle. The concrete source
    /// operations remain in Interaction; no mutable popup state is duplicated.
    pub pointer_dom_services: Option<std::rc::Rc<dyn std::any::Any>>,
    pub dialog_pointerdown_target: Option<u64>,
    pub popover_pointerdown_target: Option<u64>,
    pub interest_invokers: HashMap<u64, InterestInvokerData>,
    pub interest_targets: HashMap<u64, u64>,
    pub elements_with_interest: Vec<u64>,
    pub menu_item_owners: HashMap<u64, Option<u64>>,
    pub menu_elements_enabled: bool,
    pub menu_triangles: HashMap<MenuTriangleId, MenuSafeTriangleData>,
    pub active_menu_triangle: Option<MenuTriangleId>,
    pub next_menu_triangle_id: u64,
    pub popovers: HashMap<u64, PopoverData>,
    pub may_be_implicit_anchors: HashSet<u64>,
    pub autofocus_candidates: Vec<u64>,
    pub autofocus_processed_flag: bool,
    pub dialogs: HashMap<u64, DialogData>,
    pub auto_stack: Vec<u64>,
    pub hint_stack: Vec<u64>,
    pub hint_stack_parent: Option<u64>,
    pub popover_showing: bool,
    pub popover_hiding_nesting_count: usize,
    pub all_open_dialogs: Vec<u64>,
    pub top_layer_pending_removal: Vec<(u64, PopupTopLayerReason)>,
    pub close_watcher_stack: CloseWatcherStackData,
    pub close_watchers: HashMap<CloseWatcherId, CloseWatcherData>,
    pub next_close_watcher_id: u64,
    // HTMLCollection::CollectionItemsCache, invalidated by canonical DOM revisions.
    pub invoker_collections: HashMap<(u64, InvokerCollectionKind), InvokerCollectionCache>,
    // InspectorDOMAgent owns weak node sets; canonical DOM node identities
    // remain valid while this document's arena lives.
    pub inspector_forced_popovers: HashSet<u64>,
    pub inspector_forced_interest_invokers: HashSet<u64>,
}
impl PopupLifecycleState {
    pub fn CreateCloseWatcher(&mut self, owner: CloseWatcherOwner) -> CloseWatcherId {
        self.next_close_watcher_id = self
            .next_close_watcher_id
            .checked_add(1)
            .expect("CloseWatcher identity exhausted");
        let id = CloseWatcherId(self.next_close_watcher_id);
        self.close_watchers.insert(id, CloseWatcherData::new(owner));
        id
    }
}
// Source: InvokerData and InterestInvokerTargetData. These relations remain
// on the canonical elements, distinct from popover invoker associations.
#[derive(Default)]
pub struct InterestInvokerData {
    pub state: InterestState,
    pub active_target: Option<u64>,
    pub gained_task: Option<PopupTaskId>,
    pub lost_task: Option<PopupTaskId>,
    pub suppress_next_focus_interest: bool,
}
pub struct MenuSafeTriangleData {
    pub invoker: Option<u64>,
    pub submenu: Option<u64>,
    pub triangle: foundation::gfx::QuadF,
    pub submenu_quad: foundation::gfx::QuadF,
    pub deferred_gains: Vec<DeferredInterestGain>,
    pub deferred_losses: Vec<DeferredInterestLoss>,
    pub expire_task: Option<PopupTaskId>,
}

#[derive(Default)]
pub struct InvokerCollectionCache {
    pub revision: Option<u64>,
    pub items: Vec<u64>,
}
