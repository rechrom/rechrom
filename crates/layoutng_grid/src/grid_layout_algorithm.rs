use crate::{
    grid_baseline_accumulator::GridBaselineAccumulator,
    grid_break_token_data::{GridBreakTokenData, GridItemPlacementData},
    grid_layout_utils::*,
    grid_line_resolver::GridLineResolver,
    grid_node::GridNode,
    grid_oof_placement::ComputeGridOutOfFlowItemContainingRect,
    grid_sizing_tree::{
        kNoSubgriddedItemData, GridSizingSubtree, GridSizingTree, SubgriddedItemData,
    },
    grid_sizing_tree_builder_impl::*,
    grid_track_sizing_algorithm::{
        GridItemContributionType, GridTrackSizingAlgorithm, SizingConstraint,
    },
};
use foundation::{
    kIndefiniteSize, DynamicTo, EBoxDecorationBreak, EBreakBetween, EReadingFlow, HeapVector,
    IsHorizontalWritingMode, IsParallelWritingMode, LayoutUnit, MakeGarbageCollected, Member, To,
};
use layoutng_assembly::{
    block_break_token::BlockBreakToken,
    box_fragment_builder::BoxFragmentBuilder,
    internal::{
        algorithm_entry::NativeAlgorithm,
        block_node::BlockNode,
        break_appeal::BreakAppeal,
        constraint_space::{ConstraintSpace, LayoutResultCacheSlot},
        constraint_space_builder::ConstraintSpaceBuilder,
        early_break::*,
        fragmentation_utils::*,
        gap::{
            gap_geometry::{ContainerType, GapGeometry},
            gap_utils::{GapSegmentStateAggregator, GapSegmentStateAggregatorOps},
        },
        grid_item::{GridItemData, GridItemIndices, GridItems},
        grid_lanes_item_group::VirtualItems,
        grid_layout_data::{GridLayoutData, GridLayoutSubtree, GridLayoutTree},
        grid_track_collection::{
            GridLayoutTrackCollection, GridRangeBuilder, GridSizingTrackCollection,
        },
        layout_algorithm::{
            LayoutAlgorithm, LayoutAlgorithmBuilder, LayoutAlgorithmParams, RelayoutAlgorithm,
            RelayoutType,
        },
        layout_box::LayoutBox,
        layout_input_node::MinMaxSizesFloatInput,
        layout_node_metadata::Node,
        length_utils::*,
        min_max_sizes::{MinMaxSizes, MinMaxSizesResult},
        relative_utils::*,
        scroll_layout_scope::DelayScrollOffsetClampScope,
    },
    layout_result::{EStatus, LayoutResult},
    logical_box_fragment::LogicalBoxFragment,
    physical_box_fragment::PhysicalBoxFragment,
};
use layoutng_geometry::geometry::{
    box_strut::BoxStrut,
    logical_offset::LogicalOffset,
    logical_rect::LogicalRect,
    logical_size::{kIndefiniteLogicalSize, LogicalSize},
    static_position::{LogicalAlignmentDirection, LogicalStaticPosition},
};
use layoutng_style::style::{
    computed_style::ComputedStyle,
    computed_style_initial_values::ComputedStyleInitialValues,
    grid_area::{GridArea, GridSpan},
    grid_enums::GridTrackSizingDirection::{self, *},
};
use std::cell::Cell;
use std::ops::{Deref, DerefMut};
use GridItemContributionType::*;

