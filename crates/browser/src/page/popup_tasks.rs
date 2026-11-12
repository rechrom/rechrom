//! Page task-runner assembly for weak Document popup tasks.
//! Source: element.cc:12910-12977 (kMiscPlatformAPI delayed cancellable tasks),
//! html_dialog_element.cc:272-291 (live bounding-client-rect query).
//! Interaction owns the task bodies; Page selects one due ordinary task and
//! lends the current dispatcher only while that task runs.
use super::*;
use foundation::popup_types::PopupTaskId;
use interaction::adapter::page_event_dispatcher::gesture_input::{
    GestureTimerServices, GestureTimerTask,
};
use interaction::adapter::page_event_dispatcher::pointer_dom_effects::{
    InstallPopupLifecycleServices, PointerDomError, PopupLifecycleServices, PopupTask,
};
use interaction::adapter::page_event_handler_services::{
    EventHandlerTimerKind, EventHandlerTimerTask, PageEventHandlerTimerServices,
};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub(super) enum PageInteractionTask {
    // layout_shift_tracker.h:187: TimerFired is empty. Taking this ordinary
    // task expires the one-shot retained by LayoutShiftInputTracker.
    LayoutShiftInputTimer,
    EventHandler {
        frame: u64,
        document_generation: u64,
        task: EventHandlerTimerTask,
    },
    FormRestore(Rc<RefCell<Option<Box<dyn FnOnce()>>>>),
    FormNavigation(Rc<RefCell<Option<Box<dyn FnOnce()>>>>),
    AnchorNavigation(Rc<RefCell<Option<Box<dyn FnOnce()>>>>),
    SelectionAutoscroll(interaction::input::base::time::time::TimeTicks),

    DirectoryFilesChosen {
        target: u64,
        files: Result<Vec<Arc<fileapi::File>>, fileapi::NativeFileError>,
    },
    SystemDragTarget {
        event: Arc<interaction::input::public::common::input::web_mouse_event::WebMouseEvent>,
        action: interaction::default::core::page::drag_controller::DragTargetAction,
        result: Rc<
            Cell<Option<interaction::default::core::page::drag_controller::DragTargetOperation>>,
        >,
    },
    SystemDragEnd {
        event: Arc<interaction::input::public::common::input::web_mouse_event::WebMouseEvent>,
        operation: interaction::dispatch::core::clipboard::data_transfer::DragOperation,
    },
    Popup(PopupTask),
    Gesture(GestureTimerTask),
    Scrollbar(interaction::adapter::page_pointer_scroll_controls::ScrollbarTimerTask),
    ChildScrollbar {
        frame: u64,
        generation: u64,
        task: interaction::adapter::page_pointer_scroll_controls::ScrollbarTimerTask,
    },
}

pub(super) fn PopupMutationError(error: PointerDomError) -> dom::dom_mutation::DOMMutationError {
    // Preserve the native diagnostic while the established DOM mutation ABI
    // carries its static DOMException message to the script binding.
    eprintln!("rechrom-popup-error\t{error}");
    dom::error::DOMException {
        kind: dom::error::DOMExceptionKind::InvalidStateError,
        message: "popup lifecycle mutation failed",
    }
}

struct ScheduledPopupTask {
    document: dom::persistent_document::DOMOwnerHandle,
    deadline: Option<Instant>,
    task: PageInteractionTask,
}

