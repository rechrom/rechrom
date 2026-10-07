// C++: font_engine/fonts/typesetting_features.h. Its ToString declaration has
// no definition in the supplied source tree.
// cpp: font_engine/fonts/typesetting_features.h:36-45
pub const kMaxTypesettingFeatureIndex: u32 = 2;
#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TypesettingFeature {
    kKerning = 1,
    kLigatures = 2,
    kCaps = 1 << kMaxTypesettingFeatureIndex,
}
pub type TypesettingFeatures = u32;
pub const kKerning: TypesettingFeatures = TypesettingFeature::kKerning as u32;
pub const kLigatures: TypesettingFeatures = TypesettingFeature::kLigatures as u32;
pub const kCaps: TypesettingFeatures = TypesettingFeature::kCaps as u32;
