use crate::persistent_document::{DOMNamespace, PseudoElement};
use crate::svg_path_parser::ParseSVGPathDefault;
use crate::{Child, Element, ParsedDocument as Document, UserInteractionState};
use font_engine::text::native::character_break_iterator::LengthOfGraphemeCluster;
use font_engine::text::native::unicode_category::Category;
use foundation::{unicode, StringView};
use layoutng_assembly::internal::form_control_types::FormControlType;
use layoutng_assembly::internal::layout_input::{
    ComputedStyle, Display, DocumentRole, ElementData, ExtendedStyle, FrameDimension,
    FrameDimensionType, ListStyleType, MathTokenKind, NodeKind, Offset, PaintPathCommand,
    PaintPathVerb, SvgLengthAdjust, SvgShapeData, SvgShapeGeometry, SvgTextPathData,
    SvgViewBoxData, TextAreaSizing, TextFieldSizing, WhiteSpace,
};
use layoutng_assembly::internal::layout_object::LayoutObject;
use layoutng_assembly::layout_engine::LayoutTreeUpdate;
use layoutng_style::style::appearance::AppearanceValue;

unsafe extern "C" {
    fn strtod(begin: *const std::ffi::c_char, end: *mut *mut std::ffi::c_char) -> f64;
    pub(super) fn strtol(
        begin: *const std::ffi::c_char,
        end: *mut *mut std::ffi::c_char,
        base: std::ffi::c_int,
    ) -> std::ffi::c_long;
    fn strtoul(
        begin: *const std::ffi::c_char,
        end: *mut *mut std::ffi::c_char,
        base: std::ffi::c_int,
    ) -> std::ffi::c_ulong;
}

pub(super) trait LayoutNodeView {
    fn node_name(&self) -> &str;
    fn node_attribute(&self, name: &str) -> Option<&str>;
}
impl LayoutNodeView for Element {
    fn node_name(&self) -> &str {
        &self.tag
    }
    fn node_attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }
}
impl LayoutNodeView for crate::persistent_document::DOMNode {
    fn node_name(&self) -> &str {
        self.Name()
    }
    fn node_attribute(&self, name: &str) -> Option<&str> {
        self.FindAttribute(name).map(|a| a.value.as_str())
    }
}
pub(super) fn attribute<'a>(element: &'a impl LayoutNodeView, name: &str) -> Option<&'a str> {
    element.node_attribute(name)
}

pub(super) fn c_string_bytes(input: &str) -> Vec<u8> {
    let mut bytes = input.as_bytes().to_vec();
    bytes.push(0);
    bytes
}

