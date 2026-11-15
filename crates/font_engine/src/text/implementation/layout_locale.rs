// C++: third_party/blink/renderer/platform/text/layout_locale.h/.cc
// Partial mapping. The supplied source omits definitions for CaseMapLocale,
// GetQuotesData, LocaleWithBreakKeyword, SetHyphenationForTesting, and
// AcceptLanguagesChanged. The ICU UScriptCode type is also not connected.
use std::ffi::{c_char, CString};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, LazyLock, Mutex, OnceLock};

use foundation::{g_null_atom, AtomicString};


use super::hyphenation::Hyphenation;
use super::hyphenation_services::CurrentHyphenationResolver;

// cpp: third_party/blink/renderer/platform/text/layout_locale.h:26-26
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LineBreakStrictness {
    #[default]
    kDefault,
    kNormal,
    kStrict,
    kLoose,
}

// Raw values from icu_bidi/unicode/uscript.h. A typed UScriptCode connection
// remains pending; the values are retained here only for source behavior.
const USCRIPT_COMMON: i32 = 0;
const USCRIPT_ARABIC: i32 = 2;
const USCRIPT_HANGUL: i32 = 18;
const USCRIPT_HEBREW: i32 = 19;
const USCRIPT_LATIN: i32 = 25;
const USCRIPT_KATAKANA_OR_HIRAGANA: i32 = 54;
const USCRIPT_SIMPLIFIED_HAN: i32 = 73;
const USCRIPT_TRADITIONAL_HAN: i32 = 74;

// cpp: third_party/blink/renderer/platform/text/layout_locale.cc:19-24
fn LowerAscii(value: String) -> String {
    value.to_ascii_lowercase()
}

// cpp: third_party/blink/renderer/platform/text/layout_locale.cc:26-29
fn Language(locale: &str) -> String {
    let separator = locale.find(['-', '_']).unwrap_or(locale.len());
    LowerAscii(locale[..separator].to_owned())
}

// cpp: third_party/blink/renderer/platform/text/layout_locale.cc:31-49
fn ScriptForLocale(locale: &str) -> i32 {
    let lower = LowerAscii(locale.to_owned());
    if lower.contains("hant") {
        return USCRIPT_TRADITIONAL_HAN;
    }
    if lower.contains("hans") {
        return USCRIPT_SIMPLIFIED_HAN;
    }
    let language = Language(&lower);
    if language == "ja" {
        return USCRIPT_KATAKANA_OR_HIRAGANA;
    }
    if language == "ko" {
        return USCRIPT_HANGUL;
    }
    if language == "zh" || language == "cmn" || language == "yue" {
        return USCRIPT_SIMPLIFIED_HAN;
    }
    if language == "ar" {
        return USCRIPT_ARABIC;
    }
    if language == "he" || language == "iw" {
        return USCRIPT_HEBREW;
    }
    USCRIPT_LATIN
}

// cpp: third_party/blink/renderer/platform/text/layout_locale.cc:51-60
static REGISTRY: LazyLock<Mutex<Vec<Arc<LayoutLocale>>>> = LazyLock::new(|| Mutex::new(Vec::new()));

// cpp: third_party/blink/renderer/platform/text/layout_locale.h:33-114
pub struct LayoutLocale {
    string_: AtomicString,
    string_for_sk_font_mgr_: OnceLock<CString>,
    #[allow(dead_code)]
    harfbuzz_language_: Option<harfrust::Language>,
    script_: i32,
    script_for_han_: AtomicI32,
    has_script_for_han_: AtomicBool,
    is_macrolanguage_chinese_computed_: AtomicBool,
    is_macrolanguage_chinese_: AtomicBool,
}

