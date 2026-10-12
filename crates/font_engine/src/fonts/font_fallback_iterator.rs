// Chromium FontFallbackIterator, translated from revision
// 78ef49f74feba3d0dde852881feef92bbf86aad7:
// third_party/blink/renderer/platform/fonts/font_fallback_iterator.{h,cc}.
// Resolved CSS faces are flattened at the package boundary; unicode_ranges_
// retains each segmented face's range and the source retains its lifetime.
#![allow(non_snake_case)]

use std::collections::HashSet;

use super::font::Font;
use super::font_fallback_metadata::{FontFallbackMetadata, FontFallbackRole};
use super::font_fallback_priority::{
    FontFallbackPriority, IsEmojiPresentationEmoji, IsNonTextFallbackPriority,
    IsTextPresentationEmoji,
};
use super::simple_font_data::SimpleFontData;
use crate::text::native::character::Character;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FallbackStage {
    FontGroupFonts,
    FallbackPriorityFonts,
    SystemFonts,
    LastResort,
    FirstCandidateForNotdefGlyph,
    OutOfLuck,
}

#[derive(Clone)]
pub(crate) struct FontFallbackIterator<'a> {
    font: &'a Font,
    priority: FontFallbackPriority,
    stage: FallbackStage,
    group_index: usize,
    attempted_system_fonts: HashSet<*const SimpleFontData>,
    unique_entire_faces: HashSet<u32>,
    first_candidate: Option<*const SimpleFontData>,
    loading_ranges: Vec<Vec<(u32, u32)>>,
}

impl<'a> FontFallbackIterator<'a> {
    pub(crate) fn new(font: &'a Font, priority: FontFallbackPriority) -> Self {
        Self {
            font,
            priority,
            stage: FallbackStage::FontGroupFonts,
            group_index: 0,
            attempted_system_fonts: HashSet::new(),
            unique_entire_faces: HashSet::new(),
            first_candidate: None,
            loading_ranges: Vec::new(),
        }
    }

    pub(crate) fn Reset(&mut self) {
        self.stage = FallbackStage::FontGroupFonts;
        self.group_index = 0;
        self.attempted_system_fonts.clear();
        self.unique_entire_faces.clear();
        self.first_candidate = None;
        self.loading_ranges.clear();
    }

    pub(crate) fn HasNext(&self) -> bool {
        self.stage != FallbackStage::OutOfLuck
    }

    pub(crate) fn HasLikelyScript(character: u32) -> bool {
        let mut status = 0;
        let script = unsafe { icu_bidi::native::uscript_getScript(character as i32, &mut status) };
        status <= 0 && script != 0 && script != 1
    }

    pub(crate) fn ChooseHintIndex(hints: &[u32]) -> usize {
        assert!(!hints.is_empty());
        hints
            .iter()
            .enumerate()
            .skip(1)
            .find(|(_, character)| Self::HasLikelyScript(**character))
            .map_or(0, |(index, _)| index)
    }

    pub(crate) fn NeedsSystemFallback(&self) -> bool {
        self.stage != FallbackStage::FontGroupFonts
            || self.group_index >= self.font.FontGroupLength()
    }

    pub(crate) fn NeedsHintList(&self) -> bool {
        if self.stage != FallbackStage::FontGroupFonts
            || self.group_index >= self.font.FontGroupLength()
        {
            return false;
        }
        let candidate = self.font.FontDataAt(self.group_index);
        !candidate.is_null()
            && !unsafe { &*candidate }
                .PlatformData()
                .unicode_ranges_
                .is_empty()
    }

    fn Unique(&mut self, candidate: *const SimpleFontData) -> Option<*const SimpleFontData> {
        if candidate.is_null() {
            return None;
        }
        let data = unsafe { &*candidate };
        let platform = data.PlatformData();
        if data.IsLoading() || self.unique_entire_faces.contains(&platform.UniqueID()) {
            return None;
        }
        // A subset must not suppress a later range of the same physical face.
        if platform.unicode_ranges_.is_empty() {
            self.unique_entire_faces.insert(platform.UniqueID());
        }
        self.first_candidate.get_or_insert(candidate);
        Some(candidate)
    }

