use crate::{
    grid_data::GridPlacementData, grid_layout_algorithm::GridLayoutAlgorithm,
    grid_line_resolver::GridLineResolver, grid_placement::GridPlacement,
    grid_sizing_tree::GridSizingSubtree, layout_grid::LayoutGrid,
};
use foundation::{HeapVector, LayoutUnit, MakeGarbageCollected, Member, Traceable, Visitor};
use layoutng_assembly::internal::{
    block_node::BlockNode,
    constraint_space::ConstraintSpace,
    grid_item::{GridItemData, GridItems},
    grid_layout_data::GridLayoutData,
    layout_algorithm::{
        LayoutAlgorithmFromBlockNode, LayoutAlgorithmInputNode, LayoutAlgorithmParams,
    },
    layout_box::LayoutBox,
    layout_input_node::LayoutInputNode,
    length_utils::CalculateInitialFragmentGeometry,
    min_max_sizes::{MinMaxSizes, MinMaxSizesResult},
};
use layoutng_style::style::{
    computed_style::ComputedStyle,
    computed_style_initial_values::ComputedStyleInitialValues,
    grid_area::GridSpan,
    grid_enums::GridTrackSizingDirection::{self, *},
};
use std::ops::{Deref, DerefMut};

// cpp: layoutng_grid/grid_node.h:21-80
#[repr(C)]
#[derive(Clone)]
pub struct GridNode {
    pub base: BlockNode,
}
impl Deref for GridNode {
    type Target = BlockNode;
    fn deref(&self) -> &BlockNode {
        &self.base
    }
}
impl DerefMut for GridNode {
    fn deref_mut(&mut self) -> &mut BlockNode {
        &mut self.base
    }
}
impl Traceable for GridNode {
    fn Trace(&self, v: &mut Visitor<'_>) {
        self.base.Trace(v);
    }
}
// cpp: layoutng_grid/grid_node.h:82-86
impl foundation::DowncastFrom<LayoutInputNode> for GridNode {
    fn AllowFrom(node: &LayoutInputNode) -> bool {
        node.IsGrid()
    }
}
impl LayoutAlgorithmInputNode for GridNode {
    fn Style(&self) -> &ComputedStyle {
        self.base.Style()
    }
}
impl LayoutAlgorithmFromBlockNode for GridNode {
    fn FromBlockNode(node: BlockNode) -> Self {
        Self::new(node.GetLayoutBox())
    }
    fn ToBlockNode(&self) -> BlockNode {
        self.base.clone()
    }
}
impl GridNode {
    // cpp: layoutng_grid/grid_node.cc:14-17
    pub fn new(box_: *mut LayoutBox) -> Self {
        debug_assert!(!box_.is_null());
        debug_assert!(unsafe { &*box_ }.IsLayoutGrid());
        Self {
            base: BlockNode::new(box_),
        }
    }
    fn LayoutGrid(&self) -> *mut LayoutGrid {
        self.GetLayoutBox().cast()
    }
    // cpp: layoutng_grid/grid_node.cc:19-21
    pub fn HasCachedPlacementData(&self) -> bool {
        unsafe { &*self.LayoutGrid() }.HasCachedPlacementData()
    }
    // cpp: layoutng_grid/grid_node.cc:23-25
    pub fn CachedPlacementData(&self) -> &GridPlacementData {
        unsafe { &*self.LayoutGrid() }.CachedPlacementData()
    }
    // cpp: layoutng_grid/grid_node.cc:27-29
    pub fn CachedLineResolver(&self) -> &GridLineResolver {
        &self.CachedPlacementData().line_resolver
    }
    // cpp: layoutng_grid/grid_node.cc:31-33
    pub fn InvalidateSubgridMinMaxSizesCache(&self) {
        unsafe { &mut *self.GetLayoutBox() }.SetSubgridMinMaxSizesCacheDirty(true);
    }
    // cpp: layoutng_grid/grid_node.cc:35-39
    pub fn ShouldInvalidateSubgridMinMaxSizesCacheFor(&self, layout: &GridLayoutData) -> bool {
        unsafe { &*self.LayoutGrid() }.ShouldInvalidateSubgridMinMaxSizesCacheFor(layout)
    }
    // cpp: layoutng_grid/grid_node.cc:42-55
    pub fn ConstructGridItems(
        &self,
        resolver: &GridLineResolver,
        invalidate: &mut bool,
        parent_auto: bool,
        oof: Option<&mut HeapVector<Member<LayoutBox>>>,
        nested: Option<&mut bool>,
    ) -> *mut GridItems {
        self.ConstructGridItemsWithStyles(
            resolver,
            self.Style(),
            self.Style(),
            resolver.HasStandaloneAxis(kForColumns),
            resolver.HasStandaloneAxis(kForRows),
            invalidate,
            parent_auto,
            oof,
            nested,
        )
    }
    // cpp: layoutng_grid/grid_node.cc:57-166
    pub fn ConstructGridItemsWithStyles(
        &self,
        resolver: &GridLineResolver,
        root: &ComputedStyle,
        parent: &ComputedStyle,
        consider_columns: bool,
        consider_rows: bool,
        invalidate: &mut bool,
        parent_auto: bool,
        mut oof: Option<&mut HeapVector<Member<LayoutBox>>>,
        mut nested: Option<&mut bool>,
    ) -> *mut GridItems {
        if let Some(flag) = nested.as_deref_mut() {
            *flag = false;
        }
        let items_ptr = MakeGarbageCollected(GridItems::default());
        let items = unsafe { &mut *items_ptr };
        let grid = self.LayoutGrid();
        let mut cached: *const GridPlacementData = std::ptr::null();
        if unsafe { &*grid }.HasCachedPlacementData() {
            cached = unsafe { &*grid }.CachedPlacementData();
            items.ReserveInitialCapacity(unsafe { &*cached }.grid_item_positions.len() as u32);
            if *invalidate || resolver != &unsafe { &*cached }.line_resolver {
                cached = std::ptr::null();
            }
        }
        *invalidate |= cached.is_null();
        let mut sort = false;
        let initial_order = ComputedStyleInitialValues::InitialOrder();
        let mut child = self.FirstChild();
        while child.is_non_null() {
            if child.IsOutOfFlowPositioned() {
                if let Some(v) = oof.as_deref_mut() {
                    v.push(Member::from_ptr(child.GetLayoutBox()));
                }
                child = child.NextSibling();
                continue;
            }
            debug_assert!(child.IsBlock());
            let item = MakeGarbageCollected(GridItemData::new(
                BlockNode::new(child.GetLayoutBox()),
                parent,
                root,
                consider_columns,
                consider_rows,
            ));
            if parent_auto {
                unsafe { &mut *item }.is_auto_placed = true;
            }
            sort |= child.Style().Order() != initial_order;
            if let Some(flag) = nested.as_deref_mut() {
                *flag |= unsafe { &*item }.IsSubgrid();
            }
            items.AppendItem(item);
            child = child.NextSibling();
        }
        if sort {
            items.SortByOrderProperty();
        }
        #[cfg(debug_assertions)]
        if !cached.is_null() {
            let actual =
                GridPlacement::new(self.Style(), resolver).RunAutoPlacementAlgorithm(items);
            debug_assert!(unsafe { &*cached } == &actual);
        }
        if cached.is_null() {
            let data = GridPlacement::new(self.Style(), resolver).RunAutoPlacementAlgorithm(items);
            unsafe { &mut *grid }.SetCachedPlacementData(data);
            cached = unsafe { &*grid }.CachedPlacementData();
        }
        let positions = &unsafe { &*cached }.grid_item_positions;
        let mut it = items.begin();
        let end = items.end();
        let mut index = 0;
        while it.NotEqual(&end) {
            unsafe { &mut *it.Get() }.resolved_position = positions[index].clone();
            index += 1;
            it.Advance();
        }
        items_ptr
    }
    // cpp: layoutng_grid/grid_node.cc:168-200
    pub fn AdjustSubgriddedItemSpan(&self, subgrid: &GridItemData, item: &mut GridItemData) {
        if subgrid.is_auto_placed {
            item.is_auto_placed = true;
        }
        TranslateSpan(subgrid, &mut item.resolved_position.columns, kForColumns);
        TranslateSpan(subgrid, &mut item.resolved_position.rows, kForRows);
    }
    // cpp: layoutng_grid/grid_node.cc:202-206
    pub fn ComputeSetIndicesForSubgrid(
        &self,
        item: &mut GridItemData,
        layout: &mut GridLayoutData,
    ) {
        item.ComputeSetIndices(unsafe { &*layout.Columns() });
        item.ComputeSetIndices(unsafe { &*layout.Rows() });
    }
    // cpp: layoutng_grid/grid_node.cc:208-228
    pub fn ComputeSubgridMinMaxSizes(
        &self,
        subtree: &GridSizingSubtree,
        space: &ConstraintSpace,
    ) -> MinMaxSizesResult {
        debug_assert!(subtree.HasValidRootFor(self));
        let layout = unsafe { &*subtree.LayoutData() };
        debug_assert!(layout.IsSubgridWithStandaloneAxis(kForColumns));
        let grid = unsafe { &mut *self.LayoutGrid() };
        if !grid.HasCachedSubgridMinMaxSizes() {
            let geometry = CalculateInitialFragmentGeometry(space, self, std::ptr::null(), true);
            let params = LayoutAlgorithmParams::new(self.base.clone(), &geometry, space);
            let sizes = GridLayoutAlgorithm::new(&params).ComputeSubgridMinMaxSizes(subtree);
            grid.SetSubgridMinMaxSizesCache(sizes, layout);
        }
        MinMaxSizesResult::new(*grid.CachedSubgridMinMaxSizes(), false)
    }
    // cpp: layoutng_grid/grid_node.cc:230-255
    pub fn ComputeSubgridIntrinsicBlockSize(
        &self,
        subtree: &GridSizingSubtree,
        space: &ConstraintSpace,
    ) -> LayoutUnit {
        debug_assert!(subtree.HasValidRootFor(self));
        let layout = unsafe { &*subtree.LayoutData() };
        debug_assert!(layout.IsSubgridWithStandaloneAxis(kForRows));
        let grid = unsafe { &mut *self.LayoutGrid() };
        if !grid.HasCachedSubgridMinMaxSizes() {
            let geometry = CalculateInitialFragmentGeometry(space, self, std::ptr::null(), true);
            let params = LayoutAlgorithmParams::new(self.base.clone(), &geometry, space);
            let size = GridLayoutAlgorithm::new(&params).ComputeSubgridIntrinsicBlockSize(subtree);
            grid.SetSubgridMinMaxSizesCache(
                MinMaxSizes {
                    min_size: size,
                    max_size: size,
                },
                layout,
            );
        }
        grid.CachedSubgridMinMaxSizes().max_size
    }
}
// cpp: layoutng_grid/grid_node.cc:193-211
fn TranslateSpan(subgrid: &GridItemData, span: &mut GridSpan, direction: GridTrackSizingDirection) {
    if subgrid.MustConsiderGridItemsForSizing(direction) {
        if subgrid.IsOppositeDirectionInRootGrid(direction) {
            let count = subgrid.SpanSize(direction);
            debug_assert!(span.EndLine() <= count);
            *span = GridSpan::TranslatedDefiniteGridSpan(
                count - span.EndLine(),
                count - span.StartLine(),
            );
        }
        span.Translate(subgrid.StartLine(direction));
    }
}

