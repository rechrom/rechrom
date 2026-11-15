//! Immutable LocaleIcu formatter/symbol data for embedding-owned form controls,
//! from ICU4X compiled CLDR data.
//! Chromium platform/text/locale_icu.cc:166-218,241-276,309-431,454-490.
//!
//! LocaleIcu reads patterns from `udat_open` styles and `udatpg` skeletons;
//! here the same field sets and lengths resolve through ICU4X, and the
//! resolved LDML pattern string is what is returned.
use std::{ffi::CStr, io};

use icu_calendar::Gregorian;
use icu_datetime::fieldsets::{T, YM, YMD, YMDT};
use icu_datetime::input::{Date, DateTime, Time};
use icu_datetime::options::TimePrecision;
use icu_datetime::pattern::{DateTimePattern, FixedCalendarDateTimeNames};
use icu_datetime::{DateTimeFormatterPreferences, FixedCalendarDateTimeFormatter};
use icu_decimal::input::Decimal;
use icu_decimal::{DecimalFormatter, DecimalFormatterPreferences};
use icu_locale_core::Locale;
use writeable::{TryWriteable, Writeable};

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

fn error(message: impl std::fmt::Display) -> io::Error {
    io::Error::other(format!("ICU temporal locale unavailable: {message}"))
}

fn locale(name: &CStr) -> Locale {
    let name = name.to_string_lossy();
    let name = name.split(['@', '.']).next().unwrap_or("").replace('_', "-");
    name.parse().unwrap_or(Locale::UNKNOWN)
}

fn sample_date() -> Date<Gregorian> {
    Date::try_new_gregorian(2000, 1, 15).expect("valid date")
}

macro_rules! resolved_pattern {
    ($prefs:expr, $field_set:expr, $input:expr) => {{
        let formatter = FixedCalendarDateTimeFormatter::<Gregorian, _>::try_new($prefs, $field_set)
            .map_err(error)?;
        let pattern = formatter.format($input).pattern();
        io::Result::Ok(pattern.write_to_string().into_owned())
    }};
}

/// Formats each input with an explicit pattern such as "MMMM", "LLL" or "a".
macro_rules! format_names {
    ($prefs:expr, $pattern:expr, $inputs:expr) => {{
        let pattern: DateTimePattern = $pattern.parse().map_err(error)?;
        let mut names = FixedCalendarDateTimeNames::<Gregorian>::try_new($prefs).map_err(error)?;
        let formatter = names.include_for_pattern(&pattern).map_err(error)?;
        $inputs
            .iter()
            .map(|input| {
                formatter
                    .format(input)
                    .try_write_to_string()
                    .map(|value| value.into_owned())
                    .map_err(|(failure, _)| error(failure))
            })
            .collect::<io::Result<Vec<String>>>()
    }};
}

fn months(prefs: DateTimeFormatterPreferences, pattern: &str) -> io::Result<Vec<String>> {
    let noon = Time::try_new(12, 0, 0, 0).map_err(error)?;
    let dates: Vec<DateTime<Gregorian>> = (1..=12)
        .map(|month| DateTime {
            date: Date::try_new_gregorian(2000, month, 15).expect("valid date"),
            time: noon,
        })
        .collect();
    format_names!(prefs, pattern, dates)
}

fn day_periods(prefs: DateTimeFormatterPreferences) -> io::Result<Vec<String>> {
    let times = [9, 21]
        .map(|hour| Time::try_new(hour, 0, 0, 0).map(|time| DateTime { date: sample_date(), time }));
    let times = [times[0].map_err(error)?, times[1].map_err(error)?];
    format_names!(prefs, "a", times)
}

/// The current Gregorian year (UTC), as DateTimeNumericFieldElement uses it.
fn current_year() -> io::Result<u32> {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(error)?
        .as_secs() as i64;
    // Civil-from-days (Howard Hinnant), days since 1970-01-01.
    let days = seconds.div_euclid(86_400) + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days - era * 146_097;
    let year_of_era = (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let month = if month_index < 10 { month_index + 3 } else { month_index - 9 };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    u32::try_from(year).map_err(error)
}

/// Match LocaleIcu's date/time styles, yyyyMMMM skeleton, decimal symbols,
/// and standalone month formatting.
pub fn current(locale_name: &CStr) -> io::Result<DateTimeLocaleData> {
    let locale = locale(locale_name);
    let prefs = DateTimeFormatterPreferences::from(&locale);
    let date = sample_date();
    let time = Time::try_new(13, 45, 30, 0).map_err(error)?;
    let date_time = DateTime { date, time };

    // udat_open(UDAT_NONE, UDAT_SHORT) and friends: SHORT date, MEDIUM time
    // (with seconds), SHORT time (minutes).
    let date_format = resolved_pattern!(prefs, YMD::short(), &date)?;
    let time_format = resolved_pattern!(
        prefs,
        T::medium().with_time_precision(TimePrecision::Second),
        &time
    )?;
    let short_time_format = resolved_pattern!(
        prefs,
        T::short().with_time_precision(TimePrecision::Minute),
        &time
    )?;
    let date_time_format = resolved_pattern!(
        prefs,
        YMDT::short().with_time_precision(TimePrecision::Second),
        &date_time
    )?;
    let short_date_time_format = resolved_pattern!(
        prefs,
        YMDT::short().with_time_precision(TimePrecision::Minute),
        &date_time
    )?;
    // udatpg_getBestPattern("yyyyMMMM").
    let month_format = resolved_pattern!(prefs, YM::long(), &date)?;

    let number = DecimalFormatter::try_new(DecimalFormatterPreferences::from(&locale), Default::default())
        .map_err(error)?;
    let mut digits = [0; 10];
    for (index, digit) in digits.iter_mut().enumerate() {
        let symbol = number.format(&Decimal::from(index as u32)).write_to_string().into_owned();
        let mut chars = symbol.chars();
        *digit = chars
            .next()
            .filter(|_| chars.next().is_none())
            .ok_or_else(|| error("multi-character digit"))? as u32;
    }

    let month_labels = months(prefs, "MMMM")?;
    let short_month_labels = months(prefs, "MMM")?;
    let standalone_month_labels = months(prefs, "LLLL").unwrap_or_else(|_| month_labels.clone());
    let short_standalone_month_labels =
        months(prefs, "LLL").unwrap_or_else(|_| short_month_labels.clone());
    let ampm = day_periods(prefs)
        .unwrap_or_else(|_| vec!["AM".to_owned(), "PM".to_owned()]);
    Ok(DateTimeLocaleData {
        date_format,
        month_format,
        time_format,
        short_time_format,
        date_time_format,
        short_date_time_format,
        month_labels,
        short_month_labels,
        standalone_month_labels,
        short_standalone_month_labels,
        ampm_labels: [ampm[0].clone(), ampm[1].clone()],
        digits,
        current_year: current_year()?,
    })
}
