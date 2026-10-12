//! On-demand CoreText discovery at the browser's host boundary.
//!
//! Only descriptors and their metadata cross this adapter. It does not shape,
//! rasterize, call FontEngine, enumerate the system catalog, or open font files.
//! Apple documents mandatory descriptor keys as exact-match constraints:
//! https://developer.apple.com/documentation/coretext/ctfontdescriptorcreatematchingfontdescriptors(_:_:)
//! Contextual fallback uses the selected font, UTF-16 text, and BCP-47 language:
//! https://developer.apple.com/documentation/coretext/ctfontcreateforstringwithlanguage(_:_:_:_:)
//! Native trait and emoji behavior follows Chromium's macOS host implementation:
//! https://chromium.googlesource.com/chromium/src/+/main/third_party/blink/renderer/platform/fonts/mac/font_cache_mac.mm
#![allow(non_snake_case)]

use super::HostFace;
use font_engine::{FontPresentation, FontVariation, SystemFontQuery};
use std::ffi::c_void;
use std::ptr;

type Ref = *const c_void;
#[repr(C)]
#[derive(Clone, Copy)]
struct Range {
    location: isize,
    length: isize,
}

#[cfg_attr(target_os = "macos", link(name = "CoreFoundation", kind = "framework"))]
extern "C" {
    fn CFRelease(value: Ref);
    fn CFGetTypeID(value: Ref) -> usize;
    fn CFStringGetTypeID() -> usize;
    fn CFNumberGetTypeID() -> usize;
    fn CFArrayGetTypeID() -> usize;
    fn CFDictionaryGetTypeID() -> usize;
    fn CFURLGetTypeID() -> usize;
    #[cfg(target_os = "macos")]
    fn CFDataGetLength(data: Ref) -> isize;
    #[cfg(target_os = "macos")]
    fn CFDataGetBytePtr(data: Ref) -> *const u8;
    fn CFCharacterSetIsLongCharacterMember(set: Ref, character: u32) -> bool;
    fn CFStringCreateWithCharacters(allocator: Ref, chars: *const u16, count: isize) -> Ref;
    fn CFStringGetLength(string: Ref) -> isize;
    fn CFStringGetCharacters(string: Ref, range: Range, buffer: *mut u16);
    fn CFNumberCreate(allocator: Ref, kind: isize, value: Ref) -> Ref;
    fn CFNumberGetValue(number: Ref, kind: isize, value: *mut c_void) -> bool;
    fn CFDictionaryCreate(
        allocator: Ref,
        keys: *const Ref,
        values: *const Ref,
        count: isize,
        key_callbacks: Ref,
        value_callbacks: Ref,
    ) -> Ref;
    fn CFDictionaryGetValue(dictionary: Ref, key: Ref) -> Ref;
    fn CFDictionaryGetCount(dictionary: Ref) -> isize;
    fn CFDictionaryGetKeysAndValues(dictionary: Ref, keys: *mut Ref, values: *mut Ref);
    fn CFArrayCreate(allocator: Ref, values: *const Ref, count: isize, callbacks: Ref) -> Ref;
    fn CFArrayGetCount(array: Ref) -> isize;
    fn CFArrayGetValueAtIndex(array: Ref, index: isize) -> Ref;
    fn CFSetCreate(allocator: Ref, values: *const Ref, count: isize, callbacks: Ref) -> Ref;
    fn CFURLCopyFileSystemPath(url: Ref, style: isize) -> Ref;
    fn CFURLCreateFromFileSystemRepresentation(
        allocator: Ref,
        bytes: *const u8,
        length: isize,
        is_directory: bool,
    ) -> Ref;
    static kCFTypeDictionaryKeyCallBacks: u8;
    static kCFTypeDictionaryValueCallBacks: u8;
    static kCFTypeArrayCallBacks: u8;
    static kCFTypeSetCallBacks: u8;
}

