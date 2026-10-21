// Copyright 2016 Google Inc. BSD-style license.
use crate::include::core::SkColorType::SkColorType::*;
use crate::src::core::SkArenaAlloc::SkArenaAlloc;
use crate::src::core::SkImageInfo::SkColorTypeBytesPerPixel;
use crate::src::core::SkRasterPipelineOpContexts::*;
use crate::src::core::SkRasterPipelineOpList::SkRasterPipelineOp as Op;
use core::ffi::c_void;
use core::ptr;
// Source: src/core/SkRasterPipeline.h:30-37
// explicit_tail_calls supplies the language guarantee rather than relying on
// an optimizer choosing a tail call. Preserve every native excluded target.
// Stage implementations supply the matching native by-value ABI; this constant
// selects only the original tail-call policy and does not dispatch execution.
pub const SK_HAS_MUSTTAIL: bool = cfg!(not(any(
    target_os = "emscripten",
    target_arch = "arm",
    target_arch = "loongarch64",
    target_arch = "powerpc",
    target_arch = "powerpc64",
    all(target_os = "windows", SK_BUILD_FOR_ANDROID_FRAMEWORK)
)));
// Source: src/core/SkRasterPipeline.h:56-68
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SkRasterPipelineStage {
    pub r#fn: crate::src::core::SkOpts::StageFn,
    pub ctx: *mut c_void,
}
// Source: src/core/SkRasterPipeline.h:55-55
// Source: src/core/SkRasterPipeline.h:69-69
const _: () =
    assert!(core::mem::size_of::<SkRasterPipelineStage>() == 2 * core::mem::size_of::<usize>());
// Source: src/core/SkRasterPipeline.h:97-101
#[repr(C)]
#[derive(Clone, Copy)]
pub struct StageList {
    pub prev: *mut StageList,
    pub stage: Op,
    pub ctx: *mut c_void,
}
// Source: src/core/SkRasterPipeline.cpp:29-29
/// Native mutable configuration: initialize before concurrent dispatch or
/// synchronize every access. This mirrors the original non-atomic bool.
pub static mut gForceHighPrecisionRasterPipeline: bool = false;

