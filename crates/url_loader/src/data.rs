//! data: URL payload decoding; no document or resource scheduling.
#![allow(non_snake_case)]
use crate::metadata::{invalid, LowerASCII, ParseContentType, PercentDecodeData, Trim};
use crate::{URLRequest, URLResponse};
use std::io;

// cpp: resource_loader/default_resource_loader.cc:137-176
fn DecodeBase64Data(input: &[u8], limit: usize) -> io::Result<Vec<u8>> {
    let value_of = |value: u8| -> i32 {
        match value {
            b'A'..=b'Z' => i32::from(value - b'A'),
            b'a'..=b'z' => i32::from(value - b'a') + 26,
            b'0'..=b'9' => i32::from(value - b'0') + 52,
            b'+' => 62,
            b'/' => 63,
            _ => -1,
        }
    };
    let mut output = Vec::with_capacity(limit.min(input.len().saturating_mul(3) / 4));
    let mut accumulator = 0_u32;
    let mut bits = 0_u32;
    let mut padding = false;
    for &character in input {
        if character.is_ascii_whitespace() {
            continue;
        }
        if character == b'=' {
            padding = true;
            continue;
        }
        if padding {
            return Err(invalid("invalid base64 padding in data URL"));
        }
        let value = value_of(character);
        if value < 0 {
            return Err(invalid("invalid base64 character in data URL"));
        }
        accumulator = (accumulator << 6) | value as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            if output.len() == limit {
                return Err(invalid("URL response exceeds configured size limit"));
            }
            output.push((accumulator >> bits) as u8);
            accumulator &= (1_u32 << bits) - 1;
        }
    }
    if bits >= 6 || accumulator != 0 {
        return Err(invalid("invalid base64 data URL"));
    }
    Ok(output)
}

// cpp: resource_loader/default_resource_loader.cc:178-224
pub(crate) fn DecodeDataURL(request: &URLRequest, limit: usize) -> io::Result<URLResponse> {
    let url = request
        .url
        .strip_prefix("data:")
        .ok_or_else(|| invalid("not a data URL"))?;
    let comma = url
        .find(',')
        .ok_or_else(|| invalid("data URL has no payload separator"))?;
    let mut metadata = url[..comma].to_owned();
    let mut base64 = false;
    let mut token_begin = 0;
    while token_begin <= metadata.len() {
        let token_end = metadata[token_begin..].find(';').map(|n| token_begin + n);
        let token = Trim(&metadata[token_begin..token_end.unwrap_or(metadata.len())]);
        if LowerASCII(&token) == "base64" {
            base64 = true;
            let erase_start = if token_begin == 0 { 0 } else { token_begin - 1 };
            let erase_length = token.len() + usize::from(token_begin != 0);
            metadata.replace_range(
                erase_start..(erase_start + erase_length).min(metadata.len()),
                "",
            );
            break;
        }
        let Some(end) = token_end else {
            break;
        };
        token_begin = end + 1;
    }
    let mut response = URLResponse {
        final_url: request.url.clone(),
        status_code: 200,
        mime_type: "text/plain".to_owned(),
        text_encoding: "us-ascii".to_owned(),
        ..URLResponse::default()
    };
    if !metadata.is_empty() {
        let mut content_type = metadata;
        if content_type.starts_with(';') {
            content_type.insert_str(0, "text/plain");
        }
        ParseContentType(
            &content_type,
            &mut response.mime_type,
            &mut response.text_encoding,
        );
    }
    let encoded = PercentDecodeData(url[comma + 1..].as_bytes());
    response.body = if base64 {
        DecodeBase64Data(&encoded, limit)?
    } else {
        if encoded.len() > limit {
            return Err(invalid("URL response exceeds configured size limit"));
        }
        encoded
    };
    Ok(response)
}
