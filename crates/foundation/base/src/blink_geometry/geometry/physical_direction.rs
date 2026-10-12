// C++: src/foundation/blink_geometry/geometry/physical_direction.h

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PhysicalDirection {
    kUp = 0,
    kRight,
    kDown,
    kLeft,
}
