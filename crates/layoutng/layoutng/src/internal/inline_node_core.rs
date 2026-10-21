#![allow(non_snake_case)]

use foundation::{String as BlinkString, To, UnsupportedLayout};

use super::inline_node::InlineNode;
use super::layout_block_flow::LayoutBlockFlow;
use super::layout_box::LayoutBox;
use super::layout_input_node::{LayoutInputNode, LayoutInputNodeType};
use super::layout_pass_scope::LayoutPassScope;

impl InlineNode {
    // cpp: layoutng/internal/inline_node_core.cc:14-16
    pub fn GetLayoutBlockFlow(&self) -> *mut LayoutBlockFlow {
        To::<LayoutBlockFlow>(self.GetLayoutBox())
    }

    // cpp: layoutng/internal/inline_node_core.cc:18-24
    pub fn new(block: *mut LayoutBlockFlow) -> Self {
        let base = LayoutInputNode::Create(block.cast::<LayoutBox>(), LayoutInputNodeType::kInline);
        debug_assert!(!block.is_null());
        if unsafe { &*block }.GetInlineNodeData().is_null() {
            unsafe { &mut *block }.ResetInlineNodeData();
        }
        Self { base }
    }

    // cpp: layoutng/internal/inline_node_core.cc:26-32
    pub fn IsBlockLevel(&self) -> bool {
        let algorithms = LayoutPassScope::Algorithms();
        let callback = if algorithms.is_null() {
            None
        } else {
            unsafe { &*algorithms }.inline_support.is_block_level
        };
        let callback = callback.unwrap_or_else(|| {
            std::panic::panic_any(UnsupportedLayout::new(
                "inline layout module is not installed",
            ))
        });
        callback(self)
    }

    // cpp: layoutng/internal/inline_node_core.cc:34-36
    pub fn ToString(&self) -> BlinkString {
        BlinkString::from("InlineNode")
    }
}
