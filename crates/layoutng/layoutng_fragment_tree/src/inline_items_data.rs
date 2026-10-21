use std::ops::Deref;

use foundation::{HeapVector, Member, String, TextOffsetMap, Visitor};
use layoutng::internal::inline_item::{InlineItem, InlineItems};
use layoutng::internal::inline_item_segment::InlineItemSegments;
use layoutng::internal::inline_item_text_index::InlineItemTextIndex;
use layoutng::internal::inline_node_data::InlineNodeData;
use layoutng::internal::svg_inline_node_data::SvgInlineNodeData;
use layoutng_inline::offset_mapping::OffsetMapping;

// C++ stores this discriminator in one byte and uses it for checked casts.
// cpp: layoutng_fragment_tree/inline_items_data.h:82-87
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InlineItemsDataType {
    kBase,
    kNodeData,
    kWithOffsetMap,
}

// The type tag is kept in the base object. Derived Rust structs place the
// base first, preserving the C++ base-pointer downcast contract.
// cpp: layoutng_fragment_tree/inline_items_data.h:23-25
// cpp: layoutng_fragment_tree/inline_items_data.h:31-43
// cpp: layoutng_fragment_tree/inline_items_data.h:98-100
#[repr(C)]
pub struct InlineItemsData {
    pub text_content: String,
    pub items: InlineItems,
    pub segments: Member<InlineItemSegments>,
    pub offset_mapping: Member<OffsetMapping>,
    data_type_: InlineItemsDataType,
}

impl Default for InlineItemsData {
    // cpp: layoutng_fragment_tree/inline_items_data.h:77
    fn default() -> Self {
        Self::new(InlineItemsDataType::kBase)
    }
}

// cpp: layoutng_fragment_tree/inline_items_data.h:62-67
pub type OpenTagItems = HeapVector<Member<InlineItem>, 16>;

#[allow(non_snake_case)]
impl InlineItemsData {
    // cpp: layoutng_fragment_tree/inline_items_data.h:92-92
    // Derived data lives in other Bazel packages, which are separate Rust crates.
    pub fn new(data_type: InlineItemsDataType) -> Self {
        Self {
            text_content: String::default(),
            items: InlineItems::default(),
            segments: Member::default(),
            offset_mapping: Member::default(),
            data_type_: data_type,
        }
    }

    // cpp: layoutng_fragment_tree/inline_items_data.h:27-29
    pub fn End(&self) -> InlineItemTextIndex {
        InlineItemTextIndex {
            item_index: self.items.len() as u32,
            text_offset: self.text_content.length() as u32,
        }
    }

    // cpp: layoutng_fragment_tree/inline_items_data.h:45-47
    pub fn IsValidOffset(&self, index: u32, offset: u32) -> bool {
        index < self.items.len() as u32
            && unsafe { &*self.items[index as usize].Get() }.IsValidOffset(offset)
    }

    // cpp: layoutng_fragment_tree/inline_items_data.h:48-50
    pub fn IsValidOffsetAt(&self, index: &InlineItemTextIndex) -> bool {
        self.IsValidOffset(index.item_index, index.text_offset)
    }

    // cpp: layoutng_fragment_tree/inline_items_data.h:52-54
    pub fn AssertOffset(&self, index: u32, offset: u32) {
        unsafe { &*self.items[index as usize].Get() }.AssertOffset(offset);
    }

    // cpp: layoutng_fragment_tree/inline_items_data.h:55-57
    pub fn AssertOffsetAt(&self, index: &InlineItemTextIndex) {
        self.AssertOffset(index.item_index, index.text_offset);
    }

    // cpp: layoutng_fragment_tree/inline_items_data.h:58-60
    pub fn AssertEndOffset(&self, index: u32, offset: u32) {
        unsafe { &*self.items[index as usize].Get() }.AssertEndOffset(offset);
    }

    // cpp: layoutng_fragment_tree/inline_items_data.h:65-75
    // The bodies are exported by //src/layoutng_inline after shared assembly.
    pub fn GetOpenTagItems(&self, start_index: u32, size: u32, open_items: &mut OpenTagItems) {
        unsafe { InlineItemsDataGetOpenTagItems(self, start_index, size, open_items) }
    }