impl SkRasterPipeline {
    // Source: src/core/SkRasterPipeline.cpp:31-33
    /// Safety: alloc must remain live, unmoved and exclusively synchronized for
    /// every pipeline operation. StageRec shares this same native non-owning arena.
    pub unsafe fn new(alloc: *mut SkArenaAlloc) -> Self {
        let mut pipeline = Self {
            fAlloc: alloc,
            fRewindCtx: ptr::null_mut(),
            fStages: ptr::null_mut(),
            fTailPointer: ptr::null_mut(),
            fNumStages: 0,
            fMemoryCtxInfos: Vec::new(),
        };
        pipeline.reset();
        pipeline
    }
    // Source: src/core/SkRasterPipeline.cpp:35-42
    pub fn reset(&mut self) {
        self.fRewindCtx = ptr::null_mut();
        self.fStages = ptr::null_mut();
        self.fTailPointer = ptr::null_mut();
        self.fNumStages = 0;
        self.fMemoryCtxInfos.clear();
    }
    // Source: src/core/SkRasterPipeline.cpp:44-57
    /// Safety: ctx must have the exact context type required by op and outlive
    /// every execution or extension of this pipeline that retains it.
    /// stack_checkpoint/stack_rewind are internal control operations; use
    /// appendStackRewind so every rewind shares this pipeline's canonical context.
    pub unsafe fn append(&mut self, op: Op, ctx: *mut c_void) {
        // Adapted release-mode rejection of invalid raw control-stage construction.
        assert!(
            !matches!(op, Op::stack_checkpoint | Op::stack_rewind),
            "use appendStackRewind for canonical control stages"
        );
        crate::SkASSERT!(@source b"src/core/SkRasterPipeline.cpp\0".as_ptr(), 45; @condition b"op != Op::uniform_color\0".as_ptr(); op != Op::uniform_color);
        crate::SkASSERT!(@source b"src/core/SkRasterPipeline.cpp\0".as_ptr(), 46; @condition b"op != Op::unbounded_uniform_color\0".as_ptr(); op != Op::unbounded_uniform_color);
        crate::SkASSERT!(@source b"src/core/SkRasterPipeline.cpp\0".as_ptr(), 47; @condition b"op != Op::set_rgb\0".as_ptr(); op != Op::set_rgb);
        crate::SkASSERT!(@source b"src/core/SkRasterPipeline.cpp\0".as_ptr(), 48; @condition b"op != Op::unbounded_set_rgb\0".as_ptr(); op != Op::unbounded_set_rgb);
        crate::SkASSERT!(@source b"src/core/SkRasterPipeline.cpp\0".as_ptr(), 49; @condition b"op != Op::parametric\0".as_ptr(); op != Op::parametric);
        crate::SkASSERT!(@source b"src/core/SkRasterPipeline.cpp\0".as_ptr(), 50; @condition b"op != Op::gamma_\0".as_ptr(); op != Op::gamma_);
        crate::SkASSERT!(@source b"src/core/SkRasterPipeline.cpp\0".as_ptr(), 51; @condition b"op != Op::PQish\0".as_ptr(); op != Op::PQish);
        crate::SkASSERT!(@source b"src/core/SkRasterPipeline.cpp\0".as_ptr(), 52; @condition b"op != Op::HLGish\0".as_ptr(); op != Op::HLGish);
        crate::SkASSERT!(@source b"src/core/SkRasterPipeline.cpp\0".as_ptr(), 53; @condition b"op != Op::HLGinvish\0".as_ptr(); op != Op::HLGinvish);
        crate::SkASSERT!(@source b"src/core/SkRasterPipeline.cpp\0".as_ptr(), 54; @condition b"op != Op::stack_checkpoint\0".as_ptr(); op != Op::stack_checkpoint);
        crate::SkASSERT!(@source b"src/core/SkRasterPipeline.cpp\0".as_ptr(), 55; @condition b"op != Op::stack_rewind\0".as_ptr(); op != Op::stack_rewind);
        self.uncheckedAppend(op, ctx);
    }
    // Source: src/core/SkRasterPipeline.cpp:59-65
    unsafe fn tailPointer(&mut self) -> *mut u8 {
        if self.fTailPointer.is_null() {
            self.fTailPointer = (*self.fAlloc).make(255u8);
        }
        self.fTailPointer
    }
    // Source: src/core/SkRasterPipeline.cpp:67-178
    unsafe fn uncheckedAppend(&mut self, op: Op, ctx: *mut c_void) {
        let mut is_load = false;
        let mut is_store = false;
        let mut ct = kUnknown_SkColorType;
        match op {
            Op::load_a8 | Op::load_a8_dst => {
                ct = kAlpha_8_SkColorType;
                is_load = true;
            }
            Op::store_a8 => {
                ct = kAlpha_8_SkColorType;
                is_store = true;
            }
            Op::load_565 | Op::load_565_dst => {
                ct = kRGB_565_SkColorType;
                is_load = true;
            }
            Op::store_565 => {
                ct = kRGB_565_SkColorType;
                is_store = true;
            }
            Op::load_4444 | Op::load_4444_dst => {
                ct = kARGB_4444_SkColorType;
                is_load = true;
            }
            Op::store_4444 => {
                ct = kARGB_4444_SkColorType;
                is_store = true;
            }
            Op::load_8888 | Op::load_8888_dst => {
                ct = kRGBA_8888_SkColorType;
                is_load = true;
            }
            Op::store_8888 => {
                ct = kRGBA_8888_SkColorType;
                is_store = true;
            }
            Op::load_rg88 | Op::load_rg88_dst => {
                ct = kR8G8_unorm_SkColorType;
                is_load = true;
            }
            Op::store_rg88 => {
                ct = kR8G8_unorm_SkColorType;
                is_store = true;
            }
            Op::load_16161616 | Op::load_16161616_dst => {
                ct = kR16G16B16A16_unorm_SkColorType;
                is_load = true;
            }
            Op::store_16161616 => {
                ct = kR16G16B16A16_unorm_SkColorType;
                is_store = true;
            }
            Op::load_a16 | Op::load_a16_dst => {
                ct = kA16_unorm_SkColorType;
                is_load = true;
            }
            Op::store_a16 => {
                ct = kA16_unorm_SkColorType;
                is_store = true;
            }
            Op::load_r16 | Op::load_r16_dst => {
                ct = kR16_unorm_SkColorType;
                is_load = true;
            }
            Op::store_r16 => {
                ct = kR16_unorm_SkColorType;
                is_store = true;
            }
            Op::load_rg1616 | Op::load_rg1616_dst => {
                ct = kR16G16_unorm_SkColorType;
                is_load = true;
            }
            Op::store_rg1616 => {
                ct = kR16G16_unorm_SkColorType;
                is_store = true;
            }
            Op::load_f16 | Op::load_f16_dst => {
                ct = kRGBA_F16_SkColorType;
                is_load = true;
            }
            Op::store_f16 => {
                ct = kRGBA_F16_SkColorType;
                is_store = true;
            }
            Op::load_af16 | Op::load_af16_dst => {
                ct = kA16_float_SkColorType;
                is_load = true;
            }
            Op::store_af16 => {
                ct = kA16_float_SkColorType;
                is_store = true;
            }
            Op::load_rf16 | Op::load_rf16_dst => {
                ct = kR16_float_SkColorType;
                is_load = true;
            }
            Op::store_rf16 => {
                ct = kR16_float_SkColorType;
                is_store = true;
            }
            Op::load_rgf16 | Op::load_rgf16_dst => {
                ct = kR16G16_float_SkColorType;
                is_load = true;
            }
            Op::store_rgf16 => {
                ct = kR16G16_float_SkColorType;
                is_store = true;
            }
            Op::load_f32 | Op::load_f32_dst => {
                ct = kRGBA_F32_SkColorType;
                is_load = true;
            }
            Op::store_f32 => {
                ct = kRGBA_F32_SkColorType;
                is_store = true;
            }
            Op::load_1010102 | Op::load_1010102_dst => {
                ct = kRGBA_1010102_SkColorType;
                is_load = true;
            }
            Op::store_1010102 => {
                ct = kRGBA_1010102_SkColorType;
                is_store = true;
            }
            Op::load_1010102_xr | Op::load_1010102_xr_dst => {
                ct = kBGR_101010x_XR_SkColorType;
                is_load = true;
            }
            Op::store_1010102_xr => {
                ct = kBGR_101010x_XR_SkColorType;
                is_store = true;
            }
            Op::load_10101010_xr | Op::load_10101010_xr_dst => {
                ct = kBGRA_10101010_XR_SkColorType;
                is_load = true;
            }
            Op::store_10101010_xr => {
                ct = kBGRA_10101010_XR_SkColorType;
                is_store = true;
            }
            Op::load_10x6 | Op::load_10x6_dst => {
                ct = kRGBA_10x6_SkColorType;
                is_load = true;
            }
            Op::store_10x6 => {
                ct = kRGBA_10x6_SkColorType;
                is_store = true;
            }

            Op::debug_r
            | Op::debug_g
            | Op::debug_b
            | Op::debug_a
            | Op::debug_r_255
            | Op::debug_g_255
            | Op::debug_b_255
            | Op::debug_a_255
            | Op::debug_x
            | Op::debug_y => {
                ct = kRGBA_8888_SkColorType;
                is_store = true;
            }
            Op::store_r8 => {
                ct = kR8_unorm_SkColorType;
                is_store = true;
            }
            Op::srcover_rgba_8888 => {
                ct = kRGBA_8888_SkColorType;
                is_load = true;
                is_store = true;
            }
            Op::scale_u8 | Op::lerp_u8 => {
                ct = kAlpha_8_SkColorType;
                is_load = true;
            }
            Op::scale_565 | Op::lerp_565 => {
                ct = kRGB_565_SkColorType;
                is_load = true;
            }
            Op::emboss => {
                let emboss = ctx as *mut EmbossCtx;
                self.addMemoryContext(
                    ptr::addr_of_mut!((*emboss).add),
                    SkColorTypeBytesPerPixel(kAlpha_8_SkColorType),
                    true,
                    false,
                );
                self.addMemoryContext(
                    ptr::addr_of_mut!((*emboss).mul),
                    SkColorTypeBytesPerPixel(kAlpha_8_SkColorType),
                    true,
                    false,
                );
            }
            Op::init_lane_masks => {
                (*(ctx as *mut InitLaneMasksCtx)).tail = self.tailPointer();
            }
            Op::branch_if_all_lanes_active => {
                (*(ctx as *mut BranchIfAllLanesActiveCtx)).tail = self.tailPointer();
            }
            _ => {}
        }
        self.fStages = (*self.fAlloc).make(StageList {
            prev: self.fStages,
            stage: op,
            ctx,
        });
        self.fNumStages += 1;
        if is_load || is_store {
            crate::SkASSERT!(@source b"src/core/SkRasterPipeline.cpp\0".as_ptr(), 172; @condition b"ct != kUnknown_SkColorType\0".as_ptr(); ct != kUnknown_SkColorType);
            self.addMemoryContext(
                ctx as *mut MemoryCtx,
                SkColorTypeBytesPerPixel(ct),
                is_load,
                is_store,
            );
        }
    }
    // Source: src/core/SkRasterPipeline.h:83-83
    /// Native omitted-context argument. The same append stage contract applies.
    pub unsafe fn append_default(&mut self, op: Op) {
        self.append(op, ptr::null_mut());
    }
    // Source: src/core/SkRasterPipeline.h:84-84
    /// Safety: the original context type and lifetime contract of append applies.
    pub unsafe fn append_const(&mut self, op: Op, ctx: *const c_void) {
        self.append(op, ctx.cast_mut());
    }
    // Source: src/core/SkRasterPipeline.h:118-120
    /// Safety: alloc and every deferred pipeline context retain their original leases.
    pub unsafe fn appendConstantColor4f(
        &mut self,
        alloc: *mut SkArenaAlloc,
        color: &crate::src::core::SkColor::SkColor4f,
    ) {
        self.appendConstantColor(alloc, color.vec());
    }
    // Source: src/core/SkRasterPipeline.h:125-127
    /// Safety: alloc and every deferred pipeline context retain their original leases.
    pub unsafe fn appendSetRGB4f(
        &mut self,
        alloc: *mut SkArenaAlloc,
        color: &crate::src::core::SkColor::SkColor4f,
    ) {
        let rgba = color.vec();
        self.appendSetRGB(alloc, &[rgba[0], rgba[1], rgba[2]]);
    }
    // Source: src/core/SkRasterPipeline.cpp:180-184
    pub unsafe fn append_packed(&mut self, op: Op, ctx: usize) {
        self.append(op, ctx as *mut c_void);
    }
    // Source: src/core/SkRasterPipeline.cpp:186-234
    /// Safety: source stages and referenced contexts must remain live for every
    /// operation on the extended pipeline; both arena capabilities must be valid.
    pub unsafe fn extend(&mut self, src: &Self) {
        if src.empty() {
            return;
        }
        // Adapted validation, before allocation/mutation: native extension rewrites
        // all but the oldest stage. Reject noncanonical control graphs rather
        // than silently assigning a second checkpoint context to that last copy.
        let mut check = src.fStages;
        while !check.is_null() {
            assert!(
                (*check).stage != Op::stack_checkpoint,
                "nested checkpoint extension is unsupported"
            );
            if (*check).stage == Op::stack_rewind {
                assert!(
                    !src.fRewindCtx.is_null() && (*check).ctx == src.fRewindCtx.cast(),
                    "source rewind must use its canonical context"
                );
                if (*check).prev.is_null() {
                    assert!(
                        (*check).ctx == self.fRewindCtx.cast(),
                        "oldest rewind cannot be extended across different canonical contexts"
                    );
                }
            }
            check = (*check).prev;
        }
        if !src.fRewindCtx.is_null() && self.fRewindCtx.is_null() {
            self.fRewindCtx = (*self.fAlloc).make(RewindCtx::new());
        }
        let stages = (*self.fAlloc).makeArrayDefault::<StageList>(src.fNumStages as usize);
        let mut n = src.fNumStages;
        let mut st = src.fStages;
        loop {
            n -= 1;
            if n <= 0 {
                break;
            }
            let current = stages.add(n as usize);
            current.write(*st);
            (*current).prev = stages.add(n as usize - 1);
            match (*current).stage {
                Op::stack_rewind => (*current).ctx = self.fRewindCtx as *mut c_void,
                Op::init_lane_masks => {
                    (*((*current).ctx as *mut InitLaneMasksCtx)).tail = self.tailPointer()
                }
                Op::branch_if_all_lanes_active => {
                    (*((*current).ctx as *mut BranchIfAllLanesActiveCtx)).tail = self.tailPointer()
                }
                _ => {}
            }
            st = (*st).prev;
        }
        stages.write(*st);
        (*stages).prev = self.fStages;
        self.fStages = stages.add(src.fNumStages as usize - 1);
        self.fNumStages += src.fNumStages;
        for info in &src.fMemoryCtxInfos {
            self.addMemoryContext(info.context, info.bytesPerPixel, info.load, info.store);
        }
    }
    // Source: src/core/SkRasterPipeline.cpp:236-244
    pub fn GetOpName(op: Op) -> &'static str {
        op.name()
    }
    // Source: src/core/SkRasterPipeline.cpp:246-257
    pub fn dump(&self) {
        unsafe {
            crate::SkDebugf!(@source b"src/core/SkRasterPipeline.cpp\0".as_ptr(), 247; b"SkRasterPipeline, %d stages\n\0".as_ptr(),self.fNumStages as core::ffi::c_int);
        }
        let mut stages = Vec::new();
        let mut st = self.fStages;
        unsafe {
            while !st.is_null() {
                stages.push(Self::GetOpNameC((*st).stage));
                st = (*st).prev;
            }
        }
        for name in stages.into_iter().rev() {
            unsafe {
                crate::SkDebugf!(@source b"src/core/SkRasterPipeline.cpp\0".as_ptr(), 254; b"\t%s\n\0".as_ptr(),name.as_ptr());
            }
        }
        unsafe {
            crate::SkDebugf!(@source b"src/core/SkRasterPipeline.cpp\0".as_ptr(), 256; b"\n\0".as_ptr());
        }
    }
    // Source: src/core/SkRasterPipeline.cpp:259-274
    /// Safety: alloc must be live and shared under the pipeline arena contract.
    pub unsafe fn appendSetRGB(&mut self, alloc: *mut SkArenaAlloc, rgb: &[f32; 3]) {
        let arg = (*alloc).makeArrayDefault::<f32>(3);
        arg.add(0).write(rgb[0]);
        arg.add(1).write(rgb[1]);
        arg.add(2).write(rgb[2]);
        let mut op = Op::unbounded_set_rgb;
        if 0.0 <= rgb[0]
            && rgb[0] <= 1.0
            && 0.0 <= rgb[1]
            && rgb[1] <= 1.0
            && 0.0 <= rgb[2]
            && rgb[2] <= 1.0
        {
            op = Op::set_rgb;
        }
        self.uncheckedAppend(op, arg as *mut c_void);
    }
    // Source: src/core/SkRasterPipeline.cpp:276-305
    /// Safety: alloc must outlive every use of the appended color context.
    pub unsafe fn appendConstantColor(&mut self, alloc: *mut SkArenaAlloc, rgba: &[f32; 4]) {
        crate::SkASSERT!(@source b"src/core/SkRasterPipeline.cpp\0".as_ptr(), 278; @condition b"0 <= rgba[3] && rgba[3] <= 1\0".as_ptr(); 0.0 <= rgba[3] && rgba[3] <= 1.0);
        if rgba[0] == 0.0 && rgba[1] == 0.0 && rgba[2] == 0.0 && rgba[3] == 1.0 {
            self.append_default(Op::black_color);
        } else if rgba[0] == 1.0 && rgba[1] == 1.0 && rgba[2] == 1.0 && rgba[3] == 1.0 {
            self.append_default(Op::white_color);
        } else {
            // rgba lowp slots are unused by unbounded_uniform_color. Zero
            // initializes those otherwise unspecified slots for Rust validity.
            let ctx = (*alloc).make(UniformColorCtx {
                r: rgba[0],
                g: rgba[1],
                b: rgba[2],
                a: rgba[3],
                rgba: [0; 4],
            });
            if 0.0 <= rgba[0]
                && rgba[0] <= rgba[3]
                && 0.0 <= rgba[1]
                && rgba[1] <= rgba[3]
                && 0.0 <= rgba[2]
                && rgba[2] <= rgba[3]
            {
                (*ctx).rgba[0] = (rgba[0] * 255.0 + 0.5) as u16;
                (*ctx).rgba[1] = (rgba[1] * 255.0 + 0.5) as u16;
                (*ctx).rgba[2] = (rgba[2] * 255.0 + 0.5) as u16;
                (*ctx).rgba[3] = (rgba[3] * 255.0 + 0.5) as u16;
                self.uncheckedAppend(Op::uniform_color, ctx as *mut c_void);
            } else {
                self.uncheckedAppend(Op::unbounded_uniform_color, ctx as *mut c_void);
            }
        }
    }
    // Source: src/core/SkRasterPipeline.cpp:307-336
    /// Safety: alloc must outlive every use of the matrix stage context.
    pub unsafe fn appendMatrix(
        &mut self,
        alloc: *mut SkArenaAlloc,
        matrix: &crate::src::core::SkMatrix::SkMatrix,
    ) {
        use crate::src::core::SkMatrix::SkMatrix;
        let mt = matrix.getType();
        if mt == SkMatrix::kIdentity_Mask {
            return;
        }
        if mt == SkMatrix::kTranslate_Mask {
            let trans = (*alloc).makeArrayDefault::<f32>(2);
            trans.add(0).write(matrix.getTranslateX());
            trans.add(1).write(matrix.getTranslateY());
            self.append(Op::matrix_translate, trans as *mut c_void);
        } else if mt | (SkMatrix::kScale_Mask | SkMatrix::kTranslate_Mask)
            == SkMatrix::kScale_Mask | SkMatrix::kTranslate_Mask
        {
            let scale_trans = (*alloc).makeArrayDefault::<f32>(4);
            scale_trans.add(0).write(matrix.getScaleX());
            scale_trans.add(1).write(matrix.getScaleY());
            scale_trans.add(2).write(matrix.getTranslateX());
            scale_trans.add(3).write(matrix.getTranslateY());
            self.append(Op::matrix_scale_translate, scale_trans as *mut c_void);
        } else {
            let storage = (*alloc).makeArrayDefault::<f32>(9);
            let mut values = [0.0f32; 9];
            matrix.get9(&mut values);
            for (i, value) in values.into_iter().enumerate() {
                storage.add(i).write(value);
            }
            if !matrix.hasPerspective() {
                self.append(Op::matrix_2x3, storage as *mut c_void);
            } else {
                self.append(Op::matrix_perspective, storage as *mut c_void);
            }
        }
    }
    // Source: src/core/SkRasterPipeline.cpp:338-406
    /// Safety: ctx and its writable pixel memory must satisfy each appended
    /// operation's layout and remain live through pipeline execution/extensions.
    pub unsafe fn appendLoad(
        &mut self,
        ct: crate::include::core::SkColorType::SkColorType,
        ctx: *mut MemoryCtx,
    ) {
        match ct {
            kUnknown_SkColorType => {
                crate::SkASSERT!(@source b"src/core/SkRasterPipeline.cpp\0".as_ptr(), 340; @condition b"false\0".as_ptr(); false);
            }
            kAlpha_8_SkColorType => {
                self.append(Op::load_a8, ctx.cast());
            }
            kA16_unorm_SkColorType => {
                self.append(Op::load_a16, ctx.cast());
            }
            kA16_float_SkColorType => {
                self.append(Op::load_af16, ctx.cast());
            }
            kRGB_565_SkColorType => {
                self.append(Op::load_565, ctx.cast());
            }
            kARGB_4444_SkColorType => {
                self.append(Op::load_4444, ctx.cast());
            }
            kR8G8_unorm_SkColorType => {
                self.append(Op::load_rg88, ctx.cast());
            }
            kR16_unorm_SkColorType => {
                self.append(Op::load_r16, ctx.cast());
            }
            kR16_float_SkColorType => {
                self.append(Op::load_rf16, ctx.cast());
            }
            kR16G16_unorm_SkColorType => {
                self.append(Op::load_rg1616, ctx.cast());
            }
            kR16G16_float_SkColorType => {
                self.append(Op::load_rgf16, ctx.cast());
            }
            kRGBA_8888_SkColorType => {
                self.append(Op::load_8888, ctx.cast());
            }
            kRGBA_1010102_SkColorType => {
                self.append(Op::load_1010102, ctx.cast());
            }
            kR16G16B16A16_unorm_SkColorType => {
                self.append(Op::load_16161616, ctx.cast());
            }
            kRGBA_F16Norm_SkColorType | kRGBA_F16_SkColorType => {
                self.append(Op::load_f16, ctx.cast());
            }
            kRGBA_F32_SkColorType => {
                self.append(Op::load_f32, ctx.cast());
            }
            kRGBA_10x6_SkColorType => {
                self.append(Op::load_10x6, ctx.cast());
            }
            kGray_8_SkColorType => {
                self.append(Op::load_a8, ctx.cast());
                self.append(Op::alpha_to_gray, ptr::null_mut());
            }
            kR8_unorm_SkColorType => {
                self.append(Op::load_a8, ctx.cast());
                self.append(Op::alpha_to_red, ptr::null_mut());
            }
            kRGB_888x_SkColorType => {
                self.append(Op::load_8888, ctx.cast());
                self.append(Op::force_opaque, ptr::null_mut());
            }
            kBGRA_1010102_SkColorType => {
                self.append(Op::load_1010102, ctx.cast());
                self.append(Op::swap_rb, ptr::null_mut());
            }
            kRGB_101010x_SkColorType => {
                self.append(Op::load_1010102, ctx.cast());
                self.append(Op::force_opaque, ptr::null_mut());
            }
            kBGR_101010x_SkColorType => {
                self.append(Op::load_1010102, ctx.cast());
                self.append(Op::force_opaque, ptr::null_mut());
                self.append(Op::swap_rb, ptr::null_mut());
            }
            kBGRA_10101010_XR_SkColorType => {
                self.append(Op::load_10101010_xr, ctx.cast());
                self.append(Op::swap_rb, ptr::null_mut());
            }
            kBGR_101010x_XR_SkColorType => {
                self.append(Op::load_1010102_xr, ctx.cast());
                self.append(Op::force_opaque, ptr::null_mut());
                self.append(Op::swap_rb, ptr::null_mut());
            }
            kRGB_F16F16F16x_SkColorType => {
                self.append(Op::load_f16, ctx.cast());
                self.append(Op::force_opaque, ptr::null_mut());
            }
            kBGRA_8888_SkColorType => {
                self.append(Op::load_8888, ctx.cast());
                self.append(Op::swap_rb, ptr::null_mut());
            }
            kSRGBA_8888_SkColorType => {
                self.append(Op::load_8888, ctx.cast());
                self.appendTransferFunction(
                    crate::modules::skcms::skcms::skcms_sRGB_TransferFunction(),
                );
            }
        }
    }
    // Source: src/core/SkRasterPipeline.cpp:408-480
    /// Safety: ctx and its writable pixel memory must satisfy each appended
    /// operation's layout and remain live through pipeline execution/extensions.
    pub unsafe fn appendLoadDst(
        &mut self,
        ct: crate::include::core::SkColorType::SkColorType,
        ctx: *mut MemoryCtx,
    ) {
        match ct {
            kUnknown_SkColorType => {
                crate::SkASSERT!(@source b"src/core/SkRasterPipeline.cpp\0".as_ptr(), 411; @condition b"false\0".as_ptr(); false);
            }
            kAlpha_8_SkColorType => {
                self.append(Op::load_a8_dst, ctx.cast());
            }
            kA16_unorm_SkColorType => {
                self.append(Op::load_a16_dst, ctx.cast());
            }
            kA16_float_SkColorType => {
                self.append(Op::load_af16_dst, ctx.cast());
            }
            kRGB_565_SkColorType => {
                self.append(Op::load_565_dst, ctx.cast());
            }
            kARGB_4444_SkColorType => {
                self.append(Op::load_4444_dst, ctx.cast());
            }
            kR8G8_unorm_SkColorType => {
                self.append(Op::load_rg88_dst, ctx.cast());
            }
            kR16_unorm_SkColorType => {
                self.append(Op::load_r16_dst, ctx.cast());
            }
            kR16_float_SkColorType => {
                self.append(Op::load_rf16_dst, ctx.cast());
            }
            kR16G16_unorm_SkColorType => {
                self.append(Op::load_rg1616_dst, ctx.cast());
            }
            kR16G16_float_SkColorType => {
                self.append(Op::load_rgf16_dst, ctx.cast());
            }
            kRGBA_8888_SkColorType => {
                self.append(Op::load_8888_dst, ctx.cast());
            }
            kRGBA_1010102_SkColorType => {
                self.append(Op::load_1010102_dst, ctx.cast());
            }
            kR16G16B16A16_unorm_SkColorType => {
                self.append(Op::load_16161616_dst, ctx.cast());
            }
            kRGBA_F16Norm_SkColorType | kRGBA_F16_SkColorType => {
                self.append(Op::load_f16_dst, ctx.cast());
            }
            kRGBA_F32_SkColorType => {
                self.append(Op::load_f32_dst, ctx.cast());
            }
            kRGBA_10x6_SkColorType => {
                self.append(Op::load_10x6_dst, ctx.cast());
            }
            kGray_8_SkColorType => {
                self.append(Op::load_a8_dst, ctx.cast());
                self.append(Op::alpha_to_gray_dst, ptr::null_mut());
            }
            kR8_unorm_SkColorType => {
                self.append(Op::load_a8_dst, ctx.cast());
                self.append(Op::alpha_to_red_dst, ptr::null_mut());
            }
            kRGB_888x_SkColorType => {
                self.append(Op::load_8888_dst, ctx.cast());
                self.append(Op::force_opaque_dst, ptr::null_mut());
            }
            kBGRA_1010102_SkColorType => {
                self.append(Op::load_1010102_dst, ctx.cast());
                self.append(Op::swap_rb_dst, ptr::null_mut());
            }
            kRGB_101010x_SkColorType => {
                self.append(Op::load_1010102_dst, ctx.cast());
                self.append(Op::force_opaque_dst, ptr::null_mut());
            }
            kBGR_101010x_SkColorType => {
                self.append(Op::load_1010102_dst, ctx.cast());
                self.append(Op::force_opaque_dst, ptr::null_mut());
                self.append(Op::swap_rb_dst, ptr::null_mut());
            }
            kBGR_101010x_XR_SkColorType => {
                self.append(Op::load_1010102_xr_dst, ctx.cast());
                self.append(Op::force_opaque_dst, ptr::null_mut());
                self.append(Op::swap_rb_dst, ptr::null_mut());
            }
            kBGRA_10101010_XR_SkColorType => {
                self.append(Op::load_10101010_xr_dst, ctx.cast());
                self.append(Op::swap_rb_dst, ptr::null_mut());
            }
            kRGB_F16F16F16x_SkColorType => {
                self.append(Op::load_f16_dst, ctx.cast());
                self.append(Op::force_opaque_dst, ptr::null_mut());
            }
            kBGRA_8888_SkColorType => {
                self.append(Op::load_8888_dst, ctx.cast());
                self.append(Op::swap_rb_dst, ptr::null_mut());
            }
            kSRGBA_8888_SkColorType => {
                self.append(Op::load_8888_dst, ctx.cast());
                self.append(Op::swap_src_dst, ptr::null_mut());
                self.appendTransferFunction(
                    crate::modules::skcms::skcms::skcms_sRGB_TransferFunction(),
                );
                self.append(Op::swap_src_dst, ptr::null_mut());
            }
        }
    }
    // Source: src/core/SkRasterPipeline.cpp:482-547
    /// Safety: ctx and its writable pixel memory must satisfy each appended
    /// operation's layout and remain live through pipeline execution/extensions.
    pub unsafe fn appendStore(
        &mut self,
        ct: crate::include::core::SkColorType::SkColorType,
        ctx: *mut MemoryCtx,
    ) {
        match ct {
            kUnknown_SkColorType => {
                crate::SkASSERT!(@source b"src/core/SkRasterPipeline.cpp\0".as_ptr(), 484; @condition b"false\0".as_ptr(); false);
            }
            kAlpha_8_SkColorType => {
                self.append(Op::store_a8, ctx.cast());
            }
            kR8_unorm_SkColorType => {
                self.append(Op::store_r8, ctx.cast());
            }
            kA16_unorm_SkColorType => {
                self.append(Op::store_a16, ctx.cast());
            }
            kA16_float_SkColorType => {
                self.append(Op::store_af16, ctx.cast());
            }
            kRGB_565_SkColorType => {
                self.append(Op::store_565, ctx.cast());
            }
            kARGB_4444_SkColorType => {
                self.append(Op::store_4444, ctx.cast());
            }
            kR8G8_unorm_SkColorType => {
                self.append(Op::store_rg88, ctx.cast());
            }
            kR16_unorm_SkColorType => {
                self.append(Op::store_r16, ctx.cast());
            }
            kR16_float_SkColorType => {
                self.append(Op::store_rf16, ctx.cast());
            }
            kR16G16_unorm_SkColorType => {
                self.append(Op::store_rg1616, ctx.cast());
            }
            kR16G16_float_SkColorType => {
                self.append(Op::store_rgf16, ctx.cast());
            }
            kRGBA_8888_SkColorType => {
                self.append(Op::store_8888, ctx.cast());
            }
            kRGBA_1010102_SkColorType => {
                self.append(Op::store_1010102, ctx.cast());
            }
            kR16G16B16A16_unorm_SkColorType => {
                self.append(Op::store_16161616, ctx.cast());
            }
            kRGBA_F16Norm_SkColorType | kRGBA_F16_SkColorType => {
                self.append(Op::store_f16, ctx.cast());
            }
            kRGBA_F32_SkColorType => {
                self.append(Op::store_f32, ctx.cast());
            }
            kRGBA_10x6_SkColorType => {
                self.append(Op::store_10x6, ctx.cast());
            }
            kRGB_888x_SkColorType => {
                self.append(Op::force_opaque, ptr::null_mut());
                self.append(Op::store_8888, ctx.cast());
            }
            kBGRA_1010102_SkColorType => {
                self.append(Op::swap_rb, ptr::null_mut());
                self.append(Op::store_1010102, ctx.cast());
            }
            kRGB_101010x_SkColorType => {
                self.append(Op::force_opaque, ptr::null_mut());
                self.append(Op::store_1010102, ctx.cast());
            }
            kBGR_101010x_SkColorType => {
                self.append(Op::force_opaque, ptr::null_mut());
                self.append(Op::swap_rb, ptr::null_mut());
                self.append(Op::store_1010102, ctx.cast());
            }
            kBGR_101010x_XR_SkColorType => {
                self.append(Op::force_opaque, ptr::null_mut());
                self.append(Op::swap_rb, ptr::null_mut());
                self.append(Op::store_1010102_xr, ctx.cast());
            }
            kRGB_F16F16F16x_SkColorType => {
                self.append(Op::force_opaque, ptr::null_mut());
                self.append(Op::store_f16, ctx.cast());
            }
            kBGRA_10101010_XR_SkColorType => {
                self.append(Op::swap_rb, ptr::null_mut());
                self.append(Op::store_10101010_xr, ctx.cast());
            }
            kGray_8_SkColorType => {
                self.append(Op::bt709_luminance_or_luma_to_alpha, ptr::null_mut());
                self.append(Op::store_a8, ctx.cast());
            }
            kBGRA_8888_SkColorType => {
                self.append(Op::swap_rb, ptr::null_mut());
                self.append(Op::store_8888, ctx.cast());
            }
            kSRGBA_8888_SkColorType => {
                self.appendTransferFunction(
                    crate::modules::skcms::skcms::skcms_sRGB_Inverse_TransferFunction(),
                );
                self.append(Op::store_8888, ctx.cast());
            }
        }
    }
    // Source: src/core/SkRasterPipeline.cpp:549-565
    /// Safety: tf must remain valid and unmoved through every pipeline execution
    /// and extension retaining its context pointer.
    pub unsafe fn appendTransferFunction(
        &mut self,
        tf: &crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
    ) {
        use crate::modules::skcms::src::skcms_public::skcms_TFType::*;
        let ctx = (tf as *const _ as *mut c_void);
        match crate::modules::skcms::skcms::skcms_TransferFunction_getType(tf) {
            skcms_TFType_sRGBish => {
                if tf.a == 1.0
                    && tf.b == 0.0
                    && tf.c == 0.0
                    && tf.d == 0.0
                    && tf.e == 0.0
                    && tf.f == 0.0
                {
                    self.uncheckedAppend(Op::gamma_, ctx);
                } else {
                    self.uncheckedAppend(Op::parametric, ctx);
                }
            }
            skcms_TFType_PQish => self.uncheckedAppend(Op::PQish, ctx),
            skcms_TFType_HLGish => self.uncheckedAppend(Op::HLGish, ctx),
            skcms_TFType_HLGinvish => self.uncheckedAppend(Op::HLGinvish, ctx),
            _ => {
                crate::SkASSERT!(@source b"src/core/SkRasterPipeline.cpp\0".as_ptr(), 552; @condition b"false\0".as_ptr(); false)
            }
        }
    }
    // Source: src/core/SkRasterPipeline.cpp:569-573
    pub unsafe fn appendClampIfNormalized(
        &mut self,
        info: &crate::src::core::SkImageInfo::SkImageInfo,
    ) {
        if crate::src::core::SkImageInfoPriv::SkColorTypeIsNormalized(info.colorType()) {
            self.uncheckedAppend(Op::clamp_01, ptr::null_mut());
        }
    }
    // Source: src/core/SkRasterPipeline.cpp:575-580
    pub unsafe fn appendStackRewind(&mut self) {
        if self.fRewindCtx.is_null() {
            self.fRewindCtx = (*self.fAlloc).make(RewindCtx::new());
        }
        self.uncheckedAppend(Op::stack_rewind, self.fRewindCtx as *mut c_void);
    }
}
// Source: src/core/SkRasterPipeline.cpp:582-586
unsafe fn prepend_to_pipeline(
    ip: &mut *mut SkRasterPipelineStage,
    stage_fn: crate::src::core::SkOpts::StageFn,
    ctx: *mut c_void,
) {
    *ip = (*ip).sub(1);
    (*ip).write(SkRasterPipelineStage {
        r#fn: stage_fn,
        ctx,
    });
}
impl SkRasterPipeline {
    // Source: src/core/SkRasterPipeline.cpp:588-604
    unsafe fn buildLowpPipeline(&self, mut ip: *mut SkRasterPipelineStage) -> bool {
        use crate::src::core::SkOpts;
        if unsafe { gForceHighPrecisionRasterPipeline } || !self.fRewindCtx.is_null() {
            return false;
        }
        let Some(terminal) = *SkOpts::just_return_lowp else {
            return false;
        };
        unsafe {
            prepend_to_pipeline(&mut ip, terminal, ptr::null_mut());
        }
        let mut st = self.fStages;
        while !st.is_null() {
            let index = unsafe { (*st).stage as usize };
            // Native program fallback is whole-program: partially emitted lowp
            // cells are overwritten by highp from the original end pointer.
            if index >= crate::src::core::SkRasterPipelineOpList::kNumRasterPipelineLowpOps {
                return false;
            }
            let Some(stage_fn) = SkOpts::ops_lowp[index] else {
                return false;
            };
            unsafe {
                prepend_to_pipeline(&mut ip, stage_fn, (*st).ctx);
                st = (*st).prev;
            }
        }
        true
    }
    // Source: src/core/SkRasterPipeline.cpp:606-623
    unsafe fn buildHighpPipeline(
        &self,
        mut ip: *mut SkRasterPipelineStage,
    ) -> Result<(), MissingStage> {
        use crate::src::core::SkOpts;
        unsafe {
            prepend_to_pipeline(&mut ip, *SkOpts::just_return_highp, ptr::null_mut());
        }
        let mut st = self.fStages;
        while !st.is_null() {
            let op = unsafe { (*st).stage };
            // Rust's explicit translation boundary: an absent actual producer
            // is an error, never an executable tag or State-entry wrapper.
            let Some(stage_fn) = SkOpts::ops_highp[op as usize] else {
                return Err(MissingStage { op });
            };
            unsafe {
                prepend_to_pipeline(&mut ip, stage_fn, (*st).ctx);
                st = (*st).prev;
            }
        }
        if !self.fRewindCtx.is_null() {
            let op = Op::stack_checkpoint;
            let Some(stage_fn) = SkOpts::ops_highp[op as usize] else {
                return Err(MissingStage { op });
            };
            unsafe {
                prepend_to_pipeline(&mut ip, stage_fn, self.fRewindCtx.cast());
            }
        }
        Ok(())
    }
    // Source: src/core/SkRasterPipeline.cpp:625-632
    unsafe fn buildPipeline(
        &self,
        ip: *mut SkRasterPipelineStage,
    ) -> Result<crate::src::core::SkOpts::StartPipelineFn, MissingStage> {
        use crate::src::core::SkOpts;
        if unsafe { self.buildLowpPipeline(ip) } {
            // The table's matching selected start routine receives the exact
            // native program/context cells and patch/tail arguments below.
            if let Some(start) = *SkOpts::start_pipeline_lowp {
                return Ok(start);
            }
        }
        unsafe { self.buildHighpPipeline(ip) }?;
        Ok(*SkOpts::start_pipeline_highp)
    }
    // Source: src/core/SkRasterPipeline.cpp:634-643
    fn stagesNeeded(&self) -> usize {
        let mut stages = self.fNumStages as usize + 1;
        if !self.fRewindCtx.is_null() {
            stages += 1;
        }
        stages
    }
    // Rust transition boundary, not an upstream implementation claim. Check
    // the whole native list before allocation, execution, or any pixel writes.
    /// Checks only opcode availability in the actual stage list and any implicit
    /// stack checkpoint. It does not allocate a program, execute stages, touch
    /// contexts/pixels, or retain a second stage-list snapshot.
    ///
    /// This is an adapted transition boundary while the native stage table is
    /// incomplete; it is not counted as translated upstream source. The
    /// exact selected native function tables drive availability and emission.
    ///
    /// # Safety
    /// The pipeline's arena and linked stage nodes must remain live and immutable
    /// during traversal. This check does not establish context or pixel leases.
    /// A successful result remains applicable only while the stage list and the
    /// immutable CPU dispatch table are unchanged.
    pub unsafe fn validate_executable(&self) -> Result<(), MissingStage> {
        use crate::src::core::SkOpts;
        if self.empty() {
            return Ok(());
        }
        if !unsafe { gForceHighPrecisionRasterPipeline }
            && self.fRewindCtx.is_null()
            && SkOpts::just_return_lowp.is_some()
            && SkOpts::start_pipeline_lowp.is_some()
        {
            let mut st = self.fStages;
            let mut supported = true;
            while !st.is_null() {
                let op = unsafe { (*st).stage as usize };
                if op >= crate::src::core::SkRasterPipelineOpList::kNumRasterPipelineLowpOps
                    || SkOpts::ops_lowp[op].is_none()
                {
                    supported = false;
                    break;
                }
                st = unsafe { (*st).prev };
            }
            if supported {
                return Ok(());
            }
        }
        let mut st = self.fStages;
        while !st.is_null() {
            let op = unsafe { (*st).stage };
            if SkOpts::ops_highp[op as usize].is_none() {
                return Err(MissingStage { op });
            }
            st = unsafe { (*st).prev };
        }
        if !self.fRewindCtx.is_null() && SkOpts::ops_highp[Op::stack_checkpoint as usize].is_none()
        {
            return Err(MissingStage {
                op: Op::stack_checkpoint,
            });
        }
        Ok(())
    }
    // Source: src/core/SkRasterPipeline.cpp:645-667
    /// Safety: every appended context and all referenced pixel memory must
    /// remain live and satisfy its stage's read/write bounds for this rectangle.
    /// Branch offsets must target live stages within this program allocation;
    /// checkpoint/rewind must use the canonical control graph. Calls sharing
    /// mutable contexts must be synchronized. Missing translations
    /// return before the program executes or modifies any destination bytes.
    pub unsafe fn run(&self, x: usize, y: usize, w: usize, h: usize) -> Result<(), MissingStage> {
        if self.empty() {
            return Ok(());
        }
        let stages_needed = self.stagesNeeded();
        let mut program = PipelineScratch::<SkRasterPipelineStage, 32>::new(stages_needed);
        let num_memory_ctxs = self.fMemoryCtxInfos.len();
        let mut patches = PipelineScratch::<MemoryCtxPatch, 2>::new(num_memory_ctxs);
        for i in 0..num_memory_ctxs {
            patches.as_mut_ptr().add(i).write(MemoryCtxPatch {
                info: self.fMemoryCtxInfos[i],
                backup: ptr::null_mut(),
                scratch: [0; kMaxScratchPerPatch],
            });
        }
        let program_ptr = program.as_mut_ptr();
        let start_pipeline = self.buildPipeline(program_ptr.add(stages_needed))?;
        let patches = core::slice::from_raw_parts_mut(patches.as_mut_ptr(), num_memory_ctxs);
        start_pipeline(
            x,
            y,
            x.wrapping_add(w),
            y.wrapping_add(h),
            program_ptr,
            patches,
            self.fTailPointer,
        );
        Ok(())
    }
    // Source: src/core/SkRasterPipeline.cpp:669-694
    /// Safety: arena, stage nodes and context storage must outlive the compiled
    /// program. Contexts retain native non-owning pointers. Pixel backing spans
    /// need only be live/exclusive for each synchronous run: refresh MemoryCtx
    /// pixels before that run and satisfy its bounds. No run may observe a stale
    /// pointer from a previous stack mask. Dropping this program does not access
    /// pixels. Synchronize context mutation, execution and arena reset/destruction.
    pub unsafe fn compile(&self) -> Result<CompiledPipeline<'_>, MissingStage> {
        self.compile_for_arena(self.fAlloc)
    }
    // Source: src/core/SkRasterPipeline.cpp:669-694
    /// Safety: 'arena must be bounded by an existing, actually live owner of
    /// this same arena, its stage nodes and all retained context storage.
    /// This does not prolong any allocation's lifetime. Prevent arena reset/drop
    /// for the returned program lifetime; serialize context mutation and execution.
    /// Pixel spans may change between runs and need not survive program drop.
    /// Before every run, update any expired MemoryCtx pixels to current live,
    /// exclusively leased spans that satisfy the actual stage bounds. Contexts
    /// and all other stage-owned resources remain live through program drop.
    /// Pointer equality checks identity, not actual storage or lease lifetime.
    /// The compiled program may outlive this local pipeline object, as in the
    /// native arena-backed std::function capture. No static lifetime is created.
    pub unsafe fn compile_for_arena<'arena>(
        &self,
        arena: *mut SkArenaAlloc,
    ) -> Result<CompiledPipeline<'arena>, MissingStage> {
        assert!(!arena.is_null());
        assert_eq!(
            arena, self.fAlloc,
            "compiled program must use its actual stage arena"
        );

        if self.empty() {
            return Ok(CompiledPipeline {
                program: ptr::null_mut(),
                patches: ptr::null_mut(),
                num_memory_ctxs: 0,
                tail_pointer: ptr::null_mut(),
                start_pipeline: *crate::src::core::SkOpts::start_pipeline_highp,
                _owner: core::marker::PhantomData,
            });
        }
        let stages_needed = self.stagesNeeded();
        let program = (*arena).makeArrayDefault::<SkRasterPipelineStage>(stages_needed);
        let num_memory_ctxs = self.fMemoryCtxInfos.len();
        let patches = (*arena).makeArrayDefault::<MemoryCtxPatch>(num_memory_ctxs);
        for i in 0..num_memory_ctxs {
            patches.add(i).write(MemoryCtxPatch {
                info: self.fMemoryCtxInfos[i],
                backup: ptr::null_mut(),
                scratch: [0; kMaxScratchPerPatch],
            });
        }
        let tail_pointer = self.fTailPointer;
        let start_pipeline = self.buildPipeline(program.add(stages_needed))?;
        Ok(CompiledPipeline {
            program,
            patches,
            num_memory_ctxs,
            tail_pointer,
            start_pipeline,
            _owner: core::marker::PhantomData,
        })
    }
    // Source: src/core/SkRasterPipeline.cpp:696-712
    fn addMemoryContext(
        &mut self,
        ctx: *mut MemoryCtx,
        bytes_per_pixel: i32,
        load: bool,
        store: bool,
    ) {
        if let Some(info) = self
            .fMemoryCtxInfos
            .iter_mut()
            .find(|info| info.context == ctx)
        {
            crate::SkASSERT!(@source b"src/core/SkRasterPipeline.cpp\0".as_ptr(), 705; @condition b"bytesPerPixel == info->bytesPerPixel\0".as_ptr(); (bytes_per_pixel) == (info.bytesPerPixel));
            info.load |= load;
            info.store |= store;
        } else {
            self.fMemoryCtxInfos.push(MemoryCtxInfo {
                context: ctx,
                bytesPerPixel: bytes_per_pixel,
                load,
                store,
            });
        }
    }
    // Source: src/core/SkRasterPipeline.h:104-105
    pub fn getStageList(&self) -> *const StageList {
        self.fStages
    }
    pub fn getNumStages(&self) -> i32 {
        self.fNumStages
    }
    // Source: src/core/SkRasterPipeline.h:139-139
    pub fn empty(&self) -> bool {
        self.fStages.is_null()
    }
    pub fn memory_contexts(&self) -> &[MemoryCtxInfo] {
        &self.fMemoryCtxInfos
    }
}
// Source: src/core/SkRasterPipeline.h:160-170
pub struct SkRasterPipeline {
    fAlloc: *mut SkArenaAlloc,
    fRewindCtx: *mut RewindCtx,
    fStages: *mut StageList,
    fTailPointer: *mut u8,
    fNumStages: i32,
    fMemoryCtxInfos: Vec<MemoryCtxInfo>,
}

