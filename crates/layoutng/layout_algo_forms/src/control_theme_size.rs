#![allow(non_snake_case)]

use foundation::{gfx, PhysicalSize, UnsupportedLayout};
use layoutng_assembly::internal::layout_input_types::ControlThemeMetrics;
use layoutng_geometry::geometry::logical_size::{LogicalSize, ToLogicalSize};
use layoutng_style::style::appearance::AppearanceValue;
use layoutng_style::style::computed_style::ComputedStyle;

// cpp: layoutng_forms/control_theme_size.cc:16-33
fn ControlThemeSize(metrics: &ControlThemeMetrics, part: AppearanceValue) -> gfx::Size {
    let size = match part {
        AppearanceValue::kCheckbox => &metrics.checkbox,
        AppearanceValue::kRadio => &metrics.radio,
        _ => panic!("Appearance has no intrinsic theme part size"),
    };
    let size = size.unwrap_or_else(|| {
        std::panic::panic_any(UnsupportedLayout::new(
            if part == AppearanceValue::kCheckbox {
                "Checkbox dimensions require ConstraintSpace::control_theme.checkbox"
            } else {
                "Radio dimensions require ConstraintSpace::control_theme.radio"
            },
        ))
    });
    gfx::Size::new(size.width, size.height)
}

// cpp: layoutng_forms/control_theme_size.h:11-13
// cpp: layoutng_forms/control_theme_size.cc:35-40
pub fn ThemePartIntrinsicSize(
    style: &ComputedStyle,
    metrics: &ControlThemeMetrics,
    part: AppearanceValue,
) -> LogicalSize {
    let mut size = PhysicalSize::FromSize(&ControlThemeSize(metrics, part));
    size.ScaleFloat(style.EffectiveZoom());
    ToLogicalSize(size, style.GetWritingMode())
}
