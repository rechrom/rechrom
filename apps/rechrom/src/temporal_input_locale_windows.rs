//! LocaleWin's native LCID selection, user preferences and LDML conversion.
//! Chromium platform/text/locale_win.cc:52-184,225-399,478-592,633-687.
use super::*;
use std::ptr;

#[repr(C)]
#[derive(Default)]
struct SystemTime {
    year: u16,
    month: u16,
    day_of_week: u16,
    day: u16,
    hour: u16,
    minute: u16,
    second: u16,
    milliseconds: u16,
}
#[link(name = "kernel32")]
extern "system" {
    fn GetLocaleInfoW(locale: u32, kind: u32, result: *mut u16, capacity: i32) -> i32;
    fn LocaleNameToLCID(locale: *const u16, flags: u32) -> u32;
    fn GetLocalTime(time: *mut SystemTime);
}
fn info(locale: u32, kind: u32) -> io::Result<String> {
    let length = unsafe { GetLocaleInfoW(locale, kind, ptr::null_mut(), 0) };
    if length <= 0 {
        return Err(io::Error::last_os_error());
    }
    let mut result = vec![0; length as usize];
    if unsafe { GetLocaleInfoW(locale, kind, result.as_mut_ptr(), length) } != length {
        return Err(io::Error::last_os_error());
    }
    String::from_utf16(&result[..length as usize - 1])
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}
fn number(locale: u32, kind: u32, on_failure: u32) -> u32 {
    let mut result = on_failure;
    unsafe {
        GetLocaleInfoW(
            locale,
            kind | 0x20000000,
            (&mut result as *mut u32).cast(),
            2,
        );
    }
    result
}
fn commit_literal(literal: &mut String, converted: &mut String) {
    // DateTimeFormat::QuoteAndAppend, including adjacent literal apostrophes.
    if literal.is_empty() {
        return;
    }
    if !literal
        .chars()
        .any(|ch| ch.is_ascii_alphabetic() || ch == '\'')
    {
        converted.push_str(literal);
    } else if !literal.contains('\'') {
        converted.push('\'');
        converted.push_str(literal);
        converted.push('\'');
    } else {
        let leading = literal.chars().take_while(|ch| *ch == '\'').count();
        for _ in 0..leading {
            converted.push_str("''");
        }
        if leading < literal.len() {
            converted.push('\'');
            converted.push_str(&literal[leading..].replace('\'', "''"));
            converted.push('\'');
        }
    }
    literal.clear();
}
fn convert_windows_format(format: &str) -> String {
    let chars: Vec<_> = format.chars().collect();
    let mut converted = String::new();
    let mut literal = String::new();
    let mut quoted = false;
    let mut last_quote_can_be_literal = false;
    let mut index = 0;
    while index < chars.len() {
        let ch = chars[index];
        if quoted {
            if ch == '\'' {
                quoted = false;
                if last_quote_can_be_literal && index > 0 && chars[index - 1] == '\'' {
                    literal.push('\'');
                    last_quote_can_be_literal = false;
                } else {
                    last_quote_can_be_literal = true;
                }
            } else {
                literal.push(ch);
            }
            index += 1;
            continue;
        }
        if ch == '\'' {
            quoted = true;
            if last_quote_can_be_literal && index > 0 && chars[index - 1] == '\'' {
                literal.push(ch);
                last_quote_can_be_literal = false;
            } else {
                last_quote_can_be_literal = true;
            }
        } else if ch.is_ascii_alphabetic() {
            commit_literal(&mut literal, &mut converted);
            let mut count = 1;
            while chars.get(index + count) == Some(&ch) {
                count += 1;
            }
            match ch {
                'h' | 'H' | 'm' | 's' | 'M' | 'y' => {
                    converted.extend(std::iter::repeat_n(ch, count))
                }
                'd' if count <= 2 => converted.extend(std::iter::repeat_n(ch, count)),
                'd' => converted.push_str(if count == 3 { "EEE" } else { "EEEE" }),
                'g' if count == 1 => converted.push('G'),
                'g' => (), // Windows imperial-era gg is deliberately omitted.
                't' => converted.push('a'),
                _ => literal.extend(std::iter::repeat_n(ch, count)),
            }
            index += count;
            continue;
        } else {
            literal.push(ch);
        }
        index += 1;
    }
    commit_literal(&mut literal, &mut converted);
    converted
}
pub(super) fn current(ui_locale: &CStr) -> io::Result<TemporalInputLocaleData> {
    const USER_DEFAULT: u32 = 0x400;
    let requested = ui_locale
        .to_str()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let user_language = info(USER_DEFAULT, 0x59)?; // LOCALE_SISO639LANGNAME
    let language = requested.split('-').next().unwrap_or("");
    let locale = if language.eq_ignore_ascii_case(&user_language) {
        USER_DEFAULT // Preserve the user's date/time overrides for this language.
    } else {
        let name: Vec<u16> = requested.encode_utf16().chain(Some(0)).collect();
        if name.len() > 85 || requested.is_empty() {
            return Err(io::Error::other("invalid Windows UI locale"));
        }
        let locale = unsafe { LocaleNameToLCID(name.as_ptr(), 0) };
        if locale == 0 {
            return Err(io::Error::last_os_error());
        }
        locale
    };
    let pattern = |kind| info(locale, kind).map(|value| convert_windows_format(&value));
    let date_format = pattern(0x1f)?; // LOCALE_SSHORTDATE
    let month_format = pattern(0x1006)?; // LOCALE_SYEARMONTH
    let time_format = pattern(0x1003)?; // LOCALE_STIMEFORMAT
    let short_time_format = pattern(0x79)?; // LOCALE_SSHORTTIME
    let month_labels = (0x38..=0x43)
        .map(|kind| info(locale, kind))
        .collect::<io::Result<Vec<_>>>()?;
    let short_month_labels = (0x44..=0x4f)
        .map(|kind| info(locale, kind))
        .collect::<io::Result<Vec<_>>>()?;
    let mut digits = std::array::from_fn(|index| b'0' as u32 + index as u32);
    if number(locale, 0x1014, 1) != 1 {
        // LOCALE_IDIGITSUBSTITUTION
        let native = info(locale, 0x13)?; // LOCALE_SNATIVEDIGITS
        let units: Vec<_> = native.encode_utf16().collect();
        if units.len() != 10 {
            return Err(io::Error::other("invalid Windows native digits"));
        }
        for (digit, unit) in digits.iter_mut().zip(units) {
            *digit = unit as u32;
        }
    }
    let mut now = SystemTime::default();
    unsafe {
        GetLocalTime(&mut now);
    }
    Ok(TemporalInputLocaleData {
        date_time_format: format!("{date_format} {time_format}"),
        short_date_time_format: format!("{date_format} {short_time_format}"),
        date_format,
        month_format,
        time_format,
        short_time_format,
        // LocaleWin explicitly uses format-context names for standalone names.
        standalone_month_labels: month_labels.clone(),
        short_standalone_month_labels: short_month_labels.clone(),
        month_labels,
        short_month_labels,
        ampm_labels: [info(locale, 0x28)?, info(locale, 0x29)?],
        digits,
        current_year: now.year as u32,
    })
}
