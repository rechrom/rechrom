//! Chromium idn_spoof_checker.cc:184-594,648-836.
use crate::skeleton_generator::SkeletonGenerator;
use crate::{Error, SkeletonType, TopDomainEntry};
use icu_bidi::spoof::{FrozenUnicodeSet, Regex, SpoofChecker, UnicodeSet};
use std::cell::RefCell;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IDNSpoofCheckerResult {
    None,
    Safe,
    ICUSpoofChecks,
    DeviationCharacters,
    TLDSpecificCharacters,
    UnsafeMiddleDot,
    WholeScriptConfusable,
    DigitLookalikes,
    NonAsciiLatinCharMixedWithNonLatin,
    DangerousPattern,
}
struct WholeScript {
    all: FrozenUnicodeSet,
    lookalikes: FrozenUnicodeSet,
    tlds: &'static [&'static str],
}
pub struct IDNSpoofChecker {
    checker: SpoofChecker,
    skeleton: SkeletonGenerator,
    non_ascii_latin: FrozenUnicodeSet,
    _deviation_characters: FrozenUnicodeSet,
    kana: FrozenUnicodeSet,
    combining: FrozenUnicodeSet,
    icelandic: FrozenUnicodeSet,
    digits: FrozenUnicodeSet,
    digit_lookalikes: FrozenUnicodeSet,
    whole: Vec<WholeScript>,
}
fn set(pattern: &str) -> Result<FrozenUnicodeSet, Error> {
    Ok(UnicodeSet::Pattern(&pattern.encode_utf16().collect::<Vec<_>>())?.IntoFrozen())
}
fn codepoints(text: &[u16]) -> impl Iterator<Item = i32> + '_ {
    char::decode_utf16(text.iter().copied()).map(|c| match c {
        Ok(c) => c as i32,
        Err(e) => i32::from(e.unpaired_surrogate()),
    })
}
thread_local! {static DANGEROUS:RefCell<Option<Regex>>=const{RefCell::new(None)};}
impl IDNSpoofChecker {
    pub fn new() -> Result<Self, Error> {
        let mut checker = SpoofChecker::Open()?;
        checker.SetRestrictionLevel(0x30000000);
        let mut allowed = UnicodeSet::RecommendedAndInclusion()?;
        for c in [
            0x338, 0x58a, 0x2010, 0x2019, 0x2027, 0x30a0, 0x2bb, 0x2bc, 0x2ec, 0x138,
        ] {
            allowed.Remove(c);
        }
        #[cfg(target_vendor = "apple")]
        for c in [0x620, 0xf8c, 0xf8d, 0xf8e, 0xf8f] {
            allowed.Remove(c);
        }
        for (first, last) in [
            (0x1cd, 0x1dc),
            (0x1c80, 0x1c8f),
            (0x1e00, 0x1e9b),
            (0x1f00, 0x1fff),
            (0xa640, 0xa69f),
            (0xa720, 0xa7ff),
        ] {
            allowed.RemoveRange(first, last);
        }
        checker.SetAllowedUnicodeSet(&allowed)?;
        checker.SetChecks(checker.Checks()? | 0x40000000)?;
        let mut whole = Vec::new();
        let scripts: &[(&str, &str, &[&str])] = &[
            ("[[:Armn:]]", "[ագզէլհյոսւօՙ]", &["am"]),
            (
                "[[:Cyrl:]]",
                "[аысԁеԍһіюкјӏорԗԛѕтԝхуъьҽпгѵѡ]",
                &["bg", "by", "kz", "pyc", "ru", "su", "ua", "uz"],
            ),
            ("[[:Ethi:]]", "[ሀሠሰስበነተከዐዕዘጠፐꬅ]", &["er", "et"]),
            ("[[:Geor:]]", "[იოყძხჽჿ]", &["ge"]),
            ("[[:Grek:]]", "[αικνρυωηοτ]", &["gr"]),
            ("[[:Hebr:]]", "[דוחיןסװײ׳ﬦ]", &["il"]),
            ("[[:Beng:]]", "[০৭]", &[]),
            ("[[:Deva:]]", "[ऽ०ॱ]", &[]),
            ("[[:Gujr:]]", "[ડટ૦૧]", &[]),
            ("[[:Guru:]]", "[੦੧]", &[]),
            ("[[:Knda:]]", "[ಽ೦೧]", &[]),
            ("[[:Mlym:]]", "[ടഠധനറ൦]", &[]),
            ("[[:Orya:]]", "[ଠ୦୮]", &[]),
            ("[[:Taml:]]", "[டப௦]", &[]),
            ("[[:Telu:]]", "[౦౧]", &[]),
            ("[[:Mymr:]]", "[ခဂငထပဝ၀၂ၔၜ႐႑႕႖႗]", &["mm"]),
            #[cfg(target_os = "linux")]
            ("[[:Thai:]]", "[ทนบพรหเแ๐ดลปฟม]", &["th"]),
            #[cfg(not(target_os = "linux"))]
            ("[[:Thai:]]", "[บพเแ๐]", &["th"]),
        ];
        for &(all, lookalikes, tlds) in scripts {
            whole.push(WholeScript {
                all: set(all)?,
                lookalikes: set(lookalikes)?,
                tlds,
            });
        }
        Ok(Self {
            checker,
            skeleton: SkeletonGenerator::new()?,
            non_ascii_latin: set("[[:Latin:] - [a-zA-Z]]")?,
            _deviation_characters: set("[\\u00df\\u03c2\\u200c\\u200d]")?,
            kana: set("[\\u3078-\\u307a\\u30d8-\\u30da\\u30fb-\\u30fe]")?,
            combining: set("[\\u0300-\\u0339]")?,
            icelandic: set("[\\u00fe\\u00f0]")?,
            digits: set("[0-9]")?,
            digit_lookalikes: set("[θ२২੨੨૨೩೭շзҙӡउওਤ੩૩౩ဒვპੜკ੫丩ㄐճ৪੪୫૭୨౨]")?,
            whole,
        })
    }
    fn digit_lookalike(&self, label: &[u16]) -> bool {
        let mut found = false;
        for c in codepoints(label) {
            if self.digits.Contains(c) {
                continue;
            }
            if self.digit_lookalikes.Contains(c) {
                found = true;
                continue;
            }
            return false;
        }
        found
    }
    pub fn SafeToDisplayAsUnicode(
        &self,
        label: &[u16],
        tld: &str,
        tld_unicode: &[u16],
    ) -> IDNSpoofCheckerResult {
        use IDNSpoofCheckerResult::*;
        let Ok(result) = self.checker.Check(label) else {
            return ICUSpoofChecks;
        };
        if result & 0xffff != 0 {
            return ICUSpoofChecks;
        }
        if label.len() > 1 && tld != "is" && tld != "fo" && self.icelandic.ContainsSomeText(label) {
            return TLDSpecificCharacters;
        }
        if label.len() > 1 && tld != "az" && label.contains(&0x259) {
            return TLDSpecificCharacters;
        }
        for (index, &c) in label.iter().enumerate() {
            if c == 0xb7
                && (tld != "cat"
                    || index == 0
                    || index + 1 == label.len()
                    || label[index - 1] != 108
                    || label[index + 1] != 108)
            {
                return UnsafeMiddleDot;
            }
        }
        let restriction = result & 0x7f000000;
        if restriction == 0x10000000 {
            return Safe;
        }
        if restriction == 0x20000000
            && !self.kana.ContainsSomeText(label)
            && !self.combining.ContainsSomeText(label)
        {
            const ALLOWED: &[&str] = &[
                "секс",
                "как",
                "коса",
                "курс",
                "парк",
                "такий",
                "укроп",
                "сахарок",
                "покраска",
                "театр",
                "астро",
                "пхукет",
            ];
            for script in &self.whole {
                let mut letters = UnicodeSet::Empty();
                for c in codepoints(label) {
                    if script.all.Contains(c) {
                        letters.Add(c);
                    }
                }
                if !letters.IsEmpty()
                    && script.lookalikes.ContainsAll(&letters)
                    && !script.all.ContainsSomeText(tld_unicode)
                    && !script.tlds.contains(&tld)
                    && !ALLOWED
                        .iter()
                        .any(|s| s.encode_utf16().eq(label.iter().copied()))
                {
                    return WholeScriptConfusable;
                }
            }
            return if self.digit_lookalike(label) {
                DigitLookalikes
            } else {
                Safe
            };
        }
        if self.digit_lookalike(label) {
            return DigitLookalikes;
        }
        if self.non_ascii_latin.ContainsSomeText(label)
            && !self.skeleton.ShouldRemoveDiacriticsFromLabel(label)
        {
            return NonAsciiLatinCharMixedWithNonLatin;
        }
        let dangerous = DANGEROUS.with(|slot| {
            let mut slot = slot.borrow_mut();
            let regex = slot.get_or_insert_with(|| {
                Regex::Open(
                    &include_str!("dangerous_pattern.txt")
                        .encode_utf16()
                        .collect::<Vec<_>>(),
                )
                .expect("source dangerous pattern compiles")
            });
            regex.Find(label).expect("valid ICU matcher")
        });
        if dangerous {
            DangerousPattern
        } else {
            Safe
        }
    }
    pub fn GetSkeletons(&self, host: &[u16]) -> Result<std::collections::BTreeSet<String>, Error> {
        self.skeleton.GetSkeletons(&self.checker, host)
    }
    pub fn GetSimilarTopDomain(&self, host: &[u16]) -> Result<Option<TopDomainEntry>, Error> {
        for skeleton in self.GetSkeletons(host)? {
            if let Some(top) = crate::top_domains::lookup(&skeleton, SkeletonType::Full)? {
                let domain = top.domain.encode_utf16().collect::<Vec<_>>();
                if host == domain || is_subdomain(host, &domain) {
                    return Ok(None);
                }
                return Ok(Some(top));
            }
        }
        Ok(None)
    }
}
fn is_subdomain(host: &[u16], domain: &[u16]) -> bool {
    host.len() > domain.len()
        && host[host.len() - domain.len() - 1] == 46
        && host[host.len() - domain.len()..]
            .iter()
            .zip(domain)
            .all(|(&a, &b)| ascii_lower(a) == ascii_lower(b))
}
fn ascii_lower(c: u16) -> u16 {
    if (65..=90).contains(&c) {
        c + 32
    } else {
        c
    }
}
