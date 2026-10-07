#![allow(non_snake_case, non_upper_case_globals)]

use foundation::{UChar, UChar32};

use super::html_entity_search::HTMLEntitySearch;
use super::html_entity_table::{HTMLEntityTable, HTMLEntityTableEntry};
use super::segmented_string::{PrependType, SegmentedString};

// cpp: html/parser/html_entity_parser.h:39-65
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DecodedHTMLEntity {
    pub length: u32,
    pub data: [UChar; 4],
}

pub trait EntityAppendValue {
    fn append_to(self, decoded: &mut DecodedHTMLEntity);
}

impl DecodedHTMLEntity {
    // cpp: html/parser/html_entity_parser.h:43-44
    pub const kMaxLength: u32 = 4;

    // cpp: html/parser/html_entity_parser.h:47
    pub fn IsEmpty(&self) -> bool {
        self.length == 0
    }

    // cpp: html/parser/html_entity_parser.h:49-61
    pub fn Append<T: EntityAppendValue>(&mut self, c: T) {
        c.append_to(self);
    }
}

impl EntityAppendValue for UChar {
    // cpp: html/parser/html_entity_parser.h:49-52
    fn append_to(self, decoded: &mut DecodedHTMLEntity) {
        assert!(decoded.length < DecodedHTMLEntity::kMaxLength);
        decoded.data[decoded.length as usize] = self;
        decoded.length += 1;
    }
}

impl EntityAppendValue for UChar32 {
    // cpp: html/parser/html_entity_parser.h:54-61
    fn append_to(self, decoded: &mut DecodedHTMLEntity) {
        if (0..=0xFFFF).contains(&self) {
            decoded.Append(self as UChar);
            return;
        }
        let supplementary = self - 0x10000;
        decoded.Append((0xD800 + (supplementary >> 10)) as UChar);
        decoded.Append((0xDC00 + (supplementary & 0x3FF)) as UChar);
    }
}

// cpp: html/parser/html_entity_parser.cc:41-46
const kWindowsLatin1ExtensionArray: [UChar; 32] = [
    0x20AC, 0x0081, 0x201A, 0x0192, 0x201E, 0x2026, 0x2020, 0x2021, 0x02C6, 0x2030, 0x0160, 0x2039,
    0x0152, 0x008D, 0x017D, 0x008F, 0x0090, 0x2018, 0x2019, 0x201C, 0x201D, 0x2022, 0x2013, 0x2014,
    0x02DC, 0x2122, 0x0161, 0x203A, 0x0153, 0x009D, 0x017E, 0x0178,
];

// cpp: html/parser/html_entity_parser.cc:48-52
fn AdjustEntity(value: UChar32) -> UChar {
    if (value & !0x1F) != 0x0080 {
        return value as UChar;
    }
    kWindowsLatin1ExtensionArray[(value - 0x80) as usize]
}

// cpp: html/parser/html_entity_parser.cc:54-60
fn AppendMatchToDecoded(
    match_entry: &HTMLEntityTableEntry,
    decoded_entity: &mut DecodedHTMLEntity,
) {
    decoded_entity.Append(match_entry.first_value);
    if match_entry.second_value != 0 {
        decoded_entity.Append(match_entry.second_value);
    }
}

// cpp: html/parser/html_entity_parser.cc:62
const kInvalidUnicode: UChar32 = -1;

// cpp: html/parser/html_entity_parser.cc:64
type ConsumedCharacterBuffer = Vec<UChar>;

// cpp: html/parser/html_entity_parser.cc:66-76
fn UnconsumeCharacters(
    source: &mut SegmentedString,
    consumed_characters: &ConsumedCharacterBuffer,
) {
    if consumed_characters.len() == 1 {
        source.Push(consumed_characters[0]);
    } else if consumed_characters.len() == 2 {
        source.Push(consumed_characters[1]);
        source.Push(consumed_characters[0]);
    } else {
        let pushed =
            SegmentedString::from_string(&foundation::BlinkString::from_utf16(consumed_characters));
        source.Prepend(&pushed, PrependType::kUnconsume);
    }
}

