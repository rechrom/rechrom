//! Paint replay into the source-aligned `skia` Canvas.
//!
//! This receiver is private to `raster`. It consumes cc-selected
//! `RasterPlayback` and writes the engine's unpublished allocation. Unsupported
//! replay vocabulary is returned as an error; there is no alternate receiver.

use super::{backend::*, *};
use crate::convert::{resources, ToSkia};
use compositor_core::{
    adapter::{
        display_item_list::DisplayCanvas,
        geometry::{Rect, Transform as _},
        picture_layer_impl::PictureRasterSource,
    },
    runtime::paint::{
        ContentDisplayItemList, CpuPaintCanvas, CpuPaintColorFilter, CpuPaintFlags, CpuPaintOp,
        CpuPaintOpBuffer, CpuPaintOpRef, CpuPaintRecord, CpuPaintShader, RasterPlaybackParams,
    },
};
#[cfg(any(test, feature = "artifact_test_support"))]
use paint::paint_engine::PaintArtifact;
use raster_commands::commands::{
    self as command, CommandKind, DrawCommand, FontSmoothing, ImageInterpolationQuality,
    ImageOrientation, MaskLayer, PaintBlendMode, PaintMaskComposite, PaintMaskMode,
    PaintPathCommand, PaintPathVerb, ResourceContext,
};
use skia::{
    include::core::{
        SkAlphaType::SkAlphaType,
        SkClipOp::SkClipOp,
        SkColorType::SkColorType,
        SkFontArguments::{Coordinate, SkFontArguments, VariationPosition},
        SkFontTypes::SkFontHinting,
        SkPathTypes::{SkPathDirection, SkPathFillType},
        SkSamplingOptions::{SkCubicResampler, SkFilterMode, SkMipmapMode, SkSamplingOptions},
        SkSize::SkISize,
        SkSurfaceProps::SkSurfaceProps,
        SkTileMode::SkTileMode,
    },
    include::effects::SkGradient::{
        ColorSpace as GradientColorSpace, Colors as GradientColors, HueMethod as GradientHueMethod,
        InPremul as GradientInPremul, Interpolation as GradientInterpolation, SkGradient,
    },
    raster_target::{PixelFormat, RasterTarget},
    src::core::{
        SkBlendMode::SkBlendMode,
        SkCanvas::SrcRectConstraint,
        SkColor::SkColor4f,
        SkConvertPixels::SkConvertPixels,
        SkFont::{Edging, SkFont},
        SkImageFilter::SkImageFilter,
        SkImageInfo::SkImageInfo,
        SkM44::SkM44,
        SkMatrix::SkMatrix,
        SkPaint::{Cap, Join, SkPaint, Style},
        SkPath::SkPath,
        SkPathBuilder::SkPathBuilder,
        SkPixmap::SkPixmap,
        SkPoint::SkIPoint,
        SkPoint::SkPoint,
        SkRRect::SkRRect,
        SkRect::SkRect,
    },
    src::shaders::SkEmptyShader::Empty as EmptyShader,
    src::shaders::SkPictureShader::CachedImageInfo,
};
#[cfg(target_vendor = "apple")]
use std::borrow::Cow;
use std::sync::Arc;

pub(super) struct TranslatedSkiaBackend {
    _image_cache_byte_limit: usize,
    typeface_cache: TypefaceCache,
}

#[cfg(target_vendor = "apple")]
struct CachedTypeface {
    identity: command::FontDataIdentity,
    variations: Vec<command::FontVariation>,
    font_size_bits: u64,
    typeface: skia::src::core::SkTypeface::SkTypefaceRef,
}

#[cfg(target_vendor = "apple")]
#[derive(Default)]
struct TypefaceCache {
    entries: std::sync::Mutex<Vec<CachedTypeface>>,
}

#[cfg(not(target_vendor = "apple"))]
#[derive(Default)]
struct TypefaceCache;

#[cfg(target_vendor = "apple")]
impl TypefaceCache {
    fn GetOrCreate(
        &self,
        item: &DrawCommand,
        face: &command::FontFace<'_>,
    ) -> io::Result<skia::src::core::SkTypeface::SkTypefaceRef> {
        let source_variations = if item.font_variations.is_empty() {
            &face.variations
        } else {
            &item.font_variations
        };
        let variations = command::canonical_font_variations(source_variations);
        // SkTypeface identity is face + variation coordinates. Text size lives
        // on SkFont and must not split the typeface cache; doing so repeatedly
        // rebuilt the selected face of a TTC into an owned standalone SFNT.
        // Apple's private `hvgl` faces are the exception because their guarded
        // font reconstruction is explicitly size-dependent.
        let font_size_bits = sfnt_face_table(face.bytes, face.face_index, b"hvgl")
            .is_some()
            .then(|| item.font_size.to_bits())
            .unwrap_or_default();
        let matches = |entry: &CachedTypeface| {
            entry.font_size_bits == font_size_bits
                && entry.variations.as_slice() == variations.as_ref()
                && entry.identity.matches(face)
        };
        if let Some(typeface) = self
            .entries
            .lock()
            .map_err(|_| io::Error::other("translated typeface cache poisoned"))?
            .iter()
            .find(|entry| matches(entry))
            .map(|entry| entry.typeface.clone())
        {
            return Ok(typeface);
        }

        let typeface = create_raster_typeface(item, face, variations.as_ref())?;
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| io::Error::other("translated typeface cache poisoned"))?;
        if let Some(entry) = entries.iter().find(|entry| matches(entry)) {
            return Ok(entry.typeface.clone());
        }
        // Chromium retains resolved SkTypeface objects in its strike/font
        // caches. Keep the same expensive exact-face construction out of tile
        // replay while bounding this backend-local retained identity set.
        if entries.len() >= 64 {
            entries.remove(0);
        }
        entries.push(CachedTypeface {
            identity: command::FontDataIdentity::new(face),
            variations: variations.into_owned(),
            font_size_bits,
            typeface: typeface.clone(),
        });
        Ok(typeface)
    }
}

#[cfg(target_vendor = "apple")]
fn sfnt_checksum(bytes: &[u8]) -> u32 {
    bytes.chunks(4).fold(0u32, |sum, chunk| {
        let mut word = [0u8; 4];
        word[..chunk.len()].copy_from_slice(chunk);
        sum.wrapping_add(u32::from_be_bytes(word))
    })
}

#[cfg(target_vendor = "apple")]
fn sfnt_face_table<'a>(bytes: &'a [u8], face_index: u32, tag: &[u8; 4]) -> Option<&'a [u8]> {
    let index = face_index & 0xffff;
    let face_offset = if bytes.starts_with(b"ttcf") {
        let count = u32::from_be_bytes(bytes.get(8..12)?.try_into().ok()?);
        if index >= count {
            return None;
        }
        let at = 12usize.checked_add((index as usize).checked_mul(4)?)?;
        u32::from_be_bytes(bytes.get(at..at + 4)?.try_into().ok()?) as usize
    } else {
        if index != 0 {
            return None;
        }
        0
    };
    let header = bytes.get(face_offset..face_offset.checked_add(12)?)?;
    let count = usize::from(u16::from_be_bytes(header.get(4..6)?.try_into().ok()?));
    let directory_start = face_offset.checked_add(12)?;
    let directory =
        bytes.get(directory_start..directory_start.checked_add(count.checked_mul(16)?)?)?;
    // Skia/ttf-parser preserve the final duplicate table record. Do the same
    // for the exact font identity check below.
    let mut result = None;
    for record in directory.chunks_exact(16) {
        if record.get(..4)? != tag {
            continue;
        }
        let offset = u32::from_be_bytes(record.get(8..12)?.try_into().ok()?) as usize;
        let length = u32::from_be_bytes(record.get(12..16)?.try_into().ok()?) as usize;
        result = bytes.get(offset..offset.checked_add(length)?);
    }
    result
}

#[cfg(target_vendor = "apple")]
mod exact_system_font {
    use super::*;
    use core::ffi::c_void;
    use skia::src::ports::SkTypeface_mac_ct::{
        CFData, CFString, CFStringRef, CTFont, CTFontRef, SkMakeTypefaceFromCTFont,
    };

    struct Owned<T>(*mut T);

    impl<T> Owned<T> {
        unsafe fn new(value: *mut T) -> Option<Self> {
            (!value.is_null()).then_some(Self(value))
        }

        fn get(&self) -> *mut T {
            self.0
        }
    }

    impl<T> Drop for Owned<T> {
        fn drop(&mut self) {
            unsafe { CFRelease(self.0.cast()) }
        }
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFStringCreateWithBytes(
            allocator: *const c_void,
            bytes: *const u8,
            length: isize,
            encoding: u32,
            external_representation: bool,
        ) -> CFStringRef;
        fn CFDataGetLength(value: *mut CFData) -> isize;
        fn CFDataGetBytePtr(value: *mut CFData) -> *const u8;
        fn CFRelease(value: *const c_void);
    }

    #[link(name = "CoreText", kind = "framework")]
    unsafe extern "C" {
        fn CTFontCreateWithName(name: CFStringRef, size: f64, matrix: *const c_void) -> CTFontRef;
        fn CTFontCreateUIFontForLanguage(
            ui_type: u32,
            size: f64,
            language: CFStringRef,
        ) -> CTFontRef;
        fn CTFontCopyTable(font: CTFontRef, tag: u32, options: u32) -> *mut CFData;
    }

    fn table_matches(font: CTFontRef, bytes: &[u8], face_index: u32, tag: &[u8; 4]) -> bool {
        let Some(expected) = sfnt_face_table(bytes, face_index, tag) else {
            return false;
        };
        let Some(actual) =
            (unsafe { Owned::new(CTFontCopyTable(font, u32::from_be_bytes(*tag), 0)) })
        else {
            return false;
        };
        let length = unsafe { CFDataGetLength(actual.get()) };
        let data = unsafe { CFDataGetBytePtr(actual.get()) };
        length >= 0
            && !data.is_null()
            && unsafe { core::slice::from_raw_parts(data, length as usize) } == expected
    }

    /// Font selection already chose the exact face. Recover an installed
    /// system face by its PostScript identity and accept it only when the same
    /// core SFNT tables match. Chromium keeps the selected CTFont/SkTypeface;
    /// this guarded reconstruction preserves that path instead of copying a
    /// large TTC into a standalone SFNT. Web fonts whose tables do not match
    /// an installed face continue through the stream fallback below. Apple's
    /// private `hvgl` faces additionally require this path for correctness.
    pub(super) fn make(
        bytes: &[u8],
        face_index: u32,
        resolved_family: &str,
        size: f32,
        arguments: &SkFontArguments,
    ) -> Option<skia::src::core::SkTypeface::SkTypefaceRef> {
        let matches = |font: CTFontRef| {
            [b"name", b"head", b"maxp", b"cmap"]
                .iter()
                .all(|tag| table_matches(font, bytes, face_index, *tag))
        };
        let finish = |font: CTFontRef| {
            let base = unsafe { SkMakeTypefaceFromCTFont(font) }?;
            unsafe { base.makeClone(arguments) }
        };

        // CoreText intentionally rejects direct requests for dot-prefixed
        // system UI PostScript names. Chromium retains the CTFont returned by
        // CTFontCreateUIFontForLanguage; recreate that same public API result
        // and keep the table-identity guard before accepting it.
        if resolved_family.starts_with('.') {
            let ui_type = if resolved_family.to_ascii_lowercase().contains("mono") {
                1 // kCTFontUIFontUserFixedPitch
            } else {
                2 // kCTFontUIFontSystem
            };
            let font = unsafe {
                Owned::<CTFont>::new(CTFontCreateUIFontForLanguage(
                    ui_type,
                    f64::from(size.max(f32::MIN_POSITIVE)),
                    core::ptr::null_mut(),
                ))
            }?;
            let matched = matches(font.get());
            return matched.then(|| finish(font.get())).flatten();
        }

        let names = sfnt_face_table(bytes, face_index, b"name")?;
        let count = usize::from(u16::from_be_bytes(names.get(2..4)?.try_into().ok()?));
        let strings = usize::from(u16::from_be_bytes(names.get(4..6)?.try_into().ok()?));
        let records = names.get(6..6usize.checked_add(count.checked_mul(12)?)?)?;
        for record in records.chunks_exact(12) {
            if u16::from_be_bytes(record.get(6..8)?.try_into().ok()?) != 6 {
                continue;
            }
            let encoding = match u16::from_be_bytes(record.get(..2)?.try_into().ok()?) {
                0 | 3 => 0x1000_0100, // kCFStringEncodingUTF16BE
                1 => 0,               // kCFStringEncodingMacRoman
                _ => continue,
            };
            let length = usize::from(u16::from_be_bytes(record.get(8..10)?.try_into().ok()?));
            let offset = strings.checked_add(usize::from(u16::from_be_bytes(
                record.get(10..12)?.try_into().ok()?,
            )))?;
            let name = names.get(offset..offset.checked_add(length)?)?;
            let Some(name) = (unsafe {
                Owned::<CFString>::new(CFStringCreateWithBytes(
                    core::ptr::null(),
                    name.as_ptr(),
                    name.len() as isize,
                    encoding,
                    false,
                ))
            }) else {
                continue;
            };
            let Some(font) = (unsafe {
                Owned::<CTFont>::new(CTFontCreateWithName(
                    name.get(),
                    f64::from(size.max(f32::MIN_POSITIVE)),
                    core::ptr::null(),
                ))
            }) else {
                continue;
            };
            if !matches(font.get()) {
                continue;
            }
            return finish(font.get());
        }
        None
    }
}

/// CoreText's singular data-descriptor API, and therefore Skia's
/// `SkTypeface_Mac::MakeFromStream`, accepts one SFNT face and rejects a
/// nonzero TTC collection index. Font selection has already happened before
/// raster. Preserve that exact selected face by rebuilding only its container;
/// the table bytes and glyph ids remain unchanged.
#[cfg(target_vendor = "apple")]
fn standalone_font_data(bytes: &[u8], face_index: u32) -> io::Result<Cow<'_, [u8]>> {
    let index = face_index & 0xffff;
    if !bytes.starts_with(b"ttcf") {
        if index != 0 {
            return Err(io::Error::other(
                "non-collection translated glyph font has a nonzero face index",
            ));
        }
        return Ok(Cow::Borrowed(bytes));
    }

    let count = bytes
        .get(8..12)
        .and_then(|value| <[u8; 4]>::try_from(value).ok())
        .map(u32::from_be_bytes)
        .ok_or_else(|| io::Error::other("translated glyph TTC has no face count"))?;
    if index >= count {
        return Err(io::Error::other(
            "translated glyph TTC face index is outside the collection",
        ));
    }
    let index_at =
        12usize
            .checked_add((index as usize).checked_mul(4).ok_or_else(|| {
                io::Error::other("translated glyph TTC face-index offset overflow")
            })?)
            .ok_or_else(|| io::Error::other("translated glyph TTC face-index offset overflow"))?;
    let start = bytes
        .get(index_at..index_at + 4)
        .and_then(|value| <[u8; 4]>::try_from(value).ok())
        .map(u32::from_be_bytes)
        .map(|value| value as usize)
        .ok_or_else(|| io::Error::other("translated glyph TTC has no face offset"))?;
    let header = bytes
        .get(
            start
                ..start.checked_add(12).ok_or_else(|| {
                    io::Error::other("translated glyph SFNT header offset overflow")
                })?,
        )
        .ok_or_else(|| io::Error::other("translated glyph TTC face has no SFNT header"))?;
    let table_count = usize::from(u16::from_be_bytes([header[4], header[5]]));
    let directory_len = table_count
        .checked_mul(16)
        .ok_or_else(|| io::Error::other("translated glyph SFNT directory size overflow"))?;
    let directory_start = start + 12;
    let directory = bytes
        .get(
            directory_start..directory_start.checked_add(directory_len).ok_or_else(|| {
                io::Error::other("translated glyph SFNT directory offset overflow")
            })?,
        )
        .ok_or_else(|| io::Error::other("translated glyph TTC face has no SFNT directory"))?;

    let mut output = Vec::from(header);
    output.extend_from_slice(directory);
    let mut head = None;
    for (table_index, record) in directory.chunks_exact(16).enumerate() {
        let start = u32::from_be_bytes(record[8..12].try_into().unwrap()) as usize;
        let length = u32::from_be_bytes(record[12..16].try_into().unwrap()) as usize;
        let table = bytes
            .get(
                start
                    ..start
                        .checked_add(length)
                        .ok_or_else(|| io::Error::other("translated glyph table overflow"))?,
            )
            .ok_or_else(|| io::Error::other("translated glyph TTC table is outside font data"))?;
        output.resize(
            output
                .len()
                .checked_add(3)
                .ok_or_else(|| io::Error::other("translated glyph font size overflow"))?
                & !3,
            0,
        );
        let offset = output.len();
        output.extend_from_slice(table);
        if &record[..4] == b"head" {
            if length < 12 {
                return Err(io::Error::other("translated glyph head table is truncated"));
            }
            head = Some(offset);
            output[offset + 8..offset + 12].fill(0);
        }
        let checksum = sfnt_checksum(&output[offset..]);
        let at = 12 + table_index * 16;
        output[at + 4..at + 8].copy_from_slice(&checksum.to_be_bytes());
        output[at + 8..at + 12].copy_from_slice(
            &u32::try_from(offset)
                .map_err(|_| io::Error::other("translated glyph SFNT exceeds 4 GiB"))?
                .to_be_bytes(),
        );
    }
    output.resize(
        output
            .len()
            .checked_add(3)
            .ok_or_else(|| io::Error::other("translated glyph font size overflow"))?
            & !3,
        0,
    );
    if let Some(head) = head {
        let adjustment = 0xb1b0_afbau32.wrapping_sub(sfnt_checksum(&output));
        output[head + 8..head + 12].copy_from_slice(&adjustment.to_be_bytes());
    }
    Ok(Cow::Owned(output))
}

#[cfg(target_vendor = "apple")]
fn create_raster_typeface(
    item: &DrawCommand,
    face: &command::FontFace<'_>,
    variations: &[command::FontVariation],
) -> io::Result<skia::src::core::SkTypeface::SkTypefaceRef> {
    if face.bytes.is_empty() {
        return Err(io::Error::other("translated glyph font has no bytes"));
    }
    let coordinates = variations
        .iter()
        .map(|axis| Coordinate {
            axis: axis.tag,
            value: axis.value,
        })
        .collect::<Vec<_>>();
    let coordinate_count = i32::try_from(coordinates.len())
        .map_err(|_| io::Error::other("translated glyph variation count overflow"))?;
    let mut arguments = SkFontArguments::new();
    arguments.setCollectionIndex(0);
    arguments.setVariationDesignPosition(VariationPosition {
        coordinates: coordinates.as_ptr(),
        coordinateCount: coordinate_count,
    });
    if let Some(typeface) = exact_system_font::make(
        face.bytes,
        face.face_index,
        face.resolved_family,
        item.font_size as f32,
        &arguments,
    ) {
        return Ok(typeface);
    }
    let font_data = standalone_font_data(face.bytes, face.face_index)?;
    let stream: core::pin::Pin<Box<dyn skia::src::core::SkStream::SkStreamAsset>> = unsafe {
        skia::src::core::SkStream::SkMemoryStream::MakeCopy(font_data.as_ptr(), font_data.len())
    };
    let stream = unsafe {
        skia::src::core::SkStream::TransferableStreamAsset::from_transferable_stream(stream)
    };
    unsafe {
        skia::src::ports::SkTypeface_mac_ct::SkTypeface_Mac::MakeFromStream(stream, &arguments)
    }
    .ok_or_else(|| io::Error::other("translated CoreText typeface creation failed"))
}

#[cfg(target_vendor = "apple")]
fn raster_typeface(
    item: &DrawCommand,
    resources: &ResourceContext<'_>,
    cache: Option<&TypefaceCache>,
) -> io::Result<skia::src::core::SkTypeface::SkTypefaceRef> {
    let catalog = resources
        .resources
        .as_ref()
        .ok_or_else(|| io::Error::other("translated glyph run has no font resources"))?;
    let face = catalog
        .fonts
        .get(item.font_face_index as usize)
        .ok_or_else(|| io::Error::other("translated glyph font index is outside resources"))?;
    if let Some(cache) = cache {
        return cache.GetOrCreate(item, face);
    }
    let variations = if item.font_variations.is_empty() {
        &face.variations
    } else {
        &item.font_variations
    };
    create_raster_typeface(item, face, variations)
}

/// Source: third_party/blink/renderer/platform/fonts/mac/font_platform_data_mac.mm
/// `FontPlatformData::CreateSkFont`.
///
/// Glyph shaping has already selected the exact face and positions before
/// raster.  Retain Blink's macOS scaler configuration here so replay does not
/// silently change subpixel positioning, linear metrics, or the default
/// smoothing selected by `-webkit-font-smoothing: auto`.
#[cfg(target_vendor = "apple")]
fn ConfigureAppleFont(font: &mut SkFont, smoothing: FontSmoothing) {
    let (antialias, smooth) = match smoothing {
        FontSmoothing::kNone => (false, false),
        FontSmoothing::kAntialiased => (true, false),
        FontSmoothing::kAuto | FontSmoothing::kSubpixelAntialiased => (true, true),
    };
    font.setEdging(if antialias && smooth {
        Edging::kSubpixelAntiAlias
    } else if antialias {
        Edging::kAntiAlias
    } else {
        Edging::kAlias
    });
    font.setEmbeddedBitmaps(false);
    font.setSubpixel(true);
    font.setLinearMetrics(true);
    if smoothing == FontSmoothing::kAntialiased {
        font.setHinting(SkFontHinting::kNone);
    }
}

/// Source: third_party/blink/renderer/platform/fonts/simple_font_data.cc and
/// third_party/blink/renderer/platform/fonts/skia/skia_text_metrics.cc.
///
/// Blink shapes ordinary Apple text with widths from the same configured
/// SkFont strike that later rasterizes it.  The immutable font-engine package
/// keeps this host implementation behind FontBackendFactory so Layout does not
/// acquire a dependency on Raster or Skia.
#[cfg(target_vendor = "apple")]
#[derive(Default)]
pub struct SkiaFontBackendFactory;

#[cfg(target_vendor = "apple")]
struct SkiaFontBackend {
    bytes: Arc<[u8]>,
    face_index: u32,
    resolved_family: Arc<str>,
    variations: Vec<font_engine::FontVariation>,
    typeface: Option<skia::src::core::SkTypeface::SkTypefaceRef>,
    size_dependent_typefaces:
        Arc<std::sync::Mutex<Vec<(u32, skia::src::core::SkTypeface::SkTypefaceRef)>>>,
}

#[cfg(target_vendor = "apple")]
impl SkiaFontBackend {
    fn CreateTypeface(
        bytes: &[u8],
        face_index: u32,
        resolved_family: &str,
        variations: &[font_engine::FontVariation],
        size: f32,
    ) -> skia::src::core::SkTypeface::SkTypefaceRef {
        let mut item = DrawCommand::default();
        item.font_size = f64::from(size);
        item.font_variations = variations
            .iter()
            .map(|axis| command::FontVariation {
                tag: axis.tag,
                value: axis.value,
            })
            .collect();
        let face = command::FontFace {
            family: "",
            resolved_family,
            weight: 400.0,
            italic: false,
            bytes: &bytes,
            bytes_owner: None,
            face_index,
            variations: Vec::new(),
        };
        create_raster_typeface(&item, &face, &item.font_variations)
            .expect("font selection admitted an invalid Skia typeface")
    }

    fn New(
        bytes: Arc<[u8]>,
        face_index: u32,
        resolved_family: Arc<str>,
        variations: Vec<font_engine::FontVariation>,
    ) -> Self {
        // Apple's private hvgl faces cannot be reconstructed from their SFNT
        // bytes and are selected through CTFont at the requested size.
        let size_dependent = sfnt_face_table(&bytes, face_index, b"hvgl").is_some();
        let typeface = (!size_dependent)
            .then(|| Self::CreateTypeface(&bytes, face_index, &resolved_family, &variations, 12.0));
        Self {
            bytes,
            face_index,
            resolved_family,
            variations,
            typeface,
            size_dependent_typefaces: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    fn Typeface(&self, size: f32) -> skia::src::core::SkTypeface::SkTypefaceRef {
        if let Some(typeface) = &self.typeface {
            return typeface.clone();
        }
        let key = size.to_bits();
        let mut entries = self
            .size_dependent_typefaces
            .lock()
            .expect("size-dependent typeface cache poisoned");
        if let Some((_, typeface)) = entries.iter().find(|(bits, _)| *bits == key) {
            return typeface.clone();
        }
        let typeface = Self::CreateTypeface(
            &self.bytes,
            self.face_index,
            &self.resolved_family,
            &self.variations,
            size,
        );
        if entries.len() >= 8 {
            entries.remove(0);
        }
        entries.push((key, typeface.clone()));
        typeface
    }

    fn Font(&self, size: f32, synthetic_bold: bool, synthetic_italic: bool) -> SkFont {
        let mut font = SkFont::with_size(Some(self.Typeface(size)), size);
        font.setEmbolden(synthetic_bold);
        font.setSkewX(if synthetic_italic { -0.25 } else { 0.0 });
        ConfigureAppleFont(&mut font, FontSmoothing::kAuto);
        font
    }
}

#[cfg(target_vendor = "apple")]
impl font_engine::FontBackendFactory for SkiaFontBackendFactory {
    fn Create(
        &self,
        bytes: &[u8],
        face_index: u32,
        variations: &[font_engine::FontVariation],
        _resolved_family: &str,
        _metrics_family: &str,
        _weight: f64,
        _italic: bool,
    ) -> Arc<dyn font_engine::FontBackend> {
        Arc::new(SkiaFontBackend::New(
            Arc::from(bytes),
            face_index,
            Arc::from(_resolved_family),
            variations.to_vec(),
        ))
    }
}

#[cfg(target_vendor = "apple")]
impl font_engine::FontBackend for SkiaFontBackend {
    fn Metrics(
        &self,
        size: f32,
        synthetic_bold: bool,
        synthetic_italic: bool,
    ) -> font_engine::FontBackendMetrics {
        let font = self.Font(size, synthetic_bold, synthetic_italic);
        let mut metrics =
            unsafe { std::mem::zeroed::<skia::include::core::SkFontMetrics::SkFontMetrics>() };
        unsafe { font.getMetrics(Some(&mut metrics)) }
            .expect("Skia font strike metrics must be available");
        let mut underline_position = 0.0;
        let mut underline_thickness = 0.0;
        font_engine::FontBackendMetrics {
            ascent: -metrics.fAscent,
            descent: metrics.fDescent,
            leading: metrics.fLeading,
            x_height: metrics.fXHeight,
            cap_height: metrics.fCapHeight,
            has_underline_position: metrics.hasUnderlinePosition(&mut underline_position),
            has_underline_thickness: metrics.hasUnderlineThickness(&mut underline_thickness),
            underline_position,
            underline_thickness,
        }
    }

    fn GlyphMetrics(
        &self,
        glyph: u16,
        size: f32,
        synthetic_bold: bool,
        synthetic_italic: bool,
    ) -> font_engine::FontBackendGlyphMetrics {
        let font = self.Font(size, synthetic_bold, synthetic_italic);
        let advance = unsafe { font.getWidth(glyph) }.expect("Skia glyph width must be available");
        let bounds = unsafe { font.getPath(glyph) }
            .expect("Skia glyph path query must succeed")
            .map(|path| *path.getBounds())
            .unwrap_or_else(|| {
                unsafe { font.getGlyphBounds(glyph, None) }
                    .expect("Skia bitmap glyph bounds must be available")
            });
        font_engine::FontBackendGlyphMetrics {
            x: bounds.fLeft,
            y: bounds.fTop,
            width: bounds.width(),
            height: bounds.height(),
            advance,
        }
    }

    fn GlyphAdvanceForShaping(
        &self,
        glyph: u16,
        size: f32,
        synthetic_bold: bool,
        synthetic_italic: bool,
    ) -> f32 {
        let font = self.Font(size, synthetic_bold, synthetic_italic);
        unsafe { font.getWidth(glyph) }.expect("Skia glyph width must be available")
    }

    fn WithVariations(
        &self,
        variations: &[font_engine::FontVariation],
    ) -> Arc<dyn font_engine::FontBackend> {
        // FontPlatformData retains the selected SkTypeface when the resolved
        // axis coordinates are unchanged. In particular, system font catalog
        // defaults pass through ResolveFontVariations unchanged; rebuilding
        // the same CTFont here blocks first layout on another full font open.
        if variations == self.variations {
            return Arc::new(Self {
                bytes: self.bytes.clone(),
                face_index: self.face_index,
                resolved_family: self.resolved_family.clone(),
                variations: self.variations.clone(),
                typeface: self.typeface.clone(),
                size_dependent_typefaces: self.size_dependent_typefaces.clone(),
            });
        }
        Arc::new(Self::New(
            self.bytes.clone(),
            self.face_index,
            self.resolved_family.clone(),
            variations.to_vec(),
        ))
    }
}

#[cfg(not(target_vendor = "apple"))]
fn raster_typeface(
    _item: &DrawCommand,
    _resources: &ResourceContext<'_>,
    _cache: Option<&TypefaceCache>,
) -> io::Result<skia::src::core::SkTypeface::SkTypefaceRef> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "translated typeface creation is not connected on this platform",
    ))
}

