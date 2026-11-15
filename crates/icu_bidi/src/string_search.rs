//! Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
//! base/i18n/string_search.cc:19-108. Primary-strength collation search.
#![allow(non_snake_case)]
use crate::collation_search::{CollationSearch, UCOL_PRIMARY};
use std::{
    ffi::{CStr, CString},
    sync::{Mutex, OnceLock},
};

static LOCALE: OnceLock<Mutex<Option<CString>>> = OnceLock::new();

/// The embedding initializes the default locale from its UI locale before
/// starting workers, as Chromium initializes ICU's. A later different
/// language is rejected: existing searchers retain their locale.
pub fn InitializeDefaultLocale(locale: &CStr) -> std::io::Result<()> {
    let mut selected = LOCALE.get_or_init(|| Mutex::new(None)).lock().unwrap();
    if let Some(selected) = selected.as_ref() {
        return if selected.as_c_str() == locale {
            Ok(())
        } else {
            Err(std::io::Error::other(
                "ICU UI locale cannot change after initialization",
            ))
        };
    }
    *selected = Some(locale.to_owned());
    Ok(())
}

fn default_locale() -> String {
    LOCALE
        .get()
        .and_then(|locale| locale.lock().unwrap().as_ref().map(|value| value.to_string_lossy().into_owned()))
        .unwrap_or_default()
}

fn BasicFind(pattern: &[u16], text: &[u16]) -> Option<(usize, usize)> {
    if pattern.is_empty() {
        return Some((0, 0));
    }
    text.windows(pattern.len())
        .position(|part| part == pattern)
        .map(|i| (i, pattern.len()))
}

/// The first match of `pattern` in `text`, ignoring case and accents.
pub fn StringSearchIgnoringCaseAndAccents(pattern: &[u16], text: &[u16]) -> Option<(usize, usize)> {
    if pattern.is_empty() {
        return BasicFind(pattern, text);
    }
    let mut search = CollationSearch::new(&default_locale(), UCOL_PRIMARY);
    search.set_pattern(pattern);
    search.set_text(text);
    search.next()
}
