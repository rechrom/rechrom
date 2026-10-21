// Copyright 2020 Google LLC. BSD-style license.
// Real compiled-effect owner, reflection, Compiler factories and CPU RP execution.
// Unverified translation; native source lifetimes are retained by the base Program.
// Source: include/effects/SkRuntimeEffect.h:63-75
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UniformType {
    kFloat,
    kFloat2,
    kFloat3,
    kFloat4,
    kFloat2x2,
    kFloat3x3,
    kFloat4x4,
    kInt,
    kInt2,
    kInt3,
    kInt4,
}
// Source: include/effects/SkRuntimeEffect.h:77-97
pub const kArray_Flag: u32 = 0x1;
pub const kColor_Flag: u32 = 0x2;
pub const kVertex_Flag: u32 = 0x4;
pub const kFragment_Flag: u32 = 0x8;
pub const kHalfPrecision_Flag: u32 = 0x10;
// Source: include/effects/SkRuntimeEffect.h:99-103
pub struct Uniform<'a> {
    pub name: &'a [u8],
    pub offset: usize,
    pub r#type: UniformType,
    pub count: i32,
    pub flags: u32,
}
// Source: include/effects/SkRuntimeEffect.h:105-106
impl Uniform<'_> {
    pub fn isArray(&self) -> bool {
        self.flags & kArray_Flag != 0
    }
    pub fn isColor(&self) -> bool {
        self.flags & kColor_Flag != 0
    }
}
// Source: include/effects/SkRuntimeEffect.h:111-115
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChildType {
    kShader,
    kColorFilter,
    kBlender,
}
// Source: include/effects/SkRuntimeEffect.h:117-121
pub struct Child<'a> {
    pub name: &'a [u8],
    pub r#type: ChildType,
    pub index: i32,
}
// Source: include/effects/SkRuntimeEffect.h:123-148
#[repr(C)]
pub struct Options<'a> {
    pub forceUnoptimized: bool,
    pub fName: &'a [u8],
    pub(crate) allowPrivateAccess: bool,
    pub(crate) fStableKey: u32,
    pub(crate) maxVersionAllowed: crate::include::sksl::SkSLVersion::Version,
}
impl Default for Options<'_> {
    fn default() -> Self {
        Self {
            forceUnoptimized: false,
            fName: &[],
            allowPrivateAccess: false,
            fStableKey: 0,
            maxVersionAllowed: crate::include::sksl::SkSLVersion::Version::k100,
        }
    }
}
// Source: include/effects/SkRuntimeEffect.h:150-155
// Public named fields retain their original move-out API. Rust representation
// order supplies native reverse destruction without a Drop impl or extra owner;
// named producer initializers keep native effect-then-error evaluation order.
pub struct EffectResult {
    // Source: include/effects/SkRuntimeEffect.h:154
    pub errorText: Vec<u8>,
    // Source: include/effects/SkRuntimeEffect.h:153
    pub effect: Option<std::sync::Arc<SkRuntimeEffect>>,
}
impl SkRuntimeEffect {
    // Source: include/effects/SkRuntimeEffect.h:169-171
    pub fn MakeForColorFilterDefault(sksl: Vec<u8>) -> EffectResult {
        Self::MakeForColorFilter(sksl, &Options::default())
    }
    // Source: include/effects/SkRuntimeEffect.h:177-179
    pub fn MakeForShaderDefault(sksl: Vec<u8>) -> EffectResult {
        Self::MakeForShader(sksl, &Options::default())
    }
    // Source: include/effects/SkRuntimeEffect.h:184-186
    pub fn MakeForBlenderDefault(sksl: Vec<u8>) -> EffectResult {
        Self::MakeForBlender(sksl, &Options::default())
    }
}
// Source: include/effects/SkRuntimeEffect.h:189-195
// Source: include/effects/SkRuntimeEffect.h:210-210
// Adapted: a typed sum retains each actual owner without an unsound downcast from
// an erased Rust trait object. Shader/filter use their existing Arc owners;
// Blender uses its existing native intrusive sk_sp. Generic serialization projects
// the same canonical FlattenableOwner variants; no proxy/counter is introduced.
#[derive(Clone, Default)]
pub enum ChildPtr {
    #[default]
    Empty,
    Shader(std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>),
    ColorFilter(std::sync::Arc<dyn crate::src::core::SkColorFilter::SkColorFilter>),
    Blender(crate::include::core::SkRefCnt::sk_sp<dyn crate::include::core::SkBlender::SkBlender>),
}
// Source: include/effects/SkRuntimeEffect.h:205-205
impl ChildPtr {
    /// Borrow the exact original child's native flattenable virtual protocol.
    pub fn flattenable(&self) -> Option<&dyn crate::src::core::SkFlattenable::SkFlattenableView> {
        match self {
            Self::Empty => None,
            Self::Shader(child) => Some(&**child),
            Self::ColorFilter(child) => Some(&**child),
            Self::Blender(child) => child
                .get()
                .map(|child| child as &dyn crate::src::core::SkFlattenable::SkFlattenableView),
        }
    }
}
// Source: include/effects/SkRuntimeEffect.h:245-248
pub struct TracedShader {
    pub shader: Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>,
    pub debugTrace:
        Option<std::sync::Arc<crate::src::sksl::tracing::SkSLDebugTracePriv::DebugTracePriv>>,
}
// Source: include/effects/SkRuntimeEffect.h:245-248
impl Drop for TracedShader {
    fn drop(&mut self) {
        // Native aggregate members release in reverse order. Move the same
        // Option owners, without retaining them; tuple cleanup keeps shader
        // teardown after trace release even if a Rust destructor unwinds.
        let owners = (self.debugTrace.take(), self.shader.take());
        drop(owners);
    }
}
// Source: include/effects/SkRuntimeEffect.h:258-270
impl SkRuntimeEffect {
    pub fn uniforms(&self) -> &[Uniform<'_>] {
        &self.fUniforms
    }
    pub fn children(&self) -> &[Child<'_>] {
        &self.fChildren
    }
    pub fn allowShader(&self) -> bool {
        self.fFlags & kAllowShader_Flag != 0
    }
    pub fn allowColorFilter(&self) -> bool {
        self.fFlags & kAllowColorFilter_Flag != 0
    }
    pub fn allowBlender(&self) -> bool {
        self.fFlags & kAllowBlender_Flag != 0
    }
}
// Source: include/effects/SkRuntimeEffect.h:276-286
const kUsesSampleCoords_Flag: u32 = 0x001;
const kAllowColorFilter_Flag: u32 = 0x002;
const kAllowShader_Flag: u32 = 0x004;
const kAllowBlender_Flag: u32 = 0x008;
const kSamplesOutsideMain_Flag: u32 = 0x010;
const kUsesColorTransform_Flag: u32 = 0x020;
const kAlwaysOpaque_Flag: u32 = 0x040;
const kAlphaUnchanged_Flag: u32 = 0x080;
const kDisableOptimization_Flag: u32 = 0x100;
// Source: include/effects/SkRuntimeEffect.h:307-312
impl SkRuntimeEffect {
    pub fn hash(&self) -> u32 {
        self.fHash
    }
    pub fn usesSampleCoords(&self) -> bool {
        self.fFlags & kUsesSampleCoords_Flag != 0
    }
    pub fn samplesOutsideMain(&self) -> bool {
        self.fFlags & kSamplesOutsideMain_Flag != 0
    }
    pub fn usesColorTransform(&self) -> bool {
        self.fFlags & kUsesColorTransform_Flag != 0
    }
    pub fn alwaysOpaque(&self) -> bool {
        self.fFlags & kAlwaysOpaque_Flag != 0
    }
    pub fn isAlphaUnchanged(&self) -> bool {
        self.fFlags & kAlphaUnchanged_Flag != 0
    }
}
// Source: include/effects/SkRuntimeEffect.h:321-335
// Reflection names and main are private raw borrows into fBaseProgram's stable IR.
// Public observers shorten every name borrow to &self, never export 'static data.
// Adapted private Rust field order expresses the native default destructor's
// reverse member teardown. This Rust struct has no native ABI/layout claim.
// Named-field constructor expressions retain their original evaluation order.
pub struct SkRuntimeEffect {
    // Source: include/effects/SkRuntimeEffect.h:335-335
    fFlags: u32,
    // Source: include/effects/SkRuntimeEffect.h:333-333
    pub(crate) fSampleUsages: Vec<crate::src::sksl::SkSLSampleUsage::SampleUsage>,
    // Source: include/effects/SkRuntimeEffect.h:332-332
    fChildren: Vec<Child<'static>>,
    // Source: include/effects/SkRuntimeEffect.h:331-331
    fUniforms: Vec<Uniform<'static>>,
    // Source: include/effects/SkRuntimeEffect.h:330-330
    pub(crate) fMain:
        core::ptr::NonNull<crate::src::sksl::ir::SkSLFunctionDefinition::FunctionDefinition>,
    // Source: include/effects/SkRuntimeEffect.h:329-329
    fCompileRPProgramOnce: crate::include::private::SkOnce::SkOnce,
    // Source: include/effects/SkRuntimeEffect.h:328-328
    fRPProgram: core::cell::UnsafeCell<Option<CachedRPProgram>>,
    // Source: include/effects/SkRuntimeEffect.h:327-327
    pub(crate) fBaseProgram: Box<crate::src::sksl::ir::SkSLProgram::Program>,
    // Source: include/effects/SkRuntimeEffect.h:325-325
    pub(crate) fName: Vec<u8>,
    // Source: include/effects/SkRuntimeEffect.h:324-324
    pub(crate) fStableKey: u32,
    // Source: include/effects/SkRuntimeEffect.h:322-322
    pub(crate) fHash: u32,
}
// The same actual trace allocation stays alive through RP Program destruction.
// No evaluator or second compiler owner is introduced by this lifetime adapter.
struct CachedRPProgram {
    program: Option<Box<crate::src::sksl::codegen::SkSLRasterPipelineBuilder::Program>>,
    trace: Option<std::sync::Arc<crate::src::sksl::tracing::SkSLDebugTracePriv::DebugTracePriv>>,
}
// Native shared const Effect publishes its immutable base IR; only SkOnce
// mutates compiled state. Trace generation obeys the native exclusive trace lease.
// Native const Program graph has no safe mutation projection. TraceHook is
// Send+Sync under its real mutable-Vec lease; all RP execution is unsafe and
// excludes concurrent execution/readback of the same trace. SkOnce serializes
// publication; held Arc keeps the original trace allocation alive.
unsafe impl Send for SkRuntimeEffect {}
unsafe impl Sync for SkRuntimeEffect {}
// Source: include/effects/SkRuntimeEffect.h:364-367
// UNVERIFIED production constructor: actual effect ownership, allocated zeroed
// uniform block and exactly one native-empty ChildPtr per reflected child.
impl SkRuntimeEffectBuilder {
    pub fn new(effect: std::sync::Arc<SkRuntimeEffect>) -> Self {
        let uniforms = super::SkData::SkData::MakeZeroInitialized(effect.uniformSize());
        let children = vec![ChildPtr::Empty; effect.children().len()];
        Self {
            fEffect: core::mem::ManuallyDrop::new(effect),
            fUniforms: core::mem::ManuallyDrop::new(uniforms),
            fChildren: children,
        }
    }
    // Source: include/effects/SkRuntimeEffect.h:368-371
    pub fn with_uniforms(
        effect: std::sync::Arc<SkRuntimeEffect>,
        uniforms: std::sync::Arc<super::SkData::SkData>,
    ) -> Self {
        let children = vec![ChildPtr::Empty; effect.children().len()];
        Self {
            fEffect: core::mem::ManuallyDrop::new(effect),
            fUniforms: core::mem::ManuallyDrop::new(uniforms),
            fChildren: children,
        }
    }
}

