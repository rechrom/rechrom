use crate::core::page::focus_controller::FocusDispatchScope;
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/dom/document.cc:5896-6157
// The document retains focus; this transaction holds no borrow over callbacks.
use super::{element::SelectionBehaviorOnFocus, focus_params::FocusParams};
use crate::core::page::focus_controller::FocusElementRef;

/// Identity and the two Element operations used by the document transaction.
/// An embedding may retain an opaque DOM identity instead of exposing Blink's
/// frame/element object graph. Both forms execute the same source transaction.
pub trait DocumentFocusElement: Clone {
    fn Same(&self, other: &Self) -> bool;
    fn IsRootEditableElement(&self) -> bool;
    fn SetFocused(
        &self,
        focused: bool,
        focus_type: crate::public::mojom::input::focus_type::FocusType,
    );
}
impl DocumentFocusElement for FocusElementRef {
    fn Same(&self, other: &Self) -> bool {
        std::rc::Rc::ptr_eq(self, other)
    }
    fn IsRootEditableElement(&self) -> bool {
        self.as_ref().IsRootEditableElement()
    }
    fn SetFocused(
        &self,
        focused: bool,
        focus_type: crate::public::mojom::input::focus_type::FocusType,
    ) {
        self.as_ref().SetFocused(focused, focus_type);
    }
}

pub trait DocumentFocusOwner<E: DocumentFocusElement = FocusElementRef> {
    fn StopClearFocusedElementTimer(&self);
    fn FocusedElement(&self) -> Option<E>;
    fn BelongsToDocument(&self, element: &E) -> bool;
    fn IsBeingRemoved(&self, element: &E) -> bool;
    fn StoreFocusedElement(&self, element: Option<E>, params: Option<&FocusParams>);
    fn CommonAncestor(&self, old: &E, new: &E) -> Option<E>;
    fn SetFocusWithin(
        &self,
        element: &E,
        focused: bool,
        ancestor: Option<&E>,
        search_snap_container: bool,
    );
    fn ElementLostFocus(&self, element: &E);
    fn ElementGainedFocus(&self, element: &E);
    fn UpdateFocusgroupLastFocused(&self, element: &E);
    fn IsPageFocused(&self) -> bool;
    fn DispatchFocusEvent(
        &self,
        element: &E,
        kind: &'static str,
        related: Option<&E>,
        params: &FocusParams,
        scope: &FocusDispatchScope<'_>,
    );
    fn BlurEditContext(&self, element: &E);
    fn FocusEditContext(&self, element: &E);
    fn UpdateStyleAndLayoutTreeForElement(&self, element: &E);
    fn IsFocusable(&self, element: &E) -> bool;
    fn AcceptsEditingFocus(&self, element: &E) -> bool;
    /// Set the sequential navigation starting point, update the last user
    /// focus type, and notify focused-element observers, in that order.
    fn DidStoreFocusedElement(&self, element: &E, params: &FocusParams);
    /// Clear selection-after-layout, update style/layout for this node, and
    /// invoke its UpdateSelectionOnFocus with the original options/behavior.
    fn UpdateSelectionOnFocus(&self, element: &E, params: &FocusParams);
    fn NotifyFocusedElementChanged(&self, old: Option<&E>, new: Option<&E>, params: &FocusParams);
    fn UpdateStyleAndLayoutTree(&self);
    fn SelectionDidChangeFocus(&self);
}
fn Same<E: DocumentFocusElement>(a: Option<&E>, b: Option<&E>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => a.Same(b),
        (None, None) => true,
        _ => false,
    }
}
fn Finish<E: DocumentFocusElement>(owner: &dyn DocumentFocusOwner<E>) {
    owner.UpdateStyleAndLayoutTree();
    owner.SelectionDidChangeFocus();
}
fn ClearAncestor<E: DocumentFocusElement>(owner: &dyn DocumentFocusOwner<E>, ancestor: Option<&E>) {
    if let (Some(ancestor), Some(focused)) = (ancestor, owner.FocusedElement()) {
        let next = owner.CommonAncestor(ancestor, &focused);
        if !Same(Some(ancestor), next.as_ref()) {
            owner.SetFocusWithin(ancestor, false, next.as_ref(), false);
        }
    }
}
pub fn SetFocusedElement<E: DocumentFocusElement>(
    owner: &dyn DocumentFocusOwner<E>,
    new: Option<E>,
    params: &FocusParams,
) -> bool {
    SetFocusedElementWithScope(owner, new, params, &FocusDispatchScope::default())
}

