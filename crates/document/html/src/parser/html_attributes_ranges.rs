#![allow(non_snake_case)]

use super::html_parser_idioms::kAttributePrealloc;

// cpp: html/parser/html_attributes_ranges.h:23-52
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Range {
    pub start: i32,
    pub end: i32,
}

impl Default for Range {
    fn default() -> Self {
        // C++ leaves these integers uninitialized until their tokenizer phase;
        // Rust definite initialization uses the same invalid sentinel.
        Self { start: -1, end: -1 }
    }
}

impl Range {
    pub const kInvalidOffset: i32 = -1;

    // cpp: html/parser/html_attributes_ranges.h:29-34
    pub fn Clear(&mut self) {
        #[cfg(debug_assertions)]
        {
            self.start = Self::kInvalidOffset;
            self.end = Self::kInvalidOffset;
        }
    }

    // cpp: html/parser/html_attributes_ranges.h:36-40
    pub fn CheckValidStart(&self) {
        debug_assert_ne!(self.start, Self::kInvalidOffset);
        debug_assert!(self.start >= 0);
    }

    // cpp: html/parser/html_attributes_ranges.h:42-48
    pub fn CheckValid(&self) {
        self.CheckValidStart();
        debug_assert_ne!(self.end, Self::kInvalidOffset);
        debug_assert!(self.end >= 0);
        debug_assert!(self.start <= self.end);
    }
}

// cpp: html/parser/html_attributes_ranges.h:54-57
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Attribute {
    pub name_range: Range,
    pub value_range: Range,
}

// cpp: html/parser/html_attributes_ranges.h:59-100
pub struct HTMLAttributesRanges {
    attributes_: Vec<Attribute>,
    current_attribute_: Option<usize>,
}

impl Default for HTMLAttributesRanges {
    fn default() -> Self {
        Self {
            attributes_: Vec::with_capacity(kAttributePrealloc),
            current_attribute_: None,
        }
    }
}

impl HTMLAttributesRanges {
    // cpp: html/parser/html_attributes_ranges.h:61-64
    pub fn Clear(&mut self) {
        self.current_attribute_ = None;
        self.attributes_.clear();
    }

    // cpp: html/parser/html_attributes_ranges.h:66-71
    pub fn AddAttribute(&mut self, offset: i32) {
        self.attributes_.push(Attribute::default());
        self.current_attribute_ = Some(self.attributes_.len() - 1);
        let current = self.current_attribute_.unwrap();
        self.attributes_[current].name_range.start = offset;
        self.attributes_[current].name_range.CheckValidStart();
    }

    // cpp: html/parser/html_attributes_ranges.h:73-79
    pub fn EndAttributeName(&mut self, offset: i32) {
        let current = self.current_attribute_.expect("current attribute required");
        let attribute = &mut self.attributes_[current];
        attribute.name_range.end = offset;
        attribute.name_range.CheckValid();
        attribute.value_range.start = offset;
        attribute.value_range.end = offset;
    }

    // cpp: html/parser/html_attributes_ranges.h:81-86
    pub fn BeginAttributeValue(&mut self, offset: i32) {
        let current = self.current_attribute_.expect("current attribute required");
        let attribute = &mut self.attributes_[current];
        attribute.value_range.Clear();
        attribute.value_range.start = offset;
        attribute.value_range.CheckValidStart();
    }

    // cpp: html/parser/html_attributes_ranges.h:88-92
    pub fn EndAttributeValue(&mut self, offset: i32) {
        let current = self.current_attribute_.expect("current attribute required");
        let attribute = &mut self.attributes_[current];
        attribute.value_range.end = offset;
        attribute.value_range.CheckValid();
    }

    // cpp: html/parser/html_attributes_ranges.h:94-94
    pub fn attributes(&self) -> &[Attribute] {
        &self.attributes_
    }
}
