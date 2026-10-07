use super::inline_item::InlineItems;
use super::inline_node::InlineNode;
use super::svg_inline_node_data::SvgInlineNodeData;
use foundation::{String, Vector};

// cpp: layoutng/internal/svg_text_attributes_request.h:12-17
pub struct SvgTextAttributesBuildRequest<'a> {
    pub node: &'a InlineNode,
    pub text: &'a String,
    pub items: &'a InlineItems,
    pub include_ifc_offsets: bool,
}

impl<'a> SvgTextAttributesBuildRequest<'a> {
    pub fn new(node: &'a InlineNode, text: &'a String, items: &'a InlineItems) -> Self {
        Self {
            node,
            text,
            items,
            include_ifc_offsets: false,
        }
    }
}

// cpp: layoutng/internal/svg_text_attributes_request.h:19-22
pub struct SvgTextAttributesBuildResult {
    pub data: *mut SvgInlineNodeData,
    pub ifc_text_content_offsets: Vector<u32>,
}

impl Default for SvgTextAttributesBuildResult {
    fn default() -> Self {
        Self {
            data: std::ptr::null_mut(),
            ifc_text_content_offsets: Vector::default(),
        }
    }
}
