//! Scroll offsets named by cc/paint raster on both threads. Moved from
//! compositor_tile::prioritized_tile_types, which re-exports it.

use compositor_tree::{geometry::PointF, ElementId};
use std::collections::BTreeMap;

/// Source value: cc/paint/scroll_offset_map.h:14 (ordered flat_map).
pub type ScrollOffsetMap = BTreeMap<ElementId, PointF>;
