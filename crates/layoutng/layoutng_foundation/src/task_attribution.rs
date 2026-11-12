//! Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! task_attribution_info_impl.{h,cc}; task_attribution_task_state.cc:24-76;
//! task_attribution_tracker_impl.cc:144-307; top_level_override_scope.cc:15-42.
//! The opaque slot is the VM's actual continuation-preserved embedder data.
use std::{
    any::Any,
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::{Rc, Weak},
    sync::atomic::{AtomicU32, Ordering},
};
pub type ContinuationData = Option<Rc<dyn Any>>;
pub type ContinuationSlot = Rc<RefCell<ContinuationData>>;
static NEXT_ID: AtomicU32 = AtomicU32::new(1);
pub trait TaskAttributionTaskState: Any {
    fn GetTaskAttributionInfo(&self) -> Option<Rc<TaskAttributionInfo>>;
}
struct External(Rc<dyn TaskAttributionTaskState>);
pub struct TaskAttributionInfo {
    self_: Weak<Self>,
    id: u32,
    pub async_data_for_test: u32,
    pub soft_navigation_context: Option<Rc<dyn Any>>,
    pub resource_timing_context: Option<Rc<dyn Any>>,
    pub script_tool_context: Option<Rc<dyn Any>>,
}
impl TaskAttributionInfo {
    pub fn new(
        soft: Option<Rc<dyn Any>>,
        resource: Option<Rc<dyn Any>>,
        tool: Option<Rc<dyn Any>>,
        test: u32,
    ) -> Rc<Self> {
        Rc::new_cyclic(|self_| Self {
            self_: self_.clone(),
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            async_data_for_test: test,
            soft_navigation_context: soft,
            resource_timing_context: resource,
            script_tool_context: tool,
        })
    }
    pub fn Id(&self) -> u32 {
        self.id
    }
    pub fn ForkAndSetSoftNavigation(&self, value: Option<Rc<dyn Any>>) -> Rc<Self> {
        Self::new(
            value,
            self.resource_timing_context.clone(),
            self.script_tool_context.clone(),
            self.async_data_for_test,
        )
    }
    pub fn ForkAndSetResourceTiming(&self, value: Option<Rc<dyn Any>>) -> Rc<Self> {
        Self::new(
            self.soft_navigation_context.clone(),
            value,
            self.script_tool_context.clone(),
            self.async_data_for_test,
        )
    }
    pub fn ForkAndSetScriptTool(&self, value: Option<Rc<dyn Any>>) -> Rc<Self> {
        Self::new(
            self.soft_navigation_context.clone(),
            self.resource_timing_context.clone(),
            value,
            self.async_data_for_test,
        )
    }
}
impl TaskAttributionTaskState for TaskAttributionInfo {
    fn GetTaskAttributionInfo(&self) -> Option<Rc<TaskAttributionInfo>> {
        self.self_.upgrade()
    }
}
#[derive(Clone, Copy, Debug)]
pub enum TaskScopeType {
    Callback,
    ScheduledAction,
    ScriptExecution,
    PostMessage,
    PopState,
    SchedulerPostTask,
    RequestIdleCallback,
    XMLHttpRequest,
    SoftNavigation,
    ResourceTiming,
    MiscEvent,
    Microtask,
    ScriptToolExecution,
}
pub struct TaskAttributionTracker {
    self_: Weak<Self>,
    slot: ContinuationSlot,
    override_top_level: Cell<bool>,
    same_document: RefCell<VecDeque<Rc<TaskAttributionInfo>>>,
    nested: RefCell<Vec<ContinuationData>>,
}
impl TaskAttributionTracker {
    pub fn new(slot: ContinuationSlot) -> Rc<Self> {
        Rc::new_cyclic(|self_| Self {
            self_: self_.clone(),
            slot,
            override_top_level: Cell::new(false),
            same_document: Default::default(),
            nested: Default::default(),
        })
    }
    pub fn Slot(&self) -> ContinuationSlot {
        self.slot.clone()
    }
    pub fn CurrentTaskState(&self) -> Option<Rc<TaskAttributionInfo>> {
        self.slot
            .borrow()
            .as_ref()
            .and_then(|v| v.downcast_ref::<External>())
            .and_then(|v| v.0.GetTaskAttributionInfo())
    }
    pub fn SetCurrentTaskStateIfTopLevel(
        &self,
        state: Option<Rc<TaskAttributionInfo>>,
        in_context: bool,
        kind: TaskScopeType,
    ) -> Option<TaskScope> {
        let override_check = self.override_top_level.replace(false);
        let state = state?;
        if !override_check && in_context {
            return None;
        }
        Some(self.SetCurrentTaskState(state, kind))
    }
    pub fn SetCurrentTaskState(
        &self,
        state: Rc<dyn TaskAttributionTaskState>,
        kind: TaskScopeType,
    ) -> TaskScope {
        let previous = self.slot.borrow_mut().replace(Rc::new(External(state)));
        TaskScope {
            tracker: self.self_.clone(),
            previous,
            kind,
        }
    }
    pub fn OverrideTopLevelCheck(&self, do_override: bool) -> TopLevelOverrideScope {
        if do_override {
            self.override_top_level.set(true);
        }
        TopLevelOverrideScope {
            tracker: do_override.then(|| self.self_.clone()),
        }
    }
    pub fn AsyncSameDocumentNavigationStarted(&self) -> Option<u32> {
        let state = self.CurrentTaskState()?;
        let id = state.Id();
        self.same_document.borrow_mut().push_back(state);
        Some(id)
    }
    pub fn CommitSameDocumentNavigation(&self, id: u32) -> Option<Rc<TaskAttributionInfo>> {
        let mut tasks = self.same_document.borrow_mut();
        while let Some(state) = tasks.pop_front() {
            if state.Id() == id {
                return Some(state);
            }
        }
        None
    }
    pub fn ResetSameDocumentNavigationTasks(&self) {
        self.same_document.borrow_mut().clear();
    }
    pub fn OnBeginNestedRunLoop(&self) {
        let state = self.slot.borrow_mut().take();
        self.nested.borrow_mut().push(state);
    }
    pub fn OnExitNestedRunLoop(&self) {
        *self.slot.borrow_mut() = self
            .nested
            .borrow_mut()
            .pop()
            .expect("balanced nested run-loop task attribution");
    }
}
pub struct TaskScope {
    tracker: Weak<TaskAttributionTracker>,
    previous: ContinuationData,
    pub kind: TaskScopeType,
}
impl Drop for TaskScope {
    fn drop(&mut self) {
        if let Some(t) = self.tracker.upgrade() {
            *t.slot.borrow_mut() = self.previous.take();
        }
    }
}
pub struct TopLevelOverrideScope {
    tracker: Option<Weak<TaskAttributionTracker>>,
}
impl Drop for TopLevelOverrideScope {
    fn drop(&mut self) {
        if let Some(t) = self.tracker.as_ref().and_then(Weak::upgrade) {
            t.override_top_level.set(false);
        }
    }
}
