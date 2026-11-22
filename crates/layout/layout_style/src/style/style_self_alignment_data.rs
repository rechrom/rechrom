use super::computed_style_constants::{ItemPosition, ItemPositionType, OverflowAlignment};

// cpp: layoutng_style/style/style_self_alignment_data.h:13-59
/// The C++ unsigned fields occupy 4, 1, and 2 bits respectively.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StyleSelfAlignmentData {
    bits_: u32,
}

#[allow(non_snake_case)]
impl StyleSelfAlignmentData {
    const POSITION_MASK: u32 = 0b1111;
    const POSITION_TYPE_MASK: u32 = 0b1 << 4;
    const OVERFLOW_MASK: u32 = 0b11 << 5;

    // cpp: layoutng_style/style/style_self_alignment_data.h:21-27
    pub const fn new_nonlegacy(position: ItemPosition, overflow: OverflowAlignment) -> Self {
        Self::new(position, overflow, ItemPositionType::kNonLegacy)
    }

    // cpp: layoutng_style/style/style_self_alignment_data.h:21-27
    pub const fn new(
        position: ItemPosition,
        overflow: OverflowAlignment,
        position_type: ItemPositionType,
    ) -> Self {
        Self {
            bits_: ((position as u32) & 0b1111)
                | (((position_type as u32) & 0b1) << 4)
                | (((overflow as u32) & 0b11) << 5),
        }
    }

    // cpp: layoutng_style/style/style_self_alignment_data.h:29-31
    pub fn SetPosition(&mut self, position: ItemPosition) {
        self.bits_ = (self.bits_ & !Self::POSITION_MASK) | ((position as u32) & 0b1111);
    }

    // cpp: layoutng_style/style/style_self_alignment_data.h:32-34
    pub fn SetPositionType(&mut self, position_type: ItemPositionType) {
        self.bits_ =
            (self.bits_ & !Self::POSITION_TYPE_MASK) | (((position_type as u32) & 0b1) << 4);
    }

    // cpp: layoutng_style/style/style_self_alignment_data.h:35-37
    pub fn SetOverflow(&mut self, overflow: OverflowAlignment) {
        self.bits_ = (self.bits_ & !Self::OVERFLOW_MASK) | (((overflow as u32) & 0b11) << 5);
    }

    // cpp: layoutng_style/style/style_self_alignment_data.h:39-41
    pub fn GetPosition(&self) -> ItemPosition {
        match self.bits_ & Self::POSITION_MASK {
            0 => ItemPosition::kLegacy,
            1 => ItemPosition::kAuto,
            2 => ItemPosition::kNormal,
            3 => ItemPosition::kStretch,
            4 => ItemPosition::kBaseline,
            5 => ItemPosition::kLastBaseline,
            6 => ItemPosition::kAnchorCenter,
            7 => ItemPosition::kCenter,
            8 => ItemPosition::kStart,
            9 => ItemPosition::kEnd,
            10 => ItemPosition::kSelfStart,
            11 => ItemPosition::kSelfEnd,
            12 => ItemPosition::kFlexStart,
            13 => ItemPosition::kFlexEnd,
            14 => ItemPosition::kLeft,
            15 => ItemPosition::kRight,
            _ => unreachable!(),
        }
    }

    // cpp: layoutng_style/style/style_self_alignment_data.h:42-44
    pub fn PositionType(&self) -> ItemPositionType {
        if self.bits_ & Self::POSITION_TYPE_MASK == 0 {
            ItemPositionType::kNonLegacy
        } else {
            ItemPositionType::kLegacy
        }
    }

    // cpp: layoutng_style/style/style_self_alignment_data.h:45-47
    pub fn Overflow(&self) -> OverflowAlignment {
        match (self.bits_ & Self::OVERFLOW_MASK) >> 5 {
            0 => OverflowAlignment::kDefault,
            1 => OverflowAlignment::kUnsafe,
            2 => OverflowAlignment::kSafe,
            _ => unreachable!("constructor and setter accept only defined overflow values"),
        }
    }
}

// Derived equality above compares exactly the encoded bit fields.