#[cfg_attr(target_os = "macos", link(name = "CoreText", kind = "framework"))]
extern "C" {
    fn CTFontDescriptorCreateWithAttributes(attributes: Ref) -> Ref;
    fn CTFontDescriptorCreateMatchingFontDescriptors(descriptor: Ref, mandatory: Ref) -> Ref;
    fn CTFontCreateWithFontDescriptor(descriptor: Ref, size: f64, matrix: Ref) -> Ref;
    fn CTFontCreateUIFontForLanguage(ui_type: u32, size: f64, language: Ref) -> Ref;
    fn CTFontCreateForStringWithLanguage(
        font: Ref,
        string: Ref,
        range: Range,
        language: Ref,
    ) -> Ref;
    fn CTFontCopyAttribute(font: Ref, attribute: Ref) -> Ref;
    fn CTFontCopyFamilyName(font: Ref) -> Ref;
    fn CTFontCopyFullName(font: Ref) -> Ref;
    fn CTFontCopyPostScriptName(font: Ref) -> Ref;
    fn CTFontCopySupportedLanguages(font: Ref) -> Ref;
    fn CTFontCopyTraits(font: Ref) -> Ref;
    fn CTFontCopyCharacterSet(font: Ref) -> Ref;
    #[cfg(target_os = "macos")]
    fn CTFontCopyTable(font: Ref, tag: u32, options: u32) -> Ref;
    fn CTFontCopyDefaultCascadeListForLanguages(font: Ref, languages: Ref) -> Ref;
    fn CTFontGetSymbolicTraits(font: Ref) -> u32;
    fn CTFontCopyVariation(font: Ref) -> Ref;
    static kCTFontFamilyNameAttribute: Ref;
    static kCTFontNameAttribute: Ref;
    static kCTFontDisplayNameAttribute: Ref;
    static kCTFontURLAttribute: Ref;
    static kCTFontTraitsAttribute: Ref;
    static kCTFontLanguagesAttribute: Ref;
    static kCTFontWeightTrait: Ref;
    static kCTFontSymbolicTrait: Ref;
    static kCTFontVariationAttribute: Ref;
    static kCTFontCascadeListAttribute: Ref;
}

/// Owns a Create/Copy result. Borrowed values never enter this wrapper.
struct Owned(Ref);
impl Owned {
    unsafe fn new(value: Ref) -> Option<Self> {
        (!value.is_null()).then(|| Self(value))
    }
    fn get(&self) -> Ref {
        self.0
    }
}
impl Drop for Owned {
    fn drop(&mut self) {
        unsafe { CFRelease(self.0) }
    }
}

unsafe fn String16(chars: &[u16]) -> Option<Owned> {
    Owned::new(CFStringCreateWithCharacters(
        ptr::null(),
        chars.as_ptr(),
        chars.len().try_into().ok()?,
    ))
}
unsafe fn StringRef(value: &str) -> Option<Owned> {
    String16(&value.encode_utf16().collect::<Vec<_>>())
}
unsafe fn ReadString(value: Ref) -> Option<String> {
    if value.is_null() || CFGetTypeID(value) != CFStringGetTypeID() {
        return None;
    }
    let length = CFStringGetLength(value);
    let mut chars = vec![0; usize::try_from(length).ok()?];
    CFStringGetCharacters(
        value,
        Range {
            location: 0,
            length,
        },
        chars.as_mut_ptr(),
    );
    String::from_utf16(&chars).ok()
}
unsafe fn ReadNumber(value: Ref) -> Option<f64> {
    if value.is_null() || CFGetTypeID(value) != CFNumberGetTypeID() {
        return None;
    }
    let mut result = 0.0f64;
    (CFNumberGetValue(
        value,
        6, /* kCFNumberFloat64Type */
        ptr::from_mut(&mut result).cast(),
    ) && result.is_finite())
    .then_some(result)
}
unsafe fn Number(value: f64) -> Option<Owned> {
    Owned::new(CFNumberCreate(ptr::null(), 6, ptr::from_ref(&value).cast()))
}
unsafe fn Dictionary(entries: &[(Ref, Ref)]) -> Option<Owned> {
    let keys: Vec<_> = entries.iter().map(|(key, _)| *key).collect();
    let values: Vec<_> = entries.iter().map(|(_, value)| *value).collect();
    Owned::new(CFDictionaryCreate(
        ptr::null(),
        keys.as_ptr(),
        values.as_ptr(),
        entries.len() as isize,
        ptr::addr_of!(kCFTypeDictionaryKeyCallBacks).cast(),
        ptr::addr_of!(kCFTypeDictionaryValueCallBacks).cast(),
    ))
}
unsafe fn Array(values: &[Ref]) -> Option<Owned> {
    Owned::new(CFArrayCreate(
        ptr::null(),
        values.as_ptr(),
        values.len() as isize,
        ptr::addr_of!(kCFTypeArrayCallBacks).cast(),
    ))
}
unsafe fn ReadArray(value: Ref) -> Vec<Ref> {
    if value.is_null() || CFGetTypeID(value) != CFArrayGetTypeID() {
        return Vec::new();
    }
    (0..CFArrayGetCount(value))
        .map(|i| CFArrayGetValueAtIndex(value, i))
        .collect()
}

