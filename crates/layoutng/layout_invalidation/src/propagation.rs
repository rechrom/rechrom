//! Tree-agnostic invalidation propagation.
//!
//! The resident `LayoutObject` graph stays private to the LayoutNG assembly.
//! Its adapter implements these traits, while the traversal policy and its
//! profiler symbols remain owned by this package.

pub trait AnchorInvalidationTree {
    type Node: Copy;

    fn parent(&self, node: Self::Node) -> Option<Self::Node>;
    fn may_contain_anchor(&self, node: Self::Node) -> bool;
    fn mark_may_contain_anchor(&mut self, node: Self::Node);
}

#[inline(never)]
pub fn propagate_may_contain_anchor<T: AnchorInvalidationTree>(
    tree: &mut T,
    start: T::Node,
) -> usize {
    let mut marked = 0;
    let mut current = Some(start);
    while let Some(node) = current {
        if tree.may_contain_anchor(node) {
            break;
        }
        tree.mark_may_contain_anchor(node);
        marked += 1;
        current = tree.parent(node);
    }
    marked
}

pub trait InlineCollectionInvalidationTree {
    type Node: Copy;

    fn parent(&self, node: Self::Node) -> Option<Self::Node>;
    fn needs_collect_inlines(&self, node: Self::Node) -> bool;
    fn mark_needs_collect_inlines(&mut self, node: Self::Node);
    fn is_layout_inline(&self, node: Self::Node) -> bool;
}

#[inline(never)]
pub fn propagate_needs_collect_inlines<T: InlineCollectionInvalidationTree>(
    tree: &mut T,
    start: T::Node,
) -> usize {
    let mut marked = 0;
    let mut current = Some(start);
    while let Some(node) = current {
        if tree.needs_collect_inlines(node) {
            break;
        }
        tree.mark_needs_collect_inlines(node);
        marked += 1;
        if !tree.is_layout_inline(node) {
            break;
        }
        current = tree.parent(node);
    }
    marked
}

pub trait IntrinsicWidthInvalidationTree {
    type Node: Copy;

    fn container(&self, node: Self::Node) -> Option<Self::Node>;
    fn intrinsic_widths_dirty(&self, node: Self::Node) -> bool;
    fn should_invalidate_beyond(&self, node: Self::Node) -> bool;
    fn is_layout_view(&self, node: Self::Node) -> bool;
    fn mark_intrinsic_widths_dirty(&mut self, node: Self::Node);
    fn is_out_of_flow_positioned(&self, node: Self::Node) -> bool;
}

#[inline(never)]
pub fn propagate_intrinsic_widths_dirty<T: IntrinsicWidthInvalidationTree>(
    tree: &mut T,
    source: T::Node,
) -> usize {
    let mut marked = 0;
    let mut current = tree.container(source);
    while let Some(node) = current {
        if tree.intrinsic_widths_dirty(node) && !tree.should_invalidate_beyond(node) {
            break;
        }
        let container = tree.container(node);
        if container.is_none() && !tree.is_layout_view(node) {
            break;
        }
        tree.mark_intrinsic_widths_dirty(node);
        marked += 1;
        if tree.is_out_of_flow_positioned(node) {
            break;
        }
        current = container;
    }
    marked
}

pub trait ContainerChainInvalidationTree {
    type Node: Copy;

    fn container(&self, node: Self::Node) -> Option<Self::Node>;
    fn containing_block(&self, node: Self::Node) -> Option<Self::Node>;
    fn self_needs_full_layout(&self, node: Self::Node) -> bool;
    fn child_needs_full_layout(&self, node: Self::Node) -> bool;
    fn mark_child_needs_full_layout(&mut self, node: Self::Node);
    fn needs_simplified_layout(&self, node: Self::Node) -> bool;
    fn needs_simplified_layout_only(&self, node: Self::Node) -> bool;
    fn mark_needs_simplified_layout(&mut self, node: Self::Node);
    fn is_text_or_svg_child(&self, node: Self::Node) -> bool;
    fn is_out_of_flow_positioned(&self, node: Self::Node) -> bool;
}

