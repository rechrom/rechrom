#![allow(non_snake_case, non_upper_case_globals)]

use super::grapheme_break_property_data::{
    GraphemeBreakProperty, GraphemeProperty, IndicConjunctBreak, IndicConjunctProperty,
    IsExtendedPictographic,
};
use foundation::StringView;
use std::cell::Cell;

// cpp: font_engine/text/native/character_break_iterator.h:38-38
pub const kTextBreakDone: i32 = -1;

// cpp: font_engine/text/native/character_break_iterator.cc:28-54
struct DecodedText {
    code_points: Vec<u32>,
    ends: Vec<u32>,
}

fn Decode<F: Fn(u32) -> u16>(length: u32, is_8bit: bool, at: F) -> DecodedText {
    let mut result = DecodedText {
        code_points: Vec::with_capacity(length as usize),
        ends: Vec::with_capacity(length as usize),
    };
    let mut offset = 0;
    while offset < length {
        let mut code_point = u32::from(at(offset));
        offset += 1;
        if !is_8bit && (0xD800..=0xDBFF).contains(&code_point) && offset < length {
            let trail = u32::from(at(offset));
            if (0xDC00..=0xDFFF).contains(&trail) {
                code_point = 0x10000 + ((code_point - 0xD800) << 10) + (trail - 0xDC00);
                offset += 1;
            }
        }
        result.code_points.push(code_point);
        result.ends.push(offset);
    }
    result
}

// cpp: font_engine/text/native/character_break_iterator.cc:56-60
fn IsControl(property: GraphemeProperty) -> bool {
    property == GraphemeProperty::kControl
        || property == GraphemeProperty::kCr
        || property == GraphemeProperty::kLf
}

// cpp: font_engine/text/native/character_break_iterator.cc:62-77
fn HasIndicConjunctBefore(code_points: &[u32], right_index: usize) -> bool {
    let mut saw_linker = false;
    let mut i = right_index;
    while i > 0 {
        i -= 1;
        let property = IndicConjunctProperty(code_points[i]);
        if property == IndicConjunctBreak::kLinker {
            saw_linker = true;
            continue;
        }
        if property == IndicConjunctBreak::kExtend {
            continue;
        }
        return saw_linker && property == IndicConjunctBreak::kConsonant;
    }
    false
}

// cpp: font_engine/text/native/character_break_iterator.cc:79-95
fn HasExtendedPictographicBeforeZwj(code_points: &[u32], right_index: usize) -> bool {
    if right_index == 0
        || GraphemeBreakProperty(code_points[right_index - 1]) != GraphemeProperty::kZwj
    {
        return false;
    }
    let mut i = right_index - 1;
    while i > 0 && GraphemeBreakProperty(code_points[i - 1]) == GraphemeProperty::kExtend {
        i -= 1;
    }
    i > 0 && IsExtendedPictographic(code_points[i - 1])
}

// cpp: font_engine/text/native/character_break_iterator.cc:97-159
fn ShouldBreak(code_points: &[u32], right_index: usize) -> bool {
    debug_assert!(right_index > 0 && right_index < code_points.len());
    let left = GraphemeBreakProperty(code_points[right_index - 1]);
    let right = GraphemeBreakProperty(code_points[right_index]);
    if left == GraphemeProperty::kCr && right == GraphemeProperty::kLf {
        return false;
    }
    if IsControl(left) || IsControl(right) {
        return true;
    }
    if left == GraphemeProperty::kL
        && matches!(
            right,
            GraphemeProperty::kL
                | GraphemeProperty::kV
                | GraphemeProperty::kLv
                | GraphemeProperty::kLvt
        )
    {
        return false;
    }
    if matches!(left, GraphemeProperty::kLv | GraphemeProperty::kV)
        && matches!(right, GraphemeProperty::kV | GraphemeProperty::kT)
    {
        return false;
    }
    if matches!(left, GraphemeProperty::kLvt | GraphemeProperty::kT)
        && right == GraphemeProperty::kT
    {
        return false;
    }
    if matches!(
        right,
        GraphemeProperty::kExtend | GraphemeProperty::kZwj | GraphemeProperty::kSpacingMark
    ) {
        return false;
    }
    if left == GraphemeProperty::kPrepend {
        return false;
    }
    if IndicConjunctProperty(code_points[right_index]) == IndicConjunctBreak::kConsonant
        && HasIndicConjunctBefore(code_points, right_index)
    {
        return false;
    }
    if IsExtendedPictographic(code_points[right_index])
        && HasExtendedPictographicBeforeZwj(code_points, right_index)
    {
        return false;
    }
    if left == GraphemeProperty::kRegionalIndicator && right == GraphemeProperty::kRegionalIndicator
    {
        let mut count = 0;
        let mut i = right_index;
        while i > 0
            && GraphemeBreakProperty(code_points[i - 1]) == GraphemeProperty::kRegionalIndicator
        {
            count += 1;
            i -= 1;
        }
        return count % 2 == 0;
    }
    true
}

