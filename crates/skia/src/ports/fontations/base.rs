// Mapped from Skia src/ports/fontations/src/base.rs (Skia cacf77bdba7b).
// Changes: crate paths (crate:: -> super::), read_fonts/font_types reached
// through skrifa::raw, the cxx AxisWrapper as a Rust trait; tests omitted.
// Copyright 2023 Google LLC
// Use of this source code is governed by a BSD-style license that can be found
// in the LICENSE file.

use skrifa::raw::types::GlyphId;
use skrifa::raw::{FileRef, FontRef, ReadError, TableProvider};
use skrifa::{
    attribute::Style,
    charmap::MappingIndex,
    instance::{Location, Size},
    metrics::{GlyphMetrics, Metrics as SkrifaMetrics},
    outline::OutlineGlyphFormat,
    setting::VariationSetting,
    MetadataProvider, OutlineGlyphCollection, Tag,
};
use std::pin::Pin;

use super::ffi::{AxisWrapper, BridgeFontStyle, Metrics, OutlineFormat, SkiaDesignCoordinate};

pub struct BridgeFontRef<'a> {
    font: Option<FontRef<'a>>,
    has_any_color: bool,
}

impl<'a> BridgeFontRef<'a> {
    pub fn with_font<T>(&'a self, f: impl FnOnce(&'a FontRef) -> Option<T>) -> Option<T> {
        f(self.font.as_ref()?)
    }
}

#[derive(Default)]
pub struct BridgeOutlineCollection<'a>(pub Option<OutlineGlyphCollection<'a>>);

#[derive(Default)]
pub struct BridgeNormalizedCoords {
    pub normalized_coords: Location,
    filtered_user_coords: Vec<VariationSetting>,
}

pub struct BridgeMappingIndex(MappingIndex);

pub fn make_mapping_index<'a>(font_ref: &'a BridgeFontRef) -> Box<BridgeMappingIndex> {
    font_ref
        .with_font(|f| Some(Box::new(BridgeMappingIndex(MappingIndex::new(f)))))
        .unwrap()
}

pub fn lookup_glyph_or_zero(
    font_ref: &BridgeFontRef,
    map: &BridgeMappingIndex,
    codepoints: &[u32],
    glyphs: &mut [u16],
) {
    glyphs.fill(0);
    font_ref.with_font(|f| {
        let mappings = map.0.charmap(f);
        for it in codepoints.iter().zip(glyphs.iter_mut()) {
            let (codepoint, glyph) = it;
            // Remove u16 conversion when implementing large glyph id support in Skia.
            *glyph = u16::try_from(mappings.map(*codepoint).unwrap_or_default().to_u32())
                .unwrap_or_default();
        }
        Some(())
    });
}

pub fn num_glyphs(font_ref: &BridgeFontRef) -> u16 {
    font_ref
        .with_font(|f| Some(f.maxp().ok()?.num_glyphs()))
        .unwrap_or_default()
}

pub fn fill_glyph_to_unicode_map(font_ref: &BridgeFontRef, map: &mut [u32]) {
    map.fill(0);
    font_ref.with_font(|f| {
        let mappings = f.charmap().mappings();
        for (codepoint, glyphid) in mappings {
            if let Some(c) = map.get_mut(glyphid.to_u32() as usize).filter(|c| **c == 0) {
                *c = codepoint;
            }
        }
        Some(())
    });
}

pub fn unhinted_advance_width_or_zero(
    font_ref: &BridgeFontRef,
    size: f32,
    coords: &BridgeNormalizedCoords,
    glyph_id: u16,
) -> f32 {
    font_ref
        .with_font(|f| {
            GlyphMetrics::new(f, Size::new(size), coords.normalized_coords.coords())
                .advance_width(GlyphId::from(glyph_id))
        })
        .unwrap_or_default()
}

pub fn outline_format(outlines: &BridgeOutlineCollection) -> OutlineFormat {
    let outlines = outlines.0.as_ref();
    match outlines.and_then(|o| o.format()) {
        Some(OutlineGlyphFormat::Glyf) => OutlineFormat::Glyf,
        Some(OutlineGlyphFormat::Cff) => OutlineFormat::Cff,
        Some(OutlineGlyphFormat::Cff2) => OutlineFormat::Cff2,
        _ => OutlineFormat::NoOutlines,
    }
}

