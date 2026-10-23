// Copyright 2024 Google LLC. BSD-style license.
// Native CPU utilities, passes and A8/RGBA algorithms. The shader-based
// shader execution now calls the real RuntimeEffect builder and device owner.
// Current translation is UNVERIFIED; all validation is prohibited.

// Source: src/core/SkBlurEngine.h:65-65
pub const fn IsEffectivelyIdentity(sigma: f32) -> bool {
    sigma <= 0.03
}

// Source: src/core/SkBlurEngine.h:69-69
// Source: src/core/SkBlurEngine.h:71-71
pub fn SigmaToRadius(sigma: f32) -> i32 {
    if IsEffectivelyIdentity(sigma) {
        0
    } else {
        crate::src::core::SkFloatingPoint::sk_float_ceil2int(3.0 * sigma)
    }
}

// Source: src/core/SkBlurEngine.h:89-91
pub fn BoxBlurWindow(sigma: f32) -> i32 {
    let possible_window = crate::src::core::SkFloatingPoint::sk_float_floor2int(
        sigma * 3.0 * (2.0 * crate::src::core::SkFloatingPoint::SK_FloatPI).sqrt() / 4.0 + 0.5,
    );
    1.max(possible_window)
}

// Source: src/core/SkBlurEngine.h:57-58
pub trait SkBlurEngine: Sync {
    fn findAlgorithm(
        &self,
        sigma: crate::include::core::SkSize::SkSize,
        color_type: crate::include::core::SkColorType::SkColorType,
    ) -> Option<&dyn Algorithm>;
}
// Source: src/core/SkBlurEngine.h:97-127
pub trait Algorithm: Sync {
    fn maxSigma(&self) -> f32;
    fn supportsOnlyDecalTiling(&self) -> bool;
    /// Safety: source backing and all pixel/context resources retain their native
    /// synchronous CPU leases; source/destination bounds and sigma match this algorithm.
    unsafe fn blur(
        &self,
        sigma: crate::include::core::SkSize::SkSize,
        input: std::sync::Arc<dyn crate::src::core::SkSpecialImage::SkSpecialImage>,
        src_rect: &crate::src::core::SkRect::SkIRect,
        tile_mode: crate::include::core::SkTileMode::SkTileMode,
        dst_rect: &crate::src::core::SkRect::SkIRect,
    ) -> Result<
        Option<std::sync::Arc<dyn crate::src::core::SkSpecialImage::SkSpecialImage>>,
        crate::RasterError,
    >;
}

// Source: src/core/SkBlurEngine.h:176-176
pub const fn KernelWidth(radius: i32) -> i32 {
    2 * radius + 1
}
// Source: src/core/SkBlurEngine.h:180-180
pub const fn LinearKernelWidth(radius: i32) -> i32 {
    radius + 1
}
// Source: src/core/SkBlurEngine.h:188-188
pub const kMaxSamples: usize = 28;
// Source: src/core/SkBlurEngine.h:194-194
pub const kMaxLinearSigma: f32 = 4.0;

// Source: src/core/SkBlurEngine.h:257-258
pub fn Compute1DBlurKernel(sigma: f32, radius: i32, kernel: &mut [f32]) {
    use crate::include::core::SkSize::{SkISize, SkSize};
    Compute2DBlurKernel(SkSize::Make(sigma, 0.0), SkISize::Make(radius, 0), kernel);
}

// Rust's associated pixel type expresses the source template's supported eight-bit channel pixels.
mod pixel_private {
    pub trait Sealed {}
    impl Sealed for u8 {}
    impl Sealed for u32 {}
}
pub trait BlurPixel: pixel_private::Sealed + Copy {
    const ZERO: Self;
}
impl BlurPixel for u8 {
    const ZERO: Self = 0;
}
impl BlurPixel for u32 {
    const ZERO: Self = 0;
}

// Source: src/core/SkBlurEngine.cpp:62-62
// Source: src/core/SkBlurEngine.cpp:64-65
// Source: src/core/SkBlurEngine.cpp:68-72
// Source: src/core/SkBlurEngine.cpp:74-78
// Source: src/core/SkBlurEngine.cpp:80-81
// Source: src/core/SkBlurEngine.cpp:83-83
// Source: src/core/SkBlurEngine.cpp:87-90
// Source: src/core/SkBlurEngine.cpp:92-92
// Source: src/core/SkBlurEngine.cpp:94-94
// Source: src/core/SkBlurEngine.cpp:97-97
// Source: src/core/SkBlurEngine.cpp:99-102
// Source: src/core/SkBlurEngine.cpp:104-104
// Source: src/core/SkBlurEngine.cpp:106-108
// Source: src/core/SkBlurEngine.cpp:112-112
// Source: src/core/SkBlurEngine.cpp:115-115
// Source: src/core/SkBlurEngine.cpp:117-122
// Source: src/core/SkBlurEngine.cpp:126-128
// Source: src/core/SkBlurEngine.cpp:133-135
// Source: src/core/SkBlurEngine.cpp:138-138
pub trait Pass {
    type Pixel: BlurPixel;
    fn border(&self) -> i32;
    fn startBlur(&mut self);
    unsafe fn blurSegment(
        &mut self,
        n: i32,
        src: *const Self::Pixel,
        src_stride: i32,
        dst: *mut Self::Pixel,
        dst_stride: i32,
    );
    /// Safety: the provided source and destination strided spans must be live for each accessed pixel.
    unsafe fn blur(
        &mut self,
        src_left: i32,
        src_right: i32,
        dst_right: i32,
        src: *const Self::Pixel,
        src_stride: i32,
        dst: *mut Self::Pixel,
        dst_stride: i32,
    ) {
        self.startBlur();
        let src_start = src_left - self.border();
        let src_end = src_right - self.border();
        let dst_end = dst_right;
        let (mut src_idx, mut dst_idx) = (src_start, 0);
        let (mut src_cursor, mut dst_cursor) = (src, dst);
        if dst_idx < src_idx {
            let common_end = src_idx.min(dst_end);
            while dst_idx < common_end {
                unsafe {
                    *dst_cursor = Self::Pixel::ZERO;
                }
                dst_cursor = dst_cursor.wrapping_offset(dst_stride as isize);
                // Source prefetch is a performance-only hint, not emitted by the portable Rust path.
                dst_idx += 1;
            }
        } else if src_idx < dst_idx {
            let common_end = dst_idx.min(src_end);
            if src_idx < common_end {
                let n = common_end - src_idx;
                unsafe {
                    self.blurSegment(n, src_cursor, src_stride, core::ptr::null_mut(), 0);
                }
                src_idx += n;
                src_cursor = src_cursor.wrapping_offset((n * src_stride) as isize);
            }
            if src_idx < dst_idx {
                let n = dst_idx - src_idx;
                unsafe {
                    self.blurSegment(n, core::ptr::null(), 0, core::ptr::null_mut(), 0);
                }
                src_idx += n;
            }
        }
        let common_end = dst_end.min(src_end);
        if dst_idx < common_end {
            crate::SkASSERT!(@source b"src/core/SkBlurEngine.cpp\0".as_ptr(),115; @condition b"srcIdx == dstIdx\0".as_ptr(); (src_idx)==( dst_idx));
            let n = common_end - dst_idx;
            unsafe {
                self.blurSegment(n, src_cursor, src_stride, dst_cursor, dst_stride);
            }
            src_cursor = src_cursor.wrapping_offset((n * src_stride) as isize);
            dst_cursor = dst_cursor.wrapping_offset((n * dst_stride) as isize);
            dst_idx += n;
            src_idx += n;
        }
        if dst_idx < dst_end {
            let n = dst_end - dst_idx;
            unsafe {
                self.blurSegment(n, core::ptr::null(), 0, dst_cursor, dst_stride);
            }
        }
    }
}

// Source: src/core/SkBlurEngine.cpp:141-141
// Source: src/core/SkBlurEngine.cpp:143-149
// Source: src/core/SkBlurEngine.cpp:152-153
pub struct PassMakerData {
    fWindow: i32,
    fSigma: f32,
}
impl PassMakerData {
    fn new(window: i32, sigma: f32) -> Self {
        Self {
            fWindow: window,
            fSigma: sigma,
        }
    }
}
// Associated pixels and native raw-pointer dispatch retain runtime maker selection. Scratch
// remains caller-owned; no Box/Vec or retained backing allocation is introduced.
pub trait PassMaker {
    type Pixel: BlurPixel;
    fn data(&self) -> &PassMakerData;
    fn window(&self) -> i32 {
        self.data().fWindow
    }
    fn sigma(&self) -> f32 {
        self.data().fSigma
    }
    fn bufferSizeBytes(&self) -> usize;
    /// Safety: arena and initialized scratch remain live until all passes using them are done.
    /// Scratch has bufferSizeBytes bytes with native vector alignment (16 for RGBA,4 for A8).
    /// Only one pass may access shared scratch at a time; before reusing it the prior pass must
    /// never be dereferenced/reborrowed again. The old arena object contains only trivially dropped
    /// reference fields; no destructor may inspect invalidated scratch. No pass object is moved
    /// or dropped separately from its native arena. u8/u32 dispatch is kept distinct.
    unsafe fn makePass<'a>(
        &self,
        buffer: *mut u8,
        alloc: *mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    ) -> *mut (dyn Pass<Pixel = Self::Pixel> + 'a);
}
// Source: src/core/SkBlurEngine.cpp:158-269
/// Safety: source backing is readable through the full native strided region;
/// the input bitmap and all pass scratch have their synchronous CPU leases.
unsafe fn eval_blur_passes<T: BlurPixel>(
    maker_x: &dyn PassMaker<Pixel = T>,
    maker_y: &dyn PassMaker<Pixel = T>,
    mut src: crate::src::core::SkBitmap::SkBitmap,
    original_src: &crate::src::core::SkRect::SkIRect,
    original_dst: &crate::src::core::SkRect::SkIRect,
    alloc: *mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
) -> Result<
    Option<std::sync::Arc<dyn crate::src::core::SkSpecialImage::SkSpecialImage>>,
    crate::RasterError,
