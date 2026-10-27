//! Element metadata construction and incremental input refresh.
//!
//! Loaded by the reciprocal assembly boundary so validation and derived-state
//! mutation remain attributable to layout invalidation without exposing the
//! private Element record to the standalone package.

use super::*;

#[allow(non_snake_case)]
impl Element {
    // cpp: layoutng/internal/layout_node_metadata.cc:229-399
    #[inline(never)]
    pub fn new(
        input: &NativeNodeConstructionData,
        style: *const NativeComputedStyle,
    ) -> Result<Self, ElementMetadataError> {
        let container = ContainerNode::new(input, style, NodeType::kElementNode)
            .map_err(ElementMetadataError::Node)?;
        let mut element = Self {
            container,
            element_type_: ElementType::kElement,
            is_form_control_element_: false,
            remembered_inline_size_: None,
            remembered_block_size_: None,
            used_canvas_transform_: None,
            saved_layer_scroll_offset_: ScrollOffset::default(),
            timeline_trigger_names_: Vec::new(),
            anchor_scroll_data_: Member::default(),
            out_of_flow_data_: Member::default(),
            active_transform_animations_: 0,
            may_be_implicit_anchor_: false,
        };
        validate_element_input_prefix(input).map_err(ElementMetadataError::InvalidInput)?;
        prepare_element_dimensions(&mut element, input)
            .map_err(ElementMetadataError::InvalidInput)?;
        element.InputSvgLengthAdjust();
        if input
            .style
            .extended
            .as_ref()
            .is_some_and(|style| !style.timeline_trigger_names.is_empty())
        {
            PopulateTimelineTriggerNames(&mut element, input)?;
        }
        Ok(element)
    }

    #[inline(never)]
    pub(crate) fn UpdateElementInput(&mut self, input: &NativeNodeConstructionData) {
        validate_element_input_prefix(input).unwrap_or_else(|error| panic!("{error:?}"));
        self.node.UpdateInputData(input);
        self.remembered_inline_size_ = None;
        self.remembered_block_size_ = None;
        self.used_canvas_transform_ = None;
        prepare_element_dimensions(self, input).unwrap_or_else(|error| panic!("{error:?}"));
        self.InputSvgLengthAdjust();
        let textarea = foundation::DynamicTo::<super::super::form_node_metadata::HTMLTextAreaElement>(
            self as *mut Element,
        );
        if !textarea.is_null() {
            let sizing = input
                .element
                .as_ref()
                .and_then(|data| data.text_area_sizing.as_ref());
            unsafe { &mut *textarea }.UpdateSizing(
                sizing.map_or(2, |sizing| sizing.rows),
                sizing.map_or(20, |sizing| sizing.columns),
            );
        }
        let control = foundation::DynamicTo::<
            super::super::form_node_metadata::HTMLFormControlElement,
        >(self as *mut Element);
        if !control.is_null() {
            unsafe { &mut *control }.autofill_state_ = input.element.as_ref().map_or(
                super::super::form_control_types::AutofillState::kNotFilled,
                |element| element.autofill_state,
            );
        }
    }
}

// The pure first stage of Element construction. The owning constructor calls
// this after constructing ContainerNode, as in the C++ initializer list.
// cpp: layoutng/internal/layout_node_metadata.cc:231-327
fn validate_element_input_prefix(input: &NativeNodeConstructionData) -> Result<(), &'static str> {
    let data = input.element.as_ref();
    if input.kind == NodeKind::kText || !input.text.is_empty() {
        return Err("Element metadata requires an element input; text needs a Text node");
    }
    // SVGTextPathElement creates LayoutSVGTextPath before its referenced
    // geometry necessarily exists. Missing geometry is a normal LayoutPath
    // result; present geometry still receives the typed validation below.
    if let Some(data) = data {
        if data.svg_shape.is_some()
            || input.kind == NodeKind::kSvgShape
            || data.svg_text_path.is_some()
        {
            ValidateSvgElementInput(input.kind, data)?;
        }
        if data.supports_base_appearance && input.kind != NodeKind::kFormControl {
            return Err("Base-appearance capability requires a form control");
        }
        if data.select_uses_menu_list.is_some()
            && (input.kind != NodeKind::kFormControl
                || (data.form_control_type != Some(FormControlType::kSelectOne)
                    && data.form_control_type != Some(FormControlType::kSelectMultiple)))
        {
            return Err("Select presentation requires a select form control");
        }
        if let Some(sizing) = data.text_area_sizing.as_ref() {
            if input.kind != NodeKind::kFormControl
                || data.form_control_type != Some(FormControlType::kTextArea)
                || sizing.rows == 0
                || sizing.columns == 0
            {
                return Err("Textarea sizing requires a textarea and positive rows/columns");
            }
        }
        if data.text_field_sizing.is_some()
            && (input.kind != NodeKind::kFormControl
                || data.form_control_type.is_none()
                || data.form_control_type.is_some_and(|kind| {
                    (kind as i32) < FormControlType::kInputButton as i32
                        || (kind as i32) > FormControlType::kInputWeek as i32
                }))
        {
            return Err("Text-field sizing requires an input control");
        }
        if data.text_control_inner_editor
            || data.text_control_container
            || data.text_control_spin_button
            || data.text_control_placeholder
        {
            let part_count = data.text_control_inner_editor as u32
                + data.text_control_container as u32
                + data.text_control_spin_button as u32
                + data.text_control_placeholder as u32;
            if input.kind != NodeKind::kBox
                || part_count > 1
                || data.form_control_type.is_some()
                || data.file_upload_button
                || data.image_map_area
                || data.html_image
                || data.first_letter_pseudo
            {
                return Err("Text control part has conflicting node identity");
            }
        }
        if data
            .content_lock
            .is_some_and(|lock| !lock.is_locked && !lock.should_layout_children)
        {
            return Err("An unlocked content-lock context must allow child layout");
        }
    }
    Ok(())
}

