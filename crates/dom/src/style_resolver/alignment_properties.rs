#![allow(non_snake_case)]
use crate::style_resolver::selector::SourceSpace;
use layoutng_assembly::internal::layout_input::*;

// cpp: style_resolver/style_resolver.cc:4347-4352
#[derive(Clone, Copy, Debug, PartialEq)]
struct ParsedAlignment<T> {
    value: T,
    overflow: OverflowAlignment,
    legacy: bool,
}
// cpp: style_resolver/style_resolver.cc:4354-4362
fn StripOverflowAlignment(input: &str) -> (&str, OverflowAlignment) {
    let value = input.trim_matches(|c: char| c.source_space());
    for (prefix, overflow) in [
        ("safe ", OverflowAlignment::kSafe),
        ("unsafe ", OverflowAlignment::kUnsafe),
    ] {
        if let Some(value) = value.strip_prefix(prefix) {
            return (value.trim_matches(|c: char| c.source_space()), overflow);
        }
    }
    (value, OverflowAlignment::kDefault)
}
// cpp: style_resolver/style_resolver.cc:4364-4395
fn ParseContentAlignment(input: &str) -> Option<ParsedAlignment<ContentAlignment>> {
    let (value, overflow) = StripOverflowAlignment(input);
    let value = match value {
        "normal" => ContentAlignment::kNormal,
        "baseline" | "first baseline" => ContentAlignment::kBaseline,
        "last baseline" => ContentAlignment::kLastBaseline,
        "start" => ContentAlignment::kStart,
        "end" => ContentAlignment::kEnd,
        "flex-start" => ContentAlignment::kFlexStart,
        "flex-end" => ContentAlignment::kFlexEnd,
        "center" => ContentAlignment::kCenter,
        "left" => ContentAlignment::kLeft,
        "right" => ContentAlignment::kRight,
        "space-between" => ContentAlignment::kSpaceBetween,
        "space-around" => ContentAlignment::kSpaceAround,
        "space-evenly" => ContentAlignment::kSpaceEvenly,
        "stretch" => ContentAlignment::kStretch,
        _ => return None,
    };
    let accepts_overflow = matches!(
        value,
        ContentAlignment::kStart
            | ContentAlignment::kEnd
            | ContentAlignment::kFlexStart
            | ContentAlignment::kFlexEnd
            | ContentAlignment::kCenter
            | ContentAlignment::kLeft
            | ContentAlignment::kRight
    );
    if overflow != OverflowAlignment::kDefault && !accepts_overflow {
        return None;
    }
    Some(ParsedAlignment {
        value,
        overflow,
        legacy: false,
    })
}
// cpp: style_resolver/style_resolver.cc:4397-4443
fn ParseSelfAlignment(
    input: &str,
    allow_auto: bool,
    allow_legacy: bool,
) -> Option<ParsedAlignment<SelfAlignment>> {
    let mut original = input.trim_matches(|c: char| c.source_space());
    let mut legacy = false;
    if let Some(value) = original.strip_prefix("legacy ") {
        if !allow_legacy {
            return None;
        }
        legacy = true;
        original = value.trim_matches(|c: char| c.source_space());
    }
    let (value, overflow) = StripOverflowAlignment(original);
    if legacy && overflow != OverflowAlignment::kDefault {
        return None;
    }
    let value = match value {
        "auto" if allow_auto => SelfAlignment::kAuto,
        "normal" => SelfAlignment::kNormal,
        "stretch" => SelfAlignment::kStretch,
        "baseline" | "first baseline" => SelfAlignment::kBaseline,
        "last baseline" => SelfAlignment::kLastBaseline,
        "center" => SelfAlignment::kCenter,
        "start" => SelfAlignment::kStart,
        "end" => SelfAlignment::kEnd,
        "self-start" => SelfAlignment::kSelfStart,
        "self-end" => SelfAlignment::kSelfEnd,
        "flex-start" => SelfAlignment::kFlexStart,
        "flex-end" => SelfAlignment::kFlexEnd,
        "left" => SelfAlignment::kLeft,
        "right" => SelfAlignment::kRight,
        _ => return None,
    };
    let accepts_overflow = matches!(
        value,
        SelfAlignment::kCenter
            | SelfAlignment::kStart
            | SelfAlignment::kEnd
            | SelfAlignment::kSelfStart
            | SelfAlignment::kSelfEnd
            | SelfAlignment::kFlexStart
            | SelfAlignment::kFlexEnd
            | SelfAlignment::kLeft
            | SelfAlignment::kRight
    );
    if overflow != OverflowAlignment::kDefault && !accepts_overflow {
        return None;
    }
    if legacy
        && !matches!(
            value,
            SelfAlignment::kCenter | SelfAlignment::kLeft | SelfAlignment::kRight
        )
    {
        return None;
    }
    Some(ParsedAlignment {
        value,
        overflow,
        legacy,
    })
}
// cpp: style_resolver/style_resolver.cc:4445-4471
fn ParseAlignItems(input: &str) -> Option<ParsedAlignment<AlignItems>> {
    let parsed = ParseSelfAlignment(input, false, false)?;
    let value = match parsed.value {
        SelfAlignment::kNormal => AlignItems::kNormal,
        SelfAlignment::kStretch => AlignItems::kStretch,
        SelfAlignment::kBaseline => AlignItems::kBaseline,
        SelfAlignment::kLastBaseline => AlignItems::kLastBaseline,
        SelfAlignment::kCenter => AlignItems::kCenter,
        SelfAlignment::kStart => AlignItems::kStart,
        SelfAlignment::kEnd => AlignItems::kEnd,
        SelfAlignment::kSelfStart => AlignItems::kSelfStart,
        SelfAlignment::kSelfEnd => AlignItems::kSelfEnd,
        SelfAlignment::kFlexStart => AlignItems::kFlexStart,
        SelfAlignment::kFlexEnd => AlignItems::kFlexEnd,
        SelfAlignment::kAuto | SelfAlignment::kLeft | SelfAlignment::kRight => return None,
    };
    Some(ParsedAlignment {
        value,
        overflow: parsed.overflow,
        legacy: false,
    })
}
// cpp: style_resolver/style_resolver.cc:5064-5121
pub(crate) fn ApplyAlignmentProperty(style: &mut ComputedStyle, property: &str, raw: &str) -> bool {
    let value = raw
        .trim_matches(|c: char| c.source_space())
        .to_ascii_lowercase();
    match property {
        "align-items" => {
            if let Some(parsed) = ParseAlignItems(&value) {
                style.align_items = parsed.value;
                style
                    .extended
                    .get_or_insert_with(Default::default)
                    .align_items_overflow = parsed.overflow;
            }
        }
        "align-content" => {
            if let Some(parsed) = ParseContentAlignment(&value) {
                let extra = style.extended.get_or_insert_with(Default::default);
                extra.align_content = Some(parsed.value);
                extra.align_content_overflow = parsed.overflow;
            }
        }
        "-internal-align-content-block" => {
            style
                .extended
                .get_or_insert_with(Default::default)
                .align_content_block_center = value == "center"
        }
        "align-self" | "justify-items" | "justify-self" => {
            if let Some(parsed) = ParseSelfAlignment(
                &value,
                property != "justify-items",
                property == "justify-items",
            ) {
                let extra = style.extended.get_or_insert_with(Default::default);
                match property {
                    "align-self" => {
                        extra.align_self = Some(parsed.value);
                        extra.align_self_overflow = parsed.overflow;
                    }
                    "justify-items" => {
                        extra.justify_items = Some(parsed.value);
                        extra.justify_items_overflow = parsed.overflow;
                        extra.justify_items_legacy = parsed.legacy;
                    }
                    _ => {
                        extra.justify_self = Some(parsed.value);
                        extra.justify_self_overflow = parsed.overflow;
                    }
                }
            }
        }
        "justify-content" => {
            if let Some(parsed) = ParseContentAlignment(&value) {
                let value = match parsed.value {
                    ContentAlignment::kNormal => Some(JustifyContent::kNormal),
                    ContentAlignment::kStart => Some(JustifyContent::kStart),
                    ContentAlignment::kEnd => Some(JustifyContent::kEnd),
                    ContentAlignment::kFlexStart => Some(JustifyContent::kFlexStart),
                    ContentAlignment::kFlexEnd => Some(JustifyContent::kFlexEnd),
                    ContentAlignment::kCenter => Some(JustifyContent::kCenter),
                    ContentAlignment::kLeft => Some(JustifyContent::kLeft),
                    ContentAlignment::kRight => Some(JustifyContent::kRight),
                    ContentAlignment::kSpaceBetween => Some(JustifyContent::kSpaceBetween),
                    ContentAlignment::kSpaceAround => Some(JustifyContent::kSpaceAround),
                    ContentAlignment::kSpaceEvenly => Some(JustifyContent::kSpaceEvenly),
                    ContentAlignment::kBaseline
                    | ContentAlignment::kLastBaseline
                    | ContentAlignment::kStretch => None,
                };
                if let Some(value) = value {
                    let extra = style.extended.get_or_insert_with(Default::default);
                    extra.justify_content = value;
                    extra.justify_content_overflow = parsed.overflow;
                }
            }
        }
        _ => return false,
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    fn decode(value: &str) -> String {
        String::from_utf8(
            (0..value.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&value[i..i + 2], 16).unwrap())
                .collect(),
        )
        .unwrap()
    }
    #[test]
    fn complete_alignment_parsers_match_cpp() {
        for line in
            include_str!("../../../../artifacts/cpp-reference/alignment-parser-results.tsv").lines()
        {
            let fields: Vec<_> = line.split('\t').collect();
            let value = decode(fields[1]);
            let parsed = match fields[0] {
                "content" => ParseContentAlignment(&value)
                    .map(|v| (v.value as i32, v.overflow as i32, v.legacy)),
                "items" => {
                    ParseAlignItems(&value).map(|v| (v.value as i32, v.overflow as i32, v.legacy))
                }
                _ => ParseSelfAlignment(
                    &value,
                    matches!(fields[0], "self-auto" | "self-both"),
                    matches!(fields[0], "self-legacy" | "self-both"),
                )
                .map(|v| (v.value as i32, v.overflow as i32, v.legacy)),
            };
            let expected = if fields[2] == "none" {
                None
            } else {
                Some((
                    fields[2].parse().unwrap(),
                    fields[3].parse().unwrap(),
                    fields[4] == "1",
                ))
            };
            assert_eq!(parsed, expected, "{} {:?}", fields[0], value);
        }
    }
    #[test]
    fn sequential_flex_and_alignment_declarations_match_cpp() {
        let mut style = ComputedStyle::default();
        style
            .extended
            .get_or_insert_with(Default::default)
            .font_size = 24.0;
        for line in
            include_str!("../../../../artifacts/cpp-reference/flex-alignment-results.tsv").lines()
        {
            let fields: Vec<_> = line.split('\t').collect();
            let value = decode(fields[1]);
            crate::style_resolver::apply(&mut style, fields[0], &value, (1024.0, 768.0));
            let e = style.extended.as_ref().unwrap();
            let actual = [
                Some(style.flex_grow as f64),
                Some(style.flex_shrink as f64),
                style.flex_basis,
                e.flex_basis_percent,
                Some(e.flex_basis_calculated as u8 as f64),
                Some(e.flex_basis_sizing as i32 as f64),
                Some(style.flex_direction as i32 as f64),
                Some(style.flex_wrap as u8 as f64),
                Some(e.wrap_reverse as u8 as f64),
                Some(e.order as f64),
                Some(style.align_items as i32 as f64),
                Some(e.align_items_overflow as i32 as f64),
                e.align_content.map(|v| v as i32 as f64),
                Some(e.align_content_overflow as i32 as f64),
                Some(e.align_content_block_center as u8 as f64),
                e.align_self.map(|v| v as i32 as f64),
                Some(e.align_self_overflow as i32 as f64),
                e.justify_items.map(|v| v as i32 as f64),
                Some(e.justify_items_overflow as i32 as f64),
                Some(e.justify_items_legacy as u8 as f64),
                e.justify_self.map(|v| v as i32 as f64),
                Some(e.justify_self_overflow as i32 as f64),
                Some(e.justify_content as i32 as f64),
                Some(e.justify_content_overflow as i32 as f64),
            ];
            let expected: Vec<Option<f64>> = fields[2..]
                .iter()
                .map(|s| {
                    if *s == "none" {
                        None
                    } else {
                        Some(s.parse().unwrap())
                    }
                })
                .collect();
            assert_eq!(
                actual.as_slice(),
                expected.as_slice(),
                "{} {:?}",
                fields[0],
                value
            );
        }
    }
}
