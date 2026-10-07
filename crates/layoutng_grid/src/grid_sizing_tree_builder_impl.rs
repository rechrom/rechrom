use crate::{
    grid_lanes_extension::{GridLanesSizingSubtreeRequest, GridLanesSubgriddedItemsRequest},
    grid_layout_algorithm::GridLayoutAlgorithm,
    grid_layout_utils::*,
    grid_line_resolver::GridLineResolver,
    grid_node::GridNode,
    grid_sizing_tree::{
        kNoSubgriddedItemData, GridSizingSubtree, GridSizingTree, NoGridSizingSubtree,
        SubgriddedItemData,
    },
    grid_track_sizing_algorithm::SizingConstraint,
};
use foundation::{HeapVector, MakeGarbageCollected, Member, UnsupportedLayout};
use layoutng_assembly::internal::{
    constraint_space::ConstraintSpace,
    grid_item::{GridItemData, GridItems},
    grid_lanes_item_group::VirtualItems,
    grid_layout_data::{GridLayoutData, GridLayoutTree},
    layout_algorithm::LayoutAlgorithmParams,
    layout_box::LayoutBox,
    layout_pass_scope::LayoutPassScope,
};
use layoutng_geometry::geometry::{box_strut::BoxStrut, logical_size::LogicalSize};
use layoutng_style::style::{
    grid_area::GridArea,
    grid_enums::GridTrackSizingDirection::{self, *},
};

// cpp: layoutng_grid/grid_sizing_tree_builder_impl.h:18-43
// Source template operations required by both Grid and Grid Lanes.
pub trait GridSizingAlgorithm {
    fn Node(&self) -> &GridNode;
    fn GetConstraintSpace(&self) -> &ConstraintSpace;
    fn BorderScrollbarPadding(&self) -> &BoxStrut;
    fn GetGridAvailableSize(&self) -> LogicalSize;
    fn BuildSizingCollection(
        &self,
        direction: GridTrackSizingDirection,
        resolver: &GridLineResolver,
        items: &mut GridItems,
        layout: &mut GridLayoutData,
        constraint: SizingConstraint,
        intrinsic: bool,
        virtual_items: &mut *mut VirtualItems,
    );
    fn CreateSubgridConstraintSpace(&self, item: &SubgriddedItemData) -> ConstraintSpace;
}

// cpp: layoutng_grid/grid_sizing_tree_builder_impl.h:47-108
pub fn AppendSubgriddedItems(node: &GridNode, items: &mut GridItems) {
    let root = node.Style();
    let mut i = 0;
    while i < items.Size() {
        let current = items.At(i) as *mut GridItemData;
        i += 1;
        let item = unsafe { &*current };
        if !item.must_consider_grid_items_for_column_sizing
            && !item.must_consider_grid_items_for_row_sizing
        {
            continue;
        }
        let mut invalidate = false;
        let sub = if item.node.IsGridLanes() {
            let algorithms = LayoutPassScope::Algorithms();
            let callback = if algorithms.is_null() {
                None
            } else {
                unsafe { &*algorithms }
                    .grid_lanes_support
                    .construct_subgridded_items
            }
            .unwrap_or_else(|| {
                std::panic::panic_any(UnsupportedLayout::new(
                    "Grid-lanes subgrid requires the grid-lanes package",
                ))
            });
            let request = GridLanesSubgriddedItemsRequest {
                node: &item.node,
                root_grid_style: root,
                subgrid_style: item.node.Style(),
                consider_columns: item.must_consider_grid_items_for_column_sizing,
                must_invalidate_placement_cache: &mut invalidate,
                parent_is_auto_placed: item.is_auto_placed,
            };
            callback((&request as *const GridLanesSubgriddedItemsRequest).cast())
        } else {
            let grid = GridNode::new(item.node.GetLayoutBox());
            grid.ConstructGridItemsWithStyles(
                grid.CachedLineResolver(),
                root,
                grid.Style(),
                item.must_consider_grid_items_for_column_sizing,
                item.must_consider_grid_items_for_row_sizing,
                &mut invalidate,
                item.is_auto_placed,
                None,
                None,
            )
        };
        debug_assert!(
            !invalidate,
            "cached line resolver must produce the same placement"
        );
        let child = unsafe { &mut *sub };
        let mut it = child.begin();
        let end = child.end();
        while it.NotEqual(&end) {
            let subitem = unsafe { &mut *it.Get() };
            it.Advance();
            subitem.is_subgridded_to_parent_grid = true;
            if !item.is_parallel_with_root_grid {
                std::mem::swap(
                    &mut subitem.resolved_position.columns,
                    &mut subitem.resolved_position.rows,
                );
            }
            node.AdjustSubgriddedItemSpan(item, subitem);
        }
        items.Append(sub);
    }
}

