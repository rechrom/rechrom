//! Page-facing facade for the staged Chromium interaction translation.
//!
//! Page retains the existing source state owners and binds translated native
//! EventHandler/managers for each synchronous raw-input transaction.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub mod adapter;
pub mod core;

pub use adapter::dom_event::{DomEventListenerQuery, DomEventPaths, DomEventTargets};
pub use adapter::interaction::{
    DocumentEditorCommandOperation, DocumentEditorCommandResult, DOMSelectionSnapshot,
    FormDataCreation, PointerCaptureError, TextControlSelectionSnapshot,
};
pub use adapter::interaction_contract::{
    FormNavigationPreparation, FormSubmissionParameters, FormSubmissionScheduling,
    InteractionDOMMutationEmitter, InteractionDocument, InteractionEffect, InteractionOutput,
    InteractionOutputEmitter, InteractionPseudoState, InteractionResult, InteractionState,
    InteractionStateDocument, InteractionStateDocumentWithAppearance, InteractionStateMutation,
    InteractionStateNeedsPaint, PreparedFormSubmission, ValidateInteractionState,
};
pub use adapter::native_interaction::{
    Interaction as NativeInteraction, InteractionAssembly, InteractionDOMAssembly,
    InteractionDOMDispatch, InteractionDOMEvents, InteractionError, InteractionFocusInput,
    InteractionFormSubmission, InteractionHost, InteractionInputCompletion, InteractionInputMethod,
    InteractionInputResult, InteractionPort, InteractionStateOwner, InteractionWheelInput,
};
pub use adapter::page_interaction::Interaction;
pub use adapter::page_main_thread_scroll::{WheelDOMAcknowledgement, WheelDOMAcknowledgements};
pub use adapter::page_wheel_event_regions::{BlockingWheelEventRegions, WheelEventRegionResolver};
pub use adapter::{frame_aligned_input_queue, input_event};
pub use adapter::{
    page_event as event, page_event_dispatcher as event_dispatcher, page_event_path as event_path,
    page_input_type as input_type,
};
pub(crate) use adapter::{
    page_default_event_handler as default_event_handler,
    page_editing_owners as editing_owners,
    page_focus_controller as focus_controller,
};

pub mod ownership {
    pub use crate::adapter::interaction_contract::{
        Attribute, HasAttribute, InteractionDOMMutationEmitter, InteractionDocument, IsElement,
        ReadNode,
    };
}
pub use interaction_default as default;
pub use interaction_dispatch as dispatch;
pub use interaction_hit_test as hit_test;
pub use interaction_input as input;
pub use interaction_scroll as scroll;

/// Canonical selected cursor values; native cursor construction belongs to the host.
pub mod cursor {
    pub use crate::core::input::event_handler_cursor::{Cursor, CursorBitmap, CursorType};
}
pub use cursor::{Cursor, CursorType};

pub use adapter::page_keyboard_event_manager::{
    CreatePageKeyboardEventManager, PageKeyboardDocument, PageKeyboardDocumentServices,
    PageKeyboardKeycodeConverter, PageKeyboardScrollNodes, PageKeyboardScrollNodesForPage,
};