pub fn units_per_em_or_zero(font_ref: &BridgeFontRef) -> u16 {
    font_ref
        .with_font(|f| Some(f.head().ok()?.units_per_em()))
        .unwrap_or_default()
}

pub fn convert_metrics(skrifa_metrics: &SkrifaMetrics) -> Metrics {
    Metrics {
        top: skrifa_metrics.bounds.map_or(0.0, |b| b.y_max),
        bottom: skrifa_metrics.bounds.map_or(0.0, |b| b.y_min),
        x_min: skrifa_metrics.bounds.map_or(0.0, |b| b.x_min),
        x_max: skrifa_metrics.bounds.map_or(0.0, |b| b.x_max),
        ascent: skrifa_metrics.ascent,
        descent: skrifa_metrics.descent,
        leading: skrifa_metrics.leading,
        avg_char_width: skrifa_metrics.average_width.unwrap_or(0.0),
        max_char_width: skrifa_metrics.max_width.unwrap_or(0.0),
        x_height: -skrifa_metrics.x_height.unwrap_or(0.0),
        cap_height: -skrifa_metrics.cap_height.unwrap_or(0.0),
        underline_position: skrifa_metrics.underline.map_or(f32::NAN, |u| u.offset),
        underline_thickness: skrifa_metrics.underline.map_or(f32::NAN, |u| u.thickness),
        strikeout_position: skrifa_metrics.strikeout.map_or(f32::NAN, |s| s.offset),
        strikeout_thickness: skrifa_metrics.strikeout.map_or(f32::NAN, |s| s.thickness),
    }
}

pub fn get_skia_metrics(
    font_ref: &BridgeFontRef,
    size: f32,
    coords: &BridgeNormalizedCoords,
) -> Metrics {
    font_ref
        .with_font(|f| {
            let fontations_metrics =
                SkrifaMetrics::new(f, Size::new(size), coords.normalized_coords.coords());
            Some(convert_metrics(&fontations_metrics))
        })
        .unwrap_or_default()
}

pub fn get_unscaled_metrics(font_ref: &BridgeFontRef, coords: &BridgeNormalizedCoords) -> Metrics {
    font_ref
        .with_font(|f| {
            let fontations_metrics =
                SkrifaMetrics::new(f, Size::unscaled(), coords.normalized_coords.coords());
            Some(convert_metrics(&fontations_metrics))
        })
        .unwrap_or_default()
}

/// Implements the behavior expected for `SkTypeface::getTableData`, compare
/// documentation for this method and the FreeType implementation in Skia.
/// * If the target data array is empty, do not copy any data into it, but
///   return the size of the table.
/// * If the target data buffer is shorted than from offset to the end of the
///   table, truncate the data.
/// * If offset is longer than the table's length, return 0.
pub fn table_data(font_ref: &BridgeFontRef, tag: u32, offset: usize, data: &mut [u8]) -> usize {
    let table_data = font_ref
        .with_font(|f| f.table_data(Tag::from_be_bytes(tag.to_be_bytes())))
        .unwrap_or_default();
    let table_data = table_data.as_ref();
    // Remaining table data size measured from offset to end, or 0 if offset is
    // too large.
    let mut to_copy_length = table_data.len().saturating_sub(offset);
    match data.len() {
        0 => to_copy_length,
        _ => {
            to_copy_length = to_copy_length.min(data.len());
            let table_offset_data = table_data
                .get(offset..offset + to_copy_length)
                .unwrap_or_default();
            data.get_mut(..table_offset_data.len())
                .map_or(0, |data_slice| {
                    data_slice.copy_from_slice(table_offset_data);
                    data_slice.len()
                })
        }
    }
}

pub fn table_tags(font_ref: &BridgeFontRef, tags: &mut [u32]) -> u16 {
    font_ref
        .with_font(|f| {
            let table_directory = &f.table_directory;
            let table_tags_iter = table_directory
                .table_records()
                .iter()
                .map(|table| u32::from_be_bytes(table.tag.get().into_bytes()));
            tags.iter_mut()
                .zip(table_tags_iter)
                .for_each(|(out_tag, table_tag)| *out_tag = table_tag);
            Some(table_directory.num_tables())
        })
        .unwrap_or_default()
}

