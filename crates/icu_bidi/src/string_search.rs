//! Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
//! base/i18n/string_search.cc:19-108. ICU primary-strength collation search.
use std::{
    ffi::{c_char, c_void, CStr, CString},
    ptr::NonNull,
    sync::{Mutex, OnceLock},
};
pub enum UStringSearch {}
pub enum UCollator {}
unsafe extern "C" {
    #[link_name = concat!("uloc_getDefault_", env!("ICU_MAJOR"))]
    fn uloc_getDefault() -> *const c_char;
    #[link_name = concat!("uloc_setDefault_", env!("ICU_MAJOR"))]
    fn uloc_setDefault(locale: *const c_char, status: *mut i32);
    #[link_name = concat!("usearch_open_", env!("ICU_MAJOR"))]
    fn usearch_open(
        pattern: *const u16,
        pattern_length: i32,
        text: *const u16,
        text_length: i32,
        locale: *const c_char,
        break_iterator: *mut c_void,
        status: *mut i32,
    ) -> *mut UStringSearch;
    #[link_name = concat!("usearch_close_", env!("ICU_MAJOR"))]
    fn usearch_close(search: *mut UStringSearch);
    #[link_name = concat!("usearch_getCollator_", env!("ICU_MAJOR"))]
    fn usearch_getCollator(search: *const UStringSearch) -> *mut UCollator;
    #[link_name = concat!("ucol_setStrength_", env!("ICU_MAJOR"))]
    fn ucol_setStrength(collator: *mut UCollator, strength: i32);
    #[link_name = concat!("usearch_reset_", env!("ICU_MAJOR"))]
    fn usearch_reset(search: *mut UStringSearch);
    #[link_name = concat!("usearch_setText_", env!("ICU_MAJOR"))]
    fn usearch_setText(search: *mut UStringSearch, text: *const u16, length: i32, status: *mut i32);
    #[link_name = concat!("usearch_first_", env!("ICU_MAJOR"))]
    fn usearch_first(search: *mut UStringSearch, status: *mut i32) -> i32;
    #[link_name = concat!("usearch_getMatchedLength_", env!("ICU_MAJOR"))]
    fn usearch_getMatchedLength(search: *const UStringSearch) -> i32;
}
/// The embedding initializes ICU from its UI locale before starting workers,
/// as Chromium does, rather than inheriting ICU's en_US_POSIX C-locale tailors.
/// Reject a later language change: existing ICU objects retain their locale.
pub fn InitializeDefaultLocale(locale: &CStr) -> std::io::Result<()> {
    static LOCALE: OnceLock<Mutex<Option<CString>>> = OnceLock::new();
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
    let mut status = 0;
    unsafe { uloc_setDefault(locale.as_ptr(), &mut status) };
    if status > 0 {
        return Err(std::io::Error::other(format!(
            "ICU UI locale initialization failed: {status}"
        )));
    }
    *selected = Some(locale.to_owned());
    Ok(())
}
struct Search(NonNull<UStringSearch>);
impl Drop for Search {
    fn drop(&mut self) {
        unsafe {
            usearch_close(self.0.as_ptr());
        }
    }
}
fn BasicFind(pattern: &[u16], text: &[u16]) -> Option<(usize, usize)> {
    if pattern.is_empty() {
        return Some((0, 0));
    }
    text.windows(pattern.len())
        .position(|part| part == pattern)
        .map(|i| (i, pattern.len()))
}
/// Every pointer is borrowed only while its original UTF-16 slice is live.
/// Source ICU failure falls back to the original exact substring operation.
pub fn StringSearchIgnoringCaseAndAccents(pattern: &[u16], text: &[u16]) -> Option<(usize, usize)> {
    let (Ok(pattern_length), Ok(text_length)) =
        (i32::try_from(pattern.len()), i32::try_from(text.len()))
    else {
        return BasicFind(pattern, text);
    };
    let mut status = 0;
    let search = unsafe {
        usearch_open(
            pattern.as_ptr(),
            pattern_length,
            pattern.as_ptr(),
            pattern_length,
            uloc_getDefault(),
            std::ptr::null_mut(),
            &mut status,
        )
    };
    let Some(search) = NonNull::new(search).map(Search) else {
        return BasicFind(pattern, text);
    };
    if status > 0 {
        return BasicFind(pattern, text);
    }
    unsafe {
        ucol_setStrength(usearch_getCollator(search.0.as_ptr()), 0); // UCOL_PRIMARY
        usearch_reset(search.0.as_ptr());
        status = 0;
        usearch_setText(search.0.as_ptr(), text.as_ptr(), text_length, &mut status);
        if status > 0 {
            return BasicFind(pattern, text);
        }
        let index = usearch_first(search.0.as_ptr(), &mut status);
        if status > 0 || index == -1 {
            return None;
        } // USEARCH_DONE
        Some((
            index as usize,
            usearch_getMatchedLength(search.0.as_ptr()) as usize,
        ))
    }
}
