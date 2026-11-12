// Copyright The Chromium Authors. BSD license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: element.cc:8339-8403; container_node.cc:1873-1901;
// html_dialog_element.cc:58-111,672-705.
// Traversal operations borrow the existing DOM/flat-tree owner one node at a
// time. These routines never retain a copied subtree or computed style.
pub trait FocusDelegateEnvironment {
    fn ShadowRoot(&self, node: u64) -> Option<u64>;
    fn ShadowRootIsUserAgent(&self, root: u64) -> bool;
    fn DelegatesFocus(&self, node: u64) -> bool;
    fn FocusedElement(&self) -> Option<u64>;
    fn ShadowIncludingInclusiveAncestor(&self, ancestor: u64, node: u64) -> bool;
    fn NextElement(&self, root: u64, after: Option<u64>) -> Option<u64>;
    fn FirstFlatChild(&self, node: u64) -> Option<u64>;
    fn NextFlatNode(&self, node: u64, root: u64, skip_children: bool) -> Option<u64>;
    fn IsElement(&self, node: u64) -> bool;
    fn IsDialog(&self, node: u64) -> bool;
    fn IsAutofocusable(&self, node: u64) -> bool;
    fn IsFocusable(&self, node: u64) -> bool;
    fn AdjustedTabIndex(&self, node: u64) -> i32;
    fn Connected(&self, node: u64) -> bool;
    fn SetPreviousFocus(&self, dialog: u64, focused: Option<u64>);
    fn Focus(&self, node: u64);
    fn ClearFocusedElement(&self);
    fn IsModal(&self, node: u64) -> bool;
    fn FinalizeAutofocus(&self, node: u64);
}

// Source: element.cc:8345-8370.
pub fn GetFocusableArea<E: FocusDelegateEnvironment + ?Sized>(
    e: &E,
    node: u64,
    in_descendant_traversal: bool,
) -> Option<u64> {
    if !e.DelegatesFocus(node) {
        return None;
    }
    if let Some(focused) = e
        .FocusedElement()
        .filter(|&focused| e.ShadowIncludingInclusiveAncestor(node, focused))
    {
        return Some(focused);
    }
    GetFocusDelegate(e, node, in_descendant_traversal)
}

// Source: container_node.cc:1873-1901.
pub fn GetAutofocusDelegate<E: FocusDelegateEnvironment + ?Sized>(e: &E, root: u64) -> Option<u64> {
    let mut element = e.NextElement(root, None);
    while let Some(node) = element {
        if e.IsAutofocusable(node) {
            let area = if e.IsFocusable(node) {
                Some(node)
            } else {
                GetFocusableArea(e, node, false)
            };
            if area.is_some() {
                return area;
            }
        }
        element = e.NextElement(root, Some(node));
    }
    None
}

// Source: element.cc:8372-8403. The parameter participates in the source
// virtual call chain; the current source body forwards it only recursively.
pub fn GetFocusDelegate<E: FocusDelegateEnvironment + ?Sized>(
    e: &E,
    node: u64,
    _in_descendant_traversal: bool,
) -> Option<u64> {
    let shadow = e.ShadowRoot(node);
    if shadow.is_some_and(|root| !e.ShadowRootIsUserAgent(root) && !e.DelegatesFocus(node)) {
        return None;
    }
    let root = if e.DelegatesFocus(node) {
        shadow.expect("delegating host retains ShadowRoot")
    } else {
        node
    };
    if let Some(delegate) = GetAutofocusDelegate(e, root) {
        return Some(delegate);
    }
    let mut descendant = e.NextElement(root, None);
    while let Some(current) = descendant {
        if e.IsFocusable(current) && (!e.IsDialog(node) || e.AdjustedTabIndex(current) >= 0) {
            return Some(current);
        }
        if let Some(area) = GetFocusableArea(e, current, true) {
            return Some(area);
        }
        descendant = e.NextElement(root, Some(current));
    }
    None
}

// Source: html_dialog_element.cc:58-111. FlatTreeTraversal skips the children
// of nested dialogs, but still considers the nested dialog element itself.
pub fn SetFocusForDialogLegacy<E: FocusDelegateEnvironment + ?Sized>(e: &E, dialog: u64) {
    if !e.Connected(dialog) {
        return;
    }
    e.SetPreviousFocus(dialog, e.FocusedElement());
    let mut control = None;
    let mut node = e.FirstFlatChild(dialog);
    while let Some(current) = node {
        node = e.NextFlatNode(current, dialog, e.IsDialog(current));
        if !e.IsElement(current) {
            continue;
        }
        if e.IsAutofocusable(current) && e.IsFocusable(current) {
            control = Some(current);
            break;
        }
        if control.is_none() && e.IsFocusable(current) {
            control = Some(current);
        }
    }
    let control = control.unwrap_or(dialog);
    if e.IsFocusable(control) {
        e.Focus(control);
    } else {
        e.ClearFocusedElement();
    }
    e.FinalizeAutofocus(control);
}

// Source: html_dialog_element.cc:672-705.
pub fn SetFocusForDialog<E: FocusDelegateEnvironment + ?Sized>(e: &E, dialog: u64) {
    e.SetPreviousFocus(dialog, e.FocusedElement());
    let mut control = GetFocusDelegate(e, dialog, false);
    if e.IsAutofocusable(dialog) {
        control = Some(dialog);
    }
    let control = control.unwrap_or(dialog);
    if e.IsFocusable(control) {
        e.Focus(control);
    } else if e.IsModal(dialog) {
        e.ClearFocusedElement();
    }
    e.FinalizeAutofocus(control);
}
