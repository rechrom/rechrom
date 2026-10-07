#![allow(non_snake_case)]
use crate::style_resolver::{
    border_radius::Length, number::Number, selector::SourceSpace, shadow::SplitWhitespace,
};
use css_parser::length_percentage_parser::ParseLengthPercentage;
use layoutng_assembly::internal::layout_input::{
    ComputedStyle, Display, FlexBasisSizing, IntrinsicSizing, TextDirection, WritingMode,
};

// cpp: style_resolver/style_resolver.cc:3646-3676
pub(crate) fn ApplyInsetValue(
    style: &mut ComputedStyle,
    side: usize,
    input: &str,
    font_size: f64,
) -> bool {
    let value = input
        .trim_matches(|c: char| c.source_space())
        .to_ascii_lowercase();
    let (pixels, percentage, calculated) =
        if let Some(length) = ParseLengthPercentage(&value, font_size) {
            (
                length.has_pixels.then_some(length.pixels),
                length.has_percentage.then_some(length.percentage),
                true,
            )
        } else if let Some(number) = value.strip_suffix('%').and_then(Number) {
            (None, Some(number), false)
        } else if let Some(length) = Length(&value, font_size) {
            (Some(length), None, false)
        } else if value == "auto" {
            (None, None, false)
        } else {
            return false;
        };
    match side {
        0 => style.top = pixels,
        1 => style.right = pixels,
        2 => style.bottom = pixels,
        _ => style.left = pixels,
    };
    let extra = style.extended.get_or_insert_with(Default::default);
    extra.inset_percentages[side] = percentage;
    extra.inset_calculated[side] = calculated;
    true
}

// cpp: style_resolver/style_resolver.cc:5144-5188
pub(crate) fn ApplyInset(style: &mut ComputedStyle, property: &str, value: &str) {
    let font_size = style.extended.as_ref().map_or(16.0, |e| e.font_size);
    if property == "inset" {
        let parts = SplitWhitespace(value);
        if parts.is_empty() || parts.len() > 4 {
            return;
        }
        let expanded = [
            0,
            if parts.len() > 1 { 1 } else { 0 },
            if parts.len() > 2 { 2 } else { 0 },
            if parts.len() > 3 {
                3
            } else if parts.len() > 1 {
                1
            } else {
                0
            },
        ];
        let mut candidate = style.clone();
        let mut valid = true;
        for side in 0..4 {
            valid &= ApplyInsetValue(&mut candidate, side, &parts[expanded[side]], font_size);
        }
        if valid {
            *style = candidate;
        }
    } else {
        let side = match property {
            "top" => 0,
            "right" => 1,
            "bottom" => 2,
            "left" => 3,
            _ => return,
        };
        ApplyInsetValue(style, side, value, font_size);
    }
}

// cpp: style_resolver/style_resolver.cc:4647-4718
// None denotes a physical/non-pair property. An empty expansion denotes an
// invalid logical shorthand, which must not mutate either physical side.
pub(crate) fn ExpandLogicalPair(
    style: &ComputedStyle,
    property: &str,
    value: &str,
    mode: WritingMode,
    direction: TextDirection,
) -> Option<Vec<(String, String)>> {
    // Blink resolves the shorthand property ID before expanding its physical
    // sides. Ordinary longhands must not format all logical shorthand names.
    if !matches!(
        property,
        "margin-inline"
            | "margin-block"
            | "padding-inline"
            | "padding-block"
            | "inset-inline"
            | "inset-block"
            | "border-inline"
            | "border-block"
            | "border-inline-width"
            | "border-block-width"
            | "border-inline-style"
            | "border-block-style"
            | "border-inline-color"
            | "border-block-color"
    ) {
        return None;
    }
    use crate::style_resolver::border_longhands::{
        ParseBorder, ParseBorderLineStyle, ParseBorderWidth,
    };
    use crate::style_resolver::css_wide::{LogicalSide::*, PhysicalSideName};
    let pair = |inline| {
        if inline {
            [
                PhysicalSideName(kInlineStart, mode, direction),
                PhysicalSideName(kInlineEnd, mode, direction),
            ]
        } else {
            [
                PhysicalSideName(kBlockStart, mode, direction),
                PhysicalSideName(kBlockEnd, mode, direction),
            ]
        }
    };
    let font_size = style.extended.as_ref().map_or(16.0, |e| e.font_size);
    for prefix in ["margin", "padding", "inset"] {
        for axis in ["inline", "block"] {
            if property != format!("{prefix}-{axis}") {
                continue;
            }
            let parts = SplitWhitespace(value);
            if parts.is_empty()
                || parts.len() > 2
                || parts.iter().any(|p| {
                    crate::style_resolver::edge_values::ParseEdgeValue(
                        p,
                        font_size,
                        prefix != "padding",
                    )
                    .is_none()
                })
            {
                return Some(Vec::new());
            }
            let sides = pair(axis == "inline");
            let prefix = if prefix == "inset" {
                String::new()
            } else {
                format!("{prefix}-")
            };
            return Some(vec![
                (format!("{prefix}{}", sides[0]), parts[0].clone()),
                (
                    format!("{prefix}{}", sides[1]),
                    parts.get(1).unwrap_or(&parts[0]).clone(),
                ),
            ]);
        }
    }
    for axis in ["inline", "block"] {
        let sides = pair(axis == "inline");
        let prefix = format!("border-{axis}");
        if property == prefix {
            return Some(
                if ParseBorder(value, font_size, style.paint.color).is_some() {
                    sides
                        .iter()
                        .map(|side| (format!("border-{side}"), value.to_owned()))
                        .collect()
                } else {
                    Vec::new()
                },
            );
        }
        for suffix in ["width", "style", "color"] {
            if property != format!("{prefix}-{suffix}") {
                continue;
            }
            let parts = SplitWhitespace(value);
            let valid = |p: &str| match suffix {
                "width" => ParseBorderWidth(p, font_size).is_some(),
                "style" => ParseBorderLineStyle(p).is_some(),
                _ => {
                    p == "currentcolor"
                        || layoutng_assembly::css_color_parser::ParseCSSColor(p).is_some()
                }
            };
            if parts.is_empty() || parts.len() > 2 || parts.iter().any(|p| !valid(p)) {
                return Some(Vec::new());
            }
            return Some(vec![
                (format!("border-{}-{suffix}", sides[0]), parts[0].clone()),
                (
                    format!("border-{}-{suffix}", sides[1]),
                    parts.get(1).unwrap_or(&parts[0]).clone(),
                ),
            ]);
        }
    }
    None
}

