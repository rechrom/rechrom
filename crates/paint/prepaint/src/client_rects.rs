//! DOM box geometry does not need clips, display items, outlines or effects.
//! Keep physical coordinate mapping separate from logical paint traversal order.
#![allow(non_snake_case)]
use crate::pre_paint_tree_walk::{
    ChildGeometryOrigin, FlatToWorldSpaceOffset, ResolveChildGeometryOrigin,
    ResolveFragmentGeometry, SvgViewBoxTransform,
};
use layoutng_assembly::fragment_tree::{FragmentKind, FragmentPaintData};
use layoutng_assembly::internal::layout_input::{Offset, Position, TransformMatrix};
use layoutng_assembly::layout_engine::NativePaintFragmentCursor;
use paint_model::geometry_mapper::{MapRectToRoot, MultiplyTransforms, TranslationTransform};
use paint_model::PaintRect;
use std::collections::HashMap;

const NONE: usize = usize::MAX;
// One contiguous index, with no Box or child/transform vector per fragment.
// The links exist only to retain the established logical fragment rect order.
struct GeometryNode<'a> {
    fragment: GeometryFragmentSource<'a>,
    absolute_offset: Offset,
    rect: Option<PaintRect>,
    parent: usize,
    first_child: usize,
    last_child: usize,
    next_sibling: usize,
    physical_subtree_end: usize,
}

#[derive(Clone, Copy)]
struct GeometryFragmentSource<'a>(NativePaintFragmentCursor<'a>);

impl<'a> GeometryFragmentSource<'a> {
    fn Children(self) -> Vec<Self> {
        self.0.Children().into_iter().map(Self).collect()
    }
}

impl std::ops::Deref for GeometryFragmentSource<'_> {
    type Target = FragmentPaintData;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// CSSOM line boxes are not paint children. Keep their native enumeration
// order while using the physical container's established coordinate mapping.
struct InlineRectMapper<'a> {
    by_container: HashMap<
        u64,
        Vec<(
            u64,
            usize,
            &'a layoutng_assembly::fragment_tree::FragmentPaintRect,
        )>,
    >,
    rects: HashMap<u64, Vec<Option<PaintRect>>>,
}
fn Gather<'a>(
    fragment: GeometryFragmentSource<'a>,
    parent_offset: Offset,
    paint_space_origin: Offset,
    transforms: &mut Vec<TransformMatrix>,
    property_world: TransformMatrix,
    parent: usize,
    nodes: &mut Vec<GeometryNode<'a>>,
    inline: &mut InlineRectMapper<'a>,
) -> usize {
    let inherited = transforms.len();
    let delta = FlatToWorldSpaceOffset(transforms, &property_world).unwrap_or_default();
    let geometry = ResolveFragmentGeometry(
        &fragment,
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
        // LayoutSVGHiddenContainer overrides QuadsInAncestorInternal with
        // no quads. Its descendants still expose their own SVG geometry.
        rect: (fragment.kind == FragmentKind::kBox && !fragment.paint.svg_hidden_container)
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
    let child_origin = ChildGeometryOrigin(&fragment, geometry.absolute_offset);
    let svg_view_box = SvgViewBoxTransform(&fragment);
    let local = transforms.len();
    if let Some(matrix) = svg_view_box {
        transforms.push(matrix);
        child_property_world = MultiplyTransforms(&child_property_world, &matrix);
    }
    if let Some(rects) = inline.by_container.get(&fragment.fragment_instance_id) {
        for &(id, index, rect) in rects {
            inline.rects.get_mut(&id).unwrap()[index] = MapRectToRoot(
                PaintRect {
                    x: child_origin.x + rect.offset.x - geometry.paint_space_origin.x,
                    y: child_origin.y + rect.offset.y - geometry.paint_space_origin.y,
                    width: rect.size.width,
                    height: rect.size.height,
                },
                transforms,
            );
        }
    }
    for child in fragment.Children() {
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
                inline,
            )
        } else {
            let origin =
                ResolveChildGeometryOrigin(&child, child_origin, geometry.paint_space_origin);
            Gather(
                child,
                origin,
                geometry.paint_space_origin,
                transforms,
                child_property_world,
                index,
                nodes,
                inline,
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

pub fn CollectNativeFragmentClientRects(
    root: NativePaintFragmentCursor<'_>,
) -> HashMap<u64, Vec<PaintRect>> {
    CollectFragmentClientRectsFromSource(GeometryFragmentSource(root))
}

fn CollectFragmentClientRectsFromSource(
    root: GeometryFragmentSource<'_>,
) -> HashMap<u64, Vec<PaintRect>> {
    let mut nodes = Vec::new();
    let mut inline = InlineRectMapper {
        by_container: HashMap::new(),
        rects: HashMap::new(),
    };
    Gather(
        root,
        Offset::default(),
        Offset::default(),
        &mut Vec::new(),
        TransformMatrix::default(),
        NONE,
        &mut nodes,
        &mut inline,
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
    for (id, rects) in inline.rects {
        output.insert(id, rects.into_iter().flatten().collect());
    }
    // cpp: document.cc:AdjustRectForScrollAndAbsoluteZoom;
    // cpp: layout/adjust_for_absolute_zoom.h. Client quads are collected in
    // LayoutZoomFactor coordinates, then exposed to script in CSS pixels. CSS
    // `zoom` remains represented by the geometry itself; only the frame-level
    // absolute layout zoom is removed here.
    let layout_zoom = root
        .paint
        .resources
        .as_ref()
        .map_or(1.0, |resources| resources.layout_zoom_factor)
        .max(f64::MIN_POSITIVE);
    if layout_zoom != 1.0 {
        for rects in output.values_mut() {
            for rect in rects {
                rect.x /= layout_zoom;
                rect.y /= layout_zoom;
                rect.width /= layout_zoom;
                rect.height /= layout_zoom;
            }
        }
    }
    output
}
