//! Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! DateComponents parsing/numeric projection: platform/text/date_components.cc
//! 58-343,487-553; WTF date_math.cc115-136,237-250. Input subclasses retain
//! their source unit (month or milliseconds); no locale/host Date parsing.
//! Retained editor fields: date_time_edit_element.cc:97-351,469-506,
//! 683-698,723-775; date_time_field_elements.cc:41-74,168-397,579-624,
//! 673-712; date_time_numeric_field_element.cc:117-174,217-284;
//! date_time_symbolic_field_element.cc:98-126,169-193.
use super::input_type::Type;
use super::type_ahead::{
    kCycleFirstChar, kMatchIndex, kMatchPrefix, TypeAhead, TypeAheadDataSource,
};
use std::rc::Rc;

fn leap(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}
fn month_days(year: i64, month: i64) -> i64 {
    if month == 2 && leap(year) {
        29
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][month as usize - 1]
    }
}
fn days(year: i64, month: i64, day: i64) -> i64 {
    let y = year - 1;
    let mut result =
        365 * (year - 1970) + y / 4 - 1970 / 4 - (y / 100 - 1970 / 100) + y / 400 - 1970 / 400;
    for m in 1..month {
        result += month_days(year, m);
    }
    result + day - 1
}
fn weekday(year: i64) -> i64 {
    (days(year, 1, 1) + 4).rem_euclid(7)
}
fn digits(value: &[u16], position: &mut usize, count: usize) -> Option<i64> {
    let mut number = 0i64;
    for _ in 0..count {
        let digit = (*value.get(*position)?).checked_sub(b'0' as u16)?;
        if digit > 9 {
            return None;
        }
        number = number.checked_mul(10)?.checked_add(digit as i64)?;
        *position += 1;
    }
    Some(number)
}
fn take(value: &[u16], position: &mut usize, byte: u8) -> Option<()> {
    if *value.get(*position)? != byte as u16 {
        return None;
    }
    *position += 1;
    Some(())
}
fn year(value: &[u16], position: &mut usize) -> Option<i64> {
    let count = value[*position..]
        .iter()
        .take_while(|&&c| (48..=57).contains(&c))
        .count();
    if count < 4 {
        return None;
    }
    let year = digits(value, position, count)?;
    (1..=275760).contains(&year).then_some(year)
}
fn month(value: &[u16], position: &mut usize) -> Option<(i64, i64)> {
    let year = year(value, position)?;
    take(value, position, b'-')?;
    let month = digits(value, position, 2)?;
    if !(1..=12).contains(&month) || (year == 275760 && month > 9) {
        return None;
    }
    Some((year, month))
}
fn date(value: &[u16], position: &mut usize) -> Option<(i64, i64, i64)> {
    let (year, month) = month(value, position)?;
    take(value, position, b'-')?;
    let day = digits(value, position, 2)?;
    if day < 1 || day > month_days(year, month) || (year == 275760 && month == 9 && day > 13) {
        return None;
    }
    Some((year, month, day))
}
fn time(value: &[u16], position: &mut usize) -> Option<i64> {
    let hour = digits(value, position, 2)?;
    take(value, position, b':')?;
    let minute = digits(value, position, 2)?;
    if hour > 23 || minute > 59 {
        return None;
    }
    let mut second = 0;
    let mut millisecond = 0;
    if value.get(*position) == Some(&(b':' as u16)) {
        *position += 1;
        second = digits(value, position, 2)?;
        if second > 59 {
            return None;
        }
        if value.get(*position) == Some(&(b'.' as u16)) {
            *position += 1;
            let count = value[*position..]
                .iter()
                .take_while(|&&c| (48..=57).contains(&c))
                .count();
            if count == 0 || count > 3 {
                return None;
            }
            millisecond = digits(value, position, count)? * [0, 100, 10, 1][count];
        }
    }
    Some(((hour * 60 + minute) * 60 + second) * 1000 + millisecond)
}
/// BaseTemporalInputType::ParseToNumberOrNaN and subclass parsers. The whole
/// UTF-16 input must be consumed, preserving Chromium's strict HTML grammar.
pub fn ParseToNumber(kind: Type, value: &[u16]) -> Option<i64> {
    let mut position = 0;
    let number = match kind {
        Type::kMonth => {
            let (y, m) = month(value, &mut position)?;
            (y - 1970) * 12 + m - 1
        }
        Type::kDate | Type::kDateTimeLocal => {
            let (y, m, d) = date(value, &mut position)?;
            let mut milliseconds = days(y, m, d) * 86400000;
            if kind == Type::kDateTimeLocal {
                if !matches!(value.get(position), Some(0x54 | 0x20)) {
                    return None;
                }
                position += 1;
                let t = time(value, &mut position)?;
                if y == 275760 && m == 9 && d == 13 && t != 0 {
                    return None;
                }
                milliseconds += t;
            }
            milliseconds
        }
        Type::kTime => time(value, &mut position)?,
        Type::kWeek => {
            let y = year(value, &mut position)?;
            take(value, &mut position, b'-')?;
            take(value, &mut position, b'W')?;
            let week = digits(value, &mut position, 2)?;
            let weekday = weekday(y);
            let max = if weekday == 4 || (weekday == 3 && leap(y)) {
                53
            } else {
                52
            };
            if week < 1 || week > max || (y == 275760 && week > 37) {
                return None;
            }
            let offset = if weekday > 4 {
                8 - weekday
            } else if weekday == 0 {
                1
            } else {
                1 - weekday
            };
            (days(y, 1, 1) + offset + (week - 1) * 7) * 86400000
        }
        _ => return None,
    };
    (position == value.len()).then_some(number)
}

