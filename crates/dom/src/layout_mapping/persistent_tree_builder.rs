#![allow(non_snake_case)]
use crate::layout_mapping::layout_object_tree_builder::*;
use crate::persistent_document::{DOMNamespace, DOMNodeType, PseudoElement};
use crate::svg_path_parser::ParseSVGPathDefault;
use crate::{Document, UserInteractionState, DOM};
use layoutng_assembly::internal::form_control_types::FormControlType;
use layoutng_assembly::internal::layout_input::*;
use layoutng_assembly::internal::layout_object::LayoutObject;
#[cfg(test)]
use layoutng_assembly::layout_engine::LayoutEngine;
use layoutng_assembly::layout_engine::{LayoutMutation, LayoutTreeUpdate};
use layoutng_style::style::appearance::AppearanceValue;

// cpp: dom_to_layout/layout_object_tree_builder.cc:376-631
fn ElementMetadata(
    document: &crate::Document,
    index: usize,
    interaction_state: &UserInteractionState,
) -> ElementData {
    let node = document.Node(index);
    let mut data = ElementData::default();
    // cpp: dom_to_layout/layout_object_tree_builder.cc:411-426
    data.nowrap_attribute = attribute(node, "nowrap").is_some();
    data.html_image = node.Name() == "img";
    data.html_ordered_or_unordered_list = node.Name() == "ol" || node.Name() == "ul";
    if node.Name() == "html" {
        data.viewport_defining = true;
        data.document_role = DocumentRole::kDocumentElement;
    } else if node.Name() == "body" {
        data.document_role = DocumentRole::kBody;
    }
    if node.Name() == "img" {
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
    if node.Name() == "td" || node.Name() == "th" {
        data.row_span = attribute_number(node, "rowspan", 1.0).max(1.0) as u32;
        data.column_span = attribute_number(node, "colspan", 1.0).max(1.0) as u32;
    } else if node.Name() == "col" || node.Name() == "colgroup" {
        data.column_span = attribute_number(node, "span", 1.0).max(1.0) as u32;
    }

    // cpp: dom_to_layout/layout_object_tree_builder.cc:506-513
    // cpp: dom_to_layout/layout_object_tree_builder.cc:514-543
    data.scroll_offset = document.ScrollOffsetFor(index);
    if node.Namespace() == DOMNamespace::kSVG {
        element_metadata_svg(node, ResolveSVGTextPath(document, index), &mut data);
    }
    if node.Namespace() == DOMNamespace::kMathML {
        element_metadata_mathml(node, &mut data);
    }
    if node.Namespace() != DOMNamespace::kHTML {
        return data;
    }
    data.control_checked = document.ControlChecked(index);
    data.control_disabled = attribute(node, "disabled").is_some();
    let node_id = node.Id();
    data.control_hovered = interaction_state.hovered_node_id == Some(node_id);
    data.control_active = interaction_state.pressed_node_id == Some(node_id);
    data.control_focused = interaction_state.focus_visible_node_id == Some(node_id);
    let mut fieldset = node.Parent();
    while let Some(fieldset_index) = fieldset {
        let ancestor = document.Node(fieldset_index);
        if ancestor.IsHTMLElement("fieldset") && attribute(ancestor, "disabled").is_some() {
            let first_legend = ancestor
                .Children()
                .iter()
                .copied()
                .find(|&child| document.Node(child).IsHTMLElement("legend"));
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
                current = document.Node(current_index).Parent();
            }
            if !inside_first_legend {
                data.control_disabled = true;
                break;
            }
        }
        fieldset = ancestor.Parent();
    }
    data.control_read_only = attribute(node, "readonly").is_some();
    // cpp: dom_to_layout/layout_object_tree_builder.cc:544-567
    if node.Name() == "marquee" {
        let direction = lower_ascii(attribute(node, "direction").unwrap_or("left"));
        data.marquee_horizontal = direction != "up" && direction != "down";
    }
    if node.Name() == "frameset" {
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
    } else if node.Name() == "frame" {
        data.frame_border = frame_border(node);
        data.frame_no_resize = attribute(node, "noresize").is_some();
    }

    // cpp: dom_to_layout/layout_object_tree_builder.cc:568-625
    if node.Name() == "textarea" {
        data.form_control_type = Some(FormControlType::kTextArea);
        data.text_area_sizing = Some(TextAreaSizing {
            columns: positive_attribute_integer(node, "cols", 20),
            rows: positive_attribute_integer(node, "rows", 2),
        });
    } else if node.Name() == "select" {
        let multiple = attribute(node, "multiple").is_some();
        data.form_control_type = Some(if multiple {
            FormControlType::kSelectMultiple
        } else {
            FormControlType::kSelectOne
        });
        data.select_uses_menu_list =
            Some(!multiple && positive_attribute_integer(node, "size", 0) <= 1);
    } else if node.Name() == "button" {
        let button_type = lower_ascii(attribute(node, "type").unwrap_or("submit"));
        data.form_control_type = Some(match button_type.as_str() {
            "reset" => FormControlType::kButtonReset,
            "button" => FormControlType::kButtonButton,
            _ => FormControlType::kButtonSubmit,
        });
    } else if node.Name() == "input" {
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
    if data.form_control_type == Some(FormControlType::kInputRange) {
        let ratio = range_value_ratio_from_control_value(node, &document.ControlValue(index));
        data.range_value_ratio = Some(ratio);
        data.control_value_ratio = Some(ratio);
    }
    data
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:182-193
fn FindElementById(document: &Document, index: usize, id: &str) -> Option<usize> {
    let node = document.Node(index);
    if node.Type() == DOMNodeType::kElement && attribute(node, "id") == Some(id) {
        return Some(index);
    }
    node.Children()
        .iter()
        .find_map(|&child| FindElementById(document, child, id))
}
// cpp: dom_to_layout/layout_object_tree_builder.cc:260-300
fn ResolveSVGTextPath(document: &Document, index: usize) -> Option<SvgTextPathData> {
    let node = document.Node(index);
    let href = attribute(node, "href")?;
    let id = href.strip_prefix('#').filter(|id| !id.is_empty())?;
    let path = document.Node(FindElementById(document, document.Root(), id)?);
    if path.Namespace() != DOMNamespace::kSVG || path.Name() != "path" {
        return None;
    }
    let parsed = ParseSVGPathDefault(attribute(path, "d")?)?;
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
                        .map(|p| (p[1].x - p[0].x).hypot(p[1].y - p[0].y))
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
// cpp: dom_to_layout/layout_object_tree_builder.cc:633-703
fn ElementKind(
    document: &Document,
    index: usize,
    element: &ElementData,
    style: &ComputedStyle,
) -> NodeKind {
    let node = document.Node(index);
    match node.Namespace() {
        DOMNamespace::kSVG => match node.Name() {
            "svg" => NodeKind::kSvgRoot,
            "foreignObject" => NodeKind::kSvgForeignObject,
            "text" => NodeKind::kSvgText,
            "textPath" if element.svg_text_path.is_some() => NodeKind::kSvgTextPath,
            "tspan" => NodeKind::kSvgTSpan,
            "a" if {
                let mut ancestor = node.Parent();
                let mut inline = false;
                while let Some(p) = ancestor {
                    let a = document.Node(p);
                    if a.Namespace() == DOMNamespace::kSVG && a.Name() == "text" {
                        inline = true;
                        break;
                    }
                    ancestor = a.Parent();
                }
                inline
            } =>
            {
                NodeKind::kSvgInline
            }
            _ if element.svg_shape.is_some() => NodeKind::kSvgShape,
            _ => NodeKind::kSvgGroup,
        },
        DOMNamespace::kHTML => element_kind_html(node, element, style),
        DOMNamespace::kMathML => element_kind_mathml(node),
        DOMNamespace::kNone => NodeKind::kBox,
    }
}
// cpp: dom_to_layout/layout_object_tree_builder.cc:302-314
fn AppendDOMText(document: &Document, index: usize, text: &mut String) {
    for &child in document.Node(index).Children() {
        if document.Node(child).Type() == DOMNodeType::kText {
            text.push_str(document.Node(child).Data());
        } else {
            AppendDOMText(document, child, text);
        }
    }
}
fn CollectOptions(document: &Document, index: usize, options: &mut Vec<usize>) {
    if document.Node(index).IsHTMLElement("option") {
        options.push(index);
    }
    for &child in document.Node(index).Children() {
        CollectOptions(document, child, options);
    }
}
// cpp: dom_to_layout/layout_object_tree_builder.cc:316-356
fn SelectDisplayedText(document: &Document, index: usize) -> String {
    let mut options = Vec::new();
    CollectOptions(document, index, &mut options);
    let value = document.ControlValue(index);
    let trim = |value: &str| {
        value
            .trim_matches(|c| matches!(c, ' ' | '\t' | '\r' | '\n' | '\x0c'))
            .to_owned()
    };
    let selected = options
        .iter()
        .copied()
        .find(|&i| {
            let option = document.Node(i);
            let option_value = attribute(option, "value")
                .map(str::to_owned)
                .unwrap_or_else(|| {
                    let mut text = String::new();
                    AppendDOMText(document, i, &mut text);
                    trim(&text)
                });
            option_value == value
        })
        .or_else(|| {
            options
                .iter()
                .copied()
                .find(|&i| attribute(document.Node(i), "selected").is_some())
        })
        .or_else(|| options.first().copied());
    let Some(selected) = selected else {
        return String::new();
    };
    attribute(document.Node(selected), "label")
        .map(str::to_owned)
        .unwrap_or_else(|| {
            let mut text = String::new();
            AppendDOMText(document, selected, &mut text);
            trim(&text)
        })
}
// cpp: dom_to_layout/layout_object_tree_builder.cc:806-851
fn DefaultListMarkerText(document: &Document, index: usize, style: &ComputedStyle) -> String {
    let defaults = ExtendedStyle::default();
    let extra = style.extended.as_ref().unwrap_or(&defaults);
    match extra.list_style_type {
        ListStyleType::kNone => return String::new(),
        ListStyleType::kCircle => return "○ ".into(),
        ListStyleType::kSquare => return "■ ".into(),
        ListStyleType::kDisc => return "• ".into(),
        ListStyleType::kDecimal | ListStyleType::kLowerAlpha => {}
    }
    let Some(parent) = document
        .Node(index)
        .Parent()
        .filter(|&p| document.Node(p).IsHTMLElement("ol"))
    else {
        return "1. ".into();
    };
    let list = document.Node(parent);
    let reversed = attribute(list, "reversed").is_some();
    let mut value = attribute_integer(list, "start").unwrap_or_else(|| {
        if reversed {
            list.Children()
                .iter()
                .filter(|&&i| document.Node(i).IsHTMLElement("li"))
                .count() as i32
        } else {
            1
        }
    });
    let step = if reversed { -1 } else { 1 };
    for &child in list.Children() {
        let node = document.Node(child);
        if !node.IsHTMLElement("li") {
            continue;
        }
        if let Some(explicit) = attribute_integer(node, "value") {
            value = explicit;
        }
        if child == index {
            if extra.list_style_type == ListStyleType::kLowerAlpha {
                let mut marker = String::new();
                let mut current = value;
                while current > 0 {
                    current -= 1;
                    marker.insert(0, (b'a' + (current % 26) as u8) as char);
                    current /= 26;
                }
                if !marker.is_empty() {
                    return format!("{marker}. ");
                }
            }
            return format!("{value}. ");
        }
        value = value.wrapping_add(step);
    }
    format!("{value}. ")
}
// cpp: dom_to_layout/layout_object_tree_builder.cc:870-902
fn SuppressesCollapsibleWhitespaceChildren(
    document: &Document,
    index: usize,
    tree: &LayoutTreeUpdate<'_>,
    parent: &LayoutObject,
) -> bool {
    let mut node = Some(index);
    while let Some(i) = node {
        if document.Node(i).Type() == DOMNodeType::kElement {
            if let Some(resolved) = document.ResolvedStyleFor(i).filter(|r| r.generates_box) {
                if matches!(
                    resolved.style.display,
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
                return !tree.NeedsCollapsibleWhitespace(parent);
            }
        }
        node = document.Node(i).Parent();
    }
    false
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:958-1015
struct FirstLetterCandidate<'a> {
    text_node: Option<usize>,
    generated: Option<&'a PseudoElement>,
    source_style: &'a ComputedStyle,
    prefix: String,
    remainder: String,
}
struct FirstLetterSearch<'a> {
    punctuation: FirstLetterPunctuationState,
    leading: Option<FirstLetterCandidate<'a>>,
    result: Option<FirstLetterCandidate<'a>>,
    stopped: bool,
}
fn ConsiderFirstLetterText<'a>(
    text: &str,
    style: &'a ComputedStyle,
    node: Option<usize>,
    generated: Option<&'a PseudoElement>,
    search: &mut FirstLetterSearch<'a>,
) {
    if search.stopped {
        return;
    }
    let native = foundation::String::FromUtf8(text.as_bytes());
    let length = first_letter_length(
        &foundation::StringView::from(&native),
        preserves_breaks(style),
        &mut search.punctuation,
    );
    if length == 0 {
        if search.punctuation == FirstLetterPunctuationState::Disallow {
            search.stopped = true;
        }
        return;
    }
    let candidate = FirstLetterCandidate {
        text_node: node,
        generated,
        source_style: style,
        prefix: foundation::String::from_utf16(&native.Span16().unwrap()[..length as usize]).Utf8(),
        remainder: foundation::String::from_utf16(&native.Span16().unwrap()[length as usize..])
            .Utf8(),
    };
    if search.punctuation == FirstLetterPunctuationState::Seen {
        if search.leading.is_none() {
            search.leading = Some(candidate);
        }
        return;
    }
    search.result = search.leading.take().or(Some(candidate));
    search.stopped = true;
}
// cpp: dom_to_layout/layout_object_tree_builder.cc:1029-1118
fn ConsiderGeneratedFirstLetter<'a>(
    pseudo: Option<&'a PseudoElement>,
    search: &mut FirstLetterSearch<'a>,
) {
    let Some(p) = pseudo else {
        return;
    };
    if search.stopped {
        return;
    }
    if p.style.floating != FloatSide::kNone
        || matches!(p.style.position, Position::kAbsolute | Position::kFixed)
    {
        return;
    }
    let block = is_block_container_display(p.style.display);
    if p.style.display != Display::kInline && !p.display_contents && !block {
        search.stopped = true;
        return;
    }
    if !p.text.is_empty() {
        ConsiderFirstLetterText(&p.text, &p.style, None, Some(p), search);
    }
    if block && !search.stopped {
        search.stopped = true;
    }
}
fn IsAtomicFirstLetterBoundary(document: &Document, index: usize, style: &ComputedStyle) -> bool {
    let element = ElementMetadata(document, index, &UserInteractionState::default());
    let mut adjusted = style.clone();
    apply_control_appearance(&element, &mut adjusted);
    matches!(
        ElementKind(document, index, &element, &adjusted),
        NodeKind::kLineBreak
            | NodeKind::kWordBreak
            | NodeKind::kReplaced
            | NodeKind::kFormControl
            | NodeKind::kFrame
            | NodeKind::kSvgRoot
    )
}
fn FindFirstLetterContent<'a>(
    document: &'a Document,
    index: usize,
    style: &'a ComputedStyle,
    search: &mut FirstLetterSearch<'a>,
) {
    let resolved = document.ResolvedStyleFor(index);
    ConsiderGeneratedFirstLetter(resolved.and_then(|r| r.before.as_ref()), search);
    for &child in document.Node(index).Children() {
        if search.stopped {
            return;
        }
        let node = document.Node(child);
        if node.Type() == DOMNodeType::kText {
            ConsiderFirstLetterText(node.Data(), style, Some(child), None, search);
            continue;
        }
        if node.Type() != DOMNodeType::kElement {
            continue;
        }
        let Some(r) = document.ResolvedStyleFor(child) else {
            continue;
        };
        if !r.generates_box {
            if r.display_contents {
                FindFirstLetterContent(document, child, &r.style, search);
            }
            continue;
        }
        if r.style.floating != FloatSide::kNone
            || matches!(r.style.position, Position::kAbsolute | Position::kFixed)
        {
            continue;
        }
        if IsAtomicFirstLetterBoundary(document, child, &r.style) {
            search.stopped = true;
            return;
        }
        if r.style.display == Display::kInline {
            FindFirstLetterContent(document, child, &r.style, search);
        } else if is_block_container_display(r.style.display) {
            if r.first_letter.is_some() {
                search.stopped = true;
                return;
            }
            FindFirstLetterContent(document, child, &r.style, search);
            search.stopped = true;
            return;
        } else {
            search.stopped = true;
            return;
        }
    }
    if !search.stopped {
        ConsiderGeneratedFirstLetter(resolved.and_then(|r| r.after.as_ref()), search);
    }
}
// cpp: dom_to_layout/layout_object_tree_builder.cc:1120-1137
fn FindFirstLetterCandidate<'a>(
    document: &'a Document,
    index: usize,
    style: &'a ComputedStyle,
) -> Option<FirstLetterCandidate<'a>> {
    let mut search = FirstLetterSearch {
        punctuation: FirstLetterPunctuationState::NotSeen,
        leading: None,
        result: None,
        stopped: false,
    };
    FindFirstLetterContent(document, index, style, &mut search);
    search.result
}
struct FirstLetterInsertion<'a> {
    originating_id: u64,
    pseudo: &'a PseudoElement,
    candidate: FirstLetterCandidate<'a>,
    consumed: bool,
}
// cpp: dom_to_layout/layout_object_tree_builder.cc:1139-1165
fn AppendInitialLetter(
    tree: &mut LayoutTreeUpdate<'_>,
    parent: *mut LayoutObject,
    insertion: &mut FirstLetterInsertion<'_>,
    remaining_id: u64,
    name: &str,
    generated: bool,
) {
    let data = ElementData {
        first_letter_pseudo: true,
        ..Default::default()
    };
    let letter = tree.AddBox(
        unsafe { &mut *parent },
        insertion.originating_id | (1 << 46),
        &insertion.pseudo.style,
        "::first-letter".into(),
        Some(data),
        NodeKind::kBox,
        true,
        std::ptr::null(),
    ) as *mut LayoutObject;
    tree.AddText(
        unsafe { &mut *letter },
        insertion.originating_id | (1 << 45),
        insertion.candidate.prefix.clone(),
        "::first-letter-text".into(),
        std::ptr::null(),
        true,
        std::ptr::null(),
    );
    tree.AddText(
        unsafe { &mut *parent },
        remaining_id,
        insertion.candidate.remainder.clone(),
        name.into(),
        insertion.candidate.source_style,
        generated,
        std::ptr::null(),
    );
    insertion.consumed = true;
}
// cpp: dom_to_layout/layout_object_tree_builder.cc:1167-1206
fn AppendPseudoElement(
    tree: &mut LayoutTreeUpdate<'_>,
    id: u64,
    parent: *mut LayoutObject,
    pseudo: Option<&PseudoElement>,
    before: bool,
    mut first: Option<&mut FirstLetterInsertion<'_>>,
) {
    let Some(p) = pseudo else {
        return;
    };
    let box_bit = 1u64 << if before { 50 } else { 48 };
    let text_bit = 1u64 << if before { 49 } else { 47 };
    let name = if before { "::before" } else { "::after" };
    let object = if p.display_contents {
        parent
    } else {
        tree.AddBox(
            unsafe { &mut *parent },
            id | box_bit,
            &p.style,
            name.into(),
            None,
            NodeKind::kBox,
            true,
            std::ptr::null(),
        ) as *mut LayoutObject
    };
    if !p.text.is_empty() {
        let first = first.as_deref_mut().filter(|f| {
            !f.consumed
                && f.candidate
                    .generated
                    .is_some_and(|candidate| std::ptr::eq(candidate, p))
        });
        if let Some(first) = first {
            AppendInitialLetter(
                tree,
                object,
                first,
                id | text_bit,
                &format!("{name}-text"),
                true,
            );
        } else {
            tree.AddText(
                unsafe { &mut *object },
                id | text_bit,
                p.text.clone(),
                format!("{name}-text"),
                if p.display_contents {
                    &p.style
                } else {
                    std::ptr::null()
                },
                true,
                std::ptr::null(),
            );
        }
    }
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:1208-1485
fn AppendChildren<'a>(
    tree: &mut LayoutTreeUpdate<'_>,
    document: &'a Document,
    interaction: &UserInteractionState,
    index: usize,
    parent: *mut LayoutObject,
    flattened: Option<&'a ComputedStyle>,
    mut first: Option<&mut FirstLetterInsertion<'a>>,
) {
    let dom_parent = document.Node(index);
    for &child in dom_parent.Children() {
        let node = document.Node(child);
        if node.Type() == DOMNodeType::kText {
            // SVG character data creates layout text only in SVG text-content
            // elements. Text under <g>, <defs>, shapes, etc. stays in the DOM
            // but is not a child of SVGContentContainer in Chromium.
            let svg_text_parent = dom_parent.Namespace() == DOMNamespace::kSVG
                && matches!(dom_parent.Name(), "text" | "tspan" | "textPath" | "a");
            if dom_parent.Namespace() == DOMNamespace::kSVG && !svg_text_parent {
                continue;
            }
            let style = flattened.or_else(|| document.ResolvedStyleFor(index).map(|r| &r.style));
            if style.is_some_and(|s| !preserves_breaks(s))
                && contains_only_collapsible_whitespace(node.Data())
                && SuppressesCollapsibleWhitespaceChildren(document, index, tree, unsafe {
                    &*parent
                })
            {
                continue;
            }
            if let Some(letter) = first
                .as_deref_mut()
                .filter(|f| !f.consumed && f.candidate.text_node == Some(child))
            {
                AppendInitialLetter(tree, parent, letter, node.Id(), "#text", false);
            } else if !node.Data().is_empty() {
                tree.AddText(
                    unsafe { &mut *parent },
                    node.Id(),
                    node.Data().into(),
                    "#text".into(),
                    style.map_or(std::ptr::null(), |s| s),
                    false,
                    std::ptr::null(),
                );
            }
            continue;
        }
        if node.Type() != DOMNodeType::kElement {
            continue;
        }
        if node.Namespace() == DOMNamespace::kSVG
            && matches!(node.Name(), "title" | "desc" | "metadata")
        {
            continue;
        }
        let resolved = document
            .ResolvedStyleFor(child)
            .expect("ResolveComputedStyles must run before tree build");
        if !resolved.generates_box {
            if resolved.display_contents {
                AppendPseudoElement(
                    tree,
                    node.Id(),
                    parent,
                    resolved.before.as_ref(),
                    true,
                    first.as_deref_mut(),
                );
                AppendChildren(
                    tree,
                    document,
                    interaction,
                    child,
                    parent,
                    Some(&resolved.style),
                    first.as_deref_mut(),
                );
                AppendPseudoElement(
                    tree,
                    node.Id(),
                    parent,
                    resolved.after.as_ref(),
                    false,
                    first.as_deref_mut(),
                );
            }
            continue;
        }
        let element = ElementMetadata(document, child, interaction);
        // Most elements project their resolved style without modification.
        // Own a copy only when resource/control adapters actually need it.
        let mut style = std::borrow::Cow::Borrowed(&resolved.style);
        if node.Name() == "img" {
            if let Some(image) = attribute(node, "src").and_then(|s| document.ImageResourceFor(s)) {
                style.to_mut().paint.image_resource_id = Some(image.id);
            }
        }
        if element.form_control_type.is_some() {
            apply_control_appearance(&element, style.to_mut());
        }
        let kind = ElementKind(document, child, &element, &style);
        let control = element.form_control_type;
        let range = control == Some(FormControlType::kInputRange);
        let file = control == Some(FormControlType::kInputFile);
        let menu = element.select_uses_menu_list.unwrap_or(false);
        let select_list = matches!(
            control,
            Some(FormControlType::kSelectOne | FormControlType::kSelectMultiple)
        ) && !menu;
        let disabled = element.control_disabled;
        if range && style.height.is_none() {
            style.to_mut().height = Some(16.0);
        }
        if select_list && style.height.is_none() {
            let rows = positive_attribute_integer(node, "size", 4);
            let defaults = ExtendedStyle::default();
            let e = style.extended.as_ref().unwrap_or(&defaults);
            let height = rows as f64 * e.line_height.unwrap_or(e.font_size * 1.2) + 2.0;
            style.to_mut().height = Some(height);
        }
        let object = if matches!(kind, NodeKind::kReplaced | NodeKind::kFrame) {
            tree.AddReplaced(
                unsafe { &mut *parent },
                node.Id(),
                &style,
                Some(element),
                node.Name().into(),
                kind,
            )
        } else {
            tree.AddBox(
                unsafe { &mut *parent },
                node.Id(),
                &style,
                node.Name().into(),
                Some(element),
                kind,
                false,
                std::ptr::null(),
            )
        } as *mut LayoutObject;
        if range {
            let mut track_style = ComputedStyle::default();
            track_style.display = Display::kFlex;
            track_style.height = Some(16.0);
            track_style
                .extended
                .get_or_insert_with(Default::default)
                .width_percent = Some(100.0);
            let track = tree.AddBox(
                unsafe { &mut *object },
                node.Id() | (1 << 61),
                &track_style,
                "::-webkit-slider-container".into(),
                Some(ElementData {
                    control_host_ancestor: Some(1),
                    ..Default::default()
                }),
                NodeKind::kBox,
                false,
                std::ptr::null(),
            ) as *mut LayoutObject;
            let mut thumb_style = ComputedStyle::default();
            thumb_style.width = Some(16.0);
            thumb_style.height = Some(16.0);
            thumb_style
                .extended
                .get_or_insert_with(Default::default)
                .effective_appearance = AppearanceValue::kSliderThumbHorizontal;
            tree.AddBox(
                unsafe { &mut *track },
                node.Id() | (1 << 60),
                &thumb_style,
                "::-webkit-slider-thumb".into(),
                None,
                NodeKind::kSliderThumb,
                false,
                std::ptr::null(),
            );
        }
        if file {
            let mut button_style = ComputedStyle::default();
            button_style.display = Display::kInlineBlock;
            button_style.border = Edges {
                top: 2.0,
                right: 2.0,
                bottom: 2.0,
                left: 2.0,
            };
            button_style.padding = Edges {
                top: 1.0,
                right: 6.0,
                bottom: 1.0,
                left: 6.0,
            };
            button_style
                .extended
                .get_or_insert_with(Default::default)
                .effective_appearance = AppearanceValue::kButton;
            let button = tree.AddBox(
                unsafe { &mut *object },
                node.Id() | (1 << 59),
                &button_style,
                "::-webkit-file-upload-button".into(),
                Some(ElementData {
                    form_control_type: Some(FormControlType::kInputButton),
                    control_host_ancestor: Some(1),
                    control_disabled: disabled,
                    file_upload_button: true,
                    ..Default::default()
                }),
                NodeKind::kFormControl,
                false,
                std::ptr::null(),
            ) as *mut LayoutObject;
            tree.AddText(
                unsafe { &mut *button },
                node.Id() | (1 << 58),
                if attribute(node, "multiple").is_some() {
                    "Choose files"
                } else {
                    "Choose file"
                }
                .into(),
                "::file-upload-label".into(),
                std::ptr::null(),
                false,
                std::ptr::null(),
            );
            let mut status_style = ComputedStyle::default();
            status_style.display = Display::kInline;
            let status = tree.AddBox(
                unsafe { &mut *object },
                node.Id() | (1 << 57),
                &status_style,
                "::file-status".into(),
                None,
                NodeKind::kBox,
                false,
                std::ptr::null(),
            ) as *mut LayoutObject;
            tree.AddText(
                unsafe { &mut *status },
                node.Id() | (1 << 56),
                "No file selected".into(),
                "::file-status-label".into(),
                std::ptr::null(),
                false,
                std::ptr::null(),
            );
        }
        if matches!(
            control,
            Some(
                FormControlType::kInputButton
                    | FormControlType::kInputSubmit
                    | FormControlType::kInputReset
            )
        ) {
            let label = attribute(node, "value").unwrap_or(
                if control == Some(FormControlType::kInputSubmit) {
                    "Submit"
                } else if control == Some(FormControlType::kInputReset) {
                    "Reset"
                } else {
                    ""
                },
            );
            if !label.is_empty() {
                tree.AddText(
                    unsafe { &mut *object },
                    node.Id() | (1 << 55),
                    label.into(),
                    "::button-label".into(),
                    std::ptr::null(),
                    false,
                    std::ptr::null(),
                );
            }
        }
        if menu {
            let mut inner_style = list_marker_style(&style);
            inner_style.display = Display::kBlock;
            inner_style.flex_grow = 1.0;
            inner_style.flex_shrink = 1.0;
            inner_style.min_width = Some(0.0);
            inner_style
                .extended
                .get_or_insert_with(Default::default)
                .SetOverflow(Overflow::kHidden);
            let inner = tree.AddBox(
                unsafe { &mut *object },
                node.Id() | (1 << 52),
                &inner_style,
                "::-internal-select-inner-element".into(),
                Some(ElementData {
                    control_host_ancestor: Some(1),
                    ..Default::default()
                }),
                NodeKind::kBox,
                false,
                std::ptr::null(),
            ) as *mut LayoutObject;
            tree.AddText(
                unsafe { &mut *inner },
                node.Id() | (1 << 51),
                SelectDisplayedText(document, child),
                "::select-value".into(),
                std::ptr::null(),
                false,
                std::ptr::null(),
            );
        }
        let textarea = control == Some(FormControlType::kTextArea);
        if control.is_some_and(is_single_line_text_control) || textarea {
            let mut value = document.ControlValue(child);
            let mut placeholder = false;
            if value.is_empty() {
                if let Some(p) = attribute(node, "placeholder") {
                    value = p.into();
                    placeholder = true;
                }
            }
            let mut editor_style = if placeholder && resolved.placeholder.is_some() {
                resolved.placeholder.as_ref().unwrap().style.clone()
            } else {
                list_marker_style(&style)
            };
            editor_style.display = Display::kBlock;
            let e = editor_style.extended.get_or_insert_with(Default::default);
            e.width_percent = Some(100.0);
            if textarea {
                e.white_space = WhiteSpace::kPreWrap;
                e.overflow_wrap = OverflowWrap::kBreakWord;
            }
            let editor = tree.AddBox(
                unsafe { &mut *object },
                node.Id() | (1 << 54),
                &editor_style,
                "::-webkit-textfield-editor".into(),
                Some(ElementData {
                    control_host_ancestor: Some(1),
                    text_control_inner_editor: true,
                    ..Default::default()
                }),
                NodeKind::kBox,
                false,
                std::ptr::null(),
            ) as *mut LayoutObject;
            let trailing_newline = textarea && !placeholder && value.ends_with('\n');
            if !value.is_empty() {
                tree.AddText(
                    unsafe { &mut *editor },
                    node.Id() | (1 << 53),
                    value,
                    if placeholder {
                        "::placeholder"
                    } else {
                        "::text-control-value"
                    }
                    .into(),
                    std::ptr::null(),
                    false,
                    std::ptr::null(),
                );
            }
            // TextControlElement::AdjustPlaceholderBreakElement keeps a caret
            // line after the final newline without changing the control value.
            if trailing_newline {
                let mut break_style = editor_style.clone();
                break_style.display = Display::kInline;
                break_style.extended.as_mut().unwrap().width_percent = None;
                tree.AddBox(
                    unsafe { &mut *editor },
                    node.Id() | (1 << 50),
                    &break_style,
                    "::text-control-placeholder-break".into(),
                    None,
                    NodeKind::kLineBreak,
                    false,
                    std::ptr::null(),
                );
            }
        }
        let marker_text = DefaultListMarkerText(document, child, &style);
        if style.display == Display::kListItem && !marker_text.is_empty() {
            let marker = tree.AddBox(
                unsafe { &mut *object },
                node.Id() | (1 << 63),
                &list_marker_style(&style),
                "::marker".into(),
                None,
                NodeKind::kListMarker,
                true,
                std::ptr::null(),
            ) as *mut LayoutObject;
            let marker_type = style
                .extended
                .as_ref()
                .map_or(ListStyleType::kDisc, |e| e.list_style_type);
            let symbol = matches!(
                marker_type,
                ListStyleType::kDisc | ListStyleType::kCircle | ListStyleType::kSquare
            );
            if symbol && marker_text.ends_with(' ') {
                tree.AddText(
                    unsafe { &mut *marker },
                    node.Id() | (1 << 62),
                    marker_text[..marker_text.len() - 1].into(),
                    "::marker-symbol".into(),
                    std::ptr::null(),
                    true,
                    std::ptr::null(),
                );
                let mut suffix = list_marker_style(&style);
                suffix.width = Some(12.0);
                suffix.height = Some(0.0);
                tree.AddBox(
                    unsafe { &mut *marker },
                    node.Id() | (1 << 61),
                    &suffix,
                    "::marker-suffix".into(),
                    None,
                    NodeKind::kBox,
                    true,
                    std::ptr::null(),
                );
            } else {
                tree.AddText(
                    unsafe { &mut *marker },
                    node.Id() | (1 << 62),
                    marker_text,
                    "::marker-text".into(),
                    std::ptr::null(),
                    true,
                    std::ptr::null(),
                );
            }
        }
        if !matches!(kind, NodeKind::kReplaced | NodeKind::kFrame) && !textarea && !menu && !range {
            let mut own_first = if is_block_container_display(style.display) {
                resolved.first_letter.as_ref().and_then(|pseudo| {
                    FindFirstLetterCandidate(document, child, &style).map(|candidate| {
                        FirstLetterInsertion {
                            originating_id: node.Id(),
                            pseudo,
                            candidate,
                            consumed: false,
                        }
                    })
                })
            } else {
                None
            };
            if let Some(own) = own_first.as_mut() {
                AppendPseudoElement(
                    tree,
                    node.Id(),
                    object,
                    resolved.before.as_ref(),
                    true,
                    Some(own),
                );
                AppendChildren(tree, document, interaction, child, object, None, Some(own));
                AppendPseudoElement(
                    tree,
                    node.Id(),
                    object,
                    resolved.after.as_ref(),
                    false,
                    Some(own),
                );
            } else {
                AppendPseudoElement(
                    tree,
                    node.Id(),
                    object,
                    resolved.before.as_ref(),
                    true,
                    first.as_deref_mut(),
                );
                AppendChildren(
                    tree,
                    document,
                    interaction,
                    child,
                    object,
                    None,
                    first.as_deref_mut(),
                );
                AppendPseudoElement(
                    tree,
                    node.Id(),
                    object,
                    resolved.after.as_ref(),
                    false,
                    first.as_deref_mut(),
                );
            }
        }
    }
}

