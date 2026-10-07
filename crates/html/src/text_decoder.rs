//! Incremental text decoding for the document parser's byte input boundary.
//! Preserve the supported encodings of the existing complete-response path.
use std::io;
pub(crate) struct TextDecoder {
    windows1252: bool,
    prefix_checked: bool,
    pending: Vec<u8>,
}
impl TextDecoder {
    pub(crate) fn new(label: &str) -> io::Result<Self> {
        let label = label.trim().to_ascii_lowercase();
        let windows1252 = matches!(
            label.as_str(),
            "iso-8859-1" | "latin1" | "latin-1" | "windows-1252" | "cp1252" | "us-ascii"
        );
        if !windows1252 && !matches!(label.as_str(), "" | "utf-8" | "utf8") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unsupported text encoding: {label}"),
            ));
        }
        Ok(Self {
            windows1252,
            prefix_checked: windows1252,
            pending: vec![],
        })
    }
    pub(crate) fn append(&mut self, bytes: &[u8], eof: bool) -> io::Result<String> {
        if self.windows1252 {
            const C1: [u16; 32] = [
                0x20ac, 0xfffd, 0x201a, 0x0192, 0x201e, 0x2026, 0x2020, 0x2021, 0x02c6, 0x2030,
                0x0160, 0x2039, 0x0152, 0xfffd, 0x017d, 0xfffd, 0xfffd, 0x2018, 0x2019, 0x201c,
                0x201d, 0x2022, 0x2013, 0x2014, 0x02dc, 0x2122, 0x0161, 0x203a, 0x0153, 0xfffd,
                0x017e, 0x0178,
            ];
            return Ok(bytes
                .iter()
                .map(|&b| {
                    char::from_u32(if (0x80..=0x9f).contains(&b) {
                        u32::from(C1[usize::from(b - 0x80)])
                    } else {
                        u32::from(b)
                    })
                    .unwrap()
                })
                .collect());
        }
        self.pending.extend_from_slice(bytes);
        if !self.prefix_checked {
            let bom = [0xef, 0xbb, 0xbf];
            if self.pending.len() < 3 && bom.starts_with(&self.pending) && !eof {
                return Ok(String::new());
            }
            if self.pending.starts_with(&bom) {
                self.pending.drain(..3);
            }
            self.prefix_checked = true;
        }
        match std::str::from_utf8(&self.pending) {
            Ok(text) => {
                let result = text.to_owned();
                self.pending.clear();
                Ok(result)
            }
            Err(error) if error.error_len().is_none() && !eof => {
                let end = error.valid_up_to();
                let result = std::str::from_utf8(&self.pending[..end])
                    .unwrap()
                    .to_owned();
                self.pending.drain(..end);
                Ok(result)
            }
            Err(error) => Err(io::Error::new(io::ErrorKind::InvalidData, error)),
        }
    }
}
