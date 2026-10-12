use std::io;
use std::sync::Arc;

use layoutng_assembly::internal::paint_input::PaintReflectionMaskLayer;
use skia::compat::commands as target;
use skia::compat::layer_filters::RenderReflectionMaskLayer;

use crate::convert::ToSkia;

/// Resolve Layout's selected reflection-mask sources at the Raster boundary.
/// Bitmap/image-set pixels are retained directly. Document images are replayed
/// for the exact ContainerKey chosen by Paint, so effective zoom and DPR are
/// applied exactly once before the compositor samples the NinePiece record.
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
                        image_resource::PaintImageContent::Document(content) => {
                            let key = layer.document_image_container_key.as_ref().ok_or_else(|| {
                                io::Error::new(
                                    io::ErrorKind::Unsupported,
                                    "reflection-document-container",
                                )
                            })?;
                            let record = content
                                .exact_record_for(key)
                                .and_then(|record| {
                                    record.as_any().downcast_ref::<
                                        paint::paint_engine::DocumentPaintArtifactRecord,
                                    >()
                                })
                                .ok_or_else(|| {
                                    io::Error::new(
                                        io::ErrorKind::Unsupported,
                                        "reflection-document-record",
                                    )
                                })?;
                            let (width, height, pixels) =
                                crate::layer_replay::rasterize_pattern_document_record(record, 0)
                                    .ok_or_else(|| {
                                        io::Error::new(
                                            io::ErrorKind::Unsupported,
                                            "reflection-document-raster",
                                        )
                                    })?;
                            (
                                image.id,
                                width,
                                height,
                                Arc::new(pixels),
                                f64::from(width) / f64::from(image.width.max(1)),
                                f64::from(height) / f64::from(image.height.max(1)),
                            )
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
