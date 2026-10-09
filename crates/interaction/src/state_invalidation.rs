//! Paint admission for changes to user interaction state.

use dom::{Document, UserInteractionState};

pub fn ValidateInteractionState(document: &Document, state: UserInteractionState) -> bool {
    [
        state.focused_node_id,
        state.focus_visible_node_id,
        state.hovered_node_id,
        state.pressed_node_id,
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
    {
        return true;
    }
    let appearance_none =
        layoutng_assembly::fragment_tree::FormControlPaintData::default().appearance;
    for (previous, current) in [
        (old.hovered_node_id, new.hovered_node_id),
        (old.pressed_node_id, new.pressed_node_id),
    ] {
        if previous == current {
            continue;
        }
        for id in [previous, current].into_iter().flatten() {
            if Some(id) == old.focused_node_id || Some(id) == old.focus_visible_node_id {
                return true;
            }
            let Some(index) = document.FindNodeById(id) else {
                return true;
            };
            let Some(resolved) = document.ResolvedStyleFor(index) else {
                return true;
            };
            if std::iter::once(&resolved.style)
                .chain(
                    [
                        resolved.before.as_ref(),
                        resolved.after.as_ref(),
                        resolved.first_letter.as_ref(),
                        resolved.placeholder.as_ref(),
                    ]
                    .into_iter()
                    .flatten()
                    .map(|pseudo| &pseudo.style),
                )
                .any(|style| {
                    style
                        .extended
                        .as_ref()
                        .is_some_and(|extended| extended.effective_appearance != appearance_none)
                })
            {
                return true;
            }
        }
    }
    // Selector matching does not currently consume interaction state. The
    // remaining hover/pressed state only affects native control appearance.
    false
}
