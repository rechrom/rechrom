use std::io;
use url_loader::URLResponse;

// cpp: browser/browser.cc:192-203
fn AppendUTF8(code_point: u32, output: &mut String) {
    output.push(char::from_u32(code_point).expect("Windows-1252 code point is valid"));
}

// cpp: browser/browser.cc:205-219
fn DecodeWindows1252(bytes: &[u8]) -> String {
    const C1_CODE_POINTS: [u16; 32] = [
        0x20ac, 0xfffd, 0x201a, 0x0192, 0x201e, 0x2026, 0x2020, 0x2021, 0x02c6, 0x2030, 0x0160,
        0x2039, 0x0152, 0xfffd, 0x017d, 0xfffd, 0xfffd, 0x2018, 0x2019, 0x201c, 0x201d, 0x2022,
        0x2013, 0x2014, 0x02dc, 0x2122, 0x0161, 0x203a, 0x0153, 0xfffd, 0x017e, 0x0178,
    ];
    let mut decoded = String::with_capacity(bytes.len());
    for &byte in bytes {
        let code_point = if (0x80..=0x9f).contains(&byte) {
            u32::from(C1_CODE_POINTS[usize::from(byte - 0x80)])
        } else {
            u32::from(byte)
        };
        AppendUTF8(code_point, &mut decoded);
    }
    decoded
}

// cpp: browser/browser.cc:221-238
pub fn DecodeText(response: &URLResponse) -> io::Result<String> {
    let encoding = response.text_encoding.to_ascii_lowercase();
    if matches!(
        encoding.as_str(),
        "iso-8859-1" | "latin1" | "latin-1" | "windows-1252" | "cp1252" | "us-ascii"
    ) {
        return Ok(DecodeWindows1252(&response.body));
    }
    if !encoding.is_empty() && encoding != "utf-8" && encoding != "utf8" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unsupported text encoding: {}", response.text_encoding),
        ));
    }
    let offset = usize::from(response.body.starts_with(&[0xef, 0xbb, 0xbf])) * 3;
    if offset == response.body.len() {
        return Ok(String::new());
    }
    String::from_utf8(response.body[offset..].to_vec())
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}