/// Temporary typed error while the remaining scoped stages are translated.
/// The full chain is validated before any pixel execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MissingStage {
    pub op: Op,
}
impl core::fmt::Display for MissingStage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "SkRasterPipeline stage {:?} is pending translation",
            self.op
        )
    }
}
impl std::error::Error for MissingStage {}
// Rust representation of the original arena-backed std::function capture.
// It retains the program and patches, and prevents safe shared concurrent calls.
pub struct CompiledPipeline<'a> {
    program: *mut SkRasterPipelineStage,
    patches: *mut MemoryCtxPatch,
    num_memory_ctxs: usize,
    tail_pointer: *mut u8,
    start_pipeline: crate::src::core::SkOpts::StartPipelineFn,
    _owner: core::marker::PhantomData<&'a SkArenaAlloc>,
}
impl CompiledPipeline<'_> {
    // Source: src/core/SkRasterPipeline.cpp:688-693
    /// Safety: retain compile() arena/stage/context leases. Before this call,
    /// refresh MemoryCtx pixel pointers whose previous backing expired. Every
    /// accessed pixel span must be live, exclusive and satisfy stage bounds for
    /// this rectangle until this synchronous call returns. Serialize context
    /// mutation and shared external contexts; program Drop does not read pixels.
    pub unsafe fn run(&mut self, x: usize, y: usize, w: usize, h: usize) {
        if self.program.is_null() {
            return;
        }
        let patches = if self.num_memory_ctxs == 0 {
            &mut []
        } else {
            core::slice::from_raw_parts_mut(self.patches, self.num_memory_ctxs)
        };
        (self.start_pipeline)(
            x,
            y,
            x.wrapping_add(w),
            y.wrapping_add(h),
            self.program,
            patches,
            self.tail_pointer,
        );
    }
}
// Rust storage substitution for native AutoSTMalloc: preserve the stack
// thresholds, allocate only the overflow case, and never materialize invalid T.
enum PipelineScratch<T: Copy, const N: usize> {
    Inline([core::mem::MaybeUninit<T>; N]),
    Heap(Vec<core::mem::MaybeUninit<T>>),
}
impl<T: Copy, const N: usize> PipelineScratch<T, N> {
    fn new(count: usize) -> Self {
        if count <= N {
            Self::Inline([core::mem::MaybeUninit::uninit(); N])
        } else {
            let mut heap = Vec::with_capacity(count);
            heap.resize_with(count, core::mem::MaybeUninit::uninit);
            Self::Heap(heap)
        }
    }
    fn as_mut_ptr(&mut self) -> *mut T {
        match self {
            Self::Inline(a) => a.as_mut_ptr().cast(),
            Self::Heap(a) => a.as_mut_ptr().cast(),
        }
    }
}

