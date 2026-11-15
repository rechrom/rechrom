//! OpenType layout and MATH table queries that Blink makes through HarfBuzz's
//! `hb_ot_layout_*` and `hb_ot_math_*` APIs and HarfRust does not expose.
//!
//! The readers follow the OpenType binary layout directly and mirror the
//! HarfBuzz semantics Blink depends on:
//! - harfbuzz/src/hb-ot-layout.cc: `hb_ot_layout_table_get_feature_tags`,
//!   `hb_ot_layout_lookup_collect_glyphs` (before, input and after sets; the
//!   output set is not requested by Blink, so nested lookups are not visited).
//! - harfbuzz/src/hb-ot-math-table.hh: `MathConstants::get_value`.
//!
//! Every read is bounds-checked; malformed data reads as absent, matching
//! HarfBuzz's sanitizer, which turns an invalid table into an empty one.

#[derive(Clone, Copy)]
struct Data<'a>(&'a [u8]);

impl<'a> Data<'a> {
    fn u16(self, offset: usize) -> Option<u16> {
        let bytes = self.0.get(offset..offset.checked_add(2)?)?;
        Some(u16::from_be_bytes([bytes[0], bytes[1]]))
    }

    fn i16(self, offset: usize) -> Option<i16> {
        self.u16(offset).map(|value| value as i16)
    }

    fn u32(self, offset: usize) -> Option<u32> {
        let bytes = self.0.get(offset..offset.checked_add(4)?)?;
        Some(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// A table referenced by a 16-bit offset from the start of `self`.
    /// A null offset means the table is absent.
    fn sub16(self, offset_position: usize) -> Option<Data<'a>> {
        match self.u16(offset_position)? as usize {
            0 => None,
            offset => self.0.get(offset..).map(Data),
        }
    }

    fn sub32(self, offset_position: usize) -> Option<Data<'a>> {
        match self.u32(offset_position)? as usize {
            0 => None,
            offset => self.0.get(offset..).map(Data),
        }
    }

    fn u16_array(self, offset: usize, count: usize) -> impl Iterator<Item = u16> + 'a {
        let data = self;
        (0..count).map_while(move |index| data.u16(offset + index * 2))
    }
}

/// `hb_ot_layout_has_positioning` / `hb_ot_layout_has_substitution`: a GSUB
/// or GPOS table with a supported major version.
pub(crate) fn HasLayoutData(table: &[u8]) -> bool {
    Data(table).u16(0) == Some(1)
}

/// `hb_ot_layout_table_get_feature_tags` for the whole FeatureList.
pub(crate) fn FeatureTags(table: &[u8]) -> Vec<u32> {
    let table = Data(table);
    if !HasLayoutData(table.0) {
        return Vec::new();
    }
    let Some(list) = table.sub16(6) else {
        return Vec::new();
    };
    let count = list.u16(0).unwrap_or(0) as usize;
    (0..count)
        .map_while(|index| list.u32(2 + index * 6))
        .collect()
}

/// Whether `glyph` belongs to the before, input or after glyph set that
/// `hb_ot_layout_lookup_collect_glyphs` reports for any lookup in the table.
pub(crate) fn LookupsReferenceGlyph(table: &[u8], is_gpos: bool, glyph: u16) -> bool {
    let table = Data(table);
    if !HasLayoutData(table.0) {
        return false;
    }
    let Some(list) = table.sub16(8) else {
        return false;
    };
    let count = list.u16(0).unwrap_or(0) as usize;
    let extension_type = if is_gpos { 9 } else { 7 };
    (0..count).any(|index| {
        let Some(lookup) = list.sub16(2 + index * 2) else {
            return false;
        };
        let (Some(lookup_type), Some(subtable_count)) = (lookup.u16(0), lookup.u16(4)) else {
            return false;
        };
        (0..subtable_count as usize).any(|subtable_index| {
            let Some(mut subtable) = lookup.sub16(6 + subtable_index * 2) else {
                return false;
            };
            let mut kind = lookup_type;
            if kind == extension_type {
                // ExtensionPosFormat1 / ExtensionSubstFormat1.
                let Some(inner) = subtable.u16(2) else {
                    return false;
                };
                let Some(target) = subtable.sub32(4) else {
                    return false;
                };
                kind = inner;
                subtable = target;
            }
            let collector = Collector { glyph };
            if is_gpos {
                collector.positioning(kind, subtable)
            } else {
                collector.substitution(kind, subtable)
            }
            .unwrap_or(false)
        })
    })
}

