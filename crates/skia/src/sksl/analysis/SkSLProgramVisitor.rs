// v58 additions are UNVERIFIED; user prohibited all validation.
// v51 inherited expression dispatcher UNVERIFIED; user prohibited validation.
use crate::src::sksl::ir::{
    SkSLExpression::Expression, SkSLIRNode::IRNodeBox, SkSLProgramElement::ProgramElement,
    SkSLStatement::Statement,
};
// Source: src/sksl/analysis/SkSLProgramVisitor.h:34-46
// Adapted virtual protocol: default recursive implementations in Analysis.cpp
// use their real concrete IR types below. No false/empty traversal substitute.
// Safety: implementors preserve native const visitation and propagation semantics.
pub unsafe trait ProgramVisitor {
    fn visitExpression(&mut self, expression: &dyn Expression) -> bool;
    fn visitStatement(&mut self, statement: &dyn Statement) -> bool;
    fn visitProgramElement(&mut self, programElement: &dyn ProgramElement) -> bool;
}
// Source: src/sksl/analysis/SkSLProgramVisitor.h:67-69
pub fn visitExpressionPtr(
    visitor: &mut dyn ProgramVisitor,
    e: &Option<IRNodeBox<dyn Expression>>,
) -> bool {
    visitor.visitExpression(&**e.as_ref().expect("native nonnull expression"))
}
// Source: src/sksl/analysis/SkSLProgramVisitor.h:70-72
pub fn visitStatementPtr(
    visitor: &mut dyn ProgramVisitor,
    s: &Option<IRNodeBox<dyn Statement>>,
) -> bool {
    visitor.visitStatement(&**s.as_ref().expect("native nonnull statement"))
}

// Source: src/sksl/SkSLAnalysis.cpp:563-638
// Adapted native virtual `this`: callback performs the derived visitExpression override.
// Actual ChildCall and PostfixExpression owners are translated in v54.
pub unsafe fn visitExpression(
    visit: &mut dyn FnMut(&dyn Expression) -> bool,
    e: &dyn Expression,
) -> bool {
    use crate::src::sksl::ir::{
        SkSLBinaryExpression::BinaryExpression,
        SkSLChildCall::ChildCall,
        SkSLExpression::{ExpressionExt, Kind},
        SkSLFieldAccess::FieldAccess,
        SkSLFunctionCall::FunctionCall,
        SkSLIndexExpression::IndexExpression,
        SkSLPostfixExpression::PostfixExpression,
        SkSLPrefixExpression::PrefixExpression,
        SkSLSwizzle::Swizzle,
        SkSLTernaryExpression::TernaryExpression,
    };
    match e.kind() {
        Kind::kEmpty
        | Kind::kFunctionReference
        | Kind::kLiteral
        | Kind::kMethodReference
        | Kind::kPoison
        | Kind::kSetting
        | Kind::kTypeReference
        | Kind::kVariableReference => false,
        Kind::kBinary => {
            let b = e.node().as_node::<BinaryExpression>();
            (b.left().is_some() && visit(&**b.left().as_ref().unwrap_unchecked()))
                || (b.right().is_some() && visit(&**b.right().as_ref().unwrap_unchecked()))
        }
        Kind::kChildCall => {
            let c = e.node().as_node::<ChildCall>();
            for arg in c.arguments().as_base().as_slice() {
                if let Some(arg) = arg {
                    if visit(&**arg) {
                        return true;
                    }
                }
            }
            false
        }
        Kind::kConstructorArray
        | Kind::kConstructorArrayCast
        | Kind::kConstructorCompound
        | Kind::kConstructorCompoundCast
        | Kind::kConstructorDiagonalMatrix
        | Kind::kConstructorMatrixResize
        | Kind::kConstructorScalarCast
        | Kind::kConstructorSplat
        | Kind::kConstructorStruct => {
            let c = e.constructor_view();
            for arg in c.argumentSpan() {
                if visit(&**arg.as_ref().unwrap_unchecked()) {
                    return true;
                }
            }
            false
        }
        Kind::kFieldAccess => visit(
            &**e.node()
                .as_node::<FieldAccess>()
                .base()
                .as_ref()
                .unwrap_unchecked(),
        ),
        Kind::kFunctionCall => {
            let c = e.node().as_node::<FunctionCall>();
            for arg in c.arguments().as_base().as_slice() {
                if let Some(arg) = arg {
                    if visit(&**arg) {
                        return true;
                    }
                }
            }
            false
        }
        Kind::kIndex => {
            let i = e.node().as_node::<IndexExpression>();
            visit(&**i.base().as_ref().unwrap_unchecked())
                || visit(&**i.index().as_ref().unwrap_unchecked())
        }
        Kind::kPostfix => visit(
            &**e.node()
                .as_node::<PostfixExpression>()
                .operand()
                .as_ref()
                .unwrap_unchecked(),
        ),
        Kind::kPrefix => visit(
            &**e.node()
                .as_node::<PrefixExpression>()
                .operand()
                .as_ref()
                .unwrap_unchecked(),
        ),
        Kind::kSwizzle => {
            let ss = e.node().as_node::<Swizzle>();
            ss.base().is_some() && visit(&**ss.base().as_ref().unwrap_unchecked())
        }
        Kind::kTernary => {
            let t = e.node().as_node::<TernaryExpression>();
            visit(&**t.test().as_ref().unwrap_unchecked())
                || (t.ifTrue().is_some() && visit(&**t.ifTrue().as_ref().unwrap_unchecked()))
                || (t.ifFalse().is_some() && visit(&**t.ifFalse().as_ref().unwrap_unchecked()))
        }
        _ => core::hint::unreachable_unchecked(),
    }
}
// Source: src/sksl/SkSLAnalysis.cpp:640-696
// Native two virtual recursion channels remain distinct, with callbacks invoking actual overrides.
pub unsafe fn visitStatement(
    visitStmt: &mut dyn FnMut(&dyn Statement) -> bool,
    visitExpr: &mut dyn FnMut(&dyn Expression) -> bool,
    s: &dyn Statement,
) -> bool {
    use crate::src::sksl::ir::{
        SkSLBlock::Block, SkSLDoStatement::DoStatement,
        SkSLExpressionStatement::ExpressionStatement, SkSLForStatement::ForStatement,
        SkSLIfStatement::IfStatement, SkSLReturnStatement::ReturnStatement, SkSLStatement::Kind,
        SkSLSwitchCase::SwitchCase, SkSLSwitchStatement::SwitchStatement,
        SkSLVarDeclarations::VarDeclaration,
    };
    match s.kind() {
        Kind::kBreak | Kind::kContinue | Kind::kDiscard | Kind::kNop => false,
        Kind::kBlock => {
            for stmt in s.node().as_node::<Block>().children().as_slice() {
                if let Some(stmt) = stmt {
                    if visitStmt(&**stmt) {
                        return true;
                    }
                }
            }
            false
        }
        Kind::kSwitchCase => visitStmt(
            &**s.node()
                .as_node::<SwitchCase>()
                .statement()
                .as_ref()
                .unwrap_unchecked(),
        ),
        Kind::kDo => {
            let d = s.node().as_node::<DoStatement>();
            visitExpr(&**d.test().as_ref().unwrap_unchecked())
                || visitStmt(&**d.statement().as_ref().unwrap_unchecked())
        }
        Kind::kExpression => visitExpr(
            &**s.node()
                .as_node::<ExpressionStatement>()
                .expression()
                .as_ref()
                .unwrap_unchecked(),
        ),
        Kind::kFor => {
            let f = s.node().as_node::<ForStatement>();
            f.initializer().as_ref().is_some_and(|v| visitStmt(&**v))
                || f.test().as_ref().is_some_and(|v| visitExpr(&**v))
                || f.next().as_ref().is_some_and(|v| visitExpr(&**v))
                || visitStmt(&**f.statement().as_ref().unwrap_unchecked())
        }
        Kind::kIf => {
            let i = s.node().as_node::<IfStatement>();
            i.test().as_ref().is_some_and(|v| visitExpr(&**v))
                || i.ifTrue().as_ref().is_some_and(|v| visitStmt(&**v))
                || i.ifFalse().as_ref().is_some_and(|v| visitStmt(&**v))
        }
        Kind::kReturn => s
            .node()
            .as_node::<ReturnStatement>()
            .expression()
            .as_ref()
            .is_some_and(|v| visitExpr(&**v)),
        Kind::kSwitch => {
            let sw = s.node().as_node::<SwitchStatement>();
            visitExpr(&**sw.value().as_ref().unwrap_unchecked())
                || visitStmt(&**sw.caseBlock().as_ref().unwrap_unchecked())
        }
        Kind::kVarDeclaration => s
            .node()
            .as_node::<VarDeclaration>()
            .value()
            .as_ref()
            .is_some_and(|v| visitExpr(&**v)),
        _ => core::hint::unreachable_unchecked(),
    }
}

