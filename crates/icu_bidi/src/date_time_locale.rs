//! Immutable LocaleIcu formatter/symbol data for embedding-owned form controls.
//! Chromium platform/text/locale_icu.cc:166-218,241-276,309-431,454-490.
use std::{
    ffi::{c_char, c_void, CStr},
    io, ptr,
};

pub struct DateTimeLocaleData {
    pub date_format: String,
    pub month_format: String,
    pub time_format: String,
    pub short_time_format: String,
    pub date_time_format: String,
    pub short_date_time_format: String,
    pub month_labels: Vec<String>,
    pub short_month_labels: Vec<String>,
    pub standalone_month_labels: Vec<String>,
    pub short_standalone_month_labels: Vec<String>,
    pub ampm_labels: [String; 2],
    pub digits: [u32; 10],
    pub current_year: u32,
}
const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const SHORT_MONTH_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
unsafe extern "C" {
    #[link_name=concat!("udat_open_",env!("ICU_MAJOR"))]
    fn udat_open(
        time: i32,
        date: i32,
        locale: *const c_char,
        zone: *const u16,
        zone_len: i32,
        pattern: *const u16,
        pattern_len: i32,
        status: *mut i32,
    ) -> *mut c_void;
    #[link_name=concat!("udat_close_",env!("ICU_MAJOR"))]
    fn udat_close(format: *mut c_void);
    #[link_name=concat!("udat_toPattern_",env!("ICU_MAJOR"))]
    fn udat_toPattern(
        format: *const c_void,
        localized: i8,
        result: *mut u16,
        capacity: i32,
        status: *mut i32,
    ) -> i32;
    #[link_name=concat!("udat_countSymbols_",env!("ICU_MAJOR"))]
    fn udat_countSymbols(format: *const c_void, kind: i32) -> i32;
    #[link_name=concat!("udat_getSymbols_",env!("ICU_MAJOR"))]
    fn udat_getSymbols(
        format: *const c_void,
        kind: i32,
        index: i32,
        result: *mut u16,
        capacity: i32,
        status: *mut i32,
    ) -> i32;
    #[link_name=concat!("udat_setContext_",env!("ICU_MAJOR"))]
    fn udat_setContext(format: *mut c_void, context: i32, status: *mut i32);
    #[link_name=concat!("udat_format_",env!("ICU_MAJOR"))]
    fn udat_format(
        format: *const c_void,
        date: f64,
        result: *mut u16,
        capacity: i32,
        position: *mut c_void,
        status: *mut i32,
    ) -> i32;
    #[link_name=concat!("udatpg_open_",env!("ICU_MAJOR"))]
    fn udatpg_open(locale: *const c_char, status: *mut i32) -> *mut c_void;
    #[link_name=concat!("udatpg_close_",env!("ICU_MAJOR"))]
    fn udatpg_close(generator: *mut c_void);
    #[link_name=concat!("udatpg_getBestPattern_",env!("ICU_MAJOR"))]
    fn udatpg_getBestPattern(
        generator: *mut c_void,
        skeleton: *const u16,
        length: i32,
        result: *mut u16,
        capacity: i32,
        status: *mut i32,
    ) -> i32;
    #[link_name=concat!("unum_open_",env!("ICU_MAJOR"))]
    fn unum_open(
        style: i32,
        pattern: *const u16,
        length: i32,
        locale: *const c_char,
        parse_error: *mut c_void,
        status: *mut i32,
    ) -> *mut c_void;
    #[link_name=concat!("unum_close_",env!("ICU_MAJOR"))]
    fn unum_close(format: *mut c_void);
    #[link_name=concat!("unum_getSymbol_",env!("ICU_MAJOR"))]
    fn unum_getSymbol(
        format: *const c_void,
        symbol: i32,
        result: *mut u16,
        capacity: i32,
        status: *mut i32,
    ) -> i32;
    #[link_name=concat!("ucal_open_",env!("ICU_MAJOR"))]
    fn ucal_open(
        zone: *const u16,
        zone_len: i32,
        locale: *const c_char,
        kind: i32,
        status: *mut i32,
    ) -> *mut c_void;
    #[link_name=concat!("ucal_close_",env!("ICU_MAJOR"))]
    fn ucal_close(calendar: *mut c_void);
    #[link_name=concat!("ucal_get_",env!("ICU_MAJOR"))]
    fn ucal_get(calendar: *const c_void, field: i32, status: *mut i32) -> i32;
}
struct Owned(*mut c_void, unsafe extern "C" fn(*mut c_void));
impl Owned {
    fn new(
        pointer: *mut c_void,
        close: unsafe extern "C" fn(*mut c_void),
        status: i32,
    ) -> io::Result<Self> {
        let owned = Self(pointer, close);
        if pointer.is_null() || status > 0 {
            Err(error(status))
        } else {
            Ok(owned)
        }
    }
}
impl Drop for Owned {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { (self.1)(self.0) }
        }
    }
}
fn error(status: i32) -> io::Error {
    io::Error::other(format!("ICU temporal locale unavailable: {status}"))
}
fn string(mut get: impl FnMut(*mut u16, i32, *mut i32) -> i32) -> io::Result<String> {
    let mut status = 0;
    let length = get(ptr::null_mut(), 0, &mut status);
    if length < 0 || (status > 0 && status != 15) {
        return Err(error(status));
    }
    let mut result = vec![0; length as usize];
    status = 0;
    let written = get(result.as_mut_ptr(), length, &mut status);
    if status > 0 || written != length {
        return Err(error(status));
    }
    String::from_utf16(&result).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}
