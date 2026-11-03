/// Source: `painted_selection_bound.h:15-25`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintSelectionBoundType {
    Left,
    Right,
    Center,
    Empty,
    Hidden,
}

/// Source: `painted_selection_bound.h:19-25`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaintSelectionBound {
    pub bound_type: PaintSelectionBoundType,
    pub edge_start: [i32; 2],
    pub edge_end: [i32; 2],
}
