// Copyright 2026 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/page/focus_controller.cc:369-928,
// 946-1110,1176-1752,2474-2486.
// Navigation retains only per-call owner/reading-flow maps. DOM identities,
// slot assignment and layout reading order are read from their original owners.

use super::focus_controller::{FocusElementRef, FocusFrameRef};
use crate::public::mojom::input::focus_type::FocusType;
use std::{collections::HashMap, rc::Rc};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FocusNavigationNode(pub u64);
type Node = FocusNavigationNode;

/// Element/ComputedStyle facts at Chromium's focus lifecycle point. Attribute
/// parsing remains with the element; no tag-name based focus policy lives here.
#[derive(Clone, Copy, Debug)]
pub struct FocusNavigationFacts {
    pub is_element: bool,
    pub is_dialog: bool,
    pub is_autofocusable: bool,
    pub is_slot: bool,
    pub has_custom_focus_logic: bool,
    pub has_author_shadow_root: bool,
    pub delegates_focus: bool,
    pub integral_tab_index: Option<i32>,
    pub has_display_contents_style: bool,
    pub is_iframe: bool,
    pub reading_order: i32,
    pub is_reading_flow_container: bool,
    pub is_scroll_marker_scope: bool,
    pub is_frame_owner: bool,
    pub is_plugin: bool,
    pub is_pseudo: bool,
    pub has_layout_box: bool,
}

/// Borrowed canonical DOM topology and the minimal Layout reading-order facts.
/// These operations expose source relations, never a FindNextFocusable callback.
/// Implementations must release document/layout borrows before returning.
pub trait FocusNavigationTree {
    fn Root(&self) -> Node;
    fn Facts(&self, node: Node) -> FocusNavigationFacts;
    fn IsFocusable(&self, node: Node) -> bool;
    fn IsKeyboardFocusable(&self, node: Node) -> bool;
    fn Parent(&self, node: Node) -> Option<Node>;
    fn FlatTreeParent(&self, node: Node) -> Option<Node>;
    fn LayoutParent(&self, node: Node) -> Option<Node>;
    fn LayoutTreeBuilderFirstChild(&self, node: Node) -> Option<Node>;
    fn LayoutTreeBuilderNextSibling(&self, node: Node) -> Option<Node>;
    fn FirstChild(&self, node: Node) -> Option<Node>;
    fn LastChild(&self, node: Node) -> Option<Node>;
    fn NextSibling(&self, node: Node) -> Option<Node>;
    fn PreviousSibling(&self, node: Node) -> Option<Node>;
    fn TreeScopeRoot(&self, node: Node) -> Node;
    fn ShadowRoot(&self, host: Node) -> Option<Node>;
    fn ShadowHost(&self, root: Node) -> Option<Node>;
    fn AssignedSlot(&self, node: Node) -> Option<Node>;
    fn HasAssignedNodes(&self, slot: Node) -> bool;
    fn OpenPopoverInvoker(&self, node: Node) -> Option<Node>;
    fn OpenPopoverTarget(&self, invoker: Node) -> Option<Node>;
    fn LayoutReadingFlowNodes(&self, owner: Node) -> Vec<Node>;
    fn ScrollMarkerOrigin(&self, marker: Node) -> Option<Node>;
    fn ScrollMarkerFor(&self, node: Node) -> Option<Node>;
    fn ContentFrame(&self, node: Node) -> Option<FocusFrameRef>;
    fn FrameOwner(&self) -> Option<Node>;
    /// The source owner may live in another Document. Keep its retained
    /// identity instead of interpreting its numeric node id in this tree.
    fn FrameOwnerElement(&self) -> Option<FocusElementRef> {
        None
    }
    fn Element(&self, node: Node) -> Option<FocusElementRef>;
    fn NodeForElement(&self, element: &FocusElementRef) -> Option<Node>;
    /// Source focusgroup segment owner provides its retained entry state.
    /// The feature-disabled state is None, not an invented entry element.
    fn FocusgroupSegment(&self, node: Node) -> Option<FocusgroupSegment>;
    fn FocusgroupSegmentEntry(&self, segment: FocusgroupSegment) -> Node;
    fn FocusedDirectionalKeyHandlerRoot(&self) -> Option<Node>;
    fn FocusgroupOwner(&self, node: Node) -> Option<Node>;
}

