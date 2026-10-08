//! PaintOp solid-colour analysis result used by tile raster and composition.

use crate::{
    engine::{RasterRowSupport, RowSupport},
    layer_replay::RasterLayerState,
    tile_worker::RasterProduct,
};
use layer_tile::RasterTask;
use std::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RasterDrawMode {
    Resource,
    SolidColor { premul_rgba: u32 },
}

pub(crate) fn raster_product(
    plan_index: usize,
    task: &RasterTask,
    composition: RasterLayerState,
    premul_rgba: u32,
    mut rgba: Vec<u8>,
) -> io::Result<RasterProduct> {
    debug_assert_eq!(premul_rgba >> 24, 255);
    super::engine::tile_bytes(task.pixel_size)?;
    rgba.clear();
    let (width, height) = (task.pixel_size.0 as usize, task.pixel_size.1 as usize);
    let mut rows = Vec::with_capacity(height);
    rows.resize_with(height, || RowSupport {
        extent: (0, width),
        opaque: true,
        runs: (0, 0),
    });
    Ok(RasterProduct {
        plan_index,
        task: task.clone(),
        composition,
        rgba,
        bgra: Vec::new(),
        row_support: RasterRowSupport {
            rows,
            runs: Vec::new(),
            opaque: true,
            draw_mode: RasterDrawMode::SolidColor { premul_rgba },
        },
    })
}