pub fn SetFocusedElementWithScope<E: DocumentFocusElement>(
    owner: &dyn DocumentFocusOwner<E>,
    mut new: Option<E>,
    params: &FocusParams,
    scope: &FocusDispatchScope<'_>,
) -> bool {
    owner.StopClearFocusedElementTimer();
    if new
        .as_ref()
        .is_some_and(|e| !owner.BelongsToDocument(e) || owner.IsBeingRemoved(e))
    {
        return true;
    }
    let old = owner.FocusedElement();
    if Same(old.as_ref(), new.as_ref()) {
        owner.StoreFocusedElement(new, Some(params));
        return true;
    }
    owner.StoreFocusedElement(None, None);
    let ancestor = old
        .as_ref()
        .zip(new.as_ref())
        .and_then(|(a, b)| owner.CommonAncestor(a, b));
    let mut blocked = false;
    if let Some(old) = &old {
        old.SetFocused(false, params.r#type);
        owner.SetFocusWithin(old, false, ancestor.as_ref(), true);
        owner.ElementLostFocus(old);
        if !params.omit_blur_events && owner.IsPageFocused() {
            owner.DispatchFocusEvent(old, "blur", new.as_ref(), params, scope);
            if owner.FocusedElement().is_some() {
                blocked = true;
                new = None;
                ClearAncestor(owner, ancestor.as_ref());
            }
            owner.DispatchFocusEvent(old, "focusout", new.as_ref(), params, scope);
            owner.DispatchFocusEvent(old, "DOMFocusOut", new.as_ref(), params, scope);
            if owner.FocusedElement().is_some() {
                blocked = true;
                new = None;
                ClearAncestor(owner, ancestor.as_ref());
            }
        }
        owner.BlurEditContext(old);
    }
    if new.as_ref().is_some_and(|e| !owner.BelongsToDocument(e)) {
        return true;
    }
    if let Some(new) = &new {
        owner.UpdateStyleAndLayoutTreeForElement(new);
    }
    if let Some(new) = new.as_ref().filter(|e| owner.IsFocusable(e)) {
        if new.IsRootEditableElement() && !owner.AcceptsEditingFocus(new) {
            Finish(owner);
            return false;
        }
        owner.StoreFocusedElement(Some(new.clone()), Some(params));
        owner.DidStoreFocusedElement(new, params);
        new.SetFocused(true, params.r#type);
        let Some(current) = owner.FocusedElement() else {
            return false;
        };
        owner.SetFocusWithin(&current, true, ancestor.as_ref(), true);
        owner.ElementGainedFocus(&current);
        owner.UpdateFocusgroupLastFocused(&current);
        if !Same(owner.FocusedElement().as_ref(), Some(new)) {
            Finish(owner);
            return false;
        }
        owner.UpdateSelectionOnFocus(new, params);
        if owner.IsPageFocused() {
            for kind in ["focus", "focusin", "DOMFocusIn"] {
                owner.DispatchFocusEvent(new, kind, old.as_ref(), params, scope);
                if !Same(owner.FocusedElement().as_ref(), Some(new)) {
                    Finish(owner);
                    return false;
                }
            }
        }
    }
    if !blocked {
        owner.NotifyFocusedElementChanged(old.as_ref(), owner.FocusedElement().as_ref(), params);
    }
    Finish(owner);
    if let Some(new) = &new {
        owner.FocusEditContext(new);
    }
    !blocked
}
pub fn ClearFocusedElement<E: DocumentFocusElement>(owner: &dyn DocumentFocusOwner<E>) {
    ClearFocusedElementWithScope(owner, &FocusDispatchScope::default());
}
pub fn ClearFocusedElementWithScope<E: DocumentFocusElement>(
    owner: &dyn DocumentFocusOwner<E>,
    scope: &FocusDispatchScope<'_>,
) {
    SetFocusedElementWithScope(
        owner,
        None,
        &FocusParams {
            selection_behavior: SelectionBehaviorOnFocus::kNone,
            ..FocusParams::default()
        },
        scope,
    );
}