> {
    let mut src_bounds = *original_src;
    let mut dst_bounds = *original_dst;
    if maker_x.window() > 1 {
        dst_bounds.outset(0, SigmaToRadius(maker_y.sigma()));
    }
    let mut dst = crate::src::core::SkBitmap::SkBitmap::default();
    let origin = dst_bounds.top_left();
    if !dst.tryAllocPixels(
        &src.info().makeWH(dst_bounds.width(), dst_bounds.height()),
        0,
    ) {
        return Ok(None);
    }
    dst.eraseColor(crate::src::core::SkColor::SK_ColorTRANSPARENT)?;
    let buffer_size = maker_x.bufferSizeBytes().max(maker_y.bufferSizeBytes());
    let buffer = (*alloc).makeBytesAlignedTo(buffer_size, core::mem::size_of::<T>() * 4);
    // Rust adaptation: makePass constructs typed scratch slices, so initialize
    // the complete region before publishing those references. Reuse is sequential.
    core::ptr::write_bytes(buffer, 0, buffer_size);
    let mut loop_start = src_bounds.fLeft.max(dst_bounds.fLeft);
    let mut loop_end = src_bounds.fRight.min(dst_bounds.fRight);
    let mut dst_y_offset = 0;
    if maker_x.window() > 1 {
        loop_start = src_bounds.fTop.max(dst_bounds.fTop);
        loop_end = src_bounds.fBottom.min(dst_bounds.fBottom);
        if loop_start < loop_end {
            let mut src_addr = src.getAddr(0, loop_start - src_bounds.fTop).cast::<T>();
            let mut dst_addr = dst.getAddr(0, loop_start - dst_bounds.fTop).cast::<T>();
            let pass = maker_x.makePass(buffer, alloc);
            for _ in loop_start..loop_end {
                (*pass).blur(
                    src_bounds.fLeft - dst_bounds.fLeft,
                    src_bounds.fRight - dst_bounds.fLeft,
                    dst_bounds.width(),
                    src_addr,
                    1,
                    dst_addr,
                    1,
                );
                src_addr = src_addr.wrapping_offset(src.rowBytesAsPixels() as isize);
                dst_addr = dst_addr.wrapping_offset(dst.rowBytesAsPixels() as isize);
            }
        }
        // Native Bitmap copy shares the same PixelRef; no image snapshot is made.
        src = dst.clone();
        loop_start = original_dst.fLeft;
        loop_end = original_dst.fRight;
        dst_y_offset = original_dst.fTop - dst_bounds.fTop;
        src_bounds = dst_bounds;
        dst_bounds = *original_dst;
    }
    if maker_y.window() > 1 && loop_start < loop_end {
        let mut src_addr = src.getAddr(loop_start - src_bounds.fLeft, 0).cast::<T>();
        let mut dst_addr = dst
            .getAddr(loop_start - dst_bounds.fLeft, dst_y_offset)
            .cast::<T>();
        // The previous pass and every scratch borrow it contained are now dead.
        let pass = maker_y.makePass(buffer, alloc);
        for _ in loop_start..loop_end {
            (*pass).blur(
                src_bounds.fTop - dst_bounds.fTop,
                src_bounds.fBottom - dst_bounds.fTop,
                dst_bounds.height(),
                src_addr,
                src.rowBytesAsPixels(),
                dst_addr,
                dst.rowBytesAsPixels(),
            );
            src_addr = src_addr.wrapping_add(1);
            dst_addr = dst_addr.wrapping_add(1);
        }
    }
    // Source: src/core/SkBlurEngine.cpp:259-263
    // Retain the native identity copy when the build omits the Gaussian path.
    // Both bitmaps retain their real PixelRef backing across this synchronous
    // copy; propagate actual conversion errors rather than dropping them.
    // Native defined/undefined switch maps to presence (even an empty value)
    // of this compile-time environment setting; no runtime read is introduced.
    if option_env!("SK_AVOID_SLOW_RASTER_PIPELINE_BLURS").is_some()
        && maker_x.window() == 1
        && maker_y.window() == 1
    {
        unsafe {
            dst.writePixels(
                src.pixmap(),
                src_bounds.left() - dst_bounds.left(),
                src_bounds.top() - dst_bounds.top(),
            )
        }?;
    }
    dst.setImmutable();
    dst_bounds = original_dst.make_offset(-origin.fX, -origin.fY);
    Ok(crate::src::core::SkSpecialImage::MakeFromRasterBitmap(
        dst_bounds,
        &dst,
        crate::include::core::SkSurfaceProps::SkSurfaceProps::default(),
    ))
}

// Source: src/core/SkBlurEngine.cpp:280-280
// Source: src/core/SkBlurEngine.cpp:282-283
// Source: src/core/SkBlurEngine.cpp:285-285
// Source: src/core/SkBlurEngine.cpp:287-290
// Source: src/core/SkBlurEngine.cpp:292-292
// Source: src/core/SkBlurEngine.cpp:294-294
// Source: src/core/SkBlurEngine.cpp:299-299
pub struct GaussianPassMaker<T: BlurPixel + 'static, const N: usize> {
    data: PassMakerData,
    pixel: core::marker::PhantomData<T>,
}
impl<T: BlurPixel + 'static, const N: usize> GaussianPassMaker<T, N> {
    pub const kMaxSigma: f32 = 2.0;
    /// Safety: finite nonnegative sigma and valid arena; pixel width N equals size_of<T>.
    pub unsafe fn MakeMaker(
        sigma: f32,
        alloc: *mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    ) -> Option<*mut Self> {
        if sigma >= Self::kMaxSigma {
            return None;
        }
        let maker = Self {
            data: PassMakerData::new(2 * SigmaToRadius(sigma) + 1, sigma),
            pixel: core::marker::PhantomData,
        };
        Some(unsafe { (*alloc).make(maker) })
    }
}
impl<T: BlurPixel + 'static, const N: usize> PassMaker for GaussianPassMaker<T, N> {
    type Pixel = T;
    fn data(&self) -> &PassMakerData {
        &self.data
    }
    unsafe fn makePass<'a>(
        &self,
        buffer: *mut u8,
        alloc: *mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    ) -> *mut (dyn Pass<Pixel = T> + 'a) {
        let scratch: &'a mut [f32] = unsafe {
            core::slice::from_raw_parts_mut(
                buffer.cast(),
                self.bufferSizeBytes() / core::mem::size_of::<f32>(),
            )
        };
        let pass = GaussianPass::<T, N>::Make(self.sigma(), scratch);
        unsafe { (*alloc).make_with(|_, _| pass) }
    }
    fn bufferSizeBytes(&self) -> usize {
        self.window() as usize * (core::mem::size_of::<[f32; N]>() + core::mem::size_of::<f32>())
    }
}

// Source: src/core/SkBlurEngine.cpp:302-304
// Source: src/core/SkBlurEngine.cpp:306-306
// Source: src/core/SkBlurEngine.cpp:308-309
// Source: src/core/SkBlurEngine.cpp:311-311
// Source: src/core/SkBlurEngine.cpp:315-320
// Borrowed scratch slices retain the original arena-provided storage lifetime. The pass object moves by value.
pub struct GaussianPass<'a, T: BlurPixel, const N: usize> {
    fBorder: i32,
    fWindow: usize,
    fKernel: &'a [f32],
    fSrcBuffer: &'a mut [[f32; N]],
    fSrcBufferBase: usize,
    pixel: core::marker::PhantomData<T>,
}
impl<'a, T: BlurPixel, const N: usize> GaussianPass<'a, T, N> {
    pub fn Make(sigma: f32, buffers: &'a mut [f32]) -> Self {
        // Source: src/core/SkBlurEngine.cpp:277-278
        assert!(N * core::mem::size_of::<u8>() == core::mem::size_of::<T>());
        let radius = SigmaToRadius(sigma);
        let kernel_width = (2 * radius + 1) as usize;
        let (src, kernel) = buffers.split_at_mut(kernel_width * N);
        let kernel = &mut kernel[..kernel_width];
        let src_buffer = unsafe {
            core::slice::from_raw_parts_mut(src.as_mut_ptr().cast::<[f32; N]>(), kernel_width)
        };
        Compute1DBlurKernel(sigma, radius, kernel);
        Self::new(radius, kernel, src_buffer)
    }
    pub fn new(radius: i32, kernel: &'a [f32], src_buffer: &'a mut [[f32; N]]) -> Self {
        Self {
            fBorder: radius,
            fWindow: (2 * radius + 1) as usize,
            fKernel: kernel,
            fSrcBuffer: src_buffer,
            fSrcBufferBase: 0,
            pixel: core::marker::PhantomData,
        }
    }
}

// Source: src/core/SkBlurEngine.cpp:323-323
// Source: src/core/SkBlurEngine.cpp:325-325
// Source: src/core/SkBlurEngine.cpp:327-327
// Source: src/core/SkBlurEngine.cpp:330-332
// Source: src/core/SkBlurEngine.cpp:335-335
// Source: src/core/SkBlurEngine.cpp:337-341
// Source: src/core/SkBlurEngine.cpp:343-345
// Source: src/core/SkBlurEngine.cpp:348-351
// Source: src/core/SkBlurEngine.cpp:354-354
// Source: src/core/SkBlurEngine.cpp:357-359
// Source: src/core/SkBlurEngine.cpp:363-364
// Source: src/core/SkBlurEngine.cpp:366-366
// Source: src/core/SkBlurEngine.cpp:369-369
// Source: src/core/SkBlurEngine.cpp:372-375
impl<T: BlurPixel, const N: usize> Pass for GaussianPass<'_, T, N> {
    type Pixel = T;
    fn border(&self) -> i32 {
        self.fBorder
    }
    fn startBlur(&mut self) {
        self.fSrcBuffer[..self.fWindow].fill([0.0; N]);
        self.fSrcBufferBase = 0;
    }
    unsafe fn blurSegment(
        &mut self,
        mut n: i32,
        mut src_ptr: *const T,
        src_stride: i32,
        mut dst_ptr: *mut T,
        dst_stride: i32,
    ) {
        let mut base = self.fSrcBufferBase;
        while n > 0 {
            n -= 1;
            let leading_edge = if !src_ptr.is_null() {
                core::array::from_fn(
                    |i| unsafe { *src_ptr.cast::<u8>().add(i) } as f32 * (1.0 / 255.0),
                )
            } else {
                [0.0; N]
            };
            self.fSrcBuffer[(base + self.fWindow - 1) % self.fWindow] = leading_edge;
            if !dst_ptr.is_null() {
                let mut sum = [0.0; N];
                for i in 0..self.fWindow {
                    let s = (i + base) % self.fWindow;
                    for c in 0..N {
                        sum[c] += self.fSrcBuffer[s][c] * self.fKernel[i];
                    }
                }
                for c in 0..N {
                    // skvx::pin is min(max(x,0),255); finite convolution weights bound these values.
                    let value = (sum[c] * 255.0 + 0.5).max(0.0).min(255.0) as u8;
                    unsafe {
                        *dst_ptr.cast::<u8>().add(c) = value;
                    }
                }
                dst_ptr = dst_ptr.wrapping_offset(dst_stride as isize);
            }
            if !src_ptr.is_null() {
                src_ptr = src_ptr.wrapping_offset(src_stride as isize);
            }
            base = (base + 1) % self.fWindow;
        }
        self.fSrcBufferBase = base;
    }
}

