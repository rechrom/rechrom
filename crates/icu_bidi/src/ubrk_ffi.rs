//! ICU unicode/ubrk.h ABI used by Blink's native word/sentence iterators.
#![allow(non_snake_case)]
use std::ffi::c_char;
pub enum UBreakIterator {}
pub const UBRK_WORD: i32 = 1;
pub const UBRK_SENTENCE: i32 = 3;
pub const UBRK_DONE: i32 = -1;
pub const UBRK_WORD_NONE: i32 = 0;
unsafe extern "C" {
    #[link_name = concat!("ubrk_open_", env!("ICU_MAJOR"))]
    pub fn ubrk_open(
        kind: i32,
        locale: *const c_char,
        text: *const u16,
        length: i32,
        error: *mut i32,
    ) -> *mut UBreakIterator;
    #[link_name = concat!("ubrk_close_", env!("ICU_MAJOR"))]
    pub fn ubrk_close(iterator: *mut UBreakIterator);
    #[link_name = concat!("ubrk_setText_", env!("ICU_MAJOR"))]
    pub fn ubrk_setText(
        iterator: *mut UBreakIterator,
        text: *const u16,
        length: i32,
        error: *mut i32,
    );
    #[link_name = concat!("ubrk_first_", env!("ICU_MAJOR"))]
    pub fn ubrk_first(iterator: *mut UBreakIterator) -> i32;
    #[link_name = concat!("ubrk_next_", env!("ICU_MAJOR"))]
    pub fn ubrk_next(iterator: *mut UBreakIterator) -> i32;
    #[link_name = concat!("ubrk_following_", env!("ICU_MAJOR"))]
    pub fn ubrk_following(iterator: *mut UBreakIterator, offset: i32) -> i32;
    #[link_name = concat!("ubrk_preceding_", env!("ICU_MAJOR"))]
    pub fn ubrk_preceding(iterator: *mut UBreakIterator, offset: i32) -> i32;
    #[link_name = concat!("ubrk_isBoundary_", env!("ICU_MAJOR"))]
    pub fn ubrk_isBoundary(iterator: *mut UBreakIterator, offset: i32) -> i8;
    #[link_name = concat!("ubrk_getRuleStatus_", env!("ICU_MAJOR"))]
    pub fn ubrk_getRuleStatus(iterator: *mut UBreakIterator) -> i32;
}
