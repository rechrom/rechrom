#![allow(non_snake_case, non_upper_case_globals)]

use foundation::UChar;

use super::segmented_string::SegmentedString;

// cpp: html/parser/input_stream_preprocessor.h:38
pub const kEndOfFileMarker: u8 = 0;

// cpp: html/parser/input_stream_preprocessor.h:40-41
pub const kReplacementCharacter: UChar = 0xFFFD;

// The template's only required Tokenizer operation is declared on the owner
// interface here. HTMLTokenizer supplies this implementation when translated.
pub trait TokenizerNullPolicy {
    fn ShouldSkipNullCharacters(&self) -> bool;
}

// cpp: html/parser/input_stream_preprocessor.h:43-52,170-172
pub struct InputStreamPreprocessor<Tokenizer: TokenizerNullPolicy> {
    // The source object stores its owning tokenizer's self-pointer. This raw
    // pointer retains that relationship without inventing an owned tokenizer.
    tokenizer_: *const Tokenizer,
    skip_next_new_line_: bool,
}

impl<Tokenizer: TokenizerNullPolicy> InputStreamPreprocessor<Tokenizer> {
    // cpp: html/parser/input_stream_preprocessor.h:49-50
    pub fn new(tokenizer: *const Tokenizer) -> Self {
        Self {
            tokenizer_: tokenizer,
            skip_next_new_line_: false,
        }
    }

    // cpp: html/parser/input_stream_preprocessor.h:54-61
    pub fn Peek(&mut self, source: &mut SegmentedString, cc: &mut UChar) -> bool {
        *cc = source.CurrentChar();
        self.ProcessNextInputCharacter(source, cc)
    }

    // cpp: html/parser/input_stream_preprocessor.h:63-67
    pub fn Advance(&mut self, source: &mut SegmentedString, cc: &mut UChar) -> bool {
        *cc = source.AdvanceAndUpdateLineNumber();
        self.ProcessNextInputCharacter(source, cc)
    }

    // cpp: html/parser/input_stream_preprocessor.h:69-72
    pub fn AdvancePastNonNewline(&mut self, source: &mut SegmentedString, cc: &mut UChar) -> bool {
        *cc = source.AdvancePastNonNewline();
        self.ProcessNextInputCharacter(source, cc)
    }

    // cpp: html/parser/input_stream_preprocessor.h:74-90
    pub fn AdvancePastCarriageReturn(
        &mut self,
        source: &mut SegmentedString,
        cc: &mut UChar,
    ) -> bool {
        debug_assert_eq!(*cc, b'\r' as UChar);
        *cc = source.AdvancePastNonNewline();
        if source.IsEmpty() {
            self.skip_next_new_line_ = true;
            return false;
        }
        if *cc == b'\n' as UChar {
            *cc = source.AdvancePastNewlineAndUpdateLineNumber();
            if source.IsEmpty() {
                return false;
            }
        }
        true
    }

    // cpp: html/parser/input_stream_preprocessor.h:92-112
    pub fn ProcessNullCharacter(&mut self, source: &mut SegmentedString, cc: &mut UChar) -> bool {
        debug_assert_eq!(*cc, 0);
        if source.IsEmpty() {
            return false;
        }
        if self.ShouldTreatNullAsEndOfFileMarker(source) {
            return true;
        }
        if !self.should_skip_null_characters() {
            *cc = kReplacementCharacter;
            return true;
        }
        *cc = source.AdvancePastNonNewline();
        while *cc == 0 {
            if source.IsEmpty() {
                return false;
            }
            if self.ShouldTreatNullAsEndOfFileMarker(source) {
                return true;
            }
            *cc = source.AdvancePastNonNewline();
        }
        true
    }

    // cpp: html/parser/input_stream_preprocessor.h:115-131
    fn ProcessNextInputCharacter(&mut self, source: &mut SegmentedString, cc: &mut UChar) -> bool {
        const kSpecialCharacterMask: UChar = (b'\n' | b'\r' | 0) as UChar;
        if (*cc & !kSpecialCharacterMask) != 0 {
            self.skip_next_new_line_ = false;
            return true;
        }
        if source.IsEmpty() {
            return false;
        }
        self.ProcessNextInputSpecialCharacter(source, cc)
    }

    // cpp: html/parser/input_stream_preprocessor.h:133-164
    fn ProcessNextInputSpecialCharacter(
        &mut self,
        source: &mut SegmentedString,
        cc: &mut UChar,
    ) -> bool {
        loop {
            debug_assert_eq!(*cc, source.CurrentChar());
            if *cc == b'\n' as UChar && self.skip_next_new_line_ {
                self.skip_next_new_line_ = false;
                *cc = source.AdvancePastNewlineAndUpdateLineNumber();
                if source.IsEmpty() {
                    return false;
                }
            }
            if *cc == b'\r' as UChar {
                *cc = b'\n' as UChar;
                self.skip_next_new_line_ = true;
            } else {
                self.skip_next_new_line_ = false;
                if *cc == 0 && !self.ShouldTreatNullAsEndOfFileMarker(source) {
                    if self.should_skip_null_characters() {
                        *cc = source.AdvancePastNonNewline();
                        if source.IsEmpty() {
                            return false;
                        }
                        continue;
                    }
                    *cc = kReplacementCharacter;
                }
            }
            return true;
        }
    }

    // cpp: html/parser/input_stream_preprocessor.h:166-168
    fn ShouldTreatNullAsEndOfFileMarker(&self, source: &SegmentedString) -> bool {
        source.IsClosed() && source.length() == 1
    }

    fn should_skip_null_characters(&self) -> bool {
        // SAFETY: HTMLTokenizer owns this preprocessor and supplies its own
        // stable pointer; the owner must outlive every preprocessing call.
        unsafe { (*self.tokenizer_).ShouldSkipNullCharacters() }
    }
}