#[derive(Default)]
pub(super) struct PagePopupTasks {
    next_id: Cell<u64>,
    tasks: RefCell<HashMap<u64, ScheduledPopupTask>>,
    ready_order: RefCell<BTreeMap<(Instant, u64), ()>>,
}
impl PagePopupTasks {
    pub(super) fn IsScheduled(&self, task: PopupTaskId) -> bool {
        self.tasks.borrow().contains_key(&task.0)
    }
    pub(super) fn Post(
        &self,
        document: dom::persistent_document::DOMOwnerHandle,
        delay_seconds: f64,
        task: PageInteractionTask,
    ) -> PopupTaskId {
        assert!(delay_seconds.is_finite() && delay_seconds >= 0.0);
        let id = self
            .next_id
            .get()
            .checked_add(1)
            .expect("popup task id exhausted");
        self.next_id.set(id);
        // A CSS delay can exceed the host Instant range. Such a source task
        // remains cancellable but cannot become due in this clock domain.
        let deadline = Duration::try_from_secs_f64(delay_seconds)
            .ok()
            .and_then(|delay| Instant::now().checked_add(delay));
        self.tasks.borrow_mut().insert(
            id,
            ScheduledPopupTask {
                document,
                deadline,
                task,
            },
        );
        if let Some(deadline) = deadline {
            self.ready_order.borrow_mut().insert((deadline, id), ());
        }
        PopupTaskId(id)
    }
    pub(super) fn Cancel(&self, id: PopupTaskId) {
        if let Some(task) = self.tasks.borrow_mut().remove(&id.0) {
            if let Some(deadline) = task.deadline {
                self.ready_order.borrow_mut().remove(&(deadline, id.0));
            }
        }
    }
    pub(super) fn NextDeadline(&self) -> Option<Instant> {
        self.ready_order
            .borrow()
            .first_key_value()
            .map(|((deadline, _), _)| *deadline)
    }
    fn TakeDue(&self, now: Instant) -> Option<ScheduledPopupTask> {
        let key = self
            .ready_order
            .borrow()
            .first_key_value()
            .map(|(key, _)| *key)?;
        if key.0 > now {
            return None;
        }
        self.ready_order.borrow_mut().remove(&key);
        self.tasks.borrow_mut().remove(&key.1)
    }
    fn RetainDocument(&self, document: dom::persistent_document::DOMOwnerHandle) {
        let expired: Vec<_> = self
            .tasks
            .borrow()
            .iter()
            .filter_map(|(id, task)| (task.document != document).then_some(PopupTaskId(*id)))
            .collect();
        for id in expired {
            self.Cancel(id);
        }
    }
}

/// One retained task-runner connection per LocalFrame Document. The source
/// EventHandlerState owns generations; Browser owns only cancellable tokens.
pub(super) struct NativeEventHandlerTimerServices {
    domain: Weak<super::local_frame::LocalFrameDomain>,
    frame: u64,
    document: dom::persistent_document::DOMOwnerHandle,
    active: [Cell<Option<PopupTaskId>>; 3],
}
impl NativeEventHandlerTimerServices {
    pub(super) fn new(domain: &Rc<super::local_frame::LocalFrameDomain>, frame: u64) -> Rc<Self> {
        Rc::new(Self {
            domain: Rc::downgrade(domain),
            frame,
            document: domain
                .Document(frame)
                .expect("timer Document")
                .borrow()
                .GetDocument()
                .RootHandle(),
            active: Default::default(),
        })
    }
    fn Index(kind: EventHandlerTimerKind) -> usize {
        match kind {
            EventHandlerTimerKind::Hover => 0,
            EventHandlerTimerKind::CursorUpdate => 1,
            EventHandlerTimerKind::ActiveInterval => 2,
        }
    }
}
impl PageEventHandlerTimerServices for NativeEventHandlerTimerServices {
    fn Stop(&self, kind: EventHandlerTimerKind) {
        if let Some(task) = self.active[Self::Index(kind)].take() {
            if let Some(page) = self
                .domain
                .upgrade()
                .and_then(|domain| domain.page.upgrade())
            {
                page.popup_tasks.Cancel(task);
            }
        }
    }
    fn IsActive(&self, kind: EventHandlerTimerKind) -> bool {
        self.active[Self::Index(kind)].get().is_some_and(|task| {
            self.domain
                .upgrade()
                .and_then(|domain| domain.page.upgrade())
                .is_some_and(|page| page.popup_tasks.IsScheduled(task))
        })
    }
    fn StartOneShot(&self, task: EventHandlerTimerTask, delay: Duration) {
        self.Stop(task.kind);
        let domain = self
            .domain
            .upgrade()
            .expect("live EventHandler task runner");
        let document = domain
            .Document(self.frame)
            .expect("attached timer Document");
        assert_eq!(
            document.borrow().GetDocument().RootHandle(),
            self.document,
            "timer owner belongs to current Document"
        );
        let page = domain
            .page
            .upgrade()
            .expect("live EventHandler Page task runner");
        let generation = domain
            .Frame(self.frame)
            .map_or(0, |frame| frame.generation.get());
        let index = Self::Index(task.kind);
        let root_document = page.document.borrow().GetDocument().RootHandle();
        let token = page.popup_tasks.Post(
            root_document,
            delay.as_secs_f64(),
            PageInteractionTask::EventHandler {
                frame: self.frame,
                document_generation: generation,
                task,
            },
        );
        self.active[index].set(Some(token));
    }
}
impl Drop for NativeEventHandlerTimerServices {
    fn drop(&mut self) {
        for kind in [
            EventHandlerTimerKind::Hover,
            EventHandlerTimerKind::CursorUpdate,
            EventHandlerTimerKind::ActiveInterval,
        ] {
            self.Stop(kind);
        }
    }
}

