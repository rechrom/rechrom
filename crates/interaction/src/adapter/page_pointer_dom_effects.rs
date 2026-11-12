//! Source DOM pointer effects over the canonical document popup/interest state.
//! The native dispatcher is borrowed only for the synchronous invocation.
use super::*;
use dom::dom_mutation::{DOMMutation, DOMMutationType};
use foundation::popup_types::*;
use interaction_default::core::html::closewatcher::close_watcher::{
    AllowCancel, CloseWatcher, CloseWatcherEnvironment, HistoryUserActivation, NativeCloseWatcher,
    WatcherStack, WatcherState,
};
use interaction_default::core::{
    dom::element_interest::{self, InterestEnvironment},
    dom::focus_delegate::{self, FocusDelegateEnvironment},
    html::dialog_lifecycle::{self, DialogLifecycleEnvironment},
    html::menu_safe_triangle::{self, MenuSafeTriangleEnvironment, TriangleFacts},
    html::pointer_light_dismiss::{self, LightDismissEnvironment},
    html::popover_lifecycle::{self, PopoverLifecycleEnvironment},
    loader::anchor_element_interaction_tracker::AnchorElementInteractionTracker,
};
use interaction_dispatch::core::dom::events::event_init::EventInit;
use interaction_dispatch::core::events::interest_event::InterestEvent;
use interaction_dispatch::core::events::toggle_event::ToggleEvent;

#[derive(Clone, Debug)]
pub enum PointerDomError {
    MissingLifecycleService,
    MissingDialogGeometry(u64),
    Lifecycle(String),
}
impl std::fmt::Display for PointerDomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for PointerDomError {}

/// The source posts weak invoker/target tasks; the document task owner supplies
/// cancellation and retains the document identity rather than a scoped borrow.
#[derive(Clone, Copy, Debug)]
pub enum PopupTask {
    InterestGained {
        invoker: u64,
        target: u64,
    },
    InterestLost {
        invoker: u64,
        target: u64,
    },
    Toggle {
        node: u64,
        old_open: bool,
        new_open: bool,
        source: Option<u64>,
        dialog: bool,
    },
    DialogClose {
        node: u64,
    },
    MenuTriangleExpire {
        triangle: MenuTriangleId,
    },
}
pub trait PopupLifecycleServices {
    fn PostTask(&self, delay_seconds: f64, task: PopupTask) -> PopupTaskId;
    fn CancelTask(&self, task: PopupTaskId);
    fn DialogRectContains(&self, node: u64, x: f64, y: f64) -> Result<bool, PointerDomError>;
    fn AbsoluteQuads(
        &self,
        node: u64,
    ) -> Result<Option<Vec<foundation::gfx::QuadF>>, PointerDomError>;
}
pub struct PopupLifecycleServicesHandle(pub Rc<dyn PopupLifecycleServices>);
pub fn InstallPopupLifecycleServices(
    document: &InteractionDocument,
    services: Rc<dyn PopupLifecycleServices>,
) {
    let mut owner = document.borrow_mut();
    let state = owner.GetDocumentMut().PopupLifecycleStateMut();
    assert!(
        state.pointer_dom_services.is_none(),
        "one popup lifecycle service per Document"
    );
    state.pointer_dom_services = Some(Rc::new(PopupLifecycleServicesHandle(services)));
}

struct Effects<'a, 'listener, 'frame> {
    dispatcher: &'a EventDispatcher<'listener, 'frame>,
}
impl HistoryUserActivation for Effects<'_, '_, '_> {
    fn IsHistoryUserActivationActive(&self) -> bool {
        self.dispatcher
            .input_frame
            .as_ref()
            .expect("CloseWatcher requires its LocalFrame owner")
            .history_activation
            .borrow()
            .IsActive()
    }
    fn ConsumeHistoryUserActivation(&self) {
        self.dispatcher
            .interaction_output
            .as_ref()
            .expect("CloseWatcher requires its Page effect route")(
            crate::InteractionOutput::Effect(
                crate::InteractionEffect::ConsumeHistoryUserActivation,
            ),
        );
    }
}
impl CloseWatcherEnvironment for Effects<'_, '_, '_> {
    type Error = PointerDomError;
    fn ReadGroups<T>(&self, read: impl FnOnce(&[Vec<CloseWatcherId>]) -> T) -> T {
        self.Read(|d| read(&d.PopupLifecycleState().close_watcher_stack.watcher_groups))
    }
    fn WriteGroups<T>(&self, write: impl FnOnce(&mut Vec<Vec<CloseWatcherId>>) -> T) -> T {
        self.Write(|d| {
            write(
                &mut d
                    .PopupLifecycleStateMut()
                    .close_watcher_stack
                    .watcher_groups,
            )
        })
    }
    fn AllowedGroups(&self) -> usize {
        self.Read(|d| d.PopupLifecycleState().close_watcher_stack.allowed_groups)
    }
    fn SetAllowedGroups(&self, count: usize) {
        self.Write(|d| {
            d.PopupLifecycleStateMut()
                .close_watcher_stack
                .allowed_groups = count
        });
    }
    fn NextUserInteractionCreatesANewAllowedGroup(&self) -> bool {
        self.Read(|d| {
            d.PopupLifecycleState()
                .close_watcher_stack
                .next_user_interaction_creates_a_new_allowed_group
        })
    }
    fn SetNextUserInteractionCreatesANewAllowedGroup(&self, value: bool) {
        self.Write(|d| {
            d.PopupLifecycleStateMut()
                .close_watcher_stack
                .next_user_interaction_creates_a_new_allowed_group = value
        });
    }
    fn State(&self, watcher: CloseWatcherId) -> WatcherState {
        self.Read(|d| {
            let w = d
                .PopupLifecycleState()
                .close_watchers
                .get(&watcher)
                .expect("canonical CloseWatcher identity");
            WatcherState {
                closed: w.closed,
                enabled: w.enabled,
                dispatching_cancel: w.dispatching_cancel,
            }
        })
    }
    fn SetDispatchingCancel(&self, watcher: CloseWatcherId, value: bool) {
        self.Write(|d| {
            d.PopupLifecycleStateMut()
                .close_watchers
                .get_mut(&watcher)
                .expect("canonical CloseWatcher identity")
                .dispatching_cancel = value
        });
    }
    fn SetEnabledState(&self, watcher: CloseWatcherId, enabled: bool) {
        self.Write(|d| {
            d.PopupLifecycleStateMut()
                .close_watchers
                .get_mut(&watcher)
                .expect("canonical watcher")
                .enabled = enabled
        });
    }
    fn BindCloseListener(&self) {
        self.Write(|d| {
            let stack = &mut d.PopupLifecycleStateMut().close_watcher_stack;
            stack.close_listener = Some(
                stack
                    .close_signal_host
                    .clone()
                    .expect("LocalFrame close-signal host is bound"),
            );
        });
    }
    fn ResetCloseListener(&self) {
        self.Write(|d| {
            d.PopupLifecycleStateMut()
                .close_watcher_stack
                .close_listener = None
        });
    }
    fn SetClosedAndClearAbortAlgorithm(&self, watcher: CloseWatcherId) {
        let owner = self.Write(|d| {
            let w = d
                .PopupLifecycleStateMut()
                .close_watchers
                .get_mut(&watcher)
                .expect("canonical CloseWatcher identity");
            w.closed = true;
            w.abort_algorithm = None;
            w.script_owner.clone()
        });
        if let Some(owner) = owner {
            let owner = owner
                .downcast::<NativeCloseWatcher>()
                .expect("CloseWatcher native owner");
            owner.abort_handle.borrow_mut().take();
            let notify = owner.did_destroy.borrow_mut().take();
            if let Some(notify) = notify {
                notify();
            }
        }
    }
    fn HasWindow(&self) -> bool {
        self.dispatcher.targets.WindowDocument().IsAttached()
    }
    fn IsDocumentActive(&self) -> bool {
        self.Read(|d| d.IsActive())
    }
    // HTMLDialogElement::CloseWatcherFiredCancel and the native popover
    // listener. Only the dialog listener forwards cancel to its DOM element.
    fn DispatchCancel(
        &self,
        watcher: CloseWatcherId,
        cancelable: bool,
    ) -> Result<bool, PointerDomError> {
        let owner = self.Read(|d| d.PopupLifecycleState().close_watchers[&watcher].owner);
        match owner {
            CloseWatcherOwner::Dialog(node) => {
                let mut event =
                    interaction_dispatch::core::dom::events::event::Event::CreateWithInitializer(
                        "cancel",
                        &EventInit {
                            cancelable,
                            ..Default::default()
                        },
                        self.dispatcher.clock.as_ref(),
                    );
                event.SetTrusted(true);
                let event = EventRef::new(event);
                super::ports::DispatchCanonical(self.dispatcher, event.clone(), node);
                let prevented = event.borrow().BaseEvent().defaultPrevented();
                event.borrow_mut().BaseEventMut().SetDefaultHandled();
                Ok(prevented)
            }
            CloseWatcherOwner::Popover(_) => Ok(false),
            CloseWatcherOwner::Script => {
                let target = self.ScriptWatcher(watcher)?;
                let mut event =
                    interaction_dispatch::core::dom::events::event::Event::CreateWithInitializer(
                        "cancel",
                        &EventInit {
                            cancelable,
                            ..Default::default()
                        },
                        self.dispatcher.clock.as_ref(),
                    );
                event.SetTrusted(true);
                let event = EventRef::new(event);
                super::ports::DispatchCloseWatcher(self.dispatcher, event.clone(), &target);
                let prevented = event.borrow().BaseEvent().defaultPrevented();
                Ok(prevented)
            }
        }
    }
    // Native close listeners reenter the existing lifecycle, sharing the same
    // watcher, top-layer, focus restoration, task and mutation owners.
    fn DispatchClose(&self, watcher: CloseWatcherId) -> Result<(), PointerDomError> {
        let owner = self.Read(|d| d.PopupLifecycleState().close_watchers[&watcher].owner);
        match owner {
            CloseWatcherOwner::Dialog(node) => {
                let (value, source) = self.Read(|d| {
                    let data = d
                        .PopupLifecycleState()
                        .dialogs
                        .get(&node)
                        .expect("CloseWatcher dialog owner");
                    (
                        data.request_close_return_value.clone(),
                        data.request_close_source_element,
                    )
                });
                dialog_lifecycle::Close(self, node, value.as_deref(), source, false)?;
                self.Write(|d| {
                    if let Some(dialog) = d.PopupLifecycleStateMut().dialogs.get_mut(&node) {
                        dialog.request_close_source_element = None;
                    }
                });
                Ok(())
            }
            CloseWatcherOwner::Popover(node) => {
                popover_lifecycle::HidePopoverInternal(
                    self,
                    node,
                    None,
                    HidePopoverFocusBehavior::FocusPreviousElement,
                    HidePopoverTransitionBehavior::FireEventsAndWaitForTransitions,
                )?;
                Ok(())
            }
            CloseWatcherOwner::Script => {
                let target = self.ScriptWatcher(watcher)?;
                let mut event =
                    interaction_dispatch::core::dom::events::event::Event::CreateWithInitializer(
                        "close",
                        &EventInit::default(),
                        self.dispatcher.clock.as_ref(),
                    );
                event.SetTrusted(true);
                super::ports::DispatchCloseWatcher(self.dispatcher, EventRef::new(event), &target);
                Ok(())
            }
        }
    }
}

