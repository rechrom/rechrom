//! Forced-repaint validation for display-item and subsequence caches.
//!
//! Source entity: Chromium revision 6c1d401fcca5e1b0030563a90c2f2bba168e0c15,
//! `third_party/blink/renderer/platform/graphics/paint/
//! paint_under_invalidation_checker.{h,cc}`.

use foundation::PaintInvalidationReason;
use paint_model::PaintResult;

use crate::paint_controller::paint_controller_debug_data::DebugDataAsStringForArtifacts;
use crate::{PaintRecording, RecordedDisplayItem};

pub type DisplayItemClientId = u64;

/// Source: `paint_controller.h`, `SubsequenceMarkers`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SubsequenceMarkers {
    pub client_id: DisplayItemClientId,
    pub start_chunk_index: usize,
    pub end_chunk_index: usize,
    pub is_moved_from_cached_subsequence: bool,
    /// Chromium keeps this on the persistent PaintLayer. Rechrom's retained
    /// frame boundary owns cached subsequences, so the marker carries the same
    /// previous result across PaintController cycles without crossing the
    /// package boundary with a mutable source PaintLayer pointer.
    pub paint_result: PaintResult,
}

/// Result of checking the item produced by a forced repaint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CheckNewItemResult {
    Matched,
    /// Cache skipping deliberately permits a temporary mismatch. The caller
    /// resumes normal matching at the cached item, as in source lines 67-74.
    CacheSkipping {
        old_item_index: usize,
    },
}

/// Source: `paint_under_invalidation_checker.h:18-75`.
///
/// Chromium stores a reference to PaintController. Rust keeps the state as a
/// controller-owned value and passes the two resident artifacts explicitly,
/// avoiding a self-referential controller while retaining the same ownership
/// and call direction.
#[derive(Debug, Default)]
pub struct PaintUnderInvalidationChecker {
    old_item_index_: Option<usize>,
    old_chunk_index_: Option<usize>,
    new_chunk_index_: Option<usize>,
    subsequence_client_id_: Option<DisplayItemClientId>,
}

impl PaintUnderInvalidationChecker {
    /// Source: `paint_under_invalidation_checker.cc:28-40`.
    pub fn IsChecking(&self) -> bool {
        if self.old_item_index_.is_some() {
            debug_assert!(
                self.subsequence_client_id_.is_none()
                    || (self.old_chunk_index_.is_some() && self.new_chunk_index_.is_some())
            );
            return true;
        }
        debug_assert!(self.subsequence_client_id_.is_none());
        debug_assert!(self.old_chunk_index_.is_none());
        debug_assert!(self.new_chunk_index_.is_none());
        false
    }

    /// Source: `paint_under_invalidation_checker.cc:42-48`.
    fn IsCheckingSubsequence(&self) -> bool {
        if self.subsequence_client_id_.is_some() {
            debug_assert!(self.IsChecking());
            return true;
        }
        false
    }

    pub(crate) fn SubsequenceClientId(&self) -> Option<DisplayItemClientId> {
        self.subsequence_client_id_
    }

    /// Source: `paint_under_invalidation_checker.cc:50-56`.
    fn Stop(&mut self) {
        debug_assert!(self.IsChecking());
        self.old_chunk_index_ = None;
        self.new_chunk_index_ = None;
        self.old_item_index_ = None;
        self.subsequence_client_id_ = None;
    }

    /// Source: `paint_under_invalidation_checker.cc:58-62`.
    pub(crate) fn WouldUseCachedItem(&mut self, old_item_index: usize) {
        debug_assert!(!self.IsChecking());
        self.old_item_index_ = Some(old_item_index);
    }

