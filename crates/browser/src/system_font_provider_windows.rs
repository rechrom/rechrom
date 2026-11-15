//! DirectWrite host requests, following Skia's src/ports/SkFontMgr_win_dw.cpp
//! (Skia cacf77bdba7b) family matching and IDWriteFontFallback::MapCharacters
//! fallback as Chromium's FontCache::GetDWriteFallbackFamily uses it
//! (font_cache_skia_win.cc, Chromium 6c1d401f). Only platform metadata is
//! queried here: the installed file and face index are opened separately by
//! OpenFace and rendered by the Fontations typeface.
//!
//! The system menu/small-caption/status fonts are read as ui/gfx/win/
//! system_fonts_win.cc does and stored in FontCache's Windows statics, which
//! WebViewImpl::SetRendererPreferences fills in Chromium.
//!
//! Like the Fontconfig and CoreText hosts, the system interfaces are declared
//! here directly; the vtable slots follow the Windows SDK dwrite*.h headers.

#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]

use super::HostFace;
use font_engine::{FontPresentation, FontVariation, SystemFontQuery};
use std::ffi::c_void;
use std::sync::atomic::{AtomicU32, Ordering};

type HRESULT = i32;
type BOOL = i32;
type PCWSTR = *const u16;
const S_OK: HRESULT = 0;
const E_FAIL: HRESULT = 0x8000_4005_u32 as i32;

#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
struct GUID(u32, u16, u16, [u8; 8]);
const IID_IUnknown: GUID = GUID(0, 0, 0, [0xc0, 0, 0, 0, 0, 0, 0, 0x46]);
const IID_IDWriteFactory2: GUID = GUID(
    0x0439fc60,
    0xca44,
    0x4994,
    [0x8d, 0xee, 0x3a, 0x9a, 0xf7, 0xb7, 0x32, 0xec],
);
const IID_IDWriteFont2: GUID = GUID(
    0x29748ed6,
    0x8c9c,
    0x4a6a,
    [0xbe, 0x0b, 0xd9, 0x12, 0xe8, 0x53, 0x89, 0x44],
);
const IID_IDWriteFontFace5: GUID = GUID(
    0x98eff3a5,
    0xb667,
    0x479a,
    [0xb1, 0x45, 0xe2, 0xfa, 0x5b, 0x9f, 0xdc, 0x29],
);
const IID_IDWriteLocalFontFileLoader: GUID = GUID(
    0xb2d9f3ec,
    0xc9fe,
    0x4a11,
    [0xa2, 0xec, 0xd8, 0x62, 0x08, 0xf7, 0xc0, 0xa2],
);
const IID_IDWriteTextAnalysisSource: GUID = GUID(
    0x688e1a58,
    0x5094,
    0x47c8,
    [0xad, 0xc8, 0xfb, 0xce, 0xa6, 0x0a, 0xe9, 0x2b],
);

// dwrite.h / dwrite_3.h enumerations used here.
const DWRITE_FACTORY_TYPE_SHARED: i32 = 0;
const DWRITE_FONT_WEIGHT_REGULAR: i32 = 400;
const DWRITE_FONT_STRETCH_NORMAL: i32 = 5;
const DWRITE_FONT_STYLE_NORMAL: i32 = 0;
const DWRITE_FONT_STYLE_ITALIC: i32 = 2;
const DWRITE_FONT_SIMULATIONS_NONE: i32 = 0;
const DWRITE_FONT_SIMULATIONS_BOLD: i32 = 1;
const DWRITE_FONT_SIMULATIONS_OBLIQUE: i32 = 2;
const DWRITE_NUMBER_SUBSTITUTION_METHOD_NONE: i32 = 2;
const DWRITE_READING_DIRECTION_LEFT_TO_RIGHT: i32 = 0;
const DWRITE_INFORMATIONAL_STRING_FULL_NAME: i32 = 16;
const DWRITE_INFORMATIONAL_STRING_POSTSCRIPT_NAME: i32 = 17;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct DWRITE_FONT_AXIS_VALUE {
    axisTag: u32,
    value: f32,
}

#[link(name = "dwrite")]
unsafe extern "system" {
    fn DWriteCreateFactory(
        factory_type: i32,
        iid: *const GUID,
        factory: *mut *mut c_void,
    ) -> HRESULT;
}

/// An owned COM interface pointer (SkTScopedComPtr): released on drop.
struct Com(*mut c_void);

impl Com {
    fn new(raw: *mut c_void) -> Option<Self> {
        (!raw.is_null()).then_some(Self(raw))
    }
    /// The function in vtable slot `index` (IUnknown occupies 0-2).
    unsafe fn slot<F: Copy>(&self, index: usize) -> F {
        unsafe {
            let vtable = *(self.0 as *const *const *const c_void);
            std::mem::transmute_copy(&*vtable.add(index))
        }
    }
    unsafe fn query(&self, iid: &GUID) -> Option<Com> {
        let mut out = std::ptr::null_mut();
        unsafe {
            let query: unsafe extern "system" fn(
                *mut c_void,
                *const GUID,
                *mut *mut c_void,
            ) -> HRESULT = self.slot(0);
            if query(self.0, iid, &mut out) < 0 {
                return None;
            }
        }
        Com::new(out)
    }
    /// Calls a method whose last parameter receives one interface pointer.
    unsafe fn out(&self, call: impl FnOnce(*mut *mut c_void) -> HRESULT) -> Option<Com> {
        let mut out = std::ptr::null_mut();
        if call(&mut out) < 0 {
            if let Some(object) = Com::new(out) {
                drop(object);
            }
            return None;
        }
        Com::new(out)
    }
}

