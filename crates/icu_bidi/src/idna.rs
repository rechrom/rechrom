//! ICU UTS #46 owner used by Chromium email_input_type.cc:92-119 and
//! url_formatter, implemented on the `idna` crate's UTS #46 processing.
//! Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15; ICU uidna.h:203-228,312-315.
#![allow(non_snake_case)]
use idna::uts46::{AsciiDenyList, DnsLength, Hyphens, Uts46};

pub const UIDNA_CHECK_BIDI: u32 = 4;
pub const UIDNA_NONTRANSITIONAL_TO_ASCII: u32 = 0x10;
pub const UIDNA_NONTRANSITIONAL_TO_UNICODE: u32 = 0x20;
/// UIDNA_ERROR_DISALLOWED, reported for any UTS #46 processing error.
const UIDNA_ERROR_GENERIC: u32 = 0x80;
const UIDNA_ERROR_LABEL_HAS_DOT: u32 = 0x200;

pub struct Idna {
    options: u32,
}

pub struct IdnaResult {
    pub value: Vec<u16>,
    pub failed: bool,
    pub errors: u32,
}

impl IdnaResult {
    fn invalid_input() -> Self {
        IdnaResult {
            value: Vec::new(),
            failed: true,
            errors: 0,
        }
    }
}

impl Idna {
    pub fn OpenUTS46(options: u32) -> Result<Self, i32> {
        Ok(Self { options })
    }

    /// Transitional processing (ICU's default without the NONTRANSITIONAL
    /// options) is nontransitional processing after mapping the UTS #46
    /// deviation characters.
    fn prepare(&self, name: &str, nontransitional: bool) -> String {
        if nontransitional {
            return name.to_owned();
        }
        let mut result = String::with_capacity(name.len());
        for character in name.chars() {
            match character {
                '\u{00DF}' => result.push_str("ss"),
                '\u{03C2}' => result.push('\u{03C3}'),
                '\u{200C}' | '\u{200D}' => {}
                other => result.push(other),
            }
        }
        result
    }

    pub fn LabelToUnicode(&self, name: &[u16]) -> IdnaResult {
        let Ok(name) = String::from_utf16(name) else {
            return IdnaResult::invalid_input();
        };
        let name = self.prepare(&name, self.options & UIDNA_NONTRANSITIONAL_TO_UNICODE != 0);
        let (value, result) =
            Uts46::new().to_unicode(name.as_bytes(), AsciiDenyList::EMPTY, Hyphens::Check);
        let mut errors = if result.is_err() { UIDNA_ERROR_GENERIC } else { 0 };
        if value.contains('.') {
            errors |= UIDNA_ERROR_LABEL_HAS_DOT;
        }
        IdnaResult {
            value: if errors == 0 {
                value.encode_utf16().collect()
            } else {
                Vec::new()
            },
            failed: false,
            errors,
        }
    }

    pub fn NameToASCII(&self, name: &[u16]) -> IdnaResult {
        let Ok(name) = String::from_utf16(name) else {
            return IdnaResult::invalid_input();
        };
        let name = self.prepare(&name, self.options & UIDNA_NONTRANSITIONAL_TO_ASCII != 0);
        let uts46 = Uts46::new();
        match uts46.to_ascii(
            name.as_bytes(),
            AsciiDenyList::EMPTY,
            Hyphens::Check,
            DnsLength::VerifyAllowRootDot,
        ) {
            Ok(value) => IdnaResult {
                value: value.encode_utf16().collect(),
                failed: false,
                errors: 0,
            },
            Err(_) => {
                // ICU still returns the processed name alongside its errors.
                let lenient = uts46
                    .to_ascii(name.as_bytes(), AsciiDenyList::EMPTY, Hyphens::Allow, DnsLength::Ignore)
                    .map(|value| value.encode_utf16().collect())
                    .unwrap_or_default();
                IdnaResult {
                    value: lenient,
                    failed: false,
                    errors: UIDNA_ERROR_GENERIC,
                }
            }
        }
    }
}
