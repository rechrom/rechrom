// Copyright The Chromium Authors. BSD license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: html_element.cc:1938-2245,2260-2478,2498-2803.
// The environment provides atomic operations on the existing element/document
// owner. Event calls and focus calls may reenter every function below.
use foundation::popup_types::*;

pub trait PopoverLifecycleEnvironment {
    type Error;
    fn Ready(&self, node: u64, show: bool) -> bool;
    fn Kind(&self, node: u64) -> PopoverValueType;
    fn Open(&self, node: u64) -> bool;
    fn HintNewBehavior(&self) -> bool;
    fn StackLen(&self, hint: bool) -> usize;
    fn StackAt(&self, hint: bool, index: usize) -> u64;
    fn StackContains(&self, hint: bool, node: u64) -> bool;
    fn StackPop(&self, hint: bool) -> Option<u64>;
    fn StackPush(&self, hint: bool, node: u64);
    fn HintParent(&self) -> Option<u64>;
    fn SetHintParent(&self, node: Option<u64>);
    fn HidingNestingCount(&self) -> usize;
    fn BeginHide(&self, node: u64) -> bool;
    fn EndHide(&self, node: u64, was_hiding: bool);
    fn BeginShow(&self) -> bool;
    fn EndShow(&self, was_showing: bool);
    fn ForcedOpen(&self, node: u64) -> bool;
    fn BeforeToggle(&self, node: u64, opening: bool, source: Option<u64>) -> bool;
    fn QueueToggle(&self, node: u64, opening: bool, source: Option<u64>)
        -> Result<(), Self::Error>;
    fn MarkInvokersDirty(&self, node: u64);
    fn ScheduleRemoval(&self, node: u64);
    fn RemoveImmediately(&self, node: u64);
    fn CommitHidden(&self, node: u64);
    fn CommitShown(&self, node: u64, invoker: Option<u64>, stack_hint: Option<bool>);
    fn LoseInterestWithoutClosing(&self, node: u64) -> Result<(), Self::Error>;
    fn FocusOnShow(&self, node: u64);
    fn RestoreFocusOnHide(&self, node: u64, focus: HidePopoverFocusBehavior);
    fn FocusedElement(&self) -> Option<u64>;
    fn RememberPreviouslyFocused(&self, node: u64, focused: Option<u64>);
    fn FindAncestor(
        &self,
        node: u64,
        invoker: Option<u64>,
        stack_hint: Option<bool>,
    ) -> Option<u64>;
    fn WarningReentrantPopover(&self);
    fn AttributeType(&self, node: u64) -> PopoverValueType;
    fn SetType(&self, node: u64, kind: PopoverValueType);
    fn RemoveData(&self, node: u64);
    fn IsMenuList(&self, node: u64) -> bool;
    fn DatalistManagesPopover(&self, node: u64) -> bool;
    fn MaybeCreateMenuTriangle(&self, node: u64, invoker: Option<u64>) -> Result<(), Self::Error>;
    fn RecheckMenuTriangleIfMenuList(&self, node: u64) -> Result<(), Self::Error>;
}

// Source: html_element.cc:1295-1372. Re-read the live attribute after any
// beforetoggle listener has changed it; never restore a stale parsed type.
pub fn UpdatePopoverAttribute<E: PopoverLifecycleEnvironment + ?Sized>(
    e: &E,
    node: u64,
) -> Result<(), E::Error> {
    if e.DatalistManagesPopover(node) {
        return Ok(());
    }
    let mut kind = e.AttributeType(node);
    if e.Kind(node) != PopoverValueType::None {
        if e.Kind(node) == kind {
            return Ok(());
        }
        if e.Open(node) {
            HidePopoverInternal(
                e,
                node,
                None,
                HidePopoverFocusBehavior::FocusPreviousElement,
                HidePopoverTransitionBehavior::FireEventsAndWaitForTransitions,
            )?;
            kind = e.AttributeType(node);
        }
    }
    if kind == PopoverValueType::None {
        if e.Kind(node) != PopoverValueType::None {
            if e.IsMenuList(node) {
                e.SetType(node, PopoverValueType::Auto);
            } else {
                e.RemoveData(node);
            }
        }
    } else {
        e.SetType(node, kind);
    }
    Ok(())
}
struct Hiding<'a, E: PopoverLifecycleEnvironment + ?Sized> {
    e: &'a E,
    node: u64,
    was: bool,
}
impl<E: PopoverLifecycleEnvironment + ?Sized> Drop for Hiding<'_, E> {
    fn drop(&mut self) {
        self.e.EndHide(self.node, self.was);
    }
}
struct Showing<'a, E: PopoverLifecycleEnvironment + ?Sized> {
    e: &'a E,
    was: bool,
}
impl<E: PopoverLifecycleEnvironment + ?Sized> Drop for Showing<'_, E> {
    fn drop(&mut self) {
        self.e.EndShow(self.was);
    }
}
fn Top<E: PopoverLifecycleEnvironment + ?Sized>(e: &E, hint: bool) -> Option<u64> {
    e.StackLen(hint).checked_sub(1).map(|i| e.StackAt(hint, i))
}
fn TopIgnoring<E: PopoverLifecycleEnvironment + ?Sized>(
    e: &E,
    hint: bool,
    held: &[u64],
) -> Option<u64> {
    let mut index = e.StackLen(hint);
    for expected in held {
        index = index.checked_sub(1)?;
        if e.StackAt(hint, index) != *expected {
            return None;
        }
    }
    index.checked_sub(1).map(|i| e.StackAt(hint, i))
}