impl TranslatedSkiaBackend {
    pub fn new(image_cache_byte_limit: usize) -> Self {
        Self {
            _image_cache_byte_limit: image_cache_byte_limit,
            typeface_cache: TypefaceCache::default(),
        }
    }
}

fn raster_error(context: &'static str, error: impl core::fmt::Debug) -> io::Error {
    io::Error::other(format!("{context}: {error:?}"))
}

fn pixel_format(format: SharedImageFormat) -> io::Result<PixelFormat> {
    if format == SinglePlaneFormat::RGBA_8888 {
        return Ok(PixelFormat::Rgba8888);
    }
    if format == SinglePlaneFormat::BGRA_8888 {
        return Ok(PixelFormat::Bgra8888);
    }
    if format == SinglePlaneFormat::BGRX_8888 {
        return Ok(PixelFormat::Bgrx8888);
    }
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        format!("translated raster format {format:?} is not implemented"),
    ))
}

fn rect(value: command::PaintRect) -> SkRect {
    SkRect::MakeLTRB(
        value.x as f32,
        value.y as f32,
        (value.x + value.width) as f32,
        (value.y + value.height) as f32,
    )
}

fn cc_rect(value: Rect) -> SkRect {
    SkRect::MakeLTRB(
        value.x as f32,
        value.y as f32,
        value.right() as f32,
        value.bottom() as f32,
    )
}

fn color(value: command::Color) -> SkColor4f {
    SkColor4f {
        fR: value.red,
        fG: value.green,
        fB: value.blue,
        fA: value.alpha,
    }
}

fn css_blend(mode: PaintBlendMode) -> SkBlendMode {
    match mode {
        PaintBlendMode::kNormal => SkBlendMode::kSrcOver,
        PaintBlendMode::kMultiply => SkBlendMode::kMultiply,
        PaintBlendMode::kScreen => SkBlendMode::kScreen,
        PaintBlendMode::kOverlay => SkBlendMode::kOverlay,
        PaintBlendMode::kDarken => SkBlendMode::kDarken,
        PaintBlendMode::kLighten => SkBlendMode::kLighten,
        PaintBlendMode::kColorDodge => SkBlendMode::kColorDodge,
        PaintBlendMode::kColorBurn => SkBlendMode::kColorBurn,
        PaintBlendMode::kHardLight => SkBlendMode::kHardLight,
        PaintBlendMode::kSoftLight => SkBlendMode::kSoftLight,
        PaintBlendMode::kDifference => SkBlendMode::kDifference,
        PaintBlendMode::kExclusion => SkBlendMode::kExclusion,
        PaintBlendMode::kHue => SkBlendMode::kHue,
        PaintBlendMode::kSaturation => SkBlendMode::kSaturation,
        PaintBlendMode::kColor => SkBlendMode::kColor,
        PaintBlendMode::kLuminosity => SkBlendMode::kLuminosity,
        PaintBlendMode::kPlusLighter => SkBlendMode::kPlus,
    }
}

fn sk_blend_mode(mode: paint::cc::paint::blend_mode::BlendMode) -> SkBlendMode {
    use paint::cc::paint::blend_mode::BlendMode as Paint;
    match mode {
        Paint::kClear => SkBlendMode::kClear,
        Paint::kSrc => SkBlendMode::kSrc,
        Paint::kDst => SkBlendMode::kDst,
        Paint::kSrcOver => SkBlendMode::kSrcOver,
        Paint::kDstOver => SkBlendMode::kDstOver,
        Paint::kSrcIn => SkBlendMode::kSrcIn,
        Paint::kDstIn => SkBlendMode::kDstIn,
        Paint::kSrcOut => SkBlendMode::kSrcOut,
        Paint::kDstOut => SkBlendMode::kDstOut,
        Paint::kSrcATop => SkBlendMode::kSrcATop,
        Paint::kDstATop => SkBlendMode::kDstATop,
        Paint::kXor => SkBlendMode::kXor,
        Paint::kPlus => SkBlendMode::kPlus,
        Paint::kModulate => SkBlendMode::kModulate,
        Paint::kScreen => SkBlendMode::kScreen,
        Paint::kOverlay => SkBlendMode::kOverlay,
        Paint::kDarken => SkBlendMode::kDarken,
        Paint::kLighten => SkBlendMode::kLighten,
        Paint::kColorDodge => SkBlendMode::kColorDodge,
        Paint::kColorBurn => SkBlendMode::kColorBurn,
        Paint::kHardLight => SkBlendMode::kHardLight,
        Paint::kSoftLight => SkBlendMode::kSoftLight,
        Paint::kDifference => SkBlendMode::kDifference,
        Paint::kExclusion => SkBlendMode::kExclusion,
        Paint::kMultiply => SkBlendMode::kMultiply,
        Paint::kHue => SkBlendMode::kHue,
        Paint::kSaturation => SkBlendMode::kSaturation,
        Paint::kColor => SkBlendMode::kColor,
        Paint::kLuminosity => SkBlendMode::kLuminosity,
    }
}

fn path(commands: &[PaintPathCommand], even_odd: bool, inverse: bool) -> SkPath {
    let fill = match (even_odd, inverse) {
        (false, false) => SkPathFillType::kWinding,
        (true, false) => SkPathFillType::kEvenOdd,
        (false, true) => SkPathFillType::kInverseWinding,
        (true, true) => SkPathFillType::kInverseEvenOdd,
    };
    let mut builder = SkPathBuilder::with_fill_type(fill);
    for item in commands {
        match item.verb {
            PaintPathVerb::kMoveTo => {
                builder.moveToXY(item.point.x as f32, item.point.y as f32);
            }
            PaintPathVerb::kLineTo => {
                builder.lineToXY(item.point.x as f32, item.point.y as f32);
            }
            PaintPathVerb::kQuadraticTo => {
                builder.quadToXY(
                    item.control1.x as f32,
                    item.control1.y as f32,
                    item.point.x as f32,
                    item.point.y as f32,
                );
            }
            PaintPathVerb::kConicTo => {
                builder.conicToXY(
                    item.control1.x as f32,
                    item.control1.y as f32,
                    item.point.x as f32,
                    item.point.y as f32,
                    item.conic_weight as f32,
                );
            }
            PaintPathVerb::kCubicTo => {
                builder.cubicToXY(
                    item.control1.x as f32,
                    item.control1.y as f32,
                    item.control2.x as f32,
                    item.control2.y as f32,
                    item.point.x as f32,
                    item.point.y as f32,
                );
            }
            PaintPathVerb::kClose => {
                builder.close();
            }
        }
    }
    builder.detach(None)
}

fn rrect(bounds: command::PaintRect, radii: command::PaintCornerRadii) -> SkRRect {
    let mut value = SkRRect::default();
    let point = |radius: command::PaintCornerRadius| SkPoint {
        fX: radius.x as f32,
        fY: radius.y as f32,
    };
    value.setRectRadii(
        &rect(bounds),
        &[
            point(radii.top_left),
            point(radii.top_right),
            point(radii.bottom_right),
            point(radii.bottom_left),
        ],
    );
    value
}

fn converted_rrect(value: &compositor_core::adapter::geometry::RRectF) -> SkRRect {
    let mut result = SkRRect::default();
    result.setRectRadii(
        &SkRect::MakeLTRB(
            value.rect.x(),
            value.rect.y(),
            value.rect.right(),
            value.rect.bottom(),
        ),
        &value.corner_radii.map(|radius| SkPoint {
            fX: radius.x(),
            fY: radius.y(),
        }),
    );
    result
}

fn translated_tile_mode(value: paint::cc::paint::tile_mode::TileMode) -> SkTileMode {
    use paint::cc::paint::tile_mode::TileMode as Paint;
    match value {
        Paint::kClamp => SkTileMode::kClamp,
        Paint::kRepeat => SkTileMode::kRepeat,
        Paint::kMirror => SkTileMode::kMirror,
        Paint::kDecal => SkTileMode::kDecal,
    }
}

fn gradient_matrix(value: &command::TransformMatrix) -> io::Result<SkMatrix> {
    let m = value.values;
    if !m.iter().all(|value| value.is_finite())
        || m[2] != 0.
        || m[6] != 0.
        || m[8] != 0.
        || m[9] != 0.
        || m[10] != 1.
        || m[11] != 0.
        || m[14] != 0.
    {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "translated gradient requires a finite 2D transform",
        ));
    }
    Ok(SkMatrix::MakeAll(
        m[0] as f32,
        m[4] as f32,
        m[12] as f32,
        m[1] as f32,
        m[5] as f32,
        m[13] as f32,
        m[3] as f32,
        m[7] as f32,
        m[15] as f32,
    ))
}

// Source: third_party/blink/renderer/platform/graphics/gradient.cc:212-275.
// Blink resolves every authored stop to sRGB values before handing the colors
// to Skia, then selects the interpolation coordinates independently. Linear
// RGB spaces share the same interpolation math once the colors are in Skia's
// intermediate linear space.
fn gradient_interpolation_space(
    value: command::PaintColorInterpolationSpace,
) -> GradientColorSpace {
    use command::PaintColorInterpolationSpace as Source;
    match value {
        Source::kNone | Source::kSRGB | Source::kSRGBLegacy => GradientColorSpace::kSRGB,
        Source::kSRGBLinear
        | Source::kDisplayP3Linear
        | Source::kRec2100Linear
        | Source::kXYZD50
        | Source::kXYZD65 => GradientColorSpace::kSRGBLinear,
        Source::kDisplayP3 => GradientColorSpace::kDisplayP3,
        Source::kHSL => GradientColorSpace::kHSL,
        Source::kHWB => GradientColorSpace::kHWB,
        Source::kA98RGB => GradientColorSpace::kA98RGB,
        Source::kProPhotoRGB => GradientColorSpace::kProphotoRGB,
        Source::kRec2020 => GradientColorSpace::kRec2020,
        Source::kLab => GradientColorSpace::kLab,
        Source::kLch => GradientColorSpace::kLCH,
        Source::kOklab => GradientColorSpace::kOKLab,
        Source::kOklch => GradientColorSpace::kOKLCH,
    }
}

fn gradient_shader(
    shader: &command::PaintShader,
) -> io::Result<std::sync::Arc<dyn skia::src::shaders::SkShader::SkShader>> {
    if shader
        .stops
        .iter()
        .any(|stop| stop.missing_components != 0 || stop.is_hint)
    {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "translated extended-color CSS gradient is not connected",
        ));
    }
    if shader.stops.is_empty() {
        // Blink rejects an empty CSS gradient before paint; SVG and an
        // unavailable generated-image payload instead contribute transparent
        // pixels. Preserve that non-fatal paint result at the raster boundary.
        return Ok(EmptyShader());
    }
    if let Some((index, stop)) = shader.stops.iter().enumerate().find(|(_, stop)| {
        !stop.offset.is_finite()
            || ![
                stop.color.red,
                stop.color.green,
                stop.color.blue,
                stop.color.alpha,
            ]
            .into_iter()
            .all(f32::is_finite)
    }) {
        return Err(io::Error::other(format!(
            "translated gradient stop {index} is non-finite (offset={}, rgba={:?})",
            stop.offset,
            [
                stop.color.red,
                stop.color.green,
                stop.color.blue,
                stop.color.alpha,
            ]
        )));
    }
    let colors = shader
        .stops
        .iter()
        .map(|stop| color(stop.color))
        .collect::<Vec<_>>();
    let positions = shader
        .stops
        .iter()
        .map(|stop| stop.offset as f32)
        .collect::<Vec<_>>();
    let tile_mode = match shader.spread {
        command::PaintSpreadMethod::kPad => SkTileMode::kClamp,
        command::PaintSpreadMethod::kReflect => SkTileMode::kMirror,
        command::PaintSpreadMethod::kRepeat => SkTileMode::kRepeat,
    };
    let colors = GradientColors::new(&colors, &positions, tile_mode, None);
    let gradient = SkGradient::new(
        colors,
        GradientInterpolation {
            fInPremul: if shader.interpolate_premultiplied {
                GradientInPremul::kYes
            } else {
                GradientInPremul::kNo
            },
            fColorSpace: gradient_interpolation_space(shader.color_interpolation_space),
            fHueMethod: match shader.hue_interpolation_method {
                command::PaintHueInterpolationMethod::kShorter => GradientHueMethod::kShorter,
                command::PaintHueInterpolationMethod::kLonger => GradientHueMethod::kLonger,
                command::PaintHueInterpolationMethod::kIncreasing => GradientHueMethod::kIncreasing,
                command::PaintHueInterpolationMethod::kDecreasing => GradientHueMethod::kDecreasing,
            },
        },
    );
    let mut matrix = gradient_matrix(&shader.transform)?;
    let point = |value: command::Offset| SkPoint {
        fX: value.x as f32,
        fY: value.y as f32,
    };
    let result = match shader.kind {
        command::PaintShaderKind::kLinearGradient => {
            skia::include::effects::SkGradient::LinearGradient(
                Some(&[point(shader.start), point(shader.end)]),
                &gradient,
                Some(&matrix),
            )
        }
        command::PaintShaderKind::kRadialGradient => {
            if let Some(radius_y) = shader.radius_y {
                if shader.radius > 0. && (radius_y - shader.radius).abs() > 1e-9 {
                    let center = point(shader.center);
                    let mut ellipse = SkMatrix::Translate(center.fX, center.fY);
                    ellipse.postConcat(&SkMatrix::Scale(1., (radius_y / shader.radius) as f32));
                    ellipse.postConcat(&SkMatrix::Translate(-center.fX, -center.fY));
                    matrix.postConcat(&ellipse);
                }
            }
            skia::include::effects::SkGradient::TwoPointConicalGradient(
                point(shader.focal),
                shader.focal_radius as f32,
                point(shader.center),
                shader.radius as f32,
                &gradient,
                Some(&matrix),
            )
        }
        command::PaintShaderKind::kConicGradient => {
            skia::include::effects::SkGradient::SweepGradient(
                point(shader.center),
                (shader.start_angle + shader.rotation_angle - 90.) as f32,
                (shader.end_angle + shader.rotation_angle - 90.) as f32,
                &gradient,
                Some(&matrix),
            )
        }
        command::PaintShaderKind::kPattern => None,
    };
    result.ok_or_else(|| io::Error::other("translated Skia rejected gradient shader"))
}

fn generated_shader_paint(flags: &CpuPaintFlags<SkRegion>, alpha: f32) -> io::Result<SkPaint> {
    let Some(shader) = &flags.shader else {
        return converted_paint(flags, alpha);
    };
    if flags.image_filter.is_some() || flags.color_filter.is_some() || flags.looper.is_some() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "translated gradient with filter or looper is not connected",
        ));
    }
    let CpuPaintShader::GeneratedPattern { source, mask_layer } = shader.as_ref() else {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "translated non-gradient cc shader is not connected",
        ));
    };
    let authored = mask_layer
        .and_then(|index| source.op().mask_layers.get(index))
        .and_then(|layer| layer.paint_shader.as_ref())
        .or(source.op().paint_shader.as_ref())
        .ok_or_else(|| io::Error::other("translated generated shader has no authored value"))?;
    let mut paint = converted_paint_core(flags, alpha);
    let authored = authored.to_skia();
    let shader = gradient_shader(&authored)?;
    paint.setShader(Some(shader));
    // Source: third_party/blink/renderer/platform/graphics/gradient.cc
    // Gradient::ApplyToFlags. Blink preserves the legacy rule that every
    // gradient is dithered when it hands the shader to Skia.
    paint.setDither(true);
    Ok(paint)
}

fn converted_paint_core(flags: &CpuPaintFlags<SkRegion>, alpha: f32) -> SkPaint {
    let mut paint = SkPaint::default();
    let source = flags.getColor4f();
    paint.setColor4f(
        &SkColor4f {
            fR: source.red,
            fG: source.green,
            fB: source.blue,
            fA: source.alpha * alpha,
        },
        None,
    );
    paint.setAntiAlias(flags.isAntiAlias());
    paint.setDither(flags.isDither());
    paint.setBlendMode(sk_blend_mode(flags.getBlendMode()));
    paint.setStyle(match flags.getStyle() {
        paint::cc::paint::paint_flags::Style::kFill_Style => Style::kFill_Style,
        paint::cc::paint::paint_flags::Style::kStroke_Style => Style::kStroke_Style,
    });
    paint.setStrokeWidth(flags.getStrokeWidth());
    paint.setStrokeMiter(flags.getStrokeMiter());
    paint.setStrokeCap(match flags.getStrokeCap() {
        paint::cc::paint::paint_flags::Cap::kButt_Cap => Cap::kButt_Cap,
        paint::cc::paint::paint_flags::Cap::kRound_Cap => Cap::kRound_Cap,
        paint::cc::paint::paint_flags::Cap::kSquare_Cap => Cap::kSquare_Cap,
    });
    paint.setStrokeJoin(match flags.getStrokeJoin() {
        paint::cc::paint::paint_flags::Join::kMiter_Join => Join::kMiter_Join,
        paint::cc::paint::paint_flags::Join::kRound_Join => Join::kRound_Join,
        paint::cc::paint::paint_flags::Join::kBevel_Join => Join::kBevel_Join,
    });
    paint
}

fn converted_paint(flags: &CpuPaintFlags<SkRegion>, alpha: f32) -> io::Result<SkPaint> {
    if flags.shader.is_some()
        || flags.image_filter.is_some()
        || flags.color_filter.is_some()
        || flags.looper.is_some()
    {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "translated complex cc PaintFlags are not connected",
        ));
    }
    Ok(converted_paint_core(flags, alpha))
}

fn converted_restore_paint(flags: &CpuPaintFlags<SkRegion>, alpha: f32) -> io::Result<SkPaint> {
    if flags.shader.is_some() || flags.looper.is_some() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "translated saveLayer shader or looper is not connected",
        ));
    }
    let mut paint = converted_paint_core(flags, alpha);
    if let Some(filter) = flags.color_filter {
        match filter {
            CpuPaintColorFilter::Luma => paint.setColorFilter(
                skia::src::effects::colorfilters::SkRuntimeColorFilter::MakeLumaColorFilter()
                    .ok_or_else(|| {
                        io::Error::other("translated Skia rejected the luma color filter")
                    })?
                    .into(),
            ),
        }
    }
    Ok(paint)
}

fn filter_tile_mode(mode: paint::cc::paint::filter_operation::BlurTileMode) -> SkTileMode {
    use paint::cc::paint::filter_operation::BlurTileMode as Source;
    match mode {
        Source::kClamp => SkTileMode::kClamp,
        Source::kRepeat => SkTileMode::kRepeat,
        Source::kMirror => SkTileMode::kMirror,
        Source::kDecal => SkTileMode::kDecal,
    }
}

type TranslatedFilterValue = (Option<Arc<dyn SkImageFilter>>, bool);

#[derive(Clone)]
struct SvgFilterValue {
    validated: Option<Arc<dyn SkImageFilter>>,
    unvalidated: Option<Arc<dyn SkImageFilter>>,
    linear_rgb: bool,
    origin_tainted: bool,
}

impl SvgFilterValue {
    fn shared(
        filter: Option<Arc<dyn SkImageFilter>>,
        linear_rgb: bool,
        origin_tainted: bool,
    ) -> Self {
        Self {
            validated: filter.clone(),
            unvalidated: filter,
            linear_rgb,
            origin_tainted,
        }
    }

    fn select(&self, requires_pm_color_validation: bool) -> Option<Arc<dyn SkImageFilter>> {
        if requires_pm_color_validation {
            self.validated.clone()
        } else {
            self.unvalidated.clone()
        }
    }
}

fn translated_filter_space(
    input: Option<Arc<dyn SkImageFilter>>,
    input_linear_rgb: bool,
    output_linear_rgb: bool,
) -> Option<Arc<dyn SkImageFilter>> {
    if input_linear_rgb == output_linear_rgb {
        return input;
    }
    use skia::src::effects::{colorfilters, imagefilters};
    let color = if output_linear_rgb {
        colorfilters::SkColorSpaceXformColorFilter::SRGBToLinearGamma()
    } else {
        colorfilters::SkColorSpaceXformColorFilter::LinearToSRGBGamma()
    };
    imagefilters::SkColorFilterImageFilter::ColorFilter(Some(color), input, None)
}

fn svg_arithmetic_filter_variants(
    k: [f32; 4],
    background: Option<Arc<dyn SkImageFilter>>,
    foreground: Option<Arc<dyn SkImageFilter>>,
    crop: SkRect,
) -> (
    Option<Arc<dyn SkImageFilter>>,
    Option<Arc<dyn SkImageFilter>>,
) {
    use skia::src::effects::imagefilters;
    (
        imagefilters::SkBlendImageFilter::Arithmetic(
            k[0],
            k[1],
            k[2],
            k[3],
            true,
            background.clone(),
            foreground.clone(),
            Some(crop),
        ),
        imagefilters::SkBlendImageFilter::Arithmetic(
            k[0],
            k[1],
            k[2],
            k[3],
            false,
            background,
            foreground,
            Some(crop),
        ),
    )
}

fn translated_filter_operations_in_space(
    operations: &paint::cc::paint::filter_operations::FilterOperations,
    matrix: &SkM44,
    mut input: Option<Arc<dyn SkImageFilter>>,
    mut linear_rgb: bool,
) -> io::Result<TranslatedFilterValue> {
    use paint::cc::paint::filter_operation::FilterType;
    use paint::cc::paint::render_surface_filters::{ColorMatrixKind, CssColorMatrix};
    use skia::src::effects::{colorfilters, imagefilters};

    let scale_x = matrix.rc(0, 0).hypot(matrix.rc(0, 1));
    let scale_y = matrix.rc(1, 0).hypot(matrix.rc(1, 1));
    for operation in operations.operations() {
        let color_kind = match operation.r#type() {
            FilterType::GRAYSCALE => Some(ColorMatrixKind::Grayscale),
            FilterType::SEPIA => Some(ColorMatrixKind::Sepia),
            FilterType::SATURATE => Some(ColorMatrixKind::Saturate),
            FilterType::HUE_ROTATE => Some(ColorMatrixKind::HueRotate),
            FilterType::INVERT => Some(ColorMatrixKind::Invert),
            FilterType::OPACITY => Some(ColorMatrixKind::Opacity),
            FilterType::BRIGHTNESS => Some(ColorMatrixKind::Brightness),
            FilterType::CONTRAST => Some(ColorMatrixKind::Contrast),
            _ => None,
        };
        if let Some(kind) = color_kind {
            let values = CssColorMatrix(kind, operation.amount());
            let color = colorfilters::SkMatrixColorFilter::Matrix(
                &values,
                skia::src::core::SkColorFilter::Clamp::kYes,
            )
            .ok_or_else(|| io::Error::other("translated Skia rejected CSS color matrix"))?;
            input = imagefilters::SkColorFilterImageFilter::ColorFilter(Some(color), input, None);
            continue;
        }
        input = match operation.r#type() {
            FilterType::BLUR => imagefilters::SkBlurImageFilter::Blur(
                operation.amount() * scale_x,
                operation.amount() * scale_y,
                filter_tile_mode(operation.blur_tile_mode()),
                input,
                None,
            ),
            FilterType::DROP_SHADOW => {
                let offset = operation.offset();
                let source = operation.drop_shadow_color();
                imagefilters::SkDropShadowImageFilter::DropShadow(
                    matrix.rc(0, 0) * offset.x() as f32 + matrix.rc(0, 1) * offset.y() as f32,
                    matrix.rc(1, 0) * offset.x() as f32 + matrix.rc(1, 1) * offset.y() as f32,
                    operation.amount() * scale_x,
                    operation.amount() * scale_y,
                    SkColor4f {
                        fR: source.red,
                        fG: source.green,
                        fB: source.blue,
                        fA: source.alpha,
                    },
                    None,
                    input,
                    None,
                )
            }
            FilterType::COLOR_MATRIX => {
                let color = colorfilters::SkMatrixColorFilter::Matrix(
                    operation.matrix(),
                    skia::src::core::SkColorFilter::Clamp::kYes,
                )
                .ok_or_else(|| io::Error::other("translated Skia rejected color matrix"))?;
                imagefilters::SkColorFilterImageFilter::ColorFilter(Some(color), input, None)
            }
            FilterType::SATURATING_BRIGHTNESS => {
                let mut values = [0.; 20];
                values[0] = 1.;
                values[6] = 1.;
                values[12] = 1.;
                values[18] = 1.;
                values[4] = operation.amount();
                values[9] = operation.amount();
                values[14] = operation.amount();
                let color = colorfilters::SkMatrixColorFilter::Matrix(
                    &values,
                    skia::src::core::SkColorFilter::Clamp::kYes,
                )
                .ok_or_else(|| {
                    io::Error::other("translated Skia rejected saturating brightness matrix")
                })?;
                imagefilters::SkColorFilterImageFilter::ColorFilter(Some(color), input, None)
            }
            FilterType::REFERENCE => {
                if let Some(filter) = operation.image_filter() {
                    let value =
                        translated_filter_graph(filter.as_ref(), matrix, input, linear_rgb)?;
                    linear_rgb = value.1;
                    value.0
                } else {
                    input
                }
            }
            FilterType::OFFSET => {
                let offset = operation.offset();
                imagefilters::SkMatrixTransformImageFilter::Offset(
                    matrix.rc(0, 0) * offset.x() as f32 + matrix.rc(0, 1) * offset.y() as f32,
                    matrix.rc(1, 0) * offset.x() as f32 + matrix.rc(1, 1) * offset.y() as f32,
                    input,
                    None,
                )
            }
            FilterType::ALPHA_THRESHOLD => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "translated alpha-threshold image filter is not connected",
                ));
            }
            FilterType::ZOOM => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "translated magnifier image filter is not connected",
                ));
            }
            FilterType::GRAYSCALE
            | FilterType::SEPIA
            | FilterType::SATURATE
            | FilterType::HUE_ROTATE
            | FilterType::INVERT
            | FilterType::BRIGHTNESS
            | FilterType::CONTRAST
            | FilterType::OPACITY => unreachable!("handled CSS color matrix"),
        };
    }
    Ok((input, linear_rgb))
}

fn translated_filter_operations(
    operations: &paint::cc::paint::filter_operations::FilterOperations,
    matrix: &SkM44,
    input: Option<Arc<dyn SkImageFilter>>,
) -> io::Result<Option<Arc<dyn SkImageFilter>>> {
    let (input, linear_rgb) =
        translated_filter_operations_in_space(operations, matrix, input, false)?;
    // Source: filter_effect_builder.cc:500-506. CSS filter output is always
    // converted back to the device sRGB space after the ordered chain.
    Ok(translated_filter_space(input, linear_rgb, false))
}

fn translated_canvas_filter(
    kind: &raster_commands::filter_graph::RenderSvgFilterKind,
    input: Option<Arc<dyn SkImageFilter>>,
) -> io::Result<Option<Arc<dyn SkImageFilter>>> {
    use raster_commands::filter_graph::{RenderSvgEdgeMode, RenderSvgFilterKind};
    use skia::src::effects::{colorfilters, imagefilters};

    Ok(match kind {
        RenderSvgFilterKind::ComponentTransfer(functions) => {
            let tables = functions.each_ref().map(svg_component_transfer_table);
            let color = colorfilters::SkTableColorFilter::TableARGB(
                Some(&tables[3]),
                Some(&tables[0]),
                Some(&tables[1]),
                Some(&tables[2]),
            )
            .ok_or_else(|| {
                io::Error::other("translated Skia rejected Canvas component-transfer tables")
            })?;
            imagefilters::SkColorFilterImageFilter::ColorFilter(Some(color), input, None)
        }
        RenderSvgFilterKind::Turbulence {
            base_x,
            base_y,
            octaves,
            seed,
            stitch,
            fractal_noise,
        } => {
            let (base_x, base_y) = if *base_x < 0.0 || *base_y < 0.0 {
                (0.0, 0.0)
            } else {
                (*base_x, *base_y)
            };
            // BuildFilterEffect creates Filter(1.0) and never assigns a
            // primitive subregion to shorthand/Canvas operations. Therefore
            // FETurbulence::FilterPrimitiveSubregion().size() is exactly 0x0.
            let tile = stitch.then(|| SkISize::Make(0, 0));
            let shader = if *fractal_noise {
                skia::src::shaders::SkPerlinNoiseShaderImpl::MakeFractalNoise(
                    base_x,
                    base_y,
                    (*octaves).min(9),
                    *seed,
                    tile.as_ref(),
                )
            } else {
                skia::src::shaders::SkPerlinNoiseShaderImpl::MakeTurbulence(
                    base_x,
                    base_y,
                    (*octaves).min(9),
                    *seed,
                    tile.as_ref(),
                )
            };
            // FETurbulence intentionally ignores its chained SourceGraphic
            // while FilterEffectBuilder still retains that input for taint and
            // bounds propagation.
            imagefilters::SkShaderImageFilter::Shader(shader, false, None)
        }
        RenderSvgFilterKind::Convolve {
            order_x,
            order_y,
            kernel,
            divisor,
            bias,
            target_x,
            target_y,
            edge_mode,
            preserve_alpha,
        } => {
            let kernel_area = order_x
                .checked_mul(*order_y)
                .filter(|area| *area <= i32::MAX as u32);
            if kernel_area.is_none()
                || kernel_area.map(|area| area as usize) != Some(kernel.len())
                || *target_x < 0
                || *target_y < 0
                || *target_x as u32 >= *order_x
                || *target_y as u32 >= *order_y
            {
                imagefilters::SkCropImageFilter::Empty()
            } else {
                let size = SkISize::Make(
                    i32::try_from(*order_x).expect("validated convolution width"),
                    i32::try_from(*order_y).expect("validated convolution height"),
                );
                let target = SkIPoint::make(*target_x, *target_y);
                let tile = match edge_mode {
                    RenderSvgEdgeMode::None => SkTileMode::kDecal,
                    RenderSvgEdgeMode::Duplicate => SkTileMode::kClamp,
                    RenderSvgEdgeMode::Wrap => SkTileMode::kRepeat,
                };
                let reversed = kernel.iter().rev().copied().collect::<Vec<_>>();
                let divisor = if *divisor == 0.0 {
                    let sum = kernel.iter().sum::<f32>();
                    if sum == 0.0 {
                        1.0
                    } else {
                        sum
                    }
                } else {
                    *divisor
                };
                imagefilters::SkMatrixConvolutionImageFilter::MatrixConvolution(
                    size,
                    Some(&reversed),
                    divisor.recip(),
                    bias * 255.0,
                    target,
                    tile,
                    !preserve_alpha,
                    input,
                    None,
                )
                .or_else(imagefilters::SkCropImageFilter::Empty)
            }
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "non-Canvas SVG primitive reached Canvas filter replay",
            ));
        }
    })
}

