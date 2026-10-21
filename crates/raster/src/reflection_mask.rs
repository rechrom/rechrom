use std::io;
use std::sync::Arc;

use layoutng_assembly::internal::paint_input::PaintReflectionMaskLayer;
use raster_commands::commands as target;
use raster_commands::filter_graph::RenderReflectionMaskLayer;

use crate::convert::ToSkia;

/// Resolve Layout's selected reflection-mask sources at the Raster boundary.
/// Bitmap/image-set pixels are retained directly. Nested documents require an
/// immutable record precompiled at the compositor Paint boundary; raster never
/// reconstructs property-tree conversion policy.
pub fn CompileReflectionMaskLayers(
    layers: &[PaintReflectionMaskLayer],
) -> io::Result<Vec<RenderReflectionMaskLayer>> {
    layers
        .iter()
        .map(|layer| {
            let (resource_id, width, height, rgba, source_scale_x, source_scale_y) =
                match layer.image.as_ref() {
                    None => (0, 0, 0, Arc::new(Vec::new()), 1.0, 1.0),
                    Some(image) => match &image.content {
                        image_resource::PaintImageContent::Bitmap(pixels) => (
                            image.id,
                            image.width,
                            image.height,
                            Arc::clone(pixels),
                            1.0,
                            1.0,
                        ),
                        image_resource::PaintImageContent::Document(_) => {
                            // Nested document property conversion belongs to the
                            // compositor's PaintChunksToCcLayer input stage. Raster
                            // may only consume its compiled immutable record; the
                            // producer has not supplied that typed payload yet.
                            return Err(io::Error::new(io::ErrorKind::Unsupported,
                                "reflection document mask requires compositor-precompiled native PaintRecord"));
                        }
                    },
                };
            Ok(RenderReflectionMaskLayer {
                resource_id,
                image_width: width,
                image_height: height,
                image_rgba: rgba,
                image_interpolation_quality: layer.image_interpolation_quality.to_skia(),
                paint_shader: layer.paint_shader.as_ref().map(ToSkia::to_skia),
                clip_rect: target::PaintRect {
                    x: layer.clip_origin.x,
                    y: layer.clip_origin.y,
                    width: layer.clip_size.width,
                    height: layer.clip_size.height,
                },
                source_rect: target::PaintRect {
                    x: layer.source_origin.x * source_scale_x,
                    y: layer.source_origin.y * source_scale_y,
                    width: layer.source_size.width * source_scale_x,
                    height: layer.source_size.height * source_scale_y,
                },
                tile_rect: target::PaintRect {
                    x: layer.tile_origin.x,
                    y: layer.tile_origin.y,
                    width: layer.tile_size.width,
                    height: layer.tile_size.height,
                },
                repeat_x: layer.repeat_x,
                repeat_y: layer.repeat_y,
                tile_spacing: target::Size {
                    width: layer.tile_spacing.width,
                    height: layer.tile_spacing.height,
                },
            })
        })
        .collect()
}
