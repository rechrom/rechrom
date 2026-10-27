pub use style_model::at_rule_descriptors::*;

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::{String, StringView};
    use style_model::css_property_names::CSSPropertyID;

    #[test]
    fn descriptor_names_aliases_and_property_conversions() {
        for &(name, alias) in super::super::at_rule_names::AT_RULE_NAMES.iter() {
            let canonical = AsAtRuleDescriptorID(StringView::from(name));
            assert_ne!(canonical, AtRuleDescriptorID::Invalid, "{name}");
            assert_ne!(canonical, AtRuleDescriptorID::Variable, "{name}");
            assert_eq!(
                CSSPropertyIDAsAtRuleDescriptor(AtRuleDescriptorIDAsCSSPropertyID(canonical)),
                canonical
            );
            let uppercase = name.to_ascii_uppercase();
            assert_eq!(
                AsAtRuleDescriptorID(StringView::from(uppercase.as_str())),
                canonical
            );
            let latin1 = String::from_latin1(name.as_bytes());
            assert_eq!(AsAtRuleDescriptorID(StringView::from(&latin1)), canonical);
            if !alias.is_empty() {
                assert_eq!(AsAtRuleDescriptorID(StringView::from(alias)), canonical);
            }
        }
        assert_eq!(
            CSSPropertyIDAsAtRuleDescriptor(CSSPropertyID::kColor),
            AtRuleDescriptorID::Invalid
        );
    }

    #[test]
    fn custom_descriptor_detection_precedes_length_and_character_validation() {
        for name in [
            "--x",
            "--💡",
            "--abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz",
        ] {
            assert_eq!(
                AsAtRuleDescriptorID(StringView::from(name)),
                AtRuleDescriptorID::Variable
            );
        }
        let units = String::from_utf16(&[45, 45, 0]);
        assert_eq!(
            AsAtRuleDescriptorID(StringView::from(&units)),
            AtRuleDescriptorID::Variable
        );
        assert_eq!(
            AsAtRuleDescriptorID(StringView::from("--")),
            AtRuleDescriptorID::Invalid
        );
    }

    #[test]
    fn hash_collision_and_invalid_character_rejection() {
        // "crs" has the same generated hash as "src"; the final name check is
        // required even after a successful hash-table lookup.
        for name in [
            "",
            "crs",
            "src ",
            "src\0",
            "src\u{7f}",
            "sṛc",
            "font-feature-settings-extra-long",
        ] {
            assert_eq!(
                AsAtRuleDescriptorID(StringView::from(name)),
                AtRuleDescriptorID::Invalid,
                "{name:?}"
            );
        }
        let surrogate = String::from_utf16(&[115, 114, 0xd800]);
        assert_eq!(
            AsAtRuleDescriptorID(StringView::from(&surrogate)),
            AtRuleDescriptorID::Invalid
        );
    }
}