fn translated_command_filters(
    operations: &[command::PaintFilterOperation],
    matrix: &SkM44,
) -> io::Result<Option<Arc<dyn SkImageFilter>>> {
    use command::PaintFilterType;
    use paint::cc::paint::render_surface_filters::{ColorMatrixKind, CssColorMatrix};
    use skia::src::effects::{colorfilters, imagefilters};

    let scale_x = matrix.rc(0, 0).hypot(matrix.rc(0, 1));
    let scale_y = matrix.rc(1, 0).hypot(matrix.rc(1, 1));
    let mut input = None;
    for operation in operations {
        let color_kind = match operation.r#type {
            PaintFilterType::kGrayscale => Some(ColorMatrixKind::Grayscale),
            PaintFilterType::kSepia => Some(ColorMatrixKind::Sepia),
            PaintFilterType::kSaturate => Some(ColorMatrixKind::Saturate),
            PaintFilterType::kHueRotate => Some(ColorMatrixKind::HueRotate),
            PaintFilterType::kInvert => Some(ColorMatrixKind::Invert),
            PaintFilterType::kOpacity => Some(ColorMatrixKind::Opacity),
            PaintFilterType::kBrightness => Some(ColorMatrixKind::Brightness),
            PaintFilterType::kContrast => Some(ColorMatrixKind::Contrast),
            _ => None,
        };
        if let Some(kind) = color_kind {
            let values = CssColorMatrix(kind, operation.amount as f32);
            let color = colorfilters::SkMatrixColorFilter::Matrix(
                &values,
                skia::src::core::SkColorFilter::Clamp::kYes,
            )
            .ok_or_else(|| io::Error::other("translated Skia rejected command CSS color matrix"))?;
            input = imagefilters::SkColorFilterImageFilter::ColorFilter(Some(color), input, None);
            continue;
        }
        let explicit_matrix = match operation.r#type {
            PaintFilterType::kLuminanceToAlpha => {
                let mut matrix = [0.0; 20];
                matrix[15] = 0.2125;
                matrix[16] = 0.7154;
                matrix[17] = 0.0721;
                Some(matrix)
            }
            PaintFilterType::kColorMatrix => {
                let mut matrix = [0.0; 20];
                matrix[0] = 1.0;
                matrix[6] = 1.0;
                matrix[12] = 1.0;
                matrix[18] = 1.0;
                if operation.color_matrix.len() == matrix.len() {
                    for (output, value) in matrix.iter_mut().zip(&operation.color_matrix) {
                        *output = *value as f32;
                    }
                }
                Some(matrix)
            }
            _ => None,
        };
        if let Some(values) = explicit_matrix {
            let color = colorfilters::SkMatrixColorFilter::Matrix(
                &values,
                skia::src::core::SkColorFilter::Clamp::kYes,
            )
            .ok_or_else(|| io::Error::other("translated Skia rejected explicit color matrix"))?;
            input = imagefilters::SkColorFilterImageFilter::ColorFilter(Some(color), input, None);
            continue;
        }
        if matches!(
            operation.r#type,
            PaintFilterType::kConvolveMatrix
                | PaintFilterType::kComponentTransfer
                | PaintFilterType::kTurbulence
        ) {
            let kind = operation.canvas_filter.as_ref().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "command Canvas filter lost its typed payload",
                )
            })?;
            input = translated_canvas_filter(kind, input)?;
            continue;
        }
        input = match operation.r#type {
            PaintFilterType::kBlur => imagefilters::SkBlurImageFilter::Blur(
                operation.amount as f32 * scale_x,
                operation.amount as f32 * scale_y,
                SkTileMode::kDecal,
                input,
                None,
            ),
            PaintFilterType::kDropShadow => imagefilters::SkDropShadowImageFilter::DropShadow(
                matrix.rc(0, 0) * operation.offset.x as f32
                    + matrix.rc(0, 1) * operation.offset.y as f32,
                matrix.rc(1, 0) * operation.offset.x as f32
                    + matrix.rc(1, 1) * operation.offset.y as f32,
                operation.blur_radius as f32 * scale_x,
                operation.blur_radius as f32 * scale_y,
                color(operation.color),
                None,
                input,
                None,
            ),
            PaintFilterType::kReference => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "translated command reference filter belongs to renderer effect replay",
                ));
            }
            PaintFilterType::kBoxReflect => {
                let empty = Vec::new();
                let mask_layers = match operation.reflection_mask.as_ref() {
                    Some(mask) => {
                        if let Some(error) = mask.error.as_deref() {
                            return Err(io::Error::new(
                                io::ErrorKind::Unsupported,
                                format!("reflection mask conversion failed: {error}"),
                            ));
                        }
                        mask.layers.as_slice()
                    }
                    None => empty.as_slice(),
                };
                translated_reflection_filter(
                    operation.reflection_direction,
                    operation.reflection_offset,
                    operation.reflection_has_mask,
                    mask_layers,
                    input,
                )?
            }
            PaintFilterType::kBrightness
            | PaintFilterType::kContrast
            | PaintFilterType::kGrayscale
            | PaintFilterType::kHueRotate
            | PaintFilterType::kInvert
            | PaintFilterType::kOpacity
            | PaintFilterType::kSaturate
            | PaintFilterType::kSepia => unreachable!("handled command CSS color matrix"),
            PaintFilterType::kLuminanceToAlpha | PaintFilterType::kColorMatrix => {
                unreachable!("handled command explicit color matrix")
            }
            PaintFilterType::kConvolveMatrix
            | PaintFilterType::kComponentTransfer
            | PaintFilterType::kTurbulence => unreachable!("handled command Canvas filter"),
        };
    }
    Ok(input)
}

/// Build the translated Skia image-filter chain for one renderer/raster
/// operation list. The caller supplies the exact local-to-device transform
/// used by the destination Canvas; no render-pass, resource or target owner is
/// retained by the returned immutable filter graph.
pub fn BuildTranslatedFilterOperations(
    operations: &paint::cc::paint::filter_operations::FilterOperations,
    matrix: &SkM44,
) -> io::Result<Option<Arc<dyn SkImageFilter>>> {
    translated_filter_operations(operations, matrix, None)
}

fn translated_filter_graph(
    filter: &dyn paint::cc::paint::filter_operation::PaintFilter,
    matrix: &SkM44,
    input: Option<Arc<dyn SkImageFilter>>,
    input_linear_rgb: bool,
) -> io::Result<TranslatedFilterValue> {
    use paint::cc::paint::filter_operation::PaintFilterReplay;
    match filter.ReplayGraph() {
        PaintFilterReplay::Operations(operations) => {
            translated_filter_operations_in_space(operations, matrix, input, input_linear_rgb)
        }
        PaintFilterReplay::Reference {
            operation,
            reference_box,
            reference_viewport,
            reference_origin,
            reference_scale,
            input_origin_tainted,
        } => translated_reference_filter(
            operation,
            reference_box,
            reference_viewport,
            reference_origin,
            reference_scale,
            matrix,
            input,
            input_linear_rgb,
            input_origin_tainted,
        ),
        PaintFilterReplay::Canvas(operation) => {
            let kind = operation.canvas_filter.as_ref().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "canonical Canvas filter lost its typed payload",
                )
            })?;
            let kind = crate::svg_filter_graph::translate_canvas_filter(kind);
            let input = translated_filter_space(input, input_linear_rgb, false);
            translated_canvas_filter(&kind, input).map(|filter| (filter, false))
        }
        PaintFilterReplay::Reflection(operation) => {
            let layers = crate::reflection_mask::CompileReflectionMaskLayers(
                &operation.reflection_mask_layers,
            )?;
            translated_reflection_filter(
                match operation.reflection_direction {
                    layoutng_assembly::internal::paint_input::PaintReflectionDirection::kAbove => {
                        command::PaintReflectionDirection::kAbove
                    }
                    layoutng_assembly::internal::paint_input::PaintReflectionDirection::kBelow => {
                        command::PaintReflectionDirection::kBelow
                    }
                    layoutng_assembly::internal::paint_input::PaintReflectionDirection::kLeft => {
                        command::PaintReflectionDirection::kLeft
                    }
                    layoutng_assembly::internal::paint_input::PaintReflectionDirection::kRight => {
                        command::PaintReflectionDirection::kRight
                    }
                },
                operation.reflection_offset,
                operation.reflection_has_mask,
                &layers,
                input,
            )
            .map(|filter| (filter, input_linear_rgb))
        }
    }
}

fn translated_filter_graph_srgb(
    filter: &dyn paint::cc::paint::filter_operation::PaintFilter,
    matrix: &SkM44,
    input: Option<Arc<dyn SkImageFilter>>,
) -> io::Result<Option<Arc<dyn SkImageFilter>>> {
    let (filter, linear_rgb) = translated_filter_graph(filter, matrix, input, false)?;
    Ok(translated_filter_space(filter, linear_rgb, false))
}

/// Source: paint_filter_builder.cc:106-168. Blink rasterizes the private
/// reflection-mask PaintRecord into one bounded image before composing it with
/// SourceGraphic. Rechrom receives the record's already-resolved nine-piece
/// draws, but preserves the same buffer limit, local bounds, SrcIn crop, flip,
/// and final SrcOver ordering.
fn translated_reflection_mask_filter(
    layers: &[raster_commands::filter_graph::RenderReflectionMaskLayer],
) -> io::Result<Option<Arc<dyn SkImageFilter>>> {
    use skia::src::effects::imagefilters;

    let mut bounds: Option<command::PaintRect> = None;
    for layer in layers {
        let clip = layer.clip_rect;
        if clip.width <= 0.0
            || clip.height <= 0.0
            || ![clip.x, clip.y, clip.width, clip.height]
                .into_iter()
                .all(f64::is_finite)
        {
            continue;
        }
        bounds = Some(bounds.map_or(clip, |current| {
            let left = current.x.min(clip.x);
            let top = current.y.min(clip.y);
            let right = (current.x + current.width).max(clip.x + clip.width);
            let bottom = (current.y + current.height).max(clip.y + clip.height);
            command::PaintRect {
                x: left,
                y: top,
                width: right - left,
                height: bottom - top,
            }
        }));
    }
    let Some(bounds) = bounds else {
        // A specified but pending/empty reflection mask is transparent; it
        // must not become the much stronger unmasked reflection.
        return Ok(imagefilters::SkCropImageFilter::Empty());
    };
    let rounded_left = bounds.x.floor();
    let rounded_top = bounds.y.floor();
    let rounded_right = (bounds.x + bounds.width).ceil();
    let rounded_bottom = (bounds.y + bounds.height).ceil();
    let width = rounded_right - rounded_left;
    let height = rounded_bottom - rounded_top;
    const MAX_MASK_BUFFER_PIXELS: f64 = 50.0 * 1024.0 * 1024.0 / 4.0;
    let pixels_count = width * height;
    if pixels_count <= 0.0 || pixels_count >= MAX_MASK_BUFFER_PIXELS {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "reflection mask requires Chromium's oversized RecordPaintFilter fallback",
        ));
    }
    let width = i32::try_from(width as i64)
        .map_err(|_| io::Error::other("reflection mask width exceeds Skia limits"))?;
    let height = i32::try_from(height as i64)
        .map_err(|_| io::Error::other("reflection mask height exceeds Skia limits"))?;
    let row_bytes = (width as usize)
        .checked_mul(4)
        .ok_or_else(|| io::Error::other("reflection mask row size overflow"))?;
    let byte_len = row_bytes
        .checked_mul(height as usize)
        .ok_or_else(|| io::Error::other("reflection mask allocation overflow"))?;
    let mut pixels = vec![0; byte_len];
    let images = layers
        .iter()
        .filter(|layer| {
            layer.image_width > 0 && layer.image_height > 0 && !layer.image_rgba.is_empty()
        })
        .map(|layer| {
            command::Image::shared(
                layer.resource_id,
                layer.image_width,
                layer.image_height,
                &layer.image_rgba,
            )
        })
        .collect();
    let resources = ResourceContext {
        resources: Some(command::ResourceCatalog {
            fonts: Vec::new(),
            images,
        }),
    };
    {
        let mut target = RasterTarget::new(
            &mut pixels,
            width,
            height,
            row_bytes,
            PixelFormat::Rgba8888,
            None,
        )
        .map_err(|error| raster_error("reflection mask target failed", error))?;
        target.set_matrix(&SkMatrix::Translate(-bounds.x as f32, -bounds.y as f32));
        {
            let mut canvas = TranslatedCanvas::new(&mut target, false, 1.0);
            for layer in layers {
                if let Some(shader) = &layer.paint_shader {
                    canvas.draw_gradient(&DrawCommand {
                        r#type: CommandKind::kDrawTiledGradient,
                        rect: layer.clip_rect,
                        tile_rect: layer.tile_rect,
                        repeat_x: layer.repeat_x,
                        repeat_y: layer.repeat_y,
                        tile_spacing: layer.tile_spacing,
                        paint_shader: Some(shader.clone()),
                        ..Default::default()
                    })?;
                } else if layer.image_width > 0
                    && layer.image_height > 0
                    && !layer.image_rgba.is_empty()
                {
                    canvas.draw_tiled_image(
                        &DrawCommand {
                            r#type: CommandKind::kDrawTiledImage,
                            rect: layer.clip_rect,
                            source_rect: layer.source_rect,
                            tile_rect: layer.tile_rect,
                            resource_id: layer.resource_id,
                            image_interpolation_quality: layer.image_interpolation_quality,
                            repeat_x: layer.repeat_x,
                            repeat_y: layer.repeat_y,
                            tile_spacing: layer.tile_spacing,
                            ..Default::default()
                        },
                        &resources,
                        SkBlendMode::kSrcOver,
                        false,
                    )?;
                }
            }
            canvas.finish_replay()?;
        }
        target
            .finish()
            .map_err(|error| raster_error("reflection mask finish failed", error))?;
    }
    let info = SkImageInfo::MakeWithColorSpace(
        width,
        height,
        SkColorType::kRGBA_8888_SkColorType,
        SkAlphaType::kPremul_SkAlphaType,
        None,
    );
    let pixmap = SkPixmap::new(&info, pixels.as_ptr(), row_bytes);
    let image = unsafe { skia::src::image::SkImage_RasterFactories::RasterFromPixmapCopy(&pixmap) }
        .ok_or_else(|| io::Error::other("reflection mask image allocation failed"))?;
    let image_rect = SkRect::MakeWH(width as f32, height as f32);
    let placed_rect = SkRect::MakeXYWH(
        bounds.x as f32,
        bounds.y as f32,
        width as f32,
        height as f32,
    );
    let placed = imagefilters::SkImageImageFilter::Image(
        Some(image),
        image_rect,
        placed_rect,
        SkSamplingOptions::with_filter(SkFilterMode::kLinear),
    );
    Ok(imagefilters::SkCropImageFilter::Crop(
        rect(bounds),
        SkTileMode::kDecal,
        placed,
    ))
}

fn translated_reflection_filter(
    direction: command::PaintReflectionDirection,
    offset: f64,
    has_mask: bool,
    mask_layers: &[raster_commands::filter_graph::RenderReflectionMaskLayer],
    input: Option<Arc<dyn SkImageFilter>>,
) -> io::Result<Option<Arc<dyn SkImageFilter>>> {
    use skia::src::effects::imagefilters;

    let masked_input = if has_mask {
        let mask = translated_reflection_mask_filter(mask_layers)?;
        let crop = mask_layers.first().and_then(|_| {
            mask_layers.iter().fold(None, |bounds, layer| {
                let clip = rect(layer.clip_rect);
                if clip.isEmpty() {
                    return bounds;
                }
                Some(bounds.map_or(clip, |mut current: SkRect| {
                    current.join(&clip);
                    current
                }))
            })
        });
        imagefilters::SkBlendImageFilter::Blend(SkBlendMode::kSrcIn, mask, input.clone(), crop)
    } else {
        input.clone()
    };
    let mut flip =
        match direction {
            command::PaintReflectionDirection::kAbove
            | command::PaintReflectionDirection::kBelow => SkMatrix::Scale(1.0, -1.0),
            command::PaintReflectionDirection::kLeft
            | command::PaintReflectionDirection::kRight => SkMatrix::Scale(-1.0, 1.0),
        };
    match direction {
        command::PaintReflectionDirection::kAbove | command::PaintReflectionDirection::kBelow => {
            flip.postTranslate(0.0, offset as f32)
        }
        command::PaintReflectionDirection::kLeft | command::PaintReflectionDirection::kRight => {
            flip.postTranslate(offset as f32, 0.0)
        }
    };
    let reflected = imagefilters::SkMatrixTransformImageFilter::MatrixTransform(
        &flip,
        SkSamplingOptions::with_filter(SkFilterMode::kLinear),
        masked_input,
    );
    Ok(imagefilters::SkBlendImageFilter::Blend(
        SkBlendMode::kSrcOver,
        input,
        reflected,
        None,
    ))
}

// Source: fe_component_transfer.cc:42-100,148-166. Chromium evaluates every
// component-transfer function in double precision, clamps the result in the
// byte domain and then truncates through the uint8_t conversion.
fn svg_component_transfer_table(
    function: &raster_commands::filter_graph::RenderSvgTransfer,
) -> [u8; 256] {
    use raster_commands::filter_graph::RenderSvgTransfer;

    let clamp_to_u8 = |value: f64| value.clamp(0.0, 255.0) as u8;
    std::array::from_fn(|index| match function {
        RenderSvgTransfer::Identity => index as u8,
        RenderSvgTransfer::Table(values) => {
            if values.is_empty() {
                index as u8
            } else {
                let c = index as f64 / 255.0;
                let last = values.len() - 1;
                let position = c * last as f64;
                let at = position as usize;
                let first = values[at] as f64;
                let second = values[(at + 1).min(last)] as f64;
                clamp_to_u8(255.0 * (first + (position - at as f64) * (second - first)))
            }
        }
        RenderSvgTransfer::Discrete(values) => {
            if values.is_empty() {
                index as u8
            } else {
                let at =
                    ((index as f64 * values.len() as f64 / 255.0) as usize).min(values.len() - 1);
                clamp_to_u8(255.0 * values[at] as f64)
            }
        }
        RenderSvgTransfer::Linear { slope, intercept } => {
            clamp_to_u8(*slope as f64 * index as f64 + 255.0 * *intercept as f64)
        }
        RenderSvgTransfer::Gamma {
            amplitude,
            exponent,
            offset,
        } => clamp_to_u8(
            255.0
                * (*amplitude as f64 * (index as f64 / 255.0).powf(*exponent as f64)
                    + *offset as f64),
        ),
    })
}

// Source: fe_turbulence.cc:131-133. Chromium's saturated_cast<int32_t>
// truncates finite fractional extents toward zero and clamps overflow.
// Rust's float-to-int cast has exactly those saturation/truncation semantics.
fn turbulence_tile_extent(value: f32) -> i32 {
    value as i32
}

fn turbulence_unscaled_tile_extent(parameter_extent: f32, filter_scale: f32) -> i32 {
    turbulence_tile_extent(parameter_extent / filter_scale)
}

// Source: fe_lighting.cc:140-148. Blink derives the PaintFilter material from
// the clamped specular constant, rather than solely from the DOM element type.
// In particular, feSpecularLighting with a non-positive specularConstant uses
// a diffuse material whose (diffuse) constant is zero.
fn resolved_lighting_type(
    authored: raster_commands::filter_graph::RenderSvgLightingType,
    constant: f32,
) -> raster_commands::filter_graph::RenderSvgLightingType {
    use raster_commands::filter_graph::RenderSvgLightingType;
    if authored == RenderSvgLightingType::Specular && constant > 0.0 {
        RenderSvgLightingType::Specular
    } else {
        RenderSvgLightingType::Diffuse
    }
}

// Source: svg_preserve_aspect_ratio.cc:201-303. FEImage passes its primitive
// subregion as `dest_rect` and the oriented image bounds as `src_rect`; meet
// changes the destination while slice crops the source.
fn svg_preserve_aspect_ratio_rects(
    mut dest: SkRect,
    mut src: SkRect,
    align: raster_commands::filter_graph::RenderSvgPreserveAspectRatioAlign,
    meet_or_slice: raster_commands::filter_graph::RenderSvgMeetOrSlice,
) -> (SkRect, SkRect) {
    use raster_commands::filter_graph::{RenderSvgMeetOrSlice, RenderSvgPreserveAspectRatioAlign};
    if align == RenderSvgPreserveAspectRatioAlign::None {
        return (dest, src);
    }
    let image_width = src.width();
    let image_height = src.height();
    let dest_width = dest.width();
    let dest_height = dest.height();
    let image_height_per_width = src.height() / src.width();
    match meet_or_slice {
        RenderSvgMeetOrSlice::Meet => {
            if dest_height > dest_width * image_height_per_width {
                let height = dest_width * image_height_per_width;
                let offset = match align {
                    RenderSvgPreserveAspectRatioAlign::XMinYMid
                    | RenderSvgPreserveAspectRatioAlign::XMidYMid
                    | RenderSvgPreserveAspectRatioAlign::XMaxYMid => (dest_height - height) / 2.0,
                    RenderSvgPreserveAspectRatioAlign::XMinYMax
                    | RenderSvgPreserveAspectRatioAlign::XMidYMax
                    | RenderSvgPreserveAspectRatioAlign::XMaxYMax => dest_height - height,
                    _ => 0.0,
                };
                dest.fTop += offset;
                dest.fBottom = dest.fTop + height;
            }
            if dest_width > dest_height / image_height_per_width {
                let width = dest_height / image_height_per_width;
                let offset = match align {
                    RenderSvgPreserveAspectRatioAlign::XMidYMin
                    | RenderSvgPreserveAspectRatioAlign::XMidYMid
                    | RenderSvgPreserveAspectRatioAlign::XMidYMax => (dest_width - width) / 2.0,
                    RenderSvgPreserveAspectRatioAlign::XMaxYMin
                    | RenderSvgPreserveAspectRatioAlign::XMaxYMid
                    | RenderSvgPreserveAspectRatioAlign::XMaxYMax => dest_width - width,
                    _ => 0.0,
                };
                dest.fLeft += offset;
                dest.fRight = dest.fLeft + width;
            }
        }
        RenderSvgMeetOrSlice::Slice => {
            if dest_height < dest_width * image_height_per_width {
                let height = dest.height() * src.width() / dest.width();
                let offset = match align {
                    RenderSvgPreserveAspectRatioAlign::XMinYMid
                    | RenderSvgPreserveAspectRatioAlign::XMidYMid
                    | RenderSvgPreserveAspectRatioAlign::XMaxYMid => (image_height - height) / 2.0,
                    RenderSvgPreserveAspectRatioAlign::XMinYMax
                    | RenderSvgPreserveAspectRatioAlign::XMidYMax
                    | RenderSvgPreserveAspectRatioAlign::XMaxYMax => image_height - height,
                    _ => 0.0,
                };
                src.fTop += offset;
                src.fBottom = src.fTop + height;
            }
            if dest_width < dest_height / image_height_per_width {
                let width = dest.width() * src.height() / dest.height();
                let offset = match align {
                    RenderSvgPreserveAspectRatioAlign::XMidYMin
                    | RenderSvgPreserveAspectRatioAlign::XMidYMid
                    | RenderSvgPreserveAspectRatioAlign::XMidYMax => (image_width - width) / 2.0,
                    RenderSvgPreserveAspectRatioAlign::XMaxYMin
                    | RenderSvgPreserveAspectRatioAlign::XMaxYMid
                    | RenderSvgPreserveAspectRatioAlign::XMaxYMax => image_width - width,
                    _ => 0.0,
                };
                src.fLeft += offset;
                src.fRight = src.fLeft + width;
            }
        }
    }
    (dest, src)
}

// Source: filter_effect.cc:144-163. SVG effects clip to the intersection of
// their authored primitive subregion and the filter region. The feature that
// makes this unconditional is stable in this Chromium revision.
fn svg_filter_crop(mut primitive_subregion: SkRect, filter_region: SkRect) -> SkRect {
    if !primitive_subregion.intersect(&filter_region) {
        SkRect::MakeEmpty()
    } else {
        primitive_subregion
    }
}

// Source: ui/gfx/geometry/rect_f.cc:373-381. FEImage uses this to select the
// source portion corresponding to its crop after preserveAspectRatio changes
// the full primitive destination rectangle.
fn map_svg_rect(rect: SkRect, source: SkRect, destination: SkRect) -> SkRect {
    if source.isEmpty() {
        return SkRect::MakeEmpty();
    }
    let width_scale = destination.width() / source.width();
    let height_scale = destination.height() / source.height();
    SkRect::MakeXYWH(
        destination.x() + (rect.x() - source.x()) * width_scale,
        destination.y() + (rect.y() - source.y()) * height_scale,
        rect.width() * width_scale,
        rect.height() * height_scale,
    )
}

// Source: cc/paint/paint_filter.cc:753-769 and paint_flags.cc:178-207.
// FEImage constructs ImagePaintFilter with kHigh quality. ImagePaintFilter
// classifies a strict two-axis enlargement as kUpscale; every other geometry
// is kUnknown. Those two cases deliberately use different Skia sampling.
fn fe_image_sampling(source: SkRect, destination: SkRect) -> SkSamplingOptions {
    if source.height() < destination.height() && source.width() < destination.width() {
        SkSamplingOptions::with_cubic(SkCubicResampler::Mitchell())
    } else {
        SkSamplingOptions::with_filter_mipmap(SkFilterMode::kLinear, SkMipmapMode::kLinear)
    }
}

// Source: third_party/blink/renderer/platform/graphics/graphics_context.h:495-505
// and cc/paint/paint_flags.cc:178-207. GraphicsContext classifies ordinary
// image draws as kUpscale only when both axes grow and kUnknown otherwise.
// Medium quality deliberately uses linear filtering with linear mipmaps for
// both classifications; preserving the mipmap mode matters for normal bitmap
// downscales such as object-fit: contain.
fn image_sampling(quality: ImageInterpolationQuality) -> SkSamplingOptions {
    match quality {
        ImageInterpolationQuality::kNone => SkSamplingOptions::with_filter(SkFilterMode::kNearest),
        ImageInterpolationQuality::kLow => SkSamplingOptions::with_filter(SkFilterMode::kLinear),
        ImageInterpolationQuality::kMedium => {
            SkSamplingOptions::with_filter_mipmap(SkFilterMode::kLinear, SkMipmapMode::kLinear)
        }
    }
}

