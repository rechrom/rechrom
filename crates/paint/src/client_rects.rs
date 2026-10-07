//! DOM box geometry does not need clips, display items, outlines or effects.
//! Keep physical coordinate mapping separate from logical paint traversal order.
#![allow(non_snake_case)]
use crate::geometry_mapper::{MapRectToRoot, MultiplyTransforms, TranslationTransform};
use crate::pre_paint_tree_walk::{
    ChildGeometryOrigin, FlatToWorldSpaceOffset, ResolveChildGeometryOrigin,
    ResolveFragmentGeometry, SvgViewBoxTransform,
};
use crate::PaintRect;
use layoutng_assembly::fragment_tree::{FragmentKind, FragmentNode};
use layoutng_assembly::internal::layout_input::{Offset, Position, TransformMatrix};
use std::collections::HashMap;

const NONE: usize = usize::MAX;
// One contiguous index, with no Box or child/transform vector per fragment.
// The links exist only to retain the established logical fragment rect order.
struct GeometryNode<'a> {
    fragment: &'a FragmentNode,
    absolute_offset: Offset,
    rect: Option<PaintRect>,
    parent: usize,
    first_child: usize,
    last_child: usize,
    next_sibling: usize,
    physical_subtree_end: usize,
}

fn Gather<'a>(
    fragment: &'a FragmentNode,
    parent_offset: Offset,
    paint_space_origin: Offset,
    transforms: &mut Vec<TransformMatrix>,
    property_world: TransformMatrix,
    parent: usize,
    nodes: &mut Vec<GeometryNode<'a>>,
) -> usize {
    let inherited = transforms.len();
    let delta = FlatToWorldSpaceOffset(transforms, &property_world).unwrap_or_default();
    let geometry = ResolveFragmentGeometry(
        fragment,
        parent_offset,
        paint_space_origin,
        transforms,
        delta,
    );
    // The lightweight geometry walk keeps the same real scroll coordinate
    // spaces as PrePaint without constructing clips, effects or property nodes.
    let mut child_property_world = if geometry.resets_paint_space {
        let mut world = TransformMatrix::default();
        for matrix in transforms.iter() {
            world = MultiplyTransforms(&world, matrix);
        }
        world
    } else {
        let mut world = property_world;
        for matrix in &transforms[inherited..] {
            world = MultiplyTransforms(&world, matrix);
        }
        world
    };
    if fragment.paint.establishes_paint_state {
        child_property_world = MultiplyTransforms(
            &child_property_world,
            &TranslationTransform(
                -fragment.paint.scroll_offset.x,
                -fragment.paint.scroll_offset.y,
            ),
        );
    }
    let index = nodes.len();
    nodes.push(GeometryNode {
        fragment,
        absolute_offset: geometry.absolute_offset,
        rect: (fragment.kind == FragmentKind::kBox)
            .then(|| {
                MapRectToRoot(
                    PaintRect {
                        x: geometry.paint_offset.x,
                        y: geometry.paint_offset.y,
                        width: fragment.size.width,
                        height: fragment.size.height,
                    },
                    transforms,
                )
            })
            .flatten(),
        parent,
        first_child: NONE,
        last_child: NONE,
        next_sibling: NONE,
        physical_subtree_end: 0,
    });
    let child_origin = ChildGeometryOrigin(fragment, geometry.absolute_offset);
    let svg_view_box = SvgViewBoxTransform(fragment);
    let local = transforms.len();
    if let Some(matrix) = svg_view_box {
        transforms.push(matrix);
        child_property_world = MultiplyTransforms(&child_property_world, &matrix);
    }
    for child in &fragment.children {
        let child_index = if child.paint.fixed_to_view {
            // Match PrePaint's LayoutView fixed-position context. Client rects
            // are viewport-relative and therefore must not inherit document
            // scroll geometry either.
            Gather(
                child,
                Offset::default(),
                Offset::default(),
                &mut Vec::new(),
                TransformMatrix::default(),
                index,
                nodes,
            )
        } else {
            let origin =
                ResolveChildGeometryOrigin(child, child_origin, geometry.paint_space_origin);
            Gather(
                child,
                origin,
                geometry.paint_space_origin,
                transforms,
                child_property_world,
                index,
                nodes,
            )
        };
        AppendChild(nodes, index, child_index);
    }
    transforms.truncate(local);
    nodes[index].physical_subtree_end = nodes.len();
    transforms.truncate(inherited);
    index
}