    /// Source: `paint_under_invalidation_checker.cc:64-102`.
    pub(crate) fn CheckNewItem(
        &mut self,
        old_artifact: &mut PaintRecording,
        new_artifact: &mut PaintRecording,
        is_skipping_cache: bool,
        old_subsequences: &[SubsequenceMarkers],
        new_subsequences: &[SubsequenceMarkers],
    ) -> CheckNewItemResult {
        debug_assert!(self.IsChecking());
        let old_item_index = self.old_item_index_.expect("checking old display item");
        if is_skipping_cache {
            self.Stop();
            return CheckNewItemResult::CacheSkipping { old_item_index };
        }

        let new_item = new_artifact
            .display_items
            .last()
            .expect("forced repaint must append a display item");
        if old_item_index >= old_artifact.display_items.len() {
            self.ShowItemError(
                "extra display item",
                new_artifact,
                old_artifact,
                new_item,
                None,
                old_subsequences,
                new_subsequences,
            );
        }
        let old_item = &old_artifact.display_items[old_item_index];
        if !new_item.EqualsForUnderInvalidation(new_artifact, old_item, old_artifact) {
            self.ShowItemError(
                "display item changed",
                new_artifact,
                old_artifact,
                new_item,
                Some((old_artifact, old_item)),
                old_subsequences,
                new_subsequences,
            );
        }

        // DisplayItem owns its PaintRecord in Blink. Rechrom's resident item
        // owns a range into PaintRecording::items, so preserve the exact source
        // move while relocating that range into the new artifact's backing.
        let old_record = old_artifact.items[old_item.record_begin..old_item.record_end].to_vec();
        let (new_record_begin, new_record_end) = (new_item.record_begin, new_item.record_end);
        debug_assert_eq!(new_record_end, new_artifact.items.len());

        let moved = new_artifact
            .display_items
            .ReplaceLastByMoving(&mut old_artifact.display_items[old_item_index]);
        moved.SetPaintInvalidationReason(if moved.IsCacheable() {
            PaintInvalidationReason::kNone
        } else {
            PaintInvalidationReason::kUncacheable
        });
        let new_items = std::sync::Arc::make_mut(&mut new_artifact.items);
        new_items.truncate(new_record_begin);
        let moved_record_begin = new_items.len();
        new_items.extend(old_record);
        let moved_record_end = new_items.len();
        let moved = new_artifact
            .display_items
            .last_mut()
            .expect("moved cached display item");
        moved.record_begin = moved_record_begin;
        moved.record_end = moved_record_end;

        if self.subsequence_client_id_.is_some() {
            self.old_item_index_ = Some(old_item_index + 1);
        } else {
            self.Stop();
        }
        CheckNewItemResult::Matched
    }

    /// Source: `paint_under_invalidation_checker.cc:104-114`.
    pub fn WouldUseCachedSubsequence(
        &mut self,
        client_id: DisplayItemClientId,
        markers: SubsequenceMarkers,
        old_artifact: &PaintRecording,
        new_artifact: &PaintRecording,
    ) {
        debug_assert!(!self.IsChecking());
        debug_assert_eq!(markers.client_id, client_id);
        let first_chunk = &old_artifact.chunks[markers.start_chunk_index];
        self.old_chunk_index_ = Some(markers.start_chunk_index);
        self.new_chunk_index_ = Some(new_artifact.chunks.len());
        self.old_item_index_ = Some(first_chunk.begin_index as usize);
        self.subsequence_client_id_ = Some(client_id);
    }

    /// Source: `paint_under_invalidation_checker.cc:116-128`.
    pub fn CheckNewChunk(
        &mut self,
        old_artifact: &PaintRecording,
        new_artifact: &PaintRecording,
        markers: SubsequenceMarkers,
        old_subsequences: &[SubsequenceMarkers],
        new_subsequences: &[SubsequenceMarkers],
    ) {
        debug_assert!(self.IsChecking());
        if !self.IsCheckingSubsequence() {
            return;
        }
        let new_chunk_index = self.new_chunk_index_.expect("checking new chunk");
        if new_artifact.chunks.len() > new_chunk_index + 1 {
            self.CheckNewChunkInternal(
                old_artifact,
                new_artifact,
                markers,
                old_subsequences,
                new_subsequences,
            );
        }
    }

