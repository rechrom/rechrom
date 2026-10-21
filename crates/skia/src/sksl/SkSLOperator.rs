// Source: src/sksl/SkSLOperator.h:19-55
// Adapted transparent8 value retains every native enum cast identity; named
// constants and all classifications below follow original source order.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OperatorKind(pub u8);
impl OperatorKind {
    pub const PLUS: Self = Self(0);
    pub const MINUS: Self = Self(1);
    pub const STAR: Self = Self(2);
    pub const SLASH: Self = Self(3);
    pub const PERCENT: Self = Self(4);
    pub const SHL: Self = Self(5);
    pub const SHR: Self = Self(6);
    pub const LOGICALNOT: Self = Self(7);
    pub const LOGICALAND: Self = Self(8);
    pub const LOGICALOR: Self = Self(9);
    pub const LOGICALXOR: Self = Self(10);
    pub const BITWISENOT: Self = Self(11);
    pub const BITWISEAND: Self = Self(12);
    pub const BITWISEOR: Self = Self(13);
    pub const BITWISEXOR: Self = Self(14);
    pub const EQ: Self = Self(15);
    pub const EQEQ: Self = Self(16);
    pub const NEQ: Self = Self(17);
    pub const LT: Self = Self(18);
    pub const GT: Self = Self(19);
    pub const LTEQ: Self = Self(20);
    pub const GTEQ: Self = Self(21);
    pub const PLUSEQ: Self = Self(22);
    pub const MINUSEQ: Self = Self(23);
    pub const STAREQ: Self = Self(24);
    pub const SLASHEQ: Self = Self(25);
    pub const PERCENTEQ: Self = Self(26);
    pub const SHLEQ: Self = Self(27);
    pub const SHREQ: Self = Self(28);
    pub const BITWISEANDEQ: Self = Self(29);
    pub const BITWISEOREQ: Self = Self(30);
    pub const BITWISEXOREQ: Self = Self(31);
    pub const PLUSPLUS: Self = Self(32);
    pub const MINUSMINUS: Self = Self(33);
    pub const COMMA: Self = Self(34);
}
// Source: src/sksl/SkSLOperator.h:57-77
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OperatorPrecedence(pub u8);
impl OperatorPrecedence {
    pub const kParentheses: Self = Self(1);
    pub const kPostfix: Self = Self(2);
    pub const kPrefix: Self = Self(3);
    pub const kMultiplicative: Self = Self(4);
    pub const kAdditive: Self = Self(5);
    pub const kShift: Self = Self(6);
    pub const kRelational: Self = Self(7);
    pub const kEquality: Self = Self(8);
    pub const kBitwiseAnd: Self = Self(9);
    pub const kBitwiseXor: Self = Self(10);
    pub const kBitwiseOr: Self = Self(11);
    pub const kLogicalAnd: Self = Self(12);
    pub const kLogicalXor: Self = Self(13);
    pub const kLogicalOr: Self = Self(14);
    pub const kTernary: Self = Self(15);
    pub const kAssignment: Self = Self(16);
    pub const kSequence: Self = Self(17);
    pub const kExpression: Self = Self::kSequence;
    pub const kStatement: Self = Self(18);
}
// Source: src/sksl/SkSLOperator.h:81-81
pub type Kind = OperatorKind;
impl Operator {
    // Source: src/sksl/SkSLOperator.h:83-83
    pub fn new(op: Kind) -> Self {
        Self { fKind: op }
    }
    // Source: src/sksl/SkSLOperator.h:85-85
    pub fn kind(&self) -> Kind {
        self.fKind
    }
    // Source: src/sksl/SkSLOperator.h:87-89
    pub fn isEquality(&self) -> bool {
        self.fKind == Kind::EQEQ || self.fKind == Kind::NEQ
    }
    // Source: src/sksl/SkSLOperator.h:133-135
    pub fn isAllowedInStrictES2Mode(&self) -> bool {
        !self.isOnlyValidForIntegralTypes()
    }
}
// Source: src/sksl/SkSLOperator.h:153-153
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct Operator {
    fKind: Kind,
}
impl Operator {
    // Source: src/sksl/SkSLOperator.cpp:21-56
    pub fn getBinaryPrecedence(&self) -> OperatorPrecedence {
        match self.kind() {
            Kind::STAR | Kind::SLASH | Kind::PERCENT => OperatorPrecedence::kMultiplicative,
            Kind::PLUS | Kind::MINUS => OperatorPrecedence::kAdditive,
            Kind::SHL | Kind::SHR => OperatorPrecedence::kShift,
            Kind::LT | Kind::GT | Kind::LTEQ | Kind::GTEQ => OperatorPrecedence::kRelational,
            Kind::EQEQ | Kind::NEQ => OperatorPrecedence::kEquality,
            Kind::BITWISEAND => OperatorPrecedence::kBitwiseAnd,
            Kind::BITWISEXOR => OperatorPrecedence::kBitwiseXor,
            Kind::BITWISEOR => OperatorPrecedence::kBitwiseOr,
            Kind::LOGICALAND => OperatorPrecedence::kLogicalAnd,
            Kind::LOGICALXOR => OperatorPrecedence::kLogicalXor,
            Kind::LOGICALOR => OperatorPrecedence::kLogicalOr,
            Kind::EQ
            | Kind::PLUSEQ
            | Kind::MINUSEQ
            | Kind::STAREQ
            | Kind::SLASHEQ
            | Kind::PERCENTEQ
            | Kind::SHLEQ
            | Kind::SHREQ
            | Kind::BITWISEANDEQ
            | Kind::BITWISEXOREQ
            | Kind::BITWISEOREQ => OperatorPrecedence::kAssignment,
            Kind::COMMA => OperatorPrecedence::kSequence,
            _ => {
                eprintln!("unsupported binary operator");
                std::process::abort();
            }
        }
    }
    // Source: src/sksl/SkSLOperator.cpp:58-97
    pub fn operatorName(&self) -> &'static str {
        match self.kind() {
            Kind::PLUS => " + ",
            Kind::MINUS => " - ",
            Kind::STAR => " * ",
            Kind::SLASH => " / ",
            Kind::PERCENT => " % ",
            Kind::SHL => " << ",
            Kind::SHR => " >> ",
            Kind::LOGICALNOT => "!",
            Kind::LOGICALAND => " && ",
            Kind::LOGICALOR => " || ",
            Kind::LOGICALXOR => " ^^ ",
            Kind::BITWISENOT => "~",
            Kind::BITWISEAND => " & ",
            Kind::BITWISEOR => " | ",
            Kind::BITWISEXOR => " ^ ",
            Kind::EQ => " = ",
            Kind::EQEQ => " == ",
            Kind::NEQ => " != ",
            Kind::LT => " < ",
            Kind::GT => " > ",
            Kind::LTEQ => " <= ",
            Kind::GTEQ => " >= ",
            Kind::PLUSEQ => " += ",
            Kind::MINUSEQ => " -= ",
            Kind::STAREQ => " *= ",
            Kind::SLASHEQ => " /= ",
            Kind::PERCENTEQ => " %= ",
            Kind::SHLEQ => " <<= ",
            Kind::SHREQ => " >>= ",
            Kind::BITWISEANDEQ => " &= ",
            Kind::BITWISEOREQ => " |= ",
            Kind::BITWISEXOREQ => " ^= ",
            Kind::PLUSPLUS => "++",
            Kind::MINUSMINUS => "--",
            Kind::COMMA => ", ",
            _ => std::process::abort(),
        }
    }
    // Source: src/sksl/SkSLOperator.cpp:99-108
    pub fn tightOperatorName(&self) -> &'static str {
        let mut name = self.operatorName();
        if name.starts_with(' ') {
            name = &name[1..];
        }
        if name.ends_with(' ') {
            name = &name[..name.len() - 1];
        }
        name
    }
    // Source: src/sksl/SkSLOperator.cpp:110-127
    pub fn isAssignment(&self) -> bool {
        match self.kind() {
            Kind::EQ
            | Kind::PLUSEQ
            | Kind::MINUSEQ
            | Kind::STAREQ
            | Kind::SLASHEQ
            | Kind::PERCENTEQ
            | Kind::SHLEQ
            | Kind::SHREQ
            | Kind::BITWISEOREQ
            | Kind::BITWISEXOREQ
            | Kind::BITWISEANDEQ => true,
            _ => false,
        }
    }
    // Source: src/sksl/SkSLOperator.cpp:129-131
    pub fn isCompoundAssignment(&self) -> bool {
        self.isAssignment() && self.kind() != Kind::EQ
    }
    // Source: src/sksl/SkSLOperator.cpp:133-147
    pub fn removeAssignment(&self) -> Self {
        match self.kind() {
            Kind::PLUSEQ => Self::new(Kind::PLUS),
            Kind::MINUSEQ => Self::new(Kind::MINUS),
            Kind::STAREQ => Self::new(Kind::STAR),
            Kind::SLASHEQ => Self::new(Kind::SLASH),
            Kind::PERCENTEQ => Self::new(Kind::PERCENT),
            Kind::SHLEQ => Self::new(Kind::SHL),
            Kind::SHREQ => Self::new(Kind::SHR),
            Kind::BITWISEOREQ => Self::new(Kind::BITWISEOR),
            Kind::BITWISEXOREQ => Self::new(Kind::BITWISEXOR),
            Kind::BITWISEANDEQ => Self::new(Kind::BITWISEAND),
            _ => *self,
        }
    }
    // Source: src/sksl/SkSLOperator.cpp:149-159
    pub fn isRelational(&self) -> bool {
        match self.kind() {
            Kind::LT | Kind::GT | Kind::LTEQ | Kind::GTEQ => true,
            _ => false,
        }
    }
    // Source: src/sksl/SkSLOperator.cpp:161-179
    pub fn isOnlyValidForIntegralTypes(&self) -> bool {
        match self.kind() {
            Kind::SHL
            | Kind::SHR
            | Kind::BITWISEAND
            | Kind::BITWISEOR
            | Kind::BITWISEXOR
            | Kind::PERCENT
            | Kind::SHLEQ
            | Kind::SHREQ
            | Kind::BITWISEANDEQ
            | Kind::BITWISEOREQ
            | Kind::BITWISEXOREQ
            | Kind::PERCENTEQ => true,
            _ => false,
        }
    }
    // Source: src/sksl/SkSLOperator.cpp:181-207
    pub fn isValidForMatrixOrVector(&self) -> bool {
        match self.kind() {
            Kind::PLUS
            | Kind::MINUS
            | Kind::STAR
            | Kind::SLASH
            | Kind::PERCENT
            | Kind::SHL
            | Kind::SHR
            | Kind::BITWISEAND
            | Kind::BITWISEOR
            | Kind::BITWISEXOR
            | Kind::PLUSEQ
            | Kind::MINUSEQ
            | Kind::STAREQ
            | Kind::SLASHEQ
            | Kind::PERCENTEQ
            | Kind::SHLEQ
            | Kind::SHREQ
            | Kind::BITWISEANDEQ
            | Kind::BITWISEOREQ
            | Kind::BITWISEXOREQ => true,
            _ => false,
        }
    }
}