impl Drop for Com {
    fn drop(&mut self) {
        unsafe {
            let release: unsafe extern "system" fn(*mut c_void) -> u32 = self.slot(2);
            release(self.0);
        }
    }
}

type Method0<R> = unsafe extern "system" fn(*mut c_void) -> R;
type MethodOut = unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HRESULT;

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

// Source: src/ports/SkFontMgr_win_dw.cpp:521-619 (FontFallbackSource), the
// IDWriteTextAnalysisSource over the fallback text.
#[repr(C)]
struct FontFallbackSource {
    vtable: *const FontFallbackSourceVtbl,
    ref_count: AtomicU32,
    string: Vec<u16>,
    locale: Vec<u16>,
    number_substitution: *mut c_void,
}

#[repr(C)]
struct FontFallbackSourceVtbl {
    query_interface: unsafe extern "system" fn(
        *mut FontFallbackSource,
        *const GUID,
        *mut *mut c_void,
    ) -> HRESULT,
    add_ref: unsafe extern "system" fn(*mut FontFallbackSource) -> u32,
    release: unsafe extern "system" fn(*mut FontFallbackSource) -> u32,
    get_text_at_position: unsafe extern "system" fn(
        *mut FontFallbackSource,
        u32,
        *mut *const u16,
        *mut u32,
    ) -> HRESULT,
    get_text_before_position: unsafe extern "system" fn(
        *mut FontFallbackSource,
        u32,
        *mut *const u16,
        *mut u32,
    ) -> HRESULT,
    get_paragraph_reading_direction: unsafe extern "system" fn(*mut FontFallbackSource) -> i32,
    get_locale_name: unsafe extern "system" fn(
        *mut FontFallbackSource,
        u32,
        *mut u32,
        *mut *const u16,
    ) -> HRESULT,
    get_number_substitution: unsafe extern "system" fn(
        *mut FontFallbackSource,
        u32,
        *mut u32,
        *mut *mut c_void,
    ) -> HRESULT,
}

static FONT_FALLBACK_SOURCE_VTBL: FontFallbackSourceVtbl = FontFallbackSourceVtbl {
    query_interface: FontFallbackSource::QueryInterface,
    add_ref: FontFallbackSource::AddRef,
    release: FontFallbackSource::Release,
    get_text_at_position: FontFallbackSource::GetTextAtPosition,
    get_text_before_position: FontFallbackSource::GetTextBeforePosition,
    get_paragraph_reading_direction: FontFallbackSource::GetParagraphReadingDirection,
    get_locale_name: FontFallbackSource::GetLocaleName,
    get_number_substitution: FontFallbackSource::GetNumberSubstitution,
};

impl FontFallbackSource {
    fn new(string: Vec<u16>, locale: Vec<u16>, number_substitution: &Com) -> Com {
        Com(Box::into_raw(Box::new(Self {
            vtable: &FONT_FALLBACK_SOURCE_VTBL,
            ref_count: AtomicU32::new(1),
            string,
            locale,
            number_substitution: number_substitution.0,
        }))
        .cast())
    }

    // IUnknown methods
    unsafe extern "system" fn QueryInterface(
        this: *mut Self,
        riid: *const GUID,
        object: *mut *mut c_void,
    ) -> HRESULT {
        unsafe {
            if *riid == IID_IUnknown || *riid == IID_IDWriteTextAnalysisSource {
                *object = this.cast();
                Self::AddRef(this);
                return S_OK;
            }
            *object = std::ptr::null_mut();
        }
        E_FAIL
    }

    unsafe extern "system" fn AddRef(this: *mut Self) -> u32 {
        unsafe { (*this).ref_count.fetch_add(1, Ordering::AcqRel) + 1 }
    }

    unsafe extern "system" fn Release(this: *mut Self) -> u32 {
        let new_count = unsafe { (*this).ref_count.fetch_sub(1, Ordering::AcqRel) - 1 };
        if new_count == 0 {
            drop(unsafe { Box::from_raw(this) });
        }
        new_count
    }

    // IDWriteTextAnalysisSource methods
    unsafe extern "system" fn GetTextAtPosition(
        this: *mut Self,
        text_position: u32,
        text_string: *mut *const u16,
        text_length: *mut u32,
    ) -> HRESULT {
        unsafe {
            let string = &(*this).string;
            let length = string.len() as u32;
            if length <= text_position {
                *text_string = std::ptr::null();
                *text_length = 0;
                return S_OK;
            }
            *text_string = string.as_ptr().add(text_position as usize);
            *text_length = length - text_position;
        }
        S_OK
    }

    unsafe extern "system" fn GetTextBeforePosition(
        this: *mut Self,
        text_position: u32,
        text_string: *mut *const u16,
        text_length: *mut u32,
    ) -> HRESULT {
        unsafe {
            let string = &(*this).string;
            if text_position < 1 || string.len() as u32 <= text_position {
                *text_string = std::ptr::null();
                *text_length = 0;
                return S_OK;
            }
            *text_string = string.as_ptr();
            *text_length = text_position;
        }
        S_OK
    }

