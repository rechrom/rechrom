#![allow(non_snake_case)]
use crate::layout_mapping::layout_object_tree_builder::*;
use crate::persistent_document::{
    DOMInputType, DOMNamespace, DOMNodeType, DOMShadowRootMode, PseudoContent, PseudoElement,
};
use crate::svg_path_parser::ParseSVGPathDefault;
use crate::{Document, UserInteractionState, DOM};
use layoutng_assembly::internal::form_control_types::FormControlType;
use layoutng_assembly::internal::layout_input::*;
use layoutng_assembly::internal::layout_object::LayoutObject;
#[cfg(test)]
use layoutng_assembly::layout_engine::LayoutEngine;
use layoutng_assembly::layout_engine::{LayoutMutation, LayoutTreeUpdate};
use layoutng_style::style::appearance::AppearanceValue;
use layoutng_style::style::computed_style::{
    ComputedStyle as StyleComputedStyle, ComputedStyleBuilder,
};
use std::collections::HashSet;

fn CanonicalIsBlockContainer(display: foundation::EDisplay) -> bool {
    matches!(
        display,
        foundation::EDisplay::kBlock
            | foundation::EDisplay::kFlowRoot
            | foundation::EDisplay::kListItem
            | foundation::EDisplay::kInlineListItem
            | foundation::EDisplay::kFlowRootListItem
            | foundation::EDisplay::kInlineFlowRootListItem
            | foundation::EDisplay::kTableCell
            | foundation::EDisplay::kTableCaption
    )
}

fn ResolveImageMapAreaGeometry(
    document: &crate::Document,
    area: usize,
) -> Option<(ImageMapAreaShape, Vec<f64>)> {
    let (shape, coordinates) = document.ImageMapAreaGeometry(area)?;
    Some((shape, coordinates.to_vec()))
}