#[derive(Clone, Copy)]
pub struct FocusgroupSegment {
    pub owner: Node,
    pub first: Node,
}

// Source: focus_controller.cc:1763-1782.
pub fn ReadingFlowContainerOrDisplayContents(
    tree: &dyn FocusNavigationTree,
    node: Node,
    get_closest_ancestor: bool,
) -> Option<Node> {
    let facts = tree.Facts(node);
    if facts.is_reading_flow_container {
        return Some(node);
    }
    if facts.is_element && (facts.has_display_contents_style || get_closest_ancestor) {
        let parent = tree.LayoutParent(node)?;
        if tree.Facts(parent).is_reading_flow_container {
            return Some(parent);
        }
    }
    None
}
// Source: focus_controller.cc:394-415.
pub fn IsReadingFlowScopeOwner(tree: &dyn FocusNavigationTree, node: Node) -> bool {
    let facts = tree.Facts(node);
    ReadingFlowContainerOrDisplayContents(tree, node, false).is_some()
        || (!facts.is_iframe
            && facts.is_element
            && tree
                .LayoutParent(node)
                .is_some_and(|parent| tree.Facts(parent).is_reading_flow_container))
}
// Source: element.cc:877-939. Keep visual flow, slot promotion, display:contents
// ancestry, non-reading items and stable reading-order in the original order.
pub fn ElementReadingFlowChildren(tree: &dyn FocusNavigationTree, node: Node) -> Vec<Node> {
    let facts = tree.Facts(node);
    let non_container_slot = facts.is_slot && !facts.is_reading_flow_container;
    let parent = if facts.has_display_contents_style || non_container_slot {
        tree.LayoutParent(node)
    } else {
        Some(node)
    };
    let Some(parent) = parent.filter(|&parent| tree.Facts(parent).is_reading_flow_container) else {
        return Vec::new();
    };
    let mut children = Vec::new();
    let mut visited = std::collections::HashSet::new();
    for item in tree.LayoutReadingFlowNodes(parent) {
        let mut current = Some(item);
        while let Some(item) = current {
            let mut anchor = item;
            if !non_container_slot && !tree.Facts(item).is_pseudo {
                if let Some(slot) = tree
                    .AssignedSlot(item)
                    .filter(|&slot| !tree.Facts(slot).is_reading_flow_container)
                {
                    anchor = slot;
                }
            }
            let parent = tree
                .FlatTreeParent(anchor)
                .filter(|&parent| tree.Facts(parent).is_element);
            if parent == Some(node) {
                if visited.insert(item) {
                    children.push(item);
                }
                break;
            }
            current = parent;
        }
    }
    let mut child = tree.LayoutTreeBuilderFirstChild(node);
    while let Some(node) = child {
        if visited.insert(node) {
            children.push(node);
        }
        child = tree.LayoutTreeBuilderNextSibling(node);
    }
    children.sort_by_key(|&node| tree.Facts(node).reading_order);
    children
}

// Source: focus_controller.cc:369-382. Ignore a self-containing invoker,
// using the live flat tree, to avoid a popover/focus-scope owner cycle.
fn InvokerForOpenPopover(tree: &dyn FocusNavigationTree, node: Node) -> Option<Node> {
    let invoker = tree.OpenPopoverInvoker(node)?;
    let mut ancestor = Some(invoker);
    while let Some(current) = ancestor {
        if current == node {
            return None;
        }
        ancestor = tree.FlatTreeParent(current);
    }
    Some(invoker)
}