// Source: src/core/SkBlurEngine.cpp:395-399
// Source: src/core/SkBlurEngine.cpp:402-402
// Source: src/core/SkBlurEngine.cpp:404-406
// Source: src/core/SkBlurEngine.cpp:409-411
// Source: src/core/SkBlurEngine.cpp:419-420
// Source: src/core/SkBlurEngine.cpp:424-424
pub struct ThreeBoxApproxPassMaker {
    data: PassMakerData,
}
impl ThreeBoxApproxPassMaker {
    /// Safety: sigma is in the native finite nonnegative supported range and arena remains live.
    pub unsafe fn MakeMaker(
        sigma: f32,
        alloc: *mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    ) -> Option<*mut Self> {
        crate::SkASSERT!(@source b"src/core/SkBlurEngine.cpp\0".as_ptr(),396; @condition b"0 <= sigma\0".as_ptr(); 0.0 <= sigma);
        let window = BoxBlurWindow(sigma);
        if 255 <= window {
            return None;
        }
        Some(unsafe {
            (*alloc).make(Self {
                data: PassMakerData::new(window, sigma),
            })
        })
    }
}
impl PassMaker for ThreeBoxApproxPassMaker {
    type Pixel = u32;
    fn data(&self) -> &PassMakerData {
        &self.data
    }
    unsafe fn makePass<'a>(
        &self,
        buffer: *mut u8,
        alloc: *mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    ) -> *mut (dyn Pass<Pixel = u32> + 'a) {
        let scratch: &'a mut [[u32; 4]] = unsafe {
            core::slice::from_raw_parts_mut(
                buffer.cast(),
                self.bufferSizeBytes() / core::mem::size_of::<[u32; 4]>(),
            )
        };
        unsafe { (*alloc).make_with(|_, _| ThreeBoxApproxPass::Make(self.window(), scratch)) }
    }
    fn bufferSizeBytes(&self) -> usize {
        let onePassSize = (self.window() - 1) as usize;
        let bufferCount = if (self.window() & 1) == 1 {
            3 * onePassSize
        } else {
            3 * onePassSize + 1
        };
        bufferCount * core::mem::size_of::<[u32; 4]>()
    }
}

// Source: src/core/SkBlurEngine.cpp:427-427
// Source: src/core/SkBlurEngine.cpp:429-432
// Source: src/core/SkBlurEngine.cpp:435-435
// Source: src/core/SkBlurEngine.cpp:480-480
// Source: src/core/SkBlurEngine.cpp:484-488
// Source: src/core/SkBlurEngine.cpp:491-502
// Shared arena scratch becomes a borrowed contiguous array span; offsets retain all three ring ranges.
pub struct ThreeBoxApproxPass<'a> {
    fBorder: i32,
    buffer: &'a mut [[u32; 4]],
    starts: [usize; 3],
    ends: [usize; 3],
    fDivider: crate::src::core::SkVx::ScaledDividerU32,
    fSum0: [u32; 4],
    fSum1: [u32; 4],
    fSum2: [u32; 4],
    cursors: [usize; 3],
}
impl<'a> ThreeBoxApproxPass<'a> {
    pub fn Make(window: i32, buffers: &'a mut [[u32; 4]]) -> Self {
        let pass_size = (window - 1) as usize;
        let buffer1 = pass_size;
        let buffer2 = buffer1 + pass_size;
        let buffers_end = buffer2
            + if (window & 1) != 0 {
                pass_size
            } else {
                pass_size + 1
            };
        let border = if (window & 1) == 1 {
            3 * ((window - 1) / 2)
        } else {
            3 * (window / 2) - 1
        };
        let window2 = window * window;
        let window3 = window2 * window;
        let divisor = if (window & 1) == 1 {
            window3
        } else {
            window3 + window2
        };
        Self {
            fBorder: border,
            buffer: &mut buffers[..buffers_end],
            starts: [0, buffer1, buffer2],
            ends: [buffer1, buffer2, buffers_end],
            fDivider: crate::src::core::SkVx::ScaledDividerU32::new(divisor as u32),
            fSum0: [0; 4],
            fSum1: [0; 4],
            fSum2: [0; 4],
            cursors: [0, buffer1, buffer2],
        }
    }

    #[cfg(all(target_arch = "aarch64", not(skvx_disable_simd)))]
    unsafe fn blur_segment_neon(
        &mut self,
        mut n: i32,
        mut src32: *const u32,
        src_stride: i32,
        mut dst32: *mut u32,
        dst_stride: i32,
    ) {
        use core::arch::aarch64::*;

        let mut cursor = self.cursors;
        let mut sum0 = unsafe { vld1q_u32(self.fSum0.as_ptr()) };
        let mut sum1 = unsafe { vld1q_u32(self.fSum1.as_ptr()) };
        let mut sum2 = unsafe { vld1q_u32(self.fSum2.as_ptr()) };
        let factor = self.fDivider.divisorFactor();

        while n > 0 {
            n -= 1;
            let leading = if src32.is_null() {
                unsafe { vdupq_n_u32(0) }
            } else {
                let packed = unsafe { core::ptr::read_unaligned(src32) };
                let bytes = unsafe { vcreate_u8(packed as u64) };
                unsafe { vmovl_u16(vget_low_u16(vmovl_u8(bytes))) }
            };

            sum0 = unsafe { vaddq_u32(sum0, leading) };
            sum1 = unsafe { vaddq_u32(sum1, sum0) };
            sum2 = unsafe { vaddq_u32(sum2, sum1) };
            let lo = unsafe { vmull_n_u32(vget_low_u32(sum2), factor) };
            let hi = unsafe { vmull_n_u32(vget_high_u32(sum2), factor) };
            let blurred = unsafe { vcombine_u32(vshrn_n_u64::<32>(lo), vshrn_n_u64::<32>(hi)) };

            let trailing2 = unsafe { vld1q_u32(self.buffer[cursor[2]].as_ptr()) };
            sum2 = unsafe { vsubq_u32(sum2, trailing2) };
            unsafe { vst1q_u32(self.buffer[cursor[2]].as_mut_ptr(), sum1) };
            cursor[2] = if cursor[2] + 1 < self.ends[2] {
                cursor[2] + 1
            } else {
                self.starts[2]
            };

            let trailing1 = unsafe { vld1q_u32(self.buffer[cursor[1]].as_ptr()) };
            sum1 = unsafe { vsubq_u32(sum1, trailing1) };
            unsafe { vst1q_u32(self.buffer[cursor[1]].as_mut_ptr(), sum0) };
            cursor[1] = if cursor[1] + 1 < self.ends[1] {
                cursor[1] + 1
            } else {
                self.starts[1]
            };

            let trailing0 = unsafe { vld1q_u32(self.buffer[cursor[0]].as_ptr()) };
            sum0 = unsafe { vsubq_u32(sum0, trailing0) };
            unsafe { vst1q_u32(self.buffer[cursor[0]].as_mut_ptr(), leading) };
            cursor[0] = if cursor[0] + 1 < self.ends[0] {
                cursor[0] + 1
            } else {
                self.starts[0]
            };

            if !dst32.is_null() {
                let words = unsafe { vmovn_u32(blurred) };
                let bytes = unsafe { vmovn_u16(vcombine_u16(words, vdup_n_u16(0))) };
                unsafe {
                    core::ptr::write_unaligned(
                        dst32,
                        vget_lane_u32::<0>(vreinterpret_u32_u8(bytes)),
                    );
                }
                dst32 = dst32.wrapping_offset(dst_stride as isize);
            }
            if !src32.is_null() {
                src32 = src32.wrapping_offset(src_stride as isize);
            }
        }

        unsafe {
            vst1q_u32(self.fSum0.as_mut_ptr(), sum0);
            vst1q_u32(self.fSum1.as_mut_ptr(), sum1);
            vst1q_u32(self.fSum2.as_mut_ptr(), sum2);
        }
        self.cursors = cursor;
    }
}
// Source: src/core/SkBlurEngine.cpp:505-511
// Source: src/core/SkBlurEngine.cpp:513-515
// Source: src/core/SkBlurEngine.cpp:556-559
// Source: src/core/SkBlurEngine.cpp:634-639
// Source: src/core/SkBlurEngine.cpp:642-645
// Source: src/core/SkBlurEngine.cpp:647-647
// Source: src/core/SkBlurEngine.cpp:649-657
// Source: src/core/SkBlurEngine.cpp:659-659
// Source: src/core/SkBlurEngine.cpp:662-663
// Source: src/core/SkBlurEngine.cpp:666-668
// Source: src/core/SkBlurEngine.cpp:670-673
// Source: src/core/SkBlurEngine.cpp:675-678
// Source: src/core/SkBlurEngine.cpp:680-684
// Source: src/core/SkBlurEngine.cpp:689-691
// Source: src/core/SkBlurEngine.cpp:693-695
// Source: src/core/SkBlurEngine.cpp:699-703
// Source: src/core/SkBlurEngine.cpp:706-711
impl Pass for ThreeBoxApproxPass<'_> {
    type Pixel = u32;
    fn border(&self) -> i32 {
        self.fBorder
    }
    fn startBlur(&mut self) {
        self.fSum0 = [0; 4];
        self.fSum1 = [0; 4];
        self.fSum2 = [self.fDivider.half(); 4];
        self.buffer.fill([0; 4]);
        self.cursors = self.starts;
    }
    unsafe fn blurSegment(
        &mut self,
        mut n: i32,
        mut src32: *const u32,
        src_stride: i32,
        mut dst32: *mut u32,
        dst_stride: i32,
    ) {
        #[cfg(all(target_arch = "aarch64", not(skvx_disable_simd)))]
        {
            unsafe { self.blur_segment_neon(n, src32, src_stride, dst32, dst_stride) };
            return;
        }
        let mut cursor = self.cursors;
        let (mut sum0, mut sum1, mut sum2) = (self.fSum0, self.fSum1, self.fSum2);
        let mut process_value = |leading_edge: [u32; 4]| -> [u8; 4] {
            for c in 0..4 {
                sum0[c] = sum0[c].wrapping_add(leading_edge[c]);
                sum1[c] = sum1[c].wrapping_add(sum0[c]);
                sum2[c] = sum2[c].wrapping_add(sum1[c]);
            }
            let blurred = self.fDivider.divide4(sum2);
            for c in 0..4 {
                sum2[c] = sum2[c].wrapping_sub(self.buffer[cursor[2]][c]);
            }
            self.buffer[cursor[2]] = sum1;
            cursor[2] = if cursor[2] + 1 < self.ends[2] {
                cursor[2] + 1
            } else {
                self.starts[2]
            };
            for c in 0..4 {
                sum1[c] = sum1[c].wrapping_sub(self.buffer[cursor[1]][c]);
            }
            self.buffer[cursor[1]] = sum0;
            cursor[1] = if cursor[1] + 1 < self.ends[1] {
                cursor[1] + 1
            } else {
                self.starts[1]
            };
            for c in 0..4 {
                sum0[c] = sum0[c].wrapping_sub(self.buffer[cursor[0]][c]);
            }
            self.buffer[cursor[0]] = leading_edge;
            cursor[0] = if cursor[0] + 1 < self.ends[0] {
                cursor[0] + 1
            } else {
                self.starts[0]
            };
            core::array::from_fn(|c| blurred[c] as u8)
        };
        let load_edge = |ptr: *const u32| -> [u32; 4] {
            core::array::from_fn(|c| unsafe { *ptr.cast::<u8>().add(c) } as u32)
        };
        if src32.is_null() && dst32.is_null() {
            while n > 0 {
                n -= 1;
                let _ = process_value([0; 4]);
            }
        } else if !src32.is_null() && dst32.is_null() {
            while n > 0 {
                n -= 1;
                let _ = process_value(load_edge(src32));
                src32 = src32.wrapping_offset(src_stride as isize);
            }
        } else if src32.is_null() && !dst32.is_null() {
            while n > 0 {
                n -= 1;
                let value = process_value([0; 4]);
                unsafe {
                    core::ptr::copy_nonoverlapping(value.as_ptr(), dst32.cast::<u8>(), 4);
                }
                dst32 = dst32.wrapping_offset(dst_stride as isize);
            }
        } else if !src32.is_null() && !dst32.is_null() {
            while n > 0 {
                n -= 1;
                let value = process_value(load_edge(src32));
                unsafe {
                    core::ptr::copy_nonoverlapping(value.as_ptr(), dst32.cast::<u8>(), 4);
                }
                src32 = src32.wrapping_offset(src_stride as isize);
                dst32 = dst32.wrapping_offset(dst_stride as isize);
            }
        }
        self.cursors = cursor;
        self.fSum0 = sum0;
        self.fSum1 = sum1;
        self.fSum2 = sum2;
    }
}