fn ResolveFocusedImageMapArea(
    document: &crate::Document,
    image: usize,
    interaction_state: &UserInteractionState,
) -> Option<FocusedImageMapArea> {
    let area_id = interaction_state.focused_node_id?;
    if !document.ImageMapAreaHasPotentialPath(area_id) {
        return None;
    }
    if document.ImageMapImageForAreaId(area_id) != Some(document.Node(image).Id()) {
        return None;
    }
    let area = document.FindNodeById(area_id)?;
    let resolved = document.ResolvedStyleFor(area)?;
    let source = unsafe { resolved.computed_style_owner.Get().as_ref() }?;
    let (shape, coordinates) = ResolveImageMapAreaGeometry(document, area)?;
    Some(
        layoutng_assembly::internal::layout_input::FocusedImageMapArea {
            shape,
            coordinates,
            has_outline: source.HasOutline(),
            outline_style_is_auto: source.OutlineStyleIsAuto(),
            outline_color: layoutng_assembly::internal::boundary::paint_style_export::StyleColor(
                source,
                source.OutlineColor(),
            ),
            effective_zoom: source.EffectiveZoom(),
        },
    )
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:376-631
pub(super) fn ElementMetadata(
    document: &crate::Document,
    index: usize,
    interaction_state: &UserInteractionState,
    respect_image_orientation: bool,
) -> ElementData {
    let node = document.Node(index);
    let mut data = ElementData::default();
    data.ultimate_originating_element_id = document
        .PseudoElementFacts(index)
        .map(|facts| document.Node(facts.originating_element).Id());

    data.generated_scroll_marker_group_id = document.GeneratedPseudoElement(index,
        layoutng_style::style::computed_style_constants::PseudoId::kPseudoIdScrollMarkerGroupBefore)
        .or_else(|| document.GeneratedPseudoElement(index,
            layoutng_style::style::computed_style_constants::PseudoId::kPseudoIdScrollMarkerGroupAfter))
        .map(|group| document.Node(group).Id());

    data.element_canvas_transform_enabled = document.CanvasRuntimeFeatures().1;
    data.used_canvas_transform =
        document
            .UsedCanvasTransformForLayout(index)
            .map(|matrix| TransformMatrix {
                values: matrix.GetColMajor(),
            });
    data.content_lock = document.DisplayLockContentState(index);
    data.region_capture_crop_id = node.RegionCaptureCropId();
    data.tracked_element_sub_rects = node.TrackedElementSubRects().to_vec();
    // cpp: dom_to_layout/layout_object_tree_builder.cc:411-426
    data.nowrap_attribute = attribute(node, "nowrap").is_some();
    data.html_anchor = node.Name() == "a";
    data.html_canvas = node.IsHTMLElement("canvas");
    data.html_hr = node.IsHTMLElement("hr");
    data.html_media = document.IsMediaElement(index);
    data.html_video = document.IsVideoElement(index);
    data.video = document.VideoLayoutStateForElement(index);
    data.media_child_kind = document.MediaChildKindForLayout(index);
    data.vtt_cue_box_block_flow = document.VTTCueBoxForcesBlockFlowLayout(index);
    data.media_is_fullscreen = data.html_media && document.FullscreenFlag(index);
    if data.html_video {
        if let Some(frame) = document.VideoFrameMetadataFor(node.Id()) {
            data.video_frame_resource_id = Some(frame.resource_id);
        }
    }
    data.canvas_layout_subtree = data.html_canvas && attribute(node, "layoutsubtree").is_some();
    if data.html_canvas {
        let dimension = |name: &str, fallback: u32| {
            attribute(node, name)
                .and_then(canvas::ParseCanvasDimension)
                .filter(|value| *value <= i32::MAX as u32)
                .unwrap_or(fallback)
        };
        let width = dimension("width", 300);
        let height = dimension("height", 150);
        data.natural_width = Some(f64::from(width));
        data.natural_height = Some(f64::from(height));
        data.natural_aspect_ratio = (height != 0).then_some(f64::from(width) / f64::from(height));
    }
    data.drawable_attribute = attribute(node, "drawable").is_some();
    data.html_image = node.Name() == "img";
    data.html_image_uses_fallback = data.html_image && document.HTMLImageUsesFallbackContent(index);
    data.embedded_content_kind = (node.Namespace() == DOMNamespace::kHTML)
        .then(|| match node.Name() {
            "frame" | "iframe" | "fencedframe" => Some(EmbeddedContentKind::Frame),
            "embed" | "object" => Some(EmbeddedContentKind::Object),
            _ => None,
        })
        .flatten();
    data.image_map_area = node.Name() == "area";
    data.html_ordered_or_unordered_list = node.Name() == "ol" || node.Name() == "ul";
    data.html_list_owner = node.IsHTMLElement("ol")
        || node.IsHTMLElement("ul")
        || node.IsHTMLElement("menu");
    if node.IsHTMLElement("ol") {
        data.ordered_list_reversed = attribute(node, "reversed").is_some();
        data.ordered_list_start = attribute_integer(node, "start");
    }
    if node.IsHTMLElement("li") {
        data.list_item_explicit_value = attribute_integer(node, "value");
    }
    // cpp: core/dom/element.cc:13896-13910
    data.replaced_respects_css_overflow = (node.Namespace() == DOMNamespace::kHTML
        && match node.Name() {
            "img" => !data.html_image_uses_fallback,
            "canvas" | "video" | "frame" | "iframe" | "fencedframe" | "embed" | "object" => true,
            _ => false,
        })
        || (node.Namespace() == DOMNamespace::kSVG
            && node.Name() == "svg"
            && document.CorrespondingSVGElement(index).is_none()
            && node.Parent().is_some_and(|parent| {
                let parent = document.Node(parent);
                parent.Type() != DOMNodeType::kDocument
                    && svg_has_outermost_parent(Some((parent.Namespace(), parent.Name())))
            }));
    if node.Name() == "html" {
        data.viewport_defining = true;
        data.document_role = DocumentRole::kDocumentElement;
    } else if node.Name() == "body" {
        data.document_role = DocumentRole::kBody;
    }
    let primary_image_layout = (node.Name() == "img" && !data.html_image_uses_fallback)
        || (document.InputType(index) == Some(DOMInputType::kImage)
            && !document.ImageInputHasFallbackContent(index));
    if primary_image_layout {
        // ImageElementBase::setDecodingMode: invalid/missing values retain the
        // unspecified preference.
        data.image_decoding_mode = match attribute(node, "decoding").map(lower_ascii).as_deref() {
            Some("sync") => image_resource::ImageDecodingMode::kSync,
            Some("async") => image_resource::ImageDecodingMode::kAsync,
            _ => image_resource::ImageDecodingMode::kUnspecified,
        };
        data.focused_image_map_area =
            ResolveFocusedImageMapArea(document, index, interaction_state);
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
        if let Some(content) = document
            .ImageLoaderLayoutUpdate(index)
            .and_then(|update| update.content)
        {
            data.image_resource_error = content.ErrorOccurred();
            if let Some(image) = content.Metadata() {
                data.image_resource_is_loaded = true;
                data.image_resource_known_to_be_opaque = image.known_to_be_opaque;
                data.image_resource_maybe_animated = document.ImageResourceMaybeAnimated(image.id);
                let candidate_density = document
                    .ResponsiveImageSelectionFor(index)
                    .and_then(|selection| selection.density);
                let (image_width, image_height) =
                    image.NaturalSizeForCandidate(respect_image_orientation, candidate_density);
                // LayoutImage::UpdateNaturalSizeIfNeeded takes natural sizing
                // exclusively from ImageResource once decoded.  HTML width /
                // height attributes are presentational sizing inputs, not a
                // replacement for the resource's intrinsic aspect ratio.
                // Retaining their square ratio here makes object-fit:contain
                // incorrectly behave like fill for a non-square image.
                data.natural_width = Some(image_width);
                data.natural_height = Some(image_height);
                data.natural_aspect_ratio =
                    (image_height > 0.0).then_some(image_width / image_height);
                // cpp: core/layout/layout_image.cc:177-185. Match
                // HTMLImageElement::image_device_pixel_ratio_: this is
                // inverse selected density unless the response's Content-DPR
                // header overrides srcset and every other DPR source.
                data.image_device_pixel_ratio = 1.0
                    / document
                        .ImageResourceContentDPR(image.id)
                        .or(candidate_density)
                        .unwrap_or(image.resolution_scale);
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
        if node.Name() == "filter" {
            // LayoutSVGResourceFilter::FindCycleFromSelf (cc:68-82) visits
            // direct feImage href targets, not a flattened Filter graph.
            for child in node.Children().iter().copied() {
                let image = document.Node(child);
                if !image.IsElement("feImage", DOMNamespace::kSVG) {
                    continue;
                }
                let target = attribute(image, "href")
                    .or_else(|| attribute(image, "xlink:href"))
                    .and_then(|url| url.strip_prefix('#'))
                    .filter(|id| !id.contains('%'))
                    .and_then(|id| document.FindSVGResourceById(child, id));
                if let Some(target) =
                    target.filter(|target| document.Node(*target).Namespace() == DOMNamespace::kSVG)
                {
                    data.svg_filter_image_targets
                        .push(document.Node(target).Id());
                }
            }
        }

        if node.Name() == "use" {
            use crate::persistent_document::SVGUseInstanceStatus as Status;
            data.svg_use = Some(match document.SVGUseInstanceStatus(index) {
                Some(Status::UnsupportedReference) => SvgUseData::UnsupportedReference,
                Some(Status::UnsupportedViewport) => SvgUseData::UnsupportedViewport,
                Some(Status::UnsupportedSwitch) => SvgUseData::UnsupportedSwitch,
                Some(_) => SvgUseData::Local,
                None => SvgUseData::UnbuiltInstance,
            });
        }

        // cpp: SVGSVGElement::{CurrentViewBox,CurrentPreserveAspectRatio,
        // ViewBoxToViewTransform}. SVGImageViewInfo is document-owned and
        // overrides the root's authored values only for the active recording.
        if node.IsElement("svg", DOMNamespace::kSVG) {
            data.svg_current_translate = document.SVGCurrentTranslate(index);
            if let Some(view) = document.SVGImageViewSpecFor(index) {
                if let Some([x, y, width, height]) = view.view_box {
                    let has_intrinsic_dimensions =
                        data.natural_width.is_some() && data.natural_height.is_some();
                    let inherited_preserve = data.svg_view_box.map(|view_box| {
                        (
                            view_box.preserve_none,
                            view_box.slice,
                            view_box.align_x,
                            view_box.align_y,
                        )
                    });
                    data.svg_view_box = Some(SvgViewBoxData {
                        x,
                        y,
                        width,
                        height,
                        preserve_none: inherited_preserve.is_some_and(|preserve| preserve.0),
                        slice: inherited_preserve.is_some_and(|preserve| preserve.1),
                        align_x: inherited_preserve.map_or(0, |preserve| preserve.2),
                        align_y: inherited_preserve.map_or(0, |preserve| preserve.3),
                        ..Default::default()
                    });
                    if data.natural_width.is_none() {
                        data.natural_width = Some(width);
                    }
                    if data.natural_height.is_none() {
                        data.natural_height = Some(height);
                    }
                    if !has_intrinsic_dimensions {
                        data.natural_aspect_ratio = (height > 0.0).then_some(width / height);
                    }
                }
                if data.svg_view_box.is_none() && view.transform.is_some() {
                    if let (Some(width), Some(height)) = (data.natural_width, data.natural_height) {
                        if width > 0.0 && height > 0.0 {
                            data.svg_view_box = Some(SvgViewBoxData {
                                width,
                                height,
                                preserve_none: true,
                                ..Default::default()
                            });
                        }
                    }
                }
                if let (Some(view_box), Some(preserve)) =
                    (data.svg_view_box.as_mut(), view.preserve_aspect_ratio)
                {
                    view_box.preserve_none = preserve.preserve_none;
                    view_box.slice = preserve.slice;
                    view_box.align_x = preserve.align_x;
                    view_box.align_y = preserve.align_y;
                }
                if let (Some(view_box), Some([a, b, c, d, e, f])) =
                    (data.svg_view_box.as_mut(), view.transform)
                {
                    let mut matrix = TransformMatrix::default();
                    matrix.values[0] = a;
                    matrix.values[1] = b;
                    matrix.values[4] = c;
                    matrix.values[5] = d;
                    matrix.values[12] = e;
                    matrix.values[13] = f;
                    view_box.view_transform = Some(matrix);
                }
            }
            if document.CorrespondingSVGElement(index).is_some()
                || !svg_has_outermost_parent(node.Parent().map(|parent| {
                    let parent = document.Node(parent);
                    (parent.Namespace(), parent.Name())
                }))
            {
                element_metadata_svg_viewport(node, &mut data);
            }
        }
    }
    if node.Namespace() == DOMNamespace::kMathML {
        element_metadata_mathml(node, &mut data);
    }
    if node.Namespace() != DOMNamespace::kHTML {
        return data;
    }
    data.control_checked = document.ControlChecked(index);
    data.control_indeterminate = document.ShouldAppearIndeterminate(index);
    data.control_is_progress = node.IsHTMLElement("progress");
    data.control_disabled = attribute(node, "disabled").is_some();
    let node_id = node.Id();
    data.control_hovered = document.IsElementHovered(index);
    data.control_active = document.IsElementActive(index);
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
        data.select_uses_menu_list = Some(document.SelectUsesMenuList(index));
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
        data.input_image_use_fallback = control_type == FormControlType::kInputImage
            && document.ImageInputHasFallbackContent(index);
        if is_single_line_text_control(control_type) {
            data.text_field_sizing = Some(TextFieldSizing {
                preferred_size: positive_attribute_integer(node, "size", 20) as i32,
                includes_decoration: false,
            });
        }
        if control_type == FormControlType::kInputFile {
            data.selected_file_count = document
                .InputFiles(index)
                .expect("file input owns canonical FileList")
                .length();
            data.file_no_file_label = Some(
                document
                    .InputFileNoFileLabel(index)
                    .unwrap_or("No file selected")
                    .to_owned(),
            );
        }
    }
    if data.form_control_type == Some(FormControlType::kInputRange) {
        let ratio = range_value_ratio_from_control_value(node, &document.ControlValue(index));
        data.range_value_ratio = Some(ratio);
        data.control_value_ratio = Some(ratio);
    }
    data
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:260-300
fn ResolveSVGTextPath(document: &Document, index: usize) -> Option<SvgTextPathData> {
    let node = document.Node(index);
    let href = attribute(node, "href")?;
    let id = href.strip_prefix('#').filter(|id| !id.is_empty())?;
    let path = document.Node(document.FindSVGResourceById(index, id)?);
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
    source: Option<&StyleComputedStyle>,
) -> NodeKind {
    let node = document.Node(index);
    match node.Namespace() {
        DOMNamespace::kSVG => match node.Name() {
            "svg" => {
                if document.CorrespondingSVGElement(index).is_some() {
                    NodeKind::kSvgViewportContainer
                } else if svg_has_outermost_parent(node.Parent().map(|parent| {
                    let parent = document.Node(parent);
                    (parent.Namespace(), parent.Name())
                })) {
                    NodeKind::kSvgRoot
                } else {
                    NodeKind::kSvgViewportContainer
                }
            }
            "foreignObject" => NodeKind::kSvgForeignObject,
            // SVGGElement::CreateLayoutObject preserves resources below a
            // display:none group through LayoutSVGHiddenContainer.
            "g" if source.is_some_and(|style| style.Display() == foundation::EDisplay::kNone) => {
                NodeKind::kSvgHiddenContainer
            }
            "defs" => NodeKind::kSvgHiddenContainer,
            "clipPath" => NodeKind::kSvgResourceClipper,
            "mask" => NodeKind::kSvgResourceMasker,
            "linearGradient" | "radialGradient" => NodeKind::kSvgResourceGradient,
            "pattern" => NodeKind::kSvgResourcePattern,
            "filter" => NodeKind::kSvgResourceFilter,
            "marker" => NodeKind::kSvgResourceMarker,
            "text" => NodeKind::kSvgText,
            // SVGTextPathElement::CreateLayoutObject preserves element identity
            // independently of href resolution. LayoutSVGTextPath::LayoutPath
            // represents an unresolved or empty resource with None.
            "textPath" => NodeKind::kSvgTextPath,
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
        DOMNamespace::kHTML => element_kind_html(node, element, style, source),
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
fn DefaultListMarkerText(document: &Document, index: usize, style: &StyleComputedStyle) -> String {
    let style_name = unsafe { style.ListStyleType().Get().as_ref() }
        .filter(|style| style.IsCounterStyle())
        .map(|style| style.GetCounterStyleName().Utf8())
        .unwrap_or_else(|| "none".into());
    match style_name.as_str() {
        "none" => return String::new(),
        "circle" => return "○ ".into(),
        "square" => return "■ ".into(),
        "disc" => return "• ".into(),
        _ => {}
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
            if matches!(style_name.as_str(), "lower-alpha" | "lower-latin") {
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
// Author shadow attachment consumes DOM's single active flat-tree view. UA
// controls still have specialized owners below (source file nodes, synthesized
// editor/slider/menu, and host-painted meter/progress). Preserve that existing
// projection until each adapter migrates together with its source shadow tree;
// entering a control UA root here would duplicate content or source theme painting.
// SVGUseElement owns real SVG descendants and uses ordinary flat-tree attachment.
pub(super) fn AttachmentChildren(
    document: &Document,
    index: usize,
) -> std::borrow::Cow<'_, [usize]> {
    // cpp: html_details_element.cc:80-147. Blink's manually assigned UA
    // shadow slots render the first <summary> before the details-content slot.
    // A closed content slot is display-locked, so its assigned descendants do
    // not enter layout. Keep that attachment behavior at this boundary until
    // the UA slot nodes themselves are projected by the DOM owner.
    if document.Node(index).IsHTMLElement("details") {
        let children = document.FlatTreeChildren(index);
        if let Some((summary_position, summary)) = children
            .iter()
            .copied()
            .enumerate()
            .find(|(_, child)| document.Node(*child).IsHTMLElement("summary"))
        {
            if document.Node(index).FindAttribute("open").is_none() {
                return std::borrow::Cow::Owned(vec![summary]);
            }
            if summary_position != 0 {
                let mut assigned = Vec::with_capacity(children.len());
                assigned.push(summary);
                assigned.extend(children.iter().copied().enumerate().filter_map(
                    |(position, child)| (position != summary_position).then_some(child),
                ));
                return std::borrow::Cow::Owned(assigned);
            }
            return children;
        }
    }
    if !document.Node(index).IsElement("use", DOMNamespace::kSVG)
        && !(document.InputType(index) == Some(DOMInputType::kImage)
            && document.ImageInputHasFallbackContent(index))
        && document.ShadowRoot(index).is_some_and(|root| {
            document.ShadowRootMode(root) == Some(DOMShadowRootMode::kUserAgent)
        })
    {
        return std::borrow::Cow::Borrowed(document.Node(index).Children());
    }
    document.FlatTreeChildren(index)
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
                    unsafe { resolved.computed_style_owner.Get().as_ref() }
                        .map(|style| style.Display()),
                    Some(
                        foundation::EDisplay::kFlex
                            | foundation::EDisplay::kInlineFlex
                            | foundation::EDisplay::kGrid
                            | foundation::EDisplay::kInlineGrid
                            | foundation::EDisplay::kTable
                            | foundation::EDisplay::kInlineTable
                            | foundation::EDisplay::kTableRow
                            | foundation::EDisplay::kTableRowGroup
                            | foundation::EDisplay::kTableHeaderGroup
                            | foundation::EDisplay::kTableFooterGroup
                            | foundation::EDisplay::kTableColumnGroup
                            | foundation::EDisplay::kWebkitBox
                            | foundation::EDisplay::kWebkitInlineBox
                    )
                ) {
                    return true;
                }
                return !tree.NeedsCollapsibleWhitespace(parent);
            }
        }
        node = document.FlatTreeParentElement(i);
    }
    false
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:958-1015
struct FirstLetterCandidate<'a> {
    text_node: Option<usize>,
    generated: Option<&'a PseudoElement>,
    source_style: &'a StyleComputedStyle,
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
    style: &'a StyleComputedStyle,
    preserve_breaks: bool,
    node: Option<usize>,
    generated: Option<&'a PseudoElement>,
    search: &mut FirstLetterSearch<'a>,
) {
    if search.stopped {
        return;
    }
    let source = foundation::String::FromUtf8(text.as_bytes());
    let length = first_letter_length(
        &foundation::StringView::from(&source),
        preserve_breaks,
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
        prefix: foundation::String::from_utf16(&source.Span16().unwrap()[..length as usize]).Utf8(),
        remainder: foundation::String::from_utf16(&source.Span16().unwrap()[length as usize..])
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
    let Some(source) = (unsafe { p.computed_style_owner.Get().as_ref() }) else {
        return;
    };
    if source.Floating() != foundation::EFloat::kNone
        || matches!(
            source.GetPosition(),
            foundation::EPosition::kAbsolute | foundation::EPosition::kFixed
        )
    {
        return;
    }
    let block = CanonicalIsBlockContainer(source.Display());
    if source.Display() != foundation::EDisplay::kInline && !p.display_contents && !block {
        search.stopped = true;
        return;
    }
    if !p.text.is_empty() {
        ConsiderFirstLetterText(
            &p.text,
            source,
            source.ShouldPreserveBreaks(),
            None,
            Some(p),
            search,
        );
    }
    if block && !search.stopped {
        search.stopped = true;
    }
}
fn IsAtomicFirstLetterBoundary(
    document: &Document,
    index: usize,
    source: &StyleComputedStyle,
) -> bool {
    let element = ElementMetadata(
        document,
        index,
        &UserInteractionState::default(),
        source.ImageOrientation()
            == foundation::RespectImageOrientationEnum::kRespectImageOrientation,
    );
    matches!(
        ElementKind(
            document,
            index,
            &element,
            &ComputedStyle::default(),
            Some(source),
        ),
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
    source: &'a StyleComputedStyle,
    search: &mut FirstLetterSearch<'a>,
) {
    let resolved = document.ResolvedStyleFor(index);
    ConsiderGeneratedFirstLetter(resolved.and_then(|r| r.before.as_ref()), search);
    for &child in AttachmentChildren(document, index).iter() {
        if search.stopped {
            return;
        }
        let node = document.Node(child);
        if node.Type() == DOMNodeType::kText {
            ConsiderFirstLetterText(
                node.Data(),
                source,
                source.ShouldPreserveBreaks(),
                Some(child),
                None,
                search,
            );
            continue;
        }
        if node.Type() != DOMNodeType::kElement {
            continue;
        }
        // Source: HTMLImageElement::CreateLayoutObject. A collapsed broken
        // image creates no LayoutObject; fallback content uses an ordinary
        // HTMLElement box and attaches the canonical UA alt subtree.
        if node.IsHTMLElement("img") && document.HTMLImageIsCollapsed(child) {
            continue;
        }
        let Some(r) = document.ResolvedStyleFor(child) else {
            continue;
        };
        let Some(child_source) = (unsafe { r.computed_style_owner.Get().as_ref() }) else {
            continue;
        };
        if !r.generates_box {
            if r.display_contents {
                FindFirstLetterContent(document, child, child_source, search);
            }
            continue;
        }
        if child_source.Floating() != foundation::EFloat::kNone
            || matches!(
                child_source.GetPosition(),
                foundation::EPosition::kAbsolute | foundation::EPosition::kFixed
            )
        {
            continue;
        }
        if IsAtomicFirstLetterBoundary(document, child, child_source) {
            search.stopped = true;
            return;
        }
        if child_source.Display() == foundation::EDisplay::kInline {
            FindFirstLetterContent(document, child, child_source, search);
        } else if CanonicalIsBlockContainer(child_source.Display()) {
            if r.first_letter.is_some() {
                search.stopped = true;
                return;
            }
            FindFirstLetterContent(document, child, child_source, search);
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
    source: &'a StyleComputedStyle,
) -> Option<FirstLetterCandidate<'a>> {
    let mut search = FirstLetterSearch {
        punctuation: FirstLetterPunctuationState::NotSeen,
        leading: None,
        result: None,
        stopped: false,
    };
    FindFirstLetterContent(document, index, source, &mut search);
    search.result
}
struct FirstLetterInsertion<'a> {
    originating_index: usize,
    originating_id: u64,
    svg_clip_resources: layoutng_assembly::internal::paint_input::SvgClipResourceBinding,
    pseudo: &'a PseudoElement,
    candidate: FirstLetterCandidate<'a>,
    consumed: bool,
}
// cpp: dom_to_layout/layout_object_tree_builder.cc:1139-1165
fn AppendInitialLetter(
    tree: &mut LayoutTreeUpdate<'_>,
    document: &Document,
    parent: *mut LayoutObject,
    insertion: &mut FirstLetterInsertion<'_>,
    remaining_id: u64,
    name: &str,
    generated: bool,
    svg_filter_graphs: &std::sync::Arc<
        Vec<layoutng_assembly::internal::paint_input::SvgFilterGraph>,
    >,
) {
    let data = ElementData {
        first_letter_pseudo: true,
        ..Default::default()
    };
    let letter_style = unsafe { insertion.pseudo.computed_style_owner.Get().as_ref() }
        .expect("first-letter pseudo style must be owned");
    let resolved_resources = document.ResolvedStyleResourcesFor(insertion.originating_index);
    let replaced_image_shader = None;
    let resources = layoutng_assembly::style_projection::StyleResourceBindings::FromParts(
        svg_filter_graphs,
        &insertion.svg_clip_resources,
        &resolved_resources.svg_mask_resource,
        resolved_resources.svg_pattern_content_target,
        &resolved_resources.svg_fill_server,
        &resolved_resources.svg_stroke_server,
        &resolved_resources.svg_markers,
        &replaced_image_shader,
    );
    let letter = tree.AddBoxFromStyle(
        unsafe { &mut *parent },
        insertion.originating_id | (1 << 46),
        letter_style,
        &resources,
        None,
        "::first-letter".into(),
        Some(data),
        NodeKind::kBox,
        true,
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
    let remaining = tree.AddTextFromStyle(
        unsafe { &mut *parent },
        remaining_id,
        insertion.candidate.remainder.clone(),
        name.into(),
        insertion.candidate.source_style,
        &resources,
        generated,
    );
    if let Some(node) = insertion.candidate.text_node.filter(|_| !generated) {
        LayoutTreeUpdate::SetEditingNodeFacts(remaining, document.EditingNodeFacts(node));
    }
    insertion.consumed = true;
}
#[derive(Clone, Copy)]
struct MarkerObjectBits {
    box_bit: u64,
    text_bit: u64,
    suffix_bit: u64,
    image_bit: u64,
}

// cpp: PseudoElement::AttachLayoutTree(kPseudoIdMarker)
// cpp: ListMarker::UpdateMarkerContentIfNeeded
fn AppendListMarker(
    tree: &mut LayoutTreeUpdate<'_>,
    document: &Document,
    originating_index: usize,
    id: u64,
    parent: *mut LayoutObject,
    item_style: &StyleComputedStyle,
    svg_clip_resources: &layoutng_assembly::internal::paint_input::SvgClipResourceBinding,
    svg_filter_graphs: &std::sync::Arc<
        Vec<layoutng_assembly::internal::paint_input::SvgFilterGraph>,
    >,
    marker_pseudo: Option<&PseudoElement>,
    pseudo_parent: usize,
    name: &str,
    bits: MarkerObjectBits,
) {
    let pseudo_style =
        marker_pseudo.and_then(|pseudo| unsafe { pseudo.computed_style_owner.Get().as_ref() });
    let marker_has_custom_content =
        pseudo_style.is_some_and(|source| !source.ContentBehavesAsNormal());
    let marker_node = document.GeneratedPseudoElement(
        pseudo_parent,
        layoutng_style::style::computed_style_constants::PseudoId::kPseudoIdMarker,
    );
    let marker_style = pseudo_style
        .or_else(|| marker_node.and_then(|n| document.ResolvedStyleFor(n)?.ComputedStyle()));
    let Some(marker_style) = marker_style else {
        return;
    };
    let marker_image = (!marker_has_custom_content)
        .then(|| layoutng_assembly::style_projection::ProjectListMarkerImage(marker_style))
        .flatten();
    let marker_text = if marker_has_custom_content {
        marker_pseudo.map_or_else(String::new, |pseudo| pseudo.text.clone())
    } else {
        DefaultListMarkerText(document, originating_index, marker_style)
    };
    if !matches!(
        item_style.Display(),
        foundation::EDisplay::kListItem
            | foundation::EDisplay::kInlineListItem
            | foundation::EDisplay::kFlowRootListItem
            | foundation::EDisplay::kInlineFlowRootListItem
    ) || (!marker_has_custom_content && marker_image.is_none() && marker_text.is_empty())
    {
        return;
    }
    let resolved_resources = document.ResolvedStyleResourcesFor(originating_index);
    let replaced_image_shader = None;
    let resources = layoutng_assembly::style_projection::StyleResourceBindings::FromParts(
        svg_filter_graphs,
        svg_clip_resources,
        &resolved_resources.svg_mask_resource,
        resolved_resources.svg_pattern_content_target,
        &resolved_resources.svg_fill_server,
        &resolved_resources.svg_stroke_server,
        &resolved_resources.svg_markers,
        &replaced_image_shader,
    );
    tree.WithGeneratedPseudoElementOwner(
        document
            .Node(marker_node.expect("canonical marker owner"))
            .Id(),
        |tree| {
            let marker = tree.AddBoxFromStyle(
                unsafe { &mut *parent },
                document
                    .Node(marker_node.expect("generated marker retains its DOM pseudo owner"))
                    .Id(),
                marker_style,
                &resources,
                None,
                name.into(),
                None,
                NodeKind::kListMarker,
                false,
            ) as *mut LayoutObject;
            if marker_has_custom_content {
                let pseudo = marker_pseudo.expect("custom marker content has resolved style");
                if pseudo.content.is_empty() && !marker_text.is_empty() {
                    tree.AddText(
                        unsafe { &mut *marker },
                        id | bits.text_bit,
                        marker_text,
                        format!("{name}-text"),
                        std::ptr::null(),
                        true,
                        std::ptr::null(),
                    );
                }
                for (ordinal, content) in pseudo.content.iter().enumerate() {
                    let sequence = ((ordinal as u64 + 1) & 0x7ff) << 32;
                    match content {
                        PseudoContent::Text(value) if !value.is_empty() => {
                            tree.AddText(
                                unsafe { &mut *marker },
                                id | bits.text_bit | sequence,
                                value.clone(),
                                format!("{name}-text-{ordinal}"),
                                std::ptr::null(),
                                true,
                                std::ptr::null(),
                            );
                        }
                        PseudoContent::Image {
                            resource_id,
                            shader,
                            natural_width,
                            natural_height,
                            resolution_scale: _,
                            known_to_be_opaque,
                        } if *resource_id != 0 || shader.is_some() => {
                            let mut element = ElementData {
                                natural_width: *natural_width,
                                natural_height: *natural_height,
                                image_resource_is_loaded: *resource_id != 0,
                                image_resource_known_to_be_opaque: *resource_id != 0
                                    && *known_to_be_opaque,
                                image_resource_maybe_animated: *resource_id != 0
                                    && document.ImageResourceMaybeAnimated(*resource_id),
                                // NaturalSize is already divided by the selected
                                // image-set resolution at the Style boundary.
                                image_device_pixel_ratio: 1.0,
                                ..Default::default()
                            };
                            if let (Some(width), Some(height)) = (*natural_width, *natural_height) {
                                if width > 0.0 && height > 0.0 {
                                    element.natural_aspect_ratio = Some(width / height);
                                }
                            }
                            let image_resources =
                        layoutng_assembly::style_projection::StyleResourceBindings::FromParts(
                            svg_filter_graphs,
                            svg_clip_resources,
                            &resolved_resources.svg_mask_resource,
                            resolved_resources.svg_pattern_content_target,
                            &resolved_resources.svg_fill_server,
                            &resolved_resources.svg_stroke_server,
                            &resolved_resources.svg_markers,
                            shader,
                        );
                            tree.AddGeneratedReplacedFromStyle(
                                unsafe { &mut *marker },
                                id | bits.image_bit | sequence,
                                marker_style,
                                &image_resources,
                                (*resource_id != 0).then_some(*resource_id),
                                element,
                                format!("{name}-image-{ordinal}"),
                            );
                        }
                        _ => {}
                    }
                }
            } else if let Some(image) = marker_image {
                let image_resources =
                    layoutng_assembly::style_projection::StyleResourceBindings::FromParts(
                        svg_filter_graphs,
                        svg_clip_resources,
                        &resolved_resources.svg_mask_resource,
                        resolved_resources.svg_pattern_content_target,
                        &resolved_resources.svg_fill_server,
                        &resolved_resources.svg_stroke_server,
                        &resolved_resources.svg_markers,
                        &image.shader,
                    );
                tree.AddGeneratedReplacedFromStyle(
                    unsafe { &mut *marker },
                    id | bits.image_bit,
                    marker_style,
                    &image_resources,
                    (image.resource_id != 0).then_some(image.resource_id),
                    ElementData {
                        natural_width: Some(image.natural_width),
                        natural_height: Some(image.natural_height),
                        natural_aspect_ratio: Some(image.natural_width / image.natural_height),
                        image_resource_is_loaded: image.resource_id != 0,
                        image_resource_known_to_be_opaque: image.resource_id != 0
                            && image.known_to_be_opaque,
                        image_resource_maybe_animated: image.resource_id != 0
                            && document.ImageResourceMaybeAnimated(image.resource_id),
                        ..Default::default()
                    },
                    format!("{name}-image"),
                );
            } else {
                let marker_type = unsafe { marker_style.ListStyleType().Get().as_ref() }
                    .filter(|style| style.IsCounterStyle())
                    .map(|style| style.GetCounterStyleName().Utf8());
                let symbol = marker_type
                    .as_ref()
                    .is_some_and(|name| matches!(name.as_str(), "disc" | "circle" | "square"));
                if symbol && marker_text.ends_with(' ') {
                    tree.AddText(
                        unsafe { &mut *marker },
                        id | bits.text_bit,
                        marker_text[..marker_text.len() - 1].into(),
                        format!("{name}-symbol"),
                        std::ptr::null(),
                        true,
                        std::ptr::null(),
                    );
                    let mut suffix_builder = ComputedStyleBuilder::from_style(marker_style);
                    suffix_builder.SetWidth(&foundation::Length::Fixed(12.0));
                    suffix_builder.SetHeight(&foundation::Length::Fixed(0.0));
                    let suffix = suffix_builder.TakeStyle();
                    tree.AddBoxFromStyle(
                        unsafe { &mut *marker },
                        id | bits.suffix_bit,
                        unsafe { &*suffix },
                        &resources,
                        None,
                        format!("{name}-suffix"),
                        None,
                        NodeKind::kBox,
                        true,
                    );
                } else {
                    tree.AddText(
                        unsafe { &mut *marker },
                        id | bits.text_bit,
                        marker_text,
                        format!("{name}-text"),
                        std::ptr::null(),
                        true,
                        std::ptr::null(),
                    );
                }
            }
        },
    );
}

// Element::AttachPrecedingPseudoElements/AttachOverscrollPseudoElements,
// element.h:2488-2494; element.cc:6273-6291. The tracker is the sole membership
// owner. The attachment context records aliases to these exact native boxes.
fn AppendOverscrollAreaParents(
    tree: &mut LayoutTreeUpdate<'_>,
    document: &Document,
    interaction: &UserInteractionState,
    container: usize,
    parent: *mut LayoutObject,
    svg_filter_graphs: &std::sync::Arc<
        Vec<layoutng_assembly::internal::paint_input::SvgFilterGraph>,
    >,
    svg_clip_path_graphs: &std::sync::Arc<
        layoutng_assembly::internal::paint_input::SvgClipResourceSnapshot,
    >,
) {
    if !unsafe { &*parent }.IsBox() {
        return;
    }
    let Some(members) = document.OverscrollMembersDOMSorted(container) else {
        return;
    };
    for &area in members.iter() {
        let Some(pseudo) = document.GeneratedPseudoElement(
            area,
            layoutng_style::style::computed_style_constants::PseudoId::kPseudoIdOverscrollAreaParent,
        ) else {
            continue;
        };
        let Some(style) = document
            .ResolvedStyleFor(pseudo)
            .and_then(|style| style.ComputedStyle())
        else {
            continue;
        };
        if style.Display() == foundation::EDisplay::kNone {
            continue;
        }
        let clip = super::layout_object_tree_builder::svg_clip_resource_binding(
            style,
            svg_clip_path_graphs,
            document.SVGResourceScope(area),
        );
        let resolved = document.ResolvedStyleResourcesFor(area);
        let image = None;
        let resources = layoutng_assembly::style_projection::StyleResourceBindings::FromParts(
            svg_filter_graphs,
            &clip,
            &resolved.svg_mask_resource,
            resolved.svg_pattern_content_target,
            &resolved.svg_fill_server,
            &resolved.svg_stroke_server,
            &resolved.svg_markers,
            &image,
        );
        let id = document.Node(pseudo).Id();
        let data = ElementMetadata(
            document,
            pseudo,
            interaction,
            style.ImageOrientation()
                == foundation::RespectImageOrientationEnum::kRespectImageOrientation,
        );
        let object = tree.WithGeneratedPseudoElementOwner(id, |tree| {
            tree.AddBoxFromStyle(
                unsafe { &mut *parent },
                id,
                style,
                &resources,
                None,
                "::-internal-overscroll-area-parent".into(),
                Some(data),
                NodeKind::kBox,
                false,
            ) as *mut LayoutObject
        });
        tree.BindOverscrollAreaParent(document.Node(area).Id(), object);
    }
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:1167-1206
fn AppendPseudoElement(
    tree: &mut LayoutTreeUpdate<'_>,
    document: &Document,
    originating_index: usize,
    id: u64,
    parent: *mut LayoutObject,
    pseudo: Option<&PseudoElement>,
    before: bool,
    mut first: Option<&mut FirstLetterInsertion<'_>>,
    svg_filter_graphs: &std::sync::Arc<
        Vec<layoutng_assembly::internal::paint_input::SvgFilterGraph>,
    >,
    svg_clip_path_graphs: &std::sync::Arc<
        layoutng_assembly::internal::paint_input::SvgClipResourceSnapshot,
    >,
) {
    let Some(p) = pseudo else {
        return;
    };
    let text_bit = 1u64 << if before { 49 } else { 47 };
    let name = if before { "::before" } else { "::after" };
    let Some(source) = (unsafe { p.computed_style_owner.Get().as_ref() }) else {
        return;
    };
    let clip_resources = super::layout_object_tree_builder::svg_clip_resource_binding(
        source,
        svg_clip_path_graphs,
        document.SVGResourceScope(originating_index),
    );
    let resolved_resources = document.ResolvedStyleResourcesFor(originating_index);
    let replaced_image_shader = None;
    let resources = layoutng_assembly::style_projection::StyleResourceBindings::FromParts(
        svg_filter_graphs,
        &clip_resources,
        &resolved_resources.svg_mask_resource,
        resolved_resources.svg_pattern_content_target,
        &resolved_resources.svg_fill_server,
        &resolved_resources.svg_stroke_server,
        &resolved_resources.svg_markers,
        &replaced_image_shader,
    );
    let pseudo_node = document
        .GeneratedPseudoElement(
            originating_index,
            if before {
                layoutng_style::style::computed_style_constants::PseudoId::kPseudoIdBefore
            } else {
                layoutng_style::style::computed_style_constants::PseudoId::kPseudoIdAfter
            },
        )
        .expect("style-generated pseudo has a canonical DOM owner");
    tree.WithGeneratedPseudoElementOwner(document.Node(pseudo_node).Id(), |tree| {
        let object = if p.display_contents {
            parent
        } else {
            tree.AddBoxFromStyle(
                unsafe { &mut *parent },
                document.Node(pseudo_node).Id(),
                source,
                &resources,
                None,
                name.into(),
                None,
                NodeKind::kBox,
                false,
            ) as *mut LayoutObject
        };
        AppendListMarker(
            tree,
            document,
            originating_index,
            id,
            object,
            source,
            &clip_resources,
            svg_filter_graphs,
            p.marker.as_deref(),
            pseudo_node,
            if before {
                "::before::marker"
            } else {
                "::after::marker"
            },
            if before {
                MarkerObjectBits {
                    box_bit: 1 << 59,
                    text_bit: 1 << 57,
                    suffix_bit: 1 << 55,
                    image_bit: 1 << 53,
                }
            } else {
                MarkerObjectBits {
                    box_bit: 1 << 58,
                    text_bit: 1 << 56,
                    suffix_bit: 1 << 54,
                    image_bit: 1 << 52,
                }
            },
        );
        // Older fixtures may still construct the concatenated text projection
        // directly. Production style resolution retains the ordered ContentData
        // chain below, matching PseudoElement::AttachLayoutTree.
        if p.content.is_empty() && !p.text.is_empty() {
            let first = first.as_deref_mut().filter(|f| {
                !f.consumed
                    && f.candidate
                        .generated
                        .is_some_and(|candidate| std::ptr::eq(candidate, p))
            });
            if let Some(first) = first {
                AppendInitialLetter(
                    tree,
                    document,
                    object,
                    first,
                    id | text_bit,
                    &format!("{name}-text"),
                    true,
                    svg_filter_graphs,
                );
            } else {
                if p.display_contents {
                    tree.AddTextFromStyle(
                        unsafe { &mut *object },
                        id | text_bit,
                        p.text.clone(),
                        format!("{name}-text"),
                        source,
                        &resources,
                        true,
                    );
                } else {
                    tree.AddText(
                        unsafe { &mut *object },
                        id | text_bit,
                        p.text.clone(),
                        format!("{name}-text"),
                        std::ptr::null(),
                        true,
                        std::ptr::null(),
                    );
                }
            }
            return;
        }
        let mut first_consumed_all_text = false;
        for (ordinal, content) in p.content.iter().enumerate() {
            let sequence = ((ordinal as u64 + 1) & 0x7ff) << 32;
            match content {
                PseudoContent::Text(value) if !value.is_empty() => {
                    if first_consumed_all_text {
                        continue;
                    }
                    let first = first.as_deref_mut().filter(|f| {
                        !f.consumed
                            && f.candidate
                                .generated
                                .is_some_and(|candidate| std::ptr::eq(candidate, p))
                    });
                    if let Some(first) = first {
                        // The current first-letter adapter reasons over the
                        // concatenated textual view. Emit it once and let the
                        // existing split preserve that behavior.
                        AppendInitialLetter(
                            tree,
                            document,
                            object,
                            first,
                            id | text_bit,
                            &format!("{name}-text"),
                            true,
                            svg_filter_graphs,
                        );
                        first_consumed_all_text = true;
                    } else {
                        if p.display_contents {
                            tree.AddTextFromStyle(
                                unsafe { &mut *object },
                                id | text_bit | sequence,
                                value.clone(),
                                format!("{name}-text-{ordinal}"),
                                source,
                                &resources,
                                true,
                            );
                        } else {
                            tree.AddText(
                                unsafe { &mut *object },
                                id | text_bit | sequence,
                                value.clone(),
                                format!("{name}-text-{ordinal}"),
                                std::ptr::null(),
                                true,
                                std::ptr::null(),
                            );
                        }
                    }
                }
                PseudoContent::Image {
                    resource_id,
                    shader,
                    natural_width,
                    natural_height,
                    resolution_scale: _,
                    known_to_be_opaque,
                } if *resource_id != 0 || shader.is_some() => {
                    let mut element = ElementData {
                        natural_width: *natural_width,
                        natural_height: *natural_height,
                        image_resource_is_loaded: *resource_id != 0,
                        image_resource_known_to_be_opaque: *resource_id != 0 && *known_to_be_opaque,
                        image_resource_maybe_animated: *resource_id != 0
                            && document.ImageResourceMaybeAnimated(*resource_id),
                        // NaturalSize is already divided by the selected
                        // image-set resolution at the Style boundary.
                        image_device_pixel_ratio: 1.0,
                        ..Default::default()
                    };
                    if let (Some(width), Some(height)) = (*natural_width, *natural_height) {
                        if width > 0.0 && height > 0.0 {
                            element.natural_aspect_ratio = Some(width / height);
                        }
                    }
                    let image_bit = 1u64 << if before { 44 } else { 43 };
                    let image_resources =
                        layoutng_assembly::style_projection::StyleResourceBindings::FromParts(
                            svg_filter_graphs,
                            &clip_resources,
                            &resolved_resources.svg_mask_resource,
                            resolved_resources.svg_pattern_content_target,
                            &resolved_resources.svg_fill_server,
                            &resolved_resources.svg_stroke_server,
                            &resolved_resources.svg_markers,
                            shader,
                        );
                    tree.AddGeneratedReplacedFromStyle(
                        unsafe { &mut *object },
                        id | image_bit | sequence,
                        source,
                        &image_resources,
                        (*resource_id != 0).then_some(*resource_id),
                        element,
                        format!("{name}-image-{ordinal}"),
                    );
                }
                _ => {}
            }
        }
    });
}

// cpp: dom_to_layout/layout_object_tree_builder.cc:1208-1485
fn AppendChildren<'a>(
    tree: &mut LayoutTreeUpdate<'_>,
    document: &'a Document,
    interaction: &UserInteractionState,
    index: usize,
    parent: *mut LayoutObject,
    flattened: Option<(usize, &'a StyleComputedStyle)>,
    first: Option<&mut FirstLetterInsertion<'a>>,
    svg_filter_graphs: &std::sync::Arc<
        Vec<layoutng_assembly::internal::paint_input::SvgFilterGraph>,
    >,
    svg_clip_path_graphs: &std::sync::Arc<
        layoutng_assembly::internal::paint_input::SvgClipResourceSnapshot,
    >,
) {
    let children = AttachmentChildren(document, index);
    AppendSelectedChildren(
        tree,
        document,
        interaction,
        index,
        parent,
        flattened,
        first,
        svg_filter_graphs,
        svg_clip_path_graphs,
        &children,
    );
}

fn AppendSelectedChildren<'a>(
    tree: &mut LayoutTreeUpdate<'_>,
    document: &'a Document,
    interaction: &UserInteractionState,
    index: usize,
    parent: *mut LayoutObject,
    flattened: Option<(usize, &'a StyleComputedStyle)>,
    mut first: Option<&mut FirstLetterInsertion<'a>>,
    svg_filter_graphs: &std::sync::Arc<
        Vec<layoutng_assembly::internal::paint_input::SvgFilterGraph>,
    >,
    svg_clip_path_graphs: &std::sync::Arc<
        layoutng_assembly::internal::paint_input::SvgClipResourceSnapshot,
    >,
    children: &[usize],
) {
    let dom_parent = document.Node(index);
    // The original host/slot remains the inheritance and attachment context;
    // its ShadowRoot fragment is a carrier, never a styled layout parent.
    for &child in children {
        let node = document.Node(child);
        // Element::AttachLayoutTree redirects only this area's attachment; its
        // DOM/flat-tree parent still supplies style and metadata relationships.
        let parent = tree.OverscrollAreaParent(node.Id()).unwrap_or(parent);

        if node.Type() == DOMNodeType::kText {
            // SVG character data creates layout text only in SVG text-content
            // elements. Text under <g>, <defs>, shapes, etc. stays in the DOM
            // but is not a child of SVGContentContainer in Chromium.
            let svg_text_parent = dom_parent.Namespace() == DOMNamespace::kSVG
                && matches!(dom_parent.Name(), "text" | "tspan" | "textPath" | "a");
            if dom_parent.Namespace() == DOMNamespace::kSVG && !svg_text_parent {
                continue;
            }
            let style = flattened.or_else(|| {
                document
                    .ResolvedStyleFor(index)
                    .and_then(|resolved| unsafe {
                        resolved
                            .computed_style_owner
                            .Get()
                            .as_ref()
                            .map(|style| (index, style))
                    })
            });
            let preserves_breaks = style.is_some_and(|(_, style)| style.ShouldPreserveBreaks());
            if !preserves_breaks
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
                AppendInitialLetter(
                    tree,
                    document,
                    parent,
                    letter,
                    node.Id(),
                    "#text",
                    false,
                    svg_filter_graphs,
                );
            } else if !node.Data().is_empty() {
                if let Some((style_index, style)) = style {
                    let clip = super::layout_object_tree_builder::svg_clip_resource_binding(
                        style,
                        svg_clip_path_graphs,
                        document.SVGResourceScope(style_index),
                    );
                    let resolved_resources = document.ResolvedStyleResourcesFor(style_index);
                    let replaced_image_shader = None;
                    let resources =
                        layoutng_assembly::style_projection::StyleResourceBindings::FromParts(
                            svg_filter_graphs,
                            &clip,
                            &resolved_resources.svg_mask_resource,
                            resolved_resources.svg_pattern_content_target,
                            &resolved_resources.svg_fill_server,
                            &resolved_resources.svg_stroke_server,
                            &resolved_resources.svg_markers,
                            &replaced_image_shader,
                        );
                    let object = tree.AddTextFromStyle(
                        unsafe { &mut *parent },
                        node.Id(),
                        node.Data().into(),
                        "#text".into(),
                        style,
                        &resources,
                        false,
                    );
                    LayoutTreeUpdate::SetEditingNodeFacts(object, document.EditingNodeFacts(child));
                } else {
                    let object = tree.AddText(
                        unsafe { &mut *parent },
                        node.Id(),
                        node.Data().into(),
                        "#text".into(),
                        std::ptr::null(),
                        false,
                        std::ptr::null(),
                    );
                    LayoutTreeUpdate::SetEditingNodeFacts(object, document.EditingNodeFacts(child));
                }
            }
            continue;
        }
        if node.Type() != DOMNodeType::kElement {
            continue;
        }
        // HTMLNoScriptElement::LayoutObjectIsNeeded: preserve its DOM and
        // canonical style, while script-enabled documents create no layout subtree.
        if node.IsHTMLElement("noscript") && document.ScriptingEnabled() {
            continue;
        }
        if node.IsElement("textPath", DOMNamespace::kSVG)
            && !svg_text_path_layout_needed(dom_parent.Namespace(), dom_parent.Name(), unsafe {
                &*parent
            })
        {
            continue;
        }
        if node.Namespace() == DOMNamespace::kSVG
            && matches!(node.Name(), "title" | "desc" | "metadata")
        {
            continue;
        }
        let Some(resolved) = document.ResolvedStyleFor(child) else {
            continue;
        };
        let Some(source_style) = (unsafe { resolved.computed_style_owner.Get().as_ref() }) else {
            continue;
        };
        if !resolved.generates_box {
            if matches!(
                document.InputType(child),
                Some(
                    DOMInputType::kHidden
                        | DOMInputType::kColor
                        | DOMInputType::kFile
                        | DOMInputType::kImage
                )
            ) {
                document
                    .AttributeLifecycle()
                    .RecordInputAttached(node.Id(), None);
            }
            if resolved.display_contents {
                AppendPseudoElement(
                    tree,
                    document,
                    child,
                    node.Id(),
                    parent,
                    resolved.before.as_ref(),
                    true,
                    first.as_deref_mut(),
                    svg_filter_graphs,
                    svg_clip_path_graphs,
                );
                AppendChildren(
                    tree,
                    document,
                    interaction,
                    child,
                    parent,
                    Some((child, source_style)),
                    first.as_deref_mut(),
                    svg_filter_graphs,
                    svg_clip_path_graphs,
                );
                AppendPseudoElement(
                    tree,
                    document,
                    child,
                    node.Id(),
                    parent,
                    resolved.after.as_ref(),
                    false,
                    first.as_deref_mut(),
                    svg_filter_graphs,
                    svg_clip_path_graphs,
                );
            }
            continue;
        }
        let element = ElementMetadata(
            document,
            child,
            interaction,
            source_style.ImageOrientation()
                == foundation::RespectImageOrientationEnum::kRespectImageOrientation,
        );
        let clip_resources = super::layout_object_tree_builder::svg_clip_resource_binding(
            source_style,
            svg_clip_path_graphs,
            document.SVGResourceScope(child),
        );
        let kind = ElementKind(
            document,
            child,
            &element,
            &ComputedStyle::default(),
            Some(source_style),
        );
        let control = element.form_control_type;
        let range = control == Some(FormControlType::kInputRange);
        let file = control == Some(FormControlType::kInputFile);
        let menu = element.select_uses_menu_list.unwrap_or(false);
        let select_list = matches!(
            control,
            Some(FormControlType::kSelectOne | FormControlType::kSelectMultiple)
        ) && !menu;
        let adjusted_style = if (range || select_list) && source_style.Height().IsAuto() {
            let mut builder = ComputedStyleBuilder::from_style(source_style);
            let height = if range {
                16.0
            } else {
                positive_attribute_integer(node, "size", 4) as f32
                    * source_style.ComputedLineHeight()
                    + 2.0
            };
            builder.SetHeight(&foundation::Length::Fixed(height));
            builder.TakeStyle()
        } else {
            std::ptr::null_mut()
        };
        let attached_style = unsafe { adjusted_style.as_ref() }.unwrap_or(source_style);
        let resolved_resources = document.ResolvedStyleResourcesFor(child);
        let replaced_image_shader = None;
        let resources = layoutng_assembly::style_projection::StyleResourceBindings::FromParts(
            svg_filter_graphs,
            &clip_resources,
            &resolved_resources.svg_mask_resource,
            resolved_resources.svg_pattern_content_target,
            &resolved_resources.svg_fill_server,
            &resolved_resources.svg_stroke_server,
            &resolved_resources.svg_markers,
            &replaced_image_shader,
        );
        let broken_image_resources = document.HTMLImageBrokenResources(child);
        let image_resource_update = (kind == NodeKind::kReplaced
            && (node.Name() == "img" || document.InputType(child) == Some(DOMInputType::kImage)))
        .then(|| document.ImageLoaderLayoutUpdate(child))
        .flatten();
        let image_resource_id = image_resource_update
            .as_ref()
            .and_then(|update| update.content.as_ref())
            .and_then(|content| content.Metadata())
            .map(|image| image.id)
            .or_else(|| {
                broken_image_resources
                    .as_ref()
                    .and_then(|variants| tree.SelectedBrokenImageResourceId(variants))
            });
        let created_before = tree.CreatedObjectCount();
        let object = if matches!(kind, NodeKind::kReplaced | NodeKind::kFrame) {
            tree.AddReplacedFromStyle(
                unsafe { &mut *parent },
                node.Id(),
                attached_style,
                &resources,
                image_resource_id,
                Some(element),
                node.Name().into(),
                kind,
            )
        } else {
            tree.AddBoxFromStyle(
                unsafe { &mut *parent },
                node.Id(),
                attached_style,
                &resources,
                image_resource_id,
                node.Name().into(),
                Some(element),
                kind,
                false,
            )
        } as *mut LayoutObject;
        LayoutTreeUpdate::BeginElementLayoutTreeAttachment(unsafe { &mut *object });
        let newly_attached_media =
            tree.CreatedObjectCount() != created_before && document.IsMediaElement(child);
        let newly_attached_input = tree.CreatedObjectCount() != created_before
            && matches!(
                document.InputType(child),
                Some(
                    DOMInputType::kHidden
                        | DOMInputType::kColor
                        | DOMInputType::kFile
                        | DOMInputType::kImage
                )
            );
        if let Some(variants) = broken_image_resources {
            tree.UseBrokenImageResource(unsafe { &mut *object }, variants);
        } else if let Some(update) = image_resource_update {
            LayoutTreeUpdate::UpdateHTMLImageResource(unsafe { &mut *object }, update);
        }
        LayoutTreeUpdate::SetEditingNodeFacts(
            unsafe { &mut *object },
            document.EditingNodeFacts(child),
        );
        if range {
            let mut track_builder = ComputedStyleBuilder::from_style(attached_style);
            track_builder.SetDisplay(foundation::EDisplay::kFlex);
            track_builder.SetHeight(&foundation::Length::Fixed(16.0));
            track_builder.SetWidth(&foundation::Length::Percent(100.0));
            let track_style = track_builder.TakeStyle();
            let track = tree.AddBoxFromStyle(
                unsafe { &mut *object },
                node.Id() | (1 << 61),
                unsafe { &*track_style },
                &resources,
                None,
                "::-webkit-slider-container".into(),
                Some(ElementData {
                    control_host_ancestor: Some(1),
                    ..Default::default()
                }),
                NodeKind::kBox,
                false,
            ) as *mut LayoutObject;
            let mut thumb_builder = ComputedStyleBuilder::from_style(unsafe { &*track_style });
            thumb_builder.SetWidth(&foundation::Length::Fixed(16.0));
            thumb_builder.SetHeight(&foundation::Length::Fixed(16.0));
            thumb_builder.SetEffectiveAppearance(AppearanceValue::kSliderThumbHorizontal);
            let thumb_style = thumb_builder.TakeStyle();
            tree.AddBoxFromStyle(
                unsafe { &mut *track },
                node.Id() | (1 << 60),
                unsafe { &*thumb_style },
                &resources,
                None,
                "::-webkit-slider-thumb".into(),
                None,
                NodeKind::kSliderThumb,
                false,
            );
        }
        if file {
            // FileInputType owns real UA-shadow nodes. Project their resolved
            // styles and stable node identities instead of fabricating a
            // second button/status subtree at layout time.
            let button_index = document
                .FileUploadButtonElement(child)
                .expect("file input must retain its upload button");
            let button_node = document.Node(button_index);
            if let Some(button_resolved) = document
                .ResolvedStyleFor(button_index)
                .filter(|style| style.generates_box)
            {
                let button_source = unsafe { button_resolved.computed_style_owner.Get().as_ref() }
                    .expect("resolved file button style must be owned");
                let mut button_data = ElementMetadata(
                    document,
                    button_index,
                    interaction,
                    button_source.ImageOrientation()
                        == foundation::RespectImageOrientationEnum::kRespectImageOrientation,
                );
                button_data.control_host_ancestor = Some(1);
                button_data.file_upload_button = true;
                let button_clip = super::layout_object_tree_builder::svg_clip_resource_binding(
                    button_source,
                    svg_clip_path_graphs,
                    document.SVGResourceScope(button_index),
                );
                let button_resolved_resources = document.ResolvedStyleResourcesFor(button_index);
                let replaced_image_shader = None;
                let button_resources =
                    layoutng_assembly::style_projection::StyleResourceBindings::FromParts(
                        svg_filter_graphs,
                        &button_clip,
                        &button_resolved_resources.svg_mask_resource,
                        button_resolved_resources.svg_pattern_content_target,
                        &button_resolved_resources.svg_fill_server,
                        &button_resolved_resources.svg_stroke_server,
                        &button_resolved_resources.svg_markers,
                        &replaced_image_shader,
                    );
                let button = tree.AddBoxFromStyle(
                    unsafe { &mut *object },
                    button_node.Id(),
                    button_source,
                    &button_resources,
                    None,
                    "::-webkit-file-upload-button".into(),
                    Some(button_data),
                    NodeKind::kFormControl,
                    false,
                ) as *mut LayoutObject;
                let label = attribute(button_node, "value").unwrap_or_default();
                if !label.is_empty() {
                    tree.AddText(
                        unsafe { &mut *button },
                        button_node.Id() | (1 << 58),
                        label.into(),
                        "::file-upload-label".into(),
                        std::ptr::null(),
                        false,
                        std::ptr::null(),
                    );
                }
            }
            let status_index = document
                .FileStatusElement(child)
                .expect("file input must retain its status element");
            let status_node = document.Node(status_index);
            if let Some(status_resolved) = document
                .ResolvedStyleFor(status_index)
                .filter(|style| style.generates_box)
            {
                let status_source = unsafe { status_resolved.computed_style_owner.Get().as_ref() };
                let status_source =
                    status_source.expect("resolved file status style must be owned");
                let status_clip = super::layout_object_tree_builder::svg_clip_resource_binding(
                    status_source,
                    svg_clip_path_graphs,
                    document.SVGResourceScope(status_index),
                );
                let status_resolved_resources = document.ResolvedStyleResourcesFor(status_index);
                let replaced_image_shader = None;
                let status_resources =
                    layoutng_assembly::style_projection::StyleResourceBindings::FromParts(
                        svg_filter_graphs,
                        &status_clip,
                        &status_resolved_resources.svg_mask_resource,
                        status_resolved_resources.svg_pattern_content_target,
                        &status_resolved_resources.svg_fill_server,
                        &status_resolved_resources.svg_stroke_server,
                        &status_resolved_resources.svg_markers,
                        &replaced_image_shader,
                    );
                let status = tree.AddBoxFromStyle(
                    unsafe { &mut *object },
                    status_node.Id(),
                    status_source,
                    &status_resources,
                    None,
                    "::file-status".into(),
                    Some(ElementMetadata(
                        document,
                        status_index,
                        interaction,
                        status_source.ImageOrientation()
                            == foundation::RespectImageOrientationEnum::kRespectImageOrientation,
                    )),
                    NodeKind::kBox,
                    false,
                ) as *mut LayoutObject;
                for &text_index in status_node.Children() {
                    let text = document.Node(text_index);
                    if text.Type() == DOMNodeType::kText && !text.Data().is_empty() {
                        tree.AddText(
                            unsafe { &mut *status },
                            text.Id(),
                            text.Data().into(),
                            "::file-status-label".into(),
                            std::ptr::null(),
                            false,
                            std::ptr::null(),
                        );
                    }
                }
            }
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
            let mut inner_builder = ComputedStyleBuilder::from_style(attached_style);
            inner_builder.SetDisplay(foundation::EDisplay::kBlock);
            inner_builder.SetFlexGrow(1.0);
            inner_builder.SetFlexShrink(1.0);
            inner_builder.SetMinWidth(&foundation::Length::Fixed(0.0));
            inner_builder.SetOverflowX(foundation::EOverflow::kHidden);
            inner_builder.SetOverflowY(foundation::EOverflow::kHidden);
            let inner_style = inner_builder.TakeStyle();
            let inner = tree.AddBoxFromStyle(
                unsafe { &mut *object },
                node.Id() | (1 << 52),
                unsafe { &*inner_style },
                &resources,
                None,
                "::-internal-select-inner-element".into(),
                Some(ElementData {
                    control_host_ancestor: Some(1),
                    ..Default::default()
                }),
                NodeKind::kBox,
                false,
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
            let suggested = document.SuggestedValue(child);
            if let Some(placeholder) = document.TextControlPlaceholderElement(child) {
                if let Some(placeholder_resolved) = document
                    .ResolvedStyleFor(placeholder)
                    .filter(|style| style.generates_box)
                {
                    let placeholder_style = unsafe {
                        placeholder_resolved
                            .computed_style_owner
                            .Get()
                            .as_ref()
                            .expect("resolved text-control placeholder style must be owned")
                    };
                    let placeholder_clip =
                        super::layout_object_tree_builder::svg_clip_resource_binding(
                            placeholder_style,
                            svg_clip_path_graphs,
                            document.SVGResourceScope(placeholder),
                        );
                    let placeholder_resolved_resources =
                        document.ResolvedStyleResourcesFor(placeholder);
                    let replaced_image_shader = None;
                    let placeholder_resources =
                        layoutng_assembly::style_projection::StyleResourceBindings::FromParts(
                            svg_filter_graphs,
                            &placeholder_clip,
                            &placeholder_resolved_resources.svg_mask_resource,
                            placeholder_resolved_resources.svg_pattern_content_target,
                            &placeholder_resolved_resources.svg_fill_server,
                            &placeholder_resolved_resources.svg_stroke_server,
                            &placeholder_resolved_resources.svg_markers,
                            &replaced_image_shader,
                        );
                    let mut placeholder_data = ElementMetadata(
                        document,
                        placeholder,
                        interaction,
                        placeholder_style.ImageOrientation()
                            == foundation::RespectImageOrientationEnum::kRespectImageOrientation,
                    );
                    placeholder_data.control_host_ancestor = Some(1);
                    placeholder_data.text_control_placeholder = true;
                    let placeholder_object = tree.AddBoxFromStyle(
                        unsafe { &mut *object },
                        document.Node(placeholder).Id(),
                        placeholder_style,
                        &placeholder_resources,
                        None,
                        "::placeholder".into(),
                        Some(placeholder_data),
                        NodeKind::kBox,
                        false,
                    ) as *mut LayoutObject;
                    for &text_index in document.Node(placeholder).Children() {
                        let text = document.Node(text_index);
                        if text.Type() == DOMNodeType::kText && !text.Data().is_empty() {
                            tree.AddText(
                                unsafe { &mut *placeholder_object },
                                text.Id(),
                                text.Data().into(),
                                if suggested.is_some_and(|value| !value.is_empty()) {
                                    "::-internal-input-suggested"
                                } else {
                                    "::placeholder"
                                }
                                .into(),
                                std::ptr::null(),
                                false,
                                std::ptr::null(),
                            );
                        }
                    }
                }
            }
            let editor_index = document
                .TextControlInnerEditorElement(child)
                .expect("text control UA shadow tree must retain an inner editor");
            let editor_resolved = document
                .ResolvedStyleFor(editor_index)
                .filter(|style| style.generates_box)
                .expect("text-control inner editor must generate a layout box");
            let editor_style = unsafe {
                editor_resolved
                    .computed_style_owner
                    .Get()
                    .as_ref()
                    .expect("resolved text-control inner editor style must be owned")
            };
            let editor_clip = super::layout_object_tree_builder::svg_clip_resource_binding(
                editor_style,
                svg_clip_path_graphs,
                document.SVGResourceScope(editor_index),
            );
            let editor_resolved_resources = document.ResolvedStyleResourcesFor(editor_index);
            let replaced_image_shader = None;
            let editor_resources =
                layoutng_assembly::style_projection::StyleResourceBindings::FromParts(
                    svg_filter_graphs,
                    &editor_clip,
                    &editor_resolved_resources.svg_mask_resource,
                    editor_resolved_resources.svg_pattern_content_target,
                    &editor_resolved_resources.svg_fill_server,
                    &editor_resolved_resources.svg_stroke_server,
                    &editor_resolved_resources.svg_markers,
                    &replaced_image_shader,
                );
            let editor = tree.AddBoxFromStyle(
                unsafe { &mut *object },
                document.Node(editor_index).Id(),
                editor_style,
                &editor_resources,
                None,
                "::-webkit-textfield-editor".into(),
                Some(ElementData {
                    control_host_ancestor: Some(1),
                    text_control_inner_editor: true,
                    ..Default::default()
                }),
                NodeKind::kBox,
                false,
            ) as *mut LayoutObject;
            // TextControlElement owns the UA Text/BR nodes. Keeping their DOM
            // identities here makes fragments, hit testing and live IME ranges
            // refer to the same editor instead of a synthesized value leaf.
            for &editor_child in document.Node(editor_index).Children() {
                let editor_node = document.Node(editor_child);
                if editor_node.Type() == DOMNodeType::kText {
                    if !editor_node.Data().is_empty() {
                        tree.AddText(
                            unsafe { &mut *editor },
                            editor_node.Id(),
                            editor_node.Data().into(),
                            "::text-control-value".into(),
                            std::ptr::null(),
                            false,
                            std::ptr::null(),
                        );
                    }
                } else if editor_node.IsHTMLElement("br") {
                    let mut break_builder = ComputedStyleBuilder::from_style(editor_style);
                    break_builder.SetDisplay(foundation::EDisplay::kInline);
                    break_builder.ResetWidth();
                    let break_style = break_builder.TakeStyle();
                    tree.AddBoxFromStyle(
                        unsafe { &mut *editor },
                        editor_node.Id(),
                        unsafe { &*break_style },
                        &editor_resources,
                        None,
                        if document.TextControlIsPlaceholderBreak(editor_child) {
                            "::text-control-placeholder-break"
                        } else {
                            "::text-control-line-break"
                        }
                        .into(),
                        None,
                        NodeKind::kLineBreak,
                        false,
                    );
                }
            }
        }
        AppendOverscrollAreaParents(
            tree,
            document,
            interaction,
            child,
            object,
            svg_filter_graphs,
            svg_clip_path_graphs,
        );
        AppendListMarker(
            tree,
            document,
            child,
            node.Id(),
            object,
            source_style,
            &clip_resources,
            svg_filter_graphs,
            resolved.marker.as_ref(),
            child,
            "::marker",
            MarkerObjectBits {
                box_bit: 1 << 63,
                text_bit: 1 << 62,
                suffix_bit: 1 << 61,
                image_bit: 1 << 60,
            },
        );
        if !matches!(kind, NodeKind::kReplaced | NodeKind::kFrame) && !textarea && !menu && !range {
            let mut own_first = if CanonicalIsBlockContainer(source_style.Display()) {
                resolved.first_letter.as_ref().and_then(|pseudo| {
                    FindFirstLetterCandidate(document, child, source_style).map(|candidate| {
                        FirstLetterInsertion {
                            originating_index: child,
                            originating_id: node.Id(),
                            svg_clip_resources: clip_resources.clone(),
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
                    document,
                    child,
                    node.Id(),
                    object,
                    resolved.before.as_ref(),
                    true,
                    Some(own),
                    svg_filter_graphs,
                    svg_clip_path_graphs,
                );
                AppendChildren(
                    tree,
                    document,
                    interaction,
                    child,
                    object,
                    None,
                    Some(own),
                    svg_filter_graphs,
                    svg_clip_path_graphs,
                );
                AppendPseudoElement(
                    tree,
                    document,
                    child,
                    node.Id(),
                    object,
                    resolved.after.as_ref(),
                    false,
                    Some(own),
                    svg_filter_graphs,
                    svg_clip_path_graphs,
                );
            } else {
                AppendPseudoElement(
                    tree,
                    document,
                    child,
                    node.Id(),
                    object,
                    resolved.before.as_ref(),
                    true,
                    first.as_deref_mut(),
                    svg_filter_graphs,
                    svg_clip_path_graphs,
                );
                AppendChildren(
                    tree,
                    document,
                    interaction,
                    child,
                    object,
                    None,
                    first.as_deref_mut(),
                    svg_filter_graphs,
                    svg_clip_path_graphs,
                );
                AppendPseudoElement(
                    tree,
                    document,
                    child,
                    node.Id(),
                    object,
                    resolved.after.as_ref(),
                    false,
                    first.as_deref_mut(),
                    svg_filter_graphs,
                    svg_clip_path_graphs,
                );
            }
        }
        LayoutTreeUpdate::FinishElementLayoutTreeAttachment(unsafe { &mut *object });
        if newly_attached_media {
            document.AttributeLifecycle().RecordMediaAttached(node.Id());
        }
        if newly_attached_input {
            document
                .AttributeLifecycle()
                .RecordInputAttached(node.Id(), Some(unsafe { &*object }.IsLayoutImage()));
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
        tree.Reattach(
            &document
                .StyleState()
                .layout_reattach_nodes
                .iter()
                .copied()
                .collect::<Vec<_>>(),
        );
        let svg_filter_graphs = super::svg_filter_graph::build_persistent(document);
        let svg_clip_path_graphs = document
            .SVGResources()
            .cloned()
            .unwrap_or_else(|| super::svg_clip_path_graph::build_persistent(document));
        let index = document
            .Node(document.Root())
            .Children()
            .iter()
            .copied()
            .find(|&i| document.Node(i).Type() == DOMNodeType::kElement);
        // Blink creates the document's LayoutView before a document element
        // exists. Synchronous geometry/scroll getters can enter this lifecycle
        // while response bytes have not yet produced any HTML nodes.
        let Some(index) = index else {
            let root = tree.CreateRootDefault(
                document.Node(document.Root()).Id(),
                &ComputedStyle::default(),
            );
            LayoutTreeUpdate::SetEditingNodeFacts(root, document.EditingNodeFacts(document.Root()));
            return;
        };
        if document.DocumentType(document.Root())
            == crate::persistent_document::DOMDocumentType::kSVG
        {
            // Document::Initialize cc:3298-3306 creates the view separately;
            // SVGSVGElement creates its own LayoutSVGRoot below that view.
            let style = document
                .ViewportStyle()
                .expect("SVG Document viewport style owner");
            let input = ComputedStyle {
                style: style.Get(),
                ..Default::default()
            };
            let view = tree.CreateRootDefault(document.Node(document.Root()).Id(), &input)
                as *mut LayoutObject;
            LayoutTreeUpdate::SetEditingNodeFacts(
                unsafe { &mut *view },
                document.EditingNodeFacts(document.Root()),
            );
            AppendSelectedChildren(
                &mut tree,
                document,
                interaction,
                document.Root(),
                view,
                None,
                None,
                &svg_filter_graphs,
                &svg_clip_path_graphs,
                &[index],
            );
            return;
        }
        let node = document.Node(index);
        let Some(resolved) = document.ResolvedStyleFor(index) else {
            let root = tree.CreateRootDefault(
                document.Node(document.Root()).Id(),
                &ComputedStyle::default(),
            );
            LayoutTreeUpdate::SetEditingNodeFacts(root, document.EditingNodeFacts(document.Root()));
            return;
        };
        if !resolved.generates_box {
            let root = tree.CreateRootDefault(
                document.Node(document.Root()).Id(),
                &ComputedStyle::default(),
            );
            LayoutTreeUpdate::SetEditingNodeFacts(root, document.EditingNodeFacts(document.Root()));
            return;
        }
        let Some(source_style) = (unsafe { resolved.computed_style_owner.Get().as_ref() }) else {
            let root = tree.CreateRootDefault(
                document.Node(document.Root()).Id(),
                &ComputedStyle::default(),
            );
            LayoutTreeUpdate::SetEditingNodeFacts(root, document.EditingNodeFacts(document.Root()));
            return;
        };
        let root_clip_resources = super::layout_object_tree_builder::svg_clip_resource_binding(
            source_style,
            &svg_clip_path_graphs,
            document.SVGResourceScope(index),
        );
        let resolved_resources = document.ResolvedStyleResourcesFor(index);
        let replaced_image_shader = None;
        let root_resources = layoutng_assembly::style_projection::StyleResourceBindings::FromParts(
            &svg_filter_graphs,
            &root_clip_resources,
            &resolved_resources.svg_mask_resource,
            resolved_resources.svg_pattern_content_target,
            &resolved_resources.svg_fill_server,
            &resolved_resources.svg_stroke_server,
            &resolved_resources.svg_markers,
            &replaced_image_shader,
        );
        let root = tree.CreateRootFromStyle(
            node.Id(),
            source_style,
            &root_resources,
            None,
            node.Name().into(),
            Some(ElementMetadata(
                document,
                index,
                interaction,
                source_style.ImageOrientation()
                    == foundation::RespectImageOrientationEnum::kRespectImageOrientation,
            )),
        ) as *mut LayoutObject;
        LayoutTreeUpdate::SetEditingNodeFacts(
            unsafe { &mut *root },
            document.EditingNodeFacts(index),
        );
        LayoutTreeUpdate::BeginElementLayoutTreeAttachment(unsafe { &mut *root });
        let mut first = resolved.first_letter.as_ref().and_then(|pseudo| {
            FindFirstLetterCandidate(document, index, source_style).map(|candidate| {
                FirstLetterInsertion {
                    originating_index: index,
                    originating_id: node.Id(),
                    svg_clip_resources: root_clip_resources.clone(),
                    pseudo,
                    candidate,
                    consumed: false,
                }
            })
        });
        AppendOverscrollAreaParents(
            &mut tree,
            document,
            interaction,
            index,
            root,
            &svg_filter_graphs,
            &svg_clip_path_graphs,
        );
        AppendPseudoElement(
            &mut tree,
            document,
            index,
            node.Id(),
            root,
            resolved.before.as_ref(),
            true,
            first.as_mut(),
            &svg_filter_graphs,
            &svg_clip_path_graphs,
        );
        AppendChildren(
            &mut tree,
            document,
            interaction,
            index,
            root,
            None,
            first.as_mut(),
            &svg_filter_graphs,
            &svg_clip_path_graphs,
        );
        AppendPseudoElement(
            &mut tree,
            document,
            index,
            node.Id(),
            root,
            resolved.after.as_ref(),
            false,
            first.as_mut(),
            &svg_filter_graphs,
            &svg_clip_path_graphs,
        );
        LayoutTreeUpdate::FinishElementLayoutTreeAttachment(unsafe { &mut *root });
    };
    // The projection borrows DOM style/metadata only during this callback.
    // The receiver must synchronously apply it to its owned source tree.
    receive(LayoutMutation::TreeUpdate(&mut projection));
}

// RebuildLayoutTree/WhitespaceAttacher roots retain complete attachment context.
// Inline/display:contents changes use their enclosing box; first-letter changes
// include the owning first-letter subtree, not an isolated text fragment.
fn AttachmentRoot(document: &Document, mut index: usize) -> Option<usize> {
    if !document.IsConnectedToDocument(index) {
        return None;
    }
    loop {
        if document.Node(index).Type() == DOMNodeType::kElement
            && document.ResolvedStyleFor(index).is_some_and(|s| {
                s.own_generates_box
                    && !s.display_contents
                    && unsafe { s.computed_style_owner.Get().as_ref() }
                        .is_some_and(|style| style.Display() != foundation::EDisplay::kInline)
            })
        {
            break;
        }
        index = document.FlatTreeParentElement(index)?;
    }
    let mut ancestor = document.FlatTreeParentElement(index);
    while let Some(parent) = ancestor {
        if document
            .ResolvedStyleFor(parent)
            .is_some_and(|s| s.first_letter.is_some())
        {
            index = parent;
        }
        ancestor = document.FlatTreeParentElement(parent);
    }
    Some(index)
}
fn IsWithinAttachmentRoot(document: &Document, root: usize, mut index: usize) -> bool {
    loop {
        if root == index {
            return true;
        }
        let Some(parent) = document.FlatTreeParentElement(index) else {
            return false;
        };
        index = parent;
    }
}

// Dirty roots cover their flat-tree descendants. Walk a node's ancestor path
// once, as attachment dirty flags do in Blink, rather than testing every root.
fn IsWithinAttachmentRoots(document: &Document, roots: &HashSet<usize>, mut index: usize) -> bool {
    if roots.is_empty() {
        return false;
    }
    loop {
        if roots.contains(&index) {
            return true;
        }
        let Some(parent) = document.FlatTreeParentElement(index) else {
            return false;
        };
        index = parent;
    }
}

pub(super) fn EmitLayoutSubtrees(
    owner: &DOM,
    interaction: &UserInteractionState,
    receive: &mut impl FnMut(LayoutMutation<'_>) -> bool,
) -> bool {
    let document = owner.GetDocument();
    let state = document.StyleState();
    if state.layout_inputs_require_tree {
        return false;
    }
    let mut select_trace = browser_tracing::span("layout", "DOM.SelectAttachmentRoots");
    select_trace.set("tree_roots", state.layout_tree_roots.len() as f64);
    select_trace.set("style_nodes", state.layout_style_nodes.len() as f64);
    select_trace.set("geometry_nodes", state.geometry_input_nodes.len() as f64);
    select_trace.set("reattach_nodes", state.layout_reattach_nodes.len() as f64);
    let mut roots = state
        .layout_tree_roots
        .iter()
        .filter_map(|&id| {
            document
                .FindNodeById(id)
                .and_then(|index| AttachmentRoot(document, index))
        })
        .collect::<Vec<_>>();
    let mut candidates = roots.iter().copied().collect::<HashSet<_>>();
    for id in state
        .layout_style_nodes
        .iter()
        .chain(&state.geometry_input_nodes)
        .chain(&state.layout_reattach_nodes)
    {
        let Some(index) = document
            .FindNodeById(*id)
            .filter(|&i| document.IsConnectedToDocument(i))
        else {
            continue;
        };
        if IsWithinAttachmentRoots(document, &candidates, index) {
            continue;
        }
        // Element::ReattachLayoutTree starts at the element itself. Missing
        // layout objects/anonymous contexts expand to the parent below; inline
        // and first-letter context already expands in AttachmentRoot.
        let Some(root) = AttachmentRoot(document, index) else {
            return false;
        };
        if candidates.insert(root) {
            roots.push(root);
        }
    }
    roots.sort_unstable();
    roots.dedup();
    roots.retain(|&root| {
        document
            .FlatTreeParentElement(root)
            .is_none_or(|parent| !IsWithinAttachmentRoots(document, &candidates, parent))
    });
    select_trace.set("selected_roots", roots.len() as f64);
    drop(select_trace);
    if roots.is_empty() {
        return false;
    }
    let resources_trace = browser_tracing::span("layout", "DOM.PrepareAttachmentResources");
    let filters = super::svg_filter_graph::build_persistent(document);
    let clips = document
        .SVGResources()
        .cloned()
        .unwrap_or_else(|| super::svg_clip_path_graph::build_persistent(document));
    drop(resources_trace);
    let _apply_trace = browser_tracing::span("layout", "DOM.ApplyAttachmentSubtrees");
    let mut reattach = state
        .layout_reattach_nodes
        .iter()
        .copied()
        .collect::<Vec<_>>();
    let mut completed = HashSet::new();
    for mut root in roots {
        if IsWithinAttachmentRoots(document, &completed, root) {
            continue;
        }
        loop {
            let Some(parent_index) = document.FlatTreeParentElement(root) else {
                return false;
            };
            let mut update = |tree: &mut LayoutTreeUpdate<'_>, parent: &mut LayoutObject| {
                tree.Reattach(&reattach);
                AppendSelectedChildren(
                    tree,
                    document,
                    interaction,
                    parent_index,
                    parent,
                    None,
                    None,
                    &filters,
                    &clips,
                    &[root],
                );
            };
            if receive(LayoutMutation::SubtreeUpdate {
                node_id: document.Node(root).Id(),
                parent_id: document.Node(parent_index).Id(),
                update: &mut update,
            }) {
                // A rejected narrow update can expand to a common ancestor.
                // That completed rebuild covers later roots and consumes its
                // reattach requests even if another root must expand as well.
                reattach.retain(|&id| {
                    document
                        .FindNodeById(id)
                        .is_none_or(|index| !IsWithinAttachmentRoot(document, root, index))
                });
                completed.insert(root);
                break;
            }
            // Anonymous/flattened source parents require a wider attachment
            // context. A rejected transaction made no change to the source tree.
            let Some(parent) = AttachmentRoot(document, parent_index) else {
                return false;
            };
            root = parent;
        }
    }
    true
}

/// Fixture adapter using the same mutation emission as the Page lifecycle.
/// The engine owns the source tree; DOM only supplies its projection.
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

    #[test]
    fn image_decoding_attribute_projects_typed_paint_preference() {
        let owner = crate::test_html::html_parser::ParseHTML(
            "<img id='sync' decoding='SYNC'><img id='async' decoding='async'><img id='bad' decoding='later'>",
        );
        let document = owner.GetDocument();
        let interaction = UserInteractionState::default();
        let mode = |id| {
            ElementMetadata(
                document,
                document.FindNodeById(id).unwrap(),
                &interaction,
                true,
            )
            .image_decoding_mode
        };
        assert_eq!(mode("sync"), image_resource::ImageDecodingMode::kSync);
        assert_eq!(mode("async"), image_resource::ImageDecodingMode::kAsync);
        assert_eq!(mode("bad"), image_resource::ImageDecodingMode::kUnspecified);
    }

    #[test]
    fn resolves_referenced_text_path_on_the_persistent_dom() {
        let owner = crate::test_html::html_parser::ParseHTML(
            "<svg><path id='route' d='M0 0 L200 0'></path><text><textPath id='label' \
             href='#route' startOffset='25%' side='RIGHT'>x</textPath></text></svg>",
        );
        let document = owner.GetDocument();
        let label = document.FindNodeById("label").unwrap();
        let path = ResolveSVGTextPath(document, label).unwrap();
        assert_eq!(path.points.len(), 2);
        assert_eq!(path.start_offset, 50.0);
        assert!(path.reverse_direction);

        let invalid = crate::test_html::html_parser::ParseHTML(
            "<svg><path id='route' d='M0 0 L10 0 M20 0 L30 0'></path><text>\
             <textPath id='label' href='#route'>x</textPath></text></svg>",
        );
        let document = invalid.GetDocument();
        let label = document.FindNodeById("label").unwrap();
        assert!(ResolveSVGTextPath(document, label).is_none());
    }

    #[test]
    fn details_attachment_matches_manual_summary_and_content_slots() {
        let mut owner = crate::test_html::html_parser::ParseHTML(
            "<details id='closed'><p id='before'></p><summary id='closed-summary'></summary><p id='closed-content'></p></details><details id='open' open><p id='open-before'></p><summary id='open-summary'></summary><p id='open-content'></p></details>",
        );
        let document = owner.GetDocumentMut();
        let ids: BTreeMap<_, _> = (0..document.NodeCount())
            .filter_map(|index| {
                document
                    .Node(index)
                    .FindAttribute("id")
                    .map(|attribute| (attribute.value.clone(), index))
            })
            .collect();

        assert_eq!(
            AttachmentChildren(document, ids["closed"]).as_ref(),
            &[ids["closed-summary"]]
        );
        assert_eq!(
            AttachmentChildren(document, ids["open"]).as_ref(),
            &[ids["open-summary"], ids["open-before"], ids["open-content"]]
        );
    }
    fn dump(
        document: &Document,
        index: usize,
        interaction: &UserInteractionState,
    ) -> Vec<Option<f64>> {
        let fallback = ComputedStyle::default();
        let s = document
            .ResolvedStyleFor(index)
            .map_or(&fallback, |r| &r.style);
        let d = ElementMetadata(
            document,
            index,
            interaction,
            s.paint.respect_image_orientation,
        );
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
        n!(ElementKind(document, index, &d, s, None) as i32);
        n!(d.scroll_offset.x);
        n!(d.scroll_offset.y);
        n!(d.nowrap_attribute);
        n!(d.html_anchor);
        n!(d.html_canvas);
        n!(d.html_hr);
        n!(d.element_canvas_transform_enabled);
        n!(d.used_canvas_transform.is_some());
        if let Some(matrix) = &d.used_canvas_transform {
            for value in matrix.values {
                n!(value);
            }
        }
        n!(d.html_media);
        n!(d.html_video);
        optional!(d.video_frame_resource_id);
        n!(d.canvas_layout_subtree);
        n!(d.drawable_attribute);
        n!(d.html_image);
        n!(d.html_ordered_or_unordered_list);
        n!(d.html_list_owner);
        n!(d.ordered_list_reversed);
        optional!(d.ordered_list_start);
        optional!(d.list_item_explicit_value);
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
        optional!(d.svg_clip_path_object_bounding_box);
        optional!(d.svg_path_length);
        n!(d.svg_path_element);
        n!(d.svg_marker.is_some());
        if let Some(marker) = &d.svg_marker {
            // The fixture compares authored values across independently built
            // metadata owners. Serialization is used only by this test dump.
            for length in [
                &marker.lengths.width,
                &marker.lengths.height,
                &marker.lengths.ref_x,
                &marker.lengths.ref_y,
            ] {
                let text = length.CssText().Utf8();
                n!(text.len());
                for byte in text.bytes() {
                    n!(byte as u32);
                }
            }
            n!(marker.container_sizes.is_some());
            if let Some(sizes) = marker.container_sizes {
                optional!(sizes.Width());
                optional!(sizes.Height());
            }
            n!(marker.stroke_width_units);
            match marker.orient {
                layoutng_assembly::internal::layout_input::SvgMarkerOrient::Angle(v) => {
                    n!(0);
                    n!(v);
                }
                layoutng_assembly::internal::layout_input::SvgMarkerOrient::Auto => n!(1),
                layoutng_assembly::internal::layout_input::SvgMarkerOrient::AutoStartReverse => {
                    n!(2)
                }
            }
        }

        n!(d.svg_filter_image_targets.len());
        for target in &d.svg_filter_image_targets {
            n!(*target);
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
    fn nested_svg_metadata_preserves_viewport_lengths_and_parent_boundaries() {
        let mut owner = crate::test_html::html_parser::ParseHTML(
            "<html><body><svg id='outer' width='200' height='100'>\
             <g><svg id='nested' x='10%' y='2.54cm' width='50%' height='20' \
             viewBox='0 0 100 0' preserveAspectRatio='none'></svg></g>\
             <svg id='defaults'></svg>\
             <foreignObject><svg id='foreign' width='40' height='30'></svg></foreignObject>\
             </svg></body></html>",
        );
        let ids: BTreeMap<_, _> = (0..owner.GetDocument().NodeCount())
            .filter_map(|index| {
                owner
                    .GetDocument()
                    .Node(index)
                    .FindAttribute("id")
                    .map(|attr| (attr.value.clone(), index))
            })
            .collect();
        let interaction = UserInteractionState::default();
        let style = ComputedStyle::default();
        let document = owner.GetDocument();
        for id in ["outer", "foreign"] {
            let data = ElementMetadata(document, ids[id], &interaction, true);
            assert_eq!(
                ElementKind(document, ids[id], &data, &style, None),
                NodeKind::kSvgRoot
            );
            assert!(data.natural_width.is_some());
            assert!(data.svg_viewport.is_none());
        }
        let nested = ElementMetadata(document, ids["nested"], &interaction, true);
        assert_eq!(
            ElementKind(document, ids["nested"], &nested, &style, None),
            NodeKind::kSvgViewportContainer
        );
        assert_eq!(nested.natural_width, None);
        assert_eq!(nested.natural_height, None);
        assert_eq!(nested.natural_aspect_ratio, None);
        assert_eq!(
            nested.svg_viewport,
            Some(SvgViewportData {
                x: SvgViewportLength::kPercentage(10.0),
                y: SvgViewportLength::kUserUnits(96.0),
                width: SvgViewportLength::kPercentage(50.0),
                height: SvgViewportLength::kUserUnits(20.0),
            })
        );
        assert_eq!(nested.svg_view_box.unwrap().height, 0.0);
        assert!(nested.svg_view_box.unwrap().preserve_none);
        let defaults = ElementMetadata(document, ids["defaults"], &interaction, true);
        assert_eq!(defaults.svg_viewport, Some(SvgViewportData::default()));
        let nested_id = document.Node(ids["nested"]).Id();
        owner
            .ApplyMutation(&crate::dom_mutation::DOMMutation {
                mutation_type: crate::dom_mutation::DOMMutationType::kSetAttribute,
                target_node_id: nested_id,
                name: "width".into(),
                value: "75%".into(),
                ..Default::default()
            })
            .unwrap();
        let changed = ElementMetadata(owner.GetDocument(), ids["nested"], &interaction, true);
        assert_eq!(
            changed.svg_viewport.unwrap().width,
            SvgViewportLength::kPercentage(75.0)
        );
        assert!(nested != changed);
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
        let absolute = ElementMetadata(document, find("absolute"), &interaction, true);
        assert_eq!(absolute.natural_width, Some(50.0));
        assert_eq!(absolute.natural_height, Some(58.0));
        assert_eq!(absolute.natural_aspect_ratio, Some(50.0 / 58.0));
        let percent = ElementMetadata(document, find("percent"), &interaction, true);
        assert_eq!(percent.natural_width, None);
        assert_eq!(percent.natural_height, None);
        assert_eq!(percent.natural_aspect_ratio, Some(2.0));
    }
    #[test]
    fn geometry_only_input_for_suppressed_element_does_not_reattach_ancestor() {
        let _heap = foundation::LayoutHeapScope::new();
        fn resolved_style(
            display: foundation::EDisplay,
            generates_box: bool,
        ) -> crate::persistent_document::ResolvedNodeStyle {
            let initial = unsafe { &*StyleComputedStyle::GetInitialStyleSingleton() };
            let mut builder = ComputedStyleBuilder::from_style(initial);
            builder.SetDisplay(display);
            let mut resolved = crate::persistent_document::ResolvedNodeStyle::default();
            resolved.generates_box = generates_box;
            resolved.own_generates_box = generates_box;
            resolved.SetComputedStyleOwner(foundation::Persistent::from_ptr(builder.TakeStyle()));
            resolved
        }

        let mut owner = crate::test_html::html_parser::ParseHTML(
            "<html><body><div id='suppressed'></div><div id='resident'></div></body></html>",
        );
        let suppressed = owner.GetDocument().FindNodeById("suppressed").unwrap();
        let suppressed_id = owner.GetDocument().Node(suppressed).Id();
        let document = owner.GetDocumentMut();
        for index in 0..document.NodeCount() {
            if document.Node(index).Type() == DOMNodeType::kElement {
                document.SetResolvedStyle(
                    index,
                    if index == suppressed {
                        resolved_style(foundation::EDisplay::kNone, false)
                    } else {
                        resolved_style(foundation::EDisplay::kBlock, true)
                    },
                );
            }
        }

        let assembly = layoutng_assembly::layout_assembly::LayoutAssembly::default();
        let mut engine = LayoutEngine::new(&assembly);
        let interaction = UserInteractionState::default();
        owner.EmitLayoutMutations(&interaction, |mutation| engine.ApplyMutation(mutation));
        let state = owner.GetDocument().StyleState();
        assert!(state.layout_tree_roots.is_empty());
        assert!(state.layout_reattach_nodes.is_empty());
        assert!(state.layout_style_nodes.is_empty());

        owner
            .GetDocumentMut()
            .StyleStateMut()
            .geometry_input_nodes
            .insert(suppressed_id);
        let mut received = 0;
        owner.EmitLayoutMutations(&interaction, |mutation| {
            received += 1;
            engine.ApplyMutation(mutation)
        });

        assert_eq!(
            received, 0,
            "a missing LayoutObject has no geometry consumer"
        );
        assert!(owner
            .GetDocument()
            .StyleState()
            .geometry_input_nodes
            .is_empty());
    }

    #[test]
    fn emitted_dom_projection_updates_only_when_received_and_reuses_siblings() {
        let _heap = foundation::LayoutHeapScope::new();
        fn resolved_style(
            width: Option<f64>,
            height: Option<f64>,
        ) -> crate::persistent_document::ResolvedNodeStyle {
            let initial = unsafe { &*StyleComputedStyle::GetInitialStyleSingleton() };
            let mut builder = ComputedStyleBuilder::from_style(initial);
            builder.SetDisplay(foundation::EDisplay::kBlock);
            if let Some(width) = width {
                builder.SetWidth(&foundation::Length::Fixed(width));
            }
            if let Some(height) = height {
                builder.SetHeight(&foundation::Length::Fixed(height));
            }
            let mut resolved = crate::persistent_document::ResolvedNodeStyle::default();
            resolved.SetComputedStyleOwner(foundation::Persistent::from_ptr(builder.TakeStyle()));
            resolved
        }
        fn source(root: *mut LayoutObject, id: u64) -> *mut LayoutObject {
            let mut object = root;
            while !object.is_null() {
                let node = unsafe { &*object }.GetNode();
                if !node.is_null() && unsafe { &*node }.InputId() == id {
                    return object;
                }
                object = unsafe { &*object }.NextInPreOrder(root);
            }
            panic!("missing source node {id}");
        }
        fn width(object: *mut LayoutObject) -> Option<f64> {
            let node = unsafe { &*object }.GetNode();
            unsafe { &*node }.InputStyle().width
        }
        let mut owner = crate::test_html::html_parser::ParseHTML(
            "<html><body><div id='changed' style='display:block;width:80px;height:30px'></div><div id='sibling' style='display:block;width:40px;height:20px'></div></body></html>",
        );
        let ids: BTreeMap<_, _> = (0..owner.GetDocument().NodeCount())
            .filter_map(|index| {
                owner
                    .GetDocument()
                    .Node(index)
                    .FindAttribute("id")
                    .map(|attr| (attr.value.clone(), index))
            })
            .collect();
        // This test exercises the DOM-to-layout projection boundary. Style
        // calculation is supplied by the browser's StyleEngine lifecycle.
        let document = owner.GetDocumentMut();
        for index in 0..document.NodeCount() {
            if document.Node(index).Type() == DOMNodeType::kElement {
                document.SetResolvedStyle(index, resolved_style(None, None));
            }
        }
        for (id, width, height) in [("changed", 80.0, 30.0), ("sibling", 40.0, 20.0)] {
            let index = ids[id];
            document.SetResolvedStyle(index, resolved_style(Some(width), Some(height)));
        }
        let changed_id = owner.GetDocument().Node(ids["changed"]).Id();
        let sibling_id = owner.GetDocument().Node(ids["sibling"]).Id();
        let assembly = layoutng_assembly::layout_assembly::LayoutAssembly::default();
        let mut engine = LayoutEngine::new(&assembly);
        let interaction = UserInteractionState::default();
        owner.EmitLayoutMutations(&interaction, |mutation| engine.ApplyMutation(mutation));
        let root =
            engine.GetLayoutTree().unwrap().Root() as *const LayoutObject as *mut LayoutObject;
        let changed = source(root, changed_id);
        let sibling = source(root, sibling_id);
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
        let document = owner.GetDocumentMut();
        document.SetResolvedStyle(ids["changed"], resolved_style(Some(120.0), Some(30.0)));
        assert_eq!(
            unsafe {
                &*owner
                    .GetDocument()
                    .ResolvedStyleFor(ids["changed"])
                    .unwrap()
                    .computed_style_owner
                    .Get()
            }
            .Width()
            .Pixels(),
            120.0
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
            assert_eq!(source(root, sibling_id), sibling);
            engine.ApplyMutation(mutation)
        });
        assert_eq!(received, 1);
        let updated_root =
            engine.GetLayoutTree().unwrap().Root() as *const LayoutObject as *mut LayoutObject;
        assert_eq!(updated_root, root);
        assert_eq!(source(updated_root, changed_id), changed);
        assert_eq!(width(changed), Some(120.0));
        assert_eq!(source(updated_root, sibling_id), sibling);
        assert_eq!(unsafe { &*sibling }.StyleRef() as *const _, sibling_style);
        assert_eq!(engine.GetLayoutTree().unwrap().UpdateStats().created, 0);
    }
    #[test]
    fn persistent_metadata_and_namespace_kinds_match_cpp_including_live_values() {
        // This end-to-end reference test still requires browser assembly of
        // the concrete Chromium style backend and the cpp-reference fixtures.
        // Keep its assertions visible until that assembly can run them.
        let mut owner = crate::test_html::html_parser::ParseHTML(include_str!(
            "../../../../artifacts/cpp-reference/persistent-tree-metadata.html"
        ));
        owner
            .GetDocumentMut()
            .AppendStyleSheet(style::ParseCSS(include_str!(
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
                orientation: Default::default(),
                known_to_be_opaque: false,
            },
        );
        let interaction = UserInteractionState {
            hovered_node_id: Some(owner.GetDocument().Node(nodes["range"]).Id()),
            pressed_node_id: Some(owner.GetDocument().Node(nodes["check"]).Id()),
            focus_visible_node_id: Some(owner.GetDocument().Node(nodes["input"]).Id()),
            ..Default::default()
        };
        owner
            .GetDocumentMut()
            .SetUserActionTargets(interaction.hovered_node_id, interaction.pressed_node_id);
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
            let mut style_engine = style::StyleEngine::new(&owner);
            style_engine
                .Update(
                    &mut owner,
                    &style::media_queries::media_values_cached::MediaValuesCachedData {
                        viewport_width: 1024.0,
                        viewport_height: 768.0,
                        small_viewport_width: 1024.0,
                        small_viewport_height: 768.0,
                        large_viewport_width: 1024.0,
                        large_viewport_height: 768.0,
                        dynamic_viewport_width: 1024.0,
                        dynamic_viewport_height: 768.0,
                        device_pixel_ratio: 1.0,
                        em_size: 16.0,
                        ..Default::default()
                    },
                    &[],
                )
                .expect("style update");
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
