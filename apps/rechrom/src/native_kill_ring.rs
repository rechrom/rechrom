//! Embedding-owned Chromium KillRing platform implementation.
//! Source: blink/core/editing/kill_ring_mac.mm:30-84 and kill_ring_none.cc:32-47.
//! The kill ring is AppKit process state, never the ordinary clipboard.
use foundation::String;
use interaction::default::core::editing::kill_ring::{KillRingPlatform, RequiredKillRingPlatform};

pub(crate) struct NativeKillRing;

#[cfg(target_os = "macos")]
mod mac {
    use super::*;
    extern "C" {
        fn rechrom_kill_ring_mutate(operation: u32, text: *const u16, length: usize) -> bool;
        fn rechrom_kill_ring_yank(text: *mut *mut u16, length: *mut usize) -> bool;
        fn rechrom_kill_ring_free(text: *mut u16);
    }
    fn mutate(operation: u32, text: &[u16]) -> Result<(), RequiredKillRingPlatform> {
        if unsafe { rechrom_kill_ring_mutate(operation, text.as_ptr(), text.len()) } {
            Ok(())
        } else {
            Err(RequiredKillRingPlatform::NativeFailure)
        }
    }
    impl KillRingPlatform for NativeKillRing {
        fn Append(&self, text: &String) -> Result<(), RequiredKillRingPlatform> {
            mutate(0, text.Span16().unwrap_or_default())
        }
        fn Prepend(&self, text: &String) -> Result<(), RequiredKillRingPlatform> {
            mutate(1, text.Span16().unwrap_or_default())
        }
        fn Yank(&self) -> Result<String, RequiredKillRingPlatform> {
            let mut text = std::ptr::null_mut();
            let mut length = 0;
            if !unsafe { rechrom_kill_ring_yank(&mut text, &mut length) } {
                return Err(RequiredKillRingPlatform::NativeFailure);
            }
            if text.is_null() {
                return Ok(String::new());
            }
            let result = String::from_utf16(unsafe { std::slice::from_raw_parts(text, length) });
            unsafe { rechrom_kill_ring_free(text) };
            Ok(result)
        }
        fn StartNewSequence(&self) -> Result<(), RequiredKillRingPlatform> {
            mutate(2, &[])
        }
        fn SetToYankedState(&self) -> Result<(), RequiredKillRingPlatform> {
            mutate(3, &[])
        }
    }
}

// Chromium's Linux/Windows implementation deliberately has no retained text,
// native clipboard access, initialization, or sequence state.
#[cfg(not(target_os = "macos"))]
impl KillRingPlatform for NativeKillRing {
    fn Append(&self, _: &String) -> Result<(), RequiredKillRingPlatform> {
        Ok(())
    }
    fn Prepend(&self, _: &String) -> Result<(), RequiredKillRingPlatform> {
        Ok(())
    }
    fn Yank(&self) -> Result<String, RequiredKillRingPlatform> {
        Ok(String::new())
    }
    fn StartNewSequence(&self) -> Result<(), RequiredKillRingPlatform> {
        Ok(())
    }
    fn SetToYankedState(&self) -> Result<(), RequiredKillRingPlatform> {
        Ok(())
    }
}

/// Source RenderWidgetHostViewMacEditCommandHelper::CommandNameForSelector,
/// .mm:156-177; RenderWidgetHostViewCocoa::doCommandBySelector, .mm:2708-2767.
/// Winit already runs interpretKeyEvents; consume its original callbacks once.
pub(crate) fn editing_commands(
    event: &winit::event::KeyEvent,
) -> Vec<interaction::default::core::editing::commands::editor_command_table::EditingCommandType> {
    #[cfg(target_os = "macos")]
    {
        use interaction::default::core::editing::commands::editor_command_table::EditingCommandType;
        use winit::platform::macos::KeyEventExtMacOS;
        event
            .edit_commands()
            .iter()
            .filter_map(|selector| {
                let command = match selector.as_str() {
                    "insertParagraphSeparator:" | "insertNewlineIgnoringFieldEditor:" => {
                        "InsertNewline"
                    }
                    "insertTabIgnoringFieldEditor:" => "InsertTab",
                    "pageDown:" => "MovePageDown",
                    "pageDownAndModifySelection:" => "MovePageDownAndModifySelection",
                    "pageUp:" => "MovePageUp",
                    "pageUpAndModifySelection:" => "MovePageUpAndModifySelection",
                    "showGuessPanel:" => "ToggleSpellPanel",
                    selector => selector.strip_suffix(':')?,
                };
                // Native insertion commands remain on keypress/text-input routing.
                if command
                    .get(..6)
                    .is_some_and(|prefix| prefix.eq_ignore_ascii_case("insert"))
                {
                    return None;
                }
                let id = EditingCommandType::FromCommandName(
                    &command.encode_utf16().collect::<Vec<_>>(),
                );
                Some(id)
            })
            .collect()
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = event;
        Vec::new()
    }
}
