use crate::persistent_document::{DOMNamespace, PseudoElement};
use crate::{Element, ParsedDocument as Document};
use css_parser::{Rule, Selector};
use layoutng_assembly::internal::layout_input::{ComputedStyle, Display, Edges};
use layoutng_assembly::internal::paint_input::{PaintFilterOperation, PaintFilterType};
use layoutng_assembly::internal::paint_input::{SvgStrokeLineCap, SvgStrokeLineJoin};
use std::collections::HashMap;

pub use css_parser::length_percentage_parser;
#[path = "../../../css_parser/src/transform_parser.rs"]
mod transform_parser;

// cpp: style_resolver/style_resolver.h:13-13
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CSSMediaType {
    #[default]
    kScreen,
    kPrint,
}

// cpp: style_resolver/style_resolver.h:15-25
/// A preference supplied by the embedding application, never read from the OS.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PreferredColorScheme {
    #[default]
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StyleEnvironment {
    pub media_type: CSSMediaType,
    pub viewport_width: Option<f64>,
    pub viewport_height: Option<f64>,
    pub resolution_dppx: Option<f64>,
    pub preferred_color_scheme: PreferredColorScheme,
}

/// Shared pure media-query evaluation for CSS rules and Window.matchMedia.
pub fn MediaQueryMatches(query: &str, environment: &StyleEnvironment) -> bool {
    media::MediaConditionMatches(query, environment)
}

// cpp: style_resolver/style_resolver.h:30-30
// cpp: style_resolver/style_resolver.cc:8083-8085
#[allow(non_snake_case)]
pub fn AddStyleSheet(owner: &mut crate::persistent_document::DOM, sheet: cssom::CSSStyleSheet) {
    owner.GetDocumentMut().AppendStyleSheet(sheet);
}

mod alignment_properties;
mod background;
mod border_longhands;
mod border_radius;
mod box_properties;
mod cascade;
mod clip_path;
mod css_wide;
mod cursor;
mod edge_values;
mod finish_style;
mod flex_properties;
mod flow_properties;
mod fonts;
mod gap;
mod generated_content;
mod grid_properties;
pub mod initial_style;
mod linear_gradient;
mod media;
mod number;
mod persistent_cascade;
mod persistent_resolver;
pub use persistent_resolver::{ResolveComputedStyles, ResolveDocumentStyles, StyleEngine};
pub mod persistent_selector;
pub mod selector;
mod shadow;
mod source_supported_properties;
mod style_rule_index;
mod text_properties;
mod transform_origin;
mod variables;
mod zoom;

// cpp: style_resolver/style_resolver.cc:911-990
fn resolve_viewport_relative_lengths(input: &str, width: f64, height: f64) -> String {
    if !width.is_finite() || !height.is_finite() {
        return input.to_owned();
    }
    let bytes = input.as_bytes();
    let mut result = String::with_capacity(input.len());
    let mut cursor = 0;
    while cursor < bytes.len() {
        if matches!(bytes[cursor], b'\'' | b'"') {
            let quote = bytes[cursor];
            let start = cursor;
            cursor += 1;
            while cursor < bytes.len() {
                let current = bytes[cursor];
                cursor += 1;
                if current == b'\\' && cursor < bytes.len() {
                    cursor += 1;
                    continue;
                }
                if current == quote {
                    break;
                }
            }
            result.push_str(&input[start..cursor]);
            continue;
        }
        let number_start =
            bytes[cursor].is_ascii_digit() || matches!(bytes[cursor], b'.' | b'+' | b'-');
        let token_start = cursor == 0
            || !(bytes[cursor - 1].is_ascii_alphanumeric()
                || matches!(bytes[cursor - 1], b'_' | b'#'));
        if !number_start || !token_start {
            let character = input[cursor..].chars().next().unwrap();
            result.push(character);
            cursor += character.len_utf8();
            continue;
        }
        let start = cursor;
        let mut number_end = cursor;
        if matches!(bytes[number_end], b'+' | b'-') {
            number_end += 1;
        }
        while number_end < bytes.len() && bytes[number_end].is_ascii_digit() {
            number_end += 1;
        }
        if number_end < bytes.len() && bytes[number_end] == b'.' {
            number_end += 1;
            while number_end < bytes.len() && bytes[number_end].is_ascii_digit() {
                number_end += 1;
            }
        }
        if number_end < bytes.len() && matches!(bytes[number_end], b'e' | b'E') {
            let exponent = number_end;
            number_end += 1;
            if number_end < bytes.len() && matches!(bytes[number_end], b'+' | b'-') {
                number_end += 1;
            }
            let digits = number_end;
            while number_end < bytes.len() && bytes[number_end].is_ascii_digit() {
                number_end += 1;
            }
            if digits == number_end {
                number_end = exponent;
            }
        }
        let number = input[start..number_end].parse::<f64>().ok();
        let Some(number) = number.filter(|number| number.is_finite()) else {
            let character = input[cursor..].chars().next().unwrap();
            result.push(character);
            cursor += character.len_utf8();
            continue;
        };
        let suffix = input[number_end..].get(..4).unwrap_or(&input[number_end..]);
        let suffix = suffix.to_ascii_lowercase();
        let unit = [
            "vmin", "vmax", "dvw", "dvh", "svw", "svh", "lvw", "lvh", "vw", "vh",
        ]
        .into_iter()
        .find(|candidate| suffix.starts_with(candidate));
        let Some(unit) = unit else {
            let character = input[cursor..].chars().next().unwrap();
            result.push(character);
            cursor += character.len_utf8();
            continue;
        };
        let unit_end = number_end + unit.len();
        if unit_end < bytes.len()
            && (bytes[unit_end].is_ascii_alphanumeric() || matches!(bytes[unit_end], b'_' | b'-'))
        {
            let character = input[cursor..].chars().next().unwrap();
            result.push(character);
            cursor += character.len_utf8();
            continue;
        }
        let basis = if unit == "vh" || unit.ends_with("vh") {
            height
        } else if unit == "vw" || unit.ends_with("vw") {
            width
        } else if unit == "vmin" {
            width.min(height)
        } else {
            width.max(height)
        };
        result.push_str(&format!("{:.6}px", number * basis / 100.0));
        cursor = unit_end;
    }
    result
}

