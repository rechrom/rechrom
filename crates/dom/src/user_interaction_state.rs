// cpp: dom/user_interaction_state.h:13-22
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UserInteractionState {
    pub focused_node_id: Option<u64>,
    pub focus_visible_node_id: Option<u64>,
    pub hovered_node_id: Option<u64>,
    pub pressed_node_id: Option<u64>,
}
