#![allow(non_snake_case)]

use foundation::{IsLtr, LayoutUnit, RuntimeEnabledFeatures, String};

use super::shape_options::ShapeOptions;
use super::shape_result::ShapeResult;
use super::shape_result_view::{Segment, ShapeResultView};
use super::text_spacing_trim::{ShouldTrimEnd, ShouldTrimStartOfWrappedLine, TextSpacingTrim};
use crate::fonts::font::Font;
use crate::text::native::character::Character;
use crate::text::native::hyphenation::Hyphenation;
use crate::text::native::text_break_iterator::{BreakSpaceType, LazyLineBreakIterator};

// C++'s pure virtual Shape has no body in this package. A backend supplied by
// the derived owner preserves virtual dispatch without inventing shaping.
// cpp: font_engine/fonts/shaping/shaping_line_breaker.h:93-95
pub trait ShapeLineBackend {
    fn Shape(&self, start: u32, end: u32, options: ShapeOptions) -> *const ShapeResult;
}

// cpp: font_engine/fonts/shaping/shaping_line_breaker.h:47-63
#[derive(Clone, Copy, Debug, Default)]
pub struct Result {
    pub break_offset: u32,
    pub has_trailing_spaces: bool,
    pub is_overflow: bool,
    pub is_hyphenated: bool,
}

// cpp: font_engine/fonts/shaping/shaping_line_breaker.h:98-101
#[derive(Clone, Copy, Debug, Default)]
struct EdgeOffset {
    offset: u32,
    han_kerning: bool,
}

// cpp: font_engine/fonts/shaping/shaping_line_breaker.h:111-129
#[derive(Clone, Copy, Debug, Default)]
struct BreakOpportunity {
    offset: u32,
    non_hangable_run_end: Option<u32>,
    is_hyphenated: bool,
}

impl BreakOpportunity {
    fn new(offset: u32, is_hyphenated: bool) -> Self {
        Self {
            offset,
            non_hangable_run_end: None,
            is_hyphenated,
        }
    }
    fn with_run_end(offset: u32, run_end: u32, is_hyphenated: bool) -> Self {
        Self {
            offset,
            non_hangable_run_end: Some(run_end),
            is_hyphenated,
        }
    }
}

// cpp: font_engine/fonts/shaping/shaping_line_breaker.cc:31-35
fn FlipRtl<const RTL: bool>(value: LayoutUnit) -> LayoutUnit {
    if RTL {
        -value
    } else {
        value
    }
}

// cpp: font_engine/fonts/shaping/shaping_line_breaker.cc:37-40
fn IsBreakableSpace(ch: u16) -> bool {
    LazyLineBreakIterator::IsBreakableSpace(ch) || Character::IsOtherSpaceSeparator(i32::from(ch))
}

// cpp: font_engine/fonts/shaping/shaping_line_breaker.cc:42-45
fn IsAllSpaces(text: &String, start: u32, end: u32) -> bool {
    text.Span16().expect("non-null line text")[start as usize..end as usize]
        .iter()
        .copied()
        .all(IsBreakableSpace)
}

// cpp: font_engine/fonts/shaping/shaping_line_breaker.cc:47-61
fn ShouldHyphenate(text: &String, word_start: u32, word_end: u32, line_start: u32) -> bool {
    if word_start <= line_start {
        return true;
    }
    if IsAllSpaces(text, word_end, text.length()) {
        return IsAllSpaces(text, 0, word_start);
    }
    true
}

// cpp: font_engine/fonts/shaping/shaping_line_breaker.cc:63-69
fn CheckBreakOffset(offset: u32, start: u32, end: u32) {
    assert!(offset > start);
    assert!(offset <= end);
}

// cpp: font_engine/fonts/shaping/shaping_line_breaker.cc:71-85
fn FindNonHangableEnd(text: &String, candidate: u32) -> u32 {
    debug_assert!(candidate < text.length());
    debug_assert!(IsBreakableSpace(text.CodeUnitAt(candidate)));
    let mut non_hangable_end = candidate;
    while non_hangable_end > 0 {
        non_hangable_end -= 1;
        if !IsBreakableSpace(text.CodeUnitAt(non_hangable_end)) {
            return non_hangable_end + 1;
        }
    }
    non_hangable_end
}

