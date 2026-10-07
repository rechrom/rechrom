#![allow(non_snake_case, non_camel_case_types)]

use super::html_parser_idioms::{kAttributePrealloc, AttemptStaticStringCreationBuffer};
use super::literal_buffer::{LCharLiteralBuffer, UCharLiteralBuffer};
use foundation::{AtomicString, BlinkString};

// cpp: html/parser/html_token.h:42-58
#[derive(Default)]
pub struct DoctypeData {
    pub has_public_identifier_: bool,
    pub has_system_identifier_: bool,
    pub public_identifier_: Vec<u16>,
    pub system_identifier_: Vec<u16>,
    pub force_quirks_: bool,
}

// cpp: html/parser/html_token.h:60-63
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DOMPartTokenType {
    kChildNodePartStart,
    kChildNodePartEnd,
}

// cpp: html/parser/html_token.h:65-76
pub struct DOMPartData {
    pub metadata_: Vec<BlinkString>,
    pub type_: DOMPartTokenType,
}
impl DOMPartData {
    pub fn new(part_type: DOMPartTokenType) -> Self {
        Self {
            metadata_: Vec::new(),
            type_: part_type,
        }
    }
}

// cpp: html/parser/html_token.h:78-85
#[derive(Default)]
pub struct DOMPartsNeeded {
    pub needs_node_part: bool,
    pub needs_attribute_parts: Vec<AtomicString>,
}
impl DOMPartsNeeded {
    pub fn IsNeeded(&self) -> bool {
        self.needs_node_part || !self.needs_attribute_parts.is_empty()
    }
}

// cpp: html/parser/html_token.h:91-100
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(i32)]
pub enum TokenType {
    #[default]
    kUninitialized,
    DOCTYPE,
    kStartTag,
    kEndTag,
    kComment,
    kCharacter,
    kEndOfFile,
    kProcessingInstruction,
}

// cpp: html/parser/html_token.h:427-430
// This debug-only function is declared but never defined in the source tree.
// Retain only the declaration; callers require a real external definition.
#[cfg(debug_assertions)]
unsafe extern "C" {
    #[link_name = "_ZN5blink8ToStringENS_9HTMLToken9TokenTypeE"]
    pub fn DebugTokenTypeToString(token_type: TokenType) -> *const std::ffi::c_char;
}

// cpp: html/parser/html_token.h:102-124
#[derive(Default)]
pub struct Attribute {
    name_: UCharLiteralBuffer<32>,
    value_: UCharLiteralBuffer<32>,
}
impl Attribute {
    pub fn GetName(&self) -> AtomicString {
        self.name_.AsAtomicString()
    }
    pub fn GetValue(&self) -> AtomicString {
        self.value_.AsAtomicString()
    }
    pub fn NameBuffer(&self) -> &UCharLiteralBuffer<32> {
        &self.name_
    }
    pub fn NameAttemptStaticStringCreation(&self) -> BlinkString {
        AttemptStaticStringCreationBuffer(&self.name_)
    }
    pub fn NameIsEmpty(&self) -> bool {
        self.name_.IsEmpty()
    }
    pub fn AppendToName(&mut self, character: u16) {
        self.name_.AddChar(character);
    }
    pub fn Value(&self) -> BlinkString {
        self.value_.AsString()
    }
    pub fn AppendToValue(&mut self, character: u16) {
        self.value_.AddChar(character);
    }
    pub fn ClearValue(&mut self) {
        self.value_.clear();
    }
}

// cpp: html/parser/html_token.h:87-89,126-137,406-426
pub struct HTMLToken {
    data_: UCharLiteralBuffer<256>,
    attributes_: Vec<Attribute>,
    current_attribute_: Option<usize>,
    doctype_data_: Option<DoctypeData>,
    processing_instruction_target_: Option<UCharLiteralBuffer<256>>,
    type_: TokenType,
    has_entity_: bool,
    self_closing_: bool,
}

