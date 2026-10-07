use crate::{
    grid_data::GridPlacementData, grid_oof_placement::ComputeGridOutOfFlowItemContainingRect,
    subgrid_min_max_sizes_cache::SubgridMinMaxSizesCache,
};
use foundation::{LayoutUnit, MakeGarbageCollected, Member, Traceable, Visitor};
use layoutng_assembly::{
    internal::{
        block_node::BlockNode,
        fragmentation_utils::FindPreviousBreakToken,
        grid_item::GridItemData,
        grid_layout_data::GridLayoutData,
        grid_track_collection::GridLayoutTrackCollection,
        layout_block::LayoutBlock,
        layout_box::LayoutBox,
        layout_node_metadata::Element,
        layout_object::{LayoutObject, LayoutObjectClass, StyleChangeContext},
        min_max_sizes::MinMaxSizes,
    },
    physical_box_fragment::PhysicalBoxFragment,
};
use layoutng_geometry::geometry::logical_rect::LogicalRect;
use layoutng_style::style::{
    computed_style::ComputedStyle,
    grid_enums::GridTrackSizingDirection::{self, *},
    style_difference::StyleDifference,
};
use std::ops::{Deref, DerefMut};

// cpp: layoutng_grid/layout_grid.h:19-127
#[repr(C)]
pub struct LayoutGrid {
    block: LayoutBlock,
    cached_placement_data_: Option<GridPlacementData>,
    cached_subgrid_min_max_sizes_: Member<SubgridMinMaxSizesCache>,
}
impl Deref for LayoutGrid {
    type Target = LayoutBlock;
    fn deref(&self) -> &LayoutBlock {
        &self.block
    }
}
impl DerefMut for LayoutGrid {
    fn deref_mut(&mut self) -> &mut LayoutBlock {
        &mut self.block
    }
}
impl Traceable for LayoutGrid {
    fn Trace(&self, v: &mut Visitor<'_>) {
        LayoutGrid::Trace(self, v);
    }
}
impl LayoutGrid {
    // cpp: layoutng_grid/layout_grid.cc:20-20
    pub fn new(element: *mut Element) -> Self {
        // Preserve the downstream LayoutGrid override at the assembly's
        // LayoutObject virtual call site, without a Cargo dependency cycle.
        layoutng_assembly::RegisterGridStyleDidChange(
            |object, difference, old_style, new_style, context| {
                unsafe { &mut *object.cast::<LayoutGrid>() }
                    .StyleDidChange(difference, old_style, new_style, context);
            },
        );
        use layoutng_assembly::{GridObjectVirtuals, RegisterGridObjectVirtuals};
        RegisterGridObjectVirtuals(GridObjectVirtuals {
            add_child: |o, child, before| {
                unsafe { &mut *o.cast::<LayoutGrid>() }.AddChild(child, before)
            },
            remove_child: |o, child| unsafe { &mut *o.cast::<LayoutGrid>() }.RemoveChild(child),
            stitched_row_gap_index: |o, fragment, gap, line| {
                unsafe { &*o.cast::<LayoutGrid>() }.StitchedRowGapIndex(
                    fragment,
                    gap as u32,
                    line.map(|v| v as u32),
                ) as usize
            },
            adjust_anchor_containing_block: |o, data, style, padding, query| {
                unsafe { &*(o as *const LayoutBox).cast::<LayoutGrid>() }
                    .AdjustOutOfFlowContainingBlockForAnchor(data, style, padding, query)
            },
        });
        let block = LayoutBlock::new(element.cast());
        block.SetRuntimeClass(LayoutObjectClass::Grid);
        Self {
            block,
            cached_placement_data_: None,
            cached_subgrid_min_max_sizes_: Member::default(),
        }
    }
    // cpp: layoutng_grid/layout_grid.h:23-29
    pub fn GetName(&self) -> &'static str {
        self.CheckIsNotDestroyed();
        "LayoutGrid"
    }
    // cpp: layoutng_grid/layout_grid.h:102-105
    pub fn IsLayoutGrid(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }
    // cpp: layoutng_grid/layout_grid.cc:22-25
    pub fn Trace(&self, v: &mut Visitor<'_>) {
        self.block.Trace(v);
        v.Trace(&self.cached_subgrid_min_max_sizes_);
    }
    // cpp: layoutng_grid/layout_grid.cc:27-38
    pub fn AdjustOutOfFlowContainingBlockForAnchor(
        &self,
        layout: *const GridLayoutData,
        style: &ComputedStyle,
        padding: &LogicalRect,
        query: &LayoutBox,
    ) -> LogicalRect {
        let data = if layout.is_null() {
            self.LayoutData()
        } else {
            layout
        };
        let item = MakeGarbageCollected(GridItemData::new_with_parent_style(
            BlockNode::new((query as *const LayoutBox).cast_mut()),
            style,
        ));
        ComputeGridOutOfFlowItemContainingRect(
            self.CachedPlacementData(),
            unsafe { &*data },
            style,
            padding,
            unsafe { &mut *item },
        )
    }
    // cpp: layoutng_grid/layout_grid.cc:40-43
    pub fn MarkGridDirty(&mut self) {
        self.CheckIsNotDestroyed();
        self.SetGridPlacementDirty(true);
    }
    // cpp: layoutng_grid/layout_grid.cc:45-53
    pub fn AddChild(&mut self, child: *mut LayoutObject, before: *mut LayoutObject) {
        self.CheckIsNotDestroyed();
        self.block.AddChildBase(child, before);
        self.MarkGridDirty();
    }
    // cpp: layoutng_grid/layout_grid.cc:55-60
    pub fn RemoveChild(&mut self, child: *mut LayoutObject) {
        self.CheckIsNotDestroyed();
        self.block.RemoveChildBase(child);
        self.MarkGridDirty();
    }
    // cpp: layoutng_grid/layout_grid.cc:116-140
    pub fn GridPlacementInputsDidChange(
        new: &ComputedStyle,
        old: &ComputedStyle,
        diff: &StyleDifference,
        direction: Option<GridTrackSizingDirection>,
    ) -> bool {
        let new_areas = new.GridTemplateAreas().Get();
        let old_areas = old.GridTemplateAreas().Get();
        let areas_equal = new_areas == old_areas
            || (!new_areas.is_null()
                && !old_areas.is_null()
                && unsafe { &*new_areas == &*old_areas });
        if new.GetGridAutoFlow() != old.GetGridAutoFlow() || !areas_equal {
            return true;
        }
        if (direction.is_none() || direction == Some(kForColumns))
            && GridPlacementInputsDidChangeInDirection(new, old, diff, kForColumns)
        {
            return true;
        }
        if (direction.is_none() || direction == Some(kForRows))
            && GridPlacementInputsDidChangeInDirection(new, old, diff, kForRows)
        {
            return true;
        }
        false
    }
    // cpp: layoutng_grid/layout_grid.cc:142-155
    pub fn StyleDidChange(
        &mut self,
        diff: StyleDifference,
        old: *const ComputedStyle,
        new: &ComputedStyle,
        context: &StyleChangeContext,
    ) {
        self.CheckIsNotDestroyed();
        self.block.StyleDidChange(diff, old, new, context);
        if old.is_null() {
            return;
        }
        if Self::GridPlacementInputsDidChange(new, unsafe { &*old }, &diff, None) {
            self.SetGridPlacementDirty(true);
        }
    }
    // cpp: layoutng_grid/layout_grid.cc:157-159
    pub fn HasCachedPlacementData(&self) -> bool {
        self.cached_placement_data_.is_some() && !self.IsGridPlacementDirty()
    }
    // cpp: layoutng_grid/layout_grid.cc:161-164
    pub fn CachedPlacementData(&self) -> &GridPlacementData {
        debug_assert!(self.HasCachedPlacementData());
        self.cached_placement_data_.as_ref().unwrap()
    }
    // cpp: layoutng_grid/layout_grid.cc:166-169
    pub fn SetCachedPlacementData(&mut self, data: GridPlacementData) {
        self.cached_placement_data_ = Some(data);
        self.SetGridPlacementDirty(false);
    }
    // cpp: layoutng_grid/layout_grid.cc:171-173
    pub fn HasCachedSubgridMinMaxSizes(&self) -> bool {
        !self.cached_subgrid_min_max_sizes_.Get().is_null()
            && !self.IsSubgridMinMaxSizesCacheDirty()
    }
    // cpp: layoutng_grid/layout_grid.cc:175-178
    pub fn CachedSubgridMinMaxSizes(&self) -> &MinMaxSizes {
        debug_assert!(self.HasCachedSubgridMinMaxSizes());
        unsafe { &*self.cached_subgrid_min_max_sizes_.Get() }.CachedMinMaxSizes()
    }
    // cpp: layoutng_grid/layout_grid.cc:180-185
    pub fn SetSubgridMinMaxSizesCache(&mut self, sizes: MinMaxSizes, layout: &GridLayoutData) {
        self.cached_subgrid_min_max_sizes_ = Member::from_ptr(MakeGarbageCollected(
            SubgridMinMaxSizesCache::new(sizes, layout),
        ));
        self.SetSubgridMinMaxSizesCacheDirty(false);
    }
    // cpp: layoutng_grid/layout_grid.cc:187-191
    pub fn ShouldInvalidateSubgridMinMaxSizesCacheFor(&self, layout: &GridLayoutData) -> bool {
        self.HasCachedSubgridMinMaxSizes()
            && !unsafe { &*self.cached_subgrid_min_max_sizes_.Get() }.IsValidFor(layout)
    }
    // cpp: layoutng_grid/layout_grid.cc:193-195
    pub fn LayoutData(&self) -> *const GridLayoutData {
        Self::GetGridLayoutDataFromFragments(&self.block)
    }
    // cpp: layoutng_grid/layout_grid.cc:197-215
    pub fn StitchedRowGapIndex(
        &self,
        fragment: &PhysicalBoxFragment,
        gap: u32,
        line: Option<u32>,
    ) -> u32 {
        self.CheckIsNotDestroyed();
        assert!(!fragment.IsOnlyForNode());
        let previous = FindPreviousBreakToken(fragment);
        if previous.is_null() {
            return gap;
        }
        unsafe {
            &*foundation::To::<crate::grid_break_token_data::GridBreakTokenData>(
                (*previous).TokenData(),
            )
        }
        .GetFirstUnprocessedRowGapIndex(line)
            + gap
    }
    // cpp: layoutng_grid/layout_grid.cc:218-227
    pub fn GetGridLayoutDataFromFragments(block: &LayoutBlock) -> *const GridLayoutData {
        let count = block.PhysicalFragmentCount();
        if count == 0 {
            return std::ptr::null();
        }
        unsafe { &*block.GetLayoutResult(count - 1) }.GetGridLayoutData()
    }
    // cpp: layoutng_grid/layout_grid.cc:230-240
    pub fn ComputeGridGap(
        layout: *const GridLayoutData,
        direction: GridTrackSizingDirection,
    ) -> LayoutUnit {
        if layout.is_null() {
            return LayoutUnit::default();
        }
        let tracks = if direction == kForColumns {
            unsafe { &*layout }.Columns()
        } else {
            unsafe { &*layout }.Rows()
        };
        unsafe { &*tracks }.GutterSize()
    }
    // cpp: layoutng_grid/layout_grid.cc:242-248
    pub fn AutoRepeatCountForDirection(&self, direction: GridTrackSizingDirection) -> u32 {
        self.CheckIsNotDestroyed();
        if !self.HasCachedPlacementData() {
            return 0;
        }
        self.CachedPlacementData().AutoRepeatTrackCount(direction)
    }
    // cpp: layoutng_grid/layout_grid.cc:250-256
    pub fn ExplicitGridStartForDirection(&self, direction: GridTrackSizingDirection) -> u32 {
        self.CheckIsNotDestroyed();
        if !self.HasCachedPlacementData() {
            return 0;
        }
        self.CachedPlacementData().StartOffset(direction)
    }
    // cpp: layoutng_grid/layout_grid.cc:258-267
    pub fn ExplicitGridEndForDirection(&self, direction: GridTrackSizingDirection) -> u32 {
        self.CheckIsNotDestroyed();
        if !self.HasCachedPlacementData() {
            return 0;
        }
        self.ExplicitGridStartForDirection(direction)
            .checked_add(self.CachedPlacementData().ExplicitGridTrackCount(direction))
            .unwrap()
    }
    // cpp: layoutng_grid/layout_grid.cc:269-272
    pub fn GridGap(&self, direction: GridTrackSizingDirection) -> LayoutUnit {
        self.CheckIsNotDestroyed();
        Self::ComputeGridGap(self.LayoutData(), direction)
    }
    // cpp: layoutng_grid/layout_grid.cc:274-279
    pub fn GridItemOffset(&self, _direction: GridTrackSizingDirection) -> LayoutUnit {
        self.CheckIsNotDestroyed();
        LayoutUnit::default()
    }
    // cpp: layoutng_grid/layout_grid.cc:281-285
    pub fn TrackSizesForComputedStyle(
        &self,
        direction: GridTrackSizingDirection,
    ) -> Vec<LayoutUnit> {
        self.CheckIsNotDestroyed();
        Self::CollectTrackSizesForComputedStyle(self.LayoutData(), direction)
    }
    // cpp: layoutng_grid/layout_grid.cc:288-320
    pub fn CollectTrackSizesForComputedStyle(
        layout: *const GridLayoutData,
        direction: GridTrackSizingDirection,
    ) -> Vec<LayoutUnit> {
        if layout.is_null() {
            return Vec::new();
        }
        let tracks = unsafe {
            &*if direction == kForColumns {
                (*layout).Columns()
            } else {
                (*layout).Rows()
            }
        };
        let mut sizes = Vec::with_capacity(tracks.EndLineOfImplicitGrid().min(10_000_000) as usize);
        for i in 0..tracks.RangeCount() {
            let repeater = Self::ComputeTrackSizeRepeaterForRange(tracks, i);
            for j in 0..tracks.RangeTrackCount(i) {
                sizes.push(repeater[j as usize % repeater.len()]);
                debug_assert!(sizes.len() <= 10_000_000);
                if sizes.len() == 10_000_000 {
                    return sizes;
                }
            }
        }
        sizes
    }
    // cpp: layoutng_grid/layout_grid.cc:322-328
    pub fn GridTrackPositions(&self, direction: GridTrackSizingDirection) -> Vec<LayoutUnit> {
        self.CheckIsNotDestroyed();
        let layout = unsafe { &*self.LayoutData() };
        Self::ComputeExpandedPositions(unsafe {
            &*if direction == kForColumns {
                layout.Columns()
            } else {
                layout.Rows()
            }
        })
    }
    // cpp: layoutng_grid/layout_grid.cc:331-365
    pub fn ComputeTrackSizeRepeaterForRange(
        tracks: &GridLayoutTrackCollection,
        index: u32,
    ) -> Vec<LayoutUnit> {
        let count = tracks.RangeSetCount(index);
        if count == 0 {
            return vec![LayoutUnit::default()];
        }
        let mut sizes = Vec::with_capacity(count as usize);
        let begin = tracks.RangeBeginSetIndex(index);
        for i in begin..begin + count {
            let size = tracks.GetSetOffset(i + 1) - tracks.GetSetOffset(i);
            let track_count = tracks.GetSetTrackCount(i);
            debug_assert!(size >= LayoutUnit::default());
            let size = (size - tracks.GutterSize() * track_count).ClampNegativeToZero();
            debug_assert!(track_count > 0);
            sizes.push(size / track_count);
        }
        sizes
    }
    // cpp: layoutng_grid/layout_grid.cc:368-409
    pub fn ComputeExpandedPositions(tracks: &GridLayoutTrackCollection) -> Vec<LayoutUnit> {
        let mut positions = Vec::with_capacity(
            tracks
                .EndLineOfImplicitGrid()
                .wrapping_add(1)
                .min(10_000_001) as usize,
        );
        let mut offset = tracks.GetSetOffset(0);
        positions.push(offset);
        let mut last_gutter = LayoutUnit::default();
        'build: for i in 0..tracks.RangeCount() {
            let sizes = Self::ComputeTrackSizeRepeaterForRange(tracks, i);
            last_gutter = if tracks.RangeSetCount(i) != 0 {
                tracks.GutterSize()
            } else {
                LayoutUnit::default()
            };
            for j in 0..tracks.RangeTrackCount(i) {
                offset += sizes[j as usize % sizes.len()] + last_gutter;
                positions.push(offset);
                debug_assert!(positions.len() <= 10_000_001);
                if positions.len() == 10_000_001 {
                    break 'build;
                }
            }
        }
        *positions.last_mut().unwrap() -= last_gutter;
        positions
    }
}

// cpp: layoutng_grid/layout_grid.cc:66-116
fn GridPlacementInputsDidChangeInDirection(
    new: &ComputedStyle,
    old: &ComputedStyle,
    diff: &StyleDifference,
    direction: GridTrackSizingDirection,
) -> bool {
    let nt = new.TemplateTracks(direction);
    let n = nt.GetTrackList();
    if diff.NeedsFullLayout() && n.AutoRepeatTrackCount() != 0 {
        return true;
    }
    let ot = old.TemplateTracks(direction);
    let o = ot.GetTrackList();
    if n.TrackCountWithoutAutoRepeat() != o.TrackCountWithoutAutoRepeat()
        || n.AutoRepeatTrackCount() != o.AutoRepeatTrackCount()
    {
        return true;
    }
    if n != o {
        return true;
    }
    if nt.GetNamedGridLines() != ot.GetNamedGridLines() {
        return true;
    }
    new.AutoTracks(direction) != old.AutoTracks(direction)
}

// cpp: layoutng_grid/layout_grid.h:119-125
impl foundation::DowncastFrom<LayoutObject> for LayoutGrid {
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsLayoutGrid()
    }
}
