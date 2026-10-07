#![allow(non_snake_case)]
use crate::style_resolver::{
    cascade::{CascadedDeclaration, ResolveDeclarationVariables},
    css_wide::{CSSWideKeywordSource, CSSWideSource},
    CustomProperties,
};
use layoutng_assembly::internal::layout_input::ComputedStyle;

// Blink: css/resolver/style_builder_converter.cc:ConvertZoom and
// css/resolver/style_resolver_state.cc:StyleResolverState::SetZoom.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NumericKind {
    Number,
    Length,
}

#[derive(Clone, Copy, Debug)]
struct Numeric {
    value: f64,
    kind: NumericKind,
}

struct NumericParser<'a> {
    input: &'a str,
    cursor: usize,
}

impl<'a> NumericParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, cursor: 0 }
    }

    fn parse(mut self) -> Option<Numeric> {
        let value = self.sum()?;
        self.space();
        (self.cursor == self.input.len()).then_some(value)
    }

    fn sum(&mut self) -> Option<Numeric> {
        let mut value = self.product()?;
        loop {
            self.space();
            let Some(operation @ (b'+' | b'-')) = self.peek() else {
                return Some(value);
            };
            self.cursor += 1;
            let right = self.product()?;
            if value.kind != right.kind {
                return None;
            }
            value.value = if operation == b'+' {
                value.value + right.value
            } else {
                value.value - right.value
            };
        }
    }

    fn product(&mut self) -> Option<Numeric> {
        let mut value = self.primary()?;
        loop {
            self.space();
            let Some(operation @ (b'*' | b'/')) = self.peek() else {
                return Some(value);
            };
            self.cursor += 1;
            let right = self.primary()?;
            value = match operation {
                b'*' if value.kind == NumericKind::Number => Numeric {
                    value: value.value * right.value,
                    kind: right.kind,
                },
                b'*' if right.kind == NumericKind::Number => Numeric {
                    value: value.value * right.value,
                    kind: value.kind,
                },
                b'/' if right.value != 0.0 && right.kind == NumericKind::Number => Numeric {
                    value: value.value / right.value,
                    kind: value.kind,
                },
                b'/' if right.value != 0.0 && value.kind == right.kind => Numeric {
                    value: value.value / right.value,
                    kind: NumericKind::Number,
                },
                _ => return None,
            };
        }
    }

    fn primary(&mut self) -> Option<Numeric> {
        self.space();
        if self.consume(b'(') {
            let value = self.sum()?;
            self.space();
            return self.consume(b')').then_some(value);
        }
        let start = self.cursor;
        if self.peek().is_some_and(|byte| byte.is_ascii_alphabetic()) {
            while self.peek().is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'-') {
                self.cursor += 1;
            }
            let name = self.input[start..self.cursor].to_ascii_lowercase();
            self.space();
            if !self.consume(b'(') {
                return None;
            }
            return self.function(&name);
        }
        self.numeric_token()
    }

    fn function(&mut self, name: &str) -> Option<Numeric> {
        if name == "calc" {
            let value = self.sum()?;
            self.space();
            return self.consume(b')').then_some(value);
        }
        if !matches!(name, "min" | "max" | "clamp") {
            return None;
        }
        let mut values = Vec::new();
        loop {
            values.push(self.sum()?);
            self.space();
            if self.consume(b')') {
                break;
            }
            if !self.consume(b',') {
                return None;
            }
        }
        if values.is_empty() || (name == "clamp" && values.len() != 3) {
            return None;
        }
        let kind = values[0].kind;
        if values.iter().any(|value| value.kind != kind) {
            return None;
        }
        let value = match name {
            "min" => values.iter().map(|value| value.value).reduce(f64::min)?,
            "max" => values.iter().map(|value| value.value).reduce(f64::max)?,
            _ => values[0].value.max(values[1].value.min(values[2].value)),
        };
        Some(Numeric { value, kind })
    }

    fn numeric_token(&mut self) -> Option<Numeric> {
        self.space();
        let start = self.cursor;
        if self.peek().is_some_and(|byte| matches!(byte, b'+' | b'-')) {
            self.cursor += 1;
        }
        let integer = self.cursor;
        while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
            self.cursor += 1;
        }
        if self.peek() == Some(b'.') {
            self.cursor += 1;
            while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
                self.cursor += 1;
            }
        }
        if self.cursor == integer || (self.cursor == integer + 1 && &self.input[integer..self.cursor] == ".") {
            return None;
        }
        if self.peek().is_some_and(|byte| matches!(byte, b'e' | b'E')) {
            let exponent = self.cursor;
            self.cursor += 1;
            if self.peek().is_some_and(|byte| matches!(byte, b'+' | b'-')) {
                self.cursor += 1;
            }
            let digits = self.cursor;
            while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
                self.cursor += 1;
            }
            if digits == self.cursor {
                self.cursor = exponent;
            }
        }
        let number_end = self.cursor;
        if self.peek() == Some(b'%') {
            self.cursor += 1;
            return Some(Numeric {
                value: self.input[start..number_end].parse::<f64>().ok()? / 100.0,
                kind: NumericKind::Number,
            });
        }
        while self.peek().is_some_and(|byte| byte.is_ascii_alphabetic()) {
            self.cursor += 1;
        }
        let number = self.input[start..number_end].parse::<f64>().ok()?;
        let unit = &self.input[number_end..self.cursor];
        if unit.is_empty() {
            return Some(Numeric {
                value: number,
                kind: NumericKind::Number,
            });
        }
        let pixels = crate::style_resolver::border_radius::Length(
            &self.input[start..self.cursor],
            16.0,
        )?;
        Some(Numeric {
            value: pixels,
            kind: NumericKind::Length,
        })
    }

    fn space(&mut self) {
        while self.peek().is_some_and(|byte| byte.is_ascii_whitespace()) {
            self.cursor += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.as_bytes().get(self.cursor).copied()
    }

    fn consume(&mut self, byte: u8) -> bool {
        if self.peek() == Some(byte) {
            self.cursor += 1;
            true
        } else {
            false
        }
    }
}

