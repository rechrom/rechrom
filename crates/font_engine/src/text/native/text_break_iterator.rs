#![allow(non_snake_case, non_upper_case_globals)]

use std::cell::RefCell;
use std::fmt;

use foundation::String;

use super::character::Character;
use super::character_break_iterator::{kTextBreakDone, CharacterBreakIterator};
use super::fast_line_break_data::{kFastLineBreakMaxChar, kFastLineBreakMinChar, GetFastLineBreak};
use super::layout_locale::{LayoutLocale, LineBreakStrictness};
use super::line_break_rule_engine::NextLineBreakRuleBoundary;
use super::line_break_tailoring_data::kBreakAll;
use super::phrase_break_services::CurrentNativePhraseBreakResolver;

const UCHAR_LINE_BREAK: i32 = 0x1008;
const U_LB_ALPHABETIC: i32 = 2;
const U_LB_BREAK_AFTER: i32 = 4;
const U_LB_COMBINING_MARK: i32 = 9;
const U_LB_IDEOGRAPHIC: i32 = 14;
const U_LB_NUMERIC: i32 = 19;
const U_LB_COMPLEX_CONTEXT: i32 = 24;
const U_LB_COUNT: i32 = 49;
const K_SOFT_HYPHEN: u32 = 0xAD;
const K_HYPHEN: u32 = 0x2010;
const K_EN_DASH: u32 = 0x2013;
const U_GC_L_MASK: u32 = (1 << 1) | (1 << 2) | (1 << 3) | (1 << 4) | (1 << 5);
const U_GC_M_MASK: u32 = (1 << 6) | (1 << 7) | (1 << 8);
const U_GC_N_MASK: u32 = (1 << 9) | (1 << 10) | (1 << 11);

use icu_bidi::native as icu_api;

// cpp: third_party/blink/renderer/platform/text/text_break_iterator.cc:186-260
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FastBreakResult {
    kNoBreak,
    kCanBreak,
    kUnknown,
}

#[inline]
fn IsAsciiAlphanumeric(ch: u16) -> bool {
    (u16::from(b'0')..=u16::from(b'9')).contains(&ch)
        || (u16::from(b'A')..=u16::from(b'Z')).contains(&ch)
        || (u16::from(b'a')..=u16::from(b'z')).contains(&ch)
}

#[inline]
fn ShouldBreakFast(
    previous_previous: u16,
    previous: u16,
    current: u16,
    disable_soft_hyphen: bool,
) -> FastBreakResult {
    if previous < kFastLineBreakMinChar || current < kFastLineBreakMinChar {
        return FastBreakResult::kNoBreak;
    }
    if previous == u16::from(b'-') {
        if current <= 0x7F {
            if (u16::from(b'0')..=u16::from(b'9')).contains(&current) {
                return if IsAsciiAlphanumeric(previous_previous) {
                    FastBreakResult::kCanBreak
                } else {
                    FastBreakResult::kNoBreak
                };
            }
        } else {
            return FastBreakResult::kUnknown;
        }
    }
    if previous <= kFastLineBreakMaxChar && current <= kFastLineBreakMaxChar {
        if !GetFastLineBreak(previous, current) {
            return FastBreakResult::kNoBreak;
        }
        if disable_soft_hyphen && previous == K_SOFT_HYPHEN as u16 {
            return FastBreakResult::kNoBreak;
        }
        return FastBreakResult::kCanBreak;
    }
    FastBreakResult::kUnknown
}

// cpp: font_engine/text/native/text_break_iterator.h:81-104
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LineBreakType {
    #[default]
    kNormal,
    kBreakAll,
    kBreakCharacter,
    kKeepAll,
    kPhrase,
}

// cpp: font_engine/text/native/text_break_iterator.h:107-116
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BreakSpaceType {
    #[default]
    kAfterSpaceRun,
    kAfterEverySpace,
}

// cpp: font_engine/text/native/text_break_iterator.cc:149-155
fn LineBreakProperty(code_point: u32) -> i32 {
    if code_point == u32::from(b'+') {
        return U_LB_ALPHABETIC;
    }
    unsafe { icu_api::u_getIntPropertyValue(code_point as i32, UCHAR_LINE_BREAK) }
}

