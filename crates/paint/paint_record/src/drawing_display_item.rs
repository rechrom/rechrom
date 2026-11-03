//! Source-shaped `DrawingDisplayItem` record analysis.
//!
//! Rechrom stores all PaintOps in the containing PaintArtifact and retains a
//! range on each item. Methods therefore accept the resident artifact where
//! Blink's C++ subtype owns a `PaintRecord` value directly.

use layoutng_assembly::internal::layout_input::Offset;
use layoutng_assembly::internal::paint_input::PaintBlendMode;
use paint_chunk::PaintChunkBackgroundColorInfo;
use paint_model::display_item_id::{DisplayItemId, DisplayItemIdType};
use paint_model::PaintRect;

use crate::pre_paint_tree_walk::EnclosingRectInPropertySpace;
use crate::{DisplayItem, DisplayItemType, PaintRecording, RecordedDisplayItem};

// Source: drawing_display_item.cc:67-82.
fn PaintFlagsMayChangeColorOrMovePixelsExceptShader(op: &DisplayItem) -> bool {
    let fill = !matches!(
        op.r#type,
        DisplayItemType::kStrokeEllipse
            | DisplayItemType::kStrokePath
            | DisplayItemType::kStrokeRect
            | DisplayItemType::kStrokeLine
            | DisplayItemType::kStrokeWavyLine
    );
    !fill
        || op.is_shadow
        || op.blur_radius != 0.0
        || !op.canonical_filters.is_empty()
        || op.blend_mode != PaintBlendMode::kNormal
}

fn IsDrawAreaAnalysisCandidate(op: &DisplayItem) -> bool {
    (op.r#type as u8) >= DisplayItemType::kDrawRect as u8
        && !PaintFlagsMayChangeColorOrMovePixelsExceptShader(op)
        && op.paint_shader.is_none()
}

// Source: drawing_display_item.cc:116-157.
pub(crate) fn DrawingBackgroundColor(
    id: DisplayItemId,
    visual: PaintRect,
    record: &[DisplayItem],
) -> PaintChunkBackgroundColorInfo {
    let empty = PaintChunkBackgroundColorInfo::default();
    if record.is_empty() {
        return empty;
    }
    let mut may_be_solid_color = record.len() == 1;
    for op in record {
        if !IsDrawAreaAnalysisCandidate(op) {
            if ![
                DisplayItemIdType::kBoxDecorationBackground,
                DisplayItemIdType::kDocumentBackground,
                DisplayItemIdType::kDocumentRootBackdrop,
                DisplayItemIdType::kScrollCorner,
            ]
            .contains(&id.r#type)
            {
                return empty;
            }
            continue;
        }
        let item_rect = match op.r#type {
            DisplayItemType::kDrawRect => op.rect,
            DisplayItemType::kDrawRoundedRect => {
                may_be_solid_color = false;
                op.rect
            }
            _ => return empty,
        };
        let mut value = PaintChunkBackgroundColorInfo::default();
        value.background_color = op.color;
        value.background_color.alpha *= op.opacity;
        value.background_area = item_rect.width as f32 * item_rect.height as f32;
        value.background_is_solid_color = may_be_solid_color
            && item_rect.x <= visual.x
            && item_rect.y <= visual.y
            && item_rect.x + item_rect.width >= visual.x + visual.width
            && item_rect.y + item_rect.height >= visual.y + visual.height;
        return value;
    }
    empty
}

// Source: drawing_display_item.h:65-73; drawing_display_item.cc:285-320.
pub(crate) fn TightenVisualRect(
    visual_rect: PaintRect,
    record: &[DisplayItem],
    delta: Offset,
) -> (PaintRect, bool) {
    if let [op] = record {
        if matches!(
            op.r#type,
            DisplayItemType::kDrawRect | DisplayItemType::kDrawRoundedRect
        ) && IsDrawAreaAnalysisCandidate(op)
        {
            return (EnclosingRectInPropertySpace(op.rect, delta), true);
        }
    }
    (EnclosingRectInPropertySpace(visual_rect, delta), false)
}

// Source: drawing_display_item.cc:159-283. This remains bounded and
// independent of image pixel contents. The backend-neutral ABI has no
// certified image/shader opacity, so only represented solid geometry supplies
// a proof; unrepresented source cases stay conservative.
pub fn CalculateRectKnownToBeOpaqueForRecord(
    visual_rect: PaintRect,
    record: &[DisplayItem],
) -> PaintRect {
    const OP_COUNT_LIMIT: usize = 8;
    if record.is_empty() {
        return PaintRect::default();
    }
    let mut opaque = PaintRect::default();
    let mut clip = visual_rect.enclosed();
    for op in record.iter().take(OP_COUNT_LIMIT) {
        match op.r#type {
            DisplayItemType::kSave => continue,
            DisplayItemType::kClipRect => {
                clip = clip.intersection(op.rect.enclosed());
                continue;
            }
            kind if (kind as u8) < DisplayItemType::kDrawRect as u8 => break,
            _ => {}
        }
        if PaintFlagsMayChangeColorOrMovePixelsExceptShader(op)
            || !op.mask_layers.is_empty()
            || op.r#type == DisplayItemType::kDrawMask
            || op.paint_shader.is_some()
            || op.color.alpha != 1.0
            || op.opacity != 1.0
        {
            continue;
        }
        let op_opaque_rect = match op.r#type {
            DisplayItemType::kDrawRect => op.rect.enclosed(),
            DisplayItemType::kDrawRoundedRect => {
                let radii = op.corner_radii;
                if [
                    radii.top_left.x,
                    radii.top_left.y,
                    radii.top_right.x,
                    radii.top_right.y,
                    radii.bottom_left.x,
                    radii.bottom_left.y,
                    radii.bottom_right.x,
                    radii.bottom_right.y,
                ]
                .iter()
                .any(|value| !value.is_finite() || *value < 0.0)
                {
                    continue;
                }
                let left = radii.top_left.x.max(radii.bottom_left.x) * 0.3;
                let top = radii.top_left.y.max(radii.top_right.y) * 0.3;
                let right = radii.top_right.x.max(radii.bottom_right.x) * 0.3;
                let bottom = radii.bottom_left.y.max(radii.bottom_right.y) * 0.3;
                PaintRect {
                    x: op.rect.x + left,
                    y: op.rect.y + top,
                    width: op.rect.width - left - right,
                    height: op.rect.height - top - bottom,
                }
                .enclosed()
            }
            _ => continue,
        };
        opaque = PaintRect::maximum_covered_rect(opaque, op_opaque_rect).intersection(clip);
        if opaque == visual_rect.enclosed() {
            break;
        }
    }
    debug_assert!(visual_rect.contains(opaque) || opaque.is_empty());
    opaque
}

