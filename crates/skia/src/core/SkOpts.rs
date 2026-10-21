// Copyright 2015 Google Inc. BSD-style license.
// Actual native function-address tables and their matching start/stride values.
// No operation enum stands in for a function pointer; no State/PC execution adapter.
use super::SkRasterPipeline::SkRasterPipelineStage;
use super::SkRasterPipelineOpContexts::MemoryCtxPatch;
use super::SkRasterPipelineOpList::{kNumRasterPipelineHighpOps, kNumRasterPipelineLowpOps};
use crate::src::opts::SkRasterPipeline_opts as scalar;
// Source: src/core/SkOpts.h:75-75
pub type StageFn = unsafe extern "C" fn();
// Source: src/core/SkOpts.h:79-86
pub type StartPipelineFn = unsafe fn(
    usize,
    usize,
    usize,
    usize,
    *mut SkRasterPipelineStage,
    &mut [MemoryCtxPatch],
    *mut u8,
);

// Source: src/core/SkOpts.cpp:25-43
// This retains the existing initializer argument name only as a binding snapshot.
// It contains actual function addresses; there is no kind/State/PC/runner wrapper.
#[derive(Clone, Copy)]
pub(crate) struct PipelineDispatch {
    ops: &'static [Option<StageFn>],
    terminal: Option<StageFn>,
    start: StartPipelineFn,
    stride: usize,
}
impl PipelineDispatch {
    fn table<const N: usize>(self) -> [Option<StageFn>; N] {
        core::array::from_fn(|i| self.ops[i])
    }
}
// Source: src/core/SkOpts.cpp:25-26
// Disabled lowp inherits the highp vector depth in the original lowp_N lookup.
pub static raster_pipeline_lowp_stride: std::sync::LazyLock<usize> =
    std::sync::LazyLock::new(|| {
        let (low, high) = active_dispatches();
        low.map_or(high.stride, |d| d.stride)
    });
pub static raster_pipeline_highp_stride: std::sync::LazyLock<usize> =
    std::sync::LazyLock::new(|| active_dispatches().1.stride);
// Source: src/core/SkOpts.cpp:28-35
pub static ops_highp: std::sync::LazyLock<[Option<StageFn>; kNumRasterPipelineHighpOps]> =
    std::sync::LazyLock::new(|| active_dispatches().1.table());
pub static just_return_highp: std::sync::LazyLock<StageFn> = std::sync::LazyLock::new(|| {
    // Every highp constructor below installs its actual nonnull just_return.
    unsafe { active_dispatches().1.terminal.unwrap_unchecked() }
});
pub static start_pipeline_highp: std::sync::LazyLock<StartPipelineFn> =
    std::sync::LazyLock::new(|| active_dispatches().1.start);
// Source: src/core/SkOpts.cpp:37-43
pub static ops_lowp: std::sync::LazyLock<[Option<StageFn>; kNumRasterPipelineLowpOps]> =
    std::sync::LazyLock::new(|| match active_dispatches().0 {
        Some(d) => d.table(),
        None => scalar::disabled_lowp::OPS,
    });
pub static just_return_lowp: std::sync::LazyLock<Option<StageFn>> =
    std::sync::LazyLock::new(|| match active_dispatches().0 {
        Some(d) => d.terminal,
        None => scalar::disabled_lowp::JUST_RETURN,
    });
pub static start_pipeline_lowp: std::sync::LazyLock<Option<StartPipelineFn>> =
    std::sync::LazyLock::new(|| {
        Some(match active_dispatches().0 {
            Some(d) => d.start,
            None => scalar::disabled_lowp::start_pipeline as StartPipelineFn,
        })
    });