fn apply(style: &mut ComputedStyle, property: &str, value: &str, viewport: (f64, f64)) {
    let resolved = resolve_viewport_relative_lengths(value, viewport.0, viewport.1);
    let raw_value = resolved.trim_matches(|c: char| selector::SourceSpace::source_space(c));
    let normalized = raw_value.to_ascii_lowercase();
    let value = match property {
        "font-family" | "background" | "background-image" | "mask-image" | "-webkit-mask-image"
        | "shape-outside" | "clip-path" => raw_value,
        _ => normalized.as_str(),
    };
    let font_size = style
        .extended
        .as_ref()
        .map_or(16.0, |extra| extra.font_size);
    if text_properties::ApplyTextProperty(style, property, raw_value) {
        return;
    }
    if flow_properties::ApplyFlowProperty(style, property, raw_value) {
        return;
    }
    if grid_properties::ApplyGridProperty(style, property, raw_value) {
        return;
    }
    if alignment_properties::ApplyAlignmentProperty(style, property, raw_value)
        || flex_properties::ApplyFlexProperty(style, property, raw_value)
    {
        return;
    }
    match property {
        // cpp: style_resolver/style_resolver.cc:6253-6259,7504-7514
        // ApplyDeclaration has no content field on ordinary element styles.
        // Pseudo resolution consumes it separately through generated_content.
        "content" => {}
        "zoom" => zoom::ApplyZoom(style, value),
        "width" | "height" | "min-width" | "max-width" | "min-height" | "max-height"
        | "flex-basis" => {
            box_properties::ApplyDimension(style, property, value);
        }
        "margin" | "margin-top" | "margin-right" | "margin-bottom" | "margin-left" | "padding"
        | "padding-top" | "padding-right" | "padding-bottom" | "padding-left" => {
            edge_values::ApplyEdges(style, property, value);
        }
        "display" => {
            if let Some(display) = box_properties::ParseBoxDisplay(value) {
                style.display = display;
            }
        }
        // cpp: style_resolver/style_resolver.cc:5144-5184
        "inset" | "top" | "right" | "bottom" | "left" => {
            box_properties::ApplyInset(style, property, value)
        }
        // cpp: style_resolver/style_resolver.cc:6220-6228
        "transform" => {
            if value == "none" {
                style.paint.transform = None;
            } else if let Some(transform) = transform_parser::ParseCSSTransform(value, font_size) {
                style.paint.transform = Some(transform);
            }
        }
        // Rust input extension: Blink EventHandler consumes these inherited
        // CSS values, independently of the authored resolver's paint subset.
        "cursor" => {
            if let Some(cursor) = crate::style_resolver::cursor::ParseCursor(value) {
                style.paint.cursor = cursor;
            }
        }
        "pointer-events" => match value {
            "none" => style.paint.pointer_events_none = true,
            "auto" | "all" | "visible" | "visiblepainted" | "visiblefill" | "visiblestroke"
            | "painted" | "fill" | "stroke" | "bounding-box" => {
                style.paint.pointer_events_none = false
            }
            _ => {}
        },
        // cpp: style_resolver/style_resolver.cc:6253-6257
        "visibility" => match value {
            "visible" => style.paint.visible = true,
            "hidden" | "collapse" => style.paint.visible = false,
            _ => {}
        },
        // cpp: style_resolver/style_resolver.cc:5601-5609
        "filter" => {
            if value == "none" {
                style.paint.filters.clear();
            } else {
                let mut filters = Vec::new();
                let mut remaining = value.trim();
                let mut valid = true;
                while !remaining.is_empty() {
                    let Some(after_name) = remaining.strip_prefix("blur") else {
                        valid = false;
                        break;
                    };
                    let after_name = after_name.trim_start();
                    let Some(arguments) = after_name.strip_prefix('(') else {
                        valid = false;
                        break;
                    };
                    let Some(close) = arguments.find(')') else {
                        valid = false;
                        break;
                    };
                    let Some(radius) = border_radius::Length(arguments[..close].trim(), font_size)
                        .filter(|radius| *radius >= 0.0)
                    else {
                        valid = false;
                        break;
                    };
                    filters.push(PaintFilterOperation {
                        r#type: PaintFilterType::kBlur,
                        amount: radius,
                        ..Default::default()
                    });
                    remaining = arguments[close + 1..].trim_start();
                }
                if valid && !filters.is_empty() {
                    style.paint.filters = filters;
                }
            }
        }
        "outline" | "outline-width" | "outline-style" | "outline-color" | "outline-offset"
        | "column-rule" | "column-rule-width" | "column-rule-style" | "column-rule-color" => {
            border_longhands::ApplyBorder(style, property, value, font_size)
        }
        "gap" | "row-gap" | "column-gap" => gap::ApplyGap(style, property, value, font_size),
        "-webkit-text-size-adjust" | "-webkit-text-decoration" => {}
        // cpp: style_resolver/style_resolver.cc:5966-5967,6204-6205
        "color" => {
            if let Some(color) = layoutng_assembly::css_color_parser::ParseCSSColor(value) {
                style.paint.color = color;
            }
        }
        // cpp: style_resolver/style_resolver.cc:5998-6053
        "fill" | "stroke" => {
            let resolved = if value.eq_ignore_ascii_case("none") {
                Some((None, false))
            } else if value.eq_ignore_ascii_case("currentcolor") {
                Some((Some(style.paint.color), true))
            } else {
                layoutng_assembly::css_color_parser::ParseCSSColor(value)
                    .map(|parsed| (Some(parsed), false))
            };
            if let Some((paint, current_color)) = resolved {
                if property == "fill" {
                    style.paint.svg_fill = paint;
                    style.paint.svg_fill_current_color = current_color;
                    style.paint.svg_fill_server = None;
                } else {
                    style.paint.svg_stroke = paint;
                    style.paint.svg_stroke_current_color = current_color;
                    style.paint.svg_stroke_server = None;
                }
            }
        }
        "stroke-width" => {
            let parsed =
                border_radius::Length(value, font_size).or_else(|| value.parse::<f64>().ok());
            if let Some(width) = parsed.filter(|width| width.is_finite() && *width >= 0.0) {
                style.paint.svg_stroke_width = width;
            }
        }
        "stroke-linecap" => match value {
            "butt" => style.paint.svg_stroke_line_cap = SvgStrokeLineCap::kButt,
            "round" => style.paint.svg_stroke_line_cap = SvgStrokeLineCap::kRound,
            "square" => style.paint.svg_stroke_line_cap = SvgStrokeLineCap::kSquare,
            _ => {}
        },
        "stroke-linejoin" => match value {
            "miter" => style.paint.svg_stroke_line_join = SvgStrokeLineJoin::kMiter,
            "round" => style.paint.svg_stroke_line_join = SvgStrokeLineJoin::kRound,
            "bevel" => style.paint.svg_stroke_line_join = SvgStrokeLineJoin::kBevel,
            _ => {}
        },
        "fill-rule" => match value {
            "evenodd" => style.paint.svg_fill_even_odd = true,
            "nonzero" => style.paint.svg_fill_even_odd = false,
            _ => {}
        },
        "stroke-miterlimit" => {
            if let Ok(limit) = value.parse::<f64>() {
                if limit.is_finite() && limit >= 1.0 {
                    style.paint.svg_stroke_miter_limit = limit;
                }
            }
        }
        "opacity" => {
            if let Ok(opacity) = value.parse::<f32>() {
                if opacity.is_finite() {
                    style.paint.opacity = opacity.clamp(0.0, 1.0);
                }
            }
        }
        "background-color" => {
            if let Some(color) = layoutng_assembly::css_color_parser::ParseCSSColor(value) {
                style.paint.background_color = color;
            }
        }
        "background" => {
            let mut background = background::BackgroundCascadeState::default();
            background.Import(&style.paint);
            background.Apply(style, property, raw_value);
            background.Export(style);
        }
        "mask-image" | "-webkit-mask-image" => background::ApplyMaskImages(style, raw_value),
        "border-width"
        | "border-style"
        | "border-color"
        | "border-top-width"
        | "border-right-width"
        | "border-bottom-width"
        | "border-left-width"
        | "border-top-style"
        | "border-right-style"
        | "border-bottom-style"
        | "border-left-style"
        | "border-top-color"
        | "border-right-color"
        | "border-bottom-color"
        | "border-left-color" => {
            border_longhands::ApplyBorderLonghand(style, property, value, font_size);
        }
        "border" | "border-top" | "border-right" | "border-bottom" | "border-left" => {
            border_longhands::ApplyBorder(style, property, value, font_size)
        }
        "border-radius"
        | "border-top-left-radius"
        | "border-top-right-radius"
        | "border-bottom-right-radius"
        | "border-bottom-left-radius" => {
            border_radius::ApplyBorderRadius(style, property, value);
        }
        "box-shadow" | "text-shadow" => shadow::ApplyShadow(style, property, value),
        // cpp: style_resolver/style_resolver.cc:6214-6219
        "clip-path" => {
            if normalized == "none" {
                style.paint.clip_path = None;
            } else if let Some(path) = clip_path::ParseClipPath(raw_value, font_size) {
                style.paint.clip_path = Some(path);
            }
        }
        // cpp: style_resolver/style_resolver.cc:5968-5974
        "object-fit" => {
            use layoutng_assembly::internal::layout_input::ObjectFit;
            let fit = match value {
                "fill" => Some(ObjectFit::kFill),
                "contain" => Some(ObjectFit::kContain),
                "cover" => Some(ObjectFit::kCover),
                "none" => Some(ObjectFit::kNone),
                "scale-down" => Some(ObjectFit::kScaleDown),
                _ => None,
            };
            if let Some(fit) = fit {
                style.paint.object_fit = fit;
            }
        }
        // cpp: style_resolver/style_resolver.cc:5975-5979
        "object-position" => {
            if let Some((percentage, offset)) =
                background::ParseBackgroundPositionLayer(raw_value, font_size)
            {
                style.paint.object_position = percentage;
                style.paint.object_position_offset = offset;
            }
        }
        // cpp: style_resolver/style_resolver.cc:6248-6250
        "transform-origin" => {
            if let Some(origin) = transform_origin::ParseTransformOrigin(raw_value, font_size) {
                style.paint.transform_origin = Some(origin);
            }
        }
        // cpp: style_resolver/style_resolver.cc:6229-6247
        "will-change" => {
            style.paint.will_change_transform = false;
            if value != "auto" {
                for item in value.split(',') {
                    let item = item.trim_matches(|c: char| selector::SourceSpace::source_space(c));
                    if matches!(
                        item,
                        "transform"
                            | "translate"
                            | "scale"
                            | "rotate"
                            | "perspective"
                            | "transform-style"
                    ) {
                        style.paint.will_change_transform = true;
                        break;
                    }
                }
            }
        }
        _ if source_supported_properties::IsSourceSupported(property) => {
            panic!("CSS property {property} requires source style_resolver translation")
        }
        _ => {}
    }
}