// Source: src/core/SkBlurEngine.cpp:729-731
// Source: src/core/SkBlurEngine.cpp:737-739
// Source: src/core/SkBlurEngine.cpp:742-742
// Source: src/core/SkBlurEngine.cpp:744-746
// Source: src/core/SkBlurEngine.cpp:749-750
// Source: src/core/SkBlurEngine.cpp:757-758
// Source: src/core/SkBlurEngine.cpp:762-762
pub struct TentPassMaker {
    data: PassMakerData,
}
impl TentPassMaker {
    /// Safety: sigma is in the native finite nonnegative supported range and arena remains live.
    pub unsafe fn MakeMaker(
        sigma: f32,
        alloc: *mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    ) -> Option<*mut Self> {
        crate::SkASSERT!(@source b"src/core/SkBlurEngine.cpp\0".as_ptr(),730; @condition b"0 <= sigma\0".as_ptr(); 0.0 <= sigma);
        let window = 3 * BoxBlurWindow(sigma) / 2;
        if window >= 4104 {
            return None;
        }
        Some(unsafe {
            (*alloc).make(Self {
                data: PassMakerData::new(window, sigma),
            })
        })
    }
}
impl PassMaker for TentPassMaker {
    type Pixel = u32;
    fn data(&self) -> &PassMakerData {
        &self.data
    }
    unsafe fn makePass<'a>(
        &self,
        buffer: *mut u8,
        alloc: *mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    ) -> *mut (dyn Pass<Pixel = u32> + 'a) {
        let scratch: &'a mut [[u32; 4]] = unsafe {
            core::slice::from_raw_parts_mut(
                buffer.cast(),
                self.bufferSizeBytes() / core::mem::size_of::<[u32; 4]>(),
            )
        };
        unsafe {
            (*alloc).make_with(|_, _| {
                TentPass::Make(self.window(), scratch).expect("native maker window is below4104")
            })
        }
    }
    fn bufferSizeBytes(&self) -> usize {
        let onePassSize = (self.window() - 1) as usize;
        let bufferCount = 2 * onePassSize;
        bufferCount * core::mem::size_of::<[u32; 4]>()
    }
}

// Source: src/core/SkBlurEngine.cpp:765-767
// Source: src/core/SkBlurEngine.cpp:771-774
// Source: src/core/SkBlurEngine.cpp:815-815
// Source: src/core/SkBlurEngine.cpp:817-818
// Source: src/core/SkBlurEngine.cpp:821-830
pub struct TentPass<'a> {
    fBorder: i32,
    buffer: &'a mut [[u32; 4]],
    starts: [usize; 2],
    ends: [usize; 2],
    fDivider: crate::src::core::SkVx::ScaledDividerU32,
    fSum0: [u32; 4],
    fSum1: [u32; 4],
    cursors: [usize; 2],
}
impl<'a> TentPass<'a> {
    pub fn Make(window: i32, buffers: &'a mut [[u32; 4]]) -> Option<Self> {
        if window > 4104 {
            return None;
        }
        let pass_size = (window - 1) as usize;
        let buffer1 = pass_size;
        let buffers_end = buffer1 + pass_size;
        let border = window - 1;
        let divisor = window * window;
        Some(Self {
            fBorder: border,
            buffer: &mut buffers[..buffers_end],
            starts: [0, buffer1],
            ends: [buffer1, buffers_end],
            fDivider: crate::src::core::SkVx::ScaledDividerU32::new(divisor as u32),
            fSum0: [0; 4],
            fSum1: [0; 4],
            cursors: [0, buffer1],
        })
    }
}
// Source: src/core/SkBlurEngine.cpp:833-837
// Source: src/core/SkBlurEngine.cpp:839-840
// Source: src/core/SkBlurEngine.cpp:877-884
// Source: src/core/SkBlurEngine.cpp:887-889
// Source: src/core/SkBlurEngine.cpp:891-891
// Source: src/core/SkBlurEngine.cpp:893-898
// Source: src/core/SkBlurEngine.cpp:900-900
// Source: src/core/SkBlurEngine.cpp:903-904
// Source: src/core/SkBlurEngine.cpp:907-909
// Source: src/core/SkBlurEngine.cpp:911-914
// Source: src/core/SkBlurEngine.cpp:916-919
// Source: src/core/SkBlurEngine.cpp:921-925
// Source: src/core/SkBlurEngine.cpp:930-933
// Source: src/core/SkBlurEngine.cpp:936-939
// Source: src/core/SkBlurEngine.cpp:942-945
impl Pass for TentPass<'_> {
    type Pixel = u32;
    fn border(&self) -> i32 {
        self.fBorder
    }
    fn startBlur(&mut self) {
        self.fSum0 = [0; 4];
        self.fSum1 = [self.fDivider.half(); 4];
        self.buffer.fill([0; 4]);
        self.cursors = self.starts;
    }
    unsafe fn blurSegment(
        &mut self,
        mut n: i32,
        mut src32: *const u32,
        src_stride: i32,
        mut dst32: *mut u32,
        dst_stride: i32,
    ) {
        let mut cursor = self.cursors;
        let (mut sum0, mut sum1) = (self.fSum0, self.fSum1);
        let mut process_value = |leading_edge: [u32; 4]| -> [u8; 4] {
            for c in 0..4 {
                sum0[c] = sum0[c].wrapping_add(leading_edge[c]);
                sum1[c] = sum1[c].wrapping_add(sum0[c]);
            }
            let blurred = self.fDivider.divide4(sum1);
            for c in 0..4 {
                sum1[c] = sum1[c].wrapping_sub(self.buffer[cursor[1]][c]);
            }
            self.buffer[cursor[1]] = sum0;
            cursor[1] = if cursor[1] + 1 < self.ends[1] {
                cursor[1] + 1
            } else {
                self.starts[1]
            };
            for c in 0..4 {
                sum0[c] = sum0[c].wrapping_sub(self.buffer[cursor[0]][c]);
            }
            self.buffer[cursor[0]] = leading_edge;
            cursor[0] = if cursor[0] + 1 < self.ends[0] {
                cursor[0] + 1
            } else {
                self.starts[0]
            };
            core::array::from_fn(|c| blurred[c] as u8)
        };
        let load_edge = |ptr: *const u32| -> [u32; 4] {
            core::array::from_fn(|c| unsafe { *ptr.cast::<u8>().add(c) } as u32)
        };
        if src32.is_null() && dst32.is_null() {
            while n > 0 {
                n -= 1;
                let _ = process_value([0; 4]);
            }
        } else if !src32.is_null() && dst32.is_null() {
            while n > 0 {
                n -= 1;
                let _ = process_value(load_edge(src32));
                src32 = src32.wrapping_offset(src_stride as isize);
            }
        } else if src32.is_null() && !dst32.is_null() {
            while n > 0 {
                n -= 1;
                let value = process_value([0; 4]);
                unsafe {
                    core::ptr::copy_nonoverlapping(value.as_ptr(), dst32.cast::<u8>(), 4);
                }
                dst32 = dst32.wrapping_offset(dst_stride as isize);
            }
        } else if !src32.is_null() && !dst32.is_null() {
            while n > 0 {
                n -= 1;
                let value = process_value(load_edge(src32));
                unsafe {
                    core::ptr::copy_nonoverlapping(value.as_ptr(), dst32.cast::<u8>(), 4);
                }
                src32 = src32.wrapping_offset(src_stride as isize);
                dst32 = dst32.wrapping_offset(dst_stride as isize);
            }
        }
        self.cursors = cursor;
        self.fSum0 = sum0;
        self.fSum1 = sum1;
    }
}