// Rust namespace projection of the genuine default bindings/initializers in
// their original implementation files. Selected ARM64 has no runtime replacement
// in these units; this is not a second dispatch table or initialization backend.
pub use super::SkBitmapProcState_opts::{
    Init_BitmapProcState, S32_alpha_D32_filter_DX, S32_alpha_D32_filter_DXDY,
};
#[cfg(target_arch = "aarch64")]
pub use super::SkBlitMask_opts::Init_BlitMask;
pub use super::SkBlitRow_opts::blit_row_s32a_opaque;
pub use super::SkBlitRow_opts::{blit_row_color32, Init_BlitRow};
pub use super::SkMemset_opts::{
    memset16, memset32, memset64, rect_memset16, rect_memset32, rect_memset64, Init_Memset,
};
#[cfg(target_arch = "aarch64")]
pub use super::SkSwizzler_opts::Init_Swizzler;
// Preserve the previously exposed real non-ARM defaults; runtime replacement
// implementations remain pending and no ARM64 Init function is exported there.
#[cfg(not(target_arch = "aarch64"))]
pub const S32_alpha_D32_filter_DX: Option<crate::src::core::SkBitmapProcState::SampleProc32> = Some(
    crate::src::opts::SkBitmapProcState_opts::S32_alpha_D32_filter_DX
        as crate::src::core::SkBitmapProcState::SampleProc32,
);
#[cfg(not(target_arch = "aarch64"))]
pub use crate::src::opts::SkBitmapProcState_opts::S32_alpha_D32_filter_DXDY;

// Source: src/core/SkOpts.h:71-71
// Source: src/core/SkOpts.cpp:74-76
pub fn Init() {
    let _ = active_dispatches();
}

// Source: src/core/SkOpts.cpp:25-43
pub(crate) const fn scalar_dispatch() -> PipelineDispatch {
    use scalar::scalar_wide_abi as n;
    static OPS: [Option<StageFn>; kNumRasterPipelineHighpOps] =
        super::SkRasterPipelineOpList::native_highp_function_table!(n, StageFn);
    PipelineDispatch {
        ops: &OPS,
        terminal: Some(unsafe {
            core::mem::transmute::<n::StageFn, StageFn>(n::just_return as n::StageFn)
        }),
        start: n::start_pipeline,
        stride: 1,
    }
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
// Source: src/core/SkOpts.cpp:25-43
pub(crate) const fn neon_highp_dispatch() -> PipelineDispatch {
    use scalar::neon_highp_stages as n;
    static OPS: [Option<StageFn>; kNumRasterPipelineHighpOps] =
        super::SkRasterPipelineOpList::native_highp_function_table!(n, StageFn);
    PipelineDispatch {
        ops: &OPS,
        terminal: Some(unsafe {
            core::mem::transmute::<n::StageFn, StageFn>(n::just_return as n::StageFn)
        }),
        start: n::start_pipeline,
        stride: 4,
    }
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
// Source: src/core/SkOpts.cpp:25-43
pub(crate) const fn neon_lowp_dispatch() -> PipelineDispatch {
    #[cfg(any(
        SKRP_CPU_SCALAR,
        SK_ENABLE_OPTIMIZE_SIZE,
        SK_DISABLE_LOWP_RASTER_PIPELINE
    ))]
    {
        PipelineDispatch {
            ops: &scalar::disabled_lowp::OPS,
            terminal: scalar::disabled_lowp::JUST_RETURN,
            start: scalar::disabled_lowp::start_pipeline,
            stride: 4,
        }
    }
    #[cfg(not(any(
        SKRP_CPU_SCALAR,
        SK_ENABLE_OPTIMIZE_SIZE,
        SK_DISABLE_LOWP_RASTER_PIPELINE
    )))]
    {
        use scalar::neon_lowp_stages as n;
        static OPS: [Option<StageFn>; kNumRasterPipelineLowpOps] =
            super::SkRasterPipelineOpList::native_lowp_function_table!(n, StageFn);
        PipelineDispatch {
            ops: &OPS,
            terminal: Some(unsafe {
                core::mem::transmute::<n::StageFn, StageFn>(n::just_return as n::StageFn)
            }),
            start: n::start_pipeline,
            stride: 8,
        }
    }
}