fn translated_reference_filter(
    operation: &layoutng_assembly::internal::paint_input::PaintFilterOperation,
    reference_box: paint::PaintRect,
    reference_viewport: Option<paint::PaintRect>,
    reference_origin: layoutng_assembly::internal::layout_input::Offset,
    reference_scale: f64,
    matrix: &SkM44,
    source: Option<Arc<dyn SkImageFilter>>,
    source_linear_rgb: bool,
    source_origin_tainted: bool,
) -> io::Result<TranslatedFilterValue> {
    use raster_commands::filter_graph::{
        RenderSvgBlend, RenderSvgComposite, RenderSvgEdgeMode, RenderSvgFilterKind, RenderSvgLight,
        RenderSvgLightingType, RenderSvgTransfer,
    };
    use skia::include::core::SkSize::SkISize;
    use skia::src::core::{
        SkColor::{SkColorChannel, SkColorSetRGB},
        SkPoint::SkIPoint,
        SkPoint3::SkPoint3,
    };
    use skia::src::effects::{colorfilters, imagefilters};

    if let Some(error) = &operation.reference_error {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("SVG reference filter: {error}"),
        ));
    }
    let graph = operation.reference_graph.as_ref().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::Unsupported,
            "SVG reference filter has no resolved graph",
        )
    })?;
    let scale_x = matrix.rc(0, 0).hypot(matrix.rc(0, 1));
    let scale_y = matrix.rc(1, 0).hypot(matrix.rc(1, 1));
    let graph = crate::svg_filter_graph::TranslateSvgFilterGraphWithTransform(
        graph,
        reference_box,
        reference_viewport,
        f64::from(scale_x),
        f64::from(scale_y),
        f64::from(matrix.rc(0, 3)),
        f64::from(matrix.rc(1, 3)),
        reference_origin.x,
        reference_origin.y,
        reference_scale,
    )?;
    let crop = |region: [f32; 4]| {
        SkRect::MakeXYWH(region[0], region[1], region[2].max(0.0), region[3].max(0.0))
    };
    let filter_region = crop(graph.region);
    let source_alpha = |input: Option<Arc<dyn SkImageFilter>>| {
        let mut matrix = [0.0; 20];
        matrix[18] = 1.0;
        colorfilters::SkMatrixColorFilter::Matrix(
            &matrix,
            skia::src::core::SkColorFilter::Clamp::kYes,
        )
        .and_then(|color| {
            imagefilters::SkColorFilterImageFilter::ColorFilter(Some(color), input, None)
        })
    };
    let solid = |color: [f32; 4], region: SkRect| -> io::Result<Option<Arc<dyn SkImageFilter>>> {
        let color = SkColor4f {
            fR: color[0],
            fG: color[1],
            fB: color[2],
            fA: color[3],
        };
        let shader = skia::src::shaders::SkColorShader::Color4f(&color, None)
            .ok_or_else(|| io::Error::other("translated Skia rejected SVG solid color"))?;
        Ok(imagefilters::SkShaderImageFilter::Shader(
            Some(shader),
            false,
            Some(region),
        ))
    };
    let fill_paint = graph
        .fill_paint
        .map(|color| solid(color, crop(graph.region)))
        .transpose()?
        .flatten()
        .map(|filter| SvgFilterValue::shared(Some(filter), false, source_origin_tainted));
    let stroke_paint = graph
        .stroke_paint
        .map(|color| solid(color, crop(graph.region)))
        .transpose()?
        .flatten()
        .map(|filter| SvgFilterValue::shared(Some(filter), false, source_origin_tainted));
    let channel = |value: usize| -> io::Result<SkColorChannel> {
        match value {
            0 => Ok(SkColorChannel::kR),
            1 => Ok(SkColorChannel::kG),
            2 => Ok(SkColorChannel::kB),
            3 => Ok(SkColorChannel::kA),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "SVG displacement-map channel is outside RGBA",
            )),
        }
    };
    let source = SvgFilterValue::shared(source, source_linear_rgb, source_origin_tainted);
    let mut named: std::collections::HashMap<String, SvgFilterValue> =
        std::collections::HashMap::new();
    let mut previous: Option<SvgFilterValue> = None;
    let lookup = |name: &str,
                  previous: &Option<SvgFilterValue>,
                  named: &std::collections::HashMap<String, SvgFilterValue>|
     -> io::Result<SvgFilterValue> {
        match name {
            "" => Ok(previous.clone().unwrap_or_else(|| source.clone())),
            "SourceGraphic" => Ok(source.clone()),
            "SourceAlpha" => source_alpha(source.validated.clone())
                .map(|filter| {
                    SvgFilterValue::shared(Some(filter), source.linear_rgb, source.origin_tainted)
                })
                .ok_or_else(|| io::Error::other("translated Skia rejected SVG SourceAlpha filter")),
            // Fixed Chromium resolves these unsupported built-ins through the
            // previous-effect/SourceGraphic fallback in SVGFilterBuilder.
            "BackgroundImage" | "BackgroundAlpha" => {
                Ok(previous.clone().unwrap_or_else(|| source.clone()))
            }
            "FillPaint" => Ok(fill_paint
                .clone()
                .or_else(|| previous.clone())
                .or_else(|| Some(source.clone()))
                .expect("SourceGraphic is always a filter value")),
            "StrokePaint" => Ok(stroke_paint
                .clone()
                .or_else(|| previous.clone())
                .or_else(|| Some(source.clone()))
                .expect("SourceGraphic is always a filter value")),
            value => Ok(named
                .get(value)
                .cloned()
                .or_else(|| previous.clone())
                .unwrap_or_else(|| source.clone())),
        }
    };
    for primitive in &graph.primitives {
        let arithmetic = matches!(
            &primitive.kind,
            RenderSvgFilterKind::Composite {
                operator: RenderSvgComposite::Arithmetic,
                ..
            }
        );
        // Source: fe_composite.cc:179-194 and paint_filter_builder.cc:66-78.
        // Arithmetic may carry invalid premultiplied intermediates into a
        // following arithmetic primitive; every other consumer requests the
        // validated variant of its input.
        let input_requires_validation = !arithmetic;
        let input_value = lookup(&primitive.input, &previous, &named)?;
        let inputs_origin_tainted = match &primitive.kind {
            RenderSvgFilterKind::Flood(_)
            | RenderSvgFilterKind::Turbulence { .. }
            | RenderSvgFilterKind::Image { .. } => false,
            RenderSvgFilterKind::Blend { input2, .. }
            | RenderSvgFilterKind::Composite { input2, .. }
            | RenderSvgFilterKind::DisplacementMap { input2, .. } => {
                input_value.origin_tainted || lookup(input2, &previous, &named)?.origin_tainted
            }
            RenderSvgFilterKind::Merge(inputs) => {
                inputs.iter().try_fold(false, |tainted, name| {
                    Ok::<_, io::Error>(tainted || lookup(name, &previous, &named)?.origin_tainted)
                })?
            }
            _ => input_value.origin_tainted,
        };
        let output_origin_tainted = primitive.origin_tainted || inputs_origin_tainted;
        let input = translated_filter_space(
            input_value.select(input_requires_validation),
            input_value.linear_rgb,
            primitive.linear_rgb,
        );
        let primitive_subregion = crop(primitive.region);
        let primitive_crop = svg_filter_crop(primitive_subregion, filter_region);
        // Source: svg_fe_image.h:43-45 and svg_fe_image.cc:94-110. FEImage
        // ignores SVGFilterBuilder's operating-space setter and always emits
        // sRGB; every other SVG primitive uses its resolved interpolation.
        let output_linear_rgb =
            primitive.linear_rgb && !matches!(&primitive.kind, RenderSvgFilterKind::Image { .. });
        let arithmetic_variants = if let RenderSvgFilterKind::Composite {
            input2,
            operator: RenderSvgComposite::Arithmetic,
            k,
        } = &primitive.kind
        {
            let background = lookup(input2, &previous, &named)?;
            let background = translated_filter_space(
                background.select(false),
                background.linear_rgb,
                primitive.linear_rgb,
            );
            Some(svg_arithmetic_filter_variants(
                *k,
                background,
                input.clone(),
                primitive_crop,
            ))
        } else {
            None
        };
        let output = match &primitive.kind {
            RenderSvgFilterKind::Offset { x, y } => {
                imagefilters::SkMatrixTransformImageFilter::Offset(
                    *x,
                    *y,
                    input,
                    Some(primitive_crop),
                )
            }
            RenderSvgFilterKind::GaussianBlur { x, y } => imagefilters::SkBlurImageFilter::Blur(
                *x,
                *y,
                SkTileMode::kDecal,
                input,
                Some(primitive_crop),
            ),
            RenderSvgFilterKind::DropShadow {
                sigma_x,
                sigma_y,
                dx,
                dy,
                color,
            } => {
                // Source: fe_drop_shadow.cc:66-83. Adapt the authored flood
                // color to the primitive operating space before constructing
                // Skia's draw-shadow-and-foreground graph.
                let adapt = |value: f32| {
                    let value = value.clamp(0.0, 1.0);
                    if primitive.linear_rgb {
                        if value <= 0.04045 {
                            value / 12.92
                        } else {
                            ((value + 0.055) / 1.055).powf(2.4)
                        }
                    } else {
                        value
                    }
                };
                imagefilters::SkDropShadowImageFilter::DropShadow(
                    *dx,
                    *dy,
                    *sigma_x,
                    *sigma_y,
                    SkColor4f {
                        fR: adapt(color[0]),
                        fG: adapt(color[1]),
                        fB: adapt(color[2]),
                        fA: color[3].clamp(0.0, 1.0),
                    },
                    None,
                    input,
                    Some(primitive_crop),
                )
            }
            RenderSvgFilterKind::ColorMatrix(values) => {
                let color = colorfilters::SkMatrixColorFilter::Matrix(
                    values,
                    skia::src::core::SkColorFilter::Clamp::kYes,
                )
                .ok_or_else(|| io::Error::other("translated Skia rejected SVG color matrix"))?;
                imagefilters::SkColorFilterImageFilter::ColorFilter(
                    Some(color),
                    input,
                    Some(primitive_crop),
                )
            }
            RenderSvgFilterKind::Flood(color) => {
                translated_filter_space(solid(*color, primitive_crop)?, false, primitive.linear_rgb)
            }
            RenderSvgFilterKind::Blend { input2, mode } => {
                let background = lookup(input2, &previous, &named)?;
                let background = translated_filter_space(
                    background.select(input_requires_validation),
                    background.linear_rgb,
                    primitive.linear_rgb,
                );
                let mode = match mode {
                    RenderSvgBlend::Normal => SkBlendMode::kSrcOver,
                    RenderSvgBlend::Multiply => SkBlendMode::kMultiply,
                    RenderSvgBlend::Screen => SkBlendMode::kScreen,
                    RenderSvgBlend::Darken => SkBlendMode::kDarken,
                    RenderSvgBlend::Lighten => SkBlendMode::kLighten,
                    RenderSvgBlend::Overlay => SkBlendMode::kOverlay,
                    RenderSvgBlend::ColorDodge => SkBlendMode::kColorDodge,
                    RenderSvgBlend::ColorBurn => SkBlendMode::kColorBurn,
                    RenderSvgBlend::HardLight => SkBlendMode::kHardLight,
                    RenderSvgBlend::SoftLight => SkBlendMode::kSoftLight,
                    RenderSvgBlend::Difference => SkBlendMode::kDifference,
                    RenderSvgBlend::Exclusion => SkBlendMode::kExclusion,
                    RenderSvgBlend::Hue => SkBlendMode::kHue,
                    RenderSvgBlend::Saturation => SkBlendMode::kSaturation,
                    RenderSvgBlend::Color => SkBlendMode::kColor,
                    RenderSvgBlend::Luminosity => SkBlendMode::kLuminosity,
                };
                imagefilters::SkBlendImageFilter::Blend(
                    mode,
                    background,
                    input,
                    Some(primitive_crop),
                )
            }
            RenderSvgFilterKind::Composite {
                input2,
                operator,
                k,
            } => {
                let background = lookup(input2, &previous, &named)?;
                let background = translated_filter_space(
                    background.select(input_requires_validation),
                    background.linear_rgb,
                    primitive.linear_rgb,
                );
                match operator {
                    RenderSvgComposite::Arithmetic => arithmetic_variants
                        .as_ref()
                        .expect("arithmetic variants")
                        .0
                        .clone(),
                    operator => {
                        let mode = match operator {
                            RenderSvgComposite::Over => SkBlendMode::kSrcOver,
                            RenderSvgComposite::In => SkBlendMode::kSrcIn,
                            RenderSvgComposite::Out => SkBlendMode::kSrcOut,
                            RenderSvgComposite::Atop => SkBlendMode::kSrcATop,
                            RenderSvgComposite::Xor => SkBlendMode::kXor,
                            RenderSvgComposite::Lighter => SkBlendMode::kPlus,
                            RenderSvgComposite::Arithmetic => unreachable!(),
                        };
                        imagefilters::SkBlendImageFilter::Blend(
                            mode,
                            background,
                            input,
                            Some(primitive_crop),
                        )
                    }
                }
            }
            RenderSvgFilterKind::Merge(inputs) => {
                let filters = inputs
                    .iter()
                    .map(|name| {
                        lookup(name, &previous, &named).map(|input| {
                            translated_filter_space(
                                input.select(input_requires_validation),
                                input.linear_rgb,
                                primitive.linear_rgb,
                            )
                        })
                    })
                    .collect::<io::Result<Vec<_>>>()?;
                imagefilters::SkMergeImageFilter::Merge(Some(&filters), Some(primitive_crop))
            }
            RenderSvgFilterKind::ComponentTransfer(functions) => {
                let tables = functions.each_ref().map(svg_component_transfer_table);
                let color = colorfilters::SkTableColorFilter::TableARGB(
                    Some(&tables[3]),
                    Some(&tables[0]),
                    Some(&tables[1]),
                    Some(&tables[2]),
                )
                .ok_or_else(|| {
                    io::Error::other("translated Skia rejected SVG component-transfer tables")
                })?;
                imagefilters::SkColorFilterImageFilter::ColorFilter(
                    Some(color),
                    input,
                    Some(primitive_crop),
                )
            }
            RenderSvgFilterKind::Morphology { dilate, x, y } => {
                if *dilate {
                    imagefilters::SkMorphologyImageFilter::Dilate(
                        *x,
                        *y,
                        input,
                        Some(primitive_crop),
                    )
                } else {
                    imagefilters::SkMorphologyImageFilter::Erode(
                        *x,
                        *y,
                        input,
                        Some(primitive_crop),
                    )
                }
            }
            RenderSvgFilterKind::Tile { source_region } => imagefilters::SkCropImageFilter::Tile(
                crop(*source_region),
                primitive_subregion,
                input,
            ),
            RenderSvgFilterKind::DisplacementMap {
                input2,
                scale,
                x_channel,
                y_channel,
            } => {
                let displacement = lookup(input2, &previous, &named)?;
                if displacement.origin_tainted {
                    input
                } else {
                    let displacement = translated_filter_space(
                        displacement.select(input_requires_validation),
                        displacement.linear_rgb,
                        primitive.linear_rgb,
                    );
                    imagefilters::SkDisplacementMapImageFilter::DisplacementMap(
                        channel(*x_channel)?,
                        channel(*y_channel)?,
                        *scale,
                        displacement,
                        input,
                        Some(primitive_crop),
                    )
                }
            }
            RenderSvgFilterKind::Turbulence {
                base_x,
                base_y,
                octaves,
                seed,
                stitch,
                fractal_noise,
            } => {
                let (base_x, base_y) = if *base_x < 0.0 || *base_y < 0.0 {
                    (0.0, 0.0)
                } else {
                    (*base_x, *base_y)
                };
                let scale = reference_scale.max(f64::EPSILON) as f32;
                let tile = stitch.then(|| {
                    // Source: fe_turbulence.cc:131-140. The stitch tile is the
                    // unscaled FilterPrimitiveSubregion size, while the base
                    // frequency alone is divided by Filter::Scale(). `crop`
                    // subregion is already in zoomed image-filter parameter
                    // space here; the crop may be smaller at the filter edge.
                    SkISize::Make(
                        turbulence_unscaled_tile_extent(primitive_subregion.width(), scale),
                        turbulence_unscaled_tile_extent(primitive_subregion.height(), scale),
                    )
                });
                let shader = if *fractal_noise {
                    skia::src::shaders::SkPerlinNoiseShaderImpl::MakeFractalNoise(
                        base_x / scale,
                        base_y / scale,
                        (*octaves).min(9),
                        *seed,
                        tile.as_ref(),
                    )
                } else {
                    skia::src::shaders::SkPerlinNoiseShaderImpl::MakeTurbulence(
                        base_x / scale,
                        base_y / scale,
                        (*octaves).min(9),
                        *seed,
                        tile.as_ref(),
                    )
                };
                imagefilters::SkShaderImageFilter::Shader(shader, false, Some(primitive_crop))
            }
            RenderSvgFilterKind::Image {
                href,
                local_subtree,
                align,
                meet_or_slice,
                width,
                height,
                orientation,
                pixels,
            } => {
                if *local_subtree {
                    return Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        format!("feImage local subtree {href:?} has no compiled PaintRecord"),
                    ));
                }
                if *width == 0 || *height == 0 || pixels.is_empty() {
                    imagefilters::SkCropImageFilter::Empty()
                } else {
                    // Source: svg_fe_image.cc:267-288. The stable feature
                    // always applies encoded orientation before querying the
                    // source size and constructing the image filter.
                    let (oriented_pixels, oriented_width, oriented_height) = orient_image_pixels(
                        pixels.as_ref().clone(),
                        *width as usize,
                        *height as usize,
                        *orientation,
                    )?;
                    let oriented_pixels = Arc::new(oriented_pixels);
                    let resources = ResourceContext {
                        resources: Some(command::ResourceCatalog {
                            fonts: Vec::new(),
                            images: vec![command::Image::shared(
                                0,
                                oriented_width as u32,
                                oriented_height as u32,
                                &oriented_pixels,
                            )],
                        }),
                    };
                    let image = raster_image(
                        &DrawCommand {
                            resource_id: 0,
                            ..Default::default()
                        },
                        &resources,
                    )?;
                    let (aspect_destination, aspect_source) = svg_preserve_aspect_ratio_rects(
                        primitive_subregion,
                        SkRect::MakeWH(oriented_width as f32, oriented_height as f32),
                        *align,
                        *meet_or_slice,
                    );
                    let source = if primitive_crop == aspect_destination {
                        aspect_source
                    } else {
                        map_svg_rect(primitive_crop, aspect_destination, aspect_source)
                    };
                    let filter = imagefilters::SkImageImageFilter::Image(
                        image,
                        source,
                        primitive_crop,
                        fe_image_sampling(source, primitive_crop),
                    );
                    filter
                }
            }
            RenderSvgFilterKind::Convolve {
                order_x,
                order_y,
                kernel,
                divisor,
                bias,
                target_x,
                target_y,
                edge_mode,
                preserve_alpha,
            } => {
                let kernel_area = order_x
                    .checked_mul(*order_y)
                    .filter(|area| *area <= i32::MAX as u32);
                if kernel_area.is_none()
                    || kernel_area.map(|area| area as usize) != Some(kernel.len())
                    || *target_x < 0
                    || *target_y < 0
                    || *target_x as u32 >= *order_x
                    || *target_y as u32 >= *order_y
                {
                    imagefilters::SkCropImageFilter::Empty()
                } else {
                    let size = SkISize::Make(
                        i32::try_from(*order_x).expect("validated convolution width"),
                        i32::try_from(*order_y).expect("validated convolution height"),
                    );
                    let target = SkIPoint::make(*target_x, *target_y);
                    let tile = match edge_mode {
                        RenderSvgEdgeMode::None => SkTileMode::kDecal,
                        RenderSvgEdgeMode::Duplicate => SkTileMode::kClamp,
                        RenderSvgEdgeMode::Wrap => SkTileMode::kRepeat,
                    };
                    let reversed = kernel.iter().rev().copied().collect::<Vec<_>>();
                    let divisor = if *divisor == 0.0 {
                        let sum = kernel.iter().sum::<f32>();
                        if sum == 0.0 {
                            1.0
                        } else {
                            sum
                        }
                    } else {
                        *divisor
                    };
                    imagefilters::SkMatrixConvolutionImageFilter::MatrixConvolution(
                        size,
                        Some(&reversed),
                        divisor.recip(),
                        bias * 255.0,
                        target,
                        tile,
                        !preserve_alpha,
                        input,
                        Some(primitive_crop),
                    )
                    .or_else(imagefilters::SkCropImageFilter::Empty)
                }
            }
            RenderSvgFilterKind::Lighting {
                lighting_type,
                light,
                color,
                surface_scale,
                constant,
                shininess,
            } => {
                let constant = constant.max(0.0);
                let shininess = shininess.clamp(1.0, 128.0);
                let lighting_type = resolved_lighting_type(*lighting_type, constant);
                let adapt = |value: f32| {
                    let value = value.clamp(0.0, 1.0);
                    let value = if primitive.linear_rgb {
                        if value <= 0.04045 {
                            value / 12.92
                        } else {
                            ((value + 0.055) / 1.055).powf(2.4)
                        }
                    } else {
                        value
                    };
                    (value * 255.0).round() as u32
                };
                let color = SkColorSetRGB(adapt(color[0]), adapt(color[1]), adapt(color[2]));
                let point = |value: [f32; 3]| SkPoint3::Make(value[0], value[1], value[2]);
                match (&lighting_type, light) {
                    (
                        RenderSvgLightingType::Diffuse,
                        Some(RenderSvgLight::Distant { direction }),
                    ) => imagefilters::SkLightingImageFilter::DistantLitDiffuse(
                        point(*direction),
                        color,
                        *surface_scale,
                        constant,
                        input,
                        Some(primitive_crop),
                    ),
                    (RenderSvgLightingType::Diffuse, Some(RenderSvgLight::Point { position })) => {
                        imagefilters::SkLightingImageFilter::PointLitDiffuse(
                            point(*position),
                            color,
                            *surface_scale,
                            constant,
                            input,
                            Some(primitive_crop),
                        )
                    }
                    (
                        RenderSvgLightingType::Diffuse,
                        Some(RenderSvgLight::Spot {
                            position,
                            target,
                            falloff_exponent,
                            limiting_cone_angle,
                        }),
                    ) => imagefilters::SkLightingImageFilter::SpotLitDiffuse(
                        point(*position),
                        point(*target),
                        *falloff_exponent,
                        if *limiting_cone_angle == 0.0
                            || !(-90.0..=90.0).contains(limiting_cone_angle)
                        {
                            90.0
                        } else {
                            *limiting_cone_angle
                        },
                        color,
                        *surface_scale,
                        constant,
                        input,
                        Some(primitive_crop),
                    ),
                    (
                        RenderSvgLightingType::Specular,
                        Some(RenderSvgLight::Distant { direction }),
                    ) => imagefilters::SkLightingImageFilter::DistantLitSpecular(
                        point(*direction),
                        color,
                        *surface_scale,
                        constant,
                        shininess,
                        input,
                        Some(primitive_crop),
                    ),
                    (RenderSvgLightingType::Specular, Some(RenderSvgLight::Point { position })) => {
                        imagefilters::SkLightingImageFilter::PointLitSpecular(
                            point(*position),
                            color,
                            *surface_scale,
                            constant,
                            shininess,
                            input,
                            Some(primitive_crop),
                        )
                    }
                    (
                        RenderSvgLightingType::Specular,
                        Some(RenderSvgLight::Spot {
                            position,
                            target,
                            falloff_exponent,
                            limiting_cone_angle,
                        }),
                    ) => imagefilters::SkLightingImageFilter::SpotLitSpecular(
                        point(*position),
                        point(*target),
                        *falloff_exponent,
                        if *limiting_cone_angle == 0.0
                            || !(-90.0..=90.0).contains(limiting_cone_angle)
                        {
                            90.0
                        } else {
                            *limiting_cone_angle
                        },
                        color,
                        *surface_scale,
                        constant,
                        shininess,
                        input,
                        Some(primitive_crop),
                    ),
                    (_, None) => imagefilters::SkCropImageFilter::Empty(),
                }
                .or_else(imagefilters::SkCropImageFilter::Empty)
            }
            kind => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    format!("translated SVG filter primitive is not connected: {kind:?}"),
                ));
            }
        };
        let output = SvgFilterValue {
            validated: output.clone(),
            unvalidated: arithmetic_variants.map_or(output, |variants| variants.1),
            linear_rgb: output_linear_rgb,
            origin_tainted: output_origin_tainted,
        };
        let result_is_builtin = primitive.result == "SourceGraphic"
            || primitive.result == "SourceAlpha"
            || (primitive.result == "FillPaint" && graph.fill_paint.is_some())
            || (primitive.result == "StrokePaint" && graph.stroke_paint.is_some());
        if result_is_builtin {
            continue;
        }
        if !primitive.result.is_empty() {
            named.insert(primitive.result.clone(), output.clone());
        }
        previous = Some(output);
    }
    let output = previous.unwrap_or(source);
    Ok((
        skia::src::effects::imagefilters::SkCropImageFilter::Crop(
            filter_region,
            SkTileMode::kDecal,
            output.validated,
        ),
        output.linear_rgb,
    ))
}

fn luma_image_filter(
    input: Option<Arc<dyn SkImageFilter>>,
) -> io::Result<Option<Arc<dyn SkImageFilter>>> {
    let color = skia::src::effects::colorfilters::SkRuntimeColorFilter::MakeLumaColorFilter()
        .ok_or_else(|| io::Error::other("translated Skia rejected the luma color filter"))?;
    Ok(
        skia::src::effects::imagefilters::SkColorFilterImageFilter::ColorFilter(
            Some(color),
            input,
            None,
        ),
    )
}

#[derive(Clone)]
struct TranslatedLooperPaint(SkPaint);

impl paint::cc::paint::draw_looper::LooperPaint for TranslatedLooperPaint {
    fn setAlpha(&mut self, alpha: u8) {
        self.0.setAlphaf(f32::from(alpha) * (1. / 255.));
    }

    fn setMaskFilterNormalBlur(&mut self, sigma: f32, respect_ctm: bool) {
        self.0
            .setMaskFilter(skia::src::core::SkMaskFilter::MakeBlur(
                skia::include::core::SkBlurTypes::SkBlurStyle::kNormal_SkBlurStyle,
                sigma,
                respect_ctm,
            ));
    }

    fn setColorFilterSRGBBlend(
        &mut self,
        color: paint::cc::paint::color::Color4f,
        mode: paint::cc::paint::blend_mode::BlendMode,
    ) {
        let color = SkColor4f {
            fR: color.red,
            fG: color.green,
            fB: color.blue,
            fA: color.alpha,
        };
        self.0.setColorFilter(
            skia::src::effects::colorfilters::SkBlendModeColorFilter::Blend(
                &color,
                None,
                sk_blend_mode(mode),
            ),
        );
    }
}

fn command_paint(item: &DrawCommand, stroke: bool) -> SkPaint {
    let mut paint = SkPaint::default();
    paint.setColor4f(&color(item.color), None);
    paint.setAntiAlias(item.antialias);
    paint.setBlendMode(css_blend(item.blend_mode));
    if stroke {
        paint.setStyle(Style::kStroke_Style);
        paint.setStrokeWidth(item.stroke_width as f32);
        paint.setStrokeMiter(item.miter_limit as f32);
        paint.setStrokeCap(match item.svg_line_cap {
            command::SvgStrokeLineCap::kRound => Cap::kRound_Cap,
            command::SvgStrokeLineCap::kSquare => Cap::kSquare_Cap,
            command::SvgStrokeLineCap::kButt => Cap::kButt_Cap,
        });
        paint.setStrokeJoin(match item.svg_line_join {
            command::SvgStrokeLineJoin::kRound => Join::kRound_Join,
            command::SvgStrokeLineJoin::kBevel => Join::kBevel_Join,
            command::SvgStrokeLineJoin::kMiter => Join::kMiter_Join,
        });
        if !item.dash_intervals.is_empty() {
            let intervals = item
                .dash_intervals
                .iter()
                .map(|value| *value as f32)
                .collect::<Vec<_>>();
            paint.setPathEffect(skia::src::effects::SkDashPathEffect::Make(
                &intervals,
                item.dash_offset as f32,
            ));
        } else if item.corner_radius > 0.0 {
            // GraphicsContext::DrawFocusRingPath applies the platform corner
            // effect to its stroked generic path.
            paint.setPathEffect(skia::src::effects::SkCornerPathEffect::Make(
                item.corner_radius as f32,
            ));
        }
    }
    if item.blur_radius > 0. {
        paint.setMaskFilter(skia::src::core::SkMaskFilter::MakeBlur(
            skia::include::core::SkBlurTypes::SkBlurStyle::kNormal_SkBlurStyle,
            (item.blur_radius * 0.5) as f32,
            true,
        ));
    }
    paint
}

/// Source: cc/paint/paint_op_buffer.cc PaintFlags playback and
/// third_party/blink/renderer/platform/graphics/gradient.cc::ApplyToFlags.
///
/// SVG shape painters attach their resolved paint server to an ordinary
/// DrawRect/DrawOval/DrawPath operation.  Those operations must consume the
/// same PaintShader as the dedicated CSS-gradient operations; falling back to
/// DrawCommand::color here would incorrectly paint SVGPaint's fallback color.
fn command_paint_with_gradient_shader(item: &DrawCommand, stroke: bool) -> io::Result<SkPaint> {
    let mut paint = command_paint(item, stroke);
    let Some(shader) = item.paint_shader.as_ref() else {
        return Ok(paint);
    };
    if shader.kind == command::PaintShaderKind::kPattern {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "SVG pattern shader requires record-shader playback",
        ));
    }
    paint.setShader(Some(gradient_shader(shader)?));
    // Source: third_party/blink/renderer/platform/graphics/gradient.cc
    // Gradient::ApplyToFlags.  A resolved SVG paint server replaces the
    // fallback SVGPaint color; the fallback may be absent and therefore
    // transparent.  Leaving that transparent color on SkPaint multiplies the
    // shader alpha to zero and silently drops the gradient.  Chromium applies
    // the shader with an opaque neutral paint color and carries opacity in the
    // PaintFlags alpha instead.
    paint.setColor4f(
        &SkColor4f {
            fR: 1.,
            fG: 1.,
            fB: 1.,
            fA: shader.flags_alpha,
        },
        None,
    );
    paint.setDither(true);
    Ok(paint)
}