impl Default for HTMLToken {
    fn default() -> Self {
        Self {
            data_: UCharLiteralBuffer::default(),
            attributes_: Vec::with_capacity(kAttributePrealloc),
            current_attribute_: None,
            doctype_data_: None,
            processing_instruction_target_: None,
            type_: TokenType::kUninitialized,
            has_entity_: false,
            // C++ leaves this uninitialized until BeginStartTag/BeginEndTag.
            self_closing_: false,
        }
    }
}

impl HTMLToken {
    // cpp: html/parser/html_token.h:142-153
    pub fn Take(&mut self) -> Box<HTMLToken> {
        let mut copy = Box::new(HTMLToken::default());
        copy.data_ = std::mem::take(&mut self.data_);
        copy.attributes_ = std::mem::take(&mut self.attributes_);
        copy.doctype_data_ = self.doctype_data_.take();
        copy.type_ = self.type_;
        copy.self_closing_ = self.self_closing_;
        copy.has_entity_ = self.has_entity_;
        self.Clear();
        copy
    }

    // cpp: html/parser/html_token.h:155-167
    pub fn Clear(&mut self) {
        if self.type_ == TokenType::kUninitialized {
            return;
        }
        self.type_ = TokenType::kUninitialized;
        self.has_entity_ = false;
        self.data_.clear();
        self.processing_instruction_target_ = None;
        if self.current_attribute_.is_some() {
            self.current_attribute_ = None;
            self.attributes_.clear();
        }
    }

    // cpp: html/parser/html_token.h:169-175
    pub fn IsUninitialized(&self) -> bool {
        self.type_ == TokenType::kUninitialized
    }
    pub fn GetType(&self) -> TokenType {
        self.type_
    }
    pub fn MakeEndOfFile(&mut self) {
        debug_assert_eq!(self.type_, TokenType::kUninitialized);
        self.type_ = TokenType::kEndOfFile;
    }

    // cpp: html/parser/html_token.h:177-197
    pub fn Data(&self) -> &UCharLiteralBuffer<256> {
        debug_assert!(matches!(
            self.type_,
            TokenType::kCharacter
                | TokenType::kComment
                | TokenType::kStartTag
                | TokenType::kEndTag
                | TokenType::kProcessingInstruction
        ));
        &self.data_
    }
    pub fn GetName(&self) -> &UCharLiteralBuffer<256> {
        debug_assert!(matches!(
            self.type_,
            TokenType::kStartTag | TokenType::kEndTag | TokenType::DOCTYPE
        ));
        &self.data_
    }
    pub fn GetProcessingInstructionTarget(&self) -> &UCharLiteralBuffer<256> {
        debug_assert_eq!(self.type_, TokenType::kProcessingInstruction);
        self.processing_instruction_target_
            .as_ref()
            .expect("processing instruction target required")
    }
    pub fn AppendToName(&mut self, character: u16) {
        debug_assert!(matches!(
            self.type_,
            TokenType::kStartTag
                | TokenType::kEndTag
                | TokenType::DOCTYPE
                | TokenType::kProcessingInstruction
        ));
        debug_assert_ne!(character, 0);
        self.data_.AddChar(character);
    }

