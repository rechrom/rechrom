// cpp: font_engine/fonts/font_smoothing_mode.h:34-39
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontSmoothingMode {
    kAutoSmoothing,
    kNoSmoothing,
    kAntialiased,
    kSubpixelAntialiased,
}

// ToString at font_smoothing_mode.h:41 has no supplied definition.
