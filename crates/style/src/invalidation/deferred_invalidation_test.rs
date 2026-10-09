// Behavioral tests for the mapped PendingInvalidations/StyleInvalidator chain.
// Fixture DOM methods store actual flags and relations; no matching/traversal
// policy is delegated to the fixture.
use super::invalidation_dom::{InvalidationDom, StyleChangeType};
use super::invalidation_set::*;
use super::pending_invalidations::PendingInvalidations;
use super::style_invalidator::StyleInvalidator;
use foundation::AtomicString;

struct Element {
    tag: AtomicString,
    id: Option<AtomicString>,
    classes: Vec<AtomicString>,
    pseudo: AtomicString,
    attributes: Vec<AtomicString>,
    part: bool,
    computed: bool,
    sibling_functions: bool,
    pseudo_sibling_functions: bool,
    query_tree_counting: bool,
}
impl Default for Element {
    fn default() -> Self {
        Self {
            tag: AtomicString::from_str("div"),
            id: None,
            classes: Vec::new(),
            pseudo: AtomicString::default(),
            attributes: Vec::new(),
            part: false,
            computed: true,
            sibling_functions: false,
            pseudo_sibling_functions: false,
            query_tree_counting: false,
        }
    }
}
impl InvalidationElement for Element {
    fn LocalNameForSelectorMatching(&self) -> &AtomicString {
        &self.tag
    }
    fn IdForStyleResolution(&self) -> Option<&AtomicString> {
        self.id.as_ref()
    }
    fn ClassNames(&self) -> &[AtomicString] {
        &self.classes
    }
    fn ShadowPseudoId(&self) -> &AtomicString {
        &self.pseudo
    }
    fn HasAttributes(&self) -> bool {
        !self.attributes.is_empty()
    }
    fn HasAttributeIgnoringNamespace(&self, name: &AtomicString) -> bool {
        self.attributes.contains(name)
    }
    fn HasPart(&self) -> bool {
        self.part
    }
    fn HasComputedStyle(&self) -> bool {
        self.computed
    }
    fn HasSiblingFunctions(&self) -> bool {
        self.sibling_functions
    }
    fn PseudoElementStylesDependOnSiblingFunctions(&self) -> bool {
        self.pseudo_sibling_functions
    }
    fn ContainerQueryDependsOnTreeCounting(&self) -> bool {
        self.query_tree_counting
    }
}
#[derive(Default)]
struct Node {
    element: Option<Element>,
    parent: Option<usize>,
    children: Vec<usize>,
    host: Option<usize>,
    shadow: Option<usize>,
    slot: bool,
    assigned: Vec<usize>,
    change: StyleChangeType,
    invalidation: bool,
    child_invalidation: bool,
}
struct Dom {
    nodes: Vec<Node>,
    nth_calls: Vec<usize>,
}
impl Dom {
    fn new() -> Self {
        Self {
            nodes: vec![Node::default()],
            nth_calls: Vec::new(),
        }
    }
    fn add(&mut self, parent: usize, classes: &[&str]) -> usize {
        let n = self.nodes.len();
        self.nodes.push(Node {
            element: Some(Element {
                classes: classes.iter().map(|s| AtomicString::from_str(s)).collect(),
                ..Element::default()
            }),
            parent: Some(parent),
            ..Node::default()
        });
        self.nodes[parent].children.push(n);
        n
    }
    fn shadow(&mut self, host: usize) -> usize {
        let n = self.nodes.len();
        self.nodes.push(Node {
            host: Some(host),
            ..Node::default()
        });
        self.nodes[host].shadow = Some(n);
        n
    }
    fn run(&mut self, pending: &mut PendingInvalidations<usize>, root: usize) {
        StyleInvalidator::new(pending.GetPendingInvalidationMap()).Invalidate(&0, Some(root), self);
    }
    fn next(&self, node: usize) -> Option<usize> {
        let parent = self.nodes[node].parent?;
        let siblings = &self.nodes[parent].children;
        siblings
            .get(siblings.iter().position(|n| *n == node)? + 1)
            .copied()
    }
}
impl InvalidationDom for Dom {
    type Node = usize;
    type Element = Element;
    fn element(&self, node: &usize) -> Option<&Element> {
        self.nodes[*node].element.as_ref()
    }
    fn is_document(&self, node: &usize) -> bool {
        *node == 0
    }
    fn shadow_host(&self, node: &usize) -> Option<usize> {
        self.nodes[*node].host
    }
    fn parent_node(&self, node: &usize) -> Option<usize> {
        self.nodes[*node].parent
    }
    fn parent_or_shadow_host_node(&self, node: &usize) -> Option<usize> {
        self.nodes[*node].parent.or(self.nodes[*node].host)
    }
    fn next_sibling(&self, node: &usize) -> Option<usize> {
        self.next(*node)
    }
    fn next_element_sibling(&self, node: &usize) -> Option<usize> {
        let mut next = self.next(*node);
        while let Some(n) = next {
            if self.element(&n).is_some() {
                return Some(n);
            }
            next = self.next(n);
        }
        None
    }
    fn element_children(&self, node: &usize) -> Vec<usize> {
        self.nodes[*node]
            .children
            .iter()
            .copied()
            .filter(|n| self.element(n).is_some())
            .collect()
    }
    fn shadow_root(&self, node: &usize) -> Option<usize> {
        self.nodes[*node].shadow
    }
    fn document_element(&self, _: &usize) -> Option<usize> {
        self.nodes[0].children.first().copied()
    }
    fn style_change_type(&self, node: &usize) -> StyleChangeType {
        self.nodes[*node].change
    }
    fn set_needs_style_recalc(&mut self, node: &usize, change: StyleChangeType) {
        self.nodes[*node].change = self.nodes[*node].change.max(change);
    }
    fn needs_style_recalc(&self, node: &usize) -> bool {
        self.nodes[*node].change != StyleChangeType::NoStyleChange
    }
    fn needs_style_invalidation(&self, node: &usize) -> bool {
        self.nodes[*node].invalidation
    }
    fn child_needs_style_invalidation(&self, node: &usize) -> bool {
        self.nodes[*node].child_invalidation
    }
    fn set_needs_style_invalidation(&mut self, node: &usize) {
        self.nodes[*node].invalidation = true;
        let mut ancestor = self.parent_or_shadow_host_node(node);
        while let Some(n) = ancestor {
            self.nodes[n].child_invalidation = true;
            ancestor = self.parent_or_shadow_host_node(&n);
        }
    }
    fn clear_needs_style_invalidation(&mut self, node: &usize) {
        self.nodes[*node].invalidation = false;
    }
    fn clear_child_needs_style_invalidation(&mut self, node: &usize) {
        self.nodes[*node].child_invalidation = false;
    }
    fn is_html_slot(&self, node: &usize) -> bool {
        self.nodes[*node].slot
    }
    fn flattened_assigned_nodes(&self, node: &usize) -> Vec<usize> {
        self.nodes[*node].assigned.clone()
    }
    fn possibly_schedule_nth_pseudo_invalidations(&mut self, node: &usize) {
        self.nth_calls.push(*node);
    }
}
fn descendants(class: &str) -> InvalidationSetRef {
    let set = DescendantInvalidationSet::Create();
    set.borrow_mut().AddClass(&AtomicString::from_str(class));
    set
}
fn schedule(
    pending: &mut PendingInvalidations<usize>,
    dom: &mut Dom,
    node: usize,
    set: InvalidationSetRef,
) {
    pending.ScheduleInvalidationSetsForNode(
        &InvalidationLists {
            descendants: vec![set],
            siblings: Vec::new(),
        },
        &node,
        dom,
    );
}