    // cpp: html/parser/html_token.h:201-263
    pub fn ForceQuirks(&self) -> bool {
        debug_assert_eq!(self.type_, TokenType::DOCTYPE);
        self.doctype_data_
            .as_ref()
            .expect("doctype data required")
            .force_quirks_
    }
    pub fn SetForceQuirks(&mut self) {
        debug_assert_eq!(self.type_, TokenType::DOCTYPE);
        self.doctype_data_
            .as_mut()
            .expect("doctype data required")
            .force_quirks_ = true;
    }
    pub fn BeginDOCTYPE(&mut self) {
        debug_assert_eq!(self.type_, TokenType::kUninitialized);
        self.type_ = TokenType::DOCTYPE;
        self.doctype_data_ = Some(DoctypeData::default());
    }
    pub fn BeginDOCTYPEWithCharacter(&mut self, character: u16) {
        debug_assert_ne!(character, 0);
        self.BeginDOCTYPE();
        self.data_.AddChar(character);
    }
    pub fn PublicIdentifier(&self) -> &[u16] {
        debug_assert_eq!(self.type_, TokenType::DOCTYPE);
        &self
            .doctype_data_
            .as_ref()
            .expect("doctype data required")
            .public_identifier_
    }
    pub fn SystemIdentifier(&self) -> &[u16] {
        debug_assert_eq!(self.type_, TokenType::DOCTYPE);
        &self
            .doctype_data_
            .as_ref()
            .expect("doctype data required")
            .system_identifier_
    }
    pub fn SetPublicIdentifierToEmptyString(&mut self) {
        debug_assert_eq!(self.type_, TokenType::DOCTYPE);
        let data = self.doctype_data_.as_mut().expect("doctype data required");
        data.has_public_identifier_ = true;
        data.public_identifier_.clear();
    }
    pub fn SetSystemIdentifierToEmptyString(&mut self) {
        debug_assert_eq!(self.type_, TokenType::DOCTYPE);
        let data = self.doctype_data_.as_mut().expect("doctype data required");
        data.has_system_identifier_ = true;
        data.system_identifier_.clear();
    }
    pub fn AppendToPublicIdentifier(&mut self, character: u16) {
        debug_assert_ne!(character, 0);
        debug_assert_eq!(self.type_, TokenType::DOCTYPE);
        let data = self.doctype_data_.as_mut().expect("doctype data required");
        debug_assert!(data.has_public_identifier_);
        data.public_identifier_.push(character);
    }
    pub fn AppendToSystemIdentifier(&mut self, character: u16) {
        debug_assert_ne!(character, 0);
        debug_assert_eq!(self.type_, TokenType::DOCTYPE);
        let data = self.doctype_data_.as_mut().expect("doctype data required");
        debug_assert!(data.has_system_identifier_);
        data.system_identifier_.push(character);
    }
    pub fn ReleaseDoctypeData(&mut self) -> Option<DoctypeData> {
        self.doctype_data_.take()
    }

    // cpp: html/parser/html_token.h:267-332
    pub fn SelfClosing(&self) -> bool {
        debug_assert!(matches!(
            self.type_,
            TokenType::kStartTag | TokenType::kEndTag
        ));
        self.self_closing_
    }
    pub fn SetSelfClosing(&mut self) {
        debug_assert!(matches!(
            self.type_,
            TokenType::kStartTag | TokenType::kEndTag
        ));
        self.self_closing_ = true;
    }
    pub fn BeginStartTag(&mut self, character: u8) {
        debug_assert_ne!(character, 0);
        debug_assert_eq!(self.type_, TokenType::kUninitialized);
        self.type_ = TokenType::kStartTag;
        self.self_closing_ = false;
        debug_assert!(self.current_attribute_.is_none());
        debug_assert!(self.attributes_.is_empty());
        self.data_.AddChar(u16::from(character));
    }
    pub fn BeginEndTag(&mut self, character: u8) {
        debug_assert_eq!(self.type_, TokenType::kUninitialized);
        self.type_ = TokenType::kEndTag;
        self.self_closing_ = false;
        debug_assert!(self.current_attribute_.is_none());
        debug_assert!(self.attributes_.is_empty());
        self.data_.AddChar(u16::from(character));
    }
    pub fn BeginEndTagBuffer(&mut self, characters: &LCharLiteralBuffer<32>) {
        debug_assert_eq!(self.type_, TokenType::kUninitialized);
        self.type_ = TokenType::kEndTag;
        self.self_closing_ = false;
        debug_assert!(self.current_attribute_.is_none());
        debug_assert!(self.attributes_.is_empty());
        self.data_.AppendLiteral(characters);
    }
    pub fn AddNewAttribute(&mut self, character: u16) {
        debug_assert!(matches!(
            self.type_,
            TokenType::kStartTag | TokenType::kEndTag
        ));
        self.attributes_.push(Attribute::default());
        self.current_attribute_ = Some(self.attributes_.len() - 1);
        self.attributes_.last_mut().unwrap().AppendToName(character);
    }
    pub fn AppendToAttributeName(&mut self, character: u16) {
        debug_assert_ne!(character, 0);
        debug_assert!(matches!(
            self.type_,
            TokenType::kStartTag | TokenType::kEndTag
        ));
        self.attributes_[self.current_attribute_.expect("current attribute required")]
            .AppendToName(character);
    }
    pub fn AppendToAttributeValue(&mut self, character: u16) {
        debug_assert_ne!(character, 0);
        debug_assert!(matches!(
            self.type_,
            TokenType::kStartTag | TokenType::kEndTag
        ));
        self.attributes_[self.current_attribute_.expect("current attribute required")]
            .AppendToValue(character);
    }
    pub fn Attributes(&self) -> &[Attribute] {
        debug_assert!(matches!(
            self.type_,
            TokenType::kStartTag | TokenType::kEndTag
        ));
        &self.attributes_
    }

