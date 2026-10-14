//! Solid painted scrollbar layer inputs, corresponding to Blink's
//! ScrollbarDisplayItem and cc::PaintedScrollbarLayerImpl's thumb_color branch.
use crate::{FramePlan, LayerId};
use paint::paint_engine::{DisplayItemType as Kind, PaintRect, RecordedDisplayItemKind};

#[derive(Clone)]
pub struct ScrollbarSolidRect {
    pub rect: PaintRect,
    pub color: [u8; 4],
    pub is_thumb: bool,
}
#[derive(Clone)]
pub struct SolidScrollbarLayer {
    pub layer_id: LayerId,
    pub horizontal: bool,
    pub track: PaintRect,
    pub start_inset: f64,
    pub end_inset: f64,
    pub parts: Vec<ScrollbarSolidRect>,
}
impl FramePlan {
    /// Consume backend-certified primitive scrollbar recordings. Unsupported
    /// recordings keep their ordinary painted layer; no generic theme is
    /// substituted for the actual native colors, dimensions or property state.
    pub fn SolidScrollbarLayers(&self, scroll_node: u64) -> Vec<SolidScrollbarLayer> {
        self.layers
            .iter()
            .filter_map(|layer| {
                let [index] = layer.record_indices.as_slice() else {
                    return None;
                };
                let record = &self.GetPaintArtifact().display_items[*index];
                if record.kind != RecordedDisplayItemKind::Scrollbar
                    || record
                        .scroll_translation
                        .as_ref()
                        .and_then(|node| node.scroll.as_ref())
                        .is_none_or(|scroll| scroll.id != scroll_node)
                {
                    return None;
                }
                let mut effect = Some(&layer.properties.effect);
                while let Some(node) = effect {
                    if node.opacity != 1.0
                        || node.blend_mode
                            != layoutng_assembly::internal::paint_input::PaintBlendMode::kNormal
                        || node.is_mask
                        || !node.filters.is_empty()
                        || !node.backdrop_filters.is_empty()
                    {
                        return None;
                    }
                    effect = node.parent.as_ref();
                }
                let (translation, clip) = crate::resolved_compositor_properties(
                    &layer.properties,
                    self.config.raster_scale,
                )
                .ok()?;
                let content = self
                    .GetRasterRecords()
                    .iter()
                    .find(|content| content.record_index == *index)?;
                let mut parts = Vec::new();
                let mut track = None;
                let mut horizontal = false;
                let mut start_inset: f64 = 0.0;
                let mut end_inset: f64 = 0.0;
                let mut clips = Vec::new();
                for item in &*content.items {
                    match item.r#type {
                        Kind::kSave | Kind::kRestore => continue,
                        Kind::kClipRect => {
                            clips.push(item.rect);
                            continue;
                        }
                        Kind::kDrawScrollbarTrack
                        | Kind::kDrawScrollbarThumb
                        | Kind::kDrawScrollbarButton => {}
                        _ => return None,
                    }
                    if item.paint_shader.is_some()
                        || !item.filters.is_empty()
                        || item.blend_mode
                            != layoutng_assembly::internal::paint_input::PaintBlendMode::kNormal
                        || item.opacity != 1.0
                    {
                        return None;
                    }
                    let rect = PaintRect {
                        x: item.rect.x + translation.0 / self.config.raster_scale,
                        y: item.rect.y + translation.1 / self.config.raster_scale,
                        ..item.rect
                    };
                    if item.r#type == Kind::kDrawScrollbarTrack {
                        track = Some(rect);
                        horizontal = item.horizontal;
                    }
                    if item.r#type == Kind::kDrawScrollbarButton {
                        let extent = if item.horizontal {
                            rect.width
                        } else {
                            rect.height
                        };
                        if item.at_start {
                            start_inset = extent;
                        } else {
                            end_inset = extent;
                        }
                    }
                    let alpha = item.color.alpha.clamp(0.0, 1.0);
                    let channel =
                        |value: f32| (value.clamp(0.0, 1.0) * alpha * 255.0).round() as u8;
                    parts.push(ScrollbarSolidRect {
                        rect,
                        color: [
                            channel(item.color.red),
                            channel(item.color.green),
                            channel(item.color.blue),
                            (alpha * 255.0).round() as u8,
                        ],
                        is_thumb: item.r#type == Kind::kDrawScrollbarThumb,
                    });
                }
                let track = track?;
                let contains = |outer: PaintRect, inner: PaintRect| {
                    outer.x <= inner.x
                        && outer.y <= inner.y
                        && outer.x + outer.width >= inner.x + inner.width
                        && outer.y + outer.height >= inner.y + inner.height
                };
                if clips.iter().any(|rect| {
                    !contains(
                        PaintRect {
                            x: rect.x + translation.0 / self.config.raster_scale,
                            y: rect.y + translation.1 / self.config.raster_scale,
                            ..*rect
                        },
                        track,
                    )
                }) || clip.is_some_and(|rect| {
                    !contains(
                        PaintRect {
                            x: rect.x / self.config.raster_scale,
                            y: rect.y / self.config.raster_scale,
                            width: rect.width / self.config.raster_scale,
                            height: rect.height / self.config.raster_scale,
                        },
                        track,
                    )
                }) {
                    return None;
                }
                Some(SolidScrollbarLayer {
                    layer_id: layer.id,
                    horizontal,
                    track,
                    start_inset,
                    end_inset,
                    parts,
                })
            })
            .collect()
    }
}
