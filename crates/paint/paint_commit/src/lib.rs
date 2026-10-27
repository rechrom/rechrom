#![allow(non_snake_case)]

//! Immutable PaintArtifact publication helpers.
//!
//! Commit receives completed values, freezes them behind `Arc`, and derives
//! client receipts. It never walks layout or records paint operations.

use std::{collections::HashSet, sync::Arc};

use paint_model::display_item_id::DisplayItemId;

pub mod scroll_paint_update;

pub use paint_model::{PaintCompositorViewport, PaintCompositorViewportMode};
pub use paint_record::{
    DocumentPaintArtifactRecord, ImageDependencyIndex, ImagePaintDependency, PaintMutation,
    PaintMutationResult, PaintResourceUpdate,
};

/// Completed recording value as exposed across the Paint package boundary.
/// Only this commit package gives the mutable recording its published name.
pub type PaintArtifact = paint_record::PaintRecording;

/// One completed PaintEngine publication. The artifact is immutable behind
/// `Arc`; the client receipt is opaque and can only be consumed through
/// Layout's commit interface. Neither value contains a copied fragment tree.
pub struct PaintOutput {
    artifact: Arc<PaintArtifact>,
    layout_commit: layoutng_assembly::layout_engine::PaintClientCommit,
}

impl PaintOutput {
    pub fn new(
        artifact: Arc<PaintArtifact>,
        layout_commit: layoutng_assembly::layout_engine::PaintClientCommit,
    ) -> Self {
        Self {
            artifact,
            layout_commit,
        }
    }
}

impl layoutng_assembly::layout_engine::PaintCommitOutput for PaintOutput {
    type Artifact = Arc<PaintArtifact>;

    fn IntoLayoutCommit(
        self,
    ) -> (
        Self::Artifact,
        layoutng_assembly::layout_engine::PaintClientCommit,
    ) {
        (self.artifact, self.layout_commit)
    }
}

/// Seal metadata derived from completed record and chunk streams, then assign
/// the immutable recording identity. This is the commit-stage hot path.
pub fn FinalizeArtifact(recording: &mut paint_record::PaintRecording) {
    recording.recorded_properties = recording
        .chunks
        .iter()
        .map(|chunk| chunk.properties.clone())
        .collect();
    recording.image_dependencies = Arc::new(ImageDependencyIndex::build(
        &recording.items,
        &recording.display_items,
        &recording.chunks,
    ));
    recording.RefreshImageContentRevisions();
    recording.recording_revision = foundation::NewUniqueObjectId();
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PaintCommitMetrics {
    pub clients: usize,
    pub recording_revision: u64,
}

#[inline]
pub fn CommitArtifact(recording: paint_record::PaintRecording) -> Arc<PaintArtifact> {
    Arc::new(recording)
}

pub fn CollectClientIds(
    display_items: impl IntoIterator<Item = DisplayItemId>,
    chunks: impl IntoIterator<Item = DisplayItemId>,
) -> HashSet<u64> {
    display_items
        .into_iter()
        .chain(chunks)
        .map(|id| id.client_id)
        .collect()
}