// LocalFrame::SetHadUserInteraction: history activation and stack allowance
// advance together, independently from transient activation consumption.
pub(crate) fn SetHadUserInteraction(dispatcher: &EventDispatcher<'_, '_>, value: bool) {
    let now = dispatcher.clock.Now();
    RecordHadUserInteraction(dispatcher, value, now);
    if value {
        dispatcher
            .interaction_output
            .as_ref()
            .expect("LocalFrame activation effect route")(crate::InteractionOutput::Effect(
            crate::InteractionEffect::NotifyHistoryUserActivation(now),
        ));
    }
}
pub(crate) fn RecordHadUserInteraction(
    dispatcher: &EventDispatcher<'_, '_>,
    value: bool,
    now: interaction_input::base::time::time::TimeTicks,
) {
    let frame = dispatcher
        .input_frame
        .as_ref()
        .expect("LocalFrame activation owner");
    if value {
        frame.history_activation.borrow_mut().Activate(now);
    } else {
        frame.history_activation.borrow_mut().Clear();
    }
    WatcherStack::SetHadUserInteraction(&Effects { dispatcher }, value);
}
pub(crate) fn CloseWatcherEscapeKeyHandler(
    dispatcher: &EventDispatcher<'_, '_>,
    event: &interaction_dispatch::core::events::keyboard_event::KeyboardEventRef,
) -> Result<(), PointerDomError> {
    WatcherStack::EscapeKeyHandler(&Effects { dispatcher }, event)
}
pub(crate) fn CreateScriptCloseWatcher(
    dispatcher: &EventDispatcher<'_, '_>,
    host_id: u64,
    target: Rc<interaction_dispatch::core::dom::events::event_target::EventTarget>,
    aborted: bool,
) -> Result<Rc<NativeCloseWatcher>, PointerDomError> {
    let e = Effects { dispatcher };
    if !e.HasWindow() || !e.IsDocumentActive() {
        return Err(PointerDomError::Lifecycle(
            "CloseWatchers cannot be created in detached Windows".into(),
        ));
    }
    let watcher = e.Write(|d| {
        d.PopupLifecycleStateMut()
            .CreateCloseWatcher(CloseWatcherOwner::Script)
    });
    let native = Rc::new(NativeCloseWatcher {
        watcher,
        host_id,
        target,
        abort_handle: RefCell::new(None),
        did_destroy: RefCell::new(None),
    });
    e.Write(|d| {
        let watcher = d
            .PopupLifecycleStateMut()
            .close_watchers
            .get_mut(&watcher)
            .unwrap();
        watcher.closed = aborted;
        watcher.script_owner = Some(native.clone());
    });
    if !aborted {
        WatcherStack::Add(&e, watcher);
    }
    Ok(native)
}
pub(crate) fn InvokeScriptCloseWatcher(
    dispatcher: &EventDispatcher<'_, '_>,
    watcher: CloseWatcherId,
    request: bool,
) -> Result<(), PointerDomError> {
    let e = Effects { dispatcher };
    if request {
        CloseWatcher::RequestClose(&e, watcher, AllowCancel::kAlways)?;
    } else {
        CloseWatcher::close(&e, watcher)?;
    }
    Ok(())
}
pub(crate) fn DestroyScriptCloseWatcher(
    dispatcher: &EventDispatcher<'_, '_>,
    watcher: CloseWatcherId,
) {
    CloseWatcher::destroy(&Effects { dispatcher }, watcher);
}
pub(crate) fn SignalCloseWatchers(
    dispatcher: &EventDispatcher<'_, '_>,
) -> Result<(), PointerDomError> {
    let e = Effects { dispatcher };
    if WatcherStack::AnyEnabledWatchers(&e) {
        WatcherStack::Signal(&e)?;
    }
    Ok(())
}
impl Effects<'_, '_, '_> {
    fn ScriptWatcher(
        &self,
        watcher: CloseWatcherId,
    ) -> Result<Rc<NativeCloseWatcher>, PointerDomError> {
        self.Read(|d| {
            d.PopupLifecycleState().close_watchers[&watcher]
                .script_owner
                .clone()
        })
        .and_then(|owner| owner.downcast().ok())
        .ok_or_else(|| {
            PointerDomError::Lifecycle("CloseWatcher native EventTarget owner is absent".into())
        })
    }
    fn EnsureOpenDialog(&self, node: u64) {
        if !LightDismissEnvironment::IsOpenAndActiveDialog(self, node) {
            return;
        }
        self.Write(|d| {
            let s = d.PopupLifecycleStateMut();
            if !s.all_open_dialogs.contains(&node) {
                s.all_open_dialogs.push(node);
            }
            let existing = s.dialogs.entry(node).or_default().close_watcher;
            if existing.is_none() {
                let watcher = s.CreateCloseWatcher(CloseWatcherOwner::Dialog(node));
                s.dialogs.get_mut(&node).unwrap().close_watcher = Some(watcher);
                return Some(watcher);
            }
            None
        })
        .inspect(|&watcher| {
            WatcherStack::Add(self, watcher);
        });
        DialogLifecycleEnvironment::SetCloseWatcherEnabledState(self, node);
    }
    fn FocusDialogControl(&self, node: u64) {
        self.dispatcher
            .focus_services
            .lifecycle
            .UpdateStyleAndLayoutTreeForElement(node);
        if foundation::RuntimeEnabledFeatures::DialogNewFocusBehaviorEnabled() {
            focus_delegate::SetFocusForDialog(self, node);
        } else {
            focus_delegate::SetFocusForDialogLegacy(self, node);
        }
    }
    fn Toggle(
        &self,
        node: u64,
        kind: &str,
        old_open: bool,
        new_open: bool,
        source: Option<u64>,
        cancelable: bool,
    ) -> bool {
        let source = source.and_then(|source| self.dispatcher.targets.ResolveTarget(source));
        let references = foundation::RuntimeEnabledFeatures::ShadowRootReferenceTargetEnabled();
        let mut event = ToggleEvent::WithInitializer(
            kind,
            &EventInit {
                cancelable,
                composed: source.is_some() && references,
                ..Default::default()
            },
            if old_open { "open" } else { "closed" },
            if new_open { "open" } else { "closed" },
            source,
            references,
            self.dispatcher.clock.as_ref(),
        );
        event.BaseEventMut().SetTrusted(true);
        let event = EventRef::new(event);
        super::ports::DispatchCanonical(self.dispatcher, event.clone(), node);
        let allowed = !event.borrow().BaseEvent().defaultPrevented();
        allowed
    }
    fn Focus(&self, node: u64, prevent_scroll: bool) {
        self.FocusWithSelection(
            node,
            interaction_default::core::dom::element::SelectionBehaviorOnFocus::kRestore,
            prevent_scroll,
        );
    }
    fn FocusWithSelection(
        &self,
        node: u64,
        selection_behavior: interaction_default::core::dom::element::SelectionBehaviorOnFocus,
        prevent_scroll: bool,
    ) {
        use interaction_default::{
            core::{dom::focus_params::FocusParams, page::focus_controller::FocusDispatchScope},
            public::mojom::input::focus_type::FocusType,
        };
        let mut params = FocusParams::default();
        params.r#type = FocusType::kScript;
        params.selection_behavior = selection_behavior;
        if prevent_scroll {
            let options = foundation::FocusOptions {
                prevent_scroll: true,
                ..Default::default()
            };
            params.options = Rc::new(options);
        }
        self.dispatcher
            .Context()
            .focus_controller
            .FocusWithParamsAndScope(
                node,
                &params,
                &FocusDispatchScope {
                    focus_events: Some(self.dispatcher),
                    ..Default::default()
                },
            );
    }
    fn DestroyWatcher(&self, id: CloseWatcherId) {
        CloseWatcher::destroy(self, id);
    }
    // Source html_dialog_element.cc:225-238 and close_watcher.cc:242-281.
    fn RequestDialogClose(&self, node: u64) -> Result<(), PointerDomError> {
        dialog_lifecycle::RequestCloseInternal(self, node, Some(&[]), None)
    }
    fn CloseWatcherRequestCloseAlwaysImpl(&self, node: u64) -> Result<(), PointerDomError> {
        if !LightDismissEnvironment::IsOpenAndActiveDialog(self, node) {
            return Ok(());
        }
        let watcher = self
            .Read(|d| {
                d.PopupLifecycleState()
                    .dialogs
                    .get(&node)
                    .and_then(|data| data.close_watcher)
            })
            .ok_or_else(|| {
                PointerDomError::Lifecycle("open dialog requires its CloseWatcher owner".into())
            })?;
        CloseWatcher::RequestClose(self, watcher, AllowCancel::kAlways)?;
        Ok(())
    }
    fn Read<T>(&self, read: impl FnOnce(&dom::Document) -> T) -> T {
        let owner = self.dispatcher.document.borrow();
        read(owner.GetDocument())
    }
    fn Write<T>(&self, write: impl FnOnce(&mut dom::Document) -> T) -> T {
        let mut owner = self.dispatcher.document.borrow_mut();
        write(owner.GetDocumentMut())
    }
    fn Services(&self) -> Result<Rc<dyn PopupLifecycleServices>, PointerDomError> {
        self.Read(|d| d.PopupLifecycleState().pointer_dom_services.clone())
            .and_then(|owner| owner.downcast::<PopupLifecycleServicesHandle>().ok())
            .map(|owner| owner.0.clone())
            .ok_or(PointerDomError::MissingLifecycleService)
    }
    fn Id(&self, node: &EventTargetRef) -> Option<u64> {
        self.dispatcher.targets.NodeId(node)
    }
    fn Open(&self, node: u64) -> bool {
        self.Read(|d| d.FindNodeById(node).is_some_and(|node| d.PopoverOpen(node)))
    }
    fn Stack(&self, hint: bool) -> Vec<u64> {
        self.Read(|d| {
            if hint {
                d.PopupLifecycleState().hint_stack.clone()
            } else {
                d.PopupLifecycleState().auto_stack.clone()
            }
        })
    }
    fn InvalidateInterest(&self, node: u64) {
        self.Write(|d| {
            if let Some(node) = d.FindNodeById(node) {
                d.InvalidateNodeStyle(node);
            }
        })
    }
    fn InterestTarget(&self, node: u64) -> Option<u64> {
        self.Read(|d| {
            let index = d.FindNodeById(node)?;
            let element = d.Node(index);
            let target = d
                .GetElementAttributeResolvingReferenceTarget(index, "interestfor")
                .or_else(|| d.MenuItemInvokedSubmenu(index))?;
            if !d.IsInTreeScope(index) || !d.IsInTreeScope(target) {
                return None;
            }
            let valid = if element.IsHTMLElement("button") {
                !d.IsDisabledFormControl(index)
            } else if element.IsHTMLElement("a") || element.IsHTMLElement("area") {
                element.FindAttribute("href").is_some()
            } else if element.IsHTMLElement("menuitem") {
                element.FindAttribute("disabled").is_none()
            } else {
                false
            };
            valid.then(|| d.Node(target).Id())
        })
    }
    fn Cancel(&self, node: u64, gained: bool) {
        let task = self.Write(|d| {
            d.PopupLifecycleStateMut()
                .interest_invokers
                .get_mut(&node)
                .and_then(|data| {
                    if gained {
                        data.gained_task.take()
                    } else {
                        data.lost_task.take()
                    }
                })
        });
        if let Some(task) = task {
            if let Ok(services) = self.Services() {
                services.CancelTask(task);
            }
        }
    }
    // Source element.cc:12910-12977; element.h:300-301.
    fn Schedule(&self, node: u64, gained: bool) -> Result<(), PointerDomError> {
        let delay = self.Read(|d| {
            let node = d.FindNodeById(node)?;
            let style = d.ResolvedStyleFor(node)?.ComputedStyle()?;
            if style.IsEnsuredInDisplayNone() {
                return None;
            }
            let delay = if gained {
                style.InterestDelayStart()
            } else {
                style.InterestDelayEnd()
            };
            Some(if delay.IsNormal() {
                if gained {
                    0.5
                } else {
                    0.25
                }
            } else {
                delay.DelaySeconds()
            })
        });
        let Some(delay) = delay.filter(|delay| delay.is_finite()) else {
            return Ok(());
        };
        assert!(delay >= 0.0);
        let target = if gained {
            self.InterestTarget(node)
        } else {
            self.Read(|d| {
                d.PopupLifecycleState()
                    .interest_invokers
                    .get(&node)
                    .and_then(|data| data.active_target)
            })
        };
        let Some(target) = target else {
            return Ok(());
        };
        let services = self.Services()?;
        self.Cancel(node, gained);
        let task = services.PostTask(
            delay,
            if gained {
                PopupTask::InterestGained {
                    invoker: node,
                    target,
                }
            } else {
                PopupTask::InterestLost {
                    invoker: node,
                    target,
                }
            },
        );
        self.Write(|d| {
            let data = d
                .PopupLifecycleStateMut()
                .interest_invokers
                .entry(node)
                .or_default();
            if gained {
                data.gained_task = Some(task);
            } else {
                data.lost_task = Some(task);
            }
        });
        Ok(())
    }
}

