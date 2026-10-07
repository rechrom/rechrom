#![allow(non_snake_case)]
use crate::style_resolver::{
    border_radius::{Length, SplitTopLevel},
    fonts::FontSizeOf,
    number::{Number, PositiveInteger},
    selector::SourceSpace,
    shadow::SplitWhitespace,
};
use layoutng_assembly::internal::layout_input::*;

// cpp: style_resolver/style_resolver.cc:1890-1900
fn ParseBreakRule(value: &str, inside: bool) -> BreakRule {
    if matches!(value, "avoid" | "avoid-page" | "avoid-column") {
        BreakRule::kAvoid
    } else if !inside
        && matches!(
            value,
            "always" | "page" | "column" | "left" | "right" | "recto" | "verso"
        )
    {
        BreakRule::kAlways
    } else {
        BreakRule::kAuto
    }
}
// cpp: style_resolver/style_resolver.cc:4482-4498
fn ParseContainIntrinsicLength(value: &str, font_size: f64) -> Option<ContainIntrinsicLength> {
    let parts = SplitWhitespace(value);
    let (automatic, value) = match parts.as_slice() {
        [value] => (false, value),
        [auto, value] if auto == "auto" => (true, value),
        _ => return None,
    };
    let length = if value == "none" {
        None
    } else {
        Some(Length(value, font_size).filter(|n| *n >= 0.0)?)
    };
    Some(ContainIntrinsicLength {
        has_auto: automatic,
        length,
    })
}