fn raster_image(
    item: &DrawCommand,
    resources: &ResourceContext<'_>,
) -> io::Result<Option<std::sync::Arc<dyn skia::src::image::SkImage::SkImage>>> {
    let Some(resource) = resources.resources.as_ref().and_then(|catalog| {
        catalog
            .images
            .iter()
            .find(|image| image.id == item.resource_id)
    }) else {
        // Chromium image providers may decline a decoded image for a frame.
        // The existing receiver treats that as an empty draw, not a fatal task.
        return Ok(None);
    };
    let width = i32::try_from(resource.width)
        .map_err(|_| io::Error::other("translated image width exceeds Skia limits"))?;
    let height = i32::try_from(resource.height)
        .map_err(|_| io::Error::other("translated image height exceeds Skia limits"))?;
    let row_bytes = usize::try_from(resource.width)
        .ok()
        .and_then(|width| width.checked_mul(4))
        .ok_or_else(|| io::Error::other("translated image row byte overflow"))?;
    let byte_len = row_bytes
        .checked_mul(resource.height as usize)
        .ok_or_else(|| io::Error::other("translated image byte count overflow"))?;
    if resource.rgba8.len() != byte_len || width <= 0 || height <= 0 {
        return Err(io::Error::other(
            "translated image bytes do not match their dimensions",
        ));
    }

    // Source: ImageFrame and WEBPImageDecoder::RGBOutputMode. Chromium keeps
    // decoder-produced premultiplied frames in their canonical representation;
    // only straight-alpha producers pass through SkConvertPixels here.
    let pixels = if resource.alpha_type == command::ImageAlphaType::Premultiplied {
        resource.rgba8.to_vec()
    } else {
        let source_info = SkImageInfo::MakeWithColorSpace(
            width,
            height,
            SkColorType::kRGBA_8888_SkColorType,
            SkAlphaType::kUnpremul_SkAlphaType,
            None,
        );
        let premul_info = SkImageInfo::MakeWithColorSpace(
            width,
            height,
            SkColorType::kRGBA_8888_SkColorType,
            SkAlphaType::kPremul_SkAlphaType,
            None,
        );
        let mut pixels = vec![0; byte_len];
        let converted = unsafe {
            SkConvertPixels(
                &premul_info,
                pixels.as_mut_ptr(),
                row_bytes,
                &source_info,
                resource.rgba8.as_ptr(),
                row_bytes,
            )
        }
        .map_err(|error| raster_error("translated image premultiplication failed", error))?;
        if !converted {
            return Err(io::Error::other(
                "translated image premultiplication rejected its layout",
            ));
        }
        pixels
    };
    let (pixels, width, height) = orient_image_pixels(
        pixels,
        resource.width as usize,
        resource.height as usize,
        item.image_orientation,
    )?;
    let width = i32::try_from(width)
        .map_err(|_| io::Error::other("translated oriented image width exceeds Skia limits"))?;
    let height = i32::try_from(height)
        .map_err(|_| io::Error::other("translated oriented image height exceeds Skia limits"))?;
    let row_bytes = width as usize * 4;
    let image_info = SkImageInfo::MakeWithColorSpace(
        width,
        height,
        SkColorType::kRGBA_8888_SkColorType,
        SkAlphaType::kPremul_SkAlphaType,
        None,
    );
    let pixmap = SkPixmap::new(&image_info, pixels.as_ptr(), row_bytes);
    unsafe { skia::src::image::SkImage_RasterFactories::RasterFromPixmapCopy(&pixmap) }
        .ok_or_else(|| io::Error::other("translated image allocation failed"))
        .map(Some)
}

/// Source: SkCanvas image-orientation receiver and
/// `LayoutngCanvasDrawImageRect`. Resource bytes stay in encoded storage order;
/// source rectangles address the returned oriented coordinate space.
fn orient_image_pixels(
    pixels: Vec<u8>,
    width: usize,
    height: usize,
    orientation: ImageOrientation,
) -> io::Result<(Vec<u8>, usize, usize)> {
    if orientation == ImageOrientation::kDefault {
        return Ok((pixels, width, height));
    }
    let (oriented_width, oriented_height) = if orientation.uses_width_as_height() {
        (height, width)
    } else {
        (width, height)
    };
    let byte_len = oriented_width
        .checked_mul(oriented_height)
        .and_then(|length| length.checked_mul(4))
        .ok_or_else(|| io::Error::other("translated oriented image byte count overflow"))?;
    if pixels.len() != byte_len {
        return Err(io::Error::other(
            "translated oriented image storage does not match its dimensions",
        ));
    }
    let mut oriented = vec![0; byte_len];
    for y in 0..height {
        for x in 0..width {
            let (oriented_x, oriented_y) = match orientation {
                ImageOrientation::kDefault => unreachable!(),
                ImageOrientation::kFlipHorizontal => (width - 1 - x, y),
                ImageOrientation::kRotate180 => (width - 1 - x, height - 1 - y),
                ImageOrientation::kFlipVertical => (x, height - 1 - y),
                ImageOrientation::kTranspose => (y, x),
                ImageOrientation::kRotate90 => (height - 1 - y, x),
                ImageOrientation::kTransverse => (height - 1 - y, width - 1 - x),
                ImageOrientation::kRotate270 => (y, width - 1 - x),
            };
            let source = (y * width + x) * 4;
            let destination = (oriented_y * oriented_width + oriented_x) * 4;
            oriented[destination..destination + 4].copy_from_slice(&pixels[source..source + 4]);
        }
    }
    Ok((oriented, oriented_width, oriented_height))
}

struct TranslatedCanvas<'target, 'pixels> {
    target: &'target mut RasterTarget<'pixels>,
    typeface_cache: Option<&'target TypefaceCache>,
    error: Option<io::Error>,
    depth: usize,
    mask_stack: Vec<Vec<MaskLayer>>,
    nearest: bool,
    // skia/ext/opacity_filter_canvas.cc:17-47. PictureDrawQuad opacity
    // filters each draw primitive; it must not flatten the display list into
    // one temporary layer because overlapping primitives then blend
    // differently.
    opacity: f32,
}

impl<'target, 'pixels> TranslatedCanvas<'target, 'pixels> {
    fn new(target: &'target mut RasterTarget<'pixels>, nearest: bool, opacity: f32) -> Self {
        Self {
            target,
            typeface_cache: None,
            error: None,
            depth: 0,
            mask_stack: Vec::new(),
            nearest,
            opacity,
        }
    }

    fn with_typeface_cache(mut self, cache: &'target TypefaceCache) -> Self {
        self.typeface_cache = Some(cache);
        self
    }

    fn unwind_to_depth(&mut self, depth: usize) {
        while self.depth > depth {
            self.depth -= 1;
            if let Err(error) = self.target.restore() {
                self.error
                    .get_or_insert(raster_error("translated cleanup restore failed", error));
                break;
            }
        }
    }

    fn fail(&mut self, message: impl Into<String>) {
        self.error
            .get_or_insert_with(|| io::Error::new(io::ErrorKind::Unsupported, message.into()));
    }

    /// Replay Paint's canonical filter value before the browser-neutral command
    /// projection. A flat PaintRecord (notably an SVG document image) has no
    /// compositor Effect traversal, but it still consumes the exact operations
    /// already built by PrePaint; Raster never becomes a second filter owner.
    fn execute_display_item(
        &mut self,
        item: &paint::paint_engine::DisplayItem,
        resources: &ResourceContext<'_>,
        alpha: f32,
    ) {
        let mut command = item.to_skia();
        command.flags_alpha = alpha;
        command.color.alpha *= alpha;
        if let Some(shader) = &mut command.paint_shader {
            shader.flags_alpha *= alpha;
            for stop in &mut shader.stops {
                stop.color.alpha *= alpha;
            }
        }
        if item.r#type != paint::paint_engine::DisplayItemType::kSaveLayerFilter
            || item.canonical_filters.is_empty()
        {
            self.execute(&command, resources);
            return;
        }
        if self.error.is_some() {
            return;
        }
        let result = (|| -> io::Result<()> {
            let filters = item.canonical_filters.canonical().map_err(|error| {
                io::Error::new(
                    io::ErrorKind::Unsupported,
                    format!("flat canonical filter: {error}"),
                )
            })?;
            let matrix = self.target.local_to_device();
            // SaveLayer restore paint starts opaque. DisplayItem::color is a
            // draw-op value and defaults transparent; applying it here would
            // discard the entire filtered layer.
            let mut paint = SkPaint::default();
            paint.setBlendMode(css_blend(command.blend_mode));
            paint.setAlphaf(command.opacity);
            paint.setImageFilter(BuildTranslatedFilterOperations(filters, &matrix)?);
            let bounds =
                (command.rect.width > 0. && command.rect.height > 0.).then(|| rect(command.rect));
            self.target
                .save_layer(bounds.as_ref(), Some(&paint))
                .map(|_| self.depth += 1)
                .map_err(|error| raster_error("translated reference saveLayer failed", error))
        })();
        if let Err(error) = result {
            self.error.get_or_insert(error);
        }
    }

    fn looped_shape(
        &mut self,
        outer: &compositor_core::adapter::geometry::RRectF,
        inner: Option<&compositor_core::adapter::geometry::RRectF>,
        flags: &CpuPaintFlags<SkRegion>,
        alpha: f32,
    ) {
        if flags.shader.is_some() || flags.image_filter.is_some() || flags.color_filter.is_some() {
            self.fail("translated looped shape has unsupported additional paint effects");
            return;
        }
        let paint = TranslatedLooperPaint(converted_paint_core(flags, alpha));
        let draw = |canvas: &mut Self, paint: TranslatedLooperPaint| {
            let result = unsafe {
                if let Some(inner) = inner {
                    canvas.target.draw_drrect(
                        &converted_rrect(outer),
                        &converted_rrect(inner),
                        &paint.0,
                    )
                } else {
                    canvas.target.draw_rrect(&converted_rrect(outer), &paint.0)
                }
            };
            if let Err(error) = result {
                canvas
                    .error
                    .get_or_insert(raster_error("translated looped shape draw failed", error));
            }
        };
        if let Some(looper) = &flags.looper {
            looper.Apply(self, &paint, draw);
        } else {
            draw(self, paint);
        }
    }

    fn draw_tiled_image(
        &mut self,
        item: &DrawCommand,
        resources: &ResourceContext<'_>,
        blend: SkBlendMode,
        antialias: bool,
    ) -> io::Result<()> {
        let tile_rect = if item.tile_rect.width > 0. && item.tile_rect.height > 0. {
            item.tile_rect
        } else {
            command::PaintRect {
                x: item.source_rect.x * item.tile_scale.x + item.tile_phase.x,
                y: item.source_rect.y * item.tile_scale.y + item.tile_phase.y,
                width: item.source_rect.width * item.tile_scale.x,
                height: item.source_rect.height * item.tile_scale.y,
            }
        };
        let step_x = tile_rect.width + item.tile_spacing.width;
        let step_y = tile_rect.height + item.tile_spacing.height;
        if step_x <= 0.
            || step_y <= 0.
            || item.rect.width <= 0.
            || item.rect.height <= 0.
            || item.source_rect.width <= 0.
            || item.source_rect.height <= 0.
        {
            return Ok(());
        }
        let Some(image) = raster_image(item, resources)? else {
            return Ok(());
        };
        let sampling = image_sampling(item.image_interpolation_quality);
        if std::env::var_os("BROWSER_PROFILE_INPUT").is_some() {
            let matrix = self.target.local_to_device();
            eprintln!(
                "raster-image-profile kind=tiled quality={:?} repeat_x={} repeat_y={} ctm=[{:.3},{:.3},{:.3},{:.3},{:.3},{:.3}] src=[{:.3},{:.3}] tile=[{:.3},{:.3}]",
                item.image_interpolation_quality,
                item.repeat_x,
                item.repeat_y,
                matrix.rc(0, 0),
                matrix.rc(0, 1),
                matrix.rc(0, 3),
                matrix.rc(1, 0),
                matrix.rc(1, 1),
                matrix.rc(1, 3),
                item.source_rect.width,
                item.source_rect.height,
                tile_rect.width,
                tile_rect.height,
            );
        }
        let mut paint = SkPaint::default();
        paint.setAlphaf(item.flags_alpha);
        paint.setAntiAlias(antialias);
        paint.setBlendMode(blend);
        // Source: blink Image::DrawPattern/CreatePatternShader. A complete
        // image with no spacing is retained by one image shader; Skia owns the
        // repeat phase and samples across tile boundaries in a single draw.
        // Besides matching Chromium's DPR-scaled sampling, this avoids one
        // drawImageRect per visible background tile.
        let full_image = item.source_rect.x == 0.
            && item.source_rect.y == 0.
            && item.source_rect.width == image.width() as f64
            && item.source_rect.height == image.height() as f64;
        if item.repeat_x
            && item.repeat_y
            && item.tile_spacing.width == 0.
            && item.tile_spacing.height == 0.
            && full_image
        {
            let mut local = SkMatrix::Translate(tile_rect.x as f32, tile_rect.y as f32);
            local.preScale(
                (tile_rect.width / item.source_rect.width) as f32,
                (tile_rect.height / item.source_rect.height) as f32,
            );
            let shader = image
                .makeShaderWithMatrix(SkTileMode::kRepeat, SkTileMode::kRepeat, &sampling, &local)
                .ok_or_else(|| io::Error::other("translated image pattern shader failed"))?;
            paint.setShader(Some(shader));
            unsafe { self.target.draw_rect(&rect(item.rect), &paint) }
                .map_err(|error| raster_error("translated image pattern draw failed", error))?;
            return Ok(());
        }
        let first_x = if item.repeat_x {
            tile_rect.x - ((tile_rect.x - item.rect.x) / step_x).ceil() * step_x
        } else {
            tile_rect.x
        };
        let first_y = if item.repeat_y {
            tile_rect.y - ((tile_rect.y - item.rect.y) / step_y).ceil() * step_y
        } else {
            tile_rect.y
        };

        self.target.save();
        self.target
            .clip_rect(&rect(item.rect), SkClipOp::kIntersect, false);
        let draw_result = (|| -> io::Result<()> {
            let mut y = first_y;
            let mut rows = 0;
            while y < item.rect.y + item.rect.height && rows < 10_000 {
                rows += 1;
                let mut x = first_x;
                let mut columns = 0;
                while x < item.rect.x + item.rect.width && columns < 10_000 {
                    columns += 1;
                    unsafe {
                        self.target.draw_image_rect(
                            &image,
                            Some(&rect(item.source_rect)),
                            &rect(command::PaintRect {
                                x,
                                y,
                                width: tile_rect.width,
                                height: tile_rect.height,
                            }),
                            &sampling,
                            Some(&paint),
                            SrcRectConstraint::kStrict_SrcRectConstraint,
                        )
                    }
                    .map_err(|error| raster_error("translated tiled image draw failed", error))?;
                    x += if item.repeat_x {
                        step_x
                    } else {
                        item.rect.width + step_x
                    };
                }
                y += if item.repeat_y {
                    step_y
                } else {
                    item.rect.height + step_y
                };
            }
            Ok(())
        })();
        let restore_result = self
            .target
            .restore()
            .map_err(|error| raster_error("translated tiled image restore failed", error));
        draw_result.and(restore_result)
    }

