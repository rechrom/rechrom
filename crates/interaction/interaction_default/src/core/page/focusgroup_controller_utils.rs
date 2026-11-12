// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/page/focusgroup_controller_utils.cc:31-169,301-339,426-443,
// 507-547,577-645,737-931; core/dom/focusgroup_flags.cc:181-195.
//! Sequential focusgroup queries over borrowed canonical DOM identities.
//! The only mappings are Chromium's call-scoped reading-flow links; this
//! owner never retains or materializes a second document tree.

use std::collections::{HashMap, HashSet};

use foundation::focusgroup_flags::{FocusgroupBehavior, FocusgroupData, FocusgroupFlags};

use super::focus_navigation::{FocusNavigationNode as Node, FocusgroupSegment};

// Source: focusgroup_controller_utils.h:26-43; .cc:172-300.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusgroupDirection {
    kNone,
    kBackwardInline,
    kForwardInline,
    kBackwardBlock,
    kForwardBlock,
}
#[derive(Clone, Copy)]
pub enum FocusgroupType {
    kGrid,
    kLinear,
}
#[derive(Clone, Copy)]
pub enum FocusgroupItemPosition {
    kFirst,
    kLast,
}
pub trait DirectionalFocusgroupTree: FocusgroupTree {
    fn V2Enabled(&self) -> bool;
    fn WritingDirection(&self, node: Node) -> Option<foundation::WritingDirectionMode>;
    fn IsTable(&self, node: Node) -> bool;
    fn IsTableCell(&self, node: Node) -> bool;
}
pub fn FocusgroupDirectionForEvent(
    tree: &dyn DirectionalFocusgroupTree,
    element: Node,
    key: &str,
    modifiers: bool,
) -> FocusgroupDirection {
    use foundation::LogicalDirection as L;
    use FocusgroupDirection as D;
    if modifiers {
        return D::kNone;
    }
    let Some(writing) = tree.WritingDirection(element) else {
        return D::kNone;
    };
    let logical = match key {
        "ArrowDown" => writing.Bottom(),
        "ArrowRight" => writing.Right(),
        "ArrowUp" => writing.Top(),
        "ArrowLeft" => writing.Left(),
        _ => return D::kNone,
    };
    match logical {
        L::kInlineStart => D::kBackwardInline,
        L::kInlineEnd => D::kForwardInline,
        L::kBlockStart => D::kBackwardBlock,
        L::kBlockEnd => D::kForwardBlock,
    }
}
pub fn IsDirectionForward(direction: FocusgroupDirection) -> bool {
    matches!(
        direction,
        FocusgroupDirection::kForwardInline | FocusgroupDirection::kForwardBlock
    )
}
pub fn IsDirectionBackward(direction: FocusgroupDirection) -> bool {
    matches!(
        direction,
        FocusgroupDirection::kBackwardInline | FocusgroupDirection::kBackwardBlock
    )
}
pub fn IsDirectionInline(direction: FocusgroupDirection) -> bool {
    matches!(
        direction,
        FocusgroupDirection::kBackwardInline | FocusgroupDirection::kForwardInline
    )
}
pub fn IsDirectionBlock(direction: FocusgroupDirection) -> bool {
    matches!(
        direction,
        FocusgroupDirection::kBackwardBlock | FocusgroupDirection::kForwardBlock
    )
}
pub fn IsAxisSupported(flags: FocusgroupFlags, direction: FocusgroupDirection) -> bool {
    flags.Intersects(FocusgroupFlags::kInline) && IsDirectionInline(direction)
        || flags.Intersects(FocusgroupFlags::kBlock) && IsDirectionBlock(direction)
}
pub fn WrapsInDirection(flags: FocusgroupFlags, direction: FocusgroupDirection) -> bool {
    flags.Intersects(FocusgroupFlags::kWrapInline) && IsDirectionInline(direction)
        || flags.Intersects(FocusgroupFlags::kWrapBlock) && IsDirectionBlock(direction)
}
pub fn FindNearestFocusgroupAncestor(
    tree: &dyn DirectionalFocusgroupTree,
    element: Node,
    kind: FocusgroupType,
) -> Option<Node> {
    let mut ancestor = tree.ParentElement(element);
    while let Some(node) = ancestor {
        let behavior = tree.Data(node).behavior;
        if behavior != FocusgroupBehavior::kNoBehavior {
            return match kind {
                FocusgroupType::kGrid => {
                    assert!(tree.V2Enabled());
                    (behavior == FocusgroupBehavior::kGrid && tree.IsTable(node)).then_some(node)
                }
                FocusgroupType::kLinear => (behavior != FocusgroupBehavior::kGrid).then_some(node),
            };
        }
        ancestor = tree.ParentElement(node);
    }
    None
}

