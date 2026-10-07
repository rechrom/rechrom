//! Source table package. Object factories and algorithms are installed together.
pub mod layout_table_column_visitor;
pub mod table_borders;
pub mod table_break_token_data;
pub mod table_child_iterator;
pub mod table_constants;
pub mod table_layout_algorithm_types;
pub mod table_layout_utils;
pub mod table_node;
pub mod table_row_break_token_data;
pub mod table_row_layout_algorithm;
pub mod table_section_layout_algorithm;
pub mod table_size_distribution;

pub mod layout_table;
pub mod layout_table_caption;
pub mod layout_table_cell;
pub mod layout_table_column;
pub mod layout_table_row;
pub mod layout_table_section;
#[cfg(test)]
mod native_test_thread;
#[cfg(test)]
mod table_distribution_tests;

pub mod assembly;
mod table_object_virtuals;
pub mod table_tree_insertion;

#[cfg(test)]
mod table_native_object_tests;

pub mod table_row_measurement;

pub mod table_layout_support;

mod table_fragment_generation;
pub mod table_layout_algorithm;
