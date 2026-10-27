//! Invalidation-owned comparison of variable-size element metadata input.

use super::ElementData;

impl PartialEq for ElementData {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        element_data_eq(self, other)
    }
}

// Trait-method symbols omit the impl's source module. Keep the variable-size
// comparison in a package-qualified free-function frame for sampling.
#[inline(never)]
fn element_data_eq(element: &ElementData, other: &ElementData) -> bool {
    (element.used_canvas_transform == other.used_canvas_transform)
        && (element.content_lock == other.content_lock)
        && (element.remembered_inline_size == other.remembered_inline_size)
        && (element.remembered_block_size == other.remembered_block_size)
        && (element.nowrap_attribute == other.nowrap_attribute)
        && (element.image_map_area == other.image_map_area)
        && (element.html_image == other.html_image)
        && (element.html_ordered_or_unordered_list == other.html_ordered_or_unordered_list)
        && (element.first_letter_pseudo == other.first_letter_pseudo)
        && (element.viewport_defining == other.viewport_defining)
        && (element.root_editable == other.root_editable)
        && (element.implicit_anchor_id == other.implicit_anchor_id)
        && (element.scroll_offset == other.scroll_offset)
        && (element.root_scroller_visible_size == other.root_scroller_visible_size)
        && (element.natural_width == other.natural_width)
        && (element.natural_height == other.natural_height)
        && (element.natural_aspect_ratio == other.natural_aspect_ratio)
        && (element.image_device_pixel_ratio == other.image_device_pixel_ratio)
        && (element.replaced_respects_css_overflow == other.replaced_respects_css_overflow)
        && (element.row_span == other.row_span)
        && (element.column_span == other.column_span)
        && (element.stretchy == other.stretchy)
        && (element.symmetric == other.symmetric)
        && (element.large_op == other.large_op)
        && (element.movable_limits == other.movable_limits)
        && (element.accent == other.accent)
        && (element.accent_under == other.accent_under)
        && (element.math_token_kind == other.math_token_kind)
        && (element.math_operator_form == other.math_operator_form)
        && (element.frame_rows == other.frame_rows)
        && (element.frame_columns == other.frame_columns)
        && (element.frame_row_deltas == other.frame_row_deltas)
        && (element.frame_column_deltas == other.frame_column_deltas)
        && (element.frame_border_thickness == other.frame_border_thickness)
        && (element.frame_border == other.frame_border)
        && (element.frame_no_resize == other.frame_no_resize)
        && (element.frame_has_border_color == other.frame_has_border_color)
        && (element.svg_use == other.svg_use)
        && (element.svg_x == other.svg_x)
        && (element.svg_y == other.svg_y)
        && (element.svg_dx == other.svg_dx)
        && (element.svg_dy == other.svg_dy)
        && (element.svg_rotate == other.svg_rotate)
        && (element.svg_shape == other.svg_shape)
        && (element.svg_view_box == other.svg_view_box)
        && (element.svg_viewport == other.svg_viewport)
        && (element.svg_text_length == other.svg_text_length)
        && (element.svg_text_path == other.svg_text_path)
        && (element.svg_length_adjust == other.svg_length_adjust)
        && (element.custom_layout_name == other.custom_layout_name)
        && (match (&element.custom_layout, &other.custom_layout) {
            (Some(a), Some(b)) => std::sync::Arc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        })
        && (element.form_control_type == other.form_control_type)
        && (element.select_uses_menu_list == other.select_uses_menu_list)
        && (element.control_checked == other.control_checked)
        && (element.control_indeterminate == other.control_indeterminate)
        && (element.control_disabled == other.control_disabled)
        && (element.control_read_only == other.control_read_only)
        && (element.control_hovered == other.control_hovered)
        && (element.control_active == other.control_active)
        && (element.control_focused == other.control_focused)
        && (element.control_value_ratio == other.control_value_ratio)
        && (element.supports_base_appearance == other.supports_base_appearance)
        && (element.selected_file_count == other.selected_file_count)
        && (element.file_no_file_label == other.file_no_file_label)
        && (element.file_upload_button == other.file_upload_button)
        && (element.text_field_sizing == other.text_field_sizing)
        && (element.text_area_sizing == other.text_area_sizing)
        && (element.text_control_inner_editor == other.text_control_inner_editor)
        && (element.text_control_container == other.text_control_container)
        && (element.text_control_spin_button == other.text_control_spin_button)
        && (element.text_control_placeholder == other.text_control_placeholder)
        && (element.autofill_state == other.autofill_state)
        && (element.range_value_ratio == other.range_value_ratio)
        && (element.control_host_ancestor == other.control_host_ancestor)
        && (element.marquee_horizontal == other.marquee_horizontal)
        && (element.document_role == other.document_role)
}
