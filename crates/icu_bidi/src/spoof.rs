//! ICU primitives for Chromium url_formatter/spoof_checks, on ICU4X
//! properties and the Unicode UTS #39 security data in `data/`. Policy,
//! script allowlists, skeleton rules and top-domain decisions stay in that
//! owner.
//!
//! - `UnicodeSet`: the ICU UnicodeSet pattern subset Chromium uses (nested
//!   sets, ranges, `\u` escapes, `[:Property:]`, `-` and `&` operators).
//! - `SpoofChecker`: ICU `uspoof_check2` (restriction level, mixed numbers,
//!   hidden overlay, character limit, invisible marks) and UTS #39 skeletons.
//! - `Regex`: an ICU regular expression over UTF-16 text.
//! - `Transliterator`: the rule forms Chromium's skeleton rules use.
#![allow(non_snake_case)]
// UScriptCode bit positions are ICU4C's numbering.
#![allow(deprecated)]

use std::collections::HashMap;
use std::ops::RangeInclusive;
use std::sync::OnceLock;

use icu_normalizer::{ComposingNormalizerBorrowed, DecomposingNormalizerBorrowed};
use icu_properties::props::{GeneralCategory, GeneralCategoryGroup, Script};
use icu_properties::script::ScriptWithExtensions;
use icu_properties::{CodePointMapData, PropertyParser};

use crate::ustring::code_points;

const U_ILLEGAL_ARGUMENT_ERROR: i32 = 1;
const U_REGEX_RULE_SYNTAX: i32 = 0x10300;

// ---------------------------------------------------------------------------
// UnicodeSet

/// A set of code points as sorted, disjoint, non-adjacent inclusive ranges.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UnicodeSet {
    ranges: Vec<RangeInclusive<u32>>,
}

impl UnicodeSet {
    fn from_ranges(mut ranges: Vec<RangeInclusive<u32>>) -> Self {
        ranges.sort_by_key(|range| *range.start());
        let mut merged: Vec<RangeInclusive<u32>> = Vec::with_capacity(ranges.len());
        for range in ranges {
            if range.start() > range.end() {
                continue;
            }
            match merged.last_mut() {
                Some(last) if *range.start() <= last.end().saturating_add(1) => {
                    if range.end() > last.end() {
                        *last = *last.start()..=*range.end();
                    }
                }
                _ => merged.push(range),
            }
        }
        Self { ranges: merged }
    }

    fn union(&self, other: &Self) -> Self {
        Self::from_ranges(self.ranges.iter().chain(&other.ranges).cloned().collect())
    }

    fn complement(&self) -> Self {
        let mut ranges = Vec::with_capacity(self.ranges.len() + 1);
        let mut next = 0u32;
        for range in &self.ranges {
            if *range.start() > next {
                ranges.push(next..=range.start() - 1);
            }
            next = range.end() + 1;
        }
        if next <= 0x10ffff {
            ranges.push(next..=0x10ffff);
        }
        Self { ranges }
    }

    fn intersection(&self, other: &Self) -> Self {
        self.complement().union(&other.complement()).complement()
    }

    fn difference(&self, other: &Self) -> Self {
        self.intersection(&other.complement())
    }

    pub fn Empty() -> Self {
        Self::default()
    }

    pub fn Pattern(pattern: &[u16]) -> Result<Self, i32> {
        let pattern = String::from_utf16(pattern).map_err(|_| U_ILLEGAL_ARGUMENT_ERROR)?;
        let mut parser = SetParser {
            chars: pattern.chars().collect(),
            index: 0,
        };
        let set = parser.set()?;
        parser.skip_whitespace();
        if parser.index != parser.chars.len() {
            return Err(U_ILLEGAL_ARGUMENT_ERROR);
        }
        Ok(set)
    }

    pub fn Freeze(&mut self) {}

    pub fn Add(&mut self, ch: i32) {
        self.AddRange(ch, ch);
    }

    pub fn AddRange(&mut self, start: i32, end: i32) {
        *self = self.union(&Self::from_ranges(vec![start as u32..=end as u32]));
    }

    pub fn AddAll(&mut self, other: &Self) {
        *self = self.union(other);
    }

    pub fn Remove(&mut self, ch: i32) {
        self.RemoveRange(ch, ch);
    }

