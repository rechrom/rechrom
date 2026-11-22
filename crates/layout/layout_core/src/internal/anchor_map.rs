//! Compatibility facade for out-of-flow-owned anchor map state and policy.

#![allow(non_snake_case)]

#[path = "../../../layout_algo_out_of_flow/src/anchor_map.rs"]
mod layout_algo_out_of_flow_anchor_map;

pub use layout_algo_out_of_flow_anchor_map::*;