/// Mark the nearest container chain for either full-child or simplified
/// layout. Out-of-flow descendants jump to their containing block and switch
/// the remainder of the chain to simplified layout, matching LayoutNG's
/// native invalidation contract.
#[inline(never)]
pub fn propagate_container_chain_for_layout<T: ContainerChainInvalidationTree>(
    tree: &mut T,
    start: T::Node,
) -> usize {
    let mut marked = 0;
    let mut current = tree.container(start);
    let mut last = start;
    let mut simplified = tree.needs_simplified_layout_only(start);
    while let Some(mut node) = current {
        if tree.self_needs_full_layout(node) {
            break;
        }
        let mut container = tree.container(node);
        if !tree.is_text_or_svg_child(last) && tree.is_out_of_flow_positioned(last) {
            let Some(containing_block) = tree.containing_block(last) else {
                break;
            };
            node = containing_block;
            container = tree.container(node);
            simplified = true;
        }
        if simplified {
            if tree.needs_simplified_layout(node) {
                break;
            }
            tree.mark_needs_simplified_layout(node);
        } else {
            if tree.child_needs_full_layout(node) {
                break;
            }
            tree.mark_child_needs_full_layout(node);
        }
        marked += 1;
        last = node;
        current = container;
    }
    marked
}

pub trait MulticolDescendantTree {
    type Node: Copy;

    fn is_inside_multicol(&self, node: Self::Node) -> bool;
    fn set_inside_multicol(&mut self, node: Self::Node, value: bool);
    fn is_multicol_container(&self, node: Self::Node) -> bool;
    fn next_in_preorder(&self, node: Self::Node, root: Self::Node) -> Option<Self::Node>;
    fn next_after_children(&self, node: Self::Node, root: Self::Node) -> Option<Self::Node>;
}

#[inline(never)]
pub fn propagate_inside_multicol<T: MulticolDescendantTree>(
    tree: &mut T,
    root: T::Node,
    inside_multicol: bool,
) -> usize {
    let mut updated = 0;
    let mut current = Some(root);
    while let Some(node) = current {
        let was_inside_multicol = tree.is_inside_multicol(node);
        tree.set_inside_multicol(node, inside_multicol);
        updated += 1;
        current = if inside_multicol == was_inside_multicol || tree.is_multicol_container(node) {
            tree.next_after_children(node, root)
        } else {
            tree.next_in_preorder(node, root)
        };
    }
    updated
}

pub trait SubtreeChangeListenerTree {
    type Node: Copy;

    fn listener_registered(&self, node: Self::Node) -> bool;
    fn set_listener_registered(&mut self, node: Self::Node, value: bool);
    fn first_child(&self, node: Self::Node) -> Option<Self::Node>;
    fn next_sibling(&self, node: Self::Node) -> Option<Self::Node>;
}

#[inline(never)]
pub fn propagate_subtree_change_listener<T: SubtreeChangeListenerTree>(
    tree: &mut T,
    root: T::Node,
    value: bool,
) -> usize {
    fn visit<T: SubtreeChangeListenerTree>(tree: &mut T, node: T::Node, value: bool) -> usize {
        if tree.listener_registered(node) == value {
            return 0;
        }
        tree.set_listener_registered(node, value);
        let mut updated = 1;
        let mut child = tree.first_child(node);
        while let Some(current) = child {
            updated += visit(tree, current, value);
            child = tree.next_sibling(current);
        }
        updated
    }

    visit(tree, root, value)
}

pub trait PaintInvalidationTree {
    type Node: Copy;