    unsafe extern "system" fn GetParagraphReadingDirection(_this: *mut Self) -> i32 {
        // TODO: this is also interesting.
        DWRITE_READING_DIRECTION_LEFT_TO_RIGHT
    }

    unsafe extern "system" fn GetLocaleName(
        this: *mut Self,
        _text_position: u32,
        _text_length: *mut u32,
        locale_name: *mut *const u16,
    ) -> HRESULT {
        unsafe {
            *locale_name = (*this).locale.as_ptr();
        }
        S_OK
    }

    unsafe extern "system" fn GetNumberSubstitution(
        this: *mut Self,
        _text_position: u32,
        _text_length: *mut u32,
        number_substitution: *mut *mut c_void,
    ) -> HRESULT {
        unsafe {
            *number_substitution = (*this).number_substitution;
        }
        S_OK
    }
}

/// The en-us (else first) string of an IDWriteLocalizedStrings, as Skia's
/// sk_get_locale_string reads family and informational names.
unsafe fn localized(strings: &Com) -> Option<String> {
    let english = wide("en-us");
    let mut index = 0u32;
    let mut exists: BOOL = 0;
    unsafe {
        let find: unsafe extern "system" fn(*mut c_void, PCWSTR, *mut u32, *mut BOOL) -> HRESULT =
            strings.slot(4);
        if find(strings.0, english.as_ptr(), &mut index, &mut exists) < 0 {
            return None;
        }
        if exists == 0 {
            index = 0;
        }
        let string_length: unsafe extern "system" fn(*mut c_void, u32, *mut u32) -> HRESULT =
            strings.slot(7);
        let mut length = 0u32;
        if string_length(strings.0, index, &mut length) < 0 {
            return None;
        }
        let get_string: unsafe extern "system" fn(*mut c_void, u32, *mut u16, u32) -> HRESULT =
            strings.slot(8);
        let mut buffer = vec![0u16; length as usize + 1];
        if get_string(strings.0, index, buffer.as_mut_ptr(), length + 1) < 0 {
            return None;
        }
        Some(String::from_utf16_lossy(&buffer[..length as usize]))
    }
}

/// IDWriteFont::GetInformationalStrings.
unsafe fn informational(font: &Com, id: i32) -> Option<String> {
    let mut exists: BOOL = 0;
    unsafe {
        let get: unsafe extern "system" fn(
            *mut c_void,
            i32,
            *mut *mut c_void,
            *mut BOOL,
        ) -> HRESULT = font.slot(9);
        let strings = font.out(|out| get(font.0, id, out, &mut exists))?;
        if exists == 0 {
            return None;
        }
        localized(&strings)
    }
}

/// The installed file, face index, style and instance axis values of an
/// IDWriteFont: the identity SkTypeface_win_dw reads through its local font
/// file loader, with the file handed to Fontations instead of DirectWrite.
unsafe fn host_face(font: &Com) -> Option<HostFace> {
    unsafe {
        let create_font_face: MethodOut = font.slot(13);
        let face = font.out(|out| create_font_face(font.0, out))?;

        let get_files: unsafe extern "system" fn(
            *mut c_void,
            *mut u32,
            *mut *mut c_void,
        ) -> HRESULT = face.slot(4);
        let mut count = 0u32;
        if get_files(face.0, &mut count, std::ptr::null_mut()) < 0 || count == 0 {
            return None;
        }
        let mut raw_files = vec![std::ptr::null_mut(); count as usize];
        if get_files(face.0, &mut count, raw_files.as_mut_ptr()) < 0 {
            return None;
        }
        let files: Vec<Option<Com>> = raw_files.into_iter().map(Com::new).collect();
        let file = files.first()?.as_ref()?;
        let get_reference_key: unsafe extern "system" fn(
            *mut c_void,
            *mut *const c_void,
            *mut u32,
        ) -> HRESULT = file.slot(3);
        let mut key = std::ptr::null();
        let mut key_size = 0u32;
        if get_reference_key(file.0, &mut key, &mut key_size) < 0 {
            return None;
        }
        let get_loader: MethodOut = file.slot(4);
        let loader = file.out(|out| get_loader(file.0, out))?;
        // Only fonts backed by local files can be opened by path.
        let local = loader.query(&IID_IDWriteLocalFontFileLoader)?;
        let path_length_from_key: unsafe extern "system" fn(
            *mut c_void,
            *const c_void,
            u32,
            *mut u32,
        ) -> HRESULT = local.slot(4);
        let mut path_length = 0u32;
        if path_length_from_key(local.0, key, key_size, &mut path_length) < 0 {
            return None;
        }
        let path_from_key: unsafe extern "system" fn(
            *mut c_void,
            *const c_void,
            u32,
            *mut u16,
            u32,
        ) -> HRESULT = local.slot(5);
        let mut path = vec![0u16; path_length as usize + 1];
        if path_from_key(local.0, key, key_size, path.as_mut_ptr(), path_length + 1) < 0 {
            return None;
        }
        let path = String::from_utf16_lossy(&path[..path_length as usize]);

        let get_font_family: MethodOut = font.slot(3);
        let family = font.out(|out| get_font_family(font.0, out))?;
        let get_family_names: MethodOut = family.slot(6);
        let names = family.out(|out| get_family_names(family.0, out))?;
        let family = localized(&names)?;

        let get_index: Method0<u32> = face.slot(5);
        let get_weight: Method0<i32> = font.slot(4);
        let get_style: Method0<i32> = font.slot(6);
        let color = font.query(&IID_IDWriteFont2).is_some_and(|font2| {
            let is_color_font: Method0<BOOL> = font2.slot(18);
            is_color_font(font2.0) != 0
        });
        let variations = face
            .query(&IID_IDWriteFontFace5)
            .map(|face5| {
                let has_variations: Method0<BOOL> = face5.slot(55);
                if has_variations(face5.0) == 0 {
                    return Vec::new();
                }
                let axis_value_count: Method0<u32> = face5.slot(53);
                let get_axis_values: unsafe extern "system" fn(
                    *mut c_void,
                    *mut DWRITE_FONT_AXIS_VALUE,
                    u32,
                ) -> HRESULT = face5.slot(54);
                let mut values =
                    vec![DWRITE_FONT_AXIS_VALUE::default(); axis_value_count(face5.0) as usize];
                if get_axis_values(face5.0, values.as_mut_ptr(), values.len() as u32) < 0 {
                    return Vec::new();
                }
                values
                    .iter()
                    .map(|value| FontVariation {
                        // DWRITE_MAKE_FONT_AXIS_TAG packs the first character
                        // into the low byte.
                        tag: u32::from_be_bytes(value.axisTag.to_le_bytes()),
                        value: value.value,
                    })
                    .collect()
            })
            .unwrap_or_default();
        Some(HostFace {
            family,
            path,
            index: get_index(face.0),
            weight: f64::from(get_weight(font.0)),
            italic: get_style(font.0) != DWRITE_FONT_STYLE_NORMAL,
            locales: Vec::new(),
            color,
            variations,
            unique_name: informational(font, DWRITE_INFORMATIONAL_STRING_POSTSCRIPT_NAME)
                .unwrap_or_default(),
        })
    }
}

