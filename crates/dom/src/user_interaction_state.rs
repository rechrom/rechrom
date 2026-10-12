// cpp: dom/user_interaction_state.h:13-22
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UserInteractionState {
    pub focused_node_id: Option<u64>,
    pub focus_visible_node_id: Option<u64>,
    pub hovered_node_id: Option<u64>,
    pub pressed_node_id: Option<u64>,
    /// cpp: ListBoxSelectType::active_selection_end_ together with
    /// is_in_non_contiguous_selection_. This is present only while a multiple
    /// listbox is moving its active end without changing selectedness.
    pub multi_select_focused_option_id: Option<u64>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InteractionStateMutation {
    pub state: UserInteractionState,
}
