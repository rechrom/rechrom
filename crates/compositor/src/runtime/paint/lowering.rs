//! Paint replay ABI -> actual cc primitive ownership. Original authored payloads
//! remain in their immutable PaintRecord allocation. Only new cc operations and
//! scalar shader/shape values are allocated; no PaintArtifact is copied.
use super::{CpuPaintFlags, CpuPaintOp, PaintOp, PaintRecord};
use crate::adapter::display_item_list::DisplayPaintOpBuffer;
use crate::{
    adapter::geometry::{RRectF, RectF, Vector2dF},
    runtime::types::CpuSkiaBackend,
};
use paint::paint_engine::{DisplayItemType as N, PaintRect};
use skia::{include::core::SkColor::SkColor4f, BlendMode};
use std::sync::Arc;

#[derive(Clone)]
pub struct NativePaintSource {
    record: PaintRecord,
    index: usize,
}
impl NativePaintSource {
    pub(crate) fn At(record: PaintRecord, index: usize) -> Self {
        assert!(index < record.len());
        Self { record, index }
    }
    pub fn op(&self) -> &PaintOp {
        &self.record.ops()[self.index]
    }
    pub fn record(&self) -> &PaintRecord {
        &self.record
    }
}
/// Actual immutable shader input. The lease resolves the original native image
/// or generated-image owner and its sampling/orientation/container properties.
/// A mask index refers to that same record's resident mask layer, never a copy.
pub enum CpuPaintShader<R: CpuSkiaBackend = crate::runtime::skia::SkRegion> {
    ImagePattern {
        source: NativePaintSource,
        mask_layer: Option<usize>,
    },
    GeneratedPattern {
        source: NativePaintSource,
        mask_layer: Option<usize>,
    },
    /// Source: cc/paint/paint_shader.cc:210-224. The shader owns the actual
    /// immutable cc record; neither its commands nor Paint resources are copied.
    PaintRecord {
        record: super::CpuPaintRecord<R>,
        tile_rect: RectF,
        tile_mode_x: skia::raster::SpreadMode,
        tile_mode_y: skia::raster::SpreadMode,
        local_matrix: Option<crate::runtime::geometry::Transform<R>>,
    },
}
impl<R: CpuSkiaBackend> CpuPaintShader<R> {
    pub fn source(&self) -> Option<(&NativePaintSource, Option<usize>)> {
        match self {
            Self::ImagePattern { source, mask_layer }
            | Self::GeneratedPattern { source, mask_layer } => Some((source, *mask_layer)),
            Self::PaintRecord { .. } => None,
        }
    }
}
#[derive(Clone, Copy)]
pub enum CpuPaintColorFilter {
    Luma,
}
fn rect(r: PaintRect) -> RectF {
    RectF::new(r.x as f32, r.y as f32, r.width as f32, r.height as f32)
}
fn rrect(r: PaintRect, radii: paint::paint_engine::PaintCornerRadii) -> RRectF {
    RRectF {
        rect: rect(r),
        corner_radii: [
            radii.top_left,
            radii.top_right,
            radii.bottom_right,
            radii.bottom_left,
        ]
        .map(|r| Vector2dF::new(r.x as f32, r.y as f32)),
    }
}
fn flags<R: CpuSkiaBackend>(op: &PaintOp) -> CpuPaintFlags<R> {
    let mut f = CpuPaintFlags::default();
    f.setColor(SkColor4f::new(
        op.color.red,
        op.color.green,
        op.color.blue,
        op.color.alpha,
    ));
    f.setAntiAlias(op.antialias);
    f.setBlendMode(native_blend(op.blend_mode));
    f
}
/// Lower the actual source record once, when ContentDisplayItemList receives
/// it. Native primitive leases preserve blobs, glyphs, paths and resources;
/// compound commands below become an actual cc operation sequence.
pub fn LowerPaintRecord<R: CpuSkiaBackend>(
    record: PaintRecord,
    values: Arc<dyn super::CpuPaintValueServices<R>>,
) -> Vec<CpuPaintOp<R>> {
    let mut output = Vec::with_capacity(record.len());
    for (index, op) in record.ops().iter().enumerate() {
        let source = NativePaintSource {
            record: record.clone(),
            index,
        };
        match op.r#type {
            // Source: third_party/blink/renderer/platform/graphics/image.cc:245-342.
            // Image::DrawPattern emits DrawRect with its actual image shader,
            // not DrawImageRect. Sampling/tiling remain in the shader value.
            N::kDrawTiledImage => {
                let mut f = flags(op);
                f.setColor(SkColor4f::new(0., 0., 0., 1.));
                f.shader = Some(Arc::new(CpuPaintShader::ImagePattern {
                    source,
                    mask_layer: None,
                }));
                output.push(CpuPaintOp::DrawRect(rect(op.rect), f));
            }
            // Source: third_party/blink/renderer/platform/graphics/generated_image.cc:43-94.
            N::kDrawGradientRect | N::kDrawTiledGradient => {
                let mut f = flags(op);
                f.setColor(SkColor4f::new(0., 0., 0., 1.));
                f.shader = Some(Arc::new(CpuPaintShader::GeneratedPattern {
                    source,
                    mask_layer: None,
                }));
                output.push(CpuPaintOp::DrawRect(rect(op.rect), f));
            }
            N::kStrokeWavyLine => LowerWavy(source, values.clone(), &mut output),
            N::kDrawMask => LowerMask(source, &mut output),
            N::kDrawBoxShadow => LowerBoxShadow(source, &mut output),
            _ => output.push(CpuPaintOp::NativeAtom(source)),
        }
    }
    output
}
// Source: third_party/blink/renderer/core/paint/decoration_line_painter.cc:
// 225-238,281-289,306-324,497-526. The tile is a true stroked-path record;
// the decoration itself is a DrawRect with Repeat-X/Decal-Y PaintRecordShader.
fn LowerWavy<R: CpuSkiaBackend>(
    source: NativePaintSource,
    values: Arc<dyn super::CpuPaintValueServices<R>>,
    out: &mut Vec<CpuPaintOp<R>>,
) {
    let op = source.op();
    let thickness = op.stroke_width as f32;
    let wave = paint::wavy_geometry::MakeWave(thickness);
    let path = Arc::new(paint::wavy_geometry::WavyPath(&wave));
    let Some(bounds) = paint::wavy_geometry::ComputeWavyPatternRect(thickness, &wave, &path) else {
        // Native Skia rejects non-finite/empty stroke geometry as no drawing.
        return;
    };
    let [left, top, right, bottom] = bounds;
    let mut tile = super::CpuPaintOpBuffer::<R>::new(values);
    tile.push(CpuPaintOp::Translate(-left, -top));
    let mut stroke = flags(op);
    stroke.setAntiAlias(true);
    stroke.setStyle(paint::cc::paint::paint_flags::Style::kStroke_Style);
    stroke.setStrokeWidth(thickness);
    tile.push(CpuPaintOp::DrawPath {
        path,
        flags: stroke,
    });
    let record = super::CpuPaintRecord::Buffer(Arc::new(tile));
    let mut f = CpuPaintFlags::default();
    f.setAntiAlias(true);
    f.shader = Some(Arc::new(CpuPaintShader::PaintRecord {
        record,
        tile_rect: RectF::new(0., 0., right - left, bottom - top),
        tile_mode_x: skia::raster::SpreadMode::Repeat,
        tile_mode_y: skia::raster::SpreadMode::kDecal,
        local_matrix: None,
    }));
    out.push(CpuPaintOp::Save);
    if op.is_shadow {
        out.push(CpuPaintOp::Translate(
            op.shadow_offset.x as f32,
            op.shadow_offset.y as f32,
        ));
        if op.blur_radius > 0. {
            let mut ops = paint::cc::paint::filter_operations::FilterOperations::default();
            ops.Append(
                &paint::cc::paint::filter_operation::FilterOperation::CreateBlurFilter(
                    (op.blur_radius * 0.5) as f32,
                ),
            );
            let mut layer = CpuPaintFlags::default();
            layer.image_filter = Some(Arc::new(super::CpuOperationsFilter(ops)));
            out.push(CpuPaintOp::SaveLayer {
                bounds: None,
                flags: layer,
            });
        }
    }
    let vertical = op.rect.width == 0. && op.rect.height != 0.;
    if vertical {
        // The producer's vertical line is the same native local line after the
        // writing-mode rotation. Source line_relative_rect.cc:19–82 maps
        // the line-relative inline axis down and block axis left for both
        // supported vertical modes; op.rect already supplies its physical origin.
        // Express that same rotation as a real cc matrix.
        let mut matrix = [0.; 16];
        matrix[1] = 1.;
        matrix[4] = -1.;
        matrix[10] = 1.;
        matrix[15] = 1.;
        matrix[12] = op.rect.x;
        matrix[13] = op.rect.y;
        out.push(CpuPaintOp::Concat(
            crate::runtime::geometry::Transform::FromColMajor(&matrix),
        ));
        out.push(CpuPaintOp::Translate(left, top));
    } else {
        out.push(CpuPaintOp::Translate(
            op.rect.x as f32 + left,
            op.rect.y as f32 + top,
        ));
    }
    out.push(CpuPaintOp::DrawRect(
        RectF::new(
            0.,
            0.,
            if vertical {
                op.rect.height as f32
            } else {
                op.rect.width as f32
            },
            bottom - top,
        ),
        f,
    ));
    if op.is_shadow && op.blur_radius > 0. {
        out.push(CpuPaintOp::Restore);
    }
    out.push(CpuPaintOp::Restore);
}
fn LowerMask<R: CpuSkiaBackend>(source: NativePaintSource, out: &mut Vec<CpuPaintOp<R>>) {
    use paint::paint_engine::{PaintMaskComposite as Composite, PaintMaskMode as Mode};
    let op = source.op();
    // Source: third_party/blink/renderer/core/paint/box_painter_base.cc:1335-1425,1777-1795.
    // PaintMaskImages/PaintFillLayer: isolated mask plane, back-to-front fill
    // layers; the final owning effect's DstIn is a separate source operation.
    out.push(CpuPaintOp::SaveLayer {
        bounds: None,
        flags: CpuPaintFlags::default(),
    });
    for (index, layer) in op.mask_layers.iter().enumerate().rev() {
        if layer.resource_id == 0 && layer.paint_shader.is_none() {
            continue;
        }
        let mut f = CpuPaintFlags::default();
        f.setBlendMode(if index + 1 == op.mask_layers.len() {
            BlendMode::kSrcOver
        } else {
            match layer.composite {
                Composite::kAdd => BlendMode::kSrcOver,
                Composite::kSubtract => BlendMode::kSrcOut,
                Composite::kIntersect => BlendMode::kSrcIn,
                Composite::kExclude => BlendMode::kXor,
            }
        });
        if layer.mode == Mode::kLuminance {
            f.color_filter = Some(CpuPaintColorFilter::Luma);
        }
        out.push(CpuPaintOp::SaveLayer {
            bounds: None,
            flags: f,
        });
        out.push(CpuPaintOp::ClipPath {
            path: Arc::new(paint::paint_clip_path::PaintClipPath::from_path(
                paint::paint_clip_path::rounded_bounds(layer.clip_rect, layer.clip_radii),
            )),
            difference: false,
            antialias: true,
        });
        let mut f = CpuPaintFlags::default();
        f.shader = Some(Arc::new(if layer.paint_shader.is_some() {
            CpuPaintShader::GeneratedPattern {
                source: source.clone(),
                mask_layer: Some(index),
            }
        } else {
            CpuPaintShader::ImagePattern {
                source: source.clone(),
                mask_layer: Some(index),
            }
        }));
        out.push(CpuPaintOp::DrawRect(rect(layer.clip_rect), f));
        out.push(CpuPaintOp::Restore);
    }
    out.push(CpuPaintOp::Restore);
}
fn LowerBoxShadow<R: CpuSkiaBackend>(source: NativePaintSource, out: &mut Vec<CpuPaintOp<R>>) {
    use paint::cc::paint::draw_looper::{kOverrideAlphaFlag, DrawLooperBuilder, Point};
    let op = source.op();
    // Source: third_party/blink/renderer/core/paint/box_painter_base.cc:276-377,502-578.
    let mut builder = DrawLooperBuilder::default();
    builder.AddShadow(
        Point {
            x: op.shadow_offset.x as f32,
            y: op.shadow_offset.y as f32,
        },
        (op.blur_radius * 0.5) as f32,
        SkColor4f::new(op.color.red, op.color.green, op.color.blue, op.color.alpha),
        kOverrideAlphaFlag,
        false,
    );
    let mut f = CpuPaintFlags::default();
    f.setAntiAlias(op.antialias);
    f.looper = Some(builder.Detach());
    out.push(CpuPaintOp::Save);
    if op.inset {
        let inner = rrect(op.rect, op.corner_radii);
        let mut hole = inner;
        OutsetWithCornerCorrection(&mut hole, -(op.spread as f32));
        if hole.rect.width() <= 0. || hole.rect.height() <= 0. {
            f.looper = None;
            f.setColor(SkColor4f::new(
                op.color.red,
                op.color.green,
                op.color.blue,
                op.color.alpha,
            ));
            out.push(CpuPaintOp::DrawRRect(inner, f));
        } else {
            out.push(CpuPaintOp::ClipPath {
                path: Arc::new(paint::paint_clip_path::PaintClipPath::from_path(
                    paint::paint_clip_path::rounded_bounds(op.rect, op.corner_radii),
                )),
                difference: false,
                antialias: true,
            });
            let blur = op.blur_radius as f32;
            let extra = blur + (-(op.spread as f32)).max(0.);
            let mut outer = rect(op.rect);
            outer = RectF::new(
                outer.x() - extra,
                outer.y() - extra,
                outer.width() + 2. * extra,
                outer.height() + 2. * extra,
            );
            let ox = op.shadow_offset.x as f32;
            let oy = op.shadow_offset.y as f32;
            let left = outer.x().min(outer.x() - ox);
            let top = outer.y().min(outer.y() - oy);
            let right = outer.right().max(outer.right() - ox);
            let bottom = outer.bottom().max(outer.bottom() - oy);
            let outer = RRectF {
                rect: RectF::new(left, top, right - left, bottom - top),
                corner_radii: [Vector2dF::default(); 4],
            };
            f.setColor(SkColor4f::new(
                op.color.red,
                op.color.green,
                op.color.blue,
                1.,
            ));
            out.push(CpuPaintOp::DrawDRRect {
                outer,
                inner: hole,
                flags: f,
            });
        }
    } else {
        let mut hole = op.rect;
        let mut radii = op.inner_corner_radii;
        if op.shadow_has_opaque_background {
            hole.x += 1.;
            hole.y += 1.;
            hole.width -= 2.;
            hole.height -= 2.;
            for r in [
                &mut radii.top_left,
                &mut radii.top_right,
                &mut radii.bottom_left,
                &mut radii.bottom_right,
            ] {
                r.x = (r.x - 1.).max(0.);
                r.y = (r.y - 1.).max(0.);
            }
        }
        if hole.width > 0. && hole.height > 0. {
            out.push(CpuPaintOp::ClipPath {
                path: Arc::new(paint::paint_clip_path::PaintClipPath::from_path(
                    paint::paint_clip_path::rounded_bounds(hole, radii),
                )),
                difference: true,
                antialias: true,
            });
        }
        let casting = PaintRect {
            x: op.rect.x - op.spread,
            y: op.rect.y - op.spread,
            width: op.rect.width + 2. * op.spread,
            height: op.rect.height + 2. * op.spread,
        };
        if casting.width > 0. && casting.height > 0. {
            out.push(CpuPaintOp::DrawRRect(rrect(casting, op.corner_radii), f));
        }
    }
    out.push(CpuPaintOp::Restore);
}
// Source: third_party/blink/renderer/platform/geometry/float_rounded_rect.cc:133-191,209-219,289-307.
fn OutsetWithCornerCorrection(value: &mut RRectF, outset: f32) {
    if outset == 0. {
        return;
    }
    let size = value.rect.size();
    value.rect = RectF::new(
        value.rect.x() - outset,
        value.rect.y() - outset,
        value.rect.width() + 2. * outset,
        value.rect.height() + 2. * outset,
    );
    if !size.IsEmpty() {
        for corner in &mut value.corner_radii {
            if corner.x() == 0. && corner.y() == 0. {
                continue;
            }
            let coverage = 2. * (corner.x() / size.width()).min(corner.y() / size.height());
            let adjust = |radius: f32| {
                if radius > outset || coverage > 1. {
                    radius + outset
                } else {
                    let ratio = radius / outset;
                    radius + outset * (1. - (1. - ratio).powi(3) * (1. - coverage.powi(3)))
                }
            };
            *corner = Vector2dF::new(adjust(corner.x()), adjust(corner.y()));
        }
    }
    let horizontal = (value.corner_radii[0].x() + value.corner_radii[1].x())
        .max(value.corner_radii[3].x() + value.corner_radii[2].x());
    let vertical = (value.corner_radii[0].y() + value.corner_radii[3].y())
        .max(value.corner_radii[1].y() + value.corner_radii[2].y());
    let mut factor = 1f32;
    if horizontal > value.rect.width() {
        factor = factor.min(value.rect.width() / horizontal);
    }
    if vertical > value.rect.height() {
        factor = factor.min(value.rect.height() / vertical);
    }
    for corner in &mut value.corner_radii {
        *corner = Vector2dF::new(corner.x() * factor, corner.y() * factor);
    }
}

fn native_blend(m: paint::paint_property_tree::PaintBlendMode) -> BlendMode {
    use paint::paint_property_tree::PaintBlendMode::*;
    match m {
        kNormal => BlendMode::kSrcOver,
        kPlusLighter => BlendMode::kPlus,
        kMultiply => BlendMode::kMultiply,
        kScreen => BlendMode::kScreen,
        kOverlay => BlendMode::kOverlay,
        kDarken => BlendMode::kDarken,
        kLighten => BlendMode::kLighten,
        kColorDodge => BlendMode::kColorDodge,
        kColorBurn => BlendMode::kColorBurn,
        kHardLight => BlendMode::kHardLight,
        kSoftLight => BlendMode::kSoftLight,
        kDifference => BlendMode::kDifference,
        kExclusion => BlendMode::kExclusion,
        kHue => BlendMode::kHue,
        kSaturation => BlendMode::kSaturation,
        kColor => BlendMode::kColor,
        kLuminosity => BlendMode::kLuminosity,
    }
}