// Source: html_element.cc:2260-2290.
pub fn CloseEntirePopoverStack<E: PopoverLifecycleEnvironment + ?Sized>(
    e: &E,
    hint: bool,
    focus: HidePopoverFocusBehavior,
    transition: HidePopoverTransitionBehavior,
) -> Result<PopoverHideResult, E::Error> {
    let mut held = Vec::new();
    while let Some(top) = Top(e, hint) {
        if HidePopoverInternal(e, top, None, focus, transition)?
            == PopoverHideResult::ForcedOpenByInspector
        {
            assert_eq!(e.StackPop(hint), Some(top));
            held.push(top);
        }
    }
    let forced = !held.is_empty();
    for node in held.into_iter().rev() {
        e.StackPush(hint, node);
    }
    Ok(if forced {
        PopoverHideResult::ForcedOpenByInspector
    } else {
        PopoverHideResult::Hidden
    })
}

fn HideStackUntil<E: PopoverLifecycleEnvironment + ?Sized>(
    e: &E,
    hint: bool,
    endpoint: u64,
    focus: HidePopoverFocusBehavior,
    mut transition: HidePopoverTransitionBehavior,
    held_out: &mut Vec<u64>,
) -> Result<PopoverHideResult, E::Error> {
    if e.HintNewBehavior() && !e.StackContains(hint, endpoint) {
        return CloseEntirePopoverStack(e, hint, focus, transition);
    }
    let mut result = PopoverHideResult::Hidden;
    loop {
        held_out.clear();
        if e.HintNewBehavior() && !e.StackContains(hint, endpoint) {
            return Ok(PopoverHideResult::Hidden);
        }
        let mut last = None;
        for index in (0..e.StackLen(hint)).rev() {
            let node = e.StackAt(hint, index);
            if node == endpoint {
                break;
            }
            last = Some(node);
        }
        let Some(last) = last else {
            return Ok(PopoverHideResult::Hidden);
        };
        while e.Open(last) {
            let top = Top(e, hint).expect("source popover stack contains last to hide");
            if HidePopoverInternal(e, top, None, focus, transition)?
                == PopoverHideResult::ForcedOpenByInspector
            {
                held_out.push(top);
                assert_eq!(e.StackPop(hint), Some(top));
                if e.HintNewBehavior() && hint && e.StackLen(hint) == 0 {
                    e.SetHintParent(None);
                }
                if top == last {
                    break;
                }
            }
        }
        let repeating = (held_out.is_empty() || e.StackLen(hint) != 0)
            && e.StackContains(hint, endpoint)
            && Top(e, hint) != Some(endpoint);
        if repeating {
            assert!(!e.HintNewBehavior());
            transition = HidePopoverTransitionBehavior::NoEventsNoWaiting;
            e.WarningReentrantPopover();
        }
        if !held_out.is_empty() {
            for &node in held_out.iter().rev() {
                e.StackPush(hint, node);
            }
            result = PopoverHideResult::ForcedOpenByInspector;
        }
        if !repeating {
            return Ok(result);
        }
    }
}