    fn RangeContributes(&mut self, candidate: *const SimpleFontData, hints: &[u32]) -> bool {
        let data = unsafe { &*candidate };
        let ranges = &data.PlatformData().unicode_ranges_;
        if ranges.is_empty() {
            return true;
        }
        let pending = data.GetCustomFontData();
        let contributes = hints.iter().any(|hint| {
            ranges
                .iter()
                .any(|&(start, end)| (start..=end).contains(hint))
                && (!data.IsLoading()
                    || pending.is_some_and(|custom| custom.IsPendingDataUrl())
                    || !self.loading_ranges.iter().any(|loading| {
                        loading
                            .iter()
                            .any(|&(start, end)| (start..=end).contains(hint))
                    }))
        });
        if contributes {
            if let Some(custom) = pending {
                custom.BeginLoadIfNeeded();
            }
            if data.IsLoading() {
                self.loading_ranges.push(ranges.clone());
                return false;
            }
        }
        contributes
    }

    fn SystemFontForHint(
        &mut self,
        hint: u32,
        priority: FontFallbackPriority,
        text: &[u16],
    ) -> *const SimpleFontData {
        if hint == 0
            || Character::IsPrivateUse(hint as i32)
            || Character::IsNonCharacter(hint as i32)
        {
            return std::ptr::null();
        }
        let description = self.font.GetFontDescription();
        let locale = description.LocaleOrDefault().Ascii();
        let requested_weight = description.Weight().RawValue() as f32 / 4.0;
        let requested_italic = description.Style().RawValue() >= 14 * 4;
        let metadata = self.font.FallbackMetadata();
        // Keep the request's original presentation even in the ordinary
        // system stage following the priority stage.
        let host_ranks = self.font.SystemFallbackRanks(text, self.priority);
        let mut indices: Vec<_> = (self.font.FontGroupLength()..self.font.FontDataCount())
            .filter(|&index| {
                !matches!(
                    metadata[index].role,
                    FontFallbackRole::kFamilyOnly | FontFallbackRole::kLastResort
                ) && host_ranks
                    .as_ref()
                    .is_none_or(|ranks| ranks[index].is_some())
            })
            .collect();
        indices.sort_by(|&left, &right| {
            host_ranks
                .as_ref()
                .map(|ranks| ranks[left].cmp(&ranks[right]))
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    FallbackCandidateRank(
                        &metadata[left],
                        hint,
                        &locale,
                        priority,
                        requested_weight,
                        requested_italic,
                    )
                    .partial_cmp(&FallbackCandidateRank(
                        &metadata[right],
                        hint,
                        &locale,
                        priority,
                        requested_weight,
                        requested_italic,
                    ))
                    .unwrap_or(std::cmp::Ordering::Equal)
                })
                .then_with(|| left.cmp(&right))
        });
        // Match the host's character fallback query lazily. Never realize all
        // catalog faces merely to produce a pre-ranked vector for the shaper.
        for index in indices {
            let candidate = self.font.FontDataAt(index);
            if !candidate.is_null() {
                let data = unsafe { &*candidate };
                // Scalar coverage is only a hint: an admitted face can still
                // fail a combining/variation cluster. Advance through the
                // remaining admitted candidates when the shaper retries the
                // same hint, rather than selecting that face forever.
                if !data.IsLoading()
                    && !self.attempted_system_fonts.contains(&candidate)
                    && !self
                        .unique_entire_faces
                        .contains(&data.PlatformData().UniqueID())
                    && data.PlatformData().FontContainsCharacter(hint)
                {
                    self.attempted_system_fonts.insert(candidate);
                    return candidate;
                }
            }
        }
        std::ptr::null()
    }

    pub(crate) fn Next(&mut self, hints: &[u32]) -> Option<*const SimpleFontData> {
        let text: Vec<u16> = hints
            .iter()
            .filter_map(|&hint| char::from_u32(hint))
            .flat_map(|character| character.encode_utf16(&mut [0; 2]).to_vec())
            .collect();
        self.NextForText(hints, &text)
    }

    pub(crate) fn NextForText(
        &mut self,
        hints: &[u32],
        text: &[u16],
    ) -> Option<*const SimpleFontData> {
        // Family unicode-range hints can span several outstanding slices.
        // If this call skips directly into system fallback, its hint must
        // still describe the exact slice used by the contextual host query.
        let system_hints: Vec<u32> = char::decode_utf16(text.iter().copied())
            .map(|character| character.unwrap_or(char::REPLACEMENT_CHARACTER) as u32)
            .collect();
        loop {
            let candidate = match self.stage {
                FallbackStage::OutOfLuck => return None,
                FallbackStage::FontGroupFonts => {
                    if self.group_index == self.font.FontGroupLength() {
                        self.stage = if IsNonTextFallbackPriority(self.priority) {
                            FallbackStage::FallbackPriorityFonts
                        } else {
                            FallbackStage::SystemFonts
                        };
                        continue;
                    }
                    let candidate = self.font.FontDataAt(self.group_index);
                    self.group_index += 1;
                    if candidate.is_null() || !self.RangeContributes(candidate, hints) {
                        continue;
                    }
                    candidate
                }
                FallbackStage::FallbackPriorityFonts => {
                    self.stage = FallbackStage::SystemFonts;
                    self.SystemFontForHint(
                        system_hints.first().copied().unwrap_or(0),
                        self.priority,
                        text,
                    )
                }
                FallbackStage::SystemFonts => {
                    let hint = system_hints
                        .get(if system_hints.is_empty() {
                            0
                        } else {
                            Self::ChooseHintIndex(&system_hints)
                        })
                        .copied()
                        .unwrap_or(0);
                    if hint == 0 {
                        self.stage = FallbackStage::LastResort;
                        continue;
                    }
                    let candidate = self.SystemFontForHint(hint, FontFallbackPriority::kText, text);
                    if candidate.is_null() {
                        self.stage = FallbackStage::LastResort;
                        continue;
                    }
                    candidate
                }
                FallbackStage::LastResort => {
                    self.stage = FallbackStage::FirstCandidateForNotdefGlyph;
                    (self.font.FontGroupLength()..self.font.FontDataCount())
                        .find(|&index| {
                            self.font.FallbackMetadata()[index].role
                                == FontFallbackRole::kLastResort
                        })
                        .map_or(self.font.PrimaryFont(), |index| self.font.FontDataAt(index))
                }
                FallbackStage::FirstCandidateForNotdefGlyph => {
                    self.stage = FallbackStage::OutOfLuck;
                    return self.first_candidate.or_else(|| {
                        let primary = self.font.PrimaryFont();
                        (!primary.is_null()).then_some(primary)
                    });
                }
            };
            if let Some(candidate) = self.Unique(candidate) {
                return Some(candidate);
            }
        }
    }
}

