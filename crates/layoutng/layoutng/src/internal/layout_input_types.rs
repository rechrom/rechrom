pub use foundation::PaintColor as Color;

// cpp: layoutng/internal/layout_input_types.h:19-23
pub use foundation::LayoutIntSize as IntSize;

// cpp: layoutng/internal/layout_input_types.h:25-37
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollbarThemeMetrics {
    pub auto_thickness: i32,
    pub thin_thickness: i32,
    pub minimum_thumb_length: i32,
    pub has_buttons: bool,
    pub uses_overlay_scrollbars: bool,
    /// ScrollbarTheme::IsSolidColor(). Native macOS scrollbars are painted;
    /// keep the platform choice as input rather than inferring it in cc.
    pub is_solid_color: bool,
    /// Selects NativeThemeMac thumb rasterization for the painted path.
    pub uses_macos_painted_scrollbars: bool,
    /// ScrollbarTheme::AllowsHitTest(). Most desktop themes allow it; mobile
    /// overlay themes may disable scrollbar hit testing entirely.
    pub allows_hit_test: bool,
    /// ScrollbarTheme::ShouldDisableInvisibleScrollbars(). Platform themes
    /// resolve this before Layout; the macOS theme supplies false.
    pub should_disable_invisible_scrollbars: bool,
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
            is_solid_color: true,
            uses_macos_painted_scrollbars: false,
            allows_hit_test: true,
            should_disable_invisible_scrollbars: true,
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