// Source: src/core/SkBlurEngine.cpp:950-953
// Source: src/core/SkBlurEngine.cpp:955-955
// Source: src/core/SkBlurEngine.cpp:957-959
// Source: src/core/SkBlurEngine.cpp:962-967
// Source: src/core/SkBlurEngine.cpp:971-971
pub struct A8PassMaker {
    data: PassMakerData,
}
impl A8PassMaker {
    /// Safety: sigma is in the native finite nonnegative supported range and arena remains live.
    pub unsafe fn MakeMaker(
        sigma: f32,
        alloc: *mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    ) -> Option<*mut Self> {
        crate::SkASSERT!(@source b"src/core/SkBlurEngine.cpp\0".as_ptr(),951; @condition b"0 <= sigma\0".as_ptr(); 0.0 <= sigma);
        let window = 1.max(
            (((sigma * 3.0) as f64 * (2.0 * crate::src::core::SkFloatingPoint::SK_DoublePI).sqrt()
                / 4.0
                + 0.5)
                .floor()) as i32,
        );
        Some(unsafe {
            (*alloc).make(Self {
                data: PassMakerData::new(window, sigma),
            })
        })
    }
}
impl PassMaker for A8PassMaker {
    type Pixel = u8;
    fn data(&self) -> &PassMakerData {
        &self.data
    }
    unsafe fn makePass<'a>(
        &self,
        buffer: *mut u8,
        alloc: *mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    ) -> *mut (dyn Pass<Pixel = u8> + 'a) {
        let scratch: &'a mut [u32] = unsafe {
            core::slice::from_raw_parts_mut(
                buffer.cast(),
                self.bufferSizeBytes() / core::mem::size_of::<u32>(),
            )
        };
        unsafe { (*alloc).make_with(|_, _| A8Pass::Make(self.window(), scratch)) }
    }
    fn bufferSizeBytes(&self) -> usize {
        let window = self.window();
        let pass0Size = (window - 1) as usize;
        let pass1Size = (window - 1) as usize;
        let pass2Size = if (window & 1) == 1 {
            window - 1
        } else {
            window
        } as usize;
        (pass0Size + pass1Size + pass2Size) * core::mem::size_of::<u32>()
    }
}

// Source: src/core/SkBlurEngine.cpp:974-982
// Source: src/core/SkBlurEngine.cpp:1027-1027
// Source: src/core/SkBlurEngine.cpp:1031-1033
// Source: src/core/SkBlurEngine.cpp:1035-1035
// Source: src/core/SkBlurEngine.cpp:1037-1038
// Source: src/core/SkBlurEngine.cpp:1041-1053
pub struct A8Pass<'a> {
    fBorder: i32,
    fWeight: u64,
    buffer: &'a mut [u32],
    starts: [usize; 3],
    ends: [usize; 3],
    cursors: [usize; 3],
    fSum0: u32,
    fSum1: u32,
    fSum2: u32,
}
impl<'a> A8Pass<'a> {
    pub fn Make(window: i32, buffers: &'a mut [u32]) -> Self {
        let pass0_size = (window - 1) as usize;
        let pass1_size = (window - 1) as usize;
        let pass2_size = if (window & 1) == 1 {
            window - 1
        } else {
            window
        } as usize;
        let buffer1 = pass0_size;
        let buffer2 = buffer1 + pass1_size;
        let buffer2_end = buffer2 + pass2_size;
        let border = if (window & 1) == 1 {
            3 * ((window - 1) / 2)
        } else {
            3 * (window / 2) - 1
        };
        let window2 = window * window;
        let window3 = window2 * window;
        let divisor = if (window & 1) == 1 {
            window3
        } else {
            window3 + window2
        };
        let weight = (1.0 / divisor as f64 * ((1u64 << 32) as f64)).round() as u64;
        Self {
            fBorder: border,
            fWeight: weight,
            buffer: &mut buffers[..buffer2_end],
            starts: [0, buffer1, buffer2],
            ends: [buffer1, buffer2, buffer2_end],
            cursors: [0, buffer1, buffer2],
            fSum0: 0,
            fSum1: 0,
            fSum2: 0,
        }
    }
}
// Source: src/core/SkBlurEngine.cpp:1056-1059
// Source: src/core/SkBlurEngine.cpp:1061-1061
// Source: src/core/SkBlurEngine.cpp:1063-1065
// Source: src/core/SkBlurEngine.cpp:1068-1071
// Source: src/core/SkBlurEngine.cpp:1073-1074
// Source: src/core/SkBlurEngine.cpp:1077-1082
// Source: src/core/SkBlurEngine.cpp:1084-1085
// Source: src/core/SkBlurEngine.cpp:1087-1087
// Source: src/core/SkBlurEngine.cpp:1089-1094
// Source: src/core/SkBlurEngine.cpp:1096-1096
// Source: src/core/SkBlurEngine.cpp:1099-1101
// Source: src/core/SkBlurEngine.cpp:1103-1106
// Source: src/core/SkBlurEngine.cpp:1108-1111
// Source: src/core/SkBlurEngine.cpp:1113-1117
// Source: src/core/SkBlurEngine.cpp:1122-1127
impl Pass for A8Pass<'_> {
    type Pixel = u8;
    fn border(&self) -> i32 {
        self.fBorder
    }
    fn startBlur(&mut self) {
        self.fSum0 = 0;
        self.fSum1 = 0;
        self.fSum2 = 0;
        self.buffer.fill(0);
        self.cursors = self.starts;
    }
    unsafe fn blurSegment(
        &mut self,
        mut n: i32,
        mut src8: *const u8,
        src_stride: i32,
        mut dst8: *mut u8,
        dst_stride: i32,
    ) {
        if n <= 0 {
            return;
        }
        let mut cursor = self.cursors;
        let (mut sum0, mut sum1, mut sum2) = (self.fSum0, self.fSum1, self.fSum2);
        let weight = self.fWeight;
        let mut process_value = |leading_edge: u32| -> u8 {
            sum0 = sum0.wrapping_add(leading_edge);
            sum1 = sum1.wrapping_add(sum0);
            sum2 = sum2.wrapping_add(sum1);
            let blurred = Self::finalScale(weight, sum2);
            sum2 = sum2.wrapping_sub(self.buffer[cursor[2]]);
            self.buffer[cursor[2]] = sum1;
            cursor[2] = if cursor[2] + 1 < self.ends[2] {
                cursor[2] + 1
            } else {
                self.starts[2]
            };
            sum1 = sum1.wrapping_sub(self.buffer[cursor[1]]);
            self.buffer[cursor[1]] = sum0;
            cursor[1] = if cursor[1] + 1 < self.ends[1] {
                cursor[1] + 1
            } else {
                self.starts[1]
            };
            sum0 = sum0.wrapping_sub(self.buffer[cursor[0]]);
            self.buffer[cursor[0]] = leading_edge;
            cursor[0] = if cursor[0] + 1 < self.ends[0] {
                cursor[0] + 1
            } else {
                self.starts[0]
            };
            blurred
        };
        if src8.is_null() && dst8.is_null() {
            while n > 0 {
                n -= 1;
                let _ = process_value(0);
            }
        } else if !src8.is_null() && dst8.is_null() {
            while n > 0 {
                n -= 1;
                let _ = process_value(unsafe { *src8 } as u32);
                src8 = src8.wrapping_offset(src_stride as isize);
            }
        } else if src8.is_null() && !dst8.is_null() {
            while n > 0 {
                n -= 1;
                unsafe {
                    *dst8 = process_value(0);
                }
                dst8 = dst8.wrapping_offset(dst_stride as isize);
            }
        } else if !src8.is_null() && !dst8.is_null() {
            while n > 0 {
                n -= 1;
                unsafe {
                    *dst8 = process_value(*src8 as u32);
                }
                src8 = src8.wrapping_offset(src_stride as isize);
                dst8 = dst8.wrapping_offset(dst_stride as isize);
            }
        }
        self.cursors = cursor;
        self.fSum0 = sum0;
        self.fSum1 = sum1;
        self.fSum2 = sum2;
    }
}
// Source: src/core/SkBlurEngine.cpp:1130-1130
// Source: src/core/SkBlurEngine.cpp:1132-1133
// Source: src/core/SkBlurEngine.cpp:1138-1144
// Source: src/core/SkBlurEngine.cpp:1146-1151
impl A8Pass<'_> {
    // The source's immutable weight is an explicit argument to avoid borrowing the live scratch closure.
    fn finalScale(weight: u64, sum: u32) -> u8 {
        ((weight.wrapping_mul(sum as u64).wrapping_add(1u64 << 31)) >> 32) as u8
    }
}

