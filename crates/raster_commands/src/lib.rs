//! Backend-neutral immutable raster command and resource vocabulary.
//!
//! This package contains no Skia implementation and no compositor scheduling.
//! Legacy and translated raster backends consume the same values.

pub mod commands;
pub mod filter_graph;