    pub fn RemoveRange(&mut self, start: i32, end: i32) {
        *self = self.difference(&Self::from_ranges(vec![start as u32..=end as u32]));
    }

    fn contains(&self, ch: u32) -> bool {
        let index = self.ranges.partition_point(|range| *range.end() < ch);
        self.ranges.get(index).is_some_and(|range| range.contains(&ch))
    }

    pub fn Contains(&self, ch: i32) -> bool {
        u32::try_from(ch).is_ok_and(|ch| self.contains(ch))
    }

    pub fn ContainsAll(&self, other: &Self) -> bool {
        other.difference(self).ranges.is_empty()
    }

    pub fn IsEmpty(&self) -> bool {
        self.ranges.is_empty()
    }

    pub fn ContainsSomeText(&self, text: &[u16]) -> bool {
        code_points(text).any(|(_, ch)| self.contains(ch))
    }

    pub fn ContainsAllText(&self, text: &[u16]) -> bool {
        code_points(text).all(|(_, ch)| self.contains(ch))
    }

    /// `uspoof_getRecommendedSet` ∪ `uspoof_getInclusionSet`: UTS #39
    /// Identifier_Type Recommended and Inclusion.
    pub fn RecommendedAndInclusion() -> Result<Self, i32> {
        Ok(security_data().allowed.clone())
    }

    pub fn IntoFrozen(self) -> FrozenUnicodeSet {
        FrozenUnicodeSet(self)
    }
}

#[derive(Clone)]
pub struct FrozenUnicodeSet(UnicodeSet);

impl std::ops::Deref for FrozenUnicodeSet {
    type Target = UnicodeSet;
    fn deref(&self) -> &UnicodeSet {
        &self.0
    }
}

/// `[:Name:]` / `\p{Name}`: a script, general category or category group.
fn property_set(name: &str) -> Result<UnicodeSet, i32> {
    let name = name.trim();
    let (key, value) = match name.split_once('=') {
        Some((key, value)) => (Some(key.trim().to_ascii_lowercase()), value.trim()),
        None => (None, name),
    };
    let scripts = || {
        PropertyParser::<Script>::new().get_loose(value).map(|script| {
            UnicodeSet::from_ranges(
                CodePointMapData::<Script>::new()
                    .iter_ranges_for_value(script)
                    .collect(),
            )
        })
    };
    let script_extensions = || {
        PropertyParser::<Script>::new().get_loose(value).map(|script| {
            let data = ScriptWithExtensions::new();
            UnicodeSet::from_ranges(data.get_script_extensions_ranges(script).collect())
        })
    };
    let categories = || {
        PropertyParser::<GeneralCategoryGroup>::new()
            .get_loose(value)
            .map(|group| {
                UnicodeSet::from_ranges(
                    CodePointMapData::<GeneralCategory>::new()
                        .iter_ranges_for_group(group)
                        .collect(),
                )
            })
    };
    let found = match key.as_deref() {
        None => categories().or_else(scripts),
        Some("sc" | "script") => scripts(),
        Some("scx" | "script_extensions") => script_extensions(),
        Some("gc" | "general_category") => categories(),
        Some(_) => None,
    };
    found.ok_or(U_ILLEGAL_ARGUMENT_ERROR)
}

struct SetParser {
    chars: Vec<char>,
    index: usize,
}