impl RecordedDisplayItem {
    // Source: drawing_display_item.h:32-35. The range is the PaintRecord owner
    // mapping for Rechrom's flat immutable artifact ABI.
    pub fn GetPaintRecord<'a>(&self, artifact: &'a PaintRecording) -> &'a [DisplayItem] {
        assert!(!self.IsTombstone());
        assert!(self.IsDrawing());
        &artifact.items[self.record_begin..self.record_end]
    }

    // Source: drawing_display_item.h:37-44,104-110.
    pub fn RectKnownToBeOpaque(&self) -> PaintRect {
        assert!(!self.IsTombstone());
        self.record_rect_known_to_be_opaque
    }

    pub fn BackgroundColor(&self) -> PaintChunkBackgroundColorInfo {
        assert!(!self.IsTombstone());
        self.record_background
    }

    // Source fast paths: drawing_display_item.cc:86-110. Bitmap-result
    // equivalence for structurally different records remains an explicit
    // backend capability and is not fabricated here.
    pub fn EqualsDrawingRecordFastPath(
        &self,
        artifact: &PaintRecording,
        other: &Self,
        other_artifact: &PaintRecording,
    ) -> bool {
        if self.IsTombstone()
            || other.IsTombstone()
            || !self.IsDrawing()
            || !other.IsDrawing()
            || self.VisualRect() != other.VisualRect()
        {
            return false;
        }
        let record = self.GetPaintRecord(artifact);
        let other_record = other.GetPaintRecord(other_artifact);
        (record.is_empty() && other_record.is_empty()) || record == other_record
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::PaintInvalidationReason;
    use paint_model::display_item_id::DisplayItemIdType;

    fn item(begin: usize, end: usize) -> RecordedDisplayItem {
        RecordedDisplayItem {
            is_cacheable: true,
            is_tombstone: false,
            paint_invalidation_reason: Some(PaintInvalidationReason::kNone),
            scrollbar: None,
            foreign_layer: None,
            kind: crate::RecordedDisplayItemKind::Drawing,
            id: DisplayItemId {
                client_id: 1,
                r#type: DisplayItemIdType::kBoxDecorationBackground,
                fragment: 0,
            },
            visual_rect: PaintRect {
                x: 0.0,
                y: 0.0,
                width: 20.0,
                height: 20.0,
            },
            visual_rect_is_accurate: true,
            draws_content: begin != end,
            raster_effect_outset: crate::RasterEffectOutset::kNone,
            record_rect_known_to_be_opaque: PaintRect {
                x: 1.0,
                y: 1.0,
                width: 18.0,
                height: 18.0,
            },
            record_background: PaintChunkBackgroundColorInfo {
                background_area: 400.0,
                ..Default::default()
            },
            record_has_text: false,
            record_begin: begin,
            record_end: end,
            scroll_translation: None,
        }
    }

    #[test]
    fn record_access_and_cached_analysis_stay_on_the_item() {
        let op = DisplayItem {
            r#type: DisplayItemType::kDrawRect,
            rect: PaintRect {
                x: 0.0,
                y: 0.0,
                width: 20.0,
                height: 20.0,
            },
            ..Default::default()
        };
        let artifact = PaintRecording {
            items: std::sync::Arc::new(vec![op]),
            ..Default::default()
        };
        let item = item(0, 1);
        assert_eq!(item.GetPaintRecord(&artifact).len(), 1);
        assert_eq!(item.RectKnownToBeOpaque().width, 18.0);
        assert_eq!(item.BackgroundColor().background_area, 400.0);
        assert!(item.EqualsDrawingRecordFastPath(&artifact, &item, &artifact));
    }

    #[test]
    fn tightening_does_not_add_the_source_todo_image_case() {
        let visual = PaintRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let image = DisplayItem {
            r#type: DisplayItemType::kDrawImageRect,
            rect: PaintRect {
                x: 10.0,
                y: 10.0,
                width: 20.0,
                height: 20.0,
            },
            ..Default::default()
        };
        assert_eq!(
            TightenVisualRect(visual, &[image], Offset::default()),
            (visual, false)
        );
    }
}
