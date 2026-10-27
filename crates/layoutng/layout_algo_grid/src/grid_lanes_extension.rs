use crate::{
    grid_line_resolver::GridLineResolver,
    grid_sizing_tree::{GridSizingTree, SubgriddedItemData},
};
use layoutng_assembly::internal::{block_node::BlockNode, constraint_space::ConstraintSpace};
use layoutng_geometry::geometry::fragment_geometry::FragmentGeometry;
use layoutng_style::style::{computed_style::ComputedStyle, grid_area::GridArea};

// cpp: layoutng_grid/grid_lanes_extension.h:19-29
pub struct GridLanesSizingSubtreeRequest<'a> {
    pub node: &'a BlockNode,
    pub fragment_geometry: &'a FragmentGeometry,
    pub space: &'a ConstraintSpace,
    pub subgrid_area: &'a GridArea,
    pub subgridded_item: &'a SubgriddedItemData,
    pub parent_line_resolver: &'a GridLineResolver,
    pub can_inherit_line_names_from_parent: bool,
    pub sizing_tree: &'a mut GridSizingTree,
    pub must_invalidate_placement_cache: bool,
}
// cpp: layoutng_grid/grid_lanes_extension.h:31-38
pub struct GridLanesSubgriddedItemsRequest<'a> {
    pub node: &'a BlockNode,
    pub root_grid_style: &'a ComputedStyle,
    pub subgrid_style: &'a ComputedStyle,
    pub consider_columns: bool,
    pub must_invalidate_placement_cache: *mut bool,
    pub parent_is_auto_placed: bool,
}
