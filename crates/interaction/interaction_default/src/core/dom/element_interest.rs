// Copyright The Chromium Authors. BSD license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: element.cc:1930-2086,12865-13148.
use foundation::popup_types::{InterestLostCancelable, InterestLostPopoverBehavior, InterestState};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterestSource {
    Hover,
    DeHover,
    Focus,
    Blur,
}

pub trait InterestEnvironment {
    type Error;
    fn IsInActiveTreeScope(&self, node: u64) -> bool;
    fn ParentElementOrLocalOwner(&self, node: u64) -> Option<u64>;
    fn InterestForElement(&self, node: u64) -> Option<u64>;
    fn SourceInterestInvoker(&self, node: u64) -> Option<u64>;
    fn State(&self, node: u64) -> InterestState;
    fn HasInvokerData(&self, node: u64) -> bool;
    fn TakeSuppressNextFocusInterest(&self, node: u64) -> bool;
    fn CancelGainedTask(&self, node: u64);
    fn CancelLostTask(&self, node: u64);
    fn ScheduleGainedTask(&self, node: u64) -> Result<(), Self::Error>;
    fn ScheduleLostTask(&self, node: u64) -> Result<(), Self::Error>;
    fn DeferGained(
        &self,
        invoker: u64,
        target: u64,
        state: InterestState,
    ) -> Result<bool, Self::Error>;
    fn DeferLost(
        &self,
        invoker: u64,
        target: u64,
        cancelable: InterestLostCancelable,
        behavior: InterestLostPopoverBehavior,
    ) -> Result<bool, Self::Error>;
    fn ForcedInterest(&self, invoker: u64) -> bool;
    fn DispatchInterest(
        &self,
        kind: &'static str,
        target: u64,
        invoker: u64,
        cancelable: bool,
    ) -> bool;
    fn ChangeInterestState(&self, invoker: u64, target: Option<u64>, state: InterestState);
    fn SetSourceInterestInvoker(&self, target: u64, invoker: u64);
    fn ShowPopoverForInterest(&self, target: u64, invoker: u64) -> Result<(), Self::Error>;
    fn HidePopoverForInterest(&self, target: u64, invoker: u64) -> Result<(), Self::Error>;
}

fn Continue<E: InterestEnvironment + ?Sized>(
    e: &E,
    invoker: u64,
    target: Option<u64>,
    state: InterestState,
) -> bool {
    target.is_some_and(|target| {
        e.IsInActiveTreeScope(invoker)
            && e.InterestForElement(invoker) == Some(target)
            && (state != InterestState::NoInterest
                || e.SourceInterestInvoker(target) == Some(invoker))
    })
}

// Source: element.cc:1945-2023.
pub fn InterestGained<E: InterestEnvironment + ?Sized>(
    e: &E,
    invoker: u64,
    target: Option<u64>,
    state: InterestState,
) -> Result<bool, E::Error> {
    assert_ne!(state, InterestState::NoInterest);
    if !Continue(e, invoker, target, state) {
        return Ok(false);
    }
    let target = target.unwrap();
    if e.DeferGained(invoker, target, state)? {
        return Ok(false);
    }
    if let Some(existing) = e.SourceInterestInvoker(target) {
        if existing == invoker {
            if state == InterestState::ExplicitInterest {
                e.ChangeInterestState(invoker, Some(target), state);
            }
            e.CancelLostTask(invoker);
            return Ok(false);
        }
        if !InterestLost(
            e,
            existing,
            Some(target),
            InterestLostCancelable::Cancelable,
            InterestLostPopoverBehavior::ClosePopovers,
        )? || !Continue(e, invoker, Some(target), state)
        {
            return Ok(false);
        }
    }
    if e.DispatchInterest("interest", target, invoker, true) {
        return Ok(false);
    }
    e.SetSourceInterestInvoker(target, invoker);
    e.ChangeInterestState(invoker, Some(target), state);
    e.ShowPopoverForInterest(target, invoker)?;
    Ok(true)
}