// Source: include/effects/SkRuntimeEffect.h:377-430
// Borrowed projection of the original builder's fUniforms and reflected fVar.
// This is not a constructible RuntimeEffectBuilder or a compiled-effect owner.
pub struct BuilderUniform<'owner, 'var, 'name> {
    fOwner: &'owner mut std::sync::Arc<crate::src::core::SkData::SkData>,
    fVar: Option<&'var Uniform<'name>>,
}
impl<'owner, 'var, 'name> BuilderUniform<'owner, 'var, 'name> {
    /// Project the actual original uniform field and its reflected metadata.
    /// Metadata must describe that field's real allocated uniform data block.
    pub fn project(
        owner: &'owner mut std::sync::Arc<crate::src::core::SkData::SkData>,
        var: Option<&'var Uniform<'name>>,
    ) -> Self {
        Self {
            fOwner: owner,
            fVar: var,
        }
    }
    // Source: include/effects/SkRuntimeEffect.h:383-395
    /// T has the original native trivial-copy layout and initialized object
    /// representation for every copied uniform byte. Metadata offset/size fit
    /// the data block. No raw alias or Weak upgrade races the exclusive write.
    pub unsafe fn assign<T: Copy>(&mut self, val: &T) -> &mut Self {
        if let Some(var) = self.fVar {
            if core::mem::size_of::<T>() != var.sizeInBytes() {
                crate::SkDEBUGFAIL!(@source b"include/effects/SkRuntimeEffect.h\0".as_ptr(),389; b"Incorrect value size\0".as_ptr());
            } else {
                let data = unsafe { writableUniformData(self.fOwner) };
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        (val as *const T).cast::<u8>(),
                        data.add(var.offset),
                        core::mem::size_of::<T>(),
                    );
                }
            }
        } else {
            crate::SkDEBUGFAIL!(@source b"include/effects/SkRuntimeEffect.h\0".as_ptr(),387; b"Assigning to missing variable\0".as_ptr());
        }
        self
    }
    // Source: include/effects/SkRuntimeEffect.h:397-410
    /// Reflected offset addresses aligned writable floats in the original data
    /// block; preserve assign's exclusive COW/write and aliasing contract.
    pub unsafe fn assign_matrix(
        &mut self,
        val: &crate::src::core::SkMatrix::SkMatrix,
    ) -> &mut Self {
        if let Some(var) = self.fVar {
            if var.sizeInBytes() != 9 * core::mem::size_of::<f32>() {
                crate::SkDEBUGFAIL!(@source b"include/effects/SkRuntimeEffect.h\0".as_ptr(),401; b"Incorrect value size\0".as_ptr());
            } else {
                let data = unsafe {
                    writableUniformData(self.fOwner)
                        .add(var.offset)
                        .cast::<f32>()
                };
                unsafe {
                    data.add(0).write(val.get(0));
                    data.add(1).write(val.get(3));
                    data.add(2).write(val.get(6));
                    data.add(3).write(val.get(1));
                    data.add(4).write(val.get(4));
                    data.add(5).write(val.get(7));
                    data.add(6).write(val.get(2));
                    data.add(7).write(val.get(5));
                    data.add(8).write(val.get(8));
                }
            }
        } else {
            crate::SkDEBUGFAIL!(@source b"include/effects/SkRuntimeEffect.h\0".as_ptr(),399; b"Assigning to missing variable\0".as_ptr());
        }
        self
    }
    // Source: include/effects/SkRuntimeEffect.h:412-426
    /// val supplies count readable trivial-copy objects if the native size
    /// equality succeeds. All copied bytes are initialized; metadata addresses
    /// the actual original uniform block and no alias races the resulting write.
    pub unsafe fn set<T: Copy>(&mut self, val: *const T, count: i32) -> bool {
        let Some(var) = self.fVar else {
            crate::SkDEBUGFAIL!(@source b"include/effects/SkRuntimeEffect.h\0".as_ptr(),416; b"Assigning to missing variable\0".as_ptr());
            return false;
        };
        // Native sizeof(T) promotes signed count to size_t before multiplication.
        let bytes = core::mem::size_of::<T>().wrapping_mul(count as usize);
        if bytes != var.sizeInBytes() {
            crate::SkDEBUGFAIL!(@source b"include/effects/SkRuntimeEffect.h\0".as_ptr(),419; b"Incorrect value size\0".as_ptr());
            return false;
        }
        let data = unsafe { writableUniformData(self.fOwner) };
        if bytes != 0 {
            unsafe {
                core::ptr::copy_nonoverlapping(val.cast::<u8>(), data.add(var.offset), bytes);
            }
        }
        true
    }
}
// Source: include/effects/SkRuntimeEffect.h:432-454
// The slice is the original fChildren member; replacement writes to its actual
// slot before releasing the previous retained child, matching native sk_sp.
pub struct BuilderChild<'owner, 'child, 'name> {
    fOwner: &'owner mut [ChildPtr],
    fChild: Option<&'child Child<'name>>,
}
impl<'owner, 'child, 'name> BuilderChild<'owner, 'child, 'name> {
    pub fn project(owner: &'owner mut [ChildPtr], child: Option<&'child Child<'name>>) -> Self {
        Self {
            fOwner: owner,
            fChild: child,
        }
    }
    // Source: include/effects/SkRuntimeEffect.h:433-441
    // ChildPtr is the existing Shader/ColorFilter/Blender retained-owner sum.
    // No child-type check is added here: native validates at make-time.
    pub fn assign(&mut self, val: ChildPtr) -> &mut Self {
        if let Some(child) = self.fChild {
            drop(core::mem::replace(
                &mut self.fOwner[child.index as usize],
                val,
            ));
        } else {
            crate::SkDEBUGFAIL!(@source b"include/effects/SkRuntimeEffect.h\0".as_ptr(),435; b"Assigning to missing child\0".as_ptr());
        }
        self
    }
    // Source: include/effects/SkRuntimeEffect.h:443-450
    pub fn assign_null(&mut self) -> &mut Self {
        if let Some(child) = self.fChild {
            drop(core::mem::replace(
                &mut self.fOwner[child.index as usize],
                ChildPtr::default(),
            ));
        } else {
            crate::SkDEBUGFAIL!(@source b"include/effects/SkRuntimeEffect.h\0".as_ptr(),445; b"Assigning to missing child\0".as_ptr());
        }
        self
    }
}
// Source: include/effects/SkRuntimeEffect.h:456-466
impl SkRuntimeEffectBuilder {
    pub fn effect(&self) -> &SkRuntimeEffect {
        &self.fEffect
    }
    pub fn uniform(&mut self, name: &[u8]) -> BuilderUniform<'_, '_, '_> {
        BuilderUniform::project(&mut self.fUniforms, self.fEffect.findUniform(name))
    }
    pub fn child(&mut self, name: &[u8]) -> BuilderChild<'_, '_, '_> {
        BuilderChild::project(&mut self.fChildren, self.fEffect.findChild(name))
    }
    pub fn uniforms(&self) -> std::sync::Arc<super::SkData::SkData> {
        std::sync::Arc::clone(&self.fUniforms)
    }
    pub fn children(&self) -> &[ChildPtr] {
        &self.fChildren
    }
}
// Source: include/effects/SkRuntimeEffect.h:482-487
/// This is the actual builder uniform Arc field, exclusively borrowed through
/// the full write. Existing raw aliases may not observe mutation and no Weak
/// upgrade/new retained owner may race uniqueness-check-to-write. The returned
/// pointer cannot outlive or conflict with that original field's write lease.
pub unsafe fn writableUniformData(
    uniforms: &mut std::sync::Arc<crate::src::core::SkData::SkData>,
) -> *mut u8 {
    if std::sync::Arc::strong_count(uniforms) != 1 {
        let copy = unsafe {
            crate::src::core::SkData::SkData::MakeWithCopy(uniforms.data(), uniforms.size())
        };
        drop(core::mem::replace(uniforms, copy));
    }
    uniforms.writable_data()
}

