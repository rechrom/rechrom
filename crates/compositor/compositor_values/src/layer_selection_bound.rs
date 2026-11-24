use compositor_tree::geometry::Point;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelectionBoundType {
    LEFT,
    RIGHT,
    CENTER,
    HIDDEN,
    #[default]
    EMPTY,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Selection<T> {
    pub start: T,
    pub end: T,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LayerSelectionBound {
    pub r#type: SelectionBoundType,
    pub edge_start: Point,
    pub edge_end: Point,
    pub layer_id: i32,
    pub hidden: bool,
}

impl LayerSelectionBound {
    pub fn ToString(&self) -> String {
        format!(
            "LayerSelectionBound({},{}, {},{}, {})",
            self.edge_start.x(),
            self.edge_start.y(),
            self.edge_end.x(),
            self.edge_end.y(),
            i32::from(self.hidden)
        )
    }
}

pub type LayerSelection = Selection<LayerSelectionBound>;