impl SetParser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.index).copied()
    }

    fn starts_with(&self, text: &str) -> bool {
        text.chars()
            .enumerate()
            .all(|(offset, ch)| self.chars.get(self.index + offset) == Some(&ch))
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.index += 1;
        }
    }

    fn expect(&mut self, ch: char) -> Result<(), i32> {
        if self.peek() == Some(ch) {
            self.index += 1;
            Ok(())
        } else {
            Err(U_ILLEGAL_ARGUMENT_ERROR)
        }
    }

    fn hex(&mut self, digits: usize) -> Result<u32, i32> {
        let text: String = self.chars.get(self.index..self.index + digits).ok_or(U_ILLEGAL_ARGUMENT_ERROR)?.iter().collect();
        self.index += digits;
        u32::from_str_radix(&text, 16).map_err(|_| U_ILLEGAL_ARGUMENT_ERROR)
    }

    fn escaped(&mut self) -> Result<u32, i32> {
        self.expect('\\')?;
        let ch = self.peek().ok_or(U_ILLEGAL_ARGUMENT_ERROR)?;
        self.index += 1;
        match ch {
            'u' => self.hex(4),
            'U' => self.hex(8),
            'x' if self.peek() == Some('{') => {
                self.index += 1;
                let start = self.index;
                while self.peek().is_some_and(|ch| ch != '}') {
                    self.index += 1;
                }
                let text: String = self.chars[start..self.index].iter().collect();
                self.expect('}')?;
                u32::from_str_radix(&text, 16).map_err(|_| U_ILLEGAL_ARGUMENT_ERROR)
            }
            'x' => self.hex(2),
            other => Ok(other as u32),
        }
    }

    fn literal(&mut self) -> Result<u32, i32> {
        match self.peek() {
            Some('\\') => self.escaped(),
            Some(ch) if !matches!(ch, '[' | ']' | '-' | '&' | '^') => {
                self.index += 1;
                Ok(ch as u32)
            }
            _ => Err(U_ILLEGAL_ARGUMENT_ERROR),
        }
    }

    fn property(&mut self) -> Result<UnicodeSet, i32> {
        let (close, negated) = if self.starts_with("[:") {
            self.index += 2;
            let negated = self.peek() == Some('^');
            if negated {
                self.index += 1;
            }
            (":]", negated)
        } else {
            let negated = self.starts_with("\\P");
            self.index += 2;
            self.expect('{')?;
            ("}", negated)
        };
        let start = self.index;
        while !self.starts_with(close) {
            if self.peek().is_none() {
                return Err(U_ILLEGAL_ARGUMENT_ERROR);
            }
            self.index += 1;
        }
        let name: String = self.chars[start..self.index].iter().collect();
        self.index += close.len();
        let set = property_set(&name)?;
        Ok(if negated { set.complement() } else { set })
    }

    fn item(&mut self) -> Result<UnicodeSet, i32> {
        if self.starts_with("[:") || self.starts_with("\\p") || self.starts_with("\\P") {
            return self.property();
        }
        if self.peek() == Some('[') {
            return self.set();
        }
        let start = self.literal()?;
        self.skip_whitespace();
        if self.peek() == Some('-') && self.chars.get(self.index + 1).is_some_and(|ch| *ch != '[' && *ch != ']' && !ch.is_whitespace()) {
            self.index += 1;
            self.skip_whitespace();
            let end = self.literal()?;
            if end < start {
                return Err(U_ILLEGAL_ARGUMENT_ERROR);
            }
            return Ok(UnicodeSet::from_ranges(vec![start..=end]));
        }
        Ok(UnicodeSet::from_ranges(vec![start..=start]))
    }

    fn set(&mut self) -> Result<UnicodeSet, i32> {
        self.skip_whitespace();
        if self.starts_with("[:") || self.starts_with("\\p") || self.starts_with("\\P") {
            return self.property();
        }
        self.expect('[')?;
        let negated = self.peek() == Some('^');
        if negated {
            self.index += 1;
        }
        let mut result = UnicodeSet::default();
        loop {
            self.skip_whitespace();
            match self.peek() {
                None => return Err(U_ILLEGAL_ARGUMENT_ERROR),
                Some(']') => {
                    self.index += 1;
                    break;
                }
                Some('-') => {
                    self.index += 1;
                    self.skip_whitespace();
                    if self.peek() == Some(']') {
                        // A trailing '-' is a literal hyphen.
                        result = result.union(&UnicodeSet::from_ranges(vec![0x2d..=0x2d]));
                        continue;
                    }
                    let other = self.item()?;
                    result = result.difference(&other);
                }
                Some('&') => {
                    self.index += 1;
                    let other = self.item()?;
                    result = result.intersection(&other);
                }
                Some(_) => {
                    let item = self.item()?;
                    result = result.union(&item);
                }
            }
        }
        Ok(if negated { result.complement() } else { result })
    }
}

// ---------------------------------------------------------------------------
// UTS #39 data

struct SecurityData {
    allowed: UnicodeSet,
    confusables: HashMap<u32, Vec<u32>>,
}

