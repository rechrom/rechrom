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
    document: &Document,
    old: UserInteractionState,
    new: UserInteractionState,
) -> bool {
    if old.focused_node_id != new.focused_node_id
        || old.focus_visible_node_id != new.focus_visible_node_id
        || old.multi_select_focused_option_id != new.multi_select_focused_option_id
    {
        return true;
    }
    // Element::SetHovered/SetActive schedules pseudo-state style invalidation,
    // then InvalidateIfHasEffectiveAppearance for native theme paint. Ordinary
    // HTML/SVG hover alone is not a blanket paint invalidation; StyleEngine
    // determines whether the resulting CSS diff actually needs Paint.
    let appearance = |id: Option<u64>| {
        let Some(id) = id else {
            return false;
        };
        let Some(mut index) = document.FindNodeById(id) else {
            return false;
        };
        loop {
            if let Some(resolved) = document.ResolvedStyleFor(index) {
                if unsafe { resolved.native_style.Get().as_ref() }
                    .is_some_and(|style| style.HasEffectiveAppearance())
                {
                    return true;
                }
            }
            let Some(parent) = document.FlatTreeParentElement(index) else {
                return false;
            };
            index = parent;
        }
    };
    (old.hovered_node_id != new.hovered_node_id
        && (appearance(old.hovered_node_id) || appearance(new.hovered_node_id)))
        || (old.pressed_node_id != new.pressed_node_id
            && (appearance(old.pressed_node_id) || appearance(new.pressed_node_id)))
}
