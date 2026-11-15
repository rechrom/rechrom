//! Translation of src/ports/SkTypeface_fontations.cpp and
//! SkTypeface_fontations_priv.h (Skia cacf77bdba7b): the SkTypeface backed by
//! the Fontations Rust libraries (skrifa / read-fonts), through the bridge in
//! src/ports/fontations.
//!
//! Phase 1 maps outline glyphs: typeface identity, metrics, names, tables,
//! variations and the scaler context's outline/hinting path. COLRv0/v1 and
//! embedded-bitmap glyphs (colr.rs, bitmap.rs, ColorPainter, BoundsPainter)
//! are not mapped yet; until then such glyphs take the outline path.
#![allow(non_snake_case)]

use crate::include::core::SkFontArguments::{Coordinate, SkFontArguments, VariationPosition};
use crate::include::core::SkFontMetrics::SkFontMetrics;
use crate::include::core::SkFontParameters::Axis;
use crate::include::core::SkFontStyle::{SkFontStyle, Slant};
use crate::include::core::SkFontTypes::SkFontHinting;
use crate::src::core::SkData::SkData;
use crate::src::core::SkGlyph::SkGlyph;
use crate::src::core::SkMask::Format;
use crate::src::core::SkMatrix::SkMatrix;
use crate::src::core::SkPath::SkPath;
use crate::src::core::SkPoint::SkPoint;
use crate::src::core::SkScalerContext::{
    flags, GeneratedPath, GlyphMetrics, PreMatrixScale, SkScalerContext, SkScalerContextRec,
};
use crate::src::core::SkTypeface::{SkTypeface, SkTypefaceRef, SkTypefaceVirtuals};
use crate::src::ports::fontations as fontations_ffi;
use fontations_ffi::ffi::{
    AutoHintingControl, AxisWrapper as _, BridgeFontStyle, BridgeLocalizedName,
    BridgeScalerMetrics, FfiPoint, OutlineFormat, SkiaDesignCoordinate,
};
use fontations_ffi::{
    BridgeFontRef, BridgeGlyphStyles, BridgeHintingInstance, BridgeLocalizedStrings,
    BridgeMappingIndex, BridgeNormalizedCoords, BridgeOutlineCollection,
};
use std::ffi::c_void;
use std::ptr::NonNull;
use std::sync::{Arc, Mutex, OnceLock};

const fn SkSetFourByteTag(a: u8, b: u8, c: u8, d: u8) -> u32 {
    u32::from_be_bytes([a, b, c, d])
}

// Source: src/ports/SkTypeface_fontations.cpp:49-49
const kSkShowTextBlitCoverage: bool = false;

// Source: src/ports/SkTypeface_fontations.cpp:51-64
fn streamToData(
    font_data: Option<std::pin::Pin<Box<dyn crate::src::core::SkStream::SkStreamAsset>>>,
) -> Arc<SkData> {
    let Some(mut font_data) = font_data else {
        return SkData::MakeEmpty();
    };
    // TODO(drott): Remove this once SkData::MakeFromStream is able to do this itself.
    if let Some(data) = font_data.getData() {
        return data;
    }
    use crate::src::core::SkStream::{SkStreamAsset, SkStreamRewindable};
    let length = SkStreamAsset::getLength(&*font_data);
    let base = font_data.as_mut().getMemoryBase();
    if !base.is_null() && length != 0 {
        return unsafe { SkData::MakeWithCopy(base, length) };
    }
    SkStreamRewindable::rewind(font_data.as_mut());
    SkData::MakeFromStream(font_data.as_mut(), length).unwrap_or_else(SkData::MakeEmpty)
}

/// The font bytes of an SkData that the typeface keeps alive for as long as
/// every bridge object borrowing them (the source's sk_sp<const SkData>
/// fFontData outliving fBridgeFontRef).
unsafe fn data_bytes(data: &Arc<SkData>) -> &'static [u8] {
    if data.size() == 0 {
        return &[];
    }
    unsafe { std::slice::from_raw_parts(data.bytes(), data.size()) }
}

// Source: src/ports/SkTypeface_fontations.cpp:66-70
unsafe fn make_bridge_font_ref(font_data: &Arc<SkData>, index: u32) -> Box<BridgeFontRef<'static>> {
    fontations_ffi::make_font_ref(unsafe { data_bytes(font_data) }, index)
}

// Source: src/ports/SkTypeface_fontations.cpp:72-83: SkiaDesignCoordinate and
// VariationPosition::Coordinate have the same layout.
const _: () = assert!(
    core::mem::size_of::<SkiaDesignCoordinate>() == core::mem::size_of::<Coordinate>()
        && core::mem::offset_of!(SkiaDesignCoordinate, axis) == core::mem::offset_of!(Coordinate, axis)
        && core::mem::offset_of!(SkiaDesignCoordinate, value) == core::mem::offset_of!(Coordinate, value)
);

fn coordinates(position: &VariationPosition) -> &[SkiaDesignCoordinate] {
    if position.coordinates.is_null() || position.coordinateCount <= 0 {
        return &[];
    }
    // Cast is safe because of the layout assertion above.
    unsafe {
        std::slice::from_raw_parts(
            position.coordinates.cast::<SkiaDesignCoordinate>(),
            position.coordinateCount as usize,
        )
    }
}

// Source: src/ports/SkTypeface_fontations.cpp:85-94
fn make_normalized_coords(
    bridgeFontRef: &BridgeFontRef,
    variationPosition: &VariationPosition,
) -> Box<BridgeNormalizedCoords> {
    fontations_ffi::resolve_into_normalized_coords(bridgeFontRef, coordinates(variationPosition))
}

