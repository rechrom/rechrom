// Copyright 2021 Google LLC. BSD-style license. Direct pinned-source translation, unverified.
use super::{
    SkColorSpace::SkColorSpace, SkData::SkData, SkRect::SkRect, SkRuntimeEffect as re,
    SkString::SkString,
};
use crate::include::core::SkAlphaType::SkAlphaType;
use crate::include::private::SkTArray::STArray;
use crate::src::sksl::{ir::SkSLProgram::Program, SkSLContext::Context};
use core::{ffi::c_void, ptr::NonNull};
use core::{mem::ManuallyDrop, pin::Pin};
use std::sync::Arc;
// Source: include/core/SkMesh.h:27-31
// Genuine incomplete native declaration: no layout, marker object or constructor.
unsafe extern "C" {
    pub type GrDirectContext;
}
// Source: include/core/SkMesh.h:66-71
pub const kMaxStride: usize = 1024;
pub const kMaxAttributes: usize = 8;
pub const kStrideAlignment: usize = 4;
pub const kOffsetAlignment: usize = 4;
pub const kMaxVaryings: usize = 6;
// Source: include/core/SkMesh.h:73-88
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AttributeType {
    kFloat,
    kFloat2,
    kFloat3,
    kFloat4,
    kUByte4_unorm,
}
impl AttributeType {
    pub const kLast: Self = Self::kUByte4_unorm;
}
#[derive(Clone)]
pub struct Attribute {
    pub r#type: AttributeType,
    pub offset: usize,
    pub name: SkString,
}
// Source: include/core/SkMesh.h:90-106
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VaryingType {
    kFloat,
    kFloat2,
    kFloat3,
    kFloat4,
    kHalf,
    kHalf2,
    kHalf3,
    kHalf4,
}
impl VaryingType {
    pub const kLast: Self = Self::kHalf4;
}
#[derive(Clone)]
pub struct Varying {
    pub r#type: VaryingType,
    pub name: SkString,
}
// Source: include/core/SkMesh.h:108-109
pub type Uniform<'a> = re::Uniform<'a>;
pub type Child<'a> = re::Child<'a>;
pub use re::ChildPtr;
// Source: include/core/SkMesh.h:113-116
pub struct SpecificationResult {
    pub specification: Option<Arc<SkMeshSpecification>>,
    pub error: SkString,
}
// Source: include/core/SkMesh.h:205-209
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ColorType {
    kNone,
    kHalf4,
    kFloat4,
}
// Source: include/core/SkMesh.h:244-256
// Reflected names borrow stable owned Program IR; public views shorten all name leases to &self.
pub struct SkMeshSpecification {
    pub(crate) fAttributes: Vec<Attribute>,
    pub(crate) fVaryings: Vec<Varying>,
    pub(crate) fUniforms: Vec<Uniform<'static>>,
    pub(crate) fChildren: Vec<Child<'static>>,
    pub(crate) fVS: core::mem::ManuallyDrop<Box<Program>>,
    pub(crate) fFS: core::mem::ManuallyDrop<Box<Program>>,
    pub(crate) fStride: usize,
    pub(crate) fHash: u32,
    pub(crate) fPassthroughLocalCoordsVaryingIndex: i32,
    pub(crate) fDeadVaryingMask: u32,
    pub(crate) fColorType: ColorType,
    pub(crate) fColorSpace: Option<Arc<SkColorSpace>>,
    pub(crate) fAlphaType: SkAlphaType,
}
fn string(b: &[u8]) -> SkString {
    SkString::from_view(b)
}
fn bytes(s: &SkString) -> &[u8] {
    unsafe { core::slice::from_raw_parts(s.data(), s.size()) }
}
fn cbytes(s: &SkString) -> &[u8] {
    let b = bytes(s);
    &b[..b.iter().position(|&x| x == 0).unwrap_or(b.len())]
}
fn named(prefix: &[u8], name: &[u8], suffix: &[u8]) -> Vec<u8> {
    let mut b = prefix.to_vec();
    b.extend_from_slice(name);
    b.extend_from_slice(suffix);
    b
}
fn failure(b: impl AsRef<[u8]>) -> SpecificationResult {
    SpecificationResult {
        specification: None,
        error: string(b.as_ref()),
    }
}
// Source: src/core/SkMesh.cpp:60-62
macro_rules! RETURN_FAILURE {($fmt:literal $(,$arg:expr)* $(,)?)=>{
 return SpecificationResult{specification:None,error:unsafe{super::SkString::SkStringPrintf(concat!($fmt,"\0").as_ptr() $(,$arg)*)}};
};}
macro_rules! RETURN_ERROR {($fmt:literal $(,$arg:expr)* $(,)?)=>{
 return Err(unsafe{super::SkString::SkStringPrintf(concat!($fmt,"\0").as_ptr() $(,$arg)*)});
};}
// Source: src/core/SkMesh.cpp:69-73
fn find_uniform(uniforms: &[Uniform<'_>], name: &[u8]) -> Option<usize> {
    uniforms.iter().position(|u| u.name == name)
}
// Source: src/core/SkMesh.cpp:76-135
unsafe fn gather_uniforms_and_check_for_main(
    program: &Program,
    uniforms: &mut Vec<Uniform<'static>>,
    children: &mut Vec<Child<'static>>,
    stage: u32,
    offset: &mut usize,
) -> Result<(), SkString> {
    use crate::src::sksl::ir::{
        SkSLFunctionDefinition::FunctionDefinition,
        SkSLProgramElement::Kind,
        SkSLVarDeclarations::{GlobalVarDeclaration, VarDeclaration},
    };
    let mut foundMain = false;
    for elem in program.elements().begin() {
        let elem = elem.expect("native nonnull program element");
        if elem.kind() == Kind::kFunction {
            if elem
                .node()
                .as_node::<FunctionDefinition>()
                .declaration()
                .isMain()
            {
                foundMain = true;
            }
        } else if elem.kind() == Kind::kGlobalVar {
            let global = elem.node().as_node::<GlobalVarDeclaration>();
            let vd = global.declaration().node().as_node::<VarDeclaration>();
            let var = vd.var().unwrap_unchecked().as_ref();
            if var.modifierFlags().isUniform() {
                if var.symbol_type().isEffectChild() {
                    let child = re::VarAsChild(var, children.len() as i32);
                    children.push(core::mem::transmute::<Child<'_>, Child<'static>>(child));
                } else if let Some(i) = find_uniform(uniforms, var.name()) {
                    let mut ignoredOffset = 0;
                    let uniform = re::VarAsUniform(
                        var,
                        program.fContext.as_ref().unwrap_unchecked(),
                        &mut ignoredOffset,
                    );
                    let prior = &mut uniforms[i];
                    if uniform.isArray() != prior.isArray()
                        || uniform.r#type != prior.r#type
                        || uniform.count != prior.count
                    {
                        return Err(unsafe {
                            super::SkString::SkStringPrintf(b"Uniform %.*s declared with different types in vertex and fragment shaders.\0".as_ptr(),var.name().len() as core::ffi::c_int,var.name().as_ptr())
                        });
                    }
                    if uniform.isColor() != prior.isColor() {
                        return Err(unsafe {
                            super::SkString::SkStringPrintf(b"Uniform %.*s declared with different color layout in vertex and fragment shaders.\0".as_ptr(),var.name().len() as core::ffi::c_int,var.name().as_ptr())
                        });
                    }
                    prior.flags |= stage;
                } else {
                    let mut uniform =
                        re::VarAsUniform(var, program.fContext.as_ref().unwrap_unchecked(), offset);
                    uniform.flags |= stage;
                    uniforms.push(core::mem::transmute::<Uniform<'_>, Uniform<'static>>(
                        uniform,
                    ));
                }
            }
        }
    }
    if foundMain {
        Ok(())
    } else {
        Err(string(b"No main function found."))
    }
}
// Source: src/core/SkMesh.cpp:139-158
unsafe fn get_fs_color_type(program: &Program) -> ColorType {
    use crate::src::sksl::ir::{
        SkSLFunctionDefinition::FunctionDefinition, SkSLProgramElement::Kind,
    };
    for elem in program.elements().begin() {
        let elem = elem.unwrap_unchecked();
        if elem.kind() == Kind::kFunction {
            let decl = elem.node().as_node::<FunctionDefinition>().declaration();
            if decl.isMain() {
                crate::SkASSERT!(@source b"src/core/SkMesh.cpp\0".as_ptr(), 145; @condition b"decl.parameters().size() == 1 || decl.parameters().size() == 2\0".as_ptr(); decl.parameters().len()==1||decl.parameters().len()==2);
                if decl.parameters().len() == 1 {
                    return ColorType::kNone;
                }
                let ty = decl.parameters()[1]
                    .unwrap_unchecked()
                    .as_ref()
                    .symbol_type();
                let ctx = program.fContext.as_ref().unwrap_unchecked();
                crate::SkASSERT!(@source b"src/core/SkMesh.cpp\0".as_ptr(), 150; @condition b"paramType.matches(*fsProgram.fContext->fTypes.fHalf4) || paramType.matches(*fsProgram.fContext->fTypes.fFloat4)\0".as_ptr(); ty.matches(&**ctx.fTypes.as_ref().fHalf4)||ty.matches(&**ctx.fTypes.as_ref().fFloat4));
                return if ty.matches(&**ctx.fTypes.as_ref().fHalf4) {
                    ColorType::kHalf4
                } else {
                    ColorType::kFloat4
                };
            }
        }
    }
    crate::SkUNREACHABLE!(@source b"src/core/SkMesh.cpp\0".as_ptr(), 157; )
}
// Source: src/core/SkMesh.cpp:163-173
fn check_name(name: &SkString) -> bool {
    !name.isEmpty()
        && bytes(name)
            .iter()
            .all(|&c| c == b'_' || c.is_ascii_alphanumeric())
}
// Source: src/core/SkMesh.cpp:175-184
pub(crate) fn attribute_type_size(t: AttributeType) -> usize {
    match t {
        AttributeType::kFloat | AttributeType::kUByte4_unorm => 4,
        AttributeType::kFloat2 => 8,
        AttributeType::kFloat3 => 12,
        AttributeType::kFloat4 => 16,
    }
}
// Source: src/core/SkMesh.cpp:186-195
fn attribute_type_string(t: AttributeType) -> &'static [u8] {
    match t {
        AttributeType::kFloat => b"float\0",
        AttributeType::kFloat2 => b"float2\0",
        AttributeType::kFloat3 => b"float3\0",
        AttributeType::kFloat4 => b"float4\0",
        AttributeType::kUByte4_unorm => b"half4\0",
    }
}
// Source: src/core/SkMesh.cpp:197-209
fn varying_type_string(t: VaryingType) -> &'static [u8] {
    match t {
        VaryingType::kFloat => b"float\0",
        VaryingType::kFloat2 => b"float2\0",
        VaryingType::kFloat3 => b"float3\0",
        VaryingType::kFloat4 => b"float4\0",
        VaryingType::kHalf => b"half\0",
        VaryingType::kHalf2 => b"half2\0",
        VaryingType::kHalf3 => b"half3\0",
        VaryingType::kHalf4 => b"half4\0",
    }
}
// Source: src/core/SkMesh.cpp:212-252
pub fn check_vertex_offsets_and_stride(
    attributes: &[Attribute],
    stride: usize,
) -> Result<(), SkString> {
    const {
        assert!(
            kMaxStride <= 2048
                && kMaxAttributes <= 8
                && kStrideAlignment >= 4
                && kOffsetAlignment >= 4
                && kMaxVaryings <= 6
        );
        assert!(kStrideAlignment.is_power_of_two());
    }
    if attributes.is_empty() {
        RETURN_ERROR!("At least 1 attribute is required.");
    }
    if attributes.len() > kMaxAttributes {
        RETURN_ERROR!("A maximum of %zu attributes is allowed.", kMaxAttributes);
    }
    if stride == 0 || stride & (kStrideAlignment - 1) != 0 {
        RETURN_ERROR!(
            "Vertex stride must be a non-zero multiple of %zu.",
            kStrideAlignment
        );
    }
    if stride > kMaxStride {
        RETURN_ERROR!("Stride cannot exceed %zu.", kMaxStride);
    }
    for a in attributes {
        if a.offset & (kOffsetAlignment - 1) != 0 {
            RETURN_ERROR!(
                "Attribute offset must be a multiple of %zu.",
                kOffsetAlignment
            );
        }
        if a.offset >= stride || a.offset + attribute_type_size(a.r#type) > stride {
            RETURN_ERROR!("Attribute offset plus size cannot exceed stride.");
        }
    }
    Ok(())
}
// Source: src/core/SkMesh.cpp:254-387
unsafe fn check_for_passthrough_local_coords_and_dead_varyings(
    program: &Program,
    dead: &mut u32,
) -> i32 {
    use crate::src::sksl::{
        analysis::SkSLProgramVisitor as pv,
        ir::{
            SkSLExpression::{Expression, ExpressionExt},
            SkSLFieldAccess::FieldAccess,
            SkSLFunctionDefinition::FunctionDefinition,
            SkSLProgramElement::{Kind as PEKind, ProgramElement},
            SkSLReturnStatement::ReturnStatement,
            SkSLStatement::{Kind as StKind, Statement},
            SkSLStructDefinition::StructDefinition,
            SkSLType::Type,
            SkSLVariable::Variable,
            SkSLVariableReference::VariableReference,
        },
    };
    struct Visitor<'a> {
        context: &'a Context,
        varyingsType: Option<NonNull<dyn Type>>,
        varyings: Option<NonNull<dyn Variable>>,
        passthrough: i32,
        inMain: bool,
        useMask: u32,
    }
    impl Visitor<'_> {
        fn failed(&mut self) {
            if self.passthrough >= 0 {
                self.useMask |= 1u32 << self.passthrough;
            }
            self.passthrough = -2;
        }
    }
    unsafe impl pv::ProgramVisitor for Visitor<'_> {
        fn visitProgramElement(&mut self, p: &dyn ProgramElement) -> bool {
            unsafe {
                if p.kind() == PEKind::kStructDefinition {
                    let def = p.node().as_node::<StructDefinition>();
                    if def.r#type().name() == b"Varyings" {
                        self.varyingsType =
                            Some(core::mem::transmute::<
                                NonNull<dyn Type + '_>,
                                NonNull<dyn Type>,
                            >(NonNull::from(def.r#type())));
                    }
                    return false;
                }
                if p.kind() == PEKind::kFunction
                    && p.node()
                        .as_node::<FunctionDefinition>()
                        .declaration()
                        .isMain()
                {
                    crate::SkASSERT!(@source b"src/core/SkMesh.cpp\0".as_ptr(), 282; @condition b"!fVaryings\0".as_ptr(); self.varyings.is_none());
                    self.varyings = p
                        .node()
                        .as_node::<FunctionDefinition>()
                        .declaration()
                        .parameters()[0];
                    debug_assert!(self
                        .varyingsType
                        .unwrap_unchecked()
                        .as_ref()
                        .matches(self.varyings.unwrap_unchecked().as_ref().symbol_type()));
                    self.inMain = true;
                    let result = pv::inheritedProgramElement(self, p);
                    self.inMain = false;
                    return result;
                }
                pv::inheritedProgramElement(self, p)
            }
        }
        fn visitStatement(&mut self, s: &dyn Statement) -> bool {
            unsafe {
                if !self.inMain {
                    return pv::inheritedStatement(self, s);
                }
                {
                    crate::SkASSERT!(@source b"src/core/SkMesh.cpp\0".as_ptr(), 300; @condition b"fVaryings\0".as_ptr(); self.varyings.is_some());
                    crate::SkASSERT!(@source b"src/core/SkMesh.cpp\0".as_ptr(), 301; @condition b"fVaryingsType\0".as_ptr(); self.varyingsType.is_some());
                };
                if self.passthrough == -2 || s.kind() != StKind::kReturn {
                    return pv::inheritedStatement(self, s);
                }
                let rs = s.node().as_node::<ReturnStatement>();
                let expr = rs.expression().as_ref().unwrap_unchecked();
                if !expr.node().is::<FieldAccess>() {
                    self.failed();
                    return pv::inheritedStatement(self, s);
                }
                let fa = expr.node().as_node::<FieldAccess>();
                let base = fa.base().as_ref().unwrap_unchecked();
                if !base.node().is::<VariableReference>() {
                    self.failed();
                    return pv::inheritedStatement(self, s);
                }
                let br = base.node().as_node::<VariableReference>();
                if !core::ptr::addr_eq(
                    br.variable().unwrap_unchecked().as_ptr(),
                    self.varyings.unwrap_unchecked().as_ptr(),
                ) {
                    self.failed();
                    return pv::inheritedStatement(self, s);
                }
                if self.passthrough >= 0 {
                    if fa.fieldIndex() != self.passthrough {
                        self.failed();
                        return pv::inheritedStatement(self, s);
                    }
                    return false;
                }
                let field = &self
                    .varyings
                    .unwrap_unchecked()
                    .as_ref()
                    .symbol_type()
                    .fields()[fa.fieldIndex() as usize];
                if !field
                    .fType
                    .unwrap_unchecked()
                    .as_ref()
                    .matches(&**self.context.fTypes.as_ref().fFloat2)
                {
                    self.failed();
                    return pv::inheritedStatement(self, s);
                }
                self.passthrough = fa.fieldIndex();
                false
            }
        }
        fn visitExpression(&mut self, e: &dyn Expression) -> bool {
            unsafe {
                let Some(ty) = self.varyingsType else {
                    return false;
                };
                if !e.node().is::<FieldAccess>() {
                    return pv::inheritedExpression(self, e);
                }
                let fa = e.node().as_node::<FieldAccess>();
                if !fa
                    .base()
                    .as_ref()
                    .unwrap_unchecked()
                    .expression_type()
                    .matches(ty.as_ref())
                {
                    return pv::inheritedExpression(self, e);
                }
                self.useMask |= 1u32 << fa.fieldIndex();
                false
            }
        }
    }
    let mut visitor = Visitor {
        context: program.fContext.as_ref().unwrap_unchecked(),
        varyingsType: None,
        varyings: None,
        passthrough: -1,
        inMain: false,
        useMask: 0,
    };
    use pv::ProgramVisitor;
    for elem in program.elements().begin() {
        if visitor.visitProgramElement(elem.unwrap_unchecked()) {
            break;
        }
    }
    *dead = !visitor.useMask;
    visitor.passthrough
}
impl SkMeshSpecification {
    pub const kMaxStride: usize = kMaxStride;
    pub const kMaxAttributes: usize = kMaxAttributes;
    pub const kStrideAlignment: usize = kStrideAlignment;
    pub const kOffsetAlignment: usize = kOffsetAlignment;
    pub const kMaxVaryings: usize = kMaxVaryings;
    // Source: src/core/SkMesh.cpp:389-401
    pub unsafe fn Make(
        attributes: &[Attribute],
        stride: usize,
        varyings: &[Varying],
        vs: &SkString,
        fs: &SkString,
    ) -> SpecificationResult {
        Self::MakeWithColorSpace(
            attributes,
            stride,
            varyings,
            vs,
            fs,
            Some(SkColorSpace::MakeSRGB()),
        )
    }
    // Source: src/core/SkMesh.cpp:403-410
    pub unsafe fn MakeWithColorSpace(
        attributes: &[Attribute],
        stride: usize,
        varyings: &[Varying],
        vs: &SkString,
        fs: &SkString,
        cs: Option<Arc<SkColorSpace>>,
    ) -> SpecificationResult {
        Self::MakeWithAlphaType(
            attributes,
            stride,
            varyings,
            vs,
            fs,
            cs,
            SkAlphaType::kPremul_SkAlphaType,
        )
    }
    // Source: src/core/SkMesh.cpp:412-471
    pub unsafe fn MakeWithAlphaType(
        attributes: &[Attribute],
        stride: usize,
        varyings: &[Varying],
        vs: &SkString,
        fs: &SkString,
        cs: Option<Arc<SkColorSpace>>,
        at: SkAlphaType,
    ) -> SpecificationResult {
        let mut attrs = string(b"struct Attributes {\n");
        for a in attributes {
            unsafe {
                attrs.appendf(
                    b"  %s %s;\n\0".as_ptr(),
                    attribute_type_string(a.r#type).as_ptr(),
                    a.name.c_str(),
                );
            }
        }
        unsafe {
            attrs.append(b"};\n".as_ptr(), 3);
        }
        let mut provided = false;
        for v in varyings {
            if bytes(&v.name) == b"position" {
                if v.r#type != VaryingType::kFloat2 {
                    return failure(b"Varying \"position\" must have type float2.");
                }
                provided = true;
            }
        }
        let mut temp = std::pin::pin!(STArray::<Varying, 6>::new());
        temp.as_mut().init();
        let varyings = if !provided {
            if varyings.len() > kMaxVaryings - 1 {
                RETURN_FAILURE!("A maximum of %zu varyings is allowed.", kMaxVaryings);
            }
            for v in varyings {
                temp.as_mut().push_back(v.clone());
            }
            temp.as_mut().push_back(Varying {
                r#type: VaryingType::kFloat2,
                name: string(b"position"),
            });
            temp.as_ref().get_ref().as_slice()
        } else {
            varyings
        };
        let mut varies = string(b"struct Varyings {\n");
        for v in varyings {
            unsafe {
                varies.appendf(
                    b"  %s %s;\n\0".as_ptr(),
                    varying_type_string(v.r#type).as_ptr(),
                    v.name.c_str(),
                );
            }
        }
        unsafe {
            varies.append(b"};\n".as_ptr(), 3);
        }
        let mut fullVS = bytes(&varies).to_vec();
        fullVS.extend_from_slice(bytes(&attrs));
        fullVS.extend_from_slice(cbytes(vs));
        let mut fullFS = bytes(&varies).to_vec();
        fullFS.extend_from_slice(cbytes(fs));
        Self::MakeFromSourceWithStructs(attributes, stride, varyings, &fullVS, &fullFS, cs, at)
    }
    // Source: src/core/SkMesh.cpp:473-593
    unsafe fn MakeFromSourceWithStructs(
        attributes: &[Attribute],
        stride: usize,
        varyings: &[Varying],
        vs: &[u8],
        fs: &[u8],
        mut cs: Option<Arc<SkColorSpace>>,
        mut at: SkAlphaType,
    ) -> SpecificationResult {
        if let Err(error) = check_vertex_offsets_and_stride(attributes, stride) {
            return SpecificationResult {
                specification: None,
                error,
            };
        }
        for a in attributes {
            if !check_name(&a.name) {
                RETURN_FAILURE!("\"%s\" is not a valid attribute name.", a.name.c_str());
            }
        }
        if varyings.len() > kMaxVaryings {
            RETURN_FAILURE!("A maximum of %zu varyings is allowed.", kMaxVaryings);
        }
        for v in varyings {
            if !check_name(&v.name) {
                RETURN_FAILURE!("\"%s\" is not a valid varying name.", v.name.c_str());
            }
        }
        let mut uniforms = Vec::new();
        let mut children = Vec::new();
        let mut offset = 0;
        let mut compiler = crate::src::sksl::SkSLCompiler::Compiler::new();
        let mut settings = crate::src::sksl::SkSLProgramSettings::ProgramSettings::default();
        settings.fUseMemoryPool = false;
        use crate::src::sksl::SkSLAnalysis::CallsColorTransformIntrinsics;
        use crate::src::sksl::SkSLProgramKind::ProgramKind;
        let Some(vsp) =
            compiler
                .as_mut()
                .convertProgram(ProgramKind::kMeshVertex, vs.to_vec(), &settings)
        else {
            let mut error = compiler.as_mut().get_unchecked_mut().errorText(true);
            error.push(0);
            RETURN_FAILURE!("VS: %s", error.as_ptr());
        };
        if let Err(error) = gather_uniforms_and_check_for_main(
            &vsp,
            &mut uniforms,
            &mut children,
            re::kVertex_Flag,
            &mut offset,
        ) {
            return SpecificationResult {
                specification: None,
                error,
            };
        }
        if CallsColorTransformIntrinsics(&vsp) {
            RETURN_FAILURE!("Color transform intrinsics are not permitted in custom mesh shaders");
        }
        let Some(fsp) =
            compiler
                .as_mut()
                .convertProgram(ProgramKind::kMeshFragment, fs.to_vec(), &settings)
        else {
            let mut error = compiler.as_mut().get_unchecked_mut().errorText(true);
            error.push(0);
            RETURN_FAILURE!("FS: %s", error.as_ptr());
        };
        if let Err(error) = gather_uniforms_and_check_for_main(
            &fsp,
            &mut uniforms,
            &mut children,
            re::kFragment_Flag,
            &mut offset,
        ) {
            return SpecificationResult {
                specification: None,
                error,
            };
        }
        if CallsColorTransformIntrinsics(&fsp) {
            RETURN_FAILURE!("Color transform intrinsics are not permitted in custom mesh shaders");
        }
        let ct = get_fs_color_type(&fsp);
        if ct == ColorType::kNone {
            cs = None;
            at = SkAlphaType::kPremul_SkAlphaType;
        } else {
            if cs.is_none() {
                return failure(b"Must provide a color space if FS returns a color.");
            }
            if at == SkAlphaType::kUnknown_SkAlphaType {
                return failure(b"Must provide a valid alpha type if FS returns a color.");
            }
        }
        let mut dead = 0;
        let passthrough = check_for_passthrough_local_coords_and_dead_varyings(&fsp, &mut dead);
        if passthrough >= 0 {
            crate::SkASSERT!(@source b"src/core/SkMesh.cpp\0".as_ptr(), 577; @condition b"varyings[passthroughLocalCoordsVaryingIndex].type == Varying::Type::kFloat2\0".as_ptr(); varyings[passthrough as usize].r#type==VaryingType::kFloat2);
        }
        SpecificationResult {
            specification: Some(Arc::new(Self::new(
                attributes,
                stride,
                varyings,
                passthrough,
                dead,
                uniforms,
                children,
                vsp,
                fsp,
                ct,
                cs,
                at,
            ))),
            error: SkString::default(),
        }
    }
    // Source: src/core/SkMesh.cpp:597-640
    unsafe fn new(
        attributes: &[Attribute],
        stride: usize,
        varyings: &[Varying],
        passthrough: i32,
        dead: u32,
        uniforms: Vec<Uniform<'static>>,
        children: Vec<Child<'static>>,
        vs: Box<Program>,
        fs: Box<Program>,
        ct: ColorType,
        cs: Option<Arc<SkColorSpace>>,
        at: SkAlphaType,
    ) -> Self {
        use super::SkChecksum::Hash32;
        let mut hash = Hash32(vs.fSource.as_ref().unwrap_unchecked(), 0);
        hash = Hash32(fs.fSource.as_ref().unwrap_unchecked(), hash);
        for a in attributes {
            hash = Hash32(&a.offset.to_ne_bytes(), hash);
            hash = Hash32(&(a.r#type as u32).to_ne_bytes(), hash);
        }
        hash = Hash32(&stride.to_ne_bytes(), hash);
        let csHash = cs.as_ref().map_or(0, |cs| cs.hash());
        hash = Hash32(&csHash.to_ne_bytes(), hash);
        hash = Hash32(&(at as u32).to_ne_bytes(), hash);
        Self {
            fAttributes: attributes.to_vec(),
            fVaryings: varyings.to_vec(),
            fUniforms: uniforms,
            fChildren: children,
            fVS: core::mem::ManuallyDrop::new(vs),
            fFS: core::mem::ManuallyDrop::new(fs),
            fStride: stride,
            fHash: hash,
            fPassthroughLocalCoordsVaryingIndex: passthrough,
            fDeadVaryingMask: dead,
            fColorType: ct,
            fColorSpace: cs,
            fAlphaType: at,
        }
    }
    // Source: include/core/SkMesh.h:166-166,179-183,197-199
    pub fn attributes(&self) -> &[Attribute] {
        &self.fAttributes
    }
    pub fn uniforms(&self) -> &[Uniform<'_>] {
        &self.fUniforms
    }
    pub fn children(&self) -> &[Child<'_>] {
        &self.fChildren
    }
    pub fn stride(&self) -> usize {
        self.fStride
    }
    pub fn colorSpace(&self) -> Option<&SkColorSpace> {
        self.fColorSpace.as_deref()
    }
    // Source: src/core/SkMesh.cpp:642-645
    pub fn uniformSize(&self) -> usize {
        self.fUniforms.last().map_or(0, |u| {
            (u.offset.wrapping_add(u.sizeInBytes()).wrapping_add(3)) & !3
        })
    }
    // Source: src/core/SkMesh.cpp:674-681
    pub fn findUniform(&self, name: &[u8]) -> Option<&Uniform<'_>> {
        self.fUniforms.iter().find(|u| u.name == name)
    }
    pub fn findChild(&self, name: &[u8]) -> Option<&Child<'_>> {
        self.fChildren.as_slice().iter().find(|u| u.name == name)
    }
    pub fn findAttribute(&self, name: &[u8]) -> Option<&Attribute> {
        self.fAttributes.iter().find(|a| cbytes(&a.name) == name)
    }
    pub fn findVarying(&self, name: &[u8]) -> Option<&Varying> {
        self.fVaryings.iter().find(|v| cbytes(&v.name) == name)
    }
}
// Source: include/core/SkMesh.h:284-314
// Genuine pure virtual interfaces. The private Buffer protocol is inherited by
// every admitted concrete VB/IB, matching native static-cast preconditions.
pub trait IndexBuffer: super::SkMeshPriv::Buffer {
    fn size(&self) -> usize;
    unsafe fn onUpdate(
        &self,
        dc: *mut GrDirectContext,
        data: *const u8,
        offset: usize,
        size: usize,
    ) -> bool;
    // Source: src/core/SkMesh.cpp:881-886
    unsafe fn update(
        &self,
        dc: *mut GrDirectContext,
        data: *const u8,
        offset: usize,
        size: usize,
    ) -> bool {
        check_update(data, offset, size, self.size()) && self.onUpdate(dc, data, offset, size)
    }
}
pub trait VertexBuffer: super::SkMeshPriv::Buffer {
    fn size(&self) -> usize;
    unsafe fn onUpdate(
        &self,
        dc: *mut GrDirectContext,
        data: *const u8,
        offset: usize,
        size: usize,
    ) -> bool;
    // Source: src/core/SkMesh.cpp:873-878
    unsafe fn update(
        &self,
        dc: *mut GrDirectContext,
        data: *const u8,
        offset: usize,
        size: usize,
    ) -> bool {
        check_update(data, offset, size, self.size()) && self.onUpdate(dc, data, offset, size)
    }
}
// Source: include/core/SkMesh.h:326-326
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    kTriangles,
    kTriangleStrip,
}
// Source: include/core/SkMesh.h:384-405
// Source: src/core/SkMesh.cpp:723-731
pub struct SkMesh {
    fSpec: Option<Arc<SkMeshSpecification>>,
    fVB: Option<Arc<dyn VertexBuffer>>,
    fIB: Option<Arc<dyn IndexBuffer>>,
    fUniforms: Option<Arc<SkData>>,
    fChildren: ManuallyDrop<Pin<Box<STArray<ChildPtr, 2>>>>,
    fVOffset: usize,
    fVCount: usize,
    fIOffset: usize,
    fICount: usize,
    fMode: Mode,
    fBounds: SkRect,
}
impl Default for SkMesh {
    fn default() -> Self {
        Self {
            fSpec: None,
            fVB: None,
            fIB: None,
            fUniforms: None,
            fChildren: ManuallyDrop::new(mesh_children(&[])),
            fVOffset: 0,
            fVCount: 0,
            fIOffset: 0,
            fICount: 0,
            fMode: Mode::kTriangles,
            fBounds: SkRect::MakeEmpty(),
        }
    }
}
// Source: include/core/SkMesh.h:408-408
pub struct MeshResult {
    pub mesh: SkMesh,
    pub error: SkString,
}
impl SkMesh {
    // Source: src/core/SkMesh.cpp:694-716
    pub fn Make(
        spec: Option<Arc<SkMeshSpecification>>,
        mode: Mode,
        vb: Option<Arc<dyn VertexBuffer>>,
        vertexCount: usize,
        vertexOffset: usize,
        uniforms: Option<Arc<SkData>>,
        children: &[ChildPtr],
        bounds: &SkRect,
    ) -> MeshResult {
        let mut mesh = Self {
            fSpec: spec,
            fMode: mode,
            fVB: vb,
            fUniforms: uniforms,
            fChildren: ManuallyDrop::new(mesh_children(children)),
            fVCount: vertexCount,
            fVOffset: vertexOffset,
            fBounds: *bounds,
            fIB: None,
            fIOffset: 0,
            fICount: 0,
        };
        let (valid, error) = unsafe { mesh.validate() };
        if !valid {
            mesh = Self::default();
        }
        MeshResult { mesh, error }
    }
    // Source: src/core/SkMesh.cpp:718-751
    pub fn MakeIndexed(
        spec: Option<Arc<SkMeshSpecification>>,
        mode: Mode,
        vb: Option<Arc<dyn VertexBuffer>>,
        vertexCount: usize,
        vertexOffset: usize,
        ib: Option<Arc<dyn IndexBuffer>>,
        indexCount: usize,
        indexOffset: usize,
        uniforms: Option<Arc<SkData>>,
        children: &[ChildPtr],
        bounds: &SkRect,
    ) -> MeshResult {
        if ib.is_none() {
            return MeshResult {
                mesh: Self::default(),
                error: string(b"An index buffer is required."),
            };
        }
        let mut mesh = Self {
            fSpec: spec,
            fMode: mode,
            fVB: vb,
            fVCount: vertexCount,
            fVOffset: vertexOffset,
            fIB: ib,
            fUniforms: uniforms,
            fChildren: ManuallyDrop::new(mesh_children(children)),
            fICount: indexCount,
            fIOffset: indexOffset,
            fBounds: *bounds,
        };
        let (valid, error) = unsafe { mesh.validate() };
        if !valid {
            mesh = Self::default();
        }
        MeshResult { mesh, error }
    }
    // Source: include/core/SkMesh.h:365-379
    pub fn refSpec(&self) -> Option<Arc<SkMeshSpecification>> {
        self.fSpec.clone()
    }
    pub fn spec(&self) -> Option<&SkMeshSpecification> {
        self.fSpec.as_deref()
    }
    pub fn mode(&self) -> Mode {
        self.fMode
    }
    pub fn refVertexBuffer(&self) -> Option<Arc<dyn VertexBuffer>> {
        self.fVB.clone()
    }
    pub fn vertexBuffer(&self) -> Option<&dyn VertexBuffer> {
        self.fVB.as_deref()
    }
    pub fn vertexOffset(&self) -> usize {
        self.fVOffset
    }
    pub fn vertexCount(&self) -> usize {
        self.fVCount
    }
    pub fn refIndexBuffer(&self) -> Option<Arc<dyn IndexBuffer>> {
        self.fIB.clone()
    }
    pub fn indexBuffer(&self) -> Option<&dyn IndexBuffer> {
        self.fIB.as_deref()
    }
    pub fn indexOffset(&self) -> usize {
        self.fIOffset
    }
    pub fn indexCount(&self) -> usize {
        self.fICount
    }
    pub fn refUniforms(&self) -> Option<Arc<SkData>> {
        self.fUniforms.clone()
    }
    pub fn uniforms(&self) -> Option<&SkData> {
        self.fUniforms.as_deref()
    }
    pub fn children(&self) -> &[ChildPtr] {
        self.fChildren.as_slice()
    }
    pub fn bounds(&self) -> SkRect {
        self.fBounds
    }
    // Source: src/core/SkMesh.cpp:753-757
    pub fn isValid(&self) -> bool {
        let valid = self.fSpec.is_some();
        crate::SkASSERT!(@source b"src/core/SkMesh.cpp\0".as_ptr(), 755; @condition b"valid == std::get<0>(this->validate())\0".as_ptr(); (valid) == (unsafe{self.validate().0}));
        valid
    }
    // Source: src/core/SkMesh.cpp:767-860
    unsafe fn validate(&self) -> (bool, SkString) {
        macro_rules! fail{($fmt:literal $(,$arg:expr)* $(,)?)=>{return(false,unsafe{super::SkString::SkStringPrintf(concat!($fmt,"\0").as_ptr() $(,$arg)*)})};}
        let Some(spec) = self.fSpec.as_ref() else {
            fail!("SkMeshSpecification is required.");
        };
        let Some(vb) = self.fVB.as_ref() else {
            fail!("A vertex buffer is required.");
        };
        if spec.children().len() != self.fChildren.as_slice().len() {
            fail!(
                "The mesh specification declares %zu child effects, but the mesh supplies %d.",
                spec.children().len(),
                self.fChildren.as_slice().len() as core::ffi::c_int
            );
        }
        for (index, child) in self.fChildren.as_slice().iter().enumerate() {
            let expected = &spec.children()[index];
            if let Some(actual) = child.r#type() {
                if expected.r#type != actual {
                    fail!(
                        "Child effect '%.*s' was specified as a %s, but passed as a %s.",
                        expected.name.len() as core::ffi::c_int,
                        expected.name.as_ptr(),
                        re::ChildTypeToStrCStr(expected.r#type).as_ptr(),
                        re::ChildTypeToStrCStr(actual).as_ptr()
                    );
                }
            }
        }
        let mut sm = super::SkSafeMath::SkSafeMath::new();
        let vsize = sm.mul(spec.stride(), self.fVCount);
        if sm.add(vsize, self.fVOffset) > vb.size() {
            fail!("The vertex buffer offset and vertex count reads beyond the end of the vertex buffer.");
        }
        if self.fVOffset % spec.stride() != 0 {
            fail!(
                "The vertex offset (%zu) must be a multiple of the vertex stride (%zu).",
                self.fVOffset,
                spec.stride()
            );
        }
        let uniformSize = spec.uniformSize();
        if uniformSize != 0 {
            if self
                .fUniforms
                .as_ref()
                .is_none_or(|u| u.size() < uniformSize)
            {
                // Native error formatting dereferences fUniforms even on the null branch.
                // Do not invent a zero byte size in that native undefined case.
                let actual = self
                    .fUniforms
                    .as_ref()
                    .expect("native null uniform-data dereference in error formatting")
                    .size();
                fail!(
                    "The uniform data is %zu bytes but must be at least %zu.",
                    actual,
                    uniformSize
                );
            }
        }
        if let Some(ib) = self.fIB.as_ref() {
            if self.fICount < min_vcount_for_mode(self.fMode) {
                fail!(
                    "%s mode requires at least %zu indices but index count is %zu.",
                    mode_to_str(self.fMode).as_ptr(),
                    min_vcount_for_mode(self.fMode),
                    self.fICount
                );
            }
            let isize = sm.mul(2, self.fICount);
            if sm.add(isize, self.fIOffset) > ib.size() {
                fail!("The index buffer offset and index count reads beyond the end of the index buffer.");
            }
            if self.fIOffset & 1 != 0 {
                fail!("The index offset must be a multiple of 2.");
            }
        } else {
            if self.fVCount < min_vcount_for_mode(self.fMode) {
                fail!(
                    "%s mode requires at least %zu vertices but vertex count is %zu.",
                    mode_to_str(self.fMode).as_ptr(),
                    min_vcount_for_mode(self.fMode),
                    self.fICount
                );
            }
            crate::SkASSERT!(@source b"src/core/SkMesh.cpp\0".as_ptr(), 851; @condition b"!fICount\0".as_ptr(); (self.fICount) == (0));
            crate::SkASSERT!(@source b"src/core/SkMesh.cpp\0".as_ptr(), 852; @condition b"!fIOffset\0".as_ptr(); (self.fIOffset) == (0));
        }
        if !sm.ok() {
            fail!("Overflow");
        }
        (true, SkString::default())
    }
}
// Source: src/core/SkMesh.cpp:759-765
fn min_vcount_for_mode(mode: Mode) -> usize {
    match mode {
        Mode::kTriangles | Mode::kTriangleStrip => 3,
    }
}
// Source: src/core/SkMesh.cpp:826-832
fn mode_to_str(mode: Mode) -> &'static core::ffi::CStr {
    match mode {
        Mode::kTriangles => c"triangles",
        Mode::kTriangleStrip => c"triangle-strip",
    }
}
// Source: src/core/SkMesh.cpp:864-872
fn check_update(data: *const u8, offset: usize, size: usize, bufferSize: usize) -> bool {
    let mut sm = super::SkSafeMath::SkSafeMath::new();
    !data.is_null()
        && size != 0
        && offset & 3 == 0
        && size & 3 == 0
        && sm.add(offset, size) <= bufferSize
        && sm.ok()
}
// Source: src/core/SkMesh.cpp:905-907
pub mod SkMeshes {
    use super::*;
    pub unsafe fn MakeIndexBuffer(data: *const u8, size: usize) -> Arc<dyn IndexBuffer> {
        Arc::new(super::super::SkMeshPriv::CpuIndexBuffer::Make(data, size))
    }
    pub unsafe fn CopyIndexBuffer(
        src: Option<&Arc<dyn IndexBuffer>>,
    ) -> Option<Arc<dyn IndexBuffer>> {
        let src = src?;
        let data = src.peek();
        if data.is_null() {
            None
        } else {
            Some(MakeIndexBuffer(data, src.size()))
        }
    }
    pub unsafe fn MakeVertexBuffer(data: *const u8, size: usize) -> Arc<dyn VertexBuffer> {
        Arc::new(super::super::SkMeshPriv::CpuVertexBuffer::Make(data, size))
    }
    pub unsafe fn CopyVertexBuffer(
        src: Option<&Arc<dyn VertexBuffer>>,
    ) -> Option<Arc<dyn VertexBuffer>> {
        let src = src?;
        let data = src.peek();
        if data.is_null() {
            None
        } else {
            Some(MakeVertexBuffer(data, src.size()))
        }
    }
}