// Source: src/core/SkBlurEngine.cpp:1154-1203
pub struct RasterA8BlurAlgorithm;
impl Algorithm for RasterA8BlurAlgorithm {
    fn maxSigma(&self) -> f32 {
        crate::SkASSERT!(@source b"src/core/SkBlurEngine.cpp\0".as_ptr(),1159; @condition b"SkBlurEngine::BoxBlurWindow(kMaxSigma) <= 255\0".as_ptr(); BoxBlurWindow(135.0)<=255);
        135.0
    }
    fn supportsOnlyDecalTiling(&self) -> bool {
        true
    }
    unsafe fn blur(
        &self,
        sigma: crate::include::core::SkSize::SkSize,
        input: std::sync::Arc<dyn crate::src::core::SkSpecialImage::SkSpecialImage>,
        source: &crate::src::core::SkRect::SkIRect,
        tile: crate::include::core::SkTileMode::SkTileMode,
        destination: &crate::src::core::SkRect::SkIRect,
    ) -> Result<
        Option<std::sync::Arc<dyn crate::src::core::SkSpecialImage::SkSpecialImage>>,
        crate::RasterError,
    > {
        crate::SkASSERT!(@source b"src/core/SkBlurEngine.cpp\0".as_ptr(),1174; @condition b"tileMode == SkTileMode::kDecal\0".as_ptr(); tile==crate::include::core::SkTileMode::SkTileMode::kDecal);
        crate::SkASSERT!(@source b"src/core/SkBlurEngine.cpp\0".as_ptr(),1175; @condition b"SkIRect::MakeSize(input->dimensions()).contains(originalSrcBounds)\0".as_ptr(); crate::src::core::SkRect::SkIRect::make_size(input.dimensions()).contains_rect(source));
        let mut src = crate::src::core::SkBitmap::SkBitmap::default();
        if !crate::src::core::SkSpecialImage::AsBitmap(Some(input.as_ref()), Some(&mut src)) {
            return Ok(None);
        }
        crate::SkASSERT!(@source b"src/core/SkBlurEngine.cpp\0".as_ptr(),1182; @condition b"src.colorType() == kAlpha_8_SkColorType\0".as_ptr(); (
            src.colorType())==(
            crate::include::core::SkColorType::SkColorType::kAlpha_8_SkColorType
        ));
        let mut storage = core::pin::pin!(
            crate::src::core::SkArenaAlloc::SkSTArenaAlloc::<1024>::new(0)
        );
        let alloc = storage.as_mut().arena() as *mut _;
        let maker = |value: f32| -> *mut dyn PassMaker<Pixel = u8> {
            crate::SkASSERT!(@source b"src/core/SkBlurEngine.cpp\0".as_ptr(),1187; @condition b"0 <= sigma && sigma <= 135\0".as_ptr(); 0.0<=value&&value<=135.0);
            if let Some(maker) = GaussianPassMaker::<u8, 1>::MakeMaker(value, alloc) {
                return maker;
            }
            if let Some(maker) = A8PassMaker::MakeMaker(value, alloc) {
                return maker;
            }
            crate::SK_ABORT!(@source b"src/core/SkBlurEngine.cpp\0".as_ptr(),1194; "Sigma is out of range.");
        };
        let x = maker(sigma.width());
        let y = maker(sigma.height());
        eval_blur_passes(&*x, &*y, src, source, destination, alloc)
    }
}
// Source: src/core/SkBlurEngine.cpp:1205-1272
pub struct Raster8888BlurAlgorithm;
impl Algorithm for Raster8888BlurAlgorithm {
    fn maxSigma(&self) -> f32 {
        crate::SkASSERT!(@source b"src/core/SkBlurEngine.cpp\0".as_ptr(),1218; @condition b"SkBlurEngine::BoxBlurWindow(kMaxSigma) <= 255\0".as_ptr(); BoxBlurWindow(135.0)<=255);
        135.0
    }
    fn supportsOnlyDecalTiling(&self) -> bool {
        true
    }
    unsafe fn blur(
        &self,
        sigma: crate::include::core::SkSize::SkSize,
        input: std::sync::Arc<dyn crate::src::core::SkSpecialImage::SkSpecialImage>,
        source: &crate::src::core::SkRect::SkIRect,
        tile: crate::include::core::SkTileMode::SkTileMode,
        destination: &crate::src::core::SkRect::SkIRect,
    ) -> Result<
        Option<std::sync::Arc<dyn crate::src::core::SkSpecialImage::SkSpecialImage>>,
        crate::RasterError,
    > {
        use crate::include::core::SkColorType::SkColorType::{
            kBGRA_8888_SkColorType, kRGBA_8888_SkColorType,
        };
        crate::SkASSERT!(@source b"src/core/SkBlurEngine.cpp\0".as_ptr(),1236; @condition b"tileMode == SkTileMode::kDecal\0".as_ptr(); tile==crate::include::core::SkTileMode::SkTileMode::kDecal);
        crate::SkASSERT!(@source b"src/core/SkBlurEngine.cpp\0".as_ptr(),1238; @condition b"SkIRect::MakeSize(input->dimensions()).contains(originalSrcBounds)\0".as_ptr(); crate::src::core::SkRect::SkIRect::make_size(input.dimensions()).contains_rect(source));
        let mut src = crate::src::core::SkBitmap::SkBitmap::default();
        if !crate::src::core::SkSpecialImage::AsBitmap(Some(input.as_ref()), Some(&mut src)) {
            return Ok(None);
        }
        crate::SkASSERT!(@source b"src/core/SkBlurEngine.cpp\0".as_ptr(),1245; @condition b"src.colorType() == kRGBA_8888_SkColorType || src.colorType() == kBGRA_8888_SkColorType\0".as_ptr(); matches!(src.colorType(),kRGBA_8888_SkColorType|kBGRA_8888_SkColorType));
        let mut storage = core::pin::pin!(
            crate::src::core::SkArenaAlloc::SkSTArenaAlloc::<1024>::new(0)
        );
        let alloc = storage.as_mut().arena() as *mut _;
        let maker = |value: f32| -> *mut dyn PassMaker<Pixel = u32> {
            crate::SkASSERT!(@source b"src/core/SkBlurEngine.cpp\0".as_ptr(),1250; @condition b"0 <= sigma && sigma <= 2183\0".as_ptr(); 0.0<=value&&value<=2183.0);
            // Source: src/core/SkBlurEngine.cpp:1251-1255
            if option_env!("SK_AVOID_SLOW_RASTER_PIPELINE_BLURS").is_none() {
                if let Some(maker) = GaussianPassMaker::<u32, 4>::MakeMaker(value, alloc) {
                    return maker;
                }
            }
            if let Some(maker) = ThreeBoxApproxPassMaker::MakeMaker(value, alloc) {
                return maker;
            }
            if let Some(maker) = TentPassMaker::MakeMaker(value, alloc) {
                return maker;
            }
            crate::SK_ABORT!(@source b"src/core/SkBlurEngine.cpp\0".as_ptr(),1262; "Sigma is out of range.");
        };
        let x = maker(sigma.width());
        let y = maker(sigma.height());
        eval_blur_passes(&*x, &*y, src, source, destination, alloc)
    }
}

// Source: src/core/SkBlurEngine.cpp:1274-1282
pub struct RasterShaderBlurAlgorithm;
impl RasterShaderBlurAlgorithm {
    unsafe fn makeDevice(
        &self,
        image_info: &crate::src::core::SkImageInfo::SkImageInfo,
    ) -> crate::include::core::SkRefCnt::sk_sp<dyn crate::src::core::SkDevice::SkDeviceVirtual>
    {
        let mut device = unsafe {
            crate::src::core::SkBitmapDevice::SkBitmapDevice::Create(
                image_info,
                &crate::include::core::SkSurfaceProps::SkSurfaceProps::default(),
                None,
            )
        };
        // The intrusive owner is transferred to its actual virtual device base.
        match device.release() {
            Some(ptr) => unsafe {
                crate::include::core::SkRefCnt::sk_sp::from_raw(
                    ptr.as_ptr() as *mut dyn crate::src::core::SkDevice::SkDeviceVirtual
                )
            },
            None => crate::include::core::SkRefCnt::sk_sp::default(),
        }
    }
}

// Source: src/core/SkBlurEngine.cpp:1284-1312
struct RasterBlurEngine {
    fShaderBlurAlgorithm: RasterShaderBlurAlgorithm,
    fRGBA8BlurAlgorithm: Raster8888BlurAlgorithm,
    fA8BlurAlgorithm: RasterA8BlurAlgorithm,
}
impl SkBlurEngine for RasterBlurEngine {
    fn findAlgorithm(
        &self,
        _sigma: crate::include::core::SkSize::SkSize,
        color_type: crate::include::core::SkColorType::SkColorType,
    ) -> Option<&dyn Algorithm> {
        use crate::include::core::SkColorType::SkColorType::*;
        let rgba8_blur =
            color_type == kRGBA_8888_SkColorType || color_type == kBGRA_8888_SkColorType;
        let a8_blur = color_type == kAlpha_8_SkColorType;
        if a8_blur {
            Some(&self.fA8BlurAlgorithm)
        } else if rgba8_blur {
            Some(&self.fRGBA8BlurAlgorithm)
        } else {
            Some(&self.fShaderBlurAlgorithm)
        }
    }
}

// Source: src/core/SkBlurEngine.cpp:1316-1319
pub fn GetRasterBlurEngine() -> &'static dyn SkBlurEngine {
    static INSTANCE: RasterBlurEngine = RasterBlurEngine {
        fShaderBlurAlgorithm: RasterShaderBlurAlgorithm,
        fRGBA8BlurAlgorithm: Raster8888BlurAlgorithm,
        fA8BlurAlgorithm: RasterA8BlurAlgorithm,
    };
    &INSTANCE
}

// Source: src/core/SkBlurEngine.cpp:1324-1326
// Source: src/core/SkBlurEngine.cpp:1329-1330
// Source: src/core/SkBlurEngine.cpp:1334-1337
// Source: src/core/SkBlurEngine.cpp:1341-1344
// Source: src/core/SkBlurEngine.cpp:1348-1349
// Source: src/core/SkBlurEngine.cpp:1351-1357
// Source: src/core/SkBlurEngine.cpp:1360-1361
// Source: src/core/SkBlurEngine.cpp:1365-1367
// Source: src/core/SkBlurEngine.cpp:1370-1370
pub fn Compute2DBlurKernel(
    sigma: crate::include::core::SkSize::SkSize,
    radius: crate::include::core::SkSize::SkISize,
    kernel: &mut [f32],
) {
    crate::SkASSERT!(@source b"src/core/SkBlurEngine.cpp\0".as_ptr(),1329; @condition b"SkBlurEngine::SigmaToRadius(sigma.width()) == radius.width() && SkBlurEngine::SigmaToRadius(sigma.height()) == radius.height()\0".as_ptr();
        SigmaToRadius(sigma.width()) == radius.width()
            && SigmaToRadius(sigma.height()) == radius.height()
    );
    let width = KernelWidth(radius.width());
    let height = KernelWidth(radius.height());
    let kernel_size = (width as i64 * height as i64) as usize;
    crate::SkASSERT!(@source b"src/core/SkBlurEngine.cpp\0".as_ptr(),1337; @condition b"kernelSize <= kernel.size()\0".as_ptr(); kernel_size <= kernel.len());
    let two_sigma_squared_x = 2.0 * sigma.width() * sigma.width();
    let two_sigma_squared_y = 2.0 * sigma.height() * sigma.height();
    crate::SkASSERT!(@source b"src/core/SkBlurEngine.cpp\0".as_ptr(),1343; @condition b"(radius.width() == 0 || !SkScalarNearlyZero(twoSigmaSqrdX)) && (radius.height() == 0 || !SkScalarNearlyZero(twoSigmaSqrdY))\0".as_ptr(); (radius.width()==0 || !(two_sigma_squared_x.abs()<=1.0/4096.0)) && (radius.height()==0 || !(two_sigma_squared_y.abs()<=1.0/4096.0)));
    let sigma_x_denom = if radius.width() > 0 {
        1.0 / two_sigma_squared_x
    } else {
        1.0
    };
    let sigma_y_denom = if radius.height() > 0 {
        1.0 / two_sigma_squared_y
    } else {
        1.0
    };
    let mut sum = 0.0;
    for x in 0..width {
        let mut x_term = (x - radius.width()) as f32;
        x_term = x_term * x_term * sigma_x_denom;
        for y in 0..height {
            let y_term = (y - radius.height()) as f32;
            let xy_term = (-(x_term + y_term * y_term * sigma_y_denom)).exp();
            kernel[(y * width + x) as usize] = xy_term;
            sum += xy_term;
        }
    }
    let scale = 1.0 / sum;
    for i in 0..kernel_size {
        kernel[i] *= scale;
    }
    kernel[kernel_size..].fill(0.0);
}