pub trait FocusgroupTree {
    fn Enabled(&self) -> bool;
    fn ParentElement(&self, node: Node) -> Option<Node>;
    fn NextElement(&self, node: Node, skip_subtree: bool) -> Option<Node>;
    fn PreviousElement(&self, node: Node, skip_subtree: bool) -> Option<Node>;
    fn Data(&self, node: Node) -> FocusgroupData;
    fn IsInTopLayer(&self, node: Node) -> bool;
    fn IsFocusable(&self, node: Node) -> bool;
    fn IsKeyboardFocusable(&self, node: Node) -> bool;
    fn FocusedElement(&self) -> Option<Node>;
    fn LastFocused(&self, owner: Node) -> Option<Node>;
    fn IsFocusgroupStart(&self, node: Node) -> bool;
    fn NativeArrowKeyAxes(&self, node: Node) -> FocusgroupFlags;
    /// FocusController::ReadingFlowContainerOrDisplayContents(node, true).
    fn ReadingFlowContainer(&self, node: Node) -> Option<Node>;
    /// Real non-pseudo elements from the resident Layout reading-flow list.
    fn ReadingFlowElements(&self, node: Node) -> Vec<Node>;
}

#[derive(Default)]
struct FocusgroupVisualOrderTraversalContext {
    reading_flow_elements: HashSet<Node>,
    reading_flow_next_elements: HashMap<Node, Option<Node>>,
    reading_flow_previous_elements: HashMap<Node, Node>,
}

impl FocusgroupVisualOrderTraversalContext {
    fn VisitReadingFlowContainerIfNeeded(&mut self, tree: &dyn FocusgroupTree, node: Option<Node>) {
        let Some(container) = node.and_then(|node| tree.ReadingFlowContainer(node)) else {
            return;
        };
        if !self.reading_flow_elements.insert(container) {
            return;
        }
        let after = self
            .reading_flow_next_elements
            .get(&container)
            .copied()
            .unwrap_or_else(|| tree.NextElement(container, true));
        let mut previous = container;
        for child in tree.ReadingFlowElements(container) {
            self.reading_flow_previous_elements.insert(child, previous);
            self.reading_flow_next_elements
                .insert(previous, Some(child));
            previous = child;
        }
        self.reading_flow_next_elements.insert(previous, after);
        if let Some(after) = after {
            self.reading_flow_previous_elements.insert(after, previous);
        }
    }

    fn NextInDirection(
        &mut self,
        tree: &dyn FocusgroupTree,
        current: Node,
        forward: bool,
        skip_subtree: bool,
    ) -> Option<Node> {
        self.VisitReadingFlowContainerIfNeeded(tree, Some(current));
        if forward {
            return self
                .reading_flow_next_elements
                .get(&current)
                .copied()
                .unwrap_or_else(|| tree.NextElement(current, skip_subtree));
        }
        let previous = self
            .reading_flow_previous_elements
            .get(&current)
            .copied()
            .or_else(|| tree.PreviousElement(current, skip_subtree));
        self.VisitReadingFlowContainerIfNeeded(tree, previous);
        self.reading_flow_previous_elements
            .get(&current)
            .copied()
            .or(previous)
    }
}

