//! Source-shaped owner for Blink's heterogeneous DisplayItemList.
//!
//! Rust's `RecordedDisplayItemKind` replaces C++ placement-new subclasses, but
//! the list remains the sole owner of ordering, ranges, capacity and source
//! tombstone behavior.

use crate::RecordedDisplayItem;
use std::ops::{Deref, DerefMut};

// Source: display_item_list.h:234-250.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DisplayItemListJsonOption {
    #[default]
    kDefault,
    kCompact,
    kShowPaintRecords,
}

// Source: third_party/blink/renderer/platform/graphics/paint/display_item_list.h:23-40,99-143.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DisplayItemList {
    items_: Vec<RecordedDisplayItem>,
}

impl DisplayItemList {
    pub fn ReserveCapacity(&mut self, initial_capacity: usize) {
        self.items_.reserve(initial_capacity);
    }

    pub fn reserve(&mut self, additional: usize) {
        self.items_.reserve(additional);
    }

    pub fn push(&mut self, item: RecordedDisplayItem) {
        self.items_.push(item);
    }

    pub fn clear(&mut self) {
        self.items_.clear();
    }

    pub fn IsEmpty(&self) -> bool {
        self.items_.is_empty()
    }

    pub fn MemoryUsageInBytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.items_.capacity() * std::mem::size_of::<RecordedDisplayItem>()
    }

    // Source: display_item_list.h:145-182. Rust slices are the source range
    // type and keep the exact backing list rather than projecting item copies.
    pub fn ItemsInRange(&self, begin_index: usize, end_index: usize) -> &[RecordedDisplayItem] {
        &self.items_[begin_index..end_index]
    }

    pub fn ItemsInRangeMut(
        &mut self,
        begin_index: usize,
        end_index: usize,
    ) -> &mut [RecordedDisplayItem] {
        &mut self.items_[begin_index..end_index]
    }

    // Source: display_item_list.h:197-207,264-277. The source byte-copies a
    // trivially-copyable item then preserves identity fields in a tombstone.
    // Rust clones owned payloads and applies the same logical state transition.
    pub fn AppendByMoving(&mut self, item: &mut RecordedDisplayItem) -> &mut RecordedDisplayItem {
        assert!(!item.IsTombstone());
        let moved = item.clone();
        item.CreateTombstone();
        self.items_.push(moved);
        self.items_.last_mut().expect("just appended DisplayItem")
    }

    pub fn ReplaceLastByMoving(
        &mut self,
        item: &mut RecordedDisplayItem,
    ) -> &mut RecordedDisplayItem {
        assert!(!item.IsTombstone());
        let moved = item.clone();
        item.CreateTombstone();
        *self
            .items_
            .last_mut()
            .expect("source ReplaceLastByMoving requires a last item") = moved;
        self.items_.last_mut().expect("last item remains present")
    }

    // Source: display_item_list.h:209-232. Unlike a single-item move, Blink
    // clears every old subsequence slot because raster invalidation will not
    // inspect it. CreateSubsequenceTombstone performs that zero-state mapping.
    pub fn AppendSubsequenceByMoving(
        &mut self,
        from: &mut DisplayItemList,
        begin_index: usize,
        end_index: usize,
    ) {
        assert!(end_index >= begin_index);
        if end_index == begin_index {
            return;
        }
        assert!(begin_index < from.len());
        assert!(end_index <= from.len());
        self.items_
            .extend_from_slice(&from.items_[begin_index..end_index]);
        for item in &mut from.items_[begin_index..end_index] {
            item.CreateSubsequenceTombstone();
        }
    }

    // Source: display_item_list.cc:22-37,45-50. Item property formatting is
    // sourced from the resident artifact/id values; kShowPaintRecords remains
    // the same item list plus record payload metadata owned by PaintRecording.
    pub fn DisplayItemsAsJSON(
        artifact: &crate::PaintRecording,
        first_item_index: usize,
        display_items: &[RecordedDisplayItem],
        option: DisplayItemListJsonOption,
    ) -> serde_json::Value {
        let mut values = Vec::with_capacity(display_items.len());
        for (offset, item) in display_items.iter().enumerate() {
            let index = first_item_index + offset;
            if option == DisplayItemListJsonOption::kCompact {
                values.push(serde_json::Value::String(format!(
                    "{}: {}",
                    index,
                    artifact.IdAsString(&item.id)
                )));
                continue;
            }
            let mut object = item
                .PropertiesAsJSON(artifact)
                .as_object()
                .expect("DisplayItem properties are a JSON object")
                .clone();
            object.insert("index".into(), serde_json::json!(index));
            if option == DisplayItemListJsonOption::kShowPaintRecords {
                object.insert(
                    "recordRange".into(),
                    serde_json::json!([item.record_begin, item.record_end]),
                );
            }
            values.push(serde_json::Value::Object(object));
        }
        serde_json::Value::Array(values)
    }
}

