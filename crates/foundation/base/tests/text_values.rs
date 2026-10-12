use foundation::layout_features::runtime_enabled_features::RuntimeEnabledFeatures;
use foundation::text::native::character_property::CharacterProperty;
use foundation::text::native::east_asian_spacing_type::EastAsianSpacingType;
use foundation::text::native::han_kerning_char_type::HanKerningCharType;
use foundation::text::native::tab_size::{TabSize, TabSizeValueType};
use foundation::text::native::unicode_bidi::{IsIsolated, IsOverride, UnicodeBidi};

#[test]
fn character_property_packing_matches_cpp_probe() {
    assert_eq!(std::mem::size_of::<CharacterProperty>(), 2);
    let mut property = CharacterProperty::new();
    property.set_is_cjk_ideograph_or_symbol(true);
    assert_eq!(property.AsUnsigned(), 1);
    property = CharacterProperty::new();
    property.set_is_potential_custom_element_name_char(true);
    assert_eq!(property.AsUnsigned(), 2);
    property = CharacterProperty::new();
    property.set_is_bidi_control(true);
    assert_eq!(property.AsUnsigned(), 4);
    property = CharacterProperty::new();
    property.set_is_hangul(true);
    assert_eq!(property.AsUnsigned(), 8);
    property = CharacterProperty::new();
    property.set_han_kerning(HanKerningCharType::kCloseQuote);
    assert_eq!(property.AsUnsigned(), 160);
    property = CharacterProperty::new();
    property.set_east_asian_spacing(EastAsianSpacingType::kWide);
    assert_eq!(property.AsUnsigned(), 768);
}

#[test]
fn tab_size_keeps_source_feature_branch() {
    assert!(RuntimeEnabledFeatures::TabSizeWithSpacingEnabled());
    assert_eq!(std::mem::size_of::<TabSize>(), 8);
    let spaces = TabSize::spaces(4.0);
    assert_eq!(spaces.GetPixelSize(8.0, 1.0, 2.0), 44.0);
    let length = TabSize::new(12.0, TabSizeValueType::kLength);
    assert_eq!(length.GetPixelSize(8.0, 1.0, 2.0), 12.0);
}

#[test]
fn unicode_bidi_helpers_match_cpp() {
    assert!(IsIsolated(&UnicodeBidi::kPlaintext));
    assert!(!IsIsolated(&UnicodeBidi::kBidiOverride));
    assert!(IsOverride(UnicodeBidi::kIsolateOverride));
    assert!(!IsOverride(UnicodeBidi::kIsolate));
}
