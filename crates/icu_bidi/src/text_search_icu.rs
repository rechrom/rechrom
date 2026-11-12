//! ICU usearch ABI and scoped searcher ownership for Blink TextSearcherIcu.
//! Source text_searcher_icu.cc:44-121,155-218,255-283.
#![allow(non_snake_case)]
use std::{
    cell::{Cell, RefCell},
    ffi::{c_char, c_void, CStr, CString},
    marker::PhantomData,
    ptr::NonNull,
};
enum UStringSearch {}
enum UCollator {}
unsafe extern "C" {
    #[link_name=concat!("usearch_open_",env!("ICU_MAJOR"))]
    fn usearch_open(
        pattern: *const u16,
        pattern_length: i32,
        text: *const u16,
        text_length: i32,
        locale: *const c_char,
        break_iterator: *mut c_void,
        status: *mut i32,
    ) -> *mut UStringSearch;
    #[link_name=concat!("usearch_close_",env!("ICU_MAJOR"))]
    fn usearch_close(search: *mut UStringSearch);
    #[link_name=concat!("usearch_setPattern_",env!("ICU_MAJOR"))]
    fn usearch_setPattern(
        search: *mut UStringSearch,
        pattern: *const u16,
        length: i32,
        status: *mut i32,
    );
    #[link_name=concat!("usearch_setText_",env!("ICU_MAJOR"))]
    fn usearch_setText(search: *mut UStringSearch, text: *const u16, length: i32, status: *mut i32);
    #[link_name=concat!("usearch_setOffset_",env!("ICU_MAJOR"))]
    fn usearch_setOffset(search: *mut UStringSearch, offset: i32, status: *mut i32);
    #[link_name=concat!("usearch_next_",env!("ICU_MAJOR"))]
    fn usearch_next(search: *mut UStringSearch, status: *mut i32) -> i32;
    #[link_name=concat!("usearch_getMatchedLength_",env!("ICU_MAJOR"))]
    fn usearch_getMatchedLength(search: *const UStringSearch) -> i32;
    #[link_name=concat!("usearch_getCollator_",env!("ICU_MAJOR"))]
    fn usearch_getCollator(search: *const UStringSearch) -> *mut UCollator;
    #[link_name=concat!("ucol_setStrength_",env!("ICU_MAJOR"))]
    fn ucol_setStrength(collator: *mut UCollator, strength: i32);
    #[link_name=concat!("ucol_getStrength_",env!("ICU_MAJOR"))]
    fn ucol_getStrength(collator: *const UCollator) -> i32;
    #[link_name=concat!("usearch_reset_",env!("ICU_MAJOR"))]
    fn usearch_reset(search: *mut UStringSearch);
    #[link_name=concat!("usearch_setAttribute_",env!("ICU_MAJOR"))]
    fn usearch_setAttribute(
        search: *mut UStringSearch,
        attribute: i32,
        value: i32,
        status: *mut i32,
    );
}
struct Searcher {
    pointer: NonNull<UStringSearch>,
    locale: CString,
}
impl Drop for Searcher {
    fn drop(&mut self) {
        unsafe { usearch_close(self.pointer.as_ptr()) };
    }
}
thread_local! {static SEARCHER_IN_USE:Cell<bool>=const {Cell::new(false)}; static SEARCHER:RefCell<Option<Searcher>>=const {RefCell::new(None)};}
static NEWLINE: [u16; 1] = [10];
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
/// Release resets ICU pointers before returning the original reusable owner.
pub struct TextSearch<'a> {
    searcher: Option<Searcher>,
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
        let name = CString::new([locale.to_bytes(), b"@collation=search"].concat())
            .expect("source UI locale");
        let existing = if local {
            None
        } else {
            SEARCHER.with(|pool| pool.borrow_mut().take())
        };
        let searcher = existing
            .filter(|searcher| searcher.locale == name)
            .unwrap_or_else(|| {
                let mut status = 0;
                let pointer = unsafe {
                    usearch_open(
                        NEWLINE.as_ptr(),
                        1,
                        NEWLINE.as_ptr(),
                        1,
                        name.as_ptr(),
                        std::ptr::null_mut(),
                        &mut status,
                    )
                };
                assert!(status <= 0, "ICU CreateSearcher failed: {status}");
                Searcher {
                    pointer: NonNull::new(pointer).expect("source ICU searcher"),
                    locale: name,
                }
            });
        let pointer = searcher.pointer.as_ptr();
        let mut status = 0;
        unsafe {
            let strength = if case_sensitive { 2 } else { 0 };
            let collator = usearch_getCollator(pointer);
            if ucol_getStrength(collator) != strength {
                ucol_setStrength(collator, strength);
                usearch_reset(pointer);
            }
            usearch_setPattern(
                pointer,
                pattern.as_ptr(),
                i32::try_from(pattern.len()).expect("ICU pattern length"),
                &mut status,
            );
            assert!(status <= 0, "ICU SetPattern failed: {status}");
            status = 0;
            usearch_setAttribute(pointer, 0, if overlap { 1 } else { 0 }, &mut status);
            assert!(status <= 0, "ICU SetAllowOverlapMatches failed: {status}");
            status = 0;
            usearch_setText(
                pointer,
                text.as_ptr(),
                i32::try_from(text.len()).expect("ICU text length"),
                &mut status,
            );
            assert!(status <= 0, "ICU SetText failed: {status}");
        }
        Self {
            searcher: Some(searcher),
            text_len: text.len(),
            local,
            _lease: lease,
            _strings: PhantomData,
        }
    }
    pub fn SetOffset(&mut self, offset: usize) {
        let mut status = 0;
        unsafe {
            usearch_setOffset(
                self.searcher.as_ref().unwrap().pointer.as_ptr(),
                i32::try_from(offset).expect("ICU offset"),
                &mut status,
            )
        };
        assert!(status <= 0, "ICU SetOffset failed: {status}");
    }
    pub fn NextMatch(&mut self) -> Option<(usize, usize)> {
        let pointer = self.searcher.as_ref().unwrap().pointer.as_ptr();
        let mut status = 0;
        let start = unsafe { usearch_next(pointer, &mut status) };
        assert!(status <= 0, "ICU NextMatch failed: {status}");
        if start < 0 || start as usize >= self.text_len {
            return None;
        }
        let length = unsafe { usearch_getMatchedLength(pointer) };
        (length > 0).then_some((start as usize, length.max(0) as usize))
    }
}
impl Drop for TextSearch<'_> {
    fn drop(&mut self) {
        let Some(searcher) = self.searcher.take() else {
            return;
        };
        if self.local {
            return;
        }
        let mut status = 0;
        unsafe {
            usearch_setPattern(searcher.pointer.as_ptr(), NEWLINE.as_ptr(), 1, &mut status);
            assert!(status <= 0);
            status = 0;
            usearch_setText(searcher.pointer.as_ptr(), NEWLINE.as_ptr(), 1, &mut status);
            assert!(status <= 0);
        }
        SEARCHER.with(|pool| {
            let previous = pool.borrow_mut().replace(searcher);
            assert!(
                previous.is_none(),
                "source global TextSearcherIcu cannot overlap; use local tag"
            );
        });
    }
}

