use std::sync::Arc;

use crate::paint_property_tree::{ClipPaintPropertyNode, TransformPaintPropertyNode};

#[derive(Clone, Debug, PartialEq)]
pub enum PaintCompositorViewportMode {
    SingleLayoutViewport,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaintCompositorViewport {
    pub mode: PaintCompositorViewportMode,
    pub viewport_clip: Arc<ClipPaintPropertyNode>,
    pub overscroll_elasticity_transform: Arc<TransformPaintPropertyNode>,
    pub page_scale: Arc<TransformPaintPropertyNode>,
    pub visual_scroll_translation: Arc<TransformPaintPropertyNode>,
    pub layout_scroll_translation: Option<Arc<TransformPaintPropertyNode>>,
    pub scroll_translation_nodes: Vec<Arc<TransformPaintPropertyNode>>,
}