// Source: src/sksl/SkSLOperator.cpp:209-217
impl Operator {
    pub fn isMatrixMultiply(
        &self,
        left: &dyn crate::src::sksl::ir::SkSLType::Type,
        right: &dyn crate::src::sksl::ir::SkSLType::Type,
    ) -> bool {
        if self.kind() != Kind::STAR && self.kind() != Kind::STAREQ {
            return false;
        }
        if left.isMatrix() {
            return right.isMatrix() || right.isVector();
        }
        left.isVector() && right.isMatrix()
    }
    // Source: src/sksl/SkSLOperator.cpp:223-386
    // Adapted borrowed native Type** outputs retain write order, including failed conversions.
    pub unsafe fn determineBinaryType<'a>(
        &self,
        context: &'a crate::src::sksl::SkSLContext::Context,
        left: &'a dyn crate::src::sksl::ir::SkSLType::Type,
        right: &'a dyn crate::src::sksl::ir::SkSLType::Type,
        outLeftType: &mut &'a dyn crate::src::sksl::ir::SkSLType::Type,
        outRightType: &mut &'a dyn crate::src::sksl::ir::SkSLType::Type,
        outResultType: &mut &'a dyn crate::src::sksl::ir::SkSLType::Type,
    ) -> bool {
        use crate::src::sksl::ir::SkSLType::CoercionCost;
        let allowNarrowing = context
            .fConfig
            .expect("native config")
            .as_ref()
            .fSettings
            .fAllowNarrowingConversions;
        let boolean: &dyn crate::src::sksl::ir::SkSLType::Type = &**context.fTypes.as_ref().fBool;
        match self.kind() {
            Kind::EQ => {
                if left.isVoid() {
                    return false;
                }
                *outLeftType = left;
                *outRightType = left;
                *outResultType = left;
                return right.canCoerceTo(left, allowNarrowing);
            }
            Kind::EQEQ | Kind::NEQ => {
                if left.isVoid() || left.isOpaque() {
                    return false;
                }
                let rtl = right.coercionCost(left);
                let ltr = left.coercionCost(right);
                if rtl < ltr {
                    if rtl.isPossible(allowNarrowing) {
                        *outLeftType = left;
                        *outRightType = left;
                        *outResultType = boolean;
                        return true;
                    }
                } else if ltr.isPossible(allowNarrowing) {
                    *outLeftType = right;
                    *outRightType = right;
                    *outResultType = boolean;
                    return true;
                }
                return false;
            }
            Kind::LOGICALOR | Kind::LOGICALAND | Kind::LOGICALXOR => {
                *outLeftType = boolean;
                *outRightType = boolean;
                *outResultType = boolean;
                return left.canCoerceTo(boolean, allowNarrowing)
                    && right.canCoerceTo(boolean, allowNarrowing);
            }
            Kind::COMMA => {
                if left.isOpaque() || right.isOpaque() {
                    return false;
                }
                *outLeftType = left;
                *outRightType = right;
                *outResultType = right;
                return true;
            }
            _ => {}
        }
        let lc = left.componentType();
        let rc = right.componentType();
        if lc.isBoolean() || rc.isBoolean() {
            return false;
        }
        let assignment = self.isAssignment();
        if self.isMatrixMultiply(left, right) {
            if !self.determineBinaryType(
                context,
                left.componentType(),
                right.componentType(),
                outLeftType,
                outRightType,
                outResultType,
            ) {
                return false;
            }
            *outLeftType = outResultType.toCompound(context, left.columns(), left.rows());
            *outRightType = outResultType.toCompound(context, right.columns(), right.rows());
            let leftColumns = left.columns();
            let leftRows = left.rows();
            let mut rightColumns = right.columns();
            let mut rightRows = right.rows();
            if right.isVector() {
                std::mem::swap(&mut rightColumns, &mut rightRows);
                crate::SkASSERT!(@source b"src/sksl/SkSLOperator.cpp\0".as_ptr(),309; @condition b"rightColumns == 1\0".as_ptr(); (rightColumns)==(1));
            }
            *outResultType = if rightColumns > 1 {
                outResultType.toCompound(context, rightColumns, leftRows)
            } else {
                outResultType.toCompound(context, leftRows, rightColumns)
            };
            if assignment
                && (outResultType.columns() != leftColumns || outResultType.rows() != leftRows)
            {
                return false;
            }
            return leftColumns == rightRows;
        }
        let lv = left.isVector() || left.isMatrix();
        let valid = self.isValidForMatrixOrVector();
        if lv && valid && right.isScalar() {
            if !self.determineBinaryType(
                context,
                left.componentType(),
                right,
                outLeftType,
                outRightType,
                outResultType,
            ) {
                return false;
            }
            *outLeftType = outLeftType.toCompound(context, left.columns(), left.rows());
            if !self.isRelational() {
                *outResultType = outResultType.toCompound(context, left.columns(), left.rows());
            }
            return true;
        }
        let rv = right.isVector() || right.isMatrix();
        if !assignment && rv && valid && left.isScalar() {
            if !self.determineBinaryType(
                context,
                left,
                right.componentType(),
                outLeftType,
                outRightType,
                outResultType,
            ) {
                return false;
            }
            *outRightType = outRightType.toCompound(context, right.columns(), right.rows());
            if !self.isRelational() {
                *outResultType = outResultType.toCompound(context, right.columns(), right.rows());
            }
            return true;
        }
        let rtl = right.coercionCost(left);
        let ltr = if assignment {
            CoercionCost::Impossible()
        } else {
            left.coercionCost(right)
        };
        if (left.isScalar() && right.isScalar()) || (lv && valid) {
            if self.isOnlyValidForIntegralTypes() && (!lc.isInteger() || !rc.isInteger()) {
                return false;
            }
            if rtl.isPossible(allowNarrowing) && rtl < ltr {
                *outLeftType = left;
                *outRightType = left;
                *outResultType = left;
            } else if ltr.isPossible(allowNarrowing) {
                *outLeftType = right;
                *outRightType = right;
                *outResultType = right;
            } else {
                return false;
            }
            if self.isRelational() {
                *outResultType = boolean;
            }
            return true;
        }
        false
    }
}
