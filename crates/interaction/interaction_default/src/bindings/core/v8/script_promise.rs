// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: bindings/core/v8/script_promise{,_resolver,_property}.{h,cc}.
// VM-neutral stack invocation belongs to the shared bindings root; navigation
// adds only its native class projection, never a stored runtime service.
use crate::core::navigation_api::navigation_api::NavigationHistoryEntry;
pub use interaction_dispatch::bindings::script_invocation::*;
use std::rc::Rc;
pub trait NavigationScriptInvocation: ScriptInvocation {
    fn HistoryEntryValue(&self, entry: Option<Rc<dyn NavigationHistoryEntry>>) -> Value {
        entry.map_or_else(|| self.Null(), |entry| self.NativeObjectValue(entry))
    }
}
impl<T: ScriptInvocation + ?Sized> NavigationScriptInvocation for T {}