fn AppendChild(nodes: &mut [GeometryNode<'_>], parent: usize, child: usize) {
    let previous = nodes[parent].last_child;
    if previous == NONE {
        nodes[parent].first_child = child;
    } else {
        nodes[previous].next_sibling = child;
    }
    nodes[parent].last_child = child;
    nodes[child].parent = parent;
    nodes[child].next_sibling = NONE;
}

fn FirstLogicalTreeOrder(nodes: &[GeometryNode<'_>], index: usize) -> u64 {
    let order = nodes[index].fragment.paint.logical_tree_order;
    if order != 0 {
        return order;
    }
    let mut first = u64::MAX;
    let mut child = nodes[index].first_child;
    while child != NONE {
        let order = FirstLogicalTreeOrder(nodes, child);
        if order != 0 {
            first = first.min(order);
        }
        child = nodes[child].next_sibling;
    }
    if first == u64::MAX {
        0
    } else {
        first
    }
}

fn DistanceSquared(nodes: &[GeometryNode<'_>], candidate: usize, child: usize) -> f64 {
    let left = nodes[candidate].absolute_offset.x;
    let top = nodes[candidate].absolute_offset.y;
    let right = left + nodes[candidate].fragment.size.width;
    let bottom = top + nodes[candidate].fragment.size.height;
    let x = nodes[child].absolute_offset.x;
    let y = nodes[child].absolute_offset.y;
    let dx = if x < left {
        left - x
    } else if x > right {
        x - right
    } else {
        0.0
    };
    let dy = if y < top {
        top - y
    } else if y > bottom {
        y - bottom
    } else {
        0.0
    };
    dx * dx + dy * dy
}

fn LogicalParentOccurrence(
    nodes: &[GeometryNode<'_>],
    child: usize,
    candidates: &[usize],
) -> Option<usize> {
    let fragmentainer = nodes[child].fragment.fragmentainer_instance_id;
    let same_fragmentainer = candidates
        .iter()
        .any(|&index| nodes[index].fragment.fragmentainer_instance_id == fragmentainer);
    let mut smallest = None;
    for &candidate in candidates {
        if same_fragmentainer
            && nodes[candidate].fragment.fragmentainer_instance_id != fragmentainer
        {
            continue;
        }
        smallest = Some(match smallest {
            None => candidate,
            Some(previous) => {
                let distance = DistanceSquared(nodes, candidate, child);
                let previous_distance = DistanceSquared(nodes, previous, child);
                let precedes = if distance != previous_distance {
                    distance < previous_distance
                } else {
                    nodes[candidate].fragment.fragment_instance_id
                        < nodes[previous].fragment.fragment_instance_id
                };
                if precedes {
                    candidate
                } else {
                    previous
                }
            }
        });
    }
    smallest
}

fn RestoreLogicalOutOfFlowParents(nodes: &mut [GeometryNode<'_>]) {
    let mut source_nodes: HashMap<u64, Vec<usize>> = HashMap::new();
    let mut instances = HashMap::new();
    for (index, node) in nodes.iter().enumerate() {
        let fragment = node.fragment;
        if fragment.fragment_instance_id != 0 {
            assert!(
                instances
                    .insert(fragment.fragment_instance_id, index)
                    .is_none(),
                "FragmentTree contains duplicate fragment_instance_id values"
            );
        }
        if fragment.node_id != 0
            && fragment.paint.has_source
            && fragment.paint.establishes_paint_state
        {
            source_nodes
                .entry(fragment.node_id)
                .or_default()
                .push(index);
        }
    }
    let mut detached = Vec::new();
    let mut index = 1;
    while index < nodes.len() {
        let fragment = nodes[index].fragment;
        let logical_id = fragment.paint.logical_parent_node_id;
        let logical_parent =
            if let Some(instance) = fragment.paint.logical_parent_fragment_instance_id {
                let exact = *instances
                    .get(&instance)
                    .expect("logical_parent_fragment_instance_id does not name a Fragment");
                if let Some(id) = logical_id {
                    assert_eq!(
                        nodes[exact].fragment.node_id, id,
                        "logical parent node id and fragment instance id disagree"
                    );
                }
                Some(exact)
            } else {
                logical_id
                    .and_then(|id| source_nodes.get(&id))
                    .and_then(|candidates| LogicalParentOccurrence(nodes, index, candidates))
            };
        let is_out_of_flow = matches!(
            fragment.paint.position,
            Position::kAbsolute | Position::kFixed
        );
        if let Some(logical) = logical_parent.filter(|&logical| {
            is_out_of_flow
                && logical_id.is_some()
                && logical != index
                && !(logical < index && index < nodes[logical].physical_subtree_end)
        }) {
            let parent = nodes[index].parent;
            let mut previous = NONE;
            let mut child = nodes[parent].first_child;
            while child != index {
                previous = child;
                child = nodes[child].next_sibling;
            }
            let next = nodes[index].next_sibling;
            if previous == NONE {
                nodes[parent].first_child = next;
            } else {
                nodes[previous].next_sibling = next;
            }
            if nodes[parent].last_child == index {
                nodes[parent].last_child = previous;
            }
            nodes[index].next_sibling = NONE;
            detached.push((logical, index));
            // The paint walk also leaves the detached subtree's own physical
            // geometry and parenting intact at this point.
            index = nodes[index].physical_subtree_end;
        } else {
            index += 1;
        }
    }
    for (parent, child) in detached {
        let order = FirstLogicalTreeOrder(nodes, child);
        let mut previous = NONE;
        let mut sibling = nodes[parent].first_child;
        while sibling != NONE {
            let sibling_order = FirstLogicalTreeOrder(nodes, sibling);
            if order != 0 && sibling_order != 0 && sibling_order > order {
                break;
            }
            previous = sibling;
            sibling = nodes[sibling].next_sibling;
        }
        nodes[child].parent = parent;
        nodes[child].next_sibling = sibling;
        if previous == NONE {
            nodes[parent].first_child = child;
        } else {
            nodes[previous].next_sibling = child;
        }
        if sibling == NONE {
            nodes[parent].last_child = child;
        }
    }
}

fn PrepareOrder(
    nodes: &[GeometryNode<'_>],
    index: usize,
    orders: &mut [u64],
    postorder: &mut Vec<usize>,
) -> u64 {
    let mut first = u64::MAX;
    let mut child = nodes[index].first_child;
    while child != NONE {
        let order = PrepareOrder(nodes, child, orders, postorder);
        if order != 0 {
            first = first.min(order);
        }
        child = nodes[child].next_sibling;
    }
    let own = nodes[index].fragment.paint.logical_tree_order;
    orders[index] = if own != 0 {
        own
    } else if first != u64::MAX {
        first
    } else {
        0
    };
    postorder.push(index);
    orders[index]
}

fn RestoreLogicalSiblingOrder(nodes: &mut [GeometryNode<'_>]) {
    if !nodes
        .iter()
        .any(|node| node.fragment.paint.logical_tree_order != 0)
    {
        return;
    }
    let mut orders = vec![0; nodes.len()];
    let mut postorder = Vec::with_capacity(nodes.len());
    PrepareOrder(nodes, 0, &mut orders, &mut postorder);
    let mut siblings = Vec::new();
    let mut ordered = Vec::new();
    for index in postorder {
        siblings.clear();
        ordered.clear();
        let mut child = nodes[index].first_child;
        while child != NONE {
            siblings.push(child);
            if orders[child] != 0 {
                ordered.push(child);
            }
            child = nodes[child].next_sibling;
        }
        // Preserve anonymous zero-order slots, as the paint tree does.
        ordered.sort_by_key(|&child| orders[child]);
        let mut ordered = ordered.iter();
        for child in &mut siblings {
            if orders[*child] != 0 {
                *child = *ordered.next().unwrap();
            }
        }
        nodes[index].first_child = siblings.first().copied().unwrap_or(NONE);
        nodes[index].last_child = siblings.last().copied().unwrap_or(NONE);
        for (slot, &child) in siblings.iter().enumerate() {
            nodes[child].next_sibling = siblings.get(slot + 1).copied().unwrap_or(NONE);
        }
    }
}

fn Collect(nodes: &[GeometryNode<'_>], index: usize, output: &mut HashMap<u64, Vec<PaintRect>>) {
    if let Some(rect) = nodes[index].rect {
        output
            .entry(nodes[index].fragment.node_id)
            .or_default()
            .push(rect);
    }
    let mut child = nodes[index].first_child;
    while child != NONE {
        Collect(nodes, child, output);
        child = nodes[child].next_sibling;
    }
}

pub(crate) fn CollectFragmentClientRects(root: &FragmentNode) -> HashMap<u64, Vec<PaintRect>> {
    let mut nodes = Vec::new();
    Gather(
        root,
        Offset::default(),
        Offset::default(),
        &mut Vec::new(),
        TransformMatrix::default(),
        NONE,
        &mut nodes,
    );
    if nodes.iter().any(|node| {
        node.fragment.paint.logical_parent_node_id.is_some()
            || node
                .fragment
                .paint
                .logical_parent_fragment_instance_id
                .is_some()
    }) {
        RestoreLogicalOutOfFlowParents(&mut nodes);
    }
    RestoreLogicalSiblingOrder(&mut nodes);
    let mut output = HashMap::new();
    Collect(&nodes, 0, &mut output);
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pre_paint_tree_walk::{PaintTreeNode, PrePaintTreeWalk};
    use layoutng_assembly::internal::layout_input::{NodeKind, Size, SvgShapeData, SvgViewBoxData};
    use layoutng_assembly::internal::paint_input::{PaintTransform, PaintTransformOrigin};
    use std::sync::Arc;

    fn fragment(id: u64, instance: u64, x: f64, y: f64, width: f64, height: f64) -> FragmentNode {
        let mut node = FragmentNode {
            node_id: id,
            fragment_instance_id: instance,
            offset: Offset { x, y },
            size: Size { width, height },
            ..Default::default()
        };
        node.paint.has_source = true;
        node.paint.establishes_paint_state = true;
        node
    }

    fn PaintWalkRects(root: &FragmentNode) -> HashMap<u64, Vec<PaintRect>> {
        fn visit(node: &PaintTreeNode<'_>, output: &mut HashMap<u64, Vec<PaintRect>>) {
            let fragment = node.fragment.as_deref().unwrap();
            if fragment.kind == FragmentKind::kBox {
                if let Some(rect) = MapRectToRoot(
                    PaintRect {
                        x: node.paint_offset.x,
                        y: node.paint_offset.y,
                        width: fragment.size.width,
                        height: fragment.size.height,
                    },
                    &node.transforms,
                ) {
                    output.entry(fragment.node_id).or_default().push(rect);
                }
            }
            for child in &node.children {
                visit(child, output);
            }
        }
        let tree = PrePaintTreeWalk::default().Walk(root);
        let mut output = HashMap::new();
        visit(&tree, &mut output);
        output
    }

    #[test]
    fn nested_transforms_scroll_sticky_inline_and_zoom_match_paint_walk() {
        let mut root = fragment(1, 1, 4.25, 8.5, 300.0, 200.0);
        root.paint.scroll_offset = Offset { x: 7.25, y: 3.5 };
        let mut parent = fragment(2, 2, 30.5, 20.75, 80.0, 60.0);
        let mut matrix = TransformMatrix::default();
        matrix.values[0] = 1.25;
        matrix.values[1] = 0.375;
        matrix.values[4] = -0.25;
        matrix.values[5] = 0.75;
        parent.paint.style.transform = Some(PaintTransform::from_matrix(&matrix));
        parent.paint.style.transform_origin = Some(PaintTransformOrigin {
            pixels: Offset { x: 2.25, y: 4.75 },
            percentages: Offset { x: 25.0, y: 75.0 },
            z: 0.0,
        });
        parent.paint.scroll_offset = Offset { x: 1.5, y: 2.25 };
        let mut sticky = fragment(7, 3, 12.25, 6.5, 25.0, 15.0);
        sticky.paint.position = Position::kSticky;
        sticky.paint.sticky_offset = Offset { x: -3.25, y: 2.5 };
        sticky.paint.effective_zoom = 1.75;
        let mut inline = fragment(7, 4, 3.75, 9.5, 12.0, 8.0);
        inline.paint.establishes_paint_state = false;
        inline.paint.style.transform = Some(PaintTransform::from_matrix(
            &TranslationTransformForTest(99.0, 99.0),
        ));
        let mut line = FragmentNode {
            kind: FragmentKind::kLine,
            offset: Offset { x: 1.5, y: 2.0 },
            ..Default::default()
        };
        line.children.push(inline);
        sticky.children.push(line);
        parent.children.push(sticky);
        root.children.push(parent);
        root.children.push(fragment(7, 5, 0.25, 0.75, 0.0, 0.0));
        let actual = CollectFragmentClientRects(&root);
        assert_eq!(actual, PaintWalkRects(&root));
        assert_eq!(actual[&7].len(), 3); // Multiple inline fragments and empty border boxes survive.
    }

    fn TranslationTransformForTest(x: f64, y: f64) -> TransformMatrix {
        crate::geometry_mapper::TranslationTransform(x, y)
    }

    #[test]
    fn svg_viewbox_local_transform_and_fractional_origin_have_exact_coordinates() {
        let mut svg = fragment(1, 1, 10.25, 20.25, 200.0, 100.0);
        svg.content_size = svg.size;
        svg.paint.source_kind = NodeKind::kSvgRoot;
        svg.paint.svg_view_box = Some(SvgViewBoxData {
            width: 100.0,
            height: 50.0,
            preserve_none: true,
            ..Default::default()
        });
        let mut shape = fragment(2, 2, 7.0, 8.0, 5.0, 6.0);
        shape.paint.source_kind = NodeKind::kSvgShape;
        shape.paint.svg_shape = Some(Arc::new(SvgShapeData {
            local_transform: Some(TranslationTransformForTest(3.0, 4.0)),
            ..Default::default()
        }));
        svg.children.push(shape);
        let actual = CollectFragmentClientRects(&svg);
        assert_eq!(actual, PaintWalkRects(&svg));
        assert_eq!(
            actual[&2],
            vec![PaintRect {
                x: 16.0,
                y: 28.0,
                width: 10.0,
                height: 12.0
            }]
        );
    }

    #[test]
    fn logical_oof_occurrences_and_anonymous_slots_keep_paint_preorder() {
        let mut root = fragment(1, 1, 0.0, 0.0, 400.0, 300.0);
        let mut first_parent = fragment(10, 2, 10.0, 20.0, 50.0, 50.0);
        first_parent.fragmentainer_instance_id = 1;
        first_parent.paint.logical_tree_order = 30;
        first_parent
            .children
            .push(fragment(42, 3, 1.0, 1.0, 11.0, 12.0));
        let mut second_parent = fragment(10, 4, 110.0, 20.0, 50.0, 50.0);
        second_parent.fragmentainer_instance_id = 2;
        second_parent.paint.logical_tree_order = 10;
        let mut existing = fragment(42, 5, 2.0, 2.0, 21.0, 22.0);
        existing.paint.logical_tree_order = 50;
        second_parent.children.push(existing);
        let anonymous = fragment(42, 6, 4.0, 4.0, 31.0, 32.0); // Zero-order sibling slot.
        let mut moved = fragment(42, 7, 116.0, 25.0, 41.0, 42.0);
        moved.fragmentainer_instance_id = 2;
        moved.paint.position = Position::kAbsolute;
        moved.paint.logical_parent_node_id = Some(10);
        moved.paint.logical_tree_order = 20;
        let mut exact = fragment(42, 8, 70.0, 80.0, 51.0, 52.0);
        exact.paint.position = Position::kFixed;
        exact.paint.logical_parent_node_id = Some(10);
        exact.paint.logical_parent_fragment_instance_id = Some(2);
        exact.paint.logical_tree_order = 40;
        root.children = vec![first_parent, anonymous, second_parent, moved, exact];
        let actual = CollectFragmentClientRects(&root);
        assert_eq!(actual, PaintWalkRects(&root));
        assert_eq!(
            actual[&42]
                .iter()
                .map(|rect| rect.width)
                .collect::<Vec<_>>(),
            vec![41.0, 21.0, 31.0, 11.0, 51.0]
        );
    }

    #[test]
    fn perspective_unmappable_boxes_are_skipped_without_affecting_sibling_stack() {
        let mut root = fragment(1, 1, 0.0, 0.0, 100.0, 100.0);
        let mut parent = fragment(2, 2, 5.0, 6.0, 20.0, 20.0);
        let mut matrix = TransformMatrix::default();
        matrix.values[3] = 0.01;
        matrix.values[7] = 0.02;
        parent.paint.style.transform = Some(PaintTransform::from_matrix(&matrix));
        parent.children.push(fragment(3, 3, 2.5, 3.25, 6.0, 7.0));
        let mut invalid = fragment(4, 4, 10.0, 10.0, 5.0, 5.0);
        let mut behind = TransformMatrix::default();
        behind.values[15] = -1.0;
        invalid.paint.style.transform = Some(PaintTransform::from_matrix(&behind));
        root.children = vec![parent, invalid, fragment(5, 5, 12.25, 14.5, 8.0, 9.0)];
        let actual = CollectFragmentClientRects(&root);
        assert_eq!(actual, PaintWalkRects(&root));
        assert!(!actual.contains_key(&4));
        assert_eq!(
            actual[&5],
            vec![PaintRect {
                x: 12.25,
                y: 14.5,
                width: 8.0,
                height: 9.0
            }]
        );
    }
}
