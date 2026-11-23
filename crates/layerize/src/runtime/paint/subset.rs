//! Borrowing view over the producer's actual Arc<PaintArtifact>. Indices are
//! the canonical subset's own storage, never copies of chunks or node trees.
use super::{DisplayItem, PaintArtifact, PaintChunk};
use std::sync::Arc;

// Source: third_party/blink/renderer/platform/graphics/paint/paint_chunk_subset.h:17-41,142-144.
#[derive(Clone, Default)]
pub struct PaintChunkSubset {
    artifact: Option<Arc<PaintArtifact>>,
    indices: Vec<usize>,
}

/// Source iterator value: the chunk and its artifact-relative index continue
/// to borrow the original PaintArtifact and display-item storage.
pub struct PaintChunkSubsetEntry<'a> {
    artifact: &'a PaintArtifact,
    chunk_index: usize,
}

impl<'a> PaintChunkSubsetEntry<'a> {
    pub fn IndexInPaintArtifact(&self) -> usize {
        self.chunk_index
    }

    pub fn chunk(&self) -> &'a PaintChunk {
        &self.artifact.chunks[self.chunk_index]
    }

    pub fn DisplayItems(&self) -> &'a [DisplayItem] {
        let chunk = self.chunk();
        self.artifact
            .GetDisplayItemList()
            .ItemsInRange(chunk.begin_index as usize, chunk.end_index as usize)
    }
}

impl PaintChunkSubset {
    pub fn from_first(artifact: Arc<PaintArtifact>, index: usize) -> Self {
        assert!(index < artifact.chunks.len());
        Self {
            artifact: Some(artifact),
            indices: vec![index],
        }
    }

    pub fn from_chunk(artifact: Arc<PaintArtifact>, chunk: &PaintChunk) -> Self {
        let index = artifact
            .chunks
            .iter()
            .position(|candidate| std::ptr::eq(candidate, chunk))
            .expect("source PaintChunkSubset chunk belongs to PaintArtifact");
        Self::from_first(artifact, index)
    }