    /// Source: `paint_under_invalidation_checker.cc:130-157`.
    pub fn WillEndSubsequence(
        &mut self,
        client_id: DisplayItemClientId,
        start_chunk_index: usize,
        markers: Option<SubsequenceMarkers>,
        active_markers: SubsequenceMarkers,
        old_artifact: &PaintRecording,
        new_artifact: &PaintRecording,
        old_subsequences: &[SubsequenceMarkers],
        new_subsequences: &[SubsequenceMarkers],
    ) {
        debug_assert!(self.IsChecking());
        if !self.IsCheckingSubsequence() {
            return;
        }
        match markers {
            None if start_chunk_index != new_artifact.chunks.len() => {
                self.ShowSubsequenceError(
                    "unexpected subsequence",
                    Some(client_id),
                    new_artifact,
                    old_artifact,
                    None,
                    None,
                    old_subsequences,
                    new_subsequences,
                );
            }
            Some(markers)
                if markers.end_chunk_index - markers.start_chunk_index
                    != new_artifact.chunks.len() - start_chunk_index =>
            {
                self.ShowSubsequenceError(
                    "new subsequence wrong length",
                    Some(client_id),
                    new_artifact,
                    old_artifact,
                    None,
                    None,
                    old_subsequences,
                    new_subsequences,
                );
            }
            Some(_) => {
                let end_chunk_index = new_artifact.chunks.len();
                if self.new_chunk_index_.expect("checking new chunk") < end_chunk_index {
                    debug_assert_eq!(
                        self.new_chunk_index_.expect("checking new chunk") + 1,
                        end_chunk_index
                    );
                    self.CheckNewChunkInternal(
                        old_artifact,
                        new_artifact,
                        active_markers,
                        old_subsequences,
                        new_subsequences,
                    );
                    debug_assert_eq!(self.new_chunk_index_, Some(end_chunk_index));
                }
            }
            None => {}
        }
        if self.subsequence_client_id_ == Some(client_id) {
            self.Stop();
        }
    }

    /// Source: `paint_under_invalidation_checker.cc:159-177`.
    fn CheckNewChunkInternal(
        &mut self,
        old_artifact: &PaintRecording,
        new_artifact: &PaintRecording,
        markers: SubsequenceMarkers,
        old_subsequences: &[SubsequenceMarkers],
        new_subsequences: &[SubsequenceMarkers],
    ) {
        debug_assert_eq!(self.subsequence_client_id_, Some(markers.client_id));
        let new_chunk_index = self.new_chunk_index_.expect("checking new chunk");
        let old_chunk_index = self.old_chunk_index_.expect("checking old chunk");
        let new_chunk = &new_artifact.chunks[new_chunk_index];
        if old_chunk_index >= markers.end_chunk_index {
            self.ShowSubsequenceError(
                "extra chunk",
                None,
                new_artifact,
                old_artifact,
                Some(new_chunk),
                None,
                old_subsequences,
                new_subsequences,
            );
        } else {
            let old_chunk = &old_artifact.chunks[old_chunk_index];
            if !old_chunk.EqualsForUnderInvalidationChecking(new_chunk) {
                self.ShowSubsequenceError(
                    "chunk changed",
                    None,
                    new_artifact,
                    old_artifact,
                    Some(new_chunk),
                    Some((old_artifact, old_chunk)),
                    old_subsequences,
                    new_subsequences,
                );
            }
        }
        self.new_chunk_index_ = Some(new_chunk_index + 1);
        self.old_chunk_index_ = Some(old_chunk_index + 1);
    }

    /// Source: `paint_under_invalidation_checker.cc:179-214`.
    fn ShowItemError(
        &self,
        reason: &str,
        new_artifact: &PaintRecording,
        old_artifact: &PaintRecording,
        new_item: &RecordedDisplayItem,
        old: Option<(&PaintRecording, &RecordedDisplayItem)>,
        old_subsequences: &[SubsequenceMarkers],
        new_subsequences: &[SubsequenceMarkers],
    ) -> ! {
        let subsequence = self.subsequence_client_id_.map_or_else(String::new, |id| {
            format!(
                " (in cached subsequence for {})",
                new_artifact.ClientDebugName(id as usize)
            )
        });
        let old_description = old.map_or_else(String::new, |(artifact, item)| {
            format!("; old display item={}", item.AsDebugString(artifact))
        });
        let debug_data = DebugDataAsStringForArtifacts(
            Some((old_artifact, old_subsequences)),
            Some((new_artifact, new_subsequences)),
            crate::DisplayItemListJsonOption::kDefault,
        );
        panic!(
            "Under-invalidation: {reason}{subsequence}; new display item={}{}; see https://crbug.com/619103.\n{debug_data}",
            new_item.AsDebugString(new_artifact),
            old_description
        );
    }