// Source: src/sksl/SkSLAnalysis.cpp:698-718
// Real FunctionDefinition header owner/getter and GlobalVarDeclaration are used.
// Leaf false results are exactly the native exhaustive leaf cases, not missing IR.
pub unsafe fn visitProgramElement(
    visitStmt: &mut dyn FnMut(&dyn Statement) -> bool,
    pe: &dyn ProgramElement,
) -> bool {
    use crate::src::sksl::ir::{
        SkSLFunctionDefinition::FunctionDefinition, SkSLProgramElement::Kind,
        SkSLVarDeclarations::GlobalVarDeclaration,
    };
    match pe.kind() {
        Kind::kExtension
        | Kind::kFunctionPrototype
        | Kind::kInterfaceBlock
        | Kind::kModifiers
        | Kind::kStructDefinition => false,
        Kind::kFunction => visitStmt(
            &**pe
                .node()
                .as_node::<FunctionDefinition>()
                .body()
                .as_ref()
                .unwrap_unchecked(),
        ),
        Kind::kGlobalVar => visitStmt(&**pe.node().as_node::<GlobalVarDeclaration>().declaration()),
        _ => core::hint::unreachable_unchecked(),
    }
}
// Rust dynamic-this adapters for the already actual inherited recursive methods.
// Safety: callbacks are synchronous; one visitor lease is reborrowed sequentially,
// never held by both callbacks at once. No replacement traversal or generic false.
pub unsafe fn inheritedExpression(visitor: &mut dyn ProgramVisitor, e: &dyn Expression) -> bool {
    visitExpression(&mut |child| visitor.visitExpression(child), e)
}
pub unsafe fn inheritedStatement(visitor: &mut dyn ProgramVisitor, s: &dyn Statement) -> bool {
    let this = visitor as *mut dyn ProgramVisitor;
    visitStatement(
        &mut |stmt| (*this).visitStatement(stmt),
        &mut |expr| (*this).visitExpression(expr),
        s,
    )
}
pub unsafe fn inheritedProgramElement(
    visitor: &mut dyn ProgramVisitor,
    pe: &dyn ProgramElement,
) -> bool {
    visitProgramElement(&mut |stmt| visitor.visitStatement(stmt), pe)
}