// cpp: dom_to_layout/layout_object_tree_builder.h:23-26
// cpp: dom_to_layout/layout_object_tree_builder.cc:1487-1539
pub fn EmitLayoutMutations(
    owner: &mut DOM,
    interaction: &UserInteractionState,
    mut receive: impl FnMut(LayoutMutation<'_>),
) {
    let _heap_scope = foundation::LayoutHeapScope::new();
    let mut projection = |mut tree: &mut LayoutTreeUpdate<'_>| {
        let document = owner.GetDocument();
        let index = document
            .Node(document.Root())
            .Children()
            .iter()
            .copied()
            .find(|&i| document.Node(i).Type() == DOMNodeType::kElement)
            .expect("document has no root element");
        let node = document.Node(index);
        let resolved = document
            .ResolvedStyleFor(index)
            .expect("ResolveComputedStyles must run before tree build");
        assert!(resolved.generates_box, "document root has display:none");
        let root = tree.CreateRoot(
            node.Id(),
            &resolved.style,
            node.Name().into(),
            Some(ElementMetadata(document, index, interaction)),
            std::ptr::null(),
        ) as *mut LayoutObject;
        let mut first = resolved.first_letter.as_ref().and_then(|pseudo| {
            FindFirstLetterCandidate(document, index, &resolved.style).map(|candidate| {
                FirstLetterInsertion {
                    originating_id: node.Id(),
                    pseudo,
                    candidate,
                    consumed: false,
                }
            })
        });
        AppendPseudoElement(
            &mut tree,
            node.Id(),
            root,
            resolved.before.as_ref(),
            true,
            first.as_mut(),
        );
        AppendChildren(
            &mut tree,
            document,
            interaction,
            index,
            root,
            None,
            first.as_mut(),
        );
        AppendPseudoElement(
            &mut tree,
            node.Id(),
            root,
            resolved.after.as_ref(),
            false,
            first.as_mut(),
        );
    };
    // The projection borrows DOM style/metadata only during this callback.
    // The receiver must synchronously apply it to its owned native tree.
    receive(LayoutMutation::TreeUpdate(&mut projection));
}

