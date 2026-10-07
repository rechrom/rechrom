#![allow(non_snake_case)]

use foundation::{LayoutUnit, TextDirection, Vector, WritingDirectionMode, WritingMode};
use layoutng_geometry::geometry::box_strut::BoxStrut;
use layoutng_geometry::geometry::logical_size::LogicalSize;

use super::table_column_location::TableColumnLocation;

// cpp: layoutng/internal/table_constraint_space_data.h:26-37
#[derive(Clone, Copy)]
pub struct Section {
    pub start_row_index: u32,
    pub row_count: u32,
}

#[allow(non_snake_case)]
impl Section {
    // cpp: layoutng/internal/table_constraint_space_data.h:27-28
    pub fn new(start_row_index: u32, row_count: u32) -> Self {
        Self {
            start_row_index,
            row_count,
        }
    }

    // cpp: layoutng/internal/table_constraint_space_data.h:30-33
    pub fn MaySkipLayout(&self, other: &Self) -> bool {
        self.row_count == other.row_count
    }
}

// cpp: layoutng/internal/table_constraint_space_data.h:40-63
#[derive(Clone, Copy)]
pub struct Row {
    pub block_size: LayoutUnit,
    pub start_cell_index: u32,
    pub cell_count: u32,
    pub baseline: Option<LayoutUnit>,
    pub is_collapsed: bool,
}

#[allow(non_snake_case)]
impl Row {
    // cpp: layoutng/internal/table_constraint_space_data.h:41-50
    pub fn new(
        block_size: LayoutUnit,
        start_cell_index: u32,
        cell_count: u32,
        baseline: Option<LayoutUnit>,
        is_collapsed: bool,
    ) -> Self {
        Self {
            block_size,
            start_cell_index,
            cell_count,
            baseline,
            is_collapsed,
        }
    }

    // cpp: layoutng/internal/table_constraint_space_data.h:52-56
    pub fn MaySkipLayout(&self, other: &Self) -> bool {
        self.block_size == other.block_size
            && self.cell_count == other.cell_count
            && self.baseline == other.baseline
            && self.is_collapsed == other.is_collapsed
    }
}

// cpp: layoutng/internal/table_constraint_space_data.h:66-96
#[derive(Clone, Copy, PartialEq)]
pub struct Cell {
    pub borders: BoxStrut,
    pub rowspan_block_size: LayoutUnit,
    pub start_column: u32,
    pub is_initial_block_size_indefinite: bool,
    pub has_descendant_that_depends_on_percentage_block_size: bool,
}

#[allow(non_snake_case)]
impl Cell {
    // cpp: layoutng/internal/table_constraint_space_data.h:67-77
    pub fn new(
        borders: BoxStrut,
        rowspan_block_size: LayoutUnit,
        start_column: u32,
        is_initial_block_size_indefinite: bool,
        has_descendant_that_depends_on_percentage_block_size: bool,
    ) -> Self {
        Self {
            borders,
            rowspan_block_size,
            start_column,
            is_initial_block_size_indefinite,
            has_descendant_that_depends_on_percentage_block_size,
        }
    }

    // cpp: layoutng/internal/table_constraint_space_data.h:79-87
    pub fn Equals(&self, other: &Self) -> bool {
        self.borders == other.borders
            && self.rowspan_block_size == other.rowspan_block_size
            && self.start_column == other.start_column
            && self.is_initial_block_size_indefinite == other.is_initial_block_size_indefinite
            && self.has_descendant_that_depends_on_percentage_block_size
                == other.has_descendant_that_depends_on_percentage_block_size
    }
}

// C++ RefCounted ownership is exposed to callers through their scoped
// refcounted handle; this value itself contains no reference-count operations.
// cpp: layoutng/internal/table_constraint_space_data.h:22-23
// cpp: layoutng/internal/table_constraint_space_data.h:177-188
pub struct TableConstraintSpaceData {
    pub column_locations: Vector<TableColumnLocation>,
    pub sections: Vector<Section>,
    pub rows: Vector<Row>,
    pub cells: Vector<Cell>,
    pub table_writing_direction: WritingDirectionMode,
    pub table_border_spacing: LogicalSize,
    pub is_table_block_size_specified: bool,
    pub has_collapsed_borders: bool,
}