fn specificity(rule: &Rule, element: &Element) -> Option<u8> {
    rule.selectors
        .iter()
        .filter_map(|selector| match selector {
            Selector::Tag(tag) if tag == &element.tag => Some(1),
            Selector::Class(class) if element.classes.contains(class) => Some(10),
            Selector::Id(id) if element.id.as_ref() == Some(id) => Some(100),
            _ => None,
        })
        .max()
}

#[allow(non_snake_case)]
pub fn Resolve(document: &Document, rules: &[Rule], width: f64, height: f64) -> Vec<ComputedStyle> {
    document
        .elements
        .iter()
        .map(|element| {
            let mut style = ComputedStyle::default();
            // cpp: style_resolver/style_resolver.cc:3880-3905
            if matches!(element.tag.as_str(), "a" | "span") {
                style.display = Display::kInline;
            }
            // cpp: style_resolver/style_resolver.cc:3909
            if element.tag == "body" {
                style.margin = Edges {
                    top: 8.0,
                    right: 8.0,
                    bottom: 8.0,
                    left: 8.0,
                };
            }
            let mut matching: Vec<(u8, usize)> = rules
                .iter()
                .enumerate()
                .filter_map(|(index, rule)| {
                    specificity(rule, element).map(|specificity| (specificity, index))
                })
                .collect();
            matching.sort_unstable();
            for (_, index) in matching {
                for (property, value) in &rules[index].declarations {
                    apply_after_font_prepass(&mut style, property, value, (width, height));
                }
            }
            // cpp: style_resolver/style_resolver.cc:7790-7800
            if let Some(inline_style) = &element.inline_style {
                for (property, value) in css_parser::ParseDeclarations(inline_style) {
                    apply(&mut style, &property, &value, (width, height));
                }
            }
            style
        })
        .collect()
}

fn media_conditions_match(conditions: &[String], width: f64, height: f64) -> bool {
    let environment = StyleEnvironment {
        viewport_width: Some(width),
        viewport_height: Some(height),
        resolution_dppx: Some(1.0),
        ..Default::default()
    };
    conditions
        .iter()
        .all(|condition| media::MediaConditionMatches(condition, &environment))
}