fn open(locale: &CStr, time: i32, date: i32) -> io::Result<Owned> {
    let mut status = 0;
    let zone = [b'G' as u16, b'M' as u16, b'T' as u16];
    unsafe {
        Owned::new(
            udat_open(
                time,
                date,
                locale.as_ptr(),
                zone.as_ptr(),
                3,
                ptr::null(),
                -1,
                &mut status,
            ),
            udat_close,
            status,
        )
    }
}
fn pattern(format: &Owned) -> io::Result<String> {
    string(|result, capacity, status| unsafe {
        udat_toPattern(format.0, 1, result, capacity, status)
    })
}
fn labels(format: &Owned, kind: i32, count: i32, standalone: bool) -> io::Result<Vec<String>> {
    if unsafe { udat_countSymbols(format.0, kind) } != count {
        return Err(error(1));
    }
    (0..count)
        .map(|index| {
            string(|result, capacity, status| unsafe {
                if standalone {
                    // LocaleIcu works around ICU #11552 by formatting a date in each
                    // month with LLLL/LLL and standalone capitalization context.
                    udat_format(
                        format.0,
                        (15.0 + index as f64 * 30.0) * 86_400_000.0,
                        result,
                        capacity,
                        ptr::null_mut(),
                        status,
                    )
                } else {
                    udat_getSymbols(format.0, kind, index, result, capacity, status)
                }
            })
        })
        .collect()
}
fn standalone(locale: &CStr, short: bool) -> io::Result<Vec<String>> {
    let mut status = 0;
    let pattern = [b'L' as u16; 4];
    let format = unsafe {
        Owned::new(
            udat_open(
                -2,
                -2,
                locale.as_ptr(),
                ptr::null(),
                -1,
                pattern.as_ptr(),
                if short { 3 } else { 4 },
                &mut status,
            ),
            udat_close,
            status,
        )?
    };
    unsafe { udat_setContext(format.0, 0x104, &mut status) };
    if status > 0 {
        return Err(error(status));
    }
    labels(&format, if short { 11 } else { 10 }, 12, true)
}
/// Match LocaleIcu's date/time styles, yyyyMMMM skeleton, decimal symbols,
/// and standalone month formatting. No process locale or ISO approximation.
pub fn current(locale: &CStr) -> io::Result<DateTimeLocaleData> {
    let date = open(locale, -1, 3)?;
    let time = open(locale, 2, -1)?;
    let short_time = open(locale, 3, -1)?;
    let date_time = open(locale, 2, 3)?;
    let short_date_time = open(locale, 3, 3)?;
    let mut status = 0;
    let generator = unsafe {
        Owned::new(
            udatpg_open(locale.as_ptr(), &mut status),
            udatpg_close,
            status,
        )?
    };
    let skeleton: Vec<u16> = "yyyyMMMM".encode_utf16().collect();
    let month_format = string(|result, capacity, status| unsafe {
        udatpg_getBestPattern(
            generator.0,
            skeleton.as_ptr(),
            skeleton.len() as i32,
            result,
            capacity,
            status,
        )
    })?;
    status = 0;
    let number = unsafe {
        Owned::new(
            unum_open(
                1,
                ptr::null(),
                0,
                locale.as_ptr(),
                ptr::null_mut(),
                &mut status,
            ),
            unum_close,
            status,
        )?
    };
    let mut digits = [0; 10];
    for (index, digit) in digits.iter_mut().enumerate() {
        let symbol = string(|result, capacity, status| unsafe {
            unum_getSymbol(
                number.0,
                if index == 0 { 4 } else { 17 + index as i32 },
                result,
                capacity,
                status,
            )
        })?;
        let mut chars = symbol.chars();
        *digit = chars
            .next()
            .filter(|_| chars.next().is_none())
            .ok_or_else(|| error(1))? as u32;
    }
    status = 0;
    // DateTimeNumericFieldElement uses the local current Gregorian year,
    // independently of the locale's preferred display calendar.
    let calendar = unsafe {
        Owned::new(
            ucal_open(ptr::null(), -1, c"".as_ptr(), 1, &mut status),
            ucal_close,
            status,
        )?
    };
    let year = unsafe { ucal_get(calendar.0, 1, &mut status) };
    if status > 0 || !(1..=275760).contains(&year) {
        return Err(error(status));
    }
    // LocaleIcu requires exactly twelve symbols. Calendars with a thirteenth
    // month use the source Locale fallback labels, not a truncated symbol set.
    let month_labels =
        labels(&date, 1, 12, false).unwrap_or_else(|_| MONTH_NAMES.map(str::to_owned).into());
    let short_month_labels =
        labels(&date, 2, 12, false).unwrap_or_else(|_| SHORT_MONTH_NAMES.map(str::to_owned).into());
    let standalone_month_labels =
        standalone(locale, false).unwrap_or_else(|_| month_labels.clone());
    let short_standalone_month_labels =
        standalone(locale, true).unwrap_or_else(|_| short_month_labels.clone());
    let ampm =
        labels(&time, 5, 2, false).unwrap_or_else(|_| vec!["AM".to_owned(), "PM".to_owned()]);
    Ok(DateTimeLocaleData {
        date_format: pattern(&date)?,
        month_format,
        time_format: pattern(&time)?,
        short_time_format: pattern(&short_time)?,
        date_time_format: pattern(&date_time)?,
        short_date_time_format: pattern(&short_date_time)?,
        month_labels,
        short_month_labels,
        standalone_month_labels,
        short_standalone_month_labels,
        ampm_labels: [ampm[0].clone(), ampm[1].clone()],
        digits,
        current_year: year as u32,
    })
}