    fn replay_svg_pattern_commands(
        &mut self,
        commands: &[command::SvgPatternCommand],
        resources: &ResourceContext<'_>,
    ) -> io::Result<()> {
        use command::SvgPatternCommand;
        for pattern_command in commands {
            match pattern_command {
                SvgPatternCommand::SaveLayerOpacity(opacity) => {
                    let mut paint = SkPaint::default();
                    paint.setAlphaf(*opacity);
                    self.target
                        .save_layer(None, Some(&paint))
                        .map_err(|error| raster_error("SVG pattern opacity layer failed", error))?;
                    self.depth += 1;
                }
                SvgPatternCommand::SaveLayerBlend(mode) => {
                    let mut paint = SkPaint::default();
                    paint.setBlendMode(css_blend(*mode));
                    self.target
                        .save_layer(None, Some(&paint))
                        .map_err(|error| raster_error("SVG pattern blend layer failed", error))?;
                    self.depth += 1;
                }
                SvgPatternCommand::Restore => {
                    if self.depth == 0 {
                        return Err(io::Error::other("SVG pattern restore underflow"));
                    }
                    self.depth -= 1;
                    self.target
                        .restore()
                        .map_err(|error| raster_error("SVG pattern restore failed", error))?;
                }
                SvgPatternCommand::DrawShape(primitive)
                | SvgPatternCommand::DrawGlyph(primitive) => {
                    self.target.save();
                    self.depth += 1;
                    let transform = primitive.transform.values.map(|value| value as f32);
                    self.target.concat44(&SkM44::ColMajor(&transform));
                    let rect = command::PaintRect {
                        x: primitive.bounds_offset.x,
                        y: primitive.bounds_offset.y,
                        width: primitive.bounds_width,
                        height: primitive.bounds_height,
                    };
                    let shape_type = |stroke| match primitive.geometry {
                        command::SvgPatternGeometry::kPath => {
                            if stroke {
                                CommandKind::kStrokePath
                            } else {
                                CommandKind::kDrawPath
                            }
                        }
                        command::SvgPatternGeometry::kRectangle => {
                            if stroke {
                                CommandKind::kStrokeRect
                            } else {
                                CommandKind::kDrawRect
                            }
                        }
                        command::SvgPatternGeometry::kEllipse => {
                            if stroke {
                                CommandKind::kStrokeEllipse
                            } else {
                                CommandKind::kDrawEllipse
                            }
                        }
                    };
                    let common = |stroke: bool| DrawCommand {
                        r#type: shape_type(stroke),
                        rect,
                        path: primitive.path.clone(),
                        even_odd: primitive.fill_even_odd,
                        stroke_width: primitive.stroke_width,
                        dash_intervals: primitive.stroke_dash_array.clone(),
                        dash_offset: primitive.stroke_dash_offset,
                        svg_line_cap: primitive.stroke_line_cap,
                        svg_line_join: primitive.stroke_line_join,
                        miter_limit: primitive.stroke_miter_limit,
                        antialias: primitive.antialias,
                        non_scaling_stroke: primitive.non_scaling_stroke,
                        ..Default::default()
                    };
                    if primitive.fill_server.is_some()
                        || primitive.fill.is_some_and(|color| color.alpha > 0.)
                    {
                        let mut fill = common(false);
                        fill.color = primitive.fill.unwrap_or_default();
                        fill.paint_shader = primitive.fill_server.clone();
                        self.execute(&fill, resources);
                    }
                    if primitive.stroke_width > 0.
                        && (primitive.stroke_server.is_some()
                            || primitive.stroke.is_some_and(|color| color.alpha > 0.))
                    {
                        let mut stroke = common(true);
                        stroke.color = primitive.stroke.unwrap_or_default();
                        stroke.paint_shader = primitive.stroke_server.clone();
                        self.execute(&stroke, resources);
                    }
                    self.depth -= 1;
                    self.target
                        .restore()
                        .map_err(|error| raster_error("SVG pattern shape restore failed", error))?;
                    if self.error.is_some() {
                        return Err(self.error.take().unwrap());
                    }
                }
                SvgPatternCommand::DrawImage(image) => {
                    if !image.rgba8.is_empty() {
                        return Err(io::Error::new(
                            io::ErrorKind::Unsupported,
                            "inline SVG pattern image replay is not connected",
                        ));
                    }
                    self.target.save();
                    self.depth += 1;
                    self.target.concat44(&SkM44::ColMajor(
                        &image.transform.values.map(|value| value as f32),
                    ));
                    self.execute(
                        &DrawCommand {
                            r#type: CommandKind::kDrawImageRect,
                            rect: image.destination,
                            source_rect: image.source,
                            resource_id: image.resource_id,
                            image_interpolation_quality: image.image_interpolation_quality,
                            ..Default::default()
                        },
                        resources,
                    );
                    self.depth -= 1;
                    self.target
                        .restore()
                        .map_err(|error| raster_error("SVG pattern image restore failed", error))?;
                    if self.error.is_some() {
                        return Err(self.error.take().unwrap());
                    }
                }
                SvgPatternCommand::SaveLayerFilter(_)
                | SvgPatternCommand::SaveLayerReferenceFilter(_)
                | SvgPatternCommand::SaveLayerMask(_) => {
                    return Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "SVG pattern effect replay is not connected",
                    ));
                }
            }
        }
        if self.depth != 0 {
            return Err(io::Error::other("SVG pattern save/restore is unbalanced"));
        }
        Ok(())
    }

    /// Source: SkPictureShader::CachedImageInfo and
    /// core/paint/svg_object_painter.cc paint-server application.  Layout and
    /// Paint have already frozen the SVG pattern subtree into commands; Raster
    /// only materializes that immutable record into one repeating SkShader.
    fn svg_pattern_shader(
        &mut self,
        shader: &command::PaintShader,
        resources: &ResourceContext<'_>,
    ) -> io::Result<std::sync::Arc<dyn skia::src::shaders::SkShader::SkShader>> {
        let pattern = shader
            .pattern
            .as_ref()
            .ok_or_else(|| io::Error::other("SVG vector pattern has no frozen record"))?;
        if !shader.tile_width.is_finite()
            || !shader.tile_height.is_finite()
            || shader.tile_width <= 0.
            || shader.tile_height <= 0.
        {
            return Err(io::Error::other("SVG pattern tile is empty"));
        }
        let local = gradient_matrix(&shader.transform)?;
        let device = self.target.local_to_device();
        if device.rc(3, 0) != 0. || device.rc(3, 1) != 0. || device.rc(3, 3) != 1. {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "SVG pattern requires a finite 2D canvas matrix",
            ));
        }
        let device = SkMatrix::MakeAll(
            device.rc(0, 0),
            device.rc(0, 1),
            device.rc(0, 3),
            device.rc(1, 0),
            device.rc(1, 1),
            device.rc(1, 3),
            0.,
            0.,
            1.,
        );
        let bounds = SkRect::MakeWH(shader.tile_width as f32, shader.tile_height as f32);
        let total = SkMatrix::Concat(&device, &local);
        let tile = CachedImageInfo::Make(
            &bounds,
            &total,
            SkColorType::kRGBA_8888_SkColorType,
            None,
            0,
            &SkSurfaceProps::default(),
        );
        if !tile.success {
            return Ok(EmptyShader());
        }
        let width = tile.imageInfo.width();
        let height = tile.imageInfo.height();
        let row_bytes = (width as usize)
            .checked_mul(4)
            .ok_or_else(|| io::Error::other("SVG pattern row size overflow"))?;
        let length = row_bytes
            .checked_mul(height as usize)
            .ok_or_else(|| io::Error::other("SVG pattern allocation overflow"))?;
        let mut pixels = vec![0; length];
        {
            let mut target = RasterTarget::new(
                &mut pixels,
                width,
                height,
                row_bytes,
                PixelFormat::Rgba8888,
                None,
            )
            .map_err(|error| raster_error("SVG pattern target failed", error))?;
            target.set_matrix(&tile.matrixForDraw);
            target.concat44(&SkM44::ColMajor(
                &pattern.content_transform.values.map(|value| value as f32),
            ));
            {
                let mut replay = TranslatedCanvas::new(&mut target, false, 1.);
                replay.replay_svg_pattern_commands(&pattern.commands, resources)?;
                replay.finish_replay()?;
            }
            target
                .finish()
                .map_err(|error| raster_error("SVG pattern finish failed", error))?;
        }
        let info = SkImageInfo::MakeWithColorSpace(
            width,
            height,
            SkColorType::kRGBA_8888_SkColorType,
            SkAlphaType::kPremul_SkAlphaType,
            None,
        );
        let pixmap = SkPixmap::new(&info, pixels.as_ptr(), row_bytes);
        let image =
            unsafe { skia::src::image::SkImage_RasterFactories::RasterFromPixmapCopy(&pixmap) }
                .ok_or_else(|| io::Error::other("SVG pattern image allocation failed"))?;
        let image_to_record = tile
            .matrixForDraw
            .invert()
            .ok_or_else(|| io::Error::other("SVG pattern tile matrix is singular"))?;
        let shader_matrix = SkMatrix::Concat(&local, &image_to_record);
        image
            .makeShaderWithMatrix(
                SkTileMode::kRepeat,
                SkTileMode::kRepeat,
                &SkSamplingOptions::with_filter(SkFilterMode::kLinear),
                &shader_matrix,
            )
            .ok_or_else(|| io::Error::other("SVG pattern shader creation failed"))
    }

    fn command_paint_with_shader(
        &mut self,
        item: &DrawCommand,
        stroke: bool,
        resources: &ResourceContext<'_>,
    ) -> io::Result<SkPaint> {
        let Some(shader) = item.paint_shader.as_ref() else {
            return Ok(command_paint(item, stroke));
        };
        if shader.kind != command::PaintShaderKind::kPattern {
            return command_paint_with_gradient_shader(item, stroke);
        }
        let mut paint = command_paint(item, stroke);
        paint.setShader(Some(self.svg_pattern_shader(shader, resources)?));
        paint.setColor4f(
            &SkColor4f {
                fR: 1.,
                fG: 1.,
                fB: 1.,
                fA: shader.flags_alpha
                    * shader
                        .pattern
                        .as_ref()
                        .map_or(1., |pattern| pattern.opacity),
            },
            None,
        );
        Ok(paint)
    }

    fn draw_record_shader_rect(
        &mut self,
        destination: &compositor_core::adapter::geometry::RectF,
        flags: &CpuPaintFlags<SkRegion>,
        folded_alpha: f32,
        record: &CpuPaintRecord<SkRegion>,
        tile_rect: &compositor_core::adapter::geometry::RectF,
        tile_mode_x: paint::cc::paint::tile_mode::TileMode,
        tile_mode_y: paint::cc::paint::tile_mode::TileMode,
        local_matrix: Option<&compositor_core::runtime::geometry::Transform<SkRegion>>,
        params: &mut RasterPlaybackParams<'_, SkRegion>,
    ) -> io::Result<()> {
        if flags.image_filter.is_some() || flags.color_filter.is_some() || flags.looper.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "translated record shader with filter or looper is not connected",
            ));
        }
        let mut local = SkMatrix::I().clone();
        if let Some(matrix) = local_matrix {
            if matrix.rc(3, 0) != 0. || matrix.rc(3, 1) != 0. || matrix.rc(3, 3) != 1. {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "translated record shader requires a finite 2D local matrix",
                ));
            }
            local = SkMatrix::MakeAll(
                matrix.rc(0, 0) as f32,
                matrix.rc(0, 1) as f32,
                matrix.rc(0, 3) as f32,
                matrix.rc(1, 0) as f32,
                matrix.rc(1, 1) as f32,
                matrix.rc(1, 3) as f32,
                0.,
                0.,
                1.,
            );
        }
        let device = self.target.local_to_device();
        if device.rc(3, 0) != 0. || device.rc(3, 1) != 0. || device.rc(3, 3) != 1. {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "translated record shader requires a finite 2D canvas matrix",
            ));
        }
        let device = SkMatrix::MakeAll(
            device.rc(0, 0),
            device.rc(0, 1),
            device.rc(0, 3),
            device.rc(1, 0),
            device.rc(1, 1),
            device.rc(1, 3),
            0.,
            0.,
            1.,
        );
        let bounds = SkRect::MakeLTRB(
            tile_rect.x(),
            tile_rect.y(),
            tile_rect.right(),
            tile_rect.bottom(),
        );
        let total = SkMatrix::Concat(&device, &local);
        let tile = CachedImageInfo::Make(
            &bounds,
            &total,
            SkColorType::kRGBA_8888_SkColorType,
            None,
            0,
            &SkSurfaceProps::default(),
        );
        if !tile.success {
            return Ok(());
        }
        let width = tile.imageInfo.width();
        let height = tile.imageInfo.height();
        let row_bytes = (width as usize)
            .checked_mul(4)
            .ok_or_else(|| io::Error::other("translated record shader row size overflow"))?;
        let length = row_bytes
            .checked_mul(height as usize)
            .ok_or_else(|| io::Error::other("translated record shader allocation overflow"))?;
        let mut pixels = Vec::new();
        pixels.try_reserve_exact(length).map_err(io::Error::other)?;
        pixels.resize(length, 0);
        {
            let mut target = RasterTarget::new(
                &mut pixels,
                width,
                height,
                row_bytes,
                PixelFormat::Rgba8888,
                None,
            )
            .map_err(|error| raster_error("translated record shader target failed", error))?;
            target.set_matrix(&tile.matrixForDraw);
            {
                let mut replay = TranslatedCanvas::new(&mut target, false, 1.);
                replay.record(record, params);
                replay.finish_replay()?;
            }
            target
                .finish()
                .map_err(|error| raster_error("translated record shader finish failed", error))?;
        }
        let info = SkImageInfo::MakeWithColorSpace(
            width,
            height,
            SkColorType::kRGBA_8888_SkColorType,
            SkAlphaType::kPremul_SkAlphaType,
            None,
        );
        let pixmap = SkPixmap::new(&info, pixels.as_ptr(), row_bytes);
        let image =
            unsafe { skia::src::image::SkImage_RasterFactories::RasterFromPixmapCopy(&pixmap) }
                .ok_or_else(|| {
                    io::Error::other("translated record shader image allocation failed")
                })?;
        let image_to_record = tile
            .matrixForDraw
            .invert()
            .ok_or_else(|| io::Error::other("translated record shader tile matrix is singular"))?;
        let shader_matrix = SkMatrix::Concat(&local, &image_to_record);
        let shader = image
            .makeShaderWithMatrix(
                translated_tile_mode(tile_mode_x),
                translated_tile_mode(tile_mode_y),
                &SkSamplingOptions::with_filter(SkFilterMode::kLinear),
                &shader_matrix,
            )
            .ok_or_else(|| io::Error::other("translated record image shader creation failed"))?;
        let mut paint = converted_paint_core(flags, folded_alpha);
        paint.setShader(Some(shader));
        unsafe {
            self.target.draw_rect(
                &SkRect::MakeLTRB(
                    destination.x(),
                    destination.y(),
                    destination.right(),
                    destination.bottom(),
                ),
                &paint,
            )
        }
        .map_err(|error| raster_error("translated record shader draw failed", error))
    }

    fn draw_gradient_tile(
        &mut self,
        item: &DrawCommand,
        destination: command::PaintRect,
        tile_rect: command::PaintRect,
    ) -> io::Result<()> {
        let Some(shader) = item.paint_shader.as_ref() else {
            return Err(io::Error::other(
                "translated gradient command has no paint shader",
            ));
        };
        if item.blend_mode != command::PaintBlendMode::kNormal {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "translated tiled gradient requires normal blending",
            ));
        }

        // Source: skia/compat/canvas.rs paint_for_shader and
        // Canvas::draw_gradient_tile_impl. Gradient geometry is authored in
        // the original tile's coordinate space. Rebase it to one local tile,
        // then place that tile at its resolved destination through the shader
        // matrix. This preserves repeating-gradient phase and tile spacing.
        let mut shader = shader.clone();
        let offset = |point: &mut command::Offset| {
            point.x -= tile_rect.x;
            point.y -= tile_rect.y;
        };
        offset(&mut shader.start);
        offset(&mut shader.end);
        offset(&mut shader.center);
        offset(&mut shader.center_offset);
        offset(&mut shader.focal);
        offset(&mut shader.tile_offset);
        shader.transform.values[12] += destination.x;
        shader.transform.values[13] += destination.y;

        let mut paint = command_paint(item, false);
        paint.setShader(Some(gradient_shader(&shader)?));
        // Source: third_party/blink/renderer/platform/graphics/gradient.cc
        // Gradient::ApplyToFlags. Without this flag, 8-bit output quantizes a
        // long CSS gradient into visible full-width color bands.
        paint.setDither(true);
        unsafe {
            self.target
                .draw_rect(&rect(destination), &paint)
                .map_err(|error| raster_error("translated gradient draw failed", error))
        }
    }

    fn draw_gradient(&mut self, item: &DrawCommand) -> io::Result<()> {
        if item.rect.width <= 0. || item.rect.height <= 0. {
            return Ok(());
        }
        let tile_rect = if item.tile_rect.width > 0. && item.tile_rect.height > 0. {
            item.tile_rect
        } else {
            item.rect
        };
        let same = |left: f64, right: f64| (left - right).abs() <= 1e-6;
        let single_tile = same(item.rect.x, tile_rect.x)
            && same(item.rect.y, tile_rect.y)
            && same(item.rect.width, tile_rect.width)
            && same(item.rect.height, tile_rect.height);
        if item.r#type != CommandKind::kDrawTiledGradient || single_tile {
            return self.draw_gradient_tile(item, item.rect, tile_rect);
        }

        let step_x = tile_rect.width + item.tile_spacing.width;
        let step_y = tile_rect.height + item.tile_spacing.height;
        if step_x <= 0. || step_y <= 0. {
            return Ok(());
        }
        let first_x = if item.repeat_x {
            tile_rect.x - ((tile_rect.x - item.rect.x) / step_x).ceil() * step_x
        } else {
            tile_rect.x
        };
        let first_y = if item.repeat_y {
            tile_rect.y - ((tile_rect.y - item.rect.y) / step_y).ceil() * step_y
        } else {
            tile_rect.y
        };

        self.target.save();
        self.target
            .clip_rect(&rect(item.rect), SkClipOp::kIntersect, false);
        let draw_result = (|| -> io::Result<()> {
            let mut y = first_y;
            let mut rows = 0;
            while y < item.rect.y + item.rect.height && rows < 10_000 {
                rows += 1;
                let mut x = first_x;
                let mut columns = 0;
                while x < item.rect.x + item.rect.width && columns < 10_000 {
                    columns += 1;
                    self.draw_gradient_tile(
                        item,
                        command::PaintRect {
                            x,
                            y,
                            width: tile_rect.width,
                            height: tile_rect.height,
                        },
                        tile_rect,
                    )?;
                    x += if item.repeat_x {
                        step_x
                    } else {
                        item.rect.width + step_x
                    };
                }
                y += if item.repeat_y {
                    step_y
                } else {
                    item.rect.height + step_y
                };
            }
            Ok(())
        })();
        let restore_result = self
            .target
            .restore()
            .map_err(|error| raster_error("translated gradient restore failed", error));
        draw_result.and(restore_result)
    }

    /// Source: `LayoutngCanvasDrawBoxShadow` / Blink box-shadow playback.
    /// The outer form clips the unshifted border box and draws the expanded
    /// caster after the shadow offset. The inset form draws the inverse caster
    /// around a shifted hole and clips the result to the border box.
    fn draw_box_shadow(&mut self, item: &DrawCommand) -> io::Result<()> {
        let mut paint = command_paint(item, false);
        paint.setStyle(Style::kFill_Style);

        if !item.inset {
            let shadow = command::PaintRect {
                x: item.rect.x - item.spread,
                y: item.rect.y - item.spread,
                width: item.rect.width + item.spread * 2.,
                height: item.rect.height + item.spread * 2.,
            };
            if shadow.width <= 0. || shadow.height <= 0. {
                return Ok(());
            }
            let mut hole = item.rect;
            let mut hole_radii = item.inner_corner_radii;
            if item.shadow_has_opaque_background {
                hole.x += 1.;
                hole.y += 1.;
                hole.width = (hole.width - 2.).max(0.);
                hole.height = (hole.height - 2.).max(0.);
                for radius in [
                    &mut hole_radii.top_left,
                    &mut hole_radii.top_right,
                    &mut hole_radii.bottom_right,
                    &mut hole_radii.bottom_left,
                ] {
                    radius.x = (radius.x - 1.).max(0.);
                    radius.y = (radius.y - 1.).max(0.);
                }
            }
            self.target.save();
            let draw = (|| -> io::Result<()> {
                if hole.width > 0. && hole.height > 0. {
                    self.target
                        .clip_rrect(&rrect(hole, hole_radii), SkClipOp::kDifference, true);
                }
                self.target
                    .translate(item.shadow_offset.x as f32, item.shadow_offset.y as f32);
                unsafe {
                    self.target
                        .draw_rrect(&rrect(shadow, item.corner_radii), &paint)
                }
                .map_err(|error| raster_error("translated outer box shadow failed", error))
            })();
            let restore = self
                .target
                .restore()
                .map_err(|error| raster_error("translated outer shadow restore failed", error));
            return draw.and(restore);
        }

        if item.rect.width <= 0. || item.rect.height <= 0. {
            return Ok(());
        }
        let hole = command::PaintRect {
            x: item.rect.x + item.spread,
            y: item.rect.y + item.spread,
            width: (item.rect.width - item.spread * 2.).max(0.),
            height: (item.rect.height - item.spread * 2.).max(0.),
        };
        let mut hole_radii = item.corner_radii;
        for radius in [
            &mut hole_radii.top_left,
            &mut hole_radii.top_right,
            &mut hole_radii.bottom_right,
            &mut hole_radii.bottom_left,
        ] {
            radius.x = (radius.x - item.spread).max(0.);
            radius.y = (radius.y - item.spread).max(0.);
        }
        let extent = (1.5 * item.blur_radius).ceil() + (-item.spread).max(0.) + 2.;
        let left = item.rect.x.min(item.rect.x - item.shadow_offset.x) - extent;
        let top = item.rect.y.min(item.rect.y - item.shadow_offset.y) - extent;
        let right = (item.rect.x + item.rect.width)
            .max(item.rect.x + item.rect.width - item.shadow_offset.x)
            + extent;
        let bottom = (item.rect.y + item.rect.height)
            .max(item.rect.y + item.rect.height - item.shadow_offset.y)
            + extent;
        let mut caster = SkPathBuilder::with_fill_type(SkPathFillType::kEvenOdd);
        caster.addRectDefault(
            &SkRect::MakeLTRB(left as f32, top as f32, right as f32, bottom as f32),
            SkPathDirection::kCW,
        );
        if hole.width > 0. && hole.height > 0. {
            caster.addRRectDefault(&rrect(hole, hole_radii), SkPathDirection::kCW);
        }
        let caster = caster.detach(None);

        self.target.save();
        let draw = (|| -> io::Result<()> {
            self.target.clip_rrect(
                &rrect(item.rect, item.corner_radii),
                SkClipOp::kIntersect,
                item.antialias,
            );
            self.target
                .translate(item.shadow_offset.x as f32, item.shadow_offset.y as f32);
            unsafe { self.target.draw_path(&caster, &paint) }
                .map_err(|error| raster_error("translated inset box shadow failed", error))
        })();
        let restore = self
            .target
            .restore()
            .map_err(|error| raster_error("translated inset shadow restore failed", error));
        draw.and(restore)
    }

    fn draw_mask_layers(
        &mut self,
        layers: &[MaskLayer],
        resources: &ResourceContext<'_>,
    ) -> io::Result<()> {
        // Source: BoxPainterBase mask-mode/mask-composite playback. CSS lists
        // are front-to-back, so restore isolated layers from bottom to top.
        for (reverse_index, layer) in layers.iter().rev().enumerate() {
            // Keep the source list position: if the declared bottom source is
            // absent, the next layer still applies its authored composite.
            if layer.resource_id == 0 && layer.paint_shader.is_none() {
                continue;
            }
            let mut restore_paint = SkPaint::default();
            restore_paint.setBlendMode(if reverse_index == 0 {
                SkBlendMode::kSrcOver
            } else {
                match layer.composite {
                    PaintMaskComposite::kAdd => SkBlendMode::kSrcOver,
                    PaintMaskComposite::kSubtract => SkBlendMode::kSrcOut,
                    PaintMaskComposite::kIntersect => SkBlendMode::kSrcIn,
                    PaintMaskComposite::kExclude => SkBlendMode::kXor,
                }
            });
            if layer.mode == PaintMaskMode::kLuminance {
                restore_paint.setColorFilter(
                    skia::src::effects::colorfilters::SkRuntimeColorFilter::MakeLumaColorFilter(),
                );
            }
            self.target
                .save_layer(None, Some(&restore_paint))
                .map_err(|error| raster_error("translated mask source layer failed", error))?;
            self.target.save();
            let draw = (|| -> io::Result<()> {
                if layer.has_clip {
                    if layer.clip_radii.HasRadius() {
                        self.target.clip_rrect(
                            &rrect(layer.clip_rect, layer.clip_radii),
                            SkClipOp::kIntersect,
                            true,
                        );
                    } else {
                        self.target
                            .clip_rect(&rect(layer.clip_rect), SkClipOp::kIntersect, false);
                    }
                }
                if let Some(shader) = &layer.paint_shader {
                    self.draw_gradient(&DrawCommand {
                        r#type: CommandKind::kDrawTiledGradient,
                        rect: layer.clip_rect,
                        tile_rect: layer.tile_rect,
                        repeat_x: layer.repeat_x,
                        repeat_y: layer.repeat_y,
                        tile_spacing: layer.tile_spacing,
                        paint_shader: Some(shader.clone()),
                        ..Default::default()
                    })
                } else {
                    self.draw_tiled_image(
                        &DrawCommand {
                            r#type: CommandKind::kDrawTiledImage,
                            rect: layer.clip_rect,
                            source_rect: layer.source_rect,
                            tile_rect: layer.tile_rect,
                            resource_id: layer.resource_id,
                            image_interpolation_quality: layer.image_interpolation_quality,
                            image_orientation: layer.image_orientation,
                            repeat_x: layer.repeat_x,
                            repeat_y: layer.repeat_y,
                            tile_scale: layer.tile_scale,
                            tile_spacing: layer.tile_spacing,
                            ..Default::default()
                        },
                        resources,
                        SkBlendMode::kSrcOver,
                        false,
                    )
                }
            })();
            let restore_clip = self
                .target
                .restore()
                .map_err(|error| raster_error("translated mask clip restore failed", error));
            let restore_layer = self
                .target
                .restore()
                .map_err(|error| raster_error("translated mask source restore failed", error));
            draw.and(restore_clip).and(restore_layer)?;
        }
        Ok(())
    }

    fn begin_mask(&mut self, item: &DrawCommand) -> io::Result<()> {
        self.target
            .save_layer(None, None)
            .map_err(|error| raster_error("translated mask content layer failed", error))?;
        self.mask_stack.push(item.mask_layers.clone());
        self.depth += 1;
        Ok(())
    }

    fn end_mask(&mut self, resources: &ResourceContext<'_>) -> io::Result<()> {
        let Some(layers) = self.mask_stack.pop() else {
            return Ok(());
        };
        if self.depth == 0 {
            return Err(io::Error::other(
                "translated mask content restore underflow",
            ));
        }
        let mut dst_in = SkPaint::default();
        dst_in.setBlendMode(SkBlendMode::kDstIn);
        self.target
            .save_layer(None, Some(&dst_in))
            .map_err(|error| raster_error("translated mask composition layer failed", error))?;
        self.depth += 1;
        let draw = self.draw_mask_layers(&layers, resources);
        self.depth -= 1;
        let restore_mask = self
            .target
            .restore()
            .map_err(|error| raster_error("translated mask composition restore failed", error));
        self.depth -= 1;
        let restore_content = self
            .target
            .restore()
            .map_err(|error| raster_error("translated masked content restore failed", error));
        draw.and(restore_mask).and(restore_content)
    }

    fn execute(&mut self, item: &DrawCommand, resources: &ResourceContext<'_>) {
        if self.error.is_some() {
            return;
        }
        // Source: skia/ext/opacity_filter_canvas.cc:17-47. Picture replay
        // leaves matrix, clip and save/restore operations unchanged and
        // filters only the paint of each draw. Nearest-neighbor similarly
        // overrides only direct image draws.
        let is_draw = !matches!(
            item.r#type,
            CommandKind::kSave
                | CommandKind::kRestore
                | CommandKind::kConcat
                | CommandKind::kClipRect
                | CommandKind::kClipRoundedRect
                | CommandKind::kClipOutRoundedRect
                | CommandKind::kClipPath
                | CommandKind::kClipOutRect
                | CommandKind::kSaveLayer
                | CommandKind::kSaveLayerAlpha
                | CommandKind::kSaveLayerBlend
                | CommandKind::kSaveLayerFilter
                | CommandKind::kSaveLayerDstIn
                | CommandKind::kBeginMask
                | CommandKind::kEndMask
        );
        let filtered = (is_draw && (self.opacity < 1. || self.nearest)).then(|| {
            let mut filtered = item.clone();
            if self.opacity < 1. {
                if matches!(
                    filtered.r#type,
                    CommandKind::kDrawImageRect | CommandKind::kDrawTiledImage
                ) {
                    filtered.flags_alpha *= self.opacity;
                } else {
                    filtered.color.alpha *= self.opacity;
                    if let Some(shader) = &mut filtered.paint_shader {
                        match shader.kind {
                            command::PaintShaderKind::kPattern => {
                                shader.flags_alpha *= self.opacity
                            }
                            _ => {
                                for stop in &mut shader.stops {
                                    stop.color.alpha *= self.opacity;
                                }
                            }
                        }
                    }
                }
            }
            if self.nearest
                && matches!(
                    filtered.r#type,
                    CommandKind::kDrawImageRect | CommandKind::kDrawTiledImage
                )
            {
                filtered.image_interpolation_quality = ImageInterpolationQuality::kNone;
            }
            filtered
        });
        let item = filtered.as_ref().unwrap_or(item);
        let result = match item.r#type {
            CommandKind::kSave => {
                self.target.save();
                self.depth += 1;
                Ok(())
            }
            CommandKind::kRestore => {
                if self.depth == 0 {
                    Err(io::Error::other("translated Paint restore underflow"))
                } else {
                    self.depth -= 1;
                    self.target
                        .restore()
                        .map_err(|error| raster_error("translated restore failed", error))
                }
            }
            CommandKind::kConcat => {
                let matrix = item.transform.values.map(|value| value as f32);
                self.target.concat44(&SkM44::ColMajor(&matrix));
                Ok(())
            }
            CommandKind::kClipRect | CommandKind::kClipOutRect => {
                self.target.clip_rect(
                    &rect(item.rect),
                    if item.r#type == CommandKind::kClipOutRect {
                        SkClipOp::kDifference
                    } else {
                        SkClipOp::kIntersect
                    },
                    item.antialias,
                );
                Ok(())
            }
            CommandKind::kClipRoundedRect | CommandKind::kClipOutRoundedRect => {
                self.target.clip_rrect(
                    &rrect(item.rect, item.corner_radii),
                    if item.r#type == CommandKind::kClipOutRoundedRect {
                        SkClipOp::kDifference
                    } else {
                        SkClipOp::kIntersect
                    },
                    item.antialias,
                );
                Ok(())
            }
            CommandKind::kClipPath => {
                self.target.clip_path(
                    &path(&item.path, item.even_odd, item.inverse_winding),
                    SkClipOp::kIntersect,
                    item.antialias,
                );
                Ok(())
            }
            CommandKind::kBeginMask => self.begin_mask(item),
            CommandKind::kEndMask => self.end_mask(resources),
            CommandKind::kDrawMask => self.draw_mask_layers(&item.mask_layers, resources),
            CommandKind::kSaveLayerFilter => (|| -> io::Result<()> {
                let matrix = self.target.local_to_device();
                let mut paint = SkPaint::default();
                paint.setBlendMode(css_blend(item.blend_mode));
                paint.setAlphaf(item.opacity);
                paint.setImageFilter(translated_command_filters(&item.filters, &matrix)?);
                let bounds =
                    (item.rect.width > 0. && item.rect.height > 0.).then(|| rect(item.rect));
                self.target
                    .save_layer(bounds.as_ref(), Some(&paint))
                    .map(|_| {
                        self.depth += 1;
                    })
                    .map_err(|error| raster_error("translated filtered saveLayer failed", error))
            })(),
            CommandKind::kSaveLayer
            | CommandKind::kSaveLayerAlpha
            | CommandKind::kSaveLayerBlend
            | CommandKind::kSaveLayerDstIn => {
                let mut paint = command_paint(item, false);
                paint.setAlphaf(item.opacity);
                if item.r#type == CommandKind::kSaveLayerDstIn {
                    // BoxFragmentPainter records mask content into an isolated
                    // layer whose restore paint intersects the existing
                    // destination. This is the SkCanvas equivalent of
                    // the legacy receiver's SavedFrame::dst_in flag.
                    paint.setBlendMode(SkBlendMode::kDstIn);
                }
                let bounds =
                    (item.rect.width > 0. && item.rect.height > 0.).then(|| rect(item.rect));
                self.target
                    .save_layer(bounds.as_ref(), Some(&paint))
                    .map(|_| {
                        self.depth += 1;
                    })
                    .map_err(|error| raster_error("translated saveLayer failed", error))
            }
            CommandKind::kDrawRect | CommandKind::kStrokeRect => (|| -> io::Result<()> {
                let paint = self.command_paint_with_shader(
                    item,
                    item.r#type == CommandKind::kStrokeRect,
                    resources,
                )?;
                unsafe { self.target.draw_rect(&rect(item.rect), &paint) }
                    .map_err(|error| raster_error("translated drawRect failed", error))
            })(),
            CommandKind::kDrawRoundedRect => unsafe {
                self.target
                    .draw_rrect(
                        &rrect(item.rect, item.corner_radii),
                        &command_paint(item, false),
                    )
                    .map_err(|error| raster_error("translated drawRRect failed", error))
            },
            CommandKind::kDrawDoubleRoundedRect => unsafe {
                self.target
                    .draw_drrect(
                        &rrect(item.rect, item.corner_radii),
                        &rrect(item.inner_rect, item.inner_corner_radii),
                        &command_paint(item, false),
                    )
                    .map_err(|error| raster_error("translated drawDRRect failed", error))
            },
            CommandKind::kDrawEllipse | CommandKind::kStrokeEllipse => (|| -> io::Result<()> {
                let paint = self.command_paint_with_shader(
                    item,
                    item.r#type == CommandKind::kStrokeEllipse,
                    resources,
                )?;
                unsafe { self.target.draw_oval(&rect(item.rect), &paint) }
                    .map_err(|error| raster_error("translated drawOval failed", error))
            })(),
            CommandKind::kDrawPath | CommandKind::kStrokePath => (|| -> io::Result<()> {
                let paint = self.command_paint_with_shader(
                    item,
                    item.r#type == CommandKind::kStrokePath,
                    resources,
                )?;
                unsafe {
                    self.target.draw_path(
                        &path(&item.path, item.even_odd, item.inverse_winding),
                        &paint,
                    )
                }
                .map_err(|error| raster_error("translated drawPath failed", error))
            })(),
            CommandKind::kDrawGradientRect | CommandKind::kDrawTiledGradient => {
                self.draw_gradient(item)
            }
            CommandKind::kDrawImageRect => (|| -> io::Result<()> {
                if item.source_rect.width <= 0.
                    || item.source_rect.height <= 0.
                    || item.rect.width <= 0.
                    || item.rect.height <= 0.
                {
                    return Ok(());
                }
                if let Some(image) = raster_image(item, resources)? {
                    let sampling = image_sampling(item.image_interpolation_quality);
                    if std::env::var_os("BROWSER_PROFILE_INPUT").is_some() {
                        let matrix = self.target.local_to_device();
                        eprintln!(
                            "raster-image-profile kind=rect quality={:?} ctm=[{:.3},{:.3},{:.3},{:.3},{:.3},{:.3}] src=[{:.3},{:.3}] dst=[{:.3},{:.3}]",
                            item.image_interpolation_quality,
                            matrix.rc(0, 0),
                            matrix.rc(0, 1),
                            matrix.rc(0, 3),
                            matrix.rc(1, 0),
                            matrix.rc(1, 1),
                            matrix.rc(1, 3),
                            item.source_rect.width,
                            item.source_rect.height,
                            item.rect.width,
                            item.rect.height,
                        );
                    }
                    let mut paint = SkPaint::default();
                    paint.setAlphaf(item.flags_alpha);
                    paint.setAntiAlias(item.antialias);
                    paint.setBlendMode(css_blend(item.blend_mode));
                    unsafe {
                        self.target.draw_image_rect(
                            &image,
                            Some(&rect(item.source_rect)),
                            &rect(item.rect),
                            &sampling,
                            Some(&paint),
                            SrcRectConstraint::kStrict_SrcRectConstraint,
                        )
                    }
                    .map_err(|error| raster_error("translated drawImageRect failed", error))?;
                }
                Ok(())
            })(),
            CommandKind::kDrawTiledImage => {
                self.draw_tiled_image(item, resources, css_blend(item.blend_mode), item.antialias)
            }
            CommandKind::kDrawBoxShadow => self.draw_box_shadow(item),
            CommandKind::kDrawGlyphRun => (|| -> io::Result<()> {
                if item.glyphs.is_empty() {
                    return Ok(());
                }
                if item.transform != command::TransformMatrix::default()
                    || item.glyphs.iter().any(|glyph| glyph.canvas_rotation != 0)
                {
                    return Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "translated transformed glyph run replay is not connected",
                    ));
                }
                let typeface = raster_typeface(item, resources, self.typeface_cache)?;
                let mut font = SkFont::with_size(Some(typeface), item.font_size as f32);
                font.setEmbolden(item.synthetic_bold);
                font.setSkewX(if item.synthetic_italic { -0.25 } else { 0.0 });
                #[cfg(target_vendor = "apple")]
                ConfigureAppleFont(&mut font, item.font_smoothing);
                #[cfg(not(target_vendor = "apple"))]
                {
                    font.setSubpixel(item.font_smoothing == FontSmoothing::kSubpixelAntialiased);
                    font.setEdging(match item.font_smoothing {
                        FontSmoothing::kNone => Edging::kAlias,
                        FontSmoothing::kSubpixelAntialiased => Edging::kSubpixelAntiAlias,
                        FontSmoothing::kAuto | FontSmoothing::kAntialiased => Edging::kAntiAlias,
                    });
                }
                let glyphs = item
                    .glyphs
                    .iter()
                    .map(|glyph| {
                        u16::try_from(glyph.id)
                            .map_err(|_| io::Error::other("translated glyph id exceeds u16"))
                    })
                    .collect::<io::Result<Vec<_>>>()?;
                let positions = item
                    .glyphs
                    .iter()
                    .map(|glyph| SkPoint {
                        fX: glyph.offset.x as f32,
                        fY: glyph.offset.y as f32,
                    })
                    .collect::<Vec<_>>();
                let mut paint = command_paint(item, item.stroke_glyphs);
                if item.stroke_glyphs {
                    paint.setStrokeWidth((item.font_size as f32 * 0.03).max(0.5));
                }
                unsafe {
                    self.target.draw_glyphs(
                        &glyphs,
                        &positions,
                        SkPoint {
                            fX: item.text_blob_origin.x as f32,
                            fY: item.text_blob_origin.y as f32,
                        },
                        &font,
                        &paint,
                    )
                }
                .map_err(|error| raster_error("translated drawGlyphs failed", error))
            })(),
            CommandKind::kStrokeLine => unsafe {
                self.target
                    .draw_line(
                        SkPoint {
                            fX: item.rect.x as f32,
                            fY: item.rect.y as f32,
                        },
                        SkPoint {
                            fX: (item.rect.x + item.rect.width) as f32,
                            fY: (item.rect.y + item.rect.height) as f32,
                        },
                        &command_paint(item, true),
                    )
                    .map_err(|error| raster_error("translated drawLine failed", error))
            },
            CommandKind::kDrawScrollbarTrack
            | CommandKind::kDrawScrollbarThumb
            | CommandKind::kDrawScrollbarButton
            | CommandKind::kDrawScrollbarCorner => unsafe {
                self.target
                    .draw_rect(&rect(item.rect), &command_paint(item, false))
                    .map_err(|error| raster_error("translated scrollbar draw failed", error))
            },
            other => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("translated Skia replay for {other:?} is not connected"),
            )),
        };
        if let Err(error) = result {
            self.error.get_or_insert(error);
        }
    }

    fn simple(&mut self, kind: CommandKind) {
        self.execute(
            &DrawCommand {
                r#type: kind,
                ..Default::default()
            },
            &ResourceContext::default(),
        );
    }

    fn record(
        &mut self,
        record: &CpuPaintRecord<SkRegion>,
        params: &mut RasterPlaybackParams<'_, SkRegion>,
    ) {
        if let Some(buffer) = record.buffer() {
            self.Playback(buffer, params, true, None);
        }
    }

    fn finish_replay(self) -> io::Result<()> {
        if let Some(error) = self.error {
            return Err(error);
        }
        Ok(())
    }
}

impl DisplayCanvas for TranslatedCanvas<'_, '_> {
    fn LocalClipBounds(&self) -> Option<Rect> {
        let value = self.target.local_clip_bounds();
        (!value.isEmpty()).then(|| {
            let left = value.fLeft.floor() as i32;
            let top = value.fTop.floor() as i32;
            let right = value.fRight.ceil() as i32;
            let bottom = value.fBottom.ceil() as i32;
            Rect::new(
                left,
                top,
                right.saturating_sub(left),
                bottom.saturating_sub(top),
            )
        })
    }
}

impl paint::cc::paint::draw_looper::LooperCanvas for TranslatedCanvas<'_, '_> {
    type Matrix = SkM44;

    fn getSaveCount(&self) -> i32 {
        self.target.save_count()
    }

    fn save(&mut self) {
        self.simple(CommandKind::kSave);
    }

    fn restoreToCount(&mut self, count: i32) {
        while self.target.save_count() > count {
            if self.depth == 0 {
                self.error.get_or_insert_with(|| {
                    io::Error::other("translated DrawLooper restore underflow")
                });
                break;
            }
            self.depth -= 1;
            if let Err(error) = self.target.restore() {
                self.error
                    .get_or_insert(raster_error("translated DrawLooper restore failed", error));
                break;
            }
        }
    }

    fn getLocalToDevice(&self) -> Self::Matrix {
        self.target.local_to_device()
    }

    fn setMatrixPostTranslate(&mut self, mut matrix: Self::Matrix, x: f32, y: f32) {
        matrix.postTranslate(x, y, 0.);
        self.target.set_matrix44(&matrix);
    }

    fn translate(&mut self, x: f32, y: f32) {
        self.target.translate(x, y);
    }
}