/// DateComponents::SetMillisecondsSinceEpochFor{Date,DateTimeLocal,Week},
/// SetMonthsSinceEpoch, and ToString (date_components.cc:364-505,556-629).
/// MsToYear's estimate and one-year correction follow date_math.cc:143-156.
#[derive(Clone, Copy)]
pub enum SecondFormat {
    None,
    Second,
    Millisecond,
}
pub fn Serialize(kind: Type, value: f64, format: SecondFormat) -> Vec<u16> {
    if !value.is_finite() {
        return Vec::new();
    }
    let mut fields = DateTimeFieldsState::default();
    if kind == Type::kMonth {
        let months = value.round() as i64;
        let year = 1970 + months.div_euclid(12);
        if !(1..=275760).contains(&year) {
            return Vec::new();
        }
        fields.year = Some(year as u32);
        fields.month = Some((months.rem_euclid(12) + 1) as u32);
    } else {
        if kind != Type::kTime && !(-62135596800000.0..=8640000000000000.0).contains(&value) {
            return Vec::new();
        }
        let milliseconds = if kind == Type::kTime {
            value.floor()
        } else {
            value.round()
        };
        let day = (milliseconds / 86400000.0).floor() as i64;
        if kind != Type::kTime {
            let mut year = (milliseconds / (86400000.0 * 365.2425)).floor() as i64 + 1970;
            if days(year, 1, 1) > day {
                year -= 1;
            } else if days(year + 1, 1, 1) <= day {
                year += 1;
            }
            if kind == Type::kWeek {
                let first_week = |year| {
                    let weekday = weekday(year);
                    days(year, 1, 1)
                        + if weekday > 4 {
                            8 - weekday
                        } else {
                            1 - weekday
                        }
                };
                if day < first_week(year) {
                    year -= 1;
                } else if day >= first_week(year + 1) {
                    year += 1;
                }
                fields.week = Some(((day - first_week(year)) / 7 + 1) as u32);
            } else {
                let mut day_in_year = day - days(year, 1, 1);
                let mut month = 1;
                while day_in_year >= month_days(year, month) {
                    day_in_year -= month_days(year, month);
                    month += 1;
                }
                fields.month = Some(month as u32);
                fields.day = Some((day_in_year + 1) as u32);
            }
            if !(1..=275760).contains(&year) {
                return Vec::new();
            }
            fields.year = Some(year as u32);
        }
        if matches!(kind, Type::kTime | Type::kDateTimeLocal) {
            let time = milliseconds.rem_euclid(86400000.0) as u32;
            fields.hour = Some(time / 3600000);
            fields.minute = Some(time / 60000 % 60);
            fields.second = (time % 60000 != 0).then_some(time / 1000 % 60);
            fields.millisecond = (time % 1000 != 0).then_some(time % 1000);
        }
    }
    if matches!(kind, Type::kTime | Type::kDateTimeLocal) {
        // DateComponents::ToStringForTime preserves .SSS, unlike the
        // datetime-local editor's FormatDateTimeFieldsState normalization.
        let mut text = if kind == Type::kDateTimeLocal {
            format!(
                "{:04}-{:02}-{:02}T",
                fields.year.unwrap(),
                fields.month.unwrap(),
                fields.day.unwrap()
            )
        } else {
            String::new()
        };
        text.push_str(&format!(
            "{:02}:{:02}",
            fields.hour.unwrap(),
            fields.minute.unwrap()
        ));
        let milliseconds = fields.millisecond.unwrap_or(0);
        let seconds = fields.second.unwrap_or(0);
        if milliseconds != 0 || seconds != 0 || !matches!(format, SecondFormat::None) {
            text.push_str(&format!(":{seconds:02}"));
        }
        if milliseconds != 0 || matches!(format, SecondFormat::Millisecond) {
            text.push_str(&format!(".{milliseconds:03}"));
        }
        let units: Vec<u16> = text.encode_utf16().collect();
        return if ParseToNumber(kind, &units).is_some() {
            units
        } else {
            Vec::new()
        };
    }
    fields.Value(kind)
}