struct PagePopupLifecycleServices {
    page: Weak<PageState>,
    document: dom::persistent_document::DOMOwnerHandle,
}
impl PagePopupLifecycleServices {
    fn State(&self) -> Result<Rc<PageState>, PointerDomError> {
        self.page
            .upgrade()
            .filter(|state| {
                let owner = state.document.borrow();
                owner.GetDocument().RootHandle() == self.document && owner.GetDocument().IsActive()
            })
            .ok_or_else(|| PointerDomError::Lifecycle("popup Document is detached".into()))
    }
}
impl PopupLifecycleServices for PagePopupLifecycleServices {
    fn PostTask(&self, delay_seconds: f64, task: PopupTask) -> PopupTaskId {
        self.State()
            .expect("live popup task runner")
            .popup_tasks
            .Post(
                self.document,
                delay_seconds,
                PageInteractionTask::Popup(task),
            )
    }
    fn CancelTask(&self, task: PopupTaskId) {
        if let Some(state) = self.page.upgrade() {
            state.popup_tasks.Cancel(task);
        }
    }
    fn DialogRectContains(&self, node: u64, x: f64, y: f64) -> Result<bool, PointerDomError> {
        let state = self.State()?;
        state.EnsureLayoutOutputForNode(node);
        let layout = state.layout_engine.borrow();
        let output = layout
            .Output()
            .ok_or(PointerDomError::MissingDialogGeometry(node))?;
        let Some(node) = output.Node(node) else {
            // Element::GetBoundingClientRect returns an empty DOMRect when
            // the element has no associated layout object.
            return Ok(false);
        };
        let zoom = state.constraints.borrow().layout_zoom_factor as f32;
        let standardized_zoom = state
            .document
            .borrow()
            .GetDocument()
            .StandardizedBrowserZoomEnabled();
        let rect = node.HTMLBoundingClientRectForDocument(zoom, standardized_zoom);
        Ok(x >= rect.x() as f64
            && x < rect.right() as f64
            && y >= rect.y() as f64
            && y < rect.bottom() as f64)
    }
    fn AbsoluteQuads(
        &self,
        node: u64,
    ) -> Result<Option<Vec<foundation::gfx::QuadF>>, PointerDomError> {
        let state = self.State()?;
        state.EnsureLayoutOutputForNode(node);
        let layout = state.layout_engine.borrow();
        let output = layout
            .Output()
            .ok_or(PointerDomError::MissingDialogGeometry(node))?;
        Ok(output.Node(node).map(|node| node.AbsoluteQuads()))
    }
}