impl CpuPaintCanvas<SkRegion> for TranslatedCanvas<'_, '_> {
    fn Playback(
        &mut self,
        buffer: &CpuPaintOpBuffer<SkRegion>,
        params: &mut RasterPlaybackParams<'_, SkRegion>,
        local_ctm: bool,
        offsets: Option<&[usize]>,
    ) {
        if self.error.is_some() {
            return;
        }
        let depth = self.depth;
        if local_ctm {
            self.simple(CommandKind::kSave);
        }
        use paint::cc::paint::paint_op_buffer_iterator::{FoldedOp, PlaybackFoldingIterator};
        for item in PlaybackFoldingIterator::new(buffer, offsets) {
            if self.error.is_some() {
                break;
            }
            let draw_alpha = item.alpha * self.opacity;
            match item.op {
                FoldedOp::Borrowed(CpuPaintOpRef::PaintRecord { op, record }) => {
                    self.execute_display_item(op, &resources(record.artifact()), item.alpha);
                }
                FoldedOp::Borrowed(CpuPaintOpRef::Converted(op)) => match op {
                    CpuPaintOp::Atom(source) => {
                        self.execute_display_item(
                            source.op(),
                            &resources(source.record().artifact()),
                            item.alpha,
                        );
                    }
                    CpuPaintOp::DrawRect(value, flags) => {
                        if let Some(shader) = &flags.shader {
                            if let CpuPaintShader::PaintRecord {
                                record,
                                tile_rect,
                                tile_mode_x,
                                tile_mode_y,
                                local_matrix,
                            } = shader.as_ref()
                            {
                                if let Err(error) = self.draw_record_shader_rect(
                                    value,
                                    flags,
                                    draw_alpha,
                                    record,
                                    tile_rect,
                                    *tile_mode_x,
                                    *tile_mode_y,
                                    local_matrix.as_ref(),
                                    params,
                                ) {
                                    self.error.get_or_insert(error);
                                }
                                continue;
                            }
                            if let CpuPaintShader::ImagePattern { source, mask_layer } =
                                shader.as_ref()
                            {
                                let mut command = if let Some(index) = mask_layer {
                                    let layer = source.op().mask_layers.get(*index).ok_or_else(|| {
                                        io::Error::other(
                                            "translated image shader mask index is outside its source",
                                        )
                                    });
                                    let layer = match layer {
                                        Ok(layer) => layer,
                                        Err(error) => {
                                            self.error.get_or_insert(error);
                                            continue;
                                        }
                                    };
                                    DrawCommand {
                                        rect: layer.clip_rect.to_skia(),
                                        source_rect: layer.source_rect.to_skia(),
                                        tile_rect: layer.tile_rect.to_skia(),
                                        resource_id: layer.resource_id,
                                        image_interpolation_quality: layer
                                            .image_interpolation_quality
                                            .to_skia(),
                                        repeat_x: layer.repeat_x,
                                        repeat_y: layer.repeat_y,
                                        tile_scale: layer.tile_scale.to_skia(),
                                        tile_spacing: layer.tile_spacing.to_skia(),
                                        paint_shader: layer.paint_shader.to_skia(),
                                        ..Default::default()
                                    }
                                } else {
                                    source.op().to_skia()
                                };
                                command.r#type = CommandKind::kDrawTiledImage;
                                command.rect = command::PaintRect {
                                    x: value.x() as f64,
                                    y: value.y() as f64,
                                    width: value.width() as f64,
                                    height: value.height() as f64,
                                };
                                command.flags_alpha = flags.getAlphaf() * draw_alpha;
                                if self.nearest {
                                    command.image_interpolation_quality =
                                        ImageInterpolationQuality::kNone;
                                }
                                if let Err(error) = self.draw_tiled_image(
                                    &command,
                                    &resources(source.record().artifact()),
                                    sk_blend_mode(flags.getBlendMode()),
                                    flags.isAntiAlias(),
                                ) {
                                    self.error.get_or_insert(error);
                                }
                                continue;
                            }
                        }
                        let paint = match generated_shader_paint(flags, draw_alpha) {
                            Ok(paint) => paint,
                            Err(error) => {
                                self.error.get_or_insert(error);
                                continue;
                            }
                        };
                        if let Err(error) = unsafe {
                            self.target.draw_rect(
                                &SkRect::MakeLTRB(
                                    value.x(),
                                    value.y(),
                                    value.right(),
                                    value.bottom(),
                                ),
                                &paint,
                            )
                        } {
                            self.error.get_or_insert(raster_error(
                                "translated converted drawRect failed",
                                error,
                            ));
                        }
                    }
                    CpuPaintOp::DrawPath { path: value, flags } => {
                        let paint = match generated_shader_paint(flags, draw_alpha) {
                            Ok(paint) => paint,
                            Err(error) => {
                                self.error.get_or_insert(error);
                                continue;
                            }
                        };
                        if let Err(error) = unsafe { self.target.draw_path(value.as_ref(), &paint) }
                        {
                            self.error.get_or_insert(raster_error(
                                "translated converted drawPath failed",
                                error,
                            ));
                        }
                    }
                    CpuPaintOp::DrawRRect(value, flags) => {
                        if flags.looper.is_some() {
                            self.looped_shape(value, None, flags, draw_alpha);
                            continue;
                        }
                        let paint = match converted_paint(flags, draw_alpha) {
                            Ok(paint) => paint,
                            Err(error) => {
                                self.error.get_or_insert(error);
                                continue;
                            }
                        };
                        if let Err(error) =
                            unsafe { self.target.draw_rrect(&converted_rrect(value), &paint) }
                        {
                            self.error.get_or_insert(raster_error(
                                "translated converted drawRRect failed",
                                error,
                            ));
                        }
                    }
                    CpuPaintOp::DrawDRRect {
                        outer,
                        inner,
                        flags,
                    } => {
                        if flags.looper.is_some() {
                            self.looped_shape(outer, Some(inner), flags, draw_alpha);
                            continue;
                        }
                        let paint = match converted_paint(flags, draw_alpha) {
                            Ok(paint) => paint,
                            Err(error) => {
                                self.error.get_or_insert(error);
                                continue;
                            }
                        };
                        if let Err(error) = unsafe {
                            self.target.draw_drrect(
                                &converted_rrect(outer),
                                &converted_rrect(inner),
                                &paint,
                            )
                        } {
                            self.error.get_or_insert(raster_error(
                                "translated converted drawDRRect failed",
                                error,
                            ));
                        }
                    }
                    CpuPaintOp::Save => self.simple(CommandKind::kSave),
                    CpuPaintOp::Restore => self.simple(CommandKind::kRestore),
                    CpuPaintOp::Translate(x, y) => self.target.translate(*x, *y),
                    CpuPaintOp::Concat(transform) => {
                        let values =
                            std::array::from_fn(|index| transform.rc(index % 4, index / 4) as f32);
                        self.target.concat44(&SkM44::ColMajor(&values));
                    }
                    CpuPaintOp::ClipRect {
                        rect: value,
                        difference,
                        antialias,
                    } => {
                        self.target.clip_rect(
                            &SkRect::MakeLTRB(value.x(), value.y(), value.right(), value.bottom()),
                            if *difference {
                                SkClipOp::kDifference
                            } else {
                                SkClipOp::kIntersect
                            },
                            *antialias,
                        );
                    }
                    CpuPaintOp::ClipRRect(value, antialias) => {
                        let mut rr = SkRRect::default();
                        rr.setRectRadii(
                            &SkRect::MakeLTRB(
                                value.rect.x(),
                                value.rect.y(),
                                value.rect.right(),
                                value.rect.bottom(),
                            ),
                            &value.corner_radii.map(|point| SkPoint {
                                fX: point.x(),
                                fY: point.y(),
                            }),
                        );
                        self.target
                            .clip_rrect(&rr, SkClipOp::kIntersect, *antialias);
                    }
                    CpuPaintOp::ClipPath {
                        path,
                        difference,
                        antialias,
                    } => {
                        let Some(path) = path.single_path() else {
                            self.fail("translated clip-path union replay is not connected");
                            continue;
                        };
                        self.target.clip_path(
                            path,
                            if *difference {
                                SkClipOp::kDifference
                            } else {
                                SkClipOp::kIntersect
                            },
                            *antialias,
                        );
                    }
                    CpuPaintOp::SaveLayer { bounds, flags } => {
                        let mut paint = match converted_restore_paint(flags, 1.) {
                            Ok(paint) => paint,
                            Err(error) => {
                                self.error.get_or_insert(error);
                                continue;
                            }
                        };
                        if let Some(filter) = &flags.image_filter {
                            match translated_filter_graph_srgb(
                                filter.as_ref(),
                                &self.target.local_to_device(),
                                None,
                            ) {
                                Ok(filter) => paint.setImageFilter(filter),
                                Err(error) => {
                                    self.error.get_or_insert(error);
                                    continue;
                                }
                            }
                        }
                        let bounds = bounds.map(|value| {
                            SkRect::MakeLTRB(value.x(), value.y(), value.right(), value.bottom())
                        });
                        match self.target.save_layer(bounds.as_ref(), Some(&paint)) {
                            Ok(_) => self.depth += 1,
                            Err(error) => {
                                self.error.get_or_insert(raster_error(
                                    "translated converted saveLayer failed",
                                    error,
                                ));
                            }
                        }
                    }
                    CpuPaintOp::SaveLayerAlpha { bounds, alpha } => {
                        let mut paint = SkPaint::default();
                        paint.setAlphaf(*alpha);
                        let bounds = bounds.map(|value| {
                            SkRect::MakeLTRB(value.x(), value.y(), value.right(), value.bottom())
                        });
                        match self.target.save_layer(bounds.as_ref(), Some(&paint)) {
                            Ok(_) => self.depth += 1,
                            Err(error) => {
                                self.error.get_or_insert(raster_error(
                                    "translated converted saveLayerAlpha failed",
                                    error,
                                ));
                            }
                        }
                    }
                    CpuPaintOp::SaveLayerFilters {
                        bounds,
                        filters,
                        backdrop_filter,
                        flags,
                    } => {
                        let matrix = self.target.local_to_device();
                        let backdrop = match translated_filter_graph_srgb(
                            backdrop_filter.as_ref(),
                            &matrix,
                            None,
                        ) {
                            Ok(filter) => filter,
                            Err(error) => {
                                self.error.get_or_insert(error);
                                continue;
                            }
                        };
                        let branches = if let Some(filter) = &flags.image_filter {
                            translated_filter_graph_srgb(filter.as_ref(), &matrix, None)
                                .map(|filter| vec![filter])
                        } else {
                            filters
                                .iter()
                                .map(|filter| {
                                    translated_filter_graph_srgb(filter.as_ref(), &matrix, None)
                                })
                                .collect::<io::Result<Vec<_>>>()
                        };
                        let mut branches = match branches {
                            Ok(branches) => branches,
                            Err(error) => {
                                self.error.get_or_insert(error);
                                continue;
                            }
                        };
                        if branches.len() > skia::src::core::SkCanvas::kMaxFiltersPerLayer as usize
                        {
                            self.fail("translated saveLayer has too many independent filters");
                            continue;
                        }
                        let mut paint = match converted_restore_paint(flags, 1.) {
                            Ok(paint) => paint,
                            Err(error) => {
                                self.error.get_or_insert(error);
                                continue;
                            }
                        };
                        if flags.color_filter.is_some() && !branches.is_empty() {
                            let converted = branches
                                .into_iter()
                                .map(luma_image_filter)
                                .collect::<io::Result<Vec<_>>>();
                            branches = match converted {
                                Ok(branches) => branches,
                                Err(error) => {
                                    self.error.get_or_insert(error);
                                    continue;
                                }
                            };
                            paint.setColorFilter(None);
                        }
                        // fFilters supersedes Paint.imageFilter, matching
                        // SkCanvas.cpp's SaveLayerRec normalization.
                        paint.setImageFilter(None);
                        let bounds = bounds.map(|value| {
                            SkRect::MakeLTRB(value.x(), value.y(), value.right(), value.bottom())
                        });
                        let mut record = skia::src::core::SkCanvas::SaveLayerRec::new(
                            bounds.as_ref(),
                            Some(&paint),
                            0,
                        );
                        record.fFilters = &branches;
                        record.fBackdrop = backdrop.as_ref();
                        match self.target.save_layer_rec(&record) {
                            Ok(_) => self.depth += 1,
                            Err(error) => {
                                self.error.get_or_insert(raster_error(
                                    "translated converted saveLayerFilters failed",
                                    error,
                                ));
                            }
                        }
                    }
                    CpuPaintOp::DrawRecord(record) => self.record(record, params),
                    CpuPaintOp::DrawScrollingContents { element, list } => {
                        self.simple(CommandKind::kSave);
                        if let Some(offset) = params
                            .raster_inducing_scroll_offsets
                            .and_then(|scrolls| scrolls.get(element))
                        {
                            self.target.translate(-offset.x(), -offset.y());
                            list.RasterWithParams(self, params);
                        } else {
                            self.fail(
                                "translated DrawScrollingContents requires its scroll offset",
                            );
                        }
                        self.simple(CommandKind::kRestore);
                    }
                    CpuPaintOp::DrawColor {
                        color: value,
                        blend,
                    } => {
                        let value = SkColor4f {
                            fR: value.red,
                            fG: value.green,
                            fB: value.blue,
                            fA: value.alpha * draw_alpha,
                        };
                        if let Err(error) = self.target.draw_color(&value, sk_blend_mode(*blend)) {
                            self.error
                                .get_or_insert(raster_error("translated drawColor failed", error));
                        }
                    }
                    CpuPaintOp::DrawTransparentColorSrc => {
                        if let Err(error) = self.target.draw_color(
                            &SkColor4f {
                                fR: 0.,
                                fG: 0.,
                                fB: 0.,
                                fA: 0.,
                            },
                            SkBlendMode::kSrc,
                        ) {
                            self.error
                                .get_or_insert(raster_error("translated clear failed", error));
                        }
                    }
                    CpuPaintOp::Noop => {}
                },
                FoldedOp::OwnedDrawColor { color: value } => {
                    let value = SkColor4f {
                        fR: value[0],
                        fG: value[1],
                        fB: value[2],
                        fA: value[3] * self.opacity,
                    };
                    if let Err(error) = self.target.draw_color(&value, SkBlendMode::kSrcOver) {
                        self.error
                            .get_or_insert(raster_error("translated folded color failed", error));
                    }
                }
            }
        }
        // Cleanup must continue after a replay error. Calling `simple()` here
        // would hit execute()'s error guard without reducing `depth`, leaving
        // this loop permanently live on an unsupported paint operation.
        self.unwind_to_depth(depth);
    }
}

/// Replay one finalized cc display list into an already configured
/// `skia` target. Renderer owns PictureDrawQuad geometry, clipping and
/// pass state; this adapter owns only PaintOp replay. It is intentionally
/// independent of [`RasterEngine`] and does not allocate, publish or retain a
/// tile resource.
pub fn ReplayDisplayItemListIntoTranslatedTarget(
    list: &ContentDisplayItemList<SkRegion>,
    nearest: bool,
    scrolls: &ScrollOffsetMap,
    opacity: f32,
    target: &mut RasterTarget<'_>,
) -> io::Result<()> {
    // Source uses SkScalarRoundToInt(opacity * 255) < 255 to decide whether
    // OpacityFilterCanvas is installed. Preserve its quantized all-opaque
    // fast path while retaining an independent nearest-only wrapper.
    let wrap = (opacity * 255.).round() < 255. || nearest;
    let opacity = if wrap && opacity < 1. { opacity } else { 1. };
    let mut replay = TranslatedCanvas::new(target, nearest, opacity);
    list.RasterWithParams(
        &mut replay,
        &mut RasterPlaybackParams {
            raster_inducing_scroll_offsets: Some(scrolls),
            image_provider: None,
        },
    );
    replay.unwind_to_depth(0);
    replay.finish_replay()
}

#[cfg(any(test, feature = "artifact_test_support"))]
fn rasterize_paint_artifact(
    list: &PaintArtifact,
    width: u32,
    height: u32,
    scale: f64,
    transparent: bool,
) -> io::Result<Vec<u8>> {
    let row_bytes = (width as usize)
        .checked_mul(4)
        .ok_or_else(|| io::Error::other("translated raster row size overflow"))?;
    let byte_len = row_bytes
        .checked_mul(height as usize)
        .ok_or_else(|| io::Error::other("translated raster target size overflow"))?;
    let mut pixels = vec![0; byte_len];
    let mut target = RasterTarget::new(
        &mut pixels,
        width as i32,
        height as i32,
        row_bytes,
        PixelFormat::Rgba8888,
        None,
    )
    .map_err(|error| raster_error("translated paint artifact target creation failed", error))?;
    if !transparent {
        target
            .draw_color(
                &SkColor4f {
                    fR: 1.,
                    fG: 1.,
                    fB: 1.,
                    fA: 1.,
                },
                SkBlendMode::kSrc,
            )
            .map_err(|error| raster_error("translated paint artifact clear failed", error))?;
    }
    target.scale(scale as f32, scale as f32);
    let command_resources = resources(list);
    let replay_result = {
        let mut canvas = TranslatedCanvas::new(&mut target, false, 1.);
        for item in list.items.iter() {
            canvas.execute_display_item(item, &command_resources, 1.0);
        }
        canvas.unwind_to_depth(0);
        canvas.finish_replay()
    };
    let finish_result = target
        .finish()
        .map_err(|error| raster_error("translated paint artifact finish failed", error));
    replay_result.and(finish_result)?;
    Ok(pixels)
}

/// Rasterize a Paint publication onto an opaque white RGBA target. This is a
/// standalone inspection/export entry point; production tile raster continues
/// to enter through [`RasterEngine`] with compositor-selected work.
#[cfg(any(test, feature = "artifact_test_support"))]
pub fn RasterizePaintArtifact(list: &PaintArtifact, width: u32, height: u32) -> Vec<u8> {
    rasterize_paint_artifact(list, width, height, 1., false)
        .expect("translated PaintArtifact rasterization failed")
}

/// Rasterize a Paint publication onto an opaque white scaled RGBA target.
#[cfg(any(test, feature = "artifact_test_support"))]
pub fn RasterizePaintArtifactWithScale(
    list: &PaintArtifact,
    width: u32,
    height: u32,
    scale: f64,
) -> Vec<u8> {
    rasterize_paint_artifact(list, width, height, scale, false)
        .expect("translated scaled PaintArtifact rasterization failed")
}

/// Rasterize a Paint publication onto a transparent RGBA target.
#[cfg(any(test, feature = "artifact_test_support"))]
pub fn RasterizePaintArtifactTransparent(list: &PaintArtifact, width: u32, height: u32) -> Vec<u8> {
    rasterize_paint_artifact(list, width, height, 1., true)
        .expect("translated transparent PaintArtifact rasterization failed")
}

fn PrepareRasterPlayback(
    canvas: &mut TranslatedCanvas<'_, '_>,
    content_rect: &compositor_core::adapter::geometry::Rect,
    playback_rect: &compositor_core::adapter::geometry::Rect,
    requires_clear: bool,
) -> io::Result<()> {
    let is_partial_raster = content_rect != playback_rect;
    let clear = |canvas: &mut TranslatedCanvas<'_, '_>, message| {
        canvas
            .target
            .draw_color(
                &SkColor4f {
                    fR: 0.,
                    fG: 0.,
                    fB: 0.,
                    fA: 0.,
                },
                SkBlendMode::kSrc,
            )
            .map_err(|error| raster_error(message, error))
    };
    if requires_clear && !is_partial_raster {
        clear(canvas, "translated target clear failed")?;
    }
    canvas.simple(CommandKind::kSave);
    canvas
        .target
        .translate(-content_rect.x as f32, -content_rect.y as f32);
    canvas
        .target
        .clip_rect(&cc_rect(*playback_rect), SkClipOp::kIntersect, false);
    // Source: cc/raster/raster_source.cc:86-111. SkCanvas::clear
    // respects the device clip but ignores the current transform. A partial
    // transparent replay therefore clears only the playback rect after the
    // tile translation and clip are installed.
    if requires_clear && is_partial_raster {
        clear(canvas, "translated playback clear failed")?;
    }
    Ok(())
}

impl RasterBackend for TranslatedSkiaBackend {
    fn Raster(
        &self,
        request: PlatformRasterPlayback<'_, '_>,
        target: &mut RasterBackendTarget,
    ) -> io::Result<()> {
        let size = target.size();
        let format = pixel_format(target.format())?;
        let row_bytes = target.row_bytes();
        let mut target = RasterTarget::new(
            target.pixels_mut(),
            size.width(),
            size.height(),
            row_bytes,
            format,
            None,
        )
        .map_err(|error| raster_error("translated raster target creation failed", error))?;
        let replay_result = (|| -> io::Result<()> {
            let mut canvas = TranslatedCanvas::new(&mut target, false, 1.)
                .with_typeface_cache(&self.typeface_cache);
            let source = request.source;
            PrepareRasterPlayback(
                &mut canvas,
                request.content_rect,
                request.invalid_content_rect,
                source.requires_clear,
            )?;
            let translation = request.raster_transform.translation();
            let scale = request.raster_transform.scale();
            canvas.target.translate(translation.x(), translation.y());
            canvas.target.scale(
                scale.x() / source.recording_scale_factor,
                scale.y() / source.recording_scale_factor,
            );
            let list = source
                .GetDisplayItemList()
                .as_ref()
                .ok_or_else(|| io::Error::other("translated RasterSource has no display list"))?;
            let mut params = RasterPlaybackParams {
                raster_inducing_scroll_offsets: Some(request.raster_inducing_scroll_offsets),
                image_provider: Some(request.image_provider),
            };
            for _ in 0..source.slow_down_raster_scale_factor_for_debug.max(1) {
                list.RasterWithParams(&mut canvas, &mut params);
            }
            canvas.unwind_to_depth(0);
            canvas.finish_replay()
        })();
        let finish_result = target
            .finish()
            .map_err(|error| raster_error("translated raster target finish failed", error));
        match (replay_result, finish_result) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(error), _) | (_, Err(error)) => Err(error),
        }
    }
}

#[cfg(all(test, target_vendor = "apple"))]
mod tests {
    use super::*;

    #[test]
    fn apple_font_configuration_matches_blink_defaults() {
        let mut automatic = SkFont::default();
        ConfigureAppleFont(&mut automatic, FontSmoothing::kAuto);
        assert_eq!(automatic.getEdging(), Edging::kSubpixelAntiAlias);
        assert!(automatic.isSubpixel());
        assert!(automatic.isLinearMetrics());
        assert!(!automatic.isEmbeddedBitmaps());

        let mut antialiased = SkFont::default();
        ConfigureAppleFont(&mut antialiased, FontSmoothing::kAntialiased);
        assert_eq!(antialiased.getEdging(), Edging::kAntiAlias);
        assert_eq!(antialiased.getHinting(), SkFontHinting::kNone);
        assert!(antialiased.isSubpixel());

        let mut unsmoothed = SkFont::default();
        ConfigureAppleFont(&mut unsmoothed, FontSmoothing::kNone);
        assert_eq!(unsmoothed.getEdging(), Edging::kAlias);
        assert!(unsmoothed.isSubpixel());
    }

    #[test]
    fn component_transfer_lut_uses_chromium_double_math_and_u8_truncation() {
        use raster_commands::filter_graph::RenderSvgTransfer;

        let identity = svg_component_transfer_table(&RenderSvgTransfer::Identity);
        assert_eq!(identity, std::array::from_fn(|index| index as u8));

        let linear = svg_component_transfer_table(&RenderSvgTransfer::Linear {
            slope: 0.5,
            intercept: 0.0,
        });
        assert_eq!(linear[3], 1, "1.5 is truncated, not rounded");
        assert_eq!(linear[255], 127, "127.5 is truncated, not rounded");

        let table = svg_component_transfer_table(&RenderSvgTransfer::Table(vec![0.0, 1.0]));
        assert_eq!(table[1], 1);
        assert_eq!(table[254], 254);

        let discrete = svg_component_transfer_table(&RenderSvgTransfer::Discrete(vec![0.25, 0.75]));
        assert_eq!(discrete[0], 63);
        assert_eq!(discrete[127], 63);
        assert_eq!(discrete[128], 191);
    }

    #[test]
    fn turbulence_stitch_extent_saturates_and_truncates_like_chromium() {
        assert_eq!(turbulence_tile_extent(3.9), 3);
        assert_eq!(turbulence_tile_extent(3.0e9), i32::MAX);
        assert_eq!(turbulence_unscaled_tile_extent(7.8, 2.0), 3);
    }

    #[test]
    fn non_positive_specular_constant_selects_diffuse_material() {
        use raster_commands::filter_graph::RenderSvgLightingType::{Diffuse, Specular};

        assert_eq!(resolved_lighting_type(Specular, -1.0), Diffuse);
        assert_eq!(resolved_lighting_type(Specular, 0.0), Diffuse);
        assert_eq!(resolved_lighting_type(Specular, 0.5), Specular);
        assert_eq!(resolved_lighting_type(Diffuse, 0.5), Diffuse);
    }

    #[test]
    fn fe_image_preserve_aspect_ratio_matches_chromium_rect_transform() {
        use raster_commands::filter_graph::{
            RenderSvgMeetOrSlice::{Meet, Slice},
            RenderSvgPreserveAspectRatioAlign::{None, XMaxYMin, XMinYMax},
        };

        let destination = SkRect::MakeXYWH(10.0, 20.0, 100.0, 100.0);
        let source = SkRect::MakeWH(200.0, 100.0);
        let (meet_destination, meet_source) =
            svg_preserve_aspect_ratio_rects(destination, source, XMinYMax, Meet);
        assert_eq!(meet_destination, SkRect::MakeXYWH(10.0, 70.0, 100.0, 50.0));
        assert_eq!(meet_source, source);
        assert_eq!(
            map_svg_rect(
                SkRect::MakeXYWH(10.0, 80.0, 100.0, 20.0),
                meet_destination,
                meet_source,
            ),
            SkRect::MakeXYWH(0.0, 20.0, 200.0, 40.0)
        );

        let (slice_destination, slice_source) =
            svg_preserve_aspect_ratio_rects(destination, source, XMaxYMin, Slice);
        assert_eq!(slice_destination, destination);
        assert_eq!(slice_source, SkRect::MakeXYWH(100.0, 0.0, 100.0, 100.0));

        let (none_destination, none_source) =
            svg_preserve_aspect_ratio_rects(destination, source, None, Slice);
        assert_eq!(none_destination, destination);
        assert_eq!(none_source, source);
    }

    #[test]
    fn fe_image_high_quality_sampling_matches_image_paint_filter() {
        let source = SkRect::MakeWH(20.0, 10.0);
        let enlarged = fe_image_sampling(source, SkRect::MakeWH(40.0, 30.0));
        assert!(enlarged.useCubic());
        assert_eq!(enlarged.cubic(), SkCubicResampler::Mitchell());

        let reduced = fe_image_sampling(source, SkRect::MakeWH(10.0, 5.0));
        assert!(!reduced.useCubic());
        assert_eq!(reduced.filter(), SkFilterMode::kLinear);
        assert_eq!(reduced.mipmap(), SkMipmapMode::kLinear);

        let mixed_axes = fe_image_sampling(source, SkRect::MakeWH(40.0, 5.0));
        assert!(!mixed_axes.useCubic());
        assert_eq!(mixed_axes.filter(), SkFilterMode::kLinear);
        assert_eq!(mixed_axes.mipmap(), SkMipmapMode::kLinear);
    }

    #[test]
    fn ordinary_image_sampling_preserves_filter_quality_mipmap_semantics() {
        let none = image_sampling(ImageInterpolationQuality::kNone);
        assert_eq!(none.filter(), SkFilterMode::kNearest);
        assert_eq!(none.mipmap(), SkMipmapMode::kNone);

        let low = image_sampling(ImageInterpolationQuality::kLow);
        assert_eq!(low.filter(), SkFilterMode::kLinear);
        assert_eq!(low.mipmap(), SkMipmapMode::kNone);

        let medium = image_sampling(ImageInterpolationQuality::kMedium);
        assert_eq!(medium.filter(), SkFilterMode::kLinear);
        assert_eq!(medium.mipmap(), SkMipmapMode::kLinear);
    }

    #[test]
    fn svg_primitive_crop_intersects_the_filter_region() {
        assert_eq!(
            svg_filter_crop(
                SkRect::MakeXYWH(-10.0, -20.0, 50.0, 60.0),
                SkRect::MakeXYWH(0.0, 0.0, 20.0, 20.0),
            ),
            SkRect::MakeXYWH(0.0, 0.0, 20.0, 20.0)
        );
        assert!(svg_filter_crop(
            SkRect::MakeXYWH(30.0, 30.0, 10.0, 10.0),
            SkRect::MakeXYWH(0.0, 0.0, 20.0, 20.0),
        )
        .isEmpty());
    }

    #[test]
    fn oriented_image_pixels_keep_source_rects_in_oriented_space() {
        // Encoded storage is two columns by three rows:
        //   1 2
        //   3 4
        //   5 6
        // A clockwise quarter-turn is three columns by two rows. Paint's
        // source rectangle may therefore crop the oriented left column
        // directly; unlike Chromium's DrawImage receiver, this raster boundary
        // has already applied CorrectSrcRectForImageOrientation.
        // cpp: core/paint/image_painter.cc:239-244
        let pixels = (1u8..=6)
            .flat_map(|label| [label, 0, 0, u8::MAX])
            .collect::<Vec<_>>();
        let (oriented, width, height) =
            orient_image_pixels(pixels, 2, 3, ImageOrientation::kRotate90)
                .expect("orient image pixels");
        assert_eq!((width, height), (3, 2));
        assert_eq!(
            oriented
                .chunks_exact(4)
                .map(|pixel| pixel[0])
                .collect::<Vec<_>>(),
            vec![5, 3, 1, 6, 4, 2]
        );
        assert_eq!(
            (0..height)
                .map(|y| oriented[(y * width) * 4])
                .collect::<Vec<_>>(),
            vec![5, 6]
        );
    }

