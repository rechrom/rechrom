//! Platform-neutral editing boundary values. Layout owns no DOM nodes: these
//! anchors retain the embedding's canonical node identity and UTF-16/child
//! offset, including neighbor anchors that must survive until Range conversion.
use std::marker::PhantomData;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextAffinity {
    #[default]
    Downstream,
    Upstream,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PositionAnchorType {
    #[default]
    OffsetInAnchor,
    BeforeAnchor,
    AfterAnchor,
    AfterChildren,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NodeTraversal;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FlatTreeTraversal;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EditingAlgorithm<Traversal>(PhantomData<Traversal>);
pub type EditingStrategy = EditingAlgorithm<NodeTraversal>;
pub type EditingInFlatTreeStrategy = EditingAlgorithm<FlatTreeTraversal>;

/// Source: core/editing/position.h. Null is an absent anchor, not node zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PositionTemplate<Strategy> {
    pub node_id: Option<u64>,
    pub offset: usize,
    pub anchor: PositionAnchorType,
    strategy: PhantomData<Strategy>,
}
impl<S> Default for PositionTemplate<S> {
    fn default() -> Self {
        Self {
            node_id: None,
            offset: 0,
            anchor: PositionAnchorType::OffsetInAnchor,
            strategy: PhantomData,
        }
    }
}
impl<S> PositionTemplate<S> {
    pub fn new(node_id: u64, offset: usize) -> Self {
        Self::WithAnchor(node_id, offset, PositionAnchorType::OffsetInAnchor)
    }
    pub fn WithAnchor(node_id: u64, offset: usize, anchor: PositionAnchorType) -> Self {
        Self {
            node_id: Some(node_id),
            offset,
            anchor,
            strategy: PhantomData,
        }
    }
    pub fn IsNull(&self) -> bool {
        self.node_id.is_none()
    }
}
pub type Position = PositionTemplate<EditingStrategy>;
pub type PositionInFlatTree = PositionTemplate<EditingInFlatTreeStrategy>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EphemeralRangeTemplate<S> {
    pub start: PositionTemplate<S>,
    pub end: PositionTemplate<S>,
}
pub type EphemeralRange = EphemeralRangeTemplate<EditingStrategy>;
pub type EphemeralRangeInFlatTree = EphemeralRangeTemplate<EditingInFlatTreeStrategy>;

#[derive(Clone, Copy, Debug, Default)]
pub struct PositionWithAffinityTemplate<S> {
    pub position: PositionTemplate<S>,
    pub affinity: TextAffinity,
}
// Source: core/editing/position_with_affinity.cc:36-42. Null positions do
// not acquire a different identity from an otherwise unused affinity value.
impl<S: PartialEq> PartialEq for PositionWithAffinityTemplate<S> {
    fn eq(&self, other: &Self) -> bool {
        if self.position.IsNull() {
            other.position.IsNull()
        } else {
            self.position == other.position && self.affinity == other.affinity
        }
    }
}
impl<S: Eq> Eq for PositionWithAffinityTemplate<S> {}
pub type PositionWithAffinity = PositionWithAffinityTemplate<EditingStrategy>;
pub type PositionInFlatTreeWithAffinity = PositionWithAffinityTemplate<EditingInFlatTreeStrategy>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SelectionTemplate<S> {
    pub anchor: PositionTemplate<S>,
    pub focus: PositionTemplate<S>,
    pub affinity: TextAffinity,
}
pub type SelectionInDomTree = SelectionTemplate<EditingStrategy>;
pub type SelectionInFlatTree = SelectionTemplate<EditingInFlatTreeStrategy>;

/// A canonical result, constructed only after the editing owner has performed
/// visible-position canonicalization against the current resident layout.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VisiblePositionTemplate<S> {
    pub position: PositionWithAffinityTemplate<S>,
}
pub type VisiblePosition = VisiblePositionTemplate<EditingStrategy>;
pub type VisiblePositionInFlatTree = VisiblePositionTemplate<EditingInFlatTreeStrategy>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VisibleSelectionTemplate<S> {
    pub anchor: VisiblePositionTemplate<S>,
    pub focus: VisiblePositionTemplate<S>,
}
pub type VisibleSelection = VisibleSelectionTemplate<EditingStrategy>;
pub type VisibleSelectionInFlatTree = VisibleSelectionTemplate<EditingInFlatTreeStrategy>;

/// DOM-owned facts needed by Layout's source position providers. The embedding
/// refreshes them with the same node mutation as text/style metadata.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EditingNodeFacts {
    /// Canonical DOM UserActionElements state used by snap-target selection.
    pub has_focus_within: bool,
    pub is_editable: bool,
    pub ignores_content: bool,
    /// IsDisplayInsideTable checks native HTMLTableElement identity, not CSS
    /// display:table (editing_utilities.cc:1260-1262).
    pub is_html_table: bool,
    /// DOM UTF-16 length or DOM child count, never rendered text length.
    pub last_offset: usize,
    /// Source TextControlElement::IsTextControl and Node::GetShadowRoot.
    pub is_text_control: bool,
    pub has_user_agent_shadow_root: bool,
    /// Present only when this node's canonical containing TreeScope is UA.
    pub user_agent_shadow_host_id: Option<u64>,
    /// Canonical DOM TreeScope identity; attachment parents may cross a shadow
    /// boundary while DOM Node::IsDescendantOf never crosses that boundary.
    pub tree_scope_id: u64,
}
/// Platform editing policy supplied by the embedding, not selected by Layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditingPositionOptions {
    pub bidi_caret_affinity: bool,
    pub prevent_text_selection_jump: bool,
    pub editing_line_movement_with_ruby_fix: bool,
    pub move_caret_to_horizontal_boundary_when_past_top_or_bottom: bool,
}
impl Default for EditingPositionOptions {
    fn default() -> Self {
        Self {
            bidi_caret_affinity: false,
            prevent_text_selection_jump: true,
            editing_line_movement_with_ruby_fix: true,
            move_caret_to_horizontal_boundary_when_past_top_or_bottom: true,
        }
    }
}

/// Position-local rendered-text facts used by the DOM-owned VisibleUnits walk.
/// Caret offsets are relative to this native text fragment's start.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditingTextFacts {
    pub start_offset: u32,
    pub caret_min_offset: usize,
    pub caret_max_offset: usize,
    pub has_non_collapsed_text: bool,
    pub contains_caret_offset: bool,
    pub is_before_non_collapsed_character: bool,
    pub is_after_non_collapsed_character: bool,
    pub visible: bool,
    pub blocked_by_display_lock: bool,
    pub writing_mode: foundation::WritingMode,
}

/// Facts from the node's own LayoutObject, as distinct from the associated
/// first-letter LayoutText selected by a position-local text query.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditingCaretFacts {
    pub direction: foundation::TextDirection,
    pub used_user_modify: foundation::EUserModify,
    pub visible: bool,
    pub blocked_by_display_lock: bool,
    pub writing_mode: foundation::WritingMode,
    pub is_text: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditingInlineAdjustmentFacts {
    pub is_text: bool,
    pub is_inline: bool,
    pub is_atomic_inline: bool,
    pub is_in_ng_inline_context: bool,
    pub is_block_flow: bool,
    pub is_layout_block: bool,
}
