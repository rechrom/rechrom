//! Actual text-content metadata emitted by the CPU DisplayItemList binding.
//! The value is the native cc::NodeInfo shape and owns no DOM or Paint object.

use crate::adapter::{display_item_list::DisplayNodeInfo, geometry::Rect};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NodeInfo {
    pub node_id: u64,
    pub visual_rect: Rect,
}

impl DisplayNodeInfo for NodeInfo {
    type NodeId = u64;

    fn node_id(&self) -> Self::NodeId {
        self.node_id
    }

    fn visual_rect_mut(&mut self) -> &mut Rect {
        &mut self.visual_rect
    }

    fn new(node_id: Self::NodeId, visual_rect: Rect) -> Self {
        Self {
            node_id,
            visual_rect,
        }
    }
}

impl crate::adapter::picture_layer::PictureLayerNodeInfo for NodeInfo {
    fn visual_rect(&self) -> Rect {
        self.visual_rect
    }

    fn set_visual_rect(&mut self, rect: Rect) {
        self.visual_rect = rect;
    }
}
