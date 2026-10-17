//! Document-owned style state. Mutation producers record changes here; the
//! style resolver consumes them only after a successful update.
#![allow(non_snake_case)]
use std::collections::{HashMap, HashSet};

/// ContainerNode::ChildrenChange's element boundaries. Unknown boundaries
/// (for example a batched slot reassignment) retain conservative invalidation.
#[derive(Clone, Copy, Debug)]
pub struct ChildrenChangeBoundary {
    pub before: Option<usize>,
    pub after: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ControlPseudoState {
    Default,
    HasDatalist,
    Indeterminate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StyleChange {
    Node(usize),
    /// Attribute mutation with old/new selector keys retained below.
    AttributeValue(usize),
    /// TextControlElement::UpdatePlaceholderVisibility changed its pseudo state.
    PlaceholderShown(usize),
    /// Element::UpdateDirectionalityAndDescendant changed the cached direction.
    Directionality(usize),
    /// Form association/group observers notify a specific pseudo, without
    /// changing this control's attributes or every sibling selector input.
    ControlPseudo(usize, ControlPseudoState),
    /// Only the inline declaration attribute changed. The resolver may keep
    /// selector dependencies local after proving no rule reads that attribute.
    InlineStyle(usize),
    /// The retained exportparts NamesMap changed. ::part invalidation starts
    /// at this host's attached ShadowRoot and reaches all exposed descendants.
    ExportParts(usize),
    Children(usize),
    Animation(usize),
    /// Resolved image payload changed; DOM and selector inputs are unchanged.
    Resource(usize),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StyleUpdateImpact {
    pub reattach: bool,
    pub layout: bool,
    pub paint: bool,
}

impl StyleUpdateImpact {
    pub fn Merge(&mut self, other: Self) {
        self.reattach |= other.reattach;
        self.layout |= other.layout;
        self.paint |= other.paint;
    }
    pub fn IsEmpty(self) -> bool {
        !self.reattach && !self.layout && !self.paint
    }
    pub const LAYOUT: Self = Self {
        reattach: false,
        layout: true,
        paint: true,
    };
    pub const TREE: Self = Self {
        reattach: true,
        layout: true,
        paint: true,
    };
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StyleUpdateStats {
    pub resolved_nodes: usize,
    pub changed_nodes: usize,
    pub rule_sets_built: usize,
}

/// DOM mutation facts consumed by compiled StyleEngine invalidation sets.
#[derive(Default)]
pub struct AttributeStyleChange {
    pub names: HashSet<String>,
    pub classes: HashSet<String>,
    pub ids: HashSet<String>,
}

pub struct StyleState {
    pub sheet_revision: u64,
    pub parsed_style_elements: HashSet<u64>,
    pub dirty_style_elements: HashSet<u64>,
    // Media type, viewport width/height, resolution, preferred color scheme.
    pub environment: Option<(u8, Option<f64>, Option<f64>, Option<f64>, u8)>,
    pub all_dirty: bool,
    pub changes: HashSet<StyleChange>,
    /// CSSOM's Node::kInlineIndependentStyleChange fact. The resolver combines
    /// it with stronger pending invalidations and inheritance propagation.
    pub independent_inline_styles: HashSet<usize>,
    pub attribute_changes: HashMap<usize, AttributeStyleChange>,
    /// Nodes whose old or new state can match the left side of a sibling
    /// combinator. Kept separately so StyleChange remains compact and deduped.
    pub sibling_sensitive_nodes: HashSet<usize>,
    /// Nodes whose old or new selector state can affect a descendant target.
    /// This is the small equivalent of Blink's descendant invalidation sets:
    /// ordinary local attribute changes do not dirty the whole subtree.
    pub descendant_sensitive_nodes: HashSet<usize>,
    /// Child-list changes that only appended at the end during this update.
    /// This lets style invalidation resolve the new subtree without revisiting
    /// every existing sibling. A remove or insert-before clears this proof.
    pub append_only_children: HashSet<usize>,
    pub non_append_children: HashSet<usize>,
    pub child_change_boundaries: HashMap<usize, Option<Vec<ChildrenChangeBoundary>>>,
    /// Newly connected element subtrees and existing last/last-of-type roots
    /// whose structural pseudo state may have changed after an append.
    pub inserted_style_subtrees: HashSet<usize>,
    pub append_affected_subtrees: HashSet<usize>,
    /// Unknown/document-wide native input changes require full tree attachment.
    pub layout_inputs_require_tree: bool,
    /// Known dirty attachment roots. An unknown or document-wide input still
    /// sets layout_inputs_require_tree; these identities never replace DOM.
    pub layout_tree_roots: HashSet<u64>,
    /// Style's NeedsReattachLayoutTree result for individual native objects.
    pub layout_reattach_nodes: HashSet<u64>,
    pub impact: StyleUpdateImpact,
    pub stats: StyleUpdateStats,
    /// Style inputs not yet delivered to resident LayoutObjects. A synchronous
    /// layout consumes this delta independently of the frame's paint work.
    pub layout_style_nodes: HashSet<u64>,
    /// Style clients whose fragments/paint still need updating this frame.
    /// Cleared by Paint commit, not by synchronous layout or a second recalc.
    pub paint_style_nodes: HashSet<u64>,
    /// Generated/flattened paint input changes need ordinary fragment export.
    pub paint_style_requires_export: bool,
    /// Geometry-query inputs changed during native style resolution or DOM
    /// flat-tree attachment or rendered character data. Equal-style
    /// redistribution still revokes geometry;
    /// paint-only changes retain their ordinary layout/paint impact.
    pub measurement_geometry_revision: u64,
    /// Attribute metadata awaiting validation by the resident LayoutEngine.
    /// Acknowledging query geometry does not consume the frame's paint impact.
    pub geometry_input_nodes: HashSet<u64>,
}

impl Default for StyleState {
    fn default() -> Self {
        Self {
            sheet_revision: 0,
            environment: None,
            parsed_style_elements: Default::default(),
            dirty_style_elements: Default::default(),
            all_dirty: true,
            changes: Default::default(),
            independent_inline_styles: Default::default(),
            attribute_changes: Default::default(),
            sibling_sensitive_nodes: Default::default(),
            descendant_sensitive_nodes: Default::default(),
            append_only_children: Default::default(),
            non_append_children: Default::default(),
            child_change_boundaries: Default::default(),
            inserted_style_subtrees: Default::default(),
            append_affected_subtrees: Default::default(),
            layout_inputs_require_tree: true,
            layout_tree_roots: Default::default(),
            layout_reattach_nodes: Default::default(),
            impact: StyleUpdateImpact::TREE,
            stats: Default::default(),
            layout_style_nodes: Default::default(),
            paint_style_nodes: Default::default(),
            paint_style_requires_export: true,
            measurement_geometry_revision: 0,
            geometry_input_nodes: Default::default(),
        }
    }
}
