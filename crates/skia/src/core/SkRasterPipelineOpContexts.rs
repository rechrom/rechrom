// Copyright 2023 Google LLC. BSD-style license.
// Raw pointers describe upstream non-owning arena contexts. They do not allocate,
// free or extend the lifetime of pixel storage; execution must supply valid leases.
// Source: src/core/SkRasterPipelineOpContexts.h:26-32
pub const kMaxStride: usize = 16;
pub const kMaxStride_highp: usize = 16;
pub const kMaxScratchPerPatch: usize = 256;
// Source: src/core/SkRasterPipelineOpContexts.h:35-38
#[repr(C)]
pub struct MemoryCtx {
    pub pixels: *mut core::ffi::c_void,
    pub stride: i32,
}
// Source: src/core/SkRasterPipelineOpContexts.h:55-61
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MemoryCtxInfo {
    pub context: *mut MemoryCtx,
    pub bytesPerPixel: i32,
    pub load: bool,
    pub store: bool,
}
// Source: src/core/SkRasterPipelineOpContexts.h:63-68
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MemoryCtxPatch {
    pub scratch: [u8; kMaxScratchPerPatch],
    pub info: MemoryCtxInfo,
    pub backup: *mut core::ffi::c_void,
}
// Source: src/core/SkRasterPipelineOpContexts.h:70-78
#[repr(C)]
pub struct GatherCtx {
    pub pixels: *const core::ffi::c_void,
    pub stride: i32,
    pub width: f32,
    pub height: f32,
    pub weights: [f32; 16],
    pub roundDownAtInteger: bool,
}
impl GatherCtx {
    pub fn new(
        pixels: *const core::ffi::c_void,
        stride: i32,
        width: f32,
        height: f32,
        weights: [f32; 16],
    ) -> Self {
        Self {
            pixels,
            stride,
            width,
            height,
            weights,
            roundDownAtInteger: false,
        }
    }
}
// Source: src/core/SkRasterPipelineOpContexts.h:81-93
#[repr(C)]
pub struct SamplerCtx {
    pub x: [f32; 16],
    pub y: [f32; 16],
    pub fx: [f32; 16],
    pub fy: [f32; 16],
    pub scalex: [f32; 16],
    pub scaley: [f32; 16],
    pub weights: [f32; 16],
    pub wx: [[f32; 16]; 4],
    pub wy: [[f32; 16]; 4],
}
// Source: src/core/SkRasterPipelineOpContexts.h:95-103
#[repr(C)]
pub struct TileCtx {
    pub scale: f32,
    pub invScale: f32,
    pub mirrorBiasDir: i32,
}
impl TileCtx {
    pub fn new(scale: f32, invScale: f32) -> Self {
        Self {
            scale,
            invScale,
            mirrorBiasDir: -1,
        }
    }
}
// Source: src/core/SkRasterPipelineOpContexts.h:105-114
#[repr(C)]
pub struct DecalTileCtx {
    pub mask: [u32; 16],
    pub limit_x: f32,
    pub limit_y: f32,
    pub inclusiveEdge_x: f32,
    pub inclusiveEdge_y: f32,
}
impl DecalTileCtx {
    pub fn new(mask: [u32; 16], limit_x: f32, limit_y: f32) -> Self {
        Self {
            mask,
            limit_x,
            limit_y,
            inclusiveEdge_x: 0.0,
            inclusiveEdge_y: 0.0,
        }
    }
}
// Source: src/core/SkRasterPipelineOpContexts.h:116-124
#[repr(C)]
pub struct PerlinNoiseCtx {
    pub noiseType: crate::src::shaders::SkPerlinNoiseShaderType::SkPerlinNoiseShaderType,
    pub baseFrequencyX: f32,
    pub baseFrequencyY: f32,
    pub stitchDataInX: f32,
    pub stitchDataInY: f32,
    pub stitching: bool,
    pub numOctaves: i32,
    pub latticeSelector: *const u8,
    pub noiseData: *const u16,
}
// Source: src/core/SkRasterPipelineOpContexts.h:127-142
#[repr(C)]
pub struct MipmapCtx {
    pub x: [f32; 16],
    pub y: [f32; 16],
    pub r: [f32; 16],
    pub g: [f32; 16],
    pub b: [f32; 16],
    pub a: [f32; 16],
    pub scaleX: f32,
    pub scaleY: f32,
    pub lowerWeight: f32,
}
// Source: src/core/SkRasterPipelineOpContexts.h:144-147
#[repr(C)]
pub struct CoordClampCtx {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}
// Source: src/core/SkRasterPipelineOpContexts.h:150-157
// Native uninitialized inactive lanes use MaybeUninit. The function pointer
// is a Rust CPU callback, with no claim of C++ vtable or calling-convention ABI.
#[repr(C)]
pub struct CallbackCtx {
    pub r#fn: unsafe fn(*mut CallbackCtx, i32),
    pub rgba: [core::mem::MaybeUninit<f32>; 4 * kMaxStride_highp],
    pub read_from: *mut f32,
}
impl CallbackCtx {
    // Source: src/core/SkRasterPipelineOpContexts.h:156-156
    // Adapted in-place initialization retains the original self-relative default
    // without creating a movable value whose pointer already targets its old slot.
    /// Safety: place is writable aligned storage for Self at its final address.
    /// Keep that storage and every callback/read_from lease live and stationary
    /// through all deferred execution. Callback sees only4*active_pixels initialized
    /// rgba values; it must leave4*active_pixels initialized readable output values
    /// at read_from, may redirect that pointer to another live buffer, and must not
    /// reset/move/drop the context or its arena while dispatch is in progress.
    pub unsafe fn init_at(place: *mut Self, callback: unsafe fn(*mut Self, i32)) {
        core::ptr::addr_of_mut!((*place).r#fn).write(callback);
        core::ptr::addr_of_mut!((*place).rgba)
            .write([core::mem::MaybeUninit::uninit(); 4 * kMaxStride_highp]);
        let rgba = core::ptr::addr_of_mut!((*place).rgba).cast::<f32>();
        core::ptr::addr_of_mut!((*place).read_from).write(rgba);
    }
}
// Source: src/core/SkRasterPipelineOpContexts.h:159-170
#[repr(C)]
pub struct RewindCtx {
    pub r: [f32; 16],
    pub g: [f32; 16],
    pub b: [f32; 16],
    pub a: [f32; 16],
    pub dr: [f32; 16],
    pub dg: [f32; 16],
    pub db: [f32; 16],
    pub da: [f32; 16],
    pub base: *mut u8,
    pub stage: *mut crate::src::core::SkRasterPipeline::SkRasterPipelineStage,
}
impl RewindCtx {
    pub fn new() -> Self {
        Self {
            r: [0.0; 16],
            g: [0.0; 16],
            b: [0.0; 16],
            a: [0.0; 16],
            dr: [0.0; 16],
            dg: [0.0; 16],
            db: [0.0; 16],
            da: [0.0; 16],
            base: core::ptr::null_mut(),
            stage: core::ptr::null_mut(),
        }
    }
}
// Source: src/core/SkRasterPipelineOpContexts.h:172-184
pub const kRGBAChannels: usize = 4;
#[repr(C)]
pub struct GradientCtx {
    pub stopCount: usize,
    pub factors: [*mut f32; kRGBAChannels],
    pub biases: [*mut f32; kRGBAChannels],
    pub ts: *mut f32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EvenlySpaced2StopGradientCtx {
    pub factor: [f32; kRGBAChannels],
    pub bias: [f32; kRGBAChannels],
}
// Source: src/core/SkRasterPipelineOpContexts.h:186-195
#[repr(C)]
pub struct Conical2PtCtx {
    pub fMask: [u32; 16],
    pub fP0: f32,
    pub fP1: f32,
}
#[repr(C)]
pub struct UniformColorCtx {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
    pub rgba: [u16; 4],
}
// Source: src/core/SkRasterPipelineOpContexts.h:197-203
#[repr(C)]
pub struct EmbossCtx {
    pub mul: MemoryCtx,
    pub add: MemoryCtx,
}
#[repr(C)]
pub struct TablesCtx {
    pub r: *const u8,
    pub g: *const u8,
    pub b: *const u8,
    pub a: *const u8,
}
// Source: src/core/SkRasterPipelineOpContexts.h:205-205
pub type SkRPOffset = u32;
// Source: src/core/SkRasterPipelineOpContexts.h:207-209
#[repr(C)]
pub struct InitLaneMasksCtx {
    pub tail: *mut u8,
}
// Source: src/core/SkRasterPipelineOpContexts.h:211-214
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ConstantCtx {
    pub value: i32,
    pub dst: u32,
}
// Source: src/core/SkRasterPipelineOpContexts.h:216-219
#[repr(C)]
#[derive(Clone, Copy)]
pub struct UniformCtx {
    pub dst: *mut i32,
    pub src: *const i32,
}
// Source: src/core/SkRasterPipelineOpContexts.h:221-224
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BinaryOpCtx {
    pub dst: u32,
    pub src: u32,
}
// Source: src/core/SkRasterPipelineOpContexts.h:226-229
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TernaryOpCtx {
    pub dst: u32,
    pub delta: u32,
}
// Source: src/core/SkRasterPipelineOpContexts.h:231-234
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MatrixMultiplyCtx {
    pub dst: u32,
    pub leftColumns: u8,
    pub leftRows: u8,
    pub rightColumns: u8,
    pub rightRows: u8,
}
// Source: src/core/SkRasterPipelineOpContexts.h:269-275
#[repr(C)]
pub struct BranchCtx {
    pub offset: i32,
}
#[repr(C)]
pub struct BranchIfAllLanesActiveCtx {
    pub branch: BranchCtx,
    pub tail: *mut u8,
}
impl BranchIfAllLanesActiveCtx {
    pub fn new(offset: i32) -> Self {
        Self {
            branch: BranchCtx { offset },
            tail: core::ptr::null_mut(),
        }
    }
}
// Source: src/core/SkRasterPipelineOpContexts.h:236-243
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SwizzleCtx {
    pub dst: u32,
    pub offsets: [u8; 4],
}
const _: () = assert!(kMaxStride_highp <= 16);
// Source: src/core/SkRasterPipelineOpContexts.h:245-249
#[repr(C)]
pub struct ShuffleCtx {
    pub ptr: *mut i32,
    pub count: i32,
    pub offsets: [u16; 16],
}
// Source: src/core/SkRasterPipelineOpContexts.h:251-255
#[repr(C)]
pub struct SwizzleCopyCtx {
    pub dst: *mut i32,
    pub src: *const i32,
    pub offsets: [u16; 4],
}
// Source: src/core/SkRasterPipelineOpContexts.h:257-263
#[repr(C)]
pub struct CopyIndirectCtx {
    pub dst: *mut i32,
    pub src: *const i32,
    pub indirectOffset: *const u32,
    pub indirectLimit: u32,
    pub slots: u32,
}
// Source: src/core/SkRasterPipelineOpContexts.h:265-267
// C++ inheritance is explicit base composition; no C++ object ABI is asserted.
#[repr(C)]
pub struct SwizzleCopyIndirectCtx {
    pub base: CopyIndirectCtx,
    pub offsets: [u16; 4],
}
// Source: src/core/SkRasterPipelineOpContexts.h:277-280
// Explicit base composition replaces C++ inheritance, without an ABI claim.
#[repr(C)]
pub struct BranchIfEqualCtx {
    pub branch: BranchCtx,
    pub value: i32,
    pub ptr: *const i32,
}
// Source: src/core/SkRasterPipelineOpContexts.h:282-285
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CaseOpCtx {
    pub expectedValue: i32,
    pub offset: u32,
}
// Source: src/core/SkRasterPipelineOpContexts.h:287-291
// Hook and data leases must remain exclusive/live through deferred execution.
// Lifetime-bearing Rust trait pointer replaces C++ virtual dispatch (no ABI claim).
pub struct TraceFuncCtx<'hook> {
    pub traceMask: *const i32,
    pub traceHook: *mut (dyn crate::src::sksl::tracing::SkSLTraceHook::TraceHook + 'hook),
    pub funcIdx: i32,
}
// Source: src/core/SkRasterPipelineOpContexts.h:293-297
// Hook and data leases must remain exclusive/live through deferred execution.
// Lifetime-bearing Rust trait pointer replaces C++ virtual dispatch (no ABI claim).
pub struct TraceScopeCtx<'hook> {
    pub traceMask: *const i32,
    pub traceHook: *mut (dyn crate::src::sksl::tracing::SkSLTraceHook::TraceHook + 'hook),
    pub delta: i32,
}
// Source: src/core/SkRasterPipelineOpContexts.h:299-303
// Hook and data leases must remain exclusive/live through deferred execution.
// Lifetime-bearing Rust trait pointer replaces C++ virtual dispatch (no ABI claim).
pub struct TraceLineCtx<'hook> {
    pub traceMask: *const i32,
    pub traceHook: *mut (dyn crate::src::sksl::tracing::SkSLTraceHook::TraceHook + 'hook),
    pub lineNumber: i32,
}
// Source: src/core/SkRasterPipelineOpContexts.h:305-312
// Hook and data leases must remain exclusive/live through deferred execution.
// Lifetime-bearing Rust trait pointer replaces C++ virtual dispatch (no ABI claim).
pub struct TraceVarCtx<'hook> {
    pub traceMask: *const i32,
    pub traceHook: *mut (dyn crate::src::sksl::tracing::SkSLTraceHook::TraceHook + 'hook),
    pub slotIdx: i32,
    pub numSlots: i32,
    pub data: *const i32,
    pub indirectOffset: *const u32,
    pub indirectLimit: u32,
}