fn parse_code_points(field: &str) -> Vec<u32> {
    field
        .split_whitespace()
        .filter_map(|value| u32::from_str_radix(value, 16).ok())
        .collect()
}

fn security_data() -> &'static SecurityData {
    static DATA: OnceLock<SecurityData> = OnceLock::new();
    DATA.get_or_init(|| {
        let mut allowed = Vec::new();
        for line in include_str!("../data/IdentifierType.txt").lines() {
            let line = line.split('#').next().unwrap_or("");
            let Some((range, types)) = line.split_once(';') else {
                continue;
            };
            if !types
                .split_whitespace()
                .any(|kind| kind == "Recommended" || kind == "Inclusion")
            {
                continue;
            }
            let range = range.trim();
            let (start, end) = range.split_once("..").unwrap_or((range, range));
            if let (Ok(start), Ok(end)) = (u32::from_str_radix(start, 16), u32::from_str_radix(end, 16)) {
                allowed.push(start..=end);
            }
        }
        let mut confusables = HashMap::new();
        for line in include_str!("../data/confusables.txt").lines() {
            let line = line.trim_start_matches('\u{feff}').split('#').next().unwrap_or("");
            let mut fields = line.split(';');
            let (Some(source), Some(target)) = (fields.next(), fields.next()) else {
                continue;
            };
            if let [source] = parse_code_points(source)[..] {
                confusables.insert(source, parse_code_points(target));
            }
        }
        SecurityData {
            allowed: UnicodeSet::from_ranges(allowed),
            confusables,
        }
    })
}

fn nfd(text: &[u16]) -> Vec<u16> {
    DecomposingNormalizerBorrowed::new_nfd()
        .normalize_utf16(text)
        .into_owned()
}

fn push_code_point(output: &mut Vec<u16>, code_point: u32) {
    match char::from_u32(code_point) {
        Some(ch) => {
            let mut buffer = [0u16; 2];
            output.extend_from_slice(ch.encode_utf16(&mut buffer));
        }
        None => output.push(code_point as u16),
    }
}

/// UTS #39 skeleton: NFD, confusable prototype mapping, NFD.
fn skeleton(text: &[u16]) -> Vec<u16> {
    let confusables = &security_data().confusables;
    let mut mapped = Vec::with_capacity(text.len());
    for (_, code_point) in code_points(&nfd(text)) {
        match confusables.get(&code_point) {
            Some(prototype) => prototype.iter().for_each(|cp| push_code_point(&mut mapped, *cp)),
            None => push_code_point(&mut mapped, code_point),
        }
    }
    nfd(&mapped)
}

// ---------------------------------------------------------------------------
// SpoofChecker

const USPOOF_RESTRICTION_LEVEL: i32 = 16;
const USPOOF_INVISIBLE: i32 = 32;
const USPOOF_CHAR_LIMIT: i32 = 64;
const USPOOF_MIXED_NUMBERS: i32 = 128;
const USPOOF_HIDDEN_OVERLAY: i32 = 256;
const USPOOF_ALL_CHECKS: i32 = 0xFFFF;
const USPOOF_AUX_INFO: i32 = 0x40000000;
const USPOOF_ASCII: i32 = 0x10000000;
const USPOOF_SINGLE_SCRIPT_RESTRICTIVE: i32 = 0x20000000;
const USPOOF_HIGHLY_RESTRICTIVE: i32 = 0x30000000;
const USPOOF_MODERATELY_RESTRICTIVE: i32 = 0x40000000;
const USPOOF_MINIMALLY_RESTRICTIVE: i32 = 0x50000000;
const USPOOF_UNRESTRICTIVE: i32 = 0x60000000;

// ICU4C UScriptCode values.
const COMMON: u16 = 0;
const INHERITED: u16 = 1;
const BOPOMOFO: u16 = 5;
const CHEROKEE: u16 = 6;
const CYRILLIC: u16 = 8;
const GREEK: u16 = 14;
const HAN: u16 = 17;
const HANGUL: u16 = 18;
const HIRAGANA: u16 = 20;
const KATAKANA: u16 = 22;
const LATIN: u16 = 25;
const JAPANESE: u16 = 105;
const KOREAN: u16 = 119;
const HAN_WITH_BOPOMOFO: u16 = 172;