    fn parent(&self, node: Self::Node) -> Option<Self::Node>;
    fn should_check_for_paint_invalidation(&self, node: Self::Node) -> bool;
    fn should_check_layout_for_paint_invalidation(&self, node: Self::Node) -> bool;
    fn descendant_should_check_layout_for_paint_invalidation(&self, node: Self::Node) -> bool;
    fn mark_layout_paint_invalidation(&mut self, node: Self::Node);
    fn mark_descendant_layout_paint_invalidation(&mut self, node: Self::Node);
    fn mark_paint_invalidation(&mut self, node: Self::Node);
}

/// Mark a layout-caused paint invalidation and propagate the distinct
/// descendant-layout bit through ancestors until an existing proof is found.
#[inline(never)]
pub fn propagate_layout_paint_invalidation<T: PaintInvalidationTree>(
    tree: &mut T,
    source: T::Node,
) -> usize {
    if tree.should_check_layout_for_paint_invalidation(source) {
        debug_assert!(tree.should_check_for_paint_invalidation(source));
        return 0;
    }
    tree.mark_layout_paint_invalidation(source);
    let mut marked = 1;
    let mut ancestor = tree.parent(source);
    while let Some(node) = ancestor {
        if tree.descendant_should_check_layout_for_paint_invalidation(node) {
            break;
        }
        tree.mark_descendant_layout_paint_invalidation(node);
        marked += 1;
        ancestor = tree.parent(node);
    }
    marked
}

/// Mark a paint-only invalidation. Ancestors need only the aggregate paint
/// check bit, so propagation stops at the first already-marked node.
#[inline(never)]
pub fn propagate_paint_invalidation<T: PaintInvalidationTree>(
    tree: &mut T,
    source: T::Node,
) -> usize {
    if tree.should_check_for_paint_invalidation(source) {
        return 0;
    }
    tree.mark_paint_invalidation(source);
    let mut marked = 1;
    let mut ancestor = tree.parent(source);
    while let Some(node) = ancestor {
        if tree.should_check_for_paint_invalidation(node) {
            break;
        }
        tree.mark_paint_invalidation(node);
        marked += 1;
        ancestor = tree.parent(node);
    }
    marked
}

pub trait SpannerParentInvalidationTree {
    type Node: Copy + PartialEq;

    fn parent(&self, node: Self::Node) -> Option<Self::Node>;
    fn containing_block(&self, node: Self::Node) -> Option<Self::Node>;
    fn container(&self, node: Self::Node) -> Option<Self::Node>;
    fn mark_child_layout_only(&mut self, node: Self::Node);
    fn mark_child_layout_chain(&mut self, node: Self::Node);
}

