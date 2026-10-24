//! Paint replay into the source-aligned `skia` Canvas.
//!
//! This receiver is private to `raster`.  It deliberately consumes the same
//! cc-selected `RasterPlayback` as the legacy receiver and writes the same
//! unpublished engine allocation.  Unsupported replay vocabulary is returned
//! as an error; the engine never falls back to the legacy receiver after a
//! partially executed command stream.

use super::{backend::*, *};
use crate::convert::{resources, ToSkia};
use compositor_core::{
    adapter::{
        display_item_list::DisplayCanvas, geometry::Rect, picture_layer_impl::PictureRasterSource,
    },
    runtime::paint::{
        ContentDisplayItemList, CpuPaintCanvas, CpuPaintColorFilter, CpuPaintFlags, CpuPaintOp,
        CpuPaintOpBuffer, CpuPaintOpRef, CpuPaintRecord, CpuPaintShader, RasterPlaybackParams,
    },
};
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
        SkPathTypes::{SkPathDirection, SkPathFillType},
        SkSamplingOptions::{SkFilterMode, SkSamplingOptions},
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
        // native reconstruction is explicitly size-dependent.
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
    // for the exact native-font identity check below.
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
        fn CTFontCopyTable(font: CTFontRef, tag: u32, options: u32) -> *mut CFData;
    }

    fn native_table_matches(font: CTFontRef, bytes: &[u8], face_index: u32, tag: &[u8; 4]) -> bool {
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

    /// Apple's private `hvgl` system faces cannot be reconstructed from SFNT
    /// data. Font selection already chose the exact face, so recover its native
    /// PostScript identity and accept it only when the same core SFNT tables
    /// match. This is the same guarded path used by the host-readable receiver.
    pub(super) fn make(
        bytes: &[u8],
        face_index: u32,
        size: f32,
        arguments: &SkFontArguments,
    ) -> Option<skia::src::core::SkTypeface::SkTypefaceRef> {
        sfnt_face_table(bytes, face_index, b"hvgl")?;
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
            if ![b"name", b"head", b"maxp", b"cmap"]
                .iter()
                .all(|tag| native_table_matches(font.get(), bytes, face_index, *tag))
            {
                continue;
            }
            let base = unsafe { SkMakeTypefaceFromCTFont(font.get()) }?;
            return unsafe { base.makeClone(arguments) };
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
        skia::src::core::SkStream::TransferableStreamAsset::from_native_transferable(stream)
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

fn native_blend(mode: paint::cc::paint::blend_mode::BlendMode) -> SkBlendMode {
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
    paint.setBlendMode(native_blend(flags.getBlendMode()));
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

fn translated_filter_operations(
    operations: &paint::cc::paint::filter_operations::FilterOperations,
    matrix: &SkM44,
    mut input: Option<Arc<dyn SkImageFilter>>,
) -> io::Result<Option<Arc<dyn SkImageFilter>>> {
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
                    translated_filter_graph(filter.as_ref(), matrix, input)?
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
    Ok(input)
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
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "translated command box-reflect filter requires reflection geometry",
                ));
            }
            PaintFilterType::kBrightness
            | PaintFilterType::kContrast
            | PaintFilterType::kGrayscale
            | PaintFilterType::kHueRotate
            | PaintFilterType::kInvert
            | PaintFilterType::kOpacity
            | PaintFilterType::kSaturate
            | PaintFilterType::kSepia => unreachable!("handled command CSS color matrix"),
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
) -> io::Result<Option<Arc<dyn SkImageFilter>>> {
    use paint::cc::paint::filter_operation::PaintFilterReplay;
    match filter.ReplayGraph() {
        PaintFilterReplay::Operations(operations) => {
            translated_filter_operations(operations, matrix, input)
        }
        PaintFilterReplay::Reference {
            operation,
            reference_box,
            reference_scale,
        } => translated_reference_filter(operation, reference_box, reference_scale, matrix, input),
        PaintFilterReplay::Reflection(_) => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "translated reflection filter requires reference coordinate mapping",
        )),
    }
}