#[test]
fn document_schedule_matches_only_descendants_and_clears_all_flags() {
    let mut dom = Dom::new();
    let root = dom.add(0, &[]);
    let yes = dom.add(root, &["b"]);
    let no = dom.add(root, &["c"]);
    let mut pending = PendingInvalidations::default();
    schedule(&mut pending, &mut dom, 0, descendants("b"));
    dom.run(&mut pending, root);
    assert_eq!(dom.nodes[yes].change, StyleChangeType::LocalStyleChange);
    assert_eq!(dom.nodes[no].change, StyleChangeType::NoStyleChange);
    assert_eq!(dom.nodes[0].change, StyleChangeType::NoStyleChange);
    assert!(pending.GetPendingInvalidationMap().is_empty());
    assert!(dom
        .nodes
        .iter()
        .all(|n| !n.invalidation && !n.child_invalidation));
}

#[test]
fn no_map_entry_for_sibling_set_without_next_sibling() {
    let mut dom = Dom::new();
    let root = dom.add(0, &[]);
    let last = dom.add(root, &[]);
    let mut pending = PendingInvalidations::default();
    pending.ScheduleInvalidationSetsForNode(
        &InvalidationLists {
            descendants: Vec::new(),
            siblings: vec![SiblingInvalidationSet::Create(None)],
        },
        &last,
        &mut dom,
    );
    assert!(pending.GetPendingInvalidationMap().is_empty());
    assert!(!dom.nodes[last].invalidation);
}

