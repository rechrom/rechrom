//! HTMLInputElement -> PasswordInputType caps-lock notification.
//! Source: html_input_element.cc:2503-2505; password_input_type.cc:127-150.
use super::*;
use interaction_default::core::html::forms::password_input_type::{self, PasswordCapsLockContext};

struct Context<'scope> {
    document: &'scope InteractionDocument,
    host: &'scope dyn super::super::page_frame_selection::PageFrameSelectionHost,
    frame_focused_and_active: bool,
    node: u64,
}
impl PasswordCapsLockContext for Context<'_> {
    type Error = &'static str;
    fn FrameIsFocusedAndActive(&self) -> bool {
        self.frame_focused_and_active
    }
    fn IsFocusedElement(&self) -> bool {
        ReadNode(self.document, self.node, |document, _, node| {
            document.IsElementFocused(node)
        })
        .unwrap_or(false)
    }
    fn CurrentCapsLockState(&self) -> Result<bool, Self::Error> {
        struct Platform<'a>(&'a dyn super::super::page_frame_selection::PageFrameSelectionHost);
        impl crate::core::input::keyboard_event_manager::KeyboardPlatformModifiers for Platform<'_> {
            fn GetCurrentKeyModifiers(
                &self,
            ) -> Result<
                crate::core::input::keyboard_event_manager::NativeKeyboardModifiers,
                &'static str,
            > {
                self.0.CurrentKeyboardModifiers()
            }
        }
        KeyboardEventManager::CurrentCapsLockState(&Platform(self.host))
    }
    fn ShouldDrawCapsLockIndicator(&self) -> bool {
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        document
            .FindNodeById(self.node)
            .is_some_and(|node| document.InputShouldDrawCapsLockIndicator(node))
    }
    fn SetShouldDrawCapsLockIndicator(&self, value: bool) {
        let mut owner = self.document.borrow_mut();
        let document = owner.GetDocumentMut();
        if let Some(node) = document.FindNodeById(self.node) {
            document.SetInputShouldDrawCapsLockIndicator(node, value);
        }
    }
    fn SetShouldDoFullPaintInvalidation(&self) {
        self.host.ApplyEffect(super::super::page_frame_selection::FrameSelectionEffect::InvalidateInputCapsLockIndicatorPaint(self.node));
    }
}

pub(super) fn CapsLockStateMayHaveChanged(
    dispatcher: &EventDispatcher<'_, '_>,
    target: &EventTargetRef,
) -> Result<(), &'static str> {
    let targets = dispatcher.KeyboardTargets();
    if targets.NodeId(target) != dispatcher.KeyboardState().borrow().focused_node_id {
        return Ok(());
    }
    dispatcher
        .FrameSelection()
        .NotifyCapsLockStateMayHaveChanged()
}

/// Shared by KeyboardEventManager and FrameSelection focus/activation changes.
/// All decisions stay in the translated PasswordInputType source function.
pub fn UpdatePasswordCapsLockIndicator(
    document: &InteractionDocument,
    focused: Option<u64>,
    frame_focused_and_active: bool,
    host: &dyn super::super::page_frame_selection::PageFrameSelectionHost,
) -> Result<(), &'static str> {
    let Some(node) = focused else {
        return Ok(());
    };
    let is_password = ReadNode(document, node, |document, _, node| {
        document.InputType(node) == Some(dom::DOMInputType::kPassword)
    })
    .unwrap_or(false);
    if !is_password {
        return Ok(());
    }
    password_input_type::CapsLockStateMayHaveChanged(&Context {
        document,
        host,
        frame_focused_and_active,
        node,
    })
}