    pub fn OffsetMap(&self) -> &Option<TextOffsetMap> {
        unsafe { InlineItemsDataOffsetMap(self) }
    }

    #[cfg(debug_assertions)]
    pub fn CheckConsistency(&self) {
        unsafe { InlineItemsDataCheckConsistency(self) }
    }

    // cpp: layoutng_fragment_tree/inline_items_data.h:79
    // cpp: layoutng_fragment_tree/inline_items_data.cc:12-21
    pub fn Trace(&self, visitor: &mut Visitor) {
        if self.IsNodeData() {
            let node_data = unsafe { &*(self as *const Self as *const InlineNodeData) };
            node_data.TraceAfterDispatch(visitor);
        } else if self.IsWithOffsetMap() {
            let with_offset =
                unsafe { &*(self as *const Self as *const InlineItemsDataWithOffsetMap) };
            with_offset.TraceAfterDispatch(visitor);
        } else {
            self.TraceAfterDispatch(visitor);
        }
    }

    // cpp: layoutng_fragment_tree/inline_items_data.h:80
    // cpp: layoutng_fragment_tree/inline_items_data.cc:23-27
    pub fn TraceAfterDispatch(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.items);
        visitor.Trace(&self.segments);
        visitor.Trace(&self.offset_mapping);
    }

    // cpp: layoutng_fragment_tree/inline_items_data.h:93
    pub fn IsNodeData(&self) -> bool {
        self.data_type_ == InlineItemsDataType::kNodeData
    }

    // cpp: layoutng_fragment_tree/inline_items_data.h:94-96
    pub fn IsWithOffsetMap(&self) -> bool {
        self.data_type_ == InlineItemsDataType::kWithOffsetMap
    }
}

unsafe extern "Rust" {
    fn InlineItemsDataGetOpenTagItems(
        data: &InlineItemsData,
        start_index: u32,
        size: u32,
        open_items: &mut OpenTagItems,
    );
    fn InlineItemsDataOffsetMap(data: &InlineItemsData) -> &Option<TextOffsetMap>;
    #[cfg(debug_assertions)]
    fn InlineItemsDataCheckConsistency(data: &InlineItemsData);
}

// The C++ body belongs to this source file but is a member of a type owned
// by the later layoutng package. Its Rust method can call this helper with
// private fields, keeping those fields private across crate boundaries.
// cpp: layoutng_fragment_tree/inline_items_data.cc:29-33
#[allow(non_snake_case)]
pub fn TraceInlineNodeDataAfterDispatch(
    base: &InlineItemsData,
    first_line_items: &Member<InlineItemsData>,
    svg_node_data: &Member<SvgInlineNodeData>,
    visitor: &mut Visitor,
) {
    visitor.Trace(first_line_items);
    visitor.Trace(svg_node_data);
    base.TraceAfterDispatch(visitor);
}

// cpp: layoutng_fragment_tree/inline_items_data.h:102-113
#[repr(C)]
pub struct InlineItemsDataWithOffsetMap {
    pub base_: InlineItemsData,
    pub offset_map: Option<TextOffsetMap>,
}

impl Default for InlineItemsDataWithOffsetMap {
    // cpp: layoutng_fragment_tree/inline_items_data.h:106
    fn default() -> Self {
        Self {
            base_: InlineItemsData::new(InlineItemsDataType::kWithOffsetMap),
            offset_map: None,
        }
    }
}

impl Deref for InlineItemsDataWithOffsetMap {
    type Target = InlineItemsData;

    fn deref(&self) -> &Self::Target {
        &self.base_
    }
}

impl foundation::Traceable for InlineItemsDataWithOffsetMap {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.TraceAfterDispatch(visitor);
    }
}

#[allow(non_snake_case)]
impl InlineItemsDataWithOffsetMap {
    // cpp: layoutng_fragment_tree/inline_items_data.h:108-110
    pub fn TraceAfterDispatch(&self, visitor: &mut Visitor) {
        self.base_.TraceAfterDispatch(visitor);
    }

    // cpp: layoutng_fragment_tree/inline_items_data.h:115-120
    pub fn AllowFrom(value: &InlineItemsData) -> bool {
        value.IsWithOffsetMap()
    }
}