#[test]
fn display_none_does_not_schedule_descendants_or_leak_nth_to_other_children() {
    let mut dom = Dom::new();
    let root = dom.add(0, &[]);
    let none = dom.add(root, &[]);
    dom.nodes[none].element.as_mut().unwrap().computed = false;
    let hidden = dom.add(none, &["b"]);
    let other = dom.add(root, &[]);
    let visible = dom.add(other, &["b"]);
    let mut pending = PendingInvalidations::default();
    schedule(&mut pending, &mut dom, none, descendants("b"));
    assert!(pending.GetPendingInvalidationMap().is_empty());
    let nth = NthSiblingInvalidationSet::Create();
    nth.borrow_mut().AddClass(&AtomicString::from_str("b"));
    nth.borrow_mut().SetInvalidatesSelf();
    pending.ScheduleInvalidationSetsForNode(
        &InvalidationLists {
            descendants: Vec::new(),
            siblings: vec![nth],
        },
        &none,
        &mut dom,
    );
    schedule(&mut pending, &mut dom, other, descendants("b"));
    dom.run(&mut pending, root);
    assert_eq!(dom.nodes[hidden].change, StyleChangeType::NoStyleChange);
    assert_eq!(dom.nodes[other].change, StyleChangeType::NoStyleChange);
    assert_eq!(dom.nodes[visible].change, StyleChangeType::LocalStyleChange);
}

#[test]
fn direct_adjacent_limit_and_sibling_descendants_restore_at_recursion_boundary() {
    let mut dom = Dom::new();
    let root = dom.add(0, &[]);
    let trigger = dom.add(root, &[]);
    let first = dom.add(root, &["b"]);
    let nested = dom.add(first, &["c"]);
    let second = dom.add(root, &["b"]);
    let outside = dom.add(second, &["c"]);
    let sibling = SiblingInvalidationSet::Create(None);
    sibling.borrow_mut().AddClass(&AtomicString::from_str("b"));
    sibling.borrow_mut().SetInvalidatesSelf();
    sibling
        .borrow_mut()
        .EnsureSiblingDescendants()
        .AddClass(&AtomicString::from_str("c"));
    let mut pending = PendingInvalidations::default();
    pending.ScheduleInvalidationSetsForNode(
        &InvalidationLists {
            descendants: Vec::new(),
            siblings: vec![sibling],
        },
        &trigger,
        &mut dom,
    );
    dom.run(&mut pending, root);
    assert_eq!(dom.nodes[first].change, StyleChangeType::LocalStyleChange);
    assert_eq!(dom.nodes[nested].change, StyleChangeType::LocalStyleChange);
    assert_eq!(dom.nodes[second].change, StyleChangeType::NoStyleChange);
    assert_eq!(dom.nodes[outside].change, StyleChangeType::NoStyleChange);
}