/// Skia's DWriteStyle(SkFontStyle) (src/utils/win/SkDWrite.h:84-93).
fn DWriteStyle(weight: f64, italic: bool) -> (i32, i32) {
    (
        weight.round().clamp(1.0, 999.0) as i32,
        if italic {
            DWRITE_FONT_STYLE_ITALIC
        } else {
            DWRITE_FONT_STYLE_NORMAL
        },
    )
}

/// Chrome's default generic families on Windows
/// (chrome/app/resources/locale_settings_win.grd). system-ui is
/// FontCache::SystemFontFamily(), the menu font (font_cache_skia_win.cc).
fn Generic(generic: &str) -> String {
    match generic {
        "serif" | "ui-serif" | "standard" => "Times New Roman".into(),
        "sans-serif" | "ui-sans-serif" => "Arial".into(),
        "monospace" | "ui-monospace" => "Consolas".into(),
        "cursive" => "Comic Sans MS".into(),
        "fantasy" => "Impact".into(),
        "math" => "Cambria Math".into(),
        "emoji" => "Segoe UI Emoji".into(),
        "system-ui" | "-apple-system" | "BlinkMacSystemFont" => {
            font_engine::fonts::font_cache_win::SystemFontFamily()
        }
        other => other.into(),
    }
}

struct DirectWrite {
    factory: Com,
    collection: Com,
    fallback: Option<Com>,
}