// cpp: layoutng_grid/grid_sizing_tree_builder_impl.h:111-280
pub fn BuildGridSizingSubtree<A: GridSizingAlgorithm>(
    algorithm: &A,
    resolver: &GridLineResolver,
    tree: &mut GridSizingTree,
    oof: Option<&mut HeapVector<Member<LayoutBox>>>,
    subgrid: &SubgriddedItemData,
    _parent: Option<&GridLineResolver>,
    constraint: SizingConstraint,
    mut invalidate: bool,
    ignore_children: bool,
    intrinsic: bool,
) {
    let node = algorithm.Node();
    let style = node.Style();
    tree.AddToPreorderTraversal(node);
    let space = algorithm.GetConstraintSpace();
    let mode = space.GetWritingMode();
    let mut items = MakeGarbageCollected(GridItems::default());
    let layout = MakeGarbageCollected(GridLayoutData::default());
    let mut nested = false;
    if !ignore_children {
        items = node.ConstructGridItems(
            resolver,
            &mut invalidate,
            subgrid.IsPresent() && subgrid.is_auto_placed,
            oof,
            Some(&mut nested),
        );
    }
    let area = SubgriddedAreaInParent(subgrid);
    let columns = area.columns.IsIndefinite();
    let rows = area.rows.IsIndefinite();
    let mut virtual_items = std::ptr::null_mut();
    if columns {
        algorithm.BuildSizingCollection(
            kForColumns,
            resolver,
            unsafe { &mut *items },
            unsafe { &mut *layout },
            constraint,
            intrinsic,
            &mut virtual_items,
        );
    }
    if rows {
        algorithm.BuildSizingCollection(
            kForRows,
            resolver,
            unsafe { &mut *items },
            unsafe { &mut *layout },
            constraint,
            intrinsic,
            &mut virtual_items,
        );
    }
    if !nested {
        tree.SetSizingNodeData(node, items, layout, virtual_items);
        return;
    }
    let grid_columns = style.HasGridTrackAxis(kForColumns);
    let grid_rows = style.HasGridTrackAxis(kForRows);
    if grid_columns {
        InitializeTrackCollection(
            subgrid,
            style,
            space,
            algorithm.BorderScrollbarPadding(),
            algorithm.GetGridAvailableSize(),
            kForColumns,
            unsafe { &mut *layout },
        );
    }
    if grid_rows {
        InitializeTrackCollection(
            subgrid,
            style,
            space,
            algorithm.BorderScrollbarPadding(),
            algorithm.GetGridAvailableSize(),
            kForRows,
            unsafe { &mut *layout },
        );
    }
    if columns && grid_columns {
        unsafe { &mut *(*layout).SizingCollection(kForColumns) }.CacheDefiniteSetsGeometry();
    }
    if rows && grid_rows {
        unsafe { &mut *(*layout).SizingCollection(kForRows) }.CacheDefiniteSetsGeometry();
    }
    let mut it = unsafe { &mut *items }.begin();
    let end = unsafe { &mut *items }.end();
    // The source mutating range and const end cover the same direct-item bound.
    while it.NotEqual(&end) {
        let item = unsafe { &mut *it.Get() };
        it.Advance();
        if !item.IsSubgrid() {
            continue;
        }
        node.ComputeSetIndicesForSubgrid(item, unsafe { &mut *layout });
        let data = SubgriddedItemData::new(item, layout, mode);
        let child_space = algorithm.CreateSubgridConstraintSpace(&data);
        let geometry =
            CalculateInitialFragmentGeometryForSubgrid(item, &child_space, &NoGridSizingSubtree());
        let inherit = !(style.IsDisplayGridLanes() && item.is_auto_placed);
        if item.node.IsGridLanes() {
            let algorithms = LayoutPassScope::Algorithms();
            let callback = if algorithms.is_null() {
                None
            } else {
                unsafe { &*algorithms }
                    .grid_lanes_support
                    .build_sizing_subtree
            }
            .unwrap_or_else(|| {
                std::panic::panic_any(UnsupportedLayout::new(
                    "Grid-lanes subgrid requires the grid-lanes package",
                ))
            });
            let child_area = SubgriddedAreaInParent(&data);
            let request = GridLanesSizingSubtreeRequest {
                node: &item.node,
                fragment_geometry: &geometry,
                space: &child_space,
                subgrid_area: &child_area,
                subgridded_item: &data,
                parent_line_resolver: resolver,
                can_inherit_line_names_from_parent: inherit,
                sizing_tree: tree,
                must_invalidate_placement_cache: invalidate,
            };
            callback((&request as *const GridLanesSizingSubtreeRequest).cast());
        } else {
            assert!(item.node.IsGrid());
            let params = LayoutAlgorithmParams::new(item.node.clone(), &geometry, &child_space);
            let child = GridLayoutAlgorithm::new(&params);
            let child_resolver = child.BuildGridLineResolver(
                &SubgriddedAreaInParent(&data),
                Some(resolver),
                inherit,
            );
            BuildGridSizingSubtree(
                &child,
                &child_resolver,
                tree,
                None,
                &data,
                Some(resolver),
                SizingConstraint::kLayout,
                invalidate,
                false,
                false,
            );
        }
        item.ResetPlacementIndices();
    }
    AppendSubgriddedItems(node, unsafe { &mut *items });
    if columns {
        algorithm.BuildSizingCollection(
            kForColumns,
            resolver,
            unsafe { &mut *items },
            unsafe { &mut *layout },
            constraint,
            intrinsic,
            &mut virtual_items,
        );
    }
    if rows {
        algorithm.BuildSizingCollection(
            kForRows,
            resolver,
            unsafe { &mut *items },
            unsafe { &mut *layout },
            constraint,
            intrinsic,
            &mut virtual_items,
        );
    }
    tree.SetSizingNodeData(node, items, layout, virtual_items);
}

