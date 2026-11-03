use crate::PaintSelectionBound;

/// Source: `layer_selection_data.h:16-30`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PaintLayerSelectionData {
    pub start: Option<PaintSelectionBound>,
    pub end: Option<PaintSelectionBound>,
    pub any_selection_was_painted: bool,
}