// Source: src/ports/SkTypeface_fontations.cpp:108-108
fn isLCD(rec: &SkScalerContextRec) -> bool {
    Format::kLCD16_Format == rec.fMaskFormat
}

// Source: src/ports/SkTypeface_fontations.cpp:110-110
fn bothZero(a: f32, b: f32) -> bool {
    0.0 == a && 0.0 == b
}

// Source: src/ports/SkTypeface_fontations.cpp:112-115
fn isAxisAligned(rec: &SkScalerContextRec) -> bool {
    0.0 == rec.fPreSkewX
        && (bothZero(rec.fPost2x2[0][1], rec.fPost2x2[1][0])
            || bothZero(rec.fPost2x2[0][0], rec.fPost2x2[1][1]))
}

// Source: src/ports/SkTypeface_fontations.cpp:119-122
pub fn SkTypeface_Make_Fontations_FromStream(
    fontData: Option<std::pin::Pin<Box<dyn crate::src::core::SkStream::SkStreamAsset>>>,
    args: &SkFontArguments,
) -> Option<SkTypefaceRef> {
    SkTypeface_Fontations::MakeFromStream(fontData, args)
}

// Source: src/ports/SkTypeface_fontations.cpp:124-127
pub fn SkTypeface_Make_Fontations(fontData: Arc<SkData>, args: &SkFontArguments) -> Option<SkTypefaceRef> {
    SkTypeface_Fontations::MakeFromData(fontData, args)
}

// Source: src/ports/SkTypeface_fontations_priv.h:188-259
// Fields that borrow the font bytes are declared before the bridge font ref and
// the data, so Rust's declaration-order drop releases them first.
pub struct SkTypeface_Fontations {
    fPalette: Vec<u32>,
    fGlyphStyles: Box<BridgeGlyphStyles>,
    fOutlines: Box<BridgeOutlineCollection<'static>>,
    fBridgeNormalizedCoords: Box<BridgeNormalizedCoords>,
    fMappingIndex: Box<BridgeMappingIndex>,
    // fBridgeFontRef accesses the data in fFontData. fFontData needs to be kept around for the
    // lifetime of fBridgeFontRef to safely request parsed data.
    fBridgeFontRef: Box<BridgeFontRef<'static>>,
    // Incoming ttc index requested when this typeface was instantiated from data.
    fTtcIndex: u32,
    fFontData: Arc<SkData>,
    fGlyphMasksMayNeedCurrentColor: OnceLock<bool>,
}

// The bridge objects are immutable after construction (skrifa's lazily
// initialized caches are synchronized), as the source relies on when sharing
// the typeface across threads.
unsafe impl Send for SkTypeface_Fontations {}
unsafe impl Sync for SkTypeface_Fontations {}

impl SkTypeface_Fontations {
    // Source: src/ports/SkTypeface_fontations_priv.h:212-212
    pub const FactoryId: u32 = SkSetFourByteTag(b'f', b'n', b't', b'a');

