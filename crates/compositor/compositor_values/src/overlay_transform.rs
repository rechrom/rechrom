// Source dependency: ui/gfx/overlay_transform.h gfx::OverlayTransform.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OverlayTransform {
    #[default]
    None,
    FlipHorizontal,
    FlipVertical,
    Rotate90,
    Rotate180,
    Rotate270,
    FlipVerticalRotate90,
    FlipVerticalRotate270,
    Invalid,
}
