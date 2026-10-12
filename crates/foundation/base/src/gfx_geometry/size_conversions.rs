// C++: src/foundation/gfx_geometry/size_conversions.h

use super::size::Size;
use super::size_f::SizeF;

// C++ declarations are implemented in size_conversions_impl.rs.
pub fn ToFlooredSize(size: SizeF) -> Size {
    super::size_conversions_impl::ToFlooredSize(size)
}

pub fn ToCeiledSize(size: SizeF) -> Size {
    super::size_conversions_impl::ToCeiledSize(size)
}

pub fn ToRoundedSize(size: SizeF) -> Size {
    super::size_conversions_impl::ToRoundedSize(size)
}