// Source: src/core/SkMesh.cpp:626-626
// Native default destructors destroy members in reverse declaration order.
impl Drop for SkMeshSpecification {
    fn drop(&mut self) {
        self.fColorSpace.take();
        unsafe {
            core::mem::ManuallyDrop::drop(&mut self.fFS);
            core::mem::ManuallyDrop::drop(&mut self.fVS);
        }
        self.fChildren.clear();
        self.fUniforms.clear();
        self.fVaryings.clear();
        self.fAttributes.clear();
    }
}
// Source: src/core/SkMesh.cpp:724-724
impl Drop for SkMesh {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.fChildren);
        }
        self.fUniforms.take();
        self.fIB.take();
        self.fVB.take();
        self.fSpec.take();
    }
}

// Stable structural storage for the actual self-relative native STArray. Moving
// a SkMesh value moves its owner handle, never invalidates inline child slots.
fn mesh_children(children: &[ChildPtr]) -> Pin<Box<STArray<ChildPtr, 2>>> {
    let mut values = Box::pin(STArray::new());
    values.as_mut().init();
    let mut iter = children.iter();
    values.as_mut().push_back_n_with(children.len() as i32, || {
        iter.next().expect("native child count").clone()
    });
    values
}
// Source: src/core/SkMesh.cpp:726-726
impl Clone for SkMesh {
    fn clone(&self) -> Self {
        Self {
            fSpec: self.fSpec.clone(),
            fVB: self.fVB.clone(),
            fIB: self.fIB.clone(),
            fUniforms: self.fUniforms.clone(),
            fChildren: ManuallyDrop::new(mesh_children(self.children())),
            fVOffset: self.fVOffset,
            fVCount: self.fVCount,
            fIOffset: self.fIOffset,
            fICount: self.fICount,
            fMode: self.fMode,
            fBounds: self.fBounds,
        }
    }
}
