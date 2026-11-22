//! DCHECK paint-artifact diagnostics.
//!
//! Source entity: Chromium revision 6c1d401fcca5e1b0030563a90c2f2bba168e0c15,
//! `third_party/blink/renderer/platform/graphics/paint/
//! paint_controller_debug_data.cc`.

use super::{PaintController, SubsequenceMarkers};
use crate::{DisplayItemListJsonOption, PaintRecording};

/// Source: `paint_controller_debug_data.cc:18-43`.
struct PaintArtifactAsJSON<'a> {
    artifact_: &'a PaintRecording,
    subsequences_: &'a [SubsequenceMarkers],
    next_subsequence_: usize,
    option_: DisplayItemListJsonOption,
}

impl<'a> PaintArtifactAsJSON<'a> {
    fn new(
        artifact: &'a PaintRecording,
        subsequences: &'a [SubsequenceMarkers],
        option: DisplayItemListJsonOption,
    ) -> Self {
        Self {
            artifact_: artifact,
            subsequences_: subsequences,
            next_subsequence_: 0,
            option_: option,
        }
    }

    /// Source: `paint_controller_debug_data.cc:30-33`.
    fn ToString(mut self) -> String {
        let value = self.ChunksAsJSONArrayRecursive(0, self.artifact_.GetPaintChunks().len());
        serde_json::to_string_pretty(&value)
            .expect("paint artifact debug data contains only source JSON values")
    }

    /// Source: `paint_controller_debug_data.cc:45-61`.
    fn SubsequenceAsJSONObjectRecursive(&mut self) -> serde_json::Value {
        let subsequence = self.subsequences_[self.next_subsequence_];
        self.next_subsequence_ += 1;

        let mut object = serde_json::Map::new();
        object.insert(
            "subsequence".into(),
            serde_json::Value::String(format!(
                "client: {:#x} {}",
                subsequence.client_id,
                self.artifact_
                    .ClientDebugName(subsequence.client_id as usize)
            )),
        );
        object.insert(
            "chunks".into(),
            self.ChunksAsJSONArrayRecursive(
                subsequence.start_chunk_index,
                subsequence.end_chunk_index,
            ),
        );
        serde_json::Value::Object(object)
    }

    /// Source: `paint_controller_debug_data.cc:63-93`.
    fn ChunksAsJSONArrayRecursive(
        &mut self,
        start_chunk_index: usize,
        end_chunk_index: usize,
    ) -> serde_json::Value {
        let mut array = Vec::new();
        let mut chunk_index = start_chunk_index;

        while self.next_subsequence_ != self.subsequences_.len()
            && self.subsequences_[self.next_subsequence_].start_chunk_index < end_chunk_index
        {
            let subsequence = self.subsequences_[self.next_subsequence_];
            if subsequence.client_id == 0 {
                // Skip unfinished subsequences during painting.
                self.next_subsequence_ += 1;
                continue;
            }
            debug_assert!(subsequence.start_chunk_index >= chunk_index);
            debug_assert!(subsequence.end_chunk_index <= end_chunk_index);

            if chunk_index < subsequence.start_chunk_index {
                Self::AppendArray(
                    &mut array,
                    self.artifact_.ChunksAsJSON(
                        chunk_index,
                        subsequence.start_chunk_index,
                        self.option_,
                    ),
                );
            }
            array.push(self.SubsequenceAsJSONObjectRecursive());
            chunk_index = subsequence.end_chunk_index;
        }

        if chunk_index < end_chunk_index {
            Self::AppendArray(
                &mut array,
                self.artifact_
                    .ChunksAsJSON(chunk_index, end_chunk_index, self.option_),
            );
        }
        serde_json::Value::Array(array)
    }

    fn AppendArray(destination: &mut Vec<serde_json::Value>, value: serde_json::Value) {
        let serde_json::Value::Array(mut values) = value else {
            unreachable!("PaintRecording::ChunksAsJSON always returns an array")
        };
        destination.append(&mut values);
    }
}

