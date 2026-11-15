// C++: src/foundation/gfx_geometry/size_conversions.cc

use super::size::Size;
use super::size_f::SizeF;

fn ClampToInt(value: f32) -> i32 {
    if value.is_nan() {
        0
    } else if value >= i32::MAX as f32 {
        i32::MAX
    } else if value <= i32::MIN as f32 {
        i32::MIN
    } else {
        value as i32
    }
}

pub fn ToFlooredSize(size: SizeF) -> Size {
    Size::new(
        ClampToInt(size.width().floor()),
        ClampToInt(size.height().floor()),
    )
}

pub fn ToCeiledSize(size: SizeF) -> Size {
    Size::new(
        ClampToInt(size.width().ceil()),
        ClampToInt(size.height().ceil()),
    )
}

pub fn ToRoundedSize(size: SizeF) -> Size {
    Size::new(
        ClampToInt(size.width().round()),
        ClampToInt(size.height().round()),
    )
}