// Source: include/effects/SkRuntimeEffect.h:482-487
impl SkRuntimeEffectBuilder {
    pub unsafe fn writableUniformData(&mut self) -> *mut u8 {
        writableUniformData(&mut self.fUniforms)
    }
}
// Source: include/effects/SkRuntimeEffect.h:489-491
#[derive(Clone)]
pub struct SkRuntimeEffectBuilder {
    fEffect: core::mem::ManuallyDrop<std::sync::Arc<SkRuntimeEffect>>,
    fUniforms: core::mem::ManuallyDrop<std::sync::Arc<super::SkData::SkData>>,
    fChildren: Vec<ChildPtr>,
}
// Source: include/effects/SkRuntimeEffect.h:471
// Source: include/effects/SkRuntimeEffect.h:489-491
// Native reverse member destruction: children, uniform block, compiled effect.
impl Drop for SkRuntimeEffectBuilder {
    fn drop(&mut self) {
        // Same native h489-491 order as runtime instances: vector values and
        // allocation, uniform block, effect. Tuple guards also clean later owners
        // if a Rust child destructor unwinds; fields remain in source order.
        struct Owner<'a, T>(&'a mut core::mem::ManuallyDrop<T>);
        impl<T> Drop for Owner<'_, T> {
            fn drop(&mut self) {
                unsafe {
                    core::mem::ManuallyDrop::drop(self.0);
                }
            }
        }
        let owners = (
            RuntimeChildren(core::mem::take(&mut self.fChildren)),
            Owner(&mut self.fUniforms),
            Owner(&mut self.fEffect),
        );
        drop(owners);
    }
}
// Source: src/core/SkRuntimeEffect.cpp:71-71
const kRPEnableLiveTrace: bool = false;
// Source: src/core/SkRuntimeEffect.cpp:75-100
// Adapted: return the initialized variant instead of writing an uninitialized Uniform.
unsafe fn init_uniform_type(
    ctx: &crate::src::sksl::SkSLContext::Context,
    ty: &dyn crate::src::sksl::ir::SkSLType::Type,
) -> Option<UniformType> {
    let t = unsafe { ctx.fTypes.as_ref() };
    if ty.matches(&**t.fFloat) {
        return Some(UniformType::kFloat);
    }
    if ty.matches(&**t.fHalf) {
        return Some(UniformType::kFloat);
    }
    if ty.matches(&**t.fFloat2) {
        return Some(UniformType::kFloat2);
    }
    if ty.matches(&**t.fHalf2) {
        return Some(UniformType::kFloat2);
    }
    if ty.matches(&**t.fFloat3) {
        return Some(UniformType::kFloat3);
    }
    if ty.matches(&**t.fHalf3) {
        return Some(UniformType::kFloat3);
    }
    if ty.matches(&**t.fFloat4) {
        return Some(UniformType::kFloat4);
    }
    if ty.matches(&**t.fHalf4) {
        return Some(UniformType::kFloat4);
    }
    if ty.matches(&**t.fFloat2x2) {
        return Some(UniformType::kFloat2x2);
    }
    if ty.matches(&**t.fHalf2x2) {
        return Some(UniformType::kFloat2x2);
    }
    if ty.matches(&**t.fFloat3x3) {
        return Some(UniformType::kFloat3x3);
    }
    if ty.matches(&**t.fHalf3x3) {
        return Some(UniformType::kFloat3x3);
    }
    if ty.matches(&**t.fFloat4x4) {
        return Some(UniformType::kFloat4x4);
    }
    if ty.matches(&**t.fHalf4x4) {
        return Some(UniformType::kFloat4x4);
    }
    if ty.matches(&**t.fInt) {
        return Some(UniformType::kInt);
    }
    if ty.matches(&**t.fInt2) {
        return Some(UniformType::kInt2);
    }
    if ty.matches(&**t.fInt3) {
        return Some(UniformType::kInt3);
    }
    if ty.matches(&**t.fInt4) {
        return Some(UniformType::kInt4);
    }
    None
}
// Source: src/core/SkRuntimeEffect.cpp:102-132
/// Safety: context's canonical builtin type leases must remain live throughout reflection.
pub unsafe fn VarAsUniform<'a>(
    var: &'a dyn crate::src::sksl::ir::SkSLVariable::Variable,
    context: &crate::src::sksl::SkSLContext::Context,
    offset: &mut usize,
) -> Uniform<'a> {
    crate::SkASSERT!(@source b"src/core/SkRuntimeEffect.cpp\0".as_ptr(),106; @condition b"var.modifierFlags().isUniform()\0".as_ptr(); var.modifierFlags().isUniform());
    let mut flags = 0;
    let mut count = 1;
    let mut ty = var.symbol_type();
    if ty.isArray() {
        flags |= kArray_Flag;
        count = ty.columns();
        ty = ty.componentType();
    }
    if ty.hasPrecision() && !ty.highPrecision() {
        flags |= kHalfPrecision_Flag;
    }
    // Native SkAssertResult requires a supported type; never invent a fallback type.
    let reflected_type = unsafe { init_uniform_type(context, ty) }
        .expect("runtime uniform must have a supported reflected type");
    if var.layout().fFlags.0 & crate::src::sksl::ir::SkSLLayout::LayoutFlag::kColor.0 != 0 {
        flags |= kColor_Flag;
    }
    let uni = Uniform {
        name: var.name(),
        offset: *offset,
        r#type: reflected_type,
        count,
        flags,
    };
    *offset = offset.wrapping_add(uni.sizeInBytes());
    crate::SkASSERT!(@source b"src/core/SkRuntimeEffect.cpp\0".as_ptr(),130; @condition b"SkIsAlign4(*offset)\0".as_ptr(); *offset & 3 == 0);
    uni
}
// Source: src/core/SkRuntimeEffect.cpp:134-141
fn child_type(ty: &dyn crate::src::sksl::ir::SkSLType::Type) -> ChildType {
    use crate::src::sksl::ir::SkSLType::TypeKind;
    match ty.typeKind() {
        TypeKind::kBlender => ChildType::kBlender,
        TypeKind::kColorFilter => ChildType::kColorFilter,
        TypeKind::kShader => ChildType::kShader,
        _ => crate::SkUNREACHABLE!(@source b"src/core/SkRuntimeEffect.cpp\0".as_ptr(),139;),
    }
}
// Source: src/core/SkRuntimeEffect.cpp:143-150
pub fn ChildTypeToStr(ty: ChildType) -> &'static str {
    match ty {
        ChildType::kBlender => "blender",
        ChildType::kColorFilter => "color filter",
        ChildType::kShader => "shader",
    }
}
// Source: src/core/SkRuntimeEffect.cpp:143-150
// Native immutable C-string projection; preserves existing Rust string API.
pub fn ChildTypeToStrCStr(ty: ChildType) -> &'static core::ffi::CStr {
    match ty {
        ChildType::kBlender => c"blender",
        ChildType::kColorFilter => c"color filter",
        ChildType::kShader => c"shader",
    }
}