pub fn variation_position(
    coords: &BridgeNormalizedCoords,
    coordinates: &mut [SkiaDesignCoordinate],
) -> isize {
    if !coordinates.is_empty() {
        if coords.filtered_user_coords.len() > coordinates.len() {
            return -1;
        }
        let skia_design_coordinates =
            coords
                .filtered_user_coords
                .iter()
                .map(|setting| SkiaDesignCoordinate {
                    axis: u32::from_be_bytes(setting.selector.into_bytes()),
                    value: setting.value,
                });
        for (i, coord) in skia_design_coordinates.enumerate() {
            coordinates[i] = coord;
        }
    }
    coords.filtered_user_coords.len().try_into().unwrap()
}

pub fn coordinates_for_shifted_named_instance_index(
    font_ref: &BridgeFontRef,
    shifted_index: u32,
    coords: &mut [SkiaDesignCoordinate],
) -> isize {
    font_ref
        .with_font(|f| {
            let fvar = f.fvar().ok()?;
            let instances = fvar.instances().ok()?;
            let index: usize = ((shifted_index >> 16) - 1).try_into().unwrap();
            let instance_coords = instances.get(index).ok()?.coordinates;

            if coords.len() != 0 {
                if coords.len() < instance_coords.len() {
                    return None;
                }
                let axis_coords = f.axes().iter().zip(instance_coords.iter()).enumerate();
                for (i, axis_coord) in axis_coords {
                    coords[i] = SkiaDesignCoordinate {
                        axis: u32::from_be_bytes(axis_coord.0.tag().to_be_bytes()),
                        value: axis_coord.1.get().to_f32(),
                    };
                }
            }

            Some(instance_coords.len() as isize)
        })
        .unwrap_or(0)
}

pub fn num_axes(font_ref: &BridgeFontRef) -> usize {
    font_ref
        .with_font(|f| Some(f.axes().len()))
        .unwrap_or_default()
}

pub fn populate_axes(font_ref: &BridgeFontRef, axis_wrapper: &mut dyn AxisWrapper) -> isize {
    font_ref
        .with_font(|f| {
            let axes = f.axes();
            // Populate incoming allocated SkFontParameters::Variation::Axis[] only when a
            // buffer is passed.
            if axis_wrapper.size() > 0 {
                for (i, axis) in axes.iter().enumerate() {
                    if !axis_wrapper.populate_axis(
                        i,
                        u32::from_be_bytes(axis.tag().into_bytes()),
                        axis.min_value(),
                        axis.default_value(),
                        axis.max_value(),
                        axis.is_hidden(),
                    ) {
                        return None;
                    }
                }
            }
            isize::try_from(axes.len()).ok()
        })
        .unwrap_or(-1)
}

fn make_font_ref_internal<'a>(font_data: &'a [u8], index: u32) -> Result<FontRef<'a>, ReadError> {
    match FileRef::new(font_data) {
        Ok(file_ref) => match file_ref {
            FileRef::Font(font_ref) => {
                // Indices with the higher bits set are meaningful here and do not result in an
                // error, as they may refer to a named instance and are taken into account by the
                // Fontations typeface implementation,
                // compare `coordinates_for_shifted_named_instance_index()`.
                if index & 0xFFFF > 0 {
                    Err(ReadError::InvalidCollectionIndex(index))
                } else {
                    Ok(font_ref)
                }
            }
            FileRef::Collection(collection) => collection.get(index),
        },
        Err(e) => Err(e),
    }
}

pub fn make_font_ref<'a>(font_data: &'a [u8], index: u32) -> Box<BridgeFontRef<'a>> {
    let font = make_font_ref_internal(font_data, index).ok();
    let has_any_color = font
        .as_ref()
        .map(|f| {
            f.cbdt().is_ok() ||
            f.sbix().is_ok() ||
            // ColorGlyphCollection::get_with_format() first thing checks for presence of colr(),
            // so we do the same:
            f.colr().is_ok() ||
            f.ebdt().is_ok()
        })
        .unwrap_or_default();

    Box::new(BridgeFontRef {
        font,
        has_any_color,
    })
}

pub fn font_ref_is_valid(bridge_font_ref: &BridgeFontRef) -> bool {
    bridge_font_ref.font.is_some()
}

pub fn has_any_color_table(bridge_font_ref: &BridgeFontRef) -> bool {
    bridge_font_ref.has_any_color
}