impl MenuSafeTriangleEnvironment for Effects<'_, '_, '_> {
    type Error = PointerDomError;
    fn OwningMenuIsList(&self, invoker: u64) -> bool {
        self.Write(|d| {
            if let Some(node) = d.FindNodeById(invoker) {
                d.EnsureMenuItemAncestorCache(node);
            }
        });
        self.Read(|d| {
            d.PopupLifecycleState()
                .menu_item_owners
                .get(&invoker)
                .copied()
                .flatten()
                .and_then(|owner| d.FindNodeById(owner))
                .is_some_and(|node| d.Node(node).IsHTMLElement("menulist"))
        })
    }
    fn LastMousePosition(&self) -> Option<foundation::gfx::PointF> {
        if self.dispatcher.mouse_state.IsMousePositionUnknown() {
            return None;
        }
        // This Page is the local frame root and has an inactive VisualViewport,
        // so EventHandler::ViewportToFrame retains the same widget coordinates.
        let point = self
            .dispatcher
            .mouse_state
            .LastKnownMousePositionInViewport();
        Some(foundation::gfx::PointF::new(point.x(), point.y()))
    }
    fn UpdateStyleAndLayoutTreeForElement(&self, node: u64) {
        self.dispatcher
            .focus_services
            .lifecycle
            .UpdateStyleAndLayoutTreeForElement(node);
    }
    fn AbsoluteQuads(
        &self,
        node: u64,
    ) -> Result<Option<Vec<foundation::gfx::QuadF>>, PointerDomError> {
        self.Services()?.AbsoluteQuads(node)
    }
    fn Create(
        &self,
        invoker: u64,
        submenu: u64,
        triangle: foundation::gfx::QuadF,
        submenu_quad: foundation::gfx::QuadF,
    ) -> Result<MenuTriangleId, PointerDomError> {
        let services = self.Services()?;
        let id = self.Write(|d| {
            let s = d.PopupLifecycleStateMut();
            s.next_menu_triangle_id = s
                .next_menu_triangle_id
                .checked_add(1)
                .expect("MenuSafeTriangle identity exhausted");
            let id = MenuTriangleId(s.next_menu_triangle_id);
            s.menu_triangles.insert(
                id,
                dom::popup_state::MenuSafeTriangleData {
                    invoker: Some(invoker),
                    submenu: Some(submenu),
                    triangle,
                    submenu_quad,
                    deferred_gains: Vec::new(),
                    deferred_losses: Vec::new(),
                    expire_task: None,
                },
            );
            id
        });
        let task = services.PostTask(1.0, PopupTask::MenuTriangleExpire { triangle: id });
        self.Write(|d| {
            d.PopupLifecycleStateMut()
                .menu_triangles
                .get_mut(&id)
                .expect("new source triangle")
                .expire_task = Some(task)
        });
        Ok(id)
    }
    fn SetActive(&self, id: Option<MenuTriangleId>) {
        self.Write(|d| {
            let active = &mut d.PopupLifecycleStateMut().active_menu_triangle;
            assert_ne!(
                active.is_none(),
                id.is_none(),
                "Document::SetMenuSafeTriangle changes one live owner"
            );
            *active = id;
        });
    }
    fn Active(&self) -> Option<MenuTriangleId> {
        self.Read(|d| d.PopupLifecycleState().active_menu_triangle)
    }
    fn Facts(&self, id: MenuTriangleId) -> Option<TriangleFacts> {
        self.Read(|d| {
            let data = d.PopupLifecycleState().menu_triangles.get(&id)?;
            Some(TriangleFacts {
                invoker: data.invoker?,
                submenu: data.submenu?,
                triangle: data.triangle,
                submenu_quad: data.submenu_quad,
            })
        })
    }
    fn MenuStackLen(&self) -> usize {
        self.Read(|d| d.PopupLifecycleState().auto_stack.len())
    }
    fn MenuStackAt(&self, index: usize) -> u64 {
        self.Read(|d| d.PopupLifecycleState().auto_stack[index])
    }
    fn IsMenuList(&self, node: u64) -> bool {
        PopoverLifecycleEnvironment::IsMenuList(self, node)
    }
    fn InclusiveFlatAncestor(&self, ancestor: u64, node: u64) -> bool {
        self.Read(|d| {
            d.FindNodeById(ancestor)
                .zip(d.FindNodeById(node))
                .is_some_and(|(ancestor, node)| d.ContainsNodeInTree(ancestor, node, true))
        })
    }
    fn RemoveLosses(&self, id: MenuTriangleId, invoker: u64) -> bool {
        self.Write(|d| {
            let losses = &mut d
                .PopupLifecycleStateMut()
                .menu_triangles
                .get_mut(&id)
                .expect("active triangle")
                .deferred_losses;
            let before = losses.len();
            losses.retain(|data| data.invoker != invoker);
            before != losses.len()
        })
    }
    fn RemoveGains(&self, id: MenuTriangleId, invoker: u64) -> bool {
        self.Write(|d| {
            let gains = &mut d
                .PopupLifecycleStateMut()
                .menu_triangles
                .get_mut(&id)
                .expect("active triangle")
                .deferred_gains;
            let before = gains.len();
            gains.retain(|data| data.invoker != invoker);
            before != gains.len()
        })
    }
    fn UpdateGain(&self, id: MenuTriangleId, invoker: u64, state: InterestState) -> bool {
        self.Write(|d| {
            let gains = &mut d
                .PopupLifecycleStateMut()
                .menu_triangles
                .get_mut(&id)
                .expect("active triangle")
                .deferred_gains;
            if let Some(data) = gains.iter_mut().find(|data| data.invoker == invoker) {
                data.state = state;
                true
            } else {
                false
            }
        })
    }
    fn UpdateLoss(
        &self,
        id: MenuTriangleId,
        invoker: u64,
        cancelable: InterestLostCancelable,
        behavior: InterestLostPopoverBehavior,
    ) -> bool {
        self.Write(|d| {
            let losses = &mut d
                .PopupLifecycleStateMut()
                .menu_triangles
                .get_mut(&id)
                .expect("active triangle")
                .deferred_losses;
            if let Some(data) = losses.iter_mut().find(|data| data.invoker == invoker) {
                data.cancelable = cancelable;
                data.behavior = behavior;
                true
            } else {
                false
            }
        })
    }
    fn PushGain(&self, id: MenuTriangleId, gain: DeferredInterestGain) {
        self.Write(|d| {
            d.PopupLifecycleStateMut()
                .menu_triangles
                .get_mut(&id)
                .expect("active triangle")
                .deferred_gains
                .push(gain)
        });
    }
    fn PushLoss(&self, id: MenuTriangleId, loss: DeferredInterestLoss) {
        self.Write(|d| {
            d.PopupLifecycleStateMut()
                .menu_triangles
                .get_mut(&id)
                .expect("active triangle")
                .deferred_losses
                .push(loss)
        });
    }
    fn LossAt(&self, id: MenuTriangleId, index: usize) -> Option<DeferredInterestLoss> {
        self.Read(|d| {
            d.PopupLifecycleState()
                .menu_triangles
                .get(&id)?
                .deferred_losses
                .get(index)
                .copied()
        })
    }
    fn GainAt(&self, id: MenuTriangleId, index: usize) -> Option<DeferredInterestGain> {
        self.Read(|d| {
            d.PopupLifecycleState()
                .menu_triangles
                .get(&id)?
                .deferred_gains
                .get(index)
                .copied()
        })
    }
    fn InterestLost(&self, loss: DeferredInterestLoss) -> Result<(), PointerDomError> {
        element_interest::InterestLost(
            self,
            loss.invoker,
            Some(loss.target),
            loss.cancelable,
            loss.behavior,
        )?;
        Ok(())
    }
    fn InterestGained(&self, gain: DeferredInterestGain) -> Result<(), PointerDomError> {
        element_interest::InterestGained(self, gain.invoker, Some(gain.target), gain.state)?;
        Ok(())
    }
    fn Clear(&self, id: MenuTriangleId, from_timer: bool) {
        let data = self.Write(|d| d.PopupLifecycleStateMut().menu_triangles.remove(&id));
        if !from_timer {
            if let Some(task) = data.and_then(|data| data.expire_task) {
                self.Services()
                    .expect("existing triangle retains task service")
                    .CancelTask(task);
            }
        }
    }
}

