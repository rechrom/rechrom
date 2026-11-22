//! Compatibility facade for the table-algorithm node interface.

#![allow(non_snake_case)]

#[path = "../../../layout_algo_table/src/table_node_interface.rs"]
mod layout_algo_table_table_node_interface;

pub use layout_algo_table_table_node_interface::*;
