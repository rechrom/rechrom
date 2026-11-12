//! Chromium skeleton_generator.cc:46-342, preserving source variant bounds,
//! breadth-first enumeration, ICU rules and alternate U+04CF skeleton.
use crate::Error;
use icu_bidi::spoof::{FrozenUnicodeSet, SpoofChecker, Transliterator, UnicodeSet};
use std::{
    collections::{BTreeSet, VecDeque},
    sync::Mutex,
};
include!("skeleton_rules.rs");
pub struct SkeletonGenerator {
    diacritics: Mutex<Transliterator>,
    extra: Mutex<Transliterator>,
    lgc: FrozenUnicodeSet,
    multiple_with_diacritics: BTreeSet<u16>,
}
const MAPPING: &[(u16, &[&str])] = &[
    (0xfe, &["b", "p"]),
    (0x153, &["ce", "oe"]),
    (0x142, &["l", "t"]),
    (0x131, &["l", "i"]),
];
impl SkeletonGenerator {
    pub fn new() -> Result<Self, Error> {
        let mut result = Self {
            diacritics: Mutex::new(Transliterator::FromRules("DropAcc", DIACRITIC_RULES)?),
            extra: Mutex::new(Transliterator::FromRules("ExtraConf", EXTRA_RULES)?),
            lgc: UnicodeSet::Pattern(
                &"[[:Latin:][:Greek:][:Cyrillic:][0-9\\u002e_\\u002d][\\u0300-\\u0339]]"
                    .encode_utf16()
                    .collect::<Vec<_>>(),
            )?
            .IntoFrozen(),
            multiple_with_diacritics: BTreeSet::new(),
        };
        for &(unit, _) in MAPPING {
            if result.MaybeRemoveDiacritics(&[unit])? != [unit] {
                result.multiple_with_diacritics.insert(unit);
            }
        }
        Ok(result)
    }
    pub fn ShouldRemoveDiacriticsFromLabel(&self, label: &[u16]) -> bool {
        self.lgc.ContainsAllText(label)
    }
    pub fn MaybeRemoveDiacritics(&self, host: &[u16]) -> Result<Vec<u16>, Error> {
        let host = host.strip_suffix(&[46]).unwrap_or(host);
        if self.ShouldRemoveDiacriticsFromLabel(host) {
            Ok(self
                .diacritics
                .lock()
                .expect("transliterator owner")
                .Transliterate(host)?)
        } else {
            Ok(host.to_vec())
        }
    }
    pub fn GetSkeletons(
        &self,
        checker: &SpoofChecker,
        input: &[u16],
    ) -> Result<BTreeSet<String>, Error> {
        let host = self.MaybeRemoveDiacritics(input)?;
        let mut variants = Self::GenerateSupplementalHostnames(&host, 128);
        if input
            .iter()
            .any(|c| self.multiple_with_diacritics.contains(c))
        {
            variants.extend(Self::GenerateSupplementalHostnames(input, 128));
        }
        let mut skeletons = BTreeSet::new();
        for variant in variants {
            let host = variant.strip_suffix(&[46]).unwrap_or(&variant);
            let host = self
                .extra
                .lock()
                .expect("transliterator owner")
                .Transliterate(host)?;
            if host.contains(&0x4cf) {
                let alternate = host
                    .iter()
                    .map(|&c| if c == 0x4cf { 0x6c } else { c })
                    .collect::<Vec<_>>();
                if let Ok(skeleton) = checker.Skeleton(&alternate) {
                    skeletons.insert(String::from_utf16_lossy(&skeleton));
                }
            }
            if let Ok(skeleton) = checker.Skeleton(&host) {
                skeletons.insert(String::from_utf16_lossy(&skeleton));
            }
        }
        Ok(skeletons)
    }
    pub fn GenerateSupplementalHostnames(input: &[u16], max: usize) -> BTreeSet<Vec<u16>> {
        let mut output = BTreeSet::new();
        if input.is_empty() || input.len() > 32 || max == 0 {
            return output;
        }
        // Source roundtrip UTF16ToUTF8→UnicodeString replaces isolated surrogates.
        let input = String::from_utf16_lossy(input)
            .encode_utf16()
            .collect::<Vec<_>>();
        if input
            .iter()
            .filter(|&&c| MAPPING.iter().any(|&(k, _)| c == k))
            .count()
            > 5
        {
            return output;
        }
        let mut queue = VecDeque::from([(0, Vec::<u16>::new())]);
        while let Some((index, current)) = queue.pop_front() {
            if index == input.len() {
                output.insert(current);
                if output.len() == max {
                    break;
                }
                continue;
            }
            let c = input[index];
            let mut original = current.clone();
            original.push(c);
            queue.push_back((index + 1, original));
            if let Some(&(_, alternates)) = MAPPING.iter().find(|&&(key, _)| key == c) {
                for alternate in alternates {
                    let mut next = current.clone();
                    next.extend(alternate.encode_utf16());
                    queue.push_back((index + 1, next));
                }
            }
        }
        output
    }
}