// cpp: font_engine/text/native/text_break_iterator.cc:157-175
fn ShouldBreakAfterBreakAll(
    previous: i32,
    current: i32,
    current_character: u32,
    strictness: LineBreakStrictness,
) -> bool {
    if previous < 0 || previous >= U_LB_COUNT || current < 0 || current >= U_LB_COUNT {
        return false;
    }
    if kBreakAll[previous as usize][current as usize / 8] & (0x80 >> (current % 8)) == 0 {
        return false;
    }
    if current == U_LB_BREAK_AFTER
        && current_character != 0x007C
        && strictness != LineBreakStrictness::kLoose
    {
        return false;
    }
    true
}

// cpp: font_engine/text/native/text_break_iterator.cc:177-188
fn ShouldKeepAfterKeepAll(previous_previous: u32, previous: u32, current: u32) -> bool {
    let previous_category = unsafe { icu_api::u_charType(previous as i32) } as u32;
    let base = if (1_u32 << previous_category) & U_GC_M_MASK != 0 {
        previous_previous
    } else {
        previous
    };
    let base_category = unsafe { icu_api::u_charType(base as i32) } as u32;
    let current_category = unsafe { icu_api::u_charType(current as i32) } as u32;
    ((1_u32 << base_category) & (U_GC_L_MASK | U_GC_N_MASK) != 0)
        && !foundation::unicode::HasLineBreakingPropertyComplexContext(base as i32)
        && ((1_u32 << current_category) & (U_GC_L_MASK | U_GC_N_MASK) != 0)
        && !foundation::unicode::HasLineBreakingPropertyComplexContext(current as i32)
}

// cpp: font_engine/text/native/text_break_iterator.cc:190-194
fn IsCodePointBoundary(text: &String, position: u32) -> bool {
    let units = text.Span16().expect("non-null line-break text");
    position == 0
        || position == text.length()
        || !((0xDC00..=0xDFFF).contains(&units[position as usize])
            && (0xD800..=0xDBFF).contains(&units[position as usize - 1]))
}

// cpp: font_engine/text/native/text_break_iterator.cc:196-210
fn CodePointBefore(text: &String, position: u32, minimum: u32, begin: &mut u32) -> u32 {
    let units = text.Span16().expect("non-null line-break text");
    let mut i = position - 1;
    let mut value = u32::from(units[i as usize]);
    if (0xDC00..=0xDFFF).contains(&value) && i > minimum {
        let lead = u32::from(units[i as usize - 1]);
        if (0xD800..=0xDBFF).contains(&lead) {
            i -= 1;
            value = 0x10000 + ((lead - 0xD800) << 10) + (value - 0xDC00);
        }
    }
    *begin = i;
    value
}

// cpp: font_engine/text/native/text_break_iterator.h:131-241
pub struct LazyLineBreakIterator {
    string_: String,
    locale_: *const LayoutLocale,
    character_iterator_: RefCell<Option<CharacterBreakIterator>>,
    rule_boundaries_: RefCell<Vec<u32>>,
    start_offset_: u32,
    break_type_: LineBreakType,
    break_space_: BreakSpaceType,
    strictness_: LineBreakStrictness,
    disable_soft_hyphen_: bool,
}

impl LazyLineBreakIterator {
    // cpp: font_engine/text/native/text_break_iterator.h:135-142
    pub fn new(string: &String, locale: *const LayoutLocale, break_type: LineBreakType) -> Self {
        Self {
            string_: string.clone(),
            locale_: locale,
            character_iterator_: RefCell::new(None),
            // Chromium does not create the native iterator until the fast
            // line-break path cannot decide.  Keep the translated rule cache
            // empty for the same reason: `start_offset_` is the implicit
            // initial boundary, so a one-element allocation here is both
            // redundant and eager.
            rule_boundaries_: RefCell::new(Vec::new()),
            start_offset_: 0,
            break_type_: break_type,
            break_space_: BreakSpaceType::kAfterSpaceRun,
            strictness_: LineBreakStrictness::kDefault,
            disable_soft_hyphen_: false,
        }
    }

    // cpp: font_engine/text/native/text_break_iterator.h:140-145
    pub fn with_replacement_string(other: &Self, string: String) -> Self {
        let mut iterator = Self::new(&string, other.Locale(), other.BreakType());
        iterator.SetBreakSpace(other.BreakSpace());
        iterator.SetStrictness(other.Strictness());
        iterator
    }

    pub fn GetString(&self) -> &String {
        &self.string_
    }

    // cpp: font_engine/text/native/text_break_iterator.h:153-157
    pub fn ResetStringAndReleaseIterator(&mut self, string: String, locale: *const LayoutLocale) {
        self.string_ = string;
        self.start_offset_ = 0;
        self.SetLocale(locale);
        self.character_iterator_.replace(None);
        self.rule_boundaries_.get_mut().clear();
    }

