#![allow(non_snake_case, non_camel_case_types)]
use crate::style_resolver::background::BackgroundCascadeState;
use crate::style_resolver::border_longhands::BorderSide;
use crate::style_resolver::selector::SourceSpace;
use layoutng_assembly::internal::layout_input::{
    ComputedStyle, Edges, ExtendedStyle, TextDirection, WritingMode,
};

// cpp: style_resolver/style_resolver.cc:4518-4536
#[derive(Clone, Copy)]
pub(crate) enum LogicalSide {
    kInlineStart,
    kInlineEnd,
    kBlockStart,
    kBlockEnd,
}
pub(crate) fn PhysicalSideName(
    side: LogicalSide,
    writing_mode: WritingMode,
    direction: TextDirection,
) -> &'static str {
    use LogicalSide::*;
    if writing_mode == WritingMode::kHorizontalTb {
        if matches!(side, kBlockStart) {
            return "top";
        }
        if matches!(side, kBlockEnd) {
            return "bottom";
        }
        return if matches!(side, kInlineStart) == (direction == TextDirection::kLtr) {
            "left"
        } else {
            "right"
        };
    }
    if matches!(side, kInlineStart) {
        return if direction == TextDirection::kLtr {
            "top"
        } else {
            "bottom"
        };
    }
    if matches!(side, kInlineEnd) {
        return if direction == TextDirection::kLtr {
            "bottom"
        } else {
            "top"
        };
    }
    if writing_mode == WritingMode::kVerticalRl {
        return if matches!(side, kBlockStart) {
            "right"
        } else {
            "left"
        };
    }
    if matches!(side, kBlockStart) {
        "left"
    } else {
        "right"
    }
}
// cpp: style_resolver/style_resolver.cc:4538-4580
pub(crate) fn MapLogicalProperty(
    property: &str,
    writing_mode: WritingMode,
    direction: TextDirection,
) -> String {
    // CSSProperty::ToPhysical switches on a resolved property ID. This string
    // boundary must likewise classify the input before constructing its one
    // physical name, rather than allocating every candidate for each property.
    let size = match property {
        "inline-size" => Some((false, "")),
        "block-size" => Some((true, "")),
        "min-inline-size" => Some((false, "min-")),
        "min-block-size" => Some((true, "min-")),
        "max-inline-size" => Some((false, "max-")),
        "max-block-size" => Some((true, "max-")),
        "contain-intrinsic-inline-size" => Some((false, "contain-intrinsic-")),
        "contain-intrinsic-block-size" => Some((true, "contain-intrinsic-")),
        _ => None,
    };
    if let Some((block, prefix)) = size {
        let axis = if (writing_mode == WritingMode::kHorizontalTb) != block {
            "width"
        } else {
            "height"
        };
        return format!("{prefix}{axis}");
    }
    let (prefix, logical, suffix) = if let Some(logical) = property.strip_prefix("margin-") {
        ("margin-", logical, "")
    } else if let Some(logical) = property.strip_prefix("padding-") {
        ("padding-", logical, "")
    } else if let Some(logical) = property.strip_prefix("inset-") {
        ("", logical, "")
    } else if let Some(logical) = property.strip_prefix("border-") {
        let (logical, suffix) = ["-width", "-style", "-color"]
            .iter()
            .copied()
            .find_map(|suffix| {
                logical
                    .strip_suffix(suffix)
                    .map(|logical| (logical, suffix))
            })
            .unwrap_or((logical, ""));
        ("border-", logical, suffix)
    } else {
        return property.into();
    };
    let side = match logical {
        "inline-start" => LogicalSide::kInlineStart,
        "inline-end" => LogicalSide::kInlineEnd,
        "block-start" => LogicalSide::kBlockStart,
        "block-end" => LogicalSide::kBlockEnd,
        _ => return property.into(),
    };
    let physical = PhysicalSideName(side, writing_mode, direction);
    format!("{prefix}{physical}{suffix}")
}
// cpp: style_resolver/style_resolver.cc:3809-3816
fn EdgeAt(edge: &mut Edges, side: usize) -> &mut f64 {
    match side {
        0 => &mut edge.top,
        1 => &mut edge.right,
        2 => &mut edge.bottom,
        _ => &mut edge.left,
    }
}
fn EdgeValue(edge: &Edges, side: usize) -> f64 {
    match side {
        0 => edge.top,
        1 => edge.right,
        2 => edge.bottom,
        _ => edge.left,
    }
}
// cpp: style_resolver/style_resolver.cc:6258-6262
#[derive(Clone, Copy)]
pub(crate) struct CSSWideSource<'a> {
    pub style: &'a ComputedStyle,
    pub generates_box: bool,
    pub display_contents: bool,
}
impl<'a> CSSWideSource<'a> {
    pub(crate) fn new(style: &'a ComputedStyle) -> Self {
        Self {
            style,
            generates_box: true,
            display_contents: false,
        }
    }
}
// cpp: style_resolver/style_resolver.cc:6264-6369
pub(crate) fn CopyCSSProperty(
    property: &str,
    writing_mode: WritingMode,
    direction: TextDirection,
    source: &CSSWideSource<'_>,
    destination: &mut ComputedStyle,
    generates_box: &mut bool,
    display_contents: &mut bool,
    background: &mut BackgroundCascadeState,
) -> bool {
    let logical_pair = |inline_axis: bool| {
        let (start, end) = if inline_axis {
            (LogicalSide::kInlineStart, LogicalSide::kInlineEnd)
        } else {
            (LogicalSide::kBlockStart, LogicalSide::kBlockEnd)
        };
        [
            PhysicalSideName(start, writing_mode, direction),
            PhysicalSideName(end, writing_mode, direction),
        ]
    };
    for prefix in ["margin", "padding", "inset"] {
        for axis in ["inline", "block"] {
            if property != format!("{prefix}-{axis}") {
                continue;
            }
            let sides = logical_pair(axis == "inline");
            let physical_prefix = if prefix == "inset" {
                String::new()
            } else {
                format!("{prefix}-")
            };
            for side in sides {
                CopyCSSProperty(
                    &format!("{physical_prefix}{side}"),
                    writing_mode,
                    direction,
                    source,
                    destination,
                    generates_box,
                    display_contents,
                    background,
                );
            }
            return true;
        }
    }
    for axis in ["inline", "block"] {
        let sides = logical_pair(axis == "inline");
        let prefix = format!("border-{axis}");
        for suffix in ["", "-width", "-style", "-color"] {
            if property != format!("{prefix}{suffix}") {
                continue;
            }
            for side in sides {
                CopyCSSProperty(
                    &format!("border-{side}{suffix}"),
                    writing_mode,
                    direction,
                    source,
                    destination,
                    generates_box,
                    display_contents,
                    background,
                );
            }
            return true;
        }
    }
    let property = MapLogicalProperty(property, writing_mode, direction);
    if property == "all" {
        let saved_direction = destination.direction;
        let saved_image = destination.paint.image_resource_id;
        let to = destination.extended.get_or_insert_with(Default::default);
        let (saved_bidi, saved_language, saved_appearance, saved_zoom) = (
            to.unicode_bidi,
            to.language.clone(),
            to.effective_appearance,
            to.effective_zoom,
        );
        *destination = source.style.clone();
        destination.direction = saved_direction;
        let to = destination.extended.get_or_insert_with(Default::default);
        to.unicode_bidi = saved_bidi;
        to.language = saved_language;
        to.effective_appearance = saved_appearance;
        to.effective_zoom = saved_zoom;
        destination.paint.image_resource_id = saved_image;
        *generates_box = source.generates_box;
        *display_contents = source.display_contents;
        background.Import(&destination.paint);
        return true;
    }
    let defaults = ExtendedStyle::default();
    let from = source.style.extended.as_ref().unwrap_or(&defaults);
    let to = destination.extended.get_or_insert_with(Default::default);
    let from_paint = &*source.style.paint;
    let to_paint = &mut *destination.paint;
    macro_rules! copy_dimension {
        ($field:ident,$percent:ident,$calculated:ident) => {{
            destination.$field = source.style.$field;
            to.$percent = from.$percent;
            to.$calculated = from.$calculated;
        }};
    }
    macro_rules! copy_edge {
        ($margin:expr,$side:expr) => {{
            let margin = $margin;
            let side = $side;
            *EdgeAt(
                if margin {
                    &mut destination.margin
                } else {
                    &mut destination.padding
                },
                side,
            ) = EdgeValue(
                if margin {
                    &source.style.margin
                } else {
                    &source.style.padding
                },
                side,
            );
            if margin {
                to.margin_percentages[side] = from.margin_percentages[side];
                to.margin_calculated[side] = from.margin_calculated[side];
                to.margin_auto[side] = from.margin_auto[side];
            } else {
                to.padding_percentages[side] = from.padding_percentages[side];
                to.padding_calculated[side] = from.padding_calculated[side];
            }
        }};
    }
    macro_rules! copy_border_side {
        ($side:expr,$width:expr,$line:expr,$color:expr) => {{
            let side = $side;
            if $width {
                *EdgeAt(&mut destination.border, side) = EdgeValue(&source.style.border, side);
            }
            if $line {
                destination.border_styles[side] = source.style.border_styles[side];
            }
            if $color {
                to_paint.border_colors[side] = from_paint.border_colors[side];
            }
        }};
    }
    // cpp: style_resolver/style_resolver.cc:6370-6880
    if property == "display" {
        destination.display = source.style.display.clone();
        *generates_box = source.generates_box;
        *display_contents = source.display_contents;
    } else if property == "zoom" {
        to.zoom = from.zoom;
    } else if property == "width" {
        copy_dimension!(width, width_percent, width_calculated);
        to.width_sizing = from.width_sizing.clone();
    } else if property == "height" {
        copy_dimension!(height, height_percent, height_calculated);
        to.height_sizing = from.height_sizing.clone();
    } else if property == "min-width" {
        copy_dimension!(min_width, min_width_percent, min_width_calculated);
        to.min_width_sizing = from.min_width_sizing.clone();
    } else if property == "max-width" {
        copy_dimension!(max_width, max_width_percent, max_width_calculated);
        to.max_width_sizing = from.max_width_sizing.clone();
    } else if property == "min-height" {
        copy_dimension!(min_height, min_height_percent, min_height_calculated);
        to.min_height_sizing = from.min_height_sizing.clone();
    } else if property == "max-height" {
        copy_dimension!(max_height, max_height_percent, max_height_calculated);
        to.max_height_sizing = from.max_height_sizing.clone();
    } else if property == "flex-basis" {
        copy_dimension!(flex_basis, flex_basis_percent, flex_basis_calculated);
        to.flex_basis_sizing = from.flex_basis_sizing.clone();
    } else if property == "margin" || property == "padding" {
        let margin = property == "margin";
        for side in 0..4 {
            copy_edge!(margin, side);
        }
    } else if property.starts_with("margin-") || property.starts_with("padding-") {
        let margin = property.starts_with("margin-");
        let side = if property.ends_with("top") {
            0
        } else if property.ends_with("right") {
            1
        } else if property.ends_with("bottom") {
            2
        } else {
            3
        };
        copy_edge!(margin, side);
    } else if property == "border" {
        for side in 0..4 {
            copy_border_side!(side, true, true, true);
        }
    } else if property == "border-width" || property == "border-style" || property == "border-color"
    {
        for side in 0..4 {
            copy_border_side!(
                side,
                property == "border-width",
                property == "border-style",
                property == "border-color"
            );
        }
    } else if (property == "border-top"
        || property == "border-right"
        || property == "border-bottom"
        || property == "border-left"
        || property.ends_with("-width")
        || property.ends_with("-style")
        || property.ends_with("-color"))
        && property.starts_with("border-")
        && BorderSide(&property).is_some()
    {
        let side = BorderSide(&property).unwrap();
        let width = property.ends_with("-width");
        let line = property.ends_with("-style");
        let color = property.ends_with("-color");
        copy_border_side!(
            side,
            width || (!line && !color),
            line || (!width && !color),
            color || (!width && !line)
        );
    } else if property == "outline" {
        to_paint.outline_width = from_paint.outline_width.clone();
        to_paint.outline_style = from_paint.outline_style.clone();
        to_paint.outline_color = from_paint.outline_color.clone();
    } else if property == "outline-width" {
        to_paint.outline_width = from_paint.outline_width.clone();
    } else if property == "outline-style" {
        to_paint.outline_style = from_paint.outline_style.clone();
    } else if property == "outline-color" {
        to_paint.outline_color = from_paint.outline_color.clone();
    } else if property == "outline-offset" {
        to_paint.outline_offset = from_paint.outline_offset.clone();
    } else if property == "flex" {
        destination.flex_basis = source.style.flex_basis.clone();
        destination.flex_grow = source.style.flex_grow.clone();
        destination.flex_shrink = source.style.flex_shrink.clone();
        to.flex_basis_percent = from.flex_basis_percent.clone();
        to.flex_basis_calculated = from.flex_basis_calculated.clone();
    } else if property == "flex-grow" {
        destination.flex_grow = source.style.flex_grow.clone();
    } else if property == "flex-shrink" {
        destination.flex_shrink = source.style.flex_shrink.clone();
    } else if property == "gap" {
        destination.gap = source.style.gap.clone();
        to.row_gap = from.row_gap.clone();
        to.column_gap = from.column_gap.clone();
        to.row_gap_percent = from.row_gap_percent.clone();
        to.column_gap_percent = from.column_gap_percent.clone();
        to.row_gap_calculated = from.row_gap_calculated.clone();
        to.column_gap_calculated = from.column_gap_calculated.clone();
    } else if property == "row-gap" {
        to.row_gap = from.row_gap.clone();
        to.row_gap_percent = from.row_gap_percent.clone();
        to.row_gap_calculated = from.row_gap_calculated.clone();
    } else if property == "column-gap" {
        to.column_gap = from.column_gap.clone();
        to.column_gap_percent = from.column_gap_percent.clone();
        to.column_gap_calculated = from.column_gap_calculated.clone();
    } else if property == "order" {
        to.order = from.order.clone();
    } else if property == "flex-direction" {
        destination.flex_direction = source.style.flex_direction.clone();
    } else if property == "flex-wrap" {
        destination.flex_wrap = source.style.flex_wrap.clone();
        to.wrap_reverse = from.wrap_reverse.clone();
    } else if property == "flex-flow" {
        destination.flex_direction = source.style.flex_direction.clone();
        destination.flex_wrap = source.style.flex_wrap.clone();
        to.wrap_reverse = from.wrap_reverse.clone();
    } else if property == "align-items" {
        destination.align_items = source.style.align_items.clone();
        to.align_items_overflow = from.align_items_overflow.clone();
    } else if property == "align-content" {
        to.align_content = from.align_content.clone();
        to.align_content_overflow = from.align_content_overflow.clone();
    } else if property == "-internal-align-content-block" {
        to.align_content_block_center = from.align_content_block_center.clone();
    } else if property == "align-self" {
        to.align_self = from.align_self.clone();
        to.align_self_overflow = from.align_self_overflow.clone();
    } else if property == "justify-items" {
        to.justify_items = from.justify_items.clone();
        to.justify_items_overflow = from.justify_items_overflow.clone();
        to.justify_items_legacy = from.justify_items_legacy.clone();
    } else if property == "justify-self" {
        to.justify_self = from.justify_self.clone();
        to.justify_self_overflow = from.justify_self_overflow.clone();
    } else if property == "justify-content" {
        to.justify_content = from.justify_content.clone();
        to.justify_content_overflow = from.justify_content_overflow.clone();
    } else if property == "direction" {
        destination.direction = source.style.direction.clone();
    } else if property == "writing-mode" {
        destination.writing_mode = source.style.writing_mode.clone();
    } else if property == "position" {
        destination.position = source.style.position.clone();
    } else if property == "z-index" {
        to_paint.z_index = from_paint.z_index.clone();
    } else if property == "inset" {
        destination.top = source.style.top.clone();
        destination.right = source.style.right.clone();
        destination.bottom = source.style.bottom.clone();
        destination.left = source.style.left.clone();
        to.inset_percentages = from.inset_percentages.clone();
        to.inset_calculated = from.inset_calculated.clone();
    } else if property == "left" || property == "top" || property == "right" || property == "bottom"
    {
        let side = match property.as_str() {
            "top" => 0,
            "right" => 1,
            "bottom" => 2,
            _ => 3,
        };
        match property.as_str() {
            "left" => destination.left = source.style.left,
            "top" => destination.top = source.style.top,
            "right" => destination.right = source.style.right,
            _ => destination.bottom = source.style.bottom,
        };
        to.inset_percentages[side] = from.inset_percentages[side];
        to.inset_calculated[side] = from.inset_calculated[side];
    } else if property == "float" {
        destination.floating = source.style.floating.clone();
    } else if property == "shape-outside" {
        to.shape_outside = from.shape_outside.clone();
    } else if property == "shape-margin" {
        to.shape_margin = from.shape_margin.clone();
    } else if property == "shape-image-threshold" {
        to.shape_image_threshold = from.shape_image_threshold.clone();
    } else if property == "columns" {
        destination.column_count = source.style.column_count.clone();
        to.explicit_column_count = from.explicit_column_count.clone();
        to.column_width = from.column_width.clone();
    } else if property == "column-count" {
        destination.column_count = source.style.column_count.clone();
        to.explicit_column_count = from.explicit_column_count.clone();
    } else if property == "column-width" {
        to.column_width = from.column_width.clone();
    } else if property == "grid-template-columns" {
        destination.grid_template_columns = source.style.grid_template_columns.clone();
        to.grid_columns = from.grid_columns.clone();
        to.subgrid_columns = from.subgrid_columns.clone();
    } else if property == "grid-template-rows" {
        to.grid_rows = from.grid_rows.clone();
        to.subgrid_rows = from.subgrid_rows.clone();
    } else if property == "grid-template-areas" {
        to.grid_template_areas = from.grid_template_areas.clone();
    } else if property == "grid-auto-columns" {
        to.grid_auto_columns = from.grid_auto_columns.clone();
    } else if property == "grid-auto-rows" {
        to.grid_auto_rows = from.grid_auto_rows.clone();
    } else if property == "grid-auto-flow" {
        to.grid_auto_flow_column = from.grid_auto_flow_column.clone();
        to.grid_auto_flow_dense = from.grid_auto_flow_dense.clone();
    } else if property == "grid-column" {
        to.grid_column_start = from.grid_column_start.clone();
        to.grid_column_end = from.grid_column_end.clone();
    } else if property == "grid-row" {
        to.grid_row_start = from.grid_row_start.clone();
        to.grid_row_end = from.grid_row_end.clone();
    } else if property == "grid-area" {
        to.grid_row_start = from.grid_row_start.clone();
        to.grid_column_start = from.grid_column_start.clone();
        to.grid_row_end = from.grid_row_end.clone();
        to.grid_column_end = from.grid_column_end.clone();
    } else if property == "grid-column-start" {
        to.grid_column_start = from.grid_column_start.clone();
    } else if property == "grid-column-end" {
        to.grid_column_end = from.grid_column_end.clone();
    } else if property == "grid-row-start" {
        to.grid_row_start = from.grid_row_start.clone();
    } else if property == "grid-row-end" {
        to.grid_row_end = from.grid_row_end.clone();
    } else if property == "table-layout" {
        to.table_layout_fixed = from.table_layout_fixed.clone();
    } else if property == "empty-cells" {
        to.empty_cells = from.empty_cells.clone();
    } else if property == "margin-trim" {
        to.margin_trim = from.margin_trim.clone();
    } else if property == "overflow-clip-margin" {
        to.overflow_clip_margin = from.overflow_clip_margin.clone();
    } else if property == "border-collapse" {
        to.border_collapse = from.border_collapse.clone();
    } else if property == "caption-side" {
        to.caption_side = from.caption_side.clone();
    } else if property == "border-spacing" {
        to.border_spacing = from.border_spacing.clone();
        to.vertical_border_spacing = from.vertical_border_spacing.clone();
    } else if property == "column-span" {
        to.column_span_all = from.column_span_all.clone();
    } else if property == "column-fill" {
        to.column_fill_balance = from.column_fill_balance.clone();
    } else if property == "column-wrap" {
        to.column_wrap = from.column_wrap.clone();
    } else if property == "initial-letter" {
        to.initial_letter = from.initial_letter.clone();
    } else if property == "break-before" {
        to.break_before = from.break_before.clone();
    } else if property == "break-after" {
        to.break_after = from.break_after.clone();
    } else if property == "break-inside" {
        to.break_inside = from.break_inside.clone();
    } else if property == "widows" {
        to.widows = from.widows.clone();
    } else if property == "orphans" {
        to.orphans = from.orphans.clone();
    } else if property == "clear" {
        to.clear_side = from.clear_side.clone();
    } else if property == "contain" {
        to.layout_containment = from.layout_containment.clone();
        to.size_containment = from.size_containment.clone();
    } else if property == "contain-intrinsic-width" {
        to.contain_intrinsic_width = from.contain_intrinsic_width.clone();
    } else if property == "contain-intrinsic-height" {
        to.contain_intrinsic_height = from.contain_intrinsic_height.clone();
    } else if property == "contain-intrinsic-size" {
        to.contain_intrinsic_width = from.contain_intrinsic_width.clone();
        to.contain_intrinsic_height = from.contain_intrinsic_height.clone();
    } else if property == "aspect-ratio" {
        to.aspect_ratio = from.aspect_ratio.clone();
    } else if property == "font" {
        to.font_size = from.font_size.clone();
        to.font_families = from.font_families.clone();
        to.font_weight = from.font_weight.clone();
        to.font_italic = from.font_italic.clone();
        to.line_height = from.line_height.clone();
        to.line_height_percent = from.line_height_percent.clone();
    } else if property == "font-size" {
        to.font_size = from.font_size.clone();
    } else if property == "font-family" {
        to.font_families = from.font_families.clone();
    } else if property == "font-weight" {
        to.font_weight = from.font_weight.clone();
    } else if property == "font-style" {
        to.font_italic = from.font_italic.clone();
    } else if property == "-webkit-font-smoothing" {
        to.font_smoothing = from.font_smoothing.clone();
    } else if property == "line-height" {
        to.line_height = from.line_height.clone();
        to.line_height_percent = from.line_height_percent.clone();
    } else if property == "letter-spacing" {
        to.letter_spacing = from.letter_spacing.clone();
    } else if property == "word-spacing" {
        to.word_spacing = from.word_spacing.clone();
    } else if property == "text-indent" {
        to.text_indent = from.text_indent.clone();
        to.text_indent_percent = from.text_indent_percent.clone();
        to.text_indent_calculated = from.text_indent_calculated.clone();
        to.text_indent_each_line = from.text_indent_each_line.clone();
        to.text_indent_hanging = from.text_indent_hanging.clone();
    } else if property == "white-space" {
        to.white_space = from.white_space.clone();
        to.text_wrap_mode = from.text_wrap_mode.clone();
    } else if property == "text-wrap-mode" {
        to.text_wrap_mode = from.text_wrap_mode.clone();
    } else if property == "text-wrap-style" {
        to.text_wrap_style = from.text_wrap_style.clone();
    } else if property == "text-wrap" {
        to.text_wrap_mode = from.text_wrap_mode.clone();
        to.text_wrap_style = from.text_wrap_style.clone();
    } else if property == "text-align" {
        to.text_align = from.text_align.clone();
    } else if property == "text-align-last" {
        to.text_align_last = from.text_align_last.clone();
    } else if property == "vertical-align" {
        to.vertical_align = from.vertical_align.clone();
        to.vertical_align_length = from.vertical_align_length.clone();
        to.vertical_align_percent = from.vertical_align_percent.clone();
        to.vertical_align_calculated = from.vertical_align_calculated.clone();
    } else if property == "unicode-bidi" {
        to.unicode_bidi = from.unicode_bidi.clone();
    } else if property == "overflow-wrap" {
        to.overflow_wrap = from.overflow_wrap.clone();
    } else if property == "word-break" {
        to.word_break = from.word_break.clone();
    } else if property == "line-break" {
        to.line_break = from.line_break.clone();
    } else if property == "hyphens" {
        to.hyphens = from.hyphens.clone();
    } else if property == "tab-size" {
        to.tab_size = from.tab_size.clone();
        to.tab_size_is_length = from.tab_size_is_length.clone();
    } else if property == "text-orientation" {
        to.text_orientation = from.text_orientation.clone();
    } else if property == "text-combine-upright" || property == "-webkit-text-combine" {
        to.text_combine = from.text_combine.clone();
    } else if property == "text-transform" {
        to.text_transform = from.text_transform.clone();
    } else if property == "ruby-position" || property == "-webkit-ruby-position" {
        to.ruby_position = from.ruby_position.clone();
    } else if property == "ruby-align" {
        to.ruby_align = from.ruby_align.clone();
    } else if property == "ruby-overhang" {
        to.ruby_overhang = from.ruby_overhang.clone();
    } else if property == "line-clamp" || property == "-webkit-line-clamp" {
        to.line_clamp = from.line_clamp.clone();
    } else if property == "overflow" {
        to.overflow_x = from.overflow_x.clone();
        to.overflow_y = from.overflow_y.clone();
    } else if property == "overflow-x" {
        to.overflow_x = from.overflow_x.clone();
    } else if property == "overflow-y" {
        to.overflow_y = from.overflow_y.clone();
    } else if property == "scrollbar-width" {
        to.scrollbar_width = from.scrollbar_width.clone();
    } else if property == "scrollbar-gutter" {
        to.scrollbar_gutter = from.scrollbar_gutter.clone();
    } else if property == "field-sizing" {
        to.field_sizing = from.field_sizing.clone();
    } else if property == "box-sizing" {
        to.box_sizing = from.box_sizing.clone();
    } else if property == "list-style" {
        to.list_style_type = from.list_style_type.clone();
        to.list_style_position = from.list_style_position.clone();
    } else if property == "list-style-type" {
        to.list_style_type = from.list_style_type.clone();
    } else if property == "list-style-position" {
        to.list_style_position = from.list_style_position.clone();
    } else if property == "box-decoration-break" {
        to.box_decoration_break = from.box_decoration_break.clone();
    } else if property == "color" {
        to_paint.color = from_paint.color.clone();
    } else if property == "object-fit" {
        to_paint.object_fit = from_paint.object_fit.clone();
    } else if property == "object-position" {
        to_paint.object_position = from_paint.object_position.clone();
        to_paint.object_position_offset = from_paint.object_position_offset.clone();
    } else if property == "fill" {
        to_paint.svg_fill = from_paint.svg_fill.clone();
        to_paint.svg_fill_current_color = from_paint.svg_fill_current_color.clone();
        to_paint.svg_fill_server = from_paint.svg_fill_server.clone();
    } else if property == "stroke" {
        to_paint.svg_stroke = from_paint.svg_stroke.clone();
        to_paint.svg_stroke_current_color = from_paint.svg_stroke_current_color.clone();
        to_paint.svg_stroke_server = from_paint.svg_stroke_server.clone();
    } else if property == "stroke-width" {
        to_paint.svg_stroke_width = from_paint.svg_stroke_width.clone();
    } else if property == "stroke-dasharray" {
        to_paint.svg_stroke_dash_array = from_paint.svg_stroke_dash_array.clone();
    } else if property == "stroke-dashoffset" {
        to_paint.svg_stroke_dash_offset = from_paint.svg_stroke_dash_offset.clone();
    } else if property == "stroke-linecap" {
        to_paint.svg_stroke_line_cap = from_paint.svg_stroke_line_cap.clone();
    } else if property == "stroke-linejoin" {
        to_paint.svg_stroke_line_join = from_paint.svg_stroke_line_join.clone();
    } else if property == "stroke-miterlimit" {
        to_paint.svg_stroke_miter_limit = from_paint.svg_stroke_miter_limit.clone();
    } else if property == "fill-rule" {
        to_paint.svg_fill_even_odd = from_paint.svg_fill_even_odd.clone();
    } else if property == "vector-effect" {
        to_paint.svg_non_scaling_stroke = from_paint.svg_non_scaling_stroke.clone();
    } else if property == "shape-rendering" {
        to_paint.svg_shape_antialias = from_paint.svg_shape_antialias.clone();
    } else if property == "paint-order" {
        to_paint.svg_paint_order = from_paint.svg_paint_order.clone();
    } else if property.starts_with("background") {
        let mut imported = BackgroundCascadeState::default();
        imported.Import(from_paint);
        if property == "background" {
            *background = imported;
            to_paint.background_color = from_paint.background_color.clone();
        } else if property == "background-image" {
            background.images = imported.images;
        } else if property == "background-repeat" {
            background.repeats = imported.repeats;
        } else if property == "background-size" {
            background.sizes = imported.sizes;
        } else if property == "background-position" {
            background.positions = imported.positions;
        } else if property == "background-origin" {
            background.origins = imported.origins;
        } else if property == "background-clip" {
            background.clips = imported.clips;
        } else if property == "background-blend-mode" {
            background.blend_modes = imported.blend_modes;
        } else if property == "background-color" {
            to_paint.background_color = from_paint.background_color.clone();
        } else {
            return false;
        }
    } else if property == "mask-image" || property == "-webkit-mask-image" {
        to_paint.mask_images = from_paint.mask_images.clone();
    } else if property == "border-radius" {
        to_paint.border_radii = from_paint.border_radii.clone();
        to_paint.border_radii_percentages = from_paint.border_radii_percentages.clone();
        to_paint.border_radius = from_paint.border_radius.clone();
    } else if property == "border-top-left-radius"
        || property == "border-top-right-radius"
        || property == "border-bottom-right-radius"
        || property == "border-bottom-left-radius"
    {
        let mut target = to_paint.border_radii.unwrap_or_default();
        let mut target_percentages = to_paint.border_radii_percentages;
        let value = from_paint.border_radii.unwrap_or_default();
        let value_percentages = from_paint.border_radii_percentages.clone();
        macro_rules! corner {
            ($field:ident) => {{
                target.$field = value.$field;
                target_percentages.$field = value_percentages.$field;
            }};
        }
        match property.as_str() {
            "border-top-left-radius" => corner!(top_left),
            "border-top-right-radius" => corner!(top_right),
            "border-bottom-right-radius" => corner!(bottom_right),
            _ => corner!(bottom_left),
        }
        to_paint.border_radii = Some(target);
        to_paint.border_radii_percentages = target_percentages;
    } else if property == "box-shadow" {
        to_paint.box_shadows = from_paint.box_shadows.clone();
    } else if property == "text-shadow" {
        to_paint.text_shadows = from_paint.text_shadows.clone();
    } else if property == "text-decoration" {
        to_paint.text_decoration = from_paint.text_decoration.clone();
    } else if property == "text-decoration-line" {
        to_paint.text_decoration.underline = from_paint.text_decoration.underline.clone();
        to_paint.text_decoration.overline = from_paint.text_decoration.overline.clone();
        to_paint.text_decoration.line_through = from_paint.text_decoration.line_through;
    } else if property == "text-decoration-style" {
        to_paint.text_decoration.style = from_paint.text_decoration.style.clone();
    } else if property == "text-decoration-color" {
        to_paint.text_decoration.color = from_paint.text_decoration.color.clone();
    } else if property == "text-decoration-thickness" {
        to_paint.text_decoration.thickness = from_paint.text_decoration.thickness.clone();
    } else if property == "text-underline-offset" {
        to_paint.text_decoration.underline_offset = from_paint.text_decoration.underline_offset;
    } else if property == "text-decoration-skip-ink" {
        to_paint.text_decoration.skip_ink = from_paint.text_decoration.skip_ink.clone();
    } else if property == "opacity" {
        to_paint.opacity = from_paint.opacity.clone();
    } else if property == "filter" {
        to_paint.filters = from_paint.filters.clone();
    } else if property == "mix-blend-mode" {
        to_paint.blend_mode = from_paint.blend_mode.clone();
    } else if property == "isolation" {
        to_paint.isolate_blending = from_paint.isolate_blending.clone();
    } else if property == "clip-path" {
        to_paint.clip_path = from_paint.clip_path.clone();
    } else if property == "transform" {
        to_paint.transform = from_paint.transform.clone();
    } else if property == "will-change" {
        to_paint.will_change_transform = from_paint.will_change_transform.clone();
    } else if property == "transform-origin" {
        to_paint.transform_origin = from_paint.transform_origin.clone();
    } else if property == "cursor" {
        to_paint.cursor = from_paint.cursor;
    } else if property == "pointer-events" {
        to_paint.pointer_events_none = from_paint.pointer_events_none;
    } else if property == "visibility" {
        to_paint.visible = from_paint.visible.clone();
    } else {
        return false;
    }
    return true;
}
// cpp: style_resolver/style_resolver.cc:6882-6894
pub(crate) fn CSSWideKeywordSource<'a>(
    value: &str,
    initial: &CSSWideSource<'a>,
    inherited: &CSSWideSource<'a>,
    unset: &CSSWideSource<'a>,
    reverted: &CSSWideSource<'a>,
) -> Option<CSSWideSource<'a>> {
    let keyword = value
        .trim_matches(|c: char| c.source_space())
        .to_ascii_lowercase();
    match keyword.as_str() {
        "initial" => Some(*initial),
        "inherit" => Some(*inherited),
        "unset" => Some(*unset),
        "revert" | "revert-layer" => Some(*reverted),
        _ => None,
    }
}
// cpp: style_resolver/style_resolver.cc:6896-6912
pub(crate) fn ApplyCSSWideKeyword(
    declaration: &cssom::CSSDeclaration,
    writing_mode: WritingMode,
    direction: TextDirection,
    initial: &CSSWideSource<'_>,
    inherited: &CSSWideSource<'_>,
    unset: &CSSWideSource<'_>,
    reverted: &CSSWideSource<'_>,
    style: &mut ComputedStyle,
    generates_box: &mut bool,
    display_contents: &mut bool,
    background: &mut BackgroundCascadeState,
) -> bool {
    CSSWideKeywordSource(&declaration.value, initial, inherited, unset, reverted).is_some_and(
        |source| {
            CopyCSSProperty(
                &declaration.property,
                writing_mode,
                direction,
                &source,
                style,
                generates_box,
                display_contents,
                background,
            )
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use layoutng_assembly::internal::layout_input::UnicodeBidi;
    use layoutng_assembly::internal::paint_input::{
        BackgroundBox, BackgroundImageLayer, PaintBlendMode,
    };
    use layoutng_style::style::appearance::AppearanceValue;

    #[test]
    fn logical_wide_values_copy_atomic_dimensions_and_physical_edge_state() {
        let mut source = ComputedStyle::default();
        source.width = Some(23.0);
        source.margin = Edges {
            top: 1.0,
            right: 2.0,
            bottom: 3.0,
            left: 4.0,
        };
        let e = source.extended.get_or_insert_with(Default::default);
        e.width_percent = Some(40.0);
        e.width_calculated = true;
        e.margin_percentages = [Some(10.0), Some(20.0), Some(30.0), Some(40.0)];
        e.margin_calculated = [true, false, true, false];
        e.margin_auto = [false, true, false, true];
        let mut target = ComputedStyle::default();
        target.width = Some(999.0);
        target
            .extended
            .get_or_insert_with(Default::default)
            .width_percent = Some(99.0);
        let mut background = BackgroundCascadeState::default();
        let (mut generates, mut contents) = (true, false);
        let from = CSSWideSource::new(&source);
        assert!(CopyCSSProperty(
            "block-size",
            WritingMode::kVerticalRl,
            TextDirection::kRtl,
            &from,
            &mut target,
            &mut generates,
            &mut contents,
            &mut background
        ));
        assert_eq!(target.width, Some(23.0));
        assert_eq!(target.extended.as_ref().unwrap().width_percent, Some(40.0));
        assert!(target.extended.as_ref().unwrap().width_calculated);
        assert!(CopyCSSProperty(
            "margin-inline",
            WritingMode::kVerticalRl,
            TextDirection::kRtl,
            &from,
            &mut target,
            &mut generates,
            &mut contents,
            &mut background
        ));
        assert_eq!(
            target.margin,
            Edges {
                top: 1.0,
                bottom: 3.0,
                ..Default::default()
            }
        );
        let e = target.extended.as_ref().unwrap();
        assert_eq!(e.margin_percentages, [Some(10.0), None, Some(30.0), None]);
        assert_eq!(e.margin_calculated, [true, false, true, false]);
    }

    #[test]
    fn all_preserves_excluded_host_fields_and_background_state_roundtrips() {
        let mut source = ComputedStyle::default();
        source.width = Some(42.0);
        source.direction = TextDirection::kRtl;
        source.paint.image_resource_id = Some(999);
        source.paint.background_clip = BackgroundBox::kContentBox;
        source.paint.background_images = vec![
            BackgroundImageLayer {
                source_url: "one.png".into(),
                blend_mode: PaintBlendMode::kMultiply,
                clip: BackgroundBox::kPaddingBox,
                ..Default::default()
            },
            BackgroundImageLayer {
                source_url: "two.png".into(),
                blend_mode: PaintBlendMode::kScreen,
                clip: BackgroundBox::kContentBox,
                ..Default::default()
            },
        ];
        let mut target = ComputedStyle::default();
        target.paint.image_resource_id = Some(7);
        let e = target.extended.get_or_insert_with(Default::default);
        e.language = "zh".into();
        e.unicode_bidi = UnicodeBidi::kIsolate;
        e.effective_appearance = AppearanceValue::kTextField;
        e.effective_zoom = 2.0;
        let mut background = BackgroundCascadeState::default();
        let (mut generates, mut contents) = (true, false);
        let from = CSSWideSource {
            style: &source,
            generates_box: false,
            display_contents: true,
        };
        assert!(CopyCSSProperty(
            "all",
            WritingMode::kHorizontalTb,
            TextDirection::kLtr,
            &from,
            &mut target,
            &mut generates,
            &mut contents,
            &mut background
        ));
        assert_eq!(target.width, Some(42.0));
        assert_eq!(target.direction, TextDirection::kLtr);
        assert_eq!(target.paint.image_resource_id, Some(7));
        let e = target.extended.as_ref().unwrap();
        assert_eq!(e.language, "zh");
        assert_eq!(e.unicode_bidi, UnicodeBidi::kIsolate);
        assert_eq!(e.effective_appearance, AppearanceValue::kTextField);
        assert_eq!(e.effective_zoom, 2.0);
        assert!(!generates && contents);
        background.Export(&mut target);
        assert_eq!(
            target.paint.background_images[0].blend_mode,
            PaintBlendMode::kMultiply
        );
        assert_eq!(
            target.paint.background_images[1].blend_mode,
            PaintBlendMode::kScreen
        );
        assert_eq!(target.paint.background_clip, BackgroundBox::kContentBox);
        let mut empty = ComputedStyle::default();
        empty.paint.background_clip = BackgroundBox::kPaddingBox;
        let mut imported = BackgroundCascadeState::default();
        imported.Import(&empty.paint);
        imported.Export(&mut empty);
        assert_eq!(empty.paint.background_clip, BackgroundBox::kPaddingBox);
        assert!(!CopyCSSProperty(
            "unsupported-property",
            WritingMode::kHorizontalTb,
            TextDirection::kLtr,
            &from,
            &mut target,
            &mut generates,
            &mut contents,
            &mut BackgroundCascadeState::default()
        ));
    }
}
