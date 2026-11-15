//! Expected values are ICU4C's results for the same calls.
use crate::spoof::{SpoofChecker, Transliterator, UnicodeSet};
use crate::{date_time_locale, idna, string_search, text_search_icu, ubidi_ffi as u, ubrk_ffi as brk};

fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

#[test]
fn properties_use_icu4c_values() {
    unsafe {
        assert_eq!(u::u_charType('A' as i32), 1);
        assert_eq!(u::u_charType(0x0301), 6);
        assert_eq!(u::u_charDirection(0x05D0), 1);
        assert_eq!(u::u_getIntPropertyValue(0x4E00, 0x1004), 5);
        assert_eq!(u::u_getIntPropertyValue(0x0E01, 0x1008), 24);
        assert_eq!(u::u_getIntPropertyValue('(' as i32, 0x1015), 1);
        assert_eq!(u::u_getBidiPairedBracket('(' as i32), ')' as i32);
        assert_eq!(u::u_getIntPropertyValue(0x00C5, 0x1003), 1);
        assert_eq!(u::u_hasBinaryProperty(0xFE0F, 36), 1);
        let mut status = 0;
        assert_eq!(u::uscript_getScript(0x0627, &mut status), 2);
        let name = std::ffi::CStr::from_ptr(u::uscript_getShortName(25));
        assert_eq!(name.to_str().unwrap(), "Latn");
    }
}

#[test]
fn case_folding_preflights() {
    let source = utf16("Straße");
    let mut status = 0;
    let length = unsafe {
        u::u_strFoldCase(std::ptr::null_mut(), 0, source.as_ptr(), source.len() as i32, 0, &mut status)
    };
    assert_eq!((length, status), (7, 15));
    let mut output = vec![0; length as usize];
    status = 0;
    unsafe { u::u_strFoldCase(output.as_mut_ptr(), length, source.as_ptr(), source.len() as i32, 0, &mut status) };
    assert_eq!(String::from_utf16(&output).unwrap(), "strasse");
}

#[test]
fn bidi_runs_and_visual_order() {
    let text = utf16("ab \u{05D0}\u{05D1} cd");
    unsafe {
        let bidi = u::ubidi_open();
        let mut error = 0;
        u::ubidi_setPara(bidi, text.as_ptr(), text.len() as i32, 0xFE, std::ptr::null_mut(), &mut error);
        assert_eq!(error, 0);
        assert_eq!(u::ubidi_getParaLevel(bidi), 0);
        assert_eq!(u::ubidi_getDirection(bidi), 2);
        let (mut end, mut level) = (0, 0);
        u::ubidi_getLogicalRun(bidi, 3, &mut end, &mut level);
        assert_eq!((end, level), (5, 1));
        u::ubidi_close(bidi);
        let levels = [0u8, 1, 1, 0];
        let mut indices = [0i32; 4];
        u::ubidi_reorderVisual(levels.as_ptr(), 4, indices.as_mut_ptr());
        assert_eq!(indices, [0, 2, 1, 3]);
    }
}

#[test]
fn word_boundaries_and_status() {
    let text = utf16("Hello, 42 world");
    unsafe {
        let mut error = 0;
        let iterator = brk::ubrk_open(brk::UBRK_WORD, c"en".as_ptr(), text.as_ptr(), text.len() as i32, &mut error);
        assert_eq!(error, 0);
        assert_eq!(brk::ubrk_first(iterator), 0);
        assert_eq!(brk::ubrk_next(iterator), 5);
        assert_eq!(brk::ubrk_getRuleStatus(iterator), brk::UBRK_WORD_LETTER);
        assert_eq!(brk::ubrk_following(iterator, 7), 9);
        assert_eq!(brk::ubrk_getRuleStatus(iterator), brk::UBRK_WORD_NUMBER);
        assert_eq!(brk::ubrk_preceding(iterator, 9), 7);
        assert_eq!(brk::ubrk_isBoundary(iterator, 3), 0);
        brk::ubrk_close(iterator);
    }
}

