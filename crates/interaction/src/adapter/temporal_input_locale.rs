//! Native Locale format/symbol values, without a platform formatter or DOM
//! reaching into the retained DateTimeEdit field owners.
use super::page_form_validation::{DateTimeField, TemporalInputLocale};
use dom::DOMInputType;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemporalInputLocaleData {
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
pub struct TemporalFieldLayout {
    pub order: Vec<DateTimeField>,
    pub month_labels: Vec<Rc<[u16]>>,
    pub ampm_labels: Vec<Rc<[u16]>>,
    pub current_year: Option<u32>,
}
impl TemporalInputLocaleData {
    /// DateTimeFormat::Parse + DateTimeFormatValidator. Quoted literals and
    /// escaped apostrophes never become fields; unsupported formats use the
    /// owning input subclass's existing fallback.
    fn Layout(
        &self,
        kind: DOMInputType,
        seconds: bool,
        milliseconds: bool,
    ) -> Option<TemporalFieldLayout> {
        use DateTimeField::*;
        let format = match kind {
            DOMInputType::kDate => &self.date_format,
            DOMInputType::kMonth => &self.month_format,
            DOMInputType::kTime => {
                if seconds {
                    &self.time_format
                } else {
                    &self.short_time_format
                }
            }
            DOMInputType::kDateTimeLocal => {
                if seconds {
                    &self.date_time_format
                } else {
                    &self.short_date_time_format
                }
            }
            // WeekInputType::SetupLayoutParameters uses a localized literal
            // around fixed yyyy/ww fields, independent of date ordering.
            DOMInputType::kWeek => {
                return Some(TemporalFieldLayout {
                    order: vec![Year, Week],
                    month_labels: Vec::new(),
                    ampm_labels: Vec::new(),
                    current_year: Some(self.current_year),
                })
            }
            _ => return None,
        };
        let chars: Vec<char> = format.chars().collect();
        let mut index = 0;
        let mut quoted = false;
        let mut order = Vec::new();
        let mut labels = &self.month_labels;
        while index < chars.len() {
            let ch = chars[index];
            index += 1;
            if ch == '\'' {
                if chars.get(index) == Some(&'\'') {
                    index += 1;
                } else {
                    quoted = !quoted;
                }
                continue;
            }
            if quoted || !ch.is_ascii_alphabetic() {
                continue;
            }
            let mut count = 1;
            while chars.get(index) == Some(&ch) {
                count += 1;
                index += 1;
            }
            let field = match ch {
                'y' => Year,
                'd' => Day,
                'w' => Week,
                'H' => Hour,
                'h' => Hour12,
                'K' => Hour11,
                'k' => Hour24,
                'm' => Minute,
                's' => Second,
                'S' => Millisecond,
                'a' | 'b' | 'B' => AmPm,
                'M' | 'L' => {
                    if (3..=5).contains(&count) {
                        labels = match (ch, count) {
                            ('L', 4) => &self.standalone_month_labels,
                            ('L', _) => &self.short_standalone_month_labels,
                            (_, 4) => &self.month_labels,
                            _ => &self.short_month_labels,
                        };
                        SymbolicMonth
                    } else {
                        Month
                    }
                }
                // DateTimeFormat recognizes these tokens; the source edit
                // builder and validator deliberately ignore their fields.
                'c' | 'e' | 'g' | 'q' | 'r' | 'u' | 'v' | 'x' | 'z' | 'A' | 'D' | 'E' | 'F'
                | 'G' | 'O' | 'Q' | 'U' | 'V' | 'W' | 'X' | 'Y' | 'Z' => continue,
                _ => return None,
            };
            if order.contains(&field) {
                return None;
            }
            order.push(field);
            if field == Second && milliseconds {
                order.push(Millisecond);
            }
        }
        if quoted {
            return None;
        }
        let has = |field| order.contains(&field);
        let date = has(Year) && (has(Month) || has(SymbolicMonth)) && has(Day);
        let hour = has(Hour) || has(Hour24) || ((has(Hour11) || has(Hour12)) && has(AmPm));
        let valid = match kind {
            DOMInputType::kDate => date,
            DOMInputType::kMonth => has(Year) && (has(Month) || has(SymbolicMonth)),
            DOMInputType::kTime => hour && has(Minute),
            DOMInputType::kDateTimeLocal => date && hour && has(Minute),
            _ => false,
        };
        if !valid || (has(SymbolicMonth) && labels.len() != 12) {
            return None;
        }
        let units = |label: &String| Rc::from(label.encode_utf16().collect::<Vec<_>>());
        Some(TemporalFieldLayout {
            order,
            month_labels: labels.iter().map(units).collect(),
            ampm_labels: self.ampm_labels.iter().map(units).collect(),
            current_year: Some(self.current_year),
        })
    }
}
impl TemporalInputLocale for TemporalInputLocaleData {
    fn FieldOrder(
        &self,
        kind: DOMInputType,
        seconds: bool,
        milliseconds: bool,
    ) -> Option<Vec<DateTimeField>> {
        Some(self.Layout(kind, seconds, milliseconds)?.order)
    }
    fn FieldLayout(
        &self,
        kind: DOMInputType,
        seconds: bool,
        milliseconds: bool,
    ) -> Option<TemporalFieldLayout> {
        self.Layout(kind, seconds, milliseconds)
    }
    fn ConvertFromLocalizedDigit(&self, code: u32) -> Option<u32> {
        self.digits
            .iter()
            .position(|digit| *digit == code)
            .map(|digit| digit as u32)
            .or_else(|| code.checked_sub(48).filter(|digit| *digit <= 9))
    }
}