impl FocusDelegateEnvironment for Effects<'_, '_, '_> {
    fn ShadowRoot(&self, node: u64) -> Option<u64> {
        self.Read(|d| {
            d.FindNodeById(node)
                .and_then(|n| d.ShadowRoot(n))
                .map(|n| d.Node(n).Id())
        })
    }
    fn ShadowRootIsUserAgent(&self, root: u64) -> bool {
        self.Read(|d| {
            d.FindNodeById(root).is_some_and(|n| {
                d.ShadowRootMode(n) == Some(dom::persistent_document::DOMShadowRootMode::kUserAgent)
            })
        })
    }
    fn DelegatesFocus(&self, node: u64) -> bool {
        self.Read(|d| {
            d.FindNodeById(node)
                .is_some_and(|n| d.ShadowRootDelegatesFocus(n))
        })
    }
    fn FocusedElement(&self) -> Option<u64> {
        self.dispatcher.state.borrow().focused_node_id
    }
    fn ShadowIncludingInclusiveAncestor(&self, ancestor: u64, node: u64) -> bool {
        self.Read(|d| {
            d.FindNodeById(ancestor)
                .zip(d.FindNodeById(node))
                .is_some_and(|(ancestor, node)| {
                    d.IsShadowIncludingInclusiveAncestor(ancestor, node)
                })
        })
    }
    fn NextElement(&self, root: u64, after: Option<u64>) -> Option<u64> {
        self.Read(|d| {
            d.NextPopupCollectionElement(
                d.FindNodeById(root)?,
                after.and_then(|id| d.FindNodeById(id)),
            )
            .map(|n| d.Node(n).Id())
        })
    }
    fn FirstFlatChild(&self, node: u64) -> Option<u64> {
        self.Read(|d| {
            d.FlatTreeChildren(d.FindNodeById(node)?)
                .first()
                .map(|&n| d.Node(n).Id())
        })
    }
    fn NextFlatNode(&self, node: u64, root: u64, skip_children: bool) -> Option<u64> {
        self.Read(|d| {
            let mut current = d.FindNodeById(node)?;
            let root = d.FindNodeById(root)?;
            if !skip_children {
                if let Some(&child) = d.FlatTreeChildren(current).first() {
                    return Some(d.Node(child).Id());
                }
            }
            while current != root {
                let parent = d.FlatTreeParent(current)?;
                let children = d.FlatTreeChildren(parent);
                let index = children.iter().position(|&n| n == current)?;
                if let Some(&next) = children.get(index + 1) {
                    return Some(d.Node(next).Id());
                }
                current = parent;
            }
            None
        })
    }
    fn IsElement(&self, node: u64) -> bool {
        self.Read(|d| {
            d.FindNodeById(node)
                .is_some_and(|n| d.Node(n).Type() == dom::DOMNodeType::kElement)
        })
    }
    fn IsDialog(&self, node: u64) -> bool {
        self.Read(|d| {
            d.FindNodeById(node)
                .is_some_and(|n| d.Node(n).IsHTMLElement("dialog"))
        })
    }
    fn IsAutofocusable(&self, node: u64) -> bool {
        self.Read(|d| {
            d.FindNodeById(node).is_some_and(|n| {
                matches!(
                    d.Node(n).Namespace(),
                    dom::DOMNamespace::kHTML | dom::DOMNamespace::kSVG
                ) && d.Node(n).FindAttribute("autofocus").is_some()
            })
        })
    }
    fn IsFocusable(&self, node: u64) -> bool {
        self.Read(|d| {
            d.FindNodeById(node).is_some_and(|index| {
                super::super::page_element_focusability::PageElementFocusability::new(
                    d,
                    self.dispatcher.focus_services.layout.as_ref(),
                    self.dispatcher.state.borrow().focused_node_id,
                )
                .IsFocusable(index)
            })
        })
    }
    fn AdjustedTabIndex(&self, node: u64) -> i32 {
        let navigation = self
            .dispatcher
            .focus_navigation
            .borrow()
            .clone()
            .expect("popup focus uses the current canonical FocusController");
        navigation.AdjustedTabIndex(node)
    }
    fn Connected(&self, node: u64) -> bool {
        DialogLifecycleEnvironment::Connected(self, node)
    }
    fn SetPreviousFocus(&self, dialog: u64, focused: Option<u64>) {
        self.Write(|d| {
            d.PopupLifecycleStateMut()
                .dialogs
                .entry(dialog)
                .or_default()
                .previously_focused_element = focused
        });
    }
    fn Focus(&self, node: u64) {
        Effects::Focus(self, node, false);
    }
    fn ClearFocusedElement(&self) {
        use interaction_default::core::page::focus_controller::FocusDispatchScope;
        self.dispatcher
            .Context()
            .focus_controller
            .ClearFocusedElementWithScope(&FocusDispatchScope {
                focus_events: Some(self.dispatcher),
                ..Default::default()
            });
    }
    fn IsModal(&self, node: u64) -> bool {
        LightDismissEnvironment::DialogIsModal(self, node)
    }
    fn FinalizeAutofocus(&self, _node: u64) {
        self.Write(|d| {
            if d.IsActive() {
                d.FinalizeAutofocus();
            }
        });
    }
}

impl DialogLifecycleEnvironment for Effects<'_, '_, '_> {
    type Error = PointerDomError;
    fn Open(&self, node: u64) -> bool {
        self.Read(|d| {
            d.FindNodeById(node)
                .is_some_and(|node| d.Node(node).FindAttribute("open").is_some())
        })
    }
    fn Active(&self) -> bool {
        self.Read(|d| d.IsActive())
    }
    fn Connected(&self, node: u64) -> bool {
        self.Read(|d| {
            d.FindNodeById(node)
                .is_some_and(|node| d.IsConnectedToDocument(node))
        })
    }
    fn Modal(&self, node: u64) -> bool {
        LightDismissEnvironment::DialogIsModal(self, node)
    }
    fn PopoverOpen(&self, node: u64) -> bool {
        Effects::Open(self, node)
    }
    fn BeginClose(&self, node: u64) -> bool {
        self.Write(|d| {
            let data = d.PopupLifecycleStateMut().dialogs.entry(node).or_default();
            if data.is_closing {
                return false;
            }
            data.is_closing = true;
            true
        })
    }
    fn EndClose(&self, node: u64) {
        self.Write(|d| {
            if let Some(data) = d.PopupLifecycleStateMut().dialogs.get_mut(&node) {
                data.is_closing = false;
            }
        });
    }
    fn ToggleEvents(
        &self,
        node: u64,
        opening: bool,
        source: Option<u64>,
        modal: bool,
    ) -> Result<bool, PointerDomError> {
        if !self.Toggle(node, "beforetoggle", !opening, opening, source, opening) {
            return Ok(false);
        }
        if opening
            && (DialogLifecycleEnvironment::Open(self, node)
                || (modal
                    && (!DialogLifecycleEnvironment::Connected(self, node)
                        || Effects::Open(self, node))))
        {
            return Ok(false);
        }
        let services = self.Services()?;
        let (old_open, pending) = self.Write(|d| {
            let data = d.PopupLifecycleStateMut().dialogs.entry(node).or_default();
            let old = data
                .pending_toggle_event
                .map_or(!opening, |event| event.old_open);
            let pending = data.pending_toggle_event_task.take();
            data.pending_toggle_event = Some(PopupToggleEvent {
                old_open: old,
                new_open: opening,
                source,
                composed: source.is_some()
                    && foundation::RuntimeEnabledFeatures::ShadowRootReferenceTargetEnabled(),
            });
            (old, pending)
        });
        if let Some(task) = pending {
            services.CancelTask(task);
        }
        let task = services.PostTask(
            0.0,
            PopupTask::Toggle {
                node,
                old_open,
                new_open: opening,
                source,
                dialog: true,
            },
        );
        self.Write(|d| {
            d.PopupLifecycleStateMut()
                .dialogs
                .entry(node)
                .or_default()
                .pending_toggle_event_task = Some(task)
        });
        Ok(true)
    }
    fn RemoveOpenAttribute(&self, node: u64) {
        (self.dispatcher.emit)(&DOMMutation {
            mutation_type: DOMMutationType::kRemoveAttribute,
            target_node_id: node,
            name: "open".into(),
            ..Default::default()
        });
    }
    fn SetModal(&self, node: u64, modal: bool) {
        self.Write(|d| {
            d.PopupLifecycleStateMut()
                .dialogs
                .entry(node)
                .or_default()
                .is_modal = modal;
            if let Some(index) = d.FindNodeById(node) {
                d.InvalidateNodeStyle(index);
            }
        });
    }
    fn EraseOpenDialog(&self, node: u64) {
        self.Write(|d| {
            d.PopupLifecycleStateMut()
                .all_open_dialogs
                .retain(|&id| id != node)
        });
    }
    fn ScheduleRemoval(&self, node: u64) {
        self.Write(|d| {
            if let Some(node) = d.FindNodeById(node) {
                d.ScheduleForTopLayerRemoval(node, PopupTopLayerReason::Dialog);
            }
        });
    }
    fn InertSubtreesChanged(&self) {
        self.Write(|d| d.InvalidateAllStyles());
    }
    fn SetReturnValue(&self, node: u64, value: Option<&[u16]>) {
        if let Some(value) = value {
            self.Write(|d| {
                d.PopupLifecycleStateMut()
                    .dialogs
                    .entry(node)
                    .or_default()
                    .return_value = value.to_vec()
            });
        }
    }
    fn ScheduleCloseEvent(&self, node: u64) -> Result<(), PointerDomError> {
        self.Services()?
            .PostTask(0.0, PopupTask::DialogClose { node });
        Ok(())
    }
    fn DestroyCloseWatcher(&self, node: u64) {
        let watcher = self.Write(|d| {
            d.PopupLifecycleStateMut()
                .dialogs
                .get_mut(&node)
                .and_then(|data| data.close_watcher.take())
        });
        if let Some(watcher) = watcher {
            self.DestroyWatcher(watcher);
        }
    }
    fn TakePreviousFocus(&self, node: u64) -> Option<u64> {
        self.Write(|d| {
            d.PopupLifecycleStateMut()
                .dialogs
                .get_mut(&node)
                .and_then(|data| data.previously_focused_element.take())
        })
    }
    fn ContainsFocused(&self, node: u64) -> bool {
        self.Read(|d| {
            self.dispatcher
                .state
                .borrow()
                .focused_node_id
                .and_then(|id| d.FindNodeById(id))
                .zip(d.FindNodeById(node))
                .is_some_and(|(focused, node)| d.ContainsNodeInTree(node, focused, true))
        })
    }
    fn Focus(&self, node: u64) {
        self.FocusWithSelection(
            node,
            interaction_default::core::dom::element::SelectionBehaviorOnFocus::kNone,
            true,
        );
    }
    fn AddToTopLayer(&self, node: u64) {
        self.Write(|d| {
            if let Some(node) = d.FindNodeById(node) {
                d.AddToTopLayer(node);
            }
        });
    }
    fn AddOpenAttribute(&self, node: u64) {
        (self.dispatcher.emit)(&DOMMutation {
            mutation_type: DOMMutationType::kSetAttribute,
            target_node_id: node,
            name: "open".into(),
            ..Default::default()
        });
        self.EnsureOpenDialog(node);
    }
    fn SetCloseWatcherEnabledState(&self, node: u64) {
        let enabled = LightDismissEnvironment::DialogClosedBy(self, node) != ClosedByState::None;
        let watcher = self.Read(|d| {
            d.PopupLifecycleState()
                .dialogs
                .get(&node)
                .and_then(|data| data.close_watcher)
        });
        if let Some(watcher) = watcher {
            CloseWatcher::setEnabled(self, watcher, enabled);
        }
    }
    fn HidePopoversForDialog(&self, node: u64) -> Result<(), PointerDomError> {
        let endpoint = PopoverLifecycleEnvironment::FindAncestor(
            self,
            node,
            None,
            if self.HintNewBehavior() {
                None
            } else if self.FindAncestor(node, None, Some(true)).is_some() {
                Some(true)
            } else {
                Some(false)
            },
        );
        LightDismissEnvironment::HideAllPopoversUntil(
            self,
            endpoint,
            HidePopoverFocusBehavior::None,
            HidePopoverTransitionBehavior::FireEventsAndWaitForTransitions,
        )
    }
    fn SetFocusForDialog(&self, node: u64) {
        self.FocusDialogControl(node);
    }
    fn InvalidState(&self, message: &'static str) -> PointerDomError {
        PointerDomError::Lifecycle(message.into())
    }
    fn SetRequestClose(&self, node: u64, value: Option<&[u16]>, source: Option<u64>) {
        let watcher = self.Write(|d| {
            let data = d.PopupLifecycleStateMut().dialogs.entry(node).or_default();
            data.request_close_return_value = value.map(|value| value.to_vec());
            data.request_close_source_element = source;
            data.close_watcher
        });
        if let Some(watcher) = watcher {
            CloseWatcher::setEnabled(self, watcher, true);
        }
    }
    fn CloseWatcherRequestCloseAlways(&self, node: u64) -> Result<(), PointerDomError> {
        self.CloseWatcherRequestCloseAlwaysImpl(node)
    }
}