#[cfg(any(
    target_arch = "x86_64",
    all(target_arch = "x86", target_feature = "sse2")
))]
// Source: src/core/SkOpts.cpp:25-43
pub(crate) const fn sse2_highp_dispatch() -> PipelineDispatch {
    use scalar::x86_sse_highp_stages as n;
    static OPS: [Option<StageFn>; kNumRasterPipelineHighpOps] =
        super::SkRasterPipelineOpList::native_highp_function_table!(n, StageFn);
    PipelineDispatch {
        ops: &OPS,
        terminal: Some(unsafe {
            core::mem::transmute::<n::StageFn, StageFn>(n::just_return as n::StageFn)
        }),
        start: n::start_pipeline,
        stride: 4,
    }
}

#[cfg(any(
    target_arch = "x86_64",
    all(target_arch = "x86", target_feature = "sse2")
))]
// Source: src/core/SkOpts.cpp:25-43
pub(crate) const fn x86_sse_lowp_stages_dispatch() -> PipelineDispatch {
    #[cfg(any(
        SKRP_CPU_SCALAR,
        SK_ENABLE_OPTIMIZE_SIZE,
        SK_DISABLE_LOWP_RASTER_PIPELINE
    ))]
    {
        PipelineDispatch {
            ops: &scalar::disabled_lowp::OPS,
            terminal: scalar::disabled_lowp::JUST_RETURN,
            start: scalar::disabled_lowp::start_pipeline,
            stride: 4,
        }
    }
    #[cfg(not(any(
        SKRP_CPU_SCALAR,
        SK_ENABLE_OPTIMIZE_SIZE,
        SK_DISABLE_LOWP_RASTER_PIPELINE
    )))]
    {
        use scalar::x86_sse_lowp_stages as n;
        static OPS: [Option<StageFn>; kNumRasterPipelineLowpOps] =
            super::SkRasterPipelineOpList::native_lowp_function_table!(n, StageFn);
        PipelineDispatch {
            ops: &OPS,
            terminal: Some(unsafe {
                core::mem::transmute::<n::StageFn, StageFn>(n::just_return as n::StageFn)
            }),
            start: n::start_pipeline,
            stride: 8,
        }
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
// Source: src/core/SkOpts.cpp:25-43
pub(crate) const fn x86_sse41_highp_stages_dispatch() -> PipelineDispatch {
    use scalar::x86_sse41_highp_stages as n;
    static OPS: [Option<StageFn>; kNumRasterPipelineHighpOps] =
        super::SkRasterPipelineOpList::native_highp_function_table!(n, StageFn);
    PipelineDispatch {
        ops: &OPS,
        terminal: Some(unsafe {
            core::mem::transmute::<n::StageFn, StageFn>(n::just_return as n::StageFn)
        }),
        start: n::start_pipeline,
        stride: 4,
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
// Source: src/core/SkOpts.cpp:25-43
pub(crate) const fn x86_sse41_lowp_stages_dispatch() -> PipelineDispatch {
    #[cfg(any(
        SKRP_CPU_SCALAR,
        SK_ENABLE_OPTIMIZE_SIZE,
        SK_DISABLE_LOWP_RASTER_PIPELINE
    ))]
    {
        PipelineDispatch {
            ops: &scalar::disabled_lowp::OPS,
            terminal: scalar::disabled_lowp::JUST_RETURN,
            start: scalar::disabled_lowp::start_pipeline,
            stride: 4,
        }
    }
    #[cfg(not(any(
        SKRP_CPU_SCALAR,
        SK_ENABLE_OPTIMIZE_SIZE,
        SK_DISABLE_LOWP_RASTER_PIPELINE
    )))]
    {
        use scalar::x86_sse41_lowp_stages as n;
        static OPS: [Option<StageFn>; kNumRasterPipelineLowpOps] =
            super::SkRasterPipelineOpList::native_lowp_function_table!(n, StageFn);
        PipelineDispatch {
            ops: &OPS,
            terminal: Some(unsafe {
                core::mem::transmute::<n::StageFn, StageFn>(n::just_return as n::StageFn)
            }),
            start: n::start_pipeline,
            stride: 8,
        }
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
// Source: src/core/SkOpts.cpp:25-43
pub(crate) const fn x86_avx_highp_stages_dispatch() -> PipelineDispatch {
    use scalar::x86_avx_highp_stages as n;
    static OPS: [Option<StageFn>; kNumRasterPipelineHighpOps] =
        super::SkRasterPipelineOpList::native_highp_function_table!(n, StageFn);
    PipelineDispatch {
        ops: &OPS,
        terminal: Some(unsafe {
            core::mem::transmute::<n::StageFn, StageFn>(n::just_return as n::StageFn)
        }),
        start: n::start_pipeline,
        stride: 8,
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
// Source: src/core/SkOpts.cpp:25-43
pub(crate) const fn x86_avx_lowp_stages_dispatch() -> PipelineDispatch {
    #[cfg(any(
        SKRP_CPU_SCALAR,
        SK_ENABLE_OPTIMIZE_SIZE,
        SK_DISABLE_LOWP_RASTER_PIPELINE
    ))]
    {
        PipelineDispatch {
            ops: &scalar::disabled_lowp::OPS,
            terminal: scalar::disabled_lowp::JUST_RETURN,
            start: scalar::disabled_lowp::start_pipeline,
            stride: 8,
        }
    }
    #[cfg(not(any(
        SKRP_CPU_SCALAR,
        SK_ENABLE_OPTIMIZE_SIZE,
        SK_DISABLE_LOWP_RASTER_PIPELINE
    )))]
    {
        use scalar::x86_avx_lowp_stages as n;
        static OPS: [Option<StageFn>; kNumRasterPipelineLowpOps] =
            super::SkRasterPipelineOpList::native_lowp_function_table!(n, StageFn);
        PipelineDispatch {
            ops: &OPS,
            terminal: Some(unsafe {
                core::mem::transmute::<n::StageFn, StageFn>(n::just_return as n::StageFn)
            }),
            start: n::start_pipeline,
            stride: 8,
        }
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
// Source: src/core/SkOpts.cpp:25-43
pub(crate) const fn avx2_highp_dispatch() -> PipelineDispatch {
    use scalar::x86_avx2_highp_stages as n;
    static OPS: [Option<StageFn>; kNumRasterPipelineHighpOps] =
        super::SkRasterPipelineOpList::native_highp_function_table!(n, StageFn);
    PipelineDispatch {
        ops: &OPS,
        terminal: Some(unsafe {
            core::mem::transmute::<n::StageFn, StageFn>(n::just_return as n::StageFn)
        }),
        start: n::start_pipeline,
        stride: 8,
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
// Source: src/core/SkOpts.cpp:25-43
pub(crate) const fn x86_avx2_lowp_stages_dispatch() -> PipelineDispatch {
    #[cfg(any(
        SKRP_CPU_SCALAR,
        SK_ENABLE_OPTIMIZE_SIZE,
        SK_DISABLE_LOWP_RASTER_PIPELINE
    ))]
    {
        PipelineDispatch {
            ops: &scalar::disabled_lowp::OPS,
            terminal: scalar::disabled_lowp::JUST_RETURN,
            start: scalar::disabled_lowp::start_pipeline,
            stride: 8,
        }
    }
    #[cfg(not(any(
        SKRP_CPU_SCALAR,
        SK_ENABLE_OPTIMIZE_SIZE,
        SK_DISABLE_LOWP_RASTER_PIPELINE
    )))]
    {
        use scalar::x86_avx2_lowp_stages as n;
        static OPS: [Option<StageFn>; kNumRasterPipelineLowpOps] =
            super::SkRasterPipelineOpList::native_lowp_function_table!(n, StageFn);
        PipelineDispatch {
            ops: &OPS,
            terminal: Some(unsafe {
                core::mem::transmute::<n::StageFn, StageFn>(n::just_return as n::StageFn)
            }),
            start: n::start_pipeline,
            stride: 16,
        }
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
// Source: src/core/SkOpts.cpp:25-43
pub(crate) const fn ml4_highp_dispatch() -> PipelineDispatch {
    use scalar::x86_ml4_highp_stages as n;
    static OPS: [Option<StageFn>; kNumRasterPipelineHighpOps] =
        super::SkRasterPipelineOpList::native_highp_function_table!(n, StageFn);
    PipelineDispatch {
        ops: &OPS,
        terminal: Some(unsafe {
            core::mem::transmute::<n::StageFn, StageFn>(n::just_return as n::StageFn)
        }),
        start: n::start_pipeline,
        stride: 16,
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
// Source: src/core/SkOpts.cpp:25-43
pub(crate) const fn x86_ml4_lowp_stages_dispatch() -> PipelineDispatch {
    #[cfg(any(
        SKRP_CPU_SCALAR,
        SK_ENABLE_OPTIMIZE_SIZE,
        SK_DISABLE_LOWP_RASTER_PIPELINE
    ))]
    {
        PipelineDispatch {
            ops: &scalar::disabled_lowp::OPS,
            terminal: scalar::disabled_lowp::JUST_RETURN,
            start: scalar::disabled_lowp::start_pipeline,
            stride: 16,
        }
    }
    #[cfg(not(any(
        SKRP_CPU_SCALAR,
        SK_ENABLE_OPTIMIZE_SIZE,
        SK_DISABLE_LOWP_RASTER_PIPELINE
    )))]
    {
        use scalar::x86_ml4_lowp_stages as n;
        static OPS: [Option<StageFn>; kNumRasterPipelineLowpOps] =
            super::SkRasterPipelineOpList::native_lowp_function_table!(n, StageFn);
        PipelineDispatch {
            ops: &OPS,
            terminal: Some(unsafe {
                core::mem::transmute::<n::StageFn, StageFn>(n::just_return as n::StageFn)
            }),
            start: n::start_pipeline,
            stride: 16,
        }
    }
}