// Source: html_element.cc:2300-2478. The caller's inspector list is in the
// source's top-to-bottom order and refers to the original popover identities.
pub fn HideAllPopoversUntil<E: PopoverLifecycleEnvironment + ?Sized>(
    e: &E,
    endpoint: Option<u64>,
    focus: HidePopoverFocusBehavior,
    transition: HidePopoverTransitionBehavior,
    held: &mut Vec<u64>,
) -> Result<PopoverHideResult, E::Error> {
    if endpoint.is_some_and(|node| !e.Open(node)) {
        return Ok(PopoverHideResult::Hidden);
    }
    let Some(endpoint) = endpoint else {
        if CloseEntirePopoverStack(e, true, focus, transition)?
            == PopoverHideResult::ForcedOpenByInspector
        {
            return Ok(PopoverHideResult::ForcedOpenByInspector);
        }
        if e.HintNewBehavior() {
            e.SetHintParent(None);
        }
        return CloseEntirePopoverStack(e, false, focus, transition);
    };
    if e.StackContains(true, endpoint) {
        let result = HideStackUntil(e, true, endpoint, focus, transition, held)?;
        if e.HintNewBehavior() && e.HidingNestingCount() == 0 {
            let result = if let Some(parent) = e
                .HintParent()
                .filter(|&parent| e.StackContains(false, parent))
            {
                HideStackUntil(e, false, parent, focus, transition, &mut Vec::new())?
            } else {
                CloseEntirePopoverStack(e, false, focus, transition)?
            };
            if result == PopoverHideResult::ForcedOpenByInspector {
                return Ok(result);
            }
        }
        return Ok(result);
    }
    if e.StackContains(false, endpoint) {
        if CloseEntirePopoverStack(e, true, focus, transition)?
            == PopoverHideResult::ForcedOpenByInspector
        {
            return Ok(PopoverHideResult::ForcedOpenByInspector);
        }
        if e.HintNewBehavior() {
            e.SetHintParent(None);
        }
        return HideStackUntil(e, false, endpoint, focus, transition, held);
    }
    Ok(PopoverHideResult::Hidden)
}

// Source: html_element.cc:2498-2803.
pub fn HidePopoverInternal<E: PopoverLifecycleEnvironment + ?Sized>(
    e: &E,
    node: u64,
    invoker: Option<u64>,
    focus: HidePopoverFocusBehavior,
    mut transition: HidePopoverTransitionBehavior,
) -> Result<PopoverHideResult, E::Error> {
    if e.ForcedOpen(node) {
        return Ok(PopoverHideResult::ForcedOpenByInspector);
    }
    if !e.Ready(node, false) {
        return Ok(PopoverHideResult::Hidden);
    }
    let was = e.BeginHide(node);
    let _scope = Hiding { e, node, was };
    if was {
        transition = HidePopoverTransitionBehavior::NoEventsNoWaiting;
    }
    let mut held = Vec::new();
    let mut hint = None;
    if matches!(
        e.Kind(node),
        PopoverValueType::Auto | PopoverValueType::Hint
    ) {
        if e.HintNewBehavior() && e.HintParent() == Some(node) {
            CloseEntirePopoverStack(e, true, focus, transition)?;
            e.SetHintParent(None);
        }
        HideAllPopoversUntil(e, Some(node), focus, transition, &mut held)?;
        if !e.Ready(node, false) {
            return Ok(PopoverHideResult::Hidden);
        }
        hint = Some(TopIgnoring(e, false, &held) != Some(node));
    }
    e.MarkInvokersDirty(node);
    if transition == HidePopoverTransitionBehavior::FireEventsAndWaitForTransitions {
        if !e.BeforeToggle(node, false, invoker) {
            return Ok(PopoverHideResult::Hidden);
        }
        if !e.Ready(node, false) {
            return Ok(PopoverHideResult::Hidden);
        }
        if hint
            .is_some_and(|hint| e.StackLen(hint) != 0 && TopIgnoring(e, hint, &held) != Some(node))
        {
            e.WarningReentrantPopover();
            HideAllPopoversUntil(
                e,
                Some(node),
                focus,
                HidePopoverTransitionBehavior::NoEventsNoWaiting,
                &mut held,
            )?;
            if !e.Ready(node, false) {
                return Ok(PopoverHideResult::Hidden);
            }
        }
        e.LoseInterestWithoutClosing(node)?;
        if !e.Ready(node, false) {
            return Ok(PopoverHideResult::Hidden);
        }
        if hint
            .is_some_and(|hint| e.StackLen(hint) != 0 && TopIgnoring(e, hint, &held) != Some(node))
        {
            HideAllPopoversUntil(
                e,
                Some(node),
                focus,
                HidePopoverTransitionBehavior::NoEventsNoWaiting,
                &mut held,
            )?;
            if !e.Ready(node, false) {
                return Ok(PopoverHideResult::Hidden);
            }
        }
        e.QueueToggle(node, false, invoker)?;
        e.ScheduleRemoval(node);
    } else {
        e.RemoveImmediately(node);
    }
    e.CommitHidden(node);
    e.RecheckMenuTriangleIfMenuList(node)?;
    e.RestoreFocusOnHide(node, focus);
    Ok(PopoverHideResult::Hidden)
}