impl PopoverLifecycleEnvironment for Effects<'_, '_, '_> {
    type Error = PointerDomError;
    fn Ready(&self, node: u64, show: bool) -> bool {
        self.Read(|d| {
            let Some(index) = d.FindNodeById(node) else {
                return false;
            };
            d.IsActive()
                && d.IsPopover(index)
                && d.IsConnectedToDocument(index)
                && d.PopoverOpen(index) != show
                && (!show || (!d.DialogIsModal(index) && d.FullscreenElement() != Some(index)))
        })
    }
    fn Kind(&self, node: u64) -> PopoverValueType {
        self.Read(|d| {
            d.PopupLifecycleState()
                .popovers
                .get(&node)
                .map_or(PopoverValueType::None, |p| p.popover_type)
        })
    }
    fn Open(&self, node: u64) -> bool {
        Effects::Open(self, node)
    }
    fn HintNewBehavior(&self) -> bool {
        foundation::RuntimeEnabledFeatures::PopoverHintNewBehaviorEnabled()
    }
    fn StackLen(&self, hint: bool) -> usize {
        self.Read(|d| {
            if hint {
                d.PopupLifecycleState().hint_stack.len()
            } else {
                d.PopupLifecycleState().auto_stack.len()
            }
        })
    }
    fn StackAt(&self, hint: bool, index: usize) -> u64 {
        self.Read(|d| {
            if hint {
                d.PopupLifecycleState().hint_stack[index]
            } else {
                d.PopupLifecycleState().auto_stack[index]
            }
        })
    }
    fn StackContains(&self, hint: bool, node: u64) -> bool {
        self.Read(|d| {
            if hint {
                d.PopupLifecycleState().hint_stack.contains(&node)
            } else {
                d.PopupLifecycleState().auto_stack.contains(&node)
            }
        })
    }
    fn StackPop(&self, hint: bool) -> Option<u64> {
        self.Write(|d| {
            if hint {
                d.PopupLifecycleStateMut().hint_stack.pop()
            } else {
                d.PopupLifecycleStateMut().auto_stack.pop()
            }
        })
    }
    fn StackPush(&self, hint: bool, node: u64) {
        self.Write(|d| {
            if hint {
                d.PopupLifecycleStateMut().hint_stack.push(node)
            } else {
                d.PopupLifecycleStateMut().auto_stack.push(node)
            }
        });
    }
    fn HintParent(&self) -> Option<u64> {
        self.Read(|d| d.PopupLifecycleState().hint_stack_parent)
    }
    fn SetHintParent(&self, node: Option<u64>) {
        self.Write(|d| d.PopupLifecycleStateMut().hint_stack_parent = node);
    }
    fn HidingNestingCount(&self) -> usize {
        self.Read(|d| d.PopupLifecycleState().popover_hiding_nesting_count)
    }
    fn BeginHide(&self, node: u64) -> bool {
        self.Write(|d| {
            let s = d.PopupLifecycleStateMut();
            let was = s.popovers.get(&node).is_some_and(|p| p.hiding_this_popover);
            if let Some(p) = s.popovers.get_mut(&node) {
                p.hiding_this_popover = true;
            }
            s.popover_hiding_nesting_count += 1;
            was
        })
    }
    fn EndHide(&self, node: u64, was_hiding: bool) {
        self.Write(|d| {
            let s = d.PopupLifecycleStateMut();
            if let Some(p) = s.popovers.get_mut(&node) {
                p.hiding_this_popover = was_hiding;
            }
            s.popover_hiding_nesting_count = s
                .popover_hiding_nesting_count
                .checked_sub(1)
                .expect("balanced ScopedPopoverHiding");
        });
    }
    fn BeginShow(&self) -> bool {
        self.Write(|d| std::mem::replace(&mut d.PopupLifecycleStateMut().popover_showing, true))
    }
    fn EndShow(&self, was: bool) {
        self.Write(|d| d.PopupLifecycleStateMut().popover_showing = was);
    }
    fn ForcedOpen(&self, node: u64) -> bool {
        self.Read(|d| {
            d.PopupLifecycleState()
                .inspector_forced_popovers
                .contains(&node)
        })
    }
    fn BeforeToggle(&self, node: u64, opening: bool, source: Option<u64>) -> bool {
        self.Toggle(node, "beforetoggle", !opening, opening, source, opening)
    }
    fn QueueToggle(
        &self,
        node: u64,
        opening: bool,
        source: Option<u64>,
    ) -> Result<(), PointerDomError> {
        let services = self.Services()?;
        let (old_open, pending) = self.Write(|d| {
            let Some(data) = d.PopupLifecycleStateMut().popovers.get_mut(&node) else {
                return (!opening, None);
            };
            let pending = data.pending_toggle_event_task.take();
            let old = if pending.is_some() {
                !data.pending_toggle_event_started_closed
            } else {
                !opening
            };
            if pending.is_none() {
                data.pending_toggle_event_started_closed = opening;
            }
            (old, pending)
        });
        if let Some(pending) = pending {
            services.CancelTask(pending);
        }
        let task = services.PostTask(
            0.0,
            PopupTask::Toggle {
                node,
                old_open,
                new_open: opening,
                source,
                dialog: false,
            },
        );
        self.Write(|d| {
            if let Some(data) = d.PopupLifecycleStateMut().popovers.get_mut(&node) {
                data.pending_toggle_event_task = Some(task);
            }
        });
        Ok(())
    }
    fn MarkInvokersDirty(&self, node: u64) {
        self.InvalidateInterest(node);
    }
    fn ScheduleRemoval(&self, node: u64) {
        self.Write(|d| {
            if let Some(node) = d.FindNodeById(node) {
                d.ScheduleForTopLayerRemoval(node, PopupTopLayerReason::Popover);
            }
        });
    }
    fn RemoveImmediately(&self, node: u64) {
        self.Write(|d| {
            if let Some(node) = d.FindNodeById(node) {
                d.RemoveFromTopLayerImmediately(node);
            }
        });
    }
    fn CommitHidden(&self, node: u64) {
        self.Write(|d| {
            let Some(index) = d.FindNodeById(node) else {
                return;
            };
            {
                let s = d.PopupLifecycleStateMut();
                s.auto_stack.retain(|&id| id != node);
                s.hint_stack.retain(|&id| id != node);
                if s.hint_stack.is_empty() {
                    s.hint_stack_parent = None;
                }
                if let Some(data) = s.popovers.get_mut(&node) {
                    data.visibility_state = PopoverVisibilityState::Hidden;
                }
            }
            d.SetPopoverInvoker(index, None);
            d.RemoveAllOpenPopover(index);
            d.InvalidateNodeStyle(index);
            if let Some(host) = d.ShadowHost(d.TreeScopeRoot(index)) {
                d.InvalidateNodeStyle(host);
            }
        });
    }
    fn CommitShown(&self, node: u64, invoker: Option<u64>, stack_hint: Option<bool>) {
        self.Write(|d| {
            let Some(index) = d.FindNodeById(node) else {
                return;
            };
            let watcher = stack_hint.map(|_| {
                d.PopupLifecycleStateMut()
                    .CreateCloseWatcher(CloseWatcherOwner::Popover(node))
            });
            // Stack insertion follows after releasing this DOM write borrow.
            {
                let s = d.PopupLifecycleStateMut();
                if let Some(hint) = stack_hint {
                    let stack = if hint {
                        &mut s.hint_stack
                    } else {
                        &mut s.auto_stack
                    };
                    assert!(!stack.contains(&node));
                    stack.push(node);
                }
                let data = s.popovers.entry(node).or_default();
                data.previously_focused_element = None;
                data.visibility_state = PopoverVisibilityState::Showing;
                data.implicit_anchor = invoker;
                data.close_watcher = watcher;
            }
            d.AddToTopLayer(index);
            let source = invoker.and_then(|id| d.FindNodeById(id));
            d.SetPopoverInvoker(index, source);
            d.AddAllOpenPopover(index);
            d.InvalidateNodeStyle(index);
            if let Some(host) = d.ShadowHost(d.TreeScopeRoot(index)) {
                d.InvalidateNodeStyle(host);
            }
        });
        let watcher = self.Read(|d| {
            d.PopupLifecycleState()
                .popovers
                .get(&node)
                .and_then(|data| data.close_watcher)
        });
        if let Some(watcher) = watcher {
            WatcherStack::Add(self, watcher);
        }
    }
    fn LoseInterestWithoutClosing(&self, node: u64) -> Result<(), PointerDomError> {
        if let Some(invoker) = InterestEnvironment::SourceInterestInvoker(self, node) {
            element_interest::InterestLost(
                self,
                invoker,
                Some(node),
                InterestLostCancelable::Cancelable,
                InterestLostPopoverBehavior::DontClosePopovers,
            )?;
        }
        Ok(())
    }
    fn FocusOnShow(&self, node: u64) {
        self.dispatcher
            .focus_services
            .lifecycle
            .UpdateStyleAndLayoutTreeForElement(node);
        if FocusDelegateEnvironment::IsDialog(self, node) {
            focus_delegate::SetFocusForDialog(self, node);
            return;
        }
        let control = if FocusDelegateEnvironment::IsAutofocusable(self, node) {
            Some(node)
        } else {
            focus_delegate::GetAutofocusDelegate(self, node)
        };
        if let Some(control) = control {
            FocusDelegateEnvironment::Focus(self, control);
            FocusDelegateEnvironment::FinalizeAutofocus(self, control);
        }
    }
    fn RestoreFocusOnHide(&self, node: u64, focus: HidePopoverFocusBehavior) {
        let (previous, watcher) = self.Write(|d| {
            d.PopupLifecycleStateMut()
                .popovers
                .get_mut(&node)
                .map_or((None, None), |data| {
                    (
                        data.previously_focused_element.take(),
                        data.close_watcher.take(),
                    )
                })
        });
        if let Some(previous) = previous {
            let contains = self.Read(|d| {
                self.dispatcher
                    .state
                    .borrow()
                    .focused_node_id
                    .and_then(|id| d.FindNodeById(id))
                    .zip(d.FindNodeById(node))
                    .is_some_and(|(focused, node)| d.ContainsNodeInTree(node, focused, false))
            });
            if focus == HidePopoverFocusBehavior::FocusPreviousElement && contains {
                if self.InterestTarget(previous) == Some(node) {
                    self.Write(|d| d.SuppressNextFocusInterest(previous));
                }
                self.Focus(previous, true);
            }
        }
        if let Some(watcher) = watcher {
            self.DestroyWatcher(watcher);
        }
    }
    fn FocusedElement(&self) -> Option<u64> {
        self.dispatcher.state.borrow().focused_node_id
    }
    fn RememberPreviouslyFocused(&self, node: u64, focused: Option<u64>) {
        self.Write(|d| {
            if let Some(data) = d.PopupLifecycleStateMut().popovers.get_mut(&node) {
                data.previously_focused_element = focused;
            }
        });
    }
    fn FindAncestor(
        &self,
        node: u64,
        invoker: Option<u64>,
        stack_hint: Option<bool>,
    ) -> Option<u64> {
        let mut positions = std::collections::HashMap::new();
        let mut index = 0;
        for hint in [false, true] {
            if stack_hint.is_some_and(|wanted| wanted != hint) {
                continue;
            }
            for node in self.Stack(hint) {
                positions.insert(node, index);
                index += 1;
            }
        }
        positions.insert(node, index);
        let mut best = None;
        for mut candidate in [
            InterestEnvironment::ParentElementOrLocalOwner(self, node),
            invoker,
        ] {
            while let Some(current) = candidate {
                let Some(ancestor) = LightDismissEnvironment::NearestOpenPopover(self, current)
                else {
                    break;
                };
                if !positions.contains_key(&ancestor) {
                    break;
                }
                if !self.HintNewBehavior()
                    && self.Kind(node) != PopoverValueType::Hint
                    && self.Kind(ancestor) != PopoverValueType::Auto
                {
                    candidate = InterestEnvironment::ParentElementOrLocalOwner(self, ancestor);
                    continue;
                }
                if best.is_none_or(|best| positions[&best] < positions[&ancestor]) {
                    best = Some(ancestor);
                }
                break;
            }
        }
        best
    }
    fn WarningReentrantPopover(&self) {
        eprintln!("rechrom-popup-warning\tbeforetoggle handler opened another popover");
    }
    fn AttributeType(&self, node: u64) -> PopoverValueType {
        self.Read(|d| {
            let Some(node) = d.FindNodeById(node) else {
                return PopoverValueType::None;
            };
            match d
                .Node(node)
                .FindAttribute("popover")
                .map(|attr| attr.value.to_ascii_lowercase())
                .as_deref()
            {
                Some("" | "auto") => PopoverValueType::Auto,
                Some("hint") => PopoverValueType::Hint,
                Some(_) => PopoverValueType::Manual,
                None => PopoverValueType::None,
            }
        })
    }
    fn SetType(&self, node: u64, kind: PopoverValueType) {
        self.Write(|d| {
            d.PopupLifecycleStateMut()
                .popovers
                .entry(node)
                .or_default()
                .popover_type = kind
        });
    }
    fn RemoveData(&self, node: u64) {
        self.Write(|d| {
            if let Some(index) = d.FindNodeById(node) {
                d.SetPopoverInvoker(index, None);
            }
            d.PopupLifecycleStateMut().popovers.remove(&node);
        });
    }
    fn IsMenuList(&self, node: u64) -> bool {
        self.Read(|d| {
            d.FindNodeById(node)
                .is_some_and(|node| d.Node(node).IsHTMLElement("menulist"))
        })
    }
    fn DatalistManagesPopover(&self, node: u64) -> bool {
        self.Read(|d| {
            d.SelectSourceFeatures().customizable_combobox
                && d.FindNodeById(node)
                    .is_some_and(|node| d.Node(node).IsHTMLElement("datalist"))
        })
    }
    fn MaybeCreateMenuTriangle(
        &self,
        node: u64,
        invoker: Option<u64>,
    ) -> Result<(), PointerDomError> {
        if !self.Read(|d| d.PopupLifecycleState().menu_elements_enabled) {
            return Ok(());
        }
        if let Some(invoker) = invoker {
            let submenu = self.Read(|d| {
                let index = d.FindNodeById(invoker)?;
                d.Node(index)
                    .IsHTMLElement("menuitem")
                    .then(|| d.MenuItemInvokedSubmenu(index))
                    .flatten()
                    .map(|n| d.Node(n).Id())
            });
            if submenu == Some(node) {
                menu_safe_triangle::MaybeCreate(self, invoker, node)?;
            }
        }
        Ok(())
    }
    fn RecheckMenuTriangleIfMenuList(&self, node: u64) -> Result<(), PointerDomError> {
        if !PopoverLifecycleEnvironment::IsMenuList(self, node) {
            return Ok(());
        }
        if let Some(id) = MenuSafeTriangleEnvironment::Active(self) {
            menu_safe_triangle::Recheck(self, id)?;
        }
        Ok(())
    }
}