// cpp: layoutng_grid/grid_layout_algorithm.h:23-252
#[repr(C)]
pub struct GridLayoutAlgorithm {
    base: LayoutAlgorithm<GridNode, BoxFragmentBuilder, BlockBreakToken>,
    grid_available_size_: LogicalSize,
    grid_min_available_size_: LogicalSize,
    grid_max_available_size_: LogicalSize,
    contain_intrinsic_block_size_: Option<LayoutUnit>,
}
impl Deref for GridLayoutAlgorithm {
    type Target = LayoutAlgorithm<GridNode, BoxFragmentBuilder, BlockBreakToken>;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl DerefMut for GridLayoutAlgorithm {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
// cpp: layoutng_grid/grid_layout_algorithm.h:142-146
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BaselineCollectionPhase {
    kBaselinesForTrackSizing,
    kFinalBaselines,
    kBaselinesForStandaloneAxes,
}
impl NativeAlgorithm for GridLayoutAlgorithm {
    fn new(p: &LayoutAlgorithmParams) -> Self {
        Self::new(p)
    }
    fn layout(&mut self) -> *const LayoutResult {
        self.Layout()
    }
    fn compute_min_max_sizes(&mut self, input: &MinMaxSizesFloatInput) -> MinMaxSizesResult {
        self.ComputeMinMaxSizes(input)
    }
}
impl RelayoutAlgorithm<GridNode> for GridLayoutAlgorithm {
    fn base(&self) -> &LayoutAlgorithm<GridNode, BoxFragmentBuilder, BlockBreakToken> {
        &self.base
    }
    fn base_mut(&mut self) -> &mut LayoutAlgorithm<GridNode, BoxFragmentBuilder, BlockBreakToken> {
        &mut self.base
    }
    unsafe fn from_base<'a>(
        base: &'a LayoutAlgorithm<GridNode, BoxFragmentBuilder, BlockBreakToken>,
    ) -> &'a Self {
        unsafe { &*(base as *const _ as *const Self) }
    }
}
impl GridSizingAlgorithm for GridLayoutAlgorithm {
    fn Node(&self) -> &GridNode {
        self.base.Node()
    }
    fn GetConstraintSpace(&self) -> &ConstraintSpace {
        self.base.GetConstraintSpace()
    }
    fn BorderScrollbarPadding(&self) -> &BoxStrut {
        self.base.BorderScrollbarPadding()
    }
    fn GetGridAvailableSize(&self) -> LogicalSize {
        self.grid_available_size_
    }
    fn BuildSizingCollection(
        &self,
        d: GridTrackSizingDirection,
        r: &GridLineResolver,
        i: &mut GridItems,
        l: &mut GridLayoutData,
        c: SizingConstraint,
        n: bool,
        v: &mut *mut VirtualItems,
    ) {
        GridLayoutAlgorithm::BuildSizingCollection(self, d, r, i, l, c, n, v);
    }
    fn CreateSubgridConstraintSpace(&self, item: &SubgriddedItemData) -> ConstraintSpace {
        self.CreateConstraintSpaceForLayout(
            item,
            std::ptr::null(),
            None,
            LayoutUnit::default(),
            false,
            None,
            None,
        )
    }
}
impl GridLayoutAlgorithm {
    // cpp: layoutng_grid/grid_layout_algorithm.cc:25-58
    pub fn new(params: &LayoutAlgorithmParams) -> Self {
        debug_assert!(params.space.IsNewFormattingContext());
        let base = LayoutAlgorithm::from_params(params);
        let available = *base.ChildAvailableSize();
        let mut this = Self {
            base,
            grid_available_size_: available,
            grid_min_available_size_: available,
            grid_max_available_size_: available,
            contain_intrinsic_block_size_: None,
        };
        ComputeAvailableSizes(
            this.base.BorderScrollbarPadding(),
            this.base.Node(),
            this.base.GetConstraintSpace(),
            &this.base.container_builder_,
            &mut this.grid_available_size_,
            &mut this.grid_min_available_size_,
            &mut this.grid_max_available_size_,
        );
        if this.grid_available_size_.block_size == kIndefiniteSize
            && this.Node().ShouldApplyBlockSizeContainment()
        {
            let intrinsic = this.ComputeIntrinsicBlockSizeIgnoringChildren();
            this.contain_intrinsic_block_size_ = Some(intrinsic);
            let size = ComputeBlockSizeForFragment(
                this.GetConstraintSpace(),
                this.Node(),
                this.BorderPadding(),
                intrinsic,
                this.container_builder_.InlineSize(),
                kIndefiniteSize,
            );
            let size = (size - this.BorderScrollbarPadding().BlockSum()).ClampNegativeToZero();
            this.grid_available_size_.block_size = size;
            this.grid_min_available_size_.block_size = size;
            this.grid_max_available_size_.block_size = size;
        }
        this
    }
    pub fn Node(&self) -> &GridNode {
        self.base.Node()
    }
    pub fn GetConstraintSpace(&self) -> &ConstraintSpace {
        self.base.GetConstraintSpace()
    }
    pub fn BorderScrollbarPadding(&self) -> &BoxStrut {
        self.base.BorderScrollbarPadding()
    }
    // cpp: layoutng_grid/grid_layout_algorithm.h:90-90
    pub fn GetGridAvailableSize(&self) -> LogicalSize {
        self.grid_available_size_
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:60-67
    pub fn Layout(&mut self) -> *const LayoutResult {
        let result = self.LayoutInternal();
        if unsafe { &*result }.Status() == EStatus::kDisableFragmentation {
            debug_assert!(self.GetConstraintSpace().HasBlockFragmentation());
            return self.base.RelayoutWithoutFragmentation::<Self>();
        }
        result
    }

    // cpp: layoutng_grid/grid_layout_algorithm.cc:259-332
    pub fn ComputeMinMaxSizes(&mut self, _input: &MinMaxSizesFloatInput) -> MinMaxSizesResult {
        let override_size = self.Node().OverrideIntrinsicContentInlineSize();
        if override_size != kIndefiniteSize {
            return self.FixedMinMaxSizes(override_size);
        }
        let subtree = self.GetConstraintSpace().GetGridLayoutSubtree();
        if !subtree.is_null() {
            return self.FixedMinMaxSizes(
                unsafe { &*(*(*subtree).LayoutData()).Columns() }.CalculateSetSpanSize(),
            );
        }
        let resolver = self.BuildGridLineResolver(&GridArea::default(), None, true);
        let mut tree = if self.Node().ShouldApplyInlineSizeContainment() {
            BuildGridSizingTreeIgnoringChildren(self, &resolver, SizingConstraint::kLayout, false)
        } else {
            BuildGridSizingTree(self, &resolver, None, SizingConstraint::kLayout, false)
        };
        let mut depends = false;
        let max =
            self.ComputeTotalColumnSize(&mut tree, SizingConstraint::kMaxContent, &mut depends);
        let min =
            self.ComputeTotalColumnSize(&mut tree, SizingConstraint::kMinContent, &mut depends);
        let mut sizes = MinMaxSizes {
            min_size: LayoutUnit::default(),
            max_size: max,
        };
        sizes.EncompassValue(min);
        sizes += self.BorderScrollbarPadding().InlineSum();
        MinMaxSizesResult::new(sizes, depends)
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:264-267
    fn FixedMinMaxSizes(&self, mut size: LayoutUnit) -> MinMaxSizesResult {
        size += self.BorderScrollbarPadding().InlineSum();
        MinMaxSizesResult::new(
            MinMaxSizes {
                min_size: size,
                max_size: size,
            },
            false,
        )
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:286-312
    fn ComputeTotalColumnSize(
        &self,
        tree: &mut GridSizingTree,
        constraint: SizingConstraint,
        depends: &mut bool,
    ) -> LayoutUnit {
        self.InitializeTrackSizesTree(tree, None);
        let mut additional = false;
        self.CompleteTrackSizingAlgorithmTree(kForColumns, constraint, tree, &mut additional);
        let column_subtree =
            tree.HasSubgridWithIndefiniteStandaloneAxis() && tree.HasBlockSizeDependentGridItem();
        if additional || tree.HasBlockSizeDependentGridItem() {
            *depends = true;
            self.CompleteTrackSizingAlgorithmTree(kForRows, constraint, tree, &mut additional);
            if additional || column_subtree {
                self.InitializeTrackSizesTree(tree, Some(kForColumns));
                self.CompleteTrackSizingAlgorithmTree(
                    kForColumns,
                    constraint,
                    tree,
                    std::ptr::null_mut(),
                );
            }
        }
        unsafe { &*(*tree.LayoutData(0)).Columns() }.CalculateSetSpanSize()
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:334-342
    pub fn ComputeSubgridMinMaxSizes(&self, subtree: &GridSizingSubtree) -> MinMaxSizes {
        debug_assert!(subtree.HasValidRootFor(self.Node()));
        MinMaxSizes {
            min_size: self.ComputeSubgridIntrinsicSize(
                subtree,
                kForColumns,
                SizingConstraint::kMinContent,
            ),
            max_size: self.ComputeSubgridIntrinsicSize(
                subtree,
                kForColumns,
                SizingConstraint::kMaxContent,
            ),
        }
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:344-350
    pub fn ComputeSubgridIntrinsicBlockSize(&self, subtree: &GridSizingSubtree) -> LayoutUnit {
        debug_assert!(subtree.HasValidRootFor(self.Node()));
        self.ComputeSubgridIntrinsicSize(subtree, kForRows, SizingConstraint::kMaxContent)
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:352-519
    fn ComputeGridGeometry(
        &mut self,
        items: &mut *mut GridItems,
        intrinsic: &mut LayoutUnit,
        oof: &mut HeapVector<Member<LayoutBox>>,
    ) -> *const GridLayoutSubtree {
        debug_assert!(items.is_null());
        debug_assert_ne!(self.grid_available_size_.inline_size, kIndefiniteSize);
        let node = self.Node().clone();
        let space = self.GetConstraintSpace() as *const ConstraintSpace;
        let bsp = *self.BorderScrollbarPadding();
        let layout_subtree = unsafe { &*space }.GetGridLayoutSubtree();
        if !layout_subtree.is_null() {
            let data = unsafe { &*(*layout_subtree).LayoutData() };
            if !node.ChildLayoutBlockedByDisplayLock() {
                let mut invalidate = false;
                *items = node.ConstructGridItems(
                    node.CachedLineResolver(),
                    &mut invalidate,
                    false,
                    Some(oof),
                    None,
                );
                debug_assert!(!invalidate);
                GridTrackSizingAlgorithm::CacheGridItemsProperties(
                    unsafe { &*data.Columns() },
                    unsafe { &mut **items },
                );
                GridTrackSizingAlgorithm::CacheGridItemsProperties(
                    unsafe { &*data.Rows() },
                    unsafe { &mut **items },
                );
            }
            *intrinsic = self.CalculateIntrinsicBlockSize(unsafe { &**items }, data);
            return layout_subtree;
        }
        let resolver = self.BuildGridLineResolver(&GridArea::default(), None, true);
        let mut tree = if node.ChildLayoutBlockedByDisplayLock() {
            BuildGridSizingTreeIgnoringChildren(self, &resolver, SizingConstraint::kLayout, false)
        } else {
            BuildGridSizingTree(self, &resolver, Some(oof), SizingConstraint::kLayout, false)
        };
        self.InitializeTrackSizesTree(&mut tree, None);
        let mut additional = false;
        self.CompleteTrackSizingAlgorithmTree(
            kForColumns,
            SizingConstraint::kLayout,
            &mut tree,
            &mut additional,
        );
        self.CompleteTrackSizingAlgorithmTree(
            kForRows,
            SizingConstraint::kLayout,
            &mut tree,
            &mut additional,
        );
        let data = tree.LayoutData(0);
        *intrinsic =
            self.CalculateIntrinsicBlockSize(unsafe { &*tree.GetGridItems(0) }, unsafe { &*data });
        let style = self.Style();
        let automatic_min = !style.AspectRatio().IsAuto()
            && !style.IsOverflowValueScrollableBlock()
            && style.LogicalMinHeight().HasAuto();
        if self.grid_available_size_.block_size == kIndefiniteSize || automatic_min {
            let size = ComputeBlockSizeForFragment(
                unsafe { &*space },
                &node,
                self.BorderPadding(),
                *intrinsic,
                self.container_builder_.InlineSize(),
                kIndefiniteSize,
            );
            debug_assert_ne!(size, kIndefiniteSize);
            let size = (size - bsp.BlockSum()).ClampNegativeToZero();
            self.grid_available_size_.block_size = size;
            self.grid_min_available_size_.block_size = size;
            self.grid_max_available_size_.block_size = size;
            let tracks = unsafe { &mut *(*data).SizingCollection(kForRows) };
            additional |= NeedsAdditionalLayoutPass(
                self.Style(),
                unsafe { &*space },
                &node,
                self.BorderPadding(),
                tracks,
                self.container_builder_.InlineSize(),
            );
            if !additional
                && *self.Style().AlignContent() != ComputedStyleInitialValues::InitialAlignContent()
            {
                let first = GridTrackSizingAlgorithm::ComputeFirstSetGeometry(
                    tracks,
                    self.Style(),
                    &self.grid_available_size_,
                    &bsp,
                );
                tracks.FinalizeSetsGeometry(first.start_offset, first.gutter_size);
            }
        }
        if additional {
            for direction in [kForColumns, kForRows] {
                self.InitializeTrackSizesTree(&mut tree, Some(direction));
                self.CompleteTrackSizingAlgorithmTree(
                    direction,
                    SizingConstraint::kLayout,
                    &mut tree,
                    std::ptr::null_mut(),
                );
            }
        } else if tree.HasSubgridWithIndefiniteStandaloneAxis()
            && tree.HasBlockSizeDependentGridItem()
        {
            self.InitializeTrackSizesTree(&mut tree, Some(kForColumns));
            self.CompleteTrackSizingAlgorithmTree(
                kForColumns,
                SizingConstraint::kLayout,
                &mut tree,
                std::ptr::null_mut(),
            );
        }
        if tree.HasDeferredSubgridBaseline() {
            self.ResolveBaselinesInStandaloneAxes(
                &GridSizingSubtree::new(&mut tree, 0),
                &mut tree,
                SizingConstraint::kLayout,
                false,
            );
        }
        self.CompleteFinalBaselineAlignment(&mut tree);
        *items = tree.GetGridItems(0);
        MakeGarbageCollected(GridLayoutSubtree::new(tree.FinalizeTree(), 0))
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:369-388
    fn CalculateIntrinsicBlockSize(&self, items: &GridItems, data: &GridLayoutData) -> LayoutUnit {
        if let Some(size) = self.contain_intrinsic_block_size_ {
            return size;
        }
        let bsp = self.BorderScrollbarPadding();
        let mut size = unsafe { &*data.Rows() }.CalculateSetSpanSize() + bsp.BlockSum();
        if items.IsEmpty() && self.Node().HasLineIfEmpty() {
            size = size.max(bsp.BlockSum() + self.Node().EmptyLineBlockSize(self.GetBreakToken()));
        }
        ClampIntrinsicBlockSize(
            self.GetConstraintSpace(),
            self.Node(),
            self.GetBreakToken(),
            bsp,
            size,
            None,
        )
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:521-543
    fn ComputeIntrinsicBlockSizeIgnoringChildren(&self) -> LayoutUnit {
        let node = self.Node();
        let override_size = node.OverrideIntrinsicContentBlockSize();
        debug_assert!(node.ShouldApplyBlockSizeContainment());
        if override_size != kIndefiniteSize {
            return self.BorderScrollbarPadding().BlockSum() + override_size;
        }
        let resolver = self.BuildGridLineResolver(&GridArea::default(), None, true);
        let mut tree =
            BuildGridSizingTreeIgnoringChildren(self, &resolver, SizingConstraint::kLayout, false);
        self.InitializeTrackSizesTree(&mut tree, Some(kForRows));
        self.CompleteTrackSizingAlgorithmTree(
            kForRows,
            SizingConstraint::kLayout,
            &mut tree,
            std::ptr::null_mut(),
        );
        unsafe { &*(*tree.LayoutData(0)).Rows() }.CalculateSetSpanSize()
            + self.BorderScrollbarPadding().BlockSum()
    }
}

// cpp: layoutng_grid/grid_layout_algorithm.cc:547-551
fn Baseline(
    item: &GridItemData,
    layout: &GridLayoutData,
    direction: GridTrackSizingDirection,
) -> LayoutUnit {
    GetTrackBaseline(item, layout, direction)
}
// cpp: layoutng_grid/grid_layout_algorithm.cc:553-565
fn ComputeBlockSizeForSubgrid(
    subtree: &GridSizingSubtree,
    item: &GridItemData,
    space: &ConstraintSpace,
) -> LayoutUnit {
    debug_assert!(subtree.IsPresent());
    debug_assert!(item.IsSubgrid());
    let node = GridNode::new(item.node.GetLayoutBox());
    ComputeBlockSizeForFragment(
        space,
        &node,
        &(ComputeBorders(space, &node) + ComputePadding(space, node.Style())),
        node.ComputeSubgridIntrinsicBlockSize(subtree, space),
        space.AvailableSize().inline_size,
        kIndefiniteSize,
    )
}

// Source local contribution lambdas share this stack context. Raw item access
// preserves their mutation contract without overlapping Rust mutable borrows.
// cpp: layoutng_grid/grid_layout_algorithm.cc:575-707
struct ContributionContext<'a> {
    algorithm: &'a GridLayoutAlgorithm,
    subtree: &'a GridSizingSubtree,
    item: *mut GridItemData,
    node: BlockNode,
    space: ConstraintSpace,
    subgridded: SubgriddedItemData,
    direction: GridTrackSizingDirection,
    constraint: SizingConstraint,
    parallel: bool,
    columns: bool,
    shim: Cell<LayoutUnit>,
}
impl ContributionContext<'_> {
    // cpp: layoutng_grid/grid_layout_algorithm.cc:603-620
    fn CalculateBaselineShim(&self, baseline: LayoutUnit) {
        let item = unsafe { &*self.item };
        let track = Baseline(item, unsafe { &*self.subtree.LayoutData() }, self.direction);
        if track == LayoutUnit::Min() {
            return;
        }
        let extra = GetExtraMarginForBaseline(
            &ComputeMarginsForDirection(
                &self.space,
                self.node.Style(),
                item.BaselineWritingDirection(self.direction),
            ),
            &self.subgridded,
            self.direction,
            self.algorithm.GetConstraintSpace().GetWritingMode(),
        );
        self.shim.set(track - baseline - extra);
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:622-629
    fn MinMaxSizesFunc(&self, kind: SizeType) -> MinMaxSizesResult {
        if unsafe { &*self.item }.IsSubgrid() {
            GridNode::new(self.node.GetLayoutBox()).ComputeSubgridMinMaxSizes(
                &self.subtree.SubgridSizingSubtree(unsafe { &*self.item }),
                &self.space,
            )
        } else {
            self.node.ComputeMinMaxSizes(
                self.node.Style().GetWritingMode(),
                kind,
                &self.space,
                MinMaxSizesFloatInput::default(),
            )
        }
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:631-659
    fn MinOrMaxContentSize(&self, min: bool) -> LayoutUnit {
        let result = ComputeMinAndMaxContentContributionForSelfWithCallback(
            &self.node,
            &self.space,
            &|kind| self.MinMaxSizesFunc(kind),
        );
        if unsafe { &*self.item }.is_parallel_with_root_grid && result.depends_on_block_constraints
        {
            unsafe { &mut *self.item }.is_sizing_dependent_on_block_size = true;
            self.subtree.SetHasBlockSizeDependentGridItem();
        }
        let content = if min {
            result.sizes.min_size
        } else {
            result.sizes.max_size
        };
        if unsafe { &*self.item }.IsBaselineAligned(self.direction) {
            self.CalculateBaselineShim(GetSynthesizedLogicalBaseline(
                unsafe { &*self.item },
                content,
                self.direction,
            ));
        }
        content + self.shim.get()
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:675-710
    fn BlockContributionSize(&self) -> LayoutUnit {
        debug_assert!(!self.parallel);
        if unsafe { &*self.item }.IsSubgrid() {
            return ComputeBlockSizeForSubgrid(
                &self.subtree.SubgridSizingSubtree(unsafe { &*self.item }),
                unsafe { &*self.item },
                &self.space,
            );
        }
        if self.columns {
            unsafe { &mut *self.item }.is_sizing_dependent_on_block_size = true;
            self.subtree.SetHasBlockSizeDependentGridItem();
        }
        let result = if self.space.AvailableSize().inline_size == kIndefiniteSize {
            let fallback = self.algorithm.CreateConstraintSpaceForMeasure(
                &self.subgridded,
                self.direction,
                Some(self.MinOrMaxContentSize(false)),
            );
            LayoutGridItemForMeasure(unsafe { &*self.item }, &fallback, self.constraint, false)
        } else {
            LayoutGridItemForMeasure(unsafe { &*self.item }, &self.space, self.constraint, false)
        };
        let item = unsafe { &*self.item };
        let fragment =
            LogicalBoxFragment::new(item.BaselineWritingDirection(self.direction), unsafe {
                &*To::<PhysicalBoxFragment>(unsafe { &*result }.GetPhysicalFragment())
            });
        if item.IsBaselineAligned(self.direction) {
            self.CalculateBaselineShim(GetLogicalBaseline(
                &fragment,
                item.parent_grid_font_baseline,
                item.IsLastBaselineSpecified(self.direction),
            ));
        }
        fragment.BlockSize() + self.shim.get()
    }
    fn ContentContribution(&self, min: bool) -> LayoutUnit {
        if self.parallel {
            self.MinOrMaxContentSize(min)
        } else {
            self.BlockContributionSize()
        }
    }
}
impl GridLayoutAlgorithm {
    // cpp: layoutng_grid/grid_layout_algorithm.cc:569-806
    fn ContributionSizeForGridItem(
        &self,
        subtree: &GridSizingSubtree,
        kind: GridItemContributionType,
        direction: GridTrackSizingDirection,
        constraint: SizingConstraint,
        item: &mut GridItemData,
    ) -> LayoutUnit {
        debug_assert!(item.IsConsideredForSizing(direction));
        let columns = direction == kForColumns;
        let parallel = columns == item.is_parallel_with_root_grid;
        let mode = self.GetConstraintSpace().GetWritingMode();
        let subgridded = if item.is_subgridded_to_parent_grid {
            subtree.LookupSubgriddedItemData(item)
        } else {
            SubgriddedItemData::new(item, subtree.LayoutData(), mode)
        };
        let space = self.CreateConstraintSpaceForMeasure(&subgridded, direction, None);
        let node = item.node.clone();
        let ctx = ContributionContext {
            algorithm: self,
            subtree,
            item,
            node,
            space,
            subgridded,
            direction,
            constraint,
            parallel,
            columns,
            shim: Cell::new(LayoutUnit::default()),
        };
        let tracks = if columns {
            subgridded.Columns(Some(mode))
        } else {
            subgridded.Rows(Some(mode))
        };
        let margins = ComputeMarginsFor(&ctx.space, ctx.node.Style(), self.GetConstraintSpace());
        let indices = subgridded.SetIndices(tracks.Direction());
        let margin = (if columns {
            margins.InlineSum()
        } else {
            margins.BlockSum()
        }) + tracks.StartExtraMargin(indices.begin)
            + tracks.EndExtraMargin(indices.end);
        let mut contribution = match kind {
            kForContentBasedMinimums | kForIntrinsicMaximums => ctx.ContentContribution(true),
            kForMaxContentMinimums | kForMaxContentMaximums => ctx.ContentContribution(false),
            kForIntrinsicMinimums => {
                let special = !unsafe { &*ctx.item }.IsSpanningAutoMinimumTrack(direction)
                    || (unsafe { &*ctx.item }.IsSpanningFlexibleTrack(direction)
                        && unsafe { &*ctx.item }.SpanSize(direction) > 1);
                let mut clamp = false;
                let mut value = CalculateIntrinsicMinimumContribution(
                    parallel,
                    special,
                    &mut || ctx.ContentContribution(true),
                    &mut || ctx.ContentContribution(false),
                    &mut || {
                        assert!(unsafe { &*ctx.item }.IsSubgrid());
                        let child = subtree.SubgridSizingSubtree(unsafe { &*ctx.item });
                        if unsafe { &*child.LayoutData() }.IsSubgridWithStandaloneAxis(kForColumns)
                        {
                            GridNode::new(ctx.node.GetLayoutBox())
                                .ComputeSubgridMinMaxSizes(&child, &ctx.space)
                        } else {
                            MinMaxSizesResult::default()
                        }
                    },
                    &ctx.space,
                    ctx.item,
                    &mut clamp,
                );
                if clamp {
                    let max = tracks.CalculateSetSpanSizeRange(indices.begin, indices.end);
                    if max != kIndefiniteSize {
                        value += margin;
                        let bp = ComputeBorders(&ctx.space, &ctx.node)
                            + ComputePadding(&ctx.space, ctx.node.Style());
                        let sum = if parallel {
                            bp.InlineSum()
                        } else {
                            bp.BlockSum()
                        };
                        value = ClampIntrinsicMinSize(value, margin + ctx.shim.get() + sum, max);
                        value -= margin;
                    }
                }
                value
            }
            kForFreeSpace => unreachable!("free space is only distributed by maximize/stretch"),
        };
        contribution += margin;
        contribution.ClampNegativeToZero()
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:808-846
    pub fn BuildSizingCollection(
        &self,
        direction: GridTrackSizingDirection,
        resolver: &GridLineResolver,
        items: &mut GridItems,
        layout: &mut GridLayoutData,
        _constraint: SizingConstraint,
        _intrinsic: bool,
        _virtual_items: &mut *mut VirtualItems,
    ) {
        let offset = if self.Node().HasCachedPlacementData() {
            self.Node().CachedPlacementData().StartOffset(direction)
        } else {
            0
        };
        let mut builder = GridRangeBuilder::new(
            self.Style(),
            direction,
            resolver.AutoRepetitions(direction),
            offset,
        );
        let mut baselines = false;
        let range = items.IncludeSubgriddedItems();
        let mut it = range.begin();
        let end = range.end();
        while it.NotEqual(&end) {
            let item = unsafe { &mut *it.Get() };
            it.Advance();
            if item.IsConsideredForSizing(direction) {
                baselines |= item.IsBaselineSpecified(direction);
            }
            if item.MustCachePlacementIndices(direction) {
                let start = item.StartLine(direction);
                let span = item.SpanSize(direction);
                let indices = item.RangeIndices(direction);
                builder.EnsureTrackCoverage(start, span, &mut indices.begin, &mut indices.end);
            }
        }
        layout.SetTrackCollection(
            MakeGarbageCollected(GridSizingTrackCollection::new(
                builder.FinalizeRanges(false),
                direction,
                false,
            ))
            .cast(),
        );
        if baselines {
            layout.CreateBaselines(direction);
        }
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:848-864
    pub fn BuildGridLineResolver(
        &self,
        area: &GridArea,
        parent: Option<&GridLineResolver>,
        inherit: bool,
    ) -> GridLineResolver {
        let columns = self.ComputeAutomaticRepetitions(&area.columns, kForColumns);
        let rows = self.ComputeAutomaticRepetitions(&area.rows, kForRows);
        if let Some(parent) = parent {
            GridLineResolver::for_subgrid(self.Style(), parent, *area, columns, rows, inherit)
        } else {
            GridLineResolver::new(self.Style(), columns, rows)
        }
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:867-905
    fn ComputeAutomaticRepetitions(
        &self,
        span: &GridSpan,
        direction: GridTrackSizingDirection,
    ) -> u32 {
        let columns = direction == kForColumns;
        let style = self.Style();
        let tracks = style.TemplateTracks(direction).GetTrackList();
        if !tracks.HasAutoRepeater() {
            return 0;
        }
        if tracks.IsSubgriddedAxis() {
            if span.IsIndefinite() {
                return 0;
            }
            return self.ComputeAutomaticRepetitionsForSubgrid(span.IntegerSpan(), direction);
        }
        let gutter = GridTrackSizingAlgorithm::CalculateGutterSize(
            style,
            &self.grid_available_size_,
            direction,
        );
        CalculateAutomaticRepetitions(
            tracks,
            gutter,
            if columns {
                self.grid_available_size_.inline_size
            } else {
                self.grid_available_size_.block_size
            },
            if columns {
                self.grid_min_available_size_.inline_size
            } else {
                self.grid_min_available_size_.block_size
            },
            if columns {
                self.grid_max_available_size_.inline_size
            } else {
                self.grid_max_available_size_.block_size
            },
            None,
        )
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:907-941
    fn ComputeAutomaticRepetitionsForSubgrid(
        &self,
        span: u32,
        direction: GridTrackSizingDirection,
    ) -> u32 {
        let tracks = self.Style().TemplateTracks(direction).GetTrackList();
        debug_assert!(tracks.HasAutoRepeater());
        let non_auto = tracks.NonAutoRepeatLineCount();
        if non_auto > span {
            return 0;
        }
        let repeat = tracks.AutoRepeatTrackCount();
        if repeat > span {
            return 0;
        }
        debug_assert!(repeat > 0);
        (span - non_auto + 1) / repeat
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:943-1037
    fn ComputeGridItemBaselines(
        &self,
        tree: *const GridLayoutTree,
        subtree: &GridSizingSubtree,
        direction: GridTrackSizingDirection,
        constraint: SizingConstraint,
        phase: BaselineCollectionPhase,
        after: bool,
    ) {
        let layout = subtree.LayoutData();
        if !unsafe { &*layout }.HasBaselines(direction) {
            return;
        }
        let tracks = subtree.SizingCollection(direction);
        let mode = self.GetConstraintSpace().GetWritingMode();
        if phase != BaselineCollectionPhase::kBaselinesForStandaloneAxes {
            unsafe { &mut *layout }
                .ResetBaselines(direction, unsafe { &*tracks }.base.GetSetCount());
        }
        let range = unsafe { &mut *subtree.GetGridItems() }.IncludeSubgriddedItems();
        let mut it = range.begin();
        let end = range.end();
        while it.NotEqual(&end) {
            let item = unsafe { &mut *it.Get() };
            it.Advance();
            if !item.IsBaselineSpecified(direction) || !item.IsConsideredForSizing(direction) {
                continue;
            }
            if phase == BaselineCollectionPhase::kBaselinesForStandaloneAxes && !item.IsSubgrid() {
                continue;
            }
            let mut child = std::ptr::null_mut();
            if item.IsSubgrid() {
                let index = if phase == BaselineCollectionPhase::kBaselinesForStandaloneAxes {
                    subtree.LookupSubgridSubtreeIndex(item)
                } else {
                    subtree.LookupSubgridIndex(item)
                };
                child = MakeGarbageCollected(GridLayoutSubtree::new(tree, index));
                if unsafe { &*child }.HasUnresolvedGeometry() {
                    assert_eq!(phase, BaselineCollectionPhase::kBaselinesForTrackSizing);
                    subtree.SetHasDeferredSubgridBaseline();
                    continue;
                }
            }
            let data = if item.is_subgridded_to_parent_grid {
                subtree.LookupSubgriddedItemData(item)
            } else {
                SubgriddedItemData::new(item, layout, mode)
            };
            let space = if phase == BaselineCollectionPhase::kBaselinesForTrackSizing {
                self.CreateConstraintSpaceForMeasure(&data, direction, None)
            } else {
                self.CreateConstraintSpaceForLayout(
                    &data,
                    child,
                    None,
                    LayoutUnit::default(),
                    false,
                    None,
                    None,
                )
            };
            if CalculateInitialFragmentGeometry(&space, &item.node, std::ptr::null(), false)
                .border_box_size
                .inline_size
                == kIndefiniteSize
            {
                continue;
            }
            let result = LayoutGridItemForMeasure(item, &space, constraint, after);
            MeasureAndStoreItemBaseline(
                unsafe { &*result },
                item,
                &data,
                &space,
                direction,
                item.parent_grid_font_baseline,
                mode,
                unsafe { &mut *layout },
            );
        }
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:1039-1104
    pub fn InitializeTrackSizes(
        &self,
        subtree: &GridSizingSubtree,
        data: &SubgriddedItemData,
        direction: Option<GridTrackSizingDirection>,
    ) {
        debug_assert!(subtree.HasValidRootFor(self.Node()));
        let layout = subtree.LayoutData();
        let items = subtree.GetGridItems();
        let style = self.Style();
        let bsp = self.BorderScrollbarPadding();
        for d in direction.map_or_else(|| vec![kForColumns, kForRows], |d| vec![d]) {
            InitializeTrackCollection(
                data,
                style,
                self.GetConstraintSpace(),
                bsp,
                self.GetGridAvailableSize(),
                d,
                unsafe { &mut *layout },
            );
            if unsafe { &*layout }.HasSubgriddedAxis(d) {
                let tracks = unsafe {
                    &*if d == kForColumns {
                        (*layout).Columns()
                    } else {
                        (*layout).Rows()
                    }
                };
                let mut it = unsafe { &mut *items }.begin();
                let end = unsafe { &mut *items }.end();
                while it.NotEqual(&end) {
                    unsafe { &mut *it.Get() }.ComputeSetIndices(tracks);
                    it.Advance();
                }
            } else {
                let tracks = unsafe { &mut *(*layout).SizingCollection(d) };
                GridTrackSizingAlgorithm::CacheGridItemsProperties(&tracks.base, unsafe {
                    &mut *items
                });
                if !tracks.base.HasNonDefiniteTrack() {
                    let first = GridTrackSizingAlgorithm::ComputeFirstSetGeometry(
                        tracks,
                        style,
                        &self.grid_available_size_,
                        bsp,
                    );
                    tracks.FinalizeSetsGeometry(first.start_offset, first.gutter_size);
                } else {
                    if data.IsPresent() {
                        subtree.SetSubgridHasIndefiniteStandaloneAxis();
                    }
                    tracks.CacheInitializedSetsGeometry(if d == kForColumns {
                        bsp.inline_start
                    } else {
                        bsp.block_start
                    });
                }
                if unsafe { &*layout }.HasBaselines(d) {
                    unsafe { &mut *layout }.ResetBaselines(d, tracks.base.GetSetCount());
                }
            }
        }
        InitializeTrackSizesForEachSubgrid(subtree, self, direction);
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:1106-1112
    fn InitializeTrackSizesTree(
        &self,
        tree: &mut GridSizingTree,
        direction: Option<GridTrackSizingDirection>,
    ) {
        self.InitializeTrackSizes(
            &GridSizingSubtree::new(tree, 0),
            &kNoSubgriddedItemData,
            direction,
        );
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:1165-1190
    fn ComputeUsedTrackSizes(
        &self,
        subtree: &GridSizingSubtree,
        direction: GridTrackSizingDirection,
        constraint: SizingConstraint,
    ) {
        debug_assert!(subtree.HasValidRootFor(self.Node()));
        let style = self.Style();
        let tracks = unsafe { &mut *(*subtree.LayoutData()).SizingCollection(direction) };
        tracks.BuildSets(style, &self.grid_available_size_);
        AccommodateSubgridExtraMargins(subtree, tracks, direction);
        GridTrackSizingAlgorithm::new(
            style,
            &self.grid_available_size_,
            &self.grid_min_available_size_,
            constraint,
        )
        .ComputeUsedTrackSizes(
            &mut |kind, item| {
                self.ContributionSizeForGridItem(subtree, kind, direction, constraint, item)
            },
            tracks,
            unsafe { &mut *subtree.GetGridItems() },
            false,
        );
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:1192-1262
    pub fn CompleteTrackSizingAlgorithm(
        &self,
        subtree: &GridSizingSubtree,
        data: &SubgriddedItemData,
        direction: GridTrackSizingDirection,
        constraint: SizingConstraint,
        additional: *mut bool,
    ) {
        debug_assert!(subtree.HasValidRootFor(self.Node()));
        let layout = subtree.LayoutData();
        let style = self.Style();
        let bsp = self.BorderScrollbarPadding();
        let columns = direction == kForColumns;
        let non_definite = unsafe {
            &*if columns {
                (*layout).Columns()
            } else {
                (*layout).Rows()
            }
        }
        .HasNonDefiniteTrack();
        if non_definite {
            if unsafe { &*layout }.HasSubgriddedAxis(direction) {
                debug_assert!(data.IsSubgrid());
                unsafe { &mut *layout }.SetTrackCollection(CreateSubgridTrackCollection(
                    data,
                    style,
                    self.GetConstraintSpace(),
                    bsp,
                    self.GetGridAvailableSize(),
                    direction,
                ));
            } else {
                self.ComputeUsedTrackSizes(subtree, direction, constraint);
                let check = !columns && !additional.is_null() && !unsafe { *additional };
                let tracks = unsafe { &mut *(*layout).SizingCollection(direction) };
                let dependent = if check {
                    BlockSizeDependentGridItems(unsafe { &*subtree.GetGridItems() }, tracks)
                } else {
                    Vec::new()
                };
                let first = GridTrackSizingAlgorithm::ComputeFirstSetGeometry(
                    tracks,
                    style,
                    &self.grid_available_size_,
                    bsp,
                );
                tracks.FinalizeSetsGeometry(first.start_offset, first.gutter_size);
                if check {
                    unsafe {
                        *additional =
                            MayChangeBlockSizeDependentGridItemContributions(&dependent, tracks);
                    }
                }
            }
        }
        ForEachSubgrid(
            subtree,
            self,
            &mut |algo, child, data| {
                algo.CompleteTrackSizingAlgorithm(
                    child,
                    data,
                    data.RelativeDirectionInSubgrid(direction),
                    constraint,
                    additional,
                )
            },
            true,
        );
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:1264-1281
    fn CompleteTrackSizingAlgorithmTree(
        &self,
        direction: GridTrackSizingDirection,
        constraint: SizingConstraint,
        tree: &mut GridSizingTree,
        additional: *mut bool,
    ) {
        let subtree = GridSizingSubtree::new(tree, 0);
        ValidateMinMaxSizesCache(self.Node(), &subtree, direction);
        self.ComputeBaselineAlignment(
            tree.FinalizeTree(),
            &subtree,
            &kNoSubgriddedItemData,
            Some(direction),
            constraint,
            false,
        );
        self.CompleteTrackSizingAlgorithm(
            &subtree,
            &kNoSubgriddedItemData,
            direction,
            constraint,
            additional,
        );
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:1283-1337
    pub fn ComputeBaselineAlignment(
        &self,
        tree: *const GridLayoutTree,
        subtree: &GridSizingSubtree,
        data: &SubgriddedItemData,
        direction: Option<GridTrackSizingDirection>,
        constraint: SizingConstraint,
        after: bool,
    ) {
        debug_assert!(subtree.HasValidRootFor(self.Node()));
        let layout = subtree.LayoutData();
        for d in direction.map_or_else(|| vec![kForColumns, kForRows], |d| vec![d]) {
            if unsafe { &*layout }.HasSubgriddedAxis(d) {
                debug_assert!(data.IsSubgrid());
                let columns = if data.is_parallel_with_root_grid {
                    d == kForColumns
                } else {
                    d == kForRows
                };
                let baseline_direction = if columns { kForColumns } else { kForRows };
                let parent = unsafe { &*data.ParentLayoutData() }.GetBaselines(baseline_direction);
                if !parent.is_null() {
                    unsafe { &mut *layout }.SetBaselines(
                        d,
                        CreateSubgridBaselines(
                            data,
                            self.Style(),
                            self.GetConstraintSpace(),
                            self.BorderScrollbarPadding(),
                            self.GetGridAvailableSize(),
                            d,
                            unsafe { &*parent },
                        ),
                    );
                }
            } else {
                self.ComputeGridItemBaselines(
                    tree,
                    subtree,
                    d,
                    constraint,
                    if direction.is_some() {
                        BaselineCollectionPhase::kBaselinesForTrackSizing
                    } else {
                        BaselineCollectionPhase::kFinalBaselines
                    },
                    after,
                );
            }
        }
        ComputeBaselineAlignmentForEachSubgrid(subtree, self, tree, direction, constraint, after);
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:1339-1370
    pub fn ResolveBaselinesInStandaloneAxes(
        &self,
        subtree: &GridSizingSubtree,
        tree: *mut GridSizingTree,
        constraint: SizingConstraint,
        after: bool,
    ) {
        ForEachSubgrid(
            subtree,
            self,
            &mut |algo, child, _| {
                algo.ResolveBaselinesInStandaloneAxes(child, tree, constraint, after)
            },
            true,
        );
        let layout = unsafe { &*subtree.LayoutData() };
        if layout.HasSubgriddedAxis(kForColumns) && layout.HasSubgriddedAxis(kForRows) {
            return;
        }
        let layout_tree = subtree.FinalizeTree();
        for direction in [kForColumns, kForRows] {
            if !layout.HasSubgriddedAxis(direction) {
                self.ComputeGridItemBaselines(
                    layout_tree,
                    subtree,
                    direction,
                    constraint,
                    BaselineCollectionPhase::kBaselinesForStandaloneAxes,
                    after,
                );
            }
        }
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:1372-1378
    fn CompleteFinalBaselineAlignment(&self, tree: &mut GridSizingTree) {
        self.ComputeBaselineAlignment(
            tree.FinalizeTree(),
            &GridSizingSubtree::new(tree, 0),
            &kNoSubgriddedItemData,
            None,
            SizingConstraint::kLayout,
            false,
        );
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:1380-1394
    fn ComputeSubgridIntrinsicSize(
        &self,
        subtree: &GridSizingSubtree,
        direction: GridTrackSizingDirection,
        constraint: SizingConstraint,
    ) -> LayoutUnit {
        debug_assert!(subtree.HasValidRootFor(self.Node()));
        self.ComputeUsedTrackSizes(subtree, direction, constraint);
        (if direction == kForColumns {
            self.BorderScrollbarPadding().InlineSum()
        } else {
            self.BorderScrollbarPadding().BlockSum()
        }) + unsafe { &*(*subtree.LayoutData()).SizingCollection(direction) }.TotalTrackSize()
    }
}
// cpp: layoutng_grid/grid_layout_algorithm.cc:1116-1119
struct BlockSizeDependentGridItem {
    row_set_indices: GridItemIndices,
    cached_block_size: LayoutUnit,
}
// cpp: layoutng_grid/grid_layout_algorithm.cc:1121-1144
fn BlockSizeDependentGridItems(
    items: &GridItems,
    tracks: &GridSizingTrackCollection,
) -> Vec<BlockSizeDependentGridItem> {
    debug_assert_eq!(tracks.base.Direction(), kForRows);
    let mut dependent = Vec::with_capacity(items.Size() as usize);
    let mut it = items.begin_const();
    let end = items.end_const();
    while it.NotEqual(&end) {
        let item = unsafe { &*it.Get() };
        it.Advance();
        if !item.is_sizing_dependent_on_block_size {
            continue;
        }
        let indices = *item.SetIndices(kForRows);
        dependent.push(BlockSizeDependentGridItem {
            row_set_indices: indices,
            cached_block_size: tracks
                .base
                .CalculateSetSpanSizeRange(indices.begin, indices.end),
        });
    }
    dependent
}
// cpp: layoutng_grid/grid_layout_algorithm.cc:1146-1161
fn MayChangeBlockSizeDependentGridItemContributions(
    items: &[BlockSizeDependentGridItem],
    tracks: &GridSizingTrackCollection,
) -> bool {
    debug_assert_eq!(tracks.base.Direction(), kForRows);
    for item in items {
        let size = tracks
            .base
            .CalculateSetSpanSizeRange(item.row_set_indices.begin, item.row_set_indices.end);
        debug_assert_ne!(size, kIndefiniteSize);
        if size != item.cached_block_size {
            return true;
        }
    }
    false
}

impl GridLayoutAlgorithm {
    // cpp: layoutng_grid/grid_layout_algorithm.cc:1396-1446
    fn CreateConstraintSpace(
        &self,
        slot: LayoutResultCacheSlot,
        item: &GridItemData,
        area: &LogicalSize,
        fixed: &LogicalSize,
        subtree: *const GridLayoutSubtree,
        min_encompass: bool,
        child_offset: Option<LayoutUnit>,
    ) -> ConstraintSpace {
        let parent = self.GetConstraintSpace();
        let mut builder = ConstraintSpaceBuilder::new_with_inline_size_adjustment(
            parent,
            item.node.Style().GetWritingDirection(),
            true,
            false,
        );
        builder.SetCacheSlot(slot);
        builder.SetIsPaintedAtomically(true);
        let mut available = *area;
        if fixed.inline_size != kIndefiniteSize {
            available.inline_size = fixed.inline_size;
            builder.SetIsFixedInlineSize(true);
        }
        if fixed.block_size != kIndefiniteSize {
            available.block_size = fixed.block_size;
            builder.SetIsFixedBlockSize(true);
        }
        builder.SetAvailableSize(available);
        if !subtree.is_null() {
            debug_assert!(item.IsSubgrid());
            debug_assert!(!unsafe { &*subtree }.HasUnresolvedGeometry());
            builder.SetGridLayoutSubtree(subtree);
        }
        builder.SetPercentageResolutionSize(*area);
        builder.SetInlineAutoBehavior(item.column_auto_behavior);
        builder.SetBlockAutoBehavior(item.row_auto_behavior);
        if parent.HasBlockFragmentation() {
            if let Some(offset) = child_offset {
                if min_encompass {
                    builder.SetMinBlockSizeShouldEncompassIntrinsicSize();
                }
                SetupSpaceBuilderForFragmentationFromBuilder(
                    &self.container_builder_,
                    &item.node,
                    offset,
                    &mut builder,
                );
            }
        }
        builder.ToConstraintSpace()
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:1448-1496
    pub fn CreateConstraintSpaceForLayout(
        &self,
        item: &SubgriddedItemData,
        subtree: *const GridLayoutSubtree,
        area_out: Option<&mut LogicalRect>,
        unavailable: LayoutUnit,
        min_encompass: bool,
        child_offset: Option<LayoutUnit>,
        _fixed_inline: Option<LayoutUnit>,
    ) -> ConstraintSpace {
        let mode = self.GetConstraintSpace().GetWritingMode();
        let mut inline = LayoutUnit::default();
        let mut block = LayoutUnit::default();
        let mut area = LogicalSize::new(
            item.CalculateAvailableSize(item.Columns(Some(mode)), &mut inline),
            item.CalculateAvailableSize(item.Rows(Some(mode)), &mut block),
        );
        if let Some(out) = area_out {
            out.offset = LogicalOffset::new(inline, block);
            out.size = area;
        }
        if area.block_size != kIndefiniteSize {
            area.block_size -= unavailable;
            debug_assert!(area.block_size >= LayoutUnit::default());
        }
        let mut fixed = kIndefiniteLogicalSize;
        if item.IsSubgrid() {
            let shrunk = ShrinkLogicalSize(
                area,
                &ComputeMarginsForInlineSize(
                    item.node.Style(),
                    area.inline_size,
                    self.GetConstraintSpace().GetWritingDirection(),
                ),
            );
            fixed = LogicalSize::new(
                if item.has_subgridded_columns {
                    shrunk.inline_size
                } else {
                    kIndefiniteSize
                },
                if item.has_subgridded_rows {
                    shrunk.block_size
                } else {
                    kIndefiniteSize
                },
            );
        }
        self.CreateConstraintSpace(
            LayoutResultCacheSlot::kLayout,
            item,
            &area,
            &fixed,
            subtree,
            min_encompass,
            child_offset,
        )
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:1498-1538
    fn CreateConstraintSpaceForMeasure(
        &self,
        item: &SubgriddedItemData,
        direction: GridTrackSizingDirection,
        fixed_inline: Option<LayoutUnit>,
    ) -> ConstraintSpace {
        let mut area = kIndefiniteLogicalSize;
        let mode = self.GetConstraintSpace().GetWritingMode();
        if direction == kForColumns {
            area.block_size =
                item.CalculateAvailableSize(item.Rows(Some(mode)), std::ptr::null_mut());
        } else {
            area.inline_size =
                item.CalculateAvailableSize(item.Columns(Some(mode)), std::ptr::null_mut());
        }
        let mut fixed = if item.IsSubgrid() {
            ShrinkLogicalSize(
                area,
                &ComputeMarginsForInlineSize(
                    item.node.Style(),
                    area.inline_size,
                    self.GetConstraintSpace().GetWritingDirection(),
                ),
            )
        } else {
            kIndefiniteLogicalSize
        };
        if let Some(size) = fixed_inline {
            let parallel = IsParallelWritingMode(item.node.Style().GetWritingMode(), mode);
            let target = if parallel {
                &mut fixed.inline_size
            } else {
                &mut fixed.block_size
            };
            debug_assert_eq!(*target, kIndefiniteSize);
            *target = size;
        }
        self.CreateConstraintSpace(
            LayoutResultCacheSlot::kMeasure,
            item,
            &area,
            &fixed,
            std::ptr::null(),
            false,
            None,
        )
    }
}
// cpp: layoutng_grid/grid_layout_algorithm.cc:1542-1727
struct GapAccumulator {
    gap_geometry_: *mut GapGeometry,
    row_gap_data_: GridTrackGapData,
    column_gap_data_: GridTrackGapData,
    col_gutter_size_: LayoutUnit,
    row_gutter_size_: LayoutUnit,
    main_gaps_aggregator_: GapSegmentStateAggregator,
    cross_gaps_aggregator_: GapSegmentStateAggregator,
}
impl GapAccumulator {
    // cpp: layoutng_grid/grid_layout_algorithm.cc:1546-1548
    fn new() -> Self {
        Self {
            gap_geometry_: MakeGarbageCollected(GapGeometry::new(ContainerType::kGrid)),
            row_gap_data_: Default::default(),
            column_gap_data_: Default::default(),
            col_gutter_size_: Default::default(),
            row_gutter_size_: Default::default(),
            main_gaps_aggregator_: GapSegmentStateAggregator::new(0),
            cross_gaps_aggregator_: GapSegmentStateAggregator::new(0),
        }
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:1558-1570
    fn BuildMainGaps(&mut self, data: &GridLayoutData) {
        let rows = unsafe { &*data.Rows() };
        self.row_gap_data_ = BuildGridTrackGapData(rows, GridTrackGapType::kMain, unsafe {
            &mut *self.gap_geometry_
        });
        self.row_gutter_size_ = rows.GutterSize();
        self.cross_gaps_aggregator_ =
            GapSegmentStateAggregator::new(self.row_gap_data_.track_count);
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:1572-1581
    fn BuildCrossGaps(&mut self, data: &GridLayoutData) {
        let columns = unsafe { &*data.Columns() };
        self.column_gap_data_ = BuildGridTrackGapData(columns, GridTrackGapType::kCross, unsafe {
            &mut *self.gap_geometry_
        });
        self.col_gutter_size_ = columns.GutterSize();
        self.main_gaps_aggregator_ =
            GapSegmentStateAggregator::new(self.column_gap_data_.track_count);
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:1583-1586
    fn BuildGapGeometry(&mut self, data: &GridLayoutData) {
        self.BuildMainGaps(data);
        self.BuildCrossGaps(data);
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:1602-1607
    fn AggregateCellStates(&mut self, item: &GridItemData) {
        self.main_gaps_aggregator_
            .ProcessItem(item.Span(kForRows), item.Span(kForColumns));
        self.cross_gaps_aggregator_
            .ProcessItem(item.Span(kForColumns), item.Span(kForRows));
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:1615-1659
    fn GetRowGapToSetIndicesMap(&self, data: &GridLayoutData) -> Vec<u32> {
        let rows = unsafe { &*data.Rows() };
        let count = rows.RangeCount();
        let mut map = Vec::new();
        for range in 0..count {
            let sets = rows.RangeSetCount(range);
            let begin = rows.RangeBeginSetIndex(range);
            let tracks = rows.RangeTrackCount(range);
            for index in 0..tracks {
                if range == count - 1 && index == tracks - 1 {
                    break;
                }
                if sets != 0 {
                    map.push(begin + index % sets);
                }
                assert!(map.len() <= 10_000_000);
                if map.len() == 10_000_000 {
                    return map;
                }
            }
        }
        map
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:1661-1695
    fn FinalizeGapGeometry(
        &mut self,
        rows: &GridLayoutTrackCollection,
        columns: &GridLayoutTrackCollection,
    ) -> *const GapGeometry {
        let geometry = unsafe { &mut *self.gap_geometry_ };
        if geometry.MainGapCount() == 0 && geometry.CrossGapCount() == 0 {
            return std::ptr::null();
        }
        geometry.SetInlineGapSize(self.col_gutter_size_);
        geometry.SetBlockGapSize(self.row_gutter_size_);
        if self.main_gaps_aggregator_.GetCellCount() > 0 && geometry.MainGapCount() > 0 {
            self.FinalizeMainGapRanges(rows);
        }
        if self.cross_gaps_aggregator_.GetCellCount() > 0 && geometry.CrossGapCount() > 0 {
            self.FinalizeCrossGapRanges(columns);
        }
        let geometry = unsafe { &mut *self.gap_geometry_ };
        geometry.SetContentInlineOffsets(
            self.column_gap_data_.content_start,
            self.column_gap_data_.content_end,
        );
        geometry.SetContentBlockOffsets(
            self.row_gap_data_.content_start,
            self.row_gap_data_.content_end,
        );
        self.gap_geometry_
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:1700-1709
    fn FinalizeMainGapRanges(&mut self, rows: &GridLayoutTrackCollection) {
        assert_eq!(rows.Direction(), kForRows);
        let geometry = unsafe { &mut *self.gap_geometry_ };
        let mut index = 0;
        for gap in &self.row_gap_data_.gaps {
            self.main_gaps_aggregator_
                .FinalizeMainGapSegmentStateRangesFor(
                    geometry.MainGapAtMut(index),
                    gap.line_index - 1,
                );
            index += 1;
        }
        assert_eq!(index, geometry.MainGapCount());
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:1710-1719
    fn FinalizeCrossGapRanges(&mut self, columns: &GridLayoutTrackCollection) {
        assert_eq!(columns.Direction(), kForColumns);
        let geometry = unsafe { &mut *self.gap_geometry_ };
        let mut index = 0;
        for gap in &self.column_gap_data_.gaps {
            self.cross_gaps_aggregator_
                .FinalizeCrossGapSegmentStateRangesFor(
                    geometry.CrossGapAt(index),
                    gap.line_index - 1,
                );
            index += 1;
        }
        assert_eq!(index, geometry.CrossGapCount());
    }
}
impl GridLayoutAlgorithm {
    // cpp: layoutng_grid/grid_layout_algorithm.cc:1731-1888
    fn PlaceGridItems(
        &mut self,
        items: &GridItems,
        subtree: &GridLayoutSubtree,
        row_breaks: &mut Vec<EBreakBetween>,
        mut placements: Option<&mut Vec<GridItemPlacementData>>,
        gap_out: Option<&mut *const GapGeometry>,
        track_map: Option<&mut Vec<u32>>,
    ) {
        let space = self.GetConstraintSpace() as *const ConstraintSpace;
        let data = unsafe { &*subtree.LayoutData() };
        let propagate = unsafe { &*space }.ShouldPropagateChildBreakValues();
        if propagate {
            *row_breaks =
                vec![EBreakBetween::kAuto; unsafe { &*data.Rows() }.GetSetCount() as usize + 1];
        }
        let mut baselines = GridBaselineAccumulator::new(self.Style().GetFontBaseline());
        let direction = unsafe { &*space }.GetWritingDirection();
        let mode = unsafe { &*space }.GetWritingMode();
        let mut next = subtree.FirstChild();
        let mut gaps = if self.Style().HasGapRule() || gap_out.is_some() {
            let mut gap = GapAccumulator::new();
            gap.BuildGapGeometry(data);
            if let Some(map) = track_map {
                *map = gap.GetRowGapToSetIndicesMap(data);
            }
            Some(gap)
        } else {
            None
        };
        let mut it = items.begin_const();
        let end = items.end_const();
        while it.NotEqual(&end) {
            let item = unsafe { &*it.Get() };
            it.Advance();
            let mut child = std::ptr::null();
            if item.IsSubgrid() {
                debug_assert!(!next.is_null());
                child = next;
                next = unsafe { &*next }.NextSibling();
            }
            let mut area = LogicalRect::default();
            let child_space = self.CreateConstraintSpaceForLayout(
                &SubgriddedItemData::new(item, subtree.LayoutData(), mode),
                child,
                Some(&mut area),
                LayoutUnit::default(),
                false,
                None,
                None,
            );
            let style = item.node.Style();
            let margins = ComputeMarginsFor(&child_space, style, unsafe { &*space });
            let result = item.node.Layout(
                &child_space,
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
            );
            let physical =
                unsafe { &*To::<PhysicalBoxFragment>(unsafe { &*result }.GetPhysicalFragment()) };
            let fragment = LogicalBoxFragment::new(direction, physical);
            let inline_baseline = ComputeBaselineOffset(
                item,
                data,
                &LogicalBoxFragment::new(item.BaselineWritingDirection(kForColumns), physical),
                &fragment,
                item.parent_grid_font_baseline,
                kForColumns,
                area.size.inline_size,
            );
            let block_baseline = ComputeBaselineOffset(
                item,
                data,
                &LogicalBoxFragment::new(item.BaselineWritingDirection(kForRows), physical),
                &fragment,
                item.parent_grid_font_baseline,
                kForRows,
                area.size.block_size,
            );
            area.offset += LogicalOffset::new(
                AlignmentOffset(
                    area.size.inline_size,
                    fragment.InlineSize(),
                    margins.inline_start,
                    margins.inline_end,
                    inline_baseline,
                    item.Alignment(kForColumns),
                    item.IsOverflowSafe(kForColumns),
                ),
                AlignmentOffset(
                    area.size.block_size,
                    fragment.BlockSize(),
                    margins.block_start,
                    margins.block_end,
                    block_baseline,
                    item.Alignment(kForRows),
                    item.IsOverflowSafe(kForRows),
                ),
            );
            let mut relative = LogicalOffset::default();
            if style.GetPosition() == foundation::EPosition::kRelative {
                relative += ComputeRelativeOffsetForBoxFragment(physical, direction, &area.size);
            }
            if let Some(placements) = placements.as_deref_mut() {
                placements.push(GridItemPlacementData::new(
                    area.offset,
                    unsafe { &*result }.HasDescendantThatDependsOnPercentageBlockSize(),
                    Some(relative),
                ));
            } else {
                self.container_builder_.AddResult(
                    unsafe { &*result },
                    area.offset,
                    Some(margins),
                    Some(relative),
                    std::ptr::null(),
                );
                baselines.AccumulateItem(item, &fragment, area.offset.block_offset);
            }
            if propagate {
                let before = JoinFragmentainerBreakValues(
                    style.BreakBefore(),
                    unsafe { &*result }.InitialBreakBefore(),
                );
                let after = JoinFragmentainerBreakValues(
                    style.BreakAfter(),
                    unsafe { &*result }.FinalBreakAfter(),
                );
                let indices = item.SetIndices(kForRows);
                row_breaks[indices.begin as usize] =
                    JoinFragmentainerBreakValues(row_breaks[indices.begin as usize], before);
                row_breaks[indices.end as usize] =
                    JoinFragmentainerBreakValues(row_breaks[indices.end as usize], after);
            }
            if let Some(gap) = gaps.as_mut() {
                gap.AggregateCellStates(item);
            }
        }
        if let Some(gap) = gaps.as_mut() {
            let geometry =
                gap.FinalizeGapGeometry(unsafe { &*data.Rows() }, unsafe { &*data.Columns() });
            if !geometry.is_null() {
                if let Some(out) = gap_out {
                    *out = geometry;
                } else {
                    self.container_builder_.SetGapGeometry(geometry);
                }
            }
        }
        if data.HasBaselines(kForRows) {
            baselines.AccumulateRows(unsafe { &*data.Rows() }, unsafe {
                &*data.GetBaselines(kForRows)
            });
        }
        if let Some(first) = baselines.FirstBaseline() {
            self.container_builder_.SetFirstBaseline(first);
        }
        if let Some(last) = baselines.LastBaseline() {
            self.container_builder_.SetLastBaseline(last);
        }
    }
}
// cpp: layoutng_grid/grid_layout_algorithm.cc:1892-1908
struct ResultAndOffsets {
    result: Member<LayoutResult>,
    offset: LogicalOffset,
    relative_offset: Option<LogicalOffset>,
}
impl foundation::Traceable for ResultAndOffsets {
    fn Trace(&self, visitor: &mut foundation::Visitor<'_>) {
        visitor.Trace(&self.result);
    }
}

// cpp: layoutng_grid/grid_layout_algorithm.cc:1992-2001
// The C++ PlaceItems lambda shares these variables across row-expansion passes.
struct FragmentationPlacementState {
    results: Vec<ResultAndOffsets>,
    baselines: GridBaselineAccumulator,
    max_row_expansion: LayoutUnit,
    max_item_block_end: LayoutUnit,
    expansion_row: u32,
    breakpoint_row: u32,
    has_subsequent_children: bool,
}
impl GridLayoutAlgorithm {
    // cpp: layoutng_grid/grid_layout_algorithm.cc:1950-1984
    fn MinBlockSizeShouldEncompassIntrinsicSize(&self, item: &GridItemData, depends: bool) -> bool {
        if depends || item.node.IsMonolithic() {
            return false;
        }
        let style = item.node.Style();
        debug_assert_eq!(
            self.GetConstraintSpace().GetWritingMode(),
            style.GetWritingMode()
        );
        if !style.LogicalHeight().HasAutoOrContentOrIntrinsic()
            && style.BoxDecorationBreak() != EBoxDecorationBreak::kClone
        {
            return false;
        }
        if item.SpanSize(kForRows) > 1 {
            return false;
        }
        if item.IsSpanningFixedMaximumTrack(kForRows) && !item.IsSpanningIntrinsicTrack(kForRows) {
            return false;
        }
        !item.IsSpanningFixedMinimumTrack(kForRows)
            || self.Style().LogicalHeight().HasAutoOrContentOrIntrinsic()
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:2029-2259
    fn PlaceFragmentationItems(
        &mut self,
        items: &GridItems,
        subtree: &GridLayoutSubtree,
        row_breaks: &[EBreakBetween],
        placements: &[GridItemPlacementData],
        row_adjustments: &[LayoutUnit],
        stitched: LayoutUnit,
        gap_adjustment: LayoutUnit,
        fragmentainer_space: LayoutUnit,
        cloned_start: LayoutUnit,
        previous_expansion_row: u32,
    ) -> FragmentationPlacementState {
        let space_ptr = self.GetConstraintSpace() as *const ConstraintSpace;
        let space = unsafe { &*space_ptr };
        let direction = space.GetWritingDirection();
        let mut state = FragmentationPlacementState {
            results: Vec::new(),
            baselines: GridBaselineAccumulator::new(self.Style().GetFontBaseline()),
            max_row_expansion: LayoutUnit::default(),
            max_item_block_end: LayoutUnit::default(),
            expansion_row: u32::MAX,
            breakpoint_row: u32::MAX,
            has_subsequent_children: false,
        };
        let mut next_subgrid = subtree.FirstChild();
        let tokens = if self.GetBreakToken().is_null() {
            &[][..]
        } else {
            unsafe { &*self.GetBreakToken() }.ChildBreakTokens()
        };
        let mut token_index = 0;
        let data = unsafe { &*subtree.LayoutData() };
        let mut it = items.begin_const();
        let end = items.end_const();
        let mut index = 0;
        while it.NotEqual(&end) {
            let item = unsafe { &*it.Get() };
            it.Advance();
            let placement = &placements[index];
            index += 1;
            let mut token: *const BlockBreakToken = std::ptr::null();
            if token_index < tokens.len() {
                let child = To::<BlockBreakToken>(tokens[token_index].Get());
                if unsafe { &*child }.InputNode() == item.node {
                    token = child;
                    token_index += 1;
                }
            }
            let child_offset = if IsBreakInside(token) {
                self.BorderScrollbarPadding().block_start
            } else {
                placement.offset.block_offset - stitched + cloned_start
            };
            let fragment_offset = self.FragmentainerOffsetForChildren() + child_offset;
            let encompass = self.MinBlockSizeShouldEncompassIntrinsicSize(
                item,
                placement.has_descendant_that_depends_on_percentage_block_size,
            );
            let mut unavailable = LayoutUnit::default();
            if IsBreakInside(self.GetBreakToken()) && IsBreakInside(token) {
                let parent_data = unsafe {
                    &*To::<GridBreakTokenData>(unsafe { &*self.GetBreakToken() }.TokenData())
                };
                unavailable = parent_data.offset_in_stitched_container
                    - (placement.offset.block_offset + unsafe { &*token }.ConsumedBlockSize());
            }
            let mut child_subgrid = std::ptr::null_mut();
            if item.IsSubgrid() {
                debug_assert!(!next_subgrid.is_null());
                child_subgrid = next_subgrid;
                next_subgrid = unsafe { &*next_subgrid }.NextSibling();
            }
            let mut area = LogicalRect::default();
            let child_space = self.CreateConstraintSpaceForLayout(
                &SubgriddedItemData::new(item, data, direction.GetWritingMode()),
                child_subgrid,
                Some(&mut area),
                unavailable,
                encompass,
                Some(child_offset),
                None,
            );
            let row = item.SetIndices(kForRows).begin;
            area.offset.block_offset += row_adjustments[row as usize] + gap_adjustment - stitched;
            if fragmentainer_space != kIndefiniteSize
                && area.offset.block_offset >= fragmentainer_space
            {
                if space.IsInsideBalancedColumns() && !space.IsInitialColumnBalancingPass() {
                    let _disable = layoutng_assembly::internal::disable_layout_side_effects_scope::DisableLayoutSideEffectsScope::new();
                    let result =
                        item.node
                            .Layout(&child_space, token, std::ptr::null(), std::ptr::null());
                    let capacity = self.FragmentainerCapacityForChildren();
                    PropagateSpaceShortageDefault(
                        result,
                        fragment_offset,
                        capacity,
                        &mut *self.container_builder_,
                    );
                }
                state.has_subsequent_children = true;
                continue;
            } else if area.offset.block_offset < LayoutUnit::default()
                && token.is_null()
                && IsBreakInside(self.GetBreakToken())
            {
                continue;
            }
            let result = item
                .node
                .Layout(&child_space, token, std::ptr::null(), std::ptr::null());
            debug_assert_eq!(unsafe { &*result }.Status(), EStatus::kSuccess);
            state.results.push(ResultAndOffsets {
                result: Member::from_ptr(result.cast_mut()),
                offset: LogicalOffset::new(placement.offset.inline_offset, child_offset),
                relative_offset: placement.relative_offset,
            });
            let fragment = LogicalBoxFragment::new(direction, unsafe {
                &*To::<PhysicalBoxFragment>(unsafe { &*result }.GetPhysicalFragment())
            });
            state
                .baselines
                .AccumulateItem(item, &fragment, child_offset);
            let separation = area.offset.block_offset > LayoutUnit::default();
            if separation && row < state.breakpoint_row {
                let between = row_breaks[row as usize];
                if IsForcedBreakValue(space, between) {
                    self.container_builder_.SetHasForcedBreak();
                    state.breakpoint_row = state.breakpoint_row.min(row);
                    continue;
                }
                self.container_builder_.SetPreviousBreakAfter(between);
                let appeal = CalculateBreakAppealBeforeChild(
                    space,
                    item.node.base.clone(),
                    unsafe { &*result },
                    &self.container_builder_,
                    separation,
                );
                let capacity = self.FragmentainerCapacityForChildren();
                if !MovePastBreakpointFull(
                    space,
                    item.node.base.clone(),
                    unsafe { &*result },
                    fragment_offset,
                    capacity,
                    appeal,
                    std::ptr::null_mut(),
                    false,
                    std::ptr::null_mut(),
                ) {
                    state.breakpoint_row = state.breakpoint_row.min(row);
                    PropagateSpaceShortageDefault(
                        result,
                        fragment_offset,
                        capacity,
                        &mut *self.container_builder_,
                    );
                    if IsAvoidBreakValue(space, between) {
                        let mut preceding = row as i32 - 1;
                        while preceding >= 0 {
                            let i = preceding as usize;
                            let offset = unsafe { &*data.Rows() }.GetSetOffset(i as u32)
                                + row_adjustments[i]
                                + gap_adjustment
                                - stitched;
                            if offset <= LayoutUnit::default() {
                                break;
                            }
                            if row_breaks[i] == EBreakBetween::kAuto {
                                state.breakpoint_row = state.breakpoint_row.min(i as u32);
                                break;
                            }
                            preceding -= 1;
                        }
                    }
                    continue;
                }
            }
            if space.GetGridLayoutSubtree().is_null()
                && encompass
                && row <= state.expansion_row
                && (previous_expansion_row == u32::MAX || row > previous_expansion_row)
                && fragmentainer_space != kIndefiniteSize
                && area.BlockEndOffset() <= fragmentainer_space
            {
                if state.expansion_row != row {
                    state.expansion_row = row;
                    state.max_row_expansion = LayoutUnit::default();
                }
                let expansion = if !unsafe { &*result }
                    .GetPhysicalFragment()
                    .GetBreakToken()
                    .is_null()
                {
                    (fragmentainer_space - area.BlockEndOffset()).AddEpsilon()
                } else {
                    fragment.BlockSize() - area.BlockEndOffset()
                };
                state.max_row_expansion = state.max_row_expansion.max(expansion);
            }
            state.max_item_block_end = state
                .max_item_block_end
                .max(child_offset + fragment.BlockSize());
        }
        state
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:2265-2278
    fn AdjustItemOffsets(
        items: &GridItems,
        placements: &mut [GridItemPlacementData],
        row: u32,
        delta: LayoutUnit,
        exact: bool,
    ) {
        let mut it = items.begin_const();
        for placement in placements {
            let current_row = it.Current().SetIndices(kForRows).begin;
            it.Advance();
            if if exact {
                current_row == row
            } else {
                current_row >= row
            } {
                placement.offset.block_offset += delta;
            }
        }
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:1910-2549
    fn PlaceGridItemsForFragmentation(
        &mut self,
        items: &GridItems,
        subtree: &GridLayoutSubtree,
        row_breaks: &[EBreakBetween],
        full_gap_geometry: *const GapGeometry,
        track_to_set: &[u32],
        column_segment_starts: &mut Vec<u32>,
        placements: &mut Vec<GridItemPlacementData>,
        row_adjustments: &mut Vec<LayoutUnit>,
        intrinsic: &mut LayoutUnit,
        stitched: &mut LayoutUnit,
        gap_adjustment: &mut LayoutUnit,
        first_gap: &mut u32,
    ) {
        let mut fragmentainer_space = self.FragmentainerSpaceLeftForChildren();
        let mut cloned_start = LayoutUnit::default();
        if fragmentainer_space != kIndefiniteSize {
            cloned_start = ClonedBlockStartDecoration(&self.container_builder_);
            fragmentainer_space -= cloned_start;
        }
        let data = unsafe { &*subtree.LayoutData() };
        let mut previous_expansion = u32::MAX;
        let mut state = self.PlaceFragmentationItems(
            items,
            subtree,
            row_breaks,
            placements,
            row_adjustments,
            *stitched,
            *gap_adjustment,
            fragmentainer_space,
            cloned_start,
            previous_expansion,
        );
        // cpp: layoutng_grid/grid_layout_algorithm.cc:2287-2301
        while state.max_row_expansion != LayoutUnit::default() {
            debug_assert!(state.max_row_expansion > LayoutUnit::default());
            debug_assert!(
                previous_expansion == u32::MAX || state.expansion_row > previous_expansion
            );
            *intrinsic += state.max_row_expansion;
            Self::AdjustItemOffsets(
                items,
                placements,
                state.expansion_row + 1,
                state.max_row_expansion,
                false,
            );
            unsafe { &mut *data.Rows() }
                .AdjustSetOffsets(state.expansion_row + 1, state.max_row_expansion);
            previous_expansion = state.expansion_row;
            state = self.PlaceFragmentationItems(
                items,
                subtree,
                row_breaks,
                placements,
                row_adjustments,
                *stitched,
                *gap_adjustment,
                fragmentainer_space,
                cloned_start,
                previous_expansion,
            );
        }
        // cpp: layoutng_grid/grid_layout_algorithm.cc:2306-2348
        let mut shifted = false;
        if state.breakpoint_row != u32::MAX {
            let row_offset = unsafe { &*data.Rows() }.GetSetOffset(state.breakpoint_row)
                + row_adjustments[state.breakpoint_row as usize]
                + *gap_adjustment;
            let relative = row_offset - *stitched;
            if fragmentainer_space == kIndefiniteSize {
                fragmentainer_space = relative;
                shifted = true;
            } else {
                let delta = fragmentainer_space - relative;
                if delta > LayoutUnit::default() {
                    *intrinsic += delta;
                    Self::AdjustItemOffsets(items, placements, state.breakpoint_row, delta, false);
                    for adjustment in &mut row_adjustments[state.breakpoint_row as usize..] {
                        *adjustment += delta;
                    }
                    shifted = true;
                }
            }
        }
        if shifted {
            state = self.PlaceFragmentationItems(
                items,
                subtree,
                row_breaks,
                placements,
                row_adjustments,
                *stitched,
                *gap_adjustment,
                fragmentainer_space,
                cloned_start,
                previous_expansion,
            );
        } else if fragmentainer_space != kIndefiniteSize {
            fragmentainer_space = fragmentainer_space.max(state.max_item_block_end - cloned_start);
        }
        // cpp: layoutng_grid/grid_layout_algorithm.cc:2356-2499
        if !full_gap_geometry.is_null() && fragmentainer_space != kIndefiniteSize {
            use layoutng_assembly::internal::gap::main_gap::MainGap;
            let geometry = unsafe { &*full_gap_geometry };
            let initial_first_gap = *first_gap;
            let mut main_gaps = Vec::new();
            let mut current_set = u32::MAX;
            let half_gap = geometry.GetBlockGapSize() / 2i32;
            for gap_index in *first_gap as usize..geometry.GetMainGaps().len() {
                let mut midpoint =
                    geometry.GetMainGaps()[gap_index].GetGapOffset() + *gap_adjustment;
                assert!(gap_index < track_to_set.len());
                current_set = track_to_set[gap_index];
                midpoint += row_adjustments[current_set as usize];
                midpoint -= *stitched;
                if midpoint - half_gap > fragmentainer_space {
                    if !main_gaps.is_empty() {
                        current_set = track_to_set[gap_index - 1];
                    }
                    break;
                }
                main_gaps.push(MainGap::with_new_offset(
                    &geometry.GetMainGaps()[gap_index],
                    midpoint,
                ));
                *first_gap = gap_index as u32 + 1;
            }
            if let Some(last) = main_gaps.last() {
                let last_end = last.GetGapOffset() + half_gap;
                let next_offset = unsafe { &*data.Rows() }.GetSetOffset(current_set + 1)
                    + row_adjustments[current_set as usize + 1]
                    + *gap_adjustment
                    - *stitched;
                if next_offset >= fragmentainer_space || last_end >= fragmentainer_space {
                    let spill = (last_end - fragmentainer_space).ClampNegativeToZero();
                    if spill > LayoutUnit::default() {
                        let single = last.HasBlockedRange();
                        if single {
                            unsafe { &mut *data.Rows() }
                                .AdjustSingleSetOffset(current_set + 1, -spill);
                        } else {
                            *gap_adjustment -= spill;
                            *intrinsic -= spill;
                        }
                        Self::AdjustItemOffsets(items, placements, current_set + 1, -spill, single);
                    }
                    main_gaps.pop();
                }
            }
            if self.Style().HasGapRule()
                && (!main_gaps.is_empty() || !geometry.GetCrossGaps().is_empty())
            {
                // Preserve the source condition, including its block-start choice.
                let start = if *stitched > LayoutUnit::default() {
                    geometry.GetContentBlockStart()
                } else {
                    LayoutUnit::default()
                };
                let mut end = fragmentainer_space.min(*intrinsic - *stitched);
                if state.max_item_block_end > fragmentainer_space {
                    end = state.max_item_block_end;
                }
                let fragment_geometry = MakeGarbageCollected(GapGeometry::with_fragment_offsets(
                    geometry, main_gaps, start, end,
                ));
                unsafe { &mut *fragment_geometry }.AdjustCrossGapsRangesForFragmentation(
                    initial_first_gap,
                    *first_gap,
                    column_segment_starts,
                );
                self.container_builder_.SetGapGeometry(fragment_geometry);
            }
        }
        if state.has_subsequent_children {
            self.container_builder_.SetHasSubsequentChildren();
        }
        for entry in state.results {
            self.container_builder_.AddResult(
                unsafe { &*entry.result.Get() },
                entry.offset,
                None,
                entry.relative_offset,
                std::ptr::null(),
            );
        }
        if let Some(first) = state.baselines.FirstBaseline() {
            self.container_builder_.SetFirstBaseline(first);
        }
        if let Some(last) = state.baselines.LastBaseline() {
            self.container_builder_.SetLastBaseline(last);
        }
        if fragmentainer_space != kIndefiniteSize {
            *stitched += fragmentainer_space;
        }
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:2551-2629
    fn PlaceOutOfFlowItems(
        &mut self,
        data: &GridLayoutData,
        block_size: LayoutUnit,
        children: &mut HeapVector<Member<LayoutBox>>,
    ) {
        debug_assert!(!children.is_empty());
        let oofs = std::mem::take(children);
        let process_end =
            !InvolvedInBlockFragmentation(self.GetConstraintSpace(), self.GetBreakToken())
                || (!self.container_builder_.DidBreakSelf()
                    && !self.container_builder_.ShouldBreakInside());
        let node = self.Node().clone();
        let style_ptr = self.Style() as *const ComputedStyle;
        let style = unsafe { &*style_ptr };
        let placement = node.CachedPlacementData();
        let absolute = node.IsAbsoluteContainer();
        let fixed = node.IsAbsoluteContainer();
        let previous = if self.GetBreakToken().is_null() {
            LayoutUnit::default()
        } else {
            unsafe { &*self.GetBreakToken() }.ConsumedBlockSize()
        };
        let total_size = LogicalSize::new(self.container_builder_.InlineSize(), block_size);
        let default_size = ShrinkLogicalSize(total_size, &self.BorderScrollbarPadding());
        let border_scrollbar = *self.Borders() + *self.Scrollbar();
        let padding_box = LogicalRect::new(
            border_scrollbar.StartOffset(),
            ShrinkLogicalSize(total_size, &border_scrollbar),
        );
        for child in oofs {
            let item = MakeGarbageCollected(GridItemData::new_with_parent_style(
                BlockNode::new(child.Get()),
                style,
            ));
            let item_ref = unsafe { &mut *item };
            debug_assert!(item_ref.IsOutOfFlow());
            let position = item_ref.node.Style().GetPosition();
            let containing = if (absolute && position == foundation::EPosition::kAbsolute)
                || (fixed && position == foundation::EPosition::kFixed)
            {
                Some(ComputeGridOutOfFlowItemContainingRect(
                    placement,
                    data,
                    style,
                    &padding_box,
                    item_ref,
                ))
            } else {
                None
            };
            let mut static_pos = LogicalStaticPosition::default();
            static_pos.offset = containing
                .map(|r| r.offset)
                .unwrap_or(self.BorderScrollbarPadding().StartOffset());
            let containing_size = containing.map(|r| r.size).unwrap_or(default_size);
            AlignmentOffsetForOutOfFlow(
                item_ref.Alignment(kForColumns),
                item_ref.Alignment(kForRows),
                containing_size,
                &mut static_pos,
            );
            static_pos.offset.block_offset -= previous;
            if process_end
                || static_pos.offset.block_offset <= self.FragmentainerCapacityForChildren()
            {
                self.container_builder_
                    .AddOutOfFlowChildCandidateDefault(&item_ref.node, &static_pos);
            } else {
                children.push(child);
            }
        }
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:2631-2684
    fn SetReadingFlowNodes(&mut self, items: &GridItems) {
        let flow = self.Style().ReadingFlow();
        if !matches!(
            flow,
            EReadingFlow::kGridRows | EReadingFlow::kGridColumns | EReadingFlow::kGridOrder
        ) {
            return;
        }
        let mut nodes = Vec::with_capacity(items.Size() as usize);
        let mut reordered: Vec<*const GridItemData> = Vec::with_capacity(items.Size() as usize);
        let mut it = items.begin_const();
        let end = items.end_const();
        while it.NotEqual(&end) {
            reordered.push(it.Get());
            it.Advance();
        }
        if flow == EReadingFlow::kGridRows || flow == EReadingFlow::kGridColumns {
            let (first, second) = if flow == EReadingFlow::kGridColumns {
                (kForColumns, kForRows)
            } else {
                (kForRows, kForColumns)
            };
            reordered.sort_by_key(|p| {
                let item = unsafe { &**p };
                (item.SetIndices(first).begin, item.SetIndices(second).begin)
            });
        }
        for pointer in reordered {
            let dom = unsafe { &*pointer }.node.GetDOMNode();
            if !dom.is_null() {
                nodes.push(Member::from_ptr(dom));
            }
        }
        self.container_builder_.SetReadingFlowNodes(nodes.into());
    }
    // cpp: layoutng_grid/grid_layout_algorithm.cc:69-257
    fn LayoutInternal(&mut self) -> *const LayoutResult {
        let _delay = DelayScrollOffsetClampScope::new();
        let mut items = std::ptr::null_mut();
        let mut intrinsic = LayoutUnit::default();
        let mut oofs = HeapVector::default();
        let subtree: *const GridLayoutSubtree;
        if IsBreakInside(self.GetBreakToken()) {
            let token_data = unsafe {
                &*To::<GridBreakTokenData>(unsafe { &*self.GetBreakToken() }.TokenData())
            };
            items = token_data.grid_items.Get();
            subtree = token_data.grid_layout_subtree.Get();
            intrinsic = token_data.intrinsic_block_size;
            if self.Style().BoxDecorationBreak() == EBoxDecorationBreak::kClone
                && !unsafe { &*self.GetBreakToken() }.IsAtBlockEnd()
            {
                intrinsic += self.BorderScrollbarPadding().BlockSum();
            }
        } else {
            subtree = self.ComputeGridGeometry(&mut items, &mut intrinsic, &mut oofs);
        }
        let data = unsafe { &*unsafe { &*subtree }.LayoutData() };
        let mut stitched = LayoutUnit::default();
        let mut previous = LayoutUnit::default();
        let mut placements = Vec::new();
        let mut row_adjustments = Vec::new();
        let mut row_breaks = Vec::new();
        let mut full_geometry = std::ptr::null();
        let mut track_to_set = Vec::new();
        let mut column_segments = Vec::new();
        let mut gap_adjustment = LayoutUnit::default();
        let mut first_gap = 0;
        if InvolvedInBlockFragmentation(self.GetConstraintSpace(), self.GetBreakToken()) {
            if IsBreakInside(self.GetBreakToken()) {
                let token_data = unsafe {
                    &*To::<GridBreakTokenData>(unsafe { &*self.GetBreakToken() }.TokenData())
                };
                stitched = token_data.offset_in_stitched_container;
                previous = stitched;
                placements = token_data.grid_items_placement_data.clone();
                row_adjustments = token_data.row_offset_adjustments.clone();
                row_breaks = token_data.row_break_between.clone();
                oofs = token_data.oof_children.clone();
                full_geometry = token_data.full_gap_geometry.Get();
                track_to_set = token_data.track_idx_to_set_idx.clone();
                column_segments = token_data.column_gaps_segment_ranges_start_indices.clone();
                gap_adjustment = token_data.cumulative_gap_offset_adjustment;
                first_gap = token_data.first_unprocessed_row_gap_idx;
            } else {
                row_adjustments = vec![
                    LayoutUnit::default();
                    unsafe { &*data.Rows() }.GetSetCount() as usize + 1
                ];
                let columns = unsafe { &*data.Columns() }.EndLineOfImplicitGrid();
                if columns > 1 {
                    column_segments = vec![0; columns as usize - 1];
                }
                self.PlaceGridItems(
                    unsafe { &*items },
                    unsafe { &*subtree },
                    &mut row_breaks,
                    Some(&mut placements),
                    Some(&mut full_geometry),
                    Some(&mut track_to_set),
                );
            }
            self.PlaceGridItemsForFragmentation(
                unsafe { &*items },
                unsafe { &*subtree },
                &row_breaks,
                full_geometry,
                &track_to_set,
                &mut column_segments,
                &mut placements,
                &mut row_adjustments,
                &mut intrinsic,
                &mut stitched,
                &mut gap_adjustment,
                &mut first_gap,
            );
        } else {
            self.PlaceGridItems(
                unsafe { &*items },
                unsafe { &*subtree },
                &mut row_breaks,
                None,
                None,
                None,
            );
        }
        let node = self.Node().clone();
        let space_ptr = self.GetConstraintSpace() as *const ConstraintSpace;
        let space = unsafe { &*space_ptr };
        let block_size = ComputeBlockSizeForFragment(
            space,
            &node,
            &self.BorderPadding(),
            intrinsic,
            self.container_builder_.InlineSize(),
            kIndefiniteSize,
        );
        if node.IsScrollContainer() {
            let offset = LogicalOffset::new(
                unsafe { &*data.Columns() }.GetSetOffset(0),
                unsafe { &*data.Rows() }.GetSetOffset(0),
            );
            let size = LogicalSize::new(
                unsafe { &*data.Columns() }.CalculateSetSpanSize(),
                unsafe { &*data.Rows() }.CalculateSetSpanSize(),
            );
            self.container_builder_
                .SetInflowBounds(LogicalRect::new(offset, size));
        }
        self.container_builder_
            .SetMayHaveDescendantAboveBlockStart(false);
        self.container_builder_
            .SetHasDescendantThatDependsOnPercentageBlockSize(false);
        let fragment_intrinsic = if space.HasKnownFragmentainerBlockSize() {
            stitched - previous + self.BorderScrollbarPadding().block_end
        } else {
            intrinsic
        };
        self.container_builder_
            .SetIntrinsicBlockSize(fragment_intrinsic);
        self.container_builder_.SetFragmentsTotalBlockSize(
            if node.HasCachedPlacementData()
                && !node.CachedPlacementData().HasStandaloneAxis(kForRows)
            {
                intrinsic
            } else {
                block_size
            },
        );
        if InvolvedInBlockFragmentation(self.GetConstraintSpace(), self.GetBreakToken()) {
            let status = FinishFragmentation(&mut self.container_builder_);
            if status == BreakStatus::kDisableFragmentation {
                return self
                    .container_builder_
                    .Abort(EStatus::kDisableFragmentation);
            }
            debug_assert_eq!(status, BreakStatus::kContinue);
        } else {
            #[cfg(debug_assertions)]
            self.container_builder_.CheckNoBlockFragmentation();
        }
        if space.ShouldPropagateChildBreakValues() {
            self.container_builder_
                .SetInitialBreakBefore(*row_breaks.first().unwrap());
            self.container_builder_
                .SetPreviousBreakAfter(*row_breaks.last().unwrap());
        }
        if !oofs.is_empty() {
            self.PlaceOutOfFlowItems(data, block_size, &mut oofs);
        }
        self.container_builder_
            .SetGridLayoutData(data as *const GridLayoutData);
        self.SetReadingFlowNodes(unsafe { &*items });
        if space.HasBlockFragmentation() {
            let token_data = MakeGarbageCollected(GridBreakTokenData::new(
                items,
                subtree,
                intrinsic,
                stitched,
                &placements,
                &row_adjustments,
                &row_breaks,
                &oofs,
                full_geometry,
                &mut track_to_set,
                &mut column_segments,
                gap_adjustment,
                first_gap,
            ));
            self.container_builder_.SetBreakTokenData(token_data.cast());
        }
        self.container_builder_.HandleOofsAndSpecialDescendants();
        self.container_builder_.ToBoxFragment()
    }
}