// Source: src/core/SkRuntimeEffect.cpp:152-158
pub fn VarAsChild(var: &dyn crate::src::sksl::ir::SkSLVariable::Variable, index: i32) -> Child<'_> {
    Child {
        name: var.name(),
        r#type: child_type(var.symbol_type()),
        index,
    }
}
// Source: src/core/SkRuntimeEffect.cpp:160-171
/// Safety: reflected uniform offsets/counts describe valid float storage in original_data.
pub unsafe fn TransformUniforms(
    uniforms: &[Uniform<'_>],
    original_data: std::sync::Arc<crate::src::core::SkData::SkData>,
    dst_cs: Option<&crate::src::core::SkColorSpace::SkColorSpace>,
) -> std::sync::Arc<crate::src::core::SkData::SkData> {
    let Some(dst_cs) = dst_cs else {
        return original_data;
    };
    let srgb = crate::src::core::SkColorSpace::SkColorSpace::MakeSRGB();
    let steps = crate::src::core::SkColorSpaceXformSteps::SkColorSpaceXformSteps::new(
        Some(&srgb),
        crate::include::core::SkAlphaType::kUnpremul_SkAlphaType,
        Some(dst_cs),
        crate::include::core::SkAlphaType::kUnpremul_SkAlphaType,
    );
    unsafe { TransformUniformsWithSteps(uniforms, original_data, &steps) }
}
// Source: src/core/SkRuntimeEffect.cpp:173-216
/// Safety: reflected offsets/counts and native alignment must be valid for original_data.
/// The lazily copied Arc is exclusively written before publication.
pub unsafe fn TransformUniformsWithSteps(
    uniforms: &[Uniform<'_>],
    original_data: std::sync::Arc<crate::src::core::SkData::SkData>,
    steps: &crate::src::core::SkColorSpaceXformSteps::SkColorSpaceXformSteps,
) -> std::sync::Arc<crate::src::core::SkData::SkData> {
    use crate::src::core::SkData::SkData;
    let mut data: Option<std::sync::Arc<SkData>> = None;
    for u in uniforms {
        if u.flags & kColor_Flag != 0 {
            crate::SkASSERT!(@source b"src/core/SkRuntimeEffect.cpp\0".as_ptr(),190; @condition b"u.type == Type::kFloat3 || u.type == Type::kFloat4\0".as_ptr(); matches!(u.r#type, UniformType::kFloat3 | UniformType::kFloat4));
            if steps.fFlags.mask() != 0 {
                let writable = data.get_or_insert_with(|| unsafe {
                    SkData::MakeWithCopy(original_data.data(), original_data.size())
                });
                let mut color = unsafe { writable.writable_data().add(u.offset).cast::<f32>() };
                if u.r#type == UniformType::kFloat4 {
                    for _ in 0..u.count {
                        unsafe {
                            steps.apply(&mut *color.cast::<[f32; 4]>());
                            color = color.add(4);
                        }
                    }
                } else {
                    for _ in 0..u.count {
                        let mut rgba = unsafe { [*color, *color.add(1), *color.add(2), 1.0] };
                        steps.apply(&mut rgba);
                        unsafe {
                            std::ptr::copy_nonoverlapping(rgba.as_ptr(), color, 3);
                            color = color.add(3);
                        }
                    }
                }
            }
        }
    }
    data.unwrap_or(original_data)
}
// Source: src/core/SkRuntimeEffect.cpp:218-284
impl SkRuntimeEffect {
    pub unsafe fn getRPProgram(
        &self,
        trace: Option<
            &std::sync::Arc<crate::src::sksl::tracing::SkSLDebugTracePriv::DebugTracePriv>,
        >,
    ) -> Result<
        Option<&crate::src::sksl::codegen::SkSLRasterPipelineBuilder::Program>,
        crate::RasterError,
    > {
        crate::include::private::SkOnce::SkOnce::call_once(
            core::ptr::addr_of!(self.fCompileRPProgramOnce),
            || {
                let mut program = &*self.fBaseProgram;
                let mut main = self.fMain.as_ref();
                let mut optimizedCopy = None;
                let shouldOptimize = self.fFlags & kDisableOptimization_Flag == 0;
                let mut settings = self
                    .fBaseProgram
                    .fConfig
                    .as_ref()
                    .unwrap_unchecked()
                    .fSettings;
                let needsOptimization = !settings.fOptimize
                    || settings.fInlineThreshold
                        < crate::src::sksl::SkSLDefines::kDefaultInlineThreshold;
                if shouldOptimize && needsOptimization {
                    let mut compiler = crate::src::sksl::SkSLCompiler::Compiler::new();
                    settings.fOptimize = true;
                    settings.fInlineThreshold =
                        crate::src::sksl::SkSLDefines::kDefaultInlineThreshold;
                    optimizedCopy = compiler.as_mut().convertProgram(
                        self.fBaseProgram.fConfig.as_ref().unwrap_unchecked().fKind,
                        self.source().to_vec(),
                        &settings,
                    );
                    if let Some(copy) = optimizedCopy.as_ref() {
                        let declaration = copy.getFunction(c"main");
                        crate::SkASSERT!(@source b"src/core/SkRuntimeEffect.cpp\0".as_ptr(),250; @condition b"mainDecl\0".as_ptr(); declaration.is_some());
                        if let Some(decl) = declaration {
                            program = copy;
                            main = decl.definition().unwrap_unchecked();
                        }
                    }
                }
                // Source: src/core/SkRuntimeEffect.cpp:258-269
                // UNVERIFIED: retain the actual trace allocation while RP Program borrows
                // it. Native tempDebugTrace is local; the Rust RP owner retains its raw
                // trace pointer, so the live-trace branch moves that same value into Arc.
                // This extends its lease without inventing a second evaluator or trace.
                let tempDebugTrace =
                    crate::src::sksl::tracing::SkSLDebugTracePriv::DebugTracePriv::new();
                let (compiled, trace) = if let Some(trace) = trace {
                    let trace = std::sync::Arc::clone(trace);
                    let pointer = core::ptr::NonNull::new_unchecked(
                        std::sync::Arc::as_ptr(&trace).cast_mut(),
                    );
                    let compiled=crate::src::sksl::codegen::SkSLRasterPipelineCodeGenerator::MakeRasterPipelineProgram(program,main,Some(pointer),true);
                    (compiled, Some(trace))
                } else if kRPEnableLiveTrace {
                    let trace = std::sync::Arc::new(tempDebugTrace);
                    let pointer = core::ptr::NonNull::new_unchecked(
                        std::sync::Arc::as_ptr(&trace).cast_mut(),
                    );
                    let compiled=crate::src::sksl::codegen::SkSLRasterPipelineCodeGenerator::MakeRasterPipelineProgram(program,main,Some(pointer),false);
                    (compiled, Some(trace))
                } else {
                    let compiled=crate::src::sksl::codegen::SkSLRasterPipelineCodeGenerator::MakeRasterPipelineProgram(program,main,None,false);
                    (compiled, None)
                };
                // Source: src/core/SkRuntimeEffect.cpp:271-280
                if kRPEnableLiveTrace {
                    if let Some(program) = compiled.as_deref() {
                        crate::SkDebugf!(@source b"src/core/SkRuntimeEffect.cpp\0".as_ptr(),273; b"-----\n\n\0".as_ptr());
                        let mut stream =
                            core::pin::pin!(crate::src::core::SkStreamPriv::DebugfStream::new());
                        unsafe {
                            program.dump(stream.as_mut(), true);
                        }
                        crate::SkDebugf!(@source b"src/core/SkRuntimeEffect.cpp\0".as_ptr(),276; b"\n-----\n\n\0".as_ptr());
                    } else {
                        crate::SkDebugf!(@source b"src/core/SkRuntimeEffect.cpp\0".as_ptr(),278; b"----- RP unsupported -----\n\n\0".as_ptr());
                    }
                }
                self.fRPProgram.get().write(Some(CachedRPProgram {
                    program: compiled,
                    trace,
                }));
            },
        );
        let cached = (&*self.fRPProgram.get())
            .as_ref()
            .expect("native completed RP initialization");
        Ok(cached.program.as_deref())
    }
}

// Source: src/core/SkRuntimeEffect.cpp:286-308
/// Returns the native nonowning float span as pointer and element count.
///
/// Safety: uniform offsets/counts must describe initialized, suitably aligned
/// storage, and the byte size must be a multiple of sizeof(float). The caller
/// must retain the original data owner (including external backing storage) or
/// the arena, as selected by the native copy branch, for every span access.
/// Neither backing storage may be invalidated or mutated during shared reads.
pub unsafe fn UniformsAsSpan(
    uniforms: &[Uniform<'_>],
    original_data: std::sync::Arc<crate::src::core::SkData::SkData>,
    always_copy_into_alloc: bool,
    dest_color_space: Option<&crate::src::core::SkColorSpace::SkColorSpace>,
    alloc: &mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
) -> (*const f32, usize) {
    let transformed_data =
        unsafe { TransformUniforms(uniforms, original_data.clone(), dest_color_space) };
    if always_copy_into_alloc || !std::sync::Arc::ptr_eq(&original_data, &transformed_data) {
        let num_bytes = transformed_data.size();
        let num_floats = num_bytes / std::mem::size_of::<f32>();
        let uniforms_in_alloc = alloc.makeArrayDefault::<f32>(num_floats);
        // A native empty span can carry a null data pointer. No bytes are read
        // or written in that case; Rust's copy intrinsic requires nonnull inputs.
        if num_bytes != 0 {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    transformed_data.data(),
                    uniforms_in_alloc.cast::<u8>(),
                    num_bytes,
                );
            }
        }
        return (uniforms_in_alloc.cast_const(), num_floats);
    }
    (
        original_data.data().cast::<f32>(),
        original_data.size() / std::mem::size_of::<f32>(),
    )
}
// Source: src/core/SkRuntimeEffect.cpp:310-324
impl crate::src::core::SkRuntimeEffectPriv::RuntimeEffectRPCallbacks<'_, '_> {
    /// Children and their immutable pixel/context backing outlive every appended
    /// stage. Pipeline/arena are live, stable, exclusively callable raw owners;
    /// index selects the same reflected child and sample usage as native code.
    pub unsafe fn appendShader(&mut self, index: i32) -> Result<bool, crate::RasterError> {
        if let Some(shader) = self.fChildren[index as usize].shader() {
            if self.fSampleUsages[index as usize].isPassThrough() {
                return Ok(unsafe { shader.appendStages(&self.fStage, self.fMatrix) }?);
            }
            let mut nonPassthroughMatrix = self.fMatrix.clone();
            nonPassthroughMatrix.markTotalMatrixInvalid();
            return Ok(unsafe { shader.appendStages(&self.fStage, &nonPassthroughMatrix) }?);
        }
        unsafe {
            (&mut *self.fStage.fPipeline)
                .appendConstantColor(self.fStage.fAlloc, &[0.0, 0.0, 0.0, 0.0]);
        }
        Ok(true)
    }
    // Source: src/core/SkRuntimeEffect.cpp:325-331
    /// Same real pipeline/arena and retained child leases as appendShader.
    pub unsafe fn appendColorFilter(&mut self, index: i32) -> Result<bool, crate::RasterError> {
        if let Some(filter) = self.fChildren[index as usize].colorFilter() {
            return unsafe { filter.appendStages(&self.fStage, false) };
        }
        Ok(true)
    }
    // Source: src/core/SkRuntimeEffect.cpp:332-340
    /// Same real pipeline/arena and retained child leases as appendShader.
    pub unsafe fn appendBlender(&mut self, index: i32) -> Result<bool, crate::RasterError> {
        if let Some(blender) = self.fChildren[index as usize].blender() {
            return unsafe { blender.appendStages(&self.fStage) };
        }
        unsafe {
            (&mut *self.fStage.fPipeline).append(
                crate::src::core::SkRasterPipelineOpList::SkRasterPipelineOp::srcover,
                core::ptr::null_mut(),
            );
        }
        Ok(true)
    }
    // Source: src/core/SkRuntimeEffect.cpp:344-353
    /// color addresses the original writable program color slots retained for
    /// pipeline execution. All stage/arena contexts remain stable until then.
    pub unsafe fn toLinearSrgb(&mut self, color: *const core::ffi::c_void) {
        if let Some(dst) = self.fStage.fDstCS {
            let linear = crate::src::core::SkColorSpace::SkColorSpace::MakeSRGBLinear();
            let xform = crate::src::core::SkColorSpaceXformSteps::SkColorSpaceXformSteps::new(
                Some(dst.as_ref()),
                crate::include::core::SkAlphaType::kUnpremul_SkAlphaType,
                Some(linear.as_ref()),
                crate::include::core::SkAlphaType::kUnpremul_SkAlphaType,
            );
            if xform.fFlags.mask() != 0 {
                unsafe {
                    self.applyColorSpaceXform(&xform, color);
                }
            }
        }
    }
    // Source: src/core/SkRuntimeEffect.cpp:355-364
    /// Same program-color and stage-context leases as toLinearSrgb.
    pub unsafe fn fromLinearSrgb(&mut self, color: *const core::ffi::c_void) {
        if let Some(dst) = self.fStage.fDstCS {
            let linear = crate::src::core::SkColorSpace::SkColorSpace::MakeSRGBLinear();
            let xform = crate::src::core::SkColorSpaceXformSteps::SkColorSpaceXformSteps::new(
                Some(linear.as_ref()),
                crate::include::core::SkAlphaType::kUnpremul_SkAlphaType,
                Some(dst.as_ref()),
                crate::include::core::SkAlphaType::kUnpremul_SkAlphaType,
            );
            if xform.fFlags.mask() != 0 {
                unsafe {
                    self.applyColorSpaceXform(&xform, color);
                }
            }
        }
    }
    // Source: src/core/SkRuntimeEffect.cpp:366-376
    unsafe fn applyColorSpaceXform(
        &mut self,
        temp: &crate::src::core::SkColorSpaceXformSteps::SkColorSpaceXformSteps,
        color: *const core::ffi::c_void,
    ) {
        // Native make<T>(copy) preserves all transform payload in the actual
        // arena. Explicit field-copy is Rust's copy-constructor adaptation;
        // no pointer into the temporary stack transform is retained.
        let copy = crate::src::core::SkColorSpaceXformSteps::SkColorSpaceXformSteps {
            fFlags: temp.fFlags,
            fSrcTF: temp.fSrcTF,
            fDstTFInv: temp.fDstTFInv,
            fSrcToDstMatrix: temp.fSrcToDstMatrix,
            fSrcOotf: temp.fSrcOotf,
            fDstOotf: temp.fDstOotf,
        };
        let xform = unsafe { (&mut *self.fStage.fAlloc).make(copy) };
        let op = crate::src::core::SkRasterPipelineOpList::SkRasterPipelineOp::exchange_src;
        unsafe {
            (&mut *self.fStage.fPipeline).append(op, color.cast_mut());
        }
        unsafe {
            (&*xform).applyPipeline(&mut *self.fStage.fPipeline);
        }
        unsafe {
            (&mut *self.fStage.fPipeline).append(op, color.cast_mut());
        }
    }
}

// Source: src/core/SkRuntimeEffect.cpp:378-382
pub fn CanDrawProgram(
    caps: &crate::include::core::SkCapabilities::SkCapabilities,
    program: &crate::src::sksl::ir::SkSLProgram::Program,
) -> bool {
    let config = program.fConfig.as_ref().expect("native program config");
    crate::SkASSERT!(@source b"src/core/SkRuntimeEffect.cpp\0".as_ptr(),380; @condition b"program->fConfig->enforcesSkSLVersion()\0".as_ptr(); config.enforcesSkSLVersion());
    config.fRequiredSkSLVersion <= caps.skslVersion()
}
// Source: src/core/SkRuntimeEffect.cpp:384-387
pub fn CanDraw(
    caps: &crate::include::core::SkCapabilities::SkCapabilities,
    effect: &super::SkRuntimeEffect::SkRuntimeEffect,
) -> bool {
    CanDrawProgram(caps, &effect.fBaseProgram)
}

// Source: src/core/SkRuntimeEffect.cpp:391-401
fn flattenable_is_valid_as_child(
    child: Option<&dyn crate::src::core::SkFlattenable::SkFlattenableView>,
) -> bool {
    use crate::src::core::SkFlattenable::Type;
    let Some(child) = child else {
        return true;
    };
    match child.getFlattenableType() {
        Type::kSkShader_Type | Type::kSkColorFilter_Type | Type::kSkBlender_Type => true,
        _ => false,
    }
}
// Source: include/effects/SkRuntimeEffect.h:198-198
// Source: src/core/SkRuntimeEffect.cpp:403-405
impl ChildPtr {
    /// A nonnull legal child must use its actual canonical retained-owner variant
    /// from the real factory. Intrusive PathEffect/Drawable/ImageFilter/MaskFilter
    /// owners cannot be reinterpreted as Shader/ColorFilter/Blender trait pointers.
    pub fn from_flattenable(
        child: Option<crate::src::core::SkFlattenable::FlattenableOwner>,
    ) -> Self {
        use crate::src::core::SkFlattenable::FlattenableOwner;
        crate::SkASSERT!(@source b"src/core/SkRuntimeEffect.cpp\0".as_ptr(),404; @condition b"flattenable_is_valid_as_child(fChild.get())\0".as_ptr(); flattenable_is_valid_as_child(child.as_ref().map(|owner|owner.as_view())));
        match child {
            None => Self::Empty,
            Some(FlattenableOwner::Shader(child)) => Self::Shader(child),
            Some(FlattenableOwner::ColorFilter(child)) => Self::ColorFilter(child),
            Some(FlattenableOwner::Blender(child)) => Self::Blender(child),
            _ => unreachable!("legal runtime child requires its actual canonical owner projection"),
        }
    }
}
// Source: src/core/SkRuntimeEffect.cpp:407-423
pub fn verify_child_effects(reflected: &[Child<'_>], effect_ptrs: &[ChildPtr]) -> bool {
    if reflected.len() != effect_ptrs.len() {
        return false;
    }
    for i in 0..effect_ptrs.len() {
        if let Some(effect_type) = effect_ptrs[i].r#type() {
            if effect_type != reflected[i].r#type {
                return false;
            }
        }
    }
    true
}
// Source: src/core/SkRuntimeEffect.cpp:429-462
// `effect_children` is the genuine borrowed effect.children() metadata projection.
// None skips reflection checks exactly when the original effect pointer is null;
// no compiled effect or substitute Program is constructed here.
pub fn ReadChildEffects(
    buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    effect_children: Option<&[Child<'_>]>,
    children: &mut Vec<ChildPtr>,
) -> Result<bool, crate::RasterError> {
    let childCount = buffer.read32() as usize;
    if let Some(effect) = effect_children {
        if !buffer.validate(childCount == effect.len()) {
            return Ok(false);
        }
    }
    children.clear();
    children.reserve_exact(childCount);
    for _ in 0..childCount {
        let obj = buffer.readRawFlattenable()?;
        if !flattenable_is_valid_as_child(obj.as_ref().map(|owner| owner.as_view())) {
            buffer.validate(false);
            return Ok(false);
        }
        children.push(ChildPtr::from_flattenable(obj));
    }
    if let Some(childInfo) = effect_children {
        crate::SkASSERT!(@source b"src/core/SkRuntimeEffect.cpp\0".as_ptr(),452; @condition b"childInfo.size() == SkToSizeT(children->size())\0".as_ptr(); childInfo.len()==children.len());
        for i in 0..childCount {
            if let Some(ct) = children[i].r#type() {
                if ct != childInfo[i].r#type {
                    buffer.validate(false);
                }
            }
        }
    }
    Ok(buffer.isValid())
}
// Source: src/core/SkRuntimeEffect.cpp:464-470
pub fn WriteChildEffects(
    buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    children: &[ChildPtr],
) -> Result<(), crate::RasterError> {
    buffer.write32(children.len() as i32);
    for child in children {
        buffer.writeFlattenable(child.flattenable())?;
    }

    Ok(())
}
// Source: src/core/SkRuntimeEffect.cpp:472-488
pub fn MakeSettings(
    options: &Options<'_>,
) -> crate::src::sksl::SkSLProgramSettings::ProgramSettings {
    let mut settings = crate::src::sksl::SkSLProgramSettings::ProgramSettings::default();
    settings.fInlineThreshold = 0;
    settings.fForceNoInline = options.forceUnoptimized;
    settings.fOptimize = !options.forceUnoptimized;
    settings.fMaxVersionAllowed = options.maxVersionAllowed;
    settings.fUseMemoryPool = false;
    settings
}
// Source: src/core/SkRuntimeEffect.cpp:494-507
impl SkRuntimeEffect {
    pub fn MakeFromSource(
        sksl: Vec<u8>,
        options: &Options<'_>,
        kind: crate::src::sksl::SkSLProgramKind::ProgramKind,
    ) -> EffectResult {
        unsafe {
            let mut compiler = crate::src::sksl::SkSLCompiler::Compiler::new();
            let settings = MakeSettings(options);
            let Some(program) = compiler.as_mut().convertProgram(kind, sksl, &settings) else {
                return EffectResult {
                    effect: None,
                    errorText: compiler.as_mut().get_unchecked_mut().errorText(true),
                };
            };
            Self::MakeInternal(program, options, kind)
        }
    }
}

