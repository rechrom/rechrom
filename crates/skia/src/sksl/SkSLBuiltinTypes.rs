use super::ir::{
    SkSLIRNode::IRNodeBox,
    SkSLType::{NumberKind, TextureAccess, Type, TypeKind},
};
use core::{mem::ManuallyDrop, ptr::NonNull};
// Source: src/sksl/SkSLBuiltinTypes.h:20-171
// All fields retain original declaration order. Implicit-null native unique_ptr fields
// remain Option::None, exactly the absent initializer state in the pinned constructor.
#[repr(C)]
pub struct BuiltinTypes {
    pub fFloat: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fFloat2: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fFloat3: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fFloat4: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fHalf: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fHalf2: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fHalf3: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fHalf4: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fInt: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fInt2: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fInt3: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fInt4: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fUInt: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fUInt2: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fUInt3: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fUInt4: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fShort: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fShort2: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fShort3: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fShort4: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fUShort: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fUShort2: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fUShort3: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fUShort4: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fBool: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fBool2: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fBool3: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fBool4: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fInvalid: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fPoison: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fVoid: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fFloatLiteral: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fIntLiteral: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fFloat2x2: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fFloat2x3: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fFloat2x4: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fFloat3x2: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fFloat3x3: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fFloat3x4: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fFloat4x2: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fFloat4x3: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fFloat4x4: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fHalf2x2: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fHalf2x3: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fHalf2x4: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fHalf3x2: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fHalf3x3: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fHalf3x4: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fHalf4x2: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fHalf4x3: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fHalf4x4: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fVec2: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fVec3: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fVec4: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fIVec2: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fIVec3: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fIVec4: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fUVec2: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fUVec3: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fUVec4: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fBVec2: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fBVec3: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fBVec4: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fMat2: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fMat3: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fMat4: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fMat2x2: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fMat2x3: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fMat2x4: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fMat3x2: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fMat3x3: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fMat3x4: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fMat4x2: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fMat4x3: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fMat4x4: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fTexture2D_sample: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fTextureExternalOES: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fTexture2DRect: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fTexture2D: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fReadOnlyTexture2D: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fWriteOnlyTexture2D: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fGenTexture2D: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fReadableTexture2D: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fWritableTexture2D: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fSampler2D: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fSamplerExternalOES: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fSampler2DRect: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fSampler: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fSubpassInput: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fSubpassInputMS: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fGenType: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fGenHType: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fGenIType: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fGenUType: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fGenBType: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fMat: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fHMat: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fSquareMat: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fSquareHMat: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fVec: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fHVec: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fDVec: ManuallyDrop<Option<IRNodeBox<dyn Type>>>,
    pub fIVec: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fUVec: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fSVec: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fUSVec: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fByteVec: ManuallyDrop<Option<IRNodeBox<dyn Type>>>,
    pub fUByteVec: ManuallyDrop<Option<IRNodeBox<dyn Type>>>,
    pub fBVec: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fSkCaps: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fColorFilter: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fShader: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fBlender: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fAtomicUInt: ManuallyDrop<IRNodeBox<dyn Type>>,
    pub fAtomic_uint: ManuallyDrop<IRNodeBox<dyn Type>>,
}
// Native built-in immutable global Type graph: raw pointers borrow only stable graph
// allocations/static names. No mutable projection exists through a shared singleton.
unsafe impl Send for BuiltinTypes {}
unsafe impl Sync for BuiltinTypes {}
// Source: src/sksl/SkSLBuiltinTypes.cpp:16-19
impl BuiltinTypes {
    pub fn Get() -> &'static Self {
        static VALUE: std::sync::OnceLock<BuiltinTypes> = std::sync::OnceLock::new();
        VALUE.get_or_init(|| {
            assert!(
                !super::SkSLPool::Pool::IsAttached(),
                "global builtin graph cannot borrow a shorter-lived allocation pool"
            );
            unsafe { BuiltinTypes::new() }
        })
    }
    // Source: src/sksl/SkSLBuiltinTypes.cpp:24-219
    /// Safety: no shorter-lived attached allocation pool may back these immutable owners.
    pub unsafe fn new() -> Self {
        let fFloat = <dyn Type>::MakeScalarType(b"float", c"f", NumberKind::kFloat, 10, 32);
        let fFloat2 = <dyn Type>::MakeVectorType(b"float2", c"f2", &*fFloat, 2);
        let fFloat3 = <dyn Type>::MakeVectorType(b"float3", c"f3", &*fFloat, 3);
        let fFloat4 = <dyn Type>::MakeVectorType(b"float4", c"f4", &*fFloat, 4);
        let fHalf = <dyn Type>::MakeScalarType(b"half", c"h", NumberKind::kFloat, 9, 16);
        let fHalf2 = <dyn Type>::MakeVectorType(b"half2", c"h2", &*fHalf, 2);
        let fHalf3 = <dyn Type>::MakeVectorType(b"half3", c"h3", &*fHalf, 3);
        let fHalf4 = <dyn Type>::MakeVectorType(b"half4", c"h4", &*fHalf, 4);
        let fInt = <dyn Type>::MakeScalarType(b"int", c"i", NumberKind::kSigned, 7, 32);
        let fInt2 = <dyn Type>::MakeVectorType(b"int2", c"i2", &*fInt, 2);
        let fInt3 = <dyn Type>::MakeVectorType(b"int3", c"i3", &*fInt, 3);
        let fInt4 = <dyn Type>::MakeVectorType(b"int4", c"i4", &*fInt, 4);
        let fUInt = <dyn Type>::MakeScalarType(b"uint", c"I", NumberKind::kUnsigned, 6, 32);
        let fUInt2 = <dyn Type>::MakeVectorType(b"uint2", c"I2", &*fUInt, 2);
        let fUInt3 = <dyn Type>::MakeVectorType(b"uint3", c"I3", &*fUInt, 3);
        let fUInt4 = <dyn Type>::MakeVectorType(b"uint4", c"I4", &*fUInt, 4);
        let fShort = <dyn Type>::MakeScalarType(b"short", c"s", NumberKind::kSigned, 4, 16);
        let fShort2 = <dyn Type>::MakeVectorType(b"short2", c"s2", &*fShort, 2);
        let fShort3 = <dyn Type>::MakeVectorType(b"short3", c"s3", &*fShort, 3);
        let fShort4 = <dyn Type>::MakeVectorType(b"short4", c"s4", &*fShort, 4);
        let fUShort = <dyn Type>::MakeScalarType(b"ushort", c"S", NumberKind::kUnsigned, 3, 16);
        let fUShort2 = <dyn Type>::MakeVectorType(b"ushort2", c"S2", &*fUShort, 2);
        let fUShort3 = <dyn Type>::MakeVectorType(b"ushort3", c"S3", &*fUShort, 3);
        let fUShort4 = <dyn Type>::MakeVectorType(b"ushort4", c"S4", &*fUShort, 4);
        let fBool = <dyn Type>::MakeScalarType(b"bool", c"b", NumberKind::kBoolean, 0, 1);
        let fBool2 = <dyn Type>::MakeVectorType(b"bool2", c"b2", &*fBool, 2);
        let fBool3 = <dyn Type>::MakeVectorType(b"bool3", c"b3", &*fBool, 3);
        let fBool4 = <dyn Type>::MakeVectorType(b"bool4", c"b4", &*fBool, 4);
        let fInvalid = <dyn Type>::MakeSpecialType(c"<INVALID>", c"O", TypeKind::kOther);
        let fPoison = <dyn Type>::MakeSpecialType(
            core::ffi::CStr::from_bytes_with_nul_unchecked(
                &crate::src::sksl::SkSLCompiler::POISON_TAG,
            ),
            c"P",
            TypeKind::kOther,
        );
        let fVoid = <dyn Type>::MakeSpecialType(c"void", c"v", TypeKind::kVoid);
        let fFloatLiteral = <dyn Type>::MakeLiteralType(c"$floatLiteral", &*fFloat, 8);
        let fIntLiteral = <dyn Type>::MakeLiteralType(c"$intLiteral", &*fInt, 5);
        let fFloat2x2 = <dyn Type>::MakeMatrixType(b"float2x2", c"f22", &*fFloat, 2, 2);
        let fFloat2x3 = <dyn Type>::MakeMatrixType(b"float2x3", c"f23", &*fFloat, 2, 3);
        let fFloat2x4 = <dyn Type>::MakeMatrixType(b"float2x4", c"f24", &*fFloat, 2, 4);
        let fFloat3x2 = <dyn Type>::MakeMatrixType(b"float3x2", c"f32", &*fFloat, 3, 2);
        let fFloat3x3 = <dyn Type>::MakeMatrixType(b"float3x3", c"f33", &*fFloat, 3, 3);
        let fFloat3x4 = <dyn Type>::MakeMatrixType(b"float3x4", c"f34", &*fFloat, 3, 4);
        let fFloat4x2 = <dyn Type>::MakeMatrixType(b"float4x2", c"f42", &*fFloat, 4, 2);
        let fFloat4x3 = <dyn Type>::MakeMatrixType(b"float4x3", c"f43", &*fFloat, 4, 3);
        let fFloat4x4 = <dyn Type>::MakeMatrixType(b"float4x4", c"f44", &*fFloat, 4, 4);
        let fHalf2x2 = <dyn Type>::MakeMatrixType(b"half2x2", c"h22", &*fHalf, 2, 2);
        let fHalf2x3 = <dyn Type>::MakeMatrixType(b"half2x3", c"h23", &*fHalf, 2, 3);
        let fHalf2x4 = <dyn Type>::MakeMatrixType(b"half2x4", c"h24", &*fHalf, 2, 4);
        let fHalf3x2 = <dyn Type>::MakeMatrixType(b"half3x2", c"h32", &*fHalf, 3, 2);
        let fHalf3x3 = <dyn Type>::MakeMatrixType(b"half3x3", c"h33", &*fHalf, 3, 3);
        let fHalf3x4 = <dyn Type>::MakeMatrixType(b"half3x4", c"h34", &*fHalf, 3, 4);
        let fHalf4x2 = <dyn Type>::MakeMatrixType(b"half4x2", c"h42", &*fHalf, 4, 2);
        let fHalf4x3 = <dyn Type>::MakeMatrixType(b"half4x3", c"h43", &*fHalf, 4, 3);
        let fHalf4x4 = <dyn Type>::MakeMatrixType(b"half4x4", c"h44", &*fHalf, 4, 4);
        let fVec2 = <dyn Type>::MakeAliasType(b"vec2", &*fFloat2);
        let fVec3 = <dyn Type>::MakeAliasType(b"vec3", &*fFloat3);
        let fVec4 = <dyn Type>::MakeAliasType(b"vec4", &*fFloat4);
        let fIVec2 = <dyn Type>::MakeAliasType(b"ivec2", &*fInt2);
        let fIVec3 = <dyn Type>::MakeAliasType(b"ivec3", &*fInt3);
        let fIVec4 = <dyn Type>::MakeAliasType(b"ivec4", &*fInt4);
        let fUVec2 = <dyn Type>::MakeAliasType(b"uvec2", &*fUInt2);
        let fUVec3 = <dyn Type>::MakeAliasType(b"uvec3", &*fUInt3);
        let fUVec4 = <dyn Type>::MakeAliasType(b"uvec4", &*fUInt4);
        let fBVec2 = <dyn Type>::MakeAliasType(b"bvec2", &*fBool2);
        let fBVec3 = <dyn Type>::MakeAliasType(b"bvec3", &*fBool3);
        let fBVec4 = <dyn Type>::MakeAliasType(b"bvec4", &*fBool4);
        let fMat2 = <dyn Type>::MakeAliasType(b"mat2", &*fFloat2x2);
        let fMat3 = <dyn Type>::MakeAliasType(b"mat3", &*fFloat3x3);
        let fMat4 = <dyn Type>::MakeAliasType(b"mat4", &*fFloat4x4);
        let fMat2x2 = <dyn Type>::MakeAliasType(b"mat2x2", &*fFloat2x2);
        let fMat2x3 = <dyn Type>::MakeAliasType(b"mat2x3", &*fFloat2x3);
        let fMat2x4 = <dyn Type>::MakeAliasType(b"mat2x4", &*fFloat2x4);
        let fMat3x2 = <dyn Type>::MakeAliasType(b"mat3x2", &*fFloat3x2);
        let fMat3x3 = <dyn Type>::MakeAliasType(b"mat3x3", &*fFloat3x3);
        let fMat3x4 = <dyn Type>::MakeAliasType(b"mat3x4", &*fFloat3x4);
        let fMat4x2 = <dyn Type>::MakeAliasType(b"mat4x2", &*fFloat4x2);
        let fMat4x3 = <dyn Type>::MakeAliasType(b"mat4x3", &*fFloat4x3);
        let fMat4x4 = <dyn Type>::MakeAliasType(b"mat4x4", &*fFloat4x4);
        let fTexture2D_sample = <dyn Type>::MakeTextureType(
            c"$texture2D_sample",
            1,
            false,
            false,
            false,
            TextureAccess::kSample,
        );
        let fTextureExternalOES = <dyn Type>::MakeTextureType(
            c"textureExternalOES",
            1,
            false,
            false,
            false,
            TextureAccess::kSample,
        );
        let fTexture2DRect = <dyn Type>::MakeTextureType(
            c"texture2DRect",
            4,
            false,
            false,
            false,
            TextureAccess::kSample,
        );
        let fTexture2D = <dyn Type>::MakeTextureType(
            c"texture2D",
            1,
            false,
            false,
            false,
            TextureAccess::kReadWrite,
        );
        let fReadOnlyTexture2D = <dyn Type>::MakeTextureType(
            c"readonlyTexture2D",
            1,
            false,
            false,
            false,
            TextureAccess::kRead,
        );
        let fWriteOnlyTexture2D = <dyn Type>::MakeTextureType(
            c"writeonlyTexture2D",
            1,
            false,
            false,
            false,
            TextureAccess::kWrite,
        );
        let fGenTexture2D = <dyn Type>::MakeGenericType(
            c"$genTexture2D",
            &[
                NonNull::from(&*fReadOnlyTexture2D),
                NonNull::from(&*fWriteOnlyTexture2D),
                NonNull::from(&*fTexture2D),
            ],
            Some(NonNull::from(&*fTexture2D)),
        );
        let fReadableTexture2D = <dyn Type>::MakeGenericType(
            c"$readableTexture2D",
            &[
                NonNull::from(&*fReadOnlyTexture2D),
                NonNull::from(&*fInvalid),
                NonNull::from(&*fTexture2D),
            ],
            Some(NonNull::from(&*fTexture2D)),
        );
        let fWritableTexture2D = <dyn Type>::MakeGenericType(
            c"$writableTexture2D",
            &[
                NonNull::from(&*fInvalid),
                NonNull::from(&*fWriteOnlyTexture2D),
                NonNull::from(&*fTexture2D),
            ],
            Some(NonNull::from(&*fTexture2D)),
        );
        let fSampler2D = <dyn Type>::MakeSamplerType(c"sampler2D", &*fTexture2D_sample);
        let fSamplerExternalOES =
            <dyn Type>::MakeSamplerType(c"samplerExternalOES", &*fTextureExternalOES);
        let fSampler2DRect = <dyn Type>::MakeSamplerType(c"sampler2DRect", &*fTexture2DRect);
        let fSampler = <dyn Type>::MakeSpecialType(c"sampler", c"ss", TypeKind::kSeparateSampler);
        let fSubpassInput = <dyn Type>::MakeTextureType(
            c"subpassInput",
            6,
            false,
            false,
            false,
            TextureAccess::kRead,
        );
        let fSubpassInputMS = <dyn Type>::MakeTextureType(
            c"subpassInputMS",
            6,
            false,
            false,
            true,
            TextureAccess::kRead,
        );
        let fGenType = <dyn Type>::MakeGenericType(
            c"$genType",
            &[
                NonNull::from(&*fFloat),
                NonNull::from(&*fFloat2),
                NonNull::from(&*fFloat3),
                NonNull::from(&*fFloat4),
            ],
            Some(NonNull::from(&*fFloat)),
        );
        let fGenHType = <dyn Type>::MakeGenericType(
            c"$genHType",
            &[
                NonNull::from(&*fHalf),
                NonNull::from(&*fHalf2),
                NonNull::from(&*fHalf3),
                NonNull::from(&*fHalf4),
            ],
            Some(NonNull::from(&*fHalf)),
        );
        let fGenIType = <dyn Type>::MakeGenericType(
            c"$genIType",
            &[
                NonNull::from(&*fInt),
                NonNull::from(&*fInt2),
                NonNull::from(&*fInt3),
                NonNull::from(&*fInt4),
            ],
            Some(NonNull::from(&*fInt)),
        );
        let fGenUType = <dyn Type>::MakeGenericType(
            c"$genUType",
            &[
                NonNull::from(&*fUInt),
                NonNull::from(&*fUInt2),
                NonNull::from(&*fUInt3),
                NonNull::from(&*fUInt4),
            ],
            Some(NonNull::from(&*fUInt)),
        );
        let fGenBType = <dyn Type>::MakeGenericType(
            c"$genBType",
            &[
                NonNull::from(&*fBool),
                NonNull::from(&*fBool2),
                NonNull::from(&*fBool3),
                NonNull::from(&*fBool4),
            ],
            Some(NonNull::from(&*fBool)),
        );
        let fMat = <dyn Type>::MakeGenericType(
            c"$mat",
            &[
                NonNull::from(&*fFloat2x2),
                NonNull::from(&*fFloat2x3),
                NonNull::from(&*fFloat2x4),
                NonNull::from(&*fFloat3x2),
                NonNull::from(&*fFloat3x3),
                NonNull::from(&*fFloat3x4),
                NonNull::from(&*fFloat4x2),
                NonNull::from(&*fFloat4x3),
                NonNull::from(&*fFloat4x4),
            ],
            Some(NonNull::from(&*fFloat)),
        );
        let fHMat = <dyn Type>::MakeGenericType(
            c"$hmat",
            &[
                NonNull::from(&*fHalf2x2),
                NonNull::from(&*fHalf2x3),
                NonNull::from(&*fHalf2x4),
                NonNull::from(&*fHalf3x2),
                NonNull::from(&*fHalf3x3),
                NonNull::from(&*fHalf3x4),
                NonNull::from(&*fHalf4x2),
                NonNull::from(&*fHalf4x3),
                NonNull::from(&*fHalf4x4),
            ],
            Some(NonNull::from(&*fHalf)),
        );
        let fSquareMat = <dyn Type>::MakeGenericType(
            c"$squareMat",
            &[
                NonNull::from(&*fInvalid),
                NonNull::from(&*fFloat2x2),
                NonNull::from(&*fFloat3x3),
                NonNull::from(&*fFloat4x4),
            ],
            Some(NonNull::from(&*fFloat)),
        );
        let fSquareHMat = <dyn Type>::MakeGenericType(
            c"$squareHMat",
            &[
                NonNull::from(&*fInvalid),
                NonNull::from(&*fHalf2x2),
                NonNull::from(&*fHalf3x3),
                NonNull::from(&*fHalf4x4),
            ],
            Some(NonNull::from(&*fHalf)),
        );
        let fVec = <dyn Type>::MakeGenericType(
            c"$vec",
            &[
                NonNull::from(&*fInvalid),
                NonNull::from(&*fFloat2),
                NonNull::from(&*fFloat3),
                NonNull::from(&*fFloat4),
            ],
            Some(NonNull::from(&*fFloat)),
        );
        let fHVec = <dyn Type>::MakeGenericType(
            c"$hvec",
            &[
                NonNull::from(&*fInvalid),
                NonNull::from(&*fHalf2),
                NonNull::from(&*fHalf3),
                NonNull::from(&*fHalf4),
            ],
            Some(NonNull::from(&*fHalf)),
        );
        let fIVec = <dyn Type>::MakeGenericType(
            c"$ivec",
            &[
                NonNull::from(&*fInvalid),
                NonNull::from(&*fInt2),
                NonNull::from(&*fInt3),
                NonNull::from(&*fInt4),
            ],
            Some(NonNull::from(&*fInt)),
        );
        let fUVec = <dyn Type>::MakeGenericType(
            c"$uvec",
            &[
                NonNull::from(&*fInvalid),
                NonNull::from(&*fUInt2),
                NonNull::from(&*fUInt3),
                NonNull::from(&*fUInt4),
            ],
            Some(NonNull::from(&*fUInt)),
        );
        let fSVec = <dyn Type>::MakeGenericType(
            c"$svec",
            &[
                NonNull::from(&*fInvalid),
                NonNull::from(&*fShort2),
                NonNull::from(&*fShort3),
                NonNull::from(&*fShort4),
            ],
            Some(NonNull::from(&*fShort)),
        );
        let fUSVec = <dyn Type>::MakeGenericType(
            c"$usvec",
            &[
                NonNull::from(&*fInvalid),
                NonNull::from(&*fUShort2),
                NonNull::from(&*fUShort3),
                NonNull::from(&*fUShort4),
            ],
            Some(NonNull::from(&*fUShort)),
        );
        let fBVec = <dyn Type>::MakeGenericType(
            c"$bvec",
            &[
                NonNull::from(&*fInvalid),
                NonNull::from(&*fBool2),
                NonNull::from(&*fBool3),
                NonNull::from(&*fBool4),
            ],
            Some(NonNull::from(&*fBool)),
        );
        let fSkCaps = <dyn Type>::MakeSpecialType(c"$sk_Caps", c"O", TypeKind::kOther);
        let fColorFilter =
            <dyn Type>::MakeSpecialType(c"colorFilter", c"CF", TypeKind::kColorFilter);
        let fShader = <dyn Type>::MakeSpecialType(c"shader", c"SH", TypeKind::kShader);
        let fBlender = <dyn Type>::MakeSpecialType(c"blender", c"B", TypeKind::kBlender);
        let fAtomicUInt = <dyn Type>::MakeAtomicType(b"atomicUint", c"au");
        let fAtomic_uint = <dyn Type>::MakeAliasType(b"atomic_uint", &*fAtomicUInt);
        Self {
            fFloat: ManuallyDrop::new(fFloat),
            fFloat2: ManuallyDrop::new(fFloat2),
            fFloat3: ManuallyDrop::new(fFloat3),
            fFloat4: ManuallyDrop::new(fFloat4),
            fHalf: ManuallyDrop::new(fHalf),
            fHalf2: ManuallyDrop::new(fHalf2),
            fHalf3: ManuallyDrop::new(fHalf3),
            fHalf4: ManuallyDrop::new(fHalf4),
            fInt: ManuallyDrop::new(fInt),
            fInt2: ManuallyDrop::new(fInt2),
            fInt3: ManuallyDrop::new(fInt3),
            fInt4: ManuallyDrop::new(fInt4),
            fUInt: ManuallyDrop::new(fUInt),
            fUInt2: ManuallyDrop::new(fUInt2),
            fUInt3: ManuallyDrop::new(fUInt3),
            fUInt4: ManuallyDrop::new(fUInt4),
            fShort: ManuallyDrop::new(fShort),
            fShort2: ManuallyDrop::new(fShort2),
            fShort3: ManuallyDrop::new(fShort3),
            fShort4: ManuallyDrop::new(fShort4),
            fUShort: ManuallyDrop::new(fUShort),
            fUShort2: ManuallyDrop::new(fUShort2),
            fUShort3: ManuallyDrop::new(fUShort3),
            fUShort4: ManuallyDrop::new(fUShort4),
            fBool: ManuallyDrop::new(fBool),
            fBool2: ManuallyDrop::new(fBool2),
            fBool3: ManuallyDrop::new(fBool3),
            fBool4: ManuallyDrop::new(fBool4),
            fInvalid: ManuallyDrop::new(fInvalid),
            fPoison: ManuallyDrop::new(fPoison),
            fVoid: ManuallyDrop::new(fVoid),
            fFloatLiteral: ManuallyDrop::new(fFloatLiteral),
            fIntLiteral: ManuallyDrop::new(fIntLiteral),
            fFloat2x2: ManuallyDrop::new(fFloat2x2),
            fFloat2x3: ManuallyDrop::new(fFloat2x3),
            fFloat2x4: ManuallyDrop::new(fFloat2x4),
            fFloat3x2: ManuallyDrop::new(fFloat3x2),
            fFloat3x3: ManuallyDrop::new(fFloat3x3),
            fFloat3x4: ManuallyDrop::new(fFloat3x4),
            fFloat4x2: ManuallyDrop::new(fFloat4x2),
            fFloat4x3: ManuallyDrop::new(fFloat4x3),
            fFloat4x4: ManuallyDrop::new(fFloat4x4),
            fHalf2x2: ManuallyDrop::new(fHalf2x2),
            fHalf2x3: ManuallyDrop::new(fHalf2x3),
            fHalf2x4: ManuallyDrop::new(fHalf2x4),
            fHalf3x2: ManuallyDrop::new(fHalf3x2),
            fHalf3x3: ManuallyDrop::new(fHalf3x3),
            fHalf3x4: ManuallyDrop::new(fHalf3x4),
            fHalf4x2: ManuallyDrop::new(fHalf4x2),
            fHalf4x3: ManuallyDrop::new(fHalf4x3),
            fHalf4x4: ManuallyDrop::new(fHalf4x4),
            fVec2: ManuallyDrop::new(fVec2),
            fVec3: ManuallyDrop::new(fVec3),
            fVec4: ManuallyDrop::new(fVec4),
            fIVec2: ManuallyDrop::new(fIVec2),
            fIVec3: ManuallyDrop::new(fIVec3),
            fIVec4: ManuallyDrop::new(fIVec4),
            fUVec2: ManuallyDrop::new(fUVec2),
            fUVec3: ManuallyDrop::new(fUVec3),
            fUVec4: ManuallyDrop::new(fUVec4),
            fBVec2: ManuallyDrop::new(fBVec2),
            fBVec3: ManuallyDrop::new(fBVec3),
            fBVec4: ManuallyDrop::new(fBVec4),
            fMat2: ManuallyDrop::new(fMat2),
            fMat3: ManuallyDrop::new(fMat3),
            fMat4: ManuallyDrop::new(fMat4),
            fMat2x2: ManuallyDrop::new(fMat2x2),
            fMat2x3: ManuallyDrop::new(fMat2x3),
            fMat2x4: ManuallyDrop::new(fMat2x4),
            fMat3x2: ManuallyDrop::new(fMat3x2),
            fMat3x3: ManuallyDrop::new(fMat3x3),
            fMat3x4: ManuallyDrop::new(fMat3x4),
            fMat4x2: ManuallyDrop::new(fMat4x2),
            fMat4x3: ManuallyDrop::new(fMat4x3),
            fMat4x4: ManuallyDrop::new(fMat4x4),
            fTexture2D_sample: ManuallyDrop::new(fTexture2D_sample),
            fTextureExternalOES: ManuallyDrop::new(fTextureExternalOES),
            fTexture2DRect: ManuallyDrop::new(fTexture2DRect),
            fTexture2D: ManuallyDrop::new(fTexture2D),
            fReadOnlyTexture2D: ManuallyDrop::new(fReadOnlyTexture2D),
            fWriteOnlyTexture2D: ManuallyDrop::new(fWriteOnlyTexture2D),
            fGenTexture2D: ManuallyDrop::new(fGenTexture2D),
            fReadableTexture2D: ManuallyDrop::new(fReadableTexture2D),
            fWritableTexture2D: ManuallyDrop::new(fWritableTexture2D),
            fSampler2D: ManuallyDrop::new(fSampler2D),
            fSamplerExternalOES: ManuallyDrop::new(fSamplerExternalOES),
            fSampler2DRect: ManuallyDrop::new(fSampler2DRect),
            fSampler: ManuallyDrop::new(fSampler),
            fSubpassInput: ManuallyDrop::new(fSubpassInput),
            fSubpassInputMS: ManuallyDrop::new(fSubpassInputMS),
            fGenType: ManuallyDrop::new(fGenType),
            fGenHType: ManuallyDrop::new(fGenHType),
            fGenIType: ManuallyDrop::new(fGenIType),
            fGenUType: ManuallyDrop::new(fGenUType),
            fGenBType: ManuallyDrop::new(fGenBType),
            fMat: ManuallyDrop::new(fMat),
            fHMat: ManuallyDrop::new(fHMat),
            fSquareMat: ManuallyDrop::new(fSquareMat),
            fSquareHMat: ManuallyDrop::new(fSquareHMat),
            fVec: ManuallyDrop::new(fVec),
            fHVec: ManuallyDrop::new(fHVec),
            fDVec: ManuallyDrop::new(None),
            fIVec: ManuallyDrop::new(fIVec),
            fUVec: ManuallyDrop::new(fUVec),
            fSVec: ManuallyDrop::new(fSVec),
            fUSVec: ManuallyDrop::new(fUSVec),
            fByteVec: ManuallyDrop::new(None),
            fUByteVec: ManuallyDrop::new(None),
            fBVec: ManuallyDrop::new(fBVec),
            fSkCaps: ManuallyDrop::new(fSkCaps),
            fColorFilter: ManuallyDrop::new(fColorFilter),
            fShader: ManuallyDrop::new(fShader),
            fBlender: ManuallyDrop::new(fBlender),
            fAtomicUInt: ManuallyDrop::new(fAtomicUInt),
            fAtomic_uint: ManuallyDrop::new(fAtomic_uint),
        }
    }
}
// Native implicit destructor: reverse member order; no use of default Rust field-drop order.
impl Drop for BuiltinTypes {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.fAtomic_uint);
            ManuallyDrop::drop(&mut self.fAtomicUInt);
            ManuallyDrop::drop(&mut self.fBlender);
            ManuallyDrop::drop(&mut self.fShader);
            ManuallyDrop::drop(&mut self.fColorFilter);
            ManuallyDrop::drop(&mut self.fSkCaps);
            ManuallyDrop::drop(&mut self.fBVec);
            ManuallyDrop::drop(&mut self.fUByteVec);
            ManuallyDrop::drop(&mut self.fByteVec);
            ManuallyDrop::drop(&mut self.fUSVec);
            ManuallyDrop::drop(&mut self.fSVec);
            ManuallyDrop::drop(&mut self.fUVec);
            ManuallyDrop::drop(&mut self.fIVec);
            ManuallyDrop::drop(&mut self.fDVec);
            ManuallyDrop::drop(&mut self.fHVec);
            ManuallyDrop::drop(&mut self.fVec);
            ManuallyDrop::drop(&mut self.fSquareHMat);
            ManuallyDrop::drop(&mut self.fSquareMat);
            ManuallyDrop::drop(&mut self.fHMat);
            ManuallyDrop::drop(&mut self.fMat);
            ManuallyDrop::drop(&mut self.fGenBType);
            ManuallyDrop::drop(&mut self.fGenUType);
            ManuallyDrop::drop(&mut self.fGenIType);
            ManuallyDrop::drop(&mut self.fGenHType);
            ManuallyDrop::drop(&mut self.fGenType);
            ManuallyDrop::drop(&mut self.fSubpassInputMS);
            ManuallyDrop::drop(&mut self.fSubpassInput);
            ManuallyDrop::drop(&mut self.fSampler);
            ManuallyDrop::drop(&mut self.fSampler2DRect);
            ManuallyDrop::drop(&mut self.fSamplerExternalOES);
            ManuallyDrop::drop(&mut self.fSampler2D);
            ManuallyDrop::drop(&mut self.fWritableTexture2D);
            ManuallyDrop::drop(&mut self.fReadableTexture2D);
            ManuallyDrop::drop(&mut self.fGenTexture2D);
            ManuallyDrop::drop(&mut self.fWriteOnlyTexture2D);
            ManuallyDrop::drop(&mut self.fReadOnlyTexture2D);
            ManuallyDrop::drop(&mut self.fTexture2D);
            ManuallyDrop::drop(&mut self.fTexture2DRect);
            ManuallyDrop::drop(&mut self.fTextureExternalOES);
            ManuallyDrop::drop(&mut self.fTexture2D_sample);
            ManuallyDrop::drop(&mut self.fMat4x4);
            ManuallyDrop::drop(&mut self.fMat4x3);
            ManuallyDrop::drop(&mut self.fMat4x2);
            ManuallyDrop::drop(&mut self.fMat3x4);
            ManuallyDrop::drop(&mut self.fMat3x3);
            ManuallyDrop::drop(&mut self.fMat3x2);
            ManuallyDrop::drop(&mut self.fMat2x4);
            ManuallyDrop::drop(&mut self.fMat2x3);
            ManuallyDrop::drop(&mut self.fMat2x2);
            ManuallyDrop::drop(&mut self.fMat4);
            ManuallyDrop::drop(&mut self.fMat3);
            ManuallyDrop::drop(&mut self.fMat2);
            ManuallyDrop::drop(&mut self.fBVec4);
            ManuallyDrop::drop(&mut self.fBVec3);
            ManuallyDrop::drop(&mut self.fBVec2);
            ManuallyDrop::drop(&mut self.fUVec4);
            ManuallyDrop::drop(&mut self.fUVec3);
            ManuallyDrop::drop(&mut self.fUVec2);
            ManuallyDrop::drop(&mut self.fIVec4);
            ManuallyDrop::drop(&mut self.fIVec3);
            ManuallyDrop::drop(&mut self.fIVec2);
            ManuallyDrop::drop(&mut self.fVec4);
            ManuallyDrop::drop(&mut self.fVec3);
            ManuallyDrop::drop(&mut self.fVec2);
            ManuallyDrop::drop(&mut self.fHalf4x4);
            ManuallyDrop::drop(&mut self.fHalf4x3);
            ManuallyDrop::drop(&mut self.fHalf4x2);
            ManuallyDrop::drop(&mut self.fHalf3x4);
            ManuallyDrop::drop(&mut self.fHalf3x3);
            ManuallyDrop::drop(&mut self.fHalf3x2);
            ManuallyDrop::drop(&mut self.fHalf2x4);
            ManuallyDrop::drop(&mut self.fHalf2x3);
            ManuallyDrop::drop(&mut self.fHalf2x2);
            ManuallyDrop::drop(&mut self.fFloat4x4);
            ManuallyDrop::drop(&mut self.fFloat4x3);
            ManuallyDrop::drop(&mut self.fFloat4x2);
            ManuallyDrop::drop(&mut self.fFloat3x4);
            ManuallyDrop::drop(&mut self.fFloat3x3);
            ManuallyDrop::drop(&mut self.fFloat3x2);
            ManuallyDrop::drop(&mut self.fFloat2x4);
            ManuallyDrop::drop(&mut self.fFloat2x3);
            ManuallyDrop::drop(&mut self.fFloat2x2);
            ManuallyDrop::drop(&mut self.fIntLiteral);
            ManuallyDrop::drop(&mut self.fFloatLiteral);
            ManuallyDrop::drop(&mut self.fVoid);
            ManuallyDrop::drop(&mut self.fPoison);
            ManuallyDrop::drop(&mut self.fInvalid);
            ManuallyDrop::drop(&mut self.fBool4);
            ManuallyDrop::drop(&mut self.fBool3);
            ManuallyDrop::drop(&mut self.fBool2);
            ManuallyDrop::drop(&mut self.fBool);
            ManuallyDrop::drop(&mut self.fUShort4);
            ManuallyDrop::drop(&mut self.fUShort3);
            ManuallyDrop::drop(&mut self.fUShort2);
            ManuallyDrop::drop(&mut self.fUShort);
            ManuallyDrop::drop(&mut self.fShort4);
            ManuallyDrop::drop(&mut self.fShort3);
            ManuallyDrop::drop(&mut self.fShort2);
            ManuallyDrop::drop(&mut self.fShort);
            ManuallyDrop::drop(&mut self.fUInt4);
            ManuallyDrop::drop(&mut self.fUInt3);
            ManuallyDrop::drop(&mut self.fUInt2);
            ManuallyDrop::drop(&mut self.fUInt);
            ManuallyDrop::drop(&mut self.fInt4);
            ManuallyDrop::drop(&mut self.fInt3);
            ManuallyDrop::drop(&mut self.fInt2);
            ManuallyDrop::drop(&mut self.fInt);
            ManuallyDrop::drop(&mut self.fHalf4);
            ManuallyDrop::drop(&mut self.fHalf3);
            ManuallyDrop::drop(&mut self.fHalf2);
            ManuallyDrop::drop(&mut self.fHalf);
            ManuallyDrop::drop(&mut self.fFloat4);
            ManuallyDrop::drop(&mut self.fFloat3);
            ManuallyDrop::drop(&mut self.fFloat2);
            ManuallyDrop::drop(&mut self.fFloat);
        }
    }
}