fn LocaleRank(candidate_locales: &[String], requested: &str) -> u8 {
    if candidate_locales.is_empty() {
        return 2;
    }
    let requested = requested.replace('_', "-").to_ascii_lowercase();
    let requested_language = requested.split('-').next().unwrap_or("");
    let mut rank = 3;
    for candidate in candidate_locales {
        let candidate = candidate.replace('_', "-").to_ascii_lowercase();
        if candidate == requested
            || (!candidate.is_empty()
                && (requested.starts_with(&(candidate.clone() + "-"))
                    || candidate.starts_with(&(requested.clone() + "-"))))
        {
            return 0;
        }
        if !requested_language.is_empty()
            && candidate.split('-').next().unwrap_or("") == requested_language
        {
            rank = rank.min(1);
        }
    }
    rank
}

fn IsMathCharacter(character: u32) -> bool {
    matches!(
        character,
        0x2190..=0x21ff
            | 0x2200..=0x22ff
            | 0x27c0..=0x27ef
            | 0x2980..=0x2aff
            | 0x1d400..=0x1d7ff
    )
}

fn IsSymbolCharacter(character: u32) -> bool {
    matches!(
        character,
        0x2000..=0x206f
            | 0x20a0..=0x20cf
            | 0x2100..=0x214f
            | 0x2300..=0x27bf
            | 0x2b00..=0x2bff
    )
}

