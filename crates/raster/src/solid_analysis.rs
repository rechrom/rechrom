//! Narrow translation of cc/paint/solid_color_analyzer.cc, before raster.
//! DetermineIfSolidColor starts transparent, tracks Save/Restore/rect clips,
//! and counts DrawRect across nested records, not state instructions. A later
//! opaque full-canvas rectangle can restore a previously non-solid result.
//! TileManager::AssignGpuMemoryToTiles limits analysis to five draw operations.
//! This subset rejects unsupported operations; it never inspects raster pixels.
use super::solid::RasterDrawMode;
use crate::layer_replay::ReplayUnsupported;
use layer_tile::RasterTask;
use skia::compat::commands::{CommandKind as Kind, DrawCommand, PaintBlendMode, PaintRect};
use skia::include::core::SkColor::SkColor4f;

#[derive(Clone, Copy)]
struct Rect {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}
impl Rect {
    fn contains(self, other: Self) -> bool {
        self.left <= other.left
            && self.top <= other.top
            && self.right >= other.right
            && self.bottom >= other.bottom
    }
    fn intersect(self, other: Self) -> Self {
        Self {
            left: self.left.max(other.left),
            top: self.top.max(other.top),
            right: self.right.min(other.right),
            bottom: self.bottom.min(other.bottom),
        }
    }
}
#[derive(Clone, Copy)]
struct State {
    sx: f32,
    sy: f32,
    tx: f32,
    ty: f32,
    clip: Rect,
}
impl State {
    fn map(self, rect: PaintRect) -> Option<Rect> {
        // Match SkCanvas::drawRect's source f32 edges and positive-axis CTM.
        // Integer device edges are a deliberately narrower accepted subset
        // than SkNoDrawCanvas, avoiding an invented AA-clip approximation.
        if rect.width < 0.0 || rect.height < 0.0 {
            return None;
        }
        let left = rect.x as f32 * self.sx + self.tx;
        let top = rect.y as f32 * self.sy + self.ty;
        let right = (rect.x as f32 + rect.width as f32) * self.sx + self.tx;
        let bottom = (rect.y as f32 + rect.height as f32) * self.sy + self.ty;
        if ![left, top, right, bottom]
            .into_iter()
            .all(|v| v.is_finite() && v.fract() == 0.0 && v.abs() <= ((1u32 << 22) as f32))
        {
            return None;
        }
        Some(Rect {
            left,
            top,
            right,
            bottom,
        })
    }
}

pub(super) fn analyze<'a>(
    task: &RasterTask,
    records: impl Iterator<Item = Result<&'a [DrawCommand], ReplayUnsupported>>,
) -> RasterDrawMode {
    let (width, height) = task.pixel_size;
    let scale = task.raster_scale as f32;
    let tx = (-task.tile_rect.x * task.raster_scale) as f32;
    let ty = (-task.tile_rect.y * task.raster_scale) as f32;
    if width == 0
        || height == 0
        || !scale.is_finite()
        || scale <= 0.0
        || scale as f64 != task.raster_scale
        || tx as f64 != -task.tile_rect.x * task.raster_scale
        || ty as f64 != -task.tile_rect.y * task.raster_scale
        || task.tile_rect.width * task.raster_scale != width as f64
        || task.tile_rect.height * task.raster_scale != height as f64
        || ![tx, ty]
            .into_iter()
            .all(|v| v.is_finite() && v.fract() == 0.0 && v.abs() <= ((1u32 << 22) as f32))
    {
        return RasterDrawMode::Resource;
    }
    analyze_records(
        State {
            sx: scale,
            sy: scale,
            tx,
            ty,
            clip: Rect {
                left: 0.0,
                top: 0.0,
                right: width as f32,
                bottom: height as f32,
            },
        },
        records,
    )
}

