//! Compatibility facade for the table-algorithm node interface.

#![allow(non_snake_case)]

#[path = "../../../layout_algo_table/src/native_table_node.rs"]
mod layout_algo_table_native_table_node;

pub use layout_algo_table_native_table_node::*;