fn RoleRank(role: FontFallbackRole, hint: u32, priority: FontFallbackPriority) -> u8 {
    let order = if IsEmojiPresentationEmoji(priority) {
        [
            FontFallbackRole::kEmoji,
            FontFallbackRole::kSymbol,
            FontFallbackRole::kText,
            FontFallbackRole::kMath,
        ]
    } else if IsTextPresentationEmoji(priority) {
        [
            FontFallbackRole::kSymbol,
            FontFallbackRole::kText,
            FontFallbackRole::kMath,
            FontFallbackRole::kEmoji,
        ]
    } else if IsMathCharacter(hint) {
        [
            FontFallbackRole::kMath,
            FontFallbackRole::kSymbol,
            FontFallbackRole::kText,
            FontFallbackRole::kEmoji,
        ]
    } else if IsSymbolCharacter(hint) {
        [
            FontFallbackRole::kSymbol,
            FontFallbackRole::kText,
            FontFallbackRole::kMath,
            FontFallbackRole::kEmoji,
        ]
    } else {
        [
            FontFallbackRole::kText,
            FontFallbackRole::kSymbol,
            FontFallbackRole::kMath,
            FontFallbackRole::kEmoji,
        ]
    };
    order
        .iter()
        .position(|candidate| *candidate == role)
        .map_or(5, |rank| rank as u8)
}

// CSS Fonts directional weight matching, shared with the face-group binding.
fn FallbackWeightRank(candidate: f32, requested: f32) -> (u8, f32) {
    if requested < 400.0 {
        if candidate <= requested {
            (0, requested - candidate)
        } else {
            (1, candidate - requested)
        }
    } else if requested <= 500.0 {
        if candidate >= requested && candidate <= 500.0 {
            (0, candidate - requested)
        } else if candidate < requested {
            (1, requested - candidate)
        } else {
            (2, candidate - requested)
        }
    } else if candidate >= requested {
        (0, candidate - requested)
    } else {
        (1, requested - candidate)
    }
}

