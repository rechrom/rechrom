// Copyright 2021 The Chromium Authors. BSD license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/html/closewatcher/close_watcher.cc
use super::close_watcher::*;
use foundation::popup_types::CloseWatcherId;
use interaction_dispatch::core::dom::events::event::EventInterface;
use interaction_dispatch::core::events::keyboard_event::KeyboardEventRef;

impl WatcherStack {
    // Source: close_watcher.cc:46-55.
    pub fn AnyEnabledWatchers(e: &impl CloseWatcherEnvironment) -> bool {
        e.ReadGroups(|groups| {
            groups
                .iter()
                .flatten()
                .any(|&watcher| e.State(watcher).enabled)
        })
    }
    // Source: close_watcher.cc:57-67. The embedding's typed close-signal
    // connection replaces Mojo transport; registration lifetime is identical.
    pub fn MaybeCloseReceiver(e: &impl CloseWatcherEnvironment) {
        if !Self::AnyEnabledWatchers(e) {
            e.ResetCloseListener();
        }
    }
    pub fn BindNewPipe(e: &impl CloseWatcherEnvironment) {
        e.BindCloseListener();
    }
    // Source: close_watcher.cc:82-99 (group insertion).
    pub fn Add(e: &impl CloseWatcherEnvironment, watcher: CloseWatcherId) {
        if !Self::AnyEnabledWatchers(e) && e.State(watcher).enabled {
            Self::BindNewPipe(e);
        }
        let allowed = e.AllowedGroups();
        e.WriteGroups(|groups| {
            if groups.len() < allowed {
                groups.push(vec![watcher]);
            } else {
                groups
                    .last_mut()
                    .expect("allowed_groups is at least one")
                    .push(watcher);
            }
        });
        e.SetNextUserInteractionCreatesANewAllowedGroup(true);
    }
    // Source: close_watcher.cc:101-116 (group removal).
    pub fn Remove(e: &impl CloseWatcherEnvironment, watcher: CloseWatcherId) {
        e.WriteGroups(|groups| {
            for index in 0..groups.len() {
                if let Some(position) = groups[index].iter().position(|&id| id == watcher) {
                    groups[index].remove(position);
                    if groups[index].is_empty() {
                        groups.remove(index);
                    }
                    break;
                }
            }
        });
        Self::MaybeCloseReceiver(e);
    }
    // Source: close_watcher.cc:118-151.
    pub fn SetHadUserInteraction(e: &impl CloseWatcherEnvironment, had_user_interaction: bool) {
        if had_user_interaction {
            if e.NextUserInteractionCreatesANewAllowedGroup() {
                e.SetAllowedGroups(e.AllowedGroups() + 1);
            }
            e.SetNextUserInteractionCreatesANewAllowedGroup(false);
        } else {
            e.SetAllowedGroups(1);
            e.SetNextUserInteractionCreatesANewAllowedGroup(true);
        }
    }
    // Source: close_watcher.cc:153-156.
    pub fn CancelEventCanBeCancelable(e: &impl CloseWatcherEnvironment) -> bool {
        e.ReadGroups(|groups| groups.len()) < e.AllowedGroups() && e.IsHistoryUserActivationActive()
    }
    // Source: close_watcher.cc:158-163. Do not mark Escape default-handled.
    pub fn EscapeKeyHandler<E: CloseWatcherEnvironment>(
        e: &E,
        event: &KeyboardEventRef,
    ) -> Result<(), E::Error> {
        let signal = {
            let event = event.borrow();
            Self::AnyEnabledWatchers(e)
                && !event.BaseEvent().DefaultHandled()
                && event.BaseEvent().isTrusted()
                && event.keyCode() == 0x1b
        };
        if signal {
            Self::Signal(e)?;
        }
        Ok(())
    }
    // Source: close_watcher.cc:165-179. The last group's identity snapshot is
    // intentional: cancel/close listeners can synchronously mutate the stack.
    pub fn Signal<E: CloseWatcherEnvironment>(e: &E) -> Result<(), E::Error> {
        debug_assert!(Self::AnyEnabledWatchers(e));
        let group = e.ReadGroups(|groups| groups.last().expect("enabled watcher group").clone());
        let result = (|| {
            for watcher in group.into_iter().rev() {
                if !CloseWatcher::RequestClose(e, watcher, AllowCancel::kWithUserActivation)? {
                    break;
                }
            }
            Ok(())
        })();
        if e.AllowedGroups() > 1 {
            e.SetAllowedGroups(e.AllowedGroups() - 1);
        }
        result
    }
}

struct DispatchingCancel<'a, E: CloseWatcherEnvironment>(&'a E, CloseWatcherId);
impl<E: CloseWatcherEnvironment> Drop for DispatchingCancel<'_, E> {
    fn drop(&mut self) {
        self.0.SetDispatchingCancel(self.1, false);
    }
}
impl CloseWatcher {
    // Source: close_watcher.cc:69-80.
    pub fn setEnabled(e: &impl CloseWatcherEnvironment, watcher: CloseWatcherId, enabled: bool) {
        if e.State(watcher).enabled == enabled {
            return;
        }
        if enabled && !WatcherStack::AnyEnabledWatchers(e) {
            WatcherStack::BindNewPipe(e);
        }
        e.SetEnabledState(watcher, enabled);
        if !enabled {
            WatcherStack::MaybeCloseReceiver(e);
        }
    }
    // Source: close_watcher.cc:242-271.
    pub fn RequestClose<E: CloseWatcherEnvironment>(
        e: &E,
        watcher: CloseWatcherId,
        allow_cancel: AllowCancel,
    ) -> Result<bool, E::Error> {
        let state = e.State(watcher);
        if state.closed || state.dispatching_cancel || !e.HasWindow() || !state.enabled {
            return Ok(true);
        }
        let cancelable = matches!(allow_cancel, AllowCancel::kAlways)
            || WatcherStack::CancelEventCanBeCancelable(e);
        let prevented = {
            e.SetDispatchingCancel(watcher, true);
            let _reset = DispatchingCancel(e, watcher);
            e.DispatchCancel(watcher, cancelable)?
        };
        if prevented {
            if e.HasWindow() {
                e.ConsumeHistoryUserActivation();
            }
            return Ok(false);
        }
        Self::close(e, watcher)?;
        Ok(true)
    }
    // Source: close_watcher.cc:273-284.
    pub fn close<E: CloseWatcherEnvironment>(
        e: &E,
        watcher: CloseWatcherId,
    ) -> Result<(), E::Error> {
        let state = e.State(watcher);
        if state.closed || !e.HasWindow() || !e.IsDocumentActive() || !state.enabled {
            return Ok(());
        }
        Self::destroy(e, watcher);
        e.DispatchClose(watcher)
    }
    // Source: close_watcher.cc:286-295.
    pub fn destroy(e: &impl CloseWatcherEnvironment, watcher: CloseWatcherId) {
        if e.State(watcher).closed {
            return;
        }
        if e.HasWindow() {
            WatcherStack::Remove(e, watcher);
        }
        e.SetClosedAndClearAbortAlgorithm(watcher);
    }
}
