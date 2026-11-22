use super::computed_style_constants::OffsetRotationType;

// cpp: layoutng_style/style/style_offset_rotation.h:12-22
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StyleOffsetRotation {
    pub angle: f32,
    pub r#type: OffsetRotationType,
}

#[allow(non_snake_case)]
impl StyleOffsetRotation {
    // cpp: layoutng_style/style/style_offset_rotation.h:13-14
    pub const fn new(angle: f32, r#type: OffsetRotationType) -> Self {
        Self { angle, r#type }
    }
}