struct Collector {
    glyph: u16,
}

impl Collector {
    fn coverage(&self, coverage: Option<Data<'_>>) -> Option<bool> {
        let coverage = coverage?;
        match coverage.u16(0)? {
            1 => {
                let count = coverage.u16(2)? as usize;
                Some(coverage.u16_array(4, count).any(|glyph| glyph == self.glyph))
            }
            2 => {
                let count = coverage.u16(2)? as usize;
                Some((0..count).any(|index| {
                    let record = 4 + index * 6;
                    matches!(
                        (coverage.u16(record), coverage.u16(record + 2)),
                        (Some(start), Some(end)) if (start..=end).contains(&self.glyph)
                    )
                }))
            }
            _ => Some(false),
        }
    }

    fn coverage_at(&self, table: Data<'_>, offset_position: usize) -> bool {
        self.coverage(table.sub16(offset_position)).unwrap_or(false)
    }

    fn coverage_array(&self, table: Data<'_>, count_position: usize) -> Option<(bool, usize)> {
        let count = table.u16(count_position)? as usize;
        let found = (0..count).any(|index| self.coverage_at(table, count_position + 2 + index * 2));
        Some((found, count_position + 2 + count * 2))
    }

    /// The class of `self.glyph` in a ClassDef, `0` when it is not listed.
    fn class_of(&self, class_def: Option<Data<'_>>) -> u16 {
        let Some(class_def) = class_def else {
            return 0;
        };
        match class_def.u16(0) {
            Some(1) => {
                let (Some(start), Some(count)) = (class_def.u16(2), class_def.u16(4)) else {
                    return 0;
                };
                self.glyph
                    .checked_sub(start)
                    .filter(|index| *index < count)
                    .and_then(|index| class_def.u16(6 + index as usize * 2))
                    .unwrap_or(0)
            }
            Some(2) => {
                let count = class_def.u16(2).unwrap_or(0) as usize;
                (0..count)
                    .find_map(|index| {
                        let record = 4 + index * 6;
                        let start = class_def.u16(record)?;
                        let end = class_def.u16(record + 2)?;
                        (start..=end)
                            .contains(&self.glyph)
                            .then(|| class_def.u16(record + 4))
                            .flatten()
                    })
                    .unwrap_or(0)
            }
            _ => 0,
        }
    }

    fn substitution(&self, kind: u16, table: Data<'_>) -> Option<bool> {
        match kind {
            // Single, Multiple and Alternate: the coverage is the input set.
            1..=3 => Some(self.coverage_at(table, 2)),
            // Ligature: coverage plus every component of every ligature.
            4 => {
                if self.coverage_at(table, 2) {
                    return Some(true);
                }
                let set_count = table.u16(4)? as usize;
                Some((0..set_count).any(|set_index| {
                    let Some(set) = table.sub16(6 + set_index * 2) else {
                        return false;
                    };
                    let count = set.u16(0).unwrap_or(0) as usize;
                    (0..count).any(|index| {
                        let Some(ligature) = set.sub16(2 + index * 2) else {
                            return false;
                        };
                        let components = ligature.u16(2).unwrap_or(0) as usize;
                        ligature
                            .u16_array(4, components.saturating_sub(1))
                            .any(|glyph| glyph == self.glyph)
                    })
                }))
            }
            5 => self.context(table),
            6 => self.chain_context(table),
            // ReverseChainSingleSubstFormat1.
            8 => {
                if self.coverage_at(table, 2) {
                    return Some(true);
                }
                let (backtrack, next) = self.coverage_array(table, 4)?;
                let (lookahead, _) = self.coverage_array(table, next)?;
                Some(backtrack || lookahead)
            }
            _ => Some(false),
        }
    }

