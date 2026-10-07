#![allow(non_snake_case)]

use foundation::{HashMap, HeapVector, LayoutUnit, MakeGarbageCollected, Member, Visitor};
use layoutng_style::style::grid_enums::GridTrackSizingDirection::{self, kForColumns, kForRows};
use layoutng_style::style::grid_track_size::{GridTrackSize, GridTrackSizeHashTraits};

use super::grid_subtree::{GridSubtree, GridSubtreeTree};
use super::grid_track_collection::{
    GridLayoutTrackCollection, GridSizingTrackCollection, GridTrackBaselines,
};

// Mirrors base::ValuesEquivalent: identical pointers compare equal; otherwise
// two non-null pointees are compared by value.
fn values_equivalent<T: PartialEq>(left: *const T, right: *const T) -> bool {
    if left == right {
        true
    } else if left.is_null() || right.is_null() {
        false
    } else {
        unsafe { &*left == &*right }
    }
}

// cpp: layoutng/internal/grid_layout_data.h:22-247
pub struct GridLayoutData {
    columns_: Member<GridLayoutTrackCollection>,
    rows_: Member<GridLayoutTrackCollection>,
    column_baselines_: Member<GridTrackBaselines>,
    row_baselines_: Member<GridTrackBaselines>,
    intrinsic_repeat_track_sizes_:
        Option<HashMap<GridTrackSize, LayoutUnit, GridTrackSizeHashTraits>>,
}

impl Default for GridLayoutData {
    // cpp: layoutng/internal/grid_layout_data.h:24-24
    fn default() -> Self {
        Self {
            columns_: Member::default(),
            rows_: Member::default(),
            column_baselines_: Member::default(),
            row_baselines_: Member::default(),
            intrinsic_repeat_track_sizes_: None,
        }
    }
}

impl Clone for GridLayoutData {
    // cpp: layoutng/internal/grid_layout_data.h:28-42
    fn clone(&self) -> Self {
        let mut copy = Self::default();
        copy.columns_ = self.columns_;
        copy.rows_ = self.rows_;
        let column_baselines = self.column_baselines_.Get();
        if !column_baselines.is_null() {
            copy.column_baselines_ =
                Member::from_ptr(MakeGarbageCollected(unsafe { &*column_baselines }.clone()));
        }
        let row_baselines = self.row_baselines_.Get();
        if !row_baselines.is_null() {
            copy.row_baselines_ =
                Member::from_ptr(MakeGarbageCollected(unsafe { &*row_baselines }.clone()));
        }
        copy
    }
}

impl PartialEq for GridLayoutData {
    // cpp: layoutng/internal/grid_layout_data.h:44-49
    fn eq(&self, other: &Self) -> bool {
        values_equivalent(self.columns_.Get(), other.columns_.Get())
            && values_equivalent(self.rows_.Get(), other.rows_.Get())
            && values_equivalent(self.column_baselines_.Get(), other.column_baselines_.Get())
            && values_equivalent(self.row_baselines_.Get(), other.row_baselines_.Get())
    }
}

impl GridLayoutData {
    fn track(&self, direction: GridTrackSizingDirection) -> *mut GridLayoutTrackCollection {
        if direction == kForColumns {
            self.columns_.Get()
        } else {
            self.rows_.Get()
        }
    }

    fn baselines(&self, direction: GridTrackSizingDirection) -> *mut GridTrackBaselines {
        if direction == kForColumns {
            self.column_baselines_.Get()
        } else {
            self.row_baselines_.Get()
        }
    }

    fn baselines_slot(
        &mut self,
        direction: GridTrackSizingDirection,
    ) -> &mut Member<GridTrackBaselines> {
        if direction == kForColumns {
            &mut self.column_baselines_
        } else {
            &mut self.row_baselines_
        }
    }

    // cpp: layoutng/internal/grid_layout_data.h:51-54
    pub fn HasSubgriddedAxis(&self, direction: GridTrackSizingDirection) -> bool {
        let track = self.track(direction);
        track.is_null() || !unsafe { &*track }.IsForSizing()
    }

    // cpp: layoutng/internal/grid_layout_data.h:56-58
    pub fn HasTrackCollection(&self, direction: GridTrackSizingDirection) -> bool {
        !self.track(direction).is_null()
    }

