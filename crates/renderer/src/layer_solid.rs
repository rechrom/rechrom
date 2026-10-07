//! SolidColorDrawQuad-equivalent execution for exactly analyzed CPU tiles.
//!
//! cc/tiles/tile_draw_info.h distinguishes RESOURCE_MODE/SOLID_COLOR_MODE.
//! The mode comes from the original PaintOp solid-color analysis before raster.
//! As in cc::TileDrawInfo::SOLID_COLOR_MODE, a proven solid owns no pixel
//! resource. Tile identity and damage retain their existing ownership.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TileDrawMode {
    Resource,
    /// Actual uniform opaque premultiplied color, in little-endian RGBA order.
    SolidColor {
        premul_rgba: u32,
    },
}

/// TileManager::AssignGpuMemoryToTiles admits analyzed solids without raster
/// work (cc/tiles/tile_manager.cc:949-976). Preserve only the exact analyzed
/// color; generic edge sampling reads that value when unlike neighbors meet.
pub(crate) fn raster_product(
    plan_index: usize,
    task: &layer_tile::RasterTask,
    composition: LayerComposition,
    premul_rgba: u32,
    mut rgba: Vec<u8>,
    profile: bool,
) -> io::Result<TileProduct> {
    debug_assert_eq!(
        premul_rgba >> 24,
        255,
        "PaintOp analysis proved an opaque color"
    );
    let started = profile.then(std::time::Instant::now);
    tile_bytes(task.pixel_size)?;
    rgba.clear();
    let call_time = started.map_or(std::time::Duration::ZERO, |start| start.elapsed());
    let started = profile.then(std::time::Instant::now);
    let (width, height) = (task.pixel_size.0 as usize, task.pixel_size.1 as usize);
    let mut rows = Vec::with_capacity(height);
    rows.resize_with(height, || RowSupport {
        extent: (0, width),
        opaque: true,
        runs: (0, 0),
    });
    let row_support = TileRowSupport {
        rows,
        runs: Vec::new(),
        opaque: true,
        draw_mode: TileDrawMode::SolidColor { premul_rgba },
    };
    let bgra = Vec::new();
    Ok(TileProduct {
        plan_index,
        task: task.clone(),
        composition,
        rgba,
        bgra,
        row_support,
        call_time,
        support_time: started.map_or(std::time::Duration::ZERO, |start| start.elapsed()),
    })
}

fn neighbors_have_color(
    color: u32,
    phase: (f64, f64),
    neighbors: [Option<TileDrawMode>; 3],
) -> bool {
    if ![phase.0, phase.1]
        .iter()
        .all(|value| value.is_finite() && (0.0..=1.0).contains(value))
    {
        return false;
    }
    let required = [
        phase.0 != 0.0,
        phase.1 != 0.0,
        phase.0 != 0.0 && phase.1 != 0.0,
    ];
    required.into_iter().zip(neighbors).all(|(used, neighbor)| {
        !used || neighbor.is_none_or(|mode| mode == TileDrawMode::SolidColor { premul_rgba: color })
    })
}

pub(super) fn color_for_sampling(
    pixels: &CachedTile,
    adjacent: [Option<&CachedTile>; 3],
    phase: (f64, f64),
) -> Option<u32> {
    let TileDrawMode::SolidColor { premul_rgba } = pixels.row_support.draw_mode else {
        return None;
    };
    (premul_rgba >> 24 == 255
        && neighbors_have_color(
            premul_rgba,
            phase,
            adjacent.map(|next| next.map(|tile| tile.row_support.draw_mode)),
        ))
    .then_some(premul_rgba)
}

