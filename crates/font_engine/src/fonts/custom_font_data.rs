// C++: font_engine/fonts/custom_font_data.h. The virtual interface becomes a
// trait. A concrete zero-state base instance preserves the C++ class's
// instantiable default behavior; derived font resources implement the trait.
use foundation::{Traceable, Visitor};

// cpp: font_engine/fonts/custom_font_data.h:32-47
pub trait CustomFontData: Traceable {
    fn BeginLoadIfNeeded(&self) {}
    fn IsLoading(&self) -> bool {
        false
    }
    fn IsLoadingFallback(&self) -> bool {
        false
    }
    fn ShouldSkipDrawing(&self) -> bool {
        false
    }
    fn IsPendingDataUrl(&self) -> bool {
        false
    }
}

#[derive(Default)]
pub struct BasicCustomFontData;

impl Traceable for BasicCustomFontData {
    fn Trace(&self, _visitor: &mut Visitor<'_>) {}
}

impl CustomFontData for BasicCustomFontData {}