    // cpp: layoutng/internal/grid_layout_data.h:60-66
    pub fn IsSubgridWithStandaloneAxis(&self, direction: GridTrackSizingDirection) -> bool {
        let columns = self.columns_.Get();
        let rows = self.rows_.Get();
        if columns.is_null() || rows.is_null() {
            return false;
        }
        if direction == kForColumns {
            unsafe { &*columns }.IsForSizing() && !unsafe { &*rows }.IsForSizing()
        } else {
            unsafe { &*rows }.IsForSizing() && !unsafe { &*columns }.IsForSizing()
        }
    }

    // cpp: layoutng/internal/grid_layout_data.h:68-78
    pub fn Columns(&self) -> *mut GridLayoutTrackCollection {
        let columns = self.columns_.Get();
        debug_assert!(!columns.is_null());
        debug_assert_eq!(unsafe { &*columns }.Direction(), kForColumns);
        columns
    }

    pub fn Rows(&self) -> *mut GridLayoutTrackCollection {
        let rows = self.rows_.Get();
        debug_assert!(!rows.is_null());
        debug_assert_eq!(unsafe { &*rows }.Direction(), kForRows);
        rows
    }

    // cpp: layoutng/internal/grid_layout_data.h:80-85
    pub fn SizingCollection(
        &self,
        direction: GridTrackSizingDirection,
    ) -> *mut GridSizingTrackCollection {
        debug_assert!(!self.HasSubgriddedAxis(direction));
        let collection = if direction == kForColumns {
            self.Columns()
        } else {
            self.Rows()
        };
        // GridSizingTrackCollection embeds the collection base first, matching
        // the source's checked To<GridSizingTrackCollection> downcast.
        collection.cast::<GridSizingTrackCollection>()
    }

    // cpp: layoutng/internal/grid_layout_data.h:89-95
    pub fn OnlySubgriddedCollection(&self) -> *const GridLayoutTrackCollection {
        let columns = self.columns_.Get();
        let rows = self.rows_.Get();
        debug_assert!(!columns.is_null() && !rows.is_null());
        debug_assert_ne!(
            unsafe { &*columns }.IsForSizing(),
            unsafe { &*rows }.IsForSizing()
        );
        if unsafe { &*columns }.IsForSizing() {
            rows
        } else {
            columns
        }
    }

    // cpp: layoutng/internal/grid_layout_data.h:97-105
    pub fn SetTrackCollection(&mut self, collection: *mut GridLayoutTrackCollection) {
        debug_assert!(!collection.is_null());
        if unsafe { &*collection }.Direction() == kForColumns {
            self.columns_ = Member::from_ptr(collection);
        } else {
            self.rows_ = Member::from_ptr(collection);
        }
    }

    // cpp: layoutng/internal/grid_layout_data.h:109-112
    pub fn HasIndefiniteSet(&self) -> bool {
        (!self.columns_.Get().is_null() && unsafe { &*self.Columns() }.HasIndefiniteSet())
            || (!self.rows_.Get().is_null() && unsafe { &*self.Rows() }.HasIndefiniteSet())
    }

    // cpp: layoutng/internal/grid_layout_data.h:114-117
    pub fn HasBaselines(&self, direction: GridTrackSizingDirection) -> bool {
        !self.baselines(direction).is_null()
    }

    // cpp: layoutng/internal/grid_layout_data.h:121-130
    pub fn MajorBaseline(&self, direction: GridTrackSizingDirection, set_index: u32) -> LayoutUnit {
        let baselines = self.baselines(direction);
        if baselines.is_null() || set_index as usize >= unsafe { &*baselines }.major.len() {
            return LayoutUnit::Min();
        }
        unsafe { &*baselines }.major[set_index as usize]
    }

    // cpp: layoutng/internal/grid_layout_data.h:132-141
    pub fn MinorBaseline(&self, direction: GridTrackSizingDirection, set_index: u32) -> LayoutUnit {
        let baselines = self.baselines(direction);
        if baselines.is_null() || set_index as usize >= unsafe { &*baselines }.minor.len() {
            return LayoutUnit::Min();
        }
        unsafe { &*baselines }.minor[set_index as usize]
    }

    // cpp: layoutng/internal/grid_layout_data.h:143-147
    pub fn CreateBaselines(&mut self, direction: GridTrackSizingDirection) {
        *self.baselines_slot(direction) =
            Member::from_ptr(MakeGarbageCollected(GridTrackBaselines::default()));
    }

    // cpp: layoutng/internal/grid_layout_data.h:149-157
    pub fn ResetBaselines(&mut self, direction: GridTrackSizingDirection, set_count: u32) {
        let slot = self.baselines_slot(direction);
        if slot.Get().is_null() {
            *slot = Member::from_ptr(MakeGarbageCollected(GridTrackBaselines::default()));
        }
        unsafe { &mut *slot.Get() }.Reset(set_count);
    }

