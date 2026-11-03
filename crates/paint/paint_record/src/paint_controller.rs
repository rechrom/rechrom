//! Display-item cache state for one paint cycle.
//!
//! Source entity: Chromium revision 6c1d401fcca5e1b0030563a90c2f2bba168e0c15,
//! `third_party/blink/renderer/platform/graphics/paint/paint_controller.{h,cc}`.
//! Recording remains in `PaintContext`; this type owns the source controller's
//! PaintChunker, old artifact lookup, cache admission metrics, and end-of-cycle
//! validation.

use std::collections::HashSet;
use std::sync::Arc;

use foundation::TrustedIdentityHashMap as HashMap;
use layoutng_assembly::fragment_tree::RasterEffectOutset;
use paint_chunk::{
    DisplayItemClientInfo, PaintChunkBackgroundColorInfo, PaintChunker, PaintHitTestData,
    PaintHitTestOpaqueness, PaintSelectionBound, PaintTrackedElementSubRect,
};
use paint_model::display_item_id::DisplayItemId;
use paint_model::paint_property_tree::PropertyTreeState;
use paint_model::{PaintRect, PaintResult};

use crate::paint_under_invalidation_checker::{CheckNewItemResult, PaintUnderInvalidationChecker};
use crate::{
    DisplayItem, PaintRecording, RecordedDisplayItem, RecordedDisplayItemKind, SubsequenceMarkers,
};

#[path = "paint_controller_debug_data.rs"]
pub(crate) mod paint_controller_debug_data;

pub(crate) struct CachedDrawing {
    pub item: RecordedDisplayItem,
    pub record: Vec<DisplayItem>,
}

/// Source: `paint_controller.h:66-82`.
#[derive(Clone, Debug, Default)]
pub struct SubsequencesData {
    map: HashMap<u64, usize>,
    tree: Vec<SubsequenceMarkers>,
}

impl SubsequencesData {
    pub fn GetSubsequenceIndex(&self, client_id: u64) -> Option<usize> {
        self.map.get(&client_id).copied()
    }

    pub fn GetSubsequenceMarkers(&self, client_id: u64) -> Option<&SubsequenceMarkers> {
        self.GetSubsequenceIndex(client_id)
            .and_then(|index| self.tree.get(index))
    }
}

/// Retained paint data shared by successive PaintController cycles.
///
/// Source: `paint_controller.h:86-125`; `paint_controller.cc:31-91`.
/// Chromium stores a GC-owned PaintArtifact here. Rechrom publishes the same
/// retained value as an immutable `Arc`, so an older compositor snapshot can
/// remain alive while the next paint cycle reads the current artifact as its
/// cache source.
#[derive(Default)]
pub struct PaintControllerPersistentData {
    current_paint_artifact: Option<Arc<PaintRecording>>,
    current_subsequences: SubsequencesData,
}

impl PaintControllerPersistentData {
    /// Source: `paint_controller.h:96-105`.
    pub fn GetPaintArtifact(&self) -> Option<&Arc<PaintRecording>> {
        self.current_paint_artifact.as_ref()
    }

    /// Source: `paint_controller.h:111-112`.
    pub fn GetSubsequences(&self) -> &SubsequencesData {
        &self.current_subsequences
    }

    /// Mutable access is confined to the Paint facade so resource-only COW
    /// updates preserve the persistent artifact owner.
    #[doc(hidden)]
    pub fn GetPaintArtifactMut(&mut self) -> Option<&mut Arc<PaintRecording>> {
        self.current_paint_artifact.as_mut()
    }

    /// Source: `paint_controller.h:117-124`; `paint_controller.cc:84-91`.
    /// The commit package freezes the recording before it enters this owner.
    pub fn CommitNewDisplayItems(&mut self, artifact: Arc<PaintRecording>) -> &Arc<PaintRecording> {
        self.CommitNewDisplayItemsWithSubsequences(artifact, SubsequencesData::default())
    }

    /// Source: `paint_controller.h:117-122`; `paint_controller.cc:84-91`.
    pub fn CommitNewDisplayItemsWithSubsequences(
        &mut self,
        artifact: Arc<PaintRecording>,
        subsequences: SubsequencesData,
    ) -> &Arc<PaintRecording> {
        self.current_paint_artifact = Some(artifact);
        self.current_subsequences = subsequences;
        self.current_paint_artifact
            .as_ref()
            .expect("committed paint artifact")
    }

    /// Adopt Page's current immutable reference before a retained update. This
    /// is the package-boundary equivalent of Chromium retaining its current
    /// PaintArtifact in PaintControllerPersistentData.
    #[doc(hidden)]
    pub fn AdoptPaintArtifact(&mut self, artifact: Arc<PaintRecording>) {
        self.current_paint_artifact = Some(artifact);
    }
}

