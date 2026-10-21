use super::{
    ir::{SkSLIRNode::IRNodeBox, SkSLProgramElement::ProgramElement, SkSLSymbolTable::SymbolTable},
    SkSLCompiler::Compiler,
    SkSLContext::Context,
    SkSLErrorReporter::{ErrorReporter, ErrorReporterExt},
    SkSLLexer::{Kind, Lexer, Token},
    SkSLPosition::Position,
    SkSLProgramKind::ProgramKind,
    SkSLProgramSettings::{ProgramConfig, ProgramSettings},
};
use core::{
    mem::{ManuallyDrop, MaybeUninit},
    pin::Pin,
    ptr::NonNull,
};
// Source: src/sksl/SkSLParser.h:45-45
// Source: src/sksl/SkSLParser.h:319-319
// Source: src/sksl/SkSLParser.h:320-320
// Source: src/sksl/SkSLParser.h:321-321
// Source: src/sksl/SkSLParser.h:322-322
// Source: src/sksl/SkSLParser.h:323-323
// Source: src/sksl/SkSLParser.h:324-324
// Source: src/sksl/SkSLParser.h:325-325
// Source: src/sksl/SkSLParser.h:326-326
// Source: src/sksl/SkSLParser.h:329-329
// Source: src/sksl/SkSLParser.h:330-330
// Actual native field order. Constructor-uninitialized reporter/fatal flag stay MaybeUninit.
// The lexer borrows the same immutable heap text; neither parser moves nor token reads move it.
#[repr(C)]
pub struct Parser {
    fCompiler: NonNull<Compiler>,
    fSettings: ProgramSettings,
    fErrorReporter: MaybeUninit<NonNull<dyn ErrorReporter>>,
    fEncounteredFatalError: MaybeUninit<bool>,
    fKind: ProgramKind,
    fText: ManuallyDrop<Option<Box<Vec<u8>>>>,
    fProgramElements: ManuallyDrop<Vec<IRNodeBox<dyn ProgramElement>>>,
    fLexer: Lexer<'static>,
    fDepth: i32,
    fPushback: Token,
}
// Source: src/sksl/SkSLParser.h:163-168
// Native uninitialized prefix members remain MaybeUninit until their producer writes them.
#[repr(C)]
struct VarDeclarationsPrefix {
    fPosition: MaybeUninit<Position>,
    fModifiers: MaybeUninit<super::ir::SkSLModifiers::Modifiers>,
    fType: MaybeUninit<Option<NonNull<dyn super::ir::SkSLType::Type>>>,
    fName: MaybeUninit<Token>,
}
impl VarDeclarationsPrefix {
    fn uninit() -> Self {
        Self {
            fPosition: MaybeUninit::uninit(),
            fModifiers: MaybeUninit::uninit(),
            fType: MaybeUninit::uninit(),
            fName: MaybeUninit::uninit(),
        }
    }
}

impl Parser {
    // Source: src/sksl/SkSLParser.h:180-183
    pub fn allowUnsizedArrays(&self) -> bool {
        ProgramConfig::IsCompute(self.fKind)
            || ProgramConfig::IsFragment(self.fKind)
            || ProgramConfig::IsVertex(self.fKind)
    }
}

// v54 real Parser/token front-end owner UNVERIFIED; all validation prohibited.
// Source: src/sksl/SkSLParser.cpp:77-77
const kMaxParseDepth: i32 = 50;
// Source: src/sksl/SkSLParser.cpp:79-103
pub fn parse_modifier_token(
    token: super::SkSLLexer::Kind,
) -> super::ir::SkSLModifierFlags::ModifierFlags {
    use super::{
        ir::SkSLModifierFlags::{ModifierFlag, ModifierFlags},
        SkSLLexer::Kind,
    };
    ModifierFlags(match token {
        Kind::TK_UNIFORM => ModifierFlag::kUniform.0,
        Kind::TK_CONST => ModifierFlag::kConst.0,
        Kind::TK_IN => ModifierFlag::kIn.0,
        Kind::TK_OUT => ModifierFlag::kOut.0,
        Kind::TK_INOUT => ModifierFlag::kIn.0 | ModifierFlag::kOut.0,
        Kind::TK_FLAT => ModifierFlag::kFlat.0,
        Kind::TK_NOPERSPECTIVE => ModifierFlag::kNoPerspective.0,
        Kind::TK_PURE => ModifierFlag::kPure.0,
        Kind::TK_INLINE => ModifierFlag::kInline.0,
        Kind::TK_NOINLINE => ModifierFlag::kNoInline.0,
        Kind::TK_HIGHP => ModifierFlag::kHighp.0,
        Kind::TK_MEDIUMP => ModifierFlag::kMediump.0,
        Kind::TK_LOWP => ModifierFlag::kLowp.0,
        Kind::TK_EXPORT => ModifierFlag::kExport.0,
        Kind::TK_ES3 => ModifierFlag::kES3.0,
        Kind::TK_WORKGROUP => ModifierFlag::kWorkgroup.0,
        Kind::TK_READONLY => ModifierFlag::kReadOnly.0,
        Kind::TK_WRITEONLY => ModifierFlag::kWriteOnly.0,
        Kind::TK_BUFFER => ModifierFlag::kBuffer.0,
        Kind::TK_PIXELLOCAL => ModifierFlag::kPixelLocal.0,
        _ => ModifierFlag::kNone.0,
    })
}
// Source: src/sksl/SkSLParser.cpp:105-127
// Adapted RAII raw borrow permits native parser calls while depth guard exists.
pub struct AutoDepth {
    fParser: NonNull<Parser>,
    fDepth: i32,
}
impl AutoDepth {
    pub unsafe fn new(p: NonNull<Parser>) -> Self {
        Self {
            fParser: p,
            fDepth: 0,
        }
    }
    pub unsafe fn increase(&mut self) -> bool {
        self.fDepth += 1;
        let p = self.fParser.as_mut();
        p.fDepth += 1;
        if p.fDepth > kMaxParseDepth {
            let token = p.peek();
            p.error(token, b"exceeded max parse depth");
            p.fEncounteredFatalError.write(true);
            return false;
        }
        true
    }
}
impl Drop for AutoDepth {
    fn drop(&mut self) {
        unsafe {
            self.fParser.as_mut().fDepth -= self.fDepth;
        }
    }
}
// Source: src/sksl/SkSLParser.cpp:129-151
pub struct AutoSymbolTable {
    fParser: Option<NonNull<Parser>>,
}
impl AutoSymbolTable {
    pub unsafe fn new(
        p: NonNull<Parser>,
        newSymbolTable: &mut Option<Box<SymbolTable>>,
        enable: bool,
    ) -> Self {
        let mut this = Self { fParser: None };
        if enable {
            this.fParser = Some(p);
            let symbols = this.contextSymbolTable();
            let parent = symbols.unwrap_unchecked();
            *newSymbolTable = Some(Box::new(SymbolTable::with_parent(
                Some(parent),
                parent.as_ref().isBuiltin(),
            )));
            *symbols = Some(NonNull::from(
                newSymbolTable.as_mut().unwrap_unchecked().as_mut(),
            ));
        }
        this
    }
    unsafe fn contextSymbolTable(&mut self) -> &mut Option<NonNull<SymbolTable>> {
        let parser = self.fParser.unwrap_unchecked().as_ref();
        &mut (*parser.fCompiler.as_ref().context_ptr()).fSymbolTable
    }
}
impl Drop for AutoSymbolTable {
    fn drop(&mut self) {
        unsafe {
            if self.fParser.is_some() {
                let symbols = self.contextSymbolTable();
                *symbols = symbols.unwrap_unchecked().as_ref().parent();
            }
        }
    }
}
// Source: src/sksl/SkSLParser.cpp:153-218
// Native checkpoint owns its actual forwarding reporter. Placement keeps that reporter's
// Context-installed trait pointer stable. Drop only asserts accept/rewind, as native does.
#[repr(C)]
pub struct ParserCheckpoint {
    fParser: NonNull<Parser>,
    fPushbackCheckpoint: Token,
    fLexerCheckpoint: super::SkSLLexer::Checkpoint,
    fErrorReporter: ForwardingErrorReporter,
    fOldErrorReporter: Option<NonNull<dyn ErrorReporter>>,
    fOldEncounteredFatalError: bool,
    pin: core::marker::PhantomPinned,
}
struct ForwardedError {
    fMsg: Vec<u8>,
    fPos: Position,
}
struct ForwardingErrorReporter {
    base: super::SkSLErrorReporter::ErrorReporterData,
    fErrors: core::pin::Pin<Box<crate::include::private::SkTArray::TArray<ForwardedError, false>>>,
}
impl ForwardingErrorReporter {
    fn new() -> Self {
        Self {
            base: super::SkSLErrorReporter::ErrorReporterData::new(),
            fErrors: crate::include::private::SkTArray::TArray::new_pinned(),
        }
    }
    unsafe fn forwardErrors(&self, mut parser: NonNull<Parser>) {
        for error in self.fErrors.as_slice() {
            parser.as_mut().error_position(error.fPos, &error.fMsg);
        }
    }
}
impl ErrorReporter for ForwardingErrorReporter {
    fn state(&self) -> &super::SkSLErrorReporter::ErrorReporterData {
        &self.base
    }
    fn state_mut(&mut self) -> &mut super::SkSLErrorReporter::ErrorReporterData {
        &mut self.base
    }
    fn handleError(&mut self, msg: &[u8], pos: Position) {
        unsafe {
            self.fErrors.as_mut().push_back(ForwardedError {
                fMsg: msg.to_vec(),
                fPos: pos,
            });
        }
    }
}
impl ParserCheckpoint {
    /// Safety: parser/Compiler remain stable, native fatal flag has been initialized by declarations,
    /// and caller accepts or rewinds before destruction. Reporter callbacks follow this exact lease.
    pub unsafe fn new(parser: NonNull<Parser>) -> core::pin::Pin<Box<Self>> {
        let context = parser.as_ref().fCompiler.as_ref().context_ptr();
        let p = Box::into_raw(Box::<Self>::new_uninit()).cast::<Self>();
        core::ptr::addr_of_mut!((*p).fParser).write(parser);
        core::ptr::addr_of_mut!((*p).fPushbackCheckpoint).write(parser.as_ref().fPushback);
        core::ptr::addr_of_mut!((*p).fLexerCheckpoint)
            .write(parser.as_ref().fLexer.getCheckpoint());
        core::ptr::addr_of_mut!((*p).fErrorReporter).write(ForwardingErrorReporter::new());
        core::ptr::addr_of_mut!((*p).fOldErrorReporter).write(Some((*context).fErrors));
        core::ptr::addr_of_mut!((*p).fOldEncounteredFatalError)
            .write(parser.as_ref().fEncounteredFatalError.assume_init());
        core::ptr::addr_of_mut!((*p).pin).write(core::marker::PhantomPinned);
        let reporter = NonNull::from(
            &mut *core::ptr::addr_of_mut!((*p).fErrorReporter) as &mut dyn ErrorReporter
        );
        (*context).setErrorReporter(reporter);
        core::pin::Pin::new_unchecked(Box::from_raw(p))
    }
    pub unsafe fn accept(self: core::pin::Pin<&mut Self>) {
        let this = self.get_unchecked_mut();
        this.restoreErrorReporter();
        this.fErrorReporter.forwardErrors(this.fParser);
    }
    pub unsafe fn rewind(self: core::pin::Pin<&mut Self>) {
        let this = self.get_unchecked_mut();
        this.restoreErrorReporter();
        let parser = this.fParser.as_mut();
        parser.fPushback = this.fPushbackCheckpoint;
        parser.fLexer.rewindToCheckpoint(this.fLexerCheckpoint);
        parser
            .fEncounteredFatalError
            .write(this.fOldEncounteredFatalError);
    }
    unsafe fn restoreErrorReporter(&mut self) {
        debug_assert!(self.fOldErrorReporter.is_some());
        let context = self.fParser.as_ref().fCompiler.as_ref().context_ptr();
        (*context).setErrorReporter(self.fOldErrorReporter.unwrap_unchecked());
        self.fOldErrorReporter = None;
    }
}
impl Drop for ParserCheckpoint {
    fn drop(&mut self) {
        debug_assert!(
            self.fOldErrorReporter.is_none(),
            "Checkpoint was not accepted or rewound before destruction"
        );
    }
}
// Source: src/sksl/SkSLParser.cpp:220-230
impl Parser {
    /// Safety: Compiler is at its final address and outlives parser/guards; source is immutable.
    pub unsafe fn new(
        compiler: NonNull<Compiler>,
        settings: &ProgramSettings,
        kind: ProgramKind,
        text: Box<Vec<u8>>,
    ) -> Self {
        let input: &'static [u8] = core::slice::from_raw_parts(text.as_ptr(), text.len());
        Self {
            fCompiler: compiler,
            fSettings: *settings,
            fErrorReporter: MaybeUninit::uninit(),
            fEncounteredFatalError: MaybeUninit::uninit(),
            fKind: kind,
            fText: ManuallyDrop::new(Some(text)),
            fProgramElements: ManuallyDrop::new(Vec::new()),
            fLexer: Lexer::new(input),
            fDepth: 0,
            fPushback: Token::new(Kind::TK_NONE, -1, -1),
        }
    }
}
// Source: src/sksl/SkSLParser.cpp:232-232
// Native destruction destroys program elements before owned source; lexer is a trivial view.
impl Drop for Parser {
    fn drop(&mut self) {
        unsafe {
            while let Some(element) = self.fProgramElements.pop() {
                drop(element);
            }
            ManuallyDrop::drop(&mut self.fProgramElements);
            ManuallyDrop::drop(&mut self.fText);
        }
    }
}
impl Parser {
    // Source: src/sksl/SkSLParser.cpp:234-236
    pub unsafe fn symbolTable(&self) -> Option<NonNull<SymbolTable>> {
        self.fCompiler.as_ref().symbolTable()
    }
    // Source: src/sksl/SkSLParser.cpp:238-273
    pub unsafe fn nextRawToken(&mut self) -> Token {
        let mut token;
        if self.fPushback.fKind != Kind::TK_NONE {
            token = self.fPushback;
            self.fPushback.fKind = Kind::TK_NONE;
        } else {
            token = self.fLexer.next();
            match token.fKind {
                Kind::TK_PRIVATE_IDENTIFIER => {
                    if ProgramConfig::AllowsPrivateIdentifiers(self.fKind) {
                        token.fKind = Kind::TK_IDENTIFIER;
                    } else {
                        let mut msg = b"name '".to_vec();
                        msg.extend_from_slice(self.text(token));
                        msg.extend_from_slice(b"' is reserved");
                        self.error(token, &msg);
                        token.fKind = Kind::TK_IDENTIFIER;
                    }
                }
                Kind::TK_RESERVED => {
                    let mut msg = b"name '".to_vec();
                    msg.extend_from_slice(self.text(token));
                    msg.extend_from_slice(b"' is reserved");
                    self.error(token, &msg);
                    token.fKind = Kind::TK_IDENTIFIER;
                }
                Kind::TK_BAD_OCTAL => {
                    let mut msg = b"'".to_vec();
                    msg.extend_from_slice(self.text(token));
                    msg.extend_from_slice(b"' is not a valid octal number");
                    self.error(token, &msg);
                }
                _ => {}
            }
        }
        token
    }
}
// Source: src/sksl/SkSLParser.cpp:275-285
pub fn is_whitespace(kind: super::SkSLLexer::Kind) -> bool {
    use super::SkSLLexer::Kind;
    match kind {
        Kind::TK_WHITESPACE | Kind::TK_LINE_COMMENT | Kind::TK_BLOCK_COMMENT => true,
        _ => false,
    }
}