impl DirectWrite {
    fn new() -> Option<Self> {
        unsafe {
            let mut raw = std::ptr::null_mut();
            if DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED, &IID_IDWriteFactory2, &mut raw) < 0 {
                return None;
            }
            let factory = Com::new(raw)?;
            let get_system_font_collection: unsafe extern "system" fn(
                *mut c_void,
                *mut *mut c_void,
                BOOL,
            ) -> HRESULT = factory.slot(3);
            let collection = factory.out(|out| get_system_font_collection(factory.0, out, 0))?;
            let get_system_font_fallback: MethodOut = factory.slot(26);
            let fallback = factory.out(|out| get_system_font_fallback(factory.0, out));
            Some(Self {
                factory,
                collection,
                fallback,
            })
        }
    }

    // Source: src/ports/SkFontMgr_win_dw.cpp:362-385 (onMatchFamily /
    // onMatchFamilyStyle) and SkFontStyleSet_DirectWrite::matchStyle, which
    // resolves through FirstMatchingFontWithoutSimulations (lines 60-94).
    // The SK_WIN_FONTMGR_NO_SIMULATIONS form is used: the face is handed to
    // Fontations as file bytes, so a DirectWrite simulation cannot be carried
    // and Blink synthesizes bold/italic from the real face's style instead.
    fn MatchFamily(&self, family: &str, weight: f64, italic: bool) -> Vec<HostFace> {
        let name = wide(family);
        let mut index = 0u32;
        let mut exists: BOOL = 0;
        unsafe {
            let find_family_name: unsafe extern "system" fn(
                *mut c_void,
                PCWSTR,
                *mut u32,
                *mut BOOL,
            ) -> HRESULT = self.collection.slot(5);
            if find_family_name(self.collection.0, name.as_ptr(), &mut index, &mut exists) < 0
                || exists == 0
            {
                return Vec::new();
            }
            let get_font_family: unsafe extern "system" fn(
                *mut c_void,
                u32,
                *mut *mut c_void,
            ) -> HRESULT = self.collection.slot(4);
            let Some(font_family) = self
                .collection
                .out(|out| get_font_family(self.collection.0, index, out))
            else {
                return Vec::new();
            };
            let get_first_matching_font: unsafe extern "system" fn(
                *mut c_void,
                i32,
                i32,
                i32,
                *mut *mut c_void,
            ) -> HRESULT = font_family.slot(7);
            let (mut dw_weight, mut dw_slant) = DWriteStyle(weight, italic);
            loop {
                let Some(font) = font_family.out(|out| {
                    get_first_matching_font(
                        font_family.0,
                        dw_weight,
                        DWRITE_FONT_STRETCH_NORMAL,
                        dw_slant,
                        out,
                    )
                }) else {
                    return Vec::new();
                };
                let get_simulations: Method0<i32> = font.slot(10);
                let simulations = get_simulations(font.0);
                // If we still get simulations even though we're not asking for
                // bold or italic, we can't help it and exit the loop.
                let no_simulations = simulations == DWRITE_FONT_SIMULATIONS_NONE
                    || (dw_weight == DWRITE_FONT_WEIGHT_REGULAR
                        && dw_slant == DWRITE_FONT_STYLE_NORMAL);
                if no_simulations {
                    return host_face(&font).into_iter().collect();
                }
                if simulations & DWRITE_FONT_SIMULATIONS_BOLD != 0 {
                    dw_weight = DWRITE_FONT_WEIGHT_REGULAR;
                    continue;
                }
                if simulations & DWRITE_FONT_SIMULATIONS_OBLIQUE != 0 {
                    dw_slant = DWRITE_FONT_STYLE_NORMAL;
                    continue;
                }
                return host_face(&font).into_iter().collect();
            }
        }
    }

    /// CSS src: local() by PostScript or full font name, as Blink's
    /// FontUniqueNameLookupWin matches them case-insensitively.
    fn MatchUnique(&self, name: &str) -> Vec<HostFace> {
        if name.is_empty() {
            return Vec::new();
        }
        let wanted = name.to_lowercase();
        unsafe {
            let family_count: Method0<u32> = self.collection.slot(3);
            let get_font_family: unsafe extern "system" fn(
                *mut c_void,
                u32,
                *mut *mut c_void,
            ) -> HRESULT = self.collection.slot(4);
            for family_index in 0..family_count(self.collection.0) {
                let Some(family) = self
                    .collection
                    .out(|out| get_font_family(self.collection.0, family_index, out))
                else {
                    continue;
                };
                let font_count: Method0<u32> = family.slot(4);
                let get_font: unsafe extern "system" fn(
                    *mut c_void,
                    u32,
                    *mut *mut c_void,
                ) -> HRESULT = family.slot(5);
                for font_index in 0..font_count(family.0) {
                    let Some(font) = family.out(|out| get_font(family.0, font_index, out)) else {
                        continue;
                    };
                    let matches = [
                        DWRITE_INFORMATIONAL_STRING_POSTSCRIPT_NAME,
                        DWRITE_INFORMATIONAL_STRING_FULL_NAME,
                    ]
                    .into_iter()
                    .filter_map(|id| informational(&font, id))
                    .any(|candidate| candidate.to_lowercase() == wanted);
                    if matches {
                        return host_face(&font).into_iter().collect();
                    }
                }
            }
        }
        Vec::new()
    }

    // Source: src/ports/SkFontMgr_win_dw.cpp:621-653 (onMatchFamilyStyleCharacter)
    // and 901-969 (fallback), with no simulations allowed for the reason given
    // on MatchFamily.
    fn Fallback(
        &self,
        family: &str,
        weight: f64,
        italic: bool,
        locale: &str,
        text: &[u16],
    ) -> Vec<HostFace> {
        let Some(fallback) = &self.fallback else {
            return Vec::new();
        };
        if text.is_empty() {
            return Vec::new();
        }
        let dw_bcp47 = wide(locale);
        let dw_family_name = (!family.is_empty()).then(|| wide(family));
        unsafe {
            let create_number_substitution: unsafe extern "system" fn(
                *mut c_void,
                i32,
                PCWSTR,
                BOOL,
                *mut *mut c_void,
            ) -> HRESULT = self.factory.slot(22);
            let Some(number_substitution) = self.factory.out(|out| {
                create_number_substitution(
                    self.factory.0,
                    DWRITE_NUMBER_SUBSTITUTION_METHOD_NONE,
                    dw_bcp47.as_ptr(),
                    1,
                    out,
                )
            }) else {
                return Vec::new();
            };
            let source =
                FontFallbackSource::new(text.to_vec(), dw_bcp47.clone(), &number_substitution);
            let map_characters: unsafe extern "system" fn(
                *mut c_void,
                *mut c_void,
                u32,
                u32,
                *mut c_void,
                PCWSTR,
                i32,
                i32,
                i32,
                *mut u32,
                *mut *mut c_void,
                *mut f32,
            ) -> HRESULT = fallback.slot(3);
            let (mut dw_weight, mut dw_slant) = DWriteStyle(weight, italic);
            loop {
                let mut mapped_length = 0u32;
                let mut scale = 0f32;
                let Some(font) = fallback.out(|out| {
                    map_characters(
                        fallback.0,
                        source.0,
                        0, // textPosition,
                        text.len() as u32,
                        self.collection.0,
                        dw_family_name
                            .as_ref()
                            .map_or(std::ptr::null(), |name| name.as_ptr()),
                        dw_weight,
                        dw_slant,
                        DWRITE_FONT_STRETCH_NORMAL,
                        &mut mapped_length,
                        out,
                        &mut scale,
                    )
                }) else {
                    return Vec::new();
                };
                let get_simulations: Method0<i32> = font.slot(10);
                let simulations = get_simulations(font.0);
                if simulations == DWRITE_FONT_SIMULATIONS_NONE
                    || (dw_weight == DWRITE_FONT_WEIGHT_REGULAR
                        && dw_slant == DWRITE_FONT_STYLE_NORMAL)
                {
                    return host_face(&font).into_iter().collect();
                }
                if simulations & DWRITE_FONT_SIMULATIONS_BOLD != 0 {
                    dw_weight = DWRITE_FONT_WEIGHT_REGULAR;
                    continue;
                }
                if simulations & DWRITE_FONT_SIMULATIONS_OBLIQUE != 0 {
                    dw_slant = DWRITE_FONT_STYLE_NORMAL;
                    continue;
                }
                return host_face(&font).into_iter().collect();
            }
        }
    }
}