// C++ deleted copying is represented by omitting Clone/Copy.
// cpp: font_engine/fonts/shaping/shaping_line_breaker.h:33-168
pub struct ShapingLineBreaker<'a> {
    result_: &'a ShapeResult,
    break_iterator_: &'a LazyLineBreakIterator,
    hyphenation_: Option<&'a Hyphenation>,
    font_: &'a Font,
    backend_: &'a dyn ShapeLineBackend,
    line_start_: u32,
    dont_reshape_end_if_at_space_: bool,
    no_result_if_overflow_: bool,
    is_after_forced_break_: bool,
    text_spacing_trim_: TextSpacingTrim,
}

impl<'a> ShapingLineBreaker<'a> {
    // cpp: font_engine/fonts/shaping/shaping_line_breaker.cc:17-27
    pub fn new(
        result: &'a ShapeResult,
        break_iterator: &'a LazyLineBreakIterator,
        hyphenation: Option<&'a Hyphenation>,
        font: &'a Font,
        backend: &'a dyn ShapeLineBackend,
    ) -> Self {
        Self {
            result_: result,
            break_iterator_: break_iterator,
            hyphenation_: hyphenation,
            font_: font,
            backend_: backend,
            line_start_: 0,
            dont_reshape_end_if_at_space_: false,
            no_result_if_overflow_: false,
            is_after_forced_break_: false,
            text_spacing_trim_: TextSpacingTrim::kInitial,
        }
    }

    // cpp: font_engine/fonts/shaping/shaping_line_breaker.h:67-82
    pub fn SetLineStart(&mut self, offset: u32) {
        self.line_start_ = offset;
    }
    pub fn SetDontReshapeEndIfAtSpace(&mut self) {
        self.dont_reshape_end_if_at_space_ = true;
    }
    pub fn NoResultIfOverflow(&self) -> bool {
        self.no_result_if_overflow_
    }
    pub fn SetNoResultIfOverflow(&mut self) {
        self.no_result_if_overflow_ = true;
    }
    pub fn SetIsAfterForcedBreak(&mut self, value: bool) {
        self.is_after_forced_break_ = value;
    }
    pub fn SetTextSpacingTrim(&mut self, value: TextSpacingTrim) {
        self.text_spacing_trim_ = value;
    }
    pub fn GetShapeResult(&self) -> &ShapeResult {
        self.result_
    }

    // cpp: font_engine/fonts/shaping/shaping_line_breaker.cc:89-91
    fn GetText(&self) -> &String {
        self.break_iterator_.GetString()
    }

    // cpp: font_engine/fonts/shaping/shaping_line_breaker.h:105-107
    fn IsStartOfWrappedLine(&self, offset: u32) -> bool {
        offset != 0 && offset == self.line_start_ && !self.is_after_forced_break_
    }

    // cpp: font_engine/fonts/shaping/shaping_line_breaker.cc:93-115
    fn FirstSafeOffset(&self, mut start: u32) -> EdgeOffset {
        if !self.IsStartOfWrappedLine(start) {
            return EdgeOffset {
                offset: start,
                han_kerning: false,
            };
        }
        let text = self.GetText();
        if ShouldTrimStartOfWrappedLine(self.text_spacing_trim_)
            && Character::MaybeHanKerningOpen(i32::from(text.CodeUnitAt(start)))
        {
            start += 1;
            if start < self.result_.EndIndex() {
                return EdgeOffset {
                    offset: self.result_.CachedNextSafeToBreakOffset(start),
                    han_kerning: true,
                };
            }
            return EdgeOffset {
                offset: start,
                han_kerning: true,
            };
        }
        EdgeOffset {
            offset: self.result_.CachedNextSafeToBreakOffset(start),
            han_kerning: false,
        }
    }

