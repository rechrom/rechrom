//! CSS face activation is pure font selection; the resource owner completes
//! the emitted rule IDs. Blink: CSSFontSelectorBase/CSSSegmentedFontFace.
use crate::resolver::{EqualFamilyName, FontWeightRank};
use crate::{FontDescription, FontUnicodeRange};

#[derive(Clone, Debug, PartialEq)]
pub struct WebFontFace {
    pub rule_id: u64,
    pub family: String,
    pub weight: f64,
    pub italic: bool,
    pub unicode_ranges: Vec<FontUnicodeRange>,
    pub loaded: bool,
    pub failed: bool,
}

#[derive(Default)]
pub(crate) struct WebFontRegistry {
    pub faces: Vec<WebFontFace>,
    pub requests: Vec<u64>,
}
impl WebFontRegistry {
    pub fn WillUse(&mut self, description: &FontDescription, text: &[u16], fallback: bool) {
        let weight = description.Weight().ToFloat() as f64;
        let italic = description.Style().ToFloat() != 0.;
        let mut family = description.Family() as *const crate::FontFamily;
        while !family.is_null() {
            let entry = unsafe { &*family };
            let name = entry.FamilyName().Utf8();
            let candidates: Vec<_> = self
                .faces
                .iter()
                .filter(|face| EqualFamilyName(&face.family, &name))
                .collect();
            // FontSelectionAlgorithm picks a style/weight group before
            // CSSSegmentedFontFace matches the text's unicode ranges.
            let best = candidates.iter().min_by(|a, b| {
                (a.italic != italic, FontWeightRank(a.weight, weight))
                    .partial_cmp(&(b.italic != italic, FontWeightRank(b.weight, weight)))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            if let Some(best) = best {
                for character in char::decode_utf16(text.iter().copied()) {
                    let character = character.unwrap_or(char::REPLACEMENT_CHARACTER) as u32;
                    // Later declarations override overlapping unicode ranges.
                    if let Some(face) = candidates.iter().rev().find(|face| {
                        face.weight == best.weight
                            && face.italic == best.italic
                            && !face.failed
                            && (face.unicode_ranges.is_empty()
                                || face
                                    .unicode_ranges
                                    .iter()
                                    .any(|range| (range.start..=range.end).contains(&character)))
                    }) {
                        if !face.loaded && !self.requests.contains(&face.rule_id) {
                            self.requests.push(face.rule_id);
                        }
                    }
                }
            }
            if !fallback {
                break;
            }
            family = entry.Next();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FontFamilyType, FontQueryCollector, SharedFontFamily};
    use foundation::AtomicString;
    fn description(weight: i32) -> FontDescription {
        let mut d = FontDescription::default();
        d.SetFamily(&SharedFontFamily::Create(
            AtomicString::FromUtf8(b"Test"),
            FontFamilyType::kFamilyName,
            None,
        ));
        d.SetWeight(crate::FontSelectionValue::from_int(weight));
        d
    }
    fn face(id: u64, weight: f64, start: u32, end: u32) -> WebFontFace {
        WebFontFace {
            rule_id: id,
            family: "Test".into(),
            weight,
            italic: false,
            unicode_ranges: vec![FontUnicodeRange { start, end }],
            loaded: false,
            failed: false,
        }
    }
    #[test]
    fn text_demand_loads_only_matching_weight_and_unicode_shards() {
        let queries = FontQueryCollector::default();
        queries.RegisterWebFonts(vec![
            face(0, 400., 0x20, 0x7f),
            face(1, 400., 0x4e00, 0x9fff),
            face(2, 700., 0x20, 0x7f),
        ]);
        queries.RequestWebFonts(
            &description(400),
            &"hello".encode_utf16().collect::<Vec<_>>(),
            false,
        );
        assert_eq!(queries.DrainWebFontRequests(), vec![0]);
        queries.RequestWebFonts(
            &description(600),
            &"A中".encode_utf16().collect::<Vec<_>>(),
            false,
        );
        assert_eq!(queries.DrainWebFontRequests(), vec![2]);
        queries.RequestWebFonts(
            &description(400),
            &"中".encode_utf16().collect::<Vec<_>>(),
            false,
        );
        assert_eq!(queries.DrainWebFontRequests(), vec![1]);
    }
    #[test]
    fn overlapping_ranges_use_later_declaration_and_failed_face_falls_back() {
        let queries = FontQueryCollector::default();
        let first = face(0, 400., 0x20, 0x7f);
        let mut later = face(1, 400., 0x20, 0x7f);
        queries.RegisterWebFonts(vec![first.clone(), later.clone()]);
        queries.RequestWebFonts(&description(400), &[65], false);
        assert_eq!(queries.DrainWebFontRequests(), vec![1]);
        later.failed = true;
        queries.RegisterWebFonts(vec![first, later]);
        queries.RequestWebFonts(&description(400), &[65], false);
        assert_eq!(queries.DrainWebFontRequests(), vec![0]);
    }
    #[test]
    fn loaded_faces_and_unused_families_do_not_emit_downloads() {
        let queries = FontQueryCollector::default();
        let mut loaded = face(0, 400., 0x20, 0x7f);
        loaded.loaded = true;
        let mut unused = face(1, 400., 0x20, 0x7f);
        unused.family = "Unused".into();
        queries.RegisterWebFonts(vec![loaded, unused]);
        queries.RequestWebFonts(&description(400), &[65], false);
        assert!(queries.DrainWebFontRequests().is_empty());
    }
}