#[test]
fn collation_search_ignores_case_and_accents() {
    let text = utf16("Visit the Café now");
    assert_eq!(string_search::StringSearchIgnoringCaseAndAccents(&utf16("cafe"), &text), Some((10, 4)));
    let pattern = utf16("é");
    let mut search = text_search_icu::TextSearch::new(&pattern, &text, c"en", false, false, true);
    assert_eq!(search.NextMatch(), Some((8, 1)));
    assert_eq!(search.NextMatch(), Some((13, 1)));
    assert_eq!(search.NextMatch(), None);
}

#[test]
fn unicode_set_patterns() {
    let set = UnicodeSet::Pattern(&utf16("[[:Latin:] - [a-zA-Z]]")).unwrap();
    assert!(set.Contains(0xE9) && !set.Contains('a' as i32) && !set.Contains(0x0430));
    let set = UnicodeSet::Pattern(&utf16("[\\u0300-\\u0339 [0-9]]")).unwrap();
    assert!(set.Contains(0x301) && set.Contains('7' as i32) && !set.Contains('a' as i32));
    let recommended = UnicodeSet::RecommendedAndInclusion().unwrap();
    assert!(recommended.Contains('a' as i32) && !recommended.Contains(0x00A0));
}

#[test]
fn spoof_restriction_levels_and_skeletons() {
    let mut checker = SpoofChecker::Open().unwrap();
    checker.SetRestrictionLevel(0x30000000);
    checker.SetChecks(checker.Checks().unwrap() | 0x40000000).unwrap();
    assert_eq!(checker.Check(&utf16("example")).unwrap(), 0x10000000);
    // Latin mixed with Cyrillic is minimally restrictive and fails HIGHLY.
    let mixed = checker.Check(&utf16("ex\u{0430}mple")).unwrap();
    assert_eq!(mixed & 0x7f000000, 0x50000000);
    assert_ne!(mixed & 0xffff, 0);
    assert_eq!(checker.Skeleton(&utf16("\u{0430}")).unwrap(), utf16("a"));
}

#[test]
fn transliterator_rules() {
    let mut drop_accents =
        Transliterator::FromRules("DropAcc", "::NFD; ::[:Nonspacing Mark:] Remove; ::NFC; ł > l;").unwrap();
    assert_eq!(drop_accents.Transliterate(&utf16("café łódź")).unwrap(), utf16("cafe lodz"));
    let mut extra = Transliterator::FromRules("ExtraConf", "[æӕ] > ae; [—一] > \\-;").unwrap();
    assert_eq!(extra.Transliterate(&utf16("æ—")).unwrap(), utf16("ae-"));
}

#[test]
fn idna_to_ascii_and_unicode() {
    let idna = idna::Idna::OpenUTS46(idna::UIDNA_CHECK_BIDI | 0x30).unwrap();
    let ascii = idna.NameToASCII(&utf16("bücher.example"));
    assert_eq!((ascii.errors, String::from_utf16(&ascii.value).unwrap()), (0, "xn--bcher-kva.example".into()));
    let unicode = idna.LabelToUnicode(&utf16("xn--bcher-kva"));
    assert_eq!((unicode.errors, String::from_utf16(&unicode.value).unwrap()), (0, "bücher".into()));
}

#[test]
fn english_date_time_symbols() {
    let data = date_time_locale::current(c"en_US").unwrap();
    assert_eq!(data.month_labels[0], "January");
    assert_eq!(data.short_month_labels[11], "Dec");
    assert_eq!(data.ampm_labels, ["AM".to_owned(), "PM".to_owned()]);
    assert_eq!(data.digits[7], '7' as u32);
    assert!(data.date_format.contains('d') && data.time_format.contains("ss"));
    assert!(data.current_year >= 2024);
}
