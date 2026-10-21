#![allow(non_snake_case)]

use super::svg_character_data::SvgCharacterData;

// cpp: layoutng/internal/resolved_text_layout_attributes_iterator.h:21-71
pub struct ResolvedTextLayoutAttributesIterator<'a> {
    resolved_: &'a [(u32, SvgCharacterData)],
    default_data_: SvgCharacterData,
    index_: usize,
}

impl<'a> ResolvedTextLayoutAttributesIterator<'a> {
    // cpp: layoutng/internal/resolved_text_layout_attributes_iterator.h:36-38
    pub fn new(resolved: &'a [(u32, SvgCharacterData)]) -> Self {
        Self {
            resolved_: resolved,
            default_data_: SvgCharacterData::default(),
            index_: 0,
        }
    }

    // cpp: layoutng/internal/resolved_text_layout_attributes_iterator.h:44-63
    pub fn AdvanceTo(&mut self, addressable_index: u32) -> &SvgCharacterData {
        if self.index_ >= self.resolved_.len() {
            return &self.default_data_;
        }
        if addressable_index < self.resolved_[self.index_].0 {
            return &self.default_data_;
        }
        if addressable_index == self.resolved_[self.index_].0 {
            return &self.resolved_[self.index_].1;
        }

        let resolved_range = &self.resolved_[self.index_..];
        let next = resolved_range
            .iter()
            .position(|pair| addressable_index <= pair.0)
            .unwrap_or(resolved_range.len());
        self.index_ += next;
        self.AdvanceTo(addressable_index)
    }
}