#[derive(Default)]
pub struct PaintController {
    paint_chunker: PaintChunker,
    previous: Option<Arc<PaintRecording>>,
    previous_subsequences: SubsequencesData,
    new_subsequences: SubsequencesData,
    out_of_order_item_id_index_map: HashMap<DisplayItemId, usize>,
    next_item_to_match: usize,
    next_item_to_index: usize,
    under_invalidation_checker: PaintUnderInvalidationChecker,
    metrics: paint_reuse::PaintReuseMetrics,
}

impl PaintController {
    pub(crate) fn WithPrevious(previous: Option<Arc<PaintRecording>>) -> Self {
        Self::WithPreviousAndSubsequences(previous, SubsequencesData::default())
    }

    pub(crate) fn WithPreviousAndSubsequences(
        previous: Option<Arc<PaintRecording>>,
        previous_subsequences: SubsequencesData,
    ) -> Self {
        let candidates = previous
            .as_ref()
            .map_or(0, |artifact| artifact.display_items.len());
        Self {
            paint_chunker: PaintChunker::default(),
            previous,
            previous_subsequences,
            new_subsequences: SubsequencesData::default(),
            out_of_order_item_id_index_map: HashMap::default(),
            next_item_to_match: 0,
            next_item_to_index: 0,
            under_invalidation_checker: PaintUnderInvalidationChecker::default(),
            metrics: paint_reuse::PaintReuseMetrics {
                candidates,
                ..Default::default()
            },
        }
    }

    /// Source: `paint_controller.h:154-156`.
    pub(crate) fn CurrentPaintChunkProperties(&self) -> &PropertyTreeState {
        self.paint_chunker.current_properties()
    }

    /// Source: `paint_controller.cc:627-630`.
    pub(crate) fn UpdateCurrentPaintChunkProperties(&mut self, properties: PropertyTreeState) {
        self.paint_chunker.update_properties(properties);
    }

    /// Source: `paint_controller.cc:614-625`.
    pub(crate) fn UpdateCurrentPaintChunkIdAndProperties(
        &mut self,
        id: DisplayItemId,
        client: DisplayItemClientInfo,
        properties: PropertyTreeState,
    ) {
        self.paint_chunker
            .update_id_and_properties(id, client, properties);
    }

    /// Source: `paint_controller.h:158-163`.
    pub(crate) fn SetCurrentEffectivelyInvisible(&mut self, invisible: bool) {
        self.paint_chunker
            .set_current_effectively_invisible(invisible);
    }

    /// Source: `paint_controller.h:161-163`.
    pub(crate) fn CurrentEffectivelyInvisible(&self) -> bool {
        self.paint_chunker.current_effectively_invisible()
    }

    /// Source: `paint_controller.cc:154-157`.
    pub(crate) fn EnsureChunk(&mut self, output: &mut PaintRecording) {
        if self.paint_chunker.EnsureChunk(&mut output.chunks) {
            self.CheckNewChunk(output);
        }
    }

    /// Source: `paint_controller.h:166-168`.
    pub(crate) fn CurrentChunkIsNonEmptyAndTransparentToHitTest(
        &self,
        output: &PaintRecording,
    ) -> bool {
        self.paint_chunker
            .CurrentChunkIsNonEmptyAndTransparentToHitTest(&output.chunks)
    }

    /// Source: `paint_controller.h:200-208`.
    pub(crate) fn NumNewChunks(&self, output: &PaintRecording) -> usize {
        output.chunks.len()
    }

