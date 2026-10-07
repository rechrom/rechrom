#![allow(non_snake_case)]
use crate::style_resolver::{
    border_radius::Length,
    fonts,
    number::{Number, PositiveInteger},
    selector::SourceSpace,
    shadow::SplitWhitespace,
};
use css_parser::length_percentage_parser::ParseLengthPercentage;
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::{css_color_parser::ParseCSSColor, internal::layout_input::*};

// cpp: style_resolver/style_resolver.cc:4473-4481
fn ParseOverflow(value: &str) -> Option<Overflow> {
    Some(match value {
        "visible" => Overflow::kVisible,
        "hidden" => Overflow::kHidden,
        "clip" => Overflow::kClip,
        "scroll" => Overflow::kScroll,
        "auto" | "overlay" => Overflow::kAuto,
        _ => return None,
    })
}

// cpp: style_resolver/style_resolver.cc:2265-2315
fn ParseTextDecoration(value: &str, current: Color) -> Option<TextDecorationPaint> {
    let mut decoration = TextDecorationPaint::default();
    let (mut saw_none, mut saw_style, mut saw_color, mut saw_line) = (false, false, false, false);
    let parts = SplitWhitespace(value);
    if parts.is_empty() {
        return None;
    }
    for part in &parts {
        if part == "none" {
            if saw_none || saw_line {
                return None;
            }
            saw_none = true;
        } else if matches!(part.as_str(), "underline" | "overline" | "line-through") {
            if saw_none {
                return None;
            }
            let flag = match part.as_str() {
                "underline" => &mut decoration.underline,
                "overline" => &mut decoration.overline,
                _ => &mut decoration.line_through,
            };
            if *flag {
                return None;
            }
            *flag = true;
            saw_line = true;
        } else if !saw_style
            && matches!(
                part.as_str(),
                "solid" | "double" | "dotted" | "dashed" | "wavy"
            )
        {
            decoration.style = DecorationStyle(part).unwrap();
            saw_style = true;
        } else if !saw_color {
            decoration.color = Some(if part == "currentcolor" {
                current
            } else {
                ParseCSSColor(part)?
            });
            saw_color = true;
        } else {
            return None;
        }
    }
    Some(decoration)
}
fn DecorationStyle(value: &str) -> Option<TextDecorationStyle> {
    Some(match value {
        "solid" => TextDecorationStyle::kSolid,
        "double" => TextDecorationStyle::kDouble,
        "dotted" => TextDecorationStyle::kDotted,
        "dashed" => TextDecorationStyle::kDashed,
        "wavy" => TextDecorationStyle::kWavy,
        _ => return None,
    })
}

