use std::sync::Arc;

use layout::paint::RasterEffectOutset;
use paint_chunk::PaintChunkBackgroundColorInfo;
use paint_model::PaintRect;

pub use crate::scrollbar_display_item::ScrollbarDisplayItemData;
pub use paint_model::DisplayMaskLayer;

pub use paint_model::display_item::{DecorationPaintPass, DisplayItem};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordedDisplayItemKind {
    Drawing,
    Scrollbar,
    ForeignLayer,
}

/// A semantic item containing a PaintRecord range in the flat replay ABI.
/// Save/restore and multiple draw ops inside a recorder count as one item.
#[derive(Clone, Debug, PartialEq)]
pub struct RecordedDisplayItem {
    // Source: display_item.h:260-265; paint_controller.cc:531-545.
    pub is_cacheable: bool,
    pub is_tombstone: bool,
    pub paint_invalidation_reason: Option<foundation::PaintInvalidationReason>,
    pub scrollbar: Option<ScrollbarDisplayItemData>,
    pub foreign_layer: Option<crate::ForeignLayerDisplayItemData>,
    pub kind: RecordedDisplayItemKind,
    pub id: paint_model::display_item_id::DisplayItemId,
    pub visual_rect: PaintRect,
    pub visual_rect_is_accurate: bool,
    pub draws_content: bool,
    pub raster_effect_outset: RasterEffectOutset,
    /// DrawingDisplayItem's immutable PaintRecord analysis. PaintController
    /// moves these values with a cached item instead of rescanning its record.
    pub record_rect_known_to_be_opaque: PaintRect,
    pub record_background: PaintChunkBackgroundColorInfo,
    pub record_has_text: bool,
    pub record_begin: usize,
    pub record_end: usize,
    pub scroll_translation:
        Option<Arc<paint_model::paint_property_tree::TransformPaintPropertyNode>>,
}

impl RecordedDisplayItem {
    /// Source: `display_item.cc:40-66`.
    ///
    /// The resident artifacts are explicit because Rechrom stores a drawing's
    /// PaintRecord in the artifact's flat PaintOp backing instead of in the
    /// heterogeneous DisplayItem allocation itself.
    pub fn EqualsForUnderInvalidation(
        &self,
        artifact: &crate::PaintRecording,
        other: &Self,
        other_artifact: &crate::PaintRecording,
    ) -> bool {
        assert!(!self.IsTombstone());
        if self.id != other.id
            || self.kind != other.kind
            || self.raster_effect_outset != other.raster_effect_outset
            || self.draws_content != other.draws_content
        {
            return false;
        }

        if self.visual_rect != other.visual_rect
            // Change of empty visual rect doesn't matter.
            && self.visual_rect.is_empty()
            && other.visual_rect.is_empty()
            // Visual rect of a DrawingDisplayItem not drawing content doesn't matter.
            && (!self.IsDrawing() || self.draws_content)
        {
            return false;
        }

        match self.kind {
            RecordedDisplayItemKind::Drawing => {
                self.EqualsDrawingRecordFastPath(artifact, other, other_artifact)
            }
            RecordedDisplayItemKind::ForeignLayer => {
                self.EqualsForeignLayerForUnderInvalidation(other)
            }
            RecordedDisplayItemKind::Scrollbar => self.EqualsScrollbarForUnderInvalidation(other),
        }
    }

    // Source: display_item.h:206.
    pub fn GetId(&self) -> paint_model::display_item_id::DisplayItemId {
        self.id
    }

    // Source: display_item.h:208-211.
    pub fn ClientId(&self) -> u64 {
        self.id.client_id
    }

    // Source: display_item.h:213-225.
    pub fn VisualRect(&self) -> PaintRect {
        self.visual_rect
    }

    pub fn GetRasterEffectOutset(&self) -> RasterEffectOutset {
        self.raster_effect_outset
    }

    pub fn GetType(&self) -> paint_model::display_item_id::DisplayItemIdType {
        self.id.r#type
    }

    pub fn Fragment(&self) -> u32 {
        self.id.fragment
    }

    // Source: display_item.h:226.
    pub fn SetFragment(&mut self, fragment: u32) {
        self.id.fragment = fragment;
    }