// Source: element.cc:2025-2075.
pub fn InterestLost<E: InterestEnvironment + ?Sized>(
    e: &E,
    invoker: u64,
    target: Option<u64>,
    cancelable: InterestLostCancelable,
    behavior: InterestLostPopoverBehavior,
) -> Result<bool, E::Error> {
    if e.ForcedInterest(invoker) || !Continue(e, invoker, target, InterestState::NoInterest) {
        return Ok(false);
    }
    let target = target.unwrap();
    if e.DeferLost(invoker, target, cancelable, behavior)? {
        return Ok(false);
    }
    if e.DispatchInterest(
        "loseinterest",
        target,
        invoker,
        cancelable == InterestLostCancelable::Cancelable,
    ) {
        return Ok(false);
    }
    if e.SourceInterestInvoker(target) == Some(invoker) {
        e.ChangeInterestState(invoker, Some(target), InterestState::NoInterest);
    }
    if behavior == InterestLostPopoverBehavior::ClosePopovers {
        e.HidePopoverForInterest(target, invoker)?;
    }
    Ok(true)
}

// Source: element.cc:13037-13061. An insertion-ordered set follows the same
// original owners; only this source algorithm's small ancestor worklist exists.
fn AllSources<E: InterestEnvironment + ?Sized>(
    e: &E,
    node: u64,
    seen: &mut HashSet<u64>,
    out: &mut Vec<u64>,
) {
    if let Some(upstream) = e.SourceInterestInvoker(node) {
        if seen.insert(upstream) {
            out.push(upstream);
            AllSources(e, upstream, seen, out);
        }
    }
    if let Some(parent) = e.ParentElementOrLocalOwner(node) {
        if !seen.contains(&parent) {
            AllSources(e, parent, seen, out);
        }
    }
}

// Source: element.cc:13097-13148.
fn Schedule<E: InterestEnvironment + ?Sized>(
    e: &E,
    node: u64,
    source: InterestSource,
) -> Result<(), E::Error> {
    let has_data = e.HasInvokerData(node);
    let mut upstream = Vec::new();
    AllSources(e, node, &mut HashSet::new(), &mut upstream);
    if matches!(source, InterestSource::Hover | InterestSource::Focus) {
        let mut suppress = false;
        if has_data {
            e.CancelLostTask(node);
            let suppression = e.TakeSuppressNextFocusInterest(node);
            suppress = source == InterestSource::Focus && suppression;
        }
        for upstream in upstream {
            e.CancelLostTask(upstream);
        }
        if e.InterestForElement(node).is_some()
            && !suppress
            && (!has_data || e.State(node) == InterestState::NoInterest)
        {
            e.ScheduleGainedTask(node)?;
        }
    } else {
        if has_data {
            e.CancelGainedTask(node);
            if e.State(node) == InterestState::FullInterest {
                e.ScheduleLostTask(node)?;
            }
        }
        for upstream in upstream {
            e.ScheduleLostTask(upstream)?;
        }
    }
    Ok(())
}

// Source: element.cc:13080-13095.
pub fn HandleInterestForHoverOrFocus<E: InterestEnvironment + ?Sized>(
    e: &E,
    node: u64,
    source: InterestSource,
) -> Result<(), E::Error> {
    if !e.IsInActiveTreeScope(node) {
        return Ok(());
    }
    let mut current = Some(node);
    while let Some(node) = current {
        Schedule(e, node, source)?;
        current = e.ParentElementOrLocalOwner(node);
    }
    Ok(())
}

// Source: element.cc:2078-2086.
pub fn HandlePointerEventsForInterestFor<E: InterestEnvironment + ?Sized>(
    e: &E,
    node: u64,
    event_type: &str,
) -> Result<(), E::Error> {
    let source = if event_type == "pointerover" {
        InterestSource::Hover
    } else {
        assert_eq!(event_type, "pointerout");
        InterestSource::DeHover
    };
    HandleInterestForHoverOrFocus(e, node, source)
}