    // cpp: layoutng/internal/grid_layout_data.h:159-168
    pub fn SetMajorBaseline(
        &mut self,
        direction: GridTrackSizingDirection,
        set_index: u32,
        candidate: LayoutUnit,
    ) {
        let baselines = self.baselines(direction);
        debug_assert!(
            !baselines.is_null() && (set_index as usize) < unsafe { &*baselines }.major.len()
        );
        let current = &mut unsafe { &mut *baselines }.major[set_index as usize];
        if candidate > *current {
            *current = candidate;
        }
    }

    // cpp: layoutng/internal/grid_layout_data.h:170-179
    pub fn SetMinorBaseline(
        &mut self,
        direction: GridTrackSizingDirection,
        set_index: u32,
        candidate: LayoutUnit,
    ) {
        let baselines = self.baselines(direction);
        debug_assert!(
            !baselines.is_null() && (set_index as usize) < unsafe { &*baselines }.minor.len()
        );
        let current = &mut unsafe { &mut *baselines }.minor[set_index as usize];
        if candidate > *current {
            *current = candidate;
        }
    }

    // cpp: layoutng/internal/grid_layout_data.h:181-188
    pub fn SetBaselines(
        &mut self,
        direction: GridTrackSizingDirection,
        baselines: *mut GridTrackBaselines,
    ) {
        *self.baselines_slot(direction) = Member::from_ptr(baselines);
    }

    // cpp: layoutng/internal/grid_layout_data.h:190-194
    pub fn GetBaselines(&self, direction: GridTrackSizingDirection) -> *const GridTrackBaselines {
        self.baselines(direction)
    }

    // cpp: layoutng/internal/grid_layout_data.h:196-201
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.columns_);
        visitor.Trace(&self.rows_);
        visitor.Trace(&self.column_baselines_);
        visitor.Trace(&self.row_baselines_);
    }

    // cpp: layoutng/internal/grid_layout_data.h:203-208
    pub fn IntrinsicRepeatTrackSizes(
        &self,
    ) -> Option<&HashMap<GridTrackSize, LayoutUnit, GridTrackSizeHashTraits>> {
        self.intrinsic_repeat_track_sizes_.as_ref()
    }

    // cpp: layoutng/internal/grid_layout_data.h:210-224
    pub fn AppendIntrinsicRepeatTrackSize(&mut self, track_size: &GridTrackSize, size: LayoutUnit) {
        let sizes = self
            .intrinsic_repeat_track_sizes_
            .get_or_insert_with(HashMap::default);
        if let Some(existing) = sizes.get_mut(track_size) {
            if size > *existing {
                *existing = size;
            }
        } else {
            sizes.insert(track_size.clone(), size);
        }
    }
}

// cpp: layoutng/internal/grid_layout_data.h:265-274
pub struct GridTreeNode {
    pub has_unresolved_geometry: bool,
    pub layout_data: Member<GridLayoutData>,
    pub subtree_size: u32,
}

impl GridTreeNode {
    pub fn new(layout_data: *mut GridLayoutData, subtree_size: u32) -> Self {
        debug_assert!(!layout_data.is_null());
        Self {
            has_unresolved_geometry: unsafe { &*layout_data }.HasIndefiniteSet(),
            layout_data: Member::from_ptr(layout_data),
            subtree_size,
        }
    }

    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.layout_data);
    }
}

// cpp: layoutng/internal/grid_layout_data.h:262-307
pub struct GridLayoutTree {
    tree_data_: HeapVector<Member<GridTreeNode>>,
}

impl GridLayoutTree {
    // cpp: layoutng/internal/grid_layout_data.h:276-277
    pub fn new(tree_data: HeapVector<Member<GridTreeNode>>) -> Self {
        Self {
            tree_data_: tree_data,
        }
    }

    // cpp: layoutng/internal/grid_layout_data.h:279-295
    pub fn AreSubtreesEqual(
        &self,
        subtree_root: u32,
        other: &Self,
        other_subtree_root: u32,
    ) -> bool {
        let subtree_size = self.SubtreeSize(subtree_root);
        if subtree_size != other.SubtreeSize(other_subtree_root) {
            return false;
        }
        for index in 0..subtree_size {
            let left = self.LayoutData(subtree_root + index);
            let right = other.LayoutData(other_subtree_root + index);
            debug_assert!(!left.is_null() && !right.is_null());
            if unsafe { &*right } != unsafe { &*left } {
                return false;
            }
        }
        true
    }

