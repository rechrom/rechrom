// cpp: layoutng_geometry/geometry/overflow_clip_axes.h:7-17
/// Axes on which overflow is clipped, including control and paint clips.
pub type OverflowClipAxes = u32;

#[allow(non_upper_case_globals)]
pub const kNoOverflowClip: OverflowClipAxes = 0;
#[allow(non_upper_case_globals)]
pub const kOverflowClipX: OverflowClipAxes = 1 << 0;
#[allow(non_upper_case_globals)]
pub const kOverflowClipY: OverflowClipAxes = 1 << 1;
#[allow(non_upper_case_globals)]
pub const kOverflowClipBothAxis: OverflowClipAxes = kOverflowClipX | kOverflowClipY;