// cpp: style_resolver/style_resolver.cc:5564-5965,6141-6203
// Return true for a handled property even if its value is invalid. Source
// invalid declarations leave the previous computed value intact.
pub(crate) fn ApplyTextProperty(style: &mut ComputedStyle, property: &str, raw: &str) -> bool {
    let raw = raw.trim_matches(|c: char| c.source_space());
    let lower = raw.to_ascii_lowercase();
    let value = lower.as_str();
    let font_size = fonts::FontSizeOf(style);
    macro_rules! assign {
        ($field:ident, $value:expr) => {
            style.extended.get_or_insert_with(Default::default).$field = $value
        };
    }
    macro_rules! keywords { ($field:ident, $kind:ident, $( $name:literal => $variant:ident ),+ $(,)?) => {
        match value { $( $name => assign!($field, $kind::$variant), )+ _ => {} }
    }; }
    match property {
        "font" => {
            if let Some(parsed) = fonts::ParseFontShorthand(raw, font_size) {
                assign!(font_italic, parsed.italic);
                assign!(font_families, parsed.families);
                fonts::ApplyLineHeightValue(
                    style,
                    parsed.line_height.as_deref().unwrap_or("normal"),
                    font_size,
                );
            }
        }
        "font-size" => {
            if let Some(size) = Length(value, font_size).filter(|n| *n > 0.0) {
                assign!(font_size, size);
            }
        }
        "font-family" => {
            if let Some(families) = fonts::ParseFontFamilies(raw) {
                assign!(font_families, families);
            }
        }
        "font-weight" => {
            let weight = match value {
                "normal" => Some(400.0),
                "bold" => Some(700.0),
                _ => Number(value).filter(|n| *n >= 1.0 && *n <= 1000.0),
            };
            if let Some(weight) = weight {
                assign!(font_weight, weight);
            }
        }
        "font-style" => {
            if value == "normal" {
                assign!(font_italic, false);
            } else if value == "italic" || value.starts_with("oblique") {
                assign!(font_italic, true);
            }
        }
        "-webkit-font-smoothing" => {
            keywords!(font_smoothing, FontSmoothing, "auto" => kAuto, "none" => kNone, "antialiased" => kAntialiased, "subpixel-antialiased" => kSubpixelAntialiased)
        }
        "line-height" => fonts::ApplyLineHeightValue(style, value, font_size),
        "letter-spacing" | "word-spacing" => {
            if let Some(length) = if value == "normal" {
                Some(0.0)
            } else {
                Length(value, font_size)
            } {
                if property == "letter-spacing" {
                    assign!(letter_spacing, length);
                } else {
                    assign!(word_spacing, length);
                }
            }
        }
        "text-indent" => {
            let parts = SplitWhitespace(value);
            let (mut pixels, mut percent, mut calculated, mut each_line, mut hanging) =
                (None, None, false, false, false);
            let mut valid = !parts.is_empty();
            for part in &parts {
                if part == "each-line" {
                    if each_line {
                        valid = false;
                    }
                    each_line = true;
                } else if part == "hanging" {
                    if hanging {
                        valid = false;
                    }
                    hanging = true;
                } else if pixels.is_some() || percent.is_some() || calculated {
                    valid = false;
                } else if let Some(length) = ParseLengthPercentage(part, font_size) {
                    pixels = length.has_pixels.then_some(length.pixels);
                    percent = length.has_percentage.then_some(length.percentage);
                    calculated = true;
                } else if let Some(number) = part.strip_suffix('%').and_then(Number) {
                    percent = Some(number);
                } else if let Some(length) = Length(part, font_size) {
                    pixels = Some(length);
                } else {
                    valid = false;
                }
            }
            if valid && (pixels.is_some() || percent.is_some() || calculated) {
                let e = style.extended.get_or_insert_with(Default::default);
                e.text_indent = pixels.unwrap_or(0.0);
                e.text_indent_percent = percent;
                e.text_indent_calculated = calculated;
                e.text_indent_each_line = each_line;
                e.text_indent_hanging = hanging;
            }
        }
        "white-space" => {
            let parsed = match value {
                "normal" => Some(WhiteSpace::kNormal),
                "nowrap" => Some(WhiteSpace::kNowrap),
                "pre" => Some(WhiteSpace::kPre),
                "pre-line" => Some(WhiteSpace::kPreLine),
                "pre-wrap" => Some(WhiteSpace::kPreWrap),
                "break-spaces" => Some(WhiteSpace::kBreakSpaces),
                _ => None,
            };
            if let Some(parsed) = parsed {
                assign!(white_space, parsed);
                assign!(text_wrap_mode, TextWrapMode::kFromWhiteSpace);
            }
        }
        "text-wrap-mode" => {
            keywords!(text_wrap_mode, TextWrapMode, "wrap" => kWrap, "nowrap" => kNowrap)
        }
        "text-wrap-style" => {
            keywords!(text_wrap_style, TextWrapStyle, "auto" => kAuto, "pretty" => kPretty, "balance" => kBalance, "stable" => kStable)
        }
        "text-wrap" => {
            let parts = SplitWhitespace(value);
            let (mut mode, mut wrap_style) = (TextWrapMode::kWrap, TextWrapStyle::kAuto);
            let (mut saw_mode, mut saw_style) = (false, false);
            let mut valid = !parts.is_empty() && parts.len() <= 2;
            for part in &parts {
                if matches!(part.as_str(), "wrap" | "nowrap") {
                    if saw_mode {
                        valid = false;
                    }
                    saw_mode = true;
                    mode = if part == "wrap" {
                        TextWrapMode::kWrap
                    } else {
                        TextWrapMode::kNowrap
                    };
                } else if matches!(part.as_str(), "auto" | "pretty" | "balance" | "stable") {
                    if saw_style {
                        valid = false;
                    }
                    saw_style = true;
                    wrap_style = match part.as_str() {
                        "auto" => TextWrapStyle::kAuto,
                        "pretty" => TextWrapStyle::kPretty,
                        "balance" => TextWrapStyle::kBalance,
                        _ => TextWrapStyle::kStable,
                    };
                } else {
                    valid = false;
                }
            }
            if valid {
                assign!(text_wrap_mode, mode);
                assign!(text_wrap_style, wrap_style);
            }
        }
        "text-align" => {
            keywords!(text_align, TextAlign, "start" => kStart, "end" => kEnd, "left" => kLeft, "right" => kRight, "center" => kCenter, "justify" => kJustify, "match-parent" => kMatchParent, "-webkit-left" => kWebkitLeft, "-webkit-right" => kWebkitRight, "-webkit-center" => kWebkitCenter)
        }
        "text-align-last" => {
            keywords!(text_align_last, TextAlignLast, "auto" => kAuto, "start" => kStart, "end" => kEnd, "left" => kLeft, "right" => kRight, "center" => kCenter, "justify" => kJustify, "match-parent" => kMatchParent)
        }
        "vertical-align" => {
            let mut e = style.extended.clone().unwrap_or_default();
            e.vertical_align_length = None;
            e.vertical_align_percent = None;
            e.vertical_align_calculated = false;
            let keyword = match value {
                "baseline" => Some(VerticalAlign::kBaseline),
                "middle" => Some(VerticalAlign::kMiddle),
                "sub" => Some(VerticalAlign::kSub),
                "super" => Some(VerticalAlign::kSuper),
                "text-top" => Some(VerticalAlign::kTextTop),
                "text-bottom" => Some(VerticalAlign::kTextBottom),
                "top" => Some(VerticalAlign::kTop),
                "bottom" => Some(VerticalAlign::kBottom),
                "-webkit-baseline-middle" => Some(VerticalAlign::kBaselineMiddle),
                _ => None,
            };
            if let Some(keyword) = keyword {
                e.vertical_align = keyword;
            } else if let Some(length) = ParseLengthPercentage(value, font_size) {
                e.vertical_align_length = length.has_pixels.then_some(length.pixels);
                e.vertical_align_percent = length.has_percentage.then_some(length.percentage);
                e.vertical_align_calculated = true;
            } else if let Some(percent) = value.strip_suffix('%').and_then(Number) {
                e.vertical_align_percent = Some(percent);
            } else if let Some(length) = Length(value, font_size) {
                e.vertical_align_length = Some(length);
            } else {
                return true;
            }
            style.extended = Some(e);
        }
        "unicode-bidi" => {
            keywords!(unicode_bidi, UnicodeBidi, "isolate" => kIsolate, "isolate-override" => kIsolateOverride, "embed" => kEmbed, "bidi-override" => kOverride, "plaintext" => kPlaintext, "normal" => kNormal)
        }
        "overflow-wrap" => {
            keywords!(overflow_wrap, OverflowWrap, "normal" => kNormal, "break-word" => kBreakWord, "anywhere" => kAnywhere)
        }
        "word-break" => {
            keywords!(word_break, WordBreak, "normal" => kNormal, "break-all" => kBreakAll, "keep-all" => kKeepAll, "auto-phrase" => kAutoPhrase, "break-word" => kBreakWord)
        }
        "line-break" => {
            keywords!(line_break, LineBreak, "auto" => kAuto, "loose" => kLoose, "normal" => kNormal, "strict" => kStrict, "anywhere" => kAnywhere)
        }
        "hyphens" => {
            keywords!(hyphens, Hyphens, "none" => kNone, "manual" => kManual, "auto" => kAuto)
        }
        "tab-size" => {
            if let Some(number) = Number(value).filter(|n| *n >= 0.0) {
                assign!(tab_size, number);
                assign!(tab_size_is_length, false);
            } else if let Some(length) = Length(value, font_size).filter(|n| *n >= 0.0) {
                assign!(tab_size, length);
                assign!(tab_size_is_length, true);
            }
        }
        "text-orientation" => {
            keywords!(text_orientation, TextOrientation, "mixed" => kMixed, "upright" => kUpright, "sideways" => kSideways, "sideways-right" => kSideways)
        }
        "text-combine-upright" | "-webkit-text-combine" => {
            if value == "none" {
                assign!(text_combine, TextCombine::kNone);
            } else if value == "all" || value.starts_with("digits") {
                assign!(text_combine, TextCombine::kAll);
            }
        }
        "text-transform" => {
            if value == "none" {
                assign!(text_transform, TextTransform::kNone);
            } else {
                let parts = SplitWhitespace(value);
                let (mut transform, mut valid, mut has_case) =
                    (TextTransform::kNone, !parts.is_empty(), false);
                for part in &parts {
                    let bits = match part.as_str() {
                        "capitalize" | "uppercase" | "lowercase" => {
                            if has_case {
                                valid = false;
                                break;
                            }
                            has_case = true;
                            match part.as_str() {
                                "capitalize" => TextTransform::kCapitalize,
                                "uppercase" => TextTransform::kUppercase,
                                _ => TextTransform::kLowercase,
                            }
                        }
                        "full-width" => TextTransform::kFullWidth,
                        "full-size-kana" => TextTransform::kFullSizeKana,
                        "math-auto" => TextTransform::kMathAuto,
                        _ => {
                            valid = false;
                            break;
                        }
                    };
                    transform = transform | bits;
                }
                if valid {
                    assign!(text_transform, transform);
                }
            }
        }
        "ruby-position" | "-webkit-ruby-position" => {
            keywords!(ruby_position, RubyPosition, "over" => kOver, "before" => kOver, "under" => kUnder, "after" => kUnder)
        }
        "ruby-align" => {
            keywords!(ruby_align, RubyAlign, "center" => kCenter, "start" => kStart, "space-between" => kSpaceBetween, "space-around" => kSpaceAround)
        }
        "ruby-overhang" => {
            keywords!(ruby_overhang, RubyOverhang, "none" => kNone, "auto" => kAuto, "space" => kSpaces, "spaces" => kSpaces)
        }
        "line-clamp" | "-webkit-line-clamp" => {
            if value == "none" {
                assign!(line_clamp, 0);
            } else if let Some(count) = PositiveInteger(value) {
                assign!(line_clamp, count);
            }
        }
        "overflow" => {
            let parts = SplitWhitespace(value);
            if parts.len() == 1 || parts.len() == 2 {
                if let (Some(x), Some(y)) = (
                    ParseOverflow(&parts[0]),
                    ParseOverflow(parts.get(1).unwrap_or(&parts[0])),
                ) {
                    assign!(overflow_x, x);
                    assign!(overflow_y, y);
                }
            }
        }
        "overflow-x" | "overflow-y" => {
            if let Some(overflow) = ParseOverflow(value) {
                if property == "overflow-x" {
                    assign!(overflow_x, overflow);
                } else {
                    assign!(overflow_y, overflow);
                }
            }
        }
        "scrollbar-width" => {
            keywords!(scrollbar_width, ScrollbarWidth, "thin" => kThin, "none" => kNone, "auto" => kAuto)
        }
        "scrollbar-gutter" => {
            keywords!(scrollbar_gutter, ScrollbarGutter, "stable both-edges" => kStableBothEdges, "stable" => kStable, "auto" => kAuto)
        }
        "field-sizing" => {
            keywords!(field_sizing, FieldSizing, "content" => kContent, "fixed" => kFixed)
        }
        "box-sizing" => {
            keywords!(box_sizing, BoxSizing, "border-box" => kBorderBox, "content-box" => kContentBox)
        }
        "list-style-type" => {
            keywords!(list_style_type, ListStyleType, "disc" => kDisc, "circle" => kCircle, "square" => kSquare, "decimal" => kDecimal, "lower-alpha" => kLowerAlpha, "lower-latin" => kLowerAlpha, "none" => kNone)
        }
        "list-style-position" => {
            keywords!(list_style_position, ListStylePosition, "inside" => kInside, "outside" => kOutside)
        }
        "list-style" => {
            // Preserve the source grammar: lower-alpha/lower-latin are accepted
            // by the longhand only, despite an unreachable shorthand ternary.
            let mut e = style.extended.clone().unwrap_or_default();
            let (mut valid, mut saw_type, mut saw_position) = (true, false, false);
            for part in SplitWhitespace(value) {
                if !saw_position && matches!(part.as_str(), "inside" | "outside") {
                    e.list_style_position = if part == "inside" {
                        ListStylePosition::kInside
                    } else {
                        ListStylePosition::kOutside
                    };
                    saw_position = true;
                } else if !saw_type
                    && matches!(
                        part.as_str(),
                        "disc" | "circle" | "square" | "decimal" | "none"
                    )
                {
                    e.list_style_type = match part.as_str() {
                        "disc" => ListStyleType::kDisc,
                        "circle" => ListStyleType::kCircle,
                        "square" => ListStyleType::kSquare,
                        "decimal" => ListStyleType::kDecimal,
                        _ => ListStyleType::kNone,
                    };
                    saw_type = true;
                } else {
                    valid = false;
                    break;
                }
            }
            if valid && (saw_type || saw_position) {
                style.extended = Some(e);
            }
        }
        "box-decoration-break" => {
            keywords!(box_decoration_break, BoxDecorationBreak, "clone" => kClone, "slice" => kSlice)
        }
        "text-decoration" => {
            if let Some(parsed) = ParseTextDecoration(value, style.paint.color) {
                style.paint.text_decoration = parsed;
            }
        }
        "text-decoration-line" => {
            let mut candidate = style.paint.text_decoration;
            candidate.underline = false;
            candidate.overline = false;
            candidate.line_through = false;
            let (mut valid, mut saw_line) = (true, false);
            let parts = SplitWhitespace(value);
            if parts.len() == 1 && parts[0] == "none" {
                saw_line = true;
            } else {
                for part in &parts {
                    let flag = match part.as_str() {
                        "underline" => &mut candidate.underline,
                        "overline" => &mut candidate.overline,
                        "line-through" => &mut candidate.line_through,
                        _ => {
                            valid = false;
                            break;
                        }
                    };
                    if *flag {
                        valid = false;
                        break;
                    }
                    *flag = true;
                    saw_line = true;
                }
            }
            if valid && saw_line {
                style.paint.text_decoration = candidate;
            }
        }
        "text-decoration-style" => {
            if let Some(parsed) = DecorationStyle(value) {
                style.paint.text_decoration.style = parsed;
            }
        }
        "text-decoration-color" => {
            if let Some(color) = if value == "currentcolor" {
                Some(style.paint.color)
            } else {
                ParseCSSColor(value)
            } {
                style.paint.text_decoration.color = Some(color);
            }
        }
        "text-decoration-thickness" => {
            if matches!(value, "auto" | "from-font") {
                style.paint.text_decoration.thickness = None;
            } else if let Some(length) = Length(value, font_size).filter(|n| *n >= 0.0) {
                style.paint.text_decoration.thickness = Some(length);
            }
        }
        "text-underline-offset" => {
            if value == "auto" {
                style.paint.text_decoration.underline_offset = 0.0;
                style.paint.text_decoration.underline_offset_auto = true;
            } else if let Some(length) = Length(value, font_size) {
                style.paint.text_decoration.underline_offset = length;
                style.paint.text_decoration.underline_offset_auto = false;
            }
        }
        "text-decoration-skip-ink" => {
            if value == "none" {
                style.paint.text_decoration.skip_ink = false;
            } else if matches!(value, "auto" | "all") {
                style.paint.text_decoration.skip_ink = true;
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
    fn sequential_text_declarations_match_complete_cpp_apply_declaration() {
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
            include_str!("../../../../artifacts/cpp-reference/text-declaration-results.tsv").lines()
        {
            let fields: Vec<_> = line.split('\t').collect();
            assert!(ApplyTextProperty(&mut style, fields[0], &decode(fields[1])));
            let e = style.extended.as_ref().unwrap();
            let mut values = vec![
                Some(e.font_size),
                Some(e.font_weight),
                Some(e.font_italic as i32 as f64),
                Some(e.font_smoothing as i32 as f64),
                Some(e.letter_spacing),
                Some(e.word_spacing),
                Some(e.text_indent),
                Some(e.text_indent_calculated as i32 as f64),
                Some(e.text_indent_each_line as i32 as f64),
                Some(e.text_indent_hanging as i32 as f64),
                Some(e.white_space as i32 as f64),
                Some(e.text_wrap_mode as i32 as f64),
                Some(e.text_wrap_style as i32 as f64),
                Some(e.text_align as i32 as f64),
                Some(e.text_align_last as i32 as f64),
                Some(e.vertical_align as i32 as f64),
                Some(e.vertical_align_calculated as i32 as f64),
                Some(e.unicode_bidi as i32 as f64),
                Some(e.overflow_wrap as i32 as f64),
                Some(e.word_break as i32 as f64),
                Some(e.line_break as i32 as f64),
                Some(e.hyphens as i32 as f64),
                Some(e.tab_size),
                Some(e.tab_size_is_length as i32 as f64),
                Some(e.text_orientation as i32 as f64),
                Some(e.text_combine as i32 as f64),
                Some(e.ruby_position as i32 as f64),
                Some(e.ruby_align as i32 as f64),
                Some(e.ruby_overhang as i32 as f64),
                Some(e.line_clamp as i32 as f64),
                Some(e.overflow_x as i32 as f64),
                Some(e.overflow_y as i32 as f64),
                Some(e.scrollbar_width as i32 as f64),
                Some(e.scrollbar_gutter as i32 as f64),
                Some(e.field_sizing as i32 as f64),
                Some(e.box_sizing as i32 as f64),
                Some(e.list_style_type as i32 as f64),
                Some(e.list_style_position as i32 as f64),
                Some(e.box_decoration_break as i32 as f64),
            ];
            values.push(Some(e.text_transform.0 as f64));
            values.extend([
                e.line_height,
                e.line_height_percent,
                e.text_indent_percent,
                e.vertical_align_length,
                e.vertical_align_percent,
            ]);
            let d = style.paint.text_decoration;
            values.extend([
                Some(d.underline as u8 as f64),
                Some(d.overline as u8 as f64),
                Some(d.line_through as u8 as f64),
                Some(d.style as i32 as f64),
            ]);
            if let Some(c) = d.color {
                values.extend([c.red, c.green, c.blue, c.alpha].map(|v| Some(v as f64)));
            } else {
                values.extend([None; 4]);
            }
            values.extend([
                d.thickness,
                Some(d.underline_offset),
                Some(d.underline_offset_auto as u8 as f64),
                Some(d.skip_ink as u8 as f64),
            ]);
            let expected: Vec<_> = fields[2..fields.len() - 1]
                .iter()
                .map(|v| {
                    if *v == "none" {
                        None
                    } else {
                        Some(v.parse::<f64>().unwrap())
                    }
                })
                .collect();
            assert_eq!(values, expected, "{} {}", fields[0], decode(fields[1]));
            assert_eq!(
                e.font_families.join("|"),
                decode(fields[fields.len() - 1]),
                "{line}"
            );
        }
    }
}