impl Default for TableConstraintSpaceData {
    // cpp: layoutng/internal/table_constraint_space_data.h:181-187
    fn default() -> Self {
        Self {
            column_locations: Vector::default(),
            sections: Vector::default(),
            rows: Vector::default(),
            cells: Vector::default(),
            table_writing_direction: WritingDirectionMode::new(
                WritingMode::kHorizontalTb,
                TextDirection::kLtr,
            ),
            table_border_spacing: LogicalSize::default(),
            is_table_block_size_specified: false,
            has_collapsed_borders: false,
        }
    }
}

#[allow(non_snake_case)]
impl TableConstraintSpaceData {
    // cpp: layoutng/internal/table_constraint_space_data.h:98-105
    pub fn IsTableSpecificDataEqual(&self, other: &Self) -> bool {
        self.column_locations == other.column_locations
            && self.table_writing_direction == other.table_writing_direction
            && self.table_border_spacing == other.table_border_spacing
            && self.is_table_block_size_specified == other.is_table_block_size_specified
            && self.has_collapsed_borders == other.has_collapsed_borders
    }

    // cpp: layoutng/internal/table_constraint_space_data.h:107-138
    pub fn MaySkipRowLayout(&self, other: &Self, new_row_index: u32, old_row_index: u32) -> bool {
        debug_assert!((new_row_index as usize) < self.rows.len());
        debug_assert!((old_row_index as usize) < other.rows.len());

        let new_row = &self.rows[new_row_index as usize];
        let old_row = &other.rows[old_row_index as usize];
        if !new_row.MaySkipLayout(old_row) {
            return false;
        }

        debug_assert_eq!(new_row.cell_count, old_row.cell_count);

        let new_start_cell_index = new_row.start_cell_index;
        let old_start_cell_index = old_row.start_cell_index;
        let new_end_cell_index = new_start_cell_index.wrapping_add(new_row.cell_count);
        let old_end_cell_index = old_start_cell_index.wrapping_add(old_row.cell_count);
        let mut new_cell_index = new_start_cell_index;
        let mut old_cell_index = old_start_cell_index;
        while new_cell_index < new_end_cell_index && old_cell_index < old_end_cell_index {
            if self.cells[new_cell_index as usize] != other.cells[old_cell_index as usize] {
                return false;
            }
            new_cell_index = new_cell_index.wrapping_add(1);
            old_cell_index = old_cell_index.wrapping_add(1);
        }
        true
    }

    // cpp: layoutng/internal/table_constraint_space_data.h:140-175
    pub fn MaySkipSectionLayout(
        &self,
        other: &Self,
        new_section_index: u32,
        old_section_index: u32,
    ) -> bool {
        debug_assert!((new_section_index as usize) <= self.sections.len());
        debug_assert!((old_section_index as usize) <= other.sections.len());

        let new_section = &self.sections[new_section_index as usize];
        let old_section = &other.sections[old_section_index as usize];
        if !new_section.MaySkipLayout(old_section) {
            return false;
        }

        debug_assert_eq!(new_section.row_count, old_section.row_count);

        let new_start_row_index = new_section.start_row_index;
        let old_start_row_index = old_section.start_row_index;
        debug_assert_eq!(self.has_collapsed_borders, other.has_collapsed_borders);
        if self.has_collapsed_borders && new_start_row_index != old_start_row_index {
            return false;
        }

        let new_end_row_index = new_start_row_index.wrapping_add(new_section.row_count);
        let old_end_row_index = old_start_row_index.wrapping_add(old_section.row_count);
        let mut new_row_index = new_start_row_index;
        let mut old_row_index = old_start_row_index;
        while new_row_index < new_end_row_index && old_row_index < old_end_row_index {
            if !self.MaySkipRowLayout(other, new_row_index, old_row_index) {
                return false;
            }
            new_row_index = new_row_index.wrapping_add(1);
            old_row_index = old_row_index.wrapping_add(1);
        }
        true
    }
}