    /// Source: `paint_under_invalidation_checker.cc:216-249`.
    fn ShowSubsequenceError(
        &self,
        reason: &str,
        client_id: Option<DisplayItemClientId>,
        new_artifact: &PaintRecording,
        old_artifact: &PaintRecording,
        new_chunk: Option<&paint_chunk::PaintChunk>,
        old_chunk: Option<(&PaintRecording, &paint_chunk::PaintChunk)>,
        old_subsequences: &[SubsequenceMarkers],
        new_subsequences: &[SubsequenceMarkers],
    ) -> ! {
        let active = self.subsequence_client_id_.map_or_else(String::new, |id| {
            format!(
                " (in cached subsequence for {})",
                new_artifact.ClientDebugName(id as usize)
            )
        });
        let client = client_id.map_or_else(String::new, |id| {
            format!(
                "; subsequence client={}",
                new_artifact.ClientDebugName(id as usize)
            )
        });
        let new_chunk = new_chunk.map_or_else(String::new, |chunk| {
            format!(
                "; new paint chunk={}",
                new_artifact.PaintChunkToString(chunk, false)
            )
        });
        let old_chunk_description = old_chunk.map_or_else(String::new, |(artifact, chunk)| {
            format!(
                "; old paint chunk={}",
                artifact.PaintChunkToString(chunk, false)
            )
        });
        let debug_data = DebugDataAsStringForArtifacts(
            Some((old_artifact, old_subsequences)),
            Some((new_artifact, new_subsequences)),
            crate::DisplayItemListJsonOption::kDefault,
        );
        panic!(
            "Under-invalidation: {reason}{active}{client}{new_chunk}{old_chunk_description}; see https://crbug.com/619103.\n{debug_data}"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        DisplayItem, DisplayItemList, DisplayItemType, RasterEffectOutset, RecordedDisplayItemKind,
    };
    use paint_chunk::PaintChunk;
    use paint_model::display_item_id::{DisplayItemId, DisplayItemIdType};
    use paint_model::PaintRect;
    use std::sync::Arc;

    fn item(client_id: u64, begin: usize, end: usize) -> RecordedDisplayItem {
        RecordedDisplayItem {
            is_cacheable: true,
            is_tombstone: false,
            paint_invalidation_reason: Some(PaintInvalidationReason::kNone),
            scrollbar: None,
            foreign_layer: None,
            kind: RecordedDisplayItemKind::Drawing,
            id: DisplayItemId {
                client_id,
                r#type: DisplayItemIdType::kBoxDecorationBackground,
                fragment: 0,
            },
            visual_rect: PaintRect {
                width: 10.0,
                height: 10.0,
                ..Default::default()
            },
            visual_rect_is_accurate: true,
            draws_content: begin != end,
            raster_effect_outset: RasterEffectOutset::kNone,
            record_rect_known_to_be_opaque: PaintRect::default(),
            record_background: Default::default(),
            record_has_text: false,
            record_begin: begin,
            record_end: end,
            scroll_translation: None,
        }
    }

    fn op(x: f64) -> DisplayItem {
        DisplayItem {
            r#type: DisplayItemType::kDrawRect,
            rect: PaintRect {
                x,
                width: 10.0,
                height: 10.0,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn chunk(client_id: u64, begin: u32, end: u32) -> PaintChunk {
        PaintChunk {
            id: DisplayItemId {
                client_id,
                r#type: DisplayItemIdType::kBoxDecorationBackground,
                fragment: 0,
            },
            is_cacheable: true,
            begin_index: begin,
            end_index: end,
            ..Default::default()
        }
    }

    #[test]
    fn matching_forced_repaint_is_discarded_and_cached_item_is_moved() {
        let mut old = PaintRecording {
            items: Arc::new(vec![op(1.0)]),
            display_items: DisplayItemList::from(vec![item(7, 0, 1)]).into(),
            ..Default::default()
        };
        let mut repainted = item(7, 0, 1);
        repainted.paint_invalidation_reason = Some(PaintInvalidationReason::kLayout);
        let mut new = PaintRecording {
            items: Arc::new(vec![op(1.0)]),
            display_items: DisplayItemList::from(vec![repainted]).into(),
            ..Default::default()
        };
        let mut checker = PaintUnderInvalidationChecker::default();
        checker.WouldUseCachedItem(0);

        assert_eq!(
            checker.CheckNewItem(&mut old, &mut new, false, &[], &[]),
            CheckNewItemResult::Matched
        );
        assert!(!checker.IsChecking());
        assert!(old.display_items[0].IsTombstone());
        assert_eq!(
            new.display_items[0].GetPaintInvalidationReason(),
            Some(PaintInvalidationReason::kNone)
        );
        assert!(new.display_items[0].GetPaintRecord(&new) == [op(1.0)]);
    }

    #[test]
    fn changed_forced_repaint_reports_under_invalidation() {
        let mut old = PaintRecording {
            items: Arc::new(vec![op(1.0)]),
            display_items: DisplayItemList::from(vec![item(7, 0, 1)]).into(),
            ..Default::default()
        };
        let mut new = PaintRecording {
            items: Arc::new(vec![op(2.0)]),
            display_items: DisplayItemList::from(vec![item(7, 0, 1)]).into(),
            ..Default::default()
        };
        let mut checker = PaintUnderInvalidationChecker::default();
        checker.WouldUseCachedItem(0);

        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            checker.CheckNewItem(&mut old, &mut new, false, &[], &[]);
        }))
        .unwrap_err();
        let message = panic
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| panic.downcast_ref::<&str>().copied())
            .expect("checker panic contains a string diagnostic");
        assert!(message.contains("current paint artifact: []"));
        assert!(message.contains("new paint artifact: []"));
        assert!(!old.display_items[0].IsTombstone());
    }