// Source: src/core/SkBlurEngine.cpp:1373-1379
pub fn Compute2DBlurKernelPacked(
    sigma: crate::include::core::SkSize::SkSize,
    radii: crate::include::core::SkSize::SkISize,
    kernel: &mut [crate::src::core::SkM44::SkV4; kMaxSamples / 4],
) {
    const {
        assert!(
            core::mem::size_of::<[crate::src::core::SkM44::SkV4; kMaxSamples / 4]>()
                == core::mem::size_of::<[f32; kMaxSamples]>()
        );
        assert!(
            core::mem::align_of::<f32>() == core::mem::align_of::<crate::src::core::SkM44::SkV4>()
        );
    }
    // repr(C) SkV4 has four adjacent f32 fields, matching the source uniform view.
    let data =
        unsafe { core::slice::from_raw_parts_mut(kernel.as_mut_ptr().cast::<f32>(), kMaxSamples) };
    Compute2DBlurKernel(sigma, radii, data);
}

// Source: src/core/SkBlurEngine.cpp:1382-1385
// Source: src/core/SkBlurEngine.cpp:1387-1387
// Source: src/core/SkBlurEngine.cpp:1389-1394
// Source: src/core/SkBlurEngine.cpp:1397-1401
pub fn Compute2DBlurOffsets(
    radius: crate::include::core::SkSize::SkISize,
    offsets: &mut [crate::src::core::SkM44::SkV4; kMaxSamples / 2],
) {
    let kernel_area = KernelWidth(radius.width()) * KernelWidth(radius.height());
    crate::SkASSERT!(@source b"src/core/SkBlurEngine.cpp\0".as_ptr(),1385; @condition b"kernelArea <= kMaxSamples\0".as_ptr(); kernel_area <= kMaxSamples as i32);
    let offset_view = unsafe {
        core::slice::from_raw_parts_mut(offsets.as_mut_ptr().cast::<f32>(), kMaxSamples * 2)
    };
    let mut i = 0usize;
    for y in -radius.height()..=radius.height() {
        for x in -radius.width()..=radius.width() {
            offset_view[2 * i] = x as f32;
            offset_view[2 * i + 1] = y as f32;
            i += 1;
        }
    }
    crate::SkASSERT!(@source b"src/core/SkBlurEngine.cpp\0".as_ptr(),1397; @condition b"i == kernelArea\0".as_ptr(); (i)==( kernel_area as usize));
    let last_valid_offset = 2 * (kernel_area as usize - 1);
    while i < kMaxSamples {
        offset_view[2 * i] = offset_view[last_valid_offset];
        offset_view[2 * i + 1] = offset_view[last_valid_offset + 1];
        i += 1;
    }
}

// Source: src/core/SkBlurEngine.cpp:1405-1411
// Source: src/core/SkBlurEngine.cpp:1419-1421
// Source: src/core/SkBlurEngine.cpp:1426-1429
// Source: src/core/SkBlurEngine.cpp:1431-1432
// Source: src/core/SkBlurEngine.cpp:1434-1436
// Source: src/core/SkBlurEngine.cpp:1441-1442
// Source: src/core/SkBlurEngine.cpp:1451-1459
// Source: src/core/SkBlurEngine.cpp:1462-1463
// Source: src/core/SkBlurEngine.cpp:1465-1465
// Source: src/core/SkBlurEngine.cpp:1468-1470
// Source: src/core/SkBlurEngine.cpp:1473-1474
// Source: src/core/SkBlurEngine.cpp:1478-1478
// Source: src/core/SkBlurEngine.cpp:1481-1482
// Source: src/core/SkBlurEngine.cpp:1486-1487
pub fn Compute1DBlurLinearKernel(
    sigma: f32,
    radius: i32,
    offsets_and_kernel: &mut [crate::src::core::SkM44::SkV4; kMaxSamples / 2],
) {
    use crate::src::core::SkM44::SkV4;
    crate::SkASSERT!(@source b"src/core/SkBlurEngine.cpp\0".as_ptr(),1409; @condition b"sigma <= kMaxLinearSigma\0".as_ptr(); sigma <= kMaxLinearSigma);
    crate::SkASSERT!(@source b"src/core/SkBlurEngine.cpp\0".as_ptr(),1410; @condition b"radius == SkBlurEngine::SigmaToRadius(sigma)\0".as_ptr(); (radius)==( SigmaToRadius(sigma)));
    crate::SkASSERT!(@source b"src/core/SkBlurEngine.cpp\0".as_ptr(),1411; @condition b"LinearKernelWidth(radius) <= kMaxSamples\0".as_ptr(); LinearKernelWidth(radius) <= kMaxSamples as i32);
    let get_new_weight = |wi: f32, wj: f32| (wi + wj, wj / (wi + wj));
    const MAX_KERNEL_WIDTH: usize = KernelWidth(kMaxSamples as i32 - 1) as usize;
    crate::SkASSERT!(@source b"src/core/SkBlurEngine.cpp\0".as_ptr(),1427; @condition b"KernelWidth(radius) <= kMaxKernelWidth\0".as_ptr(); KernelWidth(radius)<=MAX_KERNEL_WIDTH as i32);
    let mut full_kernel = [0f32; MAX_KERNEL_WIDTH];
    Compute1DBlurKernel(
        sigma,
        radius,
        &mut full_kernel[..KernelWidth(radius) as usize],
    );
    let (mut kernel, mut offsets) = ([0f32; kMaxSamples], [0f32; kMaxSamples]);
    let half_size = LinearKernelWidth(radius);
    let half_radius = half_size / 2;
    let mut low_index = half_radius - 1;
    let mut index = radius;
    if (radius & 1) != 0 {
        (kernel[half_radius as usize], offsets[half_radius as usize]) = get_new_weight(
            full_kernel[index as usize] * 0.5,
            full_kernel[index as usize + 1],
        );
        kernel[low_index as usize] = kernel[half_radius as usize];
        offsets[low_index as usize] = -offsets[half_radius as usize];
        index += 1;
        low_index -= 1;
    } else {
        kernel[half_radius as usize] = full_kernel[index as usize];
        offsets[half_radius as usize] = 0.0;
    }
    index += 1;
    for i in half_radius + 1..half_size {
        (kernel[i as usize], offsets[i as usize]) =
            get_new_weight(full_kernel[index as usize], full_kernel[index as usize + 1]);
        offsets[i as usize] += (index - radius) as f32;
        kernel[low_index as usize] = kernel[i as usize];
        offsets[low_index as usize] = -offsets[i as usize];
        index += 2;
        low_index -= 1;
    }
    kernel[half_size as usize..].fill(0.0);
    for i in half_size as usize..kMaxSamples {
        offsets[i] = offsets[half_size as usize - 1];
    }
    for i in 0..kMaxSamples / 2 {
        offsets_and_kernel[i] = SkV4 {
            x: offsets[2 * i],
            y: kernel[2 * i],
            z: offsets[2 * i + 1],
            w: kernel[2 * i + 1],
        };
    }
}

// Source: src/core/SkBlurEngine.cpp:1491-1526
fn to_stablekey(
    kernel_width: i32,
    base_key: u32,
) -> crate::src::core::SkKnownRuntimeEffects::StableKey {
    crate::SkASSERT!(@source b"src/core/SkBlurEngine.cpp\0".as_ptr(),1492; @condition b"kernelWidth >= 2 && kernelWidth <= SkShaderBlurAlgorithm::kMaxSamples\0".as_ptr(); kernel_width>=2&&kernel_width<=kMaxSamples as i32);
    use crate::src::core::SkKnownRuntimeEffects::StableKey;
    match kernel_width {
        2 | 3 | 4 => StableKey(base_key),
        5 | 6 | 7 | 8 => StableKey(base_key + 1),
        9 | 10 | 11 | 12 => StableKey(base_key + 2),
        13 | 14 | 15 | 16 => StableKey(base_key + 3),
        17 | 18 | 19 | 20 => StableKey(base_key + 4),
        21 | 22 | 23 | 24 | 25 | 26 | 27 | 28 => StableKey(base_key + 5),
        _ => crate::SkUNREACHABLE!(@source b"src/core/SkBlurEngine.cpp\0".as_ptr(),1524;),
    }
}

