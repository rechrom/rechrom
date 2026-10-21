use crate::grid_line_resolver::GridLineResolver;
use foundation::Vector;
use layoutng_style::style::grid_area::GridArea;
use layoutng_style::style::grid_enums::GridTrackSizingDirection::{self, kForColumns};

// cpp: layoutng_grid/grid_data.h:18-60
// Owned vectors/resolver are moved into this record, preserving move ownership.
pub struct GridPlacementData {
    pub line_resolver: GridLineResolver,
    pub grid_item_positions: Vector<GridArea>,
    pub column_start_offset: u32,
    pub row_start_offset: u32,
}
impl GridPlacementData {
    // cpp: layoutng_grid/grid_data.h:25-26
    pub fn new(line_resolver: &GridLineResolver) -> Self {
        Self {
            line_resolver: line_resolver.clone(),
            grid_item_positions: Vec::new(),
            column_start_offset: 0,
            row_start_offset: 0,
        }
    }
    // cpp: layoutng_grid/grid_data.h:35-38
    pub fn AutoRepeatTrackCount(&self, direction: GridTrackSizingDirection) -> u32 {
        self.line_resolver.AutoRepeatTrackCount(direction)
    }
    // cpp: layoutng_grid/grid_data.h:40-43
    pub fn ExplicitGridTrackCount(&self, direction: GridTrackSizingDirection) -> u32 {
        self.line_resolver.ExplicitGridTrackCount(direction)
    }
    // cpp: layoutng_grid/grid_data.h:45-47
    pub fn HasStandaloneAxis(&self, direction: GridTrackSizingDirection) -> bool {
        self.line_resolver.HasStandaloneAxis(direction)
    }
    // cpp: layoutng_grid/grid_data.h:49-52
    pub fn StartOffset(&self, direction: GridTrackSizingDirection) -> u32 {
        if direction == kForColumns {
            self.column_start_offset
        } else {
            self.row_start_offset
        }
    }
    // cpp: layoutng_grid/grid_data.h:54-56
    pub fn SubgridSpanSize(&self, direction: GridTrackSizingDirection) -> u32 {
        self.line_resolver.SubgridSpanSize(direction)
    }
}
// cpp: layoutng_grid/grid_data.h:28-33
impl PartialEq for GridPlacementData {
    fn eq(&self, other: &Self) -> bool {
        self.grid_item_positions == other.grid_item_positions
            && self.column_start_offset == other.column_start_offset
            && self.row_start_offset == other.row_start_offset
            && self.line_resolver == other.line_resolver
    }
}
