// C++: font_engine/fonts/font_data.h. The abstract virtual base maps to a
// trait; concrete GC tracing remains with each derived font data type.
use super::simple_font_data::SimpleFontData;
use foundation::Traceable;

// cpp: font_engine/fonts/font_data.h:36-55
pub trait FontData: Traceable {
    fn FontDataForCharacter(&self, character: u32) -> *const SimpleFontData;
    fn IsCustomFont(&self) -> bool;
    fn IsLoading(&self) -> bool;
    fn IsLoadingFallback(&self) -> bool;
    fn IsSegmented(&self) -> bool;
    fn ShouldSkipDrawing(&self) -> bool;
}
