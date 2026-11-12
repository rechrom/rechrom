//! KURL EncodeWithUrlEscapeSequences, kurl.cc:786-793, and
//! url/url_canon_internal.h CHAR_COMPONENT, fixed Chromium revision
//! 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
pub fn EncodeWithUrlEscapeSequences(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut result = String::with_capacity(value.len());
    for byte in value.bytes() {
        // UriComponentEncoder's source set deliberately encodes apostrophes.
        // KURL then unescapes '/' for readability.
        if byte.is_ascii_alphanumeric()
            || matches!(
                byte,
                b'!' | b'(' | b')' | b'*' | b'-' | b'.' | b'_' | b'~' | b'/'
            )
        {
            result.push(byte as char);
        } else {
            result.push('%');
            result.push(HEX[(byte >> 4) as usize] as char);
            result.push(HEX[(byte & 15) as usize] as char);
        }
    }
    result
}

// DecodeUrlEscapeSequences(kUtf8OrIsomorphic), kurl.cc:749-769. KURL's path
// component remains percent-encoded until this File name projection.
pub fn DecodeUrlEscapeSequencesUtf8OrIsomorphic(value: &str) -> String {
    fn hex(c: u8) -> Option<u8> {
        match c {
            b'0'..=b'9' => Some(c - b'0'),
            b'a'..=b'f' => Some(c - b'a' + 10),
            b'A'..=b'F' => Some(c - b'A' + 10),
            _ => None,
        }
    }
    let raw = value.as_bytes();
    let mut result = Vec::with_capacity(raw.len());
    let mut i = 0;
    while i < raw.len() {
        if raw[i] == b'%' && i + 2 < raw.len() {
            if let (Some(a), Some(b)) = (hex(raw[i + 1]), hex(raw[i + 2])) {
                result.push(a * 16 + b);
                i += 3;
                continue;
            }
        }
        result.push(raw[i]);
        i += 1;
    }
    String::from_utf8(result)
        .unwrap_or_else(|error| error.into_bytes().into_iter().map(char::from).collect())
}

/// KURL::DecodeUrlEscapeSequences(kUtf8), kurl.cc:780-784, and
/// url::HasInvalidUrlEscapeSequences, url_util.cc:937-946.
pub fn HasInvalidUrlEscapeSequences(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            if i + 2 >= bytes.len()
                || !bytes[i + 1].is_ascii_hexdigit()
                || !bytes[i + 2].is_ascii_hexdigit()
            {
                return true;
            }
            i += 3;
        } else {
            i += 1;
        }
    }
    false
}
pub fn DecodeUrlEscapeSequencesUtf8(value: &str) -> String {
    let hex = |byte: u8| -> Option<u8> {
        match byte {
            b'0'..=b'9' => Some(byte - b'0'),
            b'a'..=b'f' => Some(byte - b'a' + 10),
            b'A'..=b'F' => Some(byte - b'A' + 10),
            _ => None,
        }
    };
    let source = value.as_bytes();
    let mut decoded = Vec::with_capacity(source.len());
    let mut i = 0;
    while i < source.len() {
        if source[i] == b'%' && i + 2 < source.len() {
            if let (Some(a), Some(b)) = (hex(source[i + 1]), hex(source[i + 2])) {
                decoded.push(a * 16 + b);
                i += 3;
                continue;
            }
        }
        decoded.push(source[i]);
        i += 1;
    }
    String::from_utf8_lossy(&decoded).into_owned()
}
