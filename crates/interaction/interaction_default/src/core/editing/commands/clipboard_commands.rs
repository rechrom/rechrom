//! Chromium clipboard_commands.cc:226-663,819-900. Clipboard acquisition is
//! an embedding service; event cancellation, frame reentry and editing gates
//! remain in the command owner. A snapshot is scoped to one Paste invocation.
use super::super::editing_behavior::EditingBehaviorType;

#[derive(Clone, Default)]
pub struct PrimaryClipboardSnapshot {
    pub plain_text: String,
    pub html: Option<ClipboardHtml>,
    pub smart_replace: bool,
    pub rtf: Option<String>,
    pub svg: Option<String>,
    pub uri_list: Option<String>,
    /// Canonical immutable File owners survive paste/beforeinput reentry.
    pub png: Option<std::sync::Arc<interaction_dispatch::core::fileapi::file::File>>,
    pub files: Vec<std::sync::Arc<interaction_dispatch::core::fileapi::file::File>>,
    /// Chromium's DataTransfer custom-data map contains UTF-16 strings, not
    /// arbitrary native clipboard flavors. Preserve unpaired surrogate units.
    pub custom_data: Vec<(String, Vec<u16>)>,
}
#[derive(Clone)]
pub struct ClipboardHtml {
    pub markup: String,
    pub source_url: String,
    pub fragment_start: u32,
    pub fragment_end: u32,
}
#[derive(Clone, Copy, Debug)]
pub enum ClipboardHostError {
    Unavailable,
    ReadFailed,
    WriteFailed,
}
pub trait ClipboardHost {
    fn ReadPrimarySelection(&self) -> Result<PrimaryClipboardSnapshot, ClipboardHostError>;
    fn ReadClipboard(&self) -> Result<PrimaryClipboardSnapshot, ClipboardHostError> {
        Err(ClipboardHostError::Unavailable)
    }
    fn WriteClipboard(
        &self,
        _snapshot: &PrimaryClipboardSnapshot,
    ) -> Result<(), ClipboardHostError> {
        Err(ClipboardHostError::Unavailable)
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct ClipboardAccessPolicy {
    pub javascript_can_access_clipboard: bool,
    pub dom_paste_allowed: bool,
    pub allow_read: bool,
    pub allow_write: bool,
}
pub fn CanReadClipboard(
    policy: ClipboardAccessPolicy,
    source: super::editor_command_table::EditorCommandSource,
) -> bool {
    source == super::editor_command_table::EditorCommandSource::kMenuOrKeyBinding
        || policy.javascript_can_access_clipboard && policy.dom_paste_allowed
        || policy.allow_read
}
pub fn CanWriteClipboard(
    policy: ClipboardAccessPolicy,
    source: super::editor_command_table::EditorCommandSource,
    transient_activation: bool,
) -> bool {
    source == super::editor_command_table::EditorCommandSource::kMenuOrKeyBinding
        || policy.javascript_can_access_clipboard
        || transient_activation
        || policy.allow_write
}
pub trait ClipboardEventTargetContext {
    type Node: Clone;
    fn SelectionIsHidden(&self) -> bool;
    fn FocusedElement(&self) -> Option<Self::Node>;
    fn IsEditable(&self, node: &Self::Node) -> bool;
    fn Body(&self) -> Option<Self::Node>;
    fn SelectionStartContainer(&self) -> Option<Self::Node>;
    fn IsElement(&self, node: &Self::Node) -> bool;
    fn ParentElement(&self, node: &Self::Node) -> Option<Self::Node>;
    fn IsInUserAgentShadowRoot(&self, node: &Self::Node) -> bool;
    fn OwnerShadowHost(&self, node: &Self::Node) -> Option<Self::Node>;
}
/// clipboard_commands.cc:177-219, stable container-node target branch.
pub fn FindEventTargetForClipboardEvent<C: ClipboardEventTargetContext>(
    context: &C,
) -> Option<C::Node> {
    if context.SelectionIsHidden() {
        if let Some(focused) = context.FocusedElement().filter(|n| !context.IsEditable(n)) {
            return Some(focused);
        }
        return context.Body();
    }
    let Some(container) = context.SelectionStartContainer() else {
        return context.Body();
    };
    let Some(target) = (if context.IsElement(&container) {
        Some(container)
    } else {
        context.ParentElement(&container)
    }) else {
        return context.Body();
    };
    if context.IsInUserAgentShadowRoot(&target) {
        context.OwnerShadowHost(&target)
    } else {
        Some(target)
    }
}
pub trait PasteCommands {
    type Error;
    fn Source(&self) -> super::editor_command_table::EditorCommandSource {
        super::editor_command_table::EditorCommandSource::kMenuOrKeyBinding
    }
    fn Behavior(&self) -> Result<EditingBehaviorType, Self::Error>;
    fn ReadPrimarySelection(&self) -> Result<PrimaryClipboardSnapshot, Self::Error>;
    fn ReadClipboard(&self) -> Result<PrimaryClipboardSnapshot, Self::Error>;
    fn DispatchPaste(&self, snapshot: &PrimaryClipboardSnapshot) -> Result<bool, Self::Error>;
    fn FrameIsCurrent(&self) -> bool;
    fn CanPaste(&self) -> bool;
    fn UpdateStyleAndLayoutForEditing(&self) -> Result<(), Self::Error>;
    fn SelectionHasFocus(&self) -> bool;
    fn CanEditRichly(&self) -> bool;
    fn DispatchBeforeInput(
        &self,
        snapshot: &PrimaryClipboardSnapshot,
        rich: bool,
    ) -> Result<bool, Self::Error>;
    fn HasActiveEditContext(&self) -> bool;
    fn PasteFromClipboard(&self, snapshot: &PrimaryClipboardSnapshot) -> Result<(), Self::Error>;
    fn PasteAsPlainText(&self, snapshot: &PrimaryClipboardSnapshot) -> Result<(), Self::Error>;
}
/// Menu/keybinding branch: selection-mode restoration is represented by a
/// primary-selection read, so no mutable global clipboard mode escapes this
/// synchronous command or listener reentry.
pub fn ExecutePasteGlobalSelection<C: PasteCommands>(commands: &C) -> Result<bool, C::Error> {
    if !commands.Behavior()?.SupportsGlobalSelection() {
        return Ok(false);
    }
    let snapshot = commands.ReadPrimarySelection()?;
    Paste(commands, &snapshot, false)
}
/// clipboard_commands.cc:569-663,819-825,842-900: menu/keybinding entry points.
pub fn ExecutePaste<C: PasteCommands>(commands: &C) -> Result<bool, C::Error> {
    if commands.Source() == super::editor_command_table::EditorCommandSource::kDom
        && commands.HasActiveEditContext()
    {
        return Ok(true);
    }
    let snapshot = commands.ReadClipboard()?;
    Paste(commands, &snapshot, false)
}
pub fn ExecutePasteAndMatchStyle<C: PasteCommands>(commands: &C) -> Result<bool, C::Error> {
    let mut snapshot = commands.ReadClipboard()?;
    snapshot.html = None;
    Paste(commands, &snapshot, true)
}
fn Paste<C: PasteCommands>(
    commands: &C,
    snapshot: &PrimaryClipboardSnapshot,
    plain: bool,
) -> Result<bool, C::Error> {
    if !commands.DispatchPaste(snapshot)? || !commands.FrameIsCurrent() || !commands.CanPaste() {
        return Ok(!plain);
    }
    commands.UpdateStyleAndLayoutForEditing()?;
    let menu =
        commands.Source() == super::editor_command_table::EditorCommandSource::kMenuOrKeyBinding;
    if menu && !commands.SelectionHasFocus() {
        return Ok(!plain);
    }
    let rich = !plain && commands.CanEditRichly();
    if menu
        && (!commands.DispatchBeforeInput(snapshot, rich)?
            || !commands.FrameIsCurrent()
            || commands.HasActiveEditContext())
    {
        return Ok(true);
    }
    if rich {
        commands.PasteFromClipboard(snapshot)?;
    } else {
        commands.PasteAsPlainText(snapshot)?;
    }
    Ok(true)
}

pub trait CopyCutCommands {
    type Error;
    fn Source(&self) -> super::editor_command_table::EditorCommandSource {
        super::editor_command_table::EditorCommandSource::kMenuOrKeyBinding
    }
    fn UpdateStyleAndLayoutForEditing(&self) -> Result<(), Self::Error>;
    fn DispatchCopyOrCut(&self, cut: bool) -> Result<bool, Self::Error>;
    fn FrameIsCurrent(&self) -> bool;
    fn CanCopy(&self) -> bool;
    fn CanCut(&self) -> bool;
    fn SelectionIsHidden(&self) -> bool;
    fn SelectionHasFocus(&self) -> bool;
    fn WriteSelectionToClipboard(&self) -> Result<(), Self::Error>;
    fn DispatchBeforeCut(&self) -> Result<bool, Self::Error>;
    fn HasActiveEditContext(&self) -> bool;
    fn DeleteSelection(&self) -> Result<(), Self::Error>;
}
/// clipboard_commands.cc:360-403,427-493. Clipboard event handlers run before
/// selection gates; cancelled writable events commit their DataTransfer.
pub fn ExecuteCopy<C: CopyCutCommands>(commands: &C) -> Result<bool, C::Error> {
    commands.UpdateStyleAndLayoutForEditing()?;
    if !commands.DispatchCopyOrCut(false)? || !commands.FrameIsCurrent() || !commands.CanCopy() {
        return Ok(true);
    }
    commands.UpdateStyleAndLayoutForEditing()?;
    if !commands.SelectionIsHidden() {
        commands.WriteSelectionToClipboard()?;
    }
    Ok(true)
}
pub fn ExecuteCut<C: CopyCutCommands>(commands: &C) -> Result<bool, C::Error> {
    let menu =
        commands.Source() == super::editor_command_table::EditorCommandSource::kMenuOrKeyBinding;
    if !menu && commands.HasActiveEditContext() {
        return Ok(true);
    }
    commands.UpdateStyleAndLayoutForEditing()?;
    if !commands.DispatchCopyOrCut(true)? || !commands.FrameIsCurrent() || !commands.CanCut() {
        return Ok(true);
    }
    commands.UpdateStyleAndLayoutForEditing()?;
    if menu && !commands.SelectionHasFocus() || !commands.CanCut() {
        return Ok(true);
    }
    commands.WriteSelectionToClipboard()?;
    if !menu
        || commands.DispatchBeforeCut()?
            && commands.FrameIsCurrent()
            && !commands.HasActiveEditContext()
    {
        commands.DeleteSelection()?;
    }
    Ok(true)
}
