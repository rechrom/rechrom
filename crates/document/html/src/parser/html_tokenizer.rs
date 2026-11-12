#![allow(non_snake_case, non_camel_case_types)]

use foundation::{BlinkString, UChar};
use std::marker::PhantomPinned;
use std::pin::Pin;

use crate::html_tag_names::{html_names::HTMLTag, LookupHtmlTag};

use super::html_attributes_ranges::HTMLAttributesRanges;
use super::html_entity_parser::{ConsumeHTMLEntity, DecodedHTMLEntity};
use super::html_parser_options::HTMLParserOptions;
use super::html_token::{HTMLToken, TokenType};
use super::input_stream_preprocessor::{
    kEndOfFileMarker, InputStreamPreprocessor, TokenizerNullPolicy,
};
use super::literal_buffer::{LCharLiteralBuffer, UCharLiteralBuffer};
use super::markup_tokenizer_inlines::{
    advance_past_non_newline_to, advance_to, consume, consume_non_newline, reconsume_in, switch_to,
    IsTokenizerWhitespace,
};
use super::segmented_string::{LookAheadResult, SegmentedString};

// cpp: html/parser/html_tokenizer.cc:59-79
struct ScanFlags;

impl ScanFlags {
    const kNullCharacter: u16 = 1 << 0;
    const kNewlineOrCarriageReturn: u16 = 1 << 1;
    const kWhitespaceNotNewline: u16 = 1 << 2;
    const kAmpersand: u16 = 1 << 3;
    const kOpenTag: u16 = 1 << 4;
    const kSlashAndCloseTag: u16 = 1 << 5;
    const kEqual: u16 = 1 << 6;
    const kQuotes: u16 = 1 << 7;
    const kOpenBrace: u16 = 1 << 8;
    const kWhitespace: u16 = Self::kWhitespaceNotNewline | Self::kNewlineOrCarriageReturn;
    const kCharacterTokenSpecial: u16 = Self::kNullCharacter
        | Self::kNewlineOrCarriageReturn
        | Self::kAmpersand
        | Self::kOpenTag
        | Self::kOpenBrace;
    const kNullOrNewline: u16 = Self::kNullCharacter | Self::kNewlineOrCarriageReturn;
    const kRCDATASpecial: u16 = Self::kNullCharacter | Self::kAmpersand | Self::kOpenTag;
    const kTagNameSpecial: u16 = Self::kWhitespace | Self::kSlashAndCloseTag | Self::kNullCharacter;
    const kAttributeNameSpecial: u16 = Self::kWhitespace
        | Self::kSlashAndCloseTag
        | Self::kNullCharacter
        | Self::kEqual
        | Self::kOpenTag
        | Self::kQuotes;
}

// cpp: html/parser/html_tokenizer.cc:81-106
const fn CreateScanFlags(cc: UChar) -> u16 {
    assert!(cc <= 0x7F);
    if cc == 0 {
        ScanFlags::kNullCharacter
    } else if cc == b'\n' as UChar || cc == b'\r' as UChar {
        ScanFlags::kNewlineOrCarriageReturn
    } else if cc == b' ' as UChar || cc == b'\t' as UChar || cc == b'\x0C' as UChar {
        ScanFlags::kWhitespaceNotNewline
    } else if cc == b'&' as UChar {
        ScanFlags::kAmpersand
    } else if cc == b'<' as UChar {
        ScanFlags::kOpenTag
    } else if cc == b'/' as UChar || cc == b'>' as UChar {
        ScanFlags::kSlashAndCloseTag
    } else if cc == b'=' as UChar {
        ScanFlags::kEqual
    } else if cc == b'"' as UChar || cc == b'\'' as UChar {
        ScanFlags::kQuotes
    } else if cc == b'{' as UChar {
        ScanFlags::kOpenBrace
    } else {
        0
    }
}

// cpp: html/parser/html_tokenizer.cc:39-56,108-110
const CHARACTER_SCAN_FLAGS: [u16; 128] = {
    let mut flags = [0; 128];
    let mut cc = 0;
    while cc < 128 {
        flags[cc] = CreateScanFlags(cc as UChar);
        cc += 1;
    }
    flags
};

// cpp: html/parser/html_tokenizer.cc:112-115
fn ToLowerCase(cc: UChar) -> UChar {
    debug_assert!((cc as u8).is_ascii_alphabetic() && cc <= 0x7F);
    cc | 0x20
}

// cpp: html/parser/html_tokenizer.cc:117-120
fn CheckScanFlag(cc: UChar, flag: u16) -> bool {
    cc <= 0x7F && (CHARACTER_SCAN_FLAGS[cc as usize] & flag) != 0
}

// cpp: html/parser/html_tokenizer.cc:122-124
fn ToLowerCaseIfAlpha(cc: UChar) -> UChar {
    if cc <= 0x7F {
        (cc as u8).to_ascii_lowercase() as UChar
    } else {
        cc
    }
}