#[cfg(target_arch = "loongarch64")]
// Source: src/core/SkOpts.cpp:25-43
pub(crate) const fn lsx_highp_dispatch() -> PipelineDispatch {
    use scalar::lsx_pipe_highp_stages as n;
    static OPS: [Option<StageFn>; kNumRasterPipelineHighpOps] =
        super::SkRasterPipelineOpList::native_highp_function_table!(n, StageFn);
    PipelineDispatch {
        ops: &OPS,
        terminal: Some(unsafe {
            core::mem::transmute::<n::StageFn, StageFn>(n::just_return as n::StageFn)
        }),
        start: n::start_pipeline,
        stride: 4,
    }
}

#[cfg(target_arch = "loongarch64")]
// Source: src/core/SkOpts.cpp:25-43
pub(crate) const fn lsx_lowp_dispatch() -> PipelineDispatch {
    #[cfg(any(
        SKRP_CPU_SCALAR,
        SK_ENABLE_OPTIMIZE_SIZE,
        SK_DISABLE_LOWP_RASTER_PIPELINE
    ))]
    {
        PipelineDispatch {
            ops: &scalar::disabled_lowp::OPS,
            terminal: scalar::disabled_lowp::JUST_RETURN,
            start: scalar::disabled_lowp::start_pipeline,
            stride: 4,
        }
    }
    #[cfg(not(any(
        SKRP_CPU_SCALAR,
        SK_ENABLE_OPTIMIZE_SIZE,
        SK_DISABLE_LOWP_RASTER_PIPELINE
    )))]
    {
        use scalar::lsx_pipe_lowp_stages as n;
        static OPS: [Option<StageFn>; kNumRasterPipelineLowpOps] =
            super::SkRasterPipelineOpList::native_lowp_function_table!(n, StageFn);
        PipelineDispatch {
            ops: &OPS,
            terminal: Some(unsafe {
                core::mem::transmute::<n::StageFn, StageFn>(n::just_return as n::StageFn)
            }),
            start: n::start_pipeline,
            stride: 8,
        }
    }
}

