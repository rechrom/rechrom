//! Embedding-owned UI locale and WebPreferences editing behavior.
//! Source RendererBlinkPlatformImpl::DefaultLocale, cc:416-418;
//! text_break_iterator_internal_icu.cc:30-44; web_preferences.h:147-160.
//! These are application/platform settings, never inferred from document lang.
use interaction::default::core::editing::editing_behavior::EditingBehaviorType;
use std::{ffi::CString, io};
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct InteractionTextSettings {
    pub editing_behavior: EditingBehaviorType,
    pub smart_insert_delete_enabled: bool,
    pub ui_locale: CString,
    pub temporal_locale:
        Option<interaction::adapter::temporal_input_locale::TemporalInputLocaleData>,
}
pub(crate) fn current() -> io::Result<InteractionTextSettings> {
    let ui_locale = platform_locale()?;
    // Existing searchers retain their construction locale. Preference refresh
    // may update date formats, but changing the process UI language requires a
    // restart instead of mutating ICU underneath live workers.
    icu_bidi::string_search::InitializeDefaultLocale(&ui_locale)?;
    let temporal_locale = crate::temporal_input_locale::current(&ui_locale)?;
    Ok(InteractionTextSettings {
        editing_behavior: editing_behavior(),
        // web_preferences.h:189. Independent of document or emulated editing behavior.
        smart_insert_delete_enabled: cfg!(target_os = "macos"),
        ui_locale,
        temporal_locale,
    })
}
fn editing_behavior() -> EditingBehaviorType {
    // ChromeOS is an embedding build flavor on Rust's Linux target, matching
    // Chromium's IS_CHROMEOS branch before the generic IS_POSIX branch.
    if cfg!(target_vendor = "apple") {
        EditingBehaviorType::Mac
    } else if cfg!(target_os = "windows") {
        EditingBehaviorType::Windows
    } else if cfg!(target_os = "android") {
        EditingBehaviorType::Android
    } else if cfg!(feature = "chromeos") {
        EditingBehaviorType::ChromeOS
    } else if cfg!(unix) {
        EditingBehaviorType::Unix
    } else {
        EditingBehaviorType::Mac
    }
}
/// Adopt the process's configured message locale before starting host/worker
/// threads. LC_MESSAGES leaves numeric/date formatting used by engines alone.
pub(crate) fn initialize() -> io::Result<()> {
    #[cfg(all(unix, not(target_vendor = "apple"), not(target_os = "android")))]
    unsafe {
        if setlocale(6, c"".as_ptr()).is_null() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "configured LC_MESSAGES is unavailable",
            ));
        }
    }
    icu_bidi::string_search::InitializeDefaultLocale(&platform_locale()?)
}
#[cfg(target_vendor = "apple")]
fn platform_locale() -> io::Result<CString> {
    use std::ffi::{c_char, c_void};
    type Ref = *const c_void;
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFLocaleCopyPreferredLanguages() -> Ref;
        fn CFLocaleCopyCurrent() -> Ref;
        fn CFLocaleGetIdentifier(locale: Ref) -> Ref;
        fn CFArrayGetCount(array: Ref) -> isize;
        fn CFArrayGetValueAtIndex(array: Ref, index: isize) -> Ref;
        fn CFStringGetLength(string: Ref) -> isize;
        fn CFStringGetMaximumSizeForEncoding(length: isize, encoding: u32) -> isize;
        fn CFStringGetCString(string: Ref, buffer: *mut c_char, size: isize, encoding: u32) -> u8;
        fn CFRelease(value: Ref);
    }
    struct Owned(Ref);
    impl Drop for Owned {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe {
                    CFRelease(self.0);
                }
            }
        }
    }
    unsafe fn utf8(string: Ref) -> io::Result<CString> {
        if string.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "platform UI locale absent",
            ));
        }
        let length = CFStringGetMaximumSizeForEncoding(CFStringGetLength(string), 0x08000100);
        if length < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "platform UI locale encoding",
            ));
        }
        let mut bytes = vec![0u8; length as usize + 1];
        if CFStringGetCString(
            string,
            bytes.as_mut_ptr().cast(),
            bytes.len() as isize,
            0x08000100,
        ) == 0
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "platform UI locale UTF-8 conversion",
            ));
        }
        Ok(std::ffi::CStr::from_ptr(bytes.as_ptr().cast()).to_owned())
    }
    unsafe {
        let languages = Owned(CFLocaleCopyPreferredLanguages());
        if !languages.0.is_null() && CFArrayGetCount(languages.0) > 0 {
            return utf8(CFArrayGetValueAtIndex(languages.0, 0));
        }
        let locale = Owned(CFLocaleCopyCurrent());
        if locale.0.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "platform locale absent",
            ));
        }
        utf8(CFLocaleGetIdentifier(locale.0))
    }
}
#[cfg(target_os = "windows")]
fn platform_locale() -> io::Result<CString> {
    // Preferred UI languages are distinct from the user's date/number locale.
    #[link(name = "kernel32")]
    extern "system" {
        fn GetUserPreferredUILanguages(
            flags: u32,
            count: *mut u32,
            buffer: *mut u16,
            size: *mut u32,
        ) -> i32;
        fn GetUserDefaultUILanguage() -> u16;
        fn LCIDToLocaleName(locale: u32, name: *mut u16, size: i32, flags: u32) -> i32;
    }
    let mut count = 0;
    let mut length = 0;
    unsafe {
        if GetUserPreferredUILanguages(8, &mut count, std::ptr::null_mut(), &mut length) != 0
            && length > 0
        {
            let mut buffer = vec![0u16; length as usize];
            if GetUserPreferredUILanguages(8, &mut count, buffer.as_mut_ptr(), &mut length) != 0
                && count > 0
            {
                let end = buffer
                    .iter()
                    .position(|unit| *unit == 0)
                    .unwrap_or(buffer.len());
                let locale = String::from_utf16(&buffer[..end])
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
                return CString::new(locale)
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error));
            }
        }
        let mut buffer = [0u16; 85];
        let length = LCIDToLocaleName(
            GetUserDefaultUILanguage() as u32,
            buffer.as_mut_ptr(),
            85,
            0,
        );
        if length <= 0 {
            return Err(io::Error::last_os_error());
        }
        let locale = String::from_utf16(&buffer[..length as usize - 1])
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        CString::new(locale).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }
}
#[cfg(target_os = "android")]
fn platform_locale() -> io::Result<CString> {
    extern "C" {
        fn __system_property_get(
            name: *const std::ffi::c_char,
            value: *mut std::ffi::c_char,
        ) -> i32;
    }
    for property in [c"persist.sys.locale", c"ro.product.locale"] {
        let mut buffer = [0 as std::ffi::c_char; 92];
        if unsafe { __system_property_get(property.as_ptr(), buffer.as_mut_ptr()) } > 0 {
            return Ok(unsafe { std::ffi::CStr::from_ptr(buffer.as_ptr()) }.to_owned());
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "Android UI locale service has no configured locale",
    ))
}
#[cfg(all(unix, not(target_vendor = "apple"), not(target_os = "android")))]
extern "C" {
    fn setlocale(category: i32, locale: *const std::ffi::c_char) -> *mut std::ffi::c_char;
}
#[cfg(all(unix, not(target_vendor = "apple"), not(target_os = "android")))]
fn platform_locale() -> io::Result<CString> {
    let value = unsafe { setlocale(6, std::ptr::null()) };
    if value.is_null() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "LC_MESSAGES locale unavailable",
        ));
    }
    let locale = unsafe { std::ffi::CStr::from_ptr(value) }
        .to_str()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    // POSIX codeset names are not part of ICU's locale ID. Keep any locale
    // modifier; C/POSIX selects ICU root rather than inventing an English UI.
    let (name, modifier) = locale
        .split_once('@')
        .map_or((locale, None), |(name, modifier)| (name, Some(modifier)));
    let language = name.split('.').next().unwrap_or(name);
    let language = if matches!(language, "C" | "POSIX") {
        String::new()
    } else if let Some(modifier) = modifier {
        format!("{language}@{modifier}")
    } else {
        language.to_owned()
    };
    CString::new(language).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}
#[cfg(not(any(unix, target_os = "windows")))]
fn platform_locale() -> io::Result<CString> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "platform UI locale service is not available",
    ))
}
