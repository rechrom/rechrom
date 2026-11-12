//! Chromium URL display IDN mechanisms, revision
//! 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
#![allow(non_snake_case)]
pub mod idn_spoof_checker;
pub mod skeleton_generator;
mod top_domains;
use icu_bidi::idna::{Idna, UIDNA_CHECK_BIDI};
use idn_spoof_checker::{IDNSpoofChecker, IDNSpoofCheckerResult};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    ICU(i32),
    CorruptTrie,
}
impl From<i32> for Error {
    fn from(value: i32) -> Self {
        Self::ICU(value)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkeletonType {
    Full,
    SeparatorsRemoved,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TopDomainEntry {
    pub domain: String,
    pub is_top_bucket: bool,
    pub skeleton_type: SkeletonType,
}
pub struct URLFormatter {
    idna: Idna,
    spoof: IDNSpoofChecker,
}
impl URLFormatter {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            idna: Idna::OpenUTS46(UIDNA_CHECK_BIDI)?,
            spoof: IDNSpoofChecker::new()?,
        })
    }
    // url_formatter.cc:460-524. Failed conversion or failed checks retain the
    // original label; absence/corruption of required machinery is an error.
    fn component(
        &self,
        label: &[u16],
        tld: &str,
        tld_unicode: &[u16],
        ignore: bool,
    ) -> (Vec<u16>, bool) {
        if !label.starts_with(&[120, 110, 45, 45]) || label.last() == Some(&45) {
            return (label.to_vec(), false);
        }
        let decoded = self.idna.LabelToUnicode(label);
        if decoded.failed || decoded.errors != 0 {
            return (label.to_vec(), false);
        }
        let safe = self
            .spoof
            .SafeToDisplayAsUnicode(&decoded.value, tld, tld_unicode)
            == IDNSpoofCheckerResult::Safe;
        (
            if ignore || safe {
                decoded.value
            } else {
                label.to_vec()
            },
            true,
        )
    }
    /// url_formatter.cc:290-381. Whole-host top-domain rejection follows all
    /// per-label decisions and uses the original host for the final rollback.
    pub fn IDNToUnicode(&self, host: &str) -> Result<Vec<u16>, Error> {
        // The source std::string host is URL-canonical ASCII; retain its
        // explicit byte widening rather than adding a second IDNA pass.
        let host16 = host.bytes().map(u16::from).collect::<Vec<_>>();
        let tld = host.rsplit_once('.').map(|(_, tld)| tld).unwrap_or("");
        let tld_unicode = if host.contains('.') {
            self.component(
                &tld.bytes().map(u16::from).collect::<Vec<_>>(),
                "",
                &[],
                true,
            )
            .0
        } else {
            vec![]
        };
        let mut output = Vec::new();
        let mut idn = false;
        for (index, label) in host16.split(|&c| c == 46).enumerate() {
            if index > 0 {
                output.push(46);
            }
            if !label.is_empty() {
                let (text, converted) = self.component(label, tld, &tld_unicode, false);
                output.extend(text);
                idn |= converted;
            }
        }
        if idn && self.spoof.GetSimilarTopDomain(&output)?.is_some() {
            return Ok(host16);
        }
        Ok(output)
    }
}
pub fn IDNToUnicode(host: &str) -> Result<Vec<u16>, Error> {
    static FORMATTER: std::sync::OnceLock<Result<URLFormatter, Error>> = std::sync::OnceLock::new();
    FORMATTER
        .get_or_init(URLFormatter::new)
        .as_ref()
        .map_err(|e| *e)?
        .IDNToUnicode(host)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pinned_preload_corpus_roundtrips() {
        let mut top = true;
        let mut count = 0;
        for line in include_str!("../data/domains.skeletons")
            .lines()
            .map(str::trim)
        {
            if line == "###END_TOP_BUCKET###" {
                top = false;
                continue;
            }
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let fields = line.split(',').map(str::trim).collect::<Vec<_>>();
            for (kind, skeleton) in fields[..fields.len() - 1].iter().enumerate() {
                let entry = top_domains::decode(skeleton)
                    .expect("valid generated trie")
                    .expect("every pinned entry exists");
                assert_eq!(entry.domain, fields[fields.len() - 1], "{skeleton}");
                assert_eq!(entry.is_top_bucket, top, "{skeleton}");
                assert_eq!(
                    entry.skeleton_type,
                    if kind == 0 {
                        SkeletonType::Full
                    } else {
                        SkeletonType::SeparatorsRemoved
                    }
                );
                count += 1;
            }
        }
        assert_eq!(count, 9293);
    }
    #[test]
    fn unicode_policy_and_top_domain_rejection() {
        let formatter = URLFormatter::new().expect("ICU spoof/transliteration data");
        // Source IDNSpoofChecker tests: safe deviation, whole-script spoof,
        // invalid ACE and real top-domain diacritic skeleton rejection.
        for (host, expected) in [
            ("example.com", "example.com"),
            ("xn--fu-hia.de", "fuß.de"),
            ("xn--mxac2c.gr", "αβς.gr"),
            ("xn--80ak6aa92e.com", "xn--80ak6aa92e.com"),
            ("xn--googl-fsa.com", "xn--googl-fsa.com"),
            ("xn--.com", "xn--.com"),
        ] {
            assert_eq!(
                formatter.IDNToUnicode(host).unwrap(),
                expected.encode_utf16().collect::<Vec<_>>(),
                "{host}"
            );
        }
        // Independent policy branches from idn_spoof_checker_unittest.cc:
        // script-local ccTLD allowances, middle dot and digit lookalikes.
        use IDNSpoofCheckerResult::*;
        for (label, tld, expected) in [
            ("аррӏе", "com", WholeScriptConfusable),
            ("аррӏе", "ru", Safe),
            ("aþ", "com", TLDSpecificCharacters),
            ("aþ", "is", Safe),
            ("əa", "com", TLDSpecificCharacters),
            ("əa", "az", Safe),
            ("l·l", "com", UnsafeMiddleDot),
            ("l·l", "cat", Safe),
            ("1շ", "com", DigitLookalikes),
            ("abcノdef", "com", DangerousPattern),
        ] {
            assert_eq!(
                formatter.spoof.SafeToDisplayAsUnicode(
                    &label.encode_utf16().collect::<Vec<_>>(),
                    tld,
                    &tld.encode_utf16().collect::<Vec<_>>()
                ),
                expected,
                "{label}.{tld}"
            );
        }
        let google = formatter
            .spoof
            .GetSimilarTopDomain(&"googlé.com".encode_utf16().collect::<Vec<_>>())
            .unwrap()
            .unwrap();
        assert_eq!(google.domain, "google.com");
        assert!(formatter
            .spoof
            .GetSimilarTopDomain(&"súbdomain.google.com".encode_utf16().collect::<Vec<_>>())
            .unwrap()
            .is_none());
    }
}
