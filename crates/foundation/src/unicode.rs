#![allow(non_snake_case, non_upper_case_globals)]

// cpp: foundation/blink_base/wtf/text/unicode.h:79-115
// Values are ICU's U_MASK(UCharCategory), using the category numbers in
// icu_bidi/unicode/uchar.h:836-896.
pub type CharCategory = u32;
pub const kNoCategory: CharCategory = 0;
pub const kOther_NotAssigned: CharCategory = 1 << 0;
pub const kLetter_Uppercase: CharCategory = 1 << 1;
pub const kLetter_Lowercase: CharCategory = 1 << 2;
pub const kLetter_Titlecase: CharCategory = 1 << 3;
pub const kLetter_Modifier: CharCategory = 1 << 4;
pub const kLetter_Other: CharCategory = 1 << 5;
pub const kMark_NonSpacing: CharCategory = 1 << 6;
pub const kMark_Enclosing: CharCategory = 1 << 7;
pub const kMark_SpacingCombining: CharCategory = 1 << 8;
pub const kNumber_DecimalDigit: CharCategory = 1 << 9;
pub const kNumber_Letter: CharCategory = 1 << 10;
pub const kNumber_Other: CharCategory = 1 << 11;
pub const kSeparator_Space: CharCategory = 1 << 12;
pub const kSeparator_Line: CharCategory = 1 << 13;
pub const kSeparator_Paragraph: CharCategory = 1 << 14;
pub const kOther_Control: CharCategory = 1 << 15;
pub const kOther_Format: CharCategory = 1 << 16;
pub const kOther_PrivateUse: CharCategory = 1 << 17;
pub const kOther_Surrogate: CharCategory = 1 << 18;
pub const kPunctuation_Dash: CharCategory = 1 << 19;
pub const kPunctuation_Open: CharCategory = 1 << 20;
pub const kPunctuation_Close: CharCategory = 1 << 21;
pub const kPunctuation_Connector: CharCategory = 1 << 22;
pub const kPunctuation_Other: CharCategory = 1 << 23;
pub const kSymbol_Math: CharCategory = 1 << 24;
pub const kSymbol_Currency: CharCategory = 1 << 25;
pub const kSymbol_Modifier: CharCategory = 1 << 26;
pub const kSymbol_Other: CharCategory = 1 << 27;
pub const kPunctuation_InitialQuote: CharCategory = 1 << 28;
pub const kPunctuation_FinalQuote: CharCategory = 1 << 29;

// cpp: foundation/blink_base/wtf/text/unicode.h:145-147
pub fn HasLineBreakingPropertyComplexContext(c: i32) -> bool {
    const UCHAR_LINE_BREAK: i32 = 0x1008;
    const U_LB_COMPLEX_CONTEXT: i32 = 24;
    unsafe {
        icu_bidi::ubidi_ffi::u_getIntPropertyValue(c, UCHAR_LINE_BREAK) == U_LB_COMPLEX_CONTEXT
    }
}

// cpp: foundation/blink_base/wtf/text/unicode.h:173-177
// The source calls Direction(c), whose ICU value for kWhiteSpaceNeutral is 9.
pub fn IsSpaceOrNewline(c: u16) -> bool {
    if c <= 0x7f {
        // cpp: foundation/blink_base/wtf/text/ascii_ctype.h:102-104
        c <= b' ' as u16 && (c == b' ' as u16 || (0x09..=0x0d).contains(&c))
    } else {
        unsafe { icu_bidi::ubidi_ffi::u_charDirection(c as i32) == 9 }
    }
}

// WTF unicode.h categoryMask; ICU's canonical general-category provider.
pub fn Category(code_point: i32) -> CharCategory {
    1 << unsafe { icu_bidi::ubidi_ffi::u_charType(code_point) }
}

// WTF unicode.h:141-143; ICU u_ispunct category mask.
pub fn IsPunct(c: i32) -> bool {
    matches!(unsafe { icu_bidi::ubidi_ffi::u_charType(c) }, 19..=23 | 28..=29)
}

