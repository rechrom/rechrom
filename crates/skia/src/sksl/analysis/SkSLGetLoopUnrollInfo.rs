// UNVERIFIED: no compilation, tests, syntax or source checks were run.
use crate::src::{
    core::{
        SkFloatingPoint::{sk_double_saturate2int, sk_ieee_double_divide},
        SkSafeMath::SkSafeMath,
    },
    sksl::{
        ir::{
            SkSLBinaryExpression::BinaryExpression,
            SkSLExpression::{Expression, ExpressionExt, Kind as ExprKind},
            SkSLForStatement::LoopUnrollInfo,
            SkSLIRNode::{IRNode, IRNodeBox},
            SkSLPostfixExpression::PostfixExpression,
            SkSLPrefixExpression::PrefixExpression,
            SkSLStatement::Statement,
            SkSLVarDeclarations::VarDeclaration,
            SkSLVariableReference::VariableReference,
        },
        SkSLConstantFolder::ConstantFolder,
        SkSLContext::Context,
        SkSLErrorReporter::{ErrorReporter, ErrorReporterExt},
        SkSLOperator::{Kind, Operator},
        SkSLPosition::{ForLoopPositions, Position},
    },
};
use core::ptr::NonNull;
// Source: src/sksl/analysis/SkSLGetLoopUnrollInfo.cpp:36-52
const kLoopTerminationLimit: i32 = 100000;
#[derive(Clone, Copy, PartialEq)]
enum Direction {
    Backwards,
    Forwards,
}
#[derive(Clone, Copy, PartialEq)]
enum Inclusive {
    No,
    Yes,
}
#[derive(Clone, Copy, PartialEq)]
enum LoopType {
    Float,
    Int,
}
// Source: src/sksl/analysis/SkSLGetLoopUnrollInfo.cpp:54-66
fn calculate_count_float(start: f64, end: f64, delta: f64, inclusive: Inclusive) -> i32 {
    let iterations = sk_ieee_double_divide(end - start, delta);
    let mut count = iterations.ceil();
    if inclusive == Inclusive::Yes && count == iterations {
        count += 1.0;
    }
    if count > kLoopTerminationLimit as f64 || !count.is_finite() {
        return kLoopTerminationLimit;
    }
    sk_double_saturate2int(count)
}
// Source: src/sksl/analysis/SkSLGetLoopUnrollInfo.cpp:68-85
fn calculate_count_int(start: i32, end: i32, delta: i32, inclusive: Inclusive) -> i32 {
    if delta == 0 {
        return kLoopTerminationLimit;
    }
    let mut math = SkSafeMath::new();
    let roundUp = if delta > 0 {
        math.subInt(delta, 1)
    } else {
        math.addInt(delta, 1)
    };
    let width = math.subInt(end, start);
    let rounded = math.addInt(width, roundUp);
    let mut iterations = math.divInt(rounded, delta);
    if inclusive == Inclusive::Yes && math.modInt(width, delta) == 0 {
        iterations = math.addInt(iterations, 1);
    }
    let advance = math.mulInt(delta, iterations);
    math.addInt(start, advance);
    if !math.ok() || iterations < 0 || iterations > kLoopTerminationLimit {
        return kLoopTerminationLimit;
    }
    iterations
}
// Source: src/sksl/analysis/SkSLGetLoopUnrollInfo.cpp:87-106
fn calculate_count(
    start: f64,
    end: f64,
    delta: f64,
    dir: Direction,
    inclusive: Inclusive,
    kind: LoopType,
) -> i32 {
    if (dir == Direction::Forwards && start > end) || (dir == Direction::Backwards && start < end) {
        return 0;
    }
    if delta == 0.0
        || (delta > 0.0 && dir == Direction::Backwards)
        || (delta < 0.0 && dir == Direction::Forwards)
    {
        return kLoopTerminationLimit;
    }
    if kind == LoopType::Int {
        return calculate_count_int(start as i32, end as i32, delta as i32, inclusive);
    }
    calculate_count_float(start, end, delta, inclusive)
}
// Source: src/sksl/analysis/SkSLGetLoopUnrollInfo.cpp:108-121
fn calculate_count_neq_int(start: i32, end: i32, delta: i32) -> i32 {
    if delta == 0 {
        return kLoopTerminationLimit;
    }
    let mut math = SkSafeMath::new();
    let width = math.subInt(end, start);
    let iterations = math.divInt(width, delta);
    let advance = math.mulInt(delta, iterations);
    let lastValue = math.addInt(start, advance);
    if !math.ok() || lastValue != end || iterations < 0 || iterations > kLoopTerminationLimit {
        return kLoopTerminationLimit;
    }
    iterations
}
// Source: src/sksl/analysis/SkSLGetLoopUnrollInfo.cpp:123-135
fn calculate_count_neq_float(start: f64, end: f64, delta: f64) -> i32 {
    if delta == 0.0 {
        return kLoopTerminationLimit;
    }
    let iterations = sk_ieee_double_divide(end - start, delta);
    let count = iterations.ceil();
    if count < 0.0 || count != iterations || !iterations.is_finite() {
        return kLoopTerminationLimit;
    }
    sk_double_saturate2int(count)
}
// Source: src/sksl/analysis/SkSLGetLoopUnrollInfo.cpp:137-142
fn calculate_count_neq(start: f64, end: f64, delta: f64, kind: LoopType) -> i32 {
    if kind == LoopType::Int {
        calculate_count_neq_int(start as i32, end as i32, delta as i32)
    } else {
        calculate_count_neq_float(start, end, delta)
    }
}
// Source: src/sksl/analysis/SkSLGetLoopUnrollInfo.cpp:144-388
/// Safety: context, IR references, reporter and index-variable backing retain the native compiler
/// leases; loopStatement is nonnull as required by the native body-write analysis.
pub unsafe fn GetLoopUnrollInfo(
    context: &Context,
    loopPos: Position,
    positions: &ForLoopPositions,
    loopInitializer: Option<&dyn Statement>,
    loopTest: &mut Option<IRNodeBox<dyn Expression>>,
    loopNext: Option<&dyn Expression>,
    loopStatement: Option<&dyn Statement>,
    errorPtr: Option<NonNull<dyn ErrorReporter>>,
) -> Option<Box<LoopUnrollInfo>> {
    let mut unused = super::SkSLNoOpErrorReporter::NoOpErrorReporter::new();
    let errors: &mut dyn ErrorReporter = match errorPtr {
        Some(mut ptr) => ptr.as_mut(),
        None => &mut unused,
    };
    let mut info = Box::new(LoopUnrollInfo {
        fIndex: None,
        fStart: 0.0,
        fDelta: 0.0,
        fCount: 0,
    });
    macro_rules! fail {
        ($pos:expr,$msg:expr) => {{
            errors.error($pos, $msg);
            return None;
        }};
    }
    let Some(init) = loopInitializer else {
        fail!(
            if positions.initPosition.valid() {
                positions.initPosition
            } else {
                loopPos
            },
            b"missing init declaration"
        );
    };
    if !init.node().is::<VarDeclaration>() {
        fail!(init.node().fPosition, b"invalid init declaration");
    }
    let decl = init.node().as_node::<VarDeclaration>();
    if !decl.baseType().isNumber() {
        fail!(init.node().fPosition, b"invalid type for loop index");
    }
    if decl.arraySize() != 0 {
        fail!(init.node().fPosition, b"invalid type for loop index");
    }
    let Some(value) = decl.value() else {
        fail!(init.node().fPosition, b"missing loop index initializer");
    };
    if !ConstantFolder::GetConstantValue(&**value, &mut info.fStart) {
        fail!(
            init.node().fPosition,
            b"loop index initializer must be a constant expression"
        );
    }
    info.fIndex = decl.var();
    let is_index = |expr: &Option<IRNodeBox<dyn Expression>>| {
        let expr = expr.as_ref().unwrap_unchecked();
        expr.node().is::<VariableReference>()
            && expr
                .node()
                .as_node::<VariableReference>()
                .variable()
                .map(|v| v.as_ptr() as *mut ())
                == info.fIndex.map(|v| v.as_ptr() as *mut ())
    };
    let Some(test) = loopTest.as_ref() else {
        fail!(
            if positions.conditionPosition.valid() {
                positions.conditionPosition
            } else {
                loopPos
            },
            b"missing condition"
        );
    };
    if !test.node().is::<BinaryExpression>() {
        fail!(test.node().fPosition, b"invalid condition");
    }
    let cond = test.node().as_node::<BinaryExpression>();
    if !is_index(cond.left()) {
        fail!(
            cond.node().fPosition,
            b"expected loop index on left hand side of condition"
        );
    }
    let op = cond.getOperator().kind();
    if !matches!(
        op,
        Kind::GT | Kind::GTEQ | Kind::LT | Kind::LTEQ | Kind::EQEQ | Kind::NEQ
    ) {
        fail!(cond.node().fPosition, b"invalid relational operator");
    }
    let mut end = 0.0;
    if !ConstantFolder::GetConstantValue(&**cond.right().as_ref().unwrap_unchecked(), &mut end) {
        fail!(
            cond.node().fPosition,
            b"loop index must be compared with a constant expression"
        );
    }
    let Some(next) = loopNext else {
        fail!(
            if positions.nextPosition.valid() {
                positions.nextPosition
            } else {
                loopPos
            },
            b"missing loop expression"
        );
    };
    match next.kind() {
        ExprKind::kBinary => {
            let n = next.node().as_node::<BinaryExpression>();
            if !is_index(n.left()) {
                fail!(
                    next.node().fPosition,
                    b"expected loop index in loop expression"
                );
            }
            if !ConstantFolder::GetConstantValue(
                &**n.right().as_ref().unwrap_unchecked(),
                &mut info.fDelta,
            ) {
                fail!(
                    next.node().fPosition,
                    b"loop index must be modified by a constant expression"
                );
            }
            match n.getOperator().kind() {
                Kind::PLUSEQ => {}
                Kind::MINUSEQ => info.fDelta = -info.fDelta,
                _ => fail!(
                    next.node().fPosition,
                    b"invalid operator in loop expression"
                ),
            }
        }
        ExprKind::kPrefix => {
            let n = next.node().as_node::<PrefixExpression>();
            if !is_index(n.operand()) {
                fail!(
                    next.node().fPosition,
                    b"expected loop index in loop expression"
                );
            }
            match n.getOperator().kind() {
                Kind::PLUSPLUS => info.fDelta = 1.0,
                Kind::MINUSMINUS => info.fDelta = -1.0,
                _ => fail!(
                    next.node().fPosition,
                    b"invalid operator in loop expression"
                ),
            }
        }
        ExprKind::kPostfix => {
            let n = next.node().as_node::<PostfixExpression>();
            if !is_index(n.operand()) {
                fail!(
                    next.node().fPosition,
                    b"expected loop index in loop expression"
                );
            }
            match n.getOperator().kind() {
                Kind::PLUSPLUS => info.fDelta = 1.0,
                Kind::MINUSMINUS => info.fDelta = -1.0,
                _ => fail!(
                    next.node().fPosition,
                    b"invalid operator in loop expression"
                ),
            }
        }
        _ => fail!(next.node().fPosition, b"invalid loop expression"),
    }
    let statement = loopStatement.unwrap_unchecked();
    if crate::src::sksl::SkSLAnalysis::StatementWritesToVariable(
        statement,
        decl.var().unwrap_unchecked().as_ref(),
    ) {
        fail!(
            statement.node().fPosition,
            b"loop index must not be modified within body of the loop"
        );
    }
    info.fCount = 0;
    let kind = if decl.baseType().isInteger() {
        debug_assert!(decl.baseType().bitWidth() == 32);
        LoopType::Int
    } else {
        debug_assert!(decl.baseType().isFloat());
        LoopType::Float
    };
    info.fCount = match op {
        Kind::LT => calculate_count(
            info.fStart,
            end,
            info.fDelta,
            Direction::Forwards,
            Inclusive::No,
            kind,
        ),
        Kind::GT => calculate_count(
            info.fStart,
            end,
            info.fDelta,
            Direction::Backwards,
            Inclusive::No,
            kind,
        ),
        Kind::LTEQ => calculate_count(
            info.fStart,
            end,
            info.fDelta,
            Direction::Forwards,
            Inclusive::Yes,
            kind,
        ),
        Kind::GTEQ => calculate_count(
            info.fStart,
            end,
            info.fDelta,
            Direction::Backwards,
            Inclusive::Yes,
            kind,
        ),
        Kind::NEQ => {
            let count = calculate_count_neq(info.fStart, end, info.fDelta, kind);
            if info
                .fIndex
                .unwrap_unchecked()
                .as_ref()
                .symbol_type()
                .componentType()
                .isFloat()
            {
                let op = if info.fDelta > 0.0 {
                    Kind::LT
                } else {
                    Kind::GT
                };
                let pos = cond.node().fPosition;
                let left = cond
                    .left()
                    .as_ref()
                    .unwrap_unchecked()
                    .clone_current()
                    .unwrap_unchecked();
                let right = cond
                    .right()
                    .as_ref()
                    .unwrap_unchecked()
                    .clone_current()
                    .unwrap_unchecked();
                *loopTest = Some(BinaryExpression::Make(
                    context,
                    pos,
                    left,
                    Operator::new(op),
                    right,
                ));
            }
            count
        }
        Kind::EQEQ => {
            if info.fStart == end {
                if info.fDelta != 0.0 {
                    1
                } else {
                    kLoopTerminationLimit
                }
            } else {
                0
            }
        }
        _ => core::hint::unreachable_unchecked(),
    };
    debug_assert!(info.fCount >= 0);
    if info.fCount >= kLoopTerminationLimit {
        fail!(
            loopPos,
            b"loop must guarantee termination in fewer iterations"
        );
    }
    Some(info)
}
