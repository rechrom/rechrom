//! Fontconfig host requests, following SkFontMgr_fontconfig family matching and
//! Chromium ui/gfx/font_fallback_linux.cc contextual fallback. Only platform
//! metadata is queried here; files are opened separately by OpenFace.

use super::HostFace;
use font_engine::{FontPresentation, FontVariation, SystemFontQuery};
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::sync::OnceLock;

type Pattern = *mut c_void;
type Config = *mut c_void;
#[repr(C)]
struct FontSet {
    count: c_int,
    capacity: c_int,
    fonts: *mut Pattern,
}
#[repr(C)]
#[derive(Clone, Copy)]
union ValueData {
    string: *const u8,
    integer: c_int,
    double: f64,
    pointer: *const c_void,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct Value {
    kind: c_int,
    data: ValueData,
}

// Runtime loading retains the existing Linux runtime dependency on Fontconfig
// without requiring development headers or an unversioned linker symlink.
struct Api {
    config_new: unsafe extern "C" fn() -> Config,
    config_destroy: unsafe extern "C" fn(Config),
    pattern_new: unsafe extern "C" fn() -> Pattern,
    pattern_destroy: unsafe extern "C" fn(Pattern),
    add_string: unsafe extern "C" fn(Pattern, *const c_char, *const u8) -> c_int,
    add_integer: unsafe extern "C" fn(Pattern, *const c_char, c_int) -> c_int,
    add_double: unsafe extern "C" fn(Pattern, *const c_char, f64) -> c_int,
    add_bool: unsafe extern "C" fn(Pattern, *const c_char, c_int) -> c_int,
    get_string: unsafe extern "C" fn(Pattern, *const c_char, c_int, *mut *mut u8) -> c_int,
    get_integer: unsafe extern "C" fn(Pattern, *const c_char, c_int, *mut c_int) -> c_int,
    get_double: unsafe extern "C" fn(Pattern, *const c_char, c_int, *mut f64) -> c_int,
    get_bool: unsafe extern "C" fn(Pattern, *const c_char, c_int, *mut c_int) -> c_int,
    get_binding:
        unsafe extern "C" fn(Pattern, *const c_char, c_int, *mut Value, *mut c_int) -> c_int,
    get_charset: unsafe extern "C" fn(Pattern, *const c_char, c_int, *mut *mut c_void) -> c_int,
    charset_has: unsafe extern "C" fn(*const c_void, u32) -> c_int,
    charset_new: unsafe extern "C" fn() -> *mut c_void,
    charset_destroy: unsafe extern "C" fn(*mut c_void),
    charset_add: unsafe extern "C" fn(*mut c_void, u32) -> c_int,
    add_charset: unsafe extern "C" fn(Pattern, *const c_char, *const c_void) -> c_int,
    substitute: unsafe extern "C" fn(Config, Pattern, c_int) -> c_int,
    defaults: unsafe extern "C" fn(Pattern),
    font_match: unsafe extern "C" fn(Config, Pattern, *mut c_int) -> Pattern,
    font_sort:
        unsafe extern "C" fn(Config, Pattern, c_int, *mut *mut c_void, *mut c_int) -> *mut FontSet,
    font_set_destroy: unsafe extern "C" fn(*mut FontSet),
    render_prepare: unsafe extern "C" fn(Config, Pattern, Pattern) -> Pattern,
}

impl Api {
    fn Get() -> Option<&'static Self> {
        static API: OnceLock<Option<Api>> = OnceLock::new();
        API.get_or_init(|| unsafe { Self::Load() }).as_ref()
    }

