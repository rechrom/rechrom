// C++: src/foundation/blink_geometry/geometry/margin_strut.h

use super::layout_unit::LayoutUnit;

// C++ struct MarginStrut -> Rust struct + impl; fields preserve source order.
#[derive(Clone, Copy, Debug, Default)]
pub struct MarginStrut {
    pub positive_margin: LayoutUnit,
    pub negative_margin: LayoutUnit,
    pub quirky_positive_margin: LayoutUnit,
    pub is_quirky_container_start: bool,
    pub discard_margins: bool,
    pub trim_leading_margins: bool,
}

impl MarginStrut {
    // C++: void Append(const LayoutUnit& value, bool is_quirky), in .cc.
    // Rust definition is in margin_strut_impl.rs.

    pub fn Sum(&self) -> LayoutUnit {
        if self.discard_margins {
            return LayoutUnit::new();
        }
        self.quirky_positive_margin.max(self.positive_margin) + self.negative_margin
    }

    pub fn QuirkyContainerSum(&self) -> LayoutUnit {
        if self.discard_margins {
            return LayoutUnit::new();
        }
        self.positive_margin + self.negative_margin
    }
}