// CSSOM entry for the static browser path. It preserves stylesheet source
// order and selector specificity while the full DOM cascade remains
// an unconnected part of the original style_resolver package.
type CustomProperties = std::sync::Arc<HashMap<String, Option<String>>>;
type AuthorCascade<'a> = Vec<(u8, usize, selector::Specificity, usize, &'a str, &'a str)>;

#[derive(Clone, Default)]
pub struct ResolvedStyles {
    pub styles: Vec<ComputedStyle>,
    pub generates_box: Vec<bool>,
    pub display_contents: Vec<bool>,
    pub before: Vec<Option<PseudoElement>>,
    pub after: Vec<Option<PseudoElement>>,
}

fn static_cascade(
    cascade: &AuthorCascade<'_>,
    width: f64,
    height: f64,
) -> Vec<cascade::CascadedDeclaration> {
    cascade
        .iter()
        .map(
            |(level, layer, specificity, order, property, value)| cascade::CascadedDeclaration {
                declaration: cssom::CSSDeclaration {
                    property: (*property).into(),
                    value: resolve_viewport_relative_lengths(value, width, height),
                    important: *level >= 3,
                },
                origin: if matches!(level, 0 | 4) {
                    cascade::Origin::kUserAgent
                } else {
                    cascade::Origin::kAuthor
                },
                layer_priority: *layer,
                specificity: *specificity,
                source_order: *order,
            },
        )
        .collect()
}
struct CascadeInitialStates {
    initial: ComputedStyle,
    unset: ComputedStyle,
    reverted: ComputedStyle,
    reverted_generates_box: bool,
    reverted_display_contents: bool,
    computed_font_size: f64,
    computed_font_weight: f64,
    computed_zoom: f32,
    computed_effective_zoom: f32,
    logical_writing_mode: layoutng_assembly::internal::layout_input::WritingMode,
    logical_direction: layoutng_assembly::internal::layout_input::TextDirection,
}
impl CascadeInitialStates {
    fn sources<'a>(
        &'a self,
        parent: Option<&'a ComputedStyle>,
        parent_box: bool,
        parent_contents: bool,
    ) -> [css_wide::CSSWideSource<'a>; 4] {
        [
            css_wide::CSSWideSource::new(&self.initial),
            css_wide::CSSWideSource {
                style: parent.unwrap_or(&self.initial),
                generates_box: if parent.is_some() { parent_box } else { true },
                display_contents: parent.is_some() && parent_contents,
            },
            css_wide::CSSWideSource::new(&self.unset),
            css_wide::CSSWideSource {
                style: &self.reverted,
                generates_box: self.reverted_generates_box,
                display_contents: self.reverted_display_contents,
            },
        ]
    }
    fn restore_font(&self, style: &mut ComputedStyle) {
        let e = style.extended.get_or_insert_with(Default::default);
        e.font_size = self.computed_font_size;
        e.font_weight = self.computed_font_weight;
        e.zoom = self.computed_zoom;
        e.effective_zoom = self.computed_effective_zoom;
    }
}
// Shared source font/axis prepasses; declaration application is still staged.
fn prepare_static_cascade(
    document: &Document,
    index: usize,
    parent: Option<&ComputedStyle>,
    parent_box: bool,
    parent_contents: bool,
    style: &mut ComputedStyle,
    own_box: bool,
    contents: bool,
    cascade: &[cascade::CascadedDeclaration],
    properties: &CustomProperties,
) -> CascadeInitialStates {
    prepare_cascade(
        initial_style::StaticInitialStyle(document, index, None, true),
        initial_style::StaticInitialStyle(document, index, parent, true),
        parent,
        parent_box,
        parent_contents,
        style,
        own_box,
        contents,
        cascade,
        properties,
    )
}
// cpp: style_resolver/style_resolver.cc:7438-7491,7811-7862
fn prepare_cascade(
    initial_style: ComputedStyle,
    unset_style: ComputedStyle,
    parent: Option<&ComputedStyle>,
    parent_box: bool,
    parent_contents: bool,
    style: &mut ComputedStyle,
    own_box: bool,
    contents: bool,
    cascade: &[cascade::CascadedDeclaration],
    properties: &CustomProperties,
) -> CascadeInitialStates {
    let mut states = CascadeInitialStates {
        initial: initial_style,
        unset: unset_style,
        reverted: style.clone(),
        reverted_generates_box: own_box,
        reverted_display_contents: contents,
        computed_font_size: 0.0,
        computed_font_weight: 0.0,
        computed_zoom: 1.0,
        computed_effective_zoom: 1.0,
        logical_writing_mode: style.writing_mode,
        logical_direction: style.direction,
    };
    let [initial, inherited, unset, reverted] = states.sources(parent, parent_box, parent_contents);
    let computed_font_size = fonts::ResolveComputedFontSize(
        cascade, properties, &initial, &inherited, &unset, &reverted,
    );
    let computed_font_weight = fonts::ResolveComputedFontWeight(
        cascade, properties, &initial, &inherited, &unset, &reverted,
    );
    let computed_zoom =
        zoom::ResolveZoom(cascade, properties, &initial, &inherited, &unset, &reverted);
    let parent_zoom = parent
        .and_then(|s| s.extended.as_ref())
        .map_or(1.0, |e| e.effective_zoom);
    let (mut writing_mode, mut direction) = (style.writing_mode, style.direction);
    for item in cascade {
        if item.declaration.property.starts_with("--") {
            continue;
        }
        let declaration = cascade::ResolveDeclarationVariables(&item.declaration, properties)
            .unwrap_or_else(|| cssom::CSSDeclaration {
                property: item.declaration.property.clone(),
                value: "unset".into(),
                ..Default::default()
            });
        let value = declaration
            .value
            .trim_matches(|c: char| selector::SourceSpace::source_space(c))
            .to_ascii_lowercase();
        let wide = css_wide::CSSWideKeywordSource(&value, &initial, &inherited, &unset, &reverted);
        if declaration.property == "writing-mode" {
            if let Some(wide) = wide {
                writing_mode = wide.style.writing_mode;
            } else {
                use layoutng_assembly::internal::layout_input::WritingMode::*;
                writing_mode = match value.as_str() {
                    "horizontal-tb" => kHorizontalTb,
                    "vertical-rl" => kVerticalRl,
                    "vertical-lr" => kVerticalLr,
                    _ => writing_mode,
                };
            }
        } else if declaration.property == "direction" {
            if let Some(wide) = wide {
                direction = wide.style.direction;
            } else {
                use layoutng_assembly::internal::layout_input::TextDirection::*;
                direction = match value.as_str() {
                    "ltr" => kLtr,
                    "rtl" => kRtl,
                    _ => direction,
                };
            }
        }
    }
    states.computed_font_size = computed_font_size;
    states.computed_font_weight = computed_font_weight;
    states.computed_zoom = computed_zoom;
    states.computed_effective_zoom = (parent_zoom * computed_zoom).clamp(1e-6, 1e6);
    states.logical_writing_mode = writing_mode;
    states.logical_direction = direction;
    states.restore_font(style);
    states
}
fn apply_after_font_prepass(
    style: &mut ComputedStyle,
    property: &str,
    value: &str,
    viewport: (f64, f64),
) {
    let font_size = fonts::FontSizeOf(style);
    match property {
        "font" => {
            if let Some(parsed) = fonts::ParseFontShorthand(value, font_size) {
                let e = style.extended.get_or_insert_with(Default::default);
                e.font_italic = parsed.italic;
                e.font_families = parsed.families;
                fonts::ApplyLineHeightValue(
                    style,
                    parsed.line_height.as_deref().unwrap_or("normal"),
                    font_size,
                );
            }
        }
        "font-family" => {
            if let Some(families) = fonts::ParseFontFamilies(value) {
                style
                    .extended
                    .get_or_insert_with(Default::default)
                    .font_families = families;
            }
        }
        "line-height" => fonts::ApplyLineHeightValue(style, value, font_size),
        "direction" => {
            use layoutng_assembly::internal::layout_input::TextDirection::*;
            match value
                .trim_matches(|c: char| selector::SourceSpace::source_space(c))
                .to_ascii_lowercase()
                .as_str()
            {
                "ltr" => style.direction = kLtr,
                "rtl" => style.direction = kRtl,
                _ => {}
            }
        }
        "writing-mode" => {
            use layoutng_assembly::internal::layout_input::WritingMode::*;
            match value
                .trim_matches(|c: char| selector::SourceSpace::source_space(c))
                .to_ascii_lowercase()
                .as_str()
            {
                "horizontal-tb" => style.writing_mode = kHorizontalTb,
                "vertical-rl" => style.writing_mode = kVerticalRl,
                "vertical-lr" => style.writing_mode = kVerticalLr,
                _ => {}
            }
        }
        _ => apply(style, property, value, viewport),
    }
}