    // cpp: font_engine/fonts/shaping/shaping_line_breaker.cc:117-150
    fn HyphenateWord(&self, offset: u32, word_start: u32, word_end: u32, backwards: bool) -> u32 {
        let hyphenation = self.hyphenation_.expect("hyphenation object required");
        debug_assert!(word_end > word_start && offset >= word_start && offset <= word_end);
        let word_len = word_end - word_start;
        if word_len < hyphenation.MinWordLength() {
            return 0;
        }
        let text = self.GetText();
        let units = text.Span16().expect("non-null line text");
        let word = String::from_utf16(&units[word_start as usize..word_end as usize]);
        let word_offset = offset - word_start;
        if backwards {
            if word_offset < hyphenation.MinPrefixLength() {
                return 0;
            }
            let prefix_length = hyphenation.LastHyphenLocation(&word, word_offset + 1);
            debug_assert!(prefix_length == 0 || prefix_length <= word_offset);
            prefix_length
        } else {
            if word_len - word_offset < hyphenation.MinSuffixLength() {
                return 0;
            }
            let prefix_length = hyphenation
                .FirstHyphenLocation(&word, if word_offset != 0 { word_offset - 1 } else { 0 });
            debug_assert!(prefix_length == 0 || prefix_length >= word_offset);
            prefix_length
        }
    }

    // cpp: font_engine/fonts/shaping/shaping_line_breaker.cc:152-172
    fn Hyphenate(&self, offset: u32, start: u32, backwards: bool) -> BreakOpportunity {
        let text = self.GetText();
        let mut word_end = self.break_iterator_.NextBreakOpportunity(offset);
        if word_end != offset && IsBreakableSpace(text.CodeUnitAt(word_end.wrapping_sub(1))) {
            word_end = offset.max(FindNonHangableEnd(text, word_end - 1));
        }
        if word_end == offset {
            debug_assert!(
                IsBreakableSpace(text.CodeUnitAt(offset))
                    || offset == self.break_iterator_.PreviousBreakOpportunity(offset, start)
            );
            return BreakOpportunity::new(word_end, false);
        }
        let previous_break_opportunity =
            self.break_iterator_.PreviousBreakOpportunity(offset, start);
        let mut word_start = previous_break_opportunity;
        while word_start < text.length()
            && LazyLineBreakIterator::IsBreakableSpace(text.CodeUnitAt(word_start))
        {
            word_start += 1;
        }
        if offset >= word_start
            && ShouldHyphenate(text, previous_break_opportunity, word_end, start)
        {
            let prefix_length = self.HyphenateWord(offset, word_start, word_end, backwards);
            if prefix_length != 0 {
                return BreakOpportunity::new(word_start + prefix_length, true);
            }
        }
        BreakOpportunity::new(
            if backwards {
                previous_break_opportunity
            } else {
                word_end
            },
            false,
        )
    }

    // cpp: font_engine/fonts/shaping/shaping_line_breaker.cc:174-211
    fn PreviousBreakOpportunity(&self, offset: u32, start: u32) -> BreakOpportunity {
        if self.hyphenation_.is_some() {
            return self.Hyphenate(offset, start, true);
        }
        let text = self.GetText();
        let break_offset = self.break_iterator_.PreviousBreakOpportunity(offset, start);
        if IsBreakableSpace(text.CodeUnitAt(break_offset.wrapping_sub(1))) {
            return BreakOpportunity::with_run_end(
                break_offset,
                FindNonHangableEnd(text, break_offset - 1),
                false,
            );
        }
        BreakOpportunity::new(break_offset, false)
    }

    fn NextBreakOpportunity(&self, offset: u32, start: u32, len: u32) -> BreakOpportunity {
        if self.hyphenation_.is_some() {
            return self.Hyphenate(offset, start, false);
        }
        let text = self.GetText();
        let break_offset = self.break_iterator_.NextBreakOpportunityTo(offset, len);
        if IsBreakableSpace(text.CodeUnitAt(break_offset.wrapping_sub(1))) {
            return BreakOpportunity::with_run_end(
                break_offset,
                FindNonHangableEnd(text, break_offset - 1),
                false,
            );
        }
        BreakOpportunity::new(break_offset, false)
    }

    // cpp: font_engine/fonts/shaping/shaping_line_breaker.cc:213-230
    fn SetBreakOffset(&self, break_offset: u32, result: &mut Result) {
        result.break_offset = break_offset;
        let text = self.GetText();
        result.is_hyphenated = text.CodeUnitAt(break_offset.wrapping_sub(1)) == 0xAD;
    }

