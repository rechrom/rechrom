//! Base facade for outline-rect collection (LayoutObject behaviour).

#![allow(non_snake_case, non_camel_case_types)]

#[path = "fragment/outline_rect_collector.rs"]
mod layout_core_outline_rect_collector;

pub use layout_core_outline_rect_collector::*;
