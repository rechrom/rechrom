use foundation::TextOffsetMap;

// cpp: layoutng/internal/variable_length_transform_result.h:26-29
#[derive(Clone)]
pub struct VariableLengthTransformResult {
    pub original_length: u32,
    pub offset_map: TextOffsetMap,
}
