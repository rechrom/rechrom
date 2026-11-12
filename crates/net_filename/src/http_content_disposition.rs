//! Chromium net/http/http_content_disposition.cc:32-470; HttpUtil iterators
//! net/http/http_util.cc:611-664,1064-1207. Byte strings retain HTTP octets.
use crate::UnescapeBinaryURLComponent;
use base64::{
    alphabet,
    engine::general_purpose::{GeneralPurpose, GeneralPurposeConfig},
    Engine,
};
use icu_bidi::filename_unicode::{ConvertToUTF16, ConvertToUtf8, ConvertToUtf8AndNormalize};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DispositionType {
    #[default]
    Inline,
    Attachment,
}
#[derive(Clone, Debug, Default)]
pub struct HttpContentDisposition {
    pub disposition_type: DispositionType,
    pub filename: String,
    pub parse_result_flags: u32,
}
pub const HAS_DISPOSITION_TYPE: u32 = 1;
pub const HAS_UNKNOWN_DISPOSITION_TYPE: u32 = 2;
pub const HAS_FILENAME: u32 = 4;
pub const HAS_EXT_FILENAME: u32 = 8;
pub const HAS_NON_ASCII_STRINGS: u32 = 16;
pub const HAS_PERCENT_ENCODED_STRINGS: u32 = 32;
pub const HAS_RFC2047_ENCODED_STRINGS: u32 = 64;
pub const HAS_SINGLE_QUOTED_FILENAME: u32 = 128;
fn trim_lws(mut s: &[u8]) -> &[u8] {
    while s.first().is_some_and(|b| matches!(b, b' ' | b'\t')) {
        s = &s[1..];
    }
    while s.last().is_some_and(|b| matches!(b, b' ' | b'\t')) {
        s = &s[..s.len() - 1];
    }
    s
}
/// StringViewTokenizer's quote/escape state, with only double quotes enabled.
fn split_quoted(input: &[u8], delimiter: u8) -> Vec<&[u8]> {
    let (mut start, mut quoted, mut escaped) = (0, false, false);
    let mut result = Vec::new();
    for (i, &b) in input.iter().enumerate() {
        if quoted {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                quoted = false;
            }
        } else if b == delimiter {
            result.push(&input[start..i]);
            start = i + 1;
        } else if b == b'"' {
            quoted = true;
        }
    }
    result.push(&input[start..]);
    result
}
fn is_token(input: &[u8]) -> bool {
    !input.is_empty()
        && input
            .iter()
            .all(|&b| b > 32 && b < 127 && !b"()<>@,;:\\\"/[]?={}".contains(&b))
}
fn unquote(input: &[u8]) -> Vec<u8> {
    if input.len() < 2 || input[0] != b'"' || input.last() != Some(&b'"') {
        return input.to_vec();
    }
    let mut out = Vec::new();
    let mut escaped = false;
    for &b in &input[1..input.len() - 1] {
        if b == b'\\' && !escaped {
            escaped = true;
            continue;
        }
        escaped = false;
        out.push(b);
    }
    out
}
fn decode_q(input: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < input.len() {
        match input[i] {
            b'_' => out.push(b' '),
            b'=' => {
                let high = (*input.get(i + 1)? as char).to_digit(16)?;
                let low = (*input.get(i + 2)? as char).to_digit(16)?;
                out.push((high * 16 + low) as u8);
                i += 2;
            }
            b if b > 32 && b < 127 && b != b'?' => out.push(b),
            _ => return None,
        }
        i += 1;
    }
    Some(out)
}
fn decode_word(input: &[u8], referrer_charset: &[u8], flags: &mut u32) -> Option<(String, bool)> {
    if input.is_empty() {
        return Some((String::new(), false));
    }
    if !input.is_ascii() {
        let output = if let Ok(value) = std::str::from_utf8(input) {
            value.to_owned()
        } else if !referrer_charset.is_empty() {
            ConvertToUTF16(input, referrer_charset)
                .and_then(|v| String::from_utf16(&v).ok())
                .unwrap_or_default()
        } else {
            String::new()
        };
        // Apple SysNativeMBToWide is strict UTF-8. If the referrer conversion
        // fails, the same invalid input produces empty output (not Latin-1).
        *flags |= HAS_NON_ASCII_STRINGS;
        return Some((output, false));
    }
    let parts: Vec<_> = input
        .split(|&b| b == b'?')
        .filter(|p| !p.is_empty())
        .collect();
    let mut rfc2047 = true;
    let mut decoded = String::new();
    let mut part_index = 0;
    let mut charset = &b""[..];
    let mut base64_encoding = false;
    for part in parts {
        match part_index {
            0 => {
                if part != b"=" {
                    rfc2047 = false;
                    break;
                }
                part_index += 1;
            }
            1 => {
                charset = part;
                part_index += 1;
            }
            2 => {
                if part.len() != 1 || !b"bBqQ".contains(&part[0]) {
                    rfc2047 = false;
                    break;
                }
                base64_encoding = matches!(part[0], b'b' | b'B');
                part_index += 1;
            }
            3 => {
                let bytes = if base64_encoding {
                    GeneralPurpose::new(
                        &alphabet::STANDARD,
                        GeneralPurposeConfig::new().with_decode_allow_trailing_bits(true),
                    )
                    .decode(part)
                    .ok()?
                } else {
                    decode_q(part)?
                };
                decoded = if bytes.is_empty() {
                    String::new()
                } else {
                    ConvertToUtf8(&bytes, charset)?
                };
                part_index += 1;
            }
            4 => {
                if part != b"=" {
                    return None;
                }
                part_index += 1;
            }
            _ => return None,
        }
    }
    if rfc2047 {
        if input.last() == Some(&b'=') {
            *flags |= HAS_RFC2047_ENCODED_STRINGS;
            return Some((decoded, true));
        }
        return None;
    }
    let decoded = UnescapeBinaryURLComponent(input);
    if decoded != input {
        *flags |= HAS_PERCENT_ENCODED_STRINGS;
    }
    Some((String::from_utf8(decoded).ok()?, false))
}
fn decode_filename(input: &[u8], charset: &[u8]) -> Option<(String, u32)> {
    let mut result = String::new();
    let mut previous_rfc2047 = true;
    let mut flags = 0;
    let mut i = 0;
    while i < input.len() {
        if b" \t\n\r".contains(&input[i]) {
            if !previous_rfc2047 {
                result.push(' ');
            }
            i += 1;
            continue;
        }
        let start = i;
        while i < input.len() && !b" \t\n\r".contains(&input[i]) {
            i += 1;
        }
        let (value, rfc) = decode_word(&input[start..i], charset, &mut flags)?;
        result.push_str(&value);
        previous_rfc2047 = rfc;
    }
    Some((result, flags))
}
fn decode_ext(input: &[u8]) -> Option<String> {
    if input.contains(&b'"') {
        return None;
    }
    let parts: Vec<_> = input.split(|&b| b == b'\'').collect();
    if parts.len() != 3 || parts[0].is_empty() || parts[2].is_empty() {
        return None;
    }
    if !parts[2].is_ascii() {
        return Some(String::new());
    }
    ConvertToUtf8AndNormalize(&UnescapeBinaryURLComponent(parts[2]), parts[0])
}
impl HttpContentDisposition {
    pub fn Parse(header: &[u8], referrer_charset: &[u8]) -> Self {
        let mut result = Self::default();
        // The string constructor reads only the first comma-separated value,
        // including an empty one. A quoted comma does not separate values.
        let values = split_quoted(header, b',');
        let header = trim_lws(values[0]);
        let delimiter = header
            .iter()
            .position(|&b| b == b';')
            .unwrap_or(header.len());
        let disposition = trim_lws(&header[..delimiter]);
        let params = if is_token(disposition) {
            result.parse_result_flags |= HAS_DISPOSITION_TYPE;
            if !disposition.eq_ignore_ascii_case(b"inline") {
                result.disposition_type = DispositionType::Attachment;
                if !disposition.eq_ignore_ascii_case(b"attachment") {
                    result.parse_result_flags |= HAS_UNKNOWN_DISPOSITION_TYPE;
                }
            }
            header.get(delimiter + 1..).unwrap_or_default()
        } else {
            header
        };
        let mut filename = String::new();
        let mut ext_filename = String::new();
        for parameter in split_quoted(params, b';') {
            let parameter = trim_lws(parameter);
            if parameter.is_empty() {
                continue;
            }
            let Some(equals) = parameter.iter().position(|&b| b == b'=') else {
                break;
            };
            if equals == 0 {
                break;
            }
            let name = trim_lws(&parameter[..equals]);
            if name.contains(&b'"') {
                break;
            }
            let raw = trim_lws(&parameter[equals + 1..]);
            if raw.is_empty() {
                break;
            }
            let value = if raw.first() == Some(&b'"') {
                if raw.len() == 1 || raw.last() != Some(&b'"') {
                    raw[1..].to_vec()
                } else {
                    unquote(raw)
                }
            } else {
                raw.to_vec()
            };
            if filename.is_empty() && name.eq_ignore_ascii_case(b"filename") {
                if let Some((decoded, flags)) = decode_filename(&value, referrer_charset) {
                    filename = decoded;
                    if !filename.is_empty() {
                        result.parse_result_flags |= flags | HAS_FILENAME;
                        if filename.starts_with('\'') {
                            result.parse_result_flags |= HAS_SINGLE_QUOTED_FILENAME;
                        }
                    }
                }
            } else if ext_filename.is_empty() && name.eq_ignore_ascii_case(b"filename*") {
                if let Some(decoded) = decode_ext(raw) {
                    ext_filename = decoded;
                    if !ext_filename.is_empty() {
                        result.parse_result_flags |= HAS_EXT_FILENAME;
                    }
                }
            }
        }
        if filename.starts_with('\'') {
            result.parse_result_flags |= HAS_SINGLE_QUOTED_FILENAME;
        }
        result.filename = if ext_filename.is_empty() {
            filename
        } else {
            ext_filename
        };
        result
    }
}
