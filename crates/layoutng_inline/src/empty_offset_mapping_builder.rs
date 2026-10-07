// C++: layoutng_inline/empty_offset_mapping_builder.h.
// The source deliberately performs no mapping for the builder template's
// no-offset-map instantiation. These empty methods are its actual behavior.
#![allow(non_snake_case)]

use layoutng::internal::layout_text::LayoutText;
use std::ffi::c_void;

// cpp: layoutng_inline/empty_offset_mapping_builder.h:18-40
#[derive(Default)]
pub struct EmptyOffsetMappingBuilder;

// cpp: layoutng_inline/empty_offset_mapping_builder.h:22-26
pub struct SourceNodeScope;

impl SourceNodeScope {
    // cpp: layoutng_inline/empty_offset_mapping_builder.h:24-24
    pub fn new(_builder: &mut EmptyOffsetMappingBuilder, _node: *const c_void) -> Self {
        Self
    }
}

impl EmptyOffsetMappingBuilder {
    // cpp: layoutng_inline/empty_offset_mapping_builder.h:28-31
    pub fn new() -> Self {
        Self
    }

    // cpp: layoutng_inline/empty_offset_mapping_builder.h:32-32
    pub fn AppendIdentityMapping(&mut self, _length: u32) {}

    // cpp: layoutng_inline/empty_offset_mapping_builder.h:33-33
    pub fn RevertIdentityMapping1(&mut self) {}

    // cpp: layoutng_inline/empty_offset_mapping_builder.h:34-34
    pub fn AppendCollapsedMapping(&mut self, _length: u32) {}

    // cpp: layoutng_inline/empty_offset_mapping_builder.h:35-35
    pub fn AppendVariableMapping(&mut self, _source_length: u32, _target_length: u32) {}

    // cpp: layoutng_inline/empty_offset_mapping_builder.h:36-36
    pub fn CollapseTrailingSpace(&mut self, _space_length: u32) {}

    // cpp: layoutng_inline/empty_offset_mapping_builder.h:37-37
    pub fn Composite(&mut self, _other: &Self) {}

    // cpp: layoutng_inline/empty_offset_mapping_builder.h:38-38
    pub fn Concatenate(&mut self, _other: &Self) {}

    // cpp: layoutng_inline/empty_offset_mapping_builder.h:39-39
    pub fn RestoreTrailingCollapsibleSpace(&mut self, _text: &LayoutText, _offset: u32) {}
}
