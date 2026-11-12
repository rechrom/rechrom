// Copyright 2012 The Chromium Authors
// Use of this source code is governed by a BSD-style license in Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/html/forms/type_ahead.h:42-80
// Source: core/html/forms/type_ahead.cc:39-138

use super::select_type::SelectEvent;
use crate::platform::wtf::text::string_to_number::StringToIntLoose;
use interaction_input::base::time::time::TimeTicks;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

// Source: type_ahead.h:42-49. Borrow the live source collection. The source
// data_source_ pointer is supplied at each transaction instead of retaining a
// trait object whose lifetime would outlive the Page's borrowed DOM ports.
pub trait TypeAheadDataSource {
    fn IndexOfSelectedOption(&self) -> i32;
    fn OptionCount(&self) -> i32;
    fn OptionAtIndex(&self, index: i32) -> Rc<[u16]>;
}
// Source: type_ahead.h:57-62.
pub const kMatchPrefix: u32 = 1 << 0;
pub const kCycleFirstChar: u32 = 1 << 1;
pub const kMatchIndex: u32 = 1 << 2;

// Source: type_ahead.h:73-79. Exactly one session belongs to each select, not
// to its current menu-list/list-box subtype or to an individual input event.
pub struct TypeAhead {
    last_type_time_: Cell<Option<TimeTicks>>,
    repeating_char_: Cell<u16>,
    buffer_: RefCell<Vec<u16>>,
}
impl Default for TypeAhead {
    fn default() -> Self {
        Self::new()
    }
}
impl TypeAhead {
    // Source: type_ahead.cc:39-40.
    pub fn new() -> Self {
        Self {
            last_type_time_: Cell::new(None),
            repeating_char_: Cell::new(0),
            buffer_: RefCell::new(Vec::new()),
        }
    }
}
// Source: type_ahead.cc:42-56.
const kTypeAheadTimeout: i64 = 1_000_000;
fn StripLeadingWhiteSpace(text: &[u16]) -> &[u16] {
    let start = text
        .iter()
        .position(|&c| c != 0xa0 && !foundation::unicode::IsSpaceOrNewline(c))
        .unwrap_or(text.len());
    &text[start..]
}