    // Source: display_item.h:246-258.
    pub fn IsDrawing(&self) -> bool {
        self.GetType().IsDrawingType()
    }

    pub fn IsForeignLayer(&self) -> bool {
        self.GetType().IsForeignLayerType()
    }

    pub fn IsScrollbar(&self) -> bool {
        matches!(
            self.GetType(),
            paint_model::display_item_id::DisplayItemIdType::kScrollbarHorizontal
                | paint_model::display_item_id::DisplayItemIdType::kScrollbarVertical
        )
    }

    // Source: display_item.h:260-268.
    pub fn GetPaintInvalidationReason(&self) -> Option<foundation::PaintInvalidationReason> {
        self.paint_invalidation_reason
    }

    pub fn SetPaintInvalidationReason(&mut self, reason: foundation::PaintInvalidationReason) {
        self.paint_invalidation_reason = Some(reason);
        self.is_cacheable = reason != foundation::PaintInvalidationReason::kUncacheable;
    }

    pub fn IsCacheable(&self) -> bool {
        self.paint_invalidation_reason
            .map_or(self.is_cacheable, |reason| {
                reason != foundation::PaintInvalidationReason::kUncacheable
            })
    }

    // Source: display_item.h:272-276.
    pub fn IsTombstone(&self) -> bool {
        self.is_tombstone
    }

    // Source: display_item.h:317-325. A single-item move keeps identity and
    // invalidation fields available to diagnostics and raster invalidation.
    pub fn CreateTombstone(&mut self) {
        // MoveItem's memcpy transfers subclass ownership without incrementing
        // scoped_refptr counts, and Destruct() intentionally skips the old
        // tombstone. Rust must explicitly release the old owner slot after the
        // destination clone has retained it.
        self.foreign_layer = None;
        self.draws_content = false;
        self.is_tombstone = true;
    }

    // Source: display_item_list.h:225-231. A moved subsequence is never read
    // for raster invalidation, so Blink zeroes every old slot rather than
    // preserving the single-item tombstone fields.
    pub fn CreateSubsequenceTombstone(&mut self) {
        self.is_cacheable = true;
        self.is_tombstone = true;
        self.paint_invalidation_reason = Some(foundation::PaintInvalidationReason::kNone);
        self.scrollbar = None;
        self.foreign_layer = None;
        self.kind = RecordedDisplayItemKind::Drawing;
        self.id = Default::default();
        self.visual_rect = Default::default();
        self.visual_rect_is_accurate = false;
        self.draws_content = false;
        self.raster_effect_outset = RasterEffectOutset::kNone;
        self.record_rect_known_to_be_opaque = Default::default();
        self.record_background = Default::default();
        self.record_has_text = false;
        self.record_begin = 0;
        self.record_end = 0;
        self.scroll_translation = None;
    }

    // Source: display_item.h:278-286; display_item.cc:203-240.
    pub fn IsSubsequenceTombstone(&self) -> bool {
        self.is_tombstone && self.id.client_id == 0
    }

    pub fn IdAsString(&self, paint_artifact: &crate::PaintRecording) -> String {
        if self.IsSubsequenceTombstone() {
            return "SUBSEQUENCE TOMBSTONE".into();
        }
        if self.IsTombstone() {
            return format!("TOMBSTONE {}", paint_artifact.IdAsString(&self.id));
        }
        paint_artifact.IdAsString(&self.id)
    }

