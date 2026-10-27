//! Compatibility facade for out-of-flow-owned anchor map state and policy.

#![allow(non_snake_case)]

#[path = "../../../layout_algo_out_of_flow/src/native_anchor_map.rs"]
mod layout_algo_out_of_flow_native_anchor_map;

pub use layout_algo_out_of_flow_native_anchor_map::*;