pub fn get_outline_collection<'a>(
    font_ref: &'a BridgeFontRef<'a>,
) -> Box<BridgeOutlineCollection<'a>> {
    Box::new(
        font_ref
            .with_font(|f| Some(BridgeOutlineCollection(Some(f.outline_glyphs()))))
            .unwrap_or_default(),
    )
}

pub fn font_or_collection<'a>(font_data: &'a [u8], num_fonts: &mut u32) -> bool {
    match FileRef::new(font_data) {
        Ok(FileRef::Collection(collection)) => {
            *num_fonts = collection.len();
            true
        }
        Ok(FileRef::Font(_)) => {
            *num_fonts = 0u32;
            true
        }
        _ => false,
    }
}

pub fn num_named_instances(font_ref: &BridgeFontRef) -> usize {
    font_ref
        .with_font(|f| Some(f.named_instances().len()))
        .unwrap_or_default()
}

pub fn resolve_into_normalized_coords(
    font_ref: &BridgeFontRef,
    design_coords: &[SkiaDesignCoordinate],
) -> Box<BridgeNormalizedCoords> {
    let variation_tuples = design_coords
        .iter()
        .map(|coord| (Tag::from_be_bytes(coord.axis.to_be_bytes()), coord.value));
    let bridge_normalized_coords = font_ref
        .with_font(|f| {
            let merged_defaults_with_user = f
                .axes()
                .iter()
                .map(|axis| (axis.tag(), axis.default_value()))
                .chain(design_coords.iter().map(|user_coord| {
                    (
                        Tag::from_be_bytes(user_coord.axis.to_be_bytes()),
                        user_coord.value,
                    )
                }));
            Some(BridgeNormalizedCoords {
                filtered_user_coords: f.axes().filter(merged_defaults_with_user).collect(),
                normalized_coords: f.axes().location(variation_tuples),
            })
        })
        .unwrap_or_default();
    Box::new(bridge_normalized_coords)
}

pub fn normalized_coords_equal(a: &BridgeNormalizedCoords, b: &BridgeNormalizedCoords) -> bool {
    a.normalized_coords.coords() == b.normalized_coords.coords()
}

#[allow(non_upper_case_globals)]
pub fn get_font_style(
    font_ref: &BridgeFontRef,
    coords: &BridgeNormalizedCoords,
    style: &mut BridgeFontStyle,
) -> bool {
    const SKIA_SLANT_UPRIGHT: i32 = 0; /* kUpright_Slant */
    const SKIA_SLANT_ITALIC: i32 = 1; /* kItalic_Slant */
    const SKIA_SLANT_OBLIQUE: i32 = 2; /* kOblique_Slant */

    font_ref
        .with_font(|f| {
            let attrs = f.attributes();
            let mut skia_weight = attrs.weight.value().round() as i32;
            let mut skia_slant = match attrs.style {
                Style::Normal => SKIA_SLANT_UPRIGHT,
                Style::Italic => SKIA_SLANT_ITALIC,
                _ => SKIA_SLANT_OBLIQUE,
            };
            //0.5, 0.625, 0.75, 0.875, 1.0, 1.125, 1.25, 1.5, 2.0 map to 1-9
            let mut skia_width = match attrs.stretch.ratio() {
                x if x <= 0.5625 => 1,
                x if x <= 0.6875 => 2,
                x if x <= 0.8125 => 3,
                x if x <= 0.9375 => 4,
                x if x <= 1.0625 => 5,
                x if x <= 1.1875 => 6,
                x if x <= 1.3750 => 7,
                x if x <= 1.7500 => 8,
                _ => 9,
            };

            const wght: Tag = Tag::new(b"wght");
            const wdth: Tag = Tag::new(b"wdth");
            const slnt: Tag = Tag::new(b"slnt");
            const ital: Tag = Tag::new(b"ital");

            let mut slnt_value = None;
            let mut ital_value = None;
            for user_coord in coords.filtered_user_coords.iter() {
                match user_coord.selector {
                    wght => skia_weight = user_coord.value.round() as i32,
                    // 50, 62.5, 75, 87.5, 100, 112.5, 125, 150, 200 map to 1-9
                    wdth => {
                        skia_width = match user_coord.value {
                            x if x <= 56.25 => 1,
                            x if x <= 68.75 => 2,
                            x if x <= 81.25 => 3,
                            x if x <= 93.75 => 4,
                            x if x <= 106.25 => 5,
                            x if x <= 118.75 => 6,
                            x if x <= 137.50 => 7,
                            x if x <= 175.00 => 8,
                            _ => 9,
                        }
                    }
                    slnt => slnt_value = Some(user_coord.value),
                    ital => ital_value = Some(user_coord.value),
                    _ => (),
                }
            }
            // Value > 0 => +, value == 0 => 0, no value => _
            // slnt\ital  _      0      +
            //       _   init  !ital   ital
            //       0  !oblq   uprt   ital
            //       +   oblq   oblq   ital
            if ital_value.is_some_and(|x| x != 0.0) {
                skia_slant = SKIA_SLANT_ITALIC;
            } else if slnt_value.is_some_and(|x| x != 0.0) {
                skia_slant = SKIA_SLANT_OBLIQUE;
            } else if ital_value.is_some_and(|x| x == 0.0) && slnt_value.is_some_and(|x| x == 0.0) {
                skia_slant = SKIA_SLANT_UPRIGHT;
            } else if ital_value.is_some_and(|x| x == 0.0) && slnt_value.is_none() {
                if skia_slant == SKIA_SLANT_ITALIC {
                   skia_slant = SKIA_SLANT_UPRIGHT;
                }
            } else if ital_value.is_none() && slnt_value.is_some_and(|x| x == 0.0) {
                if skia_slant == SKIA_SLANT_OBLIQUE {
                   skia_slant = SKIA_SLANT_UPRIGHT;
                }
            }

            *style = BridgeFontStyle {
                weight: skia_weight,
                slant: skia_slant,
                width: skia_width,
            };
            Some(true)
        })
        .unwrap_or_default()
}