#[test]
fn nth_sets_start_on_parent_and_target_all_matching_children() {
    let mut dom = Dom::new();
    let root = dom.add(0, &[]);
    let a = dom.add(root, &["b"]);
    let b = dom.add(root, &[]);
    let c = dom.add(root, &["b"]);
    let nth = NthSiblingInvalidationSet::Create();
    nth.borrow_mut().AddClass(&AtomicString::from_str("b"));
    nth.borrow_mut().SetInvalidatesSelf();
    let mut pending = PendingInvalidations::default();
    pending.ScheduleInvalidationSetsForNode(
        &InvalidationLists {
            descendants: Vec::new(),
            siblings: vec![nth],
        },
        &root,
        &mut dom,
    );
    dom.run(&mut pending, root);
    assert_eq!(dom.nodes[a].change, StyleChangeType::LocalStyleChange);
    assert_eq!(dom.nodes[b].change, StyleChangeType::NoStyleChange);
    assert_eq!(dom.nodes[c].change, StyleChangeType::LocalStyleChange);
}

#[test]
fn shadow_whole_subtree_invalidates_host_and_parts_cross_boundary() {
    let mut dom = Dom::new();
    let root = dom.add(0, &[]);
    let shadow = dom.shadow(root);
    let part = dom.add(shadow, &[]);
    dom.nodes[part].element.as_mut().unwrap().part = true;
    let mut pending = PendingInvalidations::default();
    schedule(
        &mut pending,
        &mut dom,
        root,
        InvalidationSet::PartInvalidationSet(),
    );
    dom.run(&mut pending, root);
    assert_eq!(dom.nodes[part].change, StyleChangeType::LocalStyleChange);
    let whole = DescendantInvalidationSet::Create();
    whole.borrow_mut().SetWholeSubtreeInvalid();
    schedule(&mut pending, &mut dom, shadow, whole);
    assert_eq!(dom.nodes[root].change, StyleChangeType::SubtreeStyleChange);
}

#[test]
fn slotted_sets_match_assigned_elements_and_skip_already_dirty_or_text_nodes() {
    let mut dom = Dom::new();
    let root = dom.add(0, &[]);
    let slot = dom.add(root, &[]);
    let outside = dom.add(0, &[]);
    let assigned = dom.add(outside, &["b"]);
    let dirty = dom.add(outside, &["b"]);
    let nonmatching = dom.add(outside, &["c"]);
    dom.nodes[dirty].change = StyleChangeType::SubtreeStyleChange;
    dom.nodes[slot].slot = true;
    dom.nodes[slot].assigned = vec![assigned, dirty, nonmatching, 0];
    let set = descendants("b");
    set.borrow_mut().SetInvalidatesSlotted();
    let mut pending = PendingInvalidations::default();
    schedule(&mut pending, &mut dom, root, set);
    schedule(&mut pending, &mut dom, root, descendants("c"));
    dom.run(&mut pending, root);
    assert_eq!(
        dom.nodes[assigned].change,
        StyleChangeType::LocalStyleChange
    );
    assert_eq!(dom.nodes[dirty].change, StyleChangeType::SubtreeStyleChange);
    assert_eq!(
        dom.nodes[nonmatching].change,
        StyleChangeType::NoStyleChange
    );
}

#[test]
fn reschedule_removed_siblings_as_parent_descendants_and_deduplicate_by_identity() {
    let mut dom = Dom::new();
    let root = dom.add(0, &[]);
    let removed = dom.add(root, &[]);
    let other = dom.add(root, &["b"]);
    let sibling = SiblingInvalidationSet::Create(None);
    sibling.borrow_mut().AddClass(&AtomicString::from_str("b"));
    sibling.borrow_mut().SetInvalidatesSelf();
    let mut pending = PendingInvalidations::default();
    let lists = InvalidationLists {
        descendants: Vec::new(),
        siblings: vec![sibling.clone(), sibling.clone()],
    };
    pending.ScheduleInvalidationSetsForNode(&lists, &removed, &mut dom);
    assert_eq!(
        pending.GetPendingInvalidationMap()[&removed]
            .Siblings()
            .len(),
        1
    );
    pending.RescheduleSiblingInvalidationsAsDescendants(&removed, &mut dom);
    pending.ClearInvalidation(&removed, &mut dom);
    dom.nodes[root].children.retain(|n| *n != removed);
    dom.run(&mut pending, root);
    assert_eq!(dom.nodes[other].change, StyleChangeType::LocalStyleChange);
}

