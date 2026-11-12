//! Neutral source direction shared by spatial-navigation and native geometry.
//! Chromium core/page/spatial_navigation.h:45.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpatialNavigationDirection {
    None,
    Up,
    Right,
    Down,
    Left,
}