// Source: src/core/SkRuntimeEffect.cpp:509-642
impl SkRuntimeEffect {
    pub unsafe fn MakeInternal(
        program: Box<crate::src::sksl::ir::SkSLProgram::Program>,
        options: &Options<'_>,
        kind: crate::src::sksl::SkSLProgramKind::ProgramKind,
    ) -> EffectResult {
        use crate::src::sksl::{
            ir::{
                SkSLIRNode::IRNode,
                SkSLVarDeclarations::{GlobalVarDeclaration, VarDeclaration},
            },
            SkSLProgramKind::ProgramKind as K,
        };
        let compiler = crate::src::sksl::SkSLCompiler::Compiler::new();
        let mut flags = match kind {
            K::kPrivateRuntimeColorFilter | K::kRuntimeColorFilter => {
                if !super::SkRuntimeEffectPriv::CanDrawProgram(
                    crate::include::core::SkCapabilities::SkCapabilities::RasterBackend().as_ref(),
                    &program,
                ) {
                    return EffectResult {
                        effect: None,
                        errorText: b"SkSL color filters must target #version 100".to_vec(),
                    };
                }
                kAllowColorFilter_Flag
            }
            K::kPrivateRuntimeShader | K::kRuntimeShader => kAllowShader_Flag,
            K::kPrivateRuntimeBlender | K::kRuntimeBlender => kAllowBlender_Flag,
            _ => crate::SkUNREACHABLE!(@source b"src/core/SkRuntimeEffect.cpp\0".as_ptr(),536;),
        };
        if options.forceUnoptimized {
            flags |= kDisableOptimization_Flag;
        }
        let Some(main) = program.getFunction(c"main") else {
            return EffectResult {
                effect: None,
                errorText: b"missing 'main' function".to_vec(),
            };
        };
        let coords = main.getMainCoordsParameter();
        let usage = coords.map_or_else(
            crate::src::sksl::analysis::SkSLProgramUsage::VariableCounts::default,
            |v| program.usage().unwrap_unchecked().getVariable(v),
        );
        if usage.fRead != 0 || usage.fWrite != 0 {
            flags |= kUsesSampleCoords_Flag;
        }
        if flags & (kAllowColorFilter_Flag | kAllowBlender_Flag) != 0 {
            crate::SkASSERT!(@source b"src/core/SkRuntimeEffect.cpp\0".as_ptr(),562; @condition b"!(flags & kUsesSampleCoords_Flag)\0".as_ptr(); flags&kUsesSampleCoords_Flag==0);
            crate::SkASSERT!(@source b"src/core/SkRuntimeEffect.cpp\0".as_ptr(),563; @condition b"!SkSL::Analysis::ReferencesFragCoords(*program)\0".as_ptr(); !crate::src::sksl::SkSLAnalysis::ReferencesFragCoords(&program));
        }
        if crate::src::sksl::SkSLAnalysis::CallsSampleOutsideMain(&program) {
            flags |= kSamplesOutsideMain_Flag;
        }
        if flags & kAllowColorFilter_Flag != 0
            && crate::src::sksl::SkSLAnalysis::ReturnsInputAlpha(
                main.definition().unwrap_unchecked(),
                program.usage().unwrap_unchecked(),
            )
        {
            flags |= kAlphaUnchanged_Flag;
        }
        if crate::src::sksl::SkSLAnalysis::CallsColorTransformIntrinsics(&program) {
            flags |= kUsesColorTransform_Flag;
        }
        if crate::src::sksl::SkSLAnalysis::ReturnsOpaqueColor(main.definition().unwrap_unchecked())
        {
            flags |= kAlwaysOpaque_Flag;
        }
        let mut offset = 0;
        let mut uniforms = Vec::new();
        let mut children = Vec::new();
        let mut sampleUsages = Vec::new();
        let mut elided = 0;
        for elem in program.elements() {
            let elem = elem.unwrap_unchecked();
            if elem.node().is::<GlobalVarDeclaration>() {
                let global = elem.node().as_node::<GlobalVarDeclaration>();
                let vd = global.declaration().node().as_node::<VarDeclaration>();
                let var = vd.var().unwrap_unchecked().as_ref();
                if var.symbol_type().isEffectChild() {
                    children.push(core::mem::transmute::<Child<'_>, Child<'static>>(
                        VarAsChild(var, children.len() as i32),
                    ));
                    let sample = crate::src::sksl::SkSLAnalysis::GetSampleUsage(
                        &program,
                        var,
                        usage.fWrite != 0,
                        Some(&mut elided),
                    );
                    sampleUsages.push(if sample.isSampled() {
                        sample
                    } else {
                        crate::src::sksl::SkSLSampleUsage::SampleUsage::PassThrough()
                    });
                } else if var.modifierFlags().isUniform() {
                    uniforms.push(core::mem::transmute::<Uniform<'_>, Uniform<'static>>(
                        VarAsUniform(var, compiler.context(), &mut offset),
                    ));
                }
            }
        }
        if elided == usage.fRead && usage.fWrite == 0 {
            flags &= !kUsesSampleCoords_Flag;
        }
        let main = core::ptr::NonNull::from(main.definition().unwrap_unchecked());
        EffectResult {
            effect: Some(std::sync::Arc::new(Self::new(
                program,
                options,
                main,
                uniforms,
                children,
                sampleUsages,
                flags,
            ))),
            errorText: Vec::new(),
        }
    }
}