/// date_time_fields_state.h:38-125. Missing is distinct from zero and from an
/// invalid date (for example February 30). Field validity is resolved only by
/// the owning InputType's SanitizeValue after formatting the fields.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DateTimeFieldsState {
    pub year: Option<u32>,
    pub month: Option<u32>,
    pub day: Option<u32>,
    pub week: Option<u32>,
    pub hour: Option<u32>,
    pub minute: Option<u32>,
    pub second: Option<u32>,
    pub millisecond: Option<u32>,
    pub ampm: Option<u32>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateTimeField {
    Year,
    Month,
    Day,
    Week,
    Hour,
    Hour11,
    Hour12,
    Hour24,
    AmPm,
    SymbolicMonth,
    Minute,
    Second,
    Millisecond,
}
/// The literal fallback formats in each subclass's SetupLayoutParameters.
pub fn FallbackFieldOrder(kind: Type, seconds: bool, milliseconds: bool) -> Vec<DateTimeField> {
    use DateTimeField::*;
    let mut order = match kind {
        Type::kDate => vec![Year, Month, Day],
        Type::kMonth => vec![Year, Month],
        Type::kWeek => vec![Year, Week],
        Type::kTime => vec![Hour, Minute],
        Type::kDateTimeLocal => vec![Year, Month, Day, Hour, Minute],
        _ => unreachable!("temporal type"),
    };
    if seconds {
        order.push(Second);
    }
    if milliseconds {
        order.push(Millisecond);
    }
    order
}
impl DateTimeFieldsState {
    pub fn Field(&self, field: DateTimeField) -> Option<u32> {
        use DateTimeField::*;
        match field {
            Year => self.year,
            Month | SymbolicMonth => self.month,
            Day => self.day,
            Week => self.week,
            Hour => self.hour,
            Hour11 => self.hour.map(|hour| hour % 12),
            Hour12 => self
                .hour
                .map(|hour| if hour % 12 == 0 { 12 } else { hour % 12 }),
            Hour24 => self.hour.map(|hour| if hour == 0 { 24 } else { hour }),
            AmPm => self.ampm,
            Minute => self.minute,
            Second => self.second,
            Millisecond => self.millisecond,
        }
    }
    pub fn SetField(&mut self, field: DateTimeField, value: Option<u32>) {
        use DateTimeField::*;
        match field {
            Hour11 | Hour12 => {
                self.hour = value.map(|hour| hour % 12 + self.ampm.unwrap_or(0) * 12);
                return;
            }
            Hour24 => {
                self.hour = value.map(|hour| hour % 24);
                self.ampm = self.hour.map(|hour| hour / 12);
                return;
            }
            Hour => {
                self.hour = value;
                self.ampm = value.map(|hour| hour / 12);
                return;
            }
            AmPm => {
                self.ampm = value;
                if let (Some(hour), Some(ampm)) = (self.hour, value) {
                    self.hour = Some(hour % 12 + ampm * 12);
                }
                return;
            }
            _ => {}
        }
        *match field {
            Year => &mut self.year,
            Month | SymbolicMonth => &mut self.month,
            Day => &mut self.day,
            Week => &mut self.week,
            Hour | Hour11 | Hour12 | Hour24 | AmPm => unreachable!(),
            Minute => &mut self.minute,
            Second => &mut self.second,
            Millisecond => &mut self.millisecond,
        } = value;
    }
    pub(super) fn Parse(kind: Type, value: &[u16]) -> Self {
        if ParseToNumber(kind, value).is_none() {
            return Self::default();
        }
        let mut result = Self::default();
        let mut p = 0;
        if matches!(kind, Type::kDate | Type::kDateTimeLocal) {
            let (y, m, d) = date(value, &mut p).unwrap();
            result.year = Some(y as u32);
            result.month = Some(m as u32);
            result.day = Some(d as u32);
        } else if kind == Type::kMonth {
            let (y, m) = month(value, &mut p).unwrap();
            result.year = Some(y as u32);
            result.month = Some(m as u32);
        } else if kind == Type::kWeek {
            result.year = Some(year(value, &mut p).unwrap() as u32);
            p += 2;
            result.week = Some(digits(value, &mut p, 2).unwrap() as u32);
        }
        if matches!(kind, Type::kTime | Type::kDateTimeLocal) {
            if kind == Type::kDateTimeLocal {
                p += 1;
            }
            let milliseconds = time(value, &mut p).unwrap();
            result.hour = Some((milliseconds / 3600000) as u32);
            result.ampm = result.hour.map(|hour| hour / 12);
            result.minute = Some((milliseconds / 60000 % 60) as u32);
            result.second = Some((milliseconds / 1000 % 60) as u32);
            result.millisecond = Some((milliseconds % 1000) as u32);
        }
        result
    }
    /// Subclass FormatDateTimeFieldsState followed by BaseTemporalInputType::
    /// SanitizeValue, including partial and internally inconsistent fields.
    pub fn Value(&self, kind: Type) -> Vec<u16> {
        let value = (|| -> Option<String> {
            let date = || {
                Some(format!(
                    "{:04}-{:02}-{:02}",
                    self.year?, self.month?, self.day?
                ))
            };
            let time = || -> Option<String> {
                let mut value = format!("{:02}:{:02}", self.hour?, self.minute?);
                if self.second.unwrap_or(0) != 0
                    || self.millisecond.unwrap_or(0) != 0
                    || (kind == Type::kTime
                        && (self.second.is_some() || self.millisecond.is_some()))
                {
                    value.push_str(&format!(":{:02}", self.second.unwrap_or(0)));
                }
                if self.millisecond.unwrap_or(0) != 0
                    || (kind == Type::kTime && self.millisecond.is_some())
                {
                    value.push_str(&format!(".{:03}", self.millisecond?));
                    while kind == Type::kDateTimeLocal && value.ends_with('0') {
                        value.pop();
                    }
                }
                Some(value)
            };
            Some(match kind {
                Type::kDate => date()?,
                Type::kMonth => format!("{:04}-{:02}", self.year?, self.month?),
                Type::kWeek => format!("{:04}-W{:02}", self.year?, self.week?),
                Type::kTime => time()?,
                Type::kDateTimeLocal => format!("{}T{}", date()?, time()?),
                _ => return None,
            })
        })()
        .unwrap_or_default()
        .encode_utf16()
        .collect::<Vec<_>>();
        if ParseToNumber(kind, &value).is_some() {
            value
        } else {
            Vec::new()
        }
    }
}
/// DateTimeEditElement retains numeric/symbolic field owners and their own
/// typeahead sessions. Locale ordering uses the source ISO layout only when
/// the native format cannot build the input subclass's required fields.
pub struct TemporalInputType {
    kind: Type,
    fields: DateTimeFieldsState,
    order: Vec<DateTimeField>,
    focused: usize,
    typeahead: String,
    value_revision: u64,
    layout_dirty: bool,
    layout_minimum: Vec<u16>,
    layout_maximum: Vec<u16>,
    layout_value_attribute: Vec<u16>,
    layout_step: String,
    ranges: Vec<(u32, u32)>,
    steps: Vec<(i64, i64)>,
    month_labels: Vec<Rc<[u16]>>,
    ampm_labels: Vec<Rc<[u16]>>,
    symbolic_typeahead: Vec<TypeAhead>,
    current_year: Option<u32>,
}
impl TemporalInputType {
    pub fn new(kind: Type, value: &[u16], revision: u64) -> Self {
        let order = FallbackFieldOrder(kind, false, false);
        Self {
            kind,
            fields: DateTimeFieldsState::Parse(kind, value),
            order,
            focused: 0,
            typeahead: String::new(),
            value_revision: revision,
            layout_dirty: true,
            layout_minimum: Vec::new(),
            layout_maximum: Vec::new(),
            layout_value_attribute: Vec::new(),
            layout_step: String::new(),
            ranges: Vec::new(),
            steps: Vec::new(),
            month_labels: Vec::new(),
            ampm_labels: vec![Rc::from([65, 77]), Rc::from([80, 77])],
            symbolic_typeahead: Vec::new(),
            current_year: None,
        }
    }
    pub fn SynchronizeValue(&mut self, kind: Type, value: &[u16], revision: u64) {
        if self.kind != kind {
            *self = Self::new(kind, value, revision);
        } else if self.value_revision != revision {
            self.fields = DateTimeFieldsState::Parse(kind, value);
            self.typeahead.clear();
            self.value_revision = revision;
            self.layout_dirty = true;
        }
    }
    pub fn DidCommitValue(&mut self, revision: u64) {
        self.value_revision = revision;
    }
    /// DateTimeEditElement::GetLayout replaces the locale-ordered field list,
    /// preserving the focused field by type. Unsupported formats must use the
    /// subclass's source fallback rather than a guessed locale ordering.
    pub fn NeedsLayout(
        &self,
        minimum: &[u16],
        maximum: &[u16],
        value_attribute: &[u16],
        step: &str,
    ) -> bool {
        self.layout_dirty
            || self.layout_minimum != minimum
            || self.layout_maximum != maximum
            || self.layout_value_attribute != value_attribute
            || self.layout_step != step
    }
    pub fn InvalidateLayout(&mut self) {
        self.layout_dirty = true;
    }
    pub fn SetFieldOrder(&mut self, order: Vec<DateTimeField>, minimum: &[u16], step: &str) {
        assert!(!order.is_empty(), "date-time layout needs fields");
        self.layout_dirty = false;
        self.layout_minimum = minimum.to_vec();
        self.layout_step = step.to_owned();
        if self.order != order {
            let focused = self.order[self.focused];
            let identity = |field| match field {
                DateTimeField::SymbolicMonth => DateTimeField::Month,
                DateTimeField::Hour11 | DateTimeField::Hour12 | DateTimeField::Hour24 => {
                    DateTimeField::Hour
                }
                _ => field,
            };
            self.focused = order
                .iter()
                .position(|field| identity(*field) == identity(focused))
                .unwrap_or(self.focused.min(order.len() - 1));
            self.order = order;
            self.typeahead.clear();
            self.symbolic_typeahead.clear();
        }
        if self.symbolic_typeahead.len() != self.order.len() {
            self.symbolic_typeahead = (0..self.order.len()).map(|_| TypeAhead::new()).collect();
        }
        if !self.order.contains(&DateTimeField::Second) {
            self.fields.second = None;
        }
        if !self.order.contains(&DateTimeField::Millisecond) {
            self.fields.millisecond = None;
        }
        self.ranges = self
            .order
            .iter()
            .map(|field| {
                let (min, max, _) = Self::Limits(*field);
                (min, max)
            })
            .collect();
        self.steps = vec![(1, 0); self.order.len()];
    }
    /// DateTimeEditBuilder constructor/VisitField/CreateStep. Range limits
    /// affect stepping; numeric typing is clamped only to the hard limits.
    /// The source Should*FieldDisabled helpers have no production call sites.
    pub fn ConfigureFields(
        &mut self,
        maximum: &[u16],
        value_attribute: &[u16],
        step: crate::platform::wtf::decimal::Decimal,
        base: crate::platform::wtf::decimal::Decimal,
        month_labels: Vec<Rc<[u16]>>,
        ampm_labels: Vec<Rc<[u16]>>,
        current_year: Option<u32>,
    ) {
        use crate::platform::wtf::decimal::Decimal;
        use DateTimeField::*;
        self.layout_maximum = maximum.to_vec();
        self.layout_value_attribute = value_attribute.to_vec();
        self.month_labels = month_labels;
        self.current_year = current_year;
        if ampm_labels.len() == 2 {
            self.ampm_labels = ampm_labels;
        }
        let minimum = DateTimeFieldsState::Parse(self.kind, &self.layout_minimum);
        let maximum = DateTimeFieldsState::Parse(self.kind, maximum);
        let both =
            minimum != DateTimeFieldsState::default() && maximum != DateTimeFieldsState::default();
        let same_year = both && minimum.year == maximum.year;
        let range =
            |min: Option<u32>, max: Option<u32>, condition: bool, fallback: (u32, u32)| match (
                min, max,
            ) {
                (Some(a), Some(b)) if condition && a <= b => (a, b),
                _ => fallback,
            };
        let day = range(
            minimum.day,
            maximum.day,
            same_year && minimum.month == maximum.month,
            (1, 31),
        );
        let hour = range(
            minimum.hour,
            maximum.hour,
            both && (self.kind == Type::kTime || day.0 == day.1),
            (0, 23),
        );
        let minute = range(minimum.minute, maximum.minute, hour.0 == hour.1, (0, 59));
        let second = range(
            minimum.second,
            maximum.second,
            minute.0 == minute.1,
            (0, 59),
        );
        let millisecond = range(
            minimum.millisecond,
            maximum.millisecond,
            second.0 == second.1,
            (0, 999),
        );
        for (index, field) in self.order.iter().copied().enumerate() {
            self.ranges[index] = match field {
                Year => {
                    let mut range = (minimum.year.unwrap_or(1), maximum.year.unwrap_or(275760));
                    if range.0 > range.1 {
                        range = (range.1, range.0);
                    }
                    range
                }
                Month | SymbolicMonth => range(minimum.month, maximum.month, same_year, (1, 12)),
                Week => range(minimum.week, maximum.week, same_year, (1, 53)),
                Day => day,
                Hour => hour,
                Hour11 | Hour12 => {
                    let (mut min, mut max) = if hour.1 < 12 {
                        hour
                    } else if hour.0 >= 12 {
                        (hour.0 - 12, hour.1 - 12)
                    } else {
                        (0, 11)
                    };
                    if field == Hour12 {
                        if min == 0 {
                            min = 12;
                        }
                        if max == 0 {
                            max = 12;
                        }
                        if min > max {
                            min = 1;
                            max = 12;
                        }
                    }
                    (min, max)
                }
                Hour24 => {
                    let (min, max) = hour;
                    let min = if min == 0 { 24 } else { min };
                    let max = if max == 0 { 24 } else { max };
                    if min > max {
                        (1, 24)
                    } else {
                        (min, max)
                    }
                }
                Minute => minute,
                Second => second,
                Millisecond => millisecond,
                AmPm => (0, 1),
            };
            let units = match field {
                Hour11 | Hour12 => Some((3600000, 43200000)),
                Hour | Hour24 => Some((3600000, 86400000)),
                Minute => Some((60000, 3600000)),
                Second => Some((1000, 60000)),
                Millisecond => Some((1, 1000)),
                _ => None,
            };
            if let Some((unit, size)) = units {
                let unit = Decimal::FromInt(unit);
                let size = Decimal::FromInt(size);
                let mut field_step = step;
                if field_step.Remainder(size).IsZero() {
                    field_step = size;
                }
                if size.Remainder(field_step).IsZero() && field_step.Remainder(unit).IsZero() {
                    let integer = |value: Decimal| {
                        // CreateStep's results are integral and bounded by
                        // the subfield size; read Decimal's value directly.
                        let data = value.Value();
                        if data.Coefficient() == 0 {
                            return 0;
                        }
                        let magnitude = if data.Exponent() < 0 {
                            data.Coefficient() / 10u64.pow((-data.Exponent()) as u32)
                        } else {
                            data.Coefficient() * 10u64.pow(data.Exponent() as u32)
                        } as i64;
                        if value.IsNegative() {
                            -magnitude
                        } else {
                            magnitude
                        }
                    };
                    self.steps[index] = (
                        integer(field_step / unit),
                        integer((base / unit).Floor().Remainder(size / unit)),
                    );
                }
            }
        }
    }
    pub fn AnyEditableFieldsHaveValues(&self) -> bool {
        self.order
            .iter()
            .any(|field| self.fields.Field(*field).is_some())
    }
    pub fn HasBadInput(&self, value: &[u16]) -> bool {
        value.is_empty() && self.AnyEditableFieldsHaveValues()
    }
    pub fn Fields(&self) -> DateTimeFieldsState {
        self.fields
    }
    pub fn Value(&self) -> Vec<u16> {
        if self.order.contains(&DateTimeField::AmPm) && self.fields.ampm.is_none() {
            return Vec::new();
        }
        self.fields.Value(self.kind)
    }
    /// DateTimeFieldElement::DefaultKeyboardEventHandler and numeric-field
    /// HandleKeyboardEvent/StepUp/StepDown. Locale conversion is supplied as a
    /// digit by the caller, keeping nonnumeric text out of this source owner.
    pub fn HandleKeydown(&mut self, key: &str, alt: bool, readonly: bool) -> bool {
        if key == "ArrowLeft" && self.focused > 0 {
            self.focused -= 1;
            self.typeahead.clear();
            return true;
        }
        if key == "ArrowRight" && self.focused + 1 < self.order.len() {
            self.focused += 1;
            self.typeahead.clear();
            return true;
        }
        if readonly {
            return false;
        }
        let field = self.order[self.focused];
        let (min, max) = self.ranges.get(self.focused).copied().unwrap_or_else(|| {
            let (min, max, _) = Self::Limits(field);
            (min, max)
        });
        if key == "Backspace" || key == "Delete" {
            self.fields.SetField(field, None);
            self.typeahead.clear();
            return true;
        }
        if key == "ArrowUp" || (key == "ArrowDown" && !alt) {
            let current = self.fields.Field(field);
            let (step, base) = self.steps.get(self.focused).copied().unwrap_or((1, 0));
            let round = |value: i64| {
                let n = value - base;
                if key == "ArrowUp" {
                    -(-n).div_euclid(step) * step + base
                } else {
                    n.div_euclid(step) * step + base
                }
            };
            let default = if field == DateTimeField::Year
                && DateTimeFieldsState::Parse(
                    self.kind,
                    if key == "ArrowUp" {
                        &self.layout_minimum
                    } else {
                        &self.layout_maximum
                    },
                )
                .year
                .is_none()
            {
                self.current_year
            } else {
                None
            };
            let mut next = round(if key == "ArrowUp" {
                current.map_or(default.unwrap_or(min) as i64, |v| v as i64 + 1)
            } else {
                current.map_or(default.unwrap_or(max) as i64, |v| v as i64 - 1)
            });
            if next < min as i64 || next > max as i64 {
                next = round(if key == "ArrowUp" {
                    min as i64
                } else {
                    max as i64
                });
            }
            if field == DateTimeField::Hour12
                && current.is_some_and(|old| {
                    (key == "ArrowUp" && old as i64 + step == next && next == 12)
                        || (key == "ArrowDown"
                            && old as i64 - step == next
                            && old == 12
                            && next <= 11)
                })
            {
                self.fields.SetField(
                    DateTimeField::AmPm,
                    Some(if self.fields.ampm == Some(0) { 1 } else { 0 }),
                );
            }
            self.fields.SetField(field, Some(next as u32));
            self.typeahead.clear();
            return true;
        }
        false
    }
    fn Limits(field: DateTimeField) -> (u32, u32, usize) {
        use DateTimeField::*;
        match field {
            Year => (1, 275760, 6),
            Month | SymbolicMonth => (1, 12, 2),
            Day => (1, 31, 2),
            Week => (1, 53, 2),
            Hour => (0, 23, 2),
            Hour11 => (0, 11, 2),
            Hour12 => (1, 12, 2),
            Hour24 => (1, 24, 2),
            AmPm => (0, 1, 1),
            Minute | Second => (0, 59, 2),
            Millisecond => (0, 999, 3),
        }
    }
    pub fn HandleDigit(&mut self, digit: u32) -> bool {
        if digit > 9 {
            return false;
        }
        let field = self.order[self.focused];
        if matches!(field, DateTimeField::AmPm | DateTimeField::SymbolicMonth) {
            return false;
        }
        let (min, max, length) = Self::Limits(field);
        let (_, max_range) = self.ranges.get(self.focused).copied().unwrap_or((min, max));
        let length = if field == DateTimeField::Year {
            format!("{max_range:04}").len()
        } else {
            length
        };
        if self.typeahead.len() >= length {
            self.typeahead.remove(0);
        }
        self.typeahead.push((b'0' + digit as u8) as char);
        let new_value = self.typeahead.parse::<u32>().unwrap();
        if matches!(field, DateTimeField::Hour11 | DateTimeField::Hour12) {
            if new_value > 12 {
                self.fields.SetField(DateTimeField::AmPm, Some(1));
            }
            self.fields.SetField(
                field,
                (new_value >= min).then_some(new_value.min(if field == DateTimeField::Hour12 {
                    24
                } else {
                    23
                })),
            );
        } else {
            self.fields.SetField(
                field,
                (new_value >= min).then_some(new_value.clamp(min, max)),
            );
        }
        if self.typeahead.len() >= length || new_value * 10 > max_range {
            if self.focused + 1 < self.order.len() {
                self.focused += 1;
                self.typeahead.clear();
            }
        }
        true
    }
    pub fn HandleSymbol(&mut self, code: u32, platform_time_us: i64) -> bool {
        let field = self.order[self.focused];
        if !matches!(field, DateTimeField::AmPm | DateTimeField::SymbolicMonth)
            || code < 32
            || code > 0xffff
        {
            return false;
        }
        let symbols = if field == DateTimeField::AmPm {
            &self.ampm_labels
        } else {
            &self.month_labels
        };
        if symbols.is_empty() {
            return false;
        }
        struct Symbols<'a> {
            labels: &'a [Rc<[u16]>],
            selected: i32,
        }
        impl TypeAheadDataSource for Symbols<'_> {
            fn IndexOfSelectedOption(&self) -> i32 {
                self.selected
            }
            fn OptionCount(&self) -> i32 {
                self.labels.len() as i32
            }
            fn OptionAtIndex(&self, index: i32) -> Rc<[u16]> {
                self.labels[index as usize].clone()
            }
        }
        if field == DateTimeField::AmPm {
            self.symbolic_typeahead[self.focused].ResetSession();
        }
        let offset = if field == DateTimeField::SymbolicMonth {
            1
        } else {
            0
        };
        let source = Symbols {
            labels: symbols,
            selected: self
                .fields
                .Field(field)
                .map_or(-1, |value| value as i32 - offset),
        };
        let event = super::select_type::SelectEvent {
            platform_time_us,
            ..Default::default()
        };
        let index = self.symbolic_typeahead[self.focused].HandleEvent(
            &source,
            &event,
            code as u16,
            kMatchPrefix | kCycleFirstChar | kMatchIndex,
        );
        if index >= 0 {
            self.fields.SetField(field, Some((index + offset) as u32));
        }
        true
    }
}