fn ParseZoom(value: &str) -> Option<f32> {
    let value = value.trim().to_ascii_lowercase();
    if value == "normal" {
        return Some(1.0);
    }
    let parsed = NumericParser::new(&value).parse()?;
    if parsed.kind != NumericKind::Number {
        return None;
    }
    let number = parsed.value;
    if !number.is_finite() || number < 0.0 || number > f32::MAX as f64 {
        return None;
    }
    Some(if number == 0.0 { 1.0 } else { number as f32 })
}
pub(crate) fn ApplyZoom(style: &mut ComputedStyle, value: &str) {
    if let Some(zoom) = ParseZoom(value) {
        let e = style.extended.get_or_insert_with(Default::default);
        e.effective_zoom = (e.effective_zoom / e.zoom * zoom).clamp(1e-6, 1e6);
        e.zoom = zoom;
    }
}
pub(crate) fn ResolveZoom(
    cascade: &[CascadedDeclaration],
    custom: &CustomProperties,
    initial: &CSSWideSource<'_>,
    inherited: &CSSWideSource<'_>,
    unset: &CSSWideSource<'_>,
    reverted: &CSSWideSource<'_>,
) -> f32 {
    let mut zoom = 1.0;
    for item in cascade {
        if !matches!(item.declaration.property.as_str(), "zoom" | "all") {
            continue;
        }
        let declaration =
            ResolveDeclarationVariables(&item.declaration, custom).unwrap_or_else(|| {
                cssom::CSSDeclaration {
                    property: item.declaration.property.clone(),
                    value: "unset".into(),
                    ..Default::default()
                }
            });
        // Zoom is non-inherited. CSS-wide unset therefore uses the initial own zoom.
        let value = declaration.value.trim().to_ascii_lowercase();
        if value == "unset" {
            zoom = 1.0;
        } else if let Some(source) =
            CSSWideKeywordSource(&value, initial, inherited, unset, reverted)
        {
            zoom = source.style.extended.as_ref().map_or(1.0, |e| e.zoom);
        } else if declaration.property == "zoom" {
            if let Some(parsed) = ParseZoom(&value) {
                zoom = parsed;
            }
        }
    }
    zoom
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zero_normal_percent_and_invalid_values_follow_blink() {
        for (value, expected) in [
            ("0", Some(1.0)),
            ("0%", Some(1.0)),
            ("normal", Some(1.0)),
            ("80%", Some(0.8)),
            ("2", Some(2.0)),
            ("calc(1280px / 1536px)", Some(1280.0 / 1536.0)),
            (
                "min(calc(1280px / 1536px), calc(1080px / 1536px))",
                Some(1080.0 / 1536.0),
            ),
            ("-1", None),
            ("1px", None),
            ("NaN", None),
        ] {
            assert_eq!(ParseZoom(value), expected, "{value}");
        }
    }
}