    // Source: src/ports/SkTypeface_fontations_priv.h:201-210
    pub fn getBridgeFontRef(&self) -> &BridgeFontRef<'static> {
        &self.fBridgeFontRef
    }
    pub fn getBridgeNormalizedCoords(&self) -> &BridgeNormalizedCoords {
        &self.fBridgeNormalizedCoords
    }
    pub fn getOutlines(&self) -> &BridgeOutlineCollection<'static> {
        &self.fOutlines
    }
    pub fn getGlyphStyles(&self) -> &BridgeGlyphStyles {
        &self.fGlyphStyles
    }
    pub fn getMappingIndex(&self) -> &BridgeMappingIndex {
        &self.fMappingIndex
    }
    pub fn getPalette(&self) -> &[u32] {
        &self.fPalette
    }

    // Source: src/ports/SkTypeface_fontations.cpp:157-160
    pub fn MakeFromStream(
        stream: Option<std::pin::Pin<Box<dyn crate::src::core::SkStream::SkStreamAsset>>>,
        args: &SkFontArguments,
    ) -> Option<SkTypefaceRef> {
        Self::MakeFromData(streamToData(stream), args)
    }

    // Source: src/ports/SkTypeface_fontations.cpp:162-237
    pub fn MakeFromData(data: Arc<SkData>, args: &SkFontArguments) -> Option<SkTypefaceRef> {
        let ttcIndex = (args.getCollectionIndex() as u32) & 0xFFFF;
        let bridgeFontRef = unsafe { make_bridge_font_ref(&data, ttcIndex) };
        if !fontations_ffi::font_ref_is_valid(&bridgeFontRef) {
            return None;
        }

        let mappingIndex = fontations_ffi::make_mapping_index(&bridgeFontRef);

        let mut variationPosition = args.getVariationDesignPosition();
        let mut concatenatedCoords: Vec<Coordinate> = Vec::new();
        // Handle FreeType behaviour of upper 15 bits of collection index
        // representing a named instance choice. If so, prepopulate the variation
        // coordinates with the values from the named instance and append the user
        // coordinates after that so they can override the named instance's
        // coordinates.
        if (args.getCollectionIndex() as u32) & 0xFFFF0000 != 0 {
            let numNamedInstanceCoords = fontations_ffi::coordinates_for_shifted_named_instance_index(
                &bridgeFontRef,
                args.getCollectionIndex() as u32,
                &mut [],
            )
            .max(0) as usize;
            let mut namedCoords = vec![SkiaDesignCoordinate::default(); numNamedInstanceCoords];
            let retrievedNamedInstanceCoords =
                fontations_ffi::coordinates_for_shifted_named_instance_index(
                    &bridgeFontRef,
                    args.getCollectionIndex() as u32,
                    &mut namedCoords,
                );
            if numNamedInstanceCoords as isize != retrievedNamedInstanceCoords {
                return None;
            }
            concatenatedCoords.extend(
                namedCoords
                    .iter()
                    .map(|coord| Coordinate { axis: coord.axis, value: coord.value }),
            );
            concatenatedCoords.extend(coordinates(&variationPosition).iter().map(|coord| Coordinate {
                axis: coord.axis,
                value: coord.value,
            }));
            variationPosition.coordinateCount = concatenatedCoords.len() as i32;
            variationPosition.coordinates = concatenatedCoords.as_ptr();
        }

        let normalizedCoords = make_normalized_coords(&bridgeFontRef, &variationPosition);
        let mut style = SkFontStyle::default();
        let mut fontStyle = BridgeFontStyle::default();
        if fontations_ffi::get_font_style(&bridgeFontRef, &normalizedCoords, &mut fontStyle) {
            let slant = match fontStyle.slant {
                1 => Slant::kItalic_Slant,
                2 => Slant::kOblique_Slant,
                _ => Slant::kUpright_Slant,
            };
            style = SkFontStyle::new(fontStyle.weight, fontStyle.width, slant);
        }
        // The outline collection borrows the boxed font ref, whose address is
        // stable and which the typeface keeps alive (see field order).
        let fontRefStatic: &'static BridgeFontRef<'static> =
            unsafe { &*(&*bridgeFontRef as *const BridgeFontRef<'static>) };
        let outlines = fontations_ffi::get_outline_collection(fontRefStatic);

        // Container for precomputed work for autohinting, see Skrifa's GlyphStyles.
        let glyphStyles = fontations_ffi::get_bridge_glyph_styles();

        // Source: src/ports/SkTypeface_fontations.cpp:222-226: resolve_palette
        // (colr.rs) is not mapped yet; outline glyphs use no palette.
        let palette: Vec<u32> = Vec::new();

        Some(crate::include::core::SkRefCnt::sk_sp::new(SkTypeface::new(
            style,
            true,
            Box::new(SkTypeface_Fontations {
                fPalette: palette,
                fGlyphStyles: glyphStyles,
                fOutlines: outlines,
                fBridgeNormalizedCoords: normalizedCoords,
                fMappingIndex: mappingIndex,
                fBridgeFontRef: bridgeFontRef,
                fTtcIndex: ttcIndex,
                fFontData: data,
                fGlyphMasksMayNeedCurrentColor: OnceLock::new(),
            }),
        )))
    }
}

// Source: src/ports/SkTypeface_fontations_priv.h:34-46 and
// src/ports/SkTypeface_fontations.cpp:239-260
struct AxisWrapperImpl<'a> {
    fAxisArray: &'a mut [Axis],
}

impl fontations_ffi::ffi::AxisWrapper for AxisWrapperImpl<'_> {
    fn populate_axis(&mut self, i: usize, axisTag: u32, min: f32, def: f32, max: f32, hidden: bool) -> bool {
        if i >= self.fAxisArray.len() {
            return false;
        }
        let axis = &mut self.fAxisArray[i];
        axis.tag = axisTag;
        axis.min = min;
        axis.def = def;
        axis.max = max;
        axis.setHidden(hidden);
        true
    }
    fn size(&self) -> usize {
        self.fAxisArray.len()
    }
}

// Source: src/ports/SkTypeface_fontations.cpp:333-352
struct SkrifaLocalizedStrings {
    fBridgeLocalizedStrings: Box<BridgeLocalizedStrings<'static>>,
}

impl crate::src::core::SkTypeface::LocalizedStrings for SkrifaLocalizedStrings {
    fn next(&mut self, localized_string: &mut crate::src::core::SkTypeface::LocalizedString) -> bool {
        let mut localizedName = BridgeLocalizedName::default();
        if !fontations_ffi::localized_name_next(&mut self.fBridgeLocalizedStrings, &mut localizedName) {
            return false;
        }
        localized_string.fString = localizedName.string;
        localized_string.fLanguage = localizedName.language;
        true
    }
}

impl SkTypefaceVirtuals for SkTypeface_Fontations {
    // Source: src/ports/SkTypeface_fontations.cpp:992-1041
    unsafe fn onMakeClone(&self, owner: &SkTypefaceRef, args: &SkFontArguments) -> Option<SkTypefaceRef> {
        // Matching DWrite implementation, return self if ttc index mismatches.
        if self.fTtcIndex != args.getCollectionIndex() as u32 {
            return Some(owner.clone());
        }

        let numAxes = unsafe { self.onGetVariationDesignPosition(std::ptr::null_mut(), 0) };
        let mut fusedDesignPosition = vec![Coordinate { axis: 0, value: 0.0 }; numAxes.max(0) as usize];
        let retrievedAxes = unsafe {
            self.onGetVariationDesignPosition(fusedDesignPosition.as_mut_ptr(), fusedDesignPosition.len())
        };
        if numAxes != retrievedAxes {
            return None;
        }

        // We know the internally retrieved axes are normalized, contain a value for every possible
        // axis, other axes do not exist, so we only need to override any of those.
        let argPosition = args.getVariationDesignPosition();
        for coordinate in fusedDesignPosition.iter_mut() {
            for argCoordinate in coordinates(&argPosition) {
                if coordinate.axis == argCoordinate.axis {
                    coordinate.value = argCoordinate.value;
                }
            }
        }

        let mut fusedArgs = SkFontArguments::new();
        fusedArgs.setVariationDesignPosition(VariationPosition {
            coordinates: fusedDesignPosition.as_ptr(),
            coordinateCount: numAxes,
        });
        fusedArgs.setPalette(args.getPalette());

        let normalized_args =
            make_normalized_coords(&self.fBridgeFontRef, &fusedArgs.getVariationDesignPosition());

        if !fontations_ffi::normalized_coords_equal(&normalized_args, &self.fBridgeNormalizedCoords) {
            return Self::MakeFromData(self.fFontData.clone(), &fusedArgs);
        }

        // TODO(skbug.com/330149870): Palette differences are not fused, see DWrite backend impl.
        // resolve_palette (colr.rs) is not mapped yet, so the palette is always empty here.
        Some(owner.clone())
    }

