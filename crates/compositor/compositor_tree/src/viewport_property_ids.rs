use crate::property_ids::kInvalidPropertyNodeId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewportPropertyIds {
    pub overscroll_elasticity_transform: i32,
    pub page_scale_transform: i32,
    pub inner_scroll: i32,
    pub outer_clip: i32,
    pub outer_scroll: i32,
}

impl Default for ViewportPropertyIds {
    fn default() -> Self {
        Self {
            overscroll_elasticity_transform: kInvalidPropertyNodeId,
            page_scale_transform: kInvalidPropertyNodeId,
            inner_scroll: kInvalidPropertyNodeId,
            outer_clip: kInvalidPropertyNodeId,
            outer_scroll: kInvalidPropertyNodeId,
        }
    }
}