    unsafe fn Load() -> Option<Self> {
        #[link(name = "dl")]
        unsafe extern "C" {
            fn dlopen(name: *const c_char, flags: c_int) -> *mut c_void;
            fn dlsym(handle: *mut c_void, name: *const c_char) -> *mut c_void;
        }
        let handle = dlopen(c"libfontconfig.so.1".as_ptr(), 2); // RTLD_NOW
        if handle.is_null() {
            return None;
        }
        unsafe fn Symbol<T: Copy>(handle: *mut c_void, name: &CStr) -> Option<T> {
            let symbol = dlsym(handle, name.as_ptr());
            if symbol.is_null() {
                return None;
            }
            // Every T below is the exact documented C function-pointer type.
            assert_eq!(std::mem::size_of::<T>(), std::mem::size_of_val(&symbol));
            Some(std::mem::transmute_copy(&symbol))
        }
        // The library stays loaded for the lifetime of these function pointers.
        Some(Self {
            config_new: Symbol(handle, c"FcInitLoadConfigAndFonts")?,
            config_destroy: Symbol(handle, c"FcConfigDestroy")?,
            pattern_new: Symbol(handle, c"FcPatternCreate")?,
            pattern_destroy: Symbol(handle, c"FcPatternDestroy")?,
            add_string: Symbol(handle, c"FcPatternAddString")?,
            add_integer: Symbol(handle, c"FcPatternAddInteger")?,
            add_double: Symbol(handle, c"FcPatternAddDouble")?,
            add_bool: Symbol(handle, c"FcPatternAddBool")?,
            get_string: Symbol(handle, c"FcPatternGetString")?,
            get_integer: Symbol(handle, c"FcPatternGetInteger")?,
            get_double: Symbol(handle, c"FcPatternGetDouble")?,
            get_bool: Symbol(handle, c"FcPatternGetBool")?,
            get_binding: Symbol(handle, c"FcPatternGetWithBinding")?,
            get_charset: Symbol(handle, c"FcPatternGetCharSet")?,
            charset_has: Symbol(handle, c"FcCharSetHasChar")?,
            charset_new: Symbol(handle, c"FcCharSetCreate")?,
            charset_destroy: Symbol(handle, c"FcCharSetDestroy")?,
            charset_add: Symbol(handle, c"FcCharSetAddChar")?,
            add_charset: Symbol(handle, c"FcPatternAddCharSet")?,
            substitute: Symbol(handle, c"FcConfigSubstitute")?,
            defaults: Symbol(handle, c"FcDefaultSubstitute")?,
            font_match: Symbol(handle, c"FcFontMatch")?,
            font_sort: Symbol(handle, c"FcFontSort")?,
            font_set_destroy: Symbol(handle, c"FcFontSetDestroy")?,
            render_prepare: Symbol(handle, c"FcFontRenderPrepare")?,
        })
    }

    unsafe fn String(&self, pattern: Pattern, object: &CStr, index: c_int) -> Option<String> {
        let mut value = std::ptr::null_mut();
        if (self.get_string)(pattern, object.as_ptr(), index, &mut value) != 0 || value.is_null() {
            return None;
        }
        Some(CStr::from_ptr(value.cast()).to_str().ok()?.to_owned())
    }
    unsafe fn Integer(&self, pattern: Pattern, object: &CStr, default: c_int) -> c_int {
        let mut value = default;
        if (self.get_integer)(pattern, object.as_ptr(), 0, &mut value) == 0 {
            value
        } else {
            default
        }
    }
    unsafe fn Double(&self, pattern: Pattern, object: &CStr, default: f64) -> f64 {
        let mut value = default;
        if (self.get_double)(pattern, object.as_ptr(), 0, &mut value) == 0 {
            value
        } else {
            default
        }
    }
    unsafe fn Bool(&self, pattern: Pattern, object: &CStr) -> bool {
        let mut value = 0;
        (self.get_bool)(pattern, object.as_ptr(), 0, &mut value) == 0 && value != 0
    }
    unsafe fn AddString(&self, pattern: Pattern, object: &CStr, value: &str) -> bool {
        let Ok(value) = CString::new(value) else {
            return false;
        };
        (self.add_string)(pattern, object.as_ptr(), value.as_ptr().cast()) != 0
    }
    unsafe fn Face(&self, pattern: Pattern) -> Option<HostFace> {
        if !self.Bool(pattern, c"scalable") {
            return None;
        }
        let format = self.String(pattern, c"fontformat", 0)?;
        if !matches!(format.as_str(), "TrueType" | "CFF") {
            return None;
        }
        let path = self.String(pattern, c"file", 0)?;
        if path.is_empty() {
            return None;
        }
        let index = u32::try_from(self.Integer(pattern, c"index", 0)).ok()?;
        let family = self.String(pattern, c"family", 0)?;
        let variations = self
            .String(pattern, c"fontvariations", 0)
            .map(|v| ParseVariations(&v))
            .unwrap_or_default();
        Some(HostFace {
            family,
            path,
            index,
            weight: OpenTypeWeight(self.Double(pattern, c"weight", 80.0)),
            italic: self.Integer(pattern, c"slant", 0) != 0,
            locales: Vec::new(), // Locale preference is applied by the query, not guessed from coverage.
            color: self.Bool(pattern, c"color"),
            variations,
            unique_name: self
                .String(pattern, c"postscriptname", 0)
                .unwrap_or_default(),
        })
    }
}