impl LightDismissEnvironment for Effects<'_, '_, '_> {
    type Error = PointerDomError;
    fn TopmostPopoverOrHint(&self) -> Option<u64> {
        self.Read(|d| {
            d.PopupLifecycleState()
                .hint_stack
                .last()
                .or(d.PopupLifecycleState().auto_stack.last())
                .copied()
        })
    }
    fn NearestOpenPopover(&self, node: u64) -> Option<u64> {
        self.Read(|d| {
            let mut node = d.FindNodeById(node);
            while let Some(index) = node {
                if d.PopoverOpen(index)
                    && d.PopupLifecycleState()
                        .popovers
                        .get(&d.Node(index).Id())
                        .is_some_and(|p| p.popover_type != PopoverValueType::Manual)
                {
                    return Some(d.Node(index).Id());
                }
                node = d.FlatTreeParent(index);
            }
            None
        })
    }
    fn NearestTargetPopoverForInvoker(&self, node: u64) -> Option<u64> {
        let mut node = Some(node);
        while let Some(id) = node {
            let target = self.Read(|d| {
                let index = d.FindNodeById(id)?;
                let n = d.Node(index);
                let command = (n.IsHTMLElement("button") || n.IsHTMLElement("menuitem"))
                    .then(|| d.GetElementAttributeResolvingReferenceTarget(index, "commandfor"))
                    .flatten();
                let popover_target =
                    d.GetElementAttributeResolvingReferenceTarget(index, "popovertarget");
                command
                    .or(popover_target)
                    .or_else(|| self.InterestTarget(id).and_then(|id| d.FindNodeById(id)))
                    .filter(|&target| {
                        d.PopoverOpen(target)
                            && d.PopupLifecycleState()
                                .popovers
                                .get(&d.Node(target).Id())
                                .is_some_and(|p| p.popover_type != PopoverValueType::Manual)
                    })
                    .map(|target| d.Node(target).Id())
            });
            if target.is_some() {
                return target;
            }
            node = LightDismissEnvironment::FlatTreeParent(self, id);
        }
        None
    }
    fn PopoverStackPosition(&self, popover: Option<u64>) -> usize {
        self.Read(|d| {
            let s = d.PopupLifecycleState();
            popover.map_or(0, |id| {
                s.hint_stack
                    .iter()
                    .position(|&n| n == id)
                    .map(|p| p + s.auto_stack.len() + 1)
                    .or_else(|| s.auto_stack.iter().position(|&n| n == id).map(|p| p + 1))
                    .unwrap_or(0)
            })
        })
    }
    fn PopoverPointerdownTarget(&self) -> Option<u64> {
        self.Read(|d| d.PopupLifecycleState().popover_pointerdown_target)
    }
    fn SetPopoverPointerdownTarget(&self, target: Option<u64>) {
        self.Write(|d| d.PopupLifecycleStateMut().popover_pointerdown_target = target);
    }
    fn HideAllPopoversUntil(
        &self,
        endpoint: Option<u64>,
        focus: HidePopoverFocusBehavior,
        transition: HidePopoverTransitionBehavior,
    ) -> Result<(), PointerDomError> {
        popover_lifecycle::HideAllPopoversUntil(
            self,
            endpoint,
            focus,
            transition,
            &mut Vec::new(),
        )?;
        Ok(())
    }
    fn TopmostOpenDialog(&self) -> Option<u64> {
        self.Read(|d| d.PopupLifecycleState().all_open_dialogs.last().copied())
    }
    fn FlatTreeParent(&self, node: u64) -> Option<u64> {
        self.Read(|d| {
            d.FindNodeById(node)
                .and_then(|node| d.FlatTreeParent(node))
                .map(|node| d.Node(node).Id())
        })
    }
    fn BackdropDialog(&self, node: u64) -> Option<u64> {
        self.Read(|d| {
            let node = d.FindNodeById(node)?;
            let facts = d.PseudoElementFacts(node)?;
            (facts.pseudo_id
                == layoutng_style::style::computed_style_constants::PseudoId::kPseudoIdBackdrop
                && d.Node(facts.originating_element).IsHTMLElement("dialog"))
            .then(|| d.Node(facts.originating_element).Id())
        })
    }
    fn IsOpenAndActiveDialog(&self, node: u64) -> bool {
        self.Read(|d| {
            d.IsActive()
                && d.FindNodeById(node).is_some_and(|node| {
                    d.Node(node).IsHTMLElement("dialog")
                        && d.Node(node).FindAttribute("open").is_some()
                        && d.IsConnectedToDocument(node)
                })
        })
    }
    fn DialogIsModal(&self, node: u64) -> bool {
        self.Read(|d| {
            d.FindNodeById(node)
                .is_some_and(|node| d.DialogIsModal(node))
        })
    }
    fn DialogRectContains(&self, node: u64, x: f64, y: f64) -> Result<bool, PointerDomError> {
        self.Services()?.DialogRectContains(node, x, y)
    }
    fn DialogPointerdownTarget(&self) -> Option<u64> {
        self.Read(|d| d.PopupLifecycleState().dialog_pointerdown_target)
    }
    fn SetDialogPointerdownTarget(&self, target: Option<u64>) {
        self.Write(|d| d.PopupLifecycleStateMut().dialog_pointerdown_target = target);
    }
    fn DialogClosedBy(&self, node: u64) -> ClosedByState {
        self.Read(|d| {
            let Some(index) = d.FindNodeById(node) else {
                return ClosedByState::None;
            };
            match d
                .Node(index)
                .FindAttribute("closedby")
                .map(|a| a.value.to_ascii_lowercase())
                .as_deref()
            {
                Some("any") => ClosedByState::Any,
                Some("none") => ClosedByState::None,
                Some("closerequest") => ClosedByState::CloseRequest,
                _ if d.DialogIsModal(index) => ClosedByState::CloseRequest,
                _ => ClosedByState::None,
            }
        })
    }
    fn RequestClose(&self, node: u64) -> Result<(), PointerDomError> {
        self.RequestDialogClose(node)
    }
}

