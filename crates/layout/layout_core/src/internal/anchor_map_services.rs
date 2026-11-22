//! Compatibility facade for out-of-flow anchor-map services.

#![allow(non_snake_case)]

#[path = "../../../layout_algo_out_of_flow/src/anchor_map_services.rs"]
mod layout_algo_out_of_flow_anchor_map_services;

pub use layout_algo_out_of_flow_anchor_map_services::*;