/// ICU ScriptSet: one bit per UScriptCode.
#[derive(Clone, Copy, PartialEq, Eq)]
struct ScriptSet([u64; 4]);

impl ScriptSet {
    const ALL: Self = Self([u64::MAX; 4]);

    fn test(&self, script: u16) -> bool {
        self.0[usize::from(script / 64)] & (1 << (script % 64)) != 0
    }

    fn set(&mut self, script: u16) {
        self.0[usize::from(script / 64)] |= 1 << (script % 64);
    }

    fn intersect(&mut self, other: &Self) {
        for (bits, other) in self.0.iter_mut().zip(other.0) {
            *bits &= other;
        }
    }

    fn is_empty(&self) -> bool {
        self.0.iter().all(|bits| *bits == 0)
    }

    /// `ScriptSet::setScriptExtensions` + UTS #39 augmentation, with
    /// Common/Inherited meaning every script.
    fn for_code_point(code_point: u32) -> Self {
        let mut set = Self([0; 4]);
        for script in ScriptWithExtensions::new()
            .get_script_extensions_val32(code_point)
            .iter()
        {
            set.set(script.to_icu4c_value());
        }
        if set.test(COMMON) || set.test(INHERITED) {
            return Self::ALL;
        }
        if set.test(HAN) {
            set.set(HAN_WITH_BOPOMOFO);
            set.set(JAPANESE);
            set.set(KOREAN);
        }
        if set.test(HIRAGANA) || set.test(KATAKANA) {
            set.set(JAPANESE);
        }
        if set.test(HANGUL) {
            set.set(KOREAN);
        }
        if set.test(BOPOMOFO) {
            set.set(HAN_WITH_BOPOMOFO);
        }
        set
    }

    fn resolved(text: &[u16], without: Option<u16>) -> Self {
        let mut result = Self::ALL;
        for (_, code_point) in code_points(text) {
            let set = Self::for_code_point(code_point);
            if without.is_some_and(|script| set.test(script)) {
                continue;
            }
            result.intersect(&set);
        }
        result
    }
}

fn is_dot_lead_no_lookup(code_point: u32) -> bool {
    matches!(code_point, 0x69 | 0x6a | 0x131 | 0x237 | 0x6c)
        || unsafe { crate::ubidi_ffi::u_hasBinaryProperty(code_point as i32, 27) } != 0
}

fn is_illegal_combining_dot_lead(code_point: u32) -> bool {
    if is_dot_lead_no_lookup(code_point) {
        return true;
    }
    let mut single = Vec::new();
    push_code_point(&mut single, code_point);
    let skeleton = skeleton(&single);
    code_points(&skeleton)
        .last()
        .is_some_and(|(_, last)| last != code_point && is_dot_lead_no_lookup(last))
}

/// The value of the decimal digit zero of `code_point`'s digit run.
fn digit_zero(code_point: u32) -> u32 {
    let categories = CodePointMapData::<GeneralCategory>::new();
    let mut start = code_point;
    while start > 0 && categories.get32(start - 1) == GeneralCategory::DecimalNumber {
        start -= 1;
    }
    start + (code_point - start) / 10 * 10
}

pub struct SpoofChecker {
    checks: i32,
    restriction_level: i32,
    allowed: Option<UnicodeSet>,
}

impl SpoofChecker {
    pub fn Open() -> Result<Self, i32> {
        Ok(Self {
            checks: USPOOF_ALL_CHECKS,
            restriction_level: USPOOF_HIGHLY_RESTRICTIVE,
            allowed: None,
        })
    }

    pub fn SetRestrictionLevel(&mut self, level: i32) {
        self.restriction_level = level;
        self.checks |= USPOOF_RESTRICTION_LEVEL;
    }

    pub fn Checks(&self) -> Result<i32, i32> {
        Ok(self.checks)
    }

    pub fn SetChecks(&mut self, checks: i32) -> Result<(), i32> {
        self.checks = checks;
        Ok(())
    }

    pub fn SetAllowedUnicodeSet(&mut self, set: &UnicodeSet) -> Result<(), i32> {
        self.allowed = Some(set.clone());
        self.checks |= USPOOF_CHAR_LIMIT;
        Ok(())
    }