fn IsNonKeyboardFocusableShadowHost(tree: &dyn FocusNavigationTree, node: Node) -> bool {
    let facts = tree.Facts(node);
    tree.ShadowRoot(node).is_some()
        && !facts.has_custom_focus_logic
        && !facts.delegates_focus
        && (!tree.IsFocusable(node)
            || (!tree.IsKeyboardFocusable(node)
                && (facts.integral_tab_index.unwrap_or(0) >= 0
                    || tree.FocusgroupOwner(node).is_some())))
}
// Source: focus_controller.cc:2474-2486.
pub fn AdjustedTabIndex(tree: &dyn FocusNavigationTree, node: Node) -> i32 {
    let facts = tree.Facts(node);
    if IsNonKeyboardFocusableShadowHost(tree, node)
        || (tree.OpenPopoverTarget(node).is_some() && !tree.IsKeyboardFocusable(node))
    {
        0
    } else if facts.delegates_focus || facts.is_slot || IsReadingFlowScopeOwner(tree, node) {
        facts.integral_tab_index.unwrap_or(0)
    } else {
        facts
            .integral_tab_index
            .unwrap_or(if tree.IsFocusable(node) { 0 } else { -1 })
    }
}
pub struct FocusNavigation {
    tree: Rc<dyn FocusNavigationTree>,
    root: Node,
    slot: Option<Node>,
    owners: Rc<std::cell::RefCell<HashMap<Node, Option<Node>>>>,
    next: HashMap<Node, Option<Node>>,
    previous: HashMap<Node, Option<Node>>,
    first: Option<Node>,
    last: Option<Node>,
    reading_flow: bool,
    marker: Option<Node>,
}
impl FocusNavigation {
    fn Create(
        tree: Rc<dyn FocusNavigationTree>,
        root: Node,
        owners: Rc<std::cell::RefCell<HashMap<Node, Option<Node>>>>,
    ) -> Self {
        let slot = tree.Facts(root).is_slot.then_some(root);
        let actual_root = if slot.is_some() && tree.HasAssignedNodes(root) {
            tree.ShadowHost(tree.TreeScopeRoot(root))
                .expect("an assigned slot is inside a shadow root")
        } else {
            root
        };
        let marker = tree.Facts(root).is_scroll_marker_scope.then_some(root);
        let actual_root = marker
            .and_then(|marker| tree.ScrollMarkerOrigin(marker))
            .unwrap_or(actual_root);
        let reading_flow_node = tree.ShadowHost(root).unwrap_or(root);
        let reading_flow =
            ReadingFlowContainerOrDisplayContents(tree.as_ref(), reading_flow_node, false)
                .is_some();
        let mut result = Self {
            tree,
            root: actual_root,
            slot,
            owners,
            next: HashMap::new(),
            previous: HashMap::new(),
            first: None,
            last: None,
            reading_flow,
            marker,
        };
        if reading_flow {
            let owner = result.Owner().expect("reading-flow scope has an owner");
            let mut previous = None;
            for child in ElementReadingFlowChildren(result.tree.as_ref(), owner) {
                if !result.tree.Facts(child).is_element
                    || result.tree.Facts(child).is_pseudo
                    || !result.IsOwnedByRoot(child)
                {
                    continue;
                }
                if let Some(previous) = previous {
                    result.next.insert(previous, Some(child));
                } else {
                    result.first = Some(child);
                }
                result.previous.insert(child, previous);
                previous = Some(child);
            }
            if let Some(last) = previous {
                result.next.insert(last, None);
                result.last = Some(last);
            }
        }
        result
    }
    fn FindScopeOwner(&self, current: Node) -> Option<Node> {
        let mut node = Some(current);
        while let Some(current) = node {
            if let Some(slot) = self.tree.AssignedSlot(current) {
                // Source: focus_controller.cc:2330-2344. ReadingFlowWithSlots
                // is stable in runtime_enabled_features.json5:4995-4996.
                if self.tree.Facts(slot).has_layout_box {
                    return Some(slot);
                }
                if let Some(parent) = self.tree.FlatTreeParent(slot) {
                    if IsReadingFlowScopeOwner(self.tree.as_ref(), parent) {
                        return Some(parent);
                    }
                }
                return Some(slot);
            }
            if let Some(marker) = self.tree.ScrollMarkerFor(current) {
                return Some(marker);
            }
            node = self.tree.Parent(current);
            if node.is_some_and(|parent| IsReadingFlowScopeOwner(self.tree.as_ref(), parent)) {
                return node;
            }
        }
        None
    }
    fn FindOwner(&self, node: Node) -> Option<Node> {
        if let Some(owner) = self.owners.borrow().get(&node).copied() {
            return owner;
        }
        let owner = if let Some(owner) = self.FindScopeOwner(node) {
            Some(owner)
        } else if self
            .tree
            .Parent(node)
            .is_some_and(|p| self.tree.Facts(p).is_slot)
        {
            self.tree.Parent(node)
        } else if node == self.tree.TreeScopeRoot(node) {
            self.tree
                .ShadowHost(node)
                .or_else(|| self.tree.FrameOwner())
        } else if let Some(invoker) = InvokerForOpenPopover(self.tree.as_ref(), node) {
            Some(invoker)
        } else {
            self.tree
                .Parent(node)
                .and_then(|parent| self.FindOwner(parent))
        };
        self.owners.borrow_mut().insert(node, owner);
        owner
    }
    fn Owner(&self) -> Option<Node> {
        self.slot
            .or_else(|| IsReadingFlowScopeOwner(self.tree.as_ref(), self.root).then_some(self.root))
            .or(self.marker)
            .or_else(|| self.FindOwner(self.root))
    }
    fn IsOwnedByRoot(&self, node: Node) -> bool {
        self.FindOwner(node) == self.Owner()
    }
    fn NextNode(&self, node: Node) -> Option<Node> {
        if let Some(child) = self.tree.FirstChild(node) {
            return Some(child);
        }
        let mut node = node;
        loop {
            if node == self.root {
                return None;
            }
            if let Some(sibling) = self.tree.NextSibling(node) {
                return Some(sibling);
            }
            node = self.tree.Parent(node)?;
        }
    }
    fn PreviousNode(&self, node: Node) -> Option<Node> {
        if node == self.root {
            return None;
        }
        if let Some(mut sibling) = self.tree.PreviousSibling(node) {
            while let Some(last) = self.tree.LastChild(sibling) {
                sibling = last;
            }
            Some(sibling)
        } else {
            self.tree.Parent(node)
        }
    }
    fn Next(&self, node: Node) -> Option<Node> {
        if let Some(next) = self.next.get(&node) {
            return *next;
        }
        let mut next = self.NextNode(node);
        while let Some(node) = next {
            if self.tree.Facts(node).is_element && self.IsOwnedByRoot(node) {
                return Some(node);
            }
            next = self.NextNode(node);
        }
        None
    }
    fn Previous(&self, node: Node) -> Option<Node> {
        if let Some(previous) = self.previous.get(&node) {
            return *previous;
        }
        let mut previous = self.PreviousNode(node);
        while let Some(node) = previous {
            if node == self.root {
                return None;
            }
            if self.tree.Facts(node).is_element && self.IsOwnedByRoot(node) {
                return Some(node);
            }
            previous = self.PreviousNode(node);
        }
        None
    }
    fn First(&self) -> Option<Node> {
        if self.marker.is_some() {
            return Some(self.root);
        }
        if self.first.is_some() {
            return self.first;
        }
        if self.tree.Facts(self.root).is_element && self.IsOwnedByRoot(self.root) {
            return Some(self.root);
        }
        self.Next(self.root)
    }
    fn Last(&self) -> Option<Node> {
        if self.last.is_some() {
            return self.last;
        }
        let mut last = self.tree.LastChild(self.root)?;
        while let Some(child) = self.tree.LastChild(last) {
            last = child;
        }
        if self.tree.Facts(last).is_element && self.IsOwnedByRoot(last) {
            Some(last)
        } else {
            self.Previous(last)
        }
    }
    fn IsShadowHost(&self, node: Node) -> bool {
        self.tree.ShadowRoot(node).is_some() && !self.tree.Facts(node).has_custom_focus_logic
    }
    fn IsNonKeyboardFocusableShadowHost(&self, node: Node) -> bool {
        IsNonKeyboardFocusableShadowHost(self.tree.as_ref(), node)
    }
    fn IsNonFocusableScopeOwner(&self, node: Node) -> bool {
        let facts = self.tree.Facts(node);
        self.IsNonKeyboardFocusableShadowHost(node)
            || facts.is_slot
            || ((IsReadingFlowScopeOwner(self.tree.as_ref(), node)
                || facts.is_scroll_marker_scope
                || self.tree.OpenPopoverTarget(node).is_some())
                && !self.tree.IsKeyboardFocusable(node))
    }
    fn AdjustedTabIndex(&self, node: Node) -> i32 {
        let index = AdjustedTabIndex(self.tree.as_ref(), node);
        if self.reading_flow {
            index.min(0)
        } else {
            index
        }
    }
    fn ShouldVisit(&self, node: Node) -> bool {
        let facts = self.tree.Facts(node);
        if InvokerForOpenPopover(self.tree.as_ref(), node).is_some()
            && !self.tree.IsKeyboardFocusable(node)
        {
            return false;
        }
        self.tree.IsKeyboardFocusable(node)
            || facts.delegates_focus
            || self.IsNonFocusableScopeOwner(node)
    }
    fn InnerRoot(&self, node: Node) -> Option<Node> {
        let facts = self.tree.Facts(node);
        if self.IsShadowHost(node) {
            self.tree.ShadowRoot(node)
        } else if facts.is_slot
            || IsReadingFlowScopeOwner(self.tree.as_ref(), node)
            || facts.is_scroll_marker_scope
        {
            Some(node)
        } else {
            self.tree
                .OpenPopoverTarget(node)
                .map(|popover| self.tree.ShadowRoot(popover).unwrap_or(popover))
        }
    }
}