// cpp: style_resolver/style_resolver.cc:5130-5143,5185-5190,5207-5248,5283-5302,5361-5563
// Grid track/placement parsing is implemented at a separate boundary.
pub(crate) fn ApplyFlowProperty(style: &mut ComputedStyle, property: &str, raw: &str) -> bool {
    let lower = raw
        .trim_matches(|c: char| c.source_space())
        .to_ascii_lowercase();
    let value = lower.as_str();
    let font_size = FontSizeOf(style);
    macro_rules! assign {
        ($field:ident, $value:expr) => {
            style.extended.get_or_insert_with(Default::default).$field = $value
        };
    }
    match property {
        "position" => {
            if let Some(position) = match value {
                "static" => Some(Position::kStatic),
                "relative" => Some(Position::kRelative),
                "absolute" => Some(Position::kAbsolute),
                "fixed" => Some(Position::kFixed),
                "sticky" => Some(Position::kSticky),
                _ => None,
            } {
                style.position = position;
            }
        }
        "z-index" => {
            if value == "auto" {
                style.paint.z_index = None;
            } else if let Some(number) = Number(value)
                .filter(|n| n.floor() == *n && *n >= i32::MIN as f64 && *n <= i32::MAX as f64)
            {
                style.paint.z_index = Some(number as i32);
            }
        }
        "float" => {
            if let Some(floating) = match value {
                "none" => Some(FloatSide::kNone),
                "left" => Some(FloatSide::kLeft),
                "right" => Some(FloatSide::kRight),
                "inline-start" => Some(FloatSide::kInlineStart),
                "inline-end" => Some(FloatSide::kInlineEnd),
                _ => None,
            } {
                style.floating = floating;
            }
        }
        "columns" => {
            let parts = SplitWhitespace(value);
            if !parts.is_empty() && parts.len() <= 2 {
                let (mut count, mut width, mut valid) = (None, None, true);
                for part in &parts {
                    if part == "auto" {
                        continue;
                    }
                    if let Some(number) = PositiveInteger(part).filter(|_| count.is_none()) {
                        count = Some(number);
                    } else if let Some(length) =
                        Length(part, font_size).filter(|n| *n > 0.0 && width.is_none())
                    {
                        width = Some(length);
                    } else {
                        valid = false;
                        break;
                    }
                }
                if valid {
                    style.column_count = count.unwrap_or(1);
                    assign!(explicit_column_count, count.is_some());
                    assign!(column_width, width);
                }
            }
        }
        "column-count" => {
            if value == "auto" {
                style.column_count = 1;
                assign!(explicit_column_count, false);
            } else if let Some(count) = PositiveInteger(value) {
                style.column_count = count;
                assign!(explicit_column_count, true);
            }
        }
        "column-width" => {
            if value == "auto" {
                assign!(column_width, None);
            } else if let Some(width) = Length(value, font_size).filter(|n| *n > 0.0) {
                assign!(column_width, Some(width));
            }
        }
        "grid-auto-flow" => {
            let parts = SplitWhitespace(value);
            let (mut valid, mut saw_direction, mut column, mut dense) =
                (!parts.is_empty() && parts.len() <= 2, false, false, false);
            for part in &parts {
                if matches!(part.as_str(), "row" | "column") && !saw_direction {
                    saw_direction = true;
                    column = part == "column";
                } else if part == "dense" && !dense {
                    dense = true;
                } else {
                    valid = false;
                }
            }
            if valid {
                assign!(grid_auto_flow_column, column);
                assign!(grid_auto_flow_dense, dense);
            }
        }
        "table-layout" => {
            if matches!(value, "fixed" | "auto") {
                assign!(table_layout_fixed, value == "fixed");
            }
        }
        "empty-cells" => match value {
            "hide" => assign!(empty_cells, EmptyCells::kHide),
            "show" => assign!(empty_cells, EmptyCells::kShow),
            _ => {}
        },
        "margin-trim" => {
            if value == "none" {
                assign!(margin_trim, MarginTrim::kNone);
            } else {
                let (mut trim, mut valid) = (MarginTrim::kNone, true);
                for part in SplitWhitespace(value) {
                    trim = trim
                        | match part.as_str() {
                            "block" => MarginTrim::kBlock,
                            "block-start" => MarginTrim::kBlockStart,
                            "block-end" => MarginTrim::kBlockEnd,
                            _ => {
                                valid = false;
                                MarginTrim::kNone
                            }
                        };
                }
                if valid && trim != MarginTrim::kNone {
                    assign!(margin_trim, trim);
                }
            }
        }
        "overflow-clip-margin" => {
            let parts = SplitWhitespace(value);
            let (mut parsed, mut saw_box, mut saw_length, mut valid) =
                (OverflowClipMargin::default(), false, false, true);
            for part in &parts {
                if matches!(part.as_str(), "border-box" | "padding-box" | "content-box") {
                    if saw_box {
                        valid = false;
                        break;
                    }
                    saw_box = true;
                    parsed.reference_box = match part.as_str() {
                        "border-box" => OverflowClipReferenceBox::kBorderBox,
                        "content-box" => OverflowClipReferenceBox::kContentBox,
                        _ => OverflowClipReferenceBox::kPaddingBox,
                    };
                } else if let Some(length) =
                    Length(part, font_size).filter(|n| *n >= 0.0 && !saw_length)
                {
                    parsed.margin = length;
                    saw_length = true;
                } else {
                    valid = false;
                    break;
                }
            }
            if valid && !parts.is_empty() {
                assign!(overflow_clip_margin, Some(parsed));
            }
        }
        "border-collapse" => {
            if matches!(value, "collapse" | "separate") {
                assign!(border_collapse, value == "collapse");
            }
        }
        "caption-side" => match value {
            "top" => assign!(caption_side, CaptionSide::kTop),
            "bottom" => assign!(caption_side, CaptionSide::kBottom),
            _ => {}
        },
        "border-spacing" => {
            let parts = SplitWhitespace(value);
            if parts.len() == 1 {
                if let Some(length) = Length(&parts[0], font_size).filter(|n| *n >= 0.0) {
                    assign!(border_spacing, length);
                    assign!(vertical_border_spacing, None);
                }
            } else if parts.len() == 2 {
                if let (Some(horizontal), Some(vertical)) = (
                    Length(&parts[0], font_size).filter(|n| *n >= 0.0),
                    Length(&parts[1], font_size).filter(|n| *n >= 0.0),
                ) {
                    assign!(border_spacing, horizontal);
                    assign!(vertical_border_spacing, Some(vertical));
                }
            }
        }
        "column-span" => {
            if matches!(value, "all" | "none") {
                assign!(column_span_all, value == "all");
            }
        }
        "column-fill" => {
            if matches!(value, "balance" | "auto") {
                assign!(column_fill_balance, value == "balance");
            }
        }
        "column-wrap" => match value {
            "auto" => assign!(column_wrap, ColumnWrap::kAuto),
            "nowrap" => assign!(column_wrap, ColumnWrap::kNowrap),
            "wrap" => assign!(column_wrap, ColumnWrap::kWrap),
            _ => {}
        },
        "initial-letter" => {
            if value == "normal" {
                assign!(initial_letter, InitialLetter::default());
            } else {
                let parts = SplitWhitespace(value);
                let mut parsed = InitialLetter::default();
                let mut valid = !parts.is_empty() && parts.len() <= 2;
                if valid && matches!(parts[0].as_str(), "drop" | "raise") {
                    if parts.len() != 2 {
                        valid = false;
                    } else {
                        parsed.sink_type = if parts[0] == "drop" {
                            InitialLetterSink::kDrop
                        } else {
                            InitialLetterSink::kRaise
                        };
                        if let Some(size) = Number(&parts[1]).filter(|n| *n >= 1.0) {
                            parsed.size = size;
                        } else {
                            valid = false;
                        }
                    }
                } else if valid {
                    if let Some(size) = Number(&parts[0]).filter(|n| *n >= 1.0) {
                        parsed.size = size;
                    } else {
                        valid = false;
                    }
                    if valid && parts.len() == 2 {
                        match parts[1].as_str() {
                            "drop" => parsed.sink_type = InitialLetterSink::kDrop,
                            "raise" => parsed.sink_type = InitialLetterSink::kRaise,
                            _ => {
                                if let Some(sink) = PositiveInteger(&parts[1]) {
                                    parsed.sink_type = InitialLetterSink::kInteger;
                                    parsed.sink = sink;
                                } else {
                                    valid = false;
                                }
                            }
                        }
                    }
                }
                if valid {
                    assign!(initial_letter, parsed);
                }
            }
        }
        "break-before" => assign!(break_before, ParseBreakRule(value, false)),
        "break-after" => assign!(break_after, ParseBreakRule(value, false)),
        "break-inside" => assign!(break_inside, ParseBreakRule(value, true)),
        "widows" | "orphans" => {
            if let Some(count) = PositiveInteger(value) {
                if property == "widows" {
                    assign!(widows, count);
                } else {
                    assign!(orphans, count);
                }
            }
        }
        "clear" => {
            if let Some(clear) = match value {
                "none" => Some(ClearSide::kNone),
                "left" => Some(ClearSide::kLeft),
                "right" => Some(ClearSide::kRight),
                "inline-start" => Some(ClearSide::kInlineStart),
                "inline-end" => Some(ClearSide::kInlineEnd),
                "both" => Some(ClearSide::kBoth),
                _ => None,
            } {
                assign!(clear_side, clear);
            }
        }
        "contain" => {
            let parts = SplitWhitespace(value);
            assign!(
                layout_containment,
                matches!(value, "strict" | "content") || parts.iter().any(|p| p == "layout")
            );
            assign!(
                size_containment,
                value == "strict" || parts.iter().any(|p| p == "size")
            );
        }
        "contain-intrinsic-width" | "contain-intrinsic-height" => {
            if let Some(intrinsic) = ParseContainIntrinsicLength(value, font_size) {
                if property == "contain-intrinsic-width" {
                    assign!(contain_intrinsic_width, intrinsic);
                } else {
                    assign!(contain_intrinsic_height, intrinsic);
                }
            }
        }
        "contain-intrinsic-size" => {
            let parts = SplitWhitespace(value);
            let (mut first, mut second) = (None, None);
            if let Some(single) = ParseContainIntrinsicLength(value, font_size) {
                first = Some(single);
                second = Some(single);
            } else {
                for split in 1..parts.len() {
                    if let (Some(left), Some(right)) = (
                        ParseContainIntrinsicLength(&parts[..split].join(" "), font_size),
                        ParseContainIntrinsicLength(&parts[split..].join(" "), font_size),
                    ) {
                        if first.is_some() {
                            first = None;
                            second = None;
                            break;
                        }
                        first = Some(left);
                        second = Some(right);
                    }
                }
            }
            if let (Some(first), Some(second)) = (first, second) {
                assign!(contain_intrinsic_width, first);
                assign!(contain_intrinsic_height, second);
            }
        }
        "aspect-ratio" => {
            let parts = SplitTopLevel(value, b'/');
            if parts.len() == 1 {
                if let Some(ratio) = Number(&parts[0]).filter(|n| *n > 0.0) {
                    assign!(aspect_ratio, Some(ratio));
                }
            } else if parts.len() == 2 {
                if let (Some(numerator), Some(denominator)) = (
                    Number(&parts[0]).filter(|n| *n > 0.0),
                    Number(&parts[1]).filter(|n| *n > 0.0),
                ) {
                    assign!(aspect_ratio, Some(numerator / denominator));
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
    #[test]
    fn sequential_flow_declarations_match_complete_cpp_apply_declaration() {
        let mut style = ComputedStyle::default();
        style
            .extended
            .get_or_insert_with(Default::default)
            .font_size = 24.0;
        let decode = |hex: &str| {
            String::from_utf8(
                hex.as_bytes()
                    .chunks_exact(2)
                    .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
                    .collect(),
            )
            .unwrap()
        };
        for line in
            include_str!("../../../../artifacts/cpp-reference/flow-declaration-results.tsv").lines()
        {
            let fields: Vec<_> = line.split('\t').collect();
            assert!(ApplyFlowProperty(&mut style, fields[0], &decode(fields[1])));
            let e = style.extended.as_ref().unwrap();
            let actual = vec![
                Some(style.position as i32 as f64),
                Some(style.floating as i32 as f64),
                style.paint.z_index.map(|v| v as f64),
                Some(style.column_count as f64),
                Some(e.explicit_column_count as u8 as f64),
                e.column_width,
                Some(e.grid_auto_flow_column as u8 as f64),
                Some(e.grid_auto_flow_dense as u8 as f64),
                Some(e.table_layout_fixed as u8 as f64),
                Some(e.empty_cells as i32 as f64),
                Some(e.margin_trim.0 as f64),
                e.overflow_clip_margin
                    .map(|v| v.reference_box as i32 as f64),
                e.overflow_clip_margin.map(|v| v.margin),
                Some(e.border_collapse as u8 as f64),
                Some(e.caption_side as i32 as f64),
                Some(e.border_spacing),
                e.vertical_border_spacing,
                Some(e.column_span_all as u8 as f64),
                Some(e.column_fill_balance as u8 as f64),
                Some(e.column_wrap as i32 as f64),
                Some(e.initial_letter.size),
                Some(e.initial_letter.sink_type as i32 as f64),
                Some(e.initial_letter.sink as f64),
                Some(e.break_before as i32 as f64),
                Some(e.break_after as i32 as f64),
                Some(e.break_inside as i32 as f64),
                Some(e.widows as f64),
                Some(e.orphans as f64),
                Some(e.clear_side as i32 as f64),
                Some(e.layout_containment as u8 as f64),
                Some(e.size_containment as u8 as f64),
                Some(e.contain_intrinsic_width.has_auto as u8 as f64),
                e.contain_intrinsic_width.length,
                Some(e.contain_intrinsic_height.has_auto as u8 as f64),
                e.contain_intrinsic_height.length,
                e.aspect_ratio,
            ];
            let expected: Vec<_> = fields[2..]
                .iter()
                .map(|v| {
                    if *v == "none" {
                        None
                    } else {
                        Some(v.parse::<f64>().unwrap())
                    }
                })
                .collect();
            assert_eq!(actual, expected, "{} {}", fields[0], decode(fields[1]));
        }
    }
}
