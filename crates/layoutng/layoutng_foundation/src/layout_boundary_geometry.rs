//! Engine-neutral geometry carried across Layout input/fragment boundaries.
//!
//! These values deliberately contain no LayoutObject or fragment identity, so
//! their owner must not participate in the LayoutNG ↔ fragment type cycle.

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LayoutOffset {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LayoutSize {
    pub width: f64,
    pub height: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LayoutIntSize {
    pub width: i32,
    pub height: i32,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum LayoutOverscrollType {
    kNone,
    kTransform,
}

/// Host viewport facts carried in the public Layout/Paint artifact.
// cpp: layoutng/internal/layout_input.h:756-767
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayoutViewportGeometry {
    pub size: LayoutIntSize,
    pub root_scroller_visible_size: Option<LayoutIntSize>,
    pub transition_snapshot_size: Option<LayoutIntSize>,
    pub overscroll_type: LayoutOverscrollType,
}

impl Default for LayoutViewportGeometry {
    fn default() -> Self {
        Self {
            size: LayoutIntSize::default(),
            root_scroller_visible_size: None,
            transition_snapshot_size: None,
            overscroll_type: LayoutOverscrollType::kNone,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_defaults_to_an_empty_non_overscrolled_host() {
        let viewport = LayoutViewportGeometry::default();
        assert_eq!(viewport.size, LayoutIntSize::default());
        assert!(viewport.root_scroller_visible_size.is_none());
        assert_eq!(viewport.overscroll_type, LayoutOverscrollType::kNone);
    }
}
