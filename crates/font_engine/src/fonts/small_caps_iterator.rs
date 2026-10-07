// C++: font_engine/fonts/small_caps_iterator.h.
// The supplied source declares the constructor and Consume but defines neither.
// This records the value layout; no iteration behavior is invented.
use super::utf16_text_iterator::UTF16TextIterator;

// cpp: font_engine/fonts/small_caps_iterator.h:20-24
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SmallCapsBehavior {
    kSmallCapsSameCase,
    kSmallCapsUppercaseNeeded,
    kSmallCapsInvalid,
}

// cpp: font_engine/fonts/small_caps_iterator.h:33-38
pub struct SmallCapsIterator<'a> {
    utf16_iterator_: UTF16TextIterator<'a>,
    next_u_char32_: i32,
    at_end_: bool,
    current_small_caps_behavior_: SmallCapsBehavior,
    previous_small_caps_behavior_: SmallCapsBehavior,
}

// Missing source definitions: small_caps_iterator.h:26 and :30.