// CoreText's weight scale is non-linear. The standard NSFont/CoreText weights
// map to the corresponding CSS classes; interpolate for variable-font values.
const WEIGHTS: [(f64, f64); 9] = [
    (100.0, -0.8),
    (200.0, -0.6),
    (300.0, -0.4),
    (400.0, 0.0),
    (500.0, 0.23),
    (600.0, 0.3),
    (700.0, 0.4),
    (800.0, 0.56),
    (900.0, 0.62),
];
fn Interpolate(value: f64, from_css: bool) -> f64 {
    let points = WEIGHTS.map(|(css, native)| {
        if from_css {
            (css, native)
        } else {
            (native, css)
        }
    });
    let value = if value.is_finite() {
        value
    } else if from_css {
        400.0
    } else {
        0.0
    };
    if value <= points[0].0 {
        return points[0].1;
    }
    for pair in points.windows(2) {
        if value <= pair[1].0 {
            let t = (value - pair[0].0) / (pair[1].0 - pair[0].0);
            return pair[0].1 + t * (pair[1].1 - pair[0].1);
        }
    }
    points[8].1
}

unsafe fn Variations(font: Ref) -> Vec<FontVariation> {
    let Some(variation) = Owned::new(CTFontCopyVariation(font)) else {
        return Vec::new();
    };
    if CFGetTypeID(variation.get()) != CFDictionaryGetTypeID() {
        return Vec::new();
    }
    let count = CFDictionaryGetCount(variation.get()).max(0) as usize;
    let mut keys = vec![ptr::null(); count];
    let mut values = vec![ptr::null(); count];
    CFDictionaryGetKeysAndValues(variation.get(), keys.as_mut_ptr(), values.as_mut_ptr());
    let mut result: Vec<_> = keys
        .into_iter()
        .zip(values)
        .filter_map(|(key, value)| {
            let tag = ReadNumber(key)?;
            let value = ReadNumber(value)? as f32;
            (tag >= 0.0 && tag <= u32::MAX as f64 && value.is_finite()).then_some(FontVariation {
                tag: tag as u32,
                value,
            })
        })
        .collect();
    result.sort_by_key(|axis| axis.tag);
    result
}

unsafe fn Face(font: Ref) -> Option<HostFace> {
    let family = Owned::new(CTFontCopyFamilyName(font))?;
    let family = ReadString(family.get())?;
    if family == "LastResort" || family == ".LastResort" {
        return None;
    }
    let name = Owned::new(CTFontCopyPostScriptName(font))?;
    let unique_name = ReadString(name.get())?;
    let url = Owned::new(CTFontCopyAttribute(font, kCTFontURLAttribute))?;
    if CFGetTypeID(url.get()) != CFURLGetTypeID() {
        return None;
    }
    let path = Owned::new(CFURLCopyFileSystemPath(
        url.get(),
        0, /* kCFURLPOSIXPathStyle */
    ))?;
    let path = ReadString(path.get())?;
    if !std::path::Path::new(&path).is_absolute() {
        return None;
    }
    let symbolic = CTFontGetSymbolicTraits(font);
    let weight = Owned::new(CTFontCopyTraits(font))
        .and_then(|traits| ReadNumber(CFDictionaryGetValue(traits.get(), kCTFontWeightTrait)))
        .map(|weight| Interpolate(weight, false))
        .unwrap_or(400.0);
    let locales = Owned::new(CTFontCopySupportedLanguages(font))
        .map(|languages| {
            ReadArray(languages.get())
                .into_iter()
                .filter_map(|value| ReadString(value))
                .collect()
        })
        .unwrap_or_default();
    let variations = Variations(font);
    let weight = variations
        .iter()
        .find(|axis| axis.tag == u32::from_be_bytes(*b"wght"))
        .map_or(weight, |axis| f64::from(axis.value));
    Some(HostFace {
        family,
        unique_name,
        path,
        index: 0,
        weight,
        italic: symbolic & 1 != 0,
        locales,
        color: symbolic & (1 << 13) != 0,
        variations,
    })
}

