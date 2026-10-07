//! Metadata and URL byte decoding shared by scheme implementations.
#![allow(non_snake_case)]
use crate::{RequestDestination, URLRequest};
use std::io;

pub(crate) fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}

// cpp: resource_loader/default_resource_loader.cc:22-28
pub(crate) fn LowerASCII(input: &str) -> String {
    input.to_ascii_lowercase()
}

// cpp: resource_loader/default_resource_loader.cc:30-38
pub(crate) fn Trim(input: &str) -> String {
    input
        .trim_matches(|character: char| character.is_ascii_whitespace())
        .to_owned()
}

// cpp: resource_loader/default_resource_loader.cc:40-45
fn HexDigit(value: u8) -> i32 {
    match value {
        b'0'..=b'9' => i32::from(value - b'0'),
        b'a'..=b'f' => i32::from(value - b'a') + 10,
        b'A'..=b'F' => i32::from(value - b'A') + 10,
        _ => -1,
    }
}

// cpp: resource_loader/default_resource_loader.cc:75-88
pub(crate) fn MimeTypeFor(request: &URLRequest) -> &'static str {
    match request.destination {
        RequestDestination::kDocument => "text/html",
        RequestDestination::kStyleSheet => "text/css",
        RequestDestination::kScript => "text/javascript",
        RequestDestination::kImage | RequestDestination::kFetch | RequestDestination::kFont => {
            "application/octet-stream"
        }
    }
}

// cpp: resource_loader/default_resource_loader.cc:90-117
pub(crate) fn ParseContentType(content_type: &str, mime_type: &mut String, encoding: &mut String) {
    let (main, parameters) = content_type.split_once(';').unwrap_or((content_type, ""));
    *mime_type = LowerASCII(&Trim(main));
    for parameter in parameters.split(';') {
        let parameter = Trim(parameter);
        let Some((name, value)) = parameter.split_once('=') else {
            continue;
        };
        if LowerASCII(&Trim(name)) != "charset" {
            continue;
        }
        let mut value = Trim(value);
        if value.len() >= 2
            && ((value.starts_with('"') && value.ends_with('"'))
                || (value.starts_with('\'') && value.ends_with('\'')))
        {
            value = value[1..value.len() - 1].to_owned();
        }
        *encoding = LowerASCII(&value);
    }
}

// cpp: resource_loader/default_resource_loader.cc:119-135
pub(crate) fn PercentDecodeData(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len());
    let mut index = 0;
    while index < input.len() {
        if input[index] == b'%' && index + 2 < input.len() {
            let high = HexDigit(input[index + 1]);
            let low = HexDigit(input[index + 2]);
            if high >= 0 && low >= 0 {
                output.push(((high << 4) | low) as u8);
                index += 3;
                continue;
            }
        }
        output.push(input[index]);
        index += 1;
    }
    output
}
