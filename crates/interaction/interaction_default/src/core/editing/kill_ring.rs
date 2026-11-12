//! Chromium kill_ring.h:41-54. The embedding supplies the process-global
//! platform service; Editor retains only the source sequence policy.
use foundation::String;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequiredKillRingPlatform {
    HostNotInstalled,
    NativeFailure,
}

/// Exact platform operations, independent of clipboard and thread placement.
/// macOS uses AppKit's kill ring; Chromium's other platforms return empty text.
pub trait KillRingPlatform {
    fn Append(&self, text: &String) -> Result<(), RequiredKillRingPlatform>;
    fn Prepend(&self, text: &String) -> Result<(), RequiredKillRingPlatform>;
    fn Yank(&self) -> Result<String, RequiredKillRingPlatform>;
    fn StartNewSequence(&self) -> Result<(), RequiredKillRingPlatform>;
    fn SetToYankedState(&self) -> Result<(), RequiredKillRingPlatform>;
}

/// Call-scoped leaves of editor_command.cc:1065-1120. No DOM or frame owner
/// crosses this interface, and callbacks may synchronously reenter editing.
pub trait EditorYankContext {
    fn DispatchBeforeInputInsertFromYank(&self, text: &String) -> bool;
    fn IsFrameAttached(&self) -> bool;
    fn UpdateStyleAndLayout(&self);
    fn InsertTextWithoutSendingTextEvent(&self, text: &String, select_inserted: bool);
}

/// Preserve the second Yank in YankAndSelect after beforeinput and layout:
/// either may reenter script and change the process-global native ring.
pub fn ExecuteYank(
    ring: &dyn KillRingPlatform,
    context: &dyn EditorYankContext,
    select_inserted: bool,
) -> Result<bool, RequiredKillRingPlatform> {
    let text = ring.Yank()?;
    if !context.DispatchBeforeInputInsertFromYank(&text) {
        return Ok(true);
    }
    if !context.IsFrameAttached() {
        return Ok(false);
    }
    context.UpdateStyleAndLayout();
    let text = if select_inserted { ring.Yank()? } else { text };
    context.InsertTextWithoutSendingTextEvent(&text, select_inserted);
    ring.SetToYankedState()?;
    Ok(true)
}
