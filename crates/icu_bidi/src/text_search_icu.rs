//! Scoped collation searcher ownership for Blink TextSearcherIcu, on the
//! ICU4X collator. Source text_searcher_icu.cc:44-121,155-218,255-283.
#![allow(non_snake_case)]
use crate::collation_search::{CollationSearch, UCOL_PRIMARY, UCOL_TERTIARY};
use icu_normalizer::ComposingNormalizerBorrowed;
use std::{
    cell::{Cell, RefCell},
    ffi::CStr,
    marker::PhantomData,
};

thread_local! {
    static SEARCHER_IN_USE: Cell<bool> = const { Cell::new(false) };
    static SEARCHER: RefCell<Option<CollationSearch>> = const { RefCell::new(None) };
}

struct GlobalLease;
impl GlobalLease {
    fn Acquire() -> Self {
        SEARCHER_IN_USE.with(|in_use| {
            assert!(
                !in_use.replace(true),
                "source global TextSearcherIcu cannot overlap; use local tag"
            )
        });
        Self
    }
}
impl Drop for GlobalLease {
    fn drop(&mut self) {
        SEARCHER_IN_USE.with(|in_use| in_use.set(false));
    }
}

/// The borrowed text/pattern cannot outlive the source's FindResults scope.
/// Release clears the strings before returning the reusable global owner.
pub struct TextSearch<'a> {
    searcher: Option<CollationSearch>,
    text_len: usize,
    local: bool,
    _lease: Option<GlobalLease>,
    _strings: PhantomData<(&'a [u16], &'a [u16])>,
}

impl<'a> TextSearch<'a> {
    pub fn new(
        pattern: &'a [u16],
        text: &'a [u16],
        locale: &CStr,
        case_sensitive: bool,
        overlap: bool,
        local: bool,
    ) -> Self {
        assert!(!pattern.is_empty() && !text.is_empty());
        let lease = (!local).then(GlobalLease::Acquire);
        let name = format!("{}@collation=search", locale.to_string_lossy());
        let strength = if case_sensitive { UCOL_TERTIARY } else { UCOL_PRIMARY };
        let existing = if local {
            None
        } else {
            SEARCHER.with(|pool| pool.borrow_mut().take())
        };
        let mut searcher = existing
            .filter(|searcher| searcher.locale() == name)
            .unwrap_or_else(|| CollationSearch::new(&name, strength));
        if searcher.strength() != strength {
            searcher.set_strength(strength);
        }
        searcher.set_pattern(pattern);
        searcher.set_overlap(overlap);
        searcher.set_text(text);
        Self {
            searcher: Some(searcher),
            text_len: text.len(),
            local,
            _lease: lease,
            _strings: PhantomData,
        }
    }

    pub fn SetOffset(&mut self, offset: usize) {
        self.searcher.as_mut().unwrap().set_offset(offset);
    }

    pub fn NextMatch(&mut self) -> Option<(usize, usize)> {
        let (start, length) = self.searcher.as_mut().unwrap().next()?;
        (start < self.text_len && length > 0).then_some((start, length))
    }
}

impl Drop for TextSearch<'_> {
    fn drop(&mut self) {
        let Some(mut searcher) = self.searcher.take() else {
            return;
        };
        if self.local {
            return;
        }
        searcher.set_pattern(&[10]);
        searcher.set_text(&[10]);
        SEARCHER.with(|pool| {
            let previous = pool.borrow_mut().replace(searcher);
            assert!(
                previous.is_none(),
                "source global TextSearcherIcu cannot overlap; use local tag"
            );
        });
    }
}

/// unicode_utilities.cc:307-335: an already-NFC text is returned unchanged;
/// otherwise its NFC form.
pub fn NormalizeCharactersIntoNfc(text: &[u16]) -> Vec<u16> {
    assert!(!text.is_empty());
    let normalizer = ComposingNormalizerBorrowed::new_nfc();
    if normalizer.is_normalized_utf16(text) {
        return text.to_vec();
    }
    normalizer.normalize_utf16(text).into_owned()
}