    fn is_allowed(&self, code_point: u32) -> bool {
        self.allowed.as_ref().is_none_or(|set| set.contains(code_point))
    }

    /// ICU `SpoofImpl::getRestrictionLevel`.
    fn restriction_level(&self, text: &[u16]) -> i32 {
        if !code_points(text).all(|(_, cp)| self.is_allowed(cp)) {
            return USPOOF_UNRESTRICTIVE;
        }
        if text.iter().all(|unit| *unit < 0x80) {
            return USPOOF_ASCII;
        }
        if !ScriptSet::resolved(text, None).is_empty() {
            return USPOOF_SINGLE_SCRIPT_RESTRICTIVE;
        }
        let without_latin = ScriptSet::resolved(text, Some(LATIN));
        if without_latin.test(KOREAN) || without_latin.test(JAPANESE) || without_latin.test(HAN_WITH_BOPOMOFO) {
            return USPOOF_HIGHLY_RESTRICTIVE;
        }
        if !without_latin.is_empty()
            && !without_latin.test(CYRILLIC)
            && !without_latin.test(GREEK)
            && !without_latin.test(CHEROKEE)
        {
            return USPOOF_MODERATELY_RESTRICTIVE;
        }
        USPOOF_MINIMALLY_RESTRICTIVE
    }

    /// ICU `uspoof_check2` without the deprecated confusable checks.
    pub fn Check(&self, text: &[u16]) -> Result<i32, i32> {
        let mut result = 0;
        if self.checks & (USPOOF_RESTRICTION_LEVEL | USPOOF_AUX_INFO) != 0 {
            let level = self.restriction_level(text);
            if self.checks & USPOOF_RESTRICTION_LEVEL != 0 && level > self.restriction_level {
                result |= USPOOF_RESTRICTION_LEVEL;
            }
            if self.checks & USPOOF_AUX_INFO != 0 {
                result |= level;
            }
        }
        if self.checks & USPOOF_MIXED_NUMBERS != 0 {
            let categories = CodePointMapData::<GeneralCategory>::new();
            let mut zeros = code_points(text)
                .filter(|(_, cp)| categories.get32(*cp) == GeneralCategory::DecimalNumber)
                .map(|(_, cp)| digit_zero(cp));
            if let Some(first) = zeros.next() {
                if zeros.any(|zero| zero != first) {
                    result |= USPOOF_MIXED_NUMBERS;
                }
            }
        }
        let decomposed = nfd(text);
        if self.checks & USPOOF_HIDDEN_OVERLAY != 0 {
            let mut saw_lead = false;
            for (_, code_point) in code_points(&decomposed) {
                if saw_lead && code_point == 0x307 {
                    result |= USPOOF_HIDDEN_OVERLAY;
                    break;
                }
                let combining = unsafe { crate::ubidi_ffi::u_getCombiningClass(code_point as i32) };
                if combining == 0 || combining == 230 {
                    saw_lead = is_illegal_combining_dot_lead(code_point);
                }
            }
        }
        if self.checks & USPOOF_CHAR_LIMIT != 0 && !code_points(text).all(|(_, cp)| self.is_allowed(cp)) {
            result |= USPOOF_CHAR_LIMIT;
        }
        if self.checks & USPOOF_INVISIBLE != 0 {
            let categories = CodePointMapData::<GeneralCategory>::new();
            let mut first_mark = None;
            let mut marks: Vec<u32> = Vec::new();
            for (_, code_point) in code_points(&decomposed) {
                if categories.get32(code_point) != GeneralCategory::NonspacingMark {
                    first_mark = None;
                    marks.clear();
                    continue;
                }
                let Some(first) = first_mark else {
                    first_mark = Some(code_point);
                    continue;
                };
                if marks.is_empty() {
                    marks.push(first);
                }
                if marks.contains(&code_point) {
                    result |= USPOOF_INVISIBLE;
                    break;
                }
                marks.push(code_point);
            }
        }
        Ok(result)
    }

    pub fn Skeleton(&self, text: &[u16]) -> Result<Vec<u16>, i32> {
        Ok(skeleton(text))
    }
}

// ---------------------------------------------------------------------------
// Regex

/// An ICU regular expression; ICU's `\uhhhh` and `\p{scx=...}` syntax is
/// also the `regex` crate's.
pub struct Regex(regex::Regex);