/// Source: `paint_controller_debug_data.cc:95-115`.
pub(crate) fn DebugDataAsStringForArtifacts(
    current: Option<(&PaintRecording, &[SubsequenceMarkers])>,
    new: Option<(&PaintRecording, &[SubsequenceMarkers])>,
    option: DisplayItemListJsonOption,
) -> String {
    let mut output = String::from("current paint artifact: ");
    if let Some((artifact, subsequences)) = current {
        output.push_str(&PaintArtifactAsJSON::new(artifact, subsequences, option).ToString());
    } else {
        output.push_str("null");
    }
    output.push_str("\nnew paint artifact: ");
    if let Some((artifact, subsequences)) = new {
        output.push_str(&PaintArtifactAsJSON::new(artifact, subsequences, option).ToString());
    } else {
        output.push_str("null");
    }
    output
}

impl PaintController {
    pub(crate) fn DebugDataAsString(
        &self,
        new_paint_artifact: Option<&PaintRecording>,
        option: DisplayItemListJsonOption,
    ) -> String {
        DebugDataAsStringForArtifacts(
            self.previous
                .as_deref()
                .map(|artifact| (artifact, self.previous_subsequences.tree.as_slice())),
            new_paint_artifact.map(|artifact| (artifact, self.new_subsequences.tree.as_slice())),
            option,
        )
    }

    /// Source: `paint_controller_debug_data.cc:117-120`.
    fn ShowDebugDataInternal(
        &self,
        new_paint_artifact: Option<&PaintRecording>,
        option: DisplayItemListJsonOption,
    ) {
        eprintln!("{}", self.DebugDataAsString(new_paint_artifact, option));
    }

    /// Source: `paint_controller_debug_data.cc:122-124`.
    #[allow(dead_code)]
    pub(crate) fn ShowCompactDebugData(&self, new_paint_artifact: Option<&PaintRecording>) {
        self.ShowDebugDataInternal(new_paint_artifact, DisplayItemListJsonOption::kCompact);
    }

    /// Source: `paint_controller_debug_data.cc:126-128`.
    #[allow(dead_code)]
    pub(crate) fn ShowDebugData(&self, new_paint_artifact: Option<&PaintRecording>) {
        self.ShowDebugDataInternal(new_paint_artifact, DisplayItemListJsonOption::kDefault);
    }

    /// Source: `paint_controller_debug_data.cc:130-132`.
    #[allow(dead_code)]
    pub(crate) fn ShowDebugDataWithPaintRecords(
        &self,
        new_paint_artifact: Option<&PaintRecording>,
    ) {
        self.ShowDebugDataInternal(
            new_paint_artifact,
            DisplayItemListJsonOption::kShowPaintRecords,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use paint_chunk::PaintChunk;

    #[test]
    fn source_recursive_json_preserves_nested_subsequence_order() {
        let mut artifact = PaintRecording {
            chunks: vec![
                PaintChunk::default(),
                PaintChunk::default(),
                PaintChunk::default(),
            ]
            .into(),
            ..Default::default()
        };
        artifact.RecordDebugInfo(1, "outer".into(), 0);
        artifact.RecordDebugInfo(2, "inner".into(), 0);
        let tree = [
            SubsequenceMarkers {
                client_id: 1,
                start_chunk_index: 0,
                end_chunk_index: 3,
                is_moved_from_cached_subsequence: false,
                paint_result: paint_model::PaintResult::kFullyPainted,
            },
            SubsequenceMarkers {
                client_id: 2,
                start_chunk_index: 1,
                end_chunk_index: 2,
                is_moved_from_cached_subsequence: false,
                paint_result: paint_model::PaintResult::kFullyPainted,
            },
        ];

        let value: serde_json::Value = serde_json::from_str(
            &PaintArtifactAsJSON::new(&artifact, &tree, DisplayItemListJsonOption::kDefault)
                .ToString(),
        )
        .unwrap();
        let outer = &value[0];
        assert_eq!(outer["subsequence"], "client: 0x1 outer");
        assert_eq!(outer["chunks"].as_array().unwrap().len(), 3);
        assert_eq!(outer["chunks"][1]["subsequence"], "client: 0x2 inner");
    }

    #[test]
    fn controller_debug_data_labels_current_and_new_artifacts() {
        let current = std::sync::Arc::new(PaintRecording::default());
        let controller = PaintController::WithPrevious(Some(current));
        let output = PaintRecording::default();
        let text = controller.DebugDataAsString(Some(&output), DisplayItemListJsonOption::kDefault);
        assert!(text.starts_with("current paint artifact: []"));
        assert!(text.ends_with("new paint artifact: []"));
    }
}