// Source: src/core/SkRuntimeEffect.cpp:644-681
impl SkRuntimeEffect {
    pub unsafe fn makeUnoptimizedClone(self: &std::sync::Arc<Self>) -> std::sync::Arc<Self> {
        let mut options = Options::default();
        options.forceUnoptimized = true;
        options.maxVersionAllowed = crate::include::sksl::SkSLVersion::Version::k300;
        options.allowPrivateAccess = true;
        let kind = self.fBaseProgram.fConfig.as_ref().unwrap_unchecked().fKind;
        let mut compiler = crate::src::sksl::SkSLCompiler::Compiler::new();
        let Some(program) =
            compiler
                .as_mut()
                .convertProgram(kind, self.source().to_vec(), &MakeSettings(&options))
        else {
            return self.clone();
        };
        let result = Self::MakeInternal(program, &options, kind);
        if let Some(effect) = result.effect {
            effect
        } else {
            {
                #[cfg(sk_debug)]
                {
                    let mut error_text = result.errorText.clone();
                    error_text.push(0);
                    crate::SkDEBUGFAILF!(@source b"src/core/SkRuntimeEffect.cpp\0".as_ptr(),675; "makeUnoptimizedClone: MakeInternal failed\n%s",error_text.as_ptr());
                }
            };
            self.clone()
        }
    }
}

// Source: src/core/SkRuntimeEffect.cpp:683-689
impl SkRuntimeEffect {
    pub fn MakeForColorFilter(sksl: Vec<u8>, options: &Options<'_>) -> EffectResult {
        let kind = if options.allowPrivateAccess {
            crate::src::sksl::SkSLProgramKind::ProgramKind::kPrivateRuntimeColorFilter
        } else {
            crate::src::sksl::SkSLProgramKind::ProgramKind::kRuntimeColorFilter
        };
        let result = Self::MakeFromSource(sksl, options, kind);
        crate::SkASSERT!(@source b"src/core/SkRuntimeEffect.cpp\0".as_ptr(),687; @condition b"!result.effect || result.effect->allowColorFilter()\0".as_ptr(); result.effect.as_ref().is_none_or(|e|e.allowColorFilter()));
        result
    }
}

// Source: src/core/SkRuntimeEffect.cpp:691-697
impl SkRuntimeEffect {
    pub fn MakeForShader(sksl: Vec<u8>, options: &Options<'_>) -> EffectResult {
        let kind = if options.allowPrivateAccess {
            crate::src::sksl::SkSLProgramKind::ProgramKind::kPrivateRuntimeShader
        } else {
            crate::src::sksl::SkSLProgramKind::ProgramKind::kRuntimeShader
        };
        let result = Self::MakeFromSource(sksl, options, kind);
        crate::SkASSERT!(@source b"src/core/SkRuntimeEffect.cpp\0".as_ptr(),695; @condition b"!result.effect || result.effect->allowShader()\0".as_ptr(); result.effect.as_ref().is_none_or(|e|e.allowShader()));
        result
    }
}

// Source: src/core/SkRuntimeEffect.cpp:699-705
impl SkRuntimeEffect {
    pub fn MakeForBlender(sksl: Vec<u8>, options: &Options<'_>) -> EffectResult {
        let kind = if options.allowPrivateAccess {
            crate::src::sksl::SkSLProgramKind::ProgramKind::kPrivateRuntimeBlender
        } else {
            crate::src::sksl::SkSLProgramKind::ProgramKind::kRuntimeBlender
        };
        let result = Self::MakeFromSource(sksl, options, kind);
        crate::SkASSERT!(@source b"src/core/SkRuntimeEffect.cpp\0".as_ptr(),703; @condition b"!result.effect || result.effect->allowBlender()\0".as_ptr(); result.effect.as_ref().is_none_or(|e|e.allowBlender()));
        result
    }
}