/// Fixture adapter using the same mutation emission as the Page lifecycle.
/// The engine owns the native tree; DOM only supplies its projection.
#[cfg(test)]
pub fn BuildLayoutObjectTree(
    owner: &mut DOM,
    interaction: &UserInteractionState,
    engine: &mut LayoutEngine,
) -> *mut LayoutObject {
    EmitLayoutMutations(owner, interaction, |mutation| {
        engine.ApplyMutation(mutation);
    });
    engine
        .GetLayoutTree()
        .expect("DOM projection installed a layout tree")
        .Root() as *const LayoutObject as *mut LayoutObject
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    trait Numeric {
        fn number(self) -> f64;
    }
    macro_rules! numeric {($($t:ty),*)=>{$(impl Numeric for $t {fn number(self)->f64 {self as f64}})*};}
    numeric!(f64, u32, usize, i32, i8);
    impl Numeric for bool {
        fn number(self) -> f64 {
            self as u8 as f64
        }
    }
    impl Numeric for FormControlType {
        fn number(self) -> f64 {
            self as i32 as f64
        }
    }
    fn dump(
        document: &Document,
        index: usize,
        interaction: &UserInteractionState,
    ) -> Vec<Option<f64>> {
        let d = ElementMetadata(document, index, interaction);
        let fallback = ComputedStyle::default();
        let s = document
            .ResolvedStyleFor(index)
            .map_or(&fallback, |r| &r.style);
        let mut values = Vec::new();
        macro_rules! n {
            ($value:expr) => {
                values.push(Some(Numeric::number($value)))
            };
        }
        macro_rules! optional {
            ($value:expr) => {
                values.push($value.map(Numeric::number))
            };
        }
        n!(ElementKind(document, index, &d, s) as i32);
        n!(d.scroll_offset.x);
        n!(d.scroll_offset.y);
        n!(d.nowrap_attribute);
        n!(d.html_image);
        n!(d.html_ordered_or_unordered_list);
        n!(d.viewport_defining);
        n!(d.document_role as i32);
        optional!(d.natural_width);
        optional!(d.natural_height);
        optional!(d.natural_aspect_ratio);
        n!(d.image_device_pixel_ratio);
        n!(d.row_span);
        n!(d.column_span);
        optional!(d.form_control_type);
        n!(d.control_checked);
        n!(d.control_disabled);
        n!(d.control_hovered);
        n!(d.control_active);
        n!(d.control_focused);
        n!(d.control_read_only);
        n!(d.marquee_horizontal);
        optional!(d.frame_border);
        n!(d.frame_no_resize);
        n!(d.frame_has_border_color);
        optional!(d.frame_border_thickness);
        optional!(d.select_uses_menu_list);
        optional!(d.range_value_ratio);
        optional!(d.control_value_ratio);
        n!(d.math_token_kind as i32);
        n!(d.text_area_sizing.is_some());
        if let Some(v) = d.text_area_sizing {
            n!(v.columns);
            n!(v.rows);
        }
        n!(d.text_field_sizing.is_some());
        if let Some(v) = d.text_field_sizing {
            n!(v.preferred_size);
            n!(v.includes_decoration);
        }
        for dimensions in [&d.frame_rows, &d.frame_columns] {
            n!(dimensions.len());
            for v in dimensions {
                n!(v.r#type as i32);
                n!(v.value);
            }
        }
        n!(d.svg_view_box.is_some());
        if let Some(v) = &d.svg_view_box {
            for x in [v.x, v.y, v.width, v.height] {
                n!(x);
            }
            n!(v.align_x);
            n!(v.align_y);
            n!(v.preserve_none);
            n!(v.slice);
        }
        n!(d.svg_shape.is_some());
        if let Some(v) = &d.svg_shape {
            n!(v.geometry as i32);
            n!(v.bounds_offset.x);
            n!(v.bounds_offset.y);
            n!(v.bounds_width);
            n!(v.bounds_height);
            n!(v.rectangle_radius_x);
            n!(v.rectangle_radius_y);
            n!(v.path.len());
            for p in &v.path {
                n!(p.verb as i32);
                for point in [p.point, p.control1, p.control2] {
                    n!(point.x);
                    n!(point.y);
                }
            }
        }
        n!(d.svg_text_path.is_some());
        if let Some(v) = &d.svg_text_path {
            n!(v.start_offset);
            n!(v.reverse_direction);
            n!(v.points.len());
            for p in &v.points {
                n!(p.x);
                n!(p.y);
            }
        }
        for v in [&d.svg_x, &d.svg_y, &d.svg_dx, &d.svg_dy, &d.svg_rotate] {
            n!(v.len());
            for x in v {
                n!(*x);
            }
        }
        optional!(d.svg_text_length);
        n!(d.svg_length_adjust as i32);
        values
    }
    #[test]
    fn svg_root_absolute_dimensions_are_natural_dimensions_not_viewbox_dimensions() {
        let owner = crate::test_html::html_parser::ParseHTML(
            "<html><body><svg id='absolute' width='50' height='58' viewBox='0 0 100 100'></svg><svg id='percent' width='100%' height='50%' viewBox='0 0 20 10'></svg></body></html>",
        );
        let document = owner.GetDocument();
        let find = |id| {
            (0..document.NodeCount())
                .find(|index| {
                    document
                        .Node(*index)
                        .FindAttribute("id")
                        .is_some_and(|attribute| attribute.value == id)
                })
                .unwrap()
        };
        let interaction = UserInteractionState::default();
        let absolute = ElementMetadata(document, find("absolute"), &interaction);
        assert_eq!(absolute.natural_width, Some(50.0));
        assert_eq!(absolute.natural_height, Some(58.0));
        assert_eq!(absolute.natural_aspect_ratio, Some(50.0 / 58.0));
        let percent = ElementMetadata(document, find("percent"), &interaction);
        assert_eq!(percent.natural_width, None);
        assert_eq!(percent.natural_height, None);
        assert_eq!(percent.natural_aspect_ratio, Some(2.0));
    }
    #[test]
    fn emitted_dom_projection_updates_only_when_received_and_reuses_siblings() {
        fn native(root: *mut LayoutObject, id: u64) -> *mut LayoutObject {
            let mut object = root;
            while !object.is_null() {
                let node = unsafe { &*object }.GetNode();
                if !node.is_null() && unsafe { &*node }.InputId() == id {
                    return object;
                }
                object = unsafe { &*object }.NextInPreOrder(root);
            }
            panic!("missing native node {id}");
        }
        fn width(object: *mut LayoutObject) -> Option<f64> {
            let node = unsafe { &*object }.GetNode();
            unsafe { &*node }.InputStyle().width
        }
        let mut owner = crate::test_html::html_parser::ParseHTML(
            "<html><body><div id='changed' style='display:block;width:80px;height:30px'></div><div id='sibling' style='display:block;width:40px;height:20px'></div></body></html>",
        );
        let environment = crate::style_resolver::StyleEnvironment::default();
        crate::style_resolver::ResolveComputedStyles(&mut owner, &environment, &[]);
        let ids: BTreeMap<_, _> = (0..owner.GetDocument().NodeCount())
            .filter_map(|index| {
                owner
                    .GetDocument()
                    .Node(index)
                    .FindAttribute("id")
                    .map(|attr| (attr.value.clone(), index))
            })
            .collect();
        let changed_id = owner.GetDocument().Node(ids["changed"]).Id();
        let sibling_id = owner.GetDocument().Node(ids["sibling"]).Id();
        let assembly = layoutng_assembly::layout_assembly::LayoutAssembly::default();
        let mut engine = LayoutEngine::new(&assembly);
        let interaction = UserInteractionState::default();
        owner.EmitLayoutMutations(&interaction, |mutation| {
            engine.ApplyMutation(mutation);
        });
        let root =
            engine.GetLayoutTree().unwrap().Root() as *const LayoutObject as *mut LayoutObject;
        let changed = native(root, changed_id);
        let sibling = native(root, sibling_id);
        let sibling_style = unsafe { &*sibling }.StyleRef() as *const _;
        assert_eq!(width(changed), Some(80.0));

        owner
            .ApplyMutation(&crate::dom_mutation::DOMMutation {
                mutation_type: crate::dom_mutation::DOMMutationType::kSetAttribute,
                target_node_id: changed_id,
                name: "style".into(),
                value: "display:block;width:120px;height:30px".into(),
                ..Default::default()
            })
            .unwrap();
        crate::style_resolver::ResolveComputedStyles(&mut owner, &environment, &[]);
        assert_eq!(
            owner
                .GetDocument()
                .ResolvedStyleFor(ids["changed"])
                .unwrap()
                .style
                .width,
            Some(120.0)
        );
        assert_eq!(
            width(changed),
            Some(80.0),
            "DOM/style updates do not mutate the engine tree"
        );
        let mut received = 0;
        owner.EmitLayoutMutations(&interaction, |mutation| {
            received += 1;
            assert!(matches!(&mutation, LayoutMutation::TreeUpdate(_)));
            assert_eq!(
                width(changed),
                Some(80.0),
                "emission has not applied its borrowed projection"
            );
            assert_eq!(native(root, sibling_id), sibling);
            assert!(engine.ApplyMutation(mutation));
        });
        assert_eq!(received, 1);
        let updated_root =
            engine.GetLayoutTree().unwrap().Root() as *const LayoutObject as *mut LayoutObject;
        assert_eq!(updated_root, root);
        assert_eq!(native(updated_root, changed_id), changed);
        assert_eq!(width(changed), Some(120.0));
        assert_eq!(native(updated_root, sibling_id), sibling);
        assert_eq!(unsafe { &*sibling }.StyleRef() as *const _, sibling_style);
        assert_eq!(engine.GetLayoutTree().unwrap().UpdateStats().created, 0);
    }
    #[test]
    fn persistent_metadata_and_namespace_kinds_match_cpp_including_live_values() {
        let mut owner = crate::test_html::html_parser::ParseHTML(include_str!(
            "../../../../artifacts/cpp-reference/persistent-tree-metadata.html"
        ));
        owner
            .GetDocumentMut()
            .AppendStyleSheet(cssom::ParseCSS(include_str!(
                "../../../../artifacts/cpp-reference/persistent-tree.css"
            )));
        let nodes: BTreeMap<_, _> = (0..owner.GetDocument().NodeCount())
            .filter_map(|i| {
                owner
                    .GetDocument()
                    .Node(i)
                    .FindAttribute("id")
                    .map(|a| (a.value.clone(), i))
            })
            .collect();
        owner.GetDocumentMut().SetImageResource(
            "image.png".into(),
            crate::ImageResourceMetadata {
                id: 7,
                natural_width: 32.0,
                natural_height: 24.0,
                resolution_scale: 2.0,
            },
        );
        let interaction = UserInteractionState {
            hovered_node_id: Some(owner.GetDocument().Node(nodes["range"]).Id()),
            pressed_node_id: Some(owner.GetDocument().Node(nodes["check"]).Id()),
            focus_visible_node_id: Some(owner.GetDocument().Node(nodes["input"]).Id()),
            ..Default::default()
        };
        for stage in 0..2 {
            if stage == 1 {
                let d = owner.GetDocumentMut();
                d.SetControlValue(nodes["input"], "live中文".into());
                d.SetControlValue(nodes["range"], "11".into());
                d.SetControlValue(nodes["select"], "one".into());
                d.SetControlChecked(nodes["check"], false);
                d.AppendChild(nodes["contents"], nodes["letter-next"]);
                d.SetScrollOffset(nodes["contents"], Offset { x: 3.0, y: 4.0 });
            }
            crate::style_resolver::ResolveComputedStyles(
                &mut owner,
                &crate::style_resolver::StyleEnvironment {
                    viewport_width: Some(1024.0),
                    viewport_height: Some(768.0),
                    resolution_dppx: Some(1.0),
                    ..Default::default()
                },
                &[],
            );
            let mut count = 0;
            for line in include_str!(
                "../../../../artifacts/cpp-reference/persistent-tree-metadata-results.tsv"
            )
            .lines()
            {
                let mut fields = line.split('\t');
                let key = fields.next().unwrap();
                let (s, id) = key.split_once(':').unwrap();
                if s.parse::<usize>().unwrap() != stage {
                    continue;
                }
                let expected: Vec<Option<f64>> = fields
                    .map(|v| {
                        if v == "none" {
                            None
                        } else {
                            Some(v.parse().unwrap())
                        }
                    })
                    .collect();
                assert_eq!(
                    dump(owner.GetDocument(), nodes[id], &interaction),
                    expected,
                    "stage {stage} node {id}"
                );
                count += 1;
            }
            assert_eq!(count, nodes.len());
        }
    }
}