    pub fn StartOffset(&self) -> u32 {
        self.start_offset_
    }

    pub fn SetStartOffset(&mut self, offset: u32) {
        assert!(offset <= self.string_.length());
        self.start_offset_ = offset;
        self.character_iterator_.replace(None);
        self.rule_boundaries_.get_mut().clear();
    }

    pub fn Locale(&self) -> *const LayoutLocale {
        self.locale_
    }

    pub fn SetLocale(&mut self, locale: *const LayoutLocale) {
        if locale == self.locale_ {
            return;
        }
        self.locale_ = locale;
        self.character_iterator_.replace(None);
    }

    pub fn BreakType(&self) -> LineBreakType {
        self.break_type_
    }

    pub fn SetBreakType(&mut self, break_type: LineBreakType) {
        if self.break_type_ != break_type {
            self.break_type_ = break_type;
        }
    }

    pub fn BreakSpace(&self) -> BreakSpaceType {
        self.break_space_
    }

    pub fn SetBreakSpace(&mut self, break_space: BreakSpaceType) {
        self.break_space_ = break_space;
    }

    pub fn Strictness(&self) -> LineBreakStrictness {
        self.strictness_
    }

    pub fn SetStrictness(&mut self, strictness: LineBreakStrictness) {
        if self.strictness_ != strictness {
            self.strictness_ = strictness;
            self.character_iterator_.replace(None);
            self.rule_boundaries_.get_mut().clear();
        }
    }

    pub fn IsSoftHyphenEnabled(&self) -> bool {
        !self.disable_soft_hyphen_
    }

    pub fn EnableSoftHyphen(&mut self, value: bool) {
        self.disable_soft_hyphen_ = !value;
    }

    pub fn IsBreakableSpace(ch: u16) -> bool {
        ch == 0x20 || ch == 0x09 || ch == 0x0A
    }

    // cpp: font_engine/text/native/text_break_iterator.h:191-204
    pub fn IsBreakable(&self, pos: u32) -> bool {
        let len = pos.saturating_add(1).min(self.string_.length());
        pos == self.NextBreakablePosition(pos, len)
    }

    // cpp: font_engine/text/native/text_break_iterator.cc:214-225
    pub fn NextBreakablePositionBreakCharacter(&self, mut pos: u32) -> u32 {
        debug_assert!(self.start_offset_ <= self.string_.length());
        let mut iterator = self.character_iterator_.borrow_mut();
        if iterator.is_none() {
            let units = self.string_.Span16().expect("non-null line-break text");
            *iterator = Some(CharacterBreakIterator::from_utf16(
                &units[self.start_offset_ as usize..],
            ));
        }
        debug_assert!(pos >= self.start_offset_);
        pos -= self.start_offset_;
        let next = iterator
            .as_ref()
            .unwrap()
            .Following((if pos > 0 { pos - 1 } else { 0 }) as i32);
        if next != kTextBreakDone {
            next as u32 + self.start_offset_
        } else {
            self.string_.length()
        }
    }