// Source: src/core/SkRuntimeEffect.cpp:707-736
pub type MakeEffect = for<'a, 'b> fn(Vec<u8>, &'a Options<'b>) -> EffectResult;
pub fn SkMakeCachedRuntimeEffect(
    make: MakeEffect,
    sksl: Vec<u8>,
) -> Option<std::sync::Arc<SkRuntimeEffect>> {
    type Cache =
        core::pin::Pin<Box<super::SkLRUCache::SkLRUCache<u64, std::sync::Arc<SkRuntimeEffect>>>>;
    // Native function-local no-destructor mutex is a stable, never-dropped
    // static allocation. OnceLock here maps C++ dynamic static initialization
    // of the actual cache, not the cache's synchronization policy.
    static MUTEX: crate::include::private::SkMutex::SkMutex =
        crate::include::private::SkMutex::SkMutex::new();
    struct CacheStorage(core::cell::UnsafeCell<Cache>);
    // Only accesses under MUTEX form a mutable cache lease. Retained actual
    // RuntimeEffect graphs have the same native transferable-owner contract.
    unsafe impl Send for CacheStorage {}
    unsafe impl Sync for CacheStorage {}
    static CACHE: std::sync::OnceLock<CacheStorage> = std::sync::OnceLock::new();
    let cache = CACHE.get_or_init(|| {
        CacheStorage(core::cell::UnsafeCell::new(unsafe {
            super::SkLRUCache::SkLRUCache::new(11, core::ptr::null_mut())
        }))
    });
    let key = super::SkChecksum::Hash64(&sksl, 0);
    {
        let mut storage =
            core::pin::pin!(crate::include::private::SkMutex::SkAutoMutexExclusiveStorage::new());
        unsafe {
            crate::include::private::SkMutex::SkAutoMutexExclusive::init(
                storage.as_mut(),
                core::pin::Pin::new_unchecked(&MUTEX),
            );
        }
        let cache = unsafe { &mut *cache.0.get() };
        if let Some(found) = cache.as_mut().find(&key) {
            return Some(found.clone());
        }
    }
    let mut options = Options::default();
    super::SkRuntimeEffectPriv::AllowPrivateAccess(&mut options);
    let result = make(sksl, &options);
    let Some(effect) = result.effect else {
        {
            #[cfg(sk_debug)]
            {
                let mut error_text = result.errorText.clone();
                error_text.push(0);
                crate::SkDEBUGFAILF!(@source b"src/core/SkRuntimeEffect.cpp\0".as_ptr(),726; "%s",error_text.as_ptr());
            }
        };
        return None;
    };
    crate::SkASSERT!(@source b"src/core/SkRuntimeEffect.cpp\0".as_ptr(),729; @condition b"err.isEmpty()\0".as_ptr(); result.errorText.is_empty());
    {
        let mut storage =
            core::pin::pin!(crate::include::private::SkMutex::SkAutoMutexExclusiveStorage::new());
        unsafe {
            crate::include::private::SkMutex::SkAutoMutexExclusive::init(
                storage.as_mut(),
                core::pin::Pin::new_unchecked(&MUTEX),
            );
        }
        let cache = unsafe { &mut *cache.0.get() };
        unsafe {
            cache.as_mut().insert_or_update(key, effect.clone());
        }
    }
    Some(effect)
}

// Source: src/core/SkRuntimeEffect.cpp:738-755
fn uniform_element_size(r#type: UniformType) -> usize {
    use UniformType::*;
    match r#type {
        kFloat => size_of::<f32>(),
        kFloat2 => size_of::<f32>() * 2,
        kFloat3 => size_of::<f32>() * 3,
        kFloat4 => size_of::<f32>() * 4,
        kFloat2x2 => size_of::<f32>() * 4,
        kFloat3x3 => size_of::<f32>() * 9,
        kFloat4x4 => size_of::<f32>() * 16,
        kInt => size_of::<i32>(),
        kInt2 => size_of::<i32>() * 2,
        kInt3 => size_of::<i32>() * 3,
        kInt4 => size_of::<i32>() * 4,
    }
}
// Source: src/core/SkRuntimeEffect.cpp:757-760
impl Uniform<'_> {
    pub fn sizeInBytes(&self) -> usize {
        const {
            assert!(size_of::<i32>() == size_of::<f32>());
        }
        uniform_element_size(self.r#type).wrapping_mul(self.count as usize)
    }
}
// Source: src/core/SkRuntimeEffect.cpp:762-801
impl SkRuntimeEffect {
    unsafe fn new(
        program: Box<crate::src::sksl::ir::SkSLProgram::Program>,
        options: &Options<'_>,
        main: core::ptr::NonNull<crate::src::sksl::ir::SkSLFunctionDefinition::FunctionDefinition>,
        uniforms: Vec<Uniform<'static>>,
        children: Vec<Child<'static>>,
        usages: Vec<crate::src::sksl::SkSLSampleUsage::SampleUsage>,
        flags: u32,
    ) -> Self {
        crate::SkASSERT!(@source b"src/core/SkRuntimeEffect.cpp\0".as_ptr(),779; @condition b"fChildren.size() == fSampleUsages.size()\0".as_ptr(); children.len()==usages.len());
        let mut hash = super::SkChecksum::Hash32(program.fSource.as_ref().unwrap_unchecked(), 0);
        // Native KnownOptions layout guard uses the same explicit Rust field layout.
        #[repr(C)]
        struct KnownOptions<'a> {
            forceUnoptimized: bool,
            fName: &'a [u8],
            allowPrivateAccess: bool,
            fStableKey: u32,
            maxVersionAllowed: crate::include::sksl::SkSLVersion::Version,
        }
        const {
            assert!(
                core::mem::size_of::<Options<'static>>()
                    == core::mem::size_of::<KnownOptions<'static>>()
            );
        }
        hash = super::SkChecksum::Hash32(&[options.forceUnoptimized as u8], hash);
        hash = super::SkChecksum::Hash32(&[options.allowPrivateAccess as u8], hash);
        hash = super::SkChecksum::Hash32(&options.fStableKey.to_ne_bytes(), hash);
        hash = super::SkChecksum::Hash32(&(options.maxVersionAllowed as i32).to_ne_bytes(), hash);
        let mut name = options.fName.to_vec();
        name.push(0);
        Self {
            fHash: hash,
            fStableKey: options.fStableKey,
            fName: name,
            fBaseProgram: program,
            fCompileRPProgramOnce: crate::include::private::SkOnce::SkOnce::new(),
            fRPProgram: core::cell::UnsafeCell::new(None),
            fMain: main,
            fUniforms: uniforms,
            fChildren: children,
            fSampleUsages: usages,
            fFlags: flags,
        }
    }
}
// Source: src/core/SkRuntimeEffect.cpp:803-803
// Native cpp803 default destructor: field drop now releases complete vectors,
// then cached RP program/retained trace, then base IR and finally name.
// Automatic drop glue also cleans remaining members if a Rust destructor unwinds.
// Source: src/core/SkRuntimeEffect.cpp:805-807
impl SkRuntimeEffect {
    pub fn source(&self) -> &[u8] {
        self.fBaseProgram
            .fSource
            .as_ref()
            .expect("native retained program source")
    }
}

// Source: src/core/SkRuntimeEffect.cpp:809-812
impl SkRuntimeEffect {
    pub fn uniformSize(&self) -> usize {
        self.fUniforms
            .last()
            .map_or(0, |u| (u.offset + u.sizeInBytes() + 3) & !3)
    }
}

// Source: src/core/SkRuntimeEffect.cpp:814-819
impl SkRuntimeEffect {
    pub fn findUniform(&self, name: &[u8]) -> Option<&Uniform<'_>> {
        self.fUniforms.iter().find(|u| u.name == name)
    }
}

// Source: src/core/SkRuntimeEffect.cpp:821-826
impl SkRuntimeEffect {
    pub fn findChild(&self, name: &[u8]) -> Option<&Child<'_>> {
        self.fChildren.iter().find(|c| c.name == name)
    }
}

// Source: src/core/SkRuntimeEffect.cpp:830-849
pub fn MakeDeferredShader(
    effect: &std::sync::Arc<super::SkRuntimeEffect::SkRuntimeEffect>,
    callback: Option<super::SkRuntimeEffectPriv::UniformsCallback>,
    children: &[super::SkRuntimeEffect::ChildPtr],
    localMatrix: Option<&super::SkMatrix::SkMatrix>,
) -> Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>> {
    if !effect.allowShader()
        || !super::SkRuntimeEffect::verify_child_effects(effect.children(), children)
    {
        return None;
    }
    let callback = callback?;
    let shader: std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader> = std::sync::Arc::new(
        crate::src::shaders::SkRuntimeShader::SkRuntimeShader::new_with_callback(
            effect.clone(),
            None,
            callback,
            children,
        ),
    );
    Some(if let Some(matrix) = localMatrix {
        shader.makeWithLocalMatrix(matrix)
    } else {
        shader
    })
}
// Source: src/core/SkRuntimeEffect.cpp:851-860
impl SkRuntimeEffect {
    pub fn makeShaderFromShaders(
        self: &std::sync::Arc<Self>,
        uniforms: Option<std::sync::Arc<super::SkData::SkData>>,
        shaders: &[Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>],
        localMatrix: Option<&super::SkMatrix::SkMatrix>,
    ) -> Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>> {
        let children: Vec<_> = shaders
            .iter()
            .map(|s| s.clone().map_or(ChildPtr::Empty, ChildPtr::Shader))
            .collect();
        self.makeShader(uniforms, &children, localMatrix)
    }
}