    // Source: src/ports/SkTypeface_fontations.cpp:1043-1057
    unsafe fn onCreateScalerContext(
        &self,
        typeface: NonNull<SkTypeface>,
        effects: &crate::src::core::SkScalerContext::SkScalerContextEffects<'_>,
        desc: &crate::src::core::SkDescriptor::SkDescriptor,
        path_owner: &crate::include::core::SkRefCnt::sk_sp<dyn crate::src::core::SkPathEffect::SkPathEffect>,
        mask_owner: Option<&Arc<dyn crate::src::core::SkMaskFilter::SkMaskFilter>>,
    ) -> Box<dyn crate::src::core::SkScalerContext::SkScalerContextVirtual> {
        Box::new(unsafe {
            SkFontationsScalerContext::new(self, effects, desc, typeface, path_owner, mask_owner)
        })
    }

    // Source: src/ports/SkTypeface_fontations.cpp:310-331
    fn onFilterRec(&self, rec: &mut SkScalerContextRec) {
        rec.useStrokeForFakeBold();

        // See https://issues.skia.org/issues/396360753
        // We would like Fontations anti-aliasing on a surface with unknown pixel geometry to
        // look like the FreeType backend in order to avoid perceived regressions
        // in sharpness, so we ignore SkScalerContext::kGenA8FromLCD_Flag in fRec.fFlags.
        rec.fFlags &= !flags::GEN_A8_FROM_LCD;

        // Opportunistic hinting downgrades copied from SkFontHost_FreeType.cpp
        let mut h = rec.get_hinting();
        if SkFontHinting::kFull == h && !isLCD(rec) {
            // Collapse full->normal hinting if we're not doing LCD.
            h = SkFontHinting::kNormal;
        }

        // Rotated text looks bad with hinting, so we disable it as needed.
        if !isAxisAligned(rec) {
            h = SkFontHinting::kNone;
        }
        rec.set_hinting(h);
    }

    // Source: src/ports/SkTypeface_fontations.cpp:1059-1120
    fn onGetAdvancedMetrics(&self) -> Option<Box<crate::src::core::SkAdvancedTypefaceMetrics::SkAdvancedTypefaceMetrics>> {
        use crate::src::core::SkAdvancedTypefaceMetrics::{FontFlags, FontType, SkAdvancedTypefaceMetrics, StyleFlags};
        let mut info = Box::new(SkAdvancedTypefaceMetrics::default());

        if !fontations_ffi::is_embeddable(&self.fBridgeFontRef) {
            info.fFlags |= FontFlags::kNotEmbeddable_FontFlag;
        }

        if !fontations_ffi::is_subsettable(&self.fBridgeFontRef) {
            info.fFlags |= FontFlags::kNotSubsettable_FontFlag;
        }

        if fontations_ffi::table_data(&self.fBridgeFontRef, SkSetFourByteTag(b'f', b'v', b'a', b'r'), 0, &mut []) != 0 {
            info.fFlags |= FontFlags::kVariable_FontFlag;
        }

        match fontations_ffi::outline_format(&self.fOutlines) {
            OutlineFormat::Glyf => info.fType = FontType::kTrueType_Font,
            OutlineFormat::Cff => info.fType = FontType::kCFF_Font,
            // leave info->fType SkAdvancedTypefaceMetrics::kOther;
            _ => {}
        }

        // Metrics information.
        let metrics = fontations_ffi::get_unscaled_metrics(&self.fBridgeFontRef, &self.fBridgeNormalizedCoords);
        info.fAscent = metrics.ascent as i16;
        info.fDescent = metrics.descent as i16;
        info.fCapHeight = metrics.cap_height as i16;

        info.fBBox = crate::src::core::SkRect::SkIRect::MakeLTRB(
            metrics.x_min as i32,
            metrics.top as i32,
            metrics.x_max as i32,
            metrics.bottom as i32,
        );

        // Style information.
        if fontations_ffi::is_fixed_pitch(&self.fBridgeFontRef) {
            info.fStyle |= StyleFlags::kFixedPitch_Style;
        }

        let mut fontStyle = BridgeFontStyle::default();
        if fontations_ffi::get_font_style(&self.fBridgeFontRef, &self.fBridgeNormalizedCoords, &mut fontStyle)
            && fontStyle.slant == 1
        // SkFontStyle::Slant::kItalic_Slant
        {
            info.fStyle |= StyleFlags::kItalic_Style;
        }

        if fontations_ffi::is_serif_style(&self.fBridgeFontRef) {
            info.fStyle |= StyleFlags::kSerif_Style;
        } else if fontations_ffi::is_script_style(&self.fBridgeFontRef) {
            info.fStyle |= StyleFlags::kScript_Style;
        }

        info.fItalicAngle = fontations_ffi::italic_angle(&self.fBridgeFontRef) as i16;

        Some(info)
    }

    // Source: src/ports/SkTypeface_fontations_priv.h:231-231
    fn getPostScriptGlyphNames(&self, _: &mut [crate::src::core::SkString::SkString]) {}