/// EventHandler's OneShotTimer uses its kUserInteraction task runner:
/// event_handler.cc:265-267,301-302,2051,2070-2071; event_handler.h:467.
/// Source generation and Document identity stay in the task.
pub(super) struct PageGestureTimerServices {
    page: Weak<PageState>,
    active: Cell<Option<PopupTaskId>>,
}
impl PageGestureTimerServices {
    pub(super) fn new(state: &Rc<PageState>) -> Self {
        Self {
            page: Rc::downgrade(state),
            active: Cell::new(None),
        }
    }
}
struct PageGestureScrollableArea {
    area: layoutng_assembly::native_mouse_coordinates::NativeRegisteredScrollableArea,
    runtime: Weak<PageInteractionScrollRuntime>,
}
impl interaction::core::input::gesture_manager::GestureScrollableArea
    for PageGestureScrollableArea
{
    fn ScrollsOverflow(&self) -> bool {
        self.area.ScrollsOverflow()
    }
    fn CancelScrollAnimation(&self) {
        // ScrollableArea::CancelScrollAnimation cancels the existing user
        // animator only; programmatic animation has its separate source API.
        if let Some(runtime) = self.runtime.upgrade() {
            runtime.animators.Cancel(self.area.Identity());
        }
    }
}
impl GestureTimerServices for PageGestureTimerServices {
    fn ScrollableAreas(
        &self,
    ) -> Vec<Rc<dyn interaction::core::input::gesture_manager::GestureScrollableArea>> {
        let Some(state) = self.page.upgrade() else {
            return Vec::new();
        };
        let runtime = state.scrollbar_scroll_runtime.borrow().clone();
        let areas = state.layout_engine.borrow().RegisteredScrollableAreas();
        areas
            .into_iter()
            .map(|area| {
                Rc::new(PageGestureScrollableArea {
                    area,
                    runtime: runtime.clone(),
                })
                    as Rc<dyn interaction::core::input::gesture_manager::GestureScrollableArea>
            })
            .collect()
    }
    fn StartActiveInterval(&self, delay: Duration, task: GestureTimerTask) -> Result<(), String> {
        self.StopActiveInterval();
        let state = self
            .page
            .upgrade()
            .ok_or_else(|| "gesture timer Page is detached".to_owned())?;
        let document = {
            let owner = state.document.borrow();
            let document = owner.GetDocument();
            if !document.IsActive() || document.RootHandle().Identity() != task.document_identity {
                return Err("gesture timer belongs to a detached Document".into());
            }
            document.RootHandle()
        };
        let token = state.popup_tasks.Post(
            document,
            delay.as_secs_f64(),
            PageInteractionTask::Gesture(task),
        );
        self.active.set(Some(token));
        Ok(())
    }
    fn StopActiveInterval(&self) {
        if let (Some(state), Some(task)) = (self.page.upgrade(), self.active.take()) {
            state.popup_tasks.Cancel(task);
        }
    }
}

impl PageState {
    pub(super) fn ApplyPopupMutationWithoutScript(
        &self,
        mutation: &dom::dom_mutation::DOMMutation,
    ) -> Result<bool, dom::dom_mutation::DOMMutationError> {
        if self.bindings.borrow().upgrade().is_some() {
            return Ok(false);
        }
        let engine = self.popup_source_interaction.borrow().clone();
        match engine {
            Some(engine) => engine
                .ApplyPopupMutation(&self.document.Handle(), mutation)
                .map_err(PopupMutationError),
            None => Ok(false),
        }
    }
    pub(super) fn DrainPopupLifecycleWithoutScript(&self) {
        if self.bindings.borrow().upgrade().is_some() {
            return;
        }
        let engine = self.popup_source_interaction.borrow().clone();
        if let Some(engine) = engine {
            engine
                .DrainPopupLifecycle(&self.document.Handle())
                .unwrap_or_else(|error| std::panic::panic_any(error));
        }
    }
    pub(super) fn DrainPopupLifecycleWithRuntime(&self, runtime: &mut dyn JavaScriptHostRuntime) {
        let interaction = self.popup_interaction.borrow().upgrade();
        if let Some(interaction) = interaction {
            interaction
                .DrainPopupLifecycle(runtime)
                .unwrap_or_else(|error| std::panic::panic_any(error));
        }
    }
    pub(super) fn InstallPopupTaskServices(self: &Rc<Self>) {
        let identity = self.document.borrow().GetDocument().RootHandle();
        self.popup_tasks.RetainDocument(identity);
        if self
            .document
            .borrow()
            .GetDocument()
            .PopupLifecycleState()
            .pointer_dom_services
            .is_some()
        {
            return;
        }
        InstallPopupLifecycleServices(
            &self.document.Handle(),
            Rc::new(PagePopupLifecycleServices {
                page: Rc::downgrade(self),
                document: identity,
            }),
        );
    }
}