pub(super) fn parse_c_double_prefix(input: &str) -> Option<(f64, usize)> {
    let bytes = c_string_bytes(input);
    let begin = bytes.as_ptr().cast::<std::ffi::c_char>();
    let mut end = std::ptr::null_mut();
    let value = unsafe { strtod(begin, &mut end) };
    let consumed = unsafe { end.offset_from(begin) } as usize;
    (consumed != 0).then_some((value, consumed))
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:26-30
pub(super) fn lower_ascii(value: &str) -> String {
    value.to_ascii_lowercase()
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:33-41
pub(super) fn attribute_number(element: &impl LayoutNodeView, name: &str, fallback: f64) -> f64 {
    let Some(value) = attribute(element, name) else {
        return fallback;
    };
    parse_c_double_prefix(value).map_or(fallback, |(number, _)| number)
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:43-63
pub(super) fn svg_number_list(element: &impl LayoutNodeView, name: &str) -> Vec<f64> {
    let Some(input) = attribute(element, name) else {
        return Vec::new();
    };
    let mut cursor = 0;
    let mut values = Vec::new();
    while cursor < input.len() {
        while cursor < input.len()
            && (input.as_bytes()[cursor].is_ascii_whitespace() || input.as_bytes()[cursor] == b',')
        {
            cursor += 1;
        }
        if cursor == input.len() {
            break;
        }
        let Some((value, consumed)) = parse_c_double_prefix(&input[cursor..]) else {
            return Vec::new();
        };
        if !value.is_finite() {
            return Vec::new();
        }
        values.push(value);
        cursor += consumed;
    }
    values
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:780-806
pub(super) fn list_marker_style(item: &ComputedStyle) -> ComputedStyle {
    let mut marker = ComputedStyle::default();
    marker.display = Display::kInlineBlock;
    marker.direction = item.direction;
    marker.writing_mode = item.writing_mode;
    marker.paint.color = item.paint.color;
    marker.paint.visible = item.paint.visible;
    if let Some(extra) = &item.extended {
        let mut inherited = ExtendedStyle::default();
        inherited.font_size = extra.font_size;
        inherited.line_height = extra.line_height;
        inherited.line_height_percent = extra.line_height_percent;
        inherited.font_families = extra.font_families.clone();
        inherited.font_weight = extra.font_weight;
        inherited.font_italic = extra.font_italic;
        inherited.letter_spacing = extra.letter_spacing;
        inherited.word_spacing = extra.word_spacing;
        inherited.white_space = WhiteSpace::kPre;
        inherited.language = extra.language.clone();
        marker.extended = Some(inherited);
    }
    marker
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:63-75
pub(super) fn attribute_integer(element: &impl LayoutNodeView, name: &str) -> Option<i32> {
    let value = attribute(element, name)?;
    let bytes = c_string_bytes(value);
    let begin = bytes.as_ptr().cast::<std::ffi::c_char>();
    let mut end = std::ptr::null_mut();
    let number = unsafe { strtol(begin, &mut end, 10) };
    let consumed = unsafe { end.offset_from(begin) } as usize;
    (consumed != 0 && bytes[consumed] == 0 && i32::try_from(number).is_ok())
        .then_some(number as i32)
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:78-89
pub(super) fn positive_attribute_integer(
    element: &impl LayoutNodeView,
    name: &str,
    fallback: u32,
) -> u32 {
    let Some(value) = attribute(element, name) else {
        return fallback;
    };
    if value.is_empty() {
        return fallback;
    }
    let bytes = c_string_bytes(value);
    let begin = bytes.as_ptr().cast::<std::ffi::c_char>();
    let mut end = std::ptr::null_mut();
    let number = unsafe { strtoul(begin, &mut end, 10) };
    let consumed = unsafe { end.offset_from(begin) } as usize;
    if consumed == 0 || bytes[consumed] != 0 || number == 0 || number > 0x7fff_ffff {
        return fallback;
    }
    number as u32
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:91-100
pub(super) fn finite_attribute_number(element: &impl LayoutNodeView, name: &str) -> Option<f64> {
    let value = attribute(element, name)?;
    if value.is_empty() {
        return None;
    }
    let (number, _) = parse_c_double_prefix(value)?;
    number.is_finite().then_some(number)
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:102-135
pub(super) fn frame_dimensions(element: &impl LayoutNodeView, name: &str) -> Vec<FrameDimension> {
    let Some(input) = attribute(element, name) else {
        return Vec::new();
    };
    if input.is_empty() {
        return Vec::new();
    }
    let input = input.strip_suffix(',').unwrap_or(input);
    let mut dimensions = Vec::new();
    for token in input.split(',') {
        let token =
            token.trim_matches(|character| matches!(character, ' ' | '\t' | '\r' | '\n' | '\x0c'));
        if token.is_empty() {
            dimensions.push(FrameDimension::default());
            continue;
        }
        let r#type = if token.ends_with('*') {
            FrameDimensionType::kRelative
        } else if token.ends_with('%') {
            FrameDimensionType::kPercentage
        } else {
            FrameDimensionType::kAbsolute
        };
        let parsed = parse_c_double_prefix(token);
        let (value, parsed_type) = match parsed {
            Some((value, _)) if value.is_finite() && value >= 0.0 => (value, r#type),
            Some(_) => (0.0, r#type),
            None => (0.0, FrameDimensionType::kRelative),
        };
        dimensions.push(FrameDimension {
            value,
            r#type: parsed_type,
        });
    }
    dimensions
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:137-144
pub(super) fn frame_border(element: &impl LayoutNodeView) -> Option<bool> {
    let value = lower_ascii(attribute(element, "frameborder")?);
    match value.as_str() {
        "no" | "0" => Some(false),
        "yes" | "1" => Some(true),
        _ => None,
    }
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:146-180
// Document::ControlValue is source-owned by dom and not yet translated; pass
// its result explicitly until that owner exists in Rust.
pub(super) fn range_value_ratio_from_control_value(
    node: &impl LayoutNodeView,
    live_value: &str,
) -> f64 {
    let minimum_attribute = finite_attribute_number(node, "min");
    let value_attribute = finite_attribute_number(node, "value");
    let minimum = minimum_attribute.unwrap_or(0.0);
    let maximum = minimum.max(finite_attribute_number(node, "max").unwrap_or(100.0));
    if maximum == minimum {
        return 0.0;
    }

    let mut value = value_attribute.unwrap_or(minimum + (maximum - minimum) / 2.0);
    if !live_value.is_empty() {
        if let Some((parsed, consumed)) = parse_c_double_prefix(live_value) {
            if consumed == live_value.len() && parsed.is_finite() {
                value = parsed;
            }
        }
    }
    value = value.clamp(minimum, maximum);

    let any_step = attribute(node, "step").is_some_and(|step| lower_ascii(step) == "any");
    let step = finite_attribute_number(node, "step").unwrap_or(1.0);
    if !any_step && step > 0.0 {
        let step_base = minimum_attribute.unwrap_or(value_attribute.unwrap_or(0.0));
        let rounded = step_base + ((value - step_base) / step).round() * step;
        let adjusted = if rounded > maximum {
            rounded - step
        } else if rounded < minimum {
            rounded + step
        } else {
            rounded
        };
        if adjusted >= minimum && adjusted <= maximum {
            value = adjusted;
        }
    }
    ((value - minimum) / (maximum - minimum)).clamp(0.0, 1.0)
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:182-193
fn find_element_by_id(document: &Document, index: usize, id: &str) -> Option<usize> {
    let node = &document.elements[index];
    if attribute(node, "id") == Some(id) {
        return Some(index);
    }
    for child in &node.children {
        if let Child::Element(child_index) = child {
            if let Some(result) = find_element_by_id(document, *child_index, id) {
                return Some(result);
            }
        }
    }
    None
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:195-224
fn linear_svg_shape(mut points: Vec<Offset>, close: bool) -> Option<SvgShapeData> {
    if points.is_empty() {
        return None;
    }
    if close && points.len() > 1 && points.last() == points.first() {
        points.pop();
    }
    let mut shape = SvgShapeData {
        geometry: SvgShapeGeometry::kPath,
        ..SvgShapeData::default()
    };
    shape.path.reserve(points.len() + usize::from(close));
    shape.path.push(PaintPathCommand {
        verb: PaintPathVerb::kMoveTo,
        point: points[0],
        ..PaintPathCommand::default()
    });
    for point in points.iter().skip(1) {
        shape.path.push(PaintPathCommand {
            verb: PaintPathVerb::kLineTo,
            point: *point,
            ..PaintPathCommand::default()
        });
    }
    if close {
        shape.path.push(PaintPathCommand {
            verb: PaintPathVerb::kClose,
            ..PaintPathCommand::default()
        });
    }
    let mut left = points[0].x;
    let mut right = points[0].x;
    let mut top = points[0].y;
    let mut bottom = points[0].y;
    for point in points {
        left = if point.x < left { point.x } else { left };
        right = if right < point.x { point.x } else { right };
        top = if point.y < top { point.y } else { top };
        bottom = if bottom < point.y { point.y } else { bottom };
    }
    shape.bounds_offset = Offset { x: left, y: top };
    shape.bounds_width = right - left;
    shape.bounds_height = bottom - top;
    Some(shape)
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:302-308
fn append_dom_text(document: &Document, index: usize, output: &mut String) {
    for child in &document.elements[index].children {
        match child {
            Child::Text(text_index) => output.push_str(&document.texts[*text_index]),
            Child::Element(child_index) => append_dom_text(document, *child_index, output),
        }
    }
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:310-314
fn collect_options(document: &Document, index: usize, options: &mut Vec<usize>) {
    if document.elements[index].tag == "option" {
        options.push(index);
    }
    for child in &document.elements[index].children {
        if let Child::Element(child_index) = child {
            collect_options(document, *child_index, options);
        }
    }
}

fn trim_collapsible_whitespace(text: &str) -> &str {
    text.trim_matches(|character| matches!(character, ' ' | '\t' | '\r' | '\n' | '\x0c'))
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:316-356
// The live value is supplied by Document::ControlValue in C++; its translated
// dom owner will provide this argument when that package is connected.
fn select_displayed_text_from_control_value(
    document: &Document,
    select_index: usize,
    live_value: &str,
) -> String {
    let mut options = Vec::new();
    collect_options(document, select_index, &mut options);
    let mut selected = None;
    for option_index in &options {
        let option = &document.elements[*option_index];
        let option_value = if let Some(value) = attribute(option, "value") {
            value.to_owned()
        } else {
            let mut text = String::new();
            append_dom_text(document, *option_index, &mut text);
            trim_collapsible_whitespace(&text).to_owned()
        };
        if option_value == live_value {
            selected = Some(*option_index);
            break;
        }
    }
    for option_index in &options {
        if selected.is_none() && attribute(&document.elements[*option_index], "selected").is_some()
        {
            selected = Some(*option_index);
            break;
        }
    }
    if selected.is_none() {
        selected = options.first().copied();
    }
    let Some(selected_index) = selected else {
        return String::new();
    };
    if let Some(label) = attribute(&document.elements[selected_index], "label") {
        return label.to_owned();
    }
    let mut text = String::new();
    append_dom_text(document, selected_index, &mut text);
    trim_collapsible_whitespace(&text).to_owned()
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:706-775
pub(super) fn apply_control_appearance(element: &ElementData, style: &mut ComputedStyle) {
    let Some(control_type) = element.form_control_type else {
        return;
    };
    let appearance = match control_type {
        FormControlType::kInputCheckbox => AppearanceValue::kCheckbox,
        FormControlType::kInputRadio => AppearanceValue::kRadio,
        FormControlType::kInputRange => AppearanceValue::kSliderHorizontal,
        FormControlType::kInputSearch => AppearanceValue::kSearchField,
        FormControlType::kTextArea => AppearanceValue::kTextArea,
        FormControlType::kSelectOne => AppearanceValue::kMenulist,
        FormControlType::kSelectMultiple => AppearanceValue::kListbox,
        FormControlType::kButtonButton
        | FormControlType::kButtonSubmit
        | FormControlType::kButtonReset
        | FormControlType::kButtonPopover
        | FormControlType::kInputButton
        | FormControlType::kInputSubmit
        | FormControlType::kInputReset => AppearanceValue::kButton,
        FormControlType::kInputText
        | FormControlType::kInputEmail
        | FormControlType::kInputPassword
        | FormControlType::kInputTelephone
        | FormControlType::kInputUrl
        | FormControlType::kInputNumber => AppearanceValue::kTextField,
        _ => AppearanceValue::kNone,
    };
    if appearance == AppearanceValue::kNone {
        return;
    }
    let extra = style.extended.get_or_insert_with(ExtendedStyle::default);
    let author_box = extra.has_author_background || extra.has_author_border;
    let author_box_or_shadow = author_box || !style.paint.box_shadows.is_empty();
    let suppress_native = (matches!(
        appearance,
        AppearanceValue::kButton
            | AppearanceValue::kPushButton
            | AppearanceValue::kSquareButton
            | AppearanceValue::kProgressBar
            | AppearanceValue::kMeter
    ) && author_box)
        || (matches!(
            appearance,
            AppearanceValue::kMenulist
                | AppearanceValue::kSearchField
                | AppearanceValue::kTextArea
                | AppearanceValue::kTextField
        ) && author_box_or_shadow);
    if suppress_native {
        return;
    }
    if extra.effective_appearance == AppearanceValue::kNone {
        extra.effective_appearance = appearance;
    }
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:358-371
pub(super) fn is_single_line_text_control(control_type: FormControlType) -> bool {
    matches!(
        control_type,
        FormControlType::kInputText
            | FormControlType::kInputEmail
            | FormControlType::kInputPassword
            | FormControlType::kInputSearch
            | FormControlType::kInputTelephone
            | FormControlType::kInputUrl
            | FormControlType::kInputNumber
    )
}

fn cpp_min(a: f64, b: f64) -> f64 {
    if b < a {
        b
    } else {
        a
    }
}

pub(super) fn cpp_max(a: f64, b: f64) -> f64 {
    if a < b {
        b
    } else {
        a
    }
}

// The namespace branch is selected by DOMNode::Namespace in C++. The
// cpp: dom_to_layout/layout_object_tree_builder.cc:260-300
fn resolve_svg_text_path(document: &Document, node: &Element) -> Option<SvgTextPathData> {
    let href = attribute(node, "href")?;
    let id = href.strip_prefix('#').filter(|id| !id.is_empty())?;
    let root = document.root?;
    let path = &document.elements[find_element_by_id(document, root, id)?];
    if path.namespace != DOMNamespace::kSVG || path.tag != "path" {
        return None;
    }
    let path_data = attribute(path, "d")?;
    let parsed = ParseSVGPathDefault(path_data)?;
    if !parsed.single_subpath || parsed.flattened_points.len() < 2 {
        return None;
    }
    let points = parsed.flattened_points;
    let mut start_offset = 0.0;
    if let Some(start) = attribute(node, "startOffset") {
        if start.ends_with('%') {
            if let Some((percentage, consumed)) = parse_c_double_prefix(start) {
                if start.as_bytes().get(consumed) == Some(&b'%') {
                    let length: f64 = points
                        .windows(2)
                        .map(|pair| (pair[1].x - pair[0].x).hypot(pair[1].y - pair[0].y))
                        .sum();
                    start_offset = length * percentage / 100.0;
                }
            }
        } else {
            start_offset = attribute_number(node, "startOffset", 0.0);
        }
    }
    Some(SvgTextPathData {
        points,
        start_offset,
        reverse_direction: attribute(node, "side").is_some_and(|side| lower_ascii(side) == "right"),
    })
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:376-504
pub(super) fn element_metadata_svg(
    node: &impl LayoutNodeView,
    text_path: Option<SvgTextPathData>,
    data: &mut ElementData,
) {
    // cpp: dom_to_layout/layout_object_tree_builder.cc:379-410
    if node.node_name() == "svg" {
        // LayoutSVGRoot::UnscaledNaturalSizingInfo reads SVGSVGElement's
        // intrinsic width/height independently from the viewBox. A viewBox
        // supplies only an aspect ratio; absolute width/height attributes
        // supply the replaced element's natural size.
        let intrinsic_length = |name| {
            let value = attribute(node, name)?.trim();
            let (number, consumed) = parse_c_double_prefix(value)?;
            let unit = value[consumed..].trim();
            (number.is_finite()
                && number >= 0.0
                && (unit.is_empty() || unit.eq_ignore_ascii_case("px")))
            .then_some(number)
        };
        data.natural_width = intrinsic_length("width");
        data.natural_height = intrinsic_length("height");
        let view_box = svg_number_list(node, "viewBox");
        if view_box.len() == 4 && view_box[2] > 0.0 && view_box[3] > 0.0 {
            let mut resolved = SvgViewBoxData {
                x: view_box[0],
                y: view_box[1],
                width: view_box[2],
                height: view_box[3],
                ..SvgViewBoxData::default()
            };
            if let Some(aspect_ratio) = attribute(node, "preserveAspectRatio") {
                let parts: Vec<&str> = aspect_ratio.split_ascii_whitespace().collect();
                let mut index = usize::from(parts.first() == Some(&"defer"));
                if index < parts.len() {
                    let align = parts[index];
                    index += 1;
                    if align == "none" {
                        resolved.preserve_none = true;
                    } else {
                        let bytes = align.as_bytes();
                        if bytes.len() == 8 && bytes[0] == b'x' && bytes[4] == b'Y' {
                            let horizontal = &bytes[1..4];
                            let vertical = &bytes[5..8];
                            if horizontal == b"Min" {
                                resolved.align_x = -1;
                            } else if horizontal == b"Max" {
                                resolved.align_x = 1;
                            }
                            if vertical == b"Min" {
                                resolved.align_y = -1;
                            } else if vertical == b"Max" {
                                resolved.align_y = 1;
                            }
                        }
                    }
                    if index < parts.len() && parts[index] == "slice" {
                        resolved.slice = true;
                    }
                }
            }
            data.svg_view_box = Some(resolved);
            data.natural_aspect_ratio = Some(view_box[2] / view_box[3]);
        }
        if let (Some(width), Some(height)) = (data.natural_width, data.natural_height) {
            if width > 0.0 && height > 0.0 {
                data.natural_aspect_ratio = Some(width / height);
            }
        }
    }

    // cpp: dom_to_layout/layout_object_tree_builder.cc:444-474
    if node.node_name() == "rect" {
        let x = attribute_number(node, "x", 0.0);
        let y = attribute_number(node, "y", 0.0);
        let rx_attribute = attribute(node, "rx");
        let ry_attribute = attribute(node, "ry");
        let mut rx = cpp_max(0.0, attribute_number(node, "rx", 0.0));
        let mut ry = cpp_max(0.0, attribute_number(node, "ry", 0.0));
        if rx_attribute.is_none() {
            rx = ry;
        }
        if ry_attribute.is_none() {
            ry = rx;
        }
        let width = cpp_max(0.0, attribute_number(node, "width", 0.0));
        let height = cpp_max(0.0, attribute_number(node, "height", 0.0));
        data.svg_shape = Some(SvgShapeData {
            geometry: SvgShapeGeometry::kRectangle,
            bounds_offset: Offset { x, y },
            bounds_width: width,
            bounds_height: height,
            rectangle_radius_x: cpp_min(rx, width / 2.0),
            rectangle_radius_y: cpp_min(ry, height / 2.0),
            ..SvgShapeData::default()
        });
    } else if node.node_name() == "circle" || node.node_name() == "ellipse" {
        let rx = cpp_max(
            0.0,
            attribute_number(
                node,
                if node.node_name() == "circle" {
                    "r"
                } else {
                    "rx"
                },
                0.0,
            ),
        );
        let ry = if node.node_name() == "circle" {
            rx
        } else {
            cpp_max(0.0, attribute_number(node, "ry", 0.0))
        };
        let cx = attribute_number(node, "cx", 0.0);
        let cy = attribute_number(node, "cy", 0.0);
        data.svg_shape = Some(SvgShapeData {
            geometry: SvgShapeGeometry::kEllipse,
            bounds_offset: Offset {
                x: cx - rx,
                y: cy - ry,
            },
            bounds_width: rx * 2.0,
            bounds_height: ry * 2.0,
            ..SvgShapeData::default()
        });
    } else if node.node_name() == "line" {
        // cpp: dom_to_layout/layout_object_tree_builder.cc:237-243
        data.svg_shape = linear_svg_shape(
            vec![
                Offset {
                    x: attribute_number(node, "x1", 0.0),
                    y: attribute_number(node, "y1", 0.0),
                },
                Offset {
                    x: attribute_number(node, "x2", 0.0),
                    y: attribute_number(node, "y2", 0.0),
                },
            ],
            false,
        );
    } else if node.node_name() == "polyline" || node.node_name() == "polygon" {
        if let Some(points) = attribute(node, "points") {
            let path = format!("M {points}");
            if let Some(parsed) = ParseSVGPathDefault(&path) {
                data.svg_shape =
                    linear_svg_shape(parsed.flattened_points, node.node_name() == "polygon");
            }
        }
    } else if node.node_name() == "path" {
        if let Some(path) = attribute(node, "d") {
            if let Some(parsed) = ParseSVGPathDefault(path) {
                data.svg_shape = Some(SvgShapeData {
                    geometry: SvgShapeGeometry::kPath,
                    bounds_offset: parsed.bounds_offset,
                    bounds_width: parsed.bounds_width,
                    bounds_height: parsed.bounds_height,
                    path: parsed.commands,
                    ..SvgShapeData::default()
                });
            }
        }
    }

    if node.node_name() == "textPath" {
        data.svg_text_path = text_path;
    }

    // cpp: dom_to_layout/layout_object_tree_builder.cc:483-497
    if matches!(node.node_name(), "text" | "tspan" | "textPath") {
        data.svg_x = svg_number_list(node, "x");
        data.svg_y = svg_number_list(node, "y");
        data.svg_dx = svg_number_list(node, "dx");
        data.svg_dy = svg_number_list(node, "dy");
        data.svg_rotate = svg_number_list(node, "rotate");
        data.svg_text_length = finite_attribute_number(node, "textLength");
        if let Some(adjust) = attribute(node, "lengthAdjust") {
            data.svg_length_adjust = if lower_ascii(adjust) == "spacingandglyphs" {
                SvgLengthAdjust::kSpacingAndGlyphs
            } else {
                SvgLengthAdjust::kSpacing
            };
        }
    }
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:376-504
fn element_metadata(
    document: &Document,
    index: usize,
    interaction_state: &UserInteractionState,
) -> ElementData {
    let node = &document.elements[index];
    match node.namespace {
        DOMNamespace::kSVG => {
            let mut data = ElementData::default();
            element_metadata_svg(node, resolve_svg_text_path(document, node), &mut data);
            data
        }
        DOMNamespace::kMathML => {
            let mut data = ElementData::default();
            element_metadata_mathml(node, &mut data);
            data
        }
        DOMNamespace::kHTML => element_metadata_html(document, index, interaction_state),
        DOMNamespace::kNone => ElementData::default(),
    }
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:633-703
fn element_kind(
    document: &Document,
    index: usize,
    element: &ElementData,
    style: &ComputedStyle,
) -> NodeKind {
    let node = &document.elements[index];
    match node.namespace {
        DOMNamespace::kSVG => match node.tag.as_str() {
            "svg" => NodeKind::kSvgRoot,
            "foreignObject" => NodeKind::kSvgForeignObject,
            "text" => NodeKind::kSvgText,
            "textPath" if element.svg_text_path.is_some() => NodeKind::kSvgTextPath,
            "tspan" => NodeKind::kSvgTSpan,
            "a" if node.parent.is_some_and(|parent| {
                let ancestor = &document.elements[parent];
                ancestor.namespace == DOMNamespace::kSVG && ancestor.tag == "text"
            }) =>
            {
                NodeKind::kSvgInline
            }
            _ if element.svg_shape.is_some() => NodeKind::kSvgShape,
            _ => NodeKind::kSvgGroup,
        },
        DOMNamespace::kMathML => element_kind_mathml(node),
        DOMNamespace::kHTML => element_kind_html(node, element, style),
        DOMNamespace::kNone => NodeKind::kBox,
    }
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:498-503
pub(super) fn element_metadata_mathml(node: &impl LayoutNodeView, data: &mut ElementData) {
    data.math_token_kind = match node.node_name() {
        "mn" => MathTokenKind::kNumber,
        "mtext" => MathTokenKind::kText,
        "ms" => MathTokenKind::kString,
        _ => MathTokenKind::kIdentifier,
    };
}

// The Rust DOM currently has no live control-value or persistent interaction-state
// owner. This maps the source's HTML attribute and registered-image branches;
// the other branches remain recorded as unconnected dependencies.
fn element_metadata_html(
    document: &Document,
    index: usize,
    interaction_state: &UserInteractionState,
) -> ElementData {
    let node = &document.elements[index];
    let mut data = ElementData::default();
    // cpp: dom_to_layout/layout_object_tree_builder.cc:411-426
    data.nowrap_attribute = attribute(node, "nowrap").is_some();
    data.html_image = node.tag == "img";
    data.html_ordered_or_unordered_list = node.tag == "ol" || node.tag == "ul";
    if node.tag == "html" {
        data.viewport_defining = true;
        data.document_role = DocumentRole::kDocumentElement;
    } else if node.tag == "body" {
        data.document_role = DocumentRole::kBody;
    }
    if node.tag == "img" {
        let width = attribute_number(node, "width", -1.0);
        let height = attribute_number(node, "height", -1.0);
        if width >= 0.0 {
            data.natural_width = Some(width);
        }
        if height >= 0.0 {
            data.natural_height = Some(height);
        }
        if width > 0.0 && height > 0.0 {
            data.natural_aspect_ratio = Some(width / height);
        }
        // cpp: dom_to_layout/layout_object_tree_builder.cc:427-435
        if let Some(source) = attribute(node, "src") {
            if let Some(image) = document.ImageResourceFor(source) {
                if data.natural_width.is_none() {
                    data.natural_width = Some(image.natural_width);
                }
                if data.natural_height.is_none() {
                    data.natural_height = Some(image.natural_height);
                }
                if data.natural_aspect_ratio.is_none() && image.natural_height > 0.0 {
                    data.natural_aspect_ratio = Some(image.natural_width / image.natural_height);
                }
                data.image_device_pixel_ratio = image.resolution_scale;
            }
        }
    }
    // cpp: dom_to_layout/layout_object_tree_builder.cc:438-443
    if node.tag == "td" || node.tag == "th" {
        data.row_span = attribute_number(node, "rowspan", 1.0).max(1.0) as u32;
        data.column_span = attribute_number(node, "colspan", 1.0).max(1.0) as u32;
    } else if node.tag == "col" || node.tag == "colgroup" {
        data.column_span = attribute_number(node, "span", 1.0).max(1.0) as u32;
    }

    // cpp: dom_to_layout/layout_object_tree_builder.cc:506-513
    // cpp: dom_to_layout/layout_object_tree_builder.cc:514-543
    data.control_disabled = attribute(node, "disabled").is_some();
    let node_id = index as u64 + 1;
    data.control_hovered = interaction_state.hovered_node_id == Some(node_id);
    data.control_active = interaction_state.pressed_node_id == Some(node_id);
    data.control_focused = interaction_state.focus_visible_node_id == Some(node_id);
    let mut fieldset = node.parent;
    while let Some(fieldset_index) = fieldset {
        let ancestor = &document.elements[fieldset_index];
        if ancestor.tag == "fieldset" && attribute(ancestor, "disabled").is_some() {
            let first_legend = ancestor.children.iter().find_map(|child| match child {
                Child::Element(child_index) if document.elements[*child_index].tag == "legend" => {
                    Some(*child_index)
                }
                _ => None,
            });
            let mut inside_first_legend = false;
            let mut current = Some(index);
            while let Some(current_index) = current {
                if Some(current_index) == first_legend {
                    inside_first_legend = true;
                    break;
                }
                if current_index == fieldset_index {
                    break;
                }
                current = document.elements[current_index].parent;
            }
            if !inside_first_legend {
                data.control_disabled = true;
                break;
            }
        }
        fieldset = ancestor.parent;
    }
    data.control_read_only = attribute(node, "readonly").is_some();
    // cpp: dom_to_layout/layout_object_tree_builder.cc:544-567
    if node.tag == "marquee" {
        let direction = lower_ascii(attribute(node, "direction").unwrap_or("left"));
        data.marquee_horizontal = direction != "up" && direction != "down";
    }
    if node.tag == "frameset" {
        data.frame_rows = frame_dimensions(node, "rows");
        data.frame_columns = frame_dimensions(node, "cols");
        data.frame_border = frame_border(node);
        data.frame_no_resize = attribute(node, "noresize").is_some();
        data.frame_has_border_color = attribute(node, "bordercolor").is_some();
        if let Some(border) = attribute(node, "border") {
            let bytes = c_string_bytes(border);
            let begin = bytes.as_ptr().cast::<std::ffi::c_char>();
            let mut end = std::ptr::null_mut();
            let value = unsafe { strtol(begin, &mut end, 10) };
            let consumed = unsafe { end.offset_from(begin) } as usize;
            data.frame_border_thickness = Some(if consumed == 0 {
                0
            } else {
                value.clamp(0, i32::MAX as std::ffi::c_long) as i32
            });
        }
    } else if node.tag == "frame" {
        data.frame_border = frame_border(node);
        data.frame_no_resize = attribute(node, "noresize").is_some();
    }

    // cpp: dom_to_layout/layout_object_tree_builder.cc:568-625
    if node.tag == "textarea" {
        data.form_control_type = Some(FormControlType::kTextArea);
        data.text_area_sizing = Some(TextAreaSizing {
            columns: positive_attribute_integer(node, "cols", 20),
            rows: positive_attribute_integer(node, "rows", 2),
        });
    } else if node.tag == "select" {
        let multiple = attribute(node, "multiple").is_some();
        data.form_control_type = Some(if multiple {
            FormControlType::kSelectMultiple
        } else {
            FormControlType::kSelectOne
        });
        data.select_uses_menu_list =
            Some(!multiple && positive_attribute_integer(node, "size", 0) <= 1);
    } else if node.tag == "button" {
        let button_type = lower_ascii(attribute(node, "type").unwrap_or("submit"));
        data.form_control_type = Some(match button_type.as_str() {
            "reset" => FormControlType::kButtonReset,
            "button" => FormControlType::kButtonButton,
            _ => FormControlType::kButtonSubmit,
        });
    } else if node.tag == "input" {
        let input_type = lower_ascii(attribute(node, "type").unwrap_or("text"));
        let control_type = match input_type.as_str() {
            "checkbox" => FormControlType::kInputCheckbox,
            "color" => FormControlType::kInputColor,
            "date" => FormControlType::kInputDate,
            "datetime-local" => FormControlType::kInputDatetimeLocal,
            "file" => FormControlType::kInputFile,
            "hidden" => FormControlType::kInputHidden,
            "image" => FormControlType::kInputImage,
            "month" => FormControlType::kInputMonth,
            "radio" => FormControlType::kInputRadio,
            "range" => FormControlType::kInputRange,
            "number" => FormControlType::kInputNumber,
            "password" => FormControlType::kInputPassword,
            "email" => FormControlType::kInputEmail,
            "search" => FormControlType::kInputSearch,
            "tel" => FormControlType::kInputTelephone,
            "url" => FormControlType::kInputUrl,
            "button" => FormControlType::kInputButton,
            "submit" => FormControlType::kInputSubmit,
            "reset" => FormControlType::kInputReset,
            "time" => FormControlType::kInputTime,
            "week" => FormControlType::kInputWeek,
            _ => FormControlType::kInputText,
        };
        data.form_control_type = Some(control_type);
        if is_single_line_text_control(control_type) {
            data.text_field_sizing = Some(TextFieldSizing {
                preferred_size: positive_attribute_integer(node, "size", 20) as i32,
                includes_decoration: false,
            });
        }
        if control_type == FormControlType::kInputImage {
            let width = attribute_number(node, "width", -1.0);
            let height = attribute_number(node, "height", -1.0);
            if width >= 0.0 {
                data.natural_width = Some(width);
            }
            if height >= 0.0 {
                data.natural_height = Some(height);
            }
            if width > 0.0 && height > 0.0 {
                data.natural_aspect_ratio = Some(width / height);
            }
        }
        if control_type == FormControlType::kInputFile {
            data.file_no_file_label = Some("No file selected".to_owned());
        }
    }
    data
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:657-680
// This is the HTML branch only. The Rust DOM does not yet carry the C++
// namespace identity needed to select this branch for every element.
pub(super) fn element_kind_html(
    node: &impl LayoutNodeView,
    element: &ElementData,
    style: &ComputedStyle,
) -> NodeKind {
    match node.node_name() {
        "br" => return NodeKind::kLineBreak,
        "wbr" => return NodeKind::kWordBreak,
        "img" | "canvas" | "video" | "audio" | "iframe" | "embed" | "object" => {
            return NodeKind::kReplaced
        }
        "fieldset" => return NodeKind::kFieldset,
        "legend" => return NodeKind::kLegend,
        "input" if element.form_control_type == Some(FormControlType::kInputImage) => {
            return NodeKind::kReplaced
        }
        "input" | "button" | "select" | "textarea" => return NodeKind::kFormControl,
        "frameset" => return NodeKind::kFrameSet,
        "frame" => return NodeKind::kFrame,
        "marquee" => return NodeKind::kMarquee,
        _ => {}
    }
    if style.display == Display::kListItem {
        return NodeKind::kListItem;
    }
    match node.node_name() {
        "ruby" => NodeKind::kRuby,
        "rb" => NodeKind::kRubyBase,
        "rt" => NodeKind::kRubyAnnotation,
        _ => NodeKind::kBox,
    }
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:681-703
// The caller must select this branch using the source-owned DOM namespace.
pub(super) fn element_kind_mathml(node: &impl LayoutNodeView) -> NodeKind {
    match node.node_name() {
        "mfrac" => NodeKind::kMathFraction,
        "msqrt" => NodeKind::kMathSquareRoot,
        "mroot" => NodeKind::kMathRoot,
        "mpadded" => NodeKind::kMathPadded,
        "mspace" => NodeKind::kMathSpace,
        "mo" => NodeKind::kMathOperator,
        "mi" | "mn" | "mtext" | "ms" => NodeKind::kMathToken,
        "msub" => NodeKind::kMathSub,
        "msup" => NodeKind::kMathSup,
        "msubsup" => NodeKind::kMathSubSup,
        "munder" => NodeKind::kMathUnder,
        "mover" => NodeKind::kMathOver,
        "munderover" => NodeKind::kMathUnderOver,
        "mtd" => NodeKind::kMathTableCell,
        "mtable" => NodeKind::kMathTable,
        "mmultiscripts" => NodeKind::kMathMultiscripts,
        "mprescripts" => NodeKind::kMathPrescripts,
        "none" => NodeKind::kMathNone,
        "math" => NodeKind::kMathContainer,
        _ => NodeKind::kMathRow,
    }
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:808-856
fn default_list_marker_text(
    document: &Document,
    item_index: usize,
    style: &ComputedStyle,
) -> String {
    let defaults = ExtendedStyle::default();
    let extra = style.extended.as_ref().unwrap_or(&defaults);
    match extra.list_style_type {
        ListStyleType::kNone => return String::new(),
        ListStyleType::kCircle => return "○ ".to_owned(),
        ListStyleType::kSquare => return "■ ".to_owned(),
        ListStyleType::kDisc => return "• ".to_owned(),
        ListStyleType::kDecimal | ListStyleType::kLowerAlpha => {}
    }
    let item = &document.elements[item_index];
    let Some(list_index) = item.parent else {
        return "1. ".to_owned();
    };
    let list = &document.elements[list_index];
    if list.tag != "ol" {
        return "1. ".to_owned();
    }

    let reversed = list.attributes.iter().any(|(key, _)| key == "reversed");
    let mut value = attribute_integer(list, "start").unwrap_or_else(|| {
        if reversed {
            list.children
                .iter()
                .filter(|child| {
                    matches!(child, Child::Element(index) if document.elements[*index].tag == "li")
                })
                .count() as i32
        } else {
            1
        }
    });
    let step = if reversed { -1 } else { 1 };
    for child in &list.children {
        let Child::Element(index) = *child else {
            continue;
        };
        let sibling = &document.elements[index];
        if sibling.tag != "li" {
            continue;
        }
        if let Some(explicit_value) = attribute_integer(sibling, "value") {
            value = explicit_value;
        }
        if index == item_index {
            if extra.list_style_type == ListStyleType::kLowerAlpha {
                let mut marker = String::new();
                let mut ordinal = value;
                while ordinal > 0 {
                    ordinal -= 1;
                    marker.insert(0, char::from(b'a' + (ordinal % 26) as u8));
                    ordinal /= 26;
                }
                return format!(
                    "{}. ",
                    if marker.is_empty() {
                        value.to_string()
                    } else {
                        marker
                    }
                );
            }
            return format!("{value}. ");
        }
        value += step;
    }
    format!("{value}. ")
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:1428-1462
fn append_list_marker(
    tree: &mut LayoutTreeUpdate<'_>,
    document: &Document,
    index: usize,
    style: &ComputedStyle,
    object: *mut LayoutObject,
) {
    let marker_text = default_list_marker_text(document, index, style);
    if style.display != Display::kListItem || marker_text.is_empty() {
        return;
    }
    let id = index as u64 + 1;
    let marker = tree.AddBox(
        unsafe { &mut *object },
        id | (1_u64 << 63),
        &list_marker_style(style),
        "::marker".to_owned(),
        None,
        NodeKind::kListMarker,
        true,
        std::ptr::null(),
    ) as *mut LayoutObject;
    let defaults = ExtendedStyle::default();
    let marker_type = style.extended.as_ref().unwrap_or(&defaults).list_style_type;
    let symbol = matches!(
        marker_type,
        ListStyleType::kDisc | ListStyleType::kCircle | ListStyleType::kSquare
    );
    if symbol && marker_text.ends_with(' ') {
        let symbol_text = marker_text[..marker_text.len() - 1].to_owned();
        tree.AddText(
            unsafe { &mut *marker },
            id | (1_u64 << 62),
            symbol_text,
            "::marker-symbol".to_owned(),
            std::ptr::null(),
            true,
            std::ptr::null(),
        );
        let mut suffix_style = list_marker_style(style);
        suffix_style.width = Some(12.0);
        suffix_style.height = Some(0.0);
        tree.AddBox(
            unsafe { &mut *marker },
            id | (1_u64 << 61),
            &suffix_style,
            "::marker-suffix".to_owned(),
            None,
            NodeKind::kBox,
            true,
            std::ptr::null(),
        );
    } else {
        tree.AddText(
            unsafe { &mut *marker },
            id | (1_u64 << 62),
            marker_text,
            "::marker-text".to_owned(),
            std::ptr::null(),
            true,
            std::ptr::null(),
        );
    }
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:853-864
pub(super) fn preserves_breaks(style: &ComputedStyle) -> bool {
    let Some(extra) = &style.extended else {
        return false;
    };
    matches!(
        extra.white_space,
        WhiteSpace::kPre | WhiteSpace::kPreLine | WhiteSpace::kPreWrap | WhiteSpace::kBreakSpaces
    )
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:866-868
pub(super) fn contains_only_collapsible_whitespace(text: &str) -> bool {
    text.bytes()
        .all(|byte| matches!(byte, b' ' | b'\t' | b'\r' | b'\n' | 0x0c))
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:870-902
fn suppresses_collapsible_whitespace_children(
    tree: &LayoutTreeUpdate<'_>,
    document: &Document,
    styles: &[ComputedStyle],
    generation: Option<(&[bool], &[bool])>,
    parent_index: usize,
    layout_parent: &LayoutObject,
) -> bool {
    let mut node = Some(parent_index);
    while let Some(index) = node {
        let generates_box = generation.is_none_or(|(flags, _)| flags[index]);
        if generates_box {
            if matches!(
                styles[index].display,
                Display::kFlex
                    | Display::kInlineFlex
                    | Display::kGrid
                    | Display::kInlineGrid
                    | Display::kTable
                    | Display::kInlineTable
                    | Display::kTableRow
                    | Display::kTableSection
                    | Display::kTableHeaderGroup
                    | Display::kTableFooterGroup
                    | Display::kTableColumnGroup
            ) {
                return true;
            }
            return !tree.NeedsCollapsibleWhitespace(layout_parent);
        }
        node = document.elements[index].parent;
    }
    false
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:904-912
fn first_letter_punctuation(character: i32) -> bool {
    let category = Category(character);
    matches!(
        category,
        unicode::kPunctuation_Open
            | unicode::kPunctuation_Close
            | unicode::kPunctuation_InitialQuote
            | unicode::kPunctuation_FinalQuote
            | unicode::kPunctuation_Other
    )
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:914-916
fn first_letter_punctuation_at(text: &StringView, offset: u32) -> bool {
    first_letter_punctuation(text.CodePointAt(offset))
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:918-918
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum FirstLetterPunctuationState {
    NotSeen,
    Seen,
    Disallow,
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:920-954
pub(super) fn first_letter_length(
    text: &StringView,
    preserve_breaks: bool,
    punctuation: &mut FirstLetterPunctuationState,
) -> u32 {
    let mut length = 0;
    let text_length = text.length();
    if text_length == 0 {
        return 0;
    }
    let is_newline = |character: u16| character == 0x0a || character == 0x0d;
    let is_space = |character: u16| {
        ((if preserve_breaks {
            !is_newline(character) && unicode::IsSpaceOrNewline(character)
        } else {
            unicode::IsSpaceOrNewline(character)
        }) || character == 0x00a0)
    };
    if *punctuation == FirstLetterPunctuationState::NotSeen {
        while length < text_length && is_space(text.Span16()[length as usize]) {
            length += 1;
        }
        if length == text_length {
            return 0;
        }
    }
    let punctuation_start = length;
    while length < text_length && first_letter_punctuation_at(text, length) {
        length += LengthOfGraphemeCluster(text, length);
    }
    if length == text_length {
        if length > punctuation_start {
            *punctuation = FirstLetterPunctuationState::Seen;
        }
        return length;
    }
    *punctuation = FirstLetterPunctuationState::Disallow;
    if is_space(text.Span16()[length as usize]) || is_newline(text.Span16()[length as usize]) {
        return 0;
    }
    length += LengthOfGraphemeCluster(text, length);
    while length < text_length && first_letter_punctuation_at(text, length) {
        length += LengthOfGraphemeCluster(text, length);
    }
    length
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:1017-1021
pub(super) fn is_block_container_display(display: Display) -> bool {
    matches!(
        display,
        Display::kBlock
            | Display::kFlowRoot
            | Display::kListItem
            | Display::kTableCell
            | Display::kTableCaption
    )
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:1167-1206
fn append_pseudo_element(
    tree: &mut LayoutTreeUpdate<'_>,
    parent: *mut LayoutObject,
    originating_id: u64,
    pseudo: Option<&PseudoElement>,
    before: bool,
) {
    let Some(pseudo) = pseudo else { return };
    let box_bit = 1u64 << if before { 50 } else { 48 };
    let text_bit = 1u64 << if before { 49 } else { 47 };
    let name = if before { "::before" } else { "::after" };
    if pseudo.display_contents {
        if !pseudo.text.is_empty() {
            tree.AddText(
                unsafe { &mut *parent },
                originating_id | text_bit,
                pseudo.text.clone(),
                format!("{name}-text"),
                &pseudo.style,
                true,
                std::ptr::null(),
            );
        }
        return;
    }
    let object = tree.AddBox(
        unsafe { &mut *parent },
        originating_id | box_bit,
        &pseudo.style,
        name.to_owned(),
        None,
        NodeKind::kBox,
        true,
        std::ptr::null(),
    ) as *mut LayoutObject;
    if !pseudo.text.is_empty() {
        tree.AddText(
            unsafe { &mut *object },
            originating_id | text_bit,
            pseudo.text.clone(),
            format!("{name}-text"),
            std::ptr::null(),
            true,
            std::ptr::null(),
        );
    }
}

fn append_children(
    tree: &mut LayoutTreeUpdate<'_>,
    document: &Document,
    styles: &[ComputedStyle],
    generation: Option<(&[bool], &[bool])>,
    pseudo: Option<(&[Option<PseudoElement>], &[Option<PseudoElement>])>,
    interaction_state: &UserInteractionState,
    parent_index: usize,
    parent: *mut LayoutObject,
) {
    for child in &document.elements[parent_index].children {
        match *child {
            Child::Element(index) => {
                if document.elements[index].namespace == DOMNamespace::kSVG
                    && matches!(
                        document.elements[index].tag.as_str(),
                        "title" | "desc" | "metadata"
                    )
                {
                    continue;
                }
                if let Some((generates_box, display_contents)) = generation {
                    if !generates_box[index] {
                        if display_contents[index] {
                            if let Some((before, _)) = pseudo {
                                append_pseudo_element(
                                    tree,
                                    parent,
                                    index as u64 + 1,
                                    before[index].as_ref(),
                                    true,
                                );
                            }
                            // cpp: dom_to_layout/layout_object_tree_builder.cc:1261-1271
                            append_children(
                                tree,
                                document,
                                styles,
                                generation,
                                pseudo,
                                interaction_state,
                                index,
                                parent,
                            );
                            if let Some((_, after)) = pseudo {
                                append_pseudo_element(
                                    tree,
                                    parent,
                                    index as u64 + 1,
                                    after[index].as_ref(),
                                    false,
                                );
                            }
                        }
                        continue;
                    }
                }
                // cpp: dom_to_layout/layout_object_tree_builder.cc:1279-1283
                let element = element_metadata(document, index, interaction_state);
                let mut style = styles[index].clone();
                // cpp: dom_to_layout/layout_object_tree_builder.cc:1278-1285
                if document.elements[index].tag == "img" {
                    if let Some(source) = attribute(&document.elements[index], "src") {
                        if let Some(image) = document.ImageResourceFor(source) {
                            style.paint.image_resource_id = Some(image.id);
                        }
                    }
                }
                apply_control_appearance(&element, &mut style);
                // cpp: dom_to_layout/layout_object_tree_builder.cc:1286-1315
                let kind = element_kind(document, index, &element, &style);
                let accepts_generated_content = kind != NodeKind::kReplaced
                    && kind != NodeKind::kFrame
                    && !matches!(
                        element.form_control_type,
                        Some(
                            FormControlType::kTextArea
                                | FormControlType::kSelectOne
                                | FormControlType::kSelectMultiple
                                | FormControlType::kInputRange
                        )
                    );
                let child = if kind == NodeKind::kReplaced || kind == NodeKind::kFrame {
                    tree.AddReplaced(
                        unsafe { &mut *parent },
                        index as u64 + 1,
                        &style,
                        Some(element),
                        document.elements[index].tag.clone(),
                        kind,
                    )
                } else {
                    tree.AddBox(
                        unsafe { &mut *parent },
                        index as u64 + 1,
                        &style,
                        document.elements[index].tag.clone(),
                        Some(element),
                        kind,
                        false,
                        std::ptr::null(),
                    )
                } as *mut LayoutObject;
                append_list_marker(tree, document, index, &style, child);
                if accepts_generated_content {
                    if let Some((before, _)) = pseudo {
                        append_pseudo_element(
                            tree,
                            child,
                            index as u64 + 1,
                            before[index].as_ref(),
                            true,
                        );
                    }
                }
                append_children(
                    tree,
                    document,
                    styles,
                    generation,
                    pseudo,
                    interaction_state,
                    index,
                    child,
                );
                if accepts_generated_content {
                    if let Some((_, after)) = pseudo {
                        append_pseudo_element(
                            tree,
                            child,
                            index as u64 + 1,
                            after[index].as_ref(),
                            false,
                        );
                    }
                }
            }
            Child::Text(index) => {
                let text = &document.texts[index];
                let svg_text_parent = document.elements[parent_index].namespace
                    == DOMNamespace::kSVG
                    && matches!(
                        document.elements[parent_index].tag.as_str(),
                        "text" | "tspan" | "textPath" | "a"
                    );
                if document.elements[parent_index].namespace == DOMNamespace::kSVG
                    && !svg_text_parent
                {
                    continue;
                }
                // cpp: dom_to_layout/layout_object_tree_builder.cc:1216-1245
                if !preserves_breaks(&styles[parent_index])
                    && contains_only_collapsible_whitespace(text)
                    && suppresses_collapsible_whitespace_children(
                        tree,
                        document,
                        styles,
                        generation,
                        parent_index,
                        unsafe { &*parent },
                    )
                {
                    continue;
                }
                tree.AddTextDefault(
                    unsafe { &mut *parent },
                    (document.elements.len() + index + 1) as u64,
                    text.clone(),
                );
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn Build(
    tree: &mut LayoutTreeUpdate<'_>,
    document: &Document,
    styles: &[ComputedStyle],
) -> *mut LayoutObject {
    assert_eq!(document.elements.len(), styles.len());
    let root_index = document.root.expect("missing HTML root");
    let root = tree.CreateRoot(
        root_index as u64 + 1,
        &styles[root_index],
        document.elements[root_index].tag.clone(),
        Some(element_metadata(
            document,
            root_index,
            &UserInteractionState::default(),
        )),
        std::ptr::null(),
    ) as *mut LayoutObject;
    append_children(
        tree,
        document,
        styles,
        None,
        None,
        &UserInteractionState::default(),
        root_index,
        root,
    );
    root
}

#[cfg(test)]
mod svg_text_path_tests {
    use super::*;

    fn svg_element(tag: &str, attributes: Vec<(&str, &str)>, children: Vec<Child>) -> Element {
        Element {
            tag: tag.to_owned(),
            namespace: DOMNamespace::kSVG,
            id: None,
            classes: Vec::new(),
            inline_style: None,
            attributes: attributes
                .into_iter()
                .map(|(key, value)| (key.to_owned(), value.to_owned()))
                .collect(),
            parent: None,
            children,
        }
    }

    #[test]
    fn resolves_referenced_path_percentage_and_direction_like_cpp() {
        let mut document = Document::default();
        document.elements = vec![
            svg_element("svg", vec![], vec![Child::Element(1), Child::Element(2)]),
            svg_element("path", vec![("id", "route"), ("d", "M0 0 L200 0")], vec![]),
            svg_element(
                "textPath",
                vec![
                    ("href", "#route"),
                    ("startOffset", "25%"),
                    ("side", "RIGHT"),
                ],
                vec![],
            ),
        ];
        document.root = Some(0);
        let path = resolve_svg_text_path(&document, &document.elements[2]).unwrap();
        assert_eq!(path.points.len(), 2);
        assert_eq!(path.start_offset, 50.0);
        assert!(path.reverse_direction);
        let mut invalid = document.clone();
        invalid.elements[1].attributes[1].1 = "M0 0 L10 0 M20 0 L30 0".to_owned();
        assert!(resolve_svg_text_path(&invalid, &invalid.elements[2]).is_none());
    }
}

#[allow(non_snake_case)]
pub fn BuildResolved(
    tree: &mut LayoutTreeUpdate<'_>,
    document: &Document,
    resolved: &crate::style_resolver::ResolvedStyles,
) -> *mut LayoutObject {
    BuildResolvedWithInteraction(tree, document, resolved, &UserInteractionState::default())
}

#[allow(non_snake_case)]
pub fn BuildResolvedWithInteraction(
    tree: &mut LayoutTreeUpdate<'_>,
    document: &Document,
    resolved: &crate::style_resolver::ResolvedStyles,
    interaction_state: &UserInteractionState,
) -> *mut LayoutObject {
    assert_eq!(document.elements.len(), resolved.styles.len());
    assert_eq!(resolved.styles.len(), resolved.generates_box.len());
    assert_eq!(resolved.styles.len(), resolved.display_contents.len());
    assert_eq!(resolved.styles.len(), resolved.before.len());
    assert_eq!(resolved.styles.len(), resolved.after.len());
    let root_index = document.root.expect("missing HTML root");
    assert!(
        resolved.generates_box[root_index],
        "document root needs a box"
    );
    // cpp: dom_to_layout/layout_object_tree_builder.cc:1515-1518
    let root = tree.CreateRoot(
        root_index as u64 + 1,
        &resolved.styles[root_index],
        document.elements[root_index].tag.clone(),
        Some(element_metadata(document, root_index, interaction_state)),
        std::ptr::null(),
    ) as *mut LayoutObject;
    append_pseudo_element(
        tree,
        root,
        root_index as u64 + 1,
        resolved.before[root_index].as_ref(),
        true,
    );
    append_children(
        tree,
        document,
        &resolved.styles,
        Some((&resolved.generates_box, &resolved.display_contents)),
        Some((&resolved.before, &resolved.after)),
        interaction_state,
        root_index,
        root,
    );
    append_pseudo_element(
        tree,
        root,
        root_index as u64 + 1,
        resolved.after[root_index].as_ref(),
        false,
    );
    root
}