#[allow(non_snake_case)]
impl LayoutLocale {
    // cpp: third_party/blink/renderer/platform/text/layout_locale.cc:62-66
    fn new(locale: &AtomicString) -> Self {
        let ascii = locale.Ascii();
        // hb_language_from_string
        let harfbuzz_language = harfrust::Language::new(ascii.as_bytes());
        Self {
            string_: locale.clone(),
            string_for_sk_font_mgr_: OnceLock::new(),
            harfbuzz_language_: harfbuzz_language,
            script_: ScriptForLocale(&ascii),
            script_for_han_: AtomicI32::new(USCRIPT_COMMON),
            has_script_for_han_: AtomicBool::new(false),
            is_macrolanguage_chinese_computed_: AtomicBool::new(false),
            is_macrolanguage_chinese_: AtomicBool::new(false),
        }
    }

    // cpp: third_party/blink/renderer/platform/text/layout_locale.cc:68-82
    pub fn Get(locale: &AtomicString) -> *const Self {
        Self::GetShared(locale).map_or(std::ptr::null(), |entry| Arc::as_ptr(&entry))
    }

    // Rust ownership bridge for the source's scoped_refptr<LayoutLocale>.
    pub fn GetShared(locale: &AtomicString) -> Option<Arc<Self>> {
        if locale.IsNull() {
            return None;
        }
        let mut registry = REGISTRY.lock().expect("locale registry poisoned");
        let requested = LowerAscii(locale.Ascii());
        for entry in registry.iter() {
            if LowerAscii(entry.LocaleStringValue().Ascii()) == requested {
                return Some(Arc::clone(entry));
            }
        }
        let entry = Arc::new(Self::new(locale));
        registry.push(Arc::clone(&entry));
        Some(entry)
    }

    // cpp: third_party/blink/renderer/platform/text/layout_locale.cc:84-86
    pub fn GetDefault() -> *const Self {
        Self::Get(&AtomicString::from_str("en"))
    }

    // cpp: third_party/blink/renderer/platform/text/layout_locale.cc:88-90
    pub fn GetSystem() -> *const Self {
        Self::GetDefault()
    }

    // cpp: third_party/blink/renderer/platform/text/layout_locale.h:39-41
    pub fn ValueOrDefault(locale: *const Self) -> *const Self {
        if locale.is_null() {
            Self::GetDefault()
        } else {
            locale
        }
    }

    // cpp: third_party/blink/renderer/platform/text/layout_locale.h:43-45
    pub fn LocaleStringValue(&self) -> &AtomicString {
        &self.string_
    }

    // cpp: third_party/blink/renderer/platform/text/layout_locale.h:46-48
    // SAFETY: a non-null locale must remain alive for the returned borrow.
    pub unsafe fn LocaleString<'a>(locale: *const Self) -> &'a AtomicString {
        if locale.is_null() {
            &g_null_atom
        } else {
            &(*locale).string_
        }
    }

    // cpp: third_party/blink/renderer/platform/text/layout_locale.h:50-51
    pub fn Ascii(&self) -> String {
        self.string_.Ascii()
    }

    // cpp: third_party/blink/renderer/platform/text/layout_locale.h:53-55
    // The shapers currently set the buffer language from the locale tag
    // string, so this cached value has no caller yet.
    #[allow(dead_code)]
    pub(crate) fn HarfbuzzLanguage(&self) -> Option<&harfrust::Language> {
        self.harfbuzz_language_.as_ref()
    }

    // cpp: third_party/blink/renderer/platform/text/layout_locale.h:55-56
    pub fn GetScript(&self) -> i32 {
        self.script_
    }

    // cpp: third_party/blink/renderer/platform/text/layout_locale.cc:92-95
    pub fn CreateForTesting(locale: &AtomicString) -> Arc<Self> {
        Arc::new(Self::new(locale))
    }

    // cpp: third_party/blink/renderer/platform/text/layout_locale.cc:97-117
    fn ComputeScriptForHan(&self) {
        let (script, has_script) = match self.script_ {
            USCRIPT_KATAKANA_OR_HIRAGANA => (USCRIPT_KATAKANA_OR_HIRAGANA, true),
            USCRIPT_HANGUL => (USCRIPT_HANGUL, true),
            USCRIPT_SIMPLIFIED_HAN | USCRIPT_TRADITIONAL_HAN => (self.script_, true),
            _ => (USCRIPT_SIMPLIFIED_HAN, false),
        };
        self.script_for_han_.store(script, Ordering::Relaxed);
        self.has_script_for_han_
            .store(has_script, Ordering::Relaxed);
    }

