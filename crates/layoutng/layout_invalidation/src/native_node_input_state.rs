//! Invalidation-owned construction and replacement of retained node input.

use foundation::Member;

use super::{
    CompatibilityMode, ControlThemeMetrics, DocumentRole, NativeNodeConstructionData, Node,
    StandaloneNodeData,
};

#[inline(never)]
pub(super) fn NewStandaloneNodeData(input: &NativeNodeConstructionData) -> StandaloneNodeData {
    StandaloneNodeData {
        compatibility_mode: CompatibilityMode::kStandards,
        printing: false,
        paginated: false,
        fragmentainer_block_size: None,
        vertical_scroll_enforced: false,
        device_pixel_ratio: 1.0,
        canvas_draw_element_enabled: false,
        prefer_non_composited_scrolling_enabled: false,
        viewport: None,
        minimum_font_physical_size: None,
        scrollbar_theme: None,
        control_theme: ControlThemeMetrics::default(),
        document_role: input
            .element
            .as_ref()
            .map_or(DocumentRole::kNone, |element| element.document_role),
        input_id: input.id,
        input_kind: input.kind,
        input_is_style_generated: input.style_generated,
        input_style: input.style.clone(),
        input_first_line_style_data: input.first_line_style.clone(),
        debug_name: input.debug_name.clone(),
        element_data: input.element.clone(),
        reusable_layout_environment: None,
        bound_font: Member::default(),
    }
}

#[inline(never)]
pub(super) fn UpdateNodeInputData(node: &mut Node, input: &NativeNodeConstructionData) {
    assert_eq!(node.InputId(), input.id);
    assert_eq!(node.InputKind(), input.kind);
    node.input_font_dirty_ = true;
    node.input_data_.input_style = input.style.clone();
    node.input_data_.input_first_line_style_data = input.first_line_style.clone();
    node.input_data_.document_role = input
        .element
        .as_ref()
        .map_or(DocumentRole::kNone, |element| element.document_role);
    node.input_data_.element_data = input.element.clone();
    node.input_data_.debug_name = input.debug_name.clone();
    node.input_tree_sealed_ = false;
}
