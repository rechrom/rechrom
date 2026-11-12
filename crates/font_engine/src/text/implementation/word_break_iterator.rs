/*
 * Copyright (C) 2006 Lars Knoll <lars@trolltech.com>
 * Copyright (C) 2007, 2011, 2012 Apple Inc. All rights reserved.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Library General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Library General Public License for more details.
 *
 * You should have received a copy of the GNU Library General Public License
 * along with this library; see the file COPYING.LIB.  If not, write to
 * the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA 02110-1301, USA.
 *
 */

// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: platform/text/text_break_iterator_icu.cc:633-704,819-845.
// The UI-language locale is supplied by its existing embedding owner, as in
// CurrentTextBreakLocaleID; it is not inferred from the document's language.
use icu_bidi::ubrk_ffi as icu;
use std::{
    cell::RefCell,
    ffi::{CStr, CString},
    marker::PhantomData,
    ptr::NonNull,
};

pub const kTextBreakDone: i32 = icu::UBRK_DONE;

struct PooledIterator {
    native: NonNull<icu::UBreakIterator>,
    locale: CString,
}
impl Drop for PooledIterator {
    fn drop(&mut self) {
        unsafe { icu::ubrk_close(self.native.as_ptr()) };
    }
}
thread_local! {
    static WORD: RefCell<Option<PooledIterator>> = const { RefCell::new(None) };
    static SENTENCE: RefCell<Option<PooledIterator>> = const { RefCell::new(None) };
}

/// A scoped borrow of the thread-local ICU owner and the caller's UTF-16 text.
/// The native iterator cannot escape the text's lifetime or be used after the
/// next source SetText16. No boundary arrays or second segmentation model exist.
pub struct TextBreakIterator<'a> {
    native: NonNull<icu::UBreakIterator>,
    _text: PhantomData<&'a [u16]>,
}
impl TextBreakIterator<'_> {
    pub fn First(&mut self) -> i32 {
        unsafe { icu::ubrk_first(self.native.as_ptr()) }
    }
    pub fn Next(&mut self) -> i32 {
        unsafe { icu::ubrk_next(self.native.as_ptr()) }
    }
    pub fn Following(&mut self, offset: i32) -> i32 {
        unsafe { icu::ubrk_following(self.native.as_ptr(), offset) }
    }
    pub fn Preceding(&mut self, offset: i32) -> i32 {
        unsafe { icu::ubrk_preceding(self.native.as_ptr(), offset) }
    }
    pub fn IsBoundary(&mut self, offset: i32) -> bool {
        unsafe { icu::ubrk_isBoundary(self.native.as_ptr(), offset) != 0 }
    }
    pub fn IsWordTextBreak(&self) -> bool {
        unsafe { icu::ubrk_getRuleStatus(self.native.as_ptr()) != icu::UBRK_WORD_NONE }
    }
}
impl Drop for TextBreakIterator<'_> {
    fn drop(&mut self) {
        // ICU retains the supplied buffer. Clear it while the borrow is alive,
        // including on unwinding; the pool itself remains the original owner.
        static EMPTY: [u16; 1] = [0];
        let mut error = 0;
        unsafe { icu::ubrk_setText(self.native.as_ptr(), EMPTY.as_ptr(), 0, &mut error) };
    }
}
fn WithIterator<R>(
    pool: &'static std::thread::LocalKey<RefCell<Option<PooledIterator>>>,
    kind: i32,
    text: &[u16],
    locale: &CStr,
    find: impl FnOnce(&mut TextBreakIterator<'_>) -> R,
) -> Option<R> {
    let length = i32::try_from(text.len()).ok()?;
    pool.with(|pool| {
        let mut pool = pool.borrow_mut();
        if pool
            .as_ref()
            .is_some_and(|iterator| iterator.locale.as_c_str() != locale)
        {
            *pool = None;
        }
        if pool.is_none() {
            let mut error = 0;
            let native =
                unsafe { icu::ubrk_open(kind, locale.as_ptr(), std::ptr::null(), 0, &mut error) };
            if error > 0 {
                if !native.is_null() {
                    unsafe { icu::ubrk_close(native) };
                }
                return None;
            }
            *pool = Some(PooledIterator {
                native: NonNull::new(native)?,
                locale: locale.to_owned(),
            });
        }
        let native = pool.as_ref()?.native;
        let mut error = 0;
        unsafe { icu::ubrk_setText(native.as_ptr(), text.as_ptr(), length, &mut error) };
        if error > 0 {
            return None;
        }
        let mut iterator = TextBreakIterator {
            native,
            _text: PhantomData,
        };
        Some(find(&mut iterator))
    })
}
pub fn WordBreakIterator<R>(
    text: &[u16],
    ui_locale: &CStr,
    find: impl FnOnce(&mut TextBreakIterator<'_>) -> R,
) -> Option<R> {
    WithIterator(&WORD, icu::UBRK_WORD, text, ui_locale, find)
}
pub fn SentenceBreakIterator<R>(
    text: &[u16],
    ui_locale: &CStr,
    find: impl FnOnce(&mut TextBreakIterator<'_>) -> R,
) -> Option<R> {
    WithIterator(&SENTENCE, icu::UBRK_SENTENCE, text, ui_locale, find)
}
