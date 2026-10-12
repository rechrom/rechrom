//! Paint admission for changes to user interaction state.

use dom::{Document, UserInteractionState};

pub fn ValidateInteractionState(document: &Document, state: UserInteractionState) -> bool {
    [
        state.focused_node_id,
        state.focus_visible_node_id,
        state.hovered_node_id,
        state.pressed_node_id,
        state.multi_select_focused_option_id,
    ]
    .into_iter()
    .flatten()
    .all(|id| document.FindNodeById(id).is_some())
}

pub fn InteractionStateNeedsPaint(
    _document: &Document,
    old: UserInteractionState,
    new: UserInteractionState,
) -> bool {
    if old.focused_node_id != new.focused_node_id
        || old.focus_visible_node_id != new.focus_visible_node_id
        || old.multi_select_focused_option_id != new.multi_select_focused_option_id
        || old.hovered_node_id != new.hovered_node_id
        || old.pressed_node_id != new.pressed_node_id
    {
        return true;
    }
    false
}
