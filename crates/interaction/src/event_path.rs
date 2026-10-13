#![allow(non_snake_case)]

use crate::event::{Event, EventType};
use dom::{DOMEventPath, DOMEventPathEntry, Document};

// Blink EventPath and WindowEventContext adapter. DOM owns shadow/slot tree
// traversal and retargeting; Interaction adds only the non-Node Window endpoint
// and the event-type rule that node load events do not reach it.
#[derive(Clone, Debug)]
pub struct EventPath {
    dom: Option<DOMEventPath>,
    entries: Vec<DOMEventPathEntry>,
    final_target: Option<u64>,
    has_window: bool,
}
impl EventPath {
    pub fn new(document: &Document, target: usize, event: &Event) -> Self {
        let dom = DOMEventPath::new(document, target, event.composed);
        let mut entries = dom.Entries().to_vec();
        let load = if event.custom_type.is_empty() {
            event.r#type == EventType::kLoad
        } else {
            event.custom_type == "load"
        };
        let final_target = if load { None } else { dom.PostDispatchTarget() };
        let has_window = dom.ReachesDocument() && !load;
        if has_window {
            entries.push(DOMEventPathEntry {
                invocation_target: 0,
                shadow_adjusted_target: entries.last().unwrap().shadow_adjusted_target,
            });
        }
        Self {
            dom: Some(dom),
            entries,
            final_target,
            has_window,
        }
    }
    pub fn Window() -> Self {
        Self {
            dom: None,
            entries: vec![DOMEventPathEntry {
                invocation_target: 0,
                shadow_adjusted_target: 0,
            }],
            final_target: Some(0),
            has_window: true,
        }
    }
    pub fn Entries(&self) -> &[DOMEventPathEntry] {
        &self.entries
    }
    pub fn Nodes(&self) -> impl Iterator<Item = u64> + '_ {
        self.entries.iter().map(|entry| entry.invocation_target)
    }
    pub fn PostDispatchTarget(&self) -> Option<u64> {
        self.final_target
    }
    pub fn ComposedPath(&self, current_target: u64) -> Vec<u64> {
        let Some(dom) = &self.dom else {
            return if current_target == 0 {
                vec![0]
            } else {
                Vec::new()
            };
        };
        let observer = if current_target == 0 && self.has_window {
            dom.DocumentTarget().unwrap()
        } else {
            current_target
        };
        let mut result = dom.ComposedPath(observer);
        if !result.is_empty() && self.has_window {
            result.push(0);
        }
        result
    }
}

// Shared by retained Event wrappers, not by the DOM owner. Clearing the path at
// dispatch completion releases the snapshot and makes retained composedPath()
// calls empty even when the last callback was inside a closed shadow tree.
#[derive(Debug, Default)]
pub struct EventDispatchState {
    pub path: Option<std::rc::Rc<EventPath>>,
    pub final_target: Option<u64>,
    pub finished: bool,
}