    fn SetBreakOpportunity(&self, opportunity: BreakOpportunity, result: &mut Result) {
        result.break_offset = opportunity.offset;
        let text = self.GetText();
        result.is_hyphenated = opportunity.is_hyphenated
            || text.CodeUnitAt(opportunity.offset.wrapping_sub(1)) == 0xAD;
    }

    // cpp: font_engine/fonts/shaping/shaping_line_breaker.cc:256-264
    pub fn ShapeLine(
        &self,
        start: u32,
        available_space: LayoutUnit,
        result_out: &mut Result,
    ) -> *const ShapeResultView {
        if IsLtr(self.result_.Direction()) {
            self.ShapeLineCore::<false>(start, available_space, result_out)
        } else {
            self.ShapeLineCore::<true>(start, available_space, result_out)
        }
    }

    // C++'s TextDirection template argument becomes a const bool selecting
    // the same logical/visual sign branch.
    // cpp: font_engine/fonts/shaping/shaping_line_breaker.cc:266-611
    fn ShapeLineCore<const RTL: bool>(
        &self,
        start: u32,
        mut available_space: LayoutUnit,
        result_out: &mut Result,
    ) -> *const ShapeResultView {
        debug_assert!(available_space >= LayoutUnit::new());
        let range_start = self.result_.StartIndex();
        let range_end = self.result_.EndIndex();
        debug_assert!(start >= range_start && start < range_end);
        result_out.is_overflow = false;
        result_out.is_hyphenated = false;
        result_out.has_trailing_spaces = false;
        let text = self.GetText();

        if start == range_start
            && available_space >= self.result_.SnappedWidth()
            && !(self.IsStartOfWrappedLine(start)
                && ShouldTrimStartOfWrappedLine(self.text_spacing_trim_))
            && self.result_.IsStartSafeToBreak()
        {
            self.SetBreakOffset(range_end, result_out);
            return ShapeResultView::CreateFromResult(self.result_);
        }

        self.result_.EnsurePositionData(true);
        let start_position = self.result_.CachedPositionForOffset(start - range_start);
        let mut line_start_result: *const ShapeResult = std::ptr::null();
        let first_safe = self.FirstSafeOffset(start);
        debug_assert!(first_safe.offset >= start);
        if first_safe.offset != start {
            let first_safe_position = self
                .result_
                .CachedPositionForOffset(first_safe.offset - range_start);
            line_start_result = self.backend_.Shape(
                start,
                first_safe.offset,
                ShapeOptions {
                    is_line_start: true,
                    han_kerning_start: first_safe.han_kerning,
                    ..ShapeOptions::default()
                },
            );
            let old_width = FlipRtl::<RTL>(first_safe_position - start_position);
            let diff = old_width - unsafe { &*line_start_result }.SnappedWidth();
            if diff != LayoutUnit::new() {
                available_space = (available_space + diff).max(LayoutUnit::new());
            }
        }

        let end_position = start_position + FlipRtl::<RTL>(available_space);
        debug_assert!(FlipRtl::<RTL>(end_position - start_position) >= LayoutUnit::new());
        let mut candidate_break = self.result_.CachedOffsetForPosition(end_position) + range_start;
        if candidate_break < range_end && self.result_.HasAutoSpacingAfter(candidate_break) {
            candidate_break = self.result_.AdjustOffsetForAutoSpacing(
                self.font_.TextAutoSpaceInlineSize(),
                candidate_break,
                end_position.ToFloat(),
            );
        }

        let mut last_safe = 0;
        let mut line_end_result: *const ShapeResult = std::ptr::null();
        if candidate_break < range_end
            && ShouldTrimEnd(self.text_spacing_trim_)
            && Character::MaybeHanKerningClose(i32::from(text.CodeUnitAt(candidate_break)))
        {
            let adjusted_candidate_break = candidate_break + 1;
            if self.break_iterator_.IsBreakable(adjusted_candidate_break) {
                last_safe = self
                    .result_
                    .CachedPreviousSafeToBreakOffset(candidate_break);
                line_end_result = self.backend_.Shape(
                    last_safe,
                    adjusted_candidate_break,
                    ShapeOptions {
                        han_kerning_end: true,
                        ..ShapeOptions::default()
                    },
                );
                let last_safe_position = self
                    .result_
                    .CachedPositionForOffset(last_safe - range_start);
                let width_to_last_safe = FlipRtl::<RTL>(last_safe_position - start_position);
                if width_to_last_safe.ToFloat() + unsafe { &*line_end_result }.Width()
                    <= available_space.ToFloat()
                {
                    candidate_break = adjusted_candidate_break;
                } else {
                    line_end_result = std::ptr::null();
                }
            }
        }

        if candidate_break >= range_end {
            debug_assert_eq!(candidate_break, range_end);
            self.SetBreakOffset(range_end, result_out);
            if !line_end_result.is_null()
                && RuntimeEnabledFeatures::LineBreakerHanKerningEndEnabled()
            {
                return self.ConcatShapeResults(
                    start,
                    range_end,
                    first_safe.offset,
                    last_safe,
                    line_start_result,
                    line_end_result,
                );
            }
            return self.ShapeToEnd(
                start,
                line_start_result,
                first_safe.offset,
                range_start,
                range_end,
            );
        }

        candidate_break = candidate_break.max(start);
        let is_break_after_any_space =
            self.break_iterator_.BreakSpace() == BreakSpaceType::kAfterEverySpace;
        let use_previous_break_opportunity =
            !IsBreakableSpace(text.CodeUnitAt(candidate_break)) || is_break_after_any_space;
        let mut break_opportunity;
        if use_previous_break_opportunity {
            break_opportunity = self.PreviousBreakOpportunity(candidate_break, start);
            result_out.is_overflow = break_opportunity.offset <= start;
            if result_out.is_overflow {
                if self.no_result_if_overflow_ {
                    return std::ptr::null();
                }
                break_opportunity =
                    self.NextBreakOpportunity(candidate_break.max(start + 1), start, range_end);
            }
        } else {
            break_opportunity =
                self.NextBreakOpportunity(candidate_break.max(start + 1), start, range_end);
            debug_assert!(break_opportunity.offset > start);
            debug_assert!(!result_out.is_overflow);
            if break_opportunity.offset > candidate_break
                && (break_opportunity.non_hangable_run_end.is_none()
                    || break_opportunity.non_hangable_run_end.unwrap() > candidate_break)
            {
                let previous_opportunity = self.PreviousBreakOpportunity(candidate_break, start);
                if previous_opportunity.offset > start {
                    break_opportunity = previous_opportunity;
                } else {
                    result_out.is_overflow = true;
                    if self.no_result_if_overflow_ {
                        return std::ptr::null();
                    }
                }
            }
            debug_assert!(!is_break_after_any_space);
            debug_assert!(IsBreakableSpace(text.CodeUnitAt(candidate_break)));
            if break_opportunity
                .non_hangable_run_end
                .is_some_and(|run_end| run_end <= start)
            {
                result_out.has_trailing_spaces = true;
                result_out.break_offset = range_end.min(break_opportunity.offset);
                debug_assert!(IsAllSpaces(text, start, result_out.break_offset));
                result_out.is_hyphenated = false;
                return ShapeResultView::CreateFromResultRange(
                    self.result_,
                    start,
                    result_out.break_offset,
                );
            }
        }

        let mut reshape_line_end = line_end_result.is_null();
        if break_opportunity.offset >= range_end {
            self.SetBreakOffset(range_end, result_out);
            if result_out.is_overflow {
                return self.ShapeToEnd(
                    start,
                    line_start_result,
                    first_safe.offset,
                    range_start,
                    range_end,
                );
            }
            break_opportunity.offset = range_end;
            reshape_line_end = false;
            if break_opportunity
                .non_hangable_run_end
                .is_some_and(|run_end| range_end < run_end)
            {
                break_opportunity.non_hangable_run_end = None;
            }
            if IsBreakableSpace(text.CodeUnitAt(range_end.wrapping_sub(1))) {
                break_opportunity.non_hangable_run_end =
                    Some(FindNonHangableEnd(text, range_end - 1));
            }
        }
        if self.dont_reshape_end_if_at_space_ && reshape_line_end {
            reshape_line_end =
                !IsBreakableSpace(text.CodeUnitAt(break_opportunity.offset.wrapping_sub(1)));
        }
        if !is_break_after_any_space {
            if let Some(run_end) = break_opportunity.non_hangable_run_end {
                break_opportunity.offset = (start + 1).max(run_end);
            }
        }
        CheckBreakOffset(break_opportunity.offset, start, range_end);

        if first_safe.offset >= break_opportunity.offset {
            debug_assert_ne!(first_safe.offset, start);
            self.SetBreakOpportunity(break_opportunity, result_out);
            CheckBreakOffset(result_out.break_offset, start, range_end);
            return ShapeResultView::CreateFromResult(self.backend_.Shape(
                start,
                break_opportunity.offset,
                ShapeOptions {
                    is_line_start: true,
                    han_kerning_start: first_safe.han_kerning,
                    ..ShapeOptions::default()
                },
            ));
        }
        debug_assert!(first_safe.offset >= start);
        debug_assert!(first_safe.offset <= break_opportunity.offset);

        if reshape_line_end {
            loop {
                debug_assert!(start <= break_opportunity.offset);
                if !is_break_after_any_space {
                    if let Some(run_end) = break_opportunity.non_hangable_run_end {
                        break_opportunity.offset = (start + 1).max(run_end);
                    }
                }
                last_safe = self
                    .result_
                    .CachedPreviousSafeToBreakOffset(break_opportunity.offset);
                if last_safe == break_opportunity.offset {
                    break;
                }
                if last_safe > break_opportunity.offset {
                    unreachable!("safe offset exceeds break opportunity");
                }
                if last_safe < first_safe.offset {
                    debug_assert!(last_safe == 0 || last_safe < start);
                    last_safe = start;
                    line_start_result = std::ptr::null();
                }
                debug_assert!(break_opportunity.offset <= range_end);
                if result_out.is_overflow {
                    line_end_result = self.backend_.Shape(
                        last_safe,
                        break_opportunity.offset,
                        ShapeOptions::default(),
                    );
                    break;
                }
                let safe_position = self
                    .result_
                    .CachedPositionForOffset(last_safe - range_start);
                line_end_result = self.backend_.Shape(
                    last_safe,
                    break_opportunity.offset,
                    ShapeOptions::default(),
                );
                if unsafe { &*line_end_result }.Width()
                    <= FlipRtl::<RTL>(end_position - safe_position).ToFloat()
                {
                    break;
                }
                line_end_result = std::ptr::null();
                break_opportunity =
                    self.PreviousBreakOpportunity(break_opportunity.offset - 1, start);
                if break_opportunity.offset > start {
                    continue;
                }
                result_out.is_overflow = true;
                break_opportunity = self.PreviousBreakOpportunity(candidate_break, start);
                if break_opportunity.offset <= start {
                    break_opportunity =
                        self.NextBreakOpportunity(candidate_break.max(start + 1), start, range_end);
                    if break_opportunity.offset >= range_end {
                        self.SetBreakOffset(range_end, result_out);
                        return self.ShapeToEnd(
                            start,
                            line_start_result,
                            first_safe.offset,
                            range_start,
                            range_end,
                        );
                    }
                }
            }
        }

        if line_end_result.is_null() {
            last_safe = break_opportunity.offset;
            debug_assert!(last_safe > start);
            if self.result_.HasAutoSpacingBefore(last_safe) {
                last_safe = self.result_.CachedPreviousSafeToBreakOffset(last_safe - 1);
                debug_assert!(last_safe < break_opportunity.offset);
                line_end_result = self.result_.UnapplyAutoSpacing(
                    self.font_.TextAutoSpaceInlineSize(),
                    last_safe,
                    break_opportunity.offset,
                );
            }
        }
        CheckBreakOffset(break_opportunity.offset, start, range_end);
        debug_assert!(break_opportunity.offset >= last_safe);
        debug_assert_eq!(
            break_opportunity.offset - start,
            (if line_start_result.is_null() {
                0
            } else {
                unsafe { &*line_start_result }.NumCharacters()
            }) + last_safe.saturating_sub(first_safe.offset)
                + (if line_end_result.is_null() {
                    0
                } else {
                    unsafe { &*line_end_result }.NumCharacters()
                })
        );
        self.SetBreakOpportunity(break_opportunity, result_out);
        self.ConcatShapeResults(
            start,
            break_opportunity.offset,
            first_safe.offset,
            last_safe,
            line_start_result,
            line_end_result,
        )
    }