// cpp: layoutng_grid/grid_sizing_tree_builder_impl.h:283-299
pub fn BuildGridSizingTree<A: GridSizingAlgorithm>(
    algorithm: &A,
    resolver: &GridLineResolver,
    oof: Option<&mut HeapVector<Member<LayoutBox>>>,
    constraint: SizingConstraint,
    intrinsic: bool,
) -> GridSizingTree {
    debug_assert!(algorithm
        .GetConstraintSpace()
        .GetGridLayoutSubtree()
        .is_null());
    let mut tree = GridSizingTree::default();
    BuildGridSizingSubtree(
        algorithm,
        resolver,
        &mut tree,
        oof,
        &kNoSubgriddedItemData,
        None,
        constraint,
        false,
        false,
        intrinsic,
    );
    tree
}
// cpp: layoutng_grid/grid_sizing_tree_builder_impl.h:302-318
pub fn BuildGridSizingTreeIgnoringChildren<A: GridSizingAlgorithm>(
    algorithm: &A,
    resolver: &GridLineResolver,
    constraint: SizingConstraint,
    intrinsic: bool,
) -> GridSizingTree {
    debug_assert!(algorithm
        .GetConstraintSpace()
        .GetGridLayoutSubtree()
        .is_null());
    let mut tree = GridSizingTree::default();
    BuildGridSizingSubtree(
        algorithm,
        resolver,
        &mut tree,
        None,
        &kNoSubgriddedItemData,
        None,
        constraint,
        false,
        true,
        intrinsic,
    );
    tree
}
// cpp: layoutng_grid/grid_sizing_tree_builder_impl.h:322-359
pub fn ForEachSubgrid<A: GridSizingAlgorithm>(
    subtree: &GridSizingSubtree,
    algorithm: &A,
    callback: &mut dyn FnMut(&GridLayoutAlgorithm, &GridSizingSubtree, &SubgriddedItemData),
    compute_minmax: bool,
) {
    let mut child = subtree.FirstChild();
    if !child.IsPresent() {
        return;
    }
    let layout = subtree.LayoutData();
    let items = unsafe { &*subtree.GetGridItems() };
    let mut it = items.begin_const();
    let end = items.end_const();
    while it.NotEqual(&end) {
        let item = unsafe { &*it.Get() };
        it.Advance();
        if !item.IsSubgrid() {
            continue;
        }
        let data = SubgriddedItemData::new(
            item,
            layout,
            algorithm.GetConstraintSpace().GetWritingMode(),
        );
        let space = algorithm.CreateSubgridConstraintSpace(&data);
        let none = NoGridSizingSubtree();
        let geometry = CalculateInitialFragmentGeometryForSubgrid(
            item,
            &space,
            if compute_minmax { &child } else { &none },
        );
        let params = LayoutAlgorithmParams::new(item.node.clone(), &geometry, &space);
        let sub_algorithm = GridLayoutAlgorithm::new(&params);
        debug_assert!(child.IsPresent());
        callback(&sub_algorithm, &child, &data);
        child = child.NextSibling();
    }
}
// cpp: layoutng_grid/grid_sizing_tree_builder_impl.h:364-380
pub fn InitializeTrackSizesForEachSubgrid<A: GridSizingAlgorithm>(
    subtree: &GridSizingSubtree,
    algorithm: &A,
    direction: Option<GridTrackSizingDirection>,
) {
    ForEachSubgrid(
        subtree,
        algorithm,
        &mut |algo, child, data| {
            algo.InitializeTrackSizes(
                child,
                data,
                data.RelativeDirectionFilterInSubgrid(direction),
            );
        },
        false,
    );
}
// cpp: layoutng_grid/grid_sizing_tree_builder_impl.h:385-402
pub fn CompleteTrackSizingAlgorithmForEachSubgrid<A: GridSizingAlgorithm>(
    subtree: &GridSizingSubtree,
    algorithm: &A,
    direction: GridTrackSizingDirection,
    constraint: SizingConstraint,
    additional: *mut bool,
) {
    ForEachSubgrid(
        subtree,
        algorithm,
        &mut |algo, child, data| {
            algo.CompleteTrackSizingAlgorithm(
                child,
                data,
                data.RelativeDirectionInSubgrid(direction),
                constraint,
                additional,
            );
        },
        true,
    );
}
// cpp: layoutng_grid/grid_sizing_tree_builder_impl.h:407-425
pub fn ComputeBaselineAlignmentForEachSubgrid<A: GridSizingAlgorithm>(
    subtree: &GridSizingSubtree,
    algorithm: &A,
    tree: *const GridLayoutTree,
    direction: Option<GridTrackSizingDirection>,
    constraint: SizingConstraint,
    after: bool,
) {
    ForEachSubgrid(
        subtree,
        algorithm,
        &mut |algo, child, data| {
            algo.ComputeBaselineAlignment(
                tree,
                child,
                data,
                data.RelativeDirectionFilterInSubgrid(direction),
                constraint,
                after,
            );
        },
        true,
    );
}
