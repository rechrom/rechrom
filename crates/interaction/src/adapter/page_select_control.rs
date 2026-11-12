//! Native select effects outside the DOM/default-action owner. The embedding
//! lends one current Document's original layout and host services per call.
use foundation::Persistent;
use layoutng_style::style::computed_style::ComputedStyle;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectAccessibilityNotification {
    ControlValueChanged { control: u64 },
    ListboxActiveIndexChanged { select: u64 },
    TextChanged { select: u64 },
    ActiveMenuOptionChanged { select: u64, previous: Option<u64> },
}
/// ExistingAXObjectCache is genuinely optional in Chromium. An absent client
/// is represented at the source gate, rather than a successful notification.
pub trait SelectAccessibilityClient {
    fn Notify(&self, document: u128, notification: SelectAccessibilityNotification);
}
pub trait NativeSelectPopupClient {
    fn UpdateFromElement(&self, document: u128, select: u64);
}
pub trait FormToolClient {
    fn CancelActiveScriptTool(&self, document: u128, form: u64);
}
pub trait SelectControlHost: dom::attribute_lifecycle::SelectControlLifecycle {
    fn PostUserInteractionTask(&self, task: Box<dyn FnOnce()>);
    fn ScrollOptionIntoView(&self, select: u64, option: u64);
    fn HasLayoutObject(&self, node: u64) -> bool;
    fn HasLayoutBox(&self, node: u64) -> bool;
    fn SetInnerModifiedStyleOutsideStyleRecalc(&self, inner: u64, style: Persistent<ComputedStyle>);
    fn AccessibilityClient(&self) -> Option<Rc<dyn SelectAccessibilityClient>>;
    fn NativePopupClient(&self) -> Option<Rc<dyn NativeSelectPopupClient>>;
    fn FormToolClient(&self) -> Option<Rc<dyn FormToolClient>>;
    fn FormElementReset(&self, form: u64);
    fn DidChangeSelectionInSelectControl(&self, select: u64);
    fn NotifyFormStateChanged(&self, select: u64);
    fn MultipleOptionsSelectedText(&self, count: usize) -> Rc<[u16]>;
}
