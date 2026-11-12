// Copyright The Chromium Authors. BSD license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: html_element.cc:3101-3189; html_dialog_element.cc:274-375.
// The document retains the actual stacks and pointerdown identities. Every
// lifecycle call releases the query borrow before invoking script.
use foundation::popup_types::{
    ClosedByState, HidePopoverFocusBehavior, HidePopoverTransitionBehavior,
};

pub trait LightDismissEnvironment {
    type Error;
    fn TopmostPopoverOrHint(&self) -> Option<u64>;
    fn NearestOpenPopover(&self, node: u64) -> Option<u64>;
    fn NearestTargetPopoverForInvoker(&self, node: u64) -> Option<u64>;
    fn PopoverStackPosition(&self, popover: Option<u64>) -> usize;
    fn PopoverPointerdownTarget(&self) -> Option<u64>;
    fn SetPopoverPointerdownTarget(&self, target: Option<u64>);
    fn HideAllPopoversUntil(
        &self,
        endpoint: Option<u64>,
        focus: HidePopoverFocusBehavior,
        transition: HidePopoverTransitionBehavior,
    ) -> Result<(), Self::Error>;
    fn TopmostOpenDialog(&self) -> Option<u64>;
    fn FlatTreeParent(&self, node: u64) -> Option<u64>;
    fn BackdropDialog(&self, node: u64) -> Option<u64>;
    fn IsOpenAndActiveDialog(&self, node: u64) -> bool;
    fn DialogIsModal(&self, node: u64) -> bool;
    fn DialogRectContains(
        &self,
        node: u64,
        client_x: f64,
        client_y: f64,
    ) -> Result<bool, Self::Error>;
    fn DialogPointerdownTarget(&self) -> Option<u64>;
    fn SetDialogPointerdownTarget(&self, target: Option<u64>);
    fn DialogClosedBy(&self, node: u64) -> ClosedByState;
    fn RequestClose(&self, node: u64) -> Result<(), Self::Error>;
}

// Source: html_element.cc:3101-3132.
pub fn FindTopmostRelatedPopover<E: LightDismissEnvironment + ?Sized>(
    e: &E,
    node: u64,
) -> Option<u64> {
    let direct = e.NearestOpenPopover(node);
    let invoker = e.NearestTargetPopoverForInvoker(node);
    if invoker.is_none() || e.PopoverStackPosition(direct) > e.PopoverStackPosition(invoker) {
        direct
    } else {
        invoker
    }
}

// Source: html_element.cc:3136-3174. Called before EventPath construction.
pub fn HandlePopoverLightDismiss<E: LightDismissEnvironment + ?Sized>(
    e: &E,
    event_type: &str,
    target: u64,
) -> Result<(), E::Error> {
    if e.TopmostPopoverOrHint().is_none() {
        return Ok(());
    }
    if event_type == "pointerdown" {
        e.SetPopoverPointerdownTarget(FindTopmostRelatedPopover(e, target));
    } else if event_type == "pointerup" {
        let ancestor = FindTopmostRelatedPopover(e, target);
        let same = ancestor == e.PopoverPointerdownTarget();
        e.SetPopoverPointerdownTarget(None);
        if same {
            e.HideAllPopoversUntil(
                ancestor,
                HidePopoverFocusBehavior::None,
                HidePopoverTransitionBehavior::FireEventsAndWaitForTransitions,
            )?;
        }
    }
    Ok(())
}

// Source: html_element.cc:3177-3189.
pub fn HandlePopoverLightDismissForClick<E: LightDismissEnvironment + ?Sized>(
    e: &E,
    down: u64,
    up: u64,
) -> Result<(), E::Error> {
    let down = FindTopmostRelatedPopover(e, down);
    let up = FindTopmostRelatedPopover(e, up);
    if down == up {
        e.HideAllPopoversUntil(
            up,
            HidePopoverFocusBehavior::None,
            HidePopoverTransitionBehavior::FireEventsAndWaitForTransitions,
        )?;
    }
    Ok(())
}

// Source: html_dialog_element.cc:274-301.
pub fn FindNearestDialog<E: LightDismissEnvironment + ?Sized>(
    e: &E,
    target: u64,
    x: f64,
    y: f64,
) -> Result<Option<u64>, E::Error> {
    let dialog = if e.IsOpenAndActiveDialog(target) {
        Some(target)
    } else {
        e.BackdropDialog(target)
    };
    if let Some(dialog) = dialog {
        if e.IsOpenAndActiveDialog(dialog)
            && e.DialogIsModal(dialog)
            && !e.DialogRectContains(dialog, x, y)?
        {
            return Ok(None);
        }
    }
    let mut node = Some(target);
    while let Some(current) = node {
        if e.IsOpenAndActiveDialog(current) {
            return Ok(Some(current));
        }
        node = e.FlatTreeParent(current);
    }
    Ok(None)
}

// Source: html_dialog_element.cc:308-345.
pub fn HandleDialogLightDismiss<E: LightDismissEnvironment + ?Sized>(
    e: &E,
    event_type: &str,
    target: u64,
    x: f64,
    y: f64,
) -> Result<(), E::Error> {
    if e.TopmostOpenDialog().is_none() {
        return Ok(());
    }
    let ancestor = FindNearestDialog(e, target, x, y)?;
    if event_type == "pointerdown" {
        e.SetDialogPointerdownTarget(ancestor);
    } else if event_type == "pointerup" {
        let same = ancestor == e.DialogPointerdownTarget();
        e.SetDialogPointerdownTarget(None);
        if !same {
            return Ok(());
        }
        if let Some(topmost) = e.TopmostOpenDialog() {
            if ancestor != Some(topmost) && e.DialogClosedBy(topmost) == ClosedByState::Any {
                e.RequestClose(topmost)?;
            }
        }
    }
    Ok(())
}

// Source: html_dialog_element.cc:349-375.
pub fn HandleDialogLightDismissForClick<E: LightDismissEnvironment + ?Sized>(
    e: &E,
    down: (u64, f64, f64),
    up: (u64, f64, f64),
) -> Result<(), E::Error> {
    if e.TopmostOpenDialog().is_none() {
        return Ok(());
    }
    let down = FindNearestDialog(e, down.0, down.1, down.2)?;
    let up = FindNearestDialog(e, up.0, up.1, up.2)?;
    if down == up {
        if let Some(topmost) = e.TopmostOpenDialog() {
            if down != Some(topmost) && e.DialogClosedBy(topmost) == ClosedByState::Any {
                e.RequestClose(topmost)?;
            }
        }
    }
    Ok(())
}