    pub fn from_artifact(artifact: Arc<PaintArtifact>) -> Self {
        let indices = (0..artifact.chunks.len()).collect();
        Self {
            artifact: Some(artifact),
            indices,
        }
    }
    // Source: paint_chunk_subset.h:87-110. The iterator directly indexes the
    // same artifact on each dereference. It creates no projected chunk list.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &PaintChunk> {
        self.indices
            .iter()
            .map(|&index| &self.artifact().chunks[index])
    }
    pub fn entries(&self) -> impl ExactSizeIterator<Item = PaintChunkSubsetEntry<'_>> {
        self.indices
            .iter()
            .map(|&chunk_index| PaintChunkSubsetEntry {
                artifact: self.artifact(),
                chunk_index,
            })
    }
    pub fn indices(&self) -> &[usize] {
        &self.indices
    }
    pub fn artifact(&self) -> &Arc<PaintArtifact> {
        self.artifact
            .as_ref()
            .expect("source empty PaintChunkSubset has no PaintArtifact")
    }
    pub fn len(&self) -> usize {
        self.indices.len()
    }
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }
    pub fn chunk(&self, index: usize) -> &PaintChunk {
        &self.artifact().chunks[self.indices[index]]
    }
    // Source: paint_chunk_subset.h:120-131. Swapping an artifact changes only
    // the retained immutable owner; source subset indices remain untouched.
    pub fn set_artifact(&mut self, artifact: Arc<PaintArtifact>) {
        debug_assert_eq!(self.artifact().chunks.len(), artifact.chunks.len());
        self.artifact = Some(artifact);
    }
    pub fn merge(&mut self, guest: &Self) {
        debug_assert!(Arc::ptr_eq(self.artifact(), guest.artifact()));
        self.indices.extend_from_slice(&guest.indices);
    }
    // Source: paint_chunk_subset.h:133-135.
    pub fn approximate_unshared_memory_usage(&self) -> usize {
        std::mem::size_of::<Self>() + self.indices.capacity() * std::mem::size_of::<usize>()
    }
    // Source: paint_chunk_subset.cc:11-22. JSON is diagnostic text only.
    pub fn ToJSON(&self) -> serde_json::Value {
        serde_json::Value::Array(
            self.entries()
                .map(|entry| {
                    serde_json::Value::String(format!(
                        "index={} {}",
                        entry.IndexInPaintArtifact(),
                        entry.artifact.PaintChunkToString(entry.chunk(), false)
                    ))
                })
                .collect(),
        )
    }
    pub fn to_debug_json(&self) -> String {
        serde_json::to_string_pretty(&self.ToJSON()).expect("PaintChunkSubset JSON")
    }
}
impl<B> crate::adapter::pending_layer::PendingPaintChunkSubset<B> for PaintChunkSubset
where
    B: crate::adapter::pending_layer::PendingLayerTypes<
        PaintArtifact = PaintArtifact,
        PaintChunk = PaintChunk,
    >,
{
    fn FromFirst(artifact: Arc<PaintArtifact>, index: usize) -> Self {
        Self::from_first(artifact, index)
    }
    fn artifact(&self) -> &Arc<PaintArtifact> {
        self.artifact()
    }
    fn SetPaintArtifact(&mut self, artifact: Arc<PaintArtifact>) {
        self.set_artifact(artifact)
    }
    fn Merge(&mut self, guest: &Self) {
        self.merge(guest)
    }
    fn len(&self) -> usize {
        self.len()
    }
    fn chunk(&self, index: usize) -> &PaintChunk {
        self.chunk(index)
    }
    fn ToDebugJson(&self) -> String {
        self.to_debug_json()
    }
}
impl crate::adapter::paint_artifact_compositor::ArtifactChunkSubset for PaintChunkSubset {
    fn ApproximateUnsharedMemoryUsage(&self) -> usize {
        self.approximate_unshared_memory_usage()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use paint::display_item_id::{DisplayItemId, DisplayItemIdType};

    fn item() -> DisplayItem {
        DisplayItem {
            is_cacheable: true,
            is_tombstone: false,
            paint_invalidation_reason: Some(foundation::PaintInvalidationReason::kNone),
            scrollbar: None,
            foreign_layer: None,
            kind: super::super::RecordedDisplayItemKind::Drawing,
            id: DisplayItemId::default(),
            visual_rect: Default::default(),
            visual_rect_is_accurate: true,
            draws_content: false,
            raster_effect_outset: paint::paint_engine::RasterEffectOutset::kNone,
            record_rect_known_to_be_opaque: Default::default(),
            record_background: Default::default(),
            record_has_text: false,
            record_begin: 0,
            record_end: 0,
            scroll_translation: None,
        }
    }

    fn artifact() -> Arc<PaintArtifact> {
        let mut first = PaintChunk::default();
        first.begin_index = 0;
        first.end_index = 1;
        first.id = DisplayItemId {
            client_id: 7,
            r#type: DisplayItemIdType::kLayerChunk,
            fragment: 2,
        };
        let mut second = PaintChunk::default();
        second.begin_index = 1;
        second.end_index = 3;
        let mut artifact = PaintArtifact::default();
        artifact.chunks = vec![first, second].into();
        artifact.display_items = vec![item(); 3].into();
        artifact.RecordDebugInfo(7, Arc::<str>::from("source-client"), 11);
        Arc::new(artifact)
    }

    #[test]
    fn empty_subset_does_not_fabricate_an_artifact() {
        let subset = PaintChunkSubset::default();
        assert!(subset.is_empty());
        assert_eq!(subset.ToJSON(), serde_json::json!([]));
    }

    #[test]
    fn entries_retain_artifact_indices_and_display_item_ranges() {
        let artifact = artifact();
        let subset = PaintChunkSubset::from_artifact(artifact.clone());
        let entries: Vec<_> = subset.entries().collect();
        assert_eq!(entries[0].IndexInPaintArtifact(), 0);
        assert_eq!(entries[0].DisplayItems().len(), 1);
        assert_eq!(entries[1].IndexInPaintArtifact(), 1);
        assert_eq!(entries[1].DisplayItems().len(), 2);

        let single = PaintChunkSubset::from_chunk(artifact.clone(), &artifact.chunks[1]);
        assert_eq!(single.indices(), &[1]);
        assert!(single.to_debug_json().contains("index=1"));
        assert!(subset.to_debug_json().contains("source-client"));
    }

    #[test]
    fn merge_and_artifact_swap_preserve_source_indices() {
        let first_artifact = artifact();
        let mut subset = PaintChunkSubset::from_first(first_artifact.clone(), 0);
        let guest = PaintChunkSubset::from_first(first_artifact, 1);
        subset.merge(&guest);
        assert_eq!(subset.indices(), &[0, 1]);

        let replacement = artifact();
        subset.set_artifact(replacement.clone());
        assert!(Arc::ptr_eq(subset.artifact(), &replacement));
        assert_eq!(subset.indices(), &[0, 1]);
    }
}