impl InterestEnvironment for Effects<'_, '_, '_> {
    type Error = PointerDomError;
    fn IsInActiveTreeScope(&self, node: u64) -> bool {
        self.Read(|d| {
            d.IsActive()
                && d.FindNodeById(node)
                    .is_some_and(|node| d.IsInTreeScope(node))
        })
    }
    fn ParentElementOrLocalOwner(&self, node: u64) -> Option<u64> {
        self.Read(|d| {
            let mut parent = d.FindNodeById(node).and_then(|node| d.FlatTreeParent(node));
            while let Some(node) = parent {
                if d.Node(node).Type() == dom::DOMNodeType::kElement {
                    return Some(d.Node(node).Id());
                }
                parent = d.FlatTreeParent(node);
            }
            None
        })
    }
    fn InterestForElement(&self, node: u64) -> Option<u64> {
        self.InterestTarget(node)
    }
    fn SourceInterestInvoker(&self, node: u64) -> Option<u64> {
        let invoker =
            self.Read(|d| d.PopupLifecycleState().interest_targets.get(&node).copied())?;
        self.InterestTarget(invoker).map(|_| invoker)
    }
    fn State(&self, node: u64) -> InterestState {
        self.Read(|d| {
            d.PopupLifecycleState()
                .interest_invokers
                .get(&node)
                .map_or(InterestState::NoInterest, |data| data.state)
        })
    }
    fn HasInvokerData(&self, node: u64) -> bool {
        self.Read(|d| {
            d.PopupLifecycleState()
                .interest_invokers
                .contains_key(&node)
        })
    }
    fn TakeSuppressNextFocusInterest(&self, node: u64) -> bool {
        self.Write(|d| {
            d.PopupLifecycleStateMut()
                .interest_invokers
                .get_mut(&node)
                .is_some_and(|data| std::mem::take(&mut data.suppress_next_focus_interest))
        })
    }
    fn CancelGainedTask(&self, node: u64) {
        self.Cancel(node, true)
    }
    fn CancelLostTask(&self, node: u64) {
        self.Cancel(node, false)
    }
    fn ScheduleGainedTask(&self, node: u64) -> Result<(), PointerDomError> {
        self.Schedule(node, true)
    }
    fn ScheduleLostTask(&self, node: u64) -> Result<(), PointerDomError> {
        self.Schedule(node, false)
    }
    fn DeferGained(
        &self,
        invoker: u64,
        target: u64,
        state: InterestState,
    ) -> Result<bool, PointerDomError> {
        let Some(id) = MenuSafeTriangleEnvironment::Active(self) else {
            return Ok(false);
        };
        menu_safe_triangle::ShouldDeferInterestGained(self, id, invoker, target, state)
    }
    fn DeferLost(
        &self,
        invoker: u64,
        target: u64,
        cancelable: InterestLostCancelable,
        behavior: InterestLostPopoverBehavior,
    ) -> Result<bool, PointerDomError> {
        let Some(id) = MenuSafeTriangleEnvironment::Active(self) else {
            return Ok(false);
        };
        menu_safe_triangle::ShouldDeferInterestLost(self, id, invoker, target, cancelable, behavior)
    }
    fn ForcedInterest(&self, invoker: u64) -> bool {
        self.Read(|d| {
            d.PopupLifecycleState()
                .inspector_forced_interest_invokers
                .contains(&invoker)
        })
    }
    fn DispatchInterest(
        &self,
        kind: &'static str,
        target: u64,
        invoker: u64,
        cancelable: bool,
    ) -> bool {
        let Some(source) = self.dispatcher.targets.ResolveTarget(invoker) else {
            return false;
        };
        let references = foundation::RuntimeEnabledFeatures::ShadowRootReferenceTargetEnabled();
        let mut event = InterestEvent::new(
            kind,
            source,
            cancelable,
            !references,
            references,
            self.dispatcher.clock.as_ref(),
        );
        event.BaseEventMut().SetTrusted(true);
        let event = EventRef::new(event);
        super::ports::DispatchCanonical(self.dispatcher, event.clone(), target);
        let canceled = event.borrow().BaseEvent().defaultPrevented();
        canceled
    }
    fn ChangeInterestState(&self, invoker: u64, target: Option<u64>, state: InterestState) {
        if InterestEnvironment::State(self, invoker) == state {
            return;
        }
        if state == InterestState::NoInterest {
            self.Cancel(invoker, true);
            self.Cancel(invoker, false);
        } else {
            self.Cancel(invoker, true);
        }
        self.Write(|d| {
            let s = d.PopupLifecycleStateMut();
            let data = s.interest_invokers.entry(invoker).or_default();
            if state == InterestState::NoInterest {
                s.elements_with_interest.retain(|&n| n != invoker);
                data.active_target = None;
                if let Some(target) = target {
                    s.interest_targets.remove(&target);
                }
            } else if data.state == InterestState::NoInterest {
                s.elements_with_interest.push(invoker);
                data.active_target = target;
            }
            data.state = state;
        });
        self.InvalidateInterest(invoker);
        if let Some(target) = target {
            self.InvalidateInterest(target);
        }
    }
    fn SetSourceInterestInvoker(&self, target: u64, invoker: u64) {
        self.Write(|d| {
            d.PopupLifecycleStateMut()
                .interest_targets
                .insert(target, invoker);
        });
    }
    fn ShowPopoverForInterest(&self, target: u64, invoker: u64) -> Result<(), PointerDomError> {
        if self.Read(|d| {
            d.FindNodeById(target)
                .is_some_and(|target| d.IsPopover(target) && !d.PopoverOpen(target))
        }) {
            popover_lifecycle::ShowPopoverInternal(self, target, Some(invoker))?;
        }
        Ok(())
    }
    fn HidePopoverForInterest(&self, target: u64, invoker: u64) -> Result<(), PointerDomError> {
        if self.Open(target) {
            popover_lifecycle::HidePopoverInternal(
                self,
                target,
                Some(invoker),
                HidePopoverFocusBehavior::FocusPreviousElement,
                HidePopoverTransitionBehavior::FireEventsAndWaitForTransitions,
            )?;
        }
        Ok(())
    }
}

