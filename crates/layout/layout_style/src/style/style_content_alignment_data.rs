use super::computed_style_constants::{
    ContentDistributionType, ContentPosition, OverflowAlignment,
};

// cpp: layoutng_style/style/style_content_alignment_data.h:13-57
/// The C++ unsigned fields occupy 4, 3, and 2 bits respectively.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StyleContentAlignmentData {
    bits_: u32,
}

#[allow(non_snake_case)]
impl StyleContentAlignmentData {
    const POSITION_MASK: u32 = 0b1111;
    const DISTRIBUTION_MASK: u32 = 0b111 << 4;
    const OVERFLOW_MASK: u32 = 0b11 << 7;

    // cpp: layoutng_style/style/style_content_alignment_data.h:20-26
    pub const fn new_default_overflow(
        position: ContentPosition,
        distribution: ContentDistributionType,
    ) -> Self {
        Self::new(position, distribution, OverflowAlignment::kDefault)
    }

    // cpp: layoutng_style/style/style_content_alignment_data.h:20-26
    pub const fn new(
        position: ContentPosition,
        distribution: ContentDistributionType,
        overflow: OverflowAlignment,
    ) -> Self {
        Self {
            bits_: ((position as u32) & 0b1111)
                | (((distribution as u32) & 0b111) << 4)
                | (((overflow as u32) & 0b11) << 7),
        }
    }

    // cpp: layoutng_style/style/style_content_alignment_data.h:28-30
    pub fn SetPosition(&mut self, position: ContentPosition) {
        self.bits_ = (self.bits_ & !Self::POSITION_MASK) | ((position as u32) & 0b1111);
    }

    // cpp: layoutng_style/style/style_content_alignment_data.h:31-33
    pub fn SetDistribution(&mut self, distribution: ContentDistributionType) {
        self.bits_ =
            (self.bits_ & !Self::DISTRIBUTION_MASK) | (((distribution as u32) & 0b111) << 4);
    }

    // cpp: layoutng_style/style/style_content_alignment_data.h:34-36
    pub fn SetOverflow(&mut self, overflow: OverflowAlignment) {
        self.bits_ = (self.bits_ & !Self::OVERFLOW_MASK) | (((overflow as u32) & 0b11) << 7);
    }

    // cpp: layoutng_style/style/style_content_alignment_data.h:38-40
    pub fn GetPosition(&self) -> ContentPosition {
        match self.bits_ & Self::POSITION_MASK {
            0 => ContentPosition::kNormal,
            1 => ContentPosition::kBaseline,
            2 => ContentPosition::kLastBaseline,
            3 => ContentPosition::kCenter,
            4 => ContentPosition::kStart,
            5 => ContentPosition::kEnd,
            6 => ContentPosition::kFlexStart,
            7 => ContentPosition::kFlexEnd,
            8 => ContentPosition::kLeft,
            9 => ContentPosition::kRight,
            _ => unreachable!("constructor and setter accept only defined ContentPosition values"),
        }
    }

    // cpp: layoutng_style/style/style_content_alignment_data.h:41-43
    pub fn Distribution(&self) -> ContentDistributionType {
        match (self.bits_ & Self::DISTRIBUTION_MASK) >> 4 {
            0 => ContentDistributionType::kDefault,
            1 => ContentDistributionType::kSpaceBetween,
            2 => ContentDistributionType::kSpaceAround,
            3 => ContentDistributionType::kSpaceEvenly,
            4 => ContentDistributionType::kStretch,
            _ => unreachable!("constructor and setter accept only defined distribution values"),
        }
    }

    // cpp: layoutng_style/style/style_content_alignment_data.h:44-46
    pub fn Overflow(&self) -> OverflowAlignment {
        match (self.bits_ & Self::OVERFLOW_MASK) >> 7 {
            0 => OverflowAlignment::kDefault,
            1 => OverflowAlignment::kUnsafe,
            2 => OverflowAlignment::kSafe,
            _ => unreachable!("constructor and setter accept only defined overflow values"),
        }
    }
}

// Derived equality above compares exactly the encoded bit fields.
