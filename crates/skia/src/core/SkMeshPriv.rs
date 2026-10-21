// Copyright 2021 Google LLC. BSD-style license. Unverified direct translation.
pub use super::SkMesh::ColorType;
use super::SkMesh::{AttributeType, SkMeshSpecification, VaryingType};
use super::SkSLTypeShared::SkSLType;
pub struct SkMeshSpecificationPriv;
impl SkMeshSpecificationPriv {
    // Source: src/core/SkMeshPriv.h:20-40
    pub fn Varyings(spec: &SkMeshSpecification) -> &[super::SkMesh::Varying] {
        &spec.fVaryings
    }
    pub fn VS(spec: &SkMeshSpecification) -> &crate::src::sksl::ir::SkSLProgram::Program {
        &spec.fVS
    }
    pub fn FS(spec: &SkMeshSpecification) -> &crate::src::sksl::ir::SkSLProgram::Program {
        &spec.fFS
    }
    pub fn Hash(spec: &SkMeshSpecification) -> i32 {
        spec.fHash as i32
    }
    pub fn GetColorType(spec: &SkMeshSpecification) -> ColorType {
        spec.fColorType
    }
    pub fn HasColors(spec: &SkMeshSpecification) -> bool {
        Self::GetColorType(spec) != ColorType::kNone
    }
    pub fn ColorSpace(spec: &SkMeshSpecification) -> Option<&super::SkColorSpace::SkColorSpace> {
        spec.fColorSpace.as_deref()
    }
    pub fn AlphaType(spec: &SkMeshSpecification) -> crate::include::core::SkAlphaType::SkAlphaType {
        spec.fAlphaType
    }
    // Source: src/core/SkMeshPriv.h:42-55
    pub fn VaryingTypeAsSLType(t: VaryingType) -> SkSLType {
        match t {
            VaryingType::kFloat => SkSLType::kFloat,
            VaryingType::kFloat2 => SkSLType::kFloat2,
            VaryingType::kFloat3 => SkSLType::kFloat3,
            VaryingType::kFloat4 => SkSLType::kFloat4,
            VaryingType::kHalf => SkSLType::kHalf,
            VaryingType::kHalf2 => SkSLType::kHalf2,
            VaryingType::kHalf3 => SkSLType::kHalf3,
            VaryingType::kHalf4 => SkSLType::kHalf4,
        }
    }
    // Source: src/core/SkMeshPriv.h:57-68
    pub fn AttrTypeByteSize(t: AttributeType) -> usize {
        super::SkMesh::attribute_type_size(t)
    }
    // Source: src/core/SkMeshPriv.h:70-80
    pub fn AttrTypeAsSLType(t: AttributeType) -> SkSLType {
        match t {
            AttributeType::kFloat => SkSLType::kFloat,
            AttributeType::kFloat2 => SkSLType::kFloat2,
            AttributeType::kFloat3 => SkSLType::kFloat3,
            AttributeType::kFloat4 => SkSLType::kFloat4,
            AttributeType::kUByte4_unorm => SkSLType::kHalf4,
        }
    }
    // Source: src/core/SkMeshPriv.h:82-84
    pub fn PassthroughLocalCoordsVaryingIndex(spec: &SkMeshSpecification) -> i32 {
        spec.fPassthroughLocalCoordsVaryingIndex
    }
    // Source: src/core/SkMeshPriv.h:92-95
    pub fn VaryingIsDead(spec: &SkMeshSpecification, v: i32) -> bool {
        debug_assert!(v >= 0 && (v as usize) < spec.fVaryings.len());
        (1u32 << v) & spec.fDeadVaryingMask != 0
    }
}
// Source: src/core/SkMeshPriv.h:98-113
// Abstract native Buffer, including the actual ordinary virtual defaults.
// Dynamic ownership supplies the pure virtual destructor protocol; no base object exists.
pub trait Buffer {
    fn peek(&self) -> *const u8 {
        core::ptr::null()
    }
    fn isGaneshBacked(&self) -> bool {
        false
    }
}
// Source: src/core/SkMeshPriv.h:115-116
pub trait IB: Buffer + super::SkMesh::IndexBuffer {}
pub trait VB: Buffer + super::SkMesh::VertexBuffer {}
// Source: src/core/SkMeshPriv.h:115-133
// Source: src/core/SkMeshPriv.h:139-156
// Two concrete template instantiations retain the original one SkData owner.
macro_rules! cpu_buffer {
    ($name:ident,$base:ident) => {
        pub struct $name {
            fData: std::sync::Arc<super::SkData::SkData>,
        }
        impl $name {
            // Source: src/core/SkMeshPriv.h:139-151
            /// Safety: nonnull data spans size readable bytes for the native memcpy.
            pub unsafe fn Make(data: *const u8, size: usize) -> Self {
                debug_assert!(size != 0);
                let storage = if data.is_null() {
                    super::SkData::SkData::MakeZeroInitialized(size)
                } else {
                    super::SkData::SkData::MakeWithCopy(data, size)
                };
                Self::new(storage)
            }
            // Source: src/core/SkMeshPriv.h:128-128
            fn new(data: std::sync::Arc<super::SkData::SkData>) -> Self {
                Self { fData: data }
            }
        }
        impl Buffer for $name {
            // Source: src/core/SkMeshPriv.h:123-123
            fn peek(&self) -> *const u8 {
                self.fData.data()
            }
        }
        impl super::SkMesh::$base for $name {
            // Source: src/core/SkMeshPriv.h:125-125
            fn size(&self) -> usize {
                self.fData.size()
            }
            // Source: src/core/SkMeshPriv.h:153-156
            /// Safety: caller owns the native exclusive update lease; source/destination
            /// spans are valid and nonoverlapping, after the public update bounds checks.
            unsafe fn onUpdate(
                &self,
                dc: *mut super::SkMesh::GrDirectContext,
                data: *const u8,
                offset: usize,
                size: usize,
            ) -> bool {
                if !dc.is_null() {
                    return false;
                }
                core::ptr::copy_nonoverlapping(data, self.fData.writable_data().add(offset), size);
                true
            }
        }
    };
}
// Source: src/core/SkMeshPriv.h:135-136
cpu_buffer!(CpuIndexBuffer, IndexBuffer);
cpu_buffer!(CpuVertexBuffer, VertexBuffer);

impl IB for CpuIndexBuffer {}
impl VB for CpuVertexBuffer {}