    pub fn PropertiesAsJSON(&self, paint_artifact: &crate::PaintRecording) -> serde_json::Value {
        let mut properties = serde_json::Map::new();
        properties.insert(
            "id".into(),
            serde_json::Value::String(self.IdAsString(paint_artifact)),
        );
        if self.IsSubsequenceTombstone() {
            return serde_json::Value::Object(properties);
        }

        properties.insert(
            "invalidation".into(),
            serde_json::Value::String(
                foundation::PaintInvalidationReasonToString(
                    self.paint_invalidation_reason
                        .unwrap_or(foundation::PaintInvalidationReason::kNone),
                )
                .into(),
            ),
        );
        properties.insert(
            "visualRect".into(),
            serde_json::Value::String(format!(
                "{},{} {}x{}",
                self.visual_rect.x,
                self.visual_rect.y,
                self.visual_rect.width,
                self.visual_rect.height
            )),
        );
        match self.raster_effect_outset {
            RasterEffectOutset::kNone => {}
            RasterEffectOutset::kHalfPixel => {
                properties.insert("outset".into(), serde_json::json!(0.5));
            }
            RasterEffectOutset::kWholePixel => {
                properties.insert("outset".into(), serde_json::json!(1));
            }
        }

        if self.IsTombstone() {
            return serde_json::Value::Object(properties);
        }
        if self.kind == RecordedDisplayItemKind::Scrollbar {
            let scroll_translation = self
                .scroll_translation
                .as_ref()
                .map_or_else(|| "0x0".into(), |node| format!("{:p}", Arc::as_ptr(node)));
            properties.insert(
                "scrollTranslation".into(),
                serde_json::Value::String(scroll_translation),
            );
        } else if self.kind == RecordedDisplayItemKind::ForeignLayer {
            properties.insert(
                "layer".into(),
                serde_json::json!(self.GetForeignLayer().LayerId()),
            );
        }
        serde_json::Value::Object(properties)
    }

    pub fn AsDebugString(&self, paint_artifact: &crate::PaintRecording) -> String {
        serde_json::to_string_pretty(&self.PropertiesAsJSON(paint_artifact))
            .expect("DisplayItem debug properties contain only finite source values")
    }

    pub fn DrawsContent(&self) -> bool {
        self.draws_content
    }
}

// Source: display_item.cc:271-273.
impl std::fmt::Display for RecordedDisplayItem {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(output, "{{\"id\": {}}}", self.GetId())
    }
}

#[cfg(test)]
mod recorded_display_item_debug_tests {
    use super::*;
    use paint_model::display_item_id::{DisplayItemId, DisplayItemIdType};

    fn item() -> RecordedDisplayItem {
        RecordedDisplayItem {
            is_cacheable: true,
            is_tombstone: false,
            paint_invalidation_reason: Some(foundation::PaintInvalidationReason::kLayout),
            scrollbar: None,
            foreign_layer: None,
            kind: RecordedDisplayItemKind::Drawing,
            id: DisplayItemId {
                client_id: 0x2a,
                r#type: DisplayItemIdType::kCaret,
                fragment: 3,
            },
            visual_rect: PaintRect {
                x: 1.0,
                y: 2.0,
                width: 30.0,
                height: 40.0,
            },
            visual_rect_is_accurate: true,
            draws_content: true,
            raster_effect_outset: RasterEffectOutset::kHalfPixel,
            record_rect_known_to_be_opaque: PaintRect::default(),
            record_background: PaintChunkBackgroundColorInfo::default(),
            record_has_text: false,
            record_begin: 0,
            record_end: 0,
            scroll_translation: None,
        }
    }

    #[test]
    fn properties_use_artifact_identity_reason_rect_and_outset() {
        let item = item();
        let mut artifact = crate::PaintRecording::default();
        artifact.RecordDebugInfo(0x2a, Arc::<str>::from("caret-client"), 7);
        let properties = item.PropertiesAsJSON(&artifact);
        assert_eq!(properties["id"], "0x2a:caret-client:Caret:3");
        assert_eq!(properties["invalidation"], "geometry");
        assert_eq!(properties["visualRect"], "1,2 30x40");
        assert_eq!(properties["outset"], 0.5);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&item.AsDebugString(&artifact)).unwrap(),
            properties
        );
    }

    #[test]
    fn tombstone_strings_keep_source_distinction() {
        let artifact = crate::PaintRecording::default();
        let mut moved = item();
        moved.is_tombstone = true;
        assert_eq!(moved.IdAsString(&artifact), "TOMBSTONE 0x2a:Caret:3");
        moved.id.client_id = 0;
        assert_eq!(moved.IdAsString(&artifact), "SUBSEQUENCE TOMBSTONE");
        assert_eq!(
            moved.PropertiesAsJSON(&artifact).as_object().unwrap().len(),
            1
        );
    }
}

pub use layout::editing::CaretPaintState as CaretPosition;
pub use layout::editing::EditingPaintState;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CaretGeometry {
    pub node_id: u64,
    pub rect: PaintRect,
    pub visible: bool,
}