impl Regex {
    pub fn Open(pattern: &[u16]) -> Result<Self, i32> {
        let pattern = String::from_utf16(pattern).map_err(|_| U_ILLEGAL_ARGUMENT_ERROR)?;
        regex::Regex::new(&pattern)
            .map(Self)
            .map_err(|_| U_REGEX_RULE_SYNTAX)
    }

    pub fn Find(&mut self, text: &[u16]) -> Result<bool, i32> {
        Ok(self.0.is_match(&String::from_utf16_lossy(text)))
    }
}

// ---------------------------------------------------------------------------
// Transliterator

enum Step {
    Nfd,
    Nfc,
    Remove(UnicodeSet),
    Convert(Vec<(UnicodeSet, Vec<u16>)>),
}

/// An ICU rule-based transliterator for the rule forms Chromium uses:
/// `::NFD;`, `::NFC;`, `::[set] Remove;` and single-character conversion
/// rules `x > y;` / `[set] > y;`. Consecutive conversion rules form one
/// pass, as ICU groups them into one anonymous transliterator.
pub struct Transliterator {
    steps: Vec<Step>,
}

fn unescape(text: &str) -> Vec<u16> {
    let mut output = Vec::new();
    let mut chars = text.chars();
    while let Some(ch) = chars.next() {
        let ch = if ch == '\\' { chars.next().unwrap_or('\\') } else { ch };
        let mut buffer = [0u16; 2];
        output.extend_from_slice(ch.encode_utf16(&mut buffer));
    }
    output
}

impl Transliterator {
    pub fn FromRules(_id: &str, rules: &str) -> Result<Self, i32> {
        let mut steps = Vec::new();
        for rule in rules.split(';').map(str::trim).filter(|rule| !rule.is_empty()) {
            if let Some(transform) = rule.strip_prefix("::") {
                let transform = transform.trim();
                steps.push(match transform {
                    "NFD" => Step::Nfd,
                    "NFC" => Step::Nfc,
                    _ => {
                        let set = transform
                            .strip_suffix("Remove")
                            .ok_or(U_ILLEGAL_ARGUMENT_ERROR)?
                            .trim();
                        Step::Remove(UnicodeSet::Pattern(&set.encode_utf16().collect::<Vec<_>>())?)
                    }
                });
                continue;
            }
            let (source, target) = rule.split_once('>').ok_or(U_ILLEGAL_ARGUMENT_ERROR)?;
            let source = source.trim();
            let set = if source.starts_with('[') {
                UnicodeSet::Pattern(&source.encode_utf16().collect::<Vec<_>>())?
            } else {
                let source = unescape(source);
                let mut points = code_points(&source);
                let (Some((_, ch)), None) = (points.next(), points.next()) else {
                    return Err(U_ILLEGAL_ARGUMENT_ERROR);
                };
                UnicodeSet::from_ranges(vec![ch..=ch])
            };
            let target = unescape(target.trim());
            match steps.last_mut() {
                Some(Step::Convert(rules)) => rules.push((set, target)),
                _ => steps.push(Step::Convert(vec![(set, target)])),
            }
        }
        Ok(Self { steps })
    }

    pub fn Transliterate(&mut self, input: &[u16]) -> Result<Vec<u16>, i32> {
        let mut text = input.to_vec();
        for step in &self.steps {
            text = match step {
                Step::Nfd => nfd(&text),
                Step::Nfc => ComposingNormalizerBorrowed::new_nfc()
                    .normalize_utf16(&text)
                    .into_owned(),
                Step::Remove(set) => {
                    let mut output = Vec::with_capacity(text.len());
                    for (_, code_point) in code_points(&text) {
                        if !set.contains(code_point) {
                            push_code_point(&mut output, code_point);
                        }
                    }
                    output
                }
                Step::Convert(rules) => {
                    let mut output = Vec::with_capacity(text.len());
                    for (_, code_point) in code_points(&text) {
                        match rules.iter().find(|(set, _)| set.contains(code_point)) {
                            Some((_, target)) => output.extend_from_slice(target),
                            None => push_code_point(&mut output, code_point),
                        }
                    }
                    output
                }
            };
        }
        Ok(text)
    }
}
