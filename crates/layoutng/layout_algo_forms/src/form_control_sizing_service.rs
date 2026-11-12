use foundation::DynamicTo;
use layoutng_style::style::computed_style::ComputedStyle;

use super::super::form_control_types::AutofillState;
use super::super::form_node_metadata::HTMLFormControlElement;
use super::super::layout_node_metadata::Node;

// Rust cannot add an inherent method to ComputedStyle, which belongs to the
// style crate. Callers import this extension trait for the C++ member method.
pub trait ComputedStyleControlSizingExt {
    #[allow(non_snake_case)]
    fn ApplyControlFixedSize(&self, node: *const Node) -> bool;
}

impl ComputedStyleControlSizingExt for ComputedStyle {
    // cpp: layoutng/internal/form_control_sizing_service.cc:27-39
    fn ApplyControlFixedSize(&self, node: *const Node) -> bool {
        layoutng_style::style::menu_list_inner_style::ApplyControlFixedSize(self, || {
            if node.is_null() {
                return false;
            }
            let mut control = DynamicTo::<HTMLFormControlElement>(node);
            if control.is_null() {
                control = DynamicTo::<HTMLFormControlElement>(unsafe { &*node }.OwnerShadowHost());
            }
            !control.is_null()
                && unsafe { &*control }.GetAutofillState() != AutofillState::kNotFilled
        })
    }
}
