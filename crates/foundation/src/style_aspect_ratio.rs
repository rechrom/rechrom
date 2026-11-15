// C++: src/foundation/style_values/style/style_aspect_ratio.h:17-64
// C++: src/foundation/style_values/style/style_aspect_ratio.cc:12-15
use crate::{gfx, LayoutRatioFromSizeF, PhysicalSize};

// C++: style_aspect_ratio.h:17
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EAspectRatioType {
    kAuto,
    kAutoAndRatio,
    kRatio,
}

// C++: style_aspect_ratio.h:19-63. The two-bit C++ field is a u8 here;
// its three valid values remain the same, while Rust does not expose a bitfield.
#[derive(Clone, Copy, Debug)]
pub struct StyleAspectRatio {
    type_: u8,
    ratio_: gfx::SizeF,
    layout_ratio_: PhysicalSize,
}

#[allow(non_snake_case)]
impl StyleAspectRatio {
    // C++: style_aspect_ratio.cc:12-15
    pub fn new(kind: EAspectRatioType, ratio: gfx::SizeF) -> Self {
        Self {
            type_: kind as u8,
            ratio_: ratio,
            layout_ratio_: LayoutRatioFromSizeF(ratio),
        }
    }

    // C++: style_aspect_ratio.h:27-32
    pub fn GetType(&self) -> EAspectRatioType {
        if self.layout_ratio_.IsEmpty() {
            return EAspectRatioType::kAuto;
        }
        self.GetTypeForComputedStyle()
    }

    // C++: style_aspect_ratio.h:34-36
    pub fn GetTypeForComputedStyle(&self) -> EAspectRatioType {
        match self.type_ {
            0 => EAspectRatioType::kAuto,
            1 => EAspectRatioType::kAutoAndRatio,
            2 => EAspectRatioType::kRatio,
            _ => unreachable!("aspect-ratio bitfield contains no enum value"),
        }
    }

    // C++: style_aspect_ratio.h:38
    pub fn IsAuto(&self) -> bool {
        self.GetType() == EAspectRatioType::kAuto
    }

    // C++: style_aspect_ratio.h:46-47
    pub fn GetRatio(&self) -> gfx::SizeF {
        self.ratio_
    }

    pub fn GetLayoutRatio(&self) -> PhysicalSize {
        self.layout_ratio_
    }
}

// C++: style_aspect_ratio.h:49-51. The cached fixed-point ratio is deliberately
// excluded from equality, matching the source operator.
impl PartialEq for StyleAspectRatio {
    fn eq(&self, other: &Self) -> bool {
        self.type_ == other.type_ && self.ratio_ == other.ratio_
    }
}
