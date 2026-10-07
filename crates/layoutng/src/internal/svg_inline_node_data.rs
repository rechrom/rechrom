#![allow(non_snake_case)]

use foundation::{HeapHashMap, HeapVector, Member, Traceable, Visitor};

use super::layout_object::LayoutObject;
use super::layout_text::LayoutText;
use super::svg_character_data::SvgCharacterData;

// cpp: layoutng/internal/svg_inline_node_data.h:18-32
#[derive(Clone)]
pub struct SvgTextContentRange {
    pub layout_object: Member<LayoutObject>,
    pub start_index: u32,
    pub end_index: u32,
}

impl SvgTextContentRange {
    // cpp: layoutng/internal/svg_inline_node_data.h:21-21
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.layout_object);
    }
}

impl Traceable for SvgTextContentRange {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        SvgTextContentRange::Trace(self, visitor);
    }
}

// C++ permits move construction and memberwise comparison for this record.
impl PartialEq for SvgTextContentRange {
    fn eq(&self, other: &Self) -> bool {
        self.layout_object.Get() == other.layout_object.Get()
            && self.start_index == other.start_index
            && self.end_index == other.end_index
    }
}

// cpp: layoutng/internal/svg_inline_node_data.h:38-39
pub type SvgTextChunkOffsets = HeapHashMap<Member<LayoutText>, Vec<u32>>;

// cpp: layoutng/internal/svg_inline_node_data.h:42-53
pub struct SvgInlineNodeData {
    pub character_data_list: Vec<(u32, SvgCharacterData)>,
    pub text_length_range_list: HeapVector<SvgTextContentRange>,
    pub text_path_range_list: HeapVector<SvgTextContentRange>,
    pub chunk_offsets: SvgTextChunkOffsets,
}

impl Default for SvgInlineNodeData {
    fn default() -> Self {
        Self {
            character_data_list: Vec::new(),
            text_length_range_list: HeapVector::new(),
            text_path_range_list: HeapVector::new(),
            chunk_offsets: HeapHashMap::default(),
        }
    }
}

impl SvgInlineNodeData {
    // cpp: layoutng/internal/svg_inline_node_data.h:43-47
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.text_length_range_list);
        visitor.Trace(&self.text_path_range_list);
        visitor.Trace(&self.chunk_offsets);
    }
}

impl Traceable for SvgInlineNodeData {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        SvgInlineNodeData::Trace(self, visitor);
    }
}
