//! Neutral source virtual identity for a borrowed native Layout hit cursor.
//! No LayoutObject, DOM or mutable hit-result storage crosses this value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutHitTestKind {
    Base,
    Hidden,
    Block,
    Inline,
    Replaced,
    Image,
    EmbeddedContent,
    TextControlSingleLine,
    TextControlMultiLine,
    SvgRoot,
    SvgShape,
    SvgText,
    SvgContainer,
    SvgViewportContainer,
}

/// Source text-control identity and geometry, borrowed at hit-test time.
#[derive(Clone, Copy)]
pub struct TextControlHitFacts {
    pub inner_editor_node: u64,
    pub container_node: Option<u64>,
}