pub(crate) fn HandlePopoverLightDismiss(
    dispatcher: &EventDispatcher<'_, '_>,
    event: &PointerEventRef,
    target: &EventTargetRef,
) -> Result<(), PointerDomError> {
    let e = Effects { dispatcher };
    let Some(target) = e.Id(target) else {
        return Ok(());
    };
    let kind = event.borrow().BaseEvent().r#type().to_owned();
    pointer_light_dismiss::HandlePopoverLightDismiss(&e, &kind, target)
}
pub(crate) fn HandleDialogLightDismiss(
    dispatcher: &EventDispatcher<'_, '_>,
    event: &PointerEventRef,
    target: &EventTargetRef,
) -> Result<(), PointerDomError> {
    let e = Effects { dispatcher };
    let Some(target) = e.Id(target) else {
        return Ok(());
    };
    let (kind, x, y) = {
        let event = event.borrow();
        (
            event.BaseEvent().r#type().to_owned(),
            event.clientX(),
            event.clientY(),
        )
    };
    pointer_light_dismiss::HandleDialogLightDismiss(&e, &kind, target, x, y)
}
pub(crate) fn HandlePointerEventsForInterestFor(
    dispatcher: &EventDispatcher<'_, '_>,
    target: &EventTargetRef,
    kind: &str,
) -> Result<(), PointerDomError> {
    let e = Effects { dispatcher };
    let Some(target) = e.Id(target) else {
        return Ok(());
    };
    element_interest::HandlePointerEventsForInterestFor(&e, target, kind)
}
pub(crate) fn AnchorElementInteractionTrackerOnPointerEvent(
    dispatcher: &EventDispatcher<'_, '_>,
    target: &EventTargetRef,
    event: &PointerEventRef,
) {
    let owner = dispatcher
        .document
        .borrow()
        .GetDocument()
        .AnchorElementInteractionTracker();
    if let Some(owner) = owner {
        let tracker = owner
            .downcast::<AnchorElementInteractionTracker>()
            .expect("Document anchor tracker retains its canonical source owner");
        tracker.OnPointerEvent(target, event);
    }
}
/// event_handler.cc:1093-1098. A missing optional document tracker takes the
/// source null-owner branch; an installed tracker receives the real event.
pub(crate) fn AnchorElementInteractionTrackerOnMouseMoveEvent(
    dispatcher: &EventDispatcher<'_, '_>,
    event: &WebMouseEvent,
) {
    let owner = dispatcher
        .document
        .borrow()
        .GetDocument()
        .AnchorElementInteractionTracker();
    if let Some(owner) = owner {
        owner
            .downcast::<AnchorElementInteractionTracker>()
            .expect("Document anchor tracker retains its canonical source owner")
            .OnMouseMoveEvent(event);
    }
}
pub fn RunPopupTask(
    dispatcher: &EventDispatcher<'_, '_>,
    task: PopupTask,
) -> Result<(), PointerDomError> {
    let e = Effects { dispatcher };
    match task {
        PopupTask::InterestGained { invoker, target } => {
            element_interest::InterestGained(
                &e,
                invoker,
                Some(target),
                InterestState::FullInterest,
            )?;
        }
        PopupTask::InterestLost { invoker, target } => {
            element_interest::InterestLost(
                &e,
                invoker,
                Some(target),
                InterestLostCancelable::Cancelable,
                InterestLostPopoverBehavior::ClosePopovers,
            )?;
        }
        PopupTask::Toggle {
            node,
            old_open,
            new_open,
            source,
            dialog,
        } => {
            e.Write(|d| {
                if dialog {
                    if let Some(data) = d.PopupLifecycleStateMut().dialogs.get_mut(&node) {
                        data.pending_toggle_event_task = None;
                        data.pending_toggle_event = None;
                    }
                } else if let Some(data) = d.PopupLifecycleStateMut().popovers.get_mut(&node) {
                    data.pending_toggle_event_task = None;
                }
            });
            e.Toggle(node, "toggle", old_open, new_open, source, false);
        }
        PopupTask::DialogClose { node } => {
            let mut event =
                interaction_dispatch::core::dom::events::event::Event::CreateWithInitializer(
                    "close",
                    &EventInit::default(),
                    dispatcher.clock.as_ref(),
                );
            event.SetTrusted(true);
            super::ports::DispatchCanonical(dispatcher, EventRef::new(event), node);
        }
        PopupTask::MenuTriangleExpire { triangle } => {
            if MenuSafeTriangleEnvironment::Active(&e) == Some(triangle) {
                menu_safe_triangle::Finish(&e, triangle, true)?;
            }
        }
    }
    Ok(())
}

pub(crate) fn RecheckMenuTriangle(
    dispatcher: &EventDispatcher<'_, '_>,
) -> Result<(), PointerDomError> {
    let e = Effects { dispatcher };
    if let Some(id) = MenuSafeTriangleEnvironment::Active(&e) {
        menu_safe_triangle::Recheck(&e, id)?;
    }
    Ok(())
}

pub fn SetPopoverOpen(
    dispatcher: &EventDispatcher<'_, '_>,
    node: u64,
    open: bool,
    source: Option<u64>,
) -> Result<(), PointerDomError> {
    let e = Effects { dispatcher };
    popover_lifecycle::UpdatePopoverAttribute(&e, node)?;
    if e.Kind(node) == PopoverValueType::None {
        return Err(PointerDomError::Lifecycle(
            "element is not a popover".into(),
        ));
    }
    if !DialogLifecycleEnvironment::Active(&e) || !DialogLifecycleEnvironment::Connected(&e, node) {
        return Err(PointerDomError::Lifecycle(
            "popover requires a connected active Document".into(),
        ));
    }
    if !e.Ready(node, open) {
        return Ok(());
    }
    if open {
        if e.HintNewBehavior()
            && e.Read(|d| {
                d.PopupLifecycleState().popover_showing
                    || d.PopupLifecycleState().popover_hiding_nesting_count != 0
            })
        {
            return Err(PointerDomError::Lifecycle(
                "invalid nested showPopover operation".into(),
            ));
        }
        popover_lifecycle::ShowPopoverInternal(&e, node, source)
    } else {
        popover_lifecycle::HidePopoverInternal(
            &e,
            node,
            source,
            HidePopoverFocusBehavior::FocusPreviousElement,
            HidePopoverTransitionBehavior::FireEventsAndWaitForTransitions,
        )?;
        Ok(())
    }
}

/// Form method=dialog invokes close(), without a cancel event, and preserves
/// the submitter's exact DOMString as returnValue.
pub(crate) fn CloseDialogForForm(dispatcher: &EventDispatcher<'_, '_>, node: u64, result: &[u16]) {
    dialog_lifecycle::Close(&Effects { dispatcher }, node, Some(result), None, false)
        .unwrap_or_else(|error| std::panic::panic_any(error));
}

pub fn RequestDialogClose(
    dispatcher: &EventDispatcher<'_, '_>,
    node: u64,
) -> Result<(), PointerDomError> {
    Effects { dispatcher }.RequestDialogClose(node)
}

pub fn ApplyPopupMutation(
    dispatcher: &EventDispatcher<'_, '_>,
    mutation: &DOMMutation,
) -> Result<bool, PointerDomError> {
    let e = Effects { dispatcher };
    match mutation.mutation_type {
        DOMMutationType::kSetPopoverOpen => {
            SetPopoverOpen(
                dispatcher,
                mutation.target_node_id,
                mutation.bool_value,
                (mutation.source_node_id != 0).then_some(mutation.source_node_id),
            )?;
            Ok(true)
        }
        DOMMutationType::kSetDialogModal => {
            let node = mutation.target_node_id;
            if !e.Read(|d| {
                d.FindNodeById(node)
                    .is_some_and(|node| d.Node(node).IsHTMLElement("dialog"))
            }) {
                return Err(PointerDomError::Lifecycle(
                    "dialog method requires a dialog receiver".into(),
                ));
            }
            if mutation.bool_value {
                dialog_lifecycle::ShowModal(&e, node, None)?;
            } else {
                dialog_lifecycle::Close(&e, node, mutation.utf16_value.as_deref(), None, false)?;
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub fn DrainPopupLifecycle(dispatcher: &EventDispatcher<'_, '_>) -> Result<(), PointerDomError> {
    let e = Effects { dispatcher };
    let queue = e.Read(|d| d.AttributeLifecycle().clone());
    let mut changes = queue.TakePopupLifecycleChanges();
    while let Some(change) = changes.pop_front() {
        match change {
            dom::attribute_lifecycle::PopupLifecycleChange::Attribute(change) => {
                let dialog = e.Read(|d| {
                    d.FindNodeById(change.node)
                        .is_some_and(|node| d.Node(node).IsHTMLElement("dialog"))
                });
                match change.name.as_str() {
                    "popover" => popover_lifecycle::UpdatePopoverAttribute(&e, change.node)?,
                    "open" if dialog => {
                        let closing = e.Read(|d| {
                            d.PopupLifecycleState()
                                .dialogs
                                .get(&change.node)
                                .is_some_and(|data| data.is_closing)
                        });
                        if !closing {
                            if change.new_value.is_none() {
                                if foundation::RuntimeEnabledFeatures::DialogCloseWhenOpenRemovedEnabled(){dialog_lifecycle::Close(&e,change.node,None,None,true)?;}else{e.EraseOpenDialog(change.node);e.DestroyCloseWatcher(change.node);}
                            } else if change.old_value.is_none()
                                && DialogLifecycleEnvironment::Connected(&e, change.node)
                            {
                                e.EnsureOpenDialog(change.node);
                            }
                        }
                    }
                    "closedby" if dialog => e.SetCloseWatcherEnabledState(change.node),
                    "interestfor" => {
                        let target = e.Read(|d| {
                            d.PopupLifecycleState()
                                .interest_invokers
                                .get(&change.node)
                                .and_then(|data| data.active_target)
                        });
                        if target != e.InterestTarget(change.node) {
                            e.ChangeInterestState(change.node, target, InterestState::NoInterest);
                        }
                    }
                    _ => {}
                }
            }
            dom::attribute_lifecycle::PopupLifecycleChange::DialogInserted(node, connected) => {
                if connected {
                    e.EnsureOpenDialog(node);
                }
            }
            dom::attribute_lifecycle::PopupLifecycleChange::Removed(node) => {
                // HTMLElement/HTMLDialogElement::RemovedFrom suppress events
                // and focus restoration; original rare-data identities survive.
                if e.Open(node) {
                    e.RemoveImmediately(node);
                    e.CommitHidden(node);
                    e.RestoreFocusOnHide(node, HidePopoverFocusBehavior::None);
                }
                if e.Read(|d| {
                    d.FindNodeById(node)
                        .is_some_and(|node| d.Node(node).IsHTMLElement("dialog"))
                }) {
                    e.SetModal(node, false);
                    e.RemoveImmediately(node);
                    e.EraseOpenDialog(node);
                    e.DestroyCloseWatcher(node);
                }
                if let Some(target) = e.Read(|d| {
                    d.PopupLifecycleState()
                        .interest_invokers
                        .get(&node)
                        .and_then(|data| data.active_target)
                }) {
                    e.ChangeInterestState(node, Some(target), InterestState::NoInterest);
                }
            }
        }
        // A source handler's nested setter completes new facts before this
        // outer continuation resumes with its older siblings.
        for nested in queue.TakePopupLifecycleChanges().into_iter().rev() {
            changes.push_front(nested);
        }
    }
    Ok(())
}

/// Document::GetAutofocusDelegate shares ContainerNode's original traversal
/// and the retained native focusability services used by popover/dialog focus.
pub(crate) fn DocumentAutofocusDelegate(dispatcher: &EventDispatcher<'_, '_>) -> Option<u64> {
    let effects = Effects { dispatcher };
    let root = effects.Read(|document| document.RootHandle().Id());
    focus_delegate::GetAutofocusDelegate(&effects, root)
}