    // cpp: html/parser/html_token.h:338-371
    pub fn EnsureIsCharacterToken(&mut self) {
        debug_assert!(matches!(
            self.type_,
            TokenType::kUninitialized | TokenType::kCharacter
        ));
        self.type_ = TokenType::kCharacter;
    }
    pub fn Characters(&self) -> &UCharLiteralBuffer<256> {
        debug_assert_eq!(self.type_, TokenType::kCharacter);
        &self.data_
    }
    pub fn AppendToCharacterChar(&mut self, character: i8) {
        debug_assert_eq!(self.type_, TokenType::kCharacter);
        self.data_.AddChar(character as u16);
    }
    pub fn AppendToCharacter(&mut self, character: u16) {
        debug_assert_eq!(self.type_, TokenType::kCharacter);
        self.data_.AddChar(character);
    }
    pub fn AppendToCharacterBuffer(&mut self, characters: &LCharLiteralBuffer<32>) {
        debug_assert_eq!(self.type_, TokenType::kCharacter);
        self.data_.AppendLiteral(characters);
    }
    pub fn AppendToProcessingInstructionTarget(&mut self, character: u16) {
        debug_assert_eq!(self.type_, TokenType::kProcessingInstruction);
        self.processing_instruction_target_
            .as_mut()
            .expect("processing instruction target required")
            .AddChar(character);
    }
    pub fn HasEntity(&self) -> bool {
        self.has_entity_
    }
    pub fn SetHasEntity(&mut self) {
        self.has_entity_ = true;
    }

    // cpp: html/parser/html_token.h:375-402
    pub fn Comment(&self) -> &UCharLiteralBuffer<256> {
        debug_assert_eq!(self.type_, TokenType::kComment);
        &self.data_
    }
    pub fn BeginComment(&mut self) {
        debug_assert_eq!(self.type_, TokenType::kUninitialized);
        self.type_ = TokenType::kComment;
    }
    pub fn AppendToComment(&mut self, character: u16) {
        debug_assert_ne!(character, 0);
        debug_assert_eq!(self.type_, TokenType::kComment);
        self.data_.AddChar(character);
    }
    pub fn AppendToProcessingInstructionData(&mut self, character: u16) {
        debug_assert_ne!(character, 0);
        debug_assert_eq!(self.type_, TokenType::kProcessingInstruction);
        self.data_.AddChar(character);
    }
    pub fn BeginProcessingInstruction(&mut self) {
        debug_assert_eq!(self.type_, TokenType::kUninitialized);
        self.type_ = TokenType::kProcessingInstruction;
        self.processing_instruction_target_ = Some(UCharLiteralBuffer::default());
    }
}
