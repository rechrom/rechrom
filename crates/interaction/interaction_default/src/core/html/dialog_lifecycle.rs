// Copyright The Chromium Authors. BSD license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: html_dialog_element.cc:148-238,552-616,664-676,709-750.
use foundation::popup_types::*;

pub trait DialogLifecycleEnvironment {
    type Error;
    fn Open(&self, node: u64) -> bool;
    fn Active(&self) -> bool;
    fn Connected(&self, node: u64) -> bool;
    fn Modal(&self, node: u64) -> bool;
    fn PopoverOpen(&self, node: u64) -> bool;
    fn BeginClose(&self, node: u64) -> bool;
    fn EndClose(&self, node: u64);
    fn ToggleEvents(
        &self,
        node: u64,
        opening: bool,
        source: Option<u64>,
        modal: bool,
    ) -> Result<bool, Self::Error>;
    fn RemoveOpenAttribute(&self, node: u64);
    fn SetModal(&self, node: u64, modal: bool);
    fn EraseOpenDialog(&self, node: u64);
    fn ScheduleRemoval(&self, node: u64);
    fn InertSubtreesChanged(&self);
    fn SetReturnValue(&self, node: u64, value: Option<&[u16]>);
    fn ScheduleCloseEvent(&self, node: u64) -> Result<(), Self::Error>;
    fn DestroyCloseWatcher(&self, node: u64);
    fn TakePreviousFocus(&self, node: u64) -> Option<u64>;
    fn ContainsFocused(&self, node: u64) -> bool;
    fn Focus(&self, node: u64);
    fn AddToTopLayer(&self, node: u64);
    fn AddOpenAttribute(&self, node: u64);
    fn SetCloseWatcherEnabledState(&self, node: u64);
    fn HidePopoversForDialog(&self, node: u64) -> Result<(), Self::Error>;
    fn SetFocusForDialog(&self, node: u64);
    fn InvalidState(&self, message: &'static str) -> Self::Error;
    fn SetRequestClose(&self, node: u64, value: Option<&[u16]>, source: Option<u64>);
    fn CloseWatcherRequestCloseAlways(&self, node: u64) -> Result<(), Self::Error>;
}
struct Closing<'a, E: DialogLifecycleEnvironment + ?Sized> {
    e: &'a E,
    node: u64,
}
impl<E: DialogLifecycleEnvironment + ?Sized> Drop for Closing<'_, E> {
    fn drop(&mut self) {
        self.e.EndClose(self.node);
    }
}

// Source: html_dialog_element.cc:148-224.
pub fn Close<E: DialogLifecycleEnvironment + ?Sized>(
    e: &E,
    node: u64,
    value: Option<&[u16]>,
    source: Option<u64>,
    open_being_removed: bool,
) -> Result<(), E::Error> {
    if !e.BeginClose(node) {
        return Ok(());
    }
    let was_modal;
    {
        let _closing = Closing { e, node };
        if !e.Open(node) && !open_being_removed {
            return Ok(());
        }
        e.ToggleEvents(node, false, source, false)?;
        if !e.Open(node) && !open_being_removed {
            return Ok(());
        }
        e.RemoveOpenAttribute(node);
        was_modal = e.Modal(node);
        e.SetModal(node, false);
        if e.Connected(node) {
            e.EraseOpenDialog(node);
        }
        if was_modal {
            e.ScheduleRemoval(node);
        }
        e.InertSubtreesChanged();
        e.SetReturnValue(node, value);
        e.ScheduleCloseEvent(node)?;
        e.DestroyCloseWatcher(node);
    }
    if !open_being_removed {
        if let Some(previous) = e.TakePreviousFocus(node) {
            if was_modal || e.ContainsFocused(node) {
                e.Focus(previous);
            }
        }
    }
    Ok(())
}

// Source: html_dialog_element.cc:225-238. The CloseWatcher owner supplies
// the cancel/close event lifetime; dialog never dispatches a parallel loop.
pub fn RequestCloseInternal<E: DialogLifecycleEnvironment + ?Sized>(
    e: &E,
    node: u64,
    value: Option<&[u16]>,
    source: Option<u64>,
) -> Result<(), E::Error> {
    if !e.Open(node) || !e.Active() {
        return Ok(());
    }
    e.SetRequestClose(node, value, source);
    e.CloseWatcherRequestCloseAlways(node)?;
    e.SetCloseWatcherEnabledState(node);
    Ok(())
}

// Source: html_dialog_element.cc:552-616.
pub fn ShowModal<E: DialogLifecycleEnvironment + ?Sized>(
    e: &E,
    node: u64,
    source: Option<u64>,
) -> Result<(), E::Error> {
    if e.Open(node) {
        if !e.Modal(node) {
            return Err(e.InvalidState("dialog is already open as a non-modal dialog"));
        }
        return Ok(());
    }
    if !e.Connected(node) {
        return Err(e.InvalidState("dialog is not in a Document"));
    }
    if e.PopoverOpen(node) {
        return Err(e.InvalidState("dialog is already open as a Popover"));
    }
    if !e.Active() {
        return Err(e.InvalidState("dialog Document is not fully active"));
    }
    if !e.ToggleEvents(node, true, source, true)? {
        return Ok(());
    }
    e.AddToTopLayer(node);
    e.SetModal(node, true);
    e.AddOpenAttribute(node);
    e.InertSubtreesChanged();
    e.SetCloseWatcherEnabledState(node);
    e.HidePopoversForDialog(node)?;
    e.SetFocusForDialog(node);
    Ok(())
}