fn analyze_records<'a>(
    initial: State,
    records: impl Iterator<Item = Result<&'a [DrawCommand], ReplayUnsupported>>,
) -> RasterDrawMode {
    let canvas = initial.clip;
    let mut state = initial;
    let mut saved = Vec::with_capacity(2);
    // Official initial transparent color is not an opaque SolidColor mode.
    // White-backed receiver initialization is never treated as a PaintOp.
    let mut solid = None;
    let mut draws = 0;
    for record in records {
        let Ok(commands) = record else {
            return RasterDrawMode::Resource;
        };
        for command in commands {
            match command.r#type {
                Kind::kSave => saved.push(state),
                Kind::kRestore => {
                    let Some(previous) = saved.pop() else {
                        return RasterDrawMode::Resource;
                    };
                    state = previous;
                }
                Kind::kConcat => {
                    let identity = skia::compat::commands::TransformMatrix::default();
                    let m = &command.transform.values;
                    if m.iter().enumerate().any(|(i, &v)| {
                        !v.is_finite() || (!matches!(i, 12 | 13) && v != identity.values[i])
                    }) {
                        return RasterDrawMode::Resource;
                    }
                    let dx = m[12] as f32 * state.sx;
                    let dy = m[13] as f32 * state.sy;
                    if ![dx, dy]
                        .into_iter()
                        .all(|v| v.is_finite() && v.fract() == 0.0)
                    {
                        return RasterDrawMode::Resource;
                    }
                    state.tx += dx;
                    state.ty += dy;
                }
                Kind::kClipRect => {
                    let Some(rect) = state.map(command.rect) else {
                        return RasterDrawMode::Resource;
                    };
                    state.clip = state.clip.intersect(rect);
                }
                Kind::kDrawRect => {
                    draws += 1;
                    if draws > 5 {
                        return RasterDrawMode::Resource;
                    }
                    // IsSolidColorPaint's fill/SrcOver/no-effects subset.
                    if command.blend_mode != PaintBlendMode::kNormal
                        || command.paint_shader.is_some()
                        || !command.filters.is_empty()
                        || !command.mask_layers.is_empty()
                        || command.resource_id != 0
                    {
                        return RasterDrawMode::Resource;
                    }
                    let color = command.color;
                    if ![color.red, color.green, color.blue, color.alpha]
                        .into_iter()
                        .all(|v| v.is_finite() && (0.0..=1.0).contains(&v))
                    {
                        return RasterDrawMode::Resource;
                    }
                    // CheckIfSolidShape first respects flags.nothingToDraw().
                    if color.alpha == 0.0 {
                        continue;
                    }
                    let Some(rect) = state.map(command.rect) else {
                        return RasterDrawMode::Resource;
                    };
                    // IsFullQuad requires both the clip and shape to cover the
                    // complete analysis canvas. Mac IsSolidColorPaint also
                    // requires alpha==1; partial/translucent draws invalidate
                    // solidness, but a later opaque full draw can recover it.
                    solid = if color.alpha == 1.0
                        && state.clip.contains(canvas)
                        && rect.contains(canvas)
                    {
                        // The exact same N32 conversion as CPU SkCanvas's
                        // mask_blitter::premultiply; no new quantization rule.
                        Some(skia::src::core::SkColor::premultiply(SkColor4f::new(
                            color.red,
                            color.green,
                            color.blue,
                            color.alpha,
                        )))
                    } else {
                        None
                    };
                }
                _ => return RasterDrawMode::Resource,
            }
        }
        // Each compiled record is a real independently saved DrawRecord.
        if !saved.is_empty() {
            return RasterDrawMode::Resource;
        }
        state = initial;
    }
    solid.map_or(RasterDrawMode::Resource, |rgba| {
        RasterDrawMode::SolidColor {
            premul_rgba: u32::from_le_bytes(rgba),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use skia::compat::commands::{Color, PaintShader};
    #[test]
    fn full_draw_recovers_partial_but_budget_and_shader_reject() {
        let state = State {
            sx: 1.0,
            sy: 1.0,
            tx: 0.0,
            ty: 0.0,
            clip: Rect {
                left: 0.0,
                top: 0.0,
                right: 16.0,
                bottom: 16.0,
            },
        };
        let full = DrawCommand {
            rect: PaintRect {
                x: 0.0,
                y: 0.0,
                width: 16.0,
                height: 16.0,
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
        partial.rect.width = 8.0;
        let input = [partial, full.clone()];
        let expected = RasterDrawMode::SolidColor {
            premul_rgba: u32::from_le_bytes(skia::src::core::SkColor::premultiply(SkColor4f::new(
                0.2, 0.4, 0.6, 1.0,
            ))),
        };
        assert_eq!(
            analyze_records(state, std::iter::once(Ok(input.as_slice()))),
            expected
        );
        let five = vec![full.clone(); 5];
        assert_eq!(
            analyze_records(state, std::iter::once(Ok(five.as_slice()))),
            expected
        );
        let six = vec![full.clone(); 6];
        assert_eq!(
            analyze_records(state, std::iter::once(Ok(six.as_slice()))),
            RasterDrawMode::Resource
        );
        assert_eq!(
            analyze_records(state, [Ok(&six[..3]), Ok(&six[3..])].into_iter()),
            RasterDrawMode::Resource
        );
        let mut shader = full;
        shader.paint_shader = Some(PaintShader::default());
        assert_eq!(
            analyze_records(state, std::iter::once(Ok(std::slice::from_ref(&shader)))),
            RasterDrawMode::Resource
        );
    }
}