// cpp: style_resolver/style_resolver.cc:4582-4636
pub(crate) fn ParseBoxDisplay(input: &str) -> Option<Display> {
    use Display::*;
    let value = input
        .trim_matches(|c: char| c.source_space())
        .to_ascii_lowercase();
    let display = match value.as_str() {
        "block" => kBlock,
        "inline" => kInline,
        "inline-block" => kInlineBlock,
        "flow-root" => kFlowRoot,
        "flex" => kFlex,
        "inline-flex" => kInlineFlex,
        "grid" => kGrid,
        "inline-grid" => kInlineGrid,
        "table" => kTable,
        "inline-table" => kInlineTable,
        "ruby" => kRuby,
        "block-ruby" => kBlockRuby,
        "math" => kMath,
        "block-math" => kBlockMath,
        "table-row" => kTableRow,
        "table-cell" => kTableCell,
        "table-row-group" => kTableSection,
        "table-header-group" => kTableHeaderGroup,
        "table-footer-group" => kTableFooterGroup,
        "table-caption" => kTableCaption,
        "table-column-group" => kTableColumnGroup,
        "table-column" => kTableColumn,
        "list-item" => kListItem,
        _ => {
            let parts = SplitWhitespace(&value);
            if !(2..=3).contains(&parts.len()) {
                return None;
            }
            if parts.iter().any(|p| p == "list-item") {
                return parts
                    .iter()
                    .all(|p| {
                        matches!(
                            p.as_str(),
                            "list-item" | "block" | "inline" | "flow" | "flow-root"
                        )
                    })
                    .then_some(kListItem);
            }
            if parts.len() != 2 {
                return None;
            }
            let block = match parts[0].as_str() {
                "block" => true,
                "inline" => false,
                _ => return None,
            };
            return Some(match parts[1].as_str() {
                "flow" => {
                    if block {
                        kBlock
                    } else {
                        kInline
                    }
                }
                "flow-root" => {
                    if block {
                        kFlowRoot
                    } else {
                        kInlineBlock
                    }
                }
                "flex" => {
                    if block {
                        kFlex
                    } else {
                        kInlineFlex
                    }
                }
                "grid" => {
                    if block {
                        kGrid
                    } else {
                        kInlineGrid
                    }
                }
                "table" => {
                    if block {
                        kTable
                    } else {
                        kInlineTable
                    }
                }
                "ruby" => {
                    if block {
                        kBlockRuby
                    } else {
                        kRuby
                    }
                }
                "math" => {
                    if block {
                        kBlockMath
                    } else {
                        kMath
                    }
                }
                _ => return None,
            });
        }
    };
    Some(display)
}

