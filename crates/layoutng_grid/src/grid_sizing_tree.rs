use foundation::{
    HeapHashMap, HeapVector, IsParallelWritingMode, MakeGarbageCollected, Member, Traceable,
    Visitor, WritingMode,
};
use layoutng_assembly::internal::{
    block_node::BlockNode,
    grid_item::{GridItemData, GridItems},
    grid_lanes_item_group::{GridLanesItemGroups, VirtualItems},
    grid_layout_data::{GridLayoutData, GridLayoutTree, GridTreeNode as LayoutTreeNode},
    grid_subtree::{GridSubtree, GridSubtreeTree},
    grid_track_collection::{GridLayoutTrackCollection, GridSizingTrackCollection},
    layout_box::LayoutBox,
};
use layoutng_style::style::grid_enums::GridTrackSizingDirection::{self, *};
use std::ops::Deref;

// cpp: layoutng_grid/grid_sizing_tree.h:23-81
#[derive(Clone, Copy)]
pub struct SubgriddedItemData {
    item_data_in_parent_: *const GridItemData,
    parent_layout_data_: *const GridLayoutData,
    parent_writing_mode_: WritingMode,
}
impl Default for SubgriddedItemData {
    fn default() -> Self {
        kNoSubgriddedItemData
    }
}
impl SubgriddedItemData {
    // cpp: layoutng_grid/grid_sizing_tree.h:29-34
    pub fn new(item: &GridItemData, layout: *const GridLayoutData, mode: WritingMode) -> Self {
        Self {
            item_data_in_parent_: item,
            parent_layout_data_: layout,
            parent_writing_mode_: mode,
        }
    }
    // cpp: layoutng_grid/grid_sizing_tree.h:36-46
    pub fn IsPresent(&self) -> bool {
        !self.item_data_in_parent_.is_null()
    }
    pub fn IsSubgrid(&self) -> bool {
        self.IsPresent() && self.deref().IsSubgrid()
    }
    // cpp: layoutng_grid/grid_sizing_tree.h:48-58
    pub fn Columns(&self, mode: Option<WritingMode>) -> &GridLayoutTrackCollection {
        debug_assert!(!self.parent_layout_data_.is_null());
        let parent = unsafe { &*self.parent_layout_data_ };
        unsafe {
            &*if mode.map_or(true, |mode| {
                IsParallelWritingMode(mode, self.parent_writing_mode_)
            }) {
                parent.Columns()
            } else {
                parent.Rows()
            }
        }
    }
    // cpp: layoutng_grid/grid_sizing_tree.h:60-70
    pub fn Rows(&self, mode: Option<WritingMode>) -> &GridLayoutTrackCollection {
        debug_assert!(!self.parent_layout_data_.is_null());
        let parent = unsafe { &*self.parent_layout_data_ };
        unsafe {
            &*if mode.map_or(true, |mode| {
                IsParallelWritingMode(mode, self.parent_writing_mode_)
            }) {
                parent.Rows()
            } else {
                parent.Columns()
            }
        }
    }
    // cpp: layoutng_grid/grid_sizing_tree.h:72-72
    pub fn ParentLayoutData(&self) -> *const GridLayoutData {
        self.parent_layout_data_
    }
}
impl Deref for SubgriddedItemData {
    type Target = GridItemData;
    // cpp: layoutng_grid/grid_sizing_tree.h:38-43
    fn deref(&self) -> &Self::Target {
        debug_assert!(self.IsPresent());
        unsafe { &*self.item_data_in_parent_ }
    }
}
// cpp: layoutng_grid/grid_sizing_tree.h:81-81
pub const kNoSubgriddedItemData: SubgriddedItemData = SubgriddedItemData {
    item_data_in_parent_: std::ptr::null(),
    parent_layout_data_: std::ptr::null(),
    parent_writing_mode_: WritingMode::kHorizontalTb,
};