    #[test]
    fn flat_paint_record_replays_reference_filter_with_paint_geometry() {
        use layoutng_assembly::internal::{
            layout_input::Offset,
            layout_input_types::Color,
            paint_input::{
                PaintFilterOperation, PaintFilterType, SvgFilterGraph, SvgFilterInput,
                SvgFilterPrimitive, SvgFilterPrimitiveKind, SvgFilterRegion, SvgFilterUnits,
            },
        };
        use paint::paint_engine::{DisplayItem, DisplayItemType, PaintArtifact, PaintRect};

        let reference = PaintFilterOperation {
            r#type: PaintFilterType::kReference,
            reference_graph: Some(Arc::new(SvgFilterGraph {
                id: "offset".into(),
                filter_units: SvgFilterUnits::kUserSpaceOnUse,
                primitive_units: SvgFilterUnits::kUserSpaceOnUse,
                region: SvgFilterRegion::default(),
                primitives: vec![SvgFilterPrimitive {
                    input: SvgFilterInput::kSourceGraphic,
                    result: "shifted".into(),
                    region: SvgFilterRegion::default(),
                    linear_rgb: false,
                    origin_tainted: false,
                    kind: SvgFilterPrimitiveKind::kOffset { dx: 2.0, dy: 0.0 },
                }],
                fill_paint: None,
                stroke_paint: None,
                unsupported: None,
            })),
            ..Default::default()
        };
        let box_rect = PaintRect {
            x: 0.0,
            y: 0.0,
            width: 16.0,
            height: 16.0,
        };
        let canonical_filters =
            paint::property_filter_operations::PropertyFilterOperations::from_authored(
                &[reference],
                Some(box_rect),
                Some(box_rect),
                Offset::default(),
                1.0,
            );
        let artifact = PaintArtifact {
            items: Arc::new(vec![
                DisplayItem {
                    r#type: DisplayItemType::kSaveLayerFilter,
                    canonical_filters,
                    ..Default::default()
                },
                DisplayItem {
                    r#type: DisplayItemType::kDrawRect,
                    rect: PaintRect {
                        x: 2.0,
                        y: 2.0,
                        width: 4.0,
                        height: 4.0,
                    },
                    color: Color {
                        red: 1.0,
                        green: 0.0,
                        blue: 0.0,
                        alpha: 1.0,
                    },
                    ..Default::default()
                },
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    ..Default::default()
                },
            ]),
            ..Default::default()
        };

        let pixels = rasterize_paint_artifact(&artifact, 16, 16, 1.0, true)
            .expect("flat reference filter replay");
        let alpha = |x: usize, y: usize| pixels[(y * 16 + x) * 4 + 3];
        assert_eq!(alpha(2, 3), 0, "source position must be shifted");
        assert_ne!(alpha(5, 3), 0, "reference-filter output must be drawn");
    }

    #[test]
    fn reference_filter_retains_linear_output_until_the_filter_chain_tail() {
        use layoutng_assembly::internal::{
            layout_input::Offset,
            paint_input::{
                PaintFilterOperation, PaintFilterType, SvgFilterGraph, SvgFilterInput,
                SvgFilterPrimitive, SvgFilterPrimitiveKind, SvgFilterRegion, SvgFilterUnits,
            },
        };
        use paint::paint_engine::PaintRect;

        let reference = PaintFilterOperation {
            r#type: PaintFilterType::kReference,
            reference_graph: Some(Arc::new(SvgFilterGraph {
                id: "linear-output".into(),
                filter_units: SvgFilterUnits::kUserSpaceOnUse,
                primitive_units: SvgFilterUnits::kUserSpaceOnUse,
                region: SvgFilterRegion::default(),
                primitives: vec![SvgFilterPrimitive {
                    input: SvgFilterInput::kSourceGraphic,
                    result: "linear".into(),
                    region: SvgFilterRegion::default(),
                    linear_rgb: true,
                    origin_tainted: false,
                    kind: SvgFilterPrimitiveKind::kOffset { dx: 0.0, dy: 0.0 },
                }],
                fill_paint: None,
                stroke_paint: None,
                unsupported: None,
            })),
            ..Default::default()
        };
        let reference_box = PaintRect {
            width: 16.0,
            height: 16.0,
            ..Default::default()
        };
        let canonical = paint::property_filter_operations::PropertyFilterOperations::from_authored(
            &[reference],
            Some(reference_box),
            Some(reference_box),
            Offset::default(),
            1.0,
        );
        let (_, linear_rgb) = translated_filter_operations_in_space(
            canonical.canonical().expect("canonical reference filter"),
            &SkM44::default(),
            None,
            false,
        )
        .expect("translated reference filter");
        assert!(
            linear_rgb,
            "the chain tail, not the primitive, converts to sRGB"
        );
        assert!(
            translated_filter_operations(
                canonical.canonical().expect("canonical reference filter"),
                &SkM44::default(),
                None,
            )
            .expect("translated filter chain")
            .is_some(),
            "the public chain builder must append the final sRGB conversion"
        );
    }

    #[test]
    fn fe_image_output_stays_srgb_when_authored_interpolation_is_linear() {
        use layoutng_assembly::internal::{
            layout_input::Offset,
            paint_input::{
                PaintFilterOperation, PaintFilterType, SvgFilterGraph, SvgFilterInput,
                SvgFilterPrimitive, SvgFilterPrimitiveKind, SvgFilterRegion, SvgFilterUnits,
            },
        };
        use paint::paint_engine::PaintRect;

        let reference = PaintFilterOperation {
            r#type: PaintFilterType::kReference,
            reference_graph: Some(Arc::new(SvgFilterGraph {
                id: "fixed-srgb-image".into(),
                filter_units: SvgFilterUnits::kUserSpaceOnUse,
                primitive_units: SvgFilterUnits::kUserSpaceOnUse,
                region: SvgFilterRegion::default(),
                primitives: vec![SvgFilterPrimitive {
                    input: SvgFilterInput::kSourceGraphic,
                    result: String::new(),
                    region: SvgFilterRegion::default(),
                    linear_rgb: true,
                    origin_tainted: false,
                    kind: SvgFilterPrimitiveKind::kImage {
                        href: "memory:image".into(),
                        local_subtree: false,
                        preserve_aspect_ratio: Default::default(),
                        resource_id: None,
                        width: 1,
                        height: 1,
                        orientation: Default::default(),
                        pixels: Arc::new(vec![255, 0, 0, 255]),
                    },
                }],
                fill_paint: None,
                stroke_paint: None,
                unsupported: None,
            })),
            ..Default::default()
        };
        let reference_box = PaintRect {
            width: 16.0,
            height: 16.0,
            ..Default::default()
        };
        let canonical = paint::property_filter_operations::PropertyFilterOperations::from_authored(
            &[reference],
            Some(reference_box),
            Some(reference_box),
            Offset::default(),
            1.0,
        );
        let (_, linear_rgb) = translated_filter_operations_in_space(
            canonical.canonical().expect("canonical reference filter"),
            &SkM44::default(),
            None,
            false,
        )
        .expect("translated reference filter");
        assert!(!linear_rgb, "FEImage ignores color-interpolation-filters");
    }

    #[test]
    fn arithmetic_filter_builder_keeps_distinct_premul_validation_variants() {
        use layoutng_assembly::internal::{
            layout_input::Offset,
            layout_input_types::Color,
            paint_input::{
                PaintFilterOperation, PaintFilterType, SvgCompositeOperator, SvgFilterGraph,
                SvgFilterInput, SvgFilterPrimitive, SvgFilterPrimitiveKind, SvgFilterRegion,
                SvgFilterUnits,
            },
        };
        use paint::paint_engine::{DisplayItem, DisplayItemType, PaintArtifact, PaintRect};

        let variants = svg_arithmetic_filter_variants(
            [0.0, 2.0, 0.0, -0.5],
            None,
            None,
            SkRect::MakeWH(8.0, 8.0),
        );
        let serialized = |filter: &Option<Arc<dyn SkImageFilter>>| {
            let owner = skia::src::core::SkFlattenable::FlattenableOwner::ImageFilter(
                filter.clone().expect("arithmetic filter"),
            );
            let data = owner
                .as_view()
                .serialize(None)
                .expect("serialize arithmetic filter");
            unsafe { std::slice::from_raw_parts(data.data(), data.size()) }.to_vec()
        };
        assert_ne!(
            serialized(&variants.0),
            serialized(&variants.1),
            "the enforce-premul bit must remain part of the immutable Skia graph"
        );

        let primitive = |input: SvgFilterInput, result: &str, k: [f64; 4]| SvgFilterPrimitive {
            input,
            result: result.into(),
            region: SvgFilterRegion::default(),
            linear_rgb: false,
            origin_tainted: false,
            kind: SvgFilterPrimitiveKind::kComposite {
                operator: SvgCompositeOperator::kArithmetic,
                input2: SvgFilterInput::kSourceGraphic,
                k,
            },
        };
        let reference = PaintFilterOperation {
            r#type: PaintFilterType::kReference,
            reference_graph: Some(Arc::new(SvgFilterGraph {
                id: "arithmetic-validation".into(),
                filter_units: SvgFilterUnits::kUserSpaceOnUse,
                primitive_units: SvgFilterUnits::kUserSpaceOnUse,
                region: SvgFilterRegion::default(),
                primitives: vec![
                    primitive(
                        SvgFilterInput::kSourceGraphic,
                        "invalid",
                        [0.0, 2.0, 0.0, -0.5],
                    ),
                    primitive(
                        SvgFilterInput::kNamed("invalid".into()),
                        "output",
                        [0.0, 0.5, 0.0, 0.0],
                    ),
                ],
                fill_paint: None,
                stroke_paint: None,
                unsupported: None,
            })),
            ..Default::default()
        };
        let reference_box = PaintRect {
            width: 8.0,
            height: 8.0,
            ..Default::default()
        };
        let canonical_filters =
            paint::property_filter_operations::PropertyFilterOperations::from_authored(
                &[reference],
                Some(reference_box),
                Some(reference_box),
                Offset::default(),
                1.0,
            );
        let artifact = PaintArtifact {
            items: Arc::new(vec![
                DisplayItem {
                    r#type: DisplayItemType::kSaveLayerFilter,
                    canonical_filters,
                    ..Default::default()
                },
                DisplayItem {
                    r#type: DisplayItemType::kDrawRect,
                    rect: PaintRect {
                        x: 1.0,
                        y: 1.0,
                        width: 4.0,
                        height: 4.0,
                    },
                    color: Color {
                        red: 1.0,
                        green: 0.0,
                        blue: 0.0,
                        alpha: 1.0,
                    },
                    ..Default::default()
                },
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    ..Default::default()
                },
            ]),
            ..Default::default()
        };
        let pixels =
            rasterize_paint_artifact(&artifact, 8, 8, 1.0, true).expect("arithmetic filter replay");
        let pixel = &pixels[(2 * 8 + 2) * 4..][..4];
        assert_eq!(pixel, &[128, 0, 0, 128]);
    }

    #[test]
    fn tainted_displacement_input_is_a_pass_through_filter() {
        use layoutng_assembly::internal::{
            layout_input::Offset,
            layout_input_types::Color,
            paint_input::{
                PaintFilterOperation, PaintFilterType, SvgFilterChannel, SvgFilterGraph,
                SvgFilterInput, SvgFilterPrimitive, SvgFilterPrimitiveKind, SvgFilterRegion,
                SvgFilterUnits,
            },
        };
        use paint::PaintRect;

        let flood = |result: &str, color: Color, origin_tainted| SvgFilterPrimitive {
            input: SvgFilterInput::kSourceGraphic,
            result: result.into(),
            region: SvgFilterRegion::default(),
            linear_rgb: false,
            origin_tainted,
            kind: SvgFilterPrimitiveKind::kFlood { color },
        };
        let blue = Color {
            red: 0.0,
            green: 0.0,
            blue: 1.0,
            alpha: 1.0,
        };
        let reference_box = PaintRect {
            width: 8.0,
            height: 8.0,
            ..Default::default()
        };
        let build = |primitives| {
            let operation = PaintFilterOperation {
                r#type: PaintFilterType::kReference,
                reference_graph: Some(Arc::new(SvgFilterGraph {
                    id: "displacement-taint".into(),
                    filter_units: SvgFilterUnits::kUserSpaceOnUse,
                    primitive_units: SvgFilterUnits::kUserSpaceOnUse,
                    region: SvgFilterRegion::default(),
                    primitives,
                    fill_paint: None,
                    stroke_paint: None,
                    unsupported: None,
                })),
                ..Default::default()
            };
            let canonical =
                paint::property_filter_operations::PropertyFilterOperations::from_authored(
                    &[operation],
                    Some(reference_box),
                    Some(reference_box),
                    Offset::default(),
                    1.0,
                );
            BuildTranslatedFilterOperations(
                canonical.canonical().expect("canonical SVG filter"),
                &SkM44::default(),
            )
            .expect("translated SVG filter")
        };
        let restricted = build(vec![
            flood(
                "tainted",
                Color {
                    red: 1.0,
                    green: 0.0,
                    blue: 0.0,
                    alpha: 1.0,
                },
                true,
            ),
            flood("color", blue, false),
            SvgFilterPrimitive {
                input: SvgFilterInput::kNamed("color".into()),
                result: "output".into(),
                region: SvgFilterRegion::default(),
                linear_rgb: false,
                origin_tainted: false,
                kind: SvgFilterPrimitiveKind::kDisplacementMap {
                    input2: SvgFilterInput::kNamed("tainted".into()),
                    scale: 20.0,
                    x_channel: SvgFilterChannel::kR,
                    y_channel: SvgFilterChannel::kG,
                },
            },
        ]);
        let expected = build(vec![flood("output", blue, false)]);
        let serialized = |filter: Option<Arc<dyn SkImageFilter>>| {
            let owner = skia::src::core::SkFlattenable::FlattenableOwner::ImageFilter(
                filter.expect("image filter"),
            );
            let data = owner.as_view().serialize(None).expect("serialized filter");
            unsafe { std::slice::from_raw_parts(data.data(), data.size()) }.to_vec()
        };
        assert_eq!(serialized(restricted), serialized(expected));
    }

    #[test]
    fn extended_svg_effects_build_filter_graphs() {
        use layoutng_assembly::internal::{
            layout_input::Offset,
            layout_input_types::Color,
            paint_input::{
                PaintFilterOperation, PaintFilterType, SvgFilterEdgeMode, SvgFilterGraph,
                SvgFilterInput, SvgFilterPrimitive, SvgFilterPrimitiveKind, SvgFilterRegion,
                SvgFilterUnits, SvgLightSource, SvgLightingType, SvgTurbulenceType,
            },
        };
        use paint::paint_engine::PaintRect;

        let kinds = [
            SvgFilterPrimitiveKind::kDropShadow {
                sigma_x: 2.0,
                sigma_y: 3.0,
                dx: 4.0,
                dy: 5.0,
                color: Color {
                    red: 0.2,
                    green: 0.4,
                    blue: 0.6,
                    alpha: 0.5,
                },
            },
            SvgFilterPrimitiveKind::kTurbulence {
                base_x: 0.05,
                base_y: 0.08,
                octaves: 2,
                seed: 3.0,
                stitch: true,
                kind: SvgTurbulenceType::kFractalNoise,
            },
            SvgFilterPrimitiveKind::kTurbulence {
                base_x: -0.05,
                base_y: 0.08,
                octaves: 0,
                seed: 3.0,
                stitch: false,
                kind: SvgTurbulenceType::kTurbulence,
            },
            SvgFilterPrimitiveKind::kTurbulence {
                base_x: 0.05,
                base_y: 0.08,
                octaves: -1,
                seed: 3.0,
                stitch: false,
                kind: SvgTurbulenceType::kTurbulence,
            },
            SvgFilterPrimitiveKind::kImage {
                href: "memory:image".into(),
                local_subtree: false,
                preserve_aspect_ratio: Default::default(),
                resource_id: Some(7),
                width: 2,
                height: 1,
                orientation: Default::default(),
                pixels: Arc::new(vec![0xff, 0x00, 0x00, 0xff, 0x00, 0x00, 0xff, 0xff]),
            },
            SvgFilterPrimitiveKind::kConvolveMatrix {
                order_x: 1,
                order_y: 1,
                kernel: vec![1.0],
                divisor: 1.0,
                bias: 0.0,
                target_x: 0,
                target_y: 0,
                edge_mode: SvgFilterEdgeMode::kDuplicate,
                preserve_alpha: false,
            },
            SvgFilterPrimitiveKind::kConvolveMatrix {
                order_x: 2,
                order_y: 2,
                kernel: vec![1.0],
                divisor: 0.0,
                bias: 0.0,
                target_x: -1,
                target_y: 3,
                edge_mode: SvgFilterEdgeMode::kNone,
                preserve_alpha: false,
            },
            SvgFilterPrimitiveKind::kConvolveMatrix {
                order_x: 2,
                order_y: 1,
                kernel: vec![1.0, 2.0],
                divisor: 0.0,
                bias: 0.0,
                target_x: 1,
                target_y: 0,
                edge_mode: SvgFilterEdgeMode::kNone,
                preserve_alpha: false,
            },
            SvgFilterPrimitiveKind::kLighting {
                lighting_type: SvgLightingType::kDiffuse,
                light: Some(SvgLightSource::kDistant {
                    azimuth: 45.0,
                    elevation: 60.0,
                }),
                color: Color {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
                surface_scale: 1.0,
                constant: 1.0,
                specular_exponent: 1.0,
            },
            SvgFilterPrimitiveKind::kLighting {
                lighting_type: SvgLightingType::kSpecular,
                light: Some(SvgLightSource::kSpot {
                    x: 1.0,
                    y: 2.0,
                    z: 3.0,
                    points_at_x: 4.0,
                    points_at_y: 5.0,
                    points_at_z: 6.0,
                    specular_exponent: 0.5,
                    limiting_cone_angle: 0.0,
                }),
                color: Color {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
                surface_scale: 1.0,
                constant: -2.0,
                specular_exponent: 200.0,
            },
        ];
        let reference_box = PaintRect {
            width: 16.0,
            height: 16.0,
            ..Default::default()
        };
        for (index, kind) in kinds.into_iter().enumerate() {
            let reference = PaintFilterOperation {
                r#type: PaintFilterType::kReference,
                reference_graph: Some(Arc::new(SvgFilterGraph {
                    id: "extended-effect".into(),
                    filter_units: SvgFilterUnits::kUserSpaceOnUse,
                    primitive_units: SvgFilterUnits::kUserSpaceOnUse,
                    region: SvgFilterRegion::default(),
                    primitives: vec![SvgFilterPrimitive {
                        input: if index == 0 {
                            SvgFilterInput::kNamed("missing".into())
                        } else {
                            SvgFilterInput::kSourceGraphic
                        },
                        result: "output".into(),
                        region: SvgFilterRegion::default(),
                        linear_rgb: false,
                        origin_tainted: false,
                        kind,
                    }],
                    fill_paint: None,
                    stroke_paint: None,
                    unsupported: None,
                })),
                ..Default::default()
            };
            let canonical =
                paint::property_filter_operations::PropertyFilterOperations::from_authored(
                    &[reference],
                    Some(reference_box),
                    Some(reference_box),
                    Offset::default(),
                    1.0,
                );
            assert!(BuildTranslatedFilterOperations(
                canonical.canonical().expect("canonical reference filter"),
                &SkM44::default(),
            )
            .expect("extended SVG filter translation")
            .is_some());
        }
    }

    #[test]
    fn flat_paint_record_replays_box_reflection_and_keeps_pending_mask_transparent() {
        use layoutng_assembly::internal::{
            layout_input::{Offset, Size},
            layout_input_types::Color,
            paint_input::{
                PaintFilterOperation, PaintFilterType, PaintReflectionDirection,
                PaintReflectionMaskLayer,
            },
        };
        use paint::paint_engine::{DisplayItem, DisplayItemType, PaintArtifact, PaintRect};

        let render = |reflection_has_mask, reflection_mask_layers| {
            let reflection = PaintFilterOperation {
                r#type: PaintFilterType::kBoxReflect,
                reflection_direction: PaintReflectionDirection::kBelow,
                reflection_offset: 12.0,
                reflection_has_mask,
                reflection_mask_layers,
                ..Default::default()
            };
            let canonical_filters =
                paint::property_filter_operations::PropertyFilterOperations::from_authored(
                    &[reflection],
                    None,
                    None,
                    Offset::default(),
                    1.0,
                );
            let artifact = PaintArtifact {
                items: Arc::new(vec![
                    DisplayItem {
                        r#type: DisplayItemType::kSaveLayerFilter,
                        canonical_filters,
                        ..Default::default()
                    },
                    DisplayItem {
                        r#type: DisplayItemType::kDrawRect,
                        rect: PaintRect {
                            x: 2.0,
                            y: 2.0,
                            width: 4.0,
                            height: 2.0,
                        },
                        color: Color {
                            red: 1.0,
                            green: 0.0,
                            blue: 0.0,
                            alpha: 1.0,
                        },
                        ..Default::default()
                    },
                    DisplayItem {
                        r#type: DisplayItemType::kRestore,
                        ..Default::default()
                    },
                ]),
                ..Default::default()
            };
            rasterize_paint_artifact(&artifact, 16, 16, 1.0, true)
                .expect("flat box-reflection replay")
        };

        let unmasked = render(false, Vec::new());
        let alpha = |pixels: &[u8], x: usize, y: usize| pixels[(y * 16 + x) * 4 + 3];
        assert_ne!(alpha(&unmasked, 3, 3), 0, "source must remain visible");
        assert_ne!(alpha(&unmasked, 3, 8), 0, "reflection must be drawn");
        assert_eq!(alpha(&unmasked, 3, 6), 0, "gap must remain transparent");

        let pending_mask = render(true, Vec::new());
        assert_ne!(
            alpha(&pending_mask, 3, 3),
            0,
            "pending mask must not erase the source"
        );
        assert_eq!(
            alpha(&pending_mask, 3, 8),
            0,
            "specified pending mask must not degrade to an unmasked reflection"
        );

        let bitmap_mask = PaintReflectionMaskLayer {
            image: Some(image_resource::PaintImage {
                id: 42,
                width: 1,
                height: 1,
                content: image_resource::PaintImageContent::Bitmap(Arc::new(vec![
                    0xff, 0xff, 0xff, 0xff,
                ])),
                ..Default::default()
            }),
            clip_origin: Offset { x: 2.0, y: 2.0 },
            clip_size: Size {
                width: 4.0,
                height: 2.0,
            },
            source_origin: Offset::default(),
            source_size: Size {
                width: 1.0,
                height: 1.0,
            },
            tile_origin: Offset { x: 2.0, y: 2.0 },
            tile_size: Size {
                width: 4.0,
                height: 2.0,
            },
            ..Default::default()
        };
        let resolved_mask = render(true, vec![bitmap_mask]);
        assert_ne!(
            alpha(&resolved_mask, 3, 8),
            0,
            "resolved opaque bitmap mask must admit the reflected branch"
        );
    }

    #[test]
    fn flat_paint_record_replays_canonical_scalar_filter_without_authored_input() {
        use layoutng_assembly::internal::{
            layout_input::Offset,
            layout_input_types::Color,
            paint_input::{PaintFilterOperation, PaintFilterType},
        };
        use paint::paint_engine::{DisplayItem, DisplayItemType, PaintArtifact, PaintRect};

        let canonical_filters =
            paint::property_filter_operations::PropertyFilterOperations::from_authored(
                &[PaintFilterOperation {
                    r#type: PaintFilterType::kOpacity,
                    amount: 0.5,
                    ..Default::default()
                }],
                None,
                None,
                Offset::default(),
                1.0,
            );
        let artifact = PaintArtifact {
            items: Arc::new(vec![
                DisplayItem {
                    r#type: DisplayItemType::kSaveLayerFilter,
                    canonical_filters,
                    ..Default::default()
                },
                DisplayItem {
                    r#type: DisplayItemType::kDrawRect,
                    rect: PaintRect {
                        x: 1.0,
                        y: 1.0,
                        width: 4.0,
                        height: 4.0,
                    },
                    color: Color {
                        red: 1.0,
                        green: 0.0,
                        blue: 0.0,
                        alpha: 1.0,
                    },
                    ..Default::default()
                },
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    ..Default::default()
                },
            ]),
            ..Default::default()
        };

        let pixels = rasterize_paint_artifact(&artifact, 8, 8, 1.0, true)
            .expect("flat scalar filter replay");
        assert!((126..=129).contains(&pixels[(2 * 8 + 2) * 4 + 3]));
    }

    #[test]
    fn partial_transparent_playback_preserves_clean_pixels() {
        let mut pixels = vec![u8::MAX; 8 * 8 * 4];
        let mut target = RasterTarget::new(&mut pixels, 8, 8, 8 * 4, PixelFormat::Rgba8888, None)
            .expect("test raster target");
        {
            let mut canvas = TranslatedCanvas::new(&mut target, false, 1.);
            let content = compositor_core::adapter::geometry::Rect::new(10, 20, 8, 8);
            let dirty = compositor_core::adapter::geometry::Rect::new(12, 22, 2, 3);
            PrepareRasterPlayback(&mut canvas, &content, &dirty, true)
                .expect("prepare partial raster");
            canvas.unwind_to_depth(0);
            canvas.finish_replay().expect("finish partial raster");
        }
        target.finish().expect("finish translated target");
        for y in 0..8 {
            for x in 0..8 {
                let alpha = pixels[(y * 8 + x) * 4 + 3];
                let inside_dirty = (2..4).contains(&x) && (2..5).contains(&y);
                assert_eq!(alpha, if inside_dirty { 0 } else { u8::MAX }, "{x},{y}");
            }
        }
    }

    #[test]
    fn glyph_run_reaches_translated_canvas() {
        static FONT: &[u8] = include_bytes!("../../../../assets/fonts/Roboto-Regular.ttf");
        let mut pixels = vec![0; 128 * 128 * 4];
        let mut target =
            RasterTarget::new(&mut pixels, 128, 128, 128 * 4, PixelFormat::Rgba8888, None)
                .expect("test raster target");
        {
            let mut canvas = TranslatedCanvas::new(&mut target, false, 1.);
            let resources = ResourceContext {
                resources: Some(command::ResourceCatalog {
                    fonts: vec![command::FontFace {
                        family: "Roboto",
                        resolved_family: "Roboto",
                        weight: 400.,
                        italic: false,
                        bytes: FONT,
                        bytes_owner: None,
                        face_index: 0,
                        variations: Vec::new(),
                    }]
                    .into(),
                    images: Vec::new(),
                }),
            };
            canvas.execute(
                &DrawCommand {
                    r#type: CommandKind::kDrawGlyphRun,
                    color: command::Color {
                        red: 0.,
                        green: 0.,
                        blue: 0.,
                        alpha: 1.,
                    },
                    font_size: 64.,
                    text_blob_origin: command::Offset { x: 16., y: 88. },
                    // Roboto's capital A. Paint supplies glyph IDs directly; this
                    // test intentionally performs no text shaping in the backend.
                    glyphs: vec![command::PaintGlyph {
                        id: 37,
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                &resources,
            );
            canvas
                .finish_replay()
                .expect("finish translated glyph replay");
        }
        target.finish().expect("finish translated glyph target");
        assert!(
            pixels.chunks_exact(4).any(|pixel| pixel[3] != 0),
            "translated glyph draw produced no coverage"
        );
    }

    #[test]
    fn command_color_matrix_variants_build_filters() {
        let mut values = vec![0.0; 20];
        values[0] = 1.0;
        values[6] = 1.0;
        values[12] = 1.0;
        values[18] = 1.0;
        for operation in [
            command::PaintFilterOperation {
                r#type: command::PaintFilterType::kLuminanceToAlpha,
                ..Default::default()
            },
            command::PaintFilterOperation {
                r#type: command::PaintFilterType::kColorMatrix,
                color_matrix: values,
                ..Default::default()
            },
        ] {
            assert!(translated_command_filters(&[operation], &SkM44::default())
                .expect("explicit command color matrix")
                .is_some());
        }
    }

    #[test]
    fn command_canvas_effect_classes_build_filters() {
        use raster_commands::filter_graph::{
            RenderSvgEdgeMode, RenderSvgFilterKind, RenderSvgTransfer,
        };

        let cases = [
            (
                command::PaintFilterType::kConvolveMatrix,
                RenderSvgFilterKind::Convolve {
                    order_x: 2,
                    order_y: 2,
                    kernel: vec![1.0; 4],
                    divisor: 0.0,
                    bias: 0.0,
                    target_x: 0,
                    target_y: 1,
                    edge_mode: RenderSvgEdgeMode::Duplicate,
                    preserve_alpha: false,
                },
            ),
            (
                command::PaintFilterType::kComponentTransfer,
                RenderSvgFilterKind::ComponentTransfer(std::array::from_fn(|_| {
                    RenderSvgTransfer::Identity
                })),
            ),
            (
                command::PaintFilterType::kTurbulence,
                RenderSvgFilterKind::Turbulence {
                    base_x: 0.1,
                    base_y: 0.2,
                    octaves: 12,
                    seed: 3.0,
                    stitch: true,
                    fractal_noise: true,
                },
            ),
        ];
        for (r#type, canvas_filter) in cases {
            let operation = command::PaintFilterOperation {
                r#type,
                canvas_filter: Some(canvas_filter),
                ..Default::default()
            };
            assert!(translated_command_filters(&[operation], &SkM44::default())
                .expect("typed Canvas command filter")
                .is_some());
        }
    }
}