    // Chromium instantiates LazyLineBreakIterator::NextBreakablePosition for
    // the StringImpl storage width.  Keep Latin-1 text on the same compact
    // byte path instead of projecting every query through UTF-16 code-point
    // and surrogate-boundary helpers.
    // cpp: third_party/blink/renderer/platform/text/text_break_iterator.cc:276-381
    fn NextBreakablePosition8(&self, pos: u32, len: u32) -> u32 {
        let bytes = self
            .string_
            .Span8()
            .expect("8-bit line-break text has Latin-1 storage");
        for i in pos..len {
            if i <= self.start_offset_ {
                continue;
            }
            let previous = u32::from(bytes[i as usize - 1]);
            let current = u32::from(bytes[i as usize]);
            let previous_previous = if i > self.start_offset_ + 1 {
                u32::from(bytes[i as usize - 2])
            } else {
                0
            };
            let previous_space = Self::IsBreakableSpace(previous as u16);
            let current_space = Self::IsBreakableSpace(current as u16);
            if self.break_space_ == BreakSpaceType::kAfterSpaceRun {
                if current_space {
                    continue;
                }
                if previous_space {
                    return i;
                }
            } else {
                if previous_space || Character::IsOtherSpaceSeparator(previous as i32) {
                    return i;
                }
                if (current_space || Character::IsOtherSpaceSeparator(current as i32))
                    && i + 1 < len
                {
                    return i + 1;
                }
            }

            let fast_break_result = ShouldBreakFast(
                previous_previous as u16,
                previous as u16,
                current as u16,
                self.disable_soft_hyphen_,
            );
            if fast_break_result == FastBreakResult::kCanBreak {
                return i;
            }

            if self.break_type_ == LineBreakType::kBreakAll {
                let previous_class = LineBreakProperty(previous);
                let current_class = LineBreakProperty(current);
                if (self.strictness_ == LineBreakStrictness::kLoose
                    && (current == K_HYPHEN || current == K_EN_DASH)
                    && (previous_class == U_LB_NUMERIC
                        || previous_class == U_LB_ALPHABETIC
                        || previous_class == U_LB_COMPLEX_CONTEXT
                        || previous_class == U_LB_IDEOGRAPHIC))
                    || ShouldBreakAfterBreakAll(
                        previous_class,
                        current_class,
                        current,
                        self.strictness_,
                    )
                {
                    return i;
                }
            } else if self.break_type_ == LineBreakType::kKeepAll
                && ShouldKeepAfterKeepAll(previous_previous, previous, current)
            {
                continue;
            }
            if fast_break_result == FastBreakResult::kNoBreak {
                continue;
            }

            // The Unicode fallback is uncommon for Latin-1. Acquire and grow
            // its cached rule boundaries only when the fast table cannot
            // decide, matching Chromium's lazy GetIterator() path.
            let mut rule_boundaries = self.rule_boundaries_.borrow_mut();
            while rule_boundaries
                .last()
                .copied()
                .unwrap_or(self.start_offset_)
                < i
            {
                let boundary = rule_boundaries
                    .last()
                    .copied()
                    .unwrap_or(self.start_offset_);
                let next = NextLineBreakRuleBoundary(
                    &self.string_,
                    boundary,
                    self.string_.length(),
                    self.strictness_,
                );
                assert!(next > boundary);
                rule_boundaries.push(next);
            }
            let mut is_break = rule_boundaries.binary_search(&i).is_ok();
            if self.disable_soft_hyphen_ && previous == K_SOFT_HYPHEN {
                is_break = false;
            }
            if is_break {
                return i;
            }
        }
        len
    }

