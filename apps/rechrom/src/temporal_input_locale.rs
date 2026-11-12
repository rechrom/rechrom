//! Native Locale formatter data, sampled by the embedding. Only immutable
//! format/symbol values cross Page; Interaction retains all field behavior.
//! Chromium locale_mac.mm:54-84,96-140,173-251,255-323.
use interaction::adapter::temporal_input_locale::TemporalInputLocaleData;
use std::{ffi::CStr, io};

#[cfg(target_vendor = "apple")]
pub(crate) fn current(ui_locale: &CStr) -> io::Result<Option<TemporalInputLocaleData>> {
    mac::current(ui_locale).map(Some)
}
#[cfg(target_os = "windows")]
pub(crate) fn current(ui_locale: &CStr) -> io::Result<Option<TemporalInputLocaleData>> {
    windows::current(ui_locale).map(Some)
}
#[cfg(all(not(target_vendor = "apple"), not(target_os = "windows")))]
pub(crate) fn current(ui_locale: &CStr) -> io::Result<Option<TemporalInputLocaleData>> {
    let data = icu_bidi::date_time_locale::current(ui_locale)?;
    Ok(Some(TemporalInputLocaleData {
        date_format: data.date_format,
        month_format: data.month_format,
        time_format: data.time_format,
        short_time_format: data.short_time_format,
        date_time_format: data.date_time_format,
        short_date_time_format: data.short_date_time_format,
        month_labels: data.month_labels,
        short_month_labels: data.short_month_labels,
        standalone_month_labels: data.standalone_month_labels,
        short_standalone_month_labels: data.short_standalone_month_labels,
        ampm_labels: data.ampm_labels,
        digits: data.digits,
        current_year: data.current_year,
    }))
}
#[cfg(target_os = "windows")]
#[path = "temporal_input_locale_windows.rs"]
mod windows;

