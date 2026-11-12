//! Minimal value-only input to OverscrollAreaTracker from the native pseudo
//! scroll area. Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15,
//! overscroll_area_tracker.cc:94-171. No DOM or Layout owner crosses here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverscrollAction {
    Toggle,
    Open,
    Close,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OverscrollOffset {
    pub x: f32,
    pub y: f32,
}
#[derive(Clone, Copy, Debug)]
pub struct OverscrollRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
/// Projection of at(0), at(1), and TargetSnapAreaElementIds from the same live
/// SnapContainerData. No SnapContainerData cache is stored in DOM or Browser.
#[derive(Clone, Copy, Debug)]
pub struct OverscrollToggleTargets {
    pub first_area_id: u64,
    pub target_x_id: u64,
    pub target_y_id: u64,
}
#[derive(Clone, Copy, Debug)]
pub struct OverscrollSnapData {
    pub area_count: usize,
    pub toggle_targets: Option<OverscrollToggleTargets>,
    pub second_area_rect: Option<OverscrollRect>,
}
#[derive(Clone, Copy, Debug)]
pub struct OverscrollOpenBox {
    pub scroll_origin: OverscrollOffset,
    pub content_box_width: f32,
    pub content_box_height: f32,
}
#[derive(Clone, Copy, Debug)]
pub struct OverscrollGeometry {
    pub snap: Option<OverscrollSnapData>,
    pub open_box: Option<OverscrollOpenBox>,
}
