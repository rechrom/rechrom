// cpp: layoutng/internal/layout_input_types.h:10-17
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

// cpp: layoutng/internal/layout_input_types.h:19-23
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IntSize {
    pub width: i32,
    pub height: i32,
}

// cpp: layoutng/internal/layout_input_types.h:25-37
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollbarThemeMetrics {
    pub auto_thickness: i32,
    pub thin_thickness: i32,
    pub minimum_thumb_length: i32,
    pub has_buttons: bool,
    pub uses_overlay_scrollbars: bool,
    pub track_color: Color,
    pub thumb_color: Color,
    pub button_color: Color,
    pub corner_color: Color,
}

impl Default for ScrollbarThemeMetrics {
    fn default() -> Self {
        Self {
            auto_thickness: 0,
            thin_thickness: 0,
            minimum_thumb_length: 0,
            has_buttons: false,
            uses_overlay_scrollbars: false,
            track_color: Color {
                red: 0.94,
                green: 0.94,
                blue: 0.94,
                alpha: 1.0,
            },
            thumb_color: Color {
                red: 0.55,
                green: 0.55,
                blue: 0.55,
                alpha: 1.0,
            },
            button_color: Color {
                red: 0.82,
                green: 0.82,
                blue: 0.82,
                alpha: 1.0,
            },
            corner_color: Color {
                red: 0.94,
                green: 0.94,
                blue: 0.94,
                alpha: 1.0,
            },
        }
    }
}

// cpp: layoutng/internal/layout_input_types.h:39-43
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ControlThemeMetrics {
    pub checkbox: Option<IntSize>,
    pub radio: Option<IntSize>,
}