    // Source: src/ports/SkTypeface_fontations.cpp:303-308
    fn getGlyphToUnicodeMap(&self, codepointForGlyphMap: &mut [i32]) {
        let numGlyphs = (self.onCountGlyphs().max(0) as usize).min(codepointForGlyphMap.len());
        let codepointForGlyphSlice = unsafe {
            std::slice::from_raw_parts_mut(codepointForGlyphMap.as_mut_ptr().cast::<u32>(), numGlyphs)
        };
        fontations_ffi::fill_glyph_to_unicode_map(&self.fBridgeFontRef, codepointForGlyphSlice);
    }

    // Source: src/ports/SkTypeface_fontations.cpp:987-990
    fn onOpenStream(
        &self,
        _owner: &SkTypeface,
        ttcIndex: &mut i32,
    ) -> Option<std::pin::Pin<Box<dyn crate::src::core::SkStream::SkStreamAsset>>> {
        *ttcIndex = self.fTtcIndex as i32;
        Some(Box::pin(crate::src::core::SkStream::SkMemoryStream::from_data(Some(self.fFontData.clone()))))
    }

    // Source: src/ports/SkTypeface_fontations.cpp:283-289
    fn onGlyphMaskNeedsCurrentColor(&self) -> bool {
        *self.fGlyphMasksMayNeedCurrentColor.get_or_init(|| {
            const COLRTag: u32 = SkSetFourByteTag(b'C', b'O', b'L', b'R');
            // getTableSize(COLRTag) > 0
            fontations_ffi::table_data(&self.fBridgeFontRef, COLRTag, 0, &mut []) > 0
        })
    }

    // Source: src/ports/SkTypeface_fontations.cpp:1164-1173
    unsafe fn onGetVariationDesignPosition(&self, coordinates: *mut Coordinate, capacity: usize) -> i32 {
        let copyToCoordinates: &mut [SkiaDesignCoordinate] = if capacity == 0 || coordinates.is_null() {
            &mut []
        } else {
            unsafe { std::slice::from_raw_parts_mut(coordinates.cast::<SkiaDesignCoordinate>(), capacity) }
        };
        fontations_ffi::variation_position(&self.fBridgeNormalizedCoords, copyToCoordinates) as i32
    }

    // Source: src/ports/SkTypeface_fontations.cpp:1175-1179
    fn onGetVariationDesignParameters(&self, parameters: &mut [Axis]) -> i32 {
        let mut axisWrapper = AxisWrapperImpl { fAxisArray: parameters };
        fontations_ffi::populate_axes(&self.fBridgeFontRef, &mut axisWrapper) as i32
    }

    // Source: src/ports/SkTypeface_fontations.cpp:1122-1140
    fn onGetFontDescriptor(
        &self,
        owner: &SkTypeface,
        desc: &mut crate::src::core::SkFontDescriptor::SkFontDescriptor,
        serialize: &mut bool,
    ) {
        let mut familyName = String::new();
        self.onGetFamilyName(&mut familyName);
        desc.setFamilyName(familyName.as_bytes());
        desc.setStyle(owner.fontStyle());
        desc.setFactoryId(Self::FactoryId);

        // TODO: keep the index to emit here
        desc.setPaletteIndex(0);
        let palette = self.getPalette();
        // TODO: omit override when palette[n] == CPAL[paletteIndex][n]
        let paletteOverrideCount = palette.len();
        let overrides = desc.setPaletteEntryOverrides(paletteOverrideCount as i32);
        for (i, color) in palette.iter().enumerate() {
            unsafe {
                let entry = overrides.add(i);
                (*entry).index = i as u16;
                (*entry).color = *color;
            }
        }

        *serialize = true;
    }

    // Source: src/ports/SkTypeface_fontations.cpp:291-298
    fn onCharsToGlyphs(&self, chars: &[i32], glyphs: &mut [u16]) {
        debug_assert_eq!(chars.len(), glyphs.len());
        let codepointSlice = unsafe { std::slice::from_raw_parts(chars.as_ptr().cast::<u32>(), chars.len()) };
        fontations_ffi::lookup_glyph_or_zero(&self.fBridgeFontRef, &self.fMappingIndex, codepointSlice, glyphs);
    }

    // Source: src/ports/SkTypeface_fontations.cpp:299-301
    fn onCountGlyphs(&self) -> i32 {
        i32::from(fontations_ffi::num_glyphs(&self.fBridgeFontRef))
    }

    // Source: src/ports/SkTypeface_fontations.cpp:262-264
    fn onGetUPEM(&self) -> i32 {
        i32::from(fontations_ffi::units_per_em_or_zero(&self.fBridgeFontRef))
    }

    // Source: src/ports/SkTypeface_fontations.cpp:266-269
    fn onGetFamilyName(&self, familyName: &mut String) {
        *familyName = fontations_ffi::family_name(&self.fBridgeFontRef);
    }

    // Source: src/ports/SkTypeface_fontations.cpp:271-281
    fn onGetPostScriptName(&self, postscriptName: &mut String) -> bool {
        let mut readPsName = String::new();
        if fontations_ffi::postscript_name(&self.fBridgeFontRef, &mut readPsName) {
            *postscriptName = readPsName;
            return true;
        }
        false
    }

