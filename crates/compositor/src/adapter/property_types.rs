//! Facade compatibility boundary for property-tree values.
//!
//! Tree values and tracing live in `compositor_tree` and the shared commit
//! vocabulary. The facade combines the tree trace vocabulary with tile-only
//! trace fields for legacy call sites that still serialize both kinds of state
//! into one object.

pub use compositor_tile::tile_priority::TilePriorityTrace;
pub use layerize::commit::property_types::*;