struct OwnedPattern<'a> {
    api: &'a Api,
    raw: Pattern,
}
impl Drop for OwnedPattern<'_> {
    fn drop(&mut self) {
        unsafe {
            (self.api.pattern_destroy)(self.raw);
        }
    }
}
struct OwnedFontSet<'a> {
    api: &'a Api,
    raw: *mut FontSet,
}
impl Drop for OwnedFontSet<'_> {
    fn drop(&mut self) {
        unsafe {
            (self.api.font_set_destroy)(self.raw);
        }
    }
}
#[derive(Default)]
pub(super) struct LinuxFontHost {
    config: Option<(&'static Api, Config)>,
}
impl Drop for LinuxFontHost {
    fn drop(&mut self) {
        if let Some((api, config)) = self.config {
            unsafe {
                (api.config_destroy)(config);
            }
        }
    }
}

impl LinuxFontHost {
    pub(super) fn Query(
        &mut self,
        query: &SystemFontQuery,
        base: Option<&HostFace>,
    ) -> Vec<HostFace> {
        let Some(api) = Api::Get() else {
            return Vec::new();
        };
        // SAFETY: All native objects stay within this synchronous host request;
        // Fontconfig owns returned string/charset pointers until their pattern
        // or set drops. Returned HostFaces contain only copied Rust values.
        unsafe {
            let config = if let Some((_, config)) = self.config {
                config
            } else {
                let config = (api.config_new)();
                if config.is_null() {
                    return Vec::new();
                }
                self.config = Some((api, config));
                config
            };
            if let SystemFontQuery::MatchUniqueName { name } = query {
                return MatchUnique(api, config, name);
            }
            let raw = (api.pattern_new)();
            if raw.is_null() {
                return Vec::new();
            }
            let pattern = OwnedPattern { api, raw };
            let (family, weight, italic, locale, fallback, presentation, text) = match query {
                SystemFontQuery::MatchFamily {
                    family,
                    weight,
                    italic,
                    locale,
                } => (
                    family.as_str(),
                    *weight,
                    *italic,
                    locale.as_str(),
                    false,
                    FontPresentation::Text,
                    &[][..],
                ),
                SystemFontQuery::MatchGeneric {
                    generic,
                    weight,
                    italic,
                    locale,
                } => (
                    Generic(generic),
                    *weight,
                    *italic,
                    locale.as_str(),
                    false,
                    FontPresentation::Text,
                    &[][..],
                ),
                SystemFontQuery::Fallback {
                    text,
                    weight,
                    italic,
                    locale,
                    presentation,
                    ..
                } => (
                    base.map_or("", |face| face.family.as_str()),
                    *weight,
                    *italic,
                    locale.as_str(),
                    true,
                    *presentation,
                    text.as_slice(),
                ),
                _ => return Vec::new(),
            };
            if !weight.is_finite() {
                return Vec::new();
            }
            if fallback && presentation == FontPresentation::Emoji {
                api.AddString(pattern.raw, c"family", "emoji");
                api.AddString(pattern.raw, c"lang", "und-zsye");
            }
            if !family.is_empty() && !api.AddString(pattern.raw, c"family", family) {
                return Vec::new();
            }
            if !locale.is_empty() && !api.AddString(pattern.raw, c"lang", locale) {
                return Vec::new();
            }
            (api.add_double)(pattern.raw, c"weight".as_ptr(), FontconfigWeight(weight));
            (api.add_integer)(pattern.raw, c"slant".as_ptr(), if italic { 100 } else { 0 });
            (api.add_bool)(pattern.raw, c"scalable".as_ptr(), 1);
            if fallback {
                (api.add_bool)(
                    pattern.raw,
                    c"color".as_ptr(),
                    c_int::from(presentation == FontPresentation::Emoji),
                );
            }
            let characters: Vec<u32> = char::decode_utf16(text.iter().copied())
                .filter_map(Result::ok)
                .map(u32::from)
                .filter(|&c| !matches!(c, 0x200c..=0x200d | 0xfe00..=0xfe0f | 0xe0100..=0xe01ef))
                .collect();
            if fallback {
                if characters.is_empty() {
                    return Vec::new();
                }
                let charset = (api.charset_new)();
                if charset.is_null() {
                    return Vec::new();
                }
                for character in &characters {
                    (api.charset_add)(charset, *character);
                }
                (api.add_charset)(pattern.raw, c"charset".as_ptr(), charset);
                (api.charset_destroy)(charset);
            }
            (api.substitute)(config, pattern.raw, 0); // FcMatchPattern
            (api.defaults)(pattern.raw);
            if fallback {
                let mut result = 0;
                let raw =
                    (api.font_sort)(config, pattern.raw, 1, std::ptr::null_mut(), &mut result);
                if raw.is_null() {
                    return Vec::new();
                }
                let set = OwnedFontSet { api, raw };
                let mut faces = Vec::new();
                for i in 0..(*set.raw).count.max(0) as usize {
                    let candidate = *(*set.raw).fonts.add(i);
                    let mut charset = std::ptr::null_mut();
                    if (api.get_charset)(candidate, c"charset".as_ptr(), 0, &mut charset) != 0
                        || charset.is_null()
                        || !characters
                            .iter()
                            .any(|&c| (api.charset_has)(charset, c) != 0)
                    {
                        continue;
                    }
                    let prepared = (api.render_prepare)(config, pattern.raw, candidate);
                    if prepared.is_null() {
                        continue;
                    }
                    let prepared = OwnedPattern { api, raw: prepared };
                    if let Some(mut face) = api.Face(prepared.raw) {
                        if !locale.is_empty() {
                            face.locales.push(locale.to_owned());
                        }
                        faces.push(face);
                    }
                }
                faces
            } else {
                let mut result = 0;
                let raw = (api.font_match)(config, pattern.raw, &mut result);
                if raw.is_null() {
                    return Vec::new();
                }
                let matched = OwnedPattern { api, raw };
                if !FamilyMatches(api, pattern.raw, matched.raw) {
                    return Vec::new();
                }
                api.Face(matched.raw).into_iter().collect()
            }
        }
    }
}

/// Skia's remove_weak accepts preferred weak aliases before the final strong
/// family binding, but excludes weak defaults appended after it. FcFontMatch
/// alone would silently turn a missing CSS family into the system default.
unsafe fn FamilyMatches(api: &Api, requested: Pattern, matched: Pattern) -> bool {
    let mut names = Vec::new();
    let mut last_strong = None;
    for index in 0..65536 {
        let mut value = Value {
            kind: 0,
            data: ValueData { integer: 0 },
        };
        let mut binding = 0;
        if (api.get_binding)(
            requested,
            c"family".as_ptr(),
            index,
            &mut value,
            &mut binding,
        ) != 0
        {
            break;
        }
        if value.kind != 3 || value.data.string.is_null() {
            continue;
        } // FcTypeString
        names.push(
            CStr::from_ptr(value.data.string.cast())
                .to_string_lossy()
                .into_owned(),
        );
        if binding != 0 {
            last_strong = Some(names.len());
        } // Strong or Same
    }
    let Some(limit) = last_strong else {
        return false;
    };
    for index in 0..65536 {
        let Some(name) = api.String(matched, c"family", index) else {
            break;
        };
        if names[..limit]
            .iter()
            .any(|n| n.to_lowercase() == name.to_lowercase())
        {
            return true;
        }
    }
    false
}

unsafe fn MatchUnique(api: &Api, config: Config, name: &str) -> Vec<HostFace> {
    if name.is_empty() {
        return Vec::new();
    }
    for object in [c"postscriptname", c"fullname"] {
        let raw = (api.pattern_new)();
        if raw.is_null() {
            continue;
        }
        let pattern = OwnedPattern { api, raw };
        if !api.AddString(pattern.raw, object, name) {
            return Vec::new();
        }
        (api.add_bool)(pattern.raw, c"scalable".as_ptr(), 1);
        (api.substitute)(config, pattern.raw, 0);
        (api.defaults)(pattern.raw);
        let mut result = 0;
        let raw = (api.font_match)(config, pattern.raw, &mut result);
        if raw.is_null() {
            continue;
        }
        let matched = OwnedPattern { api, raw };
        let exact = (0..65536)
            .map_while(|index| api.String(matched.raw, object, index))
            .any(|candidate| candidate.to_lowercase() == name.to_lowercase());
        if exact {
            return api.Face(matched.raw).into_iter().collect();
        }
    }
    Vec::new()
}

fn Generic(name: &str) -> &str {
    match name {
        "ui-serif" => "serif",
        "ui-sans-serif" => "sans-serif",
        "ui-monospace" => "monospace",
        "-apple-system" | "BlinkMacSystemFont" => "system-ui",
        _ => name,
    }
}
const WEIGHTS: &[(f64, f64)] = &[
    (0., 100.),
    (40., 200.),
    (50., 300.),
    (55., 350.),
    (75., 380.),
    (80., 400.),
    (100., 500.),
    (180., 600.),
    (200., 700.),
    (205., 800.),
    (210., 900.),
    (215., 1000.),
];
fn OpenTypeWeight(weight: f64) -> f64 {
    Interpolate(weight, false)
}
fn FontconfigWeight(weight: f64) -> f64 {
    Interpolate(weight, true)
}
fn Interpolate(value: f64, reverse: bool) -> f64 {
    for pair in WEIGHTS.windows(2) {
        let ((from, from_to), (to, to_to)) = if reverse {
            ((pair[0].1, pair[0].0), (pair[1].1, pair[1].0))
        } else {
            (pair[0], pair[1])
        };
        if value <= to {
            return from_to + (value.max(from) - from) / (to - from) * (to_to - from_to);
        }
    }
    if reverse {
        215.
    } else {
        1000.
    }
}
fn ParseVariations(value: &str) -> Vec<FontVariation> {
    value
        .split(',')
        .filter_map(|item| {
            let (tag, value) = item.trim().split_once('=')?;
            let tag: [u8; 4] = tag.trim().as_bytes().try_into().ok()?;
            let value: f32 = value.trim().parse().ok()?;
            value.is_finite().then_some(FontVariation {
                tag: u32::from_be_bytes(tag),
                value,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weight_roundtrips_and_variations_are_typed() {
        for css in [100., 300., 400., 450., 600., 700., 900., 1000.] {
            assert!((OpenTypeWeight(FontconfigWeight(css)) - css).abs() < 0.001);
        }
        let axes = ParseVariations("wght=625,wdth=87.5,invalid=3,opsz=NaN");
        assert_eq!(axes.len(), 2);
        assert_eq!(axes[0].value, 625.);
        assert_eq!(axes[1].tag, u32::from_be_bytes(*b"wdth"));
    }

    #[test]
    fn installed_fontconfig_queries_preserve_style_unique_name_and_context() {
        let mut host = LinuxFontHost::default();
        if Api::Get().is_none() {
            return;
        }
        let regular = host.Query(
            &SystemFontQuery::MatchGeneric {
                generic: "sans-serif".into(),
                weight: 400.,
                italic: false,
                locale: "en".into(),
            },
            None,
        );
        let Some(regular) = regular.first() else {
            return;
        }; // Fontless hosts remain supported.
        assert!(!regular.path.is_empty());
        let bold = host.Query(
            &SystemFontQuery::MatchFamily {
                family: regular.family.clone(),
                weight: 700.,
                italic: false,
                locale: "en".into(),
            },
            None,
        );
        assert!(!bold.is_empty());
        if !regular.unique_name.is_empty() {
            let unique = host.Query(
                &SystemFontQuery::MatchUniqueName {
                    name: regular.unique_name.clone(),
                },
                None,
            );
            assert_eq!(unique.len(), 1);
            assert_eq!(unique[0].path, regular.path);
            assert_eq!(unique[0].index, regular.index);
        }
        assert!(host
            .Query(
                &SystemFontQuery::MatchUniqueName {
                    name: "RechromDefinitelyMissingUniqueFont_78e262".into()
                },
                None
            )
            .is_empty());
        let fallback = host.Query(
            &SystemFontQuery::Fallback {
                text: "水😀".encode_utf16().collect(),
                weight: 400.,
                italic: false,
                locale: "zh-Hant".into(),
                presentation: FontPresentation::Emoji,
                base_face: None,
            },
            Some(regular),
        );
        for face in &fallback {
            assert_eq!(face.locales, ["zh-Hant"]);
            assert!(!face.path.is_empty());
        }
        // This is an installed-font smoke test, not a fixed-font/CJK assumption.
        println!(
            "host regular={} bold={} contextual_candidates={}",
            regular.family,
            bold[0].family,
            fallback.len()
        );
    }
}