    // cpp: font_engine/text/native/text_break_iterator.cc:227-306
    pub fn NextBreakablePosition(&self, pos: u32, len: u32) -> u32 {
        if self.string_.IsNull() {
            return 0;
        }
        assert!(self.start_offset_ <= pos && pos <= len && len <= self.string_.length());
        if self.break_type_ == LineBreakType::kBreakCharacter {
            return self.NextBreakablePositionBreakCharacter(pos).min(len);
        }
        if self.break_type_ == LineBreakType::kPhrase {
            let resolver =
                CurrentNativePhraseBreakResolver().expect("phrase resolver not installed");
            return unsafe { resolver.as_ref() }.NextBreak(
                unsafe { LayoutLocale::LocaleString(self.locale_) },
                &self.string_,
                pos,
                len,
            );
        }
        if self.string_.Is8Bit() {
            return self.NextBreakablePosition8(pos, len);
        }
        let mut rule_boundaries = self.rule_boundaries_.borrow_mut();
        let units = self.string_.Span16().expect("non-null line-break text");
        for i in pos..=len {
            if i == len {
                return len;
            }
            if i <= self.start_offset_ || !IsCodePointBoundary(&self.string_, i) {
                continue;
            }
            let mut previous_begin = 0;
            let previous =
                CodePointBefore(&self.string_, i, self.start_offset_, &mut previous_begin);
            let mut current_end = 0;
            let current =
                super::line_break_rule_engine::CodePointAt(&self.string_, i, len, &mut current_end);
            let previous_space = Self::IsBreakableSpace(previous as u16);
            let current_space = Self::IsBreakableSpace(current as u16);
            if self.break_space_ == BreakSpaceType::kAfterSpaceRun {
                if current_space {
                    continue;
                }
                if previous_space {
                    return i;
                }
            } else {
                if previous_space || Character::IsOtherSpaceSeparator((previous as u16) as i32) {
                    return i;
                }
                if (current_space || Character::IsOtherSpaceSeparator((current as u16) as i32))
                    && current_end < len
                {
                    return current_end;
                }
            }

            let previous_unit = units[i as usize - 1];
            let current_unit = units[i as usize];
            let previous_previous_unit = if i > self.start_offset_ + 1 {
                units[i as usize - 2]
            } else {
                0
            };
            let fast_break_result = ShouldBreakFast(
                previous_previous_unit,
                previous_unit,
                current_unit,
                self.disable_soft_hyphen_,
            );
            if fast_break_result == FastBreakResult::kCanBreak {
                return i;
            }

            if self.break_type_ == LineBreakType::kBreakAll {
                let mut previous_class = LineBreakProperty(previous);
                let mut scan = previous_begin;
                while previous_class == U_LB_COMBINING_MARK && scan > self.start_offset_ {
                    let mut earlier = 0;
                    previous_class = LineBreakProperty(CodePointBefore(
                        &self.string_,
                        scan,
                        self.start_offset_,
                        &mut earlier,
                    ));
                    scan = earlier;
                }
                let current_class = LineBreakProperty(current);
                if (self.strictness_ == LineBreakStrictness::kLoose
                    && (current == K_HYPHEN || current == K_EN_DASH)
                    && (previous_class == U_LB_NUMERIC
                        || previous_class == U_LB_ALPHABETIC
                        || previous_class == U_LB_COMPLEX_CONTEXT
                        || previous_class == U_LB_IDEOGRAPHIC))
                    || ShouldBreakAfterBreakAll(
                        previous_class,
                        current_class,
                        current,
                        self.strictness_,
                    )
                {
                    return i;
                }
            } else if self.break_type_ == LineBreakType::kKeepAll {
                let mut previous_previous = 0;
                if previous_begin > self.start_offset_ {
                    let mut ignored = 0;
                    previous_previous = CodePointBefore(
                        &self.string_,
                        previous_begin,
                        self.start_offset_,
                        &mut ignored,
                    );
                }
                if ShouldKeepAfterKeepAll(previous_previous, previous, current) {
                    continue;
                }
            }
            if fast_break_result == FastBreakResult::kNoBreak {
                continue;
            }

            while rule_boundaries
                .last()
                .copied()
                .unwrap_or(self.start_offset_)
                < i
            {
                let boundary = rule_boundaries
                    .last()
                    .copied()
                    .unwrap_or(self.start_offset_);
                let next = NextLineBreakRuleBoundary(
                    &self.string_,
                    boundary,
                    self.string_.length(),
                    self.strictness_,
                );
                assert!(next > boundary);
                rule_boundaries.push(next);
            }
            let mut is_break = rule_boundaries.binary_search(&i).is_ok();
            if self.disable_soft_hyphen_ && previous == K_SOFT_HYPHEN {
                is_break = false;
            }
            if is_break {
                return i;
            }
        }
        len
    }

    // cpp: font_engine/text/native/text_break_iterator.cc:308-318
    pub fn NextBreakOpportunity(&self, offset: u32) -> u32 {
        debug_assert!(offset <= self.string_.length());
        self.NextBreakablePosition(offset, self.string_.length())
    }

    pub fn NextBreakOpportunityTo(&self, offset: u32, len: u32) -> u32 {
        debug_assert!(offset <= len && len <= self.string_.length());
        self.NextBreakablePosition(offset, len)
    }

    // cpp: font_engine/text/native/text_break_iterator.cc:320-332
    pub fn PreviousBreakOpportunity(&self, offset: u32, min: u32) -> u32 {
        let mut pos = offset.min(self.string_.length());
        // Chromium includes only enough look-ahead to decide whether `pos`
        // itself is a break. After a miss the next query ends at the previous
        // position. Scanning to string_.length() here turns the backwards
        // search performed for every shaped line into repeated suffix scans.
        let mut end = pos.saturating_add(2).min(self.string_.length());
        while pos > min {
            if self.NextBreakablePosition(pos, end) == pos {
                return pos;
            }
            // There are no break opportunities at `pos` or after within the
            // decision window, matching LazyLineBreakIterator in Blink.
            end = pos;
            pos -= 1;
            while pos > min && !IsCodePointBoundary(&self.string_, pos) {
                pos -= 1;
            }
        }
        min
    }
}

// cpp: font_engine/text/native/text_break_iterator.cc:334-343
impl fmt::Display for LineBreakType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::kNormal => "Normal",
            Self::kBreakAll => "BreakAll",
            Self::kBreakCharacter => "BreakCharacter",
            Self::kKeepAll => "KeepAll",
            Self::kPhrase => "Phrase",
        })
    }
}

// cpp: font_engine/text/native/text_break_iterator.cc:345-351
impl fmt::Display for BreakSpaceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::kAfterSpaceRun => "kAfterSpaceRun",
            Self::kAfterEverySpace => "kAfterEverySpace",
        })
    }
}