fn apply_with_axes(
    style: &mut ComputedStyle,
    property: &str,
    value: &str,
    viewport: (f64, f64),
    mode: layoutng_assembly::internal::layout_input::WritingMode,
    direction: layoutng_assembly::internal::layout_input::TextDirection,
) {
    if let Some(expansion) =
        box_properties::ExpandLogicalPair(style, property, value, mode, direction)
    {
        for (property, value) in expansion {
            apply_after_font_prepass(style, &property, &value, viewport);
        }
    } else {
        let property = css_wide::MapLogicalProperty(property, mode, direction);
        apply_after_font_prepass(style, &property, value, viewport);
    }
}

fn cascade_rank(important: bool, user_agent: bool) -> u8 {
    cascade::CascadeLevel(
        important,
        if user_agent {
            cascade::Origin::kUserAgent
        } else {
            cascade::Origin::kAuthor
        },
    )
}

// cpp: style_resolver/style_resolver.cc:7265-7368
fn parse_generated_content(input: &str, element: &Element) -> Option<(bool, String)> {
    generated_content::ParseWithAttributes(input, element.namespace, |name| {
        element
            .attributes
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.clone())
    })
}

// cpp: style_resolver/style_resolver.cc:7423-7542
fn resolve_pseudo_element(
    document: &Document,
    element_index: usize,
    originating_style: &ComputedStyle,
    inherited_custom: &CustomProperties,
    originating_generates_box: bool,
    originating_display_contents: bool,
    cascade: &mut AuthorCascade<'_>,
    width: f64,
    height: f64,
) -> Option<PseudoElement> {
    let element = &document.elements[element_index];
    if cascade.is_empty() {
        return None;
    }
    cascade.sort_by_key(|(origin, layer, specificity, order, _, _)| {
        (*origin, *layer, *specificity, *order)
    });
    let typed_cascade = static_cascade(cascade, width, height);
    let custom_properties =
        cascade::ResolveCustomProperties(&typed_cascade, Some(inherited_custom));
    let mut style =
        initial_style::StaticInitialStyle(document, element_index, Some(originating_style), true);
    let mut background = background::BackgroundCascadeState::default();
    let mut generates_box = true;
    let mut display_contents = false;
    let states = prepare_static_cascade(
        document,
        element_index,
        Some(originating_style),
        originating_generates_box,
        originating_display_contents,
        &mut style,
        generates_box,
        display_contents,
        &typed_cascade,
        &custom_properties,
    );
    let [initial, inherited, unset, reverted] = states.sources(
        Some(originating_style),
        originating_generates_box,
        originating_display_contents,
    );
    let mut content = (false, String::new());
    for item in &typed_cascade {
        if item.declaration.property.starts_with("--") {
            continue;
        }
        let declaration =
            cascade::ResolveDeclarationVariables(&item.declaration, &custom_properties)
                .unwrap_or_else(|| cssom::CSSDeclaration {
                    property: item.declaration.property.clone(),
                    value: "unset".into(),
                    ..Default::default()
                });
        let property = declaration.property.as_str();
        let value = declaration
            .value
            .trim_matches(|c: char| selector::SourceSpace::source_space(c));
        if property == "initial-letter" {
            continue;
        }
        if property == "content" {
            if css_wide::CSSWideKeywordSource(value, &initial, &inherited, &unset, &reverted)
                .is_some()
            {
                content = (false, String::new());
            } else if let Some(parsed) = parse_generated_content(value, element) {
                content = parsed;
            }
            continue;
        }
        if matches!(property, "font-size" | "font-weight") {
            continue;
        }
        if property == "all"
            && css_wide::CSSWideKeywordSource(value, &initial, &inherited, &unset, &reverted)
                .is_some()
        {
            content = (false, String::new());
        }
        if css_wide::ApplyCSSWideKeyword(
            &declaration,
            states.logical_writing_mode,
            states.logical_direction,
            &initial,
            &inherited,
            &unset,
            &reverted,
            &mut style,
            &mut generates_box,
            &mut display_contents,
            &mut background,
        ) {
            states.restore_font(&mut style);
            continue;
        }
        if property == "display" {
            box_properties::ApplyDisplay(
                &mut style,
                &mut generates_box,
                &mut display_contents,
                value,
            );
            continue;
        }
        if background.Apply(&mut style, property, value) {
            continue;
        }
        apply_with_axes(
            &mut style,
            property,
            value,
            (width, height),
            states.logical_writing_mode,
            states.logical_direction,
        );
    }
    finish_style::NormalizeOverflow(&mut style);
    finish_style::ResolveSVGCurrentColor(&mut style);
    background.Export(&mut style);
    if !content.0 || (!generates_box && !display_contents) {
        return None;
    }
    Some(PseudoElement {
        style,
        text: content.1,
        display_contents,
    })
}