    fn positioning(&self, kind: u16, table: Data<'_>) -> Option<bool> {
        match kind {
            1 | 3 => Some(self.coverage_at(table, 2)),
            2 => {
                if self.coverage_at(table, 2) {
                    return Some(true);
                }
                match table.u16(0)? {
                    1 => {
                        let format1 = table.u16(4)?;
                        let format2 = table.u16(6)?;
                        let record_size =
                            2 + 2 * (format1.count_ones() + format2.count_ones()) as usize;
                        let set_count = table.u16(8)? as usize;
                        Some((0..set_count).any(|set_index| {
                            let Some(set) = table.sub16(10 + set_index * 2) else {
                                return false;
                            };
                            let count = set.u16(0).unwrap_or(0) as usize;
                            (0..count).any(|index| set.u16(2 + index * record_size) == Some(self.glyph))
                        }))
                    }
                    // Every glyph with a nonzero second class (ClassDef::collect_coverage).
                    2 => Some(self.class_of(table.sub16(10)) != 0),
                    _ => Some(false),
                }
            }
            // MarkBase, MarkLig and MarkMark: both coverages.
            4..=6 => Some(self.coverage_at(table, 2) || self.coverage_at(table, 4)),
            7 => self.context(table),
            8 => self.chain_context(table),
            _ => Some(false),
        }
    }

    // SequenceContextFormat1-3 (GSUB 5 / GPOS 7).
    fn context(&self, table: Data<'_>) -> Option<bool> {
        match table.u16(0)? {
            1 => {
                if self.coverage_at(table, 2) {
                    return Some(true);
                }
                let set_count = table.u16(4)? as usize;
                Some((0..set_count).any(|set_index| {
                    self.rules(table.sub16(6 + set_index * 2), |rule| {
                        let glyphs = rule.u16(0).unwrap_or(0) as usize;
                        rule.u16_array(4, glyphs.saturating_sub(1)).any(|glyph| glyph == self.glyph)
                    })
                }))
            }
            2 => {
                if self.coverage_at(table, 2) {
                    return Some(true);
                }
                let class = self.class_of(table.sub16(4));
                if class == 0 {
                    return Some(false);
                }
                let set_count = table.u16(6)? as usize;
                // The first input glyph comes from the coverage; the rules
                // contribute the classes of the remaining input glyphs.
                Some((0..set_count).any(|set_index| {
                    self.rules(table.sub16(8 + set_index * 2), |rule| {
                        let glyphs = rule.u16(0).unwrap_or(0) as usize;
                        rule.u16_array(4, glyphs.saturating_sub(1)).any(|value| value == class)
                    })
                }))
            }
            3 => {
                let count = table.u16(2)? as usize;
                Some((0..count).any(|index| self.coverage_at(table, 6 + index * 2)))
            }
            _ => Some(false),
        }
    }

    // ChainedSequenceContextFormat1-3 (GSUB 6 / GPOS 8).
    fn chain_context(&self, table: Data<'_>) -> Option<bool> {
        match table.u16(0)? {
            1 => {
                if self.coverage_at(table, 2) {
                    return Some(true);
                }
                let set_count = table.u16(4)? as usize;
                Some((0..set_count).any(|set_index| {
                    self.rules(table.sub16(6 + set_index * 2), |rule| {
                        self.chain_rule_contains(rule, |value| value == self.glyph)
                    })
                }))
            }
            2 => {
                if self.coverage_at(table, 2) {
                    return Some(true);
                }
                let backtrack = self.class_of(table.sub16(4));
                let input = self.class_of(table.sub16(6));
                let lookahead = self.class_of(table.sub16(8));
                if backtrack == 0 && input == 0 && lookahead == 0 {
                    return Some(false);
                }
                let set_count = table.u16(10)? as usize;
                Some((0..set_count).any(|set_index| {
                    self.rules(table.sub16(12 + set_index * 2), |rule| {
                        self.chain_class_rule_contains(rule, backtrack, input, lookahead)
                    })
                }))
            }
            3 => {
                let (backtrack, next) = self.coverage_array(table, 2)?;
                let (input, next) = self.coverage_array(table, next)?;
                let (lookahead, _) = self.coverage_array(table, next)?;
                Some(backtrack || input || lookahead)
            }
            _ => Some(false),
        }
    }

