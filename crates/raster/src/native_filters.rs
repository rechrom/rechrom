//! Borrowed native PaintFilter graph -> Skia CPU filter values. This boundary
//! executes immutable Paint values; it never selects layers or allocates tiles.
#![allow(non_snake_case)]
use paint::cc::paint::{
    filter_operation::{BlurTileMode, FilterType, PaintFilter, PaintFilterReplay},
    filter_operations::FilterOperations,
};
use skia::compat::layer_filters::{css_color_matrix, ColorMatrixKind, RenderSurfaceFilter};
use std::io;

pub fn BuildFilters(
    ops: &FilterOperations,
    matrix: skia::raster::Transform,
) -> io::Result<Vec<RenderSurfaceFilter>> {
    let mut result = Vec::with_capacity(ops.size());
    for op in ops.operations() {
        use FilterType::*;
        let color = match op.r#type() {
            GRAYSCALE => Some(ColorMatrixKind::Grayscale),
            SEPIA => Some(ColorMatrixKind::Sepia),
            SATURATE => Some(ColorMatrixKind::Saturate),
            HUE_ROTATE => Some(ColorMatrixKind::HueRotate),
            INVERT => Some(ColorMatrixKind::Invert),
            OPACITY => Some(ColorMatrixKind::Opacity),
            BRIGHTNESS => Some(ColorMatrixKind::Brightness),
            CONTRAST => Some(ColorMatrixKind::Contrast),
            _ => None,
        };
        if let Some(kind) = color {
            result.push(RenderSurfaceFilter::ColorMatrix(css_color_matrix(
                kind,
                op.amount(),
            )));
            continue;
        }
        match op.r#type() {
            BLUR => {
                if op.blur_tile_mode() != BlurTileMode::kDecal {
                    return Err(io::Error::new(io::ErrorKind::Unsupported,"native blur non-decal image-filter tile mode is not exposed by Skia CPU filter execution"));
                }
                result.push(RenderSurfaceFilter::Blur {
                    sigma_x: op.amount() * matrix.sx.hypot(matrix.kx),
                    sigma_y: op.amount() * matrix.ky.hypot(matrix.sy),
                });
            }
            DROP_SHADOW => {
                let offset = op.offset();
                let color = op.drop_shadow_color();
                result.push(RenderSurfaceFilter::DropShadow {
                    sigma_x: op.amount() * matrix.sx.hypot(matrix.kx),
                    sigma_y: op.amount() * matrix.ky.hypot(matrix.sy),
                    offset_x: matrix.sx * offset.x() as f32 + matrix.kx * offset.y() as f32,
                    offset_y: matrix.ky * offset.x() as f32 + matrix.sy * offset.y() as f32,
                    color: [color.fR, color.fG, color.fB, color.fA],
                });
            }
            COLOR_MATRIX => result.push(RenderSurfaceFilter::ColorMatrix(*op.matrix())),
            // Source: cc/paint/render_surface_filters.cc:36-42,241-244.
            SATURATING_BRIGHTNESS => {
                let mut matrix = [0.; 20];
                matrix[0] = 1.;
                matrix[6] = 1.;
                matrix[12] = 1.;
                matrix[18] = 1.;
                matrix[4] = op.amount();
                matrix[9] = op.amount();
                matrix[14] = op.amount();
                result.push(RenderSurfaceFilter::ColorMatrix(matrix));
            }
            REFERENCE => {
                if let Some(filter) = op.image_filter() {
                    result.extend(BuildGraph(filter.as_ref(), matrix)?);
                }
            }
            OFFSET => {
                let offset = op.offset();
                result.push(RenderSurfaceFilter::Offset {
                    x: matrix.sx * offset.x() as f32 + matrix.kx * offset.y() as f32,
                    y: matrix.ky * offset.x() as f32 + matrix.sy * offset.y() as f32,
                });
            }
            ALPHA_THRESHOLD => {
                use skia::include::core::SkRegion::{Op, SkRegion, SkRegionIRect};
                let mut region = SkRegion::default();
                for rect in op.shape() {
                    region.op_rect(
                        SkRegionIRect::MakeLTRB(rect.x(), rect.y(), rect.right(), rect.bottom()),
                        Op::kUnion_Op,
                    );
                }
                result.push(RenderSurfaceFilter::AlphaThreshold {
                    region,
                    local_to_device: [
                        matrix.sx, matrix.ky, matrix.kx, matrix.sy, matrix.tx, matrix.ty,
                    ],
                });
            }
            ZOOM => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "native Skia magnifier image-filter receiver is not exposed",
                ))
            }
            GRAYSCALE | SEPIA | SATURATE | HUE_ROTATE | INVERT | OPACITY | BRIGHTNESS
            | CONTRAST => unreachable!("handled native color matrix"),
        }
    }
    Ok(result)
}
pub fn BuildGraph(
    filter: &dyn PaintFilter,
    matrix: skia::raster::Transform,
) -> io::Result<Vec<RenderSurfaceFilter>> {
    match filter.ReplayGraph() {
        PaintFilterReplay::Operations(ops) => BuildFilters(ops, matrix),
        PaintFilterReplay::Reference(_) => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "SVG filter execution requires its actual reference-box and user-space geometry",
        )),
        PaintFilterReplay::Reflection(_) => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "reflection execution requires its actual native reference coordinate mapping",
        )),
    }
}