// WTF unicode.h:137-139; string_impl.cc:987-1002.
pub fn IsPrintableChar(c: i32) -> bool {
    unsafe { icu_bidi::ubidi_ffi::u_isprint(c) != 0 }
}
pub fn StartsWithIgnoringCaseAndAccents(text: &[u16], prefix: &[u16]) -> bool {
    icu_bidi::string_search::StringSearchIgnoringCaseAndAccents(prefix, text)
        .is_some_and(|(index, _)| index == 0)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum WordBoundaryClass {
    Word,
    Space,
    Other,
}

fn WordClass(character: char) -> WordBoundaryClass {
    // ICU UCharCategory values. Letters, combining marks and numbers form a
    // word; connector punctuation keeps identifiers such as `foo_bar`
    // together. Space separators form their own selection unit.
    let category = unsafe { icu_bidi::ubidi_ffi::u_charType(character as i32) };
    if (1..=11).contains(&category) || category == 22 {
        WordBoundaryClass::Word
    } else if (12..=14).contains(&category) || character.is_whitespace() {
        WordBoundaryClass::Space
    } else {
        WordBoundaryClass::Other
    }
}

/// Returns the UTF-8 byte range selected by word-granularity editing at
/// `offset`. This is a text service; input routing and selection ownership stay
/// in their respective modules.
pub fn WordBoundaryRange(text: &str, offset: usize) -> std::ops::Range<usize> {
    if text.is_empty() {
        return 0..0;
    }
    let mut offset = offset.min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    let probe = if offset == text.len() {
        text.char_indices()
            .next_back()
            .map_or(0, |(index, _)| index)
    } else {
        offset
    };
    let class = WordClass(
        text[probe..]
            .chars()
            .next()
            .expect("probe is a character boundary"),
    );
    let mut start = probe;
    for (index, character) in text[..probe].char_indices().rev() {
        if WordClass(character) != class {
            break;
        }
        start = index;
    }
    let mut end = probe;
    for character in text[probe..].chars() {
        if WordClass(character) != class {
            break;
        }
        end += character.len_utf8();
    }
    start..end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_boundary_range_groups_unicode_word_characters() {
        assert_eq!(WordBoundaryRange("alpha beta", 7), 6..10);
        assert_eq!(WordBoundaryRange("alpha beta", 10), 6..10);
        assert_eq!(WordBoundaryRange("cafe\u{301} noir", 5), 0..6);
    }
}

// WTF unicode.h IsSymbol/IsAlphanumeric; ICU u_issymbol/u_isalnum category masks.
pub fn IsSymbol(c: i32) -> bool {
    matches!(unsafe { icu_bidi::ubidi_ffi::u_charType(c) }, 24..=27)
}
pub fn IsAlphanumeric(c: i32) -> bool {
    matches!(unsafe { icu_bidi::ubidi_ffi::u_charType(c) }, 1..=5 | 9)
}

// CharacterBreakIterator::Pool and NumGraphemeClusters, source
// platform/text/character_break_iterator.cc:40-72,156-178. Iterator state is
// thread-local; the borrowed UTF16 span is detached before returning to pool.
struct CharacterBreakIteratorPool(Vec<*mut icu_bidi::ubrk_ffi::UBreakIterator>);
impl Drop for CharacterBreakIteratorPool {
    fn drop(&mut self) {
        for iterator in self.0.drain(..) {
            unsafe { icu_bidi::ubrk_ffi::ubrk_close(iterator) };
        }
    }
}
thread_local! {
    static CHARACTER_BREAK_ITERATORS: std::cell::RefCell<CharacterBreakIteratorPool> =
        const { std::cell::RefCell::new(CharacterBreakIteratorPool(Vec::new())) };
}
pub fn NumGraphemeClusters(text: &[u16], locale: &std::ffi::CStr) -> usize {
    if text.is_empty() {
        return 0;
    }
    let mut error = 0;
    let iterator = CHARACTER_BREAK_ITERATORS
        .with(|pool| pool.borrow_mut().0.pop())
        .unwrap_or_else(|| unsafe {
            icu_bidi::ubrk_ffi::ubrk_open(0, locale.as_ptr(), std::ptr::null(), 0, &mut error)
        });
    if iterator.is_null() {
        return text.len();
    }
    if error > 0 {
        unsafe {
            icu_bidi::ubrk_ffi::ubrk_close(iterator);
        }
        return text.len();
    }
    unsafe {
        icu_bidi::ubrk_ffi::ubrk_setText(
            iterator,
            text.as_ptr(),
            i32::try_from(text.len()).expect("ICU UTF16 length fits i32"),
            &mut error,
        );
    }
    let count = if error > 0 {
        text.len()
    } else {
        let mut count = 0;
        unsafe {
            icu_bidi::ubrk_ffi::ubrk_first(iterator);
            while icu_bidi::ubrk_ffi::ubrk_next(iterator) != icu_bidi::ubrk_ffi::UBRK_DONE {
                count += 1;
            }
        }
        count
    };
    error = 0;
    unsafe {
        icu_bidi::ubrk_ffi::ubrk_setText(iterator, std::ptr::null(), 0, &mut error);
    }
    CHARACTER_BREAK_ITERATORS.with(|pool| pool.borrow_mut().0.push(iterator));
    count
}
/// ICU UCharDirection values, preserving WTF::unicode::Direction.
pub fn Direction(character: u16) -> i32 {
    unsafe { icu_bidi::ubidi_ffi::u_charDirection(character as i32) }
}

/// ICU UCHAR_VARIATION_SELECTOR=36, uchar.h. Preserve the original source
/// property query; emoji classification separately borrows generated tables.
pub fn IsVariationSelector(code_point: u32) -> bool {
    unsafe { icu_bidi::ubidi_ffi::u_hasBinaryProperty(code_point as i32, 36) != 0 }
}
pub fn CombiningClass(code_point: u32) -> u8 {
    unsafe { icu_bidi::ubidi_ffi::u_getCombiningClass(code_point as i32) }
}

// Blink smart_replace_cf.cc:45-48,82-83 / smart_replace_icu.cc:58-65,99-105.
// Keep the native platform's predefined character classes; these are never
// derived from locale-sensitive JS/DOM text or a copied Unicode range table.
#[cfg(target_os = "macos")]
mod smart_replace_cf {
    use std::ffi::c_void;
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFCharacterSetGetPredefined(kind: i32) -> *const c_void;
        fn CFCharacterSetIsLongCharacterMember(set: *const c_void, character: u32) -> u8;
    }
    pub(super) fn Contains(kind: i32, character: u32) -> bool {
        unsafe {
            CFCharacterSetIsLongCharacterMember(CFCharacterSetGetPredefined(kind), character) != 0
        }
    }
}
pub fn SmartReplaceIsWhitespaceOrNewline(character: u32) -> bool {
    #[cfg(target_os = "macos")]
    {
        smart_replace_cf::Contains(3, character)
    }
    #[cfg(not(target_os = "macos"))]
    {
        unsafe { icu_bidi::ubidi_ffi::u_hasBinaryProperty(character as i32, 31) != 0 }
    }
}
pub fn SmartReplaceIsPunctuation(character: u32) -> bool {
    #[cfg(target_os = "macos")]
    {
        smart_replace_cf::Contains(11, character)
    }
    #[cfg(not(target_os = "macos"))]
    {
        IsPunct(character as i32)
    }
}