#[derive(Default)]
pub(super) struct WindowsFontHost {
    dwrite: Option<DirectWrite>,
}

impl WindowsFontHost {
    pub(super) fn Query(
        &mut self,
        query: &SystemFontQuery,
        base: Option<&HostFace>,
    ) -> Vec<HostFace> {
        if self.dwrite.is_none() {
            self.dwrite = DirectWrite::new();
        }
        let Some(dwrite) = &self.dwrite else {
            return Vec::new();
        };
        match query {
            SystemFontQuery::MatchFamily {
                family,
                weight,
                italic,
                ..
            } => dwrite.MatchFamily(family, *weight, *italic),
            SystemFontQuery::MatchGeneric {
                generic,
                weight,
                italic,
                ..
            } => dwrite.MatchFamily(&Generic(generic), *weight, *italic),
            SystemFontQuery::MatchUniqueName { name } => dwrite.MatchUnique(name),
            SystemFontQuery::Fallback {
                text,
                weight,
                italic,
                locale,
                presentation,
                ..
            } => {
                // FontCache::GetDWriteFallbackFamily passes the one fallback
                // locale; emoji presentation uses the color emoji locale.
                let locale = if *presentation == FontPresentation::Emoji {
                    "und-Zsye"
                } else if locale.is_empty() {
                    "en-us"
                } else {
                    locale.as_str()
                };
                let characters: Vec<u16> = char::decode_utf16(text.iter().copied())
                    .filter_map(Result::ok)
                    .filter(|&c| !matches!(u32::from(c), 0x200c..=0x200d | 0xfe00..=0xfe0f | 0xe0100..=0xe01ef))
                    .flat_map(|c| {
                        let mut units = [0u16; 2];
                        c.encode_utf16(&mut units).to_vec()
                    })
                    .collect();
                let family = base.map_or("", |face| face.family.as_str());
                let mut faces = dwrite.Fallback(family, *weight, *italic, locale, &characters);
                for face in &mut faces {
                    face.locales.push(locale.to_owned());
                }
                faces
            }
            SystemFontQuery::OpenFace { .. } => Vec::new(),
        }
    }
}

// --- ui/gfx/win/system_fonts_win.cc ---------------------------------------

#[repr(C)]
#[derive(Clone, Copy)]
struct LOGFONTW {
    lfHeight: i32,
    lfWidth: i32,
    lfEscapement: i32,
    lfOrientation: i32,
    lfWeight: i32,
    lfItalic: u8,
    lfUnderline: u8,
    lfStrikeOut: u8,
    lfCharSet: u8,
    lfOutPrecision: u8,
    lfClipPrecision: u8,
    lfQuality: u8,
    lfPitchAndFamily: u8,
    lfFaceName: [u16; 32],
}

#[repr(C)]
struct NONCLIENTMETRICSW {
    cbSize: u32,
    iBorderWidth: i32,
    iScrollWidth: i32,
    iScrollHeight: i32,
    iCaptionWidth: i32,
    iCaptionHeight: i32,
    lfCaptionFont: LOGFONTW,
    iSmCaptionWidth: i32,
    iSmCaptionHeight: i32,
    lfSmCaptionFont: LOGFONTW,
    iMenuWidth: i32,
    iMenuHeight: i32,
    lfMenuFont: LOGFONTW,
    lfStatusFont: LOGFONTW,
    lfMessageFont: LOGFONTW,
    iPaddedBorderWidth: i32,
}