impl Page {
    /// Native platform close signal, delivered through the registered
    /// LocalFrame host connection rather than synthesizing an Escape key.
    pub fn SignalCloseWatchers(&self) -> bool {
        let domain = self
            .state
            .local_frames
            .get()
            .expect("Page LocalFrame domain");
        domain.SignalCloseWatchers(domain.root)
    }
    pub(super) fn RunDuePopupTask(&mut self, now: Instant) -> io::Result<bool> {
        loop {
            let Some(task) = self.state.popup_tasks.TakeDue(now) else {
                return Ok(false);
            };
            let active = {
                let owner = self.state.document.borrow();
                owner.GetDocument().RootHandle() == task.document && owner.GetDocument().IsActive()
            };
            if !active {
                continue;
            }
            if matches!(&task.task, PageInteractionTask::LayoutShiftInputTimer) {
                return Ok(true);
            }
            if let PageInteractionTask::EventHandler {
                frame,
                document_generation,
                task: timer,
            } = &task.task
            {
                let domain = self.state.local_frames.get().unwrap();
                if *frame != domain.root {
                    if domain
                        .Frame(*frame)
                        .is_some_and(|owner| owner.generation.get() == *document_generation)
                    {
                        if let Some(result) = domain.WithInputListeners(*frame, |engine, _| {
                            engine.RunEventHandlerTimer(timer)
                        }) {
                            result.map_err(io::Error::other)?;
                        }
                    }
                    return Ok(true);
                }
            }
            if let PageInteractionTask::ChildScrollbar {
                frame,
                generation,
                task,
            } = &task.task
            {
                let domain = self.state.local_frames.get().unwrap();
                if let Some(frame_owner) = domain.Frame(*frame) {
                    if frame_owner.generation.get() == *generation {
                        if let Some(engine) = domain.FocusInteraction(*frame) {
                            engine.RunScrollbarTimerTask(*task)?;
                        }
                    }
                }
                return Ok(true);
            }
            if let Some(scripts) = &mut self.scripts {
                scripts.RunInteractionTask(task.task)?;
            } else {
                match task.task {
                    PageInteractionTask::LayoutShiftInputTimer => {
                        unreachable!("empty timer callback routed above")
                    }
                    PageInteractionTask::EventHandler { task, .. } => self
                        .interaction
                        .RunEventHandlerTimer(&task)
                        .map_err(io::Error::other)?,
                    PageInteractionTask::ChildScrollbar { .. } => {
                        unreachable!("child task routed above")
                    }
                    PageInteractionTask::FormRestore(task)
                    | PageInteractionTask::FormNavigation(task)
                    | PageInteractionTask::AnchorNavigation(task) => {
                        if let Some(task) = task.borrow_mut().take() {
                            task();
                        }
                    }
                    PageInteractionTask::SelectionAutoscroll(now) => self
                        .interaction
                        .AnimateSelectionAutoscroll(&self.state.document.Handle(), now)
                        .map_err(io::Error::other)?,
                    PageInteractionTask::DirectoryFilesChosen { target, files } => {
                        self.interaction.DirectoryFilesChosen(
                            &self.state.document.Handle(),
                            target,
                            files,
                        );
                    }
                    PageInteractionTask::SystemDragTarget {
                        event,
                        action,
                        result,
                    } => {
                        let operation = self
                            .interaction
                            .HandleTargetDrag(&self.state.document.Handle(), &event, &action)
                            .map_err(io::Error::other)?;
                        result.set(Some(operation));
                    }
                    PageInteractionTask::SystemDragEnd { event, operation } => {
                        self.interaction.DragSourceEndedAt(
                            &self.state.document.Handle(),
                            &event,
                            operation,
                        );
                    }
                    PageInteractionTask::Popup(task) => self
                        .interaction
                        .RunPopupTask(&self.state.document.Handle(), task)
                        .map_err(|error| io::Error::other(error.to_string()))?,
                    PageInteractionTask::Gesture(task) => self
                        .interaction
                        .RunGestureTimerTask(&self.state.document.Handle(), task)
                        .map_err(io::Error::other)?,
                    PageInteractionTask::Scrollbar(task) => {
                        self.interaction.RunScrollbarTimerTask(task)?
                    }
                }
            }
            return Ok(true);
        }
    }
}
