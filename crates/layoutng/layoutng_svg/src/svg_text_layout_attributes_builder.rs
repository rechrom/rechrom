#![allow(non_snake_case)]

use foundation::{
    DynamicTo, HeapVector, MakeGarbageCollected, Member, String as BlinkString, StringView, To,
};
use layoutng_assembly::internal::inline_item::{InlineItemType, InlineItems};
use layoutng_assembly::internal::inline_node::InlineNode;
use layoutng_assembly::internal::layout_block_flow::LayoutBlockFlow;
use layoutng_assembly::internal::layout_input::ElementData;
use layoutng_assembly::internal::layout_node_metadata::Element;
use layoutng_assembly::internal::layout_object::LayoutObject;
use layoutng_assembly::internal::svg_character_data::SvgCharacterData;
use layoutng_assembly::internal::svg_inline_node_data::{SvgInlineNodeData, SvgTextContentRange};

// C++ ClampTo<float> compares the f64 input against f32 bounds before casting.
fn ClampToFloat(value: f64) -> f32 {
    if value >= f32::MAX as f64 {
        f32::MAX
    } else if value <= f32::MIN as f64 {
        f32::MIN
    } else {
        value as f32
    }
}

struct PositioningScope {
    object: *const LayoutObject,
    input: *const ElementData,
    local_index: usize,
    start_index: u32,
}

// cpp: layoutng_svg/svg_text_layout_attributes_builder.h:31-74
pub struct SvgTextLayoutAttributesBuilder {
    block_flow: *mut LayoutBlockFlow,
    resolved: Vec<(u32, SvgCharacterData)>,
    ifc_text_content_offsets: Vec<u32>,
    text_length_range_list: HeapVector<SvgTextContentRange>,
    text_path_range_list: HeapVector<SvgTextContentRange>,
}

impl SvgTextLayoutAttributesBuilder {
    // cpp: layoutng_svg/svg_text_layout_attributes_builder.cc:15-16
    pub fn new(ifc: &InlineNode) -> Self {
        let block_flow = To::<LayoutBlockFlow>(ifc.AsLayoutInputNode().GetLayoutBox());
        Self {
            block_flow,
            resolved: Vec::new(),
            ifc_text_content_offsets: Vec::new(),
            text_length_range_list: HeapVector::new(),
            text_path_range_list: HeapVector::new(),
        }
    }