impl DateTimeFieldsState {
    /// date_time_fields_state.cc:93-121. Source 12-hour + AM/PM state is
    /// preserved at the serialization boundary even with a 24-hour field UI.
    pub fn SaveFormControlState(&self) -> Vec<std::rc::Rc<[u16]>> {
        let mut values = vec![
            self.year,
            self.month,
            self.day,
            self.hour.map(|v| if v % 12 == 0 { 12 } else { v % 12 }),
            self.minute,
            self.second,
            self.millisecond,
            self.week,
        ]
        .into_iter()
        .map(|value| {
            value
                .map_or_else(String::new, |v| v.to_string())
                .encode_utf16()
                .collect::<Vec<_>>()
                .into()
        })
        .collect::<Vec<std::rc::Rc<[u16]>>>();
        values.push(
            self.ampm
                .map_or("", |v| if v == 0 { "A" } else { "P" })
                .encode_utf16()
                .collect::<Vec<_>>()
                .into(),
        );
        values
    }
    pub fn RestoreFormControlState(state: &[std::rc::Rc<[u16]>]) -> Self {
        let number = |index: usize| -> Option<u32> {
            let units = state.get(index)?;
            if units.is_empty() || !units.iter().all(|c| (48..=57).contains(c)) {
                return None;
            }
            String::from_utf16(units).ok()?.parse().ok()
        };
        let ampm = state.get(8).and_then(|ampm| {
            if ampm.as_ref() == [65] {
                Some(0)
            } else if ampm.as_ref() == [80] {
                Some(1)
            } else {
                None
            }
        });
        let hour = number(3).and_then(|hour| {
            if !(1..=12).contains(&hour) {
                return None;
            }
            Some(hour % 12 + ampm.unwrap_or(0) * 12)
        });
        Self {
            year: number(0),
            month: number(1),
            day: number(2),
            hour,
            ampm,
            minute: number(4),
            second: number(5),
            millisecond: number(6),
            week: number(7),
        }
    }
}
impl TemporalInputType {
    pub fn RestoreFields(&mut self, state: DateTimeFieldsState) {
        self.fields = state;
        self.typeahead.clear();
    }
}