unsafe fn Matching(
    key: Ref,
    value: &str,
    weight: f64,
    italic: bool,
    locale: &str,
) -> Vec<HostFace> {
    let Some(value) = StringRef(value) else {
        return Vec::new();
    };
    let Some(weight) = Number(Interpolate(weight, true)) else {
        return Vec::new();
    };
    let Some(symbolic) = Number(if italic { 1.0 } else { 0.0 }) else {
        return Vec::new();
    };
    let Some(traits) = Dictionary(&[
        (kCTFontWeightTrait, weight.get()),
        (kCTFontSymbolicTrait, symbolic.get()),
    ]) else {
        return Vec::new();
    };
    let language = (!locale.is_empty()).then(|| StringRef(locale)).flatten();
    let languages = language
        .as_ref()
        .and_then(|language| Array(&[language.get()]));
    let mut attrs = vec![(key, value.get()), (kCTFontTraitsAttribute, traits.get())];
    if let Some(languages) = &languages {
        attrs.push((kCTFontLanguagesAttribute, languages.get()));
    }
    let Some(attrs) = Dictionary(&attrs) else {
        return Vec::new();
    };
    let Some(descriptor) = Owned::new(CTFontDescriptorCreateWithAttributes(attrs.get())) else {
        return Vec::new();
    };
    let Some(mandatory) = Owned::new(CFSetCreate(
        ptr::null(),
        &key,
        1,
        ptr::addr_of!(kCFTypeSetCallBacks).cast(),
    )) else {
        return Vec::new();
    };
    let Some(matches) = Owned::new(CTFontDescriptorCreateMatchingFontDescriptors(
        descriptor.get(),
        mandatory.get(),
    )) else {
        return Vec::new();
    };
    ReadArray(matches.get())
        .into_iter()
        .filter_map(|descriptor| {
            let font = Owned::new(CTFontCreateWithFontDescriptor(
                descriptor,
                12.0,
                ptr::null(),
            ))?;
            Face(font.get())
        })
        .collect()
}

unsafe fn FontForFace(face: &HostFace) -> Option<Owned> {
    let name = StringRef(&face.unique_name)?;
    let path = face.path.as_bytes();
    let url = Owned::new(CFURLCreateFromFileSystemRepresentation(
        ptr::null(),
        path.as_ptr(),
        path.len().try_into().ok()?,
        false,
    ))?;
    let mut numbers = Vec::new();
    for axis in &face.variations {
        numbers.push((Number(f64::from(axis.tag))?, Number(f64::from(axis.value))?));
    }
    let variations = Dictionary(
        &numbers
            .iter()
            .map(|(key, value)| (key.get(), value.get()))
            .collect::<Vec<_>>(),
    )?;
    let attrs = Dictionary(&[
        (kCTFontNameAttribute, name.get()),
        (kCTFontURLAttribute, url.get()),
        (kCTFontVariationAttribute, variations.get()),
    ])?;
    let descriptor = Owned::new(CTFontDescriptorCreateWithAttributes(attrs.get()))?;
    let font = Owned::new(CTFontCreateWithFontDescriptor(
        descriptor.get(),
        12.0,
        ptr::null(),
    ))?;
    let actual = Owned::new(CTFontCopyPostScriptName(font.get()))?;
    SameName(&ReadString(actual.get())?, &face.unique_name).then_some(font)
}