    // cpp: layoutng/internal/grid_layout_data.h:297-310
    pub fn HasUnresolvedGeometry(&self, index: u32) -> bool {
        debug_assert!((index as usize) < self.tree_data_.len());
        unsafe { &*self.tree_data_[index as usize].Get() }.has_unresolved_geometry
    }

    pub fn LayoutData(&self, index: u32) -> *mut GridLayoutData {
        debug_assert!((index as usize) < self.tree_data_.len());
        unsafe { &*self.tree_data_[index as usize].Get() }
            .layout_data
            .Get()
    }

    pub fn Size(&self) -> u32 {
        self.tree_data_.len() as u32
    }

    pub fn SubtreeSize(&self, index: u32) -> u32 {
        debug_assert!((index as usize) < self.tree_data_.len());
        unsafe { &*self.tree_data_[index as usize].Get() }.subtree_size
    }

    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.tree_data_);
    }
}

impl GridSubtreeTree for GridLayoutTree {
    fn SubtreeSize(&self, index: u32) -> u32 {
        GridLayoutTree::SubtreeSize(self, index)
    }
}

// cpp: layoutng/internal/grid_layout_data.h:311-362
pub struct GridLayoutSubtree {
    pub base: GridSubtree<GridLayoutTree>,
    layout_tree_: Member<GridLayoutTree>,
}

impl Default for GridLayoutSubtree {
    // cpp: layoutng/internal/grid_layout_data.h:314-314
    fn default() -> Self {
        Self {
            base: GridSubtree::default(),
            layout_tree_: Member::default(),
        }
    }
}

impl GridLayoutSubtree {
    // cpp: layoutng/internal/grid_layout_data.h:316-320
    pub fn new(layout_tree: *const GridLayoutTree, subtree_root: u32) -> Self {
        debug_assert!(!layout_tree.is_null());
        let mut subtree = Self {
            base: GridSubtree::default(),
            layout_tree_: Member::from_ptr(layout_tree.cast_mut()),
        };
        subtree
            .base
            .SetSubtreeRoot(unsafe { &*layout_tree }, subtree_root);
        subtree
    }

    // cpp: layoutng/internal/grid_layout_data.h:322-323
    pub fn from_subtree(
        layout_tree: *const GridLayoutTree,
        base: GridSubtree<GridLayoutTree>,
    ) -> Self {
        Self {
            base,
            layout_tree_: Member::from_ptr(layout_tree.cast_mut()),
        }
    }

    // cpp: layoutng/internal/grid_layout_data.h:325-328
    pub fn FirstChild(&self) -> *mut Self {
        let tree = self.LayoutTree();
        MakeGarbageCollected(Self::from_subtree(
            self.layout_tree_.Get(),
            self.base.FirstChild(tree),
        ))
    }

    // cpp: layoutng/internal/grid_layout_data.h:330-333
    pub fn NextSibling(&self) -> *mut Self {
        let tree = self.LayoutTree();
        MakeGarbageCollected(Self::from_subtree(
            self.layout_tree_.Get(),
            self.base.NextSibling(tree),
        ))
    }

    // cpp: layoutng/internal/grid_layout_data.h:344-346
    pub fn HasUnresolvedGeometry(&self) -> bool {
        self.LayoutTree()
            .HasUnresolvedGeometry(self.base.subtree_root_)
    }

    // cpp: layoutng/internal/grid_layout_data.h:348-350
    pub fn LayoutData(&self) -> *mut GridLayoutData {
        self.LayoutTree().LayoutData(self.base.subtree_root_)
    }

    // cpp: layoutng/internal/grid_layout_data.h:352
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.layout_tree_);
    }

    // cpp: layoutng/internal/grid_layout_data.h:355-358
    fn LayoutTree(&self) -> &GridLayoutTree {
        let tree = self.layout_tree_.Get();
        debug_assert!(!tree.is_null());
        unsafe { &*tree }
    }
}

// cpp: layoutng/internal/grid_layout_data.h:336-344
impl PartialEq for GridLayoutSubtree {
    fn eq(&self, other: &Self) -> bool {
        let left = self.layout_tree_.Get();
        let right = other.layout_tree_.Get();
        if left.is_null() || right.is_null() {
            left.is_null() && right.is_null()
        } else {
            unsafe { &*left }.AreSubtreesEqual(
                self.base.subtree_root_,
                unsafe { &*right },
                other.base.subtree_root_,
            )
        }
    }
}
