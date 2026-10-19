//! Borrowing view over the producer's actual Arc<PaintArtifact>. Indices are
//! the canonical subset's own storage, never copies of chunks or node trees.
use super::{PaintArtifact, PaintChunk};
use std::sync::Arc;

// Source: third_party/blink/renderer/platform/graphics/paint/paint_chunk_subset.h:17-41,142-144.
#[derive(Clone)]
pub struct PaintChunkSubset {
    artifact: Arc<PaintArtifact>,
    indices: Vec<usize>,
}
impl PaintChunkSubset {
    pub fn from_first(artifact: Arc<PaintArtifact>, index: usize) -> Self {
        assert!(index < artifact.chunks.len());
        Self {
            artifact,
            indices: vec![index],
        }
    }
    pub fn from_artifact(artifact: Arc<PaintArtifact>) -> Self {
        let indices = (0..artifact.chunks.len()).collect();
        Self { artifact, indices }
    }
    // Source: paint_chunk_subset.h:87-110. The iterator directly indexes the
    // same artifact on each dereference. It creates no projected chunk list.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &PaintChunk> {
        self.indices
            .iter()
            .map(|&index| &self.artifact.chunks[index])
    }
    pub fn indices(&self) -> &[usize] {
        &self.indices
    }
    pub fn artifact(&self) -> &Arc<PaintArtifact> {
        &self.artifact
    }
    pub fn len(&self) -> usize {
        self.indices.len()
    }
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }
    pub fn chunk(&self, index: usize) -> &PaintChunk {
        &self.artifact.chunks[self.indices[index]]
    }
    // Source: paint_chunk_subset.h:120-131. Swapping an artifact changes only
    // the retained immutable owner; source subset indices remain untouched.
    pub fn set_artifact(&mut self, artifact: Arc<PaintArtifact>) {
        debug_assert_eq!(self.artifact.chunks.len(), artifact.chunks.len());
        self.artifact = artifact;
    }
    pub fn merge(&mut self, guest: &Self) {
        debug_assert!(Arc::ptr_eq(&self.artifact, &guest.artifact));
        self.indices.extend_from_slice(&guest.indices);
    }
    // Source: paint_chunk_subset.h:133-135.
    pub fn approximate_unshared_memory_usage(&self) -> usize {
        std::mem::size_of::<Self>() + self.indices.capacity() * std::mem::size_of::<usize>()
    }
    // Source: paint_chunk_subset.cc:11-22. JSON is diagnostic text only.
    pub fn to_debug_json(&self) -> String {
        serde_json::Value::Array(
            self.indices
                .iter()
                .map(|&index| {
                    let chunk = &self.artifact.chunks[index];
                    serde_json::Value::String(format!(
                        "index={} {:?} {}..{}",
                        index, chunk.id, chunk.begin_index, chunk.end_index
                    ))
                })
                .collect(),
        )
        .to_string()
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