// Source: src/core/SkRasterPipeline.h:172-180
// Ownership adaptation: native inheritance becomes an embedded pipeline and
// inline arena. Pin fixes both the arena's embedded backing and the pipeline's
// arena pointer before either can be used. Native derived members are destroyed
// before the base: the builtin arena drops its allocated objects and then its
// inline storage before the pipeline's base-equivalent fields are destroyed.
// Pipeline destruction only drops its memory-context metadata Vec, and does not
// dereference the expired arena/stage/context pointers.
// No stage/context owner is cloned and no second allocation arena is introduced.
#[allow(non_camel_case_types)]
pub struct SkRasterPipeline_<const BYTES: usize> {
    fBuiltinAlloc: crate::src::core::SkArenaAlloc::SkSTArenaAlloc<BYTES>,
    pipeline: Option<SkRasterPipeline>,
}
impl<const BYTES: usize> SkRasterPipeline_<BYTES> {
    // Source: src/core/SkRasterPipeline.h:175-176
    // Heap pinning adapts native stationary local-object construction. The
    // embedded arena keeps its native BYTES first-heap-allocation default.
    pub fn new() -> core::pin::Pin<Box<Self>> {
        let mut owner = Box::pin(Self {
            fBuiltinAlloc: crate::src::core::SkArenaAlloc::SkSTArenaAlloc::new(BYTES),
            pipeline: None,
        });
        unsafe {
            // Neither field is moved after obtaining its final address. No
            // reference into the arena survives the synchronous constructor.
            let this = owner.as_mut().get_unchecked_mut();
            let alloc = core::pin::Pin::new_unchecked(&mut this.fBuiltinAlloc)
                .pinned_arena()
                .get_unchecked_mut() as *mut SkArenaAlloc;
            this.pipeline = Some(SkRasterPipeline::new(alloc));
        }
        owner
    }
    /// Borrows the native base subobject without transferring its arena lease.
    pub fn as_pipeline(&self) -> &SkRasterPipeline {
        self.pipeline
            .as_ref()
            .expect("pipeline initialized before publication")
    }
    /// Safety: do not move, replace, swap out or retain this base subobject
    /// beyond its owner. Every deferred context/execution lease required by
    /// SkRasterPipeline still applies; the embedded arena must not be reset or
    /// dropped while any execution or extended pipeline retains its allocations.
    pub unsafe fn as_pipeline_mut(self: core::pin::Pin<&mut Self>) -> &mut SkRasterPipeline {
        self.get_unchecked_mut()
            .pipeline
            .as_mut()
            .expect("pipeline initialized before publication")
    }
}