impl Parser {
    // Source: src/sksl/SkSLParser.cpp:287-301
    pub unsafe fn expectNewline(&mut self) -> bool {
        let token = self.nextRawToken();
        if token.fKind == Kind::TK_WHITESPACE {
            let tokenText = self.text(token);
            if tokenText.contains(&b'\r') || tokenText.contains(&b'\n') {
                return true;
            }
        }
        self.pushback(token);
        false
    }
    // Source: src/sksl/SkSLParser.cpp:303-310
    pub unsafe fn nextToken(&mut self) -> Token {
        loop {
            let token = self.nextRawToken();
            if !is_whitespace(token.fKind) {
                return token;
            }
        }
    }
    // Source: src/sksl/SkSLParser.cpp:312-315
    pub fn pushback(&mut self, t: Token) {
        debug_assert!(self.fPushback.fKind == Kind::TK_NONE);
        self.fPushback = t;
    }
    // Source: src/sksl/SkSLParser.cpp:317-322
    pub unsafe fn peek(&mut self) -> Token {
        if self.fPushback.fKind == Kind::TK_NONE {
            self.fPushback = self.nextToken();
        }
        self.fPushback
    }
    // Source: src/sksl/SkSLParser.cpp:324-337
    pub unsafe fn checkNext(&mut self, kind: Kind, result: Option<&mut Token>) -> bool {
        if self.fPushback.fKind != Kind::TK_NONE && self.fPushback.fKind != kind {
            return false;
        }
        let next = self.nextToken();
        if next.fKind == kind {
            if let Some(result) = result {
                *result = next;
            }
            return true;
        }
        self.pushback(next);
        false
    }
    // Source: src/sksl/SkSLParser.cpp:339-352
    pub unsafe fn expect(
        &mut self,
        kind: Kind,
        expected: &[u8],
        result: Option<&mut Token>,
    ) -> bool {
        let next = self.nextToken();
        if next.fKind == kind {
            if let Some(result) = result {
                *result = next;
            }
            true
        } else {
            let mut msg = b"expected ".to_vec();
            msg.extend_from_slice(expected);
            msg.extend_from_slice(b", but found '");
            msg.extend_from_slice(self.text(next));
            msg.push(b'\'');
            self.error(next, &msg);
            self.fEncounteredFatalError.write(true);
            false
        }
    }
    // Source: src/sksl/SkSLParser.cpp:354-365
    pub unsafe fn expectIdentifier(&mut self, result: &mut Token) -> bool {
        if !self.expect(Kind::TK_IDENTIFIER, b"an identifier", Some(result)) {
            return false;
        }
        if self
            .symbolTable()
            .unwrap_unchecked()
            .as_ref()
            .isBuiltinType(self.text(*result))
        {
            let mut msg = b"expected an identifier, but found type '".to_vec();
            msg.extend_from_slice(self.text(*result));
            msg.push(b'\'');
            self.error(*result, &msg);
            self.fEncounteredFatalError.write(true);
            return false;
        }
        true
    }
    // Source: src/sksl/SkSLParser.cpp:367-376
    pub unsafe fn checkIdentifier(&mut self, result: &mut Token) -> bool {
        if !self.checkNext(Kind::TK_IDENTIFIER, Some(result)) {
            return false;
        }
        if self
            .symbolTable()
            .unwrap_unchecked()
            .as_ref()
            .isBuiltinType(self.text(*result))
        {
            self.pushback(*result);
            return false;
        }
        true
    }
    // Source: src/sksl/SkSLParser.cpp:378-380
    pub unsafe fn text(&self, token: Token) -> &[u8] {
        let text = self.fText.as_ref().unwrap_unchecked();
        core::slice::from_raw_parts(
            text.as_ptr().offset(token.fOffset as isize),
            token.fLength as usize,
        )
    }
    // Source: src/sksl/SkSLParser.cpp:382-388
    pub fn position(t: Token) -> Position {
        if t.fOffset >= 0 {
            Position::Range(t.fOffset, t.fOffset + t.fLength)
        } else {
            Position::new()
        }
    }
    // Source: src/sksl/SkSLParser.cpp:390-392
    pub unsafe fn error(&mut self, token: Token, msg: &[u8]) {
        self.error_position(Self::position(token), msg);
    }
    // Source: src/sksl/SkSLParser.cpp:394-396
    pub unsafe fn error_position(&mut self, position: Position, msg: &[u8]) {
        let mut errors = self.fCompiler.as_ref().context().fErrors;
        errors.as_mut().error(position, msg);
    }
    // Source: src/sksl/SkSLParser.cpp:398-402
    pub fn rangeFrom(&self, start: Position) -> Position {
        let offset = if self.fPushback.fKind != Kind::TK_NONE {
            self.fPushback.fOffset
        } else {
            self.fLexer.getCheckpoint().fOffset
        };
        Position::Range(start.startOffset(), offset)
    }
    // Source: src/sksl/SkSLParser.cpp:404-406
    pub fn rangeFromToken(&self, start: Token) -> Position {
        self.rangeFrom(Self::position(start))
    }
}
use super::{
    ir::{
        SkSLBlock::{Block, Kind as BlockKind},
        SkSLBreakStatement::BreakStatement,
        SkSLContinueStatement::ContinueStatement,
        SkSLDiscardStatement::DiscardStatement,
        SkSLDoStatement::DoStatement,
        SkSLExpressionStatement::ExpressionStatement,
        SkSLForStatement::ForStatement,
        SkSLIfStatement::IfStatement,
        SkSLNop::Nop,
        SkSLReturnStatement::ReturnStatement,
        SkSLStatement::Statement,
        SkSLSwitchStatement::SwitchStatement,
    },
    SkSLDefines::StatementArray,
    SkSLPosition::ForLoopPositions,
};
type ParsedStatement = Option<IRNodeBox<dyn Statement>>;
impl Parser {
    // Frozen-scope declaration/array parsing continuation; UNVERIFIED, validation prohibited.
    // Source: src/sksl/SkSLParser.cpp:409-418
    // Source: src/sksl/SkSLParser.h:53-53
    pub unsafe fn programInheritingFrom(
        &mut self,
        _module: NonNull<super::SkSLModule::Module>,
    ) -> Option<Box<super::ir::SkSLProgram::Program>> {
        self.declarations();
        if self.fCompiler.as_ref().errorCount() == 0 {
            let source = self.fText.take();
            let elements = core::mem::take(&mut *self.fProgramElements);
            self.fCompiler.as_mut().releaseProgram(source, elements)
        } else {
            while let Some(element) = self.fProgramElements.pop() {
                drop(element);
            }
            None
        }
    }
    // Source: src/sksl/SkSLParser.cpp:420-429
    // Source: src/sksl/SkSLParser.h:55-55
    pub unsafe fn moduleInheritingFrom(
        &mut self,
        parentModule: NonNull<super::SkSLModule::Module>,
    ) -> Box<super::SkSLModule::Module> {
        self.declarations();
        let mut symbols = self.symbolTable().unwrap_unchecked();
        let text = self.fText.as_mut().unwrap_unchecked();
        symbols
            .as_mut()
            .takeOwnershipOfString(core::mem::take(text.as_mut()));
        let mut result = Box::new(super::SkSLModule::Module::default());
        result.fParent = Some(parentModule);
        *result.fSymbols = self.fCompiler.as_mut().takeGlobalSymbolsForModule();
        *result.fElements = core::mem::take(&mut *self.fProgramElements)
            .into_iter()
            .map(Some)
            .collect();
        result.fModuleType = self
            .fCompiler
            .as_ref()
            .context()
            .fConfig
            .unwrap_unchecked()
            .as_ref()
            .fModuleType;
        result
    }
    // Source: src/sksl/SkSLParser.cpp:431-467
    /// Safety: Compiler has installed the real global symbols/config/error reporter;
    /// its original compilation lease and immutable source remain live throughout.
    pub unsafe fn declarations(&mut self) {
        self.fEncounteredFatalError.write(false);
        if self.fText.as_ref().unwrap_unchecked().len() >= Position::kMaxOffset as usize {
            self.error_position(Position::default(), b"program is too large");
            return;
        }
        if self.peek().fKind == Kind::TK_DIRECTIVE {
            self.directive(true);
        }
        while !self.fEncounteredFatalError.assume_init() {
            crate::SkASSERT!(@source b"src/sksl/SkSLParser.cpp\0".as_ptr(),448; @condition b"fCompiler.context().fSymbolTable == fCompiler.globalSymbols()\0".as_ptr(); self.fCompiler.as_ref().context().fSymbolTable==self.fCompiler.as_ref().globalSymbols());
            match self.peek().fKind {
                Kind::TK_END_OF_FILE => return,
                Kind::TK_INVALID => {
                    let token = self.peek();
                    self.error(token, b"invalid token");
                    return;
                }
                Kind::TK_DIRECTIVE => self.directive(false),
                _ => {
                    self.declaration();
                }
            }
        }
    }
    // Source: src/sksl/SkSLParser.cpp:470-495
    pub unsafe fn extensionDirective(&mut self, start: Position) {
        let mut name = Token::default();
        if !self.expectIdentifier(&mut name) {
            return;
        }
        if !self.expect(Kind::TK_COLON, b"':'", None) {
            return;
        }
        let mut behavior = Token::default();
        if !self.expect(Kind::TK_IDENTIFIER, b"an identifier", Some(&mut behavior)) {
            return;
        }
        if self.expectNewline() {
            // Names remain views into the same retained Parser source. Nodes are
            // destroyed before source or transferred together to the genuine Program.
            let range = self.rangeFrom(start);
            let name = NonNull::from(self.text(name));
            let behavior = NonNull::from(self.text(behavior));
            if let Some(ext) = super::ir::SkSLExtension::Extension::Convert(
                self.fCompiler.as_ref().context(),
                range,
                name.as_ref(),
                behavior.as_ref(),
            ) {
                self.fProgramElements
                    .push(ext.cast(|p| p as *mut dyn ProgramElement));
            }
        } else {
            self.error_position(start, b"invalid #extension directive");
        }
    }
    // Source: src/sksl/SkSLParser.cpp:497-521
    // UNVERIFIED complete native directive leaf; no program element or module fabricated.
    pub unsafe fn versionDirective(&mut self, start: Position, allowVersion: bool) {
        if !allowVersion {
            self.error_position(
                start,
                b"#version directive must appear before anything else",
            );
            return;
        }
        let mut version = MaybeUninit::<SKSL_INT>::uninit();
        if !self.intLiteral(version.as_mut_ptr()) {
            return;
        }
        match version.assume_init() {
            100 => {
                let mut config = self.fCompiler.as_ref().context().fConfig.unwrap_unchecked();
                config.as_mut().fRequiredSkSLVersion =
                    crate::include::sksl::SkSLVersion::Version::k100;
            }
            300 => {
                let mut config = self.fCompiler.as_ref().context().fConfig.unwrap_unchecked();
                config.as_mut().fRequiredSkSLVersion =
                    crate::include::sksl::SkSLVersion::Version::k300;
            }
            _ => {
                self.error_position(start, b"unsupported version number");
                return;
            }
        }
        if !self.expectNewline() {
            self.error_position(start, b"invalid #version directive");
        }
    }
    // Source: src/sksl/SkSLParser.cpp:525-538
    pub unsafe fn directive(&mut self, allowVersion: bool) {
        let mut start = Token::default();
        if !self.expect(Kind::TK_DIRECTIVE, b"a directive", Some(&mut start)) {
            return;
        }
        let text = self.text(start);
        if text == b"#extension" {
            return self.extensionDirective(Self::position(start));
        }
        if text == b"#version" {
            return self.versionDirective(Self::position(start), allowVersion);
        }
        let mut message = b"unsupported directive '".to_vec();
        message.extend_from_slice(self.text(start));
        message.push(b'\'');
        self.error(start, &message);
    }
    // Source: src/sksl/SkSLParser.cpp:540-548
    pub unsafe fn modifiersDeclarationEnd(
        &mut self,
        mods: &super::ir::SkSLModifiers::Modifiers,
    ) -> bool {
        let Some(decl) = super::ir::SkSLModifiersDeclaration::ModifiersDeclaration::Convert(
            self.fCompiler.as_ref().context(),
            mods,
        ) else {
            return false;
        };
        self.fProgramElements
            .push(decl.cast(|p| p as *mut dyn ProgramElement));
        true
    }
    // Source: src/sksl/SkSLParser.cpp:552-588
    pub unsafe fn declaration(&mut self) -> bool {
        let start = self.peek();
        if start.fKind == Kind::TK_SEMICOLON {
            self.nextToken();
            self.error(start, b"expected a declaration, but found ';'");
            return false;
        }
        let mut modifiers = self.modifiers();
        let lookahead = self.peek();
        if lookahead.fKind == Kind::TK_IDENTIFIER
            && !self
                .symbolTable()
                .unwrap_unchecked()
                .as_ref()
                .isType(self.text(lookahead))
        {
            return self.interfaceBlock(&modifiers);
        }
        if lookahead.fKind == Kind::TK_SEMICOLON {
            self.nextToken();
            return self.modifiersDeclarationEnd(&modifiers);
        }
        if lookahead.fKind == Kind::TK_STRUCT {
            self.structVarDeclaration(Self::position(start), &modifiers);
            return true;
        }
        let Some(ty) = self.parseType(&mut modifiers) else {
            return false;
        };
        let mut name = Token::default();
        if !self.expectIdentifier(&mut name) {
            return false;
        }
        if self.checkNext(Kind::TK_LPAREN, None) {
            self.functionDeclarationEnd(Self::position(start), &mut modifiers, ty, name)
        } else {
            self.globalVarDeclarationEnd(Self::position(start), &modifiers, ty, name);
            true
        }
    }
    // Source: src/sksl/SkSLParser.cpp:591-636
    pub unsafe fn functionDeclarationEnd(
        &mut self,
        start: Position,
        modifiers: &mut super::ir::SkSLModifiers::Modifiers,
        returnType: NonNull<dyn super::ir::SkSLType::Type>,
        name: Token,
    ) -> bool {
        let lookahead = self.peek();
        let mut validParams = true;
        // Rust passes the actual movable owning TArray required by Convert;
        // native STArray inline capacity is an allocation-shape adaptation only.
        let mut parameters = crate::include::private::SkTArray::TArray::<
            Option<IRNodeBox<dyn super::ir::SkSLVariable::Variable>>,
            true,
        >::new();
        if lookahead.fKind == Kind::TK_RPAREN {
        } else if lookahead.fKind == Kind::TK_IDENTIFIER && self.text(lookahead) == b"void" {
            self.nextToken();
        } else {
            loop {
                let mut param = None;
                if !self.parameter(&mut param) {
                    return false;
                }
                validParams = validParams && param.is_some();
                parameters.push_back(param);
                if !self.checkNext(Kind::TK_COMMA, None) {
                    break;
                }
            }
        }
        if !self.expect(Kind::TK_RPAREN, b"')'", None) {
            return false;
        }
        let mut decl = None;
        if validParams {
            // Borrow the original immutable source through its native retained owner;
            // no parser-wide shared borrow spans token mutation or conversion callbacks.
            let range = self.rangeFrom(start);
            let name = NonNull::from(self.text(name));
            decl = super::ir::SkSLFunctionDeclaration::FunctionDeclaration::Convert(
                self.fCompiler.as_ref().context(),
                range,
                modifiers,
                name.as_ref(),
                parameters,
                start,
                returnType,
            );
        }
        if self.checkNext(Kind::TK_SEMICOLON, None) {
            self.prototypeFunction(decl)
        } else {
            self.defineFunction(decl)
        }
    }
    // Source: src/sksl/SkSLParser.cpp:638-644
    pub unsafe fn prototypeFunction(
        &mut self,
        decl: Option<NonNull<super::ir::SkSLFunctionDeclaration::FunctionDeclaration>>,
    ) -> bool {
        let Some(decl) = decl else {
            return false;
        };
        let element = IRNodeBox::new_with(
            |p: *mut super::ir::SkSLFunctionPrototype::FunctionPrototype| {
                p.write(super::ir::SkSLFunctionPrototype::FunctionPrototype::new(
                    decl.as_ref().node().fPosition,
                    decl,
                ))
            },
        );
        self.fProgramElements
            .push(element.cast(|p| p as *mut dyn ProgramElement));
        true
    }
    // Source: src/sksl/SkSLParser.cpp:646-685
    pub unsafe fn defineFunction(
        &mut self,
        decl: Option<NonNull<super::ir::SkSLFunctionDeclaration::FunctionDeclaration>>,
    ) -> bool {
        let context = self.fCompiler.as_ref().context() as *const Context;
        let bodyStart = self.peek();
        let mut table = None;
        let body;
        {
            let _symbols = AutoSymbolTable::new(NonNull::from(&mut *self), &mut table, true);
            if let Some(decl) = decl {
                for param in decl.as_ref().parameters() {
                    let param = param.unwrap_unchecked();
                    table
                        .as_mut()
                        .unwrap_unchecked()
                        .addWithoutOwnershipContext(
                            &*context,
                            NonNull::from(param.as_ref() as &dyn super::ir::SkSLSymbol::Symbol),
                        );
                }
            }
            body = self.block(false, Some(&mut table));
        }
        let (Some(mut decl), Some(mut body)) = (decl, body) else {
            return false;
        };
        debug_assert!(body.node().is::<Block>());
        let pos = self.rangeFromToken(bodyStart);
        body.as_pin_mut().node_mut().setPosition(pos);
        let Some(function) = super::ir::SkSLFunctionDefinition::FunctionDefinition::Convert(
            &*context,
            pos,
            decl.as_ref(),
            body,
        ) else {
            return false;
        };
        decl.as_mut().setDefinition(Some(NonNull::new_unchecked(
            function.as_ptr() as *mut super::ir::SkSLFunctionDefinition::FunctionDefinition
        )));
        self.fProgramElements
            .push(function.cast(|p| p as *mut dyn ProgramElement));
        true
    }
    // Source: src/sksl/SkSLParser.cpp:687-717
    pub unsafe fn arraySize(&mut self, outResult: &mut SKSL_INT) -> bool {
        *outResult = 1;
        let next = self.peek();
        if next.fKind == Kind::TK_RBRACKET {
            self.error_position(
                Self::position(next),
                b"unsized arrays are not permitted here",
            );
            return true;
        }
        let Some(sizeLiteral) = self.expression() else {
            return false;
        };
        if !sizeLiteral.node().is::<super::ir::SkSLPoison::Poison>() {
            let mut size = MaybeUninit::<SKSL_INT>::uninit();
            if !super::SkSLConstantFolder::ConstantFolder::GetConstantInt(
                &*sizeLiteral,
                size.as_mut_ptr(),
            ) {
                self.error_position(
                    sizeLiteral.node().fPosition,
                    b"array size must be an integer",
                );
                return true;
            }
            let size = size.assume_init();
            if size > i32::MAX as i64 {
                self.error_position(sizeLiteral.node().fPosition, b"array size out of bounds");
                return true;
            }
            if size <= 0 {
                self.error_position(sizeLiteral.node().fPosition, b"array size must be positive");
                return true;
            }
            *outResult = size;
        }
        true
    }
    // Source: src/sksl/SkSLParser.cpp:719-726
    pub unsafe fn arrayType(
        &mut self,
        base: NonNull<dyn super::ir::SkSLType::Type>,
        count: i32,
        pos: Position,
    ) -> NonNull<dyn super::ir::SkSLType::Type> {
        let context = self.fCompiler.as_ref().context();
        let count = super::ir::SkSLType::convertArraySizeValue(
            base.as_ref(),
            context,
            pos,
            pos,
            count as i64,
        );
        if count == 0 {
            return NonNull::from(&**context.fTypes.as_ref().fPoison);
        }
        self.symbolTable()
            .unwrap_unchecked()
            .as_mut()
            .addArrayDimension(context, base, count as i32)
    }
    // Source: src/sksl/SkSLParser.cpp:728-735
    pub unsafe fn unsizedArrayType(
        &mut self,
        base: NonNull<dyn super::ir::SkSLType::Type>,
        pos: Position,
    ) -> NonNull<dyn super::ir::SkSLType::Type> {
        let context = self.fCompiler.as_ref().context();
        if !super::ir::SkSLType::checkIfUsableInArray(base.as_ref(), context, pos) {
            return NonNull::from(&**context.fTypes.as_ref().fPoison);
        }
        self.symbolTable()
            .unwrap_unchecked()
            .as_mut()
            .addArrayDimension(context, base, super::ir::SkSLType::kUnsizedArray)
    }
    // Source: src/sksl/SkSLParser.cpp:737-758
    pub unsafe fn parseArrayDimensions(
        &mut self,
        pos: Position,
        ty: &mut NonNull<dyn super::ir::SkSLType::Type>,
    ) -> bool {
        let mut next = Token::default();
        while self.checkNext(Kind::TK_LBRACKET, Some(&mut next)) {
            if self.checkNext(Kind::TK_RBRACKET, None) {
                if self.allowUnsizedArrays() {
                    *ty = self.unsizedArrayType(*ty, self.rangeFrom(pos));
                } else {
                    self.error_position(
                        self.rangeFrom(pos),
                        b"unsized arrays are not permitted here",
                    );
                }
            } else {
                let mut size: SKSL_INT = 1;
                if !self.arraySize(&mut size) {
                    return false;
                }
                if !self.expect(Kind::TK_RBRACKET, b"']'", None) {
                    return false;
                }
                *ty = self.arrayType(*ty, size as i32, self.rangeFrom(pos));
            }
        }
        true
    }
    // Source: src/sksl/SkSLParser.cpp:760-766
    pub unsafe fn parseInitializer(
        &mut self,
        _pos: Position,
        initializer: &mut Option<IRNodeBox<dyn Expression>>,
    ) -> bool {
        if self.checkNext(Kind::TK_EQ, None) {
            *initializer = self.assignmentExpression();
            return initializer.is_some();
        }
        true
    }
    // Source: src/sksl/SkSLParser.cpp:768-772
    pub unsafe fn addGlobalVarDeclaration(
        &mut self,
        decl: Option<IRNodeBox<super::ir::SkSLVarDeclarations::VarDeclaration>>,
    ) {
        if let Some(decl) = decl {
            let statement = decl.cast(|p| p as *mut dyn Statement);
            let global = super::ir::SkSLVarDeclarations::GlobalVarDeclaration::new(statement)
                .cast(|p| p as *mut dyn ProgramElement);
            self.fProgramElements.push(global);
        }
    }
    // Source: src/sksl/SkSLParser.cpp:776-819
    pub unsafe fn globalVarDeclarationEnd(
        &mut self,
        pos: Position,
        mods: &super::ir::SkSLModifiers::Modifiers,
        baseType: NonNull<dyn super::ir::SkSLType::Type>,
        name: Token,
    ) {
        let mut ty = baseType;
        let mut initializer = None;
        if !self.parseArrayDimensions(pos, &mut ty) {
            return;
        }
        if !self.parseInitializer(pos, &mut initializer) {
            return;
        }
        let decl = super::ir::SkSLVarDeclarations::VarDeclaration::Convert(
            self.fCompiler.as_ref().context(),
            self.rangeFrom(pos),
            mods,
            ty,
            Self::position(name),
            self.text(name),
            super::ir::SkSLVariable::Storage::kGlobal,
            initializer,
        );
        self.addGlobalVarDeclaration(decl);
        while self.checkNext(Kind::TK_COMMA, None) {
            ty = baseType;
            let mut identifierName = Token::default();
            if !self.expectIdentifier(&mut identifierName) {
                return;
            }
            if !self.parseArrayDimensions(pos, &mut ty) {
                return;
            }
            let mut anotherInitializer = None;
            if !self.parseInitializer(pos, &mut anotherInitializer) {
                return;
            }
            let decl = super::ir::SkSLVarDeclarations::VarDeclaration::Convert(
                self.fCompiler.as_ref().context(),
                self.rangeFromToken(identifierName),
                mods,
                ty,
                Self::position(identifierName),
                self.text(identifierName),
                super::ir::SkSLVariable::Storage::kGlobal,
                anotherInitializer,
            );
            self.addGlobalVarDeclaration(decl);
        }
        self.expect(Kind::TK_SEMICOLON, b"';'", None);
    }
    // Source: src/sksl/SkSLParser.cpp:823-873
    pub unsafe fn localVarDeclarationEnd(
        &mut self,
        mut pos: Position,
        mods: &super::ir::SkSLModifiers::Modifiers,
        baseType: NonNull<dyn super::ir::SkSLType::Type>,
        name: Token,
    ) -> ParsedStatement {
        let mut ty = baseType;
        let mut initializer = None;
        if !self.parseArrayDimensions(pos, &mut ty) {
            return None;
        }
        if !self.parseInitializer(pos, &mut initializer) {
            return None;
        }
        let mut result = super::ir::SkSLVarDeclarations::VarDeclaration::Convert(
            self.fCompiler.as_ref().context(),
            self.rangeFrom(pos),
            mods,
            ty,
            Self::position(name),
            self.text(name),
            super::ir::SkSLVariable::Storage::kLocal,
            initializer,
        )
        .map(|v| v.cast(|p| p as *mut dyn Statement));
        loop {
            if !self.checkNext(Kind::TK_COMMA, None) {
                self.expect(Kind::TK_SEMICOLON, b"';'", None);
                break;
            }
            ty = baseType;
            let mut identifierName = Token::default();
            if !self.expectIdentifier(&mut identifierName) {
                break;
            }
            if !self.parseArrayDimensions(pos, &mut ty) {
                break;
            }
            let mut anotherInitializer = None;
            if !self.parseInitializer(pos, &mut anotherInitializer) {
                break;
            }
            let next = super::ir::SkSLVarDeclarations::VarDeclaration::Convert(
                self.fCompiler.as_ref().context(),
                self.rangeFromToken(identifierName),
                mods,
                ty,
                Self::position(identifierName),
                self.text(identifierName),
                super::ir::SkSLVariable::Storage::kLocal,
                anotherInitializer,
            )
            .map(|v| v.cast(|p| p as *mut dyn Statement));
            result = Block::MakeCompoundStatement(result, next);
        }
        pos = self.rangeFrom(pos);
        Some(self.statementOrNop(pos, result))
    }
    // Source: src/sksl/SkSLParser.cpp:876-904
    pub unsafe fn varDeclarationsOrExpressionStatement(&mut self) -> ParsedStatement {
        let nextToken = self.peek();
        if nextToken.fKind == Kind::TK_CONST {
            return self.varDeclarations();
        }
        if nextToken.fKind == Kind::TK_HIGHP
            || nextToken.fKind == Kind::TK_MEDIUMP
            || nextToken.fKind == Kind::TK_LOWP
            || self
                .symbolTable()
                .unwrap_unchecked()
                .as_ref()
                .isType(self.text(nextToken))
        {
            let mut checkpoint = ParserCheckpoint::new(NonNull::from(&mut *self));
            let mut prefix = VarDeclarationsPrefix::uninit();
            if self.varDeclarationsPrefix(&mut prefix) {
                checkpoint.as_mut().accept();
                return self.localVarDeclarationEnd(
                    prefix.fPosition.assume_init(),
                    prefix.fModifiers.assume_init_ref(),
                    prefix.fType.assume_init().unwrap_unchecked(),
                    prefix.fName.assume_init(),
                );
            }
            checkpoint.as_mut().rewind();
        }
        self.expressionStatement()
    }
    // Source: src/sksl/SkSLParser.cpp:908-916
    unsafe fn varDeclarationsPrefix(&mut self, prefixData: &mut VarDeclarationsPrefix) -> bool {
        let token = self.peek();
        prefixData.fPosition.write(Self::position(token));
        prefixData.fModifiers.write(self.modifiers());
        let ty = self.parseType(prefixData.fModifiers.assume_init_mut());
        prefixData.fType.write(ty);
        if ty.is_none() {
            return false;
        }
        let mut name = Token::default();
        let success = self.expectIdentifier(&mut name);
        prefixData.fName.write(name);
        success
    }
    // Source: src/sksl/SkSLParser.cpp:919-926
    pub unsafe fn varDeclarations(&mut self) -> ParsedStatement {
        let mut prefix = VarDeclarationsPrefix::uninit();
        if !self.varDeclarationsPrefix(&mut prefix) {
            return None;
        }
        self.localVarDeclarationEnd(
            prefix.fPosition.assume_init(),
            prefix.fModifiers.assume_init_ref(),
            prefix.fType.assume_init().unwrap_unchecked(),
            prefix.fName.assume_init(),
        )
    }
    // Source: src/sksl/SkSLParser.cpp:929-995
    // The returned Type remains owned by the native symbol table. Field names
    // borrow Parser's retained source text, and the actual StructDefinition moves
    // into fProgramElements without copying or replacing that Type owner.
    pub unsafe fn structDeclaration(&mut self) -> Option<NonNull<dyn super::ir::SkSLType::Type>> {
        use super::ir::{
            SkSLStructDefinition::StructDefinition,
            SkSLType::{Field, Type},
        };
        let mut depth = AutoDepth::new(NonNull::from(&mut *self));
        let first = self.peek();
        let start = Self::position(first);
        if !self.expect(Kind::TK_STRUCT, b"'struct'", None) {
            return None;
        }
        let mut name = Token::default();
        if !self.expectIdentifier(&mut name) {
            return None;
        }
        if !self.expect(Kind::TK_LBRACE, b"'{'", None) {
            return None;
        }
        if !depth.increase() {
            return None;
        }
        let mut fields = crate::include::private::SkTArray::TArray::<Field, true>::new();
        while !self.checkNext(Kind::TK_RBRACE, None) {
            let fieldStart = self.peek();
            let mut modifiers = self.modifiers();
            let ty = self.parseType(&mut modifiers)?;
            loop {
                let mut actualType = ty;
                let mut memberName = Token::default();
                if !self.expectIdentifier(&mut memberName) {
                    return None;
                }
                while self.checkNext(Kind::TK_LBRACKET, None) {
                    let mut size: SKSL_INT = 1;
                    if !self.arraySize(&mut size) {
                        return None;
                    }
                    if !self.expect(Kind::TK_RBRACKET, b"']'", None) {
                        return None;
                    }
                    actualType = self.arrayType(
                        actualType,
                        size as i32,
                        self.rangeFrom(Self::position(fieldStart)),
                    );
                }
                fields.push_back(Field::new(
                    self.rangeFromToken(fieldStart),
                    modifiers.fLayout,
                    modifiers.fFlags,
                    self.text(memberName),
                    Some(actualType),
                ));
                if !self.checkNext(Kind::TK_COMMA, None) {
                    break;
                }
            }
            if !self.expect(Kind::TK_SEMICOLON, b"';'", None) {
                return None;
            }
        }
        let def = StructDefinition::Convert(
            self.fCompiler.as_ref().context(),
            self.rangeFrom(start),
            self.text(name),
            fields,
        );
        // Convert delegates to native make_unique construction; IRNodeBox is
        // non-null, so its native defensive null test cannot fail on this path.
        // Erase the temporary accessor borrow, retaining the original symbol-table
        // lease required by Parser's unsafe contract rather than borrowing def.
        let result = core::mem::transmute::<NonNull<dyn Type + '_>, NonNull<dyn Type>>(
            NonNull::from(def.r#type()),
        );
        self.fProgramElements
            .push(def.cast(|p| p as *mut dyn ProgramElement));
        Some(result)
    }
    // Source: src/sksl/SkSLParser.cpp:998-1009
    pub unsafe fn structVarDeclaration(
        &mut self,
        _start: Position,
        modifiers: &super::ir::SkSLModifiers::Modifiers,
    ) {
        let Some(ty) = self.structDeclaration() else {
            return;
        };
        let mut name = Token::default();
        if self.checkIdentifier(&mut name) {
            self.globalVarDeclarationEnd(self.rangeFromToken(name), modifiers, ty, name);
        } else {
            self.expect(Kind::TK_SEMICOLON, b"';'", None);
        }
    }
    // Source: src/sksl/SkSLParser.cpp:1012-1041
    // Actual nullable unique_ptr output is assigned only after successful syntax parsing.
    // The returned Variable borrows name bytes from the original retained Parser text;
    // the containing Program must retain that backing just as in the native owner chain.
    pub unsafe fn parameter(
        &mut self,
        outParam: &mut Option<IRNodeBox<dyn super::ir::SkSLVariable::Variable>>,
    ) -> bool {
        let first = self.peek();
        let pos = Self::position(first);
        let mut modifiers = self.modifiers();
        let Some(mut ty) = self.parseType(&mut modifiers) else {
            return false;
        };
        let mut name = Token::default();
        let mut nameText = NonNull::from(b"".as_slice());
        let namePos;
        if self.checkIdentifier(&mut name) {
            nameText = NonNull::from(self.text(name));
            namePos = Self::position(name);
        } else {
            namePos = self.rangeFrom(pos);
        }
        if !self.parseArrayDimensions(pos, &mut ty) {
            return false;
        }
        let result = <dyn super::ir::SkSLVariable::Variable>::Convert(
            self.fCompiler.as_ref().context(),
            self.rangeFrom(pos),
            modifiers.fPosition,
            &modifiers.fLayout,
            modifiers.fFlags,
            ty,
            namePos,
            nameText.as_ref(),
            super::ir::SkSLVariable::Storage::kParameter,
        );
        // Native unique_ptr assignment publishes replacement before old owner destruction.
        drop(core::mem::replace(outParam, Some(result)));
        true
    }
    // Source: src/sksl/SkSLParser.cpp:1044-1059
    pub unsafe fn layoutInt(&mut self) -> i32 {
        if !self.expect(Kind::TK_EQ, b"'='", None) {
            return -1;
        }
        let mut token = Token::default();
        if !self.expect(
            Kind::TK_INT_LITERAL,
            b"a non-negative integer",
            Some(&mut token),
        ) {
            return -1;
        }
        let frag = self.text(token);
        let mut value = MaybeUninit::<SKSL_INT>::uninit();
        if !super::SkSLString::stoi_raw(frag, value.as_mut_ptr()) {
            let mut msg = b"value in layout is too large: ".to_vec();
            msg.extend_from_slice(frag);
            self.error(token, &msg);
            return -1;
        }
        value.assume_init() as i32
    }
    // Source: src/sksl/SkSLParser.cpp:1062-1071
    pub unsafe fn layoutIdentifier(&mut self) -> &[u8] {
        if !self.expect(Kind::TK_EQ, b"'='", None) {
            return b"";
        }
        let mut token = Token::default();
        if !self.expectIdentifier(&mut token) {
            return b"";
        }
        self.text(token)
    }
    // Source: src/sksl/SkSLParser.cpp:1074-1171
    pub unsafe fn layout(&mut self) -> super::ir::SkSLLayout::Layout {
        use super::ir::SkSLLayout::{Layout, LayoutFlag as F};
        type LayoutMap = crate::src::core::SkTHash::THashMap<
            &'static [u8],
            F,
            super::SkSLIntrinsicList::IntrinsicHash,
        >;
        static TOKENS: std::sync::OnceLock<LayoutMap> = std::sync::OnceLock::new();
        let tokens = TOKENS.get_or_init(|| {
            LayoutMap::from_pairs(&[
                (b"location".as_slice(), F::kLocation),
                (b"offset".as_slice(), F::kOffset),
                (b"binding".as_slice(), F::kBinding),
                (b"texture".as_slice(), F::kTexture),
                (b"sampler".as_slice(), F::kSampler),
                (b"index".as_slice(), F::kIndex),
                (b"set".as_slice(), F::kSet),
                (b"builtin".as_slice(), F::kBuiltin),
                (
                    b"input_attachment_index".as_slice(),
                    F::kInputAttachmentIndex,
                ),
                (b"origin_upper_left".as_slice(), F::kOriginUpperLeft),
                (
                    b"blend_support_all_equations".as_slice(),
                    F::kBlendSupportAllEquations,
                ),
                (b"push_constant".as_slice(), F::kPushConstant),
                (b"color".as_slice(), F::kColor),
                (b"vulkan".as_slice(), F::kVulkan),
                (b"metal".as_slice(), F::kMetal),
                (b"webgpu".as_slice(), F::kWebGPU),
                (b"direct3d".as_slice(), F::kDirect3D),
                (b"rgba8".as_slice(), F::kRGBA8),
                (b"rgba32f".as_slice(), F::kRGBA32F),
                (b"r32f".as_slice(), F::kR32F),
                (b"local_size_x".as_slice(), F::kLocalSizeX),
                (b"local_size_y".as_slice(), F::kLocalSizeY),
                (b"local_size_z".as_slice(), F::kLocalSizeZ),
            ])
        });
        let mut result = Layout::default();
        if self.checkNext(Kind::TK_LAYOUT, None) && self.expect(Kind::TK_LPAREN, b"'('", None) {
            loop {
                let token = self.nextToken();
                let text = self.text(token);
                // Temporary string_view lookup lease: table only compares/hashes bytes and never inserts this key.
                let key: &'static [u8] = core::mem::transmute(text);
                let found = tokens.find(&key).copied();
                if let Some(found) = found {
                    if result.fFlags.0 & found.0 != 0 {
                        let mut msg = b"layout qualifier '".to_vec();
                        msg.extend_from_slice(text);
                        msg.extend_from_slice(b"' appears more than once");
                        self.error(token, &msg);
                    }
                    result.fFlags.0 |= found.0;
                    match found {
                        F::kLocation => result.fLocation = self.layoutInt(),
                        F::kOffset => result.fOffset = self.layoutInt(),
                        F::kBinding => result.fBinding = self.layoutInt(),
                        F::kIndex => result.fIndex = self.layoutInt(),
                        F::kSet => result.fSet = self.layoutInt(),
                        F::kTexture => result.fTexture = self.layoutInt(),
                        F::kSampler => result.fSampler = self.layoutInt(),
                        F::kBuiltin => result.fBuiltin = self.layoutInt(),
                        F::kInputAttachmentIndex => result.fInputAttachmentIndex = self.layoutInt(),
                        F::kLocalSizeX => result.fLocalSizeX = self.layoutInt(),
                        F::kLocalSizeY => result.fLocalSizeY = self.layoutInt(),
                        F::kLocalSizeZ => result.fLocalSizeZ = self.layoutInt(),
                        _ => {}
                    }
                } else {
                    let mut msg = b"'".to_vec();
                    msg.extend_from_slice(text);
                    msg.extend_from_slice(b"' is not a valid layout qualifier");
                    self.error(token, &msg);
                }
                if self.checkNext(Kind::TK_RPAREN, None) {
                    break;
                }
                if !self.expect(Kind::TK_COMMA, b"','", None) {
                    break;
                }
            }
        }
        result
    }
    // Source: src/sksl/SkSLParser.cpp:1175-1197
    pub unsafe fn modifiers(&mut self) -> super::ir::SkSLModifiers::Modifiers {
        use super::ir::SkSLModifierFlags::ModifierFlags;
        let start = self.peek().fOffset;
        let layout = self.layout();
        let raw = self.nextRawToken();
        let mut end = raw.fOffset;
        if !is_whitespace(raw.fKind) {
            self.pushback(raw);
        }
        let mut flags = ModifierFlags(0);
        loop {
            let tokenFlag = parse_modifier_token(self.peek().fKind);
            if tokenFlag.0 == 0 {
                break;
            }
            let modifier = self.nextToken();
            let duplicateFlags = ModifierFlags(tokenFlag.0 & flags.0);
            if duplicateFlags.0 != 0 {
                let mut msg = b"'".to_vec();
                msg.extend_from_slice(&duplicateFlags.description());
                msg.extend_from_slice(b"' appears more than once");
                self.error(modifier, &msg);
            }
            flags.0 |= tokenFlag.0;
            end = Self::position(modifier).endOffset();
        }
        super::ir::SkSLModifiers::Modifiers {
            fPosition: Position::Range(start, end),
            fLayout: layout,
            fFlags: flags,
        }
    }
    // Source: src/sksl/SkSLParser.cpp:1199-1207
    pub unsafe fn statementOrNop(
        &self,
        pos: Position,
        stmt: ParsedStatement,
    ) -> IRNodeBox<dyn Statement> {
        let mut stmt = stmt.unwrap_or_else(|| Nop::Make());
        if pos.valid() && !stmt.node().fPosition.valid() {
            stmt.as_pin_mut().node_mut().setPosition(pos);
        }
        stmt
    }
    // Source: src/sksl/SkSLParser.cpp:1210-1249
    pub unsafe fn statement(&mut self, bracesIntroduceNewScope: bool) -> ParsedStatement {
        let mut depth = AutoDepth::new(NonNull::from(&mut *self));
        if !depth.increase() {
            return None;
        }
        match self.peek().fKind {
            Kind::TK_IF => self.ifStatement(),
            Kind::TK_FOR => self.forStatement(),
            Kind::TK_DO => self.doStatement(),
            Kind::TK_WHILE => self.whileStatement(),
            Kind::TK_SWITCH => self.switchStatement(),
            Kind::TK_RETURN => self.returnStatement(),
            Kind::TK_BREAK => self.breakStatement(),
            Kind::TK_CONTINUE => self.continueStatement(),
            Kind::TK_DISCARD => self.discardStatement(),
            Kind::TK_LBRACE => self.block(bracesIntroduceNewScope, None),
            Kind::TK_SEMICOLON => {
                self.nextToken();
                Some(Nop::Make())
            }
            Kind::TK_CONST => self.varDeclarations(),
            Kind::TK_HIGHP | Kind::TK_MEDIUMP | Kind::TK_LOWP | Kind::TK_IDENTIFIER => {
                self.varDeclarationsOrExpressionStatement()
            }
            _ => self.expressionStatement(),
        }
    }
    // Source: src/sksl/SkSLParser.cpp:1251-1276
    pub unsafe fn findType(
        &mut self,
        pos: Position,
        modifiers: &mut super::ir::SkSLModifiers::Modifiers,
        name: &[u8],
    ) -> NonNull<dyn super::ir::SkSLType::Type> {
        use super::ir::SkSLSymbol::Kind as S;
        let context = self.fCompiler.as_ref().context();
        let poison = NonNull::from(&**context.fTypes.as_ref().fPoison);
        let Some(symbol) = self.symbolTable().unwrap_unchecked().as_ref().find(name) else {
            let mut msg = b"no symbol named '".to_vec();
            msg.extend_from_slice(name);
            msg.push(b'\'');
            self.error_position(pos, &msg);
            return poison;
        };
        if symbol.as_ref().kind() != S::kType {
            let mut msg = b"symbol '".to_vec();
            msg.extend_from_slice(name);
            msg.extend_from_slice(b"' is not a type");
            self.error_position(pos, &msg);
            return poison;
        }
        let ty = symbol.as_ref().as_type_symbol();
        let ty: NonNull<dyn super::ir::SkSLType::Type> = core::mem::transmute(NonNull::from(ty));
        if !context.fConfig.unwrap_unchecked().as_ref().isBuiltinCode() {
            if !super::ir::SkSLTypeReference::TypeReference::VerifyType(context, Some(ty), pos) {
                return poison;
            }
        }
        let mut qualifierRange = modifiers.fPosition;
        if qualifierRange.startOffset() == qualifierRange.endOffset() {
            qualifierRange = self.rangeFrom(qualifierRange);
        }
        core::mem::transmute(NonNull::from(super::ir::SkSLType::applyQualifiers(
            ty.as_ref(),
            context,
            &mut modifiers.fFlags,
            qualifierRange,
        )))
    }
    // Source: src/sksl/SkSLParser.cpp:1279-1313
    pub unsafe fn parseType(
        &mut self,
        modifiers: &mut super::ir::SkSLModifiers::Modifiers,
    ) -> Option<NonNull<dyn super::ir::SkSLType::Type>> {
        let mut token = Token::default();
        if !self.expect(Kind::TK_IDENTIFIER, b"a type", Some(&mut token)) {
            return None;
        }
        if !self
            .symbolTable()
            .unwrap_unchecked()
            .as_ref()
            .isType(self.text(token))
        {
            let mut msg = b"no type named '".to_vec();
            msg.extend_from_slice(self.text(token));
            msg.push(b'\'');
            self.error(token, &msg);
            return Some(NonNull::from(
                &**self.fCompiler.as_ref().context().fTypes.as_ref().fInvalid,
            ));
        }
        let name = NonNull::from(self.text(token));
        let mut result = self.findType(Self::position(token), modifiers, name.as_ref());
        if result.as_ref().isInterfaceBlock() {
            let mut msg = b"expected a type, found '".to_vec();
            msg.extend_from_slice(self.text(token));
            msg.push(b'\'');
            self.error(token, &msg);
            return Some(NonNull::from(
                &**self.fCompiler.as_ref().context().fTypes.as_ref().fInvalid,
            ));
        }
        let mut bracket = Token::default();
        while self.checkNext(Kind::TK_LBRACKET, Some(&mut bracket)) {
            if self.checkNext(Kind::TK_RBRACKET, None) {
                if self.allowUnsizedArrays() {
                    result = self.unsizedArrayType(result, self.rangeFromToken(token));
                } else {
                    self.error_position(
                        self.rangeFromToken(bracket),
                        b"unsized arrays are not permitted here",
                    );
                }
            } else {
                let mut size: SKSL_INT = 1;
                if !self.arraySize(&mut size) {
                    return None;
                }
                self.expect(Kind::TK_RBRACKET, b"']'", None);
                result = self.arrayType(result, size as i32, self.rangeFromToken(token));
            }
        }
        Some(result)
    }
    // Source: src/sksl/SkSLParser.cpp:1318-1397
    // Real admitted InterfaceBlock and Field owners; no invented ProgramElement metadata.
    pub unsafe fn interfaceBlock(&mut self, mods: &super::ir::SkSLModifiers::Modifiers) -> bool {
        let mut typeName = Token::default();
        if !self.expectIdentifier(&mut typeName) {
            return false;
        }
        if self.peek().fKind != Kind::TK_LBRACE {
            let mut msg = b"no type named '".to_vec();
            msg.extend_from_slice(self.text(typeName));
            msg.push(b'\'');
            self.error(typeName, &msg);
            return false;
        }
        self.nextToken();
        let mut fields =
            crate::include::private::SkTArray::TArray::<super::ir::SkSLType::Field, true>::new();
        while !self.checkNext(Kind::TK_RBRACE, None) {
            let first = self.peek();
            let fieldPos = Self::position(first);
            let mut fieldModifiers = self.modifiers();
            let Some(ty) = self.parseType(&mut fieldModifiers) else {
                return false;
            };
            loop {
                let mut fieldName = Token::default();
                if !self.expectIdentifier(&mut fieldName) {
                    return false;
                }
                let mut actualType = ty;
                if self.checkNext(Kind::TK_LBRACKET, None) {
                    let sizeToken = self.peek();
                    if sizeToken.fKind != Kind::TK_RBRACKET {
                        let mut size: SKSL_INT = 1;
                        if !self.arraySize(&mut size) {
                            return false;
                        }
                        actualType =
                            self.arrayType(actualType, size as i32, Self::position(typeName));
                    } else if self.allowUnsizedArrays() {
                        actualType = self.unsizedArrayType(actualType, Self::position(typeName));
                    } else {
                        self.error(sizeToken, b"unsized arrays are not permitted here");
                    }
                    self.expect(Kind::TK_RBRACKET, b"']'", None);
                }
                fields.push_back(super::ir::SkSLType::Field::new(
                    self.rangeFrom(fieldPos),
                    fieldModifiers.fLayout,
                    fieldModifiers.fFlags,
                    self.text(fieldName),
                    Some(actualType),
                ));
                if !self.checkNext(Kind::TK_COMMA, None) {
                    break;
                }
            }
            if !self.expect(Kind::TK_SEMICOLON, b"';'", None) {
                return false;
            }
        }
        let mut instanceName = NonNull::from(b"".as_slice());
        let mut instanceToken = Token::default();
        let mut size: SKSL_INT = 0;
        if self.checkIdentifier(&mut instanceToken) {
            instanceName = NonNull::from(self.text(instanceToken));
            if self.checkNext(Kind::TK_LBRACKET, None) {
                if !self.arraySize(&mut size) {
                    return false;
                }
                self.expect(Kind::TK_RBRACKET, b"']'", None);
            }
        }
        self.expect(Kind::TK_SEMICOLON, b"';'", None);
        if let Some(block) = super::ir::SkSLInterfaceBlock::InterfaceBlock::Convert(
            self.fCompiler.as_ref().context(),
            Self::position(typeName),
            mods,
            self.text(typeName),
            fields,
            instanceName.as_ref(),
            size as i32,
        ) {
            self.fProgramElements
                .push(block.cast(|p| p as *mut dyn ProgramElement));
            return true;
        }
        false
    }
    // Source: src/sksl/SkSLParser.cpp:1400-1432
    pub unsafe fn ifStatement(&mut self) -> ParsedStatement {
        let mut start = Token::default();
        if !self.expect(Kind::TK_IF, b"'if'", Some(&mut start)) {
            return None;
        }
        if !self.expect(Kind::TK_LPAREN, b"'('", None) {
            return None;
        }
        let test = self.expression()?;
        if !self.expect(Kind::TK_RPAREN, b"')'", None) {
            return None;
        }
        let ifTrue = self.statement(true)?;
        let mut ifFalse = None;
        if self.checkNext(Kind::TK_ELSE, None) {
            ifFalse = Some(self.statement(true)?);
        }
        let pos = self.rangeFromToken(start);
        let converted = IfStatement::Convert(
            self.fCompiler.as_ref().context(),
            pos,
            Some(test),
            Some(ifTrue),
            ifFalse,
        );
        Some(self.statementOrNop(pos, converted))
    }
    // Source: src/sksl/SkSLParser.cpp:1435-1463
    pub unsafe fn doStatement(&mut self) -> ParsedStatement {
        let mut start = Token::default();
        if !self.expect(Kind::TK_DO, b"'do'", Some(&mut start)) {
            return None;
        }
        let statement = self.statement(true)?;
        if !self.expect(Kind::TK_WHILE, b"'while'", None) {
            return None;
        }
        if !self.expect(Kind::TK_LPAREN, b"'('", None) {
            return None;
        }
        let test = self.expression()?;
        if !self.expect(Kind::TK_RPAREN, b"')'", None) {
            return None;
        }
        if !self.expect(Kind::TK_SEMICOLON, b"';'", None) {
            return None;
        }
        let pos = self.rangeFromToken(start);
        let converted = DoStatement::Convert(
            self.fCompiler.as_ref().context(),
            pos,
            statement,
            Some(test),
        );
        Some(self.statementOrNop(pos, converted))
    }
    // Source: src/sksl/SkSLParser.cpp:1466-1489
    pub unsafe fn whileStatement(&mut self) -> ParsedStatement {
        let mut start = Token::default();
        if !self.expect(Kind::TK_WHILE, b"'while'", Some(&mut start)) {
            return None;
        }
        if !self.expect(Kind::TK_LPAREN, b"'('", None) {
            return None;
        }
        let test = self.expression()?;
        if !self.expect(Kind::TK_RPAREN, b"')'", None) {
            return None;
        }
        let statement = self.statement(true)?;
        let pos = self.rangeFromToken(start);
        let converted = ForStatement::ConvertWhile(
            self.fCompiler.as_ref().context(),
            pos,
            Some(test),
            statement,
        );
        Some(self.statementOrNop(pos, converted))
    }
    // Source: src/sksl/SkSLParser.cpp:1492-1512
    pub unsafe fn switchCaseBody(
        &mut self,
        mut values: Pin<&mut ExpressionArray>,
        mut caseBlocks: Pin<&mut StatementArray>,
        caseValue: Option<IRNodeBox<dyn Expression>>,
    ) -> bool {
        if !self.expect(Kind::TK_COLON, b"':'", None) {
            return false;
        }
        let mut statements = StatementArray::new_pinned();
        while self.peek().fKind != Kind::TK_RBRACE
            && self.peek().fKind != Kind::TK_CASE
            && self.peek().fKind != Kind::TK_DEFAULT
        {
            let Some(stmt) = self.statement(true) else {
                return false;
            };
            statements.as_mut().push_back(Some(stmt));
        }
        values.as_mut().as_base_mut().push_back(caseValue);
        caseBlocks.as_mut().push_back(Some(Block::Make(
            Position::new(),
            statements,
            BlockKind::kUnbracedBlock,
            None,
        )));
        true
    }
    // Source: src/sksl/SkSLParser.cpp:1515-1525
    pub unsafe fn switchCase(
        &mut self,
        values: Pin<&mut ExpressionArray>,
        caseBlocks: Pin<&mut StatementArray>,
    ) -> bool {
        let mut start = Token::default();
        if !self.expect(Kind::TK_CASE, b"'case'", Some(&mut start)) {
            return false;
        }
        let Some(caseValue) = self.expression() else {
            return false;
        };
        self.switchCaseBody(values, caseBlocks, Some(caseValue))
    }
    // Source: src/sksl/SkSLParser.cpp:1528-1580
    pub unsafe fn switchStatement(&mut self) -> ParsedStatement {
        let mut start = Token::default();
        if !self.expect(Kind::TK_SWITCH, b"'switch'", Some(&mut start)) {
            return None;
        }
        if !self.expect(Kind::TK_LPAREN, b"'('", None) {
            return None;
        }
        let value = self.expression()?;
        if !self.expect(Kind::TK_RPAREN, b"')'", None) {
            return None;
        }
        if !self.expect(Kind::TK_LBRACE, b"'{'", None) {
            return None;
        }
        let mut symbolTable = None;
        let mut values = ExpressionArray::new();
        let mut caseBlocks = StatementArray::new_pinned();
        {
            let _symbols = AutoSymbolTable::new(NonNull::from(&mut *self), &mut symbolTable, true);
            while self.peek().fKind == Kind::TK_CASE {
                if !self.switchCase(values.as_mut(), caseBlocks.as_mut()) {
                    return None;
                }
            }
            if self.checkNext(Kind::TK_DEFAULT, None) {
                if !self.switchCaseBody(values.as_mut(), caseBlocks.as_mut(), None) {
                    return None;
                }
            }
            if !self.expect(Kind::TK_RBRACE, b"'}'", None) {
                return None;
            }
        }
        let pos = self.rangeFromToken(start);
        let converted = SwitchStatement::Convert(
            self.fCompiler.as_ref().context(),
            pos,
            Some(value),
            values,
            caseBlocks,
            symbolTable.unwrap_unchecked(),
        );
        Some(self.statementOrNop(pos, converted))
    }
}
// New translation is unverified; original parser-offset valid domain applies.
// Source: src/sksl/SkSLParser.cpp:1582-1584
fn range_of_at_least_one_char(start: i32, end: i32) -> super::SkSLPosition::Position {
    super::SkSLPosition::Position::Range(start, core::cmp::max(end, start + 1))
}
impl Parser {
    // Source: src/sksl/SkSLParser.cpp:1588-1655
    pub unsafe fn forStatement(&mut self) -> ParsedStatement {
        let mut start = Token::default();
        if !self.expect(Kind::TK_FOR, b"'for'", Some(&mut start)) {
            return None;
        }
        let mut lparen = Token::default();
        if !self.expect(Kind::TK_LPAREN, b"'('", Some(&mut lparen)) {
            return None;
        }
        let mut symbolTable = None;
        let mut initializer = None;
        let mut test = None;
        let mut next = None;
        let statement;
        let firstSemicolonOffset;
        let mut secondSemicolon = Token::default();
        let mut rparen = Token::default();
        {
            let _symbols = AutoSymbolTable::new(NonNull::from(&mut *self), &mut symbolTable, true);
            let nextToken = self.peek();
            if nextToken.fKind == Kind::TK_SEMICOLON {
                firstSemicolonOffset = self.nextToken().fOffset;
            } else {
                initializer = Some(self.varDeclarationsOrExpressionStatement()?);
                firstSemicolonOffset = self.fLexer.getCheckpoint().fOffset - 1;
            }
            if self.peek().fKind != Kind::TK_SEMICOLON {
                test = Some(self.expression()?);
            }
            if !self.expect(Kind::TK_SEMICOLON, b"';'", Some(&mut secondSemicolon)) {
                return None;
            }
            if self.peek().fKind != Kind::TK_RPAREN {
                next = Some(self.expression()?);
            }
            if !self.expect(Kind::TK_RPAREN, b"')'", Some(&mut rparen)) {
                return None;
            }
            statement = self.statement(false)?;
        }
        let pos = self.rangeFromToken(start);
        let loopPositions = ForLoopPositions {
            initPosition: range_of_at_least_one_char(lparen.fOffset + 1, firstSemicolonOffset),
            conditionPosition: range_of_at_least_one_char(
                firstSemicolonOffset + 1,
                secondSemicolon.fOffset,
            ),
            nextPosition: range_of_at_least_one_char(secondSemicolon.fOffset + 1, rparen.fOffset),
        };
        let converted = ForStatement::Convert(
            self.fCompiler.as_ref().context(),
            pos,
            loopPositions,
            initializer,
            test,
            next,
            statement,
            symbolTable,
        );
        Some(self.statementOrNop(pos, converted))
    }
    // Source: src/sksl/SkSLParser.cpp:1658-1676
    pub unsafe fn returnStatement(&mut self) -> ParsedStatement {
        let mut start = Token::default();
        if !self.expect(Kind::TK_RETURN, b"'return'", Some(&mut start)) {
            return None;
        }
        let mut expression = None;
        if self.peek().fKind != Kind::TK_SEMICOLON {
            expression = Some(self.expression()?);
        }
        if !self.expect(Kind::TK_SEMICOLON, b"';'", None) {
            return None;
        }
        Some(ReturnStatement::Make(
            self.rangeFromToken(start),
            expression,
        ))
    }
    // Source: src/sksl/SkSLParser.cpp:1679-1688
    pub unsafe fn breakStatement(&mut self) -> ParsedStatement {
        let mut start = Token::default();
        if !self.expect(Kind::TK_BREAK, b"'break'", Some(&mut start)) {
            return None;
        }
        if !self.expect(Kind::TK_SEMICOLON, b"';'", None) {
            return None;
        }
        Some(BreakStatement::Make(Self::position(start)))
    }
    // Source: src/sksl/SkSLParser.cpp:1691-1700
    pub unsafe fn continueStatement(&mut self) -> ParsedStatement {
        let mut start = Token::default();
        if !self.expect(Kind::TK_CONTINUE, b"'continue'", Some(&mut start)) {
            return None;
        }
        if !self.expect(Kind::TK_SEMICOLON, b"';'", None) {
            return None;
        }
        Some(ContinueStatement::Make(Self::position(start)))
    }
    // Source: src/sksl/SkSLParser.cpp:1703-1713
    pub unsafe fn discardStatement(&mut self) -> ParsedStatement {
        let mut start = Token::default();
        if !self.expect(Kind::TK_DISCARD, b"'discard'", Some(&mut start)) {
            return None;
        }
        if !self.expect(Kind::TK_SEMICOLON, b"';'", None) {
            return None;
        }
        let pos = Self::position(start);
        let converted = DiscardStatement::Convert(self.fCompiler.as_ref().context(), pos);
        Some(self.statementOrNop(pos, converted))
    }
    // Source: src/sksl/SkSLParser.cpp:1716-1761
    pub unsafe fn block(
        &mut self,
        introduceNewScope: bool,
        adoptExistingSymbolTable: Option<&mut Option<Box<SymbolTable>>>,
    ) -> ParsedStatement {
        debug_assert!(!(introduceNewScope && adoptExistingSymbolTable.is_some()));
        let mut depth = AutoDepth::new(NonNull::from(&mut *self));
        let mut start = Token::default();
        if !self.expect(Kind::TK_LBRACE, b"'{'", Some(&mut start)) {
            return None;
        }
        if !depth.increase() {
            return None;
        }
        let mut newSymbolTable = None;
        let symbolTableToUse = adoptExistingSymbolTable.unwrap_or(&mut newSymbolTable);
        let mut statements = StatementArray::new_pinned();
        {
            let _symbols = AutoSymbolTable::new(
                NonNull::from(&mut *self),
                symbolTableToUse,
                introduceNewScope,
            );
            loop {
                let tokenKind = self.peek().fKind;
                if tokenKind == Kind::TK_RBRACE {
                    self.nextToken();
                    break;
                }
                if tokenKind == Kind::TK_END_OF_FILE {
                    let token = self.peek();
                    self.error(token, b"expected '}', but found end of file");
                    return None;
                }
                if let Some(statement) = self.statement(true) {
                    statements.as_mut().push_back(Some(statement));
                }
                if self.fEncounteredFatalError.assume_init() {
                    return None;
                }
            }
        }
        Some(
            Block::MakeBlock(
                self.rangeFromToken(start),
                statements,
                BlockKind::kBracedScope,
                symbolTableToUse.take(),
            )
            .cast(|p| p as *mut dyn Statement),
        )
    }
    // Source: src/sksl/SkSLParser.cpp:1764-1775
    pub unsafe fn expressionStatement(&mut self) -> ParsedStatement {
        let expr = self.expression()?;
        if !self.expect(Kind::TK_SEMICOLON, b"';'", None) {
            return None;
        }
        let pos = expr.node().fPosition;
        let converted = ExpressionStatement::Convert(self.fCompiler.as_ref().context(), expr);
        Some(self.statementOrNop(pos, converted))
    }
}