// cpp: html/parser/html_entity_parser.cc:78-134
fn ConsumeNamedEntity(
    source: &mut SegmentedString,
    decoded_entity: &mut DecodedHTMLEntity,
    not_enough_characters: &mut bool,
    additional_allowed_character: UChar,
    cc: &mut UChar,
) -> bool {
    let mut consumed_characters = ConsumedCharacterBuffer::with_capacity(64);
    let mut entity_search = HTMLEntitySearch::new();
    while !source.IsEmpty() {
        *cc = source.CurrentChar();
        entity_search.Advance(*cc);
        if !entity_search.IsEntityPrefix() {
            break;
        }
        consumed_characters.push(*cc);
        source.AdvanceExpecting(*cc);
    }
    *not_enough_characters = source.IsEmpty() && *cc != b';' as UChar;
    if *not_enough_characters {
        UnconsumeCharacters(source, &consumed_characters);
        return false;
    }
    if entity_search.MostRecentMatch().is_none() {
        UnconsumeCharacters(source, &consumed_characters);
        return false;
    }
    if entity_search
        .MostRecentMatch()
        .expect("checked match")
        .length
        != entity_search.CurrentLength()
    {
        UnconsumeCharacters(source, &consumed_characters);
        consumed_characters.clear();
        let most_recent = entity_search.MostRecentMatch().expect("checked match");
        let reference = HTMLEntityTable::EntityString(most_recent);
        for &character in reference {
            *cc = source.CurrentChar();
            debug_assert_eq!(*cc, character as UChar);
            consumed_characters.push(*cc);
            source.AdvanceExpecting(*cc);
            debug_assert!(!source.IsEmpty());
        }
        *cc = source.CurrentChar();
    }
    let most_recent = entity_search.MostRecentMatch().expect("checked match");
    if most_recent.LastCharacter() == b';'
        || additional_allowed_character == 0
        || !((*cc <= 0x7F && (*cc as u8).is_ascii_alphanumeric()) || *cc == b'=' as UChar)
    {
        AppendMatchToDecoded(most_recent, decoded_entity);
        return true;
    }
    UnconsumeCharacters(source, &consumed_characters);
    false
}

// cpp: html/parser/html_entity_parser.h:67
// cpp: html/parser/html_entity_parser.cc:138-149
pub fn AppendLegalEntityFor(c: UChar32, decoded_entity: &mut DecodedHTMLEntity) {
    if c <= 0 || c > 0x10FFFF || (0xD800..=0xDFFF).contains(&c) {
        decoded_entity.Append(0xFFFD_i32);
        return;
    }
    if c <= 0xFFFF {
        decoded_entity.Append(AdjustEntity(c));
        return;
    }
    decoded_entity.Append(c);
}