// cpp: style_resolver/style_resolver.cc:4724-4740
pub(crate) fn ApplyDisplay(
    style: &mut ComputedStyle,
    generates_box: &mut bool,
    display_contents: &mut bool,
    value: &str,
) {
    let value = value
        .trim_matches(|c: char| c.source_space())
        .to_ascii_lowercase();
    match value.as_str() {
        "none" => {
            *generates_box = false;
            *display_contents = false;
        }
        "contents" => {
            *generates_box = false;
            *display_contents = true;
        }
        _ => {
            if let Some(display) = ParseBoxDisplay(&value) {
                style.display = display;
                *generates_box = true;
                *display_contents = false;
            }
        }
    }
}

// cpp: style_resolver/style_resolver.cc:4741-4882
// Keep all four pieces of a dimension together until validation succeeds.
// The source clones a candidate style; staging this property's fields gives
// the same atomic replacement without cloning unrelated computed properties.
pub(crate) fn ApplyDimension(style: &mut ComputedStyle, property: &str, input: &str) {
    if !matches!(
        property,
        "width" | "height" | "min-width" | "max-width" | "min-height" | "max-height" | "flex-basis"
    ) {
        return;
    }
    let value = input
        .trim_matches(|c: char| c.source_space())
        .to_ascii_lowercase();
    let font_size = style.extended.as_ref().map_or(16.0, |e| e.font_size);
    let mut pixels = None;
    let mut percentage = None;
    let mut calculated = false;
    let mut intrinsic = IntrinsicSizing::kAuto;
    let mut basis = FlexBasisSizing::kAuto;
    if let Some(length) = ParseLengthPercentage(&value, font_size) {
        pixels = length.has_pixels.then_some(length.pixels);
        percentage = length.has_percentage.then_some(length.percentage);
        calculated = true;
    } else if let Some(number) = value.strip_suffix('%') {
        let Some(number) = Number(number).filter(|n| *n >= 0.0) else {
            return;
        };
        percentage = Some(number);
    } else if property != "flex-basis"
        && matches!(
            value.as_str(),
            "min-content" | "max-content" | "fit-content"
        )
    {
        intrinsic = match value.as_str() {
            "min-content" => IntrinsicSizing::kMinContent,
            "max-content" => IntrinsicSizing::kMaxContent,
            _ => IntrinsicSizing::kFitContent,
        };
    } else if property == "flex-basis"
        && matches!(
            value.as_str(),
            "content" | "min-content" | "max-content" | "fit-content"
        )
    {
        basis = match value.as_str() {
            "content" => FlexBasisSizing::kContent,
            "min-content" => FlexBasisSizing::kMinContent,
            "max-content" => FlexBasisSizing::kMaxContent,
            _ => FlexBasisSizing::kFitContent,
        };
    } else if value == "auto" {
        if matches!(property, "max-width" | "max-height") {
            return;
        }
    } else if value == "none" {
        if !matches!(property, "max-width" | "max-height") {
            return;
        }
    } else {
        let Some(length) = Length(&value, font_size).filter(|n| *n >= 0.0) else {
            return;
        };
        pixels = Some(length);
    }
    let extra = style.extended.get_or_insert_with(Default::default);
    macro_rules! commit {
        ($field:ident, $percent:ident, $calculated:ident, $sizing:ident) => {{
            style.$field = pixels;
            extra.$percent = percentage;
            extra.$calculated = calculated;
            extra.$sizing = intrinsic;
        }};
    }
    match property {
        "width" => commit!(width, width_percent, width_calculated, width_sizing),
        "height" => commit!(height, height_percent, height_calculated, height_sizing),
        "min-width" => commit!(
            min_width,
            min_width_percent,
            min_width_calculated,
            min_width_sizing
        ),
        "max-width" => commit!(
            max_width,
            max_width_percent,
            max_width_calculated,
            max_width_sizing
        ),
        "min-height" => commit!(
            min_height,
            min_height_percent,
            min_height_calculated,
            min_height_sizing
        ),
        "max-height" => commit!(
            max_height,
            max_height_percent,
            max_height_calculated,
            max_height_sizing
        ),
        _ => {
            style.flex_basis = pixels;
            extra.flex_basis_percent = percentage;
            extra.flex_basis_calculated = calculated;
            extra.flex_basis_sizing = basis;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_and_logical_box_cascade_matches_cpp_in_all_six_axes() {
        let mut style = ComputedStyle::default();
        style
            .extended
            .get_or_insert_with(Default::default)
            .font_size = 24.0;
        for line in include_str!("../../../../artifacts/cpp-reference/box-axis-results.tsv").lines()
        {
            let fields: Vec<_> = line.split('\t').collect();
            let mode = match fields[0] {
                "horizontal" => WritingMode::kHorizontalTb,
                "rl" => WritingMode::kVerticalRl,
                _ => WritingMode::kVerticalLr,
            };
            let direction = if fields[1] == "ltr" {
                TextDirection::kLtr
            } else {
                TextDirection::kRtl
            };
            crate::style_resolver::apply_with_axes(
                &mut style,
                fields[2],
                fields[3],
                (1024.0, 768.0),
                mode,
                direction,
            );
            let e = style.extended.as_ref().unwrap();
            let mut actual = Vec::<Option<f64>>::new();
            for edges in [style.margin, style.padding] {
                actual.extend([edges.top, edges.right, edges.bottom, edges.left].map(Some));
            }
            for values in [e.margin_percentages, e.padding_percentages] {
                actual.extend(values);
            }
            for values in [e.margin_calculated, e.padding_calculated, e.margin_auto] {
                actual.extend(values.map(|v| Some(u8::from(v) as f64)));
            }
            actual.extend([style.top, style.right, style.bottom, style.left]);
            actual.extend(e.inset_percentages);
            actual.extend(e.inset_calculated.map(|v| Some(u8::from(v) as f64)));
            actual.extend(
                [
                    style.border.top,
                    style.border.right,
                    style.border.bottom,
                    style.border.left,
                ]
                .map(Some),
            );
            actual.extend(style.border_styles.map(|v| Some(v as i32 as f64)));
            for color in style.paint.border_colors {
                actual.extend(
                    [color.red, color.green, color.blue, color.alpha].map(|v| Some(v as f64)),
                );
            }
            actual.extend([
                Some(style.paint.outline_width),
                Some(style.paint.outline_style as i32 as f64),
            ]);
            let color = style.paint.outline_color;
            actual
                .extend([color.red, color.green, color.blue, color.alpha].map(|v| Some(v as f64)));
            actual.push(Some(style.paint.outline_offset));
            actual.extend([
                Some(style.paint.column_rule_width),
                Some(style.paint.column_rule_style as i32 as f64),
            ]);
            let color = style.paint.column_rule_color;
            actual
                .extend([color.red, color.green, color.blue, color.alpha].map(|v| Some(v as f64)));
            let expected: Vec<_> = fields[4..]
                .iter()
                .map(|v| {
                    if *v == "none" {
                        None
                    } else {
                        Some(v.parse::<f64>().unwrap())
                    }
                })
                .collect();
            assert_eq!(actual, expected, "{line}");
        }
    }

    #[test]
    fn dimensions_match_frozen_cpp_sequential_cascade() {
        let mut style = ComputedStyle::default();
        for line in
            include_str!("../../../../artifacts/cpp-reference/box-dimension-results.tsv").lines()
        {
            let fields: Vec<_> = line.split('\t').collect();
            ApplyDimension(&mut style, fields[0], fields[1]);
            let e = style.extended.as_ref().unwrap();
            let (pixels, percent, calculated, sizing) = match fields[0] {
                "width" => (
                    style.width,
                    e.width_percent,
                    e.width_calculated,
                    e.width_sizing as i32,
                ),
                "height" => (
                    style.height,
                    e.height_percent,
                    e.height_calculated,
                    e.height_sizing as i32,
                ),
                "min-width" => (
                    style.min_width,
                    e.min_width_percent,
                    e.min_width_calculated,
                    e.min_width_sizing as i32,
                ),
                "max-width" => (
                    style.max_width,
                    e.max_width_percent,
                    e.max_width_calculated,
                    e.max_width_sizing as i32,
                ),
                "min-height" => (
                    style.min_height,
                    e.min_height_percent,
                    e.min_height_calculated,
                    e.min_height_sizing as i32,
                ),
                "max-height" => (
                    style.max_height,
                    e.max_height_percent,
                    e.max_height_calculated,
                    e.max_height_sizing as i32,
                ),
                _ => (
                    style.flex_basis,
                    e.flex_basis_percent,
                    e.flex_basis_calculated,
                    e.flex_basis_sizing as i32,
                ),
            };
            let number = |v: &str| {
                if v == "none" {
                    None
                } else {
                    Some(v.parse::<f64>().unwrap())
                }
            };
            assert_eq!(pixels, number(fields[2]), "{line}");
            assert_eq!(percent, number(fields[3]), "{line}");
            assert_eq!(calculated, fields[4] == "1", "{line}");
            assert_eq!(sizing, fields[5].parse::<i32>().unwrap(), "{line}");
        }
    }

    #[test]
    fn display_matches_frozen_cpp_box_flags() {
        let mut style = ComputedStyle::default();
        let (mut generates, mut contents) = (true, false);
        for line in
            include_str!("../../../../artifacts/cpp-reference/box-display-results.tsv").lines()
        {
            let fields: Vec<_> = line.split('\t').collect();
            ApplyDisplay(&mut style, &mut generates, &mut contents, fields[0]);
            assert_eq!(
                style.display as i32,
                fields[1].parse::<i32>().unwrap(),
                "{line}"
            );
            assert_eq!(generates, fields[2] == "1", "{line}");
            assert_eq!(contents, fields[3] == "1", "{line}");
        }
    }
}
