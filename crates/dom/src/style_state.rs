//! Document-owned style state. Mutation producers record changes here; the
//! style resolver consumes them only after a successful update.
#![allow(non_snake_case)]
use std::collections::HashSet;

// Attributes admitted to the selector-only invalidation path.  This is a DOM
// mutation classification: StyleEngine still proves whether any selector or
// attr() dependency observes the attribute before it skips ordinary style
// invalidation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum SelectorOnlyAttribute {
    DataPcr,
    ScriptSrc,
    ScriptNonce,
}

impl SelectorOnlyAttribute {
    pub const ALL: [Self; 3] = [Self::DataPcr, Self::ScriptSrc, Self::ScriptNonce];

    pub fn name(self) -> &'static str {
        match self {
            Self::DataPcr => "data-pcr",
            Self::ScriptSrc => "src",
            Self::ScriptNonce => "nonce",
        }
    }

    pub fn mask(self) -> u8 {
        match self {
            Self::DataPcr => 1,
            Self::ScriptSrc => 2,
            Self::ScriptNonce => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StyleChange {
    Node(usize),
    /// Only the inline declaration attribute changed. The resolver may keep
    /// selector dependencies local after proving no rule reads that attribute.
    InlineStyle(usize),
    /// A narrowly admitted attribute without HTML presentation effects. The
    /// resolver must prove absence of selector and attr() readers before skip.
    Attribute(usize, SelectorOnlyAttribute),
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

pub struct StyleState {
    pub sheet_revision: u64,
    pub parsed_style_elements: HashSet<u64>,
    pub dirty_style_elements: HashSet<u64>,
    // Media type, viewport width/height, resolution, preferred color scheme.
    pub environment: Option<(u8, Option<f64>, Option<f64>, Option<f64>, u8)>,
    pub all_dirty: bool,
    pub changes: HashSet<StyleChange>,
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
    /// Newly connected element subtrees and existing last/last-of-type roots
    /// whose structural pseudo state may have changed after an append.
    pub inserted_style_subtrees: HashSet<usize>,
    pub append_affected_subtrees: HashSet<usize>,
    pub impact: StyleUpdateImpact,
    pub stats: StyleUpdateStats,
    /// Geometry-query inputs changed during actual style resolution. Paint-only
    /// changes still retain their ordinary layout/paint impact.
    pub measurement_geometry_revision: u64,
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
            sibling_sensitive_nodes: Default::default(),
            descendant_sensitive_nodes: Default::default(),
            append_only_children: Default::default(),
            non_append_children: Default::default(),
            inserted_style_subtrees: Default::default(),
            append_affected_subtrees: Default::default(),
            impact: StyleUpdateImpact::TREE,
            stats: Default::default(),
            measurement_geometry_revision: 0,
        }
    }
}