#[cfg(target_arch = "loongarch64")]
// Source: src/core/SkOpts.cpp:25-43
pub(crate) const fn lasx_highp_dispatch() -> PipelineDispatch {
    use scalar::lasx_pipe_highp_stages as n;
    static OPS: [Option<StageFn>; kNumRasterPipelineHighpOps] =
        super::SkRasterPipelineOpList::native_highp_function_table!(n, StageFn);
    PipelineDispatch {
        ops: &OPS,
        terminal: Some(unsafe {
            core::mem::transmute::<n::StageFn, StageFn>(n::just_return as n::StageFn)
        }),
        start: n::start_pipeline,
        stride: 8,
    }
}

#[cfg(target_arch = "loongarch64")]
// Source: src/core/SkOpts.cpp:25-43
pub(crate) const fn lasx_lowp_dispatch() -> PipelineDispatch {
    #[cfg(any(
        SKRP_CPU_SCALAR,
        SK_ENABLE_OPTIMIZE_SIZE,
        SK_DISABLE_LOWP_RASTER_PIPELINE
    ))]
    {
        PipelineDispatch {
            ops: &scalar::disabled_lowp::OPS,
            terminal: scalar::disabled_lowp::JUST_RETURN,
            start: scalar::disabled_lowp::start_pipeline,
            stride: 8,
        }
    }
    #[cfg(not(any(
        SKRP_CPU_SCALAR,
        SK_ENABLE_OPTIMIZE_SIZE,
        SK_DISABLE_LOWP_RASTER_PIPELINE
    )))]
    {
        use scalar::lasx_pipe_lowp_stages as n;
        static OPS: [Option<StageFn>; kNumRasterPipelineLowpOps] =
            super::SkRasterPipelineOpList::native_lowp_function_table!(n, StageFn);
        PipelineDispatch {
            ops: &OPS,
            terminal: Some(unsafe {
                core::mem::transmute::<n::StageFn, StageFn>(n::just_return as n::StageFn)
            }),
            start: n::start_pipeline,
            stride: 16,
        }
    }
}