// Source: core/dom/focusgroup_flags.cc:181-195. The element itself is never
// its own owner, and an intervening explicit opt-out ends this search.
pub fn FindFocusgroupOwner(tree: &dyn FocusgroupTree, element: Node) -> Option<Node> {
    if !tree.Enabled() {
        return None;
    }
    let mut ancestor = tree.ParentElement(element);
    while let Some(node) = ancestor {
        let data = tree.Data(node);
        if data.behavior == FocusgroupBehavior::kOptOut {
            return None;
        }
        if data.IsActualFocusgroup() {
            return Some(node);
        }
        ancestor = tree.ParentElement(node);
    }
    None
}

fn IsExcludedSubtreeRoot(tree: &dyn FocusgroupTree, element: Node) -> bool {
    let data = tree.Data(element);
    data.behavior == FocusgroupBehavior::kOptOut
        || (tree.IsInTopLayer(element) && !data.IsActualFocusgroup())
}

fn FindExcludedSubtreeRoot(tree: &dyn FocusgroupTree, element: Node) -> Option<Node> {
    let mut current = Some(element);
    while let Some(node) = current {
        if IsExcludedSubtreeRoot(tree, node) {
            return Some(node);
        }
        if tree.Data(node).IsActualFocusgroup() {
            return None;
        }
        current = tree.ParentElement(node);
    }
    None
}

pub fn GetFocusgroupOwnerOfItem(tree: &dyn FocusgroupTree, element: Node) -> Option<Node> {
    if !tree.Enabled()
        || !tree.IsFocusable(element)
        || tree.IsInTopLayer(element)
        || FindExcludedSubtreeRoot(tree, element).is_some()
    {
        return None;
    }
    FindFocusgroupOwner(tree, element)
}

fn IsDescendantOf(tree: &dyn FocusgroupTree, element: Node, ancestor: Node) -> bool {
    let mut parent = tree.ParentElement(element);
    while let Some(node) = parent {
        if node == ancestor {
            return true;
        }
        parent = tree.ParentElement(node);
    }
    false
}