pub struct ScopedFocusNavigation {
    current: Option<Node>,
    navigation: FocusNavigation,
    segment_entries: HashMap<Node, Node>,
    focused_directional_key_handler_root: Option<Option<Node>>,
}
impl ScopedFocusNavigation {
    pub fn ForDocument(tree: Rc<dyn FocusNavigationTree>) -> Self {
        let root = tree.Root();
        Self::new(
            tree,
            root,
            None,
            Rc::new(std::cell::RefCell::new(HashMap::new())),
        )
    }
    pub fn ForElement(tree: Rc<dyn FocusNavigationTree>, node: Node) -> Self {
        let mut scope = Self::ForDocument(tree);
        scope = scope.CreateFor(node);
        scope
    }
    fn new(
        tree: Rc<dyn FocusNavigationTree>,
        root: Node,
        current: Option<Node>,
        owners: Rc<std::cell::RefCell<HashMap<Node, Option<Node>>>>,
    ) -> Self {
        Self {
            current,
            navigation: FocusNavigation::Create(tree, root, owners),
            segment_entries: HashMap::new(),
            focused_directional_key_handler_root: None,
        }
    }
    fn AtRoot(&self, root: Node, current: Option<Node>) -> Self {
        Self::new(
            self.navigation.tree.clone(),
            root,
            current,
            self.navigation.owners.clone(),
        )
    }
    fn CreateFor(&self, node: Node) -> Self {
        if let Some(owner) = self.navigation.FindScopeOwner(node) {
            return self.AtRoot(owner, Some(node));
        }
        let tree = &self.navigation.tree;
        let mut ancestor = tree.Parent(node);
        while let Some(parent) = ancestor {
            if tree.Facts(parent).is_slot {
                if !tree.HasAssignedNodes(parent) {
                    return self.AtRoot(parent, Some(node));
                }
                break;
            }
            ancestor = tree.Parent(parent);
        }
        let mut ancestor = Some(node);
        while let Some(parent) = ancestor {
            // The stable OpenPopoverInvokerRestrictToSameTreeScope feature
            // (runtime_enabled_features.json5:4423-4424) keeps this search in
            // the starting element's TreeScope while following the flat tree.
            if tree.TreeScopeRoot(node) != tree.TreeScopeRoot(parent) {
                break;
            }
            if InvokerForOpenPopover(tree.as_ref(), parent).is_some() {
                return self.AtRoot(parent, Some(node));
            }
            ancestor = tree.FlatTreeParent(parent);
        }
        self.AtRoot(tree.TreeScopeRoot(node), Some(node))
    }
    fn Owner(&self) -> Option<Node> {
        let owner = self.navigation.Owner()?;
        if IsReadingFlowScopeOwner(self.navigation.tree.as_ref(), owner) {
            InvokerForOpenPopover(self.navigation.tree.as_ref(), owner).or(Some(owner))
        } else {
            Some(owner)
        }
    }
    fn IsNonEntryFocusgroupItem(&mut self, node: Node) -> bool {
        let tree = &self.navigation.tree;
        let focused_handler = *self
            .focused_directional_key_handler_root
            .get_or_insert_with(|| tree.FocusedDirectionalKeyHandlerRoot());
        let Some(segment) = tree.FocusgroupSegment(node) else {
            return false;
        };
        let entry = *self
            .segment_entries
            .entry(segment.first)
            .or_insert_with(|| tree.FocusgroupSegmentEntry(segment));
        if entry != node {
            if let Some(root) = focused_handler {
                if root != node && tree.FocusgroupOwner(root) == Some(segment.owner) {
                    return false;
                }
            }
        }
        entry != node
    }
    fn CanVisit(&mut self, node: Node) -> bool {
        if !self.navigation.ShouldVisit(node) {
            return false;
        }
        let facts = self.navigation.tree.Facts(node);
        !self.IsNonEntryFocusgroupItem(node)
            || facts.has_author_shadow_root
            || facts.is_slot
            || IsReadingFlowScopeOwner(self.navigation.tree.as_ref(), node)
    }
    fn Move(&mut self, forward: bool) {
        self.current = self.current.and_then(|node| {
            if forward {
                self.navigation.Next(node)
            } else {
                self.navigation.Previous(node)
            }
        });
    }
    fn Exact(&mut self, index: i32, forward: bool) -> Option<Node> {
        while let Some(node) = self.current {
            if self.CanVisit(node) && self.navigation.AdjustedTabIndex(node) == index {
                return Some(node);
            }
            self.Move(forward);
        }
        None
    }
    fn Greater(&mut self, index: i32) -> Option<Node> {
        let mut winner = None;
        let mut winning_index = i32::MAX;
        while let Some(node) = self.current {
            let candidate = self.navigation.AdjustedTabIndex(node);
            if self.CanVisit(node)
                && candidate > index
                && (winner.is_none() || candidate < winning_index)
            {
                winner = Some(node);
                winning_index = candidate;
            }
            self.Move(true);
        }
        self.current = winner;
        winner
    }
    fn Lower(&mut self, index: i32) -> Option<Node> {
        let mut winner = None;
        let mut winning_index = 0;
        while let Some(node) = self.current {
            let candidate = self.navigation.AdjustedTabIndex(node);
            if self.CanVisit(node) && candidate < index && candidate > winning_index {
                winner = Some(node);
                winning_index = candidate;
            }
            self.Move(false);
        }
        self.current = winner;
        winner
    }
    fn NextFocusable(&mut self) -> Option<Node> {
        let initial = self.current;
        let index = initial.map(|node| self.navigation.AdjustedTabIndex(node));
        if let Some(index) = index {
            self.Move(true);
            if index < 0 {
                while let Some(node) = self.current {
                    if self.CanVisit(node) && self.navigation.AdjustedTabIndex(node) >= 0 {
                        return Some(node);
                    }
                    self.Move(true);
                }
            } else if let Some(node) = self.Exact(index, true) {
                return Some(node);
            }
            if index == 0
                || initial.is_some_and(|node| {
                    !self.navigation.ShouldVisit(node)
                        && self.navigation.tree.ScrollMarkerFor(node).is_some()
                })
            {
                return None;
            }
        }
        self.current = self.navigation.First();
        if let Some(node) = self.Greater(index.unwrap_or(0)) {
            return Some(node);
        }
        self.current = self.navigation.First();
        self.Exact(0, true)
    }
    fn PreviousFocusable(&mut self) -> Option<Node> {
        let initial = self.current;
        let index = initial
            .map(|node| self.navigation.AdjustedTabIndex(node))
            .unwrap_or(0);
        if initial.is_some() {
            self.Move(false);
        } else {
            self.current = self.navigation.Last();
        }
        if index < 0 {
            while let Some(node) = self.current {
                if self.CanVisit(node) && self.navigation.AdjustedTabIndex(node) >= 0 {
                    return Some(node);
                }
                self.Move(false);
            }
        } else if let Some(node) = self.Exact(index, false) {
            return Some(node);
        }
        self.current = self.navigation.Last();
        self.Lower(if initial.is_some() && index != 0 {
            index
        } else {
            i32::MAX
        })
    }
    fn Recursively(&mut self, forward: bool) -> Option<Node> {
        loop {
            let found = if forward {
                self.NextFocusable()?
            } else {
                self.PreviousFocusable()?
            };
            let facts = self.navigation.tree.Facts(found);
            let shadow = self.navigation.IsShadowHost(found);
            let enter = if forward {
                (facts.delegates_focus && self.navigation.AdjustedTabIndex(found) >= 0 && shadow)
                    || self.navigation.IsNonFocusableScopeOwner(found)
                    || (shadow && self.IsNonEntryFocusgroupItem(found))
            } else {
                (shadow
                    && (self.navigation.tree.IsKeyboardFocusable(found) || facts.delegates_focus))
                    || self.navigation.IsNonFocusableScopeOwner(found)
                    || IsReadingFlowScopeOwner(self.navigation.tree.as_ref(), found)
                    || (facts.is_scroll_marker_scope && self.Owner() != Some(found))
            };
            if enter {
                if let Some(root) = self.navigation.InnerRoot(found) {
                    if let Some(inner) = self.AtRoot(root, None).Recursively(forward) {
                        return Some(inner);
                    }
                }
                if self.navigation.IsNonFocusableScopeOwner(found)
                    || facts.delegates_focus
                    || self.IsNonEntryFocusgroupItem(found)
                {
                    continue;
                }
            }
            if !facts.delegates_focus {
                return Some(found);
            }
        }
    }
    pub fn FindAcrossScopes(self, direction: FocusType) -> Option<FocusElementRef> {
        self.FindAcrossScopesWithFrameAncestors(direction, true)
    }
    fn FindAcrossScopesWithFrameAncestors(
        mut self,
        direction: FocusType,
        ascend_frames: bool,
    ) -> Option<FocusElementRef> {
        let forward = match direction {
            FocusType::kForward => true,
            FocusType::kBackward => false,
            _ => panic!("sequential focus direction required"),
        };
        let mut found = None;
        if forward {
            if let Some(current) = self.current {
                let facts = self.navigation.tree.Facts(current);
                if self.navigation.IsShadowHost(current)
                    || self.navigation.tree.OpenPopoverTarget(current).is_some()
                    || IsReadingFlowScopeOwner(self.navigation.tree.as_ref(), current)
                    || facts.is_scroll_marker_scope
                {
                    if let Some(root) = self.navigation.InnerRoot(current) {
                        found = self.AtRoot(root, None).Recursively(true);
                    }
                }
            }
        }
        if found.is_none() {
            found = self.Recursively(forward);
        }
        if !forward {
            while let Some(node) = found {
                let Some(popover) = self.navigation.tree.OpenPopoverTarget(node) else {
                    break;
                };
                let root = self.navigation.tree.ShadowRoot(popover).unwrap_or(popover);
                if let Some(inner) = self.AtRoot(root, None).Recursively(false) {
                    found = Some(inner);
                } else {
                    break;
                }
            }
        }
        while found.is_none() {
            if self.Owner().is_none() && ascend_frames {
                let owner = self.navigation.tree.FrameOwnerElement()?;
                let tree = owner.Document().FocusNavigation();
                let owner = tree.NodeForElement(&owner)?;
                self = Self::ForElement(tree, owner);
                found = self.Recursively(forward);
                continue;
            }
            let mut owner = self.Owner()?;
            let facts = self.navigation.tree.Facts(owner);
            if !forward
                && (self.navigation.IsShadowHost(owner)
                    || self.navigation.tree.OpenPopoverTarget(owner).is_some()
                    || IsReadingFlowScopeOwner(self.navigation.tree.as_ref(), owner))
                && self.navigation.tree.IsKeyboardFocusable(owner)
                && !facts.delegates_focus
                && !self.IsNonEntryFocusgroupItem(owner)
            {
                found = Some(owner);
                break;
            }
            if forward {
                if let Some(popover) = self.navigation.tree.OpenPopoverTarget(owner) {
                    if popover != self.navigation.root {
                        found = self.AtRoot(popover, None).Recursively(true);
                    }
                }
            }
            if found.is_some() {
                break;
            }
            let mut outer = self.CreateFor(owner);
            while outer.Owner() == Some(owner) {
                let Some(parent) = self.navigation.tree.Parent(owner) else {
                    break;
                };
                owner = parent;
                outer = self.CreateFor(owner);
            }
            self = outer;
            found = self.Recursively(forward);
        }
        let node = found?;
        if self.navigation.tree.Facts(node).is_frame_owner {
            if let Some(frame) = self
                .navigation
                .tree
                .ContentFrame(node)
                .filter(|frame| frame.IsLocalFrame())
            {
                if let Some(document) = frame.Document() {
                    document.UpdateStyleAndLayoutForFocus();
                    if let Some(inner) = Self::ForDocument(document.FocusNavigation())
                        .FindAcrossScopesWithFrameAncestors(direction, false)
                    {
                        return Some(inner);
                    }
                }
            }
        }
        self.navigation.tree.Element(node)
    }
}
