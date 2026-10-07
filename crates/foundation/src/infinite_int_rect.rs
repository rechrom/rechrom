#![allow(non_snake_case)]

use crate::{gfx, LayoutUnit};

// cpp: foundation/blink_geometry/geometry/infinite_int_rect.h:20-32
pub fn InfiniteIntRect() -> gfx::Rect {
    let infinite_xy = LayoutUnit::Min().ToInt() / 4;
    let infinite_wh = LayoutUnit::Max().ToInt() / 2;
    debug_assert!(infinite_xy >= -(1 << f32::MANTISSA_DIGITS));
    debug_assert!(infinite_xy + infinite_wh <= 1 << f32::MANTISSA_DIGITS);
    gfx::Rect::new(
        gfx::Point::new(infinite_xy, infinite_xy),
        gfx::Size::new(infinite_wh, infinite_wh),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covers_layout_unit_range_without_exceeding_exact_float_integers() {
        let rect = InfiniteIntRect();
        assert!(rect.x() < 0 && rect.y() < 0);
        assert!(rect.width() > 0 && rect.height() > 0);
        assert!(rect.x() + rect.width() <= 1 << f32::MANTISSA_DIGITS);
    }
}
