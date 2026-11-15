// Copyright The Chromium Authors. BSD license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Pure values crossing the DOM/interaction popup boundary. Mutable rare data,
// dialog state, document stacks and CloseWatchers are owned by DOM.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PopoverValueType {
    #[default]
    None,
    Auto,
    Hint,
    Manual,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PopoverVisibilityState {
    #[default]
    Hidden,
    Showing,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopoverStackKind {
    Auto,
    Hint,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HidePopoverFocusBehavior {
    None,
    FocusPreviousElement,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HidePopoverTransitionBehavior {
    NoEventsNoWaiting,
    FireEventsAndWaitForTransitions,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopoverHideResult {
    Hidden,
    ForcedOpenByInspector,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClosedByState {
    Any,
    CloseRequest,
    None,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CloseWatcherId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PopupTaskId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PopupToggleEvent {
    pub old_open: bool,
    pub new_open: bool,
    pub source: Option<u64>,
    pub composed: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CloseWatcherOwner {
    Popover(u64),
    Dialog(u64),
    Script,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopupTopLayerReason {
    Popover,
    Dialog,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InterestState {
    #[default]
    NoInterest,
    FullInterest,
    ExplicitInterest,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterestLostCancelable {
    Cancelable,
    NotCancelable,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterestLostPopoverBehavior {
    ClosePopovers,
    DontClosePopovers,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterestInvokerKind {
    None,
    Button,
    Anchor,
    MenuItem,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MenuTriangleId(pub u64);
#[derive(Clone, Copy, Debug)]
pub struct DeferredInterestGain {
    pub invoker: u64,
    pub target: u64,
    pub state: InterestState,
}
#[derive(Clone, Copy, Debug)]
pub struct DeferredInterestLoss {
    pub invoker: u64,
    pub target: u64,
    pub cancelable: InterestLostCancelable,
    pub behavior: InterestLostPopoverBehavior,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InvokerCollectionKind {
    Popover,
    Command,
}