// Adapter for the original LayoutAlgorithm<GridNode, BoxFragmentBuilder, BlockBreakToken> specialization.
impl
    layoutng_assembly::internal::layout_algorithm::LayoutAlgorithmBuilder<
        GridNode,
        layoutng_assembly::block_break_token::BlockBreakToken,
    > for layoutng_assembly::box_fragment_builder::BoxFragmentBuilder
{
    fn New(
        node: GridNode,
        style: *const ComputedStyle,
        space: &ConstraintSpace,
        direction: foundation::WritingDirectionMode,
        break_token: *const layoutng_assembly::block_break_token::BlockBreakToken,
    ) -> Self {
        layoutng_assembly::box_fragment_builder::BoxFragmentBuilder::new(
            node.base.base,
            style,
            space,
            direction,
            break_token,
        )
    }
    fn GetGapGeometry(&self) -> *const layoutng_assembly::internal::gap::gap_geometry::GapGeometry {
        layoutng_assembly::box_fragment_builder::BoxFragmentBuilder::GetGapGeometry(self)
    }
    fn GetConstraintSpace(&self) -> &ConstraintSpace {
        layoutng_assembly::fragment_builder::FragmentBuilder::GetConstraintSpace(
            std::ops::Deref::deref(self),
        )
    }
    fn BfcLineOffset(&self) -> LayoutUnit {
        layoutng_assembly::fragment_builder::FragmentBuilder::BfcLineOffset(std::ops::Deref::deref(
            self,
        ))
    }
    fn BfcBlockOffset(&self) -> Option<LayoutUnit> {
        *layoutng_assembly::fragment_builder::FragmentBuilder::BfcBlockOffset(
            std::ops::Deref::deref(self),
        )
    }
    fn PreviousBreakToken(&self) -> *const layoutng_assembly::block_break_token::BlockBreakToken {
        layoutng_assembly::box_fragment_builder::BoxFragmentBuilder::PreviousBreakToken(self)
    }
    fn Borders(&self) -> &layoutng_geometry::geometry::box_strut::BoxStrut {
        layoutng_assembly::box_fragment_builder::BoxFragmentBuilder::Borders(self)
    }
    fn Scrollbar(&self) -> &layoutng_geometry::geometry::box_strut::BoxStrut {
        layoutng_assembly::box_fragment_builder::BoxFragmentBuilder::Scrollbar(self)
    }
    fn Padding(&self) -> &layoutng_geometry::geometry::box_strut::BoxStrut {
        layoutng_assembly::box_fragment_builder::BoxFragmentBuilder::Padding(self)
    }
    fn BorderPadding(&self) -> &layoutng_geometry::geometry::box_strut::BoxStrut {
        layoutng_assembly::box_fragment_builder::BoxFragmentBuilder::BorderPadding(self)
    }
    fn BorderScrollbarPadding(&self) -> &layoutng_geometry::geometry::box_strut::BoxStrut {
        layoutng_assembly::box_fragment_builder::BoxFragmentBuilder::BorderScrollbarPadding(self)
    }
    fn OriginalBorderScrollbarPaddingBlockStart(&self) -> LayoutUnit {
        layoutng_assembly::box_fragment_builder::BoxFragmentBuilder::OriginalBorderScrollbarPaddingBlockStart(self)
    }
    fn ChildAvailableSize(&self) -> &layoutng_geometry::geometry::logical_size::LogicalSize {
        layoutng_assembly::box_fragment_builder::BoxFragmentBuilder::ChildAvailableSize(self)
    }
    fn GetExclusionSpace(
        &mut self,
    ) -> &mut layoutng_assembly::internal::exclusions::exclusion_space::ExclusionSpace {
        layoutng_assembly::fragment_builder::FragmentBuilder::GetExclusionSpace(
            std::ops::DerefMut::deref_mut(self),
        )
    }
}