// The second construction stage validates and converts each native value in
// source order. Rust String already enforces the UTF-8 validity that C++
// checks after accepting arbitrary std::string bytes for the file label.
// cpp: layoutng/internal/layout_node_metadata.cc:328-385
fn prepare_element_dimensions(
    element: &mut Element,
    input: &NativeNodeConstructionData,
) -> Result<(), &'static str> {
    let Some(data) = input.element.as_ref() else {
        return Ok(());
    };
    if data.file_no_file_label.is_some()
        && (input.kind != NodeKind::kFormControl
            || data.form_control_type != Some(FormControlType::kInputFile))
    {
        return Err("No-file-selected label requires an input-file control");
    }
    let remembered_size = |size: Option<f64>| -> Result<Option<LayoutUnit>, &'static str> {
        let Some(size) = size else {
            return Ok(None);
        };
        if !size.is_finite() || size < 0.0 || size > LayoutUnit::Max().ToDouble() {
            return Err("Remembered intrinsic size must be finite, nonnegative and representable");
        }
        Ok(Some(LayoutUnit::from_f64(size)))
    };
    element.remembered_inline_size_ = remembered_size(data.remembered_inline_size)?;
    element.remembered_block_size_ = remembered_size(data.remembered_block_size)?;
    if let Some(transform) = data.used_canvas_transform.as_ref() {
        for value in transform.values {
            if !value.is_finite() {
                return Err("Used Canvas transform coefficients must be finite");
            }
        }
        element.used_canvas_transform_ = Some(gfx::Transform::ColMajor(&transform.values));
    }
    if data
        .root_scroller_visible_size
        .is_some_and(|size| size.width < 0 || size.height < 0)
    {
        return Err("Root scroller visible size must be nonnegative");
    }
    let valid_natural_dimension = |value: Option<f64>| {
        value.map_or(true, |value| {
            value.is_finite() && value >= 0.0 && value <= LayoutUnit::Max().ToDouble()
        })
    };
    if !valid_natural_dimension(data.natural_width)
        || !valid_natural_dimension(data.natural_height)
        || data
            .natural_aspect_ratio
            .is_some_and(|ratio| !ratio.is_finite() || ratio <= 0.0)
        || !data.image_device_pixel_ratio.is_finite()
        || data.image_device_pixel_ratio <= 0.0
    {
        return Err("Replaced natural dimensions must be finite and nonnegative");
    }
    let has_replaced_data = data.natural_width.is_some()
        || data.natural_height.is_some()
        || data.natural_aspect_ratio.is_some()
        || data.image_device_pixel_ratio != 1.0
        || data.replaced_respects_css_overflow;
    if has_replaced_data && input.kind != NodeKind::kReplaced && input.kind != NodeKind::kSvgRoot {
        return Err("Replaced natural dimensions require a replaced node");
    }
    if data.image_device_pixel_ratio != 1.0 && !data.html_image {
        return Err("Image density correction requires HTML image identity");
    }
    let offset = data.scroll_offset;
    let valid_offset = |value: f64| value.is_finite() && value.abs() <= f32::MAX as f64;
    if !valid_offset(offset.x) || !valid_offset(offset.y) {
        return Err("Scroll offset must be finite and representable as a native float");
    }
    element.saved_layer_scroll_offset_ = ScrollOffset::new(offset.x as f32, offset.y as f32);
    Ok(())
}
