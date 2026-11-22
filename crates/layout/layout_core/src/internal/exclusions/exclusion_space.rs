//! Compatibility facade for the float algorithm's source exclusion space.

#![allow(non_snake_case)]

#[path = "../../../../layout_fragment/src/float/exclusion_space.rs"]
mod layout_algo_float_exclusion_space;

pub use layout_algo_float_exclusion_space::*;
