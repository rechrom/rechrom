//! Owned, DOM-independent rule data retained between style updates.
use crate::{CSSDeclaration, CSSStyleSheet};
use std::{cell::RefCell, collections::BTreeMap, collections::HashMap, sync::Arc};

#[derive(Clone, Debug)]
pub struct ParsedSelector {
    pub compounds: Vec<String>,
    pub combinators: Vec<u8>,
    pub valid: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PseudoTarget {
    Element,
    Before,
    After,
    FirstLetter,
    Placeholder,
    Invalid,
}

impl PseudoTarget {
    pub fn index(self) -> Option<usize> {
        match self {
            Self::Element => Some(0),
            Self::Before => Some(1),
            Self::After => Some(2),
            Self::FirstLetter => Some(3),
            Self::Placeholder => Some(4),
            Self::Invalid => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CompiledSelector {
    pub selector: ParsedSelector,
    pub target: PseudoTarget,
}

// Positions are valid for the rule-set revision. Declarations stay in CSSOM;
// the cache owns only parsed selectors and indexes, without copying rule text.
pub struct CompiledStyleRule {
    pub sheet_index: usize,
    pub rule_index: usize,
    pub selectors: Arc<[CompiledSelector]>,
    pub source_order: usize,
    /// At least one admitted selector in this rule reads reverse sibling
    /// position. Used by mutation invalidation to filter target candidates.
    pub depends_on_nth_last_children: bool,
    /// Appending a sibling can change an existing element's last/only state.
    /// Target candidates for these rules are invalidated without treating the
    /// previous last element as a whole dirty subtree.
    pub depends_on_last_children: bool,
}

#[derive(Default)]
pub struct CompiledRuleIndex {
    pub universal: Vec<usize>,
    pub ids: BTreeMap<String, Vec<usize>>,
    pub classes: BTreeMap<String, Vec<usize>>,
    pub types: BTreeMap<String, Vec<usize>>,
}

// Only attributes without presentational semantics on the admitted HTML
// owners may use this path. All other mutations remain ordinary Node changes.
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

#[derive(Default)]
pub struct CompiledRuleSet {
    pub rules: Vec<CompiledStyleRule>,
    // Selector targets are independent candidate sets. A universal element
    // rule must not be revisited for each generated pseudo-element.
    pub targets: [CompiledRuleIndex; 5],
    pub layers: BTreeMap<String, usize>,
    pub declaration_count: usize,
    pub depends_on_siblings: bool,
    // Compounds whose state can affect a following `+`/`~` selector target.
    // Attribute mutations use these as invalidation gates before widening to
    // the parent subtree.
    pub sibling_invalidation_hosts: Vec<String>,
    // Compounds on the ancestor side of a descendant/child combinator. An
    // attribute mutation only dirties descendants when the element matched
    // one of these compounds before or after the mutation.
    pub descendant_invalidation_hosts: Vec<String>,
    /// Reverse-position selectors can change every existing child's match when
    /// a child is appended. Last/only selectors only affect the previous last
    /// child and are handled by the DOM mutation record.
    pub depends_on_nth_last_children: bool,
    /// Necessary simple-selector gates for the element carrying an
    /// `:nth-last-*` pseudo.  Mutation invalidation uses these to mark only
    /// parents whose existing children could actually be affected.
    pub nth_last_child_hosts: Vec<String>,
    /// Some reverse-position selector could not be reduced to a safe host
    /// gate, so affected parents must retain the conservative fallback.
    pub nth_last_child_host_fallback: bool,
    pub depends_on_descendants: bool,
    // Necessary-condition gates for proven first-compound descendant :has.
    pub bounded_has_hosts: Vec<String>,
    pub has_dependency_fallback: bool,
    // Necessary-condition gates for :empty on a changed child container.
    // Unknown child-state grammar retains sibling-parent invalidation.
    pub empty_container_hosts: Vec<String>,
    // Unknown child-state syntax with a proven, immutable tag/class/id prefix.
    // The conservative container fallback applies only to matching hosts.
    pub children_container_fallback_hosts: Vec<String>,
    pub children_container_dependency_fallback: bool,
    // Selector readers require subtree invalidation; declaration-only readers
    // require local cascade plus computed inheritance, including a new node.
    pub attribute_selector_dependencies: u8,
    // Selector + declaration/custom-property attr() readers. Unknown syntax
    // sets all three bits, retaining the appropriate cascade fallback.
    pub selector_only_attribute_dependencies: u8,
    // Includes [style] at any nesting depth and conservatively unknown syntax.
    // False proves an inline-declaration write cannot change selector matching.
    pub inline_style_may_affect_selectors: bool,
}

/// Document-owned media results. Rule Arcs retain identity as well as storage,
/// so an old allocation address cannot be reused to produce a cache hit.
pub struct MediaRuleCache {
    // media type, viewport width/height, resolution and color preference.
    pub environment: (u8, Option<f64>, Option<f64>, Option<f64>, u8),
    pub sheet_revision: u64,
    pub user_agent_rules: Arc<CompiledRuleSet>,
    pub author_rules: Arc<CompiledRuleSet>,
    pub user_agent_matches: Arc<Vec<Vec<bool>>>,
    pub author_matches: Arc<Vec<Vec<bool>>>,
}

#[derive(Default)]
pub struct StyleRuleCache {
    pub user_agent_source: Vec<CSSStyleSheet>,
    pub user_agent: Option<Arc<CompiledRuleSet>>,
    pub author_revision: Option<u64>,
    pub author: Option<Arc<CompiledRuleSet>>,
    pub media: RefCell<Option<MediaRuleCache>>,
    pub selectors: RefCell<HashMap<String, Arc<[CompiledSelector]>>>,
    // DOM queries retain only syntax, never matching results or live DOM state.
    pub query_selectors: RefCell<HashMap<String, Arc<[CompiledSelector]>>>,
    pub inline: RefCell<HashMap<u64, (String, Arc<[CSSDeclaration]>)>>,
}