// cpp: layoutng_grid/grid_sizing_tree.h:90-110
pub struct GridTreeNode {
    pub grid_items: Member<GridItems>,
    pub virtual_items: Member<VirtualItems>,
    pub layout_data: Member<GridLayoutData>,
    pub subtree_size: u32,
    pub writing_mode: WritingMode,
}
impl Default for GridTreeNode {
    fn default() -> Self {
        Self {
            grid_items: Member::default(),
            virtual_items: Member::default(),
            layout_data: Member::default(),
            subtree_size: 1,
            writing_mode: WritingMode::kHorizontalTb,
        }
    }
}
impl Traceable for GridTreeNode {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.grid_items);
        visitor.Trace(&self.layout_data);
        visitor.Trace(&self.virtual_items);
    }
}
// cpp: layoutng_grid/grid_sizing_tree.h:191-194
#[derive(Clone, Copy)]
struct SubgriddedItemIndices {
    item_index_in_parent: u32,
    parent_grid_index: u32,
}
impl Traceable for SubgriddedItemIndices {
    fn Trace(&self, _visitor: &mut Visitor<'_>) {}
}

// cpp: layoutng_grid/grid_sizing_tree.h:86-233
#[derive(Default)]
pub struct GridSizingTree {
    subgrid_index_lookup_map_: HeapHashMap<Member<LayoutBox>, u32>,
    subgridded_item_data_lookup_map_: HeapHashMap<Member<LayoutBox>, SubgriddedItemIndices>,
    tree_data_: HeapVector<GridTreeNode>,
    tree_has_baselines_: bool,
    has_subgrid_with_indefinite_standalone_axis_: bool,
    has_block_size_dependent_grid_item_: bool,
    has_deferred_subgrid_baseline_: bool,
}
impl GridSizingTree {
    // cpp: layoutng_grid/grid_sizing_tree.cc:10-18
    pub fn AddToPreorderTraversal(&mut self, node: &BlockNode) {
        debug_assert!(node.IsGrid() || node.IsGridLanes());
        let key = Member::from_ptr(node.GetLayoutBox());
        debug_assert!(!self.subgrid_index_lookup_map_.contains_key(&key));
        self.subgrid_index_lookup_map_
            .insert(key, self.tree_data_.len() as u32);
        self.tree_data_.push(GridTreeNode::default());
    }
    // cpp: layoutng_grid/grid_sizing_tree.cc:20-73
    pub fn SetSizingNodeData(
        &mut self,
        node: &BlockNode,
        items: *mut GridItems,
        layout: *mut GridLayoutData,
        virtual_items: *mut VirtualItems,
    ) {
        debug_assert!(node.IsGrid() || node.IsGridLanes());
        let data = unsafe { &*layout };
        let items_ref = unsafe { &*items };
        let standalone_columns = !data.HasSubgriddedAxis(kForColumns);
        let standalone_rows = !data.HasSubgriddedAxis(kForRows);
        let node_index = self.LookupSubgridIndex(node);
        let mut child_index = node_index + 1;
        let mut current_item_index = 0;
        let mut it = items_ref.begin_const();
        let end = items_ref.end_const();
        while it.NotEqual(&end) {
            let item = it.Current();
            if item.IsSubgrid() {
                debug_assert_eq!(child_index, self.LookupSubgridIndex(&item.node));
                let size = self.SubtreeSize(child_index);
                self.AtMut(node_index).subtree_size += size;
                child_index += size;
            }
            if (standalone_columns || item.has_subgridded_columns)
                && (standalone_rows || item.has_subgridded_rows)
            {
                current_item_index += 1;
                it.Advance();
                continue;
            }
            let key = Member::from_ptr(item.node.GetLayoutBox());
            let indices = SubgriddedItemIndices {
                item_index_in_parent: current_item_index,
                parent_grid_index: node_index,
            };
            current_item_index += 1;
            debug_assert!(!self.subgridded_item_data_lookup_map_.contains_key(&key));
            self.subgridded_item_data_lookup_map_.insert(key, indices);
            it.Advance();
        }
        let tree_node = self.AtMut(node_index);
        tree_node.grid_items = Member::from_ptr(items);
        tree_node.virtual_items = Member::from_ptr(virtual_items);
        tree_node.layout_data = Member::from_ptr(layout);
        tree_node.writing_mode = node.Style().GetWritingMode();
        if data.HasBaselines(kForColumns) || data.HasBaselines(kForRows) {
            self.tree_has_baselines_ = true;
        }
    }
    // cpp: layoutng_grid/grid_sizing_tree.h:127-145
    pub fn GetGridItems(&self, index: u32) -> *mut GridItems {
        self.At(index).grid_items.Get()
    }
    pub fn GetVirtualItems(&self, index: u32) -> *mut GridItems {
        let v = self.At(index).virtual_items.Get();
        debug_assert!(!v.is_null());
        unsafe { &*v }.items.Get()
    }
    pub fn GetVirtualItemGroups(&self, index: u32) -> &GridLanesItemGroups {
        let v = self.At(index).virtual_items.Get();
        debug_assert!(!v.is_null());
        &unsafe { &*v }.item_groups
    }
    pub fn LayoutData(&self, index: u32) -> *mut GridLayoutData {
        self.At(index).layout_data.Get()
    }
    // cpp: layoutng_grid/grid_sizing_tree.cc:75-77
    pub fn FinalizeTree(&self) -> *const GridLayoutTree {
        self.FinalizeSubtreeAt(0)
    }
    // cpp: layoutng_grid/grid_sizing_tree.cc:79-124
    pub fn FinalizeSubtreeAt(&self, root: u32) -> *const GridLayoutTree {
        debug_assert!((root as usize) < self.tree_data_.len());
        let size = self.SubtreeSize(root);
        let mut data = HeapVector::default();
        data.reserve(size as usize);
        for i in 0..size {
            let tree_node = &self.tree_data_[(root + i) as usize];
            let layout = tree_node.layout_data.Get();
            let layout_ref = unsafe { &*layout };
            let needs_copy = self.tree_has_baselines_
                && (layout_ref.HasBaselines(kForColumns) || layout_ref.HasBaselines(kForRows));
            let mut finalized = layout;
            if needs_copy {
                finalized = MakeGarbageCollected(layout_ref.clone());
            }
            data.push(Member::from_ptr(MakeGarbageCollected(LayoutTreeNode::new(
                finalized,
                tree_node.subtree_size,
            ))));
        }
        for i in (1..=size).rev() {
            let subtree = data[(i - 1) as usize].Get();
            if unsafe { &*subtree }.has_unresolved_geometry
                && unsafe { &*subtree }.subtree_size == 1
            {
                continue;
            }
            let next_index = i + unsafe { &*subtree }.subtree_size - 1;
            let mut j = i;
            while j < next_index && !unsafe { &*subtree }.has_unresolved_geometry {
                debug_assert!(j < size);
                let child = unsafe { &*data[j as usize].Get() };
                unsafe { &mut *subtree }.has_unresolved_geometry = child.has_unresolved_geometry;
                j += child.subtree_size;
            }
        }
        MakeGarbageCollected(GridLayoutTree::new(data))
    }
    // cpp: layoutng_grid/grid_sizing_tree.cc:126-140
    pub fn LookupSubgriddedItemData(&self, item: &GridItemData) -> SubgriddedItemData {
        let key = Member::from_ptr(item.node.GetLayoutBox());
        debug_assert!(self.subgridded_item_data_lookup_map_.contains_key(&key));
        let indices = self.subgridded_item_data_lookup_map_.get(&key).unwrap();
        let node = self.At(indices.parent_grid_index);
        SubgriddedItemData::new(
            unsafe { &*node.grid_items.Get() }.AtConst(indices.item_index_in_parent),
            node.layout_data.Get(),
            node.writing_mode,
        )
    }
    // cpp: layoutng_grid/grid_sizing_tree.cc:142-150
    pub fn LookupSubgridIndex(&self, node: &BlockNode) -> u32 {
        let key = Member::from_ptr(node.GetLayoutBox());
        debug_assert!(self.subgrid_index_lookup_map_.contains_key(&key));
        *self.subgrid_index_lookup_map_.get(&key).unwrap()
    }
    // cpp: layoutng_grid/grid_sizing_tree.h:161-188
    pub fn Size(&self) -> u32 {
        self.tree_data_.len() as u32
    }
    pub fn SubtreeSize(&self, index: u32) -> u32 {
        self.At(index).subtree_size
    }
    pub fn HasSubgridWithIndefiniteStandaloneAxis(&self) -> bool {
        self.has_subgrid_with_indefinite_standalone_axis_
    }
    pub fn SetSubgridHasIndefiniteStandaloneAxis(&mut self) {
        self.has_subgrid_with_indefinite_standalone_axis_ = true;
    }
    pub fn HasBlockSizeDependentGridItem(&self) -> bool {
        self.has_block_size_dependent_grid_item_
    }
    pub fn SetHasBlockSizeDependentGridItem(&mut self) {
        self.has_block_size_dependent_grid_item_ = true;
    }
    pub fn HasDeferredSubgridBaseline(&self) -> bool {
        self.has_deferred_subgrid_baseline_
    }
    pub fn SetHasDeferredSubgridBaseline(&mut self) {
        self.has_deferred_subgrid_baseline_ = true;
    }
    // cpp: layoutng_grid/grid_sizing_tree.h:196-204
    fn At(&self, index: u32) -> &GridTreeNode {
        debug_assert!((index as usize) < self.tree_data_.len());
        &self.tree_data_[index as usize]
    }
    fn AtMut(&mut self, index: u32) -> &mut GridTreeNode {
        debug_assert!((index as usize) < self.tree_data_.len());
        &mut self.tree_data_[index as usize]
    }
}
impl GridSubtreeTree for GridSizingTree {
    fn SubtreeSize(&self, index: u32) -> u32 {
        GridSizingTree::SubtreeSize(self, index)
    }
}

// cpp: layoutng_grid/grid_sizing_tree.h:237-327
pub struct GridSizingSubtree {
    pub base: GridSubtree<GridSizingTree>,
    sizing_tree_: *mut GridSizingTree,
}
impl Default for GridSizingSubtree {
    fn default() -> Self {
        Self {
            base: GridSubtree::default(),
            sizing_tree_: std::ptr::null_mut(),
        }
    }
}
impl GridSizingSubtree {
    // cpp: layoutng_grid/grid_sizing_tree.h:243-247
    pub fn new(tree: *mut GridSizingTree, root: u32) -> Self {
        let mut result = Self {
            base: GridSubtree::default(),
            sizing_tree_: tree,
        };
        result.base.SetSubtreeRoot(unsafe { &*tree }, root);
        result
    }
    // cpp: layoutng_grid/grid_sizing_tree.h:318-320
    fn from_subtree(tree: *mut GridSizingTree, base: GridSubtree<GridSizingTree>) -> Self {
        Self {
            base,
            sizing_tree_: tree,
        }
    }
    // cpp: layoutng_grid/grid_sizing_tree.h:322-325
    fn SizingTree(&self) -> &GridSizingTree {
        debug_assert!(!self.sizing_tree_.is_null());
        unsafe { &*self.sizing_tree_ }
    }
    pub fn IsPresent(&self) -> bool {
        self.base.IsPresent()
    }
    // cpp: layoutng_grid/grid_sizing_tree.h:249-257
    pub fn FirstChild(&self) -> Self {
        Self::from_subtree(self.sizing_tree_, self.base.FirstChild(self.SizingTree()))
    }
    pub fn NextSibling(&self) -> Self {
        Self::from_subtree(self.sizing_tree_, self.base.NextSibling(self.SizingTree()))
    }
    // cpp: layoutng_grid/grid_sizing_tree.h:259-275
    pub fn LookupSubgriddedItemData(&self, item: &GridItemData) -> SubgriddedItemData {
        self.SizingTree().LookupSubgriddedItemData(item)
    }
    pub fn LookupSubgridIndex(&self, item: &GridItemData) -> u32 {
        debug_assert!(item.IsSubgrid());
        self.SizingTree().LookupSubgridIndex(&item.node)
    }
    pub fn LookupSubgridSubtreeIndex(&self, item: &GridItemData) -> u32 {
        self.LookupSubgridIndex(item) - self.base.SubtreeRoot()
    }
    // cpp: layoutng_grid/grid_sizing_tree.h:277-285
    pub fn SubgridSizingSubtree(&self, item: &GridItemData) -> Self {
        debug_assert!(item.IsSubgrid());
        Self::new(
            self.sizing_tree_,
            self.SizingTree().LookupSubgridIndex(&item.node),
        )
    }
    // cpp: layoutng_grid/grid_sizing_tree.h:289-292
    pub fn HasValidRootFor(&self, node: &BlockNode) -> bool {
        !self.sizing_tree_.is_null()
            && self.SizingTree().LookupSubgridIndex(node) == self.base.SubtreeRoot()
    }
    // cpp: layoutng_grid/grid_sizing_tree.h:294-310
    pub fn GetGridItems(&self) -> *mut GridItems {
        self.SizingTree().GetGridItems(self.base.SubtreeRoot())
    }
    pub fn GetVirtualItems(&self) -> *mut GridItems {
        self.SizingTree().GetVirtualItems(self.base.SubtreeRoot())
    }
    pub fn GetVirtualItemGroups(&self) -> &GridLanesItemGroups {
        self.SizingTree()
            .GetVirtualItemGroups(self.base.SubtreeRoot())
    }
    pub fn LayoutData(&self) -> *mut GridLayoutData {
        self.SizingTree().LayoutData(self.base.SubtreeRoot())
    }
    pub fn SizingCollection(
        &self,
        direction: GridTrackSizingDirection,
    ) -> *mut GridSizingTrackCollection {
        unsafe { &*self.LayoutData() }.SizingCollection(direction)
    }
    // cpp: layoutng_grid/grid_sizing_tree.h:312-326
    pub fn FinalizeTree(&self) -> *const GridLayoutTree {
        self.SizingTree().FinalizeSubtreeAt(self.base.SubtreeRoot())
    }
    pub fn SetSubgridHasIndefiniteStandaloneAxis(&self) {
        unsafe { &mut *self.sizing_tree_ }.SetSubgridHasIndefiniteStandaloneAxis();
    }
    pub fn SetHasBlockSizeDependentGridItem(&self) {
        unsafe { &mut *self.sizing_tree_ }.SetHasBlockSizeDependentGridItem();
    }
    pub fn SetHasDeferredSubgridBaseline(&self) {
        unsafe { &mut *self.sizing_tree_ }.SetHasDeferredSubgridBaseline();
    }
}
// The source constexpr empty subtree is returned by value because the shared
// generic Default constructor is not const; it owns no storage or references.
// cpp: layoutng_grid/grid_sizing_tree.h:332-332
pub fn NoGridSizingSubtree() -> GridSizingSubtree {
    GridSizingSubtree::default()
}
