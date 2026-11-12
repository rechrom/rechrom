//! Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! navigation_activation.h:20-51. The same retained owner is projected to JS.
use super::navigation_api::{NavigationHistoryEntry, NavigationType};
use crate::bindings::core::v8::script_promise::ScriptNativeObject;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
pub struct NavigationActivation {
    entry: RefCell<Option<Rc<dyn NavigationHistoryEntry>>>,
    from: RefCell<Option<Rc<dyn NavigationHistoryEntry>>>,
    navigation_type: Cell<NavigationType>,
}
impl NavigationActivation {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            entry: Default::default(),
            from: Default::default(),
            navigation_type: Cell::new(NavigationType::Push),
        })
    }
    pub fn Update(
        &self,
        entry: Option<Rc<dyn NavigationHistoryEntry>>,
        from: Option<Rc<dyn NavigationHistoryEntry>>,
        navigation_type: NavigationType,
    ) {
        *self.entry.borrow_mut() = entry;
        *self.from.borrow_mut() = from;
        self.navigation_type.set(navigation_type);
    }
    pub fn Entry(&self) -> Option<Rc<dyn NavigationHistoryEntry>> {
        self.entry.borrow().clone()
    }
    pub fn From(&self) -> Option<Rc<dyn NavigationHistoryEntry>> {
        self.from.borrow().clone()
    }
    pub fn NavigationType(&self) -> NavigationType {
        self.navigation_type.get()
    }
}
impl ScriptNativeObject for NavigationActivation {
    fn AsAny(&self) -> &dyn std::any::Any {
        self
    }
    fn ScriptInterfaceName(&self) -> &'static str {
        "NavigationActivation"
    }
    fn VisitNativeObjects(&self, visit: &mut dyn FnMut(Rc<dyn ScriptNativeObject>)) {
        if let Some(entry) = self.Entry() {
            visit(entry);
        }
        if let Some(from) = self.From() {
            visit(from);
        }
    }
}
