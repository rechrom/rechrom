use compositor_tree::{geometry::Rect, ElementId};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScrollHitTestRect {
    pub scroll_element_id: ElementId,
    pub hit_test_rect: Rect,
}
