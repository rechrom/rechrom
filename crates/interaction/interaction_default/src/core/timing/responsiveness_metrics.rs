//! Chromium responsiveness_metrics.h:36-218; .cc:73-575,663-701.
//! Interaction grouping owns the original entry handles, including unresolved
//! pointerdowns. Metric transport receives immutable source results only.
use super::{
    performance_event_timing::{PerformanceEventTiming as Entry, UserInteractionType},
    performance_timeline_entry_id_generator::{
        PerformanceRandom, PerformanceTimelineEntryIdGenerator,
        PerformanceTimelineEntryIdInfo as Id,
    },
};
use interaction_input::base::time::time::TimeTicks;
use std::{collections::HashMap, rc::Rc};
#[derive(Clone, Copy, PartialEq, Eq)]
enum CompositionState {
    NonComposition,
    Active,
    EndOnKeyup,
}
#[derive(Clone)]
pub struct UserInteractionMetric {
    pub interaction_id: Id,
    pub interaction_type: UserInteractionType,
    pub creation_time: TimeTicks,
    pub queued_time: TimeTicks,
    pub processing_start: TimeTicks,
    pub commit_finish: TimeTicks,
    pub end_time: TimeTicks,
    pub frame_index: u64,
    pub unique_in_frame: bool,
}
pub struct ResponsivenessMetrics {
    composition_state: CompositionState,
    keycode_to_interactionid: HashMap<i32, Id>,
    last_keydown_interaction_id: Option<Id>,
    last_navigate_interaction_id: Id,
    pending_pointerdown_entries: HashMap<i32, Rc<Entry>>,
    pointerid_to_interactionid: HashMap<i32, Id>,
    current_interaction_event_queued_timestamp: TimeTicks,
    interaction_id_generator: PerformanceTimelineEntryIdGenerator,
    navigation_interaction_count: u64,
    last_recorded_frame_index: Option<u64>,
    reported_interactions_in_frame: Vec<(u64, TimeTicks)>,
}
impl ResponsivenessMetrics {
    pub fn new(random: Rc<dyn PerformanceRandom>) -> Self {
        Self {
            composition_state: CompositionState::NonComposition,
            keycode_to_interactionid: HashMap::new(),
            last_keydown_interaction_id: None,
            last_navigate_interaction_id: Id::NONE,
            pending_pointerdown_entries: HashMap::new(),
            pointerid_to_interactionid: HashMap::new(),
            current_interaction_event_queued_timestamp: TimeTicks::new(),
            interaction_id_generator: PerformanceTimelineEntryIdGenerator::new(random),
            navigation_interaction_count: 0,
            last_recorded_frame_index: None,
            reported_interactions_in_frame: Vec::new(),
        }
    }
    pub fn WillNavigateEventCommitNow(&mut self, id: Id) {
        self.last_navigate_interaction_id = id;
    }
    pub fn CurrentInteractionEventQueuedTimestamp(&self) -> TimeTicks {
        self.current_interaction_event_queued_timestamp
    }
    pub fn SetCurrentInteractionEventQueuedTimestamp(&mut self, time: TimeTicks) {
        self.current_interaction_event_queued_timestamp = time;
    }
    pub fn TryAssignInteractionId(&mut self, entry: &Rc<Entry>, topmost: Option<&Rc<Entry>>) {
        assert!(!entry.HasInteractionId());
        let name = entry.name.as_str();
        if !matches!(
            name,
            "pointercancel"
                | "contextmenu"
                | "pointerdown"
                | "pointerup"
                | "click"
                | "keydown"
                | "keypress"
                | "keyup"
                | "compositionstart"
                | "compositionupdate"
                | "compositionend"
                | "input"
                | "navigate"
                | "popstate"
                | "hashchange"
        ) {
            entry.SetInteractionIdInfo(Id::NONE);
            return;
        }
        let info = *entry.reporting_info.borrow();
        if matches!(
            name,
            "compositionstart" | "compositionupdate" | "compositionend" | "input"
        ) {
            self.HandleCompositionInteraction(entry);
            return;
        }
        if info.key_code.is_some() {
            if self.composition_state != CompositionState::NonComposition {
                self.HandleCompositionInteraction(entry);
            } else {
                self.HandleKeyboardInteraction(entry);
            }
            return;
        }
        if info.pointer_id.is_some() || name == "click" {
            self.HandlePointerInteraction(entry, topmost);
            return;
        }
        if matches!(name, "navigate" | "popstate" | "hashchange") {
            self.HandleNavigationInteraction(entry, topmost);
            return;
        }
        entry.SetInteractionIdInfo(Id::NONE);
    }
    fn AssignNewKeyboardInteractionId(&mut self, key: i32) -> Id {
        let id = self.interaction_id_generator.IncrementId();
        self.keycode_to_interactionid.insert(key, id);
        self.last_keydown_interaction_id = Some(id);
        id
    }
    fn AssignNewPointerInteractionId(&mut self, pointer: i32) -> Id {
        let id = self.interaction_id_generator.IncrementId();
        self.pointerid_to_interactionid.insert(pointer, id);
        id
    }
    fn HandleKeyboardInteraction(&mut self, entry: &Rc<Entry>) {
        let key = entry
            .reporting_info
            .borrow()
            .key_code
            .expect("keyboard reporting info");
        if entry.name == "keydown" {
            entry.SetInteractionIdInfo(self.AssignNewKeyboardInteractionId(key));
            return;
        }
        let mut id = self
            .keycode_to_interactionid
            .get(&key)
            .copied()
            .unwrap_or(Id::NONE);
        if entry.name == "keyup" {
            entry.SetInteractionIdInfo(id);
            self.keycode_to_interactionid.remove(&key);
            return;
        }
        if entry.name == "keypress" {
            if let Some(last) = self.last_keydown_interaction_id {
                id = last;
            }
            entry.SetInteractionIdInfo(id);
            return;
        }
        entry.SetInteractionIdInfo(Id::NONE);
    }
    fn HandleNavigationInteraction(&mut self, entry: &Rc<Entry>, topmost: Option<&Rc<Entry>>) {
        if entry.name == "navigate" {
            let id = if let Some(top) = topmost.filter(|top| top.IsInteraction()) {
                top.GetInteractionIdInfo().expect("interaction has ID")
            } else {
                self.navigation_interaction_count += 1;
                self.interaction_id_generator.IncrementId()
            };
            entry.SetInteractionIdInfo(id);
            self.last_navigate_interaction_id =
                entry.GetInteractionIdInfo().expect("navigation ID");
            return;
        }
        if self.last_navigate_interaction_id == Id::NONE {
            if let Some(top) = topmost.filter(|top| top.IsInteraction()) {
                self.last_navigate_interaction_id =
                    top.GetInteractionIdInfo().expect("interaction ID");
            }
        }
        entry.SetInteractionIdInfo(self.last_navigate_interaction_id);
    }
    fn HandleCompositionInteraction(&mut self, entry: &Rc<Entry>) {
        let name = entry.name.as_str();
        if self.composition_state == CompositionState::NonComposition && name != "compositionstart"
        {
            entry.SetInteractionIdInfo(Id::NONE);
            return;
        }
        match name {
            "compositionstart" => {
                self.composition_state = CompositionState::Active;
                entry.SetInteractionIdInfo(Id::NONE);
                return;
            }
            "compositionupdate" => {
                entry.SetInteractionIdInfo(Id::NONE);
                return;
            }
            "compositionend" => {
                self.composition_state = CompositionState::EndOnKeyup;
                entry.SetInteractionIdInfo(Id::NONE);
                return;
            }
            _ => {}
        }
        if self.composition_state == CompositionState::EndOnKeyup
            && !matches!(name, "input" | "keyup")
        {
            self.composition_state = CompositionState::NonComposition;
            assert!(entry.reporting_info.borrow().key_code.is_some());
            self.HandleKeyboardInteraction(entry);
            return;
        }
        match name {
            "keydown" => {
                assert!(self.composition_state == CompositionState::Active);
                let key = entry
                    .reporting_info
                    .borrow()
                    .key_code
                    .expect("composition keydown key");
                let id = self.AssignNewKeyboardInteractionId(key);
                self.last_keydown_interaction_id = Some(id);
                entry.SetInteractionIdInfo(id);
            }
            "keypress" => {
                assert!(self.composition_state == CompositionState::Active);
                entry.SetInteractionIdInfo(self.interaction_id_generator.GetValue());
            }
            "keyup" => {
                entry.SetInteractionIdInfo(self.interaction_id_generator.GetValue());
                if self.composition_state == CompositionState::EndOnKeyup {
                    self.composition_state = CompositionState::NonComposition;
                }
                self.last_keydown_interaction_id = None;
            }
            "input" => {
                let id = self
                    .last_keydown_interaction_id
                    .unwrap_or_else(|| self.interaction_id_generator.IncrementId());
                entry.SetInteractionIdInfo(id);
                self.last_keydown_interaction_id = None;
            }
            _ => entry.SetInteractionIdInfo(Id::NONE),
        }
    }
    fn HandlePointerInteraction(&mut self, entry: &Rc<Entry>, topmost: Option<&Rc<Entry>>) {
        let pointer = entry
            .reporting_info
            .borrow()
            .pointer_id
            .expect("source click/contextmenu are PointerEvents");
        let pending = if pointer != -1 {
            self.pending_pointerdown_entries.get(&pointer).cloned()
        } else {
            None
        };
        let mut id = if pointer != -1 && pending.is_none() {
            self.pointerid_to_interactionid.get(&pointer).copied()
        } else {
            None
        };
        if pending.is_some() {
            assert!(!self.pointerid_to_interactionid.contains_key(&pointer));
        }
        match entry.name.as_str() {
            "pointerdown" => {
                if let Some(pending) = pending {
                    pending.SetInteractionIdInfo(Id::NONE);
                }
                assert_ne!(pointer, -1);
                self.pending_pointerdown_entries
                    .insert(pointer, entry.clone());
                self.pointerid_to_interactionid.remove(&pointer);
            }
            "pointerup" | "contextmenu" => {
                if let Some(pending) = pending {
                    id = Some(self.AssignNewPointerInteractionId(pointer));
                    pending.SetInteractionIdInfo(id.unwrap());
                    self.pending_pointerdown_entries.remove(&pointer);
                }
                entry.SetInteractionIdInfo(if entry.name == "contextmenu" {
                    Id::NONE
                } else {
                    id.unwrap_or(Id::NONE)
                });
            }
            "click" => {
                if pointer == -1 {
                    if let Some(last) = self.last_keydown_interaction_id.take() {
                        entry.SetInteractionIdInfo(last);
                        return;
                    }
                }
                if let Some(id) = id {
                    entry.SetInteractionIdInfo(id);
                    self.pointerid_to_interactionid.remove(&pointer);
                    return;
                }
                if let Some(pending) = pending {
                    let id = self.AssignNewPointerInteractionId(pointer);
                    pending.SetInteractionIdInfo(id);
                    entry.SetInteractionIdInfo(id);
                    self.pending_pointerdown_entries.remove(&pointer);
                    return;
                }
                if topmost.is_some_and(|top| top.IsInteraction() && top.name == "click") {
                    entry.SetInteractionIdInfo(Id::NONE);
                    return;
                }
                entry.SetInteractionIdInfo(self.interaction_id_generator.IncrementId());
            }
            "pointercancel" => {
                if let Some(pending) = pending {
                    pending.SetInteractionIdInfo(Id::NONE);
                    self.pending_pointerdown_entries.remove(&pointer);
                }
                self.pointerid_to_interactionid.remove(&pointer);
                entry.SetInteractionIdInfo(Id::NONE);
            }
            _ => entry.SetInteractionIdInfo(Id::NONE),
        }
    }
    pub fn CommitAllPendingPointerdowns(&mut self) {
        let pending = std::mem::take(&mut self.pending_pointerdown_entries);
        for entry in pending.values() {
            let pointer = entry
                .reporting_info
                .borrow()
                .pointer_id
                .expect("pointerdown ID");
            entry.SetInteractionIdInfo(self.AssignNewPointerInteractionId(pointer));
        }
    }
    pub fn FlushAllEvents(&mut self) {
        self.CommitAllPendingPointerdowns();
        self.composition_state = CompositionState::NonComposition;
        self.keycode_to_interactionid.clear();
        self.pointerid_to_interactionid.clear();
        self.last_keydown_interaction_id = None;
        self.last_navigate_interaction_id = Id::NONE;
        self.reported_interactions_in_frame.clear();
        self.last_recorded_frame_index = None;
    }
    pub fn GetInteractionCount(&self, navigation_event_timing_enabled: bool) -> u64 {
        let count = self.interaction_id_generator.GetValue().non_web_exposed_id - 1;
        if navigation_event_timing_enabled {
            count
        } else {
            count - self.navigation_interaction_count
        }
    }
    /// Reporting transports are separate from interaction assignment. This
    /// output is emitted only after real presentation/fallback gives an end.
    pub fn ReportToMetrics(&mut self, entry: &Rc<Entry>) -> Option<UserInteractionMetric> {
        assert!(entry.IsReadyForReporting());
        if !entry.IsInteraction() {
            return None;
        }
        let info = *entry.reporting_info.borrow();
        let id = entry.GetInteractionIdInfo().expect("assigned");
        let end = entry.GetEndTime();
        if self.last_recorded_frame_index != Some(info.frame_index) {
            self.last_recorded_frame_index = Some(info.frame_index);
            self.reported_interactions_in_frame.clear();
        }
        let key = (id.web_exposed_id, end);
        let unique = !self.reported_interactions_in_frame.contains(&key);
        if unique {
            self.reported_interactions_in_frame.push(key);
        }
        Some(UserInteractionMetric {
            interaction_id: id,
            interaction_type: entry.InteractionType(),
            creation_time: info.creation_time,
            queued_time: info.enqueued_to_main_thread_time,
            processing_start: info.processing_start_time,
            commit_finish: info.commit_finish_time,
            end_time: end,
            frame_index: info.frame_index,
            unique_in_frame: unique,
        })
    }
}
