// C++: src/foundation/text/native/text_direction.h

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextDirection {
    kLtr = 0,
    kRtl = 1,
}

// C++: text_direction.h, IsLtr.
pub fn IsLtr(direction: TextDirection) -> bool {
    direction == TextDirection::kLtr
}

// C++: text_direction.h, IsRtl.
pub fn IsRtl(direction: TextDirection) -> bool {
    direction != TextDirection::kLtr
}

// C++: text_direction.h, DirectionFromLevel.
pub fn DirectionFromLevel(level: u32) -> TextDirection {
    if level & 1 != 0 {
        TextDirection::kRtl
    } else {
        TextDirection::kLtr
    }
}
