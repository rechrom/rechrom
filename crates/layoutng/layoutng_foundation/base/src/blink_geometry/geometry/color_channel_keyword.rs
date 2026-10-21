// C++: src/foundation/blink_geometry/geometry/color_channel_keyword.h

// C++ enum class ColorChannelKeyword -> Rust repr(C) enum.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColorChannelKeyword {
    kA,
    kB,
    kC,
    kG,
    kH,
    kL,
    kR,
    kS,
    kW,
    kX,
    kY,
    kZ,
    kAlpha,
}
