//! Chromium platform/language.cc:35-43,84-172. PlatformLanguage itself is
//! supplied by the embedding's original immutable locale owner.
use crate::AtomicString;
use std::cell::RefCell;
thread_local! {static OVERRIDE:RefCell<Vec<AtomicString>>=const{RefCell::new(Vec::new())};}
pub fn CanonicalizeLanguageIdentifier(value: &AtomicString) -> AtomicString {
    let units = value
        .Span16()
        .iter()
        .map(|unit| {
            if *unit == u16::from(b'_') {
                u16::from(b'-')
            } else {
                *unit
            }
        })
        .collect::<Vec<_>>();
    AtomicString::from_utf16(&units)
}
pub fn OverrideUserPreferredLanguagesForTesting(values: &[AtomicString]) {
    OVERRIDE.with(|slot| {
        *slot.borrow_mut() = values.iter().map(CanonicalizeLanguageIdentifier).collect()
    });
}
pub fn UserPreferredLanguages(platform: &AtomicString) -> Vec<AtomicString> {
    OVERRIDE.with(|slot| {
        let values = slot.borrow();
        if values.is_empty() {
            vec![CanonicalizeLanguageIdentifier(platform)]
        } else {
            values.clone()
        }
    })
}
pub fn IndexOfBestMatchingLanguageInList(language: &AtomicString, list: &[AtomicString]) -> usize {
    let units = language.Span16();
    let language_only = units.len() == 2 || (units.len() >= 3 && units[2] == u16::from(b'-'));
    let mut only = None;
    let mut different_locale = None;
    for (i, item) in list.iter().enumerate() {
        let canonical = CanonicalizeLanguageIdentifier(item);
        if language == &canonical {
            return i;
        }
        let value = canonical.Span16();
        if language_only && value.len() >= 2 && units[..2] == value[..2] {
            if only.is_none() && value.len() == 2 {
                only = Some(i);
            }
            if different_locale.is_none() && value.len() >= 3 {
                different_locale = Some(i);
            }
        }
    }
    only.or(different_locale).unwrap_or(list.len())
}