// Source: src/core/SkBlurEngine.cpp:1528-1532
// Source: src/core/SkBlurEngine.h:234-234
pub fn GetLinearBlur1DEffect(
    radius: i32,
) -> Option<&'static std::sync::Arc<crate::src::core::SkRuntimeEffect::SkRuntimeEffect>> {
    crate::src::core::SkKnownRuntimeEffects::GetKnownRuntimeEffect(to_stablekey(
        LinearKernelWidth(radius),
        crate::src::core::SkKnownRuntimeEffects::StableKey::k1DBlurBase.0,
    ))
}
// Source: src/core/SkBlurEngine.cpp:1534-1539
// Source: src/core/SkBlurEngine.h:214-214
pub fn GetBlur2DEffect(
    radii: &crate::include::core::SkSize::SkISize,
) -> Option<&'static std::sync::Arc<crate::src::core::SkRuntimeEffect::SkRuntimeEffect>> {
    let area = KernelWidth(radii.width()) * KernelWidth(radii.height());
    crate::src::core::SkKnownRuntimeEffects::GetKnownRuntimeEffect(to_stablekey(
        area,
        crate::src::core::SkKnownRuntimeEffects::StableKey::k2DBlurBase.0,
    ))
}
impl RasterShaderBlurAlgorithm {
    // Source: src/core/SkBlurEngine.cpp:1541-1630
    // Source: src/core/SkBlurEngine.h:149-155
    unsafe fn renderBlur(
        &self,
        builder: &mut crate::src::core::SkRuntimeEffect::SkRuntimeEffectBuilder,
        filter: crate::include::core::SkSamplingOptions::SkFilterMode,
        radii: crate::include::core::SkSize::SkISize,
        input: std::sync::Arc<dyn crate::src::core::SkSpecialImage::SkSpecialImage>,
        srcRect: &crate::src::core::SkRect::SkIRect,
        tileMode: crate::include::core::SkTileMode::SkTileMode,
        dstRect: &crate::src::core::SkRect::SkIRect,
    ) -> Result<
        Option<std::sync::Arc<dyn crate::src::core::SkSpecialImage::SkSpecialImage>>,
        crate::RasterError,
    > {
        use crate::src::core::{
            SkMatrix::SkMatrix,
            SkRect::{SkIRect, SkRect},
            SkRuntimeEffect::ChildPtr,
        };
        let info = crate::src::core::SkImageInfo::SkImageInfo::MakeDimensionsWithColorSpace(
            dstRect.size(),
            input.colorInfo().colorType(),
            crate::include::core::SkAlphaType::SkAlphaType::kPremul_SkAlphaType,
            input.colorInfo().refColorSpace(),
        );
        let owner = self.makeDevice(&info);
        let Some(mut ptr) = owner.as_ptr() else {
            return Ok(None);
        };
        let device = ptr.as_mut();
        let subset = SkIRect::make_size(dstRect.size());
        device.clipRect(
            &SkRect::Make(&subset),
            crate::include::core::SkClipOp::SkClipOp::kIntersect,
            false,
        );
        device
            .base_mut()
            .set_local_to_device(&crate::src::core::SkM44::SkM44::Translate(
                -(dstRect.left() as f32),
                -(dstRect.top() as f32),
                0.0,
            ));
        let mut paint = crate::src::core::SkPaint::SkPaint::default();
        paint.setBlendMode(crate::src::core::SkBlendMode::SkBlendMode::kSrc);
        let safe = srcRect.make_inset(radii.width(), radii.height());
        let mut fast = *dstRect;
        if *srcRect != SkIRect::make_size(input.backingStoreDimensions()) {
            if fast.intersect(&safe) {
                if fast != *dstRect && fast.width() * fast.height() < 128 * 128 {
                    fast.set_empty();
                }
            } else {
                fast.set_empty();
            }
        }
        let sampling =
            crate::include::core::SkSamplingOptions::SkSamplingOptions::with_filter(filter);
        if !fast.is_empty() {
            let untiled = srcRect.make_inset(1, 1);
            let mode = if untiled.contains_rect(&fast) {
                crate::include::core::SkTileMode::SkTileMode::kClamp
            } else {
                tileMode
            };
            let child = input.asShader(mode, &sampling, SkMatrix::I(), false);
            builder
                .child(b"child")
                .assign(child.map(ChildPtr::Shader).unwrap_or(ChildPtr::Empty));
            paint.setShader(builder.makeShader(None));
            device.drawRect(&SkRect::Make(&fast), &paint)?;
        }
        if fast != *dstRect {
            let subset = input
                .makeSubset(srcRect)
                .expect("native valid blur source subset");
            let child = subset.asShader(
                tileMode,
                &sampling,
                &SkMatrix::Translate(srcRect.left() as f32, srcRect.top() as f32),
                true,
            );
            builder
                .child(b"child")
                .assign(child.map(ChildPtr::Shader).unwrap_or(ChildPtr::Empty));
            paint.setShader(builder.makeShader(None));
        }
        if fast.is_empty() {
            device.drawRect(&SkRect::Make(dstRect), &paint)?;
        } else if fast != *dstRect {
            let borders = [
                SkIRect {
                    fLeft: dstRect.left(),
                    fTop: dstRect.top(),
                    fRight: fast.left(),
                    fBottom: dstRect.bottom(),
                },
                SkIRect {
                    fLeft: fast.right(),
                    fTop: dstRect.top(),
                    fRight: dstRect.right(),
                    fBottom: dstRect.bottom(),
                },
                SkIRect {
                    fLeft: fast.left(),
                    fTop: dstRect.top(),
                    fRight: fast.right(),
                    fBottom: fast.top(),
                },
                SkIRect {
                    fLeft: fast.left(),
                    fTop: fast.bottom(),
                    fRight: fast.right(),
                    fBottom: dstRect.bottom(),
                },
            ];
            for border in borders {
                if !border.is_empty() {
                    device.drawRect(&SkRect::Make(&border), &paint)?;
                }
            }
        }
        device.setImmutable();
        device.snapSpecial(&subset, false)
    }
    // Source: src/core/SkBlurEngine.cpp:1631-1646
    // Source: src/core/SkBlurEngine.h:156-161
    unsafe fn evalBlur2D(
        &self,
        sigma: crate::include::core::SkSize::SkSize,
        radii: crate::include::core::SkSize::SkISize,
        input: std::sync::Arc<dyn crate::src::core::SkSpecialImage::SkSpecialImage>,
        src: &crate::src::core::SkRect::SkIRect,
        tile: crate::include::core::SkTileMode::SkTileMode,
        dst: &crate::src::core::SkRect::SkIRect,
    ) -> Result<
        Option<std::sync::Arc<dyn crate::src::core::SkSpecialImage::SkSpecialImage>>,
        crate::RasterError,
    > {
        let mut kernel = [crate::src::core::SkM44::SkV4 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.0,
        }; kMaxSamples / 4];
        let mut offsets = [crate::src::core::SkM44::SkV4 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.0,
        }; kMaxSamples / 2];
        Compute2DBlurKernelPacked(sigma, radii, &mut kernel);
        Compute2DBlurOffsets(radii, &mut offsets);
        let effect = GetBlur2DEffect(&radii)
            .expect("native known 2D blur effect")
            .clone();
        let mut builder = crate::src::core::SkRuntimeEffect::SkRuntimeEffectBuilder::new(effect);
        builder.uniform(b"kernel").assign(&kernel);
        builder.uniform(b"offsets").assign(&offsets);
        self.renderBlur(
            &mut builder,
            crate::include::core::SkSamplingOptions::SkFilterMode::kNearest,
            radii,
            input,
            src,
            tile,
            dst,
        )
    }
    // Source: src/core/SkBlurEngine.cpp:1647-1669
    // Source: src/core/SkBlurEngine.h:162-168
    unsafe fn evalBlur1D(
        &self,
        sigma: f32,
        radius: i32,
        dir: crate::src::core::SkM44::SkV2,
        input: std::sync::Arc<dyn crate::src::core::SkSpecialImage::SkSpecialImage>,
        src: crate::src::core::SkRect::SkIRect,
        tile: crate::include::core::SkTileMode::SkTileMode,
        dst: crate::src::core::SkRect::SkIRect,
    ) -> Result<
        Option<std::sync::Arc<dyn crate::src::core::SkSpecialImage::SkSpecialImage>>,
        crate::RasterError,
    > {
        let mut values = [crate::src::core::SkM44::SkV4 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.0,
        }; kMaxSamples / 2];
        Compute1DBlurLinearKernel(sigma, radius, &mut values);
        let effect = GetLinearBlur1DEffect(radius)
            .expect("native known linear blur effect")
            .clone();
        let mut builder = crate::src::core::SkRuntimeEffect::SkRuntimeEffectBuilder::new(effect);
        builder.uniform(b"offsetsAndKernel").assign(&values);
        builder.uniform(b"dir").assign(&dir);
        let radii = crate::include::core::SkSize::SkISize::Make(
            if dir.x != 0.0 { radius } else { 0 },
            if dir.y != 0.0 { radius } else { 0 },
        );
        self.renderBlur(
            &mut builder,
            crate::include::core::SkSamplingOptions::SkFilterMode::kLinear,
            radii,
            input,
            &src,
            tile,
            &dst,
        )
    }
}

// Source: src/core/SkBlurEngine.h:136-137
// Source: src/core/SkBlurEngine.cpp:1671-1758
impl Algorithm for RasterShaderBlurAlgorithm {
    fn maxSigma(&self) -> f32 {
        kMaxLinearSigma
    }
    fn supportsOnlyDecalTiling(&self) -> bool {
        false
    }
    unsafe fn blur(
        &self,
        sigma: crate::include::core::SkSize::SkSize,
        mut src: std::sync::Arc<dyn crate::src::core::SkSpecialImage::SkSpecialImage>,
        src_rect: &crate::src::core::SkRect::SkIRect,
        tile_mode: crate::include::core::SkTileMode::SkTileMode,
        dst_rect: &crate::src::core::SkRect::SkIRect,
    ) -> Result<
        Option<std::sync::Arc<dyn crate::src::core::SkSpecialImage::SkSpecialImage>>,
        crate::RasterError,
    > {
        use crate::include::core::SkSize::SkISize;
        use crate::include::core::SkTileMode::SkTileMode;
        use crate::src::core::SkM44::SkV2;
        crate::SkASSERT!(@source b"src/core/SkBlurEngine.cpp\0".as_ptr(),1676; @condition b"sigma.width() <= kMaxLinearSigma && sigma.height() <= kMaxLinearSigma\0".as_ptr(); sigma.width() <= kMaxLinearSigma && sigma.height() <= kMaxLinearSigma);
        let radius_x = SigmaToRadius(sigma.width());
        let radius_y = SigmaToRadius(sigma.height());
        let kernel_area = KernelWidth(radius_x) * KernelWidth(radius_y);
        if kernel_area <= kMaxSamples as i32 && radius_x > 0 && radius_y > 0 {
            unsafe {
                self.evalBlur2D(
                    sigma,
                    SkISize::Make(radius_x, radius_y),
                    src,
                    src_rect,
                    tile_mode,
                    dst_rect,
                )
            }
        } else {
            let mut intermediate_src_rect = *src_rect;
            let mut intermediate_dst_rect = *dst_rect;
            if radius_x > 0 {
                if radius_y > 0 {
                    if tile_mode == SkTileMode::kRepeat || tile_mode == SkTileMode::kMirror {
                        let period = src_rect.height()
                            * if tile_mode == SkTileMode::kMirror {
                                2
                            } else {
                                1
                            };
                        if (dst_rect.fTop - src_rect.fTop).abs() % period != 0
                            || dst_rect.height() != src_rect.height()
                        {
                            intermediate_dst_rect.outset(0, radius_y);
                        }
                    } else {
                        intermediate_dst_rect.outset(0, radius_y);
                        intermediate_dst_rect.fTop = intermediate_dst_rect.fTop.max(src_rect.fTop);
                        intermediate_dst_rect.fBottom =
                            intermediate_dst_rect.fBottom.min(src_rect.fBottom);
                        if intermediate_dst_rect.fTop >= intermediate_dst_rect.fBottom {
                            return Ok(None);
                        }
                    }
                }
                let Some(result) = (unsafe {
                    self.evalBlur1D(
                        sigma.width(),
                        radius_x,
                        SkV2 { x: 1.0, y: 0.0 },
                        src,
                        *src_rect,
                        tile_mode,
                        intermediate_dst_rect,
                    )
                })?
                else {
                    return Ok(None);
                };
                src = result;
                intermediate_src_rect =
                    crate::src::core::SkRect::SkIRect::make_wh(src.width(), src.height());
                intermediate_dst_rect = dst_rect
                    .make_offset(-intermediate_dst_rect.left(), -intermediate_dst_rect.top());
            }
            if radius_y > 0 {
                unsafe {
                    self.evalBlur1D(
                        sigma.height(),
                        radius_y,
                        SkV2 { x: 0.0, y: 1.0 },
                        src,
                        intermediate_src_rect,
                        tile_mode,
                        intermediate_dst_rect,
                    )
                }
            } else {
                Ok(Some(src))
            }
        }
    }
}
