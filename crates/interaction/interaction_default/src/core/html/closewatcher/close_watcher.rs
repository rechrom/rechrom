// Copyright 2021 The Chromium Authors. BSD license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/html/closewatcher/close_watcher.h
// DOM retains watcher identities and the one canonical stack. These ports
// borrow only that stack's fields; no document or engine is passed backward.
use foundation::popup_types::CloseWatcherId;

#[derive(Clone, Copy)]
pub enum AllowCancel {
    kAlways,
    kWithUserActivation,
}

#[derive(Clone, Copy)]
pub struct WatcherState {
    pub closed: bool,
    pub enabled: bool,
    pub dispatching_cancel: bool,
}

pub trait HistoryUserActivation {
    fn IsHistoryUserActivationActive(&self) -> bool;
    fn ConsumeHistoryUserActivation(&self);
}

pub trait CloseWatcherEnvironment: HistoryUserActivation {
    type Error;
    fn ReadGroups<T>(&self, read: impl FnOnce(&[Vec<CloseWatcherId>]) -> T) -> T;
    fn WriteGroups<T>(&self, write: impl FnOnce(&mut Vec<Vec<CloseWatcherId>>) -> T) -> T;
    fn AllowedGroups(&self) -> usize;
    fn SetAllowedGroups(&self, count: usize);
    fn NextUserInteractionCreatesANewAllowedGroup(&self) -> bool;
    fn SetNextUserInteractionCreatesANewAllowedGroup(&self, value: bool);
    fn State(&self, watcher: CloseWatcherId) -> WatcherState;
    fn SetDispatchingCancel(&self, watcher: CloseWatcherId, value: bool);
    fn SetClosedAndClearAbortAlgorithm(&self, watcher: CloseWatcherId);
    fn SetEnabledState(&self, watcher: CloseWatcherId, enabled: bool);
    fn BindCloseListener(&self);
    fn ResetCloseListener(&self);
    fn HasWindow(&self) -> bool;
    fn IsDocumentActive(&self) -> bool;
    /// Invoke the watcher's original native/script listeners synchronously.
    fn DispatchCancel(
        &self,
        watcher: CloseWatcherId,
        cancelable: bool,
    ) -> Result<bool, Self::Error>;
    fn DispatchClose(&self, watcher: CloseWatcherId) -> Result<(), Self::Error>;
}

pub struct WatcherStack;
pub struct CloseWatcher;

/// EventTarget and AbortSignal handle are the source watcher's native members;
/// state and stack membership remain exclusively on its canonical DOM owner.
pub struct NativeCloseWatcher {
    pub watcher: CloseWatcherId,
    pub host_id: u64,
    pub target: std::rc::Rc<interaction_dispatch::core::dom::events::event_target::EventTarget>,
    pub abort_handle:
        std::cell::RefCell<Option<std::rc::Rc<crate::core::dom::abort_signal::AlgorithmHandle>>>,
    pub did_destroy: std::cell::RefCell<Option<std::rc::Rc<dyn Fn()>>>,
}
impl interaction_dispatch::bindings::script_invocation::ScriptNativeObject for NativeCloseWatcher {
    fn AsAny(&self) -> &dyn std::any::Any {
        self
    }
    fn ScriptInterfaceName(&self) -> &'static str {
        "CloseWatcher"
    }
    fn EventTarget(
        &self,
    ) -> Option<std::rc::Rc<interaction_dispatch::core::dom::events::event_target::EventTarget>>
    {
        Some(self.target.clone())
    }
    fn VisitNativeObjects(
        &self,
        visit: &mut dyn FnMut(
            std::rc::Rc<dyn interaction_dispatch::bindings::script_invocation::ScriptNativeObject>,
        ),
    ) {
        if let Some(handle) = self.abort_handle.borrow().clone() {
            visit(handle);
        }
    }
}
