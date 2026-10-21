// UNVERIFIED: user prohibited all validation.
// Source: src/core/SkSLTypeShared.h:16-16
#[repr(i8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkSLType {
    // Source: src/core/SkSLTypeShared.h:17-17
    kVoid,
    // Source: src/core/SkSLTypeShared.h:18-18
    kBool,
    // Source: src/core/SkSLTypeShared.h:19-19
    kBool2,
    // Source: src/core/SkSLTypeShared.h:20-20
    kBool3,
    // Source: src/core/SkSLTypeShared.h:21-21
    kBool4,
    // Source: src/core/SkSLTypeShared.h:22-22
    kShort,
    // Source: src/core/SkSLTypeShared.h:23-23
    kShort2,
    // Source: src/core/SkSLTypeShared.h:24-24
    kShort3,
    // Source: src/core/SkSLTypeShared.h:25-25
    kShort4,
    // Source: src/core/SkSLTypeShared.h:26-26
    kUShort,
    // Source: src/core/SkSLTypeShared.h:27-27
    kUShort2,
    // Source: src/core/SkSLTypeShared.h:28-28
    kUShort3,
    // Source: src/core/SkSLTypeShared.h:29-29
    kUShort4,
    // Source: src/core/SkSLTypeShared.h:30-30
    kFloat,
    // Source: src/core/SkSLTypeShared.h:31-31
    kFloat2,
    // Source: src/core/SkSLTypeShared.h:32-32
    kFloat3,
    // Source: src/core/SkSLTypeShared.h:33-33
    kFloat4,
    // Source: src/core/SkSLTypeShared.h:34-34
    kFloat2x2,
    // Source: src/core/SkSLTypeShared.h:35-35
    kFloat3x3,
    // Source: src/core/SkSLTypeShared.h:36-36
    kFloat4x4,
    // Source: src/core/SkSLTypeShared.h:37-37
    kHalf,
    // Source: src/core/SkSLTypeShared.h:38-38
    kHalf2,
    // Source: src/core/SkSLTypeShared.h:39-39
    kHalf3,
    // Source: src/core/SkSLTypeShared.h:40-40
    kHalf4,
    // Source: src/core/SkSLTypeShared.h:41-41
    kHalf2x2,
    // Source: src/core/SkSLTypeShared.h:42-42
    kHalf3x3,
    // Source: src/core/SkSLTypeShared.h:43-43
    kHalf4x4,
    // Source: src/core/SkSLTypeShared.h:44-44
    kInt,
    // Source: src/core/SkSLTypeShared.h:45-45
    kInt2,
    // Source: src/core/SkSLTypeShared.h:46-46
    kInt3,
    // Source: src/core/SkSLTypeShared.h:47-47
    kInt4,
    // Source: src/core/SkSLTypeShared.h:48-48
    kUInt,
    // Source: src/core/SkSLTypeShared.h:49-49
    kUInt2,
    // Source: src/core/SkSLTypeShared.h:50-50
    kUInt3,
    // Source: src/core/SkSLTypeShared.h:51-51
    kUInt4,
    // Source: src/core/SkSLTypeShared.h:52-52
    kTexture2DSampler,
    // Source: src/core/SkSLTypeShared.h:53-53
    kTextureExternalSampler,
    // Source: src/core/SkSLTypeShared.h:54-54
    kTexture2DRectSampler,
    // Source: src/core/SkSLTypeShared.h:55-55
    kTexture2D,
    // Source: src/core/SkSLTypeShared.h:56-56
    kSampler,
    // Source: src/core/SkSLTypeShared.h:57-57
    kInput,
}
// Source: src/core/SkSLTypeShared.h:59-59
impl SkSLType {
    pub const kLast: Self = Self::kInput;
}
// Source: src/core/SkSLTypeShared.h:61-61
pub const kSkSLTypeCount: i32 = SkSLType::kLast as i32 + 1;
// Source: src/core/SkSLTypeShared.h:67-115
pub const fn SkSLTypeIsFloatType(ty: SkSLType) -> bool {
    match ty {
        SkSLType::kFloat
        | SkSLType::kFloat2
        | SkSLType::kFloat3
        | SkSLType::kFloat4
        | SkSLType::kFloat2x2
        | SkSLType::kFloat3x3
        | SkSLType::kFloat4x4
        | SkSLType::kHalf
        | SkSLType::kHalf2
        | SkSLType::kHalf3
        | SkSLType::kHalf4
        | SkSLType::kHalf2x2
        | SkSLType::kHalf3x3
        | SkSLType::kHalf4x4 => true,
        SkSLType::kVoid
        | SkSLType::kTexture2DSampler
        | SkSLType::kTextureExternalSampler
        | SkSLType::kTexture2DRectSampler
        | SkSLType::kBool
        | SkSLType::kBool2
        | SkSLType::kBool3
        | SkSLType::kBool4
        | SkSLType::kShort
        | SkSLType::kShort2
        | SkSLType::kShort3
        | SkSLType::kShort4
        | SkSLType::kUShort
        | SkSLType::kUShort2
        | SkSLType::kUShort3
        | SkSLType::kUShort4
        | SkSLType::kInt
        | SkSLType::kInt2
        | SkSLType::kInt3
        | SkSLType::kInt4
        | SkSLType::kUInt
        | SkSLType::kUInt2
        | SkSLType::kUInt3
        | SkSLType::kUInt4
        | SkSLType::kTexture2D
        | SkSLType::kSampler
        | SkSLType::kInput => false,
        _ => unsafe { core::hint::unreachable_unchecked() },
    }
}
// Source: src/core/SkSLTypeShared.h:118-166
pub const fn SkSLTypeIsIntegralType(ty: SkSLType) -> bool {
    match ty {
        SkSLType::kShort
        | SkSLType::kShort2
        | SkSLType::kShort3
        | SkSLType::kShort4
        | SkSLType::kUShort
        | SkSLType::kUShort2
        | SkSLType::kUShort3
        | SkSLType::kUShort4
        | SkSLType::kInt
        | SkSLType::kInt2
        | SkSLType::kInt3
        | SkSLType::kInt4
        | SkSLType::kUInt
        | SkSLType::kUInt2
        | SkSLType::kUInt3
        | SkSLType::kUInt4 => true,
        SkSLType::kFloat
        | SkSLType::kFloat2
        | SkSLType::kFloat3
        | SkSLType::kFloat4
        | SkSLType::kFloat2x2
        | SkSLType::kFloat3x3
        | SkSLType::kFloat4x4
        | SkSLType::kHalf
        | SkSLType::kHalf2
        | SkSLType::kHalf3
        | SkSLType::kHalf4
        | SkSLType::kHalf2x2
        | SkSLType::kHalf3x3
        | SkSLType::kHalf4x4
        | SkSLType::kVoid
        | SkSLType::kTexture2DSampler
        | SkSLType::kTextureExternalSampler
        | SkSLType::kTexture2DRectSampler
        | SkSLType::kBool
        | SkSLType::kBool2
        | SkSLType::kBool3
        | SkSLType::kBool4
        | SkSLType::kTexture2D
        | SkSLType::kSampler
        | SkSLType::kInput => false,
        _ => unsafe { core::hint::unreachable_unchecked() },
    }
}
// Source: src/core/SkSLTypeShared.h:169-223
pub const fn SkSLTypeVecLength(ty: SkSLType) -> i32 {
    match ty {
        SkSLType::kFloat
        | SkSLType::kHalf
        | SkSLType::kBool
        | SkSLType::kShort
        | SkSLType::kUShort
        | SkSLType::kInt
        | SkSLType::kUInt => 1,
        SkSLType::kFloat2
        | SkSLType::kHalf2
        | SkSLType::kBool2
        | SkSLType::kShort2
        | SkSLType::kUShort2
        | SkSLType::kInt2
        | SkSLType::kUInt2 => 2,
        SkSLType::kFloat3
        | SkSLType::kHalf3
        | SkSLType::kBool3
        | SkSLType::kShort3
        | SkSLType::kUShort3
        | SkSLType::kInt3
        | SkSLType::kUInt3 => 3,
        SkSLType::kFloat4
        | SkSLType::kHalf4
        | SkSLType::kBool4
        | SkSLType::kShort4
        | SkSLType::kUShort4
        | SkSLType::kInt4
        | SkSLType::kUInt4 => 4,
        SkSLType::kFloat2x2
        | SkSLType::kFloat3x3
        | SkSLType::kFloat4x4
        | SkSLType::kHalf2x2
        | SkSLType::kHalf3x3
        | SkSLType::kHalf4x4
        | SkSLType::kVoid
        | SkSLType::kTexture2DSampler
        | SkSLType::kTextureExternalSampler
        | SkSLType::kTexture2DRectSampler
        | SkSLType::kTexture2D
        | SkSLType::kSampler
        | SkSLType::kInput => -1,
        _ => unsafe { core::hint::unreachable_unchecked() },
    }
}
// Source: src/core/SkSLTypeShared.h:229-260
pub const fn SkSLTypeCanBeUniformValue(ty: SkSLType) -> bool {
    match ty {
        SkSLType::kFloat
        | SkSLType::kFloat2
        | SkSLType::kFloat3
        | SkSLType::kFloat4
        | SkSLType::kFloat2x2
        | SkSLType::kFloat3x3
        | SkSLType::kFloat4x4
        | SkSLType::kHalf
        | SkSLType::kHalf2
        | SkSLType::kHalf3
        | SkSLType::kHalf4
        | SkSLType::kHalf2x2
        | SkSLType::kHalf3x3
        | SkSLType::kHalf4x4
        | SkSLType::kInt
        | SkSLType::kInt2
        | SkSLType::kInt3
        | SkSLType::kInt4
        | SkSLType::kUInt
        | SkSLType::kUInt2
        | SkSLType::kUInt3
        | SkSLType::kUInt4 => true,
        _ => false,
    }
}