#[allow(non_snake_case)]
pub fn ResolveCssom(
    document: &Document,
    sheets: &[cssom::CSSStyleSheet],
    width: f64,
    height: f64,
) -> ResolvedStyles {
    ResolveCssomWithUserAgent(document, &[], sheets, width, height)
}

// cpp: browser/browser.cc:1751-1758
// cpp: style_resolver/style_resolver.cc:7790-7810
#[allow(non_snake_case)]
pub fn ResolveCssomWithUserAgent(
    document: &Document,
    user_agent_sheets: &[cssom::CSSStyleSheet],
    author_sheets: &[cssom::CSSStyleSheet],
    width: f64,
    height: f64,
) -> ResolvedStyles {
    let layers = cascade::CascadeLayerOrder::new(user_agent_sheets, author_sheets);
    let indexes = style_rule_index::ActiveStyleRuleIndexes::new(user_agent_sheets, author_sheets);
    let mut resolved: Vec<(ComputedStyle, bool, bool)> =
        Vec::with_capacity(document.elements.len());
    let mut before = Vec::with_capacity(document.elements.len());
    let mut after = Vec::with_capacity(document.elements.len());
    let mut inherited_custom: Vec<CustomProperties> = Vec::with_capacity(document.elements.len());
    for (element_index, element) in document.elements.iter().enumerate() {
        let parent = element
            .parent
            .and_then(|index| resolved.get(index).map(|entry| &entry.0));
        let mut style = initial_style::StaticInitialStyle(document, element_index, parent, false);
        let mut background = background::BackgroundCascadeState::default();
        let mut generates_box = !(element.namespace == DOMNamespace::kSVG
            && matches!(
                element.tag.as_str(),
                "defs"
                    | "linearGradient"
                    | "radialGradient"
                    | "stop"
                    | "pattern"
                    | "clipPath"
                    | "mask"
                    | "marker"
                    | "symbol"
            ));
        let mut display_contents = false;
        let mut cascade: AuthorCascade<'_> = Vec::new();
        let mut before_cascade: AuthorCascade<'_> = Vec::new();
        let mut after_cascade: AuthorCascade<'_> = Vec::new();
        let mut presentation: Vec<(String, String)> = Vec::new();
        let mut order = 0usize;
        // cpp: style_resolver/style_resolver.cc:7582-7619
        if element.namespace == DOMNamespace::kSVG {
            for property in [
                "color",
                "fill",
                "stroke",
                "stroke-width",
                "opacity",
                "visibility",
                "font-size",
                "font-family",
                "transform",
                "stroke-dasharray",
                "stroke-dashoffset",
                "stroke-linecap",
                "stroke-linejoin",
                "stroke-miterlimit",
                "fill-rule",
                "vector-effect",
                "shape-rendering",
                "paint-order",
            ] {
                if let Some((_, value)) =
                    element.attributes.iter().find(|(name, _)| name == property)
                {
                    let adjusted = if property == "font-size" && value.parse::<f64>().is_ok() {
                        format!("{value}px")
                    } else {
                        value.clone()
                    };
                    // Store the owned adjusted value outside this borrowed cascade below.
                    // The common declaration list retains the same zero specificity.
                    presentation.push((property.to_owned(), adjusted));
                }
            }
            if matches!(element.tag.as_str(), "svg" | "foreignObject") {
                for property in ["width", "height"] {
                    if let Some((_, value)) =
                        element.attributes.iter().find(|(name, _)| name == property)
                    {
                        let adjusted = if value.parse::<f64>().is_ok() {
                            format!("{value}px")
                        } else {
                            value.trim().to_owned()
                        };
                        presentation.push((property.to_owned(), adjusted));
                    }
                }
            }
        }
        for (property, value) in &presentation {
            cascade.push((
                cascade_rank(false, false),
                layers.LayerPriority(cascade::Origin::kAuthor, "", false, false),
                selector::Specificity::default(),
                order,
                property,
                value,
            ));
            order += 1;
        }
        for (origin, index) in [
            (cascade::Origin::kUserAgent, &indexes.user_agent),
            (cascade::Origin::kAuthor, &indexes.author),
        ] {
            for indexed in style_rule_index::StaticCandidateRules(index, element) {
                let rule = indexed.rule;
                if !media_conditions_match(&rule.media_conditions, width, height) {
                    continue;
                }
                if let Some(specificity) =
                    selector::MatchSelector(document, element_index, &rule.selector_text)
                {
                    for (declaration_index, declaration) in rule.declarations.iter().enumerate() {
                        cascade.push((
                            cascade::CascadeLevel(declaration.important, origin),
                            layers.LayerPriority(
                                origin,
                                &rule.layer_name,
                                declaration.important,
                                false,
                            ),
                            specificity,
                            order + indexed.source_order + declaration_index,
                            declaration.property.as_str(),
                            declaration.value.as_str(),
                        ));
                    }
                }
                for (target, pseudo_cascade) in [
                    (selector::PseudoTarget::Before, &mut before_cascade),
                    (selector::PseudoTarget::After, &mut after_cascade),
                ] {
                    if let Some(specificity) = selector::MatchSelectorTarget(
                        document,
                        element_index,
                        &rule.selector_text,
                        target,
                    ) {
                        for (declaration_index, declaration) in rule.declarations.iter().enumerate()
                        {
                            pseudo_cascade.push((
                                cascade::CascadeLevel(declaration.important, origin),
                                layers.LayerPriority(
                                    origin,
                                    &rule.layer_name,
                                    declaration.important,
                                    false,
                                ),
                                specificity,
                                order + indexed.source_order + declaration_index,
                                declaration.property.as_str(),
                                declaration.value.as_str(),
                            ));
                        }
                    }
                }
            }
            order += index.declaration_count;
        }
        let inline_declarations = element
            .inline_style
            .as_deref()
            .map(cssom::ParseCSSDeclarationList)
            .unwrap_or_default();
        for declaration in &inline_declarations {
            cascade.push((
                cascade_rank(declaration.important, false),
                layers.LayerPriority(cascade::Origin::kAuthor, "", declaration.important, true),
                selector::Specificity {
                    ids: 1 << 20,
                    ..Default::default()
                },
                order,
                declaration.property.as_str(),
                declaration.value.as_str(),
            ));
            order += 1;
        }
        cascade.sort_by_key(|(origin, layer, specificity, order, _, _)| {
            (*origin, *layer, *specificity, *order)
        });
        // cpp: style_resolver/style_resolver.cc:6997-7022,7809-7810
        let typed_cascade = static_cascade(&cascade, width, height);
        let custom_properties = cascade::ResolveCustomProperties(
            &typed_cascade,
            element.parent.and_then(|index| inherited_custom.get(index)),
        );
        let parent_box = element
            .parent
            .and_then(|index| resolved.get(index))
            .is_none_or(|entry| entry.1);
        let parent_contents = element
            .parent
            .and_then(|index| resolved.get(index))
            .is_some_and(|entry| entry.2);
        let states = prepare_static_cascade(
            document,
            element_index,
            parent,
            parent_box,
            parent_contents,
            &mut style,
            generates_box,
            display_contents,
            &typed_cascade,
            &custom_properties,
        );
        let [initial, inherited, unset, reverted] =
            states.sources(parent, parent_box, parent_contents);
        for item in &typed_cascade {
            if item.declaration.property.starts_with("--") {
                continue;
            }
            let resolved_declaration =
                cascade::ResolveDeclarationVariables(&item.declaration, &custom_properties)
                    .unwrap_or_else(|| cssom::CSSDeclaration {
                        property: item.declaration.property.clone(),
                        value: "unset".into(),
                        ..Default::default()
                    });
            let property = resolved_declaration.property.as_str();
            let value = resolved_declaration
                .value
                .trim_matches(|c: char| selector::SourceSpace::source_space(c));
            if property == "font-size" {
                let flag = if let Some(source) =
                    css_wide::CSSWideKeywordSource(value, &initial, &inherited, &unset, &reverted)
                {
                    Some(
                        source
                            .style
                            .extended
                            .as_ref()
                            .is_some_and(|e| e.font_size_math),
                    )
                } else if fonts::ParseComputedFontSize(value, fonts::FontSizeOf(inherited.style))
                    .is_some()
                {
                    Some(false)
                } else {
                    None
                };
                if let Some(flag) = flag {
                    style
                        .extended
                        .get_or_insert_with(Default::default)
                        .font_size_math = flag;
                }
                continue;
            }
            if property == "font-weight" {
                continue;
            }
            if property == "initial-letter" {
                continue;
            }
            let declaration = cssom::CSSDeclaration {
                property: property.into(),
                value: value.into(),
                ..Default::default()
            };
            if css_wide::ApplyCSSWideKeyword(
                &declaration,
                states.logical_writing_mode,
                states.logical_direction,
                &initial,
                &inherited,
                &unset,
                &reverted,
                &mut style,
                &mut generates_box,
                &mut display_contents,
                &mut background,
            ) {
                states.restore_font(&mut style);
                continue;
            }
            // Only author declarations suppress native control decoration.
            let extra = style.extended.get_or_insert_with(Default::default);
            let is_author = item.origin == cascade::Origin::kAuthor;
            if is_author && (property == "background" || property.starts_with("background-")) {
                extra.has_author_background = true;
            }
            if is_author
                && (property == "border-radius"
                    || (property.starts_with("border-") && property.ends_with("-radius")))
            {
                extra.has_author_border_radius = true;
            } else if is_author
                && (property == "border"
                    || (property.starts_with("border-")
                        && property != "border-collapse"
                        && property != "border-spacing"))
            {
                extra.has_author_border = true;
            }
            if is_author && (property == "outline" || property.starts_with("outline-")) {
                extra.has_author_outline = true;
            }
            if property == "display" {
                box_properties::ApplyDisplay(
                    &mut style,
                    &mut generates_box,
                    &mut display_contents,
                    value,
                );
                continue;
            }
            if background.Apply(&mut style, property, value) {
                continue;
            }
            apply_with_axes(
                &mut style,
                property,
                value,
                (width, height),
                states.logical_writing_mode,
                states.logical_direction,
            );
        }
        finish_style::NormalizeOverflow(&mut style);
        finish_style::ResolveSVGCurrentColor(&mut style);
        background.Export(&mut style);
        before.push(if generates_box || display_contents {
            resolve_pseudo_element(
                document,
                element_index,
                &style,
                &custom_properties,
                generates_box,
                display_contents,
                &mut before_cascade,
                width,
                height,
            )
        } else {
            None
        });
        after.push(if generates_box || display_contents {
            resolve_pseudo_element(
                document,
                element_index,
                &style,
                &custom_properties,
                generates_box,
                display_contents,
                &mut after_cascade,
                width,
                height,
            )
        } else {
            None
        });
        // cpp: style_resolver/style_resolver.cc:3038-3044
        for layer in &mut style.paint.background_images {
            if !layer.source_url.is_empty() {
                if let Some(resource) = document.ImageResourceFor(&layer.source_url) {
                    layer.resource_id = resource.id;
                }
            }
        }
        resolved.push((style, generates_box, display_contents));
        inherited_custom.push(custom_properties);
    }
    let mut styles = ResolvedStyles {
        styles: Vec::with_capacity(resolved.len()),
        generates_box: Vec::with_capacity(resolved.len()),
        display_contents: Vec::with_capacity(resolved.len()),
        before,
        after,
    };
    for (style, generates_box, display_contents) in resolved {
        styles.styles.push(style);
        styles.generates_box.push(generates_box);
        styles.display_contents.push(display_contents);
    }
    styles
}