// cpp: html/parser/html_tokenizer.cc:204-218
macro_rules! flush_and_advance_to {
    ($owner:ident, $source:ident, $cc:ident, $state:expr, $may_newline:expr, $loop:lifetime) => {{
        $owner.state_ = $state;
        if $owner.FlushBufferedEndTag($source, $may_newline) {
            return true;
        }
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

// cpp: html/parser/html_tokenizer.cc:127-138
fn VectorEqualsString<const INLINE: usize>(
    vector: &LCharLiteralBuffer<INLINE>,
    string: &foundation::BlinkString,
) -> bool {
    if vector.size() != string.length() {
        return false;
    }
    if string.length() == 0 {
        return true;
    }
    vector
        .as_slice()
        .iter()
        .map(|&byte| UChar::from(byte))
        .eq(string.Span16().expect("nonempty string").iter().copied())
}

// cpp: html/parser/html_tokenizer.h:56-136
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum State {
    // cpp: html/parser/html_tokenizer.h:57
    kDataState,
    // cpp: html/parser/html_tokenizer.h:58
    kCharacterReferenceInDataState,
    // cpp: html/parser/html_tokenizer.h:59
    kRCDATAState,
    // cpp: html/parser/html_tokenizer.h:60
    kCharacterReferenceInRCDATAState,
    // cpp: html/parser/html_tokenizer.h:61
    kRAWTEXTState,
    // cpp: html/parser/html_tokenizer.h:62
    kScriptDataState,
    // cpp: html/parser/html_tokenizer.h:63
    kPLAINTEXTState,
    // cpp: html/parser/html_tokenizer.h:64
    kTagOpenState,
    // cpp: html/parser/html_tokenizer.h:65
    kEndTagOpenState,
    // cpp: html/parser/html_tokenizer.h:66
    kTagNameState,
    // cpp: html/parser/html_tokenizer.h:67
    kRCDATALessThanSignState,
    // cpp: html/parser/html_tokenizer.h:68
    kRCDATAEndTagOpenState,
    // cpp: html/parser/html_tokenizer.h:69
    kRCDATAEndTagNameState,
    // cpp: html/parser/html_tokenizer.h:70
    kRAWTEXTLessThanSignState,
    // cpp: html/parser/html_tokenizer.h:71
    kRAWTEXTEndTagOpenState,
    // cpp: html/parser/html_tokenizer.h:72
    kRAWTEXTEndTagNameState,
    // cpp: html/parser/html_tokenizer.h:73
    kScriptDataLessThanSignState,
    // cpp: html/parser/html_tokenizer.h:74
    kScriptDataEndTagOpenState,
    // cpp: html/parser/html_tokenizer.h:75
    kScriptDataEndTagNameState,
    // cpp: html/parser/html_tokenizer.h:76
    kScriptDataEscapeStartState,
    // cpp: html/parser/html_tokenizer.h:77
    kScriptDataEscapeStartDashState,
    // cpp: html/parser/html_tokenizer.h:78
    kScriptDataEscapedState,
    // cpp: html/parser/html_tokenizer.h:79
    kScriptDataEscapedDashState,
    // cpp: html/parser/html_tokenizer.h:80
    kScriptDataEscapedDashDashState,
    // cpp: html/parser/html_tokenizer.h:81
    kScriptDataEscapedLessThanSignState,
    // cpp: html/parser/html_tokenizer.h:82
    kScriptDataEscapedEndTagOpenState,
    // cpp: html/parser/html_tokenizer.h:83
    kScriptDataEscapedEndTagNameState,
    // cpp: html/parser/html_tokenizer.h:84
    kScriptDataDoubleEscapeStartState,
    // cpp: html/parser/html_tokenizer.h:85
    kScriptDataDoubleEscapedState,
    // cpp: html/parser/html_tokenizer.h:86
    kScriptDataDoubleEscapedDashState,
    // cpp: html/parser/html_tokenizer.h:87
    kScriptDataDoubleEscapedDashDashState,
    // cpp: html/parser/html_tokenizer.h:88
    kScriptDataDoubleEscapedLessThanSignState,
    // cpp: html/parser/html_tokenizer.h:89
    kScriptDataDoubleEscapeEndState,
    // cpp: html/parser/html_tokenizer.h:90
    kBeforeAttributeNameState,
    // cpp: html/parser/html_tokenizer.h:91
    kAttributeNameState,
    // cpp: html/parser/html_tokenizer.h:92
    kAfterAttributeNameState,
    // cpp: html/parser/html_tokenizer.h:93
    kBeforeAttributeValueState,
    // cpp: html/parser/html_tokenizer.h:94
    kAttributeValueDoubleQuotedState,
    // cpp: html/parser/html_tokenizer.h:95
    kAttributeValueSingleQuotedState,
    // cpp: html/parser/html_tokenizer.h:96
    kAttributeValueUnquotedState,
    // cpp: html/parser/html_tokenizer.h:97
    kCharacterReferenceInAttributeValueState,
    // cpp: html/parser/html_tokenizer.h:98
    kAfterAttributeValueQuotedState,
    // cpp: html/parser/html_tokenizer.h:99
    kSelfClosingStartTagState,
    // cpp: html/parser/html_tokenizer.h:100
    kProcessingInstructionOpenState,
    // cpp: html/parser/html_tokenizer.h:101
    kProcessingInstructionTargetState,
    // cpp: html/parser/html_tokenizer.h:102
    kAfterProcessingInstructionTargetState,
    // cpp: html/parser/html_tokenizer.h:103
    kProcessingInstructionDataState,
    // cpp: html/parser/html_tokenizer.h:104
    kProcessingInstructionQuestionableState,
    // cpp: html/parser/html_tokenizer.h:105
    kBogusCommentState,
    // cpp: html/parser/html_tokenizer.h:109
    kContinueBogusCommentState,
    // cpp: html/parser/html_tokenizer.h:110
    kMarkupDeclarationOpenState,
    // cpp: html/parser/html_tokenizer.h:111
    kCommentStartState,
    // cpp: html/parser/html_tokenizer.h:112
    kCommentStartDashState,
    // cpp: html/parser/html_tokenizer.h:113
    kCommentState,
    // cpp: html/parser/html_tokenizer.h:114
    kCommentEndDashState,
    // cpp: html/parser/html_tokenizer.h:115
    kCommentEndState,
    // cpp: html/parser/html_tokenizer.h:116
    kCommentEndBangState,
    // cpp: html/parser/html_tokenizer.h:117
    kDOCTYPEState,
    // cpp: html/parser/html_tokenizer.h:118
    kBeforeDOCTYPENameState,
    // cpp: html/parser/html_tokenizer.h:119
    kDOCTYPENameState,
    // cpp: html/parser/html_tokenizer.h:120
    kAfterDOCTYPENameState,
    // cpp: html/parser/html_tokenizer.h:121
    kAfterDOCTYPEPublicKeywordState,
    // cpp: html/parser/html_tokenizer.h:122
    kBeforeDOCTYPEPublicIdentifierState,
    // cpp: html/parser/html_tokenizer.h:123
    kDOCTYPEPublicIdentifierDoubleQuotedState,
    // cpp: html/parser/html_tokenizer.h:124
    kDOCTYPEPublicIdentifierSingleQuotedState,
    // cpp: html/parser/html_tokenizer.h:125
    kAfterDOCTYPEPublicIdentifierState,
    // cpp: html/parser/html_tokenizer.h:126
    kBetweenDOCTYPEPublicAndSystemIdentifiersState,
    // cpp: html/parser/html_tokenizer.h:127
    kAfterDOCTYPESystemKeywordState,
    // cpp: html/parser/html_tokenizer.h:128
    kBeforeDOCTYPESystemIdentifierState,
    // cpp: html/parser/html_tokenizer.h:129
    kDOCTYPESystemIdentifierDoubleQuotedState,
    // cpp: html/parser/html_tokenizer.h:130
    kDOCTYPESystemIdentifierSingleQuotedState,
    // cpp: html/parser/html_tokenizer.h:131
    kAfterDOCTYPESystemIdentifierState,
    // cpp: html/parser/html_tokenizer.h:132
    kBogusDOCTYPEState,
    // cpp: html/parser/html_tokenizer.h:133
    kCDATASectionState,
    // cpp: html/parser/html_tokenizer.h:134
    kCDATASectionBracketState,
    // cpp: html/parser/html_tokenizer.h:135
    kCDATASectionEndState,
}

// cpp: html/parser/html_tokenizer.h:43-50,307-340
pub struct HTMLTokenizer {
    state_: State,
    force_null_character_replacement_: bool,
    should_allow_cdata_: bool,
    truncated_markup_declaration_enabled_: bool,
    should_allow_dom_parts_: bool,
    options_: HTMLParserOptions,
    additional_allowed_character_: UChar,
    // The source stores `this` in its preprocessor. Option is only used while
    // constructing a future pinned owner; no parsing call may observe None.
    input_stream_preprocessor_: Option<InputStreamPreprocessor<HTMLTokenizer>>,
    appropriate_end_tag_name_: UCharLiteralBuffer<32>,
    temporary_buffer_: LCharLiteralBuffer<32>,
    buffered_end_tag_name_: LCharLiteralBuffer<32>,
    attributes_ranges_: HTMLAttributesRanges,
    token_: HTMLToken,
    #[cfg(debug_assertions)]
    token_should_be_in_uninitialized_state_: bool,
    _pin: PhantomPinned,
}

impl TokenizerNullPolicy for HTMLTokenizer {
    // cpp: html/parser/html_tokenizer.h:191-196
    fn ShouldSkipNullCharacters(&self) -> bool {
        !self.force_null_character_replacement_
            && matches!(
                self.state_,
                State::kDataState | State::kRCDATAState | State::kRAWTEXTState
            )
    }
}

impl HTMLTokenizer {
    // cpp: html/parser/html_tokenizer.h:41-50
    // cpp: html/parser/html_tokenizer.cc:151-157
    pub fn new(options: HTMLParserOptions) -> Pin<Box<Self>> {
        let mut tokenizer = Box::pin(Self {
            state_: State::kDataState,
            force_null_character_replacement_: false,
            should_allow_cdata_: false,
            truncated_markup_declaration_enabled_: options.truncated_markup_declaration,
            should_allow_dom_parts_: false,
            options_: options,
            additional_allowed_character_: 0,
            input_stream_preprocessor_: None,
            appropriate_end_tag_name_: UCharLiteralBuffer::default(),
            temporary_buffer_: LCharLiteralBuffer::default(),
            buffered_end_tag_name_: LCharLiteralBuffer::default(),
            attributes_ranges_: HTMLAttributesRanges::default(),
            token_: HTMLToken::default(),
            #[cfg(debug_assertions)]
            token_should_be_in_uninitialized_state_: true,
            _pin: PhantomPinned,
        });
        // SAFETY: Box::pin fixes the tokenizer's address for its lifetime.
        // The preprocessor only reads its owning tokenizer while that owner
        // remains pinned and the preprocessor is held inside it.
        unsafe {
            let owner = Pin::as_mut(&mut tokenizer).get_unchecked_mut();
            let owner_pointer = owner as *const Self;
            owner.input_stream_preprocessor_ = Some(InputStreamPreprocessor::new(owner_pointer));
            owner.ResetFields();
        }
        tokenizer
    }

    // cpp: html/parser/html_tokenizer.h:52
    // cpp: html/parser/html_tokenizer.cc:161-168
    pub fn Reset(mut self: Pin<&mut Self>) {
        // SAFETY: ResetFields changes fields without moving the pinned owner.
        unsafe { self.as_mut().get_unchecked_mut().ResetFields() }
    }

    fn ResetFields(&mut self) {
        self.token_.Clear();
        self.state_ = State::kDataState;
        self.force_null_character_replacement_ = false;
        self.should_allow_cdata_ = false;
        self.additional_allowed_character_ = 0;
    }

    // cpp: html/parser/html_tokenizer.h:54
    pub fn ClearToken(self: Pin<&mut Self>) {
        // SAFETY: clearing the token does not move the pinned owner.
        unsafe { self.get_unchecked_mut().token_.Clear() }
    }

    // cpp: html/parser/html_tokenizer.cc:169-185
    fn ProcessEntity(&mut self, source: &mut SegmentedString) -> bool {
        let mut not_enough_characters = false;
        let mut decoded_entity = DecodedHTMLEntity::default();
        let success = ConsumeHTMLEntity(source, &mut decoded_entity, &mut not_enough_characters, 0);
        if not_enough_characters {
            return false;
        }
        if !success {
            debug_assert!(decoded_entity.IsEmpty());
            self.BufferCharacter(b'&' as UChar);
        } else {
            self.token_.SetHasEntity();
            for i in 0..decoded_entity.length as usize {
                self.BufferCharacter(decoded_entity.data[i]);
            }
        }
        true
    }

    // cpp: html/parser/html_tokenizer.h:224-225
    // cpp: html/parser/html_tokenizer.cc:1760-1766
    fn SkipWhitespaces(&mut self, source: &mut SegmentedString, cc: &mut UChar) -> bool {
        if !CheckScanFlag(*cc, ScanFlags::kWhitespace) {
            return true;
        }
        self.SkipWhitespacesHelper(source, cc)
    }

    // cpp: html/parser/html_tokenizer.h:226-227
    // cpp: html/parser/html_tokenizer.cc:1768-1794
    fn SkipWhitespacesHelper(&mut self, source: &mut SegmentedString, cc: &mut UChar) -> bool {
        debug_assert!(!source.IsEmpty());
        debug_assert!(IsTokenizerWhitespace(*cc));
        *cc = source.CurrentChar();
        loop {
            while CheckScanFlag(*cc, ScanFlags::kWhitespaceNotNewline) {
                *cc = source.AdvancePastNonNewline();
            }
            match *cc {
                0x0A => *cc = source.AdvancePastNewlineAndUpdateLineNumber(),
                0x0D => {
                    if !self
                        .input_stream_preprocessor_
                        .as_mut()
                        .expect("tokenizer preprocessor must be initialized")
                        .AdvancePastCarriageReturn(source, cc)
                    {
                        return false;
                    }
                }
                0 => {
                    if !self
                        .input_stream_preprocessor_
                        .as_mut()
                        .expect("tokenizer preprocessor must be initialized")
                        .ProcessNullCharacter(source, cc)
                    {
                        return false;
                    }
                    if *cc == kEndOfFileMarker as UChar {
                        return true;
                    }
                }
                _ => return true,
            }
        }
    }

    // cpp: html/parser/html_tokenizer.h:262
    // cpp: html/parser/html_tokenizer.cc:1796-1841
    fn EmitData(&mut self, source: &mut SegmentedString, mut cc: UChar) -> bool {
        self.token_.EnsureIsCharacterToken();
        if cc == b'\n' as UChar {
            cc = source.CurrentChar();
        }
        loop {
            while !CheckScanFlag(cc, ScanFlags::kCharacterTokenSpecial) {
                self.token_.AppendToCharacter(cc);
                cc = source.AdvancePastNonNewline();
            }
            match cc {
                0x26 => {
                    self.state_ = State::kCharacterReferenceInDataState;
                    source.AdvanceExpecting(b'&' as UChar);
                    if !self.ProcessEntity(source) {
                        return true;
                    }
                    self.state_ = State::kDataState;
                    if source.IsEmpty() {
                        return true;
                    }
                    cc = source.CurrentChar();
                }
                0x0A => {
                    self.token_.AppendToCharacter(cc);
                    cc = source.AdvancePastNewlineAndUpdateLineNumber();
                }
                0x0D => {
                    self.token_.AppendToCharacter(b'\n' as UChar);
                    if !self
                        .input_stream_preprocessor_
                        .as_mut()
                        .expect("tokenizer preprocessor must be initialized")
                        .AdvancePastCarriageReturn(source, &mut cc)
                    {
                        return true;
                    }
                }
                0x3C => return true,
                0 => {
                    if !self
                        .input_stream_preprocessor_
                        .as_mut()
                        .expect("tokenizer preprocessor must be initialized")
                        .ProcessNullCharacter(source, &mut cc)
                    {
                        return true;
                    }
                    if cc == kEndOfFileMarker as UChar {
                        return self.EmitEndOfFile(source);
                    }
                }
                0x7B => {
                    self.token_.AppendToCharacter(cc);
                    cc = source.AdvancePastNonNewline();
                }
                _ => unreachable!("character scan flag and data branch disagree"),
            }
        }
    }

    // cpp: html/parser/html_tokenizer.h:264
    // cpp: html/parser/html_tokenizer.cc:1843-1872
    fn EmitPLAINTEXT(&mut self, source: &mut SegmentedString, mut cc: UChar) -> bool {
        self.token_.EnsureIsCharacterToken();
        if cc == b'\n' as UChar {
            cc = source.CurrentChar();
        }
        loop {
            while !CheckScanFlag(cc, ScanFlags::kNullOrNewline) {
                self.token_.AppendToCharacter(cc);
                cc = source.AdvancePastNonNewline();
            }
            match cc {
                0x0A => {
                    self.token_.AppendToCharacter(cc);
                    cc = source.AdvancePastNewlineAndUpdateLineNumber();
                }
                0x0D => {
                    self.token_.AppendToCharacter(b'\n' as UChar);
                    if !self
                        .input_stream_preprocessor_
                        .as_mut()
                        .expect("tokenizer preprocessor must be initialized")
                        .AdvancePastCarriageReturn(source, &mut cc)
                    {
                        return true;
                    }
                }
                0 => {
                    if !self
                        .input_stream_preprocessor_
                        .as_mut()
                        .expect("tokenizer preprocessor must be initialized")
                        .ProcessNullCharacter(source, &mut cc)
                    {
                        return true;
                    }
                    if cc == kEndOfFileMarker as UChar {
                        return self.EmitEndOfFile(source);
                    }
                }
                _ => unreachable!("character scan flag and plaintext branch disagree"),
            }
        }
    }

    // cpp: html/parser/html_tokenizer.cc:187-202
    fn FlushBufferedEndTag(
        &mut self,
        source: &mut SegmentedString,
        current_char_may_be_newline: bool,
    ) -> bool {
        debug_assert!(matches!(
            self.token_.GetType(),
            TokenType::kCharacter | TokenType::kUninitialized
        ));
        if current_char_may_be_newline {
            source.AdvanceAndUpdateLineNumber();
        } else {
            source.AdvancePastNonNewline();
        }
        if self.token_.GetType() == TokenType::kCharacter {
            return true;
        }
        self.token_.BeginEndTagBuffer(&self.buffered_end_tag_name_);
        self.buffered_end_tag_name_.clear();
        self.appropriate_end_tag_name_.clear();
        self.temporary_buffer_.clear();
        false
    }

    // cpp: html/parser/html_tokenizer.cc:234-238
    fn FlushEmitAndResumeInDataState(&mut self, source: &mut SegmentedString) -> bool {
        self.state_ = State::kDataState;
        self.FlushBufferedEndTag(source, false);
        true
    }

    // cpp: html/parser/html_tokenizer.h:148-153
    pub fn NumberOfBufferedCharacters(&self) -> u32 {
        if self.temporary_buffer_.size() != 0 {
            self.temporary_buffer_.size() + 2
        } else {
            0
        }
    }

    // cpp: html/parser/html_tokenizer.cc:1874-1882
    pub fn BufferedCharacters(&self) -> foundation::BlinkString {
        let mut characters = foundation::StringBuilder::new();
        foundation::StringBuilderReserveCapacity(
            &mut characters,
            self.NumberOfBufferedCharacters(),
        );
        characters.Append('<');
        characters.Append('/');
        characters.AppendString(&self.temporary_buffer_.AsString());
        characters.ToString()
    }

    // cpp: html/parser/html_tokenizer.h:158
    // cpp: html/parser/html_tokenizer.cc:1884-1888
    pub fn UpdateStateForToken(self: Pin<&mut Self>, token: &HTMLToken) {
        if !token.GetName().IsEmpty() {
            self.UpdateStateForTag(LookupHtmlTag(token.GetName().as_slice()));
        }
    }

    // cpp: html/parser/html_tokenizer.h:159
    // cpp: html/parser/html_tokenizer.cc:1890-1894
    pub fn UpdateStateForTag(self: Pin<&mut Self>, tag: HTMLTag) {
        if let Some(state) = self.as_ref().get_ref().SpeculativeStateForTag(tag) {
            self.SetState(state);
        }
    }

    // cpp: html/parser/html_tokenizer.h:176
    // cpp: html/parser/html_tokenizer.cc:1896-1919
    pub fn SpeculativeStateForTag(&self, tag: HTMLTag) -> Option<State> {
        match tag {
            HTMLTag::kTextarea | HTMLTag::kTitle => Some(State::kRCDATAState),
            HTMLTag::kPlaintext => Some(State::kPLAINTEXTState),
            HTMLTag::kScript => Some(State::kScriptDataState),
            HTMLTag::kStyle
            | HTMLTag::kIFrame
            | HTMLTag::kXmp
            | HTMLTag::kNoembed
            | HTMLTag::kNoframes => Some(State::kRAWTEXTState),
            HTMLTag::kNoscript => {
                if self.options_.scripting_flag {
                    Some(State::kRAWTEXTState)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    // cpp: html/parser/html_tokenizer.h:178-180
    pub fn ForceNullCharacterReplacement(&self) -> bool {
        self.force_null_character_replacement_
    }

    // cpp: html/parser/html_tokenizer.h:181-183
    pub fn SetForceNullCharacterReplacement(self: Pin<&mut Self>, value: bool) {
        // SAFETY: assigning this flag does not move the pinned owner.
        unsafe { self.get_unchecked_mut().force_null_character_replacement_ = value }
    }

    // cpp: html/parser/html_tokenizer.h:185
    pub fn ShouldAllowCDATA(&self) -> bool {
        self.should_allow_cdata_
    }

    // cpp: html/parser/html_tokenizer.h:186
    pub fn SetShouldAllowCDATA(self: Pin<&mut Self>, value: bool) {
        // SAFETY: assigning this flag does not move the pinned owner.
        unsafe { self.get_unchecked_mut().should_allow_cdata_ = value }
    }

    // cpp: html/parser/html_tokenizer.h:188
    pub fn GetState(&self) -> State {
        self.state_
    }

    // cpp: html/parser/html_tokenizer.h:189
    pub fn SetState(self: Pin<&mut Self>, state: State) {
        // SAFETY: assigning the state does not move the pinned owner.
        unsafe { self.get_unchecked_mut().state_ = state }
    }

    // cpp: html/parser/html_tokenizer.h:198-212
    pub fn IsEndTagBufferingState(state: State) -> bool {
        matches!(
            state,
            State::kRCDATAEndTagOpenState
                | State::kRCDATAEndTagNameState
                | State::kRAWTEXTEndTagOpenState
                | State::kRAWTEXTEndTagNameState
                | State::kScriptDataEndTagOpenState
                | State::kScriptDataEndTagNameState
                | State::kScriptDataEscapedEndTagOpenState
                | State::kScriptDataEscapedEndTagNameState
        )
    }

    // cpp: html/parser/html_tokenizer.h:218-223
    pub fn attributes_ranges(self: Pin<&mut Self>) -> &mut HTMLAttributesRanges {
        debug_assert!(self.as_ref().get_ref().options_.track_attributes_ranges);
        // SAFETY: borrowing this field does not move the pinned owner.
        unsafe { &mut self.get_unchecked_mut().attributes_ranges_ }
    }

    // cpp: html/parser/html_tokenizer.h:236-240
    fn BufferCharacter(&mut self, character: UChar) {
        debug_assert_ne!(character, kEndOfFileMarker as UChar);
        self.token_.EnsureIsCharacterToken();
        self.token_.AppendToCharacter(character);
    }

    // cpp: html/parser/html_tokenizer.h:242-247
    fn EmitAndResumeInDataState(&mut self, source: &mut SegmentedString) -> bool {
        self.SaveEndTagNameIfNeeded();
        self.state_ = State::kDataState;
        source.AdvancePastNonNewline();
        true
    }

    // cpp: html/parser/html_tokenizer.h:249-254
    fn EmitProcessingInstruction(&mut self, source: &mut SegmentedString) -> bool {
        self.temporary_buffer_.clear();
        self.state_ = State::kDataState;
        source.AdvancePastNonNewline();
        true
    }

    // cpp: html/parser/html_tokenizer.h:256-260
    fn EmitAndReconsumeInDataState(&mut self) -> bool {
        self.SaveEndTagNameIfNeeded();
        self.state_ = State::kDataState;
        true
    }

    // cpp: html/parser/html_tokenizer.h:266-274
    fn EmitEndOfFile(&mut self, source: &mut SegmentedString) -> bool {
        if self.HaveBufferedCharacterToken() {
            return true;
        }
        self.state_ = State::kDataState;
        source.AdvanceAndUpdateLineNumber();
        self.token_.Clear();
        self.token_.MakeEndOfFile();
        true
    }

    // cpp: html/parser/html_tokenizer.h:289-293
    fn SaveEndTagNameIfNeeded(&mut self) {
        debug_assert_ne!(self.token_.GetType(), TokenType::kUninitialized);
        if self.token_.GetType() == TokenType::kStartTag {
            self.appropriate_end_tag_name_.Copy(self.token_.GetName());
        }
    }

    // cpp: html/parser/html_tokenizer.cc:1921-1923
    fn TemporaryBufferIs(&self, expected_string: &foundation::BlinkString) -> bool {
        VectorEqualsString(&self.temporary_buffer_, expected_string)
    }

    // cpp: html/parser/html_tokenizer.cc:1925-1928
    fn AddToPossibleEndTag(&mut self, cc: u8) {
        debug_assert!(Self::IsEndTagBufferingState(self.state_));
        self.buffered_end_tag_name_.AddChar(cc);
    }

    // cpp: html/parser/html_tokenizer.h:294
    // cpp: html/parser/html_tokenizer.cc:1930-1933
    fn IsAppropriateEndTag(&self) -> bool {
        self.buffered_end_tag_name_
            .as_slice()
            .iter()
            .map(|&byte| UChar::from(byte))
            .eq(self.appropriate_end_tag_name_.as_slice().iter().copied())
    }

    // cpp: html/parser/html_tokenizer.cc:1935-1938
    fn ParseError(&self) {
        // The C++ standalone boundary has no diagnostics sink.
    }

    // cpp: html/parser/html_tokenizer.h:296-298
    fn HaveBufferedCharacterToken(&self) -> bool {
        self.token_.GetType() == TokenType::kCharacter
    }

    // cpp: html/parser/html_tokenizer.h:300-305
    fn ShouldWaitForMoreInput(&self, source: &SegmentedString) -> bool {
        !self.truncated_markup_declaration_enabled_ || !source.IsClosed()
    }

    // cpp: html/parser/html_tokenizer.h:138-141
    // cpp: html/parser/html_tokenizer.cc:240-254
    pub fn NextToken(self: Pin<&mut Self>, source: &mut SegmentedString) -> Option<&HTMLToken> {
        // SAFETY: NextTokenImpl mutates fields but never moves the pinned owner
        // whose address is held by input_stream_preprocessor_.
        let owner = unsafe { self.get_unchecked_mut() };
        #[cfg(debug_assertions)]
        {
            debug_assert!(
                !owner.token_should_be_in_uninitialized_state_ || owner.token_.IsUninitialized()
            );
            debug_assert!(
                !owner.token_should_be_in_uninitialized_state_
                    || owner.attributes_ranges_.attributes().is_empty()
            );
        }
        let completed_token = owner.NextTokenImpl(source);
        #[cfg(debug_assertions)]
        {
            owner.token_should_be_in_uninitialized_state_ = completed_token;
        }
        if completed_token {
            Some(&owner.token_)
        } else {
            None
        }
    }

    // cpp: html/parser/html_tokenizer.h:225-226
    // cpp: html/parser/html_tokenizer.cc:256-275
    // The C++ goto labels map to an exhaustive State match and labeled loop.
    fn NextTokenImpl(&mut self, source: &mut SegmentedString) -> bool {
        if !self.buffered_end_tag_name_.IsEmpty() && !Self::IsEndTagBufferingState(self.state_) {
            self.token_.BeginEndTagBuffer(&self.buffered_end_tag_name_);
            self.buffered_end_tag_name_.clear();
            self.appropriate_end_tag_name_.clear();
            self.temporary_buffer_.clear();
            if self.state_ == State::kDataState {
                return true;
            }
        }

        let mut cc: UChar = 0;
        if source.IsEmpty()
            || !self
                .input_stream_preprocessor_
                .as_mut()
                .expect("tokenizer preprocessor must be initialized")
                .Peek(source, &mut cc)
        {
            return self.HaveBufferedCharacterToken();
        }

        'states: loop {
            match self.state_ {
                // cpp: html/parser/html_tokenizer.cc:276-292
                State::kDataState => {
                    if cc == b'&' as UChar {
                        advance_past_non_newline_to!(
                            self,
                            source,
                            cc,
                            State::kCharacterReferenceInDataState,
                            'states
                        );
                    } else if cc == b'<' as UChar {
                        if self.HaveBufferedCharacterToken() {
                            return true;
                        }
                        advance_past_non_newline_to!(self, source, cc, State::kTagOpenState, 'states);
                    } else if cc == kEndOfFileMarker as UChar {
                        return self.EmitEndOfFile(source);
                    } else {
                        return self.EmitData(source, cc);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:294-299
                State::kCharacterReferenceInDataState => {
                    if !self.ProcessEntity(source) {
                        return self.HaveBufferedCharacterToken();
                    }
                    switch_to!(self, source, cc, State::kDataState, 'states);
                }

                // cpp: html/parser/html_tokenizer.cc:301-317
                State::kRCDATAState => {
                    while !CheckScanFlag(cc, ScanFlags::kRCDATASpecial) {
                        self.BufferCharacter(cc);
                        if !self
                            .input_stream_preprocessor_
                            .as_mut()
                            .expect("tokenizer preprocessor must be initialized")
                            .Advance(source, &mut cc)
                        {
                            return self.HaveBufferedCharacterToken();
                        }
                    }
                    if cc == b'&' as UChar {
                        advance_past_non_newline_to!(
                            self,
                            source,
                            cc,
                            State::kCharacterReferenceInRCDATAState,
                            'states
                        );
                    } else if cc == b'<' as UChar {
                        advance_past_non_newline_to!(
                            self,
                            source,
                            cc,
                            State::kRCDATALessThanSignState,
                            'states
                        );
                    } else if cc == kEndOfFileMarker as UChar {
                        return self.EmitEndOfFile(source);
                    } else {
                        unreachable!("RCDATA special character has no branch");
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:319-324
                State::kCharacterReferenceInRCDATAState => {
                    if !self.ProcessEntity(source) {
                        return self.HaveBufferedCharacterToken();
                    }
                    switch_to!(self, source, cc, State::kRCDATAState, 'states);
                }

                // cpp: html/parser/html_tokenizer.cc:326-336
                State::kRAWTEXTState => {
                    if cc == b'<' as UChar {
                        advance_past_non_newline_to!(
                            self,
                            source,
                            cc,
                            State::kRAWTEXTLessThanSignState,
                            'states
                        );
                    } else if cc == kEndOfFileMarker as UChar {
                        return self.EmitEndOfFile(source);
                    } else {
                        self.BufferCharacter(cc);
                        consume!(self, source, cc, State::kRAWTEXTState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:338-348
                State::kScriptDataState => {
                    if cc == b'<' as UChar {
                        advance_past_non_newline_to!(
                            self,
                            source,
                            cc,
                            State::kScriptDataLessThanSignState,
                            'states
                        );
                    } else if cc == kEndOfFileMarker as UChar {
                        return self.EmitEndOfFile(source);
                    } else {
                        self.BufferCharacter(cc);
                        consume!(self, source, cc, State::kScriptDataState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:350-355
                State::kPLAINTEXTState => {
                    if cc == kEndOfFileMarker as UChar {
                        return self.EmitEndOfFile(source);
                    }
                    return self.EmitPLAINTEXT(source, cc);
                }

                // cpp: html/parser/html_tokenizer.cc:357-381
                State::kTagOpenState => {
                    if cc <= 0x7F && (cc as u8).is_ascii_alphabetic() {
                        self.token_.BeginStartTag(ToLowerCase(cc) as u8);
                        advance_past_non_newline_to!(self, source, cc, State::kTagNameState, 'states);
                    } else if cc == b'!' as UChar {
                        advance_past_non_newline_to!(
                            self,
                            source,
                            cc,
                            State::kMarkupDeclarationOpenState,
                            'states
                        );
                    } else if cc == b'/' as UChar {
                        advance_past_non_newline_to!(self, source, cc, State::kEndTagOpenState, 'states);
                    } else if cc == b'?' as UChar {
                        if self.options_.processing_instructions {
                            advance_past_non_newline_to!(
                                self,
                                source,
                                cc,
                                State::kProcessingInstructionOpenState,
                                'states
                            );
                        } else {
                            self.ParseError();
                            reconsume_in!(self, State::kBogusCommentState, 'states);
                        }
                    } else {
                        self.ParseError();
                        self.BufferCharacter(b'<' as UChar);
                        reconsume_in!(self, State::kDataState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:383-401
                State::kEndTagOpenState => {
                    if cc <= 0x7F && (cc as u8).is_ascii_alphabetic() {
                        self.token_.BeginEndTag(ToLowerCase(cc) as u8);
                        self.appropriate_end_tag_name_.clear();
                        advance_past_non_newline_to!(self, source, cc, State::kTagNameState, 'states);
                    } else if cc == b'>' as UChar {
                        self.ParseError();
                        advance_past_non_newline_to!(self, source, cc, State::kDataState, 'states);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        self.BufferCharacter(b'<' as UChar);
                        self.BufferCharacter(b'/' as UChar);
                        reconsume_in!(self, State::kDataState, 'states);
                    } else {
                        self.ParseError();
                        reconsume_in!(self, State::kBogusCommentState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:403-421
                State::kTagNameState => {
                    while !CheckScanFlag(cc, ScanFlags::kTagNameSpecial) {
                        self.token_.AppendToName(ToLowerCaseIfAlpha(cc));
                        if !self
                            .input_stream_preprocessor_
                            .as_mut()
                            .expect("tokenizer preprocessor must be initialized")
                            .AdvancePastNonNewline(source, &mut cc)
                        {
                            return self.HaveBufferedCharacterToken();
                        }
                    }
                    if cc == b'/' as UChar {
                        advance_past_non_newline_to!(
                            self,
                            source,
                            cc,
                            State::kSelfClosingStartTagState,
                            'states
                        );
                    } else if cc == b'>' as UChar {
                        return self.EmitAndResumeInDataState(source);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        reconsume_in!(self, State::kDataState, 'states);
                    } else {
                        debug_assert!(IsTokenizerWhitespace(cc));
                        advance_to!(self, source, cc, State::kBeforeAttributeNameState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:423-433
                State::kRCDATALessThanSignState => {
                    if cc == b'/' as UChar {
                        self.temporary_buffer_.clear();
                        debug_assert!(self.buffered_end_tag_name_.IsEmpty());
                        advance_past_non_newline_to!(
                            self,
                            source,
                            cc,
                            State::kRCDATAEndTagOpenState,
                            'states
                        );
                    } else {
                        self.BufferCharacter(b'<' as UChar);
                        reconsume_in!(self, State::kRCDATAState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:435-446
                State::kRCDATAEndTagOpenState => {
                    if cc <= 0x7F && (cc as u8).is_ascii_alphabetic() {
                        self.temporary_buffer_.AddChar(cc as u8);
                        self.AddToPossibleEndTag(ToLowerCase(cc) as u8);
                        advance_past_non_newline_to!(
                            self,
                            source,
                            cc,
                            State::kRCDATAEndTagNameState,
                            'states
                        );
                    } else {
                        self.BufferCharacter(b'<' as UChar);
                        self.BufferCharacter(b'/' as UChar);
                        reconsume_in!(self, State::kRCDATAState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:448-478
                State::kRCDATAEndTagNameState => {
                    if cc <= 0x7F && (cc as u8).is_ascii_alphabetic() {
                        self.temporary_buffer_.AddChar(cc as u8);
                        self.AddToPossibleEndTag(ToLowerCase(cc) as u8);
                        consume_non_newline!(self, source, cc, State::kRCDATAEndTagNameState, 'states);
                    } else {
                        if IsTokenizerWhitespace(cc) {
                            if self.IsAppropriateEndTag() {
                                self.temporary_buffer_.AddChar(cc as u8);
                                flush_and_advance_to!(
                                    self,
                                    source,
                                    cc,
                                    State::kBeforeAttributeNameState,
                                    true,
                                    'states
                                );
                            }
                        } else if cc == b'/' as UChar {
                            if self.IsAppropriateEndTag() {
                                self.temporary_buffer_.AddChar(cc as u8);
                                flush_and_advance_to!(
                                    self,
                                    source,
                                    cc,
                                    State::kSelfClosingStartTagState,
                                    false,
                                    'states
                                );
                            }
                        } else if cc == b'>' as UChar && self.IsAppropriateEndTag() {
                            self.temporary_buffer_.AddChar(cc as u8);
                            return self.FlushEmitAndResumeInDataState(source);
                        }
                        self.BufferCharacter(b'<' as UChar);
                        self.BufferCharacter(b'/' as UChar);
                        self.token_.AppendToCharacterBuffer(&self.temporary_buffer_);
                        self.buffered_end_tag_name_.clear();
                        self.temporary_buffer_.clear();
                        reconsume_in!(self, State::kRCDATAState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:480-490
                State::kRAWTEXTLessThanSignState => {
                    if cc == b'/' as UChar {
                        self.temporary_buffer_.clear();
                        debug_assert!(self.buffered_end_tag_name_.IsEmpty());
                        advance_past_non_newline_to!(
                            self,
                            source,
                            cc,
                            State::kRAWTEXTEndTagOpenState,
                            'states
                        );
                    } else {
                        self.BufferCharacter(b'<' as UChar);
                        reconsume_in!(self, State::kRAWTEXTState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:492-503
                State::kRAWTEXTEndTagOpenState => {
                    if cc <= 0x7F && (cc as u8).is_ascii_alphabetic() {
                        self.temporary_buffer_.AddChar(cc as u8);
                        self.AddToPossibleEndTag(ToLowerCase(cc) as u8);
                        advance_past_non_newline_to!(
                            self,
                            source,
                            cc,
                            State::kRAWTEXTEndTagNameState,
                            'states
                        );
                    } else {
                        self.BufferCharacter(b'<' as UChar);
                        self.BufferCharacter(b'/' as UChar);
                        reconsume_in!(self, State::kRAWTEXTState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:505-535
                State::kRAWTEXTEndTagNameState => {
                    if cc <= 0x7F && (cc as u8).is_ascii_alphabetic() {
                        self.temporary_buffer_.AddChar(cc as u8);
                        self.AddToPossibleEndTag(ToLowerCase(cc) as u8);
                        consume_non_newline!(self, source, cc, State::kRAWTEXTEndTagNameState, 'states);
                    } else {
                        if IsTokenizerWhitespace(cc) {
                            if self.IsAppropriateEndTag() {
                                self.temporary_buffer_.AddChar(cc as u8);
                                flush_and_advance_to!(
                                    self,
                                    source,
                                    cc,
                                    State::kBeforeAttributeNameState,
                                    true,
                                    'states
                                );
                            }
                        } else if cc == b'/' as UChar {
                            if self.IsAppropriateEndTag() {
                                self.temporary_buffer_.AddChar(cc as u8);
                                flush_and_advance_to!(
                                    self,
                                    source,
                                    cc,
                                    State::kSelfClosingStartTagState,
                                    false,
                                    'states
                                );
                            }
                        } else if cc == b'>' as UChar && self.IsAppropriateEndTag() {
                            self.temporary_buffer_.AddChar(cc as u8);
                            return self.FlushEmitAndResumeInDataState(source);
                        }
                        self.BufferCharacter(b'<' as UChar);
                        self.BufferCharacter(b'/' as UChar);
                        self.token_.AppendToCharacterBuffer(&self.temporary_buffer_);
                        self.buffered_end_tag_name_.clear();
                        self.temporary_buffer_.clear();
                        reconsume_in!(self, State::kRAWTEXTState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:537-552
                State::kScriptDataLessThanSignState => {
                    if cc == b'/' as UChar {
                        self.temporary_buffer_.clear();
                        debug_assert!(self.buffered_end_tag_name_.IsEmpty());
                        advance_past_non_newline_to!(
                            self, source, cc, State::kScriptDataEndTagOpenState, 'states
                        );
                    } else if cc == b'!' as UChar {
                        self.BufferCharacter(b'<' as UChar);
                        self.BufferCharacter(b'!' as UChar);
                        advance_past_non_newline_to!(
                            self, source, cc, State::kScriptDataEscapeStartState, 'states
                        );
                    } else {
                        self.BufferCharacter(b'<' as UChar);
                        reconsume_in!(self, State::kScriptDataState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:553-565
                State::kScriptDataEndTagOpenState => {
                    if cc <= 0x7F && (cc as u8).is_ascii_alphabetic() {
                        self.temporary_buffer_.AddChar(cc as u8);
                        self.AddToPossibleEndTag(ToLowerCase(cc) as u8);
                        advance_past_non_newline_to!(
                            self, source, cc, State::kScriptDataEndTagNameState, 'states
                        );
                    } else {
                        self.BufferCharacter(b'<' as UChar);
                        self.BufferCharacter(b'/' as UChar);
                        reconsume_in!(self, State::kScriptDataState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:566-596
                State::kScriptDataEndTagNameState => {
                    if cc <= 0x7F && (cc as u8).is_ascii_alphabetic() {
                        self.temporary_buffer_.AddChar(cc as u8);
                        self.AddToPossibleEndTag(ToLowerCase(cc) as u8);
                        consume_non_newline!(self, source, cc, State::kScriptDataEndTagNameState, 'states);
                    } else {
                        if IsTokenizerWhitespace(cc) {
                            if self.IsAppropriateEndTag() {
                                self.temporary_buffer_.AddChar(cc as u8);
                                flush_and_advance_to!(
                                    self, source, cc, State::kBeforeAttributeNameState, true, 'states
                                );
                            }
                        } else if cc == b'/' as UChar {
                            if self.IsAppropriateEndTag() {
                                self.temporary_buffer_.AddChar(cc as u8);
                                flush_and_advance_to!(
                                    self, source, cc, State::kSelfClosingStartTagState, false, 'states
                                );
                            }
                        } else if cc == b'>' as UChar && self.IsAppropriateEndTag() {
                            self.temporary_buffer_.AddChar(cc as u8);
                            return self.FlushEmitAndResumeInDataState(source);
                        }
                        self.BufferCharacter(b'<' as UChar);
                        self.BufferCharacter(b'/' as UChar);
                        self.token_.AppendToCharacterBuffer(&self.temporary_buffer_);
                        self.buffered_end_tag_name_.clear();
                        self.temporary_buffer_.clear();
                        reconsume_in!(self, State::kScriptDataState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:598-605
                State::kScriptDataEscapeStartState => {
                    if cc == b'-' as UChar {
                        self.BufferCharacter(cc);
                        advance_past_non_newline_to!(
                            self, source, cc, State::kScriptDataEscapeStartDashState, 'states
                        );
                    } else {
                        reconsume_in!(self, State::kScriptDataState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:607-615
                State::kScriptDataEscapeStartDashState => {
                    if cc == b'-' as UChar {
                        self.BufferCharacter(cc);
                        advance_past_non_newline_to!(
                            self, source, cc, State::kScriptDataEscapedDashDashState, 'states
                        );
                    } else {
                        reconsume_in!(self, State::kScriptDataState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:616-631
                State::kScriptDataEscapedState => {
                    if cc == b'-' as UChar {
                        self.BufferCharacter(cc);
                        advance_past_non_newline_to!(
                            self, source, cc, State::kScriptDataEscapedDashState, 'states
                        );
                    } else if cc == b'<' as UChar {
                        advance_past_non_newline_to!(
                            self, source, cc, State::kScriptDataEscapedLessThanSignState, 'states
                        );
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        reconsume_in!(self, State::kDataState, 'states);
                    } else {
                        self.BufferCharacter(cc);
                        consume!(self, source, cc, State::kScriptDataEscapedState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:632-647
                State::kScriptDataEscapedDashState => {
                    if cc == b'-' as UChar {
                        self.BufferCharacter(cc);
                        advance_past_non_newline_to!(
                            self, source, cc, State::kScriptDataEscapedDashDashState, 'states
                        );
                    } else if cc == b'<' as UChar {
                        advance_past_non_newline_to!(
                            self, source, cc, State::kScriptDataEscapedLessThanSignState, 'states
                        );
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        reconsume_in!(self, State::kDataState, 'states);
                    } else {
                        self.BufferCharacter(cc);
                        advance_to!(self, source, cc, State::kScriptDataEscapedState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:648-666
                State::kScriptDataEscapedDashDashState => {
                    if cc == b'-' as UChar {
                        self.BufferCharacter(cc);
                        consume_non_newline!(
                            self, source, cc, State::kScriptDataEscapedDashDashState, 'states
                        );
                    } else if cc == b'<' as UChar {
                        advance_past_non_newline_to!(
                            self, source, cc, State::kScriptDataEscapedLessThanSignState, 'states
                        );
                    } else if cc == b'>' as UChar {
                        self.BufferCharacter(cc);
                        advance_past_non_newline_to!(self, source, cc, State::kScriptDataState, 'states);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        reconsume_in!(self, State::kDataState, 'states);
                    } else {
                        self.BufferCharacter(cc);
                        advance_to!(self, source, cc, State::kScriptDataEscapedState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:667-684
                State::kScriptDataEscapedLessThanSignState => {
                    if cc == b'/' as UChar {
                        self.temporary_buffer_.clear();
                        debug_assert!(self.buffered_end_tag_name_.IsEmpty());
                        advance_past_non_newline_to!(
                            self, source, cc, State::kScriptDataEscapedEndTagOpenState, 'states
                        );
                    } else if cc <= 0x7F && (cc as u8).is_ascii_alphabetic() {
                        self.BufferCharacter(b'<' as UChar);
                        self.BufferCharacter(cc);
                        self.temporary_buffer_.clear();
                        self.temporary_buffer_.AddChar(ToLowerCase(cc) as u8);
                        advance_past_non_newline_to!(
                            self, source, cc, State::kScriptDataDoubleEscapeStartState, 'states
                        );
                    } else {
                        self.BufferCharacter(b'<' as UChar);
                        reconsume_in!(self, State::kScriptDataEscapedState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:685-697
                State::kScriptDataEscapedEndTagOpenState => {
                    if cc <= 0x7F && (cc as u8).is_ascii_alphabetic() {
                        self.temporary_buffer_.AddChar(cc as u8);
                        self.AddToPossibleEndTag(ToLowerCase(cc) as u8);
                        advance_past_non_newline_to!(
                            self, source, cc, State::kScriptDataEscapedEndTagNameState, 'states
                        );
                    } else {
                        self.BufferCharacter(b'<' as UChar);
                        self.BufferCharacter(b'/' as UChar);
                        reconsume_in!(self, State::kScriptDataEscapedState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:698-729
                State::kScriptDataEscapedEndTagNameState => {
                    if cc <= 0x7F && (cc as u8).is_ascii_alphabetic() {
                        self.temporary_buffer_.AddChar(cc as u8);
                        self.AddToPossibleEndTag(ToLowerCase(cc) as u8);
                        consume_non_newline!(
                            self, source, cc, State::kScriptDataEscapedEndTagNameState, 'states
                        );
                    } else {
                        if IsTokenizerWhitespace(cc) {
                            if self.IsAppropriateEndTag() {
                                self.temporary_buffer_.AddChar(cc as u8);
                                flush_and_advance_to!(
                                    self, source, cc, State::kBeforeAttributeNameState, true, 'states
                                );
                            }
                        } else if cc == b'/' as UChar {
                            if self.IsAppropriateEndTag() {
                                self.temporary_buffer_.AddChar(cc as u8);
                                flush_and_advance_to!(
                                    self, source, cc, State::kSelfClosingStartTagState, false, 'states
                                );
                            }
                        } else if cc == b'>' as UChar && self.IsAppropriateEndTag() {
                            self.temporary_buffer_.AddChar(cc as u8);
                            return self.FlushEmitAndResumeInDataState(source);
                        }
                        self.BufferCharacter(b'<' as UChar);
                        self.BufferCharacter(b'/' as UChar);
                        self.token_.AppendToCharacterBuffer(&self.temporary_buffer_);
                        self.buffered_end_tag_name_.clear();
                        self.temporary_buffer_.clear();
                        reconsume_in!(self, State::kScriptDataEscapedState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:730-746
                State::kScriptDataDoubleEscapeStartState => {
                    if IsTokenizerWhitespace(cc) || cc == b'/' as UChar || cc == b'>' as UChar {
                        self.BufferCharacter(cc);
                        if self.TemporaryBufferIs(&foundation::BlinkString::from("script")) {
                            advance_to!(self, source, cc, State::kScriptDataDoubleEscapedState, 'states);
                        } else {
                            advance_to!(self, source, cc, State::kScriptDataEscapedState, 'states);
                        }
                    } else if cc <= 0x7F && (cc as u8).is_ascii_alphabetic() {
                        self.BufferCharacter(cc);
                        self.temporary_buffer_.AddChar(ToLowerCase(cc) as u8);
                        consume_non_newline!(
                            self, source, cc, State::kScriptDataDoubleEscapeStartState, 'states
                        );
                    } else {
                        reconsume_in!(self, State::kScriptDataEscapedState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:747-764
                State::kScriptDataDoubleEscapedState => {
                    if cc == b'-' as UChar {
                        self.BufferCharacter(cc);
                        advance_past_non_newline_to!(
                            self, source, cc, State::kScriptDataDoubleEscapedDashState, 'states
                        );
                    } else if cc == b'<' as UChar {
                        self.BufferCharacter(cc);
                        advance_past_non_newline_to!(
                            self, source, cc, State::kScriptDataDoubleEscapedLessThanSignState, 'states
                        );
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        reconsume_in!(self, State::kDataState, 'states);
                    } else {
                        self.BufferCharacter(cc);
                        consume!(self, source, cc, State::kScriptDataDoubleEscapedState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:765-782
                State::kScriptDataDoubleEscapedDashState => {
                    if cc == b'-' as UChar {
                        self.BufferCharacter(cc);
                        advance_past_non_newline_to!(
                            self, source, cc, State::kScriptDataDoubleEscapedDashDashState, 'states
                        );
                    } else if cc == b'<' as UChar {
                        self.BufferCharacter(cc);
                        advance_past_non_newline_to!(
                            self, source, cc, State::kScriptDataDoubleEscapedLessThanSignState, 'states
                        );
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        reconsume_in!(self, State::kDataState, 'states);
                    } else {
                        self.BufferCharacter(cc);
                        advance_to!(self, source, cc, State::kScriptDataDoubleEscapedState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:783-803
                State::kScriptDataDoubleEscapedDashDashState => {
                    if cc == b'-' as UChar {
                        self.BufferCharacter(cc);
                        consume_non_newline!(
                            self, source, cc, State::kScriptDataDoubleEscapedDashDashState, 'states
                        );
                    } else if cc == b'<' as UChar {
                        self.BufferCharacter(cc);
                        advance_past_non_newline_to!(
                            self, source, cc, State::kScriptDataDoubleEscapedLessThanSignState, 'states
                        );
                    } else if cc == b'>' as UChar {
                        self.BufferCharacter(cc);
                        advance_past_non_newline_to!(self, source, cc, State::kScriptDataState, 'states);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        reconsume_in!(self, State::kDataState, 'states);
                    } else {
                        self.BufferCharacter(cc);
                        advance_to!(self, source, cc, State::kScriptDataDoubleEscapedState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:804-813
                State::kScriptDataDoubleEscapedLessThanSignState => {
                    if cc == b'/' as UChar {
                        self.BufferCharacter(cc);
                        self.temporary_buffer_.clear();
                        advance_past_non_newline_to!(
                            self, source, cc, State::kScriptDataDoubleEscapeEndState, 'states
                        );
                    } else {
                        reconsume_in!(self, State::kScriptDataDoubleEscapedState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:814-830
                State::kScriptDataDoubleEscapeEndState => {
                    if IsTokenizerWhitespace(cc) || cc == b'/' as UChar || cc == b'>' as UChar {
                        self.BufferCharacter(cc);
                        if self.TemporaryBufferIs(&foundation::BlinkString::from("script")) {
                            advance_to!(self, source, cc, State::kScriptDataEscapedState, 'states);
                        } else {
                            advance_to!(self, source, cc, State::kScriptDataDoubleEscapedState, 'states);
                        }
                    } else if cc <= 0x7F && (cc as u8).is_ascii_alphabetic() {
                        self.BufferCharacter(cc);
                        self.temporary_buffer_.AddChar(ToLowerCase(cc) as u8);
                        consume_non_newline!(
                            self, source, cc, State::kScriptDataDoubleEscapeEndState, 'states
                        );
                    } else {
                        reconsume_in!(self, State::kScriptDataDoubleEscapedState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:831-851
                State::kBeforeAttributeNameState => {
                    if !self.SkipWhitespaces(source, &mut cc) {
                        return self.HaveBufferedCharacterToken();
                    }
                    if cc == b'/' as UChar {
                        advance_past_non_newline_to!(
                            self, source, cc, State::kSelfClosingStartTagState, 'states
                        );
                    } else if cc == b'>' as UChar {
                        return self.EmitAndResumeInDataState(source);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        reconsume_in!(self, State::kDataState, 'states);
                    } else if matches!(cc, 0x22 | 0x27 | 0x3C | 0x3D) {
                        self.ParseError();
                    }
                    self.token_.AddNewAttribute(ToLowerCaseIfAlpha(cc));
                    if self.options_.track_attributes_ranges {
                        self.attributes_ranges_
                            .AddAttribute(source.NumberOfCharactersConsumed());
                    }
                    advance_past_non_newline_to!(self, source, cc, State::kAttributeNameState, 'states);
                }

                // cpp: html/parser/html_tokenizer.cc:852-897
                State::kAttributeNameState => {
                    while !CheckScanFlag(cc, ScanFlags::kAttributeNameSpecial) {
                        self.token_.AppendToAttributeName(ToLowerCaseIfAlpha(cc));
                        if !self
                            .input_stream_preprocessor_
                            .as_mut()
                            .expect("tokenizer preprocessor must be initialized")
                            .AdvancePastNonNewline(source, &mut cc)
                        {
                            return self.HaveBufferedCharacterToken();
                        }
                    }
                    if IsTokenizerWhitespace(cc) {
                        if self.options_.track_attributes_ranges {
                            self.attributes_ranges_
                                .EndAttributeName(source.NumberOfCharactersConsumed());
                        }
                        advance_to!(self, source, cc, State::kAfterAttributeNameState, 'states);
                    } else if cc == b'/' as UChar {
                        if self.options_.track_attributes_ranges {
                            self.attributes_ranges_
                                .EndAttributeName(source.NumberOfCharactersConsumed());
                        }
                        advance_past_non_newline_to!(
                            self, source, cc, State::kSelfClosingStartTagState, 'states
                        );
                    } else if cc == b'=' as UChar {
                        if self.options_.track_attributes_ranges {
                            self.attributes_ranges_
                                .EndAttributeName(source.NumberOfCharactersConsumed());
                        }
                        advance_past_non_newline_to!(
                            self, source, cc, State::kBeforeAttributeValueState, 'states
                        );
                    } else if cc == b'>' as UChar {
                        if self.options_.track_attributes_ranges {
                            self.attributes_ranges_
                                .EndAttributeName(source.NumberOfCharactersConsumed());
                        }
                        return self.EmitAndResumeInDataState(source);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        if self.options_.track_attributes_ranges {
                            self.attributes_ranges_
                                .EndAttributeName(source.NumberOfCharactersConsumed());
                        }
                        reconsume_in!(self, State::kDataState, 'states);
                    } else {
                        debug_assert!(matches!(cc, 0x22 | 0x27 | 0x3C | 0x3D));
                        self.ParseError();
                        self.token_.AppendToAttributeName(ToLowerCaseIfAlpha(cc));
                        consume_non_newline!(self, source, cc, State::kAttributeNameState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:898-920
                State::kAfterAttributeNameState => {
                    if !self.SkipWhitespaces(source, &mut cc) {
                        return self.HaveBufferedCharacterToken();
                    }
                    if cc == b'/' as UChar {
                        advance_past_non_newline_to!(
                            self, source, cc, State::kSelfClosingStartTagState, 'states
                        );
                    } else if cc == b'=' as UChar {
                        advance_past_non_newline_to!(
                            self, source, cc, State::kBeforeAttributeValueState, 'states
                        );
                    } else if cc == b'>' as UChar {
                        return self.EmitAndResumeInDataState(source);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        reconsume_in!(self, State::kDataState, 'states);
                    } else if matches!(cc, 0x22 | 0x27 | 0x3C) {
                        self.ParseError();
                    }
                    self.token_.AddNewAttribute(ToLowerCaseIfAlpha(cc));
                    if self.options_.track_attributes_ranges {
                        self.attributes_ranges_
                            .AddAttribute(source.NumberOfCharactersConsumed());
                    }
                    advance_past_non_newline_to!(self, source, cc, State::kAttributeNameState, 'states);
                }

                // cpp: html/parser/html_tokenizer.cc:922-960
                State::kBeforeAttributeValueState => {
                    if IsTokenizerWhitespace(cc) {
                        consume!(self, source, cc, State::kBeforeAttributeValueState, 'states);
                    } else if cc == b'"' as UChar {
                        if self.options_.track_attributes_ranges {
                            self.attributes_ranges_
                                .BeginAttributeValue(source.NumberOfCharactersConsumed() + 1);
                        }
                        advance_past_non_newline_to!(
                            self, source, cc, State::kAttributeValueDoubleQuotedState, 'states
                        );
                    } else if cc == b'&' as UChar {
                        if self.options_.track_attributes_ranges {
                            self.attributes_ranges_
                                .BeginAttributeValue(source.NumberOfCharactersConsumed());
                        }
                        reconsume_in!(self, State::kAttributeValueUnquotedState, 'states);
                    } else if cc == b'\'' as UChar {
                        if self.options_.track_attributes_ranges {
                            self.attributes_ranges_
                                .BeginAttributeValue(source.NumberOfCharactersConsumed() + 1);
                        }
                        advance_past_non_newline_to!(
                            self, source, cc, State::kAttributeValueSingleQuotedState, 'states
                        );
                    } else if cc == b'>' as UChar {
                        self.ParseError();
                        return self.EmitAndResumeInDataState(source);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        reconsume_in!(self, State::kDataState, 'states);
                    } else {
                        if matches!(cc, 0x3C | 0x3D | 0x60) {
                            self.ParseError();
                        }
                        if self.options_.track_attributes_ranges {
                            self.attributes_ranges_
                                .BeginAttributeValue(source.NumberOfCharactersConsumed());
                        }
                        self.token_.AppendToAttributeValue(cc);
                        advance_past_non_newline_to!(
                            self, source, cc, State::kAttributeValueUnquotedState, 'states
                        );
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:962-986
                State::kAttributeValueDoubleQuotedState => {
                    if cc == b'"' as UChar {
                        if self.options_.track_attributes_ranges {
                            self.attributes_ranges_
                                .EndAttributeValue(source.NumberOfCharactersConsumed());
                        }
                        advance_past_non_newline_to!(
                            self, source, cc, State::kAfterAttributeValueQuotedState, 'states
                        );
                    } else if cc == b'&' as UChar {
                        self.additional_allowed_character_ = b'"' as UChar;
                        advance_past_non_newline_to!(
                            self, source, cc, State::kCharacterReferenceInAttributeValueState, 'states
                        );
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        if self.options_.track_attributes_ranges {
                            self.attributes_ranges_
                                .EndAttributeValue(source.NumberOfCharactersConsumed());
                        }
                        reconsume_in!(self, State::kDataState, 'states);
                    } else {
                        self.token_.AppendToAttributeValue(cc);
                        consume!(self, source, cc, State::kAttributeValueDoubleQuotedState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:988-1011
                State::kAttributeValueSingleQuotedState => {
                    if cc == b'\'' as UChar {
                        if self.options_.track_attributes_ranges {
                            self.attributes_ranges_
                                .EndAttributeValue(source.NumberOfCharactersConsumed());
                        }
                        advance_past_non_newline_to!(
                            self, source, cc, State::kAfterAttributeValueQuotedState, 'states
                        );
                    } else if cc == b'&' as UChar {
                        self.additional_allowed_character_ = b'\'' as UChar;
                        advance_past_non_newline_to!(
                            self, source, cc, State::kCharacterReferenceInAttributeValueState, 'states
                        );
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        if self.options_.track_attributes_ranges {
                            self.attributes_ranges_
                                .EndAttributeValue(source.NumberOfCharactersConsumed());
                        }
                        reconsume_in!(self, State::kDataState, 'states);
                    } else {
                        self.token_.AppendToAttributeValue(cc);
                        consume!(self, source, cc, State::kAttributeValueSingleQuotedState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1013-1044
                State::kAttributeValueUnquotedState => {
                    if IsTokenizerWhitespace(cc) {
                        if self.options_.track_attributes_ranges {
                            self.attributes_ranges_
                                .EndAttributeValue(source.NumberOfCharactersConsumed());
                        }
                        advance_to!(self, source, cc, State::kBeforeAttributeNameState, 'states);
                    } else if cc == b'&' as UChar {
                        self.additional_allowed_character_ = b'>' as UChar;
                        advance_past_non_newline_to!(
                            self, source, cc, State::kCharacterReferenceInAttributeValueState, 'states
                        );
                    } else if cc == b'>' as UChar {
                        if self.options_.track_attributes_ranges {
                            self.attributes_ranges_
                                .EndAttributeValue(source.NumberOfCharactersConsumed());
                        }
                        return self.EmitAndResumeInDataState(source);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        if self.options_.track_attributes_ranges {
                            self.attributes_ranges_
                                .EndAttributeValue(source.NumberOfCharactersConsumed());
                        }
                        reconsume_in!(self, State::kDataState, 'states);
                    } else {
                        if matches!(cc, 0x22 | 0x27 | 0x3C | 0x3D | 0x60) {
                            self.ParseError();
                        }
                        self.token_.AppendToAttributeValue(cc);
                        consume_non_newline!(
                            self, source, cc, State::kAttributeValueUnquotedState, 'states
                        );
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1046-1077
                State::kCharacterReferenceInAttributeValueState => {
                    let mut not_enough_characters = false;
                    let mut decoded_entity = DecodedHTMLEntity::default();
                    let success = ConsumeHTMLEntity(
                        source,
                        &mut decoded_entity,
                        &mut not_enough_characters,
                        self.additional_allowed_character_,
                    );
                    if not_enough_characters {
                        return self.HaveBufferedCharacterToken();
                    }
                    if !success {
                        debug_assert!(decoded_entity.IsEmpty());
                        self.token_.AppendToAttributeValue(b'&' as UChar);
                    } else {
                        self.token_.SetHasEntity();
                        for i in 0..decoded_entity.length as usize {
                            self.token_.AppendToAttributeValue(decoded_entity.data[i]);
                        }
                    }
                    if self.additional_allowed_character_ == b'"' as UChar {
                        switch_to!(
                            self, source, cc, State::kAttributeValueDoubleQuotedState, 'states
                        );
                    } else if self.additional_allowed_character_ == b'\'' as UChar {
                        switch_to!(
                            self, source, cc, State::kAttributeValueSingleQuotedState, 'states
                        );
                    } else if self.additional_allowed_character_ == b'>' as UChar {
                        switch_to!(self, source, cc, State::kAttributeValueUnquotedState, 'states);
                    } else {
                        unreachable!("unexpected attribute entity delimiter");
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1079-1094
                State::kAfterAttributeValueQuotedState => {
                    if IsTokenizerWhitespace(cc) {
                        advance_to!(self, source, cc, State::kBeforeAttributeNameState, 'states);
                    } else if cc == b'/' as UChar {
                        advance_past_non_newline_to!(
                            self, source, cc, State::kSelfClosingStartTagState, 'states
                        );
                    } else if cc == b'>' as UChar {
                        return self.EmitAndResumeInDataState(source);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        reconsume_in!(self, State::kDataState, 'states);
                    } else {
                        self.ParseError();
                        reconsume_in!(self, State::kBeforeAttributeNameState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1096-1107
                State::kSelfClosingStartTagState => {
                    if cc == b'>' as UChar {
                        self.token_.SetSelfClosing();
                        return self.EmitAndResumeInDataState(source);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        reconsume_in!(self, State::kDataState, 'states);
                    } else {
                        self.ParseError();
                        reconsume_in!(self, State::kBeforeAttributeNameState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1109-1124
                State::kProcessingInstructionOpenState => {
                    assert!(self.options_.processing_instructions);
                    if cc == b'_' as UChar || (cc <= 0x7F && (cc as u8).is_ascii_alphabetic()) {
                        self.token_.BeginProcessingInstruction();
                        reconsume_in!(self, State::kProcessingInstructionTargetState, 'states);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        return self.EmitEndOfFile(source);
                    } else {
                        self.ParseError();
                        self.token_.BeginComment();
                        self.token_.AppendToComment(b'?' as UChar);
                        reconsume_in!(self, State::kContinueBogusCommentState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1126-1159
                State::kProcessingInstructionTargetState => {
                    assert!(self.options_.processing_instructions);
                    if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        return self.EmitEndOfFile(source);
                    } else if cc == b'-' as UChar
                        || cc == b'_' as UChar
                        || (cc <= 0x7F && (cc as u8).is_ascii_alphanumeric())
                    {
                        self.token_.AppendToProcessingInstructionTarget(cc);
                        consume!(self, source, cc, State::kProcessingInstructionTargetState, 'states);
                    } else {
                        let target = self.token_.GetProcessingInstructionTarget().as_slice();
                        let reserved = [b"xml".as_slice(), b"xml-stylesheet".as_slice()]
                            .iter()
                            .any(|name| {
                                target.len() == name.len()
                                    && target.iter().zip(name.iter()).all(|(&unit, &byte)| {
                                        unit <= 0x7F && (unit as u8).eq_ignore_ascii_case(&byte)
                                    })
                            });
                        if !(IsTokenizerWhitespace(cc)
                            || cc == b'>' as UChar
                            || cc == b'?' as UChar)
                            || reserved
                        {
                            self.ParseError();
                            let target_data = target.to_vec();
                            self.ResetFields();
                            self.token_.BeginComment();
                            self.token_.AppendToComment(b'?' as UChar);
                            for character in target_data {
                                self.token_.AppendToComment(character);
                            }
                            reconsume_in!(self, State::kContinueBogusCommentState, 'states);
                        } else {
                            reconsume_in!(self, State::kAfterProcessingInstructionTargetState, 'states);
                        }
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1161-1178
                State::kAfterProcessingInstructionTargetState => {
                    assert!(self.options_.processing_instructions);
                    if !self.SkipWhitespaces(source, &mut cc) {
                        return self.HaveBufferedCharacterToken();
                    }
                    if cc == b'?' as UChar {
                        advance_to!(self, source, cc, State::kProcessingInstructionQuestionableState, 'states);
                    } else if cc == b'>' as UChar {
                        return self.EmitProcessingInstruction(source);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        return self.EmitEndOfFile(source);
                    } else {
                        reconsume_in!(self, State::kProcessingInstructionDataState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1180-1194
                State::kProcessingInstructionDataState => {
                    assert!(self.options_.processing_instructions);
                    if cc == b'?' as UChar {
                        advance_to!(self, source, cc, State::kProcessingInstructionQuestionableState, 'states);
                    } else if cc == b'>' as UChar {
                        return self.EmitProcessingInstruction(source);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        return self.EmitEndOfFile(source);
                    } else {
                        self.token_.AppendToProcessingInstructionData(cc);
                        consume!(self, source, cc, State::kProcessingInstructionDataState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1196-1208
                State::kProcessingInstructionQuestionableState => {
                    assert!(self.options_.processing_instructions);
                    if cc == b'>' as UChar {
                        return self.EmitProcessingInstruction(source);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        return self.EmitEndOfFile(source);
                    } else {
                        self.token_.AppendToProcessingInstructionData(b'?' as UChar);
                        reconsume_in!(self, State::kProcessingInstructionDataState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1210-1214
                State::kBogusCommentState => {
                    self.token_.BeginComment();
                    reconsume_in!(self, State::kContinueBogusCommentState, 'states);
                }

                // cpp: html/parser/html_tokenizer.cc:1216-1226
                State::kContinueBogusCommentState => {
                    if cc == b'>' as UChar {
                        return self.EmitAndResumeInDataState(source);
                    } else if cc == kEndOfFileMarker as UChar {
                        return self.EmitAndReconsumeInDataState();
                    } else {
                        self.token_.AppendToComment(cc);
                        consume!(self, source, cc, State::kContinueBogusCommentState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1228-1265
                State::kMarkupDeclarationOpenState => {
                    if cc == b'-' as UChar {
                        let result = source.LookAhead(&BlinkString::from("--"));
                        if result == LookAheadResult::kDidMatch {
                            source.AdvanceExpecting(b'-' as UChar);
                            source.AdvanceExpecting(b'-' as UChar);
                            self.token_.BeginComment();
                            switch_to!(self, source, cc, State::kCommentStartState, 'states);
                        } else if result == LookAheadResult::kNotEnoughCharacters
                            && self.ShouldWaitForMoreInput(source)
                        {
                            return self.HaveBufferedCharacterToken();
                        }
                    } else if cc == b'D' as UChar || cc == b'd' as UChar {
                        let result = source.LookAheadIgnoringCase(&BlinkString::from("doctype"));
                        if result == LookAheadResult::kDidMatch {
                            source.AdvanceExpectingIgnoringAsciiCase(b"doctype\0");
                            switch_to!(self, source, cc, State::kDOCTYPEState, 'states);
                        } else if result == LookAheadResult::kNotEnoughCharacters
                            && self.ShouldWaitForMoreInput(source)
                        {
                            return self.HaveBufferedCharacterToken();
                        }
                    } else if cc == b'[' as UChar && self.ShouldAllowCDATA() {
                        let result = source.LookAhead(&BlinkString::from("[CDATA["));
                        if result == LookAheadResult::kDidMatch {
                            source.AdvanceExpectingLiteral(b"[CDATA[\0");
                            switch_to!(self, source, cc, State::kCDATASectionState, 'states);
                        } else if result == LookAheadResult::kNotEnoughCharacters
                            && self.ShouldWaitForMoreInput(source)
                        {
                            return self.HaveBufferedCharacterToken();
                        }
                    }
                    self.ParseError();
                    reconsume_in!(self, State::kBogusCommentState, 'states);
                }

                // cpp: html/parser/html_tokenizer.cc:1267-1281
                State::kCommentStartState => {
                    if cc == b'-' as UChar {
                        advance_past_non_newline_to!(self, source, cc, State::kCommentStartDashState, 'states);
                    } else if cc == b'>' as UChar {
                        self.ParseError();
                        return self.EmitAndResumeInDataState(source);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        return self.EmitAndReconsumeInDataState();
                    } else {
                        self.token_.AppendToComment(cc);
                        advance_to!(self, source, cc, State::kCommentState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1283-1298
                State::kCommentStartDashState => {
                    if cc == b'-' as UChar {
                        advance_past_non_newline_to!(self, source, cc, State::kCommentEndState, 'states);
                    } else if cc == b'>' as UChar {
                        self.ParseError();
                        return self.EmitAndResumeInDataState(source);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        return self.EmitAndReconsumeInDataState();
                    } else {
                        self.token_.AppendToComment(b'-' as UChar);
                        self.token_.AppendToComment(cc);
                        advance_to!(self, source, cc, State::kCommentState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1300-1311
                State::kCommentState => {
                    if cc == b'-' as UChar {
                        advance_past_non_newline_to!(self, source, cc, State::kCommentEndDashState, 'states);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        return self.EmitAndReconsumeInDataState();
                    } else {
                        self.token_.AppendToComment(cc);
                        consume!(self, source, cc, State::kCommentState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1313-1325
                State::kCommentEndDashState => {
                    if cc == b'-' as UChar {
                        advance_past_non_newline_to!(self, source, cc, State::kCommentEndState, 'states);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        return self.EmitAndReconsumeInDataState();
                    } else {
                        self.token_.AppendToComment(b'-' as UChar);
                        self.token_.AppendToComment(cc);
                        advance_to!(self, source, cc, State::kCommentState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1327-1348
                State::kCommentEndState => {
                    if cc == b'>' as UChar {
                        return self.EmitAndResumeInDataState(source);
                    } else if cc == b'!' as UChar {
                        self.ParseError();
                        advance_past_non_newline_to!(self, source, cc, State::kCommentEndBangState, 'states);
                    } else if cc == b'-' as UChar {
                        self.ParseError();
                        self.token_.AppendToComment(b'-' as UChar);
                        consume_non_newline!(self, source, cc, State::kCommentEndState, 'states);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        return self.EmitAndReconsumeInDataState();
                    } else {
                        self.ParseError();
                        self.token_.AppendToComment(b'-' as UChar);
                        self.token_.AppendToComment(b'-' as UChar);
                        self.token_.AppendToComment(cc);
                        advance_to!(self, source, cc, State::kCommentState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1350-1369
                State::kCommentEndBangState => {
                    if cc == b'-' as UChar {
                        self.token_.AppendToComment(b'-' as UChar);
                        self.token_.AppendToComment(b'-' as UChar);
                        self.token_.AppendToComment(b'!' as UChar);
                        advance_past_non_newline_to!(self, source, cc, State::kCommentEndDashState, 'states);
                    } else if cc == b'>' as UChar {
                        return self.EmitAndResumeInDataState(source);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        return self.EmitAndReconsumeInDataState();
                    } else {
                        self.token_.AppendToComment(b'-' as UChar);
                        self.token_.AppendToComment(b'-' as UChar);
                        self.token_.AppendToComment(b'!' as UChar);
                        self.token_.AppendToComment(cc);
                        advance_to!(self, source, cc, State::kCommentState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1371-1384
                State::kDOCTYPEState => {
                    if IsTokenizerWhitespace(cc) {
                        advance_to!(self, source, cc, State::kBeforeDOCTYPENameState, 'states);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        self.token_.BeginDOCTYPE();
                        self.token_.SetForceQuirks();
                        return self.EmitAndReconsumeInDataState();
                    } else {
                        self.ParseError();
                        reconsume_in!(self, State::kBeforeDOCTYPENameState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1386-1404
                State::kBeforeDOCTYPENameState => {
                    if !self.SkipWhitespaces(source, &mut cc) {
                        return self.HaveBufferedCharacterToken();
                    }
                    if cc == b'>' as UChar {
                        self.ParseError();
                        self.token_.BeginDOCTYPE();
                        self.token_.SetForceQuirks();
                        return self.EmitAndResumeInDataState(source);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        self.token_.BeginDOCTYPE();
                        self.token_.SetForceQuirks();
                        return self.EmitAndReconsumeInDataState();
                    } else {
                        self.token_
                            .BeginDOCTYPEWithCharacter(ToLowerCaseIfAlpha(cc));
                        advance_past_non_newline_to!(self, source, cc, State::kDOCTYPENameState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1406-1420
                State::kDOCTYPENameState => {
                    if IsTokenizerWhitespace(cc) {
                        advance_to!(self, source, cc, State::kAfterDOCTYPENameState, 'states);
                    } else if cc == b'>' as UChar {
                        return self.EmitAndResumeInDataState(source);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        self.token_.SetForceQuirks();
                        return self.EmitAndReconsumeInDataState();
                    } else {
                        self.token_.AppendToName(ToLowerCaseIfAlpha(cc));
                        consume_non_newline!(self, source, cc, State::kDOCTYPENameState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1422-1458
                State::kAfterDOCTYPENameState => {
                    if !self.SkipWhitespaces(source, &mut cc) {
                        return self.HaveBufferedCharacterToken();
                    }
                    if cc == b'>' as UChar {
                        return self.EmitAndResumeInDataState(source);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        self.token_.SetForceQuirks();
                        return self.EmitAndReconsumeInDataState();
                    } else {
                        if cc == b'P' as UChar || cc == b'p' as UChar {
                            let result = source.LookAheadIgnoringCase(&BlinkString::from("public"));
                            if result == LookAheadResult::kDidMatch {
                                source.AdvanceExpectingIgnoringAsciiCase(b"public\0");
                                switch_to!(self, source, cc, State::kAfterDOCTYPEPublicKeywordState, 'states);
                            } else if result == LookAheadResult::kNotEnoughCharacters
                                && self.ShouldWaitForMoreInput(source)
                            {
                                return self.HaveBufferedCharacterToken();
                            }
                        } else if cc == b'S' as UChar || cc == b's' as UChar {
                            let result = source.LookAheadIgnoringCase(&BlinkString::from("system"));
                            if result == LookAheadResult::kDidMatch {
                                source.AdvanceExpectingIgnoringAsciiCase(b"system\0");
                                switch_to!(self, source, cc, State::kAfterDOCTYPESystemKeywordState, 'states);
                            } else if result == LookAheadResult::kNotEnoughCharacters
                                && self.ShouldWaitForMoreInput(source)
                            {
                                return self.HaveBufferedCharacterToken();
                            }
                        }
                        self.ParseError();
                        self.token_.SetForceQuirks();
                        advance_past_non_newline_to!(self, source, cc, State::kBogusDOCTYPEState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1460-1487
                State::kAfterDOCTYPEPublicKeywordState => {
                    if IsTokenizerWhitespace(cc) {
                        advance_to!(self, source, cc, State::kBeforeDOCTYPEPublicIdentifierState, 'states);
                    } else if cc == b'"' as UChar {
                        self.ParseError();
                        self.token_.SetPublicIdentifierToEmptyString();
                        advance_past_non_newline_to!(self, source, cc, State::kDOCTYPEPublicIdentifierDoubleQuotedState, 'states);
                    } else if cc == b'\'' as UChar {
                        self.ParseError();
                        self.token_.SetPublicIdentifierToEmptyString();
                        advance_past_non_newline_to!(self, source, cc, State::kDOCTYPEPublicIdentifierSingleQuotedState, 'states);
                    } else if cc == b'>' as UChar {
                        self.ParseError();
                        self.token_.SetForceQuirks();
                        return self.EmitAndResumeInDataState(source);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        self.token_.SetForceQuirks();
                        return self.EmitAndReconsumeInDataState();
                    } else {
                        self.ParseError();
                        self.token_.SetForceQuirks();
                        advance_past_non_newline_to!(self, source, cc, State::kBogusDOCTYPEState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1489-1514
                State::kBeforeDOCTYPEPublicIdentifierState => {
                    if !self.SkipWhitespaces(source, &mut cc) {
                        return self.HaveBufferedCharacterToken();
                    }
                    if cc == b'"' as UChar {
                        self.token_.SetPublicIdentifierToEmptyString();
                        advance_past_non_newline_to!(self, source, cc, State::kDOCTYPEPublicIdentifierDoubleQuotedState, 'states);
                    } else if cc == b'\'' as UChar {
                        self.token_.SetPublicIdentifierToEmptyString();
                        advance_past_non_newline_to!(self, source, cc, State::kDOCTYPEPublicIdentifierSingleQuotedState, 'states);
                    } else if cc == b'>' as UChar {
                        self.ParseError();
                        self.token_.SetForceQuirks();
                        return self.EmitAndResumeInDataState(source);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        self.token_.SetForceQuirks();
                        return self.EmitAndReconsumeInDataState();
                    } else {
                        self.ParseError();
                        self.token_.SetForceQuirks();
                        advance_past_non_newline_to!(self, source, cc, State::kBogusDOCTYPEState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1516-1532
                State::kDOCTYPEPublicIdentifierDoubleQuotedState => {
                    if cc == b'"' as UChar {
                        advance_past_non_newline_to!(self, source, cc, State::kAfterDOCTYPEPublicIdentifierState, 'states);
                    } else if cc == b'>' as UChar {
                        self.ParseError();
                        self.token_.SetForceQuirks();
                        return self.EmitAndResumeInDataState(source);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        self.token_.SetForceQuirks();
                        return self.EmitAndReconsumeInDataState();
                    } else {
                        self.token_.AppendToPublicIdentifier(cc);
                        consume!(self, source, cc, State::kDOCTYPEPublicIdentifierDoubleQuotedState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1534-1550
                State::kDOCTYPEPublicIdentifierSingleQuotedState => {
                    if cc == b'\'' as UChar {
                        advance_past_non_newline_to!(self, source, cc, State::kAfterDOCTYPEPublicIdentifierState, 'states);
                    } else if cc == b'>' as UChar {
                        self.ParseError();
                        self.token_.SetForceQuirks();
                        return self.EmitAndResumeInDataState(source);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        self.token_.SetForceQuirks();
                        return self.EmitAndReconsumeInDataState();
                    } else {
                        self.token_.AppendToPublicIdentifier(cc);
                        consume!(self, source, cc, State::kDOCTYPEPublicIdentifierSingleQuotedState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1552-1577
                State::kAfterDOCTYPEPublicIdentifierState => {
                    if IsTokenizerWhitespace(cc) {
                        advance_to!(self, source, cc, State::kBetweenDOCTYPEPublicAndSystemIdentifiersState, 'states);
                    } else if cc == b'>' as UChar {
                        return self.EmitAndResumeInDataState(source);
                    } else if cc == b'"' as UChar {
                        self.ParseError();
                        self.token_.SetSystemIdentifierToEmptyString();
                        advance_past_non_newline_to!(self, source, cc, State::kDOCTYPESystemIdentifierDoubleQuotedState, 'states);
                    } else if cc == b'\'' as UChar {
                        self.ParseError();
                        self.token_.SetSystemIdentifierToEmptyString();
                        advance_past_non_newline_to!(self, source, cc, State::kDOCTYPESystemIdentifierSingleQuotedState, 'states);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        self.token_.SetForceQuirks();
                        return self.EmitAndReconsumeInDataState();
                    } else {
                        self.ParseError();
                        self.token_.SetForceQuirks();
                        advance_past_non_newline_to!(self, source, cc, State::kBogusDOCTYPEState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1579-1602
                State::kBetweenDOCTYPEPublicAndSystemIdentifiersState => {
                    if !self.SkipWhitespaces(source, &mut cc) {
                        return self.HaveBufferedCharacterToken();
                    }
                    if cc == b'>' as UChar {
                        return self.EmitAndResumeInDataState(source);
                    } else if cc == b'"' as UChar {
                        self.token_.SetSystemIdentifierToEmptyString();
                        advance_past_non_newline_to!(self, source, cc, State::kDOCTYPESystemIdentifierDoubleQuotedState, 'states);
                    } else if cc == b'\'' as UChar {
                        self.token_.SetSystemIdentifierToEmptyString();
                        advance_past_non_newline_to!(self, source, cc, State::kDOCTYPESystemIdentifierSingleQuotedState, 'states);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        self.token_.SetForceQuirks();
                        return self.EmitAndReconsumeInDataState();
                    } else {
                        self.ParseError();
                        self.token_.SetForceQuirks();
                        advance_past_non_newline_to!(self, source, cc, State::kBogusDOCTYPEState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1604-1631
                State::kAfterDOCTYPESystemKeywordState => {
                    if IsTokenizerWhitespace(cc) {
                        advance_to!(self, source, cc, State::kBeforeDOCTYPESystemIdentifierState, 'states);
                    } else if cc == b'"' as UChar {
                        self.ParseError();
                        self.token_.SetSystemIdentifierToEmptyString();
                        advance_past_non_newline_to!(self, source, cc, State::kDOCTYPESystemIdentifierDoubleQuotedState, 'states);
                    } else if cc == b'\'' as UChar {
                        self.ParseError();
                        self.token_.SetSystemIdentifierToEmptyString();
                        advance_past_non_newline_to!(self, source, cc, State::kDOCTYPESystemIdentifierSingleQuotedState, 'states);
                    } else if cc == b'>' as UChar {
                        self.ParseError();
                        self.token_.SetForceQuirks();
                        return self.EmitAndResumeInDataState(source);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        self.token_.SetForceQuirks();
                        return self.EmitAndReconsumeInDataState();
                    } else {
                        self.ParseError();
                        self.token_.SetForceQuirks();
                        advance_past_non_newline_to!(self, source, cc, State::kBogusDOCTYPEState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1633-1658
                State::kBeforeDOCTYPESystemIdentifierState => {
                    if !self.SkipWhitespaces(source, &mut cc) {
                        return self.HaveBufferedCharacterToken();
                    }
                    if cc == b'"' as UChar {
                        self.token_.SetSystemIdentifierToEmptyString();
                        advance_past_non_newline_to!(self, source, cc, State::kDOCTYPESystemIdentifierDoubleQuotedState, 'states);
                    } else if cc == b'\'' as UChar {
                        self.token_.SetSystemIdentifierToEmptyString();
                        advance_past_non_newline_to!(self, source, cc, State::kDOCTYPESystemIdentifierSingleQuotedState, 'states);
                    } else if cc == b'>' as UChar {
                        self.ParseError();
                        self.token_.SetForceQuirks();
                        return self.EmitAndResumeInDataState(source);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        self.token_.SetForceQuirks();
                        return self.EmitAndReconsumeInDataState();
                    } else {
                        self.ParseError();
                        self.token_.SetForceQuirks();
                        advance_past_non_newline_to!(self, source, cc, State::kBogusDOCTYPEState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1660-1676
                State::kDOCTYPESystemIdentifierDoubleQuotedState => {
                    if cc == b'"' as UChar {
                        advance_past_non_newline_to!(self, source, cc, State::kAfterDOCTYPESystemIdentifierState, 'states);
                    } else if cc == b'>' as UChar {
                        self.ParseError();
                        self.token_.SetForceQuirks();
                        return self.EmitAndResumeInDataState(source);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        self.token_.SetForceQuirks();
                        return self.EmitAndReconsumeInDataState();
                    } else {
                        self.token_.AppendToSystemIdentifier(cc);
                        consume!(self, source, cc, State::kDOCTYPESystemIdentifierDoubleQuotedState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1678-1694
                State::kDOCTYPESystemIdentifierSingleQuotedState => {
                    if cc == b'\'' as UChar {
                        advance_past_non_newline_to!(self, source, cc, State::kAfterDOCTYPESystemIdentifierState, 'states);
                    } else if cc == b'>' as UChar {
                        self.ParseError();
                        self.token_.SetForceQuirks();
                        return self.EmitAndResumeInDataState(source);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        self.token_.SetForceQuirks();
                        return self.EmitAndReconsumeInDataState();
                    } else {
                        self.token_.AppendToSystemIdentifier(cc);
                        consume!(self, source, cc, State::kDOCTYPESystemIdentifierSingleQuotedState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1696-1710
                State::kAfterDOCTYPESystemIdentifierState => {
                    if !self.SkipWhitespaces(source, &mut cc) {
                        return self.HaveBufferedCharacterToken();
                    }
                    if cc == b'>' as UChar {
                        return self.EmitAndResumeInDataState(source);
                    } else if cc == kEndOfFileMarker as UChar {
                        self.ParseError();
                        self.token_.SetForceQuirks();
                        return self.EmitAndReconsumeInDataState();
                    } else {
                        self.ParseError();
                        advance_past_non_newline_to!(self, source, cc, State::kBogusDOCTYPEState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1712-1719
                State::kBogusDOCTYPEState => {
                    if cc == b'>' as UChar {
                        return self.EmitAndResumeInDataState(source);
                    } else if cc == kEndOfFileMarker as UChar {
                        return self.EmitAndReconsumeInDataState();
                    }
                    consume!(self, source, cc, State::kBogusDOCTYPEState, 'states);
                }

                // cpp: html/parser/html_tokenizer.cc:1721-1731
                State::kCDATASectionState => {
                    if cc == b']' as UChar {
                        advance_past_non_newline_to!(self, source, cc, State::kCDATASectionBracketState, 'states);
                    } else if cc == kEndOfFileMarker as UChar {
                        reconsume_in!(self, State::kDataState, 'states);
                    } else {
                        self.BufferCharacter(cc);
                        consume!(self, source, cc, State::kCDATASectionState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1733-1740
                State::kCDATASectionBracketState => {
                    if cc == b']' as UChar {
                        advance_past_non_newline_to!(self, source, cc, State::kCDATASectionEndState, 'states);
                    } else {
                        self.BufferCharacter(b']' as UChar);
                        reconsume_in!(self, State::kCDATASectionState, 'states);
                    }
                }

                // cpp: html/parser/html_tokenizer.cc:1742-1754
                State::kCDATASectionEndState => {
                    if cc == b']' as UChar {
                        self.BufferCharacter(b']' as UChar);
                        consume_non_newline!(self, source, cc, State::kCDATASectionEndState, 'states);
                    } else if cc == b'>' as UChar {
                        advance_past_non_newline_to!(self, source, cc, State::kDataState, 'states);
                    } else {
                        self.BufferCharacter(b']' as UChar);
                        self.BufferCharacter(b']' as UChar);
                        reconsume_in!(self, State::kCDATASectionState, 'states);
                    }
                }
            }
        }
    }
}