/// Mark the parent-to-containing-block path after a spanner or out-of-flow
/// positioning change. Intermediate containers are marked locally; the
/// containing block performs the ordinary container-chain propagation.
#[inline(never)]
pub fn propagate_spanner_or_oof_parent_change<T: SpannerParentInvalidationTree>(
    tree: &mut T,
    source: T::Node,
) -> usize {
    let Some(mut current) = tree.parent(source) else {
        return 0;
    };
    let containing_block = tree.containing_block(source);
    let mut marked = 0;
    while Some(current) != containing_block {
        tree.mark_child_layout_only(current);
        marked += 1;
        let Some(container) = tree.container(current) else {
            return marked;
        };
        current = container;
    }
    if let Some(containing_block) = containing_block {
        tree.mark_child_layout_chain(containing_block);
        marked += 1;
    }
    marked
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockTree {
        parents: Vec<Option<usize>>,
        anchors: Vec<bool>,
        collect: Vec<bool>,
        inline: Vec<bool>,
        dirty: Vec<bool>,
        beyond: Vec<bool>,
        views: Vec<bool>,
        out_of_flow: Vec<bool>,
        containing_blocks: Vec<Option<usize>>,
        self_full: Vec<bool>,
        child_full: Vec<bool>,
        simplified: Vec<bool>,
        text_or_svg_child: Vec<bool>,
        inside_multicol: Vec<bool>,
        multicol_container: Vec<bool>,
        next_preorder: Vec<Option<usize>>,
        next_after_children: Vec<Option<usize>>,
        paint_check: Vec<bool>,
        layout_paint_check: Vec<bool>,
        descendant_layout_paint_check: Vec<bool>,
        local_child_layout_marks: Vec<bool>,
        chained_child_layout_marks: Vec<bool>,
    }

    impl AnchorInvalidationTree for MockTree {
        type Node = usize;

        fn parent(&self, node: usize) -> Option<usize> {
            self.parents[node]
        }

        fn may_contain_anchor(&self, node: usize) -> bool {
            self.anchors[node]
        }

        fn mark_may_contain_anchor(&mut self, node: usize) {
            self.anchors[node] = true;
        }
    }

    impl InlineCollectionInvalidationTree for MockTree {
        type Node = usize;

        fn parent(&self, node: usize) -> Option<usize> {
            self.parents[node]
        }

        fn needs_collect_inlines(&self, node: usize) -> bool {
            self.collect[node]
        }

        fn mark_needs_collect_inlines(&mut self, node: usize) {
            self.collect[node] = true;
        }

        fn is_layout_inline(&self, node: usize) -> bool {
            self.inline[node]
        }
    }

    impl IntrinsicWidthInvalidationTree for MockTree {
        type Node = usize;

        fn container(&self, node: usize) -> Option<usize> {
            self.parents[node]
        }

        fn intrinsic_widths_dirty(&self, node: usize) -> bool {
            self.dirty[node]
        }

        fn should_invalidate_beyond(&self, node: usize) -> bool {
            self.beyond[node]
        }

        fn is_layout_view(&self, node: usize) -> bool {
            self.views[node]
        }

        fn mark_intrinsic_widths_dirty(&mut self, node: usize) {
            self.dirty[node] = true;
        }

        fn is_out_of_flow_positioned(&self, node: usize) -> bool {
            self.out_of_flow[node]
        }
    }

    impl ContainerChainInvalidationTree for MockTree {
        type Node = usize;

        fn container(&self, node: usize) -> Option<usize> {
            self.parents[node]
        }

        fn containing_block(&self, node: usize) -> Option<usize> {
            self.containing_blocks[node]
        }

        fn self_needs_full_layout(&self, node: usize) -> bool {
            self.self_full[node]
        }

        fn child_needs_full_layout(&self, node: usize) -> bool {
            self.child_full[node]
        }

        fn mark_child_needs_full_layout(&mut self, node: usize) {
            self.child_full[node] = true;
        }

        fn needs_simplified_layout(&self, node: usize) -> bool {
            self.simplified[node]
        }

        fn needs_simplified_layout_only(&self, node: usize) -> bool {
            self.simplified[node] && !self.self_full[node] && !self.child_full[node]
        }

        fn mark_needs_simplified_layout(&mut self, node: usize) {
            self.simplified[node] = true;
        }

        fn is_text_or_svg_child(&self, node: usize) -> bool {
            self.text_or_svg_child[node]
        }

        fn is_out_of_flow_positioned(&self, node: usize) -> bool {
            self.out_of_flow[node]
        }
    }

    impl MulticolDescendantTree for MockTree {
        type Node = usize;

        fn is_inside_multicol(&self, node: usize) -> bool {
            self.inside_multicol[node]
        }

        fn set_inside_multicol(&mut self, node: usize, value: bool) {
            self.inside_multicol[node] = value;
        }

        fn is_multicol_container(&self, node: usize) -> bool {
            self.multicol_container[node]
        }

        fn next_in_preorder(&self, node: usize, _root: usize) -> Option<usize> {
            self.next_preorder[node]
        }

        fn next_after_children(&self, node: usize, _root: usize) -> Option<usize> {
            self.next_after_children[node]
        }
    }

    impl PaintInvalidationTree for MockTree {
        type Node = usize;

        fn parent(&self, node: usize) -> Option<usize> {
            self.parents[node]
        }

        fn should_check_for_paint_invalidation(&self, node: usize) -> bool {
            self.paint_check[node]
        }

        fn should_check_layout_for_paint_invalidation(&self, node: usize) -> bool {
            self.layout_paint_check[node]
        }

        fn descendant_should_check_layout_for_paint_invalidation(&self, node: usize) -> bool {
            self.descendant_layout_paint_check[node]
        }

        fn mark_layout_paint_invalidation(&mut self, node: usize) {
            self.paint_check[node] = true;
            self.layout_paint_check[node] = true;
        }

        fn mark_descendant_layout_paint_invalidation(&mut self, node: usize) {
            self.paint_check[node] = true;
            self.descendant_layout_paint_check[node] = true;
        }

        fn mark_paint_invalidation(&mut self, node: usize) {
            self.paint_check[node] = true;
        }
    }

    impl SpannerParentInvalidationTree for MockTree {
        type Node = usize;

        fn parent(&self, node: usize) -> Option<usize> {
            self.parents[node]
        }

        fn containing_block(&self, node: usize) -> Option<usize> {
            self.containing_blocks[node]
        }

        fn container(&self, node: usize) -> Option<usize> {
            self.parents[node]
        }

        fn mark_child_layout_only(&mut self, node: usize) {
            self.local_child_layout_marks[node] = true;
        }

        fn mark_child_layout_chain(&mut self, node: usize) {
            self.chained_child_layout_marks[node] = true;
        }
    }

    fn chain() -> MockTree {
        MockTree {
            parents: vec![Some(1), Some(2), None],
            anchors: vec![false; 3],
            collect: vec![false; 3],
            inline: vec![true, true, false],
            dirty: vec![false; 3],
            beyond: vec![false; 3],
            views: vec![false, false, true],
            out_of_flow: vec![false; 3],
            containing_blocks: vec![Some(1), Some(2), None],
            self_full: vec![false; 3],
            child_full: vec![false; 3],
            simplified: vec![false; 3],
            text_or_svg_child: vec![false; 3],
            inside_multicol: vec![false; 3],
            multicol_container: vec![false; 3],
            next_preorder: vec![Some(1), Some(2), None],
            next_after_children: vec![None; 3],
            paint_check: vec![false; 3],
            layout_paint_check: vec![false; 3],
            descendant_layout_paint_check: vec![false; 3],
            local_child_layout_marks: vec![false; 3],
            chained_child_layout_marks: vec![false; 3],
        }
    }

    #[test]
    fn anchor_propagation_stops_at_an_existing_mark() {
        let mut tree = chain();
        tree.anchors[2] = true;
        assert_eq!(propagate_may_contain_anchor(&mut tree, 0), 2);
        assert_eq!(tree.anchors, [true, true, true]);
    }

    #[test]
    fn inline_collection_stops_at_the_first_non_inline_owner() {
        let mut tree = chain();
        assert_eq!(propagate_needs_collect_inlines(&mut tree, 0), 3);
        assert_eq!(tree.collect, [true, true, true]);
    }

    #[test]
    fn intrinsic_width_propagation_crosses_special_dirty_nodes() {
        let mut tree = chain();
        tree.dirty[1] = true;
        assert_eq!(propagate_intrinsic_widths_dirty(&mut tree, 0), 0);

        tree.beyond[1] = true;
        assert_eq!(propagate_intrinsic_widths_dirty(&mut tree, 0), 2);
        assert_eq!(tree.dirty, [false, true, true]);
    }

    #[test]
    fn intrinsic_width_propagation_stops_after_out_of_flow_container() {
        let mut tree = chain();
        tree.out_of_flow[1] = true;
        assert_eq!(propagate_intrinsic_widths_dirty(&mut tree, 0), 1);
        assert_eq!(tree.dirty, [false, true, false]);
    }

    #[test]
    fn container_chain_marks_ancestors_until_an_existing_mark() {
        let mut tree = chain();
        tree.child_full[2] = true;
        assert_eq!(propagate_container_chain_for_layout(&mut tree, 0), 1);
        assert_eq!(tree.child_full, [false, true, true]);
    }

    #[test]
    fn simplified_source_keeps_the_container_chain_simplified() {
        let mut tree = chain();
        tree.simplified[0] = true;
        assert_eq!(propagate_container_chain_for_layout(&mut tree, 0), 2);
        assert_eq!(tree.simplified, [true, true, true]);
    }

    #[test]
    fn out_of_flow_source_jumps_to_its_containing_block() {
        let mut tree = chain();
        tree.parents[0] = Some(1);
        tree.containing_blocks[0] = Some(2);
        tree.out_of_flow[0] = true;
        assert_eq!(propagate_container_chain_for_layout(&mut tree, 0), 1);
        assert_eq!(tree.child_full, [false, false, false]);
        assert_eq!(tree.simplified, [false, false, true]);
    }

    #[test]
    fn multicol_propagation_skips_nested_multicol_descendants() {
        let mut tree = chain();
        tree.multicol_container[1] = true;
        assert_eq!(propagate_inside_multicol(&mut tree, 0, true), 2);
        assert_eq!(tree.inside_multicol, [true, true, false]);
    }

    struct SubtreeMock {
        registered: [bool; 4],
        first_child: [Option<usize>; 4],
        next_sibling: [Option<usize>; 4],
    }

    impl SubtreeChangeListenerTree for SubtreeMock {
        type Node = usize;

        fn listener_registered(&self, node: usize) -> bool {
            self.registered[node]
        }

        fn set_listener_registered(&mut self, node: usize, value: bool) {
            self.registered[node] = value;
        }

        fn first_child(&self, node: usize) -> Option<usize> {
            self.first_child[node]
        }

        fn next_sibling(&self, node: usize) -> Option<usize> {
            self.next_sibling[node]
        }
    }

    #[test]
    fn subtree_listener_propagation_visits_each_changed_branch() {
        let mut tree = SubtreeMock {
            registered: [false; 4],
            first_child: [Some(1), Some(3), None, None],
            next_sibling: [None, Some(2), None, None],
        };
        assert_eq!(propagate_subtree_change_listener(&mut tree, 0, true), 4);
        assert_eq!(tree.registered, [true; 4]);
        assert_eq!(propagate_subtree_change_listener(&mut tree, 0, true), 0);
    }

    #[test]
    fn layout_paint_invalidation_marks_distinct_descendant_bits() {
        let mut tree = chain();
        assert_eq!(propagate_layout_paint_invalidation(&mut tree, 0), 3);
        assert_eq!(tree.paint_check, [true, true, true]);
        assert_eq!(tree.layout_paint_check, [true, false, false]);
        assert_eq!(tree.descendant_layout_paint_check, [false, true, true]);
        assert_eq!(propagate_layout_paint_invalidation(&mut tree, 0), 0);
    }

    #[test]
    fn paint_only_invalidation_stops_at_existing_ancestor_mark() {
        let mut tree = chain();
        tree.paint_check[2] = true;
        assert_eq!(propagate_paint_invalidation(&mut tree, 0), 2);
        assert_eq!(tree.paint_check, [true, true, true]);
        assert!(tree.layout_paint_check.iter().all(|marked| !marked));
    }

    #[test]
    fn spanner_parent_walk_marks_intermediates_before_the_containing_block() {
        let mut tree = chain();
        tree.containing_blocks[0] = Some(2);
        assert_eq!(propagate_spanner_or_oof_parent_change(&mut tree, 0), 2);
        assert_eq!(tree.local_child_layout_marks, [false, true, false]);
        assert_eq!(tree.chained_child_layout_marks, [false, false, true]);
    }
}
