//! Impl-side pending/active tree synchronization algorithms: commit into the
//! sync tree, activation, property pushes and the impl property-tree updates.
//!
//! Only the impl `compositor` facade runs these (LayerTreeHostImpl,
//! LayerTreeImpl, TreeSynchronizer). The cc property trees they update are
//! the shared `compositor_tree` values. Trace category `compositor.tree_sync`.

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

pub mod activation;
pub mod clip_tree_update;
pub mod commit_properties;
pub mod commit_sync;
pub mod layer_collections;
pub mod property_push;
pub mod property_tree_update;
pub mod render_target_update;
pub mod scrollbar_opacity_sync;