impl Deref for DisplayItemList {
    type Target = [RecordedDisplayItem];

    fn deref(&self) -> &Self::Target {
        &self.items_
    }
}

impl DerefMut for DisplayItemList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items_
    }
}

impl From<Vec<RecordedDisplayItem>> for DisplayItemList {
    fn from(items: Vec<RecordedDisplayItem>) -> Self {
        Self { items_: items }
    }
}

impl From<DisplayItemList> for Vec<RecordedDisplayItem> {
    fn from(items: DisplayItemList) -> Self {
        items.items_
    }
}

impl<'a> IntoIterator for &'a DisplayItemList {
    type Item = &'a RecordedDisplayItem;
    type IntoIter = std::slice::Iter<'a, RecordedDisplayItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.items_.iter()
    }
}

impl<'a> IntoIterator for &'a mut DisplayItemList {
    type Item = &'a mut RecordedDisplayItem;
    type IntoIter = std::slice::IterMut<'a, RecordedDisplayItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.items_.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RasterEffectOutset, RecordedDisplayItemKind};

    fn item(client_id: u64) -> RecordedDisplayItem {
        RecordedDisplayItem {
            is_cacheable: true,
            is_tombstone: false,
            paint_invalidation_reason: Some(foundation::PaintInvalidationReason::kNone),
            scrollbar: None,
            foreign_layer: None,
            kind: RecordedDisplayItemKind::Drawing,
            id: paint_model::display_item_id::DisplayItemId {
                client_id,
                ..Default::default()
            },
            visual_rect: Default::default(),
            visual_rect_is_accurate: true,
            draws_content: false,
            raster_effect_outset: RasterEffectOutset::kNone,
            record_rect_known_to_be_opaque: Default::default(),
            record_background: Default::default(),
            record_has_text: false,
            record_begin: 0,
            record_end: 0,
            scroll_translation: None,
        }
    }

    #[test]
    fn source_ranges_borrow_the_same_item_storage() {
        let list = DisplayItemList::from(vec![item(1), item(2), item(3)]);
        let range = list.ItemsInRange(1, 3);
        assert_eq!(range.len(), 2);
        assert!(std::ptr::eq(&range[0], &list[1]));
    }

    #[test]
    fn moving_items_leaves_source_tombstones() {
        let mut source = DisplayItemList::from(vec![item(1), item(2), item(3)]);
        let mut destination = DisplayItemList::default();
        destination.AppendSubsequenceByMoving(&mut source, 1, 3);
        assert_eq!(destination.len(), 2);
        assert_eq!(destination[0].id.client_id, 2);
        for tombstone in &source[1..3] {
            assert!(tombstone.IsSubsequenceTombstone());
            assert_eq!(tombstone.id, Default::default());
            assert!(!tombstone.DrawsContent());
            assert_eq!(tombstone.record_begin, 0);
            assert_eq!(tombstone.record_end, 0);
        }
    }

    #[test]
    fn moving_one_item_preserves_identity_but_clears_draws_content() {
        let mut source = item(9);
        source.draws_content = true;
        let mut destination = DisplayItemList::default();
        destination.AppendByMoving(&mut source);
        assert!(source.IsTombstone());
        assert!(!source.IsSubsequenceTombstone());
        assert_eq!(source.id.client_id, 9);
        assert!(!source.DrawsContent());
        assert!(destination[0].DrawsContent());
    }

    #[test]
    fn compact_json_uses_artifact_aware_ids_and_source_indices() {
        let mut artifact = crate::PaintRecording::default();
        artifact.RecordDebugInfo(2, std::sync::Arc::<str>::from("client"), 0);
        let list = DisplayItemList::from(vec![item(2)]);
        let json = DisplayItemList::DisplayItemsAsJSON(
            &artifact,
            7,
            &list,
            DisplayItemListJsonOption::kCompact,
        );
        assert_eq!(
            json.as_array().unwrap()[0]
                .as_str()
                .unwrap()
                .split(':')
                .next(),
            Some("7")
        );
        assert!(json.as_array().unwrap()[0]
            .as_str()
            .unwrap()
            .contains("client"));
    }
}
