//! Compatibility facade for out-of-flow anchor-map services.

#![allow(non_snake_case)]

#[path = "out_of_flow/anchor_map_services.rs"]
mod layout_algo_out_of_flow_anchor_map_services;

pub use layout_algo_out_of_flow_anchor_map_services::*;