    // cpp: third_party/blink/renderer/platform/text/layout_locale.cc:119-123
    pub fn GetScriptForHanRaw(&self) -> i32 {
        if self.script_for_han_.load(Ordering::Relaxed) == USCRIPT_COMMON {
            self.ComputeScriptForHan();
        }
        self.script_for_han_.load(Ordering::Relaxed)
    }

    // cpp: third_party/blink/renderer/platform/text/layout_locale.cc:125-129
    pub fn HasScriptForHan(&self) -> bool {
        if self.script_for_han_.load(Ordering::Relaxed) == USCRIPT_COMMON {
            self.ComputeScriptForHan();
        }
        self.has_script_for_han_.load(Ordering::Relaxed)
    }

    // cpp: third_party/blink/renderer/platform/text/layout_locale.cc:131-134
    pub fn LocaleForHan(locale: *const Self) -> *const Self {
        if !locale.is_null() && unsafe { (&*locale).HasScriptForHan() } {
            locale
        } else {
            Self::GetDefault()
        }
    }

    // cpp: third_party/blink/renderer/platform/text/layout_locale.cc:136-140
    pub fn LocaleForSkFontMgr(&self) -> *const c_char {
        self.string_for_sk_font_mgr_
            .get_or_init(|| CString::new(Language(&self.Ascii())).expect("locale contains NUL"))
            .as_ptr()
    }

    // cpp: third_party/blink/renderer/platform/text/layout_locale.cc:142-149
    pub fn LocaleForHanForSkFontMgr(&self) -> *const c_char {
        match self.GetScriptForHanRaw() {
            USCRIPT_KATAKANA_OR_HIRAGANA => c"ja".as_ptr(),
            USCRIPT_HANGUL => c"ko".as_ptr(),
            USCRIPT_TRADITIONAL_HAN => c"zh-Hant".as_ptr(),
            _ => c"zh-Hans".as_ptr(),
        }
    }

    // cpp: third_party/blink/renderer/platform/text/layout_locale.cc:151-154
    pub fn GetHyphenation(&self) -> *mut Hyphenation {
        let Some(mut resolver) = CurrentHyphenationResolver() else {
            return std::ptr::null_mut();
        };
        unsafe { resolver.as_mut().Resolve(&self.string_) }
    }

    // cpp: third_party/blink/renderer/platform/text/layout_locale.cc:156-169
    fn IsMacrolanguageChineseSlow(&self) -> bool {
        const CHINESE_LANGUAGES: [&str; 19] = [
            "cdo", "cjy", "cmn", "cnp", "cpx", "csp", "czh", "czo", "gan", "hak", "hnm", "hsn",
            "luh", "lzh", "mnp", "nan", "sjc", "wuu", "yue",
        ];
        let language = Language(&self.Ascii());
        self.is_macrolanguage_chinese_computed_
            .store(true, Ordering::Relaxed);
        let is_chinese = CHINESE_LANGUAGES.contains(&language.as_str()) || language == "zh";
        self.is_macrolanguage_chinese_
            .store(is_chinese, Ordering::Relaxed);
        is_chinese
    }

    // cpp: third_party/blink/renderer/platform/text/layout_locale.h:66-70
    pub fn IsMacrolanguageChinese(&self) -> bool {
        if self
            .is_macrolanguage_chinese_computed_
            .load(Ordering::Relaxed)
        {
            self.is_macrolanguage_chinese_.load(Ordering::Relaxed)
        } else {
            self.IsMacrolanguageChineseSlow()
        }
    }

    // cpp: third_party/blink/renderer/platform/text/layout_locale.cc:171-175
    pub fn ClearForTesting() {
        REGISTRY.lock().expect("locale registry poisoned").clear();
    }
}

// cpp: third_party/blink/renderer/platform/text/layout_locale.h:42-44
impl PartialEq for LayoutLocale {
    fn eq(&self, other: &Self) -> bool {
        self.string_ == other.string_
    }
}
