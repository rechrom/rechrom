#![allow(non_snake_case, non_upper_case_globals)]

use foundation::String;

use super::layout_locale::LineBreakStrictness;
use super::line_break_rule_data::{
    kCategoryCount, kLookAheadCount, kNormalRanges, kRows, kStateCount, kStrictRanges, Range,
};

// This private helper is defined in text_break_iterator.cc, while the public
// function declaration lives in line_break_rule_engine.h.
// cpp: font_engine/text/native/text_break_iterator.cc:32-44
pub(super) fn CodePointAt(text: &String, offset: u32, limit: u32, next: &mut u32) -> u32 {
    let units = text.Span16().expect("non-null line-break text");
    let mut value = u32::from(units[offset as usize]);
    *next = offset + 1;
    if (0xD800..=0xDBFF).contains(&value) && *next < limit {
        let trail = u32::from(units[*next as usize]);
        if (0xDC00..=0xDFFF).contains(&trail) {
            value = 0x10000 + ((value - 0xD800) << 10) + (trail - 0xDC00);
            *next += 1;
        }
    }
    value
}

// cpp: font_engine/text/native/text_break_iterator.cc:46-60
fn Category(code_point: u32, ranges: &[Range]) -> u16 {
    let mut first = 0;
    let mut last = ranges.len();
    while first < last {
        let middle = first + (last - first) / 2;
        if code_point < ranges[middle].first {
            last = middle;
        } else if code_point > ranges[middle].last {
            first = middle + 1;
        } else {
            return u16::from(ranges[middle].category);
        }
    }
    0
}

// cpp: font_engine/text/native/text_break_iterator.cc:62-67
fn RuleCategory(code_point: u32, strictness: LineBreakStrictness) -> u16 {
    if strictness == LineBreakStrictness::kStrict {
        return Category(code_point, &kStrictRanges);
    }
    Category(code_point, &kNormalRanges)
}

// cpp: font_engine/text/native/line_break_rule_engine.h:9-14
// cpp: font_engine/text/native/text_break_iterator.cc:73-126
pub fn NextLineBreakRuleBoundary(
    text: &String,
    initial: u32,
    limit: u32,
    strictness: LineBreakStrictness,
) -> u32 {
    if initial >= limit {
        return limit;
    }
    const K_ROW_WIDTH: usize = 3 + kCategoryCount as usize;
    let mut lookahead = [-1_i32; kLookAheadCount as usize];
    let mut index = 0;
    let mut code_point = CodePointAt(text, initial, limit, &mut index);
    let mut result = initial;
    let mut state: u16 = 1;
    let mut at_end = false;
    loop {
        let category = if at_end {
            1
        } else {
            RuleCategory(code_point, strictness)
        };
        assert!(category < kCategoryCount);
        let mut row = &kRows[state as usize * K_ROW_WIDTH..][..K_ROW_WIDTH];
        state = row[3 + category as usize];
        assert!(state < kStateCount);
        row = &kRows[state as usize * K_ROW_WIDTH..][..K_ROW_WIDTH];
        let accepting = row[0];
        if accepting == 1 {
            result = index;
        } else if accepting > 1
            && (accepting as usize) < lookahead.len()
            && lookahead[accepting as usize] >= 0
        {
            return lookahead[accepting as usize] as u32;
        }
        let rule = row[1];
        if rule > 1 && (rule as usize) < lookahead.len() {
            lookahead[rule as usize] = index as i32;
        }
        if state == 0 {
            break;
        }
        if at_end {
            break;
        }
        if index == limit {
            at_end = true;
        } else {
            code_point = CodePointAt(text, index, limit, &mut index);
        }
    }
    if result == initial {
        let mut next = 0;
        CodePointAt(text, initial, limit, &mut next);
        return next;
    }
    result
}