    // Source: src/ports/SkTypeface_fontations.cpp:354-356. As in the source, the
    // iterator reads the typeface's font ref and must not outlive the typeface.
    fn onCreateFamilyNameIterator(
        &self,
        _owner: &SkTypeface,
    ) -> Option<Box<dyn crate::src::core::SkTypeface::LocalizedStrings>> {
        let fontRef: &'static BridgeFontRef<'static> =
            unsafe { &*(&*self.fBridgeFontRef as *const BridgeFontRef<'static>) };
        Some(Box::new(SkrifaLocalizedStrings {
            fBridgeLocalizedStrings: fontations_ffi::get_localized_strings(fontRef),
        }))
    }

    // Source: src/ports/SkTypeface_fontations.cpp:1155-1162
    fn onGetTableTags(&self, tags: &mut [u32]) -> i32 {
        let numTables = fontations_ffi::table_tags(&self.fBridgeFontRef, &mut []);
        if tags.is_empty() {
            return i32::from(numTables);
        }
        let n = (numTables as usize).min(tags.len());
        i32::from(fontations_ffi::table_tags(&self.fBridgeFontRef, &mut tags[..n]))
    }

    // Source: src/ports/SkTypeface_fontations.cpp:1142-1153
    fn onGetTableData(&self, tag: u32, offset: usize, length: usize, data: Option<&mut [u8]>) -> usize {
        let dataSlice: &mut [u8] = match data {
            Some(data) => {
                let length = length.min(data.len());
                &mut data[..length]
            }
            None => &mut [],
        };
        let copied = fontations_ffi::table_data(&self.fBridgeFontRef, tag, offset, dataSlice);
        // If data is nullptr, the Rust side doesn't see a length limit.
        copied.min(length)
    }
}

// Source: src/ports/SkTypeface_fontations.cpp:519-525
#[allow(non_snake_case)]
mod ScalerContextBits {
    pub const PATH: u16 = 1;
    pub const COLRv0: u16 = 2;
    pub const COLRv1: u16 = 3;
    pub const BITMAP: u16 = 4;
}

// Source: src/ports/SkTypeface_fontations.cpp:358-985
pub struct SkFontationsScalerContext {
    fBase: SkScalerContext,
    // The real Fontations typeface, owned by the proxy typeface the base holds.
    fTypeface: NonNull<SkTypeface_Fontations>,
    fScale: SkPoint,
    fRemainingMatrix: SkMatrix,
    fHintingInstance: Box<BridgeHintingInstance>,
    fDoLinearMetrics: bool,
    // Keeping the path extraction target buffers around significantly avoids
    // allocation churn. (fPathMutex, fPathVerbs, fPathPoints)
    fPath: Mutex<(Vec<u8>, Vec<FfiPoint>)>,
}

impl SkFontationsScalerContext {
    fn real(&self) -> &SkTypeface_Fontations {
        // The typeface outlives every scaler it creates (SkScalerContext contract).
        unsafe { self.fTypeface.as_ref() }
    }

    // Source: src/ports/SkTypeface_fontations.cpp:360-459
    unsafe fn new(
        realTypeface: &SkTypeface_Fontations,
        effects: &crate::src::core::SkScalerContext::SkScalerContextEffects<'_>,
        desc: &crate::src::core::SkDescriptor::SkDescriptor,
        proxyTypeface: NonNull<SkTypeface>,
        path_owner: &crate::include::core::SkRefCnt::sk_sp<dyn crate::src::core::SkPathEffect::SkPathEffect>,
        mask_owner: Option<&Arc<dyn crate::src::core::SkMaskFilter::SkMaskFilter>>,
    ) -> Self {
        // proxyTypeface owns the realTypeface
        let base = unsafe { SkScalerContext::new(proxyTypeface, effects, desc, path_owner, mask_owner) };
        let mut fScale = SkPoint::make(0.0, 0.0);
        let mut fRemainingMatrix = SkMatrix::default();
        base.getRec()
            .compute_matrices(PreMatrixScale::Vertical, &mut fScale, &mut fRemainingMatrix, None, None, None);

        let mut fDoLinearMetrics = base.isLinearMetrics();
        let fRec = *base.getRec();
        let fOutlines = realTypeface.getOutlines();
        let fGlyphStyles = realTypeface.getGlyphStyles();
        let fBridgeNormalizedCoords = realTypeface.getBridgeNormalizedCoords();

        // See below for the exception for SkFontHinting::kSlight.
        let forceAutoHinting = fRec.fFlags & flags::FORCE_AUTOHINTING != 0;
        let mut autoHintingControl = if forceAutoHinting {
            AutoHintingControl::ForceForGlyfAndCff
        } else {
            AutoHintingControl::Fallback
        };

        let fHintingInstance;
        unsafe {
            // Hinting-reliant fonts exist that display incorrect contours when not executing their
            // hinting instructions. Detect those and force-enable hinting for them.
            if fontations_ffi::hinting_reliant(fOutlines) {
                fHintingInstance =
                    fontations_ffi::make_mono_hinting_instance(fOutlines, fScale.fY, fBridgeNormalizedCoords);
                fDoLinearMetrics = false;
            } else if Format::kBW_Format == fRec.fMaskFormat {
                if fRec.get_hinting() == SkFontHinting::kNone {
                    fHintingInstance = fontations_ffi::no_hinting_instance();
                    fDoLinearMetrics = true;
                } else {
                    fHintingInstance =
                        fontations_ffi::make_mono_hinting_instance(fOutlines, fScale.fY, fBridgeNormalizedCoords);
                    fDoLinearMetrics = false;
                }
            } else {
                let hinting = fRec.get_hinting();
                if hinting == SkFontHinting::kNone {
                    fHintingInstance = fontations_ffi::no_hinting_instance();
                    fDoLinearMetrics = true;
                } else if hinting == SkFontHinting::kSlight {
                    // Unhinted metrics.
                    if autoHintingControl != AutoHintingControl::ForceForGlyfAndCff
                        && autoHintingControl != AutoHintingControl::ForceOff
                    {
                        autoHintingControl = AutoHintingControl::ForceForGlyf;
                    }
                    fHintingInstance = fontations_ffi::make_hinting_instance(
                        fOutlines,
                        fGlyphStyles,
                        fScale.fY,
                        fBridgeNormalizedCoords,
                        true,  /* do_light_hinting */
                        false, /* do_lcd_antialiasing */
                        false, /* lcd_orientation_vertical */
                        autoHintingControl,
                    );
                    fDoLinearMetrics = true;
                } else if hinting == SkFontHinting::kNormal {
                    // No hinting to subpixel coordinates.
                    fHintingInstance = fontations_ffi::make_hinting_instance(
                        fOutlines,
                        fGlyphStyles,
                        fScale.fY,
                        fBridgeNormalizedCoords,
                        false, /* do_light_hinting */
                        false, /* do_lcd_antialiasing */
                        false, /* lcd_orientation_vertical */
                        autoHintingControl,
                    );
                } else {
                    // SkFontHinting::kFull: attempt to make use of hinting to subpixel coordinates.
                    fHintingInstance = fontations_ffi::make_hinting_instance(
                        fOutlines,
                        fGlyphStyles,
                        fScale.fY,
                        fBridgeNormalizedCoords,
                        false,        /* do_light_hinting */
                        isLCD(&fRec), /* do_lcd_antialiasing */
                        fRec.fFlags & flags::LCD_VERTICAL != 0, /* lcd_orientation_vertical */
                        autoHintingControl,
                    );
                }
            }
        }

        Self {
            fBase: base,
            fTypeface: NonNull::from(realTypeface),
            fScale,
            fRemainingMatrix,
            fHintingInstance,
            fDoLinearMetrics,
            fPath: Mutex::new((Vec::new(), Vec::new())),
        }
    }

    // Source: src/ports/SkTypeface_fontations.cpp:461-476
    fn getContourHeightForLetter(&self, letter: i32) -> Option<f32> {
        let real = self.real();
        let mut glyphId = [0u16; 1];
        fontations_ffi::lookup_glyph_or_zero(
            real.getBridgeFontRef(),
            real.getMappingIndex(),
            &[letter as u32],
            &mut glyphId,
        );
        if glyphId[0] == 0 {
            return None;
        }
        let mut scalerMetrics = BridgeScalerMetrics::default();
        let glyphPath =
            self.generatePathForGlyphId(glyphId[0], self.fScale.fY, &self.fHintingInstance, &mut scalerMetrics)?;
        let bounds = glyphPath.getBounds();
        Some(bounds.fBottom - bounds.fTop)
    }

    // Source: src/ports/SkTypeface_fontations.cpp:478-516
    // yScale is only used if hintinInstance is set to Unhinted,
    // otherwise the size is controlled by the configured hintingInstance.
    fn generatePathForGlyphId(
        &self,
        glyphId: u16,
        yScale: f32,
        hintingInstance: &BridgeHintingInstance,
        scalerMetrics: &mut BridgeScalerMetrics,
    ) -> Option<SkPath> {
        let real = self.real();
        // See https://crbug.com/390889644 - path extraction must be thread-safe.
        let mut guard = self.fPath.lock().unwrap_or_else(|error| error.into_inner());
        let (fPathVerbs, fPathPoints) = &mut *guard;
        let extracted = fontations_ffi::get_path_verbs_points(
            real.getOutlines(),
            glyphId,
            yScale,
            real.getBridgeNormalizedCoords(),
            hintingInstance,
            fPathVerbs,
            fPathPoints,
            scalerMetrics,
        );
        // See https://issues.skia.org/345178242 for details:
        // The FreeType backend performs a path simplification here based on the
        // equivalent of what we have here as scalerMetrics.has_overlaps
        // Since PathOps::Simplify fails or at times produces incorrect simplified
        // contours, skip that step here.
        let path = extracted.then(|| {
            // FfiPoint and SkPoint are both #[repr(C)] { f32, f32 }.
            let points = unsafe {
                std::slice::from_raw_parts(fPathPoints.as_ptr().cast::<SkPoint>(), fPathPoints.len())
            };
            SkPath::Make(
                points,
                fPathVerbs,
                &[],
                crate::include::core::SkPathTypes::SkPathFillType::kWinding,
                false,
            )
        });
        // Keep allocations in check (SK_AT_SCOPE_EXIT in the source).
        fontations_ffi::shrink_verbs_points_if_needed(fPathVerbs, fPathPoints);
        path
    }

    // Source: src/ports/SkTypeface_fontations.cpp:831-843
    fn generatePathImpl(&self, glyphId: u16, scalerMetrics: &mut BridgeScalerMetrics) -> Option<GeneratedPath> {
        let path = self.generatePathForGlyphId(glyphId, self.fScale.fY, &self.fHintingInstance, scalerMetrics)?;
        let newpath = path.tryMakeTransform(&self.fRemainingMatrix)?;
        Some(GeneratedPath {
            path: newpath,
            modified: !self.fRemainingMatrix.isIdentity(),
        })
    }
}

impl crate::src::core::SkScalerContext::SkScalerContextVirtual for SkFontationsScalerContext {
    fn base(&self) -> &SkScalerContext {
        &self.fBase
    }
    fn base_mut(&mut self) -> &mut SkScalerContext {
        &mut self.fBase
    }

    // Source: src/ports/SkTypeface_fontations.cpp:527-720 (outline branch; the
    // COLR and bitmap branches await colr.rs/bitmap.rs).
    unsafe fn generateMetrics(
        &mut self,
        glyph: &SkGlyph,
        _alloc: &mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    ) -> GlyphMetrics {
        let mut mx = GlyphMetrics::new(glyph.maskFormat());

        // Local overrides for color fonts etc. may alter the request for linear metrics.
        let doLinearMetrics = self.fDoLinearMetrics;

        let real = self.real();
        let mut x_advance = fontations_ffi::unhinted_advance_width_or_zero(
            real.getBridgeFontRef(),
            self.fScale.fY,
            real.getBridgeNormalizedCoords(),
            glyph.getGlyphID(),
        );
        if !doLinearMetrics {
            let mut scalerMetrics = BridgeScalerMetrics::default();
            if let Some(generatedPath) = self.generatePathImpl(glyph.getGlyphID(), &mut scalerMetrics) {
                mx.generatedPath = Some(generatedPath);

                if scalerMetrics.has_adjusted_advance {
                    // FreeType rounds the advance to full pixels when in hinting modes.
                    // Match FreeType and round here.
                    let hinted_advance = scalerMetrics.adjusted_advance.round();
                    // TODO(drott): Remove this workaround for fontations returning 0
                    // for a space glyph without contours, compare
                    // https://github.com/googlefonts/fontations/issues/905
                    if hinted_advance != x_advance && hinted_advance != 0.0 {
                        x_advance = hinted_advance;
                    }
                }
            }
        }
        mx.advance = self.fRemainingMatrix.mapPoint(SkPoint::make(x_advance, 0.0));

        mx.extraBits = ScalerContextBits::PATH;
        mx.computeFromPath = true;
        mx
    }

    // Source: src/ports/SkTypeface_fontations.cpp:802-829
    unsafe fn generateImage(&mut self, glyph: &SkGlyph, imageBuffer: *mut c_void) {
        let format = glyph.extraBits();
        if format == ScalerContextBits::PATH {
            let _ = unsafe { self.fBase.generateImageFromPath(glyph, imageBuffer) };
        } else {
            // COLRv0/COLRv1 and BITMAP are never assigned until colr.rs/bitmap.rs are mapped.
            let _ = kSkShowTextBlitCoverage;
            panic!("Bad format");
        }
    }

    // Source: src/ports/SkTypeface_fontations.cpp:845-851
    // For hinted glyphs, generateMetrics provides a shortcut to set a generated path on
    // SkGlyph - so this method will not be called when a path has already been set.
    unsafe fn generatePath(&mut self, glyph: &SkGlyph) -> Option<GeneratedPath> {
        debug_assert_eq!(glyph.extraBits(), ScalerContextBits::PATH);
        let mut scalerMetrics = BridgeScalerMetrics::default();
        self.generatePathImpl(glyph.getGlyphID(), &mut scalerMetrics)
    }

    // Source: src/ports/SkTypeface_fontations.cpp:906-964
    unsafe fn generateFontMetrics(&mut self, out_metrics: &mut SkFontMetrics) {
        let real = self.real();
        let metrics =
            fontations_ffi::get_skia_metrics(real.getBridgeFontRef(), self.fScale.fY, real.getBridgeNormalizedCoords());
        out_metrics.fTop = -metrics.top;
        out_metrics.fAscent = -metrics.ascent;
        out_metrics.fDescent = -metrics.descent;
        out_metrics.fBottom = -metrics.bottom;
        out_metrics.fLeading = metrics.leading;
        out_metrics.fAvgCharWidth = metrics.avg_char_width;
        out_metrics.fMaxCharWidth = metrics.max_char_width;
        out_metrics.fXMin = metrics.x_min;
        out_metrics.fXMax = metrics.x_max;
        out_metrics.fXHeight = -metrics.x_height;
        out_metrics.fCapHeight = -metrics.cap_height;
        out_metrics.fFlags = 0;

        // Cap height synthesis.
        if out_metrics.fCapHeight == 0.0 {
            out_metrics.fCapHeight = self.getContourHeightForLetter('H' as i32).unwrap_or(metrics.ascent);
        }

        if out_metrics.fXHeight == 0.0 {
            out_metrics.fXHeight = self.getContourHeightForLetter('x' as i32).unwrap_or(metrics.ascent);
        }

        if fontations_ffi::table_data(real.getBridgeFontRef(), SkSetFourByteTag(b'f', b'v', b'a', b'r'), 0, &mut []) != 0 {
            out_metrics.fFlags |= SkFontMetrics::kBoundsInvalid_Flag;
        }
        let setMetric = |dstMetric: &mut f32, srcMetric: f32, flags: &mut u32, flag: u32| {
            if srcMetric.is_nan() {
                *dstMetric = 0.0;
            } else {
                *dstMetric = srcMetric;
                *flags |= flag;
            }
        };
        setMetric(
            &mut out_metrics.fUnderlinePosition,
            -metrics.underline_position,
            &mut out_metrics.fFlags,
            SkFontMetrics::kUnderlinePositionIsValid_Flag,
        );
        setMetric(
            &mut out_metrics.fUnderlineThickness,
            metrics.underline_thickness,
            &mut out_metrics.fFlags,
            SkFontMetrics::kUnderlineThicknessIsValid_Flag,
        );

        setMetric(
            &mut out_metrics.fStrikeoutPosition,
            -metrics.strikeout_position,
            &mut out_metrics.fFlags,
            SkFontMetrics::kStrikeoutPositionIsValid_Flag,
        );
        setMetric(
            &mut out_metrics.fStrikeoutThickness,
            metrics.strikeout_thickness,
            &mut out_metrics.fFlags,
            SkFontMetrics::kStrikeoutThicknessIsValid_Flag,
        );
    }
}