pub fn is_embeddable(font_ref: &BridgeFontRef) -> bool {
    font_ref
        .with_font(|f| {
            let fs_type = f.os2().ok()?.fs_type();
            // https://learn.microsoft.com/en-us/typography/opentype/spec/os2#fstype
            // Bit 2 and bit 9 must be cleared, "Restricted License embedding" and
            // "Bitmap embedding only" must both be unset.
            // Implemented to match SkTypeface_FreeType::onGetAdvancedMetrics.
            Some(fs_type & 0x202 == 0)
        })
        .unwrap_or(true)
}

pub fn is_subsettable(font_ref: &BridgeFontRef) -> bool {
    font_ref
        .with_font(|f| {
            let fs_type = f.os2().ok()?.fs_type();
            // https://learn.microsoft.com/en-us/typography/opentype/spec/os2#fstype
            Some((fs_type & 0x100) == 0)
        })
        .unwrap_or(true)
}

pub fn is_fixed_pitch(font_ref: &BridgeFontRef) -> bool {
    font_ref
        .with_font(|f| {
            // Compare DWriteFontTypeface::onGetAdvancedMetrics().
            Some(f.post().ok()?.is_fixed_pitch() != 0 || f.hhea().ok()?.number_of_h_metrics() == 1)
        })
        .unwrap_or_default()
}

pub fn is_serif_style(font_ref: &BridgeFontRef) -> bool {
    const FAMILY_TYPE_TEXT_AND_DISPLAY: u8 = 2;
    const SERIF_STYLE_COVE: u8 = 2;
    const SERIF_STYLE_TRIANGLE: u8 = 10;
    font_ref
        .with_font(|f| {
            // Compare DWriteFontTypeface::onGetAdvancedMetrics().
            let panose = f.os2().ok()?.panose_10();
            let family_type = panose[0];

            match family_type {
                FAMILY_TYPE_TEXT_AND_DISPLAY => {
                    let serif_style = panose[1];
                    Some((SERIF_STYLE_COVE..=SERIF_STYLE_TRIANGLE).contains(&serif_style))
                }
                _ => None,
            }
        })
        .unwrap_or_default()
}

pub fn is_script_style(font_ref: &BridgeFontRef) -> bool {
    const FAMILY_TYPE_SCRIPT: u8 = 3;
    font_ref
        .with_font(|f| {
            // Compare DWriteFontTypeface::onGetAdvancedMetrics().
            let family_type = f.os2().ok()?.panose_10()[0];
            Some(family_type == FAMILY_TYPE_SCRIPT)
        })
        .unwrap_or_default()
}

pub fn italic_angle(font_ref: &BridgeFontRef) -> i32 {
    font_ref
        .with_font(|f| Some(f.post().ok()?.italic_angle().to_i32()))
        .unwrap_or_default()
}
