// C++: src/foundation/blink_geometry/geometry/logical_direction.h

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LogicalDirection {
    kBlockStart = 0,
    kBlockEnd,
    kInlineStart,
    kInlineEnd,
}