#[repr(C)]
#[derive(Default)]
struct TEXTMETRICW {
    tmHeight: i32,
    tmAscent: i32,
    tmDescent: i32,
    tmInternalLeading: i32,
    tmExternalLeading: i32,
    tmAveCharWidth: i32,
    tmMaxCharWidth: i32,
    tmWeight: i32,
    tmOverhang: i32,
    tmDigitizedAspectX: i32,
    tmDigitizedAspectY: i32,
    tmFirstChar: u16,
    tmLastChar: u16,
    tmDefaultChar: u16,
    tmBreakChar: u16,
    tmItalic: u8,
    tmUnderlined: u8,
    tmStruckOut: u8,
    tmPitchAndFamily: u8,
    tmCharSet: u8,
}

const SPI_GETNONCLIENTMETRICS: u32 = 0x0029;
const LOGPIXELSY: i32 = 90;

#[link(name = "user32")]
unsafe extern "system" {
    fn SystemParametersInfoW(action: u32, param: u32, data: *mut c_void, win_ini: u32) -> BOOL;
    fn GetDC(window: *mut c_void) -> *mut c_void;
    fn ReleaseDC(window: *mut c_void, dc: *mut c_void) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetACP() -> u32;
}

#[link(name = "gdi32")]
unsafe extern "system" {
    fn GetDeviceCaps(dc: *mut c_void, index: i32) -> i32;
    fn CreateFontIndirectW(font: *const LOGFONTW) -> *mut c_void;
    fn SelectObject(dc: *mut c_void, object: *mut c_void) -> *mut c_void;
    fn GetObjectW(object: *mut c_void, size: i32, out: *mut c_void) -> i32;
    fn GetTextMetricsW(dc: *mut c_void, metrics: *mut TEXTMETRICW) -> BOOL;
    fn DeleteObject(object: *mut c_void) -> BOOL;
}

// Source: system_fonts_win.cc:73-89 (AdjustLOGFONT), with Chrome's
// l10n/text-zoom adjust_font_callback_ unset in this embedding.
fn AdjustLOGFONT(font_scale: f64, logfont: &mut LOGFONTW) {
    let mut new_height = (f64::from(logfont.lfHeight) * font_scale).round() as i32;
    if logfont.lfHeight != 0 && new_height == 0 {
        new_height = if logfont.lfHeight > 0 { 1 } else { -1 };
    }
    logfont.lfHeight = new_height;
}

// Source: system_fonts_win.cc:57-71 (AdjustFontSize) with no minimum size
// callback.
fn AdjustFontSize(lf_height: i32, size_delta: i32) -> i32 {
    let lf_sign = if lf_height < 0 { -1 } else { 1 };
    let lf_height = (lf_height.abs() + size_delta).max(0);
    lf_sign * lf_height
}

// Source: system_fonts_win.cc:91-125 (GetFontFromLOGFONT): the mapped face
// name and the font size tmHeight - tmInternalLeading.
unsafe fn GetFontFromLOGFONT(logfont: &LOGFONTW) -> (String, i32) {
    unsafe {
        let font = CreateFontIndirectW(logfont);
        let screen_dc = GetDC(std::ptr::null_mut());
        let previous = SelectObject(screen_dc, font);

        let mut mapped_font_info = *logfont;
        GetObjectW(
            font,
            std::mem::size_of::<LOGFONTW>() as i32,
            (&mut mapped_font_info as *mut LOGFONTW).cast(),
        );
        let face_name = &mapped_font_info.lfFaceName;
        let length = face_name
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(face_name.len());
        let font_name = String::from_utf16_lossy(&face_name[..length]);

        let mut mapped_font_metrics = TEXTMETRICW::default();
        GetTextMetricsW(screen_dc, &mut mapped_font_metrics);
        let font_size =
            (mapped_font_metrics.tmHeight - mapped_font_metrics.tmInternalLeading).max(1);

        SelectObject(screen_dc, previous);
        ReleaseDC(std::ptr::null_mut(), screen_dc);
        DeleteObject(font);
        (font_name, font_size)
    }
}

/// Reads the NONCLIENTMETRICS fonts as SystemFonts::Initialize does and
/// stores the menu, small-caption and status fonts in FontCache, as
/// RenderViewHostImpl's GetFontInfo and WebViewImpl::SetRendererPreferences
/// carry them in Chromium.
pub(super) fn InitializeSystemFontMetrics() {
    static INITIALIZED: std::sync::Once = std::sync::Once::new();
    INITIALIZED.call_once(|| unsafe {
        let mut metrics: NONCLIENTMETRICSW = std::mem::zeroed();
        metrics.cbSize = std::mem::size_of::<NONCLIENTMETRICSW>() as u32;
        if SystemParametersInfoW(
            SPI_GETNONCLIENTMETRICS,
            metrics.cbSize,
            (&mut metrics as *mut NONCLIENTMETRICSW).cast(),
            0,
        ) == 0
        {
            return;
        }
        // Factor out system DPI scale that Windows will include in reported
        // font sizes (SystemFonts::GetSystemScale).
        const kDefaultDPI: f64 = 96.0;
        let screen_dc = GetDC(std::ptr::null_mut());
        let system_scale = f64::from(GetDeviceCaps(screen_dc, LOGPIXELSY)) / kDefaultDPI;
        ReleaseDC(std::ptr::null_mut(), screen_dc);
        let font_scale = 1.0 / system_scale;

        let font = |logfont: &LOGFONTW| {
            let mut logfont = *logfont;
            AdjustLOGFONT(font_scale, &mut logfont);
            logfont.lfHeight = AdjustFontSize(logfont.lfHeight, 0);
            GetFontFromLOGFONT(&logfont)
        };
        use font_engine::fonts::font_cache_win as font_cache;
        font_cache::SetAnsiCodePage(GetACP());
        let (family, height) = font(&metrics.lfMenuFont);
        font_cache::SetMenuFontMetrics(&family, height);
        let (family, height) = font(&metrics.lfSmCaptionFont);
        font_cache::SetSmallCaptionFontMetrics(&family, height);
        let (family, height) = font(&metrics.lfStatusFont);
        font_cache::SetStatusFontMetrics(&family, height);

        // Renderer preferences should_antialias_text and subpixel_rendering
        // from gfx::GetFontRenderParams, applied by WebViewImpl.
        let params = QueryFontRenderParams();
        font_cache::SetAntialiasedTextEnabled(params.antialiasing);
        font_cache::SetLCDTextEnabled(params.subpixel_rendering);
    });
}