// Source: focusgroup_controller_utils.cc:371-425,435-443,453-506,646-736.
pub fn NextFocusgroupItemInDirection(
    tree: &dyn FocusgroupTree,
    owner: Node,
    current: Node,
    direction: FocusgroupDirection,
) -> Option<Node> {
    if owner == current {
        return None;
    }
    let forward = IsDirectionForward(direction);
    let mut traversal = FocusgroupVisualOrderTraversalContext::default();
    let mut next = traversal.NextInDirection(tree, current, forward, false);
    while let Some(node) = next.filter(|&node| IsDescendantOf(tree, node, owner)) {
        let skip =
            tree.Data(node).behavior == FocusgroupBehavior::kOptOut || tree.IsInTopLayer(node);
        let nested = tree.Data(node).IsActualFocusgroup();
        if !skip
            && (nested || GetFocusgroupOwnerOfItem(tree, node) == Some(owner))
            && tree.IsKeyboardFocusable(node)
        {
            return Some(node);
        }
        next = traversal.NextInDirection(tree, node, forward, skip || nested);
    }
    None
}
pub fn IsGridFocusgroupItem(tree: &dyn DirectionalFocusgroupTree, node: Node) -> bool {
    assert!(tree.V2Enabled());
    tree.IsFocusable(node) && tree.IsTableCell(node)
}
fn FindDirectionalKeyHandlerRootForAxes(
    tree: &dyn FocusgroupTree,
    node: Node,
    owner: Node,
    axes: FocusgroupFlags,
) -> Option<Node> {
    let mut current = Some(node);
    while let Some(node) = current.filter(|&node| node != owner) {
        if tree.NativeArrowKeyAxes(node).Intersects(axes) {
            return Some(node);
        }
        current = tree.ParentElement(node);
    }
    None
}
pub fn IsInDirectionalKeyHandlerForAnyAxis(
    tree: &dyn FocusgroupTree,
    node: Node,
    owner: Node,
) -> bool {
    FindDirectionalKeyHandlerRootForAxes(
        tree,
        node,
        owner,
        FocusgroupFlags::kInline | FocusgroupFlags::kBlock,
    )
    .is_some()
}
pub fn IsInDirectionalKeyHandler(
    tree: &dyn FocusgroupTree,
    node: Node,
    direction: FocusgroupDirection,
) -> bool {
    if !tree.Enabled() {
        return false;
    }
    let Some(owner) = FindFocusgroupOwner(tree, node) else {
        return false;
    };
    let data = tree.Data(owner);
    if !data.IsActualFocusgroup() {
        return false;
    }
    let axis = if IsDirectionInline(direction) {
        FocusgroupFlags::kInline
    } else {
        FocusgroupFlags::kBlock
    };
    data.flags.Intersects(axis)
        && FindDirectionalKeyHandlerRootForAxes(tree, node, owner, axis).is_some()
}
pub fn WrappedFocusgroupCandidate(
    tree: &dyn FocusgroupTree,
    owner: Node,
    current: Node,
    direction: FocusgroupDirection,
) -> Option<Node> {
    FocusgroupItemWithin(
        tree,
        owner,
        if IsDirectionForward(direction) {
            FocusgroupItemPosition::kFirst
        } else {
            FocusgroupItemPosition::kLast
        },
    )
    .filter(|&candidate| candidate != current)
}
pub fn FocusgroupItemWithin(
    tree: &dyn FocusgroupTree,
    owner: Node,
    position: FocusgroupItemPosition,
) -> Option<Node> {
    if !tree.Data(owner).IsActualFocusgroup() {
        return None;
    }
    let mut traversal = FocusgroupVisualOrderTraversalContext::default();
    let mut result = None;
    let mut current = traversal.NextInDirection(tree, owner, true, false);
    while let Some(node) = current.filter(|&node| IsDescendantOf(tree, node, owner)) {
        let excluded =
            tree.Data(node).behavior == FocusgroupBehavior::kOptOut || tree.IsInTopLayer(node);
        let nested = tree.Data(node).IsActualFocusgroup();
        if !excluded
            && (nested || GetFocusgroupOwnerOfItem(tree, node) == Some(owner))
            && tree.IsKeyboardFocusable(node)
        {
            if matches!(position, FocusgroupItemPosition::kFirst) {
                return Some(node);
            }
            result = Some(node);
        }
        current = traversal.NextInDirection(tree, node, true, excluded || nested);
    }
    result
}

fn ContainsKeyboardFocusableContent(tree: &dyn FocusgroupTree, element: Node) -> bool {
    if tree.IsKeyboardFocusable(element) {
        return true;
    }
    let mut current = tree.NextElement(element, false);
    while let Some(node) = current.filter(|&node| IsDescendantOf(tree, node, element)) {
        if tree.IsKeyboardFocusable(node) {
            return true;
        }
        current = tree.NextElement(node, false);
    }
    false
}

fn ContainsFocusedElement(tree: &dyn FocusgroupTree, element: Node) -> bool {
    tree.FocusedElement()
        .is_some_and(|focused| focused == element || IsDescendantOf(tree, focused, element))
}

