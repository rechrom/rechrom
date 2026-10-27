//! Small, engine-neutral style values carried across Layout package boundaries.

/// Physical box edges in retained Layout/Paint coordinates.
// cpp: layoutng/internal/layout_input.h:59-64
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LayoutEdges {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

/// Linear extended-sRGB payload used by Layout and Paint boundary records.
// cpp: layoutng/internal/layout_input_types.h:10-17
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PaintColor {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

/// Writing modes stored in the public Layout -> Paint artifact. Sideways
/// modes intentionally collapse to their physical vertical direction before
/// crossing this boundary; native Layout keeps the five-value `WritingMode`.
// cpp: layoutng/internal/layout_input.h:54-57
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum LayoutPaintWritingMode {
    kHorizontalTb,
    kVerticalRl,
    kVerticalLr,
}

impl From<crate::WritingMode> for LayoutPaintWritingMode {
    fn from(mode: crate::WritingMode) -> Self {
        match mode {
            crate::WritingMode::kHorizontalTb => Self::kHorizontalTb,
            crate::WritingMode::kVerticalRl | crate::WritingMode::kSidewaysRl => Self::kVerticalRl,
            crate::WritingMode::kVerticalLr | crate::WritingMode::kSidewaysLr => Self::kVerticalLr,
        }
    }
}

/// Raster smoothing selected while Layout resolves the concrete font face.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum FontSmoothing {
    kAuto,
    kNone,
    kAntialiased,
    kSubpixelAntialiased,
}

/// One resolved OpenType variation coordinate retained for Paint.
// cpp: layoutng/internal/layout_input.h:1018-1023
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FontVariation {
    pub tag: u32,
    pub value: f32,
}

/// Used CSS border style after cascade and layout resolution.
// cpp: layoutng/internal/layout_input.h:641-653
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BorderLineStyle {
    kNone,
    kHidden,
    kSolid,
    kDashed,
    kDotted,
    kDouble,
    kGroove,
    kRidge,
    kInset,
    kOutset,
}

/// Resolved native control appearance. `kAuto` must be resolved before layout.
// cpp: layoutng_style/style/appearance.h:30-61
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum AppearanceValue {
    kNone,
    kAuto,
    kCheckbox,
    kRadio,
    kButton,
    kListbox,
    kMediaControl,
    kMenulist,
    kMenulistButton,
    kMeter,
    kProgressBar,
    kSearchField,
    kTextField,
    kTextArea,
    kInnerSpinButton,
    kMediaSlider,
    kMediaSliderThumb,
    kMediaVolumeSlider,
    kMediaVolumeSliderThumb,
    kPushButton,
    kSquareButton,
    kSliderHorizontal,
    kSliderThumbHorizontal,
    kSliderThumbVertical,
    kSearchFieldCancelButton,
    kSliderVertical,
    kBaseSelect,
    kBase,
}

/// DOM form-control classification exported to Layout and Paint.
// cpp: layoutng/internal/form_control_types.h:11-42
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum FormControlType {
    kButtonButton = 0,
    kButtonSubmit = 1,
    kButtonReset = 2,
    kButtonPopover = 3,
    kFieldset = 4,
    kInputButton = 5,
    kInputCheckbox = 6,
    kInputColor = 7,
    kInputDate = 8,
    kInputDatetimeLocal = 9,
    kInputEmail = 10,
    kInputFile = 11,
    kInputHidden = 12,
    kInputImage = 13,
    kInputMonth = 14,
    kInputNumber = 15,
    kInputPassword = 16,
    kInputRadio = 17,
    kInputRange = 18,
    kInputReset = 19,
    kInputSearch = 20,
    kInputSubmit = 21,
    kInputTelephone = 22,
    kInputText = 23,
    kInputTime = 24,
    kInputUrl = 25,
    kInputWeek = 26,
    kOutput = 27,
    kSelectOne = 28,
    kSelectMultiple = 29,
    kTextArea = 30,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_preserve_empty_boundary_values() {
        assert_eq!(LayoutEdges::default().left, 0.0);
        assert_eq!(PaintColor::default().alpha, 0.0);
        assert_ne!(
            LayoutPaintWritingMode::kHorizontalTb,
            LayoutPaintWritingMode::kVerticalRl
        );
        assert_eq!(
            LayoutPaintWritingMode::from(crate::WritingMode::kSidewaysRl),
            LayoutPaintWritingMode::kVerticalRl
        );
        assert_eq!(
            LayoutPaintWritingMode::from(crate::WritingMode::kSidewaysLr),
            LayoutPaintWritingMode::kVerticalLr
        );
        assert_ne!(FontSmoothing::kAuto, FontSmoothing::kNone);
        assert_eq!(FontVariation::default().tag, 0);
        assert_ne!(BorderLineStyle::kNone, BorderLineStyle::kSolid);
        assert_ne!(AppearanceValue::kNone, AppearanceValue::kAuto);
        assert_eq!(FormControlType::kTextArea as i32, 30);
    }
}