// Source: src/core/SkRasterPipeline.cpp:236-244
// Native #op string literals retain their static NUL terminator for %s.
impl SkRasterPipeline {
    fn GetOpNameC(op: Op) -> &'static core::ffi::CStr {
        match op {
            Op::move_src_dst => c"move_src_dst",
            Op::move_dst_src => c"move_dst_src",
            Op::swap_src_dst => c"swap_src_dst",
            Op::clamp_01 => c"clamp_01",
            Op::clamp_a_01 => c"clamp_a_01",
            Op::clamp_gamut => c"clamp_gamut",
            Op::premul => c"premul",
            Op::premul_dst => c"premul_dst",
            Op::force_opaque => c"force_opaque",
            Op::force_opaque_dst => c"force_opaque_dst",
            Op::set_rgb => c"set_rgb",
            Op::swap_rb => c"swap_rb",
            Op::swap_rb_dst => c"swap_rb_dst",
            Op::black_color => c"black_color",
            Op::white_color => c"white_color",
            Op::uniform_color => c"uniform_color",
            Op::uniform_color_dst => c"uniform_color_dst",
            Op::seed_shader => c"seed_shader",
            Op::load_a8 => c"load_a8",
            Op::load_a8_dst => c"load_a8_dst",
            Op::store_a8 => c"store_a8",
            Op::gather_a8 => c"gather_a8",
            Op::load_565 => c"load_565",
            Op::load_565_dst => c"load_565_dst",
            Op::store_565 => c"store_565",
            Op::gather_565 => c"gather_565",
            Op::load_4444 => c"load_4444",
            Op::load_4444_dst => c"load_4444_dst",
            Op::store_4444 => c"store_4444",
            Op::gather_4444 => c"gather_4444",
            Op::load_8888 => c"load_8888",
            Op::load_8888_dst => c"load_8888_dst",
            Op::store_8888 => c"store_8888",
            Op::gather_8888 => c"gather_8888",
            Op::load_rg88 => c"load_rg88",
            Op::load_rg88_dst => c"load_rg88_dst",
            Op::store_rg88 => c"store_rg88",
            Op::gather_rg88 => c"gather_rg88",
            Op::store_r8 => c"store_r8",
            Op::alpha_to_gray => c"alpha_to_gray",
            Op::alpha_to_gray_dst => c"alpha_to_gray_dst",
            Op::alpha_to_red => c"alpha_to_red",
            Op::alpha_to_red_dst => c"alpha_to_red_dst",
            Op::bt709_luminance_or_luma_to_alpha => c"bt709_luminance_or_luma_to_alpha",
            Op::bt709_luminance_or_luma_to_rgb => c"bt709_luminance_or_luma_to_rgb",
            Op::bilerp_clamp_8888 => c"bilerp_clamp_8888",
            Op::load_src => c"load_src",
            Op::store_src => c"store_src",
            Op::store_src_a => c"store_src_a",
            Op::load_dst => c"load_dst",
            Op::store_dst => c"store_dst",
            Op::scale_u8 => c"scale_u8",
            Op::scale_565 => c"scale_565",
            Op::scale_1_float => c"scale_1_float",
            Op::scale_native => c"scale_native",
            Op::lerp_u8 => c"lerp_u8",
            Op::lerp_565 => c"lerp_565",
            Op::lerp_1_float => c"lerp_1_float",
            Op::lerp_native => c"lerp_native",
            Op::dstatop => c"dstatop",
            Op::dstin => c"dstin",
            Op::dstout => c"dstout",
            Op::dstover => c"dstover",
            Op::srcatop => c"srcatop",
            Op::srcin => c"srcin",
            Op::srcout => c"srcout",
            Op::srcover => c"srcover",
            Op::clear => c"clear",
            Op::modulate => c"modulate",
            Op::multiply => c"multiply",
            Op::plus_ => c"plus_",
            Op::screen => c"screen",
            Op::xor_ => c"xor_",
            Op::darken => c"darken",
            Op::difference => c"difference",
            Op::exclusion => c"exclusion",
            Op::hardlight => c"hardlight",
            Op::lighten => c"lighten",
            Op::overlay => c"overlay",
            Op::srcover_rgba_8888 => c"srcover_rgba_8888",
            Op::matrix_translate => c"matrix_translate",
            Op::matrix_scale_translate => c"matrix_scale_translate",
            Op::matrix_2x3 => c"matrix_2x3",
            Op::matrix_perspective => c"matrix_perspective",
            Op::decal_x => c"decal_x",
            Op::decal_y => c"decal_y",
            Op::decal_x_and_y => c"decal_x_and_y",
            Op::check_decal_mask => c"check_decal_mask",
            Op::clamp_x_1 => c"clamp_x_1",
            Op::mirror_x_1 => c"mirror_x_1",
            Op::repeat_x_1 => c"repeat_x_1",
            Op::clamp_x_and_y => c"clamp_x_and_y",
            Op::evenly_spaced_gradient => c"evenly_spaced_gradient",
            Op::gradient => c"gradient",
            Op::evenly_spaced_2_stop_gradient => c"evenly_spaced_2_stop_gradient",
            Op::xy_to_unit_angle => c"xy_to_unit_angle",
            Op::xy_to_radius => c"xy_to_radius",
            Op::emboss => c"emboss",
            Op::swizzle => c"swizzle",
            Op::debug_x => c"debug_x",
            Op::debug_y => c"debug_y",
            Op::debug_r => c"debug_r",
            Op::debug_g => c"debug_g",
            Op::debug_b => c"debug_b",
            Op::debug_a => c"debug_a",
            Op::debug_r_255 => c"debug_r_255",
            Op::debug_g_255 => c"debug_g_255",
            Op::debug_b_255 => c"debug_b_255",
            Op::debug_a_255 => c"debug_a_255",
            Op::init_lane_masks => c"init_lane_masks",
            Op::store_device_xy01 => c"store_device_xy01",
            Op::exchange_src => c"exchange_src",
            Op::load_condition_mask => c"load_condition_mask",
            Op::store_condition_mask => c"store_condition_mask",
            Op::merge_condition_mask => c"merge_condition_mask",
            Op::merge_inv_condition_mask => c"merge_inv_condition_mask",
            Op::load_loop_mask => c"load_loop_mask",
            Op::store_loop_mask => c"store_loop_mask",
            Op::mask_off_loop_mask => c"mask_off_loop_mask",
            Op::reenable_loop_mask => c"reenable_loop_mask",
            Op::merge_loop_mask => c"merge_loop_mask",
            Op::case_op => c"case_op",
            Op::continue_op => c"continue_op",
            Op::load_return_mask => c"load_return_mask",
            Op::store_return_mask => c"store_return_mask",
            Op::mask_off_return_mask => c"mask_off_return_mask",
            Op::branch_if_all_lanes_active => c"branch_if_all_lanes_active",
            Op::branch_if_any_lanes_active => c"branch_if_any_lanes_active",
            Op::branch_if_no_lanes_active => c"branch_if_no_lanes_active",
            Op::branch_if_no_active_lanes_eq => c"branch_if_no_active_lanes_eq",
            Op::jump => c"jump",
            Op::bitwise_and_imm_4_ints => c"bitwise_and_imm_4_ints",
            Op::bitwise_and_imm_3_ints => c"bitwise_and_imm_3_ints",
            Op::bitwise_and_imm_2_ints => c"bitwise_and_imm_2_ints",
            Op::bitwise_and_imm_int => c"bitwise_and_imm_int",
            Op::bitwise_and_n_ints => c"bitwise_and_n_ints",
            Op::bitwise_and_int => c"bitwise_and_int",
            Op::bitwise_and_2_ints => c"bitwise_and_2_ints",
            Op::bitwise_and_3_ints => c"bitwise_and_3_ints",
            Op::bitwise_and_4_ints => c"bitwise_and_4_ints",
            Op::bitwise_or_n_ints => c"bitwise_or_n_ints",
            Op::bitwise_or_int => c"bitwise_or_int",
            Op::bitwise_or_2_ints => c"bitwise_or_2_ints",
            Op::bitwise_or_3_ints => c"bitwise_or_3_ints",
            Op::bitwise_or_4_ints => c"bitwise_or_4_ints",
            Op::bitwise_xor_imm_int => c"bitwise_xor_imm_int",
            Op::bitwise_xor_n_ints => c"bitwise_xor_n_ints",
            Op::bitwise_xor_int => c"bitwise_xor_int",
            Op::bitwise_xor_2_ints => c"bitwise_xor_2_ints",
            Op::bitwise_xor_3_ints => c"bitwise_xor_3_ints",
            Op::bitwise_xor_4_ints => c"bitwise_xor_4_ints",
            Op::cast_to_float_from_int => c"cast_to_float_from_int",
            Op::cast_to_float_from_2_ints => c"cast_to_float_from_2_ints",
            Op::cast_to_float_from_3_ints => c"cast_to_float_from_3_ints",
            Op::cast_to_float_from_4_ints => c"cast_to_float_from_4_ints",
            Op::cast_to_float_from_uint => c"cast_to_float_from_uint",
            Op::cast_to_float_from_2_uints => c"cast_to_float_from_2_uints",
            Op::cast_to_float_from_3_uints => c"cast_to_float_from_3_uints",
            Op::cast_to_float_from_4_uints => c"cast_to_float_from_4_uints",
            Op::cast_to_int_from_float => c"cast_to_int_from_float",
            Op::cast_to_int_from_2_floats => c"cast_to_int_from_2_floats",
            Op::cast_to_int_from_3_floats => c"cast_to_int_from_3_floats",
            Op::cast_to_int_from_4_floats => c"cast_to_int_from_4_floats",
            Op::cast_to_uint_from_float => c"cast_to_uint_from_float",
            Op::cast_to_uint_from_2_floats => c"cast_to_uint_from_2_floats",
            Op::cast_to_uint_from_3_floats => c"cast_to_uint_from_3_floats",
            Op::cast_to_uint_from_4_floats => c"cast_to_uint_from_4_floats",
            Op::abs_int => c"abs_int",
            Op::abs_2_ints => c"abs_2_ints",
            Op::abs_3_ints => c"abs_3_ints",
            Op::abs_4_ints => c"abs_4_ints",
            Op::floor_float => c"floor_float",
            Op::floor_2_floats => c"floor_2_floats",
            Op::floor_3_floats => c"floor_3_floats",
            Op::floor_4_floats => c"floor_4_floats",
            Op::ceil_float => c"ceil_float",
            Op::ceil_2_floats => c"ceil_2_floats",
            Op::ceil_3_floats => c"ceil_3_floats",
            Op::ceil_4_floats => c"ceil_4_floats",
            Op::invsqrt_float => c"invsqrt_float",
            Op::invsqrt_2_floats => c"invsqrt_2_floats",
            Op::invsqrt_3_floats => c"invsqrt_3_floats",
            Op::invsqrt_4_floats => c"invsqrt_4_floats",
            Op::inverse_mat2 => c"inverse_mat2",
            Op::inverse_mat3 => c"inverse_mat3",
            Op::inverse_mat4 => c"inverse_mat4",
            Op::sin_float => c"sin_float",
            Op::cos_float => c"cos_float",
            Op::tan_float => c"tan_float",
            Op::asin_float => c"asin_float",
            Op::acos_float => c"acos_float",
            Op::atan_float => c"atan_float",
            Op::atan2_n_floats => c"atan2_n_floats",
            Op::sqrt_float => c"sqrt_float",
            Op::pow_n_floats => c"pow_n_floats",
            Op::exp_float => c"exp_float",
            Op::exp2_float => c"exp2_float",
            Op::log_float => c"log_float",
            Op::log2_float => c"log2_float",
            Op::refract_4_floats => c"refract_4_floats",
            Op::copy_uniform => c"copy_uniform",
            Op::copy_2_uniforms => c"copy_2_uniforms",
            Op::copy_3_uniforms => c"copy_3_uniforms",
            Op::copy_4_uniforms => c"copy_4_uniforms",
            Op::copy_constant => c"copy_constant",
            Op::splat_2_constants => c"splat_2_constants",
            Op::splat_3_constants => c"splat_3_constants",
            Op::splat_4_constants => c"splat_4_constants",
            Op::copy_slot_masked => c"copy_slot_masked",
            Op::copy_2_slots_masked => c"copy_2_slots_masked",
            Op::copy_3_slots_masked => c"copy_3_slots_masked",
            Op::copy_4_slots_masked => c"copy_4_slots_masked",
            Op::copy_from_indirect_unmasked => c"copy_from_indirect_unmasked",
            Op::copy_from_indirect_uniform_unmasked => c"copy_from_indirect_uniform_unmasked",
            Op::copy_to_indirect_masked => c"copy_to_indirect_masked",
            Op::swizzle_copy_to_indirect_masked => c"swizzle_copy_to_indirect_masked",
            Op::copy_slot_unmasked => c"copy_slot_unmasked",
            Op::copy_2_slots_unmasked => c"copy_2_slots_unmasked",
            Op::copy_3_slots_unmasked => c"copy_3_slots_unmasked",
            Op::copy_4_slots_unmasked => c"copy_4_slots_unmasked",
            Op::copy_immutable_unmasked => c"copy_immutable_unmasked",
            Op::copy_2_immutables_unmasked => c"copy_2_immutables_unmasked",
            Op::copy_3_immutables_unmasked => c"copy_3_immutables_unmasked",
            Op::copy_4_immutables_unmasked => c"copy_4_immutables_unmasked",
            Op::swizzle_copy_slot_masked => c"swizzle_copy_slot_masked",
            Op::swizzle_copy_2_slots_masked => c"swizzle_copy_2_slots_masked",
            Op::swizzle_copy_3_slots_masked => c"swizzle_copy_3_slots_masked",
            Op::swizzle_copy_4_slots_masked => c"swizzle_copy_4_slots_masked",
            Op::swizzle_1 => c"swizzle_1",
            Op::swizzle_2 => c"swizzle_2",
            Op::swizzle_3 => c"swizzle_3",
            Op::swizzle_4 => c"swizzle_4",
            Op::shuffle => c"shuffle",
            Op::matrix_multiply_2 => c"matrix_multiply_2",
            Op::matrix_multiply_3 => c"matrix_multiply_3",
            Op::matrix_multiply_4 => c"matrix_multiply_4",
            Op::smoothstep_n_floats => c"smoothstep_n_floats",
            Op::dot_2_floats => c"dot_2_floats",
            Op::dot_3_floats => c"dot_3_floats",
            Op::dot_4_floats => c"dot_4_floats",
            Op::add_imm_float => c"add_imm_float",
            Op::add_n_floats => c"add_n_floats",
            Op::add_float => c"add_float",
            Op::add_2_floats => c"add_2_floats",
            Op::add_3_floats => c"add_3_floats",
            Op::add_4_floats => c"add_4_floats",
            Op::add_imm_int => c"add_imm_int",
            Op::add_n_ints => c"add_n_ints",
            Op::add_int => c"add_int",
            Op::add_2_ints => c"add_2_ints",
            Op::add_3_ints => c"add_3_ints",
            Op::add_4_ints => c"add_4_ints",
            Op::sub_n_floats => c"sub_n_floats",
            Op::sub_float => c"sub_float",
            Op::sub_2_floats => c"sub_2_floats",
            Op::sub_3_floats => c"sub_3_floats",
            Op::sub_4_floats => c"sub_4_floats",
            Op::sub_n_ints => c"sub_n_ints",
            Op::sub_int => c"sub_int",
            Op::sub_2_ints => c"sub_2_ints",
            Op::sub_3_ints => c"sub_3_ints",
            Op::sub_4_ints => c"sub_4_ints",
            Op::mul_imm_float => c"mul_imm_float",
            Op::mul_n_floats => c"mul_n_floats",
            Op::mul_float => c"mul_float",
            Op::mul_2_floats => c"mul_2_floats",
            Op::mul_3_floats => c"mul_3_floats",
            Op::mul_4_floats => c"mul_4_floats",
            Op::mul_imm_int => c"mul_imm_int",
            Op::mul_n_ints => c"mul_n_ints",
            Op::mul_int => c"mul_int",
            Op::mul_2_ints => c"mul_2_ints",
            Op::mul_3_ints => c"mul_3_ints",
            Op::mul_4_ints => c"mul_4_ints",
            Op::div_n_floats => c"div_n_floats",
            Op::div_float => c"div_float",
            Op::div_2_floats => c"div_2_floats",
            Op::div_3_floats => c"div_3_floats",
            Op::div_4_floats => c"div_4_floats",
            Op::div_n_ints => c"div_n_ints",
            Op::div_int => c"div_int",
            Op::div_2_ints => c"div_2_ints",
            Op::div_3_ints => c"div_3_ints",
            Op::div_4_ints => c"div_4_ints",
            Op::div_n_uints => c"div_n_uints",
            Op::div_uint => c"div_uint",
            Op::div_2_uints => c"div_2_uints",
            Op::div_3_uints => c"div_3_uints",
            Op::div_4_uints => c"div_4_uints",
            Op::max_imm_float => c"max_imm_float",
            Op::max_n_floats => c"max_n_floats",
            Op::max_float => c"max_float",
            Op::max_2_floats => c"max_2_floats",
            Op::max_3_floats => c"max_3_floats",
            Op::max_4_floats => c"max_4_floats",
            Op::max_n_ints => c"max_n_ints",
            Op::max_int => c"max_int",
            Op::max_2_ints => c"max_2_ints",
            Op::max_3_ints => c"max_3_ints",
            Op::max_4_ints => c"max_4_ints",
            Op::max_n_uints => c"max_n_uints",
            Op::max_uint => c"max_uint",
            Op::max_2_uints => c"max_2_uints",
            Op::max_3_uints => c"max_3_uints",
            Op::max_4_uints => c"max_4_uints",
            Op::min_imm_float => c"min_imm_float",
            Op::min_n_floats => c"min_n_floats",
            Op::min_float => c"min_float",
            Op::min_2_floats => c"min_2_floats",
            Op::min_3_floats => c"min_3_floats",
            Op::min_4_floats => c"min_4_floats",
            Op::min_n_ints => c"min_n_ints",
            Op::min_int => c"min_int",
            Op::min_2_ints => c"min_2_ints",
            Op::min_3_ints => c"min_3_ints",
            Op::min_4_ints => c"min_4_ints",
            Op::min_n_uints => c"min_n_uints",
            Op::min_uint => c"min_uint",
            Op::min_2_uints => c"min_2_uints",
            Op::min_3_uints => c"min_3_uints",
            Op::min_4_uints => c"min_4_uints",
            Op::mod_n_floats => c"mod_n_floats",
            Op::mod_float => c"mod_float",
            Op::mod_2_floats => c"mod_2_floats",
            Op::mod_3_floats => c"mod_3_floats",
            Op::mod_4_floats => c"mod_4_floats",
            Op::mix_n_floats => c"mix_n_floats",
            Op::mix_float => c"mix_float",
            Op::mix_2_floats => c"mix_2_floats",
            Op::mix_3_floats => c"mix_3_floats",
            Op::mix_4_floats => c"mix_4_floats",
            Op::mix_n_ints => c"mix_n_ints",
            Op::mix_int => c"mix_int",
            Op::mix_2_ints => c"mix_2_ints",
            Op::mix_3_ints => c"mix_3_ints",
            Op::mix_4_ints => c"mix_4_ints",
            Op::cmplt_imm_float => c"cmplt_imm_float",
            Op::cmplt_n_floats => c"cmplt_n_floats",
            Op::cmplt_float => c"cmplt_float",
            Op::cmplt_2_floats => c"cmplt_2_floats",
            Op::cmplt_3_floats => c"cmplt_3_floats",
            Op::cmplt_4_floats => c"cmplt_4_floats",
            Op::cmplt_imm_int => c"cmplt_imm_int",
            Op::cmplt_n_ints => c"cmplt_n_ints",
            Op::cmplt_int => c"cmplt_int",
            Op::cmplt_2_ints => c"cmplt_2_ints",
            Op::cmplt_3_ints => c"cmplt_3_ints",
            Op::cmplt_4_ints => c"cmplt_4_ints",
            Op::cmplt_imm_uint => c"cmplt_imm_uint",
            Op::cmplt_n_uints => c"cmplt_n_uints",
            Op::cmplt_uint => c"cmplt_uint",
            Op::cmplt_2_uints => c"cmplt_2_uints",
            Op::cmplt_3_uints => c"cmplt_3_uints",
            Op::cmplt_4_uints => c"cmplt_4_uints",
            Op::cmple_imm_float => c"cmple_imm_float",
            Op::cmple_n_floats => c"cmple_n_floats",
            Op::cmple_float => c"cmple_float",
            Op::cmple_2_floats => c"cmple_2_floats",
            Op::cmple_3_floats => c"cmple_3_floats",
            Op::cmple_4_floats => c"cmple_4_floats",
            Op::cmple_imm_int => c"cmple_imm_int",
            Op::cmple_n_ints => c"cmple_n_ints",
            Op::cmple_int => c"cmple_int",
            Op::cmple_2_ints => c"cmple_2_ints",
            Op::cmple_3_ints => c"cmple_3_ints",
            Op::cmple_4_ints => c"cmple_4_ints",
            Op::cmple_imm_uint => c"cmple_imm_uint",
            Op::cmple_n_uints => c"cmple_n_uints",
            Op::cmple_uint => c"cmple_uint",
            Op::cmple_2_uints => c"cmple_2_uints",
            Op::cmple_3_uints => c"cmple_3_uints",
            Op::cmple_4_uints => c"cmple_4_uints",
            Op::cmpeq_imm_float => c"cmpeq_imm_float",
            Op::cmpeq_n_floats => c"cmpeq_n_floats",
            Op::cmpeq_float => c"cmpeq_float",
            Op::cmpeq_2_floats => c"cmpeq_2_floats",
            Op::cmpeq_3_floats => c"cmpeq_3_floats",
            Op::cmpeq_4_floats => c"cmpeq_4_floats",
            Op::cmpeq_imm_int => c"cmpeq_imm_int",
            Op::cmpeq_n_ints => c"cmpeq_n_ints",
            Op::cmpeq_int => c"cmpeq_int",
            Op::cmpeq_2_ints => c"cmpeq_2_ints",
            Op::cmpeq_3_ints => c"cmpeq_3_ints",
            Op::cmpeq_4_ints => c"cmpeq_4_ints",
            Op::cmpne_imm_float => c"cmpne_imm_float",
            Op::cmpne_n_floats => c"cmpne_n_floats",
            Op::cmpne_float => c"cmpne_float",
            Op::cmpne_2_floats => c"cmpne_2_floats",
            Op::cmpne_3_floats => c"cmpne_3_floats",
            Op::cmpne_4_floats => c"cmpne_4_floats",
            Op::cmpne_imm_int => c"cmpne_imm_int",
            Op::cmpne_n_ints => c"cmpne_n_ints",
            Op::cmpne_int => c"cmpne_int",
            Op::cmpne_2_ints => c"cmpne_2_ints",
            Op::cmpne_3_ints => c"cmpne_3_ints",
            Op::cmpne_4_ints => c"cmpne_4_ints",
            Op::trace_line => c"trace_line",
            Op::trace_var => c"trace_var",
            Op::trace_enter => c"trace_enter",
            Op::trace_exit => c"trace_exit",
            Op::trace_scope => c"trace_scope",
            Op::callback => c"callback",
            Op::stack_checkpoint => c"stack_checkpoint",
            Op::stack_rewind => c"stack_rewind",
            Op::unbounded_set_rgb => c"unbounded_set_rgb",
            Op::unbounded_uniform_color => c"unbounded_uniform_color",
            Op::unpremul => c"unpremul",
            Op::unpremul_polar => c"unpremul_polar",
            Op::dither => c"dither",
            Op::load_16161616 => c"load_16161616",
            Op::load_16161616_dst => c"load_16161616_dst",
            Op::store_16161616 => c"store_16161616",
            Op::gather_16161616 => c"gather_16161616",
            Op::load_a16 => c"load_a16",
            Op::load_a16_dst => c"load_a16_dst",
            Op::store_a16 => c"store_a16",
            Op::gather_a16 => c"gather_a16",
            Op::load_r16 => c"load_r16",
            Op::load_r16_dst => c"load_r16_dst",
            Op::store_r16 => c"store_r16",
            Op::gather_r16 => c"gather_r16",
            Op::load_rg1616 => c"load_rg1616",
            Op::load_rg1616_dst => c"load_rg1616_dst",
            Op::store_rg1616 => c"store_rg1616",
            Op::gather_rg1616 => c"gather_rg1616",
            Op::load_f16 => c"load_f16",
            Op::load_f16_dst => c"load_f16_dst",
            Op::store_f16 => c"store_f16",
            Op::gather_f16 => c"gather_f16",
            Op::load_af16 => c"load_af16",
            Op::load_af16_dst => c"load_af16_dst",
            Op::store_af16 => c"store_af16",
            Op::gather_af16 => c"gather_af16",
            Op::load_rf16 => c"load_rf16",
            Op::load_rf16_dst => c"load_rf16_dst",
            Op::store_rf16 => c"store_rf16",
            Op::gather_rf16 => c"gather_rf16",
            Op::load_rgf16 => c"load_rgf16",
            Op::load_rgf16_dst => c"load_rgf16_dst",
            Op::store_rgf16 => c"store_rgf16",
            Op::gather_rgf16 => c"gather_rgf16",
            Op::load_f32 => c"load_f32",
            Op::load_f32_dst => c"load_f32_dst",
            Op::store_f32 => c"store_f32",
            Op::gather_f32 => c"gather_f32",
            Op::load_1010102 => c"load_1010102",
            Op::load_1010102_dst => c"load_1010102_dst",
            Op::store_1010102 => c"store_1010102",
            Op::gather_1010102 => c"gather_1010102",
            Op::load_1010102_xr => c"load_1010102_xr",
            Op::load_1010102_xr_dst => c"load_1010102_xr_dst",
            Op::store_1010102_xr => c"store_1010102_xr",
            Op::gather_1010102_xr => c"gather_1010102_xr",
            Op::load_10x6 => c"load_10x6",
            Op::load_10x6_dst => c"load_10x6_dst",
            Op::store_10x6 => c"store_10x6",
            Op::gather_10x6 => c"gather_10x6",
            Op::gather_10101010_xr => c"gather_10101010_xr",
            Op::load_10101010_xr => c"load_10101010_xr",
            Op::load_10101010_xr_dst => c"load_10101010_xr_dst",
            Op::store_10101010_xr => c"store_10101010_xr",
            Op::store_src_rg => c"store_src_rg",
            Op::load_src_rg => c"load_src_rg",
            Op::byte_tables => c"byte_tables",
            Op::colorburn => c"colorburn",
            Op::colordodge => c"colordodge",
            Op::softlight => c"softlight",
            Op::hue => c"hue",
            Op::saturation => c"saturation",
            Op::color => c"color",
            Op::luminosity => c"luminosity",
            Op::matrix_3x3 => c"matrix_3x3",
            Op::matrix_3x4 => c"matrix_3x4",
            Op::matrix_4x5 => c"matrix_4x5",
            Op::matrix_4x3 => c"matrix_4x3",
            Op::parametric => c"parametric",
            Op::gamma_ => c"gamma_",
            Op::PQish => c"PQish",
            Op::HLGish => c"HLGish",
            Op::HLGinvish => c"HLGinvish",
            Op::ootf => c"ootf",
            Op::rgb_to_hsl => c"rgb_to_hsl",
            Op::hsl_to_rgb => c"hsl_to_rgb",
            Op::css_lab_to_xyz => c"css_lab_to_xyz",
            Op::css_oklab_to_linear_srgb => c"css_oklab_to_linear_srgb",
            Op::css_oklab_gamut_map_to_linear_srgb => c"css_oklab_gamut_map_to_linear_srgb",
            Op::css_hcl_to_lab => c"css_hcl_to_lab",
            Op::css_hsl_to_srgb => c"css_hsl_to_srgb",
            Op::css_hwb_to_srgb => c"css_hwb_to_srgb",
            Op::gauss_a_to_rgba => c"gauss_a_to_rgba",
            Op::mirror_x => c"mirror_x",
            Op::repeat_x => c"repeat_x",
            Op::mirror_y => c"mirror_y",
            Op::repeat_y => c"repeat_y",
            Op::negate_x => c"negate_x",
            Op::bilerp_clamp_8888_force_highp => c"bilerp_clamp_8888_force_highp",
            Op::bicubic_clamp_8888 => c"bicubic_clamp_8888",
            Op::bilinear_setup => c"bilinear_setup",
            Op::bilinear_nx => c"bilinear_nx",
            Op::bilinear_px => c"bilinear_px",
            Op::bilinear_ny => c"bilinear_ny",
            Op::bilinear_py => c"bilinear_py",
            Op::bicubic_setup => c"bicubic_setup",
            Op::bicubic_n3x => c"bicubic_n3x",
            Op::bicubic_n1x => c"bicubic_n1x",
            Op::bicubic_p1x => c"bicubic_p1x",
            Op::bicubic_p3x => c"bicubic_p3x",
            Op::bicubic_n3y => c"bicubic_n3y",
            Op::bicubic_n1y => c"bicubic_n1y",
            Op::bicubic_p1y => c"bicubic_p1y",
            Op::bicubic_p3y => c"bicubic_p3y",
            Op::accumulate => c"accumulate",
            Op::perlin_noise => c"perlin_noise",
            Op::mipmap_linear_init => c"mipmap_linear_init",
            Op::mipmap_linear_update => c"mipmap_linear_update",
            Op::mipmap_linear_finish => c"mipmap_linear_finish",
            Op::xy_to_2pt_conical_strip => c"xy_to_2pt_conical_strip",
            Op::xy_to_2pt_conical_focal_on_circle => c"xy_to_2pt_conical_focal_on_circle",
            Op::xy_to_2pt_conical_well_behaved => c"xy_to_2pt_conical_well_behaved",
            Op::xy_to_2pt_conical_smaller => c"xy_to_2pt_conical_smaller",
            Op::xy_to_2pt_conical_greater => c"xy_to_2pt_conical_greater",
            Op::alter_2pt_conical_compensate_focal => c"alter_2pt_conical_compensate_focal",
            Op::alter_2pt_conical_unswap => c"alter_2pt_conical_unswap",
            Op::mask_2pt_conical_nan => c"mask_2pt_conical_nan",
            Op::mask_2pt_conical_degenerates => c"mask_2pt_conical_degenerates",
            Op::apply_vector_mask => c"apply_vector_mask",
            Op::set_base_pointer => c"set_base_pointer",
        }
    }
}