fn SameName(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

unsafe fn Unique(name: &str) -> Vec<HostFace> {
    let mut result = Vec::new();
    for key in [kCTFontNameAttribute, kCTFontDisplayNameAttribute] {
        for face in Matching(key, name, 400.0, false, "") {
            let Some(font) = FontForFace(&face) else {
                continue;
            };
            let full =
                Owned::new(CTFontCopyFullName(font.get())).and_then(|name| ReadString(name.get()));
            // CoreText can substitute an available font for an unknown name.
            // CSS local() must never accept that substitution.
            if (SameName(&face.unique_name, name) || full.is_some_and(|full| SameName(&full, name)))
                && !result.iter().any(|other: &HostFace| {
                    other.unique_name == face.unique_name && other.path == face.path
                })
            {
                result.push(face);
            }
        }
    }
    result
}

unsafe fn Generic(generic: &str, weight: f64, italic: bool, locale: &str) -> Vec<HostFace> {
    let generic = generic.to_ascii_lowercase();
    // These are the macOS browser preference defaults, not an enumerated or
    // eagerly opened catalog. Each request still asks the installed font manager.
    // Chromium: chrome/app/resources/locale_settings_mac.grd.
    let family = match generic.as_str() {
        "serif" | "ui-serif" => "Times",
        "sans-serif" => "Helvetica",
        "monospace" => "Menlo",
        "cursive" => "Apple Chancery",
        "fantasy" => "Papyrus",
        "emoji" => "Apple Color Emoji",
        "math" => "STIX Two Math",
        "system-ui" | "ui-sans-serif" | "-apple-system" | "blinkmacsystemfont" | "ui-monospace" => {
            let language = StringRef(locale);
            let font = Owned::new(CTFontCreateUIFontForLanguage(
                if generic == "ui-monospace" {
                    1 /* UserFixedPitch */
                } else {
                    2 /* System */
                },
                12.0,
                language.as_ref().map_or(ptr::null(), Owned::get),
            ));
            let Some(font) = font else {
                return Vec::new();
            };
            let Some(face) = Face(font.get()) else {
                return Vec::new();
            };
            let matches = Matching(
                kCTFontFamilyNameAttribute,
                &face.family,
                weight,
                italic,
                locale,
            );
            return if matches.is_empty() {
                vec![face]
            } else {
                matches
            };
        }
        _ => return Vec::new(),
    };
    Matching(kCTFontFamilyNameAttribute, family, weight, italic, locale)
}

fn PresentationHint(text: &[u16], presentation: FontPresentation) -> Vec<u16> {
    let mut hint = text.to_vec();
    if hint.is_empty() {
        return hint;
    }
    let first_length = if (0xd800..=0xdbff).contains(&hint[0])
        && hint
            .get(1)
            .is_some_and(|value| (0xdc00..=0xdfff).contains(value))
    {
        2
    } else {
        1
    };
    let selector = match presentation {
        FontPresentation::Text => 0xfe0e,
        FontPresentation::Emoji => 0xfe0f,
    };
    if hint
        .get(first_length)
        .is_some_and(|value| matches!(*value, 0xfe0e | 0xfe0f))
    {
        hint[first_length] = selector;
    } else {
        hint.insert(first_length, selector);
    }
    hint
}

unsafe fn CoversFirstCharacter(font: Ref, text: &[u16]) -> bool {
    let Some(Ok(character)) = char::decode_utf16(text.iter().copied()).next() else {
        return false;
    };
    Owned::new(CTFontCopyCharacterSet(font))
        .is_some_and(|set| CFCharacterSetIsLongCharacterMember(set.get(), character as u32))
}

unsafe fn StyledFallback(
    face: HostFace,
    text: &[u16],
    weight: f64,
    italic: bool,
    locale: &str,
) -> HostFace {
    if (face.weight - weight).abs() < 0.01 && face.italic == italic {
        return face;
    }
    // Blink's GetAlternateFontPlatformData retries the substitute family with
    // the requested traits and only accepts it if it still covers the character.
    let Some(family) = StringRef(&face.family) else {
        return face;
    };
    let Some(weight) = Number(Interpolate(weight, true)) else {
        return face;
    };
    let Some(symbolic) = Number(if italic { 1.0 } else { 0.0 }) else {
        return face;
    };
    let Some(traits) = Dictionary(&[
        (kCTFontWeightTrait, weight.get()),
        (kCTFontSymbolicTrait, symbolic.get()),
    ]) else {
        return face;
    };
    let language = (!locale.is_empty()).then(|| StringRef(locale)).flatten();
    let languages = language.as_ref().and_then(|value| Array(&[value.get()]));
    let mut attrs = vec![
        (kCTFontFamilyNameAttribute, family.get()),
        (kCTFontTraitsAttribute, traits.get()),
    ];
    if let Some(languages) = &languages {
        attrs.push((kCTFontLanguagesAttribute, languages.get()));
    }
    let Some(attrs) = Dictionary(&attrs) else {
        return face;
    };
    let Some(descriptor) = Owned::new(CTFontDescriptorCreateWithAttributes(attrs.get())) else {
        return face;
    };
    let Some(font) = Owned::new(CTFontCreateWithFontDescriptor(
        descriptor.get(),
        12.0,
        ptr::null(),
    )) else {
        return face;
    };
    if CoversFirstCharacter(font.get(), text) {
        if let Some(styled) = Face(font.get()) {
            if SameName(&styled.family, &face.family) {
                return styled;
            }
        }
    }
    face
}

unsafe fn Fallback(
    text: &[u16],
    weight: f64,
    italic: bool,
    locale: &str,
    presentation: FontPresentation,
    base: Option<&HostFace>,
) -> Vec<HostFace> {
    if text.is_empty() {
        return Vec::new();
    }
    let base_font = base.and_then(|face| FontForFace(face)).or_else(|| {
        // Blink uses the configured standard (serif) family when the base is
        // a web font with no native descriptor.
        let mut faces = Generic("serif", weight, italic, locale);
        // Prefer actual traits without synthesizing a native font instance.
        faces.sort_by(|a, b| {
            (a.italic != italic)
                .cmp(&(b.italic != italic))
                .then_with(|| {
                    (a.weight - weight)
                        .abs()
                        .total_cmp(&(b.weight - weight).abs())
                })
        });
        faces.first().and_then(|face| FontForFace(face))
    });
    let Some(base_font) = base_font else {
        return Vec::new();
    };
    let hint = PresentationHint(text, presentation);
    let Some(string) = String16(&hint) else {
        return Vec::new();
    };
    let language = (!locale.is_empty()).then(|| StringRef(locale)).flatten();
    let Some(font) = Owned::new(CTFontCreateForStringWithLanguage(
        base_font.get(),
        string.get(),
        Range {
            location: 0,
            length: hint.len() as isize,
        },
        language.as_ref().map_or(ptr::null(), Owned::get),
    )) else {
        return Vec::new();
    };
    let Some(mut face) = Face(font.get()) else {
        return Vec::new();
    };
    if matches!(presentation, FontPresentation::Emoji) && !face.color {
        // Blink explicitly prefers Apple Color Emoji for emoji presentation.
        // Coverage checking avoids selecting it for an unrelated hint character.
        for emoji in Generic("emoji", weight, italic, locale) {
            if let Some(font) = FontForFace(&emoji) {
                if emoji.color && CoversFirstCharacter(font.get(), text) {
                    return vec![emoji];
                }
            }
        }
    } else if matches!(presentation, FontPresentation::Text) && face.color {
        // Blink's GetSubstituteFont starts monochrome emoji in Apple Symbols
        // while retaining the color font's broader native fallback cascade.
        let languages = language.as_ref().and_then(|value| Array(&[value.get()]));
        let cascade = Owned::new(CTFontCopyDefaultCascadeListForLanguages(
            font.get(),
            languages.as_ref().map_or(ptr::null(), Owned::get),
        ));
        let name = StringRef("Apple Symbols");
        if let (Some(cascade), Some(name)) = (cascade, name) {
            if let Some(attrs) = Dictionary(&[
                (kCTFontNameAttribute, name.get()),
                (kCTFontCascadeListAttribute, cascade.get()),
            ]) {
                if let Some(descriptor) =
                    Owned::new(CTFontDescriptorCreateWithAttributes(attrs.get()))
                {
                    if let Some(mono) = Owned::new(CTFontCreateWithFontDescriptor(
                        descriptor.get(),
                        12.0,
                        ptr::null(),
                    )) {
                        if let Some(replacement) = Owned::new(CTFontCreateForStringWithLanguage(
                            mono.get(),
                            string.get(),
                            Range {
                                location: 0,
                                length: hint.len() as isize,
                            },
                            language.as_ref().map_or(ptr::null(), Owned::get),
                        )) {
                            if let Some(replacement) = Face(replacement.get()) {
                                face = replacement;
                            }
                        }
                    }
                }
            }
        }
    }
    vec![StyledFallback(face, text, weight, italic, locale)]
}

pub(super) fn Query(query: &SystemFontQuery, base: Option<&HostFace>) -> Vec<HostFace> {
    // Every CF object stays on this stack; no native pointer escapes the adapter.
    unsafe {
        match query {
            SystemFontQuery::MatchFamily {
                family,
                weight,
                italic,
                locale,
            } => Matching(kCTFontFamilyNameAttribute, family, *weight, *italic, locale),
            SystemFontQuery::MatchUniqueName { name } => Unique(name),
            SystemFontQuery::MatchGeneric {
                generic,
                weight,
                italic,
                locale,
            } => Generic(generic, *weight, *italic, locale),
            SystemFontQuery::Fallback {
                text,
                weight,
                italic,
                locale,
                presentation,
                ..
            } => Fallback(text, *weight, *italic, locale, *presentation, base),
            SystemFontQuery::OpenFace { .. } => Vec::new(),
        }
    }
}

/// Resolve the selected PostScript face only after the host opens its file.
/// CoreText's public URL/name descriptors do not carry a documented TTC index.
/// Never guess index zero for a collection: compare the SFNT name records.
/// This is file identity parsing, not shaping or a FontEngine backend call.
pub(super) fn CollectionIndex(face: &HostFace, bytes: &[u8]) -> Option<u32> {
    if bytes.get(..4)? != b"ttcf" {
        // A standalone SFNT has exactly one face. Its instantiated variable
        // PostScript name can differ from name ID 6; the descriptor URL plus
        // recorded variation coordinates already establish its identity.
        return matches!(bytes.get(..4)?, b"\0\x01\0\0" | b"OTTO" | b"true").then_some(0);
    }
    let count = ReadU32(bytes, 8)?;
    let offsets = bytes.get(12..12usize.checked_add((count as usize).checked_mul(4)?)?)?;
    let matched = offsets
        .chunks_exact(4)
        .enumerate()
        .find_map(|(index, offset)| {
            let offset = u32::from_be_bytes(offset.try_into().ok()?) as usize;
            HasPostScriptName(bytes, offset, &face.unique_name).then_some(index as u32)
        });
    if matched.is_some() {
        return matched;
    }
    // CoreText may synthesize a variable instance's PostScript name. At open
    // time only, compare that native face's unmodified identity tables against
    // the collection. Ambiguous or missing identity is rejected, never guessed.
    #[cfg(target_os = "macos")]
    {
        unsafe { NativeCollectionIndex(face, bytes, offsets) }
    }
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}
#[cfg(target_os = "macos")]
unsafe fn NativeCollectionIndex(face: &HostFace, bytes: &[u8], offsets: &[u8]) -> Option<u32> {
    let font = FontForFace(face)?;
    let mut tables = Vec::new();
    for tag in [*b"name", *b"head"] {
        let data = Owned::new(CTFontCopyTable(font.get(), u32::from_be_bytes(tag), 0))?;
        let length = usize::try_from(CFDataGetLength(data.get())).ok()?;
        let pointer = CFDataGetBytePtr(data.get());
        if length == 0 || pointer.is_null() {
            return None;
        }
        // The owned CFData remains alive until the comparison is finished.
        tables.push((tag, data, pointer, length));
    }
    let mut matches = offsets
        .chunks_exact(4)
        .enumerate()
        .filter_map(|(index, offset)| {
            let offset = u32::from_be_bytes(offset.try_into().ok()?) as usize;
            tables
                .iter()
                .all(|(tag, _, pointer, length)| {
                    let native = std::slice::from_raw_parts(*pointer, *length);
                    SfntTable(bytes, offset, tag) == Some(native)
                })
                .then_some(index as u32)
        });
    let first = matches.next()?;
    matches.next().is_none().then_some(first)
}

fn SfntTable<'a>(bytes: &'a [u8], offset: usize, tag: &[u8; 4]) -> Option<&'a [u8]> {
    let count = ReadU16(bytes, offset.checked_add(4)?)?;
    let start = offset.checked_add(12)?;
    let records = bytes.get(start..start.checked_add(usize::from(count) * 16)?)?;
    records.chunks_exact(16).find_map(|record| {
        if &record[..4] != tag {
            return None;
        }
        let start = ReadU32(record, 8)? as usize;
        let length = ReadU32(record, 12)? as usize;
        bytes.get(start..start.checked_add(length)?)
    })
}

