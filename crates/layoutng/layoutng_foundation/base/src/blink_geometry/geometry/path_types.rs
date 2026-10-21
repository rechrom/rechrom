// C++: src/foundation/blink_geometry/geometry/path_types.h

use super::sk_path_types::SkPathFillType;

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LineCap {
    kButtCap = 0,
    kRoundCap = 1,
    kSquareCap = 2,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LineJoin {
    kMiterJoin = 0,
    kRoundJoin = 1,
    kBevelJoin = 2,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WindRule {
    RULE_NONZERO = SkPathFillType::kWinding as i32,
    RULE_EVENODD = SkPathFillType::kEvenOdd as i32,
}

pub fn WebCoreWindRuleToSkFillType(rule: WindRule) -> SkPathFillType {
    match rule {
        WindRule::RULE_NONZERO => SkPathFillType::kWinding,
        WindRule::RULE_EVENODD => SkPathFillType::kEvenOdd,
    }
}

pub fn SkFillTypeToWindRule(fill_type: SkPathFillType) -> WindRule {
    match fill_type {
        SkPathFillType::kWinding => WindRule::RULE_NONZERO,
        SkPathFillType::kEvenOdd => WindRule::RULE_EVENODD,
        _ => panic!("NOTREACHED: inverse path fill type is not a WindRule"),
    }
}