// --- ui/gfx/font_render_params_win.cc ---------------------------------------

const SPI_GETFONTSMOOTHING: u32 = 0x004a;
const SPI_GETFONTSMOOTHINGTYPE: u32 = 0x200a;
const SPI_GETFONTSMOOTHINGORIENTATION: u32 = 0x2012;
const FE_FONTSMOOTHINGCLEARTYPE: u32 = 0x0002;
const FE_FONTSMOOTHINGORIENTATIONBGR: u32 = 0x0000;
const FE_FONTSMOOTHINGORIENTATIONRGB: u32 = 0x0001;

struct FontRenderParams {
    antialiasing: bool,
    /// subpixel_rendering != SUBPIXEL_RENDERING_NONE.
    subpixel_rendering: bool,
}

// Source: font_render_params_win.cc:31-61 (GetSubpixelRenderingGeometry).
// The per-display Avalon.Graphics PixelStructure registry override that
// precedes the system orientation is not read.
unsafe fn GetSubpixelRenderingGeometry() -> bool {
    let mut structure = 0u32;
    unsafe {
        if SystemParametersInfoW(
            SPI_GETFONTSMOOTHINGORIENTATION,
            0,
            (&mut structure as *mut u32).cast(),
            0,
        ) != 0
        {
            return matches!(
                structure,
                FE_FONTSMOOTHINGORIENTATIONRGB | FE_FONTSMOOTHINGORIENTATIONBGR
            );
        }
    }
    // No explicit ClearType settings, default to none.
    false
}

// Source: font_render_params_win.cc:74-97 (CachedFontRenderParams::GetParams).
unsafe fn QueryFontRenderParams() -> FontRenderParams {
    let mut params = FontRenderParams {
        antialiasing: false,
        subpixel_rendering: false,
    };
    unsafe {
        let mut enabled: BOOL = 0;
        if SystemParametersInfoW(SPI_GETFONTSMOOTHING, 0, (&mut enabled as *mut BOOL).cast(), 0) != 0
            && enabled != 0
        {
            params.antialiasing = true;
            let mut kind = 0u32;
            if SystemParametersInfoW(SPI_GETFONTSMOOTHINGTYPE, 0, (&mut kind as *mut u32).cast(), 0)
                != 0
                && kind == FE_FONTSMOOTHINGCLEARTYPE
            {
                params.subpixel_rendering = GetSubpixelRenderingGeometry();
            }
        }
    }
    params
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installed_directwrite_queries_resolve_local_files() {
        let mut host = WindowsFontHost::default();
        let arial = host.Query(
            &SystemFontQuery::MatchFamily {
                family: "Arial".into(),
                weight: 700.0,
                italic: false,
                locale: String::new(),
            },
            None,
        );
        assert_eq!(arial.len(), 1);
        assert_eq!(arial[0].family, "Arial");
        assert_eq!(arial[0].weight, 700.0);
        assert!(std::path::Path::new(&arial[0].path).is_file());
        assert!(host
            .Query(
                &SystemFontQuery::MatchFamily {
                    family: "No Such Family Rechrom".into(),
                    weight: 400.0,
                    italic: false,
                    locale: String::new(),
                },
                None,
            )
            .is_empty());
        let unique = host.Query(
            &SystemFontQuery::MatchUniqueName {
                name: "Arial-BoldMT".into(),
            },
            None,
        );
        assert_eq!(
            unique.first().map(|face| face.unique_name.as_str()),
            Some("Arial-BoldMT")
        );
        let fallback = host.Query(
            &SystemFontQuery::Fallback {
                text: "\u{0E01}".encode_utf16().collect(),
                weight: 400.0,
                italic: false,
                locale: "th".into(),
                presentation: FontPresentation::Text,
                base_face: None,
            },
            arial.first(),
        );
        assert_eq!(fallback.len(), 1, "Thai fallback");
        InitializeSystemFontMetrics();
        assert!(!font_engine::fonts::font_cache_win::MenuFontFamily().is_empty());
        assert!(font_engine::fonts::font_cache_win::MenuFontHeight() > 0);
    }
}