    fn rules(&self, set: Option<Data<'_>>, mut contains: impl FnMut(Data<'_>) -> bool) -> bool {
        let Some(set) = set else {
            return false;
        };
        let count = set.u16(0).unwrap_or(0) as usize;
        (0..count).any(|index| set.sub16(2 + index * 2).is_some_and(&mut contains))
    }

    // ChainedSequenceRule: backtrack, input (without the first glyph), lookahead.
    fn chain_rule_contains(&self, rule: Data<'_>, matches: impl Fn(u16) -> bool) -> bool {
        let Some(backtrack) = rule.u16(0) else {
            return false;
        };
        let backtrack = backtrack as usize;
        if rule.u16_array(2, backtrack).any(&matches) {
            return true;
        }
        let input_position = 2 + backtrack * 2;
        let Some(input) = rule.u16(input_position) else {
            return false;
        };
        let input = (input as usize).saturating_sub(1);
        if rule.u16_array(input_position + 2, input).any(&matches) {
            return true;
        }
        let lookahead_position = input_position + 2 + input * 2;
        let lookahead = rule.u16(lookahead_position).unwrap_or(0) as usize;
        rule.u16_array(lookahead_position + 2, lookahead).any(&matches)
    }

    fn chain_class_rule_contains(&self, rule: Data<'_>, backtrack: u16, input: u16, lookahead: u16) -> bool {
        let Some(backtrack_count) = rule.u16(0) else {
            return false;
        };
        let backtrack_count = backtrack_count as usize;
        if backtrack != 0 && rule.u16_array(2, backtrack_count).any(|value| value == backtrack) {
            return true;
        }
        let input_position = 2 + backtrack_count * 2;
        let Some(input_count) = rule.u16(input_position) else {
            return false;
        };
        let input_count = (input_count as usize).saturating_sub(1);
        if input != 0 && rule.u16_array(input_position + 2, input_count).any(|value| value == input) {
            return true;
        }
        let lookahead_position = input_position + 2 + input_count * 2;
        let lookahead_count = rule.u16(lookahead_position).unwrap_or(0) as usize;
        lookahead != 0
            && rule
                .u16_array(lookahead_position + 2, lookahead_count)
                .any(|value| value == lookahead)
    }
}

/// The design-unit value of MATH constant `index` (HarfBuzz's
/// `hb_ot_math_constant_t` order) and whether it is a percentage, which
/// HarfBuzz returns unscaled. Device and variation adjustments apply only at
/// a nonzero ppem or a varied location in HarfBuzz; Blink's scaled font sets
/// neither for MATH queries, so they are not read here.
pub(crate) fn MathConstant(table: &[u8], index: u32) -> Option<(i32, bool)> {
    let table = Data(table);
    if table.u16(0) != Some(1) {
        return None;
    }
    let constants = table.sub16(4)?;
    match index {
        0 | 1 => Some((i32::from(constants.i16(index as usize * 2)?), true)),
        2 | 3 => Some((i32::from(constants.u16(index as usize * 2)?), false)),
        4..=54 => Some((i32::from(constants.i16(8 + (index as usize - 4) * 4)?), false)),
        55 => Some((i32::from(constants.i16(8 + 51 * 4)?), true)),
        _ => None,
    }
}

/// Whether a MATH table is present (`hb_ot_math_has_data`).
pub(crate) fn HasMathData(table: &[u8]) -> bool {
    Data(table).u16(0) == Some(1)
}