enum UNormalizer2 {}
unsafe extern "C" {
    #[link_name=concat!("unorm2_getNFCInstance_",env!("ICU_MAJOR"))]
    fn unorm2_getNFCInstance(status: *mut i32) -> *const UNormalizer2;
    #[link_name=concat!("unorm2_spanQuickCheckYes_",env!("ICU_MAJOR"))]
    fn unorm2_spanQuickCheckYes(
        norm: *const UNormalizer2,
        text: *const u16,
        len: i32,
        status: *mut i32,
    ) -> i32;
    #[link_name=concat!("unorm2_normalizeSecondAndAppend_",env!("ICU_MAJOR"))]
    fn unorm2_normalizeSecondAndAppend(
        norm: *const UNormalizer2,
        first: *mut u16,
        len: i32,
        capacity: i32,
        second: *const u16,
        second_len: i32,
        status: *mut i32,
    ) -> i32;
}
/// unicode_utilities.cc:307-335: preserve an already-NFC prefix, normalize only
/// the suffix and its join, and return Chromium's explicit UTF-16 result copy.
pub fn NormalizeCharactersIntoNfc(text: &[u16]) -> Vec<u16> {
    assert!(!text.is_empty());
    let len = i32::try_from(text.len()).expect("NFC text length");
    let mut status = 0;
    let normalizer = unsafe { unorm2_getNFCInstance(&mut status) };
    assert!(status <= 0 && !normalizer.is_null());
    let prefix = unsafe { unorm2_spanQuickCheckYes(normalizer, text.as_ptr(), len, &mut status) };
    assert!(status <= 0 && prefix >= 0);
    if prefix == len {
        return text.to_vec();
    }
    let prefix = prefix as usize;
    let mut result = vec![0; text.len().max(1)];
    result[..prefix].copy_from_slice(&text[..prefix]);
    let mut written = unsafe {
        unorm2_normalizeSecondAndAppend(
            normalizer,
            result.as_mut_ptr(),
            prefix as i32,
            result.len() as i32,
            text[prefix..].as_ptr(),
            len - prefix as i32,
            &mut status,
        )
    };
    if status == 15 {
        assert!(written >= 0);
        result.resize(written as usize, 0);
        result[..prefix].copy_from_slice(&text[..prefix]);
        status = 0;
        written = unsafe {
            unorm2_normalizeSecondAndAppend(
                normalizer,
                result.as_mut_ptr(),
                prefix as i32,
                written,
                text[prefix..].as_ptr(),
                len - prefix as i32,
                &mut status,
            )
        };
    }
    assert!(
        status <= 0 && written >= 0,
        "ICU NFC normalization failed: {status}"
    );
    result.truncate(written as usize);
    result
}