#[cfg(target_vendor = "apple")]
mod mac {
    use super::*;
    use std::{
        ffi::{c_char, c_void},
        ptr,
    };
    type Ref = *const c_void;
    #[repr(C)]
    struct Range {
        location: isize,
        length: isize,
    }
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFRelease(value: Ref);
        fn CFStringCreateWithBytes(
            allocator: Ref,
            bytes: *const u8,
            count: isize,
            encoding: u32,
            external: u8,
        ) -> Ref;
        fn CFStringGetLength(value: Ref) -> isize;
        fn CFStringGetCharacters(value: Ref, range: Range, buffer: *mut u16);
        fn CFArrayGetCount(value: Ref) -> isize;
        fn CFArrayGetValueAtIndex(value: Ref, index: isize) -> Ref;
        fn CFLocaleCopyCurrent() -> Ref;
        fn CFLocaleGetIdentifier(locale: Ref) -> Ref;
        fn CFLocaleCreate(allocator: Ref, identifier: Ref) -> Ref;
        fn CFCalendarCreateWithIdentifier(allocator: Ref, identifier: Ref) -> Ref;
        fn CFCalendarSetLocale(calendar: Ref, locale: Ref);
        fn CFCalendarDecomposeAbsoluteTime(
            calendar: Ref,
            time: f64,
            components: *const c_char,
            ...
        ) -> u8;
        fn CFAbsoluteTimeGetCurrent() -> f64;
        fn CFTimeZoneCreateWithTimeIntervalFromGMT(allocator: Ref, seconds: f64) -> Ref;
        fn CFDateFormatterCreate(
            allocator: Ref,
            locale: Ref,
            date_style: isize,
            time_style: isize,
        ) -> Ref;
        fn CFDateFormatterSetProperty(formatter: Ref, key: Ref, value: Ref);
        fn CFDateFormatterGetFormat(formatter: Ref) -> Ref;
        fn CFDateFormatterCopyProperty(formatter: Ref, key: Ref) -> Ref;
        fn CFDateFormatterCreateDateFormatFromTemplate(
            allocator: Ref,
            template: Ref,
            options: usize,
            locale: Ref,
        ) -> Ref;
        fn CFNumberFormatterCreate(allocator: Ref, locale: Ref, style: isize) -> Ref;
        fn CFNumberFormatterCreateStringWithValue(
            allocator: Ref,
            formatter: Ref,
            kind: isize,
            value: Ref,
        ) -> Ref;
        static kCFGregorianCalendar: Ref;
        static kCFDateFormatterCalendar: Ref;
        static kCFDateFormatterTimeZone: Ref;
        static kCFDateFormatterMonthSymbols: Ref;
        static kCFDateFormatterShortMonthSymbols: Ref;
        static kCFDateFormatterStandaloneMonthSymbols: Ref;
        static kCFDateFormatterShortStandaloneMonthSymbols: Ref;
        static kCFDateFormatterAMSymbol: Ref;
        static kCFDateFormatterPMSymbol: Ref;
    }
    struct Owned(Ref);
    impl Owned {
        fn new(value: Ref) -> io::Result<Self> {
            if value.is_null() {
                Err(io::Error::other("native temporal locale value unavailable"))
            } else {
                Ok(Self(value))
            }
        }
    }
    impl Drop for Owned {
        fn drop(&mut self) {
            unsafe {
                CFRelease(self.0);
            }
        }
    }
    unsafe fn string(value: Ref) -> io::Result<String> {
        if value.is_null() {
            return Err(io::Error::other(
                "native temporal locale string unavailable",
            ));
        }
        let count = CFStringGetLength(value);
        let mut units = vec![0; count as usize];
        CFStringGetCharacters(
            value,
            Range {
                location: 0,
                length: count,
            },
            units.as_mut_ptr(),
        );
        String::from_utf16(&units)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }
    unsafe fn cfstring(value: &str) -> io::Result<Owned> {
        Owned::new(CFStringCreateWithBytes(
            ptr::null(),
            value.as_ptr(),
            value.len() as isize,
            0x08000100,
            0,
        ))
    }
    unsafe fn labels(formatter: Ref, key: Ref) -> io::Result<Vec<String>> {
        let array = Owned::new(CFDateFormatterCopyProperty(formatter, key))?;
        if CFArrayGetCount(array.0) != 12 {
            return Err(io::Error::other("native temporal month symbol count"));
        }
        (0..12)
            .map(|index| string(CFArrayGetValueAtIndex(array.0, index)))
            .collect()
    }
    pub(super) fn current(ui_locale: &CStr) -> io::Result<TemporalInputLocaleData> {
        unsafe {
            let current = Owned::new(CFLocaleCopyCurrent())?;
            let identifier = string(CFLocaleGetIdentifier(current.0))?;
            let language = |locale: &str| {
                locale
                    .split(['-', '_'])
                    .next()
                    .unwrap_or("")
                    .to_ascii_lowercase()
            };
            let requested = ui_locale
                .to_str()
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
            // LocaleMac::DetermineLocale preserves user date preferences when
            // the UI language matches the user's current locale language.
            let selected = if language(&identifier) == language(requested) {
                current
            } else {
                let identifier = cfstring(requested)?;
                Owned::new(CFLocaleCreate(ptr::null(), identifier.0))?
            };
            let calendar = Owned::new(CFCalendarCreateWithIdentifier(
                ptr::null(),
                kCFGregorianCalendar,
            ))?;
            CFCalendarSetLocale(calendar.0, selected.0);
            let utc = Owned::new(CFTimeZoneCreateWithTimeIntervalFromGMT(ptr::null(), 0.0))?;
            let formatter = |date, time| -> io::Result<Owned> {
                let formatter =
                    Owned::new(CFDateFormatterCreate(ptr::null(), selected.0, date, time))?;
                CFDateFormatterSetProperty(formatter.0, kCFDateFormatterCalendar, calendar.0);
                CFDateFormatterSetProperty(formatter.0, kCFDateFormatterTimeZone, utc.0);
                Ok(formatter)
            };
            let date = formatter(1, 0)?;
            let time = formatter(0, 2)?;
            let short_time = formatter(0, 1)?;
            let date_time = formatter(1, 2)?;
            let short_date_time = formatter(1, 1)?;
            let template = cfstring("yyyyMMMM")?;
            let month = Owned::new(CFDateFormatterCreateDateFormatFromTemplate(
                ptr::null(),
                template.0,
                0,
                selected.0,
            ))?;
            let am = Owned::new(CFDateFormatterCopyProperty(
                short_time.0,
                kCFDateFormatterAMSymbol,
            ))?;
            let pm = Owned::new(CFDateFormatterCopyProperty(
                short_time.0,
                kCFDateFormatterPMSymbol,
            ))?;
            let number = Owned::new(CFNumberFormatterCreate(ptr::null(), selected.0, 1))?;
            let mut digits = [0; 10];
            for (index, digit) in digits.iter_mut().enumerate() {
                let value = index as i32;
                let formatted = Owned::new(CFNumberFormatterCreateStringWithValue(
                    ptr::null(),
                    number.0,
                    3,
                    (&value as *const i32).cast(),
                ))?;
                let text = string(formatted.0)?;
                let mut chars = text.chars();
                *digit = chars
                    .next()
                    .filter(|_| chars.next().is_none())
                    .ok_or_else(|| io::Error::other("native temporal digit is not a scalar"))?
                    as u32;
            }
            let mut year = 0i32;
            if CFCalendarDecomposeAbsoluteTime(
                calendar.0,
                CFAbsoluteTimeGetCurrent(),
                c"y".as_ptr(),
                &mut year as *mut i32,
            ) == 0
                || !(1..=275760).contains(&year)
            {
                return Err(io::Error::other(
                    "native current Gregorian year unavailable",
                ));
            }
            let pattern = |formatter: Ref| -> io::Result<String> {
                Ok(string(CFDateFormatterGetFormat(formatter))?.replace('\u{202f}', " "))
            };
            Ok(TemporalInputLocaleData {
                date_format: pattern(date.0)?,
                month_format: string(month.0)?,
                time_format: pattern(time.0)?,
                short_time_format: pattern(short_time.0)?,
                date_time_format: pattern(date_time.0)?,
                short_date_time_format: pattern(short_date_time.0)?,
                month_labels: labels(date.0, kCFDateFormatterMonthSymbols)?,
                short_month_labels: labels(date.0, kCFDateFormatterShortMonthSymbols)?,
                standalone_month_labels: labels(date.0, kCFDateFormatterStandaloneMonthSymbols)?,
                short_standalone_month_labels: labels(
                    date.0,
                    kCFDateFormatterShortStandaloneMonthSymbols,
                )?,
                ampm_labels: [string(am.0)?, string(pm.0)?],
                digits,
                current_year: year as u32,
            })
        }
    }
}
