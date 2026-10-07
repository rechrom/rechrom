// C++: font_engine/fonts/canvas_rotation_in_vertical.h.

// cpp: font_engine/fonts/canvas_rotation_in_vertical.h:11-16
#[repr(i8)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CanvasRotationInVertical {
    #[default]
    kRegular = 0,
    kRotateCanvasUpright = 1,
    kOblique = 2,
    kRotateCanvasUprightOblique = 3,
}

// cpp: font_engine/fonts/canvas_rotation_in_vertical.h:18-26
pub const fn IsCanvasRotationInVerticalUpright(rotation: CanvasRotationInVertical) -> bool {
    (rotation as i8 & CanvasRotationInVertical::kRotateCanvasUpright as i8) != 0
}

// The spelling follows the C++ source's exported name.
pub const fn IsCanvasRotationOblque(rotation: CanvasRotationInVertical) -> bool {
    (rotation as i8 & CanvasRotationInVertical::kOblique as i8) != 0
}
