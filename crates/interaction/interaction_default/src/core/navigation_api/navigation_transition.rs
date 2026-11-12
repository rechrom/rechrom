//! Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! navigation_transition.cc:17-64; ScriptPromiseProperty's world-keyed promises
//! share one native settlement state and the original Agent's microtask queue.
use super::{
    navigation_api::{NavigationHistoryEntry, NavigationTransition, NavigationType},
    navigation_destination::NavigationDestination,
};
use crate::bindings::core::v8::{
    script_promise::*, script_promise_resolver::ScriptPromiseResolver,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};
#[derive(Clone)]
enum Settlement {
    Pending,
    Resolved,
    Rejected(Value),
}
struct TransitionPromise {
    state: RefCell<Settlement>,
    worlds: RefCell<HashMap<u64, Rc<ScriptPromiseResolver>>>,
    handled: bool,
}
impl TransitionPromise {
    fn new(handled: bool) -> Self {
        Self {
            state: RefCell::new(Settlement::Pending),
            worlds: RefCell::new(HashMap::new()),
            handled,
        }
    }
    fn Promise(&self, i: &dyn ScriptInvocation) -> Result<ScriptPromise, Value> {
        if let Some(resolver) = self.worlds.borrow().get(&i.WorldIdentity()) {
            return Ok(resolver.Promise());
        }
        let resolver = ScriptPromiseResolver::new(i)?;
        if self.handled {
            i.MarkPromiseHandled(&resolver.Promise());
        }
        self.worlds
            .borrow_mut()
            .insert(i.WorldIdentity(), resolver.clone());
        Self::SettleResolver(i, &resolver, &self.state.borrow());
        Ok(resolver.Promise())
    }
    fn SettleResolver(
        i: &dyn ScriptInvocation,
        resolver: &ScriptPromiseResolver,
        state: &Settlement,
    ) {
        match state {
            Settlement::Pending => {}
            Settlement::Resolved => resolver.Resolve(i, i.Undefined()),
            Settlement::Rejected(error) => resolver.Reject(i, error.clone()),
        }
    }
    fn Settle(&self, i: &dyn ScriptInvocation, state: Settlement) {
        if !matches!(*self.state.borrow(), Settlement::Pending) {
            return;
        }
        *self.state.borrow_mut() = state.clone();
        let worlds = self
            .worlds
            .borrow()
            .iter()
            .map(|(world, resolver)| (*world, resolver.clone()))
            .collect::<Vec<_>>();
        for (world, resolver) in worlds {
            i.WithWorld(world, &mut |i| Self::SettleResolver(i, &resolver, &state));
        }
    }
    fn Visit(&self, visit: &mut dyn FnMut(Value)) {
        if let Settlement::Rejected(error) = &*self.state.borrow() {
            visit(error.clone());
        }
        for resolver in self.worlds.borrow().values() {
            resolver.VisitScriptValues(visit);
        }
    }
}
pub struct NavigationTransitionOwner {
    navigation_type: NavigationType,
    from: Option<Rc<dyn NavigationHistoryEntry>>,
    destination: Rc<NavigationDestination>,
    committed: TransitionPromise,
    finished: TransitionPromise,
}
impl NavigationTransitionOwner {
    pub fn new(
        navigation_type: NavigationType,
        from: Option<Rc<dyn NavigationHistoryEntry>>,
        destination: Rc<NavigationDestination>,
    ) -> Rc<Self> {
        Rc::new(Self {
            navigation_type,
            from,
            destination,
            committed: TransitionPromise::new(false),
            finished: TransitionPromise::new(true),
        })
    }
}
impl NavigationTransition for NavigationTransitionOwner {
    fn NavigationType(&self) -> NavigationType {
        self.navigation_type
    }
    fn From(&self) -> Option<Rc<dyn NavigationHistoryEntry>> {
        self.from.clone()
    }
    fn Destination(&self) -> Rc<NavigationDestination> {
        self.destination.clone()
    }
    fn Committed(&self, i: &dyn ScriptInvocation) -> Result<ScriptPromise, Value> {
        self.committed.Promise(i)
    }
    fn Finished(&self, i: &dyn ScriptInvocation) -> Result<ScriptPromise, Value> {
        self.finished.Promise(i)
    }
    fn ResolveCommittedPromise(&self, i: &dyn ScriptInvocation) {
        self.committed.Settle(i, Settlement::Resolved);
    }
    fn ResolveFinishedPromise(&self, i: &dyn ScriptInvocation) {
        self.finished.Settle(i, Settlement::Resolved);
    }
    fn RejectFinishedPromise(&self, i: &dyn ScriptInvocation, error: Value) {
        self.committed
            .Settle(i, Settlement::Rejected(error.clone()));
        self.finished.Settle(i, Settlement::Rejected(error));
    }
}
impl ScriptNativeObject for NavigationTransitionOwner {
    fn AsAny(&self) -> &dyn std::any::Any {
        self
    }
    fn ScriptInterfaceName(&self) -> &'static str {
        "NavigationTransition"
    }
    fn VisitScriptValues(&self, visit: &mut dyn FnMut(Value)) {
        self.committed.Visit(visit);
        self.finished.Visit(visit);
    }
    fn VisitNativeObjects(&self, visit: &mut dyn FnMut(Rc<dyn ScriptNativeObject>)) {
        if let Some(from) = &self.from {
            visit(from.clone());
        }
        visit(self.destination.clone());
    }
}