    /// Source: `paint_controller.h:203-208`.
    pub(crate) fn LastChunk<'a>(
        &self,
        output: &'a PaintRecording,
    ) -> Option<&'a paint_chunk::PaintChunk> {
        output.chunks.last()
    }

    /// Source: `paint_chunker.cc:15-16`; owned by PaintController at
    /// `paint_controller.h:411`.
    pub(crate) fn Finish(&mut self, output: &mut PaintRecording) -> SubsequencesData {
        self.paint_chunker.Finish(&mut output.chunks);
        debug_assert!(!self.under_invalidation_checker.IsChecking());
        std::mem::take(&mut self.new_subsequences)
    }

    /// Source: `paint_controller.cc:462-545` and `paint_chunker.cc:91-150`.
    pub(crate) fn ProcessNewItem(
        &mut self,
        output: &mut PaintRecording,
        id: DisplayItemId,
        client: DisplayItemClientInfo,
        visual_rect: PaintRect,
        draws_content: bool,
        raster_effect_outset: RasterEffectOutset,
        visual_rect_is_accurate: bool,
        rect_known_to_be_opaque: PaintRect,
        background: PaintChunkBackgroundColorInfo,
        has_text: bool,
    ) {
        let created = self.paint_chunker.increment_display_item(
            &mut output.chunks,
            id,
            client,
            visual_rect,
            draws_content,
            raster_effect_outset,
            visual_rect_is_accurate,
            rect_known_to_be_opaque,
        );
        if created {
            self.CheckNewChunk(output);
        }
        if draws_content {
            self.paint_chunker.UpdateDrawingDerivedData(
                &mut output.chunks,
                background,
                has_text,
                visual_rect,
            );
        }
    }

    /// Source: `paint_controller.cc:159-174`.
    pub(crate) fn RecordHitTestData(
        &mut self,
        output: &mut PaintRecording,
        id: DisplayItemId,
        client: DisplayItemClientInfo,
        rect: PaintRect,
        touch_action: foundation::TouchAction,
        blocking_wheel: bool,
        opaqueness: PaintHitTestOpaqueness,
    ) {
        if rect.is_empty() {
            return;
        }
        let created = self.paint_chunker.AddHitTestDataToCurrentChunk(
            &mut output.chunks,
            id,
            client,
            rect,
            touch_action,
            blocking_wheel,
            opaqueness,
        );
        if created {
            self.CheckNewChunk(output);
        }
    }

    /// Source: `paint_controller.cc:176-188`.
    pub(crate) fn RecordRegionCaptureData(
        &mut self,
        output: &mut PaintRecording,
        id: DisplayItemId,
        client: DisplayItemClientInfo,
        crop_id: [u64; 2],
        rect: PaintRect,
    ) {
        let created = self.paint_chunker.AddRegionCaptureDataToCurrentChunk(
            &mut output.chunks,
            id,
            client,
            crop_id,
            rect,
        );
        if created {
            self.CheckNewChunk(output);
        }
    }

    /// Source: `paint_controller.cc:190-202`.
    pub(crate) fn RecordTrackedElementData(
        &mut self,
        output: &mut PaintRecording,
        id: DisplayItemId,
        client: DisplayItemClientInfo,
        element_paint_rect: PaintRect,
        sub_rects: &[PaintTrackedElementSubRect],
    ) {
        let created = self.paint_chunker.AddTrackedElementDataToCurrentChunk(
            &mut output.chunks,
            id,
            client,
            element_paint_rect,
            sub_rects,
        );
        if created {
            self.CheckNewChunk(output);
        }
    }

    /// Source: `paint_controller.cc:204-218`.
    pub(crate) fn RecordScrollHitTestData(
        &mut self,
        output: &mut PaintRecording,
        id: DisplayItemId,
        client: DisplayItemClientInfo,
        rect: PaintRect,
        opaqueness: PaintHitTestOpaqueness,
        data: PaintHitTestData,
    ) {
        self.paint_chunker.CreateScrollHitTestChunk(
            &mut output.chunks,
            id,
            client,
            rect,
            opaqueness,
            data,
        );
        self.CheckNewChunk(output);
    }

    /// Source: `paint_controller.cc:220-226`.
    pub(crate) fn RecordSelection(
        &mut self,
        output: &mut PaintRecording,
        start: Option<PaintSelectionBound>,
        end: Option<PaintSelectionBound>,
    ) {
        debug_assert!(start.is_some() || end.is_some());
        self.paint_chunker
            .AddSelectionToCurrentChunk(&mut output.chunks, start, end);
    }

    /// Source: `paint_controller.h:196-198`.
    pub(crate) fn RecordAnySelectionWasPainted(&mut self, output: &mut PaintRecording) {
        self.paint_chunker
            .RecordAnySelectionWasPainted(&mut output.chunks);
    }

    /// Source: `paint_controller.cc:111-124`.
    pub(crate) fn PrepareNewArtifact(&self, output: &mut PaintRecording) {
        const DEFAULT_DISPLAY_ITEM_LIST_CAPACITY: usize = 16;
        let display_items =
            self.previous
                .as_ref()
                .map_or(DEFAULT_DISPLAY_ITEM_LIST_CAPACITY, |previous| {
                    let current_size = previous.display_items.len();
                    if current_size == 0 {
                        DEFAULT_DISPLAY_ITEM_LIST_CAPACITY
                    } else {
                        current_size
                    }
                });
        output.display_items.reserve(display_items);
        if let Some(previous) = &self.previous {
            output.chunks.reserve(previous.chunks.len());
            // Rechrom's flat PaintOp backing is the PaintRecord payload owned
            // by each DrawingDisplayItem. Preserve its allocation alongside
            // the source DisplayItemList/paint-chunk reservations.
            Arc::make_mut(&mut output.items).reserve(previous.items.len());
        }
    }

    fn FindItemFromIdIndexMap(&self, id: DisplayItemId) -> Option<usize> {
        let index = *self.out_of_order_item_id_index_map.get(&id)?;
        let previous = self.previous.as_ref()?;
        let item = &previous.display_items[index];
        if item.is_tombstone {
            return None;
        }
        debug_assert_eq!(item.id, id);
        debug_assert!(item.is_cacheable);
        Some(index)
    }

    fn AddToIdIndexMap(&mut self, id: DisplayItemId, index: usize) {
        let replaced = self.out_of_order_item_id_index_map.insert(id, index);
        debug_assert!(replaced.is_none(), "duplicate cached DisplayItem::Id");
    }

    fn FindOutOfOrderCachedItemForward(&mut self, id: DisplayItemId) -> Option<usize> {
        let item_count = self.previous.as_ref()?.display_items.len();
        for index in self.next_item_to_index..item_count {
            let (item_id, is_tombstone, is_cacheable) = {
                let item = &self.previous.as_ref().unwrap().display_items[index];
                (item.id, item.is_tombstone, item.is_cacheable)
            };
            if is_tombstone {
                continue;
            }
            if item_id == id {
                return Some(index);
            }
            if is_cacheable {
                self.AddToIdIndexMap(item_id, index);
                self.next_item_to_index = index + 1;
            }
        }
        None
    }

    /// Source: `paint_controller.cc:640-735`.
    fn FindCachedItem(&mut self, id: DisplayItemId) -> Option<usize> {
        if let Some(previous) = self.previous.as_ref() {
            if self.next_item_to_match < previous.display_items.len() {
                let item = &previous.display_items[self.next_item_to_match];
                if !item.is_tombstone && item.id == id {
                    return Some(self.next_item_to_match);
                }
            }
        }
        if let Some(index) = self.FindItemFromIdIndexMap(id) {
            return Some(index);
        }
        self.FindOutOfOrderCachedItemForward(id)
    }

    /// Source: `paint_controller.cc:228-289`.
    ///
    /// PaintController::UseCachedItemIfPossible admission and old-item lookup.
    /// PaintContext performs the actual record/chunk append after this returns
    /// the resident old artifact and display-item index.
    pub(crate) fn TakeCachedDrawing(
        &mut self,
        id: DisplayItemId,
        client: DisplayItemClientInfo,
        check_under_invalidation: bool,
    ) -> Option<CachedDrawing> {
        self.metrics.attempts += 1;
        if self.previous.is_none() {
            self.metrics.rejected_missing_previous += 1;
            return None;
        }
        if client.is_just_created {
            self.metrics.rejected_just_created += 1;
            return None;
        }
        if !client.is_cacheable {
            self.metrics.rejected_uncacheable += 1;
            return None;
        }
        if check_under_invalidation && self.under_invalidation_checker.IsChecking() {
            return None;
        }
        let Some(index) = self.FindCachedItem(id) else {
            self.metrics.rejected_missing_id += 1;
            return None;
        };
        self.next_item_to_match = index + 1;
        if self.next_item_to_match > self.next_item_to_index {
            self.next_item_to_index = self.next_item_to_match;
        }
        let invalidated = !matches!(
            client.invalidation_reason,
            Some(foundation::PaintInvalidationReason::kNone)
        );
        if invalidated {
            self.metrics.rejected_invalid += 1;
            return None;
        }
        let previous = self.previous.as_ref().unwrap();
        debug_assert_eq!(
            previous.display_items[index].kind,
            RecordedDisplayItemKind::Drawing
        );
        self.metrics.reused += 1;
        if check_under_invalidation {
            self.under_invalidation_checker.WouldUseCachedItem(index);
            return None;
        }

        // Source: paint_controller.cc:283-286. The controller's private old
        // artifact is made unique before leaving the real tombstone. Other
        // immutable frame snapshots keep their Arc-owned value unchanged.
        let previous = Arc::make_mut(self.previous.as_mut().unwrap());
        let old = &previous.display_items[index];
        let record = previous.items[old.record_begin..old.record_end].to_vec();
        let item = old.clone();
        previous.display_items[index].CreateTombstone();
        Some(CachedDrawing { item, record })
    }

    /// Source: `paint_controller.cc:337-409`.
    pub(crate) fn UseCachedSubsequenceIfPossible(
        &mut self,
        output: &mut PaintRecording,
        client_id: u64,
        client: DisplayItemClientInfo,
        check_under_invalidation: bool,
    ) -> Option<PaintResult> {
        if self.previous.is_none() {
            return None;
        }
        if check_under_invalidation && self.under_invalidation_checker.IsChecking() {
            return None;
        }
        if client.is_just_created || !client.is_cacheable {
            return None;
        }
        let Some(subsequence_index) = self.previous_subsequences.GetSubsequenceIndex(client_id)
        else {
            return None;
        };
        let markers = self.previous_subsequences.tree[subsequence_index];
        debug_assert_eq!(markers.client_id, client_id);
        let previous = self.previous.as_ref().unwrap();
        let start_item_index = previous.chunks[markers.start_chunk_index].begin_index as usize;
        let end_item_index = previous.chunks[markers.end_chunk_index - 1].end_index as usize;
        if end_item_index > start_item_index
            && previous.display_items[start_item_index].IsTombstone()
        {
            return None;
        }
        if self.next_item_to_match == start_item_index {
            self.next_item_to_match = end_item_index;
            if self.next_item_to_match > self.next_item_to_index {
                self.next_item_to_index = self.next_item_to_match;
            }
        }
        if !matches!(
            client.invalidation_reason,
            Some(foundation::PaintInvalidationReason::kNone)
        ) {
            return None;
        }
        self.metrics.reused += end_item_index - start_item_index;
        if check_under_invalidation {
            self.under_invalidation_checker
                .WouldUseCachedSubsequence(client_id, markers, previous, output);
            return None;
        }
        self.AppendSubsequenceByMoving(
            output,
            subsequence_index,
            markers.start_chunk_index,
            markers.end_chunk_index,
        );
        Some(markers.paint_result)
    }

    /// Source: `paint_controller.cc:411-418`.
    pub(crate) fn BeginSubsequence(&mut self, client_id: u64, output: &PaintRecording) -> usize {
        self.paint_chunker.force_new_chunk();
        self.new_subsequences.tree.push(SubsequenceMarkers {
            client_id,
            start_chunk_index: output.chunks.len(),
            end_chunk_index: 0,
            is_moved_from_cached_subsequence: false,
            paint_result: PaintResult::kFullyPainted,
        });
        self.new_subsequences.tree.len() - 1
    }

    /// Source: `paint_controller.cc:420-460`.
    pub(crate) fn EndSubsequence(&mut self, subsequence_index: usize, output: &PaintRecording) {
        let (client_id, start_chunk_index) = {
            let markers = self
                .new_subsequences
                .tree
                .get(subsequence_index)
                .expect("matching BeginSubsequence");
            (markers.client_id, markers.start_chunk_index)
        };
        if let Some(active_client_id) = self.under_invalidation_checker.SubsequenceClientId() {
            let active_markers = *self
                .previous_subsequences
                .GetSubsequenceMarkers(active_client_id)
                .expect("active cached subsequence markers");
            let markers = self
                .previous_subsequences
                .GetSubsequenceMarkers(client_id)
                .copied();
            self.under_invalidation_checker.WillEndSubsequence(
                client_id,
                start_chunk_index,
                markers,
                active_markers,
                self.previous.as_ref().expect("old subsequence artifact"),
                output,
                &self.previous_subsequences.tree,
                &self.new_subsequences.tree,
            );
        }

        let end_chunk_index = output.chunks.len();
        if start_chunk_index == end_chunk_index {
            debug_assert_eq!(subsequence_index + 1, self.new_subsequences.tree.len());
            self.new_subsequences.tree.pop();
            return;
        }
        self.paint_chunker.force_new_chunk();
        let replaced = self
            .new_subsequences
            .map
            .insert(client_id, subsequence_index);
        assert!(replaced.is_none(), "multiple subsequences for one client");
        self.new_subsequences.tree[subsequence_index].end_chunk_index = end_chunk_index;
    }

    /// PaintLayerPainter stores the result before SubsequenceRecorder closes
    /// the marker, corresponding to PaintLayer::SetPreviousPaintResult().
    pub(crate) fn SetSubsequencePaintResult(
        &mut self,
        subsequence_index: usize,
        result: PaintResult,
    ) {
        self.new_subsequences.tree[subsequence_index].paint_result = result;
    }

    /// Source: `paint_controller.cc:737-820`.
    fn AppendSubsequenceByMoving(
        &mut self,
        output: &mut PaintRecording,
        subsequence_index: usize,
        start_chunk_index: usize,
        end_chunk_index: usize,
    ) {
        debug_assert!(end_chunk_index > start_chunk_index);
        let new_start_chunk_index = output.chunks.len();
        let client_id = self.previous_subsequences.tree[subsequence_index].client_id;
        let new_subsequence_index = self.BeginSubsequence(client_id, output);
        let (start_item_index, end_item_index) = {
            let previous = self.previous.as_ref().unwrap();
            (
                previous.chunks[start_chunk_index].begin_index as usize,
                previous.chunks[end_chunk_index - 1].end_index as usize,
            )
        };

        let previous = Arc::make_mut(self.previous.as_mut().unwrap());
        let records: Vec<Vec<DisplayItem>> = previous.display_items
            [start_item_index..end_item_index]
            .iter()
            .map(|item| previous.items[item.record_begin..item.record_end].to_vec())
            .collect();
        for chunk_index in start_chunk_index..end_chunk_index {
            let chunk = std::mem::take(&mut previous.chunks[chunk_index]);
            self.paint_chunker.AppendByMoving(&mut output.chunks, chunk);
        }

        let new_item_start_index = output.display_items.len();
        output.display_items.AppendSubsequenceByMoving(
            &mut previous.display_items,
            start_item_index,
            end_item_index,
        );
        for (offset, record) in records.into_iter().enumerate() {
            let item_index = new_item_start_index + offset;
            let record_begin = output.items.len();
            Arc::make_mut(&mut output.items).extend(record);
            let record_end = output.items.len();
            let client_id = {
                let item = &mut output.display_items[item_index];
                item.record_begin = record_begin;
                item.record_end = record_end;
                item.SetPaintInvalidationReason(if item.IsCacheable() {
                    foundation::PaintInvalidationReason::kNone
                } else {
                    foundation::PaintInvalidationReason::kUncacheable
                });
                item.id.client_id
            };
            if let Some(name) = previous.client_debug_names.get(&(client_id as usize)) {
                output.RecordDebugInfo(
                    client_id as usize,
                    Arc::clone(name),
                    previous.ClientOwnerNodeId(client_id as usize),
                );
            }
        }

        for old_markers in self
            .previous_subsequences
            .tree
            .iter()
            .skip(subsequence_index + 1)
            .copied()
            .take_while(|markers| markers.start_chunk_index < end_chunk_index)
        {
            let index = self.new_subsequences.tree.len();
            let markers = SubsequenceMarkers {
                client_id: old_markers.client_id,
                start_chunk_index: old_markers.start_chunk_index + new_start_chunk_index
                    - start_chunk_index,
                end_chunk_index: old_markers.end_chunk_index + new_start_chunk_index
                    - start_chunk_index,
                is_moved_from_cached_subsequence: true,
                paint_result: old_markers.paint_result,
            };
            assert!(self
                .new_subsequences
                .map
                .insert(markers.client_id, index)
                .is_none());
            self.new_subsequences.tree.push(markers);
        }
        self.EndSubsequence(new_subsequence_index, output);
        self.new_subsequences.tree[new_subsequence_index].is_moved_from_cached_subsequence = true;
        self.new_subsequences.tree[new_subsequence_index].paint_result =
            self.previous_subsequences.tree[subsequence_index].paint_result;
    }

    /// Source: `paint_controller.cc:594-607`.
    fn CheckNewChunk(&mut self, output: &PaintRecording) {
        let Some(client_id) = self.under_invalidation_checker.SubsequenceClientId() else {
            return;
        };
        let markers = *self
            .previous_subsequences
            .GetSubsequenceMarkers(client_id)
            .expect("cached subsequence markers");
        self.under_invalidation_checker.CheckNewChunk(
            self.previous.as_ref().expect("old subsequence artifact"),
            output,
            markers,
            &self.previous_subsequences.tree,
            &self.new_subsequences.tree,
        );
    }

    /// Source: `paint_controller.cc:462-526,557`.
    pub(crate) fn CheckNewItem(&mut self, output: &mut PaintRecording, is_skipping_cache: bool) {
        if !self.under_invalidation_checker.IsChecking() {
            return;
        }
        let old_subsequences = &self.previous_subsequences.tree;
        let new_subsequences = &self.new_subsequences.tree;
        let previous = Arc::make_mut(
            self.previous
                .as_mut()
                .expect("under-invalidation checking requires an old artifact"),
        );
        if let CheckNewItemResult::CacheSkipping { old_item_index } =
            self.under_invalidation_checker.CheckNewItem(
                previous,
                output,
                is_skipping_cache,
                old_subsequences,
                new_subsequences,
            )
        {
            self.next_item_to_match = old_item_index;
            self.next_item_to_index = old_item_index;
        }
    }

    pub fn Metrics(&self) -> paint_reuse::PaintReuseMetrics {
        self.metrics
    }

    /// Source: `paint_controller.cc:136-143`.
    ///
    /// Chromium DisplayItemClients are validated by Layout's reciprocal commit
    /// adapter after this returns the exact used-client receipt.
    pub fn ValidateForCommittedPaint(artifact: &mut PaintRecording) -> HashSet<u64> {
        let clients = artifact
            .display_items
            .iter()
            .map(|item| item.id)
            .chain(artifact.chunks.iter().map(|chunk| chunk.id))
            .map(|id| id.client_id)
            .collect();
        for chunk in &mut artifact.chunks {
            chunk.client_is_just_created = false;
        }
        clients
    }
}

