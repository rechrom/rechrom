use foundation::blink_geometry::geometry::path_types::{
    SkFillTypeToWindRule, WebCoreWindRuleToSkFillType, WindRule,
};
use foundation::blink_geometry::geometry::sk_path_types::{
    SkPathFillType, SkPathFillType_ConvertToNonInverse, SkPathFillType_IsEvenOdd,
    SkPathFillType_IsInverse, SkPathFillType_ToggleInverse,
};

#[test]
fn fill_type_bit_functions_preserve_cpp_discriminants() {
    let types = [
        SkPathFillType::kWinding,
        SkPathFillType::kEvenOdd,
        SkPathFillType::kInverseWinding,
        SkPathFillType::kInverseEvenOdd,
    ];
    for (index, fill_type) in types.iter().copied().enumerate() {
        assert_eq!(fill_type as usize, index);
        assert_eq!(SkPathFillType_IsEvenOdd(fill_type), index & 1 != 0);
        assert_eq!(SkPathFillType_IsInverse(fill_type), index & 2 != 0);
        assert_eq!(SkPathFillType_ToggleInverse(fill_type) as usize, index ^ 2);
        assert_eq!(
            SkPathFillType_ConvertToNonInverse(fill_type) as usize,
            index & 1
        );
    }
    assert_eq!(
        WebCoreWindRuleToSkFillType(WindRule::RULE_NONZERO),
        SkPathFillType::kWinding
    );
    assert_eq!(
        SkFillTypeToWindRule(SkPathFillType::kEvenOdd),
        WindRule::RULE_EVENODD
    );
}