    #[test]
    fn cache_skipping_stops_checking_and_returns_the_old_match_cursor() {
        let mut old = PaintRecording {
            display_items: DisplayItemList::from(vec![item(7, 0, 0), item(8, 0, 0)]).into(),
            ..Default::default()
        };
        let mut new = PaintRecording {
            display_items: DisplayItemList::from(vec![item(99, 0, 0)]).into(),
            ..Default::default()
        };
        let mut checker = PaintUnderInvalidationChecker::default();
        checker.WouldUseCachedItem(1);

        assert_eq!(
            checker.CheckNewItem(&mut old, &mut new, true, &[], &[]),
            CheckNewItemResult::CacheSkipping { old_item_index: 1 }
        );
        assert!(!checker.IsChecking());
        assert!(!old.display_items[1].IsTombstone());
    }

    #[test]
    fn cached_subsequence_checks_each_item_and_each_completed_chunk() {
        let markers = SubsequenceMarkers {
            client_id: 7,
            start_chunk_index: 0,
            end_chunk_index: 2,
            is_moved_from_cached_subsequence: false,
            paint_result: PaintResult::kFullyPainted,
        };
        let mut old = PaintRecording {
            display_items: DisplayItemList::from(vec![item(7, 0, 0), item(8, 0, 0)]).into(),
            chunks: vec![chunk(7, 0, 1), chunk(8, 1, 2)].into(),
            ..Default::default()
        };
        let mut new = PaintRecording::default();
        let mut checker = PaintUnderInvalidationChecker::default();
        checker.WouldUseCachedSubsequence(7, markers, &old, &new);

        new.chunks.push(chunk(7, 0, 1));
        new.display_items.push(item(7, 0, 0));
        checker.CheckNewItem(&mut old, &mut new, false, &[], &[]);

        new.chunks.push(chunk(8, 1, 2));
        checker.CheckNewChunk(&old, &new, markers, &[], &[]);
        new.display_items.push(item(8, 0, 0));
        checker.CheckNewItem(&mut old, &mut new, false, &[], &[]);
        checker.WillEndSubsequence(7, 0, Some(markers), markers, &old, &new, &[], &[]);

        assert!(!checker.IsChecking());
        assert!(old
            .display_items
            .iter()
            .all(RecordedDisplayItem::IsTombstone));
        assert_eq!(new.display_items.len(), 2);
    }
}
