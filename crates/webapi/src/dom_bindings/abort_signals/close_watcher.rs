//! Source: core/html/closewatcher/close_watcher.cc:28-40,187-240.
//! Web IDL and tracing projection only; Interaction owns every state transition.
use super::*;
use interaction::default::core::{
    dom::abort_signal::{AbortAlgorithm, AbortSignalInvocation},
    html::closewatcher::close_watcher::NativeCloseWatcher,
};

struct DestroyOnAbortAlgorithm {
    watcher: Rc<NativeCloseWatcher>,
    host: services::CloseWatcherHost,
}
impl AbortAlgorithm for DestroyOnAbortAlgorithm {
    fn Run(&self, _: &dyn AbortSignalInvocation) {
        (self.host.destroy)(self.watcher.watcher);
    }
    fn VisitNativeObjects(&self, visit: &mut dyn FnMut(Rc<dyn ScriptNativeObject>)) {
        visit(self.watcher.clone());
    }
}
impl AbortBindings {
    pub(in crate::dom_bindings) fn BindCloseWatcherHost(&self, host: services::CloseWatcherHost) {
        *self.close_watcher_host.borrow_mut() = Some(host);
    }
    pub(super) fn InvokeCloseWatcher(
        &self,
        receiver: u64,
        member: &str,
        args: &[HostValue],
        runtime: &mut dyn JavaScriptHostRuntime,
    ) -> HostResult {
        let Some(host) = self.close_watcher_host.borrow().clone() else {
            return HostResult {
                thrown: Some((self.exception)(
                    "InvalidStateError",
                    "CloseWatchers cannot be created in detached Windows",
                )),
                ..Default::default()
            };
        };
        if member == "__closeWatcherCreate" && receiver == 0 {
            if self.destroyed.get() {
                return HostResult {
                    thrown: Some((self.exception)(
                        "InvalidStateError",
                        "CloseWatchers cannot be created in detached Windows",
                    )),
                    ..Default::default()
                };
            }
            let signal = match args.first() {
                None | Some(HostValue::Undefined(_)) => None,
                Some(value) => match self.SignalFromValue(value) {
                    Some(signal) => Some(signal),
                    None => {
                        return DOMJavaScriptBindings::type_error("signal is not an AbortSignal")
                    }
                },
            };
            let aborted = signal.as_ref().is_some_and(|signal| signal.Aborted());
            let id = self.Allocate();
            let target =
                event_listeners::CreateGenericTarget(self.document.clone(), self.graph.clone(), id);
            let target = target
                .EventTarget()
                .expect("native CloseWatcher EventTarget");
            let watcher = match (host.create)(id, target, aborted) {
                Ok(watcher) => watcher,
                Err(message) => {
                    return HostResult {
                        thrown: Some((self.exception)("InvalidStateError", &message)),
                        ..Default::default()
                    }
                }
            };
            self.StoreAt(id, watcher.clone());
            if !aborted {
                if let Some(signal) = signal {
                    let algorithm = Rc::new(DestroyOnAbortAlgorithm {
                        watcher: watcher.clone(),
                        host: host.clone(),
                    });
                    *watcher.abort_handle.borrow_mut() = signal.AddScopedAlgorithm(algorithm);
                }
                self.Publish([C::Root {
                    id,
                    reason: 13,
                    rooted: true,
                }]);
                let graph = self.graph.clone();
                let bindings = self.self_.clone();
                let native = Rc::downgrade(&watcher);
                *watcher.did_destroy.borrow_mut() = Some(Rc::new(move || {
                    if let Some(graph) = graph.upgrade() {
                        graph.Publish([C::Root {
                            id,
                            reason: 13,
                            rooted: false,
                        }]);
                    }
                    if let (Some(bindings), Some(native)) = (bindings.upgrade(), native.upgrade()) {
                        bindings.Trace(native);
                    }
                }));
            }
            self.Trace(watcher);
            return HostResult {
                value: HostValue::Object(HostObjectRef { id }),
                ..Default::default()
            };
        }
        let Some(watcher) =
            self.NativeFromValue::<NativeCloseWatcher>(&HostValue::Object(HostObjectRef {
                id: receiver,
            }))
        else {
            return DOMJavaScriptBindings::type_error("Illegal CloseWatcher invocation");
        };
        let result = match member {
            "__closeWatcherDestroy" => {
                (host.destroy)(watcher.watcher);
                Ok(())
            }
            "__closeWatcherRequestClose" => (host.invoke)(watcher.watcher, true, runtime),
            "__closeWatcherClose" => (host.invoke)(watcher.watcher, false, runtime),
            _ => return DOMJavaScriptBindings::type_error("Illegal CloseWatcher invocation"),
        };
        self.Trace(watcher);
        match result {
            Ok(()) => HostResult::default(),
            Err(message) => HostResult {
                thrown: Some((self.exception)("InvalidStateError", &message)),
                ..Default::default()
            },
        }
    }
}