/// Draw the proven solid quad using ordinary opaque rectangular fill. The
/// caller supplies the already clipped destination bounds and original phase.
/// Integer placement needs no neighbors. Fractional placement uses this path
/// only if every participating right/down/diagonal source is the same proven
/// color. Missing neighbors preserve tile_edge_pixel's existing edge clamp.
/// Any different or unanalyzed source retains the ordinary real sampler.
/// Direct DstIn quads must not call this ordinary SrcOver entry point. A mask
/// drawn into an isolated source surface is still an ordinary source quad;
/// its parent's DstIn is applied separately at the real effect boundary.
pub(super) fn try_compose(
    target: &mut [u32],
    view: TargetView,
    bounds: SurfaceBounds,
    format: skia::PixelFormat,
    pixels: &CachedTile,
    adjacent: [Option<&CachedTile>; 3],
    phase: (f64, f64),
) -> bool {
    let Some(premul_rgba) = color_for_sampling(pixels, adjacent, phase) else {
        return false;
    };
    if bounds.left >= bounds.right || bounds.top >= bounds.bottom {
        return true;
    }
    // Validate the complete target extent before modifying any pixels; false
    // leaves the existing sampling path an untouched destination.
    if bounds.left < view.origin.0
        || bounds.top < view.origin.1
        || bounds.right - view.origin.0 > view.stride
    {
        return false;
    }
    let Some(last_end) = (bounds.bottom - 1 - view.origin.1)
        .checked_mul(view.stride)
        .and_then(|offset| offset.checked_add(bounds.right - view.origin.0))
    else {
        return false;
    };
    if last_end > target.len() {
        return false;
    }

    // SoftwareRenderer::DrawSolidColorQuad draws a color rectangle, rather
    // than a texture. Opaque SrcOver is Src; keep the existing exact channel
    // permutation and final-frame zero-X convention for its normal fill.
    let mut color = premul_rgba;
    if format != skia::PixelFormat::Rgba8888 {
        color = (color & 0xff00_ff00) | ((color & 0xff) << 16) | ((color >> 16) & 0xff);
    }
    if format == skia::PixelFormat::Bgrx8888 {
        color &= 0x00ff_ffff;
    }
    // SkARGB32_Blitter::blitRect uses this exact opaque rect fill primitive.
    let first = view.row_start(bounds.top, bounds.left);
    skia::src::opts::SkMemset_opts::rect_memset32(
        &mut target[first..],
        color,
        bounds.right - bounds.left,
        view.stride,
        bounds.bottom - bounds.top,
    );
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solid_raster_product_keeps_only_exact_color_metadata() {
        use skia::compat::commands::{Color, CommandKind, DrawCommand};
        let rect = PaintRect {
            x: 0.0,
            y: 0.0,
            width: 17.0,
            height: 3.0,
        };
        let task = layer_tile::RasterTask {
            tile_id: TileId(9),
            previous_tile_id: Some(TileId(8)),
            layer_id: layer_tile::LayerId(4),
            generation: 73,
            tile_index: (0, 0),
            tiling_rect: rect,
            tile_rect: rect,
            raster_scale: 1.0,
            pixel_size: (17, 3),
            required_for_activation: false,
            record_indices: vec![0],
        };
        let full = DrawCommand {
            r#type: CommandKind::kDrawRect,
            rect: skia::compat::commands::PaintRect {
                x: 0.0,
                y: 0.0,
                width: 17.0,
                height: 3.0,
            },
            color: Color {
                red: 0.2,
                green: 0.4,
                blue: 0.6,
                alpha: 1.0,
            },
            ..Default::default()
        };
        let mut partial = full.clone();
        partial.rect.width = 5.0;
        partial.color.red = 0.9;
        let commands = [partial, full];
        let TileDrawMode::SolidColor { premul_rgba } = super::super::layer_solid_analysis::analyze(
            &task,
            std::iter::once(Ok(commands.as_slice())),
        ) else {
            panic!("opaque final full rect must pass the actual PaintOp proof");
        };
        let reuse = vec![0x79; 17 * 3 * 4 + 64];
        let composition = LayerComposition {
            translation: (0.0, 0.0),
            clip: None,
            content_bounds: None,
            opaque_bounds: None,
            white_backing: false,
            grid_min: None,
        };
        let product = raster_product(2, &task, composition, premul_rgba, reuse, false).unwrap();
        assert!(product.rgba.is_empty());
        assert!(product.bgra.is_empty());
        assert_eq!(
            product.row_support.draw_mode,
            TileDrawMode::SolidColor { premul_rgba }
        );
        assert_eq!(
            (
                product.plan_index,
                product.task.tile_id,
                product.task.generation
            ),
            (2, task.tile_id, 73)
        );
        assert_eq!(product.task.record_indices, task.record_indices);
        assert!(product.row_support.opaque);
        assert_eq!(product.row_support.len(), 3);
        assert!(product
            .row_support
            .rows
            .iter()
            .all(|row| row.extent == (0, 17) && row.runs == (0, 0)));
    }

    #[test]
    fn participating_neighbors_must_have_the_same_analyzed_color() {
        let color = u32::from_le_bytes([29, 47, 83, 255]);
        let solid = TileDrawMode::SolidColor { premul_rgba: color };
        let different = TileDrawMode::SolidColor {
            premul_rgba: color ^ 1,
        };
        assert!(neighbors_have_color(color, (0.25, 0.5), [Some(solid); 3]));
        assert!(!neighbors_have_color(
            color,
            (0.25, 0.5),
            [Some(solid), Some(solid), Some(different)]
        ));
        assert!(!neighbors_have_color(
            color,
            (0.0, 0.5),
            [None, Some(TileDrawMode::Resource), None]
        ));
        assert!(neighbors_have_color(
            color,
            (0.0, 0.5),
            [Some(different), Some(solid), Some(different)]
        ));
        assert!(neighbors_have_color(color, (0.25, 0.5), [None; 3]));
        assert!(neighbors_have_color(
            color,
            (0.0, 0.0),
            [Some(TileDrawMode::Resource); 3]
        ));
    }
}