fn ReadU16(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_be_bytes(
        bytes.get(offset..offset.checked_add(2)?)?.try_into().ok()?,
    ))
}
fn ReadU32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_be_bytes(
        bytes.get(offset..offset.checked_add(4)?)?.try_into().ok()?,
    ))
}
fn HasPostScriptName(bytes: &[u8], offset: usize, name: &str) -> bool {
    let Some(table) = SfntTable(bytes, offset, b"name") else {
        return false;
    };
    let Some(count) = ReadU16(table, 2) else {
        return false;
    };
    let Some(storage) = ReadU16(table, 4) else {
        return false;
    };
    let Some(records) = table.get(6..6 + usize::from(count) * 12) else {
        return false;
    };
    records.chunks_exact(12).any(|record| {
        if ReadU16(record, 6) != Some(6) {
            return false;
        }
        let platform = ReadU16(record, 0).unwrap();
        let encoding = ReadU16(record, 2).unwrap();
        let start = usize::from(storage) + usize::from(ReadU16(record, 10).unwrap());
        let end = start + usize::from(ReadU16(record, 8).unwrap());
        let Some(value) = table.get(start..end) else {
            return false;
        };
        if platform == 0 || (platform == 3 && matches!(encoding, 0 | 1 | 10)) {
            if value.len() % 2 != 0 {
                return false;
            }
            let chars: Vec<_> = value
                .chunks_exact(2)
                .map(|pair| u16::from_be_bytes([pair[0], pair[1]]))
                .collect();
            String::from_utf16(&chars).is_ok_and(|value| value == name)
        } else if platform == 1 && encoding == 0 {
            // PostScript names use the ASCII subset of the Mac Roman encoding.
            value == name.as_bytes()
        } else {
            false
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn FaceNamed(name: &str) -> HostFace {
        HostFace {
            family: "Fixture".into(),
            unique_name: name.into(),
            path: "/fixture.ttc".into(),
            index: 0,
            weight: 400.0,
            italic: false,
            locales: Vec::new(),
            color: false,
            variations: Vec::new(),
        }
    }
    fn Fixture(names: &[&str]) -> Vec<u8> {
        let mut bytes = vec![0; 12 + names.len() * 4];
        bytes[..4].copy_from_slice(b"ttcf");
        bytes[4..8].copy_from_slice(&0x00010000u32.to_be_bytes());
        bytes[8..12].copy_from_slice(&(names.len() as u32).to_be_bytes());
        for (i, name) in names.iter().enumerate() {
            let offset = bytes.len();
            bytes[12 + i * 4..16 + i * 4].copy_from_slice(&(offset as u32).to_be_bytes());
            bytes.extend_from_slice(&[0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0]);
            bytes.extend_from_slice(b"name");
            bytes.extend_from_slice(&0u32.to_be_bytes());
            bytes.extend_from_slice(&((offset + 28) as u32).to_be_bytes());
            let name: Vec<_> = name.encode_utf16().flat_map(u16::to_be_bytes).collect();
            bytes.extend_from_slice(&((18 + name.len()) as u32).to_be_bytes());
            for value in [0u16, 1, 18, 3, 1, 0x409, 6, name.len() as u16, 0] {
                bytes.extend_from_slice(&value.to_be_bytes());
            }
            bytes.extend_from_slice(&name);
        }
        bytes
    }

    #[test]
    fn collection_open_preserves_nonzero_face_identity() {
        let bytes = Fixture(&["Fixture-Regular", "Fixture-Bold", "Fixture-Italic"]);
        assert_eq!(CollectionIndex(&FaceNamed("Fixture-Bold"), &bytes), Some(1));
        assert_eq!(
            CollectionIndex(&FaceNamed("Fixture-Italic"), &bytes),
            Some(2)
        );
        assert_eq!(CollectionIndex(&FaceNamed("Missing"), &bytes), None);
    }
    #[test]
    fn truncated_collection_does_not_substitute_face_zero() {
        let bytes = Fixture(&["Fixture-Regular", "Fixture-Bold"]);
        for end in 0..bytes.len() {
            assert_eq!(
                CollectionIndex(&FaceNamed("Fixture-Bold"), &bytes[..end]),
                None
            );
        }
    }
    #[test]
    fn standalone_variable_instance_preserves_index_zero() {
        // Instantiated native names need not equal the base font's name ID 6.
        assert_eq!(
            CollectionIndex(&FaceNamed("Generated-Instance"), b"OTTO"),
            Some(0)
        );
        assert_eq!(
            CollectionIndex(&FaceNamed("Generated-Instance"), b"bad!"),
            None
        );
    }
    #[test]
    fn invalid_collection_table_offsets_are_rejected() {
        let mut bytes = Fixture(&["Fixture-Regular"]);
        bytes[12..16].copy_from_slice(&u32::MAX.to_be_bytes());
        assert_eq!(CollectionIndex(&FaceNamed("Fixture-Regular"), &bytes), None);
    }
    #[test]
    fn presentation_hint_keeps_surrogate_pair_and_cluster_tail() {
        let text: Vec<_> = "\u{1f469}\u{200d}\u{1f4bb}".encode_utf16().collect();
        let result = PresentationHint(&text, FontPresentation::Emoji);
        assert_eq!(&result[..2], &text[..2]);
        assert_eq!(result[2], 0xfe0f);
        assert_eq!(&result[3..], &text[2..]);
        assert_eq!(
            PresentationHint(&[0x2764, 0xfe0f], FontPresentation::Text),
            [0x2764, 0xfe0e]
        );
    }
    #[test]
    fn css_weight_uses_coretext_non_linear_scale() {
        assert_eq!(Interpolate(400.0, true), 0.0);
        assert_eq!(Interpolate(700.0, true), 0.4);
        for (css, native) in WEIGHTS {
            assert!((Interpolate(native, false) - css).abs() < 0.001);
        }
    }
    #[cfg(target_os = "macos")]
    #[test]
    fn unknown_unique_name_is_not_a_substitute() {
        assert!(Query(
            &SystemFontQuery::MatchUniqueName {
                name: "__Rechrom_missing_face_5fe8bfe1__".into()
            },
            None
        )
        .is_empty());
    }
}
