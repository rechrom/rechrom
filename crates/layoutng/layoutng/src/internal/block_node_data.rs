#![allow(non_snake_case)]

use foundation::DynamicTo;
use layoutng_geometry::geometry::overflow_clip_axes::OverflowClipAxes;

use super::block_node::BlockNode;
use super::layout_block::LayoutBlock;
use super::layout_node_metadata::{Element, ElementType};

impl BlockNode {
    // cpp: layoutng/internal/block_node_data.cc:13-17
    pub fn IsMathMLTableCell(&self) -> bool {
        let element = DynamicTo::<Element>(self.GetDOMNode());
        !element.is_null()
            && unsafe { &*element }.GetElementType() == ElementType::kMathMLTableCellElement
    }

    // cpp: layoutng/internal/block_node_data.cc:19-25
    pub fn SliderThumbValueRatio(&self) -> f64 {
        let node = self.GetDOMNode();
        let host = unsafe { &*node }.OwnerShadowHost();
        assert!(!host.is_null());
        let data = unsafe { &*host }.InputElementData();
        data.as_ref()
            .and_then(|data| data.range_value_ratio)
            .expect("slider thumb host must carry a range value ratio")
    }

    // cpp: layoutng/internal/block_node_data.cc:28
    pub fn IsFrameSet(&self) -> bool {
        unsafe { &*self.GetLayoutBox() }.IsFrameSet()
    }

    // cpp: layoutng/internal/block_node_data.cc:30
    pub fn IsParentNGFrameSet(&self) -> bool {
        let parent = unsafe { &*self.GetLayoutBox() }.Parent();
        unsafe { &*parent }.IsFrameSet()
    }

    // cpp: layoutng/internal/block_node_data.cc:32
    pub fn IsMulticolContainer(&self) -> bool {
        unsafe { &*self.GetLayoutBox() }.IsMulticolContainer()
    }

    // cpp: layoutng/internal/block_node_data.cc:34
    pub fn MayContainAnchor(&self) -> bool {
        unsafe { &*self.GetLayoutBox() }.MayContainAnchor()
    }

    // cpp: layoutng/internal/block_node_data.cc:36
    pub fn IsOverscrollAreaParent(&self) -> bool {
        unsafe { &*self.GetLayoutBox() }.IsOverscrollAreaParent()
    }

    // cpp: layoutng/internal/block_node_data.cc:38
    pub fn HasLeftOverflow(&self) -> bool {
        unsafe { &*self.GetLayoutBox() }.HasLeftOverflow()
    }

    // cpp: layoutng/internal/block_node_data.cc:40
    pub fn HasTopOverflow(&self) -> bool {
        unsafe { &*self.GetLayoutBox() }.HasTopOverflow()
    }

    // cpp: layoutng/internal/block_node_data.cc:42
    pub fn HasNonVisibleOverflow(&self) -> bool {
        unsafe { &*self.GetLayoutBox() }.HasNonVisibleOverflow()
    }

    // cpp: layoutng/internal/block_node_data.cc:44-46
    pub fn GetOverflowClipAxes(&self) -> OverflowClipAxes {
        unsafe { &*self.GetLayoutBox() }.GetOverflowClipAxes()
    }

    // cpp: layoutng/internal/block_node_data.cc:48-62
    pub fn IsQuirkyAndFillsViewport(&self) -> bool {
        if !unsafe { &*self.GetLayoutBox() }.InQuirksModeForLayout()
            || self.IsOutOfFlowPositioned()
            || self.IsFloating()
            || self.IsInlineLevel()
        {
            return false;
        }
        self.IsDocumentElement() || self.IsBody()
    }

    // cpp: layoutng/internal/block_node_data.cc:64-66
    pub fn GetScrollMarkerGroup(&self) -> BlockNode {
        let group = unsafe { &mut *self.GetLayoutBox() }.GetScrollMarkerGroup();
        BlockNode::new(DynamicTo::<LayoutBlock>(group).cast())
    }

    // cpp: layoutng/internal/block_node_data.cc:68-70
    pub fn ShouldApplyLayoutContainment(&self) -> bool {
        unsafe { &*self.GetLayoutBox() }.ShouldApplyLayoutContainment()
    }

    // cpp: layoutng/internal/block_node_data.cc:72-74
    pub fn ShouldApplyPaintContainment(&self) -> bool {
        unsafe { &*self.GetLayoutBox() }.ShouldApplyPaintContainment()
    }

    // cpp: layoutng/internal/block_node_data.cc:76-80
    pub fn HasLineIfEmpty(&self) -> bool {
        let block = DynamicTo::<LayoutBlock>(self.GetLayoutBox());
        !block.is_null() && unsafe { &*block }.HasLineIfEmpty()
    }
}
