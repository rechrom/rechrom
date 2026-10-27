//! Compatibility facade for the replaced-element algorithm interface.

#![allow(non_snake_case)]

#[path = "../../../layout_algo_replaced/src/native_layout_replaced.rs"]
mod layout_algo_replaced_native_layout_replaced;

pub use layout_algo_replaced_native_layout_replaced::*;
