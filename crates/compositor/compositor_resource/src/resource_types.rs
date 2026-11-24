//! Rechrom-owned typed resource boundary. No GPU, tiles, or Viz implementation
//! is owned here; these values preserve the source fields used by cc core.

pub use crate::{MemoryAllocation, PriorityCutoff};

// Source vocabulary: cc/tiles/tile_priority.h:66-80
pub use compositor_tile::TileMemoryLimitPolicy;

// viz::SharedImageFormat values, shared with the main side's decoded images.
pub use compositor_values::shared_image_format::*;

pub use compositor_tile::tile_draw_info::TileDrawResource;