#[cfg(test)]
mod user_agent_cascade_tests {
    use super::*;

    #[test]
    fn author_origin_beats_more_specific_user_agent_selector() {
        let mut document = Document::default();
        let button = document.append(None, "button".into(), Some("submit".into()), vec![], None);
        let user_agent = cssom::ParseCSS(
            "#submit { background-color: red; -internal-align-content-block: center; }",
        );
        let author = cssom::ParseCSS("button { background-color: blue; }");
        let styles = ResolveCssomWithUserAgent(&document, &[user_agent], &[author], 200.0, 100.0);
        let style = &styles.styles[button];
        assert_eq!(
            style.paint.background_color,
            layoutng_assembly::css_color_parser::ParseCSSColor("blue").unwrap()
        );
        assert!(style.extended.as_ref().unwrap().align_content_block_center);
        assert!(style.extended.as_ref().unwrap().has_author_background);
    }

    #[test]
    fn layers_reverse_for_important_and_inline_beats_author_layers() {
        let markup = "<html><body><div id='normal'></div><div id='important'></div><div id='inline' style='color: black !important'></div><div id='ua'></div><div id='pseudo'></div></body></html>";
        let document = crate::test_html::Parse(markup);
        let author = cssom::ParseCSS("@layer first, second; @layer first { #normal { color: red; } #important, #inline, #ua { color: red !important; } #pseudo::before { content: 'first'; color: red !important; } } @layer second { #normal { color: blue; } #important, #inline, #ua { color: blue !important; } #pseudo::before { content: 'second'; color: blue !important; } } #normal { color: green; } #important, #inline, #ua { color: green !important; } #pseudo::before { content: 'unlayered'; color: green !important; }");
        let ua = cssom::ParseCSS("#ua { color: yellow !important; }");
        let styles = ResolveCssomWithUserAgent(&document, &[ua], &[author], 1024.0, 768.0);
        for (id, expected) in [
            ("normal", "green"),
            ("important", "red"),
            ("inline", "black"),
            ("ua", "yellow"),
        ] {
            let index = document
                .elements
                .iter()
                .position(|e| e.id.as_deref() == Some(id))
                .unwrap();
            assert_eq!(
                styles.styles[index].paint.color,
                layoutng_assembly::css_color_parser::ParseCSSColor(expected).unwrap(),
                "{id}"
            );
        }
        let pseudo = document
            .elements
            .iter()
            .position(|e| e.id.as_deref() == Some("pseudo"))
            .unwrap();
        let before = styles.before[pseudo].as_ref().unwrap();
        assert_eq!(before.text, "unlayered");
        assert_eq!(
            before.style.paint.color,
            layoutng_assembly::css_color_parser::ParseCSSColor("red").unwrap()
        );
    }
}

