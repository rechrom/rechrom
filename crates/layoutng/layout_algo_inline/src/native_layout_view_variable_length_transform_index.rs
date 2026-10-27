//! Variable-length text transform index owned by the inline algorithm.

use super::{LayoutText, LayoutView, VariableLengthTransformResult, WeakMember};

impl LayoutView {
    // cpp: layoutng/internal/layout_view_data.cc:52-58
    #[inline(never)]
    pub fn RegisterVariableLengthTransformResult(
        &mut self,
        text: &LayoutText,
        result: &VariableLengthTransformResult,
    ) {
        self.CheckIsNotDestroyed();
        assert!(text.HasVariableLengthTransform());
        self.text_to_variable_length_transform_result_.Set(
            WeakMember::from_ptr(text as *const LayoutText as *mut LayoutText),
            result.clone(),
        );
    }

    // cpp: layoutng/internal/layout_view_data.cc:60-64
    #[inline(never)]
    pub fn UnregisterVariableLengthTransformResult(&mut self, text: &LayoutText) {
        self.CheckIsNotDestroyed();
        self.text_to_variable_length_transform_result_
            .remove(&WeakMember::from_ptr(
                text as *const LayoutText as *mut LayoutText,
            ));
    }

    // cpp: layoutng/internal/layout_view_data.cc:66-71
    pub fn GetVariableLengthTransformResult(
        &self,
        text: &LayoutText,
    ) -> VariableLengthTransformResult {
        self.CheckIsNotDestroyed();
        assert!(text.HasVariableLengthTransform());
        self.text_to_variable_length_transform_result_
            .get(&WeakMember::from_ptr(
                text as *const LayoutText as *mut LayoutText,
            ))
            .expect("variable-length transform result registered")
            .clone()
    }
}
