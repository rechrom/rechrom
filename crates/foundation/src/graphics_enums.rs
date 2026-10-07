// Enum portions of the graphics interfaces used by computed style. The
// declarations for ToSkBlendMode, BlendModeToString, image transforms, and
// GetDefaultInterpolationQuality have no supplied implementation here.

// cpp: foundation/graphics_types/graphics/blend_mode.h:16-29
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompositeOperator {
    kCompositeClear,
    kCompositeCopy,
    kCompositeSourceOver,
    kCompositeSourceIn,
    kCompositeSourceOut,
    kCompositeSourceAtop,
    kCompositeDestinationOver,
    kCompositeDestinationIn,
    kCompositeDestinationOut,
    kCompositeDestinationAtop,
    kCompositeXOR,
    kCompositePlusLighter,
}

// cpp: foundation/graphics_types/graphics/blend_mode.h:31-54
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BlendMode {
    #[default]
    kNormal,
    kMultiply,
    kScreen,
    kOverlay,
    kDarken,
    kLighten,
    kColorDodge,
    kColorBurn,
    kHardLight,
    kSoftLight,
    kDifference,
    kExclusion,
    kHue,
    kSaturation,
    kColor,
    kLuminosity,
    kPlusLighter,
}

impl BlendMode {
    pub const kMaxBlendMode: Self = Self::kPlusLighter;
}

// cpp: foundation/graphics_types/graphics/image_orientation.h:45-48
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RespectImageOrientationEnum {
    kDoNotRespectImageOrientation = 0,
    kRespectImageOrientation = 1,
}

// cpp: foundation/graphics_types/graphics/graphics_context_types.h:35-40
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterpolationQuality {
    kInterpolationNone = 0,
    kInterpolationLow = 1,
    kInterpolationMedium = 2,
}