fn translated_reference_filter(
    operation: &layoutng_assembly::internal::paint_input::PaintFilterOperation,
    reference_box: paint::PaintRect,
    reference_scale: f64,
    matrix: &SkM44,
    source: Option<Arc<dyn SkImageFilter>>,
) -> io::Result<Option<Arc<dyn SkImageFilter>>> {
    use raster_commands::filter_graph::{
        RenderSvgBlend, RenderSvgComposite, RenderSvgEdgeMode, RenderSvgFilterKind,
        RenderSvgTransfer,
    };
    use skia::src::core::SkColor::SkColorChannel;
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
        f64::from(scale_x),
        f64::from(scale_y),
        f64::from(matrix.rc(0, 3)),
        f64::from(matrix.rc(1, 3)),
        0.0,
        0.0,
        reference_scale,
    )?;
    let crop = |region: [f32; 4]| {
        SkRect::MakeXYWH(region[0], region[1], region[2].max(0.0), region[3].max(0.0))
    };
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
        .flatten();
    let stroke_paint = graph
        .stroke_paint
        .map(|color| solid(color, crop(graph.region)))
        .transpose()?
        .flatten();
    let convert_space = |input: Option<Arc<dyn SkImageFilter>>, to_linear: bool| {
        let color = if to_linear {
            colorfilters::SkColorSpaceXformColorFilter::SRGBToLinearGamma()
        } else {
            colorfilters::SkColorSpaceXformColorFilter::LinearToSRGBGamma()
        };
        imagefilters::SkColorFilterImageFilter::ColorFilter(Some(color), input, None)
    };
    let transfer_table = |function: &RenderSvgTransfer| -> [u8; 256] {
        std::array::from_fn(|index| {
            let value = index as f32 * (1.0 / 255.0);
            let value = match function {
                RenderSvgTransfer::Identity => value,
                RenderSvgTransfer::Table(values) => {
                    if values.is_empty() {
                        value
                    } else if values.len() == 1 {
                        values[0]
                    } else {
                        let x = value * (values.len() - 1) as f32;
                        let at = x.floor() as usize;
                        values[at]
                            + (values[(at + 1).min(values.len() - 1)] - values[at])
                                * (x - at as f32)
                    }
                }
                RenderSvgTransfer::Discrete(values) => {
                    if values.is_empty() {
                        value
                    } else {
                        values
                            [((value * values.len() as f32).floor() as usize).min(values.len() - 1)]
                    }
                }
                RenderSvgTransfer::Linear { slope, intercept } => slope * value + intercept,
                RenderSvgTransfer::Gamma {
                    amplitude,
                    exponent,
                    offset,
                } => amplitude * value.max(0.0).powf(*exponent) + offset,
            };
            (value.clamp(0.0, 1.0) * 255.0).round_ties_even() as u8
        })
    };
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
    let mut named: std::collections::HashMap<String, Option<Arc<dyn SkImageFilter>>> =
        std::collections::HashMap::new();
    let mut previous: Option<Option<Arc<dyn SkImageFilter>>> = None;
    let lookup = |name: &str,
                  previous: &Option<Option<Arc<dyn SkImageFilter>>>,
                  named: &std::collections::HashMap<String, Option<Arc<dyn SkImageFilter>>>|
     -> io::Result<Option<Arc<dyn SkImageFilter>>> {
        match name {
            "" => Ok(previous.clone().unwrap_or_else(|| source.clone())),
            "SourceGraphic" => Ok(source.clone()),
            "SourceAlpha" => source_alpha(source.clone())
                .map(Some)
                .ok_or_else(|| io::Error::other("translated Skia rejected SVG SourceAlpha filter")),
            // Fixed Chromium resolves these unsupported built-ins through the
            // previous-effect/SourceGraphic fallback in SVGFilterBuilder.
            "BackgroundImage" | "BackgroundAlpha" => {
                Ok(previous.clone().unwrap_or_else(|| source.clone()))
            }
            "FillPaint" => Ok(fill_paint
                .clone()
                .or_else(|| previous.clone().flatten())
                .or_else(|| source.clone())),
            "StrokePaint" => Ok(stroke_paint
                .clone()
                .or_else(|| previous.clone().flatten())
                .or_else(|| source.clone())),
            value => named.get(value).cloned().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("SVG filter input {value:?} has no preceding result"),
                )
            }),
        }
    };
    for primitive in &graph.primitives {
        let mut input = lookup(&primitive.input, &previous, &named)?;
        if primitive.linear_rgb {
            input = convert_space(input, true);
        }
        let primitive_crop = crop(primitive.region);
        let mut output = match &primitive.kind {
            RenderSvgFilterKind::Offset { x, y } => {
                imagefilters::SkMatrixTransformImageFilter::Offset(
                    *x,
                    *y,
                    input,
                    Some(primitive_crop),
                )
            }
            RenderSvgFilterKind::GaussianBlur { x, y, edge_mode } => {
                let tile = match edge_mode {
                    RenderSvgEdgeMode::None => SkTileMode::kDecal,
                    RenderSvgEdgeMode::Duplicate => SkTileMode::kClamp,
                    RenderSvgEdgeMode::Wrap => SkTileMode::kRepeat,
                };
                imagefilters::SkBlurImageFilter::Blur(*x, *y, tile, input, Some(primitive_crop))
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
            RenderSvgFilterKind::Flood(color) => solid(*color, primitive_crop)?,
            RenderSvgFilterKind::Blend { input2, mode } => {
                let mut background = lookup(input2, &previous, &named)?;
                if primitive.linear_rgb {
                    background = convert_space(background, true);
                }
                let mode = match mode {
                    RenderSvgBlend::Normal => SkBlendMode::kSrcOver,
                    RenderSvgBlend::Multiply => SkBlendMode::kMultiply,
                    RenderSvgBlend::Screen => SkBlendMode::kScreen,
                    RenderSvgBlend::Darken => SkBlendMode::kDarken,
                    RenderSvgBlend::Lighten => SkBlendMode::kLighten,
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
                let mut background = lookup(input2, &previous, &named)?;
                if primitive.linear_rgb {
                    background = convert_space(background, true);
                }
                match operator {
                    RenderSvgComposite::Arithmetic => imagefilters::SkBlendImageFilter::Arithmetic(
                        k[0],
                        k[1],
                        k[2],
                        k[3],
                        true,
                        background,
                        input,
                        Some(primitive_crop),
                    ),
                    operator => {
                        let mode = match operator {
                            RenderSvgComposite::Over => SkBlendMode::kSrcOver,
                            RenderSvgComposite::In => SkBlendMode::kSrcIn,
                            RenderSvgComposite::Out => SkBlendMode::kSrcOut,
                            RenderSvgComposite::Atop => SkBlendMode::kSrcATop,
                            RenderSvgComposite::Xor => SkBlendMode::kXor,
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
                            if primitive.linear_rgb {
                                convert_space(input, true)
                            } else {
                                input
                            }
                        })
                    })
                    .collect::<io::Result<Vec<_>>>()?;
                imagefilters::SkMergeImageFilter::Merge(Some(&filters), Some(primitive_crop))
            }
            RenderSvgFilterKind::ComponentTransfer(functions) => {
                let tables = functions.each_ref().map(transfer_table);
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
                        *x as f32,
                        *y as f32,
                        input,
                        Some(primitive_crop),
                    )
                } else {
                    imagefilters::SkMorphologyImageFilter::Erode(
                        *x as f32,
                        *y as f32,
                        input,
                        Some(primitive_crop),
                    )
                }
            }
            RenderSvgFilterKind::Tile => {
                imagefilters::SkCropImageFilter::Tile(primitive_crop, crop(graph.region), input)
            }
            RenderSvgFilterKind::DisplacementMap {
                input2,
                scale,
                x_channel,
                y_channel,
            } => {
                let mut displacement = lookup(input2, &previous, &named)?;
                if primitive.linear_rgb {
                    displacement = convert_space(displacement, true);
                }
                imagefilters::SkDisplacementMapImageFilter::DisplacementMap(
                    channel(*x_channel)?,
                    channel(*y_channel)?,
                    *scale,
                    displacement,
                    input,
                    Some(primitive_crop),
                )
            }
            kind => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    format!("translated SVG filter primitive is not connected: {kind:?}"),
                ));
            }
        };
        if primitive.linear_rgb {
            output = convert_space(output, false);
        }
        if !primitive.result.is_empty() {
            named.insert(primitive.result.clone(), output.clone());
        }
        previous = Some(output);
    }
    let output = previous.unwrap_or(source);
    Ok(skia::src::effects::imagefilters::SkCropImageFilter::Crop(
        crop(graph.region),
        SkTileMode::kDecal,
        output,
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
                native_blend(mode),
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

    // Paint image resources carry unpremultiplied RGBA8. Use the translated
    // original SkConvertPixels path before constructing Skia's immutable image.
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
        let sampling = if item.image_interpolation_quality == ImageInterpolationQuality::kNone {
            SkSamplingOptions::with_filter(SkFilterMode::kNearest)
        } else {
            SkSamplingOptions::with_filter(SkFilterMode::kLinear)
        };
        let mut paint = SkPaint::default();
        paint.setAlphaf(item.flags_alpha);
        paint.setAntiAlias(antialias);
        paint.setBlendMode(blend);
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
            let matrix = matrix.native();
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
            // Keep the native list position: if the declared bottom source is
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
                let mut paint = command_paint(item, false);
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
                    // destination. This is the native SkCanvas equivalent of
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
            CommandKind::kDrawRect | CommandKind::kStrokeRect => unsafe {
                self.target
                    .draw_rect(
                        &rect(item.rect),
                        &command_paint(item, item.r#type == CommandKind::kStrokeRect),
                    )
                    .map_err(|error| raster_error("translated drawRect failed", error))
            },
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
            CommandKind::kDrawEllipse | CommandKind::kStrokeEllipse => unsafe {
                self.target
                    .draw_oval(
                        &rect(item.rect),
                        &command_paint(item, item.r#type == CommandKind::kStrokeEllipse),
                    )
                    .map_err(|error| raster_error("translated drawOval failed", error))
            },
            CommandKind::kDrawPath | CommandKind::kStrokePath => unsafe {
                self.target
                    .draw_path(
                        &path(&item.path, item.even_odd, item.inverse_winding),
                        &command_paint(item, item.r#type == CommandKind::kStrokePath),
                    )
                    .map_err(|error| raster_error("translated drawPath failed", error))
            },
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
                    let sampling =
                        if item.image_interpolation_quality == ImageInterpolationQuality::kNone {
                            SkSamplingOptions::with_filter(SkFilterMode::kNearest)
                        } else {
                            SkSamplingOptions::with_filter(SkFilterMode::kLinear)
                        };
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
                font.setSubpixel(item.font_smoothing == FontSmoothing::kSubpixelAntialiased);
                font.setEdging(match item.font_smoothing {
                    FontSmoothing::kNone => Edging::kAlias,
                    FontSmoothing::kSubpixelAntialiased => Edging::kSubpixelAntiAlias,
                    FontSmoothing::kAuto | FontSmoothing::kAntialiased => Edging::kAntiAlias,
                });
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
                FoldedOp::Borrowed(CpuPaintOpRef::Native { op, record }) => {
                    let mut command = op.to_skia();
                    command.flags_alpha = item.alpha;
                    command.color.alpha *= item.alpha;
                    if let Some(shader) = &mut command.paint_shader {
                        shader.flags_alpha *= item.alpha;
                        for stop in &mut shader.stops {
                            stop.color.alpha *= item.alpha;
                        }
                    }
                    self.execute(&command, &resources(record.artifact()));
                }
                FoldedOp::Borrowed(CpuPaintOpRef::Converted(op)) => match op {
                    CpuPaintOp::NativeAtom(source) => {
                        let mut command = source.op().to_skia();
                        command.flags_alpha = item.alpha;
                        command.color.alpha *= item.alpha;
                        if let Some(shader) = &mut command.paint_shader {
                            shader.flags_alpha *= item.alpha;
                            for stop in &mut shader.stops {
                                stop.color.alpha *= item.alpha;
                            }
                        }
                        self.execute(&command, &resources(source.record().artifact()));
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
                                    native_blend(flags.getBlendMode()),
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
                        let values = transform.native().GetColMajor().map(|value| value as f32);
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
                            match translated_filter_graph(
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
                        let backdrop = match translated_filter_graph(
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
                            translated_filter_graph(filter.as_ref(), &matrix, None)
                                .map(|filter| vec![filter])
                        } else {
                            filters
                                .iter()
                                .map(|filter| {
                                    translated_filter_graph(filter.as_ref(), &matrix, None)
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
                        if let Err(error) = self.target.draw_color(&value, native_blend(*blend)) {
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
            canvas.execute(&item.to_skia(), &command_resources);
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
pub fn RasterizePaintArtifact(list: &PaintArtifact, width: u32, height: u32) -> Vec<u8> {
    rasterize_paint_artifact(list, width, height, 1., false)
        .expect("translated PaintArtifact rasterization failed")
}

/// Rasterize a Paint publication onto an opaque white scaled RGBA target.
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
pub fn RasterizePaintArtifactTransparent(list: &PaintArtifact, width: u32, height: u32) -> Vec<u8> {
    rasterize_paint_artifact(list, width, height, 1., true)
        .expect("translated transparent PaintArtifact rasterization failed")
}

impl RasterBackend for TranslatedSkiaBackend {
    fn Raster(
        &self,
        request: NativeRasterPlayback<'_, '_>,
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
            if source.requires_clear {
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
                    .map_err(|error| raster_error("translated target clear failed", error))?;
            }
            canvas.simple(CommandKind::kSave);
            canvas.target.translate(
                -request.content_rect.x as f32,
                -request.content_rect.y as f32,
            );
            canvas.target.clip_rect(
                &cc_rect(*request.invalid_content_rect),
                SkClipOp::kIntersect,
                false,
            );
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
                        native_family: "Roboto",
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
                    glyphs: std::sync::Arc::new(vec![command::PaintGlyph {
                        id: 37,
                        ..Default::default()
                    }]),
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
}
