#![allow(non_snake_case, non_upper_case_globals)]

use foundation::{UChar, UChar32};

// cpp: html/parser/html_entity_table.h:35-44
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HTMLEntityTableEntry {
    pub first_value: UChar32,
    pub second_value: UChar,
    pub entity_offset: u16,
    pub length: u16,
}

// cpp: html/parser/html_entity_table.h:46-54
pub struct HTMLEntityTable {
    _private: (),
}

// cpp: html/parser/html_entity_table.cc:41-41
#[rustfmt::skip]
const kStaticEntityStringStorage: [u8; 14485] = [
// cpp: html/parser/html_entity_table.cc:42-73
    b'A', b'E', b'l', b'i', b'g',
    b';',
    b'A', b'M', b'P',
    b';',
    b'A', b'a', b'c', b'u', b't', b'e',
    b';',
    b'A', b'b', b'r', b'e', b'v', b'e', b';',
    b'A', b'c', b'i', b'r', b'c',
    b';',
    b'A', b'c', b'y', b';',
    b'A', b'f', b'r', b';',
    b'A', b'g', b'r', b'a', b'v', b'e',
    b';',
    b'A', b'l', b'p', b'h', b'a', b';',
    b'A', b'm', b'a', b'c', b'r', b';',
    b'A', b'n', b'd', b';',
    b'A', b'o', b'g', b'o', b'n', b';',
    b'A', b'o', b'p', b'f', b';',
    b'A', b'p', b'p', b'l', b'y', b'F', b'u', b'n', b'c', b't', b'i', b'o', b'n', b';',
    b'A', b'r', b'i', b'n', b'g',
    b';',
    b'A', b's', b'c', b'r', b';',
    b'A', b's', b's', b'i', b'g', b'n', b';',
    b'A', b't', b'i', b'l', b'd', b'e',
    b';',
    b'A', b'u', b'm', b'l',
    b';',
    b'B', b'a', b'c', b'k', b's', b'l', b'a', b's', b'h', b';',
    b'B', b'a', b'r', b'v', b';',
    b'B', b'a', b'r', b'w', b'e', b'd', b';',
    b'B', b'c', b'y', b';',
    b'B', b'e', b'c', b'a', b'u', b's', b'e', b';',
// cpp: html/parser/html_entity_table.cc:74-105
    b'B', b'e', b'r', b'n', b'o', b'u', b'l', b'l', b'i', b's', b';',
    b'B', b'e', b't', b'a', b';',
    b'B', b'f', b'r', b';',
    b'B', b'o', b'p', b'f', b';',
    b'B', b'r', b'e', b'v', b'e', b';',
    b'B', b's', b'c', b'r', b';',
    b'B', b'u', b'm', b'p', b'e', b'q', b';',
    b'C', b'H', b'c', b'y', b';',
    b'C', b'O', b'P', b'Y',
    b';',
    b'C', b'a', b'c', b'u', b't', b'e', b';',
    b'C', b'a', b'p', b';',
    b'C', b'a', b'p', b'i', b't', b'a', b'l', b'D', b'i', b'f', b'f', b'e', b'r', b'e', b'n', b't', b'i', b'a', b'l', b'D', b';',
    b'C', b'a', b'y', b'l', b'e', b'y', b's', b';',
    b'C', b'c', b'a', b'r', b'o', b'n', b';',
    b'C', b'c', b'e', b'd', b'i', b'l',
    b';',
    b'C', b'c', b'i', b'r', b'c', b';',
    b'C', b'c', b'o', b'n', b'i', b'n', b't', b';',
    b'C', b'd', b'o', b't', b';',
    b'C', b'e', b'd', b'i', b'l', b'l', b'a', b';',
    b'C', b'e', b'n', b't', b'e', b'r', b'D', b'o', b't', b';',
    b'C', b'f', b'r', b';',
    b'C', b'h', b'i', b';',
    b'C', b'i', b'r', b'c', b'l', b'e', b'D', b'o', b't', b';',
    b'C', b'i', b'r', b'c', b'l', b'e', b'M', b'i', b'n', b'u', b's', b';',
    b'C', b'i', b'r', b'c', b'l', b'e', b'P', b'l', b'u', b's', b';',
    b'C', b'i', b'r', b'c', b'l', b'e', b'T', b'i', b'm', b'e', b's', b';',
    b'C', b'l', b'o', b'c', b'k', b'w', b'i', b's', b'e', b'C', b'o', b'n', b't', b'o', b'u', b'r', b'I', b'n', b't', b'e', b'g', b'r', b'a', b'l', b';',
    b'C', b'l', b'o', b's', b'e', b'C', b'u', b'r', b'l', b'y', b'D', b'o', b'u', b'b', b'l', b'e', b'Q', b'u', b'o', b't', b'e', b';',
    b'C', b'l', b'o', b's', b'e', b'C', b'u', b'r', b'l', b'y', b'Q', b'u', b'o', b't', b'e', b';',
    b'C', b'o', b'l', b'o', b'n', b';',
// cpp: html/parser/html_entity_table.cc:106-137
    b'C', b'o', b'l', b'o', b'n', b'e', b';',
    b'C', b'o', b'n', b'g', b'r', b'u', b'e', b'n', b't', b';',
    b'C', b'o', b'n', b'i', b'n', b't', b';',
    b'C', b'o', b'p', b'f', b';',
    b'C', b'o', b'p', b'r', b'o', b'd', b'u', b'c', b't', b';',
    b'C', b'o', b'u', b'n', b't', b'e', b'r', b'C', b'l', b'o', b'c', b'k', b'w', b'i', b's', b'e', b'C', b'o', b'n', b't', b'o', b'u', b'r', b'I', b'n', b't', b'e', b'g', b'r', b'a', b'l', b';',
    b'C', b'r', b'o', b's', b's', b';',
    b'C', b's', b'c', b'r', b';',
    b'C', b'u', b'p', b';',
    b'C', b'u', b'p', b'C', b'a', b'p', b';',
    b'D', b'D', b';',
    b'D', b'D', b'o', b't', b'r', b'a', b'h', b'd', b';',
    b'D', b'J', b'c', b'y', b';',
    b'D', b'S', b'c', b'y', b';',
    b'D', b'Z', b'c', b'y', b';',
    b'D', b'a', b'g', b'g', b'e', b'r', b';',
    b'D', b'a', b'r', b'r', b';',
    b'D', b'a', b's', b'h', b'v', b';',
    b'D', b'c', b'a', b'r', b'o', b'n', b';',
    b'D', b'c', b'y', b';',
    b'D', b'e', b'l', b';',
    b'D', b'e', b'l', b't', b'a', b';',
    b'D', b'f', b'r', b';',
    b'D', b'i', b'a', b'c', b'r', b'i', b't', b'i', b'c', b'a', b'l', b'A', b'c', b'u', b't', b'e', b';',
    b'D', b'i', b'a', b'c', b'r', b'i', b't', b'i', b'c', b'a', b'l', b'D', b'o', b't', b';',
    b'D', b'i', b'a', b'c', b'r', b'i', b't', b'i', b'c', b'a', b'l', b'D', b'o', b'u', b'b', b'l', b'e', b'A', b'c', b'u', b't', b'e', b';',
    b'D', b'i', b'a', b'c', b'r', b'i', b't', b'i', b'c', b'a', b'l', b'G', b'r', b'a', b'v', b'e', b';',
    b'D', b'i', b'a', b'c', b'r', b'i', b't', b'i', b'c', b'a', b'l', b'T', b'i', b'l', b'd', b'e', b';',
    b'D', b'i', b'a', b'm', b'o', b'n', b'd', b';',
    b'D', b'o', b'p', b'f', b';',
    b'D', b'o', b't', b'D', b'o', b't', b';',
    b'D', b'o', b't', b'E', b'q', b'u', b'a', b'l', b';',
// cpp: html/parser/html_entity_table.cc:138-169
    b'D', b'o', b'u', b'b', b'l', b'e', b'C', b'o', b'n', b't', b'o', b'u', b'r', b'I', b'n', b't', b'e', b'g', b'r', b'a', b'l', b';',
    b'D', b'o', b'u', b'b', b'l', b'e', b'D', b'o', b't', b';',
    b'D', b'o', b'u', b'b', b'l', b'e', b'D', b'o', b'w', b'n', b'A', b'r', b'r', b'o', b'w', b';',
    b'D', b'o', b'u', b'b', b'l', b'e', b'L', b'e', b'f', b't', b'A', b'r', b'r', b'o', b'w', b';',
    b'D', b'o', b'u', b'b', b'l', b'e', b'L', b'e', b'f', b't', b'R', b'i', b'g', b'h', b't', b'A', b'r', b'r', b'o', b'w', b';',
    b'D', b'o', b'u', b'b', b'l', b'e', b'L', b'e', b'f', b't', b'T', b'e', b'e', b';',
    b'D', b'o', b'u', b'b', b'l', b'e', b'L', b'o', b'n', b'g', b'L', b'e', b'f', b't', b'A', b'r', b'r', b'o', b'w', b';',
    b'D', b'o', b'u', b'b', b'l', b'e', b'L', b'o', b'n', b'g', b'L', b'e', b'f', b't', b'R', b'i', b'g', b'h', b't', b'A', b'r', b'r', b'o', b'w', b';',
    b'D', b'o', b'u', b'b', b'l', b'e', b'L', b'o', b'n', b'g', b'R', b'i', b'g', b'h', b't', b'A', b'r', b'r', b'o', b'w', b';',
    b'D', b'o', b'u', b'b', b'l', b'e', b'R', b'i', b'g', b'h', b't', b'A', b'r', b'r', b'o', b'w', b';',
    b'D', b'o', b'u', b'b', b'l', b'e', b'R', b'i', b'g', b'h', b't', b'T', b'e', b'e', b';',
    b'D', b'o', b'u', b'b', b'l', b'e', b'U', b'p', b'A', b'r', b'r', b'o', b'w', b';',
    b'D', b'o', b'u', b'b', b'l', b'e', b'U', b'p', b'D', b'o', b'w', b'n', b'A', b'r', b'r', b'o', b'w', b';',
    b'D', b'o', b'u', b'b', b'l', b'e', b'V', b'e', b'r', b't', b'i', b'c', b'a', b'l', b'B', b'a', b'r', b';',
    b'D', b'o', b'w', b'n', b'A', b'r', b'r', b'o', b'w', b'B', b'a', b'r', b';',
    b'D', b'o', b'w', b'n', b'A', b'r', b'r', b'o', b'w', b'U', b'p', b'A', b'r', b'r', b'o', b'w', b';',
    b'D', b'o', b'w', b'n', b'B', b'r', b'e', b'v', b'e', b';',
    b'D', b'o', b'w', b'n', b'L', b'e', b'f', b't', b'R', b'i', b'g', b'h', b't', b'V', b'e', b'c', b't', b'o', b'r', b';',
    b'D', b'o', b'w', b'n', b'L', b'e', b'f', b't', b'T', b'e', b'e', b'V', b'e', b'c', b't', b'o', b'r', b';',
    b'D', b'o', b'w', b'n', b'L', b'e', b'f', b't', b'V', b'e', b'c', b't', b'o', b'r', b';',
    b'D', b'o', b'w', b'n', b'L', b'e', b'f', b't', b'V', b'e', b'c', b't', b'o', b'r', b'B', b'a', b'r', b';',
    b'D', b'o', b'w', b'n', b'R', b'i', b'g', b'h', b't', b'T', b'e', b'e', b'V', b'e', b'c', b't', b'o', b'r', b';',
    b'D', b'o', b'w', b'n', b'R', b'i', b'g', b'h', b't', b'V', b'e', b'c', b't', b'o', b'r', b';',
    b'D', b'o', b'w', b'n', b'R', b'i', b'g', b'h', b't', b'V', b'e', b'c', b't', b'o', b'r', b'B', b'a', b'r', b';',
    b'D', b'o', b'w', b'n', b'T', b'e', b'e', b';',
    b'D', b'o', b'w', b'n', b'T', b'e', b'e', b'A', b'r', b'r', b'o', b'w', b';',
    b'D', b'o', b'w', b'n', b'a', b'r', b'r', b'o', b'w', b';',
    b'D', b's', b'c', b'r', b';',
    b'D', b's', b't', b'r', b'o', b'k', b';',
    b'E', b'N', b'G', b';',
    b'E', b'T', b'H',
    b';',
// cpp: html/parser/html_entity_table.cc:170-201
    b'E', b'a', b'c', b'u', b't', b'e',
    b';',
    b'E', b'c', b'a', b'r', b'o', b'n', b';',
    b'E', b'c', b'i', b'r', b'c',
    b';',
    b'E', b'c', b'y', b';',
    b'E', b'd', b'o', b't', b';',
    b'E', b'f', b'r', b';',
    b'E', b'g', b'r', b'a', b'v', b'e',
    b';',
    b'E', b'l', b'e', b'm', b'e', b'n', b't', b';',
    b'E', b'm', b'a', b'c', b'r', b';',
    b'E', b'm', b'p', b't', b'y', b'S', b'm', b'a', b'l', b'l', b'S', b'q', b'u', b'a', b'r', b'e', b';',
    b'E', b'm', b'p', b't', b'y', b'V', b'e', b'r', b'y', b'S', b'm', b'a', b'l', b'l', b'S', b'q', b'u', b'a', b'r', b'e', b';',
    b'E', b'o', b'g', b'o', b'n', b';',
    b'E', b'o', b'p', b'f', b';',
    b'E', b'p', b's', b'i', b'l', b'o', b'n', b';',
    b'E', b'q', b'u', b'a', b'l', b'T', b'i', b'l', b'd', b'e', b';',
    b'E', b'q', b'u', b'i', b'l', b'i', b'b', b'r', b'i', b'u', b'm', b';',
    b'E', b's', b'c', b'r', b';',
    b'E', b's', b'i', b'm', b';',
    b'E', b't', b'a', b';',
    b'E', b'u', b'm', b'l',
    b';',
    b'E', b'x', b'i', b's', b't', b's', b';',
    b'E', b'x', b'p', b'o', b'n', b'e', b'n', b't', b'i', b'a', b'l', b'E', b';',
    b'F', b'c', b'y', b';',
    b'F', b'f', b'r', b';',
    b'F', b'i', b'l', b'l', b'e', b'd', b'S', b'm', b'a', b'l', b'l', b'S', b'q', b'u', b'a', b'r', b'e', b';',
    b'F', b'i', b'l', b'l', b'e', b'd', b'V', b'e', b'r', b'y', b'S', b'm', b'a', b'l', b'l', b'S', b'q', b'u', b'a', b'r', b'e', b';',
    b'F', b'o', b'p', b'f', b';',
    b'F', b'o', b'r', b'A', b'l', b'l', b';',
// cpp: html/parser/html_entity_table.cc:202-233
    b'F', b'o', b'u', b'r', b'i', b'e', b'r', b't', b'r', b'f', b';',
    b'F', b's', b'c', b'r', b';',
    b'G', b'J', b'c', b'y', b';',
    b'G', b'T',
    b';',
    b'G', b'a', b'm', b'm', b'a', b';',
    b'G', b'a', b'm', b'm', b'a', b'd', b';',
    b'G', b'b', b'r', b'e', b'v', b'e', b';',
    b'G', b'c', b'e', b'd', b'i', b'l', b';',
    b'G', b'c', b'i', b'r', b'c', b';',
    b'G', b'c', b'y', b';',
    b'G', b'd', b'o', b't', b';',
    b'G', b'f', b'r', b';',
    b'G', b'g', b';',
    b'G', b'o', b'p', b'f', b';',
    b'G', b'r', b'e', b'a', b't', b'e', b'r', b'E', b'q', b'u', b'a', b'l', b';',
    b'G', b'r', b'e', b'a', b't', b'e', b'r', b'E', b'q', b'u', b'a', b'l', b'L', b'e', b's', b's', b';',
    b'G', b'r', b'e', b'a', b't', b'e', b'r', b'F', b'u', b'l', b'l', b'E', b'q', b'u', b'a', b'l', b';',
    b'G', b'r', b'e', b'a', b't', b'e', b'r', b'G', b'r', b'e', b'a', b't', b'e', b'r', b';',
    b'G', b'r', b'e', b'a', b't', b'e', b'r', b'L', b'e', b's', b's', b';',
    b'G', b'r', b'e', b'a', b't', b'e', b'r', b'S', b'l', b'a', b'n', b't', b'E', b'q', b'u', b'a', b'l', b';',
    b'G', b'r', b'e', b'a', b't', b'e', b'r', b'T', b'i', b'l', b'd', b'e', b';',
    b'G', b's', b'c', b'r', b';',
    b'G', b't', b';',
    b'H', b'A', b'R', b'D', b'c', b'y', b';',
    b'H', b'a', b'c', b'e', b'k', b';',
    b'H', b'a', b't', b';',
    b'H', b'c', b'i', b'r', b'c', b';',
    b'H', b'f', b'r', b';',
    b'H', b'i', b'l', b'b', b'e', b'r', b't', b'S', b'p', b'a', b'c', b'e', b';',
    b'H', b'o', b'p', b'f', b';',
    b'H', b'o', b'r', b'i', b'z', b'o', b'n', b't', b'a', b'l', b'L', b'i', b'n', b'e', b';',
// cpp: html/parser/html_entity_table.cc:234-265
    b'H', b's', b'c', b'r', b';',
    b'H', b's', b't', b'r', b'o', b'k', b';',
    b'H', b'u', b'm', b'p', b'D', b'o', b'w', b'n', b'H', b'u', b'm', b'p', b';',
    b'H', b'u', b'm', b'p', b'E', b'q', b'u', b'a', b'l', b';',
    b'I', b'E', b'c', b'y', b';',
    b'I', b'J', b'l', b'i', b'g', b';',
    b'I', b'O', b'c', b'y', b';',
    b'I', b'a', b'c', b'u', b't', b'e',
    b';',
    b'I', b'c', b'i', b'r', b'c',
    b';',
    b'I', b'c', b'y', b';',
    b'I', b'd', b'o', b't', b';',
    b'I', b'f', b'r', b';',
    b'I', b'g', b'r', b'a', b'v', b'e',
    b';',
    b'I', b'm', b';',
    b'I', b'm', b'a', b'c', b'r', b';',
    b'I', b'm', b'a', b'g', b'i', b'n', b'a', b'r', b'y', b'I', b';',
    b'I', b'm', b'p', b'l', b'i', b'e', b's', b';',
    b'I', b'n', b't', b';',
    b'I', b'n', b't', b'e', b'r', b's', b'e', b'c', b't', b'i', b'o', b'n', b';',
    b'I', b'n', b'v', b'i', b's', b'i', b'b', b'l', b'e', b'C', b'o', b'm', b'm', b'a', b';',
    b'I', b'n', b'v', b'i', b's', b'i', b'b', b'l', b'e', b'T', b'i', b'm', b'e', b's', b';',
    b'I', b'o', b'g', b'o', b'n', b';',
    b'I', b'o', b'p', b'f', b';',
    b'I', b'o', b't', b'a', b';',
    b'I', b's', b'c', b'r', b';',
    b'I', b't', b'i', b'l', b'd', b'e', b';',
    b'I', b'u', b'k', b'c', b'y', b';',
    b'I', b'u', b'm', b'l',
    b';',
// cpp: html/parser/html_entity_table.cc:266-297
    b'J', b'c', b'i', b'r', b'c', b';',
    b'J', b'f', b'r', b';',
    b'J', b'o', b'p', b'f', b';',
    b'J', b's', b'c', b'r', b';',
    b'J', b's', b'e', b'r', b'c', b'y', b';',
    b'J', b'u', b'k', b'c', b'y', b';',
    b'K', b'H', b'c', b'y', b';',
    b'K', b'J', b'c', b'y', b';',
    b'K', b'a', b'p', b'p', b'a', b';',
    b'K', b'c', b'e', b'd', b'i', b'l', b';',
    b'K', b'c', b'y', b';',
    b'K', b'f', b'r', b';',
    b'K', b'o', b'p', b'f', b';',
    b'K', b's', b'c', b'r', b';',
    b'L', b'J', b'c', b'y', b';',
    b'L', b'T',
    b';',
    b'L', b'a', b'c', b'u', b't', b'e', b';',
    b'L', b'a', b'm', b'b', b'd', b'a', b';',
    b'L', b'a', b'n', b'g', b';',
    b'L', b'a', b'p', b'l', b'a', b'c', b'e', b't', b'r', b'f', b';',
    b'L', b'a', b'r', b'r', b';',
    b'L', b'c', b'a', b'r', b'o', b'n', b';',
    b'L', b'c', b'e', b'd', b'i', b'l', b';',
    b'L', b'c', b'y', b';',
    b'L', b'e', b'f', b't', b'A', b'n', b'g', b'l', b'e', b'B', b'r', b'a', b'c', b'k', b'e', b't', b';',
    b'L', b'e', b'f', b't', b'A', b'r', b'r', b'o', b'w', b'B', b'a', b'r', b';',
    b'L', b'e', b'f', b't', b'A', b'r', b'r', b'o', b'w', b'R', b'i', b'g', b'h', b't', b'A', b'r', b'r', b'o', b'w', b';',
    b'L', b'e', b'f', b't', b'C', b'e', b'i', b'l', b'i', b'n', b'g', b';',
    b'L', b'e', b'f', b't', b'D', b'o', b'u', b'b', b'l', b'e', b'B', b'r', b'a', b'c', b'k', b'e', b't', b';',
    b'L', b'e', b'f', b't', b'D', b'o', b'w', b'n', b'T', b'e', b'e', b'V', b'e', b'c', b't', b'o', b'r', b';',
    b'L', b'e', b'f', b't', b'D', b'o', b'w', b'n', b'V', b'e', b'c', b't', b'o', b'r', b';',
// cpp: html/parser/html_entity_table.cc:298-329
    b'L', b'e', b'f', b't', b'D', b'o', b'w', b'n', b'V', b'e', b'c', b't', b'o', b'r', b'B', b'a', b'r', b';',
    b'L', b'e', b'f', b't', b'F', b'l', b'o', b'o', b'r', b';',
    b'L', b'e', b'f', b't', b'T', b'e', b'e', b'A', b'r', b'r', b'o', b'w', b';',
    b'L', b'e', b'f', b't', b'T', b'r', b'i', b'a', b'n', b'g', b'l', b'e', b';',
    b'L', b'e', b'f', b't', b'T', b'r', b'i', b'a', b'n', b'g', b'l', b'e', b'B', b'a', b'r', b';',
    b'L', b'e', b'f', b't', b'T', b'r', b'i', b'a', b'n', b'g', b'l', b'e', b'E', b'q', b'u', b'a', b'l', b';',
    b'L', b'e', b'f', b't', b'U', b'p', b'D', b'o', b'w', b'n', b'V', b'e', b'c', b't', b'o', b'r', b';',
    b'L', b'e', b'f', b't', b'U', b'p', b'T', b'e', b'e', b'V', b'e', b'c', b't', b'o', b'r', b';',
    b'L', b'e', b'f', b't', b'U', b'p', b'V', b'e', b'c', b't', b'o', b'r', b';',
    b'L', b'e', b'f', b't', b'U', b'p', b'V', b'e', b'c', b't', b'o', b'r', b'B', b'a', b'r', b';',
    b'L', b'e', b'f', b't', b'a', b'r', b'r', b'o', b'w', b';',
    b'L', b'e', b'f', b't', b'r', b'i', b'g', b'h', b't', b'a', b'r', b'r', b'o', b'w', b';',
    b'L', b'e', b's', b's', b'E', b'q', b'u', b'a', b'l', b'G', b'r', b'e', b'a', b't', b'e', b'r', b';',
    b'L', b'e', b's', b's', b'F', b'u', b'l', b'l', b'E', b'q', b'u', b'a', b'l', b';',
    b'L', b'e', b's', b's', b'G', b'r', b'e', b'a', b't', b'e', b'r', b';',
    b'L', b'e', b's', b's', b'L', b'e', b's', b's', b';',
    b'L', b'e', b's', b's', b'S', b'l', b'a', b'n', b't', b'E', b'q', b'u', b'a', b'l', b';',
    b'L', b'e', b's', b's', b'T', b'i', b'l', b'd', b'e', b';',
    b'L', b'f', b'r', b';',
    b'L', b'l', b';',
    b'L', b'l', b'e', b'f', b't', b'a', b'r', b'r', b'o', b'w', b';',
    b'L', b'm', b'i', b'd', b'o', b't', b';',
    b'L', b'o', b'n', b'g', b'l', b'e', b'f', b't', b'a', b'r', b'r', b'o', b'w', b';',
    b'L', b'o', b'n', b'g', b'l', b'e', b'f', b't', b'r', b'i', b'g', b'h', b't', b'a', b'r', b'r', b'o', b'w', b';',
    b'L', b'o', b'n', b'g', b'r', b'i', b'g', b'h', b't', b'a', b'r', b'r', b'o', b'w', b';',
    b'L', b'o', b'p', b'f', b';',
    b'L', b'o', b'w', b'e', b'r', b'L', b'e', b'f', b't', b'A', b'r', b'r', b'o', b'w', b';',
    b'L', b'o', b'w', b'e', b'r', b'R', b'i', b'g', b'h', b't', b'A', b'r', b'r', b'o', b'w', b';',
    b'L', b's', b'c', b'r', b';',
    b'L', b's', b'h', b';',
    b'L', b's', b't', b'r', b'o', b'k', b';',
    b'L', b't', b';',
// cpp: html/parser/html_entity_table.cc:330-361
    b'M', b'a', b'p', b';',
    b'M', b'c', b'y', b';',
    b'M', b'e', b'd', b'i', b'u', b'm', b'S', b'p', b'a', b'c', b'e', b';',
    b'M', b'e', b'l', b'l', b'i', b'n', b't', b'r', b'f', b';',
    b'M', b'f', b'r', b';',
    b'M', b'i', b'n', b'u', b's', b'P', b'l', b'u', b's', b';',
    b'M', b'o', b'p', b'f', b';',
    b'M', b's', b'c', b'r', b';',
    b'M', b'u', b';',
    b'N', b'J', b'c', b'y', b';',
    b'N', b'a', b'c', b'u', b't', b'e', b';',
    b'N', b'c', b'a', b'r', b'o', b'n', b';',
    b'N', b'c', b'e', b'd', b'i', b'l', b';',
    b'N', b'c', b'y', b';',
    b'N', b'e', b'g', b'a', b't', b'i', b'v', b'e', b'M', b'e', b'd', b'i', b'u', b'm', b'S', b'p', b'a', b'c', b'e', b';',
    b'N', b'e', b'g', b'a', b't', b'i', b'v', b'e', b'T', b'h', b'i', b'c', b'k', b'S', b'p', b'a', b'c', b'e', b';',
    b'N', b'e', b'g', b'a', b't', b'i', b'v', b'e', b'T', b'h', b'i', b'n', b'S', b'p', b'a', b'c', b'e', b';',
    b'N', b'e', b'g', b'a', b't', b'i', b'v', b'e', b'V', b'e', b'r', b'y', b'T', b'h', b'i', b'n', b'S', b'p', b'a', b'c', b'e', b';',
    b'N', b'e', b's', b't', b'e', b'd', b'G', b'r', b'e', b'a', b't', b'e', b'r', b'G', b'r', b'e', b'a', b't', b'e', b'r', b';',
    b'N', b'e', b's', b't', b'e', b'd', b'L', b'e', b's', b's', b'L', b'e', b's', b's', b';',
    b'N', b'e', b'w', b'L', b'i', b'n', b'e', b';',
    b'N', b'f', b'r', b';',
    b'N', b'o', b'B', b'r', b'e', b'a', b'k', b';',
    b'N', b'o', b'n', b'B', b'r', b'e', b'a', b'k', b'i', b'n', b'g', b'S', b'p', b'a', b'c', b'e', b';',
    b'N', b'o', b'p', b'f', b';',
    b'N', b'o', b't', b';',
    b'N', b'o', b't', b'C', b'o', b'n', b'g', b'r', b'u', b'e', b'n', b't', b';',
    b'N', b'o', b't', b'C', b'u', b'p', b'C', b'a', b'p', b';',
    b'N', b'o', b't', b'D', b'o', b'u', b'b', b'l', b'e', b'V', b'e', b'r', b't', b'i', b'c', b'a', b'l', b'B', b'a', b'r', b';',
    b'N', b'o', b't', b'E', b'l', b'e', b'm', b'e', b'n', b't', b';',
    b'N', b'o', b't', b'E', b'q', b'u', b'a', b'l', b';',
    b'N', b'o', b't', b'E', b'q', b'u', b'a', b'l', b'T', b'i', b'l', b'd', b'e', b';',
// cpp: html/parser/html_entity_table.cc:362-393
    b'N', b'o', b't', b'E', b'x', b'i', b's', b't', b's', b';',
    b'N', b'o', b't', b'G', b'r', b'e', b'a', b't', b'e', b'r', b';',
    b'N', b'o', b't', b'G', b'r', b'e', b'a', b't', b'e', b'r', b'E', b'q', b'u', b'a', b'l', b';',
    b'N', b'o', b't', b'G', b'r', b'e', b'a', b't', b'e', b'r', b'F', b'u', b'l', b'l', b'E', b'q', b'u', b'a', b'l', b';',
    b'N', b'o', b't', b'G', b'r', b'e', b'a', b't', b'e', b'r', b'G', b'r', b'e', b'a', b't', b'e', b'r', b';',
    b'N', b'o', b't', b'G', b'r', b'e', b'a', b't', b'e', b'r', b'L', b'e', b's', b's', b';',
    b'N', b'o', b't', b'G', b'r', b'e', b'a', b't', b'e', b'r', b'S', b'l', b'a', b'n', b't', b'E', b'q', b'u', b'a', b'l', b';',
    b'N', b'o', b't', b'G', b'r', b'e', b'a', b't', b'e', b'r', b'T', b'i', b'l', b'd', b'e', b';',
    b'N', b'o', b't', b'H', b'u', b'm', b'p', b'D', b'o', b'w', b'n', b'H', b'u', b'm', b'p', b';',
    b'N', b'o', b't', b'H', b'u', b'm', b'p', b'E', b'q', b'u', b'a', b'l', b';',
    b'N', b'o', b't', b'L', b'e', b'f', b't', b'T', b'r', b'i', b'a', b'n', b'g', b'l', b'e', b';',
    b'N', b'o', b't', b'L', b'e', b'f', b't', b'T', b'r', b'i', b'a', b'n', b'g', b'l', b'e', b'B', b'a', b'r', b';',
    b'N', b'o', b't', b'L', b'e', b'f', b't', b'T', b'r', b'i', b'a', b'n', b'g', b'l', b'e', b'E', b'q', b'u', b'a', b'l', b';',
    b'N', b'o', b't', b'L', b'e', b's', b's', b';',
    b'N', b'o', b't', b'L', b'e', b's', b's', b'E', b'q', b'u', b'a', b'l', b';',
    b'N', b'o', b't', b'L', b'e', b's', b's', b'G', b'r', b'e', b'a', b't', b'e', b'r', b';',
    b'N', b'o', b't', b'L', b'e', b's', b's', b'L', b'e', b's', b's', b';',
    b'N', b'o', b't', b'L', b'e', b's', b's', b'S', b'l', b'a', b'n', b't', b'E', b'q', b'u', b'a', b'l', b';',
    b'N', b'o', b't', b'L', b'e', b's', b's', b'T', b'i', b'l', b'd', b'e', b';',
    b'N', b'o', b't', b'N', b'e', b's', b't', b'e', b'd', b'G', b'r', b'e', b'a', b't', b'e', b'r', b'G', b'r', b'e', b'a', b't', b'e', b'r', b';',
    b'N', b'o', b't', b'N', b'e', b's', b't', b'e', b'd', b'L', b'e', b's', b's', b'L', b'e', b's', b's', b';',
    b'N', b'o', b't', b'P', b'r', b'e', b'c', b'e', b'd', b'e', b's', b';',
    b'N', b'o', b't', b'P', b'r', b'e', b'c', b'e', b'd', b'e', b's', b'E', b'q', b'u', b'a', b'l', b';',
    b'N', b'o', b't', b'P', b'r', b'e', b'c', b'e', b'd', b'e', b's', b'S', b'l', b'a', b'n', b't', b'E', b'q', b'u', b'a', b'l', b';',
    b'N', b'o', b't', b'R', b'e', b'v', b'e', b'r', b's', b'e', b'E', b'l', b'e', b'm', b'e', b'n', b't', b';',
    b'N', b'o', b't', b'R', b'i', b'g', b'h', b't', b'T', b'r', b'i', b'a', b'n', b'g', b'l', b'e', b';',
    b'N', b'o', b't', b'R', b'i', b'g', b'h', b't', b'T', b'r', b'i', b'a', b'n', b'g', b'l', b'e', b'B', b'a', b'r', b';',
    b'N', b'o', b't', b'R', b'i', b'g', b'h', b't', b'T', b'r', b'i', b'a', b'n', b'g', b'l', b'e', b'E', b'q', b'u', b'a', b'l', b';',
    b'N', b'o', b't', b'S', b'q', b'u', b'a', b'r', b'e', b'S', b'u', b'b', b's', b'e', b't', b';',
    b'N', b'o', b't', b'S', b'q', b'u', b'a', b'r', b'e', b'S', b'u', b'b', b's', b'e', b't', b'E', b'q', b'u', b'a', b'l', b';',
    b'N', b'o', b't', b'S', b'q', b'u', b'a', b'r', b'e', b'S', b'u', b'p', b'e', b'r', b's', b'e', b't', b';',
    b'N', b'o', b't', b'S', b'q', b'u', b'a', b'r', b'e', b'S', b'u', b'p', b'e', b'r', b's', b'e', b't', b'E', b'q', b'u', b'a', b'l', b';',
// cpp: html/parser/html_entity_table.cc:394-425
    b'N', b'o', b't', b'S', b'u', b'b', b's', b'e', b't', b';',
    b'N', b'o', b't', b'S', b'u', b'b', b's', b'e', b't', b'E', b'q', b'u', b'a', b'l', b';',
    b'N', b'o', b't', b'S', b'u', b'c', b'c', b'e', b'e', b'd', b's', b';',
    b'N', b'o', b't', b'S', b'u', b'c', b'c', b'e', b'e', b'd', b's', b'E', b'q', b'u', b'a', b'l', b';',
    b'N', b'o', b't', b'S', b'u', b'c', b'c', b'e', b'e', b'd', b's', b'S', b'l', b'a', b'n', b't', b'E', b'q', b'u', b'a', b'l', b';',
    b'N', b'o', b't', b'S', b'u', b'c', b'c', b'e', b'e', b'd', b's', b'T', b'i', b'l', b'd', b'e', b';',
    b'N', b'o', b't', b'S', b'u', b'p', b'e', b'r', b's', b'e', b't', b';',
    b'N', b'o', b't', b'S', b'u', b'p', b'e', b'r', b's', b'e', b't', b'E', b'q', b'u', b'a', b'l', b';',
    b'N', b'o', b't', b'T', b'i', b'l', b'd', b'e', b';',
    b'N', b'o', b't', b'T', b'i', b'l', b'd', b'e', b'E', b'q', b'u', b'a', b'l', b';',
    b'N', b'o', b't', b'T', b'i', b'l', b'd', b'e', b'F', b'u', b'l', b'l', b'E', b'q', b'u', b'a', b'l', b';',
    b'N', b'o', b't', b'T', b'i', b'l', b'd', b'e', b'T', b'i', b'l', b'd', b'e', b';',
    b'N', b'o', b't', b'V', b'e', b'r', b't', b'i', b'c', b'a', b'l', b'B', b'a', b'r', b';',
    b'N', b's', b'c', b'r', b';',
    b'N', b't', b'i', b'l', b'd', b'e',
    b';',
    b'N', b'u', b';',
    b'O', b'E', b'l', b'i', b'g', b';',
    b'O', b'a', b'c', b'u', b't', b'e',
    b';',
    b'O', b'c', b'i', b'r', b'c',
    b';',
    b'O', b'd', b'b', b'l', b'a', b'c', b';',
    b'O', b'f', b'r', b';',
    b'O', b'g', b'r', b'a', b'v', b'e',
    b';',
    b'O', b'm', b'a', b'c', b'r', b';',
    b'O', b'm', b'e', b'g', b'a', b';',
    b'O', b'm', b'i', b'c', b'r', b'o', b'n', b';',
    b'O', b'o', b'p', b'f', b';',
    b'O', b'p', b'e', b'n', b'C', b'u', b'r', b'l', b'y', b'D', b'o', b'u', b'b', b'l', b'e', b'Q', b'u', b'o', b't', b'e', b';',
    b'O', b'p', b'e', b'n', b'C', b'u', b'r', b'l', b'y', b'Q', b'u', b'o', b't', b'e', b';',
// cpp: html/parser/html_entity_table.cc:426-457
    b'O', b'r', b';',
    b'O', b's', b'c', b'r', b';',
    b'O', b's', b'l', b'a', b's', b'h',
    b';',
    b'O', b't', b'i', b'l', b'd', b'e',
    b';',
    b'O', b't', b'i', b'm', b'e', b's', b';',
    b'O', b'u', b'm', b'l',
    b';',
    b'O', b'v', b'e', b'r', b'B', b'a', b'r', b';',
    b'O', b'v', b'e', b'r', b'B', b'r', b'a', b'c', b'e', b';',
    b'O', b'v', b'e', b'r', b'B', b'r', b'a', b'c', b'k', b'e', b't', b';',
    b'O', b'v', b'e', b'r', b'P', b'a', b'r', b'e', b'n', b't', b'h', b'e', b's', b'i', b's', b';',
    b'P', b'a', b'r', b't', b'i', b'a', b'l', b'D', b';',
    b'P', b'c', b'y', b';',
    b'P', b'f', b'r', b';',
    b'P', b'h', b'i', b';',
    b'P', b'i', b';',
    b'P', b'l', b'u', b's', b'M', b'i', b'n', b'u', b's', b';',
    b'P', b'o', b'i', b'n', b'c', b'a', b'r', b'e', b'p', b'l', b'a', b'n', b'e', b';',
    b'P', b'o', b'p', b'f', b';',
    b'P', b'r', b';',
    b'P', b'r', b'e', b'c', b'e', b'd', b'e', b's', b'T', b'i', b'l', b'd', b'e', b';',
    b'P', b'r', b'i', b'm', b'e', b';',
    b'P', b'r', b'o', b'd', b'u', b'c', b't', b';',
    b'P', b'r', b'o', b'p', b'o', b'r', b't', b'i', b'o', b'n', b';',
    b'P', b'r', b'o', b'p', b'o', b'r', b't', b'i', b'o', b'n', b'a', b'l', b';',
    b'P', b's', b'c', b'r', b';',
    b'P', b's', b'i', b';',
    b'Q', b'U', b'O', b'T',
    b';',
    b'Q', b'f', b'r', b';',
// cpp: html/parser/html_entity_table.cc:458-489
    b'Q', b'o', b'p', b'f', b';',
    b'Q', b's', b'c', b'r', b';',
    b'R', b'B', b'a', b'r', b'r', b';',
    b'R', b'E', b'G',
    b';',
    b'R', b'a', b'c', b'u', b't', b'e', b';',
    b'R', b'a', b'n', b'g', b';',
    b'R', b'a', b'r', b'r', b';',
    b'R', b'a', b'r', b'r', b't', b'l', b';',
    b'R', b'c', b'a', b'r', b'o', b'n', b';',
    b'R', b'c', b'e', b'd', b'i', b'l', b';',
    b'R', b'c', b'y', b';',
    b'R', b'e', b';',
    b'R', b'e', b'v', b'e', b'r', b's', b'e', b'E', b'q', b'u', b'i', b'l', b'i', b'b', b'r', b'i', b'u', b'm', b';',
    b'R', b'e', b'v', b'e', b'r', b's', b'e', b'U', b'p', b'E', b'q', b'u', b'i', b'l', b'i', b'b', b'r', b'i', b'u', b'm', b';',
    b'R', b'f', b'r', b';',
    b'R', b'h', b'o', b';',
    b'R', b'i', b'g', b'h', b't', b'A', b'n', b'g', b'l', b'e', b'B', b'r', b'a', b'c', b'k', b'e', b't', b';',
    b'R', b'i', b'g', b'h', b't', b'A', b'r', b'r', b'o', b'w', b'B', b'a', b'r', b';',
    b'R', b'i', b'g', b'h', b't', b'A', b'r', b'r', b'o', b'w', b'L', b'e', b'f', b't', b'A', b'r', b'r', b'o', b'w', b';',
    b'R', b'i', b'g', b'h', b't', b'C', b'e', b'i', b'l', b'i', b'n', b'g', b';',
    b'R', b'i', b'g', b'h', b't', b'D', b'o', b'u', b'b', b'l', b'e', b'B', b'r', b'a', b'c', b'k', b'e', b't', b';',
    b'R', b'i', b'g', b'h', b't', b'D', b'o', b'w', b'n', b'T', b'e', b'e', b'V', b'e', b'c', b't', b'o', b'r', b';',
    b'R', b'i', b'g', b'h', b't', b'D', b'o', b'w', b'n', b'V', b'e', b'c', b't', b'o', b'r', b';',
    b'R', b'i', b'g', b'h', b't', b'D', b'o', b'w', b'n', b'V', b'e', b'c', b't', b'o', b'r', b'B', b'a', b'r', b';',
    b'R', b'i', b'g', b'h', b't', b'F', b'l', b'o', b'o', b'r', b';',
    b'R', b'i', b'g', b'h', b't', b'T', b'e', b'e', b'A', b'r', b'r', b'o', b'w', b';',
    b'R', b'i', b'g', b'h', b't', b'U', b'p', b'D', b'o', b'w', b'n', b'V', b'e', b'c', b't', b'o', b'r', b';',
    b'R', b'i', b'g', b'h', b't', b'U', b'p', b'T', b'e', b'e', b'V', b'e', b'c', b't', b'o', b'r', b';',
    b'R', b'i', b'g', b'h', b't', b'U', b'p', b'V', b'e', b'c', b't', b'o', b'r', b';',
    b'R', b'i', b'g', b'h', b't', b'U', b'p', b'V', b'e', b'c', b't', b'o', b'r', b'B', b'a', b'r', b';',
    b'R', b'i', b'g', b'h', b't', b'a', b'r', b'r', b'o', b'w', b';',
// cpp: html/parser/html_entity_table.cc:490-521
    b'R', b'o', b'p', b'f', b';',
    b'R', b'o', b'u', b'n', b'd', b'I', b'm', b'p', b'l', b'i', b'e', b's', b';',
    b'R', b'r', b'i', b'g', b'h', b't', b'a', b'r', b'r', b'o', b'w', b';',
    b'R', b's', b'c', b'r', b';',
    b'R', b's', b'h', b';',
    b'R', b'u', b'l', b'e', b'D', b'e', b'l', b'a', b'y', b'e', b'd', b';',
    b'S', b'H', b'C', b'H', b'c', b'y', b';',
    b'S', b'H', b'c', b'y', b';',
    b'S', b'O', b'F', b'T', b'c', b'y', b';',
    b'S', b'a', b'c', b'u', b't', b'e', b';',
    b'S', b'c', b';',
    b'S', b'c', b'a', b'r', b'o', b'n', b';',
    b'S', b'c', b'e', b'd', b'i', b'l', b';',
    b'S', b'c', b'i', b'r', b'c', b';',
    b'S', b'f', b'r', b';',
    b'S', b'h', b'o', b'r', b't', b'D', b'o', b'w', b'n', b'A', b'r', b'r', b'o', b'w', b';',
    b'S', b'h', b'o', b'r', b't', b'L', b'e', b'f', b't', b'A', b'r', b'r', b'o', b'w', b';',
    b'S', b'h', b'o', b'r', b't', b'R', b'i', b'g', b'h', b't', b'A', b'r', b'r', b'o', b'w', b';',
    b'S', b'h', b'o', b'r', b't', b'U', b'p', b'A', b'r', b'r', b'o', b'w', b';',
    b'S', b'i', b'g', b'm', b'a', b';',
    b'S', b'm', b'a', b'l', b'l', b'C', b'i', b'r', b'c', b'l', b'e', b';',
    b'S', b'o', b'p', b'f', b';',
    b'S', b'q', b'r', b't', b';',
    b'S', b'q', b'u', b'a', b'r', b'e', b'I', b'n', b't', b'e', b'r', b's', b'e', b'c', b't', b'i', b'o', b'n', b';',
    b'S', b'q', b'u', b'a', b'r', b'e', b'U', b'n', b'i', b'o', b'n', b';',
    b'S', b's', b'c', b'r', b';',
    b'S', b't', b'a', b'r', b';',
    b'S', b'u', b'b', b';',
    b'S', b'u', b'c', b'h', b'T', b'h', b'a', b't', b';',
    b'S', b'u', b'm', b';',
    b'S', b'u', b'p', b';',
    b'S', b'u', b'p', b's', b'e', b't', b';',
// cpp: html/parser/html_entity_table.cc:522-553
    b'T', b'H', b'O', b'R', b'N',
    b';',
    b'T', b'R', b'A', b'D', b'E', b';',
    b'T', b'S', b'H', b'c', b'y', b';',
    b'T', b'S', b'c', b'y', b';',
    b'T', b'a', b'b', b';',
    b'T', b'a', b'u', b';',
    b'T', b'c', b'a', b'r', b'o', b'n', b';',
    b'T', b'c', b'e', b'd', b'i', b'l', b';',
    b'T', b'f', b'r', b';',
    b'T', b'h', b'e', b'r', b'e', b'f', b'o', b'r', b'e', b';',
    b'T', b'h', b'e', b't', b'a', b';',
    b'T', b'o', b'p', b'f', b';',
    b'T', b'r', b'i', b'p', b'l', b'e', b'D', b'o', b't', b';',
    b'T', b's', b'c', b'r', b';',
    b'T', b's', b't', b'r', b'o', b'k', b';',
    b'U', b'a', b'c', b'u', b't', b'e',
    b';',
    b'U', b'a', b'r', b'r', b';',
    b'U', b'a', b'r', b'r', b'o', b'c', b'i', b'r', b';',
    b'U', b'b', b'r', b'c', b'y', b';',
    b'U', b'b', b'r', b'e', b'v', b'e', b';',
    b'U', b'c', b'i', b'r', b'c',
    b';',
    b'U', b'c', b'y', b';',
    b'U', b'd', b'b', b'l', b'a', b'c', b';',
    b'U', b'f', b'r', b';',
    b'U', b'g', b'r', b'a', b'v', b'e',
    b';',
    b'U', b'm', b'a', b'c', b'r', b';',
    b'U', b'n', b'd', b'e', b'r', b'B', b'a', b'r', b';',
    b'U', b'n', b'd', b'e', b'r', b'B', b'r', b'a', b'c', b'e', b';',
// cpp: html/parser/html_entity_table.cc:554-585
    b'U', b'n', b'd', b'e', b'r', b'B', b'r', b'a', b'c', b'k', b'e', b't', b';',
    b'U', b'n', b'd', b'e', b'r', b'P', b'a', b'r', b'e', b'n', b't', b'h', b'e', b's', b'i', b's', b';',
    b'U', b'n', b'i', b'o', b'n', b'P', b'l', b'u', b's', b';',
    b'U', b'o', b'g', b'o', b'n', b';',
    b'U', b'o', b'p', b'f', b';',
    b'U', b'p', b'A', b'r', b'r', b'o', b'w', b'B', b'a', b'r', b';',
    b'U', b'p', b'A', b'r', b'r', b'o', b'w', b'D', b'o', b'w', b'n', b'A', b'r', b'r', b'o', b'w', b';',
    b'U', b'p', b'T', b'e', b'e', b';',
    b'U', b'p', b'T', b'e', b'e', b'A', b'r', b'r', b'o', b'w', b';',
    b'U', b'p', b'a', b'r', b'r', b'o', b'w', b';',
    b'U', b'p', b'd', b'o', b'w', b'n', b'a', b'r', b'r', b'o', b'w', b';',
    b'U', b'p', b'p', b'e', b'r', b'L', b'e', b'f', b't', b'A', b'r', b'r', b'o', b'w', b';',
    b'U', b'p', b'p', b'e', b'r', b'R', b'i', b'g', b'h', b't', b'A', b'r', b'r', b'o', b'w', b';',
    b'U', b'p', b's', b'i', b';',
    b'U', b'p', b's', b'i', b'l', b'o', b'n', b';',
    b'U', b'r', b'i', b'n', b'g', b';',
    b'U', b's', b'c', b'r', b';',
    b'U', b't', b'i', b'l', b'd', b'e', b';',
    b'U', b'u', b'm', b'l',
    b';',
    b'V', b'D', b'a', b's', b'h', b';',
    b'V', b'b', b'a', b'r', b';',
    b'V', b'c', b'y', b';',
    b'V', b'd', b'a', b's', b'h', b';',
    b'V', b'd', b'a', b's', b'h', b'l', b';',
    b'V', b'e', b'e', b';',
    b'V', b'e', b'r', b'b', b'a', b'r', b';',
    b'V', b'e', b'r', b't', b';',
    b'V', b'e', b'r', b't', b'i', b'c', b'a', b'l', b'L', b'i', b'n', b'e', b';',
    b'V', b'e', b'r', b't', b'i', b'c', b'a', b'l', b'S', b'e', b'p', b'a', b'r', b'a', b't', b'o', b'r', b';',
    b'V', b'e', b'r', b't', b'i', b'c', b'a', b'l', b'T', b'i', b'l', b'd', b'e', b';',
    b'V', b'f', b'r', b';',
// cpp: html/parser/html_entity_table.cc:586-617
    b'V', b'o', b'p', b'f', b';',
    b'V', b's', b'c', b'r', b';',
    b'V', b'v', b'd', b'a', b's', b'h', b';',
    b'W', b'c', b'i', b'r', b'c', b';',
    b'W', b'e', b'd', b'g', b'e', b';',
    b'W', b'f', b'r', b';',
    b'W', b'o', b'p', b'f', b';',
    b'W', b's', b'c', b'r', b';',
    b'X', b'f', b'r', b';',
    b'X', b'i', b';',
    b'X', b'o', b'p', b'f', b';',
    b'X', b's', b'c', b'r', b';',
    b'Y', b'A', b'c', b'y', b';',
    b'Y', b'I', b'c', b'y', b';',
    b'Y', b'U', b'c', b'y', b';',
    b'Y', b'a', b'c', b'u', b't', b'e',
    b';',
    b'Y', b'c', b'i', b'r', b'c', b';',
    b'Y', b'c', b'y', b';',
    b'Y', b'f', b'r', b';',
    b'Y', b'o', b'p', b'f', b';',
    b'Y', b's', b'c', b'r', b';',
    b'Y', b'u', b'm', b'l', b';',
    b'Z', b'H', b'c', b'y', b';',
    b'Z', b'a', b'c', b'u', b't', b'e', b';',
    b'Z', b'c', b'a', b'r', b'o', b'n', b';',
    b'Z', b'd', b'o', b't', b';',
    b'Z', b'e', b'r', b'o', b'W', b'i', b'd', b't', b'h', b'S', b'p', b'a', b'c', b'e', b';',
    b'Z', b'e', b't', b'a', b';',
    b'Z', b'f', b'r', b';',
    b'Z', b'o', b'p', b'f', b';',
    b'Z', b's', b'c', b'r', b';',
// cpp: html/parser/html_entity_table.cc:618-649
    b'a', b'a', b'c', b'u', b't', b'e',
    b';',
    b'a', b'b', b'r', b'e', b'v', b'e', b';',
    b'a', b'c', b'E', b';',
    b'a', b'c', b'd', b';',
    b'a', b'c', b'i', b'r', b'c',
    b';',
    b'a', b'c', b'y', b';',
    b'a', b'e', b'l', b'i', b'g',
    b';',
    b'a', b'f', b';',
    b'a', b'f', b'r', b';',
    b'a', b'g', b'r', b'a', b'v', b'e',
    b';',
    b'a', b'l', b'e', b'f', b's', b'y', b'm', b';',
    b'a', b'l', b'e', b'p', b'h', b';',
    b'a', b'l', b'p', b'h', b'a', b';',
    b'a', b'm', b'a', b'c', b'r', b';',
    b'a', b'm', b'a', b'l', b'g', b';',
    b'a', b'm', b'p',
    b';',
    b'a', b'n', b'd', b';',
    b'a', b'n', b'd', b'a', b'n', b'd', b';',
    b'a', b'n', b'd', b'd', b';',
    b'a', b'n', b'd', b's', b'l', b'o', b'p', b'e', b';',
    b'a', b'n', b'd', b'v', b';',
    b'a', b'n', b'g', b'e', b';',
    b'a', b'n', b'g', b'm', b's', b'd', b';',
    b'a', b'n', b'g', b'm', b's', b'd', b'a', b'a', b';',
    b'a', b'n', b'g', b'm', b's', b'd', b'a', b'b', b';',
    b'a', b'n', b'g', b'm', b's', b'd', b'a', b'c', b';',
    b'a', b'n', b'g', b'm', b's', b'd', b'a', b'd', b';',
// cpp: html/parser/html_entity_table.cc:650-681
    b'a', b'n', b'g', b'm', b's', b'd', b'a', b'e', b';',
    b'a', b'n', b'g', b'm', b's', b'd', b'a', b'f', b';',
    b'a', b'n', b'g', b'm', b's', b'd', b'a', b'g', b';',
    b'a', b'n', b'g', b'm', b's', b'd', b'a', b'h', b';',
    b'a', b'n', b'g', b'r', b't', b';',
    b'a', b'n', b'g', b'r', b't', b'v', b'b', b';',
    b'a', b'n', b'g', b'r', b't', b'v', b'b', b'd', b';',
    b'a', b'n', b'g', b's', b'p', b'h', b';',
    b'a', b'n', b'g', b's', b't', b';',
    b'a', b'n', b'g', b'z', b'a', b'r', b'r', b';',
    b'a', b'o', b'g', b'o', b'n', b';',
    b'a', b'o', b'p', b'f', b';',
    b'a', b'p', b'E', b';',
    b'a', b'p', b'a', b'c', b'i', b'r', b';',
    b'a', b'p', b'e', b';',
    b'a', b'p', b'i', b'd', b';',
    b'a', b'p', b'o', b's', b';',
    b'a', b'p', b'p', b'r', b'o', b'x', b';',
    b'a', b'p', b'p', b'r', b'o', b'x', b'e', b'q', b';',
    b'a', b'r', b'i', b'n', b'g',
    b';',
    b'a', b's', b'c', b'r', b';',
    b'a', b's', b't', b';',
    b'a', b's', b'y', b'm', b'p', b';',
    b'a', b's', b'y', b'm', b'p', b'e', b'q', b';',
    b'a', b't', b'i', b'l', b'd', b'e',
    b';',
    b'a', b'u', b'm', b'l',
    b';',
    b'a', b'w', b'c', b'o', b'n', b'i', b'n', b't', b';',
    b'a', b'w', b'i', b'n', b't', b';',
    b'b', b'N', b'o', b't', b';',
// cpp: html/parser/html_entity_table.cc:682-713
    b'b', b'a', b'c', b'k', b'c', b'o', b'n', b'g', b';',
    b'b', b'a', b'c', b'k', b'e', b'p', b's', b'i', b'l', b'o', b'n', b';',
    b'b', b'a', b'c', b'k', b'p', b'r', b'i', b'm', b'e', b';',
    b'b', b'a', b'c', b'k', b's', b'i', b'm', b';',
    b'b', b'a', b'c', b'k', b's', b'i', b'm', b'e', b'q', b';',
    b'b', b'a', b'r', b'v', b'e', b'e', b';',
    b'b', b'a', b'r', b'w', b'e', b'd', b';',
    b'b', b'a', b'r', b'w', b'e', b'd', b'g', b'e', b';',
    b'b', b'b', b'r', b'k', b';',
    b'b', b'b', b'r', b'k', b't', b'b', b'r', b'k', b';',
    b'b', b'c', b'o', b'n', b'g', b';',
    b'b', b'c', b'y', b';',
    b'b', b'd', b'q', b'u', b'o', b';',
    b'b', b'e', b'c', b'a', b'u', b's', b';',
    b'b', b'e', b'c', b'a', b'u', b's', b'e', b';',
    b'b', b'e', b'm', b'p', b't', b'y', b'v', b';',
    b'b', b'e', b'p', b's', b'i', b';',
    b'b', b'e', b'r', b'n', b'o', b'u', b';',
    b'b', b'e', b't', b'a', b';',
    b'b', b'e', b't', b'h', b';',
    b'b', b'e', b't', b'w', b'e', b'e', b'n', b';',
    b'b', b'f', b'r', b';',
    b'b', b'i', b'g', b'c', b'a', b'p', b';',
    b'b', b'i', b'g', b'c', b'i', b'r', b'c', b';',
    b'b', b'i', b'g', b'c', b'u', b'p', b';',
    b'b', b'i', b'g', b'o', b'd', b'o', b't', b';',
    b'b', b'i', b'g', b'o', b'p', b'l', b'u', b's', b';',
    b'b', b'i', b'g', b'o', b't', b'i', b'm', b'e', b's', b';',
    b'b', b'i', b'g', b's', b'q', b'c', b'u', b'p', b';',
    b'b', b'i', b'g', b's', b't', b'a', b'r', b';',
    b'b', b'i', b'g', b't', b'r', b'i', b'a', b'n', b'g', b'l', b'e', b'd', b'o', b'w', b'n', b';',
    b'b', b'i', b'g', b't', b'r', b'i', b'a', b'n', b'g', b'l', b'e', b'u', b'p', b';',
// cpp: html/parser/html_entity_table.cc:714-745
    b'b', b'i', b'g', b'u', b'p', b'l', b'u', b's', b';',
    b'b', b'i', b'g', b'v', b'e', b'e', b';',
    b'b', b'i', b'g', b'w', b'e', b'd', b'g', b'e', b';',
    b'b', b'k', b'a', b'r', b'o', b'w', b';',
    b'b', b'l', b'a', b'c', b'k', b'l', b'o', b'z', b'e', b'n', b'g', b'e', b';',
    b'b', b'l', b'a', b'c', b'k', b's', b'q', b'u', b'a', b'r', b'e', b';',
    b'b', b'l', b'a', b'c', b'k', b't', b'r', b'i', b'a', b'n', b'g', b'l', b'e', b';',
    b'b', b'l', b'a', b'c', b'k', b't', b'r', b'i', b'a', b'n', b'g', b'l', b'e', b'd', b'o', b'w', b'n', b';',
    b'b', b'l', b'a', b'c', b'k', b't', b'r', b'i', b'a', b'n', b'g', b'l', b'e', b'l', b'e', b'f', b't', b';',
    b'b', b'l', b'a', b'c', b'k', b't', b'r', b'i', b'a', b'n', b'g', b'l', b'e', b'r', b'i', b'g', b'h', b't', b';',
    b'b', b'l', b'a', b'n', b'k', b';',
    b'b', b'l', b'k', b'1', b'2', b';',
    b'b', b'l', b'k', b'1', b'4', b';',
    b'b', b'l', b'k', b'3', b'4', b';',
    b'b', b'l', b'o', b'c', b'k', b';',
    b'b', b'n', b'e', b';',
    b'b', b'n', b'e', b'q', b'u', b'i', b'v', b';',
    b'b', b'n', b'o', b't', b';',
    b'b', b'o', b'p', b'f', b';',
    b'b', b'o', b't', b';',
    b'b', b'o', b't', b't', b'o', b'm', b';',
    b'b', b'o', b'w', b't', b'i', b'e', b';',
    b'b', b'o', b'x', b'D', b'L', b';',
    b'b', b'o', b'x', b'D', b'R', b';',
    b'b', b'o', b'x', b'D', b'l', b';',
    b'b', b'o', b'x', b'D', b'r', b';',
    b'b', b'o', b'x', b'H', b';',
    b'b', b'o', b'x', b'H', b'D', b';',
    b'b', b'o', b'x', b'H', b'U', b';',
    b'b', b'o', b'x', b'H', b'd', b';',
    b'b', b'o', b'x', b'H', b'u', b';',
    b'b', b'o', b'x', b'U', b'L', b';',
// cpp: html/parser/html_entity_table.cc:746-777
    b'b', b'o', b'x', b'U', b'R', b';',
    b'b', b'o', b'x', b'U', b'l', b';',
    b'b', b'o', b'x', b'U', b'r', b';',
    b'b', b'o', b'x', b'V', b';',
    b'b', b'o', b'x', b'V', b'H', b';',
    b'b', b'o', b'x', b'V', b'L', b';',
    b'b', b'o', b'x', b'V', b'R', b';',
    b'b', b'o', b'x', b'V', b'h', b';',
    b'b', b'o', b'x', b'V', b'l', b';',
    b'b', b'o', b'x', b'V', b'r', b';',
    b'b', b'o', b'x', b'b', b'o', b'x', b';',
    b'b', b'o', b'x', b'd', b'L', b';',
    b'b', b'o', b'x', b'd', b'R', b';',
    b'b', b'o', b'x', b'd', b'l', b';',
    b'b', b'o', b'x', b'd', b'r', b';',
    b'b', b'o', b'x', b'h', b';',
    b'b', b'o', b'x', b'h', b'D', b';',
    b'b', b'o', b'x', b'h', b'U', b';',
    b'b', b'o', b'x', b'h', b'd', b';',
    b'b', b'o', b'x', b'h', b'u', b';',
    b'b', b'o', b'x', b'm', b'i', b'n', b'u', b's', b';',
    b'b', b'o', b'x', b'p', b'l', b'u', b's', b';',
    b'b', b'o', b'x', b't', b'i', b'm', b'e', b's', b';',
    b'b', b'o', b'x', b'u', b'L', b';',
    b'b', b'o', b'x', b'u', b'R', b';',
    b'b', b'o', b'x', b'u', b'l', b';',
    b'b', b'o', b'x', b'u', b'r', b';',
    b'b', b'o', b'x', b'v', b';',
    b'b', b'o', b'x', b'v', b'H', b';',
    b'b', b'o', b'x', b'v', b'L', b';',
    b'b', b'o', b'x', b'v', b'R', b';',
    b'b', b'o', b'x', b'v', b'h', b';',
// cpp: html/parser/html_entity_table.cc:778-809
    b'b', b'o', b'x', b'v', b'l', b';',
    b'b', b'o', b'x', b'v', b'r', b';',
    b'b', b'p', b'r', b'i', b'm', b'e', b';',
    b'b', b'r', b'v', b'b', b'a', b'r',
    b';',
    b'b', b's', b'c', b'r', b';',
    b'b', b's', b'e', b'm', b'i', b';',
    b'b', b's', b'i', b'm', b';',
    b'b', b's', b'i', b'm', b'e', b';',
    b'b', b's', b'o', b'l', b';',
    b'b', b's', b'o', b'l', b'b', b';',
    b'b', b's', b'o', b'l', b'h', b's', b'u', b'b', b';',
    b'b', b'u', b'l', b'l', b';',
    b'b', b'u', b'l', b'l', b'e', b't', b';',
    b'b', b'u', b'm', b'p', b';',
    b'b', b'u', b'm', b'p', b'E', b';',
    b'b', b'u', b'm', b'p', b'e', b';',
    b'b', b'u', b'm', b'p', b'e', b'q', b';',
    b'c', b'a', b'c', b'u', b't', b'e', b';',
    b'c', b'a', b'p', b'a', b'n', b'd', b';',
    b'c', b'a', b'p', b'b', b'r', b'c', b'u', b'p', b';',
    b'c', b'a', b'p', b'c', b'a', b'p', b';',
    b'c', b'a', b'p', b'c', b'u', b'p', b';',
    b'c', b'a', b'p', b'd', b'o', b't', b';',
    b'c', b'a', b'p', b's', b';',
    b'c', b'a', b'r', b'e', b't', b';',
    b'c', b'c', b'a', b'p', b's', b';',
    b'c', b'c', b'a', b'r', b'o', b'n', b';',
    b'c', b'c', b'e', b'd', b'i', b'l',
    b';',
    b'c', b'c', b'i', b'r', b'c', b';',
    b'c', b'c', b'u', b'p', b's', b';',
// cpp: html/parser/html_entity_table.cc:810-841
    b'c', b'c', b'u', b'p', b's', b's', b'm', b';',
    b'c', b'd', b'o', b't', b';',
    b'c', b'e', b'm', b'p', b't', b'y', b'v', b';',
    b'c', b'e', b'n', b't',
    b';',
    b'c', b'e', b'n', b't', b'e', b'r', b'd', b'o', b't', b';',
    b'c', b'f', b'r', b';',
    b'c', b'h', b'c', b'y', b';',
    b'c', b'h', b'e', b'c', b'k', b';',
    b'c', b'h', b'e', b'c', b'k', b'm', b'a', b'r', b'k', b';',
    b'c', b'h', b'i', b';',
    b'c', b'i', b'r', b'E', b';',
    b'c', b'i', b'r', b'c', b'e', b'q', b';',
    b'c', b'i', b'r', b'c', b'l', b'e', b'a', b'r', b'r', b'o', b'w', b'l', b'e', b'f', b't', b';',
    b'c', b'i', b'r', b'c', b'l', b'e', b'a', b'r', b'r', b'o', b'w', b'r', b'i', b'g', b'h', b't', b';',
    b'c', b'i', b'r', b'c', b'l', b'e', b'd', b'R', b';',
    b'c', b'i', b'r', b'c', b'l', b'e', b'd', b'S', b';',
    b'c', b'i', b'r', b'c', b'l', b'e', b'd', b'a', b's', b't', b';',
    b'c', b'i', b'r', b'c', b'l', b'e', b'd', b'c', b'i', b'r', b'c', b';',
    b'c', b'i', b'r', b'c', b'l', b'e', b'd', b'd', b'a', b's', b'h', b';',
    b'c', b'i', b'r', b'e', b';',
    b'c', b'i', b'r', b'f', b'n', b'i', b'n', b't', b';',
    b'c', b'i', b'r', b'm', b'i', b'd', b';',
    b'c', b'i', b'r', b's', b'c', b'i', b'r', b';',
    b'c', b'l', b'u', b'b', b's', b';',
    b'c', b'l', b'u', b'b', b's', b'u', b'i', b't', b';',
    b'c', b'o', b'l', b'o', b'n', b';',
    b'c', b'o', b'l', b'o', b'n', b'e', b';',
    b'c', b'o', b'l', b'o', b'n', b'e', b'q', b';',
    b'c', b'o', b'm', b'm', b'a', b';',
    b'c', b'o', b'm', b'm', b'a', b't', b';',
    b'c', b'o', b'm', b'p', b';',
// cpp: html/parser/html_entity_table.cc:842-873
    b'c', b'o', b'm', b'p', b'f', b'n', b';',
    b'c', b'o', b'm', b'p', b'l', b'e', b'm', b'e', b'n', b't', b';',
    b'c', b'o', b'm', b'p', b'l', b'e', b'x', b'e', b's', b';',
    b'c', b'o', b'n', b'g', b'd', b'o', b't', b';',
    b'c', b'o', b'p', b'f', b';',
    b'c', b'o', b'p', b'r', b'o', b'd', b';',
    b'c', b'o', b'p', b'y',
    b';',
    b'c', b'o', b'p', b'y', b's', b'r', b';',
    b'c', b'r', b'a', b'r', b'r', b';',
    b'c', b'r', b'o', b's', b's', b';',
    b'c', b's', b'c', b'r', b';',
    b'c', b's', b'u', b'b', b';',
    b'c', b's', b'u', b'b', b'e', b';',
    b'c', b's', b'u', b'p', b';',
    b'c', b's', b'u', b'p', b'e', b';',
    b'c', b't', b'd', b'o', b't', b';',
    b'c', b'u', b'd', b'a', b'r', b'r', b'l', b';',
    b'c', b'u', b'd', b'a', b'r', b'r', b'r', b';',
    b'c', b'u', b'e', b'p', b'r', b';',
    b'c', b'u', b'e', b's', b'c', b';',
    b'c', b'u', b'l', b'a', b'r', b'r', b';',
    b'c', b'u', b'l', b'a', b'r', b'r', b'p', b';',
    b'c', b'u', b'p', b'b', b'r', b'c', b'a', b'p', b';',
    b'c', b'u', b'p', b'c', b'a', b'p', b';',
    b'c', b'u', b'p', b'c', b'u', b'p', b';',
    b'c', b'u', b'p', b'd', b'o', b't', b';',
    b'c', b'u', b'p', b'o', b'r', b';',
    b'c', b'u', b'r', b'a', b'r', b'r', b';',
    b'c', b'u', b'r', b'a', b'r', b'r', b'm', b';',
    b'c', b'u', b'r', b'l', b'y', b'e', b'q', b'p', b'r', b'e', b'c', b';',
    b'c', b'u', b'r', b'l', b'y', b'e', b'q', b's', b'u', b'c', b'c', b';',
// cpp: html/parser/html_entity_table.cc:874-905
    b'c', b'u', b'r', b'l', b'y', b'v', b'e', b'e', b';',
    b'c', b'u', b'r', b'l', b'y', b'w', b'e', b'd', b'g', b'e', b';',
    b'c', b'u', b'r', b'r', b'e', b'n',
    b';',
    b'c', b'u', b'r', b'v', b'e', b'a', b'r', b'r', b'o', b'w', b'l', b'e', b'f', b't', b';',
    b'c', b'u', b'r', b'v', b'e', b'a', b'r', b'r', b'o', b'w', b'r', b'i', b'g', b'h', b't', b';',
    b'c', b'u', b'v', b'e', b'e', b';',
    b'c', b'u', b'w', b'e', b'd', b';',
    b'c', b'w', b'c', b'o', b'n', b'i', b'n', b't', b';',
    b'c', b'w', b'i', b'n', b't', b';',
    b'c', b'y', b'l', b'c', b't', b'y', b';',
    b'd', b'A', b'r', b'r', b';',
    b'd', b'H', b'a', b'r', b';',
    b'd', b'a', b'g', b'g', b'e', b'r', b';',
    b'd', b'a', b'l', b'e', b't', b'h', b';',
    b'd', b'a', b'r', b'r', b';',
    b'd', b'a', b's', b'h', b'v', b';',
    b'd', b'b', b'k', b'a', b'r', b'o', b'w', b';',
    b'd', b'c', b'a', b'r', b'o', b'n', b';',
    b'd', b'c', b'y', b';',
    b'd', b'd', b'a', b'g', b'g', b'e', b'r', b';',
    b'd', b'd', b'a', b'r', b'r', b';',
    b'd', b'd', b'o', b't', b's', b'e', b'q', b';',
    b'd', b'e', b'g',
    b';',
    b'd', b'e', b'l', b't', b'a', b';',
    b'd', b'e', b'm', b'p', b't', b'y', b'v', b';',
    b'd', b'f', b'i', b's', b'h', b't', b';',
    b'd', b'f', b'r', b';',
    b'd', b'h', b'a', b'r', b'l', b';',
    b'd', b'h', b'a', b'r', b'r', b';',
    b'd', b'i', b'a', b'm', b';',
// cpp: html/parser/html_entity_table.cc:906-937
    b'd', b'i', b'a', b'm', b'o', b'n', b'd', b';',
    b'd', b'i', b'a', b'm', b'o', b'n', b'd', b's', b'u', b'i', b't', b';',
    b'd', b'i', b'a', b'm', b's', b';',
    b'd', b'i', b'e', b';',
    b'd', b'i', b'g', b'a', b'm', b'm', b'a', b';',
    b'd', b'i', b's', b'i', b'n', b';',
    b'd', b'i', b'v', b';',
    b'd', b'i', b'v', b'i', b'd', b'e',
    b';',
    b'd', b'i', b'v', b'i', b'd', b'e', b'o', b'n', b't', b'i', b'm', b'e', b's', b';',
    b'd', b'i', b'v', b'o', b'n', b'x', b';',
    b'd', b'j', b'c', b'y', b';',
    b'd', b'l', b'c', b'o', b'r', b'n', b';',
    b'd', b'l', b'c', b'r', b'o', b'p', b';',
    b'd', b'o', b'l', b'l', b'a', b'r', b';',
    b'd', b'o', b'p', b'f', b';',
    b'd', b'o', b't', b'e', b'q', b';',
    b'd', b'o', b't', b'e', b'q', b'd', b'o', b't', b';',
    b'd', b'o', b't', b'm', b'i', b'n', b'u', b's', b';',
    b'd', b'o', b't', b'p', b'l', b'u', b's', b';',
    b'd', b'o', b't', b's', b'q', b'u', b'a', b'r', b'e', b';',
    b'd', b'o', b'u', b'b', b'l', b'e', b'b', b'a', b'r', b'w', b'e', b'd', b'g', b'e', b';',
    b'd', b'o', b'w', b'n', b'd', b'o', b'w', b'n', b'a', b'r', b'r', b'o', b'w', b's', b';',
    b'd', b'o', b'w', b'n', b'h', b'a', b'r', b'p', b'o', b'o', b'n', b'l', b'e', b'f', b't', b';',
    b'd', b'o', b'w', b'n', b'h', b'a', b'r', b'p', b'o', b'o', b'n', b'r', b'i', b'g', b'h', b't', b';',
    b'd', b'r', b'b', b'k', b'a', b'r', b'o', b'w', b';',
    b'd', b'r', b'c', b'o', b'r', b'n', b';',
    b'd', b'r', b'c', b'r', b'o', b'p', b';',
    b'd', b's', b'c', b'r', b';',
    b'd', b's', b'c', b'y', b';',
    b'd', b's', b'o', b'l', b';',
    b'd', b's', b't', b'r', b'o', b'k', b';',
// cpp: html/parser/html_entity_table.cc:938-969
    b'd', b't', b'd', b'o', b't', b';',
    b'd', b't', b'r', b'i', b';',
    b'd', b't', b'r', b'i', b'f', b';',
    b'd', b'u', b'a', b'r', b'r', b';',
    b'd', b'u', b'h', b'a', b'r', b';',
    b'd', b'w', b'a', b'n', b'g', b'l', b'e', b';',
    b'd', b'z', b'c', b'y', b';',
    b'd', b'z', b'i', b'g', b'r', b'a', b'r', b'r', b';',
    b'e', b'D', b'D', b'o', b't', b';',
    b'e', b'a', b'c', b'u', b't', b'e',
    b';',
    b'e', b'a', b's', b't', b'e', b'r', b';',
    b'e', b'c', b'a', b'r', b'o', b'n', b';',
    b'e', b'c', b'i', b'r', b';',
    b'e', b'c', b'i', b'r', b'c',
    b';',
    b'e', b'c', b'o', b'l', b'o', b'n', b';',
    b'e', b'c', b'y', b';',
    b'e', b'd', b'o', b't', b';',
    b'e', b'f', b'D', b'o', b't', b';',
    b'e', b'f', b'r', b';',
    b'e', b'g', b'r', b'a', b'v', b'e',
    b';',
    b'e', b'g', b's', b';',
    b'e', b'g', b's', b'd', b'o', b't', b';',
    b'e', b'l', b'i', b'n', b't', b'e', b'r', b's', b';',
    b'e', b'l', b'l', b';',
    b'e', b'l', b's', b';',
    b'e', b'l', b's', b'd', b'o', b't', b';',
    b'e', b'm', b'a', b'c', b'r', b';',
    b'e', b'm', b'p', b't', b'y', b';',
    b'e', b'm', b'p', b't', b'y', b's', b'e', b't', b';',
// cpp: html/parser/html_entity_table.cc:970-1001
    b'e', b'm', b's', b'p', b'1', b'3', b';',
    b'e', b'm', b's', b'p', b'1', b'4', b';',
    b'e', b'm', b's', b'p', b';',
    b'e', b'n', b'g', b';',
    b'e', b'n', b's', b'p', b';',
    b'e', b'o', b'g', b'o', b'n', b';',
    b'e', b'o', b'p', b'f', b';',
    b'e', b'p', b'a', b'r', b';',
    b'e', b'p', b'a', b'r', b's', b'l', b';',
    b'e', b'p', b'l', b'u', b's', b';',
    b'e', b'p', b's', b'i', b'v', b';',
    b'e', b'q', b'c', b'i', b'r', b'c', b';',
    b'e', b'q', b'c', b'o', b'l', b'o', b'n', b';',
    b'e', b'q', b's', b'i', b'm', b';',
    b'e', b'q', b's', b'l', b'a', b'n', b't', b'g', b't', b'r', b';',
    b'e', b'q', b's', b'l', b'a', b'n', b't', b'l', b'e', b's', b's', b';',
    b'e', b'q', b'u', b'a', b'l', b's', b';',
    b'e', b'q', b'u', b'e', b's', b't', b';',
    b'e', b'q', b'u', b'i', b'v', b'D', b'D', b';',
    b'e', b'q', b'v', b'p', b'a', b'r', b's', b'l', b';',
    b'e', b'r', b'a', b'r', b'r', b';',
    b'e', b's', b'c', b'r', b';',
    b'e', b's', b'd', b'o', b't', b';',
    b'e', b's', b'i', b'm', b';',
    b'e', b'u', b'm', b'l',
    b';',
    b'e', b'u', b'r', b'o', b';',
    b'e', b'x', b'c', b'l', b';',
    b'e', b'x', b'i', b's', b't', b';',
    b'e', b'x', b'p', b'e', b'c', b't', b'a', b't', b'i', b'o', b'n', b';',
    b'e', b'x', b'p', b'o', b'n', b'e', b'n', b't', b'i', b'a', b'l', b'e', b';',
    b'f', b'a', b'l', b'l', b'i', b'n', b'g', b'd', b'o', b't', b's', b'e', b'q', b';',
// cpp: html/parser/html_entity_table.cc:1002-1033
    b'f', b'c', b'y', b';',
    b'f', b'e', b'm', b'a', b'l', b'e', b';',
    b'f', b'f', b'i', b'l', b'i', b'g', b';',
    b'f', b'f', b'l', b'i', b'g', b';',
    b'f', b'f', b'l', b'l', b'i', b'g', b';',
    b'f', b'f', b'r', b';',
    b'f', b'j', b'l', b'i', b'g', b';',
    b'f', b'l', b'a', b't', b';',
    b'f', b'l', b't', b'n', b's', b';',
    b'f', b'n', b'o', b'f', b';',
    b'f', b'o', b'p', b'f', b';',
    b'f', b'o', b'r', b'a', b'l', b'l', b';',
    b'f', b'o', b'r', b'k', b';',
    b'f', b'o', b'r', b'k', b'v', b';',
    b'f', b'p', b'a', b'r', b't', b'i', b'n', b't', b';',
    b'f', b'r', b'a', b'c', b'1', b'2',
    b';',
    b'f', b'r', b'a', b'c', b'1', b'3', b';',
    b'f', b'r', b'a', b'c', b'1', b'4',
    b';',
    b'f', b'r', b'a', b'c', b'1', b'5', b';',
    b'f', b'r', b'a', b'c', b'1', b'6', b';',
    b'f', b'r', b'a', b'c', b'1', b'8', b';',
    b'f', b'r', b'a', b'c', b'2', b'3', b';',
    b'f', b'r', b'a', b'c', b'2', b'5', b';',
    b'f', b'r', b'a', b'c', b'3', b'4',
    b';',
    b'f', b'r', b'a', b'c', b'3', b'5', b';',
    b'f', b'r', b'a', b'c', b'3', b'8', b';',
    b'f', b'r', b'a', b'c', b'4', b'5', b';',
    b'f', b'r', b'a', b'c', b'5', b'6', b';',
    b'f', b'r', b'a', b'c', b'5', b'8', b';',
// cpp: html/parser/html_entity_table.cc:1034-1065
    b'f', b'r', b'a', b'c', b'7', b'8', b';',
    b'f', b'r', b'a', b's', b'l', b';',
    b'f', b'r', b'o', b'w', b'n', b';',
    b'f', b's', b'c', b'r', b';',
    b'g', b'E', b';',
    b'g', b'E', b'l', b';',
    b'g', b'a', b'c', b'u', b't', b'e', b';',
    b'g', b'a', b'm', b'm', b'a', b'd', b';',
    b'g', b'a', b'p', b';',
    b'g', b'b', b'r', b'e', b'v', b'e', b';',
    b'g', b'c', b'y', b';',
    b'g', b'e', b'l', b';',
    b'g', b'e', b'q', b';',
    b'g', b'e', b'q', b'q', b';',
    b'g', b'e', b'q', b's', b'l', b'a', b'n', b't', b';',
    b'g', b'e', b's', b';',
    b'g', b'e', b's', b'c', b'c', b';',
    b'g', b'e', b's', b'd', b'o', b't', b';',
    b'g', b'e', b's', b'd', b'o', b't', b'o', b';',
    b'g', b'e', b's', b'd', b'o', b't', b'o', b'l', b';',
    b'g', b'e', b's', b'l', b';',
    b'g', b'e', b's', b'l', b'e', b's', b';',
    b'g', b'f', b'r', b';',
    b'g', b'g', b';',
    b'g', b'g', b'g', b';',
    b'g', b'i', b'm', b'e', b'l', b';',
    b'g', b'j', b'c', b'y', b';',
    b'g', b'l', b';',
    b'g', b'l', b'E', b';',
    b'g', b'l', b'a', b';',
    b'g', b'l', b'j', b';',
    b'g', b'n', b'E', b';',
// cpp: html/parser/html_entity_table.cc:1066-1097
    b'g', b'n', b'a', b'p', b';',
    b'g', b'n', b'a', b'p', b'p', b'r', b'o', b'x', b';',
    b'g', b'n', b'e', b';',
    b'g', b'n', b'e', b'q', b';',
    b'g', b'n', b'e', b'q', b'q', b';',
    b'g', b'n', b's', b'i', b'm', b';',
    b'g', b'o', b'p', b'f', b';',
    b'g', b's', b'c', b'r', b';',
    b'g', b's', b'i', b'm', b';',
    b'g', b's', b'i', b'm', b'e', b';',
    b'g', b's', b'i', b'm', b'l', b';',
    b'g', b't', b';',
    b'g', b't', b'c', b'c', b';',
    b'g', b't', b'c', b'i', b'r', b';',
    b'g', b't', b'd', b'o', b't', b';',
    b'g', b't', b'l', b'P', b'a', b'r', b';',
    b'g', b't', b'q', b'u', b'e', b's', b't', b';',
    b'g', b't', b'r', b'a', b'p', b'p', b'r', b'o', b'x', b';',
    b'g', b't', b'r', b'a', b'r', b'r', b';',
    b'g', b't', b'r', b'd', b'o', b't', b';',
    b'g', b't', b'r', b'e', b'q', b'l', b'e', b's', b's', b';',
    b'g', b't', b'r', b'e', b'q', b'q', b'l', b'e', b's', b's', b';',
    b'g', b't', b'r', b'l', b'e', b's', b's', b';',
    b'g', b't', b'r', b's', b'i', b'm', b';',
    b'g', b'v', b'e', b'r', b't', b'n', b'e', b'q', b'q', b';',
    b'g', b'v', b'n', b'E', b';',
    b'h', b'A', b'r', b'r', b';',
    b'h', b'a', b'i', b'r', b's', b'p', b';',
    b'h', b'a', b'l', b'f', b';',
    b'h', b'a', b'm', b'i', b'l', b't', b';',
    b'h', b'a', b'r', b'd', b'c', b'y', b';',
    b'h', b'a', b'r', b'r', b'c', b'i', b'r', b';',
// cpp: html/parser/html_entity_table.cc:1098-1129
    b'h', b'a', b'r', b'r', b'w', b';',
    b'h', b'b', b'a', b'r', b';',
    b'h', b'c', b'i', b'r', b'c', b';',
    b'h', b'e', b'a', b'r', b't', b's', b';',
    b'h', b'e', b'a', b'r', b't', b's', b'u', b'i', b't', b';',
    b'h', b'e', b'l', b'l', b'i', b'p', b';',
    b'h', b'e', b'r', b'c', b'o', b'n', b';',
    b'h', b'f', b'r', b';',
    b'h', b'k', b's', b'e', b'a', b'r', b'o', b'w', b';',
    b'h', b'k', b's', b'w', b'a', b'r', b'o', b'w', b';',
    b'h', b'o', b'a', b'r', b'r', b';',
    b'h', b'o', b'm', b't', b'h', b't', b';',
    b'h', b'o', b'o', b'k', b'l', b'e', b'f', b't', b'a', b'r', b'r', b'o', b'w', b';',
    b'h', b'o', b'o', b'k', b'r', b'i', b'g', b'h', b't', b'a', b'r', b'r', b'o', b'w', b';',
    b'h', b'o', b'p', b'f', b';',
    b'h', b'o', b'r', b'b', b'a', b'r', b';',
    b'h', b's', b'c', b'r', b';',
    b'h', b's', b'l', b'a', b's', b'h', b';',
    b'h', b's', b't', b'r', b'o', b'k', b';',
    b'h', b'y', b'b', b'u', b'l', b'l', b';',
    b'h', b'y', b'p', b'h', b'e', b'n', b';',
    b'i', b'a', b'c', b'u', b't', b'e',
    b';',
    b'i', b'c', b';',
    b'i', b'c', b'i', b'r', b'c',
    b';',
    b'i', b'c', b'y', b';',
    b'i', b'e', b'c', b'y', b';',
    b'i', b'e', b'x', b'c', b'l',
    b';',
    b'i', b'f', b'f', b';',
    b'i', b'f', b'r', b';',
// cpp: html/parser/html_entity_table.cc:1130-1161
    b'i', b'g', b'r', b'a', b'v', b'e',
    b';',
    b'i', b'i', b';',
    b'i', b'i', b'i', b'i', b'n', b't', b';',
    b'i', b'i', b'n', b'f', b'i', b'n', b';',
    b'i', b'i', b'o', b't', b'a', b';',
    b'i', b'j', b'l', b'i', b'g', b';',
    b'i', b'm', b'a', b'c', b'r', b';',
    b'i', b'm', b'a', b'g', b'e', b';',
    b'i', b'm', b'a', b'g', b'l', b'i', b'n', b'e', b';',
    b'i', b'm', b'a', b'g', b'p', b'a', b'r', b't', b';',
    b'i', b'm', b'a', b't', b'h', b';',
    b'i', b'm', b'o', b'f', b';',
    b'i', b'm', b'p', b'e', b'd', b';',
    b'i', b'n', b'c', b'a', b'r', b'e', b';',
    b'i', b'n', b'f', b'i', b'n', b't', b'i', b'e', b';',
    b'i', b'n', b'o', b'd', b'o', b't', b';',
    b'i', b'n', b't', b'c', b'a', b'l', b';',
    b'i', b'n', b't', b'e', b'g', b'e', b'r', b's', b';',
    b'i', b'n', b't', b'e', b'r', b'c', b'a', b'l', b';',
    b'i', b'n', b't', b'l', b'a', b'r', b'h', b'k', b';',
    b'i', b'n', b't', b'p', b'r', b'o', b'd', b';',
    b'i', b'o', b'c', b'y', b';',
    b'i', b'o', b'g', b'o', b'n', b';',
    b'i', b'o', b'p', b'f', b';',
    b'i', b'p', b'r', b'o', b'd', b';',
    b'i', b'q', b'u', b'e', b's', b't',
    b';',
    b'i', b's', b'c', b'r', b';',
    b'i', b's', b'i', b'n', b'E', b';',
    b'i', b's', b'i', b'n', b'd', b'o', b't', b';',
    b'i', b's', b'i', b'n', b's', b';',
// cpp: html/parser/html_entity_table.cc:1162-1193
    b'i', b's', b'i', b'n', b's', b'v', b';',
    b'i', b's', b'i', b'n', b'v', b';',
    b'i', b't', b'i', b'l', b'd', b'e', b';',
    b'i', b'u', b'k', b'c', b'y', b';',
    b'i', b'u', b'm', b'l',
    b';',
    b'j', b'c', b'i', b'r', b'c', b';',
    b'j', b'f', b'r', b';',
    b'j', b'm', b'a', b't', b'h', b';',
    b'j', b'o', b'p', b'f', b';',
    b'j', b's', b'c', b'r', b';',
    b'j', b's', b'e', b'r', b'c', b'y', b';',
    b'j', b'u', b'k', b'c', b'y', b';',
    b'k', b'a', b'p', b'p', b'a', b';',
    b'k', b'a', b'p', b'p', b'a', b'v', b';',
    b'k', b'c', b'e', b'd', b'i', b'l', b';',
    b'k', b'f', b'r', b';',
    b'k', b'g', b'r', b'e', b'e', b'n', b';',
    b'k', b'h', b'c', b'y', b';',
    b'k', b'j', b'c', b'y', b';',
    b'k', b'o', b'p', b'f', b';',
    b'k', b's', b'c', b'r', b';',
    b'l', b'A', b'a', b'r', b'r', b';',
    b'l', b'A', b'r', b'r', b';',
    b'l', b'A', b't', b'a', b'i', b'l', b';',
    b'l', b'B', b'a', b'r', b'r', b';',
    b'l', b'E', b'g', b';',
    b'l', b'H', b'a', b'r', b';',
    b'l', b'a', b'c', b'u', b't', b'e', b';',
    b'l', b'a', b'e', b'm', b'p', b't', b'y', b'v', b';',
    b'l', b'a', b'g', b'r', b'a', b'n', b';',
    b'l', b'a', b'm', b'b', b'd', b'a', b';',
// cpp: html/parser/html_entity_table.cc:1194-1225
    b'l', b'a', b'n', b'g', b';',
    b'l', b'a', b'n', b'g', b'd', b';',
    b'l', b'a', b'n', b'g', b'l', b'e', b';',
    b'l', b'a', b'p', b';',
    b'l', b'a', b'q', b'u', b'o',
    b';',
    b'l', b'a', b'r', b'r', b'b', b';',
    b'l', b'a', b'r', b'r', b'b', b'f', b's', b';',
    b'l', b'a', b'r', b'r', b'f', b's', b';',
    b'l', b'a', b'r', b'r', b'h', b'k', b';',
    b'l', b'a', b'r', b'r', b'l', b'p', b';',
    b'l', b'a', b'r', b'r', b'p', b'l', b';',
    b'l', b'a', b'r', b'r', b's', b'i', b'm', b';',
    b'l', b'a', b'r', b'r', b't', b'l', b';',
    b'l', b'a', b't', b'a', b'i', b'l', b';',
    b'l', b'a', b't', b'e', b';',
    b'l', b'a', b't', b'e', b's', b';',
    b'l', b'b', b'a', b'r', b'r', b';',
    b'l', b'b', b'b', b'r', b'k', b';',
    b'l', b'b', b'r', b'a', b'c', b'e', b';',
    b'l', b'b', b'r', b'a', b'c', b'k', b';',
    b'l', b'b', b'r', b'k', b'e', b';',
    b'l', b'b', b'r', b'k', b's', b'l', b'd', b';',
    b'l', b'b', b'r', b'k', b's', b'l', b'u', b';',
    b'l', b'c', b'a', b'r', b'o', b'n', b';',
    b'l', b'c', b'e', b'd', b'i', b'l', b';',
    b'l', b'c', b'e', b'i', b'l', b';',
    b'l', b'c', b'u', b'b', b';',
    b'l', b'c', b'y', b';',
    b'l', b'd', b'c', b'a', b';',
    b'l', b'd', b'q', b'u', b'o', b';',
    b'l', b'd', b'q', b'u', b'o', b'r', b';',
// cpp: html/parser/html_entity_table.cc:1226-1257
    b'l', b'd', b'r', b'd', b'h', b'a', b'r', b';',
    b'l', b'd', b'r', b'u', b's', b'h', b'a', b'r', b';',
    b'l', b'd', b's', b'h', b';',
    b'l', b'e', b'f', b't', b'a', b'r', b'r', b'o', b'w', b't', b'a', b'i', b'l', b';',
    b'l', b'e', b'f', b't', b'h', b'a', b'r', b'p', b'o', b'o', b'n', b'd', b'o', b'w', b'n', b';',
    b'l', b'e', b'f', b't', b'h', b'a', b'r', b'p', b'o', b'o', b'n', b'u', b'p', b';',
    b'l', b'e', b'f', b't', b'l', b'e', b'f', b't', b'a', b'r', b'r', b'o', b'w', b's', b';',
    b'l', b'e', b'f', b't', b'r', b'i', b'g', b'h', b't', b'a', b'r', b'r', b'o', b'w', b's', b';',
    b'l', b'e', b'f', b't', b'r', b'i', b'g', b'h', b't', b'h', b'a', b'r', b'p', b'o', b'o', b'n', b's', b';',
    b'l', b'e', b'f', b't', b'r', b'i', b'g', b'h', b't', b's', b'q', b'u', b'i', b'g', b'a', b'r', b'r', b'o', b'w', b';',
    b'l', b'e', b'f', b't', b't', b'h', b'r', b'e', b'e', b't', b'i', b'm', b'e', b's', b';',
    b'l', b'e', b'g', b';',
    b'l', b'e', b'q', b';',
    b'l', b'e', b'q', b'q', b';',
    b'l', b'e', b'q', b's', b'l', b'a', b'n', b't', b';',
    b'l', b'e', b's', b'c', b'c', b';',
    b'l', b'e', b's', b'd', b'o', b't', b';',
    b'l', b'e', b's', b'd', b'o', b't', b'o', b';',
    b'l', b'e', b's', b'd', b'o', b't', b'o', b'r', b';',
    b'l', b'e', b's', b'g', b';',
    b'l', b'e', b's', b'g', b'e', b's', b';',
    b'l', b'e', b's', b's', b'a', b'p', b'p', b'r', b'o', b'x', b';',
    b'l', b'e', b's', b's', b'd', b'o', b't', b';',
    b'l', b'e', b's', b's', b'e', b'q', b'g', b't', b'r', b';',
    b'l', b'e', b's', b's', b'e', b'q', b'q', b'g', b't', b'r', b';',
    b'l', b'e', b's', b's', b'g', b't', b'r', b';',
    b'l', b'e', b's', b's', b's', b'i', b'm', b';',
    b'l', b'f', b'i', b's', b'h', b't', b';',
    b'l', b'f', b'l', b'o', b'o', b'r', b';',
    b'l', b'f', b'r', b';',
    b'l', b'g', b'E', b';',
    b'l', b'h', b'a', b'r', b'd', b';',
// cpp: html/parser/html_entity_table.cc:1258-1289
    b'l', b'h', b'a', b'r', b'u', b';',
    b'l', b'h', b'a', b'r', b'u', b'l', b';',
    b'l', b'h', b'b', b'l', b'k', b';',
    b'l', b'j', b'c', b'y', b';',
    b'l', b'l', b'a', b'r', b'r', b';',
    b'l', b'l', b'c', b'o', b'r', b'n', b'e', b'r', b';',
    b'l', b'l', b'h', b'a', b'r', b'd', b';',
    b'l', b'l', b't', b'r', b'i', b';',
    b'l', b'm', b'i', b'd', b'o', b't', b';',
    b'l', b'm', b'o', b'u', b's', b't', b';',
    b'l', b'm', b'o', b'u', b's', b't', b'a', b'c', b'h', b'e', b';',
    b'l', b'n', b'E', b';',
    b'l', b'n', b'a', b'p', b';',
    b'l', b'n', b'a', b'p', b'p', b'r', b'o', b'x', b';',
    b'l', b'n', b'e', b';',
    b'l', b'n', b'e', b'q', b';',
    b'l', b'n', b'e', b'q', b'q', b';',
    b'l', b'n', b's', b'i', b'm', b';',
    b'l', b'o', b'a', b'n', b'g', b';',
    b'l', b'o', b'a', b'r', b'r', b';',
    b'l', b'o', b'b', b'r', b'k', b';',
    b'l', b'o', b'n', b'g', b'l', b'e', b'f', b't', b'a', b'r', b'r', b'o', b'w', b';',
    b'l', b'o', b'n', b'g', b'l', b'e', b'f', b't', b'r', b'i', b'g', b'h', b't', b'a', b'r', b'r', b'o', b'w', b';',
    b'l', b'o', b'n', b'g', b'm', b'a', b'p', b's', b't', b'o', b';',
    b'l', b'o', b'n', b'g', b'r', b'i', b'g', b'h', b't', b'a', b'r', b'r', b'o', b'w', b';',
    b'l', b'o', b'o', b'p', b'a', b'r', b'r', b'o', b'w', b'l', b'e', b'f', b't', b';',
    b'l', b'o', b'o', b'p', b'a', b'r', b'r', b'o', b'w', b'r', b'i', b'g', b'h', b't', b';',
    b'l', b'o', b'p', b'a', b'r', b';',
    b'l', b'o', b'p', b'f', b';',
    b'l', b'o', b'p', b'l', b'u', b's', b';',
    b'l', b'o', b't', b'i', b'm', b'e', b's', b';',
    b'l', b'o', b'w', b'a', b's', b't', b';',
// cpp: html/parser/html_entity_table.cc:1290-1321
    b'l', b'o', b'w', b'b', b'a', b'r', b';',
    b'l', b'o', b'z', b';',
    b'l', b'o', b'z', b'f', b';',
    b'l', b'p', b'a', b'r', b';',
    b'l', b'p', b'a', b'r', b'l', b't', b';',
    b'l', b'r', b'a', b'r', b'r', b';',
    b'l', b'r', b'c', b'o', b'r', b'n', b'e', b'r', b';',
    b'l', b'r', b'h', b'a', b'r', b';',
    b'l', b'r', b'h', b'a', b'r', b'd', b';',
    b'l', b'r', b'm', b';',
    b'l', b'r', b't', b'r', b'i', b';',
    b'l', b's', b'a', b'q', b'u', b'o', b';',
    b'l', b's', b'c', b'r', b';',
    b'l', b's', b'h', b';',
    b'l', b's', b'i', b'm', b';',
    b'l', b's', b'i', b'm', b'e', b';',
    b'l', b's', b'i', b'm', b'g', b';',
    b'l', b's', b'q', b'b', b';',
    b'l', b's', b'q', b'u', b'o', b';',
    b'l', b's', b'q', b'u', b'o', b'r', b';',
    b'l', b's', b't', b'r', b'o', b'k', b';',
    b'l', b't', b'c', b'c', b';',
    b'l', b't', b'c', b'i', b'r', b';',
    b'l', b't', b'd', b'o', b't', b';',
    b'l', b't', b'h', b'r', b'e', b'e', b';',
    b'l', b't', b'i', b'm', b'e', b's', b';',
    b'l', b't', b'l', b'a', b'r', b'r', b';',
    b'l', b't', b'q', b'u', b'e', b's', b't', b';',
    b'l', b't', b'r', b'P', b'a', b'r', b';',
    b'l', b't', b'r', b'i', b'e', b';',
    b'l', b't', b'r', b'i', b'f', b';',
    b'l', b'u', b'r', b'd', b's', b'h', b'a', b'r', b';',
// cpp: html/parser/html_entity_table.cc:1322-1353
    b'l', b'u', b'r', b'u', b'h', b'a', b'r', b';',
    b'l', b'v', b'e', b'r', b't', b'n', b'e', b'q', b'q', b';',
    b'l', b'v', b'n', b'E', b';',
    b'm', b'D', b'D', b'o', b't', b';',
    b'm', b'a', b'l', b't', b';',
    b'm', b'a', b'l', b't', b'e', b's', b'e', b';',
    b'm', b'a', b'p', b';',
    b'm', b'a', b'p', b's', b't', b'o', b'd', b'o', b'w', b'n', b';',
    b'm', b'a', b'p', b's', b't', b'o', b'l', b'e', b'f', b't', b';',
    b'm', b'a', b'p', b's', b't', b'o', b'u', b'p', b';',
    b'm', b'a', b'r', b'k', b'e', b'r', b';',
    b'm', b'c', b'o', b'm', b'm', b'a', b';',
    b'm', b'c', b'y', b';',
    b'm', b'd', b'a', b's', b'h', b';',
    b'm', b'e', b'a', b's', b'u', b'r', b'e', b'd', b'a', b'n', b'g', b'l', b'e', b';',
    b'm', b'f', b'r', b';',
    b'm', b'h', b'o', b';',
    b'm', b'i', b'c', b'r', b'o', b';',
    b'm', b'i', b'd', b'a', b's', b't', b';',
    b'm', b'i', b'd', b'c', b'i', b'r', b';',
    b'm', b'i', b'd', b'd', b'o', b't',
    b';',
    b'm', b'i', b'n', b'u', b's', b'b', b';',
    b'm', b'i', b'n', b'u', b's', b'd', b';',
    b'm', b'i', b'n', b'u', b's', b'd', b'u', b';',
    b'm', b'l', b'c', b'p', b';',
    b'm', b'l', b'd', b'r', b';',
    b'm', b'n', b'p', b'l', b'u', b's', b';',
    b'm', b'o', b'd', b'e', b'l', b's', b';',
    b'm', b'o', b'p', b'f', b';',
    b'm', b's', b'c', b'r', b';',
    b'm', b's', b't', b'p', b'o', b's', b';',
// cpp: html/parser/html_entity_table.cc:1354-1385
    b'm', b'u', b';',
    b'm', b'u', b'l', b't', b'i', b'm', b'a', b'p', b';',
    b'm', b'u', b'm', b'a', b'p', b';',
    b'n', b'G', b'g', b';',
    b'n', b'G', b't', b';',
    b'n', b'G', b't', b'v', b';',
    b'n', b'L', b'e', b'f', b't', b'a', b'r', b'r', b'o', b'w', b';',
    b'n', b'L', b'e', b'f', b't', b'r', b'i', b'g', b'h', b't', b'a', b'r', b'r', b'o', b'w', b';',
    b'n', b'L', b'l', b';',
    b'n', b'L', b't', b';',
    b'n', b'L', b't', b'v', b';',
    b'n', b'R', b'i', b'g', b'h', b't', b'a', b'r', b'r', b'o', b'w', b';',
    b'n', b'V', b'D', b'a', b's', b'h', b';',
    b'n', b'V', b'd', b'a', b's', b'h', b';',
    b'n', b'a', b'b', b'l', b'a', b';',
    b'n', b'a', b'c', b'u', b't', b'e', b';',
    b'n', b'a', b'n', b'g', b';',
    b'n', b'a', b'p', b'E', b';',
    b'n', b'a', b'p', b'i', b'd', b';',
    b'n', b'a', b'p', b'o', b's', b';',
    b'n', b'a', b't', b'u', b'r', b';',
    b'n', b'a', b't', b'u', b'r', b'a', b'l', b';',
    b'n', b'a', b't', b'u', b'r', b'a', b'l', b's', b';',
    b'n', b'b', b's', b'p',
    b';',
    b'n', b'b', b'u', b'm', b'p', b';',
    b'n', b'b', b'u', b'm', b'p', b'e', b';',
    b'n', b'c', b'a', b'p', b';',
    b'n', b'c', b'a', b'r', b'o', b'n', b';',
    b'n', b'c', b'e', b'd', b'i', b'l', b';',
    b'n', b'c', b'o', b'n', b'g', b';',
    b'n', b'c', b'o', b'n', b'g', b'd', b'o', b't', b';',
// cpp: html/parser/html_entity_table.cc:1386-1417
    b'n', b'c', b'u', b'p', b';',
    b'n', b'c', b'y', b';',
    b'n', b'd', b'a', b's', b'h', b';',
    b'n', b'e', b'A', b'r', b'r', b';',
    b'n', b'e', b'a', b'r', b'h', b'k', b';',
    b'n', b'e', b'a', b'r', b'r', b';',
    b'n', b'e', b'a', b'r', b'r', b'o', b'w', b';',
    b'n', b'e', b'd', b'o', b't', b';',
    b'n', b'e', b's', b'e', b'a', b'r', b';',
    b'n', b'e', b's', b'i', b'm', b';',
    b'n', b'e', b'x', b'i', b's', b't', b';',
    b'n', b'e', b'x', b'i', b's', b't', b's', b';',
    b'n', b'f', b'r', b';',
    b'n', b'g', b'E', b';',
    b'n', b'g', b'e', b'q', b';',
    b'n', b'g', b'e', b'q', b'q', b';',
    b'n', b'g', b'e', b'q', b's', b'l', b'a', b'n', b't', b';',
    b'n', b'g', b'e', b's', b';',
    b'n', b'g', b's', b'i', b'm', b';',
    b'n', b'g', b't', b';',
    b'n', b'g', b't', b'r', b';',
    b'n', b'h', b'A', b'r', b'r', b';',
    b'n', b'h', b'a', b'r', b'r', b';',
    b'n', b'h', b'p', b'a', b'r', b';',
    b'n', b'i', b';',
    b'n', b'i', b's', b';',
    b'n', b'i', b's', b'd', b';',
    b'n', b'i', b'v', b';',
    b'n', b'j', b'c', b'y', b';',
    b'n', b'l', b'A', b'r', b'r', b';',
    b'n', b'l', b'E', b';',
    b'n', b'l', b'a', b'r', b'r', b';',
// cpp: html/parser/html_entity_table.cc:1418-1449
    b'n', b'l', b'd', b'r', b';',
    b'n', b'l', b'e', b';',
    b'n', b'l', b'e', b'f', b't', b'a', b'r', b'r', b'o', b'w', b';',
    b'n', b'l', b'e', b'f', b't', b'r', b'i', b'g', b'h', b't', b'a', b'r', b'r', b'o', b'w', b';',
    b'n', b'l', b'e', b'q', b';',
    b'n', b'l', b'e', b'q', b'q', b';',
    b'n', b'l', b'e', b'q', b's', b'l', b'a', b'n', b't', b';',
    b'n', b'l', b'e', b's', b';',
    b'n', b'l', b'e', b's', b's', b';',
    b'n', b'l', b's', b'i', b'm', b';',
    b'n', b'l', b't', b';',
    b'n', b'l', b't', b'r', b'i', b';',
    b'n', b'l', b't', b'r', b'i', b'e', b';',
    b'n', b'm', b'i', b'd', b';',
    b'n', b'o', b'p', b'f', b';',
    b'n', b'o', b't', b'i', b'n', b';',
    b'n', b'o', b't', b'i', b'n', b'E', b';',
    b'n', b'o', b't', b'i', b'n', b'd', b'o', b't', b';',
    b'n', b'o', b't', b'i', b'n', b'v', b'a', b';',
    b'n', b'o', b't', b'i', b'n', b'v', b'b', b';',
    b'n', b'o', b't', b'i', b'n', b'v', b'c', b';',
    b'n', b'o', b't', b'n', b'i', b';',
    b'n', b'o', b't', b'n', b'i', b'v', b'a', b';',
    b'n', b'o', b't', b'n', b'i', b'v', b'b', b';',
    b'n', b'o', b't', b'n', b'i', b'v', b'c', b';',
    b'n', b'p', b'a', b'r', b';',
    b'n', b'p', b'a', b'r', b'a', b'l', b'l', b'e', b'l', b';',
    b'n', b'p', b'a', b'r', b's', b'l', b';',
    b'n', b'p', b'a', b'r', b't', b';',
    b'n', b'p', b'o', b'l', b'i', b'n', b't', b';',
    b'n', b'p', b'r', b';',
    b'n', b'p', b'r', b'c', b'u', b'e', b';',
// cpp: html/parser/html_entity_table.cc:1450-1481
    b'n', b'p', b'r', b'e', b';',
    b'n', b'p', b'r', b'e', b'c', b';',
    b'n', b'p', b'r', b'e', b'c', b'e', b'q', b';',
    b'n', b'r', b'A', b'r', b'r', b';',
    b'n', b'r', b'a', b'r', b'r', b';',
    b'n', b'r', b'a', b'r', b'r', b'c', b';',
    b'n', b'r', b'a', b'r', b'r', b'w', b';',
    b'n', b'r', b'i', b'g', b'h', b't', b'a', b'r', b'r', b'o', b'w', b';',
    b'n', b'r', b't', b'r', b'i', b';',
    b'n', b'r', b't', b'r', b'i', b'e', b';',
    b'n', b's', b'c', b';',
    b'n', b's', b'c', b'c', b'u', b'e', b';',
    b'n', b's', b'c', b'e', b';',
    b'n', b's', b'c', b'r', b';',
    b'n', b's', b'h', b'o', b'r', b't', b'm', b'i', b'd', b';',
    b'n', b's', b'h', b'o', b'r', b't', b'p', b'a', b'r', b'a', b'l', b'l', b'e', b'l', b';',
    b'n', b's', b'i', b'm', b'e', b';',
    b'n', b's', b'i', b'm', b'e', b'q', b';',
    b'n', b's', b'm', b'i', b'd', b';',
    b'n', b's', b'p', b'a', b'r', b';',
    b'n', b's', b'q', b's', b'u', b'b', b'e', b';',
    b'n', b's', b'q', b's', b'u', b'p', b'e', b';',
    b'n', b's', b'u', b'b', b';',
    b'n', b's', b'u', b'b', b'E', b';',
    b'n', b's', b'u', b'b', b'e', b';',
    b'n', b's', b'u', b'b', b's', b'e', b't', b';',
    b'n', b's', b'u', b'b', b's', b'e', b't', b'e', b'q', b';',
    b'n', b's', b'u', b'b', b's', b'e', b't', b'e', b'q', b'q', b';',
    b'n', b's', b'u', b'c', b'c', b';',
    b'n', b's', b'u', b'c', b'c', b'e', b'q', b';',
    b'n', b's', b'u', b'p', b';',
    b'n', b's', b'u', b'p', b'E', b';',
// cpp: html/parser/html_entity_table.cc:1482-1513
    b'n', b's', b'u', b'p', b'e', b';',
    b'n', b's', b'u', b'p', b's', b'e', b't', b';',
    b'n', b's', b'u', b'p', b's', b'e', b't', b'e', b'q', b';',
    b'n', b's', b'u', b'p', b's', b'e', b't', b'e', b'q', b'q', b';',
    b'n', b't', b'g', b'l', b';',
    b'n', b't', b'i', b'l', b'd', b'e',
    b';',
    b'n', b't', b'l', b'g', b';',
    b'n', b't', b'r', b'i', b'a', b'n', b'g', b'l', b'e', b'l', b'e', b'f', b't', b';',
    b'n', b't', b'r', b'i', b'a', b'n', b'g', b'l', b'e', b'l', b'e', b'f', b't', b'e', b'q', b';',
    b'n', b't', b'r', b'i', b'a', b'n', b'g', b'l', b'e', b'r', b'i', b'g', b'h', b't', b';',
    b'n', b't', b'r', b'i', b'a', b'n', b'g', b'l', b'e', b'r', b'i', b'g', b'h', b't', b'e', b'q', b';',
    b'n', b'u', b';',
    b'n', b'u', b'm', b';',
    b'n', b'u', b'm', b'e', b'r', b'o', b';',
    b'n', b'u', b'm', b's', b'p', b';',
    b'n', b'v', b'D', b'a', b's', b'h', b';',
    b'n', b'v', b'H', b'a', b'r', b'r', b';',
    b'n', b'v', b'a', b'p', b';',
    b'n', b'v', b'd', b'a', b's', b'h', b';',
    b'n', b'v', b'g', b'e', b';',
    b'n', b'v', b'g', b't', b';',
    b'n', b'v', b'i', b'n', b'f', b'i', b'n', b';',
    b'n', b'v', b'l', b'A', b'r', b'r', b';',
    b'n', b'v', b'l', b'e', b';',
    b'n', b'v', b'l', b't', b';',
    b'n', b'v', b'l', b't', b'r', b'i', b'e', b';',
    b'n', b'v', b'r', b'A', b'r', b'r', b';',
    b'n', b'v', b'r', b't', b'r', b'i', b'e', b';',
    b'n', b'v', b's', b'i', b'm', b';',
    b'n', b'w', b'A', b'r', b'r', b';',
    b'n', b'w', b'a', b'r', b'h', b'k', b';',
// cpp: html/parser/html_entity_table.cc:1514-1545
    b'n', b'w', b'a', b'r', b'r', b';',
    b'n', b'w', b'a', b'r', b'r', b'o', b'w', b';',
    b'n', b'w', b'n', b'e', b'a', b'r', b';',
    b'o', b'S', b';',
    b'o', b'a', b'c', b'u', b't', b'e',
    b';',
    b'o', b'a', b's', b't', b';',
    b'o', b'c', b'i', b'r', b'c',
    b';',
    b'o', b'd', b'a', b's', b'h', b';',
    b'o', b'd', b'b', b'l', b'a', b'c', b';',
    b'o', b'd', b'i', b'v', b';',
    b'o', b'd', b's', b'o', b'l', b'd', b';',
    b'o', b'e', b'l', b'i', b'g', b';',
    b'o', b'f', b'c', b'i', b'r', b';',
    b'o', b'f', b'r', b';',
    b'o', b'g', b'r', b'a', b'v', b'e',
    b';',
    b'o', b'g', b't', b';',
    b'o', b'h', b'b', b'a', b'r', b';',
    b'o', b'h', b'm', b';',
    b'o', b'i', b'n', b't', b';',
    b'o', b'l', b'a', b'r', b'r', b';',
    b'o', b'l', b'c', b'i', b'r', b';',
    b'o', b'l', b'c', b'r', b'o', b's', b's', b';',
    b'o', b'l', b'i', b'n', b'e', b';',
    b'o', b'l', b't', b';',
    b'o', b'm', b'a', b'c', b'r', b';',
    b'o', b'm', b'e', b'g', b'a', b';',
    b'o', b'm', b'i', b'c', b'r', b'o', b'n', b';',
    b'o', b'm', b'i', b'd', b';',
    b'o', b'm', b'i', b'n', b'u', b's', b';',
// cpp: html/parser/html_entity_table.cc:1546-1577
    b'o', b'o', b'p', b'f', b';',
    b'o', b'p', b'e', b'r', b'p', b';',
    b'o', b'r', b'a', b'r', b'r', b';',
    b'o', b'r', b'd', b';',
    b'o', b'r', b'd', b'e', b'r', b';',
    b'o', b'r', b'd', b'e', b'r', b'o', b'f', b';',
    b'o', b'r', b'd', b'f',
    b';',
    b'o', b'r', b'd', b'm',
    b';',
    b'o', b'r', b'i', b'g', b'o', b'f', b';',
    b'o', b'r', b'o', b'r', b';',
    b'o', b'r', b's', b'l', b'o', b'p', b'e', b';',
    b'o', b'r', b'v', b';',
    b'o', b's', b'c', b'r', b';',
    b'o', b's', b'l', b'a', b's', b'h',
    b';',
    b'o', b's', b'o', b'l', b';',
    b'o', b't', b'i', b'l', b'd', b'e',
    b';',
    b'o', b't', b'i', b'm', b'e', b's', b'a', b's', b';',
    b'o', b'u', b'm', b'l',
    b';',
    b'o', b'v', b'b', b'a', b'r', b';',
    b'p', b'a', b'r', b'a', b';',
    b'p', b'a', b'r', b's', b'i', b'm', b';',
    b'p', b'c', b'y', b';',
    b'p', b'e', b'r', b'c', b'n', b't', b';',
    b'p', b'e', b'r', b'i', b'o', b'd', b';',
    b'p', b'e', b'r', b'm', b'i', b'l', b';',
    b'p', b'e', b'r', b't', b'e', b'n', b'k', b';',
    b'p', b'f', b'r', b';',
// cpp: html/parser/html_entity_table.cc:1578-1609
    b'p', b'h', b'i', b';',
    b'p', b'h', b'i', b'v', b';',
    b'p', b'h', b'm', b'm', b'a', b't', b';',
    b'p', b'h', b'o', b'n', b'e', b';',
    b'p', b'i', b';',
    b'p', b'i', b't', b'c', b'h', b'f', b'o', b'r', b'k', b';',
    b'p', b'i', b'v', b';',
    b'p', b'l', b'a', b'n', b'c', b'k', b';',
    b'p', b'l', b'a', b'n', b'c', b'k', b'h', b';',
    b'p', b'l', b'a', b'n', b'k', b'v', b';',
    b'p', b'l', b'u', b's', b'a', b'c', b'i', b'r', b';',
    b'p', b'l', b'u', b's', b'b', b';',
    b'p', b'l', b'u', b's', b'c', b'i', b'r', b';',
    b'p', b'l', b'u', b's', b'd', b'o', b';',
    b'p', b'l', b'u', b's', b'd', b'u', b';',
    b'p', b'l', b'u', b's', b'e', b';',
    b'p', b'l', b'u', b's', b'm', b'n',
    b';',
    b'p', b'l', b'u', b's', b's', b'i', b'm', b';',
    b'p', b'l', b'u', b's', b't', b'w', b'o', b';',
    b'p', b'm', b';',
    b'p', b'o', b'i', b'n', b't', b'i', b'n', b't', b';',
    b'p', b'o', b'p', b'f', b';',
    b'p', b'o', b'u', b'n', b'd',
    b';',
    b'p', b'r', b'E', b';',
    b'p', b'r', b'a', b'p', b';',
    b'p', b'r', b'e', b'c', b'a', b'p', b'p', b'r', b'o', b'x', b';',
    b'p', b'r', b'e', b'c', b'c', b'u', b'r', b'l', b'y', b'e', b'q', b';',
    b'p', b'r', b'e', b'c', b'n', b'a', b'p', b'p', b'r', b'o', b'x', b';',
    b'p', b'r', b'e', b'c', b'n', b'e', b'q', b'q', b';',
    b'p', b'r', b'e', b'c', b'n', b's', b'i', b'm', b';',
// cpp: html/parser/html_entity_table.cc:1610-1641
    b'p', b'r', b'e', b'c', b's', b'i', b'm', b';',
    b'p', b'r', b'i', b'm', b'e', b's', b';',
    b'p', b'r', b'n', b'E', b';',
    b'p', b'r', b'n', b'a', b'p', b';',
    b'p', b'r', b'n', b's', b'i', b'm', b';',
    b'p', b'r', b'o', b'f', b'a', b'l', b'a', b'r', b';',
    b'p', b'r', b'o', b'f', b'l', b'i', b'n', b'e', b';',
    b'p', b'r', b'o', b'f', b's', b'u', b'r', b'f', b';',
    b'p', b'r', b'o', b'p', b';',
    b'p', b'r', b'o', b'p', b't', b'o', b';',
    b'p', b'r', b's', b'i', b'm', b';',
    b'p', b'r', b'u', b'r', b'e', b'l', b';',
    b'p', b's', b'c', b'r', b';',
    b'p', b'u', b'n', b'c', b's', b'p', b';',
    b'q', b'f', b'r', b';',
    b'q', b'i', b'n', b't', b';',
    b'q', b'o', b'p', b'f', b';',
    b'q', b'p', b'r', b'i', b'm', b'e', b';',
    b'q', b's', b'c', b'r', b';',
    b'q', b'u', b'a', b't', b'e', b'r', b'n', b'i', b'o', b'n', b's', b';',
    b'q', b'u', b'a', b't', b'i', b'n', b't', b';',
    b'q', b'u', b'e', b's', b't', b'e', b'q', b';',
    b'q', b'u', b'o', b't',
    b';',
    b'r', b'A', b'a', b'r', b'r', b';',
    b'r', b'A', b't', b'a', b'i', b'l', b';',
    b'r', b'B', b'a', b'r', b'r', b';',
    b'r', b'H', b'a', b'r', b';',
    b'r', b'a', b'c', b'u', b't', b'e', b';',
    b'r', b'a', b'd', b'i', b'c', b';',
    b'r', b'a', b'e', b'm', b'p', b't', b'y', b'v', b';',
    b'r', b'a', b'n', b'g', b';',
// cpp: html/parser/html_entity_table.cc:1642-1673
    b'r', b'a', b'n', b'g', b'd', b';',
    b'r', b'a', b'n', b'g', b'e', b';',
    b'r', b'a', b'n', b'g', b'l', b'e', b';',
    b'r', b'a', b'q', b'u', b'o',
    b';',
    b'r', b'a', b'r', b'r', b'a', b'p', b';',
    b'r', b'a', b'r', b'r', b'b', b';',
    b'r', b'a', b'r', b'r', b'b', b'f', b's', b';',
    b'r', b'a', b'r', b'r', b'f', b's', b';',
    b'r', b'a', b'r', b'r', b'h', b'k', b';',
    b'r', b'a', b'r', b'r', b'l', b'p', b';',
    b'r', b'a', b'r', b'r', b'p', b'l', b';',
    b'r', b'a', b'r', b'r', b's', b'i', b'm', b';',
    b'r', b'a', b'r', b'r', b't', b'l', b';',
    b'r', b'a', b't', b'a', b'i', b'l', b';',
    b'r', b'a', b't', b'i', b'o', b';',
    b'r', b'a', b't', b'i', b'o', b'n', b'a', b'l', b's', b';',
    b'r', b'b', b'a', b'r', b'r', b';',
    b'r', b'b', b'b', b'r', b'k', b';',
    b'r', b'b', b'r', b'a', b'c', b'e', b';',
    b'r', b'b', b'r', b'a', b'c', b'k', b';',
    b'r', b'b', b'r', b'k', b'e', b';',
    b'r', b'b', b'r', b'k', b's', b'l', b'd', b';',
    b'r', b'b', b'r', b'k', b's', b'l', b'u', b';',
    b'r', b'c', b'a', b'r', b'o', b'n', b';',
    b'r', b'c', b'e', b'd', b'i', b'l', b';',
    b'r', b'c', b'e', b'i', b'l', b';',
    b'r', b'c', b'u', b'b', b';',
    b'r', b'd', b'c', b'a', b';',
    b'r', b'd', b'l', b'd', b'h', b'a', b'r', b';',
    b'r', b'd', b'q', b'u', b'o', b';',
    b'r', b'd', b'q', b'u', b'o', b'r', b';',
// cpp: html/parser/html_entity_table.cc:1674-1705
    b'r', b'd', b's', b'h', b';',
    b'r', b'e', b'a', b'l', b';',
    b'r', b'e', b'a', b'l', b'i', b'n', b'e', b';',
    b'r', b'e', b'a', b'l', b'p', b'a', b'r', b't', b';',
    b'r', b'e', b'a', b'l', b's', b';',
    b'r', b'e', b'c', b't', b';',
    b'r', b'e', b'g',
    b';',
    b'r', b'f', b'i', b's', b'h', b't', b';',
    b'r', b'f', b'l', b'o', b'o', b'r', b';',
    b'r', b'f', b'r', b';',
    b'r', b'h', b'a', b'r', b'u', b';',
    b'r', b'h', b'a', b'r', b'u', b'l', b';',
    b'r', b'h', b'o', b';',
    b'r', b'h', b'o', b'v', b';',
    b'r', b'i', b'g', b'h', b't', b'a', b'r', b'r', b'o', b'w', b't', b'a', b'i', b'l', b';',
    b'r', b'i', b'g', b'h', b't', b'h', b'a', b'r', b'p', b'o', b'o', b'n', b'd', b'o', b'w', b'n', b';',
    b'r', b'i', b'g', b'h', b't', b'h', b'a', b'r', b'p', b'o', b'o', b'n', b'u', b'p', b';',
    b'r', b'i', b'g', b'h', b't', b'l', b'e', b'f', b't', b'a', b'r', b'r', b'o', b'w', b's', b';',
    b'r', b'i', b'g', b'h', b't', b'l', b'e', b'f', b't', b'h', b'a', b'r', b'p', b'o', b'o', b'n', b's', b';',
    b'r', b'i', b'g', b'h', b't', b'r', b'i', b'g', b'h', b't', b'a', b'r', b'r', b'o', b'w', b's', b';',
    b'r', b'i', b'g', b'h', b't', b't', b'h', b'r', b'e', b'e', b't', b'i', b'm', b'e', b's', b';',
    b'r', b'i', b's', b'i', b'n', b'g', b'd', b'o', b't', b's', b'e', b'q', b';',
    b'r', b'l', b'a', b'r', b'r', b';',
    b'r', b'l', b'h', b'a', b'r', b';',
    b'r', b'l', b'm', b';',
    b'r', b'm', b'o', b'u', b's', b't', b';',
    b'r', b'm', b'o', b'u', b's', b't', b'a', b'c', b'h', b'e', b';',
    b'r', b'n', b'm', b'i', b'd', b';',
    b'r', b'o', b'a', b'n', b'g', b';',
    b'r', b'o', b'a', b'r', b'r', b';',
    b'r', b'o', b'b', b'r', b'k', b';',
// cpp: html/parser/html_entity_table.cc:1706-1737
    b'r', b'o', b'p', b'a', b'r', b';',
    b'r', b'o', b'p', b'f', b';',
    b'r', b'o', b'p', b'l', b'u', b's', b';',
    b'r', b'o', b't', b'i', b'm', b'e', b's', b';',
    b'r', b'p', b'a', b'r', b';',
    b'r', b'p', b'a', b'r', b'g', b't', b';',
    b'r', b'p', b'p', b'o', b'l', b'i', b'n', b't', b';',
    b'r', b'r', b'a', b'r', b'r', b';',
    b'r', b's', b'a', b'q', b'u', b'o', b';',
    b'r', b's', b'c', b'r', b';',
    b'r', b's', b'h', b';',
    b'r', b's', b'q', b'b', b';',
    b'r', b's', b'q', b'u', b'o', b';',
    b'r', b's', b'q', b'u', b'o', b'r', b';',
    b'r', b't', b'h', b'r', b'e', b'e', b';',
    b'r', b't', b'i', b'm', b'e', b's', b';',
    b'r', b't', b'r', b'i', b'f', b';',
    b'r', b't', b'r', b'i', b'l', b't', b'r', b'i', b';',
    b'r', b'u', b'l', b'u', b'h', b'a', b'r', b';',
    b'r', b'x', b';',
    b's', b'a', b'c', b'u', b't', b'e', b';',
    b's', b'b', b'q', b'u', b'o', b';',
    b's', b'c', b'E', b';',
    b's', b'c', b'a', b'p', b';',
    b's', b'c', b'a', b'r', b'o', b'n', b';',
    b's', b'c', b'e', b'd', b'i', b'l', b';',
    b's', b'c', b'i', b'r', b'c', b';',
    b's', b'c', b'n', b'E', b';',
    b's', b'c', b'n', b'a', b'p', b';',
    b's', b'c', b'n', b's', b'i', b'm', b';',
    b's', b'c', b'p', b'o', b'l', b'i', b'n', b't', b';',
    b's', b'c', b's', b'i', b'm', b';',
// cpp: html/parser/html_entity_table.cc:1738-1769
    b's', b'd', b'o', b't', b'b', b';',
    b's', b'd', b'o', b't', b'e', b';',
    b's', b'e', b'A', b'r', b'r', b';',
    b's', b'e', b'a', b'r', b'h', b'k', b';',
    b's', b'e', b'a', b'r', b'r', b';',
    b's', b'e', b'a', b'r', b'r', b'o', b'w', b';',
    b's', b'e', b'c', b't', b';',
    b's', b'e', b's', b'w', b'a', b'r', b';',
    b's', b'e', b't', b'm', b'i', b'n', b'u', b's', b';',
    b's', b'e', b't', b'm', b'n', b';',
    b's', b'e', b'x', b't', b';',
    b's', b'f', b'r', b';',
    b's', b'f', b'r', b'o', b'w', b'n', b';',
    b's', b'h', b'a', b'r', b'p', b';',
    b's', b'h', b'c', b'h', b'c', b'y', b';',
    b's', b'h', b'c', b'y', b';',
    b's', b'h', b'y',
    b';',
    b's', b'i', b'g', b'm', b'a', b';',
    b's', b'i', b'g', b'm', b'a', b'f', b';',
    b's', b'i', b'g', b'm', b'a', b'v', b';',
    b's', b'i', b'm', b'd', b'o', b't', b';',
    b's', b'i', b'm', b'g', b'E', b';',
    b's', b'i', b'm', b'l', b'E', b';',
    b's', b'i', b'm', b'n', b'e', b';',
    b's', b'i', b'm', b'p', b'l', b'u', b's', b';',
    b's', b'i', b'm', b'r', b'a', b'r', b'r', b';',
    b's', b'l', b'a', b'r', b'r', b';',
    b's', b'm', b'a', b'l', b'l', b's', b'e', b't', b'm', b'i', b'n', b'u', b's', b';',
    b's', b'm', b'a', b's', b'h', b'p', b';',
    b's', b'm', b'e', b'p', b'a', b'r', b's', b'l', b';',
    b's', b'm', b'i', b'l', b'e', b';',
// cpp: html/parser/html_entity_table.cc:1770-1801
    b's', b'm', b't', b';',
    b's', b'm', b't', b'e', b';',
    b's', b'm', b't', b'e', b's', b';',
    b's', b'o', b'f', b't', b'c', b'y', b';',
    b's', b'o', b'l', b'b', b'a', b'r', b';',
    b's', b'o', b'p', b'f', b';',
    b's', b'p', b'a', b'd', b'e', b's', b';',
    b's', b'p', b'a', b'd', b'e', b's', b'u', b'i', b't', b';',
    b's', b'q', b'c', b'a', b'p', b';',
    b's', b'q', b'c', b'a', b'p', b's', b';',
    b's', b'q', b'c', b'u', b'p', b's', b';',
    b's', b'q', b's', b'u', b'b', b';',
    b's', b'q', b's', b'u', b'b', b's', b'e', b't', b';',
    b's', b'q', b's', b'u', b'b', b's', b'e', b't', b'e', b'q', b';',
    b's', b'q', b's', b'u', b'p', b';',
    b's', b'q', b's', b'u', b'p', b's', b'e', b't', b';',
    b's', b'q', b's', b'u', b'p', b's', b'e', b't', b'e', b'q', b';',
    b's', b'q', b'u', b';',
    b's', b'q', b'u', b'a', b'r', b'f', b';',
    b's', b'q', b'u', b'f', b';',
    b's', b'r', b'a', b'r', b'r', b';',
    b's', b's', b'c', b'r', b';',
    b's', b's', b'e', b't', b'm', b'n', b';',
    b's', b's', b'm', b'i', b'l', b'e', b';',
    b's', b's', b't', b'a', b'r', b'f', b';',
    b's', b't', b'r', b'a', b'i', b'g', b'h', b't', b'e', b'p', b's', b'i', b'l', b'o', b'n', b';',
    b's', b't', b'r', b'a', b'i', b'g', b'h', b't', b'p', b'h', b'i', b';',
    b's', b't', b'r', b'n', b's', b';',
    b's', b'u', b'b', b'd', b'o', b't', b';',
    b's', b'u', b'b', b'e', b'd', b'o', b't', b';',
    b's', b'u', b'b', b'm', b'u', b'l', b't', b';',
    b's', b'u', b'b', b'n', b'E', b';',
// cpp: html/parser/html_entity_table.cc:1802-1833
    b's', b'u', b'b', b'n', b'e', b';',
    b's', b'u', b'b', b'p', b'l', b'u', b's', b';',
    b's', b'u', b'b', b'r', b'a', b'r', b'r', b';',
    b's', b'u', b'b', b's', b'e', b't', b'n', b'e', b'q', b';',
    b's', b'u', b'b', b's', b'e', b't', b'n', b'e', b'q', b'q', b';',
    b's', b'u', b'b', b's', b'i', b'm', b';',
    b's', b'u', b'b', b's', b'u', b'b', b';',
    b's', b'u', b'b', b's', b'u', b'p', b';',
    b's', b'u', b'c', b'c', b'a', b'p', b'p', b'r', b'o', b'x', b';',
    b's', b'u', b'c', b'c', b'c', b'u', b'r', b'l', b'y', b'e', b'q', b';',
    b's', b'u', b'c', b'c', b'n', b'a', b'p', b'p', b'r', b'o', b'x', b';',
    b's', b'u', b'c', b'c', b'n', b'e', b'q', b'q', b';',
    b's', b'u', b'c', b'c', b'n', b's', b'i', b'm', b';',
    b's', b'u', b'c', b'c', b's', b'i', b'm', b';',
    b's', b'u', b'm', b';',
    b's', b'u', b'n', b'g', b';',
    b's', b'u', b'p', b'1',
    b';',
    b's', b'u', b'p', b'2',
    b';',
    b's', b'u', b'p', b'3',
    b';',
    b's', b'u', b'p', b'd', b'o', b't', b';',
    b's', b'u', b'p', b'd', b's', b'u', b'b', b';',
    b's', b'u', b'p', b'e', b'd', b'o', b't', b';',
    b's', b'u', b'p', b'h', b's', b'o', b'l', b';',
    b's', b'u', b'p', b'h', b's', b'u', b'b', b';',
    b's', b'u', b'p', b'l', b'a', b'r', b'r', b';',
    b's', b'u', b'p', b'm', b'u', b'l', b't', b';',
    b's', b'u', b'p', b'n', b'E', b';',
    b's', b'u', b'p', b'n', b'e', b';',
    b's', b'u', b'p', b'p', b'l', b'u', b's', b';',
// cpp: html/parser/html_entity_table.cc:1834-1865
    b's', b'u', b'p', b's', b'e', b't', b'n', b'e', b'q', b';',
    b's', b'u', b'p', b's', b'e', b't', b'n', b'e', b'q', b'q', b';',
    b's', b'u', b'p', b's', b'i', b'm', b';',
    b's', b'u', b'p', b's', b'u', b'b', b';',
    b's', b'u', b'p', b's', b'u', b'p', b';',
    b's', b'w', b'A', b'r', b'r', b';',
    b's', b'w', b'a', b'r', b'h', b'k', b';',
    b's', b'w', b'a', b'r', b'r', b';',
    b's', b'w', b'a', b'r', b'r', b'o', b'w', b';',
    b's', b'w', b'n', b'w', b'a', b'r', b';',
    b's', b'z', b'l', b'i', b'g',
    b';',
    b't', b'a', b'r', b'g', b'e', b't', b';',
    b't', b'a', b'u', b';',
    b't', b'c', b'a', b'r', b'o', b'n', b';',
    b't', b'c', b'e', b'd', b'i', b'l', b';',
    b't', b'e', b'l', b'r', b'e', b'c', b';',
    b't', b'f', b'r', b';',
    b't', b'h', b'e', b'r', b'e', b'4', b';',
    b't', b'h', b'e', b'r', b'e', b'f', b'o', b'r', b'e', b';',
    b't', b'h', b'e', b't', b'a', b';',
    b't', b'h', b'e', b't', b'a', b's', b'y', b'm', b';',
    b't', b'h', b'e', b't', b'a', b'v', b';',
    b't', b'h', b'i', b'c', b'k', b'a', b'p', b'p', b'r', b'o', b'x', b';',
    b't', b'h', b'i', b'c', b'k', b's', b'i', b'm', b';',
    b't', b'h', b'i', b'n', b's', b'p', b';',
    b't', b'h', b'k', b'a', b'p', b';',
    b't', b'h', b'k', b's', b'i', b'm', b';',
    b't', b'h', b'o', b'r', b'n',
    b';',
    b't', b'i', b'm', b'e', b's', b'b', b';',
    b't', b'i', b'm', b'e', b's', b'b', b'a', b'r', b';',
// cpp: html/parser/html_entity_table.cc:1866-1897
    b't', b'i', b'm', b'e', b's', b'd', b';',
    b't', b'o', b'e', b'a', b';',
    b't', b'o', b'p', b';',
    b't', b'o', b'p', b'b', b'o', b't', b';',
    b't', b'o', b'p', b'c', b'i', b'r', b';',
    b't', b'o', b'p', b'f', b';',
    b't', b'o', b'p', b'f', b'o', b'r', b'k', b';',
    b't', b'o', b's', b'a', b';',
    b't', b'p', b'r', b'i', b'm', b'e', b';',
    b't', b'r', b'a', b'd', b'e', b';',
    b't', b'r', b'i', b'a', b'n', b'g', b'l', b'e', b'q', b';',
    b't', b'r', b'i', b'd', b'o', b't', b';',
    b't', b'r', b'i', b'm', b'i', b'n', b'u', b's', b';',
    b't', b'r', b'i', b'p', b'l', b'u', b's', b';',
    b't', b'r', b'i', b's', b'b', b';',
    b't', b'r', b'i', b't', b'i', b'm', b'e', b';',
    b't', b'r', b'p', b'e', b'z', b'i', b'u', b'm', b';',
    b't', b's', b'c', b'r', b';',
    b't', b's', b'c', b'y', b';',
    b't', b's', b'h', b'c', b'y', b';',
    b't', b's', b't', b'r', b'o', b'k', b';',
    b't', b'w', b'i', b'x', b't', b';',
    b't', b'w', b'o', b'h', b'e', b'a', b'd', b'l', b'e', b'f', b't', b'a', b'r', b'r', b'o', b'w', b';',
    b't', b'w', b'o', b'h', b'e', b'a', b'd', b'r', b'i', b'g', b'h', b't', b'a', b'r', b'r', b'o', b'w', b';',
    b'u', b'A', b'r', b'r', b';',
    b'u', b'H', b'a', b'r', b';',
    b'u', b'a', b'c', b'u', b't', b'e',
    b';',
    b'u', b'b', b'r', b'c', b'y', b';',
    b'u', b'b', b'r', b'e', b'v', b'e', b';',
    b'u', b'c', b'i', b'r', b'c',
    b';',
// cpp: html/parser/html_entity_table.cc:1898-1929
    b'u', b'c', b'y', b';',
    b'u', b'd', b'a', b'r', b'r', b';',
    b'u', b'd', b'b', b'l', b'a', b'c', b';',
    b'u', b'd', b'h', b'a', b'r', b';',
    b'u', b'f', b'i', b's', b'h', b't', b';',
    b'u', b'f', b'r', b';',
    b'u', b'g', b'r', b'a', b'v', b'e',
    b';',
    b'u', b'h', b'a', b'r', b'l', b';',
    b'u', b'h', b'a', b'r', b'r', b';',
    b'u', b'h', b'b', b'l', b'k', b';',
    b'u', b'l', b'c', b'o', b'r', b'n', b';',
    b'u', b'l', b'c', b'o', b'r', b'n', b'e', b'r', b';',
    b'u', b'l', b'c', b'r', b'o', b'p', b';',
    b'u', b'l', b't', b'r', b'i', b';',
    b'u', b'm', b'a', b'c', b'r', b';',
    b'u', b'o', b'g', b'o', b'n', b';',
    b'u', b'o', b'p', b'f', b';',
    b'u', b'p', b'a', b'r', b'r', b'o', b'w', b';',
    b'u', b'p', b'd', b'o', b'w', b'n', b'a', b'r', b'r', b'o', b'w', b';',
    b'u', b'p', b'h', b'a', b'r', b'p', b'o', b'o', b'n', b'l', b'e', b'f', b't', b';',
    b'u', b'p', b'h', b'a', b'r', b'p', b'o', b'o', b'n', b'r', b'i', b'g', b'h', b't', b';',
    b'u', b'p', b's', b'i', b';',
    b'u', b'p', b's', b'i', b'h', b';',
    b'u', b'p', b's', b'i', b'l', b'o', b'n', b';',
    b'u', b'p', b'u', b'p', b'a', b'r', b'r', b'o', b'w', b's', b';',
    b'u', b'r', b'c', b'o', b'r', b'n', b';',
    b'u', b'r', b'c', b'o', b'r', b'n', b'e', b'r', b';',
    b'u', b'r', b'c', b'r', b'o', b'p', b';',
    b'u', b'r', b'i', b'n', b'g', b';',
    b'u', b'r', b't', b'r', b'i', b';',
    b'u', b's', b'c', b'r', b';',
// cpp: html/parser/html_entity_table.cc:1930-1961
    b'u', b't', b'd', b'o', b't', b';',
    b'u', b't', b'i', b'l', b'd', b'e', b';',
    b'u', b't', b'r', b'i', b';',
    b'u', b't', b'r', b'i', b'f', b';',
    b'u', b'u', b'a', b'r', b'r', b';',
    b'u', b'u', b'm', b'l',
    b';',
    b'u', b'w', b'a', b'n', b'g', b'l', b'e', b';',
    b'v', b'A', b'r', b'r', b';',
    b'v', b'B', b'a', b'r', b';',
    b'v', b'B', b'a', b'r', b'v', b';',
    b'v', b'a', b'n', b'g', b'r', b't', b';',
    b'v', b'a', b'r', b'e', b'p', b's', b'i', b'l', b'o', b'n', b';',
    b'v', b'a', b'r', b'k', b'a', b'p', b'p', b'a', b';',
    b'v', b'a', b'r', b'n', b'o', b't', b'h', b'i', b'n', b'g', b';',
    b'v', b'a', b'r', b'p', b'h', b'i', b';',
    b'v', b'a', b'r', b'p', b'i', b';',
    b'v', b'a', b'r', b'p', b'r', b'o', b'p', b't', b'o', b';',
    b'v', b'a', b'r', b'r', b';',
    b'v', b'a', b'r', b'r', b'h', b'o', b';',
    b'v', b'a', b'r', b's', b'i', b'g', b'm', b'a', b';',
    b'v', b'a', b'r', b's', b'u', b'b', b's', b'e', b't', b'n', b'e', b'q', b';',
    b'v', b'a', b'r', b's', b'u', b'b', b's', b'e', b't', b'n', b'e', b'q', b'q', b';',
    b'v', b'a', b'r', b's', b'u', b'p', b's', b'e', b't', b'n', b'e', b'q', b';',
    b'v', b'a', b'r', b's', b'u', b'p', b's', b'e', b't', b'n', b'e', b'q', b'q', b';',
    b'v', b'a', b'r', b't', b'h', b'e', b't', b'a', b';',
    b'v', b'a', b'r', b't', b'r', b'i', b'a', b'n', b'g', b'l', b'e', b'l', b'e', b'f', b't', b';',
    b'v', b'a', b'r', b't', b'r', b'i', b'a', b'n', b'g', b'l', b'e', b'r', b'i', b'g', b'h', b't', b';',
    b'v', b'c', b'y', b';',
    b'v', b'e', b'e', b'b', b'a', b'r', b';',
    b'v', b'e', b'e', b'e', b'q', b';',
    b'v', b'e', b'l', b'l', b'i', b'p', b';',
// cpp: html/parser/html_entity_table.cc:1962-1993
    b'v', b'e', b'r', b'b', b'a', b'r', b';',
    b'v', b'e', b'r', b't', b';',
    b'v', b'f', b'r', b';',
    b'v', b'l', b't', b'r', b'i', b';',
    b'v', b'n', b's', b'u', b'b', b';',
    b'v', b'n', b's', b'u', b'p', b';',
    b'v', b'o', b'p', b'f', b';',
    b'v', b'p', b'r', b'o', b'p', b';',
    b'v', b'r', b't', b'r', b'i', b';',
    b'v', b's', b'c', b'r', b';',
    b'v', b's', b'u', b'b', b'n', b'E', b';',
    b'v', b's', b'u', b'b', b'n', b'e', b';',
    b'v', b's', b'u', b'p', b'n', b'E', b';',
    b'v', b's', b'u', b'p', b'n', b'e', b';',
    b'v', b'z', b'i', b'g', b'z', b'a', b'g', b';',
    b'w', b'c', b'i', b'r', b'c', b';',
    b'w', b'e', b'd', b'b', b'a', b'r', b';',
    b'w', b'e', b'd', b'g', b'e', b'q', b';',
    b'w', b'e', b'i', b'e', b'r', b'p', b';',
    b'w', b'f', b'r', b';',
    b'w', b'o', b'p', b'f', b';',
    b'w', b'p', b';',
    b'w', b'r', b';',
    b'w', b'r', b'e', b'a', b't', b'h', b';',
    b'w', b's', b'c', b'r', b';',
    b'x', b'c', b'a', b'p', b';',
    b'x', b'c', b'i', b'r', b'c', b';',
    b'x', b'c', b'u', b'p', b';',
    b'x', b'd', b't', b'r', b'i', b';',
    b'x', b'f', b'r', b';',
    b'x', b'h', b'A', b'r', b'r', b';',
    b'x', b'h', b'a', b'r', b'r', b';',
// cpp: html/parser/html_entity_table.cc:1994-2025
    b'x', b'i', b';',
    b'x', b'l', b'A', b'r', b'r', b';',
    b'x', b'l', b'a', b'r', b'r', b';',
    b'x', b'm', b'a', b'p', b';',
    b'x', b'n', b'i', b's', b';',
    b'x', b'o', b'd', b'o', b't', b';',
    b'x', b'o', b'p', b'f', b';',
    b'x', b'o', b'p', b'l', b'u', b's', b';',
    b'x', b'o', b't', b'i', b'm', b'e', b';',
    b'x', b'r', b'A', b'r', b'r', b';',
    b'x', b'r', b'a', b'r', b'r', b';',
    b'x', b's', b'c', b'r', b';',
    b'x', b's', b'q', b'c', b'u', b'p', b';',
    b'x', b'u', b'p', b'l', b'u', b's', b';',
    b'x', b'u', b't', b'r', b'i', b';',
    b'x', b'v', b'e', b'e', b';',
    b'x', b'w', b'e', b'd', b'g', b'e', b';',
    b'y', b'a', b'c', b'u', b't', b'e',
    b';',
    b'y', b'a', b'c', b'y', b';',
    b'y', b'c', b'i', b'r', b'c', b';',
    b'y', b'c', b'y', b';',
    b'y', b'e', b'n',
    b';',
    b'y', b'f', b'r', b';',
    b'y', b'i', b'c', b'y', b';',
    b'y', b'o', b'p', b'f', b';',
    b'y', b's', b'c', b'r', b';',
    b'y', b'u', b'c', b'y', b';',
    b'y', b'u', b'm', b'l',
    b';',
    b'z', b'a', b'c', b'u', b't', b'e', b';',
// cpp: html/parser/html_entity_table.cc:2026-2035
    b'z', b'c', b'a', b'r', b'o', b'n', b';',
    b'z', b'd', b'o', b't', b';',
    b'z', b'e', b'e', b't', b'r', b'f', b';',
    b'z', b'e', b't', b'a', b';',
    b'z', b'f', b'r', b';',
    b'z', b'h', b'c', b'y', b';',
    b'z', b'o', b'p', b'f', b';',
    b'z', b's', b'c', b'r', b';',
    b'z', b'w', b'j', b';',
    b'z', b'w', b'n', b'j', b';',
];

// cpp: html/parser/html_entity_table.cc:2043-2043
#[rustfmt::skip]
const kStaticEntityTable: [HTMLEntityTableEntry; 2231] = [
// cpp: html/parser/html_entity_table.cc:2044-2075
    HTMLEntityTableEntry { first_value: 0x000C6, second_value: 0x0000, entity_offset: 0, length: 5 }, // &AElig
    HTMLEntityTableEntry { first_value: 0x000C6, second_value: 0x0000, entity_offset: 0, length: 6 }, // &AElig;
    HTMLEntityTableEntry { first_value: 0x00026, second_value: 0x0000, entity_offset: 6, length: 3 }, // &AMP
    HTMLEntityTableEntry { first_value: 0x00026, second_value: 0x0000, entity_offset: 6, length: 4 }, // &AMP;
    HTMLEntityTableEntry { first_value: 0x000C1, second_value: 0x0000, entity_offset: 10, length: 6 }, // &Aacute
    HTMLEntityTableEntry { first_value: 0x000C1, second_value: 0x0000, entity_offset: 10, length: 7 }, // &Aacute;
    HTMLEntityTableEntry { first_value: 0x00102, second_value: 0x0000, entity_offset: 17, length: 7 }, // &Abreve;
    HTMLEntityTableEntry { first_value: 0x000C2, second_value: 0x0000, entity_offset: 24, length: 5 }, // &Acirc
    HTMLEntityTableEntry { first_value: 0x000C2, second_value: 0x0000, entity_offset: 24, length: 6 }, // &Acirc;
    HTMLEntityTableEntry { first_value: 0x00410, second_value: 0x0000, entity_offset: 30, length: 4 }, // &Acy;
    HTMLEntityTableEntry { first_value: 0x1D504, second_value: 0x0000, entity_offset: 34, length: 4 }, // &Afr;
    HTMLEntityTableEntry { first_value: 0x000C0, second_value: 0x0000, entity_offset: 38, length: 6 }, // &Agrave
    HTMLEntityTableEntry { first_value: 0x000C0, second_value: 0x0000, entity_offset: 38, length: 7 }, // &Agrave;
    HTMLEntityTableEntry { first_value: 0x00391, second_value: 0x0000, entity_offset: 45, length: 6 }, // &Alpha;
    HTMLEntityTableEntry { first_value: 0x00100, second_value: 0x0000, entity_offset: 51, length: 6 }, // &Amacr;
    HTMLEntityTableEntry { first_value: 0x02A53, second_value: 0x0000, entity_offset: 57, length: 4 }, // &And;
    HTMLEntityTableEntry { first_value: 0x00104, second_value: 0x0000, entity_offset: 61, length: 6 }, // &Aogon;
    HTMLEntityTableEntry { first_value: 0x1D538, second_value: 0x0000, entity_offset: 67, length: 5 }, // &Aopf;
    HTMLEntityTableEntry { first_value: 0x02061, second_value: 0x0000, entity_offset: 72, length: 14 }, // &ApplyFunction;
    HTMLEntityTableEntry { first_value: 0x000C5, second_value: 0x0000, entity_offset: 86, length: 5 }, // &Aring
    HTMLEntityTableEntry { first_value: 0x000C5, second_value: 0x0000, entity_offset: 86, length: 6 }, // &Aring;
    HTMLEntityTableEntry { first_value: 0x1D49C, second_value: 0x0000, entity_offset: 92, length: 5 }, // &Ascr;
    HTMLEntityTableEntry { first_value: 0x02254, second_value: 0x0000, entity_offset: 97, length: 7 }, // &Assign;
    HTMLEntityTableEntry { first_value: 0x000C3, second_value: 0x0000, entity_offset: 104, length: 6 }, // &Atilde
    HTMLEntityTableEntry { first_value: 0x000C3, second_value: 0x0000, entity_offset: 104, length: 7 }, // &Atilde;
    HTMLEntityTableEntry { first_value: 0x000C4, second_value: 0x0000, entity_offset: 111, length: 4 }, // &Auml
    HTMLEntityTableEntry { first_value: 0x000C4, second_value: 0x0000, entity_offset: 111, length: 5 }, // &Auml;
    HTMLEntityTableEntry { first_value: 0x02216, second_value: 0x0000, entity_offset: 116, length: 10 }, // &Backslash;
    HTMLEntityTableEntry { first_value: 0x02AE7, second_value: 0x0000, entity_offset: 126, length: 5 }, // &Barv;
    HTMLEntityTableEntry { first_value: 0x02306, second_value: 0x0000, entity_offset: 131, length: 7 }, // &Barwed;
    HTMLEntityTableEntry { first_value: 0x00411, second_value: 0x0000, entity_offset: 138, length: 4 }, // &Bcy;
    HTMLEntityTableEntry { first_value: 0x02235, second_value: 0x0000, entity_offset: 142, length: 8 }, // &Because;
// cpp: html/parser/html_entity_table.cc:2076-2107
    HTMLEntityTableEntry { first_value: 0x0212C, second_value: 0x0000, entity_offset: 150, length: 11 }, // &Bernoullis;
    HTMLEntityTableEntry { first_value: 0x00392, second_value: 0x0000, entity_offset: 161, length: 5 }, // &Beta;
    HTMLEntityTableEntry { first_value: 0x1D505, second_value: 0x0000, entity_offset: 166, length: 4 }, // &Bfr;
    HTMLEntityTableEntry { first_value: 0x1D539, second_value: 0x0000, entity_offset: 170, length: 5 }, // &Bopf;
    HTMLEntityTableEntry { first_value: 0x002D8, second_value: 0x0000, entity_offset: 175, length: 6 }, // &Breve;
    HTMLEntityTableEntry { first_value: 0x0212C, second_value: 0x0000, entity_offset: 181, length: 5 }, // &Bscr;
    HTMLEntityTableEntry { first_value: 0x0224E, second_value: 0x0000, entity_offset: 186, length: 7 }, // &Bumpeq;
    HTMLEntityTableEntry { first_value: 0x00427, second_value: 0x0000, entity_offset: 193, length: 5 }, // &CHcy;
    HTMLEntityTableEntry { first_value: 0x000A9, second_value: 0x0000, entity_offset: 198, length: 4 }, // &COPY
    HTMLEntityTableEntry { first_value: 0x000A9, second_value: 0x0000, entity_offset: 198, length: 5 }, // &COPY;
    HTMLEntityTableEntry { first_value: 0x00106, second_value: 0x0000, entity_offset: 203, length: 7 }, // &Cacute;
    HTMLEntityTableEntry { first_value: 0x022D2, second_value: 0x0000, entity_offset: 210, length: 4 }, // &Cap;
    HTMLEntityTableEntry { first_value: 0x02145, second_value: 0x0000, entity_offset: 214, length: 21 }, // &CapitalDifferentialD;
    HTMLEntityTableEntry { first_value: 0x0212D, second_value: 0x0000, entity_offset: 235, length: 8 }, // &Cayleys;
    HTMLEntityTableEntry { first_value: 0x0010C, second_value: 0x0000, entity_offset: 243, length: 7 }, // &Ccaron;
    HTMLEntityTableEntry { first_value: 0x000C7, second_value: 0x0000, entity_offset: 250, length: 6 }, // &Ccedil
    HTMLEntityTableEntry { first_value: 0x000C7, second_value: 0x0000, entity_offset: 250, length: 7 }, // &Ccedil;
    HTMLEntityTableEntry { first_value: 0x00108, second_value: 0x0000, entity_offset: 257, length: 6 }, // &Ccirc;
    HTMLEntityTableEntry { first_value: 0x02230, second_value: 0x0000, entity_offset: 263, length: 8 }, // &Cconint;
    HTMLEntityTableEntry { first_value: 0x0010A, second_value: 0x0000, entity_offset: 271, length: 5 }, // &Cdot;
    HTMLEntityTableEntry { first_value: 0x000B8, second_value: 0x0000, entity_offset: 276, length: 8 }, // &Cedilla;
    HTMLEntityTableEntry { first_value: 0x000B7, second_value: 0x0000, entity_offset: 284, length: 10 }, // &CenterDot;
    HTMLEntityTableEntry { first_value: 0x0212D, second_value: 0x0000, entity_offset: 294, length: 4 }, // &Cfr;
    HTMLEntityTableEntry { first_value: 0x003A7, second_value: 0x0000, entity_offset: 298, length: 4 }, // &Chi;
    HTMLEntityTableEntry { first_value: 0x02299, second_value: 0x0000, entity_offset: 302, length: 10 }, // &CircleDot;
    HTMLEntityTableEntry { first_value: 0x02296, second_value: 0x0000, entity_offset: 312, length: 12 }, // &CircleMinus;
    HTMLEntityTableEntry { first_value: 0x02295, second_value: 0x0000, entity_offset: 324, length: 11 }, // &CirclePlus;
    HTMLEntityTableEntry { first_value: 0x02297, second_value: 0x0000, entity_offset: 335, length: 12 }, // &CircleTimes;
    HTMLEntityTableEntry { first_value: 0x02232, second_value: 0x0000, entity_offset: 347, length: 25 }, // &ClockwiseContourIntegral;
    HTMLEntityTableEntry { first_value: 0x0201D, second_value: 0x0000, entity_offset: 372, length: 22 }, // &CloseCurlyDoubleQuote;
    HTMLEntityTableEntry { first_value: 0x02019, second_value: 0x0000, entity_offset: 394, length: 16 }, // &CloseCurlyQuote;
    HTMLEntityTableEntry { first_value: 0x02237, second_value: 0x0000, entity_offset: 410, length: 6 }, // &Colon;
// cpp: html/parser/html_entity_table.cc:2108-2139
    HTMLEntityTableEntry { first_value: 0x02A74, second_value: 0x0000, entity_offset: 416, length: 7 }, // &Colone;
    HTMLEntityTableEntry { first_value: 0x02261, second_value: 0x0000, entity_offset: 423, length: 10 }, // &Congruent;
    HTMLEntityTableEntry { first_value: 0x0222F, second_value: 0x0000, entity_offset: 433, length: 7 }, // &Conint;
    HTMLEntityTableEntry { first_value: 0x0222E, second_value: 0x0000, entity_offset: 356, length: 16 }, // &ContourIntegral;
    HTMLEntityTableEntry { first_value: 0x02102, second_value: 0x0000, entity_offset: 440, length: 5 }, // &Copf;
    HTMLEntityTableEntry { first_value: 0x02210, second_value: 0x0000, entity_offset: 445, length: 10 }, // &Coproduct;
    HTMLEntityTableEntry { first_value: 0x02233, second_value: 0x0000, entity_offset: 455, length: 32 }, // &CounterClockwiseContourIntegral;
    HTMLEntityTableEntry { first_value: 0x02A2F, second_value: 0x0000, entity_offset: 487, length: 6 }, // &Cross;
    HTMLEntityTableEntry { first_value: 0x1D49E, second_value: 0x0000, entity_offset: 493, length: 5 }, // &Cscr;
    HTMLEntityTableEntry { first_value: 0x022D3, second_value: 0x0000, entity_offset: 498, length: 4 }, // &Cup;
    HTMLEntityTableEntry { first_value: 0x0224D, second_value: 0x0000, entity_offset: 502, length: 7 }, // &CupCap;
    HTMLEntityTableEntry { first_value: 0x02145, second_value: 0x0000, entity_offset: 509, length: 3 }, // &DD;
    HTMLEntityTableEntry { first_value: 0x02911, second_value: 0x0000, entity_offset: 512, length: 9 }, // &DDotrahd;
    HTMLEntityTableEntry { first_value: 0x00402, second_value: 0x0000, entity_offset: 521, length: 5 }, // &DJcy;
    HTMLEntityTableEntry { first_value: 0x00405, second_value: 0x0000, entity_offset: 526, length: 5 }, // &DScy;
    HTMLEntityTableEntry { first_value: 0x0040F, second_value: 0x0000, entity_offset: 531, length: 5 }, // &DZcy;
    HTMLEntityTableEntry { first_value: 0x02021, second_value: 0x0000, entity_offset: 536, length: 7 }, // &Dagger;
    HTMLEntityTableEntry { first_value: 0x021A1, second_value: 0x0000, entity_offset: 543, length: 5 }, // &Darr;
    HTMLEntityTableEntry { first_value: 0x02AE4, second_value: 0x0000, entity_offset: 548, length: 6 }, // &Dashv;
    HTMLEntityTableEntry { first_value: 0x0010E, second_value: 0x0000, entity_offset: 554, length: 7 }, // &Dcaron;
    HTMLEntityTableEntry { first_value: 0x00414, second_value: 0x0000, entity_offset: 561, length: 4 }, // &Dcy;
    HTMLEntityTableEntry { first_value: 0x02207, second_value: 0x0000, entity_offset: 565, length: 4 }, // &Del;
    HTMLEntityTableEntry { first_value: 0x00394, second_value: 0x0000, entity_offset: 569, length: 6 }, // &Delta;
    HTMLEntityTableEntry { first_value: 0x1D507, second_value: 0x0000, entity_offset: 575, length: 4 }, // &Dfr;
    HTMLEntityTableEntry { first_value: 0x000B4, second_value: 0x0000, entity_offset: 579, length: 17 }, // &DiacriticalAcute;
    HTMLEntityTableEntry { first_value: 0x002D9, second_value: 0x0000, entity_offset: 596, length: 15 }, // &DiacriticalDot;
    HTMLEntityTableEntry { first_value: 0x002DD, second_value: 0x0000, entity_offset: 611, length: 23 }, // &DiacriticalDoubleAcute;
    HTMLEntityTableEntry { first_value: 0x00060, second_value: 0x0000, entity_offset: 634, length: 17 }, // &DiacriticalGrave;
    HTMLEntityTableEntry { first_value: 0x002DC, second_value: 0x0000, entity_offset: 651, length: 17 }, // &DiacriticalTilde;
    HTMLEntityTableEntry { first_value: 0x022C4, second_value: 0x0000, entity_offset: 668, length: 8 }, // &Diamond;
    HTMLEntityTableEntry { first_value: 0x02146, second_value: 0x0000, entity_offset: 221, length: 14 }, // &DifferentialD;
    HTMLEntityTableEntry { first_value: 0x1D53B, second_value: 0x0000, entity_offset: 676, length: 5 }, // &Dopf;
// cpp: html/parser/html_entity_table.cc:2140-2171
    HTMLEntityTableEntry { first_value: 0x000A8, second_value: 0x0000, entity_offset: 290, length: 4 }, // &Dot;
    HTMLEntityTableEntry { first_value: 0x020DC, second_value: 0x0000, entity_offset: 681, length: 7 }, // &DotDot;
    HTMLEntityTableEntry { first_value: 0x02250, second_value: 0x0000, entity_offset: 688, length: 9 }, // &DotEqual;
    HTMLEntityTableEntry { first_value: 0x0222F, second_value: 0x0000, entity_offset: 697, length: 22 }, // &DoubleContourIntegral;
    HTMLEntityTableEntry { first_value: 0x000A8, second_value: 0x0000, entity_offset: 719, length: 10 }, // &DoubleDot;
    HTMLEntityTableEntry { first_value: 0x021D3, second_value: 0x0000, entity_offset: 729, length: 16 }, // &DoubleDownArrow;
    HTMLEntityTableEntry { first_value: 0x021D0, second_value: 0x0000, entity_offset: 745, length: 16 }, // &DoubleLeftArrow;
    HTMLEntityTableEntry { first_value: 0x021D4, second_value: 0x0000, entity_offset: 761, length: 21 }, // &DoubleLeftRightArrow;
    HTMLEntityTableEntry { first_value: 0x02AE4, second_value: 0x0000, entity_offset: 782, length: 14 }, // &DoubleLeftTee;
    HTMLEntityTableEntry { first_value: 0x027F8, second_value: 0x0000, entity_offset: 796, length: 20 }, // &DoubleLongLeftArrow;
    HTMLEntityTableEntry { first_value: 0x027FA, second_value: 0x0000, entity_offset: 816, length: 25 }, // &DoubleLongLeftRightArrow;
    HTMLEntityTableEntry { first_value: 0x027F9, second_value: 0x0000, entity_offset: 841, length: 21 }, // &DoubleLongRightArrow;
    HTMLEntityTableEntry { first_value: 0x021D2, second_value: 0x0000, entity_offset: 862, length: 17 }, // &DoubleRightArrow;
    HTMLEntityTableEntry { first_value: 0x022A8, second_value: 0x0000, entity_offset: 879, length: 15 }, // &DoubleRightTee;
    HTMLEntityTableEntry { first_value: 0x021D1, second_value: 0x0000, entity_offset: 894, length: 14 }, // &DoubleUpArrow;
    HTMLEntityTableEntry { first_value: 0x021D5, second_value: 0x0000, entity_offset: 908, length: 18 }, // &DoubleUpDownArrow;
    HTMLEntityTableEntry { first_value: 0x02225, second_value: 0x0000, entity_offset: 926, length: 18 }, // &DoubleVerticalBar;
    HTMLEntityTableEntry { first_value: 0x02193, second_value: 0x0000, entity_offset: 735, length: 10 }, // &DownArrow;
    HTMLEntityTableEntry { first_value: 0x02913, second_value: 0x0000, entity_offset: 944, length: 13 }, // &DownArrowBar;
    HTMLEntityTableEntry { first_value: 0x021F5, second_value: 0x0000, entity_offset: 957, length: 17 }, // &DownArrowUpArrow;
    HTMLEntityTableEntry { first_value: 0x00311, second_value: 0x0000, entity_offset: 974, length: 10 }, // &DownBreve;
    HTMLEntityTableEntry { first_value: 0x02950, second_value: 0x0000, entity_offset: 984, length: 20 }, // &DownLeftRightVector;
    HTMLEntityTableEntry { first_value: 0x0295E, second_value: 0x0000, entity_offset: 1004, length: 18 }, // &DownLeftTeeVector;
    HTMLEntityTableEntry { first_value: 0x021BD, second_value: 0x0000, entity_offset: 1022, length: 15 }, // &DownLeftVector;
    HTMLEntityTableEntry { first_value: 0x02956, second_value: 0x0000, entity_offset: 1037, length: 18 }, // &DownLeftVectorBar;
    HTMLEntityTableEntry { first_value: 0x0295F, second_value: 0x0000, entity_offset: 1055, length: 19 }, // &DownRightTeeVector;
    HTMLEntityTableEntry { first_value: 0x021C1, second_value: 0x0000, entity_offset: 1074, length: 16 }, // &DownRightVector;
    HTMLEntityTableEntry { first_value: 0x02957, second_value: 0x0000, entity_offset: 1090, length: 19 }, // &DownRightVectorBar;
    HTMLEntityTableEntry { first_value: 0x022A4, second_value: 0x0000, entity_offset: 1109, length: 8 }, // &DownTee;
    HTMLEntityTableEntry { first_value: 0x021A7, second_value: 0x0000, entity_offset: 1117, length: 13 }, // &DownTeeArrow;
    HTMLEntityTableEntry { first_value: 0x021D3, second_value: 0x0000, entity_offset: 1130, length: 10 }, // &Downarrow;
    HTMLEntityTableEntry { first_value: 0x1D49F, second_value: 0x0000, entity_offset: 1140, length: 5 }, // &Dscr;
// cpp: html/parser/html_entity_table.cc:2172-2203
    HTMLEntityTableEntry { first_value: 0x00110, second_value: 0x0000, entity_offset: 1145, length: 7 }, // &Dstrok;
    HTMLEntityTableEntry { first_value: 0x0014A, second_value: 0x0000, entity_offset: 1152, length: 4 }, // &ENG;
    HTMLEntityTableEntry { first_value: 0x000D0, second_value: 0x0000, entity_offset: 1156, length: 3 }, // &ETH
    HTMLEntityTableEntry { first_value: 0x000D0, second_value: 0x0000, entity_offset: 1156, length: 4 }, // &ETH;
    HTMLEntityTableEntry { first_value: 0x000C9, second_value: 0x0000, entity_offset: 1160, length: 6 }, // &Eacute
    HTMLEntityTableEntry { first_value: 0x000C9, second_value: 0x0000, entity_offset: 1160, length: 7 }, // &Eacute;
    HTMLEntityTableEntry { first_value: 0x0011A, second_value: 0x0000, entity_offset: 1167, length: 7 }, // &Ecaron;
    HTMLEntityTableEntry { first_value: 0x000CA, second_value: 0x0000, entity_offset: 1174, length: 5 }, // &Ecirc
    HTMLEntityTableEntry { first_value: 0x000CA, second_value: 0x0000, entity_offset: 1174, length: 6 }, // &Ecirc;
    HTMLEntityTableEntry { first_value: 0x0042D, second_value: 0x0000, entity_offset: 1180, length: 4 }, // &Ecy;
    HTMLEntityTableEntry { first_value: 0x00116, second_value: 0x0000, entity_offset: 1184, length: 5 }, // &Edot;
    HTMLEntityTableEntry { first_value: 0x1D508, second_value: 0x0000, entity_offset: 1189, length: 4 }, // &Efr;
    HTMLEntityTableEntry { first_value: 0x000C8, second_value: 0x0000, entity_offset: 1193, length: 6 }, // &Egrave
    HTMLEntityTableEntry { first_value: 0x000C8, second_value: 0x0000, entity_offset: 1193, length: 7 }, // &Egrave;
    HTMLEntityTableEntry { first_value: 0x02208, second_value: 0x0000, entity_offset: 1200, length: 8 }, // &Element;
    HTMLEntityTableEntry { first_value: 0x00112, second_value: 0x0000, entity_offset: 1208, length: 6 }, // &Emacr;
    HTMLEntityTableEntry { first_value: 0x025FB, second_value: 0x0000, entity_offset: 1214, length: 17 }, // &EmptySmallSquare;
    HTMLEntityTableEntry { first_value: 0x025AB, second_value: 0x0000, entity_offset: 1231, length: 21 }, // &EmptyVerySmallSquare;
    HTMLEntityTableEntry { first_value: 0x00118, second_value: 0x0000, entity_offset: 1252, length: 6 }, // &Eogon;
    HTMLEntityTableEntry { first_value: 0x1D53C, second_value: 0x0000, entity_offset: 1258, length: 5 }, // &Eopf;
    HTMLEntityTableEntry { first_value: 0x00395, second_value: 0x0000, entity_offset: 1263, length: 8 }, // &Epsilon;
    HTMLEntityTableEntry { first_value: 0x02A75, second_value: 0x0000, entity_offset: 691, length: 6 }, // &Equal;
    HTMLEntityTableEntry { first_value: 0x02242, second_value: 0x0000, entity_offset: 1271, length: 11 }, // &EqualTilde;
    HTMLEntityTableEntry { first_value: 0x021CC, second_value: 0x0000, entity_offset: 1282, length: 12 }, // &Equilibrium;
    HTMLEntityTableEntry { first_value: 0x02130, second_value: 0x0000, entity_offset: 1294, length: 5 }, // &Escr;
    HTMLEntityTableEntry { first_value: 0x02A73, second_value: 0x0000, entity_offset: 1299, length: 5 }, // &Esim;
    HTMLEntityTableEntry { first_value: 0x00397, second_value: 0x0000, entity_offset: 1304, length: 4 }, // &Eta;
    HTMLEntityTableEntry { first_value: 0x000CB, second_value: 0x0000, entity_offset: 1308, length: 4 }, // &Euml
    HTMLEntityTableEntry { first_value: 0x000CB, second_value: 0x0000, entity_offset: 1308, length: 5 }, // &Euml;
    HTMLEntityTableEntry { first_value: 0x02203, second_value: 0x0000, entity_offset: 1313, length: 7 }, // &Exists;
    HTMLEntityTableEntry { first_value: 0x02147, second_value: 0x0000, entity_offset: 1320, length: 13 }, // &ExponentialE;
    HTMLEntityTableEntry { first_value: 0x00424, second_value: 0x0000, entity_offset: 1333, length: 4 }, // &Fcy;
// cpp: html/parser/html_entity_table.cc:2204-2235
    HTMLEntityTableEntry { first_value: 0x1D509, second_value: 0x0000, entity_offset: 1337, length: 4 }, // &Ffr;
    HTMLEntityTableEntry { first_value: 0x025FC, second_value: 0x0000, entity_offset: 1341, length: 18 }, // &FilledSmallSquare;
    HTMLEntityTableEntry { first_value: 0x025AA, second_value: 0x0000, entity_offset: 1359, length: 22 }, // &FilledVerySmallSquare;
    HTMLEntityTableEntry { first_value: 0x1D53D, second_value: 0x0000, entity_offset: 1381, length: 5 }, // &Fopf;
    HTMLEntityTableEntry { first_value: 0x02200, second_value: 0x0000, entity_offset: 1386, length: 7 }, // &ForAll;
    HTMLEntityTableEntry { first_value: 0x02131, second_value: 0x0000, entity_offset: 1393, length: 11 }, // &Fouriertrf;
    HTMLEntityTableEntry { first_value: 0x02131, second_value: 0x0000, entity_offset: 1404, length: 5 }, // &Fscr;
    HTMLEntityTableEntry { first_value: 0x00403, second_value: 0x0000, entity_offset: 1409, length: 5 }, // &GJcy;
    HTMLEntityTableEntry { first_value: 0x0003E, second_value: 0x0000, entity_offset: 1414, length: 2 }, // &GT
    HTMLEntityTableEntry { first_value: 0x0003E, second_value: 0x0000, entity_offset: 1414, length: 3 }, // &GT;
    HTMLEntityTableEntry { first_value: 0x00393, second_value: 0x0000, entity_offset: 1417, length: 6 }, // &Gamma;
    HTMLEntityTableEntry { first_value: 0x003DC, second_value: 0x0000, entity_offset: 1423, length: 7 }, // &Gammad;
    HTMLEntityTableEntry { first_value: 0x0011E, second_value: 0x0000, entity_offset: 1430, length: 7 }, // &Gbreve;
    HTMLEntityTableEntry { first_value: 0x00122, second_value: 0x0000, entity_offset: 1437, length: 7 }, // &Gcedil;
    HTMLEntityTableEntry { first_value: 0x0011C, second_value: 0x0000, entity_offset: 1444, length: 6 }, // &Gcirc;
    HTMLEntityTableEntry { first_value: 0x00413, second_value: 0x0000, entity_offset: 1450, length: 4 }, // &Gcy;
    HTMLEntityTableEntry { first_value: 0x00120, second_value: 0x0000, entity_offset: 1454, length: 5 }, // &Gdot;
    HTMLEntityTableEntry { first_value: 0x1D50A, second_value: 0x0000, entity_offset: 1459, length: 4 }, // &Gfr;
    HTMLEntityTableEntry { first_value: 0x022D9, second_value: 0x0000, entity_offset: 1463, length: 3 }, // &Gg;
    HTMLEntityTableEntry { first_value: 0x1D53E, second_value: 0x0000, entity_offset: 1466, length: 5 }, // &Gopf;
    HTMLEntityTableEntry { first_value: 0x02265, second_value: 0x0000, entity_offset: 1471, length: 13 }, // &GreaterEqual;
    HTMLEntityTableEntry { first_value: 0x022DB, second_value: 0x0000, entity_offset: 1484, length: 17 }, // &GreaterEqualLess;
    HTMLEntityTableEntry { first_value: 0x02267, second_value: 0x0000, entity_offset: 1501, length: 17 }, // &GreaterFullEqual;
    HTMLEntityTableEntry { first_value: 0x02AA2, second_value: 0x0000, entity_offset: 1518, length: 15 }, // &GreaterGreater;
    HTMLEntityTableEntry { first_value: 0x02277, second_value: 0x0000, entity_offset: 1533, length: 12 }, // &GreaterLess;
    HTMLEntityTableEntry { first_value: 0x02A7E, second_value: 0x0000, entity_offset: 1545, length: 18 }, // &GreaterSlantEqual;
    HTMLEntityTableEntry { first_value: 0x02273, second_value: 0x0000, entity_offset: 1563, length: 13 }, // &GreaterTilde;
    HTMLEntityTableEntry { first_value: 0x1D4A2, second_value: 0x0000, entity_offset: 1576, length: 5 }, // &Gscr;
    HTMLEntityTableEntry { first_value: 0x0226B, second_value: 0x0000, entity_offset: 1581, length: 3 }, // &Gt;
    HTMLEntityTableEntry { first_value: 0x0042A, second_value: 0x0000, entity_offset: 1584, length: 7 }, // &HARDcy;
    HTMLEntityTableEntry { first_value: 0x002C7, second_value: 0x0000, entity_offset: 1591, length: 6 }, // &Hacek;
    HTMLEntityTableEntry { first_value: 0x0005E, second_value: 0x0000, entity_offset: 1597, length: 4 }, // &Hat;
// cpp: html/parser/html_entity_table.cc:2236-2267
    HTMLEntityTableEntry { first_value: 0x00124, second_value: 0x0000, entity_offset: 1601, length: 6 }, // &Hcirc;
    HTMLEntityTableEntry { first_value: 0x0210C, second_value: 0x0000, entity_offset: 1607, length: 4 }, // &Hfr;
    HTMLEntityTableEntry { first_value: 0x0210B, second_value: 0x0000, entity_offset: 1611, length: 13 }, // &HilbertSpace;
    HTMLEntityTableEntry { first_value: 0x0210D, second_value: 0x0000, entity_offset: 1624, length: 5 }, // &Hopf;
    HTMLEntityTableEntry { first_value: 0x02500, second_value: 0x0000, entity_offset: 1629, length: 15 }, // &HorizontalLine;
    HTMLEntityTableEntry { first_value: 0x0210B, second_value: 0x0000, entity_offset: 1644, length: 5 }, // &Hscr;
    HTMLEntityTableEntry { first_value: 0x00126, second_value: 0x0000, entity_offset: 1649, length: 7 }, // &Hstrok;
    HTMLEntityTableEntry { first_value: 0x0224E, second_value: 0x0000, entity_offset: 1656, length: 13 }, // &HumpDownHump;
    HTMLEntityTableEntry { first_value: 0x0224F, second_value: 0x0000, entity_offset: 1669, length: 10 }, // &HumpEqual;
    HTMLEntityTableEntry { first_value: 0x00415, second_value: 0x0000, entity_offset: 1679, length: 5 }, // &IEcy;
    HTMLEntityTableEntry { first_value: 0x00132, second_value: 0x0000, entity_offset: 1684, length: 6 }, // &IJlig;
    HTMLEntityTableEntry { first_value: 0x00401, second_value: 0x0000, entity_offset: 1690, length: 5 }, // &IOcy;
    HTMLEntityTableEntry { first_value: 0x000CD, second_value: 0x0000, entity_offset: 1695, length: 6 }, // &Iacute
    HTMLEntityTableEntry { first_value: 0x000CD, second_value: 0x0000, entity_offset: 1695, length: 7 }, // &Iacute;
    HTMLEntityTableEntry { first_value: 0x000CE, second_value: 0x0000, entity_offset: 1702, length: 5 }, // &Icirc
    HTMLEntityTableEntry { first_value: 0x000CE, second_value: 0x0000, entity_offset: 1702, length: 6 }, // &Icirc;
    HTMLEntityTableEntry { first_value: 0x00418, second_value: 0x0000, entity_offset: 1708, length: 4 }, // &Icy;
    HTMLEntityTableEntry { first_value: 0x00130, second_value: 0x0000, entity_offset: 1712, length: 5 }, // &Idot;
    HTMLEntityTableEntry { first_value: 0x02111, second_value: 0x0000, entity_offset: 1717, length: 4 }, // &Ifr;
    HTMLEntityTableEntry { first_value: 0x000CC, second_value: 0x0000, entity_offset: 1721, length: 6 }, // &Igrave
    HTMLEntityTableEntry { first_value: 0x000CC, second_value: 0x0000, entity_offset: 1721, length: 7 }, // &Igrave;
    HTMLEntityTableEntry { first_value: 0x02111, second_value: 0x0000, entity_offset: 1728, length: 3 }, // &Im;
    HTMLEntityTableEntry { first_value: 0x0012A, second_value: 0x0000, entity_offset: 1731, length: 6 }, // &Imacr;
    HTMLEntityTableEntry { first_value: 0x02148, second_value: 0x0000, entity_offset: 1737, length: 11 }, // &ImaginaryI;
    HTMLEntityTableEntry { first_value: 0x021D2, second_value: 0x0000, entity_offset: 1748, length: 8 }, // &Implies;
    HTMLEntityTableEntry { first_value: 0x0222C, second_value: 0x0000, entity_offset: 1756, length: 4 }, // &Int;
    HTMLEntityTableEntry { first_value: 0x0222B, second_value: 0x0000, entity_offset: 363, length: 9 }, // &Integral;
    HTMLEntityTableEntry { first_value: 0x022C2, second_value: 0x0000, entity_offset: 1760, length: 13 }, // &Intersection;
    HTMLEntityTableEntry { first_value: 0x02063, second_value: 0x0000, entity_offset: 1773, length: 15 }, // &InvisibleComma;
    HTMLEntityTableEntry { first_value: 0x02062, second_value: 0x0000, entity_offset: 1788, length: 15 }, // &InvisibleTimes;
    HTMLEntityTableEntry { first_value: 0x0012E, second_value: 0x0000, entity_offset: 1803, length: 6 }, // &Iogon;
    HTMLEntityTableEntry { first_value: 0x1D540, second_value: 0x0000, entity_offset: 1809, length: 5 }, // &Iopf;
// cpp: html/parser/html_entity_table.cc:2268-2299
    HTMLEntityTableEntry { first_value: 0x00399, second_value: 0x0000, entity_offset: 1814, length: 5 }, // &Iota;
    HTMLEntityTableEntry { first_value: 0x02110, second_value: 0x0000, entity_offset: 1819, length: 5 }, // &Iscr;
    HTMLEntityTableEntry { first_value: 0x00128, second_value: 0x0000, entity_offset: 1824, length: 7 }, // &Itilde;
    HTMLEntityTableEntry { first_value: 0x00406, second_value: 0x0000, entity_offset: 1831, length: 6 }, // &Iukcy;
    HTMLEntityTableEntry { first_value: 0x000CF, second_value: 0x0000, entity_offset: 1837, length: 4 }, // &Iuml
    HTMLEntityTableEntry { first_value: 0x000CF, second_value: 0x0000, entity_offset: 1837, length: 5 }, // &Iuml;
    HTMLEntityTableEntry { first_value: 0x00134, second_value: 0x0000, entity_offset: 1842, length: 6 }, // &Jcirc;
    HTMLEntityTableEntry { first_value: 0x00419, second_value: 0x0000, entity_offset: 522, length: 4 }, // &Jcy;
    HTMLEntityTableEntry { first_value: 0x1D50D, second_value: 0x0000, entity_offset: 1848, length: 4 }, // &Jfr;
    HTMLEntityTableEntry { first_value: 0x1D541, second_value: 0x0000, entity_offset: 1852, length: 5 }, // &Jopf;
    HTMLEntityTableEntry { first_value: 0x1D4A5, second_value: 0x0000, entity_offset: 1857, length: 5 }, // &Jscr;
    HTMLEntityTableEntry { first_value: 0x00408, second_value: 0x0000, entity_offset: 1862, length: 7 }, // &Jsercy;
    HTMLEntityTableEntry { first_value: 0x00404, second_value: 0x0000, entity_offset: 1869, length: 6 }, // &Jukcy;
    HTMLEntityTableEntry { first_value: 0x00425, second_value: 0x0000, entity_offset: 1875, length: 5 }, // &KHcy;
    HTMLEntityTableEntry { first_value: 0x0040C, second_value: 0x0000, entity_offset: 1880, length: 5 }, // &KJcy;
    HTMLEntityTableEntry { first_value: 0x0039A, second_value: 0x0000, entity_offset: 1885, length: 6 }, // &Kappa;
    HTMLEntityTableEntry { first_value: 0x00136, second_value: 0x0000, entity_offset: 1891, length: 7 }, // &Kcedil;
    HTMLEntityTableEntry { first_value: 0x0041A, second_value: 0x0000, entity_offset: 1898, length: 4 }, // &Kcy;
    HTMLEntityTableEntry { first_value: 0x1D50E, second_value: 0x0000, entity_offset: 1902, length: 4 }, // &Kfr;
    HTMLEntityTableEntry { first_value: 0x1D542, second_value: 0x0000, entity_offset: 1906, length: 5 }, // &Kopf;
    HTMLEntityTableEntry { first_value: 0x1D4A6, second_value: 0x0000, entity_offset: 1911, length: 5 }, // &Kscr;
    HTMLEntityTableEntry { first_value: 0x00409, second_value: 0x0000, entity_offset: 1916, length: 5 }, // &LJcy;
    HTMLEntityTableEntry { first_value: 0x0003C, second_value: 0x0000, entity_offset: 1921, length: 2 }, // &LT
    HTMLEntityTableEntry { first_value: 0x0003C, second_value: 0x0000, entity_offset: 1921, length: 3 }, // &LT;
    HTMLEntityTableEntry { first_value: 0x00139, second_value: 0x0000, entity_offset: 1924, length: 7 }, // &Lacute;
    HTMLEntityTableEntry { first_value: 0x0039B, second_value: 0x0000, entity_offset: 1931, length: 7 }, // &Lambda;
    HTMLEntityTableEntry { first_value: 0x027EA, second_value: 0x0000, entity_offset: 1938, length: 5 }, // &Lang;
    HTMLEntityTableEntry { first_value: 0x02112, second_value: 0x0000, entity_offset: 1943, length: 11 }, // &Laplacetrf;
    HTMLEntityTableEntry { first_value: 0x0219E, second_value: 0x0000, entity_offset: 1954, length: 5 }, // &Larr;
    HTMLEntityTableEntry { first_value: 0x0013D, second_value: 0x0000, entity_offset: 1959, length: 7 }, // &Lcaron;
    HTMLEntityTableEntry { first_value: 0x0013B, second_value: 0x0000, entity_offset: 1966, length: 7 }, // &Lcedil;
    HTMLEntityTableEntry { first_value: 0x0041B, second_value: 0x0000, entity_offset: 1973, length: 4 }, // &Lcy;
// cpp: html/parser/html_entity_table.cc:2300-2331
    HTMLEntityTableEntry { first_value: 0x027E8, second_value: 0x0000, entity_offset: 1977, length: 17 }, // &LeftAngleBracket;
    HTMLEntityTableEntry { first_value: 0x02190, second_value: 0x0000, entity_offset: 751, length: 10 }, // &LeftArrow;
    HTMLEntityTableEntry { first_value: 0x021E4, second_value: 0x0000, entity_offset: 1994, length: 13 }, // &LeftArrowBar;
    HTMLEntityTableEntry { first_value: 0x021C6, second_value: 0x0000, entity_offset: 2007, length: 20 }, // &LeftArrowRightArrow;
    HTMLEntityTableEntry { first_value: 0x02308, second_value: 0x0000, entity_offset: 2027, length: 12 }, // &LeftCeiling;
    HTMLEntityTableEntry { first_value: 0x027E6, second_value: 0x0000, entity_offset: 2039, length: 18 }, // &LeftDoubleBracket;
    HTMLEntityTableEntry { first_value: 0x02961, second_value: 0x0000, entity_offset: 2057, length: 18 }, // &LeftDownTeeVector;
    HTMLEntityTableEntry { first_value: 0x021C3, second_value: 0x0000, entity_offset: 2075, length: 15 }, // &LeftDownVector;
    HTMLEntityTableEntry { first_value: 0x02959, second_value: 0x0000, entity_offset: 2090, length: 18 }, // &LeftDownVectorBar;
    HTMLEntityTableEntry { first_value: 0x0230A, second_value: 0x0000, entity_offset: 2108, length: 10 }, // &LeftFloor;
    HTMLEntityTableEntry { first_value: 0x02194, second_value: 0x0000, entity_offset: 767, length: 15 }, // &LeftRightArrow;
    HTMLEntityTableEntry { first_value: 0x0294E, second_value: 0x0000, entity_offset: 988, length: 16 }, // &LeftRightVector;
    HTMLEntityTableEntry { first_value: 0x022A3, second_value: 0x0000, entity_offset: 788, length: 8 }, // &LeftTee;
    HTMLEntityTableEntry { first_value: 0x021A4, second_value: 0x0000, entity_offset: 2118, length: 13 }, // &LeftTeeArrow;
    HTMLEntityTableEntry { first_value: 0x0295A, second_value: 0x0000, entity_offset: 1008, length: 14 }, // &LeftTeeVector;
    HTMLEntityTableEntry { first_value: 0x022B2, second_value: 0x0000, entity_offset: 2131, length: 13 }, // &LeftTriangle;
    HTMLEntityTableEntry { first_value: 0x029CF, second_value: 0x0000, entity_offset: 2144, length: 16 }, // &LeftTriangleBar;
    HTMLEntityTableEntry { first_value: 0x022B4, second_value: 0x0000, entity_offset: 2160, length: 18 }, // &LeftTriangleEqual;
    HTMLEntityTableEntry { first_value: 0x02951, second_value: 0x0000, entity_offset: 2178, length: 17 }, // &LeftUpDownVector;
    HTMLEntityTableEntry { first_value: 0x02960, second_value: 0x0000, entity_offset: 2195, length: 16 }, // &LeftUpTeeVector;
    HTMLEntityTableEntry { first_value: 0x021BF, second_value: 0x0000, entity_offset: 2211, length: 13 }, // &LeftUpVector;
    HTMLEntityTableEntry { first_value: 0x02958, second_value: 0x0000, entity_offset: 2224, length: 16 }, // &LeftUpVectorBar;
    HTMLEntityTableEntry { first_value: 0x021BC, second_value: 0x0000, entity_offset: 1026, length: 11 }, // &LeftVector;
    HTMLEntityTableEntry { first_value: 0x02952, second_value: 0x0000, entity_offset: 1041, length: 14 }, // &LeftVectorBar;
    HTMLEntityTableEntry { first_value: 0x021D0, second_value: 0x0000, entity_offset: 2240, length: 10 }, // &Leftarrow;
    HTMLEntityTableEntry { first_value: 0x021D4, second_value: 0x0000, entity_offset: 2250, length: 15 }, // &Leftrightarrow;
    HTMLEntityTableEntry { first_value: 0x022DA, second_value: 0x0000, entity_offset: 2265, length: 17 }, // &LessEqualGreater;
    HTMLEntityTableEntry { first_value: 0x02266, second_value: 0x0000, entity_offset: 2282, length: 14 }, // &LessFullEqual;
    HTMLEntityTableEntry { first_value: 0x02276, second_value: 0x0000, entity_offset: 2296, length: 12 }, // &LessGreater;
    HTMLEntityTableEntry { first_value: 0x02AA1, second_value: 0x0000, entity_offset: 2308, length: 9 }, // &LessLess;
    HTMLEntityTableEntry { first_value: 0x02A7D, second_value: 0x0000, entity_offset: 2317, length: 15 }, // &LessSlantEqual;
    HTMLEntityTableEntry { first_value: 0x02272, second_value: 0x0000, entity_offset: 2332, length: 10 }, // &LessTilde;
// cpp: html/parser/html_entity_table.cc:2332-2363
    HTMLEntityTableEntry { first_value: 0x1D50F, second_value: 0x0000, entity_offset: 2342, length: 4 }, // &Lfr;
    HTMLEntityTableEntry { first_value: 0x022D8, second_value: 0x0000, entity_offset: 2346, length: 3 }, // &Ll;
    HTMLEntityTableEntry { first_value: 0x021DA, second_value: 0x0000, entity_offset: 2349, length: 11 }, // &Lleftarrow;
    HTMLEntityTableEntry { first_value: 0x0013F, second_value: 0x0000, entity_offset: 2360, length: 7 }, // &Lmidot;
    HTMLEntityTableEntry { first_value: 0x027F5, second_value: 0x0000, entity_offset: 802, length: 14 }, // &LongLeftArrow;
    HTMLEntityTableEntry { first_value: 0x027F7, second_value: 0x0000, entity_offset: 822, length: 19 }, // &LongLeftRightArrow;
    HTMLEntityTableEntry { first_value: 0x027F6, second_value: 0x0000, entity_offset: 847, length: 15 }, // &LongRightArrow;
    HTMLEntityTableEntry { first_value: 0x027F8, second_value: 0x0000, entity_offset: 2367, length: 14 }, // &Longleftarrow;
    HTMLEntityTableEntry { first_value: 0x027FA, second_value: 0x0000, entity_offset: 2381, length: 19 }, // &Longleftrightarrow;
    HTMLEntityTableEntry { first_value: 0x027F9, second_value: 0x0000, entity_offset: 2400, length: 15 }, // &Longrightarrow;
    HTMLEntityTableEntry { first_value: 0x1D543, second_value: 0x0000, entity_offset: 2415, length: 5 }, // &Lopf;
    HTMLEntityTableEntry { first_value: 0x02199, second_value: 0x0000, entity_offset: 2420, length: 15 }, // &LowerLeftArrow;
    HTMLEntityTableEntry { first_value: 0x02198, second_value: 0x0000, entity_offset: 2435, length: 16 }, // &LowerRightArrow;
    HTMLEntityTableEntry { first_value: 0x02112, second_value: 0x0000, entity_offset: 2451, length: 5 }, // &Lscr;
    HTMLEntityTableEntry { first_value: 0x021B0, second_value: 0x0000, entity_offset: 2456, length: 4 }, // &Lsh;
    HTMLEntityTableEntry { first_value: 0x00141, second_value: 0x0000, entity_offset: 2460, length: 7 }, // &Lstrok;
    HTMLEntityTableEntry { first_value: 0x0226A, second_value: 0x0000, entity_offset: 2467, length: 3 }, // &Lt;
    HTMLEntityTableEntry { first_value: 0x02905, second_value: 0x0000, entity_offset: 2470, length: 4 }, // &Map;
    HTMLEntityTableEntry { first_value: 0x0041C, second_value: 0x0000, entity_offset: 2474, length: 4 }, // &Mcy;
    HTMLEntityTableEntry { first_value: 0x0205F, second_value: 0x0000, entity_offset: 2478, length: 12 }, // &MediumSpace;
    HTMLEntityTableEntry { first_value: 0x02133, second_value: 0x0000, entity_offset: 2490, length: 10 }, // &Mellintrf;
    HTMLEntityTableEntry { first_value: 0x1D510, second_value: 0x0000, entity_offset: 2500, length: 4 }, // &Mfr;
    HTMLEntityTableEntry { first_value: 0x02213, second_value: 0x0000, entity_offset: 2504, length: 10 }, // &MinusPlus;
    HTMLEntityTableEntry { first_value: 0x1D544, second_value: 0x0000, entity_offset: 2514, length: 5 }, // &Mopf;
    HTMLEntityTableEntry { first_value: 0x02133, second_value: 0x0000, entity_offset: 2519, length: 5 }, // &Mscr;
    HTMLEntityTableEntry { first_value: 0x0039C, second_value: 0x0000, entity_offset: 2524, length: 3 }, // &Mu;
    HTMLEntityTableEntry { first_value: 0x0040A, second_value: 0x0000, entity_offset: 2527, length: 5 }, // &NJcy;
    HTMLEntityTableEntry { first_value: 0x00143, second_value: 0x0000, entity_offset: 2532, length: 7 }, // &Nacute;
    HTMLEntityTableEntry { first_value: 0x00147, second_value: 0x0000, entity_offset: 2539, length: 7 }, // &Ncaron;
    HTMLEntityTableEntry { first_value: 0x00145, second_value: 0x0000, entity_offset: 2546, length: 7 }, // &Ncedil;
    HTMLEntityTableEntry { first_value: 0x0041D, second_value: 0x0000, entity_offset: 2553, length: 4 }, // &Ncy;
    HTMLEntityTableEntry { first_value: 0x0200B, second_value: 0x0000, entity_offset: 2557, length: 20 }, // &NegativeMediumSpace;
// cpp: html/parser/html_entity_table.cc:2364-2395
    HTMLEntityTableEntry { first_value: 0x0200B, second_value: 0x0000, entity_offset: 2577, length: 19 }, // &NegativeThickSpace;
    HTMLEntityTableEntry { first_value: 0x0200B, second_value: 0x0000, entity_offset: 2596, length: 18 }, // &NegativeThinSpace;
    HTMLEntityTableEntry { first_value: 0x0200B, second_value: 0x0000, entity_offset: 2614, length: 22 }, // &NegativeVeryThinSpace;
    HTMLEntityTableEntry { first_value: 0x0226B, second_value: 0x0000, entity_offset: 2636, length: 21 }, // &NestedGreaterGreater;
    HTMLEntityTableEntry { first_value: 0x0226A, second_value: 0x0000, entity_offset: 2657, length: 15 }, // &NestedLessLess;
    HTMLEntityTableEntry { first_value: 0x0000A, second_value: 0x0000, entity_offset: 2672, length: 8 }, // &NewLine;
    HTMLEntityTableEntry { first_value: 0x1D511, second_value: 0x0000, entity_offset: 2680, length: 4 }, // &Nfr;
    HTMLEntityTableEntry { first_value: 0x02060, second_value: 0x0000, entity_offset: 2684, length: 8 }, // &NoBreak;
    HTMLEntityTableEntry { first_value: 0x000A0, second_value: 0x0000, entity_offset: 2692, length: 17 }, // &NonBreakingSpace;
    HTMLEntityTableEntry { first_value: 0x02115, second_value: 0x0000, entity_offset: 2709, length: 5 }, // &Nopf;
    HTMLEntityTableEntry { first_value: 0x02AEC, second_value: 0x0000, entity_offset: 2714, length: 4 }, // &Not;
    HTMLEntityTableEntry { first_value: 0x02262, second_value: 0x0000, entity_offset: 2718, length: 13 }, // &NotCongruent;
    HTMLEntityTableEntry { first_value: 0x0226D, second_value: 0x0000, entity_offset: 2731, length: 10 }, // &NotCupCap;
    HTMLEntityTableEntry { first_value: 0x02226, second_value: 0x0000, entity_offset: 2741, length: 21 }, // &NotDoubleVerticalBar;
    HTMLEntityTableEntry { first_value: 0x02209, second_value: 0x0000, entity_offset: 2762, length: 11 }, // &NotElement;
    HTMLEntityTableEntry { first_value: 0x02260, second_value: 0x0000, entity_offset: 2773, length: 9 }, // &NotEqual;
    HTMLEntityTableEntry { first_value: 0x02242, second_value: 0x0338, entity_offset: 2782, length: 14 }, // &NotEqualTilde;
    HTMLEntityTableEntry { first_value: 0x02204, second_value: 0x0000, entity_offset: 2796, length: 10 }, // &NotExists;
    HTMLEntityTableEntry { first_value: 0x0226F, second_value: 0x0000, entity_offset: 2806, length: 11 }, // &NotGreater;
    HTMLEntityTableEntry { first_value: 0x02271, second_value: 0x0000, entity_offset: 2817, length: 16 }, // &NotGreaterEqual;
    HTMLEntityTableEntry { first_value: 0x02267, second_value: 0x0338, entity_offset: 2833, length: 20 }, // &NotGreaterFullEqual;
    HTMLEntityTableEntry { first_value: 0x0226B, second_value: 0x0338, entity_offset: 2853, length: 18 }, // &NotGreaterGreater;
    HTMLEntityTableEntry { first_value: 0x02279, second_value: 0x0000, entity_offset: 2871, length: 15 }, // &NotGreaterLess;
    HTMLEntityTableEntry { first_value: 0x02A7E, second_value: 0x0338, entity_offset: 2886, length: 21 }, // &NotGreaterSlantEqual;
    HTMLEntityTableEntry { first_value: 0x02275, second_value: 0x0000, entity_offset: 2907, length: 16 }, // &NotGreaterTilde;
    HTMLEntityTableEntry { first_value: 0x0224E, second_value: 0x0338, entity_offset: 2923, length: 16 }, // &NotHumpDownHump;
    HTMLEntityTableEntry { first_value: 0x0224F, second_value: 0x0338, entity_offset: 2939, length: 13 }, // &NotHumpEqual;
    HTMLEntityTableEntry { first_value: 0x022EA, second_value: 0x0000, entity_offset: 2952, length: 16 }, // &NotLeftTriangle;
    HTMLEntityTableEntry { first_value: 0x029CF, second_value: 0x0338, entity_offset: 2968, length: 19 }, // &NotLeftTriangleBar;
    HTMLEntityTableEntry { first_value: 0x022EC, second_value: 0x0000, entity_offset: 2987, length: 21 }, // &NotLeftTriangleEqual;
    HTMLEntityTableEntry { first_value: 0x0226E, second_value: 0x0000, entity_offset: 3008, length: 8 }, // &NotLess;
    HTMLEntityTableEntry { first_value: 0x02270, second_value: 0x0000, entity_offset: 3016, length: 13 }, // &NotLessEqual;
// cpp: html/parser/html_entity_table.cc:2396-2427
    HTMLEntityTableEntry { first_value: 0x02278, second_value: 0x0000, entity_offset: 3029, length: 15 }, // &NotLessGreater;
    HTMLEntityTableEntry { first_value: 0x0226A, second_value: 0x0338, entity_offset: 3044, length: 12 }, // &NotLessLess;
    HTMLEntityTableEntry { first_value: 0x02A7D, second_value: 0x0338, entity_offset: 3056, length: 18 }, // &NotLessSlantEqual;
    HTMLEntityTableEntry { first_value: 0x02274, second_value: 0x0000, entity_offset: 3074, length: 13 }, // &NotLessTilde;
    HTMLEntityTableEntry { first_value: 0x02AA2, second_value: 0x0338, entity_offset: 3087, length: 24 }, // &NotNestedGreaterGreater;
    HTMLEntityTableEntry { first_value: 0x02AA1, second_value: 0x0338, entity_offset: 3111, length: 18 }, // &NotNestedLessLess;
    HTMLEntityTableEntry { first_value: 0x02280, second_value: 0x0000, entity_offset: 3129, length: 12 }, // &NotPrecedes;
    HTMLEntityTableEntry { first_value: 0x02AAF, second_value: 0x0338, entity_offset: 3141, length: 17 }, // &NotPrecedesEqual;
    HTMLEntityTableEntry { first_value: 0x022E0, second_value: 0x0000, entity_offset: 3158, length: 22 }, // &NotPrecedesSlantEqual;
    HTMLEntityTableEntry { first_value: 0x0220C, second_value: 0x0000, entity_offset: 3180, length: 18 }, // &NotReverseElement;
    HTMLEntityTableEntry { first_value: 0x022EB, second_value: 0x0000, entity_offset: 3198, length: 17 }, // &NotRightTriangle;
    HTMLEntityTableEntry { first_value: 0x029D0, second_value: 0x0338, entity_offset: 3215, length: 20 }, // &NotRightTriangleBar;
    HTMLEntityTableEntry { first_value: 0x022ED, second_value: 0x0000, entity_offset: 3235, length: 22 }, // &NotRightTriangleEqual;
    HTMLEntityTableEntry { first_value: 0x0228F, second_value: 0x0338, entity_offset: 3257, length: 16 }, // &NotSquareSubset;
    HTMLEntityTableEntry { first_value: 0x022E2, second_value: 0x0000, entity_offset: 3273, length: 21 }, // &NotSquareSubsetEqual;
    HTMLEntityTableEntry { first_value: 0x02290, second_value: 0x0338, entity_offset: 3294, length: 18 }, // &NotSquareSuperset;
    HTMLEntityTableEntry { first_value: 0x022E3, second_value: 0x0000, entity_offset: 3312, length: 23 }, // &NotSquareSupersetEqual;
    HTMLEntityTableEntry { first_value: 0x02282, second_value: 0x20D2, entity_offset: 3335, length: 10 }, // &NotSubset;
    HTMLEntityTableEntry { first_value: 0x02288, second_value: 0x0000, entity_offset: 3345, length: 15 }, // &NotSubsetEqual;
    HTMLEntityTableEntry { first_value: 0x02281, second_value: 0x0000, entity_offset: 3360, length: 12 }, // &NotSucceeds;
    HTMLEntityTableEntry { first_value: 0x02AB0, second_value: 0x0338, entity_offset: 3372, length: 17 }, // &NotSucceedsEqual;
    HTMLEntityTableEntry { first_value: 0x022E1, second_value: 0x0000, entity_offset: 3389, length: 22 }, // &NotSucceedsSlantEqual;
    HTMLEntityTableEntry { first_value: 0x0227F, second_value: 0x0338, entity_offset: 3411, length: 17 }, // &NotSucceedsTilde;
    HTMLEntityTableEntry { first_value: 0x02283, second_value: 0x20D2, entity_offset: 3428, length: 12 }, // &NotSuperset;
    HTMLEntityTableEntry { first_value: 0x02289, second_value: 0x0000, entity_offset: 3440, length: 17 }, // &NotSupersetEqual;
    HTMLEntityTableEntry { first_value: 0x02241, second_value: 0x0000, entity_offset: 3457, length: 9 }, // &NotTilde;
    HTMLEntityTableEntry { first_value: 0x02244, second_value: 0x0000, entity_offset: 3466, length: 14 }, // &NotTildeEqual;
    HTMLEntityTableEntry { first_value: 0x02247, second_value: 0x0000, entity_offset: 3480, length: 18 }, // &NotTildeFullEqual;
    HTMLEntityTableEntry { first_value: 0x02249, second_value: 0x0000, entity_offset: 3498, length: 14 }, // &NotTildeTilde;
    HTMLEntityTableEntry { first_value: 0x02224, second_value: 0x0000, entity_offset: 3512, length: 15 }, // &NotVerticalBar;
    HTMLEntityTableEntry { first_value: 0x1D4A9, second_value: 0x0000, entity_offset: 3527, length: 5 }, // &Nscr;
    HTMLEntityTableEntry { first_value: 0x000D1, second_value: 0x0000, entity_offset: 3532, length: 6 }, // &Ntilde
// cpp: html/parser/html_entity_table.cc:2428-2459
    HTMLEntityTableEntry { first_value: 0x000D1, second_value: 0x0000, entity_offset: 3532, length: 7 }, // &Ntilde;
    HTMLEntityTableEntry { first_value: 0x0039D, second_value: 0x0000, entity_offset: 3539, length: 3 }, // &Nu;
    HTMLEntityTableEntry { first_value: 0x00152, second_value: 0x0000, entity_offset: 3542, length: 6 }, // &OElig;
    HTMLEntityTableEntry { first_value: 0x000D3, second_value: 0x0000, entity_offset: 3548, length: 6 }, // &Oacute
    HTMLEntityTableEntry { first_value: 0x000D3, second_value: 0x0000, entity_offset: 3548, length: 7 }, // &Oacute;
    HTMLEntityTableEntry { first_value: 0x000D4, second_value: 0x0000, entity_offset: 3555, length: 5 }, // &Ocirc
    HTMLEntityTableEntry { first_value: 0x000D4, second_value: 0x0000, entity_offset: 3555, length: 6 }, // &Ocirc;
    HTMLEntityTableEntry { first_value: 0x0041E, second_value: 0x0000, entity_offset: 1691, length: 4 }, // &Ocy;
    HTMLEntityTableEntry { first_value: 0x00150, second_value: 0x0000, entity_offset: 3561, length: 7 }, // &Odblac;
    HTMLEntityTableEntry { first_value: 0x1D512, second_value: 0x0000, entity_offset: 3568, length: 4 }, // &Ofr;
    HTMLEntityTableEntry { first_value: 0x000D2, second_value: 0x0000, entity_offset: 3572, length: 6 }, // &Ograve
    HTMLEntityTableEntry { first_value: 0x000D2, second_value: 0x0000, entity_offset: 3572, length: 7 }, // &Ograve;
    HTMLEntityTableEntry { first_value: 0x0014C, second_value: 0x0000, entity_offset: 3579, length: 6 }, // &Omacr;
    HTMLEntityTableEntry { first_value: 0x003A9, second_value: 0x0000, entity_offset: 3585, length: 6 }, // &Omega;
    HTMLEntityTableEntry { first_value: 0x0039F, second_value: 0x0000, entity_offset: 3591, length: 8 }, // &Omicron;
    HTMLEntityTableEntry { first_value: 0x1D546, second_value: 0x0000, entity_offset: 3599, length: 5 }, // &Oopf;
    HTMLEntityTableEntry { first_value: 0x0201C, second_value: 0x0000, entity_offset: 3604, length: 21 }, // &OpenCurlyDoubleQuote;
    HTMLEntityTableEntry { first_value: 0x02018, second_value: 0x0000, entity_offset: 3625, length: 15 }, // &OpenCurlyQuote;
    HTMLEntityTableEntry { first_value: 0x02A54, second_value: 0x0000, entity_offset: 3640, length: 3 }, // &Or;
    HTMLEntityTableEntry { first_value: 0x1D4AA, second_value: 0x0000, entity_offset: 3643, length: 5 }, // &Oscr;
    HTMLEntityTableEntry { first_value: 0x000D8, second_value: 0x0000, entity_offset: 3648, length: 6 }, // &Oslash
    HTMLEntityTableEntry { first_value: 0x000D8, second_value: 0x0000, entity_offset: 3648, length: 7 }, // &Oslash;
    HTMLEntityTableEntry { first_value: 0x000D5, second_value: 0x0000, entity_offset: 3655, length: 6 }, // &Otilde
    HTMLEntityTableEntry { first_value: 0x000D5, second_value: 0x0000, entity_offset: 3655, length: 7 }, // &Otilde;
    HTMLEntityTableEntry { first_value: 0x02A37, second_value: 0x0000, entity_offset: 3662, length: 7 }, // &Otimes;
    HTMLEntityTableEntry { first_value: 0x000D6, second_value: 0x0000, entity_offset: 3669, length: 4 }, // &Ouml
    HTMLEntityTableEntry { first_value: 0x000D6, second_value: 0x0000, entity_offset: 3669, length: 5 }, // &Ouml;
    HTMLEntityTableEntry { first_value: 0x0203E, second_value: 0x0000, entity_offset: 3674, length: 8 }, // &OverBar;
    HTMLEntityTableEntry { first_value: 0x023DE, second_value: 0x0000, entity_offset: 3682, length: 10 }, // &OverBrace;
    HTMLEntityTableEntry { first_value: 0x023B4, second_value: 0x0000, entity_offset: 3692, length: 12 }, // &OverBracket;
    HTMLEntityTableEntry { first_value: 0x023DC, second_value: 0x0000, entity_offset: 3704, length: 16 }, // &OverParenthesis;
    HTMLEntityTableEntry { first_value: 0x02202, second_value: 0x0000, entity_offset: 3720, length: 9 }, // &PartialD;
// cpp: html/parser/html_entity_table.cc:2460-2491
    HTMLEntityTableEntry { first_value: 0x0041F, second_value: 0x0000, entity_offset: 3729, length: 4 }, // &Pcy;
    HTMLEntityTableEntry { first_value: 0x1D513, second_value: 0x0000, entity_offset: 3733, length: 4 }, // &Pfr;
    HTMLEntityTableEntry { first_value: 0x003A6, second_value: 0x0000, entity_offset: 3737, length: 4 }, // &Phi;
    HTMLEntityTableEntry { first_value: 0x003A0, second_value: 0x0000, entity_offset: 3741, length: 3 }, // &Pi;
    HTMLEntityTableEntry { first_value: 0x000B1, second_value: 0x0000, entity_offset: 3744, length: 10 }, // &PlusMinus;
    HTMLEntityTableEntry { first_value: 0x0210C, second_value: 0x0000, entity_offset: 3754, length: 14 }, // &Poincareplane;
    HTMLEntityTableEntry { first_value: 0x02119, second_value: 0x0000, entity_offset: 3768, length: 5 }, // &Popf;
    HTMLEntityTableEntry { first_value: 0x02ABB, second_value: 0x0000, entity_offset: 3773, length: 3 }, // &Pr;
    HTMLEntityTableEntry { first_value: 0x0227A, second_value: 0x0000, entity_offset: 3132, length: 9 }, // &Precedes;
    HTMLEntityTableEntry { first_value: 0x02AAF, second_value: 0x0000, entity_offset: 3144, length: 14 }, // &PrecedesEqual;
    HTMLEntityTableEntry { first_value: 0x0227C, second_value: 0x0000, entity_offset: 3161, length: 19 }, // &PrecedesSlantEqual;
    HTMLEntityTableEntry { first_value: 0x0227E, second_value: 0x0000, entity_offset: 3776, length: 14 }, // &PrecedesTilde;
    HTMLEntityTableEntry { first_value: 0x02033, second_value: 0x0000, entity_offset: 3790, length: 6 }, // &Prime;
    HTMLEntityTableEntry { first_value: 0x0220F, second_value: 0x0000, entity_offset: 3796, length: 8 }, // &Product;
    HTMLEntityTableEntry { first_value: 0x02237, second_value: 0x0000, entity_offset: 3804, length: 11 }, // &Proportion;
    HTMLEntityTableEntry { first_value: 0x0221D, second_value: 0x0000, entity_offset: 3815, length: 13 }, // &Proportional;
    HTMLEntityTableEntry { first_value: 0x1D4AB, second_value: 0x0000, entity_offset: 3828, length: 5 }, // &Pscr;
    HTMLEntityTableEntry { first_value: 0x003A8, second_value: 0x0000, entity_offset: 3833, length: 4 }, // &Psi;
    HTMLEntityTableEntry { first_value: 0x00022, second_value: 0x0000, entity_offset: 3837, length: 4 }, // &QUOT
    HTMLEntityTableEntry { first_value: 0x00022, second_value: 0x0000, entity_offset: 3837, length: 5 }, // &QUOT;
    HTMLEntityTableEntry { first_value: 0x1D514, second_value: 0x0000, entity_offset: 3842, length: 4 }, // &Qfr;
    HTMLEntityTableEntry { first_value: 0x0211A, second_value: 0x0000, entity_offset: 3846, length: 5 }, // &Qopf;
    HTMLEntityTableEntry { first_value: 0x1D4AC, second_value: 0x0000, entity_offset: 3851, length: 5 }, // &Qscr;
    HTMLEntityTableEntry { first_value: 0x02910, second_value: 0x0000, entity_offset: 3856, length: 6 }, // &RBarr;
    HTMLEntityTableEntry { first_value: 0x000AE, second_value: 0x0000, entity_offset: 3862, length: 3 }, // &REG
    HTMLEntityTableEntry { first_value: 0x000AE, second_value: 0x0000, entity_offset: 3862, length: 4 }, // &REG;
    HTMLEntityTableEntry { first_value: 0x00154, second_value: 0x0000, entity_offset: 3866, length: 7 }, // &Racute;
    HTMLEntityTableEntry { first_value: 0x027EB, second_value: 0x0000, entity_offset: 3873, length: 5 }, // &Rang;
    HTMLEntityTableEntry { first_value: 0x021A0, second_value: 0x0000, entity_offset: 3878, length: 5 }, // &Rarr;
    HTMLEntityTableEntry { first_value: 0x02916, second_value: 0x0000, entity_offset: 3883, length: 7 }, // &Rarrtl;
    HTMLEntityTableEntry { first_value: 0x00158, second_value: 0x0000, entity_offset: 3890, length: 7 }, // &Rcaron;
    HTMLEntityTableEntry { first_value: 0x00156, second_value: 0x0000, entity_offset: 3897, length: 7 }, // &Rcedil;
// cpp: html/parser/html_entity_table.cc:2492-2523
    HTMLEntityTableEntry { first_value: 0x00420, second_value: 0x0000, entity_offset: 3904, length: 4 }, // &Rcy;
    HTMLEntityTableEntry { first_value: 0x0211C, second_value: 0x0000, entity_offset: 3908, length: 3 }, // &Re;
    HTMLEntityTableEntry { first_value: 0x0220B, second_value: 0x0000, entity_offset: 3183, length: 15 }, // &ReverseElement;
    HTMLEntityTableEntry { first_value: 0x021CB, second_value: 0x0000, entity_offset: 3911, length: 19 }, // &ReverseEquilibrium;
    HTMLEntityTableEntry { first_value: 0x0296F, second_value: 0x0000, entity_offset: 3930, length: 21 }, // &ReverseUpEquilibrium;
    HTMLEntityTableEntry { first_value: 0x0211C, second_value: 0x0000, entity_offset: 3951, length: 4 }, // &Rfr;
    HTMLEntityTableEntry { first_value: 0x003A1, second_value: 0x0000, entity_offset: 3955, length: 4 }, // &Rho;
    HTMLEntityTableEntry { first_value: 0x027E9, second_value: 0x0000, entity_offset: 3959, length: 18 }, // &RightAngleBracket;
    HTMLEntityTableEntry { first_value: 0x02192, second_value: 0x0000, entity_offset: 771, length: 11 }, // &RightArrow;
    HTMLEntityTableEntry { first_value: 0x021E5, second_value: 0x0000, entity_offset: 3977, length: 14 }, // &RightArrowBar;
    HTMLEntityTableEntry { first_value: 0x021C4, second_value: 0x0000, entity_offset: 3991, length: 20 }, // &RightArrowLeftArrow;
    HTMLEntityTableEntry { first_value: 0x02309, second_value: 0x0000, entity_offset: 4011, length: 13 }, // &RightCeiling;
    HTMLEntityTableEntry { first_value: 0x027E7, second_value: 0x0000, entity_offset: 4024, length: 19 }, // &RightDoubleBracket;
    HTMLEntityTableEntry { first_value: 0x0295D, second_value: 0x0000, entity_offset: 4043, length: 19 }, // &RightDownTeeVector;
    HTMLEntityTableEntry { first_value: 0x021C2, second_value: 0x0000, entity_offset: 4062, length: 16 }, // &RightDownVector;
    HTMLEntityTableEntry { first_value: 0x02955, second_value: 0x0000, entity_offset: 4078, length: 19 }, // &RightDownVectorBar;
    HTMLEntityTableEntry { first_value: 0x0230B, second_value: 0x0000, entity_offset: 4097, length: 11 }, // &RightFloor;
    HTMLEntityTableEntry { first_value: 0x022A2, second_value: 0x0000, entity_offset: 885, length: 9 }, // &RightTee;
    HTMLEntityTableEntry { first_value: 0x021A6, second_value: 0x0000, entity_offset: 4108, length: 14 }, // &RightTeeArrow;
    HTMLEntityTableEntry { first_value: 0x0295B, second_value: 0x0000, entity_offset: 1059, length: 15 }, // &RightTeeVector;
    HTMLEntityTableEntry { first_value: 0x022B3, second_value: 0x0000, entity_offset: 3201, length: 14 }, // &RightTriangle;
    HTMLEntityTableEntry { first_value: 0x029D0, second_value: 0x0000, entity_offset: 3218, length: 17 }, // &RightTriangleBar;
    HTMLEntityTableEntry { first_value: 0x022B5, second_value: 0x0000, entity_offset: 3238, length: 19 }, // &RightTriangleEqual;
    HTMLEntityTableEntry { first_value: 0x0294F, second_value: 0x0000, entity_offset: 4122, length: 18 }, // &RightUpDownVector;
    HTMLEntityTableEntry { first_value: 0x0295C, second_value: 0x0000, entity_offset: 4140, length: 17 }, // &RightUpTeeVector;
    HTMLEntityTableEntry { first_value: 0x021BE, second_value: 0x0000, entity_offset: 4157, length: 14 }, // &RightUpVector;
    HTMLEntityTableEntry { first_value: 0x02954, second_value: 0x0000, entity_offset: 4171, length: 17 }, // &RightUpVectorBar;
    HTMLEntityTableEntry { first_value: 0x021C0, second_value: 0x0000, entity_offset: 992, length: 12 }, // &RightVector;
    HTMLEntityTableEntry { first_value: 0x02953, second_value: 0x0000, entity_offset: 1094, length: 15 }, // &RightVectorBar;
    HTMLEntityTableEntry { first_value: 0x021D2, second_value: 0x0000, entity_offset: 4188, length: 11 }, // &Rightarrow;
    HTMLEntityTableEntry { first_value: 0x0211D, second_value: 0x0000, entity_offset: 4199, length: 5 }, // &Ropf;
    HTMLEntityTableEntry { first_value: 0x02970, second_value: 0x0000, entity_offset: 4204, length: 13 }, // &RoundImplies;
// cpp: html/parser/html_entity_table.cc:2524-2555
    HTMLEntityTableEntry { first_value: 0x021DB, second_value: 0x0000, entity_offset: 4217, length: 12 }, // &Rrightarrow;
    HTMLEntityTableEntry { first_value: 0x0211B, second_value: 0x0000, entity_offset: 4229, length: 5 }, // &Rscr;
    HTMLEntityTableEntry { first_value: 0x021B1, second_value: 0x0000, entity_offset: 4234, length: 4 }, // &Rsh;
    HTMLEntityTableEntry { first_value: 0x029F4, second_value: 0x0000, entity_offset: 4238, length: 12 }, // &RuleDelayed;
    HTMLEntityTableEntry { first_value: 0x00429, second_value: 0x0000, entity_offset: 4250, length: 7 }, // &SHCHcy;
    HTMLEntityTableEntry { first_value: 0x00428, second_value: 0x0000, entity_offset: 4257, length: 5 }, // &SHcy;
    HTMLEntityTableEntry { first_value: 0x0042C, second_value: 0x0000, entity_offset: 4262, length: 7 }, // &SOFTcy;
    HTMLEntityTableEntry { first_value: 0x0015A, second_value: 0x0000, entity_offset: 4269, length: 7 }, // &Sacute;
    HTMLEntityTableEntry { first_value: 0x02ABC, second_value: 0x0000, entity_offset: 4276, length: 3 }, // &Sc;
    HTMLEntityTableEntry { first_value: 0x00160, second_value: 0x0000, entity_offset: 4279, length: 7 }, // &Scaron;
    HTMLEntityTableEntry { first_value: 0x0015E, second_value: 0x0000, entity_offset: 4286, length: 7 }, // &Scedil;
    HTMLEntityTableEntry { first_value: 0x0015C, second_value: 0x0000, entity_offset: 4293, length: 6 }, // &Scirc;
    HTMLEntityTableEntry { first_value: 0x00421, second_value: 0x0000, entity_offset: 527, length: 4 }, // &Scy;
    HTMLEntityTableEntry { first_value: 0x1D516, second_value: 0x0000, entity_offset: 4299, length: 4 }, // &Sfr;
    HTMLEntityTableEntry { first_value: 0x02193, second_value: 0x0000, entity_offset: 4303, length: 15 }, // &ShortDownArrow;
    HTMLEntityTableEntry { first_value: 0x02190, second_value: 0x0000, entity_offset: 4318, length: 15 }, // &ShortLeftArrow;
    HTMLEntityTableEntry { first_value: 0x02192, second_value: 0x0000, entity_offset: 4333, length: 16 }, // &ShortRightArrow;
    HTMLEntityTableEntry { first_value: 0x02191, second_value: 0x0000, entity_offset: 4349, length: 13 }, // &ShortUpArrow;
    HTMLEntityTableEntry { first_value: 0x003A3, second_value: 0x0000, entity_offset: 4362, length: 6 }, // &Sigma;
    HTMLEntityTableEntry { first_value: 0x02218, second_value: 0x0000, entity_offset: 4368, length: 12 }, // &SmallCircle;
    HTMLEntityTableEntry { first_value: 0x1D54A, second_value: 0x0000, entity_offset: 4380, length: 5 }, // &Sopf;
    HTMLEntityTableEntry { first_value: 0x0221A, second_value: 0x0000, entity_offset: 4385, length: 5 }, // &Sqrt;
    HTMLEntityTableEntry { first_value: 0x025A1, second_value: 0x0000, entity_offset: 1224, length: 7 }, // &Square;
    HTMLEntityTableEntry { first_value: 0x02293, second_value: 0x0000, entity_offset: 4390, length: 19 }, // &SquareIntersection;
    HTMLEntityTableEntry { first_value: 0x0228F, second_value: 0x0000, entity_offset: 3260, length: 13 }, // &SquareSubset;
    HTMLEntityTableEntry { first_value: 0x02291, second_value: 0x0000, entity_offset: 3276, length: 18 }, // &SquareSubsetEqual;
    HTMLEntityTableEntry { first_value: 0x02290, second_value: 0x0000, entity_offset: 3297, length: 15 }, // &SquareSuperset;
    HTMLEntityTableEntry { first_value: 0x02292, second_value: 0x0000, entity_offset: 3315, length: 20 }, // &SquareSupersetEqual;
    HTMLEntityTableEntry { first_value: 0x02294, second_value: 0x0000, entity_offset: 4409, length: 12 }, // &SquareUnion;
    HTMLEntityTableEntry { first_value: 0x1D4AE, second_value: 0x0000, entity_offset: 4421, length: 5 }, // &Sscr;
    HTMLEntityTableEntry { first_value: 0x022C6, second_value: 0x0000, entity_offset: 4426, length: 5 }, // &Star;
    HTMLEntityTableEntry { first_value: 0x022D0, second_value: 0x0000, entity_offset: 4431, length: 4 }, // &Sub;
// cpp: html/parser/html_entity_table.cc:2556-2587
    HTMLEntityTableEntry { first_value: 0x022D0, second_value: 0x0000, entity_offset: 3266, length: 7 }, // &Subset;
    HTMLEntityTableEntry { first_value: 0x02286, second_value: 0x0000, entity_offset: 3282, length: 12 }, // &SubsetEqual;
    HTMLEntityTableEntry { first_value: 0x0227B, second_value: 0x0000, entity_offset: 3363, length: 9 }, // &Succeeds;
    HTMLEntityTableEntry { first_value: 0x02AB0, second_value: 0x0000, entity_offset: 3375, length: 14 }, // &SucceedsEqual;
    HTMLEntityTableEntry { first_value: 0x0227D, second_value: 0x0000, entity_offset: 3392, length: 19 }, // &SucceedsSlantEqual;
    HTMLEntityTableEntry { first_value: 0x0227F, second_value: 0x0000, entity_offset: 3414, length: 14 }, // &SucceedsTilde;
    HTMLEntityTableEntry { first_value: 0x0220B, second_value: 0x0000, entity_offset: 4435, length: 9 }, // &SuchThat;
    HTMLEntityTableEntry { first_value: 0x02211, second_value: 0x0000, entity_offset: 4444, length: 4 }, // &Sum;
    HTMLEntityTableEntry { first_value: 0x022D1, second_value: 0x0000, entity_offset: 4448, length: 4 }, // &Sup;
    HTMLEntityTableEntry { first_value: 0x02283, second_value: 0x0000, entity_offset: 3303, length: 9 }, // &Superset;
    HTMLEntityTableEntry { first_value: 0x02287, second_value: 0x0000, entity_offset: 3321, length: 14 }, // &SupersetEqual;
    HTMLEntityTableEntry { first_value: 0x022D1, second_value: 0x0000, entity_offset: 4452, length: 7 }, // &Supset;
    HTMLEntityTableEntry { first_value: 0x000DE, second_value: 0x0000, entity_offset: 4459, length: 5 }, // &THORN
    HTMLEntityTableEntry { first_value: 0x000DE, second_value: 0x0000, entity_offset: 4459, length: 6 }, // &THORN;
    HTMLEntityTableEntry { first_value: 0x02122, second_value: 0x0000, entity_offset: 4465, length: 6 }, // &TRADE;
    HTMLEntityTableEntry { first_value: 0x0040B, second_value: 0x0000, entity_offset: 4471, length: 6 }, // &TSHcy;
    HTMLEntityTableEntry { first_value: 0x00426, second_value: 0x0000, entity_offset: 4477, length: 5 }, // &TScy;
    HTMLEntityTableEntry { first_value: 0x00009, second_value: 0x0000, entity_offset: 4482, length: 4 }, // &Tab;
    HTMLEntityTableEntry { first_value: 0x003A4, second_value: 0x0000, entity_offset: 4486, length: 4 }, // &Tau;
    HTMLEntityTableEntry { first_value: 0x00164, second_value: 0x0000, entity_offset: 4490, length: 7 }, // &Tcaron;
    HTMLEntityTableEntry { first_value: 0x00162, second_value: 0x0000, entity_offset: 4497, length: 7 }, // &Tcedil;
    HTMLEntityTableEntry { first_value: 0x00422, second_value: 0x0000, entity_offset: 4265, length: 4 }, // &Tcy;
    HTMLEntityTableEntry { first_value: 0x1D517, second_value: 0x0000, entity_offset: 4504, length: 4 }, // &Tfr;
    HTMLEntityTableEntry { first_value: 0x02234, second_value: 0x0000, entity_offset: 4508, length: 10 }, // &Therefore;
    HTMLEntityTableEntry { first_value: 0x00398, second_value: 0x0000, entity_offset: 4518, length: 6 }, // &Theta;
    HTMLEntityTableEntry { first_value: 0x0205F, second_value: 0x200A, entity_offset: 2585, length: 11 }, // &ThickSpace;
    HTMLEntityTableEntry { first_value: 0x02009, second_value: 0x0000, entity_offset: 2604, length: 10 }, // &ThinSpace;
    HTMLEntityTableEntry { first_value: 0x0223C, second_value: 0x0000, entity_offset: 662, length: 6 }, // &Tilde;
    HTMLEntityTableEntry { first_value: 0x02243, second_value: 0x0000, entity_offset: 3469, length: 11 }, // &TildeEqual;
    HTMLEntityTableEntry { first_value: 0x02245, second_value: 0x0000, entity_offset: 3483, length: 15 }, // &TildeFullEqual;
    HTMLEntityTableEntry { first_value: 0x02248, second_value: 0x0000, entity_offset: 3501, length: 11 }, // &TildeTilde;
    HTMLEntityTableEntry { first_value: 0x1D54B, second_value: 0x0000, entity_offset: 4524, length: 5 }, // &Topf;
// cpp: html/parser/html_entity_table.cc:2588-2619
    HTMLEntityTableEntry { first_value: 0x020DB, second_value: 0x0000, entity_offset: 4529, length: 10 }, // &TripleDot;
    HTMLEntityTableEntry { first_value: 0x1D4AF, second_value: 0x0000, entity_offset: 4539, length: 5 }, // &Tscr;
    HTMLEntityTableEntry { first_value: 0x00166, second_value: 0x0000, entity_offset: 4544, length: 7 }, // &Tstrok;
    HTMLEntityTableEntry { first_value: 0x000DA, second_value: 0x0000, entity_offset: 4551, length: 6 }, // &Uacute
    HTMLEntityTableEntry { first_value: 0x000DA, second_value: 0x0000, entity_offset: 4551, length: 7 }, // &Uacute;
    HTMLEntityTableEntry { first_value: 0x0219F, second_value: 0x0000, entity_offset: 4558, length: 5 }, // &Uarr;
    HTMLEntityTableEntry { first_value: 0x02949, second_value: 0x0000, entity_offset: 4563, length: 9 }, // &Uarrocir;
    HTMLEntityTableEntry { first_value: 0x0040E, second_value: 0x0000, entity_offset: 4572, length: 6 }, // &Ubrcy;
    HTMLEntityTableEntry { first_value: 0x0016C, second_value: 0x0000, entity_offset: 4578, length: 7 }, // &Ubreve;
    HTMLEntityTableEntry { first_value: 0x000DB, second_value: 0x0000, entity_offset: 4585, length: 5 }, // &Ucirc
    HTMLEntityTableEntry { first_value: 0x000DB, second_value: 0x0000, entity_offset: 4585, length: 6 }, // &Ucirc;
    HTMLEntityTableEntry { first_value: 0x00423, second_value: 0x0000, entity_offset: 4591, length: 4 }, // &Ucy;
    HTMLEntityTableEntry { first_value: 0x00170, second_value: 0x0000, entity_offset: 4595, length: 7 }, // &Udblac;
    HTMLEntityTableEntry { first_value: 0x1D518, second_value: 0x0000, entity_offset: 4602, length: 4 }, // &Ufr;
    HTMLEntityTableEntry { first_value: 0x000D9, second_value: 0x0000, entity_offset: 4606, length: 6 }, // &Ugrave
    HTMLEntityTableEntry { first_value: 0x000D9, second_value: 0x0000, entity_offset: 4606, length: 7 }, // &Ugrave;
    HTMLEntityTableEntry { first_value: 0x0016A, second_value: 0x0000, entity_offset: 4613, length: 6 }, // &Umacr;
    HTMLEntityTableEntry { first_value: 0x0005F, second_value: 0x0000, entity_offset: 4619, length: 9 }, // &UnderBar;
    HTMLEntityTableEntry { first_value: 0x023DF, second_value: 0x0000, entity_offset: 4628, length: 11 }, // &UnderBrace;
    HTMLEntityTableEntry { first_value: 0x023B5, second_value: 0x0000, entity_offset: 4639, length: 13 }, // &UnderBracket;
    HTMLEntityTableEntry { first_value: 0x023DD, second_value: 0x0000, entity_offset: 4652, length: 17 }, // &UnderParenthesis;
    HTMLEntityTableEntry { first_value: 0x022C3, second_value: 0x0000, entity_offset: 4415, length: 6 }, // &Union;
    HTMLEntityTableEntry { first_value: 0x0228E, second_value: 0x0000, entity_offset: 4669, length: 10 }, // &UnionPlus;
    HTMLEntityTableEntry { first_value: 0x00172, second_value: 0x0000, entity_offset: 4679, length: 6 }, // &Uogon;
    HTMLEntityTableEntry { first_value: 0x1D54C, second_value: 0x0000, entity_offset: 4685, length: 5 }, // &Uopf;
    HTMLEntityTableEntry { first_value: 0x02191, second_value: 0x0000, entity_offset: 900, length: 8 }, // &UpArrow;
    HTMLEntityTableEntry { first_value: 0x02912, second_value: 0x0000, entity_offset: 4690, length: 11 }, // &UpArrowBar;
    HTMLEntityTableEntry { first_value: 0x021C5, second_value: 0x0000, entity_offset: 4701, length: 17 }, // &UpArrowDownArrow;
    HTMLEntityTableEntry { first_value: 0x02195, second_value: 0x0000, entity_offset: 914, length: 12 }, // &UpDownArrow;
    HTMLEntityTableEntry { first_value: 0x0296E, second_value: 0x0000, entity_offset: 3937, length: 14 }, // &UpEquilibrium;
    HTMLEntityTableEntry { first_value: 0x022A5, second_value: 0x0000, entity_offset: 4718, length: 6 }, // &UpTee;
    HTMLEntityTableEntry { first_value: 0x021A5, second_value: 0x0000, entity_offset: 4724, length: 11 }, // &UpTeeArrow;
// cpp: html/parser/html_entity_table.cc:2620-2651
    HTMLEntityTableEntry { first_value: 0x021D1, second_value: 0x0000, entity_offset: 4735, length: 8 }, // &Uparrow;
    HTMLEntityTableEntry { first_value: 0x021D5, second_value: 0x0000, entity_offset: 4743, length: 12 }, // &Updownarrow;
    HTMLEntityTableEntry { first_value: 0x02196, second_value: 0x0000, entity_offset: 4755, length: 15 }, // &UpperLeftArrow;
    HTMLEntityTableEntry { first_value: 0x02197, second_value: 0x0000, entity_offset: 4770, length: 16 }, // &UpperRightArrow;
    HTMLEntityTableEntry { first_value: 0x003D2, second_value: 0x0000, entity_offset: 4786, length: 5 }, // &Upsi;
    HTMLEntityTableEntry { first_value: 0x003A5, second_value: 0x0000, entity_offset: 4791, length: 8 }, // &Upsilon;
    HTMLEntityTableEntry { first_value: 0x0016E, second_value: 0x0000, entity_offset: 4799, length: 6 }, // &Uring;
    HTMLEntityTableEntry { first_value: 0x1D4B0, second_value: 0x0000, entity_offset: 4805, length: 5 }, // &Uscr;
    HTMLEntityTableEntry { first_value: 0x00168, second_value: 0x0000, entity_offset: 4810, length: 7 }, // &Utilde;
    HTMLEntityTableEntry { first_value: 0x000DC, second_value: 0x0000, entity_offset: 4817, length: 4 }, // &Uuml
    HTMLEntityTableEntry { first_value: 0x000DC, second_value: 0x0000, entity_offset: 4817, length: 5 }, // &Uuml;
    HTMLEntityTableEntry { first_value: 0x022AB, second_value: 0x0000, entity_offset: 4822, length: 6 }, // &VDash;
    HTMLEntityTableEntry { first_value: 0x02AEB, second_value: 0x0000, entity_offset: 4828, length: 5 }, // &Vbar;
    HTMLEntityTableEntry { first_value: 0x00412, second_value: 0x0000, entity_offset: 4833, length: 4 }, // &Vcy;
    HTMLEntityTableEntry { first_value: 0x022A9, second_value: 0x0000, entity_offset: 4837, length: 6 }, // &Vdash;
    HTMLEntityTableEntry { first_value: 0x02AE6, second_value: 0x0000, entity_offset: 4843, length: 7 }, // &Vdashl;
    HTMLEntityTableEntry { first_value: 0x022C1, second_value: 0x0000, entity_offset: 4850, length: 4 }, // &Vee;
    HTMLEntityTableEntry { first_value: 0x02016, second_value: 0x0000, entity_offset: 4854, length: 7 }, // &Verbar;
    HTMLEntityTableEntry { first_value: 0x02016, second_value: 0x0000, entity_offset: 4861, length: 5 }, // &Vert;
    HTMLEntityTableEntry { first_value: 0x02223, second_value: 0x0000, entity_offset: 932, length: 12 }, // &VerticalBar;
    HTMLEntityTableEntry { first_value: 0x0007C, second_value: 0x0000, entity_offset: 4866, length: 13 }, // &VerticalLine;
    HTMLEntityTableEntry { first_value: 0x02758, second_value: 0x0000, entity_offset: 4879, length: 18 }, // &VerticalSeparator;
    HTMLEntityTableEntry { first_value: 0x02240, second_value: 0x0000, entity_offset: 4897, length: 14 }, // &VerticalTilde;
    HTMLEntityTableEntry { first_value: 0x0200A, second_value: 0x0000, entity_offset: 2622, length: 14 }, // &VeryThinSpace;
    HTMLEntityTableEntry { first_value: 0x1D519, second_value: 0x0000, entity_offset: 4911, length: 4 }, // &Vfr;
    HTMLEntityTableEntry { first_value: 0x1D54D, second_value: 0x0000, entity_offset: 4915, length: 5 }, // &Vopf;
    HTMLEntityTableEntry { first_value: 0x1D4B1, second_value: 0x0000, entity_offset: 4920, length: 5 }, // &Vscr;
    HTMLEntityTableEntry { first_value: 0x022AA, second_value: 0x0000, entity_offset: 4925, length: 7 }, // &Vvdash;
    HTMLEntityTableEntry { first_value: 0x00174, second_value: 0x0000, entity_offset: 4932, length: 6 }, // &Wcirc;
    HTMLEntityTableEntry { first_value: 0x022C0, second_value: 0x0000, entity_offset: 4938, length: 6 }, // &Wedge;
    HTMLEntityTableEntry { first_value: 0x1D51A, second_value: 0x0000, entity_offset: 4944, length: 4 }, // &Wfr;
    HTMLEntityTableEntry { first_value: 0x1D54E, second_value: 0x0000, entity_offset: 4948, length: 5 }, // &Wopf;
// cpp: html/parser/html_entity_table.cc:2652-2683
    HTMLEntityTableEntry { first_value: 0x1D4B2, second_value: 0x0000, entity_offset: 4953, length: 5 }, // &Wscr;
    HTMLEntityTableEntry { first_value: 0x1D51B, second_value: 0x0000, entity_offset: 4958, length: 4 }, // &Xfr;
    HTMLEntityTableEntry { first_value: 0x0039E, second_value: 0x0000, entity_offset: 4962, length: 3 }, // &Xi;
    HTMLEntityTableEntry { first_value: 0x1D54F, second_value: 0x0000, entity_offset: 4965, length: 5 }, // &Xopf;
    HTMLEntityTableEntry { first_value: 0x1D4B3, second_value: 0x0000, entity_offset: 4970, length: 5 }, // &Xscr;
    HTMLEntityTableEntry { first_value: 0x0042F, second_value: 0x0000, entity_offset: 4975, length: 5 }, // &YAcy;
    HTMLEntityTableEntry { first_value: 0x00407, second_value: 0x0000, entity_offset: 4980, length: 5 }, // &YIcy;
    HTMLEntityTableEntry { first_value: 0x0042E, second_value: 0x0000, entity_offset: 4985, length: 5 }, // &YUcy;
    HTMLEntityTableEntry { first_value: 0x000DD, second_value: 0x0000, entity_offset: 4990, length: 6 }, // &Yacute
    HTMLEntityTableEntry { first_value: 0x000DD, second_value: 0x0000, entity_offset: 4990, length: 7 }, // &Yacute;
    HTMLEntityTableEntry { first_value: 0x00176, second_value: 0x0000, entity_offset: 4997, length: 6 }, // &Ycirc;
    HTMLEntityTableEntry { first_value: 0x0042B, second_value: 0x0000, entity_offset: 5003, length: 4 }, // &Ycy;
    HTMLEntityTableEntry { first_value: 0x1D51C, second_value: 0x0000, entity_offset: 5007, length: 4 }, // &Yfr;
    HTMLEntityTableEntry { first_value: 0x1D550, second_value: 0x0000, entity_offset: 5011, length: 5 }, // &Yopf;
    HTMLEntityTableEntry { first_value: 0x1D4B4, second_value: 0x0000, entity_offset: 5016, length: 5 }, // &Yscr;
    HTMLEntityTableEntry { first_value: 0x00178, second_value: 0x0000, entity_offset: 5021, length: 5 }, // &Yuml;
    HTMLEntityTableEntry { first_value: 0x00416, second_value: 0x0000, entity_offset: 5026, length: 5 }, // &ZHcy;
    HTMLEntityTableEntry { first_value: 0x00179, second_value: 0x0000, entity_offset: 5031, length: 7 }, // &Zacute;
    HTMLEntityTableEntry { first_value: 0x0017D, second_value: 0x0000, entity_offset: 5038, length: 7 }, // &Zcaron;
    HTMLEntityTableEntry { first_value: 0x00417, second_value: 0x0000, entity_offset: 532, length: 4 }, // &Zcy;
    HTMLEntityTableEntry { first_value: 0x0017B, second_value: 0x0000, entity_offset: 5045, length: 5 }, // &Zdot;
    HTMLEntityTableEntry { first_value: 0x0200B, second_value: 0x0000, entity_offset: 5050, length: 15 }, // &ZeroWidthSpace;
    HTMLEntityTableEntry { first_value: 0x00396, second_value: 0x0000, entity_offset: 5065, length: 5 }, // &Zeta;
    HTMLEntityTableEntry { first_value: 0x02128, second_value: 0x0000, entity_offset: 5070, length: 4 }, // &Zfr;
    HTMLEntityTableEntry { first_value: 0x02124, second_value: 0x0000, entity_offset: 5074, length: 5 }, // &Zopf;
    HTMLEntityTableEntry { first_value: 0x1D4B5, second_value: 0x0000, entity_offset: 5079, length: 5 }, // &Zscr;
    HTMLEntityTableEntry { first_value: 0x000E1, second_value: 0x0000, entity_offset: 5084, length: 6 }, // &aacute
    HTMLEntityTableEntry { first_value: 0x000E1, second_value: 0x0000, entity_offset: 5084, length: 7 }, // &aacute;
    HTMLEntityTableEntry { first_value: 0x00103, second_value: 0x0000, entity_offset: 5091, length: 7 }, // &abreve;
    HTMLEntityTableEntry { first_value: 0x0223E, second_value: 0x0000, entity_offset: 3565, length: 3 }, // &ac;
    HTMLEntityTableEntry { first_value: 0x0223E, second_value: 0x0333, entity_offset: 5098, length: 4 }, // &acE;
    HTMLEntityTableEntry { first_value: 0x0223F, second_value: 0x0000, entity_offset: 5102, length: 4 }, // &acd;
// cpp: html/parser/html_entity_table.cc:2684-2715
    HTMLEntityTableEntry { first_value: 0x000E2, second_value: 0x0000, entity_offset: 5106, length: 5 }, // &acirc
    HTMLEntityTableEntry { first_value: 0x000E2, second_value: 0x0000, entity_offset: 5106, length: 6 }, // &acirc;
    HTMLEntityTableEntry { first_value: 0x000B4, second_value: 0x0000, entity_offset: 11, length: 5 }, // &acute
    HTMLEntityTableEntry { first_value: 0x000B4, second_value: 0x0000, entity_offset: 11, length: 6 }, // &acute;
    HTMLEntityTableEntry { first_value: 0x00430, second_value: 0x0000, entity_offset: 5112, length: 4 }, // &acy;
    HTMLEntityTableEntry { first_value: 0x000E6, second_value: 0x0000, entity_offset: 5116, length: 5 }, // &aelig
    HTMLEntityTableEntry { first_value: 0x000E6, second_value: 0x0000, entity_offset: 5116, length: 6 }, // &aelig;
    HTMLEntityTableEntry { first_value: 0x02061, second_value: 0x0000, entity_offset: 5122, length: 3 }, // &af;
    HTMLEntityTableEntry { first_value: 0x1D51E, second_value: 0x0000, entity_offset: 5125, length: 4 }, // &afr;
    HTMLEntityTableEntry { first_value: 0x000E0, second_value: 0x0000, entity_offset: 5129, length: 6 }, // &agrave
    HTMLEntityTableEntry { first_value: 0x000E0, second_value: 0x0000, entity_offset: 5129, length: 7 }, // &agrave;
    HTMLEntityTableEntry { first_value: 0x02135, second_value: 0x0000, entity_offset: 5136, length: 8 }, // &alefsym;
    HTMLEntityTableEntry { first_value: 0x02135, second_value: 0x0000, entity_offset: 5144, length: 6 }, // &aleph;
    HTMLEntityTableEntry { first_value: 0x003B1, second_value: 0x0000, entity_offset: 5150, length: 6 }, // &alpha;
    HTMLEntityTableEntry { first_value: 0x00101, second_value: 0x0000, entity_offset: 5156, length: 6 }, // &amacr;
    HTMLEntityTableEntry { first_value: 0x02A3F, second_value: 0x0000, entity_offset: 5162, length: 6 }, // &amalg;
    HTMLEntityTableEntry { first_value: 0x00026, second_value: 0x0000, entity_offset: 5168, length: 3 }, // &amp
    HTMLEntityTableEntry { first_value: 0x00026, second_value: 0x0000, entity_offset: 5168, length: 4 }, // &amp;
    HTMLEntityTableEntry { first_value: 0x02227, second_value: 0x0000, entity_offset: 5172, length: 4 }, // &and;
    HTMLEntityTableEntry { first_value: 0x02A55, second_value: 0x0000, entity_offset: 5176, length: 7 }, // &andand;
    HTMLEntityTableEntry { first_value: 0x02A5C, second_value: 0x0000, entity_offset: 5183, length: 5 }, // &andd;
    HTMLEntityTableEntry { first_value: 0x02A58, second_value: 0x0000, entity_offset: 5188, length: 9 }, // &andslope;
    HTMLEntityTableEntry { first_value: 0x02A5A, second_value: 0x0000, entity_offset: 5197, length: 5 }, // &andv;
    HTMLEntityTableEntry { first_value: 0x02220, second_value: 0x0000, entity_offset: 1939, length: 4 }, // &ang;
    HTMLEntityTableEntry { first_value: 0x029A4, second_value: 0x0000, entity_offset: 5202, length: 5 }, // &ange;
    HTMLEntityTableEntry { first_value: 0x02220, second_value: 0x0000, entity_offset: 2138, length: 6 }, // &angle;
    HTMLEntityTableEntry { first_value: 0x02221, second_value: 0x0000, entity_offset: 5207, length: 7 }, // &angmsd;
    HTMLEntityTableEntry { first_value: 0x029A8, second_value: 0x0000, entity_offset: 5214, length: 9 }, // &angmsdaa;
    HTMLEntityTableEntry { first_value: 0x029A9, second_value: 0x0000, entity_offset: 5223, length: 9 }, // &angmsdab;
    HTMLEntityTableEntry { first_value: 0x029AA, second_value: 0x0000, entity_offset: 5232, length: 9 }, // &angmsdac;
    HTMLEntityTableEntry { first_value: 0x029AB, second_value: 0x0000, entity_offset: 5241, length: 9 }, // &angmsdad;
    HTMLEntityTableEntry { first_value: 0x029AC, second_value: 0x0000, entity_offset: 5250, length: 9 }, // &angmsdae;
// cpp: html/parser/html_entity_table.cc:2716-2747
    HTMLEntityTableEntry { first_value: 0x029AD, second_value: 0x0000, entity_offset: 5259, length: 9 }, // &angmsdaf;
    HTMLEntityTableEntry { first_value: 0x029AE, second_value: 0x0000, entity_offset: 5268, length: 9 }, // &angmsdag;
    HTMLEntityTableEntry { first_value: 0x029AF, second_value: 0x0000, entity_offset: 5277, length: 9 }, // &angmsdah;
    HTMLEntityTableEntry { first_value: 0x0221F, second_value: 0x0000, entity_offset: 5286, length: 6 }, // &angrt;
    HTMLEntityTableEntry { first_value: 0x022BE, second_value: 0x0000, entity_offset: 5292, length: 8 }, // &angrtvb;
    HTMLEntityTableEntry { first_value: 0x0299D, second_value: 0x0000, entity_offset: 5300, length: 9 }, // &angrtvbd;
    HTMLEntityTableEntry { first_value: 0x02222, second_value: 0x0000, entity_offset: 5309, length: 7 }, // &angsph;
    HTMLEntityTableEntry { first_value: 0x000C5, second_value: 0x0000, entity_offset: 5316, length: 6 }, // &angst;
    HTMLEntityTableEntry { first_value: 0x0237C, second_value: 0x0000, entity_offset: 5322, length: 8 }, // &angzarr;
    HTMLEntityTableEntry { first_value: 0x00105, second_value: 0x0000, entity_offset: 5330, length: 6 }, // &aogon;
    HTMLEntityTableEntry { first_value: 0x1D552, second_value: 0x0000, entity_offset: 5336, length: 5 }, // &aopf;
    HTMLEntityTableEntry { first_value: 0x02248, second_value: 0x0000, entity_offset: 211, length: 3 }, // &ap;
    HTMLEntityTableEntry { first_value: 0x02A70, second_value: 0x0000, entity_offset: 5341, length: 4 }, // &apE;
    HTMLEntityTableEntry { first_value: 0x02A6F, second_value: 0x0000, entity_offset: 5345, length: 7 }, // &apacir;
    HTMLEntityTableEntry { first_value: 0x0224A, second_value: 0x0000, entity_offset: 5352, length: 4 }, // &ape;
    HTMLEntityTableEntry { first_value: 0x0224B, second_value: 0x0000, entity_offset: 5356, length: 5 }, // &apid;
    HTMLEntityTableEntry { first_value: 0x00027, second_value: 0x0000, entity_offset: 5361, length: 5 }, // &apos;
    HTMLEntityTableEntry { first_value: 0x02248, second_value: 0x0000, entity_offset: 5366, length: 7 }, // &approx;
    HTMLEntityTableEntry { first_value: 0x0224A, second_value: 0x0000, entity_offset: 5373, length: 9 }, // &approxeq;
    HTMLEntityTableEntry { first_value: 0x000E5, second_value: 0x0000, entity_offset: 5382, length: 5 }, // &aring
    HTMLEntityTableEntry { first_value: 0x000E5, second_value: 0x0000, entity_offset: 5382, length: 6 }, // &aring;
    HTMLEntityTableEntry { first_value: 0x1D4B6, second_value: 0x0000, entity_offset: 5388, length: 5 }, // &ascr;
    HTMLEntityTableEntry { first_value: 0x0002A, second_value: 0x0000, entity_offset: 5393, length: 4 }, // &ast;
    HTMLEntityTableEntry { first_value: 0x02248, second_value: 0x0000, entity_offset: 5397, length: 6 }, // &asymp;
    HTMLEntityTableEntry { first_value: 0x0224D, second_value: 0x0000, entity_offset: 5403, length: 8 }, // &asympeq;
    HTMLEntityTableEntry { first_value: 0x000E3, second_value: 0x0000, entity_offset: 5411, length: 6 }, // &atilde
    HTMLEntityTableEntry { first_value: 0x000E3, second_value: 0x0000, entity_offset: 5411, length: 7 }, // &atilde;
    HTMLEntityTableEntry { first_value: 0x000E4, second_value: 0x0000, entity_offset: 5418, length: 4 }, // &auml
    HTMLEntityTableEntry { first_value: 0x000E4, second_value: 0x0000, entity_offset: 5418, length: 5 }, // &auml;
    HTMLEntityTableEntry { first_value: 0x02233, second_value: 0x0000, entity_offset: 5423, length: 9 }, // &awconint;
    HTMLEntityTableEntry { first_value: 0x02A11, second_value: 0x0000, entity_offset: 5432, length: 6 }, // &awint;
    HTMLEntityTableEntry { first_value: 0x02AED, second_value: 0x0000, entity_offset: 5438, length: 5 }, // &bNot;
// cpp: html/parser/html_entity_table.cc:2748-2779
    HTMLEntityTableEntry { first_value: 0x0224C, second_value: 0x0000, entity_offset: 5443, length: 9 }, // &backcong;
    HTMLEntityTableEntry { first_value: 0x003F6, second_value: 0x0000, entity_offset: 5452, length: 12 }, // &backepsilon;
    HTMLEntityTableEntry { first_value: 0x02035, second_value: 0x0000, entity_offset: 5464, length: 10 }, // &backprime;
    HTMLEntityTableEntry { first_value: 0x0223D, second_value: 0x0000, entity_offset: 5474, length: 8 }, // &backsim;
    HTMLEntityTableEntry { first_value: 0x022CD, second_value: 0x0000, entity_offset: 5482, length: 10 }, // &backsimeq;
    HTMLEntityTableEntry { first_value: 0x022BD, second_value: 0x0000, entity_offset: 5492, length: 7 }, // &barvee;
    HTMLEntityTableEntry { first_value: 0x02305, second_value: 0x0000, entity_offset: 5499, length: 7 }, // &barwed;
    HTMLEntityTableEntry { first_value: 0x02305, second_value: 0x0000, entity_offset: 5506, length: 9 }, // &barwedge;
    HTMLEntityTableEntry { first_value: 0x023B5, second_value: 0x0000, entity_offset: 5515, length: 5 }, // &bbrk;
    HTMLEntityTableEntry { first_value: 0x023B6, second_value: 0x0000, entity_offset: 5520, length: 9 }, // &bbrktbrk;
    HTMLEntityTableEntry { first_value: 0x0224C, second_value: 0x0000, entity_offset: 5529, length: 6 }, // &bcong;
    HTMLEntityTableEntry { first_value: 0x00431, second_value: 0x0000, entity_offset: 5535, length: 4 }, // &bcy;
    HTMLEntityTableEntry { first_value: 0x0201E, second_value: 0x0000, entity_offset: 5539, length: 6 }, // &bdquo;
    HTMLEntityTableEntry { first_value: 0x02235, second_value: 0x0000, entity_offset: 5545, length: 7 }, // &becaus;
    HTMLEntityTableEntry { first_value: 0x02235, second_value: 0x0000, entity_offset: 5552, length: 8 }, // &because;
    HTMLEntityTableEntry { first_value: 0x029B0, second_value: 0x0000, entity_offset: 5560, length: 8 }, // &bemptyv;
    HTMLEntityTableEntry { first_value: 0x003F6, second_value: 0x0000, entity_offset: 5568, length: 6 }, // &bepsi;
    HTMLEntityTableEntry { first_value: 0x0212C, second_value: 0x0000, entity_offset: 5574, length: 7 }, // &bernou;
    HTMLEntityTableEntry { first_value: 0x003B2, second_value: 0x0000, entity_offset: 5581, length: 5 }, // &beta;
    HTMLEntityTableEntry { first_value: 0x02136, second_value: 0x0000, entity_offset: 5586, length: 5 }, // &beth;
    HTMLEntityTableEntry { first_value: 0x0226C, second_value: 0x0000, entity_offset: 5591, length: 8 }, // &between;
    HTMLEntityTableEntry { first_value: 0x1D51F, second_value: 0x0000, entity_offset: 5599, length: 4 }, // &bfr;
    HTMLEntityTableEntry { first_value: 0x022C2, second_value: 0x0000, entity_offset: 5603, length: 7 }, // &bigcap;
    HTMLEntityTableEntry { first_value: 0x025EF, second_value: 0x0000, entity_offset: 5610, length: 8 }, // &bigcirc;
    HTMLEntityTableEntry { first_value: 0x022C3, second_value: 0x0000, entity_offset: 5618, length: 7 }, // &bigcup;
    HTMLEntityTableEntry { first_value: 0x02A00, second_value: 0x0000, entity_offset: 5625, length: 8 }, // &bigodot;
    HTMLEntityTableEntry { first_value: 0x02A01, second_value: 0x0000, entity_offset: 5633, length: 9 }, // &bigoplus;
    HTMLEntityTableEntry { first_value: 0x02A02, second_value: 0x0000, entity_offset: 5642, length: 10 }, // &bigotimes;
    HTMLEntityTableEntry { first_value: 0x02A06, second_value: 0x0000, entity_offset: 5652, length: 9 }, // &bigsqcup;
    HTMLEntityTableEntry { first_value: 0x02605, second_value: 0x0000, entity_offset: 5661, length: 8 }, // &bigstar;
    HTMLEntityTableEntry { first_value: 0x025BD, second_value: 0x0000, entity_offset: 5669, length: 16 }, // &bigtriangledown;
    HTMLEntityTableEntry { first_value: 0x025B3, second_value: 0x0000, entity_offset: 5685, length: 14 }, // &bigtriangleup;
// cpp: html/parser/html_entity_table.cc:2780-2811
    HTMLEntityTableEntry { first_value: 0x02A04, second_value: 0x0000, entity_offset: 5699, length: 9 }, // &biguplus;
    HTMLEntityTableEntry { first_value: 0x022C1, second_value: 0x0000, entity_offset: 5708, length: 7 }, // &bigvee;
    HTMLEntityTableEntry { first_value: 0x022C0, second_value: 0x0000, entity_offset: 5715, length: 9 }, // &bigwedge;
    HTMLEntityTableEntry { first_value: 0x0290D, second_value: 0x0000, entity_offset: 5724, length: 7 }, // &bkarow;
    HTMLEntityTableEntry { first_value: 0x029EB, second_value: 0x0000, entity_offset: 5731, length: 13 }, // &blacklozenge;
    HTMLEntityTableEntry { first_value: 0x025AA, second_value: 0x0000, entity_offset: 5744, length: 12 }, // &blacksquare;
    HTMLEntityTableEntry { first_value: 0x025B4, second_value: 0x0000, entity_offset: 5756, length: 14 }, // &blacktriangle;
    HTMLEntityTableEntry { first_value: 0x025BE, second_value: 0x0000, entity_offset: 5770, length: 18 }, // &blacktriangledown;
    HTMLEntityTableEntry { first_value: 0x025C2, second_value: 0x0000, entity_offset: 5788, length: 18 }, // &blacktriangleleft;
    HTMLEntityTableEntry { first_value: 0x025B8, second_value: 0x0000, entity_offset: 5806, length: 19 }, // &blacktriangleright;
    HTMLEntityTableEntry { first_value: 0x02423, second_value: 0x0000, entity_offset: 5825, length: 6 }, // &blank;
    HTMLEntityTableEntry { first_value: 0x02592, second_value: 0x0000, entity_offset: 5831, length: 6 }, // &blk12;
    HTMLEntityTableEntry { first_value: 0x02591, second_value: 0x0000, entity_offset: 5837, length: 6 }, // &blk14;
    HTMLEntityTableEntry { first_value: 0x02593, second_value: 0x0000, entity_offset: 5843, length: 6 }, // &blk34;
    HTMLEntityTableEntry { first_value: 0x02588, second_value: 0x0000, entity_offset: 5849, length: 6 }, // &block;
    HTMLEntityTableEntry { first_value: 0x0003D, second_value: 0x20E5, entity_offset: 5855, length: 4 }, // &bne;
    HTMLEntityTableEntry { first_value: 0x02261, second_value: 0x20E5, entity_offset: 5859, length: 8 }, // &bnequiv;
    HTMLEntityTableEntry { first_value: 0x02310, second_value: 0x0000, entity_offset: 5867, length: 5 }, // &bnot;
    HTMLEntityTableEntry { first_value: 0x1D553, second_value: 0x0000, entity_offset: 5872, length: 5 }, // &bopf;
    HTMLEntityTableEntry { first_value: 0x022A5, second_value: 0x0000, entity_offset: 5877, length: 4 }, // &bot;
    HTMLEntityTableEntry { first_value: 0x022A5, second_value: 0x0000, entity_offset: 5881, length: 7 }, // &bottom;
    HTMLEntityTableEntry { first_value: 0x022C8, second_value: 0x0000, entity_offset: 5888, length: 7 }, // &bowtie;
    HTMLEntityTableEntry { first_value: 0x02557, second_value: 0x0000, entity_offset: 5895, length: 6 }, // &boxDL;
    HTMLEntityTableEntry { first_value: 0x02554, second_value: 0x0000, entity_offset: 5901, length: 6 }, // &boxDR;
    HTMLEntityTableEntry { first_value: 0x02556, second_value: 0x0000, entity_offset: 5907, length: 6 }, // &boxDl;
    HTMLEntityTableEntry { first_value: 0x02553, second_value: 0x0000, entity_offset: 5913, length: 6 }, // &boxDr;
    HTMLEntityTableEntry { first_value: 0x02550, second_value: 0x0000, entity_offset: 5919, length: 5 }, // &boxH;
    HTMLEntityTableEntry { first_value: 0x02566, second_value: 0x0000, entity_offset: 5924, length: 6 }, // &boxHD;
    HTMLEntityTableEntry { first_value: 0x02569, second_value: 0x0000, entity_offset: 5930, length: 6 }, // &boxHU;
    HTMLEntityTableEntry { first_value: 0x02564, second_value: 0x0000, entity_offset: 5936, length: 6 }, // &boxHd;
    HTMLEntityTableEntry { first_value: 0x02567, second_value: 0x0000, entity_offset: 5942, length: 6 }, // &boxHu;
    HTMLEntityTableEntry { first_value: 0x0255D, second_value: 0x0000, entity_offset: 5948, length: 6 }, // &boxUL;
// cpp: html/parser/html_entity_table.cc:2812-2843
    HTMLEntityTableEntry { first_value: 0x0255A, second_value: 0x0000, entity_offset: 5954, length: 6 }, // &boxUR;
    HTMLEntityTableEntry { first_value: 0x0255C, second_value: 0x0000, entity_offset: 5960, length: 6 }, // &boxUl;
    HTMLEntityTableEntry { first_value: 0x02559, second_value: 0x0000, entity_offset: 5966, length: 6 }, // &boxUr;
    HTMLEntityTableEntry { first_value: 0x02551, second_value: 0x0000, entity_offset: 5972, length: 5 }, // &boxV;
    HTMLEntityTableEntry { first_value: 0x0256C, second_value: 0x0000, entity_offset: 5977, length: 6 }, // &boxVH;
    HTMLEntityTableEntry { first_value: 0x02563, second_value: 0x0000, entity_offset: 5983, length: 6 }, // &boxVL;
    HTMLEntityTableEntry { first_value: 0x02560, second_value: 0x0000, entity_offset: 5989, length: 6 }, // &boxVR;
    HTMLEntityTableEntry { first_value: 0x0256B, second_value: 0x0000, entity_offset: 5995, length: 6 }, // &boxVh;
    HTMLEntityTableEntry { first_value: 0x02562, second_value: 0x0000, entity_offset: 6001, length: 6 }, // &boxVl;
    HTMLEntityTableEntry { first_value: 0x0255F, second_value: 0x0000, entity_offset: 6007, length: 6 }, // &boxVr;
    HTMLEntityTableEntry { first_value: 0x029C9, second_value: 0x0000, entity_offset: 6013, length: 7 }, // &boxbox;
    HTMLEntityTableEntry { first_value: 0x02555, second_value: 0x0000, entity_offset: 6020, length: 6 }, // &boxdL;
    HTMLEntityTableEntry { first_value: 0x02552, second_value: 0x0000, entity_offset: 6026, length: 6 }, // &boxdR;
    HTMLEntityTableEntry { first_value: 0x02510, second_value: 0x0000, entity_offset: 6032, length: 6 }, // &boxdl;
    HTMLEntityTableEntry { first_value: 0x0250C, second_value: 0x0000, entity_offset: 6038, length: 6 }, // &boxdr;
    HTMLEntityTableEntry { first_value: 0x02500, second_value: 0x0000, entity_offset: 6044, length: 5 }, // &boxh;
    HTMLEntityTableEntry { first_value: 0x02565, second_value: 0x0000, entity_offset: 6049, length: 6 }, // &boxhD;
    HTMLEntityTableEntry { first_value: 0x02568, second_value: 0x0000, entity_offset: 6055, length: 6 }, // &boxhU;
    HTMLEntityTableEntry { first_value: 0x0252C, second_value: 0x0000, entity_offset: 6061, length: 6 }, // &boxhd;
    HTMLEntityTableEntry { first_value: 0x02534, second_value: 0x0000, entity_offset: 6067, length: 6 }, // &boxhu;
    HTMLEntityTableEntry { first_value: 0x0229F, second_value: 0x0000, entity_offset: 6073, length: 9 }, // &boxminus;
    HTMLEntityTableEntry { first_value: 0x0229E, second_value: 0x0000, entity_offset: 6082, length: 8 }, // &boxplus;
    HTMLEntityTableEntry { first_value: 0x022A0, second_value: 0x0000, entity_offset: 6090, length: 9 }, // &boxtimes;
    HTMLEntityTableEntry { first_value: 0x0255B, second_value: 0x0000, entity_offset: 6099, length: 6 }, // &boxuL;
    HTMLEntityTableEntry { first_value: 0x02558, second_value: 0x0000, entity_offset: 6105, length: 6 }, // &boxuR;
    HTMLEntityTableEntry { first_value: 0x02518, second_value: 0x0000, entity_offset: 6111, length: 6 }, // &boxul;
    HTMLEntityTableEntry { first_value: 0x02514, second_value: 0x0000, entity_offset: 6117, length: 6 }, // &boxur;
    HTMLEntityTableEntry { first_value: 0x02502, second_value: 0x0000, entity_offset: 6123, length: 5 }, // &boxv;
    HTMLEntityTableEntry { first_value: 0x0256A, second_value: 0x0000, entity_offset: 6128, length: 6 }, // &boxvH;
    HTMLEntityTableEntry { first_value: 0x02561, second_value: 0x0000, entity_offset: 6134, length: 6 }, // &boxvL;
    HTMLEntityTableEntry { first_value: 0x0255E, second_value: 0x0000, entity_offset: 6140, length: 6 }, // &boxvR;
    HTMLEntityTableEntry { first_value: 0x0253C, second_value: 0x0000, entity_offset: 6146, length: 6 }, // &boxvh;
// cpp: html/parser/html_entity_table.cc:2844-2875
    HTMLEntityTableEntry { first_value: 0x02524, second_value: 0x0000, entity_offset: 6152, length: 6 }, // &boxvl;
    HTMLEntityTableEntry { first_value: 0x0251C, second_value: 0x0000, entity_offset: 6158, length: 6 }, // &boxvr;
    HTMLEntityTableEntry { first_value: 0x02035, second_value: 0x0000, entity_offset: 6164, length: 7 }, // &bprime;
    HTMLEntityTableEntry { first_value: 0x002D8, second_value: 0x0000, entity_offset: 18, length: 6 }, // &breve;
    HTMLEntityTableEntry { first_value: 0x000A6, second_value: 0x0000, entity_offset: 6171, length: 6 }, // &brvbar
    HTMLEntityTableEntry { first_value: 0x000A6, second_value: 0x0000, entity_offset: 6171, length: 7 }, // &brvbar;
    HTMLEntityTableEntry { first_value: 0x1D4B7, second_value: 0x0000, entity_offset: 6178, length: 5 }, // &bscr;
    HTMLEntityTableEntry { first_value: 0x0204F, second_value: 0x0000, entity_offset: 6183, length: 6 }, // &bsemi;
    HTMLEntityTableEntry { first_value: 0x0223D, second_value: 0x0000, entity_offset: 6189, length: 5 }, // &bsim;
    HTMLEntityTableEntry { first_value: 0x022CD, second_value: 0x0000, entity_offset: 6194, length: 6 }, // &bsime;
    HTMLEntityTableEntry { first_value: 0x0005C, second_value: 0x0000, entity_offset: 6200, length: 5 }, // &bsol;
    HTMLEntityTableEntry { first_value: 0x029C5, second_value: 0x0000, entity_offset: 6205, length: 6 }, // &bsolb;
    HTMLEntityTableEntry { first_value: 0x027C8, second_value: 0x0000, entity_offset: 6211, length: 9 }, // &bsolhsub;
    HTMLEntityTableEntry { first_value: 0x02022, second_value: 0x0000, entity_offset: 6220, length: 5 }, // &bull;
    HTMLEntityTableEntry { first_value: 0x02022, second_value: 0x0000, entity_offset: 6225, length: 7 }, // &bullet;
    HTMLEntityTableEntry { first_value: 0x0224E, second_value: 0x0000, entity_offset: 6232, length: 5 }, // &bump;
    HTMLEntityTableEntry { first_value: 0x02AAE, second_value: 0x0000, entity_offset: 6237, length: 6 }, // &bumpE;
    HTMLEntityTableEntry { first_value: 0x0224F, second_value: 0x0000, entity_offset: 6243, length: 6 }, // &bumpe;
    HTMLEntityTableEntry { first_value: 0x0224F, second_value: 0x0000, entity_offset: 6249, length: 7 }, // &bumpeq;
    HTMLEntityTableEntry { first_value: 0x00107, second_value: 0x0000, entity_offset: 6256, length: 7 }, // &cacute;
    HTMLEntityTableEntry { first_value: 0x02229, second_value: 0x0000, entity_offset: 5606, length: 4 }, // &cap;
    HTMLEntityTableEntry { first_value: 0x02A44, second_value: 0x0000, entity_offset: 6263, length: 7 }, // &capand;
    HTMLEntityTableEntry { first_value: 0x02A49, second_value: 0x0000, entity_offset: 6270, length: 9 }, // &capbrcup;
    HTMLEntityTableEntry { first_value: 0x02A4B, second_value: 0x0000, entity_offset: 6279, length: 7 }, // &capcap;
    HTMLEntityTableEntry { first_value: 0x02A47, second_value: 0x0000, entity_offset: 6286, length: 7 }, // &capcup;
    HTMLEntityTableEntry { first_value: 0x02A40, second_value: 0x0000, entity_offset: 6293, length: 7 }, // &capdot;
    HTMLEntityTableEntry { first_value: 0x02229, second_value: 0xFE00, entity_offset: 6300, length: 5 }, // &caps;
    HTMLEntityTableEntry { first_value: 0x02041, second_value: 0x0000, entity_offset: 6305, length: 6 }, // &caret;
    HTMLEntityTableEntry { first_value: 0x002C7, second_value: 0x0000, entity_offset: 244, length: 6 }, // &caron;
    HTMLEntityTableEntry { first_value: 0x02A4D, second_value: 0x0000, entity_offset: 6311, length: 6 }, // &ccaps;
    HTMLEntityTableEntry { first_value: 0x0010D, second_value: 0x0000, entity_offset: 6317, length: 7 }, // &ccaron;
    HTMLEntityTableEntry { first_value: 0x000E7, second_value: 0x0000, entity_offset: 6324, length: 6 }, // &ccedil
// cpp: html/parser/html_entity_table.cc:2876-2907
    HTMLEntityTableEntry { first_value: 0x000E7, second_value: 0x0000, entity_offset: 6324, length: 7 }, // &ccedil;
    HTMLEntityTableEntry { first_value: 0x00109, second_value: 0x0000, entity_offset: 6331, length: 6 }, // &ccirc;
    HTMLEntityTableEntry { first_value: 0x02A4C, second_value: 0x0000, entity_offset: 6337, length: 6 }, // &ccups;
    HTMLEntityTableEntry { first_value: 0x02A50, second_value: 0x0000, entity_offset: 6343, length: 8 }, // &ccupssm;
    HTMLEntityTableEntry { first_value: 0x0010B, second_value: 0x0000, entity_offset: 6351, length: 5 }, // &cdot;
    HTMLEntityTableEntry { first_value: 0x000B8, second_value: 0x0000, entity_offset: 251, length: 5 }, // &cedil
    HTMLEntityTableEntry { first_value: 0x000B8, second_value: 0x0000, entity_offset: 251, length: 6 }, // &cedil;
    HTMLEntityTableEntry { first_value: 0x029B2, second_value: 0x0000, entity_offset: 6356, length: 8 }, // &cemptyv;
    HTMLEntityTableEntry { first_value: 0x000A2, second_value: 0x0000, entity_offset: 6364, length: 4 }, // &cent
    HTMLEntityTableEntry { first_value: 0x000A2, second_value: 0x0000, entity_offset: 6364, length: 5 }, // &cent;
    HTMLEntityTableEntry { first_value: 0x000B7, second_value: 0x0000, entity_offset: 6369, length: 10 }, // &centerdot;
    HTMLEntityTableEntry { first_value: 0x1D520, second_value: 0x0000, entity_offset: 6379, length: 4 }, // &cfr;
    HTMLEntityTableEntry { first_value: 0x00447, second_value: 0x0000, entity_offset: 6383, length: 5 }, // &chcy;
    HTMLEntityTableEntry { first_value: 0x02713, second_value: 0x0000, entity_offset: 6388, length: 6 }, // &check;
    HTMLEntityTableEntry { first_value: 0x02713, second_value: 0x0000, entity_offset: 6394, length: 10 }, // &checkmark;
    HTMLEntityTableEntry { first_value: 0x003C7, second_value: 0x0000, entity_offset: 6404, length: 4 }, // &chi;
    HTMLEntityTableEntry { first_value: 0x025CB, second_value: 0x0000, entity_offset: 4568, length: 4 }, // &cir;
    HTMLEntityTableEntry { first_value: 0x029C3, second_value: 0x0000, entity_offset: 6408, length: 5 }, // &cirE;
    HTMLEntityTableEntry { first_value: 0x002C6, second_value: 0x0000, entity_offset: 25, length: 5 }, // &circ;
    HTMLEntityTableEntry { first_value: 0x02257, second_value: 0x0000, entity_offset: 6413, length: 7 }, // &circeq;
    HTMLEntityTableEntry { first_value: 0x021BA, second_value: 0x0000, entity_offset: 6420, length: 16 }, // &circlearrowleft;
    HTMLEntityTableEntry { first_value: 0x021BB, second_value: 0x0000, entity_offset: 6436, length: 17 }, // &circlearrowright;
    HTMLEntityTableEntry { first_value: 0x000AE, second_value: 0x0000, entity_offset: 6453, length: 9 }, // &circledR;
    HTMLEntityTableEntry { first_value: 0x024C8, second_value: 0x0000, entity_offset: 6462, length: 9 }, // &circledS;
    HTMLEntityTableEntry { first_value: 0x0229B, second_value: 0x0000, entity_offset: 6471, length: 11 }, // &circledast;
    HTMLEntityTableEntry { first_value: 0x0229A, second_value: 0x0000, entity_offset: 6482, length: 12 }, // &circledcirc;
    HTMLEntityTableEntry { first_value: 0x0229D, second_value: 0x0000, entity_offset: 6494, length: 12 }, // &circleddash;
    HTMLEntityTableEntry { first_value: 0x02257, second_value: 0x0000, entity_offset: 6506, length: 5 }, // &cire;
    HTMLEntityTableEntry { first_value: 0x02A10, second_value: 0x0000, entity_offset: 6511, length: 9 }, // &cirfnint;
    HTMLEntityTableEntry { first_value: 0x02AEF, second_value: 0x0000, entity_offset: 6520, length: 7 }, // &cirmid;
    HTMLEntityTableEntry { first_value: 0x029C2, second_value: 0x0000, entity_offset: 6527, length: 8 }, // &cirscir;
    HTMLEntityTableEntry { first_value: 0x02663, second_value: 0x0000, entity_offset: 6535, length: 6 }, // &clubs;
// cpp: html/parser/html_entity_table.cc:2908-2939
    HTMLEntityTableEntry { first_value: 0x02663, second_value: 0x0000, entity_offset: 6541, length: 9 }, // &clubsuit;
    HTMLEntityTableEntry { first_value: 0x0003A, second_value: 0x0000, entity_offset: 6550, length: 6 }, // &colon;
    HTMLEntityTableEntry { first_value: 0x02254, second_value: 0x0000, entity_offset: 6556, length: 7 }, // &colone;
    HTMLEntityTableEntry { first_value: 0x02254, second_value: 0x0000, entity_offset: 6563, length: 8 }, // &coloneq;
    HTMLEntityTableEntry { first_value: 0x0002C, second_value: 0x0000, entity_offset: 6571, length: 6 }, // &comma;
    HTMLEntityTableEntry { first_value: 0x00040, second_value: 0x0000, entity_offset: 6577, length: 7 }, // &commat;
    HTMLEntityTableEntry { first_value: 0x02201, second_value: 0x0000, entity_offset: 6584, length: 5 }, // &comp;
    HTMLEntityTableEntry { first_value: 0x02218, second_value: 0x0000, entity_offset: 6589, length: 7 }, // &compfn;
    HTMLEntityTableEntry { first_value: 0x02201, second_value: 0x0000, entity_offset: 6596, length: 11 }, // &complement;
    HTMLEntityTableEntry { first_value: 0x02102, second_value: 0x0000, entity_offset: 6607, length: 10 }, // &complexes;
    HTMLEntityTableEntry { first_value: 0x02245, second_value: 0x0000, entity_offset: 5447, length: 5 }, // &cong;
    HTMLEntityTableEntry { first_value: 0x02A6D, second_value: 0x0000, entity_offset: 6617, length: 8 }, // &congdot;
    HTMLEntityTableEntry { first_value: 0x0222E, second_value: 0x0000, entity_offset: 264, length: 7 }, // &conint;
    HTMLEntityTableEntry { first_value: 0x1D554, second_value: 0x0000, entity_offset: 6625, length: 5 }, // &copf;
    HTMLEntityTableEntry { first_value: 0x02210, second_value: 0x0000, entity_offset: 6630, length: 7 }, // &coprod;
    HTMLEntityTableEntry { first_value: 0x000A9, second_value: 0x0000, entity_offset: 6637, length: 4 }, // &copy
    HTMLEntityTableEntry { first_value: 0x000A9, second_value: 0x0000, entity_offset: 6637, length: 5 }, // &copy;
    HTMLEntityTableEntry { first_value: 0x02117, second_value: 0x0000, entity_offset: 6642, length: 7 }, // &copysr;
    HTMLEntityTableEntry { first_value: 0x021B5, second_value: 0x0000, entity_offset: 6649, length: 6 }, // &crarr;
    HTMLEntityTableEntry { first_value: 0x02717, second_value: 0x0000, entity_offset: 6655, length: 6 }, // &cross;
    HTMLEntityTableEntry { first_value: 0x1D4B8, second_value: 0x0000, entity_offset: 6661, length: 5 }, // &cscr;
    HTMLEntityTableEntry { first_value: 0x02ACF, second_value: 0x0000, entity_offset: 6666, length: 5 }, // &csub;
    HTMLEntityTableEntry { first_value: 0x02AD1, second_value: 0x0000, entity_offset: 6671, length: 6 }, // &csube;
    HTMLEntityTableEntry { first_value: 0x02AD0, second_value: 0x0000, entity_offset: 6677, length: 5 }, // &csup;
    HTMLEntityTableEntry { first_value: 0x02AD2, second_value: 0x0000, entity_offset: 6682, length: 6 }, // &csupe;
    HTMLEntityTableEntry { first_value: 0x022EF, second_value: 0x0000, entity_offset: 6688, length: 6 }, // &ctdot;
    HTMLEntityTableEntry { first_value: 0x02938, second_value: 0x0000, entity_offset: 6694, length: 8 }, // &cudarrl;
    HTMLEntityTableEntry { first_value: 0x02935, second_value: 0x0000, entity_offset: 6702, length: 8 }, // &cudarrr;
    HTMLEntityTableEntry { first_value: 0x022DE, second_value: 0x0000, entity_offset: 6710, length: 6 }, // &cuepr;
    HTMLEntityTableEntry { first_value: 0x022DF, second_value: 0x0000, entity_offset: 6716, length: 6 }, // &cuesc;
    HTMLEntityTableEntry { first_value: 0x021B6, second_value: 0x0000, entity_offset: 6722, length: 7 }, // &cularr;
    HTMLEntityTableEntry { first_value: 0x0293D, second_value: 0x0000, entity_offset: 6729, length: 8 }, // &cularrp;
// cpp: html/parser/html_entity_table.cc:2940-2971
    HTMLEntityTableEntry { first_value: 0x0222A, second_value: 0x0000, entity_offset: 5621, length: 4 }, // &cup;
    HTMLEntityTableEntry { first_value: 0x02A48, second_value: 0x0000, entity_offset: 6737, length: 9 }, // &cupbrcap;
    HTMLEntityTableEntry { first_value: 0x02A46, second_value: 0x0000, entity_offset: 6746, length: 7 }, // &cupcap;
    HTMLEntityTableEntry { first_value: 0x02A4A, second_value: 0x0000, entity_offset: 6753, length: 7 }, // &cupcup;
    HTMLEntityTableEntry { first_value: 0x0228D, second_value: 0x0000, entity_offset: 6760, length: 7 }, // &cupdot;
    HTMLEntityTableEntry { first_value: 0x02A45, second_value: 0x0000, entity_offset: 6767, length: 6 }, // &cupor;
    HTMLEntityTableEntry { first_value: 0x0222A, second_value: 0xFE00, entity_offset: 6338, length: 5 }, // &cups;
    HTMLEntityTableEntry { first_value: 0x021B7, second_value: 0x0000, entity_offset: 6773, length: 7 }, // &curarr;
    HTMLEntityTableEntry { first_value: 0x0293C, second_value: 0x0000, entity_offset: 6780, length: 8 }, // &curarrm;
    HTMLEntityTableEntry { first_value: 0x022DE, second_value: 0x0000, entity_offset: 6788, length: 12 }, // &curlyeqprec;
    HTMLEntityTableEntry { first_value: 0x022DF, second_value: 0x0000, entity_offset: 6800, length: 12 }, // &curlyeqsucc;
    HTMLEntityTableEntry { first_value: 0x022CE, second_value: 0x0000, entity_offset: 6812, length: 9 }, // &curlyvee;
    HTMLEntityTableEntry { first_value: 0x022CF, second_value: 0x0000, entity_offset: 6821, length: 11 }, // &curlywedge;
    HTMLEntityTableEntry { first_value: 0x000A4, second_value: 0x0000, entity_offset: 6832, length: 6 }, // &curren
    HTMLEntityTableEntry { first_value: 0x000A4, second_value: 0x0000, entity_offset: 6832, length: 7 }, // &curren;
    HTMLEntityTableEntry { first_value: 0x021B6, second_value: 0x0000, entity_offset: 6839, length: 15 }, // &curvearrowleft;
    HTMLEntityTableEntry { first_value: 0x021B7, second_value: 0x0000, entity_offset: 6854, length: 16 }, // &curvearrowright;
    HTMLEntityTableEntry { first_value: 0x022CE, second_value: 0x0000, entity_offset: 6870, length: 6 }, // &cuvee;
    HTMLEntityTableEntry { first_value: 0x022CF, second_value: 0x0000, entity_offset: 6876, length: 6 }, // &cuwed;
    HTMLEntityTableEntry { first_value: 0x02232, second_value: 0x0000, entity_offset: 6882, length: 9 }, // &cwconint;
    HTMLEntityTableEntry { first_value: 0x02231, second_value: 0x0000, entity_offset: 6891, length: 6 }, // &cwint;
    HTMLEntityTableEntry { first_value: 0x0232D, second_value: 0x0000, entity_offset: 6897, length: 7 }, // &cylcty;
    HTMLEntityTableEntry { first_value: 0x021D3, second_value: 0x0000, entity_offset: 6904, length: 5 }, // &dArr;
    HTMLEntityTableEntry { first_value: 0x02965, second_value: 0x0000, entity_offset: 6909, length: 5 }, // &dHar;
    HTMLEntityTableEntry { first_value: 0x02020, second_value: 0x0000, entity_offset: 6914, length: 7 }, // &dagger;
    HTMLEntityTableEntry { first_value: 0x02138, second_value: 0x0000, entity_offset: 6921, length: 7 }, // &daleth;
    HTMLEntityTableEntry { first_value: 0x02193, second_value: 0x0000, entity_offset: 6928, length: 5 }, // &darr;
    HTMLEntityTableEntry { first_value: 0x02010, second_value: 0x0000, entity_offset: 4838, length: 5 }, // &dash;
    HTMLEntityTableEntry { first_value: 0x022A3, second_value: 0x0000, entity_offset: 6933, length: 6 }, // &dashv;
    HTMLEntityTableEntry { first_value: 0x0290F, second_value: 0x0000, entity_offset: 6939, length: 8 }, // &dbkarow;
    HTMLEntityTableEntry { first_value: 0x002DD, second_value: 0x0000, entity_offset: 3562, length: 6 }, // &dblac;
    HTMLEntityTableEntry { first_value: 0x0010F, second_value: 0x0000, entity_offset: 6947, length: 7 }, // &dcaron;
// cpp: html/parser/html_entity_table.cc:2972-3003
    HTMLEntityTableEntry { first_value: 0x00434, second_value: 0x0000, entity_offset: 6954, length: 4 }, // &dcy;
    HTMLEntityTableEntry { first_value: 0x02146, second_value: 0x0000, entity_offset: 5185, length: 3 }, // &dd;
    HTMLEntityTableEntry { first_value: 0x02021, second_value: 0x0000, entity_offset: 6958, length: 8 }, // &ddagger;
    HTMLEntityTableEntry { first_value: 0x021CA, second_value: 0x0000, entity_offset: 6966, length: 6 }, // &ddarr;
    HTMLEntityTableEntry { first_value: 0x02A77, second_value: 0x0000, entity_offset: 6972, length: 8 }, // &ddotseq;
    HTMLEntityTableEntry { first_value: 0x000B0, second_value: 0x0000, entity_offset: 6980, length: 3 }, // &deg
    HTMLEntityTableEntry { first_value: 0x000B0, second_value: 0x0000, entity_offset: 6980, length: 4 }, // &deg;
    HTMLEntityTableEntry { first_value: 0x003B4, second_value: 0x0000, entity_offset: 6984, length: 6 }, // &delta;
    HTMLEntityTableEntry { first_value: 0x029B1, second_value: 0x0000, entity_offset: 6990, length: 8 }, // &demptyv;
    HTMLEntityTableEntry { first_value: 0x0297F, second_value: 0x0000, entity_offset: 6998, length: 7 }, // &dfisht;
    HTMLEntityTableEntry { first_value: 0x1D521, second_value: 0x0000, entity_offset: 7005, length: 4 }, // &dfr;
    HTMLEntityTableEntry { first_value: 0x021C3, second_value: 0x0000, entity_offset: 7009, length: 6 }, // &dharl;
    HTMLEntityTableEntry { first_value: 0x021C2, second_value: 0x0000, entity_offset: 7015, length: 6 }, // &dharr;
    HTMLEntityTableEntry { first_value: 0x022C4, second_value: 0x0000, entity_offset: 7021, length: 5 }, // &diam;
    HTMLEntityTableEntry { first_value: 0x022C4, second_value: 0x0000, entity_offset: 7026, length: 8 }, // &diamond;
    HTMLEntityTableEntry { first_value: 0x02666, second_value: 0x0000, entity_offset: 7034, length: 12 }, // &diamondsuit;
    HTMLEntityTableEntry { first_value: 0x02666, second_value: 0x0000, entity_offset: 7046, length: 6 }, // &diams;
    HTMLEntityTableEntry { first_value: 0x000A8, second_value: 0x0000, entity_offset: 7052, length: 4 }, // &die;
    HTMLEntityTableEntry { first_value: 0x003DD, second_value: 0x0000, entity_offset: 7056, length: 8 }, // &digamma;
    HTMLEntityTableEntry { first_value: 0x022F2, second_value: 0x0000, entity_offset: 7064, length: 6 }, // &disin;
    HTMLEntityTableEntry { first_value: 0x000F7, second_value: 0x0000, entity_offset: 7070, length: 4 }, // &div;
    HTMLEntityTableEntry { first_value: 0x000F7, second_value: 0x0000, entity_offset: 7074, length: 6 }, // &divide
    HTMLEntityTableEntry { first_value: 0x000F7, second_value: 0x0000, entity_offset: 7074, length: 7 }, // &divide;
    HTMLEntityTableEntry { first_value: 0x022C7, second_value: 0x0000, entity_offset: 7081, length: 14 }, // &divideontimes;
    HTMLEntityTableEntry { first_value: 0x022C7, second_value: 0x0000, entity_offset: 7095, length: 7 }, // &divonx;
    HTMLEntityTableEntry { first_value: 0x00452, second_value: 0x0000, entity_offset: 7102, length: 5 }, // &djcy;
    HTMLEntityTableEntry { first_value: 0x0231E, second_value: 0x0000, entity_offset: 7107, length: 7 }, // &dlcorn;
    HTMLEntityTableEntry { first_value: 0x0230D, second_value: 0x0000, entity_offset: 7114, length: 7 }, // &dlcrop;
    HTMLEntityTableEntry { first_value: 0x00024, second_value: 0x0000, entity_offset: 7121, length: 7 }, // &dollar;
    HTMLEntityTableEntry { first_value: 0x1D555, second_value: 0x0000, entity_offset: 7128, length: 5 }, // &dopf;
    HTMLEntityTableEntry { first_value: 0x002D9, second_value: 0x0000, entity_offset: 272, length: 4 }, // &dot;
    HTMLEntityTableEntry { first_value: 0x02250, second_value: 0x0000, entity_offset: 7133, length: 6 }, // &doteq;
// cpp: html/parser/html_entity_table.cc:3004-3035
    HTMLEntityTableEntry { first_value: 0x02251, second_value: 0x0000, entity_offset: 7139, length: 9 }, // &doteqdot;
    HTMLEntityTableEntry { first_value: 0x02238, second_value: 0x0000, entity_offset: 7148, length: 9 }, // &dotminus;
    HTMLEntityTableEntry { first_value: 0x02214, second_value: 0x0000, entity_offset: 7157, length: 8 }, // &dotplus;
    HTMLEntityTableEntry { first_value: 0x022A1, second_value: 0x0000, entity_offset: 7165, length: 10 }, // &dotsquare;
    HTMLEntityTableEntry { first_value: 0x02306, second_value: 0x0000, entity_offset: 7175, length: 15 }, // &doublebarwedge;
    HTMLEntityTableEntry { first_value: 0x02193, second_value: 0x0000, entity_offset: 4745, length: 10 }, // &downarrow;
    HTMLEntityTableEntry { first_value: 0x021CA, second_value: 0x0000, entity_offset: 7190, length: 15 }, // &downdownarrows;
    HTMLEntityTableEntry { first_value: 0x021C3, second_value: 0x0000, entity_offset: 7205, length: 16 }, // &downharpoonleft;
    HTMLEntityTableEntry { first_value: 0x021C2, second_value: 0x0000, entity_offset: 7221, length: 17 }, // &downharpoonright;
    HTMLEntityTableEntry { first_value: 0x02910, second_value: 0x0000, entity_offset: 7238, length: 9 }, // &drbkarow;
    HTMLEntityTableEntry { first_value: 0x0231F, second_value: 0x0000, entity_offset: 7247, length: 7 }, // &drcorn;
    HTMLEntityTableEntry { first_value: 0x0230C, second_value: 0x0000, entity_offset: 7254, length: 7 }, // &drcrop;
    HTMLEntityTableEntry { first_value: 0x1D4B9, second_value: 0x0000, entity_offset: 7261, length: 5 }, // &dscr;
    HTMLEntityTableEntry { first_value: 0x00455, second_value: 0x0000, entity_offset: 7266, length: 5 }, // &dscy;
    HTMLEntityTableEntry { first_value: 0x029F6, second_value: 0x0000, entity_offset: 7271, length: 5 }, // &dsol;
    HTMLEntityTableEntry { first_value: 0x00111, second_value: 0x0000, entity_offset: 7276, length: 7 }, // &dstrok;
    HTMLEntityTableEntry { first_value: 0x022F1, second_value: 0x0000, entity_offset: 7283, length: 6 }, // &dtdot;
    HTMLEntityTableEntry { first_value: 0x025BF, second_value: 0x0000, entity_offset: 7289, length: 5 }, // &dtri;
    HTMLEntityTableEntry { first_value: 0x025BE, second_value: 0x0000, entity_offset: 7294, length: 6 }, // &dtrif;
    HTMLEntityTableEntry { first_value: 0x021F5, second_value: 0x0000, entity_offset: 7300, length: 6 }, // &duarr;
    HTMLEntityTableEntry { first_value: 0x0296F, second_value: 0x0000, entity_offset: 7306, length: 6 }, // &duhar;
    HTMLEntityTableEntry { first_value: 0x029A6, second_value: 0x0000, entity_offset: 7312, length: 8 }, // &dwangle;
    HTMLEntityTableEntry { first_value: 0x0045F, second_value: 0x0000, entity_offset: 7320, length: 5 }, // &dzcy;
    HTMLEntityTableEntry { first_value: 0x027FF, second_value: 0x0000, entity_offset: 7325, length: 9 }, // &dzigrarr;
    HTMLEntityTableEntry { first_value: 0x02A77, second_value: 0x0000, entity_offset: 7334, length: 6 }, // &eDDot;
    HTMLEntityTableEntry { first_value: 0x02251, second_value: 0x0000, entity_offset: 307, length: 5 }, // &eDot;
    HTMLEntityTableEntry { first_value: 0x000E9, second_value: 0x0000, entity_offset: 7340, length: 6 }, // &eacute
    HTMLEntityTableEntry { first_value: 0x000E9, second_value: 0x0000, entity_offset: 7340, length: 7 }, // &eacute;
    HTMLEntityTableEntry { first_value: 0x02A6E, second_value: 0x0000, entity_offset: 7347, length: 7 }, // &easter;
    HTMLEntityTableEntry { first_value: 0x0011B, second_value: 0x0000, entity_offset: 7354, length: 7 }, // &ecaron;
    HTMLEntityTableEntry { first_value: 0x02256, second_value: 0x0000, entity_offset: 7361, length: 5 }, // &ecir;
    HTMLEntityTableEntry { first_value: 0x000EA, second_value: 0x0000, entity_offset: 7366, length: 5 }, // &ecirc
// cpp: html/parser/html_entity_table.cc:3036-3067
    HTMLEntityTableEntry { first_value: 0x000EA, second_value: 0x0000, entity_offset: 7366, length: 6 }, // &ecirc;
    HTMLEntityTableEntry { first_value: 0x02255, second_value: 0x0000, entity_offset: 7372, length: 7 }, // &ecolon;
    HTMLEntityTableEntry { first_value: 0x0044D, second_value: 0x0000, entity_offset: 7379, length: 4 }, // &ecy;
    HTMLEntityTableEntry { first_value: 0x00117, second_value: 0x0000, entity_offset: 7383, length: 5 }, // &edot;
    HTMLEntityTableEntry { first_value: 0x02147, second_value: 0x0000, entity_offset: 793, length: 3 }, // &ee;
    HTMLEntityTableEntry { first_value: 0x02252, second_value: 0x0000, entity_offset: 7388, length: 6 }, // &efDot;
    HTMLEntityTableEntry { first_value: 0x1D522, second_value: 0x0000, entity_offset: 7394, length: 4 }, // &efr;
    HTMLEntityTableEntry { first_value: 0x02A9A, second_value: 0x0000, entity_offset: 6981, length: 3 }, // &eg;
    HTMLEntityTableEntry { first_value: 0x000E8, second_value: 0x0000, entity_offset: 7398, length: 6 }, // &egrave
    HTMLEntityTableEntry { first_value: 0x000E8, second_value: 0x0000, entity_offset: 7398, length: 7 }, // &egrave;
    HTMLEntityTableEntry { first_value: 0x02A96, second_value: 0x0000, entity_offset: 7405, length: 4 }, // &egs;
    HTMLEntityTableEntry { first_value: 0x02A98, second_value: 0x0000, entity_offset: 7409, length: 7 }, // &egsdot;
    HTMLEntityTableEntry { first_value: 0x02A99, second_value: 0x0000, entity_offset: 566, length: 3 }, // &el;
    HTMLEntityTableEntry { first_value: 0x023E7, second_value: 0x0000, entity_offset: 7416, length: 9 }, // &elinters;
    HTMLEntityTableEntry { first_value: 0x02113, second_value: 0x0000, entity_offset: 7425, length: 4 }, // &ell;
    HTMLEntityTableEntry { first_value: 0x02A95, second_value: 0x0000, entity_offset: 7429, length: 4 }, // &els;
    HTMLEntityTableEntry { first_value: 0x02A97, second_value: 0x0000, entity_offset: 7433, length: 7 }, // &elsdot;
    HTMLEntityTableEntry { first_value: 0x00113, second_value: 0x0000, entity_offset: 7440, length: 6 }, // &emacr;
    HTMLEntityTableEntry { first_value: 0x02205, second_value: 0x0000, entity_offset: 7446, length: 6 }, // &empty;
    HTMLEntityTableEntry { first_value: 0x02205, second_value: 0x0000, entity_offset: 7452, length: 9 }, // &emptyset;
    HTMLEntityTableEntry { first_value: 0x02205, second_value: 0x0000, entity_offset: 5561, length: 7 }, // &emptyv;
    HTMLEntityTableEntry { first_value: 0x02004, second_value: 0x0000, entity_offset: 7461, length: 7 }, // &emsp13;
    HTMLEntityTableEntry { first_value: 0x02005, second_value: 0x0000, entity_offset: 7468, length: 7 }, // &emsp14;
    HTMLEntityTableEntry { first_value: 0x02003, second_value: 0x0000, entity_offset: 7475, length: 5 }, // &emsp;
    HTMLEntityTableEntry { first_value: 0x0014B, second_value: 0x0000, entity_offset: 7480, length: 4 }, // &eng;
    HTMLEntityTableEntry { first_value: 0x02002, second_value: 0x0000, entity_offset: 7484, length: 5 }, // &ensp;
    HTMLEntityTableEntry { first_value: 0x00119, second_value: 0x0000, entity_offset: 7489, length: 6 }, // &eogon;
    HTMLEntityTableEntry { first_value: 0x1D556, second_value: 0x0000, entity_offset: 7495, length: 5 }, // &eopf;
    HTMLEntityTableEntry { first_value: 0x022D5, second_value: 0x0000, entity_offset: 7500, length: 5 }, // &epar;
    HTMLEntityTableEntry { first_value: 0x029E3, second_value: 0x0000, entity_offset: 7505, length: 7 }, // &eparsl;
    HTMLEntityTableEntry { first_value: 0x02A71, second_value: 0x0000, entity_offset: 7512, length: 6 }, // &eplus;
    HTMLEntityTableEntry { first_value: 0x003B5, second_value: 0x0000, entity_offset: 5569, length: 5 }, // &epsi;
// cpp: html/parser/html_entity_table.cc:3068-3099
    HTMLEntityTableEntry { first_value: 0x003B5, second_value: 0x0000, entity_offset: 5456, length: 8 }, // &epsilon;
    HTMLEntityTableEntry { first_value: 0x003F5, second_value: 0x0000, entity_offset: 7518, length: 6 }, // &epsiv;
    HTMLEntityTableEntry { first_value: 0x02256, second_value: 0x0000, entity_offset: 7524, length: 7 }, // &eqcirc;
    HTMLEntityTableEntry { first_value: 0x02255, second_value: 0x0000, entity_offset: 7531, length: 8 }, // &eqcolon;
    HTMLEntityTableEntry { first_value: 0x02242, second_value: 0x0000, entity_offset: 7539, length: 6 }, // &eqsim;
    HTMLEntityTableEntry { first_value: 0x02A96, second_value: 0x0000, entity_offset: 7545, length: 11 }, // &eqslantgtr;
    HTMLEntityTableEntry { first_value: 0x02A95, second_value: 0x0000, entity_offset: 7556, length: 12 }, // &eqslantless;
    HTMLEntityTableEntry { first_value: 0x0003D, second_value: 0x0000, entity_offset: 7568, length: 7 }, // &equals;
    HTMLEntityTableEntry { first_value: 0x0225F, second_value: 0x0000, entity_offset: 7575, length: 7 }, // &equest;
    HTMLEntityTableEntry { first_value: 0x02261, second_value: 0x0000, entity_offset: 5861, length: 6 }, // &equiv;
    HTMLEntityTableEntry { first_value: 0x02A78, second_value: 0x0000, entity_offset: 7582, length: 8 }, // &equivDD;
    HTMLEntityTableEntry { first_value: 0x029E5, second_value: 0x0000, entity_offset: 7590, length: 9 }, // &eqvparsl;
    HTMLEntityTableEntry { first_value: 0x02253, second_value: 0x0000, entity_offset: 288, length: 6 }, // &erDot;
    HTMLEntityTableEntry { first_value: 0x02971, second_value: 0x0000, entity_offset: 7599, length: 6 }, // &erarr;
    HTMLEntityTableEntry { first_value: 0x0212F, second_value: 0x0000, entity_offset: 7605, length: 5 }, // &escr;
    HTMLEntityTableEntry { first_value: 0x02250, second_value: 0x0000, entity_offset: 7610, length: 6 }, // &esdot;
    HTMLEntityTableEntry { first_value: 0x02242, second_value: 0x0000, entity_offset: 7616, length: 5 }, // &esim;
    HTMLEntityTableEntry { first_value: 0x003B7, second_value: 0x0000, entity_offset: 162, length: 4 }, // &eta;
    HTMLEntityTableEntry { first_value: 0x000F0, second_value: 0x0000, entity_offset: 5587, length: 3 }, // &eth
    HTMLEntityTableEntry { first_value: 0x000F0, second_value: 0x0000, entity_offset: 5587, length: 4 }, // &eth;
    HTMLEntityTableEntry { first_value: 0x000EB, second_value: 0x0000, entity_offset: 7621, length: 4 }, // &euml
    HTMLEntityTableEntry { first_value: 0x000EB, second_value: 0x0000, entity_offset: 7621, length: 5 }, // &euml;
    HTMLEntityTableEntry { first_value: 0x020AC, second_value: 0x0000, entity_offset: 7626, length: 5 }, // &euro;
    HTMLEntityTableEntry { first_value: 0x00021, second_value: 0x0000, entity_offset: 7631, length: 5 }, // &excl;
    HTMLEntityTableEntry { first_value: 0x02203, second_value: 0x0000, entity_offset: 7636, length: 6 }, // &exist;
    HTMLEntityTableEntry { first_value: 0x02130, second_value: 0x0000, entity_offset: 7642, length: 12 }, // &expectation;
    HTMLEntityTableEntry { first_value: 0x02147, second_value: 0x0000, entity_offset: 7654, length: 13 }, // &exponentiale;
    HTMLEntityTableEntry { first_value: 0x02252, second_value: 0x0000, entity_offset: 7667, length: 14 }, // &fallingdotseq;
    HTMLEntityTableEntry { first_value: 0x00444, second_value: 0x0000, entity_offset: 7681, length: 4 }, // &fcy;
    HTMLEntityTableEntry { first_value: 0x02640, second_value: 0x0000, entity_offset: 7685, length: 7 }, // &female;
    HTMLEntityTableEntry { first_value: 0x0FB03, second_value: 0x0000, entity_offset: 7692, length: 7 }, // &ffilig;
    HTMLEntityTableEntry { first_value: 0x0FB00, second_value: 0x0000, entity_offset: 7699, length: 6 }, // &fflig;
// cpp: html/parser/html_entity_table.cc:3100-3131
    HTMLEntityTableEntry { first_value: 0x0FB04, second_value: 0x0000, entity_offset: 7705, length: 7 }, // &ffllig;
    HTMLEntityTableEntry { first_value: 0x1D523, second_value: 0x0000, entity_offset: 7712, length: 4 }, // &ffr;
    HTMLEntityTableEntry { first_value: 0x0FB01, second_value: 0x0000, entity_offset: 7693, length: 6 }, // &filig;
    HTMLEntityTableEntry { first_value: 0x00066, second_value: 0x006A, entity_offset: 7716, length: 6 }, // &fjlig;
    HTMLEntityTableEntry { first_value: 0x0266D, second_value: 0x0000, entity_offset: 7722, length: 5 }, // &flat;
    HTMLEntityTableEntry { first_value: 0x0FB02, second_value: 0x0000, entity_offset: 7706, length: 6 }, // &fllig;
    HTMLEntityTableEntry { first_value: 0x025B1, second_value: 0x0000, entity_offset: 7727, length: 6 }, // &fltns;
    HTMLEntityTableEntry { first_value: 0x00192, second_value: 0x0000, entity_offset: 7733, length: 5 }, // &fnof;
    HTMLEntityTableEntry { first_value: 0x1D557, second_value: 0x0000, entity_offset: 7738, length: 5 }, // &fopf;
    HTMLEntityTableEntry { first_value: 0x02200, second_value: 0x0000, entity_offset: 7743, length: 7 }, // &forall;
    HTMLEntityTableEntry { first_value: 0x022D4, second_value: 0x0000, entity_offset: 7750, length: 5 }, // &fork;
    HTMLEntityTableEntry { first_value: 0x02AD9, second_value: 0x0000, entity_offset: 7755, length: 6 }, // &forkv;
    HTMLEntityTableEntry { first_value: 0x02A0D, second_value: 0x0000, entity_offset: 7761, length: 9 }, // &fpartint;
    HTMLEntityTableEntry { first_value: 0x000BD, second_value: 0x0000, entity_offset: 7770, length: 6 }, // &frac12
    HTMLEntityTableEntry { first_value: 0x000BD, second_value: 0x0000, entity_offset: 7770, length: 7 }, // &frac12;
    HTMLEntityTableEntry { first_value: 0x02153, second_value: 0x0000, entity_offset: 7777, length: 7 }, // &frac13;
    HTMLEntityTableEntry { first_value: 0x000BC, second_value: 0x0000, entity_offset: 7784, length: 6 }, // &frac14
    HTMLEntityTableEntry { first_value: 0x000BC, second_value: 0x0000, entity_offset: 7784, length: 7 }, // &frac14;
    HTMLEntityTableEntry { first_value: 0x02155, second_value: 0x0000, entity_offset: 7791, length: 7 }, // &frac15;
    HTMLEntityTableEntry { first_value: 0x02159, second_value: 0x0000, entity_offset: 7798, length: 7 }, // &frac16;
    HTMLEntityTableEntry { first_value: 0x0215B, second_value: 0x0000, entity_offset: 7805, length: 7 }, // &frac18;
    HTMLEntityTableEntry { first_value: 0x02154, second_value: 0x0000, entity_offset: 7812, length: 7 }, // &frac23;
    HTMLEntityTableEntry { first_value: 0x02156, second_value: 0x0000, entity_offset: 7819, length: 7 }, // &frac25;
    HTMLEntityTableEntry { first_value: 0x000BE, second_value: 0x0000, entity_offset: 7826, length: 6 }, // &frac34
    HTMLEntityTableEntry { first_value: 0x000BE, second_value: 0x0000, entity_offset: 7826, length: 7 }, // &frac34;
    HTMLEntityTableEntry { first_value: 0x02157, second_value: 0x0000, entity_offset: 7833, length: 7 }, // &frac35;
    HTMLEntityTableEntry { first_value: 0x0215C, second_value: 0x0000, entity_offset: 7840, length: 7 }, // &frac38;
    HTMLEntityTableEntry { first_value: 0x02158, second_value: 0x0000, entity_offset: 7847, length: 7 }, // &frac45;
    HTMLEntityTableEntry { first_value: 0x0215A, second_value: 0x0000, entity_offset: 7854, length: 7 }, // &frac56;
    HTMLEntityTableEntry { first_value: 0x0215D, second_value: 0x0000, entity_offset: 7861, length: 7 }, // &frac58;
    HTMLEntityTableEntry { first_value: 0x0215E, second_value: 0x0000, entity_offset: 7868, length: 7 }, // &frac78;
    HTMLEntityTableEntry { first_value: 0x02044, second_value: 0x0000, entity_offset: 7875, length: 6 }, // &frasl;
// cpp: html/parser/html_entity_table.cc:3132-3163
    HTMLEntityTableEntry { first_value: 0x02322, second_value: 0x0000, entity_offset: 7881, length: 6 }, // &frown;
    HTMLEntityTableEntry { first_value: 0x1D4BB, second_value: 0x0000, entity_offset: 7887, length: 5 }, // &fscr;
    HTMLEntityTableEntry { first_value: 0x02267, second_value: 0x0000, entity_offset: 7892, length: 3 }, // &gE;
    HTMLEntityTableEntry { first_value: 0x02A8C, second_value: 0x0000, entity_offset: 7895, length: 4 }, // &gEl;
    HTMLEntityTableEntry { first_value: 0x001F5, second_value: 0x0000, entity_offset: 7899, length: 7 }, // &gacute;
    HTMLEntityTableEntry { first_value: 0x003B3, second_value: 0x0000, entity_offset: 7058, length: 6 }, // &gamma;
    HTMLEntityTableEntry { first_value: 0x003DD, second_value: 0x0000, entity_offset: 7906, length: 7 }, // &gammad;
    HTMLEntityTableEntry { first_value: 0x02A86, second_value: 0x0000, entity_offset: 7913, length: 4 }, // &gap;
    HTMLEntityTableEntry { first_value: 0x0011F, second_value: 0x0000, entity_offset: 7917, length: 7 }, // &gbreve;
    HTMLEntityTableEntry { first_value: 0x0011D, second_value: 0x0000, entity_offset: 5612, length: 6 }, // &gcirc;
    HTMLEntityTableEntry { first_value: 0x00433, second_value: 0x0000, entity_offset: 7924, length: 4 }, // &gcy;
    HTMLEntityTableEntry { first_value: 0x00121, second_value: 0x0000, entity_offset: 6620, length: 5 }, // &gdot;
    HTMLEntityTableEntry { first_value: 0x02265, second_value: 0x0000, entity_offset: 4941, length: 3 }, // &ge;
    HTMLEntityTableEntry { first_value: 0x022DB, second_value: 0x0000, entity_offset: 7928, length: 4 }, // &gel;
    HTMLEntityTableEntry { first_value: 0x02265, second_value: 0x0000, entity_offset: 7932, length: 4 }, // &geq;
    HTMLEntityTableEntry { first_value: 0x02267, second_value: 0x0000, entity_offset: 7936, length: 5 }, // &geqq;
    HTMLEntityTableEntry { first_value: 0x02A7E, second_value: 0x0000, entity_offset: 7941, length: 9 }, // &geqslant;
    HTMLEntityTableEntry { first_value: 0x02A7E, second_value: 0x0000, entity_offset: 7950, length: 4 }, // &ges;
    HTMLEntityTableEntry { first_value: 0x02AA9, second_value: 0x0000, entity_offset: 7954, length: 6 }, // &gescc;
    HTMLEntityTableEntry { first_value: 0x02A80, second_value: 0x0000, entity_offset: 7960, length: 7 }, // &gesdot;
    HTMLEntityTableEntry { first_value: 0x02A82, second_value: 0x0000, entity_offset: 7967, length: 8 }, // &gesdoto;
    HTMLEntityTableEntry { first_value: 0x02A84, second_value: 0x0000, entity_offset: 7975, length: 9 }, // &gesdotol;
    HTMLEntityTableEntry { first_value: 0x022DB, second_value: 0xFE00, entity_offset: 7984, length: 5 }, // &gesl;
    HTMLEntityTableEntry { first_value: 0x02A94, second_value: 0x0000, entity_offset: 7989, length: 7 }, // &gesles;
    HTMLEntityTableEntry { first_value: 0x1D524, second_value: 0x0000, entity_offset: 7996, length: 4 }, // &gfr;
    HTMLEntityTableEntry { first_value: 0x0226B, second_value: 0x0000, entity_offset: 8000, length: 3 }, // &gg;
    HTMLEntityTableEntry { first_value: 0x022D9, second_value: 0x0000, entity_offset: 8003, length: 4 }, // &ggg;
    HTMLEntityTableEntry { first_value: 0x02137, second_value: 0x0000, entity_offset: 8007, length: 6 }, // &gimel;
    HTMLEntityTableEntry { first_value: 0x00453, second_value: 0x0000, entity_offset: 8013, length: 5 }, // &gjcy;
    HTMLEntityTableEntry { first_value: 0x02277, second_value: 0x0000, entity_offset: 8018, length: 3 }, // &gl;
    HTMLEntityTableEntry { first_value: 0x02A92, second_value: 0x0000, entity_offset: 8021, length: 4 }, // &glE;
    HTMLEntityTableEntry { first_value: 0x02AA5, second_value: 0x0000, entity_offset: 8025, length: 4 }, // &gla;
// cpp: html/parser/html_entity_table.cc:3164-3195
    HTMLEntityTableEntry { first_value: 0x02AA4, second_value: 0x0000, entity_offset: 8029, length: 4 }, // &glj;
    HTMLEntityTableEntry { first_value: 0x02269, second_value: 0x0000, entity_offset: 8033, length: 4 }, // &gnE;
    HTMLEntityTableEntry { first_value: 0x02A8A, second_value: 0x0000, entity_offset: 8037, length: 5 }, // &gnap;
    HTMLEntityTableEntry { first_value: 0x02A8A, second_value: 0x0000, entity_offset: 8042, length: 9 }, // &gnapprox;
    HTMLEntityTableEntry { first_value: 0x02A88, second_value: 0x0000, entity_offset: 8051, length: 4 }, // &gne;
    HTMLEntityTableEntry { first_value: 0x02A88, second_value: 0x0000, entity_offset: 8055, length: 5 }, // &gneq;
    HTMLEntityTableEntry { first_value: 0x02269, second_value: 0x0000, entity_offset: 8060, length: 6 }, // &gneqq;
    HTMLEntityTableEntry { first_value: 0x022E7, second_value: 0x0000, entity_offset: 8066, length: 6 }, // &gnsim;
    HTMLEntityTableEntry { first_value: 0x1D558, second_value: 0x0000, entity_offset: 8072, length: 5 }, // &gopf;
    HTMLEntityTableEntry { first_value: 0x00060, second_value: 0x0000, entity_offset: 39, length: 6 }, // &grave;
    HTMLEntityTableEntry { first_value: 0x0210A, second_value: 0x0000, entity_offset: 8077, length: 5 }, // &gscr;
    HTMLEntityTableEntry { first_value: 0x02273, second_value: 0x0000, entity_offset: 8082, length: 5 }, // &gsim;
    HTMLEntityTableEntry { first_value: 0x02A8E, second_value: 0x0000, entity_offset: 8087, length: 6 }, // &gsime;
    HTMLEntityTableEntry { first_value: 0x02A90, second_value: 0x0000, entity_offset: 8093, length: 6 }, // &gsiml;
    HTMLEntityTableEntry { first_value: 0x0003E, second_value: 0x0000, entity_offset: 5671, length: 2 }, // &gt
    HTMLEntityTableEntry { first_value: 0x0003E, second_value: 0x0000, entity_offset: 8099, length: 3 }, // &gt;
    HTMLEntityTableEntry { first_value: 0x02AA7, second_value: 0x0000, entity_offset: 8102, length: 5 }, // &gtcc;
    HTMLEntityTableEntry { first_value: 0x02A7A, second_value: 0x0000, entity_offset: 8107, length: 6 }, // &gtcir;
    HTMLEntityTableEntry { first_value: 0x022D7, second_value: 0x0000, entity_offset: 8113, length: 6 }, // &gtdot;
    HTMLEntityTableEntry { first_value: 0x02995, second_value: 0x0000, entity_offset: 8119, length: 7 }, // &gtlPar;
    HTMLEntityTableEntry { first_value: 0x02A7C, second_value: 0x0000, entity_offset: 8126, length: 8 }, // &gtquest;
    HTMLEntityTableEntry { first_value: 0x02A86, second_value: 0x0000, entity_offset: 8134, length: 10 }, // &gtrapprox;
    HTMLEntityTableEntry { first_value: 0x02978, second_value: 0x0000, entity_offset: 8144, length: 7 }, // &gtrarr;
    HTMLEntityTableEntry { first_value: 0x022D7, second_value: 0x0000, entity_offset: 8151, length: 7 }, // &gtrdot;
    HTMLEntityTableEntry { first_value: 0x022DB, second_value: 0x0000, entity_offset: 8158, length: 10 }, // &gtreqless;
    HTMLEntityTableEntry { first_value: 0x02A8C, second_value: 0x0000, entity_offset: 8168, length: 11 }, // &gtreqqless;
    HTMLEntityTableEntry { first_value: 0x02277, second_value: 0x0000, entity_offset: 8179, length: 8 }, // &gtrless;
    HTMLEntityTableEntry { first_value: 0x02273, second_value: 0x0000, entity_offset: 8187, length: 7 }, // &gtrsim;
    HTMLEntityTableEntry { first_value: 0x02269, second_value: 0xFE00, entity_offset: 8194, length: 10 }, // &gvertneqq;
    HTMLEntityTableEntry { first_value: 0x02269, second_value: 0xFE00, entity_offset: 8204, length: 5 }, // &gvnE;
    HTMLEntityTableEntry { first_value: 0x021D4, second_value: 0x0000, entity_offset: 8209, length: 5 }, // &hArr;
    HTMLEntityTableEntry { first_value: 0x0200A, second_value: 0x0000, entity_offset: 8214, length: 7 }, // &hairsp;
// cpp: html/parser/html_entity_table.cc:3196-3227
    HTMLEntityTableEntry { first_value: 0x000BD, second_value: 0x0000, entity_offset: 8221, length: 5 }, // &half;
    HTMLEntityTableEntry { first_value: 0x0210B, second_value: 0x0000, entity_offset: 8226, length: 7 }, // &hamilt;
    HTMLEntityTableEntry { first_value: 0x0044A, second_value: 0x0000, entity_offset: 8233, length: 7 }, // &hardcy;
    HTMLEntityTableEntry { first_value: 0x02194, second_value: 0x0000, entity_offset: 7016, length: 5 }, // &harr;
    HTMLEntityTableEntry { first_value: 0x02948, second_value: 0x0000, entity_offset: 8240, length: 8 }, // &harrcir;
    HTMLEntityTableEntry { first_value: 0x021AD, second_value: 0x0000, entity_offset: 8248, length: 6 }, // &harrw;
    HTMLEntityTableEntry { first_value: 0x0210F, second_value: 0x0000, entity_offset: 8254, length: 5 }, // &hbar;
    HTMLEntityTableEntry { first_value: 0x00125, second_value: 0x0000, entity_offset: 8259, length: 6 }, // &hcirc;
    HTMLEntityTableEntry { first_value: 0x02665, second_value: 0x0000, entity_offset: 8265, length: 7 }, // &hearts;
    HTMLEntityTableEntry { first_value: 0x02665, second_value: 0x0000, entity_offset: 8272, length: 10 }, // &heartsuit;
    HTMLEntityTableEntry { first_value: 0x02026, second_value: 0x0000, entity_offset: 8282, length: 7 }, // &hellip;
    HTMLEntityTableEntry { first_value: 0x022B9, second_value: 0x0000, entity_offset: 8289, length: 7 }, // &hercon;
    HTMLEntityTableEntry { first_value: 0x1D525, second_value: 0x0000, entity_offset: 8296, length: 4 }, // &hfr;
    HTMLEntityTableEntry { first_value: 0x02925, second_value: 0x0000, entity_offset: 8300, length: 9 }, // &hksearow;
    HTMLEntityTableEntry { first_value: 0x02926, second_value: 0x0000, entity_offset: 8309, length: 9 }, // &hkswarow;
    HTMLEntityTableEntry { first_value: 0x021FF, second_value: 0x0000, entity_offset: 8318, length: 6 }, // &hoarr;
    HTMLEntityTableEntry { first_value: 0x0223B, second_value: 0x0000, entity_offset: 8324, length: 7 }, // &homtht;
    HTMLEntityTableEntry { first_value: 0x021A9, second_value: 0x0000, entity_offset: 8331, length: 14 }, // &hookleftarrow;
    HTMLEntityTableEntry { first_value: 0x021AA, second_value: 0x0000, entity_offset: 8345, length: 15 }, // &hookrightarrow;
    HTMLEntityTableEntry { first_value: 0x1D559, second_value: 0x0000, entity_offset: 8360, length: 5 }, // &hopf;
    HTMLEntityTableEntry { first_value: 0x02015, second_value: 0x0000, entity_offset: 8365, length: 7 }, // &horbar;
    HTMLEntityTableEntry { first_value: 0x1D4BD, second_value: 0x0000, entity_offset: 8372, length: 5 }, // &hscr;
    HTMLEntityTableEntry { first_value: 0x0210F, second_value: 0x0000, entity_offset: 8377, length: 7 }, // &hslash;
    HTMLEntityTableEntry { first_value: 0x00127, second_value: 0x0000, entity_offset: 8384, length: 7 }, // &hstrok;
    HTMLEntityTableEntry { first_value: 0x02043, second_value: 0x0000, entity_offset: 8391, length: 7 }, // &hybull;
    HTMLEntityTableEntry { first_value: 0x02010, second_value: 0x0000, entity_offset: 8398, length: 7 }, // &hyphen;
    HTMLEntityTableEntry { first_value: 0x000ED, second_value: 0x0000, entity_offset: 8405, length: 6 }, // &iacute
    HTMLEntityTableEntry { first_value: 0x000ED, second_value: 0x0000, entity_offset: 8405, length: 7 }, // &iacute;
    HTMLEntityTableEntry { first_value: 0x02063, second_value: 0x0000, entity_offset: 8412, length: 3 }, // &ic;
    HTMLEntityTableEntry { first_value: 0x000EE, second_value: 0x0000, entity_offset: 8415, length: 5 }, // &icirc
    HTMLEntityTableEntry { first_value: 0x000EE, second_value: 0x0000, entity_offset: 8415, length: 6 }, // &icirc;
    HTMLEntityTableEntry { first_value: 0x00438, second_value: 0x0000, entity_offset: 8421, length: 4 }, // &icy;
// cpp: html/parser/html_entity_table.cc:3228-3259
    HTMLEntityTableEntry { first_value: 0x00435, second_value: 0x0000, entity_offset: 8425, length: 5 }, // &iecy;
    HTMLEntityTableEntry { first_value: 0x000A1, second_value: 0x0000, entity_offset: 8430, length: 5 }, // &iexcl
    HTMLEntityTableEntry { first_value: 0x000A1, second_value: 0x0000, entity_offset: 8430, length: 6 }, // &iexcl;
    HTMLEntityTableEntry { first_value: 0x021D4, second_value: 0x0000, entity_offset: 8436, length: 4 }, // &iff;
    HTMLEntityTableEntry { first_value: 0x1D526, second_value: 0x0000, entity_offset: 8440, length: 4 }, // &ifr;
    HTMLEntityTableEntry { first_value: 0x000EC, second_value: 0x0000, entity_offset: 8444, length: 6 }, // &igrave
    HTMLEntityTableEntry { first_value: 0x000EC, second_value: 0x0000, entity_offset: 8444, length: 7 }, // &igrave;
    HTMLEntityTableEntry { first_value: 0x02148, second_value: 0x0000, entity_offset: 8451, length: 3 }, // &ii;
    HTMLEntityTableEntry { first_value: 0x02A0C, second_value: 0x0000, entity_offset: 8454, length: 7 }, // &iiiint;
    HTMLEntityTableEntry { first_value: 0x0222D, second_value: 0x0000, entity_offset: 8455, length: 6 }, // &iiint;
    HTMLEntityTableEntry { first_value: 0x029DC, second_value: 0x0000, entity_offset: 8461, length: 7 }, // &iinfin;
    HTMLEntityTableEntry { first_value: 0x02129, second_value: 0x0000, entity_offset: 8468, length: 6 }, // &iiota;
    HTMLEntityTableEntry { first_value: 0x00133, second_value: 0x0000, entity_offset: 8474, length: 6 }, // &ijlig;
    HTMLEntityTableEntry { first_value: 0x0012B, second_value: 0x0000, entity_offset: 8480, length: 6 }, // &imacr;
    HTMLEntityTableEntry { first_value: 0x02111, second_value: 0x0000, entity_offset: 8486, length: 6 }, // &image;
    HTMLEntityTableEntry { first_value: 0x02110, second_value: 0x0000, entity_offset: 8492, length: 9 }, // &imagline;
    HTMLEntityTableEntry { first_value: 0x02111, second_value: 0x0000, entity_offset: 8501, length: 9 }, // &imagpart;
    HTMLEntityTableEntry { first_value: 0x00131, second_value: 0x0000, entity_offset: 8510, length: 6 }, // &imath;
    HTMLEntityTableEntry { first_value: 0x022B7, second_value: 0x0000, entity_offset: 8516, length: 5 }, // &imof;
    HTMLEntityTableEntry { first_value: 0x001B5, second_value: 0x0000, entity_offset: 8521, length: 6 }, // &imped;
    HTMLEntityTableEntry { first_value: 0x02208, second_value: 0x0000, entity_offset: 7067, length: 3 }, // &in;
    HTMLEntityTableEntry { first_value: 0x02105, second_value: 0x0000, entity_offset: 8527, length: 7 }, // &incare;
    HTMLEntityTableEntry { first_value: 0x0221E, second_value: 0x0000, entity_offset: 8462, length: 6 }, // &infin;
    HTMLEntityTableEntry { first_value: 0x029DD, second_value: 0x0000, entity_offset: 8534, length: 9 }, // &infintie;
    HTMLEntityTableEntry { first_value: 0x00131, second_value: 0x0000, entity_offset: 8543, length: 7 }, // &inodot;
    HTMLEntityTableEntry { first_value: 0x0222B, second_value: 0x0000, entity_offset: 267, length: 4 }, // &int;
    HTMLEntityTableEntry { first_value: 0x022BA, second_value: 0x0000, entity_offset: 8550, length: 7 }, // &intcal;
    HTMLEntityTableEntry { first_value: 0x02124, second_value: 0x0000, entity_offset: 8557, length: 9 }, // &integers;
    HTMLEntityTableEntry { first_value: 0x022BA, second_value: 0x0000, entity_offset: 8566, length: 9 }, // &intercal;
    HTMLEntityTableEntry { first_value: 0x02A17, second_value: 0x0000, entity_offset: 8575, length: 9 }, // &intlarhk;
    HTMLEntityTableEntry { first_value: 0x02A3C, second_value: 0x0000, entity_offset: 8584, length: 8 }, // &intprod;
    HTMLEntityTableEntry { first_value: 0x00451, second_value: 0x0000, entity_offset: 8592, length: 5 }, // &iocy;
// cpp: html/parser/html_entity_table.cc:3260-3291
    HTMLEntityTableEntry { first_value: 0x0012F, second_value: 0x0000, entity_offset: 8597, length: 6 }, // &iogon;
    HTMLEntityTableEntry { first_value: 0x1D55A, second_value: 0x0000, entity_offset: 8603, length: 5 }, // &iopf;
    HTMLEntityTableEntry { first_value: 0x003B9, second_value: 0x0000, entity_offset: 8469, length: 5 }, // &iota;
    HTMLEntityTableEntry { first_value: 0x02A3C, second_value: 0x0000, entity_offset: 8608, length: 6 }, // &iprod;
    HTMLEntityTableEntry { first_value: 0x000BF, second_value: 0x0000, entity_offset: 8614, length: 6 }, // &iquest
    HTMLEntityTableEntry { first_value: 0x000BF, second_value: 0x0000, entity_offset: 8614, length: 7 }, // &iquest;
    HTMLEntityTableEntry { first_value: 0x1D4BE, second_value: 0x0000, entity_offset: 8621, length: 5 }, // &iscr;
    HTMLEntityTableEntry { first_value: 0x02208, second_value: 0x0000, entity_offset: 7065, length: 5 }, // &isin;
    HTMLEntityTableEntry { first_value: 0x022F9, second_value: 0x0000, entity_offset: 8626, length: 6 }, // &isinE;
    HTMLEntityTableEntry { first_value: 0x022F5, second_value: 0x0000, entity_offset: 8632, length: 8 }, // &isindot;
    HTMLEntityTableEntry { first_value: 0x022F4, second_value: 0x0000, entity_offset: 8640, length: 6 }, // &isins;
    HTMLEntityTableEntry { first_value: 0x022F3, second_value: 0x0000, entity_offset: 8646, length: 7 }, // &isinsv;
    HTMLEntityTableEntry { first_value: 0x02208, second_value: 0x0000, entity_offset: 8653, length: 6 }, // &isinv;
    HTMLEntityTableEntry { first_value: 0x02062, second_value: 0x0000, entity_offset: 6547, length: 3 }, // &it;
    HTMLEntityTableEntry { first_value: 0x00129, second_value: 0x0000, entity_offset: 8659, length: 7 }, // &itilde;
    HTMLEntityTableEntry { first_value: 0x00456, second_value: 0x0000, entity_offset: 8666, length: 6 }, // &iukcy;
    HTMLEntityTableEntry { first_value: 0x000EF, second_value: 0x0000, entity_offset: 8672, length: 4 }, // &iuml
    HTMLEntityTableEntry { first_value: 0x000EF, second_value: 0x0000, entity_offset: 8672, length: 5 }, // &iuml;
    HTMLEntityTableEntry { first_value: 0x00135, second_value: 0x0000, entity_offset: 8677, length: 6 }, // &jcirc;
    HTMLEntityTableEntry { first_value: 0x00439, second_value: 0x0000, entity_offset: 7103, length: 4 }, // &jcy;
    HTMLEntityTableEntry { first_value: 0x1D527, second_value: 0x0000, entity_offset: 8683, length: 4 }, // &jfr;
    HTMLEntityTableEntry { first_value: 0x00237, second_value: 0x0000, entity_offset: 8687, length: 6 }, // &jmath;
    HTMLEntityTableEntry { first_value: 0x1D55B, second_value: 0x0000, entity_offset: 8693, length: 5 }, // &jopf;
    HTMLEntityTableEntry { first_value: 0x1D4BF, second_value: 0x0000, entity_offset: 8698, length: 5 }, // &jscr;
    HTMLEntityTableEntry { first_value: 0x00458, second_value: 0x0000, entity_offset: 8703, length: 7 }, // &jsercy;
    HTMLEntityTableEntry { first_value: 0x00454, second_value: 0x0000, entity_offset: 8710, length: 6 }, // &jukcy;
    HTMLEntityTableEntry { first_value: 0x003BA, second_value: 0x0000, entity_offset: 8716, length: 6 }, // &kappa;
    HTMLEntityTableEntry { first_value: 0x003F0, second_value: 0x0000, entity_offset: 8722, length: 7 }, // &kappav;
    HTMLEntityTableEntry { first_value: 0x00137, second_value: 0x0000, entity_offset: 8729, length: 7 }, // &kcedil;
    HTMLEntityTableEntry { first_value: 0x0043A, second_value: 0x0000, entity_offset: 1833, length: 4 }, // &kcy;
    HTMLEntityTableEntry { first_value: 0x1D528, second_value: 0x0000, entity_offset: 8736, length: 4 }, // &kfr;
    HTMLEntityTableEntry { first_value: 0x00138, second_value: 0x0000, entity_offset: 8740, length: 7 }, // &kgreen;
// cpp: html/parser/html_entity_table.cc:3292-3323
    HTMLEntityTableEntry { first_value: 0x00445, second_value: 0x0000, entity_offset: 8747, length: 5 }, // &khcy;
    HTMLEntityTableEntry { first_value: 0x0045C, second_value: 0x0000, entity_offset: 8752, length: 5 }, // &kjcy;
    HTMLEntityTableEntry { first_value: 0x1D55C, second_value: 0x0000, entity_offset: 8757, length: 5 }, // &kopf;
    HTMLEntityTableEntry { first_value: 0x1D4C0, second_value: 0x0000, entity_offset: 8762, length: 5 }, // &kscr;
    HTMLEntityTableEntry { first_value: 0x021DA, second_value: 0x0000, entity_offset: 8767, length: 6 }, // &lAarr;
    HTMLEntityTableEntry { first_value: 0x021D0, second_value: 0x0000, entity_offset: 8773, length: 5 }, // &lArr;
    HTMLEntityTableEntry { first_value: 0x0291B, second_value: 0x0000, entity_offset: 8778, length: 7 }, // &lAtail;
    HTMLEntityTableEntry { first_value: 0x0290E, second_value: 0x0000, entity_offset: 8785, length: 6 }, // &lBarr;
    HTMLEntityTableEntry { first_value: 0x02266, second_value: 0x0000, entity_offset: 1330, length: 3 }, // &lE;
    HTMLEntityTableEntry { first_value: 0x02A8B, second_value: 0x0000, entity_offset: 8791, length: 4 }, // &lEg;
    HTMLEntityTableEntry { first_value: 0x02962, second_value: 0x0000, entity_offset: 8795, length: 5 }, // &lHar;
    HTMLEntityTableEntry { first_value: 0x0013A, second_value: 0x0000, entity_offset: 8800, length: 7 }, // &lacute;
    HTMLEntityTableEntry { first_value: 0x029B4, second_value: 0x0000, entity_offset: 8807, length: 9 }, // &laemptyv;
    HTMLEntityTableEntry { first_value: 0x02112, second_value: 0x0000, entity_offset: 8816, length: 7 }, // &lagran;
    HTMLEntityTableEntry { first_value: 0x003BB, second_value: 0x0000, entity_offset: 8823, length: 7 }, // &lambda;
    HTMLEntityTableEntry { first_value: 0x027E8, second_value: 0x0000, entity_offset: 8830, length: 5 }, // &lang;
    HTMLEntityTableEntry { first_value: 0x02991, second_value: 0x0000, entity_offset: 8835, length: 6 }, // &langd;
    HTMLEntityTableEntry { first_value: 0x027E8, second_value: 0x0000, entity_offset: 8841, length: 7 }, // &langle;
    HTMLEntityTableEntry { first_value: 0x02A85, second_value: 0x0000, entity_offset: 8848, length: 4 }, // &lap;
    HTMLEntityTableEntry { first_value: 0x000AB, second_value: 0x0000, entity_offset: 8852, length: 5 }, // &laquo
    HTMLEntityTableEntry { first_value: 0x000AB, second_value: 0x0000, entity_offset: 8852, length: 6 }, // &laquo;
    HTMLEntityTableEntry { first_value: 0x02190, second_value: 0x0000, entity_offset: 6724, length: 5 }, // &larr;
    HTMLEntityTableEntry { first_value: 0x021E4, second_value: 0x0000, entity_offset: 8858, length: 6 }, // &larrb;
    HTMLEntityTableEntry { first_value: 0x0291F, second_value: 0x0000, entity_offset: 8864, length: 8 }, // &larrbfs;
    HTMLEntityTableEntry { first_value: 0x0291D, second_value: 0x0000, entity_offset: 8872, length: 7 }, // &larrfs;
    HTMLEntityTableEntry { first_value: 0x021A9, second_value: 0x0000, entity_offset: 8879, length: 7 }, // &larrhk;
    HTMLEntityTableEntry { first_value: 0x021AB, second_value: 0x0000, entity_offset: 8886, length: 7 }, // &larrlp;
    HTMLEntityTableEntry { first_value: 0x02939, second_value: 0x0000, entity_offset: 8893, length: 7 }, // &larrpl;
    HTMLEntityTableEntry { first_value: 0x02973, second_value: 0x0000, entity_offset: 8900, length: 8 }, // &larrsim;
    HTMLEntityTableEntry { first_value: 0x021A2, second_value: 0x0000, entity_offset: 8908, length: 7 }, // &larrtl;
    HTMLEntityTableEntry { first_value: 0x02AAB, second_value: 0x0000, entity_offset: 7723, length: 4 }, // &lat;
    HTMLEntityTableEntry { first_value: 0x02919, second_value: 0x0000, entity_offset: 8915, length: 7 }, // &latail;
// cpp: html/parser/html_entity_table.cc:3324-3355
    HTMLEntityTableEntry { first_value: 0x02AAD, second_value: 0x0000, entity_offset: 8922, length: 5 }, // &late;
    HTMLEntityTableEntry { first_value: 0x02AAD, second_value: 0xFE00, entity_offset: 8927, length: 6 }, // &lates;
    HTMLEntityTableEntry { first_value: 0x0290C, second_value: 0x0000, entity_offset: 8933, length: 6 }, // &lbarr;
    HTMLEntityTableEntry { first_value: 0x02772, second_value: 0x0000, entity_offset: 8939, length: 6 }, // &lbbrk;
    HTMLEntityTableEntry { first_value: 0x0007B, second_value: 0x0000, entity_offset: 8945, length: 7 }, // &lbrace;
    HTMLEntityTableEntry { first_value: 0x0005B, second_value: 0x0000, entity_offset: 8952, length: 7 }, // &lbrack;
    HTMLEntityTableEntry { first_value: 0x0298B, second_value: 0x0000, entity_offset: 8959, length: 6 }, // &lbrke;
    HTMLEntityTableEntry { first_value: 0x0298F, second_value: 0x0000, entity_offset: 8965, length: 8 }, // &lbrksld;
    HTMLEntityTableEntry { first_value: 0x0298D, second_value: 0x0000, entity_offset: 8973, length: 8 }, // &lbrkslu;
    HTMLEntityTableEntry { first_value: 0x0013E, second_value: 0x0000, entity_offset: 8981, length: 7 }, // &lcaron;
    HTMLEntityTableEntry { first_value: 0x0013C, second_value: 0x0000, entity_offset: 8988, length: 7 }, // &lcedil;
    HTMLEntityTableEntry { first_value: 0x02308, second_value: 0x0000, entity_offset: 8995, length: 6 }, // &lceil;
    HTMLEntityTableEntry { first_value: 0x0007B, second_value: 0x0000, entity_offset: 9001, length: 5 }, // &lcub;
    HTMLEntityTableEntry { first_value: 0x0043B, second_value: 0x0000, entity_offset: 9006, length: 4 }, // &lcy;
    HTMLEntityTableEntry { first_value: 0x02936, second_value: 0x0000, entity_offset: 9010, length: 5 }, // &ldca;
    HTMLEntityTableEntry { first_value: 0x0201C, second_value: 0x0000, entity_offset: 9015, length: 6 }, // &ldquo;
    HTMLEntityTableEntry { first_value: 0x0201E, second_value: 0x0000, entity_offset: 9021, length: 7 }, // &ldquor;
    HTMLEntityTableEntry { first_value: 0x02967, second_value: 0x0000, entity_offset: 9028, length: 8 }, // &ldrdhar;
    HTMLEntityTableEntry { first_value: 0x0294B, second_value: 0x0000, entity_offset: 9036, length: 9 }, // &ldrushar;
    HTMLEntityTableEntry { first_value: 0x021B2, second_value: 0x0000, entity_offset: 9045, length: 5 }, // &ldsh;
    HTMLEntityTableEntry { first_value: 0x02264, second_value: 0x0000, entity_offset: 2141, length: 3 }, // &le;
    HTMLEntityTableEntry { first_value: 0x02190, second_value: 0x0000, entity_offset: 2350, length: 10 }, // &leftarrow;
    HTMLEntityTableEntry { first_value: 0x021A2, second_value: 0x0000, entity_offset: 9050, length: 14 }, // &leftarrowtail;
    HTMLEntityTableEntry { first_value: 0x021BD, second_value: 0x0000, entity_offset: 9064, length: 16 }, // &leftharpoondown;
    HTMLEntityTableEntry { first_value: 0x021BC, second_value: 0x0000, entity_offset: 9080, length: 14 }, // &leftharpoonup;
    HTMLEntityTableEntry { first_value: 0x021C7, second_value: 0x0000, entity_offset: 9094, length: 15 }, // &leftleftarrows;
    HTMLEntityTableEntry { first_value: 0x02194, second_value: 0x0000, entity_offset: 2385, length: 15 }, // &leftrightarrow;
    HTMLEntityTableEntry { first_value: 0x021C6, second_value: 0x0000, entity_offset: 9109, length: 16 }, // &leftrightarrows;
    HTMLEntityTableEntry { first_value: 0x021CB, second_value: 0x0000, entity_offset: 9125, length: 18 }, // &leftrightharpoons;
    HTMLEntityTableEntry { first_value: 0x021AD, second_value: 0x0000, entity_offset: 9143, length: 20 }, // &leftrightsquigarrow;
    HTMLEntityTableEntry { first_value: 0x022CB, second_value: 0x0000, entity_offset: 9163, length: 15 }, // &leftthreetimes;
    HTMLEntityTableEntry { first_value: 0x022DA, second_value: 0x0000, entity_offset: 9178, length: 4 }, // &leg;
// cpp: html/parser/html_entity_table.cc:3356-3387
    HTMLEntityTableEntry { first_value: 0x02264, second_value: 0x0000, entity_offset: 9182, length: 4 }, // &leq;
    HTMLEntityTableEntry { first_value: 0x02266, second_value: 0x0000, entity_offset: 9186, length: 5 }, // &leqq;
    HTMLEntityTableEntry { first_value: 0x02A7D, second_value: 0x0000, entity_offset: 9191, length: 9 }, // &leqslant;
    HTMLEntityTableEntry { first_value: 0x02A7D, second_value: 0x0000, entity_offset: 7992, length: 4 }, // &les;
    HTMLEntityTableEntry { first_value: 0x02AA8, second_value: 0x0000, entity_offset: 9200, length: 6 }, // &lescc;
    HTMLEntityTableEntry { first_value: 0x02A7F, second_value: 0x0000, entity_offset: 9206, length: 7 }, // &lesdot;
    HTMLEntityTableEntry { first_value: 0x02A81, second_value: 0x0000, entity_offset: 9213, length: 8 }, // &lesdoto;
    HTMLEntityTableEntry { first_value: 0x02A83, second_value: 0x0000, entity_offset: 9221, length: 9 }, // &lesdotor;
    HTMLEntityTableEntry { first_value: 0x022DA, second_value: 0xFE00, entity_offset: 9230, length: 5 }, // &lesg;
    HTMLEntityTableEntry { first_value: 0x02A93, second_value: 0x0000, entity_offset: 9235, length: 7 }, // &lesges;
    HTMLEntityTableEntry { first_value: 0x02A85, second_value: 0x0000, entity_offset: 9242, length: 11 }, // &lessapprox;
    HTMLEntityTableEntry { first_value: 0x022D6, second_value: 0x0000, entity_offset: 9253, length: 8 }, // &lessdot;
    HTMLEntityTableEntry { first_value: 0x022DA, second_value: 0x0000, entity_offset: 9261, length: 10 }, // &lesseqgtr;
    HTMLEntityTableEntry { first_value: 0x02A8B, second_value: 0x0000, entity_offset: 9271, length: 11 }, // &lesseqqgtr;
    HTMLEntityTableEntry { first_value: 0x02276, second_value: 0x0000, entity_offset: 9282, length: 8 }, // &lessgtr;
    HTMLEntityTableEntry { first_value: 0x02272, second_value: 0x0000, entity_offset: 9290, length: 8 }, // &lesssim;
    HTMLEntityTableEntry { first_value: 0x0297C, second_value: 0x0000, entity_offset: 9298, length: 7 }, // &lfisht;
    HTMLEntityTableEntry { first_value: 0x0230A, second_value: 0x0000, entity_offset: 9305, length: 7 }, // &lfloor;
    HTMLEntityTableEntry { first_value: 0x1D529, second_value: 0x0000, entity_offset: 9312, length: 4 }, // &lfr;
    HTMLEntityTableEntry { first_value: 0x02276, second_value: 0x0000, entity_offset: 5165, length: 3 }, // &lg;
    HTMLEntityTableEntry { first_value: 0x02A91, second_value: 0x0000, entity_offset: 9316, length: 4 }, // &lgE;
    HTMLEntityTableEntry { first_value: 0x021BD, second_value: 0x0000, entity_offset: 9320, length: 6 }, // &lhard;
    HTMLEntityTableEntry { first_value: 0x021BC, second_value: 0x0000, entity_offset: 9326, length: 6 }, // &lharu;
    HTMLEntityTableEntry { first_value: 0x0296A, second_value: 0x0000, entity_offset: 9332, length: 7 }, // &lharul;
    HTMLEntityTableEntry { first_value: 0x02584, second_value: 0x0000, entity_offset: 9339, length: 6 }, // &lhblk;
    HTMLEntityTableEntry { first_value: 0x00459, second_value: 0x0000, entity_offset: 9345, length: 5 }, // &ljcy;
    HTMLEntityTableEntry { first_value: 0x0226A, second_value: 0x0000, entity_offset: 1390, length: 3 }, // &ll;
    HTMLEntityTableEntry { first_value: 0x021C7, second_value: 0x0000, entity_offset: 9350, length: 6 }, // &llarr;
    HTMLEntityTableEntry { first_value: 0x0231E, second_value: 0x0000, entity_offset: 9356, length: 9 }, // &llcorner;
    HTMLEntityTableEntry { first_value: 0x0296B, second_value: 0x0000, entity_offset: 9365, length: 7 }, // &llhard;
    HTMLEntityTableEntry { first_value: 0x025FA, second_value: 0x0000, entity_offset: 9372, length: 6 }, // &lltri;
    HTMLEntityTableEntry { first_value: 0x00140, second_value: 0x0000, entity_offset: 9378, length: 7 }, // &lmidot;
// cpp: html/parser/html_entity_table.cc:3388-3419
    HTMLEntityTableEntry { first_value: 0x023B0, second_value: 0x0000, entity_offset: 9385, length: 7 }, // &lmoust;
    HTMLEntityTableEntry { first_value: 0x023B0, second_value: 0x0000, entity_offset: 9392, length: 11 }, // &lmoustache;
    HTMLEntityTableEntry { first_value: 0x02268, second_value: 0x0000, entity_offset: 9403, length: 4 }, // &lnE;
    HTMLEntityTableEntry { first_value: 0x02A89, second_value: 0x0000, entity_offset: 9407, length: 5 }, // &lnap;
    HTMLEntityTableEntry { first_value: 0x02A89, second_value: 0x0000, entity_offset: 9412, length: 9 }, // &lnapprox;
    HTMLEntityTableEntry { first_value: 0x02A87, second_value: 0x0000, entity_offset: 9421, length: 4 }, // &lne;
    HTMLEntityTableEntry { first_value: 0x02A87, second_value: 0x0000, entity_offset: 9425, length: 5 }, // &lneq;
    HTMLEntityTableEntry { first_value: 0x02268, second_value: 0x0000, entity_offset: 9430, length: 6 }, // &lneqq;
    HTMLEntityTableEntry { first_value: 0x022E6, second_value: 0x0000, entity_offset: 9436, length: 6 }, // &lnsim;
    HTMLEntityTableEntry { first_value: 0x027EC, second_value: 0x0000, entity_offset: 9442, length: 6 }, // &loang;
    HTMLEntityTableEntry { first_value: 0x021FD, second_value: 0x0000, entity_offset: 9448, length: 6 }, // &loarr;
    HTMLEntityTableEntry { first_value: 0x027E6, second_value: 0x0000, entity_offset: 9454, length: 6 }, // &lobrk;
    HTMLEntityTableEntry { first_value: 0x027F5, second_value: 0x0000, entity_offset: 9460, length: 14 }, // &longleftarrow;
    HTMLEntityTableEntry { first_value: 0x027F7, second_value: 0x0000, entity_offset: 9474, length: 19 }, // &longleftrightarrow;
    HTMLEntityTableEntry { first_value: 0x027FC, second_value: 0x0000, entity_offset: 9493, length: 11 }, // &longmapsto;
    HTMLEntityTableEntry { first_value: 0x027F6, second_value: 0x0000, entity_offset: 9504, length: 15 }, // &longrightarrow;
    HTMLEntityTableEntry { first_value: 0x021AB, second_value: 0x0000, entity_offset: 9519, length: 14 }, // &looparrowleft;
    HTMLEntityTableEntry { first_value: 0x021AC, second_value: 0x0000, entity_offset: 9533, length: 15 }, // &looparrowright;
    HTMLEntityTableEntry { first_value: 0x02985, second_value: 0x0000, entity_offset: 9548, length: 6 }, // &lopar;
    HTMLEntityTableEntry { first_value: 0x1D55D, second_value: 0x0000, entity_offset: 9554, length: 5 }, // &lopf;
    HTMLEntityTableEntry { first_value: 0x02A2D, second_value: 0x0000, entity_offset: 9559, length: 7 }, // &loplus;
    HTMLEntityTableEntry { first_value: 0x02A34, second_value: 0x0000, entity_offset: 9566, length: 8 }, // &lotimes;
    HTMLEntityTableEntry { first_value: 0x02217, second_value: 0x0000, entity_offset: 9574, length: 7 }, // &lowast;
    HTMLEntityTableEntry { first_value: 0x0005F, second_value: 0x0000, entity_offset: 9581, length: 7 }, // &lowbar;
    HTMLEntityTableEntry { first_value: 0x025CA, second_value: 0x0000, entity_offset: 9588, length: 4 }, // &loz;
    HTMLEntityTableEntry { first_value: 0x025CA, second_value: 0x0000, entity_offset: 5736, length: 8 }, // &lozenge;
    HTMLEntityTableEntry { first_value: 0x029EB, second_value: 0x0000, entity_offset: 9592, length: 5 }, // &lozf;
    HTMLEntityTableEntry { first_value: 0x00028, second_value: 0x0000, entity_offset: 9597, length: 5 }, // &lpar;
    HTMLEntityTableEntry { first_value: 0x02993, second_value: 0x0000, entity_offset: 9602, length: 7 }, // &lparlt;
    HTMLEntityTableEntry { first_value: 0x021C6, second_value: 0x0000, entity_offset: 9609, length: 6 }, // &lrarr;
    HTMLEntityTableEntry { first_value: 0x0231F, second_value: 0x0000, entity_offset: 9615, length: 9 }, // &lrcorner;
    HTMLEntityTableEntry { first_value: 0x021CB, second_value: 0x0000, entity_offset: 9624, length: 6 }, // &lrhar;
// cpp: html/parser/html_entity_table.cc:3420-3451
    HTMLEntityTableEntry { first_value: 0x0296D, second_value: 0x0000, entity_offset: 9630, length: 7 }, // &lrhard;
    HTMLEntityTableEntry { first_value: 0x0200E, second_value: 0x0000, entity_offset: 9637, length: 4 }, // &lrm;
    HTMLEntityTableEntry { first_value: 0x022BF, second_value: 0x0000, entity_offset: 9641, length: 6 }, // &lrtri;
    HTMLEntityTableEntry { first_value: 0x02039, second_value: 0x0000, entity_offset: 9647, length: 7 }, // &lsaquo;
    HTMLEntityTableEntry { first_value: 0x1D4C1, second_value: 0x0000, entity_offset: 9654, length: 5 }, // &lscr;
    HTMLEntityTableEntry { first_value: 0x021B0, second_value: 0x0000, entity_offset: 9659, length: 4 }, // &lsh;
    HTMLEntityTableEntry { first_value: 0x02272, second_value: 0x0000, entity_offset: 9663, length: 5 }, // &lsim;
    HTMLEntityTableEntry { first_value: 0x02A8D, second_value: 0x0000, entity_offset: 9668, length: 6 }, // &lsime;
    HTMLEntityTableEntry { first_value: 0x02A8F, second_value: 0x0000, entity_offset: 9674, length: 6 }, // &lsimg;
    HTMLEntityTableEntry { first_value: 0x0005B, second_value: 0x0000, entity_offset: 9680, length: 5 }, // &lsqb;
    HTMLEntityTableEntry { first_value: 0x02018, second_value: 0x0000, entity_offset: 9685, length: 6 }, // &lsquo;
    HTMLEntityTableEntry { first_value: 0x0201A, second_value: 0x0000, entity_offset: 9691, length: 7 }, // &lsquor;
    HTMLEntityTableEntry { first_value: 0x00142, second_value: 0x0000, entity_offset: 9698, length: 7 }, // &lstrok;
    HTMLEntityTableEntry { first_value: 0x0003C, second_value: 0x0000, entity_offset: 571, length: 2 }, // &lt
    HTMLEntityTableEntry { first_value: 0x0003C, second_value: 0x0000, entity_offset: 8230, length: 3 }, // &lt;
    HTMLEntityTableEntry { first_value: 0x02AA6, second_value: 0x0000, entity_offset: 9705, length: 5 }, // &ltcc;
    HTMLEntityTableEntry { first_value: 0x02A79, second_value: 0x0000, entity_offset: 9710, length: 6 }, // &ltcir;
    HTMLEntityTableEntry { first_value: 0x022D6, second_value: 0x0000, entity_offset: 9716, length: 6 }, // &ltdot;
    HTMLEntityTableEntry { first_value: 0x022CB, second_value: 0x0000, entity_offset: 9722, length: 7 }, // &lthree;
    HTMLEntityTableEntry { first_value: 0x022C9, second_value: 0x0000, entity_offset: 9729, length: 7 }, // &ltimes;
    HTMLEntityTableEntry { first_value: 0x02976, second_value: 0x0000, entity_offset: 9736, length: 7 }, // &ltlarr;
    HTMLEntityTableEntry { first_value: 0x02A7B, second_value: 0x0000, entity_offset: 9743, length: 8 }, // &ltquest;
    HTMLEntityTableEntry { first_value: 0x02996, second_value: 0x0000, entity_offset: 9751, length: 7 }, // &ltrPar;
    HTMLEntityTableEntry { first_value: 0x025C3, second_value: 0x0000, entity_offset: 9373, length: 5 }, // &ltri;
    HTMLEntityTableEntry { first_value: 0x022B4, second_value: 0x0000, entity_offset: 9758, length: 6 }, // &ltrie;
    HTMLEntityTableEntry { first_value: 0x025C2, second_value: 0x0000, entity_offset: 9764, length: 6 }, // &ltrif;
    HTMLEntityTableEntry { first_value: 0x0294A, second_value: 0x0000, entity_offset: 9770, length: 9 }, // &lurdshar;
    HTMLEntityTableEntry { first_value: 0x02966, second_value: 0x0000, entity_offset: 9779, length: 8 }, // &luruhar;
    HTMLEntityTableEntry { first_value: 0x02268, second_value: 0xFE00, entity_offset: 9787, length: 10 }, // &lvertneqq;
    HTMLEntityTableEntry { first_value: 0x02268, second_value: 0xFE00, entity_offset: 9797, length: 5 }, // &lvnE;
    HTMLEntityTableEntry { first_value: 0x0223A, second_value: 0x0000, entity_offset: 9802, length: 6 }, // &mDDot;
    HTMLEntityTableEntry { first_value: 0x000AF, second_value: 0x0000, entity_offset: 52, length: 4 }, // &macr
// cpp: html/parser/html_entity_table.cc:3452-3483
    HTMLEntityTableEntry { first_value: 0x000AF, second_value: 0x0000, entity_offset: 52, length: 5 }, // &macr;
    HTMLEntityTableEntry { first_value: 0x02642, second_value: 0x0000, entity_offset: 7687, length: 5 }, // &male;
    HTMLEntityTableEntry { first_value: 0x02720, second_value: 0x0000, entity_offset: 9808, length: 5 }, // &malt;
    HTMLEntityTableEntry { first_value: 0x02720, second_value: 0x0000, entity_offset: 9813, length: 8 }, // &maltese;
    HTMLEntityTableEntry { first_value: 0x021A6, second_value: 0x0000, entity_offset: 9821, length: 4 }, // &map;
    HTMLEntityTableEntry { first_value: 0x021A6, second_value: 0x0000, entity_offset: 9497, length: 7 }, // &mapsto;
    HTMLEntityTableEntry { first_value: 0x021A7, second_value: 0x0000, entity_offset: 9825, length: 11 }, // &mapstodown;
    HTMLEntityTableEntry { first_value: 0x021A4, second_value: 0x0000, entity_offset: 9836, length: 11 }, // &mapstoleft;
    HTMLEntityTableEntry { first_value: 0x021A5, second_value: 0x0000, entity_offset: 9847, length: 9 }, // &mapstoup;
    HTMLEntityTableEntry { first_value: 0x025AE, second_value: 0x0000, entity_offset: 9856, length: 7 }, // &marker;
    HTMLEntityTableEntry { first_value: 0x02A29, second_value: 0x0000, entity_offset: 9863, length: 7 }, // &mcomma;
    HTMLEntityTableEntry { first_value: 0x0043C, second_value: 0x0000, entity_offset: 9870, length: 4 }, // &mcy;
    HTMLEntityTableEntry { first_value: 0x02014, second_value: 0x0000, entity_offset: 9874, length: 6 }, // &mdash;
    HTMLEntityTableEntry { first_value: 0x02221, second_value: 0x0000, entity_offset: 9880, length: 14 }, // &measuredangle;
    HTMLEntityTableEntry { first_value: 0x1D52A, second_value: 0x0000, entity_offset: 9894, length: 4 }, // &mfr;
    HTMLEntityTableEntry { first_value: 0x02127, second_value: 0x0000, entity_offset: 9898, length: 4 }, // &mho;
    HTMLEntityTableEntry { first_value: 0x000B5, second_value: 0x0000, entity_offset: 3592, length: 5 }, // &micro
    HTMLEntityTableEntry { first_value: 0x000B5, second_value: 0x0000, entity_offset: 9902, length: 6 }, // &micro;
    HTMLEntityTableEntry { first_value: 0x02223, second_value: 0x0000, entity_offset: 6523, length: 4 }, // &mid;
    HTMLEntityTableEntry { first_value: 0x0002A, second_value: 0x0000, entity_offset: 9908, length: 7 }, // &midast;
    HTMLEntityTableEntry { first_value: 0x02AF0, second_value: 0x0000, entity_offset: 9915, length: 7 }, // &midcir;
    HTMLEntityTableEntry { first_value: 0x000B7, second_value: 0x0000, entity_offset: 9922, length: 6 }, // &middot
    HTMLEntityTableEntry { first_value: 0x000B7, second_value: 0x0000, entity_offset: 9922, length: 7 }, // &middot;
    HTMLEntityTableEntry { first_value: 0x02212, second_value: 0x0000, entity_offset: 6076, length: 6 }, // &minus;
    HTMLEntityTableEntry { first_value: 0x0229F, second_value: 0x0000, entity_offset: 9929, length: 7 }, // &minusb;
    HTMLEntityTableEntry { first_value: 0x02238, second_value: 0x0000, entity_offset: 9936, length: 7 }, // &minusd;
    HTMLEntityTableEntry { first_value: 0x02A2A, second_value: 0x0000, entity_offset: 9943, length: 8 }, // &minusdu;
    HTMLEntityTableEntry { first_value: 0x02ADB, second_value: 0x0000, entity_offset: 9951, length: 5 }, // &mlcp;
    HTMLEntityTableEntry { first_value: 0x02026, second_value: 0x0000, entity_offset: 9956, length: 5 }, // &mldr;
    HTMLEntityTableEntry { first_value: 0x02213, second_value: 0x0000, entity_offset: 9961, length: 7 }, // &mnplus;
    HTMLEntityTableEntry { first_value: 0x022A7, second_value: 0x0000, entity_offset: 9968, length: 7 }, // &models;
    HTMLEntityTableEntry { first_value: 0x1D55E, second_value: 0x0000, entity_offset: 9975, length: 5 }, // &mopf;
// cpp: html/parser/html_entity_table.cc:3484-3515
    HTMLEntityTableEntry { first_value: 0x02213, second_value: 0x0000, entity_offset: 1666, length: 3 }, // &mp;
    HTMLEntityTableEntry { first_value: 0x1D4C2, second_value: 0x0000, entity_offset: 9980, length: 5 }, // &mscr;
    HTMLEntityTableEntry { first_value: 0x0223E, second_value: 0x0000, entity_offset: 9985, length: 7 }, // &mstpos;
    HTMLEntityTableEntry { first_value: 0x003BC, second_value: 0x0000, entity_offset: 9992, length: 3 }, // &mu;
    HTMLEntityTableEntry { first_value: 0x022B8, second_value: 0x0000, entity_offset: 9995, length: 9 }, // &multimap;
    HTMLEntityTableEntry { first_value: 0x022B8, second_value: 0x0000, entity_offset: 10004, length: 6 }, // &mumap;
    HTMLEntityTableEntry { first_value: 0x022D9, second_value: 0x0338, entity_offset: 10010, length: 4 }, // &nGg;
    HTMLEntityTableEntry { first_value: 0x0226B, second_value: 0x20D2, entity_offset: 10014, length: 4 }, // &nGt;
    HTMLEntityTableEntry { first_value: 0x0226B, second_value: 0x0338, entity_offset: 10018, length: 5 }, // &nGtv;
    HTMLEntityTableEntry { first_value: 0x021CD, second_value: 0x0000, entity_offset: 10023, length: 11 }, // &nLeftarrow;
    HTMLEntityTableEntry { first_value: 0x021CE, second_value: 0x0000, entity_offset: 10034, length: 16 }, // &nLeftrightarrow;
    HTMLEntityTableEntry { first_value: 0x022D8, second_value: 0x0338, entity_offset: 10050, length: 4 }, // &nLl;
    HTMLEntityTableEntry { first_value: 0x0226A, second_value: 0x20D2, entity_offset: 10054, length: 4 }, // &nLt;
    HTMLEntityTableEntry { first_value: 0x0226A, second_value: 0x0338, entity_offset: 10058, length: 5 }, // &nLtv;
    HTMLEntityTableEntry { first_value: 0x021CF, second_value: 0x0000, entity_offset: 10063, length: 12 }, // &nRightarrow;
    HTMLEntityTableEntry { first_value: 0x022AF, second_value: 0x0000, entity_offset: 10075, length: 7 }, // &nVDash;
    HTMLEntityTableEntry { first_value: 0x022AE, second_value: 0x0000, entity_offset: 10082, length: 7 }, // &nVdash;
    HTMLEntityTableEntry { first_value: 0x02207, second_value: 0x0000, entity_offset: 10089, length: 6 }, // &nabla;
    HTMLEntityTableEntry { first_value: 0x00144, second_value: 0x0000, entity_offset: 10095, length: 7 }, // &nacute;
    HTMLEntityTableEntry { first_value: 0x02220, second_value: 0x20D2, entity_offset: 10102, length: 5 }, // &nang;
    HTMLEntityTableEntry { first_value: 0x02249, second_value: 0x0000, entity_offset: 8038, length: 4 }, // &nap;
    HTMLEntityTableEntry { first_value: 0x02A70, second_value: 0x0338, entity_offset: 10107, length: 5 }, // &napE;
    HTMLEntityTableEntry { first_value: 0x0224B, second_value: 0x0338, entity_offset: 10112, length: 6 }, // &napid;
    HTMLEntityTableEntry { first_value: 0x00149, second_value: 0x0000, entity_offset: 10118, length: 6 }, // &napos;
    HTMLEntityTableEntry { first_value: 0x02249, second_value: 0x0000, entity_offset: 8043, length: 8 }, // &napprox;
    HTMLEntityTableEntry { first_value: 0x0266E, second_value: 0x0000, entity_offset: 10124, length: 6 }, // &natur;
    HTMLEntityTableEntry { first_value: 0x0266E, second_value: 0x0000, entity_offset: 10130, length: 8 }, // &natural;
    HTMLEntityTableEntry { first_value: 0x02115, second_value: 0x0000, entity_offset: 10138, length: 9 }, // &naturals;
    HTMLEntityTableEntry { first_value: 0x000A0, second_value: 0x0000, entity_offset: 10147, length: 4 }, // &nbsp
    HTMLEntityTableEntry { first_value: 0x000A0, second_value: 0x0000, entity_offset: 10147, length: 5 }, // &nbsp;
    HTMLEntityTableEntry { first_value: 0x0224E, second_value: 0x0338, entity_offset: 10152, length: 6 }, // &nbump;
    HTMLEntityTableEntry { first_value: 0x0224F, second_value: 0x0338, entity_offset: 10158, length: 7 }, // &nbumpe;
// cpp: html/parser/html_entity_table.cc:3516-3547
    HTMLEntityTableEntry { first_value: 0x02A43, second_value: 0x0000, entity_offset: 10165, length: 5 }, // &ncap;
    HTMLEntityTableEntry { first_value: 0x00148, second_value: 0x0000, entity_offset: 10170, length: 7 }, // &ncaron;
    HTMLEntityTableEntry { first_value: 0x00146, second_value: 0x0000, entity_offset: 10177, length: 7 }, // &ncedil;
    HTMLEntityTableEntry { first_value: 0x02247, second_value: 0x0000, entity_offset: 10184, length: 6 }, // &ncong;
    HTMLEntityTableEntry { first_value: 0x02A6D, second_value: 0x0338, entity_offset: 10190, length: 9 }, // &ncongdot;
    HTMLEntityTableEntry { first_value: 0x02A42, second_value: 0x0000, entity_offset: 10199, length: 5 }, // &ncup;
    HTMLEntityTableEntry { first_value: 0x0043D, second_value: 0x0000, entity_offset: 10204, length: 4 }, // &ncy;
    HTMLEntityTableEntry { first_value: 0x02013, second_value: 0x0000, entity_offset: 10208, length: 6 }, // &ndash;
    HTMLEntityTableEntry { first_value: 0x02260, second_value: 0x0000, entity_offset: 420, length: 3 }, // &ne;
    HTMLEntityTableEntry { first_value: 0x021D7, second_value: 0x0000, entity_offset: 10214, length: 6 }, // &neArr;
    HTMLEntityTableEntry { first_value: 0x02924, second_value: 0x0000, entity_offset: 10220, length: 7 }, // &nearhk;
    HTMLEntityTableEntry { first_value: 0x02197, second_value: 0x0000, entity_offset: 10227, length: 6 }, // &nearr;
    HTMLEntityTableEntry { first_value: 0x02197, second_value: 0x0000, entity_offset: 10233, length: 8 }, // &nearrow;
    HTMLEntityTableEntry { first_value: 0x02250, second_value: 0x0338, entity_offset: 10241, length: 6 }, // &nedot;
    HTMLEntityTableEntry { first_value: 0x02262, second_value: 0x0000, entity_offset: 5860, length: 7 }, // &nequiv;
    HTMLEntityTableEntry { first_value: 0x02928, second_value: 0x0000, entity_offset: 10247, length: 7 }, // &nesear;
    HTMLEntityTableEntry { first_value: 0x02242, second_value: 0x0338, entity_offset: 10254, length: 6 }, // &nesim;
    HTMLEntityTableEntry { first_value: 0x02204, second_value: 0x0000, entity_offset: 10260, length: 7 }, // &nexist;
    HTMLEntityTableEntry { first_value: 0x02204, second_value: 0x0000, entity_offset: 10267, length: 8 }, // &nexists;
    HTMLEntityTableEntry { first_value: 0x1D52B, second_value: 0x0000, entity_offset: 10275, length: 4 }, // &nfr;
    HTMLEntityTableEntry { first_value: 0x02267, second_value: 0x0338, entity_offset: 10279, length: 4 }, // &ngE;
    HTMLEntityTableEntry { first_value: 0x02271, second_value: 0x0000, entity_offset: 5203, length: 4 }, // &nge;
    HTMLEntityTableEntry { first_value: 0x02271, second_value: 0x0000, entity_offset: 10283, length: 5 }, // &ngeq;
    HTMLEntityTableEntry { first_value: 0x02267, second_value: 0x0338, entity_offset: 10288, length: 6 }, // &ngeqq;
    HTMLEntityTableEntry { first_value: 0x02A7E, second_value: 0x0338, entity_offset: 10294, length: 10 }, // &ngeqslant;
    HTMLEntityTableEntry { first_value: 0x02A7E, second_value: 0x0338, entity_offset: 10304, length: 5 }, // &nges;
    HTMLEntityTableEntry { first_value: 0x02275, second_value: 0x0000, entity_offset: 10309, length: 6 }, // &ngsim;
    HTMLEntityTableEntry { first_value: 0x0226F, second_value: 0x0000, entity_offset: 10315, length: 4 }, // &ngt;
    HTMLEntityTableEntry { first_value: 0x0226F, second_value: 0x0000, entity_offset: 10319, length: 5 }, // &ngtr;
    HTMLEntityTableEntry { first_value: 0x021CE, second_value: 0x0000, entity_offset: 10324, length: 6 }, // &nhArr;
    HTMLEntityTableEntry { first_value: 0x021AE, second_value: 0x0000, entity_offset: 10330, length: 6 }, // &nharr;
    HTMLEntityTableEntry { first_value: 0x02AF2, second_value: 0x0000, entity_offset: 10336, length: 6 }, // &nhpar;
// cpp: html/parser/html_entity_table.cc:3548-3579
    HTMLEntityTableEntry { first_value: 0x0220B, second_value: 0x0000, entity_offset: 10342, length: 3 }, // &ni;
    HTMLEntityTableEntry { first_value: 0x022FC, second_value: 0x0000, entity_offset: 10345, length: 4 }, // &nis;
    HTMLEntityTableEntry { first_value: 0x022FA, second_value: 0x0000, entity_offset: 10349, length: 5 }, // &nisd;
    HTMLEntityTableEntry { first_value: 0x0220B, second_value: 0x0000, entity_offset: 10354, length: 4 }, // &niv;
    HTMLEntityTableEntry { first_value: 0x0045A, second_value: 0x0000, entity_offset: 10358, length: 5 }, // &njcy;
    HTMLEntityTableEntry { first_value: 0x021CD, second_value: 0x0000, entity_offset: 10363, length: 6 }, // &nlArr;
    HTMLEntityTableEntry { first_value: 0x02266, second_value: 0x0338, entity_offset: 10369, length: 4 }, // &nlE;
    HTMLEntityTableEntry { first_value: 0x0219A, second_value: 0x0000, entity_offset: 10373, length: 6 }, // &nlarr;
    HTMLEntityTableEntry { first_value: 0x02025, second_value: 0x0000, entity_offset: 10379, length: 5 }, // &nldr;
    HTMLEntityTableEntry { first_value: 0x02270, second_value: 0x0000, entity_offset: 10384, length: 4 }, // &nle;
    HTMLEntityTableEntry { first_value: 0x0219A, second_value: 0x0000, entity_offset: 10388, length: 11 }, // &nleftarrow;
    HTMLEntityTableEntry { first_value: 0x021AE, second_value: 0x0000, entity_offset: 10399, length: 16 }, // &nleftrightarrow;
    HTMLEntityTableEntry { first_value: 0x02270, second_value: 0x0000, entity_offset: 10415, length: 5 }, // &nleq;
    HTMLEntityTableEntry { first_value: 0x02266, second_value: 0x0338, entity_offset: 10420, length: 6 }, // &nleqq;
    HTMLEntityTableEntry { first_value: 0x02A7D, second_value: 0x0338, entity_offset: 10426, length: 10 }, // &nleqslant;
    HTMLEntityTableEntry { first_value: 0x02A7D, second_value: 0x0338, entity_offset: 10436, length: 5 }, // &nles;
    HTMLEntityTableEntry { first_value: 0x0226E, second_value: 0x0000, entity_offset: 10441, length: 6 }, // &nless;
    HTMLEntityTableEntry { first_value: 0x02274, second_value: 0x0000, entity_offset: 10447, length: 6 }, // &nlsim;
    HTMLEntityTableEntry { first_value: 0x0226E, second_value: 0x0000, entity_offset: 10453, length: 4 }, // &nlt;
    HTMLEntityTableEntry { first_value: 0x022EA, second_value: 0x0000, entity_offset: 10457, length: 6 }, // &nltri;
    HTMLEntityTableEntry { first_value: 0x022EC, second_value: 0x0000, entity_offset: 10463, length: 7 }, // &nltrie;
    HTMLEntityTableEntry { first_value: 0x02224, second_value: 0x0000, entity_offset: 10470, length: 5 }, // &nmid;
    HTMLEntityTableEntry { first_value: 0x1D55F, second_value: 0x0000, entity_offset: 10475, length: 5 }, // &nopf;
    HTMLEntityTableEntry { first_value: 0x000AC, second_value: 0x0000, entity_offset: 5868, length: 3 }, // &not
    HTMLEntityTableEntry { first_value: 0x000AC, second_value: 0x0000, entity_offset: 5868, length: 4 }, // &not;
    HTMLEntityTableEntry { first_value: 0x02209, second_value: 0x0000, entity_offset: 10480, length: 6 }, // &notin;
    HTMLEntityTableEntry { first_value: 0x022F9, second_value: 0x0338, entity_offset: 10486, length: 7 }, // &notinE;
    HTMLEntityTableEntry { first_value: 0x022F5, second_value: 0x0338, entity_offset: 10493, length: 9 }, // &notindot;
    HTMLEntityTableEntry { first_value: 0x02209, second_value: 0x0000, entity_offset: 10502, length: 8 }, // &notinva;
    HTMLEntityTableEntry { first_value: 0x022F7, second_value: 0x0000, entity_offset: 10510, length: 8 }, // &notinvb;
    HTMLEntityTableEntry { first_value: 0x022F6, second_value: 0x0000, entity_offset: 10518, length: 8 }, // &notinvc;
    HTMLEntityTableEntry { first_value: 0x0220C, second_value: 0x0000, entity_offset: 10526, length: 6 }, // &notni;
// cpp: html/parser/html_entity_table.cc:3580-3611
    HTMLEntityTableEntry { first_value: 0x0220C, second_value: 0x0000, entity_offset: 10532, length: 8 }, // &notniva;
    HTMLEntityTableEntry { first_value: 0x022FE, second_value: 0x0000, entity_offset: 10540, length: 8 }, // &notnivb;
    HTMLEntityTableEntry { first_value: 0x022FD, second_value: 0x0000, entity_offset: 10548, length: 8 }, // &notnivc;
    HTMLEntityTableEntry { first_value: 0x02226, second_value: 0x0000, entity_offset: 10556, length: 5 }, // &npar;
    HTMLEntityTableEntry { first_value: 0x02226, second_value: 0x0000, entity_offset: 10561, length: 10 }, // &nparallel;
    HTMLEntityTableEntry { first_value: 0x02AFD, second_value: 0x20E5, entity_offset: 10571, length: 7 }, // &nparsl;
    HTMLEntityTableEntry { first_value: 0x02202, second_value: 0x0338, entity_offset: 10578, length: 6 }, // &npart;
    HTMLEntityTableEntry { first_value: 0x02A14, second_value: 0x0000, entity_offset: 10584, length: 8 }, // &npolint;
    HTMLEntityTableEntry { first_value: 0x02280, second_value: 0x0000, entity_offset: 10592, length: 4 }, // &npr;
    HTMLEntityTableEntry { first_value: 0x022E0, second_value: 0x0000, entity_offset: 10596, length: 7 }, // &nprcue;
    HTMLEntityTableEntry { first_value: 0x02AAF, second_value: 0x0338, entity_offset: 10603, length: 5 }, // &npre;
    HTMLEntityTableEntry { first_value: 0x02280, second_value: 0x0000, entity_offset: 10608, length: 6 }, // &nprec;
    HTMLEntityTableEntry { first_value: 0x02AAF, second_value: 0x0338, entity_offset: 10614, length: 8 }, // &npreceq;
    HTMLEntityTableEntry { first_value: 0x021CF, second_value: 0x0000, entity_offset: 10622, length: 6 }, // &nrArr;
    HTMLEntityTableEntry { first_value: 0x0219B, second_value: 0x0000, entity_offset: 10628, length: 6 }, // &nrarr;
    HTMLEntityTableEntry { first_value: 0x02933, second_value: 0x0338, entity_offset: 10634, length: 7 }, // &nrarrc;
    HTMLEntityTableEntry { first_value: 0x0219D, second_value: 0x0338, entity_offset: 10641, length: 7 }, // &nrarrw;
    HTMLEntityTableEntry { first_value: 0x0219B, second_value: 0x0000, entity_offset: 10648, length: 12 }, // &nrightarrow;
    HTMLEntityTableEntry { first_value: 0x022EB, second_value: 0x0000, entity_offset: 10660, length: 6 }, // &nrtri;
    HTMLEntityTableEntry { first_value: 0x022ED, second_value: 0x0000, entity_offset: 10666, length: 7 }, // &nrtrie;
    HTMLEntityTableEntry { first_value: 0x02281, second_value: 0x0000, entity_offset: 10673, length: 4 }, // &nsc;
    HTMLEntityTableEntry { first_value: 0x022E1, second_value: 0x0000, entity_offset: 10677, length: 7 }, // &nsccue;
    HTMLEntityTableEntry { first_value: 0x02AB0, second_value: 0x0338, entity_offset: 10684, length: 5 }, // &nsce;
    HTMLEntityTableEntry { first_value: 0x1D4C3, second_value: 0x0000, entity_offset: 10689, length: 5 }, // &nscr;
    HTMLEntityTableEntry { first_value: 0x02224, second_value: 0x0000, entity_offset: 10694, length: 10 }, // &nshortmid;
    HTMLEntityTableEntry { first_value: 0x02226, second_value: 0x0000, entity_offset: 10704, length: 15 }, // &nshortparallel;
    HTMLEntityTableEntry { first_value: 0x02241, second_value: 0x0000, entity_offset: 8067, length: 5 }, // &nsim;
    HTMLEntityTableEntry { first_value: 0x02244, second_value: 0x0000, entity_offset: 10719, length: 6 }, // &nsime;
    HTMLEntityTableEntry { first_value: 0x02244, second_value: 0x0000, entity_offset: 10725, length: 7 }, // &nsimeq;
    HTMLEntityTableEntry { first_value: 0x02224, second_value: 0x0000, entity_offset: 10732, length: 6 }, // &nsmid;
    HTMLEntityTableEntry { first_value: 0x02226, second_value: 0x0000, entity_offset: 10738, length: 6 }, // &nspar;
    HTMLEntityTableEntry { first_value: 0x022E2, second_value: 0x0000, entity_offset: 10744, length: 8 }, // &nsqsube;
// cpp: html/parser/html_entity_table.cc:3612-3643
    HTMLEntityTableEntry { first_value: 0x022E3, second_value: 0x0000, entity_offset: 10752, length: 8 }, // &nsqsupe;
    HTMLEntityTableEntry { first_value: 0x02284, second_value: 0x0000, entity_offset: 10760, length: 5 }, // &nsub;
    HTMLEntityTableEntry { first_value: 0x02AC5, second_value: 0x0338, entity_offset: 10765, length: 6 }, // &nsubE;
    HTMLEntityTableEntry { first_value: 0x02288, second_value: 0x0000, entity_offset: 10771, length: 6 }, // &nsube;
    HTMLEntityTableEntry { first_value: 0x02282, second_value: 0x20D2, entity_offset: 10777, length: 8 }, // &nsubset;
    HTMLEntityTableEntry { first_value: 0x02288, second_value: 0x0000, entity_offset: 10785, length: 10 }, // &nsubseteq;
    HTMLEntityTableEntry { first_value: 0x02AC5, second_value: 0x0338, entity_offset: 10795, length: 11 }, // &nsubseteqq;
    HTMLEntityTableEntry { first_value: 0x02281, second_value: 0x0000, entity_offset: 10806, length: 6 }, // &nsucc;
    HTMLEntityTableEntry { first_value: 0x02AB0, second_value: 0x0338, entity_offset: 10812, length: 8 }, // &nsucceq;
    HTMLEntityTableEntry { first_value: 0x02285, second_value: 0x0000, entity_offset: 10820, length: 5 }, // &nsup;
    HTMLEntityTableEntry { first_value: 0x02AC6, second_value: 0x0338, entity_offset: 10825, length: 6 }, // &nsupE;
    HTMLEntityTableEntry { first_value: 0x02289, second_value: 0x0000, entity_offset: 10831, length: 6 }, // &nsupe;
    HTMLEntityTableEntry { first_value: 0x02283, second_value: 0x20D2, entity_offset: 10837, length: 8 }, // &nsupset;
    HTMLEntityTableEntry { first_value: 0x02289, second_value: 0x0000, entity_offset: 10845, length: 10 }, // &nsupseteq;
    HTMLEntityTableEntry { first_value: 0x02AC6, second_value: 0x0338, entity_offset: 10855, length: 11 }, // &nsupseteqq;
    HTMLEntityTableEntry { first_value: 0x02279, second_value: 0x0000, entity_offset: 10866, length: 5 }, // &ntgl;
    HTMLEntityTableEntry { first_value: 0x000F1, second_value: 0x0000, entity_offset: 10871, length: 6 }, // &ntilde
    HTMLEntityTableEntry { first_value: 0x000F1, second_value: 0x0000, entity_offset: 10871, length: 7 }, // &ntilde;
    HTMLEntityTableEntry { first_value: 0x02278, second_value: 0x0000, entity_offset: 10878, length: 5 }, // &ntlg;
    HTMLEntityTableEntry { first_value: 0x022EA, second_value: 0x0000, entity_offset: 10883, length: 14 }, // &ntriangleleft;
    HTMLEntityTableEntry { first_value: 0x022EC, second_value: 0x0000, entity_offset: 10897, length: 16 }, // &ntrianglelefteq;
    HTMLEntityTableEntry { first_value: 0x022EB, second_value: 0x0000, entity_offset: 10913, length: 15 }, // &ntriangleright;
    HTMLEntityTableEntry { first_value: 0x022ED, second_value: 0x0000, entity_offset: 10928, length: 17 }, // &ntrianglerighteq;
    HTMLEntityTableEntry { first_value: 0x003BD, second_value: 0x0000, entity_offset: 10945, length: 3 }, // &nu;
    HTMLEntityTableEntry { first_value: 0x00023, second_value: 0x0000, entity_offset: 10948, length: 4 }, // &num;
    HTMLEntityTableEntry { first_value: 0x02116, second_value: 0x0000, entity_offset: 10952, length: 7 }, // &numero;
    HTMLEntityTableEntry { first_value: 0x02007, second_value: 0x0000, entity_offset: 10959, length: 6 }, // &numsp;
    HTMLEntityTableEntry { first_value: 0x022AD, second_value: 0x0000, entity_offset: 10965, length: 7 }, // &nvDash;
    HTMLEntityTableEntry { first_value: 0x02904, second_value: 0x0000, entity_offset: 10972, length: 7 }, // &nvHarr;
    HTMLEntityTableEntry { first_value: 0x0224D, second_value: 0x20D2, entity_offset: 10979, length: 5 }, // &nvap;
    HTMLEntityTableEntry { first_value: 0x022AC, second_value: 0x0000, entity_offset: 10984, length: 7 }, // &nvdash;
    HTMLEntityTableEntry { first_value: 0x02265, second_value: 0x20D2, entity_offset: 10991, length: 5 }, // &nvge;
// cpp: html/parser/html_entity_table.cc:3644-3675
    HTMLEntityTableEntry { first_value: 0x0003E, second_value: 0x20D2, entity_offset: 10996, length: 5 }, // &nvgt;
    HTMLEntityTableEntry { first_value: 0x029DE, second_value: 0x0000, entity_offset: 11001, length: 8 }, // &nvinfin;
    HTMLEntityTableEntry { first_value: 0x02902, second_value: 0x0000, entity_offset: 11009, length: 7 }, // &nvlArr;
    HTMLEntityTableEntry { first_value: 0x02264, second_value: 0x20D2, entity_offset: 11016, length: 5 }, // &nvle;
    HTMLEntityTableEntry { first_value: 0x0003C, second_value: 0x20D2, entity_offset: 11021, length: 5 }, // &nvlt;
    HTMLEntityTableEntry { first_value: 0x022B4, second_value: 0x20D2, entity_offset: 11026, length: 8 }, // &nvltrie;
    HTMLEntityTableEntry { first_value: 0x02903, second_value: 0x0000, entity_offset: 11034, length: 7 }, // &nvrArr;
    HTMLEntityTableEntry { first_value: 0x022B5, second_value: 0x20D2, entity_offset: 11041, length: 8 }, // &nvrtrie;
    HTMLEntityTableEntry { first_value: 0x0223C, second_value: 0x20D2, entity_offset: 11049, length: 6 }, // &nvsim;
    HTMLEntityTableEntry { first_value: 0x021D6, second_value: 0x0000, entity_offset: 11055, length: 6 }, // &nwArr;
    HTMLEntityTableEntry { first_value: 0x02923, second_value: 0x0000, entity_offset: 11061, length: 7 }, // &nwarhk;
    HTMLEntityTableEntry { first_value: 0x02196, second_value: 0x0000, entity_offset: 11068, length: 6 }, // &nwarr;
    HTMLEntityTableEntry { first_value: 0x02196, second_value: 0x0000, entity_offset: 11074, length: 8 }, // &nwarrow;
    HTMLEntityTableEntry { first_value: 0x02927, second_value: 0x0000, entity_offset: 11082, length: 7 }, // &nwnear;
    HTMLEntityTableEntry { first_value: 0x024C8, second_value: 0x0000, entity_offset: 11089, length: 3 }, // &oS;
    HTMLEntityTableEntry { first_value: 0x000F3, second_value: 0x0000, entity_offset: 11092, length: 6 }, // &oacute
    HTMLEntityTableEntry { first_value: 0x000F3, second_value: 0x0000, entity_offset: 11092, length: 7 }, // &oacute;
    HTMLEntityTableEntry { first_value: 0x0229B, second_value: 0x0000, entity_offset: 11099, length: 5 }, // &oast;
    HTMLEntityTableEntry { first_value: 0x0229A, second_value: 0x0000, entity_offset: 4567, length: 5 }, // &ocir;
    HTMLEntityTableEntry { first_value: 0x000F4, second_value: 0x0000, entity_offset: 11104, length: 5 }, // &ocirc
    HTMLEntityTableEntry { first_value: 0x000F4, second_value: 0x0000, entity_offset: 11104, length: 6 }, // &ocirc;
    HTMLEntityTableEntry { first_value: 0x0043E, second_value: 0x0000, entity_offset: 8593, length: 4 }, // &ocy;
    HTMLEntityTableEntry { first_value: 0x0229D, second_value: 0x0000, entity_offset: 11110, length: 6 }, // &odash;
    HTMLEntityTableEntry { first_value: 0x00151, second_value: 0x0000, entity_offset: 11116, length: 7 }, // &odblac;
    HTMLEntityTableEntry { first_value: 0x02A38, second_value: 0x0000, entity_offset: 11123, length: 5 }, // &odiv;
    HTMLEntityTableEntry { first_value: 0x02299, second_value: 0x0000, entity_offset: 5628, length: 5 }, // &odot;
    HTMLEntityTableEntry { first_value: 0x029BC, second_value: 0x0000, entity_offset: 11128, length: 7 }, // &odsold;
    HTMLEntityTableEntry { first_value: 0x00153, second_value: 0x0000, entity_offset: 11135, length: 6 }, // &oelig;
    HTMLEntityTableEntry { first_value: 0x029BF, second_value: 0x0000, entity_offset: 11141, length: 6 }, // &ofcir;
    HTMLEntityTableEntry { first_value: 0x1D52C, second_value: 0x0000, entity_offset: 11147, length: 4 }, // &ofr;
    HTMLEntityTableEntry { first_value: 0x002DB, second_value: 0x0000, entity_offset: 62, length: 5 }, // &ogon;
    HTMLEntityTableEntry { first_value: 0x000F2, second_value: 0x0000, entity_offset: 11151, length: 6 }, // &ograve
// cpp: html/parser/html_entity_table.cc:3676-3707
    HTMLEntityTableEntry { first_value: 0x000F2, second_value: 0x0000, entity_offset: 11151, length: 7 }, // &ograve;
    HTMLEntityTableEntry { first_value: 0x029C1, second_value: 0x0000, entity_offset: 11158, length: 4 }, // &ogt;
    HTMLEntityTableEntry { first_value: 0x029B5, second_value: 0x0000, entity_offset: 11162, length: 6 }, // &ohbar;
    HTMLEntityTableEntry { first_value: 0x003A9, second_value: 0x0000, entity_offset: 11168, length: 4 }, // &ohm;
    HTMLEntityTableEntry { first_value: 0x0222E, second_value: 0x0000, entity_offset: 11172, length: 5 }, // &oint;
    HTMLEntityTableEntry { first_value: 0x021BA, second_value: 0x0000, entity_offset: 11177, length: 6 }, // &olarr;
    HTMLEntityTableEntry { first_value: 0x029BE, second_value: 0x0000, entity_offset: 11183, length: 6 }, // &olcir;
    HTMLEntityTableEntry { first_value: 0x029BB, second_value: 0x0000, entity_offset: 11189, length: 8 }, // &olcross;
    HTMLEntityTableEntry { first_value: 0x0203E, second_value: 0x0000, entity_offset: 11197, length: 6 }, // &oline;
    HTMLEntityTableEntry { first_value: 0x029C0, second_value: 0x0000, entity_offset: 11203, length: 4 }, // &olt;
    HTMLEntityTableEntry { first_value: 0x0014D, second_value: 0x0000, entity_offset: 11207, length: 6 }, // &omacr;
    HTMLEntityTableEntry { first_value: 0x003C9, second_value: 0x0000, entity_offset: 11213, length: 6 }, // &omega;
    HTMLEntityTableEntry { first_value: 0x003BF, second_value: 0x0000, entity_offset: 11219, length: 8 }, // &omicron;
    HTMLEntityTableEntry { first_value: 0x029B6, second_value: 0x0000, entity_offset: 11227, length: 5 }, // &omid;
    HTMLEntityTableEntry { first_value: 0x02296, second_value: 0x0000, entity_offset: 11232, length: 7 }, // &ominus;
    HTMLEntityTableEntry { first_value: 0x1D560, second_value: 0x0000, entity_offset: 11239, length: 5 }, // &oopf;
    HTMLEntityTableEntry { first_value: 0x029B7, second_value: 0x0000, entity_offset: 9549, length: 5 }, // &opar;
    HTMLEntityTableEntry { first_value: 0x029B9, second_value: 0x0000, entity_offset: 11244, length: 6 }, // &operp;
    HTMLEntityTableEntry { first_value: 0x02295, second_value: 0x0000, entity_offset: 5636, length: 6 }, // &oplus;
    HTMLEntityTableEntry { first_value: 0x02228, second_value: 0x0000, entity_offset: 1001, length: 3 }, // &or;
    HTMLEntityTableEntry { first_value: 0x021BB, second_value: 0x0000, entity_offset: 11250, length: 6 }, // &orarr;
    HTMLEntityTableEntry { first_value: 0x02A5D, second_value: 0x0000, entity_offset: 11256, length: 4 }, // &ord;
    HTMLEntityTableEntry { first_value: 0x02134, second_value: 0x0000, entity_offset: 11260, length: 6 }, // &order;
    HTMLEntityTableEntry { first_value: 0x02134, second_value: 0x0000, entity_offset: 11266, length: 8 }, // &orderof;
    HTMLEntityTableEntry { first_value: 0x000AA, second_value: 0x0000, entity_offset: 11274, length: 4 }, // &ordf
    HTMLEntityTableEntry { first_value: 0x000AA, second_value: 0x0000, entity_offset: 11274, length: 5 }, // &ordf;
    HTMLEntityTableEntry { first_value: 0x000BA, second_value: 0x0000, entity_offset: 11279, length: 4 }, // &ordm
    HTMLEntityTableEntry { first_value: 0x000BA, second_value: 0x0000, entity_offset: 11279, length: 5 }, // &ordm;
    HTMLEntityTableEntry { first_value: 0x022B6, second_value: 0x0000, entity_offset: 11284, length: 7 }, // &origof;
    HTMLEntityTableEntry { first_value: 0x02A56, second_value: 0x0000, entity_offset: 11291, length: 5 }, // &oror;
    HTMLEntityTableEntry { first_value: 0x02A57, second_value: 0x0000, entity_offset: 11296, length: 8 }, // &orslope;
    HTMLEntityTableEntry { first_value: 0x02A5B, second_value: 0x0000, entity_offset: 11304, length: 4 }, // &orv;
// cpp: html/parser/html_entity_table.cc:3708-3739
    HTMLEntityTableEntry { first_value: 0x02134, second_value: 0x0000, entity_offset: 11308, length: 5 }, // &oscr;
    HTMLEntityTableEntry { first_value: 0x000F8, second_value: 0x0000, entity_offset: 11313, length: 6 }, // &oslash
    HTMLEntityTableEntry { first_value: 0x000F8, second_value: 0x0000, entity_offset: 11313, length: 7 }, // &oslash;
    HTMLEntityTableEntry { first_value: 0x02298, second_value: 0x0000, entity_offset: 11320, length: 5 }, // &osol;
    HTMLEntityTableEntry { first_value: 0x000F5, second_value: 0x0000, entity_offset: 11325, length: 6 }, // &otilde
    HTMLEntityTableEntry { first_value: 0x000F5, second_value: 0x0000, entity_offset: 11325, length: 7 }, // &otilde;
    HTMLEntityTableEntry { first_value: 0x02297, second_value: 0x0000, entity_offset: 5645, length: 7 }, // &otimes;
    HTMLEntityTableEntry { first_value: 0x02A36, second_value: 0x0000, entity_offset: 11332, length: 9 }, // &otimesas;
    HTMLEntityTableEntry { first_value: 0x000F6, second_value: 0x0000, entity_offset: 11341, length: 4 }, // &ouml
    HTMLEntityTableEntry { first_value: 0x000F6, second_value: 0x0000, entity_offset: 11341, length: 5 }, // &ouml;
    HTMLEntityTableEntry { first_value: 0x0233D, second_value: 0x0000, entity_offset: 11346, length: 6 }, // &ovbar;
    HTMLEntityTableEntry { first_value: 0x02225, second_value: 0x0000, entity_offset: 7501, length: 4 }, // &par;
    HTMLEntityTableEntry { first_value: 0x000B6, second_value: 0x0000, entity_offset: 4889, length: 4 }, // &para
    HTMLEntityTableEntry { first_value: 0x000B6, second_value: 0x0000, entity_offset: 11352, length: 5 }, // &para;
    HTMLEntityTableEntry { first_value: 0x02225, second_value: 0x0000, entity_offset: 10562, length: 9 }, // &parallel;
    HTMLEntityTableEntry { first_value: 0x02AF3, second_value: 0x0000, entity_offset: 11357, length: 7 }, // &parsim;
    HTMLEntityTableEntry { first_value: 0x02AFD, second_value: 0x0000, entity_offset: 7506, length: 6 }, // &parsl;
    HTMLEntityTableEntry { first_value: 0x02202, second_value: 0x0000, entity_offset: 8505, length: 5 }, // &part;
    HTMLEntityTableEntry { first_value: 0x0043F, second_value: 0x0000, entity_offset: 11364, length: 4 }, // &pcy;
    HTMLEntityTableEntry { first_value: 0x00025, second_value: 0x0000, entity_offset: 11368, length: 7 }, // &percnt;
    HTMLEntityTableEntry { first_value: 0x0002E, second_value: 0x0000, entity_offset: 11375, length: 7 }, // &period;
    HTMLEntityTableEntry { first_value: 0x02030, second_value: 0x0000, entity_offset: 11382, length: 7 }, // &permil;
    HTMLEntityTableEntry { first_value: 0x022A5, second_value: 0x0000, entity_offset: 11245, length: 5 }, // &perp;
    HTMLEntityTableEntry { first_value: 0x02031, second_value: 0x0000, entity_offset: 11389, length: 8 }, // &pertenk;
    HTMLEntityTableEntry { first_value: 0x1D52D, second_value: 0x0000, entity_offset: 11397, length: 4 }, // &pfr;
    HTMLEntityTableEntry { first_value: 0x003C6, second_value: 0x0000, entity_offset: 11401, length: 4 }, // &phi;
    HTMLEntityTableEntry { first_value: 0x003D5, second_value: 0x0000, entity_offset: 11405, length: 5 }, // &phiv;
    HTMLEntityTableEntry { first_value: 0x02133, second_value: 0x0000, entity_offset: 11410, length: 7 }, // &phmmat;
    HTMLEntityTableEntry { first_value: 0x0260E, second_value: 0x0000, entity_offset: 11417, length: 6 }, // &phone;
    HTMLEntityTableEntry { first_value: 0x003C0, second_value: 0x0000, entity_offset: 11423, length: 3 }, // &pi;
    HTMLEntityTableEntry { first_value: 0x022D4, second_value: 0x0000, entity_offset: 11426, length: 10 }, // &pitchfork;
    HTMLEntityTableEntry { first_value: 0x003D6, second_value: 0x0000, entity_offset: 11436, length: 4 }, // &piv;
// cpp: html/parser/html_entity_table.cc:3740-3771
    HTMLEntityTableEntry { first_value: 0x0210F, second_value: 0x0000, entity_offset: 11440, length: 7 }, // &planck;
    HTMLEntityTableEntry { first_value: 0x0210E, second_value: 0x0000, entity_offset: 11447, length: 8 }, // &planckh;
    HTMLEntityTableEntry { first_value: 0x0210F, second_value: 0x0000, entity_offset: 11455, length: 7 }, // &plankv;
    HTMLEntityTableEntry { first_value: 0x0002B, second_value: 0x0000, entity_offset: 5637, length: 5 }, // &plus;
    HTMLEntityTableEntry { first_value: 0x02A23, second_value: 0x0000, entity_offset: 11462, length: 9 }, // &plusacir;
    HTMLEntityTableEntry { first_value: 0x0229E, second_value: 0x0000, entity_offset: 11471, length: 6 }, // &plusb;
    HTMLEntityTableEntry { first_value: 0x02A22, second_value: 0x0000, entity_offset: 11477, length: 8 }, // &pluscir;
    HTMLEntityTableEntry { first_value: 0x02214, second_value: 0x0000, entity_offset: 11485, length: 7 }, // &plusdo;
    HTMLEntityTableEntry { first_value: 0x02A25, second_value: 0x0000, entity_offset: 11492, length: 7 }, // &plusdu;
    HTMLEntityTableEntry { first_value: 0x02A72, second_value: 0x0000, entity_offset: 11499, length: 6 }, // &pluse;
    HTMLEntityTableEntry { first_value: 0x000B1, second_value: 0x0000, entity_offset: 11505, length: 6 }, // &plusmn
    HTMLEntityTableEntry { first_value: 0x000B1, second_value: 0x0000, entity_offset: 11505, length: 7 }, // &plusmn;
    HTMLEntityTableEntry { first_value: 0x02A26, second_value: 0x0000, entity_offset: 11512, length: 8 }, // &plussim;
    HTMLEntityTableEntry { first_value: 0x02A27, second_value: 0x0000, entity_offset: 11520, length: 8 }, // &plustwo;
    HTMLEntityTableEntry { first_value: 0x000B1, second_value: 0x0000, entity_offset: 11528, length: 3 }, // &pm;
    HTMLEntityTableEntry { first_value: 0x02A15, second_value: 0x0000, entity_offset: 11531, length: 9 }, // &pointint;
    HTMLEntityTableEntry { first_value: 0x1D561, second_value: 0x0000, entity_offset: 11540, length: 5 }, // &popf;
    HTMLEntityTableEntry { first_value: 0x000A3, second_value: 0x0000, entity_offset: 11545, length: 5 }, // &pound
    HTMLEntityTableEntry { first_value: 0x000A3, second_value: 0x0000, entity_offset: 11545, length: 6 }, // &pound;
    HTMLEntityTableEntry { first_value: 0x0227A, second_value: 0x0000, entity_offset: 6713, length: 3 }, // &pr;
    HTMLEntityTableEntry { first_value: 0x02AB3, second_value: 0x0000, entity_offset: 11551, length: 4 }, // &prE;
    HTMLEntityTableEntry { first_value: 0x02AB7, second_value: 0x0000, entity_offset: 11555, length: 5 }, // &prap;
    HTMLEntityTableEntry { first_value: 0x0227C, second_value: 0x0000, entity_offset: 10597, length: 6 }, // &prcue;
    HTMLEntityTableEntry { first_value: 0x02AAF, second_value: 0x0000, entity_offset: 10604, length: 4 }, // &pre;
    HTMLEntityTableEntry { first_value: 0x0227A, second_value: 0x0000, entity_offset: 6795, length: 5 }, // &prec;
    HTMLEntityTableEntry { first_value: 0x02AB7, second_value: 0x0000, entity_offset: 11560, length: 11 }, // &precapprox;
    HTMLEntityTableEntry { first_value: 0x0227C, second_value: 0x0000, entity_offset: 11571, length: 12 }, // &preccurlyeq;
    HTMLEntityTableEntry { first_value: 0x02AAF, second_value: 0x0000, entity_offset: 10615, length: 7 }, // &preceq;
    HTMLEntityTableEntry { first_value: 0x02AB9, second_value: 0x0000, entity_offset: 11583, length: 12 }, // &precnapprox;
    HTMLEntityTableEntry { first_value: 0x02AB5, second_value: 0x0000, entity_offset: 11595, length: 9 }, // &precneqq;
    HTMLEntityTableEntry { first_value: 0x022E8, second_value: 0x0000, entity_offset: 11604, length: 9 }, // &precnsim;
    HTMLEntityTableEntry { first_value: 0x0227E, second_value: 0x0000, entity_offset: 11613, length: 8 }, // &precsim;
// cpp: html/parser/html_entity_table.cc:3772-3803
    HTMLEntityTableEntry { first_value: 0x02032, second_value: 0x0000, entity_offset: 5468, length: 6 }, // &prime;
    HTMLEntityTableEntry { first_value: 0x02119, second_value: 0x0000, entity_offset: 11621, length: 7 }, // &primes;
    HTMLEntityTableEntry { first_value: 0x02AB5, second_value: 0x0000, entity_offset: 11628, length: 5 }, // &prnE;
    HTMLEntityTableEntry { first_value: 0x02AB9, second_value: 0x0000, entity_offset: 11633, length: 6 }, // &prnap;
    HTMLEntityTableEntry { first_value: 0x022E8, second_value: 0x0000, entity_offset: 11639, length: 7 }, // &prnsim;
    HTMLEntityTableEntry { first_value: 0x0220F, second_value: 0x0000, entity_offset: 6632, length: 5 }, // &prod;
    HTMLEntityTableEntry { first_value: 0x0232E, second_value: 0x0000, entity_offset: 11646, length: 9 }, // &profalar;
    HTMLEntityTableEntry { first_value: 0x02312, second_value: 0x0000, entity_offset: 11655, length: 9 }, // &profline;
    HTMLEntityTableEntry { first_value: 0x02313, second_value: 0x0000, entity_offset: 11664, length: 9 }, // &profsurf;
    HTMLEntityTableEntry { first_value: 0x0221D, second_value: 0x0000, entity_offset: 11673, length: 5 }, // &prop;
    HTMLEntityTableEntry { first_value: 0x0221D, second_value: 0x0000, entity_offset: 11678, length: 7 }, // &propto;
    HTMLEntityTableEntry { first_value: 0x0227E, second_value: 0x0000, entity_offset: 11685, length: 6 }, // &prsim;
    HTMLEntityTableEntry { first_value: 0x022B0, second_value: 0x0000, entity_offset: 11691, length: 7 }, // &prurel;
    HTMLEntityTableEntry { first_value: 0x1D4C5, second_value: 0x0000, entity_offset: 11698, length: 5 }, // &pscr;
    HTMLEntityTableEntry { first_value: 0x003C8, second_value: 0x0000, entity_offset: 4787, length: 4 }, // &psi;
    HTMLEntityTableEntry { first_value: 0x02008, second_value: 0x0000, entity_offset: 11703, length: 7 }, // &puncsp;
    HTMLEntityTableEntry { first_value: 0x1D52E, second_value: 0x0000, entity_offset: 11710, length: 4 }, // &qfr;
    HTMLEntityTableEntry { first_value: 0x02A0C, second_value: 0x0000, entity_offset: 11714, length: 5 }, // &qint;
    HTMLEntityTableEntry { first_value: 0x1D562, second_value: 0x0000, entity_offset: 11719, length: 5 }, // &qopf;
    HTMLEntityTableEntry { first_value: 0x02057, second_value: 0x0000, entity_offset: 11724, length: 7 }, // &qprime;
    HTMLEntityTableEntry { first_value: 0x1D4C6, second_value: 0x0000, entity_offset: 11731, length: 5 }, // &qscr;
    HTMLEntityTableEntry { first_value: 0x0210D, second_value: 0x0000, entity_offset: 11736, length: 12 }, // &quaternions;
    HTMLEntityTableEntry { first_value: 0x02A16, second_value: 0x0000, entity_offset: 11748, length: 8 }, // &quatint;
    HTMLEntityTableEntry { first_value: 0x0003F, second_value: 0x0000, entity_offset: 7576, length: 6 }, // &quest;
    HTMLEntityTableEntry { first_value: 0x0225F, second_value: 0x0000, entity_offset: 11756, length: 8 }, // &questeq;
    HTMLEntityTableEntry { first_value: 0x00022, second_value: 0x0000, entity_offset: 11764, length: 4 }, // &quot
    HTMLEntityTableEntry { first_value: 0x00022, second_value: 0x0000, entity_offset: 11764, length: 5 }, // &quot;
    HTMLEntityTableEntry { first_value: 0x021DB, second_value: 0x0000, entity_offset: 11769, length: 6 }, // &rAarr;
    HTMLEntityTableEntry { first_value: 0x021D2, second_value: 0x0000, entity_offset: 10623, length: 5 }, // &rArr;
    HTMLEntityTableEntry { first_value: 0x0291C, second_value: 0x0000, entity_offset: 11775, length: 7 }, // &rAtail;
    HTMLEntityTableEntry { first_value: 0x0290F, second_value: 0x0000, entity_offset: 11782, length: 6 }, // &rBarr;
    HTMLEntityTableEntry { first_value: 0x02964, second_value: 0x0000, entity_offset: 11788, length: 5 }, // &rHar;
// cpp: html/parser/html_entity_table.cc:3804-3835
    HTMLEntityTableEntry { first_value: 0x0223D, second_value: 0x0331, entity_offset: 3687, length: 5 }, // &race;
    HTMLEntityTableEntry { first_value: 0x00155, second_value: 0x0000, entity_offset: 11793, length: 7 }, // &racute;
    HTMLEntityTableEntry { first_value: 0x0221A, second_value: 0x0000, entity_offset: 11800, length: 6 }, // &radic;
    HTMLEntityTableEntry { first_value: 0x029B3, second_value: 0x0000, entity_offset: 11806, length: 9 }, // &raemptyv;
    HTMLEntityTableEntry { first_value: 0x027E9, second_value: 0x0000, entity_offset: 11815, length: 5 }, // &rang;
    HTMLEntityTableEntry { first_value: 0x02992, second_value: 0x0000, entity_offset: 11820, length: 6 }, // &rangd;
    HTMLEntityTableEntry { first_value: 0x029A5, second_value: 0x0000, entity_offset: 11826, length: 6 }, // &range;
    HTMLEntityTableEntry { first_value: 0x027E9, second_value: 0x0000, entity_offset: 11832, length: 7 }, // &rangle;
    HTMLEntityTableEntry { first_value: 0x000BB, second_value: 0x0000, entity_offset: 11839, length: 5 }, // &raquo
    HTMLEntityTableEntry { first_value: 0x000BB, second_value: 0x0000, entity_offset: 11839, length: 6 }, // &raquo;
    HTMLEntityTableEntry { first_value: 0x02192, second_value: 0x0000, entity_offset: 6650, length: 5 }, // &rarr;
    HTMLEntityTableEntry { first_value: 0x02975, second_value: 0x0000, entity_offset: 11845, length: 7 }, // &rarrap;
    HTMLEntityTableEntry { first_value: 0x021E5, second_value: 0x0000, entity_offset: 11852, length: 6 }, // &rarrb;
    HTMLEntityTableEntry { first_value: 0x02920, second_value: 0x0000, entity_offset: 11858, length: 8 }, // &rarrbfs;
    HTMLEntityTableEntry { first_value: 0x02933, second_value: 0x0000, entity_offset: 10635, length: 6 }, // &rarrc;
    HTMLEntityTableEntry { first_value: 0x0291E, second_value: 0x0000, entity_offset: 11866, length: 7 }, // &rarrfs;
    HTMLEntityTableEntry { first_value: 0x021AA, second_value: 0x0000, entity_offset: 11873, length: 7 }, // &rarrhk;
    HTMLEntityTableEntry { first_value: 0x021AC, second_value: 0x0000, entity_offset: 11880, length: 7 }, // &rarrlp;
    HTMLEntityTableEntry { first_value: 0x02945, second_value: 0x0000, entity_offset: 11887, length: 7 }, // &rarrpl;
    HTMLEntityTableEntry { first_value: 0x02974, second_value: 0x0000, entity_offset: 11894, length: 8 }, // &rarrsim;
    HTMLEntityTableEntry { first_value: 0x021A3, second_value: 0x0000, entity_offset: 11902, length: 7 }, // &rarrtl;
    HTMLEntityTableEntry { first_value: 0x0219D, second_value: 0x0000, entity_offset: 10642, length: 6 }, // &rarrw;
    HTMLEntityTableEntry { first_value: 0x0291A, second_value: 0x0000, entity_offset: 11909, length: 7 }, // &ratail;
    HTMLEntityTableEntry { first_value: 0x02236, second_value: 0x0000, entity_offset: 11916, length: 6 }, // &ratio;
    HTMLEntityTableEntry { first_value: 0x0211A, second_value: 0x0000, entity_offset: 11922, length: 10 }, // &rationals;
    HTMLEntityTableEntry { first_value: 0x0290D, second_value: 0x0000, entity_offset: 11932, length: 6 }, // &rbarr;
    HTMLEntityTableEntry { first_value: 0x02773, second_value: 0x0000, entity_offset: 11938, length: 6 }, // &rbbrk;
    HTMLEntityTableEntry { first_value: 0x0007D, second_value: 0x0000, entity_offset: 11944, length: 7 }, // &rbrace;
    HTMLEntityTableEntry { first_value: 0x0005D, second_value: 0x0000, entity_offset: 11951, length: 7 }, // &rbrack;
    HTMLEntityTableEntry { first_value: 0x0298C, second_value: 0x0000, entity_offset: 11958, length: 6 }, // &rbrke;
    HTMLEntityTableEntry { first_value: 0x0298E, second_value: 0x0000, entity_offset: 11964, length: 8 }, // &rbrksld;
    HTMLEntityTableEntry { first_value: 0x02990, second_value: 0x0000, entity_offset: 11972, length: 8 }, // &rbrkslu;
// cpp: html/parser/html_entity_table.cc:3836-3867
    HTMLEntityTableEntry { first_value: 0x00159, second_value: 0x0000, entity_offset: 11980, length: 7 }, // &rcaron;
    HTMLEntityTableEntry { first_value: 0x00157, second_value: 0x0000, entity_offset: 11987, length: 7 }, // &rcedil;
    HTMLEntityTableEntry { first_value: 0x02309, second_value: 0x0000, entity_offset: 11994, length: 6 }, // &rceil;
    HTMLEntityTableEntry { first_value: 0x0007D, second_value: 0x0000, entity_offset: 12000, length: 5 }, // &rcub;
    HTMLEntityTableEntry { first_value: 0x00440, second_value: 0x0000, entity_offset: 1865, length: 4 }, // &rcy;
    HTMLEntityTableEntry { first_value: 0x02937, second_value: 0x0000, entity_offset: 12005, length: 5 }, // &rdca;
    HTMLEntityTableEntry { first_value: 0x02969, second_value: 0x0000, entity_offset: 12010, length: 8 }, // &rdldhar;
    HTMLEntityTableEntry { first_value: 0x0201D, second_value: 0x0000, entity_offset: 12018, length: 6 }, // &rdquo;
    HTMLEntityTableEntry { first_value: 0x0201D, second_value: 0x0000, entity_offset: 12024, length: 7 }, // &rdquor;
    HTMLEntityTableEntry { first_value: 0x021B3, second_value: 0x0000, entity_offset: 12031, length: 5 }, // &rdsh;
    HTMLEntityTableEntry { first_value: 0x0211C, second_value: 0x0000, entity_offset: 12036, length: 5 }, // &real;
    HTMLEntityTableEntry { first_value: 0x0211B, second_value: 0x0000, entity_offset: 12041, length: 8 }, // &realine;
    HTMLEntityTableEntry { first_value: 0x0211C, second_value: 0x0000, entity_offset: 12049, length: 9 }, // &realpart;
    HTMLEntityTableEntry { first_value: 0x0211D, second_value: 0x0000, entity_offset: 12058, length: 6 }, // &reals;
    HTMLEntityTableEntry { first_value: 0x025AD, second_value: 0x0000, entity_offset: 12064, length: 5 }, // &rect;
    HTMLEntityTableEntry { first_value: 0x000AE, second_value: 0x0000, entity_offset: 12069, length: 3 }, // &reg
    HTMLEntityTableEntry { first_value: 0x000AE, second_value: 0x0000, entity_offset: 12069, length: 4 }, // &reg;
    HTMLEntityTableEntry { first_value: 0x0297D, second_value: 0x0000, entity_offset: 12073, length: 7 }, // &rfisht;
    HTMLEntityTableEntry { first_value: 0x0230B, second_value: 0x0000, entity_offset: 12080, length: 7 }, // &rfloor;
    HTMLEntityTableEntry { first_value: 0x1D52F, second_value: 0x0000, entity_offset: 12087, length: 4 }, // &rfr;
    HTMLEntityTableEntry { first_value: 0x021C1, second_value: 0x0000, entity_offset: 9631, length: 6 }, // &rhard;
    HTMLEntityTableEntry { first_value: 0x021C0, second_value: 0x0000, entity_offset: 12091, length: 6 }, // &rharu;
    HTMLEntityTableEntry { first_value: 0x0296C, second_value: 0x0000, entity_offset: 12097, length: 7 }, // &rharul;
    HTMLEntityTableEntry { first_value: 0x003C1, second_value: 0x0000, entity_offset: 12104, length: 4 }, // &rho;
    HTMLEntityTableEntry { first_value: 0x003F1, second_value: 0x0000, entity_offset: 12108, length: 5 }, // &rhov;
    HTMLEntityTableEntry { first_value: 0x02192, second_value: 0x0000, entity_offset: 2254, length: 11 }, // &rightarrow;
    HTMLEntityTableEntry { first_value: 0x021A3, second_value: 0x0000, entity_offset: 12113, length: 15 }, // &rightarrowtail;
    HTMLEntityTableEntry { first_value: 0x021C1, second_value: 0x0000, entity_offset: 12128, length: 17 }, // &rightharpoondown;
    HTMLEntityTableEntry { first_value: 0x021C0, second_value: 0x0000, entity_offset: 12145, length: 15 }, // &rightharpoonup;
    HTMLEntityTableEntry { first_value: 0x021C4, second_value: 0x0000, entity_offset: 12160, length: 16 }, // &rightleftarrows;
    HTMLEntityTableEntry { first_value: 0x021CC, second_value: 0x0000, entity_offset: 12176, length: 18 }, // &rightleftharpoons;
    HTMLEntityTableEntry { first_value: 0x021C9, second_value: 0x0000, entity_offset: 12194, length: 17 }, // &rightrightarrows;
// cpp: html/parser/html_entity_table.cc:3868-3899
    HTMLEntityTableEntry { first_value: 0x0219D, second_value: 0x0000, entity_offset: 9147, length: 16 }, // &rightsquigarrow;
    HTMLEntityTableEntry { first_value: 0x022CC, second_value: 0x0000, entity_offset: 12211, length: 16 }, // &rightthreetimes;
    HTMLEntityTableEntry { first_value: 0x002DA, second_value: 0x0000, entity_offset: 87, length: 5 }, // &ring;
    HTMLEntityTableEntry { first_value: 0x02253, second_value: 0x0000, entity_offset: 12227, length: 13 }, // &risingdotseq;
    HTMLEntityTableEntry { first_value: 0x021C4, second_value: 0x0000, entity_offset: 12240, length: 6 }, // &rlarr;
    HTMLEntityTableEntry { first_value: 0x021CC, second_value: 0x0000, entity_offset: 12246, length: 6 }, // &rlhar;
    HTMLEntityTableEntry { first_value: 0x0200F, second_value: 0x0000, entity_offset: 12252, length: 4 }, // &rlm;
    HTMLEntityTableEntry { first_value: 0x023B1, second_value: 0x0000, entity_offset: 12256, length: 7 }, // &rmoust;
    HTMLEntityTableEntry { first_value: 0x023B1, second_value: 0x0000, entity_offset: 12263, length: 11 }, // &rmoustache;
    HTMLEntityTableEntry { first_value: 0x02AEE, second_value: 0x0000, entity_offset: 12274, length: 6 }, // &rnmid;
    HTMLEntityTableEntry { first_value: 0x027ED, second_value: 0x0000, entity_offset: 12280, length: 6 }, // &roang;
    HTMLEntityTableEntry { first_value: 0x021FE, second_value: 0x0000, entity_offset: 12286, length: 6 }, // &roarr;
    HTMLEntityTableEntry { first_value: 0x027E7, second_value: 0x0000, entity_offset: 12292, length: 6 }, // &robrk;
    HTMLEntityTableEntry { first_value: 0x02986, second_value: 0x0000, entity_offset: 12298, length: 6 }, // &ropar;
    HTMLEntityTableEntry { first_value: 0x1D563, second_value: 0x0000, entity_offset: 12304, length: 5 }, // &ropf;
    HTMLEntityTableEntry { first_value: 0x02A2E, second_value: 0x0000, entity_offset: 12309, length: 7 }, // &roplus;
    HTMLEntityTableEntry { first_value: 0x02A35, second_value: 0x0000, entity_offset: 12316, length: 8 }, // &rotimes;
    HTMLEntityTableEntry { first_value: 0x00029, second_value: 0x0000, entity_offset: 12324, length: 5 }, // &rpar;
    HTMLEntityTableEntry { first_value: 0x02994, second_value: 0x0000, entity_offset: 12329, length: 7 }, // &rpargt;
    HTMLEntityTableEntry { first_value: 0x02A12, second_value: 0x0000, entity_offset: 12336, length: 9 }, // &rppolint;
    HTMLEntityTableEntry { first_value: 0x021C9, second_value: 0x0000, entity_offset: 12345, length: 6 }, // &rrarr;
    HTMLEntityTableEntry { first_value: 0x0203A, second_value: 0x0000, entity_offset: 12351, length: 7 }, // &rsaquo;
    HTMLEntityTableEntry { first_value: 0x1D4C7, second_value: 0x0000, entity_offset: 12358, length: 5 }, // &rscr;
    HTMLEntityTableEntry { first_value: 0x021B1, second_value: 0x0000, entity_offset: 12363, length: 4 }, // &rsh;
    HTMLEntityTableEntry { first_value: 0x0005D, second_value: 0x0000, entity_offset: 12367, length: 5 }, // &rsqb;
    HTMLEntityTableEntry { first_value: 0x02019, second_value: 0x0000, entity_offset: 12372, length: 6 }, // &rsquo;
    HTMLEntityTableEntry { first_value: 0x02019, second_value: 0x0000, entity_offset: 12378, length: 7 }, // &rsquor;
    HTMLEntityTableEntry { first_value: 0x022CC, second_value: 0x0000, entity_offset: 12385, length: 7 }, // &rthree;
    HTMLEntityTableEntry { first_value: 0x022CA, second_value: 0x0000, entity_offset: 12392, length: 7 }, // &rtimes;
    HTMLEntityTableEntry { first_value: 0x025B9, second_value: 0x0000, entity_offset: 9642, length: 5 }, // &rtri;
    HTMLEntityTableEntry { first_value: 0x022B5, second_value: 0x0000, entity_offset: 10667, length: 6 }, // &rtrie;
    HTMLEntityTableEntry { first_value: 0x025B8, second_value: 0x0000, entity_offset: 12399, length: 6 }, // &rtrif;
// cpp: html/parser/html_entity_table.cc:3900-3931
    HTMLEntityTableEntry { first_value: 0x029CE, second_value: 0x0000, entity_offset: 12405, length: 9 }, // &rtriltri;
    HTMLEntityTableEntry { first_value: 0x02968, second_value: 0x0000, entity_offset: 12414, length: 8 }, // &ruluhar;
    HTMLEntityTableEntry { first_value: 0x0211E, second_value: 0x0000, entity_offset: 12422, length: 3 }, // &rx;
    HTMLEntityTableEntry { first_value: 0x0015B, second_value: 0x0000, entity_offset: 12425, length: 7 }, // &sacute;
    HTMLEntityTableEntry { first_value: 0x0201A, second_value: 0x0000, entity_offset: 12432, length: 6 }, // &sbquo;
    HTMLEntityTableEntry { first_value: 0x0227B, second_value: 0x0000, entity_offset: 6719, length: 3 }, // &sc;
    HTMLEntityTableEntry { first_value: 0x02AB4, second_value: 0x0000, entity_offset: 12438, length: 4 }, // &scE;
    HTMLEntityTableEntry { first_value: 0x02AB8, second_value: 0x0000, entity_offset: 12442, length: 5 }, // &scap;
    HTMLEntityTableEntry { first_value: 0x00161, second_value: 0x0000, entity_offset: 12447, length: 7 }, // &scaron;
    HTMLEntityTableEntry { first_value: 0x0227D, second_value: 0x0000, entity_offset: 10678, length: 6 }, // &sccue;
    HTMLEntityTableEntry { first_value: 0x02AB0, second_value: 0x0000, entity_offset: 10685, length: 4 }, // &sce;
    HTMLEntityTableEntry { first_value: 0x0015F, second_value: 0x0000, entity_offset: 12454, length: 7 }, // &scedil;
    HTMLEntityTableEntry { first_value: 0x0015D, second_value: 0x0000, entity_offset: 12461, length: 6 }, // &scirc;
    HTMLEntityTableEntry { first_value: 0x02AB6, second_value: 0x0000, entity_offset: 12467, length: 5 }, // &scnE;
    HTMLEntityTableEntry { first_value: 0x02ABA, second_value: 0x0000, entity_offset: 12472, length: 6 }, // &scnap;
    HTMLEntityTableEntry { first_value: 0x022E9, second_value: 0x0000, entity_offset: 12478, length: 7 }, // &scnsim;
    HTMLEntityTableEntry { first_value: 0x02A13, second_value: 0x0000, entity_offset: 12485, length: 9 }, // &scpolint;
    HTMLEntityTableEntry { first_value: 0x0227F, second_value: 0x0000, entity_offset: 12494, length: 6 }, // &scsim;
    HTMLEntityTableEntry { first_value: 0x00441, second_value: 0x0000, entity_offset: 7267, length: 4 }, // &scy;
    HTMLEntityTableEntry { first_value: 0x022C5, second_value: 0x0000, entity_offset: 7411, length: 5 }, // &sdot;
    HTMLEntityTableEntry { first_value: 0x022A1, second_value: 0x0000, entity_offset: 12500, length: 6 }, // &sdotb;
    HTMLEntityTableEntry { first_value: 0x02A66, second_value: 0x0000, entity_offset: 12506, length: 6 }, // &sdote;
    HTMLEntityTableEntry { first_value: 0x021D8, second_value: 0x0000, entity_offset: 12512, length: 6 }, // &seArr;
    HTMLEntityTableEntry { first_value: 0x02925, second_value: 0x0000, entity_offset: 12518, length: 7 }, // &searhk;
    HTMLEntityTableEntry { first_value: 0x02198, second_value: 0x0000, entity_offset: 12525, length: 6 }, // &searr;
    HTMLEntityTableEntry { first_value: 0x02198, second_value: 0x0000, entity_offset: 12531, length: 8 }, // &searrow;
    HTMLEntityTableEntry { first_value: 0x000A7, second_value: 0x0000, entity_offset: 1765, length: 4 }, // &sect
    HTMLEntityTableEntry { first_value: 0x000A7, second_value: 0x0000, entity_offset: 12539, length: 5 }, // &sect;
    HTMLEntityTableEntry { first_value: 0x0003B, second_value: 0x0000, entity_offset: 6184, length: 5 }, // &semi;
    HTMLEntityTableEntry { first_value: 0x02929, second_value: 0x0000, entity_offset: 12544, length: 7 }, // &seswar;
    HTMLEntityTableEntry { first_value: 0x02216, second_value: 0x0000, entity_offset: 12551, length: 9 }, // &setminus;
    HTMLEntityTableEntry { first_value: 0x02216, second_value: 0x0000, entity_offset: 12560, length: 6 }, // &setmn;
// cpp: html/parser/html_entity_table.cc:3932-3963
    HTMLEntityTableEntry { first_value: 0x02736, second_value: 0x0000, entity_offset: 12566, length: 5 }, // &sext;
    HTMLEntityTableEntry { first_value: 0x1D530, second_value: 0x0000, entity_offset: 12571, length: 4 }, // &sfr;
    HTMLEntityTableEntry { first_value: 0x02322, second_value: 0x0000, entity_offset: 12575, length: 7 }, // &sfrown;
    HTMLEntityTableEntry { first_value: 0x0266F, second_value: 0x0000, entity_offset: 12582, length: 6 }, // &sharp;
    HTMLEntityTableEntry { first_value: 0x00449, second_value: 0x0000, entity_offset: 12588, length: 7 }, // &shchcy;
    HTMLEntityTableEntry { first_value: 0x00448, second_value: 0x0000, entity_offset: 12595, length: 5 }, // &shcy;
    HTMLEntityTableEntry { first_value: 0x02223, second_value: 0x0000, entity_offset: 10695, length: 9 }, // &shortmid;
    HTMLEntityTableEntry { first_value: 0x02225, second_value: 0x0000, entity_offset: 10705, length: 14 }, // &shortparallel;
    HTMLEntityTableEntry { first_value: 0x000AD, second_value: 0x0000, entity_offset: 12600, length: 3 }, // &shy
    HTMLEntityTableEntry { first_value: 0x000AD, second_value: 0x0000, entity_offset: 12600, length: 4 }, // &shy;
    HTMLEntityTableEntry { first_value: 0x003C3, second_value: 0x0000, entity_offset: 12604, length: 6 }, // &sigma;
    HTMLEntityTableEntry { first_value: 0x003C2, second_value: 0x0000, entity_offset: 12610, length: 7 }, // &sigmaf;
    HTMLEntityTableEntry { first_value: 0x003C2, second_value: 0x0000, entity_offset: 12617, length: 7 }, // &sigmav;
    HTMLEntityTableEntry { first_value: 0x0223C, second_value: 0x0000, entity_offset: 1300, length: 4 }, // &sim;
    HTMLEntityTableEntry { first_value: 0x02A6A, second_value: 0x0000, entity_offset: 12624, length: 7 }, // &simdot;
    HTMLEntityTableEntry { first_value: 0x02243, second_value: 0x0000, entity_offset: 6195, length: 5 }, // &sime;
    HTMLEntityTableEntry { first_value: 0x02243, second_value: 0x0000, entity_offset: 5486, length: 6 }, // &simeq;
    HTMLEntityTableEntry { first_value: 0x02A9E, second_value: 0x0000, entity_offset: 9675, length: 5 }, // &simg;
    HTMLEntityTableEntry { first_value: 0x02AA0, second_value: 0x0000, entity_offset: 12631, length: 6 }, // &simgE;
    HTMLEntityTableEntry { first_value: 0x02A9D, second_value: 0x0000, entity_offset: 8094, length: 5 }, // &siml;
    HTMLEntityTableEntry { first_value: 0x02A9F, second_value: 0x0000, entity_offset: 12637, length: 6 }, // &simlE;
    HTMLEntityTableEntry { first_value: 0x02246, second_value: 0x0000, entity_offset: 12643, length: 6 }, // &simne;
    HTMLEntityTableEntry { first_value: 0x02A24, second_value: 0x0000, entity_offset: 12649, length: 8 }, // &simplus;
    HTMLEntityTableEntry { first_value: 0x02972, second_value: 0x0000, entity_offset: 12657, length: 8 }, // &simrarr;
    HTMLEntityTableEntry { first_value: 0x02190, second_value: 0x0000, entity_offset: 12665, length: 6 }, // &slarr;
    HTMLEntityTableEntry { first_value: 0x02216, second_value: 0x0000, entity_offset: 12671, length: 14 }, // &smallsetminus;
    HTMLEntityTableEntry { first_value: 0x02A33, second_value: 0x0000, entity_offset: 12685, length: 7 }, // &smashp;
    HTMLEntityTableEntry { first_value: 0x029E4, second_value: 0x0000, entity_offset: 12692, length: 9 }, // &smeparsl;
    HTMLEntityTableEntry { first_value: 0x02223, second_value: 0x0000, entity_offset: 10733, length: 5 }, // &smid;
    HTMLEntityTableEntry { first_value: 0x02323, second_value: 0x0000, entity_offset: 12701, length: 6 }, // &smile;
    HTMLEntityTableEntry { first_value: 0x02AAA, second_value: 0x0000, entity_offset: 12707, length: 4 }, // &smt;
    HTMLEntityTableEntry { first_value: 0x02AAC, second_value: 0x0000, entity_offset: 12711, length: 5 }, // &smte;
// cpp: html/parser/html_entity_table.cc:3964-3995
    HTMLEntityTableEntry { first_value: 0x02AAC, second_value: 0xFE00, entity_offset: 12716, length: 6 }, // &smtes;
    HTMLEntityTableEntry { first_value: 0x0044C, second_value: 0x0000, entity_offset: 12722, length: 7 }, // &softcy;
    HTMLEntityTableEntry { first_value: 0x0002F, second_value: 0x0000, entity_offset: 6201, length: 4 }, // &sol;
    HTMLEntityTableEntry { first_value: 0x029C4, second_value: 0x0000, entity_offset: 6206, length: 5 }, // &solb;
    HTMLEntityTableEntry { first_value: 0x0233F, second_value: 0x0000, entity_offset: 12729, length: 7 }, // &solbar;
    HTMLEntityTableEntry { first_value: 0x1D564, second_value: 0x0000, entity_offset: 12736, length: 5 }, // &sopf;
    HTMLEntityTableEntry { first_value: 0x02660, second_value: 0x0000, entity_offset: 12741, length: 7 }, // &spades;
    HTMLEntityTableEntry { first_value: 0x02660, second_value: 0x0000, entity_offset: 12748, length: 10 }, // &spadesuit;
    HTMLEntityTableEntry { first_value: 0x02225, second_value: 0x0000, entity_offset: 10739, length: 5 }, // &spar;
    HTMLEntityTableEntry { first_value: 0x02293, second_value: 0x0000, entity_offset: 12758, length: 6 }, // &sqcap;
    HTMLEntityTableEntry { first_value: 0x02293, second_value: 0xFE00, entity_offset: 12764, length: 7 }, // &sqcaps;
    HTMLEntityTableEntry { first_value: 0x02294, second_value: 0x0000, entity_offset: 5655, length: 6 }, // &sqcup;
    HTMLEntityTableEntry { first_value: 0x02294, second_value: 0xFE00, entity_offset: 12771, length: 7 }, // &sqcups;
    HTMLEntityTableEntry { first_value: 0x0228F, second_value: 0x0000, entity_offset: 12778, length: 6 }, // &sqsub;
    HTMLEntityTableEntry { first_value: 0x02291, second_value: 0x0000, entity_offset: 10745, length: 7 }, // &sqsube;
    HTMLEntityTableEntry { first_value: 0x0228F, second_value: 0x0000, entity_offset: 12784, length: 9 }, // &sqsubset;
    HTMLEntityTableEntry { first_value: 0x02291, second_value: 0x0000, entity_offset: 12793, length: 11 }, // &sqsubseteq;
    HTMLEntityTableEntry { first_value: 0x02290, second_value: 0x0000, entity_offset: 12804, length: 6 }, // &sqsup;
    HTMLEntityTableEntry { first_value: 0x02292, second_value: 0x0000, entity_offset: 10753, length: 7 }, // &sqsupe;
    HTMLEntityTableEntry { first_value: 0x02290, second_value: 0x0000, entity_offset: 12810, length: 9 }, // &sqsupset;
    HTMLEntityTableEntry { first_value: 0x02292, second_value: 0x0000, entity_offset: 12819, length: 11 }, // &sqsupseteq;
    HTMLEntityTableEntry { first_value: 0x025A1, second_value: 0x0000, entity_offset: 12830, length: 4 }, // &squ;
    HTMLEntityTableEntry { first_value: 0x025A1, second_value: 0x0000, entity_offset: 5749, length: 7 }, // &square;
    HTMLEntityTableEntry { first_value: 0x025AA, second_value: 0x0000, entity_offset: 12834, length: 7 }, // &squarf;
    HTMLEntityTableEntry { first_value: 0x025AA, second_value: 0x0000, entity_offset: 12841, length: 5 }, // &squf;
    HTMLEntityTableEntry { first_value: 0x02192, second_value: 0x0000, entity_offset: 12846, length: 6 }, // &srarr;
    HTMLEntityTableEntry { first_value: 0x1D4C8, second_value: 0x0000, entity_offset: 12852, length: 5 }, // &sscr;
    HTMLEntityTableEntry { first_value: 0x02216, second_value: 0x0000, entity_offset: 12857, length: 7 }, // &ssetmn;
    HTMLEntityTableEntry { first_value: 0x02323, second_value: 0x0000, entity_offset: 12864, length: 7 }, // &ssmile;
    HTMLEntityTableEntry { first_value: 0x022C6, second_value: 0x0000, entity_offset: 12871, length: 7 }, // &sstarf;
    HTMLEntityTableEntry { first_value: 0x02606, second_value: 0x0000, entity_offset: 5664, length: 5 }, // &star;
    HTMLEntityTableEntry { first_value: 0x02605, second_value: 0x0000, entity_offset: 12872, length: 6 }, // &starf;
// cpp: html/parser/html_entity_table.cc:3996-4027
    HTMLEntityTableEntry { first_value: 0x003F5, second_value: 0x0000, entity_offset: 12878, length: 16 }, // &straightepsilon;
    HTMLEntityTableEntry { first_value: 0x003D5, second_value: 0x0000, entity_offset: 12894, length: 12 }, // &straightphi;
    HTMLEntityTableEntry { first_value: 0x000AF, second_value: 0x0000, entity_offset: 12906, length: 6 }, // &strns;
    HTMLEntityTableEntry { first_value: 0x02282, second_value: 0x0000, entity_offset: 6216, length: 4 }, // &sub;
    HTMLEntityTableEntry { first_value: 0x02AC5, second_value: 0x0000, entity_offset: 10766, length: 5 }, // &subE;
    HTMLEntityTableEntry { first_value: 0x02ABD, second_value: 0x0000, entity_offset: 12912, length: 7 }, // &subdot;
    HTMLEntityTableEntry { first_value: 0x02286, second_value: 0x0000, entity_offset: 6672, length: 5 }, // &sube;
    HTMLEntityTableEntry { first_value: 0x02AC3, second_value: 0x0000, entity_offset: 12919, length: 8 }, // &subedot;
    HTMLEntityTableEntry { first_value: 0x02AC1, second_value: 0x0000, entity_offset: 12927, length: 8 }, // &submult;
    HTMLEntityTableEntry { first_value: 0x02ACB, second_value: 0x0000, entity_offset: 12935, length: 6 }, // &subnE;
    HTMLEntityTableEntry { first_value: 0x0228A, second_value: 0x0000, entity_offset: 12941, length: 6 }, // &subne;
    HTMLEntityTableEntry { first_value: 0x02ABF, second_value: 0x0000, entity_offset: 12947, length: 8 }, // &subplus;
    HTMLEntityTableEntry { first_value: 0x02979, second_value: 0x0000, entity_offset: 12955, length: 8 }, // &subrarr;
    HTMLEntityTableEntry { first_value: 0x02282, second_value: 0x0000, entity_offset: 10778, length: 7 }, // &subset;
    HTMLEntityTableEntry { first_value: 0x02286, second_value: 0x0000, entity_offset: 10786, length: 9 }, // &subseteq;
    HTMLEntityTableEntry { first_value: 0x02AC5, second_value: 0x0000, entity_offset: 10796, length: 10 }, // &subseteqq;
    HTMLEntityTableEntry { first_value: 0x0228A, second_value: 0x0000, entity_offset: 12963, length: 10 }, // &subsetneq;
    HTMLEntityTableEntry { first_value: 0x02ACB, second_value: 0x0000, entity_offset: 12973, length: 11 }, // &subsetneqq;
    HTMLEntityTableEntry { first_value: 0x02AC7, second_value: 0x0000, entity_offset: 12984, length: 7 }, // &subsim;
    HTMLEntityTableEntry { first_value: 0x02AD5, second_value: 0x0000, entity_offset: 12991, length: 7 }, // &subsub;
    HTMLEntityTableEntry { first_value: 0x02AD3, second_value: 0x0000, entity_offset: 12998, length: 7 }, // &subsup;
    HTMLEntityTableEntry { first_value: 0x0227B, second_value: 0x0000, entity_offset: 6807, length: 5 }, // &succ;
    HTMLEntityTableEntry { first_value: 0x02AB8, second_value: 0x0000, entity_offset: 13005, length: 11 }, // &succapprox;
    HTMLEntityTableEntry { first_value: 0x0227D, second_value: 0x0000, entity_offset: 13016, length: 12 }, // &succcurlyeq;
    HTMLEntityTableEntry { first_value: 0x02AB0, second_value: 0x0000, entity_offset: 10813, length: 7 }, // &succeq;
    HTMLEntityTableEntry { first_value: 0x02ABA, second_value: 0x0000, entity_offset: 13028, length: 12 }, // &succnapprox;
    HTMLEntityTableEntry { first_value: 0x02AB6, second_value: 0x0000, entity_offset: 13040, length: 9 }, // &succneqq;
    HTMLEntityTableEntry { first_value: 0x022E9, second_value: 0x0000, entity_offset: 13049, length: 9 }, // &succnsim;
    HTMLEntityTableEntry { first_value: 0x0227F, second_value: 0x0000, entity_offset: 13058, length: 8 }, // &succsim;
    HTMLEntityTableEntry { first_value: 0x02211, second_value: 0x0000, entity_offset: 13066, length: 4 }, // &sum;
    HTMLEntityTableEntry { first_value: 0x0266A, second_value: 0x0000, entity_offset: 13070, length: 5 }, // &sung;
    HTMLEntityTableEntry { first_value: 0x000B9, second_value: 0x0000, entity_offset: 13075, length: 4 }, // &sup1
// cpp: html/parser/html_entity_table.cc:4028-4059
    HTMLEntityTableEntry { first_value: 0x000B9, second_value: 0x0000, entity_offset: 13075, length: 5 }, // &sup1;
    HTMLEntityTableEntry { first_value: 0x000B2, second_value: 0x0000, entity_offset: 13080, length: 4 }, // &sup2
    HTMLEntityTableEntry { first_value: 0x000B2, second_value: 0x0000, entity_offset: 13080, length: 5 }, // &sup2;
    HTMLEntityTableEntry { first_value: 0x000B3, second_value: 0x0000, entity_offset: 13085, length: 4 }, // &sup3
    HTMLEntityTableEntry { first_value: 0x000B3, second_value: 0x0000, entity_offset: 13085, length: 5 }, // &sup3;
    HTMLEntityTableEntry { first_value: 0x02283, second_value: 0x0000, entity_offset: 6678, length: 4 }, // &sup;
    HTMLEntityTableEntry { first_value: 0x02AC6, second_value: 0x0000, entity_offset: 10826, length: 5 }, // &supE;
    HTMLEntityTableEntry { first_value: 0x02ABE, second_value: 0x0000, entity_offset: 13090, length: 7 }, // &supdot;
    HTMLEntityTableEntry { first_value: 0x02AD8, second_value: 0x0000, entity_offset: 13097, length: 8 }, // &supdsub;
    HTMLEntityTableEntry { first_value: 0x02287, second_value: 0x0000, entity_offset: 6683, length: 5 }, // &supe;
    HTMLEntityTableEntry { first_value: 0x02AC4, second_value: 0x0000, entity_offset: 13105, length: 8 }, // &supedot;
    HTMLEntityTableEntry { first_value: 0x027C9, second_value: 0x0000, entity_offset: 13113, length: 8 }, // &suphsol;
    HTMLEntityTableEntry { first_value: 0x02AD7, second_value: 0x0000, entity_offset: 13121, length: 8 }, // &suphsub;
    HTMLEntityTableEntry { first_value: 0x0297B, second_value: 0x0000, entity_offset: 13129, length: 8 }, // &suplarr;
    HTMLEntityTableEntry { first_value: 0x02AC2, second_value: 0x0000, entity_offset: 13137, length: 8 }, // &supmult;
    HTMLEntityTableEntry { first_value: 0x02ACC, second_value: 0x0000, entity_offset: 13145, length: 6 }, // &supnE;
    HTMLEntityTableEntry { first_value: 0x0228B, second_value: 0x0000, entity_offset: 13151, length: 6 }, // &supne;
    HTMLEntityTableEntry { first_value: 0x02AC0, second_value: 0x0000, entity_offset: 13157, length: 8 }, // &supplus;
    HTMLEntityTableEntry { first_value: 0x02283, second_value: 0x0000, entity_offset: 10838, length: 7 }, // &supset;
    HTMLEntityTableEntry { first_value: 0x02287, second_value: 0x0000, entity_offset: 10846, length: 9 }, // &supseteq;
    HTMLEntityTableEntry { first_value: 0x02AC6, second_value: 0x0000, entity_offset: 10856, length: 10 }, // &supseteqq;
    HTMLEntityTableEntry { first_value: 0x0228B, second_value: 0x0000, entity_offset: 13165, length: 10 }, // &supsetneq;
    HTMLEntityTableEntry { first_value: 0x02ACC, second_value: 0x0000, entity_offset: 13175, length: 11 }, // &supsetneqq;
    HTMLEntityTableEntry { first_value: 0x02AC8, second_value: 0x0000, entity_offset: 13186, length: 7 }, // &supsim;
    HTMLEntityTableEntry { first_value: 0x02AD4, second_value: 0x0000, entity_offset: 13193, length: 7 }, // &supsub;
    HTMLEntityTableEntry { first_value: 0x02AD6, second_value: 0x0000, entity_offset: 13200, length: 7 }, // &supsup;
    HTMLEntityTableEntry { first_value: 0x021D9, second_value: 0x0000, entity_offset: 13207, length: 6 }, // &swArr;
    HTMLEntityTableEntry { first_value: 0x02926, second_value: 0x0000, entity_offset: 13213, length: 7 }, // &swarhk;
    HTMLEntityTableEntry { first_value: 0x02199, second_value: 0x0000, entity_offset: 13220, length: 6 }, // &swarr;
    HTMLEntityTableEntry { first_value: 0x02199, second_value: 0x0000, entity_offset: 13226, length: 8 }, // &swarrow;
    HTMLEntityTableEntry { first_value: 0x0292A, second_value: 0x0000, entity_offset: 13234, length: 7 }, // &swnwar;
    HTMLEntityTableEntry { first_value: 0x000DF, second_value: 0x0000, entity_offset: 13241, length: 5 }, // &szlig
// cpp: html/parser/html_entity_table.cc:4060-4091
    HTMLEntityTableEntry { first_value: 0x000DF, second_value: 0x0000, entity_offset: 13241, length: 6 }, // &szlig;
    HTMLEntityTableEntry { first_value: 0x02316, second_value: 0x0000, entity_offset: 13247, length: 7 }, // &target;
    HTMLEntityTableEntry { first_value: 0x003C4, second_value: 0x0000, entity_offset: 13254, length: 4 }, // &tau;
    HTMLEntityTableEntry { first_value: 0x023B4, second_value: 0x0000, entity_offset: 5524, length: 5 }, // &tbrk;
    HTMLEntityTableEntry { first_value: 0x00165, second_value: 0x0000, entity_offset: 13258, length: 7 }, // &tcaron;
    HTMLEntityTableEntry { first_value: 0x00163, second_value: 0x0000, entity_offset: 13265, length: 7 }, // &tcedil;
    HTMLEntityTableEntry { first_value: 0x00442, second_value: 0x0000, entity_offset: 12725, length: 4 }, // &tcy;
    HTMLEntityTableEntry { first_value: 0x020DB, second_value: 0x0000, entity_offset: 6689, length: 5 }, // &tdot;
    HTMLEntityTableEntry { first_value: 0x02315, second_value: 0x0000, entity_offset: 13272, length: 7 }, // &telrec;
    HTMLEntityTableEntry { first_value: 0x1D531, second_value: 0x0000, entity_offset: 13279, length: 4 }, // &tfr;
    HTMLEntityTableEntry { first_value: 0x02234, second_value: 0x0000, entity_offset: 13283, length: 7 }, // &there4;
    HTMLEntityTableEntry { first_value: 0x02234, second_value: 0x0000, entity_offset: 13290, length: 10 }, // &therefore;
    HTMLEntityTableEntry { first_value: 0x003B8, second_value: 0x0000, entity_offset: 13300, length: 6 }, // &theta;
    HTMLEntityTableEntry { first_value: 0x003D1, second_value: 0x0000, entity_offset: 13306, length: 9 }, // &thetasym;
    HTMLEntityTableEntry { first_value: 0x003D1, second_value: 0x0000, entity_offset: 13315, length: 7 }, // &thetav;
    HTMLEntityTableEntry { first_value: 0x02248, second_value: 0x0000, entity_offset: 13322, length: 12 }, // &thickapprox;
    HTMLEntityTableEntry { first_value: 0x0223C, second_value: 0x0000, entity_offset: 13334, length: 9 }, // &thicksim;
    HTMLEntityTableEntry { first_value: 0x02009, second_value: 0x0000, entity_offset: 13343, length: 7 }, // &thinsp;
    HTMLEntityTableEntry { first_value: 0x02248, second_value: 0x0000, entity_offset: 13350, length: 6 }, // &thkap;
    HTMLEntityTableEntry { first_value: 0x0223C, second_value: 0x0000, entity_offset: 13356, length: 7 }, // &thksim;
    HTMLEntityTableEntry { first_value: 0x000FE, second_value: 0x0000, entity_offset: 13363, length: 5 }, // &thorn
    HTMLEntityTableEntry { first_value: 0x000FE, second_value: 0x0000, entity_offset: 13363, length: 6 }, // &thorn;
    HTMLEntityTableEntry { first_value: 0x002DC, second_value: 0x0000, entity_offset: 105, length: 6 }, // &tilde;
    HTMLEntityTableEntry { first_value: 0x000D7, second_value: 0x0000, entity_offset: 3663, length: 5 }, // &times
    HTMLEntityTableEntry { first_value: 0x000D7, second_value: 0x0000, entity_offset: 3663, length: 6 }, // &times;
    HTMLEntityTableEntry { first_value: 0x022A0, second_value: 0x0000, entity_offset: 13369, length: 7 }, // &timesb;
    HTMLEntityTableEntry { first_value: 0x02A31, second_value: 0x0000, entity_offset: 13376, length: 9 }, // &timesbar;
    HTMLEntityTableEntry { first_value: 0x02A30, second_value: 0x0000, entity_offset: 13385, length: 7 }, // &timesd;
    HTMLEntityTableEntry { first_value: 0x0222D, second_value: 0x0000, entity_offset: 7765, length: 5 }, // &tint;
    HTMLEntityTableEntry { first_value: 0x02928, second_value: 0x0000, entity_offset: 13392, length: 5 }, // &toea;
    HTMLEntityTableEntry { first_value: 0x022A4, second_value: 0x0000, entity_offset: 13397, length: 4 }, // &top;
    HTMLEntityTableEntry { first_value: 0x02336, second_value: 0x0000, entity_offset: 13401, length: 7 }, // &topbot;
// cpp: html/parser/html_entity_table.cc:4092-4123
    HTMLEntityTableEntry { first_value: 0x02AF1, second_value: 0x0000, entity_offset: 13408, length: 7 }, // &topcir;
    HTMLEntityTableEntry { first_value: 0x1D565, second_value: 0x0000, entity_offset: 13415, length: 5 }, // &topf;
    HTMLEntityTableEntry { first_value: 0x02ADA, second_value: 0x0000, entity_offset: 13420, length: 8 }, // &topfork;
    HTMLEntityTableEntry { first_value: 0x02929, second_value: 0x0000, entity_offset: 13428, length: 5 }, // &tosa;
    HTMLEntityTableEntry { first_value: 0x02034, second_value: 0x0000, entity_offset: 13433, length: 7 }, // &tprime;
    HTMLEntityTableEntry { first_value: 0x02122, second_value: 0x0000, entity_offset: 13440, length: 6 }, // &trade;
    HTMLEntityTableEntry { first_value: 0x025B5, second_value: 0x0000, entity_offset: 5761, length: 9 }, // &triangle;
    HTMLEntityTableEntry { first_value: 0x025BF, second_value: 0x0000, entity_offset: 5672, length: 13 }, // &triangledown;
    HTMLEntityTableEntry { first_value: 0x025C3, second_value: 0x0000, entity_offset: 5793, length: 13 }, // &triangleleft;
    HTMLEntityTableEntry { first_value: 0x022B4, second_value: 0x0000, entity_offset: 10898, length: 15 }, // &trianglelefteq;
    HTMLEntityTableEntry { first_value: 0x0225C, second_value: 0x0000, entity_offset: 13446, length: 10 }, // &triangleq;
    HTMLEntityTableEntry { first_value: 0x025B9, second_value: 0x0000, entity_offset: 5811, length: 14 }, // &triangleright;
    HTMLEntityTableEntry { first_value: 0x022B5, second_value: 0x0000, entity_offset: 10929, length: 16 }, // &trianglerighteq;
    HTMLEntityTableEntry { first_value: 0x025EC, second_value: 0x0000, entity_offset: 13456, length: 7 }, // &tridot;
    HTMLEntityTableEntry { first_value: 0x0225C, second_value: 0x0000, entity_offset: 9759, length: 5 }, // &trie;
    HTMLEntityTableEntry { first_value: 0x02A3A, second_value: 0x0000, entity_offset: 13463, length: 9 }, // &triminus;
    HTMLEntityTableEntry { first_value: 0x02A39, second_value: 0x0000, entity_offset: 13472, length: 8 }, // &triplus;
    HTMLEntityTableEntry { first_value: 0x029CD, second_value: 0x0000, entity_offset: 13480, length: 6 }, // &trisb;
    HTMLEntityTableEntry { first_value: 0x02A3B, second_value: 0x0000, entity_offset: 13486, length: 8 }, // &tritime;
    HTMLEntityTableEntry { first_value: 0x023E2, second_value: 0x0000, entity_offset: 13494, length: 9 }, // &trpezium;
    HTMLEntityTableEntry { first_value: 0x1D4C9, second_value: 0x0000, entity_offset: 13503, length: 5 }, // &tscr;
    HTMLEntityTableEntry { first_value: 0x00446, second_value: 0x0000, entity_offset: 13508, length: 5 }, // &tscy;
    HTMLEntityTableEntry { first_value: 0x0045B, second_value: 0x0000, entity_offset: 13513, length: 6 }, // &tshcy;
    HTMLEntityTableEntry { first_value: 0x00167, second_value: 0x0000, entity_offset: 13519, length: 7 }, // &tstrok;
    HTMLEntityTableEntry { first_value: 0x0226C, second_value: 0x0000, entity_offset: 13526, length: 6 }, // &twixt;
    HTMLEntityTableEntry { first_value: 0x0219E, second_value: 0x0000, entity_offset: 13532, length: 17 }, // &twoheadleftarrow;
    HTMLEntityTableEntry { first_value: 0x021A0, second_value: 0x0000, entity_offset: 13549, length: 18 }, // &twoheadrightarrow;
    HTMLEntityTableEntry { first_value: 0x021D1, second_value: 0x0000, entity_offset: 13567, length: 5 }, // &uArr;
    HTMLEntityTableEntry { first_value: 0x02963, second_value: 0x0000, entity_offset: 13572, length: 5 }, // &uHar;
    HTMLEntityTableEntry { first_value: 0x000FA, second_value: 0x0000, entity_offset: 13577, length: 6 }, // &uacute
    HTMLEntityTableEntry { first_value: 0x000FA, second_value: 0x0000, entity_offset: 13577, length: 7 }, // &uacute;
    HTMLEntityTableEntry { first_value: 0x02191, second_value: 0x0000, entity_offset: 7301, length: 5 }, // &uarr;
// cpp: html/parser/html_entity_table.cc:4124-4155
    HTMLEntityTableEntry { first_value: 0x0045E, second_value: 0x0000, entity_offset: 13584, length: 6 }, // &ubrcy;
    HTMLEntityTableEntry { first_value: 0x0016D, second_value: 0x0000, entity_offset: 13590, length: 7 }, // &ubreve;
    HTMLEntityTableEntry { first_value: 0x000FB, second_value: 0x0000, entity_offset: 13597, length: 5 }, // &ucirc
    HTMLEntityTableEntry { first_value: 0x000FB, second_value: 0x0000, entity_offset: 13597, length: 6 }, // &ucirc;
    HTMLEntityTableEntry { first_value: 0x00443, second_value: 0x0000, entity_offset: 13603, length: 4 }, // &ucy;
    HTMLEntityTableEntry { first_value: 0x021C5, second_value: 0x0000, entity_offset: 13607, length: 6 }, // &udarr;
    HTMLEntityTableEntry { first_value: 0x00171, second_value: 0x0000, entity_offset: 13613, length: 7 }, // &udblac;
    HTMLEntityTableEntry { first_value: 0x0296E, second_value: 0x0000, entity_offset: 13620, length: 6 }, // &udhar;
    HTMLEntityTableEntry { first_value: 0x0297E, second_value: 0x0000, entity_offset: 13626, length: 7 }, // &ufisht;
    HTMLEntityTableEntry { first_value: 0x1D532, second_value: 0x0000, entity_offset: 13633, length: 4 }, // &ufr;
    HTMLEntityTableEntry { first_value: 0x000F9, second_value: 0x0000, entity_offset: 13637, length: 6 }, // &ugrave
    HTMLEntityTableEntry { first_value: 0x000F9, second_value: 0x0000, entity_offset: 13637, length: 7 }, // &ugrave;
    HTMLEntityTableEntry { first_value: 0x021BF, second_value: 0x0000, entity_offset: 13644, length: 6 }, // &uharl;
    HTMLEntityTableEntry { first_value: 0x021BE, second_value: 0x0000, entity_offset: 13650, length: 6 }, // &uharr;
    HTMLEntityTableEntry { first_value: 0x02580, second_value: 0x0000, entity_offset: 13656, length: 6 }, // &uhblk;
    HTMLEntityTableEntry { first_value: 0x0231C, second_value: 0x0000, entity_offset: 13662, length: 7 }, // &ulcorn;
    HTMLEntityTableEntry { first_value: 0x0231C, second_value: 0x0000, entity_offset: 13669, length: 9 }, // &ulcorner;
    HTMLEntityTableEntry { first_value: 0x0230F, second_value: 0x0000, entity_offset: 13678, length: 7 }, // &ulcrop;
    HTMLEntityTableEntry { first_value: 0x025F8, second_value: 0x0000, entity_offset: 13685, length: 6 }, // &ultri;
    HTMLEntityTableEntry { first_value: 0x0016B, second_value: 0x0000, entity_offset: 13691, length: 6 }, // &umacr;
    HTMLEntityTableEntry { first_value: 0x000A8, second_value: 0x0000, entity_offset: 112, length: 3 }, // &uml
    HTMLEntityTableEntry { first_value: 0x000A8, second_value: 0x0000, entity_offset: 112, length: 4 }, // &uml;
    HTMLEntityTableEntry { first_value: 0x00173, second_value: 0x0000, entity_offset: 13697, length: 6 }, // &uogon;
    HTMLEntityTableEntry { first_value: 0x1D566, second_value: 0x0000, entity_offset: 13703, length: 5 }, // &uopf;
    HTMLEntityTableEntry { first_value: 0x02191, second_value: 0x0000, entity_offset: 13708, length: 8 }, // &uparrow;
    HTMLEntityTableEntry { first_value: 0x02195, second_value: 0x0000, entity_offset: 13716, length: 12 }, // &updownarrow;
    HTMLEntityTableEntry { first_value: 0x021BF, second_value: 0x0000, entity_offset: 13728, length: 14 }, // &upharpoonleft;
    HTMLEntityTableEntry { first_value: 0x021BE, second_value: 0x0000, entity_offset: 13742, length: 15 }, // &upharpoonright;
    HTMLEntityTableEntry { first_value: 0x0228E, second_value: 0x0000, entity_offset: 5702, length: 6 }, // &uplus;
    HTMLEntityTableEntry { first_value: 0x003C5, second_value: 0x0000, entity_offset: 13757, length: 5 }, // &upsi;
    HTMLEntityTableEntry { first_value: 0x003D2, second_value: 0x0000, entity_offset: 13762, length: 6 }, // &upsih;
    HTMLEntityTableEntry { first_value: 0x003C5, second_value: 0x0000, entity_offset: 13768, length: 8 }, // &upsilon;
// cpp: html/parser/html_entity_table.cc:4156-4187
    HTMLEntityTableEntry { first_value: 0x021C8, second_value: 0x0000, entity_offset: 13776, length: 11 }, // &upuparrows;
    HTMLEntityTableEntry { first_value: 0x0231D, second_value: 0x0000, entity_offset: 13787, length: 7 }, // &urcorn;
    HTMLEntityTableEntry { first_value: 0x0231D, second_value: 0x0000, entity_offset: 13794, length: 9 }, // &urcorner;
    HTMLEntityTableEntry { first_value: 0x0230E, second_value: 0x0000, entity_offset: 13803, length: 7 }, // &urcrop;
    HTMLEntityTableEntry { first_value: 0x0016F, second_value: 0x0000, entity_offset: 13810, length: 6 }, // &uring;
    HTMLEntityTableEntry { first_value: 0x025F9, second_value: 0x0000, entity_offset: 13816, length: 6 }, // &urtri;
    HTMLEntityTableEntry { first_value: 0x1D4CA, second_value: 0x0000, entity_offset: 13822, length: 5 }, // &uscr;
    HTMLEntityTableEntry { first_value: 0x022F0, second_value: 0x0000, entity_offset: 13827, length: 6 }, // &utdot;
    HTMLEntityTableEntry { first_value: 0x00169, second_value: 0x0000, entity_offset: 13833, length: 7 }, // &utilde;
    HTMLEntityTableEntry { first_value: 0x025B5, second_value: 0x0000, entity_offset: 13840, length: 5 }, // &utri;
    HTMLEntityTableEntry { first_value: 0x025B4, second_value: 0x0000, entity_offset: 13845, length: 6 }, // &utrif;
    HTMLEntityTableEntry { first_value: 0x021C8, second_value: 0x0000, entity_offset: 13851, length: 6 }, // &uuarr;
    HTMLEntityTableEntry { first_value: 0x000FC, second_value: 0x0000, entity_offset: 13857, length: 4 }, // &uuml
    HTMLEntityTableEntry { first_value: 0x000FC, second_value: 0x0000, entity_offset: 13857, length: 5 }, // &uuml;
    HTMLEntityTableEntry { first_value: 0x029A7, second_value: 0x0000, entity_offset: 13862, length: 8 }, // &uwangle;
    HTMLEntityTableEntry { first_value: 0x021D5, second_value: 0x0000, entity_offset: 13870, length: 5 }, // &vArr;
    HTMLEntityTableEntry { first_value: 0x02AE8, second_value: 0x0000, entity_offset: 13875, length: 5 }, // &vBar;
    HTMLEntityTableEntry { first_value: 0x02AE9, second_value: 0x0000, entity_offset: 13880, length: 6 }, // &vBarv;
    HTMLEntityTableEntry { first_value: 0x022A8, second_value: 0x0000, entity_offset: 10966, length: 6 }, // &vDash;
    HTMLEntityTableEntry { first_value: 0x0299C, second_value: 0x0000, entity_offset: 13886, length: 7 }, // &vangrt;
    HTMLEntityTableEntry { first_value: 0x003F5, second_value: 0x0000, entity_offset: 13893, length: 11 }, // &varepsilon;
    HTMLEntityTableEntry { first_value: 0x003F0, second_value: 0x0000, entity_offset: 13904, length: 9 }, // &varkappa;
    HTMLEntityTableEntry { first_value: 0x02205, second_value: 0x0000, entity_offset: 13913, length: 11 }, // &varnothing;
    HTMLEntityTableEntry { first_value: 0x003D5, second_value: 0x0000, entity_offset: 13924, length: 7 }, // &varphi;
    HTMLEntityTableEntry { first_value: 0x003D6, second_value: 0x0000, entity_offset: 13931, length: 6 }, // &varpi;
    HTMLEntityTableEntry { first_value: 0x0221D, second_value: 0x0000, entity_offset: 13937, length: 10 }, // &varpropto;
    HTMLEntityTableEntry { first_value: 0x02195, second_value: 0x0000, entity_offset: 13947, length: 5 }, // &varr;
    HTMLEntityTableEntry { first_value: 0x003F1, second_value: 0x0000, entity_offset: 13952, length: 7 }, // &varrho;
    HTMLEntityTableEntry { first_value: 0x003C2, second_value: 0x0000, entity_offset: 13959, length: 9 }, // &varsigma;
    HTMLEntityTableEntry { first_value: 0x0228A, second_value: 0xFE00, entity_offset: 13968, length: 13 }, // &varsubsetneq;
    HTMLEntityTableEntry { first_value: 0x02ACB, second_value: 0xFE00, entity_offset: 13981, length: 14 }, // &varsubsetneqq;
    HTMLEntityTableEntry { first_value: 0x0228B, second_value: 0xFE00, entity_offset: 13995, length: 13 }, // &varsupsetneq;
// cpp: html/parser/html_entity_table.cc:4188-4219
    HTMLEntityTableEntry { first_value: 0x02ACC, second_value: 0xFE00, entity_offset: 14008, length: 14 }, // &varsupsetneqq;
    HTMLEntityTableEntry { first_value: 0x003D1, second_value: 0x0000, entity_offset: 14022, length: 9 }, // &vartheta;
    HTMLEntityTableEntry { first_value: 0x022B2, second_value: 0x0000, entity_offset: 14031, length: 16 }, // &vartriangleleft;
    HTMLEntityTableEntry { first_value: 0x022B3, second_value: 0x0000, entity_offset: 14047, length: 17 }, // &vartriangleright;
    HTMLEntityTableEntry { first_value: 0x00432, second_value: 0x0000, entity_offset: 14064, length: 4 }, // &vcy;
    HTMLEntityTableEntry { first_value: 0x022A2, second_value: 0x0000, entity_offset: 4926, length: 6 }, // &vdash;
    HTMLEntityTableEntry { first_value: 0x02228, second_value: 0x0000, entity_offset: 5495, length: 4 }, // &vee;
    HTMLEntityTableEntry { first_value: 0x022BB, second_value: 0x0000, entity_offset: 14068, length: 7 }, // &veebar;
    HTMLEntityTableEntry { first_value: 0x0225A, second_value: 0x0000, entity_offset: 14075, length: 6 }, // &veeeq;
    HTMLEntityTableEntry { first_value: 0x022EE, second_value: 0x0000, entity_offset: 14081, length: 7 }, // &vellip;
    HTMLEntityTableEntry { first_value: 0x0007C, second_value: 0x0000, entity_offset: 14088, length: 7 }, // &verbar;
    HTMLEntityTableEntry { first_value: 0x0007C, second_value: 0x0000, entity_offset: 14095, length: 5 }, // &vert;
    HTMLEntityTableEntry { first_value: 0x1D533, second_value: 0x0000, entity_offset: 14100, length: 4 }, // &vfr;
    HTMLEntityTableEntry { first_value: 0x022B2, second_value: 0x0000, entity_offset: 14104, length: 6 }, // &vltri;
    HTMLEntityTableEntry { first_value: 0x02282, second_value: 0x20D2, entity_offset: 14110, length: 6 }, // &vnsub;
    HTMLEntityTableEntry { first_value: 0x02283, second_value: 0x20D2, entity_offset: 14116, length: 6 }, // &vnsup;
    HTMLEntityTableEntry { first_value: 0x1D567, second_value: 0x0000, entity_offset: 14122, length: 5 }, // &vopf;
    HTMLEntityTableEntry { first_value: 0x0221D, second_value: 0x0000, entity_offset: 14127, length: 6 }, // &vprop;
    HTMLEntityTableEntry { first_value: 0x022B3, second_value: 0x0000, entity_offset: 14133, length: 6 }, // &vrtri;
    HTMLEntityTableEntry { first_value: 0x1D4CB, second_value: 0x0000, entity_offset: 14139, length: 5 }, // &vscr;
    HTMLEntityTableEntry { first_value: 0x02ACB, second_value: 0xFE00, entity_offset: 14144, length: 7 }, // &vsubnE;
    HTMLEntityTableEntry { first_value: 0x0228A, second_value: 0xFE00, entity_offset: 14151, length: 7 }, // &vsubne;
    HTMLEntityTableEntry { first_value: 0x02ACC, second_value: 0xFE00, entity_offset: 14158, length: 7 }, // &vsupnE;
    HTMLEntityTableEntry { first_value: 0x0228B, second_value: 0xFE00, entity_offset: 14165, length: 7 }, // &vsupne;
    HTMLEntityTableEntry { first_value: 0x0299A, second_value: 0x0000, entity_offset: 14172, length: 8 }, // &vzigzag;
    HTMLEntityTableEntry { first_value: 0x00175, second_value: 0x0000, entity_offset: 14180, length: 6 }, // &wcirc;
    HTMLEntityTableEntry { first_value: 0x02A5F, second_value: 0x0000, entity_offset: 14186, length: 7 }, // &wedbar;
    HTMLEntityTableEntry { first_value: 0x02227, second_value: 0x0000, entity_offset: 5509, length: 6 }, // &wedge;
    HTMLEntityTableEntry { first_value: 0x02259, second_value: 0x0000, entity_offset: 14193, length: 7 }, // &wedgeq;
    HTMLEntityTableEntry { first_value: 0x02118, second_value: 0x0000, entity_offset: 14200, length: 7 }, // &weierp;
    HTMLEntityTableEntry { first_value: 0x1D534, second_value: 0x0000, entity_offset: 14207, length: 4 }, // &wfr;
    HTMLEntityTableEntry { first_value: 0x1D568, second_value: 0x0000, entity_offset: 14211, length: 5 }, // &wopf;
// cpp: html/parser/html_entity_table.cc:4220-4251
    HTMLEntityTableEntry { first_value: 0x02118, second_value: 0x0000, entity_offset: 14216, length: 3 }, // &wp;
    HTMLEntityTableEntry { first_value: 0x02240, second_value: 0x0000, entity_offset: 14219, length: 3 }, // &wr;
    HTMLEntityTableEntry { first_value: 0x02240, second_value: 0x0000, entity_offset: 14222, length: 7 }, // &wreath;
    HTMLEntityTableEntry { first_value: 0x1D4CC, second_value: 0x0000, entity_offset: 14229, length: 5 }, // &wscr;
    HTMLEntityTableEntry { first_value: 0x022C2, second_value: 0x0000, entity_offset: 14234, length: 5 }, // &xcap;
    HTMLEntityTableEntry { first_value: 0x025EF, second_value: 0x0000, entity_offset: 14239, length: 6 }, // &xcirc;
    HTMLEntityTableEntry { first_value: 0x022C3, second_value: 0x0000, entity_offset: 14245, length: 5 }, // &xcup;
    HTMLEntityTableEntry { first_value: 0x025BD, second_value: 0x0000, entity_offset: 14250, length: 6 }, // &xdtri;
    HTMLEntityTableEntry { first_value: 0x1D535, second_value: 0x0000, entity_offset: 14256, length: 4 }, // &xfr;
    HTMLEntityTableEntry { first_value: 0x027FA, second_value: 0x0000, entity_offset: 14260, length: 6 }, // &xhArr;
    HTMLEntityTableEntry { first_value: 0x027F7, second_value: 0x0000, entity_offset: 14266, length: 6 }, // &xharr;
    HTMLEntityTableEntry { first_value: 0x003BE, second_value: 0x0000, entity_offset: 14272, length: 3 }, // &xi;
    HTMLEntityTableEntry { first_value: 0x027F8, second_value: 0x0000, entity_offset: 14275, length: 6 }, // &xlArr;
    HTMLEntityTableEntry { first_value: 0x027F5, second_value: 0x0000, entity_offset: 14281, length: 6 }, // &xlarr;
    HTMLEntityTableEntry { first_value: 0x027FC, second_value: 0x0000, entity_offset: 14287, length: 5 }, // &xmap;
    HTMLEntityTableEntry { first_value: 0x022FB, second_value: 0x0000, entity_offset: 14292, length: 5 }, // &xnis;
    HTMLEntityTableEntry { first_value: 0x02A00, second_value: 0x0000, entity_offset: 14297, length: 6 }, // &xodot;
    HTMLEntityTableEntry { first_value: 0x1D569, second_value: 0x0000, entity_offset: 14303, length: 5 }, // &xopf;
    HTMLEntityTableEntry { first_value: 0x02A01, second_value: 0x0000, entity_offset: 14308, length: 7 }, // &xoplus;
    HTMLEntityTableEntry { first_value: 0x02A02, second_value: 0x0000, entity_offset: 14315, length: 7 }, // &xotime;
    HTMLEntityTableEntry { first_value: 0x027F9, second_value: 0x0000, entity_offset: 14322, length: 6 }, // &xrArr;
    HTMLEntityTableEntry { first_value: 0x027F6, second_value: 0x0000, entity_offset: 14328, length: 6 }, // &xrarr;
    HTMLEntityTableEntry { first_value: 0x1D4CD, second_value: 0x0000, entity_offset: 14334, length: 5 }, // &xscr;
    HTMLEntityTableEntry { first_value: 0x02A06, second_value: 0x0000, entity_offset: 14339, length: 7 }, // &xsqcup;
    HTMLEntityTableEntry { first_value: 0x02A04, second_value: 0x0000, entity_offset: 14346, length: 7 }, // &xuplus;
    HTMLEntityTableEntry { first_value: 0x025B3, second_value: 0x0000, entity_offset: 14353, length: 6 }, // &xutri;
    HTMLEntityTableEntry { first_value: 0x022C1, second_value: 0x0000, entity_offset: 14359, length: 5 }, // &xvee;
    HTMLEntityTableEntry { first_value: 0x022C0, second_value: 0x0000, entity_offset: 14364, length: 7 }, // &xwedge;
    HTMLEntityTableEntry { first_value: 0x000FD, second_value: 0x0000, entity_offset: 14371, length: 6 }, // &yacute
    HTMLEntityTableEntry { first_value: 0x000FD, second_value: 0x0000, entity_offset: 14371, length: 7 }, // &yacute;
    HTMLEntityTableEntry { first_value: 0x0044F, second_value: 0x0000, entity_offset: 14378, length: 5 }, // &yacy;
    HTMLEntityTableEntry { first_value: 0x00177, second_value: 0x0000, entity_offset: 14383, length: 6 }, // &ycirc;
// cpp: html/parser/html_entity_table.cc:4252-4274
    HTMLEntityTableEntry { first_value: 0x0044B, second_value: 0x0000, entity_offset: 14389, length: 4 }, // &ycy;
    HTMLEntityTableEntry { first_value: 0x000A5, second_value: 0x0000, entity_offset: 14393, length: 3 }, // &yen
    HTMLEntityTableEntry { first_value: 0x000A5, second_value: 0x0000, entity_offset: 14393, length: 4 }, // &yen;
    HTMLEntityTableEntry { first_value: 0x1D536, second_value: 0x0000, entity_offset: 14397, length: 4 }, // &yfr;
    HTMLEntityTableEntry { first_value: 0x00457, second_value: 0x0000, entity_offset: 14401, length: 5 }, // &yicy;
    HTMLEntityTableEntry { first_value: 0x1D56A, second_value: 0x0000, entity_offset: 14406, length: 5 }, // &yopf;
    HTMLEntityTableEntry { first_value: 0x1D4CE, second_value: 0x0000, entity_offset: 14411, length: 5 }, // &yscr;
    HTMLEntityTableEntry { first_value: 0x0044E, second_value: 0x0000, entity_offset: 14416, length: 5 }, // &yucy;
    HTMLEntityTableEntry { first_value: 0x000FF, second_value: 0x0000, entity_offset: 14421, length: 4 }, // &yuml
    HTMLEntityTableEntry { first_value: 0x000FF, second_value: 0x0000, entity_offset: 14421, length: 5 }, // &yuml;
    HTMLEntityTableEntry { first_value: 0x0017A, second_value: 0x0000, entity_offset: 14426, length: 7 }, // &zacute;
    HTMLEntityTableEntry { first_value: 0x0017E, second_value: 0x0000, entity_offset: 14433, length: 7 }, // &zcaron;
    HTMLEntityTableEntry { first_value: 0x00437, second_value: 0x0000, entity_offset: 7321, length: 4 }, // &zcy;
    HTMLEntityTableEntry { first_value: 0x0017C, second_value: 0x0000, entity_offset: 14440, length: 5 }, // &zdot;
    HTMLEntityTableEntry { first_value: 0x02128, second_value: 0x0000, entity_offset: 14445, length: 7 }, // &zeetrf;
    HTMLEntityTableEntry { first_value: 0x003B6, second_value: 0x0000, entity_offset: 14452, length: 5 }, // &zeta;
    HTMLEntityTableEntry { first_value: 0x1D537, second_value: 0x0000, entity_offset: 14457, length: 4 }, // &zfr;
    HTMLEntityTableEntry { first_value: 0x00436, second_value: 0x0000, entity_offset: 14461, length: 5 }, // &zhcy;
    HTMLEntityTableEntry { first_value: 0x021DD, second_value: 0x0000, entity_offset: 7326, length: 8 }, // &zigrarr;
    HTMLEntityTableEntry { first_value: 0x1D56B, second_value: 0x0000, entity_offset: 14466, length: 5 }, // &zopf;
    HTMLEntityTableEntry { first_value: 0x1D4CF, second_value: 0x0000, entity_offset: 14471, length: 5 }, // &zscr;
    HTMLEntityTableEntry { first_value: 0x0200D, second_value: 0x0000, entity_offset: 14476, length: 4 }, // &zwj;
    HTMLEntityTableEntry { first_value: 0x0200C, second_value: 0x0000, entity_offset: 14480, length: 5 }, // &zwnj;
];

// cpp: html/parser/html_entity_table.cc:4282-4282
#[rustfmt::skip]
const kUppercaseOffset: [u16; 27] = [
// cpp: html/parser/html_entity_table.cc:4283-4309
    0,
    27,
    39,
    75,
    129,
    159,
    167,
    189,
    201,
    230,
    237,
    245,
    305,
    314,
    386,
    415,
    434,
    439,
    484,
    524,
    547,
    587,
    604,
    609,
    613,
    624,
    634,
];

// cpp: html/parser/html_entity_table.cc:4314-4314
#[rustfmt::skip]
const kLowercaseOffset: [u16; 27] = [
// cpp: html/parser/html_entity_table.cc:4315-4341
    634,
    703,
    819,
    918,
    984,
    1051,
    1090,
    1150,
    1178,
    1234,
    1242,
    1252,
    1406,
    1446,
    1614,
    1675,
    1744,
    1755,
    1859,
    2017,
    2075,
    2127,
    2169,
    2180,
    2204,
    2218,
    2231,
];

impl HTMLEntityTable {
    // cpp: html/parser/html_entity_table.h:53
    // cpp: html/parser/html_entity_table.cc:4346-4348
    pub fn EntityString(entry: &HTMLEntityTableEntry) -> &'static [u8] {
        &kStaticEntityStringStorage
            [entry.entity_offset as usize..(entry.entity_offset + entry.length) as usize]
    }

    // cpp: html/parser/html_entity_table.h:51
    // cpp: html/parser/html_entity_table.cc:4354-4366
    pub fn EntriesStartingWith(c: UChar) -> &'static [HTMLEntityTableEntry] {
        if (b'A' as UChar..=b'Z' as UChar).contains(&c) {
            let index = (c - b'A' as UChar) as usize;
            let first = kUppercaseOffset[index] as usize;
            let last = kUppercaseOffset[index + 1] as usize;
            return &Self::AllEntries()[first..last];
        }
        if (b'a' as UChar..=b'z' as UChar).contains(&c) {
            let index = (c - b'a' as UChar) as usize;
            let first = kLowercaseOffset[index] as usize;
            let last = kLowercaseOffset[index + 1] as usize;
            return &Self::AllEntries()[first..last];
        }
        &[]
    }

    // cpp: html/parser/html_entity_table.h:50
    // cpp: html/parser/html_entity_table.cc:4368-4370
    pub fn AllEntries() -> &'static [HTMLEntityTableEntry] {
        &kStaticEntityTable
    }
}

impl HTMLEntityTableEntry {
    // cpp: html/parser/html_entity_table.h:38
    // cpp: html/parser/html_entity_table.cc:4350-4352
    pub fn LastCharacter(&self) -> u8 {
        *HTMLEntityTable::EntityString(self)
            .last()
            .expect("entity name is nonempty")
    }
}
