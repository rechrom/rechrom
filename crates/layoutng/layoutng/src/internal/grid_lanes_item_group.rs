#![allow(non_snake_case)]

use std::hash::{Hash, Hasher};

use foundation::{HashInts, HeapHashMap, HeapVector, MakeGarbageCollected, Member, Visitor};
use layoutng_style::style::grid_area::GridSpan;

use super::baseline_utils::BaselineGroup;
use super::grid_item::{GridItemDataVector, GridItemDataVirtualItemContributions, GridItems};

// C++ DISALLOW_NEW is a stack-allocation hint. Rust owns this key by value.
// cpp: layoutng/internal/grid_lanes_item_group.h:22-81
#[derive(Clone, Copy, Debug)]
pub struct GridLanesItemGroupProperties {
    is_deleted_: bool,
    item_span_: Option<GridSpan>,
    baseline_group_: Option<BaselineGroup>,
}

impl Default for GridLanesItemGroupProperties {
    // cpp: layoutng/internal/grid_lanes_item_group.h:26-26
    fn default() -> Self {
        Self {
            is_deleted_: false,
            item_span_: None,
            baseline_group_: None,
        }
    }
}

impl GridLanesItemGroupProperties {
    // HashTableDeletedValueType is a C++ constructor tag. Rust names the
    // deleted-value constructor directly and keeps its distinct hash.
    // cpp: layoutng/internal/grid_lanes_item_group.h:27-28
    pub fn new_deleted() -> Self {
        Self {
            is_deleted_: true,
            ..Self::default()
        }
    }

    // cpp: layoutng/internal/grid_lanes_item_group.h:30-33
    pub fn new(item_span: &GridSpan, baseline_group: Option<BaselineGroup>) -> Self {
        Self {
            is_deleted_: false,
            item_span_: Some(*item_span),
            baseline_group_: baseline_group,
        }
    }

    pub fn new_without_baseline_group(item_span: &GridSpan) -> Self {
        Self::new(item_span, None)
    }

    // cpp: layoutng/internal/grid_lanes_item_group.h:40-54
    pub fn GetHash(&self) -> u32 {
        let Some(item_span) = self.item_span_.as_ref() else {
            return if self.is_deleted_ { u32::MAX } else { 0 };
        };
        let mut hash = item_span.GetHash();
        if let Some(baseline_group) = self.baseline_group_ {
            hash = HashInts(hash, baseline_group as u32);
        }
        hash
    }

    // cpp: layoutng/internal/grid_lanes_item_group.h:56-56
    pub fn IsHashTableDeletedValue(&self) -> bool {
        self.is_deleted_
    }

    // cpp: layoutng/internal/grid_lanes_item_group.h:58-61
    pub fn Span(&self) -> &GridSpan {
        debug_assert!(self.item_span_.is_some());
        self.item_span_
            .as_ref()
            .expect("grid-lanes group has no span")
    }

    // cpp: layoutng/internal/grid_lanes_item_group.h:63-65
    pub fn GetBaselineGroup(&self) -> &Option<BaselineGroup> {
        &self.baseline_group_
    }
}

// cpp: layoutng/internal/grid_lanes_item_group.h:35-38
impl PartialEq for GridLanesItemGroupProperties {
    fn eq(&self, other: &Self) -> bool {
        self.is_deleted_ == other.is_deleted_
            && self.item_span_ == other.item_span_
            && self.baseline_group_ == other.baseline_group_
    }
}

impl Eq for GridLanesItemGroupProperties {}

// C++ HashTraits uses GetHash for the table key.
// cpp: layoutng/internal/grid_lanes_item_group.h:123-125
impl Hash for GridLanesItemGroupProperties {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.GetHash().hash(state);
    }
}

// cpp: layoutng/internal/grid_lanes_item_group.h:83-103
pub struct GridLanesItemGroup {
    pub items: GridItemDataVector,
    pub properties: GridLanesItemGroupProperties,
    pub contribution_sizes: Member<GridItemDataVirtualItemContributions>,
}

impl GridLanesItemGroup {
    // cpp: layoutng/internal/grid_lanes_item_group.h:85-90
    pub fn new(items: GridItemDataVector, properties: GridLanesItemGroupProperties) -> Self {
        Self {
            items: items,
            properties: properties,
            contribution_sizes: Member::from_ptr(MakeGarbageCollected(
                GridItemDataVirtualItemContributions::default(),
            )),
        }
    }

    // cpp: layoutng/internal/grid_lanes_item_group.h:92-95
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.items);
        visitor.Trace(&self.contribution_sizes);
    }
}

// HeapVector's inline capacity of 16 is a C++ storage hint; its Rust
// collection keeps the traced member elements and sequence semantics.
// cpp: layoutng/internal/grid_lanes_item_group.h:105-107
pub type GridLanesItemGroupMap = HeapHashMap<GridLanesItemGroupProperties, GridItemDataVector>;
pub type GridLanesItemGroups = HeapVector<Member<GridLanesItemGroup>>;

// cpp: layoutng/internal/grid_lanes_item_group.h:111-121
pub struct VirtualItems {
    pub items: Member<GridItems>,
    pub item_groups: GridLanesItemGroups,
}

impl Default for VirtualItems {
    // cpp: layoutng/internal/grid_lanes_item_group.h:112-112
    fn default() -> Self {
        Self {
            items: Member::from_ptr(MakeGarbageCollected(GridItems::default())),
            item_groups: GridLanesItemGroups::default(),
        }
    }
}

impl VirtualItems {
    // cpp: layoutng/internal/grid_lanes_item_group.h:114-117
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.items);
        visitor.Trace(&self.item_groups);
    }
}
