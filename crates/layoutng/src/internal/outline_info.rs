use layoutng_style::style::computed_style::ComputedStyle;

// cpp: layoutng/internal/outline_info.h:30-36
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LayoutOutlineInfo {
    pub width: i32,
    pub offset: i32,
}

fn clamp_to_i32(value: f32) -> i32 {
    debug_assert!(!value.is_nan());
    value as i32
}

#[allow(non_snake_case)]
impl LayoutOutlineInfo {
    // cpp: layoutng/internal/outline_info.cc:32-34
    pub fn GetFromStyle(style: &ComputedStyle) -> Self {
        Self {
            width: *style.OutlineWidth(),
            offset: *style.OutlineOffset(),
        }
    }

    // cpp: layoutng/internal/outline_info.cc:36-43
    pub fn getUnzoomedWidth(style: &ComputedStyle) -> f32 {
        let unzoomed_width = *style.OutlineWidth() as f32 / style.EffectiveZoom();
        if unzoomed_width > 0.0 && unzoomed_width <= 1.0 {
            return 1.0;
        }
        unzoomed_width.floor()
    }

    // cpp: layoutng/internal/outline_info.cc:45-49
    pub fn GetUnzoomedFromStyle(style: &ComputedStyle) -> Self {
        Self {
            width: clamp_to_i32(Self::getUnzoomedWidth(style)),
            offset: clamp_to_i32((*style.OutlineOffset() as f32 / style.EffectiveZoom()).floor()),
        }
    }
}