/// Value-only mapping between the same native SkBlendMode vocabularies.
pub fn NativeBlendMode(
    mode: compositor_core::adapter::property_types::BlendMode,
) -> skia::BlendMode {
    match mode {
        compositor_core::adapter::property_types::BlendMode::kClear => skia::BlendMode::kClear,
        compositor_core::adapter::property_types::BlendMode::kSrc => skia::BlendMode::kSrc,
        compositor_core::adapter::property_types::BlendMode::kDst => skia::BlendMode::kDst,
        compositor_core::adapter::property_types::BlendMode::kSrcOver => skia::BlendMode::kSrcOver,
        compositor_core::adapter::property_types::BlendMode::kDstOver => skia::BlendMode::kDstOver,
        compositor_core::adapter::property_types::BlendMode::kSrcIn => skia::BlendMode::kSrcIn,
        compositor_core::adapter::property_types::BlendMode::kDstIn => skia::BlendMode::kDstIn,
        compositor_core::adapter::property_types::BlendMode::kSrcOut => skia::BlendMode::kSrcOut,
        compositor_core::adapter::property_types::BlendMode::kDstOut => skia::BlendMode::kDstOut,
        compositor_core::adapter::property_types::BlendMode::kSrcATop => skia::BlendMode::kSrcATop,
        compositor_core::adapter::property_types::BlendMode::kDstATop => skia::BlendMode::kDstATop,
        compositor_core::adapter::property_types::BlendMode::kXor => skia::BlendMode::kXor,
        compositor_core::adapter::property_types::BlendMode::kPlus => skia::BlendMode::kPlus,
        compositor_core::adapter::property_types::BlendMode::kModulate => {
            skia::BlendMode::kModulate
        }
        compositor_core::adapter::property_types::BlendMode::kScreen => skia::BlendMode::kScreen,
        compositor_core::adapter::property_types::BlendMode::kOverlay => skia::BlendMode::kOverlay,
        compositor_core::adapter::property_types::BlendMode::kDarken => skia::BlendMode::kDarken,
        compositor_core::adapter::property_types::BlendMode::kLighten => skia::BlendMode::kLighten,
        compositor_core::adapter::property_types::BlendMode::kColorDodge => {
            skia::BlendMode::kColorDodge
        }
        compositor_core::adapter::property_types::BlendMode::kColorBurn => {
            skia::BlendMode::kColorBurn
        }
        compositor_core::adapter::property_types::BlendMode::kHardLight => {
            skia::BlendMode::kHardLight
        }
        compositor_core::adapter::property_types::BlendMode::kSoftLight => {
            skia::BlendMode::kSoftLight
        }
        compositor_core::adapter::property_types::BlendMode::kDifference => {
            skia::BlendMode::kDifference
        }
        compositor_core::adapter::property_types::BlendMode::kExclusion => {
            skia::BlendMode::kExclusion
        }
        compositor_core::adapter::property_types::BlendMode::kMultiply => {
            skia::BlendMode::kMultiply
        }
        compositor_core::adapter::property_types::BlendMode::kHue => skia::BlendMode::kHue,
        compositor_core::adapter::property_types::BlendMode::kSaturation => {
            skia::BlendMode::kSaturation
        }
        compositor_core::adapter::property_types::BlendMode::kColor => skia::BlendMode::kColor,
        compositor_core::adapter::property_types::BlendMode::kLuminosity => {
            skia::BlendMode::kLuminosity
        }
    }
}
