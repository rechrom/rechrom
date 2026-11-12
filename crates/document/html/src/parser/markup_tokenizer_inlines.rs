#![allow(non_snake_case, unused_macros, unused_imports)]

use foundation::UChar;

// cpp: html/parser/markup_tokenizer_inlines.h:38-40
pub fn IsTokenizerWhitespace(cc: UChar) -> bool {
    matches!(cc, 0x20 | 0x0A | 0x09 | 0x0C)
}

// cpp: html/parser/markup_tokenizer_inlines.h:48-51
// BEGIN_STATE and BEGIN_STATE_NOLABEL both become match arms; the C++ goto
// label is replaced by continuing the caller's labeled state loop.
macro_rules! begin_states {
    ($owner:ident, $( $state:path => $body:block ),+ $(,)?) => {{
        match $owner.state_ {
            $( $state => $body, )+
        }
    }};
}
pub(crate) use begin_states;

// cpp: html/parser/markup_tokenizer_inlines.h:52
macro_rules! end_state {
    () => {
        unreachable!("tokenizer state body must transfer control")
    };
}
pub(crate) use end_state;

// cpp: html/parser/markup_tokenizer_inlines.h:54-61
macro_rules! reconsume_in {
    ($owner:ident, $state:expr, $loop:lifetime) => {{
        debug_assert_ne!($owner.state_, $state);
        $owner.state_ = $state;
        continue $loop;
    }};
}
pub(crate) use reconsume_in;

// cpp: html/parser/markup_tokenizer_inlines.h:63-72
macro_rules! advance_to {
    ($owner:ident, $source:ident, $cc:ident, $state:expr, $loop:lifetime) => {{
        debug_assert_ne!($owner.state_, $state);
        $owner.state_ = $state;
        if !$owner
            .input_stream_preprocessor_
            .as_mut()
            .expect("tokenizer preprocessor must be initialized")
            .Advance($source, &mut $cc)
        {
            return $owner.HaveBufferedCharacterToken();
        }
        continue $loop;
    }};
}
pub(crate) use advance_to;

// cpp: html/parser/markup_tokenizer_inlines.h:74-84
macro_rules! advance_past_non_newline_to {
    ($owner:ident, $source:ident, $cc:ident, $state:expr, $loop:lifetime) => {{
        debug_assert_ne!($owner.state_, $state);
        $owner.state_ = $state;
        if !$owner
            .input_stream_preprocessor_
            .as_mut()
            .expect("tokenizer preprocessor must be initialized")
            .AdvancePastNonNewline($source, &mut $cc)
        {
            return $owner.HaveBufferedCharacterToken();
        }
        continue $loop;
    }};
}
pub(crate) use advance_past_non_newline_to;

// cpp: html/parser/markup_tokenizer_inlines.h:86-94
macro_rules! consume {
    ($owner:ident, $source:ident, $cc:ident, $state:expr, $loop:lifetime) => {{
        debug_assert_eq!($owner.state_, $state);
        if !$owner
            .input_stream_preprocessor_
            .as_mut()
            .expect("tokenizer preprocessor must be initialized")
            .Advance($source, &mut $cc)
        {
            return $owner.HaveBufferedCharacterToken();
        }
        continue $loop;
    }};
}
pub(crate) use consume;

// cpp: html/parser/markup_tokenizer_inlines.h:96-105
macro_rules! consume_non_newline {
    ($owner:ident, $source:ident, $cc:ident, $state:expr, $loop:lifetime) => {{
        debug_assert_eq!($owner.state_, $state);
        if !$owner
            .input_stream_preprocessor_
            .as_mut()
            .expect("tokenizer preprocessor must be initialized")
            .AdvancePastNonNewline($source, &mut $cc)
        {
            return $owner.HaveBufferedCharacterToken();
        }
        continue $loop;
    }};
}
pub(crate) use consume_non_newline;

// cpp: html/parser/markup_tokenizer_inlines.h:107-118
macro_rules! switch_to {
    ($owner:ident, $source:ident, $cc:ident, $state:expr, $loop:lifetime) => {{
        debug_assert_ne!($owner.state_, $state);
        $owner.state_ = $state;
        if $source.IsEmpty()
            || !$owner
                .input_stream_preprocessor_
                .as_mut()
                .expect("tokenizer preprocessor must be initialized")
                .Peek($source, &mut $cc)
        {
            return $owner.HaveBufferedCharacterToken();
        }
        continue $loop;
    }};
}
pub(crate) use switch_to;