fn FallbackCandidateRank(
    metadata: &FontFallbackMetadata,
    hint: u32,
    locale: &str,
    priority: FontFallbackPriority,
    requested_weight: f32,
    requested_italic: bool,
) -> (u8, u8, bool, u8, f32) {
    let (weight_direction, weight_distance) = FallbackWeightRank(metadata.weight, requested_weight);
    (
        RoleRank(metadata.role, hint, priority),
        LocaleRank(&metadata.locales, locale),
        metadata.italic != requested_italic,
        weight_direction,
        weight_distance,
    )
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::fonts::font_description::FontDescription;
    use crate::fonts::font_orientation::FontOrientation;
    use crate::fonts::font_platform_data::FontPlatformData;
    use crate::fonts::text_rendering_mode::TextRenderingMode;
    use crate::text::native::opentype_font::OpenTypeFont;
    use foundation::{LayoutHeapScope, MakeGarbageCollected};
    use std::sync::Arc;

    pub(crate) fn face(character: u32, ranges: Vec<(u32, u32)>) -> *const SimpleFontData {
        let mut bytes = crate::minimal_font::Build(false, 0, character, false, false);
        // The minimal builder emits its caller's character before Hebrew.
        // Format 12 groups must be sorted, including supplementary test input.
        let parsed = ttf_parser::Face::parse(&bytes, 0).unwrap();
        let cmap = parsed
            .raw_face()
            .table(ttf_parser::Tag::from_bytes(b"cmap"))
            .unwrap();
        let offset = cmap.as_ptr() as usize - bytes.as_ptr() as usize;
        let mut groups: Vec<_> = bytes[offset + 28..offset + cmap.len()]
            .chunks_exact(12)
            .map(|group| group.to_vec())
            .collect();
        groups.sort();
        for (index, group) in groups.iter().enumerate() {
            bytes[offset + 28 + index * 12..offset + 40 + index * 12].copy_from_slice(group);
        }
        let raw = Arc::new(OpenTypeFont::new(&bytes, 0, 16.0, &[]));
        let platform = MakeGarbageCollected(FontPlatformData::new(
            raw,
            None,
            16.0,
            false,
            false,
            TextRenderingMode::kAutoTextRendering,
            vec![],
            FontOrientation::kHorizontal,
            ranges,
        ));
        SimpleFontData::Create(platform)
    }

    fn metadata(role: FontFallbackRole) -> FontFallbackMetadata {
        FontFallbackMetadata {
            role,
            ..Default::default()
        }
    }

    #[test]
    fn deduplicates_entire_faces_and_retries_first_for_notdef() {
        let _heap = LayoutHeapScope::new();
        let first = face('A' as u32, vec![]);
        let last = face('B' as u32, vec![]);
        let font = Font::new_with_fallback_metadata(
            FontDescription::default(),
            &[first, first, last],
            first,
            &[
                metadata(FontFallbackRole::kFamilyOnly),
                metadata(FontFallbackRole::kFamilyOnly),
                metadata(FontFallbackRole::kLastResort),
            ],
            2,
        );
        let mut iterator = FontFallbackIterator::new(&font, FontFallbackPriority::kText);
        assert_eq!(iterator.Next(&[0x10ffff]), Some(first));
        assert_eq!(iterator.Next(&[0x10ffff]), Some(last));
        assert!(iterator.HasNext());
        assert_eq!(iterator.Next(&[0x10ffff]), Some(first));
        assert!(!iterator.HasNext());
        assert_eq!(iterator.Next(&[0x10ffff]), None);
        iterator.Reset();
        assert_eq!(iterator.Next(&[0x10ffff]), Some(first));
    }

    #[test]
    fn private_use_skips_system_but_still_tries_last_resort() {
        let _heap = LayoutHeapScope::new();
        let first = face('A' as u32, vec![]);
        let system = face(0xe000, vec![]);
        let last = face('B' as u32, vec![]);
        let font = Font::new_with_fallback_metadata(
            FontDescription::default(),
            &[first, system, last],
            first,
            &[
                metadata(FontFallbackRole::kFamilyOnly),
                metadata(FontFallbackRole::kText),
                metadata(FontFallbackRole::kLastResort),
            ],
            1,
        );
        let mut iterator = FontFallbackIterator::new(&font, FontFallbackPriority::kText);
        assert_eq!(iterator.Next(&[0xe000]), Some(first));
        assert_eq!(iterator.Next(&[0xe000]), Some(last));
        assert_eq!(iterator.Next(&[0xe000]), Some(first));
    }

    #[test]
    fn hints_change_after_covered_clusters_leave_the_queue() {
        let _heap = LayoutHeapScope::new();
        let first = face('A' as u32, vec![]);
        let second = face('B' as u32, vec![]);
        let third = face('C' as u32, vec![]);
        let font = Font::new_with_fallback_metadata(
            FontDescription::default(),
            &[first, second, third],
            first,
            &[
                metadata(FontFallbackRole::kFamilyOnly),
                metadata(FontFallbackRole::kText),
                metadata(FontFallbackRole::kText),
            ],
            1,
        );
        let mut iterator = FontFallbackIterator::new(&font, FontFallbackPriority::kText);
        assert_eq!(iterator.Next(&['B' as u32]), Some(first));
        assert_eq!(iterator.Next(&['B' as u32]), Some(second));
        assert_eq!(iterator.Next(&['C' as u32]), Some(third));
        assert_eq!(iterator.Next(&['C' as u32]), Some(first));
        assert!(!iterator.HasNext());
    }

    #[test]
    fn repeated_cluster_hint_advances_through_admitted_faces() {
        let _heap = LayoutHeapScope::new();
        let primary = face('A' as u32, vec![]);
        let first = face('B' as u32, vec![]);
        let second = face('B' as u32, vec![]);
        let font = Font::new_with_fallback_metadata(
            FontDescription::default(),
            &[primary, first, first, second],
            primary,
            &[
                metadata(FontFallbackRole::kFamilyOnly),
                metadata(FontFallbackRole::kText),
                metadata(FontFallbackRole::kText),
                metadata(FontFallbackRole::kText),
            ],
            1,
        );
        let mut iterator = FontFallbackIterator::new(&font, FontFallbackPriority::kText);
        // Both system faces cover B. The shaper can reject the first face for
        // the complete B+mark cluster without changing its scalar hint.
        assert_eq!(iterator.Next(&['B' as u32]), Some(primary));
        assert_eq!(iterator.Next(&['B' as u32]), Some(first));
        assert_eq!(iterator.Next(&['B' as u32]), Some(second));
        assert_eq!(iterator.Next(&['B' as u32]), Some(primary));
        assert!(!iterator.HasNext());
        assert_eq!(iterator.Next(&['B' as u32]), None);
        iterator.Reset();
        assert_eq!(iterator.Next(&['B' as u32]), Some(primary));
        assert_eq!(iterator.Next(&['B' as u32]), Some(first));
    }

    #[test]
    fn repeated_hint_does_not_revisit_a_system_range_subset() {
        let _heap = LayoutHeapScope::new();
        let primary = face('A' as u32, vec![]);
        let subset = face('B' as u32, vec![(0x42, 0x42)]);
        let next = face('B' as u32, vec![]);
        let font = Font::new_with_fallback_metadata(
            FontDescription::default(),
            &[primary, subset, next],
            primary,
            &[
                metadata(FontFallbackRole::kFamilyOnly),
                metadata(FontFallbackRole::kText),
                metadata(FontFallbackRole::kText),
            ],
            1,
        );
        let mut iterator = FontFallbackIterator::new(&font, FontFallbackPriority::kText);
        assert_eq!(iterator.Next(&['B' as u32]), Some(primary));
        assert_eq!(iterator.Next(&['B' as u32]), Some(subset));
        assert_eq!(iterator.Next(&['B' as u32]), Some(next));
        assert_eq!(iterator.Next(&['B' as u32]), Some(primary));
        assert_eq!(iterator.Next(&['B' as u32]), None);
    }

    #[test]
    fn range_subsets_preserve_shared_face_identity() {
        let _heap = LayoutHeapScope::new();
        let first = face('A' as u32, vec![(0x41, 0x41)]);
        let mut second_platform = unsafe { &*first }.PlatformData().clone();
        second_platform.unicode_ranges_ = vec![(0x30, 0x30)];
        let second = SimpleFontData::Create(MakeGarbageCollected(second_platform));
        let font = Font::new_with_ordered_fonts(FontDescription::default(), &[first, second]);
        let mut iterator = FontFallbackIterator::new(&font, FontFallbackPriority::kText);
        assert!(iterator.NeedsHintList());
        assert_eq!(iterator.Next(&[0x41, 0x30]), Some(first));
        assert_eq!(iterator.Next(&[0x30]), Some(second.cast_const()));
    }

    #[test]
    fn presentation_font_is_tried_once_before_regular_system_fallback() {
        let _heap = LayoutHeapScope::new();
        let first = face('A' as u32, vec![]);
        let text = face('B' as u32, vec![]);
        let emoji = face('B' as u32, vec![]);
        let font = Font::new_with_fallback_metadata(
            FontDescription::default(),
            &[first, text, emoji],
            first,
            &[
                metadata(FontFallbackRole::kFamilyOnly),
                metadata(FontFallbackRole::kText),
                metadata(FontFallbackRole::kEmoji),
            ],
            1,
        );
        let mut iterator =
            FontFallbackIterator::new(&font, FontFallbackPriority::kEmojiEmojiWithVS);
        assert_eq!(iterator.Next(&[0x42]), Some(first));
        assert_eq!(iterator.Next(&[0x42]), Some(emoji));
        assert_eq!(iterator.Next(&[0x42]), Some(text));
        assert_eq!(iterator.Next(&[0x42]), Some(first));
        assert!(!iterator.HasNext());
    }

    #[test]
    fn contextual_system_hint_does_not_leak_from_another_pending_slice() {
        let _heap = LayoutHeapScope::new();
        let primary = face('A' as u32, vec![]);
        let matching = face('B' as u32, vec![]);
        let unrelated = face(0x6f22, vec![]);
        let font = Font::new_with_fallback_metadata(
            FontDescription::default(),
            &[primary, matching, unrelated],
            primary,
            &[
                metadata(FontFallbackRole::kFamilyOnly),
                metadata(FontFallbackRole::kText),
                metadata(FontFallbackRole::kText),
            ],
            1,
        );
        let mut iterator = FontFallbackIterator::new(&font, FontFallbackPriority::kText);
        assert_eq!(iterator.NextForText(&[0x6f22], &[0x42]), Some(primary));
        assert_eq!(iterator.NextForText(&[0x6f22], &[0x42]), Some(matching));
    }

    #[test]
    fn prefers_script_hint_over_leading_punctuation() {
        assert_eq!(FontFallbackIterator::ChooseHintIndex(&[0x2e, 0x1000]), 1);
        assert_eq!(FontFallbackIterator::ChooseHintIndex(&[0x2e, 0x20]), 0);
    }
}
