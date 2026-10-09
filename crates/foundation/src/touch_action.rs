// C++: foundation/graphics_types/graphics/touch_action.h.
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

// cpp: foundation/graphics_types/graphics/touch_action.h:20-51
pub const kTouchActionBits: usize = 8;

// A transparent integer preserves C++ combinations produced by bitwise ops,
// including values that are not named enum constants.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TouchAction(i32);

#[allow(non_upper_case_globals)]
impl TouchAction {
    pub const kNone: Self = Self(0);
    pub const kPanLeft: Self = Self(0x1);
    pub const kPanRight: Self = Self(0x2);
    pub const kPanX: Self = Self(0x3);
    pub const kPanUp: Self = Self(0x4);
    pub const kPanDown: Self = Self(0x8);
    pub const kPanY: Self = Self(0xc);
    pub const kPan: Self = Self(0xf);
    pub const kPinchZoom: Self = Self(0x10);
    pub const kManipulation: Self = Self(0x1f);
    pub const kDoubleTapZoom: Self = Self(0x20);
    pub const kInternalPanXScrolls: Self = Self(0x40);
    pub const kInternalNotWritable: Self = Self(0x80);
    pub const kAuto: Self = Self(0xff);
    pub const kMax: Self = Self((1 << kTouchActionBits) - 1);
    // C++ casts to the underlying bit-mask type preserve unnamed combinations.
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits as i32)
    }
    pub const fn bits(self) -> i32 {
        self.0
    }
}

// cpp: foundation/graphics_types/graphics/touch_action.h:53-71
impl BitOr for TouchAction {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl BitOrAssign for TouchAction {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}
impl BitAnd for TouchAction {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
impl BitAndAssign for TouchAction {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}
impl Not for TouchAction {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0)
    }
}

// cpp: foundation/graphics_types/graphics/touch_action.h:73-213
pub fn TouchActionToString(mut touch_action: TouchAction) -> &'static str {
    touch_action &= !TouchAction::kInternalPanXScrolls;
    touch_action &= !TouchAction::kInternalNotWritable;
    const NAMES: [&str; 64] = [
        "NONE",
        "PAN_LEFT",
        "PAN_RIGHT",
        "PAN_X",
        "PAN_UP",
        "PAN_LEFT_PAN_UP",
        "PAN_RIGHT_PAN_UP",
        "PAN_X_PAN_UP",
        "PAN_DOWN",
        "PAN_LEFT_PAN_DOWN",
        "PAN_RIGHT_PAN_DOWN",
        "PAN_X_PAN_DOWN",
        "PAN_Y",
        "PAN_LEFT_PAN_Y",
        "PAN_RIGHT_PAN_Y",
        "PAN_X_PAN_Y",
        "PINCH_ZOOM",
        "PAN_LEFT_PINCH_ZOOM",
        "PAN_RIGHT_PINCH_ZOOM",
        "PAN_X_PINCH_ZOOM",
        "PAN_UP_PINCH_ZOOM",
        "PAN_LEFT_PAN_UP_PINCH_ZOOM",
        "PAN_RIGHT_PAN_UP_PINCH_ZOOM",
        "PAN_X_PAN_UP_PINCH_ZOOM",
        "PAN_DOWN_PINCH_ZOOM",
        "PAN_LEFT_PAN_DOWN_PINCH_ZOOM",
        "PAN_RIGHT_PAN_DOWN_PINCH_ZOOM",
        "PAN_X_PAN_DOWN_PINCH_ZOOM",
        "PAN_Y_PINCH_ZOOM",
        "PAN_LEFT_PAN_Y_PINCH_ZOOM",
        "PAN_RIGHT_PAN_Y_PINCH_ZOOM",
        "MANIPULATION",
        "DOUBLE_TAP_ZOOM",
        "PAN_LEFT_DOUBLE_TAP_ZOOM",
        "PAN_RIGHT_DOUBLE_TAP_ZOOM",
        "PAN_X_DOUBLE_TAP_ZOOM",
        "PAN_UP_DOUBLE_TAP_ZOOM",
        "PAN_LEFT_PAN_UP_DOUBLE_TAP_ZOOM",
        "PAN_RIGHT_PAN_UP_DOUBLE_TAP_ZOOM",
        "PAN_X_PAN_UP_DOUBLE_TAP_ZOOM",
        "PAN_DOWN_DOUBLE_TAP_ZOOM",
        "PAN_LEFT_PAN_DOWN_DOUBLE_TAP_ZOOM",
        "PAN_RIGHT_PAN_DOWN_DOUBLE_TAP_ZOOM",
        "PAN_X_PAN_DOWN_DOUBLE_TAP_ZOOM",
        "PAN_Y_DOUBLE_TAP_ZOOM",
        "PAN_LEFT_PAN_Y_DOUBLE_TAP_ZOOM",
        "PAN_RIGHT_PAN_Y_DOUBLE_TAP_ZOOM",
        "PAN_X_PAN_Y_DOUBLE_TAP_ZOOM",
        "PINCH_ZOOM_DOUBLE_TAP_ZOOM",
        "PAN_LEFT_PINCH_ZOOM_DOUBLE_TAP_ZOOM",
        "PAN_RIGHT_PINCH_ZOOM_DOUBLE_TAP_ZOOM",
        "PAN_X_PINCH_ZOOM_DOUBLE_TAP_ZOOM",
        "PAN_UP_PINCH_ZOOM_DOUBLE_TAP_ZOOM",
        "PAN_LEFT_PAN_UP_PINCH_ZOOM_DOUBLE_TAP_ZOOM",
        "PAN_RIGHT_PAN_UP_PINCH_ZOOM_DOUBLE_TAP_ZOOM",
        "PAN_X_PAN_UP_PINCH_ZOOM_DOUBLE_TAP_ZOOM",
        "PAN_DOWN_PINCH_ZOOM_DOUBLE_TAP_ZOOM",
        "PAN_LEFT_PAN_DOWN_PINCH_ZOOM_DOUBLE_TAP_ZOOM",
        "PAN_RIGHT_PAN_DOWN_PINCH_ZOOM_DOUBLE_TAP_ZOOM",
        "PAN_X_PAN_DOWN_PINCH_ZOOM_DOUBLE_TAP_ZOOM",
        "PAN_Y_PINCH_ZOOM_DOUBLE_TAP_ZOOM",
        "PAN_LEFT_PAN_Y_PINCH_ZOOM_DOUBLE_TAP_ZOOM",
        "PAN_RIGHT_PAN_Y_PINCH_ZOOM_DOUBLE_TAP_ZOOM",
        "AUTO",
    ];
    NAMES[usize::try_from(touch_action.0).expect("invalid touch action")]
}
