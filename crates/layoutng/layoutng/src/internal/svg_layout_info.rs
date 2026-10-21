// cpp: layoutng/internal/svg_layout_info.h:10-14
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SVGLayoutInfo {
    pub force_layout: bool,
    pub scale_factor_changed: bool,
    pub viewport_changed: bool,
}

// cpp: layoutng/internal/svg_layout_info.h:16-23
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SVGLayoutResult {
    pub bounds_changed: bool,
    pub has_viewport_dependence: bool,
}

impl SVGLayoutResult {
    pub fn new(bounds_changed: bool, has_viewport_dependence: bool) -> Self {
        Self {
            bounds_changed,
            has_viewport_dependence,
        }
    }
}
