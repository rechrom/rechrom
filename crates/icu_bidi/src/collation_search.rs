//! Collation-based string search with the `usearch.h` semantics Blink and
//! Chromium rely on, built on the ICU4X collator.
//!
//! A match is a text range that compares equal to the pattern at the
//! collator's strength and starts and ends on grapheme cluster boundaries,
//! as ICU's StringSearch requires. Among equal ranges at one start, the
//! shortest wins; searching continues after the previous match start when
//! overlapping matches are allowed, otherwise after its end.

use icu_collator::options::{CollatorOptions, Strength};
use icu_collator::preferences::CollationType;
use icu_collator::{Collator, CollatorBorrowed, CollatorPreferences};
use icu_locale_core::Locale;
use icu_segmenter::GraphemeClusterSegmenter;
use std::cmp::Ordering;

/// ICU4C UCollationStrength values.
pub(crate) const UCOL_PRIMARY: i32 = 0;
pub(crate) const UCOL_SECONDARY: i32 = 1;
pub(crate) const UCOL_TERTIARY: i32 = 2;

fn strength(value: i32) -> Strength {
    match value {
        UCOL_PRIMARY => Strength::Primary,
        UCOL_SECONDARY => Strength::Secondary,
        UCOL_TERTIARY => Strength::Tertiary,
        3 => Strength::Quaternary,
        _ => Strength::Identical,
    }
}

/// Parses an ICU locale id (`en_US`, `ja@collation=search`).
fn locale(name: &str) -> (Locale, bool) {
    let (base, keywords) = name.split_once('@').unwrap_or((name, ""));
    let search = keywords
        .split(';')
        .any(|keyword| keyword.trim().eq_ignore_ascii_case("collation=search"));
    let base = base.replace('_', "-");
    (base.parse().unwrap_or(Locale::UNKNOWN), search)
}

fn collator(locale_name: &str, strength_value: i32) -> CollatorBorrowed<'static> {
    let (locale, search) = locale(locale_name);
    let mut options = CollatorOptions::default();
    options.strength = Some(strength(strength_value));
    let mut preferences = CollatorPreferences::from(&locale);
    if search {
        preferences.collation_type = Some(CollationType::Search);
        if let Ok(collator) = Collator::try_new(preferences, options) {
            return collator;
        }
        preferences.collation_type = None;
    }
    Collator::try_new(preferences, options)
        .or_else(|_| Collator::try_new(CollatorPreferences::default(), options))
        .expect("root collation data")
}

pub(crate) struct CollationSearch {
    locale: String,
    strength: i32,
    collator: CollatorBorrowed<'static>,
    pattern: Vec<u16>,
    text: Vec<u16>,
    boundaries: Vec<usize>,
    offset: usize,
    previous: Option<(usize, usize)>,
    overlap: bool,
}

impl CollationSearch {
    pub(crate) fn new(locale: &str, strength: i32) -> Self {
        Self {
            locale: locale.to_owned(),
            strength,
            collator: collator(locale, strength),
            pattern: Vec::new(),
            text: Vec::new(),
            boundaries: vec![0],
            offset: 0,
            previous: None,
            overlap: false,
        }
    }

    pub(crate) fn locale(&self) -> &str {
        &self.locale
    }

    pub(crate) fn strength(&self) -> i32 {
        self.strength
    }

    pub(crate) fn set_strength(&mut self, strength: i32) {
        if strength != self.strength {
            self.strength = strength;
            self.collator = collator(&self.locale, strength);
        }
        self.reset();
    }

    pub(crate) fn set_overlap(&mut self, overlap: bool) {
        self.overlap = overlap;
    }

    pub(crate) fn set_pattern(&mut self, pattern: &[u16]) {
        self.pattern = pattern.to_vec();
        self.reset();
    }

    pub(crate) fn set_text(&mut self, text: &[u16]) {
        self.text = text.to_vec();
        self.boundaries = GraphemeClusterSegmenter::new().segment_utf16(text).collect();
        if self.boundaries.first() != Some(&0) {
            self.boundaries.insert(0, 0);
        }
        self.reset();
    }

    pub(crate) fn reset(&mut self) {
        self.offset = 0;
        self.previous = None;
    }

    pub(crate) fn set_offset(&mut self, offset: usize) {
        self.offset = offset.min(self.text.len());
        self.previous = None;
    }

/// `usearch_next`: the next match at or after the search position.
    pub(crate) fn next(&mut self) -> Option<(usize, usize)> {
        let from = match self.previous {
            Some((start, length)) if !self.overlap => start + length.max(1),
            Some((start, _)) => start + 1,
            None => self.offset,
        };
        let first_index = self.boundaries.partition_point(|boundary| *boundary < from);
        // An equal range holds the pattern's collation elements plus
        // ignorables; bound it by the pattern's length in grapheme clusters.
        let max_graphemes = GraphemeClusterSegmenter::new()
            .segment_utf16(&self.pattern)
            .count()
            .saturating_mul(2)
            + 4;
        for (index, &start) in self.boundaries.iter().enumerate().skip(first_index) {
            if start >= self.text.len() {
                break;
            }
            for &end in self.boundaries[index + 1..].iter().take(max_graphemes) {
                // A range that sorts after the pattern either differs at some
                // collation element or already extends past it, so no longer
                // range from this start can be equal.
                match self.collator.compare_utf16(&self.text[start..end], &self.pattern) {
                    Ordering::Equal => {
                        let found = (start, end - start);
                        self.previous = Some(found);
                        self.offset = start;
                        return Some(found);
                    }
                    Ordering::Greater => break,
                    Ordering::Less => {}
                }
            }
        }
        self.previous = None;
        self.offset = self.text.len();
        None
    }
}