    // cpp: layoutng_svg/svg_text_layout_attributes_builder.cc:18-142
    pub fn Build(&mut self, text: &BlinkString, items: &InlineItems) {
        let element = DynamicTo::<Element>(unsafe { &*self.block_flow }.GetNode());
        assert!(!element.is_null());
        let root_data = unsafe { &*element }
            .InputElementData()
            .as_ref()
            .expect("SVG text metadata is missing");
        let mut scopes = vec![PositioningScope {
            object: self.block_flow.cast(),
            input: root_data,
            local_index: 0,
            start_index: 0,
        }];
        let mut addressable_index = 0u32;
        let mut is_first_character = true;
        let mut in_text_path = false;
        let mut first_character_in_text_path = false;
        let mut text_path_start: Option<u32> = None;
        for item_member in items.iter() {
            let item = unsafe { &*item_member.Get() };
            let object = item.GetLayoutObject();
            if item.Type() == InlineItemType::kOpenTag {
                if unsafe { &*object }.IsSVGTextPath() {
                    assert!(!in_text_path);
                    in_text_path = true;
                    first_character_in_text_path = true;
                    text_path_start = Some(addressable_index);
                }
                if unsafe { &*object }.IsSVGTSpan() || unsafe { &*object }.IsSVGTextPath() {
                    let scope_element = DynamicTo::<Element>(unsafe { &*object }.GetNode());
                    assert!(!scope_element.is_null());
                    let scope_data = unsafe { &*scope_element }
                        .InputElementData()
                        .as_ref()
                        .expect("SVG positioning metadata is missing");
                    scopes.push(PositioningScope {
                        object,
                        input: scope_data,
                        local_index: 0,
                        start_index: addressable_index,
                    });
                }
                continue;
            }
            if item.Type() == InlineItemType::kCloseTag {
                if unsafe { &*object }.IsSVGTextPath() {
                    assert!(in_text_path);
                    let start = text_path_start.expect("SVG textPath start is missing");
                    in_text_path = false;
                    first_character_in_text_path = false;
                    if addressable_index != start {
                        self.text_path_range_list.push_back(SvgTextContentRange {
                            layout_object: Member::from_ptr(object),
                            start_index: start,
                            end_index: addressable_index - 1,
                        });
                    }
                    text_path_start = None;
                }
                if unsafe { &*object }.IsSVGTSpan() || unsafe { &*object }.IsSVGTextPath() {
                    assert!(scopes.len() > 1);
                    let scope = scopes.last().unwrap();
                    assert_eq!(scope.object, object);
                    let input = unsafe { &*scope.input };
                    if input.svg_text_length.is_some_and(|length| length > 0.0)
                        && addressable_index != scope.start_index
                    {
                        self.text_length_range_list.push_back(SvgTextContentRange {
                            layout_object: Member::from_ptr(object),
                            start_index: scope.start_index,
                            end_index: addressable_index - 1,
                        });
                    }
                    scopes.pop();
                }
                continue;
            }
            if item.Type() != InlineItemType::kText {
                continue;
            }
            let item_text =
                StringView::from_blink_string_range(text, item.StartOffset(), item.Length());
            let mut offset = 0u32;
            while offset < item.Length() {
                let mut character = SvgCharacterData::default();
                for scope in &scopes {
                    let input = unsafe { &*scope.input };
                    let index = scope.local_index;
                    if let Some(value) = input.svg_x.get(index) {
                        character.x = ClampToFloat(*value);
                    }
                    if let Some(value) = input.svg_y.get(index) {
                        character.y = ClampToFloat(*value);
                    }
                    if let Some(value) = input.svg_dx.get(index) {
                        character.dx = ClampToFloat(*value);
                    }
                    if let Some(value) = input.svg_dy.get(index) {
                        character.dy = ClampToFloat(*value);
                    }
                    if !input.svg_rotate.is_empty() {
                        let index = index.min(input.svg_rotate.len() - 1);
                        character.rotate = ClampToFloat(input.svg_rotate[index]);
                    }
                }
                character.anchored_chunk = character.HasX() || character.HasY();
                if first_character_in_text_path {
                    character.anchored_chunk = true;
                    first_character_in_text_path = false;
                }
                if is_first_character {
                    is_first_character = false;
                    character.anchored_chunk = true;
                    if unsafe { &*self.block_flow }.IsHorizontalWritingMode() && !character.HasX() {
                        character.x = 0.0;
                    }
                    if !unsafe { &*self.block_flow }.IsHorizontalWritingMode() && !character.HasY()
                    {
                        character.y = 0.0;
                    }
                }
                if character.HasX()
                    || character.HasY()
                    || character.HasRotate()
                    || character.anchored_chunk
                {
                    self.resolved.push((addressable_index, character));
                    self.ifc_text_content_offsets
                        .push(item.StartOffset() + offset);
                }
                addressable_index += 1;
                for scope in &mut scopes {
                    scope.local_index += 1;
                }
                offset = item_text.NextCodePointOffset(offset);
            }
        }
        assert!(!in_text_path);
        assert!(text_path_start.is_none());
        assert_eq!(scopes.len(), 1);
        let root_input = unsafe { &*scopes[0].input };
        if root_input
            .svg_text_length
            .is_some_and(|length| length > 0.0)
            && addressable_index > 0
        {
            self.text_length_range_list.push_back(SvgTextContentRange {
                layout_object: Member::from_ptr(self.block_flow.cast()),
                start_index: 0,
                end_index: addressable_index - 1,
            });
        }
        debug_assert_eq!(self.resolved.len(), self.ifc_text_content_offsets.len());
    }

    // cpp: layoutng_svg/svg_text_layout_attributes_builder.cc:144-150
    pub fn CreateSvgInlineNodeData(&mut self) -> *mut SvgInlineNodeData {
        let mut result = SvgInlineNodeData::default();
        result.character_data_list = std::mem::take(&mut self.resolved);
        result.text_length_range_list = std::mem::take(&mut self.text_length_range_list);
        result.text_path_range_list = std::mem::take(&mut self.text_path_range_list);
        MakeGarbageCollected(result)
    }

    // cpp: layoutng_svg/svg_text_layout_attributes_builder.cc:152-155
    pub fn IfcTextContentOffsetAt(&self, index: usize) -> u32 {
        self.ifc_text_content_offsets[index]
    }

    // cpp: layoutng_svg/svg_text_layout_attributes_builder.h:44-46
    pub fn ResolvedCharacterCount(&self) -> usize {
        self.ifc_text_content_offsets.len()
    }
}