    // cpp: font_engine/fonts/shaping/shaping_line_breaker.cc:613-636
    fn ConcatShapeResults(
        &self,
        start: u32,
        end: u32,
        first_safe: u32,
        last_safe: u32,
        line_start_result: *const ShapeResult,
        line_end_result: *const ShapeResult,
    ) -> *const ShapeResultView {
        let mut segments = Vec::with_capacity(3);
        if !line_start_result.is_null() {
            segments.push(Segment::from_result(line_start_result, 0, u32::MAX));
        }
        if last_safe > first_safe {
            segments.push(Segment::from_result(self.result_, first_safe, last_safe));
        }
        if !line_end_result.is_null() {
            segments.push(Segment::from_result(line_end_result, last_safe, u32::MAX));
        }
        let line_result = ShapeResultView::Create(&segments);
        debug_assert_eq!(end - start, unsafe { &*line_result }.NumCharacters());
        line_result
    }

    // cpp: font_engine/fonts/shaping/shaping_line_breaker.cc:640-674
    fn ShapeToEnd(
        &self,
        start: u32,
        line_start_result: *const ShapeResult,
        first_safe: u32,
        range_start: u32,
        range_end: u32,
    ) -> *const ShapeResultView {
        debug_assert_eq!(range_start, self.result_.StartIndex());
        debug_assert_eq!(range_end, self.result_.EndIndex());
        debug_assert!(start >= range_start && start < range_end && first_safe >= start);
        if line_start_result.is_null() {
            debug_assert_eq!(first_safe, start);
            if start == range_start {
                return ShapeResultView::CreateFromResult(self.result_);
            }
            return ShapeResultView::CreateFromResultRange(self.result_, start, range_end);
        }
        debug_assert_ne!(first_safe, start);
        if first_safe >= range_end {
            return ShapeResultView::CreateFromResultRange(line_start_result, start, range_end);
        }
        let segments = [
            Segment::from_result(line_start_result, 0, u32::MAX),
            Segment::from_result(self.result_, first_safe, range_end),
        ];
        ShapeResultView::Create(&segments)
    }