// cpp: html/parser/html_entity_parser.h:69-72
// cpp: html/parser/html_entity_parser.cc:151-271
pub fn ConsumeHTMLEntity(
    source: &mut SegmentedString,
    decoded_entity: &mut DecodedHTMLEntity,
    not_enough_characters: &mut bool,
    additional_allowed_character: UChar,
) -> bool {
    debug_assert!(
        additional_allowed_character == 0
            || additional_allowed_character == b'"' as UChar
            || additional_allowed_character == b'\'' as UChar
            || additional_allowed_character == b'>' as UChar
    );
    debug_assert!(!*not_enough_characters);
    debug_assert!(decoded_entity.IsEmpty());

    // cpp: html/parser/html_entity_parser.cc:161-169
    #[derive(Clone, Copy)]
    enum EntityState {
        kInitial,
        kNumber,
        kMaybeHexLowerCaseX,
        kMaybeHexUpperCaseX,
        kHex,
        kDecimal,
        kNamed,
    }
    let mut entity_state = EntityState::kInitial;
    let mut result: UChar32 = 0;
    let mut consumed_characters = ConsumedCharacterBuffer::with_capacity(64);

    // cpp: html/parser/html_entity_parser.cc:174-176
    while !source.IsEmpty() {
        let cc = source.CurrentChar();
        match entity_state {
            // cpp: html/parser/html_entity_parser.cc:177-192
            EntityState::kInitial => {
                if matches!(cc, 0x09 | 0x0A | 0x0C | 0x20 | 0x3C | 0x26) {
                    return false;
                }
                if additional_allowed_character != 0 && cc == additional_allowed_character {
                    return false;
                }
                if cc == b'#' as UChar {
                    entity_state = EntityState::kNumber;
                } else if cc <= 0x7F && (cc as u8).is_ascii_alphabetic() {
                    entity_state = EntityState::kNamed;
                    continue;
                } else {
                    return false;
                }
            }
            // cpp: html/parser/html_entity_parser.cc:193-208
            EntityState::kNumber => {
                if cc == b'x' as UChar {
                    entity_state = EntityState::kMaybeHexLowerCaseX;
                } else if cc == b'X' as UChar {
                    entity_state = EntityState::kMaybeHexUpperCaseX;
                } else if cc <= 0x7F && (cc as u8).is_ascii_digit() {
                    entity_state = EntityState::kDecimal;
                    continue;
                } else {
                    source.Push(b'#' as UChar);
                    return false;
                }
            }
            // cpp: html/parser/html_entity_parser.cc:209-217
            EntityState::kMaybeHexLowerCaseX => {
                if cc <= 0x7F && (cc as u8).is_ascii_hexdigit() {
                    entity_state = EntityState::kHex;
                    continue;
                }
                source.Push(b'x' as UChar);
                source.Push(b'#' as UChar);
                return false;
            }
            // cpp: html/parser/html_entity_parser.cc:218-226
            EntityState::kMaybeHexUpperCaseX => {
                if cc <= 0x7F && (cc as u8).is_ascii_hexdigit() {
                    entity_state = EntityState::kHex;
                    continue;
                }
                source.Push(b'X' as UChar);
                source.Push(b'#' as UChar);
                return false;
            }
            // cpp: html/parser/html_entity_parser.cc:227-240
            EntityState::kHex => {
                if cc <= 0x7F && (cc as u8).is_ascii_hexdigit() {
                    if result != kInvalidUnicode {
                        result = result * 16
                            + (cc as u8 as char).to_digit(16).expect("ASCII hex") as UChar32;
                    }
                } else if cc == b';' as UChar {
                    source.AdvanceExpecting(cc);
                    AppendLegalEntityFor(result, decoded_entity);
                    return true;
                } else {
                    AppendLegalEntityFor(result, decoded_entity);
                    return true;
                }
            }
            // cpp: html/parser/html_entity_parser.cc:241-254
            EntityState::kDecimal => {
                if cc <= 0x7F && (cc as u8).is_ascii_digit() {
                    if result != kInvalidUnicode {
                        result = result * 10 + cc as UChar32 - b'0' as UChar32;
                    }
                } else if cc == b';' as UChar {
                    source.AdvanceExpecting(cc);
                    AppendLegalEntityFor(result, decoded_entity);
                    return true;
                } else {
                    AppendLegalEntityFor(result, decoded_entity);
                    return true;
                }
            }
            // cpp: html/parser/html_entity_parser.cc:255-258
            EntityState::kNamed => {
                let mut current_character = cc;
                return ConsumeNamedEntity(
                    source,
                    decoded_entity,
                    not_enough_characters,
                    additional_allowed_character,
                    &mut current_character,
                );
            }
        }

        // cpp: html/parser/html_entity_parser.cc:261-265
        if result > 0x10FFFF {
            result = kInvalidUnicode;
        }
        consumed_characters.push(cc);
        source.AdvanceExpecting(cc);
    }
    // cpp: html/parser/html_entity_parser.cc:267-271
    debug_assert!(source.IsEmpty());
    *not_enough_characters = true;
    UnconsumeCharacters(source, &consumed_characters);
    false
}

// cpp: html/parser/html_entity_parser.h:74-76
// cpp: html/parser/html_entity_parser.cc:273-288
pub fn DecodeNamedEntity(name: &[u8]) -> Option<DecodedHTMLEntity> {
    let mut search = HTMLEntitySearch::new();
    for &c in name {
        search.Advance(c as UChar);
        if !search.IsEntityPrefix() {
            return None;
        }
    }
    search.Advance(b';' as UChar);
    if !search.IsEntityPrefix() {
        return None;
    }
    let mut decoded_entity = DecodedHTMLEntity::default();
    AppendMatchToDecoded(
        search.MostRecentMatch().expect("matched semicolon"),
        &mut decoded_entity,
    );
    Some(decoded_entity)
}
