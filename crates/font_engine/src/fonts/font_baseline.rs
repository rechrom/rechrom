// cpp: font_engine/fonts/font_baseline.h:29-57
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontBaseline {
    kAlphabeticBaseline,
    kCentralBaseline,
    kTextUnderBaseline,
    kIdeographicUnderBaseline,
    kXMiddleBaseline,
    kMathBaseline,
    kHangingBaseline,
    kTextOverBaseline,
}