#[cfg(test)]
mod svg_presentation_tests {
    use super::*;

    #[test]
    fn attributes_cascade_below_rules_and_current_color_uses_final_color() {
        let mut document = Document::default();
        let html = document.append(None, "html".into(), None, vec![], None);
        let svg = document.append_with_namespace(
            Some(html),
            "svg".into(),
            DOMNamespace::kSVG,
            None,
            vec![],
            None,
            vec![
                ("width".into(), "20".into()),
                ("fill".into(), "none".into()),
            ],
        );
        let path = document.append_with_namespace(
            Some(svg),
            "path".into(),
            DOMNamespace::kSVG,
            None,
            vec![],
            None,
            vec![
                ("stroke".into(), "currentColor".into()),
                ("stroke-width".into(), "1.5".into()),
            ],
        );
        let styles = ResolveCssom(
            &document,
            &[cssom::ParseCSS(
                "svg { fill: #123456; } path { color: #abcdef; }",
            )],
            80.0,
            50.0,
        );
        assert_eq!(styles.styles[svg].width, Some(20.0));
        assert_eq!(
            styles.styles[svg].paint.svg_fill,
            layoutng_assembly::css_color_parser::ParseCSSColor("#123456")
        );
        assert_eq!(
            styles.styles[path].paint.svg_stroke,
            layoutng_assembly::css_color_parser::ParseCSSColor("#abcdef")
        );
        assert_eq!(styles.styles[path].paint.svg_stroke_width, 1.5);
    }
}

#[cfg(test)]
mod column_tests {
    use super::*;

    #[test]
    fn columns_shorthand_and_longhands_preserve_source_state() {
        let mut style = ComputedStyle::default();
        apply(&mut style, "columns", "12em 3", (800.0, 600.0));
        assert_eq!(style.column_count, 3);
        let extra = style.extended.as_ref().unwrap();
        assert!(extra.explicit_column_count);
        assert_eq!(extra.column_width, Some(192.0));
        apply(&mut style, "columns", "0 4", (800.0, 600.0));
        assert_eq!(style.column_count, 3);
        apply(&mut style, "column-count", "auto", (800.0, 600.0));
        assert_eq!(style.column_count, 1);
        assert!(!style.extended.as_ref().unwrap().explicit_column_count);
        apply(&mut style, "column-width", "auto", (800.0, 600.0));
        assert_eq!(style.extended.as_ref().unwrap().column_width, None);
    }
}

#[cfg(test)]
mod mask_tests {
    use super::*;

    #[test]
    fn google_mask_declarations_apply_and_invalid_values_preserve_prior_layers() {
        use layoutng_assembly::internal::paint_input::{
            BackgroundBox, PaintMaskComposite, PaintMaskMode,
        };
        let mut style = ComputedStyle::default();
        apply(
            &mut style,
            "-webkit-mask-image",
            "linear-gradient(to right,#000 90%,transparent 100%), url(\"CaseSensitive.PNG\")",
            (1024.0, 768.0),
        );
        assert_eq!(style.paint.mask_images.len(), 2);
        let layer = &style.paint.mask_images[0];
        assert_eq!(layer.image.origin, BackgroundBox::kBorderBox);
        assert_eq!(layer.image.clip, BackgroundBox::kBorderBox);
        assert_eq!(layer.mode, PaintMaskMode::kAlpha);
        assert_eq!(layer.composite, PaintMaskComposite::kAdd);
        let shader = layer.image.shader.as_ref().unwrap();
        assert_eq!(shader.stops[0].offset, 0.9);
        assert_eq!(shader.stops[1].color.alpha, 0.0);
        assert_eq!(
            style.paint.mask_images[1].image.source_url,
            "CaseSensitive.PNG"
        );
        apply(
            &mut style,
            "mask-image",
            "linear-gradient(red, invalid-color)",
            (1024.0, 768.0),
        );
        assert_eq!(style.paint.mask_images.len(), 2);
        apply(&mut style, "mask-image", "none", (1024.0, 768.0));
        assert!(style.paint.mask_images.is_empty());
    }
}