#[test]
fn sibling_as_descendants_includes_subtree_sets_and_shared_combination() {
    let mut dom = Dom::new();
    let root = dom.add(0, &[]);
    let sibling = SiblingInvalidationSet::Create(None);
    sibling
        .borrow_mut()
        .EnsureSiblingDescendants()
        .SetWholeSubtreeInvalid();
    let mut pending = PendingInvalidations::default();
    pending.ScheduleSiblingInvalidationsAsDescendants(
        &InvalidationLists {
            descendants: Vec::new(),
            siblings: vec![sibling.clone()],
        },
        &root,
        &mut dom,
    );
    assert_eq!(dom.nodes[root].change, StyleChangeType::SubtreeStyleChange);
    let target = SiblingInvalidationSet::Create(Some(descendants("a")));
    let retained_child = target.borrow().Descendants().unwrap().clone();
    let source = SiblingInvalidationSet::Create(Some(descendants("b")));
    target.borrow_mut().Combine(&source.borrow());
    assert_eq!(retained_child.borrow().ToString().Utf8(), "{ .a .b }");
}

#[test]
fn tree_counting_matches_style_pseudo_and_query_dependencies_only_with_computed_style() {
    let set = InvalidationSet::TreeCountingInvalidationSet();
    let mut element = Element::default();
    assert!(!set.borrow().InvalidatesElement(&element));
    element.pseudo_sibling_functions = true;
    assert!(set.borrow().InvalidatesElement(&element));
    element.pseudo_sibling_functions = false;
    element.query_tree_counting = true;
    assert!(set.borrow().InvalidatesElement(&element));
    element.computed = false;
    assert!(!set.borrow().InvalidatesElement(&element));
}

#[test]
fn whole_subtree_clears_backings_and_traversal_flags_but_preserves_self_and_nth() {
    let set = descendants("b");
    let mut value = set.borrow_mut();
    value.SetInvalidatesSelf();
    value.SetInvalidatesNth();
    value.SetTreeBoundaryCrossing();
    value.SetInvalidatesSlotted();
    value.SetInvalidatesTreeCounting();
    value.SetWholeSubtreeInvalid();
    assert_eq!(value.ToString().Utf8(), "{ $NW }");
    assert!(value.IsEmpty());
    assert!(value.InvalidatesElement(&Element::default()));
}

#[test]
fn singleton_self_combination_preserves_identity_and_combines_into_mutable_set() {
    let singleton = InvalidationSet::SelfInvalidationSet();
    InvalidationSet::CombineSets(&singleton, &InvalidationSet::SelfInvalidationSet());
    assert!(singleton.borrow().IsSelfInvalidationSet());
    let ordinary = descendants("b");
    InvalidationSet::CombineSets(&ordinary, &singleton);
    assert_eq!(ordinary.borrow().ToString().Utf8(), "{ .b $ }");
    assert_eq!(singleton.borrow().ToString().Utf8(), "{ $ }");
    let left = SiblingInvalidationSet::Create(Some(singleton.clone()));
    let right = SiblingInvalidationSet::Create(Some(singleton));
    InvalidationSet::CombineSets(&left, &right);
    assert!(left
        .borrow()
        .Descendants()
        .unwrap()
        .borrow()
        .IsSelfInvalidationSet());
}

#[test]
fn formatting_sorts_utf16_and_keeps_isolated_surrogates() {
    let set = DescendantInvalidationSet::Create();
    set.borrow_mut()
        .AddClass(&AtomicString::from_utf16(&[0xe000]));
    set.borrow_mut()
        .AddClass(&AtomicString::from_utf16(&[0xd800]));
    assert_eq!(
        set.borrow().ToString().Span16().unwrap(),
        &[123, 32, 46, 0xd800, 32, 46, 0xe000, 32, 125]
    );
}