// Source: html_element.cc:1938-2245.
pub fn ShowPopoverInternal<E: PopoverLifecycleEnvironment + ?Sized>(
    e: &E,
    node: u64,
    invoker: Option<u64>,
) -> Result<(), E::Error> {
    if !e.Ready(node, true) {
        return Ok(());
    }
    let was = e.BeginShow();
    let _scope = Showing { e, was };
    let transition = if was || e.HidingNestingCount() != 0 {
        HidePopoverTransitionBehavior::NoEventsNoWaiting
    } else {
        HidePopoverTransitionBehavior::FireEventsAndWaitForTransitions
    };
    if !e.BeforeToggle(node, true, invoker) || !e.Ready(node, true) {
        return Ok(());
    }
    let kind = e.Kind(node);
    let mut auto = kind == PopoverValueType::Auto;
    let mut stack_hint = None;
    if matches!(kind, PopoverValueType::Auto | PopoverValueType::Hint) {
        let focus = HidePopoverFocusBehavior::None;
        if !e.HintNewBehavior() {
            if let Some(ancestor) = (kind == PopoverValueType::Hint)
                .then(|| e.FindAncestor(node, invoker, Some(true)))
                .flatten()
            {
                HideAllPopoversUntil(e, Some(ancestor), focus, transition, &mut Vec::new())?;
                stack_hint = Some(true);
            } else {
                CloseEntirePopoverStack(e, true, focus, transition)?;
                if e.Kind(node) != kind || !e.Ready(node, true) {
                    return Ok(());
                }
                let ancestor = e.FindAncestor(node, invoker, Some(false));
                if auto || ancestor.is_some() {
                    HideAllPopoversUntil(e, ancestor, focus, transition, &mut Vec::new())?;
                    stack_hint = Some(false);
                } else {
                    stack_hint = Some(true);
                }
            }
        } else {
            let parent = e.FindAncestor(node, invoker, None);
            if parent.is_some_and(|parent| e.StackContains(true, parent)) {
                if auto {
                    auto = false;
                }
                HideAllPopoversUntil(e, parent, focus, transition, &mut Vec::new())?;
                stack_hint = Some(true);
            } else {
                let result = CloseEntirePopoverStack(e, true, focus, transition)?;
                if result != PopoverHideResult::ForcedOpenByInspector {
                    e.SetHintParent(None);
                }
                if e.Kind(node) != kind || !e.Ready(node, true) {
                    return Ok(());
                }
                let parent = e.FindAncestor(node, invoker, None);
                if auto {
                    HideAllPopoversUntil(e, parent, focus, transition, &mut Vec::new())?;
                    if e.Kind(node) != kind || !e.Ready(node, true) {
                        return Ok(());
                    }
                    stack_hint = Some(false);
                } else {
                    if parent.is_some() && e.StackLen(true) == 0 {
                        e.SetHintParent(parent);
                    }
                    stack_hint = Some(true);
                }
            }
        }
        if e.Kind(node) != kind || !e.Ready(node, true) {
            return Ok(());
        }
    }
    let restore = stack_hint.is_some() && e.StackLen(true) == 0 && e.StackLen(false) == 0;
    let originally_focused = e.FocusedElement();
    e.MarkInvokersDirty(node);
    e.CommitShown(node, invoker, stack_hint);
    e.MaybeCreateMenuTriangle(node, invoker)?;
    e.FocusOnShow(node);
    if restore && e.Open(node) {
        e.RememberPreviouslyFocused(node, originally_focused);
    }
    e.QueueToggle(node, true, invoker)
}