#[cfg(test)]
mod tests {
    use super::{PaintController, SubsequencesData};
    use crate::{PaintRecording, RasterEffectOutset, RecordedDisplayItem, RecordedDisplayItemKind};
    use paint_chunk::{DisplayItemClientInfo, PaintChunk, PaintChunkCompositorData};
    use paint_model::display_item_id::{DisplayItemId, DisplayItemIdType};
    use paint_model::PaintResult;
    use std::sync::Arc;

    fn drawing(client_id: u64) -> RecordedDisplayItem {
        RecordedDisplayItem {
            is_cacheable: true,
            is_tombstone: false,
            paint_invalidation_reason: Some(foundation::PaintInvalidationReason::kNone),
            scrollbar: None,
            foreign_layer: None,
            kind: RecordedDisplayItemKind::Drawing,
            id: DisplayItemId {
                client_id,
                r#type: DisplayItemIdType::kBoxDecorationBackground,
                fragment: 0,
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

    fn chunk(item: &RecordedDisplayItem, begin_index: u32) -> PaintChunk {
        PaintChunk {
            compositor: Some(Box::new(PaintChunkCompositorData::new())),
            id: item.id,
            is_cacheable: true,
            begin_index,
            end_index: begin_index + 1,
            ..Default::default()
        }
    }

    fn subsequences(client_id: u64) -> SubsequencesData {
        let parent = crate::SubsequenceMarkers {
            client_id,
            start_chunk_index: 0,
            end_chunk_index: 2,
            is_moved_from_cached_subsequence: false,
            paint_result: PaintResult::kFullyPainted,
        };
        let child = crate::SubsequenceMarkers {
            client_id: client_id + 1,
            start_chunk_index: 1,
            end_chunk_index: 2,
            is_moved_from_cached_subsequence: false,
            paint_result: PaintResult::kFullyPainted,
        };
        let mut data = SubsequencesData::default();
        data.map.insert(client_id, 0);
        data.map.insert(client_id + 1, 1);
        data.tree.extend([parent, child]);
        data
    }

    #[test]
    fn cached_item_lookup_prefers_paint_order_and_indexes_only_skipped_items() {
        let mut previous = PaintRecording::default();
        previous.display_items = vec![drawing(1), drawing(2), drawing(3)].into();
        let ids: Vec<_> = previous.display_items.iter().map(|item| item.id).collect();
        let mut controller = PaintController::WithPrevious(Some(Arc::new(previous)));
        let valid = DisplayItemClientInfo {
            invalidation_reason: Some(foundation::PaintInvalidationReason::kNone),
            is_cacheable: true,
            is_just_created: false,
        };

        assert_eq!(
            controller
                .TakeCachedDrawing(ids[1], valid, false)
                .unwrap()
                .item
                .id,
            ids[1]
        );
        assert!(controller.previous.as_ref().unwrap().display_items[1].IsTombstone());
        assert_eq!(
            controller
                .TakeCachedDrawing(ids[0], valid, false)
                .unwrap()
                .item
                .id,
            ids[0]
        );
        let invalid = DisplayItemClientInfo {
            invalidation_reason: Some(foundation::PaintInvalidationReason::kLayout),
            ..valid
        };
        assert!(controller
            .TakeCachedDrawing(ids[2], invalid, false)
            .is_none());
        assert_eq!(controller.Metrics().reused, 2);
        assert_eq!(controller.Metrics().rejected_invalid, 1);
    }

    #[test]
    fn controller_forces_repaint_then_moves_matching_cached_item() {
        let mut previous = PaintRecording::default();
        previous.display_items = vec![drawing(7)].into();
        let id = previous.display_items[0].id;
        let mut controller = PaintController::WithPrevious(Some(Arc::new(previous)));
        let valid = DisplayItemClientInfo {
            invalidation_reason: Some(foundation::PaintInvalidationReason::kNone),
            is_cacheable: true,
            is_just_created: false,
        };

        assert!(controller.TakeCachedDrawing(id, valid, true).is_none());
        assert!(controller.under_invalidation_checker.IsChecking());

        let mut repainted = drawing(7);
        repainted.paint_invalidation_reason = Some(foundation::PaintInvalidationReason::kLayout);
        let mut output = PaintRecording::default();
        output.display_items.push(repainted);
        controller.CheckNewItem(&mut output, false);

        assert!(!controller.under_invalidation_checker.IsChecking());
        assert!(controller.previous.as_ref().unwrap().display_items[0].IsTombstone());
        assert_eq!(
            output.display_items[0].GetPaintInvalidationReason(),
            Some(foundation::PaintInvalidationReason::kNone)
        );
    }

    #[test]
    fn cached_subsequence_moves_chunks_items_and_markers_together() {
        let mut previous = PaintRecording::default();
        previous.display_items = vec![drawing(7), drawing(8)].into();
        previous.chunks = vec![
            chunk(&previous.display_items[0], 0),
            chunk(&previous.display_items[1], 1),
        ]
        .into();
        let client = DisplayItemClientInfo {
            invalidation_reason: Some(foundation::PaintInvalidationReason::kNone),
            is_cacheable: true,
            is_just_created: false,
        };
        let mut controller =
            PaintController::WithPreviousAndSubsequences(Some(Arc::new(previous)), subsequences(7));
        let mut output = PaintRecording::default();

        assert_eq!(
            controller.UseCachedSubsequenceIfPossible(&mut output, 7, client, false),
            Some(PaintResult::kFullyPainted)
        );
        assert_eq!(output.display_items.len(), 2);
        assert!(output
            .chunks
            .iter()
            .all(|chunk| chunk.is_moved_from_cached_subsequence));
        assert!(controller
            .previous
            .as_ref()
            .unwrap()
            .display_items
            .iter()
            .all(RecordedDisplayItem::IsSubsequenceTombstone));
        let committed = controller.Finish(&mut output);
        assert_eq!(
            committed.GetSubsequenceMarkers(7).unwrap().end_chunk_index,
            2
        );
        assert!(
            committed
                .GetSubsequenceMarkers(7)
                .unwrap()
                .is_moved_from_cached_subsequence
        );
        let child = committed.GetSubsequenceMarkers(8).unwrap();
        assert_eq!((child.start_chunk_index, child.end_chunk_index), (1, 2));
        assert!(child.is_moved_from_cached_subsequence);
    }

    #[test]
    fn cached_subsequence_returns_and_preserves_previous_paint_result() {
        let mut previous = PaintRecording::default();
        previous.display_items = vec![drawing(7), drawing(8)].into();
        previous.chunks = vec![
            chunk(&previous.display_items[0], 0),
            chunk(&previous.display_items[1], 1),
        ]
        .into();
        let mut previous_subsequences = subsequences(7);
        previous_subsequences.tree[0].paint_result = PaintResult::kMayBeClippedByCullRect;
        let client = DisplayItemClientInfo {
            invalidation_reason: Some(foundation::PaintInvalidationReason::kNone),
            is_cacheable: true,
            is_just_created: false,
        };
        let mut controller = PaintController::WithPreviousAndSubsequences(
            Some(Arc::new(previous)),
            previous_subsequences,
        );
        let mut output = PaintRecording::default();

        assert_eq!(
            controller.UseCachedSubsequenceIfPossible(&mut output, 7, client, false),
            Some(PaintResult::kMayBeClippedByCullRect)
        );
        let committed = controller.Finish(&mut output);
        assert_eq!(
            committed.GetSubsequenceMarkers(7).unwrap().paint_result,
            PaintResult::kMayBeClippedByCullRect
        );
    }

    #[test]
    fn newly_recorded_subsequence_keeps_layer_paint_result() {
        let mut controller = PaintController::default();
        let mut output = PaintRecording::default();
        let index = controller.BeginSubsequence(7, &output);
        output.chunks.push(PaintChunk::default());
        controller.SetSubsequencePaintResult(index, PaintResult::kMayBeClippedByCullRect);
        controller.EndSubsequence(index, &output);
        assert_eq!(
            controller
                .new_subsequences
                .GetSubsequenceMarkers(7)
                .unwrap()
                .paint_result,
            PaintResult::kMayBeClippedByCullRect
        );
    }

    #[test]
    fn controller_drives_forced_subsequence_item_and_chunk_checks() {
        let mut previous = PaintRecording::default();
        previous.display_items = vec![drawing(7), drawing(8)].into();
        previous.chunks = vec![
            chunk(&previous.display_items[0], 0),
            chunk(&previous.display_items[1], 1),
        ]
        .into();
        let client = DisplayItemClientInfo {
            invalidation_reason: Some(foundation::PaintInvalidationReason::kNone),
            is_cacheable: true,
            is_just_created: false,
        };
        let mut controller =
            PaintController::WithPreviousAndSubsequences(Some(Arc::new(previous)), subsequences(7));
        let mut output = PaintRecording::default();
        assert!(controller
            .UseCachedSubsequenceIfPossible(&mut output, 7, client, true)
            .is_none());
        let subsequence = controller.BeginSubsequence(7, &output);

        for client_id in [7, 8] {
            let item = drawing(client_id);
            controller.ProcessNewItem(
                &mut output,
                item.id,
                client,
                item.visual_rect,
                item.draws_content,
                item.raster_effect_outset,
                item.visual_rect_is_accurate,
                item.record_rect_known_to_be_opaque,
                item.record_background,
                item.record_has_text,
            );
            output.display_items.push(item);
            controller.CheckNewItem(&mut output, false);
            controller.paint_chunker.force_new_chunk();
        }
        controller.EndSubsequence(subsequence, &output);

        assert!(!controller.under_invalidation_checker.IsChecking());
        assert!(controller
            .previous
            .as_ref()
            .unwrap()
            .display_items
            .iter()
            .all(RecordedDisplayItem::IsTombstone));
    }
}