impl TypeAhead {
    // Source: type_ahead.cc:59-64.
    pub fn ShouldHandleKeyboardEvent(event: &SelectEvent) -> bool {
        event.kind == "keypress"
            && !event.ctrl
            && !event.alt
            && !event.meta
            && foundation::unicode::IsPrintableChar(event.char_code)
    }
    // Source: type_ahead.cc:66-126. The session holds only the source UTF-16
    // buffer; collection lookups and selected index always read live DOM.
    pub fn HandleEvent(
        &self,
        data_source: &impl TypeAheadDataSource,
        event: &SelectEvent,
        char_code: u16,
        match_mode: u32,
    ) -> i32 {
        let event_time = TimeTicks::FromInternalValue(event.platform_time_us);
        if let Some(last) = self.last_type_time_.get() {
            if event_time < last {
                return -1;
            }
            if i128::from(event_time.ToInternalValue()) - i128::from(last.ToInternalValue())
                > i128::from(kTypeAheadTimeout)
            {
                self.buffer_.borrow_mut().clear();
            }
        } else {
            debug_assert!(self.buffer_.borrow().is_empty());
        }
        self.last_type_time_.set(Some(event_time));
        self.buffer_.borrow_mut().push(char_code);
        let option_count = data_source.OptionCount();
        if option_count < 1 {
            return -1;
        }
        let mut search_start_offset = 1;
        let mut prefix = Vec::new();
        if match_mode & kCycleFirstChar != 0 && char_code == self.repeating_char_.get() {
            prefix.push(char_code);
            self.repeating_char_.set(char_code);
        } else if match_mode & kMatchPrefix != 0 {
            prefix = self.buffer_.borrow().clone();
            if self.buffer_.borrow().len() > 1 {
                self.repeating_char_.set(0);
                search_start_offset = 0;
            } else {
                self.repeating_char_.set(char_code);
            }
        }
        if !prefix.is_empty() {
            let selected = data_source.IndexOfSelectedOption();
            let mut index =
                ((if selected < 0 { 0 } else { selected }) + search_start_offset) % option_count;
            for _ in 0..option_count {
                let text = data_source.OptionAtIndex(index);
                if foundation::unicode::StartsWithIgnoringCaseAndAccents(
                    StripLeadingWhiteSpace(&text),
                    &prefix,
                ) {
                    return index;
                }
                index = (index + 1) % option_count;
            }
        }
        if match_mode & kMatchIndex != 0 {
            let index = StringToIntLoose(&self.buffer_.borrow()).unwrap_or(0);
            if index > 0 && index <= option_count {
                return index - 1;
            }
        }
        -1
    }
    // Source: type_ahead.cc:128-133. As in Chromium, an older event has a
    // negative delta and therefore still falls within the active-session test.
    pub fn HasActiveSession(&self, event: &SelectEvent) -> bool {
        self.last_type_time_.get().is_some_and(|last| {
            i128::from(event.platform_time_us) - i128::from(last.ToInternalValue())
                <= i128::from(kTypeAheadTimeout)
        })
    }
    // Source: type_ahead.cc:135-138. repeating_char_ intentionally survives.
    pub fn ResetSession(&self) {
        self.last_type_time_.set(None);
        self.buffer_.borrow_mut().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Options {
        labels: Vec<Rc<[u16]>>,
        selected: Cell<i32>,
    }
    impl TypeAheadDataSource for Options {
        fn IndexOfSelectedOption(&self) -> i32 {
            self.selected.get()
        }
        fn OptionCount(&self) -> i32 {
            self.labels.len() as i32
        }
        fn OptionAtIndex(&self, i: i32) -> Rc<[u16]> {
            self.labels[i as usize].clone()
        }
    }
    fn options(labels: &[&str]) -> Options {
        Options {
            labels: labels
                .iter()
                .map(|s| Rc::from(s.encode_utf16().collect::<Vec<_>>()))
                .collect(),
            selected: Cell::new(0),
        }
    }
    fn event(c: char, time: i64) -> SelectEvent {
        SelectEvent {
            kind: "keypress".into(),
            keyboard: true,
            char_code: c as i32,
            platform_time_us: time,
            ..Default::default()
        }
    }
    #[test]
    fn prefix_cycle_accent_and_timeout_follow_source_session() {
        // Match the embedding's pre-worker ICU initialization. C.UTF-8's
        // en_US_POSIX tailoring otherwise assigns case primary weights even
        // at UCOL_PRIMARY; a browser UI locale uses normal Unicode collation.
        icu_bidi::string_search::InitializeDefaultLocale(c"").unwrap();
        let options = options(&["zero", "\u{a0}Éclair", "Echo", "École"]);
        let session = TypeAhead::new();
        let mode = kMatchPrefix | kCycleFirstChar;
        assert_eq!(
            session.HandleEvent(&options, &event('e', 10), b'e' as u16, mode),
            1
        );
        options.selected.set(1);
        assert_eq!(
            session.HandleEvent(&options, &event('e', 20), b'e' as u16, mode),
            2
        );
        options.selected.set(2);
        assert_eq!(
            session.HandleEvent(&options, &event('c', 30), b'c' as u16, mode),
            -1
        );
        assert_eq!(
            session.HandleEvent(&options, &event('e', 1_000_031), b'e' as u16, mode),
            3
        );
        options.selected.set(3);
        assert_eq!(
            session.HandleEvent(&options, &event('c', 1_000_032), b'c' as u16, mode),
            3
        );
    }
    #[test]
    fn backward_time_reset_and_modifiers_keep_source_rules() {
        let options = options(&["zero", "one", "two"]);
        let session = TypeAhead::new();
        assert_eq!(
            session.HandleEvent(&options, &event('2', 50), b'2' as u16, kMatchIndex),
            1
        );
        assert_eq!(
            session.HandleEvent(&options, &event('3', 49), b'3' as u16, kMatchIndex),
            -1
        );
        assert!(session.HasActiveSession(&event('x', 49)));
        session.ResetSession();
        assert!(!session.HasActiveSession(&event('x', 50)));
        assert_eq!(
            session.HandleEvent(&options, &event('3', 50), b'3' as u16, kMatchIndex),
            2
        );
        let mut printable = event('é', 50);
        assert!(TypeAhead::ShouldHandleKeyboardEvent(&printable));
        printable.ctrl = true;
        assert!(!TypeAhead::ShouldHandleKeyboardEvent(&printable));
        assert!(!TypeAhead::ShouldHandleKeyboardEvent(&event('\r', 50)));
    }
}