// Source: src/core/SkOpts.cpp:20-43,51-76
// Native Init effects publish the same actual table/start/terminal/stride values.
pub(crate) fn active_dispatches() -> (Option<PipelineDispatch>, PipelineDispatch) {
    static SELECTED: std::sync::OnceLock<(Option<PipelineDispatch>, PipelineDispatch)> =
        std::sync::OnceLock::new();
    *SELECTED.get_or_init(|| {
        if cfg!(SKRP_CPU_SCALAR) {
            return (None, scalar_dispatch());
        }
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            let pair = native_x86_dispatches();
            return if scalar::disabled_lowp::COMPILE_DISABLED {
                (None, pair.1)
            } else {
                pair
            };
        }
        #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
        {
            let pair = native_neon_dispatches();
            return if scalar::disabled_lowp::COMPILE_DISABLED {
                (None, pair.1)
            } else {
                pair
            };
        }
        #[cfg(target_arch = "loongarch64")]
        {
            let pair = native_lasx_dispatches();
            return if scalar::disabled_lowp::COMPILE_DISABLED {
                (None, pair.1)
            } else {
                pair
            };
        }
        #[cfg(not(any(
            target_arch = "x86",
            target_arch = "x86_64",
            target_arch = "loongarch64",
            all(target_arch = "aarch64", target_feature = "neon")
        )))]
        {
            (None, scalar_dispatch())
        }
    })
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn native_x86_dispatches() -> (Option<PipelineDispatch>, PipelineDispatch) {
    use crate::src::core::SkCpu::{self, SkX64};
    // Source: src/opts/SkRasterPipeline_opts.h:88-95
    // Honor the native global compiler level before considering runtime upgrades.
    if cfg!(SKRP_CPU_SCALAR) {
        return (None, scalar_dispatch());
    }
    let mut selected = (None, scalar_dispatch());
    #[cfg(any(
        target_arch = "x86_64",
        all(target_arch = "x86", target_feature = "sse2")
    ))]
    {
        selected = (
            Some(unsafe { x86_sse_lowp_stages_dispatch() }),
            sse2_highp_dispatch(),
        );
    }
    if cfg!(all(target_feature = "sse4.1", target_feature = "ssse3")) {
        selected = (Some(unsafe { x86_sse41_lowp_stages_dispatch() }), unsafe {
            x86_sse41_highp_stages_dispatch()
        });
    }
    if cfg!(all(
        target_feature = "avx",
        target_feature = "sse4.1",
        target_feature = "ssse3"
    )) {
        selected = (Some(unsafe { x86_avx_lowp_stages_dispatch() }), unsafe {
            x86_avx_highp_stages_dispatch()
        });
    }
    let compiled_ml3 = cfg!(all(
        target_feature = "avx2",
        target_feature = "fma",
        target_feature = "f16c"
    ));
    let compiled_ml4 = compiled_ml3
        && cfg!(all(
            target_feature = "avx512f",
            target_feature = "avx512dq",
            target_feature = "avx512cd",
            target_feature = "avx512bw",
            target_feature = "avx512vl"
        ));
    if compiled_ml3 {
        selected = (Some(unsafe { x86_avx2_lowp_stages_dispatch() }), unsafe {
            avx2_highp_dispatch()
        });
    }
    if compiled_ml4 {
        selected = (Some(unsafe { x86_ml4_lowp_stages_dispatch() }), unsafe {
            ml4_highp_dispatch()
        });
    }
    // Source: src/core/SkOpts.cpp:52-69
    // Init_ml3/Init_ml4 effects are a paired descriptor replacement, not a
    // partially installed stage array. Preserve optimize-size and AVX512 flags.
    #[cfg(not(SK_ENABLE_OPTIMIZE_SIZE))]
    {
        SkCpu::CacheRuntimeFeatures();
        let ml3 = SkCpu::Supports(SkX64::ML3);
        if !compiled_ml3 && ml3 {
            unsafe {
                crate::src::opts::SkOpts_ml3::Init_ml3(&mut selected);
            }
        }
        if !compiled_ml4
            && cfg!(SK_ENABLE_AVX512_OPTS)
            && !cfg!(SK_DISABLE_AVX512_OPTS)
            && ml3
            && SkCpu::Supports(SkX64::ML4)
        {
            unsafe {
                crate::src::opts::SkOpts_ml4::Init_ml4(&mut selected);
            }
        }
    }
    selected
}

// Source: src/core/SkOpts.cpp:20-43,51-72
#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
fn native_neon_dispatches() -> (Option<PipelineDispatch>, PipelineDispatch) {
    (Some(neon_lowp_dispatch()), neon_highp_dispatch())
}
// Source: src/core/SkOpts.cpp:51-72
#[cfg(target_arch = "loongarch64")]
fn native_lasx_dispatches() -> (Option<PipelineDispatch>, PipelineDispatch) {
    let mut selected = if cfg!(target_feature = "lsx") {
        (Some(lsx_lowp_dispatch()), lsx_highp_dispatch())
    } else {
        (None, scalar_dispatch())
    };
    if cfg!(target_feature = "lasx") {
        selected = (Some(lasx_lowp_dispatch()), lasx_highp_dispatch());
    }
    #[cfg(not(SK_ENABLE_OPTIMIZE_SIZE))]
    if !cfg!(target_feature = "lasx") {
        super::SkCpu::CacheRuntimeFeatures();
        if super::SkCpu::Supports(super::SkCpu::SkLoongArch::ASX) {
            unsafe {
                crate::src::opts::SkOpts_lasx::Init_lasx(&mut selected);
            }
        }
    }
    selected
}
