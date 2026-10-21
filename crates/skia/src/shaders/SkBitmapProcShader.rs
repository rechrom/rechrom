// RasterError interface/propagation adaptation only; UNVERIFIED; no added native-line credit.
// Copyright 2006 The Android Open Source Project. BSD-style license.
use crate::src::core::SkBitmapProcState::SkBitmapProcState;
use crate::src::shaders::SkShaderBase::{
    kOpaqueAlpha_Flag, Context, ContextRec, ContextState, SkShaderBase,
};

// Source: src/shaders/SkBitmapProcShader.cpp:24-28
// Source: src/shaders/SkBitmapProcShader.cpp:30-31
impl BitmapProcShaderContext {
    unsafe fn new(
        shader: &dyn SkShaderBase,
        rec: &ContextRec<'_>,
        state: *mut SkBitmapProcState<'_>,
    ) -> Self {
        let base = ContextState::new(shader, rec);
        let mut flags = 0;
        if unsafe { (&*state).pixmapIsOpaque() } && base.getPaintAlpha() == 255 {
            flags |= kOpaqueAlpha_Flag;
        }
        Self {
            base,
            fState: core::ptr::NonNull::new_unchecked(state.cast()),
            fFlags: flags,
        }
    }
}
impl Context for BitmapProcShaderContext {
    fn state(&self) -> &ContextState {
        &self.base
    }
    // Source: src/shaders/SkBitmapProcShader.cpp:35-35
    fn getFlags(&self) -> u32 {
        self.fFlags
    }
    // Source: src/shaders/SkBitmapProcShader.cpp:37-41
    // Source: src/shaders/SkBitmapProcShader.cpp:44-48
    // Source: src/shaders/SkBitmapProcShader.cpp:50-50
    // Source: src/shaders/SkBitmapProcShader.cpp:52-56
    // Source: src/shaders/SkBitmapProcShader.cpp:58-59
    // Source: src/shaders/SkBitmapProcShader.cpp:61-63
    /// Safety: uphold Context::shadeSpan, and retain this context's successfully set up pinned
    /// state and its Accessor-owned pixel backing throughout all calls. Output must
    /// be disjoint from source image pixels, coordinate storage and retained context/state.
    unsafe fn shadeSpan(&mut self, mut x: i32, y: i32, dst: *mut u32, mut count: i32) {
        // The concrete allocation is unchanged. Its image borrow is needed only by
        // setup; native pixel backing is retained by the state's arena Accessor.
        let state_ptr = self.fState.as_ptr().cast::<SkBitmapProcState<'_>>();
        let state = unsafe { &*state_ptr };
        // Native proc aliases use initialized Rust slices. Output may be uninitialized native
        // scratch, so initialize before forming a reference; all count pixels are then overwritten.
        unsafe {
            core::ptr::write_bytes(dst, 0, count as usize);
        }
        let output = unsafe { core::slice::from_raw_parts_mut(dst, count as usize) };
        if let Some(shader) = unsafe { state.getShaderProc32() } {
            unsafe {
                shader(state_ptr.cast(), x, y, output, count);
            }
            return;
        }
        const BUF_MAX: usize = 128;
        // Initialization is a Rust slice-validity adaptation; stack size/layout remains native.
        let mut buffer = [0u32; BUF_MAX];
        let mproc =
            unsafe { state.getMatrixProc() }.expect("successful setup supplies matrix proc");
        let sproc =
            unsafe { state.getSampleProc32() }.expect("successful setup supplies sample proc");
        let max = unsafe { state.maxCountForBufferSize(core::mem::size_of_val(&buffer)) };
        crate::SkASSERT!(@source b"src/shaders/SkBitmapProcShader.cpp\0".as_ptr(),50; @condition b"state.fPixmap.addr()\0".as_ptr(); !state.fPixmap.addr().is_null());
        let mut written = 0;
        loop {
            let n = count.min(max);
            crate::SkASSERT!(@source b"src/shaders/SkBitmapProcShader.cpp\0".as_ptr(),54; @condition b"n > 0 && n < BUF_MAX*2\0".as_ptr(); n>0 && n<(BUF_MAX*2)as i32);
            unsafe {
                mproc(state, &mut buffer, n, x, y);
                sproc(state, &buffer, n, &mut output[written..]);
            }
            count -= n;
            if count == 0 {
                break;
            }
            crate::SkASSERT!(@source b"src/shaders/SkBitmapProcShader.cpp\0".as_ptr(),61; @condition b"count > 0\0".as_ptr(); count>0);
            x += n;
            written += n as usize;
        }
    }
}
// Native inherited base state and non-owning pointer to the actual arena State.
// The data pointer does not create a borrow of the temporary shader/image header;
// shadeSpan projects the original concrete State only for the duration of that call.
// Source: src/shaders/SkBitmapProcShader.cpp:68-69
struct BitmapProcShaderContext {
    base: ContextState,
    fState: core::ptr::NonNull<()>,
    fFlags: u32,
}

// Source: src/shaders/SkBitmapProcShader.cpp:76-78
// Source: src/shaders/SkBitmapProcShader.cpp:80-82
// Source: src/shaders/SkBitmapProcShader.cpp:85-87
// Source: src/shaders/SkBitmapProcShader.cpp:89-89
/// Safety: alloc remains live and unreset through all returned-context calls. Shader/image
/// headers must remain live through construction/setup; their non-owning identities are not
/// dereferenced after setup. The Accessor retains the actual Bitmap/Mipmap pixel backing.
/// State is constructed before arena registration, moved exactly
/// once into final arena placement, then pinned before setup. No subsequent moves are permitted.
/// Returned context owns neither state nor shader/image: arena teardown destroys context/state;
/// callers never delete them individually or retain a mutable arena lease across their use.
#[cfg(all(target_arch = "aarch64", target_pointer_width = "64"))]
pub unsafe fn MakeContext(
    shader: &dyn SkShaderBase,
    tmx: crate::include::core::SkTileMode::SkTileMode,
    tmy: crate::include::core::SkTileMode::SkTileMode,
    sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
    image: Option<&dyn crate::src::image::SkImage_Base::SkImage_Base>,
    rec: &ContextRec<'_>,
    alloc: *mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
) -> Result<Option<*mut dyn Context>, crate::RasterError> {
    let Some(totalInverse) = rec.fMatrixRec.totalInverse() else {
        return Ok(None);
    };
    // Constructors may panic: finish construction before make_with registers a destructor footer.
    let state_value = SkBitmapProcState::new(image, tmx, tmy);
    let state = unsafe { (*alloc).make_with(|_, _| state_value) };
    let pinned = unsafe { core::pin::Pin::new_unchecked(&mut *state) };
    if !pinned.setup(&totalInverse, rec.fPaintAlpha as u32, sampling)? {
        return Ok(None);
    }
    let context_value = unsafe { BitmapProcShaderContext::new(shader, rec, state) };
    let context = unsafe { (*alloc).make_with(|_, _| context_value) };
    Ok(Some(context as *mut dyn Context))
}