fn NextFocusgroupItemInSegmentInDirection(
    tree: &dyn FocusgroupTree,
    item: Node,
    owner: Node,
    forward: bool,
) -> Option<Node> {
    let mut traversal = FocusgroupVisualOrderTraversalContext::default();
    let mut element = traversal.NextInDirection(tree, item, forward, false);
    while let Some(node) = element.filter(|&node| IsDescendantOf(tree, node, owner)) {
        let mut excluded = None;
        let mut nested = None;
        if !forward {
            excluded = FindExcludedSubtreeRoot(tree, node)
                .or_else(|| tree.IsInTopLayer(node).then_some(node));
            nested = FindFocusgroupOwner(tree, node).filter(|&nested| nested != owner);
        } else if IsExcludedSubtreeRoot(tree, node) || tree.IsInTopLayer(node) {
            excluded = Some(node);
        } else if tree.Data(node).IsActualFocusgroup() {
            nested = Some(node);
        }
        if let Some(nested) = nested {
            if ContainsKeyboardFocusableContent(tree, nested) {
                return None;
            }
            element = traversal.NextInDirection(tree, nested, forward, true);
            continue;
        }
        if let Some(excluded) = excluded {
            if ContainsFocusedElement(tree, excluded)
                || ContainsKeyboardFocusableContent(tree, excluded)
            {
                return None;
            }
            element = traversal.NextInDirection(tree, excluded, forward, true);
            continue;
        }
        if tree.IsKeyboardFocusable(node) || tree.FocusedElement() == Some(node) {
            return Some(node);
        }
        element = traversal.NextInDirection(tree, node, forward, false);
    }
    None
}

fn FocusgroupItemInSegment(tree: &dyn FocusgroupTree, item: Node) -> Option<Node> {
    let owner = FindFocusgroupOwner(tree, item)?;
    if !tree.IsKeyboardFocusable(item) {
        return None;
    }
    let mut result = item;
    while let Some(next) = NextFocusgroupItemInSegmentInDirection(tree, result, owner, false) {
        result = next;
    }
    Some(result)
}

pub fn GetEntryElementForFocusgroupSegmentFromFirst(
    tree: &dyn FocusgroupTree,
    first: Node,
    owner: Node,
) -> Node {
    let memory = tree.LastFocused(owner);
    let mut current = Some(first);
    let mut priority = None;
    let mut first_item = None;
    let mut memory_in_segment = false;
    while let Some(item) = current {
        if tree.FocusedElement() == Some(item) {
            return item;
        }
        if memory == Some(item) {
            memory_in_segment = true;
            current = NextFocusgroupItemInSegmentInDirection(tree, item, owner, true);
            continue;
        }
        if priority.is_none() && tree.IsFocusgroupStart(item) {
            priority = Some(item);
        }
        first_item.get_or_insert(item);
        current = NextFocusgroupItemInSegmentInDirection(tree, item, owner, true);
    }
    (if memory_in_segment {
        memory
    } else {
        priority.or(first_item)
    })
    .expect("a focusgroup segment must have an entry element")
}

/// Source focus_controller.cc:843-928, with its navigation-pass entry cache
/// retained by ScopedFocusNavigation. None is the source non-participant path.
pub fn Segment(tree: &dyn FocusgroupTree, item: Node) -> Option<FocusgroupSegment> {
    if !tree.Enabled() || FindExcludedSubtreeRoot(tree, item).is_some() {
        return None;
    }
    let owner = GetFocusgroupOwnerOfItem(tree, item)?;
    let first = FocusgroupItemInSegment(tree, item)?;
    Some(FocusgroupSegment { owner, first })
}

pub fn GetDirectionalKeyHandlerRootForFocusedElement(tree: &dyn FocusgroupTree) -> Option<Node> {
    if !tree.Enabled() {
        return None;
    }
    let focused = tree.FocusedElement()?;
    let owner = FindFocusgroupOwner(tree, focused)?;
    let data = tree.Data(owner);
    if !data.IsActualFocusgroup() {
        return None;
    }
    let all_axes = FocusgroupFlags::kInline | FocusgroupFlags::kBlock;
    let axes = if data.flags.Intersects(all_axes) {
        data.flags & all_axes
    } else {
        all_axes
    };
    let mut current = Some(focused);
    while let Some(node) = current.filter(|&node| node != owner) {
        if tree.NativeArrowKeyAxes(node).Intersects(axes) {
            return Some(node);
        }
        current = tree.ParentElement(node);
    }
    None
}