use super::{
    ir::{
        SkSLBinaryExpression::BinaryExpression,
        SkSLExpression::{Expression, ExpressionExt},
        SkSLFieldAccess::FieldAccess,
        SkSLFunctionCall::FunctionCall,
        SkSLIRNode::IRNode,
        SkSLIndexExpression::IndexExpression,
        SkSLLiteral::Literal,
        SkSLPoison::Poison,
        SkSLPostfixExpression::PostfixExpression,
        SkSLPrefixExpression::PrefixExpression,
        SkSLSwizzle::Swizzle,
        SkSLTernaryExpression::TernaryExpression,
    },
    SkSLDefines::{ExpressionArray, SKSL_FLOAT, SKSL_INT},
    SkSLOperator::{Operator, OperatorKind},
};
// Native member-function pointer; method receiver is explicit in Rust.
type BinaryParseFn = unsafe fn(&mut Parser) -> Option<IRNodeBox<dyn Expression>>;
impl Parser {
    // Source: src/sksl/SkSLParser.cpp:1777-1779
    pub unsafe fn poison(&self, pos: Position) -> IRNodeBox<dyn Expression> {
        Poison::Make(pos, self.fCompiler.as_ref().context())
    }
    // Source: src/sksl/SkSLParser.cpp:1781-1795
    pub unsafe fn expressionOrPoison(
        &self,
        pos: Position,
        expr: Option<IRNodeBox<dyn Expression>>,
    ) -> IRNodeBox<dyn Expression> {
        let expr = match expr {
            Some(expr) => expr,
            None => self.poison(pos),
        };
        debug_assert!(
            !pos.valid() || expr.node().fPosition == pos,
            "expected expression position to match"
        );
        expr
    }
    // Source: src/sksl/SkSLParser.cpp:1797-1814
    unsafe fn operatorRight(
        &mut self,
        depth: &mut AutoDepth,
        op: OperatorKind,
        rightFn: BinaryParseFn,
        expr: &mut Option<IRNodeBox<dyn Expression>>,
    ) -> bool {
        self.nextToken();
        if !depth.increase() {
            return false;
        }
        let Some(right) = rightFn(self) else {
            return false;
        };
        let pos = expr
            .as_ref()
            .unwrap_unchecked()
            .node()
            .fPosition
            .rangeThrough(right.node().fPosition);
        let converted = BinaryExpression::Convert(
            self.fCompiler.as_ref().context(),
            pos,
            expr.take(),
            Operator::new(op),
            Some(right),
        );
        *expr = Some(self.expressionOrPoison(pos, converted));
        true
    }
    // Source: src/sksl/SkSLParser.cpp:1817-1838
    pub unsafe fn expression(&mut self) -> Option<IRNodeBox<dyn Expression>> {
        let mut depth = AutoDepth::new(NonNull::from(&mut *self));
        let start = self.peek();
        let mut result = self.assignmentExpression();
        if result.is_none() {
            return None;
        }
        while self.peek().fKind == Kind::TK_COMMA {
            if !self.operatorRight(
                &mut depth,
                OperatorKind::COMMA,
                Self::assignmentExpression,
                &mut result,
            ) {
                return None;
            }
        }
        let expr = result.as_ref().unwrap_unchecked();
        debug_assert!(
            expr.node().fPosition.valid(),
            "Expression has invalid position"
        );
        debug_assert!(
            expr.node().fPosition.startOffset() == Self::position(start).startOffset(),
            "Expression start must match first token"
        );
        result
    }
    // Source: src/sksl/SkSLParser.cpp:1844-1870
    pub unsafe fn assignmentExpression(&mut self) -> Option<IRNodeBox<dyn Expression>> {
        let mut depth = AutoDepth::new(NonNull::from(&mut *self));
        let mut result = self.ternaryExpression();
        if result.is_none() {
            return None;
        }
        loop {
            let op = match self.peek().fKind {
                Kind::TK_EQ => OperatorKind::EQ,
                Kind::TK_STAREQ => OperatorKind::STAREQ,
                Kind::TK_SLASHEQ => OperatorKind::SLASHEQ,
                Kind::TK_PERCENTEQ => OperatorKind::PERCENTEQ,
                Kind::TK_PLUSEQ => OperatorKind::PLUSEQ,
                Kind::TK_MINUSEQ => OperatorKind::MINUSEQ,
                Kind::TK_SHLEQ => OperatorKind::SHLEQ,
                Kind::TK_SHREQ => OperatorKind::SHREQ,
                Kind::TK_BITWISEANDEQ => OperatorKind::BITWISEANDEQ,
                Kind::TK_BITWISEXOREQ => OperatorKind::BITWISEXOREQ,
                Kind::TK_BITWISEOREQ => OperatorKind::BITWISEOREQ,
                _ => return result,
            };
            if !self.operatorRight(&mut depth, op, Self::assignmentExpression, &mut result) {
                return None;
            }
        }
    }
    // Source: src/sksl/SkSLParser.cpp:1873-1901
    pub unsafe fn ternaryExpression(&mut self) -> Option<IRNodeBox<dyn Expression>> {
        let mut depth = AutoDepth::new(NonNull::from(&mut *self));
        let base = self.logicalOrExpression()?;
        if !self.checkNext(Kind::TK_QUESTION, None) {
            return Some(base);
        }
        if !depth.increase() {
            return None;
        }
        let trueExpr = self.expression()?;
        if !self.expect(Kind::TK_COLON, b"':'", None) {
            return None;
        }
        let falseExpr = self.assignmentExpression()?;
        let pos = base
            .node()
            .fPosition
            .rangeThrough(falseExpr.node().fPosition);
        let converted = TernaryExpression::Convert(
            self.fCompiler.as_ref().context(),
            pos,
            Some(base),
            Some(trueExpr),
            Some(falseExpr),
        );
        Some(self.expressionOrPoison(pos, converted))
    }
    // Source: src/sksl/SkSLParser.cpp:1904-1917
    pub unsafe fn logicalOrExpression(&mut self) -> Option<IRNodeBox<dyn Expression>> {
        let mut depth = AutoDepth::new(NonNull::from(&mut *self));
        let mut result = self.logicalXorExpression();
        if result.is_none() {
            return None;
        }
        while self.peek().fKind == Kind::TK_LOGICALOR {
            if !self.operatorRight(
                &mut depth,
                OperatorKind::LOGICALOR,
                Self::logicalXorExpression,
                &mut result,
            ) {
                return None;
            }
        }
        result
    }
    // Source: src/sksl/SkSLParser.cpp:1920-1933
    pub unsafe fn logicalXorExpression(&mut self) -> Option<IRNodeBox<dyn Expression>> {
        let mut depth = AutoDepth::new(NonNull::from(&mut *self));
        let mut result = self.logicalAndExpression();
        if result.is_none() {
            return None;
        }
        while self.peek().fKind == Kind::TK_LOGICALXOR {
            if !self.operatorRight(
                &mut depth,
                OperatorKind::LOGICALXOR,
                Self::logicalAndExpression,
                &mut result,
            ) {
                return None;
            }
        }
        result
    }
    // Source: src/sksl/SkSLParser.cpp:1936-1949
    pub unsafe fn logicalAndExpression(&mut self) -> Option<IRNodeBox<dyn Expression>> {
        let mut depth = AutoDepth::new(NonNull::from(&mut *self));
        let mut result = self.bitwiseOrExpression();
        if result.is_none() {
            return None;
        }
        while self.peek().fKind == Kind::TK_LOGICALAND {
            if !self.operatorRight(
                &mut depth,
                OperatorKind::LOGICALAND,
                Self::bitwiseOrExpression,
                &mut result,
            ) {
                return None;
            }
        }
        result
    }
    // Source: src/sksl/SkSLParser.cpp:1952-1965
    pub unsafe fn bitwiseOrExpression(&mut self) -> Option<IRNodeBox<dyn Expression>> {
        let mut depth = AutoDepth::new(NonNull::from(&mut *self));
        let mut result = self.bitwiseXorExpression();
        if result.is_none() {
            return None;
        }
        while self.peek().fKind == Kind::TK_BITWISEOR {
            if !self.operatorRight(
                &mut depth,
                OperatorKind::BITWISEOR,
                Self::bitwiseXorExpression,
                &mut result,
            ) {
                return None;
            }
        }
        result
    }
    // Source: src/sksl/SkSLParser.cpp:1968-1981
    pub unsafe fn bitwiseXorExpression(&mut self) -> Option<IRNodeBox<dyn Expression>> {
        let mut depth = AutoDepth::new(NonNull::from(&mut *self));
        let mut result = self.bitwiseAndExpression();
        if result.is_none() {
            return None;
        }
        while self.peek().fKind == Kind::TK_BITWISEXOR {
            if !self.operatorRight(
                &mut depth,
                OperatorKind::BITWISEXOR,
                Self::bitwiseAndExpression,
                &mut result,
            ) {
                return None;
            }
        }
        result
    }
    // Source: src/sksl/SkSLParser.cpp:1984-1997
    pub unsafe fn bitwiseAndExpression(&mut self) -> Option<IRNodeBox<dyn Expression>> {
        let mut depth = AutoDepth::new(NonNull::from(&mut *self));
        let mut result = self.equalityExpression();
        if result.is_none() {
            return None;
        }
        while self.peek().fKind == Kind::TK_BITWISEAND {
            if !self.operatorRight(
                &mut depth,
                OperatorKind::BITWISEAND,
                Self::equalityExpression,
                &mut result,
            ) {
                return None;
            }
        }
        result
    }
    // Source: src/sksl/SkSLParser.cpp:2000-2017
    pub unsafe fn equalityExpression(&mut self) -> Option<IRNodeBox<dyn Expression>> {
        let mut depth = AutoDepth::new(NonNull::from(&mut *self));
        let mut result = self.relationalExpression();
        if result.is_none() {
            return None;
        }
        loop {
            let op = match self.peek().fKind {
                Kind::TK_EQEQ => OperatorKind::EQEQ,
                Kind::TK_NEQ => OperatorKind::NEQ,
                _ => return result,
            };
            if !self.operatorRight(&mut depth, op, Self::relationalExpression, &mut result) {
                return None;
            }
        }
    }
    // Source: src/sksl/SkSLParser.cpp:2020-2039
    pub unsafe fn relationalExpression(&mut self) -> Option<IRNodeBox<dyn Expression>> {
        let mut depth = AutoDepth::new(NonNull::from(&mut *self));
        let mut result = self.shiftExpression();
        if result.is_none() {
            return None;
        }
        loop {
            let op = match self.peek().fKind {
                Kind::TK_LT => OperatorKind::LT,
                Kind::TK_GT => OperatorKind::GT,
                Kind::TK_LTEQ => OperatorKind::LTEQ,
                Kind::TK_GTEQ => OperatorKind::GTEQ,
                _ => return result,
            };
            if !self.operatorRight(&mut depth, op, Self::shiftExpression, &mut result) {
                return None;
            }
        }
    }
    // Source: src/sksl/SkSLParser.cpp:2042-2059
    pub unsafe fn shiftExpression(&mut self) -> Option<IRNodeBox<dyn Expression>> {
        let mut depth = AutoDepth::new(NonNull::from(&mut *self));
        let mut result = self.additiveExpression();
        if result.is_none() {
            return None;
        }
        loop {
            let op = match self.peek().fKind {
                Kind::TK_SHL => OperatorKind::SHL,
                Kind::TK_SHR => OperatorKind::SHR,
                _ => return result,
            };
            if !self.operatorRight(&mut depth, op, Self::additiveExpression, &mut result) {
                return None;
            }
        }
    }
    // Source: src/sksl/SkSLParser.cpp:2062-2079
    pub unsafe fn additiveExpression(&mut self) -> Option<IRNodeBox<dyn Expression>> {
        let mut depth = AutoDepth::new(NonNull::from(&mut *self));
        let mut result = self.multiplicativeExpression();
        if result.is_none() {
            return None;
        }
        loop {
            let op = match self.peek().fKind {
                Kind::TK_PLUS => OperatorKind::PLUS,
                Kind::TK_MINUS => OperatorKind::MINUS,
                _ => return result,
            };
            if !self.operatorRight(&mut depth, op, Self::multiplicativeExpression, &mut result) {
                return None;
            }
        }
    }
    // Source: src/sksl/SkSLParser.cpp:2082-2100
    pub unsafe fn multiplicativeExpression(&mut self) -> Option<IRNodeBox<dyn Expression>> {
        let mut depth = AutoDepth::new(NonNull::from(&mut *self));
        let mut result = self.unaryExpression();
        if result.is_none() {
            return None;
        }
        loop {
            let op = match self.peek().fKind {
                Kind::TK_STAR => OperatorKind::STAR,
                Kind::TK_SLASH => OperatorKind::SLASH,
                Kind::TK_PERCENT => OperatorKind::PERCENT,
                _ => return result,
            };
            if !self.operatorRight(&mut depth, op, Self::unaryExpression, &mut result) {
                return None;
            }
        }
    }
    // Source: src/sksl/SkSLParser.cpp:2103-2127
    pub unsafe fn unaryExpression(&mut self) -> Option<IRNodeBox<dyn Expression>> {
        let mut depth = AutoDepth::new(NonNull::from(&mut *self));
        let start = self.peek();
        let op = match start.fKind {
            Kind::TK_PLUS => OperatorKind::PLUS,
            Kind::TK_MINUS => OperatorKind::MINUS,
            Kind::TK_LOGICALNOT => OperatorKind::LOGICALNOT,
            Kind::TK_BITWISENOT => OperatorKind::BITWISENOT,
            Kind::TK_PLUSPLUS => OperatorKind::PLUSPLUS,
            Kind::TK_MINUSMINUS => OperatorKind::MINUSMINUS,
            _ => return self.postfixExpression(),
        };
        self.nextToken();
        if !depth.increase() {
            return None;
        }
        let expr = self.unaryExpression()?;
        let pos = Position::Range(start.fOffset, expr.node().fPosition.endOffset());
        let converted = PrefixExpression::Convert(
            self.fCompiler.as_ref().context(),
            pos,
            Operator::new(op),
            expr,
        );
        Some(self.expressionOrPoison(pos, converted))
    }
    // Source: src/sksl/SkSLParser.cpp:2130-2162
    pub unsafe fn postfixExpression(&mut self) -> Option<IRNodeBox<dyn Expression>> {
        let mut depth = AutoDepth::new(NonNull::from(&mut *self));
        let mut result = self.term()?;
        loop {
            let t = self.peek();
            match t.fKind {
                Kind::TK_FLOAT_LITERAL => {
                    if self.text(t)[0] != b'.' {
                        return Some(result);
                    }
                    if !depth.increase() {
                        return None;
                    }
                    result = self.suffix(result)?;
                }
                Kind::TK_LBRACKET
                | Kind::TK_DOT
                | Kind::TK_LPAREN
                | Kind::TK_PLUSPLUS
                | Kind::TK_MINUSMINUS => {
                    if !depth.increase() {
                        return None;
                    }
                    result = self.suffix(result)?;
                }
                _ => return Some(result),
            }
        }
    }
    // Source: src/sksl/SkSLParser.cpp:2164-2176
    pub unsafe fn swizzle(
        &mut self,
        pos: Position,
        base: IRNodeBox<dyn Expression>,
        swizzleMask: &[u8],
        maskPos: Position,
    ) -> IRNodeBox<dyn Expression> {
        crate::SkASSERT!(@source b"src/sksl/SkSLParser.cpp\0".as_ptr(),2168; @condition b"!swizzleMask.empty()\0".as_ptr(); !swizzleMask.is_empty());
        if !base.expression_type().isVector() && !base.expression_type().isScalar() {
            let converted =
                FieldAccess::Convert(self.fCompiler.as_ref().context(), pos, base, swizzleMask);
            return self.expressionOrPoison(pos, converted);
        }
        let converted = Swizzle::Convert(
            self.fCompiler.as_ref().context(),
            pos,
            maskPos,
            base,
            swizzleMask,
        );
        self.expressionOrPoison(pos, converted)
    }
    // Source: src/sksl/SkSLParser.cpp:2178-2184
    pub unsafe fn call(
        &mut self,
        pos: Position,
        base: IRNodeBox<dyn Expression>,
        args: core::pin::Pin<Box<ExpressionArray>>,
    ) -> IRNodeBox<dyn Expression> {
        let converted =
            FunctionCall::ConvertValue(self.fCompiler.as_ref().context(), pos, base, args);
        self.expressionOrPoison(pos, converted)
    }
    // Source: src/sksl/SkSLParser.cpp:2188-2283
    pub unsafe fn suffix(
        &mut self,
        base: IRNodeBox<dyn Expression>,
    ) -> Option<IRNodeBox<dyn Expression>> {
        let mut depth = AutoDepth::new(NonNull::from(&mut *self));
        let next = self.nextToken();
        if !depth.increase() {
            return None;
        }
        match next.fKind {
            Kind::TK_LBRACKET => {
                if self.checkNext(Kind::TK_RBRACKET, None) {
                    let errorPos = self.rangeFromToken(next);
                    self.error_position(errorPos, b"missing index in '[]'");
                    return Some(self.poison(self.rangeFrom(base.node().fPosition)));
                }
                let index = self.expression()?;
                self.expect(
                    Kind::TK_RBRACKET,
                    b"']' to complete array access expression",
                    None,
                );
                let pos = self.rangeFrom(base.node().fPosition);
                let converted =
                    IndexExpression::Convert(self.fCompiler.as_ref().context(), pos, base, index);
                Some(self.expressionOrPoison(pos, converted))
            }
            Kind::TK_DOT | Kind::TK_FLOAT_LITERAL => {
                // Exact native DOT-to-FLOAT_LITERAL fallthrough; a failed identifier continues through this tail.
                if next.fKind == Kind::TK_DOT {
                    let mut text = MaybeUninit::<NonNull<[u8]>>::uninit();
                    if self.identifier(text.as_mut_ptr()) {
                        let pos = self.rangeFrom(base.node().fPosition);
                        let maskPos = self.rangeFrom(Self::position(next).after());
                        return Some(self.swizzle(pos, base, text.assume_init().as_ref(), maskPos));
                    }
                }
                let fieldView = NonNull::from(self.text(next));
                let fullField = fieldView.as_ref();
                debug_assert!(fullField[0] == b'.');
                let field = &fullField[1..];
                let mut pos = self.rangeFrom(base.node().fPosition);
                let start = Self::position(next);
                let start = Position::Range(start.startOffset() + 1, start.endOffset());
                let mut maskPos = self.rangeFrom(start);
                let id = self.nextRawToken();
                if id.fKind == Kind::TK_IDENTIFIER {
                    pos = self.rangeFrom(base.node().fPosition);
                    maskPos = self.rangeFrom(start);
                    let mut mask = field.to_vec();
                    mask.extend_from_slice(self.text(id));
                    return Some(self.swizzle(pos, base, &mask, maskPos));
                }
                if field.is_empty() {
                    self.error_position(pos, b"expected field name or swizzle mask after '.'");
                    return Some(self.poison(pos));
                }
                self.pushback(id);
                Some(self.swizzle(pos, base, field, maskPos))
            }
            Kind::TK_LPAREN => {
                let mut args = ExpressionArray::new();
                if self.peek().fKind != Kind::TK_RPAREN {
                    loop {
                        let expr = self.assignmentExpression()?;
                        args.as_mut().as_base_mut().push_back(Some(expr));
                        if !self.checkNext(Kind::TK_COMMA, None) {
                            break;
                        }
                    }
                }
                self.expect(Kind::TK_RPAREN, b"')' to complete function arguments", None);
                let pos = self.rangeFrom(base.node().fPosition);
                Some(self.call(pos, base, args))
            }
            Kind::TK_PLUSPLUS | Kind::TK_MINUSMINUS => {
                let op = if next.fKind == Kind::TK_PLUSPLUS {
                    OperatorKind::PLUSPLUS
                } else {
                    OperatorKind::MINUSMINUS
                };
                let pos = self.rangeFrom(base.node().fPosition);
                let converted = PostfixExpression::Convert(
                    self.fCompiler.as_ref().context(),
                    pos,
                    base,
                    Operator::new(op),
                );
                Some(self.expressionOrPoison(pos, converted))
            }
            _ => {
                let mut msg = b"expected expression suffix, but found '".to_vec();
                msg.extend_from_slice(self.text(next));
                msg.push(b'\'');
                self.error(next, &msg);
                None
            }
        }
    }
    // Source: src/sksl/SkSLParser.cpp:2286-2346
    pub unsafe fn term(&mut self) -> Option<IRNodeBox<dyn Expression>> {
        let mut depth = AutoDepth::new(NonNull::from(&mut *self));
        let t = self.peek();
        match t.fKind {
            Kind::TK_IDENTIFIER => {
                let mut text = MaybeUninit::<NonNull<[u8]>>::uninit();
                if self.identifier(text.as_mut_ptr()) {
                    let pos = Self::position(t);
                    let table = self.symbolTable().unwrap_unchecked();
                    let converted = table.as_ref().instantiateSymbolRef(
                        self.fCompiler.as_ref().context(),
                        text.assume_init().as_ref(),
                        pos,
                    );
                    return Some(self.expressionOrPoison(pos, converted));
                }
            }
            Kind::TK_INT_LITERAL => {
                let mut i = MaybeUninit::<SKSL_INT>::uninit();
                if !self.intLiteral(i.as_mut_ptr()) {
                    i.write(0);
                }
                let pos = Self::position(t);
                let value = Literal::MakeIntContext(
                    self.fCompiler.as_ref().context(),
                    pos,
                    i.assume_init(),
                )
                .cast(|p| p as *mut dyn Expression);
                return Some(self.expressionOrPoison(pos, Some(value)));
            }
            Kind::TK_FLOAT_LITERAL => {
                let mut f = MaybeUninit::<SKSL_FLOAT>::uninit();
                if !self.floatLiteral(f.as_mut_ptr()) {
                    f.write(0.0);
                }
                let pos = Self::position(t);
                let value = Literal::MakeFloatContext(
                    self.fCompiler.as_ref().context(),
                    pos,
                    f.assume_init(),
                )
                .cast(|p| p as *mut dyn Expression);
                return Some(self.expressionOrPoison(pos, Some(value)));
            }
            Kind::TK_TRUE_LITERAL | Kind::TK_FALSE_LITERAL => {
                let mut b = MaybeUninit::<bool>::uninit();
                let ok = self.boolLiteral(b.as_mut_ptr());
                debug_assert!(ok);
                let pos = Self::position(t);
                let value = Literal::MakeBoolContext(
                    self.fCompiler.as_ref().context(),
                    pos,
                    b.assume_init(),
                )
                .cast(|p| p as *mut dyn Expression);
                return Some(self.expressionOrPoison(pos, Some(value)));
            }
            Kind::TK_LPAREN => {
                self.nextToken();
                if !depth.increase() {
                    return None;
                }
                let result = self.expression();
                if let Some(mut result) = result {
                    self.expect(Kind::TK_RPAREN, b"')' to complete expression", None);
                    let pos = self.rangeFrom(Self::position(t));
                    result.as_pin_mut().node_mut().setPosition(pos);
                    return Some(result);
                }
            }
            _ => {
                self.nextToken();
                let mut msg = b"expected expression, but found '".to_vec();
                msg.extend_from_slice(self.text(t));
                msg.push(b'\'');
                self.error(t, &msg);
                self.fEncounteredFatalError.write(true);
            }
        }
        None
    }
    // Source: src/sksl/SkSLParser.cpp:2349-2360
    pub unsafe fn intLiteral(&mut self, dest: *mut SKSL_INT) -> bool {
        let mut t = Token::default();
        if !self.expect(Kind::TK_INT_LITERAL, b"integer literal", Some(&mut t)) {
            return false;
        }
        let view = NonNull::from(self.text(t));
        let s = view.as_ref();
        if !super::SkSLString::stoi_raw(s, dest) {
            let mut msg = b"integer is too large: ".to_vec();
            msg.extend_from_slice(s);
            self.error(t, &msg);
            return false;
        }
        true
    }
    // Source: src/sksl/SkSLParser.cpp:2363-2374
    pub unsafe fn floatLiteral(&mut self, dest: *mut SKSL_FLOAT) -> bool {
        let mut t = Token::default();
        if !self.expect(Kind::TK_FLOAT_LITERAL, b"float literal", Some(&mut t)) {
            return false;
        }
        let view = NonNull::from(self.text(t));
        let s = view.as_ref();
        // Genuine Apple libc++ classic sentry/num_get<float> extraction is adapted in SkSLString.rs; UNVERIFIED.
        if !super::SkSLString::stod_raw(s, dest) {
            let mut msg = b"floating-point value is too large: ".to_vec();
            msg.extend_from_slice(s);
            self.error(t, &msg);
            return false;
        }
        true
    }
    // Source: src/sksl/SkSLParser.cpp:2377-2391
    pub unsafe fn boolLiteral(&mut self, dest: *mut bool) -> bool {
        let t = self.nextToken();
        match t.fKind {
            Kind::TK_TRUE_LITERAL => {
                dest.write(true);
                true
            }
            Kind::TK_FALSE_LITERAL => {
                dest.write(false);
                true
            }
            _ => {
                let mut msg = b"expected 'true' or 'false', but found '".to_vec();
                msg.extend_from_slice(self.text(t));
                msg.push(b'\'');
                self.error(t, &msg);
                false
            }
        }
    }
    // Source: src/sksl/SkSLParser.cpp:2394-2401
    pub unsafe fn identifier(&mut self, dest: *mut NonNull<[u8]>) -> bool {
        let mut t = Token::default();
        if self.expect(Kind::TK_IDENTIFIER, b"identifier", Some(&mut t)) {
            dest.write(NonNull::from(self.text(t)));
            return true;
        }
        false
    }
}