// Source: src/core/SkRuntimeEffect.cpp:862-882
impl SkRuntimeEffect {
    pub fn makeShader(
        self: &std::sync::Arc<Self>,
        uniforms: Option<std::sync::Arc<super::SkData::SkData>>,
        children: &[ChildPtr],
        localMatrix: Option<&super::SkMatrix::SkMatrix>,
    ) -> Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>> {
        if !self.allowShader() || !verify_child_effects(self.children(), children) {
            return None;
        }
        let uniforms = uniforms.unwrap_or_else(super::SkData::SkData::MakeEmpty);
        if uniforms.size() != self.uniformSize() {
            return None;
        }
        let shader: std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader> =
            std::sync::Arc::new(crate::src::shaders::SkRuntimeShader::SkRuntimeShader::new(
                self.clone(),
                None,
                Some(uniforms),
                children,
            ));
        Some(if let Some(matrix) = localMatrix {
            std::sync::Arc::new(
                crate::src::shaders::SkLocalMatrixShader::SkLocalMatrixShader::new(shader, matrix),
            )
        } else {
            shader
        })
    }
}

// Source: src/core/SkRuntimeEffect.cpp:884-892
impl SkRuntimeEffect {
    pub fn makeColorFilterFromFilters(
        self: &std::sync::Arc<Self>,
        uniforms: Option<std::sync::Arc<super::SkData::SkData>>,
        filters: &[Option<std::sync::Arc<dyn super::SkColorFilter::SkColorFilter>>],
    ) -> Option<std::sync::Arc<dyn super::SkColorFilter::SkColorFilter>> {
        let children: Vec<_> = filters
            .iter()
            .map(|f| f.clone().map_or(ChildPtr::Empty, ChildPtr::ColorFilter))
            .collect();
        self.makeColorFilter(uniforms, &children)
    }
}

// Source: src/core/SkRuntimeEffect.cpp:894-909
impl SkRuntimeEffect {
    pub fn makeColorFilter(
        self: &std::sync::Arc<Self>,
        uniforms: Option<std::sync::Arc<super::SkData::SkData>>,
        children: &[ChildPtr],
    ) -> Option<std::sync::Arc<dyn super::SkColorFilter::SkColorFilter>> {
        if !self.allowColorFilter() || !verify_child_effects(self.children(), children) {
            return None;
        }
        let uniforms = uniforms.unwrap_or_else(super::SkData::SkData::MakeEmpty);
        if uniforms.size() != self.uniformSize() {
            return None;
        }
        Some(std::sync::Arc::new(
            crate::src::effects::colorfilters::SkRuntimeColorFilter::SkRuntimeColorFilter::new(
                self.clone(),
                uniforms,
                children,
            ),
        ))
    }
}

// Source: src/core/SkRuntimeEffect.cpp:915-930
impl SkRuntimeEffect {
    pub fn makeBlender(
        self: &std::sync::Arc<Self>,
        uniforms: Option<std::sync::Arc<super::SkData::SkData>>,
        children: &[ChildPtr],
    ) -> crate::include::core::SkRefCnt::sk_sp<dyn crate::include::core::SkBlender::SkBlender> {
        if !self.allowBlender() || !verify_child_effects(self.children(), children) {
            return Default::default();
        }
        let uniforms = uniforms.unwrap_or_else(super::SkData::SkData::MakeEmpty);
        if uniforms.size() != self.uniformSize() {
            return Default::default();
        }
        super::SkRuntimeBlender::SkRuntimeBlender::Make(self.clone(), uniforms, children)
    }
}

// Source: src/core/SkRuntimeEffect.cpp:934-943
impl SkRuntimeEffect {
    pub fn MakeTraced(
        shader: std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>,
        coord: &super::SkPoint::SkIPoint,
    ) -> Result<TracedShader, super::SkRasterPipeline::MissingStage> {
        if shader.asRuntimeEffect().is_none() {
            return Ok(TracedShader {
                shader: None,
                debugTrace: None,
            });
        }
        shader
            .asRuntimeShader()
            .expect("native effect-bearing shader is the real RuntimeShader")
            .makeTracedClone(coord)
    }
}

// Source: include/effects/SkRuntimeEffect.h:200-200
// Source: src/core/SkRuntimeEffect.cpp:947-961
impl ChildPtr {
    pub fn r#type(&self) -> Option<ChildType> {
        match self {
            Self::Empty => None,
            Self::Shader(_) => Some(ChildType::kShader),
            Self::ColorFilter(_) => Some(ChildType::kColorFilter),
            Self::Blender(child) => {
                if child.is_some() {
                    Some(ChildType::kBlender)
                } else {
                    None
                }
            }
        }
    }
    // Source: include/effects/SkRuntimeEffect.h:202-202
    // Source: src/core/SkRuntimeEffect.cpp:963-967
    pub fn shader(&self) -> Option<&dyn crate::src::shaders::SkShader::SkShader> {
        if let Self::Shader(child) = self {
            Some(&**child)
        } else {
            None
        }
    }
    // Source: include/effects/SkRuntimeEffect.h:203-203
    // Source: src/core/SkRuntimeEffect.cpp:969-973
    pub fn colorFilter(&self) -> Option<&dyn crate::src::core::SkColorFilter::SkColorFilter> {
        if let Self::ColorFilter(child) = self {
            Some(&**child)
        } else {
            None
        }
    }
    // Source: include/effects/SkRuntimeEffect.h:204-204
    // Source: src/core/SkRuntimeEffect.cpp:975-979
    pub fn blender(&self) -> Option<&dyn crate::include::core::SkBlender::SkBlender> {
        if let Self::Blender(child) = self {
            child.get()
        } else {
            None
        }
    }
}
// Source: src/core/SkRuntimeEffect.cpp:983-990
impl SkRuntimeEffect {
    /// Safety: exclusive native registration phase, before once publication;
    /// no concurrent lookup/registration, with space in the fixed factory table.
    pub unsafe fn RegisterFlattenables() {
        unsafe {
            super::SkFlattenable::Register(
                b"SkRuntimeBlender",
                super::SkRuntimeBlender::SkRuntimeBlender::CreateProc,
            );
            super::SkFlattenable::Register(b"SkRuntimeColorFilter",crate::src::effects::colorfilters::SkRuntimeColorFilter::SkRuntimeColorFilter::CreateProc);
            super::SkFlattenable::Register(
                b"SkRuntimeShader",
                crate::src::shaders::SkRuntimeShader::SkRuntimeShader::CreateProc,
            );
            super::SkFlattenable::Register(
                b"SkRTShader",
                crate::src::shaders::SkRuntimeShader::SkRuntimeShader::CreateProc,
            );
        }
    }
}

// Source: src/core/SkRuntimeEffect.cpp:992-994
impl SkRuntimeEffectBuilder {
    pub fn makeShader(
        &self,
        matrix: Option<&super::SkMatrix::SkMatrix>,
    ) -> Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>> {
        self.fEffect
            .makeShader(Some(self.uniforms()), self.children(), matrix)
    }
}
// Source: src/core/SkRuntimeEffect.cpp:996-998
impl SkRuntimeEffectBuilder {
    pub fn makeBlender(
        &self,
    ) -> crate::include::core::SkRefCnt::sk_sp<dyn crate::include::core::SkBlender::SkBlender> {
        self.fEffect
            .makeBlender(Some(self.uniforms()), self.children())
    }
}
// Source: src/core/SkRuntimeEffect.cpp:1000-1002
impl SkRuntimeEffectBuilder {
    pub fn makeColorFilter(
        &self,
    ) -> Option<std::sync::Arc<dyn super::SkColorFilter::SkColorFilter>> {
        self.fEffect
            .makeColorFilter(Some(self.uniforms()), self.children())
    }
}

impl SkRuntimeEffect {
    // Source: include/effects/SkRuntimeEffect.h:223-223
    // Source: src/core/SkRuntimeEffect.cpp:911-913
    pub fn makeColorFilterNoChildren(
        self: &std::sync::Arc<Self>,
        uniforms: Option<std::sync::Arc<super::SkData::SkData>>,
    ) -> Option<std::sync::Arc<dyn super::SkColorFilter::SkColorFilter>> {
        self.makeColorFilter(uniforms, &[])
    }
}
// Source: include/effects/SkRuntimeEffect.h:499-501
pub type SkRuntimeShaderBuilder = SkRuntimeEffectBuilder;
pub type SkRuntimeColorFilterBuilder = SkRuntimeEffectBuilder;
pub type SkRuntimeBlendBuilder = SkRuntimeEffectBuilder;

// Rust-only destruction adapter for native std::vector<ChildPtr> member owners.
// Selected Apple libc++ __vector/vector.h:746-750 destroys from end to begin.
// No new native algorithm/line credit. UNVERIFIED.
pub(crate) struct RuntimeChildren(pub(crate) Vec<ChildPtr>);
impl RuntimeChildren {
    fn clear_reverse(&mut self) {
        while let Some(child) = self.0.pop() {
            drop(child);
        }
    }
}
impl Drop for RuntimeChildren {
    fn drop(&mut self) {
        struct Finish<'a>(&'a mut RuntimeChildren);
        impl Drop for Finish<'_> {
            fn drop(&mut self) {
                self.0.clear_reverse();
            }
        }
        // Pop removes the exact slot before child destruction. If a Rust child
        // destructor panics, Finish drops the remaining sequence in native order,
        // then empty Vec field drop releases its allocation before later owners.
        let mut finish = Finish(self);
        finish.0.clear_reverse();
    }
}