    // cpp: font_engine/fonts/shaping/shaping_line_breaker.cc:677-709
    pub fn ShapeLineAt(&self, start: u32, end: u32) -> *const ShapeResultView {
        debug_assert!(end > start);
        self.result_.EnsurePositionData(true);
        let first_safe = self.FirstSafeOffset(start);
        debug_assert!(first_safe.offset >= start);
        let mut line_start_result: *const ShapeResult = std::ptr::null();
        if first_safe.offset != start {
            let options = ShapeOptions {
                is_line_start: true,
                han_kerning_start: first_safe.han_kerning,
                ..ShapeOptions::default()
            };
            if first_safe.offset >= end {
                return ShapeResultView::CreateFromResult(self.backend_.Shape(start, end, options));
            }
            line_start_result = self.backend_.Shape(start, first_safe.offset, options);
        }
        let text = self.GetText();
        let (last_safe, line_end_result) = if self.dont_reshape_end_if_at_space_
            && IsBreakableSpace(text.CodeUnitAt(end.wrapping_sub(1)))
        {
            (end, std::ptr::null())
        } else {
            let last_safe = self.result_.CachedPreviousSafeToBreakOffset(end);
            debug_assert!(last_safe >= first_safe.offset);
            let line_end_result = if last_safe != end {
                self.backend_.Shape(last_safe, end, ShapeOptions::default())
            } else {
                std::ptr::null()
            };
            (last_safe, line_end_result)
        };
        self.ConcatShapeResults(
            start,
            end,
            first_safe.offset,
            last_safe,
            line_start_result,
            line_end_result,
        )
    }

    // The ShapeLine width-search and reshape loop remains untranslated.
    // Neither C++ source file is counted complete yet.
}