// cpp: font_engine/text/native/character_break_iterator.cc:161-172
fn BuildBoundaries(text: &DecodedText) -> Vec<u32> {
    let mut boundaries = Vec::with_capacity(text.code_points.len() + 1);
    boundaries.push(0);
    for i in 1..text.code_points.len() {
        if ShouldBreak(&text.code_points, i) {
            boundaries.push(text.ends[i - 1]);
        }
    }
    if let Some(end) = text.ends.last() {
        boundaries.push(*end);
    }
    boundaries
}

// cpp: font_engine/text/native/character_break_iterator.h:44-68
pub struct CharacterBreakIterator {
    boundaries_: Vec<u32>,
    boundary_index_: Cell<usize>,
    length_: u32,
}

// cpp: font_engine/text/native/character_break_iterator.h:61-61
impl std::ops::Not for &CharacterBreakIterator {
    type Output = bool;
    fn not(self) -> bool {
        false
    }
}

impl CharacterBreakIterator {
    // cpp: font_engine/text/native/character_break_iterator.cc:176-194
    pub fn new(text: &StringView) -> Self {
        let length = text.length();
        // The Rust StringView stores UTF-16 code units for both source string
        // representations. Latin-1 units cannot contain surrogate values.
        let boundaries = BuildBoundaries(&Decode(length, false, |i| text[i as usize]));
        Self {
            boundaries_: boundaries,
            boundary_index_: Cell::new(0),
            length_: length,
        }
    }

    pub fn from_utf16(text: &[u16]) -> Self {
        let length = u32::try_from(text.len()).expect("UTF-16 span exceeds 32 bits");
        let boundaries = BuildBoundaries(&Decode(length, false, |i| text[i as usize]));
        Self {
            boundaries_: boundaries,
            boundary_index_: Cell::new(0),
            length_: length,
        }
    }

    // cpp: font_engine/text/native/character_break_iterator.cc:196-204
    pub fn Next(&mut self) -> i32 {
        let next = self.boundary_index_.get() + 1;
        if next >= self.boundaries_.len() {
            return kTextBreakDone;
        }
        self.boundary_index_.set(next);
        self.boundaries_[next] as i32
    }

    pub fn Current(&self) -> i32 {
        self.boundaries_[self.boundary_index_.get()] as i32
    }

    // cpp: font_engine/text/native/character_break_iterator.cc:206-230
    pub fn IsBreak(&self, offset: i32) -> bool {
        offset >= 0 && self.boundaries_.binary_search(&(offset as u32)).is_ok()
    }

    pub fn Preceding(&self, offset: i32) -> i32 {
        let target = offset.max(0) as u32;
        let lower = self
            .boundaries_
            .partition_point(|boundary| *boundary < target);
        if lower == 0 {
            return kTextBreakDone;
        }
        self.boundary_index_.set(lower - 1);
        self.boundaries_[lower - 1] as i32
    }

    pub fn Following(&self, offset: i32) -> i32 {
        let target = offset.max(0) as u32;
        let upper = self
            .boundaries_
            .partition_point(|boundary| *boundary <= target);
        if upper == self.boundaries_.len() {
            return kTextBreakDone;
        }
        self.boundary_index_.set(upper);
        self.boundaries_[upper] as i32
    }
}

// cpp: font_engine/text/native/character_break_iterator.cc:232-240
pub fn NumGraphemeClusters(text: &StringView) -> u32 {
    if text.IsEmpty() {
        return 0;
    }
    let mut iterator = CharacterBreakIterator::new(text);
    let mut count = 0;
    while iterator.Next() != kTextBreakDone {
        count += 1;
    }
    count
}

// cpp: font_engine/text/native/character_break_iterator.cc:242-251
pub fn LengthOfGraphemeCluster(text: &StringView, offset: u32) -> u32 {
    assert!(offset <= text.length());
    if offset == text.length() {
        return 0;
    }
    let iterator = CharacterBreakIterator::new(text);
    let following = iterator.Following(offset as i32);
    if following == kTextBreakDone {
        text.length() - offset
    } else {
        following as u32 - offset
    }
}

// cpp: font_engine/text/native/character_break_iterator.cc:253-268
pub fn GraphemesClusterList(text: &StringView, graphemes: &mut [u32]) {
    debug_assert_eq!(text.length() as usize, graphemes.len());
    if text.IsEmpty() {
        return;
    }
    let mut iterator = CharacterBreakIterator::new(text);
    let mut cluster = 0;
    let mut offset = 0;
    let mut boundary = iterator.Next();
    while boundary != kTextBreakDone {
        while offset < boundary as usize && offset < graphemes.len() {
            graphemes[offset] = cluster;
            offset += 1;
        }
        boundary = iterator.Next();
        cluster += 1;
    }
}
