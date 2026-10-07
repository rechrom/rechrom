// C++: foundation/graphics_types/graphics/image_animation_enum.h.
// cpp: foundation/graphics_types/graphics/image_animation_enum.h:5-12
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageAnimationEnum {
    kNormal,
    kRunning,
    kPaused,
    kStopped,
}

impl ImageAnimationEnum {
    pub const kMaxEnumValue: Self = Self::kStopped;
}
